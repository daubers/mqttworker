# web

An HTTP/OpenAPI service that exposes worker state from Postgres (via
[`mqttboss`](../mqttboss)) and can publish MQTT messages. Built with
[Poem](https://github.com/poem-web/poem) / `poem-openapi`, with a built-in
Swagger UI.

See the [workspace README](../README.md) for the overall architecture.

## Running

```bash
cargo run -p web
```

Requires:
- A reachable MQTT broker (connection details are set up in
  `src/mqtt.rs::connect_client`).
- `DATABASE_URL` set (via `.env` in this directory or the environment),
  since it establishes a Postgres connection via `mqttboss::db` on startup:

```
DATABASE_URL=postgres://pguser:mysecretpassword@localhost/mqttworker
```

The server listens on `0.0.0.0:3000`.

## Endpoints

Swagger UI is served at `/`, and the OpenAPI-described API is nested under
`/api`:

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/hello` | Sample endpoint. Publishes a test MQTT message and returns a greeting (optionally personalized with a `name` query param). |
| `GET` | `/api/workers` | Returns all known workers as JSON, read from the `workers` table via `mqttboss::models::workers::Workers::read`. |

Visit `http://localhost:3000/` for interactive API docs.

## Shared state

`AppState` (in `src/main.rs`) holds a mutex-guarded async MQTT client and a
mutex-guarded synchronous Postgres connection, injected into handlers via
Poem's `Data` extractor / `AddData` middleware.

## Dependencies of note

- [`poem`](https://crates.io/crates/poem) / [`poem-openapi`](https://crates.io/crates/poem-openapi) — web framework, OpenAPI generation, Swagger UI.
- [`paho-mqtt`](https://crates.io/crates/paho-mqtt) — MQTT client.
- [`mqttboss`](../mqttboss) — database models/schema and connection helper (used as a library, not run as its own process here).

## Further documentation

See the [Web Crate wiki page](https://wiki.daubney.dev/wiki/Web_Crate) for
a fuller API/implementation reference.
