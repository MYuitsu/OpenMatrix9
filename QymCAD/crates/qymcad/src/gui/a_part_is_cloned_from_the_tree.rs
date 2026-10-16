//! A PART IS CLONED FROM ITS MENU IN THE TREE, THE CLONE SAYS SO, AND STEPPING INTO IT OPENS ITS ORIGINAL.
//!
//! The owner's word: a clone is the same part once more - it is placed and mated on its own, an edit of the original
//! rebuilds it, stepping into it from the assembly lands in the original, and the tree marks it as a clone.
#[cfg(test)]
mod tests {
    use crate::gui::a_component_stepped_into_is_not_lit::tests::{calm, double_click_at};
    use crate::gui::import_door::tests::{click, frame, running, spot};
    use crate::gui::App;
    use qymcad_core::model::Id;

    /// A part with a body at the top of a new document; returns it and its name.
    fn a_part(app: &mut App) -> (Id, String) {
        super::super::joint_flow::tests::add_part_at(app, 0.0);
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        app.exit_context();
        let part = app.project.components.iter().find(|c| c.kind == qymcad_core::feature::ComponentKind::Part && !app.project.component_bodies(c.id).is_empty()).expect("the part");
        (part.id, part.name.clone())
    }

    fn right_click(at: egui::Pos2) -> Vec<Vec<egui::Event>> {
        let button = |pressed| vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() }];
        vec![vec![egui::Event::PointerMoved(at)], button(true), button(false)]
    }

    /// Every row of the tree that reads `name`, top to bottom.
    fn rows(texts: &[(String, egui::Rect)], name: &str) -> Vec<egui::Pos2> {
        let mut out: Vec<egui::Pos2> = texts.iter().filter(|(t, r)| t == name && r.center().x < 400.0).map(|(_, r)| r.center()).collect();
        out.sort_by(|a, b| a.y.total_cmp(&b.y));
        out
    }

    #[test]
    fn a_part_is_cloned_from_its_menu_and_marked() {
        let (mut app, ctx) = running();
        let (part, name) = a_part(&mut app);
        calm(&mut app, &ctx); // the part is built in the background, and its window swallows clicks
        for _ in 0..40 {
            let _ = frame(&mut app, &ctx, Vec::new()); // the tree of the context just left for settles its layout
        }
        let steps = app.disk.edits.undo.len();
        let texts = frame(&mut app, &ctx, Vec::new());
        let row = *rows(&texts, &name).first().unwrap_or_else(|| panic!("no row {name:?}"));
        for events in right_click(row) {
            let _ = frame(&mut app, &ctx, events);
        }
        let texts = frame(&mut app, &ctx, Vec::new());
        let item = spot(&texts, &crate::i18n::tr("act-clone-part")).unwrap_or_else(|| panic!("the menu of a part offers no clone; it shows {:?}", texts.iter().map(|(t, _)| t).collect::<Vec<_>>()));
        let _ = frame(&mut app, &ctx, click(item));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        let clones: Vec<Id> = app.project.components.iter().filter(|c| c.id != part && c.name == name).map(|c| c.id).collect();
        assert_eq!(clones.len(), 1, "one clone under the part's name, found {clones:?}");
        assert_eq!(app.project.instance_origin(clones[0]), part, "the new part is not a clone of the original");
        // THE TREE SAYS SO, in words beside the name: an icon alone reads as a part
        let texts = frame(&mut app, &ctx, Vec::new());
        let lines = rows(&texts, &name);
        assert_eq!(lines.len(), 2, "the tree shows {} rows {name:?}", lines.len());
        let mark = crate::i18n::tr("tree-clone-mark");
        assert!(texts.iter().any(|(t, r)| *t == mark && (r.center().y - lines[1].y).abs() < 4.0), "the clone's row is not marked {mark:?}");
        assert!(!texts.iter().any(|(t, r)| *t == mark && (r.center().y - lines[0].y).abs() < 4.0), "the original is marked as a clone");
        assert_eq!(app.disk.edits.undo.len(), steps + 1, "making a clone is not one step of undo");
    }

    #[test]
    fn stepping_into_a_clone_opens_its_original() {
        let (mut app, ctx) = running();
        let (part, name) = a_part(&mut app);
        let root = app.project.root;
        assert!(app.project.clone_part(part, root).is_some(), "a clone is made");
        crate::gui::commands::resync_after_topology_change(&mut app.part_ctx());
        let _ = frame(&mut app, &ctx, Vec::new());
        let texts = frame(&mut app, &ctx, Vec::new());
        let clone_row = *rows(&texts, &name).get(1).unwrap_or_else(|| panic!("no second row {name:?}"));
        double_click_at(&mut app, &ctx, clone_row);
        let here = qymcad_ui_state::current_ctx_id(&app.active_path, &app.project);
        assert_eq!(here, part, "stepping into the clone did not land in its original");
    }

    /// THE TREE WITH A CLONE IN IT, SEEN: the program as a person sees it after cloning a part from its menu, into
    /// `target/look/clone-tree.png`. The capture without a graphics device is right for the panels.
    #[test]
    #[ignore = "a picture to look at"]
    fn the_clone_in_the_tree_is_looked_at() {
        let (mut app, ctx) = running();
        let (_, name) = a_part(&mut app);
        calm(&mut app, &ctx);
        let texts = frame(&mut app, &ctx, Vec::new());
        let row = *rows(&texts, &name).first().expect("the part's row");
        for events in right_click(row) {
            let _ = frame(&mut app, &ctx, events);
        }
        let texts = frame(&mut app, &ctx, Vec::new());
        let item = spot(&texts, &crate::i18n::tr("act-clone-part")).expect("the menu item");
        let _ = frame(&mut app, &ctx, click(item));
        for _ in 0..3 {
            let _ = frame(&mut app, &ctx, Vec::new());
        }
        let dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/look"));
        std::fs::create_dir_all(&dir).expect("a folder for the picture");
        let bg = app.scheme.pal.viewport_bg();
        let img = crate::gui::help_raster::shot_ui([1400, 950], bg, |ui| {
            let ctx = &ui.ctx().clone();
            crate::gui::apply_theme(&mut app.scheme, &app.set, ctx);
            let shell = crate::gui::shell(&app.set);
            for slot in qymcad_shell::Slot::ORDER {
                shell.run_slot(slot, ui, &mut app);
            }
        });
        std::fs::write(dir.join("clone-tree.png"), crate::gui::color_image_to_png(&img).expect("PNG")).expect("writing");
    }

    /// A CLONE IS LED BY HAND like any part: grabbed in the viewport and dragged, it goes where the hand leads, while
    /// its geometry keeps repeating the original's.
    ///
    /// Reported behaviour: clones in an assembly cannot be moved - they stay nailed to the very coordinates of the
    /// original.
    #[test]
    fn a_clone_is_led_by_hand() {
        let mut app = App::default();
        super::super::joint_flow::tests::add_part_at(&mut app, 0.0);
        let root = app.project.root;
        while qymcad_ui_state::current_ctx_id(&app.active_path, &app.project) != root {
            app.exit_context();
        }
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        let part = app.project.components.iter().find(|c| c.kind == qymcad_core::feature::ComponentKind::Part && !app.project.component_bodies(c.id).is_empty()).expect("the part").id;
        let clone = app.project.clone_part(part, root).expect("a clone is made");
        crate::gui::commands::resync_after_topology_change(&mut app.part_ctx());
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        crate::gui::commands::refresh_edges(&mut app.part_ctx());
        let body = *app.project.component_bodies(clone).first().expect("the clone has a body");
        app.viewing.mode_3d = true;
        app.viewing.cam.init = true;
        app.viewing.cam.scale = 5.0;
        app.viewing.cam.target = [10.0, 10.0, 5.0];
        app.workbench = super::super::Workbench::Assembly;
        let (orig0, clone0) = (app.project.component_transform(part), app.project.component_transform(clone));
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 700.0));
        // Shift held throughout: a part is taken with Shift and the left button
        let raw = |events| egui::RawInput { screen_rect: Some(rect), events, modifiers: egui::Modifiers::SHIFT, ..Default::default() };
        let press = |at, down| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: down, modifiers: egui::Modifiers::SHIFT };
        let ctx = egui::Context::default();
        super::super::install_fonts(&ctx);
        let _ = ctx.run_ui(raw(Vec::new()), |c| app.viewport(c));
        let wt = app.project.body_display_transform(body, root);
        let top = app.project.regen_faces.get(&body).and_then(|fs| fs.iter().max_by(|a, b| a.centroid.z.total_cmp(&b.centroid.z))).expect("the clone has faces");
        let aim = qymcad_core::feature::apply12(&wt, [top.centroid.x, top.centroid.y, top.centroid.z]);
        let basis = app.viewing.cam.basis();
        let at = qymcad_ui_state::Screen { cam: &app.viewing.cam, set: &app.set, rect, basis: &basis }.at(aim).0;
        let _ = ctx.run_ui(raw(vec![egui::Event::PointerMoved(at)]), |c| app.viewport(c));
        let _ = ctx.run_ui(raw(vec![press(at, true)]), |c| app.viewport(c));
        for k in 1..=6 {
            let _ = ctx.run_ui(raw(vec![egui::Event::PointerMoved(at + egui::vec2(12.0 * k as f32, 0.0))]), |c| app.viewport(c));
        }
        let _ = ctx.run_ui(raw(vec![press(at + egui::vec2(72.0, 0.0), false)]), |c| app.viewport(c));
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        let (orig1, clone1) = (app.project.component_transform(part), app.project.component_transform(clone));
        let shift = |a: [f64; 12], b: [f64; 12]| (b[3] - a[3]).hypot(b[7] - a[7]).hypot(b[11] - a[11]);
        assert!(shift(clone0, clone1) > 5.0, "the clone was dragged 72 px and moved {:.2} mm (the original {:.2} mm)", shift(clone0, clone1), shift(orig0, orig1));
    }
}
