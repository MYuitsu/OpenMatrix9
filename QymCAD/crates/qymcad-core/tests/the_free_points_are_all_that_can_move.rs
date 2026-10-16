//! THE FREE POINTS ARE ALL THAT CAN MOVE: a point of a sketch is free when some motion its constraints allow moves it,
//! and held when none does - whatever the order the Jacobian is eliminated in.
//!
//! Reported behaviour: a rectangle drawn from its centre, its centre fixed - "Degrees of freedom: 2", and one corner
//! drawn yellow, three green, though the width and the height move all four.
use qymcad_core::feature::Purpose;
use qymcad_core::geom::Point2;
use qymcad_core::model::{Constraint, Project};

/// A sketch of its own, and the place of each point of it by where it stands.
fn sketch() -> (Project, usize) {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("S");
    (p, si)
}

fn id_at(p: &Project, si: usize, at: (f64, f64)) -> u64 {
    p.sketches[si].points.iter().find(|q| (q.x - at.0).abs() < 1e-9 && (q.y - at.1).abs() < 1e-9).map(|q| q.id).unwrap_or_else(|| panic!("no point at {at:?}"))
}

/// Which of `places` the sketch marks free, in their order, both by its checks and by `sketch_free_points`.
fn free_at(p: &Project, si: usize, places: &[(f64, f64)]) -> Vec<bool> {
    let by_checks = p.sketch_checks(si).free;
    let by_points = p.sketch_free_points(si);
    assert_eq!(by_checks, by_points, "the checks and the free points disagree");
    places.iter().map(|&at| p.sketches[si].points.iter().position(|q| q.id == id_at(p, si, at)).map(|k| by_points[k]).expect("the point")).collect()
}

const CORNERS: [(f64, f64); 4] = [(-20.0, -15.0), (20.0, -15.0), (20.0, 15.0), (-20.0, 15.0)];

#[test]
fn a_rectangle_with_its_centre_fixed_has_four_free_corners() {
    let (mut p, si) = sketch();
    p.add_rect_from_centre(si, Point2::new(0.0, 0.0), Point2::new(20.0, 15.0), Purpose::Real);
    let centre = id_at(&p, si, (0.0, 0.0));
    p.sketches[si].constraints.push(Constraint::Fixed { p: centre });
    p.solve_sketch(si);
    assert_eq!(p.sketch_checks(si).dof, (2, 0), "GUARD: a rectangle with its centre fixed keeps its width and height");
    assert_eq!(free_at(&p, si, &CORNERS), vec![true; 4], "every corner moves with the width or the height");
    assert_eq!(free_at(&p, si, &[(0.0, 0.0)]), vec![false], "the fixed centre is free");
}

#[test]
fn a_rectangle_with_its_sizes_laid_has_nothing_free() {
    let (mut p, si) = sketch();
    let ids = p.add_rect_from_centre(si, Point2::new(0.0, 0.0), Point2::new(20.0, 15.0), Purpose::Real);
    let centre = id_at(&p, si, (0.0, 0.0));
    p.sketches[si].constraints.push(Constraint::Fixed { p: centre });
    assert!(p.dimension_rect(si, ids[0]), "GUARD: the width and the height laid");
    p.solve_sketch(si);
    assert_eq!(p.sketch_checks(si).dof, (0, 0), "GUARD: a rectangle sized about a fixed centre is defined");
    assert_eq!(free_at(&p, si, &CORNERS), vec![false; 4], "a defined rectangle shows a free corner");
}

#[test]
fn a_line_with_one_end_fixed_has_the_other_free() {
    let (mut p, si) = sketch();
    p.add_line_entity(si, 0.0, 0.0, 30.0, 0.0, Purpose::Real);
    let start = id_at(&p, si, (0.0, 0.0));
    p.sketches[si].constraints.push(Constraint::Fixed { p: start });
    p.solve_sketch(si);
    assert_eq!(free_at(&p, si, &[(0.0, 0.0), (30.0, 0.0)]), vec![false, true]);
}

#[test]
fn a_circle_with_its_centre_fixed_has_its_centre_held() {
    let (mut p, si) = sketch();
    p.add_circle_entity(si, 5.0, 5.0, 10.0, Purpose::Real);
    let centre = id_at(&p, si, (5.0, 5.0));
    p.sketches[si].constraints.push(Constraint::Fixed { p: centre });
    p.solve_sketch(si);
    assert_eq!(free_at(&p, si, &[(5.0, 5.0)]), vec![false], "a fixed centre of a circle is free");
    assert_eq!(p.sketch_checks(si).dof.0, 1, "GUARD: the radius is free");
}
