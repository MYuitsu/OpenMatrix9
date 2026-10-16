//! WHAT A SKETCH BECOMES, ACROSS PROFILES, OPERATIONS AND VALUES: an extrusion to a length, symmetric, to two sides,
//! flipped, through all; added, cut, intersected; of a rectangle, a circle, a rectangle round a circle, two squares
//! apart, a square on the block and a circle over its edge; a revolution about either axis, whole and in part, of a
//! rectangle and of a circle off the axis or across it. Values ordinary, small, large, zero and below. What each case
//! must come to is worked out from the geometry.
use std::f64::consts::PI;

use qymcad_acceptance::bodies::{ACROSS_THE_AXIS, CIRCLE, LOFT_CIRCLES, LOFT_SAME, LOFT_SQUARES, NESTED, OVER_THE_EDGE, RECTANGLE, SQUARE_ON_TOP, SWEEP_ARC, SWEEP_ROUND, SWEEP_STRAIGHT, TWO_APART};
use qymcad_acceptance::chains::Tier;
use qymcad_acceptance::matrix::{run_all, Added, Body, Case, Expect, PartTool, Pick};
use qymcad_acceptance::probe;

/// An extrusion making the first body of a part, and one working on the body there is.
const EXTRUDE_NEW: PartTool = PartTool { hint: "tb-extrude-hint", caption: "f-length", node: "Extrude", counts: false };
const EXTRUDE_ON: PartTool = PartTool { hint: "tb-extrude-hint", caption: "f-length", node: "Combine", counts: false };
const REVOLVE: PartTool = PartTool { hint: "tb-revolve-hint", caption: "cmd-angle", node: "Revolve", counts: false };
const SWEEP: PartTool = PartTool { hint: "tb-sweep-hint", caption: "", node: "Sweep", counts: false };
const LOFT: PartTool = PartTool { hint: "tb-loft-hint", caption: "", node: "Loft", counts: false };

fn tol(change: f64) -> f64 {
    (change.abs() * 1e-3).max(1e-4)
}

fn built(change: f64, added: Added) -> Expect {
    Expect::Built { change, tol: tol(change), added: Some(added) }
}

fn faces(plane: i32, cylinder: i32, blend: i32) -> Added {
    Added { plane, cylinder, cone: 0, blend }
}

/// A case on `body`: the words and letters `steps` pressed on the bar, `value` typed into the tool's own field and
/// `more` into others.
fn case(body: &'static Body, steps: &[Pick], value: f64, more: &[(&'static str, f64)], expect: Expect, what: &str) -> Case {
    Case {
        body,
        picks: steps.to_vec(),
        bar: Vec::new(),
        value: format!("{value}"),
        more: more.iter().map(|(c, v)| (*c, format!("{v}"))).collect(),
        expect,
        what: what.to_string(),
        then: None,
        after: Vec::new(),
    }
}

fn round_trips() -> bool {
    Tier::asked() == Tier::Release
}

/// THE FIRST BODY OF A PART: a prism of the profile by the length - one way, both ways in half, two ways by two
/// lengths, flipped under the plane; the rectangle round a circle makes a plate with a hole; two squares apart would
/// make a part of two pieces, which it cannot be.
fn first_body_cases() -> Vec<Case> {
    let block = |h: f64| built(1200.0 * h, faces(6, 0, 0));
    let mut out = vec![
        case(&RECTANGLE, &[], 10.0, &[], block(10.0), "the rectangle, 10 up"),
        case(&RECTANGLE, &[], 0.1, &[], block(0.1), "the rectangle, 0.1 up"),
        case(&RECTANGLE, &[], 1000.0, &[], block(1000.0), "the rectangle, 1000 up"),
        case(&RECTANGLE, &[Pick::Bar("cmd-symmetric")], 10.0, &[], block(10.0), "the rectangle, 10 symmetric"),
        case(&RECTANGLE, &[Pick::Bar("cmd-two-sides")], 10.0, &[("f-second-side", 4.0)], block(14.0), "the rectangle, 10 up and 4 down"),
        case(&RECTANGLE, &[Pick::Bar("cmd-flip-btn")], 10.0, &[], block(10.0), "the rectangle, 10 flipped under the plane"),
        case(&CIRCLE, &[], 10.0, &[], built(PI * 100.0 * 10.0, faces(2, 1, 0)), "the circle, 10 up"),
        // several contours are picked by a click inside each, then Enter goes on to the size
        case(
            &NESTED,
            &[Pick::Space([5.0, 5.0, 0.0]), Pick::Enter],
            10.0,
            &[],
            built((1200.0 - PI * 25.0) * 10.0, faces(6, 1, 0)),
            "the ring between the rectangle and the circle, 10 up: a plate with a hole",
        ),
        case(&NESTED, &[Pick::Space([5.0, 5.0, 0.0]), Pick::Space([20.0, 15.0, 0.0]), Pick::Enter], 10.0, &[], block(10.0), "the ring and the circle, 10 up: the whole block"),
        // the first body may be in pieces, as text is (decided 26.09): two squares apart are two pieces of one part
        case(
            &TWO_APART,
            &[Pick::Space([5.0, 5.0, 0.0]), Pick::Space([25.0, 5.0, 0.0]), Pick::Enter],
            10.0,
            &[],
            Expect::Pieces { change: 2000.0, tol: 1.0, pieces: 2 },
            "two squares apart: two pieces",
        ),
    ];
    for (v, what) in [(0.0, "nothing"), (-5.0, "below zero")] {
        out.push(case(&RECTANGLE, &[], v, &[], Expect::Refused, &format!("the rectangle, {what}")));
    }
    out
}

/// ON THE BODY THERE IS: the square of 10 on the top of the block added, cut, cut through all, cut deeper than the
/// block, intersected into it; symmetric about the top, half of it goes out and half in; the circle over the front
/// edge adds a whole boss and cuts only its half inside the block.
fn on_the_body_cases() -> Vec<Case> {
    let prism = |h: f64| 100.0 * h;
    let half_disc = PI * 25.0 / 2.0;
    vec![
        case(&SQUARE_ON_TOP, &[], 5.0, &[], built(prism(5.0), faces(5, 0, 0)), "the square added 5"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut")], 5.0, &[], built(-prism(5.0), faces(5, 0, 0)), "the square cut 5"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut")], 0.1, &[], built(-prism(0.1), faces(5, 0, 0)), "the square cut 0.1"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut"), Pick::Bar("cmd-through-all")], 5.0, &[], built(-prism(10.0), faces(4, 0, 0)), "the square cut through all"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut")], 12.0, &[], built(-prism(10.0), faces(4, 0, 0)), "the square cut 12, deeper than the block"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-intersect"), Pick::Bar("cmd-flip-btn")], 5.0, &[], built(prism(5.0) - 12000.0, faces(0, 0, 0)), "the square intersected 5 into the block"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-symmetric")], 10.0, &[], built(prism(5.0), faces(5, 0, 0)), "the square added 10 symmetric: half of it out"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut"), Pick::Bar("cmd-symmetric")], 10.0, &[], built(-prism(5.0), faces(5, 0, 0)), "the square cut 10 symmetric: half of it in"),
        // the half of the boss out over the front has its own floor, at the height of the top
        case(&OVER_THE_EDGE, &[], 5.0, &[], built(PI * 25.0 * 5.0, faces(2, 1, 0)), "the circle over the edge added 5: a whole boss"),
        case(&OVER_THE_EDGE, &[Pick::Bar("cmd-cut")], 5.0, &[], built(-half_disc * 5.0, faces(1, 1, 0)), "the circle over the edge cut 5: its half inside"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-intersect")], 5.0, &[], Expect::Refused, "the square intersected 5 up, with nothing of the block there"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut"), Pick::Bar("cmd-flip-btn")], 5.0, &[], Expect::Refused, "the square cut 5 up, away from the block"),
        case(&SQUARE_ON_TOP, &[Pick::Bar("cmd-cut")], 0.0, &[], Expect::Refused, "the square cut by nothing"),
        case(&SQUARE_ON_TOP, &[], 0.0, &[], Expect::Refused, "the square added by nothing"),
    ]
}

/// REVOLUTION: the rectangle about the X axis is a cylinder of radius 30 and length 40, about Y one of radius 40 and
/// height 30; a quarter turn a quarter of it, cut by two planes. A circle off the axis sweeps a ring (the circle's area
/// round the circle its middle runs along); one across the axis would pass through itself.
fn revolve_cases() -> Vec<Case> {
    let about_x = PI * 900.0 * 40.0;
    let about_y = PI * 1600.0 * 30.0;
    let y = Pick::Letter("Y");
    vec![
        case(&RECTANGLE, &[], 360.0, &[], built(about_x, faces(2, 1, 0)), "the rectangle about X, a whole turn"),
        case(&RECTANGLE, &[y], 360.0, &[], built(about_y, faces(2, 1, 0)), "the rectangle about Y, a whole turn"),
        case(&RECTANGLE, &[], 90.0, &[], built(about_x / 4.0, faces(4, 1, 0)), "the rectangle about X, a quarter turn"),
        case(&RECTANGLE, &[], 1.0, &[], built(about_x / 360.0, faces(4, 1, 0)), "the rectangle about X, one degree"),
        // the other way is the button of the bar, not a sign: an angle is an opening, 0 < v <= 360
        case(
            &RECTANGLE,
            &[Pick::Bar("cmd-flip-btn")],
            90.0,
            &[],
            Expect::Magnitude { change: about_x / 4.0, tol: tol(about_x / 4.0), added: Some(faces(4, 1, 0)) },
            "the rectangle about X, a quarter turn the other way",
        ),
        case(&RECTANGLE, &[], -90.0, &[], Expect::Refused, "the rectangle about X, a turn below zero"),
        case(&CIRCLE, &[], 360.0, &[], built(2.0 * PI * 15.0 * PI * 100.0, faces(0, 0, 1)), "the circle about X, a whole turn: a ring"),
        case(&CIRCLE, &[y], 360.0, &[], built(2.0 * PI * 20.0 * PI * 100.0, faces(0, 0, 1)), "the circle about Y, a whole turn: a ring"),
        case(&ACROSS_THE_AXIS, &[], 360.0, &[], Expect::Refused, "a circle across the X axis, which would pass through itself"),
        case(&RECTANGLE, &[], 0.0, &[], Expect::Refused, "the rectangle about X, no turn"),
        case(&RECTANGLE, &[], 400.0, &[], Expect::Refused, "the rectangle about X, more than a whole turn"),
    ]
}

/// SWEEP: the profile carried along the path. Along a straight line it is a prism; along a quarter circle of 30 about
/// an axis in the plane of the profile it is the profile turned a quarter about that axis - its area by the arc its
/// middle runs, 30 by pi / 2 (Pappus).
fn sweep_cases() -> Vec<Case> {
    let path = [Pick::SketchRow(1)];
    let quarter = 30.0 * PI / 2.0;
    vec![
        case(&SWEEP_STRAIGHT, &path, 0.0, &[], built(4000.0, faces(6, 0, 0)), "a square along a straight line"),
        case(&SWEEP_ARC, &path, 0.0, &[], Expect::Built { change: 100.0 * quarter, tol: tol(100.0 * quarter), added: None }, "a square along a quarter circle"),
        case(&SWEEP_ROUND, &path, 0.0, &[], Expect::Built { change: PI * 25.0 * quarter, tol: tol(PI * 25.0 * quarter), added: None }, "a circle along a quarter circle"),
    ]
}

/// LOFT: from the first section to the next, 30 above. Between two squares about one middle, and two circles about one
/// middle, the body is a frustum - the height over 6 by the two ends and four times the middle; between the same
/// square twice, a prism.
fn loft_cases() -> Vec<Case> {
    let next = [Pick::SketchRow(1)];
    let frustum = |a: f64, m: f64, b: f64| 30.0 / 6.0 * (a + 4.0 * m + b);
    let squares = frustum(100.0, 225.0, 400.0);
    let circles = frustum(PI * 100.0, PI * 56.25, PI * 25.0);
    vec![
        case(&LOFT_SQUARES, &next, 0.0, &[], Expect::Built { change: squares, tol: tol(squares), added: None }, "a square of 10 to one of 20"),
        case(&LOFT_CIRCLES, &next, 0.0, &[], Expect::Built { change: circles, tol: tol(circles), added: None }, "a circle of 10 to one of 5"),
        case(&LOFT_SAME, &next, 0.0, &[], Expect::Built { change: 3000.0, tol: tol(3000.0), added: None }, "a square to the same square"),
    ]
}

macro_rules! sketch_matrix {
    ($name:ident, $doc:literal, $tool:expr, $cases:expr) => {
        probe! {
            budget = 1800;
            #[doc = $doc]
            fn $name() {
                run_all(&$tool, &$cases(), round_trips());
            }
        }
    };
}

sketch_matrix!(
    first_bodies_extruded,
    "EXTRUSION MAKING THE FIRST BODY: to a length, symmetric, two sides, flipped; a rectangle, a circle, a rectangle round a circle; two squares apart, zero and below refused.",
    EXTRUDE_NEW,
    first_body_cases
);
sketch_matrix!(
    extrusions_on_a_body,
    "EXTRUSION ON THE BLOCK: added, cut, through all, deeper than the block, intersected, symmetric, over an edge; what makes nothing, and nothing, refused.",
    EXTRUDE_ON,
    on_the_body_cases
);
sketch_matrix!(revolutions, "REVOLUTION about X and Y, whole, in part, the other way; a circle off the axis; across the axis, no turn and more than one refused.", REVOLVE, revolve_cases);
sketch_matrix!(sweeps, "SWEEP of a square and a circle along a straight line and a quarter circle.", SWEEP, sweep_cases);
sketch_matrix!(lofts, "LOFT between two squares, two circles and the same square twice.", LOFT, loft_cases);
