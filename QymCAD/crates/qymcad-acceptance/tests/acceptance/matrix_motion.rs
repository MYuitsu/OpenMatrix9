//! DRIVING A HINGE ACROSS ANGLES AND LIMITS: the hinge between the first and the second of three blocks taken in the
//! list of mates, its angle typed - small, a right angle, below zero, half a turn, past a whole turn; then limits of
//! -30 and 30 set, and an angle past them typed. The second block must turn by the angle typed, and no further than
//! a limit allows.
use qymcad::{Key, Session};
use qymcad_acceptance::contract::fixtures::Fixture;
use qymcad_acceptance::probe;

/// How far the second part is turned from how it was built, in degrees: the angle of its turn read off the trace of
/// its matrix, cos = (trace - 1) / 2.
fn turned(s: &mut Session) -> f64 {
    let p = s.document().parts[1].clone();
    let trace = p.axes[0][0] + p.axes[1][1] + p.axes[2][2];
    ((trace - 1.0) / 2.0).clamp(-1.0, 1.0).acos().to_degrees()
}

/// The angle `a` as a turn reads, 0 to 180.
fn as_turn(a: f64) -> f64 {
    let a = a.rem_euclid(360.0);
    a.min(360.0 - a)
}

/// The hinge "Revolute 1" taken in the list of mates.
fn hinge_taken() -> Session {
    let mut s = Fixture::TwoHingesInAssembly.start();
    let right = s.canvas().max.x;
    let row = s.words_at().into_iter().find(|(w, r)| r.min.x > right && w == "Revolute 1").map(|(_, r)| r).unwrap_or_else(|| panic!("the hinge is not in the list of mates"));
    s.click(row.center());
    s
}

/// Type `angle` into the angle of the hinge taken.
fn drive(s: &mut Session, angle: f64) {
    let caption = s.word("j-angle-lower");
    s.fill(&caption, &format!("{angle}")).key(Key::Enter);
}

probe! {
    budget = 900;
    /// THE ANGLE TYPED TURNS THE BLOCK BY IT: 15, 90, -45, 180 and 400 degrees.
    fn a_hinge_driven_across_angles() {
        let failed: Vec<String> = [15.0, 90.0, -45.0, 180.0, 400.0]
            .into_iter()
            .filter_map(|a: f64| {
                let mut s = hinge_taken();
                drive(&mut s, a);
                let got = turned(&mut s);
                ((got - as_turn(a)).abs() > 0.5).then(|| format!("{a} deg: the block turned {got:.2}, it should turn {:.2}", as_turn(a)))
            })
            .collect();
        assert!(failed.is_empty(), "{} of 5 angles went wrong:\n{}", failed.len(), failed.join("\n"));
    }
}

probe! {
    budget = 900;
    /// LIMITS HOLD THE HINGE: with -30 and 30 set, 20 turns the block by 20, while 60 and -90 leave it no further than 30
    /// - held at the limit or refused in words, never past it.
    fn a_hinge_held_by_its_limits() {
        let failed: Vec<String> = [(20.0, true), (60.0, false), (-90.0, false)]
            .into_iter()
            .filter_map(|(a, inside): (f64, bool)| {
                let mut s = hinge_taken();
                // the limits stand folded under the hinge's own row when there are two hinges: unfolded as a person does
                let row = s.find("Revolute 1", qymcad::pos2(1100.0, 300.0)).expect("the hinge's row");
                let limits = s.find("Limits", row.center()).expect("the limits of the hinge");
                s.click(limits.center());
                let (min, max) = (s.word("j-min"), s.word("j-max"));
                s.toggle(&min);
                s.fill(&min, "-30");
                s.toggle(&max);
                s.fill(&max, "30").key(Key::Enter);
                drive(&mut s, a);
                let got = turned(&mut s);
                if inside {
                    ((got - a.abs()).abs() > 0.5).then(|| format!("{a} deg inside the limits: the block turned {got:.2}"))
                } else {
                    (got > 30.5).then(|| format!("{a} deg past the limits of 30: the block turned {got:.2}; the program says {:?}", s.status()))
                }
            })
            .collect();
        assert!(failed.is_empty(), "{} of 3 angles went wrong:\n{}", failed.len(), failed.join("\n"));
    }
}
