# OM9-L3-01-09-008 — Surface From Edge Curves API and frozen fixture

- **Owner:** `OM9-L2-01-09` (M03 geometry creation operations)
- **Goal on record:** patch surface through 2-4 boundaries
- **Crate / adapter:** `crates/om9-kernel/src/creation.rs` over `crates/om9-kernel/src/creation_bridge.cpp`
- **Units:** millimetres and radians; tolerances are absolute millimetres
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn surface_from_edges(curves: &[Vec<Point3>], tolerance_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<SurfacePatch, CreationError>`
- `pub struct SurfacePatch { pub brep: Brep, pub max_deviation_mm: f64 }`

`CreationError` is the typed refusal contract: `Invalid(String)` for an inadmissible input,
`Budget(String)` for a measured budget, `Kernel(String)` for an adapter failure after admission,
and `StaleHandle` for a released body handle.

## Input rules (rejected with `CreationError::Invalid`)

- two to four boundary curves, each with at least two distinct points
- each curve end must meet the next curve start within the configured tolerance
- the measured OCCT filling deviation must stay within the tolerance, else the patch is refused

## Behaviour

BRepOffsetAPI_MakeFilling builds the patch through the boundary edges; G0Error() is returned as the measured deviation and the operation refuses a patch whose deviation exceeds the tolerance.

## Positive fixture and measured oracle

- four unit-square edges: area 1.0 mm², one face, deviation 0

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `surface_area_mm2` | 1.000000000000 | 1e-6 |
| `surface_deviation_mm` | 0.000000000000 | <= 1e-6 |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_09_008`.

## Negative fixture

- a boundary gap above tolerance is refused
- one curve is refused

A refusal returns before a `Brep` exists, so no body is published and no document/Undo state is
partially written. A `CreationSession` keeps `live()` unchanged when a build is refused.

## Ownership, budget and cancellation

- Every call is one synchronous complete operation; no `TopoDS_Shape` or OCCT handle crosses into safe Rust.
- The snapshot sink copies bounded arrays; source, points, nodes and bytes are checked against `Limits`.
- `CreationSession` owns bodies in a generational `HandleStore`, so a released handle can never alias a
  replacement body.
- `KernelError::Cancelled`/`BudgetExceeded` map to the typed creation error; the document transaction
  owner opens and commits only after a successful `Result`.

## Source decision

`ADD`: the construction adapter is new code against pinned OCCT 8.0.1. QymCAD `shape/build.rs`
is a thin C ABI wrapper and its role naming is not reused; see `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-kernel/src/creation.rs` | `c0918492dca0b8501935993ca88749abb7e6b30155f092de5ebd5c8ed4f6125d` |
| `crates/om9-kernel/src/creation_bridge.cpp` | `7b76f4cd2ec9a8980d60fbfc10e6636af89ab028e0e7a6bd162d452ed047c087` |
| `crates/om9-kernel/src/brep_snapshot.hxx` | `b50814193b7b3570a083b2f23f357834ef3f2b6c3e43f5ffcfd955ed7a674e22` |
| `crates/om9-kernel/src/brep_bridge.cpp` | `ecbd075b984ad191e6e4cc05266d9bfb9b9cf5417fd215d8f97fc69406c67c45` |
| `crates/om9-kernel/src/brep.rs` | `033ce1d32c96ac80c576385b6e0ef9cac873596cee31bdd86196e817a00a3485` |
| `crates/om9-kernel/src/lib.rs` | `d34a22f00c00bc0711b2a013dbbbaf8cdfff717dec6a9176e547665fa1f842fe` |
| `crates/om9-kernel/build.rs` | `553749365b799c942ee3bc25ec134b934739e7b9d329cb2f50c89a9210033197` |
| `crates/om9-kernel/tests/l2_01_09.rs` | `7957036d2766ee1090d84fd0f3039d52d4198034cf2898755a82933ebdd9dc43` |

## Receipts

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_008 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_09`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
