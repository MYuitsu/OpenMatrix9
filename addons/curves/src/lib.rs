// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix 9 **Curve** menu commands for OM9, written from the Matrix/Rhino behaviour
//! (prompts, command-line options, steps), not copied from it.
//!
//! Every command is a step function: OM9 sends the input, the command returns its new state,
//! its prompt and options, a preview and finally the curve to create. See the tests for the
//! step-by-step use of each command.
use om9_addon_abi::session::{
    self as s, Event, Geo, Input, Interactive, OptionToken, Reply, Vec3, dist, dot, fmt, norm,
    parse_number, scale, sub, unit,
};
use om9_addon_geom as g;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

pub const POLYLINE: &str = "CurvePolylinePolyline";
pub const LINE: &str = "CurveLineSingleLine";
pub const INTERP: &str = "CurveFreeFormInterpolatePoints";
pub const CONTROL: &str = "CurveFreeFormControlPoints";
pub const RECTANGLE: &str = "CurveRectangleCornertoCorner";
pub const CIRCLE: &str = "CurveCircleCenterRadius";
pub const ELLIPSE: &str = "CurveEllipseFromCenter";
pub const ARC: &str = "CurveArcCenterStartAngle";
pub const ARC_SED: &str = "CurveArcStartEndDirection";
pub const POLYGON: &str = "CurvePolygonCenterRadius";
pub const SPIRAL: &str = "CurveSpiral";
pub const HELIX: &str = "CurveHelix";
pub const POINT: &str = "CurvePointObjectSinglePoint";

/// Points closer than this to the first point close a polyline (AutoClose), in mm.
pub const AUTO_CLOSE: f64 = 0.05;

pub struct Curves;

impl Interactive for Curves {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        POLYLINE, LINE, INTERP, CONTROL, RECTANGLE, CIRCLE, ELLIPSE, ARC, ARC_SED, POLYGON, SPIRAL,
        HELIX, POINT,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            POLYLINE => polyline::step(e),
            LINE => line::step(e),
            INTERP => interp::step(e),
            CONTROL => points_curve(e),
            RECTANGLE => rect::step(e),
            CIRCLE => circle::step(e),
            ELLIPSE => ellipse::step(e),
            ARC | ARC_SED => arc::step(e),
            POLYGON => polygon::step(e),
            SPIRAL => spiral::step(e, false),
            HELIX => spiral::step(e, true),
            POINT => point(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

#[cfg(feature = "export")]
om9_addon_abi::export_interactive!(Curves);

// ---- helpers -------------------------------------------------------------------------------

pub mod arc;
pub mod circle;
pub mod ellipse;
pub mod interp;
pub mod line;
pub use om9_addon_geom::pick;
pub mod polygon;
pub mod polyline;
pub mod rect;
pub mod spiral;

/// A sub-option that opens its own choice (`Knots=Uniform` → `Knots <Uniform> ( … )`).
pub(crate) fn submenu(name: &str, value: &str) -> OptionToken {
    OptionToken {
        name: name.into(),
        value: Some(value.into()),
        kind: om9_addon_abi::session::OptionKind::Action,
    }
}

pub(crate) fn load<T: DeserializeOwned + Default>(v: &Value) -> T {
    serde_json::from_value(v.clone()).unwrap_or_default()
}
pub(crate) fn save<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).unwrap_or(Value::Null)
}
pub(crate) fn yes(v: &Option<String>, current: bool) -> bool {
    match v.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("yes" | "y" | "true" | "1" | "on") => true,
        Some("no" | "n" | "false" | "0" | "off") => false,
        _ => !current,
    }
}
pub(crate) fn number_of(v: &Option<String>) -> Option<f64> {
    v.as_deref().and_then(parse_number)
}
/// The point projected on the active CPlane (through its origin moved to `base`'s height).
pub(crate) fn on_plane(e: &Event, base: Vec3, p: Vec3) -> Vec3 {
    let n = e.plane.normal();
    sub(p, scale(n, dot(sub(p, base), n)))
}
/// Unit direction in the CPlane from `a` to `b` (CPlane x when they coincide).
pub(crate) fn dir_in_plane(e: &Event, a: Vec3, b: Vec3) -> Vec3 {
    let d = sub(on_plane(e, a, b), a);
    if norm(d) < g::EPS { e.plane.x } else { unit(d) }
}

// ---- Control-point curve (Interpolate Points is in interp.rs) ----------------------------

#[derive(Serialize, Deserialize)]
struct PointsState {
    points: Vec<Vec3>,
    degree: usize,
    persistent_close: bool,
    #[serde(default)]
    sharp: bool,
}
impl Default for PointsState {
    fn default() -> Self {
        PointsState {
            points: Vec::new(),
            degree: 3,
            persistent_close: false,
            sharp: false,
        }
    }
}

fn build_points(pts: &[Vec3], degree: usize, closed: bool, sharp: bool) -> Option<Geo> {
    // Close smoothly (periodic) unless Sharp=Yes.
    if closed && !sharp && pts.len() > 2 {
        return g::interp::periodic_poles(pts, degree);
    }
    let mut p = pts.to_vec();
    if closed && p.len() > 2 {
        p.push(p[0]);
    }
    g::control_curve(&p, degree)
}

fn points_curve(e: &Event) -> Reply {
    let mut st: PointsState = load(&e.state);
    let name = "curve";
    let reply = |st: &PointsState, hover: Option<Vec3>| {
        let mut pts = st.points.clone();
        if let Some(h) = hover.filter(|_| !pts.is_empty()) {
            pts.push(h);
        }
        let mut preview: Vec<Geo> = build_points(&pts, st.degree, st.persistent_close, st.sharp)
            .into_iter()
            .collect();
        if pts.len() >= 2 {
            preview.push(Geo::Polyline {
                points: pts.clone(),
                closed: false,
            }); // control polygon
        }
        let mut opts = vec![
            OptionToken::number("Degree", Some(st.degree as f64)),
            OptionToken::toggle("PersistentClose", st.persistent_close),
        ];
        if st.points.len() >= 3 {
            opts.push(OptionToken::action("Close"));
            opts.push(OptionToken::toggle("Sharp", st.sharp));
        }
        if !st.points.is_empty() {
            opts.push(OptionToken::action("Undo"));
        }
        let prompt = if st.points.is_empty() {
            format!("Start of {name}")
        } else {
            "Next point. Press Enter when done".to_owned()
        };
        Reply::ask(prompt, save(st))
            .with_options(opts)
            .preview(preview)
    };
    let finish = |st: &PointsState, closed: bool| match build_points(&st.points, st.degree, closed, st.sharp)
    {
        Some(c) => Reply::finish(vec![c]).say(format!("Curve through {} points", st.points.len())),
        None => Reply::finish(vec![]).say("Curve cancelled: needs two points"),
    };
    match &e.input {
        Input::Hover { point } => reply(&st, Some(*point)),
        Input::Point { point } => {
            if st.points.len() >= 3 && dist(*point, st.points[0]) <= AUTO_CLOSE {
                return finish(&st, true);
            }
            st.points.push(*point);
            reply(&st, None)
        }
        Input::Option { name, value } => match name.as_str() {
            "Degree" => match number_of(value) {
                Some(d) if (1.0..=11.0).contains(&d) && d.fract() == 0.0 => {
                    st.degree = d as usize;
                    reply(&st, None)
                }
                _ => reply(&st, None).refuse("Degree is a whole number from 1 to 11"),
            },
            "PersistentClose" => {
                st.persistent_close = yes(value, st.persistent_close);
                reply(&st, None)
            }
            "Close" => finish(&st, true),
            "Sharp" => {
                st.sharp = yes(value, st.sharp);
                reply(&st, None)
            }
            _ => reply(&st, None),
        },
        Input::Undo => {
            st.points.pop();
            reply(&st, None)
        }
        Input::Enter => finish(&st, st.persistent_close),
        _ => reply(&st, None),
    }
}

// ---- Point ---------------------------------------------------------------------------------

pub fn point(e: &Event) -> Reply {
    match &e.input {
        Input::Point { point } => Reply::finish(vec![Geo::Point { point: *point }]).say(format!(
            "Point {}, {}, {}",
            fmt(point[0]),
            fmt(point[1]),
            fmt(point[2])
        )),
        Input::Enter => Reply::finish(vec![]),
        _ => Reply::ask("Location of point object", Value::Null),
    }
}

/// Test helper: an event for `command` with `input` after `state`.
pub fn event(command: &str, input: Input, state: &Value) -> Event {
    Event {
        command: command.into(),
        input,
        state: state.clone(),
        plane: s::Plane::default(),
        selection: Vec::new(),
        pick: None,
        settings: Default::default(),
    }
}

#[cfg(test)]
mod variants_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    /// Run a command through a list of inputs; returns every reply.
    fn run(command: &str, inputs: Vec<Input>) -> Vec<Reply> {
        let mut state = Value::Null;
        let mut out = Vec::new();
        for input in std::iter::once(Input::Start).chain(inputs) {
            let hover = matches!(input, Input::Hover { .. });
            let r = Curves::step(&event(command, input, &state)).unwrap();
            if !hover {
                state = r.state.clone();
            }
            out.push(r);
        }
        out
    }
    fn pt(x: f64, y: f64) -> Input {
        Input::Point { point: [x, y, 0.0] }
    }
    fn opt(name: &str, value: Option<&str>) -> Input {
        Input::Option {
            name: name.into(),
            value: value.map(str::to_owned),
        }
    }
    fn text(t: &str) -> Input {
        Input::Text { text: t.into() }
    }

    #[test]
    fn polyline_collects_points_with_undo_length_and_autoclose() {
        let r = run(
            POLYLINE,
            vec![
                pt(0.0, 0.0),
                pt(10.0, 0.0),
                Input::Hover {
                    point: [10.0, 5.0, 0.0],
                },
            ],
        );
        assert_eq!(r[0].prompt, "Start of polyline");
        assert!(r[2].options.iter().any(|o| o.name == "Length"));
        assert!(
            !r[2].options.iter().any(|o| o.name == "Close"),
            "Close needs three points"
        );
        let Geo::Polyline { points, .. } = &r[3].preview[0] else {
            panic!()
        };
        assert_eq!(points.len(), 3, "rubber segment to the cursor");
        // Length=5 constrains the next segment's distance, keeping the clicked direction.
        let r = run(
            POLYLINE,
            vec![
                pt(0.0, 0.0),
                opt("Length", Some("5")),
                pt(100.0, 0.0),
                Input::Enter,
            ],
        );
        let last = r.last().unwrap();
        assert!(last.done);
        let Geo::Polyline { points, closed } = &last.create[0] else {
            panic!()
        };
        assert_eq!(points[1], [5.0, 0.0, 0.0]);
        assert!(!closed);
        // Undo removes the last point; clicking near the start closes it (AutoClose).
        let r = run(
            POLYLINE,
            vec![
                pt(0.0, 0.0),
                pt(10.0, 0.0),
                pt(99.0, 99.0),
                Input::Undo,
                pt(10.0, 10.0),
                pt(0.0, 0.01),
            ],
        );
        let last = r.last().unwrap();
        assert!(last.done);
        let Geo::Polyline { points, closed } = &last.create[0] else {
            panic!()
        };
        assert_eq!(points.len(), 3);
        assert!(*closed);
        // One point + Enter creates nothing.
        let r = run(POLYLINE, vec![pt(0.0, 0.0), Input::Enter]);
        assert!(r.last().unwrap().done && r.last().unwrap().create.is_empty());
        // A bad length is refused and the command keeps going.
        let r = run(POLYLINE, vec![pt(0.0, 0.0), opt("Length", Some("-1"))]);
        assert!(r.last().unwrap().error.is_some() && !r.last().unwrap().done);
    }

    #[test]
    fn single_line_with_both_sides_and_length() {
        let r = run(
            LINE,
            vec![
                opt("BothSides", None),
                pt(0.0, 0.0),
                opt("Length", Some("10")),
                pt(3.0, 0.0),
            ],
        );
        let Geo::Line { start, end } = r.last().unwrap().create[0] else {
            panic!()
        };
        // BothSides draws the length on each side (Rhino 5: twice the length you indicate).
        assert_eq!((start, end), ([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]));
    }

    #[test]
    fn interpolated_and_control_curves() {
        let r = run(
            INTERP,
            vec![
                pt(0.0, 0.0),
                pt(5.0, 5.0),
                pt(10.0, 0.0),
                pt(15.0, 5.0),
                Input::Enter,
            ],
        );
        let c = &r.last().unwrap().create[0];
        assert!(matches!(c, Geo::Nurbs { degree: 3, .. }));
        for t in [0.0, 1.0] {
            let p = g::eval_geo(c, t).unwrap();
            assert!(dist(p, if t == 0.0 { [0.0; 3] } else { [15.0, 5.0, 0.0] }) < 1e-9);
        }
        let r = run(
            CONTROL,
            vec![
                opt("Degree", Some("2")),
                pt(0.0, 0.0),
                pt(5.0, 5.0),
                pt(10.0, 0.0),
                Input::Enter,
            ],
        );
        assert!(
            matches!(&r.last().unwrap().create[0], Geo::Nurbs { degree: 2, points, .. } if points.len() == 3)
        );
        let r = run(CONTROL, vec![opt("Degree", Some("2.5"))]);
        assert!(r.last().unwrap().error.is_some());
    }

    #[test]
    fn rectangle_modes() {
        let corners = |g: &Geo| match g {
            Geo::Polyline {
                points,
                closed: true,
            } => points.clone(),
            _ => panic!("{g:?}"),
        };
        let r = run(RECTANGLE, vec![pt(1.0, 1.0), pt(11.0, 7.0)]);
        assert_eq!(
            corners(&r[2].create[0]),
            vec![
                [1.0, 1.0, 0.0],
                [11.0, 1.0, 0.0],
                [11.0, 7.0, 0.0],
                [1.0, 7.0, 0.0]
            ]
        );
        assert!(r[2].messages[0].contains("10 × 6"));
        let r = run(
            RECTANGLE,
            vec![opt("Center", None), pt(0.0, 0.0), pt(5.0, 2.0)],
        );
        assert_eq!(corners(&r[3].create[0])[0], [-5.0, -2.0, 0.0]);
        // Typed length then width.
        let r = run(RECTANGLE, vec![pt(0.0, 0.0), text("8"), text("3")]);
        assert_eq!(corners(&r[3].create[0])[2], [8.0, 3.0, 0.0]);
        // 3Point: a slanted rectangle.
        let r = run(
            RECTANGLE,
            vec![
                opt("3Point", None),
                pt(0.0, 0.0),
                pt(3.0, 4.0),
                pt(-4.0, 3.0),
            ],
        );
        let c = corners(&r[4].create[0]);
        assert!(dist(c[3], [-4.0, 3.0, 0.0]) < 1e-9);
        // Vertical: edge on the CPlane, height along its normal.
        let r = run(
            RECTANGLE,
            vec![
                opt("Vertical", None),
                pt(0.0, 0.0),
                pt(4.0, 0.0),
                Input::Point {
                    point: [4.0, 0.0, 2.0],
                },
            ],
        );
        assert_eq!(corners(&r[4].create[0])[2], [4.0, 0.0, 2.0]);
        // Rounded: the radius comes after the corners.
        let r = run(
            RECTANGLE,
            vec![opt("Rounded", None), pt(0.0, 0.0), pt(10.0, 6.0), text("1")],
        );
        assert!(matches!(&r[4].create[0], Geo::Nurbs { degree: 2, .. }));
        let r = run(
            RECTANGLE,
            vec![opt("Rounded", None), pt(0.0, 0.0), pt(10.0, 6.0), text("4")],
        );
        assert!(
            r[4].error.is_some(),
            "radius larger than half the short side"
        );
    }

    #[test]
    fn circle_modes() {
        let radius = |g: &Geo| match g {
            Geo::Circle { radius, center, .. } => (*center, *radius),
            _ => panic!(),
        };
        let r = run(CIRCLE, vec![pt(1.0, 1.0), pt(4.0, 5.0)]);
        assert_eq!(radius(&r[2].create[0]), ([1.0, 1.0, 0.0], 5.0));
        let r = run(
            CIRCLE,
            vec![pt(0.0, 0.0), opt("Diameter", None), text("17.35")],
        );
        assert!(
            (radius(&r[3].create[0]).1 - 8.675).abs() < 1e-12,
            "ring size typed as a diameter"
        );
        let r = run(
            CIRCLE,
            vec![opt("2Point", None), pt(0.0, 0.0), pt(6.0, 0.0)],
        );
        assert_eq!(radius(&r[3].create[0]), ([3.0, 0.0, 0.0], 3.0));
        let r = run(
            CIRCLE,
            vec![
                opt("3Point", None),
                pt(2.0, 0.0),
                pt(0.0, 2.0),
                pt(-2.0, 0.0),
            ],
        );
        let (c, rr) = radius(&r[4].create[0]);
        assert!(dist(c, [0.0; 3]) < 1e-12 && (rr - 2.0).abs() < 1e-12);
        let r = run(
            CIRCLE,
            vec![
                opt("3Point", None),
                pt(0.0, 0.0),
                pt(1.0, 0.0),
                pt(2.0, 0.0),
            ],
        );
        assert!(r[4].error.is_some(), "collinear points");
        let r = run(
            CIRCLE,
            vec![opt("Vertical", None), pt(0.0, 0.0), pt(3.0, 0.0)],
        );
        let Geo::Circle { y, .. } = r[3].create[0] else {
            panic!()
        };
        assert_eq!(
            y,
            [0.0, 0.0, 1.0],
            "vertical circle stands on the CPlane normal"
        );
    }

    #[test]
    fn ellipse_arc_polygon() {
        let r = run(ELLIPSE, vec![pt(0.0, 0.0), pt(4.0, 0.0), pt(1.0, 2.0)]);
        let e0 = &r[3].create[0];
        assert!(dist(g::eval_geo(e0, 0.25).unwrap(), [0.0, 2.0, 0.0]) < 1e-9);
        // Arc: center, start, 90° typed.
        let r = run(ARC, vec![pt(0.0, 0.0), pt(3.0, 0.0), text("90")]);
        let a = &r[3].create[0];
        assert!(dist(g::eval_geo(a, 1.0).unwrap(), [0.0, 3.0, 0.0]) < 1e-9);
        // Negative angle goes clockwise.
        let r = run(ARC, vec![pt(0.0, 0.0), pt(3.0, 0.0), text("-90")]);
        assert!(dist(g::eval_geo(&r[3].create[0], 1.0).unwrap(), [0.0, -3.0, 0.0]) < 1e-9);
        // End point picks the angle counter-clockwise from the start.
        let r = run(ARC, vec![pt(0.0, 0.0), pt(3.0, 0.0), pt(-1.0, 0.0)]);
        let Geo::Arc { angle, .. } = r[3].create[0] else {
            panic!()
        };
        assert!((angle - PI).abs() < 1e-12);
        let r = run(ARC_SED, vec![pt(2.0, 0.0), pt(0.0, 2.0), pt(2.0, 1.0)]);
        assert!(dist(g::eval_geo(&r[3].create[0], 1.0).unwrap(), [0.0, 2.0, 0.0]) < 1e-9);
        // Hexagon with the corner on +x; circumscribed puts the click on an edge middle.
        let r = run(
            POLYGON,
            vec![opt("NumSides", Some("6")), pt(0.0, 0.0), pt(2.0, 0.0)],
        );
        let Geo::Polyline {
            points,
            closed: true,
        } = &r[3].create[0]
        else {
            panic!()
        };
        assert_eq!(points.len(), 6);
        assert!(points.iter().all(|p| (norm(*p) - 2.0).abs() < 1e-12));
        let r = run(
            POLYGON,
            vec![
                opt("Circumscribed", None),
                opt("NumSides", Some("4")),
                pt(0.0, 0.0),
                pt(1.0, 0.0),
            ],
        );
        let Geo::Polyline { points, .. } = &r[4].create[0] else {
            panic!()
        };
        assert!(
            points
                .iter()
                .all(|p| (norm(*p) - 2f64.sqrt()).abs() < 1e-12),
            "square around a unit circle"
        );
    }

    #[test]
    fn helix_and_flat_spiral() {
        let r = run(
            HELIX,
            vec![
                opt("Turns", Some("3")),
                pt(0.0, 0.0),
                Input::Point {
                    point: [0.0, 0.0, 6.0],
                },
                pt(2.0, 0.0),
            ],
        );
        let h = &r[4].create[0];
        assert!(dist(g::eval_geo(h, 1.0).unwrap(), [2.0, 0.0, 6.0]) < 1e-6);
        assert!(r[4].messages[0].contains("3 turns"));
        let r = run(
            SPIRAL,
            vec![
                opt("Flat", None),
                opt("Turns", Some("2")),
                pt(0.0, 0.0),
                pt(1.0, 0.0),
                pt(5.0, 0.0),
            ],
        );
        let sp = &r[5].create[0];
        for i in 0..=20 {
            assert!(
                g::eval_geo(sp, i as f64 / 20.0).unwrap()[2].abs() < 1e-9,
                "flat"
            );
        }
        assert!(dist(g::eval_geo(sp, 1.0).unwrap(), [5.0, 0.0, 0.0]) < 1e-6);
        let r = run(POINT, vec![pt(1.0, 2.0)]);
        assert_eq!(
            r[1].create[0],
            Geo::Point {
                point: [1.0, 2.0, 0.0]
            }
        );
    }
}
