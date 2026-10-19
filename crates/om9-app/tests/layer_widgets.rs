// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::{LayerControl, LayerDialogControl, MatrixApp};
use om9_core::{
    geometry::Geometry,
    layers::PresetSlot,
    model::{Document, Entity, Layer},
};
fn frame(app: &mut MatrixApp, ctx: &egui::Context, mut events: Vec<Event>) -> egui::FullOutput {
    let modifiers = events
        .iter()
        .find_map(|event| match event {
            Event::PointerButton { modifiers, .. } | Event::Key { modifiers, .. } => {
                Some(*modifiers)
            }
            _ => None,
        })
        .unwrap_or(Modifiers::NONE);
    events.insert(0, Event::ModifiersChanged(modifiers));
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 1100.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    out.textures_delta.clear();
    out
}
fn start(d: Document) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(d).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: Pos2, button: PointerButton) {
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
}
fn control(
    app: &mut MatrixApp,
    ctx: &egui::Context,
    slot: u8,
    control: LayerControl,
    button: PointerButton,
) {
    let pos = app.layer_control_rect(slot, control).unwrap().center();
    click(app, ctx, pos, button);
}
fn action(app: &mut MatrixApp, ctx: &egui::Context, name: &str) {
    let pos = app.layer_action_rect(name).unwrap().center();
    click(app, ctx, pos, PointerButton::Primary);
}
fn shift_right(app: &mut MatrixApp, ctx: &egui::Context, slot: u8, control: LayerControl) {
    let pos = app.layer_control_rect(slot, control).unwrap().center();
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Secondary,
                    pressed,
                    modifiers: Modifiers::SHIFT,
                },
            ],
        );
    }
    for _ in 0..3 {
        frame(app, ctx, vec![]);
    }
}
fn modal_click(app: &mut MatrixApp, ctx: &egui::Context, control: LayerDialogControl) {
    let pos = app.layer_dialog_control_rect(control).unwrap().center();
    click(app, ctx, pos, PointerButton::Primary);
    for _ in 0..3 {
        frame(app, ctx, vec![]);
    }
}
fn key(key: egui::Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
#[test]
fn rename_modal_escape_preserves_pending_line_and_numeric_draft() {
    let (mut app, ctx) = start(Document::new("modal input owner"));
    action(&mut app, &ctx, "Advanced");
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("1,2,");
    let before = app.controller.coordinator().snapshot();
    let views = app.controller.workspace.views.clone();
    shift_right(&mut app, &ctx, 17, LayerControl::Name);
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Escape, Modifiers::NONE)],
    );
    assert!(app.controller.layer_dialog().is_none());
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "1,2,");
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.workspace.views, views);
    assert_eq!(
        app.controller.preview([3., 4., 5.]).unwrap(),
        Some(([0.; 3], [3., 4., 5.]))
    );
}
#[test]
fn rename_modal_text_and_ok_publish_only_label_and_rebind_pending_line() {
    let (mut app, ctx) = start(Document::new("real modal text"));
    action(&mut app, &ctx, "Advanced");
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("4,5,");
    shift_right(&mut app, &ctx, 21, LayerControl::Name);
    frame(
        &mut app,
        &ctx,
        vec![key(
            egui::Key::A,
            Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            },
        )],
    );
    frame(&mut app, &ctx, vec![Event::Text("Mẫu tím".into())]);
    assert_eq!(
        app.controller.layer_dialog().unwrap().name(),
        Some("Mẫu tím")
    );
    modal_click(&mut app, &ctx, LayerDialogControl::Ok);
    let id = app
        .controller
        .layer_id(PresetSlot::new(21).unwrap())
        .unwrap();
    assert_eq!(app.controller.document().layers[&id].name, "Mẫu tím");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert!(app.controller.document().current_layer.is_none());
    assert_eq!(app.transcript.draft(), "4,5,");
    assert!(app.controller.is_running());
    app.controller.submit("1,2,3").unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}
#[test]
fn color_modal_keyboard_edit_ok_and_cancel_do_not_change_current_or_membership() {
    let mut d = Document::new("real color controls");
    for n in 17..=20 {
        let mut l = Layer::preset(PresetSlot::new(n).unwrap());
        if n == 17 {
            l.color = [10, 20, 30, 90];
            d.current_layer = Some(l.id);
        }
        d.layer_order.push(l.id);
        d.layers.insert(l.id, l);
    }
    let mut e = Entity::new("member", Geometry::Point([1., 2., 3.]), d.current_layer);
    e.attributes.color = None;
    let eid = e.id;
    d.entities.insert(eid, e);
    let (mut app, ctx) = start(d);
    action(&mut app, &ctx, "Advanced");
    shift_right(&mut app, &ctx, 17, LayerControl::Swatch);
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::ArrowUp, Modifiers::NONE)],
    );
    assert_eq!(
        app.controller.layer_dialog().unwrap().color(),
        Some([11, 20, 30, 90])
    );
    modal_click(&mut app, &ctx, LayerDialogControl::Ok);
    let lid = app.controller.document().current_layer.unwrap();
    assert_eq!(
        app.controller.document().layers[&lid].color,
        [11, 20, 30, 90]
    );
    assert_eq!(
        app.controller.document().entities[&eid].attributes.layer,
        Some(lid)
    );
    assert_eq!(
        app.controller.scene().unwrap().objects[0].color,
        [11, 20, 30, 90]
    );
    let before = app.controller.coordinator().snapshot();
    shift_right(&mut app, &ctx, 17, LayerControl::Swatch);
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::ArrowUp, Modifiers::NONE)],
    );
    modal_click(&mut app, &ctx, LayerDialogControl::Cancel);
    assert_eq!(app.controller.coordinator().snapshot(), before);
}
#[test]
fn selective_show_checkbox_ids_distinguish_duplicate_labels_and_cancel_keeps_hidden() {
    let mut d = Document::new("duplicate hidden labels");
    let mut ids = Vec::new();
    for i in 0..2 {
        let mut e = Entity::new("same label", Geometry::Point([i as f64, 0., 0.]), None);
        e.attributes.hidden = true;
        ids.push(e.id);
        d.entities.insert(e.id, e);
    }
    let (mut app, ctx) = start(d);
    let pos = app.layer_action_rect("Show").unwrap().center();
    click(&mut app, &ctx, pos, PointerButton::Secondary);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    modal_click(&mut app, &ctx, LayerDialogControl::Hidden(ids[0]));
    modal_click(&mut app, &ctx, LayerDialogControl::Ok);
    assert!(
        !app.controller.document().entities[&ids[0]]
            .attributes
            .hidden
    );
    assert!(
        app.controller.document().entities[&ids[1]]
            .attributes
            .hidden
    );
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    let before = app.controller.coordinator().snapshot();
    click(&mut app, &ctx, pos, PointerButton::Secondary);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    modal_click(&mut app, &ctx, LayerDialogControl::Hidden(ids[1]));
    modal_click(&mut app, &ctx, LayerDialogControl::Cancel);
    assert_eq!(app.controller.coordinator().snapshot(), before);
}
#[test]
fn stale_modal_dismissal_and_old_ok_click_cannot_publish_pending_line() {
    use om9_app::layer_command::LayerProperty;
    let (mut app, ctx) = start(Document::new("stale dialog dismissal"));
    action(&mut app, &ctx, "Advanced");
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("8,9,");
    shift_right(&mut app, &ctx, 17, LayerControl::Name);
    let old_ok = app
        .layer_dialog_control_rect(LayerDialogControl::Ok)
        .unwrap()
        .center();
    app.controller
        .set_layer_property(PresetSlot::new(1).unwrap(), LayerProperty::Visible(false))
        .unwrap();
    let before = app.controller.coordinator().snapshot();
    click(&mut app, &ctx, old_ok, PointerButton::Primary);
    assert!(app.controller.layer_dialog().is_none());
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "8,9,");
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert!(app.controller.document().entities.is_empty());
}
#[test]
fn hidden_modal_virtualizes_large_snapshot_and_keeps_every_owned_choice() {
    let mut d = Document::new("virtualized choices");
    for i in 0..2000 {
        let mut e = Entity::new(format!("hidden-row-{i}"), Geometry::Point([0.; 3]), None);
        e.attributes.hidden = true;
        d.entities.insert(e.id, e);
    }
    let (mut app, ctx) = start(d);
    let pos = app.layer_action_rect("Show").unwrap().center();
    click(&mut app, &ctx, pos, PointerButton::Secondary);
    let output = frame(&mut app, &ctx, vec![]);
    assert_eq!(app.controller.layer_dialog().unwrap().entries().len(), 2000);
    let rendered = output
        .shapes
        .iter()
        .filter(|shape| {
            matches!(&shape.shape,egui::Shape::Text(text)
        if text.galley.job.text.starts_with("hidden-row-"))
        })
        .count();
    assert!(
        rendered > 0 && rendered < 40,
        "rendered{rendered} of2000 rows"
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn modal_ime_commit_enter_does_not_publish_or_cancel_pending_command() {
    let (mut app, ctx) = start(Document::new("modal IME lifecycle"));
    action(&mut app, &ctx, "Advanced");
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.transcript.set_draft("2,3,");
    shift_right(&mut app, &ctx, 17, LayerControl::Name);
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "Quý".into(),
                active_range_chars: None,
            }),
            key(egui::Key::Escape, Modifiers::NONE),
        ],
    );
    assert!(app.controller.layer_dialog().is_some());
    assert!(app.controller.is_running());
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Commit("Quý".into())),
            key(egui::Key::Enter, Modifiers::NONE),
        ],
    );
    assert!(app.controller.layer_dialog().is_some());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert_eq!(app.transcript.draft(), "2,3,");
    frame(
        &mut app,
        &ctx,
        vec![key(egui::Key::Escape, Modifiers::NONE)],
    );
    assert!(app.controller.layer_dialog().is_none());
    assert!(app.controller.is_running());
}
#[test]
fn invalid_rgb_text_cannot_silently_confirm_an_unchanged_color() {
    let (mut app, ctx) = start(Document::new("invalid RGB editor"));
    action(&mut app, &ctx, "Advanced");
    shift_right(&mut app, &ctx, 17, LayerControl::Swatch);
    frame(
        &mut app,
        &ctx,
        vec![key(
            egui::Key::A,
            Modifiers {
                ctrl: true,
                command: true,
                ..Modifiers::NONE
            },
        )],
    );
    frame(&mut app, &ctx, vec![Event::Paste("abc".into())]);
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    modal_click(&mut app, &ctx, LayerDialogControl::Ok);
    assert!(
        app.controller.layer_dialog().is_some(),
        "invalid channel must stay editable"
    );
    assert!(app.controller.document().layers.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn closing_rename_removes_its_editor_state_without_removing_cmd_state() {
    let (mut app, ctx) = start(Document::new("editor lifecycle"));
    action(&mut app, &ctx, "Advanced");
    shift_right(&mut app, &ctx, 17, LayerControl::Name);
    let id = ctx.memory(|m| m.focused()).unwrap();
    frame(&mut app, &ctx, vec![Event::Text(" custom".into())]);
    assert!(egui::TextEdit::load_state(&ctx, id).is_some());
    modal_click(&mut app, &ctx, LayerDialogControl::Cancel);
    assert!(
        egui::TextEdit::load_state(&ctx, id).is_none(),
        "closed modal retains input/Undo state"
    );
    assert!(egui::TextEdit::load_state(&ctx, egui::Id::new("matrix-command-transcript")).is_some());
}
#[test]
fn hidden_choice_long_and_multiline_labels_keep_fixed_clickable_rows() {
    use om9_app::layer_dialog::LayerDialogKind;
    for label in ["long label ".repeat(700), "x\n".repeat(1000)] {
        let mut d = Document::new("bounded hidden row");
        let mut e = Entity::new(&label, Geometry::Point([0.; 3]), None);
        e.attributes.hidden = true;
        let id = e.id;
        d.entities.insert(id, e);
        let (mut app, ctx) = start(d);
        app.controller
            .begin_layer_dialog(LayerDialogKind::Show, None)
            .unwrap();
        for _ in 0..10 {
            frame(&mut app, &ctx, vec![]);
        }
        let row = app
            .layer_dialog_control_rect(LayerDialogControl::Hidden(id))
            .unwrap();
        assert!(
            row.height() <= 18.01,
            "virtualized row grew to {}",
            row.height()
        );
        assert_eq!(
            app.controller.layer_dialog().unwrap().entries()[0].label(),
            label
        );
        modal_click(&mut app, &ctx, LayerDialogControl::Hidden(id));
        assert!(app.controller.layer_dialog().unwrap().entries()[0].selected());
        modal_click(&mut app, &ctx, LayerDialogControl::Ok);
        assert!(!app.controller.document().entities[&id].attributes.hidden);
        assert_eq!(app.controller.document().entities[&id].label, label);
    }
}
#[test]
fn repeated_closed_dialogs_keep_modal_area_and_scroll_state_bounded() {
    use om9_app::layer_dialog::LayerDialogKind;
    let mut d = Document::new("bounded modal state");
    let mut e = Entity::new("hidden", Geometry::Point([0.; 3]), None);
    e.attributes.hidden = true;
    d.entities.insert(e.id, e);
    let (mut app, ctx) = start(d);
    let mut baseline = None;
    for _ in 0..30 {
        let token = app
            .controller
            .begin_layer_dialog(LayerDialogKind::Show, None)
            .unwrap();
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        app.controller.cancel_layer_dialog(token).unwrap();
        for _ in 0..2 {
            frame(&mut app, &ctx, vec![]);
        }
        let retained = (
            ctx.data_mut(|data| data.count::<egui::scroll_area::State>()),
            ctx.memory(|m| format!("{:?}", m.areas()).matches("AreaState {").count()),
        );
        if let Some(first) = baseline {
            assert_eq!(retained, first, "closed dialog retained a new owner");
        } else {
            baseline = Some(retained);
        }
    }
    assert!(app.controller.document().layers.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn release_of_old_dialog_ok_cannot_confirm_a_new_token() {
    use om9_app::layer_dialog::LayerDialogKind;
    let (mut app, ctx) = start(Document::new("gesture token owner"));
    let old = app
        .controller
        .begin_layer_dialog(LayerDialogKind::Rename, Some(PresetSlot::new(17).unwrap()))
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pos = app
        .layer_dialog_control_rect(LayerDialogControl::Ok)
        .unwrap()
        .center();
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    app.controller.cancel_layer_dialog(old).unwrap();
    let new = app
        .controller
        .begin_layer_dialog(LayerDialogKind::Rename, Some(PresetSlot::new(17).unwrap()))
        .unwrap();
    app.controller
        .set_layer_dialog_name(new, "second pending")
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    assert_eq!(
        app.controller.layer_dialog().map(|dialog| dialog.token()),
        Some(new)
    );
    assert!(app.controller.document().layers.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn default_rows_are_lazy_advanced_collapsed_and_fit_sidebar() {
    let (app, _) = start(Document::new("real rows"));
    assert!(app.controller.document().layers.is_empty());
    assert!(app.controller.document().current_layer.is_none());
    for slot in (1..=16).chain([33]) {
        for control in [
            LayerControl::Name,
            LayerControl::Arrow,
            LayerControl::Swatch,
            LayerControl::Lock,
            LayerControl::Visibility,
        ] {
            let rect = app.layer_control_rect(slot, control).unwrap();
            assert!(app.sidebar_rect.unwrap().contains_rect(rect));
        }
    }
    assert!(app.layer_control_rect(17, LayerControl::Name).is_none());
}
#[test]
fn shift_right_advanced_name_opens_rename_modal_without_materializing_layers() {
    let (mut app, ctx) = start(Document::new("Advanced rename modal"));
    action(&mut app, &ctx, "Advanced");
    let pos = app
        .layer_control_rect(17, LayerControl::Name)
        .unwrap()
        .center();
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Secondary,
                    pressed,
                    modifiers: Modifiers::SHIFT,
                },
            ],
        );
    }
    let output = frame(&mut app, &ctx, vec![]);
    assert!(
        output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text)
            if text.galley.job.text == "Rename Advanced layer")
        }),
        "Shift+right-click must render the owned rename dialog"
    );
    assert!(app.controller.document().layers.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn command_acceptance_keeps_layer_hit_regions_at_the_rendered_locations() {
    let (mut app, ctx) = start(Document::new("stable layer geometry"));
    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0").unwrap();
    frame(&mut app, &ctx, vec![]);
    let before = app.layer_control_rect(1, LayerControl::Swatch).unwrap();
    app.controller.submit("1,0").unwrap();
    frame(&mut app, &ctx, vec![]);
    assert_eq!(
        app.layer_control_rect(1, LayerControl::Swatch).unwrap(),
        before
    );
}
fn membership_fixture() -> (Document, om9_core::identity::ObjectId) {
    let mut d = Document::new("widget membership");
    let layer = Layer::preset(PresetSlot::new(5).unwrap());
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let e = Entity::new("visible", Geometry::Point([0.; 3]), Some(lid));
    let id = e.id;
    d.entities.insert(id, e);
    for (name, visible, selectable) in [
        ("explicit invisible", false, true),
        ("explicit unselectable", true, false),
    ] {
        let mut e = Entity::new(name, Geometry::Point([2.; 3]), Some(lid));
        e.attributes.visible = visible;
        e.attributes.selectable = selectable;
        d.entities.insert(e.id, e);
    }
    (d, id)
}
#[test]
fn real_name_swatch_and_arrow_keep_current_separate_from_assignment_and_filter_selection() {
    let (d, eid) = membership_fixture();
    let (mut app, ctx) = start(d);
    control(
        &mut app,
        &ctx,
        1,
        LayerControl::Name,
        PointerButton::Primary,
    );
    let current = app.controller.document().current_layer;
    control(
        &mut app,
        &ctx,
        5,
        LayerControl::Swatch,
        PointerButton::Secondary,
    );
    assert_eq!(
        app.selected_objects().iter().copied().collect::<Vec<_>>(),
        vec![eid]
    );
    control(
        &mut app,
        &ctx,
        9,
        LayerControl::Arrow,
        PointerButton::Primary,
    );
    assert_eq!(app.controller.document().current_layer, current);
    let target = app
        .controller
        .layer_id(PresetSlot::new(9).unwrap())
        .unwrap();
    assert_eq!(
        app.controller.document().entities[&eid].attributes.layer,
        Some(target)
    );
    app.controller.undo().unwrap();
    assert!(
        app.controller
            .layer_id(PresetSlot::new(9).unwrap())
            .is_none()
    );
    assert_eq!(app.controller.document().current_layer, current);
}
#[test]
fn actual_hide_show_and_lock_visibility_preserve_explicit_preferences() {
    let (d, eid) = membership_fixture();
    let (mut app, ctx) = start(d);
    control(
        &mut app,
        &ctx,
        5,
        LayerControl::Swatch,
        PointerButton::Secondary,
    );
    action(&mut app, &ctx, "Hide");
    assert!(app.controller.document().entities[&eid].attributes.hidden);
    assert!(app.selected_objects().is_empty());
    action(&mut app, &ctx, "Show");
    assert!(!app.controller.document().entities[&eid].attributes.hidden);
    control(
        &mut app,
        &ctx,
        5,
        LayerControl::Lock,
        PointerButton::Primary,
    );
    assert!(
        app.controller.document().layers[&app
            .controller
            .layer_id(PresetSlot::new(5).unwrap())
            .unwrap()]
            .locked
    );
    control(
        &mut app,
        &ctx,
        5,
        LayerControl::Swatch,
        PointerButton::Secondary,
    );
    assert!(app.selected_objects().is_empty());
    control(
        &mut app,
        &ctx,
        5,
        LayerControl::Lock,
        PointerButton::Primary,
    );
    for _ in 0..2 {
        control(
            &mut app,
            &ctx,
            5,
            LayerControl::Visibility,
            PointerButton::Primary,
        );
    }
    for e in app.controller.document().entities.values() {
        if e.label == "explicit invisible" {
            assert!(!e.attributes.visible);
        }
        if e.label == "explicit unselectable" {
            assert!(!e.attributes.selectable);
        }
    }
}
#[test]
fn advanced_expand_collapse_and_group_bar_use_stable_nonsequential_label_order() {
    let (mut app, ctx) = start(Document::new("Advanced widgets"));
    let before = app.controller.coordinator().snapshot();
    action(&mut app, &ctx, "Advanced");
    assert_eq!(app.controller.coordinator().snapshot(), before);
    let labels: Vec<_> = frame(&mut app, &ctx, vec![])
        .shapes
        .into_iter()
        .filter_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                Some(t.galley.job.text.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(labels.iter().any(|s| s == "User 25"));
    assert!(labels.iter().any(|s| s == "User 21"));
    control(
        &mut app,
        &ctx,
        21,
        LayerControl::Name,
        PointerButton::Primary,
    );
    let current = app.controller.document().current_layer;
    assert_eq!(
        app.controller.document().layers[&current.unwrap()].name,
        "User 25"
    );
    action(&mut app, &ctx, "Advanced");
    assert!(app.layer_control_rect(21, LayerControl::Name).is_none());
    assert_eq!(app.controller.document().current_layer, current);
    control(
        &mut app,
        &ctx,
        1,
        LayerControl::Group,
        PointerButton::Primary,
    );
    for number in 1..=4 {
        let id = app
            .controller
            .layer_id(PresetSlot::new(number).unwrap())
            .unwrap();
        assert!(!app.controller.document().layers[&id].visible);
    }
}
#[test]
fn current_widget_rebinds_pending_line_without_erasing_numeric_draft() {
    let (mut app, ctx) = start(Document::new("pending widget"));
    app.controller.submit("Line").unwrap();
    app.controller.submit("1,2,3").unwrap();
    app.transcript.set_draft("4,5,6");
    let views = app.controller.workspace.views.clone();
    control(
        &mut app,
        &ctx,
        16,
        LayerControl::Name,
        PointerButton::Primary,
    );
    assert_eq!(app.transcript.draft(), "4,5,6");
    assert!(app.controller.is_running());
    assert_eq!(app.controller.workspace.views, views);
    app.controller.submit("4,5,6").unwrap();
    assert_eq!(
        app.controller
            .document()
            .entities
            .values()
            .next()
            .unwrap()
            .attributes
            .layer,
        app.controller.document().current_layer
    );
}
