#!/usr/bin/env python3
"""Đóng gói add-on OM9 bằng Python (cho người viết add-on chưa có om9-addon/om9.exe).

Bộ kiểm tra chính thức là host Rust (`om9-addon validate`, cũng là bộ cài trong om9.exe).
Script này kiểm tra nhanh các lỗi phổ biến và **không bao giờ dễ dãi hơn host**:
gói nào script từ chối thì host cũng từ chối (được kiểm bằng test liên thông).

    om9_addon.py build    <thư_mục_nguồn> [-o out.zip] [--catalog MainMenu.ini]
    om9_addon.py validate <file.zip> [--catalog MainMenu.ini]

Chỉ dùng thư viện chuẩn Python 3.11+.
"""
from __future__ import annotations

import argparse
import hashlib
import re
import sys
import tomllib
import zipfile
import zlib
from pathlib import Path, PurePosixPath

MANIFEST = "om9_addon.toml"
RECEIPT = ".om9_receipt.toml"
SCHEMA_MAJOR = 1
TYPES = {"add-on": {"python", "wasm", "native"}, "library": {"data"}}
PLATFORMS = {"windows-x64", "linux-x64", "macos-arm64", "any"}
PERMISSIONS = {"files", "network", "clipboard"}
ID_RE = re.compile(r"^[a-z][a-z0-9_]{1,63}$")
SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
CMD_RE = re.compile(r"^[A-Za-z][A-Za-z0-9_ ]{0,95}$")
DEFAULT_EXCLUDES = [".*", "__pycache__/", "*.pyc", "*.zip"]
MAX_FILES = 2000
MAX_UNPACKED = 200 << 20
MAX_RATIO = 200
WIN_RESERVED = {"con", "prn", "aux", "nul", *(f"com{i}" for i in range(1, 10)), *(f"lpt{i}" for i in range(1, 10))}


class AddonError(Exception):
    pass


def semver(s) -> tuple[int, int, int]:
    m = SEMVER_RE.match(s if isinstance(s, str) else "")
    if not m:
        raise AddonError(f"phiên bản không hợp lệ (cần X.Y.Z): {s!r}")
    return tuple(int(x) for x in m.groups())  # type: ignore[return-value]


def load_catalog(path: Path | None) -> set[str] | None:
    if path is None:
        return None
    codes = {m.group(1) for m in re.finditer(r"(?m)^\s*Icon\d+\s*=\s*(.+?)\s*$", path.read_text(encoding="utf-8-sig"))}
    if not codes:
        raise AddonError(f"catalog rỗng hoặc sai định dạng: {path}")
    return codes


def safe_rel(p: str) -> bool:
    return bool(p) and not p.startswith("/") and "\\" not in p and "\0" not in p \
        and not re.match(r"^[A-Za-z]:", p) and all(c not in ("", ".", "..") for c in p.split("/"))


def excluded(rel: str, patterns: list[str]) -> bool:
    """Cùng ngữ nghĩa với crate om9-addons (glob.rs): `dir/`, mẫu không '/', mẫu từ gốc."""
    parts = rel.split("/")
    for pat in (p.strip() for p in patterns):
        if not pat:
            continue
        def wm(p, s):  # * và ? không vượt '/'
            rx = "".join("[^/]*" if ch == "*" else "[^/]" if ch == "?" else re.escape(ch) for ch in p)
            return re.fullmatch(rx, s) is not None
        if pat.endswith("/"):
            d = pat[:-1].lstrip("/")
            if "/" in d:
                if rel.startswith(d + "/"):
                    return True
            elif any(wm(d, c) for c in parts[:-1]):
                return True
        elif "/" in pat:
            if wm(pat.lstrip("/"), rel):
                return True
        elif any(wm(pat, c) for c in parts):
            return True
    return False


def check_manifest(data: bytes, names: set[str], catalog: set[str] | None) -> dict:
    try:
        m = tomllib.loads(data.decode("utf-8-sig"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as e:
        raise AddonError(f"{MANIFEST} lỗi cú pháp: {e}") from None
    for k in ("schema_version", "id", "version", "name", "license", "om9_version_min"):
        if k not in m:
            raise AddonError(f"thiếu trường bắt buộc: {k}")
    if semver(m["schema_version"])[0] != SCHEMA_MAJOR:
        raise AddonError(f"schema_version {m['schema_version']} không tương thích")
    if not isinstance(m["id"], str) or not ID_RE.match(m["id"]):
        raise AddonError(f"id không hợp lệ: {m['id']!r}")
    semver(m["version"])
    semver(m["om9_version_min"])
    typ = m.get("type", "add-on")
    if typ not in TYPES:
        raise AddonError(f"type {typ!r} không hợp lệ")
    rt = m.get("runtime", "data" if typ == "library" else None)
    if rt not in TYPES[typ]:
        raise AddonError(f"type {typ} cần runtime thuộc {sorted(TYPES[typ])}, nhận {rt!r}")
    lic = m["license"]
    if not isinstance(lic, list) or not lic or not all(isinstance(x, str) and x.startswith("SPDX:") and len(x) > 5 for x in lic):
        raise AddonError('license phải là danh sách "SPDX:<mã>"')
    plats = m.get("platforms", ["any"])
    if not isinstance(plats, list) or not plats or set(plats) - PLATFORMS:
        raise AddonError(f"platforms phải là danh sách con của {sorted(PLATFORMS)}")
    if rt == "native" and "any" in plats:
        raise AddonError("runtime native phải khai báo nền tảng cụ thể")
    if rt != "data":
        e = m.get("entry")
        if not isinstance(e, str) or not safe_rel(e) or e not in names:
            raise AddonError(f"entry {e!r} không có trong gói")
    perms = m.get("permissions", {})
    if not isinstance(perms, dict) or set(perms) - PERMISSIONS or not all(isinstance(v, str) and v.strip() for v in perms.values()):
        raise AddonError("permissions phải là { files|network|clipboard = \"lý do\" }")
    cmds = m.get("commands", [])
    if typ == "add-on" and not cmds:
        raise AddonError("add-on phải khai báo ít nhất một [[commands]]")
    if typ == "library" and cmds:
        raise AddonError("library không có lệnh")
    seen = set()
    for c in cmds:
        cid = c.get("id")
        if not isinstance(cid, str) or not CMD_RE.match(cid) or cid in seen:
            raise AddonError(f"mã lệnh không hợp lệ hoặc lặp: {cid!r}")
        seen.add(cid)
        if catalog is not None and cid not in catalog and not c.get("new", False):
            raise AddonError(f"mã lệnh {cid!r} không có trong catalog")
        if not isinstance(c.get("handler"), str) or not c["handler"].strip():
            raise AddonError(f"lệnh {cid} thiếu handler")
        if "icon" in c and c["icon"] not in names:
            raise AddonError(f"icon {c['icon']!r} của {cid} không có trong gói")
    keys = set()
    for s in m.get("settings", []):
        k = s.get("key")
        if not isinstance(k, str) or not re.fullmatch(r"[a-z][a-z0-9_]{0,47}", k) or k in keys or "default" not in s:
            raise AddonError(f"setting không hợp lệ: {k!r}")
        keys.add(k)
    for a in m.get("assets", []):
        p = str(a.get("path", "")).rstrip("/")
        if not safe_rel(p) or not (p in names or any(n.startswith(p + "/") for n in names)):
            raise AddonError(f"asset path {p!r} không có trong gói")
    if typ == "library" and not m.get("assets"):
        raise AddonError("library phải khai báo ít nhất một [[assets]]")
    return m


def inspect_zip(zpath: Path, catalog: set[str] | None) -> tuple[dict, list[str]]:
    try:
        zf = zipfile.ZipFile(zpath)
    except zipfile.BadZipFile:
        raise AddonError(f"không phải file zip hợp lệ: {zpath}") from None
    with zf:
        infos = [i for i in zf.infolist() if not i.is_dir()]
        if len(infos) > MAX_FILES:
            raise AddonError("quá nhiều file")
        names = [i.filename for i in infos]
        root = MANIFEST in names
        top = {n.split("/", 1)[0] for n in names}
        prefix = ""
        if not root and len(top) == 1 and all("/" in n for n in names) and f"{next(iter(top))}/{MANIFEST}" in names:
            prefix = next(iter(top)) + "/"
        total, seen, lower = 0, set(), set()
        for i in zf.infolist():
            if not safe_rel(i.filename.rstrip("/")):
                raise AddonError(f"đường dẫn không an toàn trong zip: {i.filename!r}")
            if (i.external_attr >> 16) & 0o170000 == 0o120000:
                raise AddonError(f"không cho phép symlink: {i.filename}")
            if any(c.split(".")[0].lower() in WIN_RESERVED or c.endswith((".", " ")) or set(c) & set(':*?"<>|')
                   for c in i.filename.rstrip("/").split("/")):
                raise AddonError(f"tên file không dùng được trên Windows: {i.filename}")
            if i.is_dir():
                continue
            n = i.filename[len(prefix):]
            if n == RECEIPT or n in seen or n.lower() in lower:
                raise AddonError(f"file lặp/không hợp lệ: {n}")
            seen.add(n); lower.add(n.lower())
            total += i.file_size
            if i.file_size > 1 << 20 and (i.compress_size == 0 or i.file_size / i.compress_size > MAX_RATIO):
                raise AddonError(f"tỉ lệ nén bất thường (nghi zip bomb): {i.filename}")
        if total > MAX_UNPACKED:
            raise AddonError("dung lượng giải nén quá lớn")
        if MANIFEST not in seen:
            raise AddonError(f"thiếu {MANIFEST} ở gốc gói")
        try:
            bad = zf.testzip()
            if bad:
                raise AddonError(f"file hỏng CRC: {bad}")
            mdata = zf.read(prefix + MANIFEST)
        except (RuntimeError, NotImplementedError, zipfile.BadZipFile, zlib.error) as e:
            raise AddonError(f"không đọc được gói (mã hoá hoặc hỏng): {e}") from None
        m = check_manifest(mdata, seen, catalog)
    return m, sorted(seen)


def cmd_build(src: Path, out: Path | None, catalog: set[str] | None) -> Path:
    mpath = src / MANIFEST
    if not mpath.is_file():
        raise AddonError(f"{src} thiếu {MANIFEST}")
    pre = tomllib.loads(mpath.read_text(encoding="utf-8-sig"))
    patterns = DEFAULT_EXCLUDES + list(pre.get("build", {}).get("paths_exclude_pattern", []))
    files = sorted(p for p in src.rglob("*") if p.is_file() and not p.is_symlink()
                   and (p.name == MANIFEST and p.parent == src or not excluded(p.relative_to(src).as_posix(), patterns)))
    names = {p.relative_to(src).as_posix() for p in files}
    m = check_manifest(mpath.read_bytes(), names, catalog)
    out = out or src.parent / f"{m['id']}-{m['version']}.zip"
    tmp = out.with_suffix(".zip.part")
    with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zf:
        for p in files:  # tất định: thứ tự + thời gian cố định
            zi = zipfile.ZipInfo(p.relative_to(src).as_posix(), date_time=(2020, 1, 1, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = 0o100644 << 16
            zi.create_system = 3
            zf.writestr(zi, p.read_bytes())
    tmp.replace(out)
    inspect_zip(out, catalog)
    return out


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="om9_addon", description="Đóng gói / kiểm tra add-on OpenMatrix9")
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build"); b.add_argument("src", type=Path); b.add_argument("-o", type=Path)
    v = sub.add_parser("validate"); v.add_argument("zip", type=Path)
    for p in (b, v):
        p.add_argument("--catalog", type=Path)
    a = ap.parse_args(argv)
    try:
        cat = load_catalog(a.catalog)
        if a.cmd == "build":
            out = cmd_build(a.src, a.o, cat)
            print(f"Đã tạo {out} (sha256 {sha256(out)})")
        else:
            m, names = inspect_zip(a.zip, cat)
            print(f"OK: {m['id']} {m['version']} [{m.get('type', 'add-on')}] {len(names)} file, "
                  f"{len(m.get('commands', []))} lệnh, {len(m.get('settings', []))} setting")
    except AddonError as e:
        print(f"Lỗi: {e}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
