# STL f64 input and binary quantization guard

Follow-up: [document-derived deflection](2026-11-03-stl-document-deflection.md) changes the B-rep meshing request; the verification below describes the earlier quantization batch.

P3a partial implementation. Source and placed coordinates now remain `f64` through per-source weld and the STL writer. `TriangleSoup::new` retains its f32-input constructor for existing callers; `new_f64` accepts precise input, and stored positions are now f64. The new writer accepts an optional Euclidean quantization budget in exported units; the existing writer delegates without a numeric budget but still rejects non-finite output and triangles that collapse or reverse after f32 conversion. Encoded normals are computed from the encoded vertices.

The document STL route sets the per-vertex quantization budget to one tenth of the document's absolute tolerance in millimetres. A failure is returned before destination replacement. This is a quantization budget only: weld error, native conversion, surface approximation and transform roundoff are not yet combined into a proved total error bound. The export meshing profile still follows display deflection; no overall tolerance or printability claim follows.

Windows default features, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: `cargo test -p om9-exchange -j 2 --test stl_export_policy` passed 7 tests; `cargo test -p om9-app -j 2 --test stl_export` passed 14. Fixtures include a representable f64 triangle, a far-coordinate vertex exceeding the budget, a triangle that collapses after conversion, and a failed document export that leaves existing destination bytes unchanged. No general orientation reversal fixture, nonuniform scaling error budget, curved-surface deviation, desktop, or release performance measurement was run.

Remaining: quantized integer weld keys can merge inappropriate nearby coordinates and need range/actual-distance checks; welded degenerates are currently filtered; topology/self-intersection and memory budgeting remain incomplete. The optional tolerance is in output units, so non-unit scale callers must supply a correspondingly scaled budget. Continue the independent profile, geometric error/coverage and background export work in `implementation/design/M01/viewport-export-separation-20261102/PLAN.md`.

## Three-axis quantization at large coordinates (2026-11-04)

An authored open-triangle document fixture near 100,000 mm puts each coordinate approximately 0.003 mm from its encoded `f32` value. Every component error is below 0.004 mm, but the Euclidean error is between 0.004 and 0.006 mm. With a 0.04 mm export tolerance, the one-tenth quantization allowance is 0.004 mm and the export is rejected while existing destination bytes and the document remain unchanged. With a 0.06 mm tolerance, export succeeds; the test independently reads all encoded vertices and checks the three-axis error. This verifies the quantization rule and atomic destination on one authored fixture; it is not a source-meshing or total deviation bound.

On E: with `-j 2`, all 54 app STL export integration tests passed. No release or desktop measurement was performed.
