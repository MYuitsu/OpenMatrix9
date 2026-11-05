# Top11 Trim: area of a nonuniformly placed circular cut

A BRep rectangle and circle sharing placement `diag(2, 1, 1)` plus translation previously failed Trim: the splitter's non-adaptive OCCT surface integral reported 400.007338 instead of the original area 400 mm². The split guard now uses OCCT adaptive surface integration with relative target `1e-10`; the shared BRep area measurement uses the same adaptive integration rather than the non-adaptive default.

The controller regression removes the ellipse-shaped world region, checks the local retained area against `200 − 4π` mm², checks distances at the hole center and on the retained face, checks placement and Undo. A translated circle fixture also passes. The target is planar; this does not establish all affine, curved or periodic cases or UI interaction. Adaptive integration can cost more than the previous default; no performance benchmark was run.

Windows MSVC with pinned OCCT 8.0.1 SDK/runtime and `CARGO_TARGET_DIR=L:\om9-target`: OCCT editing and transform suites passed 77 tests; Top11/options/BRep and extended measurement suites passed 93 tests; the final strengthened affine fixture passed separately. Public source audit reported zero errors. Native changes remain synchronous OCCT adapters; Rust owns document, state and transaction.

SHA-256: `tests/top11_brep.rs` 72e68b5ec34a88bc1d706a5fad360dde58b44485e2237a9d8de888c784e4cfce; `measurement_bridge.cpp` 886ab32634d4d1c1c7ef7c47efdb79baec061e9fa454f0b72a59d19fcb6ea514; `bridge.cpp` 012d27a7e6601ae310dd40f984837ce7b81d95084376e96bb0f0387ef6ef6f3b.

Selected curve cutters are still not trimmable in the surface branch, and multi-surface mutual Trim, general geometry, file round trips, performance and desktop acceptance remain open.
