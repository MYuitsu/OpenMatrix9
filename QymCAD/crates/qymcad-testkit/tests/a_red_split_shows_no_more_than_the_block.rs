//! A RED SPLIT SHOWS NO MORE THAN THE BLOCK: a split whose plane is moved to where it cuts nothing turns red, and the
//! part keeps its last good pieces - not the whole block beside one of them.
//!
//! Reported behaviour (found by the contract of every tool with a parameter): the plane of a split moved to the bottom of
//! the block, the node red, and the part held 18000 mm^3 - the whole block of 12000 passed through into the first piece,
//! and the second piece of 6000 beside it.
use qymcad_core::model::Project;

#[test]
fn the_pieces_stay_as_they_were_when_the_split_goes_red() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let pieces = p.add_split_body(block, 0, 0, 5.0, 2);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the split did not build: {:?}", report.errors);
    let total = |shapes: &std::collections::HashMap<u64, qymcad_kernel::Shape>| pieces.iter().filter_map(|b| shapes.get(b)).map(|s| s.volume()).sum::<f64>();
    assert!((total(&shapes) - 12000.0).abs() < 1.0, "setup: the two pieces hold {:.1}, the block 12000", total(&shapes));
    for n in p.timeline.iter_mut() {
        if let qymcad_core::feature::FeatureKind::SplitBody { offset, .. } = &mut n.kind {
            *offset = 0.0; // the plane on the bottom of the block: it cuts nothing
            n.dirty = true;
        }
    }
    let (report, shapes) = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, shapes);
    assert!(!report.errors.is_empty(), "a split that cuts nothing stood green");
    let shown: f64 = p.bodies.iter().filter(|b| pieces.contains(&b.id)).map(|b| b.mesh.volume()).sum();
    assert!((shown - 12000.0).abs() < 1.0 && (total(&shapes) - 12000.0).abs() < 1.0, "the red split shows {shown:.1} mm^3 and holds {:.1}, the block is 12000", total(&shapes));
}
