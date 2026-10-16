//! THE PRIMITIVES: a box, a cylinder, a sphere, a cone, a torus and a prism, each made to the sizes typed at the
//! geometry and each measured against the volume its shape holds.
use qymcad::{Key, Session};
use qymcad_acceptance::build;
use qymcad_acceptance::probe;

use std::f64::consts::PI;

/// A part with nothing in it, ready for the primitive.
fn an_empty_part() -> Session {
    let mut s = Session::start();
    build::into_the_first_part(&mut s);
    s
}

/// The body of the part.
fn body(s: &mut Session) -> qymcad::Solid {
    s.document().bodies.iter().rfind(|b| !b.consumed && !b.sheet).cloned().unwrap_or_else(|| panic!("the part holds no body"))
}

/// Take the tool whose hint is `hint`, type the sizes under their captions and apply.
fn make(s: &mut Session, hint: &str, sizes: &[(&str, &str)]) -> qymcad::Solid {
    let hint = s.word(hint);
    s.press_hint(&hint);
    for (caption, value) in sizes {
        let caption = s.word(caption);
        s.fill(&caption, value);
    }
    s.key(Key::Enter);
    body(s)
}

/// Whether the body holds `want`, to a thousandth of it.
fn holds(b: &qymcad::Solid, want: f64) -> bool {
    (b.volume - want).abs() < want * 1e-3
}

probe! {
    /// A BOX of the three lengths typed.
    fn a_box_is_made_to_the_lengths_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-box-hint", &[("f-length-x", "30"), ("f-width-y", "20"), ("f-height-z", "10")]);
        assert!(holds(&b, 6000.0), "a box of 30 by 20 by 10 holds 6000, and it holds {}", b.volume);
        assert!(b.faces == 6, "a box has six faces, and it has {}", b.faces);
    }
}

probe! {
    /// A CYLINDER of the radius and height typed.
    fn a_cylinder_is_made_to_the_radius_and_height_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-cylinder-hint", &[("f-radius", "5"), ("f-height", "12")]);
        assert!(holds(&b, PI * 25.0 * 12.0), "a cylinder of radius 5 and height 12 holds {}, and it holds {}", PI * 25.0 * 12.0, b.volume);
        assert!((b.max[2] - 12.0).abs() < 1e-3, "the cylinder is not 12 tall: it reaches {}", b.max[2]);
    }
}

probe! {
    /// A SPHERE of the radius typed.
    fn a_sphere_is_made_to_the_radius_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-sphere-hint", &[("f-radius", "8")]);
        assert!(holds(&b, 4.0 / 3.0 * PI * 512.0), "a sphere of radius 8 holds {}, and it holds {}", 4.0 / 3.0 * PI * 512.0, b.volume);
        assert!((b.max[0] - b.min[0] - 16.0).abs() < 0.1, "a sphere of radius 8 is 16 across, and it is {}", b.max[0] - b.min[0]);
    }
}

probe! {
    /// A CONE of the two radii and the height typed.
    fn a_cone_is_made_to_the_radii_and_height_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-cone-hint", &[("f-radius-bottom", "10"), ("f-radius-top", "4"), ("f-height", "15")]);
        let want = PI * 15.0 * (100.0 + 40.0 + 16.0) / 3.0;
        assert!(holds(&b, want), "a cone from 10 to 4 over 15 holds {want}, and it holds {}", b.volume);
    }
}

probe! {
    /// A TORUS of the ring and tube radii typed.
    fn a_torus_is_made_to_the_ring_and_tube_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-torus-hint", &[("f-ring-r", "20"), ("f-tube-r", "5")]);
        let want = 2.0 * PI * PI * 20.0 * 25.0;
        assert!(holds(&b, want), "a torus of ring 20 and tube 5 holds {want}, and it holds {}", b.volume);
        assert!((b.max[0] - 25.0).abs() < 0.1, "the torus reaches 25 from the middle, and it reaches {}", b.max[0]);
    }
}

probe! {
    /// A PRISM of the circumscribed radius and height typed: six sides by default.
    fn a_prism_is_made_to_the_radius_and_height_typed() {
        let mut s = an_empty_part();
        let b = make(&mut s, "tb-prism-hint", &[("f-radius-circ", "10"), ("f-height", "12")]);
        // a regular hexagon in a circle of 10: six triangles of 10 by 10 with the angle of 60 degrees
        let want = 6.0 * 0.5 * 100.0 * (PI / 3.0).sin() * 12.0;
        assert!(holds(&b, want), "a six-sided prism in a circle of 10, 12 tall, holds {want}, and it holds {}", b.volume);
        assert!(b.faces == 8, "a six-sided prism has eight faces, and it has {}", b.faces);
    }
}
