---
title: Config Specification
sidebar_label: Specification
description: Full Vector config specification
---

Below is a full config specification. Note, this file is included with
Vector package installs, generally located at `/etc/vector/vector.spec.yml`:

import CodeHeader from '@site/src/components/CodeHeader';

<CodeHeader fileName="vector.toml" />

```toml
#                                    __   __  __  
#                                    \ \ / / / /
#                                     \ V / / /
#                                      \_/  \/
#
#                                    V E C T O R
#                            Configuration Specification
#
# ------------------------------------------------------------------------------
# Website: https://vector.dev
# Docs: https://docs.vector.dev
# Community: https://vector.dev/community
# ------------------------------------------------------------------------------
# The file contains a full specification for the `vector.toml` configuration
# file. It follows the TOML format and includes all options, types, and
# possible values.
#
# More info on Vector's configuration can be found at:
# /docs/setup/configuration

# ------------------------------------------------------------------------------
# Global
# ------------------------------------------------------------------------------
# Global options are relevant to Vector as a whole and apply to global behavior.

# The directory used for persisting Vector state, such as on-disk buffers, file
# checkpoints, and more. Please make sure the Vector project has write
# permissions to this dir.
# 
# * optional
# * no default
# * type: string
data_dir = "/var/lib/vector"

# ------------------------------------------------------------------------------
# Sources
# ------------------------------------------------------------------------------
# Sources specify data sources and are responsible for ingesting data into
# Vector.

# Ingests data through the docker engine daemon and outputs `log` events.
[sources.docker]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `docker`.
  # 
  # * required
  # * type: string
  # * must be: "docker"
  type = "docker"

  # A list of container ids to match against when filtering running containers.
  # This will attempt to match the container id from the beginning meaning you do
  # not need to include the whole id but just the first few characters. If no
  # containers ids are provided, all containers will be included.
  # 
  # * optional
  # * no default
  # * type: [string]
  include_containers = "ffd2bc2cb74a"

  #  A list of container object labels to match against when filtering running
  # containers. This should follow the described label's synatx in docker object
  # labels docs.
  # 
  # * optional
  # * no default
  # * type: [string]
  include_labels = "label_key=label_value"

# Ingests data through one or more local files and outputs `log` events.
[sources.file]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `file`.
  # 
  # * required
  # * type: string
  # * must be: "file"
  type = "file"

  # Array of file patterns to include. Globbing is supported.
  # 
  # * required
  # * type: [string]
  include = ["/var/log/nginx/*.log"]

  # The directory used to persist file checkpoint positions. By default, the
  # global `data_dir` option is used. Please make sure the Vector project has
  # write permissions to this dir.
  # 
  # * optional
  # * no default
  # * type: string
  data_dir = "/var/lib/vector"

  # Array of file patterns to exclude. Globbing is supported. *Takes precedence
  # over the `include` option.*
  # 
  # * optional
  # * no default
  # * type: [string]
  exclude = ["/var/log/nginx/*.[0-9]*.log"]

  # Delay between file discovery calls. This controls the interval at which
  # Vector searches for files.
  # 
  # * optional
  # * default: 1000
  # * type: int
  # * unit: milliseconds
  glob_minimum_cooldown = 1000

  # Ignore files with a data modification date that does not exceed this age.
  # 
  # * optional
  # * no default
  # * type: int
  # * unit: seconds
  ignore_older = 86400

  # The maximum number of a bytes a line can contain before being discarded. This
  # protects against malformed lines or tailing incorrect files.
  # 
  # * optional
  # * default: 102400
  # * type: int
  # * unit: bytes
  max_line_bytes = 102400

  # When `true` Vector will read from the beginning of new files, when `false`
  # Vector will only read new data added to the file.
  # 
  # * optional
  # * default: false
  # * type: bool
  start_at_beginning = true
  start_at_beginning = false

  #
  # Context
  #

  # The key name added to each event with the full path of the file.
  # 
  # * optional
  # * default: "file"
  # * type: string
  file_key = "file"

  # The key name added to each event representing the current host.
  # 
  # * optional
  # * default: "host"
  # * type: string
  host_key = "host"

  #
  # Multi-line
  #

  # When present, Vector will aggregate multiple lines into a single event, using
  # this pattern as the indicator that the previous lines should be flushed and a
  # new event started. The pattern will be matched against entire lines as a
  # regular expression, so remember to anchor as appropriate.
  # 
  # * optional
  # * no default
  # * type: string
  message_start_indicator = "^(INFO|ERROR)"

  # When `message_start_indicator` is present, this sets the amount of time
  # Vector will buffer lines into a single event before flushing, regardless of
  # whether or not it has seen a line indicating the start of a new message.
  # 
  # * optional
  # * default: 1000
  # * type: int
  # * unit: milliseconds
  multi_line_timeout = 1000

  #
  # Priority
  #

  # An approximate limit on the amount of data read from a single file at a given
  # time.
  # 
  # * optional
  # * default: 2048
  # * type: int
  # * unit: bytes
  max_read_bytes = 2048

  # Instead of balancing read capacity fairly across all watched files,
  # prioritize draining the oldest files before moving on to read data from
  # younger files.
  # 
  # * optional
  # * default: false
  # * type: bool
  oldest_first = true
  oldest_first = false

  #
  # Fingerprinting
  #

  [sources.file.fingerprinting]
    # The strategy used to uniquely identify files. This is important for
    # checkpointing when file rotation is used.
    # 
    # * optional
    # * default: "checksum"
    # * type: string
    # * enum: "checksum" or "device_and_inode"
    strategy = "checksum"
    strategy = "device_and_inode"

    # The number of bytes read off the head of the file to generate a unique
    # fingerprint.
    # 
    # * optional
    # * default: 256
    # * type: int
    # * unit: bytes
    # * relevant when strategy = "checksum"
    fingerprint_bytes = 256

    # The number of bytes to skip ahead (or ignore) when generating a unique
    # fingerprint. This is helpful if all files share a common header.
    # 
    # * optional
    # * default: 0
    # * type: int
    # * unit: bytes
    # * relevant when strategy = "checksum"
    ignored_header_bytes = 0

# Ingests data through log records from journald and outputs `log` events.
[sources.journald]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `journald`.
  # 
  # * required
  # * type: string
  # * must be: "journald"
  type = "journald"

  # The systemd journal is read in batches, and a checkpoint is set at the end of
  # each batch. This option limits the size of the batch.
  # 
  # * optional
  # * default: 16
  # * type: int
  batch_size = 16

  # Include only entries from the current boot.
  # 
  # * optional
  # * default: true
  # * type: bool
  current_boot_only = true
  current_boot_only = false

  # The directory used to persist the journal checkpoint position. By default,
  # the global `data_dir` is used. Please make sure the Vector project has write
  # permissions to this dir.
  # 
  # * optional
  # * no default
  # * type: string
  data_dir = "/var/lib/vector"

  # Include only entries from the local system
  # 
  # * optional
  # * default: true
  # * type: bool
  local_only = true
  local_only = false

  # The list of units names to monitor. If empty or not present, all units are
  # accepted. Unit names lacking a `"."` will have `".service"` appended to make
  # them a valid service unit name.
  # 
  # * optional
  # * default: []
  # * type: [string]
  units = ["ntpd", "sysinit.target"]

# Ingests data through Kafka 0.9 or later and outputs `log` events.
[sources.kafka]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `kafka`.
  # 
  # * required
  # * type: string
  # * must be: "kafka"
  type = "kafka"

  # A comma-separated list of host and port pairs that are the addresses of the
  # Kafka brokers in a "bootstrap" Kafka cluster that a Kafka client connects to
  # initially to bootstrap itself.
  # 
  # * required
  # * type: string
  bootstrap_servers = "10.14.22.123:9092,10.14.23.332:9092"

  # The consumer group name to be used to consume events from Kafka.
  # 
  # * required
  # * type: string
  group_id = "consumer-group-name"

  # The Kafka topics names to read events from. Regex is supported if the topic
  # begins with `^`.
  # 
  # * required
  # * type: [string]
  topics = ["^(prefix1|prefix2)-.+", "topic-1", "topic-2"]

  # If offsets for consumer group do not exist, set them using this strategy.
  # librdkafka documentation for `auto.offset.reset` option for explanation.
  # 
  # * optional
  # * default: "largest"
  # * type: string
  auto_offset_reset = "smallest"
  auto_offset_reset = "earliest"
  auto_offset_reset = "beginning"
  auto_offset_reset = "largest"
  auto_offset_reset = "latest"
  auto_offset_reset = "end"
  auto_offset_reset = "error"

  # The log field name to use for the topic key. If unspecified, the key would
  # not be added to the log event. If the message has null key, then this field
  # would not be added to the log event.
  # 
  # * optional
  # * no default
  # * type: string
  key_field = "user_id"

  # The Kafka session timeout in milliseconds.
  # 
  # * optional
  # * default: 10000
  # * type: int
  # * unit: milliseconds
  session_timeout_ms = 5000
  session_timeout_ms = 10000

# Ingests data through the StatsD UDP protocol and outputs `metric` events.
[sources.statsd]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `statsd`.
  # 
  # * required
  # * type: string
  # * must be: "statsd"
  type = "statsd"

  # UDP socket address to bind to.
  # 
  # * required
  # * type: string
  address = "127.0.0.1:8126"

# Ingests data through standard input (STDIN) and outputs `log` events.
[sources.stdin]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `stdin`.
  # 
  # * required
  # * type: string
  # * must be: "stdin"
  type = "stdin"

  # The maxiumum bytes size of a message before it is discarded.
  # 
  # * optional
  # * default: 102400
  # * type: int
  # * unit: bytes
  max_length = 102400

  #
  # Context
  #

  # The key name added to each event representing the current host.
  # 
  # * optional
  # * default: "host"
  # * type: string
  host_key = "host"

# Ingests data through the Syslog 5424 protocol and outputs `log` events.
[sources.syslog]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `syslog`.
  # 
  # * required
  # * type: string
  # * must be: "syslog"
  type = "syslog"

  # The input mode.
  # 
  # * required
  # * type: string
  # * enum: "tcp", "udp", and "unix"
  mode = "tcp"
  mode = "udp"
  mode = "unix"

  # The TCP or UDP address to listen for connections on, or "systemd#N" to use
  # the Nth socket passed by systemd socket activation.
  # 
  # * optional
  # * no default
  # * type: string
  # * relevant when mode = "tcp" or mode = "udp"
  address = "0.0.0.0:9000"
  address = "systemd"
  address = "systemd#2"

  # The maximum bytes size of incoming messages before they are discarded.
  # 
  # * optional
  # * default: 102400
  # * type: int
  # * unit: bytes
  max_length = 102400

  # The unix socket path. *This should be absolute path.*
  # 
  # * optional
  # * no default
  # * type: string
  # * relevant when mode = "unix"
  path = "/path/to/socket"

  #
  # Context
  #

  # The key name added to each event representing the current host.
  # 
  # * optional
  # * default: "host"
  # * type: string
  host_key = "host"

# Ingests data through the TCP protocol and outputs `log` events.
[sources.tcp]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `tcp`.
  # 
  # * required
  # * type: string
  # * must be: "tcp"
  type = "tcp"

  # The address to listen for connections on, or "systemd#N" to use the Nth
  # socket passed by systemd socket activation.
  # 
  # * required
  # * type: string
  address = "0.0.0.0:9000"
  address = "systemd"
  address = "systemd#3"

  # The maximum bytes size of incoming messages before they are discarded.
  # 
  # * optional
  # * default: 102400
  # * type: int
  # * unit: bytes
  max_length = 102400

  # The timeout before a connection is forcefully closed during shutdown.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  shutdown_timeout_secs = 30

  #
  # Context
  #

  # The key name added to each event representing the current host.
  # 
  # * optional
  # * default: "host"
  # * type: string
  host_key = "host"

# Ingests data through the UDP protocol and outputs `log` events.
[sources.udp]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `udp`.
  # 
  # * required
  # * type: string
  # * must be: "udp"
  type = "udp"

  # The address to bind the socket to.
  # 
  # * required
  # * type: string
  address = "0.0.0.0:9000"

  # The maximum bytes size of incoming messages before they are discarded.
  # 
  # * optional
  # * default: 102400
  # * type: int
  # * unit: bytes
  max_length = 102400

  #
  # Context
  #

  # The key name added to each event representing the current host.
  # 
  # * optional
  # * default: "host"
  # * type: string
  host_key = "host"

# Ingests data through another upstream `vector` sink and outputs `log` and `metric` events.
[sources.vector]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `vector`.
  # 
  # * required
  # * type: string
  # * must be: "vector"
  type = "vector"

  # The TCP address to listen for connections on, or "systemd#N" to use the Nth
  # socket passed by systemd socket activation.
  # 
  # * required
  # * type: string
  address = "0.0.0.0:9000"
  address = "systemd"
  address = "systemd#1"

  # The timeout before a connection is forcefully closed during shutdown.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  shutdown_timeout_secs = 30


# ------------------------------------------------------------------------------
# Transforms
# ------------------------------------------------------------------------------
# Transforms parse, structure, and enrich events.

# Accepts `log` events and allows you to add one or more log fields.
[transforms.add_fields]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `add_fields`.
  # 
  # * required
  # * type: string
  # * must be: "add_fields"
  type = "add_fields"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  #
  # Fields
  #

  [transforms.add_fields.fields]
    # A key/value pair representing the new log fields to be added. Accepts all
    # supported types. Use `.` for adding nested fields.
    # 
    # * required
    # * type: *
    my_string_field = "string value"
    my_env_var_field = "${ENV_VAR}"
    my_int_field = 1
    my_float_field = 1.2
    my_bool_field = true
    my_timestamp_field = 1979-05-27T00:32:00Z
    my_nested_fields = {key1 = "value1", key2 = "value2"}
    my_list = ["first", "second", "third"]

# Accepts `metric` events and allows you to add one or more metric tags.
[transforms.add_tags]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `add_tags`.
  # 
  # * required
  # * type: string
  # * must be: "add_tags"
  type = "add_tags"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  #
  # Tags
  #

  [transforms.add_tags.tags]
    # A key/value pair representing the new tag to be added.
    # 
    # * required
    # * type: *
    my_tag = "my value"
    my_env_tag = "${ENV_VAR}"

# Accepts `log` events and allows you to coerce log fields into fixed types.
[transforms.coercer]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `coercer`.
  # 
  # * required
  # * type: string
  # * must be: "coercer"
  type = "coercer"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  #
  # Types
  #

  [transforms.coercer.types]
    # A definition of log field type conversions. They key is the log field name
    # and the value is the type. `strptime` specifiers are supported for the
    # `timestamp` type.
    # 
    # * required
    # * type: string
    # * enum: "bool", "float", "int", "string", and "timestamp"
    status = "int"
    duration = "float"
    success = "bool"
    timestamp = "timestamp|%s"
    timestamp = "timestamp|%+"
    timestamp = "timestamp|%F"
    timestamp = "timestamp|%a %b %e %T %Y"

# Accepts `log` and `metric` events and allows you to filter events by a log field's value.
[transforms.field_filter]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `field_filter`.
  # 
  # * required
  # * type: string
  # * must be: "field_filter"
  type = "field_filter"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The target log field to compare against the `value`.
  # 
  # * required
  # * type: string
  field = "file"

  # If the value of the specified `field` matches this value then the event will
  # be permitted, otherwise it is dropped.
  # 
  # * required
  # * type: string
  value = "/var/log/nginx.log"

# Accepts `log` events and allows you to parse a log field value with Grok.
[transforms.grok_parser]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `grok_parser`.
  # 
  # * required
  # * type: string
  # * must be: "grok_parser"
  type = "grok_parser"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The Grok pattern
  # 
  # * required
  # * type: string
  pattern = "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"

  # If `true` will drop the specified `field` after parsing.
  # 
  # * optional
  # * default: true
  # * type: bool
  drop_field = true
  drop_field = false

  # The log field to execute the `pattern` against. Must be a `string` value.
  # 
  # * optional
  # * default: "message"
  # * type: string
  field = "message"

  #
  # Types
  #

  [transforms.grok_parser.types]
    # A definition of log field type conversions. They key is the log field name
    # and the value is the type. `strptime` specifiers are supported for the
    # `timestamp` type.
    # 
    # * required
    # * type: string
    # * enum: "bool", "float", "int", "string", and "timestamp"
    status = "int"
    duration = "float"
    success = "bool"
    timestamp = "timestamp|%s"
    timestamp = "timestamp|%+"
    timestamp = "timestamp|%F"
    timestamp = "timestamp|%a %b %e %T %Y"

# Accepts `log` events and allows you to parse a log field value as JSON.
[transforms.json_parser]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `json_parser`.
  # 
  # * required
  # * type: string
  # * must be: "json_parser"
  type = "json_parser"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # If `true` events with invalid JSON will be dropped, otherwise the event will
  # be kept and passed through.
  # 
  # * required
  # * type: bool
  drop_invalid = true

  # The log field to decode as JSON. Must be a `string` value type.
  # 
  # * optional
  # * default: "message"
  # * type: string
  field = "message"

  # If `target_field` is set and the log contains a field of the same name as the
  # target, it will only be overwritten if this is set to `true`.
  # 
  # * optional
  # * default: "false"
  # * type: bool
  overwrite_target = true
  overwrite_target = false

  # If this setting is present, the parsed JSON will be inserted into the log as
  # a sub-object with this name. If a field with the same name already exists,
  # the parser will fail and produce an error.
  # 
  # * optional
  # * no default
  # * type: string
  target_field = "target"

# Accepts `log` events and allows you to convert logs into one or more metrics.
[transforms.log_to_metric]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `log_to_metric`.
  # 
  # * required
  # * type: string
  # * must be: "log_to_metric"
  type = "log_to_metric"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  #
  # Metrics
  #

  [[transforms.log_to_metric.metrics]]
    # The metric type.
    # 
    # * required
    # * type: string
    # * enum: "counter", "gauge", "histogram", and "set"
    type = "counter"
    type = "gauge"
    type = "histogram"
    type = "set"

    # The log field to use as the metric.
    # 
    # * required
    # * type: string
    field = "duration"

    # The name of the metric. Defaults to `<field>_total` for `counter` and
    # `<field>` for `gauge`.
    # 
    # * required
    # * type: string
    name = "duration_total"

    # If `true` the metric will be incremented by the `field` value. If `false` the
    # metric will be incremented by 1 regardless of the `field` value.
    # 
    # * optional
    # * default: false
    # * type: bool
    # * relevant when type = "counter"
    increment_by_value = true
    increment_by_value = false

    [transforms.log_to_metric.metrics.tags]
      # Key/value pairs representing metric tags. Environment variables and field
      # interpolation is allowed.
      # 
      # * required
      # * type: string
      host = "${HOSTNAME}"
      region = "us-east-1"
      status = "{{status}}"

# Accepts `log` events and allows you to transform events with a full embedded Lua engine.
[transforms.lua]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `lua`.
  # 
  # * required
  # * type: string
  # * must be: "lua"
  type = "lua"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The inline Lua source to evaluate.
  # 
  # * required
  # * type: string
  source = """
require("script") # a `script.lua` file must be in your `search_dirs`

if event["host"] == nil then
  local f = io.popen ("/bin/hostname")
  local hostname = f:read("*a") or ""
  f:close()
  hostname = string.gsub(hostname, "\n$", "")
  event["host"] = hostname
end
"""

  # A list of directories search when loading a Lua file via the `require`
  # function.
  # 
  # * optional
  # * no default
  # * type: [string]
  search_dirs = ["/etc/vector/lua"]

# Accepts `log` events and allows you to parse a log field's value with a Regular Expression.
[transforms.regex_parser]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `regex_parser`.
  # 
  # * required
  # * type: string
  # * must be: "regex_parser"
  type = "regex_parser"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The Regular Expression to apply. Do not inlcude the leading or trailing `/`.
  # 
  # * required
  # * type: string
  regex = "^(?P<timestamp>.*) (?P<level>\\w*) (?P<message>.*)$"

  # If the specified `field` should be dropped (removed) after parsing.
  # 
  # * optional
  # * default: true
  # * type: bool
  drop_field = true
  drop_field = false

  # The log field to parse.
  # 
  # * optional
  # * default: "message"
  # * type: string
  field = "message"

  #
  # Types
  #

  [transforms.regex_parser.types]
    # A definition of log field type conversions. They key is the log field name
    # and the value is the type. `strptime` specifiers are supported for the
    # `timestamp` type.
    # 
    # * required
    # * type: string
    # * enum: "bool", "float", "int", "string", and "timestamp"
    status = "int"
    duration = "float"
    success = "bool"
    timestamp = "timestamp|%s"
    timestamp = "timestamp|%+"
    timestamp = "timestamp|%F"
    timestamp = "timestamp|%a %b %e %T %Y"

# Accepts `log` events and allows you to remove one or more log fields.
[transforms.remove_fields]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `remove_fields`.
  # 
  # * required
  # * type: string
  # * must be: "remove_fields"
  type = "remove_fields"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The log field names to drop.
  # 
  # * required
  # * type: [string]
  fields = ["field1", "field2"]

# Accepts `metric` events and allows you to remove one or more metric tags.
[transforms.remove_tags]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `remove_tags`.
  # 
  # * required
  # * type: string
  # * must be: "remove_tags"
  type = "remove_tags"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The tag names to drop.
  # 
  # * required
  # * type: [string]
  tags = ["tag1", "tag2"]

# Accepts `log` events and allows you to sample events with a configurable rate.
[transforms.sampler]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `sampler`.
  # 
  # * required
  # * type: string
  # * must be: "sampler"
  type = "sampler"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The rate at which events will be forwarded, expressed as 1/N. For example,
  # `rate = 10` means 1 out of every 10 events will be forwarded and the rest
  # will be dropped.
  # 
  # * required
  # * type: int
  rate = 10

  # A list of regular expression patterns to exclude events from sampling. If an
  # event's `"message"` key matches _any_ of these patterns it will _not_ be
  # sampled.
  # 
  # * optional
  # * no default
  # * type: [string]
  pass_list = ["[error]", "field2"]

# Accepts `log` events and allows you to split a field's value on a given separator and zip the tokens into ordered field names.
[transforms.split]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `split`.
  # 
  # * required
  # * type: string
  # * must be: "split"
  type = "split"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The field names assigned to the resulting tokens, in order.
  # 
  # * required
  # * type: [string]
  field_names = ["timestamp", "level", "message"]

  # If `true` the `field` will be dropped after parsing.
  # 
  # * optional
  # * default: true
  # * type: bool
  drop_field = true
  drop_field = false

  # The field to apply the split on.
  # 
  # * optional
  # * default: "message"
  # * type: string
  field = "message"

  # The separator to split the field on. If no separator is given, it will split
  # on whitespace.
  # 
  # * optional
  # * default: "whitespace"
  # * type: [string]
  separator = ","

  #
  # Types
  #

  [transforms.split.types]
    # A definition of log field type conversions. They key is the log field name
    # and the value is the type. `strptime` specifiers are supported for the
    # `timestamp` type.
    # 
    # * required
    # * type: string
    # * enum: "bool", "float", "int", "string", and "timestamp"
    status = "int"
    duration = "float"
    success = "bool"
    timestamp = "timestamp|%s"
    timestamp = "timestamp|%+"
    timestamp = "timestamp|%F"
    timestamp = "timestamp|%a %b %e %T %Y"

# Accepts `log` events and allows you to tokenize a field's value by splitting on white space, ignoring special wrapping characters, and zip the tokens into ordered field names.
[transforms.tokenizer]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `tokenizer`.
  # 
  # * required
  # * type: string
  # * must be: "tokenizer"
  type = "tokenizer"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The log field names assigned to the resulting tokens, in order.
  # 
  # * required
  # * type: [string]
  field_names = ["timestamp", "level", "message"]

  # If `true` the `field` will be dropped after parsing.
  # 
  # * optional
  # * default: true
  # * type: bool
  drop_field = true
  drop_field = false

  # The log field to tokenize.
  # 
  # * optional
  # * default: "message"
  # * type: string
  field = "message"

  #
  # Types
  #

  [transforms.tokenizer.types]
    # A definition of log field type conversions. They key is the log field name
    # and the value is the type. `strptime` specifiers are supported for the
    # `timestamp` type.
    # 
    # * required
    # * type: string
    # * enum: "bool", "float", "int", "string", and "timestamp"
    status = "int"
    duration = "float"
    success = "bool"
    timestamp = "timestamp|%s"
    timestamp = "timestamp|%+"
    timestamp = "timestamp|%F"
    timestamp = "timestamp|%a %b %e %T %Y"


# ------------------------------------------------------------------------------
# Sinks
# ------------------------------------------------------------------------------
# Sinks batch or stream data out of Vector.

# Batches `log` events to AWS CloudWatch Logs via the `PutLogEvents` API endpoint.
[sinks.aws_cloudwatch_logs]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `aws_cloudwatch_logs`.
  # 
  # * required
  # * type: string
  # * must be: "aws_cloudwatch_logs"
  type = "aws_cloudwatch_logs"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The group name of the target CloudWatch Logs stream.
  # 
  # * required
  # * type: string
  group_name = "{{ file }}"
  group_name = "ec2/{{ instance_id }}"
  group_name = "group-name"

  # The AWS region of the target CloudWatch Logs stream resides.
  # 
  # * required
  # * type: string
  region = "us-east-1"

  # The stream name of the target CloudWatch Logs stream.
  # 
  # * required
  # * type: string
  stream_name = "{{ instance_id }}"
  stream_name = "%Y-%m-%d"
  stream_name = "stream-name"

  # Dynamically create a log group if it does not already exist. This will ignore
  # `create_missing_stream` directly after creating the group and will create the
  # first stream.
  # 
  # * optional
  # * default: true
  # * type: bool
  create_missing_group = true
  create_missing_group = false

  # Dynamically create a log stream if it does not already exist.
  # 
  # * optional
  # * default: true
  # * type: bool
  create_missing_stream = true
  create_missing_stream = false

  # Custom endpoint for use with AWS-compatible services.
  # 
  # * optional
  # * no default
  # * type: string
  endpoint = "127.0.0.0:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 1049000
  # * type: int
  # * unit: bytes
  batch_size = 1049000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  request_timeout_secs = 30

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

  #
  # Buffer
  #

  [sinks.aws_cloudwatch_logs.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

# Streams `metric` events to AWS CloudWatch Metrics via the `PutMetricData` API endpoint.
[sinks.aws_cloudwatch_metrics]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `aws_cloudwatch_metrics`.
  # 
  # * required
  # * type: string
  # * must be: "aws_cloudwatch_metrics"
  type = "aws_cloudwatch_metrics"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # A namespace that will isolate different metrics from each other.
  # 
  # * required
  # * type: string
  namespace = "service"

  # The AWS region of the target CloudWatch stream resides.
  # 
  # * required
  # * type: string
  region = "us-east-1"

  # Custom endpoint for use with AWS-compatible services.
  # 
  # * optional
  # * no default
  # * type: string
  endpoint = "127.0.0.0:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

# Batches `log` events to AWS Kinesis Data Stream via the `PutRecords` API endpoint.
[sinks.aws_kinesis_streams]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `aws_kinesis_streams`.
  # 
  # * required
  # * type: string
  # * must be: "aws_kinesis_streams"
  type = "aws_kinesis_streams"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The AWS region of the target Kinesis stream resides.
  # 
  # * required
  # * type: string
  region = "us-east-1"

  # The stream name of the target Kinesis Logs stream.
  # 
  # * required
  # * type: string
  stream_name = "my-stream"

  # Custom endpoint for use with AWS-compatible services.
  # 
  # * optional
  # * no default
  # * type: string
  endpoint = "127.0.0.0:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # The log field used as the Kinesis record's partition key value.
  # 
  # * optional
  # * no default
  # * type: string
  partition_key_field = "user_id"

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 1049000
  # * type: int
  # * unit: bytes
  batch_size = 1049000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  request_timeout_secs = 30

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

  #
  # Buffer
  #

  [sinks.aws_kinesis_streams.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

# Batches `log` events to AWS S3 via the `PutObject` API endpoint.
[sinks.aws_s3]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `aws_s3`.
  # 
  # * required
  # * type: string
  # * must be: "aws_s3"
  type = "aws_s3"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The S3 bucket name. Do not include a leading `s3://` or a trailing `/`.
  # 
  # * required
  # * type: string
  bucket = "my-bucket"

  # The AWS region of the target S3 bucket.
  # 
  # * required
  # * type: string
  region = "us-east-1"

  # Custom endpoint for use with AWS-compatible services.
  # 
  # * optional
  # * no default
  # * type: string
  endpoint = "127.0.0.0:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 10490000
  # * type: int
  # * unit: bytes
  batch_size = 10490000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 300
  # * type: int
  # * unit: seconds
  batch_timeout = 300

  #
  # Object Names
  #

  # Whether or not to append a UUID v4 token to the end of the file. This ensures
  # there are no name collisions high volume use cases.
  # 
  # * optional
  # * default: true
  # * type: bool
  filename_append_uuid = true
  filename_append_uuid = false

  # The extension to use in the object name.
  # 
  # * optional
  # * default: "log"
  # * type: bool
  filename_extension = true
  filename_extension = false

  # The format of the resulting object file name. `strftime` specifiers are
  # supported.
  # 
  # * optional
  # * default: "%s"
  # * type: string
  filename_time_format = "%s"

  # A prefix to apply to all object key names. This should be used to partition
  # your objects, and it's important to end this value with a `/` if you want
  # this to be the root S3 "folder".
  # 
  # * optional
  # * default: "date=%F"
  # * type: string
  key_prefix = "date=%F/"
  key_prefix = "date=%F/hour=%H/"
  key_prefix = "year=%Y/month=%m/day=%d/"
  key_prefix = "application_id={{ application_id }}/date=%F/"

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  request_timeout_secs = 30

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

  #
  # Buffer
  #

  [sinks.aws_s3.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

# Streams `log` and `metric` events to a blackhole that simply discards data, designed for testing and benchmarking purposes.
[sinks.blackhole]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `blackhole`.
  # 
  # * required
  # * type: string
  # * must be: "blackhole"
  type = "blackhole"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The number of events that must be received in order to print a summary of
  # activity.
  # 
  # * required
  # * type: int
  print_amount = 1000

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

# Batches `log` events to Clickhouse via the `HTTP` Interface.
[sinks.clickhouse]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `clickhouse`.
  # 
  # * required
  # * type: string
  # * must be: "clickhouse"
  type = "clickhouse"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The host url of the Clickhouse server.
  # 
  # * required
  # * type: string
  host = "http://localhost:8123"

  # The table that data will be inserted into.
  # 
  # * required
  # * type: string
  table = "mytable"

  # The database that contains the stable that data will be inserted into.
  # 
  # * optional
  # * no default
  # * type: string
  database = "mydatabase"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 1049000
  # * type: int
  # * unit: bytes
  batch_size = 1049000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  request_timeout_secs = 30

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 9223372036854775807
  # * type: int
  retry_attempts = 9223372036854775807

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 9223372036854775807
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 9223372036854775807

  #
  # requests
  #

  # The compression strategy used to compress the encoded event data before
  # outputting.
  # 
  # * optional
  # * default: "gzip"
  # * type: string
  # * must be: "gzip" (if supplied)
  compression = "gzip"

  #
  # Basic auth
  #

  [sinks.clickhouse.basic_auth]
    # The basic authentication password.
    # 
    # * required
    # * type: string
    password = "password"
    password = "${PASSWORD_ENV_VAR}"

    # The basic authentication user name.
    # 
    # * required
    # * type: string
    user = "username"

  #
  # Buffer
  #

  [sinks.clickhouse.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Tls
  #

  [sinks.clickhouse.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

    # If `true` (the default), Vector will validate the TLS certificate of the
    # remote host. Do NOT set this to `false` unless you understand the risks of
    # not verifying the remote certificate.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_certificate = true
    verify_certificate = false

    # If `true` (the default), Vector will validate the configured remote host name
    # against the remote host's TLS certificate. Do NOT set this to `false` unless
    # you understand the risks of not verifying the remote hostname.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_hostname = true
    verify_hostname = false

# Streams `log` and `metric` events to standard output streams, such as `STDOUT` and `STDERR`.
[sinks.console]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `console`.
  # 
  # * required
  # * type: string
  # * must be: "console"
  type = "console"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # The standard stream to write to.
  # 
  # * optional
  # * default: "stdout"
  # * type: string
  # * enum: "stdout" or "stderr"
  target = "stdout"
  target = "stderr"

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "json" or "text"
  encoding = "json"
  encoding = "text"

# Batches `metric` events to Datadog metrics service using HTTP API.
[sinks.datadog_metrics]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `datadog_metrics`.
  # 
  # * required
  # * type: string
  # * must be: "datadog_metrics"
  type = "datadog_metrics"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # Datadog API key
  # 
  # * required
  # * type: string
  api_key = "3111111111111111aaaaaaaaaaaaaaaa"

  # A prefix that will be added to all metric names.
  # 
  # * required
  # * type: string
  namespace = "service"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # Datadog endpoint to send metrics to.
  # 
  # * optional
  # * default: "https://api.datadoghq.com"
  # * type: string
  host = "https://api.datadoghq.com"
  host = "https://api.datadoghq.eu"

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 20
  # * type: int
  # * unit: bytes
  batch_size = 20

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 60
  # * type: int
  # * unit: seconds
  request_timeout_secs = 60

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

# Batches `log` events to Elasticsearch via the `_bulk` API endpoint.
[sinks.elasticsearch]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `elasticsearch`.
  # 
  # * required
  # * type: string
  # * must be: "elasticsearch"
  type = "elasticsearch"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The `doc_type` for your index data. This is only relevant for Elasticsearch
  # <= 6.X. If you are using >= 7.0 you do not need to set this option since
  # Elasticsearch has removed it.
  # 
  # * optional
  # * default: "_doc"
  # * type: string
  doc_type = "_doc"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # The host of your Elasticsearch cluster. This should be the full URL as shown
  # in the example. This is required if the `provider` is not `"aws"`
  # 
  # * optional
  # * no default
  # * type: string
  host = "http://10.24.32.122:9000"

  # Index name to write events to.
  # 
  # * optional
  # * default: "vector-%F"
  # * type: string
  index = "vector-%Y-%m-%d"
  index = "application-{{ application_id }}-%Y-%m-%d"

  # The provider of the Elasticsearch service. This is used to properly
  # authenticate with the Elasticsearch cluster. For example, authentication for
  # AWS Elasticsearch Service requires that we obtain AWS credentials to properly
  # sign the request.
  # 
  # * optional
  # * default: "default"
  # * type: string
  # * enum: "default" or "aws"
  provider = "default"
  provider = "aws"

  # When using the AWS provider, the AWS region of the target Elasticsearch
  # instance.
  # 
  # * optional
  # * no default
  # * type: string
  region = "us-east-1"

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 10490000
  # * type: int
  # * unit: bytes
  batch_size = 10490000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 5
  # * type: int
  rate_limit_num = 5

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 5
  # * type: int
  request_in_flight_limit = 5

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 60
  # * type: int
  # * unit: seconds
  request_timeout_secs = 60

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

  #
  # Basic auth
  #

  [sinks.elasticsearch.basic_auth]
    # The basic authentication password.
    # 
    # * required
    # * type: string
    password = "password"
    password = "${PASSWORD_ENV_VAR}"

    # The basic authentication user name.
    # 
    # * required
    # * type: string
    user = "username"

  #
  # Buffer
  #

  [sinks.elasticsearch.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Headers
  #

  [sinks.elasticsearch.headers]
    # A custom header to be added to each outgoing Elasticsearch request.
    # 
    # * required
    # * type: string
    Authorization = "${TOKEN_ENV_VAR}"
    X-Powered-By = "Vector"

  #
  # Query
  #

  [sinks.elasticsearch.query]
    # A custom parameter to be added to each Elasticsearch request.
    # 
    # * required
    # * type: string
    X-Powered-By = "Vector"

  #
  # Tls
  #

  [sinks.elasticsearch.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

    # If `true` (the default), Vector will validate the TLS certificate of the
    # remote host. Do NOT set this to `false` unless you understand the risks of
    # not verifying the remote certificate.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_certificate = true
    verify_certificate = false

    # If `true` (the default), Vector will validate the configured remote host name
    # against the remote host's TLS certificate. Do NOT set this to `false` unless
    # you understand the risks of not verifying the remote hostname.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_hostname = true
    verify_hostname = false

# Streams `log` events to a file.
[sinks.file]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `file`.
  # 
  # * required
  # * type: string
  # * must be: "file"
  type = "file"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # File name to write events to.
  # 
  # * required
  # * type: string
  path = "vector-%Y-%m-%d.log"
  path = "application-{{ application_id }}-%Y-%m-%d.log"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # The amount of time a file can be idle  and stay open. After not receiving any
  # events for this timeout, the file will be flushed and closed.
  # 
  # * optional
  # * default: "30"
  # * type: int
  idle_timeout_secs = "30"

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "ndjson" or "text"
  encoding = "ndjson"
  encoding = "text"

# Batches `log` events to a generic HTTP endpoint.
[sinks.http]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `http`.
  # 
  # * required
  # * type: string
  # * must be: "http"
  type = "http"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The full URI to make HTTP requests to. This should include the protocol and
  # host, but can also include the port, path, and any other valid part of a URI.
  # 
  # * required
  # * type: string
  uri = "https://10.22.212.22:9000/endpoint"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  # A URI that Vector can request in order to determine the service health.
  # 
  # * optional
  # * no default
  # * type: string
  healthcheck_uri = "https://10.22.212.22:9000/_health"

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "ndjson" or "text"
  encoding = "ndjson"
  encoding = "text"

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 1049000
  # * type: int
  # * unit: bytes
  batch_size = 1049000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  batch_timeout = 5

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 10
  # * type: int
  rate_limit_num = 10

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 10
  # * type: int
  request_in_flight_limit = 10

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 30
  # * type: int
  # * unit: seconds
  request_timeout_secs = 30

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 10
  # * type: int
  retry_attempts = 10

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 10
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 10

  #
  # Basic auth
  #

  [sinks.http.basic_auth]
    # The basic authentication password.
    # 
    # * required
    # * type: string
    password = "password"
    password = "${PASSWORD_ENV_VAR}"

    # The basic authentication user name.
    # 
    # * required
    # * type: string
    user = "username"

  #
  # Buffer
  #

  [sinks.http.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Headers
  #

  [sinks.http.headers]
    # A custom header to be added to each outgoing HTTP request.
    # 
    # * required
    # * type: string
    Authorization = "${TOKEN_ENV_VAR}"
    X-Powered-By = "Vector"

  #
  # Tls
  #

  [sinks.http.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

    # If `true` (the default), Vector will validate the TLS certificate of the
    # remote host. Do NOT set this to `false` unless you understand the risks of
    # not verifying the remote certificate.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_certificate = true
    verify_certificate = false

    # If `true` (the default), Vector will validate the configured remote host name
    # against the remote host's TLS certificate. Do NOT set this to `false` unless
    # you understand the risks of not verifying the remote hostname.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_hostname = true
    verify_hostname = false

# Streams `log` events to Apache Kafka via the Kafka protocol.
[sinks.kafka]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `kafka`.
  # 
  # * required
  # * type: string
  # * must be: "kafka"
  type = "kafka"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # A list of host and port pairs that the Kafka client should contact to
  # bootstrap its cluster metadata.
  # 
  # * required
  # * type: [string]
  bootstrap_servers = ["10.14.22.123:9092", "10.14.23.332:9092"]

  # The log field name to use for the topic key. If unspecified, the key will be
  # randomly generated. If the field does not exist on the log, a blank value
  # will be used.
  # 
  # * required
  # * type: string
  key_field = "user_id"

  # The Kafka topic name to write events to.
  # 
  # * required
  # * type: string
  topic = "topic-1234"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "json" or "text"
  encoding = "json"
  encoding = "text"

  #
  # Buffer
  #

  [sinks.kafka.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Tls
  #

  [sinks.kafka.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Enable TLS during connections to the remote.
    # 
    # * optional
    # * default: false
    # * type: bool
    enabled = true
    enabled = false

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

# Exposes `metric` events to Prometheus metrics service.
[sinks.prometheus]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `prometheus`.
  # 
  # * required
  # * type: string
  # * must be: "prometheus"
  type = "prometheus"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The address to expose for scraping.
  # 
  # * required
  # * type: string
  address = "0.0.0.0:9598"

  # A prefix that will be added to all metric names.
  # It should follow Prometheus naming conventions.
  # 
  # * required
  # * type: string
  namespace = "service"

  # Default buckets to use for histogram metrics.
  # 
  # * optional
  # * default: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
  # * type: [float]
  # * unit: seconds
  buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

# Batches `log` events to a Splunk HTTP Event Collector.
[sinks.splunk_hec]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `splunk_hec`.
  # 
  # * required
  # * type: string
  # * must be: "splunk_hec"
  type = "splunk_hec"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # Your Splunk HEC host.
  # 
  # * required
  # * type: string
  host = "my-splunk-host.com"

  # Your Splunk HEC token.
  # 
  # * required
  # * type: string
  token = "A94A8FE5CCB19BA61C4C08"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "ndjson" or "text"
  encoding = "ndjson"
  encoding = "text"

  #
  # Batching
  #

  # The maximum size of a batch before it is flushed.
  # 
  # * optional
  # * default: 1049000
  # * type: int
  # * unit: bytes
  batch_size = 1049000

  # The maximum age of a batch before it is flushed.
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  batch_timeout = 1

  #
  # Requests
  #

  # The window used for the `request_rate_limit_num` option
  # 
  # * optional
  # * default: 1
  # * type: int
  # * unit: seconds
  rate_limit_duration = 1

  # The maximum number of requests allowed within the `rate_limit_duration`
  # window.
  # 
  # * optional
  # * default: 10
  # * type: int
  rate_limit_num = 10

  # The maximum number of in-flight requests allowed at any given time.
  # 
  # * optional
  # * default: 10
  # * type: int
  request_in_flight_limit = 10

  # The maximum time a request can take before being aborted. It is highly
  # recommended that you do not lower value below the service's internal timeout,
  # as this could create orphaned requests, pile on retries, and result in
  # deuplicate data downstream.
  # 
  # * optional
  # * default: 60
  # * type: int
  # * unit: seconds
  request_timeout_secs = 60

  # The maximum number of retries to make for failed requests.
  # 
  # * optional
  # * default: 5
  # * type: int
  retry_attempts = 5

  # The amount of time to wait before attempting a failed request again.
  # 
  # * optional
  # * default: 5
  # * type: int
  # * unit: seconds
  retry_backoff_secs = 5

  #
  # Buffer
  #

  [sinks.splunk_hec.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Tls
  #

  [sinks.splunk_hec.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

    # If `true` (the default), Vector will validate the TLS certificate of the
    # remote host. Do NOT set this to `false` unless you understand the risks of
    # not verifying the remote certificate.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_certificate = true
    verify_certificate = false

    # If `true` (the default), Vector will validate the configured remote host name
    # against the remote host's TLS certificate. Do NOT set this to `false` unless
    # you understand the risks of not verifying the remote hostname.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_hostname = true
    verify_hostname = false

# Streams `metric` events to StatsD metrics service.
[sinks.statsd]
  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `statsd`.
  # 
  # * required
  # * type: string
  # * must be: "statsd"
  type = "statsd"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # A prefix that will be added to all metric names.
  # 
  # * required
  # * type: string
  namespace = "service"

  # The UDP socket address to send stats to.
  # 
  # * optional
  # * default: "127.0.0.1:8125"
  # * type: string
  address = "127.0.0.1:8125"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

# Streams `log` events to a TCP connection.
[sinks.tcp]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `tcp`.
  # 
  # * required
  # * type: string
  # * must be: "tcp"
  type = "tcp"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The TCP address.
  # 
  # * required
  # * type: string
  address = "92.12.333.224:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # requests
  #

  # The encoding format used to serialize the events before outputting.
  # 
  # * required
  # * type: string
  # * enum: "json" or "text"
  encoding = "json"
  encoding = "text"

  #
  # Buffer
  #

  [sinks.tcp.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"

  #
  # Tls
  #

  [sinks.tcp.tls]
    # Absolute path to an additional CA certificate file, in DER or PEM format
    # (X.509).
    # 
    # * optional
    # * no default
    # * type: string
    ca_path = "/path/to/certificate_authority.crt"

    # Absolute path to a certificate file used to identify this connection, in DER
    # or PEM format (X.509) or PKCS#12. If this is set and is not a PKCS#12
    # archive, `key_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    crt_path = "/path/to/host_certificate.crt"

    # Enable TLS during connections to the remote.
    # 
    # * optional
    # * default: false
    # * type: bool
    enabled = true
    enabled = false

    # Pass phrase used to unlock the encrypted key file. This has no effect unless
    # `key_pass` above is set.
    # 
    # * optional
    # * no default
    # * type: string
    key_pass = "PassWord1"

    # Absolute path to a certificate key file used to identify this connection, in
    # DER or PEM format (PKCS#8). If this is set, `crt_path` must also be set.
    # 
    # * optional
    # * no default
    # * type: string
    key_path = "/path/to/host_certificate.key"

    # If `true` (the default), Vector will validate the TLS certificate of the
    # remote host. Do NOT set this to `false` unless you understand the risks of
    # not verifying the remote certificate.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_certificate = true
    verify_certificate = false

    # If `true` (the default), Vector will validate the configured remote host name
    # against the remote host's TLS certificate. Do NOT set this to `false` unless
    # you understand the risks of not verifying the remote hostname.
    # 
    # * optional
    # * default: true
    # * type: bool
    verify_hostname = true
    verify_hostname = false

# Streams `log` events to another downstream `vector` source.
[sinks.vector]
  #
  # General
  #

  # The component type. This is a required field that tells Vector which
  # component to use. The value _must_ be `vector`.
  # 
  # * required
  # * type: string
  # * must be: "vector"
  type = "vector"

  # A list of upstream source or transform IDs. See Config Composition for more
  # info.
  # 
  # * required
  # * type: [string]
  inputs = ["my-source-id"]

  # The downstream Vector address.
  # 
  # * required
  # * type: string
  address = "92.12.333.224:5000"

  # Enables/disables the sink healthcheck upon start.
  # 
  # * optional
  # * default: true
  # * type: bool
  healthcheck = true
  healthcheck = false

  #
  # Buffer
  #

  [sinks.vector.buffer]
    # The buffer's type / location. `disk` buffers are persistent and will be
    # retained between restarts.
    # 
    # * optional
    # * default: "memory"
    # * type: string
    # * enum: "memory" or "disk"
    type = "memory"
    type = "disk"

    # The maximum size of the buffer on the disk.
    # 
    # * optional
    # * no default
    # * type: int
    # * unit: bytes
    # * relevant when type = "disk"
    max_size = 104900000

    # The maximum number of events allowed in the buffer.
    # 
    # * optional
    # * default: 500
    # * type: int
    # * unit: events
    # * relevant when type = "memory"
    num_items = 500

    # The behavior when the buffer becomes full.
    # 
    # * optional
    # * default: "block"
    # * type: string
    # * enum: "block" or "drop_newest"
    when_full = "block"
    when_full = "drop_newest"
```


