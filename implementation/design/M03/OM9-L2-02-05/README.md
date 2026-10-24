# OM9-L2-02-05 — 05-febe-freeform-surface-modeling

- **Milestone:** M03 (Geometry 3DM)
- **Owner:** `OM9-L2-02-05` — ten L3 owners, 55 L4 records
- **Delivery date:** 2026-10-22
- **Status:** core services `AUTOMATED_TARGET_VALIDATED`; all ten acceptance records `READY_FOR_REVIEW` (M03-UI)
- **Crates:** `om9-core` portable surface services, `om9-kernel` exact OCCT B-Rep

## What was delivered

| L3 | Capability | Rust entry point |
|---|---|---|
| `OM9-L3-02-05-001` | Sweep1 Sweep2 And Profile Sweep Workflows | om9_core::surface_kernel::{sweep1, sweep2, path_frames, arc_length_stations, Sweep2Options} |
| `OM9-L3-02-05-002` | Loft And Curve Network Workflows | om9_core::surface_kernel::loft and om9_kernel::creation::loft |
| `OM9-L3-02-05-003` | Revolve Railrevolve And Extrude Workflows | om9_kernel::creation::{revolve, extrude, extrude_wire} |
| `OM9-L3-02-05-004` | Edge Planar And Patch Surface Workflows | om9_core::surface_kernel::plane |
| `OM9-L3-02-05-005` | Trim Split Untrim And Shrink Surface | om9_core::surface_ops::{trim_surface, untrim_surface, split_surface, shrink_surface} |
| `OM9-L3-02-05-006` | Surface Rebuild Refit And Softedit | om9_core::surface_kernel::refit |
| `OM9-L3-02-05-007` | Surface Match Blend And Join Continuity | om9_core::surface_ops::{match_surface, blend_surface, SurfaceContinuity, SurfaceEdge, MatchReport, BlendTangents} |
| `OM9-L3-02-05-008` | Surface Offset Fillet Chamfer | om9_core::surface_ops::{offset_surface, normal_extrude} |
| `OM9-L3-02-05-009` | Surface UV Direction And Isocurve Handling | om9_core::surface_kernel::{isocurve, SurfaceIso} |
| `OM9-L3-02-05-010` | Surface Normal Extrude And Unroll | om9_core::surface_ops::{normal_extrude, unroll} |

## Measured acceptance (this checkout)

```
cargo test -p om9-core --test surface_ops
cargo test -p om9-kernel --features 3dm-brep --test l2_02_05
```

16 core tests and 6 kernel tests pass. Key measured values: Coons centre 0.5, rail-revolve helix
[2cos, 2sin, t], trim 4 outer/1 inner loop, softedit centre z 0.2835, match G0 gap 5.55e-17 mm,
G1 angle 1.20e-4 rad, join 2 faces/1 shell, box offset 6733.33 mm3, cylinder unroll 31.2135x8 mm,
patch area 8 mm2 and thickened volume 4 mm3.

## Verification commands

```
cargo test -p om9-core --test surface_ops
cargo test -p om9-kernel --features 3dm-brep --test l2_02_05
cargo test -p om9-core --lib -j 2
cargo test -p om9-kernel --features 3dm-brep -j 2
cargo clippy -p om9-core --test surface_ops -j 2 -- -D warnings
cargo clippy -p om9-kernel --features 3dm-brep --test l2_02_05 -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-core/src/surface_ops.rs crates/om9-core/src/lib.rs crates/om9-core/tests/surface_ops.rs crates/om9-kernel/tests/l2_02_05.rs
python tools/public_source_audit.py
```

## Evidence layout

- `implementation/design/M03/OM9-L3-02-05-00X/{api.md,source_crosswalk.json}`
- `implementation/fixtures/M03/OM9-L3-02-05-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-02-05-00X/` — one receipt per planned L4 role.

## Deliberate scope limits

- All ten M03-UI workflows (rail/profile/boundary selection, preview, commit/cancel, warning) remain unsigned.
- Network is four-sided Coons; n-sided/non-four-sided networks are not implemented.
- Match is a boundary-row replacement with a measured angle, not a full surface optimization.
- Seam is a C0 metadata closure; true periodic re-parameterization is not represented.
- Unroll is an average arc-length planar map; exact developable unroll of a doubly curved surface is not claimed.
- Portable surface offset is a sampled/refit approximation; exact offset remains the OCCT B-Rep service.
- The planning tree keeps `SPEC_ONLY_NOT_IMPLEMENTED`; delivery is recorded through `runtime_implementation` plus this evidence package.
