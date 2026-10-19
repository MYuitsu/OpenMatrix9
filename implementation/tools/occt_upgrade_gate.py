#!/usr/bin/env python
"""OCCT upgrade gate: decide, on every OCCT bump, whether the local accuracy patch is still needed.

The conformance probe (`native/occt_conformance.cpp`) compares offset-curve D2/D3 against a
Richardson-extrapolated finite difference of the curve point, plus closed-form line/circle answers
and analytic solid invariants. This tool builds and runs that probe against a given OCCT SDK and turns
the result into the one decision that matters:

    PATCH_REQUIRED      the stock kernel still computes the offset derivatives wrongly -> keep the
                        local `TKG3d` patch (`tools/patches/occt-8.0.1-offset-d3.patch`)
    PATCH_NOT_NEEDED    upstream fixed it -> drop the patch, ship the untouched release

It also runs the control case: the patched DLL must report PATCH_NOT_NEEDED, otherwise the patch
itself is ineffective and the failure is in the test, not the kernel.

    python implementation/tools/occt_upgrade_gate.py --sdk build/standalone-deps/occt-8.0.1-sdk
    python implementation/tools/occt_upgrade_gate.py --sdk <new sdk> --patched-dll <dll> --write
"""
from __future__ import annotations

import argparse
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "implementation/evidence/M00/OM9-L2-02"
BUILD = ROOT / "build/native-occt"
EXE = BUILD / "out/Release/om9-occt-conformance.exe"


def run_probe() -> dict:
    result = subprocess.run([str(EXE)], capture_output=True, text=True)
    for line in result.stdout.splitlines():
        if line.startswith("{"):
            return json.loads(line)
    raise SystemExit(f"probe produced no JSON (exit {result.returncode}):\n{result.stdout}\n{result.stderr}")


def build(sdk: pathlib.Path) -> None:
    subprocess.run(
        ["cmake", "-S", str(BUILD), "-B", str(BUILD / "out"), "-G", "Visual Studio 17 2022",
         "-A", "x64", f"-DOM9_OCCT_ROOT={sdk}"],
        check=True, capture_output=True, text=True)
    subprocess.run(
        ["cmake", "--build", str(BUILD / "out"), "--config", "Release", "--parallel", "2"],
        check=True, capture_output=True, text=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sdk", type=pathlib.Path, required=True)
    parser.add_argument("--patched-dll", type=pathlib.Path,
                        help="patched TKG3d.dll used as the control run")
    parser.add_argument("--write", action="store_true", help="store the JSON evidence under implementation/evidence")
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()
    sdk = args.sdk.resolve()
    if not args.skip_build:
        build(sdk)
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    version = "unknown"
    exec_dir = EXE.parent

    stock = run_probe()
    version = stock["occt_version"]
    print(f"stock OCCT {version}: {stock['verdict']}")
    for check in stock["checks"]:
        if "d2" in check["name"] or "d3" in check["name"]:
            print(f"  {check['name']:34} {'PASS' if check['pass'] else 'FAIL'}  {check['measured']:.3e}")

    control = None
    if args.patched_dll:
        original = exec_dir / "TKG3d.dll"
        backup = exec_dir / "TKG3d.dll.stock"
        shutil.copy2(original, backup)
        try:
            shutil.copy2(args.patched_dll.resolve(), original)
            control = run_probe()
        finally:
            shutil.copy2(backup, original)
            backup.unlink()
        print(f"patched control: {control['verdict']}")
        if control["verdict"] != "PATCH_NOT_NEEDED":
            print("  the patch does not fix the checks -> investigate before trusting the gate")
            return 3

    if args.write:
        (EVIDENCE / f"OCCT-CONFORMANCE-{version}-stock.json").write_text(
            json.dumps(stock, indent=2) + "\n", encoding="utf-8")
        if control is not None:
            (EVIDENCE / f"OCCT-CONFORMANCE-{version}-patched.json").write_text(
                json.dumps(control, indent=2) + "\n", encoding="utf-8")
        print(f"evidence written for {version}")

    if stock["verdict"] == "PATCH_NOT_NEEDED":
        print("decision: DROP the local patch and ship the untouched OCCT release")
        return 0
    print("decision: KEEP the local patch (or route offset derivatives through om9-core)")
    return 2


if __name__ == "__main__":
    sys.exit(main())
