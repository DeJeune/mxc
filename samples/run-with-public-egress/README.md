# Run with public egress

> **Audience:** MXC consumers

Allows a ProcessContainer to reach one public IPv4 address on TCP port 443.
Host loopback and other egress destinations are denied.

| Rust | .NET | Node |
|---|---|---|
| [`rust/src/main.rs`](rust/src/main.rs) | [`dotnet/Program.cs`](dotnet/Program.cs) | [`node/src/main.ts`](node/src/main.ts) |

Requires a prepared Windows host with BaseContainer egress filtering and
`curl.exe`. See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites).
Run on a test VM with native binaries and SDK built from the same revision.

Set a real HTTPS destination and one of its public IPv4 addresses:

```powershell
$env:MXC_DESTINATION_HOST = 'www.example.com'
$env:MXC_DESTINATION_IP = '<permitted-public-ipv4>'
```

Curl's `--resolve` pins the connection to that address while preserving the
hostname for TLS verification. Curl proxying is disabled. This is an IP
egress rule, not DNS-name filtering. Replace the placeholder; documentation
addresses such as `203.0.113.10` are not routable destinations.

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

Expected stdout contains the HTTPS response body.
