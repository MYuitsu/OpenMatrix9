//! A CUT THROUGH LEAVES BODIES OF ONE PART: a slot through a block parts it in two, and each piece is a body of its own
//! in the same part, as splitting a body makes them. An edit of the cut keeps every body on its piece; a cut that no
//! longer goes through leaves one body again.
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

/// A 40x30x10 block (centred on the origin) with a slot `width` wide across it, from x = `at - width/2`: through when
/// the slot runs past both long sides, `long` = 20; stopping inside the block with `long` < 15.
fn slot(at: f64, width: f64, long: f64) -> (Project, u64, u64) {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, at - width / 2.0, -long, at + width / 2.0, long, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(block, sid, c, CombineSpan { height: 10.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    (p, cut, sid)
}

/// The bodies the cut node makes and their volumes, its own body first.
fn bodies_of(p: &Project, shapes: &std::collections::HashMap<u64, qymcad_kernel::Shape>, cut: u64) -> Vec<(u64, f64, u32)> {
    let node = p.timeline.iter().find(|n| n.kind.body() == Some(cut)).expect("the cut node");
    node.kind.bodies().into_iter().map(|b| (b, shapes.get(&b).map(|s| s.volume()).unwrap_or(0.0), shapes.get(&b).map(|s| s.solid_count()).unwrap_or(0))).collect()
}

#[test]
fn a_slot_through_a_block_leaves_two_bodies_of_its_part() {
    // left piece from -20 to 3 (23 x 30 x 10 = 6900), right piece from 7 to 20 (13 x 30 x 10 = 3900)
    let (mut p, cut, _) = slot(5.0, 4.0, 20.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the cut did not build: {:?}", report.errors);
    let got = bodies_of(&p, &shapes, cut);
    assert_eq!(got.len(), 2, "a slot through the block leaves two bodies, the cut node has {got:?}");
    assert!(got.iter().all(|b| b.2 == 1), "each body is one piece: {got:?}");
    assert!((got[0].1 - 6900.0).abs() < 1.0 && (got[1].1 - 3900.0).abs() < 1.0, "the cut's own body is the bigger piece, 6900 then 3900: {got:?}");
    let home = p.body_owner(cut);
    assert!(home.is_some() && got.iter().all(|b| p.body_owner(b.0) == home), "both pieces stand in the part of the cut");
    let visible: Vec<u64> = p.bodies.iter().filter(|b| b.visible && !p.consumed_bodies().contains(&b.id)).map(|b| b.id).collect();
    assert!(got.iter().all(|b| visible.contains(&b.0)), "both pieces are on screen: {visible:?}, the pieces are {got:?}");
}

/// THE SLOT MOVED ACROSS THE MIDDLE: each body stays on its piece - the left body grows no new name, the right one is
/// still the right one - though the right piece is now the bigger.
#[test]
fn an_edit_of_the_cut_keeps_every_body_on_its_piece() {
    let (mut p, cut, sid) = slot(5.0, 4.0, 20.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let (_, shapes) = qymcad_testkit::regenerate(&mut p);
    let before = bodies_of(&p, &shapes, cut);
    let si = p.sketch_index(sid).expect("the sketch of the cut");
    for pt in p.sketches[si].points.iter_mut() {
        pt.x -= 10.0; // the slot from -7 to -3: left 13 x 30 x 10 = 3900, right 23 x 30 x 10 = 6900
    }
    p.regen_sketch(si);
    p.mark_sketch_dirty(sid);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the moved cut does not build: {:?}", report.errors);
    let after = bodies_of(&p, &shapes, cut);
    assert_eq!(after.iter().map(|b| b.0).collect::<Vec<_>>(), before.iter().map(|b| b.0).collect::<Vec<_>>(), "the bodies of the pieces changed: {before:?} -> {after:?}");
    assert!((after[0].1 - 3900.0).abs() < 1.0 && (after[1].1 - 6900.0).abs() < 1.0, "the left body is still the left piece (3900), the right the right (6900): {after:?}");
}

/// A SLOT THAT STOPS INSIDE: the block stays whole, one body, and a piece body the cut had before goes.
#[test]
fn a_cut_that_no_longer_goes_through_leaves_one_body() {
    let (mut p, cut, sid) = slot(5.0, 4.0, 20.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let si = p.sketch_index(sid).expect("the sketch of the cut");
    for pt in p.sketches[si].points.iter_mut() {
        pt.y = pt.y.signum() * 10.0; // 20 long of the 30 across: the slot stops inside
    }
    p.regen_sketch(si);
    p.mark_sketch_dirty(sid);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the shorter cut does not build: {:?}", report.errors);
    let got = bodies_of(&p, &shapes, cut);
    assert_eq!(got.len(), 1, "a slot that stops inside leaves one body: {got:?}");
    assert!((got[0].1 - (12000.0 - 800.0)).abs() < 1.0, "the block less a slot 4 x 20 x 10 is 11200: {got:?}");
    assert_eq!(p.bodies.len(), p.timeline.iter().flat_map(|n| n.kind.bodies()).count(), "a body of a piece that is gone was left in the document");
}
