// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix 9 **Transform** menu and Top 11 commands for OM9: Move, Duplicate, Rotate, Rotate 3D,
//! Mirror, Scale 3D/2D/1D, Array Polar, Array Linear, Array. Behaviour follows the Matrix /
//! Rhino command flow (select → base points → preview → Enter), written independently.
//!
//! The add-on never touches geometry: it answers with 4×4 matrices; OM9 applies them to the
//! selection (or to copies of it) in one transaction.
use om9_addon_abi::session::{
    self as s, Event, Input, Interactive, OptionToken, Plane, Reply, TransformOp, Vec3, Want, add,
    cross, dist, dot, fmt, norm, parse_number, scale, sub, unit,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::f64::consts::TAU;

pub const MOVE: &str = "TopIconMove";
pub const COPY: &str = "TopIconDuplicate";
pub const ROTATE: &str = "TopIconRotate";
pub const MIRROR: &str = "TopIconMirror";
pub const ROTATE3D: &str = "TransformRotate3D";
pub const SCALE3D: &str = "TransformScaleScale3D";
pub const SCALE2D: &str = "TransformScaleScale2D";
pub const SCALE1D: &str = "TransformScaleScale1D";
pub const ARRAY_POLAR: &str = "TransformArrayPolar";
pub const ARRAY_LINEAR: &str = "TransformArrayArrayLinear";
pub const ARRAY: &str = "TransformArrayRectangular";

pub struct Transforms;

impl Interactive for Transforms {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        MOVE,
        COPY,
        ROTATE,
        MIRROR,
        ROTATE3D,
        SCALE3D,
        SCALE2D,
        SCALE1D,
        ARRAY_POLAR,
        ARRAY_LINEAR,
        ARRAY,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        let verb = match e.command.as_str() {
            MOVE => "move",
            COPY => "copy",
            ROTATE | ROTATE3D => "rotate",
            MIRROR => "mirror",
            SCALE3D | SCALE2D | SCALE1D => "scale",
            _ => "array",
        };
        let mut st: State = load(&e.state);
        // Every command starts by making sure something is selected.
        if !st.selected {
            if e.selection.is_empty() {
                return Ok(Reply::ask(
                    format!("Select objects to {verb}. Press Enter when done"),
                    save(&st),
                )
                .want(Want::Objects)
                .refuse_if(matches!(e.input, Input::Enter), "Nothing is selected"));
            }
            st.selected = true;
            st.center = Some(selection_center(e));
            // The selection event only advances to the first real prompt.
            if !matches!(e.input, Input::Start | Input::Selection | Input::Enter) {
                // fall through with the input (e.g. a click right after pre-selection)
            } else {
                return Ok(step_command(e, st, None));
            }
        }
        if matches!(e.input, Input::Selection) {
            return Ok(step_command(e, st, None));
        }
        Ok(step_command(e, st, Some(&e.input)))
    }
}

#[cfg(feature = "export")]
om9_addon_abi::export_interactive!(Transforms);

trait RefuseIf {
    fn refuse_if(self, cond: bool, why: &str) -> Self;
}
impl RefuseIf for Reply {
    fn refuse_if(self, cond: bool, why: &str) -> Self {
        if cond { self.refuse(why) } else { self }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct State {
    selected: bool,
    center: Option<Vec3>,
    points: Vec<Vec3>,
    copy: Option<bool>,
    /// Matrices of the copies placed so far (Duplicate places several).
    placed: Vec<[f64; 16]>,
    count: Option<usize>,
    counts: Vec<usize>,
    angle: Option<f64>,
    factor: Option<f64>,
}

fn load<T: DeserializeOwned + Default>(v: &Value) -> T {
    serde_json::from_value(v.clone()).unwrap_or_default()
}
fn save<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).unwrap_or(Value::Null)
}

fn selection_center(e: &Event) -> Vec3 {
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for o in &e.selection {
        for i in 0..3 {
            lo[i] = lo[i].min(o.bbox[0][i]);
            hi[i] = hi[i].max(o.bbox[1][i]);
        }
    }
    if lo[0].is_finite() {
        scale(add(lo, hi), 0.5)
    } else {
        e.plane.origin
    }
}

fn yes(v: &Option<String>, current: bool) -> bool {
    match v.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("yes" | "y" | "true" | "1") => true,
        Some("no" | "n" | "false" | "0") => false,
        _ => !current,
    }
}

/// Signed angle at `c` from `a` to `b`, measured around `axis`.
fn angle_between(c: Vec3, a: Vec3, b: Vec3, axis: Vec3) -> Option<f64> {
    let n = unit(axis);
    let pa = sub(a, c);
    let pb = sub(b, c);
    let pa = sub(pa, scale(n, dot(pa, n)));
    let pb = sub(pb, scale(n, dot(pb, n)));
    if norm(pa) < 1e-9 || norm(pb) < 1e-9 {
        return None;
    }
    Some(dot(cross(pa, pb), n).atan2(dot(pa, pb)))
}

/// Projection of `p` on the line through `a` along `d`, as a distance from `a`.
fn along(a: Vec3, d: Vec3, p: Vec3) -> f64 {
    dot(sub(p, a), unit(d))
}

fn finish_move(m: [f64; 16], copy: bool, msg: String) -> Reply {
    Reply::finish_transform(TransformOp {
        matrices: vec![m],
        keep_original: copy,
        per_object: Vec::new(),
    })
    .say(msg)
}

fn step_command(e: &Event, mut st: State, input: Option<&Input>) -> Reply {
    match e.command.as_str() {
        MOVE | COPY => move_copy(e, &mut st, input),
        ROTATE => rotate(e, &mut st, input, false),
        ROTATE3D => rotate(e, &mut st, input, true),
        MIRROR => mirror_cmd(e, &mut st, input),
        SCALE3D | SCALE2D | SCALE1D => scale_cmd(e, &mut st, input),
        ARRAY_POLAR => array_polar(e, &mut st, input),
        ARRAY_LINEAR => array_linear(e, &mut st, input),
        _ => array_rect(e, &mut st, input),
    }
}

// ---- Move / Duplicate ----------------------------------------------------------------------

fn move_copy(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    let duplicate = e.command == COPY;
    let ask = |st: &State, hover: Option<Vec3>| {
        let from = st.points.first().copied();
        let mut previews = st.placed.clone();
        if let (Some(a), Some(h)) = (from, hover) {
            previews.push(s::translation(sub(h, a)));
        }
        let mut opts = Vec::new();
        if duplicate && from.is_none() {
            opts.push(OptionToken::action("InPlace"));
        }
        let prompt = match (from, duplicate) {
            (None, true) => "Point to copy from",
            (None, false) => "Point to move from",
            (Some(_), true) => "Point to copy to. Press Enter when done",
            (Some(_), false) => "Point to move to",
        };
        let mut r = Reply::ask(prompt, save(st)).with_options(opts).rubber(from);
        r.preview_transforms = previews;
        r
    };
    match input {
        None => ask(st, None),
        Some(Input::Hover { point }) => ask(st, Some(*point)),
        Some(Input::Point { point }) => {
            let Some(a) = st.points.first().copied() else {
                st.points.push(*point);
                return ask(st, None);
            };
            let m = s::translation(sub(*point, a));
            let d = dist(*point, a);
            if duplicate {
                st.placed.push(m);
                return ask(st, None);
            }
            finish_move(m, false, format!("Moved {} mm", fmt(d)))
        }
        Some(Input::Text { text }) => {
            // A distance along the cursor direction is not available without a cursor; accept
            // a typed number as a move along the CPlane x axis from the base point.
            match (parse_number(text), st.points.first().copied()) {
                (Some(v), Some(a)) => {
                    let m = s::translation(scale(e.plane.x, v));
                    if duplicate {
                        st.placed.push(m);
                        let _ = a;
                        return ask(st, None);
                    }
                    finish_move(m, false, format!("Moved {} mm", fmt(v)))
                }
                _ => ask(st, None).refuse(format!("Not a point: {text}")),
            }
        }
        Some(Input::Option { name, .. }) if name == "InPlace" => {
            finish_move(s::IDENTITY, true, "Duplicated in place".into())
        }
        Some(Input::Enter) => {
            if duplicate && !st.placed.is_empty() {
                let n = st.placed.len();
                Reply::finish_transform(TransformOp {
                    matrices: st.placed.clone(),
                    keep_original: true,
                    per_object: Vec::new(),
                })
                .say(format!("{n} copies"))
            } else {
                Reply::finish(vec![])
            }
        }
        _ => ask(st, None),
    }
}

// ---- Rotate / Rotate 3D --------------------------------------------------------------------

fn rotate(e: &Event, st: &mut State, input: Option<&Input>, three_d: bool) -> Reply {
    let copy = st.copy.unwrap_or(false);
    // Points: [center] or [axis start, axis end], then the first reference point.
    let axis_points = if three_d { 2 } else { 1 };
    let axis = |st: &State| -> Option<(Vec3, Vec3)> {
        let c = *st.points.first()?;
        if three_d {
            let b = *st.points.get(1)?;
            (dist(b, c) > 1e-9).then(|| (c, sub(b, c)))
        } else {
            Some((c, e.plane.normal()))
        }
    };
    let ask = |st: &State, hover: Option<Vec3>| {
        let n = st.points.len();
        let prompt = if n == 0 {
            if three_d {
                "Start of rotation axis"
            } else {
                "Center of rotation"
            }
        } else if n < axis_points {
            "End of rotation axis"
        } else if n == axis_points {
            "Angle or first reference point"
        } else {
            "Second reference point"
        };
        let mut r = Reply::ask(prompt, save(st))
            .with_options(vec![OptionToken::toggle("Copy", copy)])
            .want(if n == axis_points {
                Want::Number
            } else {
                Want::Point
            })
            .rubber(st.points.last().copied());
        if let (Some((c, ax)), Some(h), true) = (axis(st), hover, n == axis_points + 1)
            && let Some(a) = angle_between(c, st.points[axis_points], h, ax)
        {
            r.preview_transforms = vec![s::rotation(c, ax, a)];
        }
        r
    };
    let done = |c: Vec3, ax: Vec3, a: f64| {
        finish_move(
            s::rotation(c, ax, a),
            copy,
            format!("Rotated {}°", fmt(a.to_degrees())),
        )
    };
    match input {
        None => ask(st, None),
        Some(Input::Hover { point }) => ask(st, Some(*point)),
        Some(Input::Point { point }) => {
            if st.points.len() <= axis_points {
                st.points.push(*point);
                if three_d && st.points.len() == 2 && axis(st).is_none() {
                    st.points.pop();
                    return ask(st, None).refuse("The axis needs two different points");
                }
                return ask(st, None);
            }
            let (c, ax) = axis(st).expect("axis set");
            match angle_between(c, st.points[axis_points], *point, ax) {
                Some(a) => done(c, ax, a),
                None => ask(st, None).refuse("Reference points must not be on the axis"),
            }
        }
        Some(Input::Text { text }) => match (parse_number(text), axis(st)) {
            (Some(deg), Some((c, ax))) if st.points.len() == axis_points => {
                done(c, ax, deg.to_radians())
            }
            _ => ask(st, None).refuse(format!("Not an angle: {text}")),
        },
        Some(Input::Option { name, value }) if name == "Copy" => {
            st.copy = Some(yes(value, copy));
            ask(st, None)
        }
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st, None),
    }
}

// ---- Mirror --------------------------------------------------------------------------------

fn mirror_cmd(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    // Matrix/Rhino default: Mirror keeps the original (Copy=Yes).
    let copy = st.copy.unwrap_or(true);
    let n = e.plane.normal();
    let plane_of = |a: Vec3, b: Vec3| -> Option<[f64; 16]> {
        let d = sub(b, a);
        let d = sub(d, scale(n, dot(d, n)));
        (norm(d) > 1e-9).then(|| s::mirror(a, cross(d, n)))
    };
    let ask = |st: &State, hover: Option<Vec3>| {
        let first = st.points.first().copied();
        let mut r = Reply::ask(
            if first.is_none() {
                "Start of mirror plane"
            } else {
                "End of mirror plane"
            },
            save(st),
        )
        .with_options({
            let mut o = vec![OptionToken::toggle("Copy", copy)];
            if first.is_none() {
                o.extend([OptionToken::action("XAxis"), OptionToken::action("YAxis")]);
            }
            o
        })
        .rubber(first);
        if let (Some(a), Some(h)) = (first, hover)
            && let Some(m) = plane_of(a, h)
        {
            r.preview_transforms = vec![m];
        }
        r
    };
    match input {
        None => ask(st, None),
        Some(Input::Hover { point }) => ask(st, Some(*point)),
        Some(Input::Point { point }) => match st.points.first().copied() {
            None => {
                st.points.push(*point);
                ask(st, None)
            }
            Some(a) => match plane_of(a, *point) {
                Some(m) => finish_move(m, copy, "Mirrored".into()),
                None => ask(st, None).refuse("Pick a second point away from the first"),
            },
        },
        Some(Input::Option { name, value }) => match name.as_str() {
            "Copy" => {
                st.copy = Some(yes(value, copy));
                ask(st, None)
            }
            // Mirror across the CPlane x or y axis (through the CPlane origin).
            "XAxis" => finish_move(
                s::mirror(e.plane.origin, e.plane.y),
                copy,
                "Mirrored across X".into(),
            ),
            "YAxis" => finish_move(
                s::mirror(e.plane.origin, e.plane.x),
                copy,
                "Mirrored across Y".into(),
            ),
            _ => ask(st, None),
        },
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st, None),
    }
}

// ---- Scale 3D / 2D / 1D --------------------------------------------------------------------

fn scale_cmd(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    let copy = st.copy.unwrap_or(false);
    let kind = e.command.as_str();
    let matrix = |o: Vec3, f: f64, dir: Option<Vec3>| -> [f64; 16] {
        match kind {
            SCALE3D => s::scale_about(o, &e.plane, [f, f, f]),
            SCALE2D => s::scale_about(o, &e.plane, [f, f, 1.0]),
            _ => {
                let x = unit(dir.unwrap_or(e.plane.x));
                let helper = if dot(x, e.plane.normal()).abs() < 0.9 {
                    e.plane.normal()
                } else {
                    e.plane.y
                };
                let y = unit(cross(helper, x));
                s::scale_about(o, &Plane { origin: o, x, y }, [f, 1.0, 1.0])
            }
        }
    };
    let ask = |st: &State, hover: Option<Vec3>| {
        let n = st.points.len();
        let prompt = match n {
            0 => "Origin point",
            1 => "Scale factor or first reference point",
            _ => "Second reference point",
        };
        let mut r = Reply::ask(prompt, save(st))
            .with_options(vec![OptionToken::toggle("Copy", copy)])
            .want(if n == 1 { Want::Number } else { Want::Point })
            .rubber(st.points.first().copied());
        if let (2, Some(h)) = (n, hover) {
            let (o, r1) = (st.points[0], st.points[1]);
            let d = sub(r1, o);
            let f = if kind == SCALE1D {
                along(o, d, h) / norm(d)
            } else {
                dist(o, h) / dist(o, r1)
            };
            if f.is_finite() && f.abs() > 1e-9 {
                r.preview_transforms = vec![matrix(o, f, Some(d))];
            }
        }
        r
    };
    let done = |o: Vec3, f: f64, dir: Option<Vec3>| {
        if !(f.is_finite() && f.abs() > 1e-9) {
            return None;
        }
        Some(finish_move(
            matrix(o, f, dir),
            copy,
            format!("Scale factor {}", fmt(f)),
        ))
    };
    match input {
        None => ask(st, None),
        Some(Input::Hover { point }) => ask(st, Some(*point)),
        Some(Input::Point { point }) => {
            if st.points.len() < 2 {
                if st.points.len() == 1 && dist(st.points[0], *point) < 1e-9 {
                    return ask(st, None).refuse("The reference point must differ from the origin");
                }
                st.points.push(*point);
                return ask(st, None);
            }
            let (o, r1) = (st.points[0], st.points[1]);
            let d = sub(r1, o);
            let f = if kind == SCALE1D {
                along(o, d, *point) / norm(d)
            } else {
                dist(o, *point) / dist(o, r1)
            };
            done(o, f, Some(d))
                .unwrap_or_else(|| ask(st, None).refuse("Scale factor must not be 0"))
        }
        Some(Input::Text { text }) => match (parse_number(text), st.points.first().copied()) {
            (Some(f), Some(o)) if st.points.len() == 1 && f > 0.0 => done(o, f, None)
                .unwrap_or_else(|| ask(st, None).refuse("Scale factor must not be 0")),
            _ => ask(st, None).refuse(format!("Not a scale factor: {text}")),
        },
        Some(Input::Option { name, value }) if name == "Copy" => {
            st.copy = Some(yes(value, copy));
            ask(st, None)
        }
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st, None),
    }
}

// ---- Arrays --------------------------------------------------------------------------------

fn polar_matrices(e: &Event, c: Vec3, count: usize, angle: f64) -> Vec<[f64; 16]> {
    // A full circle spaces the items by 360/n; a partial fill puts the last one on the angle.
    let full = (angle.abs() - TAU).abs() < 1e-9;
    let step = if full {
        angle / count as f64
    } else {
        angle / (count - 1).max(1) as f64
    };
    (1..count)
        .map(|i| s::rotation(c, e.plane.normal(), step * i as f64))
        .collect()
}

fn array_polar(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    let ask = |st: &State| {
        let prompt = match (st.points.is_empty(), st.count) {
            (true, _) => "Center of polar array",
            (false, None) => "Number of items",
            (false, Some(_)) => "Angle to fill. Press Enter for 360",
        };
        let mut r = Reply::ask(prompt, save(st))
            .with_options(vec![
                OptionToken::number("NumberOfItems", st.count.map(|c| c as f64)),
                OptionToken::number("Angle", Some(st.angle.unwrap_or(360.0))),
            ])
            .want(if st.points.is_empty() {
                Want::Point
            } else {
                Want::Number
            });
        if let (Some(c), Some(n)) = (st.points.first(), st.count) {
            r.preview_transforms = polar_matrices(e, *c, n, st.angle.unwrap_or(360.0).to_radians());
        }
        r
    };
    let finish = |st: &State| {
        let (Some(c), Some(n)) = (st.points.first(), st.count) else {
            return ask(st).refuse("Pick the center and the number of items");
        };
        let a = st.angle.unwrap_or(360.0);
        Reply::finish_transform(TransformOp {
            matrices: polar_matrices(e, *c, n, a.to_radians()),
            keep_original: true,
            per_object: Vec::new(),
        })
        .say(format!("Polar array: {n} items over {}°", fmt(a)))
    };
    let set_count = |st: &mut State, v: f64| {
        if v >= 2.0 && v <= 10_000.0 && v.fract() == 0.0 {
            st.count = Some(v as usize);
            None
        } else {
            Some("The number of items must be a whole number of 2 or more")
        }
    };
    match input {
        None | Some(Input::Hover { .. }) => ask(st),
        Some(Input::Point { point }) if st.points.is_empty() => {
            st.points.push(*point);
            ask(st)
        }
        Some(Input::Text { text }) => match parse_number(text) {
            Some(v) if st.count.is_none() && !st.points.is_empty() => match set_count(st, v) {
                None => ask(st),
                Some(why) => ask(st).refuse(why),
            },
            Some(a) if st.count.is_some() && a.abs() > 0.0 && a.abs() <= 360.0 => {
                st.angle = Some(a);
                finish(st)
            }
            _ => ask(st).refuse(format!("Not a valid number: {text}")),
        },
        Some(Input::Option { name, value }) => {
            let v = value.as_deref().and_then(parse_number);
            match (name.as_str(), v) {
                ("NumberOfItems", Some(v)) => {
                    if let Some(why) = set_count(st, v) {
                        return ask(st).refuse(why);
                    }
                }
                ("Angle", Some(a)) if a.abs() > 0.0 && a.abs() <= 360.0 => st.angle = Some(a),
                ("Angle", _) => return ask(st).refuse("The angle is between -360 and 360, not 0"),
                _ => {}
            }
            ask(st)
        }
        Some(Input::Enter) if st.count.is_some() => finish(st),
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st),
    }
}

fn array_linear(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    let _ = e;
    let mats = |st: &State, to: Vec3| -> Vec<[f64; 16]> {
        let (Some(n), Some(a)) = (st.count, st.points.first()) else {
            return Vec::new();
        };
        let d = sub(to, *a);
        (1..n).map(|i| s::translation(scale(d, i as f64))).collect()
    };
    let ask = |st: &State, hover: Option<Vec3>| {
        let prompt = match (st.count, st.points.len()) {
            (None, _) => "Number of items",
            (Some(_), 0) => "First reference point",
            _ => "Second reference point (spacing)",
        };
        let mut r = Reply::ask(prompt, save(st))
            .want(if st.count.is_none() {
                Want::Number
            } else {
                Want::Point
            })
            .rubber(st.points.first().copied());
        if let Some(h) = hover.filter(|_| st.points.len() == 1) {
            r.preview_transforms = mats(st, h);
        }
        r
    };
    match input {
        None => ask(st, None),
        Some(Input::Hover { point }) => ask(st, Some(*point)),
        Some(Input::Text { text }) if st.count.is_none() => match parse_number(text) {
            Some(v) if v >= 2.0 && v.fract() == 0.0 && v <= 10_000.0 => {
                st.count = Some(v as usize);
                ask(st, None)
            }
            _ => ask(st, None).refuse("The number of items must be a whole number of 2 or more"),
        },
        Some(Input::Point { point }) if st.count.is_some() => {
            if st.points.is_empty() {
                st.points.push(*point);
                return ask(st, None);
            }
            let m = mats(st, *point);
            if dist(*point, st.points[0]) < 1e-9 {
                return ask(st, None).refuse("The spacing must not be 0");
            }
            let n = st.count.unwrap_or(0);
            Reply::finish_transform(TransformOp {
                matrices: m,
                keep_original: true,
                per_object: Vec::new(),
            })
            .say(format!(
                "Linear array: {n} items, spacing {} mm",
                fmt(dist(*point, st.points[0]))
            ))
        }
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st, None).refuse_if(st.count.is_none(), "Type the number of items"),
    }
}

fn array_rect(e: &Event, st: &mut State, input: Option<&Input>) -> Reply {
    // Counts in x, y, z, then spacings in x, y, z (typed), like Matrix's Array prompts.
    let labels = [
        "Number in X direction",
        "Number in Y direction",
        "Number in Z direction",
    ];
    let spacing = ["X spacing", "Y spacing", "Z spacing"];
    let ask = |st: &State| {
        let k = st.counts.len();
        let prompt = if k < 3 {
            labels[k].to_owned()
        } else {
            spacing[st.points.len().min(2)].to_owned()
        };
        Reply::ask(prompt, save(st)).want(Want::Number)
    };
    let mats = |st: &State, d: [f64; 3]| -> Vec<[f64; 16]> {
        let n = e.plane.normal();
        let mut out = Vec::new();
        for i in 0..st.counts[0] {
            for j in 0..st.counts[1] {
                for k in 0..st.counts[2] {
                    if i + j + k == 0 {
                        continue;
                    }
                    let v = add(
                        scale(e.plane.x, d[0] * i as f64),
                        add(scale(e.plane.y, d[1] * j as f64), scale(n, d[2] * k as f64)),
                    );
                    out.push(s::translation(v));
                }
            }
        }
        out
    };
    match input {
        None | Some(Input::Hover { .. }) => ask(st),
        Some(Input::Text { text }) => {
            let Some(v) = parse_number(text) else {
                return ask(st).refuse(format!("Not a number: {text}"));
            };
            if st.counts.len() < 3 {
                if v < 1.0 || v.fract() != 0.0 || v > 1000.0 {
                    return ask(st).refuse("Counts are whole numbers from 1");
                }
                st.counts.push(v as usize);
                if st.counts.iter().product::<usize>() > 100_000 {
                    st.counts.pop();
                    return ask(st).refuse("Too many copies");
                }
                return ask(st);
            }
            // Spacings are stored as points [v, 0, 0] for simplicity.
            st.points.push([v, 0.0, 0.0]);
            if st.points.len() < 3 {
                let mut r = ask(st);
                r.preview_transforms = mats(
                    st,
                    [st.points[0][0], st.points.get(1).map_or(0.0, |p| p[0]), 0.0],
                );
                return r;
            }
            let d = [st.points[0][0], st.points[1][0], st.points[2][0]];
            let total = st.counts.iter().product::<usize>();
            Reply::finish_transform(TransformOp {
                matrices: mats(st, d),
                keep_original: true,
                per_object: Vec::new(),
            })
            .say(format!("Array: {total} items"))
        }
        Some(Input::Enter) => Reply::finish(vec![]),
        _ => ask(st),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::{Selected, apply};

    fn sel() -> Vec<Selected> {
        vec![Selected {
            id: "a".into(),
            kind: "curve".into(),
            bbox: [[0.0, 0.0, 0.0], [2.0, 2.0, 0.0]],
            ..Default::default()
        }]
    }
    fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Vec<Reply> {
        let mut state = Value::Null;
        let mut out = Vec::new();
        for input in std::iter::once(Input::Start).chain(inputs) {
            let hover = matches!(input, Input::Hover { .. });
            let e = Event {
                command: command.into(),
                input,
                state: state.clone(),
                plane: Plane::default(),
                selection: selection.clone(),
                pick: None,
                settings: Default::default(),
            };
            let r = Transforms::step(&e).unwrap();
            if !hover {
                state = r.state.clone();
            }
            out.push(r);
        }
        out
    }
    fn pt(x: f64, y: f64, z: f64) -> Input {
        Input::Point { point: [x, y, z] }
    }
    fn txt(t: &str) -> Input {
        Input::Text { text: t.into() }
    }
    fn close(a: Vec3, b: Vec3) -> bool {
        dist(a, b) < 1e-9
    }

    #[test]
    fn nothing_selected_asks_for_objects_first() {
        let r = run(MOVE, vec![], vec![Input::Enter]);
        assert_eq!(r[0].want, Want::Objects);
        assert!(r[0].prompt.contains("Select objects to move"));
        assert!(
            r[1].error.is_some() && !r[1].done,
            "Enter with nothing selected is refused"
        );
    }

    #[test]
    fn move_and_duplicate() {
        let r = run(
            MOVE,
            sel(),
            vec![
                pt(1.0, 1.0, 0.0),
                Input::Hover {
                    point: [3.0, 1.0, 0.0],
                },
                pt(4.0, 5.0, 0.0),
            ],
        );
        assert_eq!(r[0].prompt, "Point to move from");
        assert_eq!(
            r[2].preview_transforms.len(),
            1,
            "hover previews the moved selection"
        );
        let op = r[3].transform.as_ref().unwrap();
        assert!(!op.keep_original);
        assert!(close(apply(&op.matrices[0], [0.0; 3]), [3.0, 4.0, 0.0]));
        assert!(r[3].messages[0].contains("Moved 5 mm"));
        // Duplicate places copies until Enter.
        let r = run(
            COPY,
            sel(),
            vec![
                pt(0.0, 0.0, 0.0),
                pt(10.0, 0.0, 0.0),
                pt(20.0, 0.0, 0.0),
                Input::Enter,
            ],
        );
        let op = r[4].transform.as_ref().unwrap();
        assert!(op.keep_original);
        assert_eq!(op.matrices.len(), 2);
        let r = run(
            COPY,
            sel(),
            vec![Input::Option {
                name: "InPlace".into(),
                value: None,
            }],
        );
        assert_eq!(r[1].transform.as_ref().unwrap().matrices[0], s::IDENTITY);
    }

    #[test]
    fn rotate_by_angle_and_by_reference_points() {
        let r = run(ROTATE, sel(), vec![pt(0.0, 0.0, 0.0), txt("90")]);
        let m = r[2].transform.as_ref().unwrap().matrices[0];
        assert!(close(apply(&m, [1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
        let r = run(
            ROTATE,
            sel(),
            vec![
                Input::Option {
                    name: "Copy".into(),
                    value: None,
                },
                pt(0.0, 0.0, 0.0),
                pt(1.0, 0.0, 0.0),
                pt(-1.0, 0.0, 0.0),
            ],
        );
        let op = r[4].transform.as_ref().unwrap();
        assert!(op.keep_original, "Copy=Yes keeps the original");
        assert!(close(
            apply(&op.matrices[0], [1.0, 0.0, 0.0]),
            [-1.0, 0.0, 0.0]
        ));
        // Rotate 3D about the world x axis.
        let r = run(
            ROTATE3D,
            sel(),
            vec![pt(0.0, 0.0, 0.0), pt(5.0, 0.0, 0.0), txt("90")],
        );
        let m = r[3].transform.as_ref().unwrap().matrices[0];
        assert!(close(apply(&m, [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]));
        let r = run(ROTATE3D, sel(), vec![pt(0.0, 0.0, 0.0), pt(0.0, 0.0, 0.0)]);
        assert!(r[2].error.is_some());
    }

    #[test]
    fn mirror_keeps_the_original_by_default() {
        let r = run(MIRROR, sel(), vec![pt(5.0, 0.0, 0.0), pt(5.0, 10.0, 0.0)]);
        let op = r[2].transform.as_ref().unwrap();
        assert!(op.keep_original);
        assert!(close(
            apply(&op.matrices[0], [1.0, 3.0, 0.0]),
            [9.0, 3.0, 0.0]
        ));
        let r = run(
            MIRROR,
            sel(),
            vec![Input::Option {
                name: "YAxis".into(),
                value: None,
            }],
        );
        assert!(close(
            apply(
                &r[1].transform.as_ref().unwrap().matrices[0],
                [2.0, 1.0, 0.0]
            ),
            [-2.0, 1.0, 0.0]
        ));
    }

    #[test]
    fn scale_3d_2d_1d() {
        let r = run(SCALE3D, sel(), vec![pt(0.0, 0.0, 0.0), txt("2")]);
        let m = r[2].transform.as_ref().unwrap().matrices[0];
        assert!(close(apply(&m, [1.0, 1.0, 1.0]), [2.0, 2.0, 2.0]));
        let r = run(
            SCALE2D,
            sel(),
            vec![pt(0.0, 0.0, 0.0), pt(1.0, 0.0, 0.0), pt(3.0, 0.0, 0.0)],
        );
        let m = r[3].transform.as_ref().unwrap().matrices[0];
        assert!(
            close(apply(&m, [1.0, 1.0, 1.0]), [3.0, 3.0, 1.0]),
            "2D keeps the CPlane normal"
        );
        let r = run(
            SCALE1D,
            sel(),
            vec![pt(0.0, 0.0, 0.0), pt(0.0, 2.0, 0.0), pt(0.0, 5.0, 0.0)],
        );
        let m = r[3].transform.as_ref().unwrap().matrices[0];
        assert!(
            close(apply(&m, [1.0, 2.0, 1.0]), [1.0, 5.0, 1.0]),
            "1D along the picked direction"
        );
        let r = run(SCALE3D, sel(), vec![pt(0.0, 0.0, 0.0), txt("0")]);
        assert!(r[2].error.is_some());
    }

    #[test]
    fn arrays() {
        // Polar: 6 items over 360° → 5 copies at 60° steps.
        let r = run(
            ARRAY_POLAR,
            sel(),
            vec![pt(0.0, 0.0, 0.0), txt("6"), Input::Enter],
        );
        let op = r[3].transform.as_ref().unwrap();
        assert_eq!(op.matrices.len(), 5);
        let p = apply(&op.matrices[0], [10.0, 0.0, 0.0]);
        assert!(close(p, [5.0, 3f64.sqrt() * 5.0, 0.0]));
        // Partial fill: 3 items over 90° → last copy on 90°.
        let r = run(
            ARRAY_POLAR,
            sel(),
            vec![pt(0.0, 0.0, 0.0), txt("3"), txt("90")],
        );
        let op = r[3].transform.as_ref().unwrap();
        assert!(close(
            apply(&op.matrices[1], [1.0, 0.0, 0.0]),
            [0.0, 1.0, 0.0]
        ));
        let r = run(ARRAY_POLAR, sel(), vec![pt(0.0, 0.0, 0.0), txt("1")]);
        assert!(r[2].error.is_some());
        // Linear: 4 items 2.5 mm apart.
        let r = run(
            ARRAY_LINEAR,
            sel(),
            vec![txt("4"), pt(0.0, 0.0, 0.0), pt(2.5, 0.0, 0.0)],
        );
        let op = r[3].transform.as_ref().unwrap();
        assert_eq!(op.matrices.len(), 3);
        assert!(close(apply(&op.matrices[2], [0.0; 3]), [7.5, 0.0, 0.0]));
        // Rectangular 3 × 2 × 1 with 5 / 4 mm spacing → 5 copies.
        let r = run(
            ARRAY,
            sel(),
            vec![txt("3"), txt("2"), txt("1"), txt("5"), txt("4"), txt("0")],
        );
        let op = r[6].transform.as_ref().unwrap();
        assert_eq!(op.matrices.len(), 5);
        assert!(
            op.matrices
                .iter()
                .any(|m| close(apply(m, [0.0; 3]), [10.0, 4.0, 0.0]))
        );
    }
}
