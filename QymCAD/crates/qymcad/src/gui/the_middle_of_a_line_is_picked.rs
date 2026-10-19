//! THE MIDDLE OF A LINE TAKES THE CONSTRAINTS OF A POINT, through the window with the select tool: the pointer on the
//! middle of a line shows the triangle, and a click there picks the line, remembered as clicked at its middle. A
//! constraint that takes points reads it as its midpoint - a point held at the middle of the line: Vertical with the
//! centre of a circle stands the centre straight above the middle, and keeps it there when the line is dragged. Lines
//! alone stay lines: two lines clicked at their middles and Horizontal are two horizontal lines.
//!
//! Reported behaviour (#57): the middle of a line offered a snap while drawing, and a click there picked the line or an
//! end; a constraint from the middle needed a construction point put there by hand and tied by Midpoint first.
#[cfg(test)]
mod tests {
    use super::super::hand::Hand;
    use super::super::{App, Sel};
    use qymcad_core::feature::SketchPlane;
    use qymcad_core::model::{Constraint, EntityKind};

    /// A line from (0, 0) to (40, 0) and a circle about (25, 30), the select tool taken.
    fn a_line_and_a_circle() -> (App, usize) {
        let mut app = App::default();
        let si = app.create_sketch_on(SketchPlane::default());
        app.chosen.sel = Sel::Sketch(si);
        Hand::new(&mut app).sk_tool(1).click2d(0.0, 0.0).click2d(40.0, 0.0).key(egui::Key::Escape);
        Hand::new(&mut app).sk_tool(3).click2d(25.0, 30.0).click2d(30.0, 30.0).key(egui::Key::Escape);
        (app, si)
    }

    fn at(app: &App, si: usize, id: u64) -> (f64, f64) {
        app.project.sketches[si].points.iter().find(|q| q.id == id).map(|q| (q.x, q.y)).expect("a point")
    }

    fn centre(app: &App, si: usize) -> u64 {
        app.project.sketches[si].entities.iter().find_map(|e| if let EntityKind::Circle { center, .. } = e.kind { Some(center) } else { None }).expect("the circle")
    }

    fn line_ends(app: &App, si: usize) -> (u64, u64) {
        app.project.sketches[si].entities.iter().find_map(|e| if let EntityKind::Line { a, b } = e.kind { Some((a, b)) } else { None }).expect("the line")
    }

    #[test]
    fn the_middle_of_a_line_takes_a_vertical_with_the_centre_of_a_circle() {
        let (mut app, si) = a_line_and_a_circle();
        let c = centre(&app, si);
        let mut hand = Hand::new(&mut app);
        hand.sk_tool(0);
        hand.hover2d(20.0, 0.0);
        assert!(matches!(hand.app.snap_hint, Some((_, 3))), "the triangle of the middle does not show under the pointer");
        hand.click2d(20.0, 0.0);
        let picked: Vec<(u8, u64)> = hand.app.tools.sel_sk.items.clone();
        assert!(matches!(picked.as_slice(), [(1, _)]) && hand.app.tools.sel_sk.at_middle == vec![picked[0].1], "a click on the middle did not pick the line at its middle: {picked:?}");
        let centre_at = at(hand.app, si, c);
        hand.shift_click2d(centre_at.0, centre_at.1);
        Hand::new(&mut app).constraint(2);
        let mid = app.project.sketches[si].constraints.iter().find_map(|k| match *k {
            Constraint::Midpoint { p, .. } => Some(p),
            _ => None,
        });
        let mid = mid.expect("Vertical with a point did not take the middle of the line");
        let (m, k) = (at(&app, si, mid), at(&app, si, c));
        // the solve moves both the least it can, so the middle need not stay at 20: the centre stands above it
        assert!((m.0 - k.0).abs() < 1e-6, "Vertical did not stand the centre above the middle: middle {m:?}, centre {k:?}");
        // the line dragged by its far end: its middle moves, and the centre with it
        let (_, b) = line_ends(&app, si);
        let end = at(&app, si, b);
        Hand::new(&mut app).sk_tool(0).drag2d(end, (end.0 + 10.0, end.1));
        let (m, k) = (at(&app, si, mid), at(&app, si, c));
        let (a, b) = line_ends(&app, si);
        let (pa, pb) = (at(&app, si, a), at(&app, si, b));
        assert!((m.0 - (pa.0 + pb.0) / 2.0).abs() < 1e-6 && (m.0 - k.0).abs() < 1e-6, "after the drag: middle {m:?} of {pa:?}-{pb:?}, centre {k:?}");
    }

    #[test]
    fn a_click_on_the_line_off_its_middle_picks_the_line() {
        let (mut app, si) = a_line_and_a_circle();
        let mut hand = Hand::new(&mut app);
        hand.sk_tool(0);
        hand.hover2d(10.0, 0.0);
        assert!(!matches!(hand.app.snap_hint, Some((_, 3))), "setup: no triangle a quarter along the line");
        hand.click2d(10.0, 0.0);
        let picked = hand.app.tools.sel_sk.items.clone();
        assert!(matches!(picked.as_slice(), [(1, _)]), "a click on the line off its middle did not pick the line: {picked:?}");
        let _ = si;
    }

    #[test]
    fn two_lines_clicked_at_their_middles_are_made_horizontal_as_lines() {
        let mut app = App::default();
        let si = app.create_sketch_on(SketchPlane::default());
        app.chosen.sel = Sel::Sketch(si);
        Hand::new(&mut app).sk_tool(1).click2d(0.0, 0.0).click2d(40.0, 4.0).key(egui::Key::Escape).click2d(0.0, 20.0).click2d(40.0, 25.0).key(egui::Key::Escape);
        let mut hand = Hand::new(&mut app);
        hand.sk_tool(0);
        hand.hover2d(20.0, 2.0);
        hand.click2d(20.0, 2.0);
        hand.hover2d(20.0, 22.5);
        hand.shift_click2d(20.0, 22.5);
        assert_eq!(hand.app.tools.sel_sk.at_middle.len(), 2, "setup: both lines clicked at their middles: {:?}", hand.app.tools.sel_sk.items);
        Hand::new(&mut app).constraint(1);
        let s = &app.project.sketches[si];
        let h = s.constraints.iter().filter(|k| matches!(k, Constraint::Horizontal { .. })).count();
        let mids = s.constraints.iter().filter(|k| matches!(k, Constraint::Midpoint { .. })).count();
        assert!(h == 2 && mids == 0, "two lines and Horizontal: {h} Horizontal, {mids} midpoints: {:?}", s.constraints);
    }

    /// A LINE PICKED AT ITS MIDDLE SHOWS IT: a ring at the middle, of the snap's colour, stands while it is picked, so the
    /// middle is seen as what was taken; a line picked elsewhere shows none. Reported behaviour: the click at the middle
    /// lit the line as any click on it did, and nothing told the middle had been taken.
    #[test]
    fn a_line_picked_at_its_middle_shows_a_ring_there() {
        let (mut app, _) = a_line_and_a_circle();
        let mut hand = Hand::new(&mut app);
        hand.sk_tool(0);
        hand.hover2d(20.0, 0.0);
        hand.click2d(20.0, 0.0);
        hand.hover2d(20.0, 15.0);
        let middle = hand.on_screen2d((20.0, 0.0));
        let rings = hand.rings_drawn(MIDDLE_RING);
        assert!(rings.iter().any(|r| r.distance(middle) < 1.0), "no ring at the middle of the line picked there: {rings:?}");
        hand.key(egui::Key::Escape);
        hand.hover2d(10.0, 0.0);
        hand.click2d(10.0, 0.0);
        hand.hover2d(20.0, 15.0);
        let rings = hand.rings_drawn(MIDDLE_RING);
        assert!(!rings.iter().any(|r| r.distance(middle) < 1.0), "a ring at the middle of a line picked elsewhere: {rings:?}");
    }

    /// The radius of the ring that marks a line picked at its middle, px.
    const MIDDLE_RING: f32 = qymcad_render::MIDDLE_RING;
}
