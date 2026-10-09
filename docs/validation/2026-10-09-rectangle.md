# Rectangle native validation — 2026-10-09

Scope: [OM9-CURVE-004](../features/OM9-CURVE-004.md), sharp Rectangle supported
slice. Rounded Arc/Conic remains unsupported.

## Evidence

- Source read: Matrix8 Book1 printed127–128 / PDF137–138, through the Rectangle
  continuation before Circle.
- Rust semantic suite: `rectangle_session`, 12/12 passed. Eight initial tests
  failed because Rectangle was unsupported. Reviewer regressions for adjusted
  endpoint overflow and Shift with early Width failed before their fixes and
  passed afterward. Existing Curve tests: 18/18; Curve write-permission test
  also passed.
- Full Rust checkout: 128 tests in 27 suites passed at final verification,
  including concurrent Edit/Solid/Surface work.
- Python tooling: 23/23 passed.
- Clippy `--lib`: exit0, 20 existing/concurrent warnings; no Rectangle geometry
  warnings. New geometry module passes rustfmt check.
- Native Rectangle: 36/36 passed, process exit0. Report:
  [results.json](../../build/rectangle_smoke-1/364e1332381546b692bf7f983fcbd0ef/results.json).
  Screenshot:
  [rectangle-runtime.png](../../build/rectangle_smoke-1/364e1332381546b692bf7f983fcbd0ef/rectangle-runtime.png).
- Line/Polyline regression: 26/26 passed, process exit0:
  [results.json](../../build/curve_smoke-1/c257ecea6a5e4e5d8dcc6573307ad71b/results.json).
- Interp/Rebuild regression: 40/40 passed, process exit0:
  [results.json](../../build/curve_spline_smoke-1/8af8358f2d3245549c778dfaa1f8a2dd/results.json).

The native macro exercises menu/CMD/sidebar/F6; closed four-edge valid Part
wires; full Center dimensions; slanted 3Point; signed Vertical widths; units;
recoverable errors; session Undo; document Undo/Redo; idle prompt; preview with
no object edits; exact preview/click agreement; Shift independent of Ortho and
after early Width; Vertical mouse rays; document/workbench cancellation; FCStd
geometry and feature-ID persistence. Explicit Qt mouse-motion events make the
hover check deterministic when the smoke window has no OS focus.

## Build and runtime

- Source `H:/FreeCAD-src/Mod/OpenMatrix9`.
- CMake build `H:/FreeCAD-src/build/openmatrix9-final`.
- Matching SDK `H:/FreeCAD-src/build/relWithDebInfo`, MSVC2022 and Qt6/OCCT8
  dependencies under `H:/FreeCAD-src/.pixi/envs/default/Library`.
- Built native module into the existing isolated runtime output directory.
  Smoke used SDK `bin/FreeCAD.exe` with `-P` pointing to that module directory
  and a fresh profile. The older isolated executable failed before macro
  execution with Windows DLL initialization error; it is not validation proof.
- Build blockers in concurrent Surface sources were limited to `.h` versus
  `.hxx` OCCT headers, a missing array header and SDK include path; corrected
  without changing Surface behavior. A transient Edit signature/link failure
  disappeared after its concurrent implementation became consistent.
- Ordinary CMake output configuration was restored after the successful build.

## Limits

Public source audit verifies 510 authored icon bindings but reports 290
preserved reference/binary artifacts under the spec package and `code/`.
Full report: `build/rectangle-public-source-audit.json`. This task neither
imports those artifacts into runtime resources nor publishes the repository.
Git whitespace check with `core.whitespace=cr-at-eol` passed.

The captured main-window screenshot contains OpenGL capture artifacts in
the viewports. Geometry and preview coordinates were verified independently;
this capture does not certify viewport rendering quality.

Evidence establishes the documented Rectangle slice. It does not establish
Rounded/Conic, Matrix History, active layers or exhaustive Rhino compatibility.
