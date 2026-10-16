//! A SKETCH ON A FACE OF A DELETED BASE GOES RED WITH THE REASON.
//!
//! Reported behaviour: the extrude a sketch stood on was deleted, and the sketch stood clean, with neither an error
//! nor a warning.
use qymcad_core::errors::CoreError;
use qymcad_core::feature::{FaceKey, SketchPlane};
use qymcad_core::model::Project;

#[test]
fn a_sketch_on_a_face_of_a_deleted_extrude_goes_red() {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("base");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "base");
    p.add_rect_entity(si, 0.0, 0.0, 20.0, 20.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let cid = p.sketches[si].contour_ids.iter().copied().find(|c| p.contour_profile_xy(*c).is_some()).unwrap();
    let e = p.add_extrude_multi(sid, vec![cid], 10.0, qymcad_core::feature::Reach::Forward, 0.0, vec![]);
    let cube = p.finish_base_body(e, 1);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = p.regen_faces.get(&cube).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9)).cloned().expect("the top face");
    let key = FaceKey { index: 0, centroid: [top.centroid.x, top.centroid.y, top.centroid.z], normal: top.normal, id: top.id };
    let s2 = p.new_sketch("on top");
    let sid2 = p.sketches[s2].id;
    p.sketches[s2].plane = SketchPlane::Face(cube, key);
    let node = p.add_sketch_node(sid2, "on top");
    let _ = qymcad_testkit::regenerate(&mut p);
    assert!(!p.regen_errors.contains_key(&node), "setup: the sketch on the top is red before anything is deleted: {:?}", p.regen_errors);
    let extrude = p.timeline.iter().find(|n| n.kind.bodies().contains(&cube)).map(|n| n.id).expect("the extrude's node");
    p.delete_feature_op(extrude);
    // the window rebuilds only when something is marked: a sketch left unmarked is never visited, and stays clean
    assert!(p.timeline.iter().any(|n| n.id == node && n.dirty), "the deletion left the sketch on the lost face unmarked, so nothing rebuilds it");
    // only what the deletion marked is rebuilt, as in the window
    let _ = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, Default::default());
    assert!(p.regen_errors.get(&node) == Some(&CoreError::SketchFaceGone), "the extrude under the sketch was deleted and the sketch stands with {:?}", p.regen_errors.get(&node));
}

/// A SKETCH ON A DELETED WORK PLANE STAYS, RED WITH THE REASON: the plane alone goes (decided 24.09), the sketch stands
/// where the plane stood and the body on it keeps its volume. Reported behaviour: deleting the plane took the sketches
/// on it and all they made away with it.
#[test]
fn a_sketch_on_a_deleted_work_plane_stays_red() {
    use qymcad_core::feature::BasePlane;
    let mut p = Project::default();
    p.new_document();
    let pl = p.add_offset_plane(BasePlane::XY, 20.0);
    let si = p.new_sketch("on the plane");
    let sid = p.sketches[si].id;
    p.sketches[si].plane = SketchPlane::Datum(pl);
    let node = p.add_sketch_node(sid, "on the plane");
    p.add_rect_entity(si, 0.0, 0.0, 20.0, 10.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let cid = p.sketches[si].contour_ids.iter().copied().find(|c| p.contour_profile_xy(*c).is_some()).unwrap();
    let e = p.add_extrude_multi(sid, vec![cid], 5.0, qymcad_core::feature::Reach::Forward, 0.0, vec![]);
    let body = p.finish_base_body(e, 1);
    let _ = qymcad_testkit::regenerate(&mut p);
    let volume = |p: &Project| p.mesh_index(body).map(|i| p.bodies[i].mesh.volume()).unwrap_or(0.0);
    let (v0, z0) = (volume(&p), p.mesh_index(body).and_then(|i| p.bodies[i].mesh.bounds()).map(|b| b.min.z).unwrap_or(0.0));
    assert!(p.delete_plane(pl), "the plane goes");
    assert!(p.sketch_index(sid).is_some() && p.timeline.iter().any(|n| n.kind.body() == Some(body)), "the sketch or its body went with the plane");
    let _ = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, Default::default());
    assert!(p.regen_errors.get(&node) == Some(&CoreError::SketchPlaneGone), "the sketch on the deleted plane stands with {:?}", p.regen_errors.get(&node));
    let z1 = p.mesh_index(body).and_then(|i| p.bodies[i].mesh.bounds()).map(|b| b.min.z).unwrap_or(0.0);
    assert!((volume(&p) - v0).abs() < 1e-6 && (z1 - z0).abs() < 1e-9, "the body moved or changed: {v0} at z {z0} became {} at z {z1}", volume(&p));
}
