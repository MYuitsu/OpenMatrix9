# Box/Sphere construction options — 2026-10-09

OM9-SOLID-012 and OM9-SOLID-014 extend the already verified basic construction flows. User scope: finish Box/Sphere options before adding another Solid command; synchronize verified work into spec_v1.

## Implemented scope

- Box: Corners/Diagonal/3Point/Vertical/Center and oriented Cube. Typed full dimensions, signed extents, inherited Enter defaults, frozen first-point frame and native icon right-click shortcut.
- Sphere: Center/Diameter, 2Point, 3Point/Radius, 4Point/Radius, Vertical, FitPoints, AroundCurve and planar Tangent. Selected vertices/control poles/mesh/cloud points, exact native bounded edges, nested placements, ambiguous-branch choice and source preservation.
- Rust owns input state and pure math; C++/Qt owns native references, matching Part/OCCT construction, scene preview and one document transaction. Tests verify analytic geometry, closed valid BRep, metadata, input Undo, native Undo/Redo and FCStd reload.

## Fresh verification

| Check | Result |
|---|---|
| Rust Box/Sphere semantic and command tests | 23 passed across 3 suites |
| Full Rust tests after concurrent Circle implementation landed | 140 passed across 29 suites |
| Python tooling tests | 23 passed |
| Matching-SDK native build, isolated output | Exit 0 |
| Matching-SDK native build, standard SDK output | Exit 0 |
| Box/Sphere extended, standard SDK | 251/251 passed |
| Box/Sphere core, isolated output | 70/70 passed |
| Edit regression, standard SDK | 115/115 passed |
| Git whitespace check with Windows CRLF policy | Exit 0 |

Native report paths (ignored local build artifacts):

- Box/Sphere extended, standard SDK: [build/solid_options_smoke-1/51d6532222c14097a757b21470c6c2e3/results.json](../../build/solid_options_smoke-1/51d6532222c14097a757b21470c6c2e3/results.json).
- Box/Sphere core, isolated native module: [build/solid_commands_smoke-1/54c0ef266ec641e8a5e2e7228a387dab/results.json](../../build/solid_commands_smoke-1/54c0ef266ec641e8a5e2e7228a387dab/results.json).
- Edit regression, standard SDK: [build/edit_commands_smoke-1/33a63dbc423a497fabfbd52b70ebfad4/results.json](../../build/edit_commands_smoke-1/33a63dbc423a497fabfbd52b70ebfad4/results.json).

Standard runtime loaded `H:/FreeCAD-src/build/relWithDebInfo/bin/OpenMatrix9Gui.pyd` without a module-directory override. Both runtime and linked development headers use OCCT 8.0.1. Build directory: `H:/FreeCAD-src/build/openmatrix9-final`; source: `H:/FreeCAD-src/Mod/OpenMatrix9`; dependency prefix: `H:/FreeCAD-src/.pixi/envs/default/Library`; SDK: `H:/FreeCAD-src/build/relWithDebInfo`. Isolated core runtime uses `Mod/OpenMatrix9/build/solid-runtime/OpenMatrix9Gui.pyd` and verifies that provenance.

The actual native Wireframe viewport capture was visually inspected: Box/Sphere output is visible with the construction grid and viewport controls. [Capture](../../build/solid_options_smoke-1/51d6532222c14097a757b21470c6c2e3/solid-options-viewport.png).

## Failure reproductions and corrections

New construction tests initially failed on the missing options before implementation. Focused review then found and reproduced frozen-Vertical and near-antiparallel-Cube regressions: both new Rust tests failed before the correction and passed afterwards. The native solid-edge reference fixture failed on the previous isolated binary before its snapshot policy correction.

Additional native failures exposed incorrect tuple packing for `getObjectInfo` and path selection unnecessarily projecting the default origin onto a circular path. Native mouse tests now pass for bounded edge picks, elevated path centers, periodic seam-crossing centers, Tangent picks and Vertical radius/width planes. Review confirmed the corrected math and native reference paths.

## Policies, tolerances and remaining compatibility

These are explicit OpenMatrix9 decisions; they are not claims of recovered Matrix solver defaults:

- mm, finite +/-1e9 mm coordinate bounds, nonzero dimensions >=1e-7 mm. Native analytic center/bounds/radius assertions use 1e-7 mm; mouse checks compare geometry with the native world pick or projected ray, allowing viewport pixel rounding. Tangent plane tolerance is 1e-6 mm, with contact/radial residuals scaled by `max(1,radius)` and normalized orthogonality threshold 1e-6.
- Cube uses minimal diagonal rotation with deterministic exact-half-turn axis. Center dimensions are full widths. FitPoints uses geometric radial least squares; coplanar points fit a circle centered in that plane. Fit batches are atomic; maximum 1024 points.
- AroundCurve `OnCurve` is a parameter fraction. Whole objects require exactly one edge, or explicit EdgeN. Links are rejected; native edge points/global placements are preserved. Ambiguous projection/cusps and stale references fail before output.
- Tangent constraints must share a plane parallel to the captured CPlane. Native exact analytic/NURBS geometry is preserved; nine bounded initial seeds for general NURBS do not guarantee every solution branch. Equal ranked branches require `Solution=N`. General nonplanar sphere tangency remains unsupported.
- Results are standalone snapshots; associative History, active-layer/Builder/Styles integration and complete shared Matrix/Rhino foundation compatibility remain unverified. Tangent has no live preview of every branch. The catalog therefore stays `partially_implemented / validated_supported_slice`.

## Global-check limitations

The first full Rust attempt encountered an incomplete concurrent Circle test API; the fresh final full run passed 140 tests. Strict Clippy still reports issues outside Solid (Circle/CoreSnaps/Curve/Edit/Spline/Surface); the latest run has no Solid diagnostics. The public-source audit still rejects supplied private reference PDFs and existing recovered binary/source artifacts under `code/`; these original inputs were preserved. This task does not claim a clean global lint or publication audit.

## Synchronization

Exact Box/Sphere specs, acceptance items, FEATURES.json/yaml, IMPLEMENTATION_STATUS.md, stats.json, README, per-feature records, ledger and guide-only manifest hashes are updated. Engineering contracts and request-planner examples remain intact; unrelated concurrent feature records are preserved. No commit, push or publication performed.

The user authorized skill synchronization on 2026-10-09 ("có cập nhật skill"). Box/Sphere rules are now routed from the repository and installed `openmatrix9-feature-port/SKILL.md` to identical `references/solid-box-sphere-contract.md` files. Existing installed 3DM guidance and the confirmation rule are preserved. Both skill-format validators passed; reference/router equality, package resolution and entrypoint links were checked. Independent retrieval initially found all four geometry policies unavailable, then correctly applied Cube/frame, coplanar FitPoints, native/periodic AroundCurve and planar Tangent/branch rules using the new reference. Skill changes do not alter native code or historical test evidence.
