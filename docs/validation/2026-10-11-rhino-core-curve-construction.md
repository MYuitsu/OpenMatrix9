# RCORE-01 — Curve construction foundation, 2026-10-11T20:30:28+07:00

**Status: in_progress. All original RCORE-01 requirements remain required.**

Line, Polyline, Rectangle, Circle, Ellipse and InterpCrv keep their existing
FreeCAD/OCCT and Rust solvers. Safe Rust now captures an owned completed
constructor value: exact feature/command, full-u64 session generation, current
normalized points, output role, options, units/frame, analytic plans and complete
nonrational spline constructor basis. Shared JSON limits remain16MiB/1million
nodes/depth64. The FFI checks generation and buffer bounds before any write;
query and copy retain no native pointer and caller allocation stays caller-side.

The minimal C++ adapter captures bytes before native preparation/transaction and
stores read-only OM9CurveConstruction with the direct output Shape in its existing
transaction. Ellipse main output is included. This does not certify a checked
callback-safe publication, native source snapshot or common GeometryResult.
The value labels normalized session data explicitly; it does not claim complete
original accepted inputs. Circle/Ellipse output points are labeled display-only.
Effective closure is separate from the persistent option. The solver's inclusive
input coordinate boundary is preserved; finite generated points do not introduce
a new admission bound.

## Evidence

- Runtime42 native RED12/42: all six solvers and single Undo already worked;
  missing construction identity, roles and persistence caused30 failed checks.
  Actual host exit0, no fixture exception.
- Focused Rust missing-API compile RED, then3 initial tests passed. Fresh review
  found incorrect Close metadata and boundary rejection; focused RED3/5 → GREEN5/5.
  An intermediate test used a Circle whose extent the existing solver correctly
  rejects; that fixture assumption was corrected to an admitted BothSides Line.
- Native review Runtime43 RED15/26 → Runtime44 GREEN26/26: periodic/deformable
  basis comparison, effective closure, boundary Line and generated BothSides output.
  Native poles, weights, distinct knots, multiplicities, degree, domain and
  periodicity are compared against the saved Rust basis, not only field presence.
- Full Rust838 passed/0 failed/0 ignored; standalone plugin build exit0.
- Matching Runtime44:413/413 checks across9 distinct actual host exits0.
  Direct constructors42, cold restore19, basis26, cold basis27, Curve binding59,
  reference rejection36, Circle History admission33, Solid61, Edit110.
- Two fresh cold hosts independently open11 saved FCStd files, retain exact
  construction bytes and recheck native geometry/basis without a live constructor.
- All442 frozen producer inputs match; shared SDK hashes unchanged. Only plugin
  targets were built. Rust hard-link/canonicalization and existing SDK/Vulkan
  warnings remain visible in logs; the SDK Python location warning is retained.

## Review and limits

Fresh review found two real P2 regressions: one-shot Close metadata and inclusive coordinate boundary. Both were fixed in one pass with focused Rust RED3/5 to GREEN5/5 and native RED15/26 to GREEN26/26. Native warm/cold basis comparison added. No additional concrete ABI/pointer/generation finding. No second review substituted for tests.

The earlier Runtime43 seven-host360-check prototype and full Rust836 are historical,
not pooled into413 or used as final source-matching proof. Its two metadata defects
are superseded by Runtime44 in the tested scope. Runtime42 common result evidence
also remains separate. Offscreen QPA does not certify mouse/OpenGL/viewport behavior.

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Common produced GeometryResult now enforces bytes16MiB/nodes1million/depth64 before serialization/caller-buffer copy. Source-envelope admission is still necessary only: complete request/output overhead reservation and aggregate allocation/traversal/worker/cancellation bounds remain required.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Curve now persists owned normalized constructor state for all six direct commands. Complete original accepted input/quantity/batch history, native FitPoints/path/tangent/reference snapshots, common ConstructCurve GeometryResult, callback-safe checked publication, Ellipse foci, Circle History replay and Rebuild result pairing remain required.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.

The [machine checkpoint](2026-10-11-rhino-core-curve-construction.json) records
source/binary/fixture/launch/log/archive hashes. Plugin SHA256: `18739fe83b96ea034dc30292f179009a6df96333263285ab32b35ad642b7c3fd`.

Next: retain pre-normalization accepted points/quantities/batches and full native
sources; connect ConstructCurve to common GeometryResult and checked GUI publication.
