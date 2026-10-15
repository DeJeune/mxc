# MXC SDK samples

> **Audience:** MXC consumers

These samples demonstrate the supported V1 SDK surfaces in Rust, .NET, and
Node. They are organized by scenario so equivalent operations are easy to
compare while each implementation remains idiomatic for its language.

| Scenario | Purpose | Rust | .NET | Node |
|---|---|---|---|---|
| [Run in a persisted container](run-in-persisted-container/README.md) | Provision, start, execute, stop, and deprovision | `run-in-persisted-container/rust` | `run-in-persisted-container/dotnet` | `run-in-persisted-container/node` |
| [Run in a transient container](run-in-transient-container/README.md) | Build and execute a complete run-to-completion request | `run-in-transient-container/rust` | `run-in-transient-container/dotnet` | `run-in-transient-container/node` |
| [Run with filesystem containment](run-with-containment-of-filesystem/README.md) | Grant one host directory read-only | `run-with-containment-of-filesystem/rust` | `run-with-containment-of-filesystem/dotnet` | `run-with-containment-of-filesystem/node` |
| [Run with network containment](run-with-containment-of-network/README.md) | Deny outbound, inbound, and host-loopback networking | `run-with-containment-of-network/rust` | `run-with-containment-of-network/dotnet` | `run-with-containment-of-network/node` |
| [ProcessContainer runtime proxy](run-with-process-container-networking/README.md) | Use an existing identity-scoped or host runtime proxy (Windows) | `run-with-process-container-networking/rust` | `run-with-process-container-networking/dotnet` | `run-with-process-container-networking/node` |
| [Cooperative proxy](run-with-cooperative-proxy/README.md) | Proxy environment variables and direct egress (Windows) | `run-with-cooperative-proxy/rust` | `run-with-cooperative-proxy/dotnet` | `run-with-cooperative-proxy/node` |
| [Loopback server](run-with-loopback-server/README.md) | Expose a container HTTP listener to host loopback (Windows) | `run-with-loopback-server/rust` | `run-with-loopback-server/dotnet` | `run-with-loopback-server/node` |
| [Loopback client](run-with-loopback-client/README.md) | Connect to one container listener through host loopback (Windows) | `run-with-loopback-client/rust` | `run-with-loopback-client/dotnet` | `run-with-loopback-client/node` |
| [Public egress](run-with-public-egress/README.md) | Permit HTTPS to one public IP address (Windows) | `run-with-public-egress/rust` | `run-with-public-egress/dotnet` | `run-with-public-egress/node` |
| [Run with logging access denied](run-with-logging-access-denied/README.md) | Capture blocked ProcessContainer accesses | `run-with-logging-access-denied/rust` | `run-with-logging-access-denied/dotnet` | `run-with-logging-access-denied/node` |
| [Run with captured I/O](run-with-io-captured/README.md) | Return complete stdout and stderr after exit | `run-with-io-captured/rust` | `run-with-io-captured/dotnet` | `run-with-io-captured/node` |
| [Run with interactive PTY I/O](run-with-io-pty-interactive/README.md) | Forward an interactive terminal | `run-with-io-pty-interactive/rust` | `run-with-io-pty-interactive/dotnet` | `run-with-io-pty-interactive/node` |
| [Run with streaming standard I/O](run-with-io-stdio-streaming/README.md) | Stream live stdout and stderr | `run-with-io-stdio-streaming/rust` | `run-with-io-stdio-streaming/dotnet` | `run-with-io-stdio-streaming/node` |
| [Run with telemetry](run-with-telemetry/README.md) | Request consent, then opt one transient run into telemetry | `run-with-telemetry/rust` | `run-with-telemetry/dotnet` | `run-with-telemetry/node` |
| [Check containment support](dryrun-check-containment-support/README.md) | Discover support without launching a container | `dryrun-check-containment-support/rust` | `dryrun-check-containment-support/dotnet` | `dryrun-check-containment-support/node` |

## Conventions

- Each sample demonstrates one primary concept through the public versioned V1 API.
- Requests use typed SDK models rather than ["MXC request JSON"](../docs/schema.md#mxc-request-json) or generated JSON serialization types.
- Commands are harmless, bounded, and selected for the current operating system.
- Workload failures, timeouts, SDK errors, warnings, and cleanup failures remain visible.
- Projects reference the SDK in this repository so the samples track the current source.

Running a sample requires a supported and prepared host. See the applicable
backend guide under [`docs/`](../docs/) for platform-specific prerequisites.
