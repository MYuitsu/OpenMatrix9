# OM9-L3-01-10-006 — Fillet And Chamfer B-Rep Edges API and frozen fixture

- **Owner:** `OM9-L2-01-10` (M03 boolean / fillet / shell operations)
- **Goal on record:** fillet/chamfer selected edges of a solid
- **Crate / adapter:** `crates/om9-kernel/src/editing.rs` over `crates/om9-kernel/src/editing_bridge.cpp`
- **Units:** millimetres and radians; tolerances are absolute millimetres
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn fillet(solid: &Brep, edges: &[usize], radius_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError>`
- `pub fn chamfer(solid: &Brep, edges: &[usize], distance_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError>`
- `pub fn edge_endpoints(brep: &Brep, edge_index: usize) -> Result<(Point3, Point3), EditingError>`

`EditingError` is the same typed contract as the creation owner: `Invalid(String)` for an
inadmissible input, `Budget(String)` for a measured budget, `Kernel(String)` for an OCCT failure
after admission, and `StaleHandle` is unused here.

## Input rules (rejected with `CreationError::Invalid`)

- the input must contain a closed solid
- value must be finite and positive
- each edge index must be inside the published edge map
- a kernel failure (oversized radius) must not mutate the input body

## Behaviour

BRepFilletAPI_MakeFillet / MakeChamfer apply the operation to every selected edge in one build; the result must pass BRepCheck before it is snapshotted.

## Positive fixture and measured oracle

- 10³ box, one vertical edge filleted r=1: 997.853981633974 mm³, closed, 0 naked edges
- 10³ box, one vertical edge chamfered 1 mm: 995 mm³, closed

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `fillet_one_edge_volume_mm3` | 997.853981633974 | 1e-5 |
| `chamfer_one_edge_volume_mm3` | 995.000000000000 | 1e-5 |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_10_006`.

## Negative fixture

- an oversized radius fails in the kernel
- r=0 is refused before the kernel
- an unknown edge index is refused
- an open shell is refused

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

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_006 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
