//! A PART NEVER FALLS INTO PIECES: a part is one body of one piece, so an addition that touches nothing is refused in
//! words and the body stays as it was.
//!
//! Reported behaviour: a circle drawn beside a 40x30x10 block (centred on the origin) and added 5 up stood green, the body two pieces of
//! 12565 mm^3 together.
use qymcad_core::errors::CoreError;
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

#[test]
fn an_addition_that_touches_nothing_is_refused_and_the_body_stays_one_piece() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_circle_entity(si, 0.0, 40.0, 6.0, Purpose::Real);
    p.regen_sketch(si);
    let circle: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let add = p.add_combine_multi_op(block, sid, circle, CombineSpan { height: 5.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 1);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let body = shapes.get(&add).expect("the addition passes the block on");
    assert_eq!(body.solid_count(), 1, "the body of the part is {} pieces, {:.0} mm^3", body.solid_count(), body.volume());
    assert!((body.volume() - 12000.0).abs() < 1.0, "the block is {:.0} mm^3 after the refused addition, it was 12000", body.volume());
    assert!(report.errors.iter().any(|(id, e)| *id == add && matches!(e, CoreError::BodyInPieces)), "the addition that touches nothing was not refused in words: {:?}", report.errors);
}

/// A TOUCHING ADDITION STANDS: the same circle across the edge of the block joins it, one piece.
#[test]
fn an_addition_across_the_edge_joins_the_block() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_circle_entity(si, 0.0, 15.0, 6.0, Purpose::Real);
    p.regen_sketch(si);
    let circle: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let add = p.add_combine_multi_op(block, sid, circle, CombineSpan { height: 5.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 1);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the touching addition was refused: {:?}", report.errors);
    let body = shapes.get(&add).expect("the joined body");
    assert_eq!(body.solid_count(), 1, "the joined body is {} pieces", body.solid_count());
}
