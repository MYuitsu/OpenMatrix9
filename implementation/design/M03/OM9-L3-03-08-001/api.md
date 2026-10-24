# OM9-L3-03-08-001 — Top11 Duplicate Action API and frozen fixture

- **Owner:** `OM9-L2-03-08` (07-febe-matrix-top11-and-shortcut-actions), milestone M03
- **Goal on record:** Duplicate button invokes common copy service
- **Crate / host:** `om9-core::top11` + `om9-app` toolbar/controller adapter
- **Units:** millimetres and degrees
- **Evidence date:** 2026-10-23

## Scope delivered

Duplicate (Copy): In Place, plus the manual base-point flow with repeated destinations until Enter. The Vertical, From Last Point, Use Last Distance and Use Last Direction options are typed while the prompt runs or supplied as Copy options. Each placement is one transaction/Undo.

## Frozen API

- `om9_core::top11::Top11Action::Duplicate`
- `om9_core::top11::duplicate_entities`
- `om9_app::session::Controller::top11_invoke`
- `om9_app::session::Controller::top11_submit`

## Input rules

- The action reads the current document selection and the recorded ObjectIds; every referenced object must still exist.
- Points, fractions, radii and matrices are finite before use; a refusal is a typed `Error`.
- Every document change is one `TransactionRequest`; a refused action publishes nothing.

## Behaviour and measured oracle

Positive fixture: In Place: one line -> 2 objects and Undo/Redo; base flow: 2 copies at x=5 and x=10; Vertical projects (3,4,5) to (0,0,5); Use Last Distance turns a 5 mm y move into 10 mm; Use Last Direction turns it into 5 mm along x; From Last Point continues from the last destination.

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `duplicate_vertical_origin` | [0.0, 0.0, 5.0] | 1e-9 |
| `duplicate_base_copies` | 2.000000000000 | exact integer |
| `duplicate_total_objects` | 3.000000000000 | exact integer |
| `duplicate_undo_redo_passed` | true | boolean assertion |
| `duplicate_last_distance_y` | 10.000000000000 | 1e-9 |
| `duplicate_last_direction_x` | 5.000000000000 | 1e-9 |

Focused run:
`cargo test -p om9-app --no-default-features --test top11 m03_03_08_001 -- --nocapture`.

## Negative fixture

- duplicate with an empty selection
- duplicate while a command is already running
- a destination equal to the base point

A refused action returns before a mutation is prepared; the document, selection and Undo journal stay unchanged.

## Ownership, budgets and determinism

- Safe Rust owns the action catalogue, availability and the entity/geometry builders; the app owns selection, viewport input and the single transaction.
- Allocations are `try_reserve`-checked and the transaction is admitted by the coordinator `Limits` before commit.
- The same selection and points produce the same object count, placement and Undo entry.

## Source decision

`ADD`. QymCAD has no Top 11 duplicate action; its copy/pattern paths are document-model Id operations. OM9 uses the existing Entity/Mutation contract. See `source_crosswalk.json`.

## Deliberate limits

- Vertical projects onto the active CPlane normal; there is no per-copy direction lock beyond the four documented options.
- Each placed copy is its own transaction/Undo entry.
- Copies keep the source layer/attributes and add the placement delta.

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

- focused: `cargo test -p om9-app --no-default-features --test top11 m03_03_08_001 -- --nocapture`
- regression: `cargo test -p om9-app --no-default-features --lib --test top11 -j 2` (195 passed, 0 failed)
- core regression: `cargo test -p om9-core --test top11 -j 2` (10 passed, 0 failed)
- clippy: `cargo clippy -p om9-core -p om9-app --no-default-features --lib --test top11 -j 2 -- -D warnings`
- format: `cargo fmt --all -- --check`

Focused log SHA-256: `551f4482ece8eb10e014a92741f5c75f0acadfefb2f8ed12f173dfe8c947fa08`.
