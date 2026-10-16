//! A ROUNDING WITH ONE OF ITS PICKED EDGES LOST rounds the rest and says in yellow how many were left, rather than
//! rounding the rest without a word or going red with all that follows (decided 26.09).
use qymcad_core::errors::CoreError;
use qymcad_core::model::Project;

#[test]
fn a_rounding_with_one_of_two_edges_lost_warns() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let real = p.regen_edges.get(&block).and_then(|es| es.first().map(|e| e.id)).expect("an edge of the block");
    // a name no edge of the block has ever carried, and no place to find it by
    let gone = 0x6000_7777;
    let f = p.add_fillet_ref(block, 1.0, qymcad_core::refs::Ref::picks(&[real, gone]));
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the rounding went red: {:?}", report.errors);
    assert!(shapes.get(&f).is_some_and(|s| s.volume() < 12000.0 - 0.1), "the edge still there was not rounded");
    let said = p.regen_warnings.get(&f).cloned();
    assert!(matches!(said, Some(CoreError::EdgesDropped { asked: 2, dropped: 1 })), "the rounding with one of its two edges lost says {said:?}");
}
