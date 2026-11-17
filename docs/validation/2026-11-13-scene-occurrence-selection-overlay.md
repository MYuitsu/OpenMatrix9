# Scene occurrence selection overlay

2026-11-13. The B-rep subselection overlay now asks the selected scene occurrence for its published pick display, after matching the selection source stamp and resolving the occurrence through the scene. Previously the root OCCT case consulted the published root display but nested instances read the document B-rep directly; NativeBrep temporary source views fell through without drawing a B-rep edge highlight.

Headless egui fixtures checked an OCCT temporary DupEdge edge, scene replacement and cancellation; a NativeBrep box through the real background DupEdge source-view request, edge highlight and cancel; and a translated B-rep instance edge. The change does not establish desktop/GPU rendering, arbitrary instance remesh or all native B-rep input types.

With OCCT runtime on PATH, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: app `--lib dupedge_overlay` passed 1; `--lib temporary_edge_overlay` passed 1; `--lib instance_brep_overlay` passed 1. `cargo test -p om9-scene -j 2 --lib` passed 47. `cargo check -p om9-app -j 2 --no-default-features` passed with 21 warnings. Public source audit: 16,000 source files, 594 bindings, 593 SVGs, no errors; artifact pruning removed 0 and retained 169 cited older runs.

Scene's ordinary occurrence implementation still borrows display from the document when no independent representation exists. Document/Undo source-only ownership, the general representation store, cold restore and v5 remain incomplete.

SHA-256 `crates/om9-app/src/ui.rs`: `2fd6b2ef1b667864824b0acfc5dd39ef0aa0f8a6a2c905dc346563137639865a`.
