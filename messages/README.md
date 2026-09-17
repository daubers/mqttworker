# messages

Shared library crate used by every other crate in the workspace. It defines
the MQTT message types exchanged between workers, the boss, the web API,
and clients, plus small helpers for connecting to the broker.

See the [workspace README](../README.md) for the overall architecture.

## What's in here

### `messages::messages`

Message structs (de)serialized to/from JSON and published/consumed over
MQTT:

- `Message` / `MessageConfig` — the common envelope (direction, worker id,
  message type, topic, message id) embedded in every other message type.
- `CapabilitiesMessage` — a worker's hardware capabilities (memory, core
  count, architecture), published to `workers/capabilities`.
- `WorkerAnnouncement` / `WorkerAnnouncementType` — online/offline
  announcements, published to `workers/announcements`. Can optionally run a
  background thread that re-publishes itself on an interval.
- `WorkerRequestMessage`, `WorkerRequestQuery` (`RunJob` / `JobStatus` /
  `Shutdown`) — request messages sent to a worker.
- `WorkerStartJobMessage` — a request to start a job, published to
  `workers/<target_worker_id>/startjob`. Carries the job definition as a
  TOML string in `workflow`.
- `process_message(&paho_mqtt::Message) -> Option<MessageType>` — inspects
  a raw MQTT message's topic and deserializes it into the matching message
  type (`Announcement`, `Capabilities`, `WorkerRequest`), or `None` for an
  unrecognized topic.

### `messages::mqtt`

- `Broker` / `Credentials` — serde-friendly configuration structs for an
  MQTT broker connection (URI, optional username/password, subscribed
  topics). Used by every binary's own configuration file format.
- `ConnectedClient` — an async MQTT client plus its message stream.
- `connect_client_async(broker, send_online) -> ConnectedClient` — connects
  an async client (MQTT v5), optionally authenticates, and optionally
  publishes a `WorkerAnnouncement::Online` on connect.
- `connect_client_sync(send_online, &broker) -> mqtt::Client` — the
  synchronous equivalent, used by `workercli` and `mqttboss`.

## Usage

Add as a path dependency from within the workspace:

```toml
[dependencies]
messages = { path = "../messages" }
```

This crate has no binary of its own and cannot be run directly.

## Further documentation

See the [Messages Crate wiki page](https://wiki.daubney.dev/wiki/Messages_Crate)
for a fuller reference of the message envelope and each message type. Note
it predates `WorkerStartJobMessage` and the job topics
(`workers/<node_name>/startjob`, `workers/jobs/...`), which aren't
mentioned there yet.
