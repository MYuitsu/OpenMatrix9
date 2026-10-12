# RCORE-01 derived supporting geometry implementation plan

> Continue inline under the user's existing full RCORE-01 authorization. Use
> superpowers:executing-plans; do not create a new checkout or rebuild FreeCAD.

**Goal:** Preserve native offset curves/surfaces, linear extrusion and revolution
with their complete supporting basis chains in Rust-owned inspection snapshots.

**Architecture:** Safe Rust validates a bounded root-to-leaf chain and owns its
typed analytic/NURBS/Bezier leaf. C++ only walks native handles, reads parameters
and reuses checked basis adapters. Inner trimmed bases remain explicit nodes;
outer topological restriction/location/orientation remain separate observations.
No approximation, native conversion or solver is added.

**Tech stack:** dependency-free Rust, bounded existing JSON writer, OCCT/Qt bridge.
**Spec:** OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md,
adapter requirements 1/2/5/7 and C03; original chapter scope remains unchanged.

## Constraints and review focus

- Reuse H:/FreeCAD-src/build/relWithDebInfo, standalone OpenMatrix9Gui target.
- At most 16 wrapper nodes; existing degree25/poles8192/2MiB basis/8MiB JSON limits.
- Validate node category/dimension/finite parameters before cloning leaf buffers.
- Preserve signed offset, native unit direction and revolution origin; no normalization.
- Native infinite domains use explicit infinity flags, never finite sentinel values.
- Invalid envelopes, category transitions, unit vectors and output capacities reject atomically.
- Native wrappers may retain inner trims or simplify them; snapshot actual runtime basis.
- Sample oracles do not certify regularity, global parameter correspondence or solid validity.

## Task 1: owned chain and ABI

Files: new rust/src/derived_support.rs, derived_support_ffi.rs,
rust/tests/derived_support.rs; modify support_snapshot_ffi.rs and lib.rs.
Interfaces: DerivedNode reprC104; NativeSupportTree reprC424; Tree::capture;
om9_support_tree_json query/copy; om9_support_tree_depth pre-extraction gate.
Nodes 1 offset_curve,2 offset_surface,3 linear_extrusion,4 revolution,
5 trimmed_curve,6 trimmed_surface. Leaf1 analytic,2/3 NURBS,4/5 Bezier.

- [x] RED: owned nested basis/trim data, invalid chains/domains, bounded ABI output.
- [x] Implement typed chain with one owned leaf, fixed-key bounded serialization.
- [x] GREEN: full cargo test, including exact ownership and malformed-buffer gates.

## Task 2: native reader and real host

Files: CoreSupportSnapshot.cpp, CoreGeometryContract.cpp, geometry_contract.rs/
geometry_contract_ffi.rs; tests/rhino_core_derived_support_smoke.FCMacro.
Interfaces: existing curveSupportSnapshot/faceSupportSnapshot; explicit form4 derived.

- [x] RED: actual offset/extrusion/revolution descriptors lack owned data/form.
- [x] Walk native basis handles using a fixed array; read leaf through existing adapters.
- [x] Independent sampled formulas: tangent cross reference direction for offsets,
  surface partial cross product normal, extrusion and Rodrigues rotation.
- [x] Native trim/local placement/orientation/source/history/lifetime checks and
  UV offset evidence; regress analytic, Bezier, NURBS and topology snapshots.
- [x] Build plugin only; sequential host processes, verify reports/module/hashes/exit0.
- [x] Fresh read-only review; synchronize original chapter audit, progress and manifest.

Remaining full RCORE-01 gaps (world/attributes, edge/UV correspondence, solid
certification, reports, raw allocation and uniform lifecycle/commit gates) are
still mandatory; this plan does not redefine chapter completion.

Accepted evidence:91 derived +687 host regression =778 checks;610 full Rust tests;
all selected processes exit0. See docs/validation/2026-10-10-rhino-core-derived-support.md. Full chapter remains in progress.
