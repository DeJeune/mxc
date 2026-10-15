# Run with cooperative proxy

> **Audience:** MXC consumers

Runs an HTTPS client through a proxy using environment variables and a direct
egress rule, without MXC runtime proxy configuration.

| Rust | .NET | Node |
|---|---|---|
| [`rust/src/main.rs`](rust/src/main.rs) | [`dotnet/Program.cs`](dotnet/Program.cs) | [`node/src/main.ts`](node/src/main.ts) |

Requires a prepared Windows ProcessContainer host with BaseContainer egress
filtering and `curl.exe`. See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites).
Run on a test VM with native binaries and SDK built from the same revision.

Start your own HTTP CONNECT proxy on TCP port 8080. It must allow HTTPS access
to `www.example.com`. Set `MXC_PROXY_IP` to `127.0.0.1` (the default) or a
reachable public IPv4 address:

```powershell
$env:MXC_PROXY_IP = '127.0.0.1'
```

The sample permits only that IP and TCP port. It sets `HTTP_PROXY`,
`HTTPS_PROXY`, and their lowercase forms while inheriting the backend's
default environment. Local proxy mode allows host loopback and also permits
unsolicited host-loopback connections to container listeners. Public proxy
mode denies host loopback. This example does not support private-LAN proxies.

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
