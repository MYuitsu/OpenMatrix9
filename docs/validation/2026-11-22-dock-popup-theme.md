# Dock, tool panels and popups on Windows

The November 22 plan covers presentation of the standalone Rust workspace. Command parameters, picked sources, preview workers and Undo belong to the controller. View → Layout Profiles manages named layouts and the startup choice. Each profile stores presentation state. The file keeps a schema-v2 root for the startup layout, so an existing layout becomes the first profile without losing its arrangement.

## Baseline

The baseline captures from the preceding Windows executable use an isolated profile on a 96-DPI display. The Gem Loader has its own light window. Ring Rail and Layers occupy the left stack. F6 is modeless and the unsaved-document prompt owns a modal backdrop.

[Snaps and Projects](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/snaps-projects.png) · [Gem Loader](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/gem-loader.png) · [Ring Rail](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/ring-rail.png) · [Layers](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/layers.png) · [F6](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/f6.png) · [File confirmation](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/before/confirmation.png)

## Window inventory

The tool catalog uses each command owner's stable command or feature ID. Captions can be translated independently. Each tool header offers collapse, placement and close. The controller or owning editor supplies the close action. Modeless tools allow viewport interaction outside the panel. Text editors own their keyboard input.

| Surface and entry | Rust owner | Profile identity | Initial placement and close |
| --- | --- | --- | --- |
| Icon History, Main Menu, Display, Snaps, Info & Settings, Layers, Projects | `ui/chrome.rs`, sidebar palettes, layers and projects | `SectionId::ALL` | Left column. Drag the header, use its position menu, collapse, or close and reopen through View → Workspace panels. Floating menus also offer Dock home. |
| Builders reservation below Projects | `ui/chrome.rs` | `builder-slot` | Fixed insertion area. |
| Gem Loader, Custom, Emerald, Baguette, Positioner from Gems or CMD | `ui/gem.rs` | `GemTool::command()` | Builders. Close cancels the outstanding gem. |
| Gem on Curve and Gem on Surface from Gems, CMD and supported F6 edit | `ui/gem.rs` | `GemPlacementTool::command()` | Builders. Close cancels placement and its preview. |
| Halo, Cluster, Channel, Tapered, Between and Orient | `ui/gem_cluster.rs` | `ClusterTool::command()` | Builders. Close cancels the layout session. |
| Auto and Manual Pavé, Azure, Prong, Sphere and Springs | `ui/gem_pave.rs` | `PaveTool::command()` | Builders. Close cancels the preview. |
| Gem Cutter, Cutter Library and the four row cutters | `ui/gem_cutters.rs`, `ui/channel_cutters.rs` | `CutterTool::feature()`, `RowTool::feature()` | Builders. Close cancels the cutter command. |
| Ring Rail toolbar, Ring Rail, Rail Profile, Rail Sweep, Outside Rail, Ring Resizer, Eternity Band, Signet Ring, Raised Band, Award Ring | `ui/ring_rail_toolbar.rs`, `ui/chrome.rs` | `BuilderKind::command()` | Builders. Close releases the toolbar or cancels the active builder. |
| Add-on hosted builder and Metal Weights | `ui/builder_panel.rs` | Add-on owner and command code, `metal-weights` | Builders. Close follows the add-on session or Metal panel owner. |
| Gem Properties, Gem Map, Gem Reporter | `ui/gem.rs`, `ui/gem_edits.rs` | `om9-gem-properties`, `om9-gem-map`, `om9-gem-report` | Floating. Close abandons the editor or ends the report session. |
| History, Builder Styles and Profile Library | `ui/chrome.rs` | `om9-history-report`, `om9-builder-styles`, `om9-builder-profile-library` | Floating. Close dismisses the report or browser. |
| Object Health Report, Add-ons, Gumball settings | `ui.rs`, `ui/addons_window.rs`, `ui/files.rs` | `om9-object-health-report`, `om9-addons`, `gumball-settings` | Floating. Close dismisses the tool. Add-on installation and shortcut servicing run independently. |
| Project Manager from Projects or CMD | `ui/projects.rs` | `project-manager-overview` | Floating. Close releases the manager's Info draft. |
| F6, Command History | `ui.rs`, `ui/command.rs` | `om9-command-search`, `om9-command-history` | Floating by default, also dockable. F6 retains its pin and accepts the legacy Command Search rectangle. F2 and F6 still toggle their respective panels. |
| Ring Rail shapes, add-on session options, Project details | `ui/ring_rail_library.rs`, `ui/session_dialog.rs`, `ui/projects.rs` | `ring-rail-shape-library`, add-on dialog code, `project-metadata` | Dockable child editors. Close releases or hides the owner's draft. Add-on options use the add-on owner and command code for placement. |
| STL Export options and progress, memory recovery | `ui/files.rs`, `ui/undo_settings.rs` | `stl-profile`, `stl-export-progress`, `memory-recovery` | Export options and progress are dockable. Closing the progress panel requests cancellation. Recovery runs in a modal that requires confirmation before removing Undo steps. |
| New/Open/Save/close confirmation, layer dialogs, Ring Rail replacement | `ui/files.rs`, `ui/layer_dialogs.rs`, `ui/builder_panel.rs` | `owned-file-operation`, `owned-layer-dialog`, `ring-rail-replace` | Modal backdrop. Resolve or cancel with the owner before returning to the workspace. |
| Project directory editor, New Project and project confirmations | `ui/projects.rs` | `project-directory-editor`, `new-project-dialog`, `project-confirmation` | Modal with draft/generation checks. |
| Named layout editor from View | `ui/chrome.rs` | `layout-profile-editor` | Create, rename or duplicate a profile. Save and Enter accept a valid name. Escape cancels. |
| Options and undo preferences | `ui/undo_settings.rs`, `ui/options.rs` | `undo-settings` | Modal. Apply or cancel the preferences draft. |
| Main menus, viewport titles, panel placement and flyouts | `ui/chrome.rs`, `ui/viewport_menu.rs`, `ui/tool_host.rs` | Owner button or view ID | Anchored menu. Selection, Escape or an outside click closes it. |
| Context menus for palettes, Projects and Job Bags | `ui/sidebar_palettes.rs`, `ui/projects.rs` | Originating widget | Pointer-anchored menu with the originating selection. |
| ComboBox, color picker, tooltip and disabled reason | egui child widget in each owner | Widget ID or explicit salt | Dropdowns anchor to the control, tooltips follow hover and use the global theme. |
| Overlapping object/edge/face chooser | `ui.rs` | `om9-pick-popup` | Anchored at the pick. A row selects the candidate and consumes that release. |
| CMD suggestions, command and builder option tokens | `ui/command.rs` | `command-suggestions`, `cmd-session-tokens`, `cmd-builder-tokens` | Suggestions anchor to CMD. The two token strips use the CMD color contract. |
| Add-on file-drop hint | `ui/addons_window.rs` | `addon-drop-hint` | Centered feedback while a file hovers. |
| Open/Save/export/import file and folder pickers, startup failure | Windows adapters and `om9-native-owner` | Owned HWND and operation token | Windows supplies appearance, focus, cancellation and window modality. |

## Verification

### Windows pointer runs

The recordings used Win32 mouse input on standalone OM9, an isolated profile directory and a 1624 × 1011 client at 96 DPI. The four-panel run checked the saved section after each release and located the painted header at the destination. [Right dock → canvas → left dock, all four panels](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/workspace-docking.mp4) · [Destination receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/workspace-drags.json)

| Workspace panel | Operation in the floating panel | Evidence |
| --- | --- | --- |
| Snaps | Clicked a snap toggle twice, then returned the panel to the left dock. The control visibly changed on the first click. | [Video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/named/Snaps-drag.mp4) |
| Info & Settings | Hovered RhinoOptions, opened OpenMatrix9 Options, clicked behind its modal backdrop and cancelled. Layers stayed visible. | [Video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/named/InfoSettings-drag.mp4) · [Options modal](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/options-from-floating-info.png) |
| Layers | Started with a point on Metal 01. Selected Metal 02 in the floating list and created a point on it. Undo removed the new point and retained the original object ID. | [Video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/named/Layers-drag.mp4) · [Active layer](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/named/Layers-active-metal02.png) |
| Projects | Moved the filmstrips and project rows. Created and selected `workshop` in the isolated catalog. The two-row catalog remained byte-identical on a profile switch from right to left. Active row index 1 is project ID 2. | [Video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/named/Projects-drag.mp4) · [Project receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/selected-project-dock.json) |

### Commands, profiles and displays

- Gem Loader retained an 8 mm round stone and the picked point through both docks and the canvas. Apply created one object and Undo removed it. [Native drag video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/acceptance/gem-loader-docking.mp4) · [Controller and layout receipts](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/acceptance/drag-receipt.json)
- Ring Rail retained its 20 mm diameter when moved right and when switching profiles twice. Apply created one curve and one Undo entry. [Native video](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/ring-rail-profile-switch.mp4) · [Controller receipts](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/rail-profile-receipt.json)
- F6 moved between the left dock, right dock and canvas. [Left](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/f6-left.png) · [Right](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/f6-right.png) · [Floating](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/f6-float.png)
- Created `studio` from View and chose it as the startup profile. Switched to `Matrix`, closed the process and reopened OM9. Snaps reopened on the right and Projects floated. Reset Layout on Matrix preserved the saved Studio sections. [Reopened workspace](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/reopened-studio.png) · [Restart receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/restart.json)
- Both physical monitors reported 96 DPI. With Ring Rail active at 22 mm, maximizing on the portrait monitor produced a 1080 × 1849 client. The right-hand tool floated, the saved section was unchanged, and moving back to the primary monitor restored the dock. Gem on Curve and Gem on Surface replay supplied 1.0, 1.25 and 1.5 native pixels per logical point, preserved exact preview stations and source IDs at widths 1199 and 1200, and checked Apply, Undo and Cancel. [Portrait](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/secondary-monitor.png) · [Return to primary](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/returned-primary.png)

### Automated coverage

| Contract | Checks |
| --- | --- |
| Panel registration, floating and ordering | `dock_layout_persistence` exercises every registered tool ID on both sides and floating, plus schema-v2 migration, a second profile, startup, duplicate-name rejection, closed-tool state and isolated reset. `panel_ops` checks drop interpretation, recovery and the single UI mutation path. |
| Interaction | `shell_docking` replays real egui input, including Escape during a drag. `project_manager_ui` exercises project rows, search, files, drafts, modal decisions and scrollbars. `chrome_actions` covers menu, F2 and CMD entry paths. |
| Hosted tools and editors | `builder_controls`, `builder_panel`, `builder_slot`, `gem_panel`, `gem_placement`, `f6_context` and `property_editor_adapter` exercise the owners. The Sweep2 add-on replay moves the options editor through both columns and back to floating, clicks controls, and creates its surface through OK. |
| Popups, colors, focus | `layer_dialogs`, `settings_commands` and `chrome_actions` exercise their owner flows. The source guard checks the constructors and documents the seven custom Area sites. Add-on UI replay exercises Vietnamese text. Windows owns the native file picker appearance. |

The four workspace panels were also dragged by egui pointer events through right, floating and left placement at 1.0, 1.25 and 1.5 native pixels per logical point. All 36 drops preserved grid snap, the active project, the document and the first point of a pending Line command. On the final Windows binary, two additional GPU-rendered runs collapsed and expanded Projects through its header. Each receipt records the effective scale and the unchanged sidebar order. [125% receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/scale-1.25/ui-receipt.json) · [125% image](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/scale-1.25/workspace.png) · [150% receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/scale-1.5/ui-receipt.json) · [150% image](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/release/scale-1.5/workspace.png)

### Build and test results

The Windows debug executable built with `cargo build -j 2 -p om9-app --bin om9` and `CARGO_TARGET_DIR=L:/om9-target`. The SHA-256 of the executable used for both scaled captures was `9251f51eb7207e7817e721c02a857bfe3f27631f435721fe7ab7d01263e84e33`.

| Integration suite | Passing | Ignored |
| --- | ---: | ---: |
| Shell docking | 15 | 0 |
| Panel operations | 33 | 0 |
| Dock defaults and persistence | 7 + 9 | 0 |
| Chrome actions | 39 | 0 |
| Snap modes | 48 | 2 |
| Project manager UI | 33 | 1 |
| Layer dialogs and category palette | 11 + 6 | 0 |
| Settings commands | 24 | 0 |

The source audit returned zero errors for 16,249 source files, 596 icon bindings and 595 SVG files. Artifact pruning retained the latest ten runs per root and the runs referenced by records.

The broader add-on integration suite had 38 passes, four ignored and two failures. The Union icon returned native command `OM9-SOLID-003` while its test expects an add-on. The solid-menu fixture expected a Box and observed zero objects. The app library unit target also has compilation errors in existing BRep fixtures. The independently compiled window-constructor guard passed both checks.

Native video covers the four workspace sections, Gem Loader and Ring Rail. Registry-wide placement coverage comes from the layout model. Individual screenshots of every add-on and cutter at all three positions and human visual approval of each popup remain open. Both physical monitors reported 96 DPI. The higher scale captures use application zoom.

## Gem panel readability

The Gem families now use the same scoped egui style through the tool host in both docks and the floating host. Panel title, controls and secondary labels use 15, 13 and 12 logical points. The title is warm yellow, the selected shape has a mint border, and the main action has dark text on mint. Palette contrast is 12.44:1 for body text and 10.00:1 for the active button. The Loader reflows the shape grid and dimension fields. Loader and placement controls scroll separately from the command actions. First float uses 400 × 550 for the Loader and 370 × 475 for Curve and Surface.

- Windows at 96 DPI, isolated standalone profile. Opened Loader from Gems, verified it on the left and right docks and floated it using the header menu. Selected Diamond Oval 10.40 × 8.00 mm and clicked Apply. One BRep was created and one Undo removed it. [Float shape gallery](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/loader-float.png) · [Size controls](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/gem-loader-popup.png) · [Apply receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/apply-state.json)
- Observed nine Curve preview stones on a Line, then cancelled. Surface showed four preview stones in the dock and as a floating panel. Clicking Apply added four stones and an Undo entry. Undo retained the source plane and line. [Curve dock](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/curve-left.png) · [Surface float](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/surface-float.png) · [Surface Apply](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/surface-apply-state.json) · [Surface Undo](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-readability/aligned/undo-state.json)
- The theme batch passed `gem_panel` 27, `gem_placement` 21, `shell_docking` 15 and `dock_layout_persistence` 9. After adjusting the floating sizes, the final build passed `gem_placement` 21, `shell_docking` 15 and `panel_ops` 33. Native captures cover the three main Gem tools at standard DPI. The hosted Gem variants share the style, with individual visual review outstanding.
- The [openmatrix9-ui-theme skill](../../skills/openmatrix9-ui-theme/SKILL.md) was installed and checked with the skill validator. The general OM9 UI skill routes popup and dock requests to it.

### Illustrated Gem Loader sizes

Gem Shape and Size share a selectable tile renderer. Size tiles draw an outline from the preset's procedural gem specification and use a single scale per page to show relative size. The exact measurement remains below each image.

On the standalone Windows build at 96 DPI, the Oval tiles were visible in the left dock and the floating card. Clicking the picture for 10.40 × 8.00 mm updated both the selection and viewport preview. Apply created one BRep and one Undo entry. Undo removed it. Round tiles remained visible when the panel was moved to the right dock. `gem_panel` passed 27 tests and `gem_placement` passed 21. [Floating gallery](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-thumbnails/Gem-Loader-Size.png) · [Left dock](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-thumbnails/size-left.png) · [Right dock](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-thumbnails/size-right.png) · [Apply receipt](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-thumbnails/apply-state.json)

### Compact Gem Shape gallery and size dial

Gem Loader opens to a five-column shape gallery. Selecting a shape opens its illustrated size dial with up to 12 OM9 presets per page and numbered tabs. Round uses a diameter and other shapes show length and width in the center and in each preset's tooltip. The center uses the current gem specification for the drawing and measurement, and estimates carat weight from the mesh volume and material density. The first floating panel is 320 × 525 logical points.

On Windows at 96 DPI, opened Gem Loader from Gems in the left dock, floated it, and selected a 1.40 mm Round and a 5.20 × 4.00 mm Oval by their drawings. The selected sizes and the viewport preview updated. The Oval stayed selected on page 2 after moving to the right dock. Apply committed a single `Oval Diamond Cut` BRep and Undo removed it. [Shape, Round and Oval](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-dial/Gem-Loader-Shape-Round-Oval.png) · [Right dock](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-dial/oval-right.png) · [Apply](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-dial/apply-state.json) · [Undo](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-size-dial/undo-state.json)

The default-feature build in `L:/om9-target` passed `gem_panel` (28 tests, including both dial page boundaries) and `gem_placement` (21 tests) with `-j 2`. The public source audit reported no errors.

### Matrix9 size catalog and tabs

The size dial now takes the original cut and shape dimensions in the original order. There are 1,476 presets across 51 regular catalogs and six corner-to-corner catalogs. A direct data comparison verified every pair, depth ratio and weight coefficient against the installed library. The page boundary preset appears in both adjacent tabs. Diamond Round has 23 / 22 / 22 entries and Diamond Oval has 13 / 11. The depth percentage uses the width, or the corner-to-corner dimension in that mode. [Mapping](matrix9-gem-loader-size-evidence.md) · [Data comparison](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/catalog-compare.json) · [Size tables](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/Matrix9-Gem-Loader-sizes.csv)

On Windows at standard DPI, opened Gem Loader from Gems and switched among all Diamond Round and Oval tabs in the floating panel. Selected Ø 2.90 mm Round and 6.00 × 4.00 mm Oval. The Oval preview read 2.4492 mm depth and 61.23% for the factory depth ratio. Apply created one Oval BRep and one Undo removed it. Clicking the Triangle center displayed the alternate corner-to-corner measurement with a 6.00 × 5.204 mm footprint. The final spacing change was rebuilt and visually checked with both the denser Round tab and Oval tab 2. [Final gallery and dials](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/Gem-Loader-Matrix9-Tabs.png) · [Oval tab 2](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/oval-tab-2-final.png) · [Apply](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/apply-state.json) · [Undo](../../build/standalone-validation/ui-artifacts/2026-11-22-dock-popup-theme/gem-matrix9-catalog/undo-state.json)

Cargo validation in `L:/om9-target` passed `gem_panel` (28), `gem_placement` (21) and both compiled `gem_assets` variants (24 and 22). The public source audit passed. The fixture compares nominal dimensions and tab behavior. Facet geometry is procedural OM9 geometry.
