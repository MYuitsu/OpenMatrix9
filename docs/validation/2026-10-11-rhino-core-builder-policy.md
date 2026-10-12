# RCORE-01 — independent Builder admission — 2026-10-11

Independent existing affine-template Builder requests use the shared Rust operation/type/options policy before native source/template copy and GUI transactions. Create, persisted validation/recompute, explicit restore and all Match targets reuse the existing affine evaluator, shape tags, frames and slot/History rules.

**RCORE-01 and the full construction integration plan remain in progress.**

## Change and ownership

Rust owns NativeBuilder Request strings, counts, ScaleToGem and optional
initial/current dimension triples. The existing feature-ID syntax, versioned
recipe parser, affine_plan and replay budgets remain authoritative. All seven
native Part logical representations remain admitted; Mesh/cloud/instance/group
wrappers are unsupported by this API. Syntactic IDs do not enable a setting or
cutter command and do not certify its original solver.

The synchronous status FFI checks explicit byte lengths, stage/type/counts,
null/alignment/address arithmetic and zero-or-six dimensions before reading
caller-owned buffers. Existing limits include24-byte feature IDs,1MiB recipes,
1024 templates per record/targets and aggregate replay bounds. Rust retains no
foreign pointer or native geometry. Caller buffers must remain initialized,
immutable and in one live allocation per buffer; header checks cannot prove
that contract. Release panic=abort and native dependencies are not certified
memory-safe. The scalar counts query gates persisted template enumeration
before reserving its native shallow-shape list.

C++ remains a permanent exception for FreeCAD property/type resolution, OCCT
borrowed input observations/copy/transform, GUI-thread transactions and native
recompute. Existing InputTree validates source/seeds/stored templates before
private native copies. Final dimensions/options are re-admitted before the
existing affine solve. Create validates every seed before transaction; restore
validates the durable record before transaction; Match validates every record
and every target before creating the first matched record. Typed diagnostic
messages include Builder and the affected object/property. Existing dependency
invalidation, repair, Record/Update/Lock/detach, shape tags and world-frame
composition are preserved. The existing output path remains native: full
GeometryResult/conversion/maps and source/revision/lock/cache publication
remain original requirements rather than being implied by input admission.

## Verification

- Rust policy meaningful RED0/2 -> GREEN2/2: concrete Part/non-Part matrix,
  unsupported recipe/ID/counts, owned options and final dimension admission.
- Rust bounded FFI meaningful RED0/3 -> GREEN3/3: explicit bytes, concrete
  representations, null/overflow/misaligned headers, invalid UTF-8/stages/flags
  and nonfinite dimensions. Actual RED cargo exits101; GREEN exits0.
- Full Rust744 tests/108 suites, actual exit0. Standalone OpenMatrix9Gui build
  exit0; the shared FreeCAD SDK/host was reused without a root build.
- Native meaningful RED4/9 on the previous Circle History plugin: a BSpline
  seed with pole1e151 was accepted and consumed executable Redo; Mesh rejection
  and persisted invalid recipe lacked typed context. Actual host exit0, no
  fixture exception. The unchanged reproduction became GREEN9/9.
- Expanded policy GREEN61/61 covers those cases, malformed persisted recipe,
  source/template basis, noncompound/1025 templates, invalid initial dimensions,
  invalid restore preserving executable Redo, later Match target/recipe failure
  without writes to the first valid target, repair with durable links, and
  dimension/scale/offset/rotated-world output during an active Line session.
- Existing Builder warm43, grouped10, two fresh-file cold FCStd runs3+4 pass.
  Cold macros do not preimport the native module or activate the workbench;
  the actual loaded module is checked after FCStd opens, including restoring
  an explicitly deleted output slot.
- Matching integration totals1484 checks/32 actual host exits0 on plugin
  SHA256 `54faac49ef7f409b18da80ca51d88a28b667fc9399e908233bfe70019f696a74`. This includes the23 existing Curve/Solid/Surface/Edit/Cage/
  prepared-CAD suites, borrowed Cage policy, Circle History policy and two
  additional fresh Circle FCStd cold processes. Earlier checkpoint reports
  remain historical; no counts certify all original chapter requirements.
- First Rust policy RED attempt was a fixture compile error from a misspelled
  representation tag; it is preserved and excluded from behavioral RED.

Exact sources, reports, process exits, runtime and cold models are recorded in
the [receipt](2026-10-11-rhino-core-builder-policy.json).

## Remaining original scope

- Complete original GeometryResult/conversion/deviation/parameter/topology maps and metadata/source/revision/lock/cache integration for every enabled consumer, including native Builder outputs; these input gates are not completed-result certificates.
- Finish other independent consumers and Surface History JSON admission, aggregate metadata/sequence/traversal/native allocation bounds, then final all-family acceptance and one fresh review.
- Circle source-role/metadata storage and tangent-plane/constraint readers, Cage control extraction/budgets and missing/invalid dependency diagnostic propagation remain open.
- Computed Link getters, selected getSubShape cache allocation, explorer depth/stack bounds and original lifetime/worker/preservation/extrusion/annotation obligations remain open.
