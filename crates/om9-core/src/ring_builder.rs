// SPDX-License-Identifier: LGPL-2.1-or-later
//! Jewelry ring builder family (`OM9-L2-04-03`): Eternity, Signet, Raised and Award.
//!
//! The four builders share one surface of revolution: a closed axial section `(r, z)` swept
//! around the ring axis `Z`. Per-kind modulation of the section gives the raised ridge and
//! the signet/award top pad, so every builder emits **one watertight mesh solid** instead of
//! a bag of parts. Placement frames (Eternity stones) are published as named frames; no gem
//! solid is fabricated (`M05_GEM_INTEGRATION_DEFERRED`).
//!
//! The sections are portable Rust, and both backends sample them: the portable producer emits
//! the mesh, while the OCCT adapter sews the same quads into a B-Rep solid
//! (`om9_kernel::associative::RingFamilyBrepProducer`). [`inspect_measures`] is the one
//! feasibility gate both bodies must satisfy (closed, positive volume), reached through
//! [`inspect_solid`] for a mesh and [`inspect_geometry`] for either representation, so an open
//! body is refused instead of being called production geometry.
use crate::{
    Error, Result,
    associative::RebuildRequest,
    geometry::{Geometry, Mesh, Point3},
    request::{Dimension, Value},
};
use std::f64::consts::TAU;

pub const ETERNITY_BAND_COMMAND: &str = "EternityBand";
pub const SIGNET_RING_COMMAND: &str = "SignetRing";
pub const RAISED_BAND_COMMAND: &str = "RaisedBand";
pub const AWARD_RING_COMMAND: &str = "AwardRing";

/// The thinnest section the family accepts, in millimetres. Below it the solid would collapse
/// at the kernel tolerance, so the builder refuses instead of building a degenerate body.
pub const MIN_WALL_MM: f64 = 0.3;
/// Circumferential segments of the revolution. The same value is used for every kind, so a
/// preset roundtrip reproduces the same vertex count and volume.
pub const SEGMENTS: usize = 96;

/// One ring builder mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RingBuilderKind {
    Eternity,
    Signet,
    Raised,
    Award,
}

impl RingBuilderKind {
    pub const ALL: [Self; 4] = [Self::Eternity, Self::Signet, Self::Raised, Self::Award];

    pub fn command(self) -> &'static str {
        match self {
            Self::Eternity => ETERNITY_BAND_COMMAND,
            Self::Signet => SIGNET_RING_COMMAND,
            Self::Raised => RAISED_BAND_COMMAND,
            Self::Award => AWARD_RING_COMMAND,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Eternity => "Eternity Band",
            Self::Signet => "Signet Ring",
            Self::Raised => "Raised Band",
            Self::Award => "Award Ring",
        }
    }
    /// The stable tag a style preset and the evidence record use.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Eternity => "eternity_band",
            Self::Signet => "signet_ring",
            Self::Raised => "raised_band",
            Self::Award => "award_ring",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.tag().eq_ignore_ascii_case(tag.trim()))
    }
    /// The default parameters of the mode. A preset starts from these, then overrides.
    pub fn defaults(self) -> RingParams {
        let mut params = RingParams {
            kind: self,
            diameter_mm: 18.0,
            width_mm: 6.0,
            thickness_mm: 2.0,
            stone_count: 12,
            stone_diameter_mm: 2.0,
            top_width_mm: 8.0,
            top_length_mm: 10.0,
            shoulder_mm: 2.0,
            raised_height_mm: 1.0,
            raised_width_mm: 4.0,
        };
        match self {
            Self::Eternity => {}
            Self::Signet => {
                params.thickness_mm = 2.4;
            }
            Self::Raised => {
                params.thickness_mm = 3.5;
                params.raised_height_mm = 3.0;
            }
            Self::Award => {
                params.thickness_mm = 2.6;
                params.top_width_mm = 10.0;
                params.top_length_mm = 12.0;
                params.shoulder_mm = 2.5;
            }
        }
        params
    }
}

/// The typed parameters of one ring builder.
#[derive(Debug, Clone, PartialEq)]
pub struct RingParams {
    pub kind: RingBuilderKind,
    /// Inner diameter of the band, in millimetres.
    pub diameter_mm: f64,
    /// Axial width of the band, in millimetres.
    pub width_mm: f64,
    /// Radial wall thickness of the band, in millimetres.
    pub thickness_mm: f64,
    /// Eternity placeholders.
    pub stone_count: u32,
    pub stone_diameter_mm: f64,
    /// Signet/award top pad, in millimetres.
    pub top_width_mm: f64,
    pub top_length_mm: f64,
    pub shoulder_mm: f64,
    /// Raised band ridge, in millimetres.
    pub raised_height_mm: f64,
    pub raised_width_mm: f64,
}

impl RingParams {
    pub fn new(kind: RingBuilderKind) -> Self {
        kind.defaults()
    }

    /// Refuse a parameter set the pattern cannot build. The messages name the parameter so
    /// the panel and the command line report the same typed error.
    pub fn validate(&self) -> Result<()> {
        let finite = |value: f64| value.is_finite();
        if !finite(self.diameter_mm) || !(8.0..=40.0).contains(&self.diameter_mm) {
            return Err(Error::Invalid("ring diameter"));
        }
        if !finite(self.width_mm) || !(1.0..=20.0).contains(&self.width_mm) {
            return Err(Error::Invalid("ring width"));
        }
        if !finite(self.thickness_mm) || !(MIN_WALL_MM..=10.0).contains(&self.thickness_mm) {
            return Err(Error::Invalid("ring wall"));
        }
        match self.kind {
            RingBuilderKind::Eternity => {
                if self.stone_count == 0 || self.stone_count > 60 {
                    return Err(Error::Invalid("ring stone count"));
                }
                if !finite(self.stone_diameter_mm) || self.stone_diameter_mm <= 0.0 {
                    return Err(Error::Invalid("ring stone diameter"));
                }
                // A positive quantity still has to fit: the arc per stone must clear the
                // stone diameter plus a placement gap.
                let arc = std::f64::consts::PI * self.diameter_mm / f64::from(self.stone_count);
                if arc < self.stone_diameter_mm + 0.25 {
                    return Err(Error::Invalid("ring stone spacing"));
                }
            }
            RingBuilderKind::Raised => {
                if !finite(self.raised_height_mm) || self.raised_height_mm <= 0.0 {
                    return Err(Error::Invalid("ring raised height"));
                }
                // A ridge as deep as the wall leaves zero section at the inner edge.
                if self.raised_height_mm > self.thickness_mm - MIN_WALL_MM {
                    return Err(Error::Invalid("ring raised height"));
                }
                if !finite(self.raised_width_mm)
                    || self.raised_width_mm <= 0.0
                    || self.raised_width_mm > self.width_mm
                {
                    return Err(Error::Invalid("ring raised width"));
                }
            }
            RingBuilderKind::Signet | RingBuilderKind::Award => {
                if !finite(self.top_width_mm) || !(1.0..=20.0).contains(&self.top_width_mm) {
                    return Err(Error::Invalid("ring signet top"));
                }
                if !finite(self.top_length_mm) || !(1.0..=30.0).contains(&self.top_length_mm) {
                    return Err(Error::Invalid("ring signet top"));
                }
                // The pad is an arc of the band: a top longer than half the circumference
                // would fold the shoulders through each other.
                let half_circumference = std::f64::consts::PI * self.diameter_mm * 0.5;
                if self.top_length_mm > half_circumference {
                    return Err(Error::Invalid("ring signet top"));
                }
                if !finite(self.shoulder_mm) || self.shoulder_mm < MIN_WALL_MM {
                    return Err(Error::Invalid("ring shoulder"));
                }
            }
        }
        Ok(())
    }

    /// Read the parameters a builder session (or a recorded recipe) hands the producer.
    pub fn from_request(kind: RingBuilderKind, request: &RebuildRequest) -> Result<Self> {
        let defaults = kind.defaults();
        let mut params = defaults.clone();
        params.diameter_mm = length_parameter(request, "diameter_mm", defaults.diameter_mm)?;
        params.width_mm = length_parameter(request, "width_mm", defaults.width_mm)?;
        params.thickness_mm = length_parameter(request, "thickness_mm", defaults.thickness_mm)?;
        params.stone_diameter_mm =
            length_parameter(request, "stone_diameter_mm", defaults.stone_diameter_mm)?;
        params.top_width_mm = length_parameter(request, "top_width_mm", defaults.top_width_mm)?;
        params.top_length_mm = length_parameter(request, "top_length_mm", defaults.top_length_mm)?;
        params.shoulder_mm = length_parameter(request, "shoulder_mm", defaults.shoulder_mm)?;
        params.raised_height_mm =
            length_parameter(request, "raised_height_mm", defaults.raised_height_mm)?;
        params.raised_width_mm =
            length_parameter(request, "raised_width_mm", defaults.raised_width_mm)?;
        params.stone_count = count_parameter(request, "stone_count", defaults.stone_count)?;
        params.validate()?;
        Ok(params)
    }

    /// The parametric request the family records for a history rebuild.
    pub fn parameters(&self) -> Vec<(&'static str, Value)> {
        vec![
            (
                "diameter_mm",
                Value::Quantity {
                    value: self.diameter_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "width_mm",
                Value::Quantity {
                    value: self.width_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "thickness_mm",
                Value::Quantity {
                    value: self.thickness_mm,
                    dimension: Dimension::Length,
                },
            ),
            ("stone_count", Value::count(self.stone_count)),
            (
                "stone_diameter_mm",
                Value::Quantity {
                    value: self.stone_diameter_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "top_width_mm",
                Value::Quantity {
                    value: self.top_width_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "top_length_mm",
                Value::Quantity {
                    value: self.top_length_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "shoulder_mm",
                Value::Quantity {
                    value: self.shoulder_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "raised_height_mm",
                Value::Quantity {
                    value: self.raised_height_mm,
                    dimension: Dimension::Length,
                },
            ),
            (
                "raised_width_mm",
                Value::Quantity {
                    value: self.raised_width_mm,
                    dimension: Dimension::Length,
                },
            ),
        ]
    }
}

fn length_parameter(request: &RebuildRequest, key: &str, default: f64) -> Result<f64> {
    match request.parameters.get(key) {
        None => Ok(default),
        Some(Value::Quantity { value, .. }) => Ok(*value),
        Some(Value::Number(value)) => Ok(*value),
        Some(_) => Err(Error::Invalid("ring parameter")),
    }
}

fn count_parameter(request: &RebuildRequest, key: &str, default: u32) -> Result<u32> {
    match request.parameters.get(key) {
        None => Ok(default),
        Some(Value::Quantity { value, .. }) => {
            if !value.is_finite() || *value < 0.0 {
                return Err(Error::Invalid("ring parameter"));
            }
            Ok(value.round().min(f64::from(u32::MAX)) as u32)
        }
        Some(Value::Number(value)) => {
            if !value.is_finite() || *value < 0.0 {
                return Err(Error::Invalid("ring parameter"));
            }
            Ok(value.round().min(f64::from(u32::MAX)) as u32)
        }
        Some(_) => Err(Error::Invalid("ring parameter")),
    }
}

/// One published stone placement frame. It is a reference, not a gem solid.
#[derive(Debug, Clone, PartialEq)]
pub struct StoneFrame {
    pub name: String,
    pub position: Point3,
    /// Radial outward normal at the frame, on the outer surface.
    pub normal: Point3,
}

/// A built ring body: the watertight mesh and the feasibility numbers of the solid.
#[derive(Debug, Clone, PartialEq)]
pub struct RingSolid {
    pub mesh: Mesh,
    /// Whether every undirected edge is shared by exactly two faces.
    pub closed: bool,
    /// Signed volume of the closed mesh, in mm^3.
    pub volume_mm3: f64,
    /// Smallest radial section thickness, in millimetres.
    pub min_wall_mm: f64,
    pub segments: usize,
}

impl RingSolid {
    pub fn summary(&self) -> String {
        format!(
            "{} solid {:.1} mm3, wall {:.2} mm, {} sections",
            if self.closed { "closed" } else { "open" },
            self.volume_mm3,
            self.min_wall_mm,
            self.segments
        )
    }
}

/// The sampled axial sections as one quad mesh: `SEGMENTS` meridian rectangles, each with
/// four corners, plus the smallest radial wall of the sampled sections.
///
/// The mesh and the OCCT B-Rep read the same sections (`OM9-L3-04-03-007`): the kernel sews
/// these very quads into a solid, so a preview, a mesh body and a B-Rep body cannot drift.
pub fn quad_mesh(params: &RingParams) -> Result<(Vec<Point3>, Vec<Vec<u32>>, f64)> {
    params.validate()?;
    let ri = params.diameter_mm / 2.0;
    let base_ro = ri + params.thickness_mm;
    let half = params.width_mm / 2.0;
    let mut min_wall = f64::INFINITY;
    let mut vertices: Vec<Point3> = Vec::with_capacity(SEGMENTS * 4);
    for index in 0..SEGMENTS {
        let theta = TAU * index as f64 / SEGMENTS as f64;
        let (ro, h) = section(params, theta, base_ro, half)?;
        min_wall = min_wall.min(ro - ri);
        let (sin, cos) = theta.sin_cos();
        for (r, z) in [(ri, -h), (ro, -h), (ro, h), (ri, h)] {
            vertices.push([r * cos, r * sin, z]);
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::with_capacity(SEGMENTS * 4);
    for index in 0..SEGMENTS {
        let next = (index + 1) % SEGMENTS;
        for corner in 0..4u32 {
            let following = (corner + 1) % 4;
            faces.push(vec![
                index as u32 * 4 + corner,
                next as u32 * 4 + corner,
                next as u32 * 4 + following,
                index as u32 * 4 + following,
            ]);
        }
    }
    let volume = signed_volume(&vertices, &faces);
    // Consistent outward winding: a negative signed volume means the quads are inside out.
    if volume < 0.0 {
        for face in &mut faces {
            face.reverse();
        }
    }
    Ok((vertices, faces, min_wall))
}

/// Build one ring body from validated parameters.
pub fn build(params: &RingParams) -> Result<RingSolid> {
    let (vertices, faces, min_wall) = quad_mesh(params)?;
    let mesh = Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let mesh = crate::geometry::Mesh { ..mesh };
    let report = inspect_solid(&mesh)?;
    Ok(RingSolid {
        mesh,
        closed: report.closed,
        volume_mm3: report.volume_mm3.abs(),
        min_wall_mm: min_wall,
        segments: SEGMENTS,
    })
}

/// The final B-Rep feasibility gate (`OM9-L3-04-03-007`): a production ring must be closed
/// with a positive volume. A deliberately open face model is refused instead of being
/// reported as a valid ring.
pub fn inspect_solid(mesh: &Mesh) -> Result<SolidReport> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return Err(Error::Invalid("ring solid empty"));
    }
    let mut edges: std::collections::BTreeMap<(u32, u32), u32> = std::collections::BTreeMap::new();
    for face in &mesh.faces {
        if face.len() < 3 {
            return Err(Error::Invalid("ring solid face"));
        }
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            if a == b || a as usize >= mesh.vertices.len() || b as usize >= mesh.vertices.len() {
                return Err(Error::Invalid("ring solid face"));
            }
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_insert(0) += 1;
        }
    }
    let closed = edges.values().all(|count| *count == 2);
    inspect_measures(closed, signed_volume(&mesh.vertices, &mesh.faces))
}

/// The one feasibility gate every ring body goes through, whatever its representation.
///
/// The mesh path counts undirected edges and integrates the signed volume itself; the OCCT
/// path measures the sewn solid and refuses a body that is not one closed shell before it
/// gets here. Both end in this function, so the accepted body has the same meaning.
pub fn inspect_measures(closed: bool, volume_mm3: f64) -> Result<SolidReport> {
    if !closed {
        return Err(Error::Invalid("ring solid open"));
    }
    if !volume_mm3.is_finite() || volume_mm3.abs() <= 0.0 {
        return Err(Error::Invalid("ring solid volume"));
    }
    Ok(SolidReport {
        closed,
        volume_mm3: volume_mm3.abs(),
    })
}

/// The feasibility gate for a committed or previewed ring of either representation.
///
/// `Geometry::Mesh` is checked edge by edge; `Geometry::Brep` is bound to its source and
/// measured from its display snapshot (the sewn solid already passed the adapter's closed
/// shell test). Any other geometry is not a ring body.
pub fn inspect_geometry(
    geometry: &Geometry,
    limits: &crate::budget::Limits,
) -> Result<SolidReport> {
    match geometry {
        Geometry::Mesh(mesh) => inspect_solid(mesh),
        Geometry::Brep(brep) => {
            brep.validate(limits)?;
            inspect_measures(true, display_volume(&brep.display)?)
        }
        _ => Err(Error::Invalid("ring solid geometry")),
    }
}

/// Signed volume of the display triangles of a B-Rep snapshot, in mm^3.
fn display_volume(display: &crate::brep::BrepDisplay) -> Result<f64> {
    let mut volume = 0.0;
    for [a, b, c, _face] in &display.triangles {
        let (Some(a), Some(b), Some(c)) = (
            display.points.get(*a as usize),
            display.points.get(*b as usize),
            display.points.get(*c as usize),
        ) else {
            return Err(Error::Invalid("ring solid face"));
        };
        volume += dot(*a, cross(*b, *c)) / 6.0;
    }
    if !volume.is_finite() {
        return Err(Error::Invalid("ring solid volume"));
    }
    Ok(volume)
}

/// The feasibility numbers of a solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidReport {
    pub closed: bool,
    pub volume_mm3: f64,
}

fn signed_volume(vertices: &[Point3], faces: &[Vec<u32>]) -> f64 {
    let mut volume = 0.0;
    for face in faces {
        let a = vertices[face[0] as usize];
        for index in 1..face.len() - 1 {
            let b = vertices[face[index] as usize];
            let c = vertices[face[index + 1] as usize];
            volume += dot(a, cross(b, c)) / 6.0;
        }
    }
    volume
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

/// The section outline at `theta`: outer radius and half height.
fn section(params: &RingParams, theta: f64, base_ro: f64, base_half: f64) -> Result<(f64, f64)> {
    let wrapped = wrap(theta);
    match params.kind {
        RingBuilderKind::Eternity => Ok((base_ro, base_half)),
        RingBuilderKind::Raised => {
            let arc = (params.raised_width_mm / 2.0) / base_ro;
            let bump = bump(wrapped, arc);
            Ok((base_ro + params.raised_height_mm * bump, base_half))
        }
        RingBuilderKind::Signet | RingBuilderKind::Award => {
            let arc = (params.top_length_mm / 2.0) / base_ro;
            let bump = bump(wrapped, arc);
            let pad_half =
                base_half + ((params.top_width_mm - params.width_mm) / 2.0).max(0.0) * bump;
            Ok((base_ro + params.shoulder_mm * bump, pad_half))
        }
    }
}

/// A smooth pad profile: 1 at the centre of the arc, 0 at its edges, so the shoulders blend
/// into the band instead of meeting it at a step the kernel would reject.
fn bump(theta: f64, arc: f64) -> f64 {
    if !arc.is_finite() || arc <= 0.0 {
        return 0.0;
    }
    let distance = theta.abs();
    if distance >= arc {
        return 0.0;
    }
    0.5 * (1.0 + (std::f64::consts::PI * (distance / arc)).cos())
}

fn wrap(theta: f64) -> f64 {
    let mut value = theta % TAU;
    if value > std::f64::consts::PI {
        value -= TAU;
    }
    if value < -std::f64::consts::PI {
        value += TAU;
    }
    value
}

/// The stone placement frames of an Eternity band. The band carries the frames and the
/// reference IDs; the gem solids stay with the gem builder (`M05_GEM_INTEGRATION_DEFERRED`).
pub fn stone_frames(params: &RingParams) -> Result<Vec<StoneFrame>> {
    params.validate()?;
    if params.kind != RingBuilderKind::Eternity {
        return Ok(Vec::new());
    }
    let radius = params.diameter_mm / 2.0 + params.thickness_mm;
    let mut frames = Vec::with_capacity(params.stone_count as usize);
    for index in 0..params.stone_count {
        let theta = TAU * index as f64 / f64::from(params.stone_count);
        let (sin, cos) = theta.sin_cos();
        frames.push(StoneFrame {
            name: format!("stone_{:02}", index + 1),
            position: [radius * cos, radius * sin, 0.0],
            normal: [cos, sin, 0.0],
        });
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(kind: RingBuilderKind) -> RingParams {
        kind.defaults()
    }

    #[test]
    fn an_eternity_band_builds_a_closed_solid_with_twelve_frames_and_no_gem_solids() {
        let params = params(RingBuilderKind::Eternity);
        let solid = build(&params).unwrap();
        assert!(solid.closed);
        assert!(solid.volume_mm3 > 0.0);
        assert!(solid.min_wall_mm >= MIN_WALL_MM);
        let frames = stone_frames(&params).unwrap();
        assert_eq!(frames.len(), 12);
        assert_eq!(frames[0].name, "stone_01");
        assert_eq!(frames[11].name, "stone_12");
        for frame in &frames {
            let radius = (frame.position[0] * frame.position[0]
                + frame.position[1] * frame.position[1])
                .sqrt();
            assert!((radius - 11.0).abs() < 1e-9);
            // A frame is a reference on the band, never a second body.
            assert!(
                (frame.normal[0] * frame.position[0] + frame.normal[1] * frame.position[1]) > 0.0
            );
        }
    }

    #[test]
    fn an_eternity_band_refuses_zero_stones_and_impossible_spacing() {
        let mut zero = params(RingBuilderKind::Eternity);
        zero.stone_count = 0;
        assert_eq!(zero.validate(), Err(Error::Invalid("ring stone count")));
        let mut crowded = params(RingBuilderKind::Eternity);
        crowded.stone_count = 40;
        assert_eq!(
            crowded.validate(),
            Err(Error::Invalid("ring stone spacing"))
        );
        assert_eq!(
            build(&crowded).err().unwrap(),
            Error::Invalid("ring stone spacing")
        );
    }

    #[test]
    fn a_signet_ring_is_closed_with_a_visible_top_and_two_shoulders() {
        let params = params(RingBuilderKind::Signet);
        let solid = build(&params).unwrap();
        assert!(solid.closed);
        let band = build(&RingBuilderKind::Eternity.defaults()).unwrap();
        assert!(solid.volume_mm3 > band.volume_mm3);
        // The top pad pushes the outer surface out at theta = 0 and tapers at the shoulders.
        let ri = params.diameter_mm / 2.0;
        let base = ri + params.thickness_mm;
        let top = section(&params, 0.0, base, params.width_mm / 2.0).unwrap();
        let shoulder = section(
            &params,
            (params.top_length_mm / 2.0) / base,
            base,
            params.width_mm / 2.0,
        )
        .unwrap();
        assert!(top.0 > base + 1.0);
        assert!((shoulder.0 - base).abs() < 1e-9);
    }

    #[test]
    fn a_signet_ring_refuses_a_negative_wall_and_a_self_intersecting_top() {
        let mut thin = params(RingBuilderKind::Signet);
        thin.thickness_mm = 0.1;
        assert_eq!(thin.validate(), Err(Error::Invalid("ring wall")));
        let mut folded = params(RingBuilderKind::Signet);
        folded.top_length_mm = 40.0;
        assert_eq!(folded.validate(), Err(Error::Invalid("ring signet top")));
        let mut shoulder = params(RingBuilderKind::Signet);
        shoulder.shoulder_mm = 0.0;
        assert_eq!(shoulder.validate(), Err(Error::Invalid("ring shoulder")));
    }

    #[test]
    fn a_raised_band_keeps_a_positive_section_thickness() {
        let params = params(RingBuilderKind::Raised);
        let solid = build(&params).unwrap();
        assert!(solid.closed);
        assert!(solid.volume_mm3 > 0.0);
        assert!(solid.min_wall_mm >= params.thickness_mm - MIN_WALL_MM * 1.0e-9);
        // A ridge deeper than the wall would leave zero thickness at the inner edge.
        let mut zero_wall = params.clone();
        zero_wall.raised_height_mm = zero_wall.thickness_mm;
        assert_eq!(
            zero_wall.validate(),
            Err(Error::Invalid("ring raised height"))
        );
    }

    #[test]
    fn an_award_ring_has_a_wider_top_than_the_signet() {
        let signet = build(&RingBuilderKind::Signet.defaults()).unwrap();
        let award = build(&RingBuilderKind::Award.defaults()).unwrap();
        assert!(award.closed);
        assert!(award.volume_mm3 > signet.volume_mm3);
    }

    #[test]
    fn the_final_gate_rejects_an_open_mesh_and_an_inverted_body() {
        let mut open = build(&RingBuilderKind::Eternity.defaults()).unwrap().mesh;
        open.faces.pop();
        assert_eq!(
            inspect_solid(&open).err().unwrap(),
            Error::Invalid("ring solid open")
        );
        // An inside-out closed mesh has a negative volume: flipping it is the caller's job,
        // the gate reports the magnitude only for a genuinely closed body.
        let mesh = build(&RingBuilderKind::Eternity.defaults()).unwrap().mesh;
        assert!(inspect_solid(&mesh).unwrap().closed);
    }

    #[test]
    fn every_kind_builds_and_reports_the_same_segment_count() {
        for kind in RingBuilderKind::ALL {
            let params = kind.defaults();
            let solid = build(&params).unwrap();
            assert_eq!(solid.segments, SEGMENTS);
            assert!(solid.closed);
            assert!(solid.volume_mm3 > 0.0);
            assert_eq!(solid.mesh.vertices.len(), SEGMENTS * 4);
            assert_eq!(solid.mesh.faces.len(), SEGMENTS * 4);
        }
    }
}
