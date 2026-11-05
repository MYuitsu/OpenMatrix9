# Top11 Trim: exact face-region kernel groundwork

`trim_face_by_polyline` uses the existing OCCT face splitter and measures the exact distance from the pick to each resulting face. It rejects picks within tolerance of multiple pieces or outside every piece, then removes the uniquely picked face. Rust owns validation and composition; existing bounded OCCT adapters perform geometry and never own the document. Inputs and source remain unchanged on failure.

With a 10 × 20 mm planar face cut at x=3, picking either side leaves a valid one-face BRep with measured area 140 or 60 mm². Tests reject a pick on the cut, off the surface or outside the face, invalid tolerances and a non-finite pick, and check source preservation.

Windows, `CARGO_TARGET_DIR=L:\om9-target`: `cargo test -j 2 -p om9-kernel --features 3dm-brep --test l2_01_10` passed 63 tests. `python tools/public_source_audit.py` reported zero errors. Source SHA-256: `editing.rs` 9db6dbac1a37b3ecdb8730a752aa04b69c73fd38b17b482b8361e26096cacc79; `l2_01_10.rs` c179d6cae3ea7e6b562076a896546cfdf194ede3fe05dab6cb93d174d4f3dee9.

At the time of this kernel-only record the helper was not connected to Top11; the subsequent limited command integration is recorded in `2026-10-30-top11-trim-line-face.md`. The kernel fixture tests a straight cutter and a single planar face. General exact curve/surface and surface/surface intersections, multiple cutters, repeated command edits, viewport interaction and desktop acceptance remain open.
