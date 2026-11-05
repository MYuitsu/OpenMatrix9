# Boolean options: OCCT, Blender and Rhino 5

2026-11-02, Windows x64, local OCCT 8.0.1 and one unoptimized debug binary (`om9-boolean-experiment.exe`, SHA-256 `99e1e9c58b2763b91413276655ddbdd937bb98b27560997ca3c625ece4d0359d`). `tools/dice_21_no_undo_benchmark.py` created a 20 mm box and subtracted 21 spheres using Boolean2Objects; Undo was disabled only for this opt-in benchmark. Each run was sequential; options were set with temporary per-process environment flags in the experiment binary. Flags were removed from product source after testing.

| OCCT option | Startup to save (s) | Last cavity (s) | Sum of 21 Difference preview MCP calls (s) |
| --- | ---: | ---: | ---: |
| serial | 40.080 | 3.680 | 14.727 |
| parallel | 48.246 | 4.867 | 15.952 |
| OBB | 38.310 | 3.077 | 12.469 |
| OBB repeat | 38.252 | 4.233 | 12.459 |
| serial repeat | 49.043 | 3.130 | 18.167 |

The serial repeat was affected by variable system load; these are not statistically controlled runs, release measurements, p95, desktop or normal-Undo results. The evidence supports evaluating OBB as a candidate, not a precise speedup or a claim that parallel is always slower. Raw timings: `2026-11-02-boolean-options-{serial,parallel,obb,obb2,serial2}.jsonl`. All five runs succeeded through 21 cavities and produced identical canonical BRep source SHA-256 `22ce14cab9bf89e8e5b5a45b806ce1196f8d44b2b16bec5645a0ba4f3783a23f` for this fixture. The experimental binary differs from the final build in its option-selection plumbing; OBB values and geometry operations used in the test match the final choice.

Final change: `SetUseOBB(true)` for the existing OCCT union/cut/common builder, including surface Boolean users of that bridge. No gluing, tolerance change, or global parallelism was enabled. The output snapshot still passes validity/budget checks. This is an OCCT option, not copied code. OBB can have different costs on other models, so matched release and broader workload measurements remain necessary. Parallel execution is deferred pending a thread-budget design and controlled measurements. Multiple tools remain a candidate for an explicit batch API, not a replacement for separate interactive Undo transactions.

[CadQuery](https://raw.githubusercontent.com/CadQuery/cadquery/master/cadquery/occ_impl/shapes.py) uses OCCT OBB and parallel processing in parts of its Boolean implementation. Local QymCAD also exposes parallel OCCT booleans. [Blender's Boolean implementation](https://github.com/blender/blender/blob/main/source/blender/geometry/intern/mesh_boolean.cc) operates on mesh representations, and its [manual](https://docs.blender.org/manual/en/dev/modeling/modifiers/generate/booleans.html) documents distinct solver constraints. These are not drop-in BRep implementations.

Bounded Rhino 5 reference inspection: the recovered RhinoCommon BRep wrapper assembles two BRep pointer collections and sends them with tolerance to a native intersection/difference bridge. The recovered `RhinoBooleanDifference` body builds lists and calls `TL_BooleanDifference`; this is a call site, not the solver implementation. Types and pseudocode are approximate and no disassembly or runtime benchmark was performed. The available evidence does not establish threading, broad-phase acceleration or Boolean performance; no recovered code is reproduced. Earlier Undo analysis is `2026-11-02-rhino5-undo-architecture-review.md`.

Validation: `cargo test -j 2 -p om9-kernel --features 3dm-brep --test l2_01_10`: 70 passed, including Boolean volumes, open/tampered operands, empty/disjoint results and surface trim. The app MCP suite had 3 passes; the focused immediate Boolean workflow passed; the debug app built. Public-source audit: zero errors. No independent print/manifold certification is claimed.
