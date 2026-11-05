#!/usr/bin/env python3
"""Sinh fixture cho test Rust (crates/om9-addons/tests/fixtures):

inflate/*.deflate  + inflate/index.toml (độ dài + sha256 dữ liệu gốc)  — sinh bằng zlib để kiểm bộ inflate tự viết
packages/good/*.zip, packages/bad/*.zip + packages/expect.toml (chuỗi lỗi mong đợi)

    python3 tools/gen_fixtures.py <repo_om9_addon_root>
"""
import hashlib
import io
import random
import shutil
import struct
import sys
import warnings
import zipfile
import zlib
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
sys.path.insert(0, str(ROOT / "tools"))
import om9_addon as P  # noqa: E402

FIX = ROOT / "crates/om9-addons/tests/fixtures"


def inflate_fixtures():
    out = FIX / "inflate"
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    rnd = random.Random(20261022)
    text = ("Rải đá tròn đều theo đường cong; khe thực g' = L/n - d. " * 900).encode()
    data = {
        "empty": b"",
        "one": b"a",
        "text": text,
        "random": rnd.randbytes(40000),
        "abc": b"abc" * 12000,
        "zeros_long": b"\0" * 70000,  # độ dài 258 lặp, khoảng cách 1
        "far_match": rnd.randbytes(30000) + b"MARKER" * 5 + rnd.randbytes(2500) + b"MARKER" * 5,  # khoảng cách gần 32K
        "mixed": bytes(range(256)) * 200 + text[:5000],
    }
    variants = [("l0", 0, zlib.Z_DEFAULT_STRATEGY, -15), ("l1", 1, zlib.Z_DEFAULT_STRATEGY, -15),
                ("l6", 6, zlib.Z_DEFAULT_STRATEGY, -15), ("l9", 9, zlib.Z_DEFAULT_STRATEGY, -15),
                ("filtered", 6, zlib.Z_FILTERED, -15), ("huff", 6, zlib.Z_HUFFMAN_ONLY, -15),
                ("rle", 6, zlib.Z_RLE, -15), ("fixed", 6, zlib.Z_FIXED, -15), ("win9", 9, zlib.Z_DEFAULT_STRATEGY, -9)]
    lines = []
    for dn, raw in data.items():
        for vn, lvl, strat, wb in variants:
            c = zlib.compressobj(lvl, zlib.DEFLATED, wb, 9, strat)
            comp = c.compress(raw) + c.flush()
            name = f"{dn}.{vn}"
            (out / f"{name}.deflate").write_bytes(comp)
            lines.append(f'[{name}]\nlen = {len(raw)}\nsha256 = "{hashlib.sha256(raw).hexdigest()}"\n')
    # luồng nhiều block: flush giữa chừng tạo block stored rỗng + nhiều block động
    c = zlib.compressobj(6, zlib.DEFLATED, -15)
    parts = [c.compress(text[:20000]), c.flush(zlib.Z_SYNC_FLUSH), c.compress(text[20000:]), c.flush(zlib.Z_FULL_FLUSH), c.compress(b"end"), c.flush()]
    (out / "multiblock.flush.deflate").write_bytes(b"".join(parts))
    lines.append(f'[multiblock.flush]\nlen = {len(text) + 3}\nsha256 = "{hashlib.sha256(text + b"end").hexdigest()}"\n')
    (out / "index.toml").write_text("\n".join(lines), encoding="utf-8")
    return len(lines)


def zbytes(entries, method=zipfile.ZIP_DEFLATED):
    buf = io.BytesIO()
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        with zipfile.ZipFile(buf, "w", method) as zf:
            for name_or_info, data in entries:
                zf.writestr(name_or_info, data)
    return buf.getvalue()


def info(name, data, mode=None):
    zi = zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
    zi.compress_type = zipfile.ZIP_DEFLATED
    if mode is not None:
        zi.create_system = 3
        zi.external_attr = mode << 16
    return zi, data


def package_fixtures():
    out = FIX / "packages"
    shutil.rmtree(out, ignore_errors=True)
    (out / "good").mkdir(parents=True)
    (out / "bad").mkdir(parents=True)
    cat = P.load_catalog(Path("/tmp/OpenMatrix9/Resources/menu/MainMenu.ini")) if Path("/tmp/OpenMatrix9").exists() else None
    gem = P.cmd_build(ROOT / "examples/gem_on_curve", out / "good/gem_on_curve.zip", cat)
    P.cmd_build(ROOT / "examples/ring_sizes", out / "good/ring_sizes.zip", cat)
    src = zipfile.ZipFile(gem)
    files = {n: src.read(n) for n in src.namelist()}
    man = files["om9_addon.toml"].decode()
    (out / "good/nested_folder.zip").write_bytes(zbytes([(f"gem_on_curve/{n}", d) for n, d in files.items()]))

    def base(**over):
        f = dict(files)
        f.update(over)
        return [(n, d) for n, d in f.items()]

    bad = {
        "zip_slip": (base(**{"../../evil.txt": b"x"}), "không an toàn"),
        "abs_path": (base(**{"/etc/evil": b"x"}), "không an toàn"),
        "backslash": (base(**{"icons\\..\\..\\x.svg": b"x"}), "không an toàn"),
        "dup_names": (base() + [("__init__.py", "# bản thứ hai".encode())], "lặp"),
        "case_collision": (base(**{"ICONS/gem_on_crv.svg": b"<svg/>"}), "hoa/thường"),
        "bomb": (base(**{"big.bin": b"\0" * (6 << 20)}), "zip bomb"),
        "no_manifest": ([(n, d) for n, d in files.items() if n != "om9_addon.toml"], "thiếu om9_addon.toml"),
        "win_reserved": (base(**{"icons/con.svg": b"<svg/>"}), "Windows"),
        "receipt_name": (base(**{".om9_receipt.toml": b"x"}), "dành cho host"),
        "syntax": (base(**{"om9_addon.toml": (man + "\n[[[").encode()}), "cú pháp"),
        "unknown_cmd": (base(**{"om9_addon.toml": man.replace('id = "BuilderGemCountOnCrv"', 'id = "NotARealCommand"').encode()}), "không có trong catalog"),
        "missing_entry": ([(n, d) for n, d in files.items() if n != "__init__.py"], "entry"),
        "bad_setting": (base(**{"om9_addon.toml": man.replace("default = 0.2", "default = 9.0").encode()}), "default"),
        "lib_with_cmd": (base(**{"om9_addon.toml": man.replace('type = "add-on"', 'type = "library"').encode()}), "library"),
        "bad_shortcut": (base(**{"om9_addon.toml": man.replace('"Ctrl+Shift+G"', '"G"').encode()}), "CMD"),
    }
    expect = []
    for name, (entries, msg) in bad.items():
        (out / "bad" / f"{name}.zip").write_bytes(zbytes(entries))
        expect.append(f'{name} = "{msg}"')
    # symlink
    (out / "bad/symlink.zip").write_bytes(zbytes([(n, d) for n, d in files.items()] + [info("link", b"/etc/passwd", 0o120777)]))
    expect.append('symlink = "symlink"')
    # CRC hỏng: zip stored rồi sửa 1 byte dữ liệu
    raw = bytearray(zbytes(base(), zipfile.ZIP_STORED))
    i = raw.find(b"def place_gems")
    raw[i] ^= 0x20
    (out / "bad/crc_corrupt.zip").write_bytes(bytes(raw))
    expect.append('crc_corrupt = "CRC"')
    # bật cờ mã hoá (bit 0) ở mọi local + central header
    raw = bytearray(zbytes(base()))
    for sig, off in ((b"PK\x03\x04", 6), (b"PK\x01\x02", 8)):
        j = raw.find(sig)
        while j != -1:
            struct.pack_into("<H", raw, j + off, struct.unpack_from("<H", raw, j + off)[0] | 1)
            j = raw.find(sig, j + 4)
    (out / "bad/encrypted.zip").write_bytes(bytes(raw))
    expect.append('encrypted = "mã hoá"')
    (out / "bad/not_zip.zip").write_bytes(b"PK this is not a zip file" * 4)
    expect.append('not_zip = "không phải file zip"')
    (out / "expect.toml").write_text("[bad]\n" + "\n".join(expect) + "\n", encoding="utf-8")
    return len(list((out / "good").iterdir())), len(expect)


if __name__ == "__main__":
    n = inflate_fixtures()
    g, b = package_fixtures()
    print(f"inflate: {n} luồng; packages: {g} tốt, {b} xấu → {FIX}")
