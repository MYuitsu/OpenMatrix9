// SPDX-License-Identifier: LGPL-2.1-or-later
//! Settings family: prongs, channel, metal pieces and beads (`OM9-L2-04-15`).
//!
//! The family is shared geometry + placement logic for the Matrix *Setting* menu:
//!
//! * [`gems`] — the ordered open/closed gem row every sub-feature consumes (one gem contract,
//!   one output geometry contract; the parent module exclusion forbids a second gem store);
//! * [`prong`] — Prong Builder, Prong Editor and Prong on Surface, plus the row end modes and
//!   prong spacing;
//! * [`metal_piece`] — Metal from Gems (open/closed outside, inside, top and rim variants);
//! * [`channel_border`] — Channel Border Curve offsets around the same row;
//! * [`bead`] — Bead on Curve (Milgrain) and Bead on Surface;
//! * [`rail`] — Pull Object to Rail;
//! * [`associative`] — rebuild from a changed gem row and the metal mass report.
//!
//! Everything here is portable, safe Rust. The OCCT adapter in `om9-kernel` converts a
//! [`geometry::Mesh`] to a B-Rep when the host needs exact solids; this module never assumes a
//! handle it does not own and never writes the document.
use crate::{
    Error, Result,
    geometry::{Mesh, Point3, finite},
};

pub mod associative;
pub mod bead;
pub mod bezel;
pub mod channel_border;
pub mod gems;
pub mod head;
pub mod metal_piece;
pub mod prong;
pub mod rail;

/// The shortest prong the manual allows: "the minimum height for the prong is 0.1 mm".
pub const MIN_PRONG_HEIGHT_MM: f64 = 0.1;
/// A prong thinner than this collapses at the document tolerance.
pub const MIN_PRONG_DIAMETER_MM: f64 = 0.05;
/// The manual states the default prong/stone overlap: "by default, 10% of the prong overlaps
/// the stone".
pub const DEFAULT_NUDGE: f64 = 0.10;
/// Circumferential segments of a revolved prong or bead. Fixed so a preset round-trip
/// reproduces the same vertex count and volume.
pub const REVOLVE_SEGMENTS: usize = 24;
/// Guard against a runaway layout: a row or layout that asks for more prongs than this is a
/// budget refusal, not an allocation.
pub const MAX_INSTANCES: usize = 4096;

/// Euclidean length of a vector.
pub(crate) fn norm(v: Point3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub(crate) fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(crate) fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(crate) fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn mul(a: Point3, s: f64) -> Point3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[allow(dead_code)]
pub(crate) fn lerp(a: Point3, b: Point3, t: f64) -> Point3 {
    add(a, mul(sub(b, a), t))
}

/// A unit vector, refusing a zero or non-finite direction instead of returning `NaN`.
pub fn unit(v: Point3) -> Result<Point3> {
    finite(v)?;
    let length = norm(v);
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("settings frame"));
    }
    Ok(mul(v, 1.0 / length))
}

/// An orthonormal, right-handed local frame. Prongs, beads and metal sections are built in
/// local coordinates and placed through [`Frame::point`].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Frame {
    pub origin: Point3,
    /// Local X.
    pub x: Point3,
    /// Local Y.
    pub y: Point3,
    /// Local Z (the feature axis).
    pub z: Point3,
}

impl Frame {
    pub fn new(origin: Point3, x: Point3, y: Point3, z: Point3) -> Result<Self> {
        finite(origin)?;
        let x = unit(x)?;
        let y = unit(y)?;
        let z = unit(z)?;
        if dot(x, y).abs() > 1.0e-9 || dot(x, z).abs() > 1.0e-9 || dot(y, z).abs() > 1.0e-9 {
            return Err(Error::Invalid("settings frame"));
        }
        Ok(Self { origin, x, y, z })
    }

    /// A frame around `axis` with a caller supplied "north" hint. The hint is projected into
    /// the plane normal to the axis, so a frame is always well defined for a legal axis.
    pub fn from_axis(origin: Point3, axis: Point3, north_hint: Point3) -> Result<Self> {
        let z = unit(axis)?;
        let hint = sub(north_hint, mul(z, dot(north_hint, z)));
        let x = if norm(hint) <= 1.0e-9 {
            // Any vector not parallel to the axis works; X/Z/Y keeps the frame right-handed.
            let fallback = if z[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            unit(cross(z, fallback))?
        } else {
            unit(hint)?
        };
        let y = unit(cross(z, x))?;
        Ok(Self { origin, x, y, z })
    }

    /// Place a local point in world space.
    pub fn point(&self, local: Point3) -> Result<Point3> {
        finite(local)?;
        let world = add(
            self.origin,
            add(
                add(mul(self.x, local[0]), mul(self.y, local[1])),
                mul(self.z, local[2]),
            ),
        );
        finite(world)?;
        Ok(world)
    }

    /// Map a local direction to world space without translation.
    pub fn direction(&self, local: Point3) -> Point3 {
        add(
            add(mul(self.x, local[0]), mul(self.y, local[1])),
            mul(self.z, local[2]),
        )
    }
}

/// The feasibility numbers of a built solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidReport {
    /// Every undirected edge is used by exactly two faces.
    pub closed: bool,
    /// Signed volume magnitude in mm^3.
    pub volume_mm3: f64,
}

/// The family's final gate: a production prong/bead/metal body must be closed with a positive
/// volume. A deliberately open curve (Channel Border) is typed as a curve instead of being run
/// through this gate.
/// Fan-triangulate every polygon face. The CSG owner accepts triangle faces only, so a swept
/// section must be triangulated before a boolean; the fan keeps the manifold edge pairing.
pub fn triangulate_mesh(mesh: &Mesh) -> Result<Mesh> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return Err(Error::Invalid("settings solid empty"));
    }
    let mut faces = Vec::with_capacity(mesh.faces.len());
    for face in &mesh.faces {
        if face.len() < 3 {
            return Err(Error::Invalid("settings solid face"));
        }
        for index in 1..face.len() - 1 {
            faces.push(vec![face[0], face[index], face[index + 1]]);
        }
    }
    Ok(Mesh {
        vertices: mesh.vertices.clone(),
        faces,
        normals: mesh.normals.clone(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    })
}

pub fn inspect_solid(mesh: &Mesh) -> Result<SolidReport> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return Err(Error::Invalid("settings solid empty"));
    }
    let mut edges: std::collections::BTreeMap<(u32, u32), u32> = std::collections::BTreeMap::new();
    for face in &mesh.faces {
        if face.len() < 3 {
            return Err(Error::Invalid("settings solid face"));
        }
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            if a == b
                || a as usize >= mesh.vertices.len()
                || b as usize >= mesh.vertices.len()
            {
                return Err(Error::Invalid("settings solid face"));
            }
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_insert(0) += 1;
        }
    }
    if !edges.values().all(|count| *count == 2) {
        return Err(Error::Invalid("settings solid open"));
    }
    let volume = signed_volume(&mesh.vertices, &mesh.faces);
    if !volume.is_finite() || volume.abs() <= 0.0 {
        return Err(Error::Invalid("settings solid volume"));
    }
    Ok(SolidReport {
        closed: true,
        volume_mm3: volume.abs(),
    })
}

pub(crate) fn signed_volume(vertices: &[Point3], faces: &[Vec<u32>]) -> f64 {
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

fn outward(mut mesh: Mesh) -> Mesh {
    if signed_volume(&mesh.vertices, &mesh.faces) < 0.0 {
        for face in &mut mesh.faces {
            face.reverse();
        }
    }
    mesh
}

/// Revolve a closed `(radius, axial)` section around the local Z axis.
///
/// A section point on the axis becomes one pole vertex instead of a ring of coincident
/// vertices, so the result is watertight for flat caps and for a spherical cap.
pub fn revolve_profile(profile: &[(f64, f64)], segments: usize) -> Result<Mesh> {
    if profile.len() < 3 {
        return Err(Error::Invalid("settings profile"));
    }
    if !(3..=MAX_INSTANCES).contains(&segments) {
        return Err(Error::Invalid("settings revolve segments"));
    }
    for (radius, axial) in profile {
        if !radius.is_finite() || !axial.is_finite() || *radius < 0.0 {
            return Err(Error::Invalid("settings profile"));
        }
    }
    let on_axis = |radius: f64| radius <= 1.0e-9;
    let mut vertices = Vec::with_capacity(profile.len() * segments);
    // Index of the first vertex of each section point; a pole is a single vertex.
    let mut first = Vec::with_capacity(profile.len());
    for (radius, axial) in profile {
        first.push(vertices.len() as u32);
        if on_axis(*radius) {
            vertices.push([0.0, 0.0, *axial]);
        } else {
            for index in 0..segments {
                let theta = std::f64::consts::TAU * index as f64 / segments as f64;
                let (sin, cos) = theta.sin_cos();
                vertices.push([radius * cos, radius * sin, *axial]);
            }
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for index in 0..profile.len() {
        let next = (index + 1) % profile.len();
        let (r0, _) = profile[index];
        let (r1, _) = profile[next];
        let pole0 = on_axis(r0);
        let pole1 = on_axis(r1);
        if pole0 && pole1 {
            continue;
        }
        for segment in 0..segments {
            let following = (segment + 1) % segments;
            if pole0 {
                faces.push(vec![
                    first[index],
                    first[next] + following as u32,
                    first[next] + segment as u32,
                ]);
            } else if pole1 {
                faces.push(vec![
                    first[index] + segment as u32,
                    first[index] + following as u32,
                    first[next],
                ]);
            } else {
                faces.push(vec![
                    first[index] + segment as u32,
                    first[index] + following as u32,
                    first[next] + following as u32,
                    first[next] + segment as u32,
                ]);
            }
        }
    }
    let mesh = Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    Ok(outward(mesh))
}

/// One station of a swept section: the local frame of the section plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Station {
    pub origin: Point3,
    /// Section local X (the profile's radial direction).
    pub x: Point3,
    /// Section local Y (the profile's axial direction).
    pub y: Point3,
}

impl Station {
    pub fn new(origin: Point3, x: Point3, y: Point3) -> Result<Self> {
        finite(origin)?;
        let x = unit(x)?;
        let y = unit(y)?;
        if dot(x, y).abs() > 1.0e-9 {
            return Err(Error::Invalid("settings sweep frame"));
        }
        Ok(Self { origin, x, y })
    }

    fn point(&self, section: (f64, f64)) -> Point3 {
        add(
            self.origin,
            add(mul(self.x, section.0), mul(self.y, section.1)),
        )
    }
}

/// Sweep a closed 2D section along a station sequence.
///
/// An open path gets a flat cap at each end (so the body is a closed solid); a closed path is
/// a ring and gets no cap. The result is checked by [`inspect_solid`] before it is returned.
pub fn sweep_section(stations: &[Station], section: &[(f64, f64)], closed_path: bool) -> Result<Mesh> {
    if stations.len() < 2 {
        return Err(Error::Invalid("settings sweep stations"));
    }
    if section.len() < 3 {
        return Err(Error::Invalid("settings sweep section"));
    }
    for (x, y) in section {
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::Invalid("settings sweep section"));
        }
    }
    let ring = section.len();
    let mut vertices: Vec<Point3> = Vec::with_capacity(stations.len() * ring);
    for station in stations {
        for point in section {
            vertices.push(station.point(*point));
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let spans = if closed_path {
        stations.len()
    } else {
        stations.len() - 1
    };
    for station in 0..spans {
        let next = (station + 1) % stations.len();
        for index in 0..ring {
            let following = (index + 1) % ring;
            faces.push(vec![
                (station * ring + index) as u32,
                (station * ring + following) as u32,
                (next * ring + following) as u32,
                (next * ring + index) as u32,
            ]);
        }
    }
    if !closed_path {
        // Cap each end with a fan around the section centroid. The cap reuses the section
        // ring edges, so the swept body closes exactly.
        for (station, forward) in [(0usize, false), (stations.len() - 1, true)] {
            let centroid = section
                .iter()
                .fold([0.0, 0.0], |acc, p| [acc[0] + p.0, acc[1] + p.1]);
            let centroid = (centroid[0] / ring as f64, centroid[1] / ring as f64);
            let center = vertices.len() as u32;
            vertices.push(stations[station].point(centroid));
            for index in 0..ring {
                let following = (index + 1) % ring;
                let a = (station * ring + index) as u32;
                let b = (station * ring + following) as u32;
                if forward {
                    faces.push(vec![center, a, b]);
                } else {
                    faces.push(vec![center, b, a]);
                }
            }
        }
    }
    let mesh = outward(Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    });
    inspect_solid(&mesh)?;
    Ok(mesh)
}

/// A sphere of `diameter_mm` centred on `center`; used by Bead on Curve/Surface.
pub fn sphere(center: Point3, diameter_mm: f64, segments: usize) -> Result<Mesh> {
    if !diameter_mm.is_finite() || diameter_mm <= 0.0 {
        return Err(Error::Invalid("bead diameter"));
    }
    finite(center)?;
    let radius = diameter_mm / 2.0;
    let rings = segments.max(6);
    let mut profile = Vec::with_capacity(rings + 2);
    profile.push((0.0, -radius));
    for index in 1..rings {
        let angle = -std::f64::consts::FRAC_PI_2
            + std::f64::consts::PI * index as f64 / rings as f64;
        profile.push((radius * angle.cos(), radius * angle.sin()));
    }
    profile.push((0.0, radius));
    let mesh = revolve_profile(&profile, segments)?;
    let translated = Mesh {
        vertices: mesh
            .vertices
            .iter()
            .map(|p| add(*p, center))
            .collect::<Vec<_>>(),
        ..mesh
    };
    Ok(translated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_revolved_section_is_closed_and_has_the_expected_volume() {
        // A 2 mm diameter, 10 mm tall cylinder: pi r^2 h.
        let profile = [(0.0, 0.0), (1.0, 0.0), (1.0, 10.0), (0.0, 10.0)];
        let mesh = revolve_profile(&profile, 96).unwrap();
        let report = inspect_solid(&mesh).unwrap();
        let expected = std::f64::consts::PI * 10.0;
        assert!((report.volume_mm3 - expected).abs() < 0.05, "{report:?}");
    }

    #[test]
    fn a_cut_profile_is_not_silently_wound_inside_out() {
        let profile = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let mesh = revolve_profile(&profile, 24).unwrap();
        assert!(signed_volume(&mesh.vertices, &mesh.faces) > 0.0);
    }

    #[test]
    fn a_degenerate_profile_is_refused() {
        assert!(revolve_profile(&[(0.0, 0.0), (0.0, 1.0)], 24).is_err());
        assert!(revolve_profile(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)], 2).is_err());
    }

    #[test]
    fn a_frame_refuses_a_non_orthogonal_triple() {
        let frame = Frame::new(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        assert_eq!(frame.err(), Some(Error::Invalid("settings frame")));
    }

    #[test]
    fn a_swept_open_section_is_capped_and_closed() {
        let stations = vec![
            Station::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]).unwrap(),
            Station::new([4.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]).unwrap(),
        ];
        // A 1x1 mm square section over 4 mm: 4 mm^3.
        let mesh = sweep_section(&stations, &[(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)], false)
            .unwrap();
        let report = inspect_solid(&mesh).unwrap();
        assert!((report.volume_mm3 - 4.0).abs() < 0.01, "{report:?}");
    }
}
