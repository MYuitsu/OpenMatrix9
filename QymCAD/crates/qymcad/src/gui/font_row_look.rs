//! A LOOK AT THE LIST OF FONTS: the rows are drawn into a picture and examined.
//!
//! Reported behaviour, with a screenshot: "look at how the font is drawn" - the letters came out torn and the
//! holes of `o` and `e` were filled in. The rows were painted one convex polygon per loop, and a glyph is not
//! convex. The picture is how that is checked: an assertion cannot see a ragged letter.
#[cfg(test)]
mod tests {
    use super::super::help_raster::shot_ui;

    #[test]
    #[ignore = "a look, not a check: writes a picture to target/"]
    fn the_rows_of_the_font_list() {
        let mut cache = qymcad_ui_state::FontCache::default();
        let faces = qymcad_ui_state::installed_fonts();
        assert!(!faces.is_empty(), "no fonts on this machine");
        let shown: Vec<qymcad_ui_state::FontFace> = faces.into_iter().take(14).collect();
        let pal = qymcad_scheme::dark();
        let img = shot_ui([560, 400], pal.viewport_bg(), |ui| {
            for f in &shown {
                qymcad_ui_state::font_row(ui, &mut cache, f, 26.0);
            }
        });
        let png = crate::gui::color_image_to_png(&img).expect("PNG");
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/font-rows.png");
        std::fs::write(&out, png).expect("writing");
        eprintln!("SNAPSHOT {}", out.display());
    }

    /// THE SAME ROWS, LARGE, so the shape of every letter can be looked at.
    #[test]
    #[ignore = "a look, not a check: writes a picture to target/"]
    fn one_row_up_close() {
        let mut cache = qymcad_ui_state::FontCache::default();
        let faces = qymcad_ui_state::installed_fonts();
        let shown: Vec<qymcad_ui_state::FontFace> = faces.into_iter().take(4).collect();
        let pal = qymcad_scheme::dark();
        let img = shot_ui([900, 400], pal.viewport_bg(), |ui| {
            for f in &shown {
                qymcad_ui_state::font_row(ui, &mut cache, f, 90.0);
            }
        });
        let png = crate::gui::color_image_to_png(&img).expect("PNG");
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/font-row-big.png");
        std::fs::write(&out, png).expect("writing");
        eprintln!("SNAPSHOT {}", out.display());
    }
}
