# ProcessContainer networking

> **Audience:** MXC consumers

Runnable Node samples for Windows ProcessContainer networking using
`@microsoft/mxc-sdk/v1`. The SDK selects the request contract; these samples
do not author executor JSON or select a schema version.

The requests are in [`node/src/requests.ts`](node/src/requests.ts).
[`node/src/main.ts`](node/src/main.ts) probes the request, streams workload
output, reports warnings, and returns the workload exit code (124 on timeout).
The samples require a prepared Windows host with the BaseContainer tier,
Windows PowerShell, and `curl.exe`. See [networking prerequisites](../../docs/backends/process-container/networking.md#windows-update-prerequisites)
for the required KBs and runtime capabilities. Run these workloads on a test VM.
Stage native MXC binaries built from the same revision as the SDK; rebuilding
the TypeScript SDK alone does not refresh `mxc_ffi.dll`.

## Build

From this sample's `node` directory:

```powershell
npm --prefix ..\..\..\sdk\node install
npm --prefix ..\..\..\sdk\node run build
npm install
npm run build
```

`npm test` checks request construction without launching a container or
contacting a proxy.

## Runtime proxy

Start your own HTTP CONNECT proxy on `127.0.0.1:8080` before running the
sample and keep it alive until the client exits. It must allow HTTPS access
to `www.example.com`. The samples do not implement or start a proxy.

For a packaged proxy, supply its exact installed Package Family Name:

```powershell
$env:MXC_PROXY_PEER = Get-AppxPackage -Name Contoso.AgentProxy |
  Select-Object -ExpandProperty PackageFamilyName
node dist\main.js runtime-proxy
```

For an unpackaged AppContainer proxy, set `MXC_PROXY_PEER` to its profile name.
The sample sets `containment.config.network.allowedProxyPeer` and denies
general host loopback. See the [proxy manifest](../../docs/backends/process-container/examples/proxy-package.md)
for packaging and firewall requirements.

For an ordinary unpackaged host proxy, omit the identity:

```powershell
Remove-Item Env:MXC_PROXY_PEER -ErrorAction SilentlyContinue
node dist\main.js runtime-proxy
```

This uses the weaker development/testing compatibility path with
`hostLoopback: 'allow'`. It is not the strict bidirectional loopback guarantee;
see the [runtime proxy inbound limitation](../../docs/backends/process-container/networking.md#discovering-identity-less-loopback-proxy-support).
Both runtime-proxy forms deny direct egress and set
`network.runtimeConfig.networkProxy`. MXC supplies the proxy environment
variables automatically. No direct egress allow rules are added.

## Cooperative proxy

This is direct egress, not MXC runtime proxy mode. The sample sets
`HTTP_PROXY`, `HTTPS_PROXY`, and their lowercase forms while inheriting the
backend's normal environment. Only the selected IPv4 address and TCP port
8080 are permitted.

```powershell
$env:MXC_PROXY_IP = '127.0.0.1'
node dist\main.js cooperative-proxy
```

For a public proxy, replace `MXC_PROXY_IP` with its reachable public IPv4
address. The proxy must listen on port 8080. Public proxy mode denies host
loopback; local proxy mode allows it and also permits unsolicited
host-loopback connections to container listeners. There are no ingress
source-CIDR or port rules to narrow that grant.

## Container loopback server and client

In one terminal, start a one-request HTTP listener on `127.0.0.1:8081` with
outbound traffic denied:

```powershell
node dist\main.js loopback-server
```

Wait for `LISTENING=http://127.0.0.1:8081/`, then run the client in another
terminal within 60 seconds:

```powershell
node dist\main.js loopback-client
```

The client permits only `127.0.0.1:8081` over TCP and prints `OK`.
The server exits after responding. To demonstrate a server that also permits
unrestricted outbound traffic, use `loopback-server-egress-allow` instead.
Both server forms and the client explicitly allow bidirectional host
loopback. Other reachable host-loopback processes can connect to the listener;
the policy cannot restrict inbound clients by identity, address, or port.

## Direct public egress

Supply a real HTTPS destination and one of its public IPv4 addresses:

```powershell
$env:MXC_DESTINATION_HOST = 'www.example.com'
$env:MXC_DESTINATION_IP = '<permitted-public-ipv4>'
node dist\main.js public-egress
```

The sample allows only that IP on TCP port 443, denies host loopback, and
disables curl proxying. Curl's `--resolve` pins the connection to the selected
address while preserving the hostname for TLS verification. This is an IP
egress rule, not DNS-name filtering. Replace the placeholder; documentation
addresses such as `203.0.113.10` are not routable test destinations.
