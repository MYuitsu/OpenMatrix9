// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::workspace::ViewId;

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) -> egui::FullOutput {
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    )
}
fn frame_at(
    app: &mut MatrixApp,
    context: &egui::Context,
    size: Vec2,
    events: Vec<Event>,
) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn secondary_click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) -> egui::FullOutput {
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Secondary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    )
}

fn text_positions(output: &egui::FullOutput, label: &str) -> Vec<Pos2> {
    output
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let egui::Shape::Text(text) = &clipped.shape {
                (text.galley.job.text == label).then(|| text.visual_bounding_rect().center())
            } else {
                None
            }
        })
        .collect()
}
fn datum_strokes(output: &egui::FullOutput) -> usize {
    let color = egui::Color32::from_rgb(255, 170, 40);
    output
        .shapes
        .iter()
        .filter(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { stroke, .. } => stroke.color == color,
            egui::Shape::Path(path) => {
                matches!(path.stroke.color, eframe::epaint::ColorMode::Solid(c) if c == color)
            }
            _ => false,
        })
        .count()
}

fn startup() -> (MatrixApp, egui::Context, egui::FullOutput) {
    let mut app = MatrixApp::new(Document::new("chrome pointer fixtures")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    let output = frame(&mut app, &context, vec![]);
    (app, context, output)
}
fn begin_numeric_draft(app: &mut MatrixApp, context: &egui::Context) {
    frame(app, context, vec![Event::Text("Line".into())]);
    frame(app, context, vec![key(Key::Enter)]);
    frame(app, context, vec![Event::Text("0,0,".into())]);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "0,0,");
}
fn finish_numeric_draft(app: &mut MatrixApp, context: &egui::Context) {
    frame(app, context, vec![]);
    frame(app, context, vec![Event::Text("0".into())]);
    assert_eq!(
        app.transcript.draft(),
        "0,0,0",
        "view selection lost the editable numeric draft/focus"
    );
    frame(app, context, vec![key(Key::Enter)]);
    frame(app, context, vec![Event::Text("4,5,0".into())]);
    frame(app, context, vec![key(Key::Enter)]);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn snaps_are_noninvoking_even_where_the_old_sidebar_displayed_point_and_line() {
    let (mut app, context, output) = startup();
    let mut in_snaps = false;
    let mut positions = Vec::new();
    for clipped in &output.shapes {
        if let egui::Shape::Text(text) = &clipped.shape {
            if text.galley.job.text == "SNAPS" {
                in_snaps = true;
            }
            if text.galley.job.text == "INFO & SETTINGS" {
                in_snaps = false;
            }
        }
        if in_snaps
            && let egui::Shape::Mesh(mesh) = &clipped.shape
            && mesh.texture_id != egui::TextureId::default()
            && clipped.clip_rect.contains(mesh.calc_bounds().center())
        {
            positions.push(mesh.calc_bounds().center());
        }
    }
    assert!(positions.len() >= 4, "snap icons were not rendered");
    for position in positions {
        click(&mut app, &context, position);
        assert!(
            !app.controller.is_running(),
            "sidebar snap icon invoked geometry at {position:?}"
        );
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn help_popup_contains_help_and_about_without_feature_catalog_actions() {
    let (mut app, context, output) = startup();
    let help = text_positions(&output, "Help")[0];
    click(&mut app, &context, help);
    let opened = frame(&mut app, &context, vec![]);
    assert!(!text_positions(&opened, "Command help").is_empty());
    assert!(!text_positions(&opened, "About OpenMatrix9").is_empty());
    assert!(text_positions(&opened, "Matrix Interface").is_empty());
    assert!(text_positions(&opened, "Line").is_empty());
    assert!(!app.controller.is_running());
}

#[test]
fn icons_scale_with_the_screen_size() {
    let context = egui::Context::default();
    let mut small = MatrixApp::new(Document::new("small screen")).unwrap();
    frame_at(&mut small, &context, Vec2::new(1280., 720.), Vec::new());
    let mut large = MatrixApp::new(Document::new("large screen")).unwrap();
    frame_at(&mut large, &context, Vec2::new(1600., 1000.), Vec::new());
    assert!(large.screen_icon_scale() > small.screen_icon_scale());
    assert!(large.toolbar_icon_edge() > small.toolbar_icon_edge());
    assert!(large.left_icon_edge >= small.left_icon_edge);
}

#[test]
fn pinned_menu_lists_have_expected_counts() {
    for (name, expected) in [
        ("Curve", 58),
        ("Surface", 40),
        ("Solid", 43),
        ("Transform", 40),
        ("SubD", 58),
        ("Art", 7),
        ("Builder", 16),
        ("Tools", 31),
        ("Gems", 35),
        ("Setting", 12),
        ("Cutters", 11),
        ("Render", 23),
        ("File", 13),
        ("View", 15),
        ("Utilities", 22),
        ("Measure", 17),
        ("Custom", 0),
    ] {
        assert_eq!(
            om9_app::menu_catalog::category(name).unwrap().len(),
            expected
        );
    }
}

#[test]
fn sphere_variant_icons_start_the_recovered_modes() {
    for (index, prompt) in [(0,"First diameter endpoint"),(1,"First point"),(2,"First point"),(3,"Pick center on curve"),(4,"Select tangent curve"),(5,"Add points")] {
        let (mut app, context, _) = startup();let solid=app.main_category_rect(2).unwrap().center();
        click(&mut app,&context,solid);frame(&mut app,&context,vec![]);
        context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);
        let icon=app.rail_icon_rects[2+index];click(&mut app,&context,icon.center());
        assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-014"));
        assert!(app.controller.prompt().starts_with(prompt),"{}",app.controller.prompt());assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn box_variant_icons_start_the_recovered_modes() {
    for (index, prompt) in [(15,"First corner"),(16,"Start of edge"),(17,"Start of edge"),(18,"Center of base")] {
        let (mut app, context, _) = startup();
        let solid=app.main_category_rect(2).unwrap().center();click(&mut app,&context,solid);frame(&mut app,&context,vec![]);
        let icon=app.rail_icon_rects[2+index];click(&mut app,&context,icon.center());
        assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-012"));
        assert!(app.controller.prompt().starts_with(prompt),"{}",app.controller.prompt());
        assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn multi_pipe_menu_icon_starts_multiple_selection() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    let icon = app.rail_icon_rects[2 + 13];
    click(&mut app, &context, icon.center());
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-011"));
    assert_eq!(app.controller.prompt(), "Select curves for Pipe (Enter when done)");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn create_solid_menu_icon_starts_shared_command() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    assert_eq!(app.main_menu_category(), 2);
    for index in [2 + 5] {
        let rect = app.rail_icon_rects[index];
        click(&mut app, &context, rect.center());
        assert_eq!(app.controller.active_feature(), Some("SolidCreateSolid"));
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.cancel();
    }
}

#[test]
fn boolean_split_menu_icon_starts_shared_command() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    let rect = app.rail_icon_rects[2 + 4];
    click(&mut app, &context, rect.center());
    assert_eq!(app.controller.active_feature(), Some("SolidBooleanSplit"));
    assert!(app.controller.prompt().contains("first set"));
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn truncated_cone_right_click_starts_two_point_variant() {
    let (mut app,context,_)=startup();
    let solid=app.main_category_rect(2).expect("Solid category").center();
    click(&mut app,&context,solid); frame(&mut app,&context,vec![]);
    context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));
    frame(&mut app,&context,vec![]);
    let icon=app.rail_icon_rects[2+12]; secondary_click(&mut app,&context,icon.center());
    assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-020"));
    assert_eq!(app.controller.prompt(),"First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(),0);
}

#[test]
fn ellipsoid_right_click_starts_diameter_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);
    let icon = app.rail_icon_rects[2+6];
    secondary_click(&mut app, &context, icon.center());
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-013"));
    assert_eq!(app.controller.prompt(), "First end of first axis");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn torus_right_click_starts_two_point_variant() {
    let (mut app,context,_)=startup();
    let solid=app.main_category_rect(2).expect("Solid category").center();
    click(&mut app,&context,solid); frame(&mut app,&context,vec![]);
    context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);
    let icon=app.rail_icon_rects[2+7]; secondary_click(&mut app,&context,icon.center());
    assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-015"));
    assert_eq!(app.controller.prompt(),"First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(),0);
}

#[test]
fn cone_right_click_starts_two_point_variant() {
    let (mut app,context,_)=startup();let solid=app.main_category_rect(2).expect("Solid category").center();click(&mut app,&context,solid);frame(&mut app,&context,vec![]);
    context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);
    let icon=app.rail_icon_rects[2+11];secondary_click(&mut app,&context,icon.center());assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-019"));
    assert_eq!(app.controller.prompt(),"First diameter endpoint");assert_eq!(app.controller.coordinator().undo_len(),0);
}

#[test]
fn pyramid_right_click_starts_five_point_star() {
    let (mut app,context,_)=startup();let solid=app.main_category_rect(2).expect("Solid category").center();
    click(&mut app,&context,solid);frame(&mut app,&context,vec![]);context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);let icon=app.rail_icon_rects[2+10];secondary_click(&mut app,&context,icon.center());
    assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-018"));app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("5").unwrap();
    assert_eq!(app.controller.prompt(),"Second star radius or point (Automatic)");app.controller.submit("2").unwrap();app.controller.submit("6").unwrap();
    let e=app.controller.document().entities.values().next().unwrap();let om9_core::geometry::Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-5.*5.*2.*(std::f64::consts::PI/5.).sin()*2.).abs()<1e-6);
}

#[test]
fn tube_right_click_starts_two_point_variant() {
    let (mut app,context,_)=startup();let solid=app.main_category_rect(2).expect("Solid category").center();
    click(&mut app,&context,solid);frame(&mut app,&context,vec![]);context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);let icon=app.rail_icon_rects[2+9];secondary_click(&mut app,&context,icon.center());
    assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-017"));assert_eq!(app.controller.prompt(),"First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(),0);
}

#[test]
fn cylinder_right_click_starts_two_point_variant() {
    let (mut app,context,_)=startup();let solid=app.main_category_rect(2).expect("Solid category").center();
    click(&mut app,&context,solid);frame(&mut app,&context,vec![]);context.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Solid")),1usize));frame(&mut app,&context,vec![]);let icon=app.rail_icon_rects[2+8];secondary_click(&mut app,&context,icon.center());
    assert_eq!(app.controller.active_feature(),Some("OM9-SOLID-016"));assert_eq!(app.controller.prompt(),"First diameter endpoint");
}

#[test]
fn pipe_right_click_starts_chain_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    let icon = app.rail_icon_rects[2 + 12];
    secondary_click(&mut app, &context, icon.center());
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-011"));
    assert!(app.controller.prompt().contains("connected curve segments"));
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn extract_surface_right_click_starts_copy_variant() {
    use om9_app::command_options::CommandOptionInterface;
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    assert_eq!(app.main_menu_category(), 2);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    let icon = app.rail_icon_rects[2 + 7];
    secondary_click(&mut app, &context, icon.center());
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-006"));
    let copy = app
        .controller
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "Copy")
        .unwrap();
    assert_eq!(copy.label, "Copy=Yes");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn extract_surface_right_click_with_preselection() {
    use om9_app::command_options::CommandOptionInterface;
    let (mut app, context, _) = startup();
    app.controller.submit("Box 10 10 10").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().entities[&id].clone();
    let history_count = app.controller.recent().len();
    let undo_before = app.controller.coordinator().undo_len();
    frame(&mut app, &context, vec![]);
    let solid = app.main_category_rect(2).unwrap().center();
    click(&mut app, &context, solid);
    let icon = app.rail_icon_rects[history_count + 2 + 7];
    secondary_click(&mut app, &context, icon.center());
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-006"));
    let copy = app
        .controller
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "Copy")
        .unwrap();
    assert_eq!(copy.label, "Copy=Yes");
    app.controller.submit("0").unwrap();
    assert_eq!(app.controller.document().entities[&id], before);
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
}

#[test]
fn top_menu_labels_are_interactive_and_return_to_categories() {
    let (mut app, context, _) = startup();
    let file = app.top_menu_rect(0).expect("File top menu").center();
    click(&mut app, &context, file);
    assert_eq!(app.top_menu_selected(), Some(0));
    let curve = app.main_category_rect(0).expect("Curve category").center();
    click(&mut app, &context, curve);
    assert_eq!(app.top_menu_selected(), None);
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn main_menu_categories_wrap_at_the_rail_edge() {
    let (app, _context, _) = startup();
    let sidebar = app.sidebar_rect.expect("sidebar rect");
    let rects: Vec<Rect> = (0..12)
        .map(|i| app.main_category_rect(i).expect("all twelve categories"))
        .collect();
    assert_eq!(rects.len(), 12);
    for rect in &rects {
        assert!(
            rect.max.x <= sidebar.max.x + 0.5,
            "category {rect:?} overflowed the rail edge {sidebar:?}"
        );
    }
    let rows = rects
        .iter()
        .filter(|rect| (rect.center().y - rects[0].center().y).abs() <= 0.5)
        .count();
    assert!(rows >= 2, "only one category fits the first row");
    let distinct_rows = rects
        .iter()
        .map(|rect| rect.center().y.round() as i32)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        distinct_rows.len() >= 2,
        "categories did not wrap at the rail edge: {distinct_rows:?}"
    );
}

#[test]
fn main_menu_category_tabs_switch_palette_without_document_mutation() {
    let (mut app, context, _) = startup();
    let surface = app
        .main_category_rect(1)
        .expect("Surface category")
        .center();
    click(&mut app, &context, surface);
    assert_eq!(app.main_menu_category(), 1);
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn unregistered_right_click_slot_never_executes_a_previous_action() {
    let (mut app, context, _) = startup();
    let icon = *app.rail_icon_rects.first().expect("sidebar icon");
    let before = app.controller.coordinator().snapshot();
    secondary_click(&mut app, &context, icon.center());
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn f4_toggles_world_datum_without_mutating_document() {
    let (mut app, context, output) = startup();
    assert!(app.world_datum_visible());
    assert!(datum_strokes(&output) > 0, "F4 datum must paint");
    let before = app.controller.coordinator().snapshot();
    let hidden = frame(&mut app, &context, vec![key(Key::F4)]);
    assert!(!app.world_datum_visible());
    assert_eq!(datum_strokes(&hidden), 0);
    let shown = frame(&mut app, &context, vec![key(Key::F4)]);
    assert!(app.world_datum_visible());
    assert!(datum_strokes(&shown) > 0);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.view_rects.contains_key(&ViewId::LookingDown));
    assert!(app.view_rects.contains_key(&ViewId::Perspective));
    assert!(app.view_rects.contains_key(&ViewId::SideView));
    assert!(app.view_rects.contains_key(&ViewId::ThroughFinger));
}

#[test]
fn view_tab_restores_cmd_focus_and_preserves_numeric_draft() {
    let (mut app, context, _) = startup();
    begin_numeric_draft(&mut app, &context);
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let point = text_positions(&output, ViewId::SideView.title())
        .into_iter()
        .find(|p| toolbar.contains(*p))
        .unwrap();
    click(&mut app, &context, point);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn view_tab_switches_active_and_maximized_view() {
    let (mut app, context, _) = startup();
    app.controller.workspace.maximized = Some(ViewId::LookingDown);
    app.controller.workspace.active = ViewId::LookingDown;
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let point = text_positions(&output, ViewId::SideView.title())
        .into_iter()
        .find(|p| toolbar.contains(*p))
        .unwrap();
    click(&mut app, &context, point);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    assert_eq!(app.controller.workspace.maximized, Some(ViewId::SideView));
    assert_eq!(
        app.controller.workspace.visible_views(),
        vec![ViewId::SideView]
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn view_menu_restores_cmd_focus_and_preserves_numeric_draft() {
    let (mut app, context, _) = startup();
    begin_numeric_draft(&mut app, &context);
    let output = frame(&mut app, &context, vec![]);
    let menu = text_positions(&output, "View")
        .into_iter()
        .find(|p| p.y < 25.)
        .unwrap();
    click(&mut app, &context, menu);
    let opened = frame(&mut app, &context, vec![]);
    let toolbar_top = app.toolbar_rect.unwrap().top();
    let selection = text_positions(&opened, ViewId::SideView.title())
        .into_iter()
        .find(|p| p.y < toolbar_top)
        .unwrap();
    click(&mut app, &context, selection);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn display_toolbar_tab_does_not_reuse_enabled_geometry_palette() {
    let (mut app, context, output) = startup();
    let display = text_positions(&output, "Display")[0];
    click(&mut app, &context, display);
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let images: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Mesh(mesh) = &s.shape {
                let center = mesh.calc_bounds().center();
                (mesh.texture_id != egui::TextureId::default()
                    && toolbar.contains(center)
                    && s.clip_rect.contains(center))
                .then_some(center)
            } else {
                None
            }
        })
        .collect();
    assert!(!images.is_empty());
    for position in images {
        click(&mut app, &context, position);
        assert!(
            !app.controller.is_running(),
            "Display toolbar invoked geometry"
        );
        assert!(app.controller.document().entities.is_empty());
    }
}
