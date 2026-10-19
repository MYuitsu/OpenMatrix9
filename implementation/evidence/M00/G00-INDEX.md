# Bảy gate của M00 — cập nhật 2026-10-17

Máy đọc: [`G00-INDEX.json`](G00-INDEX.json). Mỗi gate = một L2 của M00; gate chỉ đóng bằng **artefact thật**
trong `implementation/evidence/M00/<gate>/` + **một review record**, không bằng code sản phẩm (M00 không có code sản phẩm).

| Gate | L2 | Nội dung | Trạng thái | Bằng chứng |
| --- | --- | --- | --- | --- |
| `G00-01` | L2-01 | QymCAD GitHub & source reuse | **OPEN** (không đổi từ 2026-10-15) | `G00-01/STATUS-SUMMARY.md`, 2 bản audit trong `docs/validation/` |
| `G00-02` | L2-02 | OCCT bridge & numerical geometry | **PARTIAL** (L3-01 4/4, WIP) | `OM9-M00-L3-02-01/`, 3 design note trong `design/M00/OM9-L2-02/`, 2 JSON conformance |
| `G00-03` | L2-03 | UI/Viewport/GPU | **NOT_STARTED** | — |
| `G00-04` | L2-04 | 3DM / openNURBS | **NOT_STARTED** (đã có probe 8.0.1/openNURBS nhưng chưa đúng L4) | `M01/OM9-L2-01-05/NATIVE-TOOLCHAIN-PROOF.json` |
| `G00-05` | L2-05 | Associative history & stable topology | **NOT_STARTED** | — |
| `G00-06` | L2-06 | License / redistribution | **NOT_STARTED** (đã xác minh license files) | `LICENSE_LGPL_21.txt`, `OCCT_LGPL_EXCEPTION.txt`, `native/occt/pin.json` |
| `G00-07` | L2-07 | Reproducible build & CI | **NOT_STARTED** (đã có fetch tool + digest) | `implementation/tools/fetch_occt.py` |

## G00-01 — OPEN, không có tiến triển mới

- 12 L4: **2 BLOCKED_APPROVAL** (`L4-01-01-01` kiến trúc, `L4-01-03-03` license) + **10 BLOCKED_DEPENDENCY**.
- Pin kế hoạch và checkout QymCAD trên đĩa đều là `746a17f1…`; re-pin (3015 file, 19 642 tham chiếu) đã xong.
- **Vẫn thiếu** 3 công cụ mà protocol M00 yêu cầu: `tools/validate_m00_v013.py`, `tools/next_m00_task.py`,
  `tools/validate_m00_execution.py` (kiểm lại 2026-10-17: cả ba đều không tồn tại).
- Kết luận: gate này **chỉ đóng bằng người ký** hai L4 approval; không có việc kỹ thuật nào khác tự mở được.

## G00-02 — PARTIAL, đây là gate đang hoạt động

| L3 | Trạng thái | Bằng chứng |
| --- | --- | --- |
| `L3-01` Toolchain & C ABI | **4/4 L4**, dashboard 🟡 WIP (chờ ký) | `OM9-M00-L3-02-01/` (audit, compile log, probe JSON, Rust proof log), `source_crosswalk.json` |
| `L3-02` Lifetime/threading/error | chưa làm | — |
| `L3-03` Numeric invariants | chưa làm | — |

Sự thật đã đo:

- OCCT **8.0.1** pin đủ digest (release + 3rdparty + source archive), license files trong gói.
- Probe: topo 6F/24E/48V, mesh 24 node/12 tri, 2 giao điểm, boolean 7 face valid, volume `400.0` và
  `349.73451754256325` (sai ≤ 1e-13).
- **Lỗi độ chính xác**: offset curve D2 sai 41,7 % / D3 99,96 % (ellipse), 71,8 % / 100,1 % (spline knot không đều)
  trên bản stock; bản vá `TKG3d` còn ≤ 1,6e-05. Gate tự phát hiện (`occt_upgrade_gate.py`).
- Rust proof harness `native/occt-rust-proof`: 4 test, box 400, drilled giải tích, input sai →
  `Standard_ConstructionError`.
- Đạo hàm offset do Rust sở hữu (`crates/om9-core/src/offset_curve.rs`, 12 test) → patch là backstop.

Đóng gate khi: L3-02 + L3-03 có bằng chứng, và L3-01..03 có review record.

## G00-03 / G00-05 / G00-07 — NOT_STARTED

Chưa có artefact nào dưới `implementation/evidence/M00/G00-0X/`. Việc cần làm đúng thứ tự: G00-02 xong trước
(gate đang mở), rồi G00-03 (UI/GPU), G00-04 (3DM), G00-06 (license), G00-07 (reproducible).

## G00-04 — có probe nhưng chưa đạt L4

`native/brep_probe.cpp` + bằng chứng `NATIVE-TOOLCHAIN-PROOF.json` đã chứng minh openNURBS pin
`eb92af3…` build và chạy được (topo, NURBS eval, transform), **và** đo được giới hạn: không có B-Rep mesher,
không có curve/surface intersector. Gate vẫn mở vì thiếu artefact riêng của 12 L4 (fixture round-trip 3DM,
version policy, exchange contract).

## G00-06 — license đã xác minh một phần, chờ người ký

- OCCT 8.0.1: `LICENSE_LGPL_21.txt` + `OCCT_LGPL_EXCEPTION.txt`; dự án link động.
- **Nghĩa vụ mới**: `TKG3d` là thành phần OCCT **đã sửa** → phải công bố source phần sửa:
  `tools/patches/occt-8.0.1-offset-d3.patch` + `tools/build_occt_offset_d3.py` (đã có).
- Còn thiếu: mục trong `THIRD-PARTY-NOTICES` nói rõ "TKG3d is a modified OCCT component", và chữ ký của
  người review cho L4 license.

## Việc tiếp theo (đúng thứ tự)

1. `G00-02` → `L3-02` lifetime/threading (probe dưới `native/`), rồi `L3-03` numeric invariants.
2. Xin chữ ký cho `L3-01` (dashboard chuyển 🟡 → ✅).
3. Sau khi G00-02 đóng: `G00-03` (UI/GPU feasibility), rồi `G00-04` (3DM), `G00-06`, `G00-07`.
4. `G00-01` chờ quyết định của người: ký 2 L4 approval hoặc bổ sung 3 công cụ protocol còn thiếu.
