# Reproducible build baseline (G00-07)

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
