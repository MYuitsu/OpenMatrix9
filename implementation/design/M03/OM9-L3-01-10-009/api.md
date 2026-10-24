# OM9-L3-01-10-009 — Geometry Health And Naked Edge Query API and frozen fixture

- **Owner:** `OM9-L2-01-10` (M03 boolean / fillet / shell operations)
- **Goal on record:** exact B-Rep health and naked edge query
- **Crate / adapter:** `crates/om9-kernel/src/editing.rs` over `crates/om9-kernel/src/editing_bridge.cpp`
- **Units:** millimetres and radians; tolerances are absolute millimetres
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn geometry_health(brep: &Brep, limits: &Limits) -> Result<GeometryHealth, EditingError>`
- `pub struct GeometryHealth { valid, closed, naked_edges, faces, edges, vertices, solids, shells, source_sha256 }`

`EditingError` is the same typed contract as the creation owner: `Invalid(String)` for an
inadmissible input, `Budget(String)` for a measured budget, `Kernel(String)` for an OCCT failure
after admission, and `StaleHandle` is unused here.

## Input rules (rejected with `CreationError::Invalid`)

- health computes from the canonical B-Rep source, never from the tessellation
- naked edges are free boundary edges with fewer than two face occurrences (seam-aware)

## Behaviour

health reports validity, closedness, naked edge count, topology counts and the source digest; the count is exact topology, so mesh boundaries never masquerade as B-Rep edges.

## Positive fixture and measured oracle

- closed 10³ box: valid, closed, 0 naked edges, 6 faces, 1 solid
- open shell (top removed): 4 naked edges, 5 faces, 0 solids
- removing one face adds exactly its 4 boundary edges

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `closed_naked_edges` | 0.000000000000 | exact |
| `open_naked_edges` | 4.000000000000 | exact |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_10_009`.

## Negative fixture

- a tampered source is refused instead of reporting health

A refusal returns before a `Brep` exists; an input body is an immutable value and is never mutated
by a failed operation.

## Ownership, selection and budgets

- Every call is one synchronous complete operation; no `TopoDS_Shape` or OCCT handle crosses into safe Rust.
- Faces and edges are selected by index into the canonical OCCT map order, which is also the order of
  the published `faces`/`edges` in `Brep::display`. `edge_endpoints` is a display-derived selection helper.
- The snapshot sink copies bounded arrays; source, points, nodes and bytes are checked against `Limits`.
- Boolean empty results are `BooleanOutcome::Empty`, not a zero-face body.

## Source decision

`ADD`: the editing adapter is new code against pinned OCCT 8.0.1. QymCAD's boolean/blend/shell asks
are a different typed façade over its own C ABI; no code is reused. See `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-kernel/src/editing.rs` | `9b93d9eff62bfdb583d5682d7da17d59ba1315fd8ef968be405e30332a9dad7c` |
| `crates/om9-kernel/src/editing_bridge.cpp` | `4d4929e3abd6b9617abd6cd593dd41bba897b3193a2be168cea197165e471a15` |
| `crates/om9-kernel/src/brep_snapshot.hxx` | `b50814193b7b3570a083b2f23f357834ef3f2b6c3e43f5ffcfd955ed7a674e22` |
| `crates/om9-kernel/src/brep_bridge.cpp` | `7bbb18de9f292ec227a5679b1349191dbf99271bab6bf0b95ed2bd43b1199fab` |
| `crates/om9-kernel/src/creation.rs` | `ce667d2def700b370fa2a20307fbb63039959b250d37c6cff299aa23410882a3` |
| `crates/om9-kernel/src/brep.rs` | `1700c7e0271399b8dd2f3a50d776f3680e20cb34103acc1ec8fbc807b63e19d8` |
| `crates/om9-kernel/src/lib.rs` | `fe8e7ed68cb107c3242c47748cf242500aa8f3a21370a184e31c4b5b075ea0ee` |
| `crates/om9-kernel/build.rs` | `e73df94ed7ad1d336ce5cb1c421b9192cdb110f9a28bcaaf87dd7535c7066c3a` |
| `crates/om9-kernel/tests/l2_01_10.rs` | `e6260dd45c8c41c37d5ef654eb6da7b424f0deea6e052fbf13f796b2155c5e03` |

## Receipts

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_009 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
