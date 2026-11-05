// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve Boolean as Matrix (Rhino) runs it: `Select curves to perform boolean operation`,
//! Enter, then `Click inside the regions to keep. Press Enter when done ( DeleteInput=None
//! CombineRegions=Yes AllRegions )`. A second click in a region drops it again.
//!
//! The curves are cut where they cross into a planar map (in the CPlane); its faces are the
//! regions. The result follows the exact input curves: each boundary run is the matching
//! piece of the curve it came from, joined into closed curves.
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Plane, Reply, Vec3, Want};
use om9_addon_geom::cmd::{load, save};
use om9_addon_geom::join;
use om9_addon_geom::pick::CurveRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type P2 = [f64; 2];

/// One directed piece of the map: from vertex `a` to `b`, along curve `curve` from `ta` to
/// `tb` (closed curves: `tb` may pass 1).
#[derive(Clone, Debug)]
struct Half {
    a: usize,
    b: usize,
    curve: usize,
    ta: f64,
    tb: f64,
}

/// The planar map of the curves and its regions.
pub(crate) struct Map {
    verts: Vec<P2>,
    halves: Vec<Half>,
    next: Vec<usize>,
    /// Region of each half-edge (the face on its left), if it bounds one.
    region: Vec<Option<usize>>,
    /// Regions: outer cycle and hole cycles (as half-edge cycles) with their area.
    regions: Vec<(Vec<usize>, Vec<Vec<usize>>, f64)>,
}

fn cross2(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn norm2(a: P2) -> f64 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}
fn sub2(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}

fn area(poly: &[P2]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| cross2(poly[i], poly[(i + 1) % n]))
        .sum::<f64>()
        / 2.0
}

fn inside(poly: &[P2], p: P2) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < a[0] + (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1])
        {
            c = !c;
        }
    }
    c
}

impl Map {
    pub(crate) fn build(curves: &[CurveRef], plane: &Plane) -> Map {
        let n_samples = 400;
        let flat = |p: Vec3| {
            let l = plane.local(p);
            [l[0], l[1]]
        };
        // Segments with their curve parameters.
        struct Seg {
            a: P2,
            b: P2,
            curve: usize,
            ta: f64,
            tb: f64,
            cuts: Vec<f64>,
        }
        let mut segs: Vec<Seg> = Vec::new();
        let mut size: f64 = 1.0;
        for (ci, c) in curves.iter().enumerate() {
            let pts: Vec<P2> = (0..=n_samples)
                .map(|i| flat(c.at(i as f64 / n_samples as f64)))
                .collect();
            for p in &pts {
                size = size.max(p[0].abs()).max(p[1].abs());
            }
            for i in 0..n_samples {
                segs.push(Seg {
                    a: pts[i],
                    b: pts[i + 1],
                    curve: ci,
                    ta: i as f64 / n_samples as f64,
                    tb: (i + 1) as f64 / n_samples as f64,
                    cuts: vec![0.0, 1.0],
                });
            }
        }
        let eps = 1e-9 * size;
        // Crossings.
        let bbox = |s: &Seg| {
            [
                s.a[0].min(s.b[0]),
                s.a[1].min(s.b[1]),
                s.a[0].max(s.b[0]),
                s.a[1].max(s.b[1]),
            ]
        };
        let boxes: Vec<[f64; 4]> = segs.iter().map(bbox).collect();
        for i in 0..segs.len() {
            for j in i + 1..segs.len() {
                let (bi, bj) = (boxes[i], boxes[j]);
                if bi[2] < bj[0] - eps
                    || bj[2] < bi[0] - eps
                    || bi[3] < bj[1] - eps
                    || bj[3] < bi[1] - eps
                {
                    continue;
                }
                let (p, r) = (segs[i].a, sub2(segs[i].b, segs[i].a));
                let (q, s) = (segs[j].a, sub2(segs[j].b, segs[j].a));
                let den = cross2(r, s);
                if den.abs() < 1e-12 * norm2(r) * norm2(s) {
                    // Parallel: on one line, each cuts the other at its ends (overlaps).
                    let rr = r[0] * r[0] + r[1] * r[1];
                    let ss = s[0] * s[0] + s[1] * s[1];
                    if rr < 1e-30 || ss < 1e-30 || cross2(sub2(q, p), r).abs() > eps * rr.sqrt() {
                        continue;
                    }
                    for end in [q, segs[j].b] {
                        let t = ((end[0] - p[0]) * r[0] + (end[1] - p[1]) * r[1]) / rr;
                        if t > 1e-9 && t < 1.0 - 1e-9 {
                            segs[i].cuts.push(t);
                        }
                    }
                    for end in [p, segs[i].b] {
                        let u = ((end[0] - q[0]) * s[0] + (end[1] - q[1]) * s[1]) / ss;
                        if u > 1e-9 && u < 1.0 - 1e-9 {
                            segs[j].cuts.push(u);
                        }
                    }
                    continue;
                }
                let qp = sub2(q, p);
                let t = cross2(qp, s) / den;
                let u = cross2(qp, r) / den;
                let tol = 1e-9;
                if (-tol..=1.0 + tol).contains(&t) && (-tol..=1.0 + tol).contains(&u) {
                    // Neighbours along one curve meet at their shared ends: not a crossing.
                    if segs[i].curve == segs[j].curve
                        && ((t > 1.0 - 1e-6 && u < 1e-6) || (t < 1e-6 && u > 1.0 - 1e-6))
                    {
                        continue;
                    }
                    segs[i].cuts.push(t.clamp(0.0, 1.0));
                    segs[j].cuts.push(u.clamp(0.0, 1.0));
                }
            }
        }
        // Vertices (merged), and the edges between them.
        let mut verts: Vec<P2> = Vec::new();
        let mut index: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        let cell = (eps * 1e3).max(1e-9);
        let mut vertex = |p: P2| -> usize {
            let key = ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(list) = index.get(&(key.0 + dx, key.1 + dy)) {
                        for &v in list {
                            let d = sub2(verts[v], p);
                            if d[0].abs() <= cell && d[1].abs() <= cell {
                                return v;
                            }
                        }
                    }
                }
            }
            verts.push(p);
            index.entry(key).or_default().push(verts.len() - 1);
            verts.len() - 1
        };
        let mut edges: Vec<(usize, usize, usize, f64, f64)> = Vec::new();
        for s in &mut segs {
            s.cuts.sort_by(f64::total_cmp);
            s.cuts.dedup_by(|x, y| (*x - *y).abs() < 1e-12);
            for w in s.cuts.windows(2) {
                let at = |k: f64| {
                    [
                        s.a[0] + (s.b[0] - s.a[0]) * k,
                        s.a[1] + (s.b[1] - s.a[1]) * k,
                    ]
                };
                let (va, vb) = (vertex(at(w[0])), vertex(at(w[1])));
                if va != vb {
                    let par = |k: f64| s.ta + (s.tb - s.ta) * k;
                    edges.push((va, vb, s.curve, par(w[0]), par(w[1])));
                }
            }
        }
        // Overlapping curves give the same edge twice: keep one.
        let mut seen = std::collections::HashSet::new();
        edges.retain(|e| seen.insert((e.0.min(e.1), e.0.max(e.1))));
        // Drop dangling tails (open curve ends) repeatedly.
        loop {
            let mut degree = vec![0usize; verts.len()];
            for e in &edges {
                degree[e.0] += 1;
                degree[e.1] += 1;
            }
            let before = edges.len();
            edges.retain(|e| degree[e.0] > 1 && degree[e.1] > 1);
            if edges.len() == before {
                break;
            }
        }
        let mut halves = Vec::new();
        for &(a, b, curve, ta, tb) in &edges {
            halves.push(Half {
                a,
                b,
                curve,
                ta,
                tb,
            });
            halves.push(Half {
                a: b,
                b: a,
                curve,
                ta: tb,
                tb: ta,
            });
        }
        // Around each vertex, outgoing half-edges by angle.
        let mut out: Vec<Vec<usize>> = vec![Vec::new(); verts.len()];
        for (h, x) in halves.iter().enumerate() {
            out[x.a].push(h);
        }
        let angle = |h: usize| {
            let d = sub2(verts[halves[h].b], verts[halves[h].a]);
            d[1].atan2(d[0])
        };
        for list in out.iter_mut() {
            list.sort_by(|&x, &y| angle(x).total_cmp(&angle(y)));
        }
        // next(h): at the end vertex, the outgoing edge just clockwise of the way back.
        let next: Vec<usize> = (0..halves.len())
            .map(|h| {
                let twin = h ^ 1;
                let list = &out[halves[h].b];
                let i = list.iter().position(|&x| x == twin).unwrap_or(0);
                list[(i + list.len() - 1) % list.len()]
            })
            .collect();
        // Cycles.
        let mut cycle_of = vec![usize::MAX; halves.len()];
        let mut cycles: Vec<(Vec<usize>, f64)> = Vec::new();
        for h0 in 0..halves.len() {
            if cycle_of[h0] != usize::MAX {
                continue;
            }
            let mut cyc = Vec::new();
            let mut h = h0;
            while cycle_of[h] == usize::MAX && cyc.len() <= halves.len() {
                cycle_of[h] = cycles.len();
                cyc.push(h);
                h = next[h];
            }
            let poly: Vec<P2> = cyc.iter().map(|&h| verts[halves[h].a]).collect();
            cycles.push((cyc, area(&poly)));
        }
        let poly_of =
            |cyc: &[usize]| -> Vec<P2> { cyc.iter().map(|&h| verts[halves[h].a]).collect() };
        // Regions: counter-clockwise cycles; clockwise ones are outlines of islands, which are
        // holes of the smallest region around them.
        let mut regions: Vec<(Vec<usize>, Vec<Vec<usize>>, f64)> = Vec::new();
        let mut region_of_cycle = vec![None; cycles.len()];
        for (k, (cyc, a)) in cycles.iter().enumerate() {
            if *a > eps * eps {
                region_of_cycle[k] = Some(regions.len());
                regions.push((cyc.clone(), Vec::new(), *a));
            }
        }
        for (k, (cyc, a)) in cycles.iter().enumerate() {
            if *a >= -eps * eps {
                continue;
            }
            let probe = verts[halves[cyc[0]].a];
            let around = regions
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    let poly = poly_of(&r.0);
                    // Not the island's own inside, and around the island.
                    !r.0.iter().any(|&h| cyc.contains(&(h ^ 1))) && inside(&poly, probe)
                })
                .min_by(|x, y| x.1.2.total_cmp(&y.1.2))
                .map(|(i, _)| i);
            if let Some(r) = around {
                regions[r].1.push(cyc.clone());
                region_of_cycle[k] = Some(r);
            }
        }
        let region = (0..halves.len())
            .map(|h| region_of_cycle[cycle_of[h]])
            .collect();
        Map {
            verts,
            halves,
            next,
            region,
            regions,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.regions.len()
    }

    /// The region a point (in the plane) falls in: the smallest one around it.
    pub(crate) fn at(&self, p: P2) -> Option<usize> {
        let poly = |cyc: &[usize]| -> Vec<P2> {
            cyc.iter().map(|&h| self.verts[self.halves[h].a]).collect()
        };
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                inside(&poly(&r.0), p) && !r.1.iter().any(|hole| inside(&poly(hole), p))
            })
            .min_by(|a, b| a.1.2.total_cmp(&b.1.2))
            .map(|(i, _)| i)
    }

    /// Boundary loops of the chosen regions (combined: shared edges removed).
    fn loops(&self, chosen: &[usize], combine: bool) -> Vec<Vec<usize>> {
        if !combine {
            let mut out = Vec::new();
            for &r in chosen {
                out.push(self.regions[r].0.clone());
                out.extend(self.regions[r].1.iter().cloned());
            }
            return out;
        }
        let picked = |h: usize| self.region[h].is_some_and(|r| chosen.contains(&r));
        let boundary = |h: usize| picked(h) && !picked(h ^ 1);
        let mut used = vec![false; self.halves.len()];
        let mut out = Vec::new();
        for h0 in 0..self.halves.len() {
            if used[h0] || !boundary(h0) {
                continue;
            }
            let mut cyc = Vec::new();
            let mut h = h0;
            while !used[h] {
                used[h] = true;
                cyc.push(h);
                // Turn around the end vertex through chosen faces to the next boundary edge.
                let mut g = self.next[h];
                let mut guard = 0;
                while !boundary(g) && guard < self.halves.len() {
                    g = self.next[g ^ 1];
                    guard += 1;
                }
                h = g;
            }
            out.push(cyc);
        }
        out
    }

    /// A loop as exact curve pieces, joined.
    fn curve_of(&self, cyc: &[usize], curves: &[CurveRef], plane: &Plane) -> Option<Geo> {
        // Runs along one curve with continuous parameters.
        let mut runs: Vec<(usize, f64, f64)> = Vec::new();
        for &h in cyc {
            let x = &self.halves[h];
            let closed = curves[x.curve].closed;
            if let Some(last) = runs.last_mut()
                && last.0 == x.curve
            {
                let joins = (last.2 - x.ta).abs() < 1e-9
                    || (closed && ((last.2 - x.ta).abs() - 1.0).abs() < 1e-9);
                if joins {
                    // Keep the parameter unwrapped across the seam of closed curves.
                    let step = x.tb - x.ta;
                    last.2 += step;
                    continue;
                }
            }
            runs.push((x.curve, x.ta, x.tb));
        }
        // A run that wraps onto the first one (same curve) belongs to it.
        if runs.len() > 1 {
            let (first, last) = (runs[0], *runs.last()?);
            if first.0 == last.0 {
                let gap = (last.2 - first.1).rem_euclid(1.0);
                if !(1e-9..=1.0 - 1e-9).contains(&gap) {
                    runs[0] = (first.0, last.1, last.2 + (first.2 - first.1));
                    runs.pop();
                }
            }
        }
        let mut pieces = Vec::new();
        for (ci, a, b) in runs {
            let c = &curves[ci];
            if (b - a).abs() >= 1.0 - 1e-9 && c.closed {
                let g = c.geo();
                pieces.push(if b > a { g } else { join::reverse(&g)? });
                continue;
            }
            let forward = |lo: f64, hi: f64| -> Option<Geo> {
                let (lo, hi) = (lo.rem_euclid(1.0), hi.rem_euclid(1.0));
                let hi = if hi < 1e-12 && c.closed { 1.0 } else { hi };
                c.sub(lo, hi)
            };
            let piece = if b > a {
                forward(a, b)
            } else {
                forward(b, a).and_then(|g| join::reverse(&g))
            };
            pieces.extend(piece);
        }
        let tol = 1e-6
            * self
                .verts
                .iter()
                .fold(1.0f64, |m, v| m.max(v[0].abs()).max(v[1].abs()));
        join::join(&pieces, tol.max(1e-6)).or_else(|| {
            let z = curves
                .first()
                .map(|c| plane.local(c.at(0.0))[2])
                .unwrap_or(0.0);
            let points = cyc.iter().map(|&h| {
                let v = self.verts[self.halves[h].a];
                plane.at(v[0], v[1], z)
            });
            Some(Geo::Polyline {
                points: points.collect(),
                closed: true,
            })
        })
    }
}

#[derive(Default, Serialize, Deserialize)]
struct State {
    selected: bool,
    picks: Vec<Vec3>,
    #[serde(default)]
    all: bool,
    #[serde(default)]
    delete: String,
    #[serde(default = "yes")]
    combine: bool,
}

fn yes() -> bool {
    true
}

const DELETE: [&str; 3] = ["None", "All", "Used"];

pub(crate) fn curve_boolean(e: &Event) -> Reply {
    let mut st: State = load(e);
    if st.delete.is_empty() {
        st.delete = "None".into();
        st.combine = true;
    }
    let curves = CurveRef::all(e);
    let options = |st: &State| {
        let mut o = vec![
            OptionToken::choice("DeleteInput", &st.delete, &DELETE),
            OptionToken::toggle("CombineRegions", st.combine),
            OptionToken::action("AllRegions"),
        ];
        if !st.picks.is_empty() {
            o.push(OptionToken::action("Undo"));
        }
        o
    };
    if !st.selected {
        if matches!(e.input, Input::Enter | Input::Start) && curves.len() >= 2 {
            st.selected = true;
        } else {
            let r = Reply::ask(
                "Select curves to perform boolean operation. Press Enter when done",
                save(&st),
            )
            .want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse("Select two or more curves")
            } else {
                r
            };
        }
    }
    let map = Map::build(&curves, &e.plane);
    let flat = |p: Vec3| {
        let l = e.plane.local(p);
        [l[0], l[1]]
    };
    let chosen = |st: &State| -> Vec<usize> {
        if st.all {
            return (0..map.len()).collect();
        }
        let mut on: Vec<usize> = Vec::new();
        for p in &st.picks {
            if let Some(r) = map.at(flat(*p)) {
                match on.iter().position(|x| *x == r) {
                    Some(i) => {
                        on.remove(i);
                    }
                    None => on.push(r),
                }
            }
        }
        on
    };
    let result = |st: &State| -> Vec<Geo> {
        map.loops(&chosen(st), st.combine)
            .iter()
            .filter_map(|cyc| map.curve_of(cyc, &curves, &e.plane))
            .collect()
    };
    let mut error = None;
    match &e.input {
        Input::Point { point } => {
            if map.at(flat(*point)).is_some() {
                st.picks.push(*point);
                st.all = false;
            } else {
                error = Some("Click inside a closed region".to_owned());
            }
        }
        Input::Undo => {
            st.picks.pop();
        }
        Input::Option { name, value } => match name.to_ascii_lowercase().as_str() {
            "undo" => {
                st.picks.pop();
            }
            "allregions" => st.all = true,
            "combineregions" => {
                st.combine = match value.as_deref() {
                    Some(v) => v.eq_ignore_ascii_case("yes"),
                    None => !st.combine,
                }
            }
            "deleteinput" => {
                st.delete = match value {
                    Some(v) => DELETE
                        .iter()
                        .find(|d| d.eq_ignore_ascii_case(v))
                        .unwrap_or(&"None")
                        .to_string(),
                    None => DELETE
                        [(DELETE.iter().position(|d| *d == st.delete).unwrap_or(0) + 1) % 3]
                        .to_string(),
                }
            }
            _ => error = Some(format!("Unknown option {name}")),
        },
        Input::Enter => {
            let geo = result(&st);
            if geo.is_empty() {
                let r = Reply::ask(
                    "Click inside the regions to keep. Press Enter when done",
                    save(&st),
                )
                .with_options(options(&st))
                .want(Want::Point);
                return r.refuse("No region is chosen");
            }
            let n = geo.len();
            let mut r = Reply::finish(geo).say(format!("Curve Boolean: {n} curves"));
            match st.delete.as_str() {
                "All" => r.delete = curves.iter().map(|c| c.id.clone()).collect(),
                "Used" => {
                    let picked = chosen(&st);
                    let mut used: Vec<String> = Vec::new();
                    for cyc in map.loops(&picked, st.combine) {
                        for h in cyc {
                            let id = &curves[map.halves[h].curve].id;
                            if !used.contains(id) {
                                used.push(id.clone());
                            }
                        }
                    }
                    r.delete = used;
                }
                _ => {}
            }
            return r;
        }
        _ => {}
    }
    let mut r = Reply::ask(
        "Click inside the regions to keep. Press Enter when done",
        save(&st),
    )
    .with_options(options(&st))
    .want(Want::Point)
    .preview(result(&st));
    r.rubber_from = None;
    if map.len() == 0 {
        error.get_or_insert_with(|| "The curves enclose no region".to_owned());
    }
    match error {
        Some(w) => r.refuse(w),
        None => r,
    }
}
