# OM9-L2-04-12 — Gem Pavé Layout And Azure

- **Milestone:** M05 — Gems and Layout
- **Owner:** `om9-core::gem_pave` (typed layout math) + `om9-app` (session, window, preview)
- **Date:** 2026-10-28
- **Target:** Standalone Rust OM9; Windows x64; `CARGO_TARGET_DIR=L:\om9-target`; `-j 2`
- **Source revision:** `71852257e59f4117c3b3bfca73f047ac546f76c2` (dirty checkout)
- **Manual gate:** `M05-UI` — **OPEN**. No reviewer signature is claimed.

## Scope

This owner implements the eight L3 of the parent L2:

| L3 | Capability | Command |
| --- | --- | --- |
| `OM9-L3-04-12-001` | Auto Pavé Builder | `gvPaveDialog` / `BuilderPaveDialog` |
| `OM9-L3-04-12-002` | Manual Pavé Builder | `gvPaveBuilder` / `BuilderPaveBuilder` |
| `OM9-L3-04-12-003` | Pavé Azure Builder | `gvPaveAzureBuilder` / `BuilderPaveAzureBuilder` |
| `OM9-L3-04-12-004` | Pavé Prong Layout | `gvPaveProngBuilder` / `BuilderPaveProngBuilder` |
| `OM9-L3-04-12-005` | Pavé Sphere Layout | `gvPaveSphere` / `BuilderPaveSphere` |
| `OM9-L3-04-12-006` | Gem Springs And Relaxation | `gvGemSprings` / `BuilderGemSprings` |
| `OM9-L3-04-12-007` | Pavé Region Controls And Preview | any open pavé builder |
| `OM9-L3-04-12-008` | Pavé History Geometry Refresh | reopen Auto/Manual and `Refresh` |

`om9_core::gem_pave` owns the geometry-free plans: `pave_surface`, `pave_azures`,
`pave_prongs`, `pave_sphere`, `spring_relax`, `pave_framework` and `refresh_pave_layout`.
The app session opens one builder from the current selection, recomputes the plan on every
option, paints the world-space preview, and commits **one** transaction (child
inserts/updates/deletes plus the `om9.gem.pave` feature extension).

The feature record stores the host surface, typed parameters and the ordered child `GemId`s
with their surface UVs. Reopening a builder matches children by station `GemId` or by index,
updates the ones that still exist, inserts new ones and deletes the trailing ones; `Refresh`
re-evaluates the stored UVs after the host surface changed and refuses conflicts instead of
silently dropping stones. Attractor and edge source IDs are persisted too, and `Refresh`
resolves them back to the current document curves before recomputing.

## Command surface

Both the typed command line and the builder window use the same session and apply path.

- Auto/Manual Pavé: `Pattern=hex|square|tri|hexpent|alldirections|hexreverse`, `Start=base|curve|edges|seeds`,
  `Spacing`, `MinSpacing`, `Base`, `Min`, `Max`, `Margin`, `Border`, `Angle`, `Flip`, `Taper`,
  `Equalize`, `Perturb`, `Pre`, `Post`, `Relax`, `Strength`, `Seed=u,v`, `Count` (exact number),
  `Apply`, `Cancel`, `Reset`.
- Auto/Manual selection: one open curve becomes the attractor (`Start=curve`); closed curves
  become region edges (`Start=edges`). The window also lists built-in and user styles with
  **Apply style** and **Save style**.
- Azure: `Hex`, `Angle`, `Depth`, `Size`, `Through`, `Seat`, `MinSpacing`, `Scale`, `Apply`,
  `Cancel`, `Reset`.
- Prong: `Fillet`, `Height`, `Drop`, `Taper`, `Scale`, `Shared`, `Unshared`, `CenterMin`,
  `CenterMax`, `Touch`, `Apply`, `Cancel`, `Reset`.
- Sphere: `Radius`, `Count` or `X`/`T`, `Removal=none|a|b`, `Min`, `Strength`, `Apply`, `Cancel`,
  `Reset`; Radius and Count also have viewport handles.
- Springs: `Spacing`, `MinSpacing`, `Iterations`, `Strength`, `Resize`, `Apply`, `Cancel`, `Reset`.

`Apply` (or an empty Enter) commits; `Cancel` drops the session without touching the document.
`Reset` returns the typed defaults while keeping the selection.

## Styles

`PaveStyle` / `PaveStyleFile` (`om9.gem.pave_style.v1`) snapshot the typed stone parameters
and the Manual count. Built-ins are code (`Classic Hex`, `Square 45`, `Tapered Border`); user
styles are added by name and never rewrite the built-ins. With `OM9_PAVE_STYLE_LIBRARY`
pointing at a JSON file, user styles persist across launches; without it they are session-local.
Duplicate names, an unknown schema and an invalid parameter are refused by name.

## Verification

| Artifact | Command | Result |
| --- | --- | --- |
| `core-gem-pave.log` | `cargo test -p om9-core --test gem_pave -j 2` | 13 passed |
| `app-gem-pave-panel.log` | `cargo test -p om9-app --test gem_pave_panel -j 2` | 14 passed |
| `workspace-check.log` | `cargo check -p om9-core -p om9-app -p om9-kernel --lib -j 2` | see `observed.json` |
| `workspace-all-targets-blocked.log` | `cargo check --workspace --all-targets -j 2` | blocked by concurrent `cutters` test; see `observed.json` |
| `clippy-gem-pave-lib.log` | scoped clippy (see `observed.json`) | see `observed.json` |
| `clippy-core-test.log` | scoped clippy for the core pavé test (see `observed.json`) | see `observed.json` |
| `clippy-app-test.log` | scoped clippy for the app pavé test (see `observed.json`) | see `observed.json` |
| `public-source-audit` | `python tools/public_source_audit.py` | 0 errors (14484 files) |

The core suite asserts the layout oracles adversarially: plane fill and boundary containment,
all six lattice patterns, angle rotation, seeds/start refusal, attractor growth and closed-edge
clipping, style round trip and duplicate refusal, round and hexagonal azure profiles with
seat-angle contraction, shared/center/unshared prong contact graphs, Fibonacci sphere count
and removal patterns, spring separation with surface projection, feature JSON round trip and
refresh, nearest-neighbour framework, closed polygons and typed budget refusals. The app suite
asserts the command surface: preview, single-transaction Apply, Undo/Redo, Cancel leaving no
phantom, child-id reuse on edit, exact manual quantity, azure curve count, prong markers, sphere
without a surface, sphere viewport handles, style save/apply, curve guide selection and source
persistence, springs placement and Undo, typed commands and `Refresh`.

The clippy logs are the scoped pavé commands: they allow the five lints that currently fire in
concurrent untracked `cutters` / `gem_cutters` modules, while the strict whole-crate command is
recorded as [`clippy-strict-blocked.log`](clippy-strict-blocked.log) with its actual failure. The
full `--workspace --all-targets` check is likewise recorded as
[`workspace-all-targets-blocked.log`](workspace-all-targets-blocked.log), not turned into a false
PASS. Re-run both after those modules settle; the scoped pavé checks are clean now.

## Boundaries / uncertainties

1. **M05-UI gate stays OPEN.** Six builder windows and the viewport preview have not been
   reviewed by a human on screen; no signature is claimed. Use
   [`MANUAL_UI_CHECKLIST.md`](MANUAL_UI_CHECKLIST.md) for the review.
2. **Metal geometry is downstream.** Azure profiles and prong curves are document curves; the
   actual azure cutting, seat and prong solids belong to M06/M07. This owner records every typed
   parameter and source id so the metal builder consumes the same feature. It does not duplicate
   M06/M07 geometry.
3. **Style persistence is opt-in.** Without `OM9_PAVE_STYLE_LIBRARY`, user styles live for the
   session only; the built-ins are always available.
4. **Attractor sampling is bounded.** The plan adds a 128-point projection of the selected
   curve as candidate stations and uses a Gaussian size falloff; the exact Matrix falloff is not
   claimed. Edge curves are clipped to their **union**, so holes and multiple disjoint regions
   are not distinguished.
5. **Direct manipulation is limited to the sphere.** Sphere Radius/Count use the shared
   builder-handle route; the other builders remain slider/command driven. The manual gate must
   judge whether that is sufficient.
