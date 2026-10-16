//! THE SCHEME HOLDS AGAINST THE SYSTEM'S THEME: the look a scheme gives the panels stays when the system says it is
//! light or dark.
//!
//! Reported behaviour (Windows 10): the chosen theme did not survive a restart. The settings kept it; the window did
//! not show it - the system's light theme came in on the first frames and the panels took the factory look of it.
use qymcad_ui_state::{apply_theme, SchemeUi, Settings};

/// One frame of `ctx`, the system saying its theme is `theme`.
fn frame(ctx: &egui::Context, theme: Option<egui::Theme>) {
    let raw = egui::RawInput { system_theme: theme, ..Default::default() };
    let _ = ctx.run_ui(raw, |_| {});
}

#[test]
fn a_dark_scheme_stays_when_the_system_turns_out_light() {
    let ctx = egui::Context::default();
    let mut scheme = SchemeUi::default();
    let set = Settings { scheme: qymcad_scheme::dracula().id.clone(), ..Settings::default() };
    frame(&ctx, None); // the first frame: the system has said nothing yet
    apply_theme(&mut scheme, &set, &ctx);
    let wanted = qymcad_scheme::visuals(&scheme.pal).panel_fill;
    frame(&ctx, Some(egui::Theme::Light)); // Windows tells its theme
    frame(&ctx, Some(egui::Theme::Light));
    assert_eq!(ctx.global_style().visuals.panel_fill, wanted, "the system's light theme took the panels from the chosen scheme");
}

#[test]
fn a_light_scheme_stays_when_the_system_is_dark() {
    let ctx = egui::Context::default();
    let mut scheme = SchemeUi::default();
    let set = Settings { scheme: qymcad_scheme::light().id.clone(), ..Settings::default() };
    frame(&ctx, Some(egui::Theme::Dark));
    apply_theme(&mut scheme, &set, &ctx);
    let wanted = qymcad_scheme::visuals(&scheme.pal).panel_fill;
    frame(&ctx, Some(egui::Theme::Dark));
    assert_eq!(ctx.global_style().visuals.panel_fill, wanted, "the system's dark theme took the panels from the chosen light scheme");
}
