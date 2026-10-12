# Actual two-application handoff test

User approved replacing the Matrix-only diagnostic with actual Matrix9 ↔ OM9 Copy/Paste, local Undo/Redo and layer/color/lock checks. The approved product layer-session plan remains incomplete.

- Run in the user's current fresh blank unsaved Matrix document. Never launch Rhino, load a `.rhp`, register a command or reload Matrix plugins.
- Launch one owned OM9 test companion with the existing selected standalone plugin and shared `relWithDebInfo` FreeCAD host. Isolate profile/documents and close only that companion at completion. No host build or runtime copy.
- Use Matrix's native `CopyToClipboard`, `Paste`, `Undo`, `Redo` and OM9's actual `ThreeDmClipboard` public pipeline plus document Undo/Redo. A bounded file queue coordinates calls on each GUI thread, outside the bootstrap command. No synthetic receive or state projection to make tests pass.
- Test native point/curve from Matrix and edited BRep/curve/locked-hidden geometry from OM9. Compare complete palettes, empty/nested layers, RGB, active layer, constrained/desired states and object own flags separately; collect per-operation timing.
- Compare one whole receiving operation against exact before/after native state. Observe and report missing state even if geometry transfers. PASS requires both directions, their Undo/Redo and owned cleanup.
- Restore the dedicated blank Matrix document's original palette/active state and delete only tracked fixture/received IDs. Clear its test history only after exact state comparison. Stop on saved/switched/foreign geometry, preserve evidence and never blind Undo after failed Paste.
- Python is application-test/bootstrap glue for installed Rhino5 IronPython and FreeCAD APIs. This does not add product policy in Python or implement the pending Matrix Rust/C# product bridge.

Prepared checks: seven offline transport/oracle/lifecycle checks and installed IronPython3 syntax/PS syntax. Actual OM9 companion smoke: eight checks, source/runtime hashes recorded. Actual user Matrix roundtrip pending; no two-way product acceptance claimed.
