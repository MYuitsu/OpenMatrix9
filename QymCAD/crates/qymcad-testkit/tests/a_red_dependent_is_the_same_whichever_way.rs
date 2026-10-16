//! A NODE THAT STANDS ON A DELETED NODE IS THE SAME WHICHEVER WAY IT IS COME TO: rebuilt after the deletion, as the
//! window rebuilds - only what the deletion marked, over the live bodies it holds - and rebuilt from the start. A patch
//! over the rim of a hole, the hole deleted: the patch hands over to the block, which has no rim. Reported by the check
//! of round trips: step by step red, undone and done again green over a rim that is no longer there.
use qymcad_core::model::Project;

fn errors_by_node(p: &Project) -> Vec<(u64, String)> {
    let mut v: Vec<(u64, String)> = p.regen_errors.iter().map(|(n, e)| (*n, format!("{e:?}"))).collect();
    v.sort();
    v
}

#[test]
fn a_patch_over_a_deleted_hole_is_the_same_rebuilt() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let top = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9).cloned()).expect("the top of the block");
    let key = qymcad_core::feature::FaceKey { index: 0, centroid: [top.centroid.x, top.centroid.y, top.centroid.z], normal: top.normal, id: top.id };
    let hole = p.add_hole(block, key, 10.0, 30.0);
    let (_, shapes) = qymcad_testkit::regenerate(&mut p);
    let holed = p.timeline.iter().find(|n| n.id == hole).and_then(|n| n.kind.body()).expect("the body of the hole");
    let rim = p.regen_edges.get(&holed).and_then(|es| es.iter().find(|e| (e.radius - 5.0).abs() < 0.05).map(|e| e.id)).expect("the rim of the hole");
    let patch = p.add_patch(holed, qymcad_core::refs::Ref::picks(&[rim]), false);
    let (report, shapes) = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, shapes);
    assert!(report.errors.is_empty(), "the patch over the rim was refused: {:?}", report.errors);
    p.delete_feature_op(hole);
    let (_, _) = qymcad_testkit::regenerate_dirty_with_shapes(&mut p, shapes);
    let step = errors_by_node(&p);
    let mut whole = p.clone();
    let _ = qymcad_testkit::regenerate(&mut whole);
    let rebuilt = errors_by_node(&whole);
    assert!(step == rebuilt, "the patch {patch} after its hole was deleted: {step:?} step by step, {rebuilt:?} rebuilt");
    assert!(!rebuilt.is_empty(), "the patch over a rim that is gone stands green");
}
