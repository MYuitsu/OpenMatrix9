#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-2.1-or-later
"""Observe one isolated OM9 GPU window through native maximize and restore.

Python exception: Windows process/window orchestration only; sizing stays Rust.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time


def wait_for(probe, message, seconds=20):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = probe()
        if value:
            return value
        time.sleep(0.1)
    raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if os.name != 'nt':
        raise RuntimeError('Windows native window test only')
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    user32 = ctypes.WinDLL('user32', use_last_error=True)
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    user32.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
    user32.IsZoomed.argtypes = [wintypes.HWND]
    user32.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]

    def native_state(hwnd):
        rect = wintypes.RECT()
        title = ctypes.create_unicode_buffer(512)
        user32.GetClientRect(hwnd, ctypes.byref(rect))
        user32.GetWindowTextW(hwnd, title, len(title))
        return {'title': title.value, 'client': [rect.right, rect.bottom], 'maximized': bool(user32.IsZoomed(hwnd))}

    def window_for(pid):
        windows = []
        @callback_type
        def callback(hwnd, _):
            found = wintypes.DWORD()
            user32.GetWindowThreadProcessId(hwnd, ctypes.byref(found))
            if found.value == pid and user32.IsWindowVisible(hwnd):
                state = native_state(hwnd)
                # Winit/IME may own additional visible, untitled zero-size
                # helper windows. Only operate the real application window.
                if state['title'] and state['client'][0] >= 800 and state['client'][1] >= 400:
                    windows.append(hwnd)
            return True
        user32.EnumWindows(callback, 0)
        return windows[0] if windows else None

    def readings():
        result = {}
        for path in output.glob('resize-*.json'):
            try:
                result[path.name] = json.loads(path.read_text())
            except (json.JSONDecodeError, OSError):
                pass
        return result

    def edges(receipt):
        return [receipt['toolbar_slot_pt']] + [receipt[key][0][2] for key in ('main', 'display', 'snaps', 'info')]

    with tempfile.TemporaryDirectory(prefix='om9-live-resize-') as temporary:
        profile = Path(temporary)
        environment = {key: os.environ[key] for key in ('SystemRoot', 'TEMP', 'TMP') if key in os.environ}
        environment.update(OM9_UI_AUDIT_WINDOW='1280x720', OM9_UI_AUDIT_HOLD='1', OM9_UI_LAYOUT=str(profile / 'layout.json'))
        with (output / 'native.log').open('w') as log:
            process = subprocess.Popen([str(binary), '--ui-audit', str(output)], cwd=profile, env=environment,
                                       stdout=log, stderr=log, creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                hwnd = wait_for(lambda: window_for(process.pid), 'Native window missing')
                initial_name, initial = next(iter(wait_for(readings, 'Initial resize metrics missing').items()))
                wait_for(lambda: list(output.glob('resize-*.png')), 'Initial GPU screenshot missing')
                before = set(readings())
                print(json.dumps({'before': native_state(hwnd)}), flush=True)
                user32.ShowWindow(hwnd, 3)  # SW_MAXIMIZE on this owned test process only.
                print(json.dumps({'after': native_state(hwnd)}), flush=True)
                changed = wait_for(lambda: {k: v for k, v in readings().items() if k not in before}, 'Maximized resize metrics missing')
                maximum_name, maximum = next(iter(changed.items()))
                maximum_png = output / ('resize-' + 'x'.join(str(round(v * maximum['pixels_per_point'])) for v in maximum['window_points']) + '.png')
                wait_for(maximum_png.exists, 'Maximized GPU screenshot missing')
                for key, count in [('display', 11), ('snaps', 16), ('info', 22)]:
                    assert len(initial[key]) == len(maximum[key]) == count
                small_edges, full_edges = edges(initial), edges(maximum)
                assert all(b > a + 0.2 for a, b in zip(small_edges, full_edges)), (small_edges, full_edges)
                # Removing only this run's metric makes the app record a fresh restored frame.
                (output / initial_name).unlink()
                user32.ShowWindow(hwnd, 9)  # SW_RESTORE
                restored = wait_for(lambda: readings().get(initial_name), 'Fresh restored metrics missing')
                assert edges(restored) == small_edges, (edges(restored), small_edges)
                assert restored['window_points'] == initial['window_points']
                assert initial['dock']['left_rail_outer_px'] == maximum['dock']['left_rail_outer_px'] == 360
                summary = {'passed': True, 'binary': str(binary), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                           'initial': initial, 'maximized': maximum, 'restored': restored,
                           'edge_order': ['toolbar', 'main', 'display', 'snaps', 'info'],
                           'initial_edges': small_edges, 'maximized_edges': full_edges,
                           'limits': ['One actual Windows monitor and its actual OS scale; not a QHD/multi-monitor certification.', 'SW_MAXIMIZE exercises the title-bar maximize action, not borderless fullscreen.']}
                (output / 'live-resize-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
                print(json.dumps({'passed': True, 'initial': initial['window_points'], 'maximized': maximum['window_points'], 'initial_edges': small_edges, 'maximized_edges': full_edges}))
            finally:
                (output / 'release').touch()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.terminate()  # Only the empty isolated audit process.
                    process.wait(timeout=5)


if __name__ == '__main__':
    main()
