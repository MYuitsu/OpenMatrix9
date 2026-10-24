# M02 `OM9-L2-03-04` — Febe Viewport Snaps Ortho Smarttrack

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-04` (M02)

## Audit result

Snap kinds, one-shot settings and SmartTrack already existed from `OM9-L2-02-03`. The audit found
three UI gaps in this L2 and implemented them without a second solver:

- **Project** mode was listed in the Snaps palette but had no Controller flag or projection path.
- **SmartTrack** had no on/off control and guides were painted only in the active view.
- The fixed palette icons could not show an off/on state because a disabled `egui` icon is also
  unclickable; they now use a clickable active-frame toggle.
- The **End/Mid keyboard shortcuts** and the one-shot indicator were missing.
- A focused command editor still owns the keyboard; a viewport pick now releases that focus so the
  shortcuts cannot steal typed text.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Snap toggle panel | `sidebar_palettes`, `chrome::icon_toggle`, `SnapSettings` | palette toggle 1, `osnap_ui` 3, `snap_panel_modes` 4 |
| `-002` | Ortho/Planar/Project | `Controller::{ortho,planar,project}`, `constrain_point`, `project_to_plane` | palette toggle 1, `snap_panel_modes` 4 |
| `-003` | Activation/indicator | `SnapSettings.one_shot`, accept/cancel, viewport badge | `snap_panel_modes` 4, `osnap_ui` 3 |
| `-004` | SmartTrack toggle/guides | `smarttrack_enabled`, all-view guide painting | `snap_panel_modes` 4, `osnap_ui` 3 |
| `-005` | End/Mid shortcuts | E/M while a command runs, focus-gated | `snap_panel_modes` 4 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| Palette toggle unit test | **1 passed** | `implementation/evidence/M02/2026-10-22-snaps-palette.log` |
| `om9-app snap_panel_modes` | **4 passed** | `implementation/evidence/M02/2026-10-22-snaps-modes.log` |
| `om9-app osnap_ui` + `osnap_brep` | **3 + 1 passed** | `implementation/evidence/M02/2026-10-22-snaps-regressions.log` |
| Snap/selection regression targets | **8 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-snaps-regressions.log` |
| `om9-app --features gpu-sphere --lib` | **149 passed** | `implementation/evidence/M02/2026-10-22-snaps-app-lib.log` |
| `om9-scene` full | **103 passed**, 21 suites | `implementation/evidence/M02/2026-10-22-snaps-scene-full.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-snaps-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- Grid Snap remains a present-but-disabled palette control; this L2 records the object-snap and
  Ortho/Planar/Project behavior, not a grid-spacing solver.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-04-snaps-ortho-smarttrack-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-04-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-04-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-04-00*/`
