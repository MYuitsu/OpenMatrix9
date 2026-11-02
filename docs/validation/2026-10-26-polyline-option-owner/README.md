# Polyline option ownership / running-version check — 2026-10-26

The reported screenshot shows Polyline's first-point prompt with Line's `BothSides=No` / `Length=` buttons. The running process was the older release executable (PID 39648, SHA256 `f6b75cee2f871c1ba816928cae1dbf426994cdd719c56d20f2fd4ddfca0a75a4`). It predates the common interface verified on October 25.

The old renderer checked `active_feature() == LINE`; that accessor falls back to the repeatable last command when a specialized pending command such as Polyline owns input. A completed Line can therefore cause the old renderer to display Line buttons over a Polyline prompt. The common interface already chooses `polyline_pending` directly, before any Line option declaration. No additional production-code change was necessary for this report.

`BothSides=No` means the Line command's two-sided drawing mode is off. Polyline has no such parameter. Polyline's Length option becomes available only after the first point in Line mode, to constrain the next segment's length; it is absent at the first-point prompt.

Added `line_repeat_slot_never_leaks_options_into_polyline` to the shared-interface integration target. The test completes a Line, preserves its repeat slot, then starts Polyline through both menu and CMD. It verifies the displayed click targets, first-point availability, Length appearing after a point, no BothSides leakage and unchanged document/Undo counts. All eight tests in the target pass; see [regression log](regression.log).

Native replay used the already verified debug executable (SHA256 `28915c56c1b2ab659624360939356c8a10dfcf8fafb0e96765ee5779cae76494`) in a new isolated profile. Menu Line → two viewport points → menu Polyline displayed only PersistentClose/Mode/Helpers. After one Polyline point, Length/Undo appeared and BothSides remained absent. The pending Polyline was cancelled; the scratch debug window remains open with its test Line. The user's older release window and its unsaved drawing were untouched.

The screenshot evidence is in `implementation/evidence/M01/polyline-option-owner-20261026`. No new release was built or installed, no human milestone gate was signed, and prior geometry limitations remain unchanged. This is verification of the existing interface behavior, with a regression test; no reusable business rule was changed.
