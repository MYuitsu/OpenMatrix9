//! A MIRROR, OR A SPLIT, ABOUT A FACE IS ONE NODE OF THE TIMELINE: the plane is read off the face at every rebuild, so
//! it follows the face when the body changes, and no datum plane stands beside the node. A block 40 x 30 x 10
//! mirrored about its face x = +20 with the original kept is 80 long; made 50 long, the mirror follows to 100.
//!
//! Reported behaviour: Enter on a mirror, or a split, by a face laid a "Plane from face" node as well, and deleting the
//! extrusion took the mirror away and left the plane hanging.
use qymcad_core::feature::FeatureKind;
use qymcad_core::model::Project;

/// A block and the key of its face x = +dx / 2.
fn block(dx: f64) -> (Project, u64, qymcad_core::feature::FaceKey) {
    let mut p = Project::default();
    p.new_document();
    let b = p.add_box(dx, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let f = p.regen_faces.get(&b).and_then(|fs| fs.iter().find(|f| f.normal[0] > 0.99)).expect("the face x = +");
    let key = qymcad_core::feature::FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    (p, b, key)
}

#[test]
fn a_mirror_about_a_face_is_one_node_and_follows_the_face() {
    let (mut p, b, key) = block(40.0);
    let before = p.timeline.len();
    let m = p.add_mirror(b, 0, true, 0);
    p.set_op_face(m, b, key);
    assert_eq!(p.timeline.len(), before + 1, "a mirror about a face laid more than its own node");
    assert!(!p.timeline.iter().any(|n| matches!(n.kind, FeatureKind::Plane { .. })), "a datum plane stands beside the mirror");
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the mirror refused: {:?}", report.errors);
    let bb = shapes.get(&m).and_then(|s| s.bbox()).expect("the mirror's body");
    assert!((bb[3] - bb[0] - 80.0).abs() < 0.1, "mirrored about x = +20 the block runs {:.2} long, not 80", bb[3] - bb[0]);
    // the block made 50 long: its face goes to x = +25 and the mirror with it
    if let Some(n) = p.timeline.iter_mut().find(|n| n.id == b) {
        if let FeatureKind::Box3 { dx, .. } = &mut n.kind {
            *dx = 50.0;
        }
        n.dirty = true;
    }
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the mirror refused after the edit: {:?}", report.errors);
    let bb = shapes.get(&m).and_then(|s| s.bbox()).expect("the mirror's body");
    assert!((bb[3] - bb[0] - 100.0).abs() < 0.1, "the block made 50 long, its mirror runs {:.2}, not 100 - the plane did not follow the face", bb[3] - bb[0]);
}

#[test]
fn a_split_about_a_face_is_one_node() {
    let (mut p, b, key) = block(40.0);
    let before = p.timeline.len();
    let s = p.add_split_face(b, 0, 0, -5.0);
    p.set_op_face(s, b, key);
    assert_eq!(p.timeline.len(), before + 1, "a split about a face laid more than its own node");
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the split of faces refused: {:?}", report.errors);
    let faces = p.regen_faces.get(&s).map(|f| f.len()).unwrap_or(0);
    assert_eq!(faces, 10, "5 in from the face x = +20, the four side faces are split in two: 6 + 4 faces");
}
