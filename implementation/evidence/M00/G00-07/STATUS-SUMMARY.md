# G00-07 — Reproducible build, CI and release baseline

Trạng thái: **IN_PROGRESS** · sinh bởi `implementation/tools/update_m00_gates.py` lúc 2026-10-17T19:56:28Z
L3 đã ký: 0/3 · L4 có bằng chứng nghiệm thu: 5/11 · L4 có artefact: 5/11

## L3

| L3 | Trạng thái | L4 | Ký bởi |
| --- | --- | ---: | --- |
| `OM9-M00-L3-07-01` Windows 10/11 x64 baseline | WIP | 3 | — |
| `OM9-M00-L3-07-02` Linux optional & packaged OCCT | WIP | 3 | — |
| `OM9-M00-L3-07-03` Continuous verification and gate handoff | WIP | 3 | — |

## Sự thật đã kiểm

- toolchain: MSVC 14.44.35207, CMake 4.x, VS2022 x64; OCCT pin 8.0.1 (the L4 text still says 7.8.1, superseded by the pin)
- fetch_occt.py --verify-only confirms every upstream digest; audit_milestone_tree.py reports 0 findings
- test matrix: cargo test -p om9-core (44 suites, 374 tests) and the native harness (12 tests) pass with -j 2
- fallback renderer measured: Microsoft Basic Render Driver (WARP) renders at 6.9 ms per 640x360 frame
- the five M00 protocol tools exist and were run: validate_m00_v013 (PASS), next_m00_task (blocked on the approval L4), validate_m00_execution (12 records, 0 invalid; self-test 6/6), m00_capture and m00_init_evidence

## Còn chặn

- L4-07-01-02 clean-checkout release build is not recorded
- L4-07-01-03 cold start plus create/open/save is not tested
- L4-07-03-02 deterministic artifacts and attestations are not built
- L4-07-02-01/02 are N/A because Linux was not chosen; L4-07-03-04 needs the owner

## Việc tiếp theo

- record the release build from a clean checkout and the cold-start test, then the owner decides the final M00 evidence
