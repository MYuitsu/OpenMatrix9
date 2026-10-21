# SPDX-License-Identifier: LGPL-2.1-or-later
"""Package the current MSVC standalone build; never infer UI/runtime test acceptance."""
import argparse
from contextlib import ExitStack, contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import zipfile

SOURCE = Path(__file__).resolve().parents[2]


def sha(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def dependencies(path):
    result = subprocess.run(["dumpbin.exe", "/nologo", "/dependents", str(path)],
                            check=True, capture_output=True, text=True)
    return re.findall(r"^\s*([A-Za-z0-9_.+-]+\.dll)\s*$", result.stdout, re.M | re.I)


def copy_dependency_closure(runtime, candidates, system, read_dependencies=dependencies):
    """App-local dependencies take precedence over a developer machine's system CRT."""
    pending = list(runtime.glob("*.exe"))
    visited = set()
    while pending:
        binary = pending.pop()
        if binary.name.lower() in visited:
            continue
        visited.add(binary.name.lower())
        for name in read_dependencies(binary):
            key = name.lower()
            if key.startswith(("api-ms-win-", "ext-ms-win-")):
                continue
            target = runtime / name
            source = candidates.get(key)
            if source:
                if not target.exists():
                    shutil.copy2(source, target)
                pending.append(target)
            elif not (system / name).is_file():
                raise RuntimeError(f"Unresolved runtime dependency: {binary.name} -> {name}")


def verify_manifest(runtime, files):
    actual = {p.relative_to(runtime).as_posix(): sha(p) for p in runtime.rglob("*")
              if p.is_file() and p.name != "release-validation.json"}
    if actual != files:
        raise RuntimeError("Runtime payload changed after build identity was recorded")


@contextmanager
def vendor_archive(path, staging):
    """The pinned upstream download wraps its actual package in one inner ZIP."""
    with ExitStack() as stack:
        archive = stack.enter_context(zipfile.ZipFile(path))
        for depth in range(3):
            if any(n.lower().endswith(".dll") for n in archive.namelist()):
                yield archive
                return
            nested = [n for n in archive.namelist() if n.lower().endswith(".zip")]
            if len(nested) != 1 or depth == 2:
                raise RuntimeError("Cannot locate a unique third-party DLL archive")
            inner = staging / f"vendor-{depth}.zip"
            inner.write_bytes(archive.read(nested[0]))
            archive = stack.enter_context(zipfile.ZipFile(inner))


def package(root, source=SOURCE, environ=None):
    environ = os.environ if environ is None else environ
    paths = json.loads((root / "build-paths.json").read_text(encoding="utf-8-sig"))
    pin = json.loads((source / "native/occt/pin.json").read_text(encoding="utf-8"))
    release = json.loads((source / "release.json").read_text(encoding="utf-8"))
    if not re.fullmatch(r"[0-9A-Za-z][0-9A-Za-z.+-]*", release["version"]):
        raise RuntimeError("Unsafe release version")
    if paths["target"] != "om9-app/om9" or paths["features"] != ["occt-brep"] or paths["workers"] != 2:
        raise RuntimeError("Expected the default standalone product build")
    if paths["cargo_lock_sha256"] != sha(source / "Cargo.lock"):
        raise RuntimeError("Cargo.lock changed after the build")
    commit = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    dirty = bool(subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip())
    if environ.get("GITHUB_ACTIONS") == "true" and (environ.get("GITHUB_SHA") != commit or dirty):
        raise RuntimeError("CI source identity must match a clean checked-out commit")
    native = Path(paths["native"])
    native_exe = native / "om9-exchange-native.exe"
    if sha(native_exe) != (native / "om9-exchange-native.sha256").read_text().strip().lower():
        raise RuntimeError("Native adapter fingerprint mismatch")
    patch = Path(paths["patch_root"])
    patch_receipt = json.loads((patch / "build-receipt.json").read_text(encoding="utf-8"))
    if (patch_receipt.get("build_exit_code") != 0 or
        patch_receipt["patch"]["sha256"] != pin["accuracy_patch"]["sha256"] or
        patch_receipt["native_dll"]["sha256"] != sha(patch / "bin/TKG3d.dll")):
        raise RuntimeError("Patched OCCT runtime identity mismatch")
    thirdparty = Path(paths["thirdparty_archive"])
    if sha(thirdparty) != pin["sources"]["windows_3rdparty"]["sha256"]:
        raise RuntimeError("OCCT third-party archive pin mismatch")

    runtime = root / "runtime/OpenMatrix9"
    runtime.mkdir(parents=True, exist_ok=False)
    shutil.copy2(paths["binary"], runtime / "om9.exe")
    for name in ("om9-exchange-native.exe", "om9-exchange-native.sha256", "om9-exchange-native.manifest.json"):
        shutil.copy2(native / name, runtime / name)
    candidates = {p.name.lower(): p for directory in (Path(paths["sdk"]) / "bin", Path(paths["crt"]))
                  for p in directory.glob("*.dll")}
    candidates["tkg3d.dll"] = patch / "bin/TKG3d.dll"
    thirdparty_runtime = root / "thirdparty-runtime"
    thirdparty_runtime.mkdir()
    licenses = runtime / "licenses"
    licenses.mkdir()
    with vendor_archive(thirdparty, thirdparty_runtime) as archive:
        for required in pin["sources"]["windows_3rdparty"]["required_runtimes"]:
            matches = [n for n in archive.namelist() if n.replace("\\", "/").endswith(required)]
            if len(matches) != 1:
                raise RuntimeError(f"Missing/ambiguous pinned runtime: {required}")
            target = thirdparty_runtime / Path(required).name
            target.write_bytes(archive.read(matches[0]))
            candidates[target.name.lower()] = target
        for name in archive.namelist():
            relative = Path(name.replace("\\", "/"))
            if relative.is_absolute() or ".." in relative.parts or ":" in name:
                raise RuntimeError("Unsafe third-party archive member")
            if not name.endswith("/") and any(t in relative.name.lower() for t in ("license", "copying", "copyright", "notice")):
                target = licenses / "occt-thirdparty" / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.read(name))
    copy_dependency_closure(runtime, candidates, Path(environ["SystemRoot"]) / "System32")
    for name in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
        shutil.copy2(source / name, runtime / name)
    shutil.copytree(source / "Resources/licenses", licenses / "om9-dependencies")
    shutil.copytree(patch / "upstream", licenses / "occt-TKG3d-source")
    for name in pin["license_files"]:
        shutil.copy2(Path(paths["sdk"]) / name, licenses / name)
    shutil.copy2(source / pin["accuracy_patch"]["path"], licenses / "occt-8.0.1-offset-d3.patch")
    shutil.copy2(source / "tools/build_occt_offset_d3.py", licenses / "build_occt_offset_d3.py")
    for name in ("LICENSE", "LICENSE.txt", "COPYING", "copyright.txt"):
        notice = Path(paths["opennurbs_source"]) / name
        if notice.is_file():
            shutil.copy2(notice, licenses / ("opennurbs-" + name))
    (runtime / "Start-OM9.cmd").write_text('@echo off\r\n"%~dp0om9.exe" %*\r\n', encoding="ascii", newline="")
    (runtime / "README.txt").write_text("OpenMatrix9 standalone Windows x64\n\nExtract the entire folder, then run Start-OM9.cmd or om9.exe.\nA GPU supported by wgpu is required. No FreeCAD or Python runtime is needed.\nThe native 3DM adapter and its digest must remain beside om9.exe.\nCI builds/packages only; UI and geometry acceptance are separate.\n", encoding="utf-8")
    files = {p.relative_to(runtime).as_posix(): sha(p) for p in runtime.rglob("*") if p.is_file()}
    record = dict(schema_version=1, ok=True, target=paths["target"], features=paths["features"],
                  validation_kind="build_only", tests_executed=False, version=release["version"],
                  source_commit=commit, source_dirty=dirty, rustc=paths["rustc"],
                  cargo_lock_sha256=paths["cargo_lock_sha256"], release_metadata_sha256=sha(source / "release.json"),
                  executable_sha256=files["om9.exe"], files=files,
                  occt_version=pin["version"], occt_accuracy_patch_sha256=pin["accuracy_patch"]["sha256"],
                  opennurbs_pin="eb92af3ba1806b0a34a99aba0d3bda83e3d46083",
                  build_url=(f"https://github.com/{environ['GITHUB_REPOSITORY']}/actions/runs/{environ['GITHUB_RUN_ID']}"
                             if environ.get("GITHUB_ACTIONS") == "true" else None))
    (runtime / "release-validation.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    verify_manifest(runtime, files)
    output = root / "packages"
    output.mkdir(exist_ok=False)
    package_path = output / f"OpenMatrix9-{release['version']}-Standalone-Windows-x64.zip"
    with zipfile.ZipFile(package_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(runtime.rglob("*")):
            if path.is_symlink():
                raise RuntimeError("Linked runtime payload")
            if path.is_file():
                archive.write(path, "OpenMatrix9/" + path.relative_to(runtime).as_posix())
    with zipfile.ZipFile(package_path) as archive:
        if archive.testzip():
            raise RuntimeError("Damaged standalone ZIP")
    (output / "SHA256SUMS.txt").write_text(f"{sha(package_path)}  {package_path.name}\n", encoding="ascii")
    evidence = root / "evidence"
    evidence.mkdir()
    shutil.copy2(runtime / "release-validation.json", evidence / "release-validation.json")
    print(json.dumps({"package": str(package_path), "sha256": sha(package_path), "payload_files": len(files)}))
    return package_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    package(parser.parse_args().root.resolve())
