// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-02-002/003`: CPlane presets are reachable through the viewport menu.
use om9_app::session::Controller;
use om9_app::viewport_menu::MenuAction;
use om9_core::{budget::Limits, model::Document};
use om9_scene::camera::{CPlane, CPlanePreset};
use om9_scene::workspace::ViewId;

#[test]
fn viewport_menu_applies_a_preset_to_the_captured_view_only() {
    let mut c = Controller::new(Document::new("cplane menu"), Limits::default()).unwrap();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    let neighbor = c.workspace.views[&ViewId::Perspective].clone();
    c.apply_view_menu(&menu, MenuAction::SetCPlane(CPlanePreset::Front))
        .unwrap();
    assert_eq!(
        c.workspace.views[&ViewId::LookingDown].cplane,
        CPlane {
            // Top started at revision 1; one menu assignment bumps it once.
            revision: 2,
            ..CPlane::preset(CPlanePreset::Front)
        }
    );
    assert_eq!(c.workspace.views[&ViewId::Perspective], neighbor);
}
