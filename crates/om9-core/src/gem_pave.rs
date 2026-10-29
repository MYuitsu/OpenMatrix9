// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pavé stone layout, azure, prong markers, sphere layout and relaxation (`OM9-L2-04-12`).
//!
//! Every entry point is a pure plan: it reads a surface and/or already placed gems and returns
//! [`GemInstance`]s, azure polylines or prong markers. Nothing here mutates a document or writes a
//! history step, so the host can preview and Cancel without a half-built layout.
//!
//! Owner split:
//!
//! * [`pave_surface`] — Auto/Manual Pavé stone layout over one surface (`OM9-L3-04-12-001/002`).
//! * [`pave_azures`] — top/through-hole/bottom/seat azure profiles per placed stone (`-003`).
//! * [`pave_prongs`] — unshared/shared/centre prong markers from the contact graph (`-004`).
//! * [`pave_sphere`] — stone layout on a sphere without a surface (`-005`).
//! * [`spring_relax`] — distance-averaging relaxation of existing stones on a surface (`-006`).
//! * [`PavePreview`] — what the builder paints (stones or the geodesic framework) (`-007`).
//! * [`PaveFeature`] / [`refresh_pave_layout`] — persisted UV sites that survive a surface edit and
//!   keep the stone ids (`-008`).
//! * [`PaveStyle`] / [`PaveStyleLibrary`] — named parameter snapshots for Save/Load Style.
//!
//! Rules: document millimetres, right-handed gem frames with local `+Z` as the table axis, no
//! second placement primitive (surface frames reuse `surface_kernel` and `gem_placement`), and a
//! layout that cannot honour its spacing is refused by name rather than silently squeezed.
//! An open selected curve is sampled as an attractor, closed curves clip the region, and the
//! persisted feature keeps their source ids so `Refresh` follows document edits.

use crate::{
    Error, Result,
    budget::Limits,
    gem::{GemId, GemInstance, GemSpec},
    geometry::{Curve, NurbsSurface, Point3, Transform},
    identity::ObjectId,
    model::Extension,
};
use serde::{Deserialize, Serialize};

/// Version of the persisted pavé feature record.
pub const PAVE_SCHEMA_VERSION: u32 = 1;
/// Extension slot of a pavé feature record.
pub const PAVE_EXTENSION_SLOT: &str = "om9.gem.pave";
/// Hard cap on stones in one pavé layout.
pub const MAX_PAVE_GEMS: usize = 1024;
/// Schema tag of a pavé style file.
pub const PAVE_STYLE_SCHEMA: &str = "om9.gem.pave_style.v1";
/// Longest accepted pavé style name, in bytes.
pub const MAX_PAVE_STYLE_NAME_BYTES: usize = 64;
/// Largest number of user styles in one file.
pub const MAX_PAVE_STYLES: usize = 256;

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
fn finite(value: f64, name: &'static str) -> Result<f64> {
    if !value.is_finite() {
        return Err(Error::Invalid(name));
    }
    Ok(value)
}
fn in_range(value: f64, low: f64, high: f64, name: &'static str) -> Result<()> {
    finite(value, name)?;
    if value < low || value > high {
        return Err(Error::Invalid(name));
    }
    Ok(())
}
fn frame(origin: Point3, x: Point3, z: Point3) -> Result<Transform> {
    let z = unit(z, "pave surface normal")?;
    let x = unit(sub(x, mul(z, dot(x, z))), "pave surface tangent")?;
    let y = cross(z, x);
    Ok(Transform([
        [x[0], y[0], z[0], origin[0]],
        [x[1], y[1], z[1], origin[1]],
        [x[2], y[2], z[2], origin[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]))
}
fn rotate_uv([u, v]: [f64; 2], degrees: f64) -> [f64; 2] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [u * cos - v * sin, u * sin + v * cos]
}
fn scaled(spec: &GemSpec, factor: f64, min_mm: f64, max_mm: f64) -> Result<GemSpec> {
    let length = (spec.length_mm * factor).clamp(min_mm, max_mm);
    let width = (spec.width_mm * factor).clamp(min_mm.min(length), max_mm.min(length));
    spec.clone().set_girdle_mm(length, width)
}

/* -------------------------------- parameters ------------------------------ */

/// Auto Pavé lattice patterns (`OM9-L3-04-12-001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PavePattern {
    Hex,
    Square,
    Tri,
    HexPent,
    AllDirections,
    HexReverse,
}

impl PavePattern {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hex => "Hex",
            Self::Square => "Square",
            Self::Tri => "Tri",
            Self::HexPent => "Hex Pent",
            Self::AllDirections => "All Directions",
            Self::HexReverse => "Hexr",
        }
    }
}

/// What the first station follows (`OM9-L3-04-12-001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaveStart {
    BaseDirection,
    Curve,
    Edges,
    SeedGems,
}

/// Pre/post sizing correction (`OM9-L3-04-12-001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaveCorrection {
    Normal,
    Basic,
    Grow,
    Shrink,
}

impl PaveCorrection {
    fn factor(self) -> f64 {
        match self {
            Self::Normal => 1.0,
            Self::Basic => 0.95,
            Self::Grow => 1.08,
            Self::Shrink => 0.92,
        }
    }
}

/// Typed Auto/Manual Pavé parameters (`OM9-L3-04-12-001`, `-002`, `-007`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaveParams {
    /// The stone loaded into the builder; its girdle is the base size.
    pub spec: GemSpec,
    pub pattern: PavePattern,
    pub start: PaveStart,
    /// Spacing between neighbouring girdles, in mm.
    pub spacing_mm: f64,
    /// Hard minimum spacing; the relax pass never moves two stones closer than this.
    pub min_spacing_mm: f64,
    /// Base size of the first stones, in mm.
    pub base_size_mm: f64,
    pub min_size_mm: f64,
    pub max_size_mm: f64,
    /// Inset from the surface domain before the first stone, in mm.
    pub margin_mm: f64,
    /// Inset from the region border for sizing, in mm (`OM9-L3-04-12-007`).
    pub border_mm: f64,
    /// Lattice rotation around the start point, in degrees.
    pub angle_deg: f64,
    /// Mirror the lattice across the start row (Auto Pavé Flip).
    #[serde(default)]
    pub flip: bool,
    /// Size variation from the inside out, `0..=1`.
    pub taper: f64,
    /// Pull all stones toward the mean size, `0..=1`.
    pub equalize: f64,
    /// Deterministic size/spacing randomness, `0..=1`.
    pub perturb: f64,
    pub pre: PaveCorrection,
    pub post: PaveCorrection,
    /// Relax iterations after the lattice is built.
    pub relax_iterations: u32,
    /// Fraction of the overlap removed per iteration, `0..=1`.
    pub relax_strength: f64,
    /// Start parameter (surface UV).
    pub start_uv: [f64; 2],
    /// Optional seed parameters from Gem on Curve/Surface; they are added as extra stations.
    pub seeds: Vec<[f64; 2]>,
    /// Open curve that attracts the lattice: stones grow and tighten near it.
    #[serde(default)]
    pub attractor: Option<Curve>,
    /// Closed curves that clip the lattice to their union.
    #[serde(default)]
    pub edge_curves: Vec<Curve>,
    /// Attractor falloff strength, `0..=1`.
    #[serde(default)]
    pub attractor_strength: f64,
    /// Attractor falloff radius in millimetres.
    #[serde(default)]
    pub attractor_radius_mm: f64,
}

impl PaveParams {
    /// A 1.5 mm stone with 1.8 mm spacing and a hexagonal lattice.
    pub fn new(spec: GemSpec) -> Result<Self> {
        let params = Self {
            base_size_mm: spec.length_mm,
            min_size_mm: (spec.length_mm * 0.4).max(0.4),
            max_size_mm: spec.length_mm * 1.6,
            spec,
            pattern: PavePattern::Hex,
            start: PaveStart::BaseDirection,
            spacing_mm: 1.8,
            min_spacing_mm: 0.15,
            margin_mm: 0.3,
            border_mm: 0.4,
            angle_deg: 0.0,
            flip: false,
            taper: 0.0,
            equalize: 0.0,
            perturb: 0.0,
            pre: PaveCorrection::Normal,
            post: PaveCorrection::Normal,
            relax_iterations: 4,
            relax_strength: 0.5,
            start_uv: [0.5, 0.5],
            seeds: Vec::new(),
            attractor: None,
            edge_curves: Vec::new(),
            attractor_strength: 0.6,
            attractor_radius_mm: 5.0,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        self.spec.validate()?;
        in_range(self.spacing_mm, 0.1, 100.0, "pave spacing")?;
        in_range(
            self.min_spacing_mm,
            0.0,
            self.spacing_mm,
            "pave minimum spacing",
        )?;
        in_range(self.base_size_mm, 0.1, 100.0, "pave base size")?;
        in_range(
            self.min_size_mm,
            0.1,
            self.base_size_mm,
            "pave minimum size",
        )?;
        in_range(
            self.max_size_mm,
            self.base_size_mm,
            200.0,
            "pave maximum size",
        )?;
        in_range(self.margin_mm, 0.0, 100.0, "pave margin")?;
        in_range(self.border_mm, 0.0, 100.0, "pave border")?;
        in_range(self.angle_deg, -360.0, 360.0, "pave angle")?;
        for (value, name) in [
            (self.taper, "pave taper"),
            (self.equalize, "pave equalize"),
            (self.perturb, "pave perturb"),
            (self.relax_strength, "pave relax strength"),
        ] {
            in_range(value, 0.0, 1.0, name)?;
        }
        if self.relax_iterations > 64 {
            return Err(Error::Invalid("pave relax iterations"));
        }
        if !self.start_uv.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("pave start"));
        }
        if self.seeds.len() > MAX_PAVE_GEMS {
            return Err(Error::Budget("pave seeds"));
        }
        if self.seeds.iter().flatten().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("pave seed"));
        }
        in_range(self.attractor_strength, 0.0, 1.0, "pave attractor strength")?;
        in_range(
            self.attractor_radius_mm,
            0.1,
            500.0,
            "pave attractor radius",
        )?;
        if self.edge_curves.len() > 32 {
            return Err(Error::Budget("pave edge curves"));
        }
        Ok(())
    }
}

/// One planned stone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaveStation {
    pub index: usize,
    pub uv: [f64; 2],
    pub point: Point3,
    pub normal: Point3,
    pub tangent: Point3,
    pub spec: GemSpec,
    pub transform: Transform,
    /// Existing stone identity when refreshing a feature.
    pub gem: Option<GemId>,
}

/// The whole pure stone plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaveLayout {
    pub params: PaveParams,
    pub stations: Vec<PaveStation>,
}

impl PaveStation {
    /// The unwritten preview instance of this station.
    pub fn instance(&self) -> GemInstance {
        GemInstance {
            spec: self.spec.clone(),
            placement: self.transform,
            label: "Pavé".to_owned(),
        }
    }
}

impl PaveLayout {
    pub fn instances(&self) -> Vec<GemInstance> {
        self.stations.iter().map(PaveStation::instance).collect()
    }
}

/* --------------------------------- lattice -------------------------------- */

fn domain_metric(surface: &NurbsSurface, uv: [f64; 2], limits: &Limits) -> Result<[f64; 2]> {
    let derivatives = crate::surface_kernel::derivatives(surface, uv[0], uv[1], limits)?;
    let du = norm(derivatives.du);
    let dv = norm(derivatives.dv);
    if !du.is_finite() || !dv.is_finite() || du <= 1.0e-9 || dv <= 1.0e-9 {
        return Err(Error::Invalid("pave surface metric"));
    }
    Ok([du, dv])
}

fn inside(surface: &NurbsSurface, uv: [f64; 2], inset: [f64; 2]) -> bool {
    let d = surface.domain;
    let span_u = d[0][1] - d[0][0];
    let span_v = d[1][1] - d[1][0];
    uv[0] >= d[0][0] + inset[0]
        && uv[0] <= d[0][1] - inset[0]
        && uv[1] >= d[1][0] + inset[1]
        && uv[1] <= d[1][1] - inset[1]
        && span_u > 2.0 * inset[0]
        && span_v > 2.0 * inset[1]
}

/// Project a closed edge curve onto the surface parameter plane. Every point must project
/// within the surface tolerance; an open curve is refused by name.
fn edge_loops(
    surface: &NurbsSurface,
    curves: &[Curve],
    limits: &Limits,
) -> Result<Vec<Vec<[f64; 2]>>> {
    let mut loops = Vec::new();
    for curve in curves {
        curve.validate(limits)?;
        if !crate::curve_ops::is_closed(curve, 1.0e-6, limits)? {
            return Err(Error::Invalid("pave edge curve must be closed"));
        }
        let points = crate::curve_ops::sample_curve(curve, 64, limits)?;
        let mut uv = Vec::with_capacity(points.len());
        for point in points {
            let projection =
                crate::curve_surface_ops::closest_point_on_surface(surface, point, limits)?;
            uv.push(projection.parameters);
        }
        if uv.len() < 3 {
            return Err(Error::Invalid("pave edge curve has too few points"));
        }
        loops.push(uv);
    }
    Ok(loops)
}

/// Ray casting in the surface parameter plane.
fn inside_loop(uv: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[j];
        if (a[1] > uv[1]) != (b[1] > uv[1]) {
            let x = (b[0] - a[0]) * (uv[1] - a[1]) / (b[1] - a[1]) + a[0];
            if uv[0] < x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

fn inside_edges(uv: [f64; 2], loops: &[Vec<[f64; 2]>]) -> bool {
    loops.is_empty() || loops.iter().any(|polygon| inside_loop(uv, polygon))
}

/// Where the lattice starts. `Curve` follows the attractor start, `Edges` starts at the
/// first edge loop centroid, and `SeedGems` follows the first seed.
fn effective_start(
    surface: &NurbsSurface,
    params: &PaveParams,
    loops: &[Vec<[f64; 2]>],
    limits: &Limits,
) -> Result<[f64; 2]> {
    Ok(match params.start {
        PaveStart::Curve => {
            let curve = params
                .attractor
                .as_ref()
                .ok_or(Error::Missing("pave attractor"))?;
            let point = crate::curve_ops::point_at(curve, 0.5, limits)?;
            let projection =
                crate::curve_surface_ops::closest_point_on_surface(surface, point, limits)?;
            projection.parameters
        }
        PaveStart::Edges => loops
            .first()
            .map(|polygon| {
                let count = polygon.len() as f64;
                polygon.iter().fold([0.0, 0.0], |acc, point| {
                    [acc[0] + point[0] / count, acc[1] + point[1] / count]
                })
            })
            .ok_or(Error::Missing("pave edge curves"))?,
        PaveStart::SeedGems => params.seeds.first().copied().unwrap_or(params.start_uv),
        PaveStart::BaseDirection => params.start_uv,
    })
}

fn attractor_factor(params: &PaveParams, point: Point3, limits: &Limits) -> Result<f64> {
    let Some(curve) = &params.attractor else {
        return Ok(1.0);
    };
    let (_, _, distance) = crate::curve_surface_ops::closest_point_on_curve(curve, point, limits)?;
    let radius = params.attractor_radius_mm.max(0.1);
    let falloff = (-(distance / radius).powi(2)).exp();
    Ok((1.0 - params.attractor_strength * falloff * 0.35).clamp(0.4, 1.0))
}

fn lattice_offsets(pattern: PavePattern, i: i64, j: i64) -> Vec<[f64; 2]> {
    let half = 0.5;
    match pattern {
        PavePattern::Square => vec![[i as f64, j as f64]],
        PavePattern::Hex | PavePattern::HexReverse => {
            vec![[
                i as f64 + if j.rem_euclid(2) == 1 { half } else { 0.0 },
                j as f64,
            ]]
        }
        PavePattern::Tri => {
            vec![[
                i as f64 + if j.rem_euclid(3) == 0 { 0.0 } else { 1.0 / 3.0 },
                j as f64 * 0.75,
            ]]
        }
        PavePattern::HexPent => {
            let mut out = Vec::new();
            let center = [i as f64 * 2.5, j as f64 * 2.5 * 0.866];
            out.push(center);
            for k in 0..5 {
                let angle = std::f64::consts::TAU * k as f64 / 5.0;
                out.push([center[0] + angle.cos(), center[1] + angle.sin()]);
            }
            out
        }
        PavePattern::AllDirections => vec![[i as f64, j as f64]],
    }
}

/// Deterministic hash in `0..1`, used for `Perturb` so a rerun is reproducible.
fn jitter(index: usize) -> f64 {
    let value = (index as f64 * 12.9898 + 78.233).sin() * 43_758.545_3;
    value - value.floor()
}

/// Build a pavé stone plan on one surface (`OM9-L3-04-12-001`, `-002`, `-007`).
///
/// The lattice is generated in the surface's UV parameter plane with the local metric at the start
/// point, so a millimetre spacing is exact on an isometric surface and bounded elsewhere. Every
/// station is evaluated through `surface_kernel`; a parameter outside the domain or a pole is
/// refused by name.
pub fn pave_surface(
    surface: &NurbsSurface,
    params: &PaveParams,
    limits: &Limits,
) -> Result<PaveLayout> {
    surface.validate(limits)?;
    params.validate()?;
    for curve in params.edge_curves.iter().chain(params.attractor.iter()) {
        curve.validate(limits)?;
    }
    let loops = edge_loops(surface, &params.edge_curves, limits)?;
    let start_uv = effective_start(surface, params, &loops, limits)?;
    let metric = domain_metric(surface, start_uv, limits)?;
    let step_uv = [params.spacing_mm / metric[0], params.spacing_mm / metric[1]];
    if step_uv[0] <= 0.0 || step_uv[1] <= 0.0 {
        return Err(Error::Invalid("pave lattice step"));
    }
    let domain = surface.domain;
    let span = [domain[0][1] - domain[0][0], domain[1][1] - domain[1][0]];
    let margin = [params.margin_mm / metric[0], params.margin_mm / metric[1]];
    let border = [params.border_mm / metric[0], params.border_mm / metric[1]];
    if !inside(surface, start_uv, margin) {
        return Err(Error::Invalid("pave start outside the region"));
    }

    // Rotated lattice basis in parameter space. On a planar surface this is a true rotation.
    let row = step_uv[1]
        * if matches!(params.pattern, PavePattern::Square) {
            1.0
        } else {
            0.866_025_4
        };
    let base_u = rotate_uv([step_uv[0], 0.0], params.angle_deg);
    let base_v = rotate_uv(
        [0.0, row * if params.flip { -1.0 } else { 1.0 }],
        params.angle_deg,
    );
    let count_u = ((span[0] / step_uv[0]).ceil() as i64 + 3).min(256);
    let count_v = ((span[1] / row).ceil() as i64 + 3).min(256);

    let mut candidates: Vec<[f64; 2]> = Vec::new();
    for j in -count_v..=count_v {
        for i in -count_u..=count_u {
            for local in lattice_offsets(params.pattern, i, j) {
                let mut uv = [
                    start_uv[0] + base_u[0] * local[0] + base_v[0] * local[1],
                    start_uv[1] + base_u[1] * local[0] + base_v[1] * local[1],
                ];
                if params.pattern == PavePattern::AllDirections {
                    uv[0] += (jitter(candidates.len()) - 0.5) * step_uv[0] * 0.45;
                    uv[1] += (jitter(candidates.len() + 17) - 0.5) * step_uv[1] * 0.45;
                }
                if inside(surface, uv, margin) && inside_edges(uv, &loops) {
                    candidates.push(uv);
                }
            }
        }
    }
    candidates.extend(
        params
            .seeds
            .iter()
            .copied()
            .filter(|uv| inside(surface, *uv, margin) && inside_edges(*uv, &loops)),
    );
    if let Some(curve) = &params.attractor {
        for point in crate::curve_ops::sample_curve(curve, 128, limits)? {
            let projection =
                crate::curve_surface_ops::closest_point_on_surface(surface, point, limits)?;
            let uv = projection.parameters;
            if inside(surface, uv, margin) && inside_edges(uv, &loops) {
                candidates.push(uv);
            }
        }
    }
    if candidates.is_empty() {
        return Err(Error::Invalid("pave region has no room"));
    }
    if candidates.len() > MAX_PAVE_GEMS {
        return Err(Error::Budget("pave gems"));
    }

    // Relax in UV: remove overlaps below the minimum spacing.
    let relax_step = [
        params.min_spacing_mm / metric[0],
        params.min_spacing_mm / metric[1],
    ];
    for _ in 0..params.relax_iterations {
        for a in 0..candidates.len() {
            for b in (a + 1)..candidates.len() {
                let delta = [
                    candidates[a][0] - candidates[b][0],
                    candidates[a][1] - candidates[b][1],
                ];
                // The step is anisotropic; normalize each axis by the local step.
                let scaled = [
                    delta[0] / relax_step[0].max(1.0e-12),
                    delta[1] / relax_step[1].max(1.0e-12),
                ];
                let distance = scaled[0].hypot(scaled[1]);
                if distance > 0.0 && distance < 1.0 {
                    let push = (1.0 - distance) * params.relax_strength * 0.5;
                    let direction = [scaled[0] / distance, scaled[1] / distance];
                    candidates[a][0] -= direction[0] * push * relax_step[0];
                    candidates[a][1] -= direction[1] * push * relax_step[1];
                    candidates[b][0] += direction[0] * push * relax_step[0];
                    candidates[b][1] += direction[1] * push * relax_step[1];
                }
            }
        }
    }
    for uv in &mut candidates {
        uv[0] = uv[0].clamp(domain[0][0] + margin[0], domain[0][1] - margin[0]);
        uv[1] = uv[1].clamp(domain[1][0] + margin[1], domain[1][1] - margin[1]);
    }

    let mut stations = Vec::new();
    let mean = params.base_size_mm;
    for (index, uv) in candidates.iter().enumerate() {
        let derivatives = crate::surface_kernel::derivatives(surface, uv[0], uv[1], limits)?;
        let normal = crate::surface_kernel::normal(surface, uv[0], uv[1], limits)?;
        let tangent = unit(derivatives.du, "pave surface tangent")?;
        let to_edge = [
            (uv[0] - domain[0][0]) / span[0].max(1.0e-12),
            (domain[0][1] - uv[0]) / span[0].max(1.0e-12),
            (uv[1] - domain[1][0]) / span[1].max(1.0e-12),
            (domain[1][1] - uv[1]) / span[1].max(1.0e-12),
        ];
        let edge = to_edge.iter().fold(1.0_f64, |acc, value| acc.min(*value));
        let border_fraction = if border[0] > 0.0 || border[1] > 0.0 {
            (edge * params.spacing_mm / params.border_mm.max(0.1)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let taper = 1.0 - params.taper * (1.0 - border_fraction);
        let equalize = 1.0 + params.equalize * (mean / params.base_size_mm - 1.0).clamp(-1.0, 1.0);
        let perturb = 1.0 + (jitter(index) - 0.5) * 0.5 * params.perturb;
        let correction = params.pre.factor() * params.post.factor();
        let edge_factor = 0.75 + 0.25 * border_fraction;
        let factor = taper
            * equalize
            * perturb
            * correction
            * edge_factor
            * attractor_factor(params, derivatives.point, limits)?;
        let spec = scaled(&params.spec, factor, params.min_size_mm, params.max_size_mm)?;
        stations.push(PaveStation {
            index,
            uv: *uv,
            point: derivatives.point,
            normal,
            tangent,
            transform: frame(derivatives.point, tangent, normal)?,
            spec,
            gem: None,
        });
    }
    // HexReverse grows from the edge inward: the outside stones are the largest.
    if params.pattern == PavePattern::HexReverse {
        for station in &mut stations {
            let d = station.uv;
            let edge = (d[0] - domain[0][0])
                .abs()
                .min((domain[0][1] - d[0]).abs())
                .min((d[1] - domain[1][0]).abs())
                .min((domain[1][1] - d[1]).abs())
                / span[0].max(span[1]).max(1.0e-12);
            let factor = (1.2 - 0.4 * edge).clamp(0.7, 1.2);
            station.spec = scaled(
                &station.spec,
                factor,
                params.min_size_mm,
                params.max_size_mm,
            )?;
        }
    }
    stations.sort_by(|a, b| a.uv.partial_cmp(&b.uv).unwrap_or(std::cmp::Ordering::Equal));
    for (index, station) in stations.iter_mut().enumerate() {
        station.index = index;
    }
    Ok(PaveLayout {
        params: params.clone(),
        stations,
    })
}

/* ---------------------------------- azure --------------------------------- */

/// Azure profile parameters (`OM9-L3-04-12-003`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AzureParams {
    /// Hexagonal azure instead of a circular one.
    pub hexagonal: bool,
    /// Rotation of the azure around the gem table axis, in degrees.
    pub angle_deg: f64,
    /// Distance from the girdle plane to the bottom profile, in mm.
    pub azure_depth_mm: f64,
    /// Extra radius beyond the stone girdle at the top, in mm.
    pub azure_size_mm: f64,
    /// Middle through-hole diameter, in mm.
    pub through_hole_mm: f64,
    /// Seat wall angle from the table axis, in degrees (`0..89`).
    pub seat_angle_deg: f64,
    /// Minimum allowed spacing between two azure top curves, in mm.
    pub min_spacing_mm: f64,
    /// Uniform scale of the azure profiles.
    pub scale: f64,
}

impl AzureParams {
    pub fn new() -> Result<Self> {
        let params = Self {
            hexagonal: false,
            angle_deg: 0.0,
            azure_depth_mm: 2.0,
            azure_size_mm: 0.2,
            through_hole_mm: 0.8,
            seat_angle_deg: 0.0,
            min_spacing_mm: 0.1,
            scale: 1.0,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        in_range(self.angle_deg, -360.0, 360.0, "azure angle")?;
        in_range(self.azure_depth_mm, 0.05, 50.0, "azure depth")?;
        in_range(self.azure_size_mm, 0.0, 20.0, "azure size")?;
        in_range(self.through_hole_mm, 0.05, 20.0, "azure through hole")?;
        in_range(self.seat_angle_deg, 0.0, 89.0, "azure seat angle")?;
        in_range(self.min_spacing_mm, 0.0, 10.0, "azure minimum spacing")?;
        in_range(self.scale, 0.1, 10.0, "azure scale")?;
        Ok(())
    }
}

/// One planned azure: four closed profiles from the girdle to the bottom.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzureProfile {
    pub gem: GemId,
    pub hexagonal: bool,
    pub top: Vec<Point3>,
    pub through: Vec<Point3>,
    pub bottom: Vec<Point3>,
    pub seat: Vec<Point3>,
    pub depth_mm: f64,
}

/// Azure plan for a stone arrangement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AzurePlan {
    pub params: AzureParams,
    pub azures: Vec<AzureProfile>,
}

fn circle_points(center: Point3, x: Point3, y: Point3, radius: f64, count: usize) -> Vec<Point3> {
    (0..count)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / count as f64;
            add(
                center,
                add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())),
            )
        })
        .collect()
}

fn hex_points(center: Point3, x: Point3, y: Point3, radius: f64) -> Vec<Point3> {
    (0..6)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / 6.0;
            add(
                center,
                add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())),
            )
        })
        .collect()
}

/// Build top/through-hole/bottom/seat azure profiles for every placed stone
/// (`OM9-L3-04-12-003`). The profiles are polylines; the host turns each into a closed curve.
pub fn pave_azures(gems: &[GemInstance], params: &AzureParams) -> Result<AzurePlan> {
    params.validate()?;
    if gems.is_empty() {
        return Err(Error::Missing("azure gems"));
    }
    if gems.len() > MAX_PAVE_GEMS {
        return Err(Error::Budget("azure gems"));
    }
    let mut azures = Vec::new();
    let tan = params.seat_angle_deg.to_radians().tan();
    for gem in gems {
        let basis = gem.basis();
        let center = [
            gem.placement.0[0][3],
            gem.placement.0[1][3],
            gem.placement.0[2][3],
        ];
        let z = basis[2];
        let x = basis[0];
        let y = basis[1];
        let top_radius = (gem.spec.length_mm * 0.5 + params.azure_size_mm) * params.scale;
        let through_radius = (params.through_hole_mm * 0.5 * params.scale).min(top_radius);
        let bottom_radius = (top_radius - params.azure_depth_mm * tan).max(through_radius * 0.8);
        let top_center = add(center, mul(z, 0.02));
        let through_center = add(center, mul(z, -params.azure_depth_mm * 0.5));
        let bottom_center = add(center, mul(z, -params.azure_depth_mm));
        let seat_center = add(center, mul(z, -params.azure_depth_mm * 0.25));
        let points = |center: Point3, radius: f64| {
            if params.hexagonal {
                hex_points(center, x, y, radius)
            } else {
                circle_points(center, x, y, radius, 48)
            }
        };
        azures.push(AzureProfile {
            gem: GemId::new(),
            hexagonal: params.hexagonal,
            top: points(top_center, top_radius),
            through: points(through_center, through_radius),
            bottom: points(bottom_center, bottom_radius),
            seat: points(seat_center, top_radius * 0.82),
            depth_mm: params.azure_depth_mm,
        });
    }
    Ok(AzurePlan {
        params: params.clone(),
        azures,
    })
}

/* ---------------------------------- prongs -------------------------------- */

/// Prong family (`OM9-L3-04-12-004`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProngKind {
    /// Touches one stone on the outside edge.
    Unshared,
    /// Touches two stones.
    Shared,
    /// Touches three stones.
    Center,
}

/// Typed pavé prong parameters (`OM9-L3-04-12-004`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProngParams {
    /// Top rounding, `0..=1` (1 is a half sphere).
    pub fillet: f64,
    /// Height above the girdle plane, in mm.
    pub height_mm: f64,
    /// Base offset into the surface, in mm.
    pub drop_mm: f64,
    /// Top/bottom size ratio, `0.2..=2`.
    pub taper: f64,
    /// Uniform scale of every prong.
    pub scale: f64,
    pub shared_size_mm: f64,
    pub unshared_size_mm: f64,
    pub center_min_mm: f64,
    pub center_max_mm: f64,
    /// Two stones closer than this fraction of their radii sum are touching.
    pub touch_fraction: f64,
}

impl ProngParams {
    pub fn new() -> Result<Self> {
        let params = Self {
            fillet: 0.5,
            height_mm: 1.0,
            drop_mm: 0.0,
            taper: 1.0,
            scale: 1.0,
            shared_size_mm: 0.5,
            unshared_size_mm: 0.6,
            center_min_mm: 0.3,
            center_max_mm: 1.2,
            touch_fraction: 1.08,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        in_range(self.fillet, 0.0, 1.0, "prong fillet")?;
        in_range(self.height_mm, 0.05, 20.0, "prong height")?;
        in_range(self.drop_mm, -10.0, 10.0, "prong drop")?;
        in_range(self.taper, 0.2, 2.0, "prong taper")?;
        in_range(self.scale, 0.1, 10.0, "prong scale")?;
        in_range(self.shared_size_mm, 0.05, 10.0, "shared prong size")?;
        in_range(self.unshared_size_mm, 0.05, 10.0, "unshared prong size")?;
        in_range(self.center_min_mm, 0.05, 10.0, "center prong minimum")?;
        in_range(
            self.center_max_mm,
            self.center_min_mm,
            20.0,
            "center prong maximum",
        )?;
        in_range(self.touch_fraction, 1.0, 1.5, "prong touch fraction")?;
        Ok(())
    }
}

/// One placed prong marker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaveProng {
    pub kind: ProngKind,
    pub position: Point3,
    pub normal: Point3,
    pub gems: Vec<GemId>,
    pub base_radius_mm: f64,
    pub top_radius_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProngPlan {
    pub params: ProngParams,
    pub prongs: Vec<PaveProng>,
}

fn gem_center(gem: &GemInstance) -> Point3 {
    [
        gem.placement.0[0][3],
        gem.placement.0[1][3],
        gem.placement.0[2][3],
    ]
}

/// Build the contact graph and place unshared/shared/center prong markers
/// (`OM9-L3-04-12-004`). The marker is a typed plan; metal meshes remain an M06/M07 concern.
pub fn pave_prongs(gems: &[GemInstance], params: &ProngParams) -> Result<ProngPlan> {
    params.validate()?;
    if gems.is_empty() {
        return Err(Error::Missing("pave prongs gems"));
    }
    if gems.len() > 256 {
        return Err(Error::Budget("pave prong gems"));
    }
    let centers: Vec<Point3> = gems.iter().map(gem_center).collect();
    let radii: Vec<f64> = gems.iter().map(|gem| gem.spec.length_mm * 0.5).collect();
    let touching = |a: usize, b: usize| {
        norm(sub(centers[a], centers[b])) <= (radii[a] + radii[b]) * params.touch_fraction
    };
    let mut prongs = Vec::new();
    let mut adjacent = vec![Vec::new(); gems.len()];
    for a in 0..gems.len() {
        for b in (a + 1)..gems.len() {
            if touching(a, b) {
                adjacent[a].push(b);
                adjacent[b].push(a);
                prongs.push(PaveProng {
                    kind: ProngKind::Shared,
                    position: mul(add(centers[a], centers[b]), 0.5),
                    normal: gems[a].basis()[2],
                    gems: vec![GemId::new(), GemId::new()],
                    base_radius_mm: params.shared_size_mm * 0.5 * params.scale,
                    top_radius_mm: params.shared_size_mm * 0.5 * params.scale * params.taper,
                    height_mm: params.height_mm,
                });
            }
        }
    }
    // Triple adjacencies: every pair must be touching.
    for a in 0..gems.len() {
        for &b in &adjacent[a] {
            if b <= a {
                continue;
            }
            for &c in &adjacent[a] {
                if c <= b || !adjacent[b].contains(&c) {
                    continue;
                }
                let center = mul(add(add(centers[a], centers[b]), centers[c]), 1.0 / 3.0);
                let radius = (params.center_min_mm
                    + (params.center_max_mm - params.center_min_mm)
                        * (radii[a] + radii[b] + radii[c])
                        / 3.0
                        / 2.0)
                    * params.scale;
                prongs.push(PaveProng {
                    kind: ProngKind::Center,
                    position: center,
                    normal: gems[a].basis()[2],
                    gems: vec![GemId::new(), GemId::new(), GemId::new()],
                    base_radius_mm: radius * 0.5,
                    top_radius_mm: radius * 0.5 * params.taper,
                    height_mm: params.height_mm,
                });
            }
        }
    }
    // Unshared: a boundary stone has at least one 60-degree sector with no neighbour.
    for (index, gem) in gems.iter().enumerate() {
        let basis = gem.basis();
        let center = centers[index];
        let mut boundary = false;
        for sector in 0..6 {
            let angle = std::f64::consts::TAU * sector as f64 / 6.0;
            let direction = add(mul(basis[0], angle.cos()), mul(basis[1], angle.sin()));
            let free = adjacent[index].iter().all(|&other| {
                let delta = sub(centers[other], center);
                dot(delta, direction) <= radii[index] * 0.5
            });
            if free {
                boundary = true;
                prongs.push(PaveProng {
                    kind: ProngKind::Unshared,
                    position: add(center, mul(direction, radii[index])),
                    normal: basis[2],
                    gems: vec![GemId::new()],
                    base_radius_mm: params.unshared_size_mm * 0.5 * params.scale,
                    top_radius_mm: params.unshared_size_mm * 0.5 * params.scale * params.taper,
                    height_mm: params.height_mm,
                });
            }
        }
        if !boundary && adjacent[index].len() < 2 {
            // An isolated stone always keeps a prong so Set Prongs is never a silent no-op.
            prongs.push(PaveProng {
                kind: ProngKind::Unshared,
                position: add(center, mul(basis[0], radii[index])),
                normal: basis[2],
                gems: vec![GemId::new()],
                base_radius_mm: params.unshared_size_mm * 0.5 * params.scale,
                top_radius_mm: params.unshared_size_mm * 0.5 * params.scale * params.taper,
                height_mm: params.height_mm,
            });
        }
    }
    if prongs.len() > 4096 {
        return Err(Error::Budget("pave prongs"));
    }
    Ok(ProngPlan {
        params: params.clone(),
        prongs,
    })
}

/* -------------------------------- sphere ---------------------------------- */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemovalPattern {
    None,
    PatternA,
    PatternB,
}

/// Pavé sphere parameters (`OM9-L3-04-12-005`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SphereParams {
    pub spec: GemSpec,
    pub radius_mm: f64,
    /// Basic mode: explicit stone count. `None` derives it from X/T.
    pub count: Option<usize>,
    pub x: u32,
    pub t: u32,
    pub removal: RemovalPattern,
    pub min_size_mm: f64,
    pub relax_amount: u32,
    pub relax_strength: f64,
}

impl SphereParams {
    pub fn new(spec: GemSpec) -> Result<Self> {
        let params = Self {
            spec,
            radius_mm: 10.0,
            count: Some(120),
            x: 8,
            t: 4,
            removal: RemovalPattern::None,
            min_size_mm: 0.6,
            relax_amount: 4,
            relax_strength: 0.4,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        self.spec.validate()?;
        in_range(self.radius_mm, 0.5, 10_000.0, "pave sphere radius")?;
        if let Some(count) = self.count {
            if !(4..=MAX_PAVE_GEMS).contains(&count) {
                return Err(Error::Budget("pave sphere count"));
            }
        } else if self.x < 2 || self.t < 1 || self.x * self.t > MAX_PAVE_GEMS as u32 {
            return Err(Error::Budget("pave sphere grid"));
        }
        in_range(
            self.min_size_mm,
            0.1,
            self.spec.length_mm,
            "pave sphere minimum size",
        )?;
        if self.relax_amount > 64 {
            return Err(Error::Invalid("pave sphere relax"));
        }
        in_range(self.relax_strength, 0.0, 1.0, "pave sphere relax strength")?;
        Ok(())
    }
}

/// Stone layout on a sphere (`OM9-L3-04-12-005`). No surface is required.
pub fn pave_sphere(params: &SphereParams) -> Result<Vec<GemInstance>> {
    params.validate()?;
    let count = params
        .count
        .unwrap_or((params.x * (params.t + 1)) as usize)
        .min(MAX_PAVE_GEMS);
    let mut points = Vec::new();
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    for index in 0..count {
        if matches!(params.removal, RemovalPattern::PatternA) && index % 5 == 2 {
            continue;
        }
        if matches!(params.removal, RemovalPattern::PatternB) && index % 7 == 3 {
            continue;
        }
        let y = 1.0 - 2.0 * (index as f64 + 0.5) / count as f64;
        let radius = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden * index as f64;
        points.push([radius * theta.cos(), y, radius * theta.sin()]);
    }
    // Relax in 3D, then re-project onto the sphere so every stone stays on the surface.
    for _ in 0..params.relax_amount {
        for a in 0..points.len() {
            for b in (a + 1)..points.len() {
                let delta = sub(points[a], points[b]);
                let distance = norm(delta);
                let target = 2.0 * params.spec.length_mm / params.radius_mm;
                if distance > 0.0 && distance < target {
                    let push = (target - distance) * params.relax_strength * 0.5;
                    let direction = mul(delta, 1.0 / distance);
                    points[a] = add(points[a], mul(direction, push));
                    points[b] = sub(points[b], mul(direction, push));
                }
            }
        }
        for point in &mut points {
            *point = unit(*point, "pave sphere point")?;
        }
    }
    let mut gems = Vec::new();
    for point in points {
        let position = mul(point, params.radius_mm);
        let z = point;
        let helper = if z[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let x = cross(helper, z);
        let size = (params.spec.length_mm).max(params.min_size_mm);
        let spec = scaled(
            &params.spec,
            size / params.spec.length_mm,
            params.min_size_mm,
            params.spec.length_mm,
        )?;
        gems.push(GemInstance {
            spec,
            placement: frame(position, x, z)?,
            label: "Pave Sphere".to_owned(),
        });
    }
    Ok(gems)
}

/* --------------------------------- springs -------------------------------- */

/// Gem Springs relaxation parameters (`OM9-L3-04-12-006`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpringParams {
    /// Desired centre distance between neighbouring stones, in mm.
    pub spacing_mm: f64,
    /// Hard minimum centre distance, in mm.
    pub min_spacing_mm: f64,
    pub iterations: u32,
    pub strength: f64,
    /// Uniform resize applied before relaxation; `1` keeps the size.
    pub resize: f64,
}

impl SpringParams {
    pub fn new() -> Result<Self> {
        let params = Self {
            spacing_mm: 2.0,
            min_spacing_mm: 0.2,
            iterations: 8,
            strength: 0.5,
            resize: 1.0,
        };
        params.validate()?;
        Ok(params)
    }
    pub fn validate(&self) -> Result<()> {
        in_range(self.spacing_mm, 0.1, 100.0, "spring spacing")?;
        in_range(
            self.min_spacing_mm,
            0.0,
            self.spacing_mm,
            "spring minimum spacing",
        )?;
        if self.iterations > 128 {
            return Err(Error::Invalid("spring iterations"));
        }
        in_range(self.strength, 0.0, 1.0, "spring strength")?;
        in_range(self.resize, 0.2, 5.0, "spring resize")?;
        Ok(())
    }
}

/// Relax already placed stones on one surface (`OM9-L3-04-12-006`).
///
/// The solver moves each stone in the surface tangent plane, projects the moved point back through
/// `curve_surface_ops::closest_point_on_surface`, and keeps the stone's own table axis. Positions
/// and sizes are returned; the caller commits them in one transaction.
pub fn spring_relax(
    surface: &NurbsSurface,
    gems: &[GemInstance],
    params: &SpringParams,
    limits: &Limits,
) -> Result<Vec<GemInstance>> {
    surface.validate(limits)?;
    params.validate()?;
    if gems.is_empty() {
        return Err(Error::Missing("spring gems"));
    }
    if gems.len() > MAX_PAVE_GEMS {
        return Err(Error::Budget("spring gems"));
    }
    let mut positions: Vec<Point3> = gems.iter().map(gem_center).collect();
    for _ in 0..params.iterations {
        for a in 0..positions.len() {
            for b in (a + 1)..positions.len() {
                let delta = sub(positions[a], positions[b]);
                let distance = norm(delta);
                if distance > 0.0 && distance < params.spacing_mm {
                    let push = (params.spacing_mm - distance) * params.strength * 0.5;
                    let direction = mul(delta, 1.0 / distance);
                    positions[a] = add(positions[a], mul(direction, push));
                    positions[b] = sub(positions[b], mul(direction, push));
                }
            }
        }
    }
    let mut out = Vec::new();
    for (gem, position) in gems.iter().zip(positions) {
        let projection =
            crate::curve_surface_ops::closest_point_on_surface(surface, position, limits)?;
        let normal = crate::surface_kernel::normal(
            surface,
            projection.parameters[0],
            projection.parameters[1],
            limits,
        )?;
        let basis = gem.basis();
        let spec = scaled(
            &gem.spec,
            params.resize,
            params.min_spacing_mm.max(0.1),
            gem.spec.length_mm * 4.0,
        )?;
        out.push(GemInstance {
            spec,
            placement: frame(projection.point, basis[0], normal)?,
            label: gem.label.clone(),
        });
    }
    Ok(out)
}

/* --------------------------------- preview -------------------------------- */

/// What a pavé builder paints (`OM9-L3-04-12-007`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PavePreview {
    Stones,
    Framework,
}

/// Framework segments between nearest lattice neighbours, for the `Display` option.
pub fn pave_framework(layout: &PaveLayout) -> Vec<[Point3; 2]> {
    let mut edges = Vec::new();
    for a in 0..layout.stations.len() {
        let mut neighbours: Vec<(f64, usize)> = Vec::new();
        for b in 0..layout.stations.len() {
            if a == b {
                continue;
            }
            let distance = norm(sub(layout.stations[a].point, layout.stations[b].point));
            neighbours.push((distance, b));
        }
        neighbours.sort_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (_, b) in neighbours.into_iter().take(3) {
            if b > a {
                edges.push([layout.stations[a].point, layout.stations[b].point]);
            }
        }
    }
    edges
}

/* -------------------------------- history --------------------------------- */

/// The persisted pavé record (`OM9-L3-04-12-008`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaveFeature {
    pub version: u32,
    /// The host surface.
    pub surface: ObjectId,
    pub params: PaveParams,
    /// Ordered child gem ids, one per station.
    pub gems: Vec<GemId>,
    /// Manual Pavé exact count; `None` means the region fill.
    #[serde(default)]
    pub quantity: Option<usize>,
    /// Source curve that supplied the attractor, when it came from the document.
    #[serde(default)]
    pub attractor_source: Option<ObjectId>,
    /// Source curves that supplied the closed region edges.
    #[serde(default)]
    pub edge_sources: Vec<ObjectId>,
    /// Surface parameters of each child, in the same order.
    pub uvs: Vec<[f64; 2]>,
}

impl PaveFeature {
    pub fn new(surface: ObjectId, params: PaveParams) -> Self {
        Self {
            version: PAVE_SCHEMA_VERSION,
            surface,
            params,
            gems: Vec::new(),
            quantity: None,
            attractor_source: None,
            edge_sources: Vec::new(),
            uvs: Vec::new(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != PAVE_SCHEMA_VERSION {
            return Err(Error::Invalid("pave feature version"));
        }
        if self.surface.is_nil() {
            return Err(Error::Invalid("pave feature surface"));
        }
        self.params.validate()?;
        if self.gems.len() > MAX_PAVE_GEMS || self.uvs.len() > MAX_PAVE_GEMS {
            return Err(Error::Budget("pave feature gems"));
        }
        if !self.gems.is_empty() && self.gems.len() != self.uvs.len() {
            return Err(Error::Invalid("pave feature uv count"));
        }
        let mut seen = std::collections::BTreeSet::new();
        if self
            .gems
            .iter()
            .any(|gem| gem.is_nil() || !seen.insert(*gem))
        {
            return Err(Error::Invalid("pave feature gem id"));
        }
        if self.uvs.iter().flatten().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("pave feature uv"));
        }
        if self.edge_sources.len() > 32 {
            return Err(Error::Budget("pave feature edge sources"));
        }
        if self.attractor_source.is_some_and(|id| id.is_nil())
            || self.edge_sources.iter().any(|id| id.is_nil())
        {
            return Err(Error::Invalid("pave feature source id"));
        }
        Ok(())
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| Error::Invalid("pave feature json"))
    }
    pub fn from_json(text: &str) -> Result<Self> {
        let feature: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("pave feature json"))?;
        feature.validate()?;
        Ok(feature)
    }
    pub fn to_extension(&self) -> Result<Extension> {
        Ok(Extension {
            version: PAVE_SCHEMA_VERSION,
            payload: self.to_json()?,
        })
    }
    pub fn from_extension(extension: &Extension) -> Result<Self> {
        if extension.version != PAVE_SCHEMA_VERSION {
            return Err(Error::Invalid("pave feature extension version"));
        }
        Self::from_json(&extension.payload)
    }
}

/// Result of recomputing a feature against an edited surface.
#[derive(Debug, Clone, PartialEq)]
pub struct PaveRefresh {
    pub stations: Vec<PaveStation>,
    /// Stones whose UV no longer exists; their ids are kept instead of being dropped.
    pub conflicts: Vec<GemId>,
}

/// Re-evaluate every persisted station after the host surface changed
/// (`OM9-L3-04-12-008`). Existing ids are preserved by position; a station outside the new domain
/// becomes an explicit conflict.
pub fn refresh_pave_layout(
    surface: &NurbsSurface,
    feature: &PaveFeature,
    limits: &Limits,
) -> Result<PaveRefresh> {
    surface.validate(limits)?;
    feature.validate()?;
    if feature.gems.is_empty() {
        return Err(Error::Missing("pave feature stations"));
    }
    if feature.gems.len() != feature.uvs.len() {
        return Err(Error::Invalid("pave feature uv count"));
    }
    let domain = surface.domain;
    let mut stations = Vec::new();
    let mut conflicts = Vec::new();
    for (index, uv) in feature.uvs.iter().enumerate() {
        if uv[0] < domain[0][0]
            || uv[0] > domain[0][1]
            || uv[1] < domain[1][0]
            || uv[1] > domain[1][1]
        {
            conflicts.push(feature.gems[index]);
            continue;
        }
        let derivatives = crate::surface_kernel::derivatives(surface, uv[0], uv[1], limits)?;
        let normal = crate::surface_kernel::normal(surface, uv[0], uv[1], limits)?;
        let tangent = unit(derivatives.du, "pave refresh tangent")?;
        let spec = feature.params.spec.clone();
        stations.push(PaveStation {
            index,
            uv: *uv,
            point: derivatives.point,
            normal,
            tangent,
            spec,
            transform: frame(derivatives.point, tangent, normal)?,
            gem: Some(feature.gems[index]),
        });
    }
    Ok(PaveRefresh {
        stations,
        conflicts,
    })
}

/// A right-handed pair of axes in the plane normal to `normal`; used by the host for marker/azure
/// circle geometry so the frame convention is not re-implemented in the UI.
pub fn marker_axes(normal: Point3) -> Result<(Point3, Point3)> {
    let z = unit(normal, "pave marker normal")?;
    let helper = if z[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let x = unit(cross(helper, z), "pave marker x")?;
    let y = cross(z, x);
    Ok((x, y))
}

/// A closed polygon curve from a planned profile; the host commits one curve per profile.
pub fn closed_polygon(points: &[Point3]) -> Result<Curve> {
    if points.len() < 3 {
        return Err(Error::Invalid("azure profile points"));
    }
    let mut segments = Vec::with_capacity(points.len());
    for index in 0..points.len() {
        let start = points[index];
        let end = points[(index + 1) % points.len()];
        if norm(sub(start, end)) <= 1.0e-12 {
            return Err(Error::Invalid("azure profile segment"));
        }
        segments.push(Curve::Line {
            start,
            end,
            domain: [0.0, 1.0],
        });
    }
    Ok(Curve::Polycurve {
        segments,
        reversed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane(size: f64) -> NurbsSurface {
        crate::surface_kernel::plane(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            size,
            size,
            &Limits::default(),
        )
        .unwrap()
    }

    fn gem(size: f64) -> GemSpec {
        GemSpec::round(size, size * 0.65).unwrap()
    }

    #[test]
    fn pave_hex_fills_the_region_with_legal_spacing() {
        let surface = plane(20.0);
        let mut params = PaveParams::new(gem(1.5)).unwrap();
        params.spacing_mm = 2.0;
        params.min_spacing_mm = 0.1;
        params.margin_mm = 0.2;
        params.relax_iterations = 0;
        let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
        assert!(layout.stations.len() > 20);
        assert!(layout.stations.iter().all(|station| {
            station.uv[0] >= surface.domain[0][0] + 0.01
                && station.uv[0] <= surface.domain[0][1] - 0.01
                && station.point[2].abs() < 1.0e-9
        }));
        assert!(
            layout
                .stations
                .iter()
                .all(|station| station.spec.length_mm >= params.min_size_mm)
        );
    }

    #[test]
    fn patterns_differ_and_square_has_axis_rows() {
        let surface = plane(12.0);
        let mut params = PaveParams::new(gem(1.2)).unwrap();
        params.margin_mm = 0.0;
        params.relax_iterations = 0;
        params.spacing_mm = 2.0;
        let hex = pave_surface(&surface, &params, &Limits::default()).unwrap();
        params.pattern = PavePattern::Square;
        let square = pave_surface(&surface, &params, &Limits::default()).unwrap();
        assert_ne!(hex.stations.len(), square.stations.len());
        let rows: std::collections::BTreeSet<i64> = square
            .stations
            .iter()
            .map(|station| (station.point[1] * 1000.0).round() as i64)
            .collect();
        assert!(rows.len() > 2);
    }

    #[test]
    fn bad_pave_parameters_are_refused() {
        let surface = plane(10.0);
        let mut params = PaveParams::new(gem(1.0)).unwrap();
        params.min_spacing_mm = 5.0;
        assert!(pave_surface(&surface, &params, &Limits::default()).is_err());
        let mut params = PaveParams::new(gem(1.0)).unwrap();
        params.start_uv = [2.0, 2.0];
        assert!(pave_surface(&surface, &params, &Limits::default()).is_err());
        let mut params = PaveParams::new(gem(1.0)).unwrap();
        params.seeds = vec![[0.0, 0.0]; MAX_PAVE_GEMS + 1];
        assert!(matches!(params.validate(), Err(Error::Budget(_))));
    }

    #[test]
    fn azure_profiles_are_closed_and_ordered() {
        let gems = vec![
            GemInstance::at(gem(2.0), [0.0, 0.0, 0.0]).unwrap(),
            GemInstance::at(gem(2.0), [3.0, 0.0, 0.0]).unwrap(),
        ];
        let params = AzureParams::new().unwrap();
        let plan = pave_azures(&gems, &params).unwrap();
        assert_eq!(plan.azures.len(), 2);
        for azure in &plan.azures {
            assert_eq!(azure.top.len(), 48);
            assert_eq!(azure.top.len(), azure.bottom.len());
            assert!(norm(sub(azure.top[0], azure.top[47])) > 0.0);
        }
        let mut bad = AzureParams::new().unwrap();
        bad.through_hole_mm = 0.0;
        assert!(pave_azures(&gems, &bad).is_err());
    }

    #[test]
    fn shared_and_center_prongs_follow_the_contact_graph() {
        let gems = vec![
            GemInstance::at(gem(2.0), [0.0, 0.0, 0.0]).unwrap(),
            GemInstance::at(gem(2.0), [1.9, 0.0, 0.0]).unwrap(),
        ];
        let plan = pave_prongs(&gems, &ProngParams::new().unwrap()).unwrap();
        assert!(
            plan.prongs
                .iter()
                .any(|prong| prong.kind == ProngKind::Shared)
        );
        assert!(
            plan.prongs
                .iter()
                .any(|prong| prong.kind == ProngKind::Unshared)
        );
        let mut params = ProngParams::new().unwrap();
        params.shared_size_mm = 0.0;
        assert!(pave_prongs(&gems, &params).is_err());
    }

    #[test]
    fn sphere_layout_stays_on_the_sphere_and_removal_reduces_the_count() {
        let mut params = SphereParams::new(gem(1.0)).unwrap();
        params.count = Some(80);
        params.relax_amount = 2;
        let gems = pave_sphere(&params).unwrap();
        assert_eq!(gems.len(), 80);
        assert!(gems.iter().all(|gem| {
            let p = gem_center(gem);
            (norm(p) - params.radius_mm).abs() < 1.0e-6
        }));
        params.removal = RemovalPattern::PatternA;
        assert!(pave_sphere(&params).unwrap().len() < 80);
        params.radius_mm = 0.0;
        assert!(pave_sphere(&params).is_err());
    }

    #[test]
    fn springs_move_overlapping_stones_apart_on_the_surface() {
        let surface = plane(10.0);
        let gems = vec![
            GemInstance::at(gem(1.0), [1.0, 1.0, 0.0]).unwrap(),
            GemInstance::at(gem(1.0), [1.2, 1.0, 0.0]).unwrap(),
        ];
        let params = SpringParams::new().unwrap();
        let relaxed = spring_relax(&surface, &gems, &params, &Limits::default()).unwrap();
        let distance = norm(sub(gem_center(&relaxed[0]), gem_center(&relaxed[1])));
        assert!(distance > 1.5);
        assert!(relaxed.iter().all(|gem| gem_center(gem)[2].abs() < 1.0e-9));
    }

    #[test]
    fn feature_round_trips_and_refresh_keeps_ids() {
        let surface = plane(10.0);
        let mut params = PaveParams::new(gem(1.0)).unwrap();
        params.margin_mm = 0.0;
        let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
        let mut feature = PaveFeature::new(ObjectId::new(), params);
        feature.gems = layout.stations.iter().map(|_| GemId::new()).collect();
        feature.uvs = layout.stations.iter().map(|station| station.uv).collect();
        let read = PaveFeature::from_extension(&feature.to_extension().unwrap()).unwrap();
        assert_eq!(read, feature);
        let refresh = refresh_pave_layout(&surface, &feature, &Limits::default()).unwrap();
        assert!(refresh.conflicts.is_empty());
        assert_eq!(refresh.stations.len(), feature.gems.len());
        assert_eq!(refresh.stations[0].gem, Some(feature.gems[0]));
        let mut broken = feature.clone();
        broken.gems.push(broken.gems[0]);
        assert!(broken.to_json().is_err());
    }

    #[test]
    fn framework_and_preview_are_bounded() {
        let surface = plane(8.0);
        let mut params = PaveParams::new(gem(1.0)).unwrap();
        params.margin_mm = 0.0;
        let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
        let edges = pave_framework(&layout);
        assert!(!edges.is_empty());
        assert!(edges.len() <= layout.stations.len() * 3);
        assert_eq!(PavePreview::Stones, PavePreview::Stones);
    }

    #[test]
    fn closed_polygon_is_closed_and_rejects_a_degenerate_profile() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let curve = closed_polygon(&points).unwrap();
        assert!(matches!(curve, Curve::Polycurve { .. }));
        assert!(closed_polygon(&points[..2]).is_err());
    }
}

/* --------------------------------- styles --------------------------------- */

/// A named pavé parameter snapshot (`Save Style` in Auto/Manual Pavé).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaveStyle {
    pub name: String,
    pub params: PaveParams,
    /// Manual Pavé exact count, when the style was saved from a Manual builder.
    #[serde(default)]
    pub quantity: Option<usize>,
}

impl PaveStyle {
    pub fn new(
        name: impl Into<String>,
        params: PaveParams,
        quantity: Option<usize>,
    ) -> Result<Self> {
        let style = Self {
            name: name.into().trim().to_owned(),
            params,
            quantity,
        };
        style.validate()?;
        Ok(style)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() || self.name.len() > MAX_PAVE_STYLE_NAME_BYTES {
            return Err(Error::Invalid("pave style name"));
        }
        self.params.validate()?;
        if let Some(quantity) = self.quantity
            && !(1..=MAX_PAVE_GEMS).contains(&quantity)
        {
            return Err(Error::Invalid("pave style quantity"));
        }
        Ok(())
    }
}

/// One style file. Built-ins are code, not file entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaveStyleFile {
    pub schema: String,
    #[serde(default)]
    pub styles: Vec<PaveStyle>,
}

impl PaveStyleFile {
    pub fn new(styles: Vec<PaveStyle>) -> Result<Self> {
        let file = Self {
            schema: PAVE_STYLE_SCHEMA.to_owned(),
            styles,
        };
        file.validate()?;
        Ok(file)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema != PAVE_STYLE_SCHEMA {
            return Err(Error::Invalid("pave style schema"));
        }
        if self.styles.len() > MAX_PAVE_STYLES {
            return Err(Error::Budget("pave styles"));
        }
        let mut names = std::collections::BTreeSet::new();
        for style in &self.styles {
            style.validate()?;
            if !names.insert(style.name.to_ascii_lowercase()) {
                return Err(Error::Invalid("pave style duplicate name"));
            }
        }
        Ok(())
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let file: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("pave style json"))?;
        file.validate()?;
        Ok(file)
    }

    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| Error::Invalid("pave style json"))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let text = self.to_json()?;
        std::fs::write(path, text).map_err(|_| Error::Invalid("pave style save"))
    }

    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|_| Error::Invalid("pave style load"))?;
        Self::from_json(&text)
    }
}

/// Built-in and user pavé styles. The built-in list is never written to disk.
#[derive(Debug, Clone, PartialEq)]
pub struct PaveStyleLibrary {
    pub builtin: Vec<PaveStyle>,
    pub user: Vec<PaveStyle>,
}

impl Default for PaveStyleLibrary {
    fn default() -> Self {
        Self::builtin()
    }
}

impl PaveStyleLibrary {
    pub fn builtin() -> Self {
        let mut styles = Vec::new();
        if let Some(style) = preset_style("Classic Hex", |params| {
            params.pattern = PavePattern::Hex;
            params.spacing_mm = 1.8;
            params.border_mm = 0.4;
        }) {
            styles.push(style);
        }
        if let Some(style) = preset_style("Square 45", |params| {
            params.pattern = PavePattern::Square;
            params.spacing_mm = 1.6;
            params.angle_deg = 45.0;
        }) {
            styles.push(style);
        }
        if let Some(style) = preset_style("Tapered Border", |params| {
            params.pattern = PavePattern::HexReverse;
            params.spacing_mm = 1.7;
            params.taper = 0.5;
            params.border_mm = 0.8;
        }) {
            styles.push(style);
        }
        Self {
            builtin: styles,
            user: Vec::new(),
        }
    }

    pub fn names(&self) -> Vec<String> {
        self.builtin
            .iter()
            .chain(&self.user)
            .map(|style| style.name.clone())
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&PaveStyle> {
        self.builtin
            .iter()
            .chain(&self.user)
            .find(|style| style.name.eq_ignore_ascii_case(name.trim()))
    }

    pub fn add_user(&mut self, style: PaveStyle) -> Result<()> {
        style.validate()?;
        if self.get(&style.name).is_some() {
            return Err(Error::Invalid("pave style name is already used"));
        }
        if self.user.len() >= MAX_PAVE_STYLES {
            return Err(Error::Budget("pave styles"));
        }
        self.user.push(style);
        Ok(())
    }

    pub fn remove_user(&mut self, name: &str) -> bool {
        let before = self.user.len();
        self.user
            .retain(|style| !style.name.eq_ignore_ascii_case(name.trim()));
        self.user.len() != before
    }

    /// Persist only the user styles; built-ins stay in code.
    pub fn save_user(&self, path: &std::path::Path) -> Result<()> {
        PaveStyleFile::new(self.user.clone())?.save(path)
    }

    /// Load a user file over the built-ins. A corrupt file is refused.
    pub fn load_user(path: &std::path::Path) -> Result<Self> {
        let mut library = Self::builtin();
        let file = PaveStyleFile::load(path)?;
        for style in file.styles {
            library.add_user(style)?;
        }
        Ok(library)
    }
}

fn preset_style(name: &str, mutate: impl FnOnce(&mut PaveParams)) -> Option<PaveStyle> {
    let spec = GemSpec::round(1.5, 1.0).ok()?;
    let mut params = PaveParams::new(spec).ok()?;
    mutate(&mut params);
    PaveStyle::new(name, params, None).ok()
}
