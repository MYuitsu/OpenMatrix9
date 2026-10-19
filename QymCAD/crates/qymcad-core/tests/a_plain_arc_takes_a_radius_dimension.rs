//! A PLAIN ARC TAKES A RADIUS DIMENSION, as a circle does: the radius tool sets 15 and holds it by a driving dimension -
//! one constraint more, one freedom less. Reported behaviour: the radius became 15 with no dimension, constraints 0
//! and 0, freedoms 5 and 5, the arc free to be dragged to any radius.
use qymcad_core::feature::{Purpose, Winding};
use qymcad_core::geom::Point2;
use qymcad_core::model::{Constraint, Project};

#[test]
fn the_radius_of_a_plain_arc_is_held_by_a_dimension() {
    let mut p = Project::default();
    let si = p.new_sketch("S");
    p.add_arc_entity(si, Point2::new(0.0, 0.0), Point2::new(10.0, 0.0), Point2::new(0.0, 10.0), Winding::Ccw, Purpose::Real);
    p.regen_sketch(si);
    let arc = p.sketches[si].entities.last().map(|e| e.id).expect("the arc");
    let (free, _) = p.sketch_dof(si);
    let held = p.sketches[si].constraints.len();
    p.put_arc_radius_dim(si, arc, 15.0);
    let (free_after, redundant) = p.sketch_dof(si);
    assert_eq!(p.sketches[si].constraints.len(), held + 1, "no dimension went on the arc");
    assert!(p.sketches[si].constraints.iter().any(|c| matches!(c, Constraint::Diameter { d, driven: false, .. } if (*d - 15.0).abs() < 1e-9)), "the dimension on the arc does not hold 15");
    assert_eq!((free_after, redundant), (free - 1, 0), "the radius took no freedom away: {free} became {free_after}, redundant {redundant}");
}
