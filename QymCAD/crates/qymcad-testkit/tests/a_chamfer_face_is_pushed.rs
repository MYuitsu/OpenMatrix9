//! THE FACE OF A CHAMFER IS PUSHED: a flat face at 45 deg moves along its normal like any flat face, the faces beside
//! it follow, and the leg of the chamfer changes.
//!
//! Reported behaviour: the face of a 2 mm chamfer on the top front edge of a 40x30x10 block, pushed 1 out or in, stood
//! green and left the body as it was, to the mm^3.
use qymcad_core::feature::FaceKey;
use qymcad_core::model::Project;

fn pushed(dist: f64) -> (f64, f64, u32, Vec<String>) {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let es = p.regen_edges.get(&block).cloned().expect("the edges of the block");
    let top = es.iter().map(|e| e.a[2].max(e.b[2])).fold(f64::MIN, f64::max);
    let front = es.iter().map(|e| e.a[1].min(e.b[1])).fold(f64::MAX, f64::min);
    let edge = es.iter().find(|e| (e.a[2] - top).abs() < 1e-6 && (e.b[2] - top).abs() < 1e-6 && (e.a[1] - front).abs() < 1e-6 && (e.b[1] - front).abs() < 1e-6).expect("the top front edge").id;
    let ch = p.add_chamfer(block, 2.0, vec![edge]);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the chamfer did not build: {:?}", report.errors);
    let before = shapes.get(&ch).map(|s| s.volume()).expect("the chamfered block");
    let f = p.regen_faces.get(&ch).and_then(|fs| fs.iter().find(|f| f.normal[1] < -0.5 && f.normal[2] > 0.5)).cloned().expect("the face of the chamfer");
    let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
    let push = p.add_push_face(ch, key, dist);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let after = shapes.get(&push).map(|s| s.volume()).unwrap_or(before);
    let solids = shapes.get(&push).map_or(0, |s| s.solid_count());
    (before, after, solids, report.errors.iter().map(|(_, e)| format!("{e:?}")).collect())
}

#[test]
fn the_face_of_a_chamfer_goes_out_and_in() {
    // a 2 x 2 chamfer: its face is 2.83 wide; moved out by 1 the legs become 2 - sqrt(2) = 0.586, in by 1 they become
    // 2 + sqrt(2) = 3.414; the triangle cut off is leg^2 / 2 over the 40 of the edge
    for (dist, leg) in [(1.0, 2.0 - 2f64.sqrt()), (-1.0, 2.0 + 2f64.sqrt())] {
        let (before, after, solids, errors) = pushed(dist);
        let want = 12000.0 - leg * leg / 2.0 * 40.0;
        assert!(errors.is_empty(), "the face of the chamfer pushed {dist} was refused: {errors:?}");
        assert_eq!(solids, 1, "the face of the chamfer pushed {dist} left {solids} solids - a sheet or pieces, not the part");
        assert!((after - want).abs() < 0.5, "the face of the chamfer pushed {dist}: the body went {before:.1} -> {after:.1}, a chamfer leg of {leg:.3} leaves {want:.1}");
    }
}

/// THE SIDE OF A CYLINDER IS PUSHED like a flat face: 2 out makes a cylinder of radius 12 from one of 10, as the
/// offset of a face in a professional CAD. Reported behaviour: refused in words, "curved face".
#[test]
fn the_side_of_a_cylinder_goes_out_and_in() {
    for (dist, r) in [(2.0, 12.0), (-2.0, 8.0)] {
        let mut p = Project::default();
        p.new_document();
        let cyl = p.add_cylinder(10.0, 10.0);
        let _ = qymcad_testkit::regenerate(&mut p);
        let f = p.regen_faces.get(&cyl).and_then(|fs| fs.iter().find(|f| f.normal[2].abs() < 0.5)).cloned().expect("the side of the cylinder");
        let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
        let push = p.add_push_face(cyl, key, dist);
        let (report, shapes) = qymcad_testkit::regenerate(&mut p);
        assert!(report.errors.is_empty(), "the side of the cylinder pushed {dist} was refused: {:?}", report.errors);
        let body = shapes.get(&push).expect("the pushed cylinder");
        let want = std::f64::consts::PI * r * r * 10.0;
        assert_eq!(body.solid_count(), 1, "the pushed cylinder is {} solids", body.solid_count());
        assert!((body.volume() - want).abs() < 1.0, "the side pushed {dist}: {:.1} mm^3, a cylinder of radius {r} is {want:.1}", body.volume());
    }
}
