//! A DATUM STANDS ON THE BODY IT WAS TAKEN FROM: a plane from a face of a block, and an axis from a face of a cylinder,
//! go red with the reason when that body's node is deleted, rather than standing green on the fingerprint the face
//! left behind.
//!
//! Reported behaviour: the extrusion a plane from a face stood on was deleted, and the plane stayed green, hanging in
//! the air.
use qymcad_core::errors::CoreError;
use qymcad_core::model::Project;

#[test]
fn a_datum_on_a_deleted_body_goes_red_with_the_reason() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let tube = p.add_cylinder(5.0, 20.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.99)).expect("the top of the block");
    let key = qymcad_core::feature::FaceKey { index: 0, centroid: [top.centroid.x, top.centroid.y, top.centroid.z], normal: top.normal, id: top.id };
    let plane = p.add_plane_from_face(block, key, 5.0);
    let side = p.regen_faces.get(&tube).and_then(|fs| fs.iter().find(|f| f.normal[2].abs() < 0.5)).map(|f| f.id).expect("the side of the cylinder");
    let axis = p.add_axis_from_face(tube, side);
    let (report, mut shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the datums refused on their bodies: {:?}", report.errors);
    // deleted as the window deletes: the node alone, then a rebuild of what the deletion marked, nothing forced -
    // the window rebuilds nothing at all when nothing is marked, so the datum on the body has to be marked
    for (node, datum) in [(block, plane), (tube, axis)] {
        p.delete_feature_op(node);
        assert!(p.timeline.iter().any(|n| n.id == datum && n.dirty), "the body under datum {datum} deleted, the datum is not marked for the rebuild");
        shapes = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, shapes).1;
    }
    for (id, what) in [(plane, "the plane from the block's face"), (axis, "the axis from the cylinder's face")] {
        assert!(matches!(p.regen_errors.get(&id), Some(CoreError::SourceBodyDeleted)), "{what}: its body deleted, it stands {:?}", p.regen_errors.get(&id));
    }
}
