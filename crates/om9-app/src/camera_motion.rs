//! Frame-to-frame camera smoothing for manual orbit, pan and zoom.
//!
//! Input arrives in bursts (especially over RDP): one frame can carry a large drag delta, so applying it
//! directly makes the camera jump. The motion keeps a target pose and eases the current pose towards it
//! every frame. The final camera always equals the target pose, so nothing is lost.

use om9_scene::camera::{Camera, Projection};

#[derive(Clone, Copy, Debug)]
pub struct CameraPose {
    pub target: [f64; 3],
    pub azimuth: f64,
    pub elevation: f64,
    pub radius: f64,
    pub ortho_span: Option<f64>,
}

impl CameraPose {
    pub fn from_camera(camera: &Camera) -> Option<Self> {
        let offset = [
            camera.eye[0] - camera.target[0],
            camera.eye[1] - camera.target[1],
            camera.eye[2] - camera.target[2],
        ];
        let radius = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
        if !radius.is_finite() || radius <= 0.0 {
            return None;
        }
        let elevation = (offset[2] / radius).clamp(-1.0, 1.0).asin();
        let azimuth = offset[1].atan2(offset[0]);
        let ortho_span = match camera.projection {
            Projection::Orthographic { vertical_span_mm } => Some(vertical_span_mm),
            Projection::Perspective { .. } => None,
        };
        Some(Self { target: camera.target, azimuth, elevation, radius, ortho_span })
    }

    pub fn to_camera(&self, template: &Camera) -> Option<Camera> {
        let mut camera = template.clone();
        let cos = self.elevation.cos();
        camera.eye = [
            self.target[0] + self.radius * cos * self.azimuth.cos(),
            self.target[1] + self.radius * cos * self.azimuth.sin(),
            self.target[2] + self.radius * self.elevation.sin(),
        ];
        camera.target = self.target;
        if let (Projection::Orthographic { .. }, Some(span)) = (&mut camera.projection, self.ortho_span) {
            if let Projection::Orthographic { vertical_span_mm } = &mut camera.projection {
                *vertical_span_mm = span;
            }
        }
        camera.revision = template.revision.checked_add(1)?;
        camera.fit_clip(self.radius).ok()?;
        Some(camera)
    }

    pub fn orbit(&mut self, yaw: f64, pitch: f64) {
        let limit = std::f64::consts::FRAC_PI_2 - 0.02;
        self.azimuth += yaw;
        self.elevation = (self.elevation + pitch).clamp(-limit, limit);
    }

    pub fn pan(&mut self, delta: [f64; 3]) {
        for axis in 0..3 {
            self.target[axis] += delta[axis];
        }
    }

    pub fn zoom(&mut self, factor: f64) {
        if let Some(span) = &mut self.ortho_span {
            *span *= factor;
        } else {
            self.radius *= factor;
        }
    }
}

fn shortest_angle(from: f64, to: f64) -> f64 {
    let mut delta = (to - from) % std::f64::consts::TAU;
    if delta > std::f64::consts::PI {
        delta -= std::f64::consts::TAU;
    } else if delta < -std::f64::consts::PI {
        delta += std::f64::consts::TAU;
    }
    delta
}

pub struct CameraMotion {
    current: CameraPose,
    target: CameraPose,
    template: Camera,
}

impl CameraMotion {
    pub fn new(camera: &Camera) -> Option<Self> {
        let pose = CameraPose::from_camera(camera)?;
        Some(Self { current: pose, target: pose, template: camera.clone() })
    }

    /// Adopts a new target camera; `false` when it is invalid or cannot be represented as a pose.
    pub fn set_target(&mut self, camera: &Camera) -> bool {
        let Some(pose) = CameraPose::from_camera(camera) else {
            return false;
        };
        self.target = pose;
        true
    }

    pub fn target_camera(&self) -> Option<Camera> {
        self.target.to_camera(&self.template)
    }

    /// Eases towards the target. Returns the camera to display and whether the motion has settled.
    pub fn advance(&mut self, dt: f64, tau: f64) -> Option<(Camera, bool)> {
        if tau <= 0.0 {
            self.current = self.target;
            let camera = self.target.to_camera(&self.template)?;
            self.template.revision = camera.revision;
            return Some((camera, true));
        }
        let dt = dt.clamp(0.0, 0.1);
        let alpha = 1.0 - (-dt / tau).exp();
        let angle = shortest_angle(self.current.azimuth, self.target.azimuth);
        self.current.azimuth += angle * alpha;
        self.current.elevation += (self.target.elevation - self.current.elevation) * alpha;
        for axis in 0..3 {
            self.current.target[axis] +=
                (self.target.target[axis] - self.current.target[axis]) * alpha;
        }
        let ln_radius = self.current.radius.ln()
            + (self.target.radius.ln() - self.current.radius.ln()) * alpha;
        self.current.radius = ln_radius.exp();
        if let (Some(current), Some(target)) = (self.current.ortho_span, self.target.ortho_span) {
            let ln_span = current.ln() + (target.ln() - current.ln()) * alpha;
            self.current.ortho_span = Some(ln_span.exp());
        }
        let close_enough = angle.abs() < 0.0005
            && (self.current.elevation - self.target.elevation).abs() < 0.0005
            && self.current.target.iter().zip(&self.target.target).all(|(a, b)| (a - b).abs() < 1e-6)
            && (self.current.radius - self.target.radius).abs() < 1e-6
            && self.current.ortho_span == self.target.ortho_span;
        if close_enough {
            self.current = self.target;
        }
        let camera = self.current.to_camera(&self.template)?;
        self.template.revision = camera.revision;
        Some((camera, close_enough))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera::perspective([0.0, -10.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.6).unwrap()
    }

    #[test]
    fn pose_roundtrip_keeps_the_camera() {
        let camera = camera();
        let pose = CameraPose::from_camera(&camera).unwrap();
        let rebuilt = pose.to_camera(&camera).unwrap();
        for axis in 0..3 {
            assert!((camera.eye[axis] - rebuilt.eye[axis]).abs() < 1e-9);
            assert!((camera.target[axis] - rebuilt.target[axis]).abs() < 1e-9);
        }
    }

    #[test]
    fn the_shortest_azimuth_path_wraps_across_pi() {
        let delta = shortest_angle(179f64.to_radians(), (-179f64).to_radians());
        assert!((delta.to_degrees() - 2.0).abs() < 1e-9, "{delta}");
    }

    #[test]
    fn a_zoom_is_interpolated_in_log_distance() {
        let camera = camera();
        let mut motion = CameraMotion::new(&camera).unwrap();
        let mut target = camera.clone();
        target.zoom(0.5).unwrap();
        assert!(motion.set_target(&target));
        let (mid, done) = motion.advance(1.0 / 60.0, 0.06).unwrap();
        assert!(!done);
        let radius = |c: &Camera| {
            ((c.eye[0] - c.target[0]).powi(2)
                + (c.eye[1] - c.target[1]).powi(2)
                + (c.eye[2] - c.target[2]).powi(2))
            .sqrt()
        };
        assert!(radius(&mid) < radius(&camera) && radius(&mid) > radius(&target));
    }

    #[test]
    fn settling_returns_the_target_camera() {
        let camera = camera();
        let mut motion = CameraMotion::new(&camera).unwrap();
        let mut target = camera.clone();
        target.orbit(0.4, 0.1).unwrap();
        assert!(motion.set_target(&target));
        let mut last = None;
        for _ in 0..120 {
            let (camera, done) = motion.advance(1.0 / 60.0, 0.06).unwrap();
            last = Some(camera);
            if done {
                break;
            }
        }
        let last = last.unwrap();
        assert!((last.eye[0] - target.eye[0]).abs() < 1e-5);
        assert!((last.eye[1] - target.eye[1]).abs() < 1e-5);
        assert!((last.eye[2] - target.eye[2]).abs() < 1e-5);
    }

    #[test]
    fn a_deep_zoom_fits_the_clip_planes() {
        let camera = camera();
        let mut target = camera.clone();
        target.zoom(0.002).unwrap();
        let pose = CameraPose::from_camera(&target).unwrap();
        let rebuilt = pose.to_camera(&target).unwrap();
        assert!(rebuilt.near_mm <= 0.0004, "near {} far {}", rebuilt.near_mm, rebuilt.far_mm);
        assert!(rebuilt.far_mm >= 10.0);
        assert!(rebuilt.near_mm < rebuilt.far_mm);
        // Radius 0.2 mm must remain outside the near plane.
        assert!(rebuilt.near_mm < 0.2 * 0.01);
    }

    #[test]
    fn zero_tau_is_immediate() {
        let camera = camera();
        let mut motion = CameraMotion::new(&camera).unwrap();
        let mut target = camera.clone();
        target.pan([1.0, 2.0]).unwrap();
        assert!(motion.set_target(&target));
        let (camera, done) = motion.advance(0.016, 0.0).unwrap();
        assert!(done);
        assert!((camera.target[0] - target.target[0]).abs() < 1e-9);
    }
}