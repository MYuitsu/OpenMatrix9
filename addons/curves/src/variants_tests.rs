// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix 9 sub-options of Polyline, InterpCrv, Rectangle, Circle and Ellipse, step by
//! step as OM9 sends them, with the geometry they create.
use crate::*;
use om9_addon_abi::session::{Selected, dist, norm};
use om9_addon_abi::session::{add, cross};
use std::f64::consts::{FRAC_PI_2, PI};

fn ev(command: &str, input: Input, state: &Value, sel: &[Selected]) -> Event {
    let mut e = event(command, input, state);
    e.selection = sel.to_vec();
    e
}

/// Every reply of the run (the first is Start).
fn run_sel(command: &str, inputs: Vec<Input>, sel: &[Selected]) -> Vec<Reply> {
    let mut state = Value::Null;
    let mut out = Vec::new();
    for input in std::iter::once(Input::Start).chain(inputs) {
        let hover = matches!(input, Input::Hover { .. });
        let r = Curves::step(&ev(command, input, &state, sel)).unwrap();
        if !hover {
            state = r.state.clone();
        }
        out.push(r);
    }
    if let Some(last) = out.last().filter(|r| r.done) {
        let inputs: Vec<Geo> = sel.iter().filter_map(|s| s.exact.clone()).collect();
        let test = std::thread::current().name().unwrap_or("").to_owned();
        om9_addon_geom::audit::record("curves", &test, command, &inputs, &last.create);
    }
    out
}
fn run(command: &str, inputs: Vec<Input>) -> Vec<Reply> {
    run_sel(command, inputs, &[])
}
fn pt(x: f64, y: f64) -> Input {
    Input::Point { point: [x, y, 0.0] }
}
fn opt(name: &str) -> Input {
    Input::Option {
        name: name.into(),
        value: None,
    }
}
fn set(name: &str, v: &str) -> Input {
    Input::Option {
        name: name.into(),
        value: Some(v.into()),
    }
}
fn text(t: &str) -> Input {
    Input::Text { text: t.into() }
}
fn names(r: &Reply) -> Vec<String> {
    r.options
        .iter()
        .map(|o| match &o.value {
            Some(v) => format!("{}={v}", o.name),
            None => o.name.clone(),
        })
        .collect()
}
fn ok(r: &[Reply]) -> &Reply {
    for (i, x) in r.iter().enumerate() {
        assert!(x.error.is_none(), "step {i}: {:?} ({})", x.error, x.prompt);
    }
    r.last().unwrap()
}
fn created(r: &[Reply]) -> Geo {
    let last = ok(r);
    assert!(last.done, "{}", last.prompt);
    last.create[0].clone()
}
fn at(g: &Geo, t: f64) -> Vec3 {
    g::eval_geo(g, t).unwrap()
}
fn tangent(g: &Geo, t: f64) -> Vec3 {
    let h = 1e-6;
    let (a, b) = if t < 0.5 { (t, t + h) } else { (t - h, t) };
    unit(sub(at(g, b), at(g, a)))
}
fn circle_sel(id: &str, cx: f64, cy: f64, r: f64) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        closed: true,
        points: (0..512)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 512.0;
                [cx + r * a.cos(), cy + r * a.sin(), 0.0]
            })
            .collect(),
        exact: Some(Geo::Circle {
            center: [cx, cy, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: r,
        }),
        ..Default::default()
    }
}

#[test]
fn polyline_prompts_follow_matrix() {
    let r = run(POLYLINE, vec![pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 10.0)]);
    assert_eq!(r[0].prompt, "Start of polyline");
    assert_eq!(names(&r[0]), ["PersistentClose=No"]);
    assert_eq!(r[1].prompt, "Next point of polyline");
    assert_eq!(
        names(&r[1]),
        ["PersistentClose=No", "Mode=Line", "Helpers=No", "Undo"]
    );
    assert_eq!(r[2].prompt, "Next point of polyline. Press Enter when done");
    assert_eq!(
        names(&r[2]),
        [
            "PersistentClose=No",
            "Mode=Line",
            "Helpers=No",
            "Length",
            "Undo"
        ]
    );
    assert_eq!(
        names(&r[3]),
        [
            "PersistentClose=No",
            "Close",
            "Mode=Line",
            "Helpers=No",
            "Length",
            "Undo"
        ]
    );
}

#[test]
fn polyline_arc_mode_is_tangent_and_exact() {
    // A line, then an arc tangent to it ending at (20, 10): a half circle of radius 5.
    let g = created(&run(
        POLYLINE,
        vec![
            pt(0.0, 0.0),
            pt(10.0, 0.0),
            opt("Mode"),
            pt(10.0, 10.0),
            Input::Enter,
        ],
    ));
    let Geo::Nurbs { degree: 2, .. } = g else {
        panic!("{g:?}")
    };
    assert!(dist(at(&g, 0.0), [0.0; 3]) < 1e-9);
    assert!(dist(at(&g, 1.0), [10.0, 10.0, 0.0]) < 1e-9);
    // Every sample of the arc part is 5 from its center (10, 5).
    for i in 0..=40 {
        let p = at(&g, 0.5 + 0.5 * i as f64 / 40.0);
        if p[0] > 10.0 + 1e-6 {
            assert!((dist(p, [10.0, 5.0, 0.0]) - 5.0).abs() < 1e-9, "{p:?}");
        }
    }
    // Arc mode shows Direction and Center instead of Length; Center makes the arc around it.
    let r = run(POLYLINE, vec![pt(0.0, 0.0), set("Mode", "Arc")]);
    assert!(names(r.last().unwrap()).contains(&"Direction".into()));
    let g = created(&run(
        POLYLINE,
        vec![
            pt(5.0, 0.0),
            set("Mode", "Arc"),
            opt("Center"),
            pt(0.0, 0.0),
            pt(0.0, 9.0),
            Input::Enter,
        ],
    ));
    assert!(
        dist(at(&g, 1.0), [0.0, 5.0, 0.0]) < 1e-9,
        "{:?}",
        at(&g, 1.0)
    );
    // Helpers draw guide lines.
    let r = run(
        POLYLINE,
        vec![
            pt(0.0, 0.0),
            opt("Helpers"),
            Input::Hover {
                point: [3.0, 1.0, 0.0],
            },
        ],
    );
    assert!(r.last().unwrap().preview.len() >= 3);
}

#[test]
fn interp_knots_submenu_tangents_and_closing() {
    let r = run(INTERP, vec![]);
    assert_eq!(r[0].prompt, "Start of curve");
    assert_eq!(
        names(&r[0]),
        [
            "Degree=3",
            "Knots=Uniform",
            "PersistentClose=No",
            "StartTangent"
        ]
    );
    // Knots opens its own choice.
    let r = run(INTERP, vec![opt("Knots")]);
    assert_eq!(r[1].prompt, "Knots <Uniform>");
    assert_eq!(names(&r[1]), ["Uniform", "Chord", "SqrtChord"]);
    let r = run(INTERP, vec![opt("Knots"), opt("Chord")]);
    assert!(names(&r[2]).contains(&"Knots=Chord".into()));
    let r = run(INTERP, vec![set("Knots", "SqrtChrd")]);
    assert!(names(&r[1]).contains(&"Knots=SqrtChord".into()));
    // After three points: Close and Sharp.
    let pts = || vec![pt(0.0, 0.0), pt(10.0, 3.0), pt(12.0, 12.0), pt(2.0, 14.0)];
    let mut inputs = pts();
    inputs.truncate(3);
    let r = run(INTERP, inputs);
    for n in ["Close", "Sharp", "EndTangent", "Undo"] {
        assert!(names(&r[3]).contains(&n.to_owned()), "{n}");
    }
    // Close: smooth seam; Sharp: kinked seam.
    let mut c = pts();
    c.push(opt("Close"));
    let g = created(&run(INTERP, c));
    assert!(dist(at(&g, 0.0), at(&g, 1.0)) < 1e-9);
    assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) < 1e-4);
    let mut s = pts();
    s.push(opt("Sharp"));
    let g = created(&run(INTERP, s));
    assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) > 1e-2);
    // Picking the start again closes smoothly.
    let mut a = pts();
    a.push(pt(0.0, 0.0));
    let g = created(&run(INTERP, a));
    assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) < 1e-4);
    // Start and end tangents.
    let g = created(&run(
        INTERP,
        vec![
            opt("StartTangent"),
            pt(0.0, 0.0),
            pt(0.0, 5.0),
            pt(10.0, 3.0),
            pt(12.0, 12.0),
            opt("EndTangent"),
            pt(20.0, 12.0),
            pt(30.0, 12.0),
        ],
    ));
    assert!(
        dist(tangent(&g, 0.0), [0.0, 1.0, 0.0]) < 1e-4,
        "{:?}",
        tangent(&g, 0.0)
    );
    assert!(
        dist(tangent(&g, 1.0), [1.0, 0.0, 0.0]) < 1e-4,
        "{:?}",
        tangent(&g, 1.0)
    );
    assert!(dist(at(&g, 1.0), [20.0, 12.0, 0.0]) < 1e-9);
}

#[test]
fn rectangle_rounded_arc_conic_and_pass_through() {
    let r = run(RECTANGLE, vec![]);
    assert_eq!(names(&r[0]), ["3Point", "Vertical", "Center", "Rounded=No"]);
    let r = run(RECTANGLE, vec![opt("Rounded"), pt(0.0, 0.0), pt(10.0, 6.0)]);
    assert_eq!(
        r[3].prompt,
        "Radius or point for rounded corner to pass through <0.6>"
    );
    assert_eq!(names(&r[3]), ["Corner=Arc"]);
    // Arc corners: the corner arc's middle is r(1 − 1/√2) inside the corner on the diagonal.
    let g = created(&run(
        RECTANGLE,
        vec![opt("Rounded"), pt(0.0, 0.0), pt(10.0, 6.0), text("2")],
    ));
    let d = (0..=2000)
        .map(|i| dist(at(&g, i as f64 / 2000.0), [0.0, 0.0, 0.0]))
        .fold(f64::INFINITY, f64::min);
    assert!((d - 2.0 * (2f64.sqrt() - 1.0)).abs() < 1e-3, "{d}");
    // Conic corners with Rho: a different (flatter for rho > 1 − 1/√2) corner.
    let r = run(
        RECTANGLE,
        vec![
            opt("Rounded"),
            pt(0.0, 0.0),
            pt(10.0, 6.0),
            set("Corner", "Conic"),
        ],
    );
    assert_eq!(names(&r[4]), ["Corner=Conic", "Rho=0.5"]);
    let g = created(&run(
        RECTANGLE,
        vec![
            opt("Rounded"),
            pt(0.0, 0.0),
            pt(10.0, 6.0),
            set("Corner", "Conic"),
            set("Rho", "0.8"),
            text("2"),
        ],
    ));
    let dc = (0..=2000)
        .map(|i| dist(at(&g, i as f64 / 2000.0), [0.0, 0.0, 0.0]))
        .fold(f64::INFINITY, f64::min);
    assert!(dc < d - 0.1, "rho 0.8 hugs the corner: {dc} vs {d}");
    // A picked point: the corner passes through it.
    let g = created(&run(
        RECTANGLE,
        vec![opt("Rounded"), pt(0.0, 0.0), pt(10.0, 6.0), pt(0.3, 0.3)],
    ));
    let dp = (0..=4000)
        .map(|i| dist(at(&g, i as f64 / 4000.0), [0.3, 0.3, 0.0]))
        .fold(f64::INFINITY, f64::min);
    assert!(dp < 1e-2, "{dp}");
    // Length, then Enter: a square.
    let Geo::Polyline { points, .. } =
        created(&run(RECTANGLE, vec![pt(0.0, 0.0), text("4"), Input::Enter]))
    else {
        panic!()
    };
    assert_eq!(points[2], [4.0, 4.0, 0.0]);
}

#[test]
fn circle_sub_options() {
    let r = run(CIRCLE, vec![]);
    assert_eq!(
        names(&r[0]),
        [
            "Deformable",
            "Vertical",
            "2Point",
            "3Point",
            "Tangent",
            "AroundCurve",
            "FitPoints"
        ]
    );
    let r = run(CIRCLE, vec![pt(0.0, 0.0)]);
    assert_eq!(r[1].prompt, "Radius");
    assert_eq!(
        names(&r[1]),
        ["Diameter", "Orientation", "Circumference", "Area"]
    );
    let r = run(CIRCLE, vec![pt(0.0, 0.0), opt("Diameter")]);
    assert_eq!(r[2].prompt, "Diameter");
    assert_eq!(
        names(&r[2]),
        ["Radius", "Orientation", "Circumference", "Area"]
    );
    let radius = |g: &Geo| match g {
        Geo::Circle { radius, .. } => *radius,
        _ => panic!("{g:?}"),
    };
    // Circumference and area.
    let g = created(&run(
        CIRCLE,
        vec![pt(0.0, 0.0), opt("Circumference"), text("62.8318530718")],
    ));
    assert!((radius(&g) - 10.0).abs() < 1e-9);
    let g = created(&run(
        CIRCLE,
        vec![pt(0.0, 0.0), opt("Area"), text("314.159265359")],
    ));
    assert!((radius(&g) - 10.0).abs() < 1e-9);
    // Orientation: the circle's normal points at the picked point.
    let r = run(
        CIRCLE,
        vec![
            pt(0.0, 0.0),
            opt("Orientation"),
            Input::Point {
                point: [1.0, 0.0, 0.0],
            },
            text("3"),
        ],
    );
    let Geo::Circle { x, y, .. } = ok(&r).create[0] else {
        panic!()
    };
    assert!(dot(cross(x, y), [1.0, 0.0, 0.0]).abs() > 1.0 - 1e-9);
    // Deformable: Degree / PointCount, a NURBS with 10 + 3 poles (clamped periodic).
    let r = run(CIRCLE, vec![opt("Deformable")]);
    assert_eq!(
        names(&r[1])[..2],
        ["Degree=3".to_owned(), "PointCount=10".to_owned()]
    );
    let g = created(&run(
        CIRCLE,
        vec![opt("Deformable"), pt(0.0, 0.0), text("5")],
    ));
    let Geo::Nurbs {
        points, degree: 3, ..
    } = &g
    else {
        panic!("{g:?}")
    };
    assert_eq!(points.len(), 13);
    for i in 0..10 {
        assert!((norm(at(&g, i as f64 / 10.0)) - 5.0).abs() < 1e-6);
    }
}

#[test]
fn circle_around_curve_tangent_and_fit() {
    // AroundCurve on a line along x: the circle stands in the YZ plane.
    let line = Selected {
        id: "l".into(),
        kind: "curve".into(),
        points: vec![[0.0; 3], [20.0, 0.0, 0.0]],
        exact: Some(Geo::Line {
            start: [0.0; 3],
            end: [20.0, 0.0, 0.0],
        }),
        ..Default::default()
    };
    let r = run_sel(
        CIRCLE,
        vec![opt("AroundCurve"), pt(5.0, 1.0), text("2")],
        std::slice::from_ref(&line),
    );
    let Geo::Circle {
        center,
        x,
        y,
        radius,
    } = ok(&r).create[0]
    else {
        panic!()
    };
    assert!(dist(center, [5.0, 0.0, 0.0]) < 1e-6 && (radius - 2.0).abs() < 1e-12);
    assert!(dot(cross(x, y), [1.0, 0.0, 0.0]).abs() > 1.0 - 1e-6);
    // Tangent to two circles with a radius: centers 20 apart, radius 5 each → r 5 touches
    // both at the midpoint (10, 0) ± … : the circle at (10, ±√(10² − 10²)) … use r = 6.
    let (a, b) = (
        circle_sel("a", 0.0, 0.0, 5.0),
        circle_sel("b", 20.0, 0.0, 5.0),
    );
    let r = run_sel(
        CIRCLE,
        vec![opt("Tangent"), text("6"), pt(10.0, 4.0)],
        &[a.clone(), b.clone()],
    );
    let Geo::Circle { center, radius, .. } = ok(&r).create[0] else {
        panic!()
    };
    assert!((radius - 6.0).abs() < 1e-9);
    // Outside-tangent to both: |c − (0,0)| = 11 → c = (10, √21).
    assert!(dist(center, [10.0, 21f64.sqrt(), 0.0]) < 1e-3, "{center:?}");
    // Three circles: the circle touching all three (outside) found near the pick.
    let c3 = circle_sel("c", 10.0, 20.0, 5.0);
    let r = run_sel(CIRCLE, vec![opt("Tangent"), pt(10.0, 7.0)], &[a, b, c3]);
    let Geo::Circle { center, radius, .. } = ok(&r).create[0] else {
        panic!()
    };
    for (cx, cy) in [(0.0, 0.0), (20.0, 0.0), (10.0, 20.0)] {
        let d = dist(center, [cx, cy, 0.0]);
        assert!((d - (radius + 5.0)).abs() < 1e-3, "{d} {radius}");
    }
    // FitPoints: point objects on a circle of radius 3 around (1, 1).
    let pts: Vec<Selected> = (0..5)
        .map(|i| {
            let a = i as f64;
            let p = [1.0 + 3.0 * a.cos(), 1.0 + 3.0 * a.sin(), 0.0];
            Selected {
                id: format!("p{i}"),
                kind: "point".into(),
                bbox: [p, p],
                ..Default::default()
            }
        })
        .collect();
    let r = run_sel(CIRCLE, vec![opt("FitPoints"), Input::Enter], &pts);
    let Geo::Circle { center, radius, .. } = ok(&r).create[0] else {
        panic!()
    };
    assert!(dist(center, [1.0, 1.0, 0.0]) < 1e-9 && (radius - 3.0).abs() < 1e-9);
}

#[test]
fn ellipse_sub_options() {
    let r = run(ELLIPSE, vec![]);
    assert_eq!(r[0].prompt, "Ellipse center");
    assert_eq!(
        names(&r[0]),
        [
            "Deformable",
            "Vertical",
            "Corner",
            "Diameter",
            "FromFoci",
            "AroundCurve"
        ]
    );
    let r = run(ELLIPSE, vec![pt(0.0, 0.0)]);
    assert_eq!(r[1].prompt, "End of first axis");
    assert_eq!(names(&r[1]), ["Corner"]);
    let semi = |g: &Geo| {
        let xs: Vec<Vec3> = (0..400).map(|i| at(g, i as f64 / 400.0)).collect();
        let c = scale(xs.iter().fold([0.0; 3], |a, p| add(a, *p)), 1.0 / 400.0);
        let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
        for p in &xs {
            lo = lo.min(dist(*p, c));
            hi = hi.max(dist(*p, c));
        }
        (c, hi, lo)
    };
    // Corner: the enclosing rectangle 10 × 4 → semi-axes 5 and 2.
    let (c, a, b) = semi(&created(&run(
        ELLIPSE,
        vec![opt("Corner"), pt(0.0, 0.0), pt(10.0, 4.0)],
    )));
    assert!(dist(c, [5.0, 2.0, 0.0]) < 1e-3 && (a - 5.0).abs() < 1e-3 && (b - 2.0).abs() < 1e-3);
    // Diameter: axis ends (−4, 0)–(4, 0), second axis 1.
    let (c, a, b) = semi(&created(&run(
        ELLIPSE,
        vec![opt("Diameter"), pt(-4.0, 0.0), pt(4.0, 0.0), pt(0.0, 1.0)],
    )));
    assert!(dist(c, [0.0; 3]) < 1e-3 && (a - 4.0).abs() < 1e-3 && (b - 1.0).abs() < 1e-3);
    // FromFoci with MarkFoci: foci (±3, 0), point (5, 0) → a = 5, b = 4, two points marked.
    let r = run(
        ELLIPSE,
        vec![
            opt("FromFoci"),
            opt("MarkFoci"),
            pt(-3.0, 0.0),
            pt(3.0, 0.0),
            pt(5.0, 0.0),
        ],
    );
    let last = ok(&r);
    assert_eq!(last.create.len(), 3, "ellipse + two focus points");
    let (_, a, b) = semi(&last.create[0]);
    assert!((a - 5.0).abs() < 1e-3 && (b - 4.0).abs() < 1e-3, "{a} {b}");
    // Corner after the center: a corner of the box around that center.
    let (c, a, b) = semi(&created(&run(
        ELLIPSE,
        vec![pt(0.0, 0.0), opt("Corner"), pt(6.0, 2.0)],
    )));
    assert!(dist(c, [0.0; 3]) < 1e-3 && (a - 6.0).abs() < 1e-3 && (b - 2.0).abs() < 1e-3);
    // Vertical: the second axis along the CPlane normal.
    let g = created(&run(
        ELLIPSE,
        vec![opt("Vertical"), pt(0.0, 0.0), pt(4.0, 0.0), text("2")],
    ));
    assert!(
        dist(at(&g, 0.25), [0.0, 0.0, 2.0]) < 1e-9,
        "{:?}",
        at(&g, 0.25)
    );
    // Deformable ellipse.
    let g = created(&run(
        ELLIPSE,
        vec![opt("Deformable"), pt(0.0, 0.0), pt(4.0, 0.0), pt(0.0, 2.0)],
    ));
    assert!(matches!(g, Geo::Nurbs { degree: 3, .. }));
}

fn arc_of(g: &Geo) -> (Vec3, f64, f64) {
    match g {
        Geo::Arc { center, radius, angle, .. } => (*center, *radius, *angle),
        other => panic!("not an arc: {other:?}"),
    }
}

#[test]
fn arc_center_start_angle_and_options() {
    let r = run(ARC, vec![]);
    assert_eq!(r[0].prompt, "Center of arc");
    assert_eq!(names(&r[0]), ["Deformable=No", "StartPoint", "Tangent", "Extension"]);
    // A typed radius after the center, then the angle; Clockwise turns the other way.
    let r = run(ARC, vec![pt(0.0, 0.0), text("2"), set("Direction", "Clockwise"), text("90")]);
    let g = &ok(&r).create[0];
    let (c, rad, ang) = arc_of(g);
    assert!(dist(c, [0.0; 3]) < 1e-12 && (rad - 2.0).abs() < 1e-12 && (ang - FRAC_PI_2).abs() < 1e-12);
    assert!(dist(g::eval_geo(g, 1.0).unwrap(), [0.0, -2.0, 0.0]) < 1e-9, "clockwise ends below");
    assert_eq!(names(&r[2]), ["Deformable=No", "Length", "Direction=CounterClockwise", "Undo"]);
    // Length: an arc 3 mm long on radius 2.
    let r = run(ARC, vec![pt(0.0, 0.0), pt(2.0, 0.0), set("Length", "3")]);
    let (_, rad, ang) = arc_of(&ok(&r).create[0]);
    assert!((rad * ang - 3.0).abs() < 1e-12);
    // Deformable: a NURBS with PointCount poles.
    let r = run(ARC, vec![opt("Deformable"), set("PointCount", "6"), pt(0.0, 0.0), pt(2.0, 0.0), text("180")]);
    match &ok(&r).create[0] {
        Geo::Nurbs { points, degree, .. } => assert!(points.len() == 6 && *degree == 3),
        other => panic!("{other:?}"),
    }
}

#[test]
fn arc_start_point_variants() {
    // Start, end, point on arc.
    let r = run(ARC, vec![opt("StartPoint"), pt(2.0, 0.0), pt(-2.0, 0.0), pt(0.0, 2.0)]);
    let (c, rad, _) = arc_of(&ok(&r).create[0]);
    assert!(dist(c, [0.0; 3]) < 1e-9 && (rad - 2.0).abs() < 1e-9);
    assert_eq!(r[2].prompt, "End of arc");
    assert_eq!(names(&r[2]), ["Direction", "ThroughPoint", "Undo"]);
    assert_eq!(r[3].prompt, "Point on arc");
    // Start, direction, end.
    let r = run(ARC, vec![opt("StartPoint"), pt(0.0, 0.0), opt("Direction"), pt(0.0, 1.0), pt(2.0, 0.0)]);
    let (c, rad, ang) = arc_of(&ok(&r).create[0]);
    assert!(dist(c, [1.0, 0.0, 0.0]) < 1e-9 && (rad - 1.0).abs() < 1e-9 && (ang - PI).abs() < 1e-9, "{c:?} {rad} {ang}");
    // Start, point, end.
    let r = run(ARC, vec![opt("StartPoint"), pt(2.0, 0.0), opt("ThroughPoint"), pt(0.0, 2.0), pt(-2.0, 0.0)]);
    assert!((arc_of(&ok(&r).create[0]).1 - 2.0).abs() < 1e-9);
    // Start, end, radius, then the side.
    let r = run(ARC, vec![opt("StartPoint"), pt(-1.0, 0.0), pt(1.0, 0.0), opt("Radius"), text("5"), pt(0.0, 1.0)]);
    let g = &ok(&r).create[0];
    let (_, rad, _) = arc_of(g);
    assert!((rad - 5.0).abs() < 1e-9);
    assert!(g::eval_geo(g, 0.5).unwrap()[1] > 0.0, "bulges toward the pick");
    // Matrix "Arc Start End Direction": start, end, direction at start.
    let r = run(ARC_SED, vec![pt(2.0, 0.0), pt(0.0, 2.0), pt(2.0, 1.0)]);
    assert_eq!(r[0].prompt, "Start of arc");
    assert_eq!(r[2].prompt, "Direction at start");
    let (c, _, _) = arc_of(&ok(&r).create[0]);
    assert!(dist(c, [0.0; 3]) < 1e-9, "{c:?}");
}

fn crv(id: &str, g: Geo) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        points: g::nurbs::samples(&g, 200),
        exact: Some(g),
        ..Default::default()
    }
}

#[test]
fn arc_tangent_and_extension() {
    let a = crv("a", Geo::Line { start: [-10.0, 0.0, 0.0], end: [10.0, 0.0, 0.0] });
    let b = crv("b", Geo::Line { start: [0.0, -10.0, 0.0], end: [0.0, 10.0, 0.0] });
    // Tangent: pick both lines, radius 2: the quarter arc around (2, 2).
    let mut st = Value::Null;
    let mut step = |input: Input, sel: Vec<Selected>, pick: Option<Vec3>| {
        let mut e = ev(ARC, input, &st, &sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
        st = r.state.clone();
        r
    };
    step(Input::Start, vec![], None);
    let r = step(opt("Tangent"), vec![], None);
    assert_eq!(r.prompt, "First tangent curve");
    let r = step(Input::Selection, vec![a.clone()], Some([5.0, 0.0, 0.0]));
    assert_eq!(r.prompt, "Second tangent curve");
    let r = step(Input::Selection, vec![a.clone(), b.clone()], Some([0.0, 5.0, 0.0]));
    assert_eq!(r.prompt, "Radius <1>");
    let r = step(text("2"), vec![a.clone(), b.clone()], None);
    let (c, rad, ang) = arc_of(&r.create[0]);
    assert!(dist(c, [2.0, 2.0, 0.0]) < 1e-6 && (rad - 2.0).abs() < 1e-6 && (ang - FRAC_PI_2).abs() < 1e-6);
    // Extension: an arc tangent to the end of the line.
    let mut st2 = Value::Null;
    let mut step2 = |input: Input, sel: Vec<Selected>, pick: Option<Vec3>| {
        let mut e = ev(ARC, input, &st2, &sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
        st2 = r.state.clone();
        r
    };
    step2(Input::Start, vec![], None);
    step2(opt("Extension"), vec![], None);
    let r = step2(Input::Selection, vec![a.clone()], Some([9.0, 0.0, 0.0]));
    assert_eq!(r.prompt, "End of arc");
    let r = step2(pt(12.0, 2.0), vec![a.clone()], None);
    let g = &r.create[0];
    let (c, rad, _) = arc_of(g);
    assert!((c[0] - 10.0).abs() < 1e-9 && (rad - 2.0).abs() < 1e-9, "tangent to the end: {c:?} {rad}");
}

fn poly_pts(g: &Geo) -> Vec<Vec3> {
    match g {
        Geo::Polyline { points, closed: true } => points.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn polygon_modes() {
    let r = run(POLYGON, vec![]);
    assert_eq!(r[0].prompt, "Center of inscribed polygon");
    assert_eq!(names(&r[0]), ["NumSides=5", "Mode=Inscribed", "Edge", "Star", "Vertical", "AroundCurve"]);
    // Circumscribed hexagon: the click is the middle of an edge.
    let r = run(POLYGON, vec![set("NumSides", "6"), set("Mode", "Circumscribed"), pt(0.0, 0.0), text("2")]);
    assert_eq!(r[3].prompt, "Midpoint of edge");
    let p = poly_pts(&ok(&r).create[0]);
    assert_eq!(p.len(), 6);
    let mid = scale(add(p[0], p[1]), 0.5);
    assert!((norm(mid) - 2.0).abs() < 1e-9);
    // Edge: a square on the edge (0,0)–(2,0), to the left (above).
    let r = run(POLYGON, vec![set("NumSides", "4"), opt("Edge"), pt(0.0, 0.0), pt(2.0, 0.0)]);
    let p = poly_pts(&ok(&r).create[0]);
    assert!(dist(p[2], [2.0, 2.0, 0.0]) < 1e-9 && dist(p[3], [0.0, 2.0, 0.0]) < 1e-9, "{p:?}");
    let r = run(POLYGON, vec![set("NumSides", "4"), opt("Edge"), pt(0.0, 0.0), opt("Flip"), pt(2.0, 0.0)]);
    assert!(poly_pts(&ok(&r).create[0])[2][1] < 0.0, "flipped below");
    // Star: 5 points, outer 3, inner 1.
    let r = run(POLYGON, vec![opt("Star"), pt(0.0, 0.0), pt(3.0, 0.0), text("1")]);
    let p = poly_pts(&ok(&r).create[0]);
    assert_eq!(p.len(), 10);
    assert!((norm(p[0]) - 3.0).abs() < 1e-9 && (norm(p[1]) - 1.0).abs() < 1e-9);
    // Automatic inner radius: the star's edges line up (pentagram).
    let r = run(POLYGON, vec![opt("Star"), pt(0.0, 0.0), pt(3.0, 0.0), opt("Automatic")]);
    let p = poly_pts(&ok(&r).create[0]);
    let (a, b, c) = (p[0], p[1], p[4]);
    assert!(norm(cross(sub(b, a), sub(c, a))) < 1e-9, "p0, p1 and p4 on one line");
    // Vertical: stands on the CPlane.
    let r = run(POLYGON, vec![opt("Vertical"), pt(0.0, 0.0), pt(2.0, 0.0)]);
    let p = poly_pts(&ok(&r).create[0]);
    assert!(p.iter().all(|q| q[1].abs() < 1e-9) && p.iter().any(|q| q[2] > 1.0), "in the xz plane: {p:?}");
}

#[test]
fn helix_and_spiral_options() {
    let r = run(HELIX, vec![]);
    assert_eq!(r[0].prompt, "Start of axis");
    assert_eq!(names(&r[0]), ["Vertical", "AroundCurve"]);
    let r = run(HELIX, vec![pt(0.0, 0.0), pt(10.0, 0.0)]);
    assert_eq!(r[2].prompt, "Radius and start point <1>");
    assert_eq!(names(&r[2]), ["Diameter", "Mode=Turns", "Turns=10", "Pitch=1", "ReverseTwist=No", "Undo"]);
    // Pitch 2 along a 10 mm axis: 5 turns of radius 1.
    let r = run(HELIX, vec![pt(0.0, 0.0), pt(10.0, 0.0), set("Pitch", "2"), text("1")]);
    let g = created(&r);
    let samples = g::nurbs::samples(&g, 2000);
    assert!(samples.iter().all(|p| ((p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0).abs() < 2e-3), "radius 1 around x");
    // Count the turns: sign changes of y going up.
    let ups = samples.windows(2).filter(|w| w[0][1] < 0.0 && w[1][1] >= 0.0).count();
    assert_eq!(ups, 5);
    // ReverseTwist turns the other way.
    let a = g::nurbs::samples(&created(&run(HELIX, vec![pt(0.0, 0.0), pt(10.0, 0.0), text("1")])), 400)[5];
    let b = g::nurbs::samples(&created(&run(HELIX, vec![pt(0.0, 0.0), pt(10.0, 0.0), opt("ReverseTwist"), text("1")])), 400)[5];
    assert!(a[1] * b[1] < 0.0, "{a:?} {b:?}");
    // Vertical: the height from the second click's distance.
    let g = created(&run(HELIX, vec![opt("Vertical"), pt(0.0, 0.0), pt(4.0, 0.0), text("1")]));
    let top = g::eval_geo(&g, 1.0).unwrap();
    assert!((top[2] - 4.0).abs() < 1e-9, "{top:?}");
    // Flat spiral: radius 1 to 5.
    let g = created(&run(SPIRAL, vec![opt("Flat"), pt(0.0, 0.0), text("1"), text("5")]));
    let (p0, p1) = (g::eval_geo(&g, 0.0).unwrap(), g::eval_geo(&g, 1.0).unwrap());
    assert!((norm(p0) - 1.0).abs() < 1e-9 && (norm(p1) - 5.0).abs() < 1e-9 && p1[2].abs() < 1e-12);
    // Diameter: typed numbers are diameters.
    let g = created(&run(SPIRAL, vec![opt("Flat"), pt(0.0, 0.0), opt("Diameter"), text("2"), text("10")]));
    assert!((norm(g::eval_geo(&g, 1.0).unwrap()) - 5.0).abs() < 1e-9);
}

#[test]
fn helix_around_a_curve() {
    let rail = Selected {
        id: "r".into(),
        kind: "curve".into(),
        points: vec![[0.0; 3], [10.0, 0.0, 0.0]],
        exact: Some(Geo::Line { start: [0.0; 3], end: [10.0, 0.0, 0.0] }),
        ..Default::default()
    };
    let mut st = Value::Null;
    let mut go = |input: Input, sel: &[Selected], pick: Option<Vec3>| {
        let mut e = ev(HELIX, input, &st, sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
        st = r.state.clone();
        r
    };
    go(Input::Start, &[], None);
    let r = go(opt("AroundCurve"), &[], None);
    assert_eq!(r.prompt, "Select curve");
    let r = go(Input::Selection, std::slice::from_ref(&rail), Some([5.0, 0.0, 0.0]));
    assert!(r.prompt.starts_with("Radius and start point"), "{}", r.prompt);
    let r = go(text("2"), std::slice::from_ref(&rail), None);
    assert!(r.done);
    for p in g::nurbs::samples(&r.create[0], 400) {
        assert!(((p[1] * p[1] + p[2] * p[2]).sqrt() - 2.0).abs() < 1e-3, "{p:?}");
    }
}

#[test]
fn control_point_curve_closes_smoothly_unless_sharp() {
    let pts = [pt(0.0, 0.0), pt(4.0, 0.0), pt(4.0, 4.0), pt(0.0, 4.0)];
    let mut inputs = pts.to_vec();
    inputs.push(opt("Close"));
    let g = created(&run(CONTROL, inputs));
    assert!(dist(at(&g, 0.0), at(&g, 1.0)) < 1e-9, "closed");
    assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) < 1e-4, "smooth seam");
    let mut inputs = pts.to_vec();
    inputs.push(opt("Sharp"));
    inputs.push(opt("Close"));
    let g = created(&run(CONTROL, inputs));
    assert!(dist(tangent(&g, 0.0), tangent(&g, 1.0)) > 1e-2, "kink at the seam");
}

#[test]
fn line_curves_are_picked_where_clicked() {
    let a = circle_sel("a", 0.0, 0.0, 1.0);
    let b = circle_sel("b", 10.0, 0.0, 1.0);
    let mut st = Value::Null;
    let mut go = |input: Input, sel: &[Selected], pick: Option<Vec3>| {
        let mut e = ev(LINE, input, &st, sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
        st = r.state.clone();
        r
    };
    // Tangent › 2Curves: two clicks near the tops give the upper common tangent at once.
    go(Input::Start, &[], None);
    go(opt("Tangent"), &[], None);
    go(opt("2Curves"), &[], None);
    go(Input::Selection, std::slice::from_ref(&a), Some([0.1, 1.0, 0.0]));
    let r = go(Input::Selection, &[a.clone(), b.clone()], Some([9.9, 1.0, 0.0]));
    assert!(r.done, "{}", r.prompt);
    let Geo::Line { start, end } = r.create[0] else { panic!() };
    assert!((start[1] - 1.0).abs() < 1e-4 && (end[1] - 1.0).abs() < 1e-4, "{start:?} {end:?}");
    // Perpendicular: the click on the curve is the start; only the end is asked.
    let mut st2 = Value::Null;
    let mut go2 = |input: Input, sel: &[Selected], pick: Option<Vec3>| {
        let mut e = ev(LINE, input, &st2, sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
        st2 = r.state.clone();
        r
    };
    go2(Input::Start, &[], None);
    go2(opt("Perpendicular"), &[], None);
    let r = go2(Input::Selection, std::slice::from_ref(&a), Some([1.0, 0.0, 0.0]));
    assert_eq!(r.prompt, "End of line");
    let r = go2(pt(5.0, 0.0), std::slice::from_ref(&a), None);
    let Geo::Line { start, end } = r.create[0] else { panic!() };
    assert!(dist(start, [1.0, 0.0, 0.0]) < 1e-6 && dist(end, [5.0, 0.0, 0.0]) < 1e-6, "{start:?} {end:?}");
}


// ---- shape audit: every creation command and variant, checked against its geometry ----------

/// Steps with a selection and a click location each; records the finished shape.
fn picks(command: &str, steps: Vec<(Input, Vec<Selected>, Option<Vec3>)>) -> Reply {
    let mut st = Value::Null;
    let mut last = Reply::default();
    let mut sel_last = Vec::new();
    for (i, (input, sel, pick)) in std::iter::once((Input::Start, Vec::new(), None)).chain(steps).enumerate() {
        let mut e = ev(command, input, &st, &sel);
        e.pick = pick;
        let r = Curves::step(&e).unwrap();
        assert!(r.error.is_none(), "{command} step {i}: {:?} ({})", r.error, r.prompt);
        st = r.state.clone();
        sel_last = sel;
        last = r;
    }
    assert!(last.done, "{command}: {}", last.prompt);
    let inputs: Vec<Geo> = sel_last.iter().filter_map(|s| s.exact.clone()).collect();
    let test = std::thread::current().name().unwrap_or("").to_owned();
    om9_addon_geom::audit::record("curves", &test, command, &inputs, &last.create);
    last
}
fn ends(g: &Geo) -> (Vec3, Vec3) {
    (at(g, 0.0), at(g, 1.0))
}
fn lin(id: &str, a: Vec3, b: Vec3) -> Selected {
    crv(id, Geo::Line { start: a, end: b })
}
fn close(a: Vec3, b: Vec3) -> bool {
    dist(a, b) < 1e-6
}
fn radius_of(g: &Geo) -> (Vec3, f64, Vec3) {
    match g {
        Geo::Circle { center, radius, x, y } => (*center, *radius, unit(cross(*x, *y))),
        other => panic!("not a circle: {other:?}"),
    }
}

#[test]
fn audit_line_variants() {
    // Plain: exactly the two clicks.
    let g = created(&run(LINE, vec![pt(1.0, 2.0), pt(7.0, 5.0)]));
    assert_eq!(ends(&g), ([1.0, 2.0, 0.0], [7.0, 5.0, 0.0]));
    // A typed length then a direction click.
    let g = created(&run(LINE, vec![pt(0.0, 0.0), text("4"), pt(10.0, 0.0)]));
    assert!(close(ends(&g).1, [4.0, 0.0, 0.0]), "{g:?}");
    // BothSides: the first click is the middle (Rhino 5: twice the distance).
    let g = created(&run(LINE, vec![opt("BothSides"), pt(0.0, 0.0), pt(3.0, 0.0)]));
    let (a, b) = ends(&g);
    assert!(close(a, [-3.0, 0.0, 0.0]) && close(b, [3.0, 0.0, 0.0]) || close(b, [-3.0, 0.0, 0.0]) && close(a, [3.0, 0.0, 0.0]), "{a:?} {b:?}");
    // Vertical: along the CPlane normal; a typed length is a distance constraint (Rhino),
    // the click then picks the side.
    let g = created(&run(LINE, vec![opt("Vertical"), pt(2.0, 2.0), text("5"), Input::Point { point: [2.0, 2.0, 1.0] }]));
    let (a, b) = ends(&g);
    assert!(close(a, [2.0, 2.0, 0.0]) && close(b, [2.0, 2.0, 5.0]), "{a:?} {b:?}");
    // Angled: base line along x, pivot 30°, end click at distance 4 → the line at 30°.
    let g = created(&run(LINE, vec![opt("Angled"), pt(0.0, 0.0), pt(10.0, 0.0), text("30"), pt(4.0, 4.0)]));
    let (a, b) = ends(&g);
    let d = unit(sub(b, a));
    assert!((d[1].atan2(d[0]).to_degrees() - 30.0).abs() < 1e-6, "angle {:?}", d);
    // FourPoint: direction from the reference line, start and end from the last two.
    let g = created(&run(LINE, vec![opt("FourPoint"), pt(0.0, 0.0), pt(0.0, 1.0), pt(5.0, 0.0), pt(7.0, 3.0)]));
    let (a, b) = ends(&g);
    assert!(close(a, [5.0, 0.0, 0.0]) && (b[0] - 5.0).abs() < 1e-6, "parallel to y: {a:?} {b:?}");
    // Bisector: vertex (0,0), sides along x and y → along the 45° diagonal, typed length 2.
    let g = created(&run(LINE, vec![opt("Bisector"), pt(0.0, 0.0), pt(5.0, 0.0), pt(0.0, 5.0), text("2"), pt(3.0, 3.0)]));
    let (a, b) = ends(&g);
    assert!(close(a, [0.0; 3]) && close(b, [2f64.sqrt(), 2f64.sqrt(), 0.0]), "{a:?} {b:?}");
    // Tangent from a circle at its top: horizontal through (0, 2).
    let c = circle_sel("c", 0.0, 0.0, 2.0);
    let r = picks(LINE, vec![
        (opt("Tangent"), vec![], None),
        (Input::Selection, vec![c.clone()], Some([0.0, 2.0, 0.0])),
        (pt(5.0, 2.0), vec![c.clone()], None),
    ]);
    let (a, b) = ends(&r.create[0]);
    assert!((a[1] - 2.0).abs() < 1e-6 && (b[1] - 2.0).abs() < 1e-6, "{a:?} {b:?}");
    // Tangent from a point outside a circle: the line touches the circle.
    let r = picks(LINE, vec![
        (opt("Tangent"), vec![], None),
        (opt("Point"), vec![], None),
        (Input::Selection, vec![c.clone()], Some([0.5, 2.0, 0.0])),
        (pt(10.0, 0.0), vec![c.clone()], None),
        (pt(0.5, 2.0), vec![c.clone()], None),
    ]);
    let (a, b) = ends(&r.create[0]);
    let foot = if close(a, [10.0, 0.0, 0.0]) { b } else { a };
    assert!((norm(foot) - 2.0).abs() < 1e-4, "ends on the circle: {foot:?}");
    assert!(dot(foot, sub([10.0, 0.0, 0.0], foot)).abs() < 1e-3, "tangent: radius ⟂ line");
    // Perpendicular from a point to a line: the foot is the projection.
    let l = lin("l", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let r = picks(LINE, vec![
        (opt("Perpendicular"), vec![], None),
        (opt("Point"), vec![], None),
        (Input::Selection, vec![l.clone()], Some([3.0, 0.0, 0.0])),
        (pt(3.0, 5.0), vec![l.clone()], None),
        (pt(3.2, 0.0), vec![l.clone()], None),
    ]);
    let (a, b) = ends(&r.create[0]);
    assert!(close(a, [3.0, 5.0, 0.0]) && close(b, [3.0, 0.0, 0.0]) || close(b, [3.0, 5.0, 0.0]) && close(a, [3.0, 0.0, 0.0]), "{a:?} {b:?}");
    // Perpendicular 2Curves between two parallel lines: a common normal of length 4.
    let m = lin("m", [0.0, 4.0, 0.0], [10.0, 4.0, 0.0]);
    let r = picks(LINE, vec![
        (opt("Perpendicular"), vec![], None),
        (opt("2Curves"), vec![], None),
        (Input::Selection, vec![l.clone()], Some([2.0, 0.0, 0.0])),
        (Input::Selection, vec![l.clone(), m.clone()], Some([2.5, 4.0, 0.0])),
    ]);
    let (a, b) = ends(&r.create[0]);
    assert!((dist(a, b) - 4.0).abs() < 1e-6 && (a[0] - b[0]).abs() < 1e-6, "{a:?} {b:?}");
    // Extension: continues the line's end along it.
    let r = picks(LINE, vec![
        (opt("Extension"), vec![], None),
        (Input::Selection, vec![l.clone()], Some([9.0, 0.0, 0.0])),
        (pt(14.0, 1.0), vec![l.clone()], None),
    ]);
    let (a, b) = ends(&r.create[0]);
    assert!(close(a, [10.0, 0.0, 0.0]) && b[1].abs() < 1e-9 && b[0] > 10.0, "{a:?} {b:?}");
    // Extension with a typed distance (Rhino 5): exactly that much longer.
    let r = picks(LINE, vec![
        (opt("Extension"), vec![], None),
        (Input::Selection, vec![l.clone()], Some([9.0, 0.0, 0.0])),
        (text("3"), vec![l.clone()], None),
        (pt(20.0, 0.0), vec![l.clone()], None),
    ]);
    assert_eq!(ends(&r.create[0]), ([10.0, 0.0, 0.0], [13.0, 0.0, 0.0]));
}

#[test]
fn audit_point_polyline_rectangle() {
    let r = run(POINT, vec![pt(3.0, 4.0)]);
    assert_eq!(ok(&r).create[0], Geo::Point { point: [3.0, 4.0, 0.0] });
    // Polyline: corners exactly at the clicks.
    let g = created(&run(POLYLINE, vec![pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 5.0), pt(0.0, 5.0), Input::Enter]));
    let pts: Vec<Vec3> = (0..=3).map(|i| at(&g, i as f64 / 3.0)).collect();
    assert_eq!(pts, [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 5.0, 0.0], [0.0, 5.0, 0.0]]);
    // PersistentClose: closed after Enter.
    let g = created(&run(POLYLINE, vec![opt("PersistentClose"), pt(0.0, 0.0), pt(10.0, 0.0), pt(5.0, 5.0), Input::Enter]));
    assert!(close(at(&g, 0.0), at(&g, 1.0)), "closed");
    // Rectangle corner to corner: 10 × 6 with corners at the clicks.
    let corners = |g: &Geo| match g {
        Geo::Polyline { points, closed: true } => points.clone(),
        other => panic!("{other:?}"),
    };
    let p = corners(&created(&run(RECTANGLE, vec![pt(0.0, 0.0), pt(10.0, 6.0)])));
    assert_eq!(p, [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 6.0, 0.0], [0.0, 6.0, 0.0]]);
    // Center: center (0,0), corner (5,3) → 10 × 6 around the center.
    let p = corners(&created(&run(RECTANGLE, vec![opt("Center"), pt(0.0, 0.0), pt(5.0, 3.0)])));
    let (xs, ys): (Vec<f64>, Vec<f64>) = p.iter().map(|q| (q[0], q[1])).unzip();
    assert!(xs.iter().all(|x| (x.abs() - 5.0).abs() < 1e-9) && ys.iter().all(|y| (y.abs() - 3.0).abs() < 1e-9), "{p:?}");
    // 3Point: an edge along a 45° line, width 2 → a rotated rectangle with right angles.
    let p = corners(&created(&run(RECTANGLE, vec![opt("3Point"), pt(0.0, 0.0), pt(4.0, 4.0), pt(0.0, 3.0)])));
    for k in 0..4 {
        let (a, b, c) = (p[k], p[(k + 1) % 4], p[(k + 2) % 4]);
        assert!(dot(sub(b, a), sub(c, b)).abs() < 1e-9, "right angle at {b:?}");
    }
    assert!((dist(p[0], p[1]) - 32f64.sqrt()).abs() < 1e-9);
    // Vertical: stands on the CPlane, height from the third click.
    let p = corners(&created(&run(RECTANGLE, vec![opt("Vertical"), pt(0.0, 0.0), pt(4.0, 0.0), text("3")])));
    assert!(p.iter().all(|q| q[1].abs() < 1e-9) && p.iter().any(|q| (q[2] - 3.0).abs() < 1e-9), "{p:?}");
}

#[test]
fn audit_circle_and_ellipse_variants() {
    // Center + radius, Diameter.
    let (c, r, n) = radius_of(&created(&run(CIRCLE, vec![pt(1.0, 1.0), text("3")])));
    assert!(close(c, [1.0, 1.0, 0.0]) && (r - 3.0).abs() < 1e-12 && (n[2].abs() - 1.0).abs() < 1e-12);
    let (_, r, _) = radius_of(&created(&run(CIRCLE, vec![pt(0.0, 0.0), opt("Diameter"), text("10")])));
    assert!((r - 5.0).abs() < 1e-12);
    // 2Point: the diameter ends.
    let (c, r, _) = radius_of(&created(&run(CIRCLE, vec![opt("2Point"), pt(-2.0, 0.0), pt(2.0, 0.0)])));
    assert!(close(c, [0.0; 3]) && (r - 2.0).abs() < 1e-9);
    // 3Point: through three points of the circle radius 5 at the origin.
    let (c, r, _) = radius_of(&created(&run(CIRCLE, vec![opt("3Point"), pt(5.0, 0.0), pt(0.0, 5.0), pt(-3.0, -4.0)])));
    assert!(close(c, [0.0; 3]) && (r - 5.0).abs() < 1e-9, "{c:?} {r}");
    // Vertical: the circle stands on the CPlane (normal in the plane).
    let (_, _, n) = radius_of(&created(&run(CIRCLE, vec![opt("Vertical"), pt(0.0, 0.0), pt(3.0, 0.0)])));
    assert!(n[2].abs() < 1e-9, "{n:?}");
    // Ellipse from center: axes 5 and 3.
    let g = created(&run(ELLIPSE, vec![pt(0.0, 0.0), pt(5.0, 0.0), pt(0.0, 3.0)]));
    for i in 0..64 {
        let p = at(&g, i as f64 / 64.0);
        assert!(((p[0] / 5.0).powi(2) + (p[1] / 3.0).powi(2) - 1.0).abs() < 1e-6, "{p:?}");
    }
}

#[test]
fn audit_polygon_spiral_helix() {
    // Inscribed polygon: every vertex at the radius, equal sides, sharp corners.
    for n in 3..=8 {
        let r = run(POLYGON, vec![set("NumSides", &n.to_string()), pt(0.0, 0.0), text("5")]);
        let p = poly_pts(&ok(&r).create[0]);
        assert_eq!(p.len(), n);
        let side = dist(p[0], p[1]);
        for k in 0..n {
            assert!((norm(p[k]) - 5.0).abs() < 1e-9);
            assert!((dist(p[k], p[(k + 1) % n]) - side).abs() < 1e-9);
        }
        om9_addon_geom::audit::record("curves", &format!("polygon_{n}"), POLYGON, &[], &ok(&r).create);
    }
    // Spiral along an axis: 3D, radius 2 → 5 over the axis 0 → 10, ends on the radii.
    let g = created(&run(SPIRAL, vec![pt(0.0, 0.0), Input::Point { point: [0.0, 0.0, 10.0] }, text("2"), text("5")]));
    let (a, b) = ends(&g);
    assert!(((a[0] * a[0] + a[1] * a[1]).sqrt() - 2.0).abs() < 1e-6 && a[2].abs() < 1e-6, "{a:?}");
    assert!(((b[0] * b[0] + b[1] * b[1]).sqrt() - 5.0).abs() < 1e-6 && (b[2] - 10.0).abs() < 1e-6, "{b:?}");
    // The radius grows steadily with height (no wobble).
    for p in g::nurbs::samples(&g, 400) {
        let want = 2.0 + 3.0 * p[2] / 10.0;
        assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - want).abs() < 1e-3, "{p:?}");
    }
    // Helix vertical: 10 turns by default along the height, radius exact.
    let g = created(&run(HELIX, vec![opt("Vertical"), pt(0.0, 0.0), pt(0.0, 5.0), text("2")]));
    for p in g::nurbs::samples(&g, 2000) {
        assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - 2.0).abs() < 1e-3, "{p:?}");
    }
}


#[test]
fn click_then_type_numbers_makes_the_shape() {
    // The usual Matrix habit: click the point, type the numbers.
    let (_, r, _) = radius_of(&created(&run(CIRCLE, vec![pt(0.0, 0.0), text("5")])));
    assert!((r - 5.0).abs() < 1e-12);
    let p = poly_pts(&created(&run(POLYGON, vec![pt(0.0, 0.0), text("5")])));
    assert!(p.iter().all(|q| (norm(*q) - 5.0).abs() < 1e-9));
    let (_, r, a) = arc_of(&created(&run(ARC, vec![pt(0.0, 0.0), text("5"), text("90")])));
    assert!((r - 5.0).abs() < 1e-12 && (a - FRAC_PI_2).abs() < 1e-12);
    let g = created(&run(ELLIPSE, vec![pt(0.0, 0.0), text("5"), text("3")]));
    for i in 0..32 {
        let q = at(&g, i as f64 / 32.0);
        assert!(((q[0] / 5.0).powi(2) + (q[1] / 3.0).powi(2) - 1.0).abs() < 1e-6, "{q:?}");
    }
    let Geo::Polyline { points, .. } = created(&run(RECTANGLE, vec![pt(0.0, 0.0), text("10"), text("6")])) else { panic!() };
    assert_eq!(points[2], [10.0, 6.0, 0.0]);
    // Line: a typed length then Enter goes along the CPlane x axis.
    let g = created(&run(LINE, vec![pt(1.0, 1.0), text("5"), Input::Enter]));
    assert_eq!(ends(&g), ([1.0, 1.0, 0.0], [6.0, 1.0, 0.0]));
}
