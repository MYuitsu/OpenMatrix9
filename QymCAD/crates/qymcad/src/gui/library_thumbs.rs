//! THE THUMBNAILS OF THE BUILT-IN LIBRARY, drawn again from the parts themselves by the same hand that draws the thumbnail
//! of a part a person saves to the library. Run by hand when the built-in parts change: the thumbnails are files of the
//! distribution, not something a check can compute.
#[cfg(test)]
mod tests {
    use super::super::App;

    fn parts(dir: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                parts(&p, out);
            } else if p.extension().is_some_and(|x| x == "qpart") {
                out.push(p.to_string_lossy().into_owned());
            }
        }
    }

    #[test]
    #[ignore = "writes the thumbnails of the built-in library into its files"]
    fn draw_the_library_thumbnails() {
        let mut all = Vec::new();
        parts(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../library/parts")), &mut all);
        assert!(!all.is_empty(), "the built-in library holds no part");
        for path in all {
            let loaded = qymcad_io::load_part(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let mut app = App { project: loaded.project.clone(), ..Default::default() };
            app.project.mark_all_dirty();
            qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
            let part = app.project.components.iter().find(|c| c.kind == qymcad_core::feature::ComponentKind::Part && !app.project.component_bodies(c.id).is_empty()).map(|c| c.id);
            let part = part.unwrap_or_else(|| panic!("{path}: no part with a body"));
            let img = crate::gui::render_scene::render_component_thumbnail(&app.draw_ctx(), part).unwrap_or_else(|| panic!("{path}: nothing to draw"));
            let png = crate::gui::color_image_to_png(&img).unwrap_or_else(|| panic!("{path}: no PNG"));
            qymcad_io::save_part(&loaded.project, &loaded.manifest, &loaded.faces, Some(&png), &path).unwrap_or_else(|e| panic!("{path}: {e}"));
            std::fs::write(format!("{}/../../target/look/thumb-{}.png", env!("CARGO_MANIFEST_DIR"), std::path::Path::new(&path).file_stem().unwrap_or_default().to_string_lossy()), &png).ok();
        }
    }
}
