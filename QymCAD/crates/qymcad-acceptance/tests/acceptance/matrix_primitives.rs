//! THE PRIMITIVES ACROSS SIZES: a box, a cylinder, a sphere, a cone (a frustum, a full cone, one upside down), a torus
//! and a six-sided prism - at their ordinary sizes, very small and very large ones, and sizes that make no body (zero,
//! below zero, a tube as thick as its ring); and a primitive made where the part holds a body already. What each must
//! hold is its shape's own volume.
use std::f64::consts::PI;

use qymcad_acceptance::bodies::{BLOCK, EMPTY};
use qymcad_acceptance::chains::Tier;
use qymcad_acceptance::matrix::{run_all, Added, Body, Case, Expect, PartTool};
use qymcad_acceptance::probe;

const BOX: PartTool = PartTool { hint: "tb-box-hint", caption: "f-length-x", node: "Box3", counts: false };
const CYLINDER: PartTool = PartTool { hint: "tb-cylinder-hint", caption: "f-radius", node: "Cylinder", counts: false };
const SPHERE: PartTool = PartTool { hint: "tb-sphere-hint", caption: "f-radius", node: "Sphere", counts: false };
const CONE: PartTool = PartTool { hint: "tb-cone-hint", caption: "f-radius-bottom", node: "Cone", counts: false };
const TORUS: PartTool = PartTool { hint: "tb-torus-hint", caption: "f-ring-r", node: "Torus", counts: false };
const PRISM: PartTool = PartTool { hint: "tb-prism-hint", caption: "f-radius-circ", node: "Prism", counts: false };

fn tol(v: f64) -> f64 {
    (v.abs() * 1e-3).max(1e-6)
}

fn made(volume: f64, plane: i32, cylinder: i32, cone: i32, blend: i32) -> Expect {
    Expect::Built { change: volume, tol: tol(volume), added: Some(Added { plane, cylinder, cone, blend }) }
}

/// A case in `body`: `value` into the tool's own field, `more` into the others.
fn case(body: &'static Body, value: f64, more: &[(&'static str, f64)], expect: Expect, what: &str) -> Case {
    Case {
        body,
        picks: Vec::new(),
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

fn boxes() -> Vec<Case> {
    let sized = |x: f64, y: f64, z: f64, what: &str| case(&EMPTY, x, &[("f-width-y", y), ("f-height-z", z)], made(x * y * z, 6, 0, 0, 0), what);
    let refused = |x: f64, y: f64, z: f64, what: &str| case(&EMPTY, x, &[("f-width-y", y), ("f-height-z", z)], Expect::Refused, what);
    vec![
        sized(30.0, 20.0, 10.0, "30 by 20 by 10"),
        sized(0.1, 0.1, 0.1, "0.1 each way"),
        sized(1000.0, 1000.0, 1000.0, "1000 each way"),
        refused(0.0, 20.0, 10.0, "no length"),
        refused(30.0, -20.0, 10.0, "a width below zero"),
        refused(30.0, 20.0, 0.0, "no height"),
        // made where the part holds the block: joined to it, or refused - a part is one body
        case(&BLOCK, 20.0, &[("f-width-y", 20.0), ("f-height-z", 20.0)], Expect::Whole, "20 each way, in the part of the block"),
    ]
}

fn cylinders() -> Vec<Case> {
    let sized = |r: f64, h: f64, what: &str| case(&EMPTY, r, &[("f-height", h)], made(PI * r * r * h, 2, 1, 0, 0), what);
    vec![
        sized(5.0, 12.0, "radius 5, 12 tall"),
        sized(0.05, 0.1, "radius 0.05, 0.1 tall"),
        sized(1000.0, 1000.0, "radius 1000, 1000 tall"),
        case(&EMPTY, 0.0, &[("f-height", 12.0)], Expect::Refused, "no radius"),
        case(&EMPTY, 5.0, &[("f-height", -12.0)], Expect::Refused, "a height below zero"),
    ]
}

fn spheres() -> Vec<Case> {
    let sized = |r: f64, what: &str| case(&EMPTY, r, &[], made(4.0 / 3.0 * PI * r * r * r, 0, 0, 0, 1), what);
    vec![
        sized(8.0, "radius 8"),
        sized(0.05, "radius 0.05"),
        sized(500.0, "radius 500"),
        case(&EMPTY, 0.0, &[], Expect::Refused, "no radius"),
        case(&EMPTY, -3.0, &[], Expect::Refused, "a radius below zero"),
    ]
}

/// A cone from `r1` at the bottom to `r2` at the top over `h`: a frustum has two flat ends, a full cone one.
fn cones() -> Vec<Case> {
    let frustum = |r1: f64, r2: f64, h: f64| PI * h / 3.0 * (r1 * r1 + r1 * r2 + r2 * r2);
    let sized = |r1: f64, r2: f64, h: f64, ends: i32, what: &str| case(&EMPTY, r1, &[("f-radius-top", r2), ("f-height", h)], made(frustum(r1, r2, h), ends, 0, 1, 0), what);
    vec![
        sized(10.0, 4.0, 15.0, 2, "from 10 to 4 over 15"),
        sized(10.0, 0.0, 15.0, 1, "from 10 to a point over 15"),
        sized(4.0, 10.0, 15.0, 2, "from 4 up to 10 over 15"),
        case(&EMPTY, 0.0, &[("f-radius-top", 0.0), ("f-height", 15.0)], Expect::Refused, "no radius at either end"),
        case(&EMPTY, 10.0, &[("f-radius-top", 4.0), ("f-height", 0.0)], Expect::Refused, "no height"),
        case(&EMPTY, 10.0, &[("f-radius-top", -4.0), ("f-height", 15.0)], Expect::Refused, "a top radius below zero"),
    ]
}

/// A torus of the ring radius and the tube radius: the tube swept round the ring. A tube as thick as the ring or
/// thicker would pass through itself.
fn tori() -> Vec<Case> {
    let sized = |ring: f64, tube: f64, what: &str| case(&EMPTY, ring, &[("f-tube-r", tube)], made(2.0 * PI * PI * ring * tube * tube, 0, 0, 0, 1), what);
    vec![
        sized(20.0, 5.0, "ring 20, tube 5"),
        sized(12.0, 11.0, "ring 12, tube 11: nearly closed in the middle"),
        sized(1.0, 0.1, "ring 1, tube 0.1"),
        case(&EMPTY, 5.0, &[("f-tube-r", 5.0)], Expect::Refused, "a tube as thick as the ring"),
        case(&EMPTY, 5.0, &[("f-tube-r", 8.0)], Expect::Refused, "a tube thicker than the ring"),
        case(&EMPTY, 20.0, &[("f-tube-r", 0.0)], Expect::Refused, "no tube"),
    ]
}

/// A six-sided prism in a circle of radius `r`: its section is 3 sqrt 3 / 2 r^2.
fn prisms() -> Vec<Case> {
    let sized = |r: f64, h: f64, what: &str| case(&EMPTY, r, &[("f-height", h)], made(1.5 * 3f64.sqrt() * r * r * h, 8, 0, 0, 0), what);
    let mut all = vec![
        sized(10.0, 12.0, "in a circle of 10, 12 tall"),
        sized(0.1, 0.1, "in a circle of 0.1, 0.1 tall"),
        case(&EMPTY, 0.0, &[("f-height", 12.0)], Expect::Refused, "in no circle"),
        case(&EMPTY, 10.0, &[("f-height", -12.0)], Expect::Refused, "a height below zero"),
    ];
    // THE COUNT OF SIDES stands in the bar, the size at the geometry. A prism of n sides in a circle of r has the
    // section n / 2 r^2 sin(2 pi / n) and n + 2 flat faces; the count is typed first, as a person sets it, and the
    // height last, so that Enter is pressed over a field at the geometry.
    for n in [3.0, 5.0, 8.0, 64.0] {
        let volume = n / 2.0 * 100.0 * (2.0 * PI / n).sin() * 12.0;
        all.push(case(&EMPTY, 10.0, &[("cmd-sides", n), ("f-height", 12.0)], made(volume, n as i32 + 2, 0, 0, 0), &format!("of {n} sides in a circle of 10, 12 tall")));
    }
    // fewer than three sides is no prism, and the bar takes no more than 64
    for n in [2.0, 65.0] {
        all.push(case(&EMPTY, 10.0, &[("cmd-sides", n), ("f-height", 12.0)], Expect::Refused, &format!("of {n} sides")));
    }
    all
}

macro_rules! primitive_matrix {
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

primitive_matrix!(boxes_across_sizes, "A BOX across sizes; no length, a width below zero, no height refused; one made where the part holds a body.", BOX, boxes);
primitive_matrix!(cylinders_across_sizes, "A CYLINDER across sizes; no radius and a height below zero refused.", CYLINDER, cylinders);
primitive_matrix!(spheres_across_sizes, "A SPHERE across sizes; no radius and one below zero refused.", SPHERE, spheres);
primitive_matrix!(cones_across_sizes, "A CONE: a frustum, a full cone, one upside down; no radius, no height and a radius below zero refused.", CONE, cones);
primitive_matrix!(tori_across_sizes, "A TORUS across sizes; a tube as thick as the ring or thicker, and no tube, refused.", TORUS, tori);
primitive_matrix!(prisms_across_sizes, "A SIX-SIDED PRISM across sizes; no circle and a height below zero refused.", PRISM, prisms);
