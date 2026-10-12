# RCORE-02 effective tolerance integration

**Goal:** Resolve, apply and persist the actual tolerance used by curve/surface Join,
with a reusable Rust operation policy for subsequent solver migrations.
**Spec:** RCORE-02 context/rule/diagnostic contracts, T02/T03/T07/T08/T09;
OM9-TOP11-008 and existing native Join supported representations.

## Decisions

Rust resolves explicit override > explicit document context > legacy operation
default. Legacy curve Join remains 1e-7 mm. Legacy surface sewing remains 1e-6 mm
(the actual host default, previously unaffected by the misleading Join textbox).
The document surface-Join rule uses 2 x absolute tolerance, matching Rhino 5 Join
Help; an explicit override is an actual cutoff in mm, clearly reported. Relative
and angular tolerance do not participate in these Join operations; reports say so.
Display precision has no effect. No other solver thresholds change in this task.

The numerical lower bound is 32 machine epsilons times max(1 mm, absolute world
coordinate bound). It is an OM9 conservative cancellation/rounding guard, not a
certified OCCT geometric error bound. Values below it reject rather than relax.
Limits 1e-9..1e6 mm preserve the existing Join UI domain; surface 2x multiplication
is checked independently for overflow/range. Invalid/future contexts fail closed.

Native extraction computes world-coordinate bounds from already frozen inputs.
The context is captured at command start, and both preview and commit resolve
against that same context. Explicit option changes preserve the context and produce
a new report. Output metadata stores the context and versioned resolved rule.
FreeCAD's current Python sewShape parser ignores its tolerance argument, so use
OCCT BRepBuilderAPI_Sewing directly in the existing native Edit adapter.

## Tasks and tests

- [x] Add `rust/tests/tolerances.rs` with precedence, operation-specific legacy
  defaults, surface 2x rule, unused relative/angular values, invalid/future context,
  numeric bound/overflow, stable encoding and ABI untouched-output checks. Run red.
- [x] Implement `rust/src/tolerances.rs`: `resolve(Operation, Option<&Context>,
  Option<f64>, coordinate_bound) -> Result<Resolved, Error>` and a bounded C ABI.
  Export the module; run targeted tests green.
- [x] Add native `CoreTolerance.{h,cpp}` with value-only result/report serialization,
  GUI-thread document context access through existing CoreUnits and no doc mutation
  on resolution. Integrate latched context/options into EditController.
- [x] Feed resolved cutoff into curve Join and actual surface sewing, record the
  report and context on outputs in the same commit transaction. Keep output shape
  validation and input-generation guards.
- [x] Add native `rhino_core_tolerance_smoke.FCMacro`: document tau .01 with curve
  endpoint gaps .005 (join) and .015 (reject); explicit override; legacy behavior;
  surface seam gaps .015/.03 for document 2tau; precision invariance; stale context;
  large-coordinate resolution rejection; Undo/Redo/save and separate-process restore.
  First reproduce old missing document tolerance and ignored seam tolerance.
- [x] Run affected Edit/History/Surface regression, matching SDK build, full Rust/
  Python/source audit; review ABI and numerical policy; synchronize precise evidence.

Sources read 2026-10-09: official Rhino 5 Units properties and Join Help. RCORE-02
reference index states no complete guide coverage; Units Help is authoritative for
the source comparison. OM9-specific precedence, bounds, serialization and validation
are labeled decisions above. This task is part of the all-P0 completion plan, not
a substitute completion target.

Verification: docs/validation/2026-10-09-rhino-core-tolerance-workers.md. Full P0 remains open in the master plan.
