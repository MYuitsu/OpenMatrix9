"""Exercise Sphere through an isolated real Windows GUI and its MCP proxy.

Usage: python tools/validate_live_sphere_stack.py path/to/om9.exe
The isolated LOCALAPPDATA prevents attaching to or altering a user's window.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main(exe):
    with tempfile.TemporaryDirectory(prefix="om9-live-sphere-") as temp:
        env = os.environ.copy()
        env["LOCALAPPDATA"] = temp
        gui = subprocess.Popen([exe], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        proxy = None
        try:
            registrations = Path(temp) / "OpenMatrix9" / "mcp-live"
            for _ in range(300):
                if registrations.exists() and list(registrations.glob("*.json")):
                    break
                if gui.poll() is not None:
                    raise RuntimeError(f"GUI exited: {gui.returncode}")
                time.sleep(0.1)
            else:
                raise RuntimeError("GUI did not publish its bridge")
            proxy = subprocess.Popen([exe, "--mcp-live"], env=env, stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                     text=True, encoding="utf-8")

            def call(method, params):
                request = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
                proxy.stdin.write(json.dumps(request) + "\n")
                proxy.stdin.flush()
                line = proxy.stdout.readline()
                if not line:
                    raise RuntimeError(f"MCP closed; GUI status: {gui.poll()}")
                result = json.loads(line)
                if "error" in result or result.get("result", {}).get("isError"):
                    raise RuntimeError(str(result))
                return result["result"]

            def tool(name, **args):
                return json.loads(call("tools/call", {"name": name, "arguments": args})["content"][0]["text"])

            call("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}})
            tool("om9_command", text="Sphere")
            tool("om9_command", text="0,0,0")
            created = tool("om9_command", text="5")["state"]
            assert created["objects_total"] == 1 and created["objects"][0]["kind"] == "Brep"
            object_id = created["objects"][0]["id"]
            assert tool("om9_undo")["state"]["objects_total"] == 0
            restored = tool("om9_redo")["state"]
            assert restored["objects_total"] == 1 and restored["objects"][0]["id"] == object_id
            tool("om9_undo")
            tool("om9_dispatch", feature="OM9-SOLID-014", origin="menu")
            tool("om9_point", point=[3, 7, 0])
            point_created = tool("om9_point", point=[8, 7, 0])["state"]
            assert point_created["objects_total"] == 1
            assert tool("om9_undo")["state"]["objects_total"] == 0
            assert gui.poll() is None
            print("PASS: GUI MCP Sphere CMD, menu/world-point, Undo and Redo")
        finally:
            if proxy and proxy.poll() is None:
                proxy.stdin.close()
                proxy.wait(timeout=10)
            if gui.poll() is None:
                gui.terminate()
                gui.wait(timeout=15)


if __name__ == "__main__":
    main(str(Path(sys.argv[1]).resolve()))
