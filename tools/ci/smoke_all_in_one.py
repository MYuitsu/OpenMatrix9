# SPDX-License-Identifier: LGPL-2.1-or-later
"""Smoke-test the extracted standalone application and its system add-ons."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def smoke(app, addon_tool):
    environment = {
        key: value for key, value in os.environ.items()
        if not key.upper().startswith(("OM9_", "DYLD_"))
    }
    if os.name == "nt":
        system = Path(os.environ["SystemRoot"])
        environment["PATH"] = os.pathsep.join(map(str, [system / "System32", system]))
    else:
        environment["PATH"] = "/usr/bin:/bin"
    catalog = Path(__file__).resolve().parents[2] / "Resources/menu/MainMenu.ini"
    with tempfile.TemporaryDirectory(prefix="OM9 bundle smoke ") as temporary:
        workspace = Path(temporary)
        environment["OM9_CONFIG_DIR"] = str(workspace / "config")

        def run(*args, input=None):
            result = subprocess.run(
                [str(arg) for arg in args], cwd=workspace, env=environment,
                input=input, capture_output=True, timeout=90,
            )
            if result.returncode:
                raise RuntimeError(
                    f"{args}: exit {result.returncode} {result.stdout[-1800:]!r} {result.stderr[-1800:]!r}"
                )
            return result.stdout

        project = workspace / "box.om9"
        model = workspace / "box.3dm"
        reopened = workspace / "reopened.om9"
        commands = [
            ("om9_new", {"name": "All-in-one smoke"}),
            ("om9_command", {"text": "Box 4 5 6"}),
            ("om9_save", {"path": str(project)}),
            ("om9_open", {"path": str(project)}),
            ("om9_state", {}),
        ]
        requests = [
            {"jsonrpc": "2.0", "id": i, "method": "tools/call",
             "params": {"name": name, "arguments": args}}
            for i, (name, args) in enumerate(commands, start=1)
        ]
        replies = run(app, "--mcp", input=b"".join(
            json.dumps(request).encode() + b"\n" for request in requests
        ))
        replies = [json.loads(line) for line in replies.splitlines()]
        if len(replies) != len(commands) or not project.is_file():
            raise RuntimeError(f"Incomplete OM9 BRep replay: {replies!r}")
        for reply in replies:
            if "error" in reply or reply["result"].get("isError"):
                raise RuntimeError(f"OM9 BRep replay failed: {reply!r}")

        run(app, "--export-3dm", project, model)
        run(app, "--import-3dm", model, reopened)
        if model.stat().st_size == 0 or reopened.stat().st_size == 0:
            raise RuntimeError("Empty 3DM round-trip")
        for addon in ("om9_matrix", "ring_sizes"):
            run(addon_tool, "--system", app.parent / "addons", "--config",
                workspace / "addon-test", "--catalog", catalog, "enable", addon, "--yes")
        print("All-in-one smoke passed: BRep save/open, native 3DM round-trip, Matrix and Ring Sizes")


if __name__ == "__main__":
    argument_parser = argparse.ArgumentParser(description=__doc__)
    argument_parser.add_argument("--app", type=Path, required=True)
    argument_parser.add_argument("--addon-tool", type=Path, required=True)
    options = argument_parser.parse_args()
    smoke(options.app.resolve(strict=True), options.addon_tool.resolve(strict=True))
