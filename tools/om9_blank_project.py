"""Create an empty .om9 project for the OS-mouse acceptance harness."""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from om9_mcp_client import Client

if len(sys.argv) != 2:
    raise SystemExit("usage: om9_blank_project.py <path.om9>")
client = Client(r"L:\om9-target\debug\om9.exe")
client.new("gumball os audit")
reply = client.tool("om9_save", path=os.path.abspath(sys.argv[1]))
client.close()
if reply["is_error"]:
    raise SystemExit(reply["data"])
print(reply["data"]["saved"])
