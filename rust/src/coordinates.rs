//! Shared RCORE-03 construction-frame and Cartesian point input.
//!
//! Points from native picking are already world coordinates and must bypass this
//! typed-input transform. The supplied component parser owns unit conversion.

pub type Point = [f64; 3];
pub type Axes = [Point; 3];
const LIMIT: f64 = 1e9;
// Dimensionless roundoff allowance for the Gram matrix, not model tolerance.
// Three-term dot products and quaternion-to-basis arithmetic accumulate several
// ulps (the rotation regression sample reaches 8 EPSILON). Keeping a 64-EPSILON
// margin accepts rounded rigid rotations without admitting geometric scale or
// shear that a native rotation/placement would silently discard.
const ORTHONORMAL_ROUNDOFF: f64 = 64. * f64::EPSILON;

/// Lexical routing only. Callers resolve exact options before numeric input;
/// malformed point syntax must never fall back to scalar/operation parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericSyntax {
    Point,
    Scalar,
    AngleConstraint,
}

pub fn classify_numeric(text: &str) -> NumericSyntax {
    let token = text.trim();
    if token.starts_with('<') {
        NumericSyntax::AngleConstraint
    } else if token.contains(',') || token.contains('<') {
        NumericSyntax::Point
    } else {
        NumericSyntax::Scalar
    }
}

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
                if (dot(axes[i], axes[j]) - if i == j { 1. } else { 0. }).abs()
                    > ORTHONORMAL_ROUNDOFF
                {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurveyorQuadrant {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Original point notation, with angle dimensions kept separate from lengths.
/// `PointInput::components` always contains the evaluated Cartesian mm vector.
#[derive(Clone, Debug, PartialEq)]
pub enum PointNotation {
    Cartesian,
    Polar {
        radius_mm: f64,
        azimuth_rad: f64,
        height_mm: Option<f64>,
    },
    Spherical {
        radius_mm: f64,
        azimuth_rad: f64,
        elevation_rad: f64,
    },
    SphericalXY {
        horizontal_mm: [f64; 2],
        elevation_rad: f64,
    },
    Surveyor {
        radius_mm: f64,
        bearing_rad: f64,
        quadrant: SurveyorQuadrant,
    },
}

/// The original syntax and explicit kind remain available to the consumer;
/// component count never determines whether a point is world or CPlane based.
#[derive(Clone, Debug, PartialEq)]
pub struct PointInput {
    pub original: String,
    pub kind: PointKind,
    pub components: Point,
    pub component_count: usize,
    pub notation: PointNotation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPoint {
    pub world: Point,
    pub input: PointInput,
    pub frame: Frame,
    pub previous: Option<Point>,
}

impl PointInput {
    /// Rhino 5 point grammar with dot decimals and comma separators. Lengths
    /// use the caller's unit policy; angles use the shared degree/DMS parser.
    /// Scalars/options/one-shot angle constraints remain phase-owned.
    pub fn parse(
        text: &str,
        mut component: impl FnMut(&str) -> Result<f64, String>,
    ) -> Result<Self, String> {
        let token = text.trim();
        // In a point-input phase, Rhino's bare 0 abbreviates CPlane 0,0,0.
        // The consumer still decides whether the current phase expects a
        // point or a scalar dimension; a zero width/radius must stay invalid.
        if token == "0" {
            return Ok(Self {
                original: text.into(),
                kind: PointKind::AbsoluteCPlane,
                components: [0.; 3],
                component_count: 3,
                notation: PointNotation::Cartesian,
            });
        }
        if token.chars().any(char::is_whitespace) {
            return Err("Whitespace is not allowed inside point coordinates".into());
        }
        // `wr` is general relative-world syntax. The source also explicitly
        // documents `rw` for polar and `@w` for surveyor; do not generalize these
        // exceptional permutations to Cartesian or unrelated notation.
        let (kind, coordinates, restricted_prefix) = if let Some(value) = token.strip_prefix("wr") {
            (PointKind::RelativeWorld, value, None)
        } else if let Some(value) = token.strip_prefix("rw") {
            (PointKind::RelativeWorld, value, Some("polar"))
        } else if let Some(value) = token.strip_prefix("@w") {
            (PointKind::RelativeWorld, value, Some("surveyor"))
        } else if let Some(value) = token.strip_prefix('w') {
            (PointKind::AbsoluteWorld, value, None)
        } else if let Some(value) = token
            .strip_prefix('r')
            .or_else(|| token.strip_prefix('R'))
            .or_else(|| token.strip_prefix('@'))
        {
            (PointKind::RelativeCPlane, value, None)
        } else {
            (PointKind::AbsoluteCPlane, token, None)
        };
        if coordinates.contains('<') {
            let (components, notation) = non_cartesian(coordinates, &mut component)?;
            if !matches!(
                (restricted_prefix, &notation),
                (None, _)
                    | (Some("polar"), PointNotation::Polar { .. })
                    | (Some("surveyor"), PointNotation::Surveyor { .. })
            ) {
                return Err("Prefix is unsupported for this point notation".into());
            }
            check_point(components)?;
            return Ok(Self {
                original: text.into(),
                kind,
                components,
                component_count: 3,
                notation,
            });
        }
        if restricted_prefix.is_some() {
            return Err("Prefix is unsupported for Cartesian coordinates".into());
        }
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
            notation: PointNotation::Cartesian,
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

fn non_cartesian(
    text: &str,
    component: &mut impl FnMut(&str) -> Result<f64, String>,
) -> Result<(Point, PointNotation), String> {
    let parts: Vec<_> = text.splitn(4, '<').collect();
    if !(2..=3).contains(&parts.len()) || parts.iter().any(|part| part.is_empty()) {
        return Err(
            "Enter distance<angle, distance<angle,z, distance<angle<elevation or x,y<elevation"
                .into(),
        );
    }
    let angle = |value: &str| crate::units::parse_angle(value).map_err(|error| error.to_string());
    if parts.len() == 3 {
        let radius_mm = radius(component(parts[0])?)?;
        let azimuth_rad = angle(parts[1])?;
        let elevation_rad = angle(parts[2])?;
        let horizontal = radius_mm * elevation_rad.cos();
        return Ok((
            [
                horizontal * azimuth_rad.cos(),
                horizontal * azimuth_rad.sin(),
                radius_mm * elevation_rad.sin(),
            ],
            PointNotation::Spherical {
                radius_mm,
                azimuth_rad,
                elevation_rad,
            },
        ));
    }
    if let Some((x, y)) = parts[0].split_once(',') {
        if x.is_empty() || y.is_empty() || y.contains(',') {
            return Err("Spherical XY input requires x,y<elevation".into());
        }
        let horizontal_mm = [component(x)?, component(y)?];
        check_point([horizontal_mm[0], horizontal_mm[1], 0.])?;
        let elevation_rad = angle(parts[1])?;
        let elevation_degrees =
            crate::units::angle_degrees(parts[1]).map_err(|error| error.to_string())?;
        // tan is undefined at odd quarter-turns. Allow only ordinary trig
        // roundoff around that singularity; do not produce a huge finite point.
        if elevation_degrees.rem_euclid(180.) == 90.
            || elevation_rad.cos().abs() <= 64. * f64::EPSILON
        {
            return Err("Spherical XY elevation is singular at 90 degrees".into());
        }
        let height = horizontal_mm[0].hypot(horizontal_mm[1]) * elevation_rad.tan();
        return Ok((
            [horizontal_mm[0], horizontal_mm[1], height],
            PointNotation::SphericalXY {
                horizontal_mm,
                elevation_rad,
            },
        ));
    }
    let radius_mm = radius(component(parts[0])?)?;
    let direction = parts[1];
    if direction.starts_with('N') || direction.starts_with('S') {
        let north = direction.starts_with('N');
        let (bearing, east) = if let Some(value) = direction[1..].strip_suffix('E') {
            (value, true)
        } else if let Some(value) = direction[1..].strip_suffix('W') {
            (value, false)
        } else {
            return Err("Surveyor bearing requires N/S angle E/W".into());
        };
        let bearing_rad = angle(bearing)?;
        if !(0. ..=std::f64::consts::FRAC_PI_2).contains(&bearing_rad) {
            return Err("Surveyor bearing must be between 0 and 90 degrees".into());
        }
        let quadrant = match (north, east) {
            (true, true) => SurveyorQuadrant::NorthEast,
            (true, false) => SurveyorQuadrant::NorthWest,
            (false, true) => SurveyorQuadrant::SouthEast,
            (false, false) => SurveyorQuadrant::SouthWest,
        };
        return Ok((
            [
                radius_mm * bearing_rad.sin() * if east { 1. } else { -1. },
                radius_mm * bearing_rad.cos() * if north { 1. } else { -1. },
                0.,
            ],
            PointNotation::Surveyor {
                radius_mm,
                bearing_rad,
                quadrant,
            },
        ));
    }
    let (azimuth, height_mm) = if let Some((a, z)) = direction.split_once(',') {
        if z.is_empty() || z.contains(',') {
            return Err("Polar input requires distance<angle,z".into());
        }
        (a, Some(component(z)?))
    } else {
        (direction, None)
    };
    let azimuth_rad = angle(azimuth)?;
    Ok((
        [
            radius_mm * azimuth_rad.cos(),
            radius_mm * azimuth_rad.sin(),
            height_mm.unwrap_or(0.),
        ],
        PointNotation::Polar {
            radius_mm,
            azimuth_rad,
            height_mm,
        },
    ))
}

// Explicit OM9 point-input policy: radius may be zero (the consuming operation
// decides degeneracy), never negative, and must fit the existing mm domain.
fn radius(value: f64) -> Result<f64, String> {
    check_point([value, 0., 0.])?;
    if value < 0. {
        Err("Polar, spherical and surveyor radius must be nonnegative".into())
    } else {
        Ok(value)
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
