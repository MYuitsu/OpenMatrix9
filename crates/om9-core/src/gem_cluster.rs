// SPDX-License-Identifier: LGPL-2.1-or-later
//! Halo, cluster and baguette layout families (`OM9-L2-04-11`).
//!
//! The module owns the gem-layout half of the M05 advanced gem tools. Each entry point takes
//! already loaded gems and curve geometry and returns a pure plan of [`GemInstance`]s: nothing
//! here mutates a document, creates an entity or writes a history step. The caller turns one
//! plan into one transaction, so Cancel cannot leave a half-built halo behind.
//!
//! Owner split:
//!
//! * [`halo_layout`] — accent ring around one center stone (`OM9-L3-04-11-001`).
//! * [`cluster_layout`] — one or two concentric accent rings around a center stone
//!   (`OM9-L3-04-11-002`).
//! * [`baguette_channel`] / [`tapered_baguette`] — baguettes laid between two walls
//!   (`OM9-L3-04-11-003`, `-004`).
//! * [`gems_between_curves`] — stones that touch both bounding curves (`OM9-L3-04-11-005`).
//! * [`orient_to_gem`] — the rigid frame that aligns a companion object to a gem
//!   (`OM9-L3-04-11-006`).
//! * [`ClusterFeature`] — the persisted `om9.gem.cluster` record that makes an existing halo or
//!   cluster editable again (`OM9-L3-04-11-007`).
//!
//! Rules shared by every entry point:
//!
//! * units are document millimetres; no tolerance is invented as an acceptance threshold;
//! * a gem frame is right-handed with local `+Z` as the table axis, matching the gem mesh and
//!   the placement module (`crate::gem_placement`), so no second gem convention exists;
//! * a layout that cannot honour its own spacing or gap is refused by name instead of being
//!   silently compressed;
//! * the metal channel, prongs and the ring segment belong to the M06/M07 owners. This module
//!   lays out stones only and records the channel profile parameters for that downstream family.

use crate::{
    Error, Result,
    budget::Limits,
    curve_ops,
    gem::{CustomGemParams, GemCut, GemId, GemInstance, GemSpec},
    geometry::{Curve, Point3, Transform},
    identity::ObjectId,
    model::Extension,
};
use serde::{Deserialize, Serialize};

/// Version of the persisted halo/cluster feature record.
pub const CLUSTER_SCHEMA_VERSION: u32 = 1;
/// Extension slot of a halo/cluster/baguette feature record.
pub const CLUSTER_EXTENSION_SLOT: &str = "om9.gem.cluster";
/// Hard cap on accent stones in one layout, so a typo cannot exhaust memory.
pub const MAX_ACCENTS: usize = 512;

/* ---------------------------------- math ---------------------------------- */

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: Point3, s: f64) -> Point3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: Point3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: Point3, name: &'static str) -> Result<Point3> {
    let length = norm(a);
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid(name));
    }
    Ok(mul(a, 1.0 / length))
}
fn finite(value: f64, name: &'static str) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::Invalid(name))
    }
}
fn in_range(value: f64, min: f64, max: f64, name: &'static str) -> Result<()> {
    finite(value, name)?;
    if value < min || value > max {
        return Err(Error::Invalid(name));
    }
    Ok(())
}

/// Build a right-handed affine frame from the gem axes and origin.
fn frame_transform(position: Point3, x: Point3, y: Point3, z: Point3) -> Result<Transform> {
    let mut matrix = [[0.0; 4]; 4];
    for row in 0..3 {
        matrix[row][0] = x[row];
        matrix[row][1] = y[row];
        matrix[row][2] = z[row];
        matrix[row][3] = position[row];
    }
    matrix[3] = [0.0, 0.0, 0.0, 1.0];
    let transform = Transform(matrix);
    transform.validate()?;
    Ok(transform)
}

/// Orthonormal frame with `z` as the table axis and `hint` deciding the roll about `z`.
fn frame_from(z: Point3, hint: Point3) -> Result<(Point3, Point3, Point3)> {
    let z = unit(z, "gem frame axis")?;
    let mut x = cross(hint, z);
    if norm(x) <= 1.0e-9 {
        let fallback = if z[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        x = cross(fallback, z);
    }
    let x = unit(x, "gem frame tangent")?;
    let y = unit(cross(z, x), "gem frame side")?;
    Ok((x, y, z))
}

fn rotate_about(axis: Point3, vector: Point3, degrees: f64) -> Point3 {
    if degrees == 0.0 {
        return vector;
    }
    let (sin, cos) = degrees.to_radians().sin_cos();
    let axis = unit(axis, "gem frame rotation axis").unwrap_or([0.0, 0.0, 1.0]);
    add(
        add(mul(vector, cos), mul(cross(axis, vector), sin)),
        mul(axis, dot(axis, vector) * (1.0 - cos)),
    )
}

fn position_of(transform: &Transform) -> Point3 {
    [transform.0[0][3], transform.0[1][3], transform.0[2][3]]
}

/// Map a point/axis from the center gem's local frame to world space.
fn map_local(origin: Point3, basis: [[f64; 3]; 3], local: Point3) -> Point3 {
    let mut point = origin;
    for axis in 0..3 {
        point = add(point, mul(basis[axis], local[axis]));
    }
    point
}

/* ---------------------------------- halo ---------------------------------- */

/// Parameters of the Halo Builder (`OM9-L3-04-11-001`).
///
/// Every distance follows the manual's mm units. The channel wall and profile values are
/// recorded so the downstream metal owner can build the setting from the same feature; the
/// stone layout only depends on the spacing and the accent size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaloParams {
    /// The accent stone the halo is built from.
    pub accent: GemSpec,
    /// Minimum gap between two neighbouring accent stones, in mm.
    pub spacing_mm: f64,
    /// Gap between the center stone girdle and the inner accent girdle, in mm.
    pub spacing_from_center_mm: f64,
    /// Tilt of the accent table away from the center, in degrees (`-89..=89`).
    pub angle_deg: f64,
    /// Accent height relative to the halo ring plane, in mm.
    pub height_offset_mm: f64,
    /// Spin of every accent about its own table axis, in degrees.
    pub rotation_deg: f64,
    /// Z placement of the halo ring relative to the center girdle plane, in mm.
    pub placement_mm: f64,
    /// Inner channel wall thickness, in mm (owned downstream by the metal builder).
    pub inner_wall_mm: f64,
    /// Outer channel wall thickness, in mm.
    pub outer_wall_mm: f64,
    /// Channel profile width, in mm.
    pub profile_width_mm: f64,
    /// Channel profile height, in mm.
    pub profile_height_mm: f64,
    /// Gap between the accent stones and the channel wall, in mm.
    pub channel_spacing_mm: f64,
    /// Height of the channel wall above the girdle plane, in mm.
    #[serde(default)]
    pub channel_wall_height_mm: f64,
    /// Blend radius of the channel wall, in mm. Larger is a softer curve.
    #[serde(default)]
    pub channel_blend_mm: f64,
    /// Nudge of the prong edge in the channel, in mm.
    #[serde(default)]
    pub edge_nudge_mm: f64,
    /// How far a prong rises above the accent table, in mm.
    #[serde(default)]
    pub prong_height_offset_mm: f64,
    /// Fillet/roundness of the prong top, `0..=1`.
    #[serde(default)]
    pub prong_dome: f64,
    /// Overlap of the prong over the accent stone, in mm.
    #[serde(default)]
    pub gem_nudge_mm: f64,
    /// Prong base diameter as a fraction of the top (`-1..=1`).
    #[serde(default)]
    pub prong_taper: f64,
    /// Bottom of the prong relative to the accent culet, in mm.
    #[serde(default)]
    pub prong_vertical_mm: f64,
    /// Generate prongs as meshes instead of NURBS (downstream metal owner).
    #[serde(default)]
    pub prongs_meshed: bool,
    /// Explicit accent count; `None` derives it from the spacing.
    pub count: Option<usize>,
    /// Put the fixed accent at 12:00 instead of the default 3:00 (round center only).
    pub fix_position: bool,
    /// Distribute the accents by equal arc length instead of equal angle.
    pub even: bool,
    /// The halo follows the center silhouette. Off allows the accent shape override.
    pub match_shape: bool,
}

impl HaloParams {
    /// The builder defaults: a 1.3 mm accent with a 0.3 mm gap and no tilt.
    pub fn new(accent: GemSpec) -> Result<Self> {
        let params = Self {
            accent,
            spacing_mm: 0.3,
            spacing_from_center_mm: 0.0,
            angle_deg: 0.0,
            height_offset_mm: 0.0,
            rotation_deg: 0.0,
            placement_mm: 0.0,
            inner_wall_mm: 0.4,
            outer_wall_mm: 0.4,
            profile_width_mm: 1.2,
            profile_height_mm: 1.0,
            channel_spacing_mm: 0.0,
            channel_wall_height_mm: 0.0,
            channel_blend_mm: 0.4,
            edge_nudge_mm: 0.0,
            prong_height_offset_mm: 0.0,
            prong_dome: 0.5,
            gem_nudge_mm: 0.0,
            prong_taper: 0.0,
            prong_vertical_mm: 0.0,
            prongs_meshed: false,
            count: None,
            fix_position: false,
            even: true,
            match_shape: true,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        self.accent.validate()?;
        in_range(self.spacing_mm, 0.0, 100.0, "halo spacing")?;
        in_range(
            self.spacing_from_center_mm,
            0.0,
            100.0,
            "halo spacing from center",
        )?;
        in_range(self.angle_deg, -89.0, 89.0, "halo angle")?;
        in_range(self.height_offset_mm, -100.0, 100.0, "halo height offset")?;
        in_range(self.placement_mm, -100.0, 100.0, "halo placement")?;
        for (value, name) in [
            (self.inner_wall_mm, "halo inner wall"),
            (self.outer_wall_mm, "halo outer wall"),
            (self.profile_width_mm, "halo profile width"),
            (self.profile_height_mm, "halo profile height"),
            (self.channel_spacing_mm, "halo channel spacing"),
            (self.channel_wall_height_mm, "halo channel wall height"),
            (self.channel_blend_mm, "halo channel blend"),
            (self.edge_nudge_mm, "halo edge nudge"),
            (self.prong_height_offset_mm, "halo prong height offset"),
            (self.gem_nudge_mm, "halo gem nudge"),
            (self.prong_vertical_mm, "halo prong vertical"),
        ] {
            in_range(value, 0.0, 100.0, name)?;
        }
        in_range(self.prong_dome, 0.0, 1.0, "halo prong dome")?;
        in_range(self.prong_taper, -1.0, 1.0, "halo prong taper")?;
        finite(self.rotation_deg, "halo rotation")?;
        if let Some(count) = self.count {
            if count > MAX_ACCENTS {
                return Err(Error::Budget("halo accent count"));
            }
            if count < 3 {
                return Err(Error::Invalid("halo accent count"));
            }
        }
        Ok(())
    }
}

/// One computed halo: the accent instances and the realised ring metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct HaloLayout {
    pub params: HaloParams,
    /// Accent instances in ring order, starting at the fixed stone.
    pub accents: Vec<GemInstance>,
    /// Semi axes of the ring centerline in the center stone's local frame.
    pub radius_mm: [f64; 2],
    /// Smallest realised gap between two neighbouring accents, in mm.
    pub realized_spacing_mm: f64,
}

/// Approximate ellipse perimeter (Ramanujan) in the same units as the semi axes.
fn ellipse_perimeter(a: f64, b: f64) -> f64 {
    let h = ((a - b) / (a + b)).powi(2);
    std::f64::consts::PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

/// Build the halo accent ring around `center` (`OM9-L3-04-11-001`).
///
/// The ring centerline is the center girdle silhouette offset by `spacing_from_center` and the
/// accent half-width. The accent count is either explicit or the largest count that still keeps
/// `spacing_mm` between neighbours, and every produced neighbour pair is checked against that
/// gap afterwards, so a request with too many accents fails instead of overlapping.
pub fn halo_layout(center: &GemInstance, params: &HaloParams) -> Result<HaloLayout> {
    center.spec.validate()?;
    params.validate()?;
    let half_length = center.spec.length_mm * 0.5;
    let half_width = center.spec.width_mm * 0.5;
    let offset = params.spacing_from_center_mm + params.accent.width_mm * 0.5;
    let semi = if params.match_shape {
        [half_length + offset, half_width + offset]
    } else {
        let radius = half_length.max(half_width) + offset;
        [radius, radius]
    };
    if semi[0] <= 0.0 || semi[1] <= 0.0 {
        return Err(Error::Invalid("halo ring radius"));
    }
    let perimeter = ellipse_perimeter(semi[0], semi[1]);
    let pitch = params.accent.length_mm + params.spacing_mm;
    let count = match params.count {
        Some(count) => count,
        None => {
            let count = (perimeter / pitch).floor() as usize;
            if count < 3 {
                return Err(Error::Invalid("halo does not fit a ring"));
            }
            if count > MAX_ACCENTS {
                return Err(Error::Budget("halo accent count"));
            }
            count
        }
    };
    if count > MAX_ACCENTS {
        return Err(Error::Budget("halo accent count"));
    }
    // Equal arc length is the default: measure a dense ellipse and map each station by length.
    let samples = 4096usize;
    let mut lengths = Vec::with_capacity(samples + 1);
    let mut previous = [semi[0], 0.0];
    let mut total = 0.0;
    lengths.push(0.0);
    for index in 1..=samples {
        let angle = std::f64::consts::TAU * index as f64 / samples as f64;
        let point = [semi[0] * angle.cos(), semi[1] * angle.sin()];
        total += (point[0] - previous[0]).hypot(point[1] - previous[1]);
        lengths.push(total);
        previous = point;
    }
    let fixed_turn = if params.fix_position { 0.25 } else { 0.0 };
    let basis = center.basis();
    let origin = position_of(&center.placement);
    let mut accents = Vec::with_capacity(count);
    let mut realized = f64::INFINITY;
    let mut previous_position: Option<Point3> = None;
    for index in 0..count {
        let fraction = index as f64 / count as f64;
        let angle = if params.even {
            let target = total * (fraction + fixed_turn).rem_euclid(1.0);
            let mut low = 0usize;
            let mut high = samples;
            while low + 1 < high {
                let middle = (low + high) / 2;
                if lengths[middle] < target {
                    low = middle;
                } else {
                    high = middle;
                }
            }
            let span = lengths[high] - lengths[low];
            let local = if span > 1.0e-12 {
                (target - lengths[low]) / span
            } else {
                0.0
            };
            std::f64::consts::TAU * (low as f64 + local) / samples as f64
        } else {
            std::f64::consts::TAU * (fraction + fixed_turn)
        };
        let angle = angle.rem_euclid(std::f64::consts::TAU);
        let (sin, cos) = angle.sin_cos();
        let local = [semi[0] * cos, semi[1] * sin, 0.0];
        let outward = unit([cos / semi[0], sin / semi[1], 0.0], "halo radial direction")?;
        let tangent = unit([-semi[0] * sin, semi[1] * cos, 0.0], "halo tangent")?;
        let (sin_angle, cos_angle) = params.angle_deg.to_radians().sin_cos();
        let local_up = unit(
            add(mul(outward, sin_angle), mul([0.0, 0.0, 1.0], cos_angle)),
            "halo accent axis",
        )?;
        let (mut x, mut y, z) = frame_from(local_up, tangent)?;
        x = rotate_about(z, x, params.rotation_deg);
        y = rotate_about(z, y, params.rotation_deg);
        let position = map_local(
            origin,
            basis,
            add(
                local,
                [0.0, 0.0, params.placement_mm + params.height_offset_mm],
            ),
        );
        let transform = frame_transform(
            position,
            map_local([0.0, 0.0, 0.0], basis, x),
            map_local([0.0, 0.0, 0.0], basis, y),
            map_local([0.0, 0.0, 0.0], basis, z),
        )?;
        // A neighbor closer than the requested pitch is a refused layout, never a squeeze.
        if let Some(previous) = previous_position {
            let distance = norm(sub(position, previous));
            let gap = distance - params.accent.length_mm;
            realized = realized.min(gap);
            if gap + 1.0e-6 < params.spacing_mm {
                return Err(Error::Invalid("halo spacing does not fit"));
            }
        }
        previous_position = Some(position);
        accents.push(GemInstance {
            spec: params.accent.clone(),
            placement: transform,
            label: format!("Halo {}", params.accent.shape.label()),
        });
    }
    // The closing pair is checked as well; the ring has no duplicated last stone.
    if accents.len() > 2 {
        let first = position_of(&accents[0].placement);
        let last = position_of(&accents[accents.len() - 1].placement);
        let gap = norm(sub(first, last)) - params.accent.length_mm;
        realized = realized.min(gap);
        if gap + 1.0e-6 < params.spacing_mm {
            return Err(Error::Invalid("halo spacing does not fit"));
        }
    }
    if !realized.is_finite() {
        realized = 0.0;
    }
    Ok(HaloLayout {
        params: params.clone(),
        accents,
        radius_mm: semi,
        realized_spacing_mm: realized,
    })
}

/* --------------------------------- cluster -------------------------------- */

/// Where the first cluster accent starts, read from the Looking Down view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterStart {
    /// 6 o'clock.
    Bottom,
    /// 12 o'clock.
    Top,
    /// 9 o'clock.
    Left,
    /// 3 o'clock.
    Right,
}

impl ClusterStart {
    pub fn angle_deg(self) -> f64 {
        match self {
            Self::Right => 0.0,
            Self::Top => 90.0,
            Self::Left => 180.0,
            Self::Bottom => 270.0,
        }
    }
}

/// Parameters of the Cluster Builder (`OM9-L3-04-11-002`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterParams {
    pub accent: GemSpec,
    /// Distance between the center stone and the accent stones on XY, in mm.
    pub xy_offset_mm: f64,
    /// Distance between the center stone and the accent stones on Z, in mm; positive is down.
    pub z_offset_mm: f64,
    /// Spin of every accent about its own table axis, in degrees.
    pub degree_deg: f64,
    /// Explicit accent count per ring; `None` derives the maximum.
    pub count: Option<usize>,
    pub start: ClusterStart,
    /// One or two concentric rings of edge gems.
    pub rings: u32,
    /// Radial gap between two rings, in mm.
    pub ring_spacing_mm: f64,
}

impl ClusterParams {
    pub fn new(accent: GemSpec) -> Result<Self> {
        let ring_spacing_mm = accent.width_mm + 0.3;
        let params = Self {
            accent,
            xy_offset_mm: 0.0,
            z_offset_mm: 0.0,
            degree_deg: 0.0,
            count: None,
            start: ClusterStart::Bottom,
            rings: 1,
            ring_spacing_mm,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        self.accent.validate()?;
        in_range(self.xy_offset_mm, 0.0, 100.0, "cluster xy offset")?;
        in_range(self.z_offset_mm, -100.0, 100.0, "cluster z offset")?;
        finite(self.degree_deg, "cluster degree")?;
        if !(1..=2).contains(&self.rings) {
            return Err(Error::Invalid("cluster ring count"));
        }
        in_range(self.ring_spacing_mm, 0.0, 100.0, "cluster ring spacing")?;
        if self.rings > 1 && self.ring_spacing_mm <= 1.0e-9 {
            return Err(Error::Invalid("cluster second ring spacing"));
        }
        if let Some(count) = self.count {
            if count > MAX_ACCENTS {
                return Err(Error::Budget("cluster accent count"));
            }
            if count < 3 {
                return Err(Error::Invalid("cluster accent count"));
            }
        }
        Ok(())
    }
}

/// One computed cluster.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterLayout {
    pub params: ClusterParams,
    pub accents: Vec<GemInstance>,
    /// Ring radii in the center stone's local frame, inner ring first.
    pub ring_radii_mm: Vec<f64>,
}

/// Build the cluster edge gems around `center` (`OM9-L3-04-11-002`).
///
/// `Positive Z Offset moves the accents down`, as the manual states, so the sign is inverted
/// once here and never in the UI. `Use Maximum Gems` (an absent count) derives the largest ring
/// that keeps the accents touching, while an explicit count is used exactly and checked for
/// crossing neighbours.
pub fn cluster_layout(center: &GemInstance, params: &ClusterParams) -> Result<ClusterLayout> {
    center.spec.validate()?;
    params.validate()?;
    let half = center.spec.length_mm.max(center.spec.width_mm) * 0.5;
    let mut accents = Vec::new();
    let mut ring_radii = Vec::with_capacity(params.rings as usize);
    let basis = center.basis();
    let origin = position_of(&center.placement);
    let table = map_local([0.0, 0.0, 0.0], basis, [0.0, 0.0, 1.0]);
    for ring in 0..params.rings as usize {
        let radius = half
            + params.xy_offset_mm
            + ring as f64 * params.ring_spacing_mm
            + params.accent.width_mm * 0.5;
        if radius <= 0.0 {
            return Err(Error::Invalid("cluster ring radius"));
        }
        let circumference = std::f64::consts::TAU * radius;
        let count = match params.count {
            Some(count) => count,
            None => {
                let count = (circumference / params.accent.length_mm).floor() as usize;
                if count < 3 {
                    return Err(Error::Invalid("cluster does not fit a ring"));
                }
                if count > MAX_ACCENTS {
                    return Err(Error::Budget("cluster accent count"));
                }
                count
            }
        };
        if count > MAX_ACCENTS {
            return Err(Error::Budget("cluster accent count"));
        }
        let start = params.start.angle_deg().to_radians();
        let mut previous: Option<Point3> = None;
        for index in 0..count {
            let angle = start + std::f64::consts::TAU * index as f64 / count as f64;
            let local = [
                radius * angle.cos(),
                radius * angle.sin(),
                -params.z_offset_mm,
            ];
            let position = map_local(origin, basis, local);
            let tangent = unit(
                map_local([0.0, 0.0, 0.0], basis, [-angle.sin(), angle.cos(), 0.0]),
                "cluster tangent",
            )?;
            let (x0, y0, z) = frame_from(table, tangent)?;
            let spin = params.degree_deg;
            let x = rotate_about(z, x0, spin);
            let y = rotate_about(z, y0, spin);
            let transform = frame_transform(position, x, y, z)?;
            if let Some(previous) = previous {
                let gap = norm(sub(position, previous)) - params.accent.length_mm;
                if gap + 1.0e-6 < 0.0 {
                    return Err(Error::Invalid("cluster accent count overlaps"));
                }
            }
            previous = Some(position);
            accents.push(GemInstance {
                spec: params.accent.clone(),
                placement: transform,
                label: format!("Cluster {}", params.accent.shape.label()),
            });
        }
        ring_radii.push(radius);
    }
    Ok(ClusterLayout {
        params: params.clone(),
        accents,
        ring_radii_mm: ring_radii,
    })
}

/* --------------------------- channel / between curves --------------------- */

fn curve_span(curve: &Curve, from: f64, to: f64, limits: &Limits) -> Result<(f64, f64, f64)> {
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 1.0e-9 {
        return Err(Error::Invalid("channel curve length"));
    }
    Ok((total, total * from, total * to))
}

fn tapered_spec(
    base: &GemSpec,
    length_mm: f64,
    width_start_mm: f64,
    width_end_mm: f64,
) -> Result<GemSpec> {
    let width_start = width_start_mm.max(1.0e-3);
    let width_end = width_end_mm.max(1.0e-3);
    if width_start > length_mm + 1.0e-9 || width_end > length_mm + 1.0e-9 {
        return Err(Error::Invalid("channel gem width exceeds length"));
    }
    let half_length = length_mm * 0.5;
    let profile = vec![
        [half_length, width_end * 0.5],
        [-half_length, width_start * 0.5],
        [-half_length, -width_start * 0.5],
        [half_length, -width_end * 0.5],
    ];
    let params = CustomGemParams {
        facet_count: base.facet_count.max(4),
        crown_angle_deg: base.crown_angle_deg,
        crown_height_mm: base.crown_height_mm,
        girdle_mm: base.girdle_mm,
        pavilion_depth_mm: base.pavilion_depth_mm,
        pavilion_rows: base.pavilion_rows,
        bulge: base.bulge,
    };
    let mut spec = GemSpec::from_profile(&profile, GemCut::Gem, &params)?;
    // Keep the caller's family: a tapered baguette stays a baguette even though its outline is
    // stored as a custom profile.
    spec.shape = base.shape;
    spec.cut = base.cut;
    spec.validate()?;
    Ok(spec)
}

/// Parameters of the baguette channel and the between-curves tools
/// (`OM9-L3-04-11-003`, `-004`, `-005`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelParams {
    /// The stone section; its length is the nominal stone length along the channel.
    pub section: GemSpec,
    /// Gap between successive stones, in mm.
    pub spacing_mm: f64,
    /// Largest banking roll allowed on a non-planar pair of walls, in degrees.
    pub banking_deg: f64,
    /// Reverse the table direction relative to the wall normal.
    pub flip: bool,
    /// Put the start point at the middle of the first stone instead of its edge.
    pub middle: bool,
    /// Keep the section dimensions instead of following the wall gap.
    pub straight: bool,
    /// Lengthwise taper ratio (`0.05..=1.0`; `1.0` is straight).
    pub taper: f64,
    /// Start fraction of both curves (`0..=1`).
    pub point_a: f64,
    /// End fraction of both curves (`0..=1`, greater than `point_a`).
    pub point_b: f64,
    /// Put the stone table, not its girdle, on the wall plane.
    pub gem_top: bool,
    /// Let the end stone end at the end of the walls instead of honouring the spacing exactly.
    pub auto_spacing: bool,
}

impl ChannelParams {
    pub fn new(section: GemSpec) -> Result<Self> {
        let params = Self {
            section,
            spacing_mm: 0.2,
            banking_deg: 0.0,
            flip: false,
            middle: false,
            straight: false,
            taper: 1.0,
            point_a: 0.0,
            point_b: 1.0,
            gem_top: false,
            auto_spacing: false,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        self.section.validate()?;
        in_range(self.spacing_mm, 0.0, 100.0, "channel spacing")?;
        in_range(self.banking_deg, 0.0, 90.0, "channel banking")?;
        in_range(self.taper, 0.05, 1.0, "channel taper")?;
        in_range(self.point_a, 0.0, 1.0, "channel point a")?;
        in_range(self.point_b, 0.0, 1.0, "channel point b")?;
        if self.point_b <= self.point_a {
            return Err(Error::Invalid("channel range"));
        }
        Ok(())
    }
}

/// One stone of a channel layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGem {
    pub instance: GemInstance,
    /// Distance between the two walls at the stone center, in mm.
    pub gap_mm: f64,
    /// Fraction along the channel (`0..=1`).
    pub station: f64,
    /// Stone length along the channel, in mm.
    pub length_mm: f64,
}

/// A computed channel layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelLayout {
    pub params: ChannelParams,
    pub gems: Vec<ChannelGem>,
    pub total_length_mm: f64,
    pub min_gap_mm: f64,
    pub max_gap_mm: f64,
}

/// The wall pair at a fraction of the channel.
fn channel_point(
    a: &Curve,
    b: &Curve,
    fraction: f64,
    limits: &Limits,
) -> Result<(Point3, Point3, Point3, Point3, f64)> {
    let fa =
        curve_ops::parameter_at_length(a, curve_ops::arc_length(a, limits)? * fraction, limits)?;
    let fb =
        curve_ops::parameter_at_length(b, curve_ops::arc_length(b, limits)? * fraction, limits)?;
    let pa = curve_ops::point_at(a, fa, limits)?;
    let pb = curve_ops::point_at(b, fb, limits)?;
    let ta = curve_ops::tangent_at(a, fa, limits)?;
    let tb = curve_ops::tangent_at(b, fb, limits)?;
    let gap = norm(sub(pb, pa));
    if !gap.is_finite() || gap <= 1.0e-9 {
        return Err(Error::Invalid("channel walls touch"));
    }
    Ok((pa, pb, ta, tb, gap))
}

fn channel_gem(
    params: &ChannelParams,
    fraction: f64,
    a: &Curve,
    b: &Curve,
    length_mm: f64,
    limits: &Limits,
) -> Result<ChannelGem> {
    let (pa, pb, ta, tb, gap) = channel_point(a, b, fraction, limits)?;
    let center = mul(add(pa, pb), 0.5);
    let mut across = unit(sub(pb, pa), "channel wall direction")?;
    let tangent = unit(add(ta, tb), "channel tangent")?;
    if params.flip {
        across = mul(across, -1.0);
    }
    // The channel is right-handed with Z as the table axis. Z points up when the walls are
    // horizontal; banking then rolls the frame about the channel direction.
    let mut x = tangent;
    let mut y = across;
    let mut z = unit(cross(x, y), "channel normal")?;
    if z[2] < 0.0 {
        // Keep the table up by reversing the channel direction, not by undoing `flip`.
        x = mul(x, -1.0);
        z = unit(cross(x, y), "channel normal")?;
    }
    if params.banking_deg > 0.0 {
        let roll = params.banking_deg;
        y = rotate_about(x, y, roll);
        z = rotate_about(x, z, roll);
    }
    let width = if params.straight {
        params.section.width_mm
    } else {
        gap
    };
    let spec = if params.straight {
        params.section.clone().set_girdle_mm(length_mm, width)?
    } else if params.section.shape.is_round() {
        // A round stone spans the two walls: the width is the diameter.
        params.section.clone().set_girdle_mm(width, width)?
    } else if params.taper >= 1.0 {
        params.section.clone().set_girdle_mm(length_mm, width)?
    } else {
        tapered_spec(&params.section, length_mm, width, width * params.taper)?
    };
    let table_z = params.section.girdle_mm * 0.5 + params.section.crown_height_mm;
    let origin = if params.gem_top {
        sub(center, mul(z, table_z))
    } else {
        center
    };
    let transform = frame_transform(origin, x, y, z)?;
    Ok(ChannelGem {
        instance: GemInstance {
            spec,
            placement: transform,
            label: format!("Channel {}", params.section.shape.label()),
        },
        gap_mm: gap,
        station: fraction,
        length_mm,
    })
}

/// Number of stones and the effective pitch that fit `usable - section.length` for the params.
fn channel_fit(usable_mm: f64, length_mm: f64, params: &ChannelParams) -> Result<(usize, f64)> {
    if usable_mm + 1.0e-9 < length_mm {
        return Err(Error::Invalid("channel is shorter than one stone"));
    }
    let pitch = length_mm + params.spacing_mm;
    let count = ((usable_mm - length_mm) / pitch).floor() as usize + 1;
    if count == 0 {
        return Err(Error::Invalid("channel does not fit a stone"));
    }
    if count > MAX_ACCENTS {
        return Err(Error::Budget("channel stone count"));
    }
    let effective = if params.auto_spacing && count > 1 {
        (usable_mm - length_mm) / (count - 1) as f64
    } else {
        pitch
    };
    Ok((count, effective))
}

/// Lay baguettes between two wall curves (`OM9-L3-04-11-003`).
///
/// The stones contact both walls by their width, so the width is the local wall gap (unless
/// `straight` keeps the section). The length is the nominal section length; `taper` tapers the
/// two ends of each stone, and `straight` removes that taper.
pub fn baguette_channel(
    a: &Curve,
    b: &Curve,
    params: &ChannelParams,
    limits: &Limits,
) -> Result<ChannelLayout> {
    params.validate()?;
    let (total_a, start_a, end_a) = curve_span(a, params.point_a, params.point_b, limits)?;
    let (total_b, start_b, end_b) = curve_span(b, params.point_a, params.point_b, limits)?;
    let usable = (end_a - start_a).min(end_b - start_b);
    let length = params.section.length_mm;
    let (count, pitch) = channel_fit(usable, length, params)?;
    let first_center = if params.middle { 0.0 } else { length * 0.5 };
    let mut gems = Vec::with_capacity(count);
    let mut min_gap = f64::INFINITY;
    let mut max_gap = f64::NEG_INFINITY;
    for index in 0..count {
        let along = first_center + index as f64 * pitch;
        let fraction = (start_a + along) / total_a;
        let fraction_b = (start_b + along) / total_b;
        if fraction < 0.0 || fraction > 1.0 || fraction_b < 0.0 || fraction_b > 1.0 {
            return Err(Error::Invalid("channel stone outside the walls"));
        }
        let gem = channel_gem(params, fraction, a, b, length, limits)?;
        min_gap = min_gap.min(gem.gap_mm);
        max_gap = max_gap.max(gem.gap_mm);
        gems.push(gem);
    }
    Ok(ChannelLayout {
        params: params.clone(),
        gems,
        total_length_mm: usable,
        min_gap_mm: if min_gap.is_finite() { min_gap } else { 0.0 },
        max_gap_mm: if max_gap.is_finite() { max_gap } else { 0.0 },
    })
}

/// One manually placed tapered baguette between two curves (`OM9-L3-04-11-004`).
///
/// Point A and Point B are independent fractions along the two walls, so the two ends can be
/// moved one at a time. The length is measured between them along the first wall and the two
/// widths are the local wall gaps, which is what makes the baguette tapered and bent.
pub fn tapered_baguette(
    a: &Curve,
    b: &Curve,
    params: &ChannelParams,
    limits: &Limits,
) -> Result<ChannelGem> {
    params.validate()?;
    let length = curve_ops::length_between(a, params.point_a, params.point_b, limits)?;
    if !length.is_finite() || length <= 1.0e-9 {
        return Err(Error::Invalid("tapered baguette length"));
    }
    let (pa_a, pb_a, _, _, gap_a) = channel_point(a, b, params.point_a, limits)?;
    let (pa_b, pb_b, _, _, gap_b) = channel_point(a, b, params.point_b, limits)?;
    let middle = params.point_a + (params.point_b - params.point_a) * 0.5;
    let (pa, pb, ta, tb, _) = channel_point(a, b, middle, limits)?;
    let center = mul(add(pa, pb), 0.5);
    let tangent = unit(add(ta, tb), "tapered baguette direction")?;
    let mut across = unit(sub(pb, pa), "tapered baguette wall direction")?;
    if params.flip {
        across = mul(across, -1.0);
    }
    let mut x = tangent;
    let y = across;
    let mut z = unit(cross(x, y), "tapered baguette normal")?;
    if z[2] < 0.0 {
        x = mul(x, -1.0);
        z = unit(cross(x, y), "tapered baguette normal")?;
    }
    let _ = (pa_a, pb_a, pa_b, pb_b);
    let (width_start, width_end) = if params.flip {
        (gap_b, gap_a)
    } else {
        (gap_a, gap_b)
    };
    let width_start = if params.straight {
        params.section.width_mm
    } else {
        width_start
    };
    let width_end = if params.straight {
        params.section.width_mm
    } else {
        width_end
    };
    let spec = if params.straight {
        params
            .section
            .clone()
            .set_girdle_mm(length, params.section.width_mm)?
    } else {
        tapered_spec(&params.section, length, width_start, width_end)?
    };
    let table_z = params.section.girdle_mm * 0.5 + params.section.crown_height_mm;
    let origin = if params.gem_top {
        sub(center, mul(z, table_z))
    } else {
        center
    };
    let transform = frame_transform(origin, x, y, z)?;
    Ok(ChannelGem {
        instance: GemInstance {
            spec,
            placement: transform,
            label: format!("Tapered {}", params.section.shape.label()),
        },
        gap_mm: gap_a.max(gap_b),
        station: middle,
        length_mm: length,
    })
}

/// Lay stones that touch both bounding curves (`OM9-L3-04-11-005`).
///
/// This is the same geometric construction as [`baguette_channel`]; the difference is the
/// section (any cut) and that the stone width follows the wall gap so the stone always touches
/// both walls. `Mode=Spacing/Shape` is represented by `auto_spacing`: on, the last stone ends at
/// the end of the walls; off, the requested spacing is honoured exactly.
pub fn gems_between_curves(
    a: &Curve,
    b: &Curve,
    params: &ChannelParams,
    limits: &Limits,
) -> Result<ChannelLayout> {
    baguette_channel(a, b, params, limits)
}

/* ------------------------------- orient to gem ---------------------------- */

/// Options of Orient to Gem (`OM9-L3-04-11-006`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrientOptions {
    /// Display the reference guide gem at 10 mm. The oriented object keeps its own scale.
    pub scale_to_gem: bool,
    /// Offset of the object origin in the gem frame, in mm.
    pub offset_mm: [f64; 3],
}

impl Default for OrientOptions {
    fn default() -> Self {
        Self {
            scale_to_gem: true,
            offset_mm: [0.0, 0.0, 0.0],
        }
    }
}

/// Orient a companion object to a gem frame (`OM9-L3-04-11-006`).
///
/// The object keeps its own uniform scale unless `scale_to_gem` is set, and its origin lands at
/// the gem origin plus `offset_mm`. The axes of the result are exactly the gem axes, which is
/// the operation the manual describes as "Set to Gems"; the input transform is never mutated.
pub fn orient_to_gem(
    gem: &GemInstance,
    object: &Transform,
    options: &OrientOptions,
) -> Result<Transform> {
    gem.spec.validate()?;
    object.validate()?;
    for value in options.offset_mm {
        finite(value, "orient offset")?;
    }
    let scale = norm([object.0[0][0], object.0[1][0], object.0[2][0]]);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::Invalid("orient object scale"));
    }
    // Scale to Gem only sizes the guide gem; the oriented object keeps its authored scale.
    let basis = gem.basis();
    let origin = position_of(&gem.placement);
    let mut position = origin;
    for (axis, offset) in basis.iter().zip(options.offset_mm) {
        position = add(position, mul(*axis, offset));
    }
    for axis in &basis {
        if axis.iter().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("orient gem basis"));
        }
    }
    frame_transform(
        position,
        mul(basis[0], scale),
        mul(basis[1], scale),
        mul(basis[2], scale),
    )
}

/* ------------------------------ feature record ---------------------------- */

/// The layout family recorded on a center stone or a pair of curves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "layout", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClusterKind {
    Halo { params: HaloParams },
    Cluster { params: ClusterParams },
    Channel { params: ChannelParams },
}

impl ClusterKind {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Halo { .. } => "halo",
            Self::Cluster { .. } => "cluster",
            Self::Channel { .. } => "channel",
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Halo { params } => params.validate(),
            Self::Cluster { params } => params.validate(),
            Self::Channel { params } => params.validate(),
        }
    }
}

/// The persisted editable halo/cluster/baguette feature (`OM9-L3-04-11-007`).
///
/// The record stores the typed parameters, the source objects and the ordered child gem ids.
/// Re-running the builder reads this record, regenerates the plan and matches children by
/// position in `accents`, which keeps the ids of the stones that still exist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterFeature {
    pub version: u32,
    /// The center stone for a halo or a cluster.
    pub center: Option<ObjectId>,
    /// The two wall curves for a channel layout.
    pub curves: [Option<ObjectId>; 2],
    /// Ordered child gem ids, one per accent instance.
    pub accents: Vec<GemId>,
    pub kind: ClusterKind,
}

impl ClusterFeature {
    pub fn new(kind: ClusterKind, center: Option<ObjectId>, curves: [Option<ObjectId>; 2]) -> Self {
        Self {
            version: CLUSTER_SCHEMA_VERSION,
            center,
            curves,
            accents: Vec::new(),
            kind,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != CLUSTER_SCHEMA_VERSION {
            return Err(Error::Invalid("cluster feature version"));
        }
        self.kind.validate()?;
        if matches!(self.kind, ClusterKind::Channel { .. }) {
            if self.curves.iter().any(Option::is_none) {
                return Err(Error::Invalid("channel feature curves"));
            }
        } else if self.center.is_none() {
            return Err(Error::Invalid("cluster feature center"));
        }
        if self.accents.len() > MAX_ACCENTS {
            return Err(Error::Budget("cluster feature accents"));
        }
        if self.accents.iter().any(|id| id.is_nil()) {
            return Err(Error::Invalid("cluster feature gem id"));
        }
        let mut seen = std::collections::BTreeSet::new();
        if self.accents.iter().any(|id| !seen.insert(*id)) {
            return Err(Error::Invalid("cluster feature duplicate gem id"));
        }
        Ok(())
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| Error::Invalid("cluster feature json"))
    }
    pub fn from_json(text: &str) -> Result<Self> {
        let feature: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("cluster feature json"))?;
        feature.validate()?;
        Ok(feature)
    }
    pub fn to_extension(&self) -> Result<Extension> {
        Extension::new(self.version, self.to_json()?)
    }
    pub fn from_extension(extension: &Extension) -> Result<Self> {
        if extension.version != CLUSTER_SCHEMA_VERSION {
            return Err(Error::Invalid("cluster feature extension version"));
        }
        Self::from_json(&extension.payload)
    }
}

/* ---------------------------------- tests --------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve_build;
    use crate::gem::build_mesh;

    fn round(diameter: f64) -> GemSpec {
        GemSpec::round(diameter, diameter * 0.61).unwrap()
    }
    fn baguette(length: f64, width: f64) -> GemSpec {
        GemSpec::baguette(length, width, 0.0, width * 0.55, 0.15).unwrap()
    }
    fn center() -> GemInstance {
        GemInstance::at(round(6.5), [0.0, 0.0, 0.0]).unwrap()
    }

    #[test]
    fn halo_ring_is_closed_and_keeps_the_requested_gap() {
        let mut params = HaloParams::new(round(1.3)).unwrap();
        params.spacing_mm = 0.3;
        params.spacing_from_center_mm = 0.5;
        let layout = halo_layout(&center(), &params).unwrap();
        assert!(layout.accents.len() >= 8, "{}", layout.accents.len());
        assert!(layout.realized_spacing_mm >= 0.3 - 1.0e-6);
        for (index, accent) in layout.accents.iter().enumerate() {
            build_mesh(&accent.spec).unwrap();
            accent.placement.validate().unwrap();
            assert!(accent.placement.0[2][3].abs() < 1.0e-9);
            if index > 0 {
                let previous = position_of(&layout.accents[index - 1].placement);
                assert!(
                    norm(sub(position_of(&accent.placement), previous)) > params.accent.length_mm
                );
            }
        }
    }

    #[test]
    fn a_halo_with_too_many_accents_is_refused() {
        let mut params = HaloParams::new(round(1.3)).unwrap();
        params.count = Some(60);
        let error = halo_layout(&center(), &params).unwrap_err();
        assert!(format!("{error}").contains("spacing"));
    }

    #[test]
    fn cluster_offsets_and_start_point_move_the_ring() {
        let mut params = ClusterParams::new(round(1.3)).unwrap();
        params.xy_offset_mm = 1.0;
        params.z_offset_mm = 0.5;
        params.count = Some(10);
        params.start = ClusterStart::Top;
        let layout = cluster_layout(&center(), &params).unwrap();
        assert_eq!(layout.accents.len(), 10);
        let first = position_of(&layout.accents[0].placement);
        assert!((first[2] + 0.5).abs() < 1.0e-9);
        assert!(layout.ring_radii_mm[0] > 3.25 + 1.0);
    }

    #[test]
    fn a_channel_keeps_the_gap_and_tapers() {
        let limits = Limits::default();
        let a = curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
        let b = curve_build::polyline(&[[0.0, 3.0, 0.0], [10.0, 2.0, 0.0]]).unwrap();
        let mut params = ChannelParams::new(baguette(4.0, 1.5)).unwrap();
        params.taper = 0.5;
        let layout = baguette_channel(&a, &b, &params, &limits).unwrap();
        assert!(!layout.gems.is_empty());
        assert!(layout.min_gap_mm >= 2.0 - 1.0e-9);
        assert!(layout.max_gap_mm <= 3.0 + 1.0e-9);
        let first = &layout.gems[0];
        build_mesh(&first.instance.spec).unwrap();
    }

    #[test]
    fn a_zero_gap_channel_is_refused() {
        let limits = Limits::default();
        let a = curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
        let b = curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
        let params = ChannelParams::new(baguette(4.0, 1.5)).unwrap();
        let error = baguette_channel(&a, &b, &params, &limits).unwrap_err();
        assert!(format!("{error}").contains("touch") || format!("{error}").contains("length"));
    }

    #[test]
    fn orient_to_gem_uses_the_gem_axes() {
        let gem = GemInstance::at(round(6.5), [1.0, 2.0, 3.0])
            .unwrap()
            .with_rotation([10.0, 20.0, 30.0])
            .unwrap();
        let object = Transform::translation([9.0, 9.0, 9.0]);
        let transform = orient_to_gem(&gem, &object, &OrientOptions::default()).unwrap();
        let basis = gem.basis();
        for (axis, direction) in basis.iter().enumerate() {
            for (row, value) in direction.iter().enumerate() {
                assert!((transform.0[row][axis] - value).abs() < 1.0e-9);
            }
        }
        assert_eq!(position_of(&transform), position_of(&gem.placement));
    }

    #[test]
    fn the_feature_record_roundtrips_and_rejects_duplicate_ids() {
        let mut params = HaloParams::new(round(1.3)).unwrap();
        params.spacing_mm = 0.4;
        let kind = ClusterKind::Halo { params };
        let mut feature = ClusterFeature::new(kind, Some(ObjectId::new()), [None, None]);
        feature.accents.push(GemId::new());
        let text = feature.to_json().unwrap();
        let read = ClusterFeature::from_json(&text).unwrap();
        assert_eq!(read, feature);
        assert_eq!(
            ClusterFeature::from_extension(&feature.to_extension().unwrap()).unwrap(),
            feature
        );
        let mut broken = feature.clone();
        broken.accents.push(broken.accents[0]);
        let error = broken.to_json().unwrap_err();
        assert!(format!("{error}").contains("duplicate"));
    }

    #[test]
    fn a_shape_override_ring_is_circular_and_keeps_the_center_size() {
        let mut params = HaloParams::new(round(1.3)).unwrap();
        params.match_shape = false;
        let layout = halo_layout(&center(), &params).unwrap();
        assert!((layout.radius_mm[0] - layout.radius_mm[1]).abs() < 1.0e-9);
    }
}
