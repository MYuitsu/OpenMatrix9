# OM9-L2-04-11 — Gem Halo / Cluster / Baguette Layouts

- **Milestone:** M05 — Gems and Layout
- **Owner:** `om9-core::gem_cluster` (typed layout math) + `om9-app` (session, window, preview)
- **Date:** 2026-10-27
- **Target:** Standalone Rust OM9; Windows x64; `CARGO_TARGET_DIR=L:\om9-target`; `-j 2`
- **Source revision:** `71852257e59f4117c3b3bfca73f047ac546f76c2` (dirty checkout)
- **Manual gate:** `M05-UI` — **OPEN**. No reviewer signature is claimed.

## Scope

This owner implements the seven L3 of the parent L2:

| L3 | Capability | Command |
| --- | --- | --- |
| `OM9-L3-04-11-001` | Halo Builder | `gvHaloBuilder` |
| `OM9-L3-04-11-002` | Cluster Builder | `gvClusterBuilder` |
| `OM9-L3-04-11-003` | Baguette Channel Layout | `gvTaperBagChannel` |
| `OM9-L3-04-11-004` | Tapered Baguettes Between Two Curves | `gvTaperBagBetweenTwoCurves` |
| `OM9-L3-04-11-005` | Gems Between Two Curves | `gvGemsBetween2Crvs` |
| `OM9-L3-04-11-006` | Orient To Gem | `gvOrientGems` |
| `OM9-L3-04-11-007` | Halo Cluster Edit Parameters | reopening any builder above |

`om9_core::gem_cluster` owns the geometry-free plans: `halo_layout`, `cluster_layout`,
`baguette_channel`, `tapered_baguette`, `gems_between_curves` and `orient_to_gem`. The app session
opens a builder from the current selection, recomputes the plan on every option, paints the world-space
preview, and commits **one** transaction (child inserts/updates/deletes plus the `om9.gem.cluster`
feature extension). Existing curve measurement is reused from `om9_core::curve_ops` and the placement
owner from `om9_core::gem_placement`; there is no second placement primitive.

The feature record stores parameters, source hosts and the ordered child `GemId`s. Reopening a builder
matches children by index, updates the ones that still exist, inserts new ones and deletes the trailing
ones; Undo restores both the children and the previous feature record.

## Command surface

Both the typed command line and the builder window use the same session and apply path.

- Halo: `Spacing`, `FromCenter`, `Angle`, `Rotation`, `Height`, `Placement`, `InnerWall`, `OuterWall`,
  `ProfileWidth`, `ProfileHeight`, `ChannelSpacing`, `WallHeight`, `Blend`, `EdgeNudge`, `ProngHeight`,
  `ProngDome`, `GemNudge`, `ProngTaper`, `ProngVertical`, `ProngsMeshed`, `Count`, `Even`, `Match`,
  `Fix`, `Apply`, `Cancel`, `Reset`.
- Cluster: `XY`, `Z` (positive moves down), `Degree`, `Start=bottom|top|left|right`, `Count`,
  `Rings`, `Spacing` (ring spacing), `Apply`, `Cancel`, `Reset`.
- Channel / Taper / Between: `Spacing`, `Taper`, `Banking`, `PointA`, `PointB`, `Flip`, `Middle`,
  `Straight`, `GemTop`, `AutoSpacing`, `Length`, `Width`, `Depth`, `Girdle`, `Apply`, `Cancel`, `Reset`.

`Apply` (or an empty Enter) commits; `Cancel` drops the session without touching the document.

## Verification

| Artifact | Command | Result |
| --- | --- | --- |
| `core-gem-cluster.log` | `cargo test -p om9-core --test gem_cluster -j 2` | 12 passed |
| `app-gem-cluster-panel.log` | `cargo test -p om9-app --test gem_cluster_panel -j 2` | 11 passed |
| `workspace-check.log` | `cargo check --workspace --all-targets -j 2` | exit 0 |
| `clippy-gem-cluster-lib.log` | `cargo clippy -p om9-core -p om9-app -p om9-kernel --lib -j 2 -- -D warnings` | exit 0 |
| `clippy-core-test.log` | `cargo clippy -p om9-core --test gem_cluster -- -D warnings` | exit 0 |
| `clippy-app-test.log` | `cargo clippy -p om9-app --test gem_cluster_panel -- -D warnings` | exit 0 |

The core suite asserts the layout oracles adversarially: closed halo ring incl. the last-to-first pair,
fixed 12:00 stone, refusal instead of squeeze, cluster start angles and downward Z, degree rotation,
tapered channel widths and profile, banking/flip, round stones spanning both walls, exact orient basis,
and the document JSON round trip of the feature. The app suite asserts the command surface: preview,
single-transaction Apply, Undo/Redo, Cancel leaving no phantom, child-id reuse on edit, and explicit
refusal of a bad selection.

## Boundaries / uncertainties

1. **M05-UI gate stays OPEN.** The six builder windows and the viewport preview have not been reviewed
   by a human on screen; no signature is claimed.
2. **Metal geometry is downstream.** Channel walls, bezels, prongs and their meshes belong to M06/M07.
   This owner records every manual channel/prong parameter (`inner_wall_mm`, `outer_wall_mm`,
   `channel_wall_height_mm`, `channel_blend_mm`, `channel_spacing_mm`, `profile_width_mm`,
   `profile_height_mm`, prong fields) in the feature so the metal builder consumes the same record.
3. **Halo style presets (Load) are not implemented:** there is no authored halo style catalogue in the
   workspace yet; the builder provides defaults and `Reset`. The manual's viewport control handles
   (yellow/gray/white/red grab handles) and the locked F4 guide gem (`Show Guide`) are also not drawn;
   every operation they expose is available from the builder window and the typed command line, so the
   manual M05-UI review must judge the missing direct-manipulation layer explicitly.
4. **Only Windows x64** was exercised in this run.
5. No Matrix gemstone binary, catalogue bitmap or decompiled resource is copied; accents are the
   procedural `GemSpec` owner.
