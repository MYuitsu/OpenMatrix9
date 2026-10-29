// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gems on curves and surfaces (`OM9-L2-04-10`).
//!
//! The module owns the placement family of the M05 gem tools: evenly spaced stones on a
//! selected curve, variable/tapered and heterogeneous lists, several curves at once, the
//! gem profile and guide curves, placement on a surface, follow/flow/splop of an existing
//! placement and the associative link that keeps a gem bound to its source curve or
//! surface.
//!
//! Ownership split:
//!
//! * [`place_on_curve`] / [`place_on_curve_variable`] — ordered stations derived from the
//!   **arc length** of the curve (`OM9-L3-04-10-001`, `-002`, `-004`).
//! * [`count_on_curve`] — the feasible count and pitch for a stone/gap/end allowance
//!   (`OM9-L3-04-10-003`).
//! * [`place_on_curves`] — one atomic result for several paths (`OM9-L3-04-10-005`).
//! * [`profile_and_guides`] — non-destructive profile/girdle/guide curves
//!   (`OM9-L3-04-10-006`).
//! * [`place_on_surface`] — UV frames on a surface (`OM9-L3-04-10-007`).
//! * [`follow_placement`] — recompute the stations after the source curve changed
//!   (`OM9-L3-04-10-008`).
//! * [`flow_placement`] / [`flow_surface_placement`] — carry the rigid gem frames from a
//!   base path/surface to a target path/surface (`OM9-L3-04-10-009`).
//! * [`splop_station`] / [`splop_placement`] — rigid projection onto a surface with the
//!   surface normal as the gem axis (`OM9-L3-04-10-010`).
//! * [`PlacementLink`] — the persisted `om9.gem.placement` record with its version and
//!   `ACTIVE`/`BROKEN`/`CONFLICT` state (`OM9-L3-04-10-011`).
//!
//! Rules shared by every entry point:
//!
//! * units are document millimetres and the curve/surface evaluators are the shared ones in
//!   [`crate::curve_ops`] / [`crate::surface_kernel`], so this module never re-invents a
//!   kernel;
//! * a station carries an orthonormal right-handed frame (gem local +Z is the table axis);
//! * a placement plan is pure data: nothing here mutates a document, creates an object or
//!   inserts a link. The caller turns a plan into one transaction;
//! * every refusal is a typed [`Error`] — no clamp, no silent drop of a station.

use crate::{
    Error, Result,
    budget::Limits,
    curve_ops,
    curve_surface_ops,
    deform,
    gem::{GemId, GemSpec},
    geometry::{Curve, NurbsSurface, Point3, Transform},
    identity::ObjectId,
    model::Extension,
};
use serde::{Deserialize, Serialize};

/// Version of the persisted placement link record.
pub const PLACEMENT_SCHEMA: &str = "om9.gem_placement.v1";
/// Extension slot written on a gem that was placed on a curve or surface.
pub const PLACEMENT_EXTENSION_SLOT: &str = crate::model::GEM_PLACEMENT_EXTENSION_SLOT;
/// Hard cap on stations per placement, so a typo cannot exhaust memory.
pub const MAX_STATIONS: usize = 100_000;

/// One station on a path: arc length, global fraction, point, unit tangent and the gem frame
/// basis (`x`,`y`,`z`, with `z` the table axis).
#[derive(Debug, Clone, PartialEq)]
pub struct GemStation {
    /// Zero-based order along the path.
    pub index: usize,
    /// Global curve parameter of the station.
    pub fraction: f64,
    /// Arc length from the curve start, in millimetres.
    pub arc_length_mm: f64,
    /// Girdle centre in world millimetres (already offset along the axis).
    pub point: Point3,
    /// Unit tangent of the path at the station.
    pub tangent: Point3,
    /// Unit gem table axis (gem local +Z).
    pub axis: Point3,
    /// Surface parameters when the station came from a surface.
    pub uv: Option<[f64; 2]>,
    /// Rigid local-to-world frame of the gem.
    pub transform: Transform,
    /// Stone placed at this station.
    pub spec: GemSpec,
    /// Offset along the axis that was applied, so a follow/flow can reapply it.
    pub z_offset_mm: f64,
    /// Spin about the axis that was applied, in degrees.
    pub spin_deg: f64,
    /// Identity of the placed gem when the station came from the document.
    pub gem: Option<GemId>,
}

impl GemStation {
    /// The gem's table centre in world millimetres.
    pub fn table_point(&self) -> Point3 {
        add(self.point, mul(self.axis, self.spec.depth_mm()))
    }
    /// `true` when the frame basis is orthonormal and right-handed.
    pub fn frame_is_orthonormal(&self) -> bool {
        let m = self.transform.0;
        let columns = [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ];
        let lengths_ok = columns
            .iter()
            .all(|column| (dot(*column, *column) - 1.0).abs() <= 1.0e-9);
        let orthogonal = dot(columns[0], columns[1]).abs() <= 1.0e-9
            && dot(columns[0], columns[2]).abs() <= 1.0e-9
            && dot(columns[1], columns[2]).abs() <= 1.0e-9;
        let handed = dot(cross(columns[0], columns[1]), columns[2]) > 0.0;
        lengths_ok && orthogonal && handed
    }
}

/// Orientation of the gem table around the path.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// Table along the world +Z (or an explicit hint) projected perpendicular to the tangent.
    #[default]
    Upright,
    /// Table along an explicit axis, projected perpendicular to the tangent.
    Axis([f64; 3]),
    /// Table along the target surface normal (surfaces only; refused on a curve).
    SurfaceNormal,
}

impl Orientation {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Upright => "upright",
            Self::Axis(_) => "axis",
            Self::SurfaceNormal => "surfacenormal",
        }
    }
}

/// The spacing rule of one `Gem on Curve` run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveParams {
    /// Arc length of the first allowed station, in millimetres.
    pub start_mm: f64,
    /// Last allowed arc length; `None` runs to the curve end.
    pub end_mm: Option<f64>,
    /// Centre-to-centre spacing along the arc, in millimetres.
    pub spacing_mm: f64,
    /// Offset along the gem axis (Matrix `Z Offset`).
    pub z_offset_mm: f64,
    /// Spin about the gem axis, in degrees.
    pub rotation_deg: f64,
    /// Which way the table faces.
    pub orientation: Orientation,
    /// Whether a stone longer than the spacing is admitted.
    pub allow_overlap: bool,
    /// Drop the station that would coincide with a closed curve's seam.
    pub skip_seam: bool,
}

impl CurveParams {
    /// A run from the curve start to its end at the given spacing.
    pub fn spacing(spacing_mm: f64) -> Self {
        Self {
            start_mm: 0.0,
            end_mm: None,
            spacing_mm,
            z_offset_mm: 0.0,
            rotation_deg: 0.0,
            orientation: Orientation::Upright,
            allow_overlap: false,
            skip_seam: true,
        }
    }
    pub fn with_range(mut self, start_mm: f64, end_mm: Option<f64>) -> Self {
        self.start_mm = start_mm;
        self.end_mm = end_mm;
        self
    }
    pub fn with_z_offset(mut self, z_offset_mm: f64) -> Self {
        self.z_offset_mm = z_offset_mm;
        self
    }
    pub fn with_rotation(mut self, rotation_deg: f64) -> Self {
        self.rotation_deg = rotation_deg;
        self
    }
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn with_overlap(mut self, allow_overlap: bool) -> Self {
        self.allow_overlap = allow_overlap;
        self
    }
    /// Reject a parameter set that cannot describe a run. The arc range against the real
    /// curve length is checked by the placement entry points.
    pub fn validate(&self) -> Result<()> {
        if !self.start_mm.is_finite() || self.start_mm < 0.0 {
            return Err(Error::Invalid("gem placement start"));
        }
        if let Some(end) = self.end_mm
            && (!end.is_finite() || end < self.start_mm)
        {
            return Err(Error::Invalid("gem placement end"));
        }
        if !self.spacing_mm.is_finite() || self.spacing_mm <= 0.0 {
            return Err(Error::Invalid("gem placement spacing"));
        }
        if !self.z_offset_mm.is_finite() {
            return Err(Error::Invalid("gem placement z offset"));
        }
        if !self.rotation_deg.is_finite() {
            return Err(Error::Invalid("gem placement rotation"));
        }
        if let Orientation::Axis(axis) = self.orientation
            && (!axis.iter().all(|value| value.is_finite()) || norm(axis) <= 1.0e-12)
        {
            return Err(Error::Invalid("gem placement axis"));
        }
        Ok(())
    }
}

/// One evaluated path station before a stone is attached.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathSample {
    pub fraction: f64,
    pub arc_length_mm: f64,
    pub point: Point3,
    pub tangent: Point3,
}

/// The result of one `Gem on Curve` run.
#[derive(Debug, Clone, PartialEq)]
pub struct CurvePlacement {
    /// Source curve when the plan is bound to a document object.
    pub curve: Option<ObjectId>,
    /// The parameters that produced the stations, kept so a follow can repeat them.
    pub params: CurveParams,
    /// Whole curve length in millimetres.
    pub total_length_mm: f64,
    /// Effective start and end arc length of the run.
    pub range: [f64; 2],
    /// Whether the source curve is closed.
    pub closed: bool,
    pub stations: Vec<GemStation>,
    /// Arc distance between consecutive stations; the seam gap is last when complete.
    pub gaps_mm: Vec<f64>,
}

impl CurvePlacement {
    pub fn count(&self) -> usize {
        self.stations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.stations.is_empty()
    }
    /// Identities of the placed gems, in station order.
    pub fn gem_ids(&self) -> Vec<GemId> {
        self.stations.iter().filter_map(|station| station.gem).collect()
    }
    /// The tightest arc gap between neighbours, including the closed-curve seam.
    pub fn min_gap_mm(&self) -> Option<f64> {
        self.gaps_mm
            .iter()
            .copied()
            .filter(|gap| gap.is_finite())
            .reduce(f64::min)
    }
}

/// Several paths placed in one atomic run (`OM9-L3-04-10-005`).
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPlacement {
    pub groups: Vec<CurvePlacement>,
}

impl MultiPlacement {
    pub fn stations(&self) -> usize {
        self.groups.iter().map(CurvePlacement::count).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.groups.iter().all(CurvePlacement::is_empty)
    }
}

/// The feasible count for a stone on a curve (`OM9-L3-04-10-003`).
#[derive(Debug, Clone, PartialEq)]
pub struct CurveFit {
    /// Number of stones that fit with the requested gap and end allowance.
    pub count: usize,
    /// `false` when not even one stone fits.
    pub fits: bool,
    /// Arc length available between the two end allowances.
    pub usable_mm: f64,
    /// Centre-to-centre pitch (`stone + gap`).
    pub spacing_mm: f64,
    /// Arc length of the first and last stone centre.
    pub first_center_mm: f64,
    pub last_center_mm: f64,
}

/// Profile, girdle and guide curves built from a placement (`OM9-L3-04-10-006`).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacementGuides {
    /// Curve through the gem tables (the profile of the row).
    pub profile: Curve,
    /// Curve through the girdle centres.
    pub girdle: Curve,
    /// One guide per station: girdle centre to table centre.
    pub guides: Vec<Curve>,
    pub table_points: Vec<Point3>,
}

/// The UV rule of one `Gem on Surface` run.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceParams {
    /// Surface parameters of the stations, in order.
    pub uv: Vec<[f64; 2]>,
    /// Offset along the surface normal.
    pub z_offset_mm: f64,
    /// Spin about the surface normal, in degrees.
    pub rotation_deg: f64,
}

impl SurfaceParams {
    pub fn new(uv: Vec<[f64; 2]>) -> Self {
        Self {
            uv,
            z_offset_mm: 0.0,
            rotation_deg: 0.0,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.uv.is_empty() {
            return Err(Error::Missing("gem surface parameters"));
        }
        if self.uv.len() > MAX_STATIONS {
            return Err(Error::Budget("gem surface stations"));
        }
        for uv in &self.uv {
            if !uv.iter().all(|value| value.is_finite()) {
                return Err(Error::Invalid("gem surface parameter"));
            }
        }
        if !self.z_offset_mm.is_finite() {
            return Err(Error::Invalid("gem surface z offset"));
        }
        if !self.rotation_deg.is_finite() {
            return Err(Error::Invalid("gem surface rotation"));
        }
        Ok(())
    }
}

/// The result of one `Gem on Surface` run.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePlacement {
    pub surface: Option<ObjectId>,
    pub params: SurfaceParams,
    pub stations: Vec<GemStation>,
}

impl SurfacePlacement {
    pub fn count(&self) -> usize {
        self.stations.len()
    }
}

/// Why one gem of a follow could not be recomputed.
#[derive(Debug, Clone, PartialEq)]
pub struct FollowConflict {
    /// Station index in the previous placement.
    pub station: usize,
    /// Gem identity, kept so the caller can show which stone is affected.
    pub gem: Option<GemId>,
    /// Stable machine-readable reason.
    pub reason: &'static str,
}

/// The result of a follow (`OM9-L3-04-10-008`).
#[derive(Debug, Clone, PartialEq)]
pub struct FollowOutcome {
    /// Recomputed placement; `gem` ids are carried over for every surviving station.
    pub placement: CurvePlacement,
    /// Stones that no longer have a valid station; never silently dropped.
    pub conflicts: Vec<FollowConflict>,
    /// Movement of each surviving station, in millimetres.
    pub movement_mm: Vec<f64>,
}

/// How a swatch of placed gems is projected back onto a surface (`OM9-L3-04-10-010`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplopParams {
    /// Offset along the surface normal, in millimetres.
    pub z_offset_mm: f64,
    /// Spin about the surface normal, in degrees.
    pub rotation_deg: f64,
    /// Hint for the gem's local X axis (the reference orientation of the sphere splop).
    pub x_hint: Point3,
}

impl Default for SplopParams {
    fn default() -> Self {
        Self {
            z_offset_mm: 0.0,
            rotation_deg: 0.0,
            x_hint: [1.0, 0.0, 0.0],
        }
    }
}

impl SplopParams {
    pub fn validate(&self) -> Result<()> {
        if !self.z_offset_mm.is_finite() || !self.rotation_deg.is_finite() {
            return Err(Error::Invalid("gem splop parameter"));
        }
        if !self.x_hint.iter().all(|value| value.is_finite()) || norm(self.x_hint) <= 1.0e-12 {
            return Err(Error::Invalid("gem splop reference"));
        }
        Ok(())
    }
}

/// The curve or surface a placed gem is bound to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PlacementSource {
    Curve { id: ObjectId },
    Surface { id: ObjectId },
}

impl PlacementSource {
    pub fn object(self) -> ObjectId {
        match self {
            Self::Curve { id } | Self::Surface { id } => id,
        }
    }
    pub fn tag(self) -> &'static str {
        match self {
            Self::Curve { .. } => "curve",
            Self::Surface { .. } => "surface",
        }
    }
}

/// Lifecycle of a placement link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkState {
    /// The gem is bound to the source and its station is current.
    Active,
    /// The source was deleted or the link was broken on purpose; the gem keeps its place.
    Broken,
    /// The source changed so the station no longer resolves; the gem keeps its last place
    /// and the conflict is visible.
    Conflict,
}

impl LinkState {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Broken => "BROKEN",
            Self::Conflict => "CONFLICT",
        }
    }
}

/// The persisted `om9.gem.placement` record (`OM9-L3-04-10-011`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementLink {
    /// Record schema, checked on read.
    pub schema: String,
    /// Record version, bumped only by a breaking change.
    pub version: u32,
    /// The gem this link belongs to.
    pub gem: GemId,
    /// The source curve or surface.
    pub source: PlacementSource,
    /// Station arc length on the source, in millimetres.
    pub arc_length_mm: f64,
    /// Station fraction on the source, `0..=1`.
    pub station: f64,
    /// Surface parameters for a surface source.
    pub uv: Option<[f64; 2]>,
    /// Source revision the station was measured at.
    pub source_revision: u64,
    pub state: LinkState,
    /// Spacing used by the run, so a follow can repeat the rule.
    pub spacing_mm: f64,
    pub z_offset_mm: f64,
}

impl PlacementLink {
    pub fn new(
        gem: GemId,
        source: PlacementSource,
        arc_length_mm: f64,
        station: f64,
        source_revision: u64,
        spacing_mm: f64,
        z_offset_mm: f64,
    ) -> Result<Self> {
        let link = Self {
            schema: PLACEMENT_SCHEMA.to_owned(),
            version: 1,
            gem,
            source,
            arc_length_mm,
            station,
            uv: None,
            source_revision,
            state: LinkState::Active,
            spacing_mm,
            z_offset_mm,
        };
        link.validate()?;
        Ok(link)
    }
    pub fn with_uv(mut self, uv: [f64; 2]) -> Result<Self> {
        if !uv.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("gem placement uv"));
        }
        self.uv = Some(uv);
        self.validate()?;
        Ok(self)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema != PLACEMENT_SCHEMA {
            return Err(Error::Invalid("gem placement schema"));
        }
        if self.version == 0 || self.version > 1 {
            return Err(Error::Invalid("gem placement version"));
        }
        if self.gem.is_nil() {
            return Err(Error::Invalid("gem placement gem id"));
        }
        if self.source.object().is_nil() {
            return Err(Error::Invalid("gem placement source"));
        }
        if !self.arc_length_mm.is_finite() || self.arc_length_mm < 0.0 {
            return Err(Error::Invalid("gem placement arc length"));
        }
        if !self.station.is_finite() || !(0.0..=1.0).contains(&self.station) {
            return Err(Error::Invalid("gem placement station"));
        }
        if let Some(uv) = self.uv
            && !uv.iter().all(|value| value.is_finite())
        {
            return Err(Error::Invalid("gem placement uv"));
        }
        if !self.spacing_mm.is_finite() || self.spacing_mm <= 0.0 {
            return Err(Error::Invalid("gem placement spacing"));
        }
        if !self.z_offset_mm.is_finite() {
            return Err(Error::Invalid("gem placement z offset"));
        }
        Ok(())
    }
    /// `true` while the source still owns the station.
    pub fn is_live(&self) -> bool {
        self.state == LinkState::Active
    }
    /// Record that the source is gone; the gem keeps its last transform.
    pub fn mark_broken(&mut self) -> Result<()> {
        self.state = LinkState::Broken;
        self.validate()
    }
    /// Record an unresolved source change; the gem keeps its last transform.
    pub fn mark_conflict(&mut self) -> Result<()> {
        self.state = LinkState::Conflict;
        self.validate()
    }
    /// Revive a link after its station was recomputed. A revision older than the recorded
    /// one is a stale write, not a refresh.
    pub fn refresh(&mut self, revision: u64, arc_length_mm: f64, station: f64) -> Result<()> {
        if revision < self.source_revision {
            return Err(Error::Stale("gem placement revision"));
        }
        self.source_revision = revision;
        self.arc_length_mm = arc_length_mm;
        self.station = station;
        self.state = LinkState::Active;
        self.validate()
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| Error::Invalid("gem placement link"))
    }
    pub fn from_json(text: &str) -> Result<Self> {
        let link: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("gem placement link"))?;
        link.validate()?;
        Ok(link)
    }
    /// The `om9.gem.placement` extension for an entity.
    pub fn to_extension(&self) -> Result<Extension> {
        self.validate()?;
        Extension::new(1, self.to_json()?)
    }
    pub fn from_extension(extension: &Extension) -> Result<Self> {
        Self::from_json(&extension.payload)
    }
}

/* --------------------------------- geometry -------------------------------- */

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

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn mul(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn unit(value: Point3, what: &'static str) -> Result<Point3> {
    let length = norm(value);
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid(what));
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

/// Remove the component of `value` along the unit `axis`.
fn perpendicular(value: Point3, axis: Point3) -> Point3 {
    sub(value, mul(axis, dot(value, axis)))
}

/// The first world axis that is not parallel to `direction`, used as a stable fallback.
fn fallback_perpendicular(direction: Point3) -> Result<Point3> {
    for candidate in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let projected = perpendicular(candidate, direction);
        if norm(projected) > 1.0e-9 {
            return unit(projected, "gem frame axis");
        }
    }
    Err(Error::Invalid("gem frame axis"))
}

/// The table axis of a station on a path.
fn curve_axis(tangent: Point3, orientation: Orientation) -> Result<Point3> {
    let hint = match orientation {
        Orientation::Upright => [0.0, 0.0, 1.0],
        Orientation::Axis(axis) => axis,
        Orientation::SurfaceNormal => {
            return Err(Error::Invalid("gem surface orientation on a curve"));
        }
    };
    let projected = perpendicular(hint, tangent);
    if norm(projected) > 1.0e-9 {
        return unit(projected, "gem curve axis");
    }
    fallback_perpendicular(tangent)
}

/// Build the rigid frame from an orthonormal basis and an origin.
fn frame_transform(origin: Point3, x: Point3, y: Point3, z: Point3) -> Result<Transform> {
    let matrix = [
        [x[0], y[0], z[0], origin[0]],
        [x[1], y[1], z[1], origin[1]],
        [x[2], y[2], z[2], origin[2]],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let transform = Transform(matrix);
    transform.validate()?;
    Ok(transform)
}

/// The frame of a gem whose local +Z is `axis`, local +X follows `along` and the whole frame
/// is spun by `spin_deg` about +Z.
fn spin_frame(
    origin: Point3,
    along: Point3,
    axis: Point3,
    spin_deg: f64,
) -> Result<(Transform, Point3, Point3, Point3)> {
    let z = unit(axis, "gem frame axis")?;
    let projected = perpendicular(along, z);
    let x0 = if norm(projected) > 1.0e-9 {
        unit(projected, "gem frame x")?
    } else {
        fallback_perpendicular(z)?
    };
    let y0 = unit(cross(z, x0), "gem frame y")?;
    let (sin, cos) = spin_deg.to_radians().sin_cos();
    let x = unit(add(mul(x0, cos), mul(y0, sin)), "gem frame x")?;
    let y = unit(cross(z, x), "gem frame y")?;
    let transform = frame_transform(origin, x, y, z)?;
    Ok((transform, x, y, z))
}

/* ------------------------------- curve placement ---------------------------- */

/// Evaluate the requested arc range into ordered stations. A closed curve's seam station is
/// dropped when it would duplicate the start.
fn path_samples(
    curve: &Curve,
    params: &CurveParams,
    limits: &Limits,
) -> Result<(Vec<PathSample>, f64, bool, [f64; 2])> {
    curve.validate(limits)?;
    params.validate()?;
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 1.0e-9 {
        return Err(Error::Invalid("gem placement curve length"));
    }
    let end = params.end_mm.unwrap_or(total);
    if params.start_mm > total + 1.0e-9 {
        return Err(Error::Invalid("gem placement start beyond curve"));
    }
    if end > total + 1.0e-9 {
        return Err(Error::Invalid("gem placement end beyond curve"));
    }
    let start = params.start_mm.min(total);
    let end = end.min(total).max(start);
    let tolerance = 1.0e-9 * total.max(1.0);
    let closed = curve_ops::is_closed(curve, tolerance, limits)?;
    let mut samples = Vec::new();
    let mut travelled = start;
    while travelled <= end + 1.0e-9 {
        let at_seam = closed && params.skip_seam && (total - travelled).abs() <= tolerance;
        if !at_seam {
            let fraction = curve_ops::parameter_at_length(curve, travelled.min(total), limits)?;
            let point = curve_ops::point_at(curve, fraction, limits)?;
            let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
            samples.push(PathSample {
                fraction,
                arc_length_mm: travelled,
                point,
                tangent,
            });
            if samples.len() > MAX_STATIONS {
                return Err(Error::Budget("gem placement stations"));
            }
        }
        travelled += params.spacing_mm;
    }
    if samples.is_empty() {
        return Err(Error::Invalid("gem placement range too short"));
    }
    Ok((samples, total, closed, [start, end]))
}

/// The footprint of a stone along the path: the larger girdle dimension.
fn footprint_mm(spec: &GemSpec) -> f64 {
    spec.length_mm.max(spec.width_mm)
}

/// Consecutive gaps plus, for a run covering a whole closed curve, the seam gap.
fn gaps_of(samples: &[PathSample], total: f64, closed: bool, full_range: bool) -> Vec<f64> {
    let mut gaps: Vec<f64> = samples
        .windows(2)
        .map(|pair| pair[1].arc_length_mm - pair[0].arc_length_mm)
        .collect();
    if closed && full_range && samples.len() > 1 {
        let first = samples[0].arc_length_mm;
        let last = samples[samples.len() - 1].arc_length_mm;
        gaps.push((total - last) + first);
    }
    gaps
}

fn check_overlap(samples: &[PathSample], specs: &[GemSpec], allow_overlap: bool) -> Result<()> {
    if allow_overlap {
        return Ok(());
    }
    for pair in samples.windows(2).enumerate() {
        let (index, pair) = pair;
        let gap = pair[1].arc_length_mm - pair[0].arc_length_mm;
        let needed = 0.5 * (footprint_mm(&specs[index]) + footprint_mm(&specs[index + 1]));
        if gap + 1.0e-9 < needed {
            return Err(Error::Invalid("gem spacing smaller than gem size"));
        }
    }
    Ok(())
}

fn spec_at(specs: &[GemSpec], index: usize) -> Result<&GemSpec> {
    specs
        .get(index)
        .ok_or(Error::Invalid("gem spec list is shorter than the stations"))
}

/// Assemble the typed placement from already-validated samples and specs.
fn assemble(
    curve_id: Option<ObjectId>,
    params: &CurveParams,
    total: f64,
    closed: bool,
    range: [f64; 2],
    samples: &[PathSample],
    specs: &[GemSpec],
) -> Result<CurvePlacement> {
    if samples.len() != specs.len() {
        return Err(Error::Invalid("gem spec count"));
    }
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(samples.len())
        .map_err(|_| Error::Budget("gem stations"))?;
    for (index, sample) in samples.iter().enumerate() {
        let spec = spec_at(specs, index)?;
        spec.validate()?;
        let axis = curve_axis(sample.tangent, params.orientation)?;
        let origin = add(sample.point, mul(axis, params.z_offset_mm));
        let (transform, _, _, _) = spin_frame(
            origin,
            sample.tangent,
            axis,
            params.rotation_deg,
        )?;
        stations.push(GemStation {
            index,
            fraction: sample.fraction,
            arc_length_mm: sample.arc_length_mm,
            point: origin,
            tangent: sample.tangent,
            axis,
            uv: None,
            transform,
            spec: spec.clone(),
            z_offset_mm: params.z_offset_mm,
            spin_deg: params.rotation_deg,
            gem: None,
        });
    }
    let full_range = range[0] <= 1.0e-9 && (total - range[1]).abs() <= 1.0e-9 * total.max(1.0);
    let gaps = gaps_of(samples, total, closed, full_range);
    let placement = CurvePlacement {
        curve: curve_id,
        params: *params,
        total_length_mm: total,
        range,
        closed,
        stations,
        gaps_mm: gaps,
    };
    placement.stations.iter().try_for_each(|station| {
        station.transform.validate()?;
        Ok::<(), Error>(())
    })?;
    Ok(placement)
}

/// Build the one station at a given arc length on a curve (`OM9-L3-04-10-008`).
///
/// This is the primitive a link-based follow uses: the gem keeps its recorded arc length,
/// while the point, tangent and frame are re-evaluated from the edited source. A station
/// beyond the new curve end is an explicit refusal, so the caller can turn it into a
/// visible conflict instead of moving the stone to a wrong place.
pub fn station_at_arc(
    curve: &Curve,
    arc_length_mm: f64,
    spec: &GemSpec,
    params: &CurveParams,
    limits: &Limits,
) -> Result<GemStation> {
    spec.validate()?;
    params.validate()?;
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 1.0e-9 {
        return Err(Error::Invalid("gem placement curve length"));
    }
    if !arc_length_mm.is_finite() || arc_length_mm < 0.0 || arc_length_mm > total + 1.0e-9 {
        return Err(Error::Invalid("gem station beyond the source curve"));
    }
    let fraction = curve_ops::parameter_at_length(curve, arc_length_mm.min(total), limits)?;
    let point = curve_ops::point_at(curve, fraction, limits)?;
    let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
    let axis = curve_axis(tangent, params.orientation)?;
    let origin = add(point, mul(axis, params.z_offset_mm));
    let (transform, _, _, axis) = spin_frame(origin, tangent, axis, params.rotation_deg)?;
    Ok(GemStation {
        index: 0,
        fraction,
        arc_length_mm,
        point: origin,
        tangent,
        axis,
        uv: None,
        transform,
        spec: spec.clone(),
        z_offset_mm: params.z_offset_mm,
        spin_deg: params.rotation_deg,
        gem: None,
    })
}

/// Place evenly spaced stones on one curve (`OM9-L3-04-10-001`).
///
/// Stations are measured in arc length, so a NURBS curve with an uneven parameterisation
/// still gets an even row. A closed curve's seam station is dropped when `skip_seam` is set,
/// and a stone longer than the spacing is refused unless `allow_overlap` is set.
pub fn place_on_curve(
    curve: &Curve,
    spec: &GemSpec,
    params: &CurveParams,
    limits: &Limits,
) -> Result<CurvePlacement> {
    spec.validate()?;
    let (samples, total, closed, range) = path_samples(curve, params, limits)?;
    let specs = vec![spec.clone(); samples.len()];
    check_overlap(&samples, &specs, params.allow_overlap)?;
    assemble(None, params, total, closed, range, &samples, &specs)
}

/// Place a list of stones in order (`OM9-L3-04-10-002`, `-004`).
///
/// A one-element list is broadcast; any other length must match the station count exactly,
/// so a caller cannot silently lose or duplicate a stone.
pub fn place_on_curve_variable(
    curve: &Curve,
    specs: &[GemSpec],
    params: &CurveParams,
    limits: &Limits,
) -> Result<CurvePlacement> {
    if specs.is_empty() {
        return Err(Error::Missing("gem spec list"));
    }
    specs.iter().try_for_each(GemSpec::validate)?;
    let (samples, total, closed, range) = path_samples(curve, params, limits)?;
    let resolved: Vec<GemSpec> = if specs.len() == 1 {
        vec![specs[0].clone(); samples.len()]
    } else if specs.len() == samples.len() {
        specs.to_vec()
    } else {
        return Err(Error::Invalid("gem spec count"));
    };
    check_overlap(&samples, &resolved, params.allow_overlap)?;
    assemble(None, params, total, closed, range, &samples, &resolved)
}

/// A tapering ladder of stones from `start` to `end` (`OM9-L3-04-10-002`).
///
/// Dimensions interpolate linearly and every produced stone is validated by the owner, so a
/// taper can never reach a zero diameter. One stone yields the start size.
pub fn taper_specs(start: &GemSpec, end: &GemSpec, count: usize) -> Result<Vec<GemSpec>> {
    if count == 0 {
        return Err(Error::Invalid("gem taper count"));
    }
    if count > MAX_STATIONS {
        return Err(Error::Budget("gem taper count"));
    }
    start.validate()?;
    end.validate()?;
    let mut specs = Vec::new();
    specs
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("gem taper"))?;
    for index in 0..count {
        let t = if count == 1 {
            0.0
        } else {
            index as f64 / (count - 1) as f64
        };
        let length = start.length_mm + (end.length_mm - start.length_mm) * t;
        let width = start.width_mm + (end.width_mm - start.width_mm) * t;
        let depth = start.depth_mm() + (end.depth_mm() - start.depth_mm()) * t;
        let spec = start
            .clone()
            .set_girdle_mm(length, width)?
            .set_depth_mm(depth)?;
        spec.validate()?;
        specs.push(spec);
    }
    Ok(specs)
}

/// The feasible count for `spec` with a gap and an end allowance (`OM9-L3-04-10-003`).
///
/// The answer is the largest `n` with `n*length + (n-1)*gap <= total - 2*allowance`; a curve
/// that cannot hold one stone reports `count = 0, fits = false` instead of a negative count.
pub fn count_on_curve(
    curve: &Curve,
    spec: &GemSpec,
    gap_mm: f64,
    end_allowance_mm: f64,
    limits: &Limits,
) -> Result<CurveFit> {
    spec.validate()?;
    if !gap_mm.is_finite() || gap_mm < 0.0 {
        return Err(Error::Invalid("gem gap"));
    }
    if !end_allowance_mm.is_finite() || end_allowance_mm < 0.0 {
        return Err(Error::Invalid("gem end allowance"));
    }
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 1.0e-9 {
        return Err(Error::Invalid("curve length"));
    }
    let usable = total - 2.0 * end_allowance_mm;
    let footprint = footprint_mm(spec);
    let pitch = footprint + gap_mm;
    if usable + 1.0e-9 < footprint || pitch <= 0.0 {
        return Ok(CurveFit {
            count: 0,
            fits: false,
            usable_mm: usable.max(0.0),
            spacing_mm: pitch,
            first_center_mm: 0.0,
            last_center_mm: 0.0,
        });
    }
    let mut count = ((usable + gap_mm) / pitch).floor().max(1.0) as usize;
    while count > 1 && count as f64 * footprint + (count as f64 - 1.0) * gap_mm > usable + 1.0e-9 {
        count -= 1;
    }
    let first = end_allowance_mm + footprint * 0.5;
    let last = first + pitch * (count as f64 - 1.0);
    Ok(CurveFit {
        count,
        fits: true,
        usable_mm: usable,
        spacing_mm: pitch,
        first_center_mm: first,
        last_center_mm: last,
    })
}

/// Place the same stone on several curves in one atomic run (`OM9-L3-04-10-005`).
///
/// The whole call fails when any path fails, so the caller never receives a half-filled
/// result. Each group keeps its own station order and identities are created by the caller,
/// so two paths cannot share a gem id.
pub fn place_on_curves(
    curves: &[Curve],
    spec: &GemSpec,
    params: &CurveParams,
    limits: &Limits,
) -> Result<MultiPlacement> {
    if curves.is_empty() {
        return Err(Error::Missing("gem placement curves"));
    }
    let mut groups = Vec::new();
    groups
        .try_reserve_exact(curves.len())
        .map_err(|_| Error::Budget("gem placement groups"))?;
    for curve in curves {
        groups.push(place_on_curve(curve, spec, params, limits)?);
    }
    Ok(MultiPlacement { groups })
}

/// Profile, girdle and guide curves for a placement (`OM9-L3-04-10-006`).
///
/// The curves are new geometry owned by the caller; the placement and the source curve are
/// not modified, so Cancel cannot leave a half-built guide behind.
pub fn profile_and_guides(
    placement: &CurvePlacement,
    limits: &Limits,
) -> Result<PlacementGuides> {
    if placement.stations.len() < 2 {
        return Err(Error::Invalid("gem profile needs two stations"));
    }
    let girdle_points: Vec<Point3> = placement.stations.iter().map(|s| s.point).collect();
    let table_points: Vec<Point3> = placement.stations.iter().map(GemStation::table_point).collect();
    let girdle = crate::curve_build::polyline(&girdle_points)?;
    let profile = crate::curve_build::polyline(&table_points)?;
    girdle.validate(limits)?;
    profile.validate(limits)?;
    let mut guides = Vec::new();
    guides
        .try_reserve_exact(placement.stations.len())
        .map_err(|_| Error::Budget("gem guides"))?;
    for (girdle_point, table_point) in girdle_points.iter().zip(table_points.iter()) {
        let distance = norm(sub(*table_point, *girdle_point));
        if !distance.is_finite() || distance <= 1.0e-12 {
            return Err(Error::Invalid("degenerate gem guide"));
        }
        let guide = crate::curve_build::polyline(&[*girdle_point, *table_point])?;
        guide.validate(limits)?;
        guides.push(guide);
    }
    Ok(PlacementGuides {
        profile,
        girdle,
        guides,
        table_points,
    })
}

/* ------------------------------ surface placement --------------------------- */

/// The frame at one surface parameter: point, unit U tangent, unit V tangent and unit normal.
fn surface_frame(
    surface: &NurbsSurface,
    uv: [f64; 2],
    limits: &Limits,
) -> Result<(Point3, Point3, Point3, Point3)> {
    let domain = surface.domain;
    for axis in 0..2 {
        let span = domain[axis][1] - domain[axis][0];
        if !span.is_finite() || span <= 0.0 {
            return Err(Error::Invalid("gem surface domain"));
        }
        let tolerance = 1.0e-9 * span;
        if uv[axis] < domain[axis][0] - tolerance || uv[axis] > domain[axis][1] + tolerance {
            return Err(Error::Invalid("gem surface parameter outside the domain"));
        }
    }
    let derivatives = crate::surface_kernel::derivatives(surface, uv[0], uv[1], limits)?;
    let normal = crate::surface_kernel::normal(surface, uv[0], uv[1], limits)?;
    let u_axis = unit(derivatives.du, "gem surface u tangent")?;
    let v_axis = unit(derivatives.dv, "gem surface v tangent")?;
    Ok((derivatives.point, u_axis, v_axis, normal))
}

/// Place stones at explicit surface parameters (`OM9-L3-04-10-007`).
///
/// A parameter outside the surface domain, or a station where the surface has no normal
/// (a pole or a degenerate patch), is refused by name; the frame is always orthonormal.
pub fn place_on_surface(
    surface: &NurbsSurface,
    specs: &[GemSpec],
    params: &SurfaceParams,
    limits: &Limits,
) -> Result<SurfacePlacement> {
    surface.validate(limits)?;
    params.validate()?;
    if specs.is_empty() {
        return Err(Error::Missing("gem surface specs"));
    }
    specs.iter().try_for_each(GemSpec::validate)?;
    if specs.len() != 1 && specs.len() != params.uv.len() {
        return Err(Error::Invalid("gem spec count"));
    }
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(params.uv.len())
        .map_err(|_| Error::Budget("gem surface stations"))?;
    for (index, uv) in params.uv.iter().enumerate() {
        let spec = if specs.len() == 1 {
            &specs[0]
        } else {
            spec_at(specs, index)?
        };
        let (point, u_axis, _, normal) = surface_frame(surface, *uv, limits)?;
        let origin = add(point, mul(normal, params.z_offset_mm));
        let (transform, _, _, axis) =
            spin_frame(origin, u_axis, normal, params.rotation_deg)?;
        stations.push(GemStation {
            index,
            fraction: 0.0,
            arc_length_mm: 0.0,
            point: origin,
            tangent: u_axis,
            axis,
            uv: Some(*uv),
            transform,
            spec: spec.clone(),
            z_offset_mm: params.z_offset_mm,
            spin_deg: params.rotation_deg,
            gem: None,
        });
    }
    Ok(SurfacePlacement {
        surface: None,
        params: params.clone(),
        stations,
    })
}

/* ---------------------------------- follow ---------------------------------- */

/// Recompute a placement after its source curve changed (`OM9-L3-04-10-008`).
///
/// Surviving stations keep their gem identity and report how far they moved. A stone whose
/// station no longer exists becomes an explicit [`FollowConflict`]; it is never dropped.
pub fn follow_placement(
    previous: &CurvePlacement,
    curve: &Curve,
    limits: &Limits,
) -> Result<FollowOutcome> {
    if previous.stations.is_empty() {
        return Err(Error::Missing("gem placement stations"));
    }
    let params = previous.params;
    let (samples, total, closed, range) = match path_samples(curve, &params, limits) {
        Ok(value) => value,
        // A source that can no longer evaluate (deleted or degenerate) is a stale link.
        Err(Error::Invalid(_)) => return Err(Error::Stale("gem placement source curve")),
        Err(error) => return Err(error),
    };
    // A station that still exists keeps its stone; a station appended by a longer source
    // takes the last known stone, so the row keeps a legal size instead of failing.
    let last_spec = previous
        .stations
        .last()
        .map(|station| station.spec.clone())
        .ok_or(Error::Missing("gem placement stations"))?;
    let specs: Vec<GemSpec> = (0..samples.len())
        .map(|index| {
            previous
                .stations
                .get(index)
                .map_or_else(|| last_spec.clone(), |station| station.spec.clone())
        })
        .collect();
    let mut placement = assemble(
        previous.curve,
        &params,
        total,
        closed,
        range,
        &samples,
        &specs,
    )?;
    let mut conflicts = Vec::new();
    if previous.closed != closed {
        for station in previous.stations.iter() {
            conflicts.push(FollowConflict {
                station: station.index,
                gem: station.gem,
                reason: "closed curve topology changed",
            });
        }
    }
    for (index, station) in placement.stations.iter_mut().enumerate() {
        if let Some(old) = previous.stations.get(index) {
            station.gem = old.gem;
        }
    }
    for old in previous.stations.iter().skip(placement.stations.len()) {
        conflicts.push(FollowConflict {
            station: old.index,
            gem: old.gem,
            reason: "source curve shortened",
        });
    }
    let mut movement = Vec::new();
    movement
        .try_reserve_exact(placement.stations.len())
        .map_err(|_| Error::Budget("gem follow movement"))?;
    for (index, station) in placement.stations.iter().enumerate() {
        // An appended station has no previous place, so it reports no movement.
        let moved = previous.stations.get(index).map_or(0.0, |old| {
            norm(sub(station.point, old.point))
        });
        movement.push(moved);
    }
    Ok(FollowOutcome {
        placement,
        conflicts,
        movement_mm: movement,
    })
}

/* ----------------------------------- flow ----------------------------------- */

/// Rodrigues rotation that carries the unit `from` onto the unit `to`, applied to `value`.
fn rotate_from_to(value: Point3, from: Point3, to: Point3) -> Result<Point3> {
    let from = unit(from, "gem flow source tangent")?;
    let to = unit(to, "gem flow target tangent")?;
    let axis = cross(from, to);
    let sin = norm(axis);
    let cos = dot(from, to).clamp(-1.0, 1.0);
    if sin <= 1.0e-12 {
        if cos > 0.0 {
            return Ok(value);
        }
        // A half turn has no unique axis; any perpendicular direction turns the vector over.
        let axis = fallback_perpendicular(from)?;
        return Ok(sub(mul(axis, 2.0 * dot(axis, value)), value));
    }
    let axis = unit(axis, "gem flow rotation")?;
    let term1 = mul(value, cos);
    let term2 = mul(cross(axis, value), sin);
    let term3 = mul(axis, dot(axis, value) * (1.0 - cos));
    Ok(add(add(term1, term2), term3))
}

/// Carry one station from a source curve to a target curve (`OM9-L3-04-10-009`).
///
/// The station keeps its share of the arc length and its stone; the rigid frame follows the
/// target tangent, so the gem is never stretched.
pub fn flow_station(
    source: &Curve,
    target: &Curve,
    station: &GemStation,
    limits: &Limits,
) -> Result<GemStation> {
    let source_total = curve_ops::arc_length(source, limits)?;
    let target_total = curve_ops::arc_length(target, limits)?;
    if source_total <= 1.0e-9 || target_total <= 1.0e-9 {
        return Err(Error::Invalid("gem flow curve length"));
    }
    let share = (station.arc_length_mm / source_total).clamp(0.0, 1.0);
    let target_arc = share * target_total;
    let fraction = curve_ops::parameter_at_length(target, target_arc, limits)?;
    let point = curve_ops::point_at(target, fraction, limits)?;
    let tangent = curve_ops::tangent_at(target, fraction, limits)?;
    let source_tangent = curve_ops::tangent_at(source, station.fraction, limits)?;
    let axis = unit(
        rotate_from_to(station.axis, source_tangent, tangent)?,
        "gem flow axis",
    )?;
    let origin = add(point, mul(axis, station.z_offset_mm));
    let (transform, _, _, axis) =
        spin_frame(origin, tangent, axis, station.spin_deg)?;
    let mut flowed = station.clone();
    flowed.index = station.index;
    flowed.fraction = fraction;
    flowed.arc_length_mm = target_arc;
    flowed.point = origin;
    flowed.tangent = tangent;
    flowed.axis = axis;
    flowed.transform = transform;
    Ok(flowed)
}

/// Flow a whole placement from a source curve to a target curve (`OM9-L3-04-10-009`).
pub fn flow_placement(
    source: &Curve,
    target: &Curve,
    placement: &CurvePlacement,
    limits: &Limits,
) -> Result<CurvePlacement> {
    if placement.stations.is_empty() {
        return Err(Error::Missing("gem placement stations"));
    }
    let source_total = curve_ops::arc_length(source, limits)?;
    let target_total = curve_ops::arc_length(target, limits)?;
    if source_total <= 1.0e-9 || target_total <= 1.0e-9 {
        return Err(Error::Invalid("gem flow curve length"));
    }
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(placement.stations.len())
        .map_err(|_| Error::Budget("gem flow stations"))?;
    for station in &placement.stations {
        stations.push(flow_station(source, target, station, limits)?);
    }
    let mut params = placement.params;
    params.spacing_mm *= target_total / source_total;
    let gaps = stations
        .windows(2)
        .map(|pair| pair[1].arc_length_mm - pair[0].arc_length_mm)
        .collect();
    let closed = curve_ops::is_closed(target, 1.0e-9 * target_total.max(1.0), limits)?;
    Ok(CurvePlacement {
        curve: None,
        params,
        total_length_mm: target_total,
        range: [
            stations[0].arc_length_mm,
            stations[stations.len() - 1].arc_length_mm,
        ],
        closed,
        stations,
        gaps_mm: gaps,
    })
}

/// Flow a placement from a base surface to a target surface (`OM9-L3-04-10-009`).
///
/// The rigid transport of [`crate::deform::rigid_flow`] keeps each gem's share of the
/// surface and turns its frame from the base normal onto the target normal, so the stone is
/// not deformed.
pub fn flow_surface_placement(
    base: &NurbsSurface,
    target: &NurbsSurface,
    placement: &CurvePlacement,
    limits: &Limits,
) -> Result<CurvePlacement> {
    if placement.stations.is_empty() {
        return Err(Error::Missing("gem placement stations"));
    }
    base.validate(limits)?;
    target.validate(limits)?;
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(placement.stations.len())
        .map_err(|_| Error::Budget("gem flow stations"))?;
    for station in &placement.stations {
        let transform = deform::rigid_flow(base, target, &station.transform, limits)?;
        let point = transform.point([0.0, 0.0, 0.0])?;
        let matrix = transform.0;
        let axis = unit(
            [matrix[0][2], matrix[1][2], matrix[2][2]],
            "gem flow axis",
        )?;
        let mut flowed = station.clone();
        flowed.point = point;
        flowed.axis = axis;
        flowed.transform = transform;
        stations.push(flowed);
    }
    Ok(CurvePlacement {
        curve: None,
        params: placement.params,
        total_length_mm: placement.total_length_mm,
        range: placement.range,
        closed: placement.closed,
        stations,
        gaps_mm: placement.gaps_mm.clone(),
    })
}

/* ---------------------------------- splop ----------------------------------- */

/// Project one gem onto a surface with the surface normal as its axis (`OM9-L3-04-10-010`).
///
/// The gem is moved rigidly: only its frame changes. The projection is the closest point of
/// the surface, the table axis is the surface normal there, and the local X axis follows the
/// reference hint, so a stone that landed near a pole or on a flipped patch is refused
/// instead of being distorted.
pub fn splop_station(
    surface: &NurbsSurface,
    reference: Point3,
    spec: &GemSpec,
    params: &SplopParams,
    limits: &Limits,
) -> Result<GemStation> {
    surface.validate(limits)?;
    spec.validate()?;
    params.validate()?;
    if !reference.iter().all(|value| value.is_finite()) {
        return Err(Error::Invalid("gem splop reference point"));
    }
    let projection = curve_surface_ops::closest_point_on_surface(surface, reference, limits)?;
    if !projection.distance.is_finite() {
        return Err(Error::Invalid("gem splop projection"));
    }
    let (point, u_axis, _, normal) = surface_frame(surface, projection.parameters, limits)?;
    let hint = perpendicular(params.x_hint, normal);
    let along = if norm(hint) > 1.0e-9 {
        unit(hint, "gem splop reference")?
    } else {
        u_axis
    };
    let origin = add(point, mul(normal, params.z_offset_mm));
    let (transform, _, _, axis) =
        spin_frame(origin, along, normal, params.rotation_deg)?;
    Ok(GemStation {
        index: 0,
        fraction: 0.0,
        arc_length_mm: 0.0,
        point: origin,
        tangent: along,
        axis,
        uv: Some(projection.parameters),
        transform,
        spec: spec.clone(),
        z_offset_mm: params.z_offset_mm,
        spin_deg: params.rotation_deg,
        gem: None,
    })
}

/// Splop every station of a placement onto a surface (`OM9-L3-04-10-010`).
pub fn splop_placement(
    surface: &NurbsSurface,
    placement: &CurvePlacement,
    params: &SplopParams,
    limits: &Limits,
) -> Result<CurvePlacement> {
    if placement.stations.is_empty() {
        return Err(Error::Missing("gem placement stations"));
    }
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(placement.stations.len())
        .map_err(|_| Error::Budget("gem splop stations"))?;
    for station in &placement.stations {
        let mut landed = splop_station(surface, station.point, &station.spec, params, limits)?;
        landed.index = station.index;
        landed.gem = station.gem;
        stations.push(landed);
    }
    Ok(CurvePlacement {
        curve: None,
        params: placement.params,
        total_length_mm: placement.total_length_mm,
        range: placement.range,
        closed: placement.closed,
        stations,
        gaps_mm: placement.gaps_mm.clone(),
    })
}
