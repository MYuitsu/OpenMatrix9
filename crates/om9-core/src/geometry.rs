// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    identity::DefinitionId,
};
use serde::{Deserialize, Serialize};

pub type Point3 = [f64; 3];
pub fn finite(p: Point3) -> Result<()> {
    if p.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(Error::Invalid("nonfinite coordinate"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Bounds {
    Unknown,
    Empty,
    Finite { min: Point3, max: Point3 },
}
impl Bounds {
    pub fn validate(self) -> Result<()> {
        if let Self::Finite { min, max } = self {
            finite(min)?;
            finite(max)?;
            if (0..3).any(|i| min[i] > max[i]) {
                return Err(Error::Invalid("bounds range"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
    Micrometer,
}
impl Unit {
    pub fn factor(self) -> f64 {
        match self {
            Self::Millimeter => 1.0,
            Self::Centimeter => 10.0,
            Self::Meter => 1000.0,
            Self::Inch => 25.4,
            Self::Foot => 304.8,
            Self::Micrometer => 0.001,
        }
    }
    pub fn length_to_mm(self, length: f64) -> Result<f64> {
        let value = length * self.factor();
        if value.is_finite() {
            Ok(value)
        } else {
            Err(Error::Invalid("length conversion"))
        }
    }
    /// Unit tokens admitted by the typed length parser (OM9-L3-01-08-001).
    pub fn parse(token: &str) -> std::result::Result<Self, crate::units::UnitError> {
        let trimmed = token.trim();
        if trimmed.eq_ignore_ascii_case("mm") {
            Ok(Self::Millimeter)
        } else if trimmed.eq_ignore_ascii_case("cm") {
            Ok(Self::Centimeter)
        } else if trimmed.eq_ignore_ascii_case("m") {
            Ok(Self::Meter)
        } else if trimmed.eq_ignore_ascii_case("in") {
            Ok(Self::Inch)
        } else if trimmed.eq_ignore_ascii_case("ft") {
            Ok(Self::Foot)
        } else if trimmed.eq_ignore_ascii_case("um")
            || trimmed == "µm"
            || trimmed.eq_ignore_ascii_case("μm")
        {
            Ok(Self::Micrometer)
        } else {
            // The token copy is bounded so a huge input cannot grow the error.
            Err(crate::units::UnitError::UnsupportedUnit {
                token: trimmed.chars().take(16).collect(),
            })
        }
    }
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Millimeter => "mm",
            Self::Centimeter => "cm",
            Self::Meter => "m",
            Self::Inch => "in",
            Self::Foot => "ft",
            Self::Micrometer => "um",
        }
    }
    /// The exact conversion ratio `millimetres = value * numerator / denominator`.
    ///
    /// Using an integer ratio instead of a rounded decimal factor removes one rounding from
    /// every conversion: 25.4 mm/in is `127/5` and 304.8 mm/ft is `1524/5`, so the factor
    /// itself is exact and the only rounding is the final division. This is the accuracy
    /// contract of OM9-L3-01-08-001 (see `tests/l2_01_08_precision.rs`).
    pub fn ratio(self) -> (f64, f64) {
        match self {
            Self::Millimeter => (1.0, 1.0),
            Self::Centimeter => (10.0, 1.0),
            Self::Meter => (1000.0, 1.0),
            Self::Inch => (127.0, 5.0),
            Self::Foot => (1524.0, 5.0),
            Self::Micrometer => (1.0, 1000.0),
        }
    }
}

/// Row-major affine matrix; source local -> occurrence chain -> world.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform(pub [[f64; 4]; 4]);
impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
impl Transform {
    pub fn identity() -> Self {
        Self([
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }
    pub fn translation(p: Point3) -> Self {
        let mut m = Self::identity();
        for (i, v) in p.iter().enumerate() {
            m.0[i][3] = *v;
        }
        m
    }
    pub fn scale(p: Point3) -> Result<Self> {
        let mut m = Self::identity();
        for (i, v) in p.iter().enumerate() {
            m.0[i][i] = *v;
        }
        m.validate()?;
        Ok(m)
    }
    pub fn validate(self) -> Result<()> {
        if !self.0.iter().flatten().all(|v| v.is_finite()) || self.0[3] != [0., 0., 0., 1.] {
            return Err(Error::Invalid("affine matrix"));
        }
        let a = self.0;
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if !det.is_finite() || det == 0.0 {
            return Err(Error::Invalid("singular/overflowed affine matrix"));
        }
        Ok(())
    }
    pub fn compose(self, local: Self) -> Result<Self> {
        self.validate()?;
        local.validate()?;
        let mut out = [[0.; 4]; 4];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = (0..4).map(|k| self.0[i][k] * local.0[k][j]).sum();
            }
        }
        let result = Self(out);
        result.validate()?;
        Ok(result)
    }
    pub fn point(self, p: Point3) -> Result<Point3> {
        self.validate()?;
        finite(p)?;
        let mut q = [0.; 3];
        for (i, v) in q.iter_mut().enumerate() {
            *v = self.0[i][3] + (0..3).map(|j| self.0[i][j] * p[j]).sum::<f64>();
        }
        finite(q)?;
        Ok(q)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NurbsCurve {
    pub degree: usize,
    pub poles: Vec<Point3>,
    pub weights: Vec<f64>,
    /// Expanded knot vector (including repeated values); preserves multiplicities.
    pub knots: Vec<f64>,
    pub domain: [f64; 2],
    pub periodic: bool,
    pub reversed: bool,
    pub seam: Option<f64>,
}
fn axis(
    degree: usize,
    count: usize,
    knots: &[f64],
    domain: [f64; 2],
    limits: &Limits,
) -> Result<()> {
    if degree == 0 || degree > limits.max_degree || count <= degree {
        return Err(Error::Invalid("NURBS degree/pole count"));
    }
    let expected = count
        .checked_add(degree)
        .and_then(|n| n.checked_add(1))
        .ok_or(Error::Overflow("knot count"))?;
    if knots.len() != expected
        || !knots.iter().all(|v| v.is_finite())
        || knots.windows(2).any(|w| w[0] > w[1])
    {
        return Err(Error::Invalid("NURBS knots"));
    }
    if !domain.iter().all(|v| v.is_finite())
        || domain[0] >= domain[1]
        || !(domain[1] - domain[0]).is_finite()
        || domain[0] < knots[degree]
        || domain[1] > knots[count]
    {
        return Err(Error::Invalid("NURBS domain"));
    }
    let mut run = 1;
    for w in knots.windows(2) {
        run = if w[0] == w[1] { run + 1 } else { 1 };
        if run > degree + 1 {
            return Err(Error::Invalid("knot multiplicity"));
        }
    }
    Ok(())
}
fn poles_weights(poles: &[Point3], weights: &[f64], limits: &Limits) -> Result<Usage> {
    let mut usage = Usage::array::<Point3>(poles.len())?;
    usage.points = u64::try_from(poles.len()).map_err(|_| Error::Overflow("pole count"))?;
    usage.add(Usage::array::<f64>(weights.len())?, limits)?;
    if weights.len() != poles.len() || weights.iter().any(|v| !v.is_finite() || *v <= 0.) {
        return Err(Error::Invalid("supported positive rational weights"));
    }
    for p in poles {
        finite(*p)?;
    }
    Ok(usage)
}
impl NurbsCurve {
    /// Admit an explicitly repeated-pole periodic basis. This is deliberately
    /// stricter than metadata validation and does not certify arbitrary imports.
    pub fn certify_repeated_periodic(&self, limits: &Limits) -> Result<()> {
        self.certify_repeated_periodic_basis(limits)?;
        if self.domain != [self.knots[self.degree], self.knots[self.poles.len()]] {
            return Err(Error::Unsupported("periodic NURBS basis is not certified"));
        }
        Ok(())
    }

    /// Certify the repeated basis independently of a possibly trimmed active interval.
    pub fn certify_repeated_periodic_basis(&self, limits: &Limits) -> Result<()> {
        self.validate(limits)?;
        let degree=self.degree;let count=self.poles.len();
        if !self.periodic || count<=degree*2 {
            return Err(Error::Unsupported("periodic NURBS basis is not certified"));
        }
        let period=count-degree;let span=self.knots[count]-self.knots[degree];
        if !span.is_finite() || span <= 0. {return Err(Error::Unsupported("periodic NURBS basis span"));}
        let seam=self.knots[degree];
        let multiplicity=self.knots.partition_point(|k|*k<=seam)-self.knots.partition_point(|k|*k<seam);
        if multiplicity>degree {return Err(Error::Unsupported("periodic NURBS seam is discontinuous"));}
        for i in 0..degree {
            if self.poles[i]!=self.poles[period+i] || self.weights[i]!=self.weights[period+i] {
                return Err(Error::Unsupported("periodic NURBS control points do not repeat"));
            }
        }
        for i in 0..self.knots.len()-period {
            let left=self.knots[i];let right=self.knots[i+period];
            let roundoff=2.*f64::EPSILON*left.abs().max(right.abs()).max(span.abs());
            if roundoff>span.abs()*1e-5 {
                return Err(Error::Unsupported("periodic NURBS knot precision is insufficient"));
            }
            if ((right-left)-span).abs()>span.abs()*1e-12+roundoff {
                return Err(Error::Unsupported("periodic NURBS knots do not repeat"));
            }
        }
        Ok(())
    }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        axis(
            self.degree,
            self.poles.len(),
            &self.knots,
            self.domain,
            limits,
        )?;
        let mut usage = poles_weights(&self.poles, &self.weights, limits)?;
        usage.add(
            Usage::spare::<Point3>(self.poles.len(), self.poles.capacity())?,
            limits,
        )?;
        usage.add(
            Usage::spare::<f64>(self.weights.len(), self.weights.capacity())?,
            limits,
        )?;
        usage.add(Usage::array::<f64>(self.knots.capacity())?, limits)?;
        if self
            .seam
            .is_some_and(|s| !s.is_finite() || s < self.domain[0] || s > self.domain[1])
        {
            return Err(Error::Invalid("curve seam"));
        }
        // Periodicity is retained as representation metadata. Operations requiring
        // verified periodic continuity must additionally admit their family.
        Ok(usage)
    }
    pub fn convert_lengths(&mut self, unit: Unit) -> Result<()> {
        // Validate all conversions before changing even a private value.
        for p in &self.poles {
            for v in p {
                unit.length_to_mm(*v)?;
            }
        }
        for p in &mut self.poles {
            for v in p {
                *v *= unit.factor();
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NurbsSurface {
    pub degree: [usize; 2],
    pub size: [usize; 2],
    /// U-major rows: index = u * size[1] + v.
    pub poles: Vec<Point3>,
    pub weights: Vec<f64>,
    pub knots: [Vec<f64>; 2],
    pub domain: [[f64; 2]; 2],
    pub periodic: [bool; 2],
    pub reversed: bool,
}
impl NurbsSurface {
    /// Admit periodic axes represented by repeated pole rows and extended knots.
    pub fn certify_repeated_periodic_basis(&self, l: &Limits) -> Result<()> {
        self.validate(l)?;
        for axis in 0..2 {
            if !self.periodic[axis] {continue;}
            let degree=self.degree[axis];
            let count=self.size[axis];
            if count<=degree.checked_mul(2).ok_or(Error::Overflow("periodic surface degree"))? {
                return Err(Error::Unsupported("periodic surface basis is not certified"));
            }
            let period=count-degree;
            let knots=&self.knots[axis];
            let span=knots[count]-knots[degree];
            if !span.is_finite() || span<=0. {return Err(Error::Unsupported("periodic surface basis span"));}
            let seam=knots[degree];
            let multiplicity=knots.partition_point(|k|*k<=seam)-knots.partition_point(|k|*k<seam);
            if multiplicity>degree {return Err(Error::Unsupported("periodic surface seam is discontinuous"));}
            for i in 0..degree {
                for j in 0..self.size[1-axis] {
                    let (a,b)=if axis==0 {(i*self.size[1]+j,(period+i)*self.size[1]+j)}
                        else {(j*self.size[1]+i,j*self.size[1]+period+i)};
                    if self.poles[a]!=self.poles[b] || self.weights[a]!=self.weights[b] {
                        return Err(Error::Unsupported("periodic surface control rows do not repeat"));
                    }
                }
            }
            for i in 0..knots.len()-period {
                let left=knots[i];let right=knots[i+period];
                let roundoff=2.*f64::EPSILON*left.abs().max(right.abs()).max(span.abs());
                if roundoff>span.abs()*1e-5 || ((right-left)-span).abs()>span.abs()*1e-12+roundoff {
                    return Err(Error::Unsupported("periodic surface knots do not repeat"));
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        let size = self.size[0]
            .checked_mul(self.size[1])
            .ok_or(Error::Overflow("surface pole grid"))?;
        if size != self.poles.len() {
            return Err(Error::Invalid("surface pole grid"));
        }
        for i in 0..2 {
            axis(
                self.degree[i],
                self.size[i],
                &self.knots[i],
                self.domain[i],
                l,
            )?;
        }
        let mut usage = poles_weights(&self.poles, &self.weights, l)?;
        usage.add(
            Usage::spare::<Point3>(self.poles.len(), self.poles.capacity())?,
            l,
        )?;
        usage.add(
            Usage::spare::<f64>(self.weights.len(), self.weights.capacity())?,
            l,
        )?;
        for knots in &self.knots {
            usage.add(Usage::array::<f64>(knots.capacity())?, l)?;
        }
        Ok(usage)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Curve {
    Line {
        start: Point3,
        end: Point3,
        domain: [f64; 2],
    },
    Circle {
        center: Point3,
        x: Point3,
        y: Point3,
        radius: f64,
        domain: [f64; 2],
        /// Native parameters can differ from angles. None keeps radians.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        angle_domain: Option<[f64; 2]>,
    },
    Nurbs(NurbsCurve),
    Polycurve {
        segments: Vec<Curve>,
        reversed: bool,
    },
}
impl Curve {
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        self.validate_depth(limits, 0)
    }
    fn validate_depth(&self, l: &Limits, depth: usize) -> Result<Usage> {
        if depth > l.max_depth {
            return Err(Error::Budget("curve depth"));
        }
        let mut total = Usage {
            nodes: 1,
            ..Usage::default()
        };
        match self {
            Self::Line { start, end, domain } => {
                finite(*start)?;
                finite(*end)?;
                valid_domain(*domain)?;
                if start == end {
                    return Err(Error::Invalid("degenerate line"));
                }
            }
            Self::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => {
                finite(*center)?;
                finite(*x)?;
                finite(*y)?;
                valid_domain(*domain)?;
                if let Some(angles) = angle_domain {
                    valid_domain(*angles)?;
                    let rate = (angles[1] - angles[0]) / (domain[1] - domain[0]);
                    if !rate.is_finite() || rate <= 0. {
                        return Err(Error::Invalid("circle angular parameter map"));
                    }
                }
                if !radius.is_finite() || *radius <= 0. {
                    return Err(Error::Invalid("radius"));
                }
                let dot = |a: Point3, b: Point3| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
                // Frame-normalization tolerance concerns dimensionless vectors,
                // not a universal geometric solver tolerance.
                if (dot(*x, *x) - 1.).abs() > 1e-12
                    || (dot(*y, *y) - 1.).abs() > 1e-12
                    || dot(*x, *y).abs() > 1e-12
                {
                    return Err(Error::Invalid("circle orthonormal frame"));
                }
            }
            Self::Nurbs(c) => total.add(c.validate(l)?, l)?,
            Self::Polycurve { segments, .. } => {
                if segments.is_empty() {
                    return Err(Error::Invalid("empty polycurve"));
                }
                total.add(Usage::array::<Curve>(segments.capacity())?, l)?;
                for c in segments {
                    total.add(c.validate_depth(l, depth + 1)?, l)?;
                }
            }
        }
        total.check(l)?;
        Ok(total)
    }
}
pub fn valid_domain(d: [f64; 2]) -> Result<()> {
    if !d.iter().all(|v| v.is_finite()) || d[0] >= d[1] || !(d[1] - d[0]).is_finite() {
        return Err(Error::Invalid("parameter domain"));
    }
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    pub vertices: Vec<Point3>,
    pub faces: Vec<Vec<u32>>,
    pub normals: Vec<Point3>,
    pub colors: Vec<[u8; 4]>,
    /// Per-corner UVs in face order (3DM-style). Empty when the mesh has no UV.
    #[serde(default)]
    pub corner_uvs: Vec<[f64; 2]>,
}
impl Mesh {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        let mut usage = channels(&self.vertices, &self.normals, &self.colors, l)?;
        usage.add(
            Usage::spare::<Point3>(self.vertices.len(), self.vertices.capacity())?,
            l,
        )?;
        usage.add(
            Usage::spare::<Point3>(self.normals.len(), self.normals.capacity())?,
            l,
        )?;
        usage.add(
            Usage::spare::<[u8; 4]>(self.colors.len(), self.colors.capacity())?,
            l,
        )?;
        usage.add(Usage::array::<Vec<u32>>(self.faces.capacity())?, l)?;
        if !self.corner_uvs.is_empty() {
            let expected = self.faces.iter().try_fold(0usize, |sum, face| sum.checked_add(face.len()))
                .ok_or(Error::Overflow("mesh corner count"))?;
            if self.corner_uvs.len() != expected
                || self
                    .corner_uvs
                    .iter()
                    .any(|uv| !uv[0].is_finite() || !uv[1].is_finite())
            {
                return Err(Error::Invalid("mesh corner uvs"));
            }
        }
        usage.add(Usage::array::<[f64; 2]>(self.corner_uvs.capacity())?, l)?;
        let face_count = u64::try_from(self.faces.len()).map_err(|_| Error::Overflow("mesh face count"))?;
        if face_count > l.max_nodes.saturating_sub(usage.nodes) {
            return Err(Error::Budget("nodes"));
        }
        let available_bytes = l.max_bytes.saturating_sub(usage.bytes);
        let mut face_capacity = 0u64;
        for face in &self.faces {
            if !(3..=4).contains(&face.len())
                || face.iter().any(|i| *i as usize >= self.vertices.len())
            {
                return Err(Error::Invalid("mesh face indices"));
            }
            for (i, v) in face.iter().enumerate() {
                if face[..i].contains(v) {
                    return Err(Error::Invalid("repeated mesh face vertex"));
                }
            }
            let capacity = u64::try_from(face.capacity()).map_err(|_| Error::Overflow("mesh face capacity"))?;
            face_capacity = face_capacity.checked_add(capacity).ok_or(Error::Overflow("mesh face capacity"))?;
            if face_capacity > available_bytes / std::mem::size_of::<u32>() as u64 {
                return Err(Error::Budget("bytes"));
            }
        }
        let face_bytes = face_capacity.checked_mul(std::mem::size_of::<u32>() as u64)
            .ok_or(Error::Overflow("mesh face bytes"))?;
        usage.add(Usage { bytes: face_bytes, nodes: face_count, ..Usage::default() }, l)?;
        Ok(usage)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cloud {
    pub points: Vec<Point3>,
    pub normals: Vec<Point3>,
    pub colors: Vec<[u8; 4]>,
    pub hidden: Vec<bool>,
}

/// Choose an interior diagonal for a quadrilateral in its dominant-axis
/// projection. Invalid or crossed quads have no consistent triangulation.
pub fn quad_diagonal_ac(p: [[f64; 3]; 4]) -> Result<bool> {
    let cross = |a: [f64; 3], b: [f64; 3]| [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
    let delta = |a: [f64; 3], b: [f64; 3]| std::array::from_fn(|i| a[i] - b[i]);
    let n1 = cross(delta(p[1], p[0]), delta(p[2], p[0]));
    let n2 = cross(delta(p[2], p[0]), delta(p[3], p[0]));
    let normal: [f64; 3] = std::array::from_fn(|i| n1[i] + n2[i]);
    let axis = (0..3).max_by(|a,b| normal[*a].abs().total_cmp(&normal[*b].abs())).unwrap();
    if !normal[axis].is_finite() || normal[axis] == 0.0 { return Err(Error::Invalid("STL degenerate or crossed quad")); }
    let u = (axis + 1) % 3;
    let v = (axis + 2) % 3;
    let area = |a: usize, b: usize, c: usize| (p[b][u]-p[a][u])*(p[c][v]-p[a][v]) - (p[b][v]-p[a][v])*(p[c][u]-p[a][u]);
    let sign = normal[axis].signum();
    let oriented = |a,b,c| { let area = area(a,b,c); area.is_finite() && area * sign > 0.0 };
    if oriented(0, 1, 2) && oriented(0, 2, 3) { Ok(true) }
    else if oriented(0, 1, 3) && oriented(1, 2, 3) { Ok(false) }
    else { Err(Error::Invalid("STL degenerate or crossed quad")) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GeometryKind {
    Brep,
    Point,
    Curve,
    Surface,
    Mesh,
    Cloud,
    Instance,
    Hatch,
    Light,
}
impl GeometryKind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Brep => "Brep",
            Self::Point => "Point",
            Self::Curve => "Curve",
            Self::Surface => "Surface",
            Self::Mesh => "Mesh",
            Self::Cloud => "Cloud",
            Self::Instance => "Instance",
            Self::Hatch => "Hatch",
            Self::Light => "Light",
        }
    }
    /// Entity envelope decoding preserves BRep as its own canonical kind.
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag {
            "Point" => Ok(Self::Point),
            "Curve" => Ok(Self::Curve),
            "Surface" => Ok(Self::Surface),
            "Mesh" => Ok(Self::Mesh),
            "Cloud" => Ok(Self::Cloud),
            "Instance" => Ok(Self::Instance),
            "Hatch" => Ok(Self::Hatch),
            "Light" => Ok(Self::Light),
            "Brep" | "BRep" | "B-rep" => Ok(Self::Brep),
            _ => Err(Error::Unsupported("geometry kind")),
        }
    }
}
/// Opaque native B-rep source. Preserve the bytes and supplied checksum even
/// when the native type or ability to regenerate a display is not certified.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeBrepSource {
    #[serde(with = "payload_chunks")]
    pub payload: std::sync::Arc<Vec<u8>>,
    pub crc: u32,
}
/// Process-local allocation identity. A representation keeps a weak payload
/// receipt, so the address cannot be recycled while its entry is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativeBrepSourceIdentity(usize, u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeBrepRepresentationIdentity(uuid::Uuid);
impl NativeBrepSource {
    pub fn canonical_source(&self) -> Self { self.clone() }
    pub fn payload(&self) -> &[u8] { &self.payload }
    pub fn crc(&self) -> u32 { self.crc }
    pub(crate) fn allocation_key(&self) -> usize { std::sync::Arc::as_ptr(&self.payload) as usize }
    pub fn identity(&self) -> NativeBrepSourceIdentity { NativeBrepSourceIdentity(self.allocation_key(), self.crc) }
    fn weak_payload(&self) -> std::sync::Weak<Vec<u8>> { std::sync::Arc::downgrade(&self.payload) }
    pub fn storage_bytes(&self) -> Result<u64> {
        Usage::array::<u8>(self.payload.capacity())?.bytes
            .checked_add((std::mem::size_of::<Vec<u8>>() + 2 * std::mem::size_of::<usize>()) as u64)
            .ok_or(Error::Overflow("native BRep payload accounting"))
    }
    pub(crate) fn payload_allocation_key(&self) -> usize { self.allocation_key() }
    pub(crate) fn payload_allocation_bytes(&self) -> Result<u64> { self.storage_bytes() }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        if self.payload.is_empty() { return Err(Error::Invalid("native BRep payload")); }
        let usage = Usage { bytes: self.storage_bytes()?, ..Usage::default() };
        usage.check(limits)?;
        Ok(usage)
    }
}

/// Native display captured alongside one immutable payload. A structural
/// pairing does not certify the native topology or prove it can be rebuilt.
#[derive(Clone)]
pub struct NativeBrepRepresentation {
    source: std::sync::Weak<Vec<u8>>,
    crc: u32,
    display: std::sync::Arc<Mesh>,
    owner: Option<std::sync::Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>>,
    identity: NativeBrepRepresentationIdentity,
}
impl std::fmt::Debug for NativeBrepRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeBrepRepresentation").field("crc", &self.crc)
            .field("vertices", &self.display.vertices.len()).finish()
    }
}
impl NativeBrepRepresentation {
    pub(crate) fn validate_binding(&self, source: &NativeBrepSource, limits: &Limits) -> Result<()> {
        source.validate(limits)?;
        self.display.validate(limits)?;
        if !self.matches(source) || self.owner.is_none() { return Err(Error::Invalid("native BRep representation binding")); }
        Ok(())
    }
    /// Create an unowned display derived from this exact retained payload.
    /// The caller must admit and bind its allocation before publication; this
    /// pairing does not certify arbitrary opaque data as safely rebuildable.
    pub fn from_source_mesh(source: &NativeBrepSource, mesh: Mesh, limits: &Limits) -> Result<Self> {
        source.validate(limits)?;
        mesh.validate(limits)?;
        if mesh.vertices.is_empty() || mesh.faces.is_empty() {
            return Err(Error::Invalid("native BRep reconstructed display"));
        }
        Ok(Self { source: source.weak_payload(), crc: source.crc, display: std::sync::Arc::new(mesh),
            owner: None, identity: NativeBrepRepresentationIdentity(uuid::Uuid::new_v4()) })
    }
    pub fn matches(&self, source: &NativeBrepSource) -> bool {
        self.crc == source.crc && self.source.ptr_eq(&source.weak_payload())
    }
    pub fn display(&self) -> &Mesh { &self.display }
    pub fn display_weak(&self) -> std::sync::Weak<Mesh> { std::sync::Arc::downgrade(&self.display) }
    pub fn identity(&self) -> NativeBrepRepresentationIdentity { self.identity }
    /// Transitional check for an existing legacy document owner. A different
    /// mesh requires its own scene receipt and cannot reuse the legacy one.
    pub fn matches_legacy_display(&self, native: &NativeBrep) -> bool {
        self.matches(&native.canonical_source()) && std::sync::Arc::ptr_eq(&self.display, &native.display)
    }
    pub fn display_bytes(&self) -> Result<u64> { native_display_allocation_bytes(&self.display) }
    pub(crate) fn display_owned_by(&self, account: &crate::transaction::ExternalMemoryAccount) -> bool {
        let Some(owner) = &self.owner else { return false; };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        guard.belongs_to(account) && self.display_bytes().is_ok_and(|bytes|
            guard.bytes().saturating_sub(guard.pending_bytes()) >= bytes)
    }
    /// The payload can be opaque; callers must retain this representation
    /// until a native adapter has established a safe rebuild disposition.
    pub fn bind_owner(&mut self, source: &NativeBrepSource,
        owner: std::sync::Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
        account: &crate::transaction::ExternalMemoryAccount, limits: &Limits) -> Result<()> {
        if self.owner.is_some() || !self.matches(source) { return Err(Error::Invalid("native BRep representation owner")); }
        let bytes = self.display_bytes()?;
        Usage { bytes, ..Usage::default() }.check(limits)?;
        {
            let guard = owner.lock().unwrap_or_else(|e| e.into_inner());
            if !guard.belongs_to(account) || guard.bytes().saturating_sub(guard.pending_bytes()) < bytes {
                return Err(Error::Invalid("native BRep representation accounting"));
            }
        }
        self.owner = Some(owner);
        Ok(())
    }
    pub fn pin(&self) -> Result<NativeBrepDisplayPin> {
        let owner = self.owner.as_ref().ok_or(Error::Missing("native BRep display owner"))?.clone();
        Ok(NativeBrepDisplayPin { display: self.display.clone(), _owner: owner })
    }
}

/// An independent reader pins both the display and its accounting lease.
#[derive(Clone)]
pub struct NativeBrepDisplayPin {
    display: std::sync::Arc<Mesh>,
    _owner: std::sync::Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
}
impl std::fmt::Debug for NativeBrepDisplayPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeBrepDisplayPin").field("vertices", &self.display.vertices.len()).finish()
    }
}
impl NativeBrepDisplayPin {
    pub fn display(&self) -> &Mesh { &self.display }
    pub fn display_weak(&self) -> std::sync::Weak<Mesh> { std::sync::Arc::downgrade(&self.display) }
}

/// Opaque native entries are deliberately not optional-eviction candidates.
/// A future adapter disposition must first establish source-only reconstruction.
#[derive(Debug, Default)]
pub struct NativeBrepRepresentationStore {
    entries: std::collections::BTreeMap<NativeBrepSourceIdentity, NativeBrepRepresentation>,
}
impl NativeBrepRepresentationStore {
    pub(crate) fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub(crate) fn len(&self) -> usize { self.entries.len() }
    pub fn insert(&mut self, source: &NativeBrepSource, representation: NativeBrepRepresentation, limits: &Limits) -> Result<()> {
        representation.validate_binding(source, limits)?;
        self.insert_prevalidated(source, representation)
    }
    pub(crate) fn insert_prevalidated(&mut self, source: &NativeBrepSource, representation: NativeBrepRepresentation) -> Result<()> {
        if !representation.matches(source) { return Err(Error::Invalid("native BRep representation binding")); }
        // A native display may be the only usable representation of an opaque
        // payload. Never silently replace it with a different candidate.
        if self.entries.contains_key(&source.identity()) {
            return Err(Error::Invalid("duplicate native BRep representation"));
        }
        self.entries.insert(source.identity(), representation);
        Ok(())
    }
    pub fn get(&self, source: &NativeBrepSource) -> Result<&NativeBrepRepresentation> {
        self.entries.get(&source.identity())
            .filter(|representation| representation.matches(source))
            .ok_or(Error::Missing("native BRep representation requires rebuild"))
    }
    pub(crate) fn contains(&self, source: NativeBrepSourceIdentity) -> bool { self.entries.contains_key(&source) }
    pub(crate) fn remove_exact(&mut self, source: NativeBrepSourceIdentity,
        representation: NativeBrepRepresentationIdentity) -> Option<NativeBrepRepresentation> {
        if self.entries.get(&source).is_none_or(|item| item.identity != representation) { return None; }
        self.entries.remove(&source)
    }
    pub(crate) fn take_orphaned(&mut self) -> Option<NativeBrepRepresentation> {
        let key = self.entries.iter().find_map(|(key, item)|
            (item.source.strong_count() == 0)
                .then_some(*key))?;
        self.entries.remove(&key)
    }
}

/// Native adapter result: export uses the opaque payload, while `display` is
/// a tessellation. Document migration must transfer these to separate owners
/// only after establishing a safe rebuild disposition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeBrep {
    pub display: std::sync::Arc<Mesh>,
    #[serde(with = "payload_chunks")]
    pub payload: std::sync::Arc<Vec<u8>>,
    pub crc: u32,
}
mod payload_chunks {
    pub fn serialize<S: serde::Serializer>(
        bytes: &[u8],
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut sequence = serializer.serialize_seq(Some(bytes.len().div_ceil(32 * 1024)))?;
        for chunk in bytes.chunks(32 * 1024) {
            sequence.serialize_element(chunk)?;
        }
        sequence.end()
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<std::sync::Arc<Vec<u8>>, D::Error> {
        struct PayloadVisitor;
        impl<'de> serde::de::Visitor<'de> for PayloadVisitor {
            type Value = std::sync::Arc<Vec<u8>>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of native B-rep byte chunks")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error> {
                let mut bytes = Vec::new();
                while let Some(chunk) = sequence.next_element::<Vec<u8>>()? {
                    bytes.try_reserve_exact(chunk.len()).map_err(serde::de::Error::custom)?;
                    bytes.extend_from_slice(&chunk);
                }
                Ok(std::sync::Arc::new(bytes))
            }
        }
        deserializer.deserialize_seq(PayloadVisitor)
    }
}
impl NativeBrep {
    pub fn canonical_source(&self) -> NativeBrepSource {
        NativeBrepSource { payload: self.payload.clone(), crc: self.crc }
    }
    /// Transfer the adapter result without copying either allocation. The
    /// caller must retain the display until a native rebuild disposition and
    /// an independently accounted representation owner are established.
    pub fn into_source_and_display(self) -> (NativeBrepSource, std::sync::Arc<Mesh>) {
        (NativeBrepSource { payload: self.payload, crc: self.crc }, self.display)
    }
    pub fn into_source_and_representation(self) -> (NativeBrepSource, NativeBrepRepresentation) {
        let source = NativeBrepSource { payload: self.payload, crc: self.crc };
        let representation = NativeBrepRepresentation { source: source.weak_payload(), crc: source.crc,
            display: self.display, owner: None, identity: NativeBrepRepresentationIdentity(uuid::Uuid::new_v4()) };
        (source, representation)
    }
    pub(crate) fn display_allocation_key(&self) -> usize {
        std::sync::Arc::as_ptr(&self.display) as usize
    }
    pub(crate) fn display_allocation_bytes(&self) -> Result<u64> {
        native_display_allocation_bytes(&self.display)
    }
    pub(crate) fn payload_allocation_key(&self) -> usize {
        std::sync::Arc::as_ptr(&self.payload) as usize
    }
    pub(crate) fn payload_allocation_bytes(&self) -> Result<u64> {
        self.canonical_source().storage_bytes()
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.payload.is_empty() {
            return Err(Error::Invalid("native BRep payload"));
        }
        let mut usage = self.display.validate(l)?;
        usage.add(Usage::array::<Mesh>(1)?, l)?;
        usage.add(Usage::array::<usize>(2)?, l)?;
        usage.add(Usage { bytes: self.payload_allocation_bytes()?, ..Usage::default() }, l)?;
        Ok(usage)
    }
}
fn native_display_allocation_bytes(mesh: &Mesh) -> Result<u64> {
        let sizes = [
            Usage::array::<Mesh>(1)?.bytes,
            Usage::array::<usize>(2)?.bytes,
            Usage::array::<Point3>(mesh.vertices.capacity())?.bytes,
            Usage::array::<Point3>(mesh.normals.capacity())?.bytes,
            Usage::array::<[u8; 4]>(mesh.colors.capacity())?.bytes,
            Usage::array::<[f64; 2]>(mesh.corner_uvs.capacity())?.bytes,
            Usage::array::<Vec<u32>>(mesh.faces.capacity())?.bytes,
        ];
        let bytes = sizes.into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("native BRep display accounting")))?;
        mesh.faces.iter().try_fold(bytes, |sum, face| {
            sum.checked_add(Usage::array::<u32>(face.capacity())?.bytes)
                .ok_or(Error::Overflow("native BRep display accounting"))
        })
}

#[cfg(test)]
mod native_brep_payload_tests {
    use super::{Mesh, NativeBrep};
    use std::sync::Arc;

    #[test]
    fn mesh_validation_counts_face_capacity_and_rejects_tight_limits() {
        let mut first=Vec::with_capacity(12);first.extend([0,1,2]);
        let mut faces=Vec::with_capacity(5);faces.push(first);faces.push(vec![0,2,3]);
        let mesh=Mesh {vertices:vec![[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.]],faces,
            normals:vec![],colors:vec![],corner_uvs:vec![]};
        let limits=crate::budget::Limits::default();let usage=mesh.validate(&limits).unwrap();
        let expected=mesh.vertices.capacity()*std::mem::size_of::<[f64;3]>()+
            mesh.faces.capacity()*std::mem::size_of::<Vec<u32>>() +
            mesh.faces.iter().map(|face|face.capacity()*std::mem::size_of::<u32>()).sum::<usize>();
        assert_eq!(usage.bytes,expected as u64);assert_eq!(usage.points,4);assert_eq!(usage.nodes,2);
        let mut exact=limits;exact.max_nodes=usage.nodes;exact.max_bytes=usage.bytes;
        assert_eq!(mesh.validate(&exact).unwrap(),usage);
        let mut tight=limits;tight.max_nodes=1;assert!(matches!(mesh.validate(&tight),Err(crate::Error::Budget("nodes"))));
        tight=limits;tight.max_bytes=usage.bytes-1;assert!(matches!(mesh.validate(&tight),Err(crate::Error::Budget("bytes"))));
        let mut invalid=mesh.clone();invalid.faces[1]=vec![0,2,2];assert!(mesh.validate(&limits).is_ok() && invalid.validate(&limits).is_err());
        invalid=mesh.clone();invalid.faces[1]=vec![0,2,4];assert!(invalid.validate(&limits).is_err());
    }

    #[test]
    fn payload_is_shared_on_clone_and_keeps_legacy_chunk_encoding() {
        let source = NativeBrep {
            display: std::sync::Arc::new(Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3, 4].into(), crc: 17,
        };
        let copy = source.clone();
        assert!(Arc::ptr_eq(&source.payload, &copy.payload));
        assert!(Arc::ptr_eq(&source.display, &copy.display));
        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["payload"], serde_json::json!([[1, 2, 3, 4]]));
        let decoded: NativeBrep = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, source);
        assert!(!Arc::ptr_eq(&decoded.payload, &source.payload));
        assert!(!Arc::ptr_eq(&decoded.display, &source.display));
    }
    #[test]
    fn cloned_display_is_copy_on_write() {
        let source = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0., 0., 0.]], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1].into(), crc: 0,
        };
        let mut copy = source.clone();
        Arc::make_mut(&mut copy.display).vertices[0] = [1., 0., 0.];
        assert_eq!(source.display.vertices[0], [0., 0., 0.]);
        assert_eq!(copy.display.vertices[0], [1., 0., 0.]);
        assert!(!Arc::ptr_eq(&source.display, &copy.display));
        assert!(Arc::ptr_eq(&source.payload, &copy.payload));
    }
    #[test]
    fn chunked_payload_preserves_boundaries_and_accepts_legacy_irregular_chunks() {
        let source = NativeBrep {
            display: std::sync::Arc::new(Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![23; 65_537].into(), crc: 42,
        };
        let mut encoded = serde_json::to_value(&source).unwrap();
        let chunks = encoded["payload"].as_array().unwrap();
        assert_eq!(chunks.iter().map(|chunk| chunk.as_array().unwrap().len()).collect::<Vec<_>>(), [32_768, 32_768, 1]);
        let decoded: NativeBrep = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded, source);
        encoded["payload"] = serde_json::json!([[1, 2], [], [3, 4, 5]]);
        let legacy: NativeBrep = serde_json::from_value(encoded).unwrap();
        assert_eq!(&*legacy.payload, &[1, 2, 3, 4, 5]);
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Light {
    pub display: Point3,
    pub location: Point3,
    pub direction: Point3,
    pub diffuse: [u8; 4],
    pub intensity: f64,
    pub style: u32,
}
impl Light {
    pub fn validate(&self, _l: &Limits) -> Result<Usage> {
        finite(self.display)?;
        finite(self.location)?;
        finite(self.direction)?;
        if !self.intensity.is_finite() || self.intensity < 0. {
            return Err(Error::Invalid("light intensity"));
        }
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hatch {
    pub display: Mesh,
    pub loops: Vec<Vec<Point3>>,
    pub pattern: Option<String>,
    pub color: [u8; 4],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    #[serde(alias = "BRep", alias = "B-rep")]
    Brep(crate::brep::CanonicalBrep),
    NativeBrep(NativeBrepSource),
    Point(Point3),
    Curve(Curve),
    Surface(NurbsSurface),
    Mesh(Mesh),
    Cloud(Cloud),
    Instance(DefinitionId),
    Hatch(Hatch),
    Light(Light),
}
fn channels(
    points: &[Point3],
    normals: &[Point3],
    colors: &[[u8; 4]],
    l: &Limits,
) -> Result<Usage> {
    let mut usage = Usage::array::<Point3>(points.len())?;
    usage.points = u64::try_from(points.len()).map_err(|_| Error::Overflow("point count"))?;
    usage.add(Usage::array::<Point3>(normals.len())?, l)?;
    usage.add(Usage::array::<[u8; 4]>(colors.len())?, l)?;
    for n in [normals.len(), colors.len()] {
        if n != 0 && n != points.len() {
            return Err(Error::Invalid("channel count"));
        }
    }
    for p in points.iter().chain(normals) {
        finite(*p)?;
    }
    Ok(usage)
}
impl Geometry {
    fn validate_mesh(m: &Mesh, l: &Limits) -> Result<Usage> {
        let mut total = Usage { bytes: std::mem::size_of::<Self>() as u64, ..Usage::default() };
        total.add(m.validate(l)?, l)?;
        Ok(total)
    }
    pub fn kind(&self) -> GeometryKind {
        match self {
            Self::Brep(_) | Self::NativeBrep(_) => GeometryKind::Brep,
            Self::Point(_) => GeometryKind::Point,
            Self::Curve(_) => GeometryKind::Curve,
            Self::Surface(_) => GeometryKind::Surface,
            Self::Mesh(_) => GeometryKind::Mesh,
            Self::Cloud(_) => GeometryKind::Cloud,
            Self::Instance(_) => GeometryKind::Instance,
            Self::Hatch(_) => GeometryKind::Hatch,
            Self::Light(_) => GeometryKind::Light,
        }
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        let mut total = Usage {
            bytes: std::mem::size_of::<Self>() as u64,
            ..Usage::default()
        };
        match self {
            Self::Brep(shape) => {
                if shape.has_allocation_owner() {
                    return Err(Error::Unsupported("transient leased BRep geometry"));
                }
                total.add(shape.validate(l)?, l)?;
            }
            Self::NativeBrep(shape) => total.add(shape.validate(l)?, l)?,
            Self::Point(p) => {
                finite(*p)?;
                total.points = 1;
            }
            Self::Curve(c) => total.add(c.validate_depth(l, 0)?, l)?,
            Self::Surface(s) => total.add(s.validate(l)?, l)?,
            Self::Instance(id) => {
                if id.is_nil() {
                    return Err(Error::Invalid("definition ID"));
                }
            }
            Self::Cloud(c) => {
                total.add(channels(&c.points, &c.normals, &c.colors, l)?, l)?;
                if !c.hidden.is_empty() && c.hidden.len() != c.points.len() {
                    return Err(Error::Invalid("cloud hidden channel"));
                }
                total.add(
                    Usage::spare::<Point3>(c.points.len(), c.points.capacity())?,
                    l,
                )?;
                total.add(
                    Usage::spare::<Point3>(c.normals.len(), c.normals.capacity())?,
                    l,
                )?;
                total.add(
                    Usage::spare::<[u8; 4]>(c.colors.len(), c.colors.capacity())?,
                    l,
                )?;
                total.add(Usage::array::<bool>(c.hidden.capacity())?, l)?;
            }
            Self::Light(light) => {
                total.add(light.validate(l)?, l)?;
            }
            Self::Hatch(h) => {
                total.add(Self::validate_mesh(&h.display, l)?, l)?;
                total.add(Usage::array::<Vec<Point3>>(h.loops.capacity())?, l)?;
                for loop_ in &h.loops {
                    if loop_.len() < 3 {
                        return Err(Error::Invalid("hatch loop"));
                    }
                    for point in loop_ {
                        finite(*point)?;
                    }
                    total.add(Usage::array::<Point3>(loop_.capacity())?, l)?;
                }
                if let Some(pattern) = &h.pattern {
                    name(pattern, l)?;
                    total.add(Usage { bytes: pattern.capacity() as u64, ..Usage::default() }, l)?;
                }
            }
            Self::Mesh(m) => return Self::validate_mesh(m, l),
        }
        total.check(l)?;
        Ok(total)
    }
}
