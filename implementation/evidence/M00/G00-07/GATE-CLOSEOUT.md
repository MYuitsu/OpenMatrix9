# G00-07 — Reproducible build closeout

| L4 | Trạng thái | Căn cứ |
| --- | --- | --- |
| L4-07-01-01 | EVIDENCED | Windows MSVC 14.44.35207 + CMake 4.x; OCCT **8.0.1** (L4 ghi 7.8.1 là bản cũ — đã chốt 8.0.1 trong `native/occt/pin.json`) |
| L4-07-01-02 | OPEN | build release từ clean checkout chưa chạy trong lượt này |
| L4-07-01-03 | OPEN | cold-start + create/open/save chưa test |
| L4-07-01-04 | EVIDENCED | support matrix + fallback: Win10/11 x64, MSVC; GPU rời hoặc fallback WARP (đo 6,9 ms/khung 640×360) |
| L4-07-02-01 | N/A | Linux không được chọn cho MVP |
| L4-07-02-02 | N/A | như trên |
| L4-07-02-03 | EVIDENCED | binary asset versioning: pin OCCT + openNURBS bằng tag/commit/digest, `fetch_occt.py` verify SHA-256 |
| L4-07-03-01 | EVIDENCED | test command matrix: `cargo test -p om9-core` (44 suite / 374 test) + harness `native/occt-rust-proof` (12 test) |
| L4-07-03-02 | OPEN | deterministic artifact + attestation chưa có |
| L4-07-03-03 | EVIDENCED | 5 tool protocol M00 đã có: `validate_m00_v013.py` PASS (7 L2/19 L3/73 L4, DAG + schema), `next_m00_task.py` (exit 2 = task kế bị chặn), `validate_m00_execution.py` (12 record, 0 invalid; `--self-test` 6/6), `m00_capture.py`, `m00_init_evidence.py`; kèm `audit_milestone_tree.py` 0 findings |
| L4-07-03-04 | OPEN (human) | quyết định cuối bằng chứng M00 trước E01 |

## Ghi chú digest

`fetch_occt.py --verify-only`: tất cả digest OK

