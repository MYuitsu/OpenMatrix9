//! A SET OF CORNERS COMES OUT AS THE CORNERS CUT ONE BY ONE: a fillet or a chamfer laid on a set of corners in one go
//! (`cut_corner_set`, which rebuilds and solves the sketch once, at its end) leaves the sketch as the same cut laid on
//! each corner by itself, one after another - the same points, curves and constraints.
mod sketch_kinds;

use qymcad_core::feature::ChamferMode;
use qymcad_core::model::{ChamferLegs, CornerAt, CornerCut, FilletSize, Project};
use sketch_kinds::{build, Kind};

fn corners(p: &Project) -> Vec<CornerAt> {
    p.sketches[0]
        .points
        .iter()
        .filter_map(|q| match p.vertex_pairs(0, q.id)[..] {
            [pair] => Some(CornerAt { point: q.id, pair }),
            _ => None,
        })
        .collect()
}

/// The sketch as text: every point to 1e-9 mm, every entity, every constraint.
fn state(p: &Project) -> String {
    let s = &p.sketches[0];
    let points: Vec<String> = s.points.iter().map(|q| format!("{} ({:.9}, {:.9})", q.id, q.x, q.y)).collect();
    format!("{points:?}\n{:?}\n{:?}", s.entities.iter().map(|e| (e.id, format!("{:?}", e.kind))).collect::<Vec<_>>(), s.constraints.iter().map(|c| format!("{c:?}")).collect::<Vec<_>>())
}

#[test]
fn a_set_of_corners_is_the_corners_one_by_one() {
    let mut p = build(Kind::Rectangles, 6);
    p.solve_sketch(0);
    let cuts = [
        ("a fillet", CornerCut::Fillet(FilletSize::radius(1.0))),
        ("a symmetric chamfer", CornerCut::Chamfer(ChamferLegs::equal(1.0))),
        ("a chamfer of two legs", CornerCut::Chamfer(ChamferLegs { mode: ChamferMode::TwoDist, first: 1.0, second: 1.5 })),
        ("a chamfer of a leg and an angle", CornerCut::Chamfer(ChamferLegs { mode: ChamferMode::DistAngle, first: 1.0, second: 40.0 })),
    ];
    let mut failures = Vec::new();
    for (what, cut) in cuts {
        let all = corners(&p);
        let mut set = p.clone();
        let laid = set.cut_corner_set(0, &all, cut, None);
        let mut one_by_one = p.clone();
        let mut laid_one = 0;
        for c in &all {
            laid_one += one_by_one.cut_corner_set(0, &[*c], cut, None);
        }
        if laid != all.len() || laid_one != all.len() {
            failures.push(format!("{what}: {laid} of {} corners in a set, {laid_one} one by one", all.len()));
        } else if state(&set) != state(&one_by_one) {
            failures.push(format!("{what}: the set differs from the corners one by one"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
