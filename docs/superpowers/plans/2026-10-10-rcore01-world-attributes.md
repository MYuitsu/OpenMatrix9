# RCORE-01 world frames and source attributes

Continue inline under the existing full RCORE-01 authorization. Use existing
shared FreeCAD SDK and plugin cache; no new FreeCAD build or geometry solver.

Goal: prove support-local to world composition once for placed Part features,
nested GeoFeature groups and nested direct App::Link definitions, while retaining
independent owned source identity and native appearance attributes. The original
chapter also requires remaining group/subelement instance references, arbitrary
attributes/userdata and other gates; these remain mandatory if not evidenced.

Native Shape already contains the feature Placement. Rust computes
`parent_world * native_path * inverse(definition_placement) * native_location`.
For direct features native_path equals definition_placement. For links reuse
getLinkedObject's native accumulated affine path (no Shape bake), after bounded
direct-link preflight. Native source Placement is rigid; affine instance paths
may mirror or scale nonuniformly. Preserve orientation sign without repairing
topology. UV pcurves remain face_uv; each face's support has its own world frame.

Source attributes: document UID, object/type/label/Label2, visibility/selectability,
native view-provider type and the full ordered ShapeAppearance Material fields
(four RGBA colors, shininess/transparency, uuid/image/imagePath). Rust owns UTF-8
strings/materials and bounded JSON. Other app properties and Rhino userdata are
explicitly uninspected, never labeled preserved. Limit4096 materials,64KiB/string,
1MiB aggregate owned input; output uses existing bounded8MiB atomic query/copy.

Files: rust/src/geometry_frame.rs and geometry_frame_ffi.rs; source_attributes.rs
and source_attributes_ffi.rs; CoreSourceAttributes.cpp/.h; CoreGeometryContract
reader; geometry_contract_ffi instance tag10/legacy unknown9; native fixture and
Rust integration tests. Reuse native object registry/source before-after guards.

- [x] RED: frame/attributes Rust APIs absent, native local_to_world/attributes null.
- [x] Pure Rust composition, rigid inverse, finite/affine/singular checks; literal
  samples for embedded placement, rotations, mirror/nonuniform scale and errors.
- [x] Rust-owned attribute data/JSON and bounded initialized-buffer ABI with UTF-8,
  capacities, flag/value, material range and atomic failure checks.
- [x] Native reader: direct/parent/support frames, all common faces, typed instance
  wrapping a definition descriptor, guarded link chain and independent attributes.
- [x] Real host proves points once against native world placement, untouched source/
  Undo/Redo, nested mixed LinkTransform cases, instance A/B separation and lifetime.
- [x] Full Rust/build/current-plugin regressions; fresh review; original audit,
  README/spec/ledger and manifest synchronize actual evidence without closing gaps.

Ruling: appearance needs its own inspected flag (bit16), in addition to presence
of a view provider (bit8). A provider without the ShapeAppearance property emits
null, rather than an empty list that would falsely imply inspected absence.
Rust RED observed in world-attributes-unknown-appearance-red-20261010.log.

Review finding P2 reproduced: accounting ABI headers192/120 undercounted retained
Rust String/Vec headers. The budget now uses size_of OwnedMaterial/SourceAttributes
(216/176 on this host), plus UTF-8 contents, before clone. It excludes allocator
overhead and temporary/native staging, and is not a process-wide memory quota.
Rust RED: world-attributes-owned-budget-red-20261010.log.

Accepted evidence:78 new+778 regression=856 host checks;620 full Rust tests;
all eight matching selected processes exit0. See docs/validation/2026-10-10-rhino-core-world-attributes.md. Full chapter remains in progress.
