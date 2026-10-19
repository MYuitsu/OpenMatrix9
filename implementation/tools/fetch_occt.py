#!/usr/bin/env python
"""Materialise the pinned OCCT 8.0.1 SDK from the pin in `native/occt/pin.json`.

The pin lists three upstream artifacts (source archive, Windows release, Windows 3rdparty) with
their SHA-256. This tool downloads whatever is missing, verifies every digest, and lays the
release out in the SDK shape `tools/build_occt_offset_d3.py` expects:

    <sdk>/include/opencascade/*.hxx
    <sdk>/lib/*.lib
    <sdk>/bin/*.dll

It never modifies the artifacts it verifies and never fetches anything the pin does not name.

    python implementation/tools/fetch_occt.py                 # report what is present/missing
    python implementation/tools/fetch_occt.py --fetch         # download, verify, materialise
    python implementation/tools/fetch_occt.py --fetch --verify-only
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import shutil
import sys
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
PIN = ROOT / "native/occt/pin.json"
DEPS = ROOT / "build/standalone-deps"


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download(url: str, target: pathlib.Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_suffix(target.suffix + ".part")
    print(f"  downloading {target.name} ({url})")
    with urllib.request.urlopen(url) as response, temporary.open("wb") as out:
        shutil.copyfileobj(response, out, 1 << 20)
    temporary.replace(target)


def verify(path: pathlib.Path, expected: dict, label: str) -> bool:
    if not path.exists():
        print(f"  MISSING {label}: {path.relative_to(ROOT)}")
        return False
    actual = sha256(path)
    if actual != expected["sha256"]:
        print(f"  DIGEST MISMATCH {label}: {actual} != {expected['sha256']}")
        return False
    if "bytes" in expected and path.stat().st_size != expected["bytes"]:
        print(f"  SIZE MISMATCH {label}: {path.stat().st_size} != {expected['bytes']}")
        return False
    print(f"  ok {label}: sha256 {actual[:16]}…, {path.stat().st_size} B")
    return True


def materialise(sdk: pathlib.Path, release: pathlib.Path) -> None:
    """Unpack the release archive into the SDK layout, replacing only the three known trees."""
    staging = sdk.with_name(sdk.name + ".staging")
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    with zipfile.ZipFile(release) as archive:
        archive.extractall(staging)
    # The published "release" archive contains a second zip with the actual package, so unpack
    # nested archives (at most two levels) before looking for the package root.
    for _ in range(2):
        nested = [p for p in staging.rglob("*.zip")]
        if not nested:
            break
        for inner in nested:
            with zipfile.ZipFile(inner) as archive:
                archive.extractall(inner.parent)
            inner.unlink()
    # Find the folder that has inc/ and win64/.
    roots = [p for p in staging.rglob("inc") if (p.parent / "win64").is_dir()]
    if len(roots) != 1:
        raise SystemExit(f"cannot locate the OCCT package root under {staging}: {roots}")
    package = roots[0].parent
    for name, source, pattern in (
        ("include/opencascade", package / "inc", "*"),
        ("lib", package / "win64/vc14/lib", "*.lib"),
        ("bin", package / "win64/vc14/bin", "*.dll"),
    ):
        target = sdk / name
        target.mkdir(parents=True, exist_ok=True)
        count = 0
        for file in source.glob(pattern):
            shutil.copy2(file, target / file.name)
            count += 1
        print(f"  {name}: {count} files")
    for notice in ("LICENSE_LGPL_21.txt", "OCCT_LGPL_EXCEPTION.txt"):
        candidate = package / notice
        if candidate.exists():
            shutil.copy2(candidate, sdk / notice)
    shutil.rmtree(staging)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fetch", action="store_true", help="download missing artifacts")
    parser.add_argument("--verify-only", action="store_true", help="never materialise the SDK")
    args = parser.parse_args()
    pin = json.loads(PIN.read_text(encoding="utf-8"))
    print(f"OCCT {pin['version']} (tag {pin['tag']}, commit {pin['commit']})")
    DEPS.mkdir(parents=True, exist_ok=True)
    targets = {
        "source_archive": DEPS / "occt-V8_0_1-source.zip",
        "windows_release": DEPS / "occt-8.0.1-release-no-pch.zip",
        "windows_3rdparty": DEPS / "occt-8.0.1-3rdparty-vc14-64.zip",
    }
    ok = True
    for key, path in targets.items():
        expected = pin["sources"][key]
        if not path.exists() and args.fetch:
            download(expected["url"], path)
        ok = verify(path, expected, key) and ok
    patch = ROOT / pin["accuracy_patch"]["path"]
    if patch.exists():
        actual = sha256(patch)
        expected = pin["accuracy_patch"]["sha256"]
        if actual != expected:
            print(f"  DIGEST MISMATCH accuracy patch: {actual} != {expected}")
            ok = False
        else:
            print(f"  ok accuracy patch: {actual[:16]}…")
    else:
        print("  MISSING accuracy patch")
        ok = False
    if not ok:
        print("pin verification failed: fix the artifacts before building")
        return 1
    sdk = DEPS / f"occt-{pin['version']}-sdk"
    if not args.verify_only:
        if sdk.exists():
            print(f"  SDK already materialised at {sdk.relative_to(ROOT)}")
        else:
            print(f"  materialising SDK at {sdk.relative_to(ROOT)}")
            materialise(sdk, targets["windows_release"])
    print("\nnext steps:")
    print(f"  1. patched accuracy build (VS2022 native tools, Ninja):")
    print(f"     python tools/build_occt_offset_d3.py --archive {targets['source_archive']} "
          f"--sdk {sdk} --output {DEPS / ('occt-' + pin['version'] + '-offset-d3')}")
    print(f"  2. gate probe: cmake -S build/native-occt -B build/native-occt/out "
          f"-DOM9_OCCT_ROOT={sdk}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
