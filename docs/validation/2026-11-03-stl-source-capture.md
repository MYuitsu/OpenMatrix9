# Immutable STL source capture

Follow-up: the UI now captures at invocation; see [capture-at-invocation evidence](2026-11-03-stl-capture-invocation.md). The limitations below describe the earlier capture-only delivery.

Follow-up: the originating coordinator now inventories externally retained document captures; see [owner-accounting evidence](2026-11-03-stl-capture-owner-accounting.md). Session-wide and scratch accounting remain incomplete.

P3b foundation only: `Controller::capture_stl_selected` retains the immutable `Arc<Document>`, selected object IDs, limits and coordinator epoch. `StlExportCapture::export_with_profile` reads that retained document rather than the live controller. The existing synchronous Controller export wraps this same implementation. Receipts and MCP include `source_epoch`; it is a session coordinator epoch, not a persistent file version or a geometry accuracy certificate. The capture is statically checked as `Send + Sync`.

Windows with default OCCT/openNURBS features, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: 22 STL integration tests and one in-process MCP profile test passed. The capture-specific test was then extended and rerun: after capture, another object was added, selection changed, and the originally captured object deleted; the exported bounds still matched the captured original and the current selection remained intact. This is a deterministic synchronous test, not a test of concurrent worker behavior.

The current UI still captures only when the synchronous Controller export is called after the file picker; it does not capture at initial Export invocation. Profile, target, cancellation and worker lifetime are not yet part of the capture. A capture holds the entire shared document allocation alive, including unselected entities, and is not yet registered with the managed-memory reservation system. No claim is made about 60% RAM admission, concurrent edits during a real background job, cleanup on New/Open/close, desktop responsiveness or atomic cancellation. All remain required by the plan.
