//! A CUT THAT ONLY TOUCHES THE BODY removed nothing and says so. Reported behaviour: a square on the top of a block
//! cut 5 up, away from it, took 0.1 mm^3 along the face it stood on, cut the top into 11 faces and stood green.
use qymcad_core::errors::CoreError;
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

#[test]
fn a_cut_away_from_the_block_is_refused_in_words() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    // the sketch on the bottom of the block (z = 0), the cut going down, away from it
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, -5.0, -5.0, 5.0, 5.0, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(block, sid, c, CombineSpan { height: 5.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Backward, ..Default::default() }, fill: &[] }, 0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let v = shapes.get(&cut).map(|s| s.volume()).unwrap_or(0.0);
    assert!(report.errors.iter().any(|(id, e)| *id == cut && matches!(e, CoreError::CutRemovedNothing)), "a cut away from the block was not refused in words ({v:.1} mm^3 left): {:?}", report.errors);
}
