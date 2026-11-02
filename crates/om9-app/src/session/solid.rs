// SPDX-License-Identifier: LGPL-2.1-or-later
//! Solid menu command bindings (`OM9-L2-03-18`). The command ids are the feature ids
//! already present in `assets/features.json`; every enabled row dispatches to the OCCT
//! owner in `om9-kernel`, never to a second geometry implementation in the UI.
use super::*;
pub(super) use super::solid_pending::*;
pub(super) use super::solid_geometry::*;

pub const FEATURES: [&str; 22] = [
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
    "OM9-SOLID-021", // Boss
    "OM9-SOLID-029", // Shell
];

pub fn static_feature(feature: &str) -> &'static str {
    match feature {
        "SolidBooleanSplit" => "SolidBooleanSplit",
        "SolidCreateSolid" => "SolidCreateSolid",
        "SolidExtractSurfaceCopy" => "SolidExtractSurfaceCopy",
        "SolidPipeChain" => "SolidPipeChain",
        "SolidGVDMultiPipe" => "SolidGVDMultiPipe",
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
        "OM9-SOLID-021" => "OM9-SOLID-021",
        "OM9-SOLID-029" => "OM9-SOLID-029",
        _ => "OM9-SOLID-000",
    }
}

pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature)
        || matches!(
            feature,
            "SolidBooleanSplit"
                | "SolidCreateSolid"
                | "SolidExtractSurfaceCopy"
                | "SolidPipeChain"
                | "SolidGVDMultiPipe"
        )
}

#[allow(dead_code)]
pub fn label(feature: &str) -> &'static str {
    match feature {
        "SolidBooleanSplit" => "Boolean Split",
        "SolidCreateSolid" => "Create Solid",
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
        "OM9-SOLID-021" => "Boss",
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
        "booleansplit" | "solidbooleansplit" => "SolidBooleanSplit",
        "createsolid" | "solidcreatesolid" => "SolidCreateSolid",
        "cap" | "capplanar" => "OM9-SOLID-005",
        "extractsrf" | "extractsurface" => "OM9-SOLID-006",
        "filletedge" | "fillet" => "OM9-SOLID-007",
        "text" | "textobject" => "OM9-SOLID-008",
        "extrudecrv" | "extrude" => "OM9-SOLID-009",
        "extrudeall" => "OM9-SOLID-010",
        "pipe" => "OM9-SOLID-011",
        "pipechain" => "SolidPipeChain",
        "solidgvdmultipipe" => "SolidGVDMultiPipe",
        "box" | "_box" => "OM9-SOLID-012",
        "ellipsoid" | "_ellipsoid" => "OM9-SOLID-013",
        "sphere" | "_sphere" => "OM9-SOLID-014",
        "torus" | "_torus" => "OM9-SOLID-015",
        "cylinder" | "_cylinder" => "OM9-SOLID-016",
        "tube" | "_tube" => "OM9-SOLID-017",
        "pyramid" | "_pyramid" => "OM9-SOLID-018",
        "cone" | "_cone" => "OM9-SOLID-019",
        "tcone" | "_tcone" | "truncatedcone" => "OM9-SOLID-020",
        "boss" => "OM9-SOLID-021",
        "shell" => "OM9-SOLID-029",
        _ => return None,
    })
}

/// Right-click defaults from the FreeCAD Qt workbench: Box starts 3Point, Sphere
/// starts 2Point without changing the primary left-click behavior.
pub fn mode_command(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "Box3Point" => ("OM9-SOLID-012", "3Point"),
        "SolidBoxDiagonal" => ("OM9-SOLID-012", "Diagonal"),
        "SolidBox3PointsHeight" => ("OM9-SOLID-012", "3Point"),
        "SolidBoxVerticalBase" => ("OM9-SOLID-012", "Vertical"),
        "SolidBoxCenterofBaseCornerHeight" => ("OM9-SOLID-012", "Center"),
        "Sphere2Point" => ("OM9-SOLID-014", "2Point"),
        "SolidSphere2Points" => ("OM9-SOLID-014", "2Point"),
        "SolidSphere3Points" => ("OM9-SOLID-014", "3Point"),
        "SolidSphere4Points" => ("OM9-SOLID-014", "4Point"),
        "SolidSphereAroundCurve" => ("OM9-SOLID-014", "AroundCurve"),
        "SolidSphereTangenttoCurves" => ("OM9-SOLID-014", "Tangent"),
        "SolidSphereFitSpheretoPoints" => ("OM9-SOLID-014", "FitPoints"),
        "EllipsoidDiameter" => ("OM9-SOLID-013", "Diameter"),
        "EllipsoidFromFoci" => ("OM9-SOLID-013", "FromFoci"),
        "Torus2Point" => ("OM9-SOLID-015", "2Point"),
        "Cylinder2Point" => ("OM9-SOLID-016", "2Point"),
        "Tube2Point" => ("OM9-SOLID-017", "2Point"),
        "PyramidStar" => ("OM9-SOLID-018", "Star5"),
        "Cone2Point" => ("OM9-SOLID-019", "2Point"),
        "TruncatedCone2Point" => ("OM9-SOLID-020", "2Point"),
        _ => return None,
    })
}

pub fn usage(feature: &str) -> &'static str {
    match feature {
        "OM9-SOLID-003" => "select at least two closed B-Rep solids",
        "OM9-SOLID-001" | "OM9-SOLID-002" => "select exactly two closed B-Rep solids",
        "OM9-SOLID-004" => "select two B-Rep solids; run Boolean2Objects U|D|R|I|X",
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
use om9_kernel::{
    creation::{self, Primitive, Profile},
    editing::{self, BooleanOp},
};

#[cfg(feature = "occt-brep")]
impl Controller {
    pub(super) fn solid_preview_pipe_straight_body(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Option<Vec<[om9_core::geometry::Point3; 2]>> {
        let p = self.solid_pending.as_ref()?;
        let at_end = if p.pipe_thick {
            (3..=4).contains(&p.values.len())
        } else {
            (1..=2).contains(&p.values.len())
        };
        if p.feature != "OM9-SOLID-011"
            || p.pipe_multiple
            || p.pipe_chain
            || !p.pipe_stations.is_empty()
            || p.pipe_station_position.is_some()
            || !at_end
            || self.selection.len() != 1
        {
            return None;
        }
        let entity = self
            .document()
            .entities
            .get(self.selection.iter().next()?)?;
        let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &entity.geometry
        else {
            return None;
        };
        let start = entity.placement.point(*start).ok()?;
        let end = entity.placement.point(*end).ok()?;
        let axis = std::array::from_fn::<_, 3, _>(|i| end[i] - start[i]);
        let length = axis[0].hypot(axis[1]).hypot(axis[2]);
        if !length.is_finite() || length <= 1e-9 {
            return Some(Vec::new());
        }
        let n = axis.map(|v| v / length);
        let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - end[i]);
        let along = (0..3).map(|i| delta[i] * n[i]).sum::<f64>();
        let radial = std::array::from_fn::<_, 3, _>(|i| delta[i] - along * n[i]);
        let measured = radial[0].hypot(radial[1]).hypot(radial[2]);
        let top = if (!p.pipe_thick && p.values.len() == 2) || (p.pipe_thick && p.values.len() == 4)
        {
            *p.values.last()?
        } else {
            measured
        };
        if !top.is_finite() || top <= 1e-9 {
            return Some(Vec::new());
        }
        let profiles = if p.pipe_thick {
            let start = [p.values[0].min(p.values[1]), p.values[0].max(p.values[1])];
            let end = [p.values[2].min(top), p.values[2].max(top)];
            if end[0] <= 1e-9 || end[1] - end[0] <= 1e-9 || start[1] - start[0] <= 1e-9 {
                return Some(Vec::new());
            }
            [(start[0], end[0]), (start[1], end[1])]
        } else {
            [(p.values[0], top), (0., 0.)]
        };
        let seed = if measured.is_finite() && measured > 1e-9 {
            radial.map(|v| v / measured)
        } else {
            let reference = if n[0].abs() < 0.8 {
                [1., 0., 0.]
            } else {
                [0., 1., 0.]
            };
            let dot = (0..3).map(|i| reference[i] * n[i]).sum::<f64>();
            let v = std::array::from_fn::<_, 3, _>(|i| reference[i] - dot * n[i]);
            let norm = v[0].hypot(v[1]).hypot(v[2]);
            v.map(|x| x / norm)
        };
        let y = [
            n[1] * seed[2] - n[2] * seed[1],
            n[2] * seed[0] - n[0] * seed[2],
            n[0] * seed[1] - n[1] * seed[0],
        ];
        let count = (if p.pipe_round { 240 } else { 176 }) * if p.pipe_thick { 2 } else { 1 };
        let limits = self.coordinator.limits();
        if limits.max_points < (count * 2) as u64 {
            return Some(Vec::new());
        }
        let mut held = self.scene_cache_usage().ok()?;
        let mut usage =
            om9_core::budget::Usage::array::<[om9_core::geometry::Point3; 2]>(count).ok()?;
        usage.points = (count * 2) as u64;
        if held.add(usage, limits).is_err()
            || self
                .coordinator
                .retained_usage_with_held_usage(held)
                .is_err()
        {
            return Some(Vec::new());
        }
        let mut edges = Vec::new();
        if edges.try_reserve_exact(count).is_err() {
            return Some(Vec::new());
        }
        for &(base, tip) in &profiles[..if p.pipe_thick { 2 } else { 1 }] {
            let radius = |t: f64| {
                let u = if p.pipe_global {
                    t
                } else {
                    t * t * (3. - 2. * t)
                };
                base + u * (tip - base)
            };
            let at = |t: f64, angle: f64| {
                std::array::from_fn(|i| {
                    start[i]
                        + t * axis[i]
                        + radius(t) * (angle.cos() * seed[i] + angle.sin() * y[i])
                })
            };
            for t in [1., 0., 0.25, 0.5, 0.75] {
                for i in 0..32 {
                    edges.push([
                        at(t, std::f64::consts::TAU * i as f64 / 32.),
                        at(t, std::f64::consts::TAU * (i + 1) as f64 / 32.),
                    ]);
                }
            }
            for i in 0..4 {
                let a = std::f64::consts::FRAC_PI_2 * i as f64;
                for j in 0..4 {
                    edges.push([at(j as f64 / 4., a), at((j + 1) as f64 / 4., a)]);
                }
            }
            if p.pipe_round {
                for (center, radius, sign) in [(start, base, -1.), (end, tip, 1.)] {
                    for i in 0..4 {
                        let a = std::f64::consts::FRAC_PI_2 * i as f64;
                        let cap = |t: f64| {
                            std::array::from_fn(|k| {
                                center[k]
                                    + radius
                                        * (t.cos() * (a.cos() * seed[k] + a.sin() * y[k])
                                            + sign * t.sin() * n[k])
                            })
                        };
                        for j in 0..8 {
                            edges.push([
                                cap(std::f64::consts::FRAC_PI_2 * j as f64 / 8.),
                                cap(std::f64::consts::FRAC_PI_2 * (j + 1) as f64 / 8.),
                            ]);
                        }
                    }
                }
            }
        }
        Some(edges)
    }

    pub(super) fn solid_preview_pipe_section(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        let Some(p) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if p.feature != "OM9-SOLID-011"
            || p.pipe_multiple_selecting
            || (!p.pipe_chain
                && (self.selection.is_empty() || (!p.pipe_multiple && self.selection.len() != 1)))
        {
            return Vec::new();
        }
        let count = if p.pipe_thick { 4 } else { 2 };
        if p.values.len() >= count && p.pipe_station_position.is_none() {
            return Vec::new();
        }
        let limits = self.coordinator.limits();
        if limits.max_points < 128 {
            return Vec::new();
        }
        let Ok(mut held) = self.scene_cache_usage() else {
            return Vec::new();
        };
        let Ok(mut usage) = om9_core::budget::Usage::array::<[om9_core::geometry::Point3; 2]>(64)
        else {
            return Vec::new();
        };
        usage.points = 128;
        if held.add(usage, limits).is_err()
            || self
                .coordinator
                .retained_usage_with_held_usage(held)
                .is_err()
        {
            return Vec::new();
        }
        let (center, axis) = if p.pipe_chain {
            if let Some(t) = p.pipe_station_position {
                let Ok([start, end]) = self.solid_pipe_chain_endpoints(p) else {
                    return Vec::new();
                };
                (
                    std::array::from_fn(|i| start[i] + t * (end[i] - start[i])),
                    std::array::from_fn::<_, 3, _>(|i| end[i] - start[i]),
                )
            } else {
                let Ok(points) = self.solid_pipe_chain_points(p) else {
                    return Vec::new();
                };
                let at_end = p.values.len() >= if p.pipe_thick { 2 } else { 1 };
                let (a, b) = if at_end {
                    (points[points.len() - 2], points[points.len() - 1])
                } else {
                    (points[0], points[1])
                };
                (
                    if at_end { b } else { a },
                    std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]),
                )
            }
        } else {
            let Some(id) = self.selected_ordered().first().copied() else {
                return Vec::new();
            };
            let Some(entity) = self.document().entities.get(&id) else {
                return Vec::new();
            };
            let Geometry::Curve(curve) = &entity.geometry else {
                return Vec::new();
            };
            let fraction = p.pipe_station_position.unwrap_or_else(|| {
                if p.values.len() >= if p.pipe_thick { 2 } else { 1 } {
                    1.
                } else {
                    0.
                }
            });
            let Ok(local) =
                om9_core::curve_ops::point_at(curve, fraction, self.coordinator.limits())
            else {
                return Vec::new();
            };
            let Ok(center) = entity.placement.point(local) else {
                return Vec::new();
            };
            let Ok(tangent) =
                om9_core::curve_ops::tangent_at(curve, fraction, self.coordinator.limits())
            else {
                return Vec::new();
            };
            let a = entity.placement.0;
            (
                center,
                std::array::from_fn::<_, 3, _>(|i| {
                    (0..3).map(|j| a[i][j] * tangent[j]).sum::<f64>()
                }),
            )
        };
        let len = axis[0].hypot(axis[1]).hypot(axis[2]);
        if !len.is_finite() || len <= 1e-12 {
            return Vec::new();
        }
        let n = axis.map(|v| v / len);
        let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
        let axial = (0..3).map(|i| delta[i] * n[i]).sum::<f64>();
        let radial = std::array::from_fn::<_, 3, _>(|i| delta[i] - axial * n[i]);
        let radius = radial[0].hypot(radial[1]).hypot(radial[2]);
        if !radius.is_finite() || radius <= 1e-9 {
            return Vec::new();
        }
        let x = radial.map(|v| v / radius);
        let y = [
            n[1] * x[2] - n[2] * x[1],
            n[2] * x[0] - n[0] * x[2],
            n[0] * x[1] - n[1] * x[0],
        ];
        let at = |t: f64| {
            std::array::from_fn(|i| center[i] + radius * (t.cos() * x[i] + t.sin() * y[i]))
        };
        (0..64)
            .map(|i| {
                [
                    at(std::f64::consts::TAU * i as f64 / 64.),
                    at(std::f64::consts::TAU * (i + 1) as f64 / 64.),
                ]
            })
            .collect()
    }

    pub(super) fn solid_preview_cone(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        let Some(p) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if !matches!(
            p.feature,
            "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
        ) || p.points.is_empty()
        {
            return Vec::new();
        }
        if p.cone_plane
            .as_deref()
            .and_then(|f| f.around_source)
            .is_some_and(|stamp| self.document().stamp(stamp.object).ok() != Some(stamp))
        {
            return Vec::new();
        }
        if p.cone_base_points == 4 && p.values.is_empty() {
            if p.points.len() as u64 >= self.coordinator.limits().max_points {
                return Vec::new();
            }
            let Some(count) = p.points.len().checked_add(1) else {
                return Vec::new();
            };
            let mut points = Vec::new();
            if points.try_reserve_exact(count).is_err() {
                return Vec::new();
            }
            points.extend_from_slice(&p.points);
            points.push(point);
            let Ok((center, x, y, radius)) = super::circle::fit_circle(&points) else {
                return Vec::new();
            };
            let at = |t: f64| {
                std::array::from_fn(|i| center[i] + radius * (t.cos() * x[i] + t.sin() * y[i]))
            };
            return (0..64)
                .map(|i| {
                    [
                        at(std::f64::consts::TAU * i as f64 / 64.),
                        at(std::f64::consts::TAU * (i + 1) as f64 / 64.),
                    ]
                })
                .collect();
        }
        let Some(frame) = p.cone_plane.as_deref() else {
            return Vec::new();
        };
        if p.values.is_empty() && p.cone_base_points == 3 && p.points.len() == 1 {
            return p
                .cone_project_base_point(point)
                .map_or_else(|_| Vec::new(), |b| vec![[p.points[0], b]]);
        }
        if p.values.is_empty() && p.cone_base_points == 6 && !p.cone_flags.point_mode() {
            return Vec::new();
        }
        if p.values.is_empty() && p.cone_flags.radius_phase() == 1 {
            return p
                .cone_project_base_point(point)
                .map_or_else(|_| Vec::new(), |b| vec![[p.points[0], b]]);
        }
        let (center, bottom) =
            if p.values.is_empty() && p.cone_base_points == 6 && p.cone_flags.point_mode() {
                let Ok(pair) = p.cone_tangent_point_base(point) else {
                    return Vec::new();
                };
                pair
            } else if p.values.is_empty() && p.cone_flags.radius_phase() == 2 {
                let Ok(pair) = p.cone_radius_base(point) else {
                    return Vec::new();
                };
                pair
            } else if p.values.is_empty() && p.cone_base_points == 3 {
                let Ok(pair) = p.cone_three_point_base(point) else {
                    return Vec::new();
                };
                pair
            } else if p.values.is_empty() && p.cone_base_points == 2 {
                let Ok(pair) = p.cone_diameter_base(point) else {
                    return Vec::new();
                };
                pair
            } else if p.values.is_empty() {
                let Ok(radius) = p.cone_point_measure(point) else {
                    return Vec::new();
                };
                (p.cone_center(), radius)
            } else {
                (p.cone_center(), p.values[0])
            };
        if !bottom.is_finite() || bottom <= 1e-9 {
            return Vec::new();
        }
        let normal = frame.normal();
        let mut edges = Vec::with_capacity(132);
        let ring = |c: [f64; 3], radius: f64, t: f64| {
            std::array::from_fn(|i| c[i] + radius * (t.cos() * frame.x[i] + t.sin() * frame.y[i]))
        };
        for i in 0..64 {
            let a = std::f64::consts::TAU * i as f64 / 64.;
            let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
            edges.push([ring(center, bottom, a), ring(center, bottom, b)]);
        }
        if p.feature == "OM9-SOLID-017" {
            if p.values.is_empty() {
                return edges;
            }
            let second = if p.values.len() == 1 {
                p.cone_point_measure(point).unwrap_or(0.)
            } else {
                p.values[1]
            };
            if !second.is_finite() || second <= 1e-9 || (second - bottom).abs() <= 1e-9 {
                return edges;
            }
            for i in 0..64 {
                let a = std::f64::consts::TAU * i as f64 / 64.;
                let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
                edges.push([ring(center, second, a), ring(center, second, b)]);
            }
            if p.values.len() == 2 {
                let Ok(h) = p.cone_point_measure(point) else {
                    return edges;
                };
                if h.is_finite() && h.abs() > 1e-9 {
                    let end = std::array::from_fn(|i| center[i] + h * normal[i]);
                    let start = if p.cone_flags.both_sides() {
                        edges.clear();
                        let start = std::array::from_fn(|i| center[i] - h * normal[i]);
                        for radius in [bottom, second] {
                            for i in 0..64 {
                                let a = std::f64::consts::TAU * i as f64 / 64.;
                                let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
                                edges.push([ring(start, radius, a), ring(start, radius, b)]);
                            }
                        }
                        start
                    } else {
                        center
                    };
                    for radius in [bottom, second] {
                        for i in 0..64 {
                            let a = std::f64::consts::TAU * i as f64 / 64.;
                            let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
                            edges.push([ring(end, radius, a), ring(end, radius, b)]);
                        }
                        for i in 0..4 {
                            let a = std::f64::consts::FRAC_PI_2 * i as f64;
                            edges.push([ring(start, radius, a), ring(end, radius, a)]);
                        }
                    }
                }
            }
            return edges;
        }
        if p.values.len() == 1 {
            if let Ok(h) = p.cone_point_measure(point) {
                if h.is_finite() && h.abs() > 1e-9 {
                    let end = std::array::from_fn(|i| center[i] + h * normal[i]);
                    if p.feature == "OM9-SOLID-016" {
                        for i in 0..64 {
                            let a = std::f64::consts::TAU * i as f64 / 64.;
                            let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
                            edges.push([ring(end, bottom, a), ring(end, bottom, b)]);
                        }
                        for i in 0..4 {
                            let a = std::f64::consts::FRAC_PI_2 * i as f64;
                            edges.push([ring(center, bottom, a), ring(end, bottom, a)]);
                        }
                    } else if p.feature == "OM9-SOLID-019" {
                        for i in 0..4 {
                            let a = std::f64::consts::FRAC_PI_2 * i as f64;
                            edges.push([ring(center, bottom, a), end]);
                        }
                    } else {
                        edges.push([center, end]);
                    }
                }
            }
        } else if p.values.len() >= 2 {
            let Ok(top) = p.cone_point_measure(point) else {
                return edges;
            };
            if top.is_finite() && top > 1e-9 {
                let end = std::array::from_fn(|i| center[i] + p.values[1] * normal[i]);
                for i in 0..64 {
                    let a = std::f64::consts::TAU * i as f64 / 64.;
                    let b = std::f64::consts::TAU * (i + 1) as f64 / 64.;
                    edges.push([ring(end, top, a), ring(end, top, b)]);
                }
                for i in 0..4 {
                    let a = std::f64::consts::FRAC_PI_2 * i as f64;
                    edges.push([ring(center, bottom, a), ring(end, top, a)]);
                }
            }
        }
        edges
    }
    pub(super) fn solid_preview_torus(&self, point: [f64; 3]) -> Vec<[[f64; 3]; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if let Some(stamp) = pending.cone_plane.as_deref().and_then(|f| f.around_source) {
            if self.document().stamp(stamp.object).ok().as_ref() != Some(&stamp) {
                return Vec::new();
            }
        }
        let Ok((center, plane, ring, tube)) = pending.torus_tube_geometry(point) else {
            return Vec::new();
        };
        let n = plane.normal();
        let mut edges = Vec::with_capacity(256);
        let radial = |angle: f64| {
            std::array::from_fn::<_, 3, _>(|i| angle.cos() * plane.x[i] + angle.sin() * plane.y[i])
        };
        for radius in [ring - tube, ring + tube] {
            let at = |t: f64| {
                let direction = radial(t);
                std::array::from_fn(|i| center[i] + radius * direction[i])
            };
            for i in 0..64 {
                edges.push([
                    at(std::f64::consts::TAU * i as f64 / 64.),
                    at(std::f64::consts::TAU * (i + 1) as f64 / 64.),
                ]);
            }
        }
        for index in 0..4 {
            let r = radial(std::f64::consts::FRAC_PI_2 * index as f64);
            let at = |t: f64| {
                std::array::from_fn(|i| {
                    center[i] + (ring + tube * t.cos()) * r[i] + tube * t.sin() * n[i]
                })
            };
            for i in 0..32 {
                edges.push([
                    at(std::f64::consts::TAU * i as f64 / 32.),
                    at(std::f64::consts::TAU * (i + 1) as f64 / 32.),
                ]);
            }
        }
        edges
    }
    pub(super) fn solid_preview_ellipsoid(&self, point: [f64; 3]) -> Vec<[[f64; 3]; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if let Some(stamp) = pending.cone_plane.as_deref().and_then(|f| f.around_source) {
            if self.document().stamp(stamp.object).ok().as_ref() != Some(&stamp) {
                return Vec::new();
            }
        }
        let Ok((center, frame, radii)) = pending.ellipsoid_at_height(point) else {
            return Vec::new();
        };
        let mut edges = Vec::with_capacity(192);
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            let at = |angle: f64| {
                std::array::from_fn(|i| {
                    center[i]
                        + radii[a] * frame[a][i] * angle.cos()
                        + radii[b] * frame[b][i] * angle.sin()
                })
            };
            for i in 0..64 {
                edges.push([
                    at(std::f64::consts::TAU * i as f64 / 64.),
                    at(std::f64::consts::TAU * (i + 1) as f64 / 64.),
                ]);
            }
        }
        edges
    }
    pub(super) fn solid_preview_sphere(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if pending.sphere_sources.iter().any(|stamp| self.document().stamp(stamp.object).ok().as_ref() != Some(stamp)) {
            return Vec::new();
        }
        let (center, radius) = if let Some(center) = pending.sphere_center
            && matches!(pending.sphere_kind, "Center" | "Vertical" | "AroundCurve")
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
            if pending.sphere_radius < 0.0 {
                return Vec::new();
            }
            if pending.sphere_radius > 0.0 {
                match sphere_chord_direction(
                    pending.points[0],
                    pending.points[1],
                    pending.sphere_radius,
                    point,
                ) {
                    Ok(center) => (center, pending.sphere_radius),
                    Err(_) => return Vec::new(),
                }
            } else {
                match Self::sphere_circumcenter(pending.points[0], pending.points[1], point) {
                    Ok((center, radius, _)) => (center, radius),
                    Err(_) => return Vec::new(),
                }
            }
        } else if pending.sphere_kind == "FourPoint" && pending.points.len() == 3 {
            let Some(center) = pending.sphere_center else {
                return Vec::new();
            };
            match Self::sphere_fourth_point(
                center,
                pending.sphere_radius,
                pending.sphere_normal,
                point,
            ) {
                Ok(sphere) => sphere,
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
                        0 => [
                            center[0],
                            center[1] + radius * cos,
                            center[2] + radius * sin,
                        ],
                        1 => [
                            center[0] + radius * cos,
                            center[1],
                            center[2] + radius * sin,
                        ],
                        _ => [
                            center[0] + radius * cos,
                            center[1] + radius * sin,
                            center[2],
                        ],
                    }
                };
                edges.push([point_at(first), point_at(second)]);
            }
        }
        edges
    }

    fn sphere_fourth_point(
        center: om9_core::geometry::Point3,
        radius: f64,
        normal: om9_core::geometry::Point3,
        point: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64)> {
        let delta = [
            point[0] - center[0],
            point[1] - center[1],
            point[2] - center[2],
        ];
        let height = delta[0] * normal[0] + delta[1] * normal[1] + delta[2] * normal[2];
        if height.abs() <= 1e-12 {
            return Err(Error::Invalid(
                "fourth point must be outside the circle plane",
            ));
        }
        let length_squared = delta.iter().map(|x| x * x).sum::<f64>();
        let offset = (length_squared - radius * radius) / (2.0 * height);
        let result = [
            center[0] + normal[0] * offset,
            center[1] + normal[1] * offset,
            center[2] + normal[2] * offset,
        ];
        let result_radius = (radius * radius + offset * offset).sqrt();
        if !result.iter().all(|x| x.is_finite()) || !result_radius.is_finite() {
            return Err(Error::Invalid("sphere must have finite dimensions"));
        }
        Ok((result, result_radius))
    }

    pub(super) fn solid_preview_pyramid(&self, point: [f64; 3]) -> Vec<[[f64; 3]; 2]> {
        let Some(p) = self.solid_pending.as_ref() else {
            return Vec::new();
        };
        if p.cone_plane
            .as_deref()
            .and_then(|f| f.around_source)
            .is_some_and(|stamp| self.document().stamp(stamp.object).ok().as_ref() != Some(&stamp))
        {
            return Vec::new();
        }
        let (Some(first), Some(stored)) = (p.points.first().copied(), p.cone_plane.as_deref())
        else {
            return Vec::new();
        };
        let mut center = if p.values.is_empty() {
            first
        } else {
            p.pyramid_center()
        };
        let mut frame = stored.plane;
        let mut edge_radius = None;
        if matches!(p.cone_base_points, 1 | 2) && p.values.is_empty() {
            let Ok((c, _, f, r)) = pyramid_edge_geometry(
                frame,
                first,
                point,
                p.pyramid_sides,
                p.cone_base_points == 2,
            ) else {
                return Vec::new();
            };
            center = c;
            frame = f;
            edge_radius = Some(r);
        }
        let normal = frame.normal();
        let d = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
        let dot = |a: [f64; 3]| (0..3).map(|i| d[i] * a[i]).sum::<f64>();
        let (mut x, mut y, mut radius) =
            if let Some(radius) = p.values.first().copied().or(edge_radius) {
                (frame.x, frame.y, radius)
            } else {
                let u = dot(frame.x);
                let v = dot(frame.y);
                let r = u.hypot(v);
                if !r.is_finite() || r <= 1e-9 {
                    return Vec::new();
                }
                let x = std::array::from_fn(|i| (u * frame.x[i] + v * frame.y[i]) / r);
                let y = [
                    normal[1] * x[2] - normal[2] * x[1],
                    normal[2] * x[0] - normal[0] * x[2],
                    normal[0] * x[1] - normal[1] * x[0],
                ];
                (x, y, r)
            };
        let sides = p.pyramid_sides as usize;
        let vertices = if p.cone_base_points == 3 {
            sides.saturating_mul(2)
        } else {
            sides
        };
        if vertices as u64 > self.coordinator.limits().max_points
            || vertices as u64 * 512 > self.coordinator.limits().max_bytes
            || (p.cone_base_points == 3 && vertices > 4096)
        {
            return Vec::new();
        }
        if p.cone_base_points == 0 && p.cone_flags.pyramid_circumscribed() {
            let angle = std::f64::consts::PI / sides as f64;
            let (s, c) = angle.sin_cos();
            let old_x = x;
            let old_y = y;
            x = std::array::from_fn(|i| c * old_x[i] - s * old_y[i]);
            y = std::array::from_fn(|i| s * old_x[i] + c * old_y[i]);
            radius /= c;
        }
        if !radius.is_finite() {
            return Vec::new();
        }
        let mut edges = Vec::new();
        if edges.try_reserve_exact(vertices.saturating_mul(2)).is_err() {
            return Vec::new();
        }
        let second = if p.cone_base_points == 3 {
            if p.values.len() >= 2 {
                p.values[1]
            } else if p.values.len() == 1 {
                dot(frame.x).hypot(dot(frame.y))
            } else {
                radius
            }
        } else {
            radius
        };
        if !second.is_finite() || second <= 1e-9 {
            return Vec::new();
        }
        let at = |i: usize| {
            let a = std::f64::consts::TAU * i as f64 / vertices as f64;
            let r = if p.cone_base_points == 3 && i % 2 == 1 {
                second
            } else {
                radius
            };
            std::array::from_fn(|j| center[j] + r * (a.cos() * x[j] + a.sin() * y[j]))
        };
        let h = if p.values.len() < if p.cone_base_points == 3 { 2 } else { 1 } {
            0.
        } else {
            dot(normal)
        };
        let apex = std::array::from_fn(|i| center[i] + h * normal[i]);
        for i in 0..vertices {
            let a = at(i);
            edges.push([a, at((i + 1) % vertices)]);
            if h.is_finite() && h.abs() > 1e-9 {
                edges.push([a, apex]);
            }
        }
        edges
    }

    pub(super) fn solid_preview_box(
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
        let frame = pending.box_plane.unwrap_or_else(|| self.input_plane());
        let axes = [frame.x, frame.y, frame.normal()];
        let dot = |delta: [f64; 3], axis: [f64; 3]| (0..3).map(|i| delta[i] * axis[i]).sum::<f64>();
        let to_world = |local: [f64; 3]| {
            std::array::from_fn(|i| first[i] + (0..3).map(|j| local[j] * axes[j][i]).sum::<f64>())
        };
        if pending.box_kind == "Center"
            && pending.points.len() == 1
            && pending.box_length > 0.0
            && let Some(width) = pending.box_width
        {
            let height = dot(std::array::from_fn(|i| point[i] - first[i]), axes[2]);
            if !height.is_finite() || height.abs() <= 1e-9 {
                return Vec::new();
            }
            return box_wire_edges(
                [pending.box_length, width, height.abs()],
                axes,
                to_world([-pending.box_length * 0.5, -width * 0.5, height.min(0.0)]),
            );
        }
        if pending.points.len() >= 2
            && matches!(pending.box_kind, "Corners" | "Diagonal" | "Center")
        {
            let second = pending.points[1];
            let height = dot(std::array::from_fn(|i| point[i] - first[i]), axes[2]);
            let delta = std::array::from_fn(|i| second[i] - first[i]);
            let x = dot(delta, axes[0]);
            let y = dot(delta, axes[1]);
            let dx = x.abs();
            let dy = y.abs();
            if !height.is_finite() || height.abs() <= 1e-9 || dx <= 1e-9 || dy <= 1e-9 {
                return Vec::new();
            }
            let center = pending.box_kind == "Center";
            let factor = if center { 2.0 } else { 1.0 };
            let origin = if center {
                to_world([-dx, -dy, height.min(0.0)])
            } else {
                to_world([x.min(0.0), y.min(0.0), height.min(0.0)])
            };
            return box_wire_edges([factor * dx, factor * dy, height.abs()], axes, origin);
        }
        if pending.points.len() == 1
            && matches!(pending.box_kind, "Corners" | "Diagonal" | "Center")
        {
            let delta = std::array::from_fn(|i| point[i] - first[i]);
            let x = dot(delta, axes[0]);
            let y = dot(delta, axes[1]);
            let factor = if pending.box_kind == "Center" {
                2.0
            } else {
                1.0
            };
            if x.abs() <= 1e-9 || y.abs() <= 1e-9 {
                return Vec::new();
            }
            let origin = if pending.box_kind == "Center" {
                to_world([-x.abs(), -y.abs(), 0.])
            } else {
                to_world([x.min(0.), y.min(0.), 0.])
            };
            return box_wire_edges(
                [factor * x.abs(), factor * y.abs(), factor * y.abs()],
                axes,
                origin,
            );
        }
        if pending.points.len() >= 2 && matches!(pending.box_kind, "ThreePoint" | "Vertical") {
            let Ok((length, box_axes)) = box_edge_axes(
                first,
                pending.points[1],
                frame,
                pending.box_kind == "Vertical",
            ) else {
                return Vec::new();
            };
            let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - first[i]);
            let width = pending.box_width.unwrap_or_else(|| dot(delta, box_axes[1]));
            let height = if pending.box_width.is_some() {
                dot(delta, box_axes[2])
            } else {
                width.abs()
            };
            if !width.is_finite()
                || !height.is_finite()
                || width.abs() <= 1e-9
                || height.abs() <= 1e-9
            {
                return Vec::new();
            }
            let origin = std::array::from_fn(|i| {
                first[i] + box_axes[1][i] * width.min(0.) + box_axes[2][i] * height.min(0.)
            });
            return box_wire_edges([length, width.abs(), height.abs()], box_axes, origin);
        }
        if pending.points.len() == 1 && matches!(pending.box_kind, "ThreePoint" | "Vertical") {
            let Ok((length, box_axes)) =
                box_edge_axes(first, point, frame, pending.box_kind == "Vertical")
            else {
                return Vec::new();
            };
            let end = std::array::from_fn(|i| first[i] + length * box_axes[0][i]);
            return vec![[first, end]];
        }
        if pending.box_kind == "Cube" {
            let target = pending.points.get(1).copied().unwrap_or(point);
            let delta = std::array::from_fn(|i| target[i] - first[i]);
            let Ok((cube_frame, side)) = cube_axes(axes, delta) else {
                return Vec::new();
            };
            return box_wire_edges([side; 3], cube_frame, first);
        }
        let delta = [
            point[0] - first[0],
            point[1] - first[1],
            point[2] - first[2],
        ];
        let (dims, axes, origin) = if pending.box_kind == "Cube" {
            let Ok((axes, side)) =
                cube_axes([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], delta)
            else {
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
            (
                dims,
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                origin,
            )
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
        ];
        edges
            .iter()
            .map(|(a, b)| [corners[*a], corners[*b]])
            .collect()
    }

    pub(super) fn solid_brep(&self, id: om9_core::identity::ObjectId) -> Result<&om9_core::brep::Brep> {
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

    pub(super) fn insert_solid(
        &mut self,
        feature: &'static str,
        name: &'static str,
        brep: om9_core::brep::Brep,
    ) -> Result<()> {
        let entity = Entity::new(name, Geometry::Brep(brep), self.document().current_layer);
        self.commit_entity(feature, name, entity)
    }
}


impl Controller {
    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_boss_transform_curve(
        curve: &om9_core::geometry::Curve,
        transform: om9_core::geometry::Transform,
    ) -> Result<om9_core::geometry::Curve> {
        use om9_core::geometry::Curve;
        Ok(match curve {
            Curve::Line { start, end, domain } => Curve::Line {
                start: transform.point(*start)?,
                end: transform.point(*end)?,
                domain: *domain,
            },
            Curve::Nurbs(n) => {
                let mut out = n.clone();
                for p in &mut out.poles {
                    *p = transform.point(*p)?;
                }
                Curve::Nurbs(out)
            }
            Curve::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => {
                let origin = transform.point(*center)?;
                let axis = |v: [f64; 3]| -> Result<[f64; 3]> {
                    let p = transform.point(std::array::from_fn(|i| center[i] + v[i]))?;
                    Ok(std::array::from_fn(|i| p[i] - origin[i]))
                };
                let mut u = axis(*x)?;
                let mut v = axis(*y)?;
                let norm = |p: [f64; 3]| p.iter().map(|a| a * a).sum::<f64>().sqrt();
                let (su, sv) = (norm(u), norm(v));
                if !su.is_finite()
                    || !sv.is_finite()
                    || su <= 0.0
                    || sv <= 0.0
                    || (su - sv).abs() > su.max(sv) * 1e-9
                {
                    return Err(Error::Unsupported(
                        "Boss arc placement must preserve circularity",
                    ));
                }
                u = u.map(|a| a / su);
                v = v.map(|a| a / sv);
                if (0..3).map(|i| u[i] * v[i]).sum::<f64>().abs() > 1e-9 {
                    return Err(Error::Unsupported(
                        "Boss arc placement must preserve circularity",
                    ));
                }
                Curve::Circle {
                    center: origin,
                    x: u,
                    y: v,
                    radius: radius * su,
                    domain: *domain,
                    angle_domain: *angle_domain,
                }
            }
            Curve::Polycurve { segments, reversed } => {
                let mut out = Vec::new();
                out.try_reserve_exact(segments.len())
                    .map_err(|_| Error::Budget("Boss curve segments"))?;
                for segment in segments {
                    out.push(Self::solid_boss_transform_curve(segment, transform)?);
                }
                Curve::Polycurve {
                    segments: out,
                    reversed: *reversed,
                }
            }
        })
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_boss_profile(
        &self,
        id: om9_core::identity::ObjectId,
        frame: om9_core::geometry::Transform,
    ) -> Result<editing::BossSection> {
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("Boss profile"))?;
        let transform = frame.compose(entity.placement)?;
        if let Geometry::Curve(curve @ om9_core::geometry::Curve::Polycurve { segments, .. }) =
            &entity.geometry
        {
            if segments
                .iter()
                .any(|segment| !matches!(segment, om9_core::geometry::Curve::Line { .. }))
            {
                curve.validate(self.coordinator.limits())?;
                let transformed = Self::solid_boss_transform_curve(curve, transform)?;
                om9_kernel::extrusion::planar_face(
                    &transformed,
                    creation::DEFAULT_DEFLECTION_MM,
                    self.coordinator.limits(),
                )
                .map_err(|e| Error::Io(format!("Boss profile: {e}")))?;
                return Ok(editing::BossSection::Curve(transformed));
            }
        }
        if let Geometry::Curve(om9_core::geometry::Curve::Nurbs(nurbs)) = &entity.geometry {
            let mut curve = nurbs.clone();
            for point in &mut curve.poles {
                *point = transform.point(*point)?;
            }
            let curve = om9_core::geometry::Curve::Nurbs(curve);
            om9_kernel::extrusion::planar_face(
                &curve,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(format!("Boss profile: {e}")))?;
            return Ok(editing::BossSection::Curve(curve));
        }
        if let Geometry::Curve(om9_core::geometry::Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        }) = &entity.geometry
        {
            let angles = angle_domain.unwrap_or(*domain);
            if ((angles[1] - angles[0]) - std::f64::consts::TAU).abs() > 1e-9 {
                return Err(Error::Invalid("Boss circle must be closed"));
            }
            let origin = transform.point(*center)?;
            let axis = |v: [f64; 3]| -> Result<[f64; 3]> {
                let p = transform.point(std::array::from_fn(|i| center[i] + v[i]))?;
                Ok(std::array::from_fn(|i| p[i] - origin[i]))
            };
            let mut u = axis(*x)?;
            let mut v = axis(*y)?;
            let norm = |p: [f64; 3]| p.iter().map(|n| n * n).sum::<f64>().sqrt();
            let (su, sv) = (norm(u), norm(v));
            if !su.is_finite()
                || !sv.is_finite()
                || su <= 0.0
                || sv <= 0.0
                || (su - sv).abs() > su.max(sv) * 1e-9
            {
                return Err(Error::Unsupported(
                    "Boss circle placement must preserve circularity",
                ));
            }
            u = u.map(|n| n / su);
            v = v.map(|n| n / sv);
            if (0..3).map(|i| u[i] * v[i]).sum::<f64>().abs() > 1e-9 {
                return Err(Error::Unsupported(
                    "Boss circle placement must preserve circularity",
                ));
            }
            let circle = om9_core::curve_build::circle_on_frame(origin, u, v, radius * su)?;
            return Ok(editing::BossSection::Circle(circle));
        }
        let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, reversed }) =
            &entity.geometry
        else {
            return Err(Error::Unsupported(
                "Boss currently requires a closed polygon, circle or planar NURBS",
            ));
        };
        if segments.len() < 3 || segments.len() as u64 > self.coordinator.limits().max_points {
            return Err(Error::Invalid("Boss polygon segment count"));
        }
        let mut points: Vec<om9_core::geometry::Point3> = Vec::new();
        points
            .try_reserve_exact(segments.len() + 1)
            .map_err(|_| Error::Budget("Boss polygon"))?;
        for i in 0..segments.len() {
            let index = if *reversed { segments.len() - 1 - i } else { i };
            let om9_core::geometry::Curve::Line { start, end, .. } = &segments[index] else {
                return Err(Error::Unsupported(
                    "Boss profile must contain only line segments",
                ));
            };
            let (a, b) = if *reversed {
                (*end, *start)
            } else {
                (*start, *end)
            };
            let (a, b) = (transform.point(a)?, transform.point(b)?);
            if let Some(last) = points.last() {
                let d = (0..3).map(|j| (last[j] - a[j]).powi(2)).sum::<f64>().sqrt();
                if d > 1e-6 {
                    return Err(Error::Invalid("Boss polygon segments are disconnected"));
                }
            } else {
                points.push(a);
            }
            points.push(b);
        }
        let first = points[0];
        let last = *points.last().expect("polygon points");
        if (0..3)
            .map(|j| (last[j] - first[j]).powi(2))
            .sum::<f64>()
            .sqrt()
            > 1e-6
        {
            return Err(Error::Invalid("Boss polygon is open"));
        }
        points.pop();
        Profile::new(points)
            .map(editing::BossSection::Polygon)
            .map_err(|e| Error::Io(format!("Boss profile: {e}")))
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_boss_validate_profile(&self, id: om9_core::identity::ObjectId) -> Result<()> {
        self.solid_boss_profile(id, om9_core::geometry::Transform::identity())
            .map(|_| ())
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_boss_validate_profile(&self, _id: om9_core::identity::ObjectId) -> Result<()> {
        Err(Error::Unsupported("Boss requires OCCT"))
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_boss_commit(
        &mut self,
        profiles: &[om9_core::identity::SourceStamp],
        boundary: om9_core::identity::ObjectId,
    ) -> Result<()> {
        let source = self.document().stamp(boundary)?;
        let placement = self.document().entities[&boundary].placement;
        let a = placement.0;
        let axes = [
            [a[0][0], a[1][0], a[2][0]],
            [a[0][1], a[1][1], a[2][1]],
            [a[0][2], a[1][2], a[2][2]],
        ];
        let dot = |x: [f64; 3], y: [f64; 3]| (0..3).map(|i| x[i] * y[i]).sum::<f64>();
        let scale = dot(axes[0], axes[0]);
        if !scale.is_finite()
            || scale <= 0.0
            || (dot(axes[1], axes[1]) - scale).abs() > scale * 1e-9
            || (dot(axes[2], axes[2]) - scale).abs() > scale * 1e-9
            || dot(axes[0], axes[1]).abs() > scale * 1e-9
            || dot(axes[0], axes[2]).abs() > scale * 1e-9
            || dot(axes[1], axes[2]).abs() > scale * 1e-9
        {
            return Err(Error::Unsupported(
                "Boss boundary placement must preserve angles and scale",
            ));
        }
        let inverse = om9_core::top11::inverse_affine(placement)?;
        let mut local_profiles = Vec::new();
        local_profiles
            .try_reserve_exact(profiles.len())
            .map_err(|_| Error::Budget("Boss profiles"))?;
        for source in profiles {
            self.document().resolve(source)?;
            local_profiles.push(self.solid_boss_profile(source.object, inverse)?);
        }
        let boundary_brep = self.solid_brep(boundary)?;
        let local = match editing::boss_parallel_planar_sections(
            &local_profiles,
            boundary_brep,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        ) {
            Ok(body) => body,
            Err(planar_error) => editing::boss_cylindrical_sections(
                &local_profiles,
                boundary_brep,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|cylinder_error| {
                Error::Io(format!("Boss: {planar_error}; cylinder: {cylinder_error}"))
            })?,
        };
        self.commit_solid_result(
            "OM9-SOLID-021",
            "Boss",
            vec![Mutation::Geometry {
                source,
                geometry: Geometry::Brep(local),
            }],
            Some(vec![boundary]),
        )
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_boss_commit(
        &mut self,
        _profiles: &[om9_core::identity::SourceStamp],
        _boundary: om9_core::identity::ObjectId,
    ) -> Result<()> {
        Err(Error::Unsupported("Boss requires OCCT"))
    }

    /// Development-only straight Rib publication. Public Rib remains disabled.
    #[cfg(feature = "occt-brep")]
    #[doc(hidden)]
    pub fn solid_rib_commit_straight(
        &mut self, source: om9_core::identity::SourceStamp, boundary: om9_core::identity::SourceStamp,
        normal: [f64;3], distance: f64,
    ) -> Result<()> {
        self.solid_rib_commit_straight_many(&[(source,normal)],boundary,distance)
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_rib_commit_straight_many(
        &mut self, sources: &[(om9_core::identity::SourceStamp,[f64;3])],
        boundary: om9_core::identity::SourceStamp, distance: f64,
    ) -> Result<()> {
        if sources.is_empty() || sources.len() as u64 > self.coordinator.limits().max_outputs {return Err(Error::Invalid("Rib needs bounded curve selection"));}
        self.document().resolve(&boundary)?;
        let placement=self.document().entities.get(&boundary.object).ok_or(Error::Stale("Rib boundary"))?.placement;
        let m=placement.0;let axes=[[m[0][0],m[1][0],m[2][0]],[m[0][1],m[1][1],m[2][1]],[m[0][2],m[1][2],m[2][2]]];
        let dot=|a:[f64;3],b:[f64;3]|(0..3).map(|i|a[i]*b[i]).sum::<f64>();let scale2=dot(axes[0],axes[0]);
        if !scale2.is_finite() || scale2<=0. || (dot(axes[1],axes[1])-scale2).abs()>scale2*1e-9
            || (dot(axes[2],axes[2])-scale2).abs()>scale2*1e-9 || dot(axes[0],axes[1]).abs()>scale2*1e-9
            || dot(axes[0],axes[2]).abs()>scale2*1e-9 || dot(axes[1],axes[2]).abs()>scale2*1e-9 {
            return Err(Error::Unsupported("Rib boundary placement must preserve angles and scale"));
        }
        let inverse=om9_core::top11::inverse_affine(placement)?;
        let det=axes[0][0]*(axes[1][1]*axes[2][2]-axes[1][2]*axes[2][1])
            -axes[1][0]*(axes[0][1]*axes[2][2]-axes[0][2]*axes[2][1])
            +axes[2][0]*(axes[0][1]*axes[1][2]-axes[0][2]*axes[1][1]);
        let local_distance=distance/scale2.sqrt()*det.signum();
        let mut curves=Vec::new();
        curves.try_reserve_exact(sources.len()).map_err(|_|Error::Budget("Rib curves"))?;
        for &(source,normal) in sources {
            self.document().resolve(&source)?;
            if source.object==boundary.object {return Err(Error::Invalid("Rib source and boundary must differ"));}
            let entity=self.document().entities.get(&source.object).ok_or(Error::Stale("Rib source"))?;
            let Geometry::Curve(curve)=&entity.geometry else {return Err(Error::Invalid("Rib needs an open curve"));};
            let local=std::array::from_fn::<_,3,_>(|i|(0..3).map(|j|inverse.0[i][j]*normal[j]).sum::<f64>());
            let length=local[0].hypot(local[1]).hypot(local[2]);if !length.is_finite() || length<1e-12 {return Err(Error::Invalid("Rib plane normal"));}
            let transform=inverse.compose(entity.placement)?;
            let points=om9_core::slab::placed_open_polyline(curve,transform,self.coordinator.limits())?;
            curves.push((points,local.map(|v|v/length)));
        }
        let refs:Vec<_>=curves.iter().map(|(points,normal)|(points.as_slice(),*normal)).collect();
        let body=editing::rib_straight_planar_boundary_many(&refs,local_distance,self.solid_brep(boundary.object)?,
            creation::DEFAULT_DEFLECTION_MM,self.coordinator.limits()).map_err(|e|Error::Io(format!("Rib: {e}")))?;
        for (source,_) in sources {self.document().resolve(source)?;}self.document().resolve(&boundary)?;
        self.commit_solid_result("OM9-SOLID-022","Rib",vec![Mutation::Geometry {source:boundary,geometry:Geometry::Brep(body)}],Some(vec![boundary.object]))
    }

    /// Development-only Rib session; not a public command route.
    #[cfg(feature = "occt-brep")]
    #[doc(hidden)]
    pub fn begin_rib_development(&mut self) -> Result<()> {
        if self.is_running() {return Err(Error::Busy);}
        let mut pending=SolidPending::start("SolidRibDevelopment").expect("Rib development state");
        pending.slab_cplane_normal=self.input_plane().normal();
        self.command_run=self.command_run.wrapping_add(1);self.solid_pending=Some(pending);Ok(())
    }

    /// Test-only entry point for the unfinished Slab session; not a user command route.
    #[doc(hidden)]
    pub fn begin_slab_development(&mut self) -> Result<()> {
        if self.is_running() { return Err(Error::Busy); }
        if self.selection.len() > 1 { return Err(Error::Invalid("select one Slab curve")); }
        let stamp = if let Some(id) = self.selection.iter().next().copied() {
            let entity = self.document().entities.get(&id).ok_or(Error::Stale("Slab source"))?;
            SlabSourceGeometry::capture(entity, self.coordinator.limits())?;
            Some(self.document().stamp(id)?)
        } else { None };
        let mut pending = SolidPending::start("SolidSlab").expect("Slab state");
        pending.slab_source = stamp;
        pending.slab_cplane_normal = self.input_plane().normal();
        pending.slab_normal = pending.slab_cplane_normal;
        if let Some(source) = stamp {
            let entity = &self.document().entities[&source.object];
            pending.slab_normal = SlabSourceGeometry::capture(entity, self.coordinator.limits())?.normal(pending.slab_cplane_normal,self.coordinator.limits())?;
        }
        self.command_run = self.command_run.wrapping_add(1);
        self.solid_pending = Some(pending);
        Ok(())
    }

    pub fn solid_slab_needs_curve(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| p.feature == "SolidSlab" && p.slab_source.is_none())
    }

    pub(crate) fn solid_slab_pick_context(&self) -> Option<u64> {
        self.solid_slab_needs_curve().then_some(self.command_run)
    }

    pub fn solid_slab_pick_curve(&mut self, id: om9_core::identity::ObjectId) -> Result<()> {
        if !self.solid_slab_needs_curve() { return Err(Error::Missing("Slab source selection")); }
        let entity = self.document().entities.get(&id).ok_or(Error::Stale("Slab source"))?;
        let source = SlabSourceGeometry::capture(entity, self.coordinator.limits())?;
        let preferred = self.solid_pending.as_ref().expect("Slab session").slab_cplane_normal;
        let normal = source.normal(preferred,self.coordinator.limits())?;
        let stamp = self.document().stamp(id)?;
        let pending = self.solid_pending.as_mut().expect("Slab session");
        pending.slab_source = Some(stamp); pending.slab_normal = normal; pending.slab_in_cplane = false;
        Ok(())
    }

    pub fn dispatch_solid(&mut self, feature: &str) -> Result<()> {
        let feature = static_feature(feature);
        if feature == "SolidGVDMultiPipe" {
            self.clear_subselection();
            let mut pending = SolidPending::start("OM9-SOLID-011").expect("Pipe session");
            pending.pipe_multiple = true;
            pending.pipe_multiple_selecting = true;
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if feature == "SolidPipeChain" {
            self.clear_subselection();
            let mut pending = SolidPending::start("OM9-SOLID-011").expect("Pipe session");
            pending.pipe_chain = true;
            pending.pipe_multiple = true;
            pending.pipe_multiple_selecting = true;
            pending.pipe_chain_order = self.solid_pipe_existing_chain_selection()?;
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if feature == "SolidExtractSurfaceCopy" {
            self.clear_subselection();
            let mut pending = SolidPending::start("OM9-SOLID-006").expect("extract session");
            pending.extract_copy = true;
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if matches!(
            feature,
            "SolidBooleanSplit" | "SolidCreateSolid" | "OM9-SOLID-021"
        ) && cfg!(feature = "occt-brep")
        {
            self.solid_pending = SolidPending::start(feature);
            return Ok(());
        }
        if let Some(binding) = crate::solid_menu::by_feature(feature) {
            if !binding.enabled() {
                return Err(Error::Unsupported(binding.reason));
            }
        } else {
            return Err(Error::Unsupported("solid command is not catalogued"));
        }
        if matches!(
            feature,
            "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-003" | "OM9-SOLID-005"
        ) {
            self.solid_pending = SolidPending::start(feature);
            return Ok(());
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
            if matches!(feature, "OM9-SOLID-006" | "OM9-SOLID-007" | "OM9-SOLID-029") {
                self.clear_subselection();
            }
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

    /// Offset input plane for the current development Slab source, when available.
    pub fn slab_offset_input_plane(&self) -> Option<super::CPlane> {
        let pending = self.solid_pending.as_ref()?;
        if pending.feature != "SolidSlab" || pending.slab_offset.is_some() { return None; }
        let source = pending.slab_source?;
        if self.document().stamp(source.object).ok()? != source { return None; }
        let entity = self.document().entities.get(&source.object)?;
        let source = SlabSourceGeometry::capture(entity, self.coordinator.limits()).ok()?;
        let x=source.axis();let n=pending.slab_normal;
        let y=[n[1]*x[2]-n[2]*x[1],n[2]*x[0]-n[0]*x[2],n[0]*x[1]-n[1]*x[0]];
        super::CPlane::new(source.anchor(),x,y).ok()
    }

    #[cfg(feature = "occt-brep")]
    fn slab_preview_data(&self, pending: &SolidPending, point: [f64; 3]) -> Result<(Vec<[f64; 3]>, [f64; 3])> {
        let stamp = pending.slab_source.ok_or(Error::Missing("Slab source"))?;
        if self.document().stamp(stamp.object)? != stamp { return Err(Error::Stale("Slab source")); }
        let entity = self.document().entities.get(&stamp.object).ok_or(Error::Stale("Slab source"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Invalid("Slab requires a curve")); };
        let points = om9_core::slab::placed_open_polyline(curve, entity.placement, self.coordinator.limits())?;
        let normal = pending.slab_normal;
        let offset = match pending.slab_offset {
            Some(value) => value,
            None => {
                let picked = om9_core::slab::signed_offset_from_pick(&points, normal, point)?;
                pending.slab_distance.map_or(picked, |distance| picked.signum() * distance)
            },
        };
        let height = if pending.slab_offset.is_some() {
            (0..3).map(|i| (point[i]-points[0][i])*normal[i]).sum::<f64>()
        } else { 1.0 };
        om9_core::slab::polyline_slab(&points, normal, offset, height, pending.slab_offset.is_some() && pending.slab_both_sides, self.coordinator.limits())
    }

    #[cfg(feature = "occt-brep")]
    fn slab_preview(&self, point: [f64; 3]) -> Vec<[[f64; 3]; 2]> {
        let Some(pending) = self.solid_pending.as_ref() else { return Vec::new() };
        let Some(stamp) = pending.slab_source else { return Vec::new() };
        if self.document().stamp(stamp.object).ok() != Some(stamp) { return Vec::new(); }
        let Some(entity) = self.document().entities.get(&stamp.object) else { return Vec::new() };
        if let Ok(geometry @ SlabSourceGeometry::ClosedPolyline(_)) = SlabSourceGeometry::capture(entity, self.coordinator.limits()) {
            let SlabSourceGeometry::ClosedPolyline(points)=&geometry else {unreachable!()};
            let offset=match pending.slab_offset {Some(v)=>v,None=>match geometry.offset(pending.slab_normal,point,self.coordinator.limits()) {
                Ok(v)=>pending.slab_distance.map_or(v,|d|v.signum()*d),Err(_)=>return Vec::new()}};
            let height=if pending.slab_offset.is_some() {(0..3).map(|i|(point[i]-points[0][i])*pending.slab_normal[i]).sum()} else {1.};
            let Ok((outer,inner,direction))=om9_core::slab::closed_polyline_slab(points,pending.slab_normal,offset,height,
                pending.slab_offset.is_some() && pending.slab_both_sides,self.coordinator.limits()) else {return Vec::new()};
            let mut edges=Vec::new();
            for ring in [&outer,&inner] {for i in 0..ring.len() {
                let a=ring[i];let b=ring[(i+1)%ring.len()];edges.push([a,b]);
                if pending.slab_offset.is_some() {
                    let ta=std::array::from_fn(|j|a[j]+direction[j]);let tb=std::array::from_fn(|j|b[j]+direction[j]);
                    edges.push([ta,tb]);edges.push([a,ta]);
                }
            }}
            return edges;
        }
        if let Ok(geometry @ SlabSourceGeometry::Smooth { .. }) = SlabSourceGeometry::capture(entity, self.coordinator.limits()) {
            let SlabSourceGeometry::Smooth {curve,points,fractions,closed} = &geometry else {unreachable!()};
            let offset=match pending.slab_offset {Some(v)=>v,None=>match geometry.offset(pending.slab_normal,point,self.coordinator.limits()) {Ok(v)=>pending.slab_distance.map_or(v,|d|v.signum()*d),Err(_)=>return Vec::new()}};
            let height=if pending.slab_offset.is_some(){(0..3).map(|i|(point[i]-points[0][i])*pending.slab_normal[i]).sum::<f64>()}else{1.};
            if !height.is_finite() || height.abs()<1e-7 {return Vec::new();}
            if *closed {
                // Match the inexpensive sampled topology guard used before fitting on commit.
                // The fitted curve still requires its own later accuracy/OCCT checks.
                let Ok(samples)=om9_core::curve_ops::sample_curve(curve,65,self.coordinator.limits()) else {return Vec::new()};
                if om9_core::slab::closed_polyline_slab(&samples,pending.slab_normal,offset,height,
                    pending.slab_offset.is_some() && pending.slab_both_sides,self.coordinator.limits()).is_err() {return Vec::new();}
            }
            let shift=if pending.slab_offset.is_some() && pending.slab_both_sides {-height} else {0.};
            let travel=height*if pending.slab_offset.is_some() && pending.slab_both_sides {2.} else {1.};
            let mut inner=Vec::new();let mut outer=Vec::new();
            for (i,p) in points.iter().enumerate() {
                let Ok(t)=om9_core::curve_ops::tangent_at(curve,fractions[i],self.coordinator.limits()) else {return Vec::new()};
                let n=pending.slab_normal;let side=[t[1]*n[2]-t[2]*n[1],t[2]*n[0]-t[0]*n[2],t[0]*n[1]-t[1]*n[0]];
                let length=side[0].hypot(side[1]).hypot(side[2]);if length<1e-12{return Vec::new();}
                let a=std::array::from_fn::<_,3,_>(|j|p[j]+shift*n[j]);
                inner.push(a);outer.push(std::array::from_fn::<_,3,_>(|j|a[j]+offset*side[j]/length));
            }
            if *closed {
                inner.pop();outer.pop();let mut edges=Vec::new();
                for ring in [&inner,&outer] {for i in 0..ring.len() {
                    let a=ring[i];let b=ring[(i+1)%ring.len()];edges.push([a,b]);
                    if pending.slab_offset.is_some() {let ta=std::array::from_fn(|j|a[j]+travel*pending.slab_normal[j]);
                        let tb=std::array::from_fn(|j|b[j]+travel*pending.slab_normal[j]);edges.push([ta,tb]);if i%16==0 {edges.push([a,ta]);}}
                }}
                return edges;
            }
            let mut boundary=inner;boundary.extend(outer.into_iter().rev());let mut edges=Vec::new();
            for i in 0..boundary.len(){let a=boundary[i];let b=boundary[(i+1)%boundary.len()];edges.push([a,b]);
                if pending.slab_offset.is_some(){let ta=std::array::from_fn(|j|a[j]+travel*pending.slab_normal[j]);let tb=std::array::from_fn(|j|b[j]+travel*pending.slab_normal[j]);edges.push([ta,tb]);if i%16==0{edges.push([a,ta]);}}}
            return edges;
        }
        if let Ok(SlabSourceGeometry::Circle(circle)) = SlabSourceGeometry::capture(entity, self.coordinator.limits()) {
            let picked = || circle_offset(&circle, pending, point);
            let offset = match pending.slab_offset { Some(v)=>v, None=>match picked(){Ok(v)=>v,Err(_)=>return Vec::new()} };
            let height = if pending.slab_offset.is_some() { (0..3).map(|i|(point[i]-circle.center[i])*pending.slab_normal[i]).sum::<f64>() } else {1.};
            let Geometry::Curve(curve) = &entity.geometry else { return Vec::new() };
            if circle.angles[1]-circle.angles[0] < std::f64::consts::TAU-1e-8 {
                let Ok(world)=Self::solid_boss_transform_curve(curve,entity.placement) else {return Vec::new()};
                let Ok(slab)=om9_core::slab::arc_slab(&world,pending.slab_normal,offset,height,pending.slab_offset.is_some() && pending.slab_both_sides,self.coordinator.limits()) else {return Vec::new()};
                let mut edges=Vec::new();
                for i in 0..128 {
                    let at=|k:usize| om9_core::curve_ops::point_at(&slab.profile,k as f64/128.,self.coordinator.limits()).ok().map(|p|std::array::from_fn::<_,3,_>(|j|p[j]+slab.base_shift[j]));
                    let (Some(a),Some(b))=(at(i),at(i+1)) else {return Vec::new()};edges.push([a,b]);
                    if pending.slab_offset.is_some() {let ta=std::array::from_fn(|j|a[j]+slab.direction[j]);let tb=std::array::from_fn(|j|b[j]+slab.direction[j]);edges.push([ta,tb]);if i%16==0 {edges.push([a,ta]);}}
                }
                return edges;
            }
            let Ok(slab) = om9_core::slab::circle_slab(curve,entity.placement,pending.slab_normal,offset,height,pending.slab_offset.is_some() && pending.slab_both_sides,self.coordinator.limits()) else {return Vec::new()};
            let mut edges=Vec::new();
            for radius in [slab.inner_radius,slab.outer_radius] {
                for i in 0..64 {
                    let point_at=|k:usize,z:f64| {let a=std::f64::consts::TAU*k as f64/64.;slab.placement.point([radius*a.cos(),radius*a.sin(),z]).expect("validated slab frame")};
                    let a=point_at(i,0.);let b=point_at(i+1,0.);edges.push([a,b]);
                    if pending.slab_offset.is_some() {let topa=point_at(i,slab.height);let topb=point_at(i+1,slab.height);edges.push([topa,topb]);if i%8==0 {edges.push([a,topa]);}}
                }
            }
            return edges;
        }
        let Ok((boundary, direction)) = self.slab_preview_data(pending, point) else { return Vec::new() };
        let mut edges = Vec::new();
        for i in 0..boundary.len() {
            let a = boundary[i]; let b = boundary[(i+1)%boundary.len()];
            edges.push([a,b]);
            if pending.slab_offset.is_some() {
                let top_a = std::array::from_fn(|j| a[j]+direction[j]);
                let top_b = std::array::from_fn(|j| b[j]+direction[j]);
                edges.push([top_a,top_b]); edges.push([a,top_a]);
            }
        }
        edges
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn slab_accept_point(&mut self, point: [f64; 3]) -> Result<()> {
        let pending = self.solid_pending.as_ref().filter(|p| p.feature == "SolidSlab").ok_or(Error::Missing("Slab session"))?;
        let source = pending.slab_source.ok_or(Error::Missing("Slab source"))?;
        if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
        let entity = self.document().entities.get(&source.object).ok_or(Error::Stale("Slab source"))?;
        let geometry = SlabSourceGeometry::capture(entity, self.coordinator.limits())?;
        let anchor = geometry.anchor();
        let normal = pending.slab_normal;
        if let Some(offset) = pending.slab_offset {
            let height = (0..3).map(|i| (point[i]-anchor[i])*normal[i]).sum::<f64>();
            let both = pending.slab_both_sides;
            self.solid_slab_commit_polyline(source, normal, offset, height, both)?;
            self.solid_pending = None;
            Ok(())
        } else {
            let picked = geometry.offset(normal, point, self.coordinator.limits())?;
            let offset = pending.slab_distance.map_or(picked, |distance| picked.signum() * distance);
            match &geometry {
                SlabSourceGeometry::Polyline(points) => { om9_core::slab::polyline_slab(points,normal,offset,1.,false,self.coordinator.limits())?; },
                SlabSourceGeometry::ClosedPolyline(points) => { om9_core::slab::closed_polyline_slab(points,normal,offset,1.,false,self.coordinator.limits())?; },
                SlabSourceGeometry::Smooth { curve, closed, .. } => {
                    if *closed {om9_core::slab::closed_nurbs_slab(curve,normal,offset,1.,false,1e-4,self.coordinator.limits())?;}
                    else {om9_core::slab::nurbs_slab(curve,normal,offset,1.,false,1e-4,self.coordinator.limits())?;}
                },
                SlabSourceGeometry::Circle(circle) => {
                    let Geometry::Curve(curve) = &entity.geometry else { unreachable!() };
                    if circle.angles[1]-circle.angles[0] < std::f64::consts::TAU-1e-8 {
                        let world=Self::solid_boss_transform_curve(curve,entity.placement)?;
                        om9_core::slab::arc_slab(&world,normal,offset,1.,false,self.coordinator.limits())?;
                    } else { om9_core::slab::circle_slab(curve,entity.placement,normal,offset,1.,false,self.coordinator.limits())?; }
                }
            }
            self.solid_pending.as_mut().expect("Slab session").slab_offset = Some(offset);
            Ok(())
        }
    }

    pub fn solid_preview(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Vec<[om9_core::geometry::Point3; 2]> {
        #[cfg(feature = "occt-brep")]
        {
            if self.solid_pending.as_ref().is_some_and(|pending| pending.feature == "SolidSlab") {
                self.slab_preview(point)
            } else if self
                .solid_pending
                .as_ref()
                .is_some_and(|pending| pending.feature == "OM9-SOLID-011")
            {
                self.solid_preview_pipe_straight_body(point)
                    .unwrap_or_else(|| self.solid_preview_pipe_section(point))
            } else if self.solid_pending.as_ref().is_some_and(|pending| {
                matches!(
                    pending.feature,
                    "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
                )
            }) {
                self.solid_preview_cone(point)
            } else if self
                .solid_pending
                .as_ref()
                .is_some_and(|pending| pending.feature == "OM9-SOLID-018")
            {
                self.solid_preview_pyramid(point)
            } else if self
                .solid_pending
                .as_ref()
                .is_some_and(|pending| pending.feature == "OM9-SOLID-013")
            {
                self.solid_preview_ellipsoid(point)
            } else if self
                .solid_pending
                .as_ref()
                .is_some_and(|pending| pending.feature == "OM9-SOLID-015")
            {
                self.solid_preview_torus(point)
            } else if self
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
        if pending.feature == "SolidRibDevelopment" {
            let mut options = vec![("Undo", "Undo"), ("Cancel", "Cancel")];
            if pending.difference_targets.is_some() {
                options.insert(0, ("Distance=", "Distance="));
            }
            return options;
        }
        if pending.feature == "SolidSlab" {
            let mut options = vec![("BothSides", if pending.slab_both_sides { "BothSides=No" } else { "BothSides=Yes" }), ("Undo", "Undo"), ("Cancel", "Cancel")];
            if pending.slab_offset.is_none() { options.insert(0, ("ThroughPoint", "ThroughPoint")); options.insert(0, ("InCPlane", if pending.slab_in_cplane { "InCPlane=No" } else { "InCPlane=Yes" })); }
            return options;
        }
        if pending.feature == "OM9-SOLID-018" {
            let mut options = vec![("Undo", "Undo"), ("Cancel", "Cancel")];
            options.insert(
                0,
                (
                    "Solid",
                    if pending.cone_flags.solid() {
                        "Solid=No"
                    } else {
                        "Solid=Yes"
                    },
                ),
            );
            if pending.values.is_empty() {
                options.insert(0, ("NumSides=", "NumSides="));
                if pending.points.is_empty() {
                    options.insert(0, ("AroundCurve", "AroundCurve"));
                    if pending.pyramid_around && !self.solid_pipe_needs_curve() {
                        options.insert(0, ("OnCurve=", "OnCurve="));
                    }
                    options.insert(
                        0,
                        if pending.cone_base_points == 0 && !pending.pyramid_around {
                            ("Edge", "Edge")
                        } else {
                            ("Center", "Center")
                        },
                    );
                    if pending.cone_base_points != 3 {
                        options.insert(0, ("Star", "Star"));
                    }
                }
                if pending.cone_base_points == 0 {
                    options.insert(
                        0,
                        if pending.cone_flags.pyramid_circumscribed() {
                            ("Inscribed", "Inscribed")
                        } else {
                            ("Circumscribed", "Circumscribed")
                        },
                    );
                }
            }
            if matches!(pending.cone_base_points, 1 | 2) && !pending.points.is_empty() {
                options.insert(0, ("Flip", "Flip"));
            }
            if pending.cone_base_points == 3 && pending.values.len() == 1 {
                options.insert(0, ("Automatic", "Automatic"));
            }
            return options;
        }
        if pending.feature == "OM9-SOLID-017" {
            let mut options = vec![
                (
                    "BothSides",
                    if pending.cone_flags.both_sides() {
                        "BothSides=No"
                    } else {
                        "BothSides=Yes"
                    },
                ),
                (
                    "Diameter",
                    if pending.cone_diameter {
                        "Radius"
                    } else {
                        "Diameter"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
            if pending.points.is_empty() {
                options.insert(0, ("2Point", "Mode=2Point"));
                options.insert(0, ("3Point", "Mode=3Point"));
                options.insert(0, ("FitPoints", "Mode=FitPoints"));
                options.insert(0, ("AroundCurve", "Mode=AroundCurve"));
                options.insert(0, ("Tangent", "Mode=Tangent"));
                if pending.cone_base_points != 0 {
                    options.insert(0, ("Center", "Mode=Center"));
                }
            }
            if matches!(pending.cone_base_points, 5 | 6)
                && pending.points.is_empty()
                && !self.solid_pipe_needs_curve()
            {
                options.insert(0, ("OnCurve=", "OnCurve="));
            }
            if pending.cone_base_points == 6
                && pending.points.len() == 1
                && pending.values.is_empty()
                && !pending.cone_flags.point_mode()
            {
                options.insert(0, ("Point", "Point"));
            }
            if pending.cone_base_points == 3
                && pending.points.len() == 2
                && pending.values.is_empty()
                && pending.cone_flags.radius_phase() < 2
            {
                options.insert(0, ("Radius=", "Radius="));
            }
            if pending.values.len() == 1 {
                options.insert(0, ("WallThickness=", "WallThickness="));
            }
            if matches!(pending.cone_base_points, 0 | 5)
                && pending.points.len() == 1
                && pending.values.is_empty()
            {
                options.insert(
                    0,
                    ("Area", if pending.cone_area { "Radius" } else { "Area" }),
                );
            }
            return options;
        }
        if matches!(pending.feature, "OM9-SOLID-016" | "OM9-SOLID-019") {
            let mut options = vec![
                (
                    "Solid",
                    if pending.cone_flags.solid() {
                        "Solid=No"
                    } else {
                        "Solid=Yes"
                    },
                ),
                (
                    "Diameter",
                    if pending.cone_diameter {
                        "Radius"
                    } else {
                        "Diameter"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
            if pending.points.is_empty() {
                options.insert(0, ("2Point", "Mode=2Point"));
                options.insert(0, ("3Point", "Mode=3Point"));
                options.insert(0, ("AroundCurve", "Mode=AroundCurve"));
                options.insert(0, ("FitPoints", "Mode=FitPoints"));
                options.insert(0, ("Tangent", "Mode=Tangent"));
                if pending.cone_base_points != 0 {
                    options.insert(0, ("Center", "Mode=Center"));
                }
            }
            if pending.feature == "OM9-SOLID-019"
                && matches!(pending.cone_base_points, 0 | 5)
                && pending.points.len() == 1
                && pending.values.is_empty()
            {
                options.insert(
                    0,
                    ("Area", if pending.cone_area { "Radius" } else { "Area" }),
                );
            }
            if matches!(pending.cone_base_points, 5 | 6)
                && pending.points.is_empty()
                && !self.solid_pipe_needs_curve()
            {
                options.insert(0, ("OnCurve=", "OnCurve="));
            }
            if pending.cone_base_points == 6
                && pending.points.len() == 1
                && pending.values.is_empty()
                && !pending.cone_flags.point_mode()
            {
                options.insert(0, ("Point", "Point"));
            }
            if pending.cone_base_points == 3
                && pending.points.len() == 2
                && pending.values.is_empty()
                && pending.cone_flags.radius_phase() < 2
            {
                options.insert(0, ("Radius=", "Radius="));
            }
            return options;
        }
        if pending.feature == "OM9-SOLID-020" {
            let mut options = vec![
                (
                    "Solid",
                    if pending.cone_flags.solid() {
                        "Solid=No"
                    } else {
                        "Solid=Yes"
                    },
                ),
                (
                    "Diameter",
                    if pending.cone_diameter {
                        "Radius"
                    } else {
                        "Diameter"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
            if pending.points.is_empty() {
                options.insert(0, ("3Point", "Mode=3Point"));
                options.insert(0, ("2Point", "Mode=2Point"));
                options.insert(0, ("FitPoints", "Mode=FitPoints"));
                options.insert(0, ("AroundCurve", "Mode=AroundCurve"));
                options.insert(0, ("Tangent", "Mode=Tangent"));
                if pending.cone_base_points != 0 {
                    options.insert(0, ("Center", "Mode=Center"));
                }
            }
            if matches!(pending.cone_base_points, 0 | 5)
                && pending.points.len() == 1
                && pending.values.is_empty()
            {
                options.insert(
                    0,
                    ("Area", if pending.cone_area { "Radius" } else { "Area" }),
                );
            }
            if matches!(pending.cone_base_points, 5 | 6)
                && pending.points.is_empty()
                && !self.solid_pipe_needs_curve()
            {
                options.insert(0, ("OnCurve=", "OnCurve="));
            }
            if pending.cone_base_points == 6
                && pending.points.len() == 1
                && pending.values.is_empty()
                && !pending.cone_flags.point_mode()
            {
                options.insert(0, ("Point", "Point"));
            }
            if pending.cone_base_points == 3
                && pending.points.len() == 2
                && pending.values.is_empty()
                && pending.cone_flags.radius_phase() == 0
            {
                options.insert(0, ("Radius=", "Radius="));
            }
            return options;
        }
        if pending.feature == "OM9-SOLID-011" {
            return vec![
                ("ShapeBlending=Global", "ShapeBlending=Global"),
                ("ShapeBlending=Local", "ShapeBlending=Local"),
                (
                    "Cap",
                    if pending.pipe_round {
                        "Cap=Flat"
                    } else if pending.pipe_capped {
                        "Cap=None"
                    } else {
                        "Cap=Round"
                    },
                ),
                (
                    "Diameter",
                    if pending.pipe_diameter {
                        "Radius"
                    } else {
                        "Diameter"
                    },
                ),
                (
                    "Thick",
                    if pending.pipe_thick {
                        "Thick=No"
                    } else {
                        "Thick=Yes"
                    },
                ),
                ("Multiple", "Multiple"),
                ("ChainEdges", "ChainEdges"),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if matches!(pending.feature, "OM9-SOLID-005" | "OM9-SOLID-021") {
            return vec![("Undo", "Undo"), ("Cancel", "Cancel")];
        }
        if pending.feature == "OM9-SOLID-006" {
            return vec![
                (
                    "Copy",
                    if pending.extract_copy {
                        "Copy=No"
                    } else {
                        "Copy=Yes"
                    },
                ),
                (
                    "OutputLayer",
                    if pending.extract_input_layer {
                        "OutputLayer=Current"
                    } else {
                        "OutputLayer=Input"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if pending.feature == "OM9-SOLID-029" {
            return vec![
                ("Thickness=", "Thickness="),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if pending.feature == "OM9-SOLID-007" {
            let show_radius = (
                "ShowRadius",
                if pending.fillet_show_radius {
                    "ShowRadius=No"
                } else {
                    "ShowRadius=Yes"
                },
            );
            if pending.fillet_preview.is_some() {
                let link_handles = (
                    "LinkHandles",
                    if pending.fillet_link_handles {
                        "LinkHandles=No"
                    } else {
                        "LinkHandles=Yes"
                    },
                );
                if pending.fillet_copy_phase != 0 {
                    return vec![("Cancel", "Cancel")];
                }
                if pending.fillet_add_mode {
                    return vec![
                        ("CurrentRadius=", "CurrentRadius="),
                        ("Undo", "Undo"),
                        ("Cancel", "Cancel"),
                    ];
                }
                if pending.fillet_remove_mode {
                    return vec![("Cancel", "Cancel")];
                }
                let mut tokens = if pending.fillet_handle.is_some() {
                    vec![
                        ("CurrentRadius=", "CurrentRadius="),
                        ("AddHandle", "AddHandle"),
                        ("SetAll=", "SetAll="),
                        ("SelectEdges", "SelectEdges"),
                        link_handles,
                        show_radius,
                        ("Undo", "Undo"),
                        ("Cancel", "Cancel"),
                    ]
                } else {
                    vec![
                        ("AddHandle", "AddHandle"),
                        ("SetAll=", "SetAll="),
                        ("SelectEdges", "SelectEdges"),
                        link_handles,
                        show_radius,
                        ("Undo", "Undo"),
                        ("Cancel", "Cancel"),
                    ]
                };
                if !pending.fillet_controls.is_empty() {
                    tokens.insert(1, ("RemoveHandle", "RemoveHandle"));
                }
                tokens.insert(1, ("CopyHandle", "CopyHandle"));
                return tokens;
            }
            return vec![
                ("NextRadius=", "NextRadius="),
                show_radius,
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if matches!(
            pending.feature,
            "OM9-SOLID-001"
                | "OM9-SOLID-002"
                | "OM9-SOLID-003"
                | "SolidBooleanSplit"
                | "SolidCreateSolid"
        ) {
            return vec![
                (
                    "DeleteInput",
                    if pending.difference_delete_input {
                        "DeleteInput=No"
                    } else {
                        "DeleteInput=Yes"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if pending.feature == "OM9-SOLID-004" {
            return vec![
                ("Union", "Union"),
                ("Difference", "Difference"),
                ("ReverseDifference", "ReverseDifference"),
                ("Intersection", "Intersection"),
                ("InversionIntersection", "InversionIntersection"),
                ("Next", "Next"),
                (
                    "DeleteInput",
                    if pending.difference_delete_input {
                        "DeleteInput=No"
                    } else {
                        "DeleteInput=Yes"
                    },
                ),
                ("Undo", "Undo"),
                ("Cancel", "Cancel"),
            ];
        }
        if pending.feature == "OM9-SOLID-013" {
            return if pending.ellipsoid_around && pending.points.is_empty() {
                vec![
                    ("OnCurve=", "OnCurve="),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else if pending.ellipsoid_foci {
                vec![
                    (
                        "MarkFoci",
                        if pending.ellipsoid_mark_foci {
                            "MarkFoci=No"
                        } else {
                            "MarkFoci=Yes"
                        },
                    ),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else if pending.points.is_empty() && pending.values.is_empty() {
                vec![
                    ("Diameter", "Diameter"),
                    ("Corner", "Corner"),
                    ("AroundCurve", "AroundCurve"),
                    ("FromFoci", "FromFoci"),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else {
                vec![("Undo", "Undo"), ("Cancel", "Cancel")]
            };
        }
        if pending.feature == "OM9-SOLID-015" {
            return if pending.torus_around && pending.points.is_empty() {
                vec![
                    ("OnCurve=", "OnCurve="),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else if pending.torus_three_point
                && pending.points.len() == 2
                && pending.cone_flags.radius_phase() < 2
            {
                vec![
                    ("Radius=", "Radius="),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else if pending.points.is_empty() && pending.values.is_empty() {
                vec![
                    ("2Point", "2Point"),
                    ("3Point", "3Point"),
                    ("Vertical", "Vertical"),
                    ("AroundCurve", "AroundCurve"),
                    ("Undo", "Undo"),
                    ("Cancel", "Cancel"),
                ]
            } else {
                vec![("Undo", "Undo"), ("Cancel", "Cancel")]
            };
        }
        if pending.sphere_mode {
            if pending.sphere_kind == "Tangent"
                && pending.sphere_curve.is_empty()
                && pending.points.is_empty()
                && pending.sphere_center.is_none()
            {
                return vec![("Point", "Point"), ("Undo", "Undo"), ("Cancel", "Cancel")];
            }
            if matches!(
                pending.sphere_kind,
                "AroundCurve" | "Tangent" | "Tangent2" | "Tangent3"
            ) && !pending.sphere_curve.is_empty()
                && pending.points.is_empty()
                && pending.sphere_center.is_none()
            {
                if matches!(pending.sphere_kind, "Tangent" | "Tangent2") {
                    return vec![("Point", "Point"), ("Undo", "Undo"), ("Cancel", "Cancel")];
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

    pub fn solid_difference_needs_object(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-001")
    }

    /// Sphere AroundCurve viewport edge selection: 1 selects source, 2 selects its center.
    pub fn solid_sphere_brep_edge_stage(&self) -> u8 {
        let Some(p) = self.solid_pending.as_ref() else { return 0; };
        if !p.sphere_mode || p.sphere_kind != "AroundCurve" || p.sphere_center.is_some() { return 0; }
        if p.sphere_curve.is_empty() { 1 } else if p.sphere_edge.is_some() { 2 } else { 3 }
    }

    /// Check a viewport hit against the captured standalone curve source.
    pub fn solid_sphere_is_source_curve(&self, hit: &om9_scene::scene::PickHit) -> bool {
        let Some(pending) = self.solid_pending.as_ref() else { return false; };
        self.solid_sphere_brep_edge_stage() == 3 && hit.path.is_empty()
            && pending.sphere_sources.first() == Some(&hit.source)
            && hit.root == hit.source.object
            && matches!(self.document().entities.get(&hit.root).map(|e| &e.geometry), Some(Geometry::Curve(_)))
    }

    pub(crate) fn solid_sphere_edge_pick_context(&self) -> Option<u64> {
        (self.solid_sphere_brep_edge_stage() == 1).then_some(self.command_run)
    }

    /// Check whether a hit identifies the already captured AroundCurve BRep edge.
    pub fn solid_sphere_is_source_edge(&self, reference: &super::SelectedSubobject) -> bool {
        let Some(pending) = self.solid_pending.as_ref() else { return false; };
        if self.solid_sphere_brep_edge_stage() != 2 || !reference.path.is_empty()
            || pending.sphere_sources.first() != Some(&reference.source) { return false; }
        let Some(index) = pending.sphere_edge else { return false; };
        let Some(entity) = self.document().entities.get(&reference.root) else { return false; };
        let Geometry::Brep(brep) = &entity.geometry else { return false; };
        matches!(&reference.subset, om9_scene::scene::Subelement::BrepEdge(key) if brep.display.edges.get(index) == Some(key))
    }

    #[cfg(feature = "occt-brep")]
    pub fn solid_sphere_pick_curve(&mut self, id: om9_core::identity::ObjectId) -> Result<()> {
        if self.solid_sphere_brep_edge_stage() != 1 { return Err(Error::Missing("Sphere AroundCurve curve selection")); }
        let entity = self.document().entities.get(&id).ok_or(Error::Stale("Sphere curve"))?;
        if !matches!(entity.geometry, Geometry::Curve(_)) { return Err(Error::Invalid("select a curve")); }
        let samples = sphere_world_curve_samples(entity, self.coordinator.limits())?;
        let stamp = self.document().stamp(id)?;
        let pending = self.solid_pending.as_mut().expect("Sphere session");
        pending.sphere_curve = samples;
        pending.sphere_sources = vec![stamp];
        pending.sphere_edge = None;
        Ok(())
    }

    #[cfg(not(feature = "occt-brep"))]
    pub fn solid_sphere_pick_curve(&mut self, _id: om9_core::identity::ObjectId) -> Result<()> {
        Err(Error::Unsupported("Sphere AroundCurve curve"))
    }

    #[cfg(feature = "occt-brep")]
    pub fn solid_sphere_pick_brep_edge(&mut self, reference: super::SelectedSubobject, world: [f64; 3]) -> Result<()> {
        let stage = self.solid_sphere_brep_edge_stage();
        if !matches!(stage, 1 | 2) { return Err(Error::Missing("Sphere AroundCurve edge selection")); }
        if !reference.path.is_empty() { return Err(Error::Unsupported("Sphere AroundCurve instance edge")); }
        if self.scene().is_none_or(|scene| scene.epoch != reference.world_epoch) || self.document().stamp(reference.root)? != reference.source {
            return Err(Error::Stale("Sphere AroundCurve edge"));
        }
        let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else { return Err(Error::Invalid("select a B-Rep edge")); };
        let entity = self.document().entities.get(&reference.root).ok_or(Error::Stale("Sphere AroundCurve source"))?;
        let Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Invalid("select a B-Rep edge")); };
        let index = brep.display.edges.iter().position(|id| *id == key).ok_or(Error::Stale("Sphere AroundCurve edge"))?;
        if stage == 2 {
            let pending = self.solid_pending.as_ref().expect("Sphere session");
            if pending.sphere_sources.first() != Some(&reference.source) || pending.sphere_edge != Some(index) {
                return Err(Error::Invalid("pick center on the selected edge"));
            }
            return self.accept_sphere_point(world);
        }
        let source = sphere_curve_source(entity, Some(index), self.coordinator.limits())?;
        let samples = sphere_world_curve_samples(&source, self.coordinator.limits())?;
        let pending = self.solid_pending.as_mut().expect("Sphere session");
        pending.sphere_curve = samples;
        pending.sphere_sources = vec![reference.source];
        pending.sphere_edge = Some(index);
        Ok(())
    }

    #[cfg(not(feature = "occt-brep"))]
    pub fn solid_sphere_pick_brep_edge(&mut self, _reference: super::SelectedSubobject, _world: [f64; 3]) -> Result<()> {
        Err(Error::Unsupported("Sphere AroundCurve B-Rep edge"))
    }





    pub fn submit_solid_text(&mut self, text: &str) -> Option<Result<()>> {
        if self.solid_pending.is_none() {
            self.submit_solid_command(text)
        } else {
            self.submit_solid_pending(text)
        }
    }

    #[inline(never)]
    fn submit_solid_pending(&mut self, text: &str) -> Option<Result<()>> {
        #[allow(unused_mut)]
        let Some(mut pending) = self.solid_pending.take() else {
            return None;
        };
        #[cfg(feature = "occt-brep")]
        if pending.feature == "SolidRibDevelopment" {
            let input=text.trim();
            if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {self.cancel();return Some(Ok(()));}
            if input.eq_ignore_ascii_case("undo") {
                if pending.difference_targets.take().is_some() {self.clear_selection();}
                else if pending.slab_source.take().is_some() {pending.rib_sources.clear();self.clear_selection();}
                else if let Some(id)=self.selected_ordered().last().copied() {let _=self.remove_objects(&[id]);}
                self.solid_pending=Some(pending);return Some(Ok(()));
            }
            if pending.slab_source.is_none() {
                if !input.is_empty() {self.solid_pending=Some(pending);return Some(Err(Error::Invalid("select Rib curve and press Enter")));}
                let ids=self.selected_ordered();
                if ids.is_empty() || ids.len() as u64 > self.coordinator.limits().max_outputs {self.solid_pending=Some(pending);return Some(Err(Error::Missing("select open Rib curves")));}
                let result=(|| -> Result<Vec<(om9_core::identity::SourceStamp,[f64;3])>> {
                    let mut sources=Vec::new();sources.try_reserve_exact(ids.len()).map_err(|_|Error::Budget("Rib curves"))?;
                    for id in ids {
                        let entity=self.document().entities.get(&id).ok_or(Error::Stale("Rib curve"))?;
                        let Geometry::Curve(curve)=&entity.geometry else {return Err(Error::Invalid("Rib needs an open curve"));};
                        let points=om9_core::slab::placed_open_polyline(curve,entity.placement,self.coordinator.limits())?;
                        let normal=om9_core::slab::source_plane_normal(&points,pending.slab_cplane_normal)?;
                        sources.push((self.document().stamp(id)?,normal));
                    }
                    Ok(sources)
                })();
                match result {Ok(sources)=>{pending.slab_source=Some(sources[0].0);pending.rib_sources=sources;self.clear_selection();self.solid_pending=Some(pending);return Some(Ok(()));},
                    Err(e)=>{self.solid_pending=Some(pending);return Some(Err(e));}}
            }
            if pending.difference_targets.is_none() {
                if !input.is_empty() {self.solid_pending=Some(pending);return Some(Err(Error::Invalid("select Rib boundary and press Enter")));}
                let ids=self.selected_ordered();
                if ids.len()!=1 {self.solid_pending=Some(pending);return Some(Err(Error::Missing("select one Rib boundary")));}
                let result=(|| -> Result<om9_core::identity::SourceStamp> {self.solid_brep(ids[0])?;self.document().stamp(ids[0])})();
                match result {Ok(stamp)=>{pending.difference_targets=Some(vec![stamp]);self.clear_selection();self.solid_pending=Some(pending);return Some(Ok(()));},
                    Err(e)=>{self.solid_pending=Some(pending);return Some(Err(e));}}
            }
            let raw=match input.split_once('=') {
                Some((key,value)) if key.trim().eq_ignore_ascii_case("distance") => value.trim(),
                _ => input,
            };
            let Some(value)=raw.parse::<f64>().ok().filter(|v|v.is_finite() && *v!=0.) else {
                self.solid_pending=Some(pending);return Some(Err(Error::Invalid("Rib distance must be finite and nonzero")));
            };
            let value=match self.document().page_unit.length_to_mm(value) {Ok(v)=>v,Err(e)=>{self.solid_pending=Some(pending);return Some(Err(e));}};
            let result=self.solid_rib_commit_straight_many(&pending.rib_sources,pending.difference_targets.as_ref().expect("Rib boundary")[0],value);
            if result.is_err() {self.solid_pending=Some(pending);}return Some(result);
        }
        #[cfg(feature = "occt-brep")]
        if pending.feature == "SolidSlab" {
            return self.submit_solid_slab_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-018" {
            return self.submit_pyramid_text(pending, text);
        }
        if matches!(
            pending.feature,
            "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
        ) {
            return self.submit_cone_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-015" {
            return self.submit_solid_torus_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-013" {
            return self.submit_solid_ellipsoid_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-021" {
            return self.submit_solid_boss_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-011" {
            return self.submit_solid_pipe_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-007" {
            return self.submit_solid_fillet_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-029" {
            return self.submit_solid_shell_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-006" {
            return self.submit_solid_extract_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-005" {
            return self.submit_solid_cap_text(pending, text);
        }
        if matches!(
            pending.feature,
            "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-003" | "SolidBooleanSplit" | "SolidCreateSolid"
        ) {
            return self.submit_solid_boolean_text(pending, text);
        }
        if pending.sphere_mode {
            return self.submit_solid_sphere_text(pending, text);
        }
        if pending.box_mode {
            return self.submit_solid_box_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-004" {
            return self.submit_solid_boolean_two_text(pending, text);
        }
        if pending.feature == "OM9-SOLID-008" {
            return self.submit_solid_text_feature(pending, text);
        }
        self.submit_solid_generic_tail(pending, text)
    }

    #[inline(never)]
    fn submit_solid_command(&mut self, text: &str) -> Option<Result<()>> {
        let mut words = text.split_whitespace();
        let name = words.next()?;
        let feature = if name.eq_ignore_ascii_case("_Pipe") {
            "OM9-SOLID-011"
        } else {
            feature_for_command(name)?
        };
        if !cfg!(feature = "occt-brep") { return Some(Err(Error::Unsupported("Solid command requires OCCT"))); }
        let args: Vec<&str> = words.collect();
        if feature == "OM9-SOLID-014" && args.len() == 1 {
            let option = args[0].trim_start_matches('_');
            let target = match option.to_ascii_lowercase().as_str() {
                "p" | "2point" => Some("SolidSphere2Points"),
                "3point" => Some("SolidSphere3Points"),
                "4point" => Some("SolidSphere4Points"),
                "aroundcurve" => Some("SolidSphereAroundCurve"),
                "tangent" => Some("SolidSphereTangenttoCurves"),
                "fitpoints" => Some("SolidSphereFitSpheretoPoints"),
                _ => None,
            };
            if let Some(target) = target {
                return Some(self.dispatch(target, InvocationOrigin::Cmd));
            }
        }
        if feature == "OM9-SOLID-012" && args.len() == 1 {
            let option = args[0].trim_start_matches('_');
            let target = if option.eq_ignore_ascii_case("diagonal") {
                Some("SolidBoxDiagonal")
            } else if option.eq_ignore_ascii_case("3point") || option.eq_ignore_ascii_case("p") {
                Some("SolidBox3PointsHeight")
            } else if option.eq_ignore_ascii_case("vertical") {
                Some("SolidBoxVerticalBase")
            } else if option.eq_ignore_ascii_case("center") {
                Some("SolidBoxCenterofBaseCornerHeight")
            } else {
                None
            };
            if let Some(target) = target {
                return Some(self.dispatch(target, InvocationOrigin::Cmd));
            }
        }
        if feature == "OM9-SOLID-018"
            && args.len() == 2
            && args[1].trim_start_matches('_').eq_ignore_ascii_case("s")
        {
            let prefix = args[0].trim_start_matches('_').to_ascii_lowercase();
            let sides = prefix
                .strip_prefix("numsides=")
                .and_then(|v| v.parse::<u32>().ok())
                .filter(|n| {
                    *n >= 3
                        && *n <= 2048
                        && u64::from(*n) * 2 <= self.coordinator.limits().max_points
                        && u64::from(*n) * 1024 <= self.coordinator.limits().max_bytes
                });
            return Some(match sides {
                Some(sides) => {
                    let mut pending = SolidPending::start(feature).expect("Pyramid session");
                    pending.cone_base_points = 3;
                    pending.pyramid_sides = sides;
                    self.solid_pending = Some(pending);
                    Ok(())
                }
                None => Err(Error::Invalid("pyramid star side count or budget")),
            });
        }
        if feature == "OM9-SOLID-018"
            && args.len() == 1
            && args[0].trim_start_matches('_').eq_ignore_ascii_case("star")
        {
            let mut pending = SolidPending::start(feature).expect("Pyramid session");
            pending.cone_base_points = 3;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-018"
            && args.len() == 1
            && args[0].trim_start_matches('_').eq_ignore_ascii_case("edge")
        {
            let mut pending = SolidPending::start(feature).expect("Pyramid session");
            pending.cone_base_points = 1;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-018"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("aroundcurve")
        {
            let mut pending = SolidPending::start(feature).expect("Pyramid session");
            pending.pyramid_around = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(feature, "OM9-SOLID-016" | "OM9-SOLID-017" | "OM9-SOLID-019")
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("tangent")
        {
            let mut pending = SolidPending::start(feature).expect("Cylinder session");
            pending.cone_base_points = 6;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(feature, "OM9-SOLID-016" | "OM9-SOLID-017" | "OM9-SOLID-019")
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("fitpoints")
        {
            let mut pending = SolidPending::start(feature).expect("Cylinder session");
            pending.cone_base_points = 4;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(feature, "OM9-SOLID-016" | "OM9-SOLID-017" | "OM9-SOLID-019")
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("aroundcurve")
        {
            let mut pending = SolidPending::start(feature).expect("Cylinder session");
            pending.cone_base_points = 5;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(feature, "OM9-SOLID-016" | "OM9-SOLID-017" | "OM9-SOLID-019")
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("3point")
        {
            let mut pending = SolidPending::start(feature).expect("Cylinder session");
            pending.cone_base_points = 3;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(feature, "OM9-SOLID-016" | "OM9-SOLID-017" | "OM9-SOLID-019")
            && args.len() == 1
            && matches!(
                args[0]
                    .trim_start_matches('_')
                    .to_ascii_lowercase()
                    .as_str(),
                "p" | "2point"
            )
        {
            let mut pending = SolidPending::start(feature).expect("Cylinder session");
            pending.cone_base_points = 2;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-015"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("vertical")
        {
            let mut pending = SolidPending::start(feature).expect("Torus session");
            pending.torus_vertical = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-015"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("aroundcurve")
        {
            let mut pending = SolidPending::start(feature).expect("Torus session");
            pending.torus_around = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-015"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("3point")
        {
            let mut pending = SolidPending::start(feature).expect("Torus session");
            pending.torus_three_point = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-015"
            && args.len() == 1
            && (args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("2point")
                || args[0].trim_start_matches('_').eq_ignore_ascii_case("p"))
        {
            let mut pending = SolidPending::start(feature).expect("Torus session");
            pending.torus_two_point = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-013"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("aroundcurve")
        {
            let mut pending = SolidPending::start(feature).expect("Ellipsoid session");
            pending.ellipsoid_around = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-013"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("corner")
        {
            let mut pending = SolidPending::start(feature).expect("Ellipsoid session");
            pending.ellipsoid_corner = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-013"
            && args.len() == 1
            && args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("fromfoci")
        {
            let mut pending = SolidPending::start(feature).expect("Ellipsoid session");
            pending.ellipsoid_foci = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "OM9-SOLID-013"
            && args.len() == 1
            && (args[0]
                .trim_start_matches('_')
                .eq_ignore_ascii_case("diameter")
                || args[0].trim_start_matches('_').eq_ignore_ascii_case("d"))
        {
            let mut pending = SolidPending::start(feature).expect("Ellipsoid session");
            pending.ellipsoid_diameter = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if feature == "SolidPipeChain"
            || (feature == "OM9-SOLID-011"
                && args.len() == 1
                && (args[0].eq_ignore_ascii_case("ChainEdges")
                    || args[0].eq_ignore_ascii_case("_C")))
        {
            return Some(self.dispatch("SolidPipeChain", InvocationOrigin::Cmd));
        }
        if feature == "SolidGVDMultiPipe" && !args.is_empty() {
            return Some(Err(Error::Invalid(
                "Solid Multi Pipe does not accept inline arguments",
            )));
        }
        if feature == "SolidGVDMultiPipe"
            || (feature == "OM9-SOLID-011"
                && args.len() == 1
                && args[0]
                    .trim_start_matches('_')
                    .eq_ignore_ascii_case("multiple"))
        {
            return Some(self.dispatch("SolidGVDMultiPipe", InvocationOrigin::Cmd));
        }
        if feature == "OM9-SOLID-006" && args.len() == 1 && args[0].parse::<f64>().is_err() {
            self.command_run = self.command_run.wrapping_add(1);
            self.clear_subselection();
            self.solid_pending = SolidPending::start(feature);
            return self.submit_solid_text(args[0]);
        }
        if feature == "OM9-SOLID-004" {
            if args.len() > 1 {
                return Some(Err(Error::Invalid(
                    "Boolean Two Objects accepts one option at a time",
                )));
            }
            let result = self.dispatch(feature, InvocationOrigin::Cmd).and_then(|_| {
                if let Some(option) = args.first() {
                    self.submit_solid_text(option)
                        .ok_or(Error::Missing("Boolean session"))??;
                }
                Ok(())
            });
            return Some(result);
        }
        if matches!(
            feature,
            "OM9-SOLID-001"
                | "OM9-SOLID-002"
                | "OM9-SOLID-003"
                | "SolidBooleanSplit"
                | "SolidCreateSolid"
        ) {
            if args.len() > 1 {
                return Some(Err(Error::Invalid(
                    "Boolean command accepts one option at a time",
                )));
            }
            let result = self.dispatch(feature, InvocationOrigin::Cmd).and_then(|_| {
                if let Some(option) = args.first() {
                    self.submit_solid_text(option)
                        .ok_or(Error::Missing("Boolean session"))??;
                }
                Ok(())
            });
            return Some(result);
        }
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
            if matches!(feature, "OM9-SOLID-006" | "OM9-SOLID-007" | "OM9-SOLID-011") {
                self.command_run = self.command_run.wrapping_add(1);
                self.clear_subselection();
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        Some(self.dispatch_solid_args(feature, &args))
    }
}

#[cfg(feature = "occt-brep")]
impl Controller {
    pub fn dispatch_solid_args(&mut self, feature: &'static str, args: &[&str]) -> Result<()> {
        if matches!(
            feature,
            "OM9-SOLID-001"
                | "OM9-SOLID-002"
                | "OM9-SOLID-003"
                | "OM9-SOLID-004"
                | "OM9-SOLID-005"
                | "SolidBooleanSplit"
                | "SolidCreateSolid"
        ) {
            return Err(Error::Unsupported(
                "solid command requires selection stages",
            ));
        }
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
            "OM9-SOLID-005" => self.solid_cap(feature),
            "OM9-SOLID-006" => {
                if args.is_empty() || args.len() > 3 {
                    return Err(Error::Missing(usage(feature)));
                }
                let value = args[0]
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or(Error::Invalid("face index"))?;
                let mut copy = false;
                let mut input_layer = false;
                let mut seen_copy = false;
                let mut seen_layer = false;
                for option in &args[1..] {
                    match option.to_ascii_lowercase().as_str() {
                        "copy=yes" | "copy=true" if !seen_copy => {
                            copy = true;
                            seen_copy = true;
                        }
                        "copy=no" | "copy=false" if !seen_copy => {
                            copy = false;
                            seen_copy = true;
                        }
                        "outputlayer=input" if !seen_layer => {
                            input_layer = true;
                            seen_layer = true;
                        }
                        "outputlayer=current" if !seen_layer => {
                            input_layer = false;
                            seen_layer = true;
                        }
                        _ => return Err(Error::Invalid("invalid or duplicate extract option")),
                    }
                }
                self.solid_extract(feature, topology_index(value)?, copy, input_layer)
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
                let (dimensions, capped, round) =
                    match args.last().map(|s| s.to_ascii_lowercase()).as_deref() {
                        Some("cap=none") => (&args[..args.len() - 1], false, false),
                        Some("cap=flat") => (&args[..args.len() - 1], true, false),
                        Some("cap=round") => (&args[..args.len() - 1], true, true),
                        _ => (args, true, false),
                    };
                let radii: Result<Vec<f64>> = dimensions
                    .iter()
                    .map(|word| {
                        word.parse::<f64>()
                            .ok()
                            .filter(|v| v.is_finite() && *v > 0.0)
                            .ok_or(Error::Invalid("Pipe radius must be finite and positive"))
                    })
                    .collect();
                let radii = radii?;
                match radii.as_slice() {
                    [radius] if round => self.solid_pipe_round(feature, *radius, *radius),
                    [start, end] if round => self.solid_pipe_round(feature, *start, *end),
                    [a, b, c, d] if round => {
                        self.solid_pipe_thick(feature, [*a, *b, *c, *d], true, true)
                    }
                    [radius] => self.solid_pipe(feature, *radius, capped),
                    [start, end] => self.solid_pipe_linear(feature, *start, *end, capped),
                    [a, b, c, d] => self.solid_pipe_thick(feature, [*a, *b, *c, *d], capped, false),
                    _ => Err(Error::Missing(usage(feature))),
                }
            }
            "OM9-SOLID-012" => {
                let v = values(3)?;
                self.solid_primitive(
                    feature,
                    Primitive::Box {
                        x_mm: self.document().page_unit.length_to_mm(v[0])?,
                        y_mm: self.document().page_unit.length_to_mm(v[1])?,
                        z_mm: self.document().page_unit.length_to_mm(v[2])?,
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
                self.solid_primitive(
                    feature,
                    Primitive::Sphere {
                        radius_mm: self.document().page_unit.length_to_mm(v[0])?,
                    },
                )
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
                if v[0] < 3.0 || v[0].fract() != 0.0 || v[0] > i32::MAX as f64 {
                    return Err(Error::Invalid(
                        "pyramid sides must be an integer of at least three",
                    ));
                }
                let sides = v[0] as u64;
                if sides > self.coordinator.limits().max_points
                    || sides * 512 > self.coordinator.limits().max_bytes
                {
                    return Err(Error::Budget("pyramid side count"));
                }
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
                    if v[0] == v[1] {
                        Primitive::Cylinder {
                            radius_mm: v[0],
                            height_mm: v[2],
                        }
                    } else {
                        Primitive::Cone {
                            bottom_radius_mm: v[0],
                            top_radius_mm: v[1],
                            height_mm: v[2],
                        }
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
    pub fn accept_sphere_point(&mut self, _point: om9_core::geometry::Point3) -> Result<()> {
        Err(Error::Unsupported("Sphere requires OCCT"))
    }
    pub fn solid_box_dimensions(&mut self, _center: [f64; 3], _length: f64, _width: f64,
        _height: f64, _plane: super::CPlane) -> Result<()> {
        Err(Error::Unsupported("Box requires OCCT"))
    }
    pub fn box_height_ray_point(&self, _ray: &om9_scene::camera::Ray) -> Option<om9_core::geometry::Point3> {
        None
    }
    pub(super) fn solid_torus_points(
        &mut self,
        _center: [f64; 3],
        _frame: [[f64; 3]; 3],
        _ring: f64,
        _tube: f64,
    ) -> Result<()> {
        Err(Error::Unsupported("Torus requires OCCT"))
    }
    pub(super) fn solid_ellipsoid_points(
        &mut self,
        _center: [f64; 3],
        _axes: [[f64; 3]; 3],
        _radii: [f64; 3],
        _foci: Option<[[f64; 3]; 2]>,
    ) -> Result<()> {
        Err(Error::Unsupported("Ellipsoid requires OCCT"))
    }
    pub(super) fn solid_cone_commit(
        &mut self,
        _feature: &'static str,
        _center: om9_core::geometry::Point3,
        _plane: super::CPlane,
        _bottom: f64,
        _height: f64,
        _top: f64,
        _solid: bool,
    ) -> Result<()> {
        Err(Error::Unsupported("Truncated Cone requires OCCT"))
    }
    pub(super) fn solid_tube_oriented(
        &mut self,
        _center: [f64; 3],
        _plane: super::CPlane,
        _first: f64,
        _second: f64,
        _height: f64,
    ) -> Result<()> {
        Err(Error::Unsupported("Tube requires OCCT"))
    }
    pub(super) fn solid_pyramid_commit(
        &mut self,
        _center: [f64; 3],
        _plane: super::CPlane,
        _sides: u32,
        _radius: f64,
        _star_radius: Option<f64>,
        _height: f64,
        _solid: bool,
    ) -> Result<()> {
        Err(Error::Unsupported("Pyramid requires OCCT"))
    }
    pub(super) fn solid_fillet_edge_endpoints(
        &self,
        _reference: &super::SelectedSubobject,
    ) -> Result<Option<(om9_core::geometry::Point3, om9_core::geometry::Point3)>> {
        Err(Error::Unsupported("Fillet Edge requires OCCT"))
    }
    pub(super) fn solid_fillet_shared_endpoint_masks(
        &self,
        _edges: &[FilletEdgePick],
        _handle: usize,
    ) -> Result<Vec<u8>> {
        Err(Error::Unsupported("Fillet Edge requires OCCT"))
    }
    pub(super) fn solid_fillet_project_pick(
        &self,
        _reference: &super::SelectedSubobject,
        _placement: om9_core::geometry::Transform,
        _world: om9_core::geometry::Point3,
    ) -> Result<(f64, om9_core::geometry::Point3)> {
        Err(Error::Unsupported("Fillet Edge requires OCCT"))
    }
    pub(super) fn solid_shell_picks(
        &mut self,
        _faces: &[super::SelectedSubobject],
        _thickness: f64,
    ) -> Result<()> {
        Err(Error::Unsupported("Shell requires OCCT"))
    }
    pub(super) fn solid_fillet_build_preview(
        &self,
        _edges: &[FilletEdgePick],
        _controls: &[FilletControl],
    ) -> Result<FilletPreview> {
        Err(Error::Unsupported("Fillet Edge requires OCCT"))
    }
    pub(super) fn solid_fillet_commit(&mut self, _preview: &FilletPreview) -> Result<()> {
        Err(Error::Unsupported("Fillet Edge requires OCCT"))
    }
    pub(super) fn solid_extract_picks(
        &mut self,
        _picks: &[super::SelectedSubobject],
        _copy: bool,
        _input_layer: bool,
    ) -> Result<()> {
        Err(Error::Unsupported("Extract Surface requires OCCT"))
    }
    pub(super) fn solid_cap(&mut self, _feature: &'static str) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn solid_create_selected(&mut self, _delete_input: bool) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn solid_split_groups(
        &mut self,
        _targets: &[om9_core::identity::SourceStamp],
        _cutters: &[om9_core::identity::ObjectId],
        _delete_input: bool,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn boolean_two_build_preview(&self, _mode: usize) -> Result<BooleanTwoPreview> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn boolean_two_commit(
        &mut self,
        _preview: &BooleanTwoPreview,
        _delete_input: bool,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn solid_union_selected(&mut self, _delete_input: bool) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn solid_binary_groups(
        &mut self,
        _feature: &'static str,
        _targets: &[om9_core::identity::SourceStamp],
        _cutters: &[om9_core::identity::ObjectId],
        _delete_input: bool,
    ) -> Result<()> {
        Err(Error::Unsupported(
            "solid commands need the OCCT kernel feature",
        ))
    }
    pub(super) fn solid_sphere_points(
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
        _plane: super::CPlane,
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
    pub(super) fn solid_pyramid_commit(
        &mut self,
        center: [f64; 3],
        plane: super::CPlane,
        sides: u32,
        radius: f64,
        star_radius: Option<f64>,
        height: f64,
        solid: bool,
    ) -> Result<()> {
        let body = if let Some(second) = star_radius {
            creation::star_pyramid(
                sides,
                radius,
                second,
                height.abs(),
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
        } else {
            creation::primitive(
                Primitive::Pyramid {
                    sides,
                    base_radius_mm: radius,
                    height_mm: height.abs(),
                },
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
        }
        .map_err(|e| Error::Io(format!("pyramid: {e}")))?;
        let body = if solid {
            body
        } else {
            let original = creation::measure(&body, self.coordinator.limits())
                .map_err(|e| Error::Io(format!("pyramid: {e}")))?;
            let mut base = None;
            for index in 0..original.faces {
                let face = editing::extract_face(
                    &body,
                    index,
                    creation::DEFAULT_DEFLECTION_MM,
                    self.coordinator.limits(),
                )
                .map_err(|e| Error::Io(format!("pyramid face: {e}")))?;
                let m = creation::measure(&face, self.coordinator.limits())
                    .map_err(|e| Error::Io(format!("pyramid face: {e}")))?;
                if m.bounds_mm[5] - m.bounds_mm[2] < height.abs() * 0.25 {
                    if base.replace(index).is_some() {
                        return Err(Error::Invalid("ambiguous pyramid base"));
                    }
                }
            }
            let shell = editing::remove_face(
                &body,
                base.ok_or(Error::Missing("pyramid base"))?,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(format!("pyramid open shell: {e}")))?;
            let measured = creation::measure(&shell, self.coordinator.limits())
                .map_err(|e| Error::Io(format!("pyramid open shell: {e}")))?;
            let expected = sides as usize * if star_radius.is_some() { 2 } else { 1 };
            if measured.closed || measured.faces != expected || measured.naked_edges != expected {
                return Err(Error::Invalid("invalid pyramid open shell"));
            }
            shell
        };
        let mut entity = Entity::new(
            label("OM9-SOLID-018"),
            Geometry::Brep(body),
            self.document().current_layer,
        );
        let mut matrix = om9_core::geometry::Transform::identity().0;
        let normal = plane.normal();
        let sign = height.signum();
        for i in 0..3 {
            matrix[i][0] = plane.x[i];
            matrix[i][1] = plane.y[i] * sign;
            matrix[i][2] = normal[i] * sign;
            matrix[i][3] = center[i];
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity("OM9-SOLID-018", label("OM9-SOLID-018"), entity)
    }
    pub(super) fn solid_cone_commit(
        &mut self,
        feature: &'static str,
        center: om9_core::geometry::Point3,
        plane: super::CPlane,
        bottom: f64,
        height: f64,
        top: f64,
        solid: bool,
    ) -> Result<()> {
        let primitive = if bottom == top {
            Primitive::Cylinder {
                radius_mm: bottom,
                height_mm: height.abs(),
            }
        } else {
            Primitive::Cone {
                bottom_radius_mm: bottom,
                top_radius_mm: top,
                height_mm: height.abs(),
            }
        };
        let body = creation::primitive(
            primitive,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|e| Error::Io(format!("Truncated Cone: {e}")))?;
        let body = if solid {
            body
        } else {
            let original = creation::measure(&body, self.coordinator.limits())
                .map_err(|e| Error::Io(format!("Truncated Cone: {e}")))?;
            let mut candidate = None;
            for index in 0..original.faces {
                let face = editing::extract_face(
                    &body,
                    index,
                    creation::DEFAULT_DEFLECTION_MM,
                    self.coordinator.limits(),
                )
                .map_err(|e| Error::Io(format!("Truncated Cone face: {e}")))?;
                let m = creation::measure(&face, self.coordinator.limits())
                    .map_err(|e| Error::Io(format!("Truncated Cone face: {e}")))?;
                if m.bounds_mm[5] - m.bounds_mm[2] > height.abs() * 0.5 {
                    if candidate.is_some() {
                        return Err(Error::Invalid("ambiguous Truncated Cone lateral face"));
                    }
                    candidate = Some((face, m));
                }
            }
            let (face, m) =
                candidate.ok_or(Error::Invalid("missing Truncated Cone lateral face"))?;
            let expected =
                std::f64::consts::PI * (bottom + top) * (height.abs().hypot(bottom - top));
            if !expected.is_finite()
                || (m.area_mm2 - expected).abs() > 1e-6 * expected.max(1.)
                || m.solids != 0
                || m.faces != 1
                || m.naked_edges != if top == 0.0 { 1 } else { 2 }
                || m.closed
            {
                return Err(Error::Invalid("invalid Truncated Cone lateral face"));
            }
            face
        };
        let sign = height.signum();
        let n = plane.normal();
        let mut matrix = om9_core::geometry::Transform::identity().0;
        for i in 0..3 {
            matrix[i][0] = plane.x[i];
            matrix[i][1] = plane.y[i] * sign;
            matrix[i][2] = n[i] * sign;
            matrix[i][3] = center[i];
        }
        let mut entity = Entity::new(
            label(feature),
            Geometry::Brep(body),
            self.document().current_layer,
        );
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity(feature, label(feature), entity)
    }
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
        let radius =
            ((center[0] - a[0]).powi(2) + (center[1] - a[1]).powi(2) + (center[2] - a[2]).powi(2))
                .sqrt();
        Ok((
            center,
            radius,
            [n[0] / nn.sqrt(), n[1] / nn.sqrt(), n[2] / nn.sqrt()],
        ))
    }

    /// The point under the pointer for a Box height step. The usual construction-plane
    /// intersection pins z to the plane, so clicking for a height would always report zero;
    /// this intersects a vertical plane through the first base point instead.
    pub fn box_height_ray_point(
        &self,
        ray: &om9_scene::camera::Ray,
    ) -> Option<om9_core::geometry::Point3> {
        let pending = self.solid_pending.as_ref()?;
        if pending.feature == "SolidSlab" && pending.slab_offset.is_some() {
            let stamp = pending.slab_source?;
            if self.document().stamp(stamp.object).ok()? != stamp { return None; }
            let entity = self.document().entities.get(&stamp.object)?;
            let geometry = SlabSourceGeometry::capture(entity, self.coordinator.limits()).ok()?;
            let anchor = geometry.anchor();
            let axis = pending.slab_normal;
            let along = (0..3).map(|i| ray.direction[i]*axis[i]).sum::<f64>();
            let normal = std::array::from_fn::<_,3,_>(|i| ray.direction[i]-along*axis[i]);
            let denominator = (0..3).map(|i| ray.direction[i]*normal[i]).sum::<f64>();
            if denominator <= 1e-12 { return None; }
            let t = (0..3).map(|i| (anchor[i]-ray.origin[i])*normal[i]).sum::<f64>() / denominator;
            return (t > 0.0).then(|| std::array::from_fn(|i| ray.origin[i]+ray.direction[i]*t));
        }
        if pending.feature == "OM9-SOLID-015"
            && pending.points.len() == if pending.torus_three_point { 3 } else { 2 }
        {
            let axis = pending.cone_plane.as_deref()?.plane.normal();
            let along = (0..3).map(|i| ray.direction[i] * axis[i]).sum::<f64>();
            let normal = std::array::from_fn::<_, 3, _>(|i| ray.direction[i] - along * axis[i]);
            let denominator = (0..3).map(|i| ray.direction[i] * normal[i]).sum::<f64>();
            if denominator > 1e-12 {
                let t = (0..3)
                    .map(|i| {
                        (pending.points.last().expect("torus ring point")[i] - ray.origin[i])
                            * normal[i]
                    })
                    .sum::<f64>()
                    / denominator;
                if t > 0.0 {
                    return Some(std::array::from_fn(|i| {
                        ray.origin[i] + ray.direction[i] * t
                    }));
                }
            }
        }
        if pending.feature == "OM9-SOLID-013"
            && (pending.points.len() == 3
                || (pending.ellipsoid_corner && pending.points.len() == 2))
        {
            let axis = pending.cone_plane.as_deref()?.plane.normal();
            let along = (0..3).map(|i| ray.direction[i] * axis[i]).sum::<f64>();
            let normal = std::array::from_fn::<_, 3, _>(|i| ray.direction[i] - along * axis[i]);
            let denominator = (0..3).map(|i| ray.direction[i] * normal[i]).sum::<f64>();
            if denominator <= 1e-12 {
                return None;
            }
            let center = pending.ellipsoid_center();
            let t = (0..3)
                .map(|i| (center[i] - ray.origin[i]) * normal[i])
                .sum::<f64>()
                / denominator;
            if t <= 0.0 {
                return None;
            }
            return Some(std::array::from_fn(|i| {
                ray.origin[i] + ray.direction[i] * t
            }));
        }
        if ((matches!(
            pending.feature,
            "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019"
        ) && pending.values.len() == 1)
            || (pending.feature == "OM9-SOLID-018"
                && pending.values.len() == if pending.cone_base_points == 3 { 2 } else { 1 })
            || (pending.feature == "OM9-SOLID-017" && pending.values.len() == 2))
            && !pending.points.is_empty()
        {
            let frame = pending.cone_plane.as_deref()?.plane;
            let axis = frame.normal();
            let along = (0..3).map(|i| ray.direction[i] * axis[i]).sum::<f64>();
            let normal = std::array::from_fn::<_, 3, _>(|i| ray.direction[i] - along * axis[i]);
            let denominator = (0..3).map(|i| ray.direction[i] * normal[i]).sum::<f64>();
            if denominator <= 1e-12 {
                return None;
            }
            let center = if pending.feature == "OM9-SOLID-018" {
                pending.pyramid_center()
            } else {
                pending.cone_center()
            };
            let t = (0..3)
                .map(|i| (center[i] - ray.origin[i]) * normal[i])
                .sum::<f64>()
                / denominator;
            if t <= 0. {
                return None;
            }
            return Some(std::array::from_fn(|i| {
                ray.origin[i] + ray.direction[i] * t
            }));
        }
        if !pending.box_mode {
            return None;
        }
        if pending.box_kind == "Vertical"
            && pending.points.len() >= 2
            && pending.box_width.is_none()
        {
            let first = pending.points[0];
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let normal = box_edge_axes(first, pending.points[1], plane, true).ok()?.1[2];
            let denominator = (0..3).map(|i| ray.direction[i] * normal[i]).sum::<f64>();
            if denominator.abs() <= 1e-12 {
                return None;
            }
            let t = (0..3)
                .map(|i| (first[i] - ray.origin[i]) * normal[i])
                .sum::<f64>()
                / denominator;
            if t <= 0. {
                return None;
            }
            return Some(std::array::from_fn(|i| {
                ray.origin[i] + ray.direction[i] * t
            }));
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
        let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
        let axis = if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
            box_edge_axes(
                first,
                pending.points[1],
                plane,
                pending.box_kind == "Vertical",
            )
            .ok()?
            .1[2]
        } else {
            plane.normal()
        };
        let along = (0..3).map(|i| ray.direction[i] * axis[i]).sum::<f64>();
        let mut normal = std::array::from_fn::<_, 3, _>(|i| ray.direction[i] - along * axis[i]);
        let length = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if length <= 1e-9 {
            normal = plane.y;
        } else {
            normal = normal.map(|v| v / length);
        }
        let denominator = ray.direction[0] * normal[0]
            + ray.direction[1] * normal[1]
            + ray.direction[2] * normal[2];
        if denominator.abs() <= 1e-12 {
            return None;
        }
        let offset = [
            first[0] - ray.origin[0],
            first[1] - ray.origin[1],
            first[2] - ray.origin[2],
        ];
        let t =
            (offset[0] * normal[0] + offset[1] * normal[1] + offset[2] * normal[2]) / denominator;
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
        let input_plane = self.input_plane();
        #[allow(unused_mut)]
        let Some(mut pending) = self.solid_pending.take() else {
            return Err(Error::Missing("sphere command"));
        };
        if !pending.sphere_mode {
            self.solid_pending = Some(pending);
            return Err(Error::Missing("sphere command"));
        }
        if pending.sphere_sources.iter().any(|stamp| self.document().stamp(stamp.object).ok().as_ref() != Some(stamp)) {
            self.solid_pending = Some(pending);
            return Err(Error::Stale("Sphere curve source"));
        }
        if pending.sphere_plane.is_none() {
            pending.sphere_plane = Some(input_plane);
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
                let center = [
                    (a[0] + b[0]) * 0.5,
                    (a[1] + b[1]) * 0.5,
                    (a[2] + b[2]) * 0.5,
                ];
                let radius =
                    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
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
                        norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
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
                    let Some(entity) = pending.sphere_sources.first().and_then(|stamp| self.document().entities.get(&stamp.object)) else {
                        self.solid_pending = Some(pending);
                        return Err(Error::Invalid("AroundCurve has no source curve"));
                    };
                    let source = match sphere_curve_source(entity, pending.sphere_edge, self.coordinator.limits()) {
                        Ok(source) => source, Err(error) => { self.solid_pending = Some(pending); return Err(error); }
                    };
                    let center = match sphere_closest_world_curve_point(&source, point, self.coordinator.limits()) {
                        Ok(center) => center,
                        Err(error) => { self.solid_pending = Some(pending); return Err(error); }
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
                if pending.sphere_kind == "ThreePoint" {
                    if pending.points.len() == 2 {
                        if pending.sphere_radius < 0.0 {
                            self.solid_pending = Some(pending);
                            return Err(Error::Missing("enter sphere radius"));
                        }
                        if pending.sphere_radius > 0.0 {
                            let center = match sphere_chord_direction(
                                pending.points[0],
                                pending.points[1],
                                pending.sphere_radius,
                                point,
                            ) {
                                Ok(center) => center,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Err(error);
                                }
                            };
                            let result = self.solid_sphere_points(center, pending.sphere_radius);
                            if result.is_err() {
                                self.solid_pending = Some(pending);
                            }
                            return result;
                        }
                        let result =
                            Self::sphere_circumcenter(pending.points[0], pending.points[1], point);
                        let (center, radius, _) = match result {
                            Ok(sphere) => sphere,
                            Err(error) => {
                                self.solid_pending = Some(pending);
                                return Err(error);
                            }
                        };
                        self.solid_pending = None;
                        return self.solid_sphere_points(center, radius);
                    }
                    pending.points.push(point);
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
                if pending.sphere_kind == "FourPoint" {
                    match pending.points.len() {
                        2 => {
                            let result = Self::sphere_circumcenter(
                                pending.points[0],
                                pending.points[1],
                                point,
                            );
                            let (center, radius, normal) = match result {
                                Ok(sphere) => sphere,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Err(error);
                                }
                            };
                            pending.points.push(point);
                            pending.sphere_center = Some(center);
                            pending.sphere_radius = radius;
                            pending.sphere_normal = normal;
                            self.solid_pending = Some(pending);
                            return Ok(());
                        }
                        3 => {
                            let center = pending.sphere_center.expect("circle center");
                            let (result, result_radius) = match Self::sphere_fourth_point(
                                center,
                                pending.sphere_radius,
                                pending.sphere_normal,
                                point,
                            ) {
                                Ok(sphere) => sphere,
                                Err(error) => {
                                    self.solid_pending = Some(pending);
                                    return Err(error);
                                }
                            };
                            self.solid_pending = None;
                            return self.solid_sphere_points(result, result_radius);
                        }
                        _ => {}
                    }
                    pending.points.push(point);
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

    pub(super) fn fit_sphere_points(&mut self, points: &[om9_core::geometry::Point3]) -> Result<()> {
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
                .max_by(|a, b| {
                    normal[*a][column]
                        .abs()
                        .total_cmp(&normal[*b][column].abs())
                })
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
        let radius_squared =
            normal[3][4] + center[0] * center[0] + center[1] * center[1] + center[2] * center[2];
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
        plane: super::CPlane,
    ) -> Result<()> {
        if [length, width]
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
            || !height.is_finite()
            || height == 0.0
        {
            return Err(Error::Invalid(
                "box length and width must be positive and height non-zero",
            ));
        }
        let brep = creation::primitive(
            Primitive::Box {
                x_mm: length,
                y_mm: width,
                z_mm: height.abs(),
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
        let axes = [plane.x, plane.y, plane.normal()];
        let mut matrix = om9_core::geometry::Transform::identity().0;
        for i in 0..3 {
            for j in 0..3 {
                matrix[i][j] = axes[j][i];
            }
            matrix[i][3] = center[i] - axes[0][i] * length * 0.5 - axes[1][i] * width * 0.5
                + axes[2][i] * height.min(0.0);
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity("OM9-SOLID-012", label("OM9-SOLID-012"), entity)
    }

    pub fn solid_box_points(
        &mut self,
        first: om9_core::geometry::Point3,
        second: om9_core::geometry::Point3,
        height: f64,
        kind: &str,
        width_override: Option<f64>,
        plane: super::CPlane,
    ) -> Result<()> {
        let world_first = first;
        let frame_axes = [plane.x, plane.y, plane.normal()];
        let world_delta = std::array::from_fn::<_, 3, _>(|i| second[i] - first[i]);
        let delta = std::array::from_fn::<_, 3, _>(|j| {
            (0..3).map(|i| world_delta[i] * frame_axes[j][i]).sum()
        });
        let first = [0., 0., 0.];
        let second = delta;
        let signed_height = if height.is_finite() && height != 0.0 {
            height
        } else {
            return Err(Error::Invalid("box height must be non-zero"));
        };
        let (dims, axes, origin) = if kind == "Cube" {
            let (axes, side) =
                cube_axes([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], delta)?;
            ([side, side, side], axes, first)
        } else if matches!(kind, "ThreePoint" | "Vertical") {
            let length = delta[0].hypot(delta[1]);
            if length <= 0.0 {
                return Err(Error::Invalid("box needs a non-zero edge"));
            }
            let x = [delta[0] / length, delta[1] / length, 0.0];
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
                    return Err(Error::Invalid(
                        "box edge is parallel to the construction plane",
                    ));
                }
                y = [y[0] / y_len, y[1] / y_len, y[2] / y_len];
                let z = [
                    x[1] * y[2] - x[2] * y[1],
                    x[2] * y[0] - x[0] * y[2],
                    x[0] * y[1] - x[1] * y[0],
                ];
                (y, z)
            };
            let signed_width = width_override.unwrap_or(length);
            (
                [length, signed_width.abs(), signed_height.abs()],
                [x, y, z],
                std::array::from_fn(|i| first[i] + y[i] * signed_width.min(0.0)),
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
                    [first[0].min(second[0]), first[1].min(second[1]), first[2]]
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
                matrix[row][column] = (0..3).map(|j| frame_axes[j][row] * axes[column][j]).sum();
            }
        }
        for row in 0..3 {
            matrix[row][3] =
                world_first[row] + (0..3).map(|j| frame_axes[j][row] * origin[j]).sum::<f64>();
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity("OM9-SOLID-012", label("OM9-SOLID-012"), entity)
    }

    pub(super) fn solid_primitive(&mut self, feature: &'static str, primitive: Primitive) -> Result<()> {
        let brep = creation::primitive(
            primitive,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("solid primitive: {error}")))?;
        self.insert_solid(feature, label(feature), brep)
    }

    pub(super) fn solid_torus_points(
        &mut self,
        center: [f64; 3],
        frame: [[f64; 3]; 3],
        ring: f64,
        tube: f64,
    ) -> Result<()> {
        let brep = creation::primitive(
            Primitive::Torus {
                ring_radius_mm: ring,
                tube_radius_mm: tube,
            },
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|e| Error::Io(format!("torus: {e}")))?;
        let mut entity = Entity::new(
            label("OM9-SOLID-015"),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut matrix = om9_core::geometry::Transform::identity().0;
        for row in 0..3 {
            for col in 0..3 {
                matrix[row][col] = frame[col][row];
            }
            matrix[row][3] = center[row];
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity("OM9-SOLID-015", label("OM9-SOLID-015"), entity)
    }

    pub(super) fn solid_ellipsoid_points(
        &mut self,
        center: [f64; 3],
        axes: [[f64; 3]; 3],
        radii: [f64; 3],
        foci: Option<[[f64; 3]; 2]>,
    ) -> Result<()> {
        let brep = creation::primitive(
            Primitive::Ellipsoid {
                x_mm: radii[0],
                y_mm: radii[1],
                z_mm: radii[2],
            },
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("ellipsoid: {error}")))?;
        let mut entity = Entity::new(
            label("OM9-SOLID-013"),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut matrix = om9_core::geometry::Transform::identity().0;
        for row in 0..3 {
            for column in 0..3 {
                matrix[row][column] = axes[column][row];
            }
            matrix[row][3] = center[row];
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        if let Some(foci) = foci {
            let mut mutations = vec![Mutation::Insert {
                document: self.document,
                entity,
            }];
            for focus in foci {
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: Entity::new(
                        "Ellipsoid focus",
                        Geometry::Point(focus),
                        self.document().current_layer,
                    ),
                });
            }
            return self.commit_solid_result(
                "OM9-SOLID-013",
                label("OM9-SOLID-013"),
                mutations,
                None,
            );
        }
        self.commit_entity("OM9-SOLID-013", label("OM9-SOLID-013"), entity)
    }

    fn world_solid_brep(&self, id: om9_core::identity::ObjectId) -> Result<om9_core::brep::Brep> {
        let brep = self.solid_brep(id)?;
        let entity = &self.document().entities[&id];
        if entity.placement == om9_core::geometry::Transform::identity() {
            Ok(brep.clone())
        } else {
            om9_kernel::transform_ops::transform_brep(
                brep,
                &entity.placement,
                brep.display.deflection_mm,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("solid placement: {error}")))
        }
    }

    pub(super) fn commit_solid_result(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mutations: Vec<Mutation>,
        select: Option<Vec<om9_core::identity::ObjectId>>,
    ) -> Result<()> {
        self.commit_solid_result_with_extra(feature, name, mutations, select, None)
    }

    fn commit_solid_result_with_extra(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mutations: Vec<Mutation>,
        select: Option<Vec<om9_core::identity::ObjectId>>,
        extra: Option<om9_core::budget::Usage>,
    ) -> Result<()> {
        let mut held = self.auxiliary_usage()?;
        if let Some(extra) = extra {
            held.add(extra, self.coordinator.limits())?;
        }
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

    pub(super) fn solid_create_selected(&mut self, delete_input: bool) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("select intersecting B-Rep surfaces"));
        }
        let mut sources = Vec::new();
        let mut inputs = Vec::new();
        sources
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("create solid inputs"))?;
        inputs
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("create solid inputs"))?;
        for id in &ids {
            sources.push(self.document().stamp(*id)?);
            inputs.push(self.world_solid_brep(*id)?);
        }
        let held = self
            .coordinator
            .retained_usage_with_held_usage(self.scene_cache_usage()?)?;
        let remaining = self.coordinator.limits().remaining_after(held)?;
        let deflection = inputs[0].display.deflection_mm;
        let regions = editing::create_solid_regions(&inputs, deflection, &remaining)
            .map_err(|e| Error::Io(format!("create solid: {e}")))?
            .ok_or(Error::Invalid("no closed solid regions"))?;
        let count = regions
            .pieces
            .checked_add(if delete_input { sources.len() } else { 0 })
            .ok_or(Error::Overflow("create solid mutations"))?;
        let mut mutations = Vec::new();
        let mut outputs = Vec::new();
        mutations
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("create solid mutations"))?;
        outputs
            .try_reserve_exact(regions.pieces)
            .map_err(|_| Error::Budget("create solid outputs"))?;
        if delete_input {
            for source in sources {
                mutations.push(Mutation::Delete { source });
            }
        }
        for index in 0..regions.pieces {
            let body = editing::split_solid_piece(&regions, index, deflection, &remaining)
                .map_err(|e| Error::Io(format!("create solid region: {e}")))?;
            let entity = Entity::new(
                "Create Solid",
                Geometry::Brep(body),
                self.document().current_layer,
            );
            outputs.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_solid_result("SolidCreateSolid", "Create Solid", mutations, Some(outputs))
    }

    pub(super) fn solid_split_groups(
        &mut self,
        targets: &[om9_core::identity::SourceStamp],
        cutters: &[om9_core::identity::ObjectId],
        delete_input: bool,
    ) -> Result<()> {
        if targets.is_empty() || cutters.is_empty() {
            return Err(Error::Missing("select the second Boolean set"));
        }
        let mut target_bodies = Vec::new();
        let mut cutter_bodies = Vec::new();
        let mut sources = std::collections::BTreeMap::new();
        target_bodies
            .try_reserve_exact(targets.len())
            .map_err(|_| Error::Budget("split targets"))?;
        cutter_bodies
            .try_reserve_exact(cutters.len())
            .map_err(|_| Error::Budget("split cutters"))?;
        for source in targets {
            if self.document().stamp(source.object).ok().as_ref() != Some(source) {
                return Err(Error::Stale("Boolean first set"));
            }
            sources.insert(source.object, *source);
            target_bodies.push(self.world_solid_brep(source.object)?);
        }
        for id in cutters {
            let source = self.document().stamp(*id)?;
            sources.insert(*id, source);
            cutter_bodies.push(self.world_solid_brep(*id)?);
        }
        let deflection = target_bodies[0].display.deflection_mm;
        let held = self
            .coordinator
            .retained_usage_with_held_usage(self.scene_cache_usage()?)?;
        let remaining = self.coordinator.limits().remaining_after(held)?;
        let pieces =
            editing::split_solid_sets(&target_bodies, &cutter_bodies, deflection, &remaining)
                .map_err(|error| Error::Io(format!("solid split: {error}")))?;
        if pieces.is_empty() {
            return Err(Error::Invalid("Boolean Split produced no pieces"));
        }
        let count = pieces
            .len()
            .checked_add(if delete_input { sources.len() } else { 0 })
            .ok_or(Error::Overflow("Boolean Split mutations"))?;
        let mut mutations = Vec::new();
        let mut outputs = Vec::new();
        mutations
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("split mutations"))?;
        outputs
            .try_reserve_exact(pieces.len())
            .map_err(|_| Error::Budget("split outputs"))?;
        if delete_input {
            for source in sources.into_values() {
                mutations.push(Mutation::Delete { source });
            }
        }
        for body in pieces {
            let entity = Entity::new(
                "Boolean Split",
                Geometry::Brep(body),
                self.document().current_layer,
            );
            outputs.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_solid_result(
            "SolidBooleanSplit",
            "Boolean Split",
            mutations,
            Some(outputs),
        )
    }

    pub(super) fn solid_binary_groups(
        &mut self,
        feature: &'static str,
        targets: &[om9_core::identity::SourceStamp],
        cutters: &[om9_core::identity::ObjectId],
        delete_input: bool,
    ) -> Result<()> {
        let op = match feature {
            "OM9-SOLID-001" => BooleanOp::Difference,
            "OM9-SOLID-002" => BooleanOp::Intersection,
            _ => return Err(Error::Invalid("unsupported binary Boolean feature")),
        };
        if targets.is_empty() || cutters.is_empty() {
            return Err(Error::Missing("select the second Boolean set"));
        }
        for target in targets {
            if self.document().stamp(target.object).ok().as_ref() != Some(target) {
                return Err(Error::Stale("Boolean first set"));
            }
            self.solid_brep(target.object)?;
        }
        let mut cutter_stamps = Vec::new();
        cutter_stamps
            .try_reserve_exact(cutters.len())
            .map_err(|_| Error::Budget("boolean cutters"))?;
        for id in cutters {
            if targets.iter().any(|target| target.object == *id) {
                return Err(Error::Invalid(
                    "an object cannot belong to both Boolean sets",
                ));
            }
            self.solid_brep(*id)?;
            cutter_stamps.push(self.document().stamp(*id)?);
        }
        let delete_targets = feature == "OM9-SOLID-001" || delete_input;
        let count = (if delete_targets { targets.len() } else { 0 })
            .checked_add(if delete_input { cutters.len() } else { 0 })
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Overflow("boolean mutations"))?;
        let mut mutations = Vec::new();
        mutations
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("boolean mutations"))?;
        let mut target_body = self.world_solid_brep(targets[0].object)?;
        let deflection = target_body.display.deflection_mm;
        for target in &targets[1..] {
            let operand = self.world_solid_brep(target.object)?;
            target_body = match editing::boolean(
                &target_body,
                &operand,
                BooleanOp::Union,
                deflection,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("solid target union: {error}")))?
            {
                editing::BooleanOutcome::Body(body) => body,
                editing::BooleanOutcome::Empty => {
                    return Err(Error::Invalid("target union produced an empty body"));
                }
            };
        }
        let mut cutter_body = self.world_solid_brep(cutter_stamps[0].object)?;
        for cutter in &cutter_stamps[1..] {
            let operand = self.world_solid_brep(cutter.object)?;
            cutter_body = match editing::boolean(
                &cutter_body,
                &operand,
                BooleanOp::Union,
                deflection,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("solid cutter union: {error}")))?
            {
                editing::BooleanOutcome::Body(body) => body,
                editing::BooleanOutcome::Empty => {
                    return Err(Error::Invalid("cutter union produced an empty body"));
                }
            };
        }
        let body = match editing::boolean(
            &target_body,
            &cutter_body,
            op,
            deflection,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("solid boolean: {error}")))?
        {
            editing::BooleanOutcome::Body(body) => body,
            editing::BooleanOutcome::Empty => {
                return Err(Error::Invalid("boolean produced an empty body"));
            }
        };
        if delete_targets {
            for target in targets {
                mutations.push(Mutation::Delete { source: *target });
            }
        }
        if delete_input {
            for cutter in cutter_stamps {
                mutations.push(Mutation::Delete { source: cutter });
            }
        }
        let entity = Entity::new(
            label(feature),
            Geometry::Brep(body),
            self.document().current_layer,
        );
        let output = entity.id;
        mutations.push(Mutation::Insert {
            document: self.document,
            entity,
        });
        self.commit_solid_result(feature, label(feature), mutations, Some(vec![output]))
    }

    pub(super) fn solid_union_selected(&mut self, delete_input: bool) -> Result<()> {
        self.solid_boolean("OM9-SOLID-003", BooleanOp::Union, delete_input)
    }

    pub(super) fn boolean_two_build_preview(&self, mode: usize) -> Result<BooleanTwoPreview> {
        let ids = self.selected_ordered();
        if ids.len() != 2 {
            return Err(Error::Missing("select exactly two B-Rep solids"));
        }
        let sources = [
            self.document().stamp(ids[0])?,
            self.document().stamp(ids[1])?,
        ];
        let a = self.world_solid_brep(ids[0])?;
        let b = self.world_solid_brep(ids[1])?;
        let deflection = a.display.deflection_mm;
        let limits = self.coordinator.limits();
        let calculate = |left: &om9_core::brep::Brep, right: &om9_core::brep::Brep, op| {
            editing::boolean(left, right, op, deflection, limits)
                .map_err(|error| Error::Io(format!("solid boolean: {error}")))
        };
        let result = match mode {
            0 => calculate(&a, &b, BooleanOp::Union)?,
            1 => calculate(&a, &b, BooleanOp::Difference)?,
            2 => calculate(&b, &a, BooleanOp::Difference)?,
            3 => calculate(&a, &b, BooleanOp::Intersection)?,
            4 => {
                match (
                    calculate(&a, &b, BooleanOp::Difference)?,
                    calculate(&b, &a, BooleanOp::Difference)?,
                ) {
                    (editing::BooleanOutcome::Empty, editing::BooleanOutcome::Empty) => {
                        editing::BooleanOutcome::Empty
                    }
                    (editing::BooleanOutcome::Body(body), editing::BooleanOutcome::Empty)
                    | (editing::BooleanOutcome::Empty, editing::BooleanOutcome::Body(body)) => {
                        editing::BooleanOutcome::Body(body)
                    }
                    (editing::BooleanOutcome::Body(left), editing::BooleanOutcome::Body(right)) => {
                        calculate(&left, &right, BooleanOp::Union)?
                    }
                }
            }
            _ => return Err(Error::Invalid("Boolean preview mode")),
        };
        let (body, usage) = match result {
            editing::BooleanOutcome::Body(body) => {
                let usage = body.validate(limits)?;
                (Some(body), usage)
            }
            editing::BooleanOutcome::Empty => (None, om9_core::budget::Usage::default()),
        };
        let mut held = self.scene_cache_usage()?;
        held.add(usage, limits)?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        Ok(BooleanTwoPreview {
            mode,
            sources,
            body,
            usage,
        })
    }

    pub(super) fn boolean_two_commit(
        &mut self,
        preview: &BooleanTwoPreview,
        delete_input: bool,
    ) -> Result<()> {
        for source in &preview.sources {
            if self.document().stamp(source.object).ok().as_ref() != Some(source) {
                return Err(Error::Stale("Boolean input"));
            }
        }
        let body = preview
            .body
            .as_ref()
            .ok_or(Error::Invalid("boolean produced an empty body"))?;
        let entity = Entity::new(
            "Boolean Two Objects",
            Geometry::Brep(body.clone()),
            self.document().current_layer,
        );
        let output = entity.id;
        let mut mutations = Vec::new();
        if delete_input {
            for source in preview.sources {
                mutations.push(Mutation::Delete { source });
            }
        }
        mutations.push(Mutation::Insert {
            document: self.document,
            entity,
        });
        self.commit_solid_result_with_extra(
            "OM9-SOLID-004",
            "Boolean Two Objects",
            mutations,
            Some(vec![output]),
            Some(preview.usage),
        )
    }

    pub(super) fn solid_boolean(
        &mut self,
        feature: &'static str,
        op: BooleanOp,
        delete_input: bool,
    ) -> Result<()> {
        let selected = self.selected_ordered();
        let multi_union = feature == "OM9-SOLID-003";
        if selected.len() < 2 || (!multi_union && selected.len() != 2) {
            if multi_union {
                return Err(Error::Missing("select at least two B-Rep solids"));
            }
            return Err(Error::Missing("select exactly two B-Rep solids"));
        }
        for id in &selected {
            self.solid_brep(*id)?;
        }
        let mut brep = self.world_solid_brep(selected[0])?;
        let deflection = brep.display.deflection_mm;
        for id in &selected[1..] {
            let operand = self.world_solid_brep(*id)?;
            let outcome =
                editing::boolean(&brep, &operand, op, deflection, self.coordinator.limits())
                    .map_err(|error| Error::Io(format!("solid boolean: {error}")))?;
            brep = match outcome {
                editing::BooleanOutcome::Body(body) => body,
                editing::BooleanOutcome::Empty => {
                    return Err(Error::Invalid("boolean produced an empty body"));
                }
            };
        }
        let entity = Entity::new(
            label(feature),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let result_id = entity.id;
        let mut mutations = Vec::new();
        if delete_input {
            for id in &selected {
                mutations.push(Mutation::Delete {
                    source: self.document().stamp(*id)?,
                });
            }
        }
        mutations.push(Mutation::Insert {
            document: self.document,
            entity,
        });
        self.commit_solid_result(feature, label(feature), mutations, Some(vec![result_id]))
    }

    pub(super) fn solid_cap(&mut self, feature: &'static str) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("select surfaces or polysurfaces to cap"));
        }
        let mut mutations = Vec::new();
        mutations
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("cap inputs"))?;
        let mut held = self.scene_cache_usage()?;
        for id in &ids {
            let source = self.document().stamp(*id)?;
            let brep = self.solid_brep(*id)?;
            let capped = editing::cap_planar(
                brep,
                1e-7,
                brep.display.deflection_mm,
                self.coordinator.limits(),
            )
            .map_err(|error| Error::Io(format!("cap planar: {error}")))?;
            held.add(
                capped.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            mutations.push(Mutation::Geometry {
                source,
                geometry: Geometry::Brep(capped),
            });
        }
        self.commit_solid_result(feature, label(feature), mutations, Some(ids))
    }

    pub(super) fn solid_extract(
        &mut self,
        feature: &'static str,
        face: usize,
        copy: bool,
        input_layer: bool,
    ) -> Result<()> {
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
        let source = self.document().stamp(id)?;
        let placement = self.document().entities[&id].placement;
        let layer = if input_layer {
            self.document().entities[&id].attributes.layer
        } else {
            self.document().current_layer
        };
        let mut entity = Entity::new(label(feature), Geometry::Brep(extracted), layer);
        entity.placement = placement;
        let result_id = entity.id;
        let mut mutations = Vec::new();
        if !copy {
            if brep.display.faces.len() == 1 {
                mutations.push(Mutation::Delete { source });
            } else {
                let remainder = editing::remove_face(
                    brep,
                    face,
                    brep.display.deflection_mm,
                    self.coordinator.limits(),
                )
                .map_err(|error| Error::Io(format!("extract surface remainder: {error}")))?;
                mutations.push(Mutation::Geometry {
                    source,
                    geometry: Geometry::Brep(remainder),
                });
            }
        }
        mutations.push(Mutation::Insert {
            document: self.document,
            entity,
        });
        self.commit_solid_result(feature, label(feature), mutations, Some(vec![result_id]))
    }

    pub(super) fn solid_extract_picks(
        &mut self,
        picks: &[super::SelectedSubobject],
        copy: bool,
        input_layer: bool,
    ) -> Result<()> {
        let mut groups: std::collections::BTreeMap<om9_core::identity::ObjectId, Vec<usize>> =
            std::collections::BTreeMap::new();
        for pick in picks {
            let om9_scene::scene::Subelement::BrepFace(face) = pick.subset else {
                return Err(Error::Invalid("select B-Rep faces"));
            };
            let brep = self.solid_brep(pick.root)?;
            let index = brep
                .display
                .faces
                .iter()
                .position(|id| *id == face)
                .ok_or(Error::Stale("B-Rep face"))?;
            let group = groups.entry(pick.root).or_default();
            group
                .try_reserve(1)
                .map_err(|_| Error::Budget("extract indices"))?;
            group.push(index);
        }
        let mut mutations = Vec::new();
        mutations
            .try_reserve(picks.len() + groups.len())
            .map_err(|_| Error::Budget("extract mutations"))?;
        let mut output_ids = Vec::new();
        output_ids
            .try_reserve(picks.len())
            .map_err(|_| Error::Budget("extract outputs"))?;
        let mut held = self.scene_cache_usage()?;
        for (id, indices) in groups {
            let source = self.document().stamp(id)?;
            let brep = self.solid_brep(id)?;
            let entity = &self.document().entities[&id];
            let layer = if input_layer {
                entity.attributes.layer
            } else {
                self.document().current_layer
            };
            for index in &indices {
                let face = editing::extract_face(
                    brep,
                    *index,
                    brep.display.deflection_mm,
                    self.coordinator.limits(),
                )
                .map_err(|error| Error::Io(format!("extract surface: {error}")))?;
                held.add(
                    face.validate(self.coordinator.limits())?,
                    self.coordinator.limits(),
                )?;
                self.coordinator.retained_usage_with_held_usage(held)?;
                let mut output = Entity::new(label("OM9-SOLID-006"), Geometry::Brep(face), layer);
                output.placement = entity.placement;
                output_ids.push(output.id);
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: output,
                });
            }
            if !copy {
                if indices.len() == brep.display.faces.len() {
                    mutations.push(Mutation::Delete { source });
                } else {
                    let remainder = editing::remove_faces(
                        brep,
                        &indices,
                        brep.display.deflection_mm,
                        self.coordinator.limits(),
                    )
                    .map_err(|error| Error::Io(format!("extract surface remainder: {error}")))?;
                    held.add(
                        remainder.validate(self.coordinator.limits())?,
                        self.coordinator.limits(),
                    )?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    mutations.push(Mutation::Geometry {
                        source,
                        geometry: Geometry::Brep(remainder),
                    });
                }
            }
        }
        self.commit_solid_result(
            "OM9-SOLID-006",
            label("OM9-SOLID-006"),
            mutations,
            Some(output_ids),
        )
    }

    pub(super) fn solid_fillet(&mut self, feature: &'static str, edge: usize, radius: f64) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one B-Rep"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_brep(id)?;
        let filleted = editing::fillet_with_radii(
            brep,
            &[(edge, radius)],
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

    pub(super) fn solid_fillet_edge_endpoints(
        &self,
        reference: &super::SelectedSubobject,
    ) -> Result<Option<(om9_core::geometry::Point3, om9_core::geometry::Point3)>> {
        let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else {
            return Err(Error::Invalid("fillet edge"));
        };
        let brep = self.solid_brep(reference.root)?;
        let index = brep
            .display
            .edges
            .iter()
            .position(|id| *id == key)
            .ok_or(Error::Stale("B-Rep edge"))?;
        editing::oriented_edge_endpoints_or_closed(brep, index, self.coordinator.limits())
            .map_err(|error| Error::Io(format!("fillet edge endpoints: {error}")))
    }

    pub(super) fn solid_fillet_project_pick(
        &self,
        reference: &super::SelectedSubobject,
        placement: om9_core::geometry::Transform,
        world: om9_core::geometry::Point3,
    ) -> Result<(f64, om9_core::geometry::Point3)> {
        let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else {
            return Err(Error::Invalid("fillet edge"));
        };
        let brep = self.solid_brep(reference.root)?;
        let index = brep
            .display
            .edges
            .iter()
            .position(|id| *id == key)
            .ok_or(Error::Stale("B-Rep edge"))?;
        let local = om9_core::top11::inverse_affine(placement)?.point(world)?;
        editing::fillet_project_edge(brep, index, local, self.coordinator.limits())
            .map_err(|error| Error::Io(format!("fillet edge projection: {error}")))
    }

    pub(super) fn solid_fillet_shared_endpoint_masks(
        &self,
        edges: &[FilletEdgePick],
        handle: usize,
    ) -> Result<Vec<u8>> {
        let anchor = edges
            .get(handle / 2)
            .ok_or(Error::Invalid("fillet handle"))?;
        if anchor.endpoints.is_none() {
            let mut masks = Vec::new();
            masks
                .try_reserve_exact(edges.len())
                .map_err(|_| Error::Budget("fillet endpoint masks"))?;
            masks.resize(edges.len(), 0);
            masks[handle / 2] = 3;
            return Ok(masks);
        }
        let brep = self.solid_brep(anchor.reference.root)?;
        let index_of = |edge: &FilletEdgePick| -> Result<usize> {
            if edge.reference.source != anchor.reference.source {
                return Err(Error::Stale("fillet source"));
            }
            let om9_scene::scene::Subelement::BrepEdge(key) = edge.reference.subset else {
                return Err(Error::Invalid("fillet edge"));
            };
            brep.display
                .edges
                .iter()
                .position(|id| *id == key)
                .ok_or(Error::Stale("B-Rep edge"))
        };
        let anchor_index = index_of(anchor)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(edges.len())
            .map_err(|_| Error::Budget("fillet edge indices"))?;
        for edge in edges {
            indices.push(index_of(edge)?);
        }
        editing::fillet_shared_endpoints(
            brep,
            anchor_index,
            handle % 2,
            &indices,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("fillet shared endpoint: {error}")))
    }

    pub(super) fn solid_fillet_build_preview(
        &self,
        edges: &[FilletEdgePick],
        controls: &[FilletControl],
    ) -> Result<FilletPreview> {
        let first = edges.first().ok_or(Error::Missing("fillet edge"))?;
        let id = first.reference.root;
        let brep = self.solid_brep(id)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(edges.len())
            .map_err(|_| Error::Budget("fillet indices"))?;
        for edge in edges {
            if edge.reference.root != id || edge.reference.source != first.reference.source {
                return Err(Error::Stale("fillet source"));
            }
            let om9_scene::scene::Subelement::BrepEdge(key) = edge.reference.subset else {
                return Err(Error::Invalid("fillet edge"));
            };
            indices.push((
                brep.display
                    .edges
                    .iter()
                    .position(|id| *id == key)
                    .ok_or(Error::Stale("B-Rep edge"))?,
                edge.start_radius,
                edge.end_radius,
            ));
        }
        let mut laws = Vec::new();
        laws.try_reserve_exact(controls.len())
            .map_err(|_| Error::Budget("fillet control indices"))?;
        for control in controls {
            let edge_index = indices
                .get(control.edge)
                .ok_or(Error::Invalid("fillet control edge"))?
                .0;
            laws.push((edge_index, control.fraction, control.radius));
        }
        let body = editing::fillet_with_radius_controls(
            brep,
            &indices,
            &laws,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("fillet preview: {error}")))?;
        let usage = body.validate(self.coordinator.limits())?;
        let mut held = self.scene_cache_usage()?;
        held.add(usage, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        Ok(FilletPreview {
            source: first.reference.source,
            placement: self.document().entities[&id].placement,
            body,
            usage,
        })
    }

    pub(super) fn solid_fillet_commit(&mut self, preview: &FilletPreview) -> Result<()> {
        if self.document().stamp(preview.source.object)? != preview.source
            || !self.selection.contains(&preview.source.object)
        {
            return Err(Error::Stale("fillet preview source"));
        }
        self.commit_solid_result_with_extra(
            "OM9-SOLID-007",
            label("OM9-SOLID-007"),
            vec![Mutation::Geometry {
                source: preview.source,
                geometry: Geometry::Brep(preview.body.clone()),
            }],
            Some(vec![preview.source.object]),
            Some(preview.usage),
        )
    }

    /// Internal Slab publication path. The interactive Slab command remains unavailable;
    /// this adapter accepts already-captured parameters in world millimeters for open polylines and full circles.
    #[doc(hidden)]
    pub fn solid_slab_commit_polyline(&mut self, source: om9_core::identity::SourceStamp,
        normal: [f64; 3], offset: f64, height: f64, both_sides: bool) -> Result<()> {
        if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
        let entity = self.document().entities.get(&source.object).ok_or(Error::Stale("Slab source"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Invalid("Slab requires a curve")); };
        let held = self.coordinator.retained_usage_with_held_usage(self.scene_cache_usage()?)?;
        let limits = self.coordinator.limits().remaining_after(held)?;
        if matches!(curve, om9_core::geometry::Curve::Nurbs(n) if n.degree>1) {
            let world = Self::solid_boss_transform_curve(curve, entity.placement)?;
            let world = match world {
                om9_core::geometry::Curve::Nurbs(n) if n.periodic => om9_core::geometry::Curve::Nurbs(
                    om9_core::slab::normalize_closed_nurbs_for_slab(&n,&limits)?),
                other=>other,
            };
            if om9_core::curve_ops::is_closed(&world,1e-7,&limits)? {
                let geometry=om9_core::slab::closed_nurbs_slab(&world,normal,offset,height,both_sides,1e-4,&limits)?;
                let mut ring_usage=geometry.outer.validate(&limits)?;
                ring_usage.add(geometry.inner.validate(&limits)?,&limits)?;
                let ring_limits=limits.remaining_after(ring_usage)?;
                let outside=om9_kernel::extrusion::solid(&geometry.outer,geometry.direction,creation::DEFAULT_DEFLECTION_MM,&ring_limits)
                    .map_err(|e|Error::Io(format!("Slab closed NURBS outer extrusion: {e}")))?;
                let outside_usage=outside.validate(&ring_limits)?;
                let inside_limits=ring_limits.remaining_after(outside_usage)?;
                let inside=om9_kernel::extrusion::solid(&geometry.inner,geometry.direction,creation::DEFAULT_DEFLECTION_MM,&inside_limits)
                    .map_err(|e|Error::Io(format!("Slab closed NURBS inner extrusion: {e}")))?;
                let mut inputs=outside_usage;inputs.add(inside.validate(&inside_limits)?,&ring_limits)?;
                let boolean_limits=ring_limits.remaining_after(inputs)?;
                let body=editing::boolean(&outside,&inside,BooleanOp::Difference,creation::DEFAULT_DEFLECTION_MM,&boolean_limits)
                    .map_err(|e|Error::Io(format!("Slab closed NURBS cut: {e}")))?
                    .body().ok_or(Error::Invalid("Slab closed NURBS cut is empty"))?.clone();
                if self.document().stamp(source.object)? != source {return Err(Error::Stale("Slab source"));}
                let mut result=Entity::new("Slab",Geometry::Brep(body),self.document().current_layer);
                result.placement=om9_core::geometry::Transform::translation(geometry.base_shift);
                let id=result.id;
                return self.commit_solid_result("SolidSlab","Slab",vec![Mutation::Insert {document:self.document,entity:result}],Some(vec![id]));
            }
            let geometry = om9_core::slab::nurbs_slab(&world, normal, offset, height, both_sides, 1e-4, &limits)?;
            let body = om9_kernel::extrusion::solid(&geometry.profile, geometry.direction, creation::DEFAULT_DEFLECTION_MM, &limits)
                .map_err(|e| Error::Io(format!("Slab NURBS extrusion: {e}")))?;
            if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
            let mut result = Entity::new("Slab", Geometry::Brep(body), self.document().current_layer);
            result.placement = om9_core::geometry::Transform::translation(geometry.base_shift);
            let id = result.id;
            return self.commit_solid_result("SolidSlab", "Slab", vec![Mutation::Insert { document: self.document, entity: result }], Some(vec![id]));
        }
        if let om9_core::geometry::Curve::Circle { domain, angle_domain, .. } = curve {
            let angles = angle_domain.unwrap_or(*domain);
            if (angles[1]-angles[0]-std::f64::consts::TAU).abs() > 1e-8 {
                let world = Self::solid_boss_transform_curve(curve, entity.placement)?;
                let geometry = om9_core::slab::arc_slab(&world, normal, offset, height, both_sides, &limits)?;
                let body = om9_kernel::extrusion::solid(&geometry.profile, geometry.direction, creation::DEFAULT_DEFLECTION_MM, &limits)
                    .map_err(|e| Error::Io(format!("Slab arc extrusion: {e}")))?;
                if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
                let mut result = Entity::new("Slab", Geometry::Brep(body), self.document().current_layer);
                result.placement = om9_core::geometry::Transform::translation(geometry.base_shift);
                let id = result.id;
                return self.commit_solid_result("SolidSlab", "Slab", vec![Mutation::Insert { document: self.document, entity: result }], Some(vec![id]));
            }
            let geometry = om9_core::slab::circle_slab(curve, entity.placement, normal, offset, height, both_sides, &limits)?;
            let outside = creation::primitive(Primitive::Cylinder { radius_mm: geometry.outer_radius, height_mm: geometry.height }, creation::DEFAULT_DEFLECTION_MM, &limits)
                .map_err(|e| Error::Io(format!("Slab outer cylinder: {e}")))?;
            let outside_usage = outside.validate(&limits)?;
            let inner_limits = limits.remaining_after(outside_usage)?;
            let inside = creation::primitive(Primitive::Cylinder { radius_mm: geometry.inner_radius, height_mm: geometry.height }, creation::DEFAULT_DEFLECTION_MM, &inner_limits)
                .map_err(|e| Error::Io(format!("Slab inner cylinder: {e}")))?;
            let mut input_usage = outside_usage;
            input_usage.add(inside.validate(&inner_limits)?, &limits)?;
            let boolean_limits = limits.remaining_after(input_usage)?;
            let outcome = editing::boolean(&outside, &inside, BooleanOp::Difference, creation::DEFAULT_DEFLECTION_MM, &boolean_limits)
                .map_err(|e| Error::Io(format!("Slab circular cut: {e}")))?;
            let body = outcome.body().ok_or(Error::Invalid("Slab circular cut is empty"))?.clone();
            if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
            let mut result = Entity::new("Slab", Geometry::Brep(body), self.document().current_layer);
            result.placement = geometry.placement;
            let id = result.id;
            return self.commit_solid_result("SolidSlab", "Slab", vec![Mutation::Insert { document: self.document, entity: result }], Some(vec![id]));
        }
        if let Ok(points) = om9_core::slab::placed_closed_polyline(curve, entity.placement, &limits) {
            let mut point_usage=om9_core::budget::Usage::array::<om9_core::geometry::Point3>(points.capacity())?;
            point_usage.points=points.len() as u64;
            let ring_limits=limits.remaining_after(point_usage)?;
            let (outer,inner,direction)=om9_core::slab::closed_polyline_slab(&points,normal,offset,height,both_sides,&ring_limits)?;
            drop(points);
            let mut profile_usage=om9_core::budget::Usage::array::<om9_core::geometry::Point3>(outer.capacity())?;
            profile_usage.points=outer.len() as u64;
            let mut inner_usage=om9_core::budget::Usage::array::<om9_core::geometry::Point3>(inner.capacity())?;
            inner_usage.points=inner.len() as u64;profile_usage.add(inner_usage,&limits)?;
            let operation_limits=limits.remaining_after(profile_usage)?;
            let outer_profile=creation::Profile::new(outer).map_err(|e|Error::Io(format!("Slab outer profile: {e}")))?;
            let inner_profile=creation::Profile::new(inner).map_err(|e|Error::Io(format!("Slab inner profile: {e}")))?;
            let outside=creation::extrude(&outer_profile,direction,true,creation::DEFAULT_DEFLECTION_MM,&operation_limits)
                .map_err(|e|Error::Io(format!("Slab outer extrusion: {e}")))?;
            let outside_usage=outside.validate(&operation_limits)?;
            let inner_limits=operation_limits.remaining_after(outside_usage)?;
            let inside=creation::extrude(&inner_profile,direction,true,creation::DEFAULT_DEFLECTION_MM,&inner_limits)
                .map_err(|e|Error::Io(format!("Slab inner extrusion: {e}")))?;
            let mut input_usage=outside_usage;input_usage.add(inside.validate(&inner_limits)?,&operation_limits)?;
            let boolean_limits=operation_limits.remaining_after(input_usage)?;
            let body=editing::boolean(&outside,&inside,BooleanOp::Difference,creation::DEFAULT_DEFLECTION_MM,&boolean_limits)
                .map_err(|e|Error::Io(format!("Slab closed cut: {e}")))?
                .body().ok_or(Error::Invalid("Slab closed cut is empty"))?.clone();
            if self.document().stamp(source.object)? != source {return Err(Error::Stale("Slab source"));}
            let result=Entity::new("Slab",Geometry::Brep(body),self.document().current_layer);
            let id=result.id;
            return self.commit_solid_result("SolidSlab","Slab",vec![Mutation::Insert {document:self.document,entity:result}],Some(vec![id]));
        }
        let points = om9_core::slab::placed_open_polyline(curve, entity.placement, &limits)?;
        let mut point_usage = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(points.capacity())?;
        point_usage.points = points.len() as u64;
        let offset_limits = limits.remaining_after(point_usage)?;
        let (boundary, direction) = om9_core::slab::polyline_slab(&points, normal, offset, height, both_sides, &offset_limits)?;
        drop(points);
        let profile = creation::Profile::new(boundary).map_err(|e| Error::Io(format!("Slab profile: {e}")))?;
        let mut profile_usage = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(profile.points().len())?;
        profile_usage.points = profile.points().len() as u64;
        let operation_limits = limits.remaining_after(profile_usage)?;
        let body = creation::extrude(&profile, direction, true, creation::DEFAULT_DEFLECTION_MM, &operation_limits)
            .map_err(|e| Error::Io(format!("Slab extrusion: {e}")))?;
        if self.document().stamp(source.object)? != source { return Err(Error::Stale("Slab source")); }
        let result = Entity::new("Slab", Geometry::Brep(body), self.document().current_layer);
        let id = result.id;
        self.commit_solid_result("SolidSlab", "Slab", vec![Mutation::Insert { document: self.document, entity: result }], Some(vec![id]))
    }

    pub(super) fn solid_extrude(&mut self, feature: &'static str, height: f64, all: bool) -> Result<()> {
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

    pub(super) fn solid_shell(&mut self, feature: &'static str, face: usize, thickness: f64) -> Result<()> {
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
    pub(super) fn solid_shell_picks(
        &mut self,
        faces: &[super::SelectedSubobject],
        thickness: f64,
    ) -> Result<()> {
        let first = faces.first().ok_or(Error::Missing("select a B-Rep face"))?;
        let id = first.root;
        if self.document().stamp(id)? != first.source || !self.selection.contains(&id) {
            return Err(Error::Stale("shell source"));
        }
        let brep = self.solid_brep(id)?;
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<usize>(faces.len())?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(faces.len())
            .map_err(|_| Error::Budget("shell face indices"))?;
        for reference in faces {
            if reference.root != id
                || reference.source != first.source
                || !self.subselection().contains(reference)
            {
                return Err(Error::Stale("shell face selection"));
            }
            let om9_scene::scene::Subelement::BrepFace(key) = reference.subset else {
                return Err(Error::Invalid("shell face"));
            };
            indices.push(
                brep.display
                    .faces
                    .iter()
                    .position(|face| *face == key)
                    .ok_or(Error::Stale("B-Rep face"))?,
            );
        }
        let body = editing::shell_faces(
            brep,
            &indices,
            thickness,
            brep.display.deflection_mm,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("shell: {error}")))?;
        self.commit_solid_result(
            "OM9-SOLID-029",
            label("OM9-SOLID-029"),
            vec![Mutation::Geometry {
                source: first.source,
                geometry: Geometry::Brep(body),
            }],
            Some(vec![id]),
        )
    }
    pub(super) fn solid_text(&mut self, feature: &'static str, text: &str, height: f64) -> Result<()> {
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
    pub(super) fn solid_pipe_round(
        &mut self,
        feature: &'static str,
        start_radius: f64,
        end_radius: f64,
    ) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let body = self.solid_pipe_shape(id, &[start_radius, end_radius], true, true)?;
        self.insert_solid(feature, label(feature), body)
    }

    pub(super) fn solid_pipe(&mut self, feature: &'static str, radius: f64, capped: bool) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_pipe_shape(id, &[radius], capped, false)?;
        self.insert_solid(feature, label(feature), brep)
    }

    pub(super) fn solid_pipe_linear(
        &mut self,
        feature: &'static str,
        start: f64,
        end: f64,
        capped: bool,
    ) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_pipe_shape(id, &[start, end], capped, false)?;
        self.insert_solid(feature, label(feature), brep)
    }

    pub(super) fn solid_pipe_thick(
        &mut self,
        feature: &'static str,
        radii: [f64; 4],
        capped: bool,
        round: bool,
    ) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let brep = self.solid_pipe_shape(id, &radii, capped, round)?;
        self.insert_solid(feature, label(feature), brep)
    }

    pub(super) fn solid_pipe_shape(
        &self,
        id: om9_core::identity::ObjectId,
        radii: &[f64],
        capped: bool,
        round: bool,
    ) -> Result<om9_core::brep::Brep> {
        let entity = &self.document().entities[&id];
        self.solid_pipe_shape_entity(entity, radii, capped, round, self.scene_cache_usage()?)
    }

    pub(super) fn solid_pipe_shape_entity(
        &self,
        entity: &Entity,
        radii: &[f64],
        capped: bool,
        round: bool,
        mut held: om9_core::budget::Usage,
    ) -> Result<om9_core::brep::Brep> {
        let limits = self.coordinator.limits();
        let deflection = creation::DEFAULT_DEFLECTION_MM;
        let circle_radii = match radii {
            [radius] => Some((*radius, None)),
            [a, b] if a == b => Some((*a, None)),
            [a, b, c, d] if a.min(*b) == c.min(*d) && a.max(*b) == c.max(*d) => {
                Some((a.max(*b), Some(a.min(*b))))
            }
            _ => None,
        };
        {
            if let (Some((outer_radius, inner_radius)), Geometry::Curve(curve)) =
                (circle_radii, &entity.geometry)
            {
                if let om9_core::geometry::Curve::Circle {
                    center,
                    x,
                    y,
                    radius: ring,
                    domain,
                    angle_domain,
                } = curve
                {
                    let angles = angle_domain.unwrap_or(*domain);
                    if ((angles[1] - angles[0]) - std::f64::consts::TAU).abs() <= 1.0e-9 {
                        let origin = entity.placement.point(*center)?;
                        let axis = |v: [f64; 3]| -> Result<[f64; 3]> {
                            let p = entity
                                .placement
                                .point(std::array::from_fn(|i| center[i] + v[i]))?;
                            Ok(std::array::from_fn(|i| p[i] - origin[i]))
                        };
                        let mut u = axis(*x)?;
                        let mut v = axis(*y)?;
                        let norm = |p: [f64; 3]| p.iter().map(|n| n * n).sum::<f64>().sqrt();
                        let (su, sv) = (norm(u), norm(v));
                        if su > 0.0 && sv > 0.0 && (su - sv).abs() <= su.max(sv) * 1.0e-9 {
                            u = u.map(|n| n / su);
                            v = v.map(|n| n / sv);
                            let dot = (0..3).map(|i| u[i] * v[i]).sum::<f64>();
                            if dot.abs() <= 1.0e-9 {
                                let w = [
                                    u[1] * v[2] - u[2] * v[1],
                                    u[2] * v[0] - u[0] * v[2],
                                    u[0] * v[1] - u[1] * v[0],
                                ];
                                let frame = om9_core::geometry::Transform([
                                    [u[0], v[0], w[0], origin[0]],
                                    [u[1], v[1], w[1], origin[1]],
                                    [u[2], v[2], w[2], origin[2]],
                                    [0.0, 0.0, 0.0, 1.0],
                                ]);
                                if inner_radius.is_some_and(|inner| inner >= outer_radius) {
                                    return Err(Error::Invalid(
                                        "Thick Pipe radii must differ at each end",
                                    ));
                                }
                                let make_torus = |r| {
                                    creation::primitive(
                                        Primitive::Torus {
                                            ring_radius_mm: ring * su,
                                            tube_radius_mm: r,
                                        },
                                        deflection,
                                        limits,
                                    )
                                    .map_err(|e| Error::Io(format!("Pipe circular rail: {e}")))
                                };
                                let outside = make_torus(outer_radius)?;
                                let torus = if !capped {
                                    let outer_face =
                                        editing::extract_face(&outside, 0, deflection, limits)
                                            .map_err(|e| {
                                                Error::Io(format!("Pipe circular wall: {e}"))
                                            })?;
                                    if let Some(inner) = inner_radius {
                                        if outer_radius - inner <= 1.0e-7 {
                                            return Err(Error::Invalid(
                                                "Thick Pipe walls are too close for open output",
                                            ));
                                        }
                                        let inside = make_torus(inner)?;
                                        let inner_face =
                                            editing::extract_face(&inside, 0, deflection, limits)
                                                .map_err(|e| {
                                                Error::Io(format!("Pipe circular inner wall: {e}"))
                                            })?;
                                        editing::sew_reversing_second(
                                            &outer_face,
                                            &inner_face,
                                            1.0e-7,
                                            deflection,
                                            limits,
                                        )
                                        .map_err(|e| {
                                            Error::Io(format!("Pipe circular walls: {e}"))
                                        })?
                                    } else {
                                        outer_face
                                    }
                                } else if let Some(inner) = inner_radius {
                                    let inside = make_torus(inner)?;
                                    editing::boolean(
                                        &outside,
                                        &inside,
                                        BooleanOp::Difference,
                                        deflection,
                                        limits,
                                    )
                                    .map_err(|e| Error::Io(format!("Pipe circular hollow: {e}")))?
                                    .body()
                                    .ok_or(Error::Invalid("Thick Pipe produced an empty body"))?
                                    .clone()
                                } else {
                                    outside
                                };
                                return om9_kernel::transform_ops::transform_brep(
                                    &torus, &frame, deflection, limits,
                                )
                                .map_err(|e| Error::Io(format!("Pipe circular placement: {e}")));
                            }
                        }
                    }
                }
            }
        }
        if round
            && let Geometry::Curve(curve) = &entity.geometry
            && !matches!(curve, om9_core::geometry::Curve::Line { .. })
        {
            if limits.max_points < 64 {
                return Err(Error::Budget("Pipe samples"));
            }
            held.add(
                om9_core::budget::Usage::array::<om9_core::geometry::Point3>(64)?,
                limits,
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            let mut points = om9_core::surface_kernel::sample_curve(curve, 64, limits)?;
            for point in &mut points {
                *point = entity.placement.point(*point)?;
            }
            let distance = |a: [f64; 3], b: [f64; 3]| {
                (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
            };
            if distance(points[0], points[points.len() - 1]) <= 1e-9 {
                return Err(Error::Unsupported(
                    "Round Pipe on this closed curve is not supported",
                ));
            }
            let (outer, inner) = match radii {
                [r] => ([*r, *r], None),
                [a, b] => ([*a, *b], None),
                [a, b, c, d] => {
                    let outer = [a.max(*b), c.max(*d)];
                    let inner = [a.min(*b), c.min(*d)];
                    if outer[0] <= inner[0] || outer[1] <= inner[1] {
                        return Err(Error::Invalid("Thick Pipe radii must differ at each end"));
                    }
                    (outer, Some(inner))
                }
                _ => return Err(Error::Invalid("Pipe radius count")),
            };
            let outside =
                creation::pipe_linear(&points, outer[0], outer[1], true, deflection, limits)
                    .map_err(|e| Error::Io(format!("Pipe Round: {e}")))?;
            let outside = self.solid_pipe_local_round_caps(outside, &points, outer, held)?;
            if let Some(inner) = inner {
                held.add(outside.validate(limits)?, limits)?;
                self.coordinator.retained_usage_with_held_usage(held)?;
                let inside =
                    creation::pipe_linear(&points, inner[0], inner[1], true, deflection, limits)
                        .map_err(|e| Error::Io(format!("Pipe Round inner: {e}")))?;
                let inside = self.solid_pipe_local_round_caps(inside, &points, inner, held)?;
                held.add(inside.validate(limits)?, limits)?;
                self.coordinator.retained_usage_with_held_usage(held)?;
                return Ok(editing::boolean(
                    &outside,
                    &inside,
                    BooleanOp::Difference,
                    deflection,
                    limits,
                )
                .map_err(|e| Error::Io(format!("Pipe Round hollow: {e}")))?
                .body()
                .ok_or(Error::Invalid("Thick Pipe produced an empty body"))?
                .clone());
            }
            return Ok(outside);
        }
        if let [radius] = radii
            && !round
        {
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("pipe needs a curve"));
            };
            let mut points = om9_core::surface_kernel::sample_curve(curve, 64, limits)?;
            for point in &mut points {
                *point = entity.placement.point(*point)?;
            }
            return creation::pipe(&points, *radius, capped, deflection, limits)
                .map_err(|e| Error::Io(format!("Pipe: {e}")));
        }
        if let ([start, end], Geometry::Curve(curve)) = (radii, &entity.geometry)
            && !round
            && !matches!(curve, om9_core::geometry::Curve::Line { .. })
        {
            let mut points = om9_core::surface_kernel::sample_curve(curve, 64, limits)?;
            for point in &mut points {
                *point = entity.placement.point(*point)?;
            }
            let first = points.first().ok_or(Error::Invalid("Pipe curve samples"))?;
            let last = points.last().ok_or(Error::Invalid("Pipe curve samples"))?;
            if (0..3)
                .map(|i| (first[i] - last[i]).powi(2))
                .sum::<f64>()
                .sqrt()
                <= 1e-9
            {
                return Err(Error::Unsupported(
                    "varying Pipe on a closed curve is not supported",
                ));
            }
            return creation::pipe_linear(&points, *start, *end, capped, deflection, limits)
                .map_err(|e| Error::Io(format!("Pipe: {e}")));
        }
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("pipe needs a curve"));
        };
        let points = if let om9_core::geometry::Curve::Line {
            start: a, end: b, ..
        } = curve
        {
            vec![entity.placement.point(*a)?, entity.placement.point(*b)?]
        } else {
            if round {
                return Err(Error::Unsupported("Round Pipe requires a straight line"));
            }
            let mut points = om9_core::surface_kernel::sample_curve(curve, 64, limits)?;
            for point in &mut points {
                *point = entity.placement.point(*point)?;
            }
            let first = points.first().ok_or(Error::Invalid("Pipe curve samples"))?;
            let last = points.last().ok_or(Error::Invalid("Pipe curve samples"))?;
            if (0..3)
                .map(|i| (first[i] - last[i]).powi(2))
                .sum::<f64>()
                .sqrt()
                <= 1e-9
            {
                return Err(Error::Unsupported(
                    "Thick Pipe on this closed curve is not supported",
                ));
            }
            points
        };
        if let [start, end] = radii {
            return if round {
                creation::pipe_round_straight(
                    [points[0], points[1]],
                    *start,
                    *end,
                    deflection,
                    limits,
                )
            } else {
                creation::pipe_linear(&points, *start, *end, capped, deflection, limits)
            }
            .map_err(|e| Error::Io(format!("Pipe: {e}")));
        }
        let [_, _, _, _] = radii else {
            return Err(Error::Invalid("Pipe radius count"));
        };
        let outer = [radii[0].max(radii[1]), radii[2].max(radii[3])];
        let inner = [radii[0].min(radii[1]), radii[2].min(radii[3])];
        if outer[0] <= inner[0] || outer[1] <= inner[1] {
            return Err(Error::Invalid("Thick Pipe radii must differ at each end"));
        }
        // Disconnected open walls must not be sewn to each other by tolerance.
        let sew_tolerance = 1.0e-7;
        if !capped && (outer[0] - inner[0] <= sew_tolerance || outer[1] - inner[1] <= sew_tolerance)
        {
            return Err(Error::Invalid(
                "Thick Pipe walls are too close for open output",
            ));
        }
        let outside = if round {
            creation::pipe_round_straight(
                [points[0], points[1]],
                outer[0],
                outer[1],
                deflection,
                limits,
            )
        } else {
            creation::pipe_linear(&points, outer[0], outer[1], capped, deflection, limits)
        }
        .map_err(|e| Error::Io(format!("Pipe outer: {e}")))?;
        let inside = if round {
            creation::pipe_round_straight(
                [points[0], points[1]],
                inner[0],
                inner[1],
                deflection,
                limits,
            )
        } else {
            creation::pipe_linear(&points, inner[0], inner[1], capped, deflection, limits)
        }
        .map_err(|e| Error::Io(format!("Pipe inner: {e}")))?;
        if !capped {
            let body =
                editing::sew_reversing_second(&outside, &inside, sew_tolerance, deflection, limits)
                    .map_err(|e| Error::Io(format!("Pipe open walls: {e}")))?;
            return Ok(body);
        }
        let result = editing::boolean(&outside, &inside, BooleanOp::Difference, deflection, limits)
            .map_err(|e| Error::Io(format!("Pipe hollow: {e}")))?;
        let body = result
            .body()
            .ok_or(Error::Invalid("Thick Pipe produced an empty body"))?;
        Ok(body.clone())
    }

    pub(super) fn solid_tube(
        &mut self,
        _feature: &'static str,
        outer: f64,
        inner: f64,
        height: f64,
    ) -> Result<()> {
        self.solid_tube_oriented(
            [0., 0., 0.],
            super::CPlane::world_xy(),
            outer,
            inner,
            height,
        )
    }

    pub(super) fn solid_tube_oriented(
        &mut self,
        center: [f64; 3],
        plane: super::CPlane,
        first: f64,
        second: f64,
        height: f64,
    ) -> Result<()> {
        let feature = "OM9-SOLID-017";
        let (outer, inner) = (first.max(second), first.min(second));
        if inner <= 0.0 || outer <= inner {
            return Err(Error::Invalid("tube needs 0 < inner < outer"));
        }
        let limits = self.coordinator.limits();
        let outside = creation::primitive(
            Primitive::Cylinder {
                radius_mm: outer,
                height_mm: height.abs(),
            },
            creation::DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|error| Error::Io(format!("tube outer: {error}")))?;
        let inside = creation::primitive(
            Primitive::Cylinder {
                radius_mm: inner,
                height_mm: height.abs(),
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
        let mut entity = Entity::new(
            label(feature),
            Geometry::Brep(brep.clone()),
            self.document().current_layer,
        );
        let mut matrix = om9_core::geometry::Transform::identity().0;
        let sign = height.signum();
        let normal = plane.normal();
        for i in 0..3 {
            matrix[i][0] = plane.x[i];
            matrix[i][1] = plane.y[i] * sign;
            matrix[i][2] = normal[i] * sign;
            matrix[i][3] = center[i];
        }
        entity.placement = om9_core::geometry::Transform(matrix);
        self.commit_entity(feature, label(feature), entity)
    }
}
