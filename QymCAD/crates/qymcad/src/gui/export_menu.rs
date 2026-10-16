//! THE EXPORT SUBMENU: one item in a menu, the formats inside it.
//!
//! Reported behaviour: the File menu listed "Export to ..." eight times over, one line per format, and the
//! component's menu in the tree did the same. The formats now sit in a submenu of their own, the same one in both
//! places: the exact formats first, the meshes below a line.

/// A format a body can be written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportChoice {
    Exact(qymcad_kernel::ExactFormat),
    Mesh(qymcad_ui_state::MeshFormat),
}

/// Where the submenu is shown: it says "export the project" in the File menu and "export" on a component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExportFrom {
    Project,
    Component,
}

use qymcad_kernel::ExactFormat;
use qymcad_ui_state::MeshFormat;

/// Every format, in the order the submenu lists them, with its hint in the File menu and on a component.
const CHOICES: [(ExportChoice, &str, &str); 8] = [
    (ExportChoice::Exact(ExactFormat::Step), "menu-export-step-hint", "tree-export-step-hint"),
    (ExportChoice::Exact(ExactFormat::Iges), "menu-export-iges-hint", "tree-export-iges-hint"),
    (ExportChoice::Mesh(MeshFormat::Stl), "menu-export-stl-hint", "tree-export-stl-hint"),
    (ExportChoice::Mesh(MeshFormat::Obj), "menu-export-obj-hint", "tree-export-obj-hint"),
    (ExportChoice::Mesh(MeshFormat::Ply), "menu-export-ply-hint", "tree-export-ply-hint"),
    (ExportChoice::Mesh(MeshFormat::Glb), "menu-export-glb-hint", "tree-export-glb-hint"),
    (ExportChoice::Mesh(MeshFormat::ThreeMf), "menu-export-3mf-hint", "tree-export-3mf-hint"),
    (ExportChoice::Mesh(MeshFormat::Amf), "menu-export-amf-hint", "tree-export-amf-hint"),
];

/// The name a person knows the format by - from the table of formats, where it lives once.
pub(crate) fn name_of(choice: ExportChoice) -> &'static str {
    match choice {
        ExportChoice::Exact(f) => crate::gui::exact_entry(f).name(),
        ExportChoice::Mesh(f) => crate::gui::mesh_entry(f).name(),
    }
}

/// Every format the submenu offers, in its order.
pub(crate) fn choices() -> impl Iterator<Item = ExportChoice> {
    CHOICES.iter().map(|(c, _, _)| *c)
}

/// The hint of a format: in the File menu it speaks of the whole project, on a component of that component.
fn hint_of(choice: ExportChoice, from: ExportFrom) -> &'static str {
    let (on_project, on_component) = CHOICES.iter().find(|(c, _, _)| *c == choice).map(|(_, p, c)| (*p, *c)).unwrap_or_default();
    match from {
        ExportFrom::Project => on_project,
        ExportFrom::Component => on_component,
    }
}

/// The submenu, drawn into a menu. Returns the format picked this frame, if one was.
pub(crate) fn export_submenu(ui: &mut egui::Ui, from: ExportFrom) -> Option<ExportChoice> {
    let title = match from {
        ExportFrom::Project => crate::i18n::tr("file-export"),
        ExportFrom::Component => crate::i18n::tr("act-export"),
    };
    let mut chosen = None;
    ui.menu_button(format!("{}  {}", egui_phosphor::regular::EXPORT, title), |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend); // a menu item is a command, not a paragraph
        for choice in choices() {
            if matches!(choice, ExportChoice::Mesh(MeshFormat::Stl)) {
                ui.separator(); // the exact formats above, the meshes below
            }
            if ui.button(crate::i18n::tr1("file-export-as", "format", name_of(choice))).on_hover_text(crate::i18n::tr(hint_of(choice, from))).clicked() {
                chosen = Some(choice);
                ui.close();
            }
        }
    });
    chosen
}
