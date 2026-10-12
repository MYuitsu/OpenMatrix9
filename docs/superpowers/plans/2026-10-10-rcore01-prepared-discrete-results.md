# RCORE-01 prepared discrete results implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline. Steps use
> meaningful RED/GREEN and one fresh final source review.

**Goal:** Apply the original owned GeometrySnapshot/admission/publication contract
to prepared mesh and float64 PointCloud rows and affine previews.

**Architecture:** Reuse the existing bounded Rust discrete model/native reader,
working-V5 capability registry, result assembler and frozen binding metadata.
C++ captures native mesh, makes its private native copy and freezes cloud data;
Rust validates the current schema and owns results. Python only binds the
native-generated report on existing host adapters.

**Tech stack:** Rust bounded JSON/model/FFI; Mesh::MeshObject/MeshPy, existing
PointCloud fields and ON_Plane validation; shared FreeCAD/OCCT/openNURBS SDK.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
original schemas, adapter items1–7, C05 and T07/T11. Full chapter remains the goal.

## Global constraints

- Shared SDK, standalone OpenMatrix9Gui and existing plugin cache only.
- Mesh/cloud remain their own representation; no CAD/mesh conversion or repair.
- Existing supported cloud schema1 keeps float64 points/normals/values, RGBA,
  ordered and plane fields. No new Python business validation.
- Reuse existing discrete count/value/index/8MiB serialization bounds and
  prepared cumulative64MiB/depth64/node limits before native copies/publication.
- Native sources remain GUI-owned; Rust gets synchronous initialized bytes or
  independent arrays, never document/Python/kernel owners.
- Upstream archive conversion remains uninspected; native storage precision
  and supported field coverage are explicit. No all-channel or global safety claim.

## Review focus

Native MeshPy subclasses/custom callbacks; module delay loading; observer
mutation of original mesh/cloud values; malformed/oversized archive arrays and
cloud optional channel counts; active plane parity with existing ON_Plane;
float64 retention and single Undo/Redo/cold persistence of report and host data.

## Task1 — Rust discrete result validation

- [x] Write focused result tests for mesh/cloud schema, field ownership, unit
  revision, bounds/index/channel errors and typed rejection; run meaningful RED.
- [x] Add `rust/src/prepared_discrete.rs`, integrate existing `prepared_result`
  assembler and working-V5 registry; reuse the current discrete model.
- [x] Run focused/full Rust tests; preserve existing CAD result ABI and behavior.

## Task2 — Native prepublication ownership and binding

- [x] Add real host fixture before implementation: observer mutations, invalid
  native/type/schema/channel rows, structured identity/no publication/source and
  executable Redo; reproduce RED against accepted snapshot plugin.
- [x] Expose existing checked mesh reader for live MeshObject owners; freeze
  cloud schema under current budgets and capture ON_Plane validity natively.
- [x] Prepare private mesh/plain cloud geometry and Rust-generated result before
  transaction; bind reports through existing Python host APIs for all rows.
- [x] Matching standalone build, host GREEN/cold/full regressions, one fresh
  source review and fixes, then synchronized original chapter/evidence ledger.

Interfaces: Task1 consumes owned JSON snapshot/current native observations and
returns existing `om9_prepared_result_json` query/copy ABI. Task2 supplies those
fields through the existing native result assembly helper; no owner escapes.

Ruling: prepared archive vertices/faces must match the published native mesh's
oriented triangle multiset at the declared float32 host precision, after applying
the native location exactly once. Rust checks cyclic rotations, reindexing and
duplicate multiplicity; original float64 vertices/quads remain retained. Source
inspection found unchanged native signatures made the exporter trust unrelated
archive data. Native/Rust RED proves mismatch before this check; mismatches now
use ConversionLoss before transaction. This is a scoped correspondence check,
not an exact float64 conversion or global manifold certificate.

Accepted discrete evidence:675 Rust;81 warm+8 cold+3701 regression=3790 host
checks;26 matching actual exits0. One fresh source review preceded controller
correspondence RED/GREEN correction. Full original chapter remains in progress.
