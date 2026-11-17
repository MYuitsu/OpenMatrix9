// SPDX-License-Identifier: LGPL-2.1-or-later
//! Load installed Windows UI fonts once, retaining egui's portable fallbacks.
//! Font files remain OS resources; they are not copied or redistributed.
use eframe::egui::FontData;
use eframe::egui::{self, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

const SEMIBOLD: &str = "om9-ui-semibold";

pub(super) fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // The named family always exists, including on systems without Segoe UI.
    let proportional = fonts.families[&FontFamily::Proportional].clone();
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), proportional);
    #[cfg(windows)]
    if let Some(windows) = std::env::var_os("WINDIR") {
        let directory = std::path::PathBuf::from(windows).join("Fonts");
        for (file, name, family) in [
            ("segoeui.ttf", "om9-segoe-ui", FontFamily::Proportional),
            (
                "seguisb.ttf",
                "om9-segoe-semibold",
                FontFamily::Name(SEMIBOLD.into()),
            ),
            ("consola.ttf", "om9-consolas", FontFamily::Monospace),
        ] {
            if let Ok(bytes) = std::fs::read(directory.join(file)) {
                let mut data = FontData::from_owned(bytes);
                data.tweak.hinting = Some(true);
                data.tweak.hinting_target =
                    egui::epaint::text::HintingTarget::Smooth(egui::epaint::text::SmoothHinting {
                        light: true,
                        ..Default::default()
                    });
                fonts.font_data.insert(name.into(), Arc::new(data));
                fonts
                    .families
                    .get_mut(&family)
                    .unwrap()
                    .insert(0, name.into());
            }
        }
    }
    // Elsewhere an installed sans font with Vietnamese letters is a fallback after egui's own
    // (which has no `ọ`, `ữ` …), so Vietnamese labels never show as boxes.
    #[cfg(not(windows))]
    for path in [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("om9-vi-fallback".into(), Arc::new(FontData::from_owned(bytes)));
            for family in [FontFamily::Proportional, FontFamily::Name(SEMIBOLD.into()), FontFamily::Monospace] {
                if let Some(list) = fonts.families.get_mut(&family) {
                    list.push("om9-vi-fallback".into());
                }
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

pub(super) fn semibold(ctx: &egui::Context, size: f32) -> FontId {
    let family = FontFamily::Name(SEMIBOLD.into());
    // Headless/widget hosts may configure fonts during their first frame;
    // egui applies that atlas change at the next frame boundary.
    if ctx.fonts_mut(|fonts| fonts.definitions().families.contains_key(&family)) {
        FontId::new(size, family)
    } else {
        FontId::proportional(size)
    }
}
