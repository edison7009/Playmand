"""Locate resolved Bevy versions and sources without updating a game's lockfile."""

import argparse
import json
from pathlib import Path
import subprocess


def summarize(metadata):
    if metadata.get("resolve") is None:
        raise ValueError("Dependency graph missing; cargo metadata must run without --no-deps.")
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    packages = {package["id"]: package for package in metadata["packages"]}
    members = metadata["workspace_members"]
    reachable = set()
    pending = list(members)
    while pending:
        package_id = pending.pop()
        if package_id in reachable:
            continue
        reachable.add(package_id)
        pending.extend(nodes[package_id]["dependencies"])
    crates = []
    for package_id in reachable:
        package = packages[package_id]
        name = package["name"]
        if name != "bevy" and not name.startswith("bevy_"):
            continue
        version = package["version"]
        crates.append({
            "name": name, "version": version, "id": package_id,
            "source": package.get("source"),
            "source_dir": str(Path(package["manifest_path"]).parent),
            "rust_version": package.get("rust_version"),
            "features": sorted(nodes[package_id]["features"]),
            # Patched/path/git crates can differ from published documentation.
            "published_docs": f"https://docs.rs/{name}/{version}/{name}/",
        })
    return {
        "workspace_root": metadata["workspace_root"],
        "scope": "Resolved dependencies reachable from all workspace members; bevy* names may include third-party crates.",
        "workspace_members": [packages[item]["name"] for item in members],
        "bevy_crates": sorted(crates, key=lambda item: (item["name"], item["version"], item["id"])),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", required=True, type=Path)
    parser.add_argument("--filter-platform", help="Cargo target triple for cross compilation")
    args = parser.parse_args()
    manifest = args.manifest_path.resolve()
    if not manifest.is_file():
        parser.error(f"Manifest does not exist: {manifest}")
    command = ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
               "--manifest-path", str(manifest)]
    if args.filter_platform:
        command.extend(["--filter-platform", args.filter_platform])
    try:
        result = subprocess.run(command, cwd=manifest.parent, capture_output=True, encoding="utf-8")
    except FileNotFoundError:
        parser.exit(1, "cargo not found in PATH; use the project's Rust toolchain environment.\n")
    if result.returncode:
        parser.exit(result.returncode, result.stderr)
    report = summarize(json.loads(result.stdout))
    report["filter_platform"] = args.filter_platform
    # ASCII JSON also works on Windows consoles with legacy encodings.
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
