# Configuration Strategy

> **Audience:** MXC developers

## Local development

Developers should use public registries like `crates.io`
and `npmjs` directly so they can iterate quickly.

## For CI/Pipelines

### Central Feed Services
Production CI pipelines use an Azure Artifacts feed (CFS) to source dependencies
from crates.io and npmjs, helping ensure secure and vetted consumption of third‑party packages.
(Microsoft engineers can consult the internal "Central Feed Services" documentation for setup details; external readers can treat the centralized feed as a Microsoft-internal Azure Artifacts mirror of the public registries.)

### Production Build and Release pipelines
- The ADO pipeline is the official build pipeline that signs the binaries and
  drives public releases. It runs on merge to `main` and on a nightly schedule.
- The shared build stages assemble the managed and native .NET SDK binaries
  into two `Microsoft.Mxc.Sdk` NuGet packages in both official and unofficial
  builds. Official builds sign the managed and native binaries before packaging,
  then sign and verify each package through ESRP with CP-401405. The NuGet.org
  payload keeps the project version; the internal-test payload includes the
  unique Azure Pipelines build ID in its prerelease version so repeated
  publications do not collide.
- A fast .NET pipeline validation stage runs in parallel with the Rust builds.
  It verifies the .NET SDK/runtime setup and packs both NuGet variants with
  synthetic native inputs so pipeline and package-contract failures surface
  without waiting for the cross-platform Rust artifacts.
- `1ES.Release.Nuget.yml` lets the runner publish the signed .NET SDK package to
  the internal `Mxc-Azure-Feed` by default, or to NuGet.org through ESRP.
- `1ES.Release.Npm.yml` publishes the signed npm SDK package through ESRP.

### Rust build storage and agent sizing

The official build does not restore or save Cargo target directories. Those
directories contain compiled libraries, test executables, intermediate files,
and full release symbols, not just downloaded dependencies. Restoring old
workspace output can cost more than rebuilding it. The 1ES Rust workflow's
`cacheOptions.enabled: true` still permits caching Cargo tool installations;
`enableTargetCache: false` disables the separate compiled-output cache.
Compilation output is reused within a job, but not between runs.

Windows/Linux builds and lint jobs put `CARGO_TARGET_DIR` under
`$(Pipeline.Workspace)/target`, on the agent work disk rather than `C:` or
`/tmp`. The Windows .NET test-support DLL uses a separate
`$(Pipeline.Workspace)/dotnet-test-support` directory so debug-only test seams
remain isolated from production binaries. Release symbols, signing, security
checks, feature selections, and test commands are unchanged.

The official entry point accepts the queue-time `rustBuildPool` parameter.
It selects the pool for the four Windows/Linux Rust build jobs and the two
Windows/Linux lint jobs. macOS keeps its Apple Silicon hosted agents; packaging,
.NET, SDK integration, and SDL jobs keep their existing pools. The default
remains `Azure-Pipelines-1ESPT-ExDShared` until a dedicated pool is provisioned
and authorized.

For Microsoft engineers configuring a faster pool in 1ES Hub:

| Setting | Starting configuration |
| --- | --- |
| VM SKU | `Standard_D8ads_v5`: 8 vCPUs, 32 GiB RAM, 300 GiB local temporary storage |
| Agent lifecycle | Stateless; do not retain build machines between jobs |
| Images | 1ES-compatible Windows Server 2022 and Ubuntu 22.04 build images, with aliases `windows-latest` and `ubuntu-latest` |
| Azure DevOps scope | Organization `microsoft`, project `Dart`; authorize the official build pipeline |
| Maximum agents | Six to allow the four build and two lint jobs to run concurrently, subject to Azure quota and ADO parallel-job capacity |

The shared pool is fixed at 2 vCPUs and 8 GiB RAM. A YAML demand cannot resize
it: choose the SKU on a dedicated 1ES Hosted Pool resource, then pass its actual
Azure DevOps pool name as `rustBuildPool` when queuing the official build.
Do not select an ARM host SKU for this pool; the ARM64 Windows/Linux build legs
currently cross-compile on x64 agents.

Start with the existing OS versions to avoid mixing a hardware change with an
image upgrade. Compare total job duration and queue time on the same commit,
and check free space on the agent work disk. Increasing pool capacity cannot
overcome an organization's ADO parallel-job limit.

Internal references: [E+D shared pool](https://eng.ms/docs/coreai/devdiv/one-engineering-system-1es/1es-docs/1es-pipeline-templates/migration/ed-sharedpool),
[1ES Hosted Pool onboarding](https://eng.ms/docs/coreai/devdiv/one-engineering-system-1es/1es-docs/1es-hosted-azure-devops-pools/createpoolimage),
[supported demands](https://eng.ms/docs/coreai/devdiv/one-engineering-system-1es/1es-docs/1es-hosted-azure-devops-pools/demands),
and [Rust cache options](https://eng.ms/docs/coreai/devdiv/one-engineering-system-1es/1es-docs/1es-pipeline-templates/features/buildworkflows/rust#cache-options).
The VM specifications are also documented in the public
[Azure Dadsv5 reference](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/general-purpose/dadsv5-series).

### Rust crates.io release

`1ES.Release.Crates.yml` is a manual pipeline. `dryRun` defaults to `true` and
runs `cargo publish --dry-run` from any ref after creating the release archives.
Publishing through ESRP requires an immutable `refs/tags/v*` tag and explicitly
setting `dryRun` to `false`.

- `mxc-sdk` uses the workspace version and complete crates.io metadata.
- `Invoke-CratePackage.ps1` verifies that `mxc-sdk` is the only Rust release
  crate and packages it into the release artifact.
- Azure Pipelines packages `mxc-sdk` once using the private Rust toolchain and
  dependency feed.
- The publish template owns the crate list; release operators cannot override
  it at queue time. GitHub CI rejects any difference from the validated list.
- Every release runs Cargo's crates.io dry run before the publish job submits
  the package through ESRP.

### PR Pipelines
- GitHub Actions runs the PR validation build automatically on every pull
  request — it mirrors the ADO build stages on native hardware for faster
  developer iteration.
- The ADO pipeline can also be triggered on PRs via `/azp run`
  (see [pull request builds](../docs/development/build-and-test/pull-requests.md)) when reviewers want
  to run the official build against a change before merge.
