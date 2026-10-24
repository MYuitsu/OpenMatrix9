# OM9-L3-01-09-001 — Analytic Solid Primitive Factories API and frozen fixture

- **Owner:** `OM9-L2-01-09` (M03 geometry creation operations)
- **Goal on record:** sphere, cylinder, box, cone, torus BRep shapes
- **Crate / adapter:** `crates/om9-kernel/src/creation.rs` over `crates/om9-kernel/src/creation_bridge.cpp`
- **Units:** millimetres and radians; tolerances are absolute millimetres
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `pub fn primitive(primitive: Primitive, deflection_mm: f64, limits: &Limits) -> Result<Brep, CreationError>`
- `pub enum Primitive { Box, Sphere, Cylinder, Cone, Torus, Ellipsoid, Pyramid }`

`CreationError` is the typed refusal contract: `Invalid(String)` for an inadmissible input,
`Budget(String)` for a measured budget, `Kernel(String)` for an adapter failure after admission,
and `StaleHandle` for a released body handle.

## Input rules (rejected with `CreationError::Invalid`)

- every size must be finite and positive (cone top radius may be zero)
- torus tube radius must be smaller than the ring radius
- pyramid needs at least three sides; base radius and height positive
- deflection must be finite and positive

## Behaviour

Each variant maps to one OCCT factory (BRepPrimAPI_MakeBox/Sphere/Cylinder/Cone/Torus, a general transform of the unit sphere for the ellipsoid, a ruled loft from a base wire to an apex for the pyramid). The adapter snapshots the resulting exact B-Rep; validity is checked before the snapshot.

## Positive fixture and measured oracle

- Box(10,20,30): volume 6000 mm³, bounds 10×20×30
- Sphere(r=5): volume 4/3π·125
- Cylinder(r=5,h=10), Cone(r1=5,r2=0,h=10), Torus(R=10,r=2), Ellipsoid(3,4,5), Pyramid(4 sides,R=10,h=10)
- the canonical source reopens to the same measured solid

| measured key | observed on this checkout | admitted tolerance |
|---|---:|---|
| `box_volume_mm3` | 6000.000000000000 | 1e-6 |
| `sphere_volume_mm3` | 523.598775598299 | 1e-6 |
| `cylinder_volume_mm3` | 785.398163397449 | 1e-6 |
| `cone_volume_mm3` | 261.799387799150 | 1e-6 |
| `torus_volume_mm3` | 789.568352087151 | 1e-6 |
| `ellipsoid_volume_mm3` | 251.217495107596 | 0.2 (general-transform BSpline approximation, measured error < 0.05%) |
| `pyramid_volume_mm3` | 666.666666666666 | 1e-5 |

The measured line is produced by
`cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_receipt -- --exact --nocapture`
and the same values are asserted by the focused test `m03_01_09_001`.

## Negative fixture

- radius 0 and NaN are refused
- negative cylinder height is refused
- zero box size is refused
- torus tube >= ring is refused
- pyramid with two sides is refused

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

- focused: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_09 m03_01_09_001 -- --exact`
- regression: `cargo test -p om9-kernel --features 3dm-brep --test l2_01_09`
- crate suite: `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-kernel --features 3dm-brep --all-targets -- -D warnings`
- format: `cargo fmt --all -- --check`
