"""Bounded own-app native UI replay and source/binary/runtime receipts.

Exercises the normal egui widget path in a real winit/wgpu window. This harness
never injects desktop input, automates another app, or promotes broad UI gates.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import time

from validate_standalone_foundation import ROOT, pe_imports, sha256


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "standalone/target/release/om9.exe")
    parser.add_argument("--output", type=Path, default=ROOT / "docs/validation/2026-10-13-standalone-ui")
    parser.add_argument("--viewport-menu", action="store_true")
    parser.add_argument("--layers", action="store_true")
    parser.add_argument("--layer-dialogs", action="store_true")
    parser.add_argument("--files", action="store_true")
    parser.add_argument("--file-close", action="store_true")
    parser.add_argument("--file-success", action="store_true")
    parser.add_argument("--startup-file", type=Path)
    args = parser.parse_args()
    if args.file_close or args.file_success:
        args.files = True
    if args.file_close and args.file_success: raise ValueError("choose close or success replay")
    if sum([args.layers,args.viewport_menu,args.layer_dialogs,args.files])>1: raise ValueError("choose one replay")
    if args.startup_file and any([args.layers,args.viewport_menu,args.layer_dialogs,args.files]): raise ValueError("startup is a separate observation")
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    source_paths = lambda: {p for p in (ROOT / "standalone").rglob("*")
                           if p.is_file() and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json"}} | set((ROOT / "Resources/icons").rglob("*.svg"))
    sources_before = {p:sha256(p) for p in source_paths()}
    binary_before = sha256(binary)
    environment = {k: os.environ[k] for k in ("SystemRoot", "TEMP", "TMP") if k in os.environ}
    environment["OM9_UI_AUDIT_HOLD"] = "1"
    if args.viewport_menu:
        environment["OM9_UI_AUDIT_VIEWPORT_MENU"] = "1"
    if args.layers:
        environment["OM9_UI_AUDIT_LAYERS"] = "1"
    if args.layer_dialogs:
        environment["OM9_UI_AUDIT_LAYER_DIALOGS"] = "1"
    if args.files:
        environment["OM9_UI_AUDIT_FILES"] = "1"
    if args.file_close:
        environment["OM9_UI_AUDIT_FILE_CLOSE"] = "1"
    if args.file_success:
        environment["OM9_UI_AUDIT_FILE_SUCCESS"] = "1"
    scratch=ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True,exist_ok=True)
    environment.update(TEMP=str(scratch),TMP=str(scratch))
    shell = shutil.which("pwsh") or shutil.which("powershell")
    if not shell or os.name != "nt":
        raise RuntimeError("Windows live UI process observation unavailable")
    with tempfile.TemporaryDirectory(prefix="om9-ui-profile-",dir=scratch) as temporary:
        profile = Path(temporary)
        assert profile.resolve().is_relative_to(scratch.resolve())
        startup_report = None
        startup_sha = None
        if args.startup_file:
            args.startup_file = args.startup_file.resolve(strict=True)
            if not args.startup_file.is_relative_to(scratch.resolve()): raise ValueError("owned startup fixture must be under scratch")
            startup_sha = sha256(args.startup_file)
            cold = subprocess.run([str(binary),"--inspect",str(args.startup_file)],cwd=profile,env=environment,capture_output=True,timeout=10,
                                  creationflags=getattr(subprocess,"CREATE_NO_WINDOW",0))
            if cold.returncode: raise AssertionError(cold.stderr.decode(errors="replace"))
            startup_report = json.loads(cold.stdout)
            environment["OM9_UI_AUDIT_STARTUP_DIRECTORY"] = str(profile)
        launch_args = ["--ui",str(args.startup_file)] if args.startup_file else ["--ui-audit",str(profile)]
        process = subprocess.Popen([str(binary), *launch_args],
                                   cwd=profile, env=environment, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE,
                                   creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            ready = profile / "ui-receipt.json"
            deadline = time.monotonic() + 35
            while not ready.is_file():
                if process.poll() is not None:
                    _, error = process.communicate(timeout=3)
                    raise RuntimeError(f"UI exited {process.returncode}: {error.decode(errors='replace')}")
                if time.monotonic() > deadline:
                    trace = profile / ("file-replay.json" if args.files else "dialog-replay.json")
                    detail = trace.read_text() if (args.layer_dialogs or args.files) and trace.is_file() else "no replay trace"
                    raise TimeoutError(f"UI replay did not produce screenshot receipt: {detail}")
                time.sleep(0.1)
            receipt = json.loads(ready.read_text(encoding="utf-8"))
            expected = ["CMD line accepted once", "actual CMD Undo", "actual CMD Redo",
                        "Esc canceled draft without publication"]
            if args.viewport_menu:
                expected = ["native title popup preserves numeric draft",
                            "native keyboard Maximize preserves model and draft",
                            "native keyboard Restore preserves all view states",
                            "native Escape preserves pending command",
                            "native outside click only dismisses menu",
                            "native popup reopened after dismissal"]
            if args.layers:
                expected=["native layer current preserves pending Line","native Line binds current at acceptance",
                          "native swatch selects eligible membership","native lazy arrow assigns without current change",
                          "native Hide owns Hidden only","native Show restores independent Hidden",
                          "native group Off preserves explicit preferences","native Lock masks pick keeps display",
                          "native Unlock restores explicit selectability","native Advanced stable label current",
                          "native collapse preserves state","native CMD Undo restores layer state",
                          "native CMD Redo restores stable preset IDs"]
            if args.layer_dialogs:
                expected=["native rename Cancel preserves pending Line and lazy model",
                          "native rename OK owns one lazy group and stable label",
                          "native color Cancel preserves layer and identity",
                          "native RGB keyboard edit and OK preserves identity and draft",
                          "native Line after dialogs binds current and by-layer style",
                          "native selective Show Cancel keeps Hidden and canvas selection",
                          "native selective Show checkbox OK restores only Hidden",
                          "native CMD Undo restores selective Show flags",
                          "native CMD Redo restores flags and preset identity",
                          "native reserved rename refuses and Cancel keeps accepted state"]
            if args.files:
                expected=["native New toolbar opens owned prompt preserves Line",
                          "native prompt Escape retains old document and draft",
                          "real Windows Save chooser Cancel preserves pending Line",
                          "native File menu Open Cancel preserves pending Line",
                          "native F6 New uses same owned prompt",
                          "native Discard replaces session only after button",
                          "real Windows Open chooser Cancel keeps fresh document",
                          "native CMD SaveAs uses real Windows chooser Cancel"]
                if args.file_close:
                    expected += ["real WM_CLOSE opens owned dirty prompt",
                                 "native close Cancel keeps unsaved document and window",
                                 "native close Save then real Windows chooser Cancel keeps window"]
                if args.file_success:
                    expected = ["real Windows Save selection writes exact owned document",
                                "native SaveAs selection waits for owned overwrite without changing file",
                                "native owned Replace widget adopts SaveAs path without geometry Undo",
                                "native New toolbar renews runtime before reopen",
                                "real Windows Open selection restores document path and scene"]
            if args.startup_file:
                expected = ["native --ui retains accepted path and window title",
                            "native --ui restores document before four viewport rendering",
                            "native --ui retains explicit restoration diagnostics and clean state"]
            object_count = len(startup_report["objects"]) if args.startup_file else 0 if args.viewport_menu else 2 if args.file_success else 1
            if receipt["checks"] != expected or receipt["objects"] != object_count or receipt["undo"] != (0 if args.file_success or args.startup_file else 6 if args.layer_dialogs else 10 if args.layers else object_count):
                diagnostic={key:receipt[key] for key in ("scope","checks","objects","undo","cmd")}
                (scratch/"ui-last-failed-receipt.json").write_text(json.dumps(receipt,ensure_ascii=False,indent=2),encoding="utf-8")
                if (profile/"workspace.png").is_file(): shutil.copyfile(profile/"workspace.png",scratch/"ui-last-failed.png")
                for tag in ["rename", "color", "selective-show"]:
                    image = profile / f"dialog-{tag}.png"
                    if image.is_file(): shutil.copyfile(image,scratch/f"ui-last-failed-{tag}.png")
                raise AssertionError("Native widget replay lifecycle mismatch: "+json.dumps(diagnostic,ensure_ascii=False))
            entities = receipt["document"]["entities"]
            if len(entities) != object_count:
                raise AssertionError("Native accepted object count mismatch")
            if args.startup_file:
                if receipt["scope"] != "owned_native_file_startup_slice" or Path(receipt["active_file_path"]) != args.startup_file:
                    raise AssertionError("Native startup path ownership mismatch")
                if receipt["startup_persistent_report"] != startup_report or sha256(args.startup_file) != startup_sha:
                    raise AssertionError("Native startup changed persistent file contents/report")
                receipt["native_startup_fixture"] = {"path":args.startup_file.relative_to(ROOT).as_posix(),"sha256":startup_sha}
                receipt["native_startup_cold_inspect"] = startup_report
            elif args.viewport_menu:
                if receipt["scope"] != "viewport_title_menu_slice" or not receipt["cmd"].endswith("1,2,"):
                    raise AssertionError("Viewport replay lost its numeric command draft")
                if not receipt.get("viewport_popup_open"):
                    raise AssertionError("Viewport screenshot captured a closed popup")
            elif args.layers or args.layer_dialogs:
                if receipt["scope"] != ("owned_layer_dialogs_slice" if args.layer_dialogs else "owned_layers_widgets_current_cmd_slice"): raise AssertionError("wrong Layers scope")
                document=receipt["document"]
                entity=next(iter(entities.values()))
                if entity["geometry"]["Curve"]["Line"]["start"] != [0,0,0] or entity["geometry"]["Curve"]["Line"]["end"] != [12,6,0]: raise AssertionError("Layers changed geometry")
                if len(document["layers"]) != (4 if args.layer_dialogs else 12) or document["layers"][document["current_layer"]]["preset_slot"] != 21: raise AssertionError("Layers lazy group/current mismatch")
                if document["layers"][entity["attributes"]["layer"]]["preset_slot"] != (21 if args.layer_dialogs else 5) or entity["attributes"]["color"] is not None or entity["attributes"].get("hidden",False): raise AssertionError("Layer membership/preferences mismatch")
                if args.layer_dialogs:
                    layer=document["layers"][document["current_layer"]]
                    if receipt.get("modal_open") or layer["name"]!="Native saved label" or layer["color"]!=[72,199,170,255]: raise AssertionError("Modal changed or failed to restore committed metadata")
            elif args.file_success:
                if receipt["scope"] != "owned_file_native_selection_slice" or receipt["file_operation_open"] or receipt["draft"]:
                    raise AssertionError("Native selected File lifecycle mismatch")
                selected = profile / "alternate.om9"
                if Path(receipt["active_file_path"]) != selected or receipt["saved_document_id"] != receipt["document"]["id"]:
                    raise AssertionError("Native selected archive/path identity mismatch")
                if sorted(entity["geometry"]["Point"] for entity in entities.values()) != [[1,2,3],[9,8,7]]:
                    raise AssertionError("Native Open changed geometry")
                picker = receipt["file_picker_receipts"]
                if [row["phase"] for row in picker] != ["SavePath","SavePath","OpenPath"]:
                    raise AssertionError("Native selection chooser phases mismatch")
                if any(row.get("pid") != process.pid or row.get("close_posted") or Path(row.get("returned_path", "")) != (profile / "selected.om9" if index == 0 else selected)
                       or not all(row.get(key) for key in ["observed","accept_posted","destroyed"])
                       or row.get("elapsed_ms", 5001) > 5000 for index,row in enumerate(picker)):
                    raise AssertionError("Native Show/GetResult selection not verified")
                cold = subprocess.run([str(binary), "--inspect", str(selected)],cwd=profile,env=environment,capture_output=True,timeout=10,
                                      creationflags=getattr(subprocess,"CREATE_NO_WINDOW",0))
                if cold.returncode: raise AssertionError(cold.stderr.decode(errors="replace"))
                receipt["native_selected_cold_inspect"] = json.loads(cold.stdout)
                if receipt["native_selected_cold_inspect"]["document_id"] != receipt["document"]["id"] or len(receipt["native_selected_cold_inspect"]["objects"]) != 2 or receipt["native_selected_cold_inspect"]["diagnostics"]:
                    raise AssertionError("Cold selected archive contents mismatch")
                receipt["native_selected_archive_sha256"] = sha256(selected)
                original = profile / "selected.om9"
                if sha256(original) != receipt["original_saved_archive_sha256"] or receipt["seeded_overwrite_archive_sha256"] != receipt["original_saved_archive_sha256"] or sha256(selected) == receipt["seeded_overwrite_archive_sha256"]:
                    raise AssertionError("Native overwrite preservation fingerprints mismatch")
            elif args.files:
                if receipt["scope"] != ("owned_file_native_close_cancel_slice" if args.file_close else "owned_file_native_cancel_slice") or receipt["file_operation_open"] or receipt["active_file_path"] is not None or receipt["draft"]:
                    raise AssertionError("File cancellation/publication ownership mismatch")
                if next(iter(entities.values()))["geometry"]!={"Point":[1,2,3]}:
                    raise AssertionError("File replacement retained old geometry or changed new output")
                picker=receipt["file_picker_receipts"]
                if [p.get("phase") for p in picker] != ["SavePath","OpenPath","SavePath"] + (["SavePath"] if args.file_close else []):
                    raise AssertionError("Missing real OS picker phases")
                if args.file_close and receipt.get("native_close_events", 0) < 2:
                    raise AssertionError("Missing actual root WM_CLOSE events")
                if any(p.get("pid")!=process.pid or not all(p.get(k) for k in ("observed","close_posted","destroyed","returned_cancel")) or p.get("elapsed_ms",5001)>5000 for p in picker):
                    raise AssertionError("OS chooser was not observed/canceled/destroyed within its owned probe")
            else:
                entity = next(iter(entities.values()))
                line = entity["geometry"]["Curve"]["Line"]
                if line["start"] != [0, 0, 0] or line["end"] != [12, 6, 0]:
                    raise AssertionError("CMD replay did not produce literal world endpoints")
                if entity["attributes"]["layer"] is not None or entity["attributes"]["color"] != [0, 130, 85, 255]:
                    raise AssertionError("Fresh-document current/color contract mismatch")
            if set(receipt["view_rects"]) != {"Looking Down", "Perspective", "Side View", "Through Finger"}:
                raise AssertionError("Four viewport title identity mismatch")
            looking = receipt["view_rects"]["Looking Down"]
            perspective = receipt["view_rects"]["Perspective"]
            finger = receipt["view_rects"]["Through Finger"]
            side = receipt["view_rects"]["Side View"]
            if not (looking[0] >= 280 and looking[1] >= 150 and looking[0] == finger[0]
                    and perspective[0] == side[0] and finger[1] > looking[1] and side[1] > perspective[1]):
                raise AssertionError("Supplied Matrix9 reference layout mismatch")
            png = profile / "workspace.png"
            data = png.read_bytes()
            if not receipt["screenshot_saved"] or data[:8] != b"\x89PNG\r\n\x1a\n":
                raise AssertionError("Actual rendered PNG missing")
            width, height = struct.unpack_from(">II", data, 16)
            if width < 720 or height < 480 or receipt["passes"] < (2 if args.startup_file else 17):
                raise AssertionError("Native render dimensions/passes mismatch")
            # Inspect only this harness-owned child; no native window operations.
            script = f"$p=Get-Process -Id {process.pid}; @{{modules=@($p.Modules | ForEach-Object {{$_.ModuleName}}); children=@(Get-CimInstance Win32_Process -Filter 'ParentProcessId={process.pid}' | Select-Object -ExpandProperty Name); mainWindowHandle=$p.MainWindowHandle.ToInt64()}} | ConvertTo-Json -Compress"
            observed = subprocess.run([shell, "-NoProfile", "-Command", script], capture_output=True,
                                      timeout=20, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            if observed.returncode:
                raise RuntimeError(observed.stderr.decode(errors="replace"))
            live = json.loads(observed.stdout)
            imports = pe_imports(binary)
            forbidden = ("freecad", "qymcad", "python", "coin", "soqt", "part.dll", "qt5", "qt6")
            if any(any(word in name.lower() for word in forbidden)
                   for name in imports + live["modules"] + live["children"]):
                raise AssertionError("Forbidden legacy/upstream host dependency")
            if live["children"] or not live["mainWindowHandle"]:
                raise AssertionError("Native root window missing or undeclared child exists")
            (profile / "release").write_text("release", encoding="utf-8")
            _, error = process.communicate(timeout=10)
            if process.returncode:
                raise RuntimeError(error.decode(errors="replace"))
            output.mkdir(parents=True, exist_ok=True)
            # A new checkpoint must not overwrite another receipt's images.
            artifact_directory = scratch / "ui-artifacts" / output.name
            artifact_directory.mkdir(parents=True, exist_ok=True)
            if args.file_success:
                selected_artifact = artifact_directory / "alternate.om9"
                shutil.copyfile(profile / "alternate.om9", selected_artifact)
                receipt["native_selected_archive"] = {"path":selected_artifact.relative_to(ROOT).as_posix(),"sha256":sha256(selected_artifact)}
                original_artifact = artifact_directory / "selected.om9"
                shutil.copyfile(profile / "selected.om9", original_artifact)
                receipt["native_original_archive"] = {"path":original_artifact.relative_to(ROOT).as_posix(),"sha256":sha256(original_artifact)}
            screenshot = artifact_directory / "workspace.png"
            shutil.copyfile(png, screenshot)
            dialog_screenshots = []
            if args.layer_dialogs:
                for tag in ["rename", "color", "selective-show"]:
                    image_path = profile / f"dialog-{tag}.png"
                    with image_path.open("rb") as image_file:
                        header = image_file.read(24)
                    if header[:8] != b"\x89PNG\r\n\x1a\n":
                        raise AssertionError(f"missing dialog screenshot: {tag}")
                    destination = artifact_directory / f"dialog-{tag}.png"
                    shutil.copyfile(image_path, destination)
                    dialog_screenshots.append({"dialog": tag, "path": destination.relative_to(ROOT).as_posix(),
                                               "sha256": sha256(destination),
                                               "size": list(struct.unpack(">II", header[16:24]))})
            if source_paths() != set(sources_before) or any(sha256(p) != sha for p,sha in sources_before.items()) or sha256(binary) != binary_before:
                raise AssertionError("Source/binary changed during native UI replay")
            receipt.update({"schema_version": 1, "target": "OM9Standalone",
                            "status": "validated_owned_file_native_cancel_slice" if args.files else "validated_owned_layer_dialogs_slice" if args.layer_dialogs else "validated_owned_layers_widgets_current_cmd_slice" if args.layers else "validated_viewport_title_menu_slice" if args.viewport_menu else "validated_supported_ui_slice",
                            "goal_complete": False,
                            "harness": {"path":Path(__file__).relative_to(ROOT).as_posix(),"sha256":sha256(Path(__file__))},
                            "timestamp": dt.datetime.now(dt.timezone(dt.timedelta(hours=7))).isoformat(timespec="seconds"),
                            "binary": {"path": binary.relative_to(ROOT).as_posix(), "sha256": sha256(binary)},
                            "screenshot": {"path": screenshot.relative_to(ROOT).as_posix(),
                                           "sha256": sha256(png), "size": [width, height]},
                            "pe_imports": imports, "loaded_modules": live["modules"], "children": live["children"],
                            "native_window_observed": True,
                            "dialog_screenshots": dialog_screenshots,
                            "sources": [{"path": p.relative_to(ROOT).as_posix(), "sha256": sha256(p)}
                                        for p in sorted(sources_before)]})
            (output / "ui-receipt.json").write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            print(json.dumps({"status": receipt["status"], "checks": len(expected), "native_window": True,
                              "modules": len(live["modules"]), "children": live["children"],
                              "screenshot": receipt["screenshot"]["size"]}))
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)


if __name__ == "__main__":
    main()
