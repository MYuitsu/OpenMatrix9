# RCORE-01 — Owned native Curve admission foundation, 2026-10-11T21:50:25+07:00

**Status: in_progress. Rust implementation is verified; native integration is pending.**
All original RCORE-01 requirements remain required. This checkpoint does not close
Task2, Task3, any full Curve feature or the chapter.

The existing six constructors/solvers are reused. `accept_native_json` admits a
bounded owned JSON packet before publishing a private candidate session. Original
native text, numeric values, source identity with full-u64 stamps, complete source
snapshots/attributes/units, and event/range associations are retained as values;
no host pointer enters Rust. Circle checkpoint Undo releases the removed source
branch. Identical snapshots are interned within a batch and across retained events.

Five fresh read-only review findings were reproduced: **8 passed/5 failed** before
fixes, then **13/13 passed**. A unit-scale mismatch was separately reproduced:
**15 passed/1 failed →16/16 passed**. The revision getter ABI initially failed to
link; after implementation, the extern-C admission/ownership test passed **1/1**.

| Review gap | Implemented behavior |
|---|---|
| Unrelated discrete samples accepted as source | Ordered indices or whole-source order bind selected values to parent × native location × complete XYZ; transformed values are checked numerically |
| Native point source missing/unrelated or empty range | Exactly one selected point and its accepted-input range must match the original point, before solver normalization |
| Missing/singular CAD root frame or invalid orientation | Captured root location must be finite, affine and invertible; orientation must be a native enum value |
| Same-generation stale non-Circle capture and Undo ABA | Monotonic admission revision covers every constructor, accepted state/configuration change and Circle Undo, separately from restorable native resource context |
| Native Polyline ranges incorrectly limited to1024 | Ranges follow constructor capacity; Polyline point1025 is accepted while FitPoints retains1024 limit |

Additional coverage verifies actual operation/source diagnostics, immutable
rejection, full-u64 revisions, point-cloud subset order with a translated frame,
Circle branch replacement, and source snapshot interning. The full Rust suite
completed **871 passed/0 failed/0 ignored**, actual cargo exit0. All238 frozen
Rust production/test/Cargo inputs matched after the run and at recording time.
This is Rust test-source matching, not a plugin build certificate. Existing cache
hard-link/path warnings remain recorded in the logs; no warning was suppressed.

The new ABI accepts immutable UTF8 bytes synchronously, validates header/address
arithmetic and schema/generation/revision/context under one state lock, catches
panic inside that boundary, retains owned values only and uses existing Curve
effect codes. Header checks cannot establish arbitrary pointer provenance.

**Native source proof remains RED9/18 on historical Runtime45.** No C++ caller
was connected and no plugin or host ran in this checkpoint. Rust CAD master
strings are intentionally opaque schema fixtures and do not prove OCCT geometry,
world correspondence, native lifetime, publication barriers or cold persistence.
The historical466/466 native regression cohort remains separate.

Native bridge schema: request `schema_version=1`, `operation=AcceptCurveNativeInput`,
actual command, canonical u64-string `generation`, `admission_revision`, and
restorable Circle `context_before`; exact unit snapshot; `action=point|reference|
fit_batch`, original text or null mouse text, raw world point/normal and ordered
FitPoints; `source_refs` plus `source_snapshots`. Indices into the common result
include numeric input at0, so source snapshot indices begin1. Discrete subsets
carry ordered zero-based `selected_point_indices`; whole-source fallback requires
empty subelement and exact full source order. CAD point observations still require
trusted native capture/kernel proof. Identical snapshots currently include the
selected-point observation; sharing different subsets still needs completion.

## Remaining required work

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Common produced GeometryResult now enforces bytes16MiB/nodes1million/depth64 before serialization/caller-buffer copy. Source-envelope admission is still necessary only: complete request/output overhead reservation and aggregate allocation/traversal/worker/cancellation bounds remain required.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Curve retains owned pre-normalization values at Rust entrypoints: exact text/units/frame/previous/pending options, original mouse points, native reference values and whole FitPoints batches with Circle branch Undo. Full native source identity/revision/attributes/snapshots, GUI-native parameter text such as OnCurve fractions, common ConstructCurve GeometryResult, checked publication, Ellipse foci, Circle History and Rebuild result pairing remain required.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.
- Runtime45 native source witness is RED9/18: raw native token, immutable identity/revision/frame/ranges, and full parent native source snapshots/attributes are absent for Circle path, Ellipse path and Circle FitPoints after whole-batch Undo. Implement these without replacing existing solvers.
- All non-Gumball/non-lib.rs frozen inputs match. Gumball and Rust module-root files changed in the parallel gumball chat; complete current checkout/build correspondence is unproven. The 448 rows are a pre-build inventory, not an exact compiled-input certificate.
- The owned native Curve JSON admission API is tested in Rust/extern-C only. No C++ declarations/callers were connected and no plugin/native host was built or launched in this checkpoint. Runtime45 native source witness remains RED9/18; prior native466/466 belongs to its preserved historical binary.
- Full parent CAD master fixtures used by Rust/ABI tests are opaque schema strings, not OCCT geometry proof. Native capture must prove full parent geometry, transformed locations, source content/attributes, selected subelements/indices, and cold FCStd persistence.
- Native root location is validated for numerical affine invertibility; it is not forced equal to source_ref.world because CAD capture may bake placements. Kernel world correspondence requires a native witness.
- Discrete selected_point_indices bind ordered subsets to transformed complete XYZ; whole-source fallback requires empty subelement and all XYZ in original order. C++ must supply truthful indices when capturing subsets. Mesh/discrete native integration remains unverified.
- Snapshot sharing currently deduplicates identical full snapshot JSON, including selected_points_world. Different subsets of the same parent may still duplicate geometry; complete indexed sharing across subsets and aggregate native allocation/lifetime bounds remain required.
- Rust checks unit snapshot model scale against session scale. Full unit-context revision/property/content freshness and active path source validity must be guarded at native capture and final checked publication.

## Next action

Wire minimal FreeCAD/OCCT Curve capture adapter to om9_curve_native_input_json with generation, admission_revision and Circle context guards. Capture full parent CAD/discrete data, attributes and selected index/range observations at admission; preserve raw GUI text, source content revalidation and Circle Undo resource contexts. Build standalone OpenMatrix9Gui only and replay native RED witness unchanged for physical/source expectations, followed by matching regressions and cold reopen. Then complete ConstructCurve GeometryResult and checked publication plus every original RCORE-01 requirement.

[Machine checkpoint](2026-10-11-rhino-core-curve-native-admission.json) retains
source fingerprints, actual exits and the separate historical native checkpoint.
