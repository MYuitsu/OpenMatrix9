// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in own-app modal replay through normal egui input and real widgets.
use super::*;
use crate::layer_dialog::LayerDialogKind;
use om9_core::{geometry::Geometry, layers::PresetSlot};
impl MatrixApp {
    pub(super) fn layer_dialog_audit_input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
    ) {
        if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.awaiting_screenshot || audit.receipt_written)
        {
            ctx.request_repaint();
            return;
        }
        let step = self.audit.as_ref().expect("audit").step;
        if step == 0 && !raw.focused {
            let audit = self.audit.as_mut().expect("audit");
            if audit.native_focus_wait_started.is_none() {
                audit.native_focus_wait_started = Some(std::time::Instant::now());
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            if audit.native_focus_wait_started.expect("wait").elapsed()
                < std::time::Duration::from_secs(3)
            {
                ctx.request_repaint_after(std::time::Duration::from_millis(25));
                return;
            }
        }
        let expected = match step {
            9..=12 | 18..=23 | 79..=88 => Some(LayerDialogKind::Rename),
            27..=31 | 35..=39 => Some(LayerDialogKind::Color),
            55..=60 | 64..=69 => Some(LayerDialogKind::Show),
            _ => None,
        };
        let issue = if step == 7 && !self.advanced_layers {
            Some("Advanced click was not accepted")
        } else if !raw.focused {
            Some("native window lost focus")
        } else if expected.is_some_and(|kind| {
            self.controller
                .layer_dialog()
                .is_none_or(|dialog| dialog.kind() != kind)
        }) {
            Some("expected layer modal is absent")
        } else if matches!(step, 11 | 12 | 20 | 21 | 81 | 82 | 29 | 37)
            && self.controller.layer_dialog().is_some_and(|dialog| {
                let token = dialog.token();
                let id = if dialog.kind() == LayerDialogKind::Rename {
                    egui::Id::new(("owned-layer-name", token.authority(), token.serial()))
                } else {
                    egui::Id::new(("owned-layer-rgb", token.authority(), token.serial(), 0usize))
                };
                !ctx.memory(|memory| memory.has_focus(id))
            })
        {
            Some("expected modal editor does not own focus")
        } else {
            None
        };
        if let Some(issue) = issue {
            let audit = self.audit.as_mut().expect("audit");
            audit.checks.push(format!("FAILED step {step}: {issue}"));
            audit.viewport_replay.push(serde_json::json!({"step":step,"failed":issue,"raw_time":raw.time,"raw_focused":raw.focused,"passes":self.passes,
                "advanced":self.advanced_layers,"advanced_rect":self.layer_action_rects.get("Advanced").map(|r|[r.min.x,r.min.y,r.max.x,r.max.y]),
                "modal":self.controller.layer_dialog().map(|dialog|format!("{:?}",dialog.kind())),"focused":format!("{:?}",ctx.memory(|m|m.focused()))}));
            let _ = std::fs::write(
                audit.directory.join("dialog-replay.json"),
                serde_json::to_vec(&audit.viewport_replay).expect("failure trace"),
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            audit.awaiting_screenshot = true;
            ctx.request_repaint();
            return;
        }
        if matches!(step, 10 | 28 | 56) {
            let audit = self.audit.as_mut().expect("audit");
            if audit.dialog_capture_step == Some(step) {
                if audit.dialog_capture_pending.is_some() {
                    ctx.request_repaint();
                    return;
                }
            } else {
                let started = audit
                    .dialog_capture_ready_at
                    .get_or_insert_with(std::time::Instant::now);
                if started.elapsed() < std::time::Duration::from_millis(300) {
                    ctx.request_repaint();
                    return;
                }
                let tag = match step {
                    10 => "rename",
                    28 => "color",
                    56 => "selective-show",
                    _ => unreachable!(),
                };
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(tag)));
                audit.dialog_capture_ready_at = None;
                audit.dialog_capture_pending = Some(tag);
                audit.dialog_capture_step = Some(step);
                ctx.request_repaint();
                return;
            }
        }
        self.audit.as_mut().expect("audit").step += 1;
        if step <= 92 {
            let audit = self.audit.as_mut().expect("audit");
            audit.viewport_replay.push(serde_json::json!({"step":step,
                "raw_time":raw.time,"raw_focused":raw.focused,"passes":self.passes,
                "native_events":raw.events.iter().map(|event|match event {
                    egui::Event::PointerMoved(p)=>serde_json::json!({"pointer_moved":[p.x,p.y]}),
                    egui::Event::PointerButton {pos,button,pressed,modifiers}=>serde_json::json!({"button":format!("{button:?}"),"position":[pos.x,pos.y],"pressed":pressed,"modifiers":format!("{modifiers:?}")}),
                    egui::Event::Key {key,pressed,repeat,..}=>serde_json::json!({"key":format!("{key:?}"),"pressed":pressed,"repeat":repeat}),
                    egui::Event::WindowFocused(focused)=>serde_json::json!({"window_focused":focused}),
                    egui::Event::Text(text)|egui::Event::Paste(text)=>serde_json::json!({"text_bytes":text.len()}),
                    _=>serde_json::json!({"event_kind":format!("{:?}",std::mem::discriminant(event))})
                }).collect::<Vec<_>>(),
                "advanced":self.advanced_layers,"advanced_rect":self.layer_action_rects.get("Advanced").map(|r|[r.min.x,r.min.y,r.max.x,r.max.y]),
                "modal":self.controller.layer_dialog().map(|dialog|format!("{:?}",dialog.kind())),
                "name":self.controller.layer_dialog().and_then(|dialog|dialog.name()),
                "draft":self.transcript.draft(),"running":self.controller.is_running(),
                "undo":self.controller.coordinator().undo_len(),
                "ok":self.layer_dialog_rects.get(&LayerDialogControl::Ok).map(|rect|[rect.min.x,rect.min.y,rect.max.x,rect.max.y]),
                "cancel":self.layer_dialog_rects.get(&LayerDialogControl::Cancel).map(|rect|[rect.min.x,rect.min.y,rect.max.x,rect.max.y]),
                "focused":format!("{:?}",ctx.memory(|memory|memory.focused())),
                "checks":audit.checks.len()}));
            let _ = std::fs::write(
                audit.directory.join("dialog-replay.json"),
                serde_json::to_vec(&audit.viewport_replay).expect("replay"),
            );
        }
        raw.events
            .push(egui::Event::ModifiersChanged(Modifiers::NONE));
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let mut pointer = |pos: Option<egui::Pos2>, button, pressed, modifiers| {
            raw.events.push(egui::Event::ModifiersChanged(modifiers));
            if let Some(pos) = pos {
                raw.events.push(egui::Event::PointerMoved(pos));
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                });
            }
        };
        let row = |control| self.layer_control_rect(21, control).map(|r| r.center());
        let modal = |control| self.layer_dialog_control_rect(control).map(|r| r.center());
        let object = self.controller.document().entities.keys().next().copied();
        match step {
            0 => raw.events.push(egui::Event::Text("Line".into())),
            1 | 3 | 45 | 72 | 75 => raw.events.push(key(Key::Enter, Modifiers::NONE)),
            2 => raw.events.push(egui::Event::Text("0,0,0".into())),
            4 => raw.events.push(egui::Event::Text("12,6,".into())),
            5 | 6 => pointer(
                self.layer_action_rect("Advanced").map(|r| r.center()),
                egui::PointerButton::Primary,
                step == 5,
                Modifiers::NONE,
            ),
            7 | 8 | 16 | 17 | 77 | 78 => pointer(
                row(LayerControl::Name),
                egui::PointerButton::Secondary,
                matches!(step, 7 | 16 | 77),
                Modifiers::SHIFT,
            ),
            25 | 26 | 33 | 34 => pointer(
                row(LayerControl::Swatch),
                egui::PointerButton::Secondary,
                matches!(step, 25 | 33),
                Modifiers::SHIFT,
            ),
            11 | 20 | 81 => {
                let m = Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                };
                raw.events.push(egui::Event::ModifiersChanged(m));
                raw.events.push(key(Key::A, m));
            }
            12 => raw
                .events
                .push(egui::Event::Text("Native canceled label".into())),
            21 => raw
                .events
                .push(egui::Event::Text("Native saved label".into())),
            82 => raw.events.push(egui::Event::Text("Metal 01".into())),
            13 | 14 | 30 | 31 | 59 | 60 | 87 | 88 => pointer(
                modal(LayerDialogControl::Cancel),
                egui::PointerButton::Primary,
                matches!(step, 13 | 30 | 59 | 87),
                Modifiers::NONE,
            ),
            22 | 23 | 38 | 39 | 68 | 69 | 83 | 84 => pointer(
                modal(LayerDialogControl::Ok),
                egui::PointerButton::Primary,
                matches!(step, 22 | 38 | 68 | 83),
                Modifiers::NONE,
            ),
            29 | 37 => raw.events.push(key(Key::ArrowUp, Modifiers::NONE)),
            41 | 42 => pointer(
                row(LayerControl::Name),
                egui::PointerButton::Primary,
                step == 41,
                Modifiers::NONE,
            ),
            44 => raw.events.push(egui::Event::Text("0".into())),
            47 | 48 => pointer(
                row(LayerControl::Swatch),
                egui::PointerButton::Secondary,
                step == 47,
                Modifiers::NONE,
            ),
            50 | 51 => pointer(
                self.layer_action_rect("Hide").map(|r| r.center()),
                egui::PointerButton::Primary,
                step == 50,
                Modifiers::NONE,
            ),
            53 | 54 | 62 | 63 => pointer(
                self.layer_action_rect("Show").map(|r| r.center()),
                egui::PointerButton::Secondary,
                matches!(step, 53 | 62),
                Modifiers::NONE,
            ),
            57 | 58 | 66 | 67 => pointer(
                object.and_then(|id| modal(LayerDialogControl::Hidden(id))),
                egui::PointerButton::Primary,
                matches!(step, 57 | 66),
                Modifiers::NONE,
            ),
            71 => raw.events.push(egui::Event::Text("Undo".into())),
            74 => raw.events.push(egui::Event::Text("Redo".into())),
            92 => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                self.audit.as_mut().expect("audit").awaiting_screenshot = true;
            }
            _ => {}
        }
        let d = self.controller.document();
        let slot = PresetSlot::new(21).expect("slot");
        let target = self.controller.layer_id(slot);
        let mut color = slot.color();
        color[0] += 1;
        let preserved = self.controller.is_running()
            && self.transcript.draft() == "12,6,"
            && self
                .controller
                .preview([1., 2., 3.])
                .ok()
                .flatten()
                .is_some_and(|p| p.0 == [0.; 3]);
        let hidden = object.is_some_and(|id| d.entities[&id].attributes.hidden);
        let matches_layer = target.is_some_and(|id| {
            d.layers[&id].name == "Native saved label" && d.layers[&id].color == color
        });
        let undo = self.controller.coordinator().undo_len();
        let modal_closed = self.controller.layer_dialog().is_none();
        let check=match step {
            15 if preserved && modal_closed && d.layers.is_empty() && undo==0=>Some("native rename Cancel preserves pending Line and lazy model"),
            24 if preserved && modal_closed && undo==1 && d.layers.len()==4 && d.current_layer.is_none()
                && target.is_some_and(|id|d.layers[&id].name=="Native saved label")=>Some("native rename OK owns one lazy group and stable label"),
            32 if preserved && modal_closed && undo==1 && target==self.audit.as_ref().expect("audit").layer_current
                && target.is_some_and(|id|d.layers[&id].color==slot.color())=>Some("native color Cancel preserves layer and identity"),
            40 if preserved && modal_closed && matches_layer && undo==2 && d.current_layer.is_none()=>Some("native RGB keyboard edit and OK preserves identity and draft"),
            46 if matches_layer && d.entities.len()==1 && undo==4 && d.current_layer==target && object.is_some_and(|id| {
                let e=&d.entities[&id]; e.attributes.layer==target && e.attributes.color.is_none()
                    && matches!(&e.geometry,Geometry::Curve(om9_core::geometry::Curve::Line{start,end,..}) if *start==[0.;3] && *end==[12.,6.,0.])
            })=>Some("native Line after dialogs binds current and by-layer style"),
            61 if hidden && modal_closed && undo==5 && self.controller.selected().is_empty()=>Some("native selective Show Cancel keeps Hidden and canvas selection"),
            70 if !hidden && modal_closed && undo==6 && matches_layer && self.controller.selected().is_empty()=>Some("native selective Show checkbox OK restores only Hidden"),
            73 if hidden && undo==5 && matches_layer && d.current_layer==target=>Some("native CMD Undo restores selective Show flags"),
            76 if !hidden && undo==6 && matches_layer && d.current_layer==target=>Some("native CMD Redo restores flags and preset identity"),
            89 if modal_closed && undo==6 && matches_layer && d.current_layer==target
                && self.audit.as_ref().expect("audit").viewport_views.as_ref()==Some(&self.controller.workspace.views)=>Some("native reserved rename refuses and Cancel keeps accepted state"),
            _=>None
        };
        if step == 24 {
            self.audit.as_mut().expect("audit").layer_current = target;
        }
        if step == 0 {
            self.audit.as_mut().expect("audit").viewport_views =
                Some(self.controller.workspace.views.clone());
        }
        if step == 85
            && self.controller.layer_dialog().is_none_or(|dialog| {
                dialog.kind() != LayerDialogKind::Rename || dialog.name() != Some("Metal 01")
            })
        {
            self.audit
                .as_mut()
                .expect("audit")
                .checks
                .push("FAILED reserved rename modal vanished".into());
        }
        if let Some(check) = check {
            self.audit
                .as_mut()
                .expect("audit")
                .checks
                .push(check.into());
        }
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(3)
                .unwrap();
            let path = root
                .join("build/standalone-validation")
                .join(format!("replay-guard-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            for entry in std::fs::read_dir(&self.0).unwrap() {
                let path = entry.unwrap().path();
                assert_eq!(path.parent(), Some(self.0.as_path()));
                assert!(path.is_file());
                std::fs::remove_file(path).unwrap();
            }
            std::fs::remove_dir(&self.0).unwrap();
        }
    }
    fn frame(app: &mut MatrixApp, ctx: &egui::Context, time: f64) -> Vec<egui::Event> {
        let mut raw = egui::RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1600., 1000.),
            )),
            ..Default::default()
        };
        <MatrixApp as eframe::App>::raw_input_hook(app, ctx, &mut raw);
        let events = raw.events.clone();
        let mut output = ctx.run_ui(raw, |ui| app.show(ui));
        output.textures_delta.clear();
        events
    }
    fn start(fixture: &Fixture) -> (MatrixApp, egui::Context) {
        let mut app = MatrixApp::new(Document::new("Replay ownership")).unwrap();
        let ctx = egui::Context::default();
        for i in 0..3 {
            frame(&mut app, &ctx, i as f64 * 0.01);
        }
        let mut audit = UiAudit::new(fixture.0.clone());
        audit.layer_dialogs = true;
        app.audit = Some(audit);
        (app, ctx)
    }
    #[test]
    fn long_advanced_gesture_is_reported_before_replay_can_edit_cmd() {
        let fixture = Fixture::new();
        let (mut app, ctx) = start(&fixture);
        for i in 0..=5 {
            frame(&mut app, &ctx, 0.1 + i as f64 * 0.01);
        }
        assert_eq!(app.transcript.draft(), "12,6,");
        frame(&mut app, &ctx, 1.3);
        assert!(!app.advanced_layers);
        let before = app.controller.coordinator().snapshot();
        frame(&mut app, &ctx, 1.31);
        assert!(
            app.audit.as_ref().unwrap().awaiting_screenshot,
            "failed readiness must terminate replay before Ctrl+A/text"
        );
        assert!(
            app.audit
                .as_ref()
                .unwrap()
                .checks
                .iter()
                .any(|check| check.starts_with("FAILED"))
        );
        for i in 0..8 {
            frame(&mut app, &ctx, 1.32 + i as f64 * 0.01);
        }
        assert_eq!(app.transcript.draft(), "12,6,");
        assert_eq!(app.controller.coordinator().snapshot(), before);
    }
    #[test]
    fn absent_rename_modal_never_receives_synthetic_text_in_cmd() {
        let fixture = Fixture::new();
        let (mut app, ctx) = start(&fixture);
        app.audit.as_mut().unwrap().step = 12;
        app.controller.submit("Line").unwrap();
        app.controller.submit("0,0,0").unwrap();
        app.transcript.set_draft("12,6,");
        let events = frame(&mut app, &ctx, 0.1);
        assert!(
            !events.iter().any(|event| matches!(
                event,
                egui::Event::Text(_) | egui::Event::Key { key: Key::A, .. }
            )),
            "no modal means no editor input"
        );
        assert_eq!(app.transcript.draft(), "12,6,");
        assert!(app.controller.is_running());
    }
    #[test]
    fn ordinary_advanced_click_records_acceptance_without_reentrant_context_lock() {
        let fixture = Fixture::new();
        let (mut app, ctx) = start(&fixture);
        for i in 0..=9 {
            frame(&mut app, &ctx, 0.1 + i as f64 * 0.01);
        }
        assert!(app.advanced_layers);
        assert_eq!(
            app.controller.layer_dialog().unwrap().kind(),
            LayerDialogKind::Rename
        );
        assert!(
            app.audit
                .as_ref()
                .unwrap()
                .layer_clicks
                .iter()
                .any(|row| row["control"] == "Advanced" && row["clicked"] == true)
        );
        assert_eq!(app.transcript.draft(), "12,6,");
    }
    #[test]
    fn startup_focus_wait_is_bounded_and_never_fabricates_os_focus() {
        let fixture = Fixture::new();
        let (mut app, ctx) = start(&fixture);
        let mut raw = egui::RawInput {
            focused: false,
            time: Some(0.1),
            ..Default::default()
        };
        <MatrixApp as eframe::App>::raw_input_hook(&mut app, &ctx, &mut raw);
        assert!(!raw.focused);
        assert!(raw.events.is_empty());
        assert_eq!(app.audit.as_ref().unwrap().step, 0);
        assert!(!app.audit.as_ref().unwrap().awaiting_screenshot);
        app.audit.as_mut().unwrap().native_focus_wait_started =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(4));
        <MatrixApp as eframe::App>::raw_input_hook(&mut app, &ctx, &mut raw);
        assert!(!raw.focused);
        assert!(raw.events.is_empty());
        assert!(app.audit.as_ref().unwrap().awaiting_screenshot);
        assert!(app.audit.as_ref().unwrap().checks[0].contains("native window lost focus"));
    }
}
