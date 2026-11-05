// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem shapes, gem meshes, gem records and carat weights shared by the gem, setting and cutter
//! commands.
//!
//! Local gem frame: origin at the girdle center, +z through the table, +x along the stone's
//! length. Outlines are counter-clockwise seen from the table.
use crate::mesh::{self, Frame, Mesh};
use om9_addon_abi::session::{Selected, Vec3, add, scale};
use serde_json::{Value, json};
use std::f64::consts::{PI, TAU};

/// The gem cuts the add-ons can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    Round,
    Oval,
    Princess,
    Cushion,
    Emerald,
    Asscher,
    Baguette,
    Marquise,
    Pear,
    Heart,
    Trillion,
    Radiant,
    CabochonRound,
    CabochonOval,
    /// Outline from a curve (Custom Gem builder).
    Custom,
}

impl Cut {
    pub const ALL: [Cut; 14] = [
        Cut::Round,
        Cut::Oval,
        Cut::Princess,
        Cut::Cushion,
        Cut::Emerald,
        Cut::Asscher,
        Cut::Baguette,
        Cut::Marquise,
        Cut::Pear,
        Cut::Heart,
        Cut::Trillion,
        Cut::Radiant,
        Cut::CabochonRound,
        Cut::CabochonOval,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Cut::Round => "round",
            Cut::Oval => "oval",
            Cut::Princess => "princess",
            Cut::Cushion => "cushion",
            Cut::Emerald => "emerald",
            Cut::Asscher => "asscher",
            Cut::Baguette => "baguette",
            Cut::Marquise => "marquise",
            Cut::Pear => "pear",
            Cut::Heart => "heart",
            Cut::Trillion => "trillion",
            Cut::Radiant => "radiant",
            Cut::CabochonRound => "cabochon_round",
            Cut::CabochonOval => "cabochon_oval",
            Cut::Custom => "custom",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Cut::Round => "Round",
            Cut::Oval => "Oval",
            Cut::Princess => "Princess",
            Cut::Cushion => "Cushion",
            Cut::Emerald => "Emerald",
            Cut::Asscher => "Asscher",
            Cut::Baguette => "Baguette",
            Cut::Marquise => "Marquise",
            Cut::Pear => "Pear",
            Cut::Heart => "Heart",
            Cut::Trillion => "Trillion",
            Cut::Radiant => "Radiant",
            Cut::CabochonRound => "Cabochon round",
            Cut::CabochonOval => "Cabochon oval",
            Cut::Custom => "Custom",
        }
    }
    pub fn from_key(k: &str) -> Option<Cut> {
        let k = k.trim().to_ascii_lowercase().replace([' ', '-'], "_");
        Cut::ALL
            .into_iter()
            .chain([Cut::Custom])
            .find(|c| c.key() == k)
    }
    /// Width / length of the usual proportion (1 for symmetric cuts).
    pub fn default_ratio(self) -> f64 {
        match self {
            Cut::Oval | Cut::CabochonOval => 0.7,
            Cut::Emerald | Cut::Radiant => 0.72,
            Cut::Baguette => 0.4,
            Cut::Marquise => 0.5,
            Cut::Pear => 0.64,
            Cut::Heart => 1.0,
            _ => 1.0,
        }
    }
    /// Step cuts get a deeper pavilion and a larger table.
    fn step_cut(self) -> bool {
        matches!(
            self,
            Cut::Emerald | Cut::Asscher | Cut::Baguette | Cut::Princess | Cut::Radiant
        )
    }
    pub fn cabochon(self) -> bool {
        matches!(self, Cut::CabochonRound | Cut::CabochonOval)
    }
    /// Jeweller's weight factor for a diamond: carat ≈ L × W × depth × factor (mm).
    fn carat_factor(self) -> f64 {
        match self {
            Cut::Round => 0.0061,
            Cut::Oval => 0.0062,
            Cut::Princess => 0.0083,
            Cut::Cushion => 0.0081,
            Cut::Emerald | Cut::Asscher => 0.0080,
            Cut::Radiant => 0.0081,
            Cut::Baguette => 0.00915,
            Cut::Marquise => 0.00565,
            Cut::Pear => 0.00615,
            Cut::Heart => 0.0059,
            Cut::Trillion => 0.0057,
            Cut::CabochonRound | Cut::CabochonOval => 0.0070,
            Cut::Custom => 0.0065,
        }
    }
}

/// Specific gravity of common gem materials (diamond = 3.52).
pub const MATERIALS: &[(&str, f64)] = &[
    ("diamond", 3.52),
    ("moissanite", 3.21),
    ("cz", 5.8),
    ("ruby", 4.0),
    ("sapphire", 4.0),
    ("emerald", 2.72),
    ("amethyst", 2.65),
    ("topaz", 3.53),
    ("garnet", 3.9),
    ("aquamarine", 2.7),
    ("peridot", 3.34),
    ("tanzanite", 3.35),
];

pub fn specific_gravity(material: &str) -> f64 {
    MATERIALS
        .iter()
        .find(|(m, _)| m.eq_ignore_ascii_case(material.trim()))
        .map_or(3.52, |m| m.1)
}

/// The proportions of a stone, in fractions of its width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Proportions {
    pub table: f64,
    pub crown: f64,
    pub girdle: f64,
    pub pavilion: f64,
}

impl Proportions {
    pub fn of(cut: Cut) -> Proportions {
        if cut.step_cut() {
            Proportions {
                table: 0.65,
                crown: 0.12,
                girdle: 0.03,
                pavilion: 0.52,
            }
        } else {
            Proportions {
                table: 0.56,
                crown: 0.15,
                girdle: 0.03,
                pavilion: 0.43,
            }
        }
    }
    pub fn depth(&self) -> f64 {
        self.crown + self.girdle + self.pavilion
    }
}

/// Girdle outline of a cut (length along x, width along y), `n` points, counter-clockwise.
pub fn outline(cut: Cut, length: f64, width: f64, n: usize) -> Vec<[f64; 2]> {
    let n = n.max(12);
    let (a, b) = (length / 2.0, width / 2.0);
    let ts = (0..n).map(|i| TAU * i as f64 / n as f64);
    match cut {
        Cut::Round | Cut::CabochonRound | Cut::Custom => {
            ts.map(|t| [a * t.cos(), a * t.sin()]).collect()
        }
        Cut::Oval | Cut::CabochonOval => ts.map(|t| [a * t.cos(), b * t.sin()]).collect(),
        Cut::Cushion => superellipse(a, b, 4.0, n),
        Cut::Princess => superellipse(a, b, 40.0, n),
        Cut::Emerald | Cut::Asscher | Cut::Radiant => {
            let c = 0.18 * width.min(length);
            octagon(a, b, c, n)
        }
        Cut::Baguette => octagon(a, b, 0.0001 * width, n),
        Cut::Marquise => {
            // Two circular arcs meeting in points at x = ±a.
            let r = (a * a + b * b) / (2.0 * b);
            ts.map(|t| {
                let s = t.sin();
                let x = a * t.cos();
                let y = (r * r - x * x).sqrt() - (r - b);
                [x, y.max(0.0) * s.signum()]
            })
            .collect()
        }
        Cut::Pear => {
            // Tear drop: point at +x, round end at −x.
            let pts: Vec<[f64; 2]> = ts
                .map(|t| {
                    let x = -t.cos();
                    let y = t.sin() * ((1.0 + t.cos()) / 2.0).sqrt();
                    [x, y]
                })
                .collect();
            ccw(normalise(&pts, a, b))
        }
        Cut::Heart => {
            let pts: Vec<[f64; 2]> = ts
                .map(|t| {
                    let x = 16.0 * t.sin().powi(3);
                    let y = 13.0 * t.cos()
                        - 5.0 * (2.0 * t).cos()
                        - 2.0 * (3.0 * t).cos()
                        - (4.0 * t).cos();
                    // point along +x: rotate (x, y) → (−y, x)
                    [-y, x]
                })
                .collect();
            let pts = normalise(&pts, a, b);
            ccw(pts)
        }
        Cut::Trillion => {
            // Triangle with slightly convex sides, one corner at +x.
            let corners: Vec<[f64; 2]> = (0..3)
                .map(|k| {
                    let t = TAU * k as f64 / 3.0;
                    [t.cos(), t.sin()]
                })
                .collect();
            let per_side = n / 3 + 1;
            let mut pts = Vec::new();
            for k in 0..3 {
                let (p, q) = (corners[k], corners[(k + 1) % 3]);
                let mid = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0];
                let out = [mid[0], mid[1]];
                let l = (out[0] * out[0] + out[1] * out[1]).sqrt();
                for j in 0..per_side {
                    let u = j as f64 / per_side as f64;
                    let bulge = 0.12 * (PI * u).sin();
                    pts.push([
                        p[0] + (q[0] - p[0]) * u + out[0] / l * bulge,
                        p[1] + (q[1] - p[1]) * u + out[1] / l * bulge,
                    ]);
                }
            }
            normalise(&resample(&pts, n), a, b)
        }
    }
}

fn superellipse(a: f64, b: f64, p: f64, n: usize) -> Vec<[f64; 2]> {
    (0..n)
        .map(|i| {
            let t = TAU * i as f64 / n as f64;
            let (c, s) = (t.cos(), t.sin());
            [
                a * c.signum() * c.abs().powf(2.0 / p),
                b * s.signum() * s.abs().powf(2.0 / p),
            ]
        })
        .collect()
}

/// Rectangle with cut corners `c`, resampled to `n` points.
fn octagon(a: f64, b: f64, c: f64, n: usize) -> Vec<[f64; 2]> {
    let c = c.min(a).min(b);
    let corners = [
        [a, -b + c],
        [a, b - c],
        [a - c, b],
        [-a + c, b],
        [-a, b - c],
        [-a, -b + c],
        [-a + c, -b],
        [a - c, -b],
    ];
    resample(&corners, n)
}

/// Resample a closed polygon to `n` points evenly by arc length.
pub fn resample(poly: &[[f64; 2]], n: usize) -> Vec<[f64; 2]> {
    let m = poly.len();
    let seg: Vec<f64> = (0..m)
        .map(|i| {
            let (p, q) = (poly[i], poly[(i + 1) % m]);
            ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt()
        })
        .collect();
    let total: f64 = seg.iter().sum();
    let mut out = Vec::with_capacity(n);
    let (mut i, mut acc) = (0, 0.0);
    for k in 0..n {
        let target = total * k as f64 / n as f64;
        while i < m - 1 && acc + seg[i] < target {
            acc += seg[i];
            i += 1;
        }
        let t = if seg[i] > 0.0 {
            (target - acc) / seg[i]
        } else {
            0.0
        };
        let (p, q) = (poly[i], poly[(i + 1) % m]);
        out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
    }
    out
}

/// Scale points into the box [−a, a] × [−b, b] (centered on the box center).
fn normalise(pts: &[[f64; 2]], a: f64, b: f64) -> Vec<[f64; 2]> {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in pts {
        for i in 0..2 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let s = [2.0 * a / (hi[0] - lo[0]), 2.0 * b / (hi[1] - lo[1])];
    pts.iter()
        .map(|p| [(p[0] - c[0]) * s[0], (p[1] - c[1]) * s[1]])
        .collect()
}

fn ccw(mut pts: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    let n = pts.len();
    let area: f64 = (0..n)
        .map(|i| {
            let (p, q) = (pts[i], pts[(i + 1) % n]);
            p[0] * q[1] - q[0] * p[1]
        })
        .sum();
    if area < 0.0 {
        pts.reverse();
    }
    pts
}

/// A gem: cut, length (x) and width (y) in mm.
#[derive(Debug, Clone, PartialEq)]
pub struct Gem {
    pub cut: Cut,
    pub length: f64,
    pub width: f64,
    /// The girdle outline of a custom stone (local, centered).
    pub custom: Option<Vec<[f64; 2]>>,
}

impl Gem {
    pub fn new(cut: Cut, length: f64, width: f64) -> Gem {
        let width = if matches!(cut, Cut::Round | Cut::CabochonRound) {
            length
        } else {
            width
        };
        Gem {
            cut,
            length,
            width,
            custom: None,
        }
    }
    /// A custom stone from a closed outline (local xy, any orientation): it is centered on
    /// its bounding box and resampled.
    pub fn from_outline(points: &[[f64; 2]]) -> Option<Gem> {
        if points.len() < 3 {
            return None;
        }
        let pts = ccw(resample(points, 96));
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in &pts {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let (length, width) = (hi[0] - lo[0], hi[1] - lo[1]);
        if !(length > 1e-6 && width > 1e-6) {
            return None;
        }
        Some(Gem {
            cut: Cut::Custom,
            length,
            width,
            custom: Some(pts.iter().map(|p| [p[0] - c[0], p[1] - c[1]]).collect()),
        })
    }
    pub fn proportions(&self) -> Proportions {
        Proportions::of(self.cut)
    }
    /// The smaller girdle dimension, that the proportions are measured on.
    pub fn base(&self) -> f64 {
        self.width.min(self.length)
    }
    pub fn depth(&self) -> f64 {
        if self.cut.cabochon() {
            0.45 * self.base()
        } else {
            self.proportions().depth() * self.base()
        }
    }
    pub fn outline(&self, n: usize) -> Vec<[f64; 2]> {
        match &self.custom {
            Some(o) => resample(o, n.max(12)),
            None => outline(self.cut, self.length, self.width, n),
        }
    }
    /// Girdle top and bottom heights (girdle center at z = 0).
    pub fn girdle(&self) -> (f64, f64) {
        let g = self.proportions().girdle * self.base() / 2.0;
        (g, -g)
    }
    /// Table (top) height and culet (bottom) height.
    pub fn top_bottom(&self) -> (f64, f64) {
        let p = self.proportions();
        let (gt, gb) = self.girdle();
        if self.cut.cabochon() {
            (0.45 * self.base(), 0.0)
        } else {
            (gt + p.crown * self.base(), gb - p.pavilion * self.base())
        }
    }
    /// Closed faceted mesh in the local gem frame.
    pub fn mesh(&self) -> Mesh {
        let n = if matches!(
            self.cut,
            Cut::Round | Cut::Oval | Cut::CabochonRound | Cut::CabochonOval
        ) {
            32
        } else {
            48
        };
        let o = self.outline(n);
        let p = self.proportions();
        let (gt, gb) = self.girdle();
        let (top, bottom) = self.top_bottom();
        if self.cut.cabochon() {
            let mut rings = Vec::new();
            for k in 0..6 {
                let a = std::f64::consts::FRAC_PI_2 * k as f64 / 6.0;
                rings.push(mesh::ring(&mesh::scaled(&o, a.cos(), a.cos()), top * a.sin()));
            }
            rings.reverse();
            return mesh::ring_stack(&rings, Some([0.0, 0.0, top]), Some([0.0, 0.0, 0.0]));
        }
        let table = mesh::scaled(&o, p.table, p.table);
        let rings = vec![
            mesh::ring(&table, top),
            mesh::ring(&o, gt),
            mesh::ring(&o, gb),
        ];
        mesh::ring_stack(&rings, Some([0.0, 0.0, top]), Some([0.0, 0.0, bottom]))
    }
    /// Diamond-equivalent carat weight scaled by the material's specific gravity.
    pub fn carat(&self, material: &str) -> f64 {
        let l = self.length.max(self.width);
        let w = self.length.min(self.width);
        let d = self.depth();
        l * w * d * self.cut.carat_factor() * specific_gravity(material) / 3.52
    }
    /// `1.50 mm` for rounds, `7.00 × 5.00 mm` otherwise.
    pub fn size_label(&self) -> String {
        if (self.length - self.width).abs() < 1e-9 {
            format!("{:.2} mm", self.length)
        } else {
            format!("{:.2} × {:.2} mm", self.length, self.width)
        }
    }
    /// The `om9.gem` record for a stone placed with `frame`.
    pub fn record(&self, frame: &Frame, source: &str, material: &str) -> Value {
        let mut r = self.record_fields(frame, source, material);
        if let Some(o) = &self.custom {
            r["outline"] = json!(resample(o, 64));
        }
        r
    }
    fn record_fields(&self, frame: &Frame, source: &str, material: &str) -> Value {
        json!({
            "cut": self.cut.key(),
            "diameter_mm": self.base(),
            "length_mm": self.length,
            "width_mm": self.width,
            "depth_mm": self.depth(),
            "material": material,
            "source": source,
            "center": frame.origin,
            "axis": frame.z,
            "x": frame.x,
        })
    }
    /// Name shown in the object list: `Gem Round 1.50 mm`.
    pub fn name(&self) -> String {
        format!("Gem {} {}", self.cut.label(), self.size_label())
    }
}

/// A stone read from a selected object's gem record.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedGem {
    pub id: String,
    pub gem: Gem,
    pub frame: Frame,
    pub material: String,
}

impl PlacedGem {
    /// The gem record of a selection entry (world mm). Round stones from older add-ons only
    /// carry a diameter.
    pub fn from_selected(s: &Selected, fallback_axis: Vec3) -> Option<PlacedGem> {
        let g = s.gem.as_ref()?;
        let v3 = |v: &Value| -> Option<Vec3> {
            let a = v.as_array()?;
            Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?])
        };
        let cut = g
            .get("cut")
            .and_then(Value::as_str)
            .and_then(Cut::from_key)
            .unwrap_or(Cut::Round);
        let d = g.get("diameter_mm").and_then(Value::as_f64)?;
        let length = g.get("length_mm").and_then(Value::as_f64).unwrap_or(d);
        let width = g.get("width_mm").and_then(Value::as_f64).unwrap_or(d);
        if !(length > 0.0 && width > 0.0) {
            return None;
        }
        let mut gem = Gem::new(cut, length, width);
        if cut == Cut::Custom {
            let outline: Option<Vec<[f64; 2]>> = g.get("outline").and_then(|o| {
                o.as_array()?
                    .iter()
                    .map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
                    .collect()
            });
            gem.custom = Some(outline.filter(|o| o.len() >= 3)?);
        }
        let center = g
            .get("center")
            .and_then(v3)
            .unwrap_or_else(|| scale(add(s.bbox[0], s.bbox[1]), 0.5));
        let axis = g.get("axis").and_then(v3).unwrap_or(fallback_axis);
        let x = g.get("x").and_then(v3).unwrap_or([1.0, 0.0, 0.0]);
        Some(PlacedGem {
            id: s.id.clone(),
            gem,
            frame: Frame::new(center, axis, x),
            material: g
                .get("material")
                .and_then(Value::as_str)
                .unwrap_or("diamond")
                .to_owned(),
        })
    }
    /// World points of the girdle outline (`n` points), offset outward by `offset` mm and
    /// lifted by `z` along the table axis.
    pub fn girdle_points(&self, n: usize, offset: f64, z: f64) -> Vec<Vec3> {
        let o = self.gem.outline(n);
        let o = if offset.abs() > 0.0 {
            mesh::offset_outline(&o, offset)
        } else {
            o
        };
        o.iter().map(|p| self.frame.point([p[0], p[1], z])).collect()
    }
}

/// Evenly spaced positions for `count` stones of size `d` with `gap` along a closed outline
/// (each: local point, outward direction of the outline there, parameter 0..1).
pub fn spaced_on_outline(outline: &[[f64; 2]], d: f64, gap: f64) -> Vec<([f64; 2], [f64; 2])> {
    let per = mesh::perimeter(outline);
    let count = (per / (d + gap)).floor() as usize;
    if count == 0 {
        return Vec::new();
    }
    let pts = resample(outline, count * 8);
    (0..count)
        .map(|k| {
            let i = k * 8;
            let p = pts[i];
            let (a, b) = (pts[(i + pts.len() - 1) % pts.len()], pts[(i + 1) % pts.len()]);
            let t = [b[0] - a[0], b[1] - a[1]];
            let l = (t[0] * t[0] + t[1] * t[1]).sqrt().max(1e-12);
            (p, [t[1] / l, -t[0] / l])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cut_makes_a_closed_mesh_of_its_size() {
        for cut in Cut::ALL {
            let r = cut.default_ratio();
            let gem = Gem::new(cut, 6.0, 6.0 * r);
            let m = gem.mesh();
            assert!(m.is_closed(), "{cut:?}");
            assert!(m.volume() > 0.0, "{cut:?} {}", m.volume());
            let b = m.bbox();
            assert!(((b[1][0] - b[0][0]) - 6.0).abs() < 0.05, "{cut:?} length {:?}", b);
            assert!(
                ((b[1][1] - b[0][1]) - gem.width).abs() < 0.1,
                "{cut:?} width {:?} vs {}",
                b,
                gem.width
            );
            assert!((b[1][2] - b[0][2] - gem.depth()).abs() < 1e-9, "{cut:?} depth");
            assert_eq!(Cut::from_key(cut.key()), Some(cut));
        }
    }

    #[test]
    fn carat_weights_match_the_trade_formulas() {
        // A 6.5 mm round brilliant (61 % depth) is about one carat.
        let ct = Gem::new(Cut::Round, 6.5, 6.5).carat("diamond");
        assert!((ct - 1.02).abs() < 0.03, "{ct}");
        // Ruby is heavier than diamond for the same size.
        assert!(Gem::new(Cut::Round, 6.5, 6.5).carat("ruby") > ct);
        // A 1.5 mm melee stone is about 0.013 ct.
        let melee = Gem::new(Cut::Round, 1.5, 1.5).carat("diamond");
        assert!((melee - 0.0126).abs() < 0.001, "{melee}");
    }

    #[test]
    fn records_round_trip() {
        let gem = Gem::new(Cut::Oval, 7.0, 5.0);
        let f = Frame::new([1.0, 2.0, 3.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
        let rec = gem.record(&f, "test", "sapphire");
        let s = Selected {
            id: "a".into(),
            kind: "mesh".into(),
            name: gem.name(),
            bbox: [[0.0; 3], [1.0; 3]],
            points: Vec::new(),
            closed: false,
            gem: Some(rec),
            mesh: None,
            exact: None,
        };
        let p = PlacedGem::from_selected(&s, [0.0, 0.0, 1.0]).unwrap();
        assert_eq!(p.gem, gem);
        // A custom stone keeps its outline through its record.
        let star: Vec<[f64; 2]> = (0..10)
            .map(|i| {
                let a = TAU * i as f64 / 10.0;
                let r = if i % 2 == 0 { 3.0 } else { 1.5 };
                [r * a.cos() + 4.0, r * a.sin()]
            })
            .collect();
        let custom = Gem::from_outline(&star).unwrap();
        assert!(custom.mesh().is_closed());
        let mut s2 = s.clone();
        s2.gem = Some(custom.record(&f, "test", "diamond"));
        let back = PlacedGem::from_selected(&s2, [0.0, 0.0, 1.0]).unwrap();
        assert_eq!(back.gem.cut, Cut::Custom);
        assert!((back.gem.length - custom.length).abs() < 1e-9);
        assert_eq!(p.frame.origin, [1.0, 2.0, 3.0]);
        assert!((p.frame.x[1] - 1.0).abs() < 1e-12);
        assert_eq!(p.material, "sapphire");
        let ring = p.girdle_points(16, 0.0, 0.0);
        assert!((ring[0][1] - 2.0 - 3.5).abs() < 1e-9, "x of the stone is world y");
    }

    #[test]
    fn halo_spacing_fills_the_outline() {
        let o = outline(Cut::Round, 10.0, 10.0, 256);
        let spots = spaced_on_outline(&o, 1.0, 0.2);
        assert_eq!(spots.len(), (PI * 10.0 / 1.2).floor() as usize);
        let first = spots[0];
        assert!((first.1[0] - 1.0).abs() < 0.05, "outward normal {:?}", first.1);
    }
}

/// Order stones into a line (nearest neighbour from one end) and tell whether the line closes
/// on itself (a ring of stones). Returns indices into `gems`.
pub fn chain(gems: &[PlacedGem]) -> (Vec<usize>, bool) {
    let n = gems.len();
    if n < 2 {
        return ((0..n).collect(), false);
    }
    let c = |i: usize| gems[i].frame.origin;
    let centroid = scale(
        (0..n).fold([0.0; 3], |a, i| add(a, c(i))),
        1.0 / n as f64,
    );
    // Start at the stone farthest from the centroid (an end of an open line).
    let mut start = 0;
    let mut far = -1.0;
    for i in 0..n {
        let d = om9_addon_abi::session::dist(c(i), centroid);
        if d > far {
            far = d;
            start = i;
        }
    }
    let mut order = vec![start];
    let mut used = vec![false; n];
    used[start] = true;
    while order.len() < n {
        let last = *order.last().unwrap();
        let next = (0..n)
            .filter(|&j| !used[j])
            .min_by(|&a, &b| {
                om9_addon_abi::session::dist(c(last), c(a))
                    .total_cmp(&om9_addon_abi::session::dist(c(last), c(b)))
            })
            .unwrap();
        used[next] = true;
        order.push(next);
    }
    let steps: Vec<f64> = order
        .windows(2)
        .map(|w| om9_addon_abi::session::dist(c(w[0]), c(w[1])))
        .collect();
    let avg = steps.iter().sum::<f64>() / steps.len() as f64;
    let closing = om9_addon_abi::session::dist(c(order[0]), c(order[n - 1]));
    let closed = n >= 3 && closing <= 1.6 * avg;
    (order, closed)
}
