# OM9-L3-03-08-003 — Top11 Mirror Move Rotate Actions API and frozen fixture

- **Owner:** `OM9-L2-03-08` (07-febe-matrix-top11-and-shortcut-actions), milestone M03
- **Goal on record:** quick toolbar bindings to transform engine
- **Crate / host:** `om9-core::top11` + `om9-app` toolbar/controller adapter
- **Units:** millimetres and degrees
- **Evidence date:** 2026-10-23

## Scope delivered

Move and Rotate collect points through accept_world_point and commit placement transforms. Mirror supports Copy or Delete-Input and switches between the two-point vertical plane and a three-point arbitrary plane by typing 3Point while its prompt runs (or by the Mirror 3Point command). It bakes rigid or non-rigid placements: affine circles become NURBS ellipses, mesh/cloud normals and winding are transformed, and BRep bodies mirror through OCCT.

## Frozen API

- `om9_core::top11::translate_placements`
- `om9_core::top11::rotate_placements`
- `om9_core::top11::rotation`
- `om9_core::top11::mirror_entities`
- `om9_core::top11::MirrorOutcome`
- `om9_app::session::top11::{Top11Pending, top11_accept_point}`

## Input rules

- The action reads the current document selection and the recorded ObjectIds; every referenced object must still exist.
- Points, fractions, radii and matrices are finite before use; a refusal is a typed `Error`.
- Every document change is one `TransactionRequest`; a refused action publishes nothing.

## Behaviour and measured oracle

Positive fixture: Move origin (5,-2,0); Rotate end (0,10,0); Mirror copies 1..4 mm to -1..-4; Delete reflects in place; a three-point plane through (0,0,0),(0,1,0),(0,0,1) also gives x=-1; a non-rigid x-scale 2 line mirrors its 2 mm world end to -2; a scaled BRep mirrors to bounds -4..0; a placed BRep mirrors to volume 24.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `mirror_delete_entities` | 1.000000000000 | exact integer |
| `mirror_delete_start_x_mm` | -1.000000000000 | 1e-9 |
| `mirror_copies` | 2.000000000000 | exact integer |
| `mirrored_start_x_mm` | -1.000000000000 | 1e-9 |
| `three_point_mirror_start_x_mm` | -1.000000000000 | 1e-9 |
| `non_rigid_mirror_end_x_mm` | -2.000000000000 | 1e-9 |
| `move_base_mm` | [5.0, -2.0, 0.0] | exact value |
| `rotate_end_mm` | [6.123233995736766e-16, 10.0, 0.0] | 1e-6 |
| `brep_cutter_entities` | 4.000000000000 | focused assertion |
| `exact_brep_fractions` | [0.25, 0.75] | focused assertion |
| `brep_non_rigid_mirror_bounds` | [-4.0000001, -1e-7, -1e-7, 1e-7, 3.0000001, 4.0000001] | 1e-4 |
| `brep_mirror_copy_bounds` | [-7.0000001, -1e-7, -1e-7, -4.9999999, 3.0000001, 4.0000001] | focused assertion |
| `exact_nurbs_polycurve_fractions` | [0.25, 0.75] | focused assertion |
| `brep_mirror_volume_mm3` | 24.000000000000 | 1e-6 |
| `brep_mirror_local_bounds` | [-12.0000001, -1e-7, -1e-7, -9.9999999, 3.0000001, 4.0000001] | 1e-4 |

Focused run:
`cargo test -p om9-app --no-default-features --test top11 m03_03_08_003 -- --nocapture`.

## Negative fixture

- Move/Rotate/Mirror with an empty selection
- mirror points that coincide or give a degenerate plane
- mirror of a singular placement or an unsupported geometry kind
- BRep mirror without the occt-brep feature
- a pending command blocks another action

A refused action returns before a mutation is prepared; the document, selection and Undo journal stay unchanged.

## Ownership, budgets and determinism

- Safe Rust owns the action catalogue, availability and the entity/geometry builders; the app owns selection, viewport input and the single transaction.
- Allocations are `try_reserve`-checked and the transaction is admitted by the coordinator `Limits` before commit.
- The same selection and points produce the same object count, placement and Undo entry.

## Source decision

`ADD`. QymCAD has sketch/part mirror and transform tool state, but no Matrix Top 11 command adapter over the OM9 document. The placement/reflection math is additive safe Rust. See `source_crosswalk.json`.

## Deliberate limits

- The toolbar Mirror starts with the two-point vertical plane; typing 3Point (or the Mirror 3Point command) switches to an arbitrary three-point plane and resets the picks.
- Non-rigid circle placements become NURBS ellipses; an improper placement does not rewrite mesh winding beyond the reflection swap.
- BRep mirror needs the occt-brep feature (the product default); without it the action refuses with a typed error.
- Rotate uses the active CPlane normal; arbitrary axis picking is not part of the toolbar path.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-core/src/top11.rs` | `70fa4ac7e94c177fcf25c14ccac39a3417e85da2a232e03988de0822974ef904` |
| `crates/om9-core/src/curve_ops.rs` | `2a78e9927fe74e15b974463c5542cfccfcf7b04bdd1118a185e4411a9470055e` |
| `crates/om9-core/src/curve_surface_ops.rs` | `fcc811f2e0fdb757edc7106bd8e0717a1446552b48ed3a04fcda51064601df33` |
| `crates/om9-core/src/surface_kernel.rs` | `4492e25e5690cd482060649ceb18948725bdf8d16abbef1f77f3f6c6909a903b` |
| `crates/om9-kernel/src/intersect.rs` | `ff95a75050654d0dbb6411ee19690996f1c84eda1feaec1c775edca25074f1cd` |
| `crates/om9-kernel/src/intersect_bridge.cpp` | `1f77c466575fddb5f6e3d9b6cb616cc37276e1aca6da0ff7d9cbb3f6afab271b` |
| `crates/om9-app/src/session/top11.rs` | `eae74e026f28b83fe0e465b9fd0ec6728bfb3c39738a73f7bae8477da15394a4` |
| `crates/om9-app/src/ui/top11.rs` | `23b30de40ea4bb0ecf19e8ca927cccf626a4fa2fc91451865d76164f6c46b77e` |
| `crates/om9-app/src/ui/chrome.rs` | `c3794665049762b5e8f3c6ad57759fdc54c743d875e17847229cf7d411729788` |
| `crates/om9-app/src/ui.rs` | `3338a9338c1b50f24299b69809d8ceea0f07459e94d4de65b7913ec9d818a374` |
| `crates/om9-app/src/session.rs` | `5b3bd86b8c9433c38e64d5464cec198419a627d112d0891dba1ec58962a3de33` |
| `crates/om9-app/src/icons.rs` | `6256b8a9a0be5f529f90aae7394d0c0a6a73c5b501caa0e9824f5037a3a9331a` |
| `crates/om9-app/tests/top11.rs` | `de4189f7e9a3e2bd3bccc994bc8bb33b4a5eb5c95bf4d90beb5ed135d9225d19` |
| `crates/om9-app/tests/top11_brep.rs` | `0b04fe9a9076a8e2b45759d9a353009541a7837d304b19ebfacf262cedfb01e5` |
| `crates/om9-core/tests/top11.rs` | `b97b3eef145bbf48f355d4f8ff4406636b6200c44235003c21b78986df328e02` |

## Receipts

- focused: `cargo test -p om9-app --no-default-features --test top11 m03_03_08_003 -- --nocapture`
- regression: `cargo test -p om9-app --no-default-features --lib --test top11 -j 2` (195 passed, 0 failed)
- core regression: `cargo test -p om9-core --test top11 -j 2` (10 passed, 0 failed)
- clippy: `cargo clippy -p om9-core -p om9-app --no-default-features --lib --test top11 -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `27052c6036384d41ad0b933e865da1b8cdb046199770dcf78f5630e92ae00b6b`.
BRep mirror log SHA-256: `331e7fb35a78a3f7b3a10e86bc97bf2cc87bf0f32df50d39f6f6a9dbef242a68`.
