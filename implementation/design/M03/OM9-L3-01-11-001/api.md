# OM9-L3-01-11-001 — Translation Rotation Scaling Matrices API and frozen fixture

- **Owner:** `OM9-L2-01-11` (M03 transforms and pattern operations)
- **Goal on record:** translation, rotation and scaling matrices applied to points and B-Reps
- **Crate / adapter:** `crates/om9-kernel/src/transform_ops.rs` over `crates/om9-kernel/src/transform_bridge.cpp`
- **Units:** millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn translation(offset: Point3) -> Result<Transform, TransformError>`
- `pub fn rotation(origin: Point3, axis: Point3, angle: f64) -> Result<Transform, TransformError>`
- `pub fn scaling(origin: Point3, factors: Point3) -> Result<Transform, TransformError>`
- `pub fn inverse(matrix: &Transform) -> Result<Transform, TransformError>`
- `pub fn compose(outer: &Transform, local: &Transform) -> Result<Transform, TransformError>`
- `pub fn apply(matrix: &Transform, point: Point3) -> Result<Point3, TransformError>`
- `pub fn transform_brep(brep: &Brep, matrix: &Transform, deflection_mm: f64, limits: &Limits) -> Result<Brep, TransformError>`

`TransformError` is the shared typed contract (`CreationError`): `Invalid(String)` for an
inadmissible input, `Budget(String)` for a measured budget, `Kernel(String)` for an OCCT failure
after admission.

## Input rules (rejected with `CreationError::Invalid`)

- every matrix value must be finite and the affine last row must be (0,0,0,1)
- the 3×3 determinant must be non-zero (else the matrix is singular)
- deflection must be finite and positive

## Behaviour

The matrix service builds row-major affine matrices and composes them in the documented order. The B-Rep adapter applies the matrix through BRepBuilderAPI_GTransform and repairs orientation when the determinant is negative.

## Positive fixture and measured oracle

- translate (1,2,3) by (4,-1,2) -> (5,1,5)
- rotate (1,0,0) 90° about Z -> (0,1,0)
- compose then inverse restores the source point
- 2×3×4 box translated by (5,0,0): volume 24 mm³, bounds x 5..7

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `translated_volume_mm3` | 24.000000000000 | 1e-6 |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_11_001`.

## Negative fixture

- a singular matrix has no inverse
- a NaN matrix is refused

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

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_001 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_11`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
