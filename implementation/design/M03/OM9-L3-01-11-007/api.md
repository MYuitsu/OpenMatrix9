# OM9-L3-01-11-007 — Orientation Two Three Point Solver API and frozen fixture

- **Owner:** `OM9-L2-01-11` (M03 transforms and pattern operations)
- **Goal on record:** rigid orientation from two or three reference points
- **Crate / adapter:** `crates/om9-kernel/src/transform_ops.rs` over `crates/om9-kernel/src/transform_bridge.cpp`
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn orient_two_point(source_a: Point3, source_b: Point3, target_a: Point3, target_b: Point3) -> Result<Transform, TransformError>`
- `pub fn orient_three_point(source: [Point3; 3], target: [Point3; 3]) -> Result<Transform, TransformError>`

`TransformError` is the shared typed contract (`CreationError`): `Invalid(String)` for an
inadmissible input, `Budget(String)` for a measured budget, `Kernel(String)` for an OCCT failure
after admission.

## Input rules (rejected with `CreationError::Invalid`)

- reference points must be distinct; three-point references must be non-collinear
- the result is rigid: distances are preserved and the determinant is +1

## Behaviour

Two points use the minimal rotation between the source and target directions; three points build orthonormal frames from (B−A, C−A) and use R = F_target · F_sourceᵀ.

## Positive fixture and measured oracle

- A(0,0,0),B(1,0,0) → A′(10,0,0),B′(10,1,0): both points map exactly, distance 1 preserved
- a three-point triangle maps all three points with determinant +1

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `orientation_distance_mm` | 1.000000000000 | 1e-9 |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_11_007`.

## Negative fixture

- coincident two-point references
- three collinear points

A refusal returns before a `Brep` or `PlacementFrame` exists; an input body is an immutable value
and is never mutated by a failed operation.

## Ownership, budgets and determinism

- Every body transform is one synchronous complete operation; no `TopoDS_Shape` or OCCT handle
  crosses into safe Rust.
- Matrix validation (finite, affine last row, non-singular determinant) happens before the native
  call; handedness is reported and a negative determinant is repaired by the adapter.
- Patterns validate every placement before transforming anything; instance mode stores one
  definition, bake mode one independent body per placement.
- The snapshot sink copies bounded arrays; source, points, nodes and bytes are checked against
  `Limits`.

## Source decision

`ADD`: the matrix/frame/pattern service is new safe Rust and the B-Rep application is a new OCCT
8.0.1 adapter. QymCAD exposes typed transform/copy asks over its own C ABI and has no placement
policy service; no code is reused. See `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-kernel/src/transform_ops.rs` | `9c8ccb954884f360dfacf84d2712777ae73b308ee4a87af1a2ab8ffd4e6202ef` |
| `crates/om9-kernel/src/transform_bridge.cpp` | `de6c911826366c34c952d06b02556d951f91b6f7aeda6986a73b6cc0e7724426` |
| `crates/om9-kernel/src/brep_snapshot.hxx` | `b50814193b7b3570a083b2f23f357834ef3f2b6c3e43f5ffcfd955ed7a674e22` |
| `crates/om9-kernel/src/brep.rs` | `1700c7e0271399b8dd2f3a50d776f3680e20cb34103acc1ec8fbc807b63e19d8` |
| `crates/om9-kernel/src/creation.rs` | `7f2e2dc0849d9497d9b202bdd23332989091ee2e8224a04ebfc8be1beb2a5e0f` |
| `crates/om9-kernel/src/lib.rs` | `7814f004c9b8d52a950045bc6108abe9c69315991864a3d9716f885cc836e529` |
| `crates/om9-kernel/build.rs` | `b54fb898bd16d7911b2ed44e77141a18e5f32f52e06c9d654f1a125281fc861b` |
| `crates/om9-kernel/tests/l2_01_11.rs` | `d445023fb216df271171b9548180f051eb3fccf8a7bcfb2332faa246c1335034` |

## Receipts

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_007 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
