//! OM9-CURVE-004 geometry. Host choices: sharp corners, full Center dimensions,
//! fixed construction axes after the first pick, 1e-7 mm minimum sides.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum Mode {
    #[default]
    Corners,
    Center,
    ThreePoint,
    Vertical,
}
#[derive(Clone, Default)]
pub struct Rectangle {
    pub mode: Mode,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub axes: Option<[[f64; 3]; 3]>,
}
pub const AXES: [[f64; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn scale(a: [f64; 3], n: f64) -> [f64; 3] {
    a.map(|v| v * n)
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: [f64; 3]) -> Result<[f64; 3], String> {
    let n = norm(a);
    if n < 1e-7 {
        Err("Rectangle side must exceed 1e-7 mm".into())
    } else {
        Ok(scale(a, 1. / n))
    }
}
impl Rectangle {
    pub fn axes(&self) -> [[f64; 3]; 3] {
        self.axes.unwrap_or(AXES)
    }
    pub fn edge(&self, a: [f64; 3], p: [f64; 3]) -> Result<[f64; 3], String> {
        let d = sub(p, a);
        let axes = self.axes();
        let d = if self.mode == Mode::Vertical {
            sub(d, scale(axes[2], dot(d, axes[2])))
        } else {
            d
        };
        let direction = unit(d)?;
        Ok(scale(direction, self.length.unwrap_or(norm(d))))
    }
    pub fn width_direction(&self, edge: [f64; 3]) -> Result<[f64; 3], String> {
        let axes = self.axes();
        if self.mode == Mode::Vertical {
            return Ok(axes[2]);
        }
        let d = cross(axes[2], unit(edge)?);
        if norm(d) > 1e-7 {
            unit(d)
        } else {
            unit(cross(axes[1], unit(edge)?))
        }
    }
    pub fn pick_normal(&self, points: &[[f64; 3]]) -> Result<[f64; 3], String> {
        if points.len() == 2 && matches!(self.mode, Mode::ThreePoint | Mode::Vertical) {
            let edge = sub(points[1], points[0]);
            unit(cross(edge, self.width_direction(edge)?))
        } else {
            Ok(self.axes()[2])
        }
    }
    fn closed(a: [f64; 3], x: [f64; 3], y: [f64; 3]) -> Result<Vec<[f64; 3]>, String> {
        unit(x)?;
        unit(y)?;
        let points = vec![a, add(a, x), add(add(a, x), y), add(a, y), a];
        if points
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1e9)
        {
            return Err("Rectangle output exceeds coordinate limits".into());
        }
        Ok(points)
    }
    pub fn geometry(
        &self,
        points: &[[f64; 3]],
        p: [f64; 3],
        square: bool,
    ) -> Result<Vec<[f64; 3]>, String> {
        let a = points[0];
        let axes = self.axes();
        if matches!(self.mode, Mode::Corners | Mode::Center) {
            let d = sub(p, a);
            let factor = if self.mode == Mode::Center { 2. } else { 1. };
            let mut x = dot(d, axes[0]) * factor;
            let mut y = dot(d, axes[1]) * factor;
            if let Some(length) = self.length {
                x = length * if x < 0. { -1. } else { 1. };
            }
            if let Some(width) = self.width {
                y = width;
            }
            if square {
                let n = self.length.unwrap_or(x.abs().max(y.abs()));
                x = n * if x < 0. { -1. } else { 1. };
                y = n * if y < 0. { -1. } else { 1. };
            }
            let x = scale(axes[0], x);
            let y = scale(axes[1], y);
            let corner = if self.mode == Mode::Center {
                sub(a, scale(add(x, y), 0.5))
            } else {
                a
            };
            Self::closed(corner, x, y)
        } else {
            let x = sub(points[1], a);
            let d = sub(p, a);
            let mut y = if self.mode == Mode::Vertical {
                scale(axes[2], dot(d, axes[2]))
            } else {
                sub(d, scale(unit(x)?, dot(d, unit(x)?)))
            };
            if let Some(width) = self.width {
                y = scale(self.width_direction(x)?, width);
            }
            if square {
                y = scale(unit(y)?, norm(x));
            }
            Self::closed(a, x, y)
        }
    }
    pub fn numeric_geometry(&self, points: &[[f64; 3]]) -> Result<Option<Vec<[f64; 3]>>, String> {
        if points.is_empty() {
            return Ok(None);
        }
        if matches!(self.mode, Mode::Corners | Mode::Center) {
            let (Some(length), Some(width)) = (self.length, self.width) else {
                return Ok(None);
            };
            let axes = self.axes();
            let a = points[0];
            let factor = if self.mode == Mode::Center { 0.5 } else { 1. };
            self.geometry(
                points,
                add(
                    a,
                    scale(add(scale(axes[0], length), scale(axes[1], width)), factor),
                ),
                false,
            )
            .map(Some)
        } else {
            let Some(width) = self.width else {
                return Ok(None);
            };
            if points.len() != 2 {
                return Ok(None);
            }
            Self::closed(
                points[0],
                sub(points[1], points[0]),
                scale(self.width_direction(sub(points[1], points[0]))?, width),
            )
            .map(Some)
        }
    }
}
