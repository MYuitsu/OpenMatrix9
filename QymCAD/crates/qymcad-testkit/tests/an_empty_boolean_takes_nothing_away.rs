//! AN INTERSECTION THAT MAKES NOTHING TAKES NOTHING AWAY.
//!
//! Reported behaviour: the intersection of two pieces that only touch said "an empty body" and the part lost half its
//! material - 8000 mm^3 became 4000. A failed union or intersection was passed through as a copy of its first input,
//! and consumed both.
use qymcad_core::geom::Point2;
use qymcad_core::model::Project;

/// A 20 x 20 x 10 box from `x0`, as a body of its own. Returns the body.
fn box_at(p: &mut Project, x0: f64) -> u64 {
    let sid = p.add_line_sketch("S", vec![Point2::new(x0, 0.0), Point2::new(x0 + 20.0, 0.0), Point2::new(x0 + 20.0, 20.0), Point2::new(x0, 20.0)], true);
    let si = p.sketch_index(sid).unwrap();
    p.regen_sketch(si);
    p.add_sketch_node(sid, "S");
    let closed: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    p.add_extrude_multi(sid, closed, 10.0, qymcad_core::feature::Reach::Forward, 0.0, vec![])
}

#[test]
fn an_intersection_of_two_touching_boxes_leaves_both() {
    let mut p = Project::default();
    p.new_document();
    let (a, b) = (box_at(&mut p, 0.0), box_at(&mut p, 20.0));
    qymcad_testkit::regenerate(&mut p);
    let meet = p.add_body_boolean(a, b, 2);
    let (rep, _) = qymcad_testkit::regenerate(&mut p);
    assert!(rep.errors.iter().any(|(id, _)| *id == meet), "an intersection of two boxes that only touch was not refused: {:?}", rep.errors);
    let gone = p.consumed_bodies();
    assert!(!gone.contains(&a) && !gone.contains(&b), "the refused intersection took its inputs out of sight: {gone:?}");
    let seen: f64 = p.bodies.iter().filter(|x| !gone.contains(&x.id) && !x.mesh.tris.is_empty()).map(|x| x.mesh.volume()).sum();
    assert!((seen - 8000.0).abs() < 1.0, "the two boxes hold 8000 and what is seen holds {seen}");
}
