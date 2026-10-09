# Surface options — native validation 2026-10-09

This earlier option snapshot is retained as historical evidence. Advanced
constraints, Refit, Slash and associative History were subsequently implemented
and verified; the [advanced validation record](2026-10-09-surface-constraints-history.md)
supersedes this record's remaining-option list and records the current module.

Feature IDs: OM9-SURFACE-001, OM9-SURFACE-003, OM9-SURFACE-009.
User chose additional options for Sweep 1, Sweep 2 and Loft and requested
verified work be synchronized into spec_v1. Full features remain partial.

## Reference evidence

Matrix 8 Book 1: Sweep 1 PDF180–183 / printed170–173;
Sweep 2 PDF184–186 / printed174–176; Loft PDF193–195 / printed183–185.
Continuation text and the Maintain Height illustration were inspected.
Source SHA256: `8595cc94deaf53144210686439de29b90aa776472eaee0271d34d767cab6e246`.
The supplied source stays in its existing local location; no source artwork,
commercial code or PDF was copied into native runtime resources.

## Supported changes

- Shared Automatic/Natural seam alignment, multi-edge numeric seams, independent
  source geometry copies, open/periodic cross-section Rebuild, dynamic/manual Preview.
- Chain Edges for rail selection with connectivity validation, input Undo and
  every original component reference persisted.
- Closed Sweep 1/2 with two or more sections on closed rails.
- Single-profile Sweep 2 Maintain Height, independently checked against crown
  height and both rails; dense transported sections joined using a native loft.
- Loft Loose control net, Tight centripetal interpolation, genuine Uniform knot
  interpolation including periodic/rational inputs, and bounded Developable pairs.

Detailed host algorithms, defaults, bounds and remaining options:
[Sweep 1](../features/OM9-SURFACE-001.md),
[Sweep 2](../features/OM9-SURFACE-003.md),
[Loft](../features/OM9-SURFACE-009.md).

## Build and runtime

Windows MSVC x64, Ninja RelWithDebInfo, Qt6/OpenCascade from
`H:/FreeCAD-src/.pixi/envs/default/Library`, matching FreeCAD SDK/executable
`H:/FreeCAD-src/build/relWithDebInfo`. Fresh isolated build:
`H:/FreeCAD-src/build/om9-surface-options`.

Configure includes `FREECAD_SOURCE_DIR`, `FREECAD_SDK_BUILD`,
`FREECAD_DEPENDENCY_PREFIX`, `CMAKE_PREFIX_PATH`,
`OPENMATRIX9_RUNTIME_OUTPUT_DIR=H:/FreeCAD-src/build/om9-surface-options/bin`;
reuse the existing OpenNURBS source checkout. Native target
`cmake --build ... --target OpenMatrix9Gui --parallel 4` completed successfully.
OCCT compatibility headers produce deprecation warnings.

Module: `build/om9-surface-options/bin/OpenMatrix9Gui.pyd`.
SHA256: `654cec419f4de353fede0af259d9eec143c1123dfe0f141b1f4613a07545e8bb`.
Both macros record and assert this module's actual import path. The isolated
build does not replace a module already loaded in the user's FreeCAD process.

From `H:/FreeCAD-src`, run:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File Mod/OpenMatrix9/tests/run_menu_smoke.ps1 -FreeCADExe H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library -Macro surface_options_smoke.FCMacro -NativeModuleDirectory H:/FreeCAD-src/build/om9-surface-options/bin -TimeoutSeconds 120
```

Replace Macro with `surface_commands_smoke.FCMacro` for regression.
Processes use private profiles/documents and exit 0.

| Check | Result / local artifact under Mod/OpenMatrix9 |
|---|---|
| New option geometry/lifecycle | **180/180 passed**, `build/surface_options_smoke-1/f48fabdcc6bb4b979a8928a6c4d6dd09/results.json` |
| Existing surface regression + module assertion | **97/97 passed**, `build/surface_commands_smoke-1/9ab468916424424d9bfbfe05cc05e655/results.json` |
| Surface state/options and spline geometry Rust tests | **16 passed**, 3 suites |
| Library and 26 existing integration targets | **132 passed**, 27 suites; excludes concurrent unfinished `circle_session` |
| Python `unittest discover -s tools/tests` | **23 passed** |
| `git -c core.whitespace=cr-at-eol diff --check` | Passed |
| Public source audit | 510 authored SVG bindings valid; **290 existing private/binary artifacts rejected**, `build/surface-options-public-source-audit.json` |

Native measurements include height 2 versus 3.6 mm for an arch whose rail width
changes 5→9 mm; both-rail contact; Loose interior deviation versus interpolating
styles; six Uniform sections and equal U/V distinct knot spacing; periodic and
rational Uniform interpolation; explicit nonuniform-U rejection followed by
Rebuild recovery; multi-edge seam prism area; reversed-circle source preservation;
Developable pair count/area and nonparallel rejection; open/periodic Rebuild;
connected/disconnected chains in a translated/rotated App::Part; closed Sweep
torus/annulus areas; Undo/Redo and FCStd geometry/settings/source/world placement.
Persistence placement matrices compare with 1e-9 numerical tolerance.

## Failures investigated and resolved

The old native module initially lacked these options. The first isolated test
then exposed the SDK PipeShell failure on a densely transported curved arch.
Diagnostic `build/sweep-height-probe-1/929979dc7cb04101af965f83b4aa5910/results.json`
measured 65-section PipeShell invalid area 17628 mm² while native loft of the same
sections remained valid, area 86.96677 mm² and crown error below 1e-12 mm.
The single-profile Sweep 2 strategy now consistently lofts the transported net.

Independent review found source edge orientation was lost when a periodic origin
created a forward edge, and whole-object mouse preference interfered with chain
subedge picks. Both were corrected. A deep geometry copy protects sources from
native shared-curve mutation. New reversed-circle/Loose and source-preservation
checks pass. Test harness fixes moved original snapshots after recompute, allowed
numeric placement serialization tolerance, and counted App::Part's generated
Origin objects before checking preview isolation.

A concurrent Solid header/source mismatch caused a build failure; the other
workspace change resolved it before a successful rebuild. Full `cargo test`
currently fails because concurrent `circle_session` tests reference a not-yet-
implemented `CurveSession::circle`. No Circle/Solid implementation was changed
to conceal these failures. Existing PDFs and private VB6/binary files under
`code/` account for source-audit failures and were preserved.

## Limits and review

Final read-only review found no further concrete correctness blockers. It noted
periodic/rational Uniform and failure-recovery gaps; corresponding native tests
were added and passed. Uniform repeated dense solves now have an explicit cost
bound. Further matrix factor reuse is an optimization opportunity.

Sampling is bounded numerical evidence, not whole-curve contact or unroll
certification. Maintain Height with multiple profiles, G1/G2, certified Refit,
Road-like orientations, blending/miters, Simple Sweep/Refit Rail, Add Slash,
tangent matching, SplitAtTangents, Point endpoints, dragged seams and associative
History remain unsupported. No full compatibility or viewport rendering
certification is claimed. No commit, publication or private input removal.
