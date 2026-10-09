# Edit, History and special Explode validation — 2026-10-09

Status: validated_supported_slice. The scoped native work is complete.
Broad feature statuses remain partially_implemented; these fixtures do not
establish full Matrix/Rhino compatibility or publication readiness.

## Final SDK reports

| Fixture | Checks | Evidence |
|---|---:|---|
| History / Join + Surface global policy | 83/83 | [Report](../../build/history_smoke-1/0a28ceafabe247a8ad0389272225c714/results.json). |
| Latest History cold restore | 4/4 | [Report](../../build/history_smoke-1/2e27955c3b79451a8ee7c47bcb1a8bff/results.json). |
| Retained 3DM special Explode | 40/40 | [Report](../../build/edit_3dm_special_smoke-1/2f68c35673a44bfba6f26c318c833ea7/results.json). |
| Native special Explode | 25/25 | [Report](../../build/edit_special_types_smoke-1/b52b0d2757dc464c8b6ccd91a864d5ef/results.json). |
| Core Edit regression | 115/115 | [Report](../../build/edit_commands_smoke-1/2d83a4f497de4cc48327549ca2f29e87/results.json). |
| Edit options regression | 57/57 | [Report](../../build/edit_options_smoke-1/6b25f835369b4e90be86b7a5e9e96aa2/results.json). |
| Surface History regression | 65/65 | [Report](../../build/surface_history_smoke-1/4cc81a475b094b43bd05e6e8f9f674bb/results.json). |
| Curve command routing regression | 27/27 | [Report](../../build/curve_smoke-1/d5a4281e069847ab9e26e095357bd90c/results.json). |
| Surface command routing regression | 97/97 | [Report](../../build/surface_commands_smoke-1/007a7bd4878d4d2982dcdd9e12be9a7b/results.json). |

Native History/special/3DM reports identify the actual SDK
H:/FreeCAD-src/build/relWithDebInfo/bin/OpenMatrix9Gui.pyd.
Cold restore starts with the namespace module absent and verifies its load,
native links and subsequent descendant updates. Earlier isolated-module
and partially failing reports are superseded by the final SDK reports above.

## Behavior and focused review

History83 covers retained separate curve parents, two generations, Record and
Update suspension/resume, global policy on existing SurfaceHistory, local surface
suspension, Lock, selected Clear, detach warning, ordinary Edit detach/Undo and
FCStd persistence. It verifies grouped-parent translation/rotation,
frame replacement/reparenting, frame Undo, disconnected/null-source propagation,
recovery and Undo. Parent-frame tracking uses native hidden links and document
notifications; callbacks disconnect on teardown. Failed Join recompute clears
geometry while retaining records, and rejects invalid parents, so active
descendants cannot consume an old valid shape. No remaining concrete blocker
was found in the supported slice during focused review.

Native special25/25 includes block menu protection versus explicit CMD, source
definition retention, grouped placement, glyph outlines, Center/Right text
layout/spacing, Label leader/frame graphics, Screen-mode atomic rejection,
linear and analytic angular dimensions, editable detached labels, native cage
components, font rejection, stale annotation options and individual geometric
member styles. Style fixtures compare RGB channels and retained source metadata;
FreeCAD may assign unique display labels such as A001/B001 while source metadata
retains A/B.

Retained3DM40/40 covers embedded nested blocks with reflection/nonuniform scale
and file units, immutable source archives, Undo/Redo and FCStd, integrity guards,
detached editable dimension text/frame, current NURBS cage boundaries/control
lattice, CV weights/knots/unit/source UUID descriptors, curve/surface/cage
morph controls and font contours. It does not reconstruct captive-object
deformation History. Native arrows/Label graphics use Coin primitive outlines;
curved arrow outlines are tessellated graphics, while angular arcs retain
analytic curves. Rich per-face/per-span style and exhaustive font/layout cases
are unverified.

## Other checks and source-audit limit

The implementer reports fresh Rust166 tests in32 suites and Python23 tests
passed, with matching SDK native build success. The reviewer inspected saved
reports and code; no native build, GUI test, Rust test or Python suite was rerun
by the reviewer.

[Source audit](../../build/edit-history-special-source-audit.json):
12,226 source files,510 icon bindings and510 SVGs;290 errors remain.
The root audit fails on preexisting private/reference/binary artifacts retained
in the workspace. This is an unresolved publication boundary, not a clean
audit result. No publication or deletion of those artifacts was performed.

Full Matrix/Gem Builder History, captive deformation reconstruction, linked-only
block definitions, custom arrow blocks, view-dependent annotation, unavailable
fonts, exhaustive topology/layout and rich source styles remain outside the
supported slice or explicitly rejected. The native special adapters preserve
current geometry; ordinary Edit snapshots break affected recorded links.

See [native Edit contract](../features/edit-native-contract.md),
[native History contract](../features/history-native-contract.md) and
[live ledger](../openmatrix9-progress.json) for ownership, lifecycle and limits.
