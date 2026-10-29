# OM9-L2-03-17 — 09-febe-matrix-surface-menu-workflows

- **Owner:** `OM9-L2-03-17` — 38 L3 / 114 L4
- **Date:** 2026-10-26
- **Status:** All 38 Surface Tools rows are functional with one stable command id each and one authored
  icon each; the Matrix menu click and the typed command line resolve to the same row, and every row
  runs its owner service through `surface_edit` / `om9_core::surface_ops` as one transaction/Undo.
  Acceptance records are READY_FOR_REVIEW (M03-UI).

## The authored menu is 40 icons, the milestone plan is 38 L3

`Resources/menu/MainMenu.ini` `[Menu8]` declares `IconCount=40`, and `menu_catalog::SURFACE` lists all
40. The milestone plan has 38 L3 rows: 36 of the menu icons plus the two Sweep History sub-commands
(`gvSweepHistory`/`gvSweep2History`), which share the Sweep 1/2 icons. The four remaining menu icons have
no L3 row of their own but run their own owner sessions:

| Menu icon | Command |
|---|---|
| `SurfaceRevolve` | `OM9-SURFACE-018` (`session/revolve.rs`); typing `Revolve` dispatches it |
| `SurfaceHeightfieldfromImage` | `OM9-SURFACE-031` (`session/heightfield.rs`); typing `Heightfield` dispatches it |
| `SurfaceSmash` | `OM9-SURFACE-141` (`session/surface_edit.rs` + `surface_ops::smash`); typing `Smash` dispatches it |
| `SurfaceSurfaceEditToolsRefittoTolerance` | `OM9-SURFACE-142` (`session/refit_srf.rs` + `surface_ops::refit_to_tolerance`); typing `RefitSrf` dispatches it |

`every_authored_surface_menu_icon_maps_to_an_available_command` pins the whole menu: 40 icons, 40 wired
to an available command, and the four menu-only command names resolve on the typed command line.

## Smash and surface Refit to Tolerance

`Smash` is the flattening `Unroll Surface` refuses on a non-developable input: it runs the same
arc-length unroll and reports the measured sampling-level stretch as a progress detail
(`stretch <value>`) instead of claiming an isometry. `Refit to Tolerance` (`RefitSrf`) waits for
`Tolerance=` (default 0.01), rebuilds the surface with `surface_kernel::refit` on the smallest degree
(1..=3) and grid (4..=24 poles per direction) that stays inside the tolerance on a 9×9 parameter grid,
and refuses a tolerance that bounded grid cannot represent. Both commit one transaction/Undo like the
other Surface rows; neither mutates the source surface.

Each row carries the documented Matrix script name (`command_line` in `surface_menu::BINDINGS`), so
typing `_Sweep1`, `gvSweepHistory`, `Sweep2`, `gvSweep2History`, `gvProfileSweep`, `Rebuild`, `BlendSrf`,
`VariableBlendSrf`, `Loft`, `NetworkSrf`, `ShrinkTrimmedSrf`, `Patch`, `PlanarSrf`, `EdgeSrf`, `Plane`,
`Untrim`, `ExtrudeCrv`, `RailRevolve`, `gvFourProfileSweep`, `FilletSrf`, `VariableFilletSrf`,
`ChamferSrf`, `VariableChamferSrf`, `OffsetSrf`, `VariableOffsetSrf`, `TweenSurfaces`, `MergeSrf`,
`MatchSrf`, `Drape`, `gvfourSweep`, `Split`, `ExtendSrf`, `Fin`, `UnrollSrf`, `SoftEditSrf`, `CutPlane`,
`SrfSeam` or `SetSurfaceTangent` reaches the same command as clicking the row.

Rebuild, Unroll, Offset, Extrude Normal and Plane from Corners run through `surface_edit` and
`om9_core::surface_ops`; the scene tessellates surfaces for display/picking without mutating document
geometry. History rows share the base authored icon but keep distinct command ids. `Rebuild` and
`Soft Edit` are also Curve menu labels: with a surface selected the Surface row wins, otherwise the
curve registry keeps the label (documented in `Controller::resolve_typed_command`).

Documented option contracts verified against the authored Matrix sheets:

- Sweep 1/2 History: `Closed` (default Yes only for closed rail(s) plus two or more profiles; hidden and
  refused otherwise), `Flip`, `Simple`/`Open`, and `Add Slash` alignment points for two rails.
- Soft Edit: base point `U`/`V`, move `Dx`/`Dy`/`Dz`, `UDistance`/`VDistance` (default 1 = whole domain),
  `Copy` (default Yes; No edits the selected surface in place), `FixEdges`, and
  `DirConstraint=None|SrfNormal|UTangent|VTangent` (the move is projected onto the constraint).
- Match Surfaces: `Source`/`Target` edges plus `Continuity=Position|Tangency|Curvature` (default Position).

## Evidence layout

- `implementation/design/M03/OM9-L3-03-17-00X/{dispatch_contract.json,wire_ui.json,source_crosswalk.json,api.md}`
- `implementation/fixtures/M03/OM9-L3-03-17-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-03-17-00X/` — one receipt per planned L4 role.

## Verification

```
cargo test -p om9-app --test surface_menu --test smash --test refit_srf
cargo test -p om9-core --test surface_ops
cargo test -p om9-app -j 2
cargo check -p om9-app -j 2
cargo clippy -p om9-app --all-targets -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/session.rs crates/om9-app/src/session/surface_edit.rs crates/om9-app/src/session/command_options.rs crates/om9-app/src/surface_menu.rs crates/om9-app/src/lib.rs crates/om9-app/src/ui/chrome.rs crates/om9-app/tests/surface_menu.rs
python tools/public_source_audit.py
```

## Limits

- The interactive point pick for Soft Edit is typed (`U`/`V` plus `Dx`/`Dy`/`Dz`); the viewport does not
  yet project a click onto surface UV parameters.
- `Smash` takes no options (the authored Surface menu declares none) and reports distortion instead of\n  refusing a doubly curved input; surface `Refit to Tolerance` takes only `Tolerance=`.\n- `refit_to_tolerance` measures its deviation on a fixed 9×9 parameter grid and searches a bounded\n  degree/grid range, so the reported bound is a sampling bound and a very small tolerance is refused.
- 37 of the 38 Matrix script names appear verbatim in the authored sheets; the Surface Extrude All
  sheet only names its menu label, so that row accepts `ExtrudeCrv` and the authored label
  `Surface Extrude All`.
- The M03-UI manual review of click paths/previews/warnings is unsigned.
- Existing surface geometry services are not duplicated in the UI and do not enable a row by themselves.
