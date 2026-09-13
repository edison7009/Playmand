"""Exercise installation in disposable projects, without changing host settings."""

import subprocess
import os
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "install_skill.py"


def files(root):
    return {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}


class InstallTests(unittest.TestCase):
    def setUp(self):
        self.sandbox = tempfile.TemporaryDirectory(prefix="playmand-install-")
        self.addCleanup(self.sandbox.cleanup)
        self.project = Path(self.sandbox.name) / "中文 game project"
        self.project.mkdir()

    def run_install(self, target="both", project=None):
        return subprocess.run(
            [sys.executable, "-X", "utf8", str(SCRIPT), "--project",
             str(project or self.project), "--target", target],
            cwd=self.sandbox.name, capture_output=True, encoding="utf-8",
        )

    def test_both_hosts_and_idempotent_install(self):
        original = self.project / "game.txt"
        original.write_text("player work", encoding="utf-8")
        result = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = files(ROOT / "skills" / "playmand")
        for host in (".agents", ".claude"):
            self.assertEqual(files(self.project / host / "skills" / "playmand"), expected)
        before = files(self.project)
        result = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(files(self.project), before)

    def test_conflict_is_detected_before_other_host_is_installed(self):
        existing = self.project / ".claude" / "skills" / "playmand"
        existing.mkdir(parents=True)
        (existing / "SKILL.md").write_text("user customized skill", encoding="utf-8")
        before = files(self.project)
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("nothing overwritten", result.stderr)
        self.assertEqual(files(self.project), before)
        self.assertFalse((self.project / ".agents").exists())

    def test_single_host(self):
        result = self.run_install("claude")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.project / ".claude" / "skills" / "playmand" / "SKILL.md").is_file())
        self.assertFalse((self.project / ".agents").exists())

    def test_missing_project_is_not_created(self):
        missing = self.project / "missing"
        result = self.run_install(project=missing)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(missing.exists())

    def test_parent_file_conflict_is_detected_before_copy(self):
        conflict = self.project / ".claude"
        conflict.write_text("user file", encoding="utf-8")
        before = files(self.project)
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Parent is not a directory", result.stderr)
        self.assertEqual(files(self.project), before)
        self.assertFalse((self.project / ".agents").exists())

    def test_non_utf8_console_does_not_interrupt_install(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--project", str(self.project)],
            cwd=self.sandbox.name, capture_output=True, encoding="cp1252",
            env={**os.environ, "PYTHONIOENCODING": "cp1252"},
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = files(ROOT / "skills" / "playmand")
        for host in (".agents", ".claude"):
            self.assertEqual(files(self.project / host / "skills" / "playmand"), expected)

    def test_user_scope_uses_home_without_changing_real_user_files(self):
        # Mock home in the child process instead of changing any host configuration.
        code = (
            "import pathlib, runpy, sys; from unittest.mock import patch; "
            "script, home = sys.argv[1:]; "
            "sys.argv = [script, '--user', '--target', 'both']; "
            "mock = patch.object(pathlib.Path, 'home', return_value=pathlib.Path(home)); "
            "mock.start(); runpy.run_path(script, run_name='__main__')"
        )
        result = subprocess.run(
            [sys.executable, "-X", "utf8", "-c", code, str(SCRIPT), str(self.project)],
            cwd=self.sandbox.name, capture_output=True, encoding="utf-8",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = files(ROOT / "skills" / "playmand")
        for host in (".agents", ".claude"):
            self.assertEqual(files(self.project / host / "skills" / "playmand"), expected)


if __name__ == "__main__":
    unittest.main()
