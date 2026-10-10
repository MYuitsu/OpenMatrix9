# OM9-INFO-008 — Viewport Tabs Toggle

Status: implemented and native verified, including normal-exit persistence and clean shutdown. Earlier failure evidence below is retained as diagnostic history and is superseded by the final results.

## Source contract

The local `specs/01-core/om9-info-008-viewport-tabs-toggle.md` specifies the public command `ViewportTabs`, under Info & Settings. It hides or shows viewport tabs to switch views while one viewport is maximized. No geometry or selection input is required. The spec lists Show/Hide/Toggle and Align/Top/Bottom/Left/Right as option cues, without reliable defaults; these cues must not be presented as recovered defaults.

## Integration design

Keep the existing four real document views and construction planes. Provide a viewport tab strip for the active document; choosing a tab activates the corresponding native view. If the current quadrant is maximized, preserve the maximized state when switching. Do not switch QMdiArea wholesale to TabbedView: native FreeCAD document tabs include all documents and would replace the four-quadrant arrangement.

Menu and CMD must dispatch the same handler under the original command name. Rust owns visibility/alignment state and input normalization; Qt owns the tab strip and native activation. Hide the owned strip outside OpenMatrix9 and recreate its active-document labels without duplicating it. No model object or document Undo entry is created. UI preference storage is separate from model geometry; its chosen default and persistence policy will be documented as OpenMatrix9 decisions.

## Required verification

Implemented command syntax: `ViewportTabs [Show|Hide|Toggle|Align Top|Align Bottom|Align Left|Align Right]`. Menu and CMD share the Rust state and native handler. OpenMatrix9 chooses visible tabs at Bottom by default; these are not recovered Matrix defaults. State is a global user preference at `BaseApp/Preferences/Mod/OpenMatrix9/ViewportTabs/State`, encoded as `(alignment << 1) | visible`, with alignment Top=0, Bottom=1, Left=2, Right=3. No geometry or document Undo entry is created.

Native evidence: `build/view-tabs-isolated-1/13f6d9fb40244792a1892955cdb07c09/results.json` passes 28 checks, including task permissions, deferred quadrant reconstruction and preserved active focus. These are Qt mouse-event tests; physical mouse coverage remains separate. The process startup log reports a camera exception and abnormal termination after the checks. A restart test without explicitly saving parameters failed because no user.cfg was written. Do not treat the successful results JSON as proof of clean shutdown or normal-exit persistence.

- Menu/mouse and CMD hide/show the same strip.
- Four tabs match the four native view labels and activate correct views.
- Switching a maximized view preserves maximization; Restore returns the four quadrants.
- Document/workbench switches and view close/reconstruction do not leave stale tab targets.
- Model geometry, selection, cameras and document Undo history remain unchanged by the visibility toggle.
- Validate declared alignment options, invalid input and persistence once implemented.

Explicit `App.saveParameter()` followed by a new native process restores hidden Right tabs: all 29 checks pass in `build/view-tabs-isolated-1/0f0f3da41f4e4d5d8d568d72301ea9de/results.json`. This proves preference serialization/load, but the startup log still shows abnormal termination at shutdown. Clean-exit persistence remains outstanding.

Final verification: fixture settling now processes deferred deletion, and reconstruction waits for the native view's destroyed signal. All 28 checks pass with normal shutdown and no explicit parameter save in `build/view-tabs-lifecycle-isolated-1/47a73e7492ce45049f0186dd9ee5a2d4/results.json`; its generated user.cfg is restored by a new process with all 29 checks and exit zero in `be6736eedb3c4146b8611481816494d7` under the same directory. Normal-exit persistence is now proved for this sequence. Physical mouse testing remains separate.
