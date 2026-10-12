# RCORE-01 — Reconciled remaining acceptance scope

This is an evidence audit, not a new completion claim. Original requirements
remain in `specs/00-rhino-core/01-object-model-kernel.md`. Historical unchecked
rows describe the evidence available at their dates; later accepted slices must
be considered before treating those rows as current failures.

T01–T11 have fixture/evidence slices in the accumulated implementation records.
They do not prove every family, parameter interval, option, ownership graph or
enabled consumer. Group/link/array/world-once, native discrete snapshots,
topology/solid/cavity validation, prepared CAD/discrete results, registry routing
and global transaction notifications already have substantial evidence.

The identity/destructor requirement changed only at the explicitly approved
acceptance boundary: exact identity remains required for reversible rejection;
physical destruction after acceptance reports disposal separately. The old
strict-destructor witnesses remain unchanged historical RED evidence.

| Remaining requirement | Existing code and fixtures to reuse | Evidence boundary |
|---|---|---|
| Conversion and correspondence over the accepted domain: deviation/locus, parameter maps, edge3D↔pcurve and native reversal | [CoreSupportSnapshot](../../Gui/CoreSupportSnapshot.cpp), [host parameter fixture](../../tests/rhino_core_host_parameter_dependency_smoke.FCMacro), [edge fixture](../../tests/rhino_core_edge_geometry_smoke.FCMacro) | Sampled analytic/derived curves, hole/seam and corrected D3 are proven slices; samples alone do not certify full-domain correspondence. |
| Registry, GeometryResult and diagnostic coverage for every enabled consumer | [CoreEditResult](../../Gui/CoreEditResult.cpp), [EditGeometry](../../Gui/EditGeometry.cpp), [CoreRebuild](../../Gui/CoreRebuild.cpp), [Edit fixture](../../tests/rhino_core_edit_result_smoke.FCMacro) | Curve/Solid/Surface/CAD Edit and prepared discrete contracts exist. Mesh/special Edit, other History/Rebuild/Cage paths and operation/input identity still need complete consumer mapping. |
| Identity, ownership and reference completeness | [CoreObjects](../../Gui/CoreObjects.cpp), [CoreDefinitionSnapshot](../../Gui/CoreDefinitionSnapshot.cpp), [reference fixture](../../tests/rhino_core_reference_context_smoke.FCMacro) | Direct/group/link/array snapshots apply world transforms once. Durable subelement remapping, overlapping selections, cycles and all attribute/channel policies remain separate acceptance obligations. |
| Independent read/edit/retain/export matrix by representation | [CoreThreeDm](../../Gui/CoreThreeDm.cpp), [preservation fixture](../../tests/three_dm_preservation_smoke.FCMacro), [polycurve fixture](../../tests/rhino_core_polycurve_smoke.FCMacro), [TextDot fixture](../../tests/three_dm_text_dot_smoke.FCMacro) | Existing retained archives, TextDot, partitions and mesh/cloud preparation do not establish complete extrusion/annotation/nested-polycurve/userdata/channel preservation. |
| Aggregate and peak allocation, raw decoding and cancellation | [native replay admission](../../../../src/App/private/PropertyReplay.h), [snapshot budget fixture](../../tests/rhino_core_replay_snapshot_budget_smoke.FCMacro), [native input tree fixture](../../tests/rhino_core_native_input_tree_smoke.FCMacro) | Constructor/history/name admission is bounded before copies. Simultaneous preparation copies, opaque values, external allocators and every traversal/worker still need scoped limits/evidence. |
| Publication, lifetime and cache beyond the verified cohorts | [CoreCurvePublication](../../Gui/CoreCurvePublication.cpp), [delayed fixture](../../tests/rhino_core_delayed_publication_smoke.FCMacro), [acceptance fixture](../../tests/rhino_core_replay_acceptance_boundary_smoke.FCMacro), [member lifetime fixture](../../tests/rhino_core_member_finalizer_lifetime_smoke.FCMacro) | Circle and delayed Rebuild have specific proofs. Remaining source/global participants, status/touch/cache, cancellation/document replacement, display/export invalidation and actual mouse/render coverage must be mapped individually. |

Latest completed checkpoint:
[native notification lock](2026-10-12-rhino-core-notification-lock.md),
3375/3375 assertions through 35 clean matching hosts. The member lifetime fixture
is a continuing RED/GREEN investigation, not included in that checkpoint.

Next work: native derived member/wrapper lifetime, aggregate preparation limits,
then mesh/special Edit GeometryResult integration using prepared discrete data.
No new line/polyline geometry implementation is implied by this audit.
