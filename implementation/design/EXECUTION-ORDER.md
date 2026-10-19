# Thứ tự thực thi — đi theo milestone để dễ theo dõi

Ngày: 2026-10-17 · Nguồn số liệu: `MILESTONE_DASHBOARD.md` (sinh từ cây + bằng chứng) và `docs/openmatrix9-progress.json`.

## 1. Quy tắc thứ tự (không nhảy cóc)

1. **Theo milestone**: M00 → M01 → M02 → M03 → M04 → M05 → M06 → M07 → M08 → M09 → AFTER.
   `depends_on_milestones` trong cây là ràng buộc thật: M01←M00; M02←M00,M01; M03←M01; M04←M01; M05←M03; M06←M03,M05; M07←M03; M08←M03,M05,M06; M09←M00..M08; AFTER←M09.
2. **Trong một milestone**: theo thứ tự L2 (`L2-01`, `L2-02`, …), trong L2 theo `L3-001`, `L3-002`, …, trong L3 theo `L4-01` → `L4-05`.
3. **Một L3 chỉ được đánh dấu ✅ khi** có bằng chứng nghiệm thu của nó (per-L4 evidence + `ui-review.json` với `reviewer.status` hợp lệ). Code xong nhưng thiếu bằng chứng = 🟡.
4. **Gate chặn**: M00 là gate (M00 ghi rõ "stop and ask"); không dùng OCCT trong M01 khi M00-L2-02 chưa đạt.

## 2. Vị trí hiện tại\n\n> M00 chỉ có **probe/harness** dưới `native/`; mọi crate sản phẩm bắt đầu từ M01.

| | |
| --- | --- |
| Milestone đang làm | **M00** — Feasibility, QymCAD source, OCCT, legal |
| L2 đang làm | **OM9-L2-02 — OCCT** |
| L3 đang làm | **OM9-L3-02-01 — Toolchain and C ABI bridge proof** |
| L4 đang làm | **OM9-M00-L4-02-01-04** — ✅ xong bằng harness `native/occt-rust-proof` (4 test + binary JSON); **M00 không có code sản phẩm** |
| Đã xong trong L3 này | L4-01 audit `build.rs`/ABI (✅), L4-02 compile reference bridge (✅ chạy thật, kết quả âm tính), L4-03 C ABI ownership + ngoại lệ (✅ probe 8.0.1), L4-04 proof từ Rust (✅ 4 test) |
| Trạng thái dashboard | **4/4 L4 có bằng chứng, WIP** — chưa ký review (M00 có human approval protocol; không tự ký) |
| Việc kế tiếp | **Ký review cho L3-01..03** để đóng G00-02, rồi sang G00-03 (UI/GPU feasibility) |

## 3. Bảng milestone (số liệu dashboard 2026-10-17)

| M | L2 | L3 | ✅ | 🟡 | ⬜ | Việc kế tiếp trong M |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| M00 | 7 | 19 | 0 | 0 | 19 | L2-02: L4-04 → L3-02 (lifetime/threading) → L3-03 (numeric invariants) |
| M01 | 13 | 117 | 38 | 5 | 74 | L2-01-05 (7 L3 chờ kernel bridge); L2-01-06 GPU (8 L3); L2-01-08 ✅7/7 |
| M02 | 9 | 59 | 0 | 0 | 59 | chờ M01 xong; gumball kernel đã port nhưng chưa có nghiệm thu L3 |
| M03 | 14 | 243 | 0 | 0 | 243 | L2-01-09/010/011 (OCCT geometry/boolean/ops) — dùng đúng pin 8.0.1 |
| M04 | 7 | 51 | 0 | 0 | 51 | history/builder; W7 kernel đã port, chưa nghiệm thu |
| M05 | 5 | 43 | 0 | 0 | 43 | gem/layout |
| M06 | 2 | 19 | 0 | 0 | 19 | settings/prongs |
| M07 | 3 | 19 | 0 | 0 | 19 | cutters |
| M08 | 8 | 51 | 0 | 0 | 51 | advanced jewelry modeling |
| M09 | 0 | 0 | 0 | 0 | 0 | 30 L4 release gate (chưa tới lượt) |
| AFTER | 34 | 170 | 0 | 0 | 170 | mở rộng sau release |

Tổng: **791 L3** — 38 ✅ / 5 🟡 / 748 ⬜.

## 4. Chuỗi kế tiếp (cụ thể, theo đúng thứ tự)

| # | Việc | Milestone/L3 | Mở khoá |
| --- | --- | --- | --- |
| 1 | ✅ harness `native/occt-rust-proof` + test Rust gọi OCCT (box 400, drilled 349.7345…) | M00-L2-02-L3-01-L4-04 | L3-01 đủ bằng chứng (chờ ký) |
| 2 | lifetime/threading/error safety: 10k shape, một call/lúc, RSS | M00-L2-02-L3-02 | đóng L3-02 |
| 3 | numeric invariants: tolerance sweep + `BRepCheck` + volume/area giải tích | M00-L2-02-L3-03 | đóng L3-03 → M00-L2-02 xong |
| 4 | 7 L3 của B-Rep bridge (handle, topology, transform, fixture, intersection, tessellation, registry) — **tạo `crates/om9-kernel` từ harness** | M01-L2-01-05 | M01 tiến |
| 5 | GPU viewport 9 L3 (L3-003 lấy mesh từ OCCT) | M01-L2-01-06 | M01 |
| 6 | 3DM read/write + exchange slices | M01-L2-01-12 / M03 | — |

## 5. Cách theo dõi

- `MILESTONE_DASHBOARD.md`: trạng thái từng L3 theo bằng chứng (`python tools/om9_milestone_dashboard.py`).
- `docs/openmatrix9-progress.json`: nhật ký từng phiên (id, bằng chứng, điều chưa chắc, việc kế tiếp).
- `implementation/design/M00/OM9-L2-02/*`: hồ sơ gate OCCT (proof, upgrade gate, offset curve Rust).
- Bất biến: khi nâng OCCT → chạy `implementation/tools/occt_upgrade_gate.py` trước khi làm bất cứ việc gì khác.
