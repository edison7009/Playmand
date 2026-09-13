"""Verify that a downloaded archive is complete, portable, and installable."""

import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
import zipfile


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("build_package", ROOT / "scripts" / "build_package.py")
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class PackageTests(unittest.TestCase):
    def test_archive_is_self_contained_and_installs_both_hosts(self):
        with tempfile.TemporaryDirectory(prefix="playmand-package-") as scratch:
            scratch = Path(scratch)
            package = builder.build(scratch / "dist")
            receipt = json.loads((package.parent / "verification.json").read_text(encoding="utf-8"))
            self.assertEqual(receipt["sha256"], hashlib.sha256(package.read_bytes()).hexdigest())
            with zipfile.ZipFile(package) as archive:
                self.assertIsNone(archive.testzip())
                for name in archive.namelist():
                    self.assertFalse(Path(name).is_absolute())
                    self.assertNotIn("..", Path(name).parts)
                    self.assertNotIn("work", Path(name).parts)
                    self.assertNotIn("__pycache__", Path(name).parts)
                    self.assertFalse(Path(name).name.startswith("Playmand_"))
                archive.extractall(scratch / "unpacked")
            extracted = scratch / "unpacked" / f"playmand-{builder.VERSION}"
            for path in builder.public_files():
                self.assertEqual(path.read_bytes(), (extracted / path.relative_to(ROOT)).read_bytes())
            for path in extracted.rglob("*.md"):
                for link in re.findall(r"\]\(([^)]+)\)", path.read_text(encoding="utf-8")):
                    if link.startswith(("https://", "http://", "#")):
                        continue
                    target = (path.parent / link.split("#", 1)[0]).resolve()
                    self.assertIn(extracted.resolve(), target.parents)
                    self.assertTrue(target.is_file(), f"Broken link in {path}: {link}")
            self.assertEqual((extracted / "LICENSE").read_bytes(),
                             (extracted / "skills" / "playmand" / "LICENSE").read_bytes())
            project = scratch / "游戏 project"
            project.mkdir()
            result = subprocess.run(
                [sys.executable, "-X", "utf8", str(extracted / "scripts" / "install_skill.py"),
                 "--project", str(project)], cwd=scratch, capture_output=True, encoding="utf-8",
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            source = extracted / "skills" / "playmand"
            for host in (".agents", ".claude"):
                installed = project / host / "skills" / "playmand"
                for path in source.rglob("*"):
                    if path.is_file():
                        self.assertEqual(path.read_bytes(), (installed / path.relative_to(source)).read_bytes())
