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
