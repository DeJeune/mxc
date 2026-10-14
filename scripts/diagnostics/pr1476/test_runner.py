# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import run


class RunnerTests(unittest.TestCase):
    def test_git_failure_includes_actionable_stderr(self):
        error = subprocess.CalledProcessError(
            128, ["git", "cat-file"], stderr="missing pinned commit"
        )
        with patch.object(run.subprocess, "check_output", side_effect=error):
            with self.assertRaisesRegex(RuntimeError, "missing pinned commit"):
                run.git("cat-file", "-e", run.BASE)

    def test_pins_and_fixture_cases(self):
        self.assertEqual(run.ROOT, Path(__file__).resolve().parents[3])
        self.assertEqual(len(run.BASE), 40)
        self.assertEqual(len(run.HEAD), 40)
        fixture = (run.HERE / "probe.rs").read_text()
        for case in run.CASES:
            self.assertIn(f"fn {case}()", fixture)

    def test_artifact_discovery_ignores_other_json_and_diagnostics(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "build.log"
            log.write_text("compiler diagnostic\n{}\n" + json.dumps({
                "reason": "compiler-artifact",
                "target": {"name": "pr1476_probe"},
                "executable": "/tmp/probe",
            }) + "\n")
            self.assertEqual(run.test_binary(log), "/tmp/probe")

    def test_missing_artifact_is_an_error(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "build.log"
            log.write_text("{}\n")
            with self.assertRaises(RuntimeError):
                run.test_binary(log)

    def test_child_exit_status_and_observations_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            result = run.run_command(
                [sys.executable, "-c",
                 "print('PR1476_OBSERVATION exact=true', flush=True);raise SystemExit(23)"],
                directory, 10, Path(directory) / "child.log",
            )
            self.assertEqual(result["exit_code"], 23)
            self.assertFalse(result["timed_out"])
            self.assertEqual(result["observations"], ["PR1476_OBSERVATION exact=true"])
            self.assertEqual(len(result["log_sha256"]), 64)

    def test_timeout_kills_and_reaps_before_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(run.subprocess, "Popen") as spawn, \
                    patch.object(run, "kill_group") as kill, \
                    patch.object(run, "cleanup_registered", return_value={"remaining": False}):
                process = spawn.return_value
                process.pid = 123
                process.wait.side_effect = [subprocess.TimeoutExpired("fixture", 30), -9]
                result = run.run_command(
                    ["fixture"], directory, 30, Path(directory) / "timeout.log",
                    pid_file=Path(directory) / "child.pid",
                )
                kill.assert_called_once_with(123)
                self.assertTrue(result["timed_out"])
                self.assertEqual(result["exit_code"], -9)
                self.assertEqual(process.wait.call_count, 2)

    def test_report_is_readable_json(self):
        with tempfile.TemporaryDirectory() as directory:
            report = {"status": "incomplete", "errors": ["build failed"]}
            run.save_report(Path(directory), report)
            self.assertEqual(json.loads((Path(directory) / "report.json").read_text()), report)
            self.assertFalse((Path(directory) / "report.json.tmp").exists())

    def test_commit_refuses_unrelated_staged_changes(self):
        with patch.object(run, "git", side_effect=[run.BRANCH, "unrelated.txt"]) as git:
            with self.assertRaisesRegex(RuntimeError, "staged changes"):
                run.commit_results(run.HERE / "results" / "fixture")
            self.assertEqual(git.call_count, 2)

    def test_commit_scopes_add_and_commit_to_one_result_directory(self):
        with patch.object(run, "git", side_effect=[run.BRANCH, "", "", ""]) as git:
            run.commit_results(run.HERE / "results" / "fixture")
            added = git.call_args_list[2].args
            committed = git.call_args_list[3].args
            self.assertEqual(added, ("add", "--", "scripts/diagnostics/pr1476/results/fixture"))
            self.assertEqual(committed[-2:], ("--", added[-1]))

    def simulate_run(self, directory, build_fails=False):
        root = Path(directory)
        here = root / "scripts" / "diagnostics" / "pr1476"
        here.mkdir(parents=True)
        (here / "probe.rs").write_bytes((run.HERE / "probe.rs").read_bytes())

        def fake_git(*args):
            if args[0] == "branch":
                return run.BRANCH
            if args[:2] == ("worktree", "add"):
                (Path(args[3]) / "src" / "mxc-sdk" / "tests").mkdir(parents=True)
            return ""

        def fake_command(command, cwd, timeout, log, env=None, pid_file=None):
            code = 0
            if "--no-run" in command:
                log.write_text(json.dumps({
                    "reason": "compiler-artifact",
                    "target": {"name": "pr1476_probe"},
                    "executable": "fixture",
                }) + "\n")
                code = 1 if build_fails else 0
            else:
                log.write_text("running 1 test\nPR1476_OBSERVATION fixture=true\n")
                if "base-native_closed_consumer" in log.name:
                    code = 101
            return {"exit_code": code, "timed_out": False, "log": log.name,
                    "cleanup": {"remaining": False}}

        def fake_control(size, controlling, delayed, kill, scratch):
            return {"exact": True, "eof": True, "exit_after_drain": 23,
                    "kill_before_drain": kill}

        with patch.object(run, "ROOT", root), patch.object(run, "HERE", here), \
                patch.object(run, "git", side_effect=fake_git) as git, \
                patch.object(run, "run_command", side_effect=fake_command), \
                patch.object(run, "os_control", side_effect=fake_control), \
                patch.object(run.sys, "platform", "darwin"), \
                patch.object(run.sys, "argv", ["run.py"]), \
                contextlib.redirect_stdout(io.StringIO()):
            code = run.main()
        reports = list((here / "results").glob("*/report.json"))
        self.assertEqual(len(reports), 1)
        removed = [call for call in git.call_args_list
                   if call.args[:2] == ("worktree", "remove")]
        self.assertEqual(len(removed), 2)
        return code, json.loads(reports[0].read_text())

    def test_complete_flow_preserves_expected_base_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            code, report = self.simulate_run(directory)
            self.assertEqual(code, 0)
            self.assertEqual(report["status"], "completed")
            self.assertEqual(len(report["revisions"][1]["tests"]), len(run.CASES))
            base = report["revisions"][0]["tests"]
            failure = next(test for test in base if test["case"] == "native_closed_consumer")
            self.assertEqual(failure["exit_code"], 101)
            self.assertEqual(report["runner_exit_code"], 0)
            comparison = next(test for test in report["comparison"]
                              if test["case"] == "native_closed_consumer")
            self.assertEqual(comparison["base"]["exit_code"], 101)
            self.assertEqual(comparison["head"]["exit_code"], 0)

    def test_build_failures_save_report_and_remove_both_worktrees(self):
        with tempfile.TemporaryDirectory() as directory:
            code, report = self.simulate_run(directory, build_fails=True)
            self.assertEqual(code, 1)
            self.assertEqual(len(report["errors"]), 2)
            self.assertTrue(all(not entry["tests"] for entry in report["revisions"]))


if __name__ == "__main__":
    unittest.main()
