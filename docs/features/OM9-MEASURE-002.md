# OM9-MEASURE-002 — Distance

Status: partially_implemented; validated_supported_slice (2026-10-09).

**Supported slice:** native two-point 3D Distance measurement, five explicit display units, shared Cartesian point input and accepted-coordinate reporting. The contract's Model_Units display default remains open.

### Inputs

Enter or pick two points without requiring geometry selection. The shared Rust parser supports CPlane `x,y` / `x,y,z`, world `wX,Y,Z`, relative CPlane `r`, `R` or `@`, and relative world `wr`. Explicit coordinate units and the command-start document input scale are supported; relative input uses the last accepted world point. Native picks, available snaps and F4 origin arrive in world millimeters and bypass typed-coordinate transforms.

### Parameters and defaults

`Unit=mm`, `cm`, `m`, `in` and `ft` select the result display unit only. Each command currently starts in `mm`; changing this option does not change coordinate interpretation or document units. The specified `Model_Units` choice/default is not implemented, so the original default requirement remains unsatisfied. Coordinate components must be finite and within ±`1e9` mm. Coincident points are valid and return zero.

### Output

Rust computes Euclidean 3D distance. The final result and both accepted world coordinates appear in command/report history; a completed owned input report is published in the main-window `om9MeasurementReport` string property. No document object, annotation, geometry edit or Undo entry is created.

### Preview / commit / cancel

The first accepted point is retained through rejected coordinate or display-unit input. Only a valid second point publishes the completed result/report. The CPlane remains latched from command start; document owner/UID and exact unit context are rechecked before acceptance and reporting. Document switch/close, stale context, Esc, command replacement or workbench cancellation clears pending state. A failed attempt does not replace the previous completed report.

### History / dependency model

The coordinate report owns accepted input text, resolved points, units, source kind, frame revision and logical view slot. It is session/UI reporting, not persisted document History or an associative link to measured geometry. Snap object identity is explicitly unavailable; no replay dependency is inferred from the source kind.

### Error / invalid-input behavior

Malformed, ambiguous, non-finite and overflowing coordinates are rejected by the shared Rust parser. Invalid `Unit=` values retain the accepted first point and current supported unit. Final matching-module fixtures verify unit separation, frame latch, same-name document replacement and unchanged model/Undo state. Model_Units and broader grammar remain open.

[Validation and remaining scope](../validation/2026-10-09-rhino-core-point-consumers.md).
