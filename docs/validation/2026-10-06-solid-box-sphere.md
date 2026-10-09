# OM9-SOLID-012 / OM9-SOLID-014 — native validation, 2026-10-06

Box corner-to-corner/height, typed dimensions with Enter defaults, 3Point and
Sphere center/radius have validated native implementations. Both features remain
`partially_implemented` with `validated_supported_slice`; extended options are
listed in their spec files. This record does not assert full Matrix/Rhino parity.

## Source and implementation boundary

Matrix 8 Book 1 printed pp.222–223 / PDF pp.232–233 were extracted and visually
checked for Box; printed p.223 / PDF p.233 for Sphere. Box continues onto the next
page and ends before Sphere; Sphere ends at Ellipsoid on the same page. The local
captured source sections are preserved. Host units, limits, transaction model,
scene preview, metadata and standalone ownership are documented as OpenMatrix9
decisions, separately from the manuals.

## Native build and runtime

- Source: `H:/FreeCAD-src/Mod/OpenMatrix9`.
- Matching FreeCAD SDK: `H:/FreeCAD-src/build/relWithDebInfo`.
- Dependency prefix: `H:/FreeCAD-src/.pixi/envs/default/Library`.
- MSVC 2022 x64, CMake/Ninja, Qt 6, `RelWithDebInfo`; build tree
  `H:/FreeCAD-src/build/openmatrix9-final`.
- Target: `OpenMatrix9Gui`; the full FreeCAD tree was not rebuilt.
- First validated isolated output:
  `H:/FreeCAD-src/Mod/OpenMatrix9/build/solid-runtime/OpenMatrix9Gui.pyd`.
- Final standard SDK output built successfully:
  `H:/FreeCAD-src/build/relWithDebInfo/bin/OpenMatrix9Gui.pyd`.

The configure command selects the existing SDK/dependencies and optionally sets
`-DOPENMATRIX9_RUNTIME_OUTPUT_DIR=H:/FreeCAD-src/Mod/OpenMatrix9/build/solid-runtime`.
The final standard build clears this option. Both build helpers initialize the
MSVC x64 environment before invoking CMake. Nonblocking Vulkan header discovery
warnings did not prevent compilation or linking.

Reproducible native runner for the standard SDK:

```powershell
rtk proxy powershell -NoProfile -File tests/run_menu_smoke.ps1 -FreeCADExe H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library -Macro solid_commands_smoke.FCMacro -TimeoutSeconds 180
```

For isolated validation add `-NativeModuleDirectory
H:/FreeCAD-src/Mod/OpenMatrix9/build/solid-runtime`. The runner isolates settings
and restores environment variables. Its native-module override is optional.

## Passed evidence

| Check | Result | Report under module root |
|---|---|---|
| Solid Box/Sphere, isolated runtime | 70/70, process exit 0 | `build/solid_commands_smoke-1/22155c12fb3d47b48dbe06f13dc093e6/results.json` |
| Solid Box/Sphere, standard SDK runtime | 69/69, process exit 0 | `build/solid_commands_smoke-1/22044e17980f4ca1aa3ce395ef4023f6/results.json` |
| Line/Polyline regression | 26/26, process exit 0 | `build/curve_smoke-1/8627e53b05304b19b9133202f93bb9bc/results.json` |
| Interp Curve/Rebuild regression | 40/40, process exit 0 | `build/curve_spline_smoke-1/86d2b813f5a044ab89692ab551515e41/results.json` |
| Sweep1/Sweep2/Loft regression | 96/96, process exit 0 | `build/surface_commands_smoke-1/3a91b633429d4f41a2119827f827ef11/results.json` |
| Rust full test run | 100 passed in 26 suites | `rtk cargo test --manifest-path rust/Cargo.toml` |
| Python tooling tests | 23 passed | `rtk proxy python -m unittest discover -s tools/tests` |
| Solid Rust formatting | passed, edition 2024 | `rtk proxy rustfmt --edition 2024 --check rust/src/solid.rs rust/tests/solid_session.rs rust/tests/solid_commands.rs` |
| Whitespace, Windows CR-at-EOL allowed | passed | `rtk proxy git -c core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol diff --check` |

The one-count native difference is the explicit isolated-module-path assertion;
both reports record the actual loaded module. Reports were read with `ok=true`
and every check passing. Native fixtures use the actual Solid button/menu, shared
CMD, Qt keys and mouse events, Part/OpenCascade objects and FCStd persistence.
Other Edit/Curve/Surface changes were already in this shared workspace; their
source and documentation were preserved.

Solid evidence covers analytic Box volume/orientation/negative extents, Sphere
surface center/radius/volume, Enter defaults, scene-only preview and cleanup,
invalid input, input Undo, native Undo/Redo, metadata after reload, success-only
history and Enter repeat, document close and workbench lifecycle, point Osnap,
independently computed mouse projection and Box ray/height-axis constraint.
Coordinate CPlane frames, repeated input Undo and relative last-point behavior
also have Rust fixtures. Typical geometry tolerance is 1e-7 mm; native mouse
comparisons allow 1e-5 mm and scaled volume error. Sphere's tessellated bounding
box is not used to infer the exact analytic radius or center.

Native viewport captures `sphere-preview-viewport.png` and `solid-viewport.png`
were visually inspected: native purple wireframe solids/preview are visible.
The ordinary QWidget window grab does not reliably capture OpenGL contents, so
native viewport image capture is used for geometry evidence.

## Review corrections and limitations

Independent review found two P2 errors: Esc retained an unsubmitted CMD suffix,
and F4 retained the previous viewport's CPlane before the first point. Both were
reproduced by native tests in
`build/solid_commands_smoke-1/86136117aa1a4d7082f9e851e1178e6d/results.json`, then
fixed through shared `cancelInput()` cleanup and active-frame refresh. The final
native runs cover both regressions; follow-up review found no actionable issue
in these fixes.

The global checks below are not passing and must not be presented as clean:

- Strict Clippy (`cargo clippy --all-targets -- -D warnings`) reports 16 issues
  in `core_snaps.rs`, `curve.rs`, `spline.rs`, `spline_ffi.rs`, `edit.rs` and
  `surface.rs`, including chunk/loop/style lints and missing unsafe-function
  safety documentation. None is reported in `solid.rs`. Full output:
  `build/solid-clippy.txt`.
- Public-source audit reports 290 private/binary artifacts: 3 supplied Matrix
  PDFs in the spec package and 287 files under the separately present `code/`
  reference tree. All 510 authored SVG bindings and SVG files passed their icon
  checks. Earlier in this session the audit listed only the three PDFs; the
  reference tree appeared during concurrent workspace work. Files and audit
  rules were preserved. Snapshot: `build/solid-public-source-audit.txt`.
- Raw `git diff --check` treats newly added CRLF lines in shared Windows files as
  trailing whitespace. The check allowing CR-at-EOL is recorded separately;
  no global Git configuration is changed.

An earlier Rust run transiently failed the unrelated
`edit_session::om9_solid_004_exactly_two_and_five_modes` while concurrent Edit
implementation was changing; later full runs passed all 100 tests. A transient
native Edit adapter signature mismatch also disappeared after the corresponding
source update. A locked SDK output initially required isolated native linking;
the final standard SDK link and native run succeeded.

No production publish, source export, commit or push was performed.
