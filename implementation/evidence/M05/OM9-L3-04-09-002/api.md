# Gem Reference Mesh And B-Rep Assets (`OM9-L3-04-09-002`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `GemLibrary::canonical` | one asset per cut x shape with an authored size ladder; procedural only |
| `GemLibrary::{asset,assets,asset_for_cut,nearest_size}` | library lookups |
| `build_mesh(&GemSpec)` | closed triangle mesh, outward orientation (edge-propagation), bbox matches the spec |
| `GemMesh` | `mesh`, `closed`, `volume_mm3`, `extents_mm`; oracle from `ring_builder::inspect_solid` |
| `om9_kernel::brep::{from_triangles,from_mesh}` | one closed OCCT solid from an indexed mesh; an open or non-manifold boundary is refused instead of sealed |
| `om9_kernel::gem::gem_brep` | the gem adapter: `GemMesh` -> `Brep` with the same volume and gem-sized display deflection |
| `GemSession::brep` / `BRep=Yes|No` | stores `Geometry::Brep` when the kernel feature is on (default), the portable mesh otherwise |
| `GemSession::preview` / `Controller::gem_world_preview` | the session mesh (including the positioner scale) and its world placement, never written to the document |
| `ui::gem::gem_preview_segments` | pure world->screen projection of the preview edges for the viewport overlay |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object. The viewport paints that mesh as a wireframe overlay while a gem tool is open, including `Move`, `Rotate`, `Scale` and `Origin`, so Apply commits exactly what was shown.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- When the OCCT kernel feature is on, Apply stores `Geometry::Brep` (one closed solid sewn from the authored facets); `BRep=No` stores `Geometry::Mesh`. The `GemRecord` payload is identical in both cases, so replacing a gem never changes its `GemId`.
- Property edits and positioner scale keep the body kind of the existing entity: a B-Rep gem is re-sewn, a mesh gem stays a mesh; a shared definition member is built in the target's kind.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); an open or non-manifold mesh is refused by the kernel instead of being sealed into a fake solid, and no silent mesh fallback hides a failed conversion.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Native exception

`crates/om9-kernel/src/brep_bridge.cpp:om9_occt_brep_from_triangles` is a selected OCCT adapter: it owns only `BRepBuilderAPI_MakePolygon/MakeFace/Sewing/MakeSolid`, `ShapeFix_Shape`, the closed-shell/volume checks and `om9_emit_brep_snapshot`. All validation, budgets, geometry input and error mapping stay in Rust (`om9-kernel/src/brep.rs`, `om9-kernel/src/gem.rs`).

## Verification

- Positive: the canonical library is authored/procedural and every asset is a closed solid; every cut family converts to one valid B-Rep with the same volume, the committed body is that B-Rep, and the open tool shows a world-space preview before Apply
- Negative: no Matrix asset is copied; zero-area profiles and open meshes are refused; kernel sewing refuses open/multi-shell input; `BRep=Yes` never falls back to a mesh body silently
- Gate: `AUTO_ONLY` (automated evidence only).
