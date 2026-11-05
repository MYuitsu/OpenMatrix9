// SPDX-License-Identifier: LGPL-2.1-or-later
//! Circle (Matrix Curve #5).
//!
//! - `Center of circle ( Deformable Vertical 2Point 3Point Tangent AroundCurve FitPoints )`
//! - `Radius ( Diameter Orientation Circumference Area )` (or `Diameter ( Radius … )`)
//! - `Deformable` turns into `Degree=3 PointCount=10`: a NURBS approximation that can be
//!   edited by its control points.
//! - `Vertical`: the circle stands perpendicular to the CPlane, facing the radius pick.
//! - `2Point`: `Start of diameter`, `End of diameter`; `3Point`: three points on the circle.
//! - `Tangent`: two curves and a radius, or three curves, then a pick near the center.
//! - `AroundCurve`: select a curve, pick the center on it; the circle is perpendicular to it.
//! - `FitPoints`: select point objects, curves (control points) or meshes (vertices), Enter.
use crate::pick::CurveRef;
use crate::{dir_in_plane, load, number_of, on_plane, save, yes};
use om9_addon_abi::session::{
    Event, Geo, Input, OptionToken, Reply, Vec3, Want, add, cross, dist, dot, fmt, norm,
    parse_number, scale, sub, unit,
};
use om9_addon_geom as g;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    /// "center", "vertical", "2point", "3point", "tangent", "around", "fit".
    pub mode: String,
    pub deformable: bool,
    pub degree: usize,
    pub point_count: usize,
    /// Typed size means: "radius", "diameter", "circumference", "area".
    pub size: String,
    pub points: Vec<Vec3>,
    /// Orientation: the normal picked after the center.
    pub normal: Option<Vec3>,
    pub orienting: bool,
    pub curves: Vec<CurveRef>,
    pub radius: Option<f64>,
}

impl Default for State {
    fn default() -> Self {
        State {
            mode: "center".into(),
            deformable: false,
            degree: 3,
            point_count: 10,
            size: "radius".into(),
            points: Vec::new(),
            normal: None,
            orienting: false,
            curves: Vec::new(),
            radius: None,
        }
    }
}

/// Least-squares circle through `pts` (≥ 3): (center, normal, radius).
pub fn fit_circle(pts: &[Vec3]) -> Option<(Vec3, Vec3, f64)> {
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let c0 = scale(pts.iter().fold([0.0; 3], |a, p| add(a, *p)), 1.0 / n);
    // Plane normal: smallest eigenvector of the covariance (power iteration on the inverse
    // is overkill here: take the normal of the best spread triangle, then refine by PCA).
    let mut cov = [[0.0; 3]; 3];
    for p in pts {
        let d = sub(*p, c0);
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    let normal = smallest_eigenvector(cov)?;
    let x = unit(if normal[0].abs() < 0.9 {
        cross(normal, [1.0, 0.0, 0.0])
    } else {
        cross(normal, [0.0, 1.0, 0.0])
    });
    let y = cross(normal, x);
    // Kåsa fit in the plane: x² + y² + D x + E y + F = 0.
    let mut ata = [[0.0; 3]; 3];
    let mut atb = [0.0; 3];
    for p in pts {
        let d = sub(*p, c0);
        let (u, v) = (dot(d, x), dot(d, y));
        let row = [u, v, 1.0];
        let rhs = -(u * u + v * v);
        for i in 0..3 {
            for j in 0..3 {
                ata[i][j] += row[i] * row[j];
            }
            atb[i] += row[i] * rhs;
        }
    }
    let s = solve3(ata, atb)?;
    let (cu, cv) = (-s[0] / 2.0, -s[1] / 2.0);
    let r2 = cu * cu + cv * cv - s[2];
    if r2 <= 0.0 {
        return None;
    }
    let center = add(c0, add(scale(x, cu), scale(y, cv)));
    Some((center, normal, r2.sqrt()))
}

fn solve3(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(a);
    if d.abs() < 1e-18 {
        return None;
    }
    let mut out = [0.0; 3];
    for (k, o) in out.iter_mut().enumerate() {
        let mut m = a;
        for i in 0..3 {
            m[i][k] = b[i];
        }
        *o = det(m) / d;
    }
    Some(out)
}

/// Unit eigenvector of the smallest eigenvalue of a symmetric 3×3 matrix (Jacobi).
#[allow(clippy::needless_range_loop)]
fn smallest_eigenvector(mut a: [[f64; 3]; 3]) -> Option<Vec3> {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        let (mut p, mut q, mut big) = (0, 1, 0.0);
        for i in 0..3 {
            for j in i + 1..3 {
                if a[i][j].abs() > big {
                    big = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }
        if big < 1e-15 {
            break;
        }
        let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = theta.signum().max(0.0) * 2.0 - 1.0;
        let t = t / (theta.abs() + (theta * theta + 1.0).sqrt());
        let c = 1.0 / (t * t + 1.0).sqrt();
        let s = t * c;
        let mut b = a;
        for k in 0..3 {
            b[k][p] = c * a[k][p] - s * a[k][q];
            b[k][q] = s * a[k][p] + c * a[k][q];
        }
        a = b;
        for k in 0..3 {
            b[p][k] = c * a[p][k] - s * a[q][k];
            b[q][k] = s * a[p][k] + c * a[q][k];
        }
        a = b;
        for row in v.iter_mut() {
            let (vp, vq) = (row[p], row[q]);
            row[p] = c * vp - s * vq;
            row[q] = s * vp + c * vq;
        }
    }
    let k = (0..3).min_by(|&i, &j| a[i][i].total_cmp(&a[j][j]))?;
    let e = [v[0][k], v[1][k], v[2][k]];
    (norm(e) > 1e-12).then(|| unit(e))
}

/// Distance from `p` to the closest point of a picked curve.
fn curve_dist(c: &CurveRef, p: Vec3) -> f64 {
    dist(c.closest(p).1, p)
}

/// Circle tangent to the curves, near `guess`, in the plane with normal `n`:
/// two curves and a radius, or three curves (radius solved too). Gauss–Newton with numeric
/// derivatives on the center (and radius).
fn tangent_circle(
    curves: &[CurveRef],
    radius: Option<f64>,
    guess: Vec3,
    x: Vec3,
    y: Vec3,
) -> Option<(Vec3, f64)> {
    let mut u = [0.0, 0.0];
    let at = |u: [f64; 2]| add(guess, add(scale(x, u[0]), scale(y, u[1])));
    let mut r = radius.unwrap_or_else(|| {
        curves.iter().map(|c| curve_dist(c, guess)).sum::<f64>() / curves.len() as f64
    });
    let solve_r = radius.is_none();
    for _ in 0..60 {
        let res = |u: [f64; 2], r: f64| -> Vec<f64> {
            curves.iter().map(|c| curve_dist(c, at(u)) - r).collect()
        };
        let f0 = res(u, r);
        let err: f64 = f0.iter().map(|v| v * v).sum();
        if err < 1e-20 {
            break;
        }
        let h = 1e-6;
        let mut cols: Vec<Vec<f64>> = Vec::new();
        for k in 0..2 {
            let mut du = u;
            du[k] += h;
            cols.push(
                res(du, r)
                    .iter()
                    .zip(&f0)
                    .map(|(a, b)| (a - b) / h)
                    .collect(),
            );
        }
        if solve_r {
            cols.push(vec![-1.0; curves.len()]);
        }
        // Normal equations (J^T J) d = -J^T f.
        let m = cols.len();
        let mut jtj = [[0.0; 3]; 3];
        let mut jtf = [0.0; 3];
        for i in 0..m {
            for j in 0..m {
                jtj[i][j] = cols[i].iter().zip(&cols[j]).map(|(a, b)| a * b).sum();
            }
            jtf[i] = -cols[i].iter().zip(&f0).map(|(a, b)| a * b).sum::<f64>();
        }
        if m == 2 {
            jtj[2][2] = 1.0;
        }
        let d = solve3(jtj, jtf)?;
        u[0] += d[0];
        u[1] += d[1];
        if solve_r {
            r += d[2];
        }
    }
    let center = at(u);
    let err: f64 = curves
        .iter()
        .map(|c| (curve_dist(c, center) - r).abs())
        .fold(0.0, f64::max);
    (err < 1e-4 && r > g::EPS).then_some((center, r))
}

impl State {
    fn shape(&self, center: Vec3, x: Vec3, y: Vec3, r: f64) -> Option<Geo> {
        if r <= g::EPS {
            return None;
        }
        if self.deformable {
            return g::interp::deformable(
                center,
                scale(x, r),
                scale(y, r),
                self.degree,
                self.point_count,
            );
        }
        Some(Geo::Circle {
            center,
            x,
            y,
            radius: r,
        })
    }

    /// Frame (x, y) of the circle at `c` facing the CPlane, a picked orientation, or vertical.
    fn frame(&self, e: &Event, c: Vec3, toward: Option<Vec3>) -> (Vec3, Vec3) {
        if let Some(nrm) = self.normal {
            let x = unit(if nrm[0].abs() < 0.9 {
                cross(nrm, [1.0, 0.0, 0.0])
            } else {
                cross(nrm, [0.0, 1.0, 0.0])
            });
            return (x, cross(nrm, x));
        }
        if self.mode == "vertical" {
            let d = toward.map_or(e.plane.x, |p| dir_in_plane(e, c, p));
            return (d, e.plane.normal());
        }
        (e.plane.x, e.plane.y)
    }

    fn radius_of(&self, v: f64) -> f64 {
        match self.size.as_str() {
            "diameter" => v / 2.0,
            "circumference" => v / std::f64::consts::TAU,
            "area" => (v / std::f64::consts::PI).sqrt(),
            _ => v,
        }
    }

    /// The circle the picks and the cursor `p` define.
    fn circle(&self, e: &Event, p: Vec3) -> Option<Geo> {
        match (self.mode.as_str(), self.points.as_slice()) {
            ("center" | "vertical" | "around", [c]) => {
                let r = if self.mode == "center" && self.normal.is_none() {
                    dist(*c, on_plane(e, *c, p))
                } else {
                    dist(*c, p)
                };
                let (x, y) = self.frame(e, *c, Some(p));
                self.shape(*c, x, y, r)
            }
            ("2point", [a]) => {
                let q = on_plane(e, *a, p);
                let x = unit(sub(q, *a));
                let n = e.plane.normal();
                (dist(*a, q) > g::EPS).then_some(())?;
                self.shape(scale(add(*a, q), 0.5), x, cross(n, x), dist(*a, q) / 2.0)
            }
            ("3point", [a, b]) => {
                let c = g::circumcenter(*a, *b, p)?;
                let nn = unit(cross(sub(*b, *a), sub(p, *a)));
                let x = unit(sub(*a, c));
                self.shape(c, x, cross(nn, x), dist(c, *a))
            }
            _ => None,
        }
    }

    fn sized_prompt(&self) -> String {
        match self.size.as_str() {
            "diameter" => "Diameter",
            "circumference" => "Circumference",
            "area" => "Area",
            _ => "Radius",
        }
        .into()
    }

    fn reply(&self, e: &Event, hover: Option<Vec3>) -> Reply {
        let n = self.points.len();
        let mut opts = Vec::new();
        let mut want = Want::Point;
        let prompt: String = if self.orienting {
            "Point to set the circle's normal direction".into()
        } else {
            match (self.mode.as_str(), n) {
                ("tangent", _) if self.curves.len() < 2 => {
                    want = Want::Objects;
                    if self.curves.is_empty() {
                        "Select first tangent curve"
                    } else {
                        "Select second tangent curve"
                    }
                    .into()
                }
                ("tangent", 0) if self.radius.is_none() && self.curves.len() == 2 => {
                    want = Want::Objects;
                    opts.push(OptionToken::number("Radius", None));
                    "Select third tangent curve or type a radius".into()
                }
                ("tangent", _) => "Point near the circle's center".into(),
                ("around", _) if self.curves.is_empty() => {
                    want = Want::Objects;
                    "Select curve to draw circle around".into()
                }
                ("around", 0) => "Center of circle on the curve".into(),
                ("fit", _) => {
                    want = Want::Objects;
                    "Select points to build a circle from. Press Enter when done".into()
                }
                ("2point", 0) => "Start of diameter".into(),
                ("2point", _) => "End of diameter".into(),
                ("3point", 0) => "First point on circle".into(),
                ("3point", 1) => "Second point on circle".into(),
                ("3point", _) => "Third point on circle".into(),
                (_, 0) => "Center of circle".into(),
                _ => {
                    want = Want::Number;
                    self.sized_prompt()
                }
            }
        };
        let picking_center = n == 0 && matches!(self.mode.as_str(), "center" | "vertical");
        if picking_center || (n == 0 && matches!(self.mode.as_str(), "2point" | "3point")) {
            if self.deformable {
                opts.push(OptionToken::number("Degree", Some(self.degree as f64)));
                opts.push(OptionToken::number(
                    "PointCount",
                    Some(self.point_count as f64),
                ));
            } else {
                opts.push(OptionToken::action("Deformable"));
            }
        }
        if picking_center {
            if self.mode == "center" {
                opts.push(OptionToken::action("Vertical"));
            }
            opts.push(OptionToken::action("2Point"));
            if self.mode == "center" {
                for o in ["3Point", "Tangent", "AroundCurve", "FitPoints"] {
                    opts.push(OptionToken::action(o));
                }
            }
        }
        if n == 1
            && matches!(self.mode.as_str(), "center" | "vertical" | "around")
            && !self.orienting
        {
            for o in ["Radius", "Diameter"] {
                if self.size != o.to_ascii_lowercase() {
                    opts.push(OptionToken::action(o));
                }
            }
            opts.push(OptionToken::action("Orientation"));
            for o in ["Circumference", "Area"] {
                opts.push(OptionToken::action(o));
            }
        }
        let preview = hover
            .filter(|_| !self.orienting)
            .and_then(|h| self.circle(e, h))
            .into_iter()
            .collect();
        let rubber = self.points.first().copied().filter(|_| {
            matches!(self.mode.as_str(), "center" | "vertical" | "around") || self.orienting
        });
        Reply::ask(prompt, save(self))
            .with_options(opts)
            .want(want)
            .preview(preview)
            .rubber(rubber)
    }
}

fn done(c: Geo, r: f64) -> Reply {
    Reply::finish(vec![c]).say(format!(
        "Circle: radius {} mm, diameter {} mm",
        fmt(r),
        fmt(2.0 * r)
    ))
}

fn radius_of_geo(g: &Geo) -> f64 {
    match g {
        Geo::Circle { radius, .. } => *radius,
        Geo::Nurbs { points, .. } => {
            let c = scale(
                points.iter().fold([0.0; 3], |a, p| add(a, *p)),
                1.0 / points.len() as f64,
            );
            g::eval_geo(g, 0.0).map_or(0.0, |p| dist(p, c))
        }
        _ => 0.0,
    }
}

/// Points of the selection to fit a circle to.
fn fit_points(e: &Event) -> Vec<Vec3> {
    let mut out = Vec::new();
    for s in &e.selection {
        match (s.kind.as_str(), &s.exact) {
            ("point", _) => out.push(scale(add(s.bbox[0], s.bbox[1]), 0.5)),
            (_, Some(Geo::Nurbs { points, .. })) => out.extend(points.iter().copied()),
            (_, Some(Geo::Polyline { points, .. })) => out.extend(points.iter().copied()),
            (_, Some(Geo::Surface { points, .. })) => out.extend(points.iter().copied()),
            _ => {
                if let Some(m) = &s.mesh {
                    out.extend(m.vertices.iter().copied());
                } else {
                    out.extend(s.points.iter().copied());
                }
            }
        }
    }
    out
}

pub fn step(e: &Event) -> Reply {
    let mut st: State = load(&e.state);
    if st.mode.is_empty() {
        st = State::default();
    }
    match &e.input {
        Input::Start | Input::Selection => {
            if st.mode == "tangent" && st.curves.len() < 3 && st.points.is_empty() {
                for c in CurveRef::all(e) {
                    if st.curves.len() < 3
                        && st.curves.iter().all(|k| k.id != c.id)
                        && (st.curves.len() < 2 || st.radius.is_none())
                    {
                        st.curves.push(c);
                    }
                }
            }
            if st.mode == "around" && st.curves.is_empty() {
                // The click on the curve is the center (as in Matrix).
                match CurveRef::picked(e, &[]) {
                    Some((c, Some(t))) if matches!(e.input, Input::Selection) => {
                        st.normal = Some(c.tangent(t));
                        st.points.push(c.at(t));
                        st.curves.push(c);
                    }
                    Some((c, _)) => st.curves.push(c),
                    None => {}
                }
            }
            st.reply(e, None)
        }
        Input::Hover { point } => st.reply(e, Some(*point)),
        Input::Point { point } => {
            if st.orienting {
                let c = st.points[0];
                let d = sub(*point, c);
                if norm(d) < g::EPS {
                    return st
                        .reply(e, None)
                        .refuse("Pick a point away from the center");
                }
                st.normal = Some(unit(d));
                st.orienting = false;
                return st.reply(e, None);
            }
            match st.mode.as_str() {
                "tangent" if st.curves.len() >= 2 => {
                    let (x, y) = (e.plane.x, e.plane.y);
                    let curves = &st.curves;
                    match tangent_circle(curves, st.radius, *point, x, y) {
                        Some((c, r)) => {
                            let geo = st.shape(c, x, y, r);
                            geo.map(|g| done(g, r)).unwrap_or_else(|| st.reply(e, None))
                        }
                        None => st
                            .reply(e, None)
                            .refuse("No tangent circle near that point"),
                    }
                }
                "tangent" => st.reply(e, None).refuse("Select the tangent curves first"),
                "around" if st.points.is_empty() => {
                    let Some(c) = st.curves.first() else {
                        return st.reply(e, None).refuse("Select a curve first");
                    };
                    let (t, q) = c.closest(*point);
                    st.normal = Some(c.tangent(t));
                    st.points.push(q);
                    st.reply(e, None)
                }
                "fit" => st.reply(e, None),
                _ => {
                    let needs = if st.mode == "3point" { 3 } else { 2 };
                    if st.points.len() + 1 < needs {
                        st.points.push(*point);
                        return st.reply(e, None);
                    }
                    match st.circle(e, *point) {
                        Some(c) => {
                            let r = radius_of_geo(&c);
                            done(c, r)
                        }
                        None => st
                            .reply(e, None)
                            .refuse("The points do not define a circle"),
                    }
                }
            }
        }
        Input::Option { name, value } => {
            let at_start = st.points.is_empty() && st.curves.is_empty();
            match name.as_str() {
                "Deformable" => st.deformable = yes(value, st.deformable),
                "Degree" => match number_of(value) {
                    Some(d) if (1.0..=11.0).contains(&d) && d.fract() == 0.0 => {
                        st.degree = d as usize
                    }
                    _ => {
                        return st
                            .reply(e, None)
                            .refuse("Degree is a whole number from 1 to 11");
                    }
                },
                "PointCount" => match number_of(value) {
                    Some(c) if c >= 3.0 && c.fract() == 0.0 && c <= 1000.0 => {
                        st.point_count = c as usize
                    }
                    _ => {
                        return st
                            .reply(e, None)
                            .refuse("PointCount is a whole number from 3");
                    }
                },
                "Vertical" if at_start => st.mode = "vertical".into(),
                "2Point" if at_start => st.mode = "2point".into(),
                "3Point" if at_start => st.mode = "3point".into(),
                "Tangent" if at_start => {
                    st.mode = "tangent".into();
                    return step_selection(st, e);
                }
                "AroundCurve" if at_start => {
                    st.mode = "around".into();
                    return step_selection(st, e);
                }
                "FitPoints" if at_start => st.mode = "fit".into(),
                "Radius" if st.mode == "tangent" => match number_of(value) {
                    Some(r) if r > 0.0 => st.radius = Some(r),
                    Some(_) => return st.reply(e, None).refuse("The radius must not be 0"),
                    None => {}
                },
                "Radius" | "Diameter" | "Circumference" | "Area" => {
                    st.size = name.to_ascii_lowercase();
                    if let Some(v) = number_of(value) {
                        return sized(st, e, v);
                    }
                }
                "Orientation" if st.points.len() == 1 => st.orienting = true,
                _ => return st.reply(e, None).refuse(format!("Unknown option {name}")),
            }
            st.reply(e, None)
        }
        Input::Text { text } => match parse_number(text) {
            Some(v) if st.mode == "tangent" && st.curves.len() == 2 && st.radius.is_none() => {
                if v <= 0.0 {
                    return st.reply(e, None).refuse("The radius must not be 0");
                }
                st.radius = Some(v);
                st.reply(e, None)
            }
            Some(v)
                if st.points.len() == 1
                    && matches!(st.mode.as_str(), "center" | "vertical" | "around") =>
            {
                sized(st, e, v)
            }
            _ => st.reply(e, None).refuse(format!("Not a radius: {text}")),
        },
        Input::Enter => match st.mode.as_str() {
            "fit" => {
                let pts = fit_points(e);
                match fit_circle(&pts) {
                    Some((c, nrm, r)) => {
                        let x = unit(if nrm[0].abs() < 0.9 {
                            cross(nrm, [1.0, 0.0, 0.0])
                        } else {
                            cross(nrm, [0.0, 1.0, 0.0])
                        });
                        match st.shape(c, x, cross(nrm, x), r) {
                            Some(g) => done(g, r),
                            None => st
                                .reply(e, None)
                                .refuse("The points do not define a circle"),
                        }
                    }
                    None => st
                        .reply(e, None)
                        .refuse("Select at least three points that are not in a line"),
                }
            }
            "tangent" if st.curves.len() == 2 && st.radius.is_none() => st
                .reply(e, None)
                .refuse("Type a radius or select a third tangent curve"),
            _ if st.orienting => {
                st.orienting = false;
                st.reply(e, None)
            }
            _ => Reply::finish(vec![]),
        },
        Input::Undo => {
            st.points.pop();
            st.reply(e, None)
        }
    }
}

/// Re-read the selection after switching to a mode that picks curves.
fn step_selection(st: State, e: &Event) -> Reply {
    let ev = Event {
        input: Input::Selection,
        state: save(&st),
        ..e.clone()
    };
    step(&ev)
}

/// A typed size at the radius step.
fn sized(st: State, e: &Event, v: f64) -> Reply {
    if v <= 0.0 {
        return st.reply(e, None).refuse("The radius must not be 0");
    }
    let r = st.radius_of(v);
    let c = st.points[0];
    let (x, y) = st.frame(e, c, None);
    match st.shape(c, x, y, r) {
        Some(g) => done(g, r),
        None => st.reply(e, None).refuse("The radius must not be 0"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_circle_recovers_a_tilted_circle() {
        let (c, x, y, r) = ([1.0, 2.0, 3.0], unit([1.0, 0.0, 1.0]), [0.0, 1.0, 0.0], 4.0);
        let pts: Vec<Vec3> = (0..7)
            .map(|i| {
                let a = i as f64 * 0.7;
                add(c, add(scale(x, r * a.cos()), scale(y, r * a.sin())))
            })
            .collect();
        let (cc, n, rr) = fit_circle(&pts).unwrap();
        assert!(dist(cc, c) < 1e-9, "{cc:?}");
        assert!((rr - r).abs() < 1e-9);
        assert!(dot(n, cross(x, y)).abs() > 1.0 - 1e-9);
    }
}
