# Sphere AroundCurve circular BRep edge — 2026-10-29

The OCCT adapter now extracts bounded circular edge frame, radius, parameter bounds and orientation through a synchronous fixed-buffer query; no native handle or document pointer crosses the ABI. Rust constructs the oriented curve and retains all session and publication ownership. Sphere AroundCurve named paths accept these circular edges in addition to straight ones, including closed circles. Other curved types remain unsupported.

Standalone OCCT 8.0.1, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_aroundcurve_accepts_named_circular_brep_edge`: cylinder closed-edge fraction, picked point, preview, stale source and Undo passed; position checked against cylinder radius and end plane.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 50 passed.
- `cargo test -j 2 -p om9-kernel --features 3dm-brep --test l2_01_10`: 60 passed. A prior attempt without `--features 3dm-brep` failed at link due to missing 3DM bridge symbols; the product feature-enabled run passed.
- `python tools/public_source_audit.py`: 14721 source files, 594 icon bindings, 593 SVGs, no errors.

The fixture covers an identity-placement full circle; trimmed arcs, periodic seam, non-circular curves, BRep transforms, direct viewport selection, Matrix original and complete Sphere/Solid remain unverified. Native exception is required to access OCCT edge geometry; Rust checks finite returned data, owns curve/session state and prevents stale publication. Native FFI is not certified memory-safe by its Rust wrapper.
