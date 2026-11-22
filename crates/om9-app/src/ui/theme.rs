// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared palette for egui panels, dialogs, anchored menus and their detached children.
use eframe::egui::{self, Color32, FontId, Stroke};

pub(super) const DOCK: Color32 = super::command::COMMAND_BG;
pub(super) const BODY: Color32 = Color32::from_rgb(23, 51, 40);
pub(super) const HEADER: Color32 = Color32::from_rgb(0, 81, 38);
pub(super) const EDGE: Color32 = Color32::from_rgb(71, 124, 94);
pub(super) const INK: Color32 = Color32::from_rgb(255, 244, 214);
pub(super) const MUTED: Color32 = Color32::from_rgb(190, 209, 195);
pub(super) const TITLE: Color32 = Color32::from_rgb(253, 216, 128);
/// Established active marker shared by the expanded and collapsed Snap controls.
pub(super) const SNAP_ON: Color32 = Color32::from_rgb(90, 190, 245);
pub(super) const ACCENT: Color32 = Color32::from_rgb(95, 224, 184);
pub(super) const SELECTED: Color32 = Color32::from_rgb(31, 104, 75);
pub(super) const HOVER: Color32 = Color32::from_rgb(51, 91, 71);
pub(super) const ERROR: Color32 = Color32::from_rgb(255, 168, 158);
pub(super) const TOOL_FIELD: Color32 = Color32::from_rgb(13, 36, 26);
pub(super) const TOOL_TITLE_SIZE: f32 = 15.;

pub(super) fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
    style.spacing.item_spacing = egui::vec2(2., 2.);
    style.spacing.button_padding = egui::vec2(3., 1.);
    style.spacing.interact_size = egui::vec2(18., 17.);
    for (key, font) in [
        (egui::TextStyle::Body, FontId::proportional(11.)),
        (egui::TextStyle::Button, FontId::proportional(11.)),
        (egui::TextStyle::Small, FontId::proportional(10.)),
        (egui::TextStyle::Monospace, FontId::monospace(12.)),
    ] {
        style.text_styles.insert(key, font);
    }
    let mut v = egui::Visuals::dark();
    v.window_fill = BODY;
    v.panel_fill = DOCK;
    v.extreme_bg_color = TOOL_FIELD;
    v.faint_bg_color = Color32::from_white_alpha(12);
    v.override_text_color = Some(INK);
    v.weak_text_color = Some(MUTED);
    v.warn_fg_color = TITLE;
    v.error_fg_color = ERROR;
    v.hyperlink_color = ACCENT;
    v.disabled_alpha = 0.62;
    v.window_corner_radius = 6.into();
    v.window_stroke = Stroke::new(1., EDGE);
    v.selection.bg_fill = SELECTED;
    v.selection.stroke = Stroke::new(1., ACCENT);
    for (widget, fill, border) in [
        (&mut v.widgets.noninteractive, BODY, EDGE),
        (&mut v.widgets.inactive, Color32::from_rgb(38, 68, 52), EDGE),
        (&mut v.widgets.hovered, HOVER, ACCENT),
        (&mut v.widgets.active, SELECTED, TITLE),
        (&mut v.widgets.open, SELECTED, ACCENT),
    ] {
        widget.bg_fill = fill;
        widget.weak_bg_fill = fill;
        widget.bg_stroke = Stroke::new(1., border);
        widget.fg_stroke = Stroke::new(1., INK);
        widget.corner_radius = 4.into();
    }
    v.text_options.font_hinting = true;
    v.text_options.subpixel_binning = true;
    style.visuals = v;
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.set_style_of(egui::Theme::Light, style);
}

pub(super) fn apply_button(label: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(label).color(INK).strong()).fill(SELECTED)
}

/// Readable controls for modeless tool panels in either dock and on the canvas.
pub(super) fn readable_tool_style(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    style.spacing.item_spacing = egui::vec2(5., 5.);
    style.spacing.button_padding = egui::vec2(6., 3.);
    style.spacing.interact_size.y = 23.;
    style.spacing.slider_width = 96.;
    for (key, size) in [
        (egui::TextStyle::Body, 13.),
        (egui::TextStyle::Button, 13.),
        (egui::TextStyle::Small, 12.),
        (egui::TextStyle::Monospace, 13.),
    ] {
        let font = if key == egui::TextStyle::Monospace {
            FontId::monospace(size)
        } else {
            FontId::proportional(size)
        };
        style.text_styles.insert(key, font);
    }
    style.visuals.selection.bg_fill = SELECTED;
    style.visuals.selection.stroke = Stroke::new(1.5, ACCENT);
}

pub(super) fn primary_button(label: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(label).strong().color(TOOL_FIELD))
        .fill(ACCENT)
        .stroke(Stroke::new(1., ACCENT))
        .min_size(egui::vec2(66., 26.))
}

pub(super) fn modal(ctx: &egui::Context, id: egui::Id) -> egui::Modal {
    egui::Modal::new(id).frame(egui::Frame::popup(&ctx.style_of(egui::Theme::Light)))
}

pub(super) fn menu(response: &egui::Response) -> egui::Popup<'_> {
    egui::Popup::menu(response).frame(egui::Frame::popup(
        &response.ctx.style_of(egui::Theme::Light),
    ))
}
