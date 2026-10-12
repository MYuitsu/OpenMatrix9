# RCORE-01 — Solid result and shared snapshots, 2026-10-11T19:19:24+07:00

**Status: in_progress. Every original chapter requirement remains required.**

Box/Sphere reuse the existing FreeCAD Part solver. The shared Rust GeometryResult
owns construction points, native source snapshots/attributes/units and an output
CAD master. Shape/result/construction/units publish through the existing checked
transaction. Callback changes reject stale sources or changed result metadata,
preserving prior executable Redo in the tested cases.

Repeated FitPoints selections keep every source reference and point range while
exactly equal owned JSON snapshots share one input slot. Reference indices address
the complete input_snapshots array: numeric construction input is index0; native
snapshots start at1. A versioned indexing marker prevents ambiguous legacy records.
Rust validates each reference against its indexed snapshot and preserves snapshot
order; no native owner or raw pointer is shared into the result.

## Verified evidence

- Matching Runtime39: **548/548 checks /12 actual host exits0**, run serially:
  warm61, guards13, CAD Fit5, cold26, binding93, policy18, callback150,
  publication86, construction67, accumulated-source budget5, indexed warm12/cold12.
- Actual matching warm fixture on Runtime35: **10/61** RED;
  Runtime39 **61/61** GREEN. Actual host exit0 is separate from failing verdicts.
- Actual matching large-source fixture: Runtime38 **3/5** RED → Runtime39
  **5/5** GREEN. Fifty-five accepted batches hit the retained-byte limit;
  the next batch is rejected without document/Undo change. Input Undo removes
  one entire batch;54 references/56 numeric points still produce one valid
  native sphere and one Undo transaction. Expectations were not weakened.
- Full Rust **826 passed /0 failed /0 ignored**, including10 focused SolidResult
  tests. Indexed-sharing RED9/10 → GREEN10/10 is synthetic schema/ABI evidence,
  separate from the real native large-source witness.
- Standalone OpenMatrix9Gui build exited0. The shared SDK binaries are unchanged.
- A distinct cold process begins with no model and opens five warm FCStd files;
  exact result bytes, construction/units and independently reconstructed CAD
  master volumes survive. Bootstrap imports OM9 first; this does not certify
  opening without OM9. A second warm/cold pair independently verifies54 alternating selected references with two distinct full source snapshots,56 numeric points, stable indices/ranges, exact saved result bytes and complete1000-vertex masters; both12/12, distinct host exits0.
- Independent source review found no actionable indexing/ownership findings.

## Environment and excluded evidence

The Windows QPA host previously stalled at QApplication startup. Five startup-only
attempts, including software OpenGL, remain excluded in the historical checkpoint.
Offscreen QPA permits the actual kernel/command/callback tests above, but reports
no valid GL context. An attempted offscreen mouse/render solid_commands fixture
did not finish and was terminated by exact owned process identity; it is excluded,
not GREEN or a semantic RED. Mouse/render/options coverage remains open. Older
Runtime38/35/32 cohorts are historical and are not added to current524 checks.

The node diagnosis reconstructs a request from actual native control data:
54 repeated full snapshots would create about1.63million JSON nodes, exceeding
the parser's1million bound despite remaining under the byte bound. It is an
explicit reconstructed model, not an intercepted failed request. Exact snapshot
sharing fixes the demonstrated repeated-source case without raising caps or
discarding geometry. Unique-source node admission remains an open requirement.

## Additional current failure — distinct selected-point snapshots

The same Runtime39 has a separate required native **RED3/5**, actual host exit0,
without fixture exception. Selecting a different vertex in each batch accepts55
batches, then the byte limit rejects the next selection. Input Undo removes one
batch, but Enter rejects construction with UnsupportedRepresentation ConstructSolid
instead of creating the54-reference result. The passing548-check cohort is not a
certificate for this case. No cap, input geometry or expected invariant was reduced.
Next: capture the exact failing request and reconcile node limits, retained input
and output duplication before accepting new source batches.

## Full scope still open

- Full original RCORE-01 remains required: GeometryRef/Snapshot/Result, rational and periodic basis/UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Global conversion/deviation bounds, parameter/topology maps, native source channels and unknown attributes remain unproven; closed valid native Solid is not their certificate.
- Current Windows QPA startup remains unresolved. Offscreen native/CMD/Qt-key/callback/FCStd tests are valid only for their scopes; actual mouse, OpenGL rendering and viewport acceptance remain required. Prior Runtime35 graphics tests are historical, not Runtime39 reruns.
- Current Runtime39 distinct-source native witness is RED3/5 with actual host exit0 and no fixture exception:55 selected vertex batches are accepted, the56th hits the byte budget, but input Undo then Enter cannot construct the54-reference result (UnsupportedRepresentation ConstructSolid). Shared identical-source and two-source indexed cold tests do not close aggregate parser/result admission. Capture actual failing packet and reconcile request nodes/output duplication/retention before accepting batches; preserve full data and expectations.
- Cold evidence covers five named recipes and a54-reference/two-source indexed result in two distinct cold processes. Bootstrap imports OM9 first. Unsignalled native root-frame mutation and actual Solid preview leases still need dedicated evidence.
- Existing CurveController/CurveGeometry commands must gain the common result path by reusing existing Line/Polyline/curve solvers. Source presence and alias dispatch do not close the enabled-consumer contract.
- Production deployment/ABI acceptance remains pending. Tests use an isolated App19/camera-guard GUI host with the single shared SDK; no FreeCAD root build or full runtime copy was performed.

Current binary/report/fixture/source/archive hashes and native selection receipts
are in the [machine-readable checkpoint](2026-10-11-rhino-core-solid-shared-snapshot.json).
Plugin SHA256: `7c69b3ec553b59bab555b95a7288137dfd3f67190117c69dc696ba0df888a96c`.

Next: verify unique-source aggregate parser admission,
then complete the common result path for existing curve commands. Continue the
original chapter audit, including conversion/maps, every family and actual UI.

## Working tree changed after this binary

Four Gumball files (Controller.cpp/.h and Overlay.cpp/.h) now differ from the
Runtime39 checkpoint. Original source fingerprints remain preserved; current
fingerprints are listed separately as source_drift. The548 passing native checks
and separate3/5 RED remain evidence for the exact Runtime39 binary. They do not
certify the whole changed working tree. Preserve the edits and rebuild the
standalone plugin before a new current-source acceptance claim.
