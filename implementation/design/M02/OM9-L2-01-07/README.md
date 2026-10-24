# M02 `OM9-L2-01-07` — Febe Picking Selection Generic Handles

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Audit date:** 2026-10-22
**Target:** standalone Rust OM9 (Windows x64) · **Owner:** `OM9-L2-01-07` (M02)
**Plan:** `OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L2.json`

## Audit result

The standalone implementation already existed in `om9-scene`, `om9-core` and `om9-app`; no
L3 was missing its feature surface. The exact L3 oracles and source/test mapping are recorded in
`implementation/design/M02/OM9-L3-01-07-00*/` and `implementation/evidence/M02/OM9-L3-01-07-00*/`.

The audit did find one real defect in candidate enumeration and fixed it (below). All other
selection/Gumball behaviour required no new feature code; the 2026-10-19 evidence remains the
historical record, superseded only for current verification counts.

## Capability map

| L3 | Capability | Code | Verification |
|---|---|---|---|
| `-001` | Screen ray and hit test | `om9_scene::{Scene::pick, pick_all}`, `picking.rs`, `spatial.rs` | `acceptance_l2_01_07` 9, `world_picking` 7, `navigation_snapshot` 2, native BRep `brep_selection` 5 |
| `-002` | Object selection set | `Controller::{select_root, select_objects, toggle_selection, add_objects, remove_objects}`, `om9_core::selection` | `acceptance_selection_set` 6, `selection_state` 7, `selection_overlay` 8 |
| `-003` | Face/edge/vertex picking | `Scene::pick_kind`, `session::subobjects`, revision-scoped keys | `subobject_pick` 4, `subobject_selection` 8, `subobject_occurrences` 5, `brep_selection` 5 |
| `-004` | Window and crossing | `selection::select_band`, viewport band gesture | `selection_regressions` 7, `rubber_band` 2, `acceptance_viewport_selection` 8 |
| `-005` | Ambiguity disambiguation | `Scene::pick_all` + viewport popup | `pick_disambiguation` 5 (10/10 runs), new scene regression 32 publications |
| `-006` | Highlight and hover | `ui::hover_frame`, selection frame, GPU highlight pass | `selection_overlay` 8, `subobject_occurrences` 5 |
| `-007` | Generic move/rotate/scale handles | `gumball_view`, `om9_core::gumball`, Gumball UI | `gumball_view` 13, `gumball_viewport` 3, `gumball_placement` 4, `gumball_interaction` 5, `gumball_numeric` 5, `om9-core gumball` 15 |
| `-008` | Drag transaction, commit, cancel | preview session, `commit_gumball`, atomic transaction | `gumball_interaction` 5, `gumball_numeric` 5, `gumball_cold` 1, `affine_gesture` 5, Esc no-write test |

## Defect found and fixed

`Scene::pick_all` (the disambiguation candidate query) applied the **nearest-only** depth
early-out even when `nearest_only == false`:

```rust
// before
if sphere_depth - radius > best_depth { continue; }

// after
if nearest_only && sphere_depth - radius > best_depth { continue; }
```

If the spatial index visited the nearer of two objects on the same ray first, the farther object
was pruned before it could become a candidate. The popup therefore opened only sometimes; after
the change it always lists both, nearest first. The failing `pick_disambiguation` run is retained
in `2026-10-22-pick-disambiguation-retry.log`.

New regression:
`crates/om9-scene/tests/acceptance_l2_01_07.rs::two_objects_on_one_ray_are_both_candidates_in_every_publication`
(32 fresh publications per run, asserting both IDs nearest-first and that a plain click still
picks only the nearest).

## Verification — 2026-10-22

| Suite | Result | Log |
|---|---:|---|
| `cargo test -p om9-core -j 2` | **403 passed**, 0 failed (47 suites) | `implementation/evidence/M02/2026-10-22-core-tests-after-fix.log` |
| `cargo test -p om9-scene -j 2` | **82 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-scene-tests-after-fix.log` |
| `om9-app` selection/Gumball targets | **63 passed**, 0 failed (1 ignored: `pick_27k`) | `implementation/evidence/M02/2026-10-22-app-selection-tests-after-fix.log` |
| `om9-app --features gpu-sphere --lib` | **148 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-app-gpu-lib.log` |
| `om9-app --features gpu-sphere --test brep_selection` | **5 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-brep-selection-after-fix.log` |
| `pick_disambiguation` repeated ten times | **10/10 runs**, 5 passed each | `implementation/evidence/M02/2026-10-22-pick-disambiguation-x10.log` |
| `pick_27k` with local `bong-27k.om9` | **1 passed**, 580.62 ms average (3000 ms test threshold; not a frame-rate claim) | `implementation/evidence/M02/2026-10-22-pick-27k-run.log` |
| `three_dm_brep_pick` with generated single-BRep fixture | **1 passed**, 0 failed | `implementation/evidence/M02/2026-10-22-three-dm-brep-pick-pass.log` |
| `cargo clippy -p om9-scene --all-targets -- -D warnings` | exit 0 | `implementation/evidence/M02/2026-10-22-scene-clippy.log` |
| `python tools/public_source_audit.py` | exit 0, 0 errors | `implementation/evidence/M02/2026-10-22-public-source-audit-final.log` |

`cargo fmt --check` still reports pre-existing formatting differences in
`crates/om9-app/src/camera_rig.rs` (outside this module); the edited test file is rustfmt-clean.

## Manual gate and limits

- **M02-UI human review is still pending.** Automated checks, synthetic replay and this audit are
  not human approval. `full_l2_complete` therefore stays `false`.
- `pick_27k` and `three_dm_brep_pick` stay `#[ignore]` by repo convention because they need
  external fixtures. Both were run on 2026-10-22 with local fixtures: `pick_27k` passed its
  current 3000 ms baseline at **580.62 ms** (not a 16.7 ms frame-rate claim), and
  `three_dm_brep_pick` passed against a single editable BRep imported from `v5_my_brep.3dm`
  (`build/tmp/v5_my_brep.om9`, sha256 `941fb834…`). They remain disabled in a default
  `cargo test` run without those fixtures.
- macOS runtime is unverified.
- QymCAD remains a reference only; no build-time dependency and no pinned-source reuse claim.

## Files

- Evidence JSON: `implementation/evidence/M02/2026-10-22-l2-01-07-selection-handles-checks.json`
- Fix note: `implementation/evidence/M02/2026-10-22-pick-all-fix.md`
- Per-L3 design: `implementation/design/M02/OM9-L3-01-07-00*/`
- Per-L3 fixtures: `implementation/fixtures/M02/OM9-L3-01-07-00*/`
- Per-L3 evidence: `implementation/evidence/M02/OM9-L3-01-07-00*/`
