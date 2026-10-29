# Stub integration note — CMD host and scene host seams (for the orchestrator)

Applies AFTER (1) the concurrent UI workflow on `ui.rs` / `workbench_regions.rs` has finished and
(2) checklist items 2.7 (CMD host) and 3.9 (document/scene host) are GREEN. The backend loops do not
touch any UI file; this note is the only place the swap is described. M00: decision D4 in
`m00-decisions.md` (covered by the user's unblock request; no visible change in normal operation).

## 1. What exists today (audit, working tree 2026-10-16)

| Seam (defined in `om9-app/src/workbench_regions.rs`) | Today's adapter | Real? |
| --- | --- | --- |
| `trait CommandInputHost { fn command_input_ready(&self) -> bool; }` (:554-558) | `struct AttachedCmdFrame` in `ui.rs:78-89`, constant `true` | **stub** |
| `trait ViewportSceneHost { fn visible_views(&self) -> Vec<ViewId>; }` (:561-565) | `impl ViewportSceneHost for om9_scene::workspace::Workspace` (:566-571), passed as `&self.controller.workspace` in `ui.rs` `refresh_region_plan` (:623-637) | already the real view owner |

`RegionError::CommandInputUnavailable` (:227) is reachable only from stub tests today.

## 2. Backend APIs that replace them (delivered by the loops)

| Delivered in | API | Replaces |
| --- | --- | --- |
| `om9-app/src/session/hosts.rs` (item 2.7) | `Controller::command_host_state() -> CommandHostState`, `Controller::command_input_ready() -> bool`, `impl CommandInputHost for Controller` | `AttachedCmdFrame` |
| `om9-app/src/session/hosts.rs` (item 3.9) | `Controller::document_scene_state() -> DocumentSceneState`, `impl ViewportSceneHost for Controller` (returns `self.workspace.visible_views()`, identical output) | `&self.controller.workspace` argument (optional swap; behavior identical) |

`command_input_ready()` is `false` only when `Controller.scene` is `None` (the state
`refresh_scene` leaves after a scene-cache failure, when no command can publish). File prompts,
layer dialogs and running commands keep it `true`, so no new layout error appears in normal use.

## 3. Exact edits (orchestrator, after the UI workflow)

1. `standalone/crates/om9-app/src/ui.rs`
   - Delete the doc comment, `struct AttachedCmdFrame;` and `impl CommandInputHost for AttachedCmdFrame`
     (ui.rs:78-89 at audit time; locate by name, line numbers move).
   - In `MatrixApp::refresh_region_plan` change the call to
     `compute_regions_for(&self.dock, WindowSize::from(window), &self.controller, &self.controller)`
     (two shared borrows of one field are fine; the result is owned).
   - Remove `CommandInputHost` from the `use crate::{.., workbench_regions::{..}}` list (ui.rs:8),
     otherwise `-D warnings` fails on an unused import.
2. `standalone/crates/om9-app/src/workbench_regions.rs` — doc comments only: module doc :19-21 and the
   trait docs :553-565 say OM9-L2-01-02/-03 are "not implemented"; reword to name the
   `Controller` implementations. Keep `impl ViewportSceneHost for Workspace` (its unit test and
   `tests/workbench_regions.rs` stubs use it).
3. Add the frame-level test in `standalone/crates/om9-app/src/session/hosts.rs` `#[cfg(test)]`
   (it must live under `session` because `Controller.scene` is private there; `MatrixApp`,
   `region_error`, `region_plan`, `dock`, `status` are public):

```rust
#[test]
fn frame_with_unready_cmd_host_raises_command_input_unavailable() {
    use crate::{ui::MatrixApp, workbench_regions::RegionError};
    use eframe::egui;
    let mut app = MatrixApp::new(om9_core::model::Document::new("cmd host")).unwrap();
    let ctx = egui::Context::default();
    let frame = |app: &mut MatrixApp| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280., 900.))),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    };
    frame(&mut app);
    frame(&mut app);
    assert!(app.region_error.is_none());
    let good = app.region_plan.clone().expect("good plan");
    let dock = app.dock.to_json().unwrap();
    app.controller.scene = None; // state left by a failed refresh_scene
    frame(&mut app);
    assert_eq!(app.region_error, Some(RegionError::CommandInputUnavailable));
    assert_eq!(app.region_plan.as_ref(), Some(&good), "last good plan kept");
    assert_eq!(app.dock.to_json().unwrap(), dock, "layout untouched");
    assert!(app.status.message().contains("command input is not ready"));
    app.controller.refresh_scene().unwrap();
    frame(&mut app);
    assert!(app.region_error.is_none(), "recovers once the host is ready");
}
```

   RED before step 1 (the stub keeps `region_error` `None`), GREEN after.
4. Optional after item 8.1 (float_roundtrip): `tests/workbench_regions_sizes.rs:695-699` saves a Point
   instead of the Line because of the old save defect; restore the Line save leg there.

## 4. Verification (from `h:\OpenMatrix9\standalone`)

```
cargo test -p om9-app --lib session::hosts
cargo test -p om9-app --test workbench_regions --test workbench_regions_sizes --test shell_docking --test status_bar_overlap
cargo test -p om9-app --test ui_events --test command_sessions --test pinned_command
cargo clippy -p om9-app --all-targets -- -D warnings
cargo test --workspace -j 1
```

Expected: all green; `workbench_regions.rs` positive/negative fixtures unchanged (its own
`FixedCommand` stub still drives the fixture case `command-input-host-not-ready`).

## 5. Ledger follow-up (finalize step)

In `docs/openmatrix9-progress.json`, the L3-002 entry still says
`"5_l2_01_02_03": "BLOCKED_DEPENDENCY ..."`, `"OM9-L2-01-02 / OM9-L2-01-03 stubbed ..."` and lists the
Line-save defect as not fixed. Update those three statements with the loops' evidence paths once the
swap and the frame test are green. Do not edit `OM9_Milestones/` planning states.

## 6. Out of scope for the swap

- Menu enable state still uses the static `Controller::availability`; switching it to the state-aware
  `command_availability` (e.g. Undo greyed with an empty journal) changes what the user sees and
  needs the user's consent first.
- The LAYERS panel position (left column below INFO & SETTINGS) belongs to the UI workflow; the
  backend only provides layer records and rules.
