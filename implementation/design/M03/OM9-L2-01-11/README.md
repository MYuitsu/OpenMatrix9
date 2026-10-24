# OM9-L2-01-11 — 03-be-occt-transforms-and-pattern-ops

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-01-11` — eight L3 owners, 41 L4 records
- **Delivery date:** 2026-10-22
- **Status:** `AUTOMATED_TARGET_VALIDATED` (`manual_test_gate: AUTO_ONLY`)
- **Kernel:** OCCT 8.0.1, pinned at `native/occt/pin.json` (`V8_0_1`, `b8f597c…`)
- **Crate:** `crates/om9-kernel` — `src/transform_ops.rs` (safe Rust) over
  `src/transform_bridge.cpp` and the shared `src/brep_snapshot.hxx`

## What was delivered

| L3 | Capability | Rust / native entry point |
|---|---|---|
| `OM9-L3-01-11-001` | Translation/rotation/scaling matrices | `transform_ops::{translation,rotation,scaling,inverse,compose,apply}`, `om9_occt_transform_brep` |
| `OM9-L3-01-11-002` | Mirror across a plane | `transform_ops::mirror` + oriented body transform |
| `OM9-L3-01-11-003` | Non-uniform scaling and units | `transform_ops::scaling` (negative determinant mirror) |
| `OM9-L3-01-11-004` | Polar and linear instance pattern | `linear_pattern`, `polar_pattern` |
| `OM9-L3-01-11-005` | Curve sampled placement frames | `frames_along_polyline`, `frames_along_curve` |
| `OM9-L3-01-11-006` | Surface UV/normal placement frames | `frame_on_surface` over the surface kernel derivatives |
| `OM9-L3-01-11-007` | Orientation two/three-point solver | `orient_two_point`, `orient_three_point` |
| `OM9-L3-01-11-008` | Array bake vs instance policy | `PatternMode::{Instance,Bake}`, `apply_pattern` |

## Ownership and rules

- Safe Rust owns every matrix rule (finite, non-singular, handedness), the frame construction and
  validation (`x × y = z`, unit axes), the pattern placement generation, the orientation solver and
  the bake/instance policy. C++ only applies the admitted affine matrix to the OCCT shape and
  repairs the orientation after a negative determinant.
- A transform is applied to an immutable `Brep` and returns a new `Brep`; the input is never
  mutated.
- A pattern in instance mode stores one definition plus placements (no geometry duplication);
  bake mode creates one independent body per placement. All placements are validated before any
  body is transformed, so a bad placement never leaves a partial result.
- Frames are right-handed orthonormal triples; a degenerate derivative or a zero tangent is a
  typed refusal, never a silently skewed frame.

## Measured acceptance (this checkout)

`cargo test -p om9-kernel --features 3dm-brep --test l2_01_11 m03_01_11_receipt -- --exact --nocapture`

| Fixture | Measured |
|---|---:|
| Translated 2×3×4 box | 24.000000000000 mm³ |
| Mirrored about x=0 | 24.000000000000 mm³, x min −2.0000001 mm |
| Scaled (2,3,4) | 576.000000000000 mm³, size x 4.0000002 mm |
| Linear pattern placements | 4 (x = 0, 10, 20, 30) |
| Instance definitions | 1 (shared geometry) |
| Baked bodies | 4 |
| Curve frame mid station | z = 5.000000000000 mm |
| Surface frame at UV (2,3) | origin (2.000000, 3.000000, 0) |
| Two-point orientation distance | 1.000000000000 mm |

Negative fixtures (singular matrix inverse, NaN matrix/normal, zero scale factor, zero mirror
normal, count 0, negative spacing, pattern overflow, empty placements, one frame sample, repeated
rail point, degenerate surface derivatives, out-of-domain UV, coincident/collinear orientation
points) all return a typed error; no body is published.

## Verification commands

```
cargo test -p om9-kernel --features 3dm-brep --test l2_01_11
cargo test -p om9-kernel --features 3dm-brep
cargo check --workspace --all-targets -j 2
cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings
cargo fmt --all -- --check
python tools/public_source_audit.py
python tools/om9_milestone_dashboard.py
```

### Runtime DLLs

The test executable links OCCT dynamically, so `TKernel.dll`/`TKShHealing.dll` and their
`tbb12.dll`/`jemalloc.dll` dependencies must be resolvable. Either add these directories to `PATH`
or keep the DLLs beside the test executable (`<target>/debug/deps`):

- `build/standalone-deps/occt-8.0.1-sdk/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/tbb-2021.13.0-x64/bin`
- `build/standalone-deps/occt-8.0.1-3rdparty/3rdparty-vc14-64/jemalloc-vc14-64/bin`

## Evidence layout

- `implementation/design/M03/OM9-L3-01-11-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-01-11-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-01-11-00X/` — one receipt per L4 (implementation, regression,
  integration-stress where planned, acceptance and the AUTO_ONLY review record).

## Deliberate scope limits

- Curve frames are delivered for an arc-length resampled polyline and for an exact NURBS curve via
  the surface kernel; the B-Rep curve adapter is a separate owner task.
- Mirroring is delivered through the affine body transform; a dedicated mirror feature object is
  an M04 history concern.
- macOS execution is not verified.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through
  `runtime_implementation` plus this evidence package.
