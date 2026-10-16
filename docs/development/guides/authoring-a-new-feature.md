# Adding a new feature

> **Audience:** MXC developers

This guide covers features used through the Rust, .NET, and Node SDKs, as well
as the executor binaries. After issue and design agreement, use only the
implementation stages that apply to your change. An SDK-only change does not
need a new schema or backend implementation.

## 1. Start with a GitHub issue

Search existing issues before opening a new one. Use the appropriate
[issue template](../../../.github/ISSUE_TEMPLATE/) to describe the problem or
feature request. Include a use case and the desired behavior; a full design
is not needed at this point. For security reports, follow
[SECURITY.md](../../../SECURITY.md) instead of opening a public issue.

Say in the issue if you want to implement the change. Wait for a maintainer
to confirm that the problem is valid, the change fits MXC, and you can take
it on. This avoids duplicate work and implementation of features the project
does not intend to support.

Maintainers decide whether a separate feature spec is needed. Small fixes
and straightforward improvements can proceed with an agreed approach in the
issue. Significant changes to policy, APIs, isolation, or architecture may
need design review first. Acceptance of the issue is not approval of a design
or a promise to merge the implementation.

## 2. Agree on the design

If maintainers request a spec, open a separate documentation PR after the
initial issue discussion. Copy the
[feature spec template](../architecture/designs/feature-spec-template.md) to
`docs/development/architecture/designs/<feature-name>.md` and link the issue.
Keep implementation out of this PR. Wait for maintainer approval and merge
of the spec before starting implementation.

Agree on the following points in the issue, or in the spec when one is required:

- The user problem, intended behavior, and non-goals.
- The supported backends, host requirements, and affected operations.
- Defaults and the meaning of omitted or explicitly empty values.
- Policy or API changes, compatibility, and any migration requirements.
- Security implications, rejection behavior, and failure cleanup.
- Alternatives considered, acceptance criteria, and the functional test plan.

Keep the detail proportional to the change. Do not write a full spec before
maintainers have agreed that design work is warranted. If the scope or design
changes during implementation, discuss it before proceeding.

Read the [repository architecture](../architecture/repository-architecture.md),
[configuration schema](../../schema.md), and the affected
[backend guide](../../backends/). Compare the corresponding
[SDK API references](../../api-reference/README.md) before designing public types.

Check all affected operations: create-and-run execution, streaming, PTY, and
container lifecycle operations. Include discovery and validation APIs when
their results need to change. Do not extend backend capabilities just to make
the SDKs look alike.

## Find the existing implementation path

All entry points use the same native engine and backend enforcement.
Rust calls the engine in-process. Node and .NET map typed requests to MXC
request JSON and call `mxc_ffi` through native bindings. The executors are
separate clients of the engine, not the SDKs' implementation.

| Area | Where to start |
| --- | --- |
| Contracts and adapters | `src/mxc-sdk/src/core/mxc_contract/` and `src/mxc-sdk/src/core/mxc_common/config_contract_adapters/` |
| Routing and authorization | `src/mxc-sdk/src/core/mxc_engine/` |
| Backend validation and enforcement | `src/mxc-sdk/src/backends/` |
| Rust SDK | `src/mxc-sdk/src/`: `lib.rs`, `policy/`, `configs.rs`, `options.rs`, and `state_aware_sdk.rs` |
| .NET SDK | `sdk/dotnet/Microsoft.Mxc.Sdk/V1/` |
| Node SDK | `sdk/node/src/v1/` and `sdk/node/src/bindings/` |
| Native C ABI | `src/ffi/mxc_ffi/` |
| Executor binaries | `src/tools/` |

Trace a similar existing feature from its request type to backend enforcement
before adding code. Reuse that path. A successful executor test does not prove
that SDK request mapping, result handling, or cleanup works.

## 3. Review policy/schema additions separately

Use a separate PR for policy/schema additions, before backend implementation
and public API exposure. Skip this stage if the request contract is unchanged.

- Add fields to the relevant development contract at their permanent JSON
  locations. Do not add an `experimental` JSON wrapper.
- Update the version-specific adapter. Preserve omitted values through parsing
  and binding; leave backend defaults and policy decisions to the backend.
- Regenerate the required artifacts using the
  [schema codegen guide](../build-and-test/schema-codegen.md). Generated schema
  and TypeScript artifacts are not permission to expose a public SDK feature.

- Add contract fixtures for accepted requests, invalid inputs, field presence,
  and rejection by older contracts. Register new request roots when needed.

Published contracts and files under `schemas/stable/` are immutable.
Keep unsupported requests rejected while implementation is incomplete.
Recognizing a field must not cause MXC to silently accept unenforced policy.

## 4. Implement and validate the internal behavior

Implement every backend the feature claims to support. For a backend-specific
feature, implement that backend and preserve rejection elsewhere.

- Keep host checks, defaults, and enforcement in the backend. Shared parsing
  handles contract shape and conversion, not backend policy.
- Reuse existing types or backend-owned configuration. A feature does not
  automatically need a new shared field in `models.rs` or a new Rust crate.
- Keep routing in `mxc_engine`. Keep the SDK and executor entry points thin.
- Preserve errors, ownership, cleanup, platform gates, and existing behavior.

Use the existing authorization and capability checks. The engine's experimental
opt-in authorizes registered experimental backends; it is not a general switch
for every new policy field. Do not add a runner guard that ignores a requested
restriction when authorization is absent.

If Windows support requires OS work, coordinate it before exposing the feature.
See the [ProcessContainer OS-work guide](process-container-adding-os-features.md).
Check capabilities at runtime; a schema version does not prove host support.

Split large changes into reviewable working slices. Validate each slice without
adding a public SDK API just to test unfinished backend work.

### Test behavior, not just request construction

Use development-contract fixtures and existing native test entry points while
the public SDK feature is unavailable. Add or extend functional scenarios in
the applicable scheduled/nightly CI suite.

- Show that the requested restriction or capability actually takes effect.
- Check unsupported backends, hosts, and missing authorization where applicable.
- Cover omitted and empty values, relevant policy interactions, and failure cleanup.
- Check that existing supported behavior still works.

Unit and adapter tests support this coverage; they do not replace it.
A config file, debug message, or test that merely repeats a new helper is not
proof of enforcement. Follow the
[validation runbook](../build-and-test/ci-validation-infrastructure.md) for test
hosts and CI wiring. Record actual results and any validation gaps.

## Promoting to Stable

Contract publication and backend graduation are separate decisions.

- Publish a new contract rather than changing an existing published one.
  Follow the [versioning design](../architecture/versioning.md) and codegen
  process for registration, artifacts, and SDK contract targets.

- Do not remove authorization because a field appears in a published schema.
  Changing a backend's experimental status is an engine registry change.
  Validate its effect across the affected entry points.

Typed SDK APIs can expose a field only when their supported published contract
contains it. Do not advance an SDK target merely because a development
contract contains the field.

## 5. Add public SDK/API support last

Start this PR only after the intended backend behavior and its functional
validation are complete. Keep public operations and types under supported
versioned entry points: `mxc_sdk::v1`, `Microsoft.Mxc.Sdk.V1`, and
`@microsoft/mxc-sdk/v1`.

Preserve existing public APIs. Breaking changes require a new versioned API.

Compare all three SDK references. Keep field meanings, defaults, optional-field
presence, errors, and ownership consistent. Use each language's normal naming
and construction style. Keep workload and policy data in requests; invocation
controls belong in the operation's options.

- **Rust:** update the applicable request/configuration types, contract mapping,
  and versioned exports. Cover the affected create-and-run, streaming, or
  lifecycle calls through the public API.

- **.NET:** update the typed requests, JSON writers, native adapters, and result
  mapping as needed. Register serialized/deserialized types in `MxcJsonContext`,
  use the `MxcJson` helpers, and add matching AOT smoke coverage.

- **Node:** update the versioned types and request/result mapping in the native
  bindings. Preserve field absence. Cover the public calls, not only an
  internal request builder.

Reuse the existing C ABI operations when they can carry the feature.
Do not add an ABI entry point merely to carry another request field.
If the ABI must change, update the affected bindings together and preserve
panic containment and native resource ownership.

### Validate SDK access

Check both the SDK-to-contract mapping and the behavior of the public operation.
Include the relevant errors, output/results, and cleanup. An executor-only test
does not cover these SDK paths.

| Coverage | Existing test area |
| --- | --- |
| Rust contracts, native behavior, and SDK APIs | `src/mxc-sdk/tests/` |
| .NET API and mapping tests | `sdk/dotnet/Microsoft.Mxc.Sdk.Tests/` |
| .NET source-generated JSON/AOT checks | `sdk/dotnet/Microsoft.Mxc.Sdk.AotSmokeTest/` |
| Node mapping and integration tests | `sdk/node/tests/unit/` and `sdk/node/tests/integration/` |
| Host-dependent backend validation | `tests/scripts/` |

Use the relevant [contributor test commands](../../../CONTRIBUTING.md#testing)
and SDK READMEs: [Rust](../../../src/mxc-sdk/README.md),
[.NET](../../../sdk/dotnet/README.md), and [Node](../../../sdk/node/README.md).
Wire applicable SDK scenarios into CI rather than leaving them as local-only
examples. Do not require unrelated platform tests or claim tests that were not run.

Update the affected SDK README, versioned API references, and useful samples
with API changes. Keep backend documentation focused on usage, supported hosts,
limits, and actionable errors. Do not add an implementation diary or a
walkthrough of every internal detail.

## Checklist

Use this to check the completed feature, not as a requirement for every
individual PR to complete every stage.

- [ ] Issue and contribution approach agreed with a maintainer; requested spec approved and merged.
- [ ] Policy/schema changes, if needed, reviewed in a separate PR.
- [ ] Applicable contracts, adapters, and generated artifacts updated; published contracts preserved.
- [ ] Intended backend(s) enforce the feature; unsupported requests are rejected.
- [ ] Required authorization and host-capability checks verified.
- [ ] Functional/regression coverage wired into CI; existing behavior checked and actual validation results recorded.
- [ ] Public SDK/API exposure lands last, after backend validation, with concise usage and support documentation.
