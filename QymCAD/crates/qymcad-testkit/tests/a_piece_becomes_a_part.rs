//! A PIECE BECOMES A PART by hand: a cut through a block leaves the part two bodies, as the professional systems leave
//! a body in several, and one of them is made a part of its own. Each part is then one piece, and an edit of the cut
//! rebuilds both.
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

/// A 40x30x10 block (centred on the origin) with a slot 4 wide through it across: pieces of 18 and 18 by 30 by 10.
fn cut_through(width: f64) -> (Project, u64, u64) {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, -width / 2.0, -20.0, width / 2.0, 20.0, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(block, sid, c, CombineSpan { height: 10.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    (p, cut, sid)
}

/// The bodies the part of the cut shows, what a part of its own took left out.
fn shown_in(p: &Project, cut: u64) -> Vec<u64> {
    let home = p.body_owner(cut).expect("the part the cut stands in");
    let consumed = p.consumed_bodies();
    p.component_bodies(home).into_iter().filter(|b| !consumed.contains(b)).collect()
}

/// The body of the piece right of the slot, x > 0, as the cut node holds it.
fn right_piece(p: &Project, cut: u64) -> u64 {
    let node = p.timeline.iter().find(|n| n.kind.body() == Some(cut)).expect("the cut node");
    node.kind.cut_pieces().iter().find(|c| c.at[0] > 0.0).map(|c| c.body).expect("a piece right of the slot")
}

#[test]
fn the_right_piece_of_a_block_cut_through_becomes_a_part() {
    let (mut p, cut, _) = cut_through(4.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the cut did not build: {:?}", report.errors);
    let right = right_piece(&p, cut);
    assert!(shapes.contains_key(&right), "setup: the cut leaves the right piece a body of its own");
    let (part, piece) = p.piece_to_part(right, "name-part-n#2".into()).expect("the piece was made a part");
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "making a part of the piece was refused: {:?}", report.errors);
    let new = shapes.get(&piece).expect("the body of the new part");
    assert_eq!(new.solid_count(), 1, "the new part is {} pieces", new.solid_count());
    assert!((new.volume() - 5400.0).abs() < 1.0, "the new part is {:.1} mm^3, the right piece is 18 x 30 x 10 = 5400", new.volume());
    let left = shown_in(&p, cut);
    assert_eq!(left.len(), 1, "the part it came from shows {left:?}, the left piece alone was expected");
    let old = shapes.get(&left[0]).expect("the body of the left piece");
    assert_eq!(old.solid_count(), 1, "the part it came from is {} pieces", old.solid_count());
    assert!((old.volume() - 5400.0).abs() < 1.0, "the part it came from is {:.1} mm^3, the left piece is 5400", old.volume());
    assert_eq!(p.body_owner(piece), Some(part), "the piece stands in no part of its own");
}

/// AN EDIT OF THE CUT REBUILDS BOTH: the slot drawn 10 wide instead of 4, each piece is 15 x 30 x 10 = 4500.
#[test]
fn a_wider_cut_rebuilds_both_parts() {
    let (mut p, cut, sid) = cut_through(4.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let right = right_piece(&p, cut);
    let (_, piece) = p.piece_to_part(right, "name-part-n#2".into()).expect("the piece was made a part");
    let _ = qymcad_testkit::regenerate(&mut p);
    let si = p.sketch_index(sid).expect("the sketch of the cut");
    for pt in p.sketches[si].points.iter_mut() {
        pt.x = if pt.x > 0.0 { 5.0 } else { -5.0 };
    }
    p.regen_sketch(si);
    p.mark_sketch_dirty(sid);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the wider cut does not build: {:?}", report.errors);
    let v = shapes.get(&piece).map(|s| s.volume()).unwrap_or(0.0);
    assert!((v - 4500.0).abs() < 1.0, "the piece after a slot of 10 is {v:.1} mm^3, 15 x 30 x 10 = 4500");
    let rest = shown_in(&p, cut).first().and_then(|b| shapes.get(b)).map(|s| s.volume()).unwrap_or(0.0);
    assert!((rest - 4500.0).abs() < 1.0, "the part the cut stands in is {rest:.1} mm^3 after a slot of 10, 4500");
}

/// THE CUT DELETED, THE PIECE HAS NOTHING TO BE: its node stands red with the reason, rather than taking the whole
/// block into the new part - which would stand a second block over the first, the part it came from whole again.
#[test]
fn a_piece_whose_cut_is_deleted_stands_red() {
    let (mut p, cut, _) = cut_through(4.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let right = right_piece(&p, cut);
    let (_, piece) = p.piece_to_part(right, "name-part-n#2".into()).expect("the piece was made a part");
    let _ = qymcad_testkit::regenerate(&mut p);
    p.delete_feature_op(cut);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    let node = p.timeline.iter().find(|n| n.kind.body() == Some(piece)).map(|n| n.id).expect("the node of the piece stays");
    assert!(
        report.errors.iter().any(|(id, _)| *id == node) || p.regen_errors.contains_key(&node),
        "the piece whose cut was deleted is not red: it reads {:?}",
        p.timeline.iter().find(|n| n.id == node).map(|n| n.kind.inputs())
    );
}
