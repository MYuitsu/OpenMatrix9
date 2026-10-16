//! A PLY THAT COLOURS ITS FACES COMES IN COLOURED, through the door a person opens it by: the part in the colour most of
//! its triangles have, the rest drawn in theirs. The owner's print head in PLY colours every one of its 465 900 faces.
//!
//! Three flat squares facing up: the first green, the other two red - so the part is red, and green is on the picture only
//! where the first square is. The colour stands before the corners, as the owner's file writes it.
#[cfg(test)]
mod tests {
    use crate::gui::a_component_stepped_into_is_not_lit::tests::calm;
    use crate::gui::import_door::tests::{answer, frame, key, running, settle};
    use qymcad_ui_state::Want;

    #[test]
    fn a_coloured_ply_comes_in_coloured() {
        let dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/ply-probe"));
        std::fs::create_dir_all(&dir).expect("a folder for the check");
        let path = dir.join("squares.ply");
        let mut text = String::from("ply\nformat ascii 1.0\nelement vertex 8\nproperty float x\nproperty float y\nproperty float z\nelement face 3\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nproperty list uchar int vertex_indices\nend_header\n");
        for j in 0..2 {
            for i in 0..4 {
                text.push_str(&format!("{} {} 0\n", 10 * i, 10 * j));
            }
        }
        text.push_str("26 204 26 255 4 0 1 5 4\n204 26 26 255 4 1 2 6 5\n204 26 26 255 4 2 3 7 6\n");
        std::fs::write(&path, text).expect("written");
        let (mut app, ctx) = running();
        answer(&mut app, &ctx, Want::Anything, &path.to_string_lossy());
        settle(&mut app, &ctx);
        calm(&mut app, &ctx);
        let _ = frame(&mut app, &ctx, key(egui::Key::Enter)); // the window about the scale, where it came up
        let _ = frame(&mut app, &ctx, Vec::new());
        let body = app.project.bodies.iter().rev().find(|b| b.visible && !b.mesh.tris.is_empty()).map(|b| b.id).expect("the squares came in");
        let mi = app.project.mesh_index(body).expect("a body");
        assert_eq!(app.project.mesh_color(mi), [204, 26, 26], "the part does not come in in the colour most of its triangles have");
        app.viewing.mode_3d = true;
        app.viewing.cam.init = true;
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(700.0, 600.0));
        crate::gui::fit3d(&mut app.viewing.cam, &app.project, rect);
        let basis = app.viewing.cam.basis();
        let img = qymcad_render::rasterize_3d(&app.painting(), rect, &basis, 1.0, 1.0).expect("a raster");
        // green standing well above the other two, as the painters give it (see `a_face_of_its_own_colour_is_drawn`)
        let green = img.pixels.iter().filter(|c| c.g() as i32 > c.r() as i32 + 60 && c.g() as i32 > c.b() as i32 + 60).count();
        assert!(green > 200, "the green square is not drawn green: {green} green pixels");
    }
}
