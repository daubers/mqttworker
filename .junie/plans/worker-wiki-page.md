---
sessionId: session-260712-215213-1hkd
---

# Requirements

> **Note:** A YouTrack project "Mqttworker" (shortName: MW) has been created at https://pm.daubney.dev/ (project id: 0-3).

### Overview & Goals
Create a wiki page for the `mqttworker` crate that documents both its current state and the planned/intended architecture.

Since the worker is currently a testbed, the page will need to cover:
- What the worker currently does (connect, announce, broadcast capabilities, receive messages)
- The intended final architecture (to be discussed)

### Current Worker Behaviour
The current `mqttworker/src/main.rs` (async, post-MW-7 refactor):
- Connects to the MQTT broker asynchronously using `mqtt::AsyncClient`
- Publishes a `WorkerAnnouncement::Online` on connect
- Uses `tokio-cron-scheduler` to publish a `CapabilitiesMessage` every 5 seconds via a separate sync MQTT client
- Listens for OS signals (SIGINT, SIGTERM, SIGHUP) and shuts down cleanly
- Disconnects the MQTT client on shutdown

### Scope
**In Scope:**
- New wiki page `Mqttworker_Crate` (or similar title)
- Link from the overview `Mqttworker` page
- Document current behaviour
- Document planned architecture (pending discussion)

**Out of Scope:**
- ~~Actual code changes to the worker~~ (now in scope — see Step 6+)

# Architecture Discussion

### Key Questions
Before writing the wiki page, the intended final architecture of the worker needs to be agreed upon. Key areas to decide:

#### 1. Job Execution ✅ Decided
Jobs are either:
- A **Docker container** to run (with a configurable default container in a config file)
- A **script** to be executed

All jobs run inside a Docker container. A sane default container image is set in the worker's configuration file.

#### 2. Job Assignment ✅ Decided
- `mqttboss` publishes job assignments to a **worker-specific topic** (e.g. `workers/<id>/request`)
- Each worker subscribes to its own topic and pulls jobs from it

#### 3. Worker Lifecycle ✅ Decided
- Workers have a **unique persistent ID** — generated once and stored in the config/data directory, reused across restarts
- Workers support **graceful shutdown via MQTT command** published to their worker-specific topic

#### 4. Status Reporting 🔶 Partially Decided
- Status reporting goes over MQTT
- May also need to be **persisted in a state store** (e.g. the existing PostgreSQL database used by `mqttboss`)
- Exact topics, message types, and persistence strategy TBD

The wiki page will document the agreed architecture alongside the current testbed state.

# Delivery Steps

###   Step 1: Create YouTrack project ✅ Done
Created the "Mqttworker" project (MW) at https://pm.daubney.dev/ linked to this codebase.

###   Step 2: Discuss and agree on the worker's intended architecture ✅ Done
Align on the final design for the mqttworker before writing documentation.

- Clarify what jobs a worker will execute (shell commands, registered tasks, etc.)
- Decide how the boss assigns work to a specific worker (dedicated topic vs shared queue)
- Decide on worker identity (persistent ID vs generated on startup)
- Decide on job status reporting (topics, message types)
- Decide on graceful shutdown via MQTT

###   Step 2b: Create YouTrack tickets for the worker ✅ Done
Created 6 tickets in the MW project at https://pm.daubney.dev/:

- MW-1: Worker identity — generate unique persistent ID on startup
- MW-2: Worker subscribes to its own worker-specific MQTT topic for job assignment
- MW-3: Worker executes jobs in Docker containers
- MW-4: Worker reports job status over MQTT
- MW-5: Worker supports graceful shutdown via MQTT command
- MW-6: Worker reads configuration from a config file

###   Step 3: Create the Mqttworker Crate wiki page ✅ Done
New wiki page created at https://wiki.daubney.dev/wiki/Mqttworker_Crate (pageid 49, newrevid 113).

- Documented the current testbed implementation (connect, announce, capabilities broadcast, message receive loop)
- Documented the agreed intended architecture with sections for: worker identity, job assignment, job execution, status reporting, graceful shutdown, configuration
- Noted which parts are not yet implemented (all planned sections)
- Linked to YouTrack tickets MW-1 through MW-6

###   Step 4: Link the new page from the overview ✅ Done
The Mqttworker overview page links to the new worker crate page (newrevid 114).

- Added `[[Mqttworker Crate|Worker node]]` link in the Architecture table row for `mqttworker`

###   Step 5: Create MW-7 ticket for async worker refactor ✅ Done
Created YouTrack ticket MW-7: "Make worker async using tokio to support bollard Docker API crate"

- Covers: adding tokio dependency, refactoring main to async, replacing sync paho-mqtt Client with AsyncClient, replacing thread::spawn/sleep with tokio equivalents, adding bollard dependency
- Motivation: bollard (Docker API crate) is async-first and requires tokio runtime

###   Step 6: Update MW-1 and MW-5 tickets with final decisions ✅ Done
Updated YouTrack tickets to reflect the decided lifecycle behaviour:
- MW-1: Worker ID is generated once, persisted to disk, and reused on restart
- MW-5: Graceful shutdown triggered by an MQTT message on the worker-specific topic

###   Step 6b: Investigate Docker Compose support in bollard
Research whether bollard supports Docker Compose files and document findings.

- Check bollard's API surface for Compose-related functionality (bollard does not natively support Docker Compose — it wraps the Docker Engine API which has no Compose endpoint)
- Evaluate alternatives: shelling out to `docker compose` CLI, using the Docker Engine API to manually orchestrate multi-container setups, or using a Rust Compose parser
- Document the recommended approach in the wiki and/or as a YouTrack ticket
- Create a MW ticket capturing the decision and any follow-up work

###   Step 7: Implement MW-7 — Refactor worker to async (tokio) ✅ Done
Refactored `mqttworker/src/main.rs` and `mqttworker/Cargo.toml` to use async/await with tokio.

- Added `tokio` and `tokio-cron-scheduler` to `mqttworker/Cargo.toml`
- `fn main()` is now `async fn main()` with `#[tokio::main]`
- Uses `mqtt::AsyncClient` for the main connection
- Capabilities broadcast uses `tokio-cron-scheduler` (every 5 seconds) with a sync client per invocation
- Graceful shutdown via OS signal handling (SIGINT, SIGTERM, SIGHUP) using `tokio::signal`

###   Step 8: Implement MW-1 — Persistent worker identity
Worker generates a UUID on first run, persists it to a file, and reloads it on subsequent starts.

- Add logic to read/write a `worker_id.txt` (or similar) file in a configurable data directory
- Use the persisted UUID as the `worker_id` in all messages
- Add `uuid` crate usage for generation (already a dependency in `messages`)

###   Step 8b: Update Mqttworker Crate wiki page to reflect async refactor ✅ Done
Updated https://wiki.daubney.dev/wiki/Mqttworker_Crate (newrevid 115) to reflect the current code state.

- Updated Current Implementation section: describes the two-file structure (`main.rs` + `mqtt.rs`), async connect, tokio-cron-scheduler for capabilities broadcast, OS signal handling
- Updated Dependencies table to include tokio and tokio-cron-scheduler
- Updated Hardcoded Values table to reflect current values (`mqtt://localhost:1883`, `worker_tmp_id`, 5-second cron)
- Added bollard note in Job Execution section (no native Compose support)
- Removed outdated references to sync thread-based implementation

###   Step 8c: Document web crate REST API on the wiki ✅ Done
Created https://wiki.daubney.dev/wiki/Web_Crate (newrevid 117) and updated Mqttworker overview (newrevid 118).

- Created new `Web_Crate` wiki page documenting `GET /api/workers` endpoint, response schema, dependencies, and how to run
- Updated Architecture table on overview page: `web` row now links to `[[Web Crate]]` and describes the `GET /api/workers` endpoint
- Added `workers/<worker_id>/request` topic to the MQTT Topics table on the overview page
- Fixed worker startup command on overview page to include `--configuration-file /path/to/config.toml`

###   Step 9: Update Mqttworker Crate wiki page to reflect configuration implementation ✅ Done
Updated https://wiki.daubney.dev/wiki/Mqttworker_Crate to reflect the current code state after MW-6 partial implementation.

- Updated `mqtt.rs` description: `connect_client_async` now takes `configuration: Config` parameter, reads broker URI/credentials from config struct
- Updated `main.rs` description: uses `clap` to parse `--configuration-file` CLI arg, loads `Config` from TOML file, passes config to `connect_client_async`, reloads config inside scheduler job
- Added `configuration.rs` section: documents `Config` struct (`node_name: String`, `broker: Broker`) and `Broker` struct (`broker_uri`, `broker_username`, `broker_password`), TOML deserialization via `serde`
- Updated Dependencies table: added `clap`, `serde`, `toml` crates
- Updated Hardcoded Values table: broker URI and credentials are now read from config file; only `worker_tmp_id` and cron interval remain hardcoded
- Updated Configuration section in Planned Architecture: mark broker URI, username/password as implemented; remaining TBD items are default Docker image and broadcast interval

###   Step 9b: Update Mqttworker Crate wiki page to reflect further configuration changes ✅ Done
Updated https://wiki.daubney.dev/wiki/Mqttworker_Crate (newrevid 116) to reflect current code state.

- Added `configuration.rs` section: documents new `Credentials` struct, `broker_authenticate: bool` field, and `topics: Vec<String>` field
- Updated `mqtt.rs` description: `connect_client_async` now takes `Arc<Config>`, uses `broker_authenticate` flag to conditionally authenticate
- Updated `main.rs` description: subscribes to all topics from `configuration.broker.topics`
- Updated Dependencies table: corrected paho-mqtt version to 0.14.0, added `smol` crate
- Updated Configuration section status table: added `broker_authenticate`, `topics`, and `node_name` as implemented; only default Docker image and broadcast interval remain

###   Step 10: Implement MW-5 — Graceful shutdown via MQTT
Worker listens for a shutdown command on its worker-specific topic and shuts down cleanly.

- Subscribe to `workers/<worker_id>/request` in addition to `#`
- Detect a `WorkerRequestShutdown` message type and trigger graceful shutdown
- Publish a `WorkerAnnouncement::ShutdownExpected` before disconnecting
- Cancel running tokio tasks cleanly (use a shutdown channel / `CancellationToken`)