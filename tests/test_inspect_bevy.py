"""Check resolution reporting where reading a dependency string would be wrong."""

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "inspect_bevy", ROOT / "skills/playmand/scripts/inspect_bevy.py")
inspector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inspector)


def graph():
    packages = []
    nodes = []
    for identity, name, version, dependencies in (
        ("game", "game", "0.1.0", ["engine", "plugin"]),
        ("engine", "bevy", "0.19.1", ["ecs-new"]),
        ("plugin", "physics", "1.0.0", ["ecs-old"]),
        ("ecs-new", "bevy_ecs", "0.19.1", []),
        ("ecs-old", "bevy_ecs", "0.18.0", []),
        ("unused", "bevy", "0.17.0", []),
    ):
        packages.append({"id": identity, "name": name, "version": version,
                         "manifest_path": f"/cache/{identity}/Cargo.toml",
                         "source": "registry+https://example.test/index"})
        nodes.append({"id": identity, "dependencies": dependencies, "features": ["std"]})
    return {"workspace_root": "/game", "workspace_members": ["game"],
            "packages": packages, "resolve": {"nodes": nodes}}


class InspectTests(unittest.TestCase):
    def test_reports_transitive_versions_and_ignores_unreachable_packages(self):
        result = inspector.summarize(graph())
        crates = result["bevy_crates"]
        self.assertEqual([(p["name"], p["version"]) for p in crates],
                         [("bevy", "0.19.1"), ("bevy_ecs", "0.18.0"), ("bevy_ecs", "0.19.1")])
        self.assertEqual(crates[0]["features"], ["std"])
        self.assertEqual(Path(crates[0]["source_dir"]), Path("/cache/engine"))
        self.assertEqual(crates[0]["published_docs"], "https://docs.rs/bevy/0.19.1/bevy/")

    def test_keeps_path_dependency_identity_even_at_the_same_version(self):
        metadata = graph()
        local = dict(metadata["packages"][1], id="local-engine", source=None,
                     manifest_path="/game/vendor/bevy/Cargo.toml")
        metadata["packages"].append(local)
        metadata["resolve"]["nodes"].append(
            {"id": "local-engine", "dependencies": [], "features": []})
        metadata["resolve"]["nodes"][0]["dependencies"].append("local-engine")
        crates = inspector.summarize(metadata)["bevy_crates"]
        engines = [item for item in crates if item["name"] == "bevy"]
        self.assertEqual(len(engines), 2)
        self.assertEqual(engines[1]["id"], "local-engine")
        self.assertIsNone(engines[1]["source"])

    def test_requires_full_dependency_graph(self):
        metadata = graph()
        metadata["resolve"] = None
        with self.assertRaisesRegex(ValueError, "without --no-deps"):
            inspector.summarize(metadata)
