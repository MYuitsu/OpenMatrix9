"""Read-only Win32 visibility/focus samples for OM9 benchmark processes.

No window is activated, moved, restored or sent input. This is a Windows
telemetry adapter; GPU publication and CAD correctness remain in the Rust audit.
"""
import argparse
import ctypes
from ctypes import wintypes
import datetime
import json
from pathlib import Path
import time

import psutil


def windows(pids):
    user = ctypes.WinDLL("user32", use_last_error=True)
    user.GetForegroundWindow.restype = wintypes.HWND
    user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user.IsIconic.argtypes = [wintypes.HWND]
    user.IsZoomed.argtypes = [wintypes.HWND]
    user.IsHungAppWindow.argtypes = [wintypes.HWND]
    user.IsWindowVisible.argtypes = [wintypes.HWND]
    user.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    foreground = user.GetForegroundWindow()
    result = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    @callback_type
    def visit(hwnd, _):
        pid = wintypes.DWORD()
        user.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value not in pids:
            return True
        rect = wintypes.RECT()
        user.GetClientRect(hwnd, ctypes.byref(rect))
        cloaked = wintypes.DWORD()
        dwm = ctypes.WinDLL("dwmapi")
        dwm.DwmGetWindowAttribute.argtypes = [wintypes.HWND, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD]
        hr = dwm.DwmGetWindowAttribute(hwnd, 14, ctypes.byref(cloaked), ctypes.sizeof(cloaked))
        result.append({"pid": pid.value, "hwnd": hwnd, "visible": bool(user.IsWindowVisible(hwnd)),
                       "minimized": bool(user.IsIconic(hwnd)), "foreground": hwnd == foreground,
                       "maximized": bool(user.IsZoomed(hwnd)), "hung": bool(user.IsHungAppWindow(hwnd)),
                       "cloaked": cloaked.value if hr == 0 else None,
                       "client_pixels": [rect.right - rect.left, rect.bottom - rect.top]})
        return True

    user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
    if not user.EnumWindows(visit, 0):
        raise ctypes.WinError(ctypes.get_last_error())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--once", action="store_true")
    args = parser.parse_args()
    idle = time.monotonic()
    with args.output.open("a", encoding="utf-8") as stream:
        while True:
            pids = set()
            for proc in psutil.process_iter(["name", "exe"]):
                if proc.info["name"] and proc.info["name"].lower() == "om9.exe" and proc.info["exe"]:
                    if Path(proc.info["exe"]).resolve() == Path("L:/om9-target/release/om9.exe").resolve():
                        pids.add(proc.pid)
            row = {"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "windows": windows(pids)}
            stream.write(json.dumps(row) + "\n")
            stream.flush()
            if pids:
                idle = time.monotonic()
            if args.once or time.monotonic() - idle > 30:
                print(json.dumps(row), flush=True)
                return
            time.sleep(1)


if __name__ == "__main__":
    main()
