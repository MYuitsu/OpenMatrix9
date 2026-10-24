# OM9-L3-02-05-008 — Surface Offset Fillet Chamfer API and frozen fixture

- **Owner:** `OM9-L2-02-05` (M03 05-febe-freeform-surface-modeling)
- **Goal on record:** surface thickness and rounding via shared kernel
- **Owner crates:** `crates/om9-core` (portable NURBS surface services), `crates/om9-kernel` (exact OCCT B-Rep)
- **Units:** canonical millimetres and radians
- **Evidence date:** 2026-10-22

## Frozen Rust API

- `om9_core::surface_ops::{offset_surface, normal_extrude}`
- `om9_kernel::editing::{offset_shape, fillet, chamfer, thicken}`

## Input rules (typed refusal, no guessed value)

- surface offset samples the exact normal and rejects a zero/non-finite distance
- B-Rep offset/fillet/chamfer stay in the shared OCCT kernel
- a zero thickness or an impossible radius is refused

## Behaviour

The portable offset samples the surface normal and refits the offset grid. The exact B-Rep offset/fillet/chamfer is delegated to the shared kernel, which validates closedness and topology after the edit.

## Positive fixture and measured oracle

- a plane offset by 2.5 mm has z=2.5 everywhere
- a cylinder offset inward by 1 mm has radius 4
- a 10x20x30 box offsets to volume 6733.33 mm3 and stays closed
- fillet/chamfer add a face and leave zero naked edges

| measured key | observed on this checkout | admitted tolerance |
|---|---|---|
| `plane_offset_z` | `2.5` | `exact` |
| `cylinder_offset_radius_mm` | `4.0` | `exact` |
| `box_offset_volume_mm3` | `6733.333333333332` | `exact` |
| `fillet_faces` | `7` | `exact` |
| `fillet_naked_edges` | `0` | `exact` |

## Negative fixture

- zero surface offset distance is refused
- zero thicken/offset and an oversized fillet are refused

## Ownership, budget and cancellation

- Safe Rust owns the NURBS control structure, UV loops, validation and measured deviation; C++ is limited to the OCCT construction/edit call and exception containment.
- Every constructor returns a validated surface or a typed error; a refused input publishes nothing.
- Grid/sample counts, degree bounds and loop membership are checked before allocation; no partial result is returned.

## Source decision

`ADD`: QymCAD exposes OCCT shape C ABI calls and profile naming only, not a portable NURBS surface workflow. The local module is new Rust code; see `source_crosswalk.json`.

## Artefact hashes (sha256, this checkout)

| file | sha256 |
|---|---|
| `crates/om9-core/src/surface_ops.rs` | `1c9a8cf3f4fde9da026a7c032c6f3c06976b593390c204bd4516250a8750b244` |
| `crates/om9-core/src/surface_kernel.rs` | `f6ee1938df7df299ba25e639c178c80c6e96c789c5e902af5b3406ace383ccc0` |
| `crates/om9-core/src/curve_build.rs` | `2b3194a5172953293d99d1f2086cde842b75ced4e000a65e69469367b9887c24` |
| `crates/om9-core/src/rhino_semantics.rs` | `9d98a59283cd252e7e0bfddd06d785e59907b3c32b5f904e797da950e49ea4cb` |
| `crates/om9-kernel/src/creation.rs` | `7f2e2dc0849d9497d9b202bdd23332989091ee2e8224a04ebfc8be1beb2a5e0f` |
| `crates/om9-kernel/src/editing.rs` | `9b93d9eff62bfdb583d5682d7da17d59ba1315fd8ef968be405e30332a9dad7c` |
| `crates/om9-core/tests/surface_ops.rs` | `d8ba5f44c8f8650670fb69f543f2a233ef2451bab9c1a20891310db2550405b5` |
| `crates/om9-kernel/tests/l2_02_05.rs` | `4a4543c0e07da00a96d2ec12eeffa4cd5ff8e1f512f2c33539ba6833cd581150` |
| `crates/om9-core/src/lib.rs` | `00e29902e540594acb83e75868329a30ae3a2c6f1cc16b3932b4024dd9b7ca15` |

## Receipts

- focused: `cargo test -p om9-core --test surface_ops <L3 filter> -- --nocapture` (logs in the evidence folder)
- regression: `cargo test -p om9-core --test surface_ops`; `cargo test -p om9-kernel --features 3dm-brep --test l2_02_05`; `cargo test -p om9-core --lib`; `cargo test -p om9-kernel --features 3dm-brep`
- clippy: `cargo clippy -p om9-core --test surface_ops -- -D warnings`; `cargo clippy -p om9-kernel --features 3dm-brep --test l2_02_05 -- -D warnings`
- format: `rustfmt --edition 2024 --check crates/om9-core/src/surface_ops.rs crates/om9-core/src/lib.rs crates/om9-core/tests/surface_ops.rs crates/om9-kernel/tests/l2_02_05.rs`
- public source: `python tools/public_source_audit.py` (exit 0)

## Deliberate limits

- Portable surface offset is a sampled/refit approximation; the exact offset remains the OCCT B-Rep service.
