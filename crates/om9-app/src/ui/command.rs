// SPDX-License-Identifier: LGPL-2.1-or-later
//! One protected text document and one native editor state. The live row is a
//! pinned projection of that same document, never a second editable text buffer.
use super::*;
use crate::command_options::CommandOptionInterface;
use std::sync::Arc;

// Sampled from the user's 2026-10-20 green/gold reference.
pub(super) const COMMAND_BG: Color32 = Color32::from_rgb(1, 57, 28);
const COMMAND_INPUT_BG: Color32 = Color32::from_rgb(0, 81, 38);
const COMMAND_GOLD: Color32 = Color32::from_rgb(253, 216, 128);
const COMMAND_SELECTION: Color32 = Color32::from_rgb(24, 108, 65);

fn apply_command_theme(ui: &mut egui::Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(COMMAND_GOLD);
    visuals.panel_fill = COMMAND_BG;
    visuals.extreme_bg_color = COMMAND_BG;
    visuals.faint_bg_color = COMMAND_INPUT_BG;
    visuals.selection.bg_fill = COMMAND_SELECTION;
    visuals.selection.stroke = Stroke::new(1., COMMAND_GOLD);
    visuals.text_cursor.stroke = Stroke::new(1., COMMAND_GOLD);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = COMMAND_INPUT_BG;
        widget.weak_bg_fill = COMMAND_INPUT_BG;
        widget.fg_stroke = Stroke::new(1., COMMAND_GOLD);
        widget.bg_stroke = Stroke::new(1., COMMAND_SELECTION);
    }
    visuals.widgets.hovered.bg_fill = COMMAND_SELECTION;
    visuals.widgets.hovered.weak_bg_fill = COMMAND_SELECTION;
}

fn consume_command_shortcut(ctx: &egui::Context, shortcut: char) -> bool {
    let text = shortcut.to_string();
    ctx.input_mut(|input| {
        if input.events.iter().any(|event| {
            matches!(event, egui::Event::Paste(_) | egui::Event::Ime(_))
                || matches!(event, egui::Event::Text(value) if value.chars().count() != 1)
        }) {
            return false;
        }
        let pressed = input.events.iter().any(|event| {
            matches!(event,
            egui::Event::Key { key: pressed_key, pressed: true, modifiers, .. }
            if pressed_key.name().eq_ignore_ascii_case(&text) && !modifiers.ctrl && !modifiers.alt && !modifiers.command)
        });
        let hit = input.events.iter().any(|event| {
            matches!(event,
            egui::Event::Key { key: pressed_key, pressed: true, repeat: false, modifiers, .. }
            if pressed_key.name().eq_ignore_ascii_case(&text) && !modifiers.ctrl && !modifiers.alt && !modifiers.command)
        });
        if pressed {
            input.events.retain(|event| !matches!(event,
                egui::Event::Key { key: pressed_key, pressed: true, .. } if pressed_key.name().eq_ignore_ascii_case(&text))
                && !matches!(event, egui::Event::Text(value) if value.eq_ignore_ascii_case(&text)));
        }
        hit
    })
}
impl MatrixApp {
    /// Available command names matching the typed prefix, prefix matches first. The
    /// features catalog is the same list the icons bind, so a suggestion is always a
    /// command this build can actually dispatch.
    pub(super) fn command_suggestions_for(&self, query: &str) -> Vec<String> {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let names: std::collections::BTreeSet<String> = self
            .features
            .iter()
            .filter(|feature| Controller::availability(&feature.id).is_ok())
            .filter_map(|feature| feature.command.clone())
            .collect();
        let mut matches: Vec<(u8, String)> = names
            .into_iter()
            .filter_map(|name| {
                let lower = name.to_ascii_lowercase();
                let rank = if lower == query {
                    0
                } else if lower.starts_with(&query) {
                    1
                } else if lower.contains(&query) {
                    2
                } else {
                    return None;
                };
                Some((rank, name))
            })
            .collect();
        matches.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.len().cmp(&right.1.len()))
                .then_with(|| {
                    left.1
                        .to_ascii_lowercase()
                        .cmp(&right.1.to_ascii_lowercase())
                })
        });
        matches.truncate(8);
        matches.into_iter().map(|(_, name)| name).collect()
    }

    fn accept_command_suggestion(&mut self, command: &str) {
        self.transcript.set_draft(&format!("{command} "));
        self.focus_cmd = true;
        self.cmd_suggestions.clear();
        self.cmd_suggestion_index = 0;
        self.cmd_suggestions_query.clear();
    }

    pub(super) fn command_ui(&mut self, ui: &mut egui::Ui) {
        apply_command_theme(ui);
        // One prompt for every invocation path (typed, menu, toolbar, MCP). The
        // controller owns the current stage text and its inline option signature;
        // refreshing here means a command that started without a feedback call
        // still names its options inside the live row instead of a chip row.
        let prompt = self.controller.prompt_with_options();
        if self.transcript.prompt() != prompt {
            self.transcript.set_prompt(&prompt);
            // The prompt grew or shrank, so the editor's stored character index
            // no longer points at the draft end. Land the caret there again.
            self.focus_cmd = true;
        }
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
        let draft = self.transcript.draft().to_owned();
        let query = draft.trim().to_owned();
        let suggestions_visible = focused
            && !popup_open
            && !composing
            && !query.is_empty()
            && !draft.ends_with(' ')
            && !query.chars().any(char::is_whitespace)
            && self.controller.command_options().is_empty();
        if suggestions_visible {
            if query != self.cmd_suggestions_query {
                self.cmd_suggestions_query = query.clone();
                self.cmd_suggestion_index = 0;
                self.cmd_suggestions = self.command_suggestions_for(&query);
            } else if self.cmd_suggestion_index >= self.cmd_suggestions.len() {
                self.cmd_suggestion_index = 0;
            }
        } else {
            self.cmd_suggestions.clear();
            self.cmd_suggestion_index = 0;
            self.cmd_suggestions_query.clear();
        }
        let suggestions_open = suggestions_visible && !self.cmd_suggestions.is_empty();
        let enter =
            focused && !composing && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
        if focused && !composing && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp))
        {
            if suggestions_open {
                let count = self.cmd_suggestions.len();
                self.cmd_suggestion_index = (self.cmd_suggestion_index + count - 1) % count;
            } else {
                self.transcript.recall_up();
                self.focus_cmd = true;
            }
        }
        if focused
            && !composing
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown))
        {
            if suggestions_open {
                self.cmd_suggestion_index =
                    (self.cmd_suggestion_index + 1) % self.cmd_suggestions.len();
            } else {
                self.transcript.recall_down();
                self.focus_cmd = true;
            }
        }
        if suggestions_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Tab)) {
            let selected = self.cmd_suggestions[self.cmd_suggestion_index].clone();
            self.accept_command_suggestion(&selected);
        }
        // Delete removes the selection when the editor holds no draft. The
        // command line normally keeps focus, so the global viewport handler
        // cannot see this key first.
        if focused
            && !composing
            && self.transcript.draft().is_empty()
            && !self.controller.is_running()
            && !self.controller.selected().is_empty()
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete))
        {
            let ids: Vec<_> = self.controller.selected().iter().copied().collect();
            let result = self.controller.delete_objects(&ids).map(|_| ());
            self.feedback(result);
        }
        // One accelerator path for every owner: the declared snapshot decides
        // which letter is advertised; typed text never bypasses the resolver.
        if !composing
            && !popup_open
            && (focused || ctx.memory(|m| m.focused().is_none()))
            && self.transcript.draft().is_empty()
        {
            for option in self.controller.command_options() {
                if let Some(shortcut) = option.shortcut
                    && consume_command_shortcut(&ctx, shortcut)
                {
                    self.submit_command_option(option.id);
                    break;
                }
            }
        }
        if enter {
            if suggestions_open {
                let selected = self.cmd_suggestions[self.cmd_suggestion_index].clone();
                if selected.eq_ignore_ascii_case(&query) {
                    self.submit_draft();
                } else {
                    self.accept_command_suggestion(&selected);
                }
            } else {
                self.submit_draft();
            }
        } else if !focused
            && !popup_open
            && self.transcript.draft().is_empty()
            && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space))
        {
            // Space repeats when the command line is idle, exactly like an empty Enter; while the editor has
            // focus Space is ordinary text and never repeats (OM9-L3-03-10-006).
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
        // The prompt names every option inline, so an idle prompt may wrap to more
        // rows; a draft keeps the single pinned row and scrolls horizontally like
        // the Matrix command line. Measuring the prompt before the history area
        // lets the panel reserve the exact height in the same frame.
        let draft_empty = self.transcript.draft().is_empty();
        let prompt_wrap = if draft_empty {
            (area.width() - 2.).max(1.)
        } else {
            f32::INFINITY
        };
        let live_height = if draft_empty {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = prompt_wrap;
            job.append(
                self.transcript.prompt(),
                0.,
                egui::TextFormat {
                    font_id: font.clone(),
                    ..Default::default()
                },
            );
            let measured = ui.fonts_mut(|f| f.layout_job(job)).size().y + 2.;
            measured.clamp(
                row_height,
                (area.height() - row_height - 2.).max(row_height),
            )
        } else {
            row_height
        };
        let history_height = (area.height() - live_height - 2.).max(1.);
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
                    color: COMMAND_GOLD,
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
                    .text_color(COMMAND_GOLD)
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
        let live_prompt = &live[..live.len() - self.transcript.draft().len()];
        let options = self.controller.command_options();
        let mut search_from = 0;
        let inline_options = !options.is_empty()
            && options.iter().all(|option| {
                if let Some(at) = live_prompt[search_from..].find(option.label.as_str()) {
                    search_from += at + option.label.len();
                    true
                } else {
                    false
                }
            });
        let mut live_job = egui::text::LayoutJob::default();
        let mut offset = 0;
        for option in options.iter().filter(|_| inline_options) {
            let label = &option.label;
            if let Some(relative) = live_prompt[offset..].find(label.as_str()) {
                let byte = offset + relative;
                live_job.append(
                    &live[offset..byte],
                    0.0,
                    egui::TextFormat {
                        font_id: font.clone(),
                        color: COMMAND_GOLD,
                        ..Default::default()
                    },
                );
                let underline = option
                    .shortcut
                    .and_then(|shortcut| {
                        label.char_indices().find_map(|(index, ch)| {
                            ch.eq_ignore_ascii_case(&shortcut).then_some(byte + index)
                        })
                    })
                    .unwrap_or(byte);
                if underline > byte {
                    live_job.append(
                        &live[byte..underline],
                        0.0,
                        egui::TextFormat {
                            font_id: font.clone(),
                            color: COMMAND_GOLD,
                            ..Default::default()
                        },
                    );
                }
                let next = underline + live[underline..].chars().next().map_or(0, char::len_utf8);
                live_job.append(
                    &live[underline..next],
                    0.0,
                    egui::TextFormat {
                        font_id: font.clone(),
                        color: COMMAND_GOLD,
                        underline: Stroke::new(1.0, COMMAND_GOLD),
                        ..Default::default()
                    },
                );
                live_job.append(
                    &live[next..byte + label.len()],
                    0.0,
                    egui::TextFormat {
                        font_id: font.clone(),
                        color: COMMAND_GOLD,
                        ..Default::default()
                    },
                );
                offset = byte + label.len();
            }
        }
        live_job.append(
            &live[offset..],
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color: COMMAND_GOLD,
                ..Default::default()
            },
        );
        live_job.wrap.max_width = if self.transcript.draft().is_empty() {
            (area.width() - 2.).max(1.)
        } else {
            f32::INFINITY
        };
        let live_galley = ui.fonts_mut(|f| f.layout_job(live_job));
        let rect = Rect::from_min_size(
            egui::pos2(area.left(), area.bottom() - live_height),
            Vec2::new(area.width(), live_height),
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
        painter.rect_filled(rect, 0, COMMAND_INPUT_BG);
        painter.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1., COMMAND_GOLD.gamma_multiply(0.45)),
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
        // Clickable underlined polyline option shortcuts drawn over the prompt.
        self.cmd_token_rects.clear();
        if inline_options && !popup_open && !composing && self.transcript.draft().is_empty() {
            let mut option_offset = 0;
            for option in &options {
                let label = &option.label;
                if let Some(relative) = live_prompt[option_offset..].find(label.as_str()) {
                    let byte = option_offset + relative;
                    option_offset = byte + label.len();
                    let chars = live[..byte].chars().count();
                    let start = live_galley.pos_from_cursor(CCursor::new(chars));
                    let end =
                        live_galley.pos_from_cursor(CCursor::new(chars + label.chars().count()));
                    // A label can straddle a wrap point; its click span then ends at
                    // the right edge of the row it starts on.
                    let (right, bottom) = if (end.top() - start.top()).abs() < 0.5 {
                        (end.right(), end.bottom())
                    } else {
                        (live_galley.rect.right(), start.bottom())
                    };
                    let hit = Rect::from_min_max(
                        origin + Vec2::new(start.left(), start.top()),
                        origin + Vec2::new(right, bottom),
                    )
                    .intersect(rect);
                    self.cmd_token_rects
                        .insert(option.id.name().to_owned(), hit);
                    let response = ui.interact(
                        hit,
                        id.with(("command-shortcut", option.id)),
                        Sense::click(),
                    );
                    if response.clicked() {
                        self.submit_command_option(option.id);
                    }
                }
            }
        }
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
                        origin + Vec2::new(a.left(), a.top()),
                        origin + Vec2::new(b.left(), b.bottom()),
                    ),
                    0,
                    COMMAND_SELECTION,
                );
                if self.cmd_ime_composing {
                    painter.line_segment(
                        [
                            origin + Vec2::new(a.left(), b.bottom()),
                            origin + Vec2::new(b.left(), b.bottom()),
                        ],
                        Stroke::new(1., COMMAND_GOLD),
                    );
                }
            }
            if range.primary.index.0 >= history_chars {
                let caret = live_galley
                    .pos_from_cursor(CCursor::new(range.primary.index.0 - history_chars))
                    .translate(origin.to_vec2());
                painter.line_segment(
                    [caret.left_top(), caret.left_bottom()],
                    Stroke::new(1., COMMAND_GOLD),
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
        painter.galley(origin, Arc::clone(&live_galley), COMMAND_GOLD);
        if suggestions_open && focused && !self.cmd_suggestions.is_empty() {
            let suggestions = self.cmd_suggestions.clone();
            let selected = self.cmd_suggestion_index.min(suggestions.len() - 1);
            let line = ui.fonts_mut(|f| f.row_height(&font)) + 6.;
            let height = line * suggestions.len() as f32 + 8.;
            let content = ui.ctx().content_rect();
            // The list drops below the command row like the Matrix8 autocomplete; the
            // above-the-row fallback exists only when the row itself sits near the
            // bottom edge of the window.
            let below = rect.bottom() + 2.;
            let top = if below + height <= content.bottom() {
                below
            } else {
                (rect.top() - height).max(content.top())
            }
            .clamp(
                content.top(),
                (content.bottom() - height).max(content.top()),
            );
            let mut picked = None;
            let shown = egui::Area::new(id.with("command-suggestions"))
                .fixed_pos(egui::pos2(rect.left(), top))
                .order(egui::Order::Foreground)
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(COMMAND_BG)
                        .stroke(Stroke::new(1., COMMAND_GOLD.gamma_multiply(0.6)))
                        .show(ui, |ui| {
                            ui.set_min_width(rect.width().clamp(260., 420.));
                            for (index, name) in suggestions.iter().enumerate() {
                                let response = ui.selectable_label(index == selected, name);
                                if response.clicked() {
                                    picked = Some(name.clone());
                                }
                            }
                        });
                });
            self.cmd_suggestions_rect = Some(shown.response.rect);
            if let Some(name) = picked {
                self.accept_command_suggestion(&name);
            }
        } else {
            self.cmd_suggestions_rect = None;
        }
        state.store(&ctx, id);
        ui.allocate_rect(rect, Sense::hover());
    }
}
