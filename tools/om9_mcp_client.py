"""Minimal JSON-RPC stdio client for the headless OM9 MCP server (`om9 --mcp`).

Used by the command-checklist audit: it drives the same Controller as the
Command Line and reads typed geometry back, so a menu click and a typed command
can be compared without a window.
"""
import json
import subprocess


class Client:
    def __init__(self, exe):
        self.proc = subprocess.Popen(
            [exe, "--mcp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
        )
        self.next_id = 0
        self.call("initialize", {"protocolVersion": "2024-11-05", "capabilities": {}})

    def call(self, method, params):
        self.next_id += 1
        message = {"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params}
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        if not line:
            return {"error": {"message": "server closed stdout"}}
        return json.loads(line)

    def tool(self, tool_name, **args):
        reply = self.call("tools/call", {"name": tool_name, "arguments": args})
        result = reply.get("result", {})
        text = result.get("content", [{}])[0].get("text", "")
        try:
            value = json.loads(text)
        except json.JSONDecodeError:
            value = {"raw": text}
        return {"is_error": result.get("isError", False), "data": value}

    def new(self, name="audit"):
        return self.tool("om9_new", name=name)

    def command(self, text):
        return self.tool("om9_command", text=text)

    def dispatch(self, feature, origin="menu"):
        return self.tool("om9_dispatch", feature=feature, origin=origin)

    def point(self, point):
        return self.tool("om9_point", point=point)

    def state(self, limit=200):
        return self.tool("om9_state", limit=limit)

    def select(self, **args):
        return self.tool("om9_select", **args)

    def close(self):
        try:
            self.proc.stdin.close()
        except OSError:
            pass
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
