// SPDX-License-Identifier: LGPL-2.1-or-later
//! The single door for menu placement in the UI (OM9-L3-01-01-003 §6).
//!
//! Menu, mouse, buttons, API and audit all reach `MatrixApp::apply_panel_op`;
//! there is no second path. This module also owns the drag/preview state, the
//! exact user-visible messages of the design and the cancel rules.
use super::*;
use crate::panel_ops::{
    self, CancelReason, ColumnSide, DropIndicator, DropTarget, DropZones, PanelNote, PanelOp,
    PanelOpError, PanelOutcome, PanelReport, WorkArea,
};

/// One menu being dragged: which one, and the last finite pointer position.
pub(super) struct SectionDrag {
    pub(super) section: SectionId,
    pub(super) last_pointer: Option<[f32; 2]>,
}

/// What the user currently sees while dragging.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropPreview {
    pub section: SectionId,
    pub target: DropTarget,
    pub indicator: Option<DropIndicator>,
}

/// The last placement operation, for the audit receipt.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelOpRecord {
    pub op: PanelOp,
    pub ok: bool,
    pub report: Option<PanelReport>,
    pub error: Option<PanelOpError>,
}

enum MessageKind {
    Info,
    Warning,
}

impl MatrixApp {
    /// **The only mutation path for menu placement.**
    pub fn apply_panel_op(
        &mut self,
        op: PanelOp,
    ) -> std::result::Result<PanelReport, PanelOpError> {
        if self.region_error.is_some()
            && matches!(op, PanelOp::Float { .. } | PanelOp::SetFloatRect { .. })
        {
            let error = PanelOpError::LayoutBlocked;
            self.report_panel_error(&error);
            self.last_panel_op = Some(PanelOpRecord {
                op,
                ok: false,
                report: None,
                error: Some(error.clone()),
            });
            return Err(error);
        }
        match self.dock.apply_panel_op(op) {
            Ok(report) => {
                self.dock_dirty = true;
                if matches!(report.outcome, PanelOutcome::Floated { .. })
                    || report
                        .notes
                        .iter()
                        .any(|note| matches!(note, PanelNote::Recovered { .. }))
                {
                    self.float_resync = true;
                }
                self.last_panel_op = Some(PanelOpRecord {
                    op,
                    ok: true,
                    report: Some(report.clone()),
                    error: None,
                });
                self.report_panel_ok(&report);
                Ok(report)
            }
            Err(error) => {
                self.report_panel_error(&error);
                self.last_panel_op = Some(PanelOpRecord {
                    op,
                    ok: false,
                    report: None,
                    error: Some(error.clone()),
                });
                Err(error)
            }
        }
    }

    pub fn section_drag(&self) -> Option<SectionId> {
        self.section_drag.as_ref().map(|drag| drag.section)
    }

    /// Start a drag from a title bar. A reserved menu never starts one.
    pub(super) fn begin_section_drag(&mut self, id: SectionId) {
        if !id.is_movable() {
            return;
        }
        self.section_drag = Some(SectionDrag {
            section: id,
            last_pointer: None,
        });
    }

    /// Cancel an active drag because the menus changed underneath it.
    pub(super) fn cancel_section_drag(&mut self, reason: CancelReason) {
        let Some(drag) = self.section_drag.take() else {
            return;
        };
        self.drop_preview = None;
        let error = PanelOpError::DragCancelled {
            section: drag.section,
            reason,
        };
        let (text, kind) = panel_message_op_error(&error);
        if matches!(reason, CancelReason::LayoutChanged) {
            self.append_panel_history(&text);
            return;
        }
        match kind {
            MessageKind::Info => self.route_panel_message(&text, MessageKind::Info),
            MessageKind::Warning => self.report_panel_error(&error),
        }
    }

    /// Per frame: update the pointer, resolve the drop, paint the indicator and
    /// apply the operation when the pointer is released.
    pub(super) fn panel_drag_frame(&mut self, ctx: &egui::Context, zones: DropZones) {
        let Some(section) = self.section_drag.as_ref().map(|drag| drag.section) else {
            self.drop_preview = None;
            return;
        };
        if !self.dock.section(section).visible {
            self.cancel_section_drag(CancelReason::LayoutChanged);
            return;
        }
        let pointer = ctx
            .input(|input| input.pointer.latest_pos().or(input.pointer.interact_pos()))
            .map(|pos| [pos.x, pos.y]);
        if let Some(pointer) = pointer.filter(|p| p[0].is_finite() && p[1].is_finite())
            && let Some(drag) = self.section_drag.as_mut()
        {
            drag.last_pointer = Some(pointer);
        }
        let last_pointer = self
            .section_drag
            .as_ref()
            .and_then(|drag| drag.last_pointer);
        let target = panel_ops::resolve_drop(&self.dock, section, last_pointer, &zones);
        let indicator = panel_ops::drop_indicator(target, &zones, self.work_area);
        self.drop_preview = Some(DropPreview {
            section,
            target,
            indicator,
        });
        if let Some(indicator) = indicator {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("om9-section-drop-preview"),
            ));
            let rect = egui::Rect::from_min_size(
                egui::pos2(indicator.rect.x, indicator.rect.y),
                egui::Vec2::new(indicator.rect.width, indicator.rect.height),
            );
            let colour = Color32::from_rgb(150, 55, 55);
            if indicator.line {
                painter.rect_filled(rect, 0, colour);
            } else {
                painter.rect_filled(rect, 0, colour.gamma_multiply(0.2));
                painter.rect_stroke(rect, 0, Stroke::new(2., colour), egui::StrokeKind::Inside);
            }
        }
        ctx.set_cursor_icon(match target {
            DropTarget::Cancel { .. } => egui::CursorIcon::NoDrop,
            _ => egui::CursorIcon::Grabbing,
        });
        if ctx.input(|input| input.pointer.any_down()) {
            ctx.request_repaint();
            return;
        }
        // Released: apply exactly what the last pressed frame previewed.
        self.section_drag = None;
        self.drop_preview = None;
        match panel_ops::drop_op(section, target, self.work_area) {
            Ok(op) => {
                let _ = self.apply_panel_op(op);
            }
            Err(error) => {
                self.report_panel_error(&error);
                self.last_panel_op = Some(PanelOpRecord {
                    op: PanelOp::Dock {
                        section,
                        side: ColumnSide::Left,
                    },
                    ok: false,
                    report: None,
                    error: Some(error),
                });
            }
        }
    }

    fn report_panel_ok(&mut self, report: &PanelReport) {
        if report.notes.is_empty() {
            // A readback of a visible window move is not worth a message.
            if matches!(report.outcome, PanelOutcome::FloatRectSet { .. }) {
                return;
            }
            let (text, _) = panel_message_outcome(&report.outcome);
            self.route_panel_message(&text, MessageKind::Info);
            return;
        }
        for note in &report.notes {
            let (text, _) = panel_message_note(note);
            self.append_panel_history(&text);
            self.dock_diagnostics.push(text);
        }
        if let Some(note) = report.notes.last() {
            let (text, _) = panel_message_note(note);
            self.status.warning(&text);
        }
    }

    fn report_panel_error(&mut self, error: &PanelOpError) {
        let (text, kind) = panel_message_op_error(error);
        // An unacknowledged failure keeps the status line: every other panel
        // message goes to the command history only (the L3-007 rule).
        if matches!(kind, MessageKind::Warning) && self.status.kind().is_failure() {
            self.append_panel_history(&text);
            return;
        }
        match kind {
            MessageKind::Info => self.route_panel_message(&text, MessageKind::Info),
            MessageKind::Warning => self.route_panel_message(&text, MessageKind::Warning),
        }
    }

    fn route_panel_message(&mut self, text: &str, kind: MessageKind) {
        self.append_panel_history(text);
        // An unacknowledged failure keeps the status line (L3-007): a panel
        // message never hides it, it only reaches the command history.
        if self.status.kind().is_failure() {
            return;
        }
        match kind {
            MessageKind::Info => {
                self.status.info(text);
            }
            MessageKind::Warning => {
                self.status.warning(text);
            }
        }
    }

    /// Append one panel line to the command history.
    ///
    /// Growing the history moves the CMD text, so the cursor the field stored on
    /// the previous frame can end up inside the protected history and
    /// `Transcript::insert` would refuse the next keystroke (the L3-007 rule,
    /// re-checked here by `panel_history_never_drops_a_keystroke_...`). Asking
    /// for CMD focus makes the next frame put the cursor back at the draft.
    fn append_panel_history(&mut self, text: &str) {
        self.transcript.append_history(text);
        self.focus_cmd = true;
    }

    /// Record an attempt that failed before the model was reached.
    fn record_panel_refusal(&mut self, op: PanelOp, error: PanelOpError) {
        self.report_panel_error(&error);
        self.last_panel_op = Some(PanelOpRecord {
            op,
            ok: false,
            report: None,
            error: Some(error),
        });
    }

    /// The live rectangles a drop can hit this frame.
    pub(super) fn panel_drop_zones(
        &self,
        window: egui::Rect,
        bars: &[(SectionId, ColumnSide, egui::Rect)],
        left_bottom: f32,
        right_bottom: f32,
    ) -> DropZones {
        let region =
            |rect: egui::Rect| RegionRect::new(rect.min.x, rect.min.y, rect.width(), rect.height());
        let views: Vec<RegionRect> = self.view_rects.values().map(|rect| region(*rect)).collect();
        let right_strip = if self.right_dock_rect.is_none()
            && window.width() >= dock_layout::NARROW_WINDOW_WIDTH
        {
            // The rightmost RIGHT_DROP_STRIP of the views' union (design §5).
            views
                .iter()
                .copied()
                .reduce(|a, b| {
                    RegionRect::new(
                        a.x.min(b.x),
                        a.y.min(b.y),
                        a.right().max(b.right()) - a.x.min(b.x),
                        a.bottom().max(b.bottom()) - a.y.min(b.y),
                    )
                })
                .map(|union| {
                    RegionRect::new(
                        union.right() - panel_ops::RIGHT_DROP_STRIP,
                        union.y,
                        panel_ops::RIGHT_DROP_STRIP,
                        union.height,
                    )
                })
        } else {
            None
        };
        DropZones {
            window: Some(region(window)),
            left: self.sidebar_rect.map(|rect| panel_ops::ColumnZone {
                rect: region(rect),
                content_bottom: left_bottom,
            }),
            right: self.right_dock_rect.map(|rect| panel_ops::ColumnZone {
                rect: region(rect),
                content_bottom: right_bottom,
            }),
            right_strip,
            bars: bars
                .iter()
                .map(|(id, side, rect)| (*id, *side, region(*rect)))
                .collect(),
            views,
        }
    }

    /// The work area of this frame, or `None` when the window is not usable.
    pub(super) fn update_work_area(&mut self, size: egui::Vec2) {
        self.work_area = WorkArea::for_window([size.x, size.y]).ok();
    }

    /// The placement wrappers kept for tests and audits: one line each.
    pub fn move_menu_section(&mut self, id: SectionId, delta: i32) -> bool {
        self.apply_panel_op(PanelOp::Move { section: id, delta })
            .is_ok()
    }

    pub fn dock_menu_section(&mut self, id: SectionId, side: DockSide) -> bool {
        let side = match ColumnSide::try_from_dock_side(side) {
            Ok(side) => side,
            Err(error) => {
                self.record_panel_refusal(
                    PanelOp::Dock {
                        section: id,
                        side: ColumnSide::Left,
                    },
                    error,
                );
                return false;
            }
        };
        self.apply_panel_op(PanelOp::Dock { section: id, side })
            .is_ok()
    }

    pub fn float_menu_section(&mut self, id: SectionId, rect: FloatRect) -> bool {
        let Some(work) = self.work_area else {
            self.record_panel_refusal(
                PanelOp::Float {
                    section: id,
                    rect,
                    work: WorkArea {
                        x: 0.,
                        y: 0.,
                        width: 0.,
                        height: 0.,
                    },
                    origin: panel_ops::FloatOrigin::Api,
                },
                PanelOpError::NoWorkArea,
            );
            return false;
        };
        self.apply_panel_op(PanelOp::Float {
            section: id,
            rect,
            work,
            origin: panel_ops::FloatOrigin::Api,
        })
        .is_ok()
    }

    pub fn toggle_menu_section(&mut self, id: SectionId) -> bool {
        self.apply_panel_op(PanelOp::ToggleCollapsed { section: id })
            .is_ok()
    }

    /// The X of a floating menu window: back to the home column, never lost.
    pub(super) fn close_float_menu(&mut self, id: SectionId) {
        let _ = self.apply_panel_op(PanelOp::DockHome { section: id });
    }

    /// The float-window readback: skipped while the layout has a region error,
    /// and skipped when the clamped rectangle is already what is stored (an
    /// egui window smaller than the float minimum would otherwise report the
    /// same recovery on every frame).
    pub(super) fn readback_float(&mut self, id: SectionId, rect: FloatRect) {
        let Some(work) = self.work_area else {
            return;
        };
        if self.dock.section(id).float == Some(work.clamp(rect)) {
            return;
        }
        let _ = self.apply_panel_op(PanelOp::SetFloatRect {
            section: id,
            rect,
            work,
        });
    }

    /// The frame-level recovery: a floating menu that no longer fits is pulled
    /// back inside the work area through the same door.
    pub(super) fn resync_floats(&mut self) {
        if !self.float_resync {
            return;
        }
        let Some(work) = self.work_area else {
            return;
        };
        self.float_resync = false;
        for id in self.dock.floating_sections() {
            let Some(rect) = self.dock.section(id).float else {
                continue;
            };
            if !work.contains(rect) {
                let _ = self.apply_panel_op(PanelOp::SetFloatRect {
                    section: id,
                    rect,
                    work,
                });
            }
        }
    }

    /// The scripted real-window drag of `OM9_UI_AUDIT_PANELS` (L3-003). It runs
    /// the same events a mouse produces: press a title bar, move, preview,
    /// release (or press Escape first), and records what the frame saw.
    pub(super) fn panel_audit_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let Some(scenario) = self.audit.as_ref().and_then(|audit| audit.panels.clone()) else {
            return;
        };
        // While a screenshot is in flight the drag is held, so the image shows
        // the state the receipt describes.
        if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.awaiting_screenshot)
        {
            ctx.request_repaint();
            return;
        }
        let step = self.audit.as_ref().expect("audit").step;
        match scenario.as_str() {
            "roll-up" => {
                self.panel_audit_roll_up(ctx, raw, step);
                return;
            }
            "float-drop" => {
                self.panel_audit_float_drop(ctx, raw, step);
                return;
            }
            "object-info" => {
                self.panel_audit_object_info(ctx, step);
                // Without this the default drag script would run as well and
                // record its own entry under the same name (round 20).
                return;
            }
            "readback-recover" => {
                self.panel_audit_readback(ctx, raw, step);
                return;
            }
            _ => {}
        }
        let source = match scenario.as_str() {
            "reorder" => SectionId::Projects,
            _ => SectionId::Display,
        };
        let Some(bar) = self.section_rect(source.key()) else {
            // The first frames have not drawn the column yet.
            ctx.request_repaint();
            return;
        };
        let event = |raw: &mut egui::RawInput, pos: [f32; 2], pressed: Option<bool>| {
            let pos = egui::pos2(pos[0], pos[1]);
            raw.events.push(egui::Event::PointerMoved(pos));
            if let Some(pressed) = pressed {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
            }
        };
        // The target is fixed on the first frame that can see every rectangle it
        // needs; docking a menu right must not move it under the script.
        if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.panel_target.is_none())
        {
            let target = self.panel_audit_target(&scenario, bar);
            match target {
                Some(target) => self.audit.as_mut().expect("audit").panel_target = Some(target),
                None => {
                    ctx.request_repaint();
                    return;
                }
            }
        }
        let target = self
            .audit
            .as_ref()
            .and_then(|audit| audit.panel_target)
            .expect("target");
        // Escape cancels one step earlier than a release applies.
        let record_step = if scenario == "escape" { 6 } else { 5 };
        match step {
            0 => event(raw, [bar.center().x, bar.center().y], Some(true)),
            1 => {
                let start = [bar.center().x, bar.center().y + 10.];
                event(raw, start, None);
            }
            2 => event(raw, target, None),
            3 => {
                // The preview is what the release will apply.
                let preview = self.drop_preview.map(|preview| {
                    serde_json::json!({
                        "target": preview.target,
                        "indicator": preview.indicator,
                    })
                });
                self.audit.as_mut().expect("audit").panel_drag_observed = preview;
            }
            4 if scenario == "escape" => {
                raw.events.push(egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                });
            }
            4 => event(raw, target, Some(false)),
            5 if scenario == "escape" => event(raw, target, Some(false)),
            // The recording step: the release has been applied by the previous
            // frame, so the state is the one the user would see.
            step if step == record_step => {
                let extra = self
                    .audit
                    .as_ref()
                    .and_then(|audit| audit.panel_drag_observed.clone())
                    .unwrap_or(serde_json::Value::Null);
                self.record_panel_checks(&scenario, extra);
                self.request_panel_shot(ctx);
            }
            // Past the script: request the image if no step did, so the process
            // can never spin without a receipt.
            _ if step > record_step
                && !self
                    .audit
                    .as_ref()
                    .is_some_and(|audit| audit.awaiting_screenshot) =>
            {
                self.request_panel_shot(ctx);
            }
            _ => {}
        }
        self.audit.as_mut().expect("audit").step += 1;
        ctx.request_repaint();
    }

    /// The roll-up scenario (design review M1): click PROJECTS, click it again.
    fn panel_audit_roll_up(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput, step: usize) {
        let Some(bar) = self.section_rect(SectionId::Projects.key()) else {
            ctx.request_repaint();
            return;
        };
        let at = [bar.center().x, bar.center().y];
        let click = |raw: &mut egui::RawInput, pressed: bool| {
            let pos = egui::pos2(at[0], at[1]);
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        };
        let state = || {
            serde_json::json!({
                "collapsed": self.dock.section(SectionId::Projects).collapsed,
                "status": self.status.message(),
                "order": self.dock.left_sections().iter().map(|id| id.key()).collect::<Vec<_>>(),
            })
        };
        match step {
            // Step 0 is a neutral click on the status row: a freshly started
            // window can swallow the very first click while it takes focus, and a
            // scenario that then reports "the click did nothing" would be lying
            // about the interface. Focusing first makes the two real clicks land.
            0 => {
                // Do not burn the step while the status row is not drawn yet: the
                // focus click must actually happen, or the first real click is lost
                // intermittently (round 21).
                let Some(status) = self.status_rect else {
                    ctx.request_repaint();
                    return;
                };
                {
                    let pos = status.center();
                    raw.events.push(egui::Event::PointerMoved(pos));
                    raw.events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    });
                    raw.events.push(egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: Modifiers::NONE,
                    });
                }
            }
            1 => click(raw, true),
            2 => click(raw, false),
            3 => {
                let after_first = state();
                self.audit.as_mut().expect("audit").panel_drag_observed =
                    Some(serde_json::json!({"after_first": after_first}));
            }
            4 => click(raw, true),
            5 => click(raw, false),
            6 => {
                let mut extra = self
                    .audit
                    .as_ref()
                    .and_then(|audit| audit.panel_drag_observed.clone())
                    .unwrap_or(serde_json::Value::Null);
                extra["after_second"] = state();
                self.record_panel_checks("roll-up", extra);
                self.request_panel_shot(ctx);
            }
            _ => {}
        }
        self.audit.as_mut().expect("audit").step += 1;
        ctx.request_repaint();
    }

    /// The float-then-close scenario (M3): drag DISPLAY into a viewport, then
    /// press the X of the floating window.
    fn panel_audit_float_drop(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
        step: usize,
    ) {
        // Only the first steps need the title bar: once DISPLAY floats there is
        // no bar in the column any more, and the later steps read the window.
        let bar = self.section_rect(SectionId::Display.key());
        if bar.is_none() && step <= 2 {
            ctx.request_repaint();
            return;
        }
        if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.panel_target.is_none())
        {
            let Some(bar) = bar else {
                ctx.request_repaint();
                return;
            };
            let Some(target) = self.panel_audit_target("float", bar) else {
                ctx.request_repaint();
                return;
            };
            self.audit.as_mut().expect("audit").panel_target = Some(target);
        }
        let target = self
            .audit
            .as_ref()
            .and_then(|audit| audit.panel_target)
            .expect("target");
        let event = |raw: &mut egui::RawInput, pos: [f32; 2], pressed: Option<bool>| {
            let pos = egui::pos2(pos[0], pos[1]);
            raw.events.push(egui::Event::PointerMoved(pos));
            if let Some(pressed) = pressed {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
            }
        };
        // The floating window's live rectangle, recorded with the evidence so a
        // later run can aim at its close box (the click itself is not scripted:
        // it is the owner's item M3).
        let window_rect = ctx
            .memory(|memory| {
                memory.area_rect(egui::Id::new(("om9-float-menu", SectionId::Display.key())))
            })
            .map(|rect| [rect.min.x, rect.min.y, rect.width(), rect.height()]);
        match step {
            0 | 1 => {
                if let Some(bar) = bar {
                    let y = if step == 0 {
                        bar.center().y
                    } else {
                        bar.center().y + 10.
                    };
                    event(
                        raw,
                        [bar.center().x, y],
                        if step == 0 { Some(true) } else { None },
                    );
                }
            }
            2 => event(raw, target, None),
            3 => {
                let preview = self.drop_preview.map(|preview| {
                    serde_json::json!({"target": preview.target, "indicator": preview.indicator})
                });
                self.audit.as_mut().expect("audit").panel_drag_observed = preview;
            }
            4 => event(raw, target, Some(false)),
            5 => {
                let extra = serde_json::json!({
                    "preview": self.audit.as_ref().and_then(|audit| audit.panel_drag_observed.clone()),
                    "floating_after_drop": self.dock.floating_sections().iter().map(|id| id.key()).collect::<Vec<_>>(),
                    "display_home": self.dock.section(SectionId::Display).home.map(|home| serde_json::json!({
                        "side": home.side,
                        "index": home.index,
                    })),
                    "display_float": self.dock.section(SectionId::Display).float.map(|rect| [rect.x, rect.y, rect.width, rect.height]),
                    "float_window_px": window_rect,
                });
                self.record_panel_checks("float-drop", extra);
                self.request_panel_shot(ctx);
            }
            _ => {}
        }
        self.audit.as_mut().expect("audit").step += 1;
        ctx.request_repaint();
    }

    /// The Object Info scenario (OM9-L3-03-14-005): draw one line, select it, and
    /// record the label the overlay paints.
    fn panel_audit_object_info(&mut self, ctx: &egui::Context, step: usize) {
        if step == 0 {
            let _ = self
                .controller
                .dispatch(crate::session::LINE, crate::session::InvocationOrigin::Cmd);
            let _ = self.controller.accept_world_point([0., 0., 0.]);
            let _ = self.controller.accept_world_point([10., 4., 0.]);
            let ids: Vec<_> = self
                .controller
                .document()
                .entities
                .keys()
                .copied()
                .collect();
            if let Some(id) = ids.first() {
                let _ = self.controller.select_objects(&[*id]);
            }
        }
        if step == 3 {
            let limits = *self.controller.coordinator().limits();
            let extra = self
                .controller
                .document()
                .entities
                .keys()
                .copied()
                .next()
                .and_then(|id| self.controller.document().object_info(id, &limits).ok())
                .map(|info| {
                    serde_json::json!({
                        "object_info_kind": info.kind,
                        "object_info_label": om9_core::object_info::label_text(&info),
                        "selected": self.controller.selected().len(),
                    })
                })
                .unwrap_or_else(|| serde_json::json!({ "object_info_kind": null }));
            self.record_panel_checks("object-info", extra);
            self.request_panel_shot(ctx);
        }
        self.audit.as_mut().expect("audit").step += 1;
        // Keep stepping until the shot is requested: once it is, the input hook
        // repaints for us. Without this the window can go idle before step 3 and
        // the receipt is never written (the intermittent timeout, round 21).
        if step < 4 {
            ctx.request_repaint();
        }
    }

    /// The readback-recovery scenario (M5): float DISPLAY, then let the real
    /// window report a rectangle that no longer fits, so the model pulls it back
    /// inside and says so.
    fn panel_audit_readback(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput, step: usize) {
        // Only the first steps need the title bar (see `panel_audit_float_drop`).
        let bar = self.section_rect(SectionId::Display.key());
        if bar.is_none() && step <= 2 {
            ctx.request_repaint();
            return;
        }
        if self
            .audit
            .as_ref()
            .is_some_and(|audit| audit.panel_target.is_none())
        {
            let Some(view) = self.view_rects.values().copied().reduce(|a, b| a.union(b)) else {
                ctx.request_repaint();
                return;
            };
            // Bottom-LEFT of the views: close to the window bottom edge (so the
            // shrink has to pull it back) and far from the right strip.
            self.audit.as_mut().expect("audit").panel_target =
                Some([view.min.x + 100., view.max.y - 20.]);
        }
        let target = self
            .audit
            .as_ref()
            .and_then(|audit| audit.panel_target)
            .expect("target");
        let event = |raw: &mut egui::RawInput, pos: [f32; 2], pressed: Option<bool>| {
            let pos = egui::pos2(pos[0], pos[1]);
            raw.events.push(egui::Event::PointerMoved(pos));
            if let Some(pressed) = pressed {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
            }
        };
        match step {
            0 | 1 => {
                if let Some(bar) = bar {
                    let y = if step == 0 {
                        bar.center().y
                    } else {
                        bar.center().y + 10.
                    };
                    event(
                        raw,
                        [bar.center().x, y],
                        if step == 0 { Some(true) } else { None },
                    );
                }
            }
            2 => event(raw, target, None),
            3 => event(raw, target, Some(false)),
            4 => {
                // Diagnostics for the M5 investigation: what this step really sees,
                // then a forced readback of a rectangle below the minimum, then the
                // state right after it (not a frame later).
                let before = self
                    .dock
                    .section(SectionId::Display)
                    .float
                    .map(|rect| [rect.x, rect.y, rect.width, rect.height]);
                let work = self.work_area;
                let mut diag = serde_json::json!({
                    "float_after_drop": before,
                    "work_area": work.map(|w| [w.x, w.y, w.width, w.height]),
                    "call": "skipped",
                });
                if let Some(work) = work {
                    self.readback_float(
                        SectionId::Display,
                        crate::dock_layout::FloatRect {
                            x: work.x + 20.,
                            y: work.y + 20.,
                            width: 300.,
                            height: 40.,
                        },
                    );
                    diag["call"] = serde_json::json!("called");
                }
                diag["message_after_force"] = serde_json::json!(self.status.message());
                diag["float_after_force"] =
                    serde_json::json!(self.dock.section(SectionId::Display).float.map(|rect| [
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height
                    ]));
                self.audit.as_mut().expect("audit").panel_drag_observed = Some(diag);
            }
            5 => {
                // The window keeps its size here: the audit window size is fixed
                // by the harness. What the step proves is that the rectangle the
                // real window reports is read back and repaired when it cannot
                // fit, with the recovery message the user sees.
                let extra = self
                    .audit
                    .as_ref()
                    .and_then(|audit| audit.panel_drag_observed.clone())
                    .unwrap_or(serde_json::Value::Null);
                let mut extra = extra;
                extra["float_after_readback"] =
                    serde_json::json!(self.dock.section(SectionId::Display).float.map(|rect| [
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height
                    ]));
                extra["notes"] =
                    serde_json::json!(self.last_panel_op.as_ref().and_then(|record| {
                        record.report.as_ref().map(|report| report.notes.clone())
                    }));
                self.record_panel_checks("readback-recover", extra);
                self.request_panel_shot(ctx);
            }
            _ => {}
        }
        self.audit.as_mut().expect("audit").step += 1;
        ctx.request_repaint();
    }

    /// The shared recording of a scenario: the columns, the status, the work
    /// area and whatever the scenario itself observed.
    fn record_panel_checks(&mut self, scenario: &str, extra: serde_json::Value) {
        let record = self.last_panel_op.as_ref().map(|record| {
            serde_json::json!({
                "op": record.op,
                "ok": record.ok,
                "outcome": record.report.as_ref().map(|report| &report.outcome),
                "notes": record.report.as_ref().map(|report| &report.notes),
                "error": record.error,
            })
        });
        let checks = serde_json::json!({
            "scenario": scenario,
            "observed_preview": self.audit.as_ref().and_then(|audit| audit.panel_drag_observed.clone()),
            "extra": extra,
            "last_op": record,
            "drag_in_progress": self.section_drag().is_some(),
            "drop_preview": self.drop_preview.map(|preview| serde_json::json!({
                "section": preview.section.key(),
                "target": preview.target,
            })),
            "left_order": self.dock.left_sections().iter().map(|id| id.key()).collect::<Vec<_>>(),
            "right_order": self.dock.right_sections().iter().map(|id| id.key()).collect::<Vec<_>>(),
            "floating": self.dock.floating_sections().iter().map(|id| id.key()).collect::<Vec<_>>(),
            "status_kind": self.status.kind().label(),
            "status_message": self.status.message(),
            "history_tail": self.transcript.history().lines().rev().take(4).collect::<Vec<_>>(),
            "work_area": self.work_area.map(|work| [work.x, work.y, work.width, work.height]),
            "entities": self.controller.document().entities.len(),
            "undo_len": self.controller.coordinator().undo_len(),
        });
        self.audit
            .as_mut()
            .expect("audit")
            .panel_checks
            .push(checks);
    }

    /// Ask for the image of this step, so a receipt can never be written
    /// without the checks it is supposed to carry.
    fn request_panel_shot(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        self.audit.as_mut().expect("audit").awaiting_screenshot = true;
    }

    /// Where the scripted drag goes for a scenario, in window points.
    fn panel_audit_target(&self, scenario: &str, bar: egui::Rect) -> Option<[f32; 2]> {
        match scenario {
            "reorder" => {
                let display = self.section_rect(SectionId::Display.key())?;
                Some([display.center().x, display.min.y + 3.])
            }
            "dock-right" => {
                let strip = self
                    .panel_drop_zones(
                        self.current_ui_context
                            .as_ref()
                            .map_or(bar, |ctx| ctx.content_rect()),
                        &self.panel_bars.clone(),
                        self.left_content_bottom,
                        self.right_content_bottom,
                    )
                    .right_strip?;
                Some([strip.x + strip.width / 2., strip.y + strip.height / 2.])
            }
            _ => {
                // A point inside one viewport, never the centre of the 2x2 union
                // (which falls in the gap between the views).
                let view = self
                    .view_rects
                    .iter()
                    .max_by_key(|(id, _)| id.title())
                    .map(|(_, rect)| *rect)?;
                Some([view.center().x, view.center().y])
            }
        }
    }
}

fn panel_message_outcome(outcome: &PanelOutcome) -> (String, MessageKind) {
    let text = match outcome {
        PanelOutcome::Collapsed { section, collapsed } => format!(
            "{} {}",
            section.title(),
            if *collapsed { "rolled up" } else { "expanded" }
        ),
        PanelOutcome::Moved { section, index, .. } => {
            format!("{} moved to menu position {}", section.title(), index + 1)
        }
        PanelOutcome::Docked { section, side, .. } => match side {
            ColumnSide::Right => format!("{} docked to the right menu column", section.title()),
            ColumnSide::Left => format!("{} docked in the left menu column", section.title()),
        },
        PanelOutcome::Floated { section, .. } => {
            format!("{} floated over the model", section.title())
        }
        PanelOutcome::FloatRectSet { section, .. } => format!("{} floated", section.title()),
    };
    (text, MessageKind::Info)
}

fn panel_message_note(note: &PanelNote) -> (String, MessageKind) {
    let text = match note {
        PanelNote::Recovered { section, .. } => {
            format!("{} was moved back inside the window", section.title())
        }
        PanelNote::OutsideMainWindow { section } => format!(
            "{} cannot float outside the main window; placed inside it",
            section.title()
        ),
    };
    (text, MessageKind::Warning)
}

fn panel_message_op_error(error: &PanelOpError) -> (String, MessageKind) {
    let warning = |text: String| (text, MessageKind::Warning);
    match error {
        PanelOpError::NotPermitted { .. } => warning(
            "Builders is reserved for builder tools and cannot be rolled up, moved or floated"
                .to_owned(),
        ),
        PanelOpError::UnknownSection { key } => {
            warning(format!("menu section '{key}' is not recognized"))
        }
        PanelOpError::Hidden { section } => warning(format!(
            "{} is hidden; show it before moving it",
            section.title()
        )),
        PanelOpError::InvalidTarget { .. } => {
            warning("menus dock only to the left or right column".to_owned())
        }
        PanelOpError::InvalidRect { section } => warning(format!(
            "{}: the floating rectangle is not valid",
            section.title()
        )),
        PanelOpError::InvalidWindow { width, height } => warning(format!(
            "the window {width}x{height} is too small or not valid for floating menus"
        )),
        PanelOpError::NoWorkArea => {
            warning("floating menus are not available before the window is shown".to_owned())
        }
        PanelOpError::LayoutBlocked => {
            warning("menus cannot float while the layout has an error".to_owned())
        }
        PanelOpError::NotDocked { section } => warning(format!(
            "{} is floating; dock it before reordering",
            section.title()
        )),
        PanelOpError::NotFloating { section } => {
            warning(format!("{} is not floating", section.title()))
        }
        PanelOpError::AlreadyFloating { section } => {
            warning(format!("{} is already floating", section.title()))
        }
        PanelOpError::AlreadyThere {
            section,
            side,
            index,
        } => warning(format!(
            "{} is already at menu position {} on the {}",
            section.title(),
            index + 1,
            side.label()
        )),
        PanelOpError::DragCancelled { section, reason } => {
            let suffix = match reason {
                CancelReason::NoTarget => ": not over a menu column or viewport",
                CancelReason::Escape => " (Esc)",
                CancelReason::PointerLost => ": pointer lost",
                CancelReason::LayoutChanged => ": menus were reset",
            };
            let text = format!("{} drag cancelled{suffix}", section.title());
            match reason {
                CancelReason::Escape => (text, MessageKind::Info),
                _ => (text, MessageKind::Warning),
            }
        }
    }
}
