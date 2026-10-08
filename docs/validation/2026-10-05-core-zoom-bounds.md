# Native bounds zoom validation

Original commands: `Zoom_Extents` (OM9-VIEW-006), `Zoom_Selected` (OM9-VIEW-007). Local specifications are authoritative; PDFs were not reread.

Rust command-name test failed before mapping implementation and passes afterward. All 32 Rust tests, cargo fmt check, Clippy with warnings denied and release build pass. Native CoreViewControls compiles and links against the patched isolated FreeCAD host.

The first 11 native assertions passed but did not prove all four projection modes. Expanded corner-projection checks exposed a clipped Perspective corner: projected y=-0.046919 at aspect2.09697, with center correct. After correcting bounding-sphere distance, all four viewport projections pass. The check was retained rather than relaxed.

Final native run: `build/zoom-bounds-isolated-1/191e8b2008cf4c36871b8fdf66289194/results.json`, 23 assertions, exit zero. Fixture: `tests/core_zoom_bounds_smoke.FCMacro`. Host: `build/host-camera-runtime/bin/FreeCAD.exe`; module: `build/zoom-bounds-runtime/OpenMatrix9Gui.pyd`. Normal startup log ends with successful system/user parameter saves. Coverage and limitations are documented in `docs/features/OM9-VIEW-006-007.md`.

This validates two commands; it does not establish completion of the full 130-spec core scope.
