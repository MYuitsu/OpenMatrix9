# M00 — nguồn sự thật (source of truth)

Ngày: 2026-10-17 · Áp dụng cho toàn bộ 7 gate `G00-01..G00-07`

## 1. Vấn đề đã sửa

Trước đây trạng thái M00 được viết tay ở **bốn nơi**: `L1.gate_status`, `G00-INDEX.json`,
`G00-0X/STATUS-SUMMARY.md` và suy luận riêng trong đầu người viết. Bốn bản đó lệch nhau (ví dụ G00-01 có
22 artefact nhưng bị ghi "chưa có gì"; G00-02 đóng nhưng registry vẫn ghi PARTIAL).

Từ nay chỉ còn **một** đường tính và **ba** loại dữ liệu tách bạch.

## 2. Bốn loại, một chiều

| Loại | Nơi lưu | Ai ghi | Có được ghi tay? |
| --- | --- | --- | --- |
| **Spec** | `OM9_Milestones/M00/**` (`L1/L2/L3/L4.json`) | người, khi đổi kế hoạch | có — nhưng **không** chứa trạng thái thực thi (chỉ `planning_state`) |
| **Computed** | không lưu; tính mỗi lần chạy | `implementation/tools/update_m00_gates.py` | không |
| **Curated** | `implementation/evidence/M00/gate-notes.json` | người | có — chỉ sự thật đã kiểm, blocker, việc tiếp theo |
| **Derived** | `G00-INDEX.json`, `G00-0X/STATUS-SUMMARY.md`, `L1.gate_status`, `MILESTONE_DASHBOARD.md` | máy sinh | **không bao giờ** |

Chiều chảy: `spec + evidence + review records + curated notes → computed → derived`. Không có chiều ngược.

## 3. Computed lấy từ đâu

`update_m00_gates.py` **import chính module dashboard** (`tools/om9_milestone_dashboard.py`) và dùng
`build()`, `evidence_files`, `l4_evidence_count`, `signed_reviewer`. Vì vậy registry và dashboard **không thể
lệch nhau**: cùng một engine.

Mỗi gate tính:

| Trường | Nguồn |
| --- | --- |
| `l3_total` | `children_l3` của `L2.json` |
| `l3_signed` | số L3 có `*ui-review*.json` với `reviewer.status` hợp lệ (SIGNED, hoặc AUTOMATED_EVIDENCE_ONLY + AUTO_ONLY) |
| `l4_evidenced` | theo đúng quy tắc artefact của dashboard (`source_crosswalk.json`, `api.md` + fixtures, file tiền tố L4) |
| `l4_artefacts` | số L4 có **bất kỳ** file nào nhắc id (kể cả artefact kiểu cũ `L4-01-01-01.evidence.json`) |
| `status` | `CLOSED` nếu mọi L3 đã ký; `PARTIAL` nếu có L3 ký nhưng chưa hết; `IN_PROGRESS` nếu có bằng chứng; còn lại `NOT_STARTED` |

`l4_artefacts` tách khỏi `l4_evidenced` là có chủ ý: G00-01 có **12 artefact** nhưng **0 bằng chứng nghiệm
thu** (tất cả là bản ghi `BLOCKED_*` từ audit 2026-10-15), nên trạng thái đúng là `NOT_STARTED` — nếu gộp
hai số này thì gate sẽ bị báo "đã có bằng chứng" một cách sai.

## 4. Quy trình cập nhật (một lệnh)

```powershell
# 1. ghi bằng chứng vào implementation/evidence/M00/OM9-M00-L3-.../ (tên file theo L4)
# 2. ký review khi cần: OM9-M00-L3-....ui-review.json  (reviewer.status + sha256 từng file)
# 3. nếu có sự thật mới cần người ghi: sửa gate-notes.json
python implementation/tools/update_m00_gates.py --write   # sinh registry + summary + L1.gate_status
python tools/om9_milestone_dashboard.py                   # view dẫn xuất
python implementation/tools/audit_milestone_tree.py       # kiểm cấu trúc cây
```

Không được sửa tay: `G00-INDEX.json`, `G00-0X/STATUS-SUMMARY.md`, `L1.gate_status`,
`MILESTONE_DASHBOARD.md`. Chúng sẽ bị ghi đè ở lần chạy sau.

## 5. Bất biến (kiểm được bằng máy)

1. `G00-INDEX.json.totals.gates == 7` và mỗi `gate_id` trong cây có đúng một entry.
2. `status == CLOSED` ⟺ `l3_signed == l3_total`.
3. `l4_evidenced <= l4_artefacts <= l4_total`.
4. `L1.gate_status[gate]` khớp từng trường với registry.
5. Dashboard chỉ đổi khi evidence/review đổi: chạy lại hai lần liên tiếp phải cho cùng file (trừ `as_of_utc`).

## 6. Trạng thái hiện tại (sinh lúc 2026-10-17T17:06:09Z)

| Gate | Trạng thái | L3 ký | L4 bằng chứng | L4 artefact |
| --- | --- | ---: | ---: | ---: |
| G00-01 QymCAD | NOT_STARTED | 0/3 | 0/12 | 12 |
| G00-02 OCCT | **CLOSED** | 3/3 | 12/12 | 12 |
| G00-03 UI/GPU | IN_PROGRESS | 0/3 | 6/10 | 6 |
| G00-04 3DM | NOT_STARTED | 0/3 | 0/12 | 0 |
| G00-05 Associative | NOT_STARTED | 0/2 | 0/8 | 0 |
| G00-06 License | NOT_STARTED | 0/2 | 0/8 | 0 |
| G00-07 Reproducible | NOT_STARTED | 0/3 | 0/11 | 0 |

Tổng: 7 gate, **1 đóng**, 24/73 L4 có artefact, 18/73 có bằng chứng nghiệm thu.

## 7. Ngoại lệ có chủ ý

- `G00-01` và `G00-06` có L4 cần **người ký** (`human_approval_required` trong gate-notes). Máy không bao
  giờ tự đặt `SIGNED`.
- `native/` chứa probe/harness của gate (`occt_probe.cpp`, `occt_conformance.cpp`, `occt-rust-proof/`,
  `gpu-probe/`). M00 **không** có crate sản phẩm; `crates/om9-kernel` thuộc M01-L2-01-05.
