# PR 1476: macOS PTY investigation

This branch adds diagnostics only; it does not modify the proposed production
fix. macOS results are **pending**. Run on macOS 15 or later with Python 3.9+,
Xcode command-line tools, and the Rust toolchain pinned in
`src/rust-toolchain.toml`. Cargo uses the repository's existing dependency setup.
No elevation or pipeline run is needed.

From a clean checkout of `user/bbonaby/1476-testing`:

```sh
python3 scripts/diagnostics/pr1476/run.py --commit-results
```

The runner writes `results/<UTC timestamp>/report.json` and per-case logs beside
this README, then commits **only that run's directory**. It never pushes.
Review the logs for local paths or sensitive diagnostics before pushing:

```sh
git push origin user/bbonaby/1476-testing
```

Omit `--commit-results` to inspect and commit the files yourself. A failed run
also saves its report and can be committed. Exit code 1 means an incomplete
investigation, a failed head probe, or an OS-control output discrepancy; inspect
the JSON, not just the exit code. Expected base regression failures do not make
the runner fail. Allow several minutes for the initial two builds. Each build
has a 20-minute limit, each Rust probe a separate 30-second watchdog. Temporary
worktrees and build artifacts are removed afterwards.

## What the source says

The following observations refer to PR head
`6a2c969e5fe43dace38b4ba3ef617056fd48e420`, compared with its merge base
`2148445e0fa9c75e68e0b12ae1fbe83d3d40a711`, not a moving `main`.

- **A real fix:** `src/mxc-sdk/src/core/mxc_pty/mod.rs:506-533` changes the
  output bridge to keep draining/discarding after its consumer closes.
  Previously the first write error stopped the drain. The same bulk-output,
  closed-consumer probe runs against both revisions; base failing and head
  passing would demonstrate this fix independently of the PR's rewritten tests.
- **Not a new automatic-buffering guarantee:** `src/mxc-sdk/src/sandbox.rs:544-597`
  already drains unclaimed output in `MxcPtyProcess::wait`. After
  `try_clone_reader` or native stream transfer, the caller owns consumption.
  The Seatbelt equivalent is
  `src/mxc-sdk/src/backends/seatbelt/common/seatbelt_runner.rs:864-939`.
  Thus the added API-doc warning is too broad without saying **claimed output**.
- **Two distinct blocking mechanisms:** bulk output can fill a finite terminal
  or native pipe queue. Separately, Apple's XNU
  [`proc_exit` path](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_exit.c)
  calls `ttywait` for a session leader's controlling terminal; `ttywait` in
  [`tty.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/tty.c)
  waits on pending output while connected. This supports an exit-time drain
  explanation even for small output. It is more precise than attributing
  everything to the final secondary `close`. These are upstream sources, not
  proof of which path a particular macOS build takes.
- The FIFO-to-pipe replacement and descriptor-inheritance changes are separate
  changes. Successful native-stream probes exercise EOF and output transfer,
  but this investigation does **not** certify every descriptor-race,
  cancellation, SDK-language, or backend behavior in the PR.

Concurrent consumption is appropriate for a streaming API: transparently
draining a caller-owned reader would race the caller; transparently retaining
all output would require a bounded-buffer/spooling policy. Documentation alone
would not fix the closed-consumer bridge defect, but this PR also changes that
implementation. The provisional verdict is **a concrete fix for that defect,
not proof that every macOS PTY hang is fixed**.

## How to interpret the report

The Python OS controls use no MXC code. They compare 10 bytes on a controlling
PTY, the same output without a controlling terminal, and 1 MiB with delayed
versus concurrent draining. A separate 10-byte case sends SIGKILL before
draining. `write_completed_before_drain=true` plus
`exit_before_drain=null` distinguishes finishing the write from becoming
reapable. `exit_after_kill_before_drain=null` records only the bounded observation
that SIGKILL had not made the process reapable yet, not that it is universally
ineffective. Output is checked byte-for-byte and hashed; the primary is closed
before timeout cleanup waits for the child.

Both Rust revisions receive the **same** `probe.rs` integration fixture:

| Probe | Question |
|---|---|
| `direct_small_delayed_reader` | Does an MXC-attached child remain unreaped before reading just 10 bytes? |
| `direct_bulk_delayed_reader` | Does starting to read release bulk-output blocking? |
| `direct_bulk_concurrent_reader` | Are all bytes and exit code 23 preserved with concurrent consumption? |
| `native_bulk_delayed_reader` | Does the native pipe propagate backpressure until read? |
| `native_bulk_concurrent_reader` | Does the native bridge preserve all bytes, EOF, and exit status? |
| `native_closed_consumer` | Does closing the output consumer stop the child from exiting? Expected base failure, head success. |
| `sdk_unclaimed_bulk_wait` | Does public Rust `wait()` drain untouched output without caller intervention? |
| `sdk_claimed_small_delayed_read` | Does claiming a reader change whether `wait()` completes before reading? |
| `sdk_claimed_bulk_concurrent_read` | Does public Rust `wait()` succeed with caller-owned concurrent consumption? |

Delayed-read probes observe a one-second window and then start reading; they
do not intentionally leave a hang running indefinitely. Rust output probes
check exactly N zero bytes plus a final marker and nonzero exit code 23. That
catches truncation or errors disguised as success. Logs preserve assertions,
errors, and observations; JSON records actual commands, exit codes, watchdogs,
revision pins, fixture hash, host/toolchain details, and child cleanup. A
watchdog/error is evidence to investigate, not a passing test. Stop if cleanup
reports a surviving child. Results from these Rust APIs do not establish .NET
or Node SDK parity.

Host-independent runner checks (these do not execute MXC):

```sh
python3 -m unittest discover -s scripts/diagnostics/pr1476 -p test_runner.py
rustfmt --edition 2024 --check scripts/diagnostics/pr1476/probe.rs
```
