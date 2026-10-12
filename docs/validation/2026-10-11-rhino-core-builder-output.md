# RCORE-01 — Builder solved-output publication — 2026-10-11

**RCORE-01 remains in progress.** This checkpoint accepts the existing native
affine-template API's output staging and publication. It preserves the original
all-family goal and does not certify full GeometryResult or Rhino builder parity.

## Defect and resulting behavior

On the prior plugin54FA, a valid degree1 BSpline seed with pole1e149 and scale1e9
produced an over-budget output. Create, explicit restore and the last Match
target rejected it after a document write; abort restored objects but consumed
the executable Redo stack. Missing-first/occupied-last restoration also consumed
Redo. RED4 ran25 checks,17 passed/8 failed, actual host exit0, no fixture error.
The exact RED fixture is preserved alongside its report. Earlier RED2 restore
setup was weak because outside-transaction setup was affected by witness Undo;
RED3/4 use explicit setup transactions. Those weak assertions are not evidence.

Create now solves every seed; restore checks every slot identity and solves all
slots; Match copies templates and solves every record/target before opening the
first transaction. OCCT retains its existing affine and world/local Location
composition. InputTree admission precedes output copies; CoreTopologySnapshot,
CoreShapeValidation and shared Rust native CAD result validity inspect the local
result. The scalar native validity helper is independent of import policy.
Import retains its own operation/backend gate; Builder separately uses the
actual NativeBuilder type/options/dimension policy. No import conversion or
lifetime evidence is relabeled as Builder evidence.

Native geometry copies and local shapes are C++ owned because OCCT/FreeCAD
objects require their native APIs. Rust owns portable request/basis/solid validity
policy. No document or shape pointer is retained by Rust. Prepared shapes exist
only during this synchronous GUI API scope and are cleared after publication or
abort. They do not become a durable validity certificate or a worker cache.
Existing detached output slots remain unchanged.

Publication recomputes only the prepared Builder records/outputs through native
nonrecursive recomputeFeature. A whole-document recompute touches unrelated
dirty Part sources and invalidates captured revisions; first-build GREEN1
exposed this on the repaired Match case. Source/generation/ancestor/unit tokens
are checked before writes and before commit. This establishes the current API
scope, not the original uniform all-family stale/lock/cache contract.

## Verification

- Final output28/28: typed output/slot rejection, unchanged source BReps/records/
  links/Undo/Redo, executable Redo, native repair, complete Match batch removed
  and restored by one Undo/Redo, later History update after preparation expires.
- Builder input policy61, original warm43/groups10, fresh-file cold3+4 pass.
  Both cold processes load the native module from FCStd without preimport.
- Prepared CAD47 passes after shared Rust validity extraction; its import
  operation, evidence and ownership remain distinct from Builder output.
- Matching196 checks/seven actual host exits0 on plugin SHA256
  `1ef0c24c42c9983bffd3ce1439804ea1882924b88cfc3df979154a6e744ab6fb`; actual loaded module paths are verified.
- Full Rust744 tests/108 suites, actual exit0. Standalone OpenMatrix9Gui build
  exit0, reusing the shared FreeCAD SDK. No FreeCAD root build or runtime clone.
- Earlier candidate runs passed but the shared plugin output changed checksum
  between runs. They are excluded from the matching total. A subsequent relink
  found LNK1168 at that output; the locker/cause was not established. Only this
  standalone plugin cache's output directory was changed to the pinned plugin
  runtime, and all seven cases reran serially there. No host was terminated.
  A cached RUNTIME_OUTPUT_DIR initially overrode the requested RUNTIME_ROOT;
  the plugin Configure arguments now set both together. Build7 linked the pyd
  and import worker into the requested isolated bin without rebuilding FreeCAD.

Exact source/artifact/runtime hashes are in the [receipt](2026-10-11-rhino-core-builder-output.json).
Historical Builder input1484/32 evidence uses the previous binary and is not
counted as this checkpoint's current matching runs.

## Remaining original scope

- Full GeometryResult/ConversionReport for native Builder and all other enabled consumers, actual operation/lifetime and output metadata.
- Measured or explicitly unavailable deviation/parameter/topology maps and conversion facts; do not relabel import evidence or invent exactness.
- Source attributes and persistence/publication provenance, uniform stale/lock/generation/worker/cache contracts across all enabled consumers.
- Aggregate native staging, metadata/getter and worker/cache allocation budgets and remaining representation/preservation obligations.
- Independent bounded Surface History JSON admission and the original RCORE-01 numbered requirements/all-family final acceptance/review.
