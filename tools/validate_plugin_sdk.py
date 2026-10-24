"""Replay the already-built OM9 SDK consumer and an extracted source SDK.

This is Windows/macOS bootstrap/validation tooling, not production plugin code.
It does not build native geometry adapters or certify interactive UI acceptance.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate(root: Path, target: Path, output: Path) -> dict:
    root, target, output = root.resolve(), target.resolve(), output.resolve()
    if target.is_relative_to(root):
        raise ValueError("CARGO_TARGET_DIR must be outside the checkout")
    output.mkdir(parents=True, exist_ok=False)
    records = []
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))

    def run(name, command, *, expected=0, env=None):
        result = subprocess.run([str(x) for x in command], cwd=root,
                                env=env or environment, text=True,
                                capture_output=True, encoding="utf-8", errors="replace")
        (output / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
        (output / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
        records.append({"name": name, "command": [str(x) for x in command],
                        "exit_code": result.returncode, "expected_exit": expected})
        if result.returncode != expected:
            raise RuntimeError(f"{name}: exit {result.returncode}, expected {expected}\n{result.stderr[-4000:]}")
        return result.stdout

    suffix = ".exe" if os.name == "nt" else ""
    tool = target / "debug" / f"om9-plugin{suffix}"
    app = target / "debug" / f"om9{suffix}"
    if os.name == "nt":
        triple, library = "x86_64-pc-windows-msvc", target / "debug/om9_hello_plugin.dll"
    else:
        import platform
        arch = "aarch64" if platform.machine() == "arm64" else "x86_64"
        triple, library = f"{arch}-apple-darwin", target / "debug/libom9_hello_plugin.dylib"
    for path in [tool, app, library]:
        if not path.is_file():
            raise FileNotFoundError(f"build the SDK host/example first: {path}")

    package = output / "hello.om9p"
    manifest = json.loads(run("pack", [tool, "pack", root / "sdk/examples/hello-plugin/descriptor.json",
                                      package, f"{triple}={library}"]))
    inspected = json.loads(run("inspect-package", [tool, "inspect", package]))
    assert inspected == manifest

    project = output / "hello.om9"
    created = json.loads(run("create", [app, "--plugin-run", package, "CreateProfile",
                                       root / "sdk/examples/hello-plugin/arguments.json", project]))
    reopened = json.loads(run("inspect-project", [app, "--inspect", project]))
    assert len(created["objects"]) == len(reopened["objects"]) == 3
    assert created["document_sha256"] == reopened["document_sha256"]
    original_hash = digest(project)
    cancellation = output / "cancel.json"
    cancellation.write_text('{"cancel":true}', encoding="utf-8")
    run("cancel", [app, "--plugin-run-on", package, "CreateProfile", cancellation, project, project], expected=1)
    assert digest(project) == original_hash
    run("unknown-command", [app, "--plugin-run-on", package, "MissingCommand", cancellation, project, project], expected=1)
    assert digest(project) == original_hash

    test_env = dict(environment, OM9_TEST_PLUGIN_PACKAGE=str(package))
    run("controller-tests", ["rtk", "cargo", "test", "-p", "om9-app", "--no-default-features",
                             "--lib", "session::plugins", "-j", "2"])
    run("native-tests", ["rtk", "cargo", "test", "-p", "om9-app", "--no-default-features",
                         "--test", "plugin_native", "-j", "2", "--", "--ignored"], env=test_env)

    bundle = output / "openmatrix9-sdk-0.1.0.zip"
    run("sdk-bundle", [tool, "sdk-bundle", root, bundle])
    # One fresh extraction directory per run: a previous batch's extracted SDK is
    # neither reused nor overwritten, and a shared absolute path cannot block a
    # later batch. `mkdtemp` creates the directory exclusively.
    external = Path(tempfile.mkdtemp(dir=target, prefix=f"sdk-external-source-{output.name}-"))
    with zipfile.ZipFile(bundle) as archive:
        names = archive.namelist()
        assert not any(name.startswith(("crates/om9-core/", "crates/om9-app/", "native/")) for name in names)
        assert all(not Path(name).is_absolute() and ".." not in Path(name).parts for name in names)
        archive.extractall(external)
    standalone_env = dict(environment, CARGO_TARGET_DIR=str(target / "sdk-external-target"))
    run("external-sdk-build", ["rtk", "cargo", "build", "--offline", "--manifest-path",
                               external / "sdk/examples/hello-plugin/Cargo.toml", "-j", "2"], env=standalone_env)
    standalone_library = target / "sdk-external-target/debug" / library.name
    assert standalone_library.is_file()
    # Keep a directly usable matching Windows/macOS package tool beside the SDK.
    shutil.copyfile(tool, output / tool.name)
    artifacts = {path.name: {"path": str(path), "bytes": path.stat().st_size, "sha256": digest(path)}
                 for path in [package, project, bundle, output / tool.name, library, app]}
    result = {"status": "PASS", "target": triple, "commands": records,
              "artifacts": artifacts, "objects_created": 3,
              "cold_project_hash_equal": True, "failed_commands_preserved_file": True,
              "standalone_sdk_build": True, "macos_runtime_verified": os.name != "nt",
              "interactive_ui_acceptance": "not performed", "native_geometry_adapters": "not exercised"}
    (output / "observed.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new output directory")
    args = parser.parse_args()
    result = validate(args.project_root, args.target_dir, args.output)
    print(json.dumps({"status": result["status"], "target": result["target"],
                      "objects": result["objects_created"], "artifacts": result["artifacts"]}, indent=2))
