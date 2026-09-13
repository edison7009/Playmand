"""Build the distributable skill from an explicit public-file allowlist."""

import hashlib
import json
from pathlib import Path
import zipfile


ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.2.1"


def public_files():
    paths = [ROOT / "README.md", ROOT / "LICENSE", ROOT / "VALIDATION.md"]
    for folder in ("skills", "docs", "scripts", "tests"):
        paths.extend(sorted(
            path for path in (ROOT / folder).rglob("*")
            if path.is_file() and not {"__pycache__", "target"}.intersection(path.relative_to(ROOT).parts)
            and path.suffix not in (".pyc", ".pyo")
        ))
    return paths


def build(output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    archive_path = output / f"playmand-{VERSION}.zip"
    paths = public_files()
    with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in paths:
            archive.write(path, f"playmand-{VERSION}/" + path.relative_to(ROOT).as_posix())
    receipt = {
        "package": archive_path.name,
        "sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
        "files": [path.relative_to(ROOT).as_posix() for path in paths],
    }
    (output / "verification.json").write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8",
    )
    return archive_path


if __name__ == "__main__":
    print(build(ROOT / "dist").name)
