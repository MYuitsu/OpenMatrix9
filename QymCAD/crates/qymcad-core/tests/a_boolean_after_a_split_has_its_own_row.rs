//! A BOOLEAN THE PERSON MADE HAS A ROW OF ITS OWN IN THE TREE; only the join that makes a part one body folds into
//! the row of the feature it joins.
//!
//! Reported behaviour: a boolean of the two pieces of a split could not be found in the tree to reopen - it was folded
//! into the row of the split.
use qymcad_core::feature::FeatureKind;
use qymcad_core::geom::Point2;
use qymcad_core::model::{Id, Project};

fn extrude(p: &mut Project, name: &str, x: f64) -> Id {
    let sid = p.add_line_sketch(name, vec![Point2::new(x, 0.0), Point2::new(x + 10.0, 0.0), Point2::new(x + 10.0, 10.0), Point2::new(x, 10.0)], true);
    p.add_sketch_node(sid, name);
    let b = p.add_extrude(sid, 10.0);
    p.finish_base_body(b, 1)
}

fn node_of(p: &Project, pick: impl Fn(&FeatureKind) -> bool) -> Id {
    p.timeline.iter().find(|n| pick(&n.kind)).map(|n| n.id).expect("the node")
}

#[test]
fn a_boolean_after_a_split_is_not_folded_into_it_and_the_join_still_is() {
    let mut fails = Vec::new();
    let mut p = Project::default();
    p.new_document();
    // the join: a second extrude of the part is fused into the first, and the two nodes are one row
    let first = extrude(&mut p, "a", 0.0);
    let _ = extrude(&mut p, "b", 5.0);
    let second = p.timeline.iter().filter(|n| matches!(n.kind, FeatureKind::Extrude { .. })).nth(1).map(|n| n.id).expect("the second extrude");
    if p.feature_op_span(second).len() != 2 {
        fails.push(format!("the join is not folded into the extrude it joins: {:?}", p.feature_op_span(second)));
    }
    // the person's boolean of the two pieces of a split is a row of its own, whichever piece is the tool
    let joined = p.timeline.iter().rev().find_map(|n| n.kind.body()).expect("the part's body");
    let _ = first;
    let pieces = p.add_split_body(joined, 0, 0, 5.0, 2);
    let split = node_of(&p, |k| matches!(k, FeatureKind::SplitBody { .. }));
    let tool = p.timeline.iter().find(|n| n.id == split).and_then(|n| n.kind.body()).expect("the split names a body");
    let base = *pieces.iter().find(|b| **b != tool).expect("the other piece");
    p.add_body_boolean(base, tool, 0);
    if p.feature_op_span(split) != vec![split] {
        fails.push(format!("the boolean after the split is folded into the split's row: {:?}", p.feature_op_span(split)));
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

/// A boss grown on one piece of a split is joined into that piece by its own node, not by a join that follows it; so a
/// boolean the person makes next, with the grown piece as the tool, is a row of its own. Reported behaviour: the
/// boolean of the two pieces was folded into the boss's row and could not be found in the tree to reopen.
#[test]
fn a_boolean_after_a_boss_on_a_piece_is_not_folded_into_the_boss() {
    let mut p = Project::default();
    p.new_document();
    let joined = extrude(&mut p, "a", 0.0);
    let pieces = p.add_split_body(joined, 0, 0, 5.0, 2);
    let (lower, upper) = (pieces[0], pieces[1]);
    let sid = p.add_line_sketch("boss", vec![Point2::new(2.0, 2.0), Point2::new(8.0, 2.0), Point2::new(8.0, 8.0), Point2::new(2.0, 8.0)], true);
    p.add_sketch_node(sid, "boss");
    let contours = p.sketches[p.sketch_index(sid).expect("the sketch")].contour_ids.clone();
    let boss = p.add_combine_multi_op(lower, sid, contours, qymcad_core::model::CombineSpan { height: 8.0, down: 0.0, extent: Default::default(), fill: &[] }, 1);
    let grown = p.timeline.iter().find(|n| n.id == boss).and_then(|n| n.kind.body()).expect("the boss names a body");
    p.add_body_boolean(upper, grown, 0);
    assert!(p.feature_op_span(boss) == vec![boss], "the person's boolean is folded into the boss's row: {:?}", p.feature_op_span(boss));
}
