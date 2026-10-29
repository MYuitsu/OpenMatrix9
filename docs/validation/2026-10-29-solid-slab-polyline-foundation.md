# Slab open-polyline miter foundation — 2026-10-29

The safe Rust Slab foundation now computes a planar open-polyline strip with mitered corners and a signed height extrusion vector. It refuses degenerate segments, reversed or excessive miters, nonplanar inputs, self-intersecting/touching boundaries and over-budget allocations. A 1024-point ceiling bounds the quadratic intersection check. This is a geometry service only: Slab remains disabled until source selection, options, preview, transaction and Undo are integrated and verified.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 2 passed. The new L-shaped fixture checks actual capped-BRep volumes for both offset sides and tests both-sided height, reversed/nonplanar/self-intersecting inputs and memory limit; the earlier straight-line fixture also passed.
- `python tools/public_source_audit.py`: 14729 source files, 594 icon bindings, 593 SVGs, no errors.

Miter and intersection checks are conservative and use fixed tolerances; they may reject valid near-touching or collinear configurations. Closed curves, smooth curves, more varied corner geometry, CPlane options, real interaction, Matrix-original behavior and the complete command remain open. This supplements `docs/validation/2026-10-29-solid-slab-line-foundation.md`.
