# Current Matrix Rust palette replay slice

Continuation of the approved layer-session handoff plan. User amendment on 2026-10-10: require Undo/Redo only in OM9; Matrix is a transfer endpoint. This replaces earlier Matrix Undo/Redo gates and custom callbacks; other transfer requirements remain.

Ruling: the separately built adapter transfers full palette metadata in the current Matrix session through the existing RunPythonScript command. Python only loads exact binaries and dispatches the native adapter. It does not reload Matrix, launch Rhino, install a plugin or edit registry. C# performs RhinoCommon/Win32 operations; metadata validation, full-path/source-wins merge, binding, stale-state checks and persistent-presence interpretation use the same Rust core as OM9. Receive requires the native command context, but does not require or override Matrix Undo recording.

The current implementation is a prepared transfer bridge API, not finished OM9 toolbar/shortcut deployment. Original Tasks1–5 otherwise remain incomplete. The native gate requires both transfer directions, OM9 receiving Undo/Redo, atomic rollback after palette/geometry errors, and owned cleanup. No custom Matrix Undo/Redo callbacks or tests are included. Existing callbacks from previous diagnostic adapters are detached on bootstrap without loading plugins or changing native history.

Files: bridge/rhino5/rust-core (separate cdylib wrapper, existing OM9 Rust policy), bridge/rhino5/OM9LayerTransfer (RhinoCommon and Win32 adapters), tests/rhino5_verify_matrix_om9_palette.py (current Matrix + owned OM9), tests/rhino5_matrix_layer_receive.py (existing native command bootstrap), tools/build-matrix-layer-bridge.ps1.

Checks: five semantic Rust cases RED→GREEN; four ABI/presence cases; strict Clippy/fmt; actual x64 C#→Rust process; installed Rhino5 C# compilation and IronPython syntax. These are preparation, not native application acceptance.

Application replay: in a dedicated fresh blank unsaved Matrix document run `_-RunPythonScript "H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5_verify_matrix_om9_palette.py"`. Source colors/unused/nested/empty palette and own layer/object flags are strict oracles. Actual OM9 public clipboard and document Undo remain unchanged. The receiving Matrix native command executes two failure injections and successful receive, without issuing Matrix Undo/Redo. Cleanup tracks exact owned native IDs and restores original palette/active/persistent witnesses.

Remaining original scope includes Selected/Session in both directions, file transfer, preexisting content and rollback history ownership, retained definitions/instances, remaining native mutation routes, FCStd/re-read, heavy metadata performance, Phase1–3 regressions, final review and packaging. Complete future batch count remains unknown.
