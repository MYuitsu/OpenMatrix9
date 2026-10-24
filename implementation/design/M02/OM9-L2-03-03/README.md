# M02 `OM9-L2-03-03` — Febe Viewport Object Selection And Gumball

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-03-03` (M02)

## Audit result

All six L3 capabilities were already implemented on the shared
`OM9-L2-01-07` selection engine and `om9-core` Gumball service. The audit added the missing
acceptance evidence and two small testability/coverage surfaces:

- a dedicated click-mapping test (plain replace, Shift add, Ctrl toggle, locked refusal);
- a popup row accessor test proving the type label + layer-colour swatch;
- an exact X-axis +5 mm Gumball drag with one Undo/Redo entry;
- an exact `MoveX 12.5` numeric edit with one Undo entry.

No second picker or second Gumball implementation was introduced.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Click/Shift/Ctrl selection | `ui.rs` adapter → `OM9-L2-01-07` controller | `matrix_selection_sequence` 2, `acceptance_viewport_selection` 8 |
| `-002` | Window/crossing marquee | `ui.rs` band + `scene::selection` | `rubber_band` 2, `selection_overlay` 8, `selection_regressions` 7 |
| `-003` | Ambiguous popup type/swatch | `ambiguous_row`, core pick-all | `pick_disambiguation` 5, `matrix_selection_sequence` 2 |
| `-004` | Gumball axis drag | viewport + controller Gumball transaction | `gumball_interaction` 6, `gumball_viewport` 3, `core gumball` 15 |
| `-005` | Alignment/relocate | `session/gumball.rs` frame modes | `gumball_placement` 4 |
| `-006` | Numeric edit | `gumball_numeric` parser + commit | `gumball_numeric` 6 |

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `om9-app matrix_selection_sequence` | **2 passed** | `implementation/evidence/M02/2026-10-22-selgmb-selection.log` |
| `om9-app gumball_interaction` | **6 passed** | `implementation/evidence/M02/2026-10-22-selgmb-gumball-axis.log` |
| `om9-app gumball_numeric` | **6 passed** | `implementation/evidence/M02/2026-10-22-selgmb-numeric.log` |
| Selection/Gumball regression targets | **43 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-selgmb-regressions.log` |
| `om9-scene selection_regressions` | **7 passed** | `implementation/evidence/M02/2026-10-22-selgmb-scene-selection.log` |
| `om9-core gumball` | **15 passed** | `implementation/evidence/M02/2026-10-22-selgmb-core-gumball.log` |
| `om9-app --features gpu-sphere --lib` | **148 passed** | `implementation/evidence/M02/2026-10-22-selgmb-app-lib.log` |
| `om9-scene` full | **103 passed**, 21 suites | `implementation/evidence/M02/2026-10-22-selgmb-scene-full.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-selgmb-public-source-audit.log` |

## Manual gate and limits

- **M02-UI human review is still pending.** Automated headless tests are not human sign-off;
  `full_l2_complete` remains `false`.
- macOS runtime is unverified.
- Popup rows list objects; overlapping faces of one object are not a separate popup row.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-03-03-selection-gumball-checks.json`
- Existing kernel note: `implementation/design/M02/OM9-L2-03-03/W4-GUMBALL-KERNEL.md`
- Per-L3 design: `implementation/design/M02/OM9-L3-03-03-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-03-03-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-03-03-00*/`
