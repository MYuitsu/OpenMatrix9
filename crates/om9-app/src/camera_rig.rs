//! App-local camera rig and quaternion math for smooth viewport navigation.
//!
//! `Camera` remains the serialized source of truth. The rig is an ephemeral pose used while a gesture
//! is running: a pivot, a camera-to-world orientation, a distance, and the orthographic span. Keeping
//! orientation as a quaternion makes orbit interpolation shortest-arc and avoids the gimbal edge cases
//! of incremental yaw/pitch state.

use om9_scene::camera::{Camera, Projection};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quat {
    pub const IDENTITY: Self = Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };

    pub fn axis_angle(axis: [f64; 3], angle: f64) -> Self {
        let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if !length.is_finite() || length <= 1e-15 || !angle.is_finite() {
            return Self::IDENTITY;
        }
        let (sin, cos) = (angle * 0.5).sin_cos();
        let scale = sin / length;
        Self { w: cos, x: axis[0] * scale, y: axis[1] * scale, z: axis[2] * scale }
    }

    /// Quaternion from orthonormal basis columns `right`, `up`, `back` (`back = -forward`).
    pub fn from_basis(right: [f64; 3], up: [f64; 3], back: [f64; 3]) -> Self {
        let m00 = right[0];
        let m01 = up[0];
        let m02 = back[0];
        let m10 = right[1];
        let m11 = up[1];
        let m12 = back[1];
        let m20 = right[2];
        let m21 = up[2];
        let m22 = back[2];
        let trace = m00 + m11 + m22;
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self {
                w: 0.25 * s,
                x: (m21 - m12) / s,
                y: (m02 - m20) / s,
                z: (m10 - m01) / s,
            }
        } else if m00 > m11 && m00 > m22 {
            let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
            Self {
                w: (m21 - m12) / s,
                x: 0.25 * s,
                y: (m01 + m10) / s,
                z: (m02 + m20) / s,
            }
        } else if m11 > m22 {
            let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
            Self {
                w: (m02 - m20) / s,
                x: (m01 + m10) / s,
                y: 0.25 * s,
                z: (m12 + m21) / s,
            }
        } else {
            let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
            Self {
                w: (m10 - m01) / s,
                x: (m02 + m20) / s,
                y: (m12 + m21) / s,
                z: 0.25 * s,
            }
        }
        .normalized()
    }

    pub fn normalized(self) -> Self {
        let length = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if !length.is_finite() || length <= 1e-15 {
            return Self::IDENTITY;
        }
        Self { w: self.w / length, x: self.x / length, y: self.y / length, z: self.z / length }
    }

    /// `self * other`: rotate by `other` first, then by `self`.
    pub fn mul(self, other: Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
        .normalized()
    }

    pub fn rotate(self, vector: [f64; 3]) -> [f64; 3] {
        let qv = [self.x, self.y, self.z];
        let uv = cross(qv, vector);
        let uuv = cross(qv, uv);
        [
            vector[0] + 2.0 * (self.w * uv[0] + uuv[0]),
            vector[1] + 2.0 * (self.w * uv[1] + uuv[1]),
            vector[2] + 2.0 * (self.w * uv[2] + uuv[2]),
        ]
    }

    pub fn dot(self, other: Self) -> f64 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn negated(self) -> Self {
        Self { w: -self.w, x: -self.x, y: -self.y, z: -self.z }
    }

    /// Shortest-arc spherical interpolation.
    pub fn slerp(self, other: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        let mut other = other;
        let mut dot = self.dot(other);
        if dot < 0.0 {
            other = other.negated();
            dot = -dot;
        }
        if dot > 0.999_5 {
            return Self {
                w: self.w + (other.w - self.w) * t,
                x: self.x + (other.x - self.x) * t,
                y: self.y + (other.y - self.y) * t,
                z: self.z + (other.z - self.z) * t,
            }
            .normalized();
        }
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;
        Self {
            w: self.w * a + other.w * b,
            x: self.x * a + other.x * b,
            y: self.y * a + other.y * b,
            z: self.z * a + other.z * b,
        }
        .normalized()
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let length = dot(v, v).sqrt();
    if !length.is_finite() || length <= 1e-15 {
        return None;
    }
    Some([v[0] / length, v[1] / length, v[2] / length])
}

#[derive(Clone, Copy, Debug)]
pub struct CameraRig {
    pub pivot: [f64; 3],
    pub orientation: Quat,
    pub distance: f64,
    pub ortho_span: Option<f64>,
}

impl CameraRig {
    pub fn from_camera(camera: &Camera) -> Option<Self> {
        let forward = normalize([
            camera.target[0] - camera.eye[0],
            camera.target[1] - camera.eye[1],
            camera.target[2] - camera.eye[2],
        ])?;
        let up_hint = normalize(camera.up).unwrap_or([0.0, 0.0, 1.0]);
        let right = normalize(cross(forward, up_hint))?;
        let up = normalize(cross(right, forward))?;
        let orientation = Quat::from_basis(right, up, [-forward[0], -forward[1], -forward[2]]);
        let distance = dot(
            [
                camera.eye[0] - camera.target[0],
                camera.eye[1] - camera.target[1],
                camera.eye[2] - camera.target[2],
            ],
            [-forward[0], -forward[1], -forward[2]],
        );
        if !distance.is_finite() || distance <= 0.0 {
            return None;
        }
        let ortho_span = match camera.projection {
            Projection::Orthographic { vertical_span_mm } => Some(vertical_span_mm),
            Projection::Perspective { .. } => None,
        };
        Some(Self { pivot: camera.target, orientation, distance, ortho_span })
    }

    pub fn to_camera(&self, template: &Camera) -> Option<Camera> {
        let forward = self.orientation.rotate([0.0, 0.0, -1.0]);
        let up = self.orientation.rotate([0.0, 1.0, 0.0]);
        let mut camera = template.clone();
        camera.eye = [
            self.pivot[0] - forward[0] * self.distance,
            self.pivot[1] - forward[1] * self.distance,
            self.pivot[2] - forward[2] * self.distance,
        ];
        camera.target = self.pivot;
        camera.up = up;
        if let Projection::Orthographic { vertical_span_mm } = &mut camera.projection
            && let Some(span) = self.ortho_span
        {
            *vertical_span_mm = span;
        }
        camera.revision = template.revision.checked_add(1)?;
        camera.fit_clip(self.distance).ok()?;
        Some(camera)
    }

    pub fn orbit_turntable(&mut self, yaw: f64, pitch: f64) {
        if !yaw.is_finite() || !pitch.is_finite() {
            return;
        }
        let yaw_rotation = Quat::axis_angle([0.0, 0.0, 1.0], yaw);
        self.orientation = yaw_rotation.mul(self.orientation);
        let elevation = self.orientation.rotate([0.0, 0.0, -1.0])[2].clamp(-1.0, 1.0).asin();
        let limit = std::f64::consts::FRAC_PI_2 - 0.02;
        // Clamp before rotating: asin after crossing a pole folds the angle and misses the flip.
        let pitch = pitch.clamp(-limit - elevation, limit - elevation);
        let right = self.orientation.rotate([1.0, 0.0, 0.0]);
        self.orientation = Quat::axis_angle(right, pitch).mul(self.orientation);
        self.clamp_elevation();
    }

    fn clamp_elevation(&mut self) {
        let limit = std::f64::consts::FRAC_PI_2 - 0.02;
        let forward = self.orientation.rotate([0.0, 0.0, -1.0]);
        let elevation = forward[2].clamp(-1.0, 1.0).asin();
        let clamped = elevation.clamp(-limit, limit);
        if (clamped - elevation).abs() > 1e-15 {
            let right = self.orientation.rotate([1.0, 0.0, 0.0]);
            self.orientation =
                Quat::axis_angle(right, clamped - elevation).mul(self.orientation);
        }
    }

    pub fn pan(&mut self, delta: [f64; 3]) {
        for axis in 0..3 {
            self.pivot[axis] += delta[axis];
        }
    }

    pub fn zoom(&mut self, factor: f64) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        if let Some(span) = &mut self.ortho_span {
            *span = (*span * factor).max(1e-6);
        } else {
            self.distance = (self.distance * factor).max(1e-6);
        }
    }

    /// Zooms so the point under the cursor stays close to the same screen position.
    pub fn zoom_toward(&mut self, cursor: [f64; 3], factor: f64) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let before = self.ortho_span.unwrap_or(self.distance);
        self.zoom(factor);
        let actual = self.ortho_span.unwrap_or(self.distance) / before;
        for axis in 0..3 {
            self.pivot[axis] = cursor[axis] + (self.pivot[axis] - cursor[axis]) * actual;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera::perspective([0.0, -10.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.6).unwrap()
    }

    #[test]
    fn rig_roundtrip_preserves_eye_target_and_up() {
        let camera = camera();
        let rig = CameraRig::from_camera(&camera).unwrap();
        let rebuilt = rig.to_camera(&camera).unwrap();
        for axis in 0..3 {
            assert!((camera.eye[axis] - rebuilt.eye[axis]).abs() < 1e-9);
            assert!((camera.target[axis] - rebuilt.target[axis]).abs() < 1e-9);
        }
        let forward = normalize([
            rebuilt.target[0] - rebuilt.eye[0],
            rebuilt.target[1] - rebuilt.eye[1],
            rebuilt.target[2] - rebuilt.eye[2],
        ])
        .unwrap();
        assert!(dot(forward, rebuilt.up).abs() < 1e-9);
        assert!((dot(rebuilt.up, rebuilt.up) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn slerp_takes_the_short_arc_and_normalizes() {
        let a = Quat::axis_angle([0.0, 0.0, 1.0], 179f64.to_radians());
        let b = Quat::axis_angle([0.0, 0.0, 1.0], (-179f64).to_radians());
        let mid = a.slerp(b, 0.5);
        assert!((mid.w * mid.w + mid.x * mid.x + mid.y * mid.y + mid.z * mid.z - 1.0).abs() < 1e-12);
        let turned = mid.rotate([1.0, 0.0, 0.0]);
        assert!(turned[0] < 0.0);
    }

    #[test]
    fn orbit_keeps_the_distance() {
        let camera = camera();
        let mut rig = CameraRig::from_camera(&camera).unwrap();
        let distance = rig.distance;
        rig.orbit_turntable(0.3, 0.2);
        assert!((rig.distance - distance).abs() < 1e-12);
        let converted = rig.to_camera(&camera).unwrap();
        let radius = ((converted.eye[0] - converted.target[0]).powi(2)
            + (converted.eye[1] - converted.target[1]).powi(2)
            + (converted.eye[2] - converted.target[2]).powi(2))
        .sqrt();
        assert!((radius - distance).abs() < 1e-9);
    }

    #[test]
    fn zoom_is_multiplicative_and_clamped() {
        let camera = camera();
        let mut rig = CameraRig::from_camera(&camera).unwrap();
        let distance = rig.distance;
        rig.zoom(0.5);
        assert!((rig.distance - distance * 0.5).abs() < 1e-12);
        rig.zoom(1e-9);
        assert!(rig.distance >= 1e-6);
    }

    #[test]
    fn zoom_toward_keeps_the_cursor_point_on_screen() {
        let camera = camera();
        let mut rig = CameraRig::from_camera(&camera).unwrap();
        let right = rig.orientation.rotate([1.0, 0.0, 0.0]);
        let up = rig.orientation.rotate([0.0, 1.0, 0.0]);
        let point = [
            rig.pivot[0] + right[0] + up[0] * 0.5,
            rig.pivot[1] + right[1] + up[1] * 0.5,
            rig.pivot[2] + right[2] + up[2] * 0.5,
        ];
        let viewport = om9_scene::camera::Viewport { origin: [0.0, 0.0], size: [800.0, 600.0] };
        let before = camera.project(point, viewport).unwrap().unwrap();
        rig.zoom_toward(point, 0.5);
        let after_camera = rig.to_camera(&camera).unwrap();
        let after = after_camera.project(point, viewport).unwrap().unwrap();
        assert!((before.screen[0] - after.screen[0]).abs() < 1e-6);
        assert!((before.screen[1] - after.screen[1]).abs() < 1e-6);
    }

    #[test]
    fn zoom_toward_moves_the_pivot_toward_the_cursor() {
        let camera = camera();
        let mut rig = CameraRig::from_camera(&camera).unwrap();
        let before = rig.pivot;
        rig.zoom_toward([1.0, 0.0, 0.0], 0.5);
        assert!((rig.pivot[0] - 0.5).abs() < 1e-12);
        assert!(rig.pivot[0] > before[0]);
    }
}
