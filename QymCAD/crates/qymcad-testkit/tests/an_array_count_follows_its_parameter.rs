//! AN ARRAY COUNT FOLLOWS ITS PARAMETER: the count typed as an expression is a dimension of the node, as the step and
//! the angle are.
//!
//! Reported behaviour (found by the contract of every tool with a parameter): a circular array of `k` = 6 copies stayed
//! at 6 copies when `k` became 12.
use qymcad_core::model::{Param, Project};

#[test]
fn a_circular_array_of_k_copies_grows_with_k() {
    let mut p = Project::default();
    p.new_document();
    p.parameters.push(Param { name: "k".into(), expr: "6".into(), value: 6.0 });
    let block = p.add_box(10.0, 10.0, 10.0);
    // the block well off the axis, so the copies stand apart and each is a piece of its own
    let moved = p.add_move(block, [1.0, 0.0, 0.0, 50.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    let arr = p.add_circular_array_axis(moved, 6, 360.0, 0);
    p.set_feat_dim(arr, "count", "k".into());
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the array did not build: {:?}", report.errors);
    assert_eq!(shapes.get(&arr).map(|s| s.solid_count()), Some(6), "k = 6 copies");
    if let Some(k) = p.parameters.iter_mut().find(|q| q.name == "k") {
        k.expr = "12".into();
        k.value = 12.0;
    }
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the array of 12 did not build: {:?}", report.errors);
    assert_eq!(shapes.get(&arr).map(|s| s.solid_count()), Some(12), "k became 12 and the array did not follow");
}
