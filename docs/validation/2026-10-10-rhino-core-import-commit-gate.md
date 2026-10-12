# RCORE-01: prepared CAD rejection before document mutation

The supported synchronous import binder now validates live prepared CAD and
preserved instance CAD previews before topology evaluation, opening a FreeCAD
transaction or assigning properties. It reuses the existing Rust basis/input
tree policy and native OCCT adapters. The C++ addition is limited to Python/Part
wrapper extraction, GUI-thread enforcement, exception mapping and native
topology validation; portable numeric policy remains in Rust.

## Observed defect and repair

On the original binder, a curve with a `1e151` pole entered the document without
an error. The batch created four objects, changed Undo 1 → 2 and consumed Redo
1 → 0. That RED run is retained in
`H:/FreeCAD-src/build/import-commit-gate-red2-20261010/results.json`.

The first repaired gate passed 29 host checks for direct curve/surface
coordinate-limit/NaN/infinity cases, valid rational controls and executable
Undo/Redo. A separate safe worker RED showed that entry did not reject with a
GUI-thread diagnostic. The adapter now rejects before document lookup.

[Independent review](../reviews/2026-10-10-import-commit-gate-review.md) found
that overridden Python container length/iteration could disagree with raw
native indexing and the later binder. The retained container RED demonstrates
a stored-valid/yielded-invalid batch bypass. Exact built-in containers and
intrinsic length checks now reject these cases before indexing or binding.

## Accepted scope

The final host fixture passes **55 checks**:

- twelve malformed curve/surface paths: geometry batches and preserved affine
  preview dictionaries, each with coordinate-limit, NaN and infinity values;
- exact persisted property member bytes, property types/statuses, object states,
  complete document/object/view inventory and Undo/Redo preserved on rejection;
- detached malformed shape BRep bytes unchanged;
- a real Redo transaction still executes after all rejections;
- valid rational curve/surface import uses one transaction; Undo removes the
  batch and layers, Redo restores both native shapes;
- worker entry and three container-override regressions reject before mutation.

Property snapshots use FreeCAD's actual PropertyContainer serialization and
compare ZIP member payloads, excluding container timestamps. They avoid computed
`App::Part.Shape`/`App::Link.Shape` getters. An unchanged-name metadata edit proves
the oracle detects property values. Native fake curve/surface count/grid overflow
is covered separately by the existing initialized native fixtures; this is not
invented as a public Python count-input API.

The accepted final artifact directory is
`H:/FreeCAD-src/build/import-commit-gate-container-green-20261010`. Exact source,
plugin/host hashes, process exits, regression reports and excluded attempts are
recorded in [the machine evidence](2026-10-10-rhino-core-import-commit-gate.json).

## Limits and remaining chapter requirements

This closes the reproduced prepared-CAD numeric mutation gap, not RCORE-01 or
all T07. Inputs must be live and immutable throughout the synchronous call;
the GIL does not certify arbitrary native pointers or application callbacks.
Per-tree Rust budgets do not yet form an aggregate batch/worker memory budget.
Input/native archive decoding and extraction buffers may allocate before full
value validation; no process-wide allocator instrumentation is claimed.

Complete metadata validation, prepared mesh/cloud payloads, unexpected binder
failure/rollback policies, pending user transactions, remaining analytic/cage
consumers, uniform diagnostics and other workers remain required work. The
preserved-preview negative fixture deliberately lacks a manifest, and requires
the native basis error so a binder/manifest failure cannot pass it. Real
preserved imports are checked by a separate regression.

The [full chapter audit](../reviews/2026-10-10-rcore01-closure-audit.md) retains
every capability, numbered adapter requirement, fixture and schema obligation.
The next core work is the common typed BRep topology/representation snapshot,
reusing existing hole/seam/shell/solid converters and tests.
