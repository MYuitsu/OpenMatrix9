# ViewportTabs and workspace lifecycle

Scope remains all 130 local 01-core specs. This evidence covers only ViewportTabs and related lifecycle behavior.

The original `ViewportTabs` command dispatches menu and CMD to one native handler and Rust state. Alignment and visibility use global user preferences; visible Bottom is an OpenMatrix9 default, not a recovered Matrix default. Four tab targets resolve live native views for the active document. Maximized switching and task permission deferral have native Qt event coverage.

## Evidence

- `build/view-tabs-isolated-1/13f6d9fb40244792a1892955cdb07c09/results.json`: 28 UI assertions pass, but the startup log shows abnormal process termination.
- `build/view-tabs-isolated-1/0f0f3da41f4e4d5d8d568d72301ea9de/results.json`: 29 UI assertions pass, including a new process reading explicitly serialized hidden Right preferences. This does not prove clean-exit persistence.
- The runner formerly checked only the JSON report. `tests/run_menu_smoke.ps1` now also requires process exit code zero.
- Native host baseline closes successfully: `build/host-lifecycle-baseline-1/efc36cd2e8514157ad48677230969e96/results.json`, exit zero.
- Before event-filter shutdown guards, the minimal OM9 workspace exited with access violation: `build/workspace-lifecycle-minimal-1/51fe5c1e8d8a46fbab4d580e2c478a54/startup.log`. The log identifies a QWidget event after leaving the event loop.
- CoreWorkspace now skips window lookup for unrelated events and skips layout/reconstruction during application/document closure. After compiling and linking a separate module, `tests/core_workspace_lifecycle_smoke.FCMacro` passes both assertions and exits zero: `build/core-workspace-lifecycle-isolated-1/7b8c4ef9a698460e81e64ecc26d6d6a4/results.json`.
- Full tab regression still fails process exit despite 28 assertions: `build/view-tabs-lifecycle-isolated-1/46fe0e94ef314791953ad5b9958afe8d/results.json`, exit -1073740791. Its log reports missing native camera. This is an unresolved distinct lifecycle failure, not a completed feature gate.

The user's open preview remains separate and has not been closed or overwritten. Physical mouse interaction is not proved by the synthetic Qt mouse tests.

## Resolution of the camera failure

Bisecting the native sequence proves 18 assertions before quadrant close exit zero (`build/tabs-before-close-isolated-1/33b905fcdbd545cb930ee01d580fc495`); adding quadrant close/reconstruction reproduces abnormal exit (`78477273a6f547cab3fe52cb7f33ed59`). Native MDIView closes by detaching from its document while Qt defers widget deletion. The fixture's nested event loops do not process deferred deletion from the outer macro callback, leaving the old native view alive until after document destruction.

The fixture now explicitly sends DeferredDelete events after each settling loop. Reconstruction is scheduled from the view's destroyed signal instead of its initial Close event. Native creation also waits if an existing view has no camera during teardown. With both changes, all 28 native tab assertions pass and the process exits zero: `build/view-tabs-lifecycle-isolated-1/6b8073e47956408db5910507f687e888/results.json`. This supersedes the unresolved camera assessment above for this sequence. A further run removes explicit parameter saving to verify ordinary shutdown persistence.

Normal shutdown without forced parameter save passes 28 assertions and exits zero at `47a73e7492ce45049f0186dd9ee5a2d4`; a new process reads that user.cfg and passes29 assertions with exit zero at `be6736eedb3c4146b8611481816494d7`, both under `build/view-tabs-lifecycle-isolated-1`. Normal-exit preference persistence is therefore verified.

Clean host build: rebuilt native View3DInventor.cpp from unmodified parent source, removing injected exception tracer, then relinked fixed CommandView.cpp. Native workspace regression passed 36 assertions and exit zero at build/core-views-camera-fixed-1/655935677f6b4243b085cfd5d5db52e8/results.json; zero missing-camera warnings. Explicit Orthographic/Perspective commands still change projection; action refresh preserves Perspective user zoom; camera/grid reload, four native views, document/workbench switch and task locks pass. Clean runnable host currently build/host-camera-diagnostic/bin/FreeCAD.exe despite directory name. Existing preview processes untouched.
