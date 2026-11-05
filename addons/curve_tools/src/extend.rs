// SPDX-License-Identifier: LGPL-2.1-or-later
//! Extend Curve as Matrix runs it (`_Extend _Type=_Natural`):
//!
//! 1. `Select boundary objects or enter extension length. Press Enter for dynamic extend
//!    ( Type=Natural )`: select boundary curves then Enter, type a length, or Enter alone.
//! 2. `Select curve to extend ( Type=Natural )`: click near the end to extend. With
//!    boundaries the end runs to the first boundary it meets; with a length it grows by that
//!    length; with dynamic extend `End of extension` follows. More curves can be picked;
//!    Enter applies them all.
//!
//! `Type`: Natural (lines stay lines, arcs arcs, other curves continue their last span),
//! Line, Arc (tangent arc with the end's curvature), Smooth (the arc merged into one curve).
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Replacement, Reply, Vec3, Want, cross, dist, dot,
};
use om9_addon_geom::cmd::{load, save};
use om9_addon_geom::join::{self, Extension};
use om9_addon_geom::pick::CurveRef;
use serde::{Deserialize, Serialize};

const TYPES: [&str; 4] = ["Natural", "Line", "Arc", "Smooth"];

#[derive(Default, Serialize, Deserialize)]
struct State {
    /// 0: boundaries/length, 1: curves to extend, 2: end of a dynamic extension.
    stage: u8,
    #[serde(default)]
    kind: String,
    length: Option<f64>,
    boundaries: Vec<CurveRef>,
    /// Extended so far: (id, new curve).
    done: Vec<(String, Geo)>,
    /// Dynamic extension waiting for its end point: the curve and which end.
    dynamic: Option<(CurveRef, bool)>,
    /// Rhino's sub-options at `End of extension`: `ToPoint` (Arc, Smooth: end exactly at the
    /// pick) or `Center` (Arc: pick the arc centre first).
    #[serde(default)]
    mode: String,
    #[serde(default)]
    center: Option<Vec3>,
}

fn how(kind: &str) -> Extension {
    match kind {
        "Line" => Extension::Line,
        "Arc" => Extension::Arc,
        "Smooth" => Extension::Smooth,
        _ => Extension::Natural,
    }
}

/// `c` extended at its end (`at_end`) far enough, with the parameter of the original end.
fn long(c: &CurveRef, at_end: bool, reach: f64, kind: &str) -> Option<(CurveRef, f64)> {
    let g = join::extend_geo(&c.geo(), at_end, reach, how(kind))?;
    let x = CurveRef {
        exact: Some(g),
        ..c.clone()
    };
    let t = x.closest(c.at(if at_end { 1.0 } else { 0.0 })).0;
    Some((x, t))
}

fn size(c: &CurveRef) -> f64 {
    (0..=16)
        .map(|i| dist(c.at(i as f64 / 16.0), c.at(0.0)))
        .fold(0.0, f64::max)
        .max(1.0)
}

/// Cut the long curve at `t` keeping the original side.
fn cut(x: &CurveRef, at_end: bool, t: f64) -> Option<Geo> {
    if at_end { x.sub(0.0, t) } else { x.sub(t, 1.0) }
}

/// The end runs to the first boundary crossing beyond it.
fn to_boundary(
    c: &CurveRef,
    at_end: bool,
    kind: &str,
    boundaries: &[CurveRef],
    n: Vec3,
) -> Option<Geo> {
    let reach = 4.0 * (size(c) + boundaries.iter().map(size).sum::<f64>());
    let (x, t0) = long(c, at_end, reach, kind)?;
    // March from the original end outward; the first sign change against any boundary.
    let steps = 4000;
    let (from, to) = if at_end { (t0, 1.0) } else { (t0, 0.0) };
    let side = |b: &CurveRef, p: Vec3| -> f64 {
        let (s, q) = b.closest(p);
        dot(cross(b.tangent(s), om9_addon_abi::session::sub(p, q)), n)
    };
    let mut best: Option<f64> = None;
    for b in boundaries {
        let mut prev_t = from;
        let mut prev = side(b, x.at(from));
        for i in 1..=steps {
            let t = from + (to - from) * i as f64 / steps as f64;
            let p = x.at(t);
            let v = side(b, p);
            // A real crossing: the sign changes where the curve is near the boundary.
            if prev * v < 0.0 && dist(b.closest(p).1, p) < 4.0 * dist(x.at(prev_t), p) + 1e-9 {
                let (mut lo, mut hi, mut flo) = (prev_t, t, prev);
                for _ in 0..80 {
                    let mid = (lo + hi) / 2.0;
                    let fm = side(b, x.at(mid));
                    if (fm < 0.0) == (flo < 0.0) {
                        lo = mid;
                        flo = fm;
                    } else {
                        hi = mid;
                    }
                }
                let hit = (lo + hi) / 2.0;
                if best.is_none_or(|bt| (hit - from).abs() < (bt - from).abs()) {
                    best = Some(hit);
                }
                break;
            }
            prev = v;
            prev_t = t;
        }
    }
    cut(&x, at_end, best?)
}

fn ask(st: &State) -> Reply {
    let kind = if st.kind.is_empty() {
        "Natural"
    } else {
        st.kind.as_str()
    };
    let mut options = vec![OptionToken::choice("Type", kind, &TYPES)];
    let (prompt, want) = match st.stage {
        0 => (
            "Select boundary objects or enter extension length. Press Enter for dynamic extend"
                .to_owned(),
            Want::Objects,
        ),
        2 => {
            // Rhino 5: the Arc type offers Center and ToPoint.
            if st.kind == "Arc" {
                options.push(OptionToken::action("Center"));
                options.push(OptionToken::action("ToPoint"));
            }
            let prompt = if st.mode == "Center" && st.center.is_none() {
                "Center of arc"
            } else {
                "End of extension"
            };
            (prompt.to_owned(), Want::Point)
        }
        _ => {
            if let Some(l) = st.length {
                options.push(OptionToken::number("ExtensionLength", Some(l)));
            }
            ("Select curve to extend".to_owned(), Want::Objects)
        }
    };
    let mut r = Reply::ask(prompt, save(st))
        .with_options(options)
        .want(want);
    r.preview = st.done.iter().map(|d| d.1.clone()).collect();
    if let Some((c, at_end)) = &st.dynamic {
        r = r.rubber(Some(c.at(if *at_end { 1.0 } else { 0.0 })));
    }
    r
}

fn apply(st: &State) -> Reply {
    if st.done.is_empty() {
        return Reply::finish(Vec::new()).say("Extend: nothing extended");
    }
    let n = st.done.len();
    let mut r = Reply::finish(Vec::new()).say(format!("Extend: {n} curves"));
    r.replace = st
        .done
        .iter()
        .map(|(id, geo)| Replacement {
            id: id.clone(),
            geo: geo.clone(),
        })
        .collect();
    r
}

pub(crate) fn extend(e: &Event) -> Reply {
    let mut st: State = load(e);
    if st.kind.is_empty() {
        st.kind = "Natural".into();
    }
    let kind = st.kind.clone();
    // Options and typed lengths first.
    match &e.input {
        Input::Option { name, value } if name.eq_ignore_ascii_case("Type") => {
            st.kind = match value {
                Some(v) => TYPES
                    .iter()
                    .find(|t| t.eq_ignore_ascii_case(v))
                    .map(|t| t.to_string())
                    .unwrap_or(kind),
                None => TYPES
                    [(TYPES.iter().position(|t| *t == st.kind).unwrap_or(0) + 1) % TYPES.len()]
                .into(),
            };
            return ask(&st);
        }
        Input::Option { name, value } if name.eq_ignore_ascii_case("ExtensionLength") => {
            match value
                .as_deref()
                .and_then(om9_addon_abi::session::parse_number)
            {
                Some(v) if v > 0.0 => st.length = Some(v),
                _ => return ask(&st).refuse("ExtensionLength: type a length more than 0"),
            }
            return ask(&st);
        }
        Input::Text { text } if st.stage <= 1 => {
            return match om9_addon_abi::session::parse_number(text) {
                Some(v) if v > 0.0 => {
                    st.length = Some(v);
                    st.boundaries.clear();
                    st.stage = 1;
                    ask(&st)
                }
                _ => ask(&st).refuse(format!("{text}: type a length")),
            };
        }
        _ => {}
    }
    match st.stage {
        0 => {
            if matches!(e.input, Input::Enter) {
                st.boundaries = CurveRef::all(e);
                st.stage = 1;
            }
            ask(&st)
        }
        2 => {
            let Some((c, at_end)) = st.dynamic.clone() else {
                st.stage = 1;
                return ask(&st);
            };
            if let Input::Option { name, .. } = &e.input {
                if name.eq_ignore_ascii_case("ToPoint") && st.kind == "Arc" {
                    st.mode = "ToPoint".into();
                } else if name.eq_ignore_ascii_case("Center") && st.kind == "Arc" {
                    st.mode = "Center".into();
                    st.center = None;
                }
                return ask(&st);
            }
            if st.mode == "Center" && st.center.is_none() {
                if let Input::Point { point } = &e.input {
                    st.center = Some(*point);
                }
                return ask(&st);
            }
            match &e.input {
                Input::Point { point } | Input::Hover { point }
                    if !st.mode.is_empty() && st.kind != "Natural" && st.kind != "Line" =>
                {
                    let made = to_point(&c, at_end, &st.kind, &st.mode, st.center, *point);
                    if matches!(e.input, Input::Hover { .. }) {
                        let mut r = ask(&st);
                        r.preview.extend(made);
                        return r;
                    }
                    match made {
                        Some(g) => {
                            st.done.retain(|d| d.0 != c.id);
                            st.done.push((c.id.clone(), g));
                            st.dynamic = None;
                            st.mode.clear();
                            st.center = None;
                            st.stage = 1;
                            ask(&st)
                        }
                        None => ask(&st).refuse("Could not extend to that point"),
                    }
                }
                Input::Point { point } | Input::Hover { point } => {
                    let reach =
                        4.0 * (size(&c) + dist(c.at(if at_end { 1.0 } else { 0.0 }), *point));
                    let made = long(&c, at_end, reach, &st.kind).and_then(|(x, t0)| {
                        // The point on the extension nearest the pick (never shorter).
                        let (t, _) = x.closest(*point);
                        let t = if at_end { t.max(t0) } else { t.min(t0) };
                        cut(&x, at_end, t)
                    });
                    if matches!(e.input, Input::Hover { .. }) {
                        let mut r = ask(&st);
                        r.preview.extend(made);
                        return r;
                    }
                    match made {
                        Some(g) => {
                            st.done.retain(|d| d.0 != c.id);
                            st.done.push((c.id.clone(), g));
                            st.dynamic = None;
                            st.stage = 1;
                            ask(&st)
                        }
                        None => ask(&st).refuse("Could not extend to that point"),
                    }
                }
                Input::Enter => {
                    st.dynamic = None;
                    st.stage = 1;
                    ask(&st)
                }
                _ => ask(&st),
            }
        }
        _ => {
            if matches!(e.input, Input::Enter) {
                return apply(&st);
            }
            if !matches!(e.input, Input::Selection | Input::Start) {
                return ask(&st);
            }
            let mut taken: Vec<&str> = st.boundaries.iter().map(|b| b.id.as_str()).collect();
            taken.extend(st.done.iter().map(|d| d.0.as_str()));
            let Some((c, t)) = CurveRef::picked(e, &taken) else {
                return ask(&st);
            };
            if c.closed {
                return ask(&st).refuse("Closed curves cannot be extended");
            }
            let at_end = t.unwrap_or(1.0) >= 0.5;
            // Work from the latest version of this curve.
            let made = if let Some(l) = st.length {
                join::extend_geo(&c.geo(), at_end, l, how(&st.kind))
            } else if st.boundaries.is_empty() {
                st.dynamic = Some((c, at_end));
                st.stage = 2;
                return ask(&st);
            } else {
                to_boundary(&c, at_end, &st.kind, &st.boundaries, e.plane.normal())
            };
            match made {
                Some(g) => {
                    st.done.push((c.id.clone(), g));
                    ask(&st)
                }
                None => ask(&st).refuse("The curve does not reach a boundary"),
            }
        }
    }
}

/// The extension that ends exactly at `p` (Rhino `ToPoint`, and Arc `Center`): a tangent arc
/// (Arc), or a curvature-continuous curve (Smooth), joined to the curve.
fn to_point(
    c: &CurveRef,
    at_end: bool,
    kind: &str,
    mode: &str,
    center: Option<Vec3>,
    p: Vec3,
) -> Option<Geo> {
    use om9_addon_abi::session::{add, norm, scale, sub};
    use om9_addon_geom::blend::{self, Continuity};
    let t = if at_end { 1.0 } else { 0.0 };
    let end = blend::end_of(c, t, true);
    let e = end.point;
    let tan = end.d[0];
    let piece = match (kind, mode) {
        ("Arc", "Center") => {
            // The centre slides onto the end's normal so the arc stays tangent; the end
            // point sets the sweep.
            let k = center?;
            let v = sub(k, e);
            let inward = sub(v, scale(tan, dot(v, tan)));
            if norm(inward) < 1e-9 {
                return None;
            }
            let r = norm(inward);
            let ctr = add(e, inward);
            let x = scale(sub(e, ctr), 1.0 / r);
            let u = sub(p, ctr);
            let mut angle = dot(u, tan).atan2(dot(u, x));
            if angle <= 0.0 {
                angle += std::f64::consts::TAU;
            }
            Geo::Arc {
                center: ctr,
                x,
                y: tan,
                radius: r,
                angle,
            }
        }
        ("Arc", _) => blend::arc_from(e, tan, p)?,
        _ => {
            let target = blend::End {
                point: p,
                d: [[0.0; 3]; 4],
            };
            blend::bezier_blend(&end, Continuity::Curvature, &target, Continuity::Position, [1.0, 1.0])
        }
    };
    let base = c.geo();
    let joined = if at_end {
        join::join(&[base, piece], 1e-6)?
    } else {
        // Keep the curve's direction: the piece is joined before its start.
        let r = join::reverse(&piece)?;
        join::join(&[r, base], 1e-6)?
    };
    Some(joined)
}
