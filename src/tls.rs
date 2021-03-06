use futures01::{try_ready, Async, Future, Poll, Stream};
#[cfg(feature = "sources-tls")]
use native_tls::TlsAcceptor;
use native_tls::{Certificate, Identity, TlsConnectorBuilder};
use openssl::{
    pkcs12::Pkcs12,
    pkey::{PKey, Private},
    x509::X509,
};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
#[cfg(feature = "sources-tls")]
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::tcp::Incoming;
#[cfg(feature = "sources-tls")]
use tokio::net::TcpListener;
use tokio_tls::TlsStream;

#[derive(Debug, Snafu)]
pub enum TlsError {
    #[snafu(display("Could not open {} file {:?}: {}", note, filename, source))]
    FileOpenFailed {
        note: &'static str,
        filename: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not read {} file {:?}: {}", note, filename, source))]
    FileReadFailed {
        note: &'static str,
        filename: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not set TCP TLS identity: {}", source))]
    TlsIdentityError { source: native_tls::Error },
    #[snafu(display("Could not export identity to DER: {}", source))]
    DerExportError { source: openssl::error::ErrorStack },
    #[snafu(display("Could not parse certificate in {:?}: {}", filename, source))]
    CertificateParseError {
        filename: PathBuf,
        source: native_tls::Error,
    },
    #[snafu(display("Must specify both TLS key_file and crt_file"))]
    MissingCrtKeyFile,
    #[snafu(display("Could not parse X509 certificate in {:?}: {}", filename, source))]
    X509ParseError {
        filename: PathBuf,
        source: openssl::error::ErrorStack,
    },
    #[snafu(display("Could not parse private key in {:?}: {}", filename, source))]
    PrivateKeyParseError {
        filename: PathBuf,
        source: openssl::error::ErrorStack,
    },
    #[snafu(display("Could not build PKCS#12 archive for identity: {}", source))]
    Pkcs12Error { source: openssl::error::ErrorStack },
    #[snafu(display("Could not parse identity in {:?}: {}", filename, source))]
    IdentityParseError {
        filename: PathBuf,
        source: native_tls::Error,
    },
    #[snafu(display("TLS configuration requires a certificate when enabled"))]
    MissingRequiredIdentity,
    #[snafu(display("TLS handshake failed: {}", source))]
    Handshake { source: native_tls::Error },
    #[snafu(display("Incoming listener failed: {}", source))]
    IncomingListener { source: crate::Error },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TlsConfig {
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub options: TlsOptions,
}

/// Standard TLS options
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TlsOptions {
    pub verify_certificate: Option<bool>,
    pub verify_hostname: Option<bool>,
    pub ca_path: Option<PathBuf>,
    pub crt_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub key_pass: Option<String>,
}

/// Directly usable settings for TLS connectors
#[derive(Clone, Default)]
pub struct TlsSettings {
    verify_certificate: bool,
    verify_hostname: bool,
    authority: Option<Certificate>,
    identity: Option<IdentityStore>, // native_tls::Identity doesn't implement Clone yet
}

#[derive(Clone)]
pub struct IdentityStore(Vec<u8>, String);

impl TlsSettings {
    pub fn from_config(
        config: &Option<TlsConfig>,
        require_ident: bool,
    ) -> crate::Result<Option<Self>> {
        match config {
            None => Ok(None),
            Some(config) => match config.enabled.unwrap_or(false) {
                false => Ok(None),
                true => {
                    let tls = Self::from_options(&Some(config.options.clone()))?;
                    if require_ident && tls.identity.is_none() {
                        Err(TlsError::MissingRequiredIdentity.into())
                    } else {
                        Ok(Some(tls))
                    }
                }
            },
        }
    }

    pub fn from_options(options: &Option<TlsOptions>) -> crate::Result<Self> {
        let default = TlsOptions::default();
        let options = options.as_ref().unwrap_or(&default);

        if options.verify_certificate == Some(false) {
            warn!("`verify_certificate` is DISABLED, this may lead to security vulnerabilities");
        }
        if options.verify_hostname == Some(false) {
            warn!("`verify_hostname` is DISABLED, this may lead to security vulnerabilities");
        }

        if options.key_path.is_some() && options.crt_path.is_none() {
            return Err(TlsError::MissingCrtKeyFile.into());
        }

        let authority = match options.ca_path {
            None => None,
            Some(ref path) => Some(load_certificate(path)?),
        };

        let identity = match options.crt_path {
            None => None,
            Some(ref crt_path) => {
                let name = crt_path.to_string_lossy().to_string();
                let cert_data = open_read(crt_path, "certificate")?;
                let key_pass: &str = options.key_pass.as_ref().map(|s| s.as_str()).unwrap_or("");

                match Identity::from_pkcs12(&cert_data, key_pass) {
                    Ok(_) => Some(IdentityStore(cert_data, key_pass.to_string())),
                    Err(err) => {
                        if options.key_path.is_none() {
                            return Err(err.into());
                        }
                        let crt = load_x509(crt_path)?;
                        let key_path = options.key_path.as_ref().unwrap();
                        let key = load_key(&key_path, &options.key_pass)?;
                        let pkcs12 = Pkcs12::builder()
                            .build("", &name, &key, &crt)
                            .context(Pkcs12Error)?;
                        let identity = pkcs12.to_der().context(DerExportError)?;

                        // Build the resulting Identity, but don't store it, as
                        // it cannot be cloned.  This is just for error
                        // checking.
                        let _identity =
                            Identity::from_pkcs12(&identity, "").context(TlsIdentityError)?;

                        Some(IdentityStore(identity, "".into()))
                    }
                }
            }
        };

        Ok(Self {
            verify_certificate: options.verify_certificate.unwrap_or(true),
            verify_hostname: options.verify_hostname.unwrap_or(true),
            authority,
            identity,
        })
    }

    pub fn identity(&self) -> Option<Identity> {
        // This data was test-built previously, so we can just use it
        // here and expect the results will not fail. This can all be
        // reworked when `native_tls::Identity` gains the Clone impl.
        self.identity.as_ref().map(|identity| {
            Identity::from_pkcs12(&identity.0, &identity.1).expect("Could not build identity")
        })
    }

    #[cfg(feature = "sources-tls")]
    pub(crate) fn acceptor(&self) -> crate::Result<TlsAcceptor> {
        match self.identity() {
            None => Err(TlsError::MissingRequiredIdentity.into()),
            Some(identity) => TlsAcceptor::new(identity).map_err(Into::into),
        }
    }
}

impl fmt::Debug for TlsSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsSettings")
            .field("verify_certificate", &self.verify_certificate)
            .field("verify_hostname", &self.verify_hostname)
            .finish()
    }
}

pub trait TlsConnectorExt {
    fn use_tls_settings(&mut self, settings: TlsSettings) -> &mut Self;
}

impl TlsConnectorExt for TlsConnectorBuilder {
    fn use_tls_settings(&mut self, settings: TlsSettings) -> &mut Self {
        self.danger_accept_invalid_certs(!settings.verify_certificate);
        self.danger_accept_invalid_hostnames(!settings.verify_hostname);
        settings.identity().map(|identity| self.identity(identity));
        if let Some(certificate) = settings.authority {
            self.add_root_certificate(certificate);
        }
        self
    }
}

/// Load a `native_tls::Certificate` (X.509) from a named file
fn load_certificate(filename: &Path) -> crate::Result<Certificate> {
    let data = open_read(filename, "certificate")?;
    Ok(Certificate::from_der(&data)
        .or_else(|_| Certificate::from_pem(&data))
        .with_context(|| CertificateParseError { filename })?)
}

/// Load a private key from a named file
fn load_key(filename: &Path, pass_phrase: &Option<String>) -> crate::Result<PKey<Private>> {
    let data = open_read(filename, "key")?;
    match pass_phrase {
        None => Ok(PKey::private_key_from_der(&data)
            .or_else(|_| PKey::private_key_from_pem(&data))
            .with_context(|| PrivateKeyParseError { filename })?),
        Some(phrase) => Ok(
            PKey::private_key_from_pkcs8_passphrase(&data, phrase.as_bytes())
                .or_else(|_| PKey::private_key_from_pem_passphrase(&data, phrase.as_bytes()))
                .with_context(|| PrivateKeyParseError { filename })?,
        ),
    }
}

/// Load an X.509 certificate from a named file
fn load_x509(filename: &Path) -> crate::Result<X509> {
    let data = open_read(filename, "certificate")?;
    Ok(X509::from_der(&data)
        .or_else(|_| X509::from_pem(&data))
        .with_context(|| X509ParseError { filename })?)
}

fn open_read(filename: &Path, note: &'static str) -> crate::Result<Vec<u8>> {
    let mut text = Vec::<u8>::new();

    File::open(filename)
        .with_context(|| FileOpenFailed { note, filename })?
        .read_to_end(&mut text)
        .with_context(|| FileReadFailed { note, filename })?;

    Ok(text)
}

pub struct MaybeTlsIncoming<I: Stream> {
    incoming: I,
    acceptor: Option<tokio_tls::TlsAcceptor>,
    state: MaybeTlsIncomingState<I::Item>,
}

enum MaybeTlsIncomingState<S> {
    Inner,
    Accepting(tokio_tls::Accept<S>),
}

impl<I: Stream> MaybeTlsIncoming<I> {
    #[cfg(feature = "sources-tls")]
    pub fn new(incoming: I, tls: Option<TlsSettings>) -> crate::Result<Self> {
        let acceptor = if let Some(tls) = tls {
            let acceptor = tls.acceptor()?;
            Some(acceptor.into())
        } else {
            None
        };

        let state = MaybeTlsIncomingState::Inner;

        Ok(Self {
            incoming,
            acceptor,
            state,
        })
    }
}

impl MaybeTlsIncoming<Incoming> {
    #[cfg(feature = "sources-tls")]
    pub fn bind(addr: &SocketAddr, tls: Option<TlsSettings>) -> crate::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let incoming = listener.incoming();

        MaybeTlsIncoming::new(incoming, tls)
    }
}

impl<I> Stream for MaybeTlsIncoming<I>
where
    I: Stream,
    I::Item: AsyncRead + AsyncWrite,
    I::Error: Into<crate::Error>,
{
    type Item = MaybeTlsStream<I::Item>;
    type Error = TlsError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match &mut self.state {
                MaybeTlsIncomingState::Inner => {
                    let stream = if let Some(stream) = try_ready!(self
                        .incoming
                        .poll()
                        .map_err(Into::into)
                        .context(IncomingListener))
                    {
                        stream
                    } else {
                        return Ok(Async::Ready(None));
                    };

                    if let Some(acceptor) = &mut self.acceptor {
                        let fut = acceptor.accept(stream);

                        self.state = MaybeTlsIncomingState::Accepting(fut);
                        continue;
                    } else {
                        return Ok(Async::Ready(Some(MaybeTlsStream::Raw(stream))));
                    }
                }

                MaybeTlsIncomingState::Accepting(fut) => {
                    let stream = try_ready!(fut.poll().context(Handshake));
                    self.state = MaybeTlsIncomingState::Inner;

                    return Ok(Async::Ready(Some(MaybeTlsStream::Tls(stream))));
                }
            }
        }
    }
}

impl<I: Stream + fmt::Debug> fmt::Debug for MaybeTlsIncoming<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaybeTlsIncoming")
            .field("incoming", &self.incoming)
            .finish()
    }
}

#[derive(Debug)]
pub enum MaybeTlsStream<S> {
    Tls(TlsStream<S>),
    Raw(S),
}

impl<S: Read + Write> Read for MaybeTlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            MaybeTlsStream::Tls(s) => s.read(buf),
            MaybeTlsStream::Raw(s) => s.read(buf),
        }
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncRead for MaybeTlsStream<S> {}

impl<S: Read + Write> Write for MaybeTlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            MaybeTlsStream::Tls(s) => s.write(buf),
            MaybeTlsStream::Raw(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            MaybeTlsStream::Tls(s) => s.flush(),
            MaybeTlsStream::Raw(s) => s.flush(),
        }
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncWrite for MaybeTlsStream<S> {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        match self {
            MaybeTlsStream::Tls(s) => s.shutdown(),
            MaybeTlsStream::Raw(s) => s.shutdown(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_downcast_matches;

    const TEST_PKCS12: &str = "tests/data/localhost.p12";
    const TEST_PEM_CRT: &str = "tests/data/localhost.crt";
    const TEST_PEM_KEY: &str = "tests/data/localhost.key";

    #[test]
    fn from_options_pkcs12() {
        let options = TlsOptions {
            crt_path: Some(TEST_PKCS12.into()),
            key_pass: Some("NOPASS".into()),
            ..Default::default()
        };
        let settings =
            TlsSettings::from_options(&Some(options)).expect("Failed to load PKCS#12 certificate");
        assert!(settings.identity.is_some());
        assert!(settings.authority.is_none());
    }

    #[test]
    fn from_options_pem() {
        let options = TlsOptions {
            crt_path: Some(TEST_PEM_CRT.into()),
            key_path: Some(TEST_PEM_KEY.into()),
            ..Default::default()
        };
        let settings =
            TlsSettings::from_options(&Some(options)).expect("Failed to load PEM certificate");
        assert!(settings.identity.is_some());
        assert!(settings.authority.is_none());
    }

    #[test]
    fn from_options_ca() {
        let options = TlsOptions {
            ca_path: Some("tests/data/Vector_CA.crt".into()),
            ..Default::default()
        };
        let settings = TlsSettings::from_options(&Some(options))
            .expect("Failed to load authority certificate");
        assert!(settings.identity.is_none());
        assert!(settings.authority.is_some());
    }

    #[test]
    fn from_options_none() {
        let settings = TlsSettings::from_options(&None).expect("Failed to generate null settings");
        assert!(settings.identity.is_none());
        assert!(settings.authority.is_none());
    }

    #[test]
    fn from_options_bad_certificate() {
        let options = TlsOptions {
            key_path: Some(TEST_PEM_KEY.into()),
            ..Default::default()
        };
        let error = TlsSettings::from_options(&Some(options))
            .expect_err("from_options failed to check certificate");
        assert_downcast_matches!(error, TlsError, TlsError::MissingCrtKeyFile);

        let options = TlsOptions {
            crt_path: Some(TEST_PEM_CRT.into()),
            ..Default::default()
        };
        let _error = TlsSettings::from_options(&Some(options))
            .expect_err("from_options failed to check certificate");
        // Actual error is an ASN parse, doesn't really matter
    }

    #[test]
    fn from_config_none() {
        assert!(TlsSettings::from_config(&None, true).unwrap().is_none());
        assert!(TlsSettings::from_config(&None, false).unwrap().is_none());
    }

    #[test]
    fn from_config_not_enabled() {
        assert!(settings_from_config(None, false, false, true).is_none());
        assert!(settings_from_config(None, false, false, false).is_none());
        assert!(settings_from_config(Some(false), false, false, true).is_none());
        assert!(settings_from_config(Some(false), false, false, false).is_none());
    }

    #[test]
    fn from_config_fails_without_certificate() {
        let config = make_config(Some(true), false, false);
        let error = TlsSettings::from_config(&Some(config), true)
            .expect_err("from_config failed to check for a certificate");
        assert_downcast_matches!(error, TlsError, TlsError::MissingRequiredIdentity);
    }

    #[test]
    fn from_config_with_certificate() {
        let config = settings_from_config(Some(true), true, true, true);
        assert!(config.is_some());
    }

    fn settings_from_config(
        enabled: Option<bool>,
        set_crt: bool,
        set_key: bool,
        require_ident: bool,
    ) -> Option<TlsSettings> {
        let config = make_config(enabled, set_crt, set_key);
        TlsSettings::from_config(&Some(config), require_ident)
            .expect("Failed to generate settings from config")
    }

    fn make_config(enabled: Option<bool>, set_crt: bool, set_key: bool) -> TlsConfig {
        TlsConfig {
            enabled,
            options: TlsOptions {
                crt_path: and_some(set_crt, TEST_PEM_CRT.into()),
                key_path: and_some(set_key, TEST_PEM_KEY.into()),
                ..Default::default()
            },
        }
    }

    // This can be eliminated once the `bool_to_option` feature migrates
    // out of nightly.
    fn and_some<T>(src: bool, value: T) -> Option<T> {
        match src {
            true => Some(value),
            false => None,
        }
    }
}
