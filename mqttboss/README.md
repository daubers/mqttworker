# mqttboss

The "boss" process. Subscribes to all worker traffic on the broker and
maintains a Postgres-backed record of known workers: their capabilities,
availability, and last-seen time. Also exposed as a library so the
[`web`](../web) crate can query the same models/schema.

See the [workspace README](../README.md) for the overall architecture.

## Running

```bash
cargo run -p mqttboss
```

The broker host, client id, and topic are currently hardcoded in
`src/main.rs` (`localhost:1883`, client id `boss`, subscribed to
`workers/#`) rather than read from a configuration file.

## Database setup

Requires PostgreSQL and the [Diesel CLI](https://diesel.rs/guides/getting-started):

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Set `DATABASE_URL` (via a `.env` file in this directory or the
environment):

```
DATABASE_URL=postgres://pguser:mysecretpassword@localhost/mqttworker
```

Run migrations:

```bash
diesel migration run
```

Migrations live in [`migrations/`](migrations) and currently create the
`workers` table and evolve it (adding an `available` column, breaking out
capability fields).

## Data model

The `workers` table (see [`src/schema.rs`](src/schema.rs)):

| Column | Type | Notes |
|---|---|---|
| `id` | `Int4` | primary key |
| `name` | `Varchar(255)` | worker id, from the message's `worker_id` |
| `last_seen` | `Timestamp?` | updated on announcements |
| `cpus` | `Int4?` | |
| `ram` | `Int4?` | |
| `disk` | `Float8?` | |
| `gpu` | `Int4?` | |
| `tags` | `Jsonb?` | |
| `available` | `Bool` | derived from announcement type |

`src/models/workers/` contains the Diesel model types (`Workers`,
`CreateWorkers`, `UpdateWorkers`) and query helpers (`create`, `read`,
`update`, `search_by_message`).

## Message handling

`src/messaging/workers.rs::process_message` is called for every MQTT
message received on `workers/#`. It uses
[`messages::messages::process_message`](../messages) to decode the payload
and then:

- **Capabilities** message: creates a `workers` row if one doesn't already
  exist for that worker id.
- **Announcement** message: creates or updates the worker's row, setting
  `available` based on the announcement type (`Online` = available,
  `ShutdownUnexpected` / `ShutdownExpected` = unavailable) and refreshing
  `last_seen` on update.
- Other message types are currently ignored.

Note: a new database connection is opened per message
(`establish_connection()` inside `process_message`), and an unrecognized
topic (`process_message` returning `None`) currently triggers a `todo!()`
panic.

## Library exports

Other crates (namely [`web`](../web)) depend on this crate as a library for
its `db`, `models`, and `schema` modules:

```toml
[dependencies]
mqttboss = { path = "../mqttboss" }
```

## Dependencies of note

- [`diesel`](https://crates.io/crates/diesel) (Postgres, chrono, serde_json features) — ORM / query builder.
- [`dotenvy`](https://crates.io/crates/dotenvy) — loads `DATABASE_URL` from `.env`.
- [`paho-mqtt`](https://crates.io/crates/paho-mqtt) — MQTT client (synchronous).
- [`cron_tab`](https://crates.io/crates/cron_tab) — scheduling (see `src/scheduler.rs`; currently unused/dead code).
- [`messages`](../messages) — shared message types.

## Further documentation

There's no dedicated wiki page for this crate, but the
[Mqttworker wiki page](https://wiki.daubney.dev/wiki/Mqttworker) covers its
role, the `workers/#` topic subscription, and the `workers` table schema
(consistent with `src/schema.rs`).
