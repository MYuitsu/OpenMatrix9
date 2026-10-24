# M02 `OM9-L2-02-03` — Febe Osnaps And Basic Smarttrack

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-02-03` (M02)

## Audit result

No object-snap or SmartTrack engine existed in the standalone tree (only disabled toolbar
icons). This delivery adds a Rust-owned snap query over the captured `Scene`:

- End, Mid, Center, Quadrant, Point, Intersection, Near, Tangent, Perpendicular and Surface
  candidates with a screen-radius filter;
- deterministic priority and one-shot override state (`accept`/`cancel`);
- transient SmartTrack anchors, CPlane-axis guides and an ephemeral inferred intersection;
- viewport marker/tooltip and click-path integration; the marker is painted from the last
  core candidate and no second snap solver runs in the UI;
- Snaps palette icons now toggle persistent snap kinds.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | End/Mid/Center/Quadrant | `osnap.rs` line/circle branches | `osnap` 11 |
| `-002` | Near/Point/Intersection | clamped segment, Point entity, exact segment intersection | `osnap` 11 |
| `-003` | Tangent/Perpendicular | circle tangent branches and radial perpendicular | `osnap` 11 |
| `-004` | Surface/Polysurface | nearest ray-triangle hit on BRep display | `osnap_brep` 1, `osnap` 11 |
| `-005` | Priority/one-shot | `SnapKind::priority`, `SnapState::{accept,cancel}` | `osnap` 11, `osnap_ui` 3 |
| `-006` | SmartTrack | `SmartTrack`, guides, inferred intersection | `osnap` 11, `osnap_ui` 3 |
| `-007` | Marker/tooltip | `paint_osnap`, `SnapCandidate::{marker_label,tooltip}` | `osnap_ui` 3 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `cargo test -p om9-scene --test osnap` | **11 passed** | `implementation/evidence/M02/2026-10-22-osnap-scene.log` |
| `cargo test -p om9-app --features occt-brep --test osnap_brep` | **1 passed** | `implementation/evidence/M02/2026-10-22-osnap-brep.log` |
| `cargo test -p om9-app --test osnap_ui` | **3 passed** | `implementation/evidence/M02/2026-10-22-osnap-ui.log` |
| `cargo test -p om9-scene -j 2` | **99 passed**, 20 suites | `implementation/evidence/M02/2026-10-22-osnap-scene-full.log` |
| `cargo test -p om9-core -j 2` | **407 passed**, 48 suites | `implementation/evidence/M02/2026-10-22-osnap-core-full.log` |
| `om9-app` viewport/selection regressions | **60 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-osnap-regressions.log` |
| `om9-app --features gpu-sphere --lib` | **148 passed** | `implementation/evidence/M02/2026-10-22-osnap-app-lib.log` |
| `cargo clippy -p om9-scene --lib --test osnap -- -D warnings` | exit 0 | `implementation/evidence/M02/2026-10-22-osnap-clippy.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-osnap-public-source-audit.log` |

`cargo clippy -p om9-app --lib --no-deps -D warnings` remains blocked by pre-existing lints in
`gpu_viewport.rs`, `ui.rs`, `camera_rig.rs` and `navigation.rs`, outside this module.

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless UI tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- Trimmed NURBS-surface snapping beyond the BRep display tessellation and grid/object projection
  snapping are outside this L2.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-02-03-osnaps-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-02-03-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-02-03-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-02-03-00*/`
