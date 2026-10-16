//! A SYMMETRIC CHAMFER IS SIZED BY ITS CUT: the value given is the length of the cut, the dimension stands on the two
//! ends of the cut, and the legs are what that length makes on the corner - cut / (2 sin(corner / 2)), 2.1213 for a cut
//! of 3 on a right angle. Two legs, or a leg and an angle, stay measured from the sharp corner.
//!
//! Reported behaviour: the field of the symmetric chamfer is called the size of the chamfer, yet the dimension it laid
//! was a leg, from the hidden sharp corner to one end of the cut.
use qymcad_core::feature::{ChamferMode, Purpose};
use qymcad_core::model::{ChamferLegs, Constraint, CornerCut, Project};

/// An L of two lines meeting at a right angle at (20, 0). Answers the project, the sketch, the corner and its two lines.
fn an_angle() -> (Project, usize, u64, (u64, u64)) {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("S");
    let left = p.add_line_entity(si, 0.0, 0.0, 20.0, 0.0, Purpose::Real);
    let up = p.add_line_entity(si, 20.0, 0.0, 20.0, 20.0, Purpose::Real);
    p.regen_sketch(si);
    let corner = p.sketches[si].points.iter().find(|q| (q.x - 20.0).abs() < 1e-6 && q.y.abs() < 1e-6).map(|q| q.id).expect("the corner");
    (p, si, corner, (left, up))
}

/// The two ends of the cut: the points the chamfer laid on the lines, not the far ends and not the corner.
fn cut_ends(p: &Project, si: usize, corner: u64) -> Vec<(u64, f64, f64)> {
    p.sketches[si].points.iter().filter(|q| q.id != corner && !(q.x.abs() < 1e-6 && q.y.abs() < 1e-6) && !((q.x - 20.0).abs() < 1e-6 && (q.y - 20.0).abs() < 1e-6)).map(|q| (q.id, q.x, q.y)).collect()
}

#[test]
fn the_value_is_the_cut_and_the_dimension_stands_on_its_ends() {
    let (mut p, si, corner, edges) = an_angle();
    assert!(p.chamfer_lines_of_pair(si, edges, ChamferLegs::equal(3.0), None), "the corner is chamfered");
    let ends = cut_ends(&p, si, corner);
    assert_eq!(ends.len(), 2, "two ends of the cut: {ends:?}");
    let (a, b) = (ends[0], ends[1]);
    let cut = (a.1 - b.1).hypot(a.2 - b.2);
    assert!((cut - 3.0).abs() < 1e-6, "a symmetric chamfer of 3 made a cut of {cut:.4}");
    let on_the_cut =
        p.sketches[si].constraints.iter().any(|c| matches!(c, Constraint::Distance { a: x, b: y, d, .. } if ((*x == a.0 && *y == b.0) || (*x == b.0 && *y == a.0)) && (d - 3.0).abs() < 1e-9));
    assert!(on_the_cut, "no dimension of 3 on the two ends of the cut: {:?}", p.sketches[si].constraints);
    let from_the_sharp = p.sketches[si].constraints.iter().any(|c| matches!(c, Constraint::Distance { a: x, b: y, .. } if *x == corner || *y == corner));
    assert!(!from_the_sharp, "a dimension of the symmetric chamfer is still measured from the sharp corner");

    // the dimension drives the cut, and the cut stays symmetric
    for c in &mut p.sketches[si].constraints {
        if let Constraint::Distance { a: x, b: y, d, .. } = c {
            if (*x == a.0 && *y == b.0) || (*x == b.0 && *y == a.0) {
                *d = 5.0;
            }
        }
    }
    let r = p.solve_sketch(si);
    let at = |id: u64| p.sketches[si].points.iter().find(|q| q.id == id).map(|q| (q.x, q.y)).expect("the point");
    let ((ax, ay), (bx, by)) = (at(a.0), at(b.0));
    let legs = [(ax - 20.0).hypot(ay), (bx - 20.0).hypot(by)];
    assert!(r < 1e-6 && ((ax - bx).hypot(ay - by) - 5.0).abs() < 1e-6, "the cut did not follow its dimension to 5: residual {r:.2e}");
    assert!((legs[0] - legs[1]).abs() < 1e-6, "the cut of 5 is not symmetric: legs {legs:?}");
}

#[test]
fn the_preview_of_a_symmetric_chamfer_is_its_cut() {
    let (p, si, corner, edges) = an_angle();
    let b = p.corner_blend(si, corner, edges, CornerCut::Chamfer(ChamferLegs::equal(3.0))).expect("a cut of 3 fits this corner");
    let cut = (b.ends[0][0] - b.ends[1][0]).hypot(b.ends[0][1] - b.ends[1][1]);
    assert!((cut - 3.0).abs() < 1e-9, "the preview of a cut of 3 is {cut:.4} long");
    // two legs of 3 are still legs of 3
    let two = ChamferLegs { mode: ChamferMode::TwoDist, first: 3.0, second: 3.0 };
    let b = p.corner_blend(si, corner, edges, CornerCut::Chamfer(two)).expect("legs of 3 fit this corner");
    let legs = [(b.ends[0][0] - 20.0).hypot(b.ends[0][1]), (b.ends[1][0] - 20.0).hypot(b.ends[1][1])];
    assert!(legs.iter().all(|l| (l - 3.0).abs() < 1e-9), "two legs of 3 are previewed as {legs:?}");
}
