//! DELETING WHAT A FEATURE STANDS ON LEAVES THE FEATURE IN THE TIMELINE, RED WITH A REASON.
//!
//! Deleting the extrusion under a chamfer, or the sketch under an extrusion, used to take every node built on it
//! away in a cascade: the work above the deleted node was lost without a word. Now the node alone goes; what stood
//! on it stays in the timeline, rebuilds, fails and says why - the person repairs it or deletes it. The cascade is
//! still there when asked for by name (`*_with_dependents`).
use qymcad_core::geom::Point2;
use qymcad_core::model::Project;
use qymcad_core::refs::Ref;

/// A 60x40x12 plate with a chamfer on its front top edge. Returns (project, sketch, plate, chamfer).
fn chamfered_plate() -> (Project, u64, u64, u64) {
    let mut p = Project::default();
    p.new_document();
    let sid = p.add_line_sketch("Sketch 1", vec![Point2::new(0.0, 0.0), Point2::new(60.0, 0.0), Point2::new(60.0, 40.0), Point2::new(0.0, 40.0)], true);
    let si = p.sketch_index(sid).unwrap();
    p.regen_sketch(si);
    if let Some(o) = p.sketch_owner(sid) {
        p.set_active_component(Some(o));
    }
    p.add_sketch_node(sid, "Sketch 1");
    let closed: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let body = p.add_extrude_multi(sid, closed, 12.0, qymcad_core::feature::Reach::Forward, 0.0, vec![]);
    qymcad_testkit::regenerate(&mut p);
    let front = p.regen_edges[&body].iter().find(|e| (e.a[2] - 12.0).abs() < 1e-6 && (e.b[2] - 12.0).abs() < 1e-6 && e.mid[1].abs() < 1e-6).map(|e| e.id).expect("the top front edge");
    let cha = p.add_chamfer_ref(body, 2.0, Ref::picks(&[front]));
    let (rep, _) = qymcad_testkit::regenerate(&mut p);
    assert!(rep.errors.is_empty(), "setup - the chamfer: {:?}", rep.errors);
    (p, sid, body, cha)
}

#[test]
fn deleting_the_extrusion_leaves_the_chamfer_red() {
    let (mut p, _, body, cha) = chamfered_plate();
    p.delete_feature_op(body);
    assert!(p.timeline.iter().any(|n| n.id == cha), "the chamfer went with the extrusion under it");
    assert!(!p.timeline.iter().any(|n| n.id == body), "the extrusion was not deleted");
    qymcad_testkit::regenerate_dirty_with_shapes(&mut p, Default::default());
    assert_eq!(p.regen_errors.get(&cha), Some(&qymcad_core::errors::CoreError::SourceBodyDeleted), "the chamfer stands on nothing and is not red with that reason");
    assert!(!p.bodies.iter().any(|b| b.id == body), "the deleted extrusion's body is still in the document");
}

#[test]
fn deleting_the_sketch_leaves_the_extrusion_red() {
    let (mut p, sid, body, cha) = chamfered_plate();
    p.delete_sketch(sid);
    assert!(p.timeline.iter().any(|n| n.id == body) && p.timeline.iter().any(|n| n.id == cha), "the nodes on the sketch went with it: {:?}", p.timeline.iter().map(|n| &n.name).collect::<Vec<_>>());
    qymcad_testkit::regenerate_dirty_with_shapes(&mut p, Default::default());
    assert!(p.regen_errors.get(&body).is_some_and(|e| !e.to_string().is_empty()), "the extrusion stands on no sketch and is not red with a reason");
}

#[test]
fn asked_by_name_the_cascade_still_takes_the_dependents() {
    let (mut p, sid, body, cha) = chamfered_plate();
    p.delete_feature_with_dependents(body);
    assert!(!p.timeline.iter().any(|n| n.id == cha), "asked to take the dependents, the chamfer stayed");
    let (mut q, _, _, _) = chamfered_plate();
    q.delete_sketch_with_dependents(sid);
    assert!(!q.timeline.iter().any(|n| n.id == body), "asked to take the dependents, the extrusion stayed");
}

/// WHAT STANDS ON A RED NODE IS RED TOO, with its own reason: its source was not built. It used to drop out of
/// view with no word, green in the timeline.
#[test]
fn a_node_on_a_red_node_is_red_too() {
    let (mut p, _, body, cha) = chamfered_plate();
    let back = p.regen_edges[&cha].iter().find(|e| (e.a[2] - 12.0).abs() < 1e-6 && (e.b[2] - 12.0).abs() < 1e-6 && (e.mid[1] - 40.0).abs() < 1e-6).map(|e| e.id).expect("the top back edge");
    let fil = p.add_fillet_ref(cha, 2.0, Ref::picks(&[back]));
    let (rep, _) = qymcad_testkit::regenerate(&mut p);
    assert!(rep.errors.is_empty(), "setup - the fillet: {:?}", rep.errors);
    p.delete_feature_op(body);
    qymcad_testkit::regenerate_dirty_with_shapes(&mut p, Default::default());
    assert_eq!(p.regen_errors.get(&cha), Some(&qymcad_core::errors::CoreError::SourceBodyDeleted), "the chamfer on the deleted extrusion is not red");
    assert_eq!(p.regen_errors.get(&fil), Some(&qymcad_core::errors::CoreError::SourceBodyNotBuilt), "the fillet on the red chamfer says nothing");
}
