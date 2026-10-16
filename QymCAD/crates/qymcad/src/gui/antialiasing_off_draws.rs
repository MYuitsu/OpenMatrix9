//! ANTIALIASING SWITCHED OFF DRAWS THE BODIES, not a crash.
//!
//! Reported behaviour (issue #2): with "Antialiasing" set to Off the program closed on start - "The resolve source,
//! color attachment at index 0's resolve texture view, must be multi-sampled (has 1 samples) while the resolve
//! destination must not be multisampled (has 1 samples)". The offscreen pass always drew into a multisample target
//! and resolved it into the colour the window shows, and a target of one sample cannot be resolved.
//!
//! Drawn through the card, by the same pipeline the window uses, with one sample per pixel and with four: the body
//! must cover the same pixels in both.
#[cfg(test)]
mod tests {
    use crate::gui::App;

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(400.0, 300.0))
    }

    /// A part holding a 40 x 30 x 10 block, the view on it from the usual three-quarter view.
    fn a_block() -> App {
        let mut app = App::default();
        let root = app.project.root;
        app.project.set_active_component(Some(root));
        let part = app.project.add_part("Block");
        app.enter_component(part);
        app.project.add_box(40.0, 30.0, 10.0);
        qymcad_ui_state::regenerate_now(&mut app.rebuild_ctx());
        assert!(!app.project.bodies.is_empty(), "setup: the block was not built");
        app.viewing.cam.target = [20.0, 15.0, 5.0];
        app.viewing.cam.scale = 5.0;
        app.viewing.cam.init = true;
        app.viewing.mode_3d = true;
        app
    }

    #[test]
    fn a_block_is_drawn_with_antialiasing_off() {
        let app = a_block();
        let Some(four) = crate::gui::gpu_shot::eyes::shot_with_samples(&app.painting(), rect(), 4) else {
            eprintln!("PASSED OVER: no graphics device to draw with");
            return;
        };
        let one = crate::gui::gpu_shot::eyes::shot_with_samples(&app.painting(), rect(), 1).expect("the card that drew with four samples draws with one");
        let covered = |img: &egui::ColorImage| img.pixels.iter().filter(|p| p.a() > 128).count();
        let (c1, c4) = (covered(&one), covered(&four));
        assert!(c4 > 1000, "setup: the block covers {c4} pixels with four samples");
        // the edges are the only pixels the two may disagree about
        let differ = one.pixels.iter().zip(&four.pixels).filter(|(a, b)| (a.a() > 128) != (b.a() > 128)).count();
        assert!(differ * 20 < c4, "with antialiasing off the block covers {c1} pixels and with four samples {c4}, {differ} of them differently");
    }
}
