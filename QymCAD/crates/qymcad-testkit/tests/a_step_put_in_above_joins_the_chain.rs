//! A STEP PUT IN ABOVE JOINS THE CHAIN: the timeline rolled back above a rounding and a cut added there - the rounding
//! then carries on from the body with the cut, and the part stays one body. Reported behaviour: the part held two
//! bodies, 11000 mm^3 with the cut and 11882.9 = 12000 - 117.1, the rounding on the block without it.
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

#[test]
fn a_cut_put_in_above_a_rounding_is_rounded_after() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let back = p.regen_edges.get(&block).and_then(|es| es.iter().filter(|e| e.a[2] > 9.9 && e.b[2] > 9.9).max_by(|a, b| a.mid[1].total_cmp(&b.mid[1])).map(|e| e.id)).expect("the top back edge");
    let round = p.add_fillet(block, 2.0, vec![back]);
    let _ = qymcad_testkit::regenerate(&mut p);
    // back to just after the block, a strip cut off the front of the top there
    let at = p.timeline.iter().position(|n| n.id == block).expect("the block") + 1;
    p.rollback = Some(at);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, -25.0, -20.0, 25.0, -10.0, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(block, sid, c, CombineSpan { height: 20.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    p.rollback = None;
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the chain did not build: {:?}", report.errors);
    let consumed = p.consumed_bodies();
    let live: Vec<u64> = [block, cut, round].into_iter().filter(|b| !consumed.contains(b)).collect();
    assert_eq!(live, vec![round], "the part holds the bodies {live:?}, not the rounding alone");
    let cut_v = shapes.get(&cut).map(|s| s.volume()).unwrap_or(0.0);
    let round_v = shapes.get(&round).map(|s| s.volume()).unwrap_or(0.0);
    assert!(round_v < cut_v, "the rounding stands on the block without the cut: {round_v:.1} against the cut {cut_v:.1}");
}
