# Independent review: reusable NURBS basis contract

Date: 2026-10-10. Baseline: `fcdb0ac`. Scope: Task 2 in
`docs/superpowers/plans/2026-10-09-rhino-core-kernel-contract.md`, against
RCORE-01's NURBS data and invalid-input requirements. Reviewed the new
`rust/src/nurbs_basis.rs`, its tests, the `spline::Basis::validate` wrapper,
unchanged generated spline consumers/tests, the final `nurbs_basis_ffi` interface
and tests, and their module exports. Refreshed at the final frozen portable
checkpoint. Geometry descriptor policy has its separate review; native adapter
placement, native runtime and full acceptance remain outside this review.

## Conclusion

No blocking finding in the reviewed portable validator, generated wrapper and
Rust basis interface at the final hashes below.
This conclusion does not accept full P0, certify Rhino runtime parity, or close
RCORE-01.T07. Native adapter allocation placement, integer ranges, ownership,
caller-established buffer lifetime and the literal all-consumer allocation
requirement require separate native evidence.

## Checks and evidence

- Count-only curve/surface preflights use checked multiplication/addition, reject
  grid/count/weight mismatches, enforce explicit policy ceilings and reject a
  logical payload above `isize::MAX` before value loops. The surface ceiling
  bounds the complete U-major grid, while knot ceilings apply to each axis.
- The safe borrowed basis core receives slices, allocates no input buffer and
  does not expand multiplicities. Both surface axes receive structural validation.
- Rust owns the explicit native conversion policy exposed by
  `om9_basis_native_limits`: degree 25, one million poles, one million knots per
  axis, 64 MiB logical payload and finite coordinates within 1e150. Generated
  limits remain separate. These OM9 ceilings do not certify all native backend
  inputs or include copies, allocator overhead and aggregate worker memory.
- The FFI performs checked policy/degree/count/product/byte preflight before
  creating slices, then checks null nonempty buffers, alignment, address overflow,
  periodic flags and the absent-weight null/zero convention. Unsafe slicing stays
  in this small interface and no pointers are retained. Its documented caller
  obligations cover initialized immutable live allocations and concurrent writes;
  numeric address checks cannot establish accessibility or lifetime.
- Degree/end multiplicity rules and periodic `sum - seam_multiplicity` cardinality
  match the local OCCT constructor contract in
  `H:/FreeCAD-src/.pixi/envs/default/Library/include/opencascade/Geom_BSplineCurve.hxx`
  lines 135–167 and `Geom_BSplineSurface.hxx` lines 185–213. In particular, applying
  the generated degree-plus-one pole minimum to every periodic native axis would
  incorrectly restrict this native convention. This is source evidence, not a
  native fixture result.
- Nonperiodic domain endpoints use the active expanded-knot indices without
  allocating that expanded vector. A degree exceeding the available nonperiodic
  poles cannot admit a positive interval within the resulting bounds.
- Poles/weights/knots/domain require finite data; weights remain positive and
  cannot overflow homogeneous coordinates. Checked multiplicity totals prevent
  integer wraparound. These are structural checks, not conditioning, geometric
  validity, or native-kernel support certification.
- The generated wrapper retains `rebuild_options`, limits of degree 1–11,
  at most 256 poles, coordinates within 1e9, and the supported legacy error
  strings. Count checks now precede pole-value checks when multiple independent
  faults are present; that follows the preflight requirement and does not change
  single-fault acceptance or supported messages.
- Earlier in this review, independently compiled the actual basis and spline sources in a temporary
  standalone `rustc --test` harness. All 28 existing focused tests passed, plus
  three additional review tests: payload above the addressable slice limit
  without allocating, malformed structure on either surface axis, and count/byte
  rejection ahead of NaN pole inspection. Result: **31 passed, 0 failed**.
  Retained output: `H:/FreeCAD-src/build/om9-basis-review-tests.log`.
- Earlier checkpoint: inspected the builder's UTF-16 focused/full logs and exit JSON. Focused:
  **28 passed**. Independently summed the full log's 60 result records:
  **441 passed, 0 failed**. Recorded focused/full/clippy exits are zero. This
  review did not independently run clippy, a native build, a release build or GUI.
- An initial independent Cargo run encountered `E0583` for the concurrent
  `geometry_contract` module export before its source file existed. The standalone
  harness avoided that work in progress. This is historical evidence, not the
  final state. The overwritten RED log cannot establish the original RED run.
- Final frozen checkpoint: independently ran Cargo's six focused integration
  targets against the combined crate. **67 passed, 0 failed, exit 0**: 20 basis,
  12 basis FFI, 3 generated spline basis, 5 spline geometry, 14 geometry contract
  and 13 descriptor FFI. The basis/generated/FFI slice is **40 passing tests**.
  Only focused Rust tests were rerun for this refresh; no SDK/native, GUI, release
  or full-suite execution was performed by this reviewer at this checkpoint.

## Reviewed content hashes (SHA-256)

| File | Hash |
|---|---|
| `rust/src/nurbs_basis.rs` | `50d4fe44e2a325ff2f614c1f06d263d42cc1f16fc3504e4240863f395e41e916` |
| `rust/tests/nurbs_basis.rs` | `90ad8b2697df44be3dd888dc6d7396c63d432e66ca2436477a79335a804567da` |
| `rust/src/spline.rs` | `ada29d3f4facac70a82fa77b75911955521a7583d6f0de18e15313abb97dfa01` |
| `rust/tests/spline_basis.rs` | `8e8133d888aca96ddf1ddb6c67e1f8cf97711460275571f84c9cfc93249fad6c` |
| `rust/src/nurbs_basis_ffi.rs` | `c2c6f1ea5afd97fc1ce41993e37bd4bd85a6a6f8c27d1753a13646966ba3be58` |
| `rust/tests/nurbs_basis_ffi.rs` | `d7f331f333401d076589874de781f0cec7f172d9f8c0ebc13819076feac375f0` |
| `rust/src/lib.rs` | `ef8648b92fc44a2d69a384f70a7bc68484979cdd993e632943337b50d746e270` |

The module-export hash covers the final portable combination. Descriptor/operation
semantics are reviewed in `2026-10-10-geometry-contract-review.md`; native source,
runtime binaries and macro changes are not covered by these Rust hashes.
