# Sphere AroundCurve straight BRep edge — 2026-10-29

AroundCurve now accepts `Path=Object.EdgeN` on a BRep when the selected edge is straight. It uses the native oriented endpoints to evaluate a line, captures the source stamp, and shares the existing point projection, fraction, preview and commit paths. A whole BRep is accepted only when it has exactly one edge; an invalid index or unsupported curved edge is rejected without commit. BRep Tangent remains unsupported.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_aroundcurve_accepts_named_straight_brep_edge`: three box-edge fixtures passed (fraction, picked point, deleted source); tests invalid path/index, preview and Undo.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 49 passed.
- `python tools/public_source_audit.py`: 14720 source files, 594 icon bindings, 593 SVGs, no errors.

The fixture uses an identity-placement box and named-path controller input. Curved BRep edges, direct subobject selection, BRep placement variants, full desktop/viewport, original Matrix behavior and complete Sphere/Solid remain open.
