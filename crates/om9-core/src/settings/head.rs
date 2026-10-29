// SPDX-License-Identifier: LGPL-2.1-or-later
//! Head Builder (`OM9-L2-04-14`): the basket/half-bezel head around one gem, its gallery rails
//! and the per-level element edits, plus the girdle seat frames and the rebuild/validity report
//! that the Bezel Builder shares.
//!
//! The manual (`matrix_8_manual_book2.pdf`, Head Builder, printed pages 468-474) fixes the
//! controls: one gem in the builder, a **prong profile** shape with a toggle to the **half-bezel**
//! shape, a **rail profile**, a prong count that depends on the gem shape, `Height Above Girdle`,
//! `Head Drop`, `Head Angle X`, `Top Angle`, `Prong Size`, `Prong Nudge`, `Prong Angle`, rail
//! count/drop/size and four edit levels (*Overall*, grouped prongs/rails, grouped ends, individual
//! elements). Everything is expressed in the gem's girdle frame: the girdle outline is the common
//! spine for the prongs, the rails, the half-bezel band and the bezel.
//!
//! This module is portable safe Rust; it never writes the document and never assumes a kernel
//! handle. The OCCT adapter converts a [`Mesh`] to a B-Rep on the host side after
//! [`inspect_solid`] has accepted it.
use crate::{
    Error, Result,
    gem::{self, GemShape, GemSpec},
    geometry::{Mesh, Point3, Transform, finite},
    settings::{
        Frame, MAX_INSTANCES, REVOLVE_SEGMENTS, Station, inspect_solid, norm, triangulate_mesh,
        unit,
    },
};

/// The shortest head height the manual allows above the girdle.
pub const MIN_HEAD_HEIGHT_MM: f64 = 0.1;
/// A prong or rail thinner than this collapses at document tolerance.
pub const MIN_HEAD_ELEMENT_MM: f64 = 0.05;
/// The default outer overlap of a prong on the stone: the manual states 10 %.
pub const DEFAULT_HEAD_NUDGE: f64 = 0.10;
/// Minimum clearance kept between a prong and the pavilion surface.
pub const PAVILION_CLEARANCE_MM: f64 = 0.05;
/// Upper bound of the prong count of one head.
pub const MAX_HEAD_PRONGS: u8 = 24;
/// Upper bound of the gallery rails of one head.
pub const MAX_HEAD_RAILS: u8 = 4;

/// The girdle spine of one gem, in world space: a closed outline plus the local height profile
/// the settings must stay out of.
#[derive(Debug, Clone, PartialEq)]
pub struct GemGirdle {
    pub gem_id: String,
    /// Girdle centre (mid-height), in mm.
    pub center: Point3,
    /// Girdle plane normal pointing to the table side, unit.
    pub normal: Point3,
    /// Girdle outline in world space, closed (the first point is not repeated).
    pub outline: Vec<Point3>,
    /// Top of the girdle band above [`Self::center`], in mm (positive).
    pub girdle_top_mm: f64,
    /// Bottom of the girdle band below [`Self::center`], in mm (positive).
    pub girdle_bottom_mm: f64,
    pub crown_height_mm: f64,
    pub pavilion_depth_mm: f64,
    /// `(radius, axial)` profile in gem local coordinates, culet first and table last.
    pub profile: Vec<[f64; 2]>,
}

impl GemGirdle {
    /// Place a gem spec through its document transform.
    pub fn from_spec(spec: &GemSpec, placement: &Transform) -> Result<Self> {
        placement.validate()?;
        let local = gem::girdle_outline(spec)?;
        let center = placement.point([0.0, 0.0, 0.0])?;
        let normal = unit(crate::settings::sub(placement.point([0.0, 0.0, 1.0])?, center))?;
        let outline = local
            .into_iter()
            .map(|point| placement.point(point))
            .collect::<Result<Vec<_>>>()?;
        let profile = gem::profile_points(spec)?;
        let girdle = Self {
            gem_id: format!("gem:{}", spec.shape.tag()),
            center,
            normal,
            outline,
            girdle_top_mm: spec.girdle_mm * 0.5,
            girdle_bottom_mm: spec.girdle_mm * 0.5,
            crown_height_mm: spec.crown_height_mm,
            pavilion_depth_mm: spec.pavilion_depth_mm,
            profile,
        };
        girdle.validate()?;
        Ok(girdle)
    }

    /// A round girdle, the shape every Head Builder mode supports.
    pub fn round(
        gem_id: impl Into<String>,
        center: Point3,
        diameter_mm: f64,
        normal: Point3,
    ) -> Result<Self> {
        if !diameter_mm.is_finite() || diameter_mm <= 0.0 {
            return Err(Error::Invalid("gem diameter"));
        }
        finite(center)?;
        let normal = unit(normal)?;
        let radius = diameter_mm * 0.5;
        let segments = 64usize;
        let frame = Frame::from_axis(center, normal, [1.0, 0.0, 0.0])?;
        let outline = (0..segments)
            .map(|index| {
                let theta = std::f64::consts::TAU * index as f64 / segments as f64;
                frame.point([radius * theta.cos(), radius * theta.sin(), 0.0])
            })
            .collect::<Result<Vec<_>>>()?;
        let girdle = 0.2_f64.min(radius * 0.05);
        let girdle_half = girdle * 0.5;
        let pavilion = radius * 0.8;
        let profile = vec![
            [0.0, -girdle_half - pavilion],
            [radius, -girdle_half],
            [radius, girdle_half],
            [radius * 0.55, girdle_half + radius * 0.35],
            [0.0, girdle_half + radius * 0.35],
        ];
        let girdle = Self {
            gem_id: gem_id.into(),
            center,
            normal,
            outline,
            girdle_top_mm: girdle_half,
            girdle_bottom_mm: girdle_half,
            crown_height_mm: radius * 0.35,
            pavilion_depth_mm: pavilion,
            profile,
        };
        girdle.validate()?;
        Ok(girdle)
    }

    pub fn validate(&self) -> Result<()> {
        if self.gem_id.trim().is_empty() {
            return Err(Error::Missing("gem id"));
        }
        finite(self.center)?;
        unit(self.normal)?;
        if self.outline.len() < 3 {
            return Err(Error::Invalid("gem girdle outline"));
        }
        for point in &self.outline {
            finite(*point)?;
            let plane = crate::settings::dot(crate::settings::sub(*point, self.center), self.normal);
            if plane.abs() > 1.0e-6 {
                return Err(Error::Invalid("gem girdle plane"));
            }
        }
        if !self.girdle_top_mm.is_finite()
            || !self.girdle_bottom_mm.is_finite()
            || !self.crown_height_mm.is_finite()
            || !self.pavilion_depth_mm.is_finite()
            || self.pavilion_depth_mm <= 0.0
        {
            return Err(Error::Invalid("gem girdle height"));
        }
        if self.profile.len() < 3 {
            return Err(Error::Invalid("gem profile"));
        }
        Ok(())
    }

    /// Shortest girdle half-width, measured over the outline vertices **and** the edge midpoints:
    /// a faceted outline is inscribed in its profile circle, so the metal may touch the inscribed
    /// radius but not the circumscribed one.
    pub fn radius_mm(&self) -> f64 {
        let mut smallest = f64::INFINITY;
        let count = self.outline.len();
        let radial = |point: Point3| {
            let offset = crate::settings::sub(point, self.center);
            let in_plane = crate::settings::sub(
                offset,
                crate::settings::mul(self.normal, crate::settings::dot(offset, self.normal)),
            );
            norm(in_plane)
        };
        for index in 0..count {
            let next = (index + 1) % count;
            smallest = smallest.min(radial(self.outline[index]));
            smallest = smallest.min(radial(crate::settings::lerp(
                self.outline[index],
                self.outline[next],
                0.5,
            )));
        }
        if smallest.is_finite() { smallest } else { 0.0 }
    }

    /// The factor between the gem profile's widest radius and the actual girdle outline. A
    /// faceted outline is inscribed in the profile circle, so the metal may sit on the inscribed
    /// radius without entering the stone.
    fn profile_scale(&self) -> f64 {
        let widest = self
            .profile
            .iter()
            .map(|point| point[0])
            .fold(0.0_f64, f64::max);
        let inset = self.radius_mm();
        if widest > 1.0e-9 && inset > 0.0 {
            (inset / widest).min(1.0)
        } else {
            1.0
        }
    }

    /// Radius of the gem surface at a local axial height (mm, girdle mid-plane = 0).
    pub fn surface_radius_at(&self, axial_mm: f64) -> f64 {
        self.surface_radius_raw(axial_mm) * self.profile_scale()
    }

    fn surface_radius_raw(&self, axial_mm: f64) -> f64 {
        if !axial_mm.is_finite() || self.profile.len() < 2 {
            return 0.0;
        }
        let mut sorted = self.profile.clone();
        sorted.sort_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal));
        if axial_mm <= sorted[0][1] || axial_mm >= sorted[sorted.len() - 1][1] {
            return if axial_mm < sorted[0][1] { 0.0 } else { sorted[sorted.len() - 1][0] };
        }
        for pair in sorted.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if axial_mm >= a[1] && axial_mm <= b[1] {
                let span = b[1] - a[1];
                if span.abs() <= 1.0e-12 {
                    return a[0].max(b[0]);
                }
                let t = (axial_mm - a[1]) / span;
                return a[0] + (b[0] - a[0]) * t;
            }
        }
        sorted[sorted.len() - 1][0]
    }

    /// The culet height below the girdle mid-plane.
    pub fn culet_mm(&self) -> f64 {
        self.profile
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min)
    }

    /// Total girdle outline length, in mm.
    pub fn outline_length_mm(&self) -> f64 {
        let count = self.outline.len();
        (0..count)
            .map(|index| {
                let next = (index + 1) % count;
                norm(crate::settings::sub(self.outline[next], self.outline[index]))
            })
            .sum()
    }

    /// Point at a fraction of the outline length, `0.0..1.0`.
    pub fn point_at_fraction(&self, fraction: f64) -> Result<Point3> {
        let samples = sample_outline(&self.outline, 256)?;
        let index = ((fraction.rem_euclid(1.0)) * samples.len() as f64).floor() as usize;
        Ok(samples[index.min(samples.len() - 1)])
    }

    /// Outward in-plane unit direction at an outline point.
    pub fn outward_at(&self, point: Point3) -> Result<Point3> {
        let offset = crate::settings::sub(point, self.center);
        let in_plane = crate::settings::sub(offset, crate::settings::mul(self.normal, crate::settings::dot(offset, self.normal)));
        unit(in_plane)
    }

    /// A closed ring of sweep stations around the outline at `outward_offset_mm`.
    pub fn ring_stations(&self, count: usize, outward_offset_mm: f64) -> Result<Vec<Station>> {
        if !(3..=MAX_INSTANCES).contains(&count) {
            return Err(Error::Invalid("head ring stations"));
        }
        let samples = sample_outline(&self.outline, count)?;
        samples
            .into_iter()
            .map(|point| {
                let outward = self.outward_at(point)?;
                let origin = crate::settings::add(point, crate::settings::mul(outward, outward_offset_mm));
                Station::new(origin, outward, self.normal)
            })
            .collect()
    }

    /// An open run of sweep stations between two outline fractions.
    pub fn arc_stations(
        &self,
        start: f64,
        end: f64,
        count: usize,
        outward_offset_mm: f64,
    ) -> Result<Vec<Station>> {
        if !(2..=MAX_INSTANCES).contains(&count) {
            return Err(Error::Invalid("head arc stations"));
        }
        if !start.is_finite() || !end.is_finite() || (end - start).abs() <= 1.0e-9 {
            return Err(Error::Invalid("head bezel arc"));
        }
        let samples = sample_outline(&self.outline, sample_budget(&self.outline, 256))?;
        let length = self.outline_length_mm();
        let mut stations = Vec::with_capacity(count);
        for index in 0..count {
            let t = index as f64 / (count - 1) as f64;
            let fraction = start + (end - start) * t;
            let target = fraction.rem_euclid(1.0) * length;
            let point = point_at_length(&samples, target)?;
            let outward = self.outward_at(point)?;
            let origin = crate::settings::add(point, crate::settings::mul(outward, outward_offset_mm));
            stations.push(Station::new(origin, outward, self.normal)?);
        }
        Ok(stations)
    }
}

/// The three per-module profile libraries of the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Prong,
    Rail,
    Edge,
}

impl ProfileKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Prong => "Prong Profile",
            Self::Rail => "Rail Profile",
            Self::Edge => "Edge Profile",
        }
    }
}

/// One library profile: a closed section in unit coordinates, radial `0..=1` and axial `0..=1`.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub name: String,
    pub points: Vec<(f64, f64)>,
}

impl Profile {
    pub fn width(&self) -> f64 {
        self.points.iter().map(|p| p.0).fold(0.0, f64::max)
    }

    pub fn height(&self) -> f64 {
        self.points.iter().map(|p| p.1).fold(0.0, f64::max)
    }

    /// A profile is usable only when it is finite, closed as a section polygon and has a
    /// positive width and height.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Missing("profile name"));
        }
        if self.points.len() < 3 {
            return Err(Error::Invalid("profile points"));
        }
        for (r, z) in &self.points {
            if !r.is_finite() || !z.is_finite() || *r < -1.0e-9 {
                return Err(Error::Invalid("profile point"));
            }
        }
        if self.width() <= 1.0e-9 || self.height() <= 1.0e-9 {
            return Err(Error::Invalid("profile size"));
        }
        Ok(())
    }

    /// Scale the unit profile into millimetres.
    pub fn scaled(&self, width_mm: f64, height_mm: f64) -> Result<Vec<(f64, f64)>> {
        self.validate()?;
        if !width_mm.is_finite() || !height_mm.is_finite() || width_mm <= 0.0 || height_mm <= 0.0 {
            return Err(Error::Invalid("profile scale"));
        }
        Ok(self
            .points
            .iter()
            .map(|(r, z)| (r * width_mm, z * height_mm))
            .collect())
    }

    /// Take an explicit section as a profile. The payload carries the exact points, so a user
    /// (Bezier) profile rebuilds without needing a session library.
    pub fn from_section(name: &str, points: &[(f64, f64)]) -> Result<Profile> {
        let profile = Profile {
            name: name.trim().to_owned(),
            points: points.to_vec(),
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Sample a closed cubic Bezier through `control` into a unit section. This is the data path
    /// of the manual's editable profile library: a Bezier profile editor (UI owner) writes the
    /// control polygon, and the sampled section keeps the builder deterministic and portable.
    pub fn from_bezier(name: &str, control: &[(f64, f64)], samples: usize) -> Result<Profile> {
        if !(3..=64).contains(&control.len()) {
            return Err(Error::Invalid("profile bezier control"));
        }
        if !(8..=512).contains(&samples) {
            return Err(Error::Invalid("profile bezier samples"));
        }
        if control
            .iter()
            .any(|(r, z)| !r.is_finite() || !z.is_finite() || *r < -1.0e-9)
        {
            return Err(Error::Invalid("profile bezier point"));
        }
        let count = control.len();
        let per_segment = (samples / count).max(2);
        let mut points = Vec::with_capacity(per_segment * count);
        for index in 0..count {
            let p0 = control[(index + count - 1) % count];
            let p1 = control[index];
            let p2 = control[(index + 1) % count];
            let p3 = control[(index + 2) % count];
            let b1 = (
                p1.0 + (p2.0 - p0.0) / 6.0,
                p1.1 + (p2.1 - p0.1) / 6.0,
            );
            let b2 = (
                p2.0 - (p3.0 - p1.0) / 6.0,
                p2.1 - (p3.1 - p1.1) / 6.0,
            );
            for step in 0..per_segment {
                let t = step as f64 / per_segment as f64;
                points.push(bezier_point(p1, b1, b2, p2, t));
            }
        }
        if points.len() > 512 {
            return Err(Error::Invalid("profile bezier samples"));
        }
        let min_r = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let max_r = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let min_z = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let max_z = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        if (max_r - min_r) <= 1.0e-9 || (max_z - min_z) <= 1.0e-9 {
            return Err(Error::Invalid("profile bezier size"));
        }
        // The library convention is a half-width section: radial `0..=0.5` (the builder doubles it
        // into the prong diameter) and axial `0..=1`.
        let points: Vec<(f64, f64)> = points
            .into_iter()
            .map(|(r, z)| {
                (
                    (r - min_r) / (max_r - min_r) * 0.5,
                    (z - min_z) / (max_z - min_z),
                )
            })
            .collect();
        Profile::from_section(name, &points)
    }
}

fn bezier_point(
    p1: (f64, f64),
    b1: (f64, f64),
    b2: (f64, f64),
    p2: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    (
        w0 * p1.0 + w1 * b1.0 + w2 * b2.0 + w3 * p2.0,
        w0 * p1.1 + w1 * b1.1 + w2 * b2.1 + w3 * p2.1,
    )
}

/// The read-only built-in profile library (the manual's "profile library" is editable; OM9
/// exposes the built-ins plus user records through the same typed list).
pub struct ProfileLibrary;

impl ProfileLibrary {
    pub fn builtin(kind: ProfileKind) -> Vec<Profile> {
        let points: &[(&str, &[(f64, f64)])] = match kind {
            ProfileKind::Prong => &[
                ("Round", &[(0.0, 0.0), (0.5, 0.02), (0.5, 0.78), (0.36, 0.94), (0.0, 1.0)]),
                ("Flat", &[(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (0.0, 1.0)]),
                ("Pointed", &[(0.0, 0.0), (0.5, 0.0), (0.5, 0.88), (0.0, 1.0)]),
                ("Cannon", &[(0.0, 0.0), (0.4, 0.0), (0.5, 0.12), (0.5, 0.82), (0.3, 1.0), (0.0, 1.0)]),
                ("Dome", &[(0.0, 0.0), (0.5, 0.0), (0.5, 0.82), (0.38, 0.96), (0.0, 1.0)]),
            ],
            ProfileKind::Rail => &[
                ("Round", &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.5), (0.7, 1.0), (0.3, 1.0), (0.0, 0.5)]),
                ("Square", &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
                ("HalfRound", &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.5, 0.7), (0.0, 1.0)]),
                ("Bead", &[(0.0, 0.0), (1.0, 0.35), (1.0, 0.65), (0.0, 1.0)]),
            ],
            ProfileKind::Edge => &[
                ("Chamfer", &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.3), (0.3, 1.0), (0.0, 1.0)]),
                ("Round", &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.5), (0.6, 0.9), (0.0, 1.0)]),
                ("Step", &[(0.0, 0.0), (1.0, 0.0), (1.0, 0.45), (0.62, 0.45), (0.62, 1.0), (0.0, 1.0)]),
                ("Knife", &[(0.0, 0.0), (1.0, 0.5), (1.0, 0.75), (0.0, 1.0)]),
            ],
        };
        points
            .iter()
            .map(|(name, points)| Profile {
                name: (*name).to_string(),
                points: points.to_vec(),
            })
            .collect()
    }

    pub fn find(kind: ProfileKind, name: &str) -> Result<Profile> {
        let wanted = name.trim().to_ascii_lowercase();
        Self::builtin(kind)
            .into_iter()
            .find(|profile| profile.name.to_ascii_lowercase() == wanted)
            .ok_or(Error::Invalid(match kind {
                ProfileKind::Prong => "prong profile",
                ProfileKind::Rail => "rail profile",
                ProfileKind::Edge => "edge profile",
            }))
    }
}

/// Resolve a profile by name, or take the explicit section the parameters carry. An explicit
/// section is how a user-edited (Bezier) profile reaches the builder and the payload.
pub fn profile_section(
    kind: ProfileKind,
    name: &str,
    section: Option<&[(f64, f64)]>,
) -> Result<Profile> {
    match section {
        Some(points) if !points.is_empty() => {
            let label = if name.trim().is_empty() {
                "Bezier"
            } else {
                name
            };
            Profile::from_section(label, points)
        }
        _ => ProfileLibrary::find(kind, name),
    }
}

/// The Head Builder shape: a prong basket or the half-bezel band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadMode {
    Basket,
    HalfBezel,
}

impl HeadMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Basket => "Basket",
            Self::HalfBezel => "Half Bezel",
        }
    }
}

/// The half-bezel band that replaces the prongs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HalfBezelParams {
    /// Start of the band around the outline, degrees from the local +X axis.
    pub position_deg: f64,
    /// Angular span of the band, degrees; the manual calls it the prong circumference.
    pub span_deg: f64,
    /// Radial thickness of the band, mm.
    pub thickness_mm: f64,
    /// Height above the girdle, mm.
    pub height_mm: f64,
    /// Seat angle at the girdle, degrees.
    pub seat_angle_deg: f64,
}

impl Default for HalfBezelParams {
    fn default() -> Self {
        Self {
            position_deg: 0.0,
            span_deg: 90.0,
            thickness_mm: 0.6,
            height_mm: 1.2,
            seat_angle_deg: 15.0,
        }
    }
}

/// The typed Head Builder parameters, level 1 of the manual.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HeadParams {
    pub mode: HeadMode,
    /// Number of prongs of the basket (ignored in half-bezel mode).
    pub prongs: u8,
    /// Built-in prong profile name (ignored in half-bezel mode).
    pub prong_profile: String,
    pub rail_profile: String,
    /// Gallery rails, `0..=4`.
    pub rail_count: u8,
    /// Prong height from the girdle to the top, before the dome.
    pub height_above_girdle_mm: f64,
    /// How far the base of the head sits below the girdle, mm (positive = down).
    pub head_drop_mm: f64,
    /// Head taper, degrees.
    pub head_angle_x_deg: f64,
    /// Top angle of the prong dome, degrees.
    pub top_angle_deg: f64,
    /// Prong diameter at the girdle, mm.
    pub prong_size_mm: f64,
    /// Share of the prong radius allowed on the stone (`0.0..=0.5`).
    pub prong_nudge: f64,
    /// Rotation of the whole prong set, degrees.
    pub prong_angle_deg: f64,
    pub dome: bool,
    /// Gallery rail drop below the girdle, mm.
    pub rail_drop_mm: f64,
    /// Radial offset of the rails from the girdle outline, mm.
    pub rail_x_offset_mm: f64,
    /// Radial thickness of the rails, mm.
    pub rail_size_mm: f64,
    /// Half-bezel band parameters (used when `mode == HalfBezel`).
    pub half_bezel: HalfBezelParams,
    /// An explicit prong section (unit box), e.g. a Bezier profile from the leaf editor.
    #[serde(default)]
    pub profile_section: Option<Vec<(f64, f64)>>,
    /// An explicit rail section (unit box); overrides `rail_profile` when present.
    #[serde(default)]
    pub rail_profile_section: Option<Vec<(f64, f64)>>,
}

impl Default for HeadParams {
    fn default() -> Self {
        Self {
            mode: HeadMode::Basket,
            prongs: 4,
            prong_profile: "Round".to_string(),
            rail_profile: "Round".to_string(),
            rail_count: 2,
            height_above_girdle_mm: 1.6,
            head_drop_mm: 0.9,
            head_angle_x_deg: 12.0,
            top_angle_deg: 30.0,
            prong_size_mm: 0.9,
            prong_nudge: DEFAULT_HEAD_NUDGE,
            prong_angle_deg: 0.0,
            dome: true,
            rail_drop_mm: 1.4,
            rail_x_offset_mm: 0.0,
            rail_size_mm: 0.55,
            half_bezel: HalfBezelParams::default(),
            profile_section: None,
            rail_profile_section: None,
        }
    }
}

impl HeadParams {
    pub fn basket(prongs: u8) -> Self {
        Self {
            prongs,
            ..Self::default()
        }
    }

    pub fn half_bezel(span_deg: f64) -> Self {
        Self {
            mode: HeadMode::HalfBezel,
            half_bezel: HalfBezelParams {
                span_deg,
                ..HalfBezelParams::default()
            },
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !(3..=MAX_HEAD_PRONGS).contains(&self.prongs) {
            return Err(Error::Invalid("head prong count"));
        }
        if self.mode == HeadMode::Basket {
            profile_section(
                ProfileKind::Prong,
                &self.prong_profile,
                self.profile_section.as_deref(),
            )?;
        }
        if self.rail_count > MAX_HEAD_RAILS {
            return Err(Error::Invalid("head rail count"));
        }
        if self.rail_count > 0 {
            profile_section(
                ProfileKind::Rail,
                &self.rail_profile,
                self.rail_profile_section.as_deref(),
            )?;
        }
        if !self.height_above_girdle_mm.is_finite() || self.height_above_girdle_mm < MIN_HEAD_HEIGHT_MM {
            return Err(Error::Invalid("head height above girdle"));
        }
        if !self.head_drop_mm.is_finite() || self.head_drop_mm < 0.0 {
            return Err(Error::Invalid("head drop"));
        }
        if !self.head_angle_x_deg.is_finite() || !(0.0..=60.0).contains(&self.head_angle_x_deg) {
            return Err(Error::Invalid("head angle x"));
        }
        if !self.top_angle_deg.is_finite() || !(0.0..=90.0).contains(&self.top_angle_deg) {
            return Err(Error::Invalid("head top angle"));
        }
        if !self.prong_size_mm.is_finite() || self.prong_size_mm < MIN_HEAD_ELEMENT_MM {
            return Err(Error::Invalid("head prong size"));
        }
        if !self.prong_nudge.is_finite() || !(0.0..=0.5).contains(&self.prong_nudge) {
            return Err(Error::Invalid("head prong nudge"));
        }
        if !self.prong_angle_deg.is_finite() {
            return Err(Error::Invalid("head prong angle"));
        }
        if !self.rail_drop_mm.is_finite() || self.rail_drop_mm < 0.0 {
            return Err(Error::Invalid("head rail drop"));
        }
        if !self.rail_x_offset_mm.is_finite() {
            return Err(Error::Invalid("head rail offset"));
        }
        if !self.rail_size_mm.is_finite() || self.rail_size_mm < MIN_HEAD_ELEMENT_MM {
            return Err(Error::Invalid("head rail size"));
        }
        if self.mode == HeadMode::HalfBezel {
            let band = &self.half_bezel;
            if !band.position_deg.is_finite() || !band.span_deg.is_finite() {
                return Err(Error::Invalid("half bezel angle"));
            }
            if band.span_deg <= 0.0 || band.span_deg > 360.0 {
                return Err(Error::Invalid("half bezel span"));
            }
            if !band.thickness_mm.is_finite() || band.thickness_mm < MIN_HEAD_ELEMENT_MM {
                return Err(Error::Invalid("half bezel thickness"));
            }
            if !band.height_mm.is_finite() || band.height_mm < MIN_HEAD_HEIGHT_MM {
                return Err(Error::Invalid("half bezel height"));
            }
            if !band.seat_angle_deg.is_finite() || !(0.0..=89.0).contains(&band.seat_angle_deg) {
                return Err(Error::Invalid("half bezel seat angle"));
            }
        }
        Ok(())
    }
}

/// One prong of a basket head.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadProng {
    pub id: String,
    /// 0-based position around the outline.
    pub index: usize,
    pub frame: Frame,
    pub height_mm: f64,
    pub diameter_mm: f64,
    pub mesh: Mesh,
    pub volume_mm3: f64,
}

/// One gallery rail of a head.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadRail {
    pub id: String,
    pub index: usize,
    pub drop_mm: f64,
    pub size_mm: f64,
    pub mesh: Mesh,
    pub volume_mm3: f64,
}

/// The half-bezel band.
#[derive(Debug, Clone, PartialEq)]
pub struct HalfBezelBand {
    pub id: String,
    pub start_deg: f64,
    pub span_deg: f64,
    pub thickness_mm: f64,
    pub height_mm: f64,
    pub mesh: Mesh,
    pub volume_mm3: f64,
}

/// A built head: prongs, rails, optional half-bezel band, all measured.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadSetting {
    pub gem_id: String,
    pub mode: HeadMode,
    pub prongs: Vec<HeadProng>,
    pub rails: Vec<HeadRail>,
    pub half_bezel: Option<HalfBezelBand>,
    /// True when the prong profile was ignored because the half-bezel shape overrides it.
    pub prong_profile_ignored: bool,
    pub volume_mm3: f64,
}

impl HeadSetting {
    pub fn bodies(&self) -> impl Iterator<Item = &Mesh> {
        self.prongs
            .iter()
            .map(|p| &p.mesh)
            .chain(self.rails.iter().map(|r| &r.mesh))
            .chain(self.half_bezel.iter().map(|b| &b.mesh))
    }

    /// The prong ids in order; the document payload keeps them stable across an edit.
    pub fn prong_ids(&self) -> Vec<&str> {
        self.prongs.iter().map(|p| p.id.as_str()).collect()
    }

    pub fn summary(&self) -> String {
        match self.mode {
            HeadMode::Basket => format!(
                "Head basket: {} prongs, {} rail{}",
                self.prongs.len(),
                self.rails.len(),
                if self.rails.len() == 1 { "" } else { "s" }
            ),
            HeadMode::HalfBezel => format!(
                "Head half bezel: {:.0} deg span, {} rail{}",
                self.half_bezel.as_ref().map(|b| b.span_deg).unwrap_or(0.0),
                self.rails.len(),
                if self.rails.len() == 1 { "" } else { "s" }
            ),
        }
    }
}

/// Build the head around one girdle.
pub fn build(girdle: &GemGirdle, params: &HeadParams) -> Result<HeadSetting> {
    girdle.validate()?;
    params.validate()?;
    if params.mode == HeadMode::HalfBezel {
        let band = build_half_bezel(girdle, &params.half_bezel)?;
        let mut setting = HeadSetting {
            gem_id: girdle.gem_id.clone(),
            mode: params.mode,
            prongs: Vec::new(),
            rails: Vec::new(),
            half_bezel: Some(band),
            prong_profile_ignored: true,
            volume_mm3: 0.0,
        };
        setting.rails = build_rails(girdle, params)?;
        setting.volume_mm3 = setting.bodies().map(volume).sum();
        validate_clearance(girdle, &setting)?;
        return Ok(setting);
    }
    let prongs = build_prongs(girdle, params)?;
    let rails = build_rails(girdle, params)?;
    let mut setting = HeadSetting {
        gem_id: girdle.gem_id.clone(),
        mode: params.mode,
        prongs,
        rails,
        half_bezel: None,
        prong_profile_ignored: false,
        volume_mm3: 0.0,
    };
    setting.volume_mm3 = setting.bodies().map(volume).sum();
    validate_clearance(girdle, &setting)?;
    Ok(setting)
}

/// The head invariant: no metal vertex may sit inside the gem surface. Below the girdle that
/// means outside the pavilion cone; above it, outside the crown. The girdle band itself is the
/// only place the metal is allowed to touch the stone.
pub fn validate_clearance(girdle: &GemGirdle, setting: &HeadSetting) -> Result<()> {
    for mesh in setting.bodies() {
        for vertex in &mesh.vertices {
            let offset = crate::settings::sub(*vertex, girdle.center);
            let axial = crate::settings::dot(offset, girdle.normal);
            if axial >= girdle.girdle_top_mm {
                continue;
            }
            let radial = norm(crate::settings::sub(
                offset,
                crate::settings::mul(girdle.normal, axial),
            ));
            let surface = girdle.surface_radius_at(axial);
            if radial < surface - PAVILION_CLEARANCE_MM {
                return Err(Error::Invalid("head metal inside pavilion"));
            }
        }
    }
    Ok(())
}

fn volume(mesh: &Mesh) -> f64 {
    inspect_solid(mesh).map(|report| report.volume_mm3).unwrap_or(0.0)
}

fn build_prongs(girdle: &GemGirdle, params: &HeadParams) -> Result<Vec<HeadProng>> {
    let profile = profile_section(
        ProfileKind::Prong,
        &params.prong_profile,
        params.profile_section.as_deref(),
    )?;
    let count = params.prongs as usize;
    let mut prongs = Vec::with_capacity(count);
    for index in 0..count {
        let fraction = index as f64 / count as f64 + params.prong_angle_deg / 360.0;
        let point = girdle.point_at_fraction(fraction)?;
        let outward = girdle.outward_at(point)?;
        let base_z = -params.head_drop_mm;
        let gem_radius = girdle.surface_radius_at(base_z);
        let center_offset = norm(crate::settings::sub(point, girdle.center));
        let inner = center_offset + params.prong_nudge * params.prong_size_mm
            - params.prong_size_mm * 0.5;
        if inner < gem_radius - PAVILION_CLEARANCE_MM {
            return Err(Error::Invalid("head prong through pavilion"));
        }
        if base_z < girdle.culet_mm() + PAVILION_CLEARANCE_MM {
            return Err(Error::Invalid("head prong below pavilion"));
        }
        let lean = (params.head_angle_x_deg * 0.5).to_radians();
        let axis = unit(crate::settings::add(
            crate::settings::mul(girdle.normal, lean.cos()),
            crate::settings::mul(outward, lean.sin()),
        ))?;
        let origin = crate::settings::add(
            point,
            crate::settings::add(
                crate::settings::mul(outward, params.prong_nudge * params.prong_size_mm),
                crate::settings::mul(girdle.normal, base_z),
            ),
        );
        let frame = Frame::from_axis(origin, axis, outward)?;
        let prong = make_prong(girdle, params, &profile, index, frame)?;
        prongs.push(prong);
    }
    Ok(prongs)
}

fn make_prong(
    girdle: &GemGirdle,
    params: &HeadParams,
    profile: &Profile,
    index: usize,
    frame: Frame,
) -> Result<HeadProng> {
    let total = params.height_above_girdle_mm + params.head_drop_mm;
    let diameter = params.prong_size_mm;
    let mut points = profile.scaled(diameter * 0.5, total)?;
    if !params.dome {
        // A flat top instead of the dome: keep the widest radius up to the top and cap it.
        points = flat_top(&points);
    }
    let local = crate::settings::revolve_profile(&points, REVOLVE_SEGMENTS)?;
    let mesh = place(&local, frame)?;
    let report = inspect_solid(&mesh)?;
    Ok(HeadProng {
        id: prong_id(&girdle.gem_id, index),
        index,
        frame,
        height_mm: params.height_above_girdle_mm,
        diameter_mm: diameter,
        mesh,
        volume_mm3: report.volume_mm3,
    })
}

fn flat_top(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let last = points[points.len() - 1];
    let mut out: Vec<(f64, f64)> = points[..points.len() - 1].to_vec();
    out.push((last.0.max(1.0e-9), last.1));
    out.push((0.0, last.1));
    out
}

fn build_rails(girdle: &GemGirdle, params: &HeadParams) -> Result<Vec<HeadRail>> {
    if params.rail_count == 0 {
        return Ok(Vec::new());
    }
    let profile = profile_section(
        ProfileKind::Rail,
        &params.rail_profile,
        params.rail_profile_section.as_deref(),
    )?;
    let count = params.rail_count as usize;
    let step = if count > 1 { 0.35 } else { 0.0 };
    let mut rails = Vec::with_capacity(count);
    for index in 0..count {
        let drop = params.rail_drop_mm + index as f64 * step;
        let stations = ring_at_height(girdle, params, drop, params.rail_x_offset_mm)?;
        let section = profile.scaled(params.rail_size_mm, params.rail_size_mm)?;
        let mesh = crate::settings::sweep_section(&stations, &section, true)?;
        let report = inspect_solid(&mesh)?;
        rails.push(HeadRail {
            id: rail_id(&girdle.gem_id, index),
            index,
            drop_mm: drop,
            size_mm: params.rail_size_mm,
            mesh,
            volume_mm3: report.volume_mm3,
        });
    }
    Ok(rails)
}

/// Ring stations at an axial height below the girdle, used by the rails and the seat.
fn ring_at_height(
    girdle: &GemGirdle,
    _params: &HeadParams,
    drop_mm: f64,
    outward_mm: f64,
) -> Result<Vec<Station>> {
    let base = girdle.ring_stations(96, outward_mm)?;
    base.into_iter()
        .map(|station| {
            let origin = crate::settings::add(
                station.origin,
                crate::settings::mul(girdle.normal, -drop_mm),
            );
            Station::new(origin, station.x, girdle.normal)
        })
        .collect()
}

fn build_half_bezel(girdle: &GemGirdle, band: &HalfBezelParams) -> Result<HalfBezelBand> {
    let start = band.position_deg / 360.0;
    let span = band.span_deg / 360.0;
    let stations = girdle.arc_stations(start, start + span, 48, 0.0)?;
    let seat = (band.seat_angle_deg.to_radians().tan() * band.thickness_mm)
        .clamp(0.0, band.height_mm * 0.5);
    let section = vec![
        (0.0, 0.0),
        (band.thickness_mm, 0.0),
        (band.thickness_mm, band.height_mm),
        (0.0, band.height_mm - seat),
    ];
    let mesh = crate::settings::sweep_section(&stations, &section, false)?;
    let report = inspect_solid(&mesh)?;
    Ok(HalfBezelBand {
        id: format!("{}:half-bezel", girdle.gem_id),
        start_deg: band.position_deg,
        span_deg: band.span_deg,
        thickness_mm: band.thickness_mm,
        height_mm: band.height_mm,
        mesh,
        volume_mm3: report.volume_mm3,
    })
}

fn prong_id(gem_id: &str, index: usize) -> String {
    format!("{gem_id}:prong:{:02}", index + 1)
}

fn rail_id(gem_id: &str, index: usize) -> String {
    format!("{gem_id}:rail:{:02}", index + 1)
}

/// Copy a local mesh through a frame.
fn place(mesh: &Mesh, frame: Frame) -> Result<Mesh> {
    let vertices = mesh
        .vertices
        .iter()
        .map(|point| frame.point(*point))
        .collect::<Result<Vec<_>>>()?;
    Ok(Mesh {
        vertices,
        faces: mesh.faces.clone(),
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    })
}

/// The four edit levels of the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadLevel {
    /// Overall control: prongs and rails together.
    Overall,
    /// Grouped prongs/rails.
    Grouped,
    /// Grouped ends.
    Ends,
    /// Individual elements.
    Individual,
}

impl HeadLevel {
    pub fn number(self) -> u8 {
        match self {
            Self::Overall => 1,
            Self::Grouped => 2,
            Self::Ends => 3,
            Self::Individual => 4,
        }
    }

    pub fn from_number(number: u8) -> Result<Self> {
        match number {
            1 => Ok(Self::Overall),
            2 => Ok(Self::Grouped),
            3 => Ok(Self::Ends),
            4 => Ok(Self::Individual),
            _ => Err(Error::Invalid("head level")),
        }
    }
}

/// A settable field of one element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadField {
    HeightAboveGirdle,
    HeadDrop,
    ProngSize,
    ProngAngle,
    RailDrop,
    RailSize,
    RailOffset,
}

/// One element edit at a level.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HeadEdit {
    pub level: HeadLevel,
    /// The edited element at level 4; ignored at the other levels.
    pub element: Option<HeadElement>,
    pub field: HeadField,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadElement {
    Prong(usize),
    Rail(usize),
}

impl HeadEdit {
    fn allowed(&self) -> bool {
        match self.level {
            HeadLevel::Overall => matches!(
                self.field,
                HeadField::HeightAboveGirdle | HeadField::HeadDrop | HeadField::ProngSize | HeadField::ProngAngle
            ),
            HeadLevel::Grouped | HeadLevel::Ends => matches!(
                self.field,
                HeadField::ProngSize
                    | HeadField::ProngAngle
                    | HeadField::RailDrop
                    | HeadField::RailSize
                    | HeadField::RailOffset
            ),
            HeadLevel::Individual => true,
        }
    }
}

/// Apply one level edit. Only the permitted element set is rebuilt; the rest of the head keeps
/// its ids, frames and vertices, which is the manual's "the structure does not drift".
pub fn apply_edit(
    setting: &mut HeadSetting,
    girdle: &GemGirdle,
    params: &mut HeadParams,
    edit: &HeadEdit,
) -> Result<()> {
    if !edit.allowed() {
        return Err(Error::Invalid("head edit level field"));
    }
    if !edit.value.is_finite() {
        return Err(Error::Invalid("head edit value"));
    }
    match edit.field {
        HeadField::HeightAboveGirdle => {
            if edit.value < MIN_HEAD_HEIGHT_MM {
                return Err(Error::Invalid("head height above girdle"));
            }
            params.height_above_girdle_mm = edit.value;
            let profile = ProfileLibrary::find(ProfileKind::Prong, &params.prong_profile)?;
            for prong in &mut setting.prongs {
                prong.height_mm = edit.value;
                *prong = make_prong(girdle, params, &profile, prong.index, prong.frame)?;
            }
        }
        HeadField::HeadDrop => {
            if edit.value < 0.0 {
                return Err(Error::Invalid("head drop"));
            }
            params.head_drop_mm = edit.value;
            let profile = ProfileLibrary::find(ProfileKind::Prong, &params.prong_profile)?;
            for prong in &mut setting.prongs {
                let base = -params.head_drop_mm;
                let outward = prong.frame.x;
                let origin = crate::settings::add(
                    prong.frame.origin,
                    crate::settings::mul(outward, params.prong_nudge * params.prong_size_mm),
                );
                let mut origin = origin;
                origin = crate::settings::add(
                    origin,
                    crate::settings::mul(girdle.normal, base),
                );
                let lean = (params.head_angle_x_deg * 0.5).to_radians();
                let axis = unit(crate::settings::add(
                    crate::settings::mul(girdle.normal, lean.cos()),
                    crate::settings::mul(outward, lean.sin()),
                ))?;
                prong.frame = Frame::from_axis(origin, axis, outward)?;
                *prong = make_prong(girdle, params, &profile, prong.index, prong.frame)?;
            }
        }
        HeadField::ProngSize => {
            if edit.value < MIN_HEAD_ELEMENT_MM {
                return Err(Error::Invalid("head prong size"));
            }
            params.prong_size_mm = edit.value;
            let profile = ProfileLibrary::find(ProfileKind::Prong, &params.prong_profile)?;
            for prong in selected_prongs(setting, edit) {
                let index = setting.prongs[prong].index;
                let frame = setting.prongs[prong].frame;
                let rebuilt = make_prong(girdle, params, &profile, index, frame)?;
                setting.prongs[prong] = rebuilt;
            }
        }
        HeadField::ProngAngle => {
            params.prong_angle_deg = edit.value;
            let angle = params.prong_angle_deg / 360.0;
            let count = setting.prongs.len();
            let profile = ProfileLibrary::find(ProfileKind::Prong, &params.prong_profile)?;
            for prong in 0..count {
                let fraction = prong as f64 / count as f64 + angle;
                let point = girdle.point_at_fraction(fraction)?;
                let outward = girdle.outward_at(point)?;
                let lean = (params.head_angle_x_deg * 0.5).to_radians();
                let axis = unit(crate::settings::add(
                    crate::settings::mul(girdle.normal, lean.cos()),
                    crate::settings::mul(outward, lean.sin()),
                ))?;
                let origin = crate::settings::add(
                    point,
                    crate::settings::add(
                        crate::settings::mul(outward, params.prong_nudge * params.prong_size_mm),
                        crate::settings::mul(girdle.normal, -params.head_drop_mm),
                    ),
                );
                let frame = Frame::from_axis(origin, axis, outward)?;
                setting.prongs[prong] = make_prong(girdle, params, &profile, prong, frame)?;
            }
        }
        HeadField::RailDrop => {
            if edit.value < 0.0 {
                return Err(Error::Invalid("head rail drop"));
            }
            params.rail_drop_mm = edit.value;
            rebuild_rails(setting, girdle, params, edit)?;
        }
        HeadField::RailSize => {
            if edit.value < MIN_HEAD_ELEMENT_MM {
                return Err(Error::Invalid("head rail size"));
            }
            params.rail_size_mm = edit.value;
            rebuild_rails(setting, girdle, params, edit)?;
        }
        HeadField::RailOffset => {
            params.rail_x_offset_mm = edit.value;
            rebuild_rails(setting, girdle, params, edit)?;
        }
    }
    setting.volume_mm3 = setting.bodies().map(volume).sum();
    Ok(())
}

fn selected_prongs(setting: &HeadSetting, edit: &HeadEdit) -> Vec<usize> {
    let count = setting.prongs.len();
    match edit.level {
        HeadLevel::Individual => match edit.element {
            Some(HeadElement::Prong(index)) if index < count => vec![index],
            _ => Vec::new(),
        },
        HeadLevel::Ends => {
            if count <= 1 {
                (0..count).collect()
            } else {
                vec![0, count - 1]
            }
        }
        HeadLevel::Overall | HeadLevel::Grouped => (0..count).collect(),
    }
}

fn rebuild_rails(
    setting: &mut HeadSetting,
    girdle: &GemGirdle,
    params: &HeadParams,
    edit: &HeadEdit,
) -> Result<()> {
    let targets: Vec<usize> = match edit.level {
        HeadLevel::Individual => match edit.element {
            Some(HeadElement::Rail(index)) if index < setting.rails.len() => vec![index],
            _ => Vec::new(),
        },
        HeadLevel::Ends => {
            if setting.rails.len() <= 1 {
                (0..setting.rails.len()).collect()
            } else {
                vec![0, setting.rails.len() - 1]
            }
        }
        HeadLevel::Overall | HeadLevel::Grouped => (0..setting.rails.len()).collect(),
    };
    if targets.is_empty() {
        return Ok(());
    }
    let profile = ProfileLibrary::find(ProfileKind::Rail, &params.rail_profile)?;
    let fresh = build_rails(girdle, params)?;
    for index in targets {
        if let Some(rail) = fresh.iter().find(|rail| rail.index == index) {
            setting.rails[index] = rail.clone();
        }
        profile.validate()?;
    }
    Ok(())
}

/// Seat frame attached to the girdle (the Bezel Builder and the seat owner share it).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeatFrame {
    pub id: String,
    pub gem_id: String,
    pub angle_deg: f64,
    pub origin: Point3,
    /// Girdle plane normal, unit; stays stable across a rebuild.
    pub normal: Point3,
    pub tangent: Point3,
    pub seat_angle_deg: f64,
    pub depth_mm: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeatParams {
    pub count: u8,
    pub seat_angle_deg: f64,
    pub depth_mm: f64,
    pub offset_mm: f64,
}

impl Default for SeatParams {
    fn default() -> Self {
        Self {
            count: 4,
            seat_angle_deg: 15.0,
            depth_mm: 0.15,
            offset_mm: 0.0,
        }
    }
}

impl SeatParams {
    pub fn validate(&self) -> Result<()> {
        if !(3..=MAX_HEAD_PRONGS).contains(&self.count) {
            return Err(Error::Invalid("seat count"));
        }
        if !self.seat_angle_deg.is_finite() || !(0.0..=89.0).contains(&self.seat_angle_deg) {
            return Err(Error::Invalid("seat angle"));
        }
        if !self.depth_mm.is_finite() || self.depth_mm < 0.0 {
            return Err(Error::Invalid("seat depth"));
        }
        if !self.offset_mm.is_finite() {
            return Err(Error::Invalid("seat offset"));
        }
        Ok(())
    }
}

/// One frame per prong position, stable ids and units (mm).
pub fn seat_frames(girdle: &GemGirdle, params: &SeatParams) -> Result<Vec<SeatFrame>> {
    girdle.validate()?;
    params.validate()?;
    let count = params.count as usize;
    let mut frames = Vec::with_capacity(count);
    for index in 0..count {
        let fraction = index as f64 / count as f64;
        let point = girdle.point_at_fraction(fraction)?;
        let outward = girdle.outward_at(point)?;
        let origin = crate::settings::add(
            point,
            crate::settings::add(
                crate::settings::mul(outward, params.offset_mm),
                crate::settings::mul(girdle.normal, -params.depth_mm),
            ),
        );
        let tangent = unit(crate::settings::cross(girdle.normal, outward))?;
        frames.push(SeatFrame {
            id: format!("{}:seat:{:02}", girdle.gem_id, index + 1),
            gem_id: girdle.gem_id.clone(),
            angle_deg: index as f64 * 360.0 / count as f64,
            origin,
            normal: girdle.normal,
            tangent,
            seat_angle_deg: params.seat_angle_deg,
            depth_mm: params.depth_mm,
        });
    }
    Ok(frames)
}

/// The document feature that owns a head and can rebuild it from a changed gem.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HeadFeature {
    pub feature_id: String,
    pub gem_id: String,
    pub params: HeadParams,
    pub revision: u64,
}

impl HeadFeature {
    pub fn new(feature_id: impl Into<String>, gem_id: impl Into<String>, params: HeadParams) -> Self {
        Self {
            feature_id: feature_id.into(),
            gem_id: gem_id.into(),
            params,
            revision: 1,
        }
    }
}

/// A validity report of a built head; `closed`/`positive_volume` are measured, not assumed.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadValidity {
    pub bodies: usize,
    pub closed: bool,
    pub positive_volume: bool,
    pub deterministic: bool,
    /// The gem the head depends on; an update may preserve or replace this id.
    pub dependencies: Vec<String>,
}

pub fn check(setting: &HeadSetting) -> Result<HeadValidity> {
    let bodies: Vec<&Mesh> = setting.bodies().collect();
    if bodies.is_empty() {
        return Err(Error::Invalid("head solid empty"));
    }
    let mut closed = true;
    let mut positive = true;
    for mesh in &bodies {
        let report = inspect_solid(mesh)?;
        closed &= report.closed;
        positive &= report.volume_mm3 > 0.0;
    }
    let mut ids: Vec<&str> = setting.prong_ids();
    ids.extend(setting.rails.iter().map(|rail| rail.id.as_str()));
    ids.sort_unstable();
    ids.dedup();
    let deterministic = ids.len() == setting.prongs.len() + setting.rails.len();
    Ok(HeadValidity {
        bodies: bodies.len(),
        closed,
        positive_volume: positive,
        deterministic,
        dependencies: vec![setting.gem_id.clone()],
    })
}

/// Rebuild the head from its own parameters; identical inputs reproduce identical bodies.
pub fn rebuild(feature: &HeadFeature, girdle: &GemGirdle) -> Result<HeadSetting> {
    if feature.gem_id != girdle.gem_id {
        return Err(Error::Invalid("head gem dependency"));
    }
    build(girdle, &feature.params)
}

/// How an update of the gem dependency affects the head identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadUpdate {
    /// Same gem: stable ids, geometry rebuilt.
    Preserved,
    /// Different gem: the head is replaced and its ids change.
    Replaced,
}

/// Update the head from a changed girdle, preserving ids when the gem did not change.
pub fn update(
    feature: &HeadFeature,
    girdle: &GemGirdle,
) -> Result<(HeadSetting, HeadUpdate)> {
    let preserved = feature.gem_id == girdle.gem_id;
    let mut next = feature.clone();
    next.revision = next.revision.saturating_add(1);
    if !preserved {
        next.gem_id = girdle.gem_id.clone();
    }
    let setting = build(girdle, &next.params)?;
    Ok((setting, if preserved { HeadUpdate::Preserved } else { HeadUpdate::Replaced }))
}

fn sample_budget(outline: &[Point3], max: usize) -> usize {
    outline.len().min(max).max(3)
}

fn sample_outline(outline: &[Point3], count: usize) -> Result<Vec<Point3>> {
    if outline.len() < 3 {
        return Err(Error::Invalid("gem girdle outline"));
    }
    let count = count.max(3);
    let loop_points: Vec<Point3> = outline.to_vec();
    let length = loop_points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let next = (index + 1) % loop_points.len();
            norm(crate::settings::sub(loop_points[next], *point))
        })
        .sum::<f64>();
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("gem girdle length"));
    }
    let mut samples = Vec::with_capacity(count);
    let mut segment = 0usize;
    let mut walked = 0.0;
    for index in 0..count {
        let target = length * index as f64 / count as f64;
        while segment + 1 < loop_points.len() {
            let next = segment + 1;
            let span = norm(crate::settings::sub(loop_points[next], loop_points[segment]));
            if walked + span >= target {
                break;
            }
            walked += span;
            segment = next;
        }
        let next = (segment + 1) % loop_points.len();
        let span = norm(crate::settings::sub(loop_points[next], loop_points[segment]));
        let t = if span <= 1.0e-12 { 0.0 } else { (target - walked) / span };
        samples.push(crate::settings::lerp(
            loop_points[segment],
            loop_points[next],
            t.clamp(0.0, 1.0),
        ));
    }
    Ok(samples)
}

fn point_at_length(samples: &[Point3], target: f64) -> Result<Point3> {
    if samples.len() < 2 {
        return Err(Error::Invalid("gem girdle samples"));
    }
    let mut walked = 0.0;
    for index in 0..samples.len() - 1 {
        let span = norm(crate::settings::sub(samples[index + 1], samples[index]));
        if walked + span >= target {
            let t = if span <= 1.0e-12 { 0.0 } else { (target - walked) / span };
            return Ok(crate::settings::lerp(samples[index], samples[index + 1], t.clamp(0.0, 1.0)));
        }
        walked += span;
    }
    Ok(samples[samples.len() - 1])
}

/// A seat cut: one closed cutter per seat frame plus the gem mesh after the cut.
#[derive(Debug, Clone, PartialEq)]
pub struct SeatCut {
    pub gem_id: String,
    pub cutters: Vec<Mesh>,
    pub volumes_mm3: Vec<f64>,
    pub total_volume_mm3: f64,
    /// The stone with every seat cut applied; the caller replaces the gem body with it.
    pub seated: Mesh,
    pub removed_mm3: f64,
}

impl SeatCut {
    pub fn count(&self) -> usize {
        self.cutters.len()
    }

    pub fn summary(&self) -> String {
        format!(
            "Seat cut: {} seats, {:.3} mm³ removed",
            self.cutters.len(),
            self.removed_mm3
        )
    }
}

/// The closed cutter bodies of the girdle seats. Each seat is a wedge under the girdle, angled by
/// `seat_angle_deg` and `depth_mm` deep, so the prong base sits in the stone instead of on it.
pub fn seat_cutters(girdle: &GemGirdle, params: &SeatParams) -> Result<Vec<Mesh>> {
    girdle.validate()?;
    params.validate()?;
    if params.depth_mm <= 0.0 {
        return Ok(Vec::new());
    }
    let depth = params.depth_mm;
    let length = (depth * 1.5).clamp(0.08, 0.8);
    let shear = params.seat_angle_deg.to_radians().tan() * depth;
    let lip = 0.06;
    let section = vec![
        (lip, lip),
        (lip, -depth),
        (-length - shear, -depth),
        (-length, lip),
    ];
    let width = (depth * 2.0).clamp(0.12, 1.2);
    let count = params.count as usize;
    let mut cutters = Vec::with_capacity(count);
    for index in 0..count {
        let fraction = index as f64 / count as f64 + params.offset_mm / 360.0;
        let point = girdle.point_at_fraction(fraction)?;
        let outward = girdle.outward_at(point)?;
        let tangent = unit(crate::settings::cross(girdle.normal, outward))?;
        let start = crate::settings::sub(point, crate::settings::mul(tangent, width * 0.5));
        let end = crate::settings::add(start, crate::settings::mul(tangent, width));
        let stations = [
            Station::new(start, outward, girdle.normal)?,
            Station::new(end, outward, girdle.normal)?,
        ];
        let mesh = crate::settings::sweep_section(&stations, &section, false)?;
        inspect_solid(&mesh).map_err(|_| Error::Invalid("seat cutter solid"))?;
        cutters.push(mesh);
    }
    Ok(cutters)
}

/// Cut the seats into the stone: subtract every seat cutter from the gem mesh.
pub fn cut_seats(gem: &Mesh, girdle: &GemGirdle, params: &SeatParams) -> Result<SeatCut> {
    let cutters = seat_cutters(girdle, params)?;
    let before = inspect_solid(gem).map_err(|_| Error::Invalid("seat cut gem"))?;
    if cutters.is_empty() {
        return Ok(SeatCut {
            gem_id: girdle.gem_id.clone(),
            cutters,
            volumes_mm3: Vec::new(),
            total_volume_mm3: 0.0,
            seated: gem.clone(),
            removed_mm3: 0.0,
        });
    }
    let mut seated = triangulate_mesh(gem)?;
    let mut volumes = Vec::with_capacity(cutters.len());
    for cutter in &cutters {
        let cutter_volume =
            inspect_solid(cutter).map_err(|_| Error::Invalid("seat cutter solid"))?;
        let cutter_mesh = triangulate_mesh(cutter)?;
        let (cut, _report) = crate::mesh_csg::mesh_boolean(
            &seated,
            &cutter_mesh,
            crate::mesh_csg::BooleanOp::Difference,
            crate::mesh_csg::CsgOptions {
                tolerance: 1.0e-6,
                ..crate::mesh_csg::CsgOptions::default()
            },
        )?;
        let report = inspect_solid(&cut).map_err(|_| Error::Invalid("seat cut result"))?;
        if report.volume_mm3 <= 0.0 {
            return Err(Error::Invalid("seat cut result"));
        }
        seated = cut;
        volumes.push(cutter_volume.volume_mm3);
    }
    let after = inspect_solid(&seated).map_err(|_| Error::Invalid("seat cut result"))?;
    Ok(SeatCut {
        gem_id: girdle.gem_id.clone(),
        total_volume_mm3: volumes.iter().sum(),
        volumes_mm3: volumes,
        seated,
        removed_mm3: (before.volume_mm3 - after.volume_mm3).max(0.0),
        cutters,
    })
}

/// A short human label for the gem shape a head was built on.
pub fn shape_label(shape: GemShape) -> &'static str {
    shape.label()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_gem() -> GemGirdle {
        GemGirdle::round("gem:r1", [0.0, 0.0, 0.0], 6.0, [0.0, 0.0, 1.0]).expect("gem")
    }

    #[test]
    fn a_basket_head_builds_closed_prongs_and_rails() {
        let girdle = round_gem();
        let params = HeadParams::basket(4);
        let head = build(&girdle, &params).expect("head");
        assert_eq!(head.prongs.len(), 4);
        assert_eq!(head.rails.len(), 2);
        assert!(head.prongs.iter().all(|p| p.volume_mm3 > 0.0));
        assert!(head.rails.iter().all(|r| r.volume_mm3 > 0.0));
        assert!(head.summary().contains("4 prongs"));
        let validity = check(&head).expect("validity");
        assert!(validity.closed && validity.positive_volume && validity.deterministic);
    }

    #[test]
    fn a_prong_base_below_the_culet_is_refused_by_name() {
        let girdle = round_gem();
        let mut params = HeadParams::basket(4);
        params.head_drop_mm = 3.5;
        let error = build(&girdle, &params).expect_err("pavilion");
        assert!(format!("{error:?}").contains("head prong below pavilion"));
    }

    #[test]
    fn metal_inside_the_pavilion_is_refused_by_name() {
        let girdle = round_gem();
        // A body sitting on the axis inside the pavilion is not settings metal.
        let inner = crate::settings::revolve_profile(
            &[(0.0, -1.0), (0.2, -1.0), (0.2, -0.5), (0.0, -0.5)],
            16,
        )
        .expect("mesh");
        let head = HeadSetting {
            gem_id: girdle.gem_id.clone(),
            mode: HeadMode::Basket,
            prongs: vec![HeadProng {
                id: "bad".to_string(),
                index: 0,
                frame: Frame::from_axis([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
                    .expect("frame"),
                height_mm: 0.5,
                diameter_mm: 0.4,
                mesh: inner,
                volume_mm3: 0.1,
            }],
            rails: Vec::new(),
            half_bezel: None,
            prong_profile_ignored: false,
            volume_mm3: 0.1,
        };
        let error = validate_clearance(&girdle, &head).expect_err("inside");
        assert!(format!("{error:?}").contains("head metal inside pavilion"));
    }

    #[test]
    fn the_half_bezel_band_replaces_the_prongs_and_ignores_the_profile() {
        let girdle = round_gem();
        let params = HeadParams::half_bezel(120.0);
        let head = build(&girdle, &params).expect("head");
        assert!(head.prongs.is_empty());
        assert!(head.prong_profile_ignored);
        let band = head.half_bezel.as_ref().expect("band");
        assert!((band.span_deg - 120.0).abs() < 1.0e-9);
        assert!(band.volume_mm3 > 0.0);
        assert!(head.summary().contains("half bezel"));
    }

    #[test]
    fn the_seat_cut_removes_stone_under_the_girdle() {
        let spec = GemSpec::round(6.0, 3.72).expect("spec");
        let entity = crate::gem::gem_entity("Gem", &spec, None).expect("gem entity");
        let mesh = match &entity.geometry {
            crate::geometry::Geometry::Mesh(mesh) => mesh.clone(),
            _ => panic!("gem mesh"),
        };
        let girdle =
            GemGirdle::from_spec(&spec, &crate::geometry::Transform::identity()).expect("girdle");
        let mut params = SeatParams::default();
        params.count = 4;
        let cutters = seat_cutters(&girdle, &params).expect("cutters");
        assert_eq!(cutters.len(), 4);
        assert!(cutters.iter().all(|cutter| inspect_solid(cutter).is_ok()));
        let cut = cut_seats(&mesh, &girdle, &params).expect("seat cut");
        let before = inspect_solid(&mesh).expect("before");
        let after = inspect_solid(&cut.seated).expect("after");
        assert!(cut.removed_mm3 > 0.0);
        assert!(after.volume_mm3 < before.volume_mm3);
        assert!((after.volume_mm3 - (before.volume_mm3 - cut.removed_mm3)).abs() < 1.0e-6);
        assert!(cut.summary().contains("4 seats"));
        params.depth_mm = 0.0;
        let none = cut_seats(&mesh, &girdle, &params).expect("no seat cut");
        assert_eq!(none.count(), 0);
        assert_eq!(none.removed_mm3, 0.0);
    }

    #[test]
    fn a_user_bezier_section_drives_the_prongs() {
        let girdle = round_gem();
        let section = Profile::from_bezier(
            "Bezier",
            &[(0.0, 0.0), (0.5, 0.1), (0.55, 0.8), (0.0, 1.0)],
            40,
        );
        let bezier = section.expect("bezier profile");
        let mut params = HeadParams::basket(4);
        params.head_drop_mm = (girdle.pavilion_depth_mm * 0.35).clamp(0.05, 0.9);
        params.rail_drop_mm = (params.head_drop_mm + 0.5).min(girdle.pavilion_depth_mm * 0.9);
        let plain = build(&girdle, &params).expect("plain head");
        params.profile_section = Some(bezier.points.clone());
        let custom = build(&girdle, &params).expect("bezier head");
        assert_eq!(custom.prongs.len(), plain.prongs.len());
        assert!((custom.volume_mm3 - plain.volume_mm3).abs() > 1.0e-6);
        assert!(custom.prongs.iter().all(|p| inspect_solid(&p.mesh).is_ok()));
        params.profile_section = Some(vec![(0.0, 0.0), (0.0, 0.0)]);
        assert!(build(&girdle, &params).is_err());
    }

    #[test]
    fn an_unknown_profile_is_a_typed_error() {
        let girdle = round_gem();
        let mut params = HeadParams::basket(4);
        params.prong_profile = "Unobtainium".to_string();
        let error = build(&girdle, &params).expect_err("profile");
        assert!(format!("{error:?}").contains("prong profile"));
        assert_eq!(ProfileLibrary::builtin(ProfileKind::Prong).len(), 5);
        assert!(ProfileLibrary::find(ProfileKind::Rail, "bead").is_ok());
        assert!(ProfileLibrary::find(ProfileKind::Edge, "chamfer").is_ok());
    }

    #[test]
    fn a_level_four_edit_touches_only_the_selected_prong() {
        let girdle = round_gem();
        let mut params = HeadParams::basket(6);
        let mut head = build(&girdle, &params).expect("head");
        let before: Vec<Point3> = head.prongs[0].mesh.vertices.clone();
        let rail_before = head.rails[0].mesh.vertices.clone();
        let edit = HeadEdit {
            level: HeadLevel::Individual,
            element: Some(HeadElement::Prong(2)),
            field: HeadField::ProngSize,
            value: 1.2,
        };
        apply_edit(&mut head, &girdle, &mut params, &edit).expect("edit");
        assert_eq!(head.prongs[0].mesh.vertices, before);
        assert_eq!(head.rails[0].mesh.vertices, rail_before);
        assert_eq!(head.prongs[2].diameter_mm, 1.2);
        assert!(head.prongs[2].volume_mm3 > 0.0);
    }

    #[test]
    fn an_overall_edit_rebuilds_every_prong() {
        let girdle = round_gem();
        let mut params = HeadParams::basket(4);
        let mut head = build(&girdle, &params).expect("head");
        let edit = HeadEdit {
            level: HeadLevel::Overall,
            element: None,
            field: HeadField::HeightAboveGirdle,
            value: 2.2,
        };
        apply_edit(&mut head, &girdle, &mut params, &edit).expect("edit");
        assert!(head.prongs.iter().all(|p| (p.height_mm - 2.2).abs() < 1.0e-9));
        let refused = HeadEdit {
            level: HeadLevel::Grouped,
            element: None,
            field: HeadField::HeightAboveGirdle,
            value: 1.0,
        };
        let error = apply_edit(&mut head, &girdle, &mut params, &refused).expect_err("level");
        assert!(format!("{error:?}").contains("head edit level field"));
    }

    #[test]
    fn seat_frames_are_stable_and_on_the_girdle() {
        let girdle = round_gem();
        let params = SeatParams::default();
        let first = seat_frames(&girdle, &params).expect("seats");
        let second = seat_frames(&girdle, &params).expect("seats");
        assert_eq!(first, second);
        assert_eq!(first.len(), 4);
        for frame in &first {
            let offset = crate::settings::sub(frame.origin, girdle.center);
            let axial = crate::settings::dot(offset, girdle.normal);
            assert!((axial + params.depth_mm).abs() < 1.0e-6);
            assert!((norm(frame.normal) - 1.0).abs() < 1.0e-9);
        }
        assert!(first[0].id.ends_with("seat:01"));
    }

    #[test]
    fn a_rebuild_is_deterministic_and_a_new_gem_replaces_the_ids() {
        let girdle = round_gem();
        let feature = HeadFeature::new("head:1", "gem:r1", HeadParams::basket(4));
        let first = rebuild(&feature, &girdle).expect("first");
        let second = rebuild(&feature, &girdle).expect("second");
        assert_eq!(first, second);
        let other = GemGirdle::round("gem:r2", [0.0, 0.0, 0.0], 6.0, [0.0, 0.0, 1.0]).expect("gem");
        let (updated, action) = update(&feature, &other).expect("update");
        assert_eq!(action, HeadUpdate::Replaced);
        assert!(updated.prong_ids()[0].starts_with("gem:r2"));
        let (preserved, action) = update(&feature, &girdle).expect("update");
        assert_eq!(action, HeadUpdate::Preserved);
        assert_eq!(preserved.prong_ids(), first.prong_ids());
    }

    #[test]
    fn non_round_girdles_build_and_keep_prong_symmetry() {
        let spec = GemSpec {
            shape: GemShape::Oval,
            cut: crate::gem::GemCut::Diamond,
            length_mm: 8.0,
            width_mm: 6.0,
            girdle_mm: 0.2,
            crown_height_mm: 1.6,
            pavilion_depth_mm: 3.2,
            crown_angle_deg: 34.0,
            facet_count: 8,
            pavilion_rows: 1,
            bulge: 0.0,
            corner_mm: 0.0,
            profile: None,
        };
        let girdle = GemGirdle::from_spec(&spec, &Transform::identity()).expect("girdle");
        let params = HeadParams::basket(8);
        let head = build(&girdle, &params).expect("head");
        assert_eq!(head.prongs.len(), 8);
        let center = girdle.center;
        // Every prong stands on the oval outline: the in-plane distance to the nearest outline
        // sample is zero, and the gem's long/short axes are both represented.
        let samples = sample_outline(&girdle.outline, 512).expect("samples");
        let mut longest = 0.0_f64;
        let mut shortest = f64::INFINITY;
        let nudge = params.prong_nudge * params.prong_size_mm;
        for prong in &head.prongs {
            let offset = crate::settings::sub(prong.frame.origin, center);
            let axial = crate::settings::dot(offset, girdle.normal);
            let in_plane = crate::settings::sub(offset, crate::settings::mul(girdle.normal, axial));
            let world = crate::settings::add(center, in_plane);
            let nearest = samples
                .iter()
                .map(|point| norm(crate::settings::sub(*point, world)))
                .fold(f64::INFINITY, f64::min);
            assert!(
                (nearest - nudge).abs() < 0.02,
                "prong sits {nearest} mm from the outline, expected the {nudge} mm nudge"
            );
            let radial = norm(in_plane);
            longest = longest.max(radial);
            shortest = shortest.min(radial);
        }
        assert!(longest > shortest + 0.5, "oval should not look round: {shortest}..{longest}");
        assert!(checked_closed(&head));
    }

    fn checked_closed(head: &HeadSetting) -> bool {
        head.bodies().all(|mesh| inspect_solid(mesh).is_ok())
    }
}
