# Dice MCP performance investigation — 2026-11-02

Windows x64, `L:\om9-target\debug\om9.exe`, default dev profile, `cargo build -p om9-app -j 2`. The probe creates a 20 mm box and sequentially subtracts four radius-1.5 mm spheres, 4 mm apart, via MCP `Boolean2Objects`, with `DeleteInput=Yes`. Each timing includes the MCP call and its state response. The baseline and modified measurements each comprise one run; they are not statistically representative.

| Operation by cavity | Before (s) | Buffered fingerprint (s) |
|---|---:|---:|
| Sphere commit 1 / 2 / 3 / 4 | 0.097 / 0.405 / 1.352 / 1.692 | 0.080 / 0.319 / 0.735 / 1.337 |
| Select 1 / 2 / 3 / 4 | 0.029 / 0.124 / 0.443 / 0.578 | 0.024 / 0.100 / 0.237 / 0.437 |
| Difference preview 1 / 2 / 3 / 4 | 0.319 / 0.541 / 0.830 / 1.237 | 0.301 / 0.480 / 0.733 / 1.027 |
| Commit 1 / 2 / 3 / 4 | 0.358 / 1.106 / 2.419 / 3.597 | 0.282 / 0.831 / 1.704 / 2.884 |

`om9_state` calls took approximately 0.3–0.6 ms. They do not explain the measured delay in this small fixture. Selection itself charges retained usage: `TransactionCoordinator::validate_retained_owners` validates each distinct retained document, including Undo/Redo. `Brep::validate` verifies the source digest and JSON display fingerprint. `retained::fingerprint` previously passed many small JSON writes directly to SHA-256; it now buffers 4096 bytes on the stack. Serialization bytes, digest algorithm, validation, and budget checks are unchanged. The measured improvement is consistent with lower hashing overhead, but this is not a sampled CPU profile and does not quantify the share of OCCT, retained validation, or scene publication independently. History growth still makes the workflow progressively slower; full 21-cavity dice, release build, desktop mouse and GPU present were not measured after the change.

The first exploratory fixture used sphere centers 3 mm apart, tangent at radius 1.5 mm; the second subtraction failed with `invalid BRep source shape`. The successful comparison uses 4 mm separation. This observation is not a general diagnosis of tangent Boolean behavior.

Validation: `cargo test -p om9-core -j 2 --test retained_imports --test retained_entity_baselines --test fingerprint_stream` 31 passed; `cargo test -p om9-app -j 2 --test solid_workflows immediate_boolean_menu_actions_have_measured_results` 1 passed (113 filtered); `cargo test -p om9-app -j 2 --test mcp_stdio` 2 passed; default app debug build passed; public source audit 0 errors. `cargo test -p om9-kernel -j 2 --test l2_01_10 m03_01_10_001` failed to link, missing `om9_occt_brep_to_3dm` and `om9_occt_brep_from_3dm_snapshot`; no kernel test result is claimed. Source fixture: `tools/dice_performance_probe.py`.

Next: profile retained validation and scene publication on a saved, representative multi-cavity fixture in release mode; preserve tamper detection and Undo budget behavior when considering cached validation. Investigate the tangent-input failure separately.
