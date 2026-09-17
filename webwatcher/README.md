# webwatcher

A terminal UI (built with [Ratatui](https://ratatui.rs/) and
[Crossterm](https://crates.io/crates/crossterm)) for watching raw MQTT
traffic live. Useful for debugging what workers, the boss, and the web API
are publishing to the broker.

See the [workspace README](../README.md) for the overall architecture.

## Running

```bash
cargo run -p webwatcher
```

Connects to `tcp://localhost:1883` (hardcoded in `src/main.rs`) with client
id `webwatcher_tui` and subscribes to every topic (`#`).

## Usage

- The top pane shows connection status and a running log of received
  messages, one per line, timestamped to the microsecond.
- Messages recognized by
  [`messages::messages::process_message`](../messages) (announcements,
  capabilities, worker requests) are pretty-printed as their decoded Rust
  struct; unrecognized payloads are shown as raw text.
- The middle pane shows a live per-topic message count.
- Only the last 100 messages are kept in memory; older ones are dropped.

### Keybindings

| Key | Action |
|---|---|
| `↑` / `↓` or mouse wheel | Scroll messages vertically |
| `←` / `→` | Scroll messages horizontally |
| `q` | Quit |

## Dependencies of note

- [`ratatui`](https://crates.io/crates/ratatui) / [`crossterm`](https://crates.io/crates/crossterm) — terminal UI and input handling.
- [`paho-mqtt`](https://crates.io/crates/paho-mqtt) — MQTT client.
- [`messages`](../messages) — shared message types, used to decode known message payloads for display.

## Further documentation

There's no dedicated wiki page for this crate; see the
[Mqttworker wiki page](https://wiki.daubney.dev/wiki/Mqttworker) for where
it fits in the overall architecture.
