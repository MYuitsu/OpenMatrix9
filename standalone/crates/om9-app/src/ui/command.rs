// SPDX-License-Identifier: LGPL-2.1-or-later
//! One protected text document and one native editor state. The live row is a
//! pinned projection of that same document, never a second editable text buffer.
use super::*;
use std::sync::Arc;

impl MatrixApp {
    pub(super) fn command_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let id = egui::Id::new("matrix-command-transcript");
        let popup_open = self.viewport_menu_owns_frame || egui::Popup::is_any_open(&ctx);
        let focused = !popup_open && ctx.memory(|m| m.has_focus(id));
        // Enter can confirm an IME candidate. Egui keeps composition purpose
        // private, so track its public event lifecycle before consuming keys.
        // A commit and Enter in one frame must also wait for a later Enter.
        let was_composing = self.cmd_ime_composing;
        let ime_frame =
            focused && ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))));
        if focused {
            ctx.input(|i| {
                for event in &i.events {
                    match event {
                        egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                            self.cmd_ime_composing = !text.is_empty()
                        }
                        egui::Event::Ime(egui::ImeEvent::Commit(_))
                        | egui::Event::WindowFocused(false) => self.cmd_ime_composing = false,
                        _ => {}
                    }
                }
            });
        } else {
            self.cmd_ime_composing = false;
        }
        let composing = was_composing || self.cmd_ime_composing || ime_frame;
        let enter =
            focused && !composing && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        if focused && !composing && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp))
        {
            self.transcript.recall_up();
            self.focus_cmd = true;
        }
        if focused
            && !composing
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown))
        {
            self.transcript.recall_down();
            self.focus_cmd = true;
        }
        if enter {
            self.submit_draft();
        }
        if self.focus_cmd && !popup_open {
            ctx.memory_mut(|m| m.request_focus(id));
            let mut state = TextEdit::load_state(&ctx, id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(CCursor::new(
                    self.transcript.text().chars().count(),
                ))));
            state.store(&ctx, id);
            self.focus_cmd = false;
        }

        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let row_height = ui.fonts_mut(|f| f.row_height(&font)) + 2.;
        let area = ui.available_rect_before_wrap();
        let history_height = (area.height() - row_height - 2.).max(1.);
        // The editor keeps every character for selection, keyboard editing,
        // clipboard, and IME. Only its live row's glyphs are hidden here; that
        // row is painted below, outside the history scroll clipping rectangle.
        let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, _: f32| {
            // Input can retire old history at the byte limit. Resolve from the
            // current document on every layout, never a pre-edit byte offset.
            // History ends at a newline; the prompt/draft occupy one live row.
            let history_bytes = buf.as_str().rfind('\n').map_or(0, |i| i + 1);
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = f32::INFINITY;
            job.append(
                &buf.as_str()[..history_bytes],
                0.,
                egui::TextFormat {
                    font_id: font.clone(),
                    color: Color32::BLACK,
                    ..Default::default()
                },
            );
            job.append(
                &buf.as_str()[history_bytes..],
                0.,
                egui::TextFormat {
                    font_id: font.clone(),
                    color: Color32::TRANSPARENT,
                    ..Default::default()
                },
            );
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let scrolled = egui::ScrollArea::both()
            .id_salt("command-scroll")
            .animated(false)
            .auto_shrink([false, false])
            .max_height(history_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.visuals_mut().text_cursor.stroke = Stroke::NONE;
                TextEdit::multiline(&mut self.transcript)
                    .interactive(!popup_open)
                    .id(id)
                    .font(font.clone())
                    .text_color(Color32::BLACK)
                    .frame(egui::Frame::new())
                    .desired_width(f32::INFINITY)
                    .desired_rows(1)
                    .return_key(None)
                    .layouter(&mut layouter)
                    .show(ui)
            });
        self.cmd_history_rect = Some(scrolled.inner_rect);
        let mut state = scrolled.inner.state;
        let full_galley = scrolled.inner.galley;
        let full_origin = scrolled.inner.galley_pos;
        let history_chars = self.transcript.history().chars().count();
        let live: String = self.transcript.text().chars().skip(history_chars).collect();
        let live_galley = ui.fonts_mut(|f| f.layout_no_wrap(live, font, Color32::BLACK));
        let rect = Rect::from_min_size(
            egui::pos2(area.left(), area.bottom() - row_height),
            Vec2::new(area.width(), row_height),
        );
        self.cmd_rect = Some(rect);
        let response = ui.interact(
            rect,
            id.with("pinned-row"),
            if popup_open {
                Sense::hover()
            } else {
                Sense::click_and_drag()
            },
        );
        let painter = ui.painter().with_clip_rect(rect);
        painter.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1., Color32::from_gray(190)),
        );
        let mut origin = rect.min + Vec2::new(0., 1.);
        let focused = ctx.memory(|m| m.has_focus(id));
        if focused
            && let Some(range) = state.cursor.char_range()
            && range.primary.index.0 >= history_chars
        {
            let local = CCursor::new(range.primary.index.0 - history_chars);
            let x = live_galley.pos_from_cursor(local).left();
            if x < self.cmd_horizontal_offset {
                self.cmd_horizontal_offset = x;
            }
            if x > self.cmd_horizontal_offset + rect.width() - 4. {
                self.cmd_horizontal_offset = (x - rect.width() + 4.).max(0.);
            }
        }
        self.cmd_horizontal_offset = self
            .cmd_horizontal_offset
            .min((live_galley.size().x - rect.width() + 4.).max(0.));
        origin.x -= self.cmd_horizontal_offset;
        let history_drag = ui.is_being_dragged(id);
        let pointer = response.interact_pointer_pos().or_else(|| {
            if history_drag {
                ui.input(|i| i.pointer.interact_pos())
            } else {
                None
            }
        });
        if let Some(pos) = pointer {
            let cursor = if pos.y < rect.top() {
                full_galley.cursor_from_pos(pos - full_origin)
            } else {
                let local = live_galley.cursor_from_pos(pos - origin);
                CCursor::new(history_chars + local.index.0)
            };
            let dragged = ui.is_being_dragged(response.id) || (rect.contains(pos) && history_drag);
            if state
                .cursor
                .pointer_interaction(ui, &response, cursor, &full_galley, dragged)
            {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
        if response.hovered() {
            ui.set_cursor_icon(egui::CursorIcon::Text);
        }
        if response.clicked() || response.dragged() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        if ctx.memory(|m| m.has_focus(id))
            && let Some(range) = state.cursor.char_range()
        {
            let selection = range.as_sorted_char_range();
            let start = selection.start.0.saturating_sub(history_chars);
            let end = selection.end.0.saturating_sub(history_chars);
            if start < end {
                let a = live_galley.pos_from_cursor(CCursor::new(start));
                let b = live_galley.pos_from_cursor(CCursor::new(end));
                painter.rect_filled(
                    Rect::from_min_max(
                        origin + Vec2::new(a.left(), 0.),
                        origin + Vec2::new(b.left(), row_height - 2.),
                    ),
                    0,
                    Color32::from_rgb(165, 201, 220),
                );
                if self.cmd_ime_composing {
                    painter.line_segment(
                        [
                            origin + Vec2::new(a.left(), row_height - 2.),
                            origin + Vec2::new(b.left(), row_height - 2.),
                        ],
                        Stroke::new(1., Color32::from_rgb(30, 70, 130)),
                    );
                }
            }
            if range.primary.index.0 >= history_chars {
                let caret = live_galley
                    .pos_from_cursor(CCursor::new(range.primary.index.0 - history_chars))
                    .translate(origin.to_vec2());
                painter.line_segment(
                    [caret.left_top(), caret.left_bottom()],
                    Stroke::new(1., Color32::BLACK),
                );
                // Native IME composition still belongs to the same TextEdit;
                // place its candidate window at the visible pinned caret.
                ui.output_mut(|o| {
                    if let Some(ime) = &mut o.ime {
                        ime.rect = rect;
                        ime.cursor_rect = caret;
                    }
                });
            }
        }
        painter.galley(origin, Arc::clone(&live_galley), Color32::BLACK);
        state.store(&ctx, id);
        ui.allocate_rect(rect, Sense::hover());
    }
}
