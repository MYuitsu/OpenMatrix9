# RCORE-01 — Native Builder conversion observations — 2026-10-11

**RCORE-01 remains in progress.** The existing affine Builder now retains actual
OCCT ModifiedShape correspondence through Copy/GTransform/Transform and the
world/local Location pipeline. It uses the same native operators as FreeCAD's
existing TopoShape evaluator. Source/output indices refer to bounded native
enumerations; Rust requires complete unique snapshot namespaces and a bijection.
The original template has an owned native CAD/topology/attribute input snapshot.

Vertices use actual native points; edges use native source parameters and output
evaluation/projection; trimmed faces use classified source UV and projected
output UV inside target trims. Rust independently resolves each frame once,
checks domains/counts and computes document-world residuals. Contradictory
measurements, invalid UV and nonexistent native IDs are rejected before publish.
Native query pairs and deviations are stored with GeometryResult and FCStd.

The report explicitly says sampled_only, recorded_parameters_only and null
global_bound/global_mapping. No interpolation, global exactness or raw parameter
preservation is invented. Actual Geom_TrimmedCurve wrappers are retained; the
circle's saved underlying degree2 NURBS has nonunit rational weights.

For huge valid geometry OCCT's distance-based projector can return no solution.
A source-parameter candidate is accepted only when actual native evaluation
equals the requested point, and only for that recorded query. Failed queries are
owned explicit KernelFailure diagnostics and unavailable_queries; their report
sets recorded_queries_complete=false and warns that geometry validity is
independent of parameter-query availability. Native geometry still goes through
the existing basis/trim/native result validators. No tolerance was increased.
This preserves existing Builder Create/Restore/Match/Undo behavior while exposing
the mapping evidence gap. Consumers must not treat unavailable queries as maps.

C++ owns OCCT operations/projectors/serialization and GUI publication. Rust owns
independent snapshot/observation values, admission/bounds/residual policy and
result serialization. Independent source review identified missing surface-query
failure handling; bounded UV failure evidence and local native query exception
handling were added (Rust face-failure RED13/14 to GREEN14/14). The reviewer
confirmed the fix with no other important issue. Native face-failure injection
remains untested; normal hole/cylinder queries and curve no-solution behavior
are covered by the matching runtime fixtures. This does not certify foreign pointer provenance, all
native allocations or the complete original chapter.

## Inspected evidence

- Native RED: 13 failures/15 assertions on the previous B120 plugin, no fixture
  exception, actual host exit0. New conversion GREEN: 17/17 on the final plugin.
- Initial candidate 15/16 identified a test's guessed outer class; the corrected
  fixture inspects saved rational basis/weights and actual native wrapper class.
- Existing output regression exposed projector failure on the 1e149 seed;
  a native basis probe independently confirmed scale3 evaluation rounding.
  Final output29 proves accepted native geometry, explicit unavailable-query
  reporting, atomic Match/Restore, executable Undo/Redo and later History.
- Matching 315 checks/10 actual host exits0, including Builder result,
  policy/warm/groups/fresh-process FCStd and prepared CAD/discrete import results.
- Full Rust 758 tests/108 nonempty suites, actual exit0. Behavioral RED
  paths cover residual contradiction, UV bounds, absent native namespace and
  unavailable-query ownership/completeness. Targeted prepared_result14/14 passes.
- Standalone plugin build9 exit0, shared FreeCAD SDK reused, no root rebuild.
  Plugin SHA256: `7d38704944fdfbde406ef9e3c138883475cc62ed8fd70db099bcc4db5c223590`.

All counts above use the same final plugin. Earlier candidate binaries and the
diagnostic probe are excluded. Relevant addition source fingerprints are checked
against the build9 snapshot; unrelated concurrent menu/archive work is outside
this checkpoint's source-completion claim. Runtime/harness/artifact fingerprints
are recorded in the [receipt](2026-10-11-rhino-core-builder-conversion.json).

## Original remaining scope

- Global conversion deviation bounds and full-domain bidirectional parameter correspondence; sampled queries alone certify neither.
- All remaining GeometryResult consumers, original representation families, attributes/userdata/preservation and operation policies.
- Uniform source lock/revision/generation/cancellation/cache/transaction contracts across every enabled entry point.
- Aggregate native/worker/cache budgets and independently bounded Surface History parsing/recompute.
- Original adapters1–7, C01–C12, T01–T11 final requirement-by-requirement acceptance and independent review.
