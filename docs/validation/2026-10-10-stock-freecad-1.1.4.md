# Official FreeCAD 1.1.4 compatibility — release 0.0.2

The stock Windows x64 host at `C:/Program Files/FreeCAD 1.1` reports FreeCAD 1.1.4, commit `4fd3bf320d9566a27e60069fc8387448aaa3a094`, Python 3.11.14, Qt 6.8.3 and OCCT 7.8.1. This differs from the development 27.1 host used for 0.0.1; its native module cannot be reused unchanged.

## Build scope

Only the standalone plugin, Rust core, openNURBS dependency and import worker were built. A header/import-library SDK was assembled from exact official 1.1.4 source and the installed DLL exports; matching dependency headers and libraries were used. No FreeCAD root target was built, and no host runtime was copied. The existing standalone Ninja cache was reused, with 60% of logical CPUs (14 workers on this machine).

Portable state and validation remain in Rust. C++ changes are required stock FreeCAD/Qt/OCCT adapters: old Boost signal API, transaction/control signatures, camera serialization, native selection and bounds. Python is bootstrap/test glue. The PY_SSIZE_T_CLEAN define is set before native headers for Python 3.11 byte-buffer calls.

OM9 resources and the worker are resolved from the actual user plugin folder. Host DLLs remain in the official FreeCAD installation. The native test oracle was rebuilt separately against the matching OCCT ABI; the eight existing BRep fixtures and analytic tolerances were retained.

## Fresh measured evidence

Exact native binary hashes are recorded with each run. [Summary and individual reports](stock-freecad-1.1.4/summary.json) cover 10 groups / 217 checks:

|Group|Checks|
|---|---:|
|Official host load, workbench, resources, default palette, Undo/Redo|12|
|Layer panel|27|
|Clipboard user workflow|26|
|Curve editor|11|
|Curve Join|6|
|Eight BRep cases, analytic mass, FCStd reopen and current export|45|
|Single-document native creation, Copy, Undo/Redo, FCStd|19|
|Fresh native import worker vs direct preparation|5|
|View controls|20|
|Real mouse selection, Line/Polyline, cancel and camera restoration|46|

The mouse test identifies the actual last geometry object rather than `doc.Objects[-1]`: the stock host creates a layer-storage object after geometry. Native subedge window containment uses exact Part subshape bounds. Existing workflow assertions remain intact.

The same module was installed in the versioned user Mod folder and passed 12 discovery checks without the `-M` plugin search flag. Test configurations remain separate from user settings. A desktop-context installation plus a direct physical-volume read matched all 536 runtime files, avoiding reliance on Codex's AppData virtualization. Actual extracted-ZIP startup passed 12 further checks, and the release-mode Rust regression passed 262 tests. Packaging evidence and hashes are recorded in the release's `release-validation.json`.

## Limits

FreeCAD 1.1 globally commits a draft in a previous document when a transaction starts in another document. A baseline test reproduced this with **no OM9 module loaded**. The accepted stock workflow uses a single active document and finishes/cancels a modeling command before switching documents; it does not claim the newer development host's independent pending-document behavior.

Historical Rhino/Matrix reports are not fresh certification of the stock binary. Full openNURBS, production Matrix palette/lock handoff, every jewelry command and advanced Hatch workflows requiring patched FreeCAD core remain incomplete or unverified. The stock compatibility run does not close those implementation packages. Seven open/in-progress openNURBS packages remain; the complete future batch total is unknown. Source/resource audit and local ZIP verification passed; GitHub asset digest verification is the remaining publication gate.
