#!/usr/bin/env python3
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Bounded, pinned base/head investigation of PR 1476; Python stdlib only."""

import argparse
import datetime
import errno
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import tempfile
import time

BASE = "2148445e0fa9c75e68e0b12ae1fbe83d3d40a711"
HEAD = "6a2c969e5fe43dace38b4ba3ef617056fd48e420"
BRANCH = "user/bbonaby/1476-testing"
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CASES = (
    "direct_small_delayed_reader",
    "direct_bulk_delayed_reader",
    "direct_bulk_concurrent_reader",
    "native_bulk_delayed_reader",
    "native_bulk_concurrent_reader",
    "native_closed_consumer",
    "sdk_unclaimed_bulk_wait",
    "sdk_claimed_small_delayed_read",
    "sdk_claimed_bulk_concurrent_read",
)


def git(*args):
    try:
        return subprocess.check_output(
            ["git", "-C", str(ROOT), *args], text=True, stderr=subprocess.PIPE
        ).strip()
    except subprocess.CalledProcessError as error:
        raise RuntimeError(f"git {' '.join(args)} failed: {error.stderr.strip()}") from error


def kill_group(pid):
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def cleanup_registered(path):
    """The fixture records only a currently owned, unreaped child session."""
    if not path.exists():
        return {"registered": False, "remaining": False}
    pid = int(path.read_text())
    if pid <= 1:
        raise RuntimeError(f"Invalid registered child pid: {pid}")
    try:
        if os.getpgid(pid) != pid:
            raise RuntimeError(f"Registered child {pid} is not its own process group")
        kill_group(pid)
    except ProcessLookupError:
        return {"registered": True, "pid": pid, "remaining": False}
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return {"registered": True, "pid": pid, "remaining": False}
        time.sleep(0.05)
    return {"registered": True, "pid": pid, "remaining": True}


def run_command(command, cwd, timeout, log, env=None, pid_file=None):
    """Redirect to disk: even a descendant holding stdout cannot wedge communicate."""
    started = time.monotonic()
    timed_out = False
    with log.open("wb") as output:
        process = subprocess.Popen(
            command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
            stdout=output, stderr=subprocess.STDOUT, start_new_session=True,
        )
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            kill_group(process.pid)
            code = process.wait(timeout=5)
        except BaseException:
            kill_group(process.pid)
            process.wait(timeout=5)
            if pid_file:
                cleanup_registered(pid_file)
            raise
    cleanup = cleanup_registered(pid_file) if pid_file else None
    raw = log.read_bytes()
    return {
        "command": [str(arg) for arg in command],
        "exit_code": code,
        "timed_out": timed_out,
        "seconds": round(time.monotonic() - started, 3),
        "log": log.name,
        "log_sha256": hashlib.sha256(raw).hexdigest(),
        "observations": [
            line for line in raw.decode("utf-8", errors="replace").splitlines()
            if "PR1476_OBSERVATION" in line
        ],
        "cleanup": cleanup,
    }


def os_control(size, controlling, delayed, kill_before_drain, scratch):
    """No MXC: compare a controlling PTY, a non-controlling PTY, and delayed drain."""
    import fcntl
    import pty
    import termios

    primary, secondary = pty.openpty()
    os.set_blocking(primary, False)
    written = scratch / "written"
    written.unlink(missing_ok=True)
    body = (
        "import os,fcntl,termios,sys;"
        + ("fcntl.ioctl(0,termios.TIOCSCTTY,0);" if controlling else "")
        + "data=b'x'*int(sys.argv[1]);"
        # os.write may be partial, so use an unbuffered file's write loop.
        + "\nwhile data:\n n=os.write(1,data)\n data=data[n:]\n"
        + "open(sys.argv[2],'w').write('written');os._exit(23)"
    )
    process = None
    output = bytearray()
    eof = False
    started = time.monotonic()
    try:
        process = subprocess.Popen(
            [sys.executable, "-c", body, str(size), str(written)],
            stdin=secondary, stdout=secondary, stderr=secondary,
            start_new_session=True,
        )
        os.close(secondary)
        secondary = None
        if delayed:
            # This is a bounded observation, not proof of an indefinite wait.
            time.sleep(1)
        before = process.poll()
        write_completed = written.exists()
        after_kill = None
        if kill_before_drain:
            process.kill()
            time.sleep(0.5)
            after_kill = process.poll()
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            if not eof:
                try:
                    chunk = os.read(primary, 65536)
                    if chunk:
                        output.extend(chunk)
                    else:
                        eof = True
                except BlockingIOError:
                    pass
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    eof = True
            if process.poll() is not None and eof:
                break
            time.sleep(0.001)
        result = {
            "size": size, "controlling_terminal": controlling, "delayed": delayed,
            "kill_before_drain": kill_before_drain,
            "write_completed_before_drain": write_completed,
            "exit_before_drain": before, "exit_after_kill_before_drain": after_kill,
            "exit_after_drain": process.poll(), "eof": eof,
            "bytes": len(output), "exact": output == b"x" * size,
            "sha256": hashlib.sha256(output).hexdigest(),
            "seconds": round(time.monotonic() - started, 3),
        }
    finally:
        # Release the primary before waiting: SIGKILL alone is not our watchdog.
        os.close(primary)
        if secondary is not None:
            os.close(secondary)
        if process is not None:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
    result["cleanup"] = "reaped"
    return result


def test_binary(log):
    binaries = []
    for line in log.read_text(errors="replace").splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (message.get("reason") == "compiler-artifact"
                and message.get("target", {}).get("name") == "pr1476_probe"
                and message.get("executable")):
            binaries.append(message["executable"])
    if len(binaries) != 1:
        raise RuntimeError(f"Expected one compiled diagnostic binary, found {binaries}")
    return binaries[0]


def save_report(directory, report):
    temporary = directory / "report.json.tmp"
    temporary.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    temporary.replace(directory / "report.json")


def commit_results(directory):
    if git("branch", "--show-current") != BRANCH:
        raise RuntimeError(f"Results can only be committed on {BRANCH}")
    if git("diff", "--cached", "--name-only"):
        raise RuntimeError("Refusing to commit: the index already contains staged changes")
    relative = directory.relative_to(ROOT).as_posix()
    git("add", "--", relative)
    git(
        "commit", "-m", "Record macOS PR 1476 diagnostic results",
        "-m", "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\n"
        "Copilot-Session: 0ec340f6-3fea-4448-af07-442ecf62762d",
        "--", relative,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--commit-results", action="store_true",
                        help="Commit only this run's results; never push")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("Run the investigation on macOS; runner unit tests work on other hosts.")
    if git("branch", "--show-current") != BRANCH:
        parser.error(f"Switch to {BRANCH} first.")
    if git("status", "--porcelain"):
        parser.error("Start with a clean checkout so fixture provenance is unambiguous.")
    for revision in (BASE, HEAD):
        try:
            git("cat-file", "-e", revision + "^{commit}")
        except RuntimeError as error:
            parser.error(f"{error}\nFetch the testing branch with full history, then retry.")
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    directory = HERE / "results" / stamp
    directory.mkdir(parents=True)
    report = {
        "schema": 1, "started_utc": stamp, "status": "running",
        "harness_commit": git("rev-parse", "HEAD"),
        "fixture_sha256": hashlib.sha256((HERE / "probe.rs").read_bytes()).hexdigest(),
        "platform": platform.platform(), "machine": platform.machine(),
        "python": platform.python_version(), "base": BASE, "head": HEAD,
        "os_controls": [], "revisions": [], "errors": [],
    }
    save_report(directory, report)
    try:
        with tempfile.TemporaryDirectory(prefix="mxc-pr1476-") as temporary:
            scratch = Path(temporary)
            for size, controlling, delayed, kill in (
                (10, True, True, False), (10, False, True, False),
                (10, True, True, True), (1_048_576, True, True, False),
                (1_048_576, True, False, False),
            ):
                print(f"OS control: size={size}, controlling={controlling}, kill={kill}", flush=True)
                report["os_controls"].append(os_control(size, controlling, delayed, kill, scratch))
                save_report(directory, report)
            for label, revision in (("base", BASE), ("head", HEAD)):
                worktree = scratch / label
                entry = {"label": label, "revision": revision, "tests": []}
                report["revisions"].append(entry)
                git("worktree", "add", "--detach", str(worktree), revision)
                try:
                    shutil.copyfile(HERE / "probe.rs",
                                    worktree / "src" / "mxc-sdk" / "tests" / "pr1476_probe.rs")
                    env = os.environ.copy()
                    env["CARGO_TARGET_DIR"] = str(scratch / "target")
                    src = worktree / "src"
                    entry["toolchain"] = run_command(
                        ["rustc", "--version"], src, 120,
                        directory / f"{label}-toolchain.log", env,
                    )
                    if (entry["toolchain"]["exit_code"] != 0
                            or entry["toolchain"]["timed_out"]):
                        report["errors"].append(f"{label}: Rust toolchain unavailable; see log")
                        continue
                    entry["build"] = run_command(
                        ["cargo", "test", "--locked", "-p", "mxc-sdk",
                         "--test", "pr1476_probe", "--no-run", "--message-format=json"],
                        src, 1200, directory / f"{label}-build.log", env,
                    )
                    save_report(directory, report)
                    if entry["build"]["exit_code"] != 0 or entry["build"]["timed_out"]:
                        report["errors"].append(f"{label}: build failed; see build log")
                        continue
                    binary = test_binary(directory / f"{label}-build.log")
                    for case in CASES:
                        print(f"{label}: {case}", flush=True)
                        pid_file = scratch / f"{label}-{case}.pid"
                        env["MXC_PR1476_PID_FILE"] = str(pid_file)
                        result = run_command(
                            [binary, case, "--exact", "--nocapture", "--test-threads=1"],
                            src, 30, directory / f"{label}-{case}.log", env, pid_file,
                        )
                        result["case"] = case
                        # A test filter silently matching zero tests is not evidence.
                        text = (directory / result["log"]).read_text(errors="replace")
                        result["one_test_ran"] = "running 1 test" in text
                        entry["tests"].append(result)
                        save_report(directory, report)
                        if result["cleanup"] and result["cleanup"].get("remaining"):
                            raise RuntimeError(f"{label}/{case}: child survived cleanup; stop")
                finally:
                    # Only this disposable, detached worktree contains the injected fixture.
                    git("worktree", "remove", "--force", str(worktree))
        report["status"] = "completed"
    except Exception as error:
        report["status"] = "incomplete"
        report["errors"].append(f"{type(error).__name__}: {error}")
    finally:
        if report["status"] == "running":
            report["status"] = "interrupted"
        save_report(directory, report)
    # Base regression failures are expected evidence, not runner failures.
    head = next((entry for entry in report["revisions"] if entry["label"] == "head"), {})
    valid = (
        report["status"] == "completed" and not report["errors"]
        and len(report["revisions"]) == 2
        and all(len(entry["tests"]) == len(CASES)
                and all(test["one_test_ran"] for test in entry["tests"])
                for entry in report["revisions"])
        and len(head.get("tests", [])) == len(CASES)
        and all(test["exit_code"] == 0 and not test["timed_out"] and test["one_test_ran"]
                for test in head.get("tests", []))
        and all(control["exact"] and control["eof"] and control["exit_after_drain"] == 23
                for control in report["os_controls"]
                if not control["kill_before_drain"])
    )
    report["runner_exit_code"] = 0 if valid else 1
    report["comparison"] = []
    indexed = {
        entry["label"]: {test["case"]: test for test in entry["tests"]}
        for entry in report["revisions"]
    }
    for case in CASES:
        comparison = {"case": case}
        for label in ("base", "head"):
            test = indexed.get(label, {}).get(case)
            comparison[label] = (
                {"exit_code": test["exit_code"], "timed_out": test["timed_out"],
                 "one_test_ran": test["one_test_ran"]} if test else None
            )
        report["comparison"].append(comparison)
    save_report(directory, report)
    print(f"Report: {directory.relative_to(ROOT) / 'report.json'}")
    if args.commit_results:
        commit_results(directory)
        print("Results committed locally. Review the report/logs before git push.")
    return 0 if valid else 1


if __name__ == "__main__":
    sys.exit(main())
