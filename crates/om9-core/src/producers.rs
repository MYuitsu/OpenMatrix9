// SPDX-License-Identifier: LGPL-2.1-or-later
//! Real geometry producers for the associative history (`OM9-L2-01-04` follow-up).
//!
//! The history kernel deliberately takes a producer, so this module supplies the concrete
//! rail/profile/sweep chain the builder owner calls:
//!
//! * [`RingRailProducer`] turns a finger diameter into the closed rail centerline;
//! * [`RailProfileProducer`] places a rectangular profile on the rail start frame;
//! * [`RailSweepProducer`] sweeps that profile along the rail through a
//!   [`SweepBackend`] (the OCCT adapter lives in `om9-kernel`, the analytic reference
//!   backend here);
//! * [`SnapshotTopology`] names the edges/faces/vertices of the produced geometry so a
//!   [`SemanticReference`](crate::associative::SemanticReference) can resolve after a
//!   rebuild.
//!
//! [`ProducerRegistry`] maps a recipe command to its producer and captures the topology
//! policy, so `recompute_registered` is the one call a host needs.
use crate::{
    Error, Result,
    associative::{
        ProducedFeature, ProducedSubshapes, RebuildRequest, RecomputeCache, RecomputeOutcome,
        Subshape, SubshapeKind, SubshapeSignature,
    },
    budget::{Limits, Usage},
    curve_build, curve_ops,
    geometry::{Curve, Geometry, GeometryKind, Point3, finite},
    identity::{HistoryId, ObjectId},
    model::Document,
    request::Value,
};
use std::collections::{BTreeMap, BTreeSet};

pub const RING_RAIL_COMMAND: &str = "RingRail";
pub const RAIL_PROFILE_COMMAND: &str = "RailProfile";
pub const RAIL_SWEEP_COMMAND: &str = "RailSweep";
pub const JOIN_HISTORY_COMMAND: &str = "JoinHistory";

fn unit(vector: Point3) -> Result<Point3> {
    finite(vector)?;
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("producer frame"));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn single_output(request: &RebuildRequest) -> Result<ObjectId> {
    match request.outputs.as_slice() {
        [output] => Ok(*output),
        _ => Err(Error::Invalid("producer output count")),
    }
}

fn input_curve<'a>(request: &'a RebuildRequest, role: &str) -> Result<&'a Curve> {
    let input = request
        .inputs
        .iter()
        .find(|input| input.role == role)
        .ok_or(Error::Missing("producer input"))?;
    match &input.entity.geometry {
        Geometry::Curve(curve) => Ok(curve),
        _ => Err(Error::Invalid("producer curve input")),
    }
}

fn length_parameter(request: &RebuildRequest, key: &str) -> Result<f64> {
    request
        .parameters
        .get(key)
        .ok_or(Error::Missing("producer parameter"))?
        .as_length()
        .map(|value| value.mm())
        .map_err(Into::into)
}

fn positive_length(request: &RebuildRequest, key: &str) -> Result<f64> {
    let value = length_parameter(request, key)?;
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::Invalid("producer parameter range"));
    }
    Ok(value)
}

fn boolean_parameter(request: &RebuildRequest, key: &str) -> Result<bool> {
    match request.parameters.get(key) {
        None => Ok(false),
        Some(Value::Boolean(value)) => Ok(*value),
        Some(_) => Err(Error::Invalid("producer boolean parameter")),
    }
}

/// A dimensionless 0..1 parameter. A missing value is 0, so older recipes that predate a
/// placement parameter keep their original seam instead of failing to replay.
fn ratio_parameter(request: &RebuildRequest, key: &str) -> Result<f64> {
    match request.parameters.get(key) {
        None => Ok(0.0),
        Some(value) => {
            let ratio = value.as_ratio()?;
            if !ratio.is_finite() {
                return Err(Error::Invalid("producer ratio parameter"));
            }
            Ok(ratio)
        }
    }
}

/// An angle parameter in degrees. A missing value is 0 for the same replay reason.
fn angle_parameter(request: &RebuildRequest, key: &str) -> Result<f64> {
    match request.parameters.get(key) {
        None => Ok(0.0),
        Some(value) => {
            let angle = value.as_angle()?.deg();
            if !angle.is_finite() {
                return Err(Error::Invalid("producer angle parameter"));
            }
            Ok(angle)
        }
    }
}

/// A backend that turns a rail and a profile curve into an output geometry. The analytic
/// backend uses `surface_kernel::sweep1`; the OCCT backend in `om9-kernel` builds a B-Rep.
pub trait SweepBackend {
    fn sweep(
        &self,
        rail: &Curve,
        profile: &Curve,
        solid: bool,
        limits: &Limits,
    ) -> Result<Geometry>;
    /// Transient native results must travel beside canonical geometry until
    /// the host transfers their display into an accounted representation.
    fn sweep_with_candidates(&self, rail: &Curve, profile: &Curve, solid: bool,
        limits: &Limits) -> Result<(Geometry, Vec<crate::brep::Brep>)> {
        Ok((self.sweep(rail, profile, solid, limits)?, Vec::new()))
    }
}

/// The analytic reference backend: a NURBS surface swept from the profile along the rail.
/// It needs no native kernel, which keeps the producer chain testable in `om9-core`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SurfaceSweepBackend;

impl SweepBackend for SurfaceSweepBackend {
    fn sweep(
        &self,
        rail: &Curve,
        profile: &Curve,
        _solid: bool,
        limits: &Limits,
    ) -> Result<Geometry> {
        crate::surface_kernel::sweep1(profile, rail, [17, 33], [3, 3], limits)
            .map(Geometry::Surface)
    }
}

/// A feature producer: the durable command record already fixed the output identities, so a
/// producer returns geometry for exactly those objects.
pub trait FeatureProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>>;
    /// Opt in to receiving caller-supplied geometry limits. This does not reserve
    /// process memory or certify every temporary allocation made by a producer.
    fn produce_with_limits(&self, _request: &RebuildRequest, _limits: &Limits,
        _cancel: &crate::transaction::Cancellation)
        -> Result<Vec<crate::associative::RebuiltOutput>> {
        Err(Error::Unsupported("limited history producer"))
    }
    fn produce_with_candidates(&self, request: &RebuildRequest)
        -> Result<(Vec<crate::associative::RebuiltOutput>, Vec<crate::brep::Brep>)> {
        Ok((self.produce(request)?, Vec::new()))
    }
    fn produce_limited_with_candidates(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation)
        -> Result<(Vec<crate::associative::RebuiltOutput>, Vec<crate::brep::Brep>)> {
        Ok((self.produce_with_limits(request, limits, cancel)?, Vec::new()))
    }
}

/// Rebuild a joined curve from the current canonical parent entities.
pub struct JoinHistoryProducer;

impl FeatureProducer for JoinHistoryProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        self.produce_with_limits(request, &Limits::default(), &crate::transaction::Cancellation::default())
    }
    fn produce_with_limits(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation)
        -> Result<Vec<crate::associative::RebuiltOutput>> {
        let output = single_output(request)?;
        let tolerance = positive_length(request, "tolerance_mm")?;
        let mut scratch = Usage::btree::<ObjectId, ()>(request.inputs.len())?;
        scratch.add(Usage::array::<&crate::model::Entity>(request.inputs.len())?, limits)?;
        for input in &request.inputs {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let Geometry::Curve(curve) = &input.entity.geometry else {
                return Err(Error::Invalid("producer curve input"));
            };
            scratch.add(curve.validate(&limits.remaining_after(scratch)?)?, limits)?;
        }
        let mut seen = std::collections::BTreeSet::new();
        if request.inputs.iter().any(|input| {
            input.role != "curve"
                || input.object != input.entity.id
                || input.object == output
                || !seen.insert(input.object)
        }) {
            return Err(Error::Invalid("join history inputs"));
        }
        let mut entities = Vec::new();
        entities
            .try_reserve_exact(request.inputs.len())
            .map_err(|_| Error::Budget("join history inputs"))?;
        entities.extend(request.inputs.iter().map(|input| &input.entity));
        let curve = crate::top11::join_curve_entities_guarded(
            &entities, tolerance, &limits.remaining_after(scratch)?,
            &mut || if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) },
        )?;
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Curve(curve),
        }])
    }
}

/// `RingRail`: a closed circle centerline at `diameter_mm` on the world XY plane.
///
/// A jewelry standard may replace the explicit diameter: `standard=US` with `us_size` resolves
/// through the frozen [`crate::ring_size`] table, and an unverified standard is refused by
/// name. The explicit `diameter_mm` remains the authoritative value when no standard is set.
#[derive(Debug, Clone, Copy, Default)]
pub struct RingRailProducer;

impl FeatureProducer for RingRailProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        let output = single_output(request)?;
        let standard = match request.parameters.get("standard") {
            None => "explicit",
            Some(Value::Choice(id)) => id.as_str(),
            Some(_) => return Err(Error::Invalid("ring size standard")),
        };
        let diameter = if standard.eq_ignore_ascii_case("explicit") || standard.eq_ignore_ascii_case("VNM") || standard.is_empty() {
            positive_length(request, "diameter_mm")?
        } else {
            let size = match request.parameters.get("us_size") {
                Some(Value::Quantity { value, dimension: crate::request::Dimension::Count | crate::request::Dimension::Ratio }) => *value,
                _ => return Err(Error::Invalid("ring size value")),
            };
            crate::ring_size::rail_diameter_mm(standard, size)?
        };
        if let Some(frame_value) = request.parameters.get("rail_frame") {
            let diameter = crate::ring_size::rail_diameter_mm("diameter", diameter)?;
            let Value::Choice(frame) = frame_value else { return Err(Error::Invalid("ring rail frame")) };
            if !matches!(frame.as_str(), "front" | "xy") { return Err(Error::Unsupported("ring rail frame")); }
            let factor = match request.parameters.get("model_mm_per_unit") { None => 1., Some(value) => value.as_ratio()? };
            if !factor.is_finite() || factor <= 0. { return Err(Error::Invalid("ring rail model units")); }
            let coordinate = |name| -> Result<f64> {
                if request.parameters.contains_key(name) { length_parameter(request, name) } else { Ok(0.) }
            };
            let center = [coordinate("center_x_mm")? / factor, coordinate("center_y_mm")? / factor, coordinate("center_z_mm")? / factor];
            let curve = match request.parameters.get("custom_rail") {
                None => crate::ring_size::rail_curve(center, diameter / factor, frame)?,
                Some(Value::Text(text)) => crate::ring_rail_shape::RailShape::from_json(text, &Limits::default())?
                    .sized_curve(diameter, center, frame, factor, &Limits::default())?,
                _ => return Err(Error::Invalid("custom Ring Rail snapshot")),
            };
            return Ok(vec![crate::associative::RebuiltOutput { object: output,
                geometry: Geometry::Curve(curve) }]);
        }
        // Recipes predating rail_frame retain their original XY orientation.
        let radius = diameter / 2.0;
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::Invalid("ring rail radius"));
        }
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Curve(Curve::Circle {
                center: [0.0, 0.0, 0.0],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius,
                domain: [0.0, std::f64::consts::TAU],
                angle_domain: None,
            }),
        }])
    }
}

/// `RailProfile`: a closed section on the plane normal to the rail at its start.
///
/// `width_mm` is across the rail (radial for a ring), `height_mm` along the rail normal.
/// `preset` selects the library shape (`rectangle`, `rounded_rectangle`, `circle`); the
/// section is centered on the rail point and the frame comes from the rail tangent, never
/// from a hard-coded world plane.
pub const OUTSIDE_RAIL_COMMAND: &str = "OutsideRail";
pub const RING_RESIZER_COMMAND: &str = "RingResizer";

/// The closed profile shapes the builder offers. The library is static in the producer and
/// the builder panel lists it; a session can hide one and the next preview refuses to use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfilePreset {
    Rectangle,
    RoundedRectangle,
    Circle,
}

impl ProfilePreset {
    pub const ALL: [Self; 3] = [Self::Rectangle, Self::RoundedRectangle, Self::Circle];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rectangle => "rectangle",
            Self::RoundedRectangle => "rounded_rectangle",
            Self::Circle => "circle",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Rectangle => "Rectangle",
            Self::RoundedRectangle => "Rounded Rectangle",
            Self::Circle => "Circle",
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.name().eq_ignore_ascii_case(name))
    }
}

/// The curve for one preset, built around `center` on the frame `(x, y)`.
pub fn profile_preset_curve(
    preset: ProfilePreset,
    center: Point3,
    x: Point3,
    y: Point3,
    width: f64,
    height: f64,
    corner: f64,
) -> Result<Curve> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(Error::Invalid("profile size"));
    }
    match preset {
        ProfilePreset::Rectangle => {
            let origin = [
                center[0] - x[0] * width / 2.0 - y[0] * height / 2.0,
                center[1] - x[1] * width / 2.0 - y[1] * height / 2.0,
                center[2] - x[2] * width / 2.0 - y[2] * height / 2.0,
            ];
            curve_build::rectangle(origin, x, y, width, height)
        }
        ProfilePreset::Circle => Ok(Curve::Circle {
            center,
            x,
            y,
            radius: width.min(height) / 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        ProfilePreset::RoundedRectangle => rounded_rectangle(center, x, y, width, height, corner),
    }
}

fn rounded_rectangle(
    center: Point3,
    x: Point3,
    y: Point3,
    width: f64,
    height: f64,
    corner: f64,
) -> Result<Curve> {
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let radius = corner.max(0.0).min(half_width.min(half_height) - 1.0e-6);
    if radius <= 0.0 {
        return profile_preset_curve(ProfilePreset::Rectangle, center, x, y, width, height, 0.0);
    }
    let half = [half_width - radius, half_height - radius];
    // Corner centers (u, v) and their start angles, counter-clockwise from +u/+v.
    let corners = [
        (half[0], half[1], 0.0f64),
        (-half[0], half[1], std::f64::consts::FRAC_PI_2),
        (-half[0], -half[1], std::f64::consts::PI),
        (half[0], -half[1], 3.0 * std::f64::consts::FRAC_PI_2),
    ];
    let mut points = Vec::with_capacity(4 * 5);
    for (u, v, start) in corners {
        for step in 0..=4 {
            let angle = start + std::f64::consts::FRAC_PI_2 * step as f64 / 4.0;
            let local_u = u + radius * angle.cos();
            let local_v = v + radius * angle.sin();
            points.push([
                center[0] + x[0] * local_u + y[0] * local_v,
                center[1] + x[1] * local_u + y[1] * local_v,
                center[2] + x[2] * local_u + y[2] * local_v,
            ]);
        }
    }
    points.push(points[0]);
    curve_build::polyline(&points)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RailProfileProducer;

impl FeatureProducer for RailProfileProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        self.produce_with_limits(request, &Limits::default(), &crate::transaction::Cancellation::default())
    }
    fn produce_with_limits(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation) -> Result<Vec<crate::associative::RebuiltOutput>> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let output = single_output(request)?;
        let rail = input_curve(request, "rail")?;
        let width = positive_length(request, "width_mm")?;
        let height = positive_length(request, "height_mm")?;
        let corner = request
            .parameters
            .get("corner_mm")
            .map(|value| value.as_length().map(|length| length.mm()))
            .transpose()?
            .unwrap_or_else(|| width.min(height) * 0.2);
        let preset = match request.parameters.get("preset") {
            None => ProfilePreset::Rectangle,
            Some(Value::Choice(name)) => {
                ProfilePreset::from_name(name).ok_or(Error::Invalid("profile preset"))?
            }
            Some(_) => return Err(Error::Invalid("profile preset")),
        };
        let position = ratio_parameter(request, "position")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(Error::Invalid("profile position 0..1"));
        }
        let roll = angle_parameter(request, "roll_deg")?;
        let point = curve_ops::point_at(rail, position, &limits)?;
        let tangent = curve_ops::tangent_at(rail, position, &limits)?;
        let reference = if cross(tangent, [0.0, 0.0, 1.0])
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            > 1.0e-12
        {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let x = unit(cross(tangent, reference))?;
        let y = unit(cross(tangent, x))?;
        // Roll rotates the section in its own plane. The tangent is the rotation axis, so the
        // profile stays normal to the rail and only its seam/clocking changes.
        let (sin, cos) = roll.to_radians().sin_cos();
        let x = unit([
            x[0] * cos + y[0] * sin,
            x[1] * cos + y[1] * sin,
            x[2] * cos + y[2] * sin,
        ])?;
        let y = unit(cross(tangent, x))?;
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Curve(profile_preset_curve(
                preset, point, x, y, width, height, corner,
            )?),
        }])
    }
}

/// `OutsideRail`: a concentric outer rail at `offset_mm` outside a closed circular rail.
///
/// The input rail must be a closed circle: offsetting an arbitrary rail is not the same
/// geometry and would silently change the ring's inside surface. A non-positive offset is
/// refused instead of producing a collapsed circle.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutsideRailProducer;

impl FeatureProducer for OutsideRailProducer {
    fn produce_with_limits(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation) -> Result<Vec<crate::associative::RebuiltOutput>> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let outputs = self.produce(request)?;
        for output in &outputs { output.geometry.validate(limits)?; }
        Ok(outputs)
    }
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        let output = single_output(request)?;
        let rail = input_curve(request, "rail")?;
        let offset = length_parameter(request, "offset_mm")?;
        if !offset.is_finite() || offset <= 0.0 {
            return Err(Error::Invalid("outside rail offset"));
        }
        let Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } = rail
        else {
            return Err(Error::Unsupported("outside rail non-circular rail"));
        };
        let outer = radius + offset;
        if !outer.is_finite() || outer <= 0.0 {
            return Err(Error::Invalid("outside rail radius"));
        }
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Curve(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: outer,
                domain: *domain,
                angle_domain: *angle_domain,
            }),
        }])
    }
}

/// `RingResizer`: an independent ring at `target_diameter_mm`, leaving the source ring
/// untouched. The inside diameter must leave material: it must exceed twice `thickness_mm`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RingResizerProducer;
impl FeatureProducer for RingResizerProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        let output = single_output(request)?;
        let ring = input_curve(request, "ring")?;
        let target = positive_length(request, "target_diameter_mm")?;
        let thickness = request
            .parameters
            .get("thickness_mm")
            .map(|value| value.as_length().map(|length| length.mm()))
            .transpose()?
            .unwrap_or(1.5);
        if !thickness.is_finite() || thickness <= 0.0 {
            return Err(Error::Invalid("ring resizer thickness"));
        }
        if target <= 2.0 * thickness {
            return Err(Error::Invalid("ring resizer target below thickness"));
        }
        let Curve::Circle {
            center,
            x,
            y,
            domain,
            angle_domain,
            ..
        } = ring
        else {
            return Err(Error::Unsupported("ring resizer non-circular ring"));
        };
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Curve(Curve::Circle {
                center: *center,
                x: *x,
                y: *y,
                radius: target / 2.0,
                domain: *domain,
                angle_domain: *angle_domain,
            }),
        }])
    }
}

/// One jewelry ring builder of `OM9-L2-04-03` (`EternityBand`, `SignetRing`, `RaisedBand`,
/// `AwardRing`). It reads the same typed parameters the panel edits and returns the family's
/// watertight mesh solid; stone placement stays a frame list, never a fabricated gem solid.
#[derive(Debug, Clone, Copy)]
pub struct RingFamilyProducer {
    pub kind: crate::ring_builder::RingBuilderKind,
}

impl FeatureProducer for RingFamilyProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        let output = single_output(request)?;
        let params = crate::ring_builder::RingParams::from_request(self.kind, request)?;
        let solid = crate::ring_builder::build(&params)?;
        Ok(vec![crate::associative::RebuiltOutput {
            object: output,
            geometry: Geometry::Mesh(solid.mesh),
        }])
    }
}

/// `RailSweep`: sweep the `profile` input along the `rail` input through a backend.
#[derive(Debug, Clone)]
pub struct RailSweepProducer<B: SweepBackend> {
    backend: B,
}
impl<B: SweepBackend> RailSweepProducer<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: SweepBackend> FeatureProducer for RailSweepProducer<B> {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<crate::associative::RebuiltOutput>> {
        self.produce_with_candidates(request).map(|(outputs, _)| outputs)
    }
    fn produce_with_candidates(&self, request: &RebuildRequest)
        -> Result<(Vec<crate::associative::RebuiltOutput>, Vec<crate::brep::Brep>)> {
        self.produce_limited_with_candidates(request, &Limits::default(), &crate::transaction::Cancellation::default())
    }
    fn produce_limited_with_candidates(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation)
        -> Result<(Vec<crate::associative::RebuiltOutput>, Vec<crate::brep::Brep>)> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let output = single_output(request)?;
        let rail = input_curve(request, "rail")?;
        let profile = input_curve(request, "profile")?;
        let solid = boolean_parameter(request, "solid")?;
        // A builder-created ring sweep opts in: the rail must be a closed planar ring rail.
        // Older or generic sweep recipes leave the flag absent and keep their previous
        // open-rail behavior instead of failing to replay.
        if boolean_parameter(request, "closed_rail")? {
            crate::curve_ops::validate_ring_rail(rail, 1.0e-6, limits)?;
        }
        let (geometry, candidates) = self
            .backend
            .sweep_with_candidates(rail, profile, solid, limits)?;
        // Native calls can finish after cancellation; their private result must
        // still be discarded before topology or publication.
        if cancel.is_canceled() { return Err(Error::Canceled); }
        Ok((vec![crate::associative::RebuiltOutput {
            object: output,
            geometry,
        }], candidates))
    }
}

/// Names the subshapes of a produced geometry. A B-Rep uses its snapshot topology (from the
/// OCCT bridge or the native mesh); a curve publishes one `curve:0`; a surface one `face:0`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SnapshotTopology;

impl SnapshotTopology {
    pub fn snapshot_subshapes(
        &self,
        geometry: &Geometry,
        limits: &Limits,
    ) -> Result<Vec<Subshape>> {
        match geometry {
            Geometry::Brep(_) => Err(Error::Unsupported("canonical BRep topology requires source evaluator")),
            Geometry::NativeBrep(_) => Err(Error::Unsupported("native BRep topology requires source evaluator")),
            Geometry::Mesh(mesh) => self.mesh_subshapes(mesh),
            Geometry::Curve(curve) => {
                let points = curve_ops::sample_curve(curve, 33, limits)?;
                let start = points[0];
                let end = points[points.len() - 1];
                Ok(vec![Subshape {
                    name: "curve:0".into(),
                    kind: SubshapeKind::Curve,
                    signature: SubshapeSignature {
                        start,
                        end,
                        measure_mm: polyline_length(&points),
                    },
                }])
            }
            Geometry::Surface(surface) => {
                let first = surface.poles.first().copied().unwrap_or([0.0; 3]);
                Ok(vec![Subshape {
                    name: "face:0".into(),
                    kind: SubshapeKind::Face,
                    signature: SubshapeSignature {
                        start: first,
                        end: first,
                        measure_mm: 0.0,
                    },
                }])
            }
            Geometry::Point(point) => Ok(vec![Subshape {
                name: "vertex:0".into(),
                kind: SubshapeKind::Vertex,
                signature: SubshapeSignature {
                    start: *point,
                    end: *point,
                    measure_mm: 0.0,
                },
            }]),
            _ => Ok(vec![]),
        }
    }

    /// Temporary display-derived naming policy. `admit` covers this method's output
    /// allocations; the caller must separately cover the live BRep and any envelopes.
    pub fn brep_subshapes_admitted(&self, display: &crate::brep::BrepDisplay,
        limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<Subshape>> {
        let count = display.edges.len().checked_add(display.faces.len())
            .and_then(|n| n.checked_add(display.vertices.len())).ok_or(Error::Overflow("BRep subshape count"))?;
        let planned = Usage::array::<Subshape>(count)?;
        planned.check(limits)?;
        admit(0, planned.bytes)?;
        let mut subshapes = Vec::new();
        subshapes.try_reserve_exact(count).map_err(|_| Error::Budget("BRep subshape table"))?;
        let mut live = Usage::array::<Subshape>(subshapes.capacity())?.bytes;
        Usage { bytes: live, ..Usage::default() }.check(limits)?;
        admit(live, 0)?;
        for (index, _) in display.edges.iter().enumerate() {
            let mut segments = display
                .segments
                .iter()
                .filter(|segment| segment[2] as usize == index);
            let Some(first) = segments.next() else {
                continue;
            };
            let start = display.points[first[0] as usize];
            let mut end = display.points[first[1] as usize];
            let mut measure = 0.0;
            for segment in std::iter::once(first).chain(segments) {
                let a = display.points[segment[0] as usize];
                let b = display.points[segment[1] as usize];
                end = b;
                measure +=
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            }
            let name = admitted_subshape_name("edge:", index, &mut live, limits, admit)?;
            subshapes.push(Subshape {
                name,
                kind: SubshapeKind::Edge,
                signature: SubshapeSignature {
                    start,
                    end,
                    measure_mm: measure,
                },
            });
        }
        for (index, _) in display.faces.iter().enumerate() {
            let mut centroid = [0.0; 3];
            let mut count = 0usize;
            let mut area = 0.0;
            for triangle in display
                .triangles
                .iter()
                .filter(|triangle| triangle[3] as usize == index)
            {
                let a = display.points[triangle[0] as usize];
                let b = display.points[triangle[1] as usize];
                let c = display.points[triangle[2] as usize];
                for point in [a, b, c] {
                    centroid[0] += point[0];
                    centroid[1] += point[1];
                    centroid[2] += point[2];
                    count += 1;
                }
                let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let normal = cross(ab, ac);
                area += 0.5
                    * (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                        .sqrt();
            }
            if count == 0 {
                continue;
            }
            let center = [
                centroid[0] / count as f64,
                centroid[1] / count as f64,
                centroid[2] / count as f64,
            ];
            let name = admitted_subshape_name("face:", index, &mut live, limits, admit)?;
            subshapes.push(Subshape {
                name,
                kind: SubshapeKind::Face,
                signature: SubshapeSignature {
                    start: center,
                    end: center,
                    measure_mm: area,
                },
            });
        }
        for (index, offset) in display.vertices.iter().enumerate() {
            let point = display.points[*offset as usize];
            let name = admitted_subshape_name("vertex:", index, &mut live, limits, admit)?;
            subshapes.push(Subshape {
                name,
                kind: SubshapeKind::Vertex,
                signature: SubshapeSignature {
                    start: point,
                    end: point,
                    measure_mm: 0.0,
                },
            });
        }
        Ok(subshapes)
    }

    fn mesh_subshapes(&self, mesh: &crate::geometry::Mesh) -> Result<Vec<Subshape>> {
        let mut subshapes = Vec::new();
        for (index, face) in mesh.faces.iter().enumerate() {
            let mut centroid = [0.0; 3];
            for vertex in face {
                let point = mesh.vertices[*vertex as usize];
                centroid[0] += point[0];
                centroid[1] += point[1];
                centroid[2] += point[2];
            }
            let count = face.len().max(1) as f64;
            let center = [
                centroid[0] / count,
                centroid[1] / count,
                centroid[2] / count,
            ];
            subshapes.push(Subshape {
                name: format!("face:{index}"),
                kind: SubshapeKind::Face,
                signature: SubshapeSignature {
                    start: center,
                    end: center,
                    measure_mm: 0.0,
                },
            });
        }
        Ok(subshapes)
    }
}

fn admitted_subshape_name(prefix: &str, index: usize, live: &mut u64, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<String> {
    use std::fmt::Write;
    let digits = index.checked_ilog10().unwrap_or(0) as usize + 1;
    let capacity = prefix.len().checked_add(digits).ok_or(Error::Overflow("BRep subshape name"))?;
    let pending = u64::try_from(capacity).map_err(|_| Error::Overflow("BRep subshape name"))?;
    let required = live.checked_add(pending).ok_or(Error::Overflow("BRep subshape names"))?;
    Usage { bytes: required, ..Usage::default() }.check(limits)?;
    admit(*live, pending)?;
    let mut name = String::new();
    name.try_reserve_exact(capacity).map_err(|_| Error::Budget("BRep subshape name"))?;
    *live = live.checked_add(name.capacity() as u64).ok_or(Error::Overflow("BRep subshape names"))?;
    Usage { bytes: *live, ..Usage::default() }.check(limits)?;
    admit(*live, 0)?;
    write!(&mut name, "{prefix}{index}").map_err(|_| Error::Invalid("BRep subshape name"))?;
    Ok(name)
}

fn polyline_length(points: &[Point3]) -> f64 {
    points
        .windows(2)
        .map(|pair| {
            ((pair[1][0] - pair[0][0]).powi(2)
                + (pair[1][1] - pair[0][1]).powi(2)
                + (pair[1][2] - pair[0][2]).powi(2))
            .sqrt()
        })
        .sum()
}

/// Selects the topology policy used for produced geometry.
pub trait TopologyProvider {
    fn subshapes(&self, geometry: &Geometry, limits: &Limits) -> Result<Vec<Subshape>>;
}

impl TopologyProvider for SnapshotTopology {
    fn subshapes(&self, geometry: &Geometry, limits: &Limits) -> Result<Vec<Subshape>> {
        self.snapshot_subshapes(geometry, limits)
    }
}

/// The concrete command -> producer table plus the topology policy. A host registers the
/// kernel backend once and then asks for any recorded feature by its command name.
pub struct ProducerRegistry {
    producers: BTreeMap<String, Box<dyn FeatureProducer>>,
    topology: Box<dyn TopologyProvider>,
}

impl Default for ProducerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProducerRegistry {
    pub fn new() -> Self {
        Self {
            producers: BTreeMap::new(),
            topology: Box::new(SnapshotTopology),
        }
    }
    pub fn with_topology(topology: impl TopologyProvider + 'static) -> Self {
        Self {
            producers: BTreeMap::new(),
            topology: Box::new(topology),
        }
    }
    pub fn register(
        &mut self,
        command: impl Into<String>,
        producer: impl FeatureProducer + 'static,
    ) -> Result<()> {
        let command = command.into();
        if command.is_empty() {
            return Err(Error::Invalid("producer command"));
        }
        if self.producers.contains_key(&command) {
            return Err(Error::Invalid("duplicate producer command"));
        }
        self.producers.insert(command, Box::new(producer));
        Ok(())
    }
    pub fn contains(&self, command: &str) -> bool {
        self.producers.contains_key(command)
    }
    pub fn commands(&self) -> impl Iterator<Item = &str> {
        self.producers.keys().map(String::as_str)
    }
    /// Produce geometry for one request and name its subshapes with the registry's policy.
    pub fn produce(&self, request: &RebuildRequest) -> Result<ProducedFeature> {
        let producer = self
            .producers
            .get(&request.command)
            .ok_or(Error::Unsupported("history producer"))?;
        let (outputs, brep_candidates) = producer.produce_with_candidates(request)?;
        let limits = Limits::default();
        let mut subshapes = Vec::with_capacity(outputs.len());
        for output in &outputs {
            subshapes.push(ProducedSubshapes {
                output: output.object,
                subshapes: self.topology.subshapes(&output.geometry, &limits)?,
            });
        }
        Ok(ProducedFeature { outputs, subshapes, brep_candidates })
    }

    /// Candidate recompute refuses producers that do not accept caller limits.
    fn produce_with_limits(&self, request: &RebuildRequest, limits: &Limits,
        cancel: &crate::transaction::Cancellation) -> Result<ProducedFeature> {
        let producer = self.producers.get(&request.command)
            .ok_or(Error::Unsupported("history producer"))?;
        let (outputs, brep_candidates) = producer.produce_limited_with_candidates(request, limits, cancel)?;
        let mut subshapes = Vec::new();
        subshapes.try_reserve_exact(outputs.len()).map_err(|_| Error::Budget("history subshapes"))?;
        for output in &outputs {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            subshapes.push(ProducedSubshapes {
                output: output.object,
                subshapes: self.topology.subshapes(&output.geometry, limits)?,
            });
        }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        Ok(ProducedFeature { outputs, subshapes, brep_candidates })
    }
}

/// The reference producer set for the builder chain, using the analytic sweep backend.
pub fn reference_registry() -> Result<ProducerRegistry> {
    let mut registry = ProducerRegistry::new();
    registry.register(JOIN_HISTORY_COMMAND, JoinHistoryProducer)?;
    registry.register(RING_RAIL_COMMAND, RingRailProducer)?;
    registry.register(RAIL_PROFILE_COMMAND, RailProfileProducer)?;
    registry.register(
        RAIL_SWEEP_COMMAND,
        RailSweepProducer::new(SurfaceSweepBackend),
    )?;
    registry.register(OUTSIDE_RAIL_COMMAND, OutsideRailProducer)?;
    registry.register(RING_RESIZER_COMMAND, RingResizerProducer)?;
    for family in crate::ring_builder::RingBuilderKind::ALL {
        registry.register(family.command(), RingFamilyProducer { kind: family })?;
    }
    Ok(registry)
}

/// Recompute one recorded feature through the registry. The producer's subshapes are checked
/// against the feature's semantic references before the rebuild is committed.
pub fn recompute_registered(
    document: &mut Document,
    cache: &mut RecomputeCache,
    id: HistoryId,
    registry: &ProducerRegistry,
) -> Result<RecomputeOutcome> {
    let command = document.history.recipes.get(&id).ok_or(Error::Missing("history recipe"))?.command.clone();
    if !registry.contains(&command) { return Err(Error::Unsupported("history producer")); }
    crate::associative::recompute_with(document, cache, id, None, |request| registry.produce(request))
}

/// Rebuild affected dependents on a private candidate, in dependency order.
///
/// The caller owns the candidate and cache and must discard both on an error, a
/// cancellation, or a returned `Failed` outcome. This function does not publish
/// or create an Undo entry. The caller must account for the candidate and producer
/// allocations separately. Producers without an implementation that accepts these
/// limits are refused; native BRep representation handoff is not supported here.
pub fn recompute_dependents(
    document: &mut Document,
    cache: &mut RecomputeCache,
    changed: &BTreeSet<ObjectId>,
    registry: &ProducerRegistry,
    limits: &Limits,
    cancel: &crate::transaction::Cancellation,
) -> Result<Vec<RecomputeOutcome>> {
    recompute_dependents_for(document, cache, changed, registry, limits, cancel, None, &mut None, &mut None)
}

fn recompute_dependents_for(
    document: &mut Document,
    cache: &mut RecomputeCache,
    changed: &BTreeSet<ObjectId>,
    registry: &ProducerRegistry,
    limits: &Limits,
    cancel: &crate::transaction::Cancellation,
    only_command: Option<&str>,
    admit_snapshot: &mut Option<&mut dyn FnMut(&Document, &crate::history::Recipe) -> Result<()>>,
    candidates: &mut Option<&mut Vec<crate::brep::Brep>>,
) -> Result<Vec<RecomputeOutcome>> {
    if cancel.is_canceled() {
        return Err(Error::Canceled);
    }
    if changed.is_empty() {
        return Ok(Vec::new());
    }
    let output_count = document.history.recipes.values().try_fold(changed.len(), |count, recipe| {
        count.checked_add(recipe.outputs.len()).ok_or(Error::Overflow("dependent history outputs"))
    })?;
    let mut held = Usage::btree::<ObjectId, ()>(output_count)?;
    held.add(Usage::array::<RecomputeOutcome>(document.history.recipes.len())?, limits)?;
    let order = document.history.execution_order_bounded(limits, held)?;
    let mut affected = changed.clone();
    let mut outcomes = Vec::new();
    outcomes.try_reserve_exact(order.len()).map_err(|_| Error::Budget("dependent history outcomes"))?;
    for id in order {
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        let recipe = document.history.recipes.get(&id).ok_or(Error::Missing("history recipe"))?;
        if recipe.state == crate::history::EvaluationState::Detached
            || !recipe.inputs.iter().any(|input| affected.contains(&input.source.object))
            || !recipe.policy.update
        {
            continue;
        }
        if recipe.policy.lock {
            return Err(Error::Permission("frozen history"));
        }
        if only_command.is_some_and(|command| recipe.command != command) {
            return Err(Error::Unsupported("dependent history producer handoff"));
        }
        if let Some(admit) = admit_snapshot.as_mut() { admit(document, recipe)?; }
        if only_command == Some(JOIN_HISTORY_COMMAND) {
            let mut inputs = BTreeSet::new();
            for input in &recipe.inputs {
                if !inputs.insert(input.source.object) {
                    return Err(Error::Invalid("duplicate join history input"));
                }
            }
        }
        let producer_limits = limits.remaining_after(held)?;
        let mut produced_candidates = Vec::new();
        let outcome = crate::associative::recompute_with(document, cache, id,
            candidates.as_ref().map(|_| &mut produced_candidates),
            |request| registry.produce_with_limits(request, &producer_limits, cancel))?;
        if let Some(candidates) = candidates.as_mut() {
            candidates.try_reserve(produced_candidates.len()).map_err(|_| Error::Budget("dependent BRep candidates"))?;
            candidates.extend(produced_candidates);
        }
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        if let RecomputeOutcome::Rebuilt { outputs, .. } = &outcome {
            affected.extend(outputs.iter().copied());
        }
        let failed = matches!(outcome, RecomputeOutcome::Failed { .. });
        outcomes.push(outcome);
        if failed {
            break;
        }
    }
    Ok(outcomes)
}

/// Assemble Join History updates for a source edit already applied to a private
/// candidate. The caller owns and accounts for the candidate, inputs and returned
/// mutations, and must discard the candidate on any error or cancellation. The
/// caller appends these mutations to the original source edit and prepares one
/// transaction; this function never publishes or writes an Undo entry. The
/// candidate must come from preparing precisely that source edit against `before`;
/// the identity, revision and source-stamp checks do not prove its provenance.
pub fn join_history_updates_from_candidate(
    before: &Document,
    candidate: &mut Document,
    changed: &BTreeSet<ObjectId>,
    registry: &ProducerRegistry,
    limits: &Limits,
    cancel: &crate::transaction::Cancellation,
) -> Result<Vec<crate::transaction::Mutation>> {
    join_history_updates_with_admission(before, candidate, changed, registry, limits, cancel, None)
}

pub(crate) fn join_history_updates_with_admission(
    before: &Document, candidate: &mut Document, changed: &BTreeSet<ObjectId>,
    registry: &ProducerRegistry, limits: &Limits, cancel: &crate::transaction::Cancellation,
    admit_snapshot: Option<&mut dyn FnMut(&Document, &crate::history::Recipe) -> Result<()>>,
) -> Result<Vec<crate::transaction::Mutation>> {
    dependent_history_updates_with_admission(before, candidate, changed, registry, limits, cancel,
        Some(JOIN_HISTORY_COMMAND), admit_snapshot, None)
}

/// Rebuild an admitted private candidate in dependency order, retaining native
/// representations beside their canonical sources until the host stages them.
pub(crate) fn dependent_history_updates_with_admission(
    before: &Document,
    candidate: &mut Document,
    changed: &BTreeSet<ObjectId>,
    registry: &ProducerRegistry,
    limits: &Limits,
    cancel: &crate::transaction::Cancellation,
    only_command: Option<&str>,
    mut admit_snapshot: Option<&mut dyn FnMut(&Document, &crate::history::Recipe) -> Result<()>>,
    mut candidates: Option<&mut Vec<crate::brep::Brep>>,
) -> Result<Vec<crate::transaction::Mutation>> {
    use crate::{associative::RecomputeOutcome, transaction::Mutation};
    if before.id != candidate.id || before.owner_session != candidate.owner_session
        || before.epoch != candidate.epoch || before.session_generation != candidate.session_generation
        || before.revision.checked_add(1) != Some(candidate.revision)
    {
        return Err(Error::Stale("dependent history candidate"));
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    for id in changed {
        if before.stamp(*id)? == candidate.stamp(*id)? {
            return Err(Error::Invalid("unchanged dependent history source"));
        }
    }
    let mut cache = RecomputeCache::default();
    let outcomes = recompute_dependents_for(candidate, &mut cache, changed, registry, limits, cancel,
        only_command, &mut admit_snapshot, &mut candidates)?;
    let mut recipes = Vec::new();
    let mut published = Vec::new();
    let mut updates = Vec::new();
    let mut seen_outputs = BTreeSet::new();
    for outcome in outcomes {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        match outcome {
            RecomputeOutcome::Rebuilt { recipe, outputs, .. } => {
                let record = candidate.history.recipes.get(&recipe).ok_or(Error::Missing("dependent history recipe"))?;
                if only_command.is_some_and(|command| record.command != command) {
                    return Err(Error::Unsupported("dependent history producer handoff"));
                }
                if let Some(admit) = admit_snapshot.as_mut() { admit(candidate, record)?; }
                recipes.try_reserve(1).map_err(|_| Error::Budget("dependent history recipes"))?;
                recipes.push(Box::new(record.clone()));
                for output in outputs {
                    if changed.contains(&output) || !seen_outputs.insert(output) {
                        return Err(Error::Invalid("dependent history output conflict"));
                    }
                    if seen_outputs.len() as u64 > limits.max_outputs { return Err(Error::Budget("dependent history outputs")); }
                    let entity = candidate.entities.get(&output).ok_or(Error::Missing("dependent history output"))?;
                    if only_command == Some(JOIN_HISTORY_COMMAND) && !matches!(entity.geometry, Geometry::Curve(_)) {
                        return Err(Error::Unsupported("dependent history output geometry"));
                    }
                    let result = candidate.results.get(&output).ok_or(Error::Missing("dependent history result"))?;
                    updates.try_reserve(1).map_err(|_| Error::Budget("dependent history mutations"))?;
                    published.try_reserve(1).map_err(|_| Error::Budget("dependent history results"))?;
                    updates.push(Mutation::Geometry { source: before.stamp(output)?, geometry: entity.geometry.clone() });
                    published.push((output, result.clone()));
                }
            }
            RecomputeOutcome::Failed { reason, .. } => return Err(Error::Io(format!("dependent history rebuild: {reason}"))),
            RecomputeOutcome::Cached { .. } => return Err(Error::Invalid("unexpected dependent history cache")),
        }
    }
    if !recipes.is_empty() {
        updates.try_reserve(1).map_err(|_| Error::Budget("dependent history mutations"))?;
        updates.push(Mutation::HistoryUpdate { document: before.id, expected_revision: before.revision, recipes, published });
    }
    Ok(updates)
}

/// Retain transient B-reps until the host has handed their displays to the
/// representation store. Cached and failed rebuilds produce no candidates.
pub fn recompute_registered_with_candidates(
    document: &mut Document, cache: &mut RecomputeCache, id: HistoryId,
    registry: &ProducerRegistry, candidates: &mut Vec<crate::brep::Brep>,
) -> Result<RecomputeOutcome> {
    candidates.clear();
    let command = document
        .history
        .recipes
        .get(&id)
        .ok_or(Error::Missing("history recipe"))?
        .command
        .clone();
    if !registry.contains(&command) {
        return Err(Error::Unsupported("history producer"));
    }
    crate::associative::recompute_with(document, cache, id, Some(candidates), |request| registry.produce(request))
}

/// The output geometry kind a producer is expected to return for a command, so a catalog or
/// the builder UI can declare an output port without running the producer.
pub fn expected_output_kind(command: &str) -> Option<GeometryKind> {
    match command {
        JOIN_HISTORY_COMMAND | RING_RAIL_COMMAND | RAIL_PROFILE_COMMAND | OUTSIDE_RAIL_COMMAND | RING_RESIZER_COMMAND => {
            Some(GeometryKind::Curve)
        }
        crate::ring_builder::ETERNITY_BAND_COMMAND
        | crate::ring_builder::SIGNET_RING_COMMAND
        | crate::ring_builder::RAISED_BAND_COMMAND
        | crate::ring_builder::AWARD_RING_COMMAND => Some(GeometryKind::Mesh),
        RAIL_SWEEP_COMMAND => None,
        _ => None,
    }
}
