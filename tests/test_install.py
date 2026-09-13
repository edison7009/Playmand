"""Exercise installation in disposable projects, without changing host settings."""

import subprocess
import shutil
import os
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "install_skill.py"


def files(root):
    return {p.relative_to(root): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def source_files():
    return {path: data for path, data in files(ROOT / "skills" / "playmand").items()
            if not {"target", "__pycache__"}.intersection(path.parts)
            and path.suffix not in (".pyc", ".pyo")}


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
        expected = source_files()
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
        expected = source_files()
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
        expected = source_files()
        for host in (".agents", ".claude"):
            self.assertEqual(files(self.project / host / "skills" / "playmand"), expected)

    def test_example_build_outputs_are_not_installed_or_treated_as_edits(self):
        distribution = Path(self.sandbox.name) / "source"
        source = distribution / "skills" / "playmand"
        source.mkdir(parents=True)
        (source / "SKILL.md").write_text("skill", encoding="utf-8")
        generated = source / "examples" / "ecs-patterns" / "target" / "debug"
        generated.mkdir(parents=True)
        (generated / "large-build.exe").write_bytes(b"generated")
        (distribution / "scripts").mkdir()
        installer = distribution / "scripts" / SCRIPT.name
        shutil.copyfile(SCRIPT, installer)
        command = [sys.executable, str(installer), "--project", str(self.project)]
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        installed = self.project / ".agents" / "skills" / "playmand"
        self.assertFalse((installed / "examples/ecs-patterns/target").exists())
        local_build = installed / "examples/ecs-patterns/target"
        local_build.mkdir(parents=True)
        (local_build / "output").write_text("local build", encoding="utf-8")
        result = subprocess.run(command, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((local_build / "output").read_text(encoding="utf-8"), "local build")


if __name__ == "__main__":
    unittest.main()
