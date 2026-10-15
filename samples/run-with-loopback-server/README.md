# Run with loopback server

> **Audience:** MXC consumers

Runs a one-request HTTP listener in a ProcessContainer on `127.0.0.1:8081`.
The request denies outbound traffic and allows bidirectional host loopback.

| Rust | .NET | Node |
|---|---|---|
| [`rust/src/main.rs`](rust/src/main.rs) | [`dotnet/Program.cs`](dotnet/Program.cs) | [`node/src/main.ts`](node/src/main.ts) |

All three run the readable [`server.ps1`](server.ps1) workload with read-only
access to its directory and stream its output. Requires a prepared Windows
host with BaseContainer general host-loopback support and Windows PowerShell.
See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites).
Run on a test VM with native binaries and SDK built from the same revision.

## Run

From the corresponding language directory:

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

Wait for `LISTENING=http://127.0.0.1:8081/`, then run the
[loopback client](../run-with-loopback-client/README.md) in a second terminal
within 60 seconds. The server exits after responding.

To allow the server to initiate internet connections too, change its
`egress.default` from deny to allow in the chosen language's request.
The ingress policy has no source-CIDR or port rules: other reachable
host-loopback processes can connect to the listener.
