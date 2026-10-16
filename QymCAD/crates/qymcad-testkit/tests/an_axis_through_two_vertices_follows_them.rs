//! AN AXIS THROUGH TWO VERTICES FOLLOWS THEM: a circular pattern of a block about the axis through the two corners of
//! its far upright edge turns half a turn about that edge; the block made longer, the edge moves and the pattern with
//! it. Reported behaviour: an axis picked through two vertices stayed where they had stood.
use qymcad_core::feature::FeatureKind;
use qymcad_core::model::Project;

#[test]
fn an_axis_through_two_vertices_follows_them() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let bounds = |p: &Project, b| p.mesh_index(b).and_then(|i| p.bodies[i].mesh.bounds()).expect("a built body");
    let bb = bounds(&p, block);
    // the upright edge at the far corner, and a datum point on each of its ends
    let edge = p.regen_edges[&block]
        .iter()
        .find(|e| (e.a[0] - bb.max.x).abs() < 1e-6 && (e.b[0] - bb.max.x).abs() < 1e-6 && (e.a[1] - bb.max.y).abs() < 1e-6 && (e.b[1] - bb.max.y).abs() < 1e-6)
        .cloned()
        .expect("the far upright edge");
    let pa = p.add_point_at_vertex(edge.a, block, edge.id, false);
    let pb = p.add_point_at_vertex(edge.b, block, edge.id, true);
    let axis = p.add_axis_two_points(pa, pb);
    let ring = p.add_circular_array_axis(block, 2, 360.0, axis);
    let _ = qymcad_testkit::regenerate(&mut p);
    let far = |p: &Project| {
        let (src, out) = (bounds(p, block), bounds(p, ring));
        (out.max.x, 2.0 * src.max.x - src.min.x)
    };
    let (got, want) = far(&p);
    assert!((got - want).abs() < 1e-6, "half a turn about the far edge reaches x = {want}, the pattern reaches {got}");
    // the block made 10 longer: its far edge moves 10 on, and the turned copy 20
    if let Some(n) = p.timeline.iter_mut().find(|n| n.kind.body() == Some(block)) {
        if let FeatureKind::Box3 { dx, .. } = &mut n.kind {
            *dx = 50.0;
        }
        n.dirty = true;
    }
    let _ = qymcad_testkit::regenerate(&mut p);
    let (got, want) = far(&p);
    assert!((got - want).abs() < 1e-6, "the block made longer: the pattern about its far edge should reach x = {want}, it reaches {got}");
}
