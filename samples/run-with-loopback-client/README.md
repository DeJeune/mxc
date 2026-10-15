# Run with loopback client

> **Audience:** MXC consumers

Connects a ProcessContainer to the [loopback server](../run-with-loopback-server/README.md)
using an egress rule limited to `127.0.0.1:8081` over TCP.

| Rust | .NET | Node |
|---|---|---|
| [`rust/src/main.rs`](rust/src/main.rs) | [`dotnet/Program.cs`](dotnet/Program.cs) | [`node/src/main.ts`](node/src/main.ts) |

Requires a prepared Windows ProcessContainer host with BaseContainer general
host-loopback support and `curl.exe`. See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites).
Run on a test VM with native binaries and SDK built from the same revision.
Host loopback is bidirectional; its allow setting cannot restrict inbound
clients by identity, address, or port.

## Run

Start the loopback server first. When it prints `LISTENING`, run from the
corresponding language directory in a second terminal:

```text
# Rust
cargo run

# .NET
dotnet run

# Node
npm --prefix ../../../sdk/node install
npm --prefix ../../../sdk/node run build
npm install
npm start
```

Expected stdout is `OK`. Curl proxying is disabled so the request goes
directly to the listener.
