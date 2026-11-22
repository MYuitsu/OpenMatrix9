// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::viewport_menu::{DisplayPreset, ENTRIES, MenuAction};

const SNAP_STEPS: [f64; 4] = [0.1, 0.25, 0.5, 1.0];

pub(super) fn style_dialog(ui: &mut egui::Ui) {
    use egui::{FontId, TextStyle};
    let heading = super::typography::semibold(ui.ctx(), 17.);
    let style = ui.style_mut();
    style.text_styles.insert(TextStyle::Heading, heading);
    style.text_styles.insert(TextStyle::Body, FontId::proportional(13.));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(13.));
    style.text_styles.insert(TextStyle::Small, FontId::proportional(12.));
    style.spacing.item_spacing = egui::vec2(8., 4.);
    style.spacing.button_padding = egui::vec2(6., 2.);
    style.spacing.interact_size.y = 22.;

}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(super) struct Preferences {
    pub(super) zoom_scale: f64,
    pub(super) single_click_maximize: bool,
    wireframe_grid: crate::display::GridVisibility,
    shaded_grid: crate::display::GridVisibility,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            zoom_scale: 0.9,
            single_click_maximize: false,
            wireframe_grid: Default::default(),
            shaded_grid: Default::default(),
        }
    }
}

impl Preferences {
    pub(super) fn grid(
        self,
        mode: om9_scene::workspace::DisplayMode,
    ) -> crate::display::GridVisibility {
        use om9_scene::workspace::DisplayMode::*;
        match mode {
            Wireframe => self.wireframe_grid,
            Shaded => self.shaded_grid,
        }
    }

    fn valid(self) -> bool {
        (0.05..=0.999).contains(&self.zoom_scale)
    }
}

#[derive(Default)]
pub(super) struct Options {
    pub(super) preferences: Preferences,
    pub(super) draft: Option<Draft>,
    pub(super) page: Option<Selection>,
    path: Option<PathBuf>,
}

#[derive(Clone)]
pub(super) struct Draft {
    preferences: Preferences,
    grid_snap: bool,
    snap_step: f64,
    object_snaps: bool,
    ortho: bool,
    planar: bool,
    project: bool,
    smarttrack: bool,
    gumball: om9_scene::gumball_view::Mode,
    gumball_settings: crate::session::gumball::GumballSettings,
    dimension_style: om9_core::annotation::Style,
}

impl Options {
    pub(super) fn load(&mut self, path: &std::path::Path) -> Result<()> {
        self.path = Some(path.to_owned());
        use std::io::Read;
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(Error::Io(error.to_string())),
        };
        let mut bytes = Vec::new();
        file.take(4097)
            .read_to_end(&mut bytes)
            .map_err(|e| Error::Io(e.to_string()))?;
        if bytes.len() > 4096 {
            return Err(Error::Budget("Options preferences file"));
        }
        let preferences: Preferences =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("Options preferences"))?;
        if !preferences.valid() {
            return Err(Error::Invalid("Zoom scale"));
        }
        self.preferences = preferences;
        Ok(())
    }

    fn save(&self, preferences: Preferences) -> Result<()> {
        if preferences == self.preferences {
            return Ok(());
        }
        let Some(path) = &self.path else {
            return Ok(());
        };
        let parent = path
            .parent()
            .ok_or(Error::Invalid("Options preferences path"))?;
        let bytes =
            serde_json::to_vec(&preferences).map_err(|_| Error::Invalid("Options preferences"))?;
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
        let staging = parent.join(format!(".options-{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(&staging, bytes).map_err(|e| Error::Io(e.to_string()))?;
        if let Err(error) = std::fs::rename(&staging, path) {
            let _ = std::fs::remove_file(staging);
            return Err(Error::Io(error.to_string()));
        }
        Ok(())
    }
}

impl MatrixApp {
    pub(super) fn options_draft(&self) -> Draft {
        Draft {
            preferences: self.options.preferences,
            grid_snap: self.controller.grid_snap(),
            snap_step: self.controller.grid_snap_step_mm(),
            object_snaps: self.object_snaps_enabled(),
            ortho: self.controller.ortho(),
            planar: self.controller.planar(),
            project: self.controller.project(),
            smarttrack: self.smarttrack_enabled(),
            gumball: self.gumball_mode(),
            gumball_settings: self.controller.gumball_settings(),
            dimension_style: self.controller.annotation_style().clone(),
        }
    }

    pub(super) fn apply_options_draft(&mut self, draft: Draft) -> Result<()> {
        if !draft.preferences.valid() {
            return Err(Error::Invalid("Zoom scale"));
        }
        if !SNAP_STEPS.contains(&draft.snap_step) {
            return Err(Error::Invalid("Grid snap spacing"));
        }
        draft.gumball_settings.validate()?;
        draft.dimension_style.validate()?;
        self.options.save(draft.preferences)?;
        self.options.preferences = draft.preferences;
        self.controller.set_grid_snap(draft.grid_snap);
        self.controller.set_grid_snap_step_mm(draft.snap_step);
        if draft.object_snaps != self.object_snaps_enabled() {
            self.set_object_snaps_enabled(draft.object_snaps);
        }
        if draft.smarttrack != self.smarttrack_enabled() {
            self.set_smarttrack_enabled(draft.smarttrack);
        }
        self.controller.set_ortho(draft.ortho);
        self.controller.set_planar(draft.planar);
        self.controller.set_project(draft.project);
        if self.controller.gumball_settings() != draft.gumball_settings {
            self.controller
                .apply_gumball_settings(draft.gumball_settings)?;
        }
        self.controller
            .set_annotation_style(draft.dimension_style)?;
        self.set_gumball_mode(draft.gumball);
        Ok(())
    }

    pub(super) fn options_page(&self, ui: &mut egui::Ui, selection: Selection, draft: &mut Draft) {
        let page = selection.page;
        if let Some((preset, name)) = selection.mode {
            ui.label(format!("Display Modes  /  {name}"));
            ui.heading(page.label);
            if MenuAction::Preset(preset).availability().is_err() {
                ui.weak("This display mode is unavailable in the OM9 viewport.");
            } else if page.label == "Grid" {
                let grid = if preset == DisplayPreset::Wireframe {
                    &mut draft.preferences.wireframe_grid
                } else {
                    &mut draft.preferences.shaded_grid
                };
                ui.checkbox(&mut grid.lines, "Show grid lines");
                ui.checkbox(&mut grid.axes, "Show grid axes");
                fields(
                    ui,
                    &[
                        "Use document grid",
                        "World axes",
                        "Grid on top",
                        "Grid transparency",
                    ],
                );
                return;
            }
            if page.label == name {
                ui.label("Expand this mode to view Grid, Objects, Shadows and Other Settings.");
                return;
            }
            fields(ui, page.fields);
            return;
        }
        ui.heading(page.label);
        match page.label {
            "View" => view_page(ui, &mut draft.preferences),
            "Grid" => {
                let response = ui.checkbox(&mut draft.grid_snap, "Grid snap");
                ui.ctx().data_mut(|data| {
                    data.insert_temp(egui::Id::new("options-grid-snap-rect"), response.rect)
                });
                egui::ComboBox::from_label("Snap spacing (mm)")
                    .selected_text(draft.snap_step.to_string())
                    .show_ui(ui, |ui| {
                        for step in SNAP_STEPS {
                            ui.selectable_value(&mut draft.snap_step, step, step.to_string());
                        }
                    });
                fields(ui, page.fields);
            }
            "Model" => {
                let doc = self.controller.document();
                ui.label(format!("Units: {:?}", doc.page_unit));
                if let Some(tolerance) = doc.tolerance() {
                    ui.label(format!("Absolute tolerance: {} mm", tolerance.absolute_mm));
                    ui.label(format!("Relative tolerance: {}", tolerance.relative));
                    ui.label(format!(
                        "Angular tolerance: {:.4}°",
                        tolerance.angular_rad.to_degrees()
                    ));
                } else {
                    ui.weak("This document has no stored tolerance.");
                }
                fields(ui, page.fields);
            }
            "Mesh" => {
                ui.label(format!(
                    "Viewport approximation: {}",
                    self.controller.viewport_detail().label()
                ));
                fields(ui, page.fields);
            }
            "Current dimension defaults" => {
                let style = &mut draft.dimension_style;
                ui.label("Used for new annotations in this session.");
                egui::Grid::new("dimension-default-fields").show(ui, |ui| {
                    ui.label("Text height (mm)");
                    ui.add(egui::DragValue::new(&mut style.height_mm).range(0.001..=1e6));
                    ui.end_row();
                    ui.label("Arrow size (mm)");
                    ui.add(egui::DragValue::new(&mut style.arrow_mm).range(0.001..=1e6));
                    ui.end_row();
                    ui.label("Display precision");
                    ui.add(egui::DragValue::new(&mut style.precision).range(0..=8));
                    ui.end_row();
                    ui.label("Annotation scale");
                    ui.add(egui::DragValue::new(&mut style.scale).range(0.001..=1e6));
                    ui.end_row();
                });
            }
            "Smart Track" => {
                ui.checkbox(&mut draft.smarttrack, "Enable Smart Track");
                fields(ui, page.fields);
            }
            "Modeling Aids" => {
                ui.checkbox(&mut draft.object_snaps, "Enable object snaps");
                ui.checkbox(&mut draft.ortho, "Ortho");
                ui.checkbox(&mut draft.planar, "Planar");
                ui.checkbox(&mut draft.project, "Project snaps to CPlane");
                fields(ui, page.fields);
            }
            "Gumball" => {
                use crate::session::GumballAlignment;
                use om9_scene::gumball_view::Mode;
                ui.horizontal(|ui| {
                    ui.label("Handle mode");
                    for (mode, label) in [
                        (Mode::Move, "Move"),
                        (Mode::Rotate, "Rotate"),
                        (Mode::Scale, "Scale"),
                    ] {
                        ui.selectable_value(&mut draft.gumball, mode, label);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Alignment");
                    for (alignment, label) in [
                        (GumballAlignment::World, "World"),
                        (GumballAlignment::Object, "Object"),
                        (GumballAlignment::CPlane, "CPlane"),
                    ] {
                        ui.selectable_value(
                            &mut draft.gumball_settings.alignment,
                            alignment,
                            label,
                        );
                    }
                });
                ui.checkbox(&mut draft.gumball_settings.snappy, "Snappy drag");
                let mut strength_percent = draft.gumball_settings.drag_strength * 100.;
                if ui
                    .add(
                        egui::Slider::new(&mut strength_percent, 1.0..=200.0)
                            .suffix("%")
                            .text("Drag strength"),
                    )
                    .changed()
                {
                    draft.gumball_settings.drag_strength = strength_percent / 100.;
                }
                fields(ui, page.fields);
            }
            "Plug-ins" => {
                ui.label(if self.controller.addon_host().is_some() {
                    "OM9 add-on host connected"
                } else {
                    "OM9 add-on host unavailable"
                });
                fields(ui, page.fields);
            }
            "OpenGL" => {
                if let Some(state) = &self.gpu_state {
                    let adapter = state.adapter.get_info();
                    ui.label(format!("GPU: {}", adapter.name));
                    ui.label(format!("Backend: {:?}", adapter.backend));
                } else {
                    ui.label("GPU adapter unavailable in this session");
                }
                fields(ui, page.fields);
            }
            "Display Modes" => {
                ui.label("Select a display mode to inspect its properties.");
                ui.label("Available in the viewport: Wireframe and Shaded.");
            }
            "Document Properties" => {
                ui.label(&self.controller.document().name);
            }
            "Rhino Options" => {
                ui.label("Application settings");
            }
            _ => fields(ui, page.fields),
        }
    }
}

fn fields(ui: &mut egui::Ui, fields: &[&str]) {
    if fields.is_empty() {
        return;
    }
    let label_font = super::typography::semibold(ui.ctx(), 13.);
    egui::Grid::new(ui.next_auto_id())
        .num_columns(2)
        .spacing(egui::vec2(24., 6.))
        .striped(true)
        .show(ui, |ui| {
            for field in fields {
                ui.label(egui::RichText::new(*field).font(label_font.clone()));
                ui.weak("Unavailable");
                ui.end_row();
            }
        });
}

fn view_page(ui: &mut egui::Ui, draft: &mut Preferences) {
    ui.strong("Zoom");
    ui.horizontal(|ui| {
        ui.label("Scale factor");
        ui.add(
            egui::DragValue::new(&mut draft.zoom_scale)
                .range(0.05..=0.999)
                .speed(0.005),
        );
    });
    ui.strong("Viewport properties");
    let response = ui.checkbox(&mut draft.single_click_maximize, "Single-click maximize");
    ui.ctx()
        .data_mut(|data| data.insert_temp(egui::Id::new("options-maximize-rect"), response.rect));
    fields(
        ui,
        &[
            "Linked viewports",
            "Wrap cursor at viewport borders",
            "Default 35mm camera lens length",
        ],
    );
    ui.separator();
    ui.strong("Pan");
    fields(
        ui,
        &[
            "Screen fraction",
            "Reverse keyboard action",
            "Always pan parallel views",
            "Auto adjust camera target after Pan and Zoom",
        ],
    );
    ui.strong("Named views");
    fields(
        ui,
        &[
            "Named views set CPlane",
            "Named views set projection",
            "Named views set clipping planes",
        ],
    );
    ui.strong("Rotate");
    fields(
        ui,
        &[
            "Increment in divisions of a circle",
            "Reverse keyboard action",
            "Rotation axes",
        ],
    );
    ui.strong("Dynamic redraw");
    fields(ui, &["Frames per second"]);
}

#[derive(Clone, Copy)]
struct Page {
    label: &'static str,
    fields: &'static [&'static str],
    children: &'static [Page],
}

const fn leaf(label: &'static str, fields: &'static [&'static str]) -> Page {
    Page {
        label,
        fields,
        children: &[],
    }
}

const fn branch(
    label: &'static str,
    fields: &'static [&'static str],
    children: &'static [Page],
) -> Page {
    Page {
        label,
        fields,
        children,
    }
}

const GENERAL: Page = leaf("General", &[]);
const VIEW: Page = branch(
    "View",
    &[],
    &[
        leaf("Display Modes", &[]),
        leaf(
            "OpenGL",
            &[
                "Hardware acceleration",
                "Display anti-aliasing",
                "Driver-specific options",
            ],
        ),
    ],
);
const DOCUMENT: Page = branch(
    "Document Properties",
    &[],
    &[
        branch(
            "Annotation",
            &["Annotation scale"],
            &[
                branch(
                    "Dimensions",
                    &["Named document dimension styles"],
                    &[leaf("Current dimension defaults", &[])],
                ),
                leaf(
                    "Hatch",
                    &["Hatch patterns", "Pattern scale", "Pattern rotation"],
                ),
                leaf(
                    "Linetypes",
                    &["Line types", "Current linetype", "Linetype scale"],
                ),
            ],
        ),
        leaf(
            "Grid",
            &[
                "Grid line count",
                "Grid line spacing",
                "Major line interval",
                "Grid axes",
                "World axes",
            ],
        ),
        leaf(
            "Mesh",
            &[
                "Render mesh settings",
                "Density",
                "Maximum angle",
                "Edge length",
                "Refine mesh",
            ],
        ),
        leaf(
            "Notes",
            &["Document notes", "Show notes when opening the document"],
        ),
        leaf(
            "Render",
            &["Renderer", "Image dimensions", "Background", "Lighting"],
        ),
        branch(
            "Rendering",
            &["Render environment", "Ground plane"],
            &[leaf(
                "Safe Frame",
                &["Show safe frame", "Action area", "Title area"],
            )],
        ),
        branch(
            "Units",
            &["Model and layout unit policies"],
            &[
                leaf("Model", &["Change model units", "Scale model geometry"]),
                leaf(
                    "Layout",
                    &["Layout units", "Layout precision", "Layout tolerances"],
                ),
            ],
        ),
        leaf("Web Browser", &["Document browser settings"]),
    ],
);
const APPLICATION: Page = branch(
    "Rhino Options",
    &[],
    &[
        leaf("Alerter", &["Warning preferences"]),
        leaf(
            "Aliases",
            &["Alias", "Command macro", "Import and export aliases"],
        ),
        branch(
            "Appearance",
            &["Language", "Command prompt", "Interface style"],
            &[leaf(
                "Colors",
                &[
                    "Viewport background",
                    "Grid colors",
                    "Axis colors",
                    "Selection color",
                ],
            )],
        ),
        branch(
            "Clayoo",
            &["Requires the Clayoo plug-in"],
            &[
                leaf("License", &["Clayoo license status"]),
                leaf("Hotkeys", &["Clayoo shortcut assignments"]),
                leaf(
                    "Update and statistics",
                    &["Clayoo update preferences", "Clayoo statistics preferences"],
                ),
            ],
        ),
        branch(
            "Context Menu",
            &["Context menu configuration"],
            &[
                leaf("Control Point Menu", &["Control point actions"]),
                leaf("Object Menu", &["Object actions"]),
            ],
        ),
        branch(
            "Files",
            &["Templates", "Autosave", "File locations"],
            &[leaf("Search Paths", &["Search path list", "Search order"])],
        ),
        GENERAL,
        leaf("Idle Processor", &["Idle processing"]),
        leaf("Keyboard", &["Key assignments", "Command macros"]),
        leaf("Libraries", &["Library locations", "Library configuration"]),
        leaf("Licenses", &["Application license information"]),
        branch(
            "Modeling Aids",
            &["Ortho angle"],
            &[
                leaf("Nudge", &["Nudge coordinate system", "Nudge increments"]),
                leaf(
                    "Smart Track",
                    &[
                        "Activation delay",
                        "Tracking lines",
                        "Smart point appearance",
                    ],
                ),
                leaf(
                    "Cursor ToolTips",
                    &["Enable cursor tooltips", "Tooltip content", "Position"],
                ),
                leaf("Gumball", &["Handle appearance", "Handle sizes"]),
            ],
        ),
        leaf("Mouse", &["Mouse buttons", "Viewport selection gestures"]),
        leaf(
            "Plug-ins",
            &["Rhino plug-in settings", "Plug-in load policy"],
        ),
        leaf("Rendering", &["Renderer preferences"]),
        leaf(
            "RhinoScript",
            &["Startup script list", "RhinoScript preferences"],
        ),
        leaf(
            "Selection Menu",
            &["Overlapping object selection menu", "Menu appearance"],
        ),
        branch(
            "Toolbars",
            &["Toolbar collections", "Toolbar layout"],
            &[leaf(
                "Size and Styles",
                &["Button size", "Toolbar appearance"],
            )],
        ),
        leaf(
            "Updates and Statistics",
            &["Update preferences", "Statistics preferences"],
        ),
        VIEW,
    ],
);

const POINTS: Page = leaf(
    "Points",
    &[
        "Show points",
        "Point style and size",
        "Show point clouds",
        "Point cloud style and size",
    ],
);
const CURVES: Page = leaf(
    "Curves",
    &[
        "Show curves",
        "Curve color",
        "Thickness",
        "Transparency",
        "Line pattern",
        "Line caps and joins",
    ],
);
const OBJECTS: &[Page] = &[
    POINTS,
    CURVES,
    leaf(
        "Surfaces",
        &[
            "Surface visibility",
            "Surface edges",
            "Isocurves",
            "Backface appearance",
        ],
    ),
    leaf(
        "Meshes",
        &["Mesh visibility", "Mesh wires", "Mesh wire thickness"],
    ),
];
const LINE_OBJECTS: &[Page] = &[
    POINTS,
    CURVES,
    leaf("Lines", &["Line color", "Line width", "Line pattern"]),
];

#[derive(Clone, Copy)]
pub(super) struct Selection {
    page: Page,
    mode: Option<(DisplayPreset, &'static str)>,
    id: egui::Id,
}

impl Selection {
    pub(super) fn id(self) -> egui::Id {
        self.id
    }

    pub(super) fn is_general(self) -> bool {
        self.page.label == GENERAL.label && self.mode.is_none()
    }
}

fn tree_node(
    ui: &mut egui::Ui,
    selected: &mut Option<Selection>,
    parent: egui::Id,
    depth: usize,
    page: Page,
    mode: Option<(DisplayPreset, &'static str)>,
) {
    let id = parent.with(page.label);
    let display_modes = page.label == "Display Modes" && mode.is_none();
    let has_children = display_modes || !page.children.is_empty();
    let open_id = id.with("expanded");
    let mut open = ui.ctx().data(|data| {
        data.get_temp::<bool>(open_id)
            .unwrap_or(depth == 0 || page.label == "View")
    });
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 15.);
        if has_children {
            if ui
                .small_button(if open { "−" } else { "+" })
                .on_hover_text(if open { "Collapse" } else { "Expand" })
                .clicked()
            {
                open = !open;
                ui.ctx().data_mut(|data| data.insert_temp(open_id, open));
            }
        } else {
            ui.add_space(16.);
        }
        let current = selected.map_or(page.label == "View", |entry| {
            entry.id == id || entry.id == egui::Id::NULL && entry.page.label == page.label
        });
        let label = if depth == 0 || current {
            egui::RichText::new(page.label).font(super::typography::semibold(ui.ctx(), 13.))
        } else {
            egui::RichText::new(page.label)
        };
        let response = ui.selectable_label(current, label);
        if mode.is_none() && page.label == "Grid" {
            ui.ctx().data_mut(|data| {
                data.insert_temp(egui::Id::new("options-tree-grid-rect"), response.rect)
            });
        }
        if response.clicked() {
            *selected = Some(Selection { page, mode, id });
        }
    });
    if !open {
        return;
    }
    if display_modes {
        for entry in ENTRIES {
            let Some(MenuAction::Preset(preset)) = entry.action else {
                continue;
            };
            let name = match preset {
                DisplayPreset::ClayooSculpt => "Clayoo Sculpt",
                DisplayPreset::DetectNakedEdges => "Detect Naked Edge",
                _ => entry.label,
            };
            let object_pages = match preset {
                DisplayPreset::Artistic
                | DisplayPreset::Pen
                | DisplayPreset::ArtColor
                | DisplayPreset::ArtColorWires
                | DisplayPreset::Chalk
                | DisplayPreset::Floorplan
                | DisplayPreset::Legacy
                | DisplayPreset::MatrixBlue
                | DisplayPreset::MatrixWhite
                | DisplayPreset::Pastel => LINE_OBJECTS,
                _ => OBJECTS,
            };
            let pages = [
                leaf(
                    "Grid",
                    &[
                        "Use document grid",
                        "Show grid",
                        "Grid axes",
                        "World axes",
                        "Grid on top",
                        "Grid transparency",
                    ],
                ),
                branch(
                    "Objects",
                    &["Object visibility", "Object colors", "Object material"],
                    object_pages,
                ),
                leaf(
                    "Shadows",
                    &[
                        "Shadow map size",
                        "Shadow color",
                        "Bias",
                        "Blur",
                        "Clipping radius",
                    ],
                ),
                leaf(
                    "Other Settings",
                    &["Background", "Shading", "Lighting", "Technical display"],
                ),
            ];
            let preset_id = id.with(name);
            mode_row(ui, selected, id, depth + 1, name, preset, &pages, preset_id);
        }
    } else {
        for &child in page.children {
            tree_node(ui, selected, id, depth + 1, child, mode);
        }
    }
}

fn mode_row(
    ui: &mut egui::Ui,
    selected: &mut Option<Selection>,
    parent: egui::Id,
    depth: usize,
    name: &'static str,
    preset: DisplayPreset,
    pages: &[Page],
    id: egui::Id,
) {
    let mode = Some((preset, name));
    let open_id = id.with("expanded");
    let mut open = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(open_id).unwrap_or(false));
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 15.);
        if ui.small_button(if open { "−" } else { "+" }).clicked() {
            open = !open;
            ui.ctx().data_mut(|data| data.insert_temp(open_id, open));
        }
        let current = selected.is_some_and(|entry| entry.id == id);
        let label = if current {
            egui::RichText::new(name).font(super::typography::semibold(ui.ctx(), 13.))
        } else {
            egui::RichText::new(name)
        };
        if ui.selectable_label(current, label).clicked() {
            *selected = Some(Selection {
                page: leaf(name, &["Grid", "Objects", "Shadows", "Other Settings"]),
                mode,
                id,
            });
        }
    });
    if open {
        for &page in pages {
            tree_node(ui, selected, parent.with(name), depth + 1, page, mode);
        }
    }
}

pub(super) fn tree(ui: &mut egui::Ui, selected: &mut Option<Selection>) {
    let root = ui.id();
    tree_node(ui, selected, root, 0, DOCUMENT, None);
    tree_node(ui, selected, root, 0, APPLICATION, None);
}

pub(super) fn view() -> Selection {
    Selection {
        page: VIEW,
        mode: None,
        id: egui::Id::NULL,
    }
}

pub(super) fn general() -> Selection {
    Selection {
        page: GENERAL,
        mode: None,
        id: egui::Id::NULL,
    }
}

pub(super) fn document_properties() -> Selection {
    Selection {
        page: DOCUMENT,
        mode: None,
        id: egui::Id::NULL,
    }
}
