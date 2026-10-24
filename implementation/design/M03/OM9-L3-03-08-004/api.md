# OM9-L3-03-08-004 — Top11 Explode Join Split Trim Actions API and frozen fixture

- **Owner:** `OM9-L2-03-08` (07-febe-matrix-top11-and-shortcut-actions), milestone M03
- **Goal on record:** toolbar bindings for edit commands
- **Crate / host:** `om9-core::top11` + `om9-app` toolbar/controller adapter
- **Units:** millimetres and degrees
- **Evidence date:** 2026-10-23

## Scope delivered

Explode a polycurve, join an unordered curve selection, split a target at every intersection with one or more picked cutting objects (curve, NURBS surface or BRep), and trim the portion picked on the target. The default build asks OCCT for exact curve/face intersections; the no-default developer build refuses a BRep cutter because it has no exact kernel path. CMD fraction forms remain available. Every action is one transaction with Undo/Redo.

## Frozen API

- `om9_core::top11::explode_entities`
- `om9_core::top11::join_entities`
- `om9_core::top11::split_entity`
- `om9_core::top11::trim_entity`
- `om9_core::top11::{QUICK_SPLIT, QUICK_TRIM}`

## Input rules

- The action reads the current document selection and the recorded ObjectIds; every referenced object must still exist.
- Points, fractions, radii and matrices are finite before use; a refusal is a typed `Error`.
- Every document change is one `TransactionRequest`; a refused action publishes nothing.

## Behaviour and measured oracle

Positive fixture: join 1, explode 2; one cutter splits 3 objects; two cutters split 5 objects and trim leaves 4; a NURBS plane cuts one line into 2 new pieces; a BRep box cuts one line into 3 new pieces; an exact sphere/line cut returns fractions 0.25 and 0.75; Trim keeps the half starting at x=0; 50 stress cycles complete.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `multi_cutter_split_entities` | 5.000000000000 | exact integer |
| `multi_cutter_trim_entities` | 4.000000000000 | exact integer |
| `joined_entities` | 1.000000000000 | exact integer |
| `exploded_entities` | 2.000000000000 | exact integer |
| `split_entities` | 3.000000000000 | exact integer |
| `trim_remaining_entities` | 2.000000000000 | exact integer |
| `trim_kept_start_x_mm` | 0.000000000000 | 1e-9 |
| `surface_cutter_entities` | 3.000000000000 | exact integer |
| `stress_cycles` | 50.000000000000 | exact integer |
| `brep_cutter_entities` | 4.000000000000 | exact integer |
| `exact_brep_fractions` | [0.25, 0.75] | 1e-6 |
| `brep_non_rigid_mirror_bounds` | [-4.0000001, -1e-7, -1e-7, 1e-7, 3.0000001, 4.0000001] | focused assertion |
| `brep_mirror_copy_bounds` | [-7.0000001, -1e-7, -1e-7, -4.9999999, 3.0000001, 4.0000001] | focused assertion |
| `exact_nurbs_polycurve_fractions` | [0.25, 0.75] | 1e-6 |
| `brep_mirror_volume_mm3` | 24.000000000000 | focused assertion |
| `brep_mirror_local_bounds` | [-12.0000001, -1e-7, -1e-7, -9.9999999, 3.0000001, 4.0000001] | focused assertion |

Focused run:
`cargo test -p om9-app --no-default-features --test top11 m03_03_08_004 -- --nocapture`.

## Negative fixture

- explode of a native single curve
- join of fewer than two curves, a gap, or different placements
- split/trim without exactly one selected target
- a cutting curve with no intersection
- target and cutter in different placements or with the same id
- split fraction at 0 or 1
- trim with from >= to

A refused action returns before a mutation is prepared; the document, selection and Undo journal stay unchanged.

## Ownership, budgets and determinism

- Safe Rust owns the action catalogue, availability and the entity/geometry builders; the app owns selection, viewport input and the single transaction.
- Allocations are `try_reserve`-checked and the transaction is admitted by the coordinator `Limits` before commit.
- The same selection and points produce the same object count, placement and Undo entry.

## Source decision

`ADD`. QymCAD split/trim tools are sketch/part workbench states; the portable curve join/split/trim kernels are OM9's curve_ops, already measured under OM9-L2-02-04. See `source_crosswalk.json`.

## Deliberate limits

- Cutters must share the target placement; several cutters are accumulated until Enter.
- The default build uses OCCT exact curve/face intersections for a BRep cutter; the no-default developer build refuses it with a typed error instead of approximating from display triangles.
- Trim removes the interval nearest the pick and keeps the complement; the pick itself is a point, not a second object.
- Join orders the selection by endpoint continuity; it does not bridge a gap.

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

- focused: `cargo test -p om9-app --no-default-features --test top11 m03_03_08_004 -- --nocapture`
- regression: `cargo test -p om9-app --no-default-features --lib --test top11 -j 2` (195 passed, 0 failed)
- core regression: `cargo test -p om9-core --test top11 -j 2` (10 passed, 0 failed)
- clippy: `cargo clippy -p om9-core -p om9-app --no-default-features --lib --test top11 -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `4fddfbd2b21638a7c13518b82d00015c91d896fc9844bc2ff6b57f083a417f3b`.
BRep mirror log SHA-256: `331e7fb35a78a3f7b3a10e86bc97bf2cc87bf0f32df50d39f6f6a9dbef242a68`.
Integration-stress log SHA-256: `e54c04048afe3630e4c38eb82146079c730f024848c9fbb2930b5cf2f6b0edb0`.
