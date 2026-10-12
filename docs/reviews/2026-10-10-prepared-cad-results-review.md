# Prepared CAD results review — 2026-10-10

Read-only review covered the prepared CAD plan, Rust admission/budget policy,
native metadata/geometry ownership, transaction boundary, report binding and
native fixtures. The reviewer did not run a host or modify source.

Two actionable findings were verified on the shared FreeCAD host before fixes:

1. Missing `tolerance` or other binding fields passed plain metadata freezing,
   then failed after `doc.addObject`. Opening a FreeCAD transaction clears Redo;
   abort removes the import objects but does not restore that prior Redo.
2. Row identity was updated incrementally. An unsupported value in the second
   row could be diagnosed with the first row's UUID; a malformed pair retained
   the first row's name as well.

`prepared-cad-metadata-red1-20261010` reproduced both findings: 85 checks,
77 failures, no unhandled fixture error, actual host exit0. Missing fields
produced raw Python KeyError and observed publication before rollback. The
second-row rejection recorded row1 with UUID `first` instead of `second`.

The fix validates binding scalar facts through Rust before the transaction:
required strings, positive finite tolerance, bool flags, color arity/channels,
32-bit integer properties and working metadata. C++ reads exact builtin values
without invoking conversion/iteration callbacks. Per-row identity is cleared
before pair validation and safe exact name/UUID strings precede recursive
freezing. Rust policy had a meaningful failing test before implementation.

Native transaction diagnostics are now noexcept at the error boundary; rollback
precedes diagnostic allocation. Unexpected Python binding failures receive a
structured KernelFailure and retain their original Python exception as cause.
A controlled binding fault after the real binder creates properties verifies
the exception boundary and rollback, without replacing geometry conversion.

The fixed focused host fixture passed89 checks and full Rust passed660 tests.
Matching final regressions and evidence are recorded in the
[validation record](../validation/2026-10-10-rhino-core-prepared-cad-results.md).
The source review made no further concrete ownership, custom-key callback,
admission or logical-budget finding within this prepared CAD scope.

This does not certify restoration of Redo after every unexpected host failure,
process-wide allocation safety, upstream conversion, complete result snapshots,
all non-CAD adapters or every operation family. Original RCORE-01 remains open.

Final regressions reproduced a readable-error compatibility issue: the new
wrapper omitted the existing injected binding reason. A focused89-check RED
failed only that assertion; bounded exact builtin exception arguments now keep
the reason and original cause. Final matching verification passed660 Rust and
3588 host checks, all22 actual exits0, including preservation/modeling.
