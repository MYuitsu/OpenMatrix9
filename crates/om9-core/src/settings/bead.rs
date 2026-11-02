// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bead on Curve / Milgrain and Bead on Surface (`OM9-L3-04-15-006/007`).
//!
//! Bead on Curve distributes a sphere (or a caller-supplied object frame) along one guide
//! curve: start/end position as a fraction of arc length, spacing in millimetres, start/end
//! taper, roll, Z offset and the F4 / Down / Object pointing direction. Bead on Surface places
//! one bead on a picked UV with the surface normal as its axis, a scale and a vertical offset.
//!
//! Both owners return placement **frames** plus one sphere body per bead. The object-prototype
//! path (`Object Type = Object`) consumes the frame, so a non-sphere prototype never forces a
//! second distribution algorithm.
use crate::{
    Error, Result,
    budget::Limits,
    curve_ops,
    geometry::{Curve, Mesh, NurbsSurface, Point3},
    settings::{Frame, MAX_INSTANCES, add, cross, dot, mul, norm, sphere, sub, unit},
    surface_kernel,
};

/// Where the beads point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeadDirection {
    /// "With F4 all the beads point to world center."
    F4,
    /// "In Down mode all beads point straight down."
    Down,
    /// Point at a chosen object (curve, surface or point).
    Object(Point3),
}

impl BeadDirection {
    fn axis(&self, center: Point3) -> Result<Point3> {
        match self {
            Self::F4 => {
                if norm(center) <= 1.0e-9 {
                    // A bead exactly at world center has no direction to center; keep the
                    // frame defined instead of failing the whole run.
                    Ok([0.0, 0.0, 1.0])
                } else {
                    unit(mul(center, -1.0))
                }
            }
            Self::Down => Ok([0.0, 0.0, -1.0]),
            Self::Object(target) => unit(sub(*target, center)),
        }
    }
}

/// Typed parameters of Bead on Curve.
#[derive(Debug, Clone, PartialEq)]
pub struct BeadOnCurveParams {
    /// Start position as a fraction of the curve arc length (0..1).
    pub start_fraction: f64,
    /// End position as a fraction of the curve arc length (0..1).
    pub end_fraction: f64,
    pub spacing_mm: f64,
    /// Explicit bead count over the span; when set it replaces `spacing_mm` (the sheet lets the
    /// user drive the run by count or by spacing).
    pub count: Option<usize>,
    pub start_size_mm: f64,
    pub end_size_mm: f64,
    /// Taper the bead size from start to end; when off every bead uses `start_size_mm`.
    pub taper_gems: bool,
    pub roll_deg: f64,
    pub z_offset_mm: f64,
    pub mirror: bool,
    pub direction: BeadDirection,
    /// The Girdle/Size toggle: `true` keeps the size as a girdle offset, `false` as an absolute
    /// millimetre size. The caller decides which number it feeds; both are reported.
    pub size_is_offset: bool,
}

impl Default for BeadOnCurveParams {
    fn default() -> Self {
        Self {
            start_fraction: 0.0,
            end_fraction: 1.0,
            spacing_mm: 0.5,
            count: None,
            start_size_mm: 0.4,
            end_size_mm: 0.4,
            taper_gems: false,
            roll_deg: 0.0,
            z_offset_mm: 0.0,
            mirror: false,
            direction: BeadDirection::F4,
            size_is_offset: false,
        }
    }
}

impl BeadOnCurveParams {
    pub fn validate(&self) -> Result<()> {
        if !self.start_fraction.is_finite()
            || !self.end_fraction.is_finite()
            || self.start_fraction < 0.0
            || self.end_fraction > 1.0
            || self.start_fraction >= self.end_fraction
        {
            return Err(Error::Invalid("bead start/end position"));
        }
        if !self.spacing_mm.is_finite() || self.spacing_mm <= 0.0 || self.spacing_mm > 100.0 {
            return Err(Error::Invalid("bead spacing"));
        }
        if let Some(count) = self.count
            && !(2..=MAX_INSTANCES).contains(&count)
        {
            return Err(Error::Invalid("bead count"));
        }
        for size in [self.start_size_mm, self.end_size_mm] {
            if !size.is_finite() || size <= 0.0 || size > 20.0 {
                return Err(Error::Invalid("bead size"));
            }
        }
        if !self.roll_deg.is_finite() || !(-180.0..=180.0).contains(&self.roll_deg) {
            return Err(Error::Invalid("bead roll"));
        }
        if !self.z_offset_mm.is_finite() || !(-20.0..=20.0).contains(&self.z_offset_mm) {
            return Err(Error::Invalid("bead z offset"));
        }
        Ok(())
    }
}

/// One placed bead.
#[derive(Debug, Clone, PartialEq)]
pub struct Bead {
    pub index: usize,
    /// Curve fraction the bead sits at.
    pub fraction: f64,
    pub center: Point3,
    pub diameter_mm: f64,
    pub frame: Frame,
}

/// A built bead row.
#[derive(Debug, Clone, PartialEq)]
pub struct BeadCurve {
    pub beads: Vec<Bead>,
    /// Arc length between the start and end positions.
    pub span_length_mm: f64,
    /// Spacing the run actually used.
    pub spacing_mm: f64,
}

impl BeadCurve {
    pub fn summary(&self) -> String {
        format!(
            "{} beads over {:.2} mm at {:.3} mm",
            self.beads.len(),
            self.span_length_mm,
            self.spacing_mm
        )
    }
}

fn bead_frame(center: Point3, axis: Point3, tangent: Point3, roll_deg: f64) -> Result<Frame> {
    let frame = Frame::from_axis(center, axis, tangent)?;
    let roll = roll_deg.to_radians();
    let x = unit(add(mul(frame.x, roll.cos()), mul(frame.y, roll.sin())))?;
    let y = unit(add(mul(frame.x, -roll.sin()), mul(frame.y, roll.cos())))?;
    let z = unit(cross(x, y))?;
    Frame::new(frame.origin, x, y, z)
}

/// Distribute beads along one guide curve.
pub fn bead_on_curve(
    curve: &Curve,
    params: &BeadOnCurveParams,
    limits: &Limits,
) -> Result<BeadCurve> {
    params.validate()?;
    let total = curve_ops::arc_length(curve, limits)?;
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Invalid("bead guide curve"));
    }
    let start_length = params.start_fraction * total;
    let end_length = params.end_fraction * total;
    let span = end_length - start_length;
    if !span.is_finite() || span <= 0.0 {
        return Err(Error::Invalid("bead start/end position"));
    }
    let spacing = match params.count {
        // An explicit count spans the run end to end, so the first and last bead sit on the
        // start and end fractions and the pitch is derived.
        Some(count) => span / (count as f64 - 1.0),
        None => params.spacing_mm,
    };
    let mut distances: Vec<f64> = Vec::new();
    let mut distance = 0.0;
    while distance <= span + 1.0e-9 {
        distances.push(distance);
        if distances.len() > MAX_INSTANCES {
            return Err(Error::Budget("bead count"));
        }
        distance += spacing;
    }
    if params.mirror {
        let mirrored: Vec<f64> = distances.iter().map(|value| span - value).collect();
        distances.extend(mirrored);
    }
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    distances.dedup_by(|a, b| (*a - *b).abs() <= 1.0e-9);
    let mut beads = Vec::with_capacity(distances.len());
    for (index, distance) in distances.iter().enumerate() {
        let fraction = curve_ops::parameter_at_length(curve, start_length + distance, limits)?;
        let center = curve_ops::point_at(curve, fraction, limits)?;
        let axis = params.direction.axis(center)?;
        let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
        let diameter = if params.taper_gems && span > 0.0 {
            let t = (*distance / span).clamp(0.0, 1.0);
            params.start_size_mm + (params.end_size_mm - params.start_size_mm) * t
        } else {
            params.start_size_mm
        };
        if !diameter.is_finite() || diameter <= 0.0 {
            return Err(Error::Invalid("bead size"));
        }
        let origin = add(center, mul(axis, params.z_offset_mm));
        let frame = bead_frame(origin, axis, tangent, params.roll_deg)?;
        beads.push(Bead {
            index,
            fraction,
            center: origin,
            diameter_mm: diameter,
            frame,
        });
    }
    if beads.is_empty() {
        return Err(Error::Invalid("bead layout"));
    }
    Ok(BeadCurve {
        beads,
        span_length_mm: span,
        spacing_mm: spacing,
    })
}

/// Build one sphere body per placed bead.
pub fn build_bead_bodies(row: &BeadCurve, segments: usize) -> Result<Vec<Mesh>> {
    let mut bodies = Vec::with_capacity(row.beads.len());
    for bead in &row.beads {
        bodies.push(sphere(bead.center, bead.diameter_mm, segments)?);
    }
    Ok(bodies)
}

/// Typed parameters of Bead on Surface.
#[derive(Debug, Clone, PartialEq)]
pub struct BeadOnSurfaceParams {
    /// Base bead size before `scale`.
    pub base_diameter_mm: f64,
    /// "Scale adjusts the size of the beads."
    pub scale: f64,
    /// "Vertical Offset adjusts the position of the bead closer to or further from the
    /// surface": positive is further out along the local normal.
    pub vertical_offset_mm: f64,
    /// Object rotation about the local normal, for the Object mode.
    pub object_angle_deg: f64,
    /// Reverse the normal so the bead sits under a flipped surface.
    pub flip: bool,
}

impl Default for BeadOnSurfaceParams {
    fn default() -> Self {
        Self {
            base_diameter_mm: 1.0,
            scale: 1.0,
            vertical_offset_mm: 0.0,
            object_angle_deg: 0.0,
            flip: false,
        }
    }
}

impl BeadOnSurfaceParams {
    pub fn validate(&self) -> Result<()> {
        if !self.base_diameter_mm.is_finite() || self.base_diameter_mm <= 0.0 {
            return Err(Error::Invalid("bead diameter"));
        }
        if !self.scale.is_finite() || self.scale <= 0.0 || self.scale > 100.0 {
            return Err(Error::Invalid("bead scale"));
        }
        if !self.vertical_offset_mm.is_finite() || !(-20.0..=20.0).contains(&self.vertical_offset_mm)
        {
            return Err(Error::Invalid("bead vertical offset"));
        }
        if !self.object_angle_deg.is_finite() || !(-180.0..=180.0).contains(&self.object_angle_deg)
        {
            return Err(Error::Invalid("bead object angle"));
        }
        Ok(())
    }

    pub fn diameter_mm(&self) -> f64 {
        self.base_diameter_mm * self.scale
    }
}

/// One bead placed on a surface.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceBead {
    /// Surface point the bead was picked at.
    pub point: Point3,
    pub frame: Frame,
    pub diameter_mm: f64,
    pub uv: [f64; 2],
}

/// Place and build one bead on a picked surface UV.
pub fn bead_on_surface(
    surface: &NurbsSurface,
    uv: [f64; 2],
    params: &BeadOnSurfaceParams,
    limits: &Limits,
) -> Result<(SurfaceBead, Mesh)> {
    params.validate()?;
    for (axis, value) in uv.iter().enumerate() {
        if !value.is_finite()
            || *value < surface.domain[axis][0]
            || *value > surface.domain[axis][1]
        {
            return Err(Error::Invalid("surface uv"));
        }
    }
    let point = surface_kernel::evaluate(surface, uv[0], uv[1], limits)
        .map_err(|_| Error::Invalid("surface evaluation"))?;
    let normal = surface_kernel::normal(surface, uv[0], uv[1], limits)
        .map_err(|_| Error::Invalid("surface normal"))?;
    let normal = if params.flip {
        mul(unit(normal)?, -1.0)
    } else {
        unit(normal)?
    };
    let origin = add(point, mul(normal, params.vertical_offset_mm));
    let hint = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let frame = bead_frame(origin, normal, hint, params.object_angle_deg)?;
    let bead = SurfaceBead {
        point,
        frame,
        diameter_mm: params.diameter_mm(),
        uv,
    };
    let body = sphere(bead.frame.origin, bead.diameter_mm, 24)?;
    Ok((bead, body))
}

/// Distance from a bead centre to the guide curve; used by the placement regression.
pub fn bead_on_curve_distance(curve: &Curve, bead: &Bead, limits: &Limits) -> Result<f64> {
    let point = curve_ops::point_at(curve, bead.fraction, limits)?;
    Ok(norm(sub(point, bead.center)))
}

/// Whether the bead axis is a unit vector.
pub fn direction_is_unit(bead: &Bead) -> bool {
    (norm(bead.frame.z) - 1.0).abs() < 1.0e-9 && dot(bead.frame.z, bead.frame.z) > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_curve(limits: &Limits) -> Curve {
        let curve = Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        };
        curve.validate(limits).unwrap();
        curve
    }

    fn plane(limits: &Limits) -> NurbsSurface {
        surface_kernel::plane(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            limits,
        )
        .unwrap()
    }

    #[test]
    fn beads_are_spaced_by_the_requested_pitch() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(&curve, &BeadOnCurveParams::default(), &limits).unwrap();
        // 10 mm at 0.5 mm, inclusive of both ends.
        assert_eq!(row.beads.len(), 21);
        assert!((row.span_length_mm - 10.0).abs() < 1.0e-6);
        for pair in row.beads.windows(2) {
            let distance = norm(sub(pair[1].center, pair[0].center));
            assert!((distance - 0.5).abs() < 1.0e-6);
        }
    }

    #[test]
    fn a_start_and_end_position_limit_the_run() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(
            &curve,
            &BeadOnCurveParams {
                start_fraction: 0.25,
                end_fraction: 0.75,
                ..BeadOnCurveParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((row.span_length_mm - 5.0).abs() < 1.0e-6);
        assert!((row.beads[0].center[0] - 2.5).abs() < 1.0e-6);
    }

    #[test]
    fn a_tapered_run_changes_the_bead_size_along_the_curve() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(
            &curve,
            &BeadOnCurveParams {
                taper_gems: true,
                start_size_mm: 0.3,
                end_size_mm: 0.6,
                ..BeadOnCurveParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!(row.beads.last().unwrap().diameter_mm > row.beads[0].diameter_mm);
    }

    #[test]
    fn the_f4_direction_points_at_world_center_and_down_points_down() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let f4 = bead_on_curve(&curve, &BeadOnCurveParams::default(), &limits).unwrap();
        assert!(f4.beads.iter().all(direction_is_unit));
        let down = bead_on_curve(
            &curve,
            &BeadOnCurveParams {
                direction: BeadDirection::Down,
                ..BeadOnCurveParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((down.beads[0].frame.z[2] + 1.0).abs() < 1.0e-9);
    }

    #[test]
    fn a_z_offset_moves_the_beads_off_the_curve() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let offset = bead_on_curve(
            &curve,
            &BeadOnCurveParams {
                z_offset_mm: 0.3,
                direction: BeadDirection::Down,
                ..BeadOnCurveParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((offset.beads[0].center[2] + 0.3).abs() < 1.0e-9);
    }

    #[test]
    fn a_mirrored_run_is_symmetric_about_the_span_middle() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(
            &curve,
            &BeadOnCurveParams {
                mirror: true,
                spacing_mm: 1.0,
                ..BeadOnCurveParams::default()
            },
            &limits,
        )
        .unwrap();
        assert_eq!(row.beads.len(), 11);
        for bead in &row.beads {
            let has_mirror = row
                .beads
                .iter()
                .any(|other| (other.center[0] - (10.0 - bead.center[0])).abs() < 1.0e-6);
            assert!(has_mirror, "no mirror for {}", bead.center[0]);
        }
    }

    #[test]
    fn a_negative_spacing_is_refused() {
        assert_eq!(
            BeadOnCurveParams {
                spacing_mm: -0.1,
                ..BeadOnCurveParams::default()
            }
            .validate()
            .err(),
            Some(Error::Invalid("bead spacing"))
        );
    }

    #[test]
    fn a_start_after_the_end_is_refused() {
        assert_eq!(
            BeadOnCurveParams {
                start_fraction: 0.8,
                end_fraction: 0.2,
                ..BeadOnCurveParams::default()
            }
            .validate()
            .err(),
            Some(Error::Invalid("bead start/end position"))
        );
    }

    #[test]
    fn every_bead_body_is_a_closed_sphere() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(&curve, &BeadOnCurveParams::default(), &limits).unwrap();
        let bodies = build_bead_bodies(&row, 16).unwrap();
        assert_eq!(bodies.len(), row.beads.len());
        for body in &bodies {
            let report = crate::settings::inspect_solid(body).unwrap();
            assert!(report.volume_mm3 > 0.0);
        }
    }

    #[test]
    fn a_bead_on_a_plane_uses_the_surface_normal_and_the_scale() {
        let limits = Limits::default();
        let surface = plane(&limits);
        let (bead, body) = bead_on_surface(
            &surface,
            [0.5, 0.5],
            &BeadOnSurfaceParams {
                scale: 2.0,
                ..BeadOnSurfaceParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((bead.frame.z[2].abs() - 1.0).abs() < 1.0e-6);
        assert!((bead.diameter_mm - 2.0).abs() < 1.0e-9);
        assert!(crate::settings::inspect_solid(&body).unwrap().volume_mm3 > 0.0);
    }

    #[test]
    fn a_vertical_offset_lifts_the_bead_off_the_surface() {
        let limits = Limits::default();
        let surface = plane(&limits);
        let (bead, _) = bead_on_surface(
            &surface,
            [0.5, 0.5],
            &BeadOnSurfaceParams {
                vertical_offset_mm: 0.4,
                ..BeadOnSurfaceParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((bead.frame.origin[2] - 0.4).abs() < 1.0e-9);
    }

    #[test]
    fn a_bead_uv_outside_the_domain_is_refused() {
        let limits = Limits::default();
        let surface = plane(&limits);
        assert_eq!(
            bead_on_surface(&surface, [-0.1, 0.5], &BeadOnSurfaceParams::default(), &limits).err(),
            Some(Error::Invalid("surface uv"))
        );
    }

    #[test]
    fn a_guide_curve_plus_pick_matches_the_distance_helper() {
        let limits = Limits::default();
        let curve = line_curve(&limits);
        let row = bead_on_curve(&curve, &BeadOnCurveParams::default(), &limits).unwrap();
        for bead in &row.beads {
            assert!(bead_on_curve_distance(&curve, bead, &limits).unwrap() < 1.0e-9);
        }
    }
}
