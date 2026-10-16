# MXC releases and SDK versioning

> **Audience:** MXC consumers and developers

MXC releases on a roughly two-week cadence. Each release publishes an updated
package for every supported SDK that has changed: Rust, .NET, and Node.
Unchanged packages do not need a new version or publication.
Changes to bundled native runtime assets count as package changes even when
the language-specific API is unchanged.

## Release schedule

Plan two release windows each month:

- The week of the Windows optional non-security release, usually the fourth
  week of the month.
- Approximately two weeks after that release, followed by the next monthly
  Windows-aligned window.

Maintainers publish the actual target dates on the
**Microsoft eXecution COntainers** GitHub project. Calendar months and Windows
release dates vary, so this is not a promise of exactly 14 days between every
release. Windows calendar alignment does not limit releases to Windows changes;
MXC releases cover all supported platforms and SDKs.

An urgent customer need can justify a break-glass patch outside these windows.
The patch still requires the applicable validation and release notes.

## SDK packages and schema versions

SDK package versions and MXC request JSON schema versions are independent.
The Rust, .NET, and Node package versions can also differ from each other.
A release is a collection of changed artifacts, not a requirement to give every
artifact the same version.

Select each SDK package version by its highest-impact change:

| Change | SDK version increment |
| --- | --- |
| A breaking change to the published SDK API or behavior | Major |
| A backward-compatible API addition or change in supported schema capabilities | Minor |
| Bug fixes or internal refactoring without either of the above | Patch |

A schema change does not automatically require a matching SDK version number.
Assess its effect on each package: a breaking SDK change is still major, not
minor merely because it originated in a schema. A package with no changes is
not republished just to match another SDK.

Package versions do not replace the supported versioned SDK API surfaces or the
schema contract selected by the SDK. Preserve published API compatibility,
published schemas, and the SDK's registered contract targets as described in the
[versioning design](development/architecture/versioning.md).

**Release tooling prerequisite:** `scripts/check-version-sync.js` currently
requires the Rust workspace, Node package, and .NET SDK project versions to
match. Independent package publication requires decoupling that gate and
auditing the packaging and release paths that consume those versions. This
policy does not claim that those tooling changes are already implemented; do
not bypass the gate to ship an independently versioned package.

## Release contents and compatibility

For each release, document:

- Which binaries and SDK packages changed, their versions, and their changes.
- Which MXC request JSON contracts and versioned SDK APIs those packages support.
- New backend features, their host prerequisites, and any migration steps.

An SDK or schema version alone does not establish Windows feature availability.
A Windows backend feature may require a particular OS build, update, or runtime
capability. Document concrete prerequisites in the relevant backend guide and
the [Windows OS-version support reference](backends/process-container/os-version-support.md);
internal monthly update shorthand is not a public support boundary.

When an older host cannot enforce a requested feature, reject the request
before creating a container. The error should identify the requested feature,
backend, detected host/build or missing capability, and the required update or
other supported remedy. Never silently discard policy to make a request run.

## Support decisions still to be made

The maintainers still need to define how long MXC versions and Windows
OS/build combinations receive support, including retirement criteria and
notice periods. The release cadence does not establish an end-of-support policy.

Feature availability must remain explicit even before that policy is settled.
For example, if a future MXC 1.3 feature requires a newer Windows update, a
customer on an older build must get a clear compatibility error and matching
documentation, not an assumption that installing MXC 1.3 updates the OS.

See [project management and triage](development/project-management.md) for
release planning, issue intake, and maintainer responsibilities.
