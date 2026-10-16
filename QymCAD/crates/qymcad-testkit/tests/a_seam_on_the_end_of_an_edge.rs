//! AN EDGE WHOSE END A SEAM GOES STRAIGHT ON FROM IS ROUNDED like any other. The seam of a round face is where the face
//! happened to be made - on the X side of a cylinder here - and on any part it can fall on the end of an edge: a
//! quarter cut out of the foot of a cylinder puts an upright edge exactly on it, the seam running up from its top.
//!
//! Reported behaviour: of eight edges of a symmetric part one would not take any radius, the seam of the round corner
//! beside it running up from its end.
use qymcad_core::feature::{Purpose, Reach};
use qymcad_core::model::{CombineSpan, Project};

#[test]
fn an_upright_edge_under_the_seam_of_a_cylinder_takes_a_rounding() {
    let mut p = Project::default();
    p.new_document();
    let cyl = p.add_cylinder(5.0, 6.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, 0.0, -10.0, 10.0, 0.0, Purpose::Real); // the quarter from the seam line (x = +5, y = 0) clockwise
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(cyl, sid, c, CombineSpan { height: 4.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "setup: the cut did not build: {:?}", report.errors);
    // the upright edge at (5, 0) from z 0 to 4, on the seam line
    let edge = p
        .regen_edges
        .get(&cut)
        .and_then(|es| es.iter().find(|e| (e.mid[0] - 5.0).abs() < 1e-3 && e.mid[1].abs() < 1e-3 && (e.mid[2] - 2.0).abs() < 1e-3).map(|e| e.id))
        .expect("the upright edge on the seam line");
    let before = shapes.get(&cut).map(|s| s.volume()).expect("the cut cylinder");
    let round = p.add_fillet(cut, 1.0, vec![edge]);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the rounding under the seam was refused: {:?}", report.errors);
    assert!(!p.regen_warnings.contains_key(&round), "the rounding left its edge out: {:?}", p.regen_warnings.get(&round));
    let after = shapes.get(&round).map(|s| s.volume()).unwrap_or(before);
    assert!((after - before).abs() > 0.1, "the rounding under the seam changed nothing: {before:.2} -> {after:.2}");
}

/// THE SAME FOR A BEVEL: the upright edge on the seam line of the cylinder takes a chamfer of 1.
#[test]
fn an_upright_edge_under_the_seam_of_a_cylinder_takes_a_chamfer() {
    let mut p = Project::default();
    p.new_document();
    let cyl = p.add_cylinder(5.0, 6.0);
    let si = p.new_sketch("Tool");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "Tool");
    p.add_rect_entity(si, 0.0, -10.0, 10.0, 0.0, Purpose::Real);
    p.regen_sketch(si);
    let c: Vec<u64> = p.sketches[si].contour_ids.iter().copied().filter(|c| p.contour_profile_xy(*c).is_some()).collect();
    let cut = p.add_combine_multi_op(cyl, sid, c, CombineSpan { height: 4.0, down: 0.0, extent: qymcad_core::feature::Extent { reach: Reach::Forward, ..Default::default() }, fill: &[] }, 0);
    let (_, shapes) = qymcad_testkit::regenerate(&mut p);
    let edge = p
        .regen_edges
        .get(&cut)
        .and_then(|es| es.iter().find(|e| (e.mid[0] - 5.0).abs() < 1e-3 && e.mid[1].abs() < 1e-3 && (e.mid[2] - 2.0).abs() < 1e-3).map(|e| e.id))
        .expect("the upright edge on the seam line");
    let before = shapes.get(&cut).map(|s| s.volume()).expect("the cut cylinder");
    let bevel = p.add_chamfer(cut, 1.0, vec![edge]);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the chamfer under the seam was refused: {:?}", report.errors);
    let after = shapes.get(&bevel).map(|s| s.volume()).unwrap_or(before);
    assert!((after - before).abs() > 0.1, "the chamfer under the seam changed nothing: {before:.2} -> {after:.2}");
}
