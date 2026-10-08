# End Snap native validation

Status: in progress; this evidence does not complete all snap or 01-core requirements.

The CMD toggle persisted State=2 but initially left the native checkable menu action unchecked. CoreSnaps now synchronizes the registered action with setBlockedChecked after state changes and preference loading, avoiding command re-entry.

CoreSnaps.cpp compilation and snaps-runtime linking succeeded. Native test tests/core_end_snap_smoke.FCMacro passed eight checks and the host exited with code 0. Evidence: build/end-snap-isolated-1/ff27e581a59443efb168761454f1e5e5/results.json.

Verified original Osnap E registration, CMD/menu shared state, exact mouse endpoint outside the C-plane, master-off plane fallback while preserving End mode, hidden-object exclusion, and no object/Undo/selection changes.

Expanded native regression passed 12 checks with exit code 0: build/end-snap-isolated-1/f648f83ce69c43d89b0d5817f10aceab/results.json. It additionally proves translated object placement, interior joined-polyline vertices, a closed circle seam and a planar surface corner using actual mouse input.

Further native regression passed 14 checks and exited normally: build/end-snap-isolated-1/8df0a69b743a4e8590e0290d582f0183/results.json. Box polysurface corners and a rotated line are verified. Rotated line endpoints project to the same pixel with world distances 10 and 5 mm; either exact endpoint is accepted, while C-plane fallback near zero fails the assertion. Occlusion preference at coincident projected endpoints remains unspecified and unverified.

Main O-Snap sidebar switch added at OM9OsnapMaster, sharing Rust state and native preference persistence with CMD. Native 17 checks passed with exit code 0: build/end-snap-isolated-1/525ba98a172c47638fd9f76462f51c19/results.json. CMD updates the switch; mouse toggle preserves End selection and restores master mode. CoreSnaps and MatrixSidebar compilation and module linking succeeded.

Remaining: nested geometry and links, text corners, general polysurface corners and closed seams, other snap modes, and integration with each point-input command. Existing user preview processes retain their previously loaded modules.

Final stylesheet correction verified: 17 checks and exit0 at build/end-snap-isolated-1/b1968eaae9d840fd8d3c75c33d978a79/results.json; startup log contains zero stylesheet parse warnings.

Workbench reactivation and native preference state added to main fixture: 19 checks and exit0 at build/end-snap-isolated-1/2911d9b062da40bb83e855604884568d/results.json. A separate new FreeCAD process reads that run's saved user.cfg: three restart checks pass and exit0 at build/snap-restart-isolated-1/13ab6cc27b5740c4827004203558b691/results.json. State=1 restores master on and independently End off. Fixture tests/core_snap_restart_smoke.FCMacro does not seed preferences itself.

Nested App::Part placement regression initially failed at build/end-snap-isolated-1/98bb8d79ad4f42f087a11be8a71df913/results.json. Extractor now applies globalPlacement * inverse(localPlacement) to already locally placed vertices. Native 20 checks and exit0 at build/end-snap-isolated-1/06eb59679bd64145807af793519b6c0d/results.json confirm a 20mm parent translation yields endpoint distance30mm. Rotation assertion compares numeric distance within1e-9mm to allow floating-point transform roundoff; plane fallback still fails. Parent-hidden visibility and links remain unverified.

Hidden App::Part parent regression failed at build/end-snap-isolated-1/cd9dac25fe00418589c8eda7fd0c32b0/results.json. Picker now checks every geometric parent ViewProvider visibility and guards cycles before supplying candidates. CoreSnaps compile/link succeeded; native21 checks and exit0 at build/end-snap-isolated-1/08927152a4594d258abdb298a9505d7d/results.json prove hidden-parent exclusion. Links and non-geometric group visibility remain outstanding.

App::Link regression reproduced with source endpoint10mm versus instance30mm. Native geometry extraction now resolves through getSubObject with a supplied Matrix4D and checks returned Edges instead of requiring a direct Shape property. Passing native22 checks and exit0: build/end-snap-isolated-1/baf28fa1a65c443fafe9c8725a4a8dc1/results.json. A visible translated Link snaps to30mm despite hidden source. Nested/scaled/link-array instances and text still require coverage.

Hidden and uniformly scaled App::Link coverage added.24 native checks and exit0 at100%: build/end-snap-isolated-1/561382b98c614627b8c8b0c12b9c6545/results.json. Same24 checks and exit0 at200%: build/end-snap-isolated-2/5c9ace2e2e564aefb016b16cf97ca285/results.json. Scaled endpoint measurement is compared to native instance topology within1e-9mm. Explicit aperture boundary in native UI remains untested; these DPI results verify near-endpoint snapping and state/lifecycle regression.

Angle actual mouse snap verified:25 native checks and exit0 at build/end-snap-isolated-1/c48079bf67974a05bddec6469495b647/results.json. Four-point Angle uses snapped Z endpoint and reports90degrees; C-plane projection would not yield this result. PictureFrame now consults shared picker before plane fallback on mouse movement/click. Native35 checks and exit0 at build/picture-snap-isolated-1/982f66da7cdf4aeabfdb940c0f922231/results.json cover exact first-corner Z placement plus existing browser, preview, task/document/workbench, Undo/Redo and reload regression. Second-corner/hover snapped geometry and parallel-plane snap remain unverified.

PictureFrame second mouse corner snap and native single-transaction Undo verified.37 checks and exit0 at build/picture-snap-isolated-1/8bdfa35338954ea5835a2e2de0c63248/results.json. First corner -10,0,10 and snapped endpoint0,0,10 produce width10, height5 and center-5,2.5,10; near-cursor C-plane coordinates would fail these geometry assertions. Hover snap geometry remains unverified.

PictureFrame hover End Snap geometry verified by inspecting native Coin preview SoCoordinate3: four vertices, exact width10mm and elevation10mm before commit.39 native checks and exit0 at build/picture-snap-isolated-1/19f88b1acb20427ea7419c4415eadfad/results.json. This covers preview geometry rather than merely preview existence.
