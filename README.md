# mqttworker

An experimental, MQTT-driven distributed job runner written in Rust. Workers
connect to an MQTT broker, announce their capabilities, and run jobs as
Docker containers on request. A "boss" process tracks worker state in
Postgres, a web API exposes that state over HTTP, a CLI can dispatch jobs,
and a terminal UI can watch the raw MQTT traffic.

> **Status:** early-stage / experimental. APIs, message formats, and
> configuration file schemas are all subject to change without notice.

## Architecture

The repository is a Cargo workspace made up of six crates:

Each crate has its own README with more detail; the table below links to them.

| Crate | Type | Purpose |
|---|---|---|
| [`messages`](messages/README.md) | library | Shared MQTT message types (capabilities, announcements, job requests) and MQTT connection helpers used by every other crate. |
| [`mqttworker`](mqttworker/README.md) | binary | The worker agent. Connects to the broker, periodically announces its capabilities, listens for job commands on `workers/<node_name>/...`, and runs jobs in Docker containers (via [Bollard](https://github.com/fussybeaver/bollard)), streaming container output back over MQTT. |
| [`mqttboss`](mqttboss/README.md) | binary + library | Subscribes to `workers/#`, tracks worker state (capabilities, availability, last-seen) and persists it to Postgres via [Diesel](https://diesel.rs/). |
| [`web`](web/README.md) | binary | An HTTP/OpenAPI service (built with [Poem](https://github.com/poem-web/poem)) that exposes worker state from Postgres and can publish MQTT messages. Includes a Swagger UI. |
| [`webwatcher`](webwatcher/README.md) | binary | A terminal UI (built with [Ratatui](https://ratatui.rs/)) that subscribes to all MQTT topics (`#`) and shows a live, scrollable log plus per-topic message counts. |
| [`workercli`](workercli/README.md) | binary | A command-line client for submitting jobs to a worker over MQTT. |

### How a job runs

1. A client (`workercli`, or anything else publishing to the right topic)
   sends a `startjob` message to `workers/<node_name>/startjob` containing a
   job definition (as a TOML string) and a target worker id.
2. The targeted `mqttworker` instance parses the job definition, pulls the
   configured Docker image, starts a container, and streams the container's
   stdin/stdout, publishing output to `workers/jobs/<job_id>/<task_id>/stdout`.
3. `mqttboss` and `webwatcher` observe worker/job activity on the broker;
   `mqttboss` persists worker capability/availability data to Postgres, and
   `web` serves that data over HTTP.

## Prerequisites

- Rust (2024 edition toolchain)
- Docker (required by `mqttworker` to run job containers)
- An MQTT broker (e.g. Mosquitto)
- PostgreSQL (required by `mqttboss` and `web`)

A `docker-compose.yml` is provided to run a local broker and database:

```bash
docker compose up -d
```

This starts:
- `eclipse-mosquitto` on `1883`
- `postgres` on `5432` (user `pguser`, password `mysecretpassword`)

## Building

```bash
cargo build --workspace
```

## Configuration

### Worker (`mqttworker`)

Takes a node configuration file and a job definition file:

```bash
cargo run -p mqttworker -- \
  --configuration-file config.toml \
  --job-definition-file job.toml
```

`config.toml`:

```toml
node_name = "worker-01"

[broker]
broker_uri = "mqtt://localhost:1883"
broker_authenticate = false
topics = ["workers/worker-01/#"]
```

`job.toml` (also the format sent in a `startjob` message payload's
`workflow` field):

```toml
[worker_requirements]
min_no_cpus = 1

[job.main]
image = "alpine:latest"
cmds = ["echo hello", "uname -a"]
```

On startup the worker runs the first task in the job definition once, then
announces its capabilities to `workers/capabilities` every 5 seconds and
listens for further commands on `workers/<node_name>/<cmd>`
(`startjob` / `stopjob`).

### Boss (`mqttboss`) and web API (`web`)

Both require a Postgres connection string via a `.env` file or environment
variable:

```
DATABASE_URL=postgres://pguser:mysecretpassword@localhost/mqttworker
```

Run migrations with the [Diesel CLI](https://diesel.rs/guides/getting-started):

```bash
cd mqttboss
diesel migration run
```

Then run the boss (connects to `localhost:1883` and subscribes to
`workers/#`):

```bash
cargo run -p mqttboss
```

And the web API (listens on `0.0.0.0:3000`, Swagger UI at `/`):

```bash
cargo run -p web
```

### CLI (`workercli`)

Reads a broker/config file (default `~/.mqttworker/cli_config.yaml`,
overridable via `--configuration-file-path` or the
`MQTTWORKER_CLIENT_CONFIG_PATH` env var):

```yaml
client_name: "cli"
broker:
  broker_uri: "mqtt://localhost:1883"
  broker_authenticate: false
  topics: []
```

Submit a job to a worker:

```bash
cargo run -p workercli -- job run --job-file-path job.toml
```

### Watcher (`webwatcher`)

A terminal UI that connects to `tcp://localhost:1883` and subscribes to all
topics:

```bash
cargo run -p webwatcher
```

Use the arrow keys or mouse wheel to scroll, and `q` to quit.

## MQTT topic layout

| Topic | Published by | Purpose |
|---|---|---|
| `workers/announcements` | any worker | Online/offline announcements. |
| `workers/capabilities` | `mqttworker` | Periodic capability broadcast (CPU, memory, architecture). |
| `workers/<node_name>/startjob` | job clients | Request a worker to start a job. |
| `workers/<node_name>/stopjob` | job clients | Request a worker to stop a job (not yet implemented). |
| `workers/jobs/<job_id>/<task_id>/stdout` | `mqttworker` | Streamed stdout from a running job container. |

## Further documentation

More architectural background, design rationale, and the roadmap for where
each crate is headed are kept on the project wiki:

- [Mqttworker](https://wiki.daubney.dev/wiki/Mqttworker) — project overview, MQTT topics, database schema, key dependencies
- [Mqttworker Crate](https://wiki.daubney.dev/wiki/Mqttworker_Crate) — worker implementation detail and planned architecture (worker identity, job assignment/execution/status reporting, graceful shutdown)
- [Messages Crate](https://wiki.daubney.dev/wiki/Messages_Crate) — message envelope and message type reference
- [Web Crate](https://wiki.daubney.dev/wiki/Web_Crate) — REST API reference

The planned architecture from the Mqttworker Crate wiki page is tracked as
issues `MW-1`–`MW-9` in the `MW` project on the
[project's YouTrack instance](https://pm.daubney.dev/issue/MW), kept in
sync with the code — see the
[Roadmap section of the mqttworker README](mqttworker/README.md#roadmap)
for current ticket states.

The wiki itself is not kept in sync the same way: it documents a "planned"
job-execution architecture that's already implemented, and still lists
five crates (no `workercli`). Treat it as design background rather than a
source of truth for current behavior — this README and the per-crate
READMEs reflect the code as it stands.

## License

No license has been specified for this project yet.
