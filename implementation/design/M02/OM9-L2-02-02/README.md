# M02 `OM9-L2-02-02` — Febe Cplanes And Coordinate Input

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-02-02` (M02)

## Audit result

The coordinate frame and typed local/world input existed, but the module did **not** satisfy
the seven L3 oracles. The following gaps were implemented in this delivery:

- named Top/Front/Right CPlane presets and a right-handed preset contract;
- origin change, rotation about the CPlane normal and a validated three-point frame;
- `Wx,y[,z]` world and `@x,y[,z]` relative typed input (relative is a frame displacement);
- F4 inserting `W0,0,0` only while a point prompt is active;
- Ortho/Planar cursor constraints used by viewport picks, including both modes together;
- explicit cross-view tests proving the point stays world-space when Top/Front/Perspective
  and camera state change.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | World/CPlane context | `CPlane` LocalFrame, per-view `ViewState.cplane`, captured session plane | `coordinate_input` 4, `coordinate_frame` 4, `cplane_coordinate_input` 10 |
| `-002` | Top/Front/Right presets | `CPlanePreset`, `CPlane::preset`, `Workspace::new` | `cplane_presets` 6, `cplane_menu` 1, `viewport_title_menu` 17 |
| `-003` | Origin/orientation change | `with_origin`, `rotated`, `from_three_points`, `Controller::set_cplane*` | `cplane_presets` 6, `cplane_coordinate_input` 10 |
| `-004` | Text point entry | `parse_input`, `resolve_with_base`, `submit_geometry_text` | `coordinate_input` 4, `cplane_coordinate_input` 10 |
| `-005` | F4 origin shortcut | `Controller::submit_world_origin`, `MatrixApp` F4 dispatch | `cplane_f4_ui` 1, `chrome_actions` 13 |
| `-006` | Ortho/Planar constraints | `Controller::constrain_point`, hover/click path, Snaps toggles | `cplane_coordinate_input` 10, `chrome_actions` 13 |
| `-007` | Cross-view consistency | world-space entities, independent per-view camera/CPlane | `cplane_coordinate_input` 10, `camera_grid` 5 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `cargo test -p om9-core -j 2` | **407 passed**, 48 suites | `implementation/evidence/M02/2026-10-22-cplane-core-full.log` |
| `cargo test -p om9-scene -j 2` | **88 passed**, 19 suites | `implementation/evidence/M02/2026-10-22-cplane-scene-full.log` |
| `om9-core coordinate_input` | **4 passed** | `implementation/evidence/M02/2026-10-22-cplane-core.log` |
| `om9-scene cplane_presets` | **6 passed** | `implementation/evidence/M02/2026-10-22-cplane-scene.log` |
| `om9-app` CPlane focused targets (`cplane_coordinate_input`, `cplane_f4_ui`, `cplane_menu`) | **12 passed** | `implementation/evidence/M02/2026-10-22-cplane-app.log` |
| CPlane regression targets (`viewport_title_menu`, `chrome_actions`, `command_sessions`, `gumball_placement`, `cplane_*`) | **51 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-cplane-regressions.log` |
| `om9-app --features gpu-sphere --lib` | **148 passed** | `implementation/evidence/M02/2026-10-22-cplane-app-lib.log` |
| `clippy -p om9-core --lib --test coordinate_input` | exit 0 | `implementation/evidence/M02/2026-10-22-cplane-clippy-core.log` |
| `clippy -p om9-scene --lib --test cplane_presets …` | exit 0 | `implementation/evidence/M02/2026-10-22-cplane-clippy-scene.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-cplane-public-source-audit-final.log` |

`cargo clippy -p om9-app --lib --no-deps -- -D warnings` remains blocked by pre-existing lints in
`gpu_viewport.rs`, `ui.rs`, `camera_rig.rs` and `navigation.rs` outside this module. The new CPlane
tests compile and run; no new warning is attributed to the changed lines.

`cargo fmt --check` still reports pre-existing differences outside this module; all new files and
the edited CPlane sections are rustfmt-clean.

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless UI tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- Other orthogonal snapping modes (grid, object snaps) are outside this L2 and remain disabled.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-02-02-cplanes-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-02-02-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-02-02-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-02-02-00*/`
