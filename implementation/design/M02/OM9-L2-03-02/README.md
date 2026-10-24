# M02 `OM9-L2-03-02` — Febe Viewport Navigation Pan Zoom Orbit

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-02` (M02)

## Audit result

Right-drag orbit/pan, wheel zoom-to-cursor and title maximize already existed and were covered
by the M01 navigation checkpoint. The audit found four missing pieces for this L2:

- Zoom Extents / Zoom Selected;
- Home and Center view commands;
- a persistent Shift decision at drag press (a later frame without a modifier event could
  otherwise turn a Shift-pan into an orbit);
- a single camera-fit primitive and explicit tests for both projections.

These are implemented in the existing Rust camera/navigation service; no second camera math or
second renderer was introduced.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Perspective right-drag orbit | navigation gesture + `Camera::orbit` | `navigation_drag` 3, `navigation_lifecycle` 8, `camera_orbit` 5 |
| `-002` | Orthographic right-drag pan | navigation gesture + `Camera::pan` | `navigation_drag` 3, `camera_grid` 5 |
| `-003` | Wheel zoom to cursor | `ZoomResponse` + `Camera::zoom` | `navigation_input` 1, `navigation_precision` 3 |
| `-004` | Shift+right Perspective pan | captured `GestureSession.pan` | `navigation_drag` 3 |
| `-005` | Zoom Extents/Selected | `Controller::{zoom_extents,zoom_selected}`, `Camera::fit_bounds` | `navigation_commands` 6, `camera_fit` 4 |
| `-006` | Home/Center | `Workspace::default_camera`, `Camera::center_at`, `view_home/view_center` | `navigation_commands` 6, `camera_fit` 4 |
| `-007` | Title maximize/restore | `Workspace::toggle_maximize`, title double-click | `viewport_layout` 3, `viewport_title_menu` 17, `camera_grid` 5 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-scene --test camera_fit` | **4 passed** | `implementation/evidence/M02/2026-10-22-nav-camera-fit.log` |
| `om9-app --test navigation_commands` | **6 passed** | `implementation/evidence/M02/2026-10-22-nav-commands.log` |
| `om9-app --test navigation_drag` | **3 passed** | `implementation/evidence/M02/2026-10-22-nav-drag.log` |
| `om9-app` navigation regressions | **25 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-nav-regressions.log` |
| `om9-scene` full | **103 passed**, 21 suites | `implementation/evidence/M02/2026-10-22-nav-scene-full.log` |
| `om9-core` full | **407 passed**, 48 suites | `implementation/evidence/M02/2026-10-22-nav-core-full.log` |
| `om9-app --features gpu-sphere --lib` | **148 passed** | `implementation/evidence/M02/2026-10-22-nav-app-lib.log` |
| `om9-app navigation_maximize` | **2 passed** | `implementation/evidence/M02/2026-10-22-nav-maximize.log` |
| `om9-app viewport_title_menu` + `chrome_actions` | **17 + 13 passed** | `implementation/evidence/M02/2026-10-22-nav-title-menu.log` |
| `cargo clippy -p om9-scene --lib --test camera_fit -- -D warnings` | exit 0 | `implementation/evidence/M02/2026-10-22-nav-clippy.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-nav-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- Presentation/performance acceptance remains open as recorded by the 2026-10-21 checkpoint.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-02-navigation-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-02-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-02-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-02-00*/`
