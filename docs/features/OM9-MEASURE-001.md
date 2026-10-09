# OM9-MEASURE-001 — Angle

Status: partially_implemented; validated_supported_slice (2026-10-09).

**Supported slice:** native four-point Angle measurement, shared Cartesian point input and accepted-coordinate reporting. The TwoObjects branch remains unimplemented.

### Inputs

Enter or pick the start/end of the first directed line, then the start/end of the second. No geometry selection is required. Typed input uses the shared Rust parser: CPlane `x,y` / `x,y,z`, world `wX,Y,Z`, relative CPlane `r`, `R` or `@`, and relative world `wr`. Relative input starts from the last accepted world point across all four phases. Explicit length units and the captured document input scale are supported. Native picks, available snaps and F4 origin are already world coordinates and are not transformed a second time.

### Parameters and defaults

The supported branch reports degrees from 0 through 180 between the two directed line vectors. Both line lengths must exceed `1e-12` mm; world coordinate components must be finite and within ±`1e9` mm. `TwoObjects` and `Unit=` currently return an explicit unsupported-option message. Object selection, planar-face normals and object orientation/principal-versus-supplement behavior from the full contract remain open.

### Output

The final Angle result and each accepted point in world millimeters appear in command/report history. A completed, owned coordinate report is also exposed through the main-window `om9MeasurementReport` string property. The command creates no document object, dimension or annotation and does not modify document geometry, selection, units or Undo state.

### Preview / commit / cancel

The command retains accepted inputs until the fourth valid point. A rejected zero-length endpoint keeps the current phase and previous accepted point; failed inputs do not publish a completed report. The command-start CPlane is latched. Document owner/UID and the exact captured unit context are checked before input acceptance and completion; document switch/close or an invalidated context ends the session. Esc, command replacement and workbench cancellation clear pending measurement state.

### History / dependency model

Command/report history and the owned UI report record accepted inputs, frame revision, logical view slot, units and source kind. They are not an associative History feature, persisted measurement object, or dependency link to snapped geometry. Snap object identity is explicitly unavailable in the present report schema.

### Error / invalid-input behavior

The shared parser rejects malformed, ambiguous, non-finite and overflowing input without advancing accepted state. Rust computes the angle; the native adapter manages FreeCAD/Qt lifecycle and reporting. The final matching-module fixtures verify failed retries, lifetime and read-only invariants. Broader grammar and TwoObjects remain open.

[Validation and remaining scope](../validation/2026-10-09-rhino-core-point-consumers.md).
