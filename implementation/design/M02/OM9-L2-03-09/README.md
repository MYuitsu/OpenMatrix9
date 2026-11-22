# M02 `OM9-L2-03-09` — Febe Matrix F6 Context Actions

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-09` (M02)

## Factory menu validation, November 2026

The [November runtime record](../../../../docs/validation/2026-11-22-f6-matrix9.md) covers the installed Matrix menu, all 424 label/script pairs and measured cage selections. Nine component manipulator macros reuse the object gumball. Set Pivot and numeric handle entry use the command session. Rotated and multi-owner fixtures verify geometry and Undo. The Matrix configuration has 79 distinct unresolved scripts after Curve Direction and Viewport to File. Curve Direction handles the four T-Splines curve-selection groups. The pending flips update the direction arrows, then Enter commits and Esc discards. Viewport to File reuses the active viewport renderer, the owned Windows Save dialog and the same command session as the Display icon and scripted capture. From Lines shares a worker session across F6, menu and CMD. Click replay covers face removal, the edge chooser and commit. A 400-face cage round-trips with its source polygons and face IDs. Four placed-profile fixtures now exercise the straight-wire geometry of Extrude Lines through From Lines. The factory row now opens the distance prompt. Direction accepts two points before taking a separate distance. Preview and commit agree in Through Finger. Placed curved profiles round-trip without refitting. The Solid option reuses the exact placed-curve prism and topology-proof path used by the archive worker. A mixed selection of a rectangle, a circle and an arc survives Save/Open with both negative and symmetric extrusion. ToBoundary freezes the input curves, accepts a single surface or a clicked polysurface face, and cuts the rails or capped solid against its trimmed domain. Esc restores the previous selection. A shaded box replay distinguishes the visible top from the bottom face. For closed prisms, SplitAtTangents has a measured reference from Rhino 5. A capped stadium changes from three faces to six while retaining its volume with either a distance or a picked boundary. The October snapshot below describes the initial dispatcher.

## Audit result

F6 existed only as a fixed Line/Point/File popup. Added the typed context adapter:

- Selection classification: General / Solid / Curve / Surface / Gem; mixed or unknown falls back to General.
- General actions come from the existing command registry, no duplicate dispatch.
- Curve/surface/solid/gem/report actions are **registered disabled** with explicit provider reasons when M03/M05 is absent.
- Custom user menu stores command IDs, preserves order, deduplicates and disables deleted IDs.
- Builder prefill emits a typed versioned request only when the M04 provider exists.
- Floating preference clamps to the active screen in logical points; pin state is retained.
- Disabled buttons use `add_enabled` and never call `invoke`.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Selected-class dispatch | `f6::classify`, `context` | f6 unit 7, `f6_context` 2 |
| `-002` | General mode | registry actions | f6 unit 7, `f6_context` 2 |
| `-003` | Curve/surface | registered disabled M03 | f6 unit 7 |
| `-004` | Gem/report | registered disabled M05 | f6 unit 7 |
| `-005` | Custom menu | `CustomMenu`, `om9.f6.v1` | f6 unit 7, `f6_context` 2 |
| `-006` | Builder prefill | `BuilderPrefill`, provider gate | f6 unit 7 |
| `-007` | Floating pin/position | `F6WindowPreference::clamp` | f6 unit 7 |
| `-008` | Disabled actions | availability + `add_enabled` | f6 unit 7 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-app f6 unit` | **7 passed** | `implementation/evidence/M02/2026-10-22-f6-unit.log` |
| `om9-app f6_context` | **2 passed** | `implementation/evidence/M02/2026-10-22-f6-context.log` |
| `om9-app --features gpu-sphere --lib` | **168 passed** | `implementation/evidence/M02/2026-10-22-f6-app-lib.log` |
| `cargo fmt --all -- --check` | exit 0 | `implementation/evidence/M02/2026-10-22-app-fmt.log` |
| App/workspace clippy | exit 0 | `implementation/evidence/M02/2026-10-22-app-clippy-final.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-f6-public-source-audit.log` |

## Factory menu review, 2026-11-22

The original baseline below describes the October implementation. The eight-mode panel now uses the factory group predicates, order, per-row colors and macro parameters. Closed-mesh booleans use the standalone worker from the menu and CMD. Reduce Mesh uses the same count/percent options and preview from either entry point. Mesh Outline reuses the active render meshes and camera, unions overlapping projections, and selects the closed output curves in one undoable operation. ShowSelected uses a temporary hidden-only viewport and the existing layer transaction. Points Off Selected isolates grip owners and keeps the points of other objects. Split Point supports choosing its target after invocation from every factory spelling. A box-mode T-Spline cage retains polygon indices across viewport triangulation. Delete Face(s) and Duplicate Face(s) validate the selected faces and publish one transaction. Report uses the factory object counts and reads direct and layer materials. The popup uses the original pointer offset. Position, pin, report updates, material assignment and project round-trip have integration fixtures. Current coverage and outstanding routes are tracked in the [F6 runtime evidence](../../../../docs/validation/2026-11-22-f6-matrix9.md).

## Manual gate and limits

- **M02-UI human review is still pending.** Context grouping, click dispatch, pin and disabled tooltips are tested headlessly; visual placement/keyboard behavior needs human review.
- macOS runtime is unverified.
- M03/M04/M05 providers are absent; their F6 actions are registered disabled, not stubbed success.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-09-f6-context-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-09-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-09-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-09-00*/`
