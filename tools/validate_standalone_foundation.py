"""Record scoped, actual standalone process receipts; never close broad gates.

Runs authored fixtures in isolated profiles, audits PE imports and live loaded
modules/children on Windows. Python is the validation harness, not an app host.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def pe_imports(path: Path) -> list[str]:
    """Bounded import-directory parsing from the actual PE image, no SDK host."""
    data = path.read_bytes()
    if len(data) < 64 or data[:2] != b"MZ":
        raise ValueError("Executable is not a PE image")
    offset = struct.unpack_from("<I", data, 60)[0]
    if offset + 24 > len(data) or data[offset:offset + 4] != b"PE\0\0":
        raise ValueError("Invalid PE header")
    sections = struct.unpack_from("<H", data, offset + 6)[0]
    optional_size = struct.unpack_from("<H", data, offset + 20)[0]
    optional = offset + 24
    if optional + optional_size > len(data):
        raise ValueError("Truncated optional header")
    magic = struct.unpack_from("<H", data, optional)[0]
    directory = optional + {0x20B: 112, 0x10B: 96}[magic]
    rva, size = struct.unpack_from("<II", data, directory + 8)
    table = optional + optional_size
    if table + 40 * sections > len(data):
        raise ValueError("Truncated section table")
    ranges = []
    for index in range(sections):
        position = table + index * 40
        virtual_size, address, raw_size, raw_pointer = struct.unpack_from("<IIII", data, position + 8)
        ranges.append((address, max(virtual_size, raw_size), raw_pointer, raw_size))

    def file_offset(address: int) -> int:
        for start, span, raw, raw_size in ranges:
            if start <= address < start + span and address - start < raw_size:
                result = raw + address - start
                if result < len(data):
                    return result
        raise ValueError("Unmapped import RVA")

    result = []
    if not rva:
        return result
    start = file_offset(rva)
    for index in range(size // 20):
        position = start + index * 20
        if position + 20 > len(data):
            raise ValueError("Truncated import descriptor")
        descriptor = struct.unpack_from("<IIIII", data, position)
        if not any(descriptor):
            return sorted(result)
        name = file_offset(descriptor[3])
        end = data.find(b"\0", name, min(name + 1024, len(data)))
        if end < 0:
            raise ValueError("Unterminated import name")
        result.append(data[name:end].decode("ascii"))
    raise ValueError("Import directory lacks terminator")


def run(binary: Path, arguments: list[str], profile: Path, environment: dict[str, str]) -> dict:
    completed = subprocess.run([str(binary), *arguments], cwd=profile, env=environment,
                               capture_output=True, timeout=30, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    if completed.returncode:
        raise RuntimeError(completed.stderr.decode("utf-8", errors="replace"))
    return json.loads(completed.stdout)


def live_audit(binary: Path, model: Path, profile: Path, environment: dict[str, str]) -> dict:
    process = subprocess.Popen([str(binary), "--audit", str(model)], cwd=profile, env=environment,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
    try:
        first = process.stdout.readline()
        report = json.loads(first)
        if process.poll() is not None:
            raise RuntimeError("Audit process exited before observation")
        shell = shutil.which("pwsh") or shutil.which("powershell")
        if not shell or os.name != "nt":
            raise RuntimeError("Live Windows module/child audit unavailable")
        # Only a validated integer PID enters this script. It inspects our own
        # live child and does not act on application windows or other processes.
        script = f"$p=Get-Process -Id {process.pid}; $modules=@($p.Modules | ForEach-Object {{$_.ModuleName}}); $children=@(Get-CimInstance Win32_Process -Filter 'ParentProcessId={process.pid}' | Select-Object -ExpandProperty Name); @{{modules=$modules;children=$children}} | ConvertTo-Json -Compress"
        observation = subprocess.run([shell, "-NoProfile", "-Command", script], capture_output=True,
                                     timeout=30, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        if observation.returncode:
            raise RuntimeError(observation.stderr.decode("utf-8", errors="replace"))
        live = json.loads(observation.stdout)
        process.communicate(b"release\n", timeout=10)
        if process.returncode:
            raise RuntimeError("Audited executable failed on release")
        return {"document_report": report, "loaded_modules": live["modules"], "children": live["children"]}
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=10)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "standalone/target/release/om9.exe")
    parser.add_argument("--output", type=Path, default=ROOT / "docs/validation/2026-10-13-standalone-process.json")
    arguments = parser.parse_args()
    binary = arguments.binary.resolve(strict=True)
    environment = {key: os.environ[key] for key in ("SystemRoot", "TEMP", "TMP") if key in os.environ}
    scratch=ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True,exist_ok=True)
    environment.update(TEMP=str(scratch),TMP=str(scratch))
    with tempfile.TemporaryDirectory(prefix="om9-standalone-profile-",dir=scratch) as temporary:
        profile = Path(temporary)
        model = profile / "authored-fixture.om9"
        created = run(binary, ["--create-fixture", str(model)], profile, environment)
        reopened = run(binary, ["--inspect", str(model)], profile, environment)
        if created != reopened or reopened["target"] != "OM9Standalone" or reopened["restoration"] != "ready":
            raise AssertionError("Cold process changed authored semantic document")
        objects = {row["label"]: row for row in reopened["objects"]}
        if set(objects) != {"Point", "Circle", "Mesh", "Cloud"}:
            raise AssertionError("Fixture type/count mismatch")
        if objects["Point"]["geometry"] != {"Point": [1.0, 2.0, 3.0]}:
            raise AssertionError("Point geometry mismatch")
        if objects["Circle"]["geometry"]["Curve"]["Circle"]["radius"] != 10.0:
            raise AssertionError("Circle radius mismatch")
        if objects["Mesh"]["geometry"]["Mesh"]["faces"] != [[0, 1, 2]]:
            raise AssertionError("Mesh topology mismatch")
        if objects["Cloud"]["geometry"]["Cloud"]["hidden"] != [False, True]:
            raise AssertionError("Cloud channel mismatch")
        live = live_audit(binary, model, profile, environment)
        imports = pe_imports(binary)
        forbidden = ("freecad", "qymcad", "python", "coin", "soqt", "part.dll")
        names = imports + live["loaded_modules"] + live["children"]
        if any(any(word in name.lower() for word in forbidden) for name in names):
            raise AssertionError("Legacy/upstream application dependency observed")
        if live["children"]:
            raise AssertionError("Undeclared application child process")
        sources = []
        for path in sorted((ROOT / "standalone").rglob("*")):
            if path.is_file() and "target" not in path.parts and path.suffix in {".rs", ".toml", ".lock", ".json"}:
                sources.append({"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)})
        receipt = {"schema_version": 1, "target": "OM9Standalone",
                   "timestamp": dt.datetime.now(dt.timezone(dt.timedelta(hours=7))).isoformat(timespec="seconds"),
                   "status": "validated_supported_headless_slice", "binary": {"path": binary.relative_to(ROOT).as_posix(), "sha256": sha256(binary)},
                   "sources": sources, "fixture": {"sha256": sha256(model), "objects": 4, "cold_process_semantic_equal": True},
                   "pe_imports": imports, "loaded_modules": live["loaded_modules"], "children": live["children"],
                   "limitations": ["No Matrix9 GUI/window/pointer acceptance", "No native CAD/openNURBS/OCCT backend", "No 3DM or Rhino clipboard interoperability", "No full M01-M12 completion or performance claim"]}
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"status": receipt["status"], "objects": 4, "imports": imports,
                          "loaded_module_count": len(live["loaded_modules"]), "children": live["children"],
                          "receipt": str(arguments.output.relative_to(ROOT))}))


if __name__ == "__main__":
    main()
