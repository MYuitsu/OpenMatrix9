# OM9-L2-02-06 — 06-febe-rhino-transform-orient-workflows

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-02-06` — six L3 owners, 31 L4 records
- **Delivery date:** 2026-10-22
- **Status:** core sessions/math `AUTOMATED_TARGET_VALIDATED`; six acceptance records `READY_FOR_REVIEW` (M03-UI)
- **Crates:** `om9-core` transform sessions/gumball, `om9-kernel` exact transform math, `om9-app` preview/commit

## What was delivered

| L3 | Capability | Rust entry point |
|---|---|---|
| `OM9-L3-02-06-001` | Interactive Move Copy And Rotate | om9_core::transform_session::{MoveSession, RotateSession} |
| `OM9-L3-02-06-002` | Interactive Scale 1d 2d 3d | om9_core::transform_session::{ScaleSession, ScaleAxes} |
| `OM9-L3-02-06-003` | Interactive Mirror And Array | om9_kernel::transform_ops::{mirror, linear_pattern, polar_pattern, apply_pattern} |
| `OM9-L3-02-06-004` | Orient By Two Three Reference Points | om9_kernel::transform_ops::{orient_two_point, orient_three_point} |
| `OM9-L3-02-06-005` | Align And Setpoints | om9_core::transform_session::{set_point_transform, align_point_to_plane, align_point_to_axis} |
| `OM9-L3-02-06-006` | Transform Cancel Preview And Undo | MoveSession/RotateSession/ScaleSession preview/cancel |

## Measured acceptance (this checkout)

```
cargo test -p om9-core --test transform_sessions
cargo test -p om9-kernel --features 3dm-brep --test l2_02_06
cargo test -p om9-app --test gumball_interaction --test gumball_numeric --test gumball_placement --test gumball_cold
```

8 core session tests, 3 kernel transform tests, 11 prior transform-math tests and 17 app gumball tests pass.
Move/rotate/scale preview and cancel are pure; commit is one Undo entry; the orientation solver is tested with a non-identity rotation.

## Verification commands

```
cargo test -p om9-core --test transform_sessions
cargo test -p om9-kernel --features 3dm-brep --test l2_02_06
cargo test -p om9-kernel --features 3dm-brep --test l2_01_11
cargo test -p om9-app --test gumball_interaction --test gumball_numeric --test gumball_placement --test gumball_cold
cargo clippy -p om9-core --test transform_sessions -j 2
cargo clippy -p om9-kernel --features 3dm-brep --test l2_02_06 -j 2
rustfmt --edition 2024 --check crates/om9-core/src/transform_session.rs crates/om9-core/tests/transform_sessions.rs crates/om9-kernel/tests/l2_02_06.rs crates/om9-kernel/src/transform_ops.rs
python tools/public_source_audit.py
```

## Evidence layout

- `implementation/design/M03/OM9-L3-02-06-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-02-06-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-02-06-00X/` — one receipt per planned L4 role.

## Deliberate scope limits

- All six M03-UI reference-picker/preview/commit/cancel workflows remain unsigned.
- The orientation solver fix (`target_frame^T * source_frame`) is covered by a non-identity rotation; the previous translation-only tests would not catch it.
- Align/SetPt include translate, rotate, scale and rotate+scale variants; the UI target picker is still manual.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through `runtime_implementation` plus this evidence package.
