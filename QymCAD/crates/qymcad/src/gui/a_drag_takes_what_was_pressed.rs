//! A DRAG TAKES WHAT THE HAND PRESSED ON.
//!
//! The window calls a press a drag only once the pointer is more than 6 px from where it went down, and that is
//! the moment it looks for what to take. A point is taken within 8 px, so the hand's speed decided the grab: at
//! 1 px a frame the drag is decided at 7 px and the point is taken, at 3 px a frame - 180 px a second, an
//! unhurried hand - it is decided at 8.5 px, the point is out of reach, and the drag pans the view instead.
//! Measured through the frame: a free point pressed on and led 5 mm stayed where it was while the view went
//! to (-3.3, -3.3).
#[cfg(test)]
mod tests {
    use super::super::hand::Hand;
    use super::super::{App, Sel};
    use qymcad_core::feature::{Purpose, SketchPlane};

    /// A POINT PRESSED ON FOLLOWS THE HAND, and the view stays where it was.
    #[test]
    fn a_point_pressed_on_follows_the_hand() {
        let mut app = App::default();
        let si = app.create_sketch_on(SketchPlane::default());
        app.chosen.sel = Sel::Sketch(si);
        let line = app.project.add_line_entity(si, 10.0, 0.0, 30.0, 0.0, Purpose::Real);
        let end = app.project.entity_point_ids(si, &[line])[1];
        app.set.snap.on = false; // the grid does not pull the point off where the hand leaves it
        let mut hand = Hand::canvas(&mut app);
        let centre = hand.app.viewing.view.center;
        hand.drag2d((30.0, 0.0), (35.0, 5.0));
        let p = app.project.sketches[si].points.iter().find(|p| p.id == end).map(|p| (p.x, p.y)).expect("the end is there");
        assert!((p.0 - 35.0).hypot(p.1 - 5.0) < 0.5, "the end pressed on at (30, 0) and led to (35, 5) stands at {p:?}");
        assert_eq!(app.viewing.view.center, centre, "the drag moved the view instead of the point");
    }
}
