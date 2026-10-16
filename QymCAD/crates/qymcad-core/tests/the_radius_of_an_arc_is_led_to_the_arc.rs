//! THE RADIUS OF AN ARC IS LED TO THE ARC: a radius laid on an arc of a sketch leads its line from the centre through the
//! middle of the arc as it runs, so the line meets the arc that is drawn - below its centre, above it, and on an arc of
//! more than half a turn.
//!
//! Reported behaviour: "the size on an arc is not put on the segment of the arc that is seen" - an arc lying below its
//! centre had its radius led to the right, past the arc.
use qymcad_core::feature::{Purpose, Winding};
use qymcad_core::geom::Point2;
use qymcad_core::model::{Constraint, Project};

/// AN ARC OF RADIUS 10 ABOUT THE ORIGIN: from `a` to `b` the way `winding` runs, and the world direction of its middle.
struct Case {
    name: &'static str,
    a: (f64, f64),
    b: (f64, f64),
    winding: Winding,
    middle: (f64, f64),
}

#[test]
fn the_radius_of_an_arc_is_led_through_its_middle() {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let cases = [
        Case { name: "the lower half", a: (-10.0, 0.0), b: (10.0, 0.0), winding: Winding::Ccw, middle: (0.0, -1.0) },
        Case { name: "the upper half", a: (10.0, 0.0), b: (-10.0, 0.0), winding: Winding::Ccw, middle: (0.0, 1.0) },
        Case { name: "three quarters counter-clockwise", a: (10.0, 0.0), b: (0.0, -10.0), winding: Winding::Ccw, middle: (-h, h) },
        Case { name: "three quarters clockwise", a: (10.0, 0.0), b: (0.0, 10.0), winding: Winding::Cw, middle: (-h, -h) },
    ];
    let mut failures = Vec::new();
    for case in cases {
        let mut p = Project::default();
        p.new_document();
        let si = p.new_sketch("S");
        p.add_arc_entity(si, Point2::new(0.0, 0.0), Point2::new(case.a.0, case.a.1), Point2::new(case.b.0, case.b.1), case.winding, Purpose::Real);
        let eid = p.sketches[si].entities.last().expect("the arc").id;
        p.put_arc_radius_dim(si, eid, 10.0);
        let off = p.sketches[si].constraints.iter().find_map(|c| match c {
            Constraint::Diameter { off, diam: false, .. } => Some(*off),
            _ => None,
        });
        let Some(off) = off else {
            failures.push(format!("{}: no radius laid", case.name));
            continue;
        };
        // the angle is on the screen, where y runs down
        let led = (off.cos(), -off.sin());
        let along = led.0 * case.middle.0 + led.1 * case.middle.1;
        if along < 0.999 {
            failures.push(format!("{}: the radius is led to ({:.3}, {:.3}), the middle of the arc is at ({:.3}, {:.3})", case.name, led.0, led.1, case.middle.0, case.middle.1));
        }
    }
    assert!(failures.is_empty(), "the radius of an arc:\n{}", failures.join("\n"));
}
