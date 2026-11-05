# Undo B-Rep inventory optimization and 21-cavity comparison

M01 `OM9-L3-01-02-007`, standalone Windows x64, default debug profile, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`. The user's 20 mm cube / 21 hemispherical cuts fixture was run through the MCP command controller, identical face order and Boolean2Objects Difference with DeleteInput=Yes. Timing runs from before process startup through completion of `.om9` save. Both variants are single runs from separate builds; they do not establish release or desktop latency.

| Metric | Earlier Undo enabled | Optimized Undo enabled | Undo disabled reference |
|---|---:|---:|---:|
| Start to save | 1212.506 s | 239.924 s | 120.182 s |
| Hõm 21 | 159.848 s | 27.468 s | 10.943 s |
| Save | 2.497 s | 1.830 s | 2.594 s |
| Undo entries at save | 43 | 43 | 0 |

The optimized run is 5.05× as fast as the earlier enabled-Undo run (80.21% less elapsed time). Its six face totals, in the same order 1/6/2/5/3/4, were 0.412/13.478/11.608/57.962/57.850/95.922 s. The exact baseline, optimized and disabled-mode timing receipts are retained in `docs/validation/`.

## Change and safety

`Document::storage_usage_internal` previously called complete `Geometry::validate` for each B-Rep even during nonallocating inventory. Repeated admission passes over retained Undo owners consequently rehashed and validated B-Rep data many times. A Rust-only B-Rep owned-buffer inventory now reports the same `Usage` without hashing or allocating; document validation still executes `Brep::validate` before acceptance. The inventory includes the `Geometry` enum itself and uses checked topology count arithmetic. Other geometry types retain their previous paths. No Rhino or Blender code was copied, no Undo behavior was changed, and no cache of mutable geometry or relaxed memory limits was introduced.

The full run checked the expected increasing Undo depth after each cavity, saved one B-Rep with 43 Undo entries, performed Undo then Redo after timing, and verified 42/1 then 43/0 Undo/Redo depths. The after-Redo object ID matched the saved object. Baseline, optimized and after-Redo native B-Rep source SHA-256 all match `22ce14cab9bf89e8e5b5a45b806ce1196f8d44b2b16bec5645a0ba4f3783a23f`; display-mesh volume is 7852.34279186927 mm³ in each, compared with analytical 7851.559747 mm³. Display-mesh bounds extend to 20.0803 and 20.0703 mm on x/y; this is tessellation evidence, not a claim of exact geometric bounds. This test does not independently certify a manifold or print-ready STL.

An initial incorrect omission of `Geometry` enum storage was caught by the existing debug equality assertion; the corrected implementation includes it. Tests assert exact BRep inventory/validation usage, that validation scratch remains budgeted, tampered B-Rep rejection without losing prior Redo, normal Undo/Redo, and caller scopes. `cargo test -p om9-core -j 2`: 994 passed, 2 ignored; `om9-app --test mcp_stdio --test solid_workflows`: 116 passed, 1 ignored; `om9-app --test stl_export`: 10 passed. The full 21-cavity fixture is `tools/dice_21_undo_optimized_benchmark.py`; its output is `build/dice-21-undo-optimized-2026-11-02/dice.om9`. Binary SHA-256 used for measurement: `8ee341dd4e1583d0484bba499012f35cd26c2a8cbe49e4467bccbe4ff148afce`.

Scope remains limited: one debug run per condition, different binary builds, not physical mouse or release timing. Deep validation still traverses retained documents and is a candidate for future immutable-owner accounting/validation work; no speedup equal to Rhino or Blender is claimed.
