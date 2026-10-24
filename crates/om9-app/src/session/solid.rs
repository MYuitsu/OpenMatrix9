// SPDX-License-Identifier: LGPL-2.1-or-later
//! Solid menu command bindings (`OM9-L2-03-18`). The command ids are the feature ids
//! already present in `assets/features.json`; every enabled row dispatches to the OCCT
//! owner in `om9-kernel`, never to a second geometry implementation in the UI.
use super::*;

pub const FEATURES: [&str; 21] = [
    "OM9-SOLID-003", // Boolean Union
    "OM9-SOLID-001", // Boolean Difference
    "OM9-SOLID-002", // Boolean Intersection
    "OM9-SOLID-004", // Boolean Two Objects
    "OM9-SOLID-005", // Cap Planar
    "OM9-SOLID-006", // Extract Surface
    "OM9-SOLID-007", // Fillet Edge
    "OM9-SOLID-008", // Text
    "OM9-SOLID-009", // Extrude Curve Straight
    "OM9-SOLID-010", // Solid Extrude All
    "OM9-SOLID-011", // Pipe
    "OM9-SOLID-012", // Box
    "OM9-SOLID-014", // Sphere
    "OM9-SOLID-013", // Ellipsoid
    "OM9-SOLID-015", // Torus
    "OM9-SOLID-016", // Cylinder
    "OM9-SOLID-017", // Tube
    "OM9-SOLID-018", // Pyramid
    "OM9-SOLID-019", // Cone
    "OM9-SOLID-020", // Truncated Cone
    "OM9-SOLID-029", // Shell
];

pub fn static_feature(feature: &str) -> &'static str {
    match feature {
        "OM9-SOLID-001" => "OM9-SOLID-001",
        "OM9-SOLID-002" => "OM9-SOLID-002",
        "OM9-SOLID-003" => "OM9-SOLID-003",
        "OM9-SOLID-004" => "OM9-SOLID-004",
        "OM9-SOLID-005" => "OM9-SOLID-005",
        "OM9-SOLID-006" => "OM9-SOLID-006",
        "OM9-SOLID-007" => "OM9-SOLID-007",
        "OM9-SOLID-008" => "OM9-SOLID-008",
        "OM9-SOLID-009" => "OM9-SOLID-009",
        "OM9-SOLID-010" => "OM9-SOLID-010",
        "OM9-SOLID-011" => "OM9-SOLID-011",
        "OM9-SOLID-012" => "OM9-SOLID-012",
        "OM9-SOLID-013" => "OM9-SOLID-013",
        "OM9-SOLID-014" => "OM9-SOLID-014",
        "OM9-SOLID-015" => "OM9-SOLID-015",
        "OM9-SOLID-016" => "OM9-SOLID-016",
        "OM9-SOLID-017" => "OM9-SOLID-017",
        "OM9-SOLID-018" => "OM9-SOLID-018",
        "OM9-SOLID-019" => "OM9-SOLID-019",
        "OM9-SOLID-020" => "OM9-SOLID-020",
        "OM9-SOLID-029" => "OM9-SOLID-029",
        _ => "OM9-SOLID-000",
    }
}

pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature)
}

#[allow(dead_code)]
pub fn label(feature: &str) -> &'static str {
    match feature {
        "OM9-SOLID-001" => "Boolean Difference",
        "OM9-SOLID-002" => "Boolean Intersection",
        "OM9-SOLID-003" => "Boolean Union",
        "OM9-SOLID-004" => "Boolean Two Objects",
        "OM9-SOLID-005" => "Cap Planar",
        "OM9-SOLID-006" => "Extract Surface",
        "OM9-SOLID-007" => "Fillet Edge",
        "OM9-SOLID-008" => "Text",
        "OM9-SOLID-009" => "Extrude Curve Straight",
        "OM9-SOLID-010" => "Solid Extrude All",
        "OM9-SOLID-011" => "Pipe",
        "OM9-SOLID-012" => "Box",
        "OM9-SOLID-013" => "Ellipsoid",
        "OM9-SOLID-014" => "Sphere",
        "OM9-SOLID-015" => "Torus",
        "OM9-SOLID-016" => "Cylinder",
        "OM9-SOLID-017" => "Tube",
        "OM9-SOLID-018" => "Pyramid",
        "OM9-SOLID-019" => "Cone",
        "OM9-SOLID-020" => "Truncated Cone",
        "OM9-SOLID-029" => "Shell",
        _ => "Solid",
    }
}

/// Typed command name to feature. Menu invocation without arguments reports the usage
/// explicitly; the command line is the deterministic construction path in this slice.
pub fn feature_for_command(name: &str) -> Option<&'static str> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "booleanunion" | "union" => "OM9-SOLID-003",
        "booleandifference" | "difference" => "OM9-SOLID-001",
        "booleanintersection" | "intersection" => "OM9-SOLID-002",
        "boolean2objects" | "boolean2" => "OM9-SOLID-004",
        "cap" | "capplanar" => "OM9-SOLID-005",
        "extractsrf" | "extractsurface" => "OM9-SOLID-006",
        "filletedge" | "fillet" => "OM9-SOLID-007",
        "text" | "textobject" => "OM9-SOLID-008",
        "extrudecrv" | "extrude" => "OM9-SOLID-009",
        "extrudeall" => "OM9-SOLID-010",
        "pipe" => "OM9-SOLID-011",
        "box" => "OM9-SOLID-012",
        "ellipsoid" => "OM9-SOLID-013",
        "sphere" => "OM9-SOLID-014",
        "torus" => "OM9-SOLID-015",
        "cylinder" => "OM9-SOLID-016",
        "tube" => "OM9-SOLID-017",
        "pyramid" => "OM9-SOLID-018",
        "cone" => "OM9-SOLID-019",
        "tcone" | "truncatedcone" => "OM9-SOLID-020",
        "shell" => "OM9-SOLID-029",
        _ => return None,
    })
}

/// Right-click defaults from the FreeCAD Qt workbench: Box starts 3Point, Sphere
/// starts 2Point without changing the primary left-click behavior.
pub fn mode_command(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "Box3Point" => ("OM9-SOLID-012", "3Point"),
        "Sphere2Point" => ("OM9-SOLID-014", "2Point"),
        _ => return None,
    })
}

pub fn usage(feature: &str) -> &'static str {
    match feature {
        "OM9-SOLID-003" | "OM9-SOLID-001" | "OM9-SOLID-002" => {
            "select exactly two closed B-Rep solids"
        }
        "OM9-SOLID-004" => "select two B-Rep solids; run Boolean2Objects U|D|I",
        "OM9-SOLID-005" => "select one open B-Rep shell",
        "OM9-SOLID-006" => "select one B-Rep; run ExtractSrf <face-index>",
        "OM9-SOLID-007" => "select one B-Rep; run FilletEdge <edge-index> <radius>",
        "OM9-SOLID-008" => "run Text <text> <height>",
        "OM9-SOLID-009" | "OM9-SOLID-010" => "select curve(s); run ExtrudeCrv <height>",
        "OM9-SOLID-011" => "select one curve; run Pipe <radius>",
        "OM9-SOLID-012" => "run Box <x> <y> <z>",
        "OM9-SOLID-013" => "run Ellipsoid <x> <y> <z>",
        "OM9-SOLID-014" => "run Sphere <radius>",
        "OM9-SOLID-015" => "run Torus <ring-radius> <tube-radius>",
        "OM9-SOLID-016" => "run Cylinder <radius> <height>",
        "OM9-SOLID-017" => "run Tube <outer-radius> <inner-radius> <height>",
        "OM9-SOLID-018" => "run Pyramid <sides> <base-radius> <height>",
        "OM9-SOLID-019" => "run Cone <bottom-radius> <height>",
        "OM9-SOLID-020" => "run TCone <bottom-radius> <top-radius> <height>",
        "OM9-SOLID-029" => "select one B-Rep; run Shell <face-index> <thickness>",
        _ => "solid command",
    }
}

#[cfg(feature = "occt-brep")]
#[derive(Default)]
struct TextPen {
    contours: Vec<Vec<om9_core::geometry::Point3>>,
    current: Vec<om9_core::geometry::Point3>,
}

#[cfg(feature = "occt-brep")]
impl skrifa::outline::OutlinePen for TextPen {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.current.len() > 2 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
        self.current.push([x as f64, y as f64, 0.0]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.current.push([x as f64, y as f64, 0.0]);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let Some(start) = self.current.last().copied() else {
            return;
        };
        for step in 1..=12 {
            let t = step as f64 / 12.0;
            let mt = 1.0 - t;
            self.current.push([
                mt * mt * start[0] + 2.0 * mt * t * cx as f64 + t * t * x as f64,
                mt * mt * start[1] + 2.0 * mt * t * cy as f64 + t * t * y as f64,
                0.0,
            ]);
        }
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let Some(start) = self.current.last().copied() else {
            return;
        };
        for step in 1..=16 {
            let t = step as f64 / 16.0;
            let mt = 1.0 - t;
            self.current.push([
                mt * mt * mt * start[0]
                    + 3.0 * mt * mt * t * cx0 as f64
                    + 3.0 * mt * t * t * cx1 as f64
                    + t * t * t * x as f64,
                mt * mt * mt * start[1]
                    + 3.0 * mt * mt * t * cy0 as f64
                    + 3.0 * mt * t * t * cy1 as f64
                    + t * t * t * y as f64,
                0.0,
            ]);
        }
    }
    fn close(&mut self) {
        if self.current.len() > 2 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
    }
}

#[cfg(feature = "occt-brep")]
fn box_wire_edges(
    dims: [f64; 3],
    axes: [[f64; 3]; 3],
    origin: om9_core::geometry::Point3,
) -> Vec<[om9_core::geometry::Point3; 2]> {
    let corner = |bits: usize| {
        [
            origin[0]
                + axes[0][0] * dims[0] * (bits & 1) as f64
                + axes[1][0] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][0] * dims[2] * ((bits >> 2) & 1) as f64,
            origin[1]
                + axes[0][1] * dims[0] * (bits & 1) as f64
                + axes[1][1] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][1] * dims[2] * ((bits >> 2) & 1) as f64,
            origin[2]
                + axes[0][2] * dims[0] * (bits & 1) as f64
                + axes[1][2] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][2] * dims[2] * ((bits >> 2) & 1) as f64,
        ]
    };
    let corners: Vec<_> = (0..8).map(corner).collect();
    [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ]
    .iter()
    .map(|(a, b)| [corners[*a], corners[*b]])
    .collect()
}

#[cfg(feature = "occt-brep")]
fn point_polyline_distance(point: om9_core::geometry::Point3, curve: &[om9_core::geometry::Point3]) -> f64 {
    curve
        .windows(2)
        .map(|window| {
            let a = window[0];
            let b = window[1];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ap = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
            let length = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            let t = if length > 1e-20 {
                ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let nearest = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
            ((point[0] - nearest[0]).powi(2)
                + (point[1] - nearest[1]).powi(2)
                + (point[2] - nearest[2]).powi(2))
            .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(feature = "occt-brep")]
fn tangent_sphere_three_curves(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    third: &[om9_core::geometry::Point3],
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let curves = [first, second, third];
    let mut candidates = Vec::new();
    for i in 0..3 {
        for j in (i + 1)..3 {
            for a in curves[i].windows(2) {
                for b in curves[j].windows(2) {
                    let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
                    let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
                    let cross = da[0] * db[1] - da[1] * db[0];
                    if cross.abs() <= 1e-12 {
                        continue;
                    }
                    let na = [da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                              -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt()];
                    let nb = [db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                              -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt()];
                    for sa in [1.0, -1.0] {
                        for sb in [1.0, -1.0] {
                            let p = [a[0][0] + na[0] * radius * sa, a[0][1] + na[1] * radius * sa];
                            let q = [b[0][0] + nb[0] * radius * sb, b[0][1] + nb[1] * radius * sb];
                            let delta = [q[0] - p[0], q[1] - p[1]];
                            let t = (delta[0] * db[1] - delta[1] * db[0]) / cross;
                            let center = [p[0] + da[0] * t, p[1] + da[1] * t, a[0][2]];
                            if curves.iter().all(|curve| {
                                (point_polyline_distance(center, curve) - radius).abs()
                                    < 1e-6 * radius.max(1.0)
                            }) {
                                candidates.push(center);
                            }
                        }
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-7 && (candidate[1] - other[1]).abs() < 1e-7
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| {
            (a[0].powi(2) + a[1].powi(2))
                .total_cmp(&(b[0].powi(2) + b[1].powi(2)))
        })
        .ok_or(Error::Invalid("no tangent sphere to the three curves"))
}

#[cfg(feature = "occt-brep")]
fn tangent_sphere_two_curves_point(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    point: om9_core::geometry::Point3,
) -> Result<om9_core::geometry::Point3> {
    let mut candidates = Vec::new();
    for a in first.windows(2) {
        for b in second.windows(2) {
            let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
            let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
            let cross = da[0] * db[1] - da[1] * db[0];
            if cross.abs() <= 1e-12 {
                continue;
            }
            let na = [da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                      -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt()];
            let nb = [db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                      -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt()];
            for sa in [1.0, -1.0] {
                for sb in [1.0, -1.0] {
                    let delta0 = [b[0][0] - a[0][0], b[0][1] - a[0][1]];
                    let delta_n = [nb[0] * sb - na[0] * sa, nb[1] * sb - na[1] * sa];
                    let t0 = (delta0[0] * db[1] - delta0[1] * db[0]) / cross;
                    let t1 = (delta_n[0] * db[1] - delta_n[1] * db[0]) / cross;
                    let c0 = [a[0][0] + da[0] * t0, a[0][1] + da[1] * t0];
                    let c1 = [na[0] * sa + da[0] * t1, na[1] * sa + da[1] * t1];
                    let ox = c0[0] - point[0];
                    let oy = c0[1] - point[1];
                    let qa = c1[0].powi(2) + c1[1].powi(2) - 1.0;
                    let qb = 2.0 * (ox * c1[0] + oy * c1[1]);
                    let qc = ox.powi(2) + oy.powi(2);
                    let mut roots = Vec::new();
                    if qa.abs() < 1e-12 {
                        if qb.abs() > 1e-12 {
                            roots.push(-qc / qb);
                        }
                    } else {
                        let disc = qb * qb - 4.0 * qa * qc;
                        if disc >= 0.0 {
                            let root = disc.sqrt();
                            roots.push((-qb + root) / (2.0 * qa));
                            roots.push((-qb - root) / (2.0 * qa));
                        }
                    }
                    for radius in roots {
                        if radius <= 1e-9 {
                            continue;
                        }
                        let center = [c0[0] + c1[0] * radius, c0[1] + c1[1] * radius, a[0][2]];
                        let da_distance = point_polyline_distance(center, first);
                        let db_distance = point_polyline_distance(center, second);
                        let point_distance = ((center[0] - point[0]).powi(2)
                            + (center[1] - point[1]).powi(2))
                        .sqrt();
                        if (da_distance - radius).abs() < 1e-6 * radius.max(1.0)
                            && (db_distance - radius).abs() < 1e-6 * radius.max(1.0)
                            && (point_distance - radius).abs() < 1e-6 * radius.max(1.0)
                        {
                            candidates.push(center);
                        }
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-6 && (candidate[1] - other[1]).abs() < 1e-6
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| {
            (a[0].powi(2) + a[1].powi(2))
                .total_cmp(&(b[0].powi(2) + b[1].powi(2)))
        })
        .ok_or(Error::Invalid("no tangent sphere to the two curves and point"))
}

#[cfg(feature = "occt-brep")]
fn tangent_sphere_curve_two_points(
    curve: &[om9_core::geometry::Point3],
    first: om9_core::geometry::Point3,
    second: om9_core::geometry::Point3,
) -> Result<om9_core::geometry::Point3> {
    let mid = [
        (first[0] + second[0]) * 0.5,
        (first[1] + second[1]) * 0.5,
        (first[2] + second[2]) * 0.5,
    ];
    let dx = second[0] - first[0];
    let dy = second[1] - first[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 1e-12 {
        return Err(Error::Invalid("the two points must be distinct"));
    }
    let perp = [-dy / length, dx / length];
    let error_at = |offset: f64| {
        let center = [mid[0] + perp[0] * offset, mid[1] + perp[1] * offset, mid[2]];
        let radius = ((center[0] - first[0]).powi(2)
            + (center[1] - first[1]).powi(2)
            + (center[2] - first[2]).powi(2))
        .sqrt();
        (point_polyline_distance(center, curve) - radius, center)
    };
    let span = curve
        .iter()
        .chain([&first, &second])
        .fold(1.0_f64, |span, point| {
            span.max(point[0].abs()).max(point[1].abs())
        })
        * 8.0;
    let step = span / 4000.0;
    let mut roots = Vec::new();
    let mut previous = error_at(-span);
    for index in 1..=8000 {
        let offset = -span + step * index as f64;
        let current = error_at(offset);
        if current.0.abs() < 1e-9 {
            roots.push(current.1);
        } else if previous.0 * current.0 < 0.0 {
            let mut low = -span + step * (index - 1) as f64;
            let mut high = offset;
            let mut low_value = previous.0;
            for _ in 0..80 {
                let middle = (low + high) * 0.5;
                let value = error_at(middle).0;
                if low_value * value <= 0.0 {
                    high = middle;
                } else {
                    low = middle;
                    low_value = value;
                }
            }
            roots.push(error_at((low + high) * 0.5).1);
        }
        previous = current;
    }
    roots
        .into_iter()
        .min_by(|a, b| {
            (a[0].powi(2) + a[1].powi(2))
                .total_cmp(&(b[0].powi(2) + b[1].powi(2)))
        })
        .ok_or(Error::Invalid("no tangent sphere through the two points"))
}

#[cfg(feature = "occt-brep")]
fn tangent_sphere_curve_point(
    curve: &[om9_core::geometry::Point3],
    point: om9_core::geometry::Point3,
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let error_at = |angle: f64| {
        let candidate = [
            point[0] + radius * angle.cos(),
            point[1] + radius * angle.sin(),
            point[2],
        ];
        (point_polyline_distance(candidate, curve) - radius).abs()
    };
    let step = std::f64::consts::TAU / 3600.0;
    let mut best: Option<(f64, f64)> = None;
    for index in 0..3600 {
        let angle = step * index as f64;
        let error = error_at(angle);
        if error < 1e-2 * radius.max(1.0)
            && best.as_ref().is_none_or(|(best_error, _)| error < *best_error)
        {
            best = Some((error, angle));
        }
    }
    let Some((_, coarse)) = best else {
        return Err(Error::Invalid("no tangent sphere for curve and point"));
    };
    let mut low = coarse - step;
    let mut high = coarse + step;
    for _ in 0..80 {
        let first = low + (high - low) / 3.0;
        let second = high - (high - low) / 3.0;
        if error_at(first) < error_at(second) {
            high = second;
        } else {
            low = first;
        }
    }
    let angle = (low + high) * 0.5;
    Ok([
        point[0] + radius * angle.cos(),
        point[1] + radius * angle.sin(),
        point[2],
    ])
}

#[cfg(feature = "occt-brep")]
fn tangent_sphere_two_curves(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let mut candidates = Vec::new();
    for a in first.windows(2) {
        for b in second.windows(2) {
            let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
            let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
            let cross = da[0] * db[1] - da[1] * db[0];
            if cross.abs() <= 1e-12 {
                continue;
            }
            let na = [da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                      -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt()];
            let nb = [db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                      -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt()];
            for sa in [1.0, -1.0] {
                for sb in [1.0, -1.0] {
                    let p = [a[0][0] + na[0] * radius * sa, a[0][1] + na[1] * radius * sa];
                    let q = [b[0][0] + nb[0] * radius * sb, b[0][1] + nb[1] * radius * sb];
                    let delta = [q[0] - p[0], q[1] - p[1]];
                    let t = (delta[0] * db[1] - delta[1] * db[0]) / cross;
                    let center = [p[0] + da[0] * t, p[1] + da[1] * t, a[0][2]];
                    let da_distance = point_polyline_distance(center, first);
                    let db_distance = point_polyline_distance(center, second);
                    if (da_distance - radius).abs() < 1e-6 * radius.max(1.0)
                        && (db_distance - radius).abs() < 1e-6 * radius.max(1.0)
                    {
                        candidates.push(center);
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-7 && (candidate[1] - other[1]).abs() < 1e-7
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| {
            (a[0].powi(2) + a[1].powi(2))
                .total_cmp(&(b[0].powi(2) + b[1].powi(2)))
        })
        .ok_or(Error::Invalid("no tangent sphere between the two curves"))
}

#[cfg(feature = "occt-brep")]
fn cube_axes(
    axes: [om9_core::geometry::Point3; 3],
    delta: om9_core::geometry::Point3,
) -> Result<([om9_core::geometry::Point3; 3], f64)> {
    let length = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("cube diagonal"));
    }
    let target = [delta[0] / length, delta[1] / length, delta[2] / length];
    let source = [
        (axes[0][0] + axes[1][0] + axes[2][0]) / 3.0_f64.sqrt(),
        (axes[0][1] + axes[1][1] + axes[2][1]) / 3.0_f64.sqrt(),
        (axes[0][2] + axes[1][2] + axes[2][2]) / 3.0_f64.sqrt(),
    ];
    let cosine = (source[0] * target[0] + source[1] * target[1] + source[2] * target[2])
        .clamp(-1.0, 1.0);
    let v = [
        source[1] * target[2] - source[2] * target[1],
        source[2] * target[0] - source[0] * target[2],
        source[0] * target[1] - source[1] * target[0],
    ];
    let sine = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
    let axis = if sine > 1e-15 {
        [v[0] / sine, v[1] / sine, v[2] / sine]
    } else {
        [
            source[1] * axes[0][2] - source[2] * axes[0][1],
            source[2] * axes[0][0] - source[0] * axes[0][2],
            source[0] * axes[0][1] - source[1] * axes[0][0],
        ]
    };
    let rotate = |point: om9_core::geometry::Point3| {
        let cross = [
            axis[1] * point[2] - axis[2] * point[1],
            axis[2] * point[0] - axis[0] * point[2],
            axis[0] * point[1] - axis[1] * point[0],
        ];
        let dot = axis[0] * point[0] + axis[1] * point[1] + axis[2] * point[2];
        [
            point[0] * cosine + cross[0] * sine + axis[0] * dot * (1.0 - cosine),
            point[1] * cosine + cross[1] * sine + axis[1] * dot * (1.0 - cosine),
            point[2] * cosine + cross[2] * sine + axis[2] * dot * (1.0 - cosine),
        ]
    };
    Ok((axes.map(rotate), length / 3.0_f64.sqrt()))
}

#[cfg(feature = "occt-brep")]
fn polygon_area(points: &[om9_core::geometry::Point3]) -> f64 {
    let mut area = 0.0;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        area += a[0] * b[1] - b[0] * a[1];
    }
    0.5 * area
}

#[cfg(feature = "occt-brep")]
fn inside(point: om9_core::geometry::Point3, polygon: &[om9_core::geometry::Point3]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[j];
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}
#[cfg(feature = "occt-brep")]
use om9_kernel::{
    creation::{self, Primitive, Profile},
    editing::{self, BooleanOp},
};

#[cfg(feature = "occt-brep")]
impl Controller {
    fn solid_preview_sphere(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        let (center, radius) = if let Some(center) = pending.sphere_center
            && matches!(pending.sphere_kind, "Center" | "Vertical")
        {
            let radius = ((point[0] - center[0]).powi(2)
                + (point[1] - center[1]).powi(2)
                + (point[2] - center[2]).powi(2))
            .sqrt();
            (center, radius)
        } else if pending.sphere_kind == "TwoPoint" && pending.points.len() == 1 {
            let first = pending.points[0];
            let center = [
                (first[0] + point[0]) * 0.5,
                (first[1] + point[1]) * 0.5,
                (first[2] + point[2]) * 0.5,
            ];
            let radius = ((point[0] - first[0]).powi(2)
                + (point[1] - first[1]).powi(2)
                + (point[2] - first[2]).powi(2))
            .sqrt()
                * 0.5;
            (center, radius)
        } else if pending.sphere_kind == "ThreePoint" && pending.points.len() == 2 {
            match Self::sphere_circumcenter(pending.points[0], pending.points[1], point) {
                Ok((center, radius, _)) => (center, radius),
                Err(_) => return Vec::new(),
            }
        } else {
            return Vec::new();
        };
        if radius <= 1e-9 {
            return Vec::new();
        }
        let mut edges = Vec::with_capacity(192);
        for axis in 0..3 {
            for index in 0..64 {
                let first = std::f64::consts::TAU * index as f64 / 64.0;
                let second = std::f64::consts::TAU * (index + 1) as f64 / 64.0;
                let point_at = |angle: f64| {
                    let (sin, cos) = angle.sin_cos();
                    match axis {
                        0 => [center[0], center[1] + radius * cos, center[2] + radius * sin],
                        1 => [center[0] + radius * cos, center[1], center[2] + radius * sin],
                        _ => [center[0] + radius * cos, center[1] + radius * sin, center[2]],
                    }
                };
                edges.push([point_at(first), point_at(second)]);
            }
        }
        edges
    }

    fn solid_preview_box(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if !pending.box_mode || pending.points.is_empty() {
            return Vec::new();
        }
        let first = pending.points[0];
        if pending.box_kind == "Center"
            && pending.points.len() == 1
            && pending.box_length > 0.0
            && let Some(width) = pending.box_width
        {
            let height = (point[2] - first[2]).abs();
            if height <= 1e-9 {
                return Vec::new();
            }
            return box_wire_edges(
                [pending.box_length, width, height],
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                [
                    first[0] - pending.box_length * 0.5,
                    first[1] - width * 0.5,
                    first[2],
                ],
            );
        }
        let delta = [point[0] - first[0], point[1] - first[1], point[2] - first[2]];
        let (dims, axes, origin) = if pending.box_kind == "Cube" {
            let Ok((axes, side)) = cube_axes(
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                delta,
            ) else {
                return Vec::new();
            };
            ([side, side, side], axes, first)
        } else if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
            let length = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
            if length <= 0.0 {
                return Vec::new();
            }
            let x = [delta[0] / length, delta[1] / length, delta[2] / length];
            let up = [0.0, 0.0, 1.0];
            let y = if pending.box_kind == "Vertical" {
                up
            } else {
                let mut y = [
                    up[1] * x[2] - up[2] * x[1],
                    up[2] * x[0] - up[0] * x[2],
                    up[0] * x[1] - up[1] * x[0],
                ];
                let norm = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if norm <= 0.0 {
                    return Vec::new();
                }
                y = [y[0] / norm, y[1] / norm, y[2] / norm];
                y
            };
            let z = [
                x[1] * y[2] - x[2] * y[1],
                x[2] * y[0] - x[0] * y[2],
                x[0] * y[1] - x[1] * y[0],
            ];
            let width = pending.box_width.unwrap_or(length);
            ([length, width, width], [x, y, z], first)
        } else {
            let center = pending.box_kind == "Center";
            let factor = if center { 2.0 } else { 1.0 };
            let dims = [
                factor * delta[0].abs(),
                factor * delta[1].abs(),
                factor * delta[1].abs(),
            ];
            let origin = if center {
                [first[0] - dims[0] * 0.5, first[1] - dims[1] * 0.5, first[2]]
            } else {
                [first[0].min(point[0]), first[1].min(point[1]), first[2]]
            };
            (dims, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], origin)
        };
        if dims.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Vec::new();
        }
        let corner = |bits: usize| {
            [
                origin[0]
                    + axes[0][0] * dims[0] * (bits & 1) as f64
                    + axes[1][0] * dims[1] * ((bits >> 1) & 1) as f64
                    + axes[2][0] * dims[2] * ((bits >> 2) & 1) as f64,
                origin[1]
                    + axes[0][1] * dims[0] * (bits & 1) as f64
                    + axes[1][1] * dims[1] * ((bits >> 1) & 1) as f64
                    + axes[2][1] * dims[2] * ((bits >> 2) & 1) as f64,
                origin[2]
                    + axes[0][2] * dims[0] * (bits & 1) as f64
                    + axes[1][2] * dims[1] * ((bits >> 1) & 1) as f64
                    + axes[2][2] * dims[2] * ((bits >> 2) & 1) as f64,
            ]
        };
        let corners: Vec<_> = (0..8).map(corner).collect();
        let edges = [
            (0, 1), (1, 3), (3, 2), (2, 0),
            (4, 5), (5, 7), (7, 6), (6, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];
        edges
            .iter()
            .map(|(a, b)| [corners[*a], corners[*b]])
            .collect()
    }

    fn solid_brep(&self, id: om9_core::identity::ObjectId) -> Result<&om9_core::brep::Brep> {
        match &self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("B-Rep"))?
            .geometry
        {
            Geometry::Brep(brep) => Ok(brep),
            _ => Err(Error::Invalid("selection is not a B-Rep solid")),
        }
    }

    fn insert_solid(
        &mut self,
        feature: &'static str,
        name: &'static str,
        brep: om9_core::brep::Brep,
    ) -> Result<()> {
        let entity = Entity::new(name, Geometry::Brep(brep), self.document().current_layer);
        self.commit_entity(feature, name, entity)
    }
}

pub struct SolidPending {
    pub feature: &'static str,
    pub values: Vec<f64>,
    pub labels: &'static [&'static str],
    text: Option<String>,
    pub points: Vec<om9_core::geometry::Point3>,
    pub box_mode: bool,
    pub box_kind: &'static str,
    pub box_width: Option<f64>,
    pub box_length: f64,
    pub sphere_mode: bool,
    pub sphere_center: Option<om9_core::geometry::Point3>,
    pub sphere_kind: &'static str,
    sphere_radius: f64,
    sphere_normal: om9_core::geometry::Point3,
    pub sphere_curve: Vec<om9_core::geometry::Point3>,
    pub sphere_curve2: Vec<om9_core::geometry::Point3>,
    pub sphere_curve3: Vec<om9_core::geometry::Point3>,
    sphere_solution_negative: bool,
}

impl SolidPending {
    pub(super) fn usage(&self) -> Result<om9_core::budget::Usage> {
        let mut usage = om9_core::budget::Usage::array::<f64>(self.values.capacity())?;
        if let Some(text) = &self.text {
            usage.bytes = usage
                .bytes
                .checked_add(text.capacity() as u64)
                .ok_or(Error::Overflow("text prompt bytes"))?;
        }
        Ok(usage)
    }
    pub fn start(feature: &'static str) -> Option<Self> {
        let labels: &'static [&'static str] = match feature {
            "OM9-SOLID-004" => &["Operation (U=Union D=Difference I=Intersection)"],
            "OM9-SOLID-008" => &["Text", "Height"],
            "OM9-SOLID-012" => &["Width", "Depth", "Height"],
            "OM9-SOLID-013" => &["X radius", "Y radius", "Z radius"],
            "OM9-SOLID-014" => &["Radius"],
            "OM9-SOLID-015" => &["Ring radius", "Tube radius"],
            "OM9-SOLID-016" => &["Radius", "Height"],
            "OM9-SOLID-017" => &["Outer radius", "Inner radius", "Height"],
            "OM9-SOLID-018" => &["Sides", "Base radius", "Height"],
            "OM9-SOLID-019" => &["Bottom radius", "Height"],
            "OM9-SOLID-020" => &["Bottom radius", "Top radius", "Height"],
            "OM9-SOLID-009" | "OM9-SOLID-010" => &["Height"],
            "OM9-SOLID-011" => &["Radius"],
            "OM9-SOLID-029" => &["Face index", "Thickness"],
            "OM9-SOLID-007" => &["Edge index", "Radius"],
            "OM9-SOLID-006" => &["Face index"],
            _ => return None,
        };
        Some(Self {
            feature,
            values: Vec::new(),
            labels,
            text: None,
            points: Vec::new(),
            box_mode: false,
            box_kind: "Corners",
            box_width: None,
            box_length: 0.0,
            sphere_mode: false,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_solution_negative: false,
        })
    }
    pub fn box_pending() -> Self {
        Self {
            feature: "OM9-SOLID-012",
            values: Vec::new(),
            labels: &[
                "First corner of base",
                "Other corner of base or length",
                "Height",
            ],
            text: None,
            points: Vec::new(),
            box_mode: true,
            box_kind: "Corners",
            box_width: None,
            box_length: 0.0,
            sphere_mode: false,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_solution_negative: false,
        }
    }
    pub fn sphere_pending() -> Self {
        Self {
            feature: "OM9-SOLID-014",
            values: Vec::new(),
            labels: &["Center", "Radius"],
            text: None,
            points: Vec::new(),
            box_mode: false,
            box_kind: "Corners",
            box_width: None,
            box_length: 0.0,
            sphere_mode: true,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_solution_negative: false,
        }
    }
    pub fn prompt(&self) -> &'static str {
        if self.box_mode {
            return match (self.points.len(), self.box_kind) {
                (0, "Center") => "Center of base",
                (0, "ThreePoint" | "Vertical") => "Start of edge",
                (0, _) => "First corner of base ( Diagonal 3Point Vertical Center ) :",
                (1, "Diagonal") => "Other corner of base or length ( 3Point ) :",
                (1, "Cube") => "Other corner of box",
                (1, "ThreePoint" | "Vertical") => "End of first edge or Length",
                (1, "Center") if self.box_length > 0.0 && self.box_width.is_none() => {
                    "Width. Press Enter to use length"
                }
                (1, "Center") if self.box_length > 0.0 => "Height. Press Enter to use width",
                (1, "Center") => "Other corner of base or length",
                (1, _) => "Other corner of base or length ( 3Point ) :",
                (_, "ThreePoint" | "Vertical") if self.box_width.is_none() => {
                    "Width. Press Enter to use length"
                }
                (_, "Cube") => "Press Enter to create the cube",
                _ => "Height. Press Enter to use width:",
            };
        }
        if self.sphere_mode {
            return match (self.sphere_kind, self.points.len(), self.sphere_center.is_some()) {
                ("TwoPoint", 0, _) => "First diameter endpoint",
                ("TwoPoint", 1, _) => "Other diameter endpoint or diameter",
                ("ThreePoint", 0, _) => "First point",
                ("ThreePoint", 1, _) => "Second point",
                ("ThreePoint", _, _) => "Third point",
                ("FourPoint", 0, _) => "First point",
                ("FourPoint", 1, _) => "Second point",
                ("FourPoint", 2, _) => "Third point",
                ("FourPoint", _, _) => "Fourth point",
                ("FitPoints", _, _) => "Add points. Press Enter to fit sphere",
                ("AroundCurve", _, false) => "Pick center on curve (OnCurve=0..1)",
                ("AroundCurve", _, true) => "Radius. Pick a point or enter a positive value",
                ("Tangent", _, false) if self.sphere_curve.is_empty() => {
                    "Select tangent curve or Point"
                }
                ("Tangent", _, false) => "Pick tangent point on curve",
                ("Tangent", _, true) => "Radius for tangent sphere",
                ("Tangent2", _, _) => "Radius for two tangent curves",
                ("Tangent3", _, _) => "Radius for three tangent curves",
                ("Tangent2Point", 0, _) => "Pick point constraint",
                ("Tangent2Point", _, _) => "Press Enter for the two curves and point",
                ("TangentCurvePoint", 0, _) => "Pick point constraint",
                ("TangentCurvePoint", 1, _) => "Pick second point or enter radius",
                ("TangentCurvePoint", _, _) => "Press Enter to use the two points",
                ("TangentPoint", 0, _) => "Pick first point constraint",
                ("TangentPoint", 1, _) => "Pick second point constraint",
                ("TangentPoint", 2, _) => "Pick third point or enter radius (Solution=1|2)",
                ("TangentPoint", _, _) => "Radius for two point constraints (Solution=1|2)",
                ("Center" | "Vertical", _, false) => "Center of sphere",
                _ => "Radius. Pick a point or enter a positive value",
            };
        }
        if self.feature == "OM9-SOLID-008" && self.text.is_some() {
            return "Height";
        }
        self.labels
            .get(self.values.len())
            .copied()
            .unwrap_or("Value")
    }
}

impl Controller {
    pub fn dispatch_solid(&mut self, feature: &str) -> Result<()> {
        let feature = static_feature(feature);
        if let Some(binding) = crate::solid_menu::by_feature(feature) {
            if !binding.enabled() {
                return Err(Error::Unsupported(binding.reason));
            }
        } else {
            return Err(Error::Unsupported("solid command is not catalogued"));
        }
        if matches!(
            feature,
            "OM9-SOLID-003" | "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-005"
        ) {
            return self.dispatch_solid_args(feature, &[]);
        }
        if feature == "OM9-SOLID-012" {
            self.solid_pending = Some(SolidPending::box_pending());
            return Ok(());
        }
        if feature == "OM9-SOLID-014" {
            self.solid_pending = Some(SolidPending::sphere_pending());
            return Ok(());
        }
        if let Some(pending) = SolidPending::start(feature) {
            self.solid_pending = Some(pending);
            return Ok(());
        }
        Err(Error::Missing(usage(feature)))
    }

    pub fn solid_fit_selection_points(&self) -> Vec<om9_core::geometry::Point3> {
        let mut points = Vec::new();
        for id in &self.selection {
            let Some(entity) = self.document().entities.get(id) else {
                continue;
            };
            match &entity.geometry {
                Geometry::Point(point) => points.push(*point),
                Geometry::Curve(curve) => {
                    if let Ok(samples) =
                        om9_core::surface_kernel::sample_curve(curve, 16, self.coordinator.limits())
                    {
                        points.extend(samples);
                    }
                }
                Geometry::Mesh(mesh) => {
                    points.extend(mesh.vertices.iter().copied());
                }
                Geometry::Brep(brep) => {
                    for offset in &brep.display.vertices {
                        if let Some(point) = brep.display.points.get(*offset as usize) {
                            points.push(*point);
                        }
                    }
                }
                _ => {}
            }
        }
        points
    }

    pub fn solid_preview_markers(&self) -> Vec<om9_core::geometry::Point3> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        let mut markers = pending.points.clone();
        if let Some(center) = pending.sphere_center {
            markers.push(center);
        }
        markers
    }

    pub fn solid_preview(&self, point: om9_core::geometry::Point3) -> Vec<[om9_core::geometry::Point3; 2]> {
        #[cfg(feature = "occt-brep")]
        {
            if self
                .solid_pending
                .as_ref()
                .is_some_and(|pending| pending.sphere_mode)
            {
                self.solid_preview_sphere(point)
            } else {
                self.solid_preview_box(point)
            }
        }
        #[cfg(not(feature = "occt-brep"))]
        {
            let _ = point;
            Vec::new()
        }
    }

    pub fn solid_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if pending.sphere_mode {
            if pending.sphere_kind == "Tangent"
                && pending.sphere_curve.is_empty()
                && pending.points.is_empty()
                && pending.sphere_center.is_none()
            {
                return vec![
                    ("Point", "Point"),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ];
            }
            if matches!(
                pending.sphere_kind,
                "AroundCurve" | "Tangent" | "Tangent2" | "Tangent3"
            ) && !pending.sphere_curve.is_empty()
                && pending.points.is_empty()
                && pending.sphere_center.is_none()
            {
                if matches!(pending.sphere_kind, "Tangent" | "Tangent2") {
                    return vec![
                        ("Point", "Point"),
                        ("Undo", "Undo"),
                        ("Cancel", "Cancel"),
                    ];
                }
                return vec![("Undo", "Undo"), ("Cancel", "Cancel")];
            }
            if pending.points.is_empty() && pending.sphere_center.is_none() {
                return vec![
                    ("Center", "Mode=Center"),
                    ("2Point", "Mode=2Point"),
                    ("3Point", "Mode=3Point"),
                    ("4Point", "Mode=4Point"),
                    ("Vertical", "Mode=Vertical"),
                    ("AroundCurve", "Mode=AroundCurve"),
                    ("FitPoints", "Mode=FitPoints"),
                    ("Tangent", "Mode=Tangent"),
                    ("Cancel", "Cancel"),
                ];
            }
            let mut tokens = Vec::new();
            if (pending.sphere_kind == "TangentPoint" && pending.points.len() >= 2)
                || (pending.sphere_kind == "Tangent" && pending.sphere_center.is_some())
            {
                tokens.push(("Solution=1", "Solution=1"));
                tokens.push(("Solution=2", "Solution=2"));
            }
            tokens.push(("Radius=", "Radius="));
            tokens.push(("Diameter=", "Diameter="));
            tokens.push(("Undo", "Undo"));
            tokens.push(("Cancel", "Cancel"));
            return tokens;
        }
        if !pending.box_mode {
            return Vec::new();
        }
        if pending.points.is_empty() {
            return vec![
                ("Diagonal", "Mode=Diagonal"),
                ("3Point", "Mode=3Point"),
                ("Vertical", "Mode=Vertical"),
                ("Center", "Mode=Center"),
                ("Cube", "Mode=Cube"),
                ("Cancel", "Cancel"),
            ];
        }
        let mut tokens = Vec::new();
        if pending.points.len() == 1 && pending.box_kind == "Diagonal" {
            tokens.push(("Cube", "Mode=Cube"));
        }
        if !pending.points.is_empty() {
            if pending.box_kind == "Center" && pending.points.len() == 1 {
                tokens.push(("Length=", "Length="));
            }
            tokens.push(("Width=", "Width="));
            tokens.push(("Height=", "Height="));
        }
        tokens.push(("Undo", "Undo"));
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    pub fn submit_solid_text(&mut self, text: &str) -> Option<Result<()>> {
        if let Some(mut pending) = self.solid_pending.take() {
            if pending.sphere_mode
                && pending.sphere_kind == "FitPoints"
                && (text.trim().eq_ignore_ascii_case("selection")
                    || (text.trim().is_empty() && pending.points.is_empty()))
            {
                let points = self.solid_fit_selection_points();
                if points.is_empty() {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Missing(
                        "select points, curves or meshes to fit",
                    )));
                }
                pending.points.extend(points);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if pending.sphere_mode {
                let raw = text.trim().to_ascii_lowercase();
                let option = raw.strip_prefix("mode=").unwrap_or(&raw).to_string();
                let selected = match option.as_str() {
                    "center" | "c" => Some("Center"),
                    "2point" | "2p" | "2"
                        if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                    {
                        Some("TwoPoint")
                    }
                    "3point" | "3p" | "3"
                        if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                    {
                        Some("ThreePoint")
                    }
                    "4point" | "4p" | "4"
                        if !matches!(
                        pending.sphere_kind,
                        "TangentPoint" | "Tangent2" | "Tangent3" | "TangentCurvePoint"
                    ) =>
                    {
                        Some("FourPoint")
                    }
                    "fitpoints" | "fit" | "f" => Some("FitPoints"),
                    "vertical" | "v" => Some("Vertical"),
                    "aroundcurve" | "a" => Some("AroundCurve"),
                    "tangent" | "t" => Some("Tangent"),
                    "point" | "p" if pending.sphere_kind == "Tangent2" => {
                        Some("Tangent2Point")
                    }
                    "point" | "p" if pending.sphere_kind == "Tangent" => {
                        Some(if pending.sphere_curve.is_empty() {
                            "TangentPoint"
                        } else {
                            "TangentCurvePoint"
                        })
                    }
                    _ => None,
                };
                if let Some(kind) = selected {
                    if !pending.points.is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "select the construction mode before the first point",
                        )));
                    }
                    if kind == "AroundCurve" || kind == "Tangent" {
                        let curves: Vec<_> = self
                            .selection
                            .iter()
                            .filter_map(|id| {
                                match &self.document().entities.get(id)?.geometry {
                                    Geometry::Curve(curve) => Some(curve),
                                    _ => None,
                                }
                            })
                            .collect();
                        if curves.len() > 3 {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Missing(
                                "select at most three curves",
                            )));
                        }
                        if curves.is_empty() {
                            pending.sphere_kind = kind;
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        if curves.len() == 3 {
                            if kind != "Tangent" {
                                self.solid_pending = Some(pending);
                                return Some(Err(Error::Missing(
                                    "tangent accepts three curves",
                                )));
                            }
                            let sample = |curve: &om9_core::geometry::Curve| {
                                om9_core::surface_kernel::sample_curve(
                                    curve,
                                    128,
                                    self.coordinator.limits(),
                                )
                            };
                            let first = match sample(curves[0]) {
                                Ok(samples) => samples,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Some(Err(error));
                                }
                            };
                            let second = match sample(curves[1]) {
                                Ok(samples) => samples,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Some(Err(error));
                                }
                            };
                            let third = match sample(curves[2]) {
                                Ok(samples) => samples,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Some(Err(error));
                                }
                            };
                            pending.sphere_curve = first;
                            pending.sphere_curve2 = second;
                            pending.sphere_curve3 = third;
                            pending.sphere_kind = "Tangent3";
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        if curves.len() == 2 {
                            if kind != "Tangent" {
                                self.solid_pending = Some(pending);
                                return Some(Err(Error::Missing(
                                    "AroundCurve accepts one curve",
                                )));
                            }
                            let first = match om9_core::surface_kernel::sample_curve(
                                curves[0],
                                128,
                                self.coordinator.limits(),
                            ) {
                                Ok(samples) => samples,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Some(Err(error));
                                }
                            };
                            let second = match om9_core::surface_kernel::sample_curve(
                                curves[1],
                                128,
                                self.coordinator.limits(),
                            ) {
                                Ok(samples) => samples,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Some(Err(error));
                                }
                            };
                            pending.sphere_curve = first;
                            pending.sphere_curve2 = second;
                            pending.sphere_kind = "Tangent2";
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        let samples = match om9_core::surface_kernel::sample_curve(
                            curves[0],
                            128,
                            self.coordinator.limits(),
                        ) {
                            Ok(samples) => samples,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        if samples.len() < 2 {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("curve has no samples")));
                        }
                        pending.sphere_curve = samples;
                    }
                    pending.sphere_kind = kind;
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if option == "undo" || option == "u" {
                    if pending.sphere_kind == "FourPoint" && pending.points.len() >= 3 {
                        pending.points.pop();
                        pending.sphere_center = None;
                    } else if pending.points.pop().is_none() {
                        pending.sphere_center = None;
                    }
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if pending.sphere_kind == "ThreePoint"
                    && pending.points.len() == 2
                    && text.trim().to_ascii_lowercase().starts_with("radius=")
                {
                    let Some(value) = text
                        .trim()
                        .split_once('=')
                        .and_then(|(_, value)| value.parse::<f64>().ok())
                        .filter(|value| value.is_finite() && *value > 0.0)
                    else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("radius must be positive")));
                    };
                    let a = pending.points[0];
                    let b = pending.points[1];
                    let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
                    let half = ((b[0] - a[0]).powi(2)
                        + (b[1] - a[1]).powi(2)
                        + (b[2] - a[2]).powi(2))
                    .sqrt()
                        * 0.5;
                    if value < half {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "radius must be at least half the chord",
                        )));
                    }
                    let offset = (value * value - half * half).sqrt();
                    let center = [mid[0], mid[1] + offset, mid[2]];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, value));
                }
                if pending.sphere_kind == "AroundCurve"
                    && text.trim().to_ascii_lowercase().starts_with("oncurve=")
                {
                    let Some(fraction) = text
                        .trim()
                        .split_once('=')
                        .and_then(|(_, value)| value.parse::<f64>().ok())
                        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
                    else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("OnCurve must be within 0..1")));
                    };
                    if pending.sphere_curve.len() < 2 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("curve has no samples")));
                    }
                    let position = fraction * (pending.sphere_curve.len() - 1) as f64;
                    let index = position.floor() as usize;
                    let next = (index + 1).min(pending.sphere_curve.len() - 1);
                    let t = position - index as f64;
                    let a = pending.sphere_curve[index];
                    let b = pending.sphere_curve[next];
                    pending.sphere_center = Some([
                        a[0] + (b[0] - a[0]) * t,
                        a[1] + (b[1] - a[1]) * t,
                        a[2] + (b[2] - a[2]) * t,
                    ]);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if pending.sphere_kind == "TwoPoint" && pending.points.len() == 1 {
                    let text_value = text.trim();
                    let value = text_value
                        .strip_prefix("Diameter=")
                        .unwrap_or(text_value)
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0);
                    let Some(diameter) = value else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("diameter must be positive")));
                    };
                    let first = pending.points[0];
                    let center = [
                        first[0] + diameter * 0.5,
                        first[1],
                        first[2],
                    ];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, diameter * 0.5));
                }
                let raw_lower = text.trim().to_ascii_lowercase();
                if matches!(pending.sphere_kind, "AroundCurve" | "Tangent")
                    && pending.sphere_curve.is_empty()
                    && (raw_lower.starts_with("path=") || raw_lower.starts_with("curve="))
                {
                    let value = text
                        .trim()
                        .split_once('=')
                        .map(|(_, name)| name.trim())
                        .unwrap_or_default();
                    let (raw_name, pick) = match value.split_once('@') {
                        Some((name, coords)) => {
                            let values: Vec<f64> = coords
                                .split(',')
                                .filter_map(|value| value.trim().parse::<f64>().ok())
                                .collect();
                            if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
                                self.solid_pending = Some(pending);
                                return Some(Err(Error::Invalid(
                                    "Curve pick syntax: Curve=Object.EdgeN@x,y,z",
                                )));
                            }
                            (name, Some([values[0], values[1], values[2]]))
                        }
                        None => (value, None),
                    };
                    let name = raw_name.split('.').next().unwrap_or(raw_name);
                    let curve = self
                        .document()
                        .entities
                        .values()
                        .find(|entity| entity.label == name)
                        .and_then(|entity| match &entity.geometry {
                            Geometry::Curve(curve) => Some(curve),
                            _ => None,
                        });
                    let Some(curve) = curve else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Missing("Path=Object.EdgeN curve")));
                    };
                    let samples = match om9_core::surface_kernel::sample_curve(
                        curve,
                        128,
                        self.coordinator.limits(),
                    ) {
                        Ok(samples) => samples,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    if let Some(pick) = pick {
                        let nearest = samples
                            .iter()
                            .min_by(|a, b| {
                                let da = (a[0] - pick[0]).powi(2)
                                    + (a[1] - pick[1]).powi(2)
                                    + (a[2] - pick[2]).powi(2);
                                let db = (b[0] - pick[0]).powi(2)
                                    + (b[1] - pick[1]).powi(2)
                                    + (b[2] - pick[2]).powi(2);
                                da.total_cmp(&db)
                            })
                            .copied();
                        if let Some(nearest) = nearest {
                            let _ = nearest;
                            if pending.sphere_kind == "AroundCurve" {
                                pending.sphere_center = Some(pick);
                            }
                        }
                    }
                    pending.sphere_curve = samples;
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if matches!(pending.sphere_kind, "Tangent" | "TangentPoint")
                    && text.trim().to_ascii_lowercase().starts_with("solution=")
                {
                    let Some(solution) = text
                        .trim()
                        .split_once('=')
                        .and_then(|(_, value)| value.parse::<u32>().ok())
                    else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("Solution must be 1 or 2")));
                    };
                    if !(1..=2).contains(&solution) {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("Solution must be 1 or 2")));
                    }
                    pending.sphere_solution_negative = solution == 2;
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if pending.sphere_kind == "Tangent" && pending.sphere_center.is_some() {
                    let Some(radius) = text
                        .trim()
                        .strip_prefix("Radius=")
                        .unwrap_or(text.trim())
                        .parse::<f64>()
                        .ok()
                        .filter(|value| *value > 0.0)
                    else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("tangent radius must be positive")));
                    };
                    let Some(point) = pending.sphere_center else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("tangent point required")));
                    };
                    let normal = pending.sphere_normal;
                    let sign = if pending.sphere_solution_negative { -1.0 } else { 1.0 };
                    let center = [
                        point[0] + normal[0] * radius * sign,
                        point[1] + normal[1] * radius * sign,
                        point[2] + normal[2] * radius * sign,
                    ];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "Tangent2Point" {
                    if pending.points.is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("pick a point constraint first")));
                    }
                    if !text.trim().is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "press Enter to fit the two curves and point",
                        )));
                    }
                    let center = match tangent_sphere_two_curves_point(
                        &pending.sphere_curve,
                        &pending.sphere_curve2,
                        pending.points[0],
                    ) {
                        Ok(center) => center,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    let radius = ((center[0] - pending.points[0][0]).powi(2)
                        + (center[1] - pending.points[0][1]).powi(2)
                        + (center[2] - pending.points[0][2]).powi(2))
                    .sqrt();
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "Tangent3" {
                    let radius = text
                        .trim()
                        .strip_prefix("Radius=")
                        .unwrap_or(text.trim())
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0);
                    let Some(radius) = radius else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("radius must be positive")));
                    };
                    let center = match tangent_sphere_three_curves(
                        &pending.sphere_curve,
                        &pending.sphere_curve2,
                        &pending.sphere_curve3,
                        radius,
                    ) {
                        Ok(center) => center,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "Tangent2" {
                    let radius = text
                        .trim()
                        .strip_prefix("Radius=")
                        .unwrap_or(text.trim())
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0);
                    let Some(radius) = radius else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("radius must be positive")));
                    };
                    let center = match tangent_sphere_two_curves(
                        &pending.sphere_curve,
                        &pending.sphere_curve2,
                        radius,
                    ) {
                        Ok(center) => center,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "TangentCurvePoint" {
                    if pending.points.len() >= 2 {
                        if !text.trim().is_empty() {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid(
                                "press Enter to fit through the two points",
                            )));
                        }
                        let center = match tangent_sphere_curve_two_points(
                            &pending.sphere_curve,
                            pending.points[0],
                            pending.points[1],
                        ) {
                            Ok(center) => center,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Some(Err(error));
                            }
                        };
                        let radius = ((center[0] - pending.points[0][0]).powi(2)
                            + (center[1] - pending.points[0][1]).powi(2)
                            + (center[2] - pending.points[0][2]).powi(2))
                        .sqrt();
                        self.solid_pending = None;
                        return Some(self.solid_sphere_points(center, radius));
                    }
                    if pending.points.is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("pick a point constraint first")));
                    }
                    let radius = text
                        .trim()
                        .strip_prefix("Radius=")
                        .unwrap_or(text.trim())
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0);
                    let Some(radius) = radius else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("radius must be positive")));
                    };
                    let center = match tangent_sphere_curve_point(
                        &pending.sphere_curve,
                        pending.points[0],
                        radius,
                    ) {
                        Ok(center) => center,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Some(Err(error));
                        }
                    };
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "TangentPoint" {
                    if pending.points.len() < 2 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "pick two point constraints first",
                        )));
                    }
                    let value = text
                        .trim()
                        .strip_prefix("Radius=")
                        .unwrap_or(text.trim())
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite() && *value > 0.0);
                    let Some(radius) = value else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("radius must be positive")));
                    };
                    let a = pending.points[0];
                    let b = pending.points[1];
                    let mid = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
                    let half = ((b[0] - a[0]).powi(2)
                        + (b[1] - a[1]).powi(2)
                        + (b[2] - a[2]).powi(2))
                    .sqrt()
                        * 0.5;
                    if radius < half {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "radius must be at least half the point distance",
                        )));
                    }
                    let offset = (radius * radius - half * half).sqrt();
                    let sign = if pending.sphere_solution_negative { -1.0 } else { 1.0 };
                    let center = [mid[0], mid[1] + offset * sign, mid[2]];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if let Some((key, value)) = text.split_once('=') {
                    let key = match key.trim().to_ascii_lowercase().as_str() {
                        "r" => "radius".to_owned(),
                        "d" => "diameter".to_owned(),
                        other => other.to_owned(),
                    };
                    let value = match value.trim().parse::<f64>() {
                        Ok(value)
                            if value.is_finite()
                                && ((key == "height" && value != 0.0)
                                    || (key != "height" && value > 0.0)) =>
                        {
                            value
                        }
                        _ => {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("option value must be positive")));
                        }
                    };
                    if pending.sphere_kind == "Tangent" {
                        let Some(point) = pending.sphere_center else {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("tangent point required")));
                        };
                        if key != "radius" {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("Tangent accepts Radius=")));
                        }
                        let normal = pending.sphere_normal;
                        let sign = if pending.sphere_solution_negative { -1.0 } else { 1.0 };
                        let center = [
                            point[0] + normal[0] * value * sign,
                            point[1] + normal[1] * value * sign,
                            point[2] + normal[2] * value * sign,
                        ];
                        self.solid_pending = None;
                        return Some(self.solid_sphere_points(center, value));
                    }
                    let Some(center) = pending.sphere_center else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("sphere center required")));
                    };
                    let radius = match key.as_str() {
                        "radius" => value,
                        "diameter" => value * 0.5,
                        _ => {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("Sphere accepts Radius= or Diameter=")));
                        }
                    };
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                let numbers: Vec<f64> = text
                    .split([',', ' ', '\t'])
                    .filter(|word| !word.is_empty())
                    .filter_map(|word| word.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .collect();
                if pending.sphere_kind == "Tangent" {
                    let Some(tangent_point) = pending.sphere_center else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "pick the tangent point on the curve first",
                        )));
                    };
                    if numbers.len() != 1 || numbers[0] <= 0.0 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("tangent radius must be positive")));
                    }
                    let radius = numbers[0];
                    let normal = pending.sphere_normal;
                    let center = [
                        tangent_point[0] + normal[0] * radius,
                        tangent_point[1] + normal[1] * radius,
                        tangent_point[2] + normal[2] * radius,
                    ];
                    self.solid_pending = None;
                    return Some(self.solid_sphere_points(center, radius));
                }
                if pending.sphere_kind == "FitPoints" {
                    if !text.trim().is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "press Enter to fit the collected points",
                        )));
                    }
                    if pending.points.len() < 4 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "fit sphere needs at least four points",
                        )));
                    }
                    let points = std::mem::take(&mut pending.points);
                    self.solid_pending = None;
                    return Some(self.fit_sphere_points(&points));
                }
                if pending.sphere_center.is_none() {
                    if numbers.len() != 3 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("sphere center needs three coordinates")));
                    }
                    pending
                        .sphere_center
                        .replace([numbers[0], numbers[1], numbers[2]]);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if numbers.len() != 1 || numbers[0] <= 0.0 {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("radius must be positive")));
                }
                let center = pending.sphere_center.expect("center");
                self.solid_pending = None;
                return Some(self.solid_sphere_points(center, numbers[0]));
            }
            if pending.box_mode {
                let raw = text.trim().to_ascii_lowercase();
                let option = raw.strip_prefix("mode=").unwrap_or(&raw).to_string();
                if option == "corners" {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "Corners is unavailable. Choose Diagonal, 3Point, Vertical or Center",
                    )));
                }
                let selected = match option.as_str() {
                    "diagonal" | "d" => Some("Diagonal"),
                    "3point" | "3p" | "3" => Some("ThreePoint"),
                    "vertical" | "v" => Some("Vertical"),
                    "center" | "c" => Some("Center"),
                    "cube" => Some("Cube"),
                    _ => None,
                };
                if let Some(kind) = selected {
                    if !pending.points.is_empty() {
                        if kind == "Cube"
                            && pending.points.len() == 1
                            && pending.box_kind == "Diagonal"
                        {
                            pending.box_kind = "Cube";
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "select the construction mode before the first point",
                        )));
                    }
                    pending.box_kind = kind;
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if option == "undo" || option == "u" {
                    if pending.box_width.take().is_none() {
                        if pending.box_length > 0.0 {
                            pending.box_length = 0.0;
                        } else {
                            pending.points.pop();
                        }
                    }
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if option == "cancel" || option == "esc" {
                    return Some(Ok(()));
                }
                if pending.box_kind == "Center" && pending.points.len() == 1 {
                    let parse = |value: &str| {
                        value
                            .trim()
                            .parse::<f64>()
                            .ok()
                            .filter(|value| value.is_finite() && *value > 0.0)
                    };
                    if pending.box_length <= 0.0 {
                        let value = text
                            .trim()
                            .strip_prefix("Length=")
                            .or_else(|| text.trim().strip_prefix("L="))
                            .unwrap_or(text.trim());
                        let Some(length) = parse(value) else {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("length must be positive")));
                        };
                        pending.box_length = length;
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    if pending.box_width.is_none() {
                        let value = text
                            .trim()
                            .strip_prefix("Width=")
                            .or_else(|| text.trim().strip_prefix("W="))
                            .unwrap_or(text.trim());
                        let width = if text.trim().is_empty() {
                            pending.box_length
                        } else if let Some(width) = parse(value) {
                            width
                        } else {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("width must be positive")));
                        };
                        pending.box_width = Some(width);
                        self.solid_pending = Some(pending);
                        return Some(Ok(()));
                    }
                    let height_value = text
                        .trim()
                        .strip_prefix("Height=")
                        .or_else(|| text.trim().strip_prefix("H="))
                        .unwrap_or(text.trim());
                    let height = if text.trim().is_empty() {
                        pending.box_width.unwrap_or(pending.box_length)
                    } else if let Some(height) = parse(height_value) {
                        height
                    } else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("height must be positive")));
                    };
                    let center = pending.points[0];
                    let length = pending.box_length;
                    let width = pending.box_width.unwrap_or(length);
                    self.solid_pending = None;
                    return Some(self.solid_box_dimensions(center, length, width, height));
                }
                if let Some((key, value)) = text.split_once('=') {
                    let key = match key.trim().to_ascii_lowercase().as_str() {
                        "l" => "length".to_owned(),
                        "w" => "width".to_owned(),
                        "h" => "height".to_owned(),
                        other => other.to_owned(),
                    };
                    let value = match value.trim().parse::<f64>() {
                        Ok(value)
                            if value.is_finite()
                                && ((key == "height" && value != 0.0)
                                    || (key != "height" && value > 0.0)) =>
                        {
                            value
                        }
                        _ => {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid(
                                "box option value must be non-zero",
                            )));
                        }
                    };
                    if pending.points.len() < 2 {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("pick both base points first")));
                    }
                    match key.as_str() {
                        "width" if matches!(pending.box_kind, "ThreePoint" | "Vertical") => {
                            pending.box_width = Some(value);
                            self.solid_pending = Some(pending);
                            return Some(Ok(()));
                        }
                        "height" if value != 0.0 => {
                            let first = pending.points[0];
                            let second = pending.points[1];
                            let width = pending.box_width;
                            self.solid_pending = None;
                            return Some(self.solid_box_points(
                                first,
                                second,
                                value,
                                pending.box_kind,
                                width,
                            ));
                        }
                        _ => {
                            self.solid_pending = Some(pending);
                            return Some(Err(Error::Invalid("Box accepts Width= or Height=")));
                        }
                    }
                }
                let numbers: Vec<f64> = text
                    .split([',', ' ', '\t'])
                    .filter(|word| !word.is_empty())
                    .filter_map(|word| word.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .collect();
                if pending.points.len() < 2 {
                    if numbers.len() == 1 && numbers[0] > 0.0 && !pending.points.is_empty() {
                        let first = pending.points[0];
                        pending
                            .points
                            .push([first[0] + numbers[0], first[1] + numbers[0], first[2]]);
                    } else if numbers.len() == 3 {
                        pending.points.push([numbers[0], numbers[1], numbers[2]]);
                    } else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid(
                            "base corner needs three coordinates or one length",
                        )));
                    }
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                let first = pending.points[0];
                let second = pending.points[1];
                if pending.box_kind == "Cube" {
                    self.solid_pending = None;
                    return Some(self.solid_box_points(
                        first, second, 1.0, "Cube", None,
                    ));
                }
                if matches!(pending.box_kind, "ThreePoint" | "Vertical")
                    && pending.box_width.is_none()
                {
                    let width = if text.trim().is_empty() {
                        ((second[0] - first[0]).powi(2)
                            + (second[1] - first[1]).powi(2)
                            + (second[2] - first[2]).powi(2))
                        .sqrt()
                    } else if numbers.len() == 1 && numbers[0] > 0.0 {
                        numbers[0]
                    } else {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("width must be positive")));
                    };
                    pending.box_width = Some(width);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                let default_height = if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
                    pending.box_width.unwrap_or(
                        ((second[0] - first[0]).powi(2)
                            + (second[1] - first[1]).powi(2)
                            + (second[2] - first[2]).powi(2))
                        .sqrt(),
                    )
                } else {
                    (second[1] - first[1]).abs()
                };
                let height = if text.trim().is_empty() {
                    default_height
                } else if numbers.len() == 1 && numbers[0] != 0.0 {
                    numbers[0]
                } else {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("height must be non-zero")));
                };
                let width = pending.box_width;
                self.solid_pending = None;
                return Some(self.solid_box_points(
                    first,
                    second,
                    height,
                    pending.box_kind,
                    width,
                ));
            }
            if pending.feature == "OM9-SOLID-004" {
                let operation = text.trim().to_ascii_lowercase();
                if !matches!(
                    operation.as_str(),
                    "u" | "union" | "d" | "difference" | "i" | "intersection"
                ) {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "choose Union, Difference or Intersection",
                    )));
                }
                return Some(self.dispatch_solid_args(pending.feature, &[operation.as_str()]));
            }
            if pending.feature == "OM9-SOLID-008" {
                if pending.text.is_none() {
                    if text.trim().is_empty() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("text must not be empty")));
                    }
                    if text.len() > self.coordinator.limits().max_name_bytes {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Budget("text bytes")));
                    }
                    let mut value = String::new();
                    if value.try_reserve_exact(text.len()).is_err() {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Budget("text input")));
                    }
                    value.push_str(text);
                    pending.text = Some(value);
                    self.solid_pending = Some(pending);
                    return Some(Ok(()));
                }
                if !text
                    .trim()
                    .parse::<f64>()
                    .is_ok_and(|v| v.is_finite() && v > 0.0 && v <= f32::MAX as f64)
                {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid(
                        "text height must be finite and positive",
                    )));
                }
                return Some(self.dispatch_solid_args(
                    pending.feature,
                    &[pending.text.as_deref().expect("text entered"), text.trim()],
                ));
            }
            for word in text.split_whitespace() {
                let value = match word.parse::<f64>() {
                    Ok(value) if value.is_finite() => value,
                    _ => {
                        self.solid_pending = Some(pending);
                        return Some(Err(Error::Invalid("solid dimension")));
                    }
                };
                pending.values.push(value);
            }
            if pending.values.len() < pending.labels.len() {
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            let args: Vec<String> = pending
                .values
                .iter()
                .map(|value| value.to_string())
                .collect();
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            return Some(self.dispatch_solid_args(pending.feature, &refs));
        }
        let mut words = text.split_whitespace();
        let name = words.next()?;
        let feature = feature_for_command(name)?;
        let args: Vec<&str> = words.collect();
        if args.is_empty() && feature == "OM9-SOLID-012" {
            self.solid_pending = Some(SolidPending::box_pending());
            return Some(Ok(()));
        }
        if args.is_empty() && feature == "OM9-SOLID-014" {
            self.solid_pending = Some(SolidPending::sphere_pending());
            return Some(Ok(()));
        }
        if args.is_empty()
            && let Some(pending) = SolidPending::start(feature)
        {
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        Some(self.dispatch_solid_args(feature, &args))
    }
}

#[cfg(feature = "occt-brep")]
impl Controller {
    pub fn dispatch_solid_args(&mut self, feature: &'static str, args: &[&str]) -> Result<()> {
        let topology_index = |value: f64| -> Result<usize> {
            if value < 0.0 || value.fract() != 0.0 {
                return Err(Error::Invalid(
                    "topology index must be a nonnegative integer",
                ));
            }
            Ok(value as usize)
        };
        let values = |count: usize| -> Result<Vec<f64>> {
            if args.len() != count {
                return Err(Error::Missing(usage(feature)));
            }
            args.iter()
                .map(|word| {
                    word.parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite())
                        .ok_or(Error::Invalid("solid dimension"))
                })
                .collect()
        };
        match feature {
            "OM9-SOLID-003" | "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-004" => {
                let op = match feature {
                    "OM9-SOLID-003" => BooleanOp::Union,
                    "OM9-SOLID-001" => BooleanOp::Difference,
                    "OM9-SOLID-002" => BooleanOp::Intersection,
                    _ => match args.first().map(|word| word.to_ascii_lowercase()) {
                        Some(word) if word == "u" || word == "union" => BooleanOp::Union,
                        Some(word) if word == "d" || word == "difference" => BooleanOp::Difference,
                        Some(word) if word == "i" || word == "intersection" => {
                            BooleanOp::Intersection
                        }
                        _ => return Err(Error::Missing(usage(feature))),
                    },
                };
                self.solid_boolean(feature, op)
            }
            "OM9-SOLID-005" => self.solid_cap(feature),
            "OM9-SOLID-006" => {
                let face = topology_index(values(1)?[0])?;
                self.solid_extract(feature, face)
            }
            "OM9-SOLID-007" => {
                let v = values(2)?;
                self.solid_fillet(feature, topology_index(v[0])?, v[1])
            }
            "OM9-SOLID-008" => {
                if args.len() < 2 {
                    return Err(Error::Missing(usage(feature)));
                }
                let height = args[args.len() - 1]
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite())
                    .ok_or(Error::Invalid("text height"))?;
                let text = args[..args.len() - 1].join(" ");
                self.solid_text(feature, &text, height)
            }
            "OM9-SOLID-009" | "OM9-SOLID-010" => {
                let height = values(1)?[0];
                self.solid_extrude(feature, height, feature == "OM9-SOLID-010")
            }
            "OM9-SOLID-011" => {
                let radius = values(1)?[0];
                self.solid_pipe(feature, radius)
            }
            "OM9-SOLID-012" => {
                let v = values(3)?;
                self.solid_primitive(
                    feature,
                    Primitive::Box {
                        x_mm: v[0],
                        y_mm: v[1],
                        z_mm: v[2],
                    },
                )
            }
            "OM9-SOLID-013" => {
                let v = values(3)?;
                self.solid_primitive(
                    feature,
                    Primitive::Ellipsoid {
                        x_mm: v[0],
                        y_mm: v[1],
                        z_mm: v[2],
                    },
                )
            }
            "OM9-SOLID-014" => {
                let v = values(1)?;
                self.solid_primitive(feature, Primitive::Sphere { radius_mm: v[0] })
            }
            "OM9-SOLID-015" => {
                let v = values(2)?;
                self.solid_primitive(
                    feature,
                    Primitive::Torus {
                        ring_radius_mm: v[0],
                        tube_radius_mm: v[1],
                    },
                )
            }
            "OM9-SOLID-016" => {
                let v = values(2)?;
                self.solid_primitive(
                    feature,
                    Primitive::Cylinder {
                        radius_mm: v[0],
                        height_mm: v[1],
                    },
                )
            }
            "OM9-SOLID-017" => {
                let v = values(3)?;
                self.solid_tube(feature, v[0], v[1], v[2])
            }
            "OM9-SOLID-018" => {
                let v = values(3)?;
                self.solid_primitive(
                    feature,
                    Primitive::Pyramid {
                        sides: v[0] as u32,
                        base_radius_mm: v[1],
                        height_mm: v[2],
                    },
                )
            }
            "OM9-SOLID-019" => {
                let v = values(2)?;
                self.solid_primitive(
                    feature,
                    Primitive::Cone {
                        bottom_radius_mm: v[0],
                        top_radius_mm: 0.0,
                        height_mm: v[1],
                    },
                )
            }
            "OM9-SOLID-020" => {
                let v = values(3)?;
                self.solid_primitive(
                    feature,
                    Primitive::Cone {
                        bottom_radius_mm: v[0],
                        top_radius_mm: v[1],
                        height_mm: v[2],
                    },
                )
            }
            "OM9-SOLID-029" => {
                let v = values(2)?;
                self.solid_shell(feature, topology_index(v[0])?, v[1])
            }
            _ => Err(Error::Unsupported(usage(feature))),
        }
    }
}

#[cfg(not(feature = "occt-brep"))]
impl Controller {
    fn solid_sphere_points(
        &mut self,
        _center: om9_core::geometry::Point3,
        _radius: f64,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }

    pub fn solid_box_points(
        &mut self,
        _first: om9_core::geometry::Point3,
        _second: om9_core::geometry::Point3,
        _height: f64,
        _kind: &str,
        _width: Option<f64>,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }

    pub fn dispatch_solid_args(&mut self, _feature: &'static str, _args: &[&str]) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
}

#[cfg(feature = "occt-brep")]
impl Controller {
    fn sphere_circumcenter(
        a: om9_core::geometry::Point3,
        b: om9_core::geometry::Point3,
        c: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64, om9_core::geometry::Point3)> {
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let nn = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        if nn <= 1e-20 {
            return Err(Error::Invalid("three points are collinear"));
        }
        let uu = u[0] * u[0] + u[1] * u[1] + u[2] * u[2];
        let vv = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        let vn = [
            v[1] * n[2] - v[2] * n[1],
            v[2] * n[0] - v[0] * n[2],
            v[0] * n[1] - v[1] * n[0],
        ];
        let nu = [
            n[1] * u[2] - n[2] * u[1],
            n[2] * u[0] - n[0] * u[2],
            n[0] * u[1] - n[1] * u[0],
        ];
        let center = [
            a[0] + (vn[0] * uu + nu[0] * vv) / (2.0 * nn),
            a[1] + (vn[1] * uu + nu[1] * vv) / (2.0 * nn),
            a[2] + (vn[2] * uu + nu[2] * vv) / (2.0 * nn),
        ];
        let radius = ((center[0] - a[0]).powi(2)
            + (center[1] - a[1]).powi(2)
            + (center[2] - a[2]).powi(2))
        .sqrt();
        Ok((center, radius, [n[0] / nn.sqrt(), n[1] / nn.sqrt(), n[2] / nn.sqrt()]))
    }

    /// The point under the pointer for a Box height step. The usual construction-plane
    /// intersection pins z to the plane, so clicking for a height would always report zero;
    /// this intersects a vertical plane through the first base point instead.
    pub fn box_height_ray_point(
        &self,
        ray: &om9_scene::camera::Ray,
    ) -> Option<om9_core::geometry::Point3> {
        let pending = self.solid_pending.as_ref()?;
        if !pending.box_mode {
            return None;
        }
        let height_step = (pending.points.len() >= 2
            && !(matches!(pending.box_kind, "ThreePoint" | "Vertical")
                && pending.box_width.is_none()))
            || (pending.box_kind == "Center"
                && pending.points.len() == 1
                && pending.box_length > 0.0
                && pending.box_width.is_some());
        if !height_step {
            return None;
        }
        let first = pending.points[0];
        let mut normal = [ray.direction[0], ray.direction[1], 0.0];
        let length = (normal[0].powi(2) + normal[1].powi(2)).sqrt();
        if length <= 1e-9 {
            normal = [0.0, 1.0, 0.0];
        } else {
            normal = [normal[0] / length, normal[1] / length, 0.0];
        }
        let denominator = ray.direction[0] * normal[0]
            + ray.direction[1] * normal[1]
            + ray.direction[2] * normal[2];
        if denominator.abs() <= 1e-12 {
            return None;
        }
        let offset = [first[0] - ray.origin[0], first[1] - ray.origin[1], first[2] - ray.origin[2]];
        let t = (offset[0] * normal[0] + offset[1] * normal[1] + offset[2] * normal[2])
            / denominator;
        if t <= 0.0 {
            return None;
        }
        Some([
            ray.origin[0] + ray.direction[0] * t,
            ray.origin[1] + ray.direction[1] * t,
            ray.origin[2] + ray.direction[2] * t,
        ])
    }

    pub fn accept_sphere_point(&mut self, point: om9_core::geometry::Point3) -> Result<()> {
        let Some(mut pending) = self.solid_pending.take() else {
            return Err(Error::Missing("sphere command"));
        };
        if !pending.sphere_mode {
            self.solid_pending = Some(pending);
            return Err(Error::Missing("sphere command"));
        }
        match pending.sphere_kind {
            "TwoPoint" => {
                pending.points.push(point);
                if pending.points.len() < 2 {
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
                let a = pending.points[0];
                let b = pending.points[1];
                let center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, (a[2] + b[2]) * 0.5];
                let radius = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2))
                    .sqrt()
                    * 0.5;
                self.solid_pending = None;
                self.solid_sphere_points(center, radius)
            }
            "Tangent2Point" | "TangentCurvePoint" => {
                pending.points.push(point);
                self.solid_pending = Some(pending);
                Ok(())
            }
            "TangentPoint" => {
                pending.points.push(point);
                if pending.points.len() >= 3 {
                    let (center, radius, _) = Self::sphere_circumcenter(
                        pending.points[0],
                        pending.points[1],
                        pending.points[2],
                    )?;
                    self.solid_pending = None;
                    return self.solid_sphere_points(center, radius);
                }
                self.solid_pending = Some(pending);
                Ok(())
            }
            "Tangent" => {
                let Some(tangent_point) = pending.sphere_center else {
                    let nearest = pending
                        .sphere_curve
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            let da = (a[0] - point[0]).powi(2)
                                + (a[1] - point[1]).powi(2)
                                + (a[2] - point[2]).powi(2);
                            let db = (b[0] - point[0]).powi(2)
                                + (b[1] - point[1]).powi(2)
                                + (b[2] - point[2]).powi(2);
                            da.total_cmp(&db)
                        })
                        .map(|(index, point)| (index, *point));
                    let Some((index, _sampled)) = nearest else {
                        self.solid_pending = Some(pending);
                        return Err(Error::Invalid("Tangent has no sampled curve"));
                    };
                    let previous = pending.sphere_curve[index.saturating_sub(1)];
                    let next =
                        pending.sphere_curve[(index + 1).min(pending.sphere_curve.len() - 1)];
                    let tangent = [
                        next[0] - previous[0],
                        next[1] - previous[1],
                        next[2] - previous[2],
                    ];
                    let length_squared =
                        tangent[0].powi(2) + tangent[1].powi(2) + tangent[2].powi(2);
                    let used = if length_squared > 1e-20 {
                        ((point[0] - previous[0]) * tangent[0]
                            + (point[1] - previous[1]) * tangent[1]
                            + (point[2] - previous[2]) * tangent[2])
                            / length_squared
                    } else {
                        0.0
                    }
                    .clamp(0.0, 1.0);
                    let tangent_point = [
                        previous[0] + tangent[0] * used,
                        previous[1] + tangent[1] * used,
                        previous[2] + tangent[2] * used,
                    ];
                    let mut normal = [
                        tangent[1] * 1.0 - tangent[2] * 0.0,
                        tangent[2] * 0.0 - tangent[0] * 1.0,
                        tangent[0] * 0.0 - tangent[1] * 0.0,
                    ];
                    let mut norm =
                        (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
                    if norm <= 1e-12 {
                        normal = [
                            tangent[1] * 0.0 - tangent[2] * 0.0,
                            tangent[2] * 1.0 - tangent[0] * 0.0,
                            tangent[0] * 0.0 - tangent[1] * 1.0,
                        ];
                        norm =
                            (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
                    }
                    if norm <= 0.0 {
                        self.solid_pending = Some(pending);
                        return Err(Error::Invalid("degenerate curve tangent"));
                    }
                    normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
                    pending.sphere_center = Some(tangent_point);
                    pending.sphere_normal = normal;
                    self.solid_pending = Some(pending);
                    return Ok(());
                };
                let normal = pending.sphere_normal;
                let delta = [
                    point[0] - tangent_point[0],
                    point[1] - tangent_point[1],
                    point[2] - tangent_point[2],
                ];
                let radius =
                    (delta[0] * normal[0] + delta[1] * normal[1] + delta[2] * normal[2]).abs();
                if radius <= 0.0 {
                    self.solid_pending = Some(pending);
                    return Err(Error::Invalid("tangent radius"));
                }
                let center = [
                    tangent_point[0] + normal[0] * radius,
                    tangent_point[1] + normal[1] * radius,
                    tangent_point[2] + normal[2] * radius,
                ];
                self.solid_pending = None;
                self.solid_sphere_points(center, radius)
            }
            "AroundCurve" => {
                if let Some(center) = pending.sphere_center {
                    let radius = ((point[0] - center[0]).powi(2)
                        + (point[1] - center[1]).powi(2)
                        + (point[2] - center[2]).powi(2))
                    .sqrt();
                    self.solid_pending = None;
                    self.solid_sphere_points(center, radius)
                } else {
                    let nearest = pending
                        .sphere_curve
                        .iter()
                        .min_by(|a, b| {
                            let da = (a[0] - point[0]).powi(2)
                                + (a[1] - point[1]).powi(2)
                                + (a[2] - point[2]).powi(2);
                            let db = (b[0] - point[0]).powi(2)
                                + (b[1] - point[1]).powi(2)
                                + (b[2] - point[2]).powi(2);
                            da.total_cmp(&db)
                        })
                        .copied();
                    let Some(center) = nearest else {
                        self.solid_pending = Some(pending);
                        return Err(Error::Invalid("AroundCurve has no sampled curve"));
                    };
                    pending.sphere_center = Some(center);
                    self.solid_pending = Some(pending);
                    Ok(())
                }
            }
            "FitPoints" => {
                pending.points.push(point);
                self.solid_pending = Some(pending);
                Ok(())
            }
            "ThreePoint" | "FourPoint" => {
                pending.points.push(point);
                if pending.sphere_kind == "ThreePoint" {
                    if pending.points.len() == 3 {
                        let (center, radius, _) = Self::sphere_circumcenter(
                            pending.points[0],
                            pending.points[1],
                            pending.points[2],
                        )?;
                        self.solid_pending = None;
                        return self.solid_sphere_points(center, radius);
                    }
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
                if pending.sphere_kind == "FourPoint" {
                    match pending.points.len() {
                        3 => {
                            let (center, radius, normal) = Self::sphere_circumcenter(
                                pending.points[0],
                                pending.points[1],
                                pending.points[2],
                            )?;
                            pending.sphere_center = Some(center);
                            pending.sphere_radius = radius;
                            pending.sphere_normal = normal;
                            self.solid_pending = Some(pending);
                            return Ok(());
                        }
                        4 => {
                            let center = pending.sphere_center.expect("circle center");
                            let radius = pending.sphere_radius;
                            let normal = pending.sphere_normal;
                            let point = pending.points[3];
                            let delta = [
                                point[0] - center[0],
                                point[1] - center[1],
                                point[2] - center[2],
                            ];
                            let height = delta[0] * normal[0]
                                + delta[1] * normal[1]
                                + delta[2] * normal[2];
                            if height.abs() <= 1e-12 {
                                self.solid_pending = Some(pending);
                                return Err(Error::Invalid(
                                    "fourth point must be outside the circle plane",
                                ));
                            }
                            let length = (delta[0].powi(2)
                                + delta[1].powi(2)
                                + delta[2].powi(2))
                            .sqrt();
                            let offset = (length * length - radius * radius) / (2.0 * height);
                            let result = [
                                center[0] + normal[0] * offset,
                                center[1] + normal[1] * offset,
                                center[2] + normal[2] * offset,
                            ];
                            let result_radius = (radius * radius + offset * offset).sqrt();
                            self.solid_pending = None;
                            return self.solid_sphere_points(result, result_radius);
                        }
                        _ => {}
                    }
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
                Ok(())
            }
            _ => {
                if let Some(center) = pending.sphere_center {
                    let radius = ((point[0] - center[0]).powi(2)
                        + (point[1] - center[1]).powi(2)
                        + (point[2] - center[2]).powi(2))
                    .sqrt();
                    self.solid_pending = None;
                    self.solid_sphere_points(center, radius)
                } else {
                    pending.sphere_center = Some(point);
                    self.solid_pending = Some(pending);
                    Ok(())
                }
            }
        }
    }

    fn fit_sphere_points(
        &mut self,
        points: &[om9_core::geometry::Point3],
    ) -> Result<()> {
        if points.len() < 4 {
            return Err(Error::Invalid("fit sphere needs at least four points"));
        }
        let mut normal = [[0.0f64; 5]; 4];
        for point in points {
            let row = [2.0 * point[0], 2.0 * point[1], 2.0 * point[2], 1.0];
            let rhs = point[0] * point[0] + point[1] * point[1] + point[2] * point[2];
            for i in 0..4 {
                for j in 0..4 {
                    normal[i][j] += row[i] * row[j];
                }
                normal[i][4] += row[i] * rhs;
            }
        }
        for column in 0..4 {
            let pivot = (column..4)
                .max_by(|a, b| normal[*a][column].abs().total_cmp(&normal[*b][column].abs()))
                .expect("pivot");
            if normal[pivot][column].abs() <= 1e-12 {
                return Err(Error::Invalid("fit sphere is degenerate"));
            }
            normal.swap(column, pivot);
            let divisor = normal[column][column];
            for value in &mut normal[column][column..] {
                *value /= divisor;
            }
            let pivot_row = normal[column];
            for (row, values) in normal.iter_mut().enumerate() {
                if row == column {
                    continue;
                }
                let factor = values[column];
                for (value, pivot_value) in values[column..].iter_mut().zip(&pivot_row[column..]) {
                    *value -= factor * pivot_value;
                }
            }
        }
        let center = [normal[0][4], normal[1][4], normal[2][4]];
        let radius_squared = normal[3][4]
            + center[0] * center[0]
            + center[1] * center[1]
            + center[2] * center[2];
        if !radius_squared.is_finite() || radius_squared <= 0.0 {
            return Err(Error::Invalid("fit sphere radius"));
        }
        let radius = radius_squared.sqrt();
        for point in points {
            let distance = ((point[0] - center[0]).powi(2)
                + (point[1] - center[1]).powi(2)
                + (point[2] - center[2]).powi(2))
            .sqrt();
            if (distance - radius).abs() > 1e-6 * radius.max(1.0) {
                return Err(Error::Invalid("points do not lie on one sphere"));
            }
        }
        self.solid_sphere_points(center, radius)
    }

    pub fn solid_sphere_points(
        &mut self,
        center: om9_core::geometry::Point3,
        radius: f64,
    ) -> Result<()> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::Invalid("sphere radius must be positive"));
        }
        let brep = creation::primitive(
            Primitive::Sphere { radius_mm: radius },
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("sphere: {error}")))?;
        let mut entity = Entity::new(
            label("OM9-SOLID-014"),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        entity.placement = om9_core::geometry::Transform::translation(center);
        self.commit_entity("OM9-SOLID-014", label("OM9-SOLID-014"), entity)
    }

    pub fn solid_box_dimensions(
        &mut self,
        center: om9_core::geometry::Point3,
        length: f64,
        width: f64,
        height: f64,
    ) -> Result<()> {
        if [length, width, height]
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(Error::Invalid("box dimensions must be positive"));
        }
        let brep = creation::primitive(
            Primitive::Box {
                x_mm: length,
                y_mm: width,
                z_mm: height,
            },
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("box: {error}")))?;
        let mut entity = Entity::new(
            label("OM9-SOLID-012"),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        entity.placement = om9_core::geometry::Transform::translation([
            center[0] - length * 0.5,
            center[1] - width * 0.5,
            center[2],
        ]);
        self.commit_entity("OM9-SOLID-012", label("OM9-SOLID-012"), entity)
    }

    pub fn solid_box_points(
        &mut self,
        first: om9_core::geometry::Point3,
        second: om9_core::geometry::Point3,
        height: f64,
        kind: &str,
        width_override: Option<f64>,
    ) -> Result<()> {
        let delta = [
            second[0] - first[0],
            second[1] - first[1],
            second[2] - first[2],
        ];
        let signed_height = if height.is_finite() && height != 0.0 {
            height
        } else {
            return Err(Error::Invalid("box height must be non-zero"));
        };
        let (dims, axes, origin) = if kind == "Cube" {
            let (axes, side) = cube_axes(
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                delta,
            )?;
            ([side, side, side], axes, first)
        } else if matches!(kind, "ThreePoint" | "Vertical") {
            let length = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
            if length <= 0.0 {
                return Err(Error::Invalid("box needs a non-zero edge"));
            }
            let x = [delta[0] / length, delta[1] / length, delta[2] / length];
            let up = [0.0, 0.0, 1.0];
            let (y, z) = if kind == "Vertical" {
                let y = up;
                let z = [
                    x[1] * y[2] - x[2] * y[1],
                    x[2] * y[0] - x[0] * y[2],
                    x[0] * y[1] - x[1] * y[0],
                ];
                (y, z)
            } else {
                let mut y = [
                    up[1] * x[2] - up[2] * x[1],
                    up[2] * x[0] - up[0] * x[2],
                    up[0] * x[1] - up[1] * x[0],
                ];
                let y_len = (y[0].powi(2) + y[1].powi(2) + y[2].powi(2)).sqrt();
                if y_len <= 0.0 {
                    return Err(Error::Invalid("box edge is parallel to the construction plane"));
                }
                y = [y[0] / y_len, y[1] / y_len, y[2] / y_len];
                let z = [
                    x[1] * y[2] - x[2] * y[1],
                    x[2] * y[0] - x[0] * y[2],
                    x[0] * y[1] - x[1] * y[0],
                ];
                (y, z)
            };
            (
                [length, width_override.unwrap_or(length), signed_height.abs()],
                [x, y, z],
                first,
            )
        } else {
            let center = kind == "Center";
            let factor = if center { 2.0 } else { 1.0 };
            (
                [
                    factor * delta[0].abs(),
                    factor * delta[1].abs(),
                    signed_height.abs(),
                ],
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                if center {
                    [
                        first[0] - factor * delta[0].abs() * 0.5,
                        first[1] - factor * delta[1].abs() * 0.5,
                        first[2],
                    ]
                } else {
                    [
                        first[0].min(second[0]),
                        first[1].min(second[1]),
                        first[2],
                    ]
                },
            )
        };
        let origin = [
            origin[0] + axes[2][0] * signed_height.min(0.0),
            origin[1] + axes[2][1] * signed_height.min(0.0),
            origin[2] + axes[2][2] * signed_height.min(0.0),
        ];
        if dims.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Err(Error::Invalid(
                "box needs a non-zero base and positive height",
            ));
        }
        let brep = creation::primitive(
            Primitive::Box {
                x_mm: dims[0],
                y_mm: dims[1],
                z_mm: dims[2],
            },
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("box: {error}")))?;
        let mut entity = Entity::new(
            label("OM9-SOLID-012"),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut matrix = om9_core::geometry::Transform::identity().0;
        for column in 0..3 {
            for row in 0..3 {
                matrix[row][column] = axes[column][row];
            }
        }
        for row in 0..3 {
            matrix[row][3] = origin[row];
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity("OM9-SOLID-012", label("OM9-SOLID-012"), entity)
    }

    fn solid_primitive(&mut self, feature: &'static str, primitive: Primitive) -> Result<()> {
        let brep = creation::primitive(
            primitive,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("solid primitive: {error}")))?;
        self.insert_solid(feature, label(feature), brep)
    }

    fn selected_breps(&self) -> Result<Vec<(om9_core::identity::ObjectId, om9_core::brep::Brep)>> {
        self.selection
            .iter()
            .map(|id| {
                let brep = self.solid_brep(*id)?;
                let entity = &self.document().entities[id];
                let world = if entity.placement == om9_core::geometry::Transform::identity() {
                    brep.clone()
                } else {
                    om9_kernel::transform_ops::transform_brep(
                        brep,
                        &entity.placement,
                        brep.display.deflection_mm,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("solid placement: {error}")))?
                };
                Ok((*id, world))
            })
            .collect()
    }

    fn commit_solid_result(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mutations: Vec<Mutation>,
        select: Option<Vec<om9_core::identity::ObjectId>>,
    ) -> Result<()> {
        let held = self.auxiliary_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
            if let Some(ids) = select {
                self.select_objects(&ids)?;
            }
            self.events.emit(
                crate::command_events::EventKind::Finished,
                feature,
                "committed",
                Some(crate::command_events::EventOutcome::Committed),
            );
        }
        Ok(())
    }

    fn solid_boolean(&mut self, feature: &'static str, op: BooleanOp) -> Result<()> {
        let selected = self.selected_breps()?;
        if selected.len() != 2 {
            return Err(Error::Missing("select exactly two B-Rep solids"));
        }
        let outcome = editing::boolean(
            &selected[0].1,
            &selected[1].1,
            op,
            selected[0].1.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("solid boolean: {error}")))?;
        let Some(brep) = outcome.body() else {
            return Err(Error::Invalid("boolean produced an empty body"));
        };
        let entity = Entity::new(
            label(feature),
            Geometry::Brep(brep.clone()),
            self.document().current_layer,
        );
        let result_id = entity.id;
        let mut mutations = Vec::new();
        for (id, _) in &selected {
            mutations.push(Mutation::Delete {
                source: self.document().stamp(*id)?,
            });
        }
        mutations.push(Mutation::Insert {
            document: self.document,
            entity,
        });
        self.commit_solid_result(feature, label(feature), mutations, Some(vec![result_id]))
    }

    fn solid_cap(&mut self, feature: &'static str) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one open B-Rep shell"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_brep(id)?;
        let capped = editing::cap_planar(
            brep,
            1e-7,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("cap planar: {error}")))?;
        let source = self.document().stamp(id)?;
        self.commit_solid_result(
            feature,
            label(feature),
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Brep(capped),
            }],
            Some(vec![id]),
        )
    }

    fn solid_extract(&mut self, feature: &'static str, face: usize) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one B-Rep"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_brep(id)?;
        let extracted = editing::extract_face(
            brep,
            face,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("extract surface: {error}")))?;
        self.insert_solid(feature, label(feature), extracted)
    }

    fn solid_fillet(&mut self, feature: &'static str, edge: usize, radius: f64) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one B-Rep"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_brep(id)?;
        let filleted = editing::fillet(
            brep,
            &[edge],
            radius,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("fillet edge: {error}")))?;
        let source = self.document().stamp(id)?;
        self.commit_solid_result(
            feature,
            label(feature),
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Brep(filleted),
            }],
            Some(vec![id]),
        )
    }

    fn solid_extrude(&mut self, feature: &'static str, height: f64, all: bool) -> Result<()> {
        if self.selection.is_empty() {
            return Err(Error::Missing("select a closed planar curve"));
        }
        let ids: Vec<_> = if all {
            self.selection.iter().copied().collect()
        } else {
            vec![*self.selection.iter().next().expect("one selected")]
        };
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let direction = [normal[0] * height, normal[1] * height, normal[2] * height];
        let mut mutations = Vec::new();
        let mut inserted = Vec::new();
        let mut staged_usage = om9_core::budget::Usage::default();
        for id in ids {
            let Geometry::Curve(curve) = &self.document().entities[&id].geometry else {
                return Err(Error::Invalid("extrude needs a curve"));
            };
            let operation_limits = self.coordinator.limits().remaining_after(staged_usage)?;
            let brep = om9_kernel::extrusion::solid(
                curve,
                direction,
                creation::DEFAULT_DEFLECTION_MM,
                &operation_limits,
            )
            .map_err(|error| Error::Io(format!("extrude: {error}")))?;
            staged_usage.add(brep.validate(&operation_limits)?, self.coordinator.limits())?;
            let entity = Entity::new(
                label(feature),
                Geometry::Brep(brep),
                self.document().current_layer,
            );
            inserted.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_solid_result(feature, label(feature), mutations, Some(inserted))
    }

    fn solid_shell(&mut self, feature: &'static str, face: usize, thickness: f64) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one B-Rep"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_brep(id)?;
        let shell = editing::shell(
            brep,
            face,
            thickness,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("shell: {error}")))?;
        let source = self.document().stamp(id)?;
        self.commit_solid_result(
            feature,
            label(feature),
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Brep(shell),
            }],
            Some(vec![id]),
        )
    }
    fn solid_text(&mut self, feature: &'static str, text: &str, height: f64) -> Result<()> {
        if text.len() > self.coordinator.limits().max_name_bytes {
            return Err(Error::Budget("text bytes"));
        }
        if !height.is_finite() || height <= 0.0 || height > f32::MAX as f64 {
            return Err(Error::Invalid("text height must be finite and positive"));
        }
        use skrifa::{
            FontRef, MetadataProvider,
            instance::{LocationRef, Size},
            outline::DrawSettings,
        };
        let font = FontRef::new(epaint_default_fonts::UBUNTU_LIGHT)
            .map_err(|_| Error::Invalid("embedded font"))?;
        let outlines = font.outline_glyphs();
        let charmap = font.charmap();
        let glyph_metrics = font.glyph_metrics(Size::new(height as f32), LocationRef::default());
        let mut contours: Vec<Vec<om9_core::geometry::Point3>> = Vec::new();
        let mut pen_x = 0.0f64;
        for character in text.chars() {
            let Some(glyph_id) = charmap.map(character) else {
                continue;
            };
            let Some(glyph) = outlines.get(glyph_id) else {
                continue;
            };
            let mut pen = TextPen::default();
            glyph
                .draw(
                    DrawSettings::unhinted(Size::new(height as f32), LocationRef::default()),
                    &mut pen,
                )
                .map_err(|_| Error::Invalid("text outline"))?;
            if pen.current.len() > 2 {
                pen.contours.push(pen.current);
            }
            for mut contour in pen.contours {
                for point in &mut contour {
                    point[0] += pen_x;
                }
                contours.push(contour);
            }
            pen_x += glyph_metrics
                .advance_width(glyph_id)
                .unwrap_or(height as f32 * 0.5) as f64;
        }
        if contours.is_empty() {
            return Err(Error::Invalid("text produced no contour"));
        }
        let extrude = height * 0.2;
        let depth: Vec<usize> = contours
            .iter()
            .map(|contour| {
                contours
                    .iter()
                    .filter(|other| {
                        !std::ptr::eq(*other, contour)
                            && inside(contour[0], other)
                            && polygon_area(other).abs() > polygon_area(contour).abs()
                    })
                    .count()
            })
            .collect();
        let mut breps: Vec<om9_core::brep::Brep> = Vec::new();
        for (index, contour) in contours.iter().enumerate() {
            if depth[index] % 2 == 1 {
                continue;
            }
            let profile = Profile::new(contour.clone())
                .map_err(|error| Error::Io(format!("text profile: {error}")))?;
            let mut shape = creation::extrude(
                &profile,
                [0.0, 0.0, extrude],
                true,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("text extrude: {error}")))?;
            for (hole_index, hole) in contours.iter().enumerate() {
                if depth[hole_index] == depth[index] + 1 && inside(hole[0], contour) {
                    let profile = Profile::new(hole.clone())
                        .map_err(|error| Error::Io(format!("text hole: {error}")))?;
                    let cut = creation::extrude(
                        &profile,
                        [0.0, 0.0, extrude * 1.2],
                        true,
                        creation::DEFAULT_DEFLECTION_MM,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("text hole extrude: {error}")))?;
                    if let Some(result) = editing::boolean(
                        &shape,
                        &cut,
                        BooleanOp::Difference,
                        creation::DEFAULT_DEFLECTION_MM,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("text hole boolean: {error}")))?
                    .body()
                    {
                        shape = result.clone();
                    }
                }
            }
            breps.push(shape);
        }
        let mut result = breps.first().cloned().ok_or(Error::Invalid("text solid"))?;
        for brep in breps.iter().skip(1) {
            if let Some(joined) = editing::boolean(
                &result,
                brep,
                BooleanOp::Union,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("text glyph union: {error}")))?
            .body()
            {
                result = joined.clone();
            }
        }
        self.insert_solid(feature, label(feature), result)
    }
    fn solid_pipe(&mut self, feature: &'static str, radius: f64) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let Geometry::Curve(curve) = &self.document().entities[&id].geometry else {
            return Err(Error::Invalid("pipe needs a curve"));
        };
        let points = om9_core::surface_kernel::sample_curve(curve, 64, self.coordinator.limits())?;
        let brep = creation::pipe(
            &points,
            radius,
            true,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("pipe: {error}")))?;
        self.insert_solid(feature, label(feature), brep)
    }

    fn solid_tube(
        &mut self,
        feature: &'static str,
        outer: f64,
        inner: f64,
        height: f64,
    ) -> Result<()> {
        if inner <= 0.0 || outer <= inner {
            return Err(Error::Invalid("tube needs 0 < inner < outer"));
        }
        let limits = self.coordinator.limits();
        let outside = creation::primitive(
            Primitive::Cylinder {
                radius_mm: outer,
                height_mm: height,
            },
            creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|error| Error::Io(format!("tube outer: {error}")))?;
        let inside = creation::primitive(
            Primitive::Cylinder {
                radius_mm: inner,
                height_mm: height,
            },
            creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|error| Error::Io(format!("tube inner: {error}")))?;
        let outcome = editing::boolean(
            &outside,
            &inside,
            BooleanOp::Difference,
            creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|error| Error::Io(format!("tube boolean: {error}")))?;
        let Some(brep) = outcome.body() else {
            return Err(Error::Invalid("tube produced an empty body"));
        };
        self.insert_solid(feature, label(feature), brep.clone())
    }
}
