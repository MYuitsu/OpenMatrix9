// SPDX-License-Identifier: LGPL-2.1-or-later
//! Fixed reference slots for Display, Snaps and Info & Settings. These are
//! presentation adapters only; unimplemented operations remain non-invoking.
use super::*;

pub(super) const DISPLAY: [&str; 11] = [
    "GridON",
    "PreviewCutterON",
    "PreviewShadeON",
    "ShadeSelectedOnlyON",
    "GVGemView_1",
    "GVSurfaceView_1",
    "Dial_Wireframe",
    "Dial_Shaded",
    "Dial_Ghosted",
    "Dial_Tech Shade",
    "Dial_Working Render",
];
pub(super) const SNAP_TOP: [&str; 9] = [
    "ToolsObjectSnapPerpendicularTo",
    "ToolsObjectSnapTangentTo",
    "ToolsObjectSnapIntersection",
    "ToolsObjectSnapPoint",
    "ToolsObjectSnapNear",
    "ToolsObjectSnapQuadrant",
    "ToolsObjectSnapCenter",
    "ToolsObjectSnapMidpoint",
    "ToolsObjectSnapEnd",
];
pub(super) const SNAP_BOTTOM: [&str; 6] = [
    "SnapOnSurface",
    "SnapOnPolysurface",
    "SnapBetween",
    "OrthoSnapON",
    "PlanarSnapON",
    "ProjectSnapON",
];
pub(super) const INFO_TOP: [&str; 11] = [
    "RhinoOptions",
    "ObjectProperties",
    "GVObjectInfo",
    "AllObjectInfo",
    "CommandHistory",
    "ProjectNotes",
    "SuperSelect",
    "RhinoHistoryON",
    "GVHistoryUpdateON",
    "GVHistoryRecordON",
    "InfoSettingsGVClearHistory",
];
pub(super) const INFO_BOTTOM: [&str; 11] = [
    "InfoSettingsViewportTabsToggle",
    "InfoSettingsDisplayProperties",
    "InfoSettingsBoxEdit",
    "InfoSettingsLibraries",
    "InfoSettingsSelectionFilter",
    "RhinoSmartTrackON",
    "InfoSettingsDesignReport",
    "InfoSettingsSmartTargetsGumballON",
    "InfoSettingsGumballAlignment",
    "InfoSettingsRelocateGumball",
    "InfoSettingsGumballON",
];

const REFERENCE_WIDTH: f32 = 296.;

impl MatrixApp {
    pub(super) fn fixed_sidebar_palette(&mut self, ui: &mut egui::Ui, section: SectionId) {
        let height = match section {
            SectionId::Display => 45.,
            SectionId::Snaps => 55.,
            SectionId::InfoSettings => 51.,
            _ => return,
        };
        let scale_x = ui.available_width() / REFERENCE_WIDTH;
        // Wider docks add spacing rather than taller rows, keeping Layers
        // visible in the supported short-window layouts.
        let capacity = scale_x.min(
            (dock_layout::LEFT_RAIL_DEFAULT - dock_layout::LEFT_RAIL_CHROME_PX) / REFERENCE_WIDTH,
        );
        // A live resize must affect these fixed rows as well as generic dock
        // icons. At full size retain their reference capacity and row counts.
        let scale = (capacity * self.palette_icon_scale())
            .max(18. / 23.)
            .min(capacity);
        let (body, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), height * scale),
            Sense::hover(),
        );
        let slot = |x: f32, y: f32, w: f32, h: f32| {
            Rect::from_min_size(
                body.min + Vec2::new(x * scale_x, y * scale),
                Vec2::new(w * scale_x, h * scale),
            )
        };
        let saved_edge = self.column_icon_edge;
        let mut icon_index = 0;
        let mut icon = |app: &mut Self, key: &str, x: f32, y: f32| {
            let rect = Rect::from_min_size(slot(x, y, 23., 23.).min, Vec2::splat(23. * scale));
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt((section.key(), "icon", icon_index))
                    .max_rect(rect),
            );
            icon_index += 1;
            app.column_icon_edge = rect.width();
            let snap_kind = snap_kind_for_icon(key);
            let toggle = snap_kind.is_some()
                || matches!(
                    key,
                    "OrthoSnapON"
                        | "PlanarSnapON"
                        | "ProjectSnapON"
                        | "RhinoSmartTrackON"
                        | "RhinoHistoryON"
                        | "GVHistoryRecordON"
                        | "GVHistoryUpdateON"
                        | "InfoSettingsGVClearHistory"
                );
            let active = snap_kind.map_or_else(
                || match key {
                    "OrthoSnapON" => app.controller.ortho(),
                    "PlanarSnapON" => app.controller.planar(),
                    "ProjectSnapON" => app.controller.project(),
                    "RhinoSmartTrackON" => app.smarttrack_enabled(),
                    "RhinoHistoryON" | "GVHistoryRecordON" => {
                        app.controller.history_record_enabled()
                    }
                    "GVHistoryUpdateON" => app.controller.history_update_enabled(),
                    _ => false,
                },
                |kind| app.snap_enabled(kind),
            );
            let response = if toggle {
                app.icon_toggle(&mut child, key, active, key)
            } else {
                app.icon_slot(&mut child, key, false, key, None)
            };
            if toggle && response.clicked() {
                if let Some(kind) = snap_kind {
                    let on = !app.snap_enabled(kind);
                    app.set_snap_enabled(kind, on);
                    app.status.info(&format!(
                        "{} {}",
                        kind.label(),
                        if on { "on" } else { "off" }
                    ));
                } else {
                    match key {
                        "OrthoSnapON" => {
                            let on = !app.controller.ortho();
                            app.controller.set_ortho(on);
                            app.status.info(if on { "Ortho on" } else { "Ortho off" });
                        }
                        "PlanarSnapON" => {
                            let on = !app.controller.planar();
                            app.controller.set_planar(on);
                            app.status.info(if on { "Planar on" } else { "Planar off" });
                        }
                        "ProjectSnapON" => {
                            let on = !app.controller.project();
                            app.controller.set_project(on);
                            app.status
                                .info(if on { "Project on" } else { "Project off" });
                        }
                        "RhinoSmartTrackON" => {
                            let on = !app.smarttrack_enabled();
                            app.set_smarttrack_enabled(on);
                            app.status.info(if on {
                                "SmartTrack on"
                            } else {
                                "SmartTrack off"
                            });
                        }
                        // Matrix History switches and the per-object clear (`-001`, `-002`, `-004`).
                        "RhinoHistoryON" | "GVHistoryRecordON" => {
                            match app.controller.toggle_history_record() {
                                Ok(message) => {
                                    app.status.info(&message);
                                }
                                Err(error) => {
                                    app.status.error(&format!("{error}"));
                                }
                            }
                        }
                        "GVHistoryUpdateON" => match app.controller.toggle_history_update() {
                            Ok(message) => {
                                app.status.info(&message);
                            }
                            Err(error) => {
                                app.status.error(&format!("{error}"));
                            }
                        },
                        "InfoSettingsGVClearHistory" => {
                            match app.controller.clear_selected_history() {
                                Ok(message) => {
                                    app.status.info(&message);
                                }
                                Err(error) => {
                                    app.status.error(&format!("{error}"));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        };
        match section {
            SectionId::Display => {
                for (i, key) in DISPLAY.iter().enumerate() {
                    let x = if i < 6 {
                        i as f32 * 24.
                    } else {
                        156. + (i - 6) as f32 * 24.
                    };
                    icon(self, key, x, 0.);
                }
            }
            SectionId::Snaps => {
                for (i, key) in SNAP_TOP.iter().enumerate() {
                    icon(self, key, i as f32 * 27., 0.);
                }
                for (i, key) in SNAP_BOTTOM.iter().enumerate() {
                    icon(self, key, i as f32 * 27., 30.);
                }
                icon(self, "GridSnapON", 272., 30.);
            }
            SectionId::InfoSettings => {
                for (row, keys) in [INFO_TOP, INFO_BOTTOM].iter().enumerate() {
                    for (i, key) in keys.iter().enumerate() {
                        let x = if i < 7 {
                            i as f32 * 26.
                        } else {
                            194. + (i - 7) as f32 * 26.
                        };
                        icon(self, key, x, row as f32 * 28.);
                    }
                }
            }
            _ => {}
        }
        self.column_icon_edge = saved_edge;
        let mut control = |label: &str, x: f32, y: f32, w: f32, h: f32, selected: bool| {
            let rect = slot(x, y, w, h);
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt((section.key(), "control", label))
                    .max_rect(rect),
            );
            child.spacing_mut().interact_size = rect.size();
            child.spacing_mut().button_padding = Vec2::new(2. * scale, 0.5 * scale);
            child
                .add_enabled(
                    false,
                    egui::Button::new(
                        egui::RichText::new(label.trim_end_matches(" ▾")).size(10. * scale),
                    )
                    .selected(selected)
                    .min_size(rect.size()),
                )
                .on_hover_text(format!("{label} — operation unavailable"));
            if label.ends_with(" ▾") {
                let center = rect.right_center() - Vec2::new(6. * scale, 0.);
                child.painter().add(egui::Shape::convex_polygon(
                    vec![
                        center + Vec2::new(-3., -1.5) * scale,
                        center + Vec2::new(3., -1.5) * scale,
                        center + Vec2::new(0., 2.) * scale,
                    ],
                    chrome::PANEL_MUTED,
                    Stroke::NONE,
                ));
            }
        };
        match section {
            SectionId::Display => {
                control("Coarse Mesh ▾", 0., 28., 152., 16., false);
                control("Shaded ▾", 156., 28., 119., 16., false);
                let track = slot(281., 0., 15., 44.);
                ui.painter().rect_filled(track, 3, chrome::PANEL_BG);
                ui.painter().rect_stroke(
                    track.shrink(0.5),
                    3,
                    Stroke::new(1., chrome::PANEL_MUTED),
                    egui::StrokeKind::Inside,
                );
                let thumb = slot(285., 9., 7., 23.);
                ui.painter().rect_filled(thumb, 2, chrome::PANEL_MUTED);
                ui.interact(track, ui.id().with("display-slider"), Sense::hover())
                    .on_hover_text("Display adjustment — operation unavailable");
            }
            SectionId::Snaps => {
                control("I", 244., 0., 50., 23., false);
                for (i, label) in ["0.1", "0.25", "0.5", "1.0"].iter().enumerate() {
                    control(
                        label,
                        158. + i as f32 * 28.,
                        31.5,
                        26.,
                        20.,
                        *label == "0.5",
                    );
                }
            }
            _ => {}
        }
    }
}

fn snap_kind_for_icon(key: &str) -> Option<om9_scene::osnap::SnapKind> {
    use om9_scene::osnap::SnapKind;
    Some(match key {
        "ToolsObjectSnapEnd" => SnapKind::End,
        "ToolsObjectSnapMidpoint" => SnapKind::Mid,
        "ToolsObjectSnapCenter" => SnapKind::Center,
        "ToolsObjectSnapQuadrant" => SnapKind::Quadrant,
        "ToolsObjectSnapPoint" => SnapKind::Point,
        "ToolsObjectSnapIntersection" => SnapKind::Intersection,
        "ToolsObjectSnapNear" => SnapKind::Near,
        "ToolsObjectSnapTangentTo" => SnapKind::Tangent,
        "ToolsObjectSnapPerpendicularTo" => SnapKind::Perpendicular,
        "SnapOnSurface" | "SnapOnPolysurface" => SnapKind::Surface,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_ortho_planar_and_smarttrack_palette_slots_toggle_their_state() {
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("toggle slots")).unwrap();
        let run = |app: &mut MatrixApp, section: SectionId, events: Vec<egui::Event>| {
            app.rail_icon_rects.clear();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(311., 400.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.set_width(295.);
                    app.icon_column = IconColumn::Left;
                    app.fixed_sidebar_palette(ui, section);
                },
            );
            output.textures_delta.clear();
        };
        let click = |app: &mut MatrixApp, section: SectionId, index: usize| {
            run(app, section, vec![]);
            let at = app.rail_icon_rects[index].center();
            run(
                app,
                section,
                vec![
                    egui::Event::PointerMoved(at),
                    egui::Event::PointerButton {
                        pos: at,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            run(
                app,
                section,
                vec![egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        };

        assert!(app.snap_enabled(om9_scene::osnap::SnapKind::End));
        click(&mut app, SectionId::Snaps, 8);
        assert!(!app.snap_enabled(om9_scene::osnap::SnapKind::End));
        assert!(!app.controller.project());
        click(&mut app, SectionId::Snaps, 14);
        assert!(app.controller.project());
        click(&mut app, SectionId::Snaps, 12);
        assert!(app.controller.ortho());
        click(&mut app, SectionId::Snaps, 13);
        assert!(app.controller.planar());
        click(&mut app, SectionId::InfoSettings, 16);
        assert!(!app.smarttrack_enabled());
    }

    #[test]
    fn reference_slots_keep_counts_rows_and_group_gaps_at_every_dock_width() {
        for width in [235., 295., 355., 455.] {
            for (section, count, first_row) in [
                (SectionId::Display, 11, 11),
                (SectionId::Snaps, 16, 9),
                (SectionId::InfoSettings, 22, 11),
            ] {
                let mut app = MatrixApp::new(Document::new("fixed palette")).unwrap();
                let context = egui::Context::default();
                let mut output = context.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width + 16., 400.),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        ui.set_width(width);
                        app.icon_column = IconColumn::Left;
                        app.fixed_sidebar_palette(ui, section);
                    },
                );
                output.textures_delta.clear();
                let slots = &app.rail_icon_rects;
                assert_eq!(slots.len(), count, "{section:?} at {width}");
                let top = slots[0].top();
                assert!(
                    slots[..first_row]
                        .iter()
                        .all(|rect| (rect.top() - top).abs() < 0.01)
                );
                if first_row < count {
                    let bottom = slots[first_row].top();
                    assert!(bottom > top);
                    assert!(
                        slots[first_row..]
                            .iter()
                            .all(|rect| (rect.top() - bottom).abs() < 0.01)
                    );
                }
                let left = slots[0].left();
                for rect in slots {
                    assert!(rect.left() >= left && rect.right() <= left + width + 0.01);
                }
                if section == SectionId::InfoSettings {
                    // The reference has seven icons, a gap, then four in each row.
                    assert!(
                        slots[7].left() - slots[6].right() > slots[1].left() - slots[0].right()
                    );
                    assert!((slots[7].left() - slots[18].left()).abs() < 0.01);
                }
                if section == SectionId::Display {
                    assert!(
                        slots[6].left() - slots[5].right() > slots[1].left() - slots[0].right()
                    );
                }
                if section == SectionId::Snaps {
                    // Grid Snap remains after the four numeric controls, at far right.
                    assert!(slots[15].left() > slots[8].left());
                }
                assert!(!app.controller.is_running());
                assert!(app.controller.document().entities.is_empty());
            }
        }
    }

    #[test]
    fn every_reference_slot_has_an_authored_icon_and_correct_right_hand_groups() {
        for key in DISPLAY
            .iter()
            .chain(SNAP_TOP.iter())
            .chain(SNAP_BOTTOM.iter())
            .chain(INFO_TOP.iter())
            .chain(INFO_BOTTOM.iter())
            .chain(["GridSnapON"].iter())
        {
            assert!(
                crate::icons::icon_name(key).is_some(),
                "missing authored icon: {key}"
            );
        }
        assert_eq!(
            &INFO_TOP[7..],
            &[
                "RhinoHistoryON",
                "GVHistoryUpdateON",
                "GVHistoryRecordON",
                "InfoSettingsGVClearHistory"
            ]
        );
        assert_eq!(
            &INFO_BOTTOM[7..],
            &[
                "InfoSettingsSmartTargetsGumballON",
                "InfoSettingsGumballAlignment",
                "InfoSettingsRelocateGumball",
                "InfoSettingsGumballON"
            ]
        );
    }
}
