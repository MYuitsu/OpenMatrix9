# Top 11 Explode BRep — 2026-10-28

The M03 normalized reference specifies that Explode separates a polysurface into surfaces. The bounded recovered Top 11 menu listing contains `Explode`, `S Explode`, and `TopIconExplode` together; this is evidence of a menu association, not a geometry solver or proof of the dispatched script. OM9 uses the existing capability-scoped OCCT `creation::measure` and `editing::extract_face` adapters; Rust owns selection, new entities, placement and the single transaction. No native bridge was changed.

With the OCCT feature, the Top 11 button is now available for BRep or mixed curve/BRep selection. The operation extracts each exact BRep face and publishes all pieces in one transaction. A single-face BRep is rejected before mutation. Other unsupported kinds remain unavailable. Face outputs preserve the source placement; a test box produces six single-face BReps with total area 52 mm² and one Undo restores the source.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: before the change the new BRep regression failed with `Unsupported("explode needs curves only")`; afterward `cargo test -p om9-app -j 2 --test top11_brep --test top11 --test top11_options` passed 8 + 19 + 4 tests. `rustfmt --edition 2024 --check` passed for the edited Top 11 and BRep-test files; `tools/public_source_audit.py` passed. Pruning removed no runs.

Fingerprints: `crates/om9-app/src/session/top11.rs` SHA256 0de77aa2766299d4e2edb6b9cde6e8ea08925093005fa7c2264d512f1976d2cd; `crates/om9-app/tests/top11_brep.rs` SHA256 188d43101fc0b5cdd486ab6c8868392ec45d181c1c2a0831b974fc713dc65b7e.

Limits: no manual UI interaction or human M03-UI signoff; native face extraction is tested on a box, not all topology. Mesh, text, dimension, group and block behavior remains open. Surface/polysurface Join and Split/Trim targets, multi-target editing, select-after-invoke, and Mirror/Rotate options remain open. No claim of full Top 11 parity.
