# Shell thickness follows document units — 2026-11-12

In a centimetre document, `Shell 0 0.1` previously sent 0.1 mm to OCCT instead of 1 mm. A saved/reopened 40 × 50 × 60 mm box produced 1176.204 mm³ instead of the independently constructed 1 mm shell volume of 11424 mm³. The regression failed before this fix.

Direct numeric input now converts the thickness once at the dispatcher. The picked-face path retains the number in document units for its command option and converts it before geometry or background verification. The background receipt compares the current value after conversion with the millimetre value actually used for the worker, preserving the existing stale-result gate. No face-selection or OCCT topology policy was changed.

The reopened-box regression checks direct CMD, menu dispatch with direct numbers, a picked face with `Thickness=`, and a picked face followed by numeric thickness; all four compare the resulting volume against a separate OCCT shell made with 1 mm and verify document Undo/Redo. Unit-conversion overflow in the picked path retains the pending command, document and Redo. The existing large-cylinder worker test also now checks direct and picked paths in centimetres, against its 0.5 mm oracle; those routes passed together with worker cancellation, stale receipt, budget and retry tests.

Validation: 15 Shell-filtered library tests passed; 98 Solid input regression tests passed; the Solid integration Shell test passed; the egui Solid interaction group passed 160 tests with three ignored. This covers the specified fixtures and background paths, not all shapes, physical desktop clicks, suffix input, asynchronous failures in every allocation or peak native memory.

The debug executable built successfully in the existing `L:/om9-target` directory with `-j 2`; the public source audit reported no errors. No physical desktop action was performed for this change.
