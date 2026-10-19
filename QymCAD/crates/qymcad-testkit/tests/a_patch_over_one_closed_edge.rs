//! A PATCH OVER ONE CLOSED EDGE: the rim of a hole right through a block is a whole circle, a boundary by itself, and
//! the patch over it is a disc of pi 5^2 = 78.5 mm^2, as in the professional systems. Reported behaviour: refused,
//! "one edge cannot define a boundary".
use qymcad_core::model::Project;

#[test]
fn the_rim_of_a_hole_takes_a_disc() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let bore = p.add_cylinder(5.0, 30.0);
    let holed = p.add_body_boolean(block, bore, 0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let rim = p.regen_edges.get(&holed).and_then(|es| es.iter().find(|e| (e.radius - 5.0).abs() < 0.05).map(|e| e.id)).expect("the rim of the hole");
    let patch = p.add_patch(holed, qymcad_core::refs::Ref::picks(&[rim]), false);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the patch over the rim of the hole was refused: {:?}", report.errors);
    let area: f64 = p.regen_faces.get(&patch).map_or(0.0, |fs| fs.iter().map(|f| f.area).sum());
    let want = std::f64::consts::PI * 25.0;
    assert!((area - want).abs() < 0.5, "the patch over the rim is {area:.1} mm^2, a disc of 5 is {want:.1}");
}
