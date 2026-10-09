//! Shared RCORE-03 construction-frame and Cartesian point input.
//!
//! Points from native picking are already world coordinates and must bypass this
//! typed-input transform. The supplied component parser owns unit conversion.

pub type Point = [f64; 3];
pub type Axes = [Point; 3];
const LIMIT: f64 = 1e9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    origin: Point,
    axes: Axes,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            origin: [0.; 3],
            axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        }
    }
}

impl Frame {
    pub fn from_three_points(
        origin: Point,
        x_point: Point,
        y_point: Point,
    ) -> Result<Self, String> {
        for point in [origin, x_point, y_point] {
            check_point(point)?;
        }
        let x: Point = std::array::from_fn(|i| x_point[i] - origin[i]);
        let hint: Point = std::array::from_fn(|i| y_point[i] - origin[i]);
        let unit = |vector: Point| -> Result<Point, String> {
            let length = dot(vector, vector).sqrt();
            if length < 1e-7 {
                return Err("Construction plane points must be distinct and noncollinear".into());
            }
            Ok(vector.map(|value| value / length))
        };
        let x = unit(x)?;
        let projection = dot(hint, x);
        let y = unit(std::array::from_fn(|i| hint[i] - projection * x[i]))?;
        let z = [
            x[1] * y[2] - x[2] * y[1],
            x[2] * y[0] - x[0] * y[2],
            x[0] * y[1] - x[1] * y[0],
        ];
        Self::new(origin, [x, y, z])
    }

    /// Reject invalid snapshots atomically; never repair or normalize a frame
    /// silently because that would change the meaning of accepted coordinates.
    pub fn new(origin: Point, axes: Axes) -> Result<Self, String> {
        check_point(origin)?;
        if axes.iter().flatten().any(|v| !v.is_finite()) {
            return Err("Invalid construction plane".into());
        }
        for i in 0..3 {
            for j in 0..3 {
                if (dot(axes[i], axes[j]) - if i == j { 1. } else { 0. }).abs() > 1e-6 {
                    return Err("Construction plane must be orthonormal".into());
                }
            }
        }
        let cross = [
            axes[0][1] * axes[1][2] - axes[0][2] * axes[1][1],
            axes[0][2] * axes[1][0] - axes[0][0] * axes[1][2],
            axes[0][0] * axes[1][1] - axes[0][1] * axes[1][0],
        ];
        if dot(cross, axes[2]) <= 0. {
            return Err("Construction plane must be right-handed".into());
        }
        Ok(Self { origin, axes })
    }

    pub fn origin(self) -> Point {
        self.origin
    }

    pub fn axes(self) -> Axes {
        self.axes
    }

    fn vector_to_world(self, value: Point) -> Point {
        std::array::from_fn(|i| {
            self.axes[0][i] * value[0] + self.axes[1][i] * value[1] + self.axes[2][i] * value[2]
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointKind {
    AbsoluteCPlane,
    AbsoluteWorld,
    RelativeCPlane,
    RelativeWorld,
}

/// The original syntax and explicit kind remain available to the consumer;
/// component count never determines whether a point is world or CPlane based.
#[derive(Clone, Debug, PartialEq)]
pub struct PointInput {
    pub original: String,
    pub kind: PointKind,
    pub components: Point,
    pub component_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPoint {
    pub world: Point,
    pub input: PointInput,
    pub frame: Frame,
    pub previous: Option<Point>,
}

impl PointInput {
    /// Cartesian Rhino 5 grammar with dot decimals and comma separators.
    /// Polar/spherical/surveyor and unspecified prefix permutations are not
    /// accepted by this Cartesian parser. Scalars/options remain phase-owned.
    pub fn parse(
        text: &str,
        mut component: impl FnMut(&str) -> Result<f64, String>,
    ) -> Result<Self, String> {
        let token = text.trim();
        if token.chars().any(char::is_whitespace) {
            return Err("Whitespace is not allowed inside point coordinates".into());
        }
        let (kind, coordinates) = if let Some(value) = token.strip_prefix("wr") {
            (PointKind::RelativeWorld, value)
        } else if let Some(value) = token.strip_prefix('w') {
            (PointKind::AbsoluteWorld, value)
        } else if let Some(value) = token
            .strip_prefix('r')
            .or_else(|| token.strip_prefix('R'))
            .or_else(|| token.strip_prefix('@'))
        {
            (PointKind::RelativeCPlane, value)
        } else {
            (PointKind::AbsoluteCPlane, token)
        };
        let mut components = [0.; 3];
        let mut count = 0;
        for (i, value) in coordinates.split(',').enumerate() {
            if i >= 3 || value.is_empty() {
                return Err("Enter x,y or x,y,z".into());
            }
            components[i] = component(value)?;
            count += 1;
        }
        if count < 2 || (kind == PointKind::AbsoluteWorld && count != 3) {
            return Err("Enter x,y or x,y,z; explicit world input requires wX,Y,Z".into());
        }
        check_point(components)?;
        Ok(Self {
            original: text.into(),
            kind,
            components,
            component_count: count,
        })
    }

    pub fn resolve(self, frame: Frame, previous: Option<Point>) -> Result<ResolvedPoint, String> {
        check_point(self.components)?;
        let local = matches!(
            self.kind,
            PointKind::AbsoluteCPlane | PointKind::RelativeCPlane
        );
        let relative = matches!(
            self.kind,
            PointKind::RelativeCPlane | PointKind::RelativeWorld
        );
        let vector = if local {
            frame.vector_to_world(self.components)
        } else {
            self.components
        };
        let base = if relative {
            let previous = previous.ok_or("Relative input requires a previous point")?;
            check_point(previous)?;
            previous
        } else if local {
            frame.origin
        } else {
            [0.; 3]
        };
        let world = std::array::from_fn(|i| base[i] + vector[i]);
        check_point(world)?;
        Ok(ResolvedPoint {
            world,
            input: self,
            frame,
            previous,
        })
    }
}

fn dot(a: Point, b: Point) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn check_point(point: Point) -> Result<(), String> {
    if point.iter().any(|v| !v.is_finite() || v.abs() > LIMIT) {
        Err("Coordinate must be finite and within 1e9 mm".into())
    } else {
        Ok(())
    }
}
