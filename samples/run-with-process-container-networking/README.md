# ProcessContainer runtime proxy

> **Audience:** MXC consumers

Runs an HTTPS client through an existing loopback proxy using the public V1
SDK. Each language has a standalone program:

| Rust | .NET | Node |
|---|---|---|
| [`rust/src/main.rs`](rust/src/main.rs) | [`dotnet/Program.cs`](dotnet/Program.cs) | [`node/src/main.ts`](node/src/main.ts) |

Requires a prepared Windows ProcessContainer host with BaseContainer proxy
support and `curl.exe`. See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites)
for KBs and runtime capabilities. Stage native binaries built from the same
revision as the SDK. Run these workloads on a test VM.

## Runtime proxy

Start your own HTTP CONNECT proxy on `127.0.0.1:8080` and keep it alive until
the sample exits. It must allow HTTPS access to `www.example.com`; the sample
does not implement or start a proxy.

For a packaged proxy, set its exact installed Package Family Name:

```powershell
$env:MXC_PROXY_PEER = Get-AppxPackage -Name Contoso.AgentProxy |
  Select-Object -ExpandProperty PackageFamilyName
```

For an unpackaged AppContainer proxy, set `MXC_PROXY_PEER` to its profile name.
The request denies general host loopback and authorizes the named peer.
See the [proxy manifest](../../docs/backends/process-container/examples/proxy-package.md)
for packaging and firewall requirements.

For an ordinary unpackaged host proxy, unset `MXC_PROXY_PEER`:

```powershell
Remove-Item Env:MXC_PROXY_PEER -ErrorAction SilentlyContinue
```

This selects the development/testing compatibility path with
`hostLoopback: allow`, not the strict bidirectional loopback guarantee.
See the [runtime proxy inbound limitation](../../docs/backends/process-container/networking.md#discovering-identity-less-loopback-proxy-support).
Both forms deny direct egress. MXC supplies the proxy environment variables
automatically.

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

Expected stdout contains the HTTPS response body. SDK errors, warnings,
workload exit codes, and timeouts are reported.

## Other networking samples

| Scenario | Sample |
|---|---|
| Cooperative local or public proxy | [Run with cooperative proxy](../run-with-cooperative-proxy/README.md) |
| Container loopback listener | [Run with loopback server](../run-with-loopback-server/README.md) |
| Container loopback client | [Run with loopback client](../run-with-loopback-client/README.md) |
| Direct public IP egress | [Run with public egress](../run-with-public-egress/README.md) |

Each sample includes Rust, .NET, and Node implementations.
