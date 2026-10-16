//! ROUNDING AND CUTTING EDGES ACROSS BODIES, PICKS AND VALUES: one edge, two apart, two meeting, the four of a face,
//! the three of a corner; straight edges of a block, round edges of a cylinder and of a hole, edges that run along an
//! earlier chamfer or rounding; values ordinary, small, near the limit the geometry sets and past it, zero and below.
//! What each case must come to is worked out from the geometry in `qymcad_acceptance::matrix`.
//!
//! The level before a release adds undo and redo, save and open, and rebuilding everything to every body built.
use std::f64::consts::PI;

use qymcad::Session;
use qymcad_acceptance::chains::Tier;
use qymcad_acceptance::bodies::{notched, BLOCK, BOSSED, CHAMFERED, CYLINDER, HOLED, NOTCHED, ROUNDED, WEDGE};
use qymcad_acceptance::matrix::{
    chamfer_area, chamfer_corner, chamfer_mitre, fillet_area, fillet_centre, fillet_corner, fillet_mitre, run_all, short, spot, Added, Body, Case, Expect, PartTool, Pick, Spot, Then,
};
use qymcad_acceptance::probe;

const FILLET: PartTool = PartTool { hint: "tb-fillet-body-hint", caption: "f-radius", node: "Fillet", counts: true };
const CHAMFER: PartTool = PartTool { hint: "tb-chamfer-body-hint", caption: "f-leg", node: "Chamfer", counts: true };

/// The edges of the 40 x 30 x 10 block, by a point in the middle of each and its length; the back and the left
/// ones are looked at from their own side when the view as it opens will not do.
const TOP_FRONT: (Spot, f64) = (spot([20.0, 0.0, 10.0], [10.0, 0.0, 10.0]), 40.0);
const TOP_RIGHT: (Spot, f64) = (spot([40.0, 15.0, 10.0], [40.0, 7.5, 10.0]), 30.0);
const TOP_BACK: (Spot, f64) = (Spot { at: [20.0, 30.0, 10.0], alt: [10.0, 30.0, 10.0], from: [1, 1, 1], end: None }, 40.0);
const TOP_LEFT: (Spot, f64) = (Spot { at: [0.0, 15.0, 10.0], alt: [0.0, 7.5, 10.0], from: [-1, -1, 1], end: None }, 30.0);
const FRONT_RIGHT: (Spot, f64) = (spot([40.0, 0.0, 5.0], [40.0, 0.0, 2.5]), 10.0);
const BOTTOM_FRONT: (Spot, f64) = (spot([20.0, 0.0, 0.0], [10.0, 0.0, 0.0]), 40.0);

/// How close the volume must come: the numbers of an exact body are near to exact, and a small rounding takes
/// little - so the tolerance follows what is taken, never above a thousandth of it.
fn tol(change: f64) -> f64 {
    (change.abs() * 1e-3).max(1e-4)
}

fn built(change: f64, added: Added) -> Expect {
    Expect::Built { change, tol: tol(change), added: Some(added) }
}

/// The case, made 15 tall afterwards: what the tool takes does not depend on the height, and the body it stood on
/// holds `base` now.
fn same_when_taller(mut c: Case, base: f64) -> Case {
    if let Expect::Built { change, .. } = c.expect {
        c.then = taller(base, change);
    }
    c
}

fn case(body: &'static Body, picks: &[Spot], value: f64, expect: Expect, what: &str) -> Case {
    Case { body, picks: picks.iter().map(|p| Pick::Edge(*p)).collect(), bar: Vec::new(), value: format!("{value}"), more: Vec::new(), expect, what: what.to_string(), then: None, after: Vec::new() }
}

/// The block made 15 tall instead of 10: its extrusion reopened and the length typed anew.
fn taller(base: f64, change: f64) -> Option<Then> {
    Some(Then { row: "cmd-extrude", caption: "f-length", value: "15", base, change, tol: tol(change) })
}

/// Faces a rounding of straight edges adds: a cylinder for each, and a corner patch where three meet.
fn cylinders(n: i32, corners: i32) -> Added {
    Added { cylinder: n, blend: corners, ..Default::default() }
}

/// Faces a chamfer adds: a plane for each edge.
fn planes(n: i32) -> Added {
    Added { plane: n, ..Default::default() }
}

/// What a tool takes of a block: the edges picked, how long they run together on a block `h` tall, how many there
/// are, and whether they meet in a corner of three.
struct Take {
    what: &'static str,
    picks: Vec<Spot>,
    length: fn(f64) -> f64,
    edges: i32,
    closes_a_corner: bool,
}

/// THE STRAIGHT EDGES OF THE BLOCK, rounded (`round`) or chamfered, with a value `v`: what is taken, worked out from
/// the lengths - each edge its waste, less what two meeting edges share, and the corner of three its own. Then the
/// block is made 15 tall above the tool, and the same edges must be done on it: the upright edge is 15 long now.
fn block_cases(round: bool, v: f64) -> Vec<Case> {
    let area = if round { fillet_area(v, PI / 2.0) } else { chamfer_area(v, PI / 2.0) };
    let mitre = if round { fillet_mitre(v) } else { chamfer_mitre(v, v) };
    // a rounded corner of three is closed by a patch of a ball, a chamfered one by a triangle
    let added = |n: i32, corner: bool| if round { cylinders(n, corner as i32) } else { planes(n + corner as i32) };
    let corner = if round { fillet_corner(v) } else { chamfer_corner(v) };
    // what the tool takes of a block `h` tall
    let takes: [Take; 7] = [
        Take { what: "the top front edge", picks: vec![TOP_FRONT.0], length: |_| 40.0, edges: 1, closes_a_corner: false },
        Take { what: "the upright front right edge", picks: vec![FRONT_RIGHT.0], length: |h| h, edges: 1, closes_a_corner: false },
        Take { what: "the bottom front edge", picks: vec![BOTTOM_FRONT.0], length: |_| 40.0, edges: 1, closes_a_corner: false },
        Take { what: "the top front and top back edges, apart", picks: vec![TOP_FRONT.0, TOP_BACK.0], length: |_| 80.0, edges: 2, closes_a_corner: false },
        Take { what: "the top front and top right edges, meeting", picks: vec![TOP_FRONT.0, TOP_RIGHT.0], length: |_| 70.0, edges: 2, closes_a_corner: false },
        Take { what: "the four edges of the top", picks: vec![TOP_FRONT.0, TOP_RIGHT.0, TOP_BACK.0, TOP_LEFT.0], length: |_| 140.0, edges: 4, closes_a_corner: false },
        Take { what: "the three edges of the top front right corner", picks: vec![TOP_FRONT.0, TOP_RIGHT.0, FRONT_RIGHT.0], length: |h| 70.0 + h, edges: 3, closes_a_corner: true },
    ];
    takes
        .into_iter()
        .map(|Take { what, picks, length, edges: n, closes_a_corner: three }| {
            let meets = match n {
                2 if what.contains("meeting") => 1.0,
                4 => 4.0,
                _ => 0.0,
            };
            let take = |h: f64| if three { -((length(h) - 3.0 * v) * area + corner) } else { -(length(h) * area - meets * mitre) };
            let mut c = case(&BLOCK, &picks, v, built(take(10.0), added(n, three)), what);
            c.then = taller(40.0 * 30.0 * 15.0, take(15.0));
            c
        })
        .collect()
}

/// VALUES THE BLOCK REFUSES, with the edge they are typed for: past the 10 of the front face, zero, below zero.
fn block_refusals() -> Vec<(Vec<Spot>, f64, &'static str)> {
    vec![
        (vec![TOP_FRONT.0], 10.5, "the top front edge, past the 10 of the front face"),
        (vec![TOP_FRONT.0, TOP_RIGHT.0, TOP_BACK.0, TOP_LEFT.0], 12.0, "the four edges of the top, past the 10 of the sides"),
        (vec![TOP_FRONT.0], 0.0, "the top front edge, nothing"),
        (vec![TOP_FRONT.0], -2.0, "the top front edge, below zero"),
    ]
}

/// The circle of radius `r` about (20, 15) at the top: at the front right and further round to the front.
fn rim(r: f64) -> Spot {
    let at = |a: f64| [20.0 + r * a.cos(), 15.0 + r * a.sin(), 10.0];
    spot(at(-PI / 4.0), at(-PI * 0.4))
}

/// A round edge: the top of the cylinder (material inside the circle of 10) or the rim of the hole (material outside
/// the circle of 5). What is taken is the area of the section swept round the circle its centre runs along.
fn round_edge_cases(round: bool, v: f64) -> Vec<Case> {
    let (area, off) = if round { (fillet_area(v, PI / 2.0), fillet_centre(v)) } else { (chamfer_area(v, PI / 2.0), v / 3.0) };
    let added = if round { Added { blend: 1, ..Default::default() } } else { Added { cone: 1, ..Default::default() } };
    vec![
        same_when_taller(case(&CYLINDER, &[rim(10.0)], v, built(-2.0 * PI * (10.0 - off) * area, added), "the top edge of the cylinder"), PI * 100.0 * 15.0),
        same_when_taller(case(&HOLED, &[rim(5.0)], v, built(-2.0 * PI * (5.0 + off) * area, added), "the rim of the hole"), 18000.0 - PI * 25.0 * 5.0),
    ]
}

/// EDGES ALONG AN EARLIER CHAMFER: the chamfer's edge on the top and on the front turn by 45 degrees; the top right
/// edge now ends on the chamfer; a chamfer on the chamfer.
fn after_chamfer_cases(round: bool) -> Vec<Case> {
    let top = spot([20.0, 2.0, 10.0], [10.0, 2.0, 10.0]);
    let front = spot([20.0, 0.0, 8.0], [10.0, 0.0, 8.0]);
    // the chamfer of 2 along the 40 of the top front edge took 80 of the block
    let base = 18000.0 - 80.0;
    let cases = if round {
        let a = |r: f64| fillet_area(r, PI / 4.0);
        vec![
            case(&CHAMFERED, &[top], 2.0, built(-40.0 * a(2.0), cylinders(1, 0)), "the edge between the chamfer and the top"),
            case(&CHAMFERED, &[front], 2.0, built(-40.0 * a(2.0), cylinders(1, 0)), "the edge between the chamfer and the front"),
            case(&CHAMFERED, &[top], 6.0, built(-40.0 * a(6.0), cylinders(1, 0)), "the edge between the chamfer and the top, near the width of the chamfer"),
            case(&CHAMFERED, &[top, front], 1.0, built(-80.0 * a(1.0), cylinders(2, 0)), "both edges of the chamfer"),
        ]
    } else {
        let a = |d: f64| chamfer_area(d, 3.0 * PI / 4.0);
        vec![
            case(&CHAMFERED, &[top], 1.0, built(-40.0 * a(1.0), planes(1)), "a chamfer on the edge between the chamfer and the top"),
            case(&CHAMFERED, &[top], 2.5, built(-40.0 * a(2.5), planes(1)), "a chamfer on the edge between the chamfer and the top, near its width"),
            case(&CHAMFERED, &[TOP_RIGHT.0], 2.0, built(-(30.0 * chamfer_area(2.0, PI / 2.0) - chamfer_mitre(2.0, 2.0)), planes(1)), "the top right edge, which ends on the chamfer"),
            case(&CHAMFERED, &[TOP_RIGHT.0], 1.0, built(-(30.0 * chamfer_area(1.0, PI / 2.0) - chamfer_mitre(2.0, 1.0)), planes(1)), "the top right edge with a smaller leg, ending on the chamfer"),
        ]
    };
    cases.into_iter().map(|c| same_when_taller(c, base)).collect()
}

/// AN EDGE THAT RUNS ON INTO AN EARLIER ROUNDING: the top right edge ends where the rounding of the top front edge
/// begins, and runs on round it tangent - down the quarter circle of 2 on the right face, then down the upright front
/// right edge. The tool takes the whole tangent chain, as it does in the professional systems: 28 of the straight
/// edge, the pi of the quarter circle and the 8 left of the upright edge. The curve of the quarter circle shifts what
/// is taken by less than the waste of one mm of the edge.
fn after_rounding_cases(round: bool) -> Vec<Case> {
    let area = if round { fillet_area(1.0, PI / 2.0) } else { chamfer_area(1.0, PI / 2.0) };
    let chain = |h: f64| 28.0 + PI + (h - 2.0);
    let mut c = case(&ROUNDED, &[TOP_RIGHT.0], 1.0, Expect::Built { change: -chain(10.0) * area, tol: area, added: None }, "the top right edge, which runs on into the rounding");
    // made 15 tall, the upright part of the chain is 13 long
    c.then = Some(Then { row: "cmd-extrude", caption: "f-length", value: "15", base: 18000.0 - 40.0 * fillet_area(2.0, PI / 2.0), change: -chain(15.0) * area, tol: area });
    vec![c]
}

/// THE NOTCH: the two pieces of the top front edge (15 each), the upright front edges of the notch (5 each), a
/// piece of the top front edge with the upright edge it meets (a corner whose third edge stays sharp), and the hollow
/// edges of the floor - the back one and a side one, 10 each - which gain what the others lose.
fn notch_cases(round: bool, v: f64) -> Vec<Case> {
    let area = if round { fillet_area(v, PI / 2.0) } else { chamfer_area(v, PI / 2.0) };
    let mitre = if round { fillet_mitre(v) } else { chamfer_mitre(v, v) };
    let added = |n: i32| if round { cylinders(n, 0) } else { planes(n) };
    let piece_left = spot([7.5, 0.0, 10.0], [4.0, 0.0, 10.0]);
    let piece_right = spot([32.5, 0.0, 10.0], [36.0, 0.0, 10.0]);
    let upright_left = short([15.0, 0.0, 7.5], [15.0, 0.0, 6.5], [15.0, 0.0, 10.0]);
    let upright_right = short([25.0, 0.0, 7.5], [25.0, 0.0, 6.5], [25.0, 0.0, 10.0]);
    let floor_back = spot([20.0, 10.0, 5.0], [18.0, 10.0, 5.0]);
    let floor_side = Spot { at: [15.0, 5.0, 5.0], alt: [15.0, 3.0, 5.0], from: [1, -1, 1], end: None };
    // the notch took 10 x 10 x 5 of the block
    let base = 18000.0 - 500.0;
    vec![
        same_when_taller(case(&NOTCHED, &[piece_left, piece_right], v, built(-30.0 * area, added(2)), "the two pieces of the top front edge"), base),
        same_when_taller(case(&NOTCHED, &[upright_left, upright_right], v, built(-10.0 * area, added(2)), "the upright front edges of the notch"), base),
        same_when_taller(case(&NOTCHED, &[piece_left, upright_left], v, built(-(20.0 * area - mitre), added(2)), "a piece of the top front edge and the upright edge it meets"), base),
        same_when_taller(case(&NOTCHED, &[floor_back], v, built(10.0 * area, added(1)), "the hollow back edge of the floor of the notch"), base),
        same_when_taller(case(&NOTCHED, &[floor_side], v, built(10.0 * area, added(1)), "a hollow side edge of the floor of the notch"), base),
    ]
}

/// THE BOSS: the hollow round edge where it meets the top (what is gained lies outside the circle of 6) and the round
/// edge of its own top (what is taken lies inside it).
fn boss_cases(round: bool, v: f64) -> Vec<Case> {
    let (area, off) = if round { (fillet_area(v, PI / 2.0), fillet_centre(v)) } else { (chamfer_area(v, PI / 2.0), v / 3.0) };
    let added = if round { Added { blend: 1, ..Default::default() } } else { Added { cone: 1, ..Default::default() } };
    let circle = |r: f64, z: f64, a: f64| [20.0 + r * a.cos(), 15.0 + r * a.sin(), z];
    let base_rim = spot(circle(6.0, 10.0, -PI / 4.0), circle(6.0, 10.0, -PI * 0.4));
    let top_rim = spot(circle(6.0, 15.0, -PI / 4.0), circle(6.0, 15.0, -PI * 0.4));
    let base = 18000.0 + PI * 36.0 * 5.0;
    vec![
        same_when_taller(case(&BOSSED, &[base_rim], v, built(2.0 * PI * (6.0 + off) * area, added), "the hollow round edge where the boss meets the top"), base),
        same_when_taller(case(&BOSSED, &[top_rim], v, built(-2.0 * PI * (6.0 - off) * area, added), "the round edge of the top of the boss"), base),
    ]
}

/// A ROUNDING THAT RUNS ON INTO A SMALLER ONE: the top right edge of the block rounded 2 at its front runs on, tangent,
/// round the quarter circle of 2. A radius of 1.5 rolls round it. One of 4 cannot roll round a circle of 2: the
/// professional systems either refuse it or let it overflow onto the faces next to it and take the smaller rounding
/// in - either will do, a broken body will not.
fn into_a_smaller_rounding() -> Vec<Case> {
    let area = |r: f64| fillet_area(r, PI / 2.0);
    let chain = 28.0 + PI + 8.0;
    vec![
        case(&ROUNDED, &[TOP_RIGHT.0], 1.5, Expect::Built { change: -chain * area(1.5), tol: area(1.5), added: None }, "the top right edge at 1.5, running on round the rounding of 2"),
        case(&ROUNDED, &[TOP_RIGHT.0], 4.0, Expect::Either, "the top right edge at 4, which cannot roll round the rounding of 2"),
    ]
}

fn refusals(round: bool) -> Vec<Case> {
    let mut out: Vec<Case> = block_refusals().into_iter().map(|(picks, v, what)| case(&BLOCK, &picks, v, Expect::Refused, what)).collect();
    out.push(case(&CYLINDER, &[rim(10.0)], 10.5, Expect::Refused, "the top edge of the cylinder, past its 10 of height and radius"));
    out.push(case(&HOLED, &[rim(5.0)], 5.5, Expect::Refused, "the rim of the hole, past its 5 of depth"));
    out.push(case(&CHAMFERED, &[spot([20.0, 2.0, 10.0], [10.0, 2.0, 10.0])], if round { 8.0 } else { 3.5 }, Expect::Refused, "the edge between the chamfer and the top, past its width"));
    out.push(case(&NOTCHED, &[spot([20.0, 10.0, 5.0], [18.0, 10.0, 5.0])], 5.5, Expect::Refused, "the hollow back edge of the floor of the notch, past the 5 of its wall"));
    let rim = spot([20.0 + 6.0 * (PI / 4.0).cos(), 15.0 - 6.0 * (PI / 4.0).sin(), 10.0], [20.0 + 6.0 * (PI * 0.4).cos(), 15.0 - 6.0 * (PI * 0.4).sin(), 10.0]);
    out.push(case(&BOSSED, &[rim], 5.5, Expect::Refused, "the hollow edge round the boss, past the 5 of its height"));
    out
}

/// The level before a release checks the round trips as well.
fn round_trips() -> bool {
    Tier::asked() == Tier::Release
}

/// Values for the edges of the block: ordinary, small, and near the 10 of its sides.
const BLOCK_VALUES: [f64; 3] = [2.0, 0.1, 9.5];

/// Values for the round edges: ordinary, small, and near the 5 of the hole's depth.
const ROUND_VALUES: [f64; 3] = [1.0, 0.1, 4.5];

macro_rules! edge_matrix {
    ($name:ident, $doc:literal, $tool:expr, $cases:expr) => {
        probe! {
            budget = 1800;
            #[doc = $doc]
            fn $name() {
                let tool = &$tool;
                let cases: Vec<Case> = $cases(tool);
                run_all(tool, &cases, round_trips());
            }
        }
    };
}

edge_matrix!(
    fillets_on_the_straight_edges_of_a_block,
    "ROUNDING THE STRAIGHT EDGES OF A BLOCK: one, two apart, two meeting, the four of the top, the three of a corner - at an ordinary, a small and a near-limit radius.",
    FILLET,
    |_: &PartTool| BLOCK_VALUES.into_iter().flat_map(|v| block_cases(true, v)).collect()
);
edge_matrix!(
    chamfers_on_the_straight_edges_of_a_block,
    "CHAMFERING THE STRAIGHT EDGES OF A BLOCK: one, two apart, two meeting, the four of the top, the three of a corner - at an ordinary, a small and a near-limit leg.",
    CHAMFER,
    |_: &PartTool| BLOCK_VALUES.into_iter().flat_map(|v| block_cases(false, v)).collect()
);
edge_matrix!(fillets_on_round_edges, "ROUNDING ROUND EDGES: the top of a cylinder and the rim of a hole, at an ordinary, a small and a near-limit radius.", FILLET, |_: &PartTool| ROUND_VALUES
    .into_iter()
    .flat_map(|v| round_edge_cases(true, v))
    .collect());
edge_matrix!(chamfers_on_round_edges, "CHAMFERING ROUND EDGES: the top of a cylinder and the rim of a hole, at an ordinary, a small and a near-limit leg.", CHAMFER, |_: &PartTool| ROUND_VALUES
    .into_iter()
    .flat_map(|v| round_edge_cases(false, v))
    .collect());
edge_matrix!(fillets_along_an_earlier_chamfer_or_rounding, "ROUNDING EDGES THAT RUN ALONG AN EARLIER CHAMFER OR ROUNDING.", FILLET, |_: &PartTool| after_chamfer_cases(true)
    .into_iter()
    .chain(after_rounding_cases(true))
    .collect());
edge_matrix!(chamfers_along_an_earlier_chamfer_or_rounding, "CHAMFERING EDGES THAT RUN ALONG AN EARLIER CHAMFER OR ROUNDING - a chamfer on a chamfer among them.", CHAMFER, |_: &PartTool| {
    after_chamfer_cases(false).into_iter().chain(after_rounding_cases(false)).collect()
});
edge_matrix!(fillets_refuse_what_the_geometry_cannot_hold, "ROUNDING REFUSES IN WORDS what the geometry cannot hold, and zero and below - the body as it was.", FILLET, |_: &PartTool| refusals(true));
edge_matrix!(chamfers_refuse_what_the_geometry_cannot_hold, "CHAMFERING REFUSES IN WORDS what the geometry cannot hold, and zero and below - the body as it was.", CHAMFER, |_: &PartTool| refusals(
    false
));
edge_matrix!(fillets_in_a_notch, "ROUNDING THE EDGES OF A NOTCH: a straight edge cut in two, the upright edges, a corner of two, the hollow edges of the floor.", FILLET, |_: &PartTool| [
    1.0, 0.1, 4.5
]
.into_iter()
.flat_map(|v| notch_cases(true, v))
.collect());
edge_matrix!(chamfers_in_a_notch, "CHAMFERING THE EDGES OF A NOTCH: a straight edge cut in two, the upright edges, a corner of two, the hollow edges of the floor.", CHAMFER, |_: &PartTool| [
    1.0, 0.1, 4.5
]
.into_iter()
.flat_map(|v| notch_cases(false, v))
.collect());
edge_matrix!(fillets_round_a_boss, "ROUNDING ROUND A BOSS: the hollow round edge where it stands on the block, and the round edge of its top.", FILLET, |_: &PartTool| [1.0, 0.1, 4.5]
    .into_iter()
    .flat_map(|v| boss_cases(true, v))
    .collect());
edge_matrix!(chamfers_round_a_boss, "CHAMFERING ROUND A BOSS: the hollow round edge where it stands on the block, and the round edge of its top.", CHAMFER, |_: &PartTool| [1.0, 0.1, 4.5]
    .into_iter()
    .flat_map(|v| boss_cases(false, v))
    .collect());
edge_matrix!(
    a_fillet_runs_on_into_a_smaller_one_whole,
    "A ROUNDING THAT RUNS ON, TANGENT, INTO A SMALLER ONE: built where it can roll round it; where it cannot, overflowing onto the faces next to it or refused in words - never a broken body.",
    FILLET,
    |_: &PartTool| into_a_smaller_rounding()
);

probe! {
    /// A CLICK ON THE MIDDLE OF A SHORT EDGE ADDS THAT EDGE, even when an edge picked before ends at its corner. In
    /// the notch, the left piece of the top front edge is picked for rounding, then the middle of the upright edge of
    /// the notch below its end - 5 mm long - is clicked, with the view as it opens.
    ///
    /// Reported behaviour: three of four edges picked are rounded, one silently not.
    fn a_click_on_the_middle_of_a_short_edge_adds_it() {
        let mut s = Session::start();
        notched(&mut s);
        let hint = s.word(FILLET.hint);
        s.press_hint(&hint);
        let piece = s.edge_at([7.5, 0.0, 10.0]);
        s.click(piece);
        let (middle, corner) = ([15.0, 0.0, 7.5], [15.0, 0.0, 10.0]);
        let apart = match (s.seen_at(middle), s.seen_at(corner)) {
            (Some(a), Some(b)) => (a - b).length(),
            other => panic!("the upright edge of the notch is not in view: {other:?}"),
        };
        let at = s.edge_at(middle);
        s.click(at);
        let status = s.status();
        let added = s.word("pk-vertex-radius-on");
        assert!(status != added, "the click on the middle of the upright edge, {apart:.1} px from its top corner, was taken for that corner - the edge was not added; the status line says {status:?}");
        assert!(status.contains('2'), "the click on the middle of the upright edge did not add it: the status line says {status:?}");
    }
}

/// THE EDGES OF A WEDGE: its sharp edge on the table, 14.04 degrees across (the faces turn by 165.96), and the edge
/// at the top of its back, 75.96 across. A rounding of r on the sharp edge touches its faces r / tan(7.02 deg) = 8.13 r
/// from the edge, so one of 5 would run off the 40 of the table - refused.
fn wedge_cases(round: bool) -> Vec<Case> {
    let sharp = (10.0f64 / 40.0).atan();
    let top = std::f64::consts::FRAC_PI_2 - sharp;
    let sharp_edge = spot([40.0, 0.0, 0.0], [40.0, -7.0, 0.0]);
    let top_edge = spot([0.0, 0.0, 10.0], [0.0, -7.0, 10.0]);
    let area = |v: f64, inside: f64| if round { fillet_area(v, PI - inside) } else { chamfer_area(v, inside) };
    let added = if round { cylinders(1, 0) } else { planes(1) };
    let mut out = Vec::new();
    for v in [1.0, 0.2, 2.0] {
        out.push(case(&WEDGE, &[sharp_edge], v, built(-30.0 * area(v, sharp), added), "the sharp edge on the table"));
        out.push(case(&WEDGE, &[top_edge], v, built(-30.0 * area(v, top), added), "the edge at the top of the back"));
    }
    out.push(case(&WEDGE, &[sharp_edge], if round { 5.0 } else { 45.0 }, Expect::Refused, "the sharp edge, past the 40 of the table"));
    out
}

edge_matrix!(
    fillets_on_a_wedge,
    "ROUNDING THE EDGES OF A WEDGE: its sharp edge of 14 degrees and the edge of 76 at the top of its back; one that would run off the table refused.",
    FILLET,
    |_: &PartTool| wedge_cases(true)
);
edge_matrix!(chamfers_on_a_wedge, "CHAMFERING THE EDGES OF A WEDGE: its sharp edge of 14 degrees and the edge of 76 at the top of its back; one past the table refused.", CHAMFER, |_: &PartTool| {
    wedge_cases(false)
});

/// The first row of the tree that begins with the word `key` gives, before its first value.
fn tree_row(s: &mut qymcad::Session, key: &str) -> qymcad::Rect {
    let line = s.word(key);
    let head = line.split('{').next().unwrap_or(&line).to_string();
    let left = s.canvas().min.x;
    s.words_at().into_iter().find(|(w, r)| w.starts_with(&head) && r.max.x < left).map(|(_, r)| r).unwrap_or_else(|| panic!("no row of the tree begins with {head:?}; on screen: {:?}", s.words()))
}

/// Press `key` in the menu of the row at `at`.
fn from_the_menu(s: &mut qymcad::Session, at: qymcad::Rect, key: &str) {
    s.click_with(at.center(), qymcad::PointerButton::Secondary, qymcad::Modifiers::default());
    let word = s.word(key);
    let item = s.widgets().into_iter().filter(|w| w.label.contains(&word) && w.enabled).min_by(|a, b| a.rect.center().distance(at.center()).total_cmp(&b.rect.center().distance(at.center())));
    let item = item.unwrap_or_else(|| panic!("the menu of the row offers no {word:?}"));
    s.click(item.rect.center());
}

probe! {
    budget = 900;
    /// AN EDGE OF A ROUNDING TAKEN AWAY ABOVE IT: the four edges of the block's top rounded 2; the timeline rolled back
    /// to the extrusion and a strip 5 deep and 5 wide cut off the whole front of the top, which takes the top front
    /// edge away; the rollback cleared. The rounding has lost one of its four edges: it must say so in red, or round
    /// what took that edge's place - never stand green with three.
    ///
    /// Reported behaviour: three of four roundings are there, one is silently not, and no error.
    fn a_rounding_that_lost_an_edge_above_it_says_so() {
        let mut s = qymcad::Session::start();
        qymcad_acceptance::build::block(&mut s);
        let hint = s.word(FILLET.hint);
        s.press_hint(&hint);
        for edge in [TOP_FRONT.0, TOP_RIGHT.0, TOP_BACK.0, TOP_LEFT.0] {
            let p = qymcad_acceptance::matrix::show(&mut s, edge);
            let at = s.edge_at(p);
            s.click(at);
        }
        let radius = s.word(FILLET.caption);
        s.fill(&radius, "2").key(qymcad::Key::Enter);
        let rounded = |s: &mut qymcad::Session| {
            let (i, _) = qymcad_acceptance::matrix::working(s).unwrap_or_else(|| panic!("the part holds no body"));
            s.inspect(i).map(|k| k.kinds.cylinder).unwrap_or(0)
        };
        assert!(rounded(&mut s) == 4, "the four edges of the top were not rounded to begin with: {} rounded faces", rounded(&mut s));
        let row = tree_row(&mut s, "feat-extrude");
        from_the_menu(&mut s, row, "act-rollback-here");
        qymcad_acceptance::bodies::on_top(&mut s, |s| qymcad_acceptance::build::draw(s, "tb-rect-hint", &[(-5.0, -5.0), (45.0, 5.0)]), Some("cmd-cut"), "5");
        let row = tree_row(&mut s, "feat-extrude");
        // clearing the rollback is what builds the rounding again over the cut: the oracles judge that step, and what
        // they say is told here rather than swallowed, so that what became of the rounding can be told as well
        let said = qymcad_acceptance::refusal(|| from_the_menu(&mut s, row, "act-clear-rollback"));
        let doc = s.document();
        let order: Vec<&str> = doc.features.iter().map(|f| f.kind.as_str()).collect();
        let (cut, fillet) = (order.iter().rposition(|k| *k == "Combine"), order.iter().position(|k| *k == "Fillet"));
        assert!(matches!((cut, fillet), (Some(c), Some(f)) if c < f), "the cut did not go in above the rounding: the timeline is {order:?}");
        let node = doc.features.iter().find(|f| f.kind == "Fillet").cloned().unwrap_or_else(|| panic!("the rounding is gone"));
        let now = rounded(&mut s);
        let bodies: Vec<(String, f64)> = doc.bodies.iter().filter(|b| !b.consumed && !b.sheet).map(|b| (b.name.clone(), b.volume)).collect();
        assert!(said.is_empty(), "clearing the rollback over the cut left the program amiss: {said}; the part then holds {bodies:?}");
        // said in red, or - as decided 26.09 - the rest built and the node yellow with the words of what was left
        assert!(node.error.is_some() || node.warning.is_some() || now >= 4, "the rounding lost its top front edge to the cut above it and stands green with {now} rounded faces of four, saying nothing; the timeline {order:?}");
    }
}

probe! {
    /// A ROUNDING THAT LEAVES AN EDGE OUT SAYS SO IN THE TREE: the block rounded 11 on its front right upright edge and
    /// its top front edge - the upright takes 11 between faces of 30 and 40, the top one cannot, its side being 10
    /// high. The node builds the one and, pointed at in the tree, says that one of the two was left sharp (decided
    /// 26.09: yellow with words, neither green and quiet nor red with all that follows).
    fn a_rounding_that_leaves_an_edge_out_says_so_in_the_tree() {
        let mut s = qymcad::Session::start();
        qymcad_acceptance::build::block(&mut s);
        let hint = s.word(FILLET.hint);
        s.press_hint(&hint);
        for edge in [FRONT_RIGHT.0, TOP_FRONT.0] {
            let p = qymcad_acceptance::matrix::show(&mut s, edge);
            let at = s.edge_at(p);
            s.click(at);
        }
        let radius = s.word(FILLET.caption);
        s.fill(&radius, "11").key(qymcad::Key::Enter);
        let row = tree_row(&mut s, "feat-fillet");
        let said = s.hint_at(row.center()).join(" ");
        let want = s.word("warn-edges-dropped").replace("{$dropped}", "1").replace("{$asked}", "2");
        let node = s.document().features.iter().find(|f| f.kind == "Fillet").cloned().unwrap_or_else(|| panic!("the rounding is not in the timeline"));
        assert!(node.error.is_none(), "the rounding went red: {:?}", node.error);
        assert!(said.contains(&want), "the row of the rounding that left an edge out says {said:?}, not {want:?}");
    }
}
