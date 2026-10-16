//! WHAT THE MEASURE IN SPACE TELLS, ACROSS BODIES AND WHAT IS CLICKED: two corners, an edge, two parallel edges, a
//! face and a corner, two faces at an angle, the diameter of a round face and of a round edge, the radius of a
//! rounding, the depth of a hole, the height of a boss. The number said must be the geometry's, to a hundredth - read
//! as a number, not found as letters (40 is not in 140).
use qymcad::Session;
use qymcad_acceptance::bodies::{BLOCK, BOSSED, CHAMFERED, CYLINDER, HOLED, ROUNDED};
use qymcad_acceptance::matrix::{short, spot, take_steps, Body, PartTool, Pick, Spot};
use qymcad_acceptance::probe;

const MEASURE: PartTool = PartTool { hint: "tb-measure3d-hint", caption: "", node: "", counts: false };

/// The numbers written in `said`, as the eye reads them: digits with a point or a comma between them.
fn numbers(said: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in said.chars().chain([' ']) {
        if c.is_ascii_digit() || ((c == '.' || c == ',') && !word.is_empty()) {
            word.push(if c == ',' { '.' } else { c });
        } else if !word.is_empty() {
            if let Ok(v) = word.trim_end_matches('.').parse::<f64>() {
                out.push(v);
            }
            word.clear();
        }
    }
    out
}

/// One measurement: on `body`, `picks` clicked, and one of `any` said.
struct Measured {
    body: &'static Body,
    picks: Vec<Pick>,
    any: Vec<f64>,
    what: &'static str,
}

fn m(body: &'static Body, picks: Vec<Pick>, any: &[f64], what: &'static str) -> Measured {
    Measured { body, picks, any: any.to_vec(), what }
}

/// Run every measurement in a session of its own and fail with all that told another number.
fn run_all(all: Vec<Measured>) {
    let mut failed = Vec::new();
    for c in &all {
        let problem = qymcad_acceptance::refusal(|| {
            let mut s = Session::start();
            (c.body.build)(&mut s);
            let hint = s.word(MEASURE.hint);
            s.press_hint(&hint);
            if let Some(p) = take_steps(&mut s, &MEASURE, &c.picks) {
                panic!("{p}");
            }
            let said = s.status();
            let got = numbers(&said);
            assert!(got.iter().any(|v| c.any.iter().any(|w| (v - w).abs() <= 0.01)), "said {said:?}, not {:?}", c.any);
        });
        if !problem.is_empty() {
            failed.push(format!("{} / {}: {problem}", c.body.name, c.what));
        }
    }
    assert!(failed.is_empty(), "{} of {} measurements did not hold:\n{}", failed.len(), all.len(), failed.join("\n"));
}

const V: fn([f64; 3]) -> Pick = |p| Pick::Vertex(spot(p, p));
const E: fn(Spot) -> Pick = Pick::Edge;
const F: fn(Spot) -> Pick = Pick::Face;

probe! {
    budget = 1800;
    /// THE MEASURE ON A BLOCK: two corners of an edge, two corners across it, an edge, two parallel edges, a face and
    /// a corner off it, two faces at a right angle.
    fn the_measure_on_a_block() {
        run_all(vec![
            m(&BLOCK, vec![V([0.0, 0.0, 10.0]), V([40.0, 0.0, 10.0])], &[40.0], "the corners of the top front edge"),
            m(&BLOCK, vec![V([0.0, 0.0, 10.0]), V([40.0, 0.0, 0.0])], &[1700f64.sqrt()], "two corners across the front face"),
            m(&BLOCK, vec![E(spot([20.0, 0.0, 10.0], [10.0, 0.0, 10.0]))], &[40.0], "the top front edge"),
            m(&BLOCK, vec![E(spot([40.0, 0.0, 5.0], [40.0, 0.0, 2.5]))], &[10.0], "the upright front right edge"),
            m(&BLOCK, vec![E(spot([20.0, 0.0, 10.0], [10.0, 0.0, 10.0])), E(spot([20.0, 0.0, 0.0], [10.0, 0.0, 0.0]))], &[10.0], "the top and bottom front edges, parallel"),
            m(&BLOCK, vec![F(spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])), V([40.0, 0.0, 0.0])], &[10.0], "the top face and a bottom corner"),
            m(&BLOCK, vec![F(spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])), F(spot([20.0, 0.0, 5.0], [10.0, 0.0, 5.0]))], &[90.0], "the top face and the front face"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// THE MEASURE ON ROUND THINGS: the side and the top edge of a cylinder of 20, the wall and the rim of a hole of
    /// 10, the side of a boss of 12, a rounding of radius 2 (its radius, or the diameter of its round face).
    fn the_measure_on_round_things() {
        let c45 = 10.0 * std::f64::consts::FRAC_1_SQRT_2;
        run_all(vec![
            m(&CYLINDER, vec![F(spot([20.0 + c45, 15.0 - c45, 5.0], [20.0 + c45, 15.0 - c45, 3.0]))], &[20.0], "the side of the cylinder"),
            m(&CYLINDER, vec![E(spot([20.0 + c45, 15.0 - c45, 10.0], [20.0 + 10.0 * 0.309, 15.0 - 10.0 * 0.951, 10.0]))], &[20.0], "the top edge of the cylinder"),
            m(&HOLED, vec![F(spot([20.0, 20.0, 7.5], [21.71, 19.7, 7.5]))], &[10.0], "the wall of the hole"),
            m(&HOLED, vec![E(spot([20.0, 10.0, 10.0], [23.54, 11.46, 10.0]))], &[10.0], "the rim of the hole"),
            m(&BOSSED, vec![F(spot([24.243, 10.757, 12.5], [24.243, 10.757, 11.5]))], &[12.0], "the side of the boss"),
            m(&ROUNDED, vec![F(short([20.0, 0.586, 9.414], [10.0, 0.586, 9.414], [20.0, 2.0, 10.0]))], &[2.0, 4.0], "the face of the rounding"),
        ]);
    }
}

probe! {
    budget = 1800;
    /// THE MEASURE OF DEPTHS, HEIGHTS AND SLOPES: the floor of a hole from the top it is bored in, the top of a boss
    /// from the top it stands on, a chamfer against the top (at 45 degrees to it, or 135 across the edge).
    fn the_measure_of_depths_heights_and_slopes() {
        let top_aside = spot([8.0, 8.0, 10.0], [32.0, 8.0, 10.0]);
        run_all(vec![
            m(&HOLED, vec![F(top_aside), F(Spot { at: [20.0, 15.0, 5.0], alt: [21.0, 14.0, 5.0], from: [0, 0, 1], end: None })], &[5.0], "the top and the floor of the hole"),
            m(&BOSSED, vec![F(top_aside), F(spot([20.0, 15.0, 15.0], [18.0, 15.0, 15.0]))], &[5.0], "the top of the block and the top of the boss"),
            m(&CHAMFERED, vec![F(spot([20.0, 15.0, 10.0], [10.0, 8.0, 10.0])), F(short([20.0, 1.0, 9.0], [10.0, 1.0, 9.0], [20.0, 2.0, 10.0]))], &[45.0, 135.0], "the top and the face of the chamfer"),
        ]);
    }
}
