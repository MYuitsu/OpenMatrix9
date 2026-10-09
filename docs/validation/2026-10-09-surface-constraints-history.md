# Surface constraints, Refit, Add Slash and History — 2026-10-09

Verified supported slices: OM9-SURFACE-001/002/003/004/009 and the Surface
dependency graph within OM9-HISTORY-001. Full features remain partial.
The user explicitly requested these missing options and spec_v1 synchronization.

## Reference and implementation

Matrix8 Book1 PDF180–183 / printed170–173 (Sweep1 and History), PDF184–186 /
printed174–176 (Sweep2 and History), PDF193–195 / printed183–185 (Loft).
The supplied PDF SHA256 is
`8595cc94deaf53144210686439de29b90aa776472eaee0271d34d767cab6e246`.
Manual continuations were inspected. Graph detachment/Undo behavior comes from
the exact OM9-HISTORY-001 shared contract, not an invented Addendum chapter.

Rust owns adaptive section fitting and bounded paired-rail parameter mapping.
C++/Qt owns native support-face constraints, curve hull bounds, picking and
document dependency execution. Python is bootstrap/host test code. Sources are
copied independently before geometric operations. Exact algorithms/defaults and
remaining combinations: [advanced contract](../features/surface-advanced-options.md),
[Surface History graph](../features/surface-history-workflow.md),
[Sweep1](../features/OM9-SURFACE-001.md), [Sweep2](../features/OM9-SURFACE-003.md),
[Loft](../features/OM9-SURFACE-009.md),
[History002](../features/OM9-SURFACE-002.md), [History004](../features/OM9-SURFACE-004.md).

## Native build and runtime

Matching MSVC x64/Ninja/Qt6/OpenCascade SDK in `H:/FreeCAD-src/build/relWithDebInfo`,
dependency prefix `H:/FreeCAD-src/.pixi/envs/default/Library`. Isolated native
target `cmake --build H:/FreeCAD-src/build/om9-surface-options --target OpenMatrix9Gui --parallel 4`
completed with exit0. The module now links the native Part SDK for persistent
`OpenMatrix9Gui::SurfaceHistory`; workbench initialization loads Part first.
Build emits OCCT deprecated-header warnings.

Validated module: `H:/FreeCAD-src/build/om9-surface-options/bin/OpenMatrix9Gui.pyd`.
SHA256: `1c0e6d930d3a5f91683fec1d72136e9619e3d800dea652ec074c6f0b9bb69b49`.
Each main macro asserts its actual import path. Validation uses private profiles,
documents and processes; it does not replace an already-loaded user module.

Run from `H:/FreeCAD-src`, substituting the Macro name for each suite:

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File Mod/OpenMatrix9/tests/run_menu_smoke.ps1 -FreeCADExe H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library -Macro surface_advanced_smoke.FCMacro -NativeModuleDirectory H:/FreeCAD-src/build/om9-surface-options/bin -TimeoutSeconds 120
```

All six final native suites exit0, **530/530 checks passed**. Artifact paths
below are relative to `Mod/OpenMatrix9`.

| Suite | Passed | Artifact |
|---|---:|---|
| Advanced Refit/Slash, production History UI/CMD, closed defaults/lifecycle | 108 | `build/surface_advanced_smoke-1/ba00b512629c49a8881fe191d3a94d87/results.json` |
| G1/G2 and Loft endpoint tangents, invalid-G0 crash regression | 77 | `build/surface_constraints_smoke-1/cafc4a45c3584f36a90d34c865c986cb/results.json` |
| Native dependency graph/restore/recovery | 65 | `build/surface_history_smoke-1/416db955978e4a199dc72f32deb8b7f9/results.json` |
| Cold FCStd restore without workbench activation | 3 | `build/surface_history_smoke-1/272d273abd1141fa9495ed81caf7a303/results.json` |
| Existing Surface option regression | 180 | `build/surface_options_smoke-1/4ccd74c2744044d6b0062bc82e108583/results.json` |
| Existing Surface baseline regression | 97 | `build/surface_commands_smoke-1/095f6e3de0c74bae9fb9ba5c96bb5a8c/results.json` |

Cold restore sets `OM9_HISTORY_RESTORE_FILE` to the main History suite's
`surface-history-3.FCStd`, invokes the same macro in a fresh process/profile,
then restores that environment variable. The native module was absent before
opening the document, restored the correct native type automatically, and a
source edit changed area by the independently expected factor5/3.

Other checks: full `cargo test --no-fail-fast` **164 passed /32 suites**, including
the concurrent Circle work; Python tools **23 passed**; `cargo clippy --lib`
exit0 with23 warnings; whitespace diff check passes. New Refit/Slash FFI functions
document their pointer safety contracts. The whole working-tree public source
audit validates510 SVG bindings but rejects290 existing local PDF/VB/binary
artifacts; report `build/surface-advanced-public-source-audit.json`. Sources were
preserved and excluded from the authored spec manifest.

## Geometric and lifecycle evidence

Refit original open spline sections stay within0.03mm and periodic circle
sections within0.01mm; original BReps remain unchanged. A tolerance1e-7mm at
translation1e8mm is refused, and0.01mm recovers. Native acceptance uses continuous
positive-weight hull/chord bounds with coordinate-scaled padding and bounded
subdivision; it is numerical OCCT evidence, not interval-arithmetic certification.

Slash(.25,.65) adds a real transported arch section whose independently computed
interior crown lies on the output within1e-4mm, while both original rails retain
contact. Crossed pairs are rejected without erasing the last valid pair. Real
viewport picks populate the paired arc fractions; Enter finishes picking,
removal works and Cancel creates no object. Undo/Redo/FCStd preserve settings.

G1 normals have maximum angle below3.8e-7rad. Nonzero-curvature G2 fixtures have
normal errors below3.5e-7rad and curvature errors below3.6e-6/mm across both rails.
Different transverse support curvature permits G1 but rejects G2, proving that
the curvature choice is not just metadata. Both Loft endpoint normal angles are
below1.5e-6rad. The tests independently inspect support/output derivatives,
original curve contact, parent world placement and unchanged original geometry.
Rational profiles, bare curves and ambiguous two-face edges disable unsupported
controls. Incompatible profiles reject, recover and cancel without output.

History tests cover all three kinds, two generations, source changes, nested
parent frame changes, input Undo/Redo, malformed settings and source recovery,
direct result edit detachment with descendants retained, Undo restoring parents,
cycle/foreign-document refusal, suspend/resume, deletion invalidation, chained
rail recomputation and FCStd. Production checkbox/CMD commits retain exact IDs
002/004; their closed torus/annulus cases preserve analytic areas and the
conditional Closed Yes default. Sweep2 History one-profile Maintain Height
defaults Yes and recomputes after a changed section.

## Failures resolved and review

Initial native RED reports established missing Refit controls, continuity controls
and native History type. Rust RED fixtures established missing fitting/mapping APIs.
SDK filling initially rejected the public shape-enum G2 ordinal: its plate bridge
expects raw order2. The adapter now uses actual plate order and independently
tests the complete normal-curvature operator and a conflicting-curvature case.

Restore originally detached links when deferred Shape restoration occurred after
the object restore flag cleared. Document-level restore guards fix both normal
and cold FCStd cases. Independent review found a narrow-parameter shortcut in
the hull bound and fixed absolute roundoff padding; the shortcut was removed and
scale-aware allowances added. Review of world-space Slash picking and History
integration found no remaining concrete blocker within the documented bounds.

A disjoint box edge with the ordinary G0 Sweep2 preview crashed this SDK. A
pre-solve original-profile contact guard now rejects the exact same edge and
Reverse callback safely; the77-check regression includes it. The mouse fixture
also exposed fixed viewport-slot camera behavior: using the actual Front slot
instead of a Z-collapsed Top view gives the intended interior rail picks.
Concurrent unrelated include/compile changes were resolved before the final
native build. No source removals, commits or publication were performed.

## Remaining scope

G1/G2: eligible open rectangular patches, original mode,2–32 matching nonrational
profiles; no fitting/Closed/height/Slash combinations. Loft tangency: eligible
Normal/Tight open profiles and support edges. Slash: one profile/open rails only.
Multiple-profile Maintain Height, Preserve First/Last Shape, Road-like orientations,
blending/miters, Simple Sweep/Refit Rail, SplitAtTangents, Point endpoints and dragged
seams remain unsupported. Global/all-command Matrix History, dedicated History
menu/F6 placement and persistent topological naming compatibility remain unverified.
Whole-surface continuity/contact and exact proprietary solver parity are not claimed.
