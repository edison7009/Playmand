"""Copy the shared Playmand skill into an existing game project; never overwrite."""

import argparse
import shutil
import sys
from pathlib import Path


def snapshot(root):
    return {
        path.relative_to(root): path.read_bytes()
        for path in root.rglob("*")
        if path.is_file() and not {"target", "__pycache__"}.intersection(path.relative_to(root).parts)
        and path.suffix not in (".pyc", ".pyo")
    }


def install(project, target):
    project = Path(project).expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError(f"Project is not a directory: {project}")
    source = Path(__file__).resolve().parents[1] / "skills" / "playmand"
    if not (source / "SKILL.md").is_file():
        raise ValueError(f"Skill source is missing: {source}")
    hosts = {"codex": ".agents", "claude": ".claude"}
    selected = hosts if target == "both" else {target: hosts[target]}
    destinations = [project / folder / "skills" / "playmand" for folder in selected.values()]
    expected = snapshot(source)

    # Check every destination before copying either host's files.
    for destination in destinations:
        resolved = destination.resolve()
        if project not in resolved.parents:
            raise ValueError(f"Destination leaves project: {destination}")
        if destination.is_symlink():
            raise ValueError(f"Refusing linked destination: {destination}")
        for parent in destination.parents:
            if parent == project:
                break
            if parent.exists() and not parent.is_dir():
                raise ValueError(f"Parent is not a directory: {parent}")
        if destination.exists() and (
            not destination.is_dir() or snapshot(destination) != expected
        ):
            raise ValueError(f"Different content already exists; nothing overwritten: {destination}")

    for destination in destinations:
        if destination.exists():
            print(f"Unchanged: {destination}")
        else:
            shutil.copytree(source, destination,
                            ignore=shutil.ignore_patterns("target", "__pycache__", "*.pyc", "*.pyo"))
            print(f"Installed: {destination}")


def main():
    for stream in (sys.stdout, sys.stderr):
        stream.reconfigure(errors="backslashreplace")
    parser = argparse.ArgumentParser(description=__doc__)
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument("--project", help="Existing game project directory")
    scope.add_argument("--user", action="store_true", help="Install for all projects of this user")
    parser.add_argument("--target", choices=("codex", "claude", "both"), default="both")
    args = parser.parse_args()
    try:
        install(Path.home() if args.user else args.project, args.target)
    except (OSError, ValueError) as error:
        parser.exit(1, f"Installation failed: {error}\n")


if __name__ == "__main__":
    main()
