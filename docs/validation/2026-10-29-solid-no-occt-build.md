# Solid no-OCCT developer build — 2026-10-29

The previous no-default-features build had 20 unresolved Sphere/Box references to OCCT-gated implementations. The Sphere pending handler is now gated and no-OCCT Box/Sphere helpers return unsupported; the ray helper has no special height projection. Solid capability checks and the CMD entry reject these native-dependent commands when OCCT is absent instead of opening an unusable session.

With `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo check -j 2 -p om9-app --no-default-features`: passed with 12 compiler warnings; this is a developer fallback, not the product renderer or full runtime validation.
- `cargo test -j 2 -p om9-app --no-default-features --test solid_no_occt`: 1 passed. It checks Sphere and Box direct helpers, Menu dispatch, CMD, no active session, no document mutations or Undo transaction. The first test run before the CMD guard failed because `Sphere` incorrectly opened a session.
- Default OCCT: `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 52 passed; `solid_box_`: 36 passed; `solid_slab_`: 5 passed.
- `python tools/public_source_audit.py`: 14733 source files, 594 icon bindings, 593 SVGs, no errors.

Only Sphere/Box rejection and the stated regressions were exercised. Other Solid commands, desktop UI, actual GPU runtime and the no-OCCT fallback as a whole remain unverified.
