# Windows GUI Sphere stack regression

The two crash dumps and their limitations are described in
`2026-11-02-mcp-sphere-crash-dumps.md`. On the old Windows debug GUI, an
isolated GUI bridge received `Sphere`, then `0,0,0`; the connection reset at
the failing point and the old process terminated. No user document was used.

The Windows MSVC `om9` binary now reserves an 8 MiB main-thread stack via the
linker, instead of the 1 MiB default. Windows commits pages as needed. This
provides headroom for existing large debug command frames; it does not reduce
their size or guarantee that every other command fits. The limit is scoped to
the executable, not individual Rust worker threads or non-MSVC builds.

The standalone executable linked at `L:/om9-target/debug/om9-fixed.exe` has
PE stack reserve 8388608 bytes and SHA-256
`0b78ae12c7006910fde395d0b6c8c7a78c24a18bf406c2121377b1d222c279e4`.
`tools/validate_live_sphere_stack.py` passed against it using a separately
launched real GUI and real stdio proxy, isolated through LOCALAPPDATA: CMD
Sphere, center, radius, BRep creation, Undo, Redo preserving ID, menu dispatch
and world-point tool, and Undo. This is not a physical mouse or keyboard test,
nor a geometry measurement. The newly linked `solid` integration-test binary
passed 52 `solid_sphere` cases, covering geometry and Sphere modes.
`tools/public_source_audit.py` reported zero errors; artifact pruning removed
zero runs and retained cited runs. Cargo linked the candidate and tests but
reported access denied when copying over the running `debug/om9.exe`; the test
executable was run directly. After confirming the old document was empty, its
GUI was closed normally, the tested candidate copied to `debug/om9.exe`, and
the desktop OpenMatrix9 MCP shortcut changed to that GUI executable. The
existing `om9-mcp.exe` proxy remained in use. The restarted normal GUI was
reached through the existing Codex MCP connection; Sphere, center `0,0,0`,
radius 5, creation and Undo succeeded. The resulting empty Untitled document
has one Redo entry from the test. Release packaging was not tested.

Next: measure and reduce large command stack frames separately, and verify
physical CMD/mouse paths and release packaging. The unrelated older
`OpenMatrix9` shortcut launches the historical FreeCAD launcher and was not
changed.
