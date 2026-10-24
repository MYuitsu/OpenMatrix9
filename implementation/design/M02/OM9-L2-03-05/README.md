# M02 `OM9-L2-03-05` — Febe Viewport Display Shading And Gems

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-05` (M02)

## Audit result

Wireframe mode and the selection overlay already existed. The audit found four missing pieces and
implemented them without fabricating M05/M07 data:

- **Shaded** was listed in the View menu but explicitly refused; it now switches only
  `ViewState.mode`, with the same scene bbox and no document/journal write.
- **Gem View cue**: a typed `gem_cue` reads only the `om9.gem` semantic extension; a colour token is
  used when present, otherwise the neutral placeholder carries the explicit `DEFERRED_WAIT_M05`.
- **Curve/cutter visibility**: a presentation-only `VisibilityFilter` resolves semantic categories;
  the cutter category is disabled (and cannot hide model geometry) until M07 records exist.
- **Backface cue** and **versioned display preset** (wireframe/edges/gem/category/backface) with a
  safe schema fallback.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Wireframe/Shaded switch | `viewport_menu`, `session`, `workspace` | `display_modes` 3, `viewport_title_menu` 18 |
| `-002` | Gem colour/neutral cue | `display::gem_cue` | `display cues` 5 |
| `-003` | Curve/cutter presets | `display::VisibilityFilter` | `display cues` 5 |
| `-004` | Backface cues | `display::surface_side` | `display cues` 5 |
| `-005` | Selection highlight | `ui` overlay + preserved IDs | `selection_overlay` 8, `display_modes` 3 |
| `-006` | Display preset save/load | `DisplayPreferences` | `display cues` 5, `display_modes` 3 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-app display_modes` | **3 passed** | `implementation/evidence/M02/2026-10-22-display-modes.log` |
| Display cue unit tests | **6 passed** | `implementation/evidence/M02/2026-10-22-display-cues.log` |
| `om9-app display_persistence` | **2 passed** | `implementation/evidence/M02/2026-10-22-display-preset-persistence.log` |
| `cargo fmt --all -- --check` | exit 0 | `implementation/evidence/M02/2026-10-22-app-fmt.log` |
| `cargo clippy -p om9-app --all-targets -- -D warnings` | exit 0 | `implementation/evidence/M02/2026-10-22-app-clippy-final.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 | `implementation/evidence/M02/2026-10-22-workspace-clippy-final.log` |
| Display/persistence/selection regressions | **33 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-display-regressions.log` |
| `om9-app --features gpu-sphere --lib` | **155 passed** | `implementation/evidence/M02/2026-10-22-display-app-lib.log` |
| `om9-scene` full | **103 passed**, 21 suites | `implementation/evidence/M02/2026-10-22-snaps-scene-full.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-display-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- The typed M05 `GemMaterialRecord` and M07 `CutterRecord` contracts are tested with synthetic fixtures; no production M05 asset library or M07 cutter generator is claimed.
- `DEFERRED_WAIT_M05` / `DEFERRED_WAIT_M07` remain intentional when those records are absent.
- Display preferences are embedded in the `.om9` archive as `om9.display.v1` and restored on reload; visual sign-off is still pending.
- Synthetic provider fixtures: `implementation/fixtures/M02/OM9-L3-03-05-002/m05-gem-material.example.json` and `.../OM9-L3-03-05-003/m07-cutter-record.example.json`. They are not production M05 assets or M07 cutters.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-05-display-shading-gems-checks.json`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-05-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-05-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-05-00*/`
