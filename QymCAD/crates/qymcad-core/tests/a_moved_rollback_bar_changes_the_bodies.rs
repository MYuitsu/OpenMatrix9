//! Two states of a document that differ only in where the rollback bar stands build different bodies.
use qymcad_core::geom::Point2;
use qymcad_core::model::Project;

/// Reported behaviour, found by undoing and redoing "roll the timeline back here" on a sample: nothing was rebuilt
/// after either, and the bodies kept the meshes and face names of the other state. The recipes of the nodes are the
/// same on both sides of the bar; what the bar moved is which of them are built.
#[test]
fn bodies_count_as_changed_when_only_the_rollback_bar_moved() {
    let mut p = Project::default();
    p.new_document();
    let sid = p.add_line_sketch("s", vec![Point2::new(0.0, 0.0), Point2::new(10.0, 0.0), Point2::new(10.0, 10.0), Point2::new(0.0, 10.0)], true);
    p.add_sketch_node(sid, "Sketch");
    let body = p.add_extrude(sid, 5.0);
    let mut rolled = p.clone();
    rolled.set_rollback(Some(1));
    assert!(rolled.changed_bodies_vs(&p).contains(&body), "the bar moved above the extrusion and its body did not count as changed");
    assert!(p.changed_bodies_vs(&rolled).contains(&body), "the bar moved back below the extrusion and its body did not count as changed");
    assert!(p.changed_bodies_vs(&p.clone()).is_empty(), "the same state counted bodies as changed");
}
