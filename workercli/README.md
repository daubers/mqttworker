# workercli

A command-line client for submitting jobs to a [`mqttworker`](../mqttworker)
instance over MQTT.

See the [workspace README](../README.md) for the overall architecture.

## Configuration

By default, `workercli` reads its configuration from
`~/.mqttworker/cli_config.yaml`. Override the path with `-c`/
`--configuration-file-path` or the `MQTTWORKER_CLIENT_CONFIG_PATH`
environment variable.

Despite the `.yaml` default filename, the file is parsed as **TOML**
(via `toml::from_str`):

```toml
client_name = "cli"

[broker]
broker_uri = "mqtt://localhost:1883"
broker_authenticate = false
topics = []
```

- `client_name` — the worker id this CLI publishes as in message envelopes.
- `broker` — see [`messages::mqtt::Broker`](../messages) for all fields
  (URI, optional authentication credentials, subscribed topics — topics
  are unused by this CLI, which only publishes).

## Usage

```bash
cargo run -p workercli -- job run --job-file-path job.toml
```

| Command | Description |
|---|---|
| `job run --job-file-path <path> [--follow]` | Reads the job definition TOML file at `<path>` and publishes it as a `startjob` message. |

The job file's contents are sent verbatim as the `workflow` field of a
`WorkerStartJobMessage` (see the [`mqttworker` README](../mqttworker) for
the job definition format), published to `workers/sample1/startjob`.

Notes on the current implementation:
- The target worker id is hardcoded to `sample1` rather than taken from a
  flag or config value.
- `--follow` is accepted but not yet implemented (job output is not
  streamed back to the CLI).

## Dependencies of note

- [`clap`](https://crates.io/crates/clap) (derive, env features) — argument parsing.
- [`messages`](../messages) — shared message types and the synchronous MQTT connection helper (`connect_client_sync`).

## Further documentation

This crate isn't yet documented on the project wiki (`wiki.daubney.dev`) —
the wiki's architecture overview on the
[Mqttworker page](https://wiki.daubney.dev/wiki/Mqttworker) still lists
only five of the workspace's six crates.
