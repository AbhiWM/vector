pub mod watch_client;

use self::watch_client::{ClientConfig, RuntimeError, Version, WatchClient};
use super::Transform;
use crate::{
    event::{self, Event, Value},
    sources::kubernetes::POD_UID,
    topology::config::{DataType, TransformConfig, TransformContext, TransformDescription},
};
use bytes::Bytes;
use evmap::{ReadHandle, WriteHandle};
use futures::stream::Stream;
use futures03::compat::Future01CompatExt;

use k8s_openapi::api::core::v1::Pod;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use string_cache::DefaultAtom as Atom;
use tokio::timer::Delay;

// *********************** Defined by Vector **************************** //
/// Node name `spec.nodeName` of Vector pod passed down with Downward API.
const NODE_NAME_ENV: &str = "VECTOR_NODE_NAME";

/// If watcher errors, for how long will we wait before trying again.
const RETRY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct KubePodMetadata {
    #[serde(default = "default_fields")]
    fields: Vec<String>,
}

inventory::submit! {
    TransformDescription::new_without_default::<KubePodMetadata>("kubernetes_pod_metadata")
}

#[typetag::serde(name = "kubernetes_pod_metadata")]
impl TransformConfig for KubePodMetadata {
    fn build(&self, cx: TransformContext) -> crate::Result<Box<dyn Transform>> {
        // Main idea is to have a background task which will premptively
        // acquire metadata for all pods on this node, and then maintaine that.

        // Construct WatchClient
        let node = std::env::var(NODE_NAME_ENV)
            .map_err(|_| BuildError::MissingNodeName { env: NODE_NAME_ENV })?;
        let wc_config = ClientConfig::in_cluster(node, cx.resolver()).context(WatchClientBuild)?;
        let watch_client = wc_config.build().context(WatchClientBuild)?;

        // Construct MetadataClient
        let (reader, writer) = evmap::new();
        let metadata_client = MetadataClient::new(self, writer, watch_client);

        // Run background task
        cx.executor().spawn_std(async move {
            let error = metadata_client.run().await;
            error!(message = "Stopped updating Pod metadata.", reason = %error);
        });

        // Construct transform
        Ok(Box::new(KubernetesPodMetadata { metadata: reader }))
    }

    fn input_type(&self) -> DataType {
        DataType::Log
    }

    fn output_type(&self) -> DataType {
        DataType::Log
    }

    fn transform_type(&self) -> &'static str {
        "kubernetes_pod_metadata"
    }
}

#[derive(Debug, Snafu)]
enum BuildError {
    #[snafu(display("Failed building WatchClient: {}", source))]
    WatchClientBuild {
        source: self::watch_client::BuildError,
    },
    #[snafu(display("Failed building watch stream: {}", source))]
    WatchStreamBuild {
        source: self::watch_client::BuildError,
    },
    #[snafu(display(
        "Missing environment variable {:?} containing node name `spec.nodeName`.",
        env
    ))]
    MissingNodeName { env: &'static str },
}

/// Background client which watches for Pod metadata changes and writes them to metadata map.
struct MetadataClient {
    fields: Vec<Box<dyn Fn(&Pod) -> Vec<(Atom, Value)> + Send + Sync + 'static>>,
    metadata: WriteHandle<Bytes, Box<(Atom, FieldValue)>>,
    client: WatchClient,
}

impl MetadataClient {
    fn new(
        trans_config: &KubePodMetadata,
        metadata: WriteHandle<Bytes, Box<(Atom, FieldValue)>>,
        client: WatchClient,
    ) -> Self {
        // Select Pod metadata fields which are extracted and then added to Events.
        let fields = all_fields()
            .into_iter()
            .filter(|(key, _)| {
                trans_config
                    .fields
                    .iter()
                    .any(|field| field.as_str() == *key)
            })
            .map(|(_, fun)| fun)
            .collect();

        Self {
            fields,
            metadata,
            client,
        }
    }

    /// Listens for pod metadata changes and updates metadata map.
    async fn run(mut self) -> BuildError {
        let mut version = None;
        let mut error = None;
        loop {
            let mut watcher = match self.client.watch_metadata(version.clone(), error.take()) {
                Ok(watcher) => watcher,
                Err(error) => return BuildError::WatchStreamBuild { source: error },
            };
            info!("Watching Pod metadata.");

            let runtime_error = loop {
                break match watcher.into_future().compat().await {
                    Ok((Some(pod), tail)) => {
                        watcher = tail;
                        version = self.update(&pod).or(version);
                        continue;
                    }
                    Ok((None, _)) => RuntimeError::WatchUnexpectedlyEnded,
                    Err((err, _)) => err,
                };
            };

            warn!(
                message = "Temporary stoped watching Pod metadata.",
                reason = %runtime_error
            );

            error = Some(runtime_error);

            // Wait for bit before trying to watch again.
            let _ = Delay::new(Instant::now() + RETRY_TIMEOUT)
                .compat()
                .await
                .expect("Timer not set.");
        }
    }

    // In the case of Deleted, we don't delete it's data, as there could still exist unprocessed logs from that pod.
    // Not deleting it will cause "memory leakage" in a sense that the data won't be used ever
    // again after some point, but the catch is that we don't know when that point is.
    // Also considering that, on average, an entry occupies ~232B, so to 'leak' 1MB of memory, ~4500 pods would need to be
    // created and destroyed on the same node, which is highly unlikely.
    //
    // An alternative would be to delay deletions of entrys by 1min. Which is a safe guess.
    //
    /// Extracts metadata from pod and updates metadata map.
    fn update(&mut self, pod: &Pod) -> Option<Version> {
        if let Some(pod_uid) = pod.metadata.as_ref().and_then(|md| md.uid.as_ref()) {
            let uid: Bytes = pod_uid.as_str().into();

            self.metadata.clear(uid.clone());

            // Insert field values for this pod.
            for (field, value) in self.fields.iter().flat_map(|fun| fun(pod)) {
                self.metadata
                    .insert(uid.clone(), Box::new((field, FieldValue(value))));
            }

            self.metadata.refresh();

            trace!(message = "Pod updated.", %pod_uid);
        }

        Version::from_pod(pod)
    }
}

#[derive(PartialEq, Debug, Clone)]
struct FieldValue(Value);

// Since we aren't using Eq feature in the evmap, we can add it.
impl Eq for FieldValue {}

pub struct KubernetesPodMetadata {
    metadata: ReadHandle<Bytes, Box<(Atom, FieldValue)>>,
}

impl Transform for KubernetesPodMetadata {
    fn transform(&mut self, mut event: Event) -> Option<Event> {
        let log = event.as_mut_log();

        if let Some(Value::Bytes(pod_uid)) = log.get(&POD_UID) {
            let pod_uid = pod_uid.clone();
            if self
                .metadata
                .get_and(&pod_uid, |fields| {
                    for pair in fields {
                        log.insert(pair.0.clone(), (pair.1).0.clone());
                    }
                })
                .is_none()
            {
                warn!(
                    message = "Metadata for pod not yet available.",
                    pod_uid = ?std::str::from_utf8(pod_uid.as_ref()),
                    rate_limit_secs = 10
                );
            }
        } else {
            warn!(
                message = "Event without field, so it can't be enriched with metadata.",
                field = POD_UID.as_ref(),
                rate_limit_secs = 10
            );
        }

        Some(event)
    }
}

fn default_fields() -> Vec<String> {
    vec!["name", "namespace", "labels", "annotations", "node_name"]
        .into_iter()
        .map(Into::into)
        .collect()
}

/// Returns list of all supported fields and their extraction function.
fn all_fields() -> Vec<(
    &'static str,
    Box<dyn Fn(&Pod) -> Vec<(Atom, Value)> + Send + Sync + 'static>,
)> {
    vec![
        // ------------------------ ObjectMeta ------------------------ //
        field("name", |pod| pod.metadata.as_ref()?.name.clone()),
        field("namespace", |pod| pod.metadata.as_ref()?.namespace.clone()),
        field("creation_timestamp", |pod| {
            pod.metadata
                .as_ref()?
                .creation_timestamp
                .clone()
                .map(|time| time.0)
        }),
        field("deletion_timestamp", |pod| {
            pod.metadata
                .as_ref()?
                .deletion_timestamp
                .clone()
                .map(|time| time.0)
        }),
        collection_field("labels", |pod| pod.metadata.as_ref()?.labels.as_ref()),
        collection_field("annotations", |pod| {
            pod.metadata.as_ref()?.annotations.as_ref()
        }),
        // ------------------------ PodSpec ------------------------ //
        field("node_name", |pod| pod.spec.as_ref()?.node_name.clone()),
        field("hostname", |pod| pod.spec.as_ref()?.hostname.clone()),
        field("priority", |pod| pod.spec.as_ref()?.priority),
        field("priority_class_name", |pod| {
            pod.spec.as_ref()?.priority_class_name.clone()
        }),
        field("service_account_name", |pod| {
            pod.spec.as_ref()?.service_account_name.clone()
        }),
        field("subdomain", |pod| pod.spec.as_ref()?.subdomain.clone()),
        // ------------------------ PodStatus ------------------------ //
        field("host_ip", |pod| pod.status.as_ref()?.host_ip.clone()),
        field("ip", |pod| pod.status.as_ref()?.pod_ip.clone()),
    ]
}

fn field<T: Into<Value>>(
    name: &'static str,
    fun: impl Fn(&Pod) -> Option<T> + Send + Sync + 'static,
) -> (
    &'static str,
    Box<dyn Fn(&Pod) -> Vec<(Atom, Value)> + Send + Sync + 'static>,
) {
    let key: Atom = with_prefix(name).into();
    let fun = move |pod: &Pod| {
        fun(pod)
            .map(|data| vec![(key.clone(), data.into())])
            .unwrap_or_default()
    };
    (name, Box::new(fun) as Box<_>)
}

fn collection_field(
    name: &'static str,
    fun: impl Fn(&Pod) -> Option<&BTreeMap<String, String>> + Send + Sync + 'static,
) -> (
    &'static str,
    Box<dyn Fn(&Pod) -> Vec<(Atom, Value)> + Send + Sync + 'static>,
) {
    let prefix_key = with_prefix(name) + ".";
    let fun = move |pod: &Pod| {
        fun(pod)
            .map(|map| {
                map.iter()
                    .map(|(key, value)| ((prefix_key.clone() + key).into(), value.into()))
                    .collect()
            })
            .unwrap_or_default()
    };
    (name, Box::new(fun) as Box<_>)
}

fn with_prefix(name: &str) -> String {
    event::log_schema()
        .kubernetes_key()
        .to_owned()
        .as_ref()
        .to_owned()
        + "."
        + name
}

#[cfg(test)]
mod tests {
    #![cfg(feature = "kubernetes-integration-tests")]

    use crate::sources::kubernetes::test::{echo, logs, user_namespace, Kube, VECTOR_YAML};
    use kube::api::{Api, RawApi};
    use uuid::Uuid;

    static NAME_MARKER: &'static str = "$(NAME)";
    static FIELD_MARKER: &'static str = "$(FIELD)";

    static ROLE_BINDING_YAML: &'static str = r#"
# Permissions to use Kubernetes API.
# Necessary for kubernetes_pod_metadata transform.
# Requires that RBAC authorization is enabled.
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: $(NAME)
subjects:
- kind: ServiceAccount
  name: default
  namespace: $(TEST_NAMESPACE)
roleRef:
  kind: ClusterRole
  name: view
  apiGroup: rbac.authorization.k8s.io
"#;

    static CONFIG_MAP_YAML_WITH_METADATA: &'static str = r#"
# ConfigMap which contains vector.toml configuration for pods.
apiVersion: v1
kind: ConfigMap
metadata:
  name: vector-config
  namespace: $(TEST_NAMESPACE)
data:
  vector-agent-config: |
    # VECTOR.TOML
    # Configuration for vector-agent

    # Set global options
    data_dir = "/tmp/vector/"

    # Ingest logs from Kubernetes
    [sources.kubernetes_logs]
      type = "kubernetes"

    [transforms.kube_metadata]
      type = "kubernetes_pod_metadata"
      inputs = ["kubernetes_logs"]
      $(FIELD)

    [sinks.out]
      type = "console"
      inputs = ["kube_metadata"]
      target = "stdout"

      encoding = "json"
      healthcheck = true

  # This line is not in VECTOR.TOML
"#;

    fn cluster_role_binding_api() -> RawApi {
        RawApi {
            group: "rbac.authorization.k8s.io".into(),
            resource: "clusterrolebindings".into(),
            prefix: "apis".into(),
            version: "v1".into(),
            ..Default::default()
        }
    }

    fn binding_name(namespace: &str) -> String {
        "binding-".to_owned() + namespace
    }

    fn metadata_config_map(fields: Option<Vec<&str>>) -> String {
        let replace = if let Some(fields) = fields {
            format!(
                "fields = [{}]",
                fields
                    .iter()
                    .map(|field| format!("{:?}", field))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            "".to_owned()
        };

        CONFIG_MAP_YAML_WITH_METADATA.replace(FIELD_MARKER, replace.as_str())
    }

    #[test]
    fn kube_metadata() {
        let namespace = format!("kube-metadata-{}", Uuid::new_v4());
        let message = "20";
        let field = "node_name";
        let user_namespace = user_namespace(namespace.as_str());
        let binding_name = binding_name(namespace.as_str());

        let kube = Kube::new(namespace.as_str());
        let user = Kube::new(user_namespace.clone().as_str());

        // Cluster role binding
        kube.create_raw_with::<k8s_openapi::api::rbac::v1::ClusterRoleBinding>(
            &cluster_role_binding_api(),
            ROLE_BINDING_YAML
                .replace(NAME_MARKER, binding_name.as_str())
                .as_str(),
        );
        let _binding = kube.deleter(cluster_role_binding_api(), binding_name.as_str());

        // Start vector
        kube.create(
            Api::v1ConfigMap,
            metadata_config_map(Some(vec![field])).as_str(),
        );
        let vector = kube.create(Api::v1DaemonSet, VECTOR_YAML);

        // Wait for running state
        kube.wait_for_running(vector.clone());

        // Start echo
        let _echo = echo(&user, "echo", message);

        // Verify logs
        // If any daemon logged message, done.
        for line in logs(&kube, &vector) {
            if line.get(super::with_prefix(field)).is_some() {
                // DONE
                return;
            } else {
                debug!(namespace=namespace.as_str(),log=%line);
            }
        }
        panic!("Vector didn't find field: {:?}", field);
    }
}
