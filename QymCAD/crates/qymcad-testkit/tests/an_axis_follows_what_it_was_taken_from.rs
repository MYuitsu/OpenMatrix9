//! A DATUM AXIS RUNS WHERE IT WAS TAKEN: along the edge, on the axis of the round face, through the two points.
//!
//! Reported behaviour: all three ways gave the world Z through the origin while the program said the axis was taken
//! - a cylinder standing at (25, 0) gave an axis at (0, 0).
use qymcad_core::geom::Point2;
use qymcad_core::model::Project;

/// A 40 x 30 x 10 block in a part. Returns (project, body).
fn block() -> (Project, u64) {
    let mut p = Project::default();
    p.new_document();
    let sid = p.add_line_sketch("Sketch 1", vec![Point2::new(0.0, 0.0), Point2::new(40.0, 0.0), Point2::new(40.0, 30.0), Point2::new(0.0, 30.0)], true);
    let si = p.sketch_index(sid).unwrap();
    p.regen_sketch(si);
    if let Some(o) = p.sketch_owner(sid) {
        p.set_active_component(Some(o));
    }
    p.add_sketch_node(sid, "Sketch 1");
    let closed: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let body = p.add_extrude_multi(sid, closed, 10.0, qymcad_core::feature::Reach::Forward, 0.0, vec![]);
    qymcad_testkit::regenerate(&mut p);
    (p, body)
}

#[test]
fn an_axis_taken_from_an_edge_runs_along_it() {
    let (mut p, body) = block();
    let edge = p.regen_edges[&body].iter().find(|e| (e.a[2] - 10.0).abs() < 1e-6 && (e.b[2] - 10.0).abs() < 1e-6 && e.mid[1].abs() < 1e-6).map(|e| e.id).expect("the top front edge");
    let (_, shapes) = qymcad_testkit::regenerate(&mut p);
    // as the window does it: the axis is added and only what is marked is rebuilt, on the live shapes in hand
    let ax = p.add_axis_from_edge(body, edge);
    qymcad_testkit::regenerate_dirty_with_shapes(&mut p, shapes);
    let a = p.datum_axes.iter().find(|d| d.id == ax).expect("the axis");
    let (o, d) = (a.origin(), a.dir());
    assert!(d[0].abs() > 0.999, "the axis along the front top edge runs {d:?}");
    assert!((o[2] - 10.0).abs() < 1e-6 && o[1].abs() < 1e-6, "the axis does not lie on the edge: it goes through {o:?}");
}

/// AN AXIS TAKEN FROM SOMETHING ASKS FOR THE PASS THAT RESOLVES IT: its node comes in marked. A window rebuilds only
/// what is marked, and a clean node left the axis at its default, the world Z through the origin.
#[test]
fn an_axis_taken_from_an_edge_is_marked_for_the_rebuild() {
    let (mut p, body) = block();
    let edge = p.regen_edges[&body].first().map(|e| e.id).expect("an edge of the block");
    let ax = p.add_axis_from_edge(body, edge);
    assert!(p.timeline.iter().any(|n| n.id == ax && n.dirty), "the axis came in clean and nothing will resolve it");
    let manual = p.add_axis_manual([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    assert!(p.timeline.iter().any(|n| n.id == manual && !n.dirty), "an axis given by its numbers has nothing to resolve");
}
