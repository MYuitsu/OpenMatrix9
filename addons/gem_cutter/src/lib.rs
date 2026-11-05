// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix 9 **Gem Cutter** (`gvAzureCutter`, Cutters › Gem Cutter) for OM9.
//!
//! One cutter per selected gem, built around the stone and sized in percent of its diameter
//! (as Matrix does, so one style fits every stone size): an upper cylinder above the girdle,
//! the girdle band, a seat cone that follows the pavilion, and the lower hole down to the
//! bottom with an optional taper. The values come from the builder panel; the same names can
//! be typed on the command line (`GirdleScale=110`).
//!
//! The cutter is a closed triangle mesh on the `Cutting` layer; the gems are not changed.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Reply, Selected, Vec3, Want, add, cross, dot, norm, scale,
    sub, unit,
};
use serde_json::{Value, json};

pub const GEM_CUTTER: &str = "BuilderGemCutter";
/// Slope of a round brilliant pavilion: 43 % depth over a 50 % radius.
pub const PAVILION_SLOPE: f64 = 0.86;
pub const SEGMENTS: usize = 32;

/// The cutter dimensions, in percent of the stone diameter (seat in percent of the girdle).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    pub girdle_scale: f64,
    pub girdle_width: f64,
    pub z_offset: f64,
    pub top_height: f64,
    pub seat_size: f64,
    pub lower_depth: f64,
    pub lower_taper: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            girdle_scale: 105.0,
            girdle_width: 5.0,
            z_offset: 0.0,
            top_height: 50.0,
            seat_size: 50.0,
            lower_depth: 150.0,
            lower_taper: 0.0,
        }
    }
}

impl Params {
    pub fn from_event(e: &Event) -> Params {
        let d = Params::default();
        Params {
            girdle_scale: e.number("girdle_scale", d.girdle_scale),
            girdle_width: e.number("girdle_width", d.girdle_width),
            z_offset: e.number("cutter_z_offset", d.z_offset),
            top_height: e.number("top_height", d.top_height),
            seat_size: e.number("seat_size", d.seat_size),
            lower_depth: e.number("lower_depth", d.lower_depth),
            lower_taper: e.number("lower_taper", d.lower_taper),
        }
    }
}

/// The revolve profile `(radius, height)` from top to bottom, for a stone of diameter `d`.
pub fn profile(d: f64, p: &Params) -> Vec<(f64, f64)> {
    let rg = d / 2.0 * p.girdle_scale / 100.0;
    let half = d * p.girdle_width / 100.0 / 2.0;
    let rs = rg * (p.seat_size / 100.0).clamp(0.01, 1.0);
    let seat_z = -half - (rg - rs) * PAVILION_SLOPE;
    let bottom = (-d * p.lower_depth / 100.0).min(seat_z - 0.01 * d);
    let rb = (rs * (1.0 + p.lower_taper / 100.0)).max(0.01 * d);
    let top = (d * p.top_height / 100.0).max(half + 0.001 * d);
    let dz = d * p.z_offset / 100.0;
    [
        (rg, top),
        (rg, half),
        (rg, -half),
        (rs, seat_z),
        (rb, bottom),
    ]
    .into_iter()
    .map(|(r, z)| (r, z + dz))
    .collect()
}

/// A closed mesh revolving `profile` (top to bottom, all radii > 0) about the axis.
pub fn revolve(
    center: Vec3,
    axis: Vec3,
    x: Vec3,
    profile: &[(f64, f64)],
    n: usize,
) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let w = unit(axis);
    let u = unit(sub(x, scale(w, dot(x, w))));
    let v = cross(w, u);
    let mut vertices = Vec::new();
    for &(r, z) in profile {
        for i in 0..n {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            vertices.push(add(
                center,
                add(
                    scale(w, z),
                    add(scale(u, r * a.cos()), scale(v, r * a.sin())),
                ),
            ));
        }
    }
    let top = vertices.len() as u32;
    vertices.push(add(center, scale(w, profile[0].1)));
    let bottom = vertices.len() as u32;
    vertices.push(add(center, scale(w, profile[profile.len() - 1].1)));
    let ring = |k: usize, i: usize| (k * n + i % n) as u32;
    let mut t = Vec::new();
    for i in 0..n {
        t.push([top, ring(0, i), ring(0, i + 1)]);
        for k in 0..profile.len() - 1 {
            t.push([ring(k, i), ring(k + 1, i), ring(k + 1, i + 1)]);
            t.push([ring(k, i), ring(k + 1, i + 1), ring(k, i + 1)]);
        }
        let last = profile.len() - 1;
        t.push([bottom, ring(last, i + 1), ring(last, i)]);
    }
    (vertices, t)
}

/// Where a selected stone is: center, table axis, x, diameter (mm).
pub fn gem_frame(s: &Selected, fallback_axis: Vec3) -> Option<(Vec3, Vec3, Vec3, f64)> {
    let g = s.gem.as_ref()?;
    let v3 = |v: &Value| -> Option<Vec3> {
        let a = v.as_array()?;
        Some([
            a.first()?.as_f64()?,
            a.get(1)?.as_f64()?,
            a.get(2)?.as_f64()?,
        ])
    };
    let d = g.get("diameter_mm").and_then(Value::as_f64)?;
    let center = g
        .get("center")
        .and_then(v3)
        .unwrap_or_else(|| scale(add(s.bbox[0], s.bbox[1]), 0.5));
    let axis = g.get("axis").and_then(v3).unwrap_or(fallback_axis);
    let x = g.get("x").and_then(v3).unwrap_or_else(|| {
        let h = if axis[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        cross(axis, h)
    });
    (d > 0.0 && norm(axis) > 0.0).then_some((center, unit(axis), x, d))
}

/// The cutter of one stone: round stones revolve the profile; fancy cuts (oval, pear,
/// emerald …) and custom stones scale their girdle outline through the same stations.
pub fn cutter_for(g: &om9_addon_geom::gems::PlacedGem, p: &Params) -> Geo {
    let d = g.gem.base();
    let stations = profile(d, p);
    let name = format!("Gem Cutter {}", g.gem.size_label());
    let (vertices, triangles) = if g.gem.cut == om9_addon_geom::gems::Cut::Round {
        revolve(g.frame.origin, g.frame.z, g.frame.x, &stations, SEGMENTS)
    } else {
        let o = g.gem.outline(64);
        let rings: Vec<Vec<Vec3>> = stations
            .iter()
            .map(|&(r, z)| {
                let k = r / (d / 2.0);
                o.iter()
                    .map(|q| g.frame.point([q[0] * k, q[1] * k, z]))
                    .collect()
            })
            .collect();
        let top = g.frame.point([0.0, 0.0, stations[0].1]);
        let bottom = g.frame.point([0.0, 0.0, stations[stations.len() - 1].1]);
        let m = om9_addon_geom::mesh::ring_stack(&rings, Some(top), Some(bottom));
        (m.vertices, m.triangles)
    };
    Geo::Mesh {
        name,
        vertices,
        triangles,
        meta: None,
        layer: Some("Cutting".into()),
    }
}

fn cutters(e: &Event) -> (Vec<Geo>, usize) {
    let p = Params::from_event(e);
    let mut out = Vec::new();
    let mut skipped = 0;
    for s in &e.selection {
        let Some(g) = om9_addon_geom::gems::PlacedGem::from_selected(s, e.plane.normal()) else {
            skipped += 1;
            continue;
        };
        out.push(cutter_for(&g, &p));
    }
    (out, skipped)
}

pub struct GemCutter;

impl Interactive for GemCutter {
    const SESSION_COMMANDS: &'static [&'static str] = &[GEM_CUTTER];
    fn step(e: &Event) -> Result<Reply, String> {
        let (geo, skipped) = cutters(e);
        let n = geo.len();
        let note = if skipped > 0 {
            format!(" ({skipped} selected objects are not gems)")
        } else {
            String::new()
        };
        if n == 0 {
            let r = Reply::ask(
                "Select gems for the cutter. Press Enter when done",
                json!({}),
            )
            .want(Want::Objects);
            return Ok(if matches!(e.input, Input::Enter) {
                r.refuse(format!("No gems selected{note}"))
            } else {
                r
            });
        }
        if matches!(e.input, Input::Enter) {
            let mut r =
                Reply::finish(geo).say(format!("Gem Cutter: {n} cutters on layer Cutting{note}"));
            r.records = om9_addon_geom::cmd::remember_style(e);
            return Ok(r);
        }
        Ok(Reply::ask(
            format!(
                "Gem Cutter: {n} cutters{note}. Adjust in the builder or here, Enter to create"
            ),
            json!({}),
        )
        .want(Want::Objects)
        .preview(geo))
    }
}

#[cfg(feature = "export")]
om9_addon_abi::export_interactive!(GemCutter);

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::{Plane, dist};

    fn gem(center: Vec3, d: f64) -> Selected {
        Selected {
            id: "g".into(),
            kind: "mesh".into(),
            gem: Some(
                json!({"cut": "round", "diameter_mm": d, "center": center, "axis": [0, 0, 1], "x": [1, 0, 0]}),
            ),
            ..Default::default()
        }
    }
    fn event(input: Input, selection: Vec<Selected>, settings: Value) -> Event {
        Event {
            command: GEM_CUTTER.into(),
            input,
            state: Value::Null,
            plane: Plane::default(),
            selection,
            pick: None,
            settings: serde_json::from_value(settings).unwrap(),
        }
    }

    #[test]
    fn profile_follows_the_stone() {
        let p = profile(2.0, &Params::default());
        assert_eq!(p[0], (1.05, 1.0), "girdle 105 %, top 50 % of d");
        assert!(
            (p[1].1 - 0.05).abs() < 1e-12 && (p[2].1 + 0.05).abs() < 1e-12,
            "girdle band 5 %"
        );
        // Seat at half the girdle radius, on the pavilion slope.
        assert!((p[3].0 - 0.525).abs() < 1e-12);
        assert!((p[3].1 - (-0.05 - 0.525 * PAVILION_SLOPE)).abs() < 1e-12);
        assert_eq!(p[4], (0.525, -3.0), "hole to 150 % of d");
        let shifted = profile(
            2.0,
            &Params {
                z_offset: 10.0,
                lower_taper: 100.0,
                ..Params::default()
            },
        );
        assert!(
            (shifted[0].1 - 1.2).abs() < 1e-12,
            "Z offset moves the cutter"
        );
        assert!(
            (shifted[4].0 - 1.05).abs() < 1e-12,
            "taper widens the bottom"
        );
    }

    #[test]
    fn revolve_is_a_closed_mesh() {
        let (v, t) = revolve(
            [0.0; 3],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            &profile(1.5, &Params::default()),
            16,
        );
        let mut edges = std::collections::BTreeMap::new();
        for tri in &t {
            for k in 0..3 {
                let (a, b) = (tri[k], tri[(k + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        assert!(
            edges.values().all(|&c| c == 2),
            "every edge shared by two triangles"
        );
        // Signed volume is positive (outward normals) and close to the revolved profile volume.
        let vol: f64 = t
            .iter()
            .map(|f| dot(v[f[0] as usize], cross(v[f[1] as usize], v[f[2] as usize])) / 6.0)
            .sum();
        assert!(vol > 0.0, "{vol}");
    }

    #[test]
    fn one_cutter_per_gem_from_the_selection_and_panel_values() {
        let r = GemCutter::step(&event(Input::Start, vec![], json!({}))).unwrap();
        assert_eq!(r.want, Want::Objects);
        let r = GemCutter::step(&event(Input::Enter, vec![], json!({}))).unwrap();
        assert!(r.error.is_some() && !r.done);
        let gems = vec![gem([0.0; 3], 1.5), gem([5.0, 0.0, 0.0], 2.0)];
        let r = GemCutter::step(&event(Input::Selection, gems.clone(), json!({}))).unwrap();
        assert_eq!(r.preview.len(), 2);
        let r = GemCutter::step(&event(
            Input::Enter,
            gems.clone(),
            json!({"girdle_scale": 120.0}),
        ))
        .unwrap();
        assert!(r.done);
        let Geo::Mesh {
            vertices, layer, ..
        } = &r.create[1]
        else {
            panic!()
        };
        assert_eq!(layer.as_deref(), Some("Cutting"));
        let widest = vertices
            .iter()
            .map(|p| dist([p[0], p[1], 0.0], [5.0, 0.0, 0.0]))
            .fold(0.0, f64::max);
        assert!(
            (widest - 1.2).abs() < 1e-9,
            "Ø2 × 120 % → radius 1.2: {widest}"
        );
        // A selected object that is not a gem is skipped and reported.
        let mut mixed = gems;
        mixed.push(Selected {
            id: "c".into(),
            kind: "curve".into(),
            ..Default::default()
        });
        let r = GemCutter::step(&event(Input::Selection, mixed, json!({}))).unwrap();
        assert!(r.prompt.contains("1 selected objects are not gems"));
    }
}
