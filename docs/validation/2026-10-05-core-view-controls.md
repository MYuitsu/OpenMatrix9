# Core view controls validation checkpoint — 2026-10-05

Full 130-spec 01-core goal remains in progress.

- Fixed native compile errors: use BaseView::getGuiDocument and include SbBox2s.
- Native CoreViewControls object compiled successfully; isolated pyd linked in build/view-controls-runtime. The SDK module used by the user's open preview was preserved.
- cargo fmt --check and cargo test --test core_view_controls: five passed, including Perspective focal-plane scale, dynamic projection scale/bounds, reversed/tiny rectangle, invalid input and cancellation.
- Native 100% run: build/view-controls-isolated-1/20804de83af749179dec5ebf21b565f2/results.json, 17 checks passed. Menu/CMD/mouse drags, scale/center, Esc rollback, no model or Undo changes, Crosshairs Qt event tracking, WB/doc lifecycle and task restrictions.
- Crosshairs hover requires explicitly delivered QMouseEvent in the automated fixture; QTest's unpressed mouseMove alone did not deliver a tracking event. Physical mouse tracking still needs direct verification.
- 150% run remains failing the independent rectangle-center assertion: build/view-controls-isolated-1.5/05014028de384200b31175b090f23434/results.json. Actual [-2.218253,15.833333,100], expected [-2.488659,16.712708,0]. Do not count DPI completion. Need inspect actual Coin viewport region against Qt logical viewport and rectangle pixel conversion before changing behavior.
- An initial ray test omitted viewport aspect. Fixed using camera.getViewVolume(aspect). A temporary camera-position inversion was disproved and removed; current code uses native boxZoom unchanged.
- Perspective-source SynchronizeViews implementation and regression assertion exist but have not yet been rerun against the isolated binary.

Next: inspect Coin viewport dimensions on window zoom at 150%, correct conversion or projection test based on evidence, rerun 100/150/200 and core workspace regression. Then continue remaining View and File/Info commands. Do not replace goal with this subset.

Follow-up evidence:
- The 150% center failure was a fixture error: rectangle start y=-5, whereas the controller clamps it to y=0. Use proportional in-bounds points. The remaining 200% mismatch was a one-pixel top-down box boundary convention; the native box center is sizeY - midpoint, without the point-pick -1 correction. Native implementation retained.
- 100%: build/view-controls-isolated-1/e64466e0661e441c8faeabcab2930e6a/results.json, 17 passed.
- 150%: build/view-controls-isolated-1.5/cb4101f85f814338ade55545a4110f72/results.json, 17 passed.
- 200%: build/view-controls-isolated-2/7df227b095be4c7bb11ea6b475a1bb0b/results.json, 17 passed.
- Core regression: build/core-views-isolated-1/d2282747d54648a0b5123faf20619501/results.json, 32 passed, including Perspective-source sync, metadata reload, multiple documents and native rendering.
- Added internal viewport pixel diagnostic property to validate the actual Coin region. No geometry changes.
- Notes implementation now underway per docs/superpowers/plans/2026-10-05-core-notes.md. Full core goal is still active; view control source coverage and physical mouse hover still need final review.

### CMD Notes during active Curve
Native regression first failed to open Notes while Line awaited its second point. Removed the idle-only dispatch condition so CMD Notes uses the menu handler, cancels pending interaction, and opens the editor. Recompiled CurveController and linked the isolated module; core_notes_smoke passed all 28 checks. Evidence: build/core-notes-isolated-1/908f0435514241999b902340dfd396df/results.json. The visible preview process was preserved. Perspective activation camera persistence remains unresolved; the full 01-core goal is incomplete.

### Host projection action feedback correction
Native trace showed QAction::setChecked triggering the projection command while active-view UI state was refreshed. In src/Gui/CommandView.cpp, both camera isActive methods now use existing Action::setBlockedChecked; actual user activation remains unchanged. The host CommandView translation unit compiled successfully using the SDK compile command with E: paths resolved to D:. Removed the ineffective delayed WindowStateChange angle restoration in CoreWorkspace and compiled that object successfully. Isolated host DLL link is running; native persistence regression is pending. Preserve the visible preview PID 22440.

Host isolated DLL link completed successfully (session 30848), and the OpenMatrix9 isolated module relink completed (98911). First separate-executable runtime attempt exited -1073740791 before producing results. Runtime home lacked data/Ext/Mod because this D: filesystem rejects junction creation (Incorrect function). Copied data/Ext and root Mod files; separate host runtime setup and camera persistence verification remain pending. No production SDK DLL was overwritten.

### Patched host runtime verified
Separate executable now starts with local Mod/pivy, Mod/Part and Mod/Material plus Materials.pyd in bin (Part native dependency). No SDK DLL replaced. 36 workspace checks pass, including .42 Perspective angle after FCStd reload/activation and explicit native projection commands in both directions: build/core-views-isolated-1/e9b287a6f2f5419cb614792de4e24425/results.json. 19 view controls checks pass: build/view-controls-isolated-1/db2e45653edb4731b9d17a432e97d594/results.json. Zoom fixture now projects native integer midpoint of rounded physical endpoints, matching NavigationStyle::boxZoom; its old nearest-rounded midpoint was one pixel wrong for odd rectangles. Host two-line correction preserved in patches/freecad-camera-action-state.patch. Higher DPI, physical hover and broader core acceptance remain open.

Notes regression on the patched host passed all 28 checks: build/core-notes-isolated-1/b384b094b15a446cb02508ef07c8c045/results.json. This includes CMD Notes during a pending Curve, persistence, Undo/Redo, selection isolation and restricted tasks.

### Patched-host DPI regression
150%: all 19 controls checks pass, evidence build/view-controls-isolated-1.5/97ca4be5614e452cbe7d495fb12de9c5/results.json. 200% remains failing: camera height stays 50 and native window zoom completion property is absent. Added QApplication event observation; both press and release reach the target viewport with expected positions, but no rubber band is created and the tool prompt remains pending. Evidence build/view-controls-isolated-2/287e005711ce4883a0c3f20ea89e4821/results.json and before-zoom.png. Screen logical geometry512x360, viewport97x52. Hiding Report View/disabling report auto-display did not resolve it; removed those unsuccessful fixture changes. Next investigate native event-filter dispatch/containing-view matching; do not classify this as a mere coordinate/screen limitation or passing DPI test.

### Input filter ordering correction
Native trace comparison: DPI100 press reaches CoreViewControls; DPI200 press reaches Qt's target widget but not the controller. Reinstalling the application filter when starting a view tool resolves that ordering conflict: original19 checks pass at200 (build/view-controls-isolated-2/a6d6822a22c1449fa354a571d64ba806/results.json). Added initial Ctrl-drag regression before any menu tool; it failed. Reordering only during workbench/doc activation did not fix it; removed attempted overlay initialization and deferred timers. Ctrl key press in a managed viewport now prepares handler precedence before mouse press, subject to task view permission. Fixture now simulates actual Control key press/release in addition to mouse modifier flags. Expanded20 checks pass at200 (build/view-controls-isolated-2/6077dc3b65564d18b6d1acdd44688ef8/results.json). Debug trace properties subsequently removed; final-build rerun pending. Crosshair hover is still synthetic; no claim of physical desktop input verification.

Final module after removing temporary native trace:20 controls checks pass at200%, build/view-controls-isolated-2/5f997763475d43859c870958bdbcd164/results.json.

Final module100%: expanded20 controls checks pass, build/view-controls-isolated-1/7bd62c319686443f81704d2b7487d74c/results.json. Input priority correction changes no Rust state/ABI or model transactions.

Final module150%: expanded20 controls checks pass, build/view-controls-isolated-1.5/bb96759bdaec4d40a30c660b1a7057af/results.json. Final control regression matrix100/150/200% all green. Physical hover remains open.
