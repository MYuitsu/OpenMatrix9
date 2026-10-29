// SPDX-License-Identifier: LGPL-2.1-or-later
//! M07 cutter family: gem seat/azure cutters, the Gem Cutter Library, the dimensional Bezel
//! Cutter and the Interactive Boolean policy (`OM9-L2-04-16`, manual book 2 pages 272-292).
//!
//! The family is *portable* cutter geometry plus the rules that decide what a cutter may cut:
//!
//! * [`gem`] — `gvAzureCutter`: the Gem/Azure/Advanced builder modes, the gem-relative
//!   percentage controls and the pavilion clearance gate (`OM9-L3-04-16-001/-002/-005`);
//! * [`library`] — the Gem Cutter Library presets with their `Scale X/Y`, `Z Scale` and
//!   `Z Offset` controls (`OM9-L3-04-16-003`);
//! * [`bezel`] — the dimensional Bezel Cutter Builder (`gvBezelCutter`, `OM9-L3-04-16-004`);
//! * [`boolean`] — the Interactive Boolean (`Off/On/One`) and the cutter body Preserve/Remove
//!   policy (`OM9-L3-04-16-006/-007`).
//!
//! Geometry is built in the gem's local frame around its axis, so a cutter follows the stone's
//! placement and orientation. Nothing here mutates a document, owns a transaction or writes a
//! second gem store: the host session commits the returned meshes through its own transaction,
//! and the one gem record is the `om9.gem` payload the gems owner publishes.
//!
//! The measured B-Rep truth stays with the kernel owner (`OM9-L2-01-10`); the host converts a
//! [`Mesh`] when it needs an exact solid. The pentagon of the family is the *lathe*: a list of
//! [`Ring`]s lofted into one closed body, which is what every Matrix gem cutter is.
use crate::{
    Error, Result,
    geometry::{Mesh, Point3},
};

pub mod bezel;
pub mod boolean;
pub mod brightcut;
pub mod channel;
pub mod gem;
pub mod library;
pub mod microprong;

/// Circumferential segments of one cutter ring. Fixed so a rebuild reproduces the same vertex
/// count, volume and Boolean result.
pub const CUTTER_SEGMENTS: usize = 48;
/// Upper bound on the cutter count a single command may create.
pub const MAX_CUTTERS: usize = 256;
/// The smallest cutter body the document tolerance can still boolean reliably, mm.
pub const MIN_CUTTER_MM: f64 = 0.02;

/// The edge profile of the cutter's hole: what the *Azure* and *Advanced* modes change.
///
/// The manual only names the library ("Changing the Edge profile changes the shape of the azure
/// or cut out hole"); the authored shapes below are OM9 vector profiles, not Matrix bitmaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EdgeShape {
    pub kind: EdgeKind,
    /// Groove count of a `Brite`/`Grooves` profile.
    pub grooves: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Round,
    Square,
    Point,
    Heart,
    Brite,
    Grooves,
}

impl EdgeKind {
    pub const ALL: [Self; 6] = [
        Self::Round,
        Self::Square,
        Self::Point,
        Self::Heart,
        Self::Brite,
        Self::Grooves,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Point => "Point",
            Self::Heart => "Heart",
            Self::Brite => "Brite",
            Self::Grooves => "Grooves",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "round" | "circle" | "oval" => Ok(Self::Round),
            "square" | "princess" | "cube" => Ok(Self::Square),
            "point" | "marquise" | "pear" => Ok(Self::Point),
            "heart" | "hart" => Ok(Self::Heart),
            "brite" | "bright" | "brightcut" | "bright cut" => Ok(Self::Brite),
            "groove" | "grooves" | "channel" => Ok(Self::Grooves),
            _ => Err(Error::Invalid("cutter edge profile")),
        }
    }
}

impl Default for EdgeShape {
    fn default() -> Self {
        Self {
            kind: EdgeKind::Round,
            grooves: 8,
        }
    }
}

impl EdgeShape {
    pub fn new(kind: EdgeKind) -> Self {
        Self {
            kind,
            grooves: 8,
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        Ok(Self::new(EdgeKind::from_tag(tag)?))
    }

    pub fn label(self) -> &'static str {
        self.kind.label()
    }

    /// Validate the groove count and return the shape.
    pub fn validate(self) -> Result<Self> {
        if matches!(self.kind, EdgeKind::Brite | EdgeKind::Grooves)
            && !(3..=64).contains(&self.grooves)
        {
            return Err(Error::Invalid("cutter edge grooves"));
        }
        Ok(self)
    }

    /// The radial scale of this profile at `theta` radians. Every shape stays inside the unit
    /// circle (`0 < scale <= 1`), so a cutter built from a gem's girdle never grows past it.
    pub fn scale(self, theta: f64) -> f64 {
        let (sin, cos) = theta.sin_cos();
        let scale = match self.kind {
            EdgeKind::Round => 1.0,
            EdgeKind::Square => {
                // A square inscribed in the girdle circle: the corners touch the circle.
                let p = 8.0_f64;
                let raw = (cos.abs().powf(p) + sin.abs().powf(p)).powf(-1.0 / p);
                raw * 2.0_f64.powf(1.0 / p) / std::f64::consts::SQRT_2
            }
            EdgeKind::Point => {
                // A diamond: the four points touch the girdle circle.
                let raw = 1.0 / cos.abs().max(sin.abs()).max(1.0e-9);
                raw / std::f64::consts::SQRT_2
            }
            EdgeKind::Heart => {
                // A round hole with the manual's heart notch at one side of the axis.
                let notch = (-((theta - std::f64::consts::PI).powi(2)) / 0.08).exp();
                (1.0 - 0.25 * notch).clamp(0.5, 1.0)
            }
            EdgeKind::Brite => {
                let n = self.grooves.max(3) as f64;
                (1.0 - 0.12 * (0.5 + 0.5 * (n * theta).cos())).clamp(0.5, 1.0)
            }
            EdgeKind::Grooves => {
                let n = self.grooves.max(3) as f64;
                (1.0 - 0.20 * (0.5 + 0.5 * (n * theta).cos())).clamp(0.4, 1.0)
            }
        };
        scale.clamp(0.05, 1.0)
    }
}

/// One control ring of a cutter body, in the gem's local frame: `z` along the gem axis,
/// `radius_x`/`radius_y` on the girdle plane, an optional rotation about that axis and the edge
/// profile that ring's hole uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub z: f64,
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation_deg: f64,
    pub shape: EdgeShape,
}

impl Ring {
    pub fn round(z: f64, radius: f64) -> Self {
        Self {
            z,
            radius_x: radius,
            radius_y: radius,
            rotation_deg: 0.0,
            shape: EdgeShape::default(),
        }
    }

    pub fn elliptical(z: f64, radius_x: f64, radius_y: f64) -> Self {
        Self {
            z,
            radius_x,
            radius_y,
            rotation_deg: 0.0,
            shape: EdgeShape::default(),
        }
    }

    /// The smallest radius this ring reaches, including the edge profile's inward grooves.
    pub fn min_radius(&self) -> f64 {
        let edge = (0..64)
            .map(|step| {
                let theta = std::f64::consts::TAU * step as f64 / 64.0;
                self.shape.scale(theta)
            })
            .fold(1.0_f64, f64::min);
        self.radius_x.min(self.radius_y) * edge
    }

    fn validate(&self) -> Result<()> {
        if !self.z.is_finite()
            || !self.radius_x.is_finite()
            || !self.radius_y.is_finite()
            || !self.rotation_deg.is_finite()
            || self.radius_x < 0.0
            || self.radius_y < 0.0
        {
            return Err(Error::Invalid("cutter ring"));
        }
        self.shape.validate()?;
        Ok(())
    }

    fn is_pole(&self) -> bool {
        self.radius_x <= 1.0e-9 && self.radius_y <= 1.0e-9
    }

    fn point(&self, theta: f64) -> Point3 {
        let scale = self.shape.scale(theta);
        let angle = theta + self.rotation_deg.to_radians();
        let (sin, cos) = angle.sin_cos();
        [
            self.radius_x * scale * cos,
            self.radius_y * scale * sin,
            self.z,
        ]
    }
}

/// Loft a bottom-to-top ring list into one closed body.
///
/// A ring with a zero radius becomes one pole vertex, so a cone tip is watertight instead of a
/// ring of coincident points. Both ends that are not poles get a flat fan cap. The result is
/// checked by [`crate::settings::inspect_solid`], so an accidental open shell is refused here
/// rather than at export.
pub fn loft(rings: &[Ring], segments: usize) -> Result<Mesh> {
    if rings.len() < 2 {
        return Err(Error::Invalid("cutter rings"));
    }
    if !(3..=crate::settings::MAX_INSTANCES).contains(&segments) {
        return Err(Error::Invalid("cutter segments"));
    }
    for ring in rings {
        ring.validate()?;
    }
    for pair in rings.windows(2) {
        if pair[1].z <= pair[0].z + 1.0e-9 {
            return Err(Error::Invalid("cutter ring order"));
        }
    }
    let mut vertices: Vec<Point3> = Vec::with_capacity(rings.len() * segments);
    let mut index: Vec<Vec<u32>> = Vec::with_capacity(rings.len());
    for ring in rings {
        if ring.is_pole() {
            let at = vertices.len() as u32;
            vertices.push([0.0, 0.0, ring.z]);
            index.push(vec![at]);
        } else {
            let mut row = Vec::with_capacity(segments);
            for segment in 0..segments {
                let theta = std::f64::consts::TAU * segment as f64 / segments as f64;
                row.push(vertices.len() as u32);
                vertices.push(ring.point(theta));
            }
            index.push(row);
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for span in 0..rings.len() - 1 {
        let lower = &index[span];
        let upper = &index[span + 1];
        if lower.len() == 1 && upper.len() == 1 {
            continue;
        }
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            if lower.len() == 1 {
                faces.push(vec![lower[0], upper[next], upper[segment]]);
            } else if upper.len() == 1 {
                faces.push(vec![lower[segment], lower[next], upper[0]]);
            } else {
                faces.push(vec![
                    lower[segment],
                    lower[next],
                    upper[next],
                    upper[segment],
                ]);
            }
        }
    }
    cap(&mut vertices, &mut faces, &index[0], rings[0].z, false);
    cap(
        &mut vertices,
        &mut faces,
        &index[rings.len() - 1],
        rings[rings.len() - 1].z,
        true,
    );
    let mesh = orient(Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    });
    crate::settings::inspect_solid(&mesh)?;
    Ok(mesh)
}

/// Flip a closed mesh whose faces were authored inside out, so the signed volume is positive.
fn orient(mut mesh: Mesh) -> Mesh {
    if crate::settings::signed_volume(&mesh.vertices, &mesh.faces) < 0.0 {
        for face in &mut mesh.faces {
            face.reverse();
        }
    }
    mesh
}

/// A flat fan cap around the ring centroid; a pole needs no cap.
fn cap(vertices: &mut Vec<Point3>, faces: &mut Vec<Vec<u32>>, ring: &[u32], z: f64, top: bool) {
    if ring.len() < 3 {
        return;
    }
    let mut centre = [0.0, 0.0, z];
    for index in ring {
        let point = vertices[*index as usize];
        centre[0] += point[0];
        centre[1] += point[1];
    }
    centre[0] /= ring.len() as f64;
    centre[1] /= ring.len() as f64;
    let at = vertices.len() as u32;
    vertices.push(centre);
    let count = ring.len();
    for index in 0..count {
        let a = ring[index];
        let b = ring[(index + 1) % count];
        if top {
            faces.push(vec![at, a, b]);
        } else {
            faces.push(vec![at, b, a]);
        }
    }
}

/// The volume of one closed cutter body, mm³.
pub fn volume_mm3(mesh: &Mesh) -> Result<f64> {
    Ok(crate::settings::inspect_solid(mesh)?.volume_mm3)
}

/// Build one closed body from explicit vertices and faces, refusing an open shell.
///
/// A face may have more than three corners (a block end is a rectangle); it is fanned in its
/// authored order, so the winding the caller wrote is the winding the solid keeps.
pub fn solid(vertices: Vec<Point3>, faces: Vec<Vec<u32>>) -> Result<Mesh> {
    let vertices_count = vertices.len();
    let mut triangles = Vec::new();
    for face in faces {
        if face.len() < 3 || face.iter().any(|index| *index as usize >= vertices_count) {
            return Err(Error::Invalid("cutter face"));
        }
        for index in 1..face.len() - 1 {
            triangles.push(vec![face[0], face[index], face[index + 1]]);
        }
    }
    let mesh = orient(Mesh {
        vertices,
        faces: triangles,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    });
    crate::settings::inspect_solid(&mesh)?;
    Ok(mesh)
}

/// Sweep equal-length corner rings along a path into one closed body.
///
/// Every section is one cross-section in world space, wound the same way around the path; the
/// first and last sections are closed by a cap. The result is admitted by [`solid`], so an
/// accidental open or non-manifold sweep is refused here instead of at export.
pub fn sweep(sections: &[Vec<Point3>]) -> Result<Mesh> {
    if sections.len() < 2 {
        return Err(Error::Invalid("cutter sweep"));
    }
    let count = sections[0].len();
    if count < 3 || sections.iter().any(|section| section.len() != count) {
        return Err(Error::Invalid("cutter sweep"));
    }
    let mut vertices = Vec::with_capacity(sections.len() * count);
    let mut rings = Vec::with_capacity(sections.len());
    for section in sections {
        let mut ring = Vec::with_capacity(count);
        for point in section {
            ring.push(vertices.len() as u32);
            vertices.push(*point);
        }
        rings.push(ring);
    }
    let mut faces = Vec::with_capacity((sections.len() - 1) * count + 2);
    for span in 0..rings.len() - 1 {
        for corner in 0..count {
            let next = (corner + 1) % count;
            faces.push(vec![
                rings[span][corner],
                rings[span][next],
                rings[span + 1][next],
                rings[span + 1][corner],
            ]);
        }
    }
    faces.push(rings[0].iter().rev().copied().collect());
    faces.push(rings[rings.len() - 1].clone());
    solid(vertices, faces)
}

/// Move a mesh from the gem's local frame into world space.
pub fn place(mesh: &Mesh, placement: &crate::geometry::Transform) -> Result<Mesh> {
    placement.validate()?;
    let vertices = mesh
        .vertices
        .iter()
        .map(|point| placement.point(*point))
        .collect::<Result<Vec<_>>>()?;
    Ok(Mesh {
        vertices,
        faces: mesh.faces.clone(),
        normals: Vec::new(),
        colors: mesh.colors.clone(),
        corner_uvs: mesh.corner_uvs.clone(),
    })
}

/// A gem's local dimensions the cutter geometry is built from, in mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GemCutterReference {
    /// Half the girdle length (local X).
    pub radius_x: f64,
    /// Half the girdle width (local Y).
    pub radius_y: f64,
    /// Half the girdle band thickness.
    pub girdle_half_mm: f64,
    pub crown_height_mm: f64,
    pub pavilion_depth_mm: f64,
}

impl GemCutterReference {
    pub fn from_spec(spec: &crate::gem::GemSpec) -> Result<Self> {
        spec.validate()?;
        let reference = Self {
            radius_x: spec.length_mm * 0.5,
            radius_y: spec.width_mm * 0.5,
            girdle_half_mm: spec.girdle_mm * 0.5,
            crown_height_mm: spec.crown_height_mm,
            pavilion_depth_mm: spec.pavilion_depth_mm,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn validate(&self) -> Result<()> {
        for value in [
            self.radius_x,
            self.radius_y,
            self.girdle_half_mm,
            self.crown_height_mm,
            self.pavilion_depth_mm,
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(Error::Invalid("cutter gem reference"));
            }
        }
        Ok(())
    }

    /// The smaller girdle half-extent: the conservative radius for a clearance check.
    pub fn min_radius_mm(&self) -> f64 {
        self.radius_x.min(self.radius_y)
    }

    /// The gem's radius at local axial `z`, by linear interpolation of the measured profile.
    ///
    /// The profile is the same one the gems owner publishes (`gem::profile_points`), so the
    /// clearance gate speaks about the actual stone, not about a nominal cone.
    pub fn radius_at(&self, profile: &[[f64; 2]], z: f64, scale: f64) -> f64 {
        if profile.len() < 2 || !z.is_finite() {
            return 0.0;
        }
        let mut points = profile.to_vec();
        points.sort_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal));
        if z <= points[0][1] || z >= points[points.len() - 1][1] {
            return if z < points[0][1] {
                0.0
            } else {
                points[points.len() - 1][0] * scale
            };
        }
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if z >= a[1] && z <= b[1] {
                let span = b[1] - a[1];
                let t = if span.abs() <= 1.0e-12 {
                    0.0
                } else {
                    (z - a[1]) / span
                };
                return (a[0] + (b[0] - a[0]) * t) * scale;
            }
        }
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cylinder_lofts_to_the_expected_volume() {
        // 2 mm diameter, 10 mm tall: pi r^2 h.
        let rings = [Ring::round(0.0, 1.0), Ring::round(10.0, 1.0)];
        let mesh = loft(&rings, 96).expect("cylinder");
        let volume = volume_mm3(&mesh).expect("closed");
        let expected = std::f64::consts::PI * 10.0;
        assert!((volume - expected).abs() < 0.05, "{volume} vs {expected}");
    }

    #[test]
    fn a_cone_gets_a_pole_and_stays_closed() {
        // Radius 1 at z=0 tapering to a point at z=3: pi r^2 h / 3.
        let rings = [Ring::round(0.0, 1.0), Ring::round(3.0, 0.0)];
        let mesh = loft(&rings, 96).expect("cone");
        let volume = volume_mm3(&mesh).expect("closed");
        let expected = std::f64::consts::PI * 3.0 / 3.0;
        assert!((volume - expected).abs() < 0.02, "{volume} vs {expected}");
    }

    #[test]
    fn a_frustum_matches_the_analytic_volume() {
        let (r0, r1, height) = (2.0_f64, 1.0_f64, 4.0_f64);
        let rings = [Ring::round(0.0, r0), Ring::round(height, r1)];
        let mesh = loft(&rings, 128).expect("frustum");
        let volume = volume_mm3(&mesh).expect("closed");
        let expected = std::f64::consts::PI * height / 3.0 * (r0 * r0 + r0 * r1 + r1 * r1);
        assert!((volume - expected).abs() < 0.15, "{volume} vs {expected}");
    }

    #[test]
    fn a_non_monotonic_ring_list_is_refused() {
        let rings = [Ring::round(0.0, 1.0), Ring::round(-1.0, 0.5)];
        assert_eq!(loft(&rings, 24).err(), Some(Error::Invalid("cutter ring order")));
    }

    #[test]
    fn every_edge_profile_stays_inside_the_unit_circle() {
        for kind in EdgeKind::ALL {
            let shape = EdgeShape::new(kind);
            for step in 0..720 {
                let theta = std::f64::consts::TAU * step as f64 / 720.0;
                let scale = shape.scale(theta);
                assert!(scale > 0.0 && scale <= 1.0 + 1.0e-12, "{kind:?} {scale}");
            }
        }
    }

    #[test]
    fn an_unknown_edge_profile_is_a_typed_refusal() {
        assert_eq!(
            EdgeShape::from_tag("teardrop").err(),
            Some(Error::Invalid("cutter edge profile"))
        );
    }
}
