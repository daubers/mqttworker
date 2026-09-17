# mqttworker

The worker agent binary. Connects to an MQTT broker, announces its
capabilities, listens for job commands, and runs jobs as Docker containers,
streaming their output back over MQTT.

See the [workspace README](../README.md) for the overall architecture.

## Running

```bash
cargo run -p mqttworker -- \
  --configuration-file config.toml \
  --job-definition-file job.toml
```

Both arguments are required:

| Flag | Description |
|---|---|
| `-c`, `--configuration-file` | Path to the node/broker configuration TOML file. |
| `-j`, `--job-definition-file` | Path to a job definition TOML file, run once on startup. |

Requires a running Docker daemon (job containers are created via the local
Docker socket) and a reachable MQTT broker.

## Configuration file

```toml
node_name = "worker-01"

[broker]
broker_uri = "mqtt://localhost:1883"
broker_authenticate = false
topics = ["workers/worker-01/#"]
```

- `node_name` — this worker's identity. It is used to build the topic this
  worker subscribes to and listens for commands on
  (`workers/<node_name>/<cmd>`).
- `broker.broker_authenticate` — defaults to `true`; set `false` for an
  unauthenticated broker.
- `broker.credentials` — optional `{ username, password }`, required when
  `broker_authenticate` is `true`.
- `broker.topics` — topics this worker subscribes to (typically
  `workers/<node_name>/#`).

## Job definition file

A job definition describes one or more named jobs, each a list of tasks.
Only the first task of the first job in the map is currently executed.

```toml
[worker_requirements]
min_no_cpus = 1

[job.main]
image = "alpine:latest"
cmds = ["echo hello", "uname -a"]
```

- `worker_requirements.min_no_cpus` — declared but not currently enforced.
- `job.<name>` — a list of tasks; each task has:
  - `image` (default `alpine:latest`) — the Docker image to pull and run.
  - `cmds` (optional) — a list of shell commands written to the
    container's stdin in order, followed by `exit`.

The same TOML format is sent as the `workflow` field of a `startjob`
message (see below) to trigger a job remotely instead of from a file.

## Behavior

On startup, `mqttworker`:

1. Connects to the configured broker and subscribes to `broker.topics`.
2. Immediately runs the first task from `--job-definition-file`.
3. Schedules a job (every 5 seconds) that opens a short-lived MQTT
   connection and publishes a `CapabilitiesMessage` to
   `workers/capabilities`.
4. Listens for messages matching `workers/<node_name>/<cmd>`:
   - `startjob` — parses the message payload as a `WorkerStartJobMessage`,
     extracts the job definition from its `workflow` field, and runs it in
     a new Docker container, streaming stdout to
     `workers/jobs/<job_id>/<task_id>/stdout`.
   - `stopjob` — logged only; not yet implemented.
5. Shuts down the scheduler cleanly on `SIGINT`, `SIGTERM`, or `SIGHUP`.
6. Automatically retries reconnecting to the broker (every 10s) if the
   connection is lost.

## Dependencies of note

- [`paho-mqtt`](https://crates.io/crates/paho-mqtt) — MQTT client.
- [`bollard`](https://crates.io/crates/bollard) — Docker Engine API client.
- [`tokio-cron-scheduler`](https://crates.io/crates/tokio-cron-scheduler) — scheduling the periodic capabilities announcement.
- [`messages`](../messages) — shared message types and MQTT connection helpers.

## Roadmap

Work on this crate is tracked in the `MW` project on the
[project's YouTrack instance](https://pm.daubney.dev/issue/MW). Ticket
states are kept in sync with the code (see each ticket's comments for the
reasoning, including where an implementation diverged from the original
acceptance criteria):

| Ticket | State | Summary |
|---|---|---|
| [`MW-7`](https://pm.daubney.dev/issue/MW-7) | Fixed | Make the worker async with tokio, to support `bollard`. |
| [`MW-6`](https://pm.daubney.dev/issue/MW-6) | Fixed | Read configuration from a file instead of hardcoded values (broker URI, credentials, auth toggle, topics, node name). |
| [`MW-3`](https://pm.daubney.dev/issue/MW-3) | Fixed | Execute jobs in Docker containers — see `src/containers.rs::run_task`. |
| [`MW-2`](https://pm.daubney.dev/issue/MW-2) | Fixed | Job assignment — solved differently than originally specced: dispatch on a `workers/<node_name>/<cmd>` regex (`startjob`/`stopjob`) rather than a single `.../request` topic. |
| [`MW-1`](https://pm.daubney.dev/issue/MW-1) | Open | Generate and persist a per-worker UUID as its identity. Not done — identity is still just the `node_name` string from the config file. |
| [`MW-4`](https://pm.daubney.dev/issue/MW-4) | Open | Report job status (`started`/`completed`/`failed`, exit code, timestamp) over MQTT. Not done — only raw container stdout is published. |
| [`MW-5`](https://pm.daubney.dev/issue/MW-5) | Open | Graceful shutdown via an MQTT `Shutdown` command. Partially done — OS signal handling (`SIGINT`/`SIGTERM`/`SIGHUP`) shuts the scheduler down, but there's no MQTT-triggered shutdown command or `ShutdownExpected` announcement. |
| [`MW-8`](https://pm.daubney.dev/issue/MW-8) | Open | Make the default Docker image for script jobs configurable (split out of `MW-3`/`MW-6`). |
| [`MW-9`](https://pm.daubney.dev/issue/MW-9) | Open | Make the capabilities broadcast interval configurable (split out of `MW-6`; currently hardcoded to 5s). |
