# Investigation: Layer panel must sit in the LEFT dock, below Info & Settings

Read-only investigation, 2026-10-16. Nothing was edited except this report. The working tree is dirty because another workflow (wf_bb045d04a4cb1388) is editing `workbench_regions.rs`, `ui.rs` and `ui/chrome.rs`; line numbers below refer to the working tree as read and may drift.

Request being checked (user, message 5): "layer cũng phải nằm bên trái chứ phía dưới info & setting, phải giống y chang matrix9 dùng ở độ phân giải nào".

## 1. Summary answer

1. **Normal product behaviour already matches Matrix.** The default layout is `icon-history, main-menu, display, snaps, info-settings, layers, projects, builder-slot`, all docked left. `SectionId::ALL` in `standalone/crates/om9-app/src/dock_layout.rs:158-167` defines the order, and `sidebar_ui` renders it in a single left panel (`ui/chrome.rs:438-466`). That order is in HEAD, not new work. The 1600x1000 L2-01-01 screenshot shows it.
2. **The Layers panel the user complained about is almost certainly an audit artifact.** The size-matrix harness runs `om9.exe --ui-audit` with `OM9_UI_AUDIT_RIGHT_DOCK=1`. `ui.rs:1852-1857` handles that variable by calling `app.dock_menu_section(SectionId::Layers, DockSide::Right)`. The most recent screenshots, `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-min-window/matrix/*/workspace.png`, therefore show Layers in a right dock. Their receipts say `dock.left_sections = [icon-history, main-menu, display, snaps, info-settings, projects, builder-slot]` and `dock.right_sections = ["layers"]`. Layers was used as the only "something to put in the optional right dock" fixture. That contradicts Matrix and the user's rule.
3. **Matrix evidence:** the Matrix8 book 1 "A Tour of the Interface" text and screenshot put Layers in the left column, after Info & Settings and before Projects, with Builders below Projects. The right column appears only when the user drags a menu there.
4. **Proportions and resolution:** Matrix evidence gives only a ~300 px column width and the statement that content "runs off the screen" depending on screen size. It gives no fixed, proportional or resizable heights. It gives no resolution-dependent reordering. Nothing in OM9 reorders by resolution either.
5. **Recommended fix** (section 6): stop using Layers as the right-dock fixture and keep it in the left column in every audit and test configuration. Add receipt fields and validator checks that fail if Layers leaves the left stack or its order changes. Update the three affected docs. No change is needed to the layer panel content (`ui/layers.rs`, owned by L2-03-14).

## 2. Where OM9 places things today (question 1)

### 2.1 Layout model: `standalone/crates/om9-app/src/dock_layout.rs`

- Header comment (lines 11-19): left column is "Icon History, Main Menu, Display, Snaps, Info & Settings, Layers, Projects", builders below the Project Manager, right column is an optional ~300 px mirror reached only by dragging.
- `SectionId` (lines 144-153) and `SectionId::ALL` (lines 158-167) give the default order: IconHistory, MainMenu, Display, Snaps, InfoSettings, Layers, Projects, BuilderSlot. `default_state()` gives `dock: DockSide::Left`, `order` = index in `ALL`, `height: SECTION_DEFAULT_HEIGHT` (120).
- Titles: `SectionId::title()/bar_title()` at lines 186-210. Layers is "LAYERS" and Info & Settings is "INFO & SETTINGS".
- `keeps_controls_while_collapsed` is true for Snaps and Layers (lines ~214-217), matching the manual statement "Snaps, Layers keep important controls on their closed title bars".
- Widths: `LEFT_RAIL_MIN/MAX/DEFAULT` = 240/420/300 (lines 32-34); `RIGHT_DOCK_MIN/MAX/DEFAULT` = 200/420/300; `NARROW_WINDOW_WIDTH` = 1200 (changed from 900 by the current uncommitted work).
- `PanelId::RightDock` default is `visible: false` (comment: "Docking menus to the right is a user action, so the column starts hidden").
- `DockLayout::resolve` (lines ~650-700): the right dock exists only if some section is docked right AND `width >= NARROW_WINDOW_WIDTH`. If `left + right + CANVAS_MIN_WIDTH > width`, the right dock yields first, then the left rail shrinks.
- `left_sections()` and `right_sections()` at lines 479-487, `dock_section()` at 439.
- `SectionState.height` is persisted and clamped (`set_section_height`, line 404) **but nothing renders from it**. A grep of `standalone/crates` finds `set_section_height` only in a dock_layout unit test, and `chrome.rs` never reads `.height` for sections. The field is dead state today.

### 2.2 Rendering: `standalone/crates/om9-app/src/ui.rs` and `ui/chrome.rs`

- `ui.rs:892-909`: `egui::Panel::left("matrix-sidebar")` with `default_size(regions.left_rail)`, `min_size = LEFT_RAIL_MIN`, `max_size = LEFT_RAIL_MAX`, `resizable(true)`, gray(82) fill, `inner_margin(2)`. It calls `sync_panel_size(PanelId::LeftRail, ui.max_rect().width())` and then `sidebar_ui`.
- `ui/chrome.rs:438-466` `sidebar_ui`: iterates `self.dock.left_sections()` inside one `egui::ScrollArea::vertical()` (`matrix-sidebar-sections`). Each visible section draws a 14 px gradient title bar via `section()` (lines 242-290), then its body via `sidebar_section_ui` (469-627). **There is no per-section height or proportion. Each body takes its natural height and the whole column scrolls.**
- Layers body: `SectionId::Layers => self.layers_ui(ui)` (`chrome.rs:616-618`). `layers_ui` is in `ui/layers.rs:30-`: Lights row plus Hide/Show, two columns of 4+4 rows, advanced toggle for slots 17-32.
- Info & Settings body (`chrome.rs:589-615`): 12 icon buttons, `Units: ...`, `Objects: ...`. Object info, Properties and Settings behaviour is not implemented as a separate panel. These are icon slots in the section, not a "properties dock".
- Other "Layers" strings in `chrome.rs` are **icons/commands, not the panel**: line 556 (Display icon row), line 596 (Info & Settings icon row), `category_palette("Setting")` (line ~1091), `toolbar_palette("Visibility")` (line ~1114). Do not confuse them when auditing.
- Right dock: `ui.rs:939-960` shows `egui::Panel::right("matrix-right-dock")` only when `regions.right_dock_visible`. `chrome.rs:862-931` `right_dock_ui` renders `self.dock.right_sections()`. The right dock is a generic mirror of the left column. There is no dedicated "Properties" or "Layers" right panel (schema 2 removed the invented one: `dock_layout.rs:28-31`).
- Left-rail width oddity (not verified as a bug): the receipts show `left_rail_px = 295` even at 1600 wide, while the plan and tests expect 300. `sync_panel_size` stores `ui.max_rect().width()`, the inner width after the 2 px margins. Treat 295 vs 300 as an open parity question for pixel-exact "y chang" claims.

### 2.3 Order and size rules, as implemented

- Order: persisted per section in `ui-layout.json` (`SectionState.order`, `dock`). Reorder, collapse, float and dock-right are user actions (L3-003). Reset restores the default order.
- Width: left rail default 300 px, clamped 240-420, resizable. The right dock is the same, shown only when a section is docked right and the window is at least 1200 pt wide.
- Height: natural content height, scrolling when the stack is taller than the window. `SectionState.height` is unused.
- Resolution dependence: none for order. Only the right dock (yields below 1200 pt) and the left rail width (shrinks only when the canvas would drop below 320 pt) react to window size.

### 2.4 How the stack fits at supported sizes (derived from screenshots, approximate)

From `build/standalone-validation/ui-artifacts/2026-10-16-m01-l2-01-01-docking/workspace.png` (1600x1000, default, nothing docked right), the section tops are about y = 25 (Icon History), 63, 161, 221, 279, 338 (Layers), 526 (Projects), 676 (Builders), ending at about 845. At 1280x720 the sidebar is about 695 px tall (`sidebar_px` in the receipt). Layers (ending near y 525) fits. Projects roughly fits. Builders falls below the fold and is reached by scrolling. For the 1366x768 capped window (about 688 high) Projects is also partly clipped. These numbers come from reading pixels off one image; they have not been measured by a test.

## 3. What the authoritative Matrix9 description says (question 2)

The earlier note was correct: `OpenMatrix9_Codex_Spec_v2/` and any `MATRIX9_UI_CONTRACT` file are **absent** from the repo (`Test-Path OpenMatrix9_Codex_Spec_v2` is False; no file matches `*MATRIX9_UI_CONTRACT*`). The substitutes in the repo are:

1. **Manual text** `OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - .../reference/text/m8book1__p11-13__a-tour-of-the-interface.md` (identical copy under L2-01-01). It says the left side is "divided into sections by the gray title bars indicating 'Icon History', 'Main Menu', 'Display', 'Snaps', 'Info & Settings', 'Layers', and 'Projects'". It says that "in the space below the Project Manager, the Builder interface ... will appear". It says menus can be rolled up, reordered by dragging above or below another menu, floated, or docked to the far right "in a 300-pixel-wide menu that mirrors its counterpart on the left". It says Snaps and Layers keep controls on their closed bars. It says that when a builder opens, "some of the interface runs off the screen (whether it does or not is dependent on the screen size)".
2. **Manual screenshot** `.../L2-03-07 - .../reference/images/m8book1_p11_img02.png` (502x272). It shows the left column top to bottom: Icon History, Main Menu, Display, Snaps, Info & Settings, a coloured two-column Layers grid, then a small Projects strip at the bottom. The four viewports fill the rest and there is no right dock. This is the clearest visual evidence that Layers is left, directly below Info & Settings.
3. **L3 spec** `OM9-L3-03-07-001` "Matrix Left Sidebar Stack And Order" (`l3.json`): goal "Icon History, Main Menu, Display, Snaps, Info, Layers, Projects", positive oracle "Sidebar displays Icon History, Main Menu, Display, Snaps, Info, Layers, Projects in named order", manual gate `M01-UI`. This is the item that actually owns the order.
4. `docs/core-requirements.json:2101` (`ui_contract: matrix9_described_v1`) repeats the same order and the right dock "~300 px". `skills/openmatrix9-ui/references/ui-evidence.md` lists "Seven sections in screenshot order" and "Sidebar starts at 300 Qt logical pixels". `docs/features/OM9-LAYER-001.md` is legacy FreeCAD evidence and says nothing about placement. `docs/features/OM9-IFACE-001-workspace.md` is a legacy workspace-command slice and has no layout statements. Neither is a layout authority.
5. `ref/matrix/vb6-lite/Matrix90/frmMenuLayers.frm`, `frmMenuInfoSettings.frm`, `frmMenuProjects.frm` and `frmMaster.frm` exist. They are VB6 control declarations that could give pixel positions and sizes of the sections. **I did not read them** (the AGENTS rule is to read only the declaration header and not whole forms). They are the next evidence source if exact heights are needed.

## 4. Fixed, proportional or resizable, and resolution dependence (question 3)

Evidenced:

- Left stack order Icon History, Main Menu, Display, Snaps, Info & Settings, Layers, Projects, then Builders (manual text, screenshot, L3-03-07-001).
- Left and right columns are about 300 px wide. The manual does not say they scale with resolution.
- Menus can be rolled up, reordered, floated and docked right by the user. The default has nothing on the right.
- Content can overflow the screen height, depending on screen size (manual p12). Matrix therefore does not squeeze the sections proportionally. It lets them run off the screen.

Not evidenced (do not claim):

- Whether the Info & Settings / Layers boundary is draggable, or whether section heights are fixed pixels or proportional.
- Whether the left width is fixed at 300 px or resizable (OM9's 240-420 range is an OM9 choice).
- Any change of order or proportion with resolution or DPI. The manual implies none, but it is silent.
- Pixel-exact section heights. The Matrix screenshot is 502x272 and scaled, and the in-repo "Matrix9" capture used by the user is not present.
- What the actual Matrix9 (as opposed to Matrix8 book 1) does. The only Matrix9 sources are the private decompiled forms.

OM9 today: order is the same at every resolution; heights are natural and the column scrolls; only the optional right dock is resolution dependent.

## 5. Tests, receipts, validators and docs that touch this (question 4)

**Code that moves Layers to the right (the root cause, would change):**

- `standalone/crates/om9-app/src/ui.rs:1852-1857` audit hook `OM9_UI_AUDIT_RIGHT_DOCK` docks `SectionId::Layers` right. This file is being edited by the other workflow, so coordinate.
- `tools/validate_m01_l3_002_ui_matrix.py`: sets `OM9_UI_AUDIT_RIGHT_DOCK` at lines ~199 and ~285; comment lines 39-42 say "with Layers docked right"; `expect_right_dock` logic at lines ~88 and ~167. Its `check_receipt` (lines ~113-190) **never checks `dock.left_sections` or `dock.right_sections`**, so the screenshot problem passed validation.

**Tests that use Layers as the right-dock fixture** (the dock model itself is fine; only the chosen section is wrong):

- `standalone/crates/om9-app/tests/shell_docking.rs:254-258` (`narrow_window_yields_the_right_column...`). The same file at ~198 already uses `Snaps`.
- `tests/status_bar_overlap.rs:31-35`.
- `tests/workbench_regions_sizes.rs:83-85`, `:364-367`, `:414-418`.
- `tests/workbench_regions.rs:373-376`, `:405-409`, `:432-436`, `:485-489`.
- In-crate unit tests in `src/dock_layout.rs`: `~958-962` and the new `right_dock_threshold_boundary_is_1200_logical_points` (~975-983) call `dock_section(SectionId::Layers, DockSide::Right)`.
- The 1200 rationale comment on `NARROW_WINDOW_WIDTH` assumes a 300 px right column; it does not mention Layers and stays valid.

**Tests that already assert the correct default and should be kept or strengthened:**

- `tests/shell_docking.rs:20-29` `DOCUMENTED_LEFT` and `left_column_renders_the_documented_sections_with_builders_below_projects` (line 131).
- `tests/startup_contract.rs:98-109` compares `left_sections` with `implementation/fixtures/M01/OM9-L3-01-01-001/positive.json` (`left_sections` includes `layers` between `info-settings` and `projects`).
- `src/dock_layout.rs` tests at ~845-860 (default order) and ~885-905 (reorder).
- `tests/layer_widgets.rs:~537` asserts layer controls are inside `sidebar_rect`, which only holds when Layers is in the left sidebar (default).

**Receipt fields in `ui-receipt.json`** (written at `ui.rs:~1392-1445`): `dock.left_sections`, `dock.right_sections`, `dock.floating_sections`, `dock.right_dock_visible`, `dock.left_rail_px`, `dock.builder_slot_visible`, `regions.sidebar_px`, `regions.right_dock_px`, `regions.plan.*`. There are no per-section rectangles in the receipt, so vertical order and visibility of Layers cannot be asserted from it today.

**Docs and skill text that mention the right Layers dock or would need a note:**

- `.agents/tasks/m01-l3-002-docking/verification.md:39` ("Left rail, four viewports and the right Layers dock visible"); `final-report-vi.md` and `closeout-*.md` reference the same runs.
- `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md` (modified, uncommitted).
- `docs/openmatrix9-progress.json` around line 7831-7913 (L2-01-01 entry; evidence lists `right_sections []` for the baseline run, which is correct, but later L3-002 evidence uses the Layers right dock).
- `skills/openmatrix9-workspace-contract/SKILL.md` section "Vùng workbench docking" (lines ~22-34 in the working tree): says "dock phải nhường khi cửa sổ hẹp hơn 1200 điểm logic" and does not mention Layers, so it is not wrong. It needs one added line only if the user approves the new rule (business-rule consent policy in AGENTS.md).
- `skills/openmatrix9-ui/references/ui-evidence.md` "Current slice acceptance" already says seven sections in screenshot order.

## 6. Ownership: which L3 item owns what (question 5)

- **OM9-L3-01-01-002 Workbench Docking Regions** owns region rectangles only: left rail, centre, optional right dock, command input, toolbar, status. Its oracle says "optional right properties dock". Its exclusion is "Không xây ... Main Menu Matrix lần hai". It must not decide which panel lives in the left column. It needs only a *neutral fixture* to exercise the optional right dock.
- **OM9-L3-01-01-003 Reorder Collapse And Float Panels** owns dragging a menu to the right dock, floating and collapsing (`settle_section_drag`, `dock_menu_section`, `float_*` in `chrome.rs`/`ui.rs`). That is where "user docks Layers right" behaviour belongs, as an explicit user action.
- **OM9-L3-01-01-004 / 005** own default restore and persistence of the layout. They must keep Layers left after Reset and reload.
- **OM9-L3-01-01-008 Builder Panel Docking Slot** owns the reserved Builders area below Projects.
- **OM9-L3-03-07-001 Matrix Left Sidebar Stack And Order** (L2-03-07) owns the left stack order, including "Layers below Info & Settings". The order assertions belong to this item or to the shared `DOCUMENTED_LEFT` test; L3-002 only needs to keep them true. L2-03-07 L3-006/007 own menu collapse/reorder/float integration and default menu layout reset.
- **OM9-L2-03-14 Layers / Object Info / Superselect** owns the *content* of the Layers panel: L3-001 category palette, L3-002 current layer and assignment, L3-003 visible and locked, L3-004 select by layer, L3-005 Object Info overlay, L3-006 superselect, L3-007 property editor, L3-008 persistence check. The needed fix does not touch `ui/layers.rs` or `session/layers.rs`.
- **OM9-L2-03-01 Jewelry Viewport Layout** (L3-001 four viewport grid) owns the viewport layout and is unaffected.

So the follow-up stays inside L3-002 verification tooling and fixtures (plus an order assertion that cites L3-03-07-001), and does not need to reopen the Layers content items.

## 7. Recommended change list (not implemented)

1. **Stop docking Layers right in the audit.** In `ui.rs` (~1852-1857) change the audit hook so it does not move Layers. Prefer a different, non-Layers section as the right-dock fixture. `Snaps` is already used by `shell_docking.rs` and keeps its controls on the closed bar, so it is the least intrusive choice. Better still, split the harness into two clearly named modes: `default` (nothing on the right, used for all "looks like Matrix" screenshots) and `user-docked-right-<section>` (labelled as a user-initiated layout, never as the Matrix default). Rename the env var or keep it with a documented fixture. This file is under concurrent edit, so apply it after wf_bb045d04a4cb1388 lands or coordinate with it.
2. **Run the size matrix in default mode first.** `tools/validate_m01_l3_002_ui_matrix.py` should run every case twice, or run the default mode at every size and the right-dock mode only where it is needed to test the 1200 threshold. The default mode must expect `right_dock_px is None`, so `expect_right_dock` becomes mode specific.
3. **Make the receipt prove the left stack.** Add to `ui-receipt.json` per-section title-bar rects (`section_rects`, already tracked in `MatrixApp.section_rects`, `ui.rs:134`), plus `layers_rect` and `sidebar_scroll_offset`. Extend `check_receipt` to fail when `dock.left_sections` differs from the documented order, when `layers` is not in the left list, when `layers` is in `right_sections` in default mode, or when the Layers title-bar y is not between the Info & Settings and Projects title bars and inside `sidebar_px`. Receipt writer is in `ui.rs` (~1392-1445), also under concurrent edit.
4. **Re-point the tests** listed in section 5 to the neutral fixture (`Snaps`), or add a helper `dock_right_fixture(app)` in a shared test support module so the choice is made in one place. Add one explicit test that Layers stays left and below Info & Settings at 1280x720, 1366x768-capped and 1600x1000 frames, and after Reset Menus and a layout reload (reuse `DOCUMENTED_LEFT`).
5. **Decide what happens to overflow.** Today the whole column scrolls. If the user wants "exactly like Matrix at any resolution", Matrix also runs off the screen (manual p12), so keep scrolling and just make sure Layers is above the fold at the supported minimum. The measured stack suggests it is. Add a frame test asserting the Layers title bar is within the sidebar rect at 1280x720 with no scroll.
6. **Re-capture and replace** the 1280x720, 1366x768 (lap100/125/150), 1440x720/900 and 1280x900 screenshots in default mode, and relabel the right-dock screenshots as a user-docked case.
7. **Docs:** correct `.agents/tasks/m01-l3-002-docking/verification.md` and `final-report-vi.md` and `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md` where they describe "the right Layers dock". Add a skill line to `skills/openmatrix9-workspace-contract/SKILL.md` only after the user approves the wording, for example: "Layers luôn nằm cột trái, ngay dưới Info & Settings, ở mọi độ phân giải; cột phải chỉ có khi người dùng kéo menu sang." Per `AGENTS.md`, present before/after text and ask first.
8. **Optional, only if the user wants pixel parity:** read the `frmMaster.frm` / `frmMenuLayers.frm` declarations for section heights, decide whether `SectionState.height` should be used or removed (it is dead today), and resolve the 295 vs 300 px rail width.

## 8. Uncertainties the user must confirm

1. **What exactly was wrong in the screenshot or app the user looked at.** The strongest explanation is the right-docked Layers in the audit screenshots, but I could not see the user's screen. If the user ran the real app (not `--ui-audit`) and still saw Layers elsewhere, the cause is different, for example a saved `ui-layout.json` with Layers docked right or floating, or Layers scrolled below the fold. Please confirm which one. The Vietnamese "chứ" is also ambiguous ("not X but Y"); I read it as "Layers must be on the left, directly below Info & Settings", which agrees with the Matrix manual.
2. **Is the optional right dock still wanted at all?** L3-002's oracle says "optional right properties dock", but the manual only describes a user-docked mirror column and OM9 schema 2 deliberately removed the invented Properties dock. Confirm that the right dock should remain a generic empty-by-default mirror column and that a different section (Snaps proposed) may be used as the test fixture.
3. **Which resolutions count as "Matrix9 resolution".** Is parity needed only at the supported range (1280x720 up to large desktop, 1366x768 capped) or at any size? Matrix itself overflows rather than reflows.
4. **Overflow policy.** Should Builders scroll off the bottom as in Matrix, or should section heights compress? The manual supports overflow; it does not support proportional compression.
5. **Left rail width:** exactly 300 px, 295 px as now, or resizable 240-420? Only "300-pixel" is evidenced.
6. **Whether to read the private VB6 forms** for exact section heights, since the repo has no Matrix9 screenshot with measurable pixels.

## 8a. Addendum: workspace rule and `docs/analysis/matrix9-main-window-layout.md`

Read after the main investigation. It is a text analysis of one Matrix 9.0 screenshot the user supplied on 2026-10-16 (about 1024x608). It corroborates sections 1-4 and tightens two points.

- **Corroborates:** left stack ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS (directly below Info & Settings), PROJECTS, then builder panels clipped at the window bottom. The column is clipped or scrollable, not shrunk to fit. This agrees with the sidebar scroll behaviour in section 2.2 and with manual p12.
- **No right dock appears in that screenshot.** The workspace rule says not to treat an optional right dock as Matrix9 behaviour without the user's decision. This answers part of uncertainty 2: the audit's right-docked Layers is not Matrix behaviour in any reading, and the generic right dock itself is an OM9 addition. Recommendation 1 and 2 should therefore make the **default** audit and every "matches Matrix" screenshot run with no right dock. Keep the right-dock cases only as clearly labelled OM9-extension tests, and ask the user whether the right dock stays in M01 at all (the L3-002 oracle "optional right properties dock" is not supported by this screenshot).
- **Width is not settled.** The analysis measured the left column at about 214 px of a 1024 px capture (about 21%). The manual says "300-pixel". Both fit a roughly 1420 px wide real window, but the capture may have been downscaled, so I cannot tell whether the column is fixed at 300 px, a fixed fraction of the window, or scales with DPI. The analysis itself says proportions across resolution are not evidenced and asks for a second capture. This reinforces uncertainty 3 and 5: do not invent a proportional rule; ask for a second capture at a different size (for example 1366x768 or 1920x1080) before changing the 240/300/420 rail rules.
- **Command area position** (green two-line `Command:` box at the top of the right workspace, above the toolbar tabs) is described in that file; the current OM9 layout already places the command panel in that position (receipt plan: `command_input` at y=21 above `toolbar`), so this is not a gap. Not re-verified visually beyond the existing screenshots.
- The user-rule line "Minimum supported window: 1280x720 logical" matches the work already done in `startup.rs` and `SKILL.md`.

## 9. What I could not verify

- I did not run the app, tests or the validator; this is static reading plus viewing existing screenshots and receipts. I did not rebuild, so the state of the uncommitted working tree (other workflow) was not tested.
- `OpenMatrix9_Codex_Spec_v2/` and `MATRIX9_UI_CONTRACT` are not in the checkout, so the "UI contract" could not be read. The Matrix8 book 1 extracts and the L3 `l3.json` files were used instead.
- The Matrix reference image `m8book1_p11_img02.png` is only 502x272, so the stated section proportions are visual estimates and are not measured.
- Section pixel heights quoted in 2.4 were estimated by eye from the 1600x1000 screenshot, not by a test.
- I did not open `ref/matrix/vb6-lite/Matrix90/*.frm` (rule: bounded reads only), the legacy Matrix9 binary, or any user-specific `ui-layout.json`.
- I could not confirm which build or mode produced the screenshot the user saw.
