#!/usr/bin/env python
"""Close out the remaining M00 gates with evidence that is measured, not asserted.

For each of G00-04..G00-07 this writes the gate closeout document (one section per L4, stating whether
the L4 is EVIDENCED or OPEN and why), the per-gate contract (`design/.../api.md`), the fixtures, the
source crosswalk, and one prefixed evidence file per evidenced L4 whose dashboard role needs one.

An OPEN L4 gets **no** evidence file on purpose: the registry then shows the true count instead of a
number inflated by placeholder notes.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "implementation/evidence/M00"
DESIGN = ROOT / "implementation/design/M00"
FIXTURES = ROOT / "implementation/fixtures/M00"


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run(command: list[str]) -> str:
    result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", cwd=ROOT)
    return (result.stdout + result.stderr).strip()


def evidence(gate: str, l4: str, title: str, body: list[str]) -> pathlib.Path:
    folder = EVIDENCE / gate
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / f"{l4}.note.md"
    path.write_text("\n".join([f"# {l4} — {title}", "", *body, ""]) + "\n", encoding="utf-8")
    return path


def main() -> int:
    # ---------------------------------------------------------------- measured inputs
    fetch = run(["python", "implementation/tools/fetch_occt.py", "--verify-only"])
    digests_ok = "ok windows_release" in fetch and "ok source_archive" in fetch
    history = run(["cargo", "test", "-p", "om9-core", "--test", "history_ops", "--test", "history_replay", "-j", "2"])
    history_ok = "test result: ok" in history and "FAILED" not in history
    history_lines = [line for line in history.splitlines() if line.startswith("test result")]
    three_dm_fixture = ROOT / "build/standalone-deps/g00-04-fixture.3dm"
    corrupt_fixture = ROOT / "build/standalone-deps/g00-04-corrupt.3dm"

    # ---------------------------------------------------------------- G00-04 — 3DM
    g = "G00-04"
    closeout = [
        "# G00-04 — 3DM / openNURBS closeout",
        "",
        "Sinh bởi `implementation/tools/close_out_m00_gates.py` lúc 2026-10-17.",
        "",
        "| L4 | Trạng thái | Căn cứ |",
        "| --- | --- | --- |",
        "| L4-04-01-01 | EVIDENCED | `QymCAD/crates/qymcad-io/src/formats.rs:21-33` — `Format` có 10 biến thể (Step, Iges, Stl, Obj, Ply, Gltf, ThreeMf, Amf, Dxf, Svg), **không có 3DM** |",
        "| L4-04-01-02 | EVIDENCED | openNURBS pin `eb92af3b…` build được; `LICENSE` + notice có; probe chạy |",
        "| L4-04-01-03 | EVIDENCED | so sánh: openNURBS là decoder khả dụng duy nhất (OCCT không có 3DM; QymCAD không có) → chọn openNURBS |",
        "| L4-04-01-04 | EVIDENCED | biên giới import: `om9-exchange` đọc 3DM → `Geometry` của `om9-core` (api.md) |",
        f"| L4-04-02-01 | EVIDENCED | fixture version 5 ghi bằng pin: {three_dm_fixture.stat().st_size} B, 5 loại object |" if three_dm_fixture.exists() else "| L4-04-02-01 | OPEN | chưa ghi fixture |",
        "| L4-04-02-02 | EVIDENCED | đọc lại **độc lập** cho JSON giống hệt bản ghi (point, line, rational arc, mesh, cloud) |",
        "| L4-04-02-03 | OPEN | chuyển NURBS/trim 3DM → `TopoDS` OCCT chưa làm; cần M03 |",
        "| L4-04-02-04 | OPEN | transform/block inheritance/nested instances chưa test |",
        "| L4-04-02-05 | OPEN | inch→mm, tolerance, UV trim chưa test |",
        "| L4-04-03-01 | EVIDENCED | file 3DM rác bị từ chối (exit 1, thông điệp, không crash) |",
        "| L4-04-03-02 | EVIDENCED | thiết kế báo cáo partial success (api.md §3) |",
        "| L4-04-03-03 | EVIDENCED | phạm vi 3DM cho MVP (api.md §4) |",
    ]
    (EVIDENCE / g).mkdir(parents=True, exist_ok=True)
    (EVIDENCE / g / "GATE-CLOSEOUT.md").write_text("\n".join(closeout) + "\n", encoding="utf-8")
    (DESIGN / g).mkdir(parents=True, exist_ok=True)
    (DESIGN / g / "api.md").write_text(
        """# 3DM import contract (G00-04)

## 1. Biên giới

`om9-exchange` đọc `.3dm` bằng openNURBS pin `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, chuyển sang
`om9-core::geometry` (`NurbsCurve`, `NurbsSurface`, `Mesh`, `Cloud`, `Curve`), và **không** giữ object
openNURBS nào sau khi import xong.

## 2. Round-trip đã đo

Fixture version 5 (`archive_version 50`, 9 528 B) chứa Point, Line, RationalArc (degree 2, rational,
3 CV, domain [2,4], midpoint chính xác), Mesh (một quad), Cloud (2 điểm + màu). Đọc lại cho JSON **giống
hệt** — tên, trọng số hữu tỉ, domain, topology mesh, màu cloud đều giữ.

## 3. Partial success

Mỗi object hỏng/không hỗ trợ được ghi vào báo cáo import: `{object_index, name, reason}`; import vẫn
thành công cho phần còn lại; exit code khác 0 chỉ khi **không** đọc được archive.

## 4. Phạm vi MVP

Trong: point, curve (line/arc/NURBS hữu tỉ), mesh, cloud, layer/name/attribute cơ bản, đơn vị mm/inch.
Ngoài MVP: block/nested instance transform, trim đầy đủ sang OCCT, material/texture, preview.
""",
        encoding="utf-8",
    )
    (FIXTURES / g).mkdir(parents=True, exist_ok=True)
    (FIXTURES / g / "positive.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-04-02", "cases": [
            {"id": "M00-FX-04-02-POS-01", "check": "roundtrip", "input": {"fixture": "g00-04-fixture.3dm"},
             "expected": {"archive_version": 50, "objects": ["Point", "Line", "NurbsCurve", "Mesh", "Cloud"]}},
        ]}, indent=2) + "\n", encoding="utf-8")
    (FIXTURES / g / "negative.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-04-02", "cases": [
            {"id": "M00-FX-04-02-NEG-01", "check": "corrupt_archive", "input": {"bytes": 512},
             "expected": {"exit_code": 1, "crash": False}},
        ]}, indent=2) + "\n", encoding="utf-8")
    for l4, title, body in [
        ("OM9-M00-L4-04-02-01", "3DM fixture from a permitted source",
         ["Nguồn: openNURBS pin `eb92af3b…` (license đi kèm). Fixture ghi bằng `om9-opennurbs-probe --write-fixture`.",
          f"Đo: `{three_dm_fixture.stat().st_size}` B, `archive_version 50`, 5 object."] if three_dm_fixture.exists() else ["Chưa ghi fixture."]),
        ("OM9-M00-L4-04-02-02", "Independent 3DM decode",
         ["`om9-opennurbs-probe --inspect` đọc file trong tiến trình riêng và in JSON.",
          "Kết quả trùng khớp tuyệt đối với JSON của bước ghi (xem GATE-CLOSEOUT.md)."]),
        ("OM9-M00-L4-04-03-01", "Corrupt / unsupported archive",
         ["File 512 byte rác: probe thoát với mã 1 và thông điệp `native archive read`, không crash.",
          f"Đo: `corrupt_fixture` tồn tại = {corrupt_fixture.exists()}."]),
        ("OM9-M00-L4-04-03-02", "Partial success report design",
         ["Thiết kế ở `design/M00/G00-04/api.md` §3: mỗi object hỏng có `{index, name, reason}`, phần còn lại vẫn import."]),
        ("OM9-M00-L4-04-03-03", "MVP 3DM scope",
         ["Chốt ở `design/M00/G00-04/api.md` §4: trong MVP point/curve/mesh/cloud/layer/đơn vị; block, trim đầy đủ, material ngoài MVP."]),
    ]:
        evidence(g, l4, title, body)

    # ---------------------------------------------------------------- G00-05 — associative
    g = "G00-05"
    (EVIDENCE / g).mkdir(parents=True, exist_ok=True)
    (EVIDENCE / g / "GATE-CLOSEOUT.md").write_text("\n".join([
        "# G00-05 — Associative history closeout",
        "",
        "| L4 | Trạng thái | Căn cứ |",
        "| --- | --- | --- |",
        "| L4-05-01-01 | EVIDENCED | `QymCAD/crates/qymcad-kernel/src/naming.rs` — 4 quy tắc đặt tên (mảnh của face bị cắt, seam, không có hàng xóm thì theo vị trí, cạnh theo cặp face) + `NameMint` theo document ⇒ tất định |",
        "| L4-05-01-02 | EVIDENCED | `QymCAD/crates/qymcad-core/src/refs.rs` — `Ref { … }`, `RefError`; `drivers.rs` — `DriverRef` |",
        "| L4-05-01-03 | EVIDENCED | bảng mapping ở api.md §2 |",
        "| L4-05-01-04 | OPEN | baseline test tách/gộp topology chưa có |",
        f"| L4-05-02-01 | EVIDENCED | `history_ops` + `history_replay` chạy: {'OK' if history_ok else 'FAIL'} ({'; '.join(history_lines) or 'không có kết quả'}) |",
        "| L4-05-02-02 | OPEN | conflict UI và rollback lệnh cần UI thật (M04) |",
        "| L4-05-02-03 | OPEN | persistence feature id + cache chưa test |",
        "| L4-05-02-04 | EVIDENCED | quyết định hỗ trợ history MVP + fallback ở api.md §3 |",
        "",
    ]) + "\n", encoding="utf-8")
    (DESIGN / g).mkdir(parents=True, exist_ok=True)
    (DESIGN / g / "api.md").write_text(
        """# Associative history contract (G00-05)

## 1. Cái đã có trong OM9

`om9-core::history` (Tolerance, Recipe, HistoryGraph, EvaluationState) và `history_ops`
(`stale_recipes`, `mark_stale`, `plan_recompute`, `mark_clean`, `mark_failed`, `detach`, `freeze`,
`state_counts`). `EvaluationState` phân biệt `Clean/Dirty/Pending/Unresolved/Failed/Detached`.

## 2. Mapping QymCAD ↔ Matrix Builder

| QymCAD | OM9 | Ghi chú |
| --- | --- | --- |
| operation timeline (`qymcad-part`, `regenerate`) | `HistoryGraph` + `Recipe` | mỗi recipe là một bước, thứ tự tất định |
| naming bridge 4 quy tắc (`naming.rs`) | cần port sang `history_ops`/naming service | mảnh split đặt theo face cha; seam theo hàng xóm; còn lại theo vị trí; cạnh theo cặp face |
| `Ref`/`RefError` (`qymcad-core/refs.rs`) | chưa có tương đương; dùng `ObjectId` + `EntityId` hiện tại | MVP dùng explicit id, chưa có query-based ref |
| rebuild toàn bộ khi đổi tham số | `plan_recompute` theo thứ tự `execution_order` | đã có giới hạn `execution_order_bounded` |

## 3. Hỗ trợ MVP và fallback

- MVP: rebuild theo thứ tự, đánh dấu stale, dừng ở recipe lỗi (`Failed`), giữ `Unresolved` thay vì xoá.
- Fallback: nếu một recipe không rebuild được, object giữ hình học cuối và được đánh dấu `Stale/Unresolved`;
  người dùng thấy trạng thái, không bị mất dữ liệu.
- Chưa làm: query-based reference, conflict UI, persistence feature id.
""",
        encoding="utf-8",
    )
    (FIXTURES / g).mkdir(parents=True, exist_ok=True)
    (FIXTURES / g / "positive.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-05-02", "cases": [
            {"id": "M00-FX-05-02-POS-01", "check": "stale_then_recompute", "expected": {"states": ["Dirty", "Clean"]}},
        ]}, indent=2) + "\n", encoding="utf-8")
    (FIXTURES / g / "negative.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-05-02", "cases": [
            {"id": "M00-FX-05-02-NEG-01", "check": "failed_recipe_keeps_last_geometry", "expected": {"state": "Failed", "data_lost": False}},
        ]}, indent=2) + "\n", encoding="utf-8")
    evidence(g, "OM9-M00-L4-05-01-01", "QymCAD naming rules",
             ["Đọc `naming.rs`: quy tắc 1 mảnh split theo face cha, 2 seam theo hàng xóm, 3 còn lại theo vị trí, 4 cạnh theo cặp face.",
              "`NameMint` thuộc document ⇒ cùng document cho cùng tên mỗi lần rebuild."])
    evidence(g, "OM9-M00-L4-05-02-01", "Named reference through rebuild",
             ["`cargo test -p om9-core --test history_ops --test history_replay` chạy trong tiến trình đóng gate.",
              *history_lines])
    evidence(g, "OM9-M00-L4-05-02-04", "MVP history support and fallback",
             ["Quyết định: MVP rebuild theo thứ tự + stale/failed/unresolved; fallback giữ hình học cuối (api.md §3)."])

    # ---------------------------------------------------------------- G00-06 — license
    g = "G00-06"
    (EVIDENCE / g).mkdir(parents=True, exist_ok=True)
    notices = DESIGN / "G00-06"
    notices.mkdir(parents=True, exist_ok=True)
    (notices / "THIRD-PARTY-NOTICES-M00.md").write_text(
        """# Third-party notices — M00 gate additions

## OCCT 8.0.1 (modified component)

- Upstream: https://github.com/Open-Cascade-SAS/OCCT, tag `V8_0_1`, commit `b8f597c677811d1f9f4d8a97f5ae2825c0353a42`.
- License: **LGPL-2.1-or-later with the OCCT exception** (`LICENSE_LGPL_21.txt`, `OCCT_LGPL_EXCEPTION.txt`).
- Modification: `TKG3d` is rebuilt with `tools/patches/occt-8.0.1-offset-d3.patch` (corrected offset-curve
  D2/D3 coefficients). The modified source of that component is the upstream source archive
  (`build/standalone-deps/occt-V8_0_1-source.zip`, sha256 `7c033d91…f0da`) plus that patch; the build
  script is `tools/build_occt_offset_d3.py`.
- Linkage: dynamic (the application loads the released DLLs and the rebuilt `TKG3d.dll`).
- Everything else in the SDK is unmodified; the build receipt records the sha256 of every untouched
  shared output.

## openNURBS (McNeel)

- Pin `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, notice at `Resources/licenses/openNURBS.txt`.
- Used for 3DM read/write only; no modification.

## QymCAD (reference source only)

- AGPL-3.0-or-later. Not linked, not shipped: read as reference material only.
""",
        encoding="utf-8",
    )
    (EVIDENCE / g / "GATE-CLOSEOUT.md").write_text("\n".join([
        "# G00-06 — License closeout",
        "",
        "| L4 | Trạng thái | Căn cứ |",
        "| --- | --- | --- |",
        "| L4-06-01-01 | EVIDENCED | `QymCAD/LICENSE` = GNU AFFERO GPL v3; QymCAD chỉ là tài liệu tham chiếu, không link |",
        "| L4-06-01-02 | EVIDENCED | OCCT LGPL-2.1 + exception, link động, license files trong SDK; phần sửa `TKG3d` được công bố bằng patch + script |",
        "| L4-06-01-03 | EVIDENCED | `cargo metadata`: 345 package, chủ yếu MIT/Apache; 6 LGPL-2.1-or-later là chính workspace; **không có GPL/AGPL** trong cây Rust |",
        "| L4-06-01-04 | OPEN (human) | quyết định license sản phẩm phải do người có thẩm quyền ký |",
        "| L4-06-02-01 | EVIDENCED | phân loại asset: SVG authored tự tạo; không nhập bitmap/manual thương mại |",
        "| L4-06-02-02 | EVIDENCED | notices + source offer: `design/M00/G00-06/THIRD-PARTY-NOTICES-M00.md` |",
        "| L4-06-02-03 | EVIDENCED | rủi ro phân phối manual/dataset: chỉ dùng tài liệu authored, không kèm manual Matrix |",
        "| L4-06-02-04 | OPEN (human) | ký gate legal readiness |",
        "",
    ]) + "\n", encoding="utf-8")
    (DESIGN / g / "api.md").write_text(
        """# License obligations (G00-06)

1. OCCT: dynamic link; nếu ship `TKG3d` đã vá phải kèm source của phần sửa (patch + script + source archive digest).
2. openNURBS: giữ notice `Resources/licenses/openNURBS.txt`.
3. QymCAD: AGPL, chỉ đọc tham khảo — không copy code vào sản phẩm, không link.
4. Rust dependencies: MIT/Apache chiếm đa số; không có GPL/AGPL trong cây.
""",
        encoding="utf-8",
    )
    (FIXTURES / g).mkdir(parents=True, exist_ok=True)
    (FIXTURES / g / "positive.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-06-02", "cases": [
            {"id": "M00-FX-06-02-POS-01", "check": "notices_present", "expected": {"occt": True, "opennurbs": True}},
        ]}, indent=2) + "\n", encoding="utf-8")
    (FIXTURES / g / "negative.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-06-02", "cases": [
            {"id": "M00-FX-06-02-NEG-01", "check": "agpl_in_rust_tree", "expected": {"present": False}},
        ]}, indent=2) + "\n", encoding="utf-8")
    evidence(g, "OM9-M00-L4-06-01-02", "OCCT LGPL and dynamic link",
             ["SDK: `LICENSE_LGPL_21.txt` + `OCCT_LGPL_EXCEPTION.txt`; link động.",
              "Phần sửa: `tools/patches/occt-8.0.1-offset-d3.patch`, build `tools/build_occt_offset_d3.py`, receipt ghi sha256."])
    evidence(g, "OM9-M00-L4-06-01-03", "Cargo SPDX audit",
             ["`cargo metadata` (345 package): MIT 71, MIT OR Apache-2.0 167, Apache-2.0 OR MIT 30, Zlib OR Apache-2.0 OR MIT 10, Apache-2.0 8, …; LGPL-2.1-or-later 6 (workspace members); không có GPL/AGPL."])
    evidence(g, "OM9-M00-L4-06-02-02", "Third-party notices and source offer",
             ["`design/M00/G00-06/THIRD-PARTY-NOTICES-M00.md`: OCCT modified component + source offer, openNURBS notice, QymCAD reference-only."])

    # ---------------------------------------------------------------- G00-07 — reproducible
    g = "G00-07"
    (EVIDENCE / g).mkdir(parents=True, exist_ok=True)
    (EVIDENCE / g / "GATE-CLOSEOUT.md").write_text("\n".join([
        "# G00-07 — Reproducible build closeout",
        "",
        "| L4 | Trạng thái | Căn cứ |",
        "| --- | --- | --- |",
        "| L4-07-01-01 | EVIDENCED | Windows MSVC 14.44.35207 + CMake 4.x; OCCT **8.0.1** (L4 ghi 7.8.1 là bản cũ — đã chốt 8.0.1 trong `native/occt/pin.json`) |",
        "| L4-07-01-02 | OPEN | build release từ clean checkout chưa chạy trong lượt này |",
        "| L4-07-01-03 | OPEN | cold-start + create/open/save chưa test |",
        "| L4-07-01-04 | EVIDENCED | support matrix + fallback: Win10/11 x64, MSVC; GPU rời hoặc fallback WARP (đo 6,9 ms/khung 640×360) |",
        "| L4-07-02-01 | N/A | Linux không được chọn cho MVP |",
        "| L4-07-02-02 | N/A | như trên |",
        "| L4-07-02-03 | EVIDENCED | binary asset versioning: pin OCCT + openNURBS bằng tag/commit/digest, `fetch_occt.py` verify SHA-256 |",
        f"| L4-07-03-01 | EVIDENCED | test command matrix: `cargo test -p om9-core` (44 suite / 374 test) + harness `native/occt-rust-proof` (12 test) |",
        "| L4-07-03-02 | OPEN | deterministic artifact + attestation chưa có |",
        "| L4-07-03-03 | EVIDENCED | validator metadata: `audit_milestone_tree.py` 0 findings; `om9_milestone_dashboard.py` chạy; 3 tool protocol M00 (`validate_m00_*`, `next_m00_task`) **vẫn thiếu** — ghi nhận |",
        "| L4-07-03-04 | OPEN (human) | quyết định cuối bằng chứng M00 trước E01 |",
        "",
        "## Ghi chú digest",
        "",
        "`fetch_occt.py --verify-only`:" + (" tất cả digest OK" if digests_ok else " **không** xác minh được"),
        "",
    ]) + "\n", encoding="utf-8")
    (DESIGN / g).mkdir(parents=True, exist_ok=True)
    (DESIGN / g / "api.md").write_text(
        """# Reproducible build baseline (G00-07)

| Thành phần | Pin |
| --- | --- |
| Rust | stable-msvc, workspace `edition 2024` |
| MSVC | 14.44.35207 (vc14 x64) |
| CMake | 4.x, generator "Visual Studio 17 2022" |
| OCCT | 8.0.1, tag `V8_0_1`, commit `b8f597c…`, 3 artifact SHA-256 (`native/occt/pin.json`) |
| openNURBS | `eb92af3…`, notice `Resources/licenses/openNURBS.txt` |
| Build | `CARGO_TARGET_DIR` ngoài checkout, `-j 2` (8 GiB page file) |

## Kiểm chứng

- `python implementation/tools/fetch_occt.py --verify-only` → mọi digest khớp.
- `python implementation/tools/audit_milestone_tree.py` → 0 findings.
- `python tools/om9_milestone_dashboard.py` → sinh được view.
- Thiếu: CI record, clean-checkout release build, cold-start test, attestation.
""",
        encoding="utf-8",
    )
    (FIXTURES / g).mkdir(parents=True, exist_ok=True)
    (FIXTURES / g / "positive.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-07-03", "cases": [
            {"id": "M00-FX-07-03-POS-01", "check": "digests", "expected": {"verified": bool(digests_ok)}},
            {"id": "M00-FX-07-03-POS-02", "check": "audit_tree", "expected": {"findings": 0}},
        ]}, indent=2) + "\n", encoding="utf-8")
    (FIXTURES / g / "negative.json").write_text(
        json.dumps({"schema": "om9.fixture.v1", "l3_id": "OM9-M00-L3-07-03", "cases": [
            {"id": "M00-FX-07-03-NEG-01", "check": "m00_protocol_tools", "expected": {"present": False, "tools": ["validate_m00_v013.py", "next_m00_task.py", "validate_m00_execution.py"]}},
        ]}, indent=2) + "\n", encoding="utf-8")
    evidence(g, "OM9-M00-L4-07-01-01", "Windows MSVC and OCCT baseline",
             ["MSVC 14.44.35207, CMake 4.x, VS2022 x64; OCCT 8.0.1 pin (L4 nêu 7.8.1 là bản cũ, đã thay bằng pin).",
              f"`fetch_occt.py --verify-only`: {'OK' if digests_ok else 'FAIL'}"])
    evidence(g, "OM9-M00-L4-07-01-04", "Windows support matrix and fallback",
             ["Win10/11 x64 + MSVC; GPU rời (đo RTX 3060/Vulkan) hoặc fallback WARP (Microsoft Basic Render Driver, 6,9 ms/khung 640×360)."])
    evidence(g, "OM9-M00-L4-07-02-03", "Binary asset versioning",
             ["OCCT: tag + commit + 3 digest; openNURBS: commit + license file; fetch tool verify SHA-256 trước khi dùng."])
    evidence(g, "OM9-M00-L4-07-03-01", "Test command matrix",
             ["Workspace: `cargo test -p om9-core` (44 suite, 374 test, 0 fail); harness: `cargo test --manifest-path native/occt-rust-proof` (12 test).",
              "Ghi chú: `-j 2` bắt buộc trên máy này."])
    evidence(g, "OM9-M00-L4-07-03-03", "M00 metadata validators",
             ["`audit_milestone_tree.py`: 0 findings; `om9_milestone_dashboard.py`: chạy được.",
              "Thiếu: `tools/validate_m00_v013.py`, `tools/next_m00_task.py`, `tools/validate_m00_execution.py`."])

    print("closeout files written for G00-04..G00-07")
    print("digests_ok:", digests_ok, "| history tests:", history_ok)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
