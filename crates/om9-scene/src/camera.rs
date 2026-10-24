// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    geometry::{Point3, finite},
};
use serde::{Deserialize, Serialize};

pub(crate) fn add(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub(crate) fn sub(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn mul(a: Point3, s: f64) -> Point3 {
    a.map(|v| v * s)
}
pub(crate) fn dot(a: Point3, b: Point3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub(crate) fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn unit(a: Point3) -> Result<Point3> {
    finite(a)?;
    let scale = a.iter().copied().map(f64::abs).fold(0., f64::max);
    if scale == 0. {
        return Err(Error::Invalid("zero direction"));
    }
    let a = a.map(|v| v / scale);
    let length = dot(a, a).sqrt();
    let out = a.map(|v| v / length);
    finite(out)?;
    Ok(out)
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    pub origin: [f64; 2],
    pub size: [f64; 2],
}
impl Viewport {
    pub fn validate(self) -> Result<()> {
        if !self.origin.iter().chain(&self.size).all(|v| v.is_finite())
            || self.size.iter().any(|v| *v <= 0.)
        {
            return Err(Error::Invalid("logical viewport rectangle"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Projection {
    Orthographic { vertical_span_mm: f64 },
    Perspective { vertical_fov_rad: f64 },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub eye: Point3,
    pub target: Point3,
    pub up: Point3,
    pub projection: Projection,
    pub near_mm: f64,
    pub far_mm: f64,
    pub revision: u64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projected {
    pub screen: [f64; 2],
    pub depth: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Point3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CPlanePreset {
    Top,
    Front,
    Right,
}

impl CPlanePreset {
    /// Right-handed axes for the world Top, Front and Right construction planes.
    pub fn axes(self) -> (Point3, Point3) {
        match self {
            Self::Top => ([1., 0., 0.], [0., 1., 0.]),
            Self::Front => ([1., 0., 0.], [0., 0., 1.]),
            Self::Right => ([0., 1., 0.], [0., 0., 1.]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CPlane {
    pub origin: Point3,
    pub x: Point3,
    pub y: Point3,
    pub revision: u64,
}
impl CPlane {
    pub fn world_xy() -> Self {
        Self {
            origin: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            revision: 1,
        }
    }
    /// The named world construction plane used by the jewellery views.
    pub fn preset(preset: CPlanePreset) -> Self {
        let (x, y) = preset.axes();
        Self {
            origin: [0.; 3],
            x,
            y,
            revision: 1,
        }
    }
    pub fn new(origin: Point3, x: Point3, y: Point3) -> Result<Self> {
        let frame = Self {
            origin,
            x,
            y,
            revision: 1,
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Move the plane origin without changing its axes.
    pub fn with_origin(self, origin: Point3) -> Result<Self> {
        self.validate()?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("CPlane revision"))?;
        let frame = Self {
            origin,
            revision,
            ..self
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Rotate the plane axes about its own normal, right-handed in `x -> y`.
    pub fn rotated(self, radians: f64) -> Result<Self> {
        if !radians.is_finite() {
            return Err(Error::Invalid("CPlane rotation"));
        }
        self.validate()?;
        let (sin, cos) = radians.sin_cos();
        let x = add(mul(self.x, cos), mul(self.y, sin));
        let y = sub(mul(self.y, cos), mul(self.x, sin));
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("CPlane revision"))?;
        let frame = Self {
            origin: self.origin,
            x,
            y,
            revision,
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Build a plane from an origin and two points that define the local +X and +Y
    /// directions. Collinear or coincident points are refused before any assignment.
    pub fn from_three_points(origin: Point3, x_point: Point3, y_point: Point3) -> Result<Self> {
        finite(origin)?;
        finite(x_point)?;
        finite(y_point)?;
        let x_hint = sub(x_point, origin);
        let y_hint = sub(y_point, origin);
        let x = unit(x_hint)?;
        let y_length = dot(y_hint, y_hint).sqrt();
        if !y_length.is_finite() || y_length <= 0. {
            return Err(Error::Invalid("CPlane frame points"));
        }
        let y_perpendicular = sub(y_hint, mul(x, dot(y_hint, x)));
        if dot(y_perpendicular, y_perpendicular).sqrt() <= 1.0e-9 * y_length {
            return Err(Error::Invalid("CPlane frame points"));
        }
        let y = unit(y_perpendicular)?;
        Self::new(origin, x, y)
    }
    pub fn validate(self) -> Result<()> {
        finite(self.origin)?;
        finite(self.x)?;
        finite(self.y)?;
        if self.revision == 0
            || (dot(self.x, self.x) - 1.).abs() > 1e-12
            || (dot(self.y, self.y) - 1.).abs() > 1e-12
            || dot(self.x, self.y).abs() > 1e-12
        {
            return Err(Error::Invalid("orthonormal CPlane"));
        }
        Ok(())
    }
    pub fn normal(self) -> Point3 {
        cross(self.x, self.y)
    }
    pub fn point(self, p: Point3) -> Result<Point3> {
        self.validate()?;
        finite(p)?;
        let out = add(
            self.origin,
            add(
                add(mul(self.x, p[0]), mul(self.y, p[1])),
                mul(self.normal(), p[2]),
            ),
        );
        finite(out)?;
        Ok(out)
    }
}

/// A CPlane is the local frame for typed coordinate input (OM9-L3-01-08-007): a local
/// point becomes world through `point`, and back through the frame axes.
impl om9_core::coords::LocalFrame for CPlane {
    fn to_world(
        &self,
        point: om9_core::coords::LocalPoint,
    ) -> om9_core::Result<om9_core::coords::WorldPoint> {
        om9_core::coords::WorldPoint::try_new(self.point(point.xyz())?)
    }
    fn to_local(
        &self,
        point: om9_core::coords::WorldPoint,
    ) -> om9_core::Result<om9_core::coords::LocalPoint> {
        self.validate()?;
        let delta = sub(point.xyz(), self.origin);
        let local = [
            dot(delta, self.x),
            dot(delta, self.y),
            dot(delta, self.normal()),
        ];
        om9_core::coords::LocalPoint::try_new(local)
    }
}
impl Ray {
    pub fn intersect_plane(self, plane: CPlane) -> Result<Option<Point3>> {
        plane.validate()?;
        finite(self.origin)?;
        finite(self.direction)?;
        let denominator = dot(self.direction, plane.normal());
        // Dimensionless parallel-ray guard, unrelated to geometric tolerances.
        if denominator.abs() <= f64::EPSILON * 16. {
            return Ok(None);
        }
        let t = dot(sub(plane.origin, self.origin), plane.normal()) / denominator;
        if !t.is_finite() || t < 0. {
            return Ok(None);
        }
        let point = add(self.origin, mul(self.direction, t));
        finite(point)?;
        Ok(Some(point))
    }
}
impl Camera {
    pub fn orthographic(
        eye: Point3,
        target: Point3,
        up: Point3,
        vertical_span_mm: f64,
    ) -> Result<Self> {
        let camera = Self {
            eye,
            target,
            up,
            projection: Projection::Orthographic { vertical_span_mm },
            near_mm: 0.01,
            far_mm: 10000.,
            revision: 1,
        };
        camera.validate()?;
        Ok(camera)
    }
    pub fn perspective(
        eye: Point3,
        target: Point3,
        up: Point3,
        vertical_fov_rad: f64,
    ) -> Result<Self> {
        let camera = Self {
            eye,
            target,
            up,
            projection: Projection::Perspective { vertical_fov_rad },
            near_mm: 0.01,
            far_mm: 10000.,
            revision: 1,
        };
        camera.validate()?;
        Ok(camera)
    }
    fn basis(&self) -> Result<[Point3; 3]> {
        let forward = unit(sub(self.target, self.eye))?;
        let right = unit(cross(forward, unit(self.up)?))?;
        let up = unit(cross(right, forward))?;
        Ok([right, up, forward])
    }
    pub fn validate(&self) -> Result<()> {
        finite(self.eye)?;
        finite(self.target)?;
        finite(self.up)?;
        self.basis()?;
        if self.revision == 0
            || !self.near_mm.is_finite()
            || !self.far_mm.is_finite()
            || self.near_mm <= 0.
            || self.far_mm <= self.near_mm
        {
            return Err(Error::Invalid("camera clipping"));
        }
        match self.projection {
            Projection::Orthographic { vertical_span_mm } => {
                if !vertical_span_mm.is_finite() || vertical_span_mm <= 0. {
                    return Err(Error::Invalid("orthographic span"));
                }
            }
            Projection::Perspective { vertical_fov_rad } => {
                if !vertical_fov_rad.is_finite()
                    || vertical_fov_rad <= 0.
                    || vertical_fov_rad >= std::f64::consts::PI
                {
                    return Err(Error::Invalid("perspective field of view"));
                }
            }
        }
        Ok(())
    }
    pub fn project(&self, p: Point3, viewport: Viewport) -> Result<Option<Projected>> {
        self.validate()?;
        viewport.validate()?;
        finite(p)?;
        let [right, up, forward] = self.basis()?;
        let delta = sub(p, self.eye);
        let z = dot(delta, forward);
        if !z.is_finite() || z < self.near_mm || z > self.far_mm {
            return Ok(None);
        }
        let height = match self.projection {
            Projection::Orthographic { vertical_span_mm } => vertical_span_mm,
            Projection::Perspective { vertical_fov_rad } => 2. * z * (vertical_fov_rad / 2.).tan(),
        };
        let scale = viewport.size[1] / height;
        let screen = [
            viewport.origin[0] + viewport.size[0] / 2. + dot(delta, right) * scale,
            viewport.origin[1] + viewport.size[1] / 2. - dot(delta, up) * scale,
        ];
        if !screen.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("camera projection arithmetic"));
        }
        Ok(Some(Projected { screen, depth: z }))
    }
    pub fn ray(&self, screen: [f64; 2], viewport: Viewport) -> Result<Ray> {
        self.validate()?;
        viewport.validate()?;
        if !screen.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("screen point"));
        }
        let [right, up, forward] = self.basis()?;
        let x = (screen[0] - viewport.origin[0] - viewport.size[0] / 2.) / viewport.size[1];
        let y = -(screen[1] - viewport.origin[1] - viewport.size[1] / 2.) / viewport.size[1];
        let ray = match self.projection {
            Projection::Orthographic { vertical_span_mm } => Ray {
                origin: add(
                    self.eye,
                    add(
                        mul(right, x * vertical_span_mm),
                        mul(up, y * vertical_span_mm),
                    ),
                ),
                direction: forward,
            },
            Projection::Perspective { vertical_fov_rad } => {
                let scale = 2. * (vertical_fov_rad / 2.).tan();
                Ray {
                    origin: self.eye,
                    direction: unit(add(forward, add(mul(right, x * scale), mul(up, y * scale))))?,
                }
            }
        };
        finite(ray.origin)?;
        finite(ray.direction)?;
        Ok(ray)
    }
    /// Keep the visible portion without changing the source edge's topology key.
    pub fn clip_segment(&self, a: Point3, b: Point3) -> Result<Option<[Point3; 2]>> {
        self.validate()?;
        finite(a)?;
        finite(b)?;
        let [_, _, forward] = self.basis()?;
        let za = dot(sub(a, self.eye), forward);
        let zb = dot(sub(b, self.eye), forward);
        let dz = zb - za;
        if dz == 0. {
            return Ok((za >= self.near_mm && za <= self.far_mm).then_some([a, b]));
        }
        let t0 = (self.near_mm - za) / dz;
        let t1 = (self.far_mm - za) / dz;
        let mut lo = t0.min(t1).max(0.);
        let mut hi = t0.max(t1).min(1.);
        if lo > hi {
            return Ok(None);
        }
        // Keep computed intersections a few parameter ULPs inside the depth slab. This avoids
        // subtractive roundoff projecting an exact near-plane intersection outside the slab.
        let slack = (hi - lo).min(f64::EPSILON * 32.);
        if lo > 0. {
            lo += slack;
        }
        if hi < 1. {
            hi = (hi - slack).max(lo);
        }
        Ok(Some([
            std::array::from_fn(|i| a[i] + (b[i] - a[i]) * lo),
            std::array::from_fn(|i| a[i] + (b[i] - a[i]) * hi),
        ]))
    }
    pub fn pan(&mut self, delta: [f64; 2]) -> Result<()> {
        if !delta.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("pan distance"));
        }
        let [right, up, _] = self.basis()?;
        let shift = add(mul(right, delta[0]), mul(up, delta[1]));
        let mut next = self.clone();
        next.eye = add(self.eye, shift);
        next.target = add(self.target, shift);
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("camera revision"))?;
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Orbits the eye around the target: `yaw` about the world up axis, `pitch` towards the poles.
    ///
    /// The distance is preserved and the elevation is clamped just short of the poles, so a turntable
    /// drag can never put the eye exactly on the up axis (which would make the basis undefined).
    pub fn orbit(&mut self, yaw: f64, pitch: f64) -> Result<()> {
        if !yaw.is_finite() || !pitch.is_finite() {
            return Err(Error::Invalid("orbit angle"));
        }
        let offset = sub(self.eye, self.target);
        let radius = dot(offset, offset).sqrt();
        if !(radius.is_finite() && radius > 0.) {
            return Err(Error::Invalid("orbit radius"));
        }
        let elevation = (offset[2] / radius).clamp(-1., 1.).asin();
        let azimuth = offset[1].atan2(offset[0]);
        let limit = std::f64::consts::FRAC_PI_2 - 0.02;
        let next_elevation = (elevation + pitch).clamp(-limit, limit);
        let next_azimuth = azimuth + yaw;
        let mut next = self.clone();
        next.eye = [
            self.target[0] + radius * next_elevation.cos() * next_azimuth.cos(),
            self.target[1] + radius * next_elevation.cos() * next_azimuth.sin(),
            self.target[2] + radius * next_elevation.sin(),
        ];
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("camera revision"))?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Fit the current view direction to a world-space bounding sphere. The sphere is
    /// expanded by `margin` and the projection is chosen from the supplied aspect ratio;
    /// a zero-size bound remains finite because the caller supplies the fallback radius.
    pub fn fit_bounds(
        &mut self,
        center: Point3,
        radius: f64,
        aspect: f64,
        margin: f64,
    ) -> Result<()> {
        finite(center)?;
        if !radius.is_finite() || radius <= 0. || !aspect.is_finite() || aspect <= 0. {
            return Err(Error::Invalid("fit bounds"));
        }
        if !margin.is_finite() || margin < 1. {
            return Err(Error::Invalid("fit margin"));
        }
        let direction = unit(sub(self.eye, self.target))?;
        let mut next = self.clone();
        next.target = center;
        next.eye = match self.projection {
            Projection::Orthographic { .. } => {
                let distance = dot(sub(self.eye, self.target), sub(self.eye, self.target))
                    .sqrt()
                    .max(radius * 2.0);
                add(center, mul(direction, distance))
            }
            Projection::Perspective { vertical_fov_rad } => {
                let half_vertical = vertical_fov_rad * 0.5;
                let half_horizontal = (half_vertical.tan() * aspect).atan();
                let half = half_vertical.min(half_horizontal);
                add(center, mul(direction, radius * margin / half.sin()))
            }
        };
        match &mut next.projection {
            Projection::Orthographic { vertical_span_mm } => {
                *vertical_span_mm = 2.0 * radius * margin / aspect.min(1.0);
            }
            Projection::Perspective { .. } => {}
        }
        next.fit_clip(radius)?;
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("camera revision"))?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Move the camera target and eye together, preserving rotation, projection and distance.
    pub fn center_at(&mut self, target: Point3) -> Result<()> {
        finite(target)?;
        let shift = sub(target, self.target);
        let mut next = self.clone();
        next.eye = add(self.eye, shift);
        next.target = target;
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("camera revision"))?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Fits the clip planes to the current orbit radius so a deep zoom does not clip the model.
    pub fn fit_clip(&mut self, radius: f64) -> Result<()> {
        if !radius.is_finite() || radius <= 0. {
            return Err(Error::Invalid("clip radius"));
        }
        self.near_mm = (radius * 0.002).clamp(1e-5, 0.01);
        self.far_mm = (radius * 8.0).max(10.0);
        self.validate()
    }

    pub fn zoom(&mut self, factor: f64) -> Result<()> {
        if !factor.is_finite() || factor <= 0. {
            return Err(Error::Invalid("zoom factor"));
        }
        let mut next = self.clone();
        match &mut next.projection {
            Projection::Orthographic { vertical_span_mm } => *vertical_span_mm *= factor,
            Projection::Perspective { .. } => {
                next.eye = add(next.target, mul(sub(next.eye, next.target), factor))
            }
        }
        next.revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("camera revision"))?;
        next.validate()?;
        *self = next;
        Ok(())
    }
}
