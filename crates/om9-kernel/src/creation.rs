// SPDX-License-Identifier: LGPL-2.1-or-later
//! Geometry creation operations (M03 `OM9-L2-01-09`): analytic primitives, extrusion, revolution,
//! pipe, planar boundary, loft, sweep and edge-curve surfaces, plus the general profile-to-B-Rep
//! adapter.
//!
//! Safe Rust owns the typed contract, every input rule (planarity, simplicity, winding, frame
//! continuity, rail separation, budgets), the sweep/pipe section math, the handle session and the
//! measured oracles. C++ is limited to the OCCT construction adapter in `creation_bridge.cpp`:
//! it maps already-validated data onto `BRepPrimAPI_*` / `BRepOffsetAPI_*` and contains exceptions.
//!
//! Every operation is one complete call returning one owned [`Brep`]; a refused input returns a
//! [`CreationError`] and never a partial body, and a released handle is stale rather than aliasing
//! the body that reuses its slot.
//!
//! Units are millimetres, angles are radians, tolerances are absolute millimetres.
use crate::{
    KernelError,
    brep::{snapshot_budget, snapshot_via},
};
use om9_core::{
    brep::Brep,
    brep_kernel::{HandleStore, KernelHandle},
    budget::Limits,
    geometry::Point3,
};
use std::{
    ffi::{CStr, c_char, c_int, c_void},
    fmt,
};
use uuid::Uuid;

/// The display deflection requested by the creation adapters when the caller has no opinion.
pub const DEFAULT_DEFLECTION_MM: f64 = 0.01;
/// Absolute geometric tolerance used to admit profiles, coplanarity and boundary continuity.
pub const GEOMETRIC_TOLERANCE_MM: f64 = 1.0e-6;
/// Hard cap on one profile/section point count so validation stays bounded before the kernel call.
const MAX_PROFILE_POINTS: usize = 4096;

/// The typed refusal contract of every creation operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationError {
    /// Typed input refusal: nothing was built and no document state changed.
    Invalid(String),
    /// A measured budget or allocation refusal.
    Budget(String),
    /// The kernel adapter refused or failed after the input was admitted.
    Kernel(String),
    /// The body handle is no longer alive.
    StaleHandle,
}

impl fmt::Display for CreationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "invalid creation input: {message}"),
            Self::Budget(message) => write!(formatter, "creation budget exceeded: {message}"),
            Self::Kernel(message) => write!(formatter, "creation kernel failure: {message}"),
            Self::StaleHandle => write!(formatter, "creation handle is stale"),
        }
    }
}

impl std::error::Error for CreationError {}

impl From<KernelError> for CreationError {
    fn from(value: KernelError) -> Self {
        match value {
            KernelError::InvalidInput(message) => Self::Invalid(message),
            KernelError::Failure(message) => Self::Kernel(message),
        }
    }
}

/* ------------------------- native adapters ------------------------- */

#[repr(C)]
struct MeasuresOut {
    volume_mm3: f64,
    area_mm2: f64,
    faces: i64,
    edges: i64,
    vertices: i64,
    solids: i64,
    shells: i64,
    naked_edges: i64,
    closed: c_int,
    bounds_mm: [f64; 6],
}

type NativeSink = unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int;

unsafe extern "C" {
    fn om9_occt_primitive(
        kind: c_int,
        params: *const f64,
        count: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_extrude(
        points: *const f64,
        count: usize,
        direction: *const f64,
        capped: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_revolve(
        points: *const f64,
        count: usize,
        axis: *const f64,
        angle: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sew_solid(
        points: *const f64,
        point_count: usize,
        quads: *const i32,
        quad_count: usize,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_planar(
        points: *const f64,
        count: usize,
        plane: *const f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_loft(
        points: *const f64,
        count: usize,
        sections: *const usize,
        section_count: usize,
        solid: c_int,
        ruled: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_pipe_shell(
        spine: *const f64,
        spine_count: usize,
        aux: *const f64,
        aux_count: usize,
        points: *const f64,
        count: usize,
        sections: *const usize,
        section_count: usize,
        solid: c_int,
        aux_mode: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_four_rail_sweep(
        rails: *const f64,
        rail_count: usize,
        rail_table: *const usize,
        sections: *const f64,
        section_count: usize,
        section_table: *const usize,
        section_ranges: *const usize,
        solid: c_int,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_tube(
        stations: *const f64,
        station_count: usize,
        radius_mm: f64,
        solid: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_tube_radii(
        stations: *const f64,
        radii: *const f64,
        station_count: usize,
        solid: c_int,
        ruled: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_round_pipe(
        endpoints: *const f64, start_radius: f64, end_radius: f64, deflection: f64,
        max_points: usize, max_nodes: usize, max_bytes: usize, sink: NativeSink,
        context: *mut c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_hemisphere(frame: *const f64, radius: f64, forward: c_int, deflection: f64,
        max_points: usize, max_nodes: usize, max_bytes: usize, sink: NativeSink,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_pipe_miter(path: *const f64, count: usize, radius: f64, solid: c_int,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: NativeSink, context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_surface(
        points: *const f64,
        count: usize,
        curves: *const usize,
        curve_count: usize,
        tolerance: f64,
        deviation: *mut f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_measure(
        source: *const u8,
        length: usize,
        volume: *mut f64,
        area: *mut f64,
        faces: *mut i64,
        edges: *mut i64,
        vertices: *mut i64,
        solids: *mut i64,
        shells: *mut i64,
        naked_edges: *mut i64,
        closed: *mut c_int,
        bounds: *mut f64,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}

fn native_message(message: &[c_char; 512]) -> String {
    // SAFETY: every adapter NUL-terminates within the fixed buffer it was given.
    unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn checked_deflection(deflection_mm: f64) -> Result<f64, CreationError> {
    if !deflection_mm.is_finite() || deflection_mm <= 0.0 {
        return Err(CreationError::Invalid(
            "deflection must be finite and positive".into(),
        ));
    }
    Ok(deflection_mm)
}

fn flatten(points: &[Point3]) -> Vec<f64> {
    let mut flat = Vec::with_capacity(points.len() * 3);
    for point in points {
        flat.extend_from_slice(point);
    }
    flat
}

fn table(offsets: &[(usize, usize)]) -> Vec<usize> {
    let mut flat = Vec::with_capacity(offsets.len() * 2);
    for (offset, length) in offsets {
        flat.push(*offset);
        flat.push(*length);
    }
    flat
}

/* ------------------------- vector helpers ------------------------- */

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(value: Point3) -> f64 {
    dot(value, value).sqrt()
}

fn distance(a: Point3, b: Point3) -> f64 {
    length(sub(a, b))
}

fn unit(value: Point3, what: &str) -> Result<Point3, CreationError> {
    let magnitude = length(value);
    if !magnitude.is_finite() || magnitude <= 1.0e-12 {
        return Err(CreationError::Invalid(format!("{what} is degenerate")));
    }
    Ok([
        value[0] / magnitude,
        value[1] / magnitude,
        value[2] / magnitude,
    ])
}

fn finite(point: Point3, what: &str) -> Result<(), CreationError> {
    if point.iter().any(|value| !value.is_finite()) {
        return Err(CreationError::Invalid(format!(
            "{what} contains a non-finite coordinate"
        )));
    }
    Ok(())
}

fn positive(value: f64, what: &str) -> Result<(), CreationError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(CreationError::Invalid(format!(
            "{what} must be finite and positive"
        )));
    }
    Ok(())
}

fn non_negative(value: f64, what: &str) -> Result<(), CreationError> {
    if !value.is_finite() || value < 0.0 {
        return Err(CreationError::Invalid(format!(
            "{what} must be finite and not negative"
        )));
    }
    Ok(())
}

/* ------------------------- primitives ------------------------- */

/// An analytic solid primitive. Every size is millimetres; the construction is placed as the OCCT
/// factory places it (box corner at the origin, sphere at the origin, cone base at z = 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    Box {
        x_mm: f64,
        y_mm: f64,
        z_mm: f64,
    },
    Sphere {
        radius_mm: f64,
    },
    Cylinder {
        radius_mm: f64,
        height_mm: f64,
    },
    Cone {
        bottom_radius_mm: f64,
        top_radius_mm: f64,
        height_mm: f64,
    },
    Torus {
        ring_radius_mm: f64,
        tube_radius_mm: f64,
    },
    Ellipsoid {
        x_mm: f64,
        y_mm: f64,
        z_mm: f64,
    },
    Pyramid {
        sides: u32,
        base_radius_mm: f64,
        height_mm: f64,
    },
}

impl Primitive {
    /// The primitive's own domain rule. Rejecting here means the kernel never sees the input.
    pub fn validate(self) -> Result<(), CreationError> {
        match self {
            Self::Box { x_mm, y_mm, z_mm } => {
                positive(x_mm, "box x")?;
                positive(y_mm, "box y")?;
                positive(z_mm, "box z")?;
            }
            Self::Sphere { radius_mm } => positive(radius_mm, "sphere radius")?,
            Self::Cylinder {
                radius_mm,
                height_mm,
            } => {
                positive(radius_mm, "cylinder radius")?;
                positive(height_mm, "cylinder height")?;
            }
            Self::Cone {
                bottom_radius_mm,
                top_radius_mm,
                height_mm,
            } => {
                positive(bottom_radius_mm, "cone bottom radius")?;
                non_negative(top_radius_mm, "cone top radius")?;
                positive(height_mm, "cone height")?;
            }
            Self::Torus {
                ring_radius_mm,
                tube_radius_mm,
            } => {
                positive(ring_radius_mm, "torus ring radius")?;
                positive(tube_radius_mm, "torus tube radius")?;
                if tube_radius_mm >= ring_radius_mm {
                    return Err(CreationError::Invalid(
                        "torus tube radius must be smaller than the ring radius".into(),
                    ));
                }
            }
            Self::Ellipsoid { x_mm, y_mm, z_mm } => {
                positive(x_mm, "ellipsoid x semi axis")?;
                positive(y_mm, "ellipsoid y semi axis")?;
                positive(z_mm, "ellipsoid z semi axis")?;
            }
            Self::Pyramid {
                sides,
                base_radius_mm,
                height_mm,
            } => {
                if sides < 3 {
                    return Err(CreationError::Invalid(
                        "pyramid needs at least three sides".into(),
                    ));
                }
                positive(base_radius_mm, "pyramid base radius")?;
                positive(height_mm, "pyramid height")?;
            }
        }
        Ok(())
    }

    fn native(self) -> (c_int, [f64; 3], usize) {
        match self {
            Self::Box { x_mm, y_mm, z_mm } => (0, [x_mm, y_mm, z_mm], 3),
            Self::Sphere { radius_mm } => (1, [radius_mm, 0.0, 0.0], 1),
            Self::Cylinder {
                radius_mm,
                height_mm,
            } => (2, [radius_mm, height_mm, 0.0], 2),
            Self::Cone {
                bottom_radius_mm,
                top_radius_mm,
                height_mm,
            } => (3, [bottom_radius_mm, top_radius_mm, height_mm], 3),
            Self::Torus {
                ring_radius_mm,
                tube_radius_mm,
            } => (4, [ring_radius_mm, tube_radius_mm, 0.0], 2),
            Self::Ellipsoid { x_mm, y_mm, z_mm } => (5, [x_mm, y_mm, z_mm], 3),
            Self::Pyramid {
                sides,
                base_radius_mm,
                height_mm,
            } => (6, [f64::from(sides), base_radius_mm, height_mm], 3),
        }
    }
}

/// Build one analytic primitive as an exact B-Rep.
pub fn primitive(
    primitive: Primitive,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    primitive.validate()?;
    let deflection = checked_deflection(deflection_mm)?;
    let (kind, scalars, count) = primitive.native();
    // Ellipsoid poles/weights are independent mathematical data, owned by safe Rust.
    // The native adapter only constructs the OCCT surface and closed topology.
    let net;
    let params = if let Primitive::Ellipsoid { x_mm, y_mm, z_mm } = primitive {
        net = ellipsoid_net([x_mm, y_mm, z_mm]);
        &net[..]
    } else {
        &scalars[..count]
    };
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_primitive(
                kind,
                params.as_ptr(),
                params.len(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Build a pyramid over a planar, alternating-radius star profile in the XY plane.
/// Rust owns point generation, validation and bounded allocation; OCCT builds the topology.
pub fn star_pyramid(
    sides: u32, first_radius: f64, second_radius: f64, height: f64,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, CreationError> {
    if sides < 3 {
        return Err(CreationError::Invalid("star needs at least three points".into()));
    }
    positive(first_radius, "star first radius")?;
    positive(second_radius, "star second radius")?;
    positive(height, "star height")?;
    let count = (sides as usize).checked_mul(2)
        .ok_or_else(|| CreationError::Budget("star vertex count".into()))?;
    if count > MAX_PROFILE_POINTS || count as u64 > limits.max_points
        || (count as u64).saturating_mul(512) > limits.max_bytes {
        return Err(CreationError::Budget("star vertex budget".into()));
    }
    let mut points = Vec::new();
    points.try_reserve_exact(count).map_err(|_| CreationError::Budget("star vertices".into()))?;
    for i in 0..count {
        let angle = std::f64::consts::TAU * i as f64 / count as f64;
        let r = if i % 2 == 0 { first_radius } else { second_radius };
        points.push([r * angle.cos(), r * angle.sin(), 0.]);
    }
    let profile = Profile::new(points)?;
    let deflection = checked_deflection(deflection_mm)?;
    let mut params = Vec::new();
    params.try_reserve_exact(1 + 3 * count).map_err(|_| CreationError::Budget("star parameters".into()))?;
    params.push(height);
    for p in profile.points() { params.extend_from_slice(p); }
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: validated parameter storage and callback context live through this synchronous call.
        unsafe { om9_occt_primitive(7, params.as_ptr(), params.len(), deflection, budget.points,
            budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(CreationError::from)
}

/// Tensor product of four rational quarter circles around Z and two along the meridian.
/// Each quadratic arc has endpoint weights 1 and middle weight sqrt(1/2); its middle
/// pole is the tangent intersection, not a point on the circle. Products of arc weights
/// give the exact surface of revolution. Diagonal scaling of poles preserves rational
/// evaluation and gives (x/a)^2 + (y/b)^2 + (z/c)^2 = 1, up to f64 roundoff.
/// Packed U-major: 45 XYZ poles, followed by 45 positive weights. No converter/fitting.
fn ellipsoid_net(axes: [f64; 3]) -> [f64; 180] {
    let circle = [
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [-1.0, 1.0],
        [-1.0, 0.0],
        [-1.0, -1.0],
        [0.0, -1.0],
        [1.0, -1.0],
        [1.0, 0.0],
    ];
    let meridian = [[0.0, -1.0], [1.0, -1.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let arc_weight = |i: usize| {
        if i.is_multiple_of(2) {
            1.0
        } else {
            std::f64::consts::FRAC_1_SQRT_2
        }
    };
    let mut net = [0.0; 180];
    for (u, [x, y]) in circle.into_iter().enumerate() {
        for (v, [radius, z]) in meridian.into_iter().enumerate() {
            let index = u * 5 + v;
            net[3 * index..3 * index + 3].copy_from_slice(&[
                axes[0] * x * radius,
                axes[1] * y * radius,
                axes[2] * z,
            ]);
            net[135 + index] = arc_weight(u) * arc_weight(v);
        }
    }
    net
}

/* ------------------------- profiles ------------------------- */

/// A validated, closed, planar and simple profile. The closing point may be given explicitly and
/// is normalised away; the stored points never repeat the first one.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    points: Vec<Point3>,
    normal: Point3,
    centroid: Point3,
}

impl Profile {
    pub fn new(points: impl Into<Vec<Point3>>) -> Result<Self, CreationError> {
        let (points, normal, centroid) = validate_closed_profile(&points.into(), "profile")?;
        Ok(Self {
            points,
            normal,
            centroid,
        })
    }

    pub fn points(&self) -> &[Point3] {
        &self.points
    }

    pub fn normal(&self) -> Point3 {
        self.normal
    }

    pub fn centroid(&self) -> Point3 {
        self.centroid
    }

    /// Exact planar area from the Newell vector, in mm².
    pub fn area_mm2(&self) -> f64 {
        0.5 * length(newell(&self.points))
    }
}

fn newell(points: &[Point3]) -> Point3 {
    let mut normal = [0.0; 3];
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    normal
}

fn plane_basis(normal: Point3) -> Result<(Point3, Point3), CreationError> {
    let candidate = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = unit(cross(normal, candidate), "profile basis")?;
    let v = cross(normal, u);
    Ok((u, v))
}

fn project(points: &[Point3], u: Point3, v: Point3) -> Vec<[f64; 2]> {
    points
        .iter()
        .map(|point| [dot(*point, u), dot(*point, v)])
        .collect()
}

fn validate_closed_profile(
    points: &[Point3],
    what: &str,
) -> Result<(Vec<Point3>, Point3, Point3), CreationError> {
    if points.len() > MAX_PROFILE_POINTS {
        return Err(CreationError::Budget(format!(
            "{what} has more than {MAX_PROFILE_POINTS} points"
        )));
    }
    let mut points = points.to_vec();
    if points.len() < 3 {
        return Err(CreationError::Invalid(format!(
            "{what} needs at least three points"
        )));
    }
    for point in &points {
        finite(*point, what)?;
    }
    if distance(points[0], points[points.len() - 1]) <= GEOMETRIC_TOLERANCE_MM {
        points.pop();
    }
    if points.len() < 3 {
        return Err(CreationError::Invalid(format!(
            "{what} needs at least three distinct points"
        )));
    }
    for window in points.windows(2) {
        if distance(window[0], window[1]) <= GEOMETRIC_TOLERANCE_MM {
            return Err(CreationError::Invalid(format!("{what} repeats a point")));
        }
    }
    let newell = newell(&points);
    let area = 0.5 * length(newell);
    if !area.is_finite() || area <= 1.0e-9 {
        return Err(CreationError::Invalid(format!(
            "{what} is degenerate (zero area)"
        )));
    }
    let normal = unit(newell, &format!("{what} normal"))?;
    let count = points.len() as f64;
    let centroid = [
        points.iter().map(|point| point[0]).sum::<f64>() / count,
        points.iter().map(|point| point[1]).sum::<f64>() / count,
        points.iter().map(|point| point[2]).sum::<f64>() / count,
    ];
    for point in &points {
        if dot(sub(*point, centroid), normal).abs() > GEOMETRIC_TOLERANCE_MM {
            return Err(CreationError::Invalid(format!("{what} is not planar")));
        }
    }
    let (u, v) = plane_basis(normal)?;
    let flat = project(&points, u, v);
    if !simple_polygon(&flat) {
        return Err(CreationError::Invalid(format!(
            "{what} is self-intersecting"
        )));
    }
    Ok((points, normal, centroid))
}

fn orient(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn on_segment(a: [f64; 2], b: [f64; 2], point: [f64; 2], epsilon: f64) -> bool {
    point[0] >= a[0].min(b[0]) - epsilon
        && point[0] <= a[0].max(b[0]) + epsilon
        && point[1] >= a[1].min(b[1]) - epsilon
        && point[1] <= a[1].max(b[1]) + epsilon
}

fn segments_intersect(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let epsilon = 1.0e-12;
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let straddle = |p: f64, q: f64| (p > epsilon && q < -epsilon) || (p < -epsilon && q > epsilon);
    if straddle(o1, o2) && straddle(o3, o4) {
        return true;
    }
    (o1.abs() <= epsilon && on_segment(a, b, c, epsilon))
        || (o2.abs() <= epsilon && on_segment(a, b, d, epsilon))
        || (o3.abs() <= epsilon && on_segment(c, d, a, epsilon))
        || (o4.abs() <= epsilon && on_segment(c, d, b, epsilon))
}

fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1])
}

/// A simple polygon has no two non-adjacent edges touching and no adjacent edge folding back.
fn simple_polygon(points: &[[f64; 2]]) -> bool {
    let count = points.len();
    if count < 4 {
        return true;
    }
    for first in 0..count {
        let a = points[first];
        let b = points[(first + 1) % count];
        if distance2(a, b) <= 1.0e-18 {
            return false;
        }
        for second in (first + 1)..count {
            let adjacent = (second + 1) % count == first || (first + 1) % count == second;
            if adjacent {
                continue;
            }
            let c = points[second];
            let d = points[(second + 1) % count];
            if segments_intersect(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

/* ------------------------- operations ------------------------- */

/// Extrude a closed planar profile along a direction. `capped` builds the closed solid; without it
/// the adapter returns the open side shell.
pub fn extrude(
    profile: &Profile,
    direction_mm: Point3,
    capped: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let deflection = checked_deflection(deflection_mm)?;
    let magnitude = length(direction_mm);
    if !magnitude.is_finite() || magnitude <= 1.0e-12 {
        return Err(CreationError::Invalid(
            "extrusion direction must be finite and non-zero".into(),
        ));
    }
    let points = flatten(&profile.points);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_extrude(
                points.as_ptr(),
                profile.points.len(),
                direction_mm.as_ptr(),
                if capped { 1 } else { 2 },
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Extrude an open polyline into the open side shell (no face is invented for the boundary).
pub fn extrude_wire(
    polyline: &[Point3],
    direction_mm: Point3,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if polyline.len() < 2 {
        return Err(CreationError::Invalid(
            "open extrusion needs at least two points".into(),
        ));
    }
    if polyline.len() > MAX_PROFILE_POINTS {
        return Err(CreationError::Budget("open extrusion point count".into()));
    }
    for point in polyline {
        finite(*point, "open profile")?;
    }
    if polyline
        .windows(2)
        .any(|window| distance(window[0], window[1]) <= GEOMETRIC_TOLERANCE_MM)
    {
        return Err(CreationError::Invalid(
            "open profile repeats a point".into(),
        ));
    }
    let magnitude = length(direction_mm);
    if !magnitude.is_finite() || magnitude <= 1.0e-12 {
        return Err(CreationError::Invalid(
            "extrusion direction must be finite and non-zero".into(),
        ));
    }
    let deflection = checked_deflection(deflection_mm)?;
    let points = flatten(polyline);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_extrude(
                points.as_ptr(),
                polyline.len(),
                direction_mm.as_ptr(),
                0,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Revolve a closed planar profile around an axis. A profile that crosses the axis is refused: the
/// resulting solid would self-intersect.
pub fn revolve(
    profile: &Profile,
    axis_origin_mm: Point3,
    axis_direction: Point3,
    angle_radians: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let axis = unit(axis_direction, "revolution axis").map_err(|_| {
        CreationError::Invalid("revolution axis direction must be finite and non-zero".into())
    })?;
    finite(axis_origin_mm, "revolution axis origin")?;
    if !angle_radians.is_finite()
        || angle_radians.abs() <= 1.0e-12
        || angle_radians.abs() > std::f64::consts::TAU + 1.0e-9
    {
        return Err(CreationError::Invalid(
            "revolution angle must be in (0, 2π]".into(),
        ));
    }
    // Radial vectors are compared against the first non-degenerate one: a change of sign means the
    // profile passes through the axis, which would make the revolved solid self-intersect. Using a
    // fixed basis perpendicular to the axis is wrong when that basis lies in the profile plane.
    let mut reference: Option<Point3> = None;
    let mut signed: Vec<f64> = Vec::with_capacity(profile.points.len());
    for point in &profile.points {
        let offset = sub(*point, axis_origin_mm);
        let projection = dot(offset, axis);
        let radial = sub(
            offset,
            [
                axis[0] * projection,
                axis[1] * projection,
                axis[2] * projection,
            ],
        );
        if reference.is_none() && length(radial) > GEOMETRIC_TOLERANCE_MM {
            reference = Some([
                radial[0] / length(radial),
                radial[1] / length(radial),
                radial[2] / length(radial),
            ]);
        }
        signed.push(match reference {
            Some(u) => dot(radial, u),
            None => 0.0,
        });
    }
    let crosses = signed.iter().any(|value| *value > GEOMETRIC_TOLERANCE_MM)
        && signed.iter().any(|value| *value < -GEOMETRIC_TOLERANCE_MM);
    if crosses {
        return Err(CreationError::Invalid(
            "revolution profile crosses the axis".into(),
        ));
    }
    let deflection = checked_deflection(deflection_mm)?;
    let points = flatten(&profile.points);
    let axis_table = [
        axis_origin_mm[0],
        axis_origin_mm[1],
        axis_origin_mm[2],
        axis[0],
        axis[1],
        axis[2],
    ];
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_revolve(
                points.as_ptr(),
                profile.points.len(),
                axis_table.as_ptr(),
                angle_radians,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Sew planar quads into one closed solid (`OM9-L2-04-03`, `OM9-L3-04-03-007`).
///
/// Rust owns the geometry: the caller passes validated section quads whose corners are shared
/// by construction. The adapter builds one planar face per quad, sews them at `tolerance_mm`,
/// and refuses anything that is not exactly one closed shell with a positive volume, so a ring
/// body can never arrive half open. The result is one owned B-Rep like every other creation.
pub fn sewed_solid(
    points: &[Point3],
    quads: &[[u32; 4]],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if points.len() < 4 || quads.len() < 4 {
        return Err(CreationError::Invalid(
            "sew solid needs four points and four faces".into(),
        ));
    }
    if points.len() > MAX_PROFILE_POINTS {
        return Err(CreationError::Budget("sew solid point count".into()));
    }
    for point in points {
        finite(*point, "sew solid point")?;
    }
    if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
        return Err(CreationError::Invalid(
            "sew tolerance must be finite and positive".into(),
        ));
    }
    let mut index: Vec<i32> = Vec::with_capacity(quads.len() * 4);
    for quad in quads {
        for vertex in quad {
            if *vertex as usize >= points.len() {
                return Err(CreationError::Invalid("sew solid face index".into()));
            }
            index.push(*vertex as i32);
        }
    }
    let table = flatten(points);
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: both buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_sew_solid(
                table.as_ptr(),
                points.len(),
                index.as_ptr(),
                quads.len(),
                tolerance_mm,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Build a planar face from one closed outer profile on its admitted plane.
///
/// Inner loops (holes) are a separate owner task: this delivery admits one boundary loop, which is
/// exactly what the planar-boundary acceptance fixture exercises.
pub fn planar(outer: &Profile, deflection_mm: f64, limits: &Limits) -> Result<Brep, CreationError> {
    let deflection = checked_deflection(deflection_mm)?;
    let points = flatten(&outer.points);
    let plane_table = [
        outer.centroid[0],
        outer.centroid[1],
        outer.centroid[2],
        outer.normal[0],
        outer.normal[1],
        outer.normal[2],
    ];
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_planar(
                points.as_ptr(),
                outer.points.len(),
                plane_table.as_ptr(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// The general profile-to-B-Rep adapter: one closed profile becomes one owned exact face.
pub fn profile_to_brep(
    profile: &Profile,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    planar(profile, deflection_mm, limits)
}

/// Loft through two or more cross-sections.
///
/// Every section must be wound consistently with the progression direction; a reversed section is
/// refused instead of silently corrected, because a correction would move the user's geometry.
pub fn loft(
    sections: &[Profile],
    solid: bool,
    ruled: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if sections.len() < 2 {
        return Err(CreationError::Invalid(
            "loft needs at least two sections".into(),
        ));
    }
    if sections.len() > MAX_PROFILE_POINTS {
        return Err(CreationError::Budget("loft section count".into()));
    }
    let mut points: Vec<Point3> = Vec::new();
    let mut offsets = Vec::with_capacity(sections.len());
    for section in sections {
        offsets.push((points.len(), section.points.len()));
        points.extend_from_slice(&section.points);
    }
    for pair in sections.windows(2) {
        let chord = sub(pair[1].centroid, pair[0].centroid);
        let span = length(chord);
        if !span.is_finite() || span <= GEOMETRIC_TOLERANCE_MM {
            return Err(CreationError::Invalid(
                "loft sections are coincident".into(),
            ));
        }
        if dot(pair[0].normal, chord) <= 0.0 {
            return Err(CreationError::Invalid(
                "loft sections are not consistently wound".into(),
            ));
        }
    }
    let deflection = checked_deflection(deflection_mm)?;
    let flat = flatten(&points);
    let section_table = table(&offsets);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_loft(
                flat.as_ptr(),
                points.len(),
                section_table.as_ptr(),
                offsets.len(),
                i32::from(solid),
                i32::from(ruled),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Sweep one or more profile wires along a spine, optionally with an
/// auxiliary spine (two-rail sweep). OCCT determines each section's spine
/// location from the section position, matching Matrix/Rhino's multi-profile
/// `SweepOneRail`/`SweepTwoRail` behavior.
pub fn pipe_shell(
    spine: &[Point3],
    aux_spine: Option<&[Point3]>,
    sections: &[Vec<Point3>],
    solid: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if spine.len() < 2 {
        return Err(CreationError::Invalid("pipe shell spine".into()));
    }
    if sections.is_empty() {
        return Err(CreationError::Invalid("pipe shell sections".into()));
    }
    for point in spine.iter().chain(aux_spine.into_iter().flatten()) {
        if !point.iter().all(|value| value.is_finite()) {
            return Err(CreationError::Invalid("pipe shell rail point".into()));
        }
    }
    let mut points = Vec::new();
    let mut offsets = Vec::with_capacity(sections.len());
    for section in sections {
        if section.len() < 2 {
            return Err(CreationError::Invalid("pipe shell profile".into()));
        }
        if section
            .iter()
            .any(|point| !point.iter().all(|value| value.is_finite()))
        {
            return Err(CreationError::Invalid("pipe shell profile point".into()));
        }
        offsets.push((points.len(), section.len()));
        points.extend_from_slice(section);
    }
    let deflection = checked_deflection(deflection_mm)?;
    let spine_flat = flatten(spine);
    let aux_flat = aux_spine.map(flatten).unwrap_or_default();
    let flat = flatten(&points);
    let section_table = table(&offsets);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_pipe_shell(
                spine_flat.as_ptr(),
                spine.len(),
                aux_flat.as_ptr(),
                aux_spine.map_or(0, <[Point3]>::len),
                flat.as_ptr(),
                points.len(),
                section_table.as_ptr(),
                offsets.len(),
                i32::from(solid),
                i32::from(aux_spine.is_some()),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Four rail sweep: one two-rail pipe-shell patch per quadrant, sewn into one
/// open polysurface. This mirrors Matrix `gvFourProfileSweep`'s four
/// `SweepTwoRail` patches followed by `Brep.JoinBreps`.
pub fn four_rail_sweep(
    rails: &[Vec<Point3>; 4],
    sections: &[Vec<Vec<Point3>>; 4],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
        return Err(CreationError::Invalid("four rail tolerance".into()));
    }
    let mut rail_points = Vec::new();
    let mut rail_offsets = Vec::with_capacity(4);
    for rail in rails {
        if rail.len() < 2 {
            return Err(CreationError::Invalid("four rail spine".into()));
        }
        rail_offsets.push((rail_points.len(), rail.len()));
        rail_points.extend_from_slice(rail);
    }
    let mut section_points = Vec::new();
    let mut section_offsets = Vec::with_capacity(4);
    let mut ranges = Vec::with_capacity(4);
    for quadrant in sections {
        let first = section_offsets.len();
        for section in quadrant {
            if section.len() < 2 {
                return Err(CreationError::Invalid("four rail section".into()));
            }
            section_offsets.push((section_points.len(), section.len()));
            section_points.extend_from_slice(section);
        }
        ranges.push((first, section_offsets.len() - first));
    }
    let rail_table = table(&rail_offsets);
    let section_table = table(&section_offsets);
    let mut range_table = Vec::with_capacity(8);
    for (first, count) in ranges {
        range_table.push(first);
        range_table.push(count);
    }
    let deflection = checked_deflection(deflection_mm)?;
    let rail_flat = flatten(&rail_points);
    let section_flat = flatten(&section_points);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_four_rail_sweep(
                rail_flat.as_ptr(),
                rail_points.len(),
                rail_table.as_ptr(),
                section_flat.as_ptr(),
                section_points.len(),
                section_table.as_ptr(),
                range_table.as_ptr(),
                0,
                tolerance_mm,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// One path station: the point, the transported in-plane axes and the unit tangent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathFrame {
    pub origin: Point3,
    pub x: Point3,
    pub y: Point3,
    pub tangent: Point3,
}

fn remove_along(value: Point3, direction: Point3) -> Point3 {
    let projection = dot(value, direction);
    [
        value[0] - direction[0] * projection,
        value[1] - direction[1] * projection,
        value[2] - direction[2] * projection,
    ]
}

fn validate_rail(points: &[Point3], what: &str) -> Result<Vec<Point3>, CreationError> {
    if points.len() < 2 {
        return Err(CreationError::Invalid(format!(
            "{what} needs at least two points"
        )));
    }
    if points.len() > MAX_PROFILE_POINTS {
        return Err(CreationError::Budget(format!("{what} point count")));
    }
    for point in points {
        finite(*point, what)?;
    }
    for window in points.windows(2) {
        if distance(window[0], window[1]) <= GEOMETRIC_TOLERANCE_MM {
            return Err(CreationError::Invalid(format!(
                "{what} has a zero-length segment"
            )));
        }
    }
    Ok(points.to_vec())
}

/// Rotation-minimizing frames along a polyline (projection transport, same rule as the surface
/// kernel): every frame is orthonormal and perpendicular to the local tangent.
pub(crate) fn polyline_frames(points: &[Point3]) -> Result<Vec<PathFrame>, CreationError> {
    let count = points.len();
    let mut tangents = Vec::with_capacity(count);
    for index in 0..count {
        let tangent = if index == 0 {
            sub(points[1], points[0])
        } else if index == count - 1 {
            sub(points[count - 1], points[count - 2])
        } else {
            sub(points[index + 1], points[index - 1])
        };
        tangents.push(unit(tangent, "rail tangent")?);
    }
    let mut x = [0.0; 3];
    for candidate in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let projected = remove_along(candidate, tangents[0]);
        if length(projected) > 1.0e-9 {
            x = unit(projected, "rail frame")?;
            break;
        }
    }
    let mut frames = Vec::with_capacity(count);
    for index in 0..count {
        if index > 0 {
            x = unit(remove_along(x, tangents[index]), "rail frame flip")?;
        }
        let y = unit(cross(tangents[index], x), "rail frame")?;
        frames.push(PathFrame {
            origin: points[index],
            x,
            y,
            tangent: tangents[index],
        });
    }
    Ok(frames)
}

/// Pipe: an exact circular section carried around a polyline path. Capped (default) is a closed
/// solid; uncapped is the open tube shell. The section seam is deterministic because the frames
/// are, and the section is a true `Geom_Circle`, so a straight pipe measures exactly `πr²h`.
pub fn pipe(
    path: &[Point3],
    radius_mm: f64,
    capped: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    positive(radius_mm, "pipe radius")?;
    let path = validate_rail(path, "pipe path")?;
    let frames = polyline_frames(&path)?;
    let mut stations = Vec::with_capacity(frames.len() * 9);
    for frame in &frames {
        stations.extend_from_slice(&frame.origin);
        stations.extend_from_slice(&frame.tangent);
        stations.extend_from_slice(&frame.x);
    }
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_tube(
                stations.as_ptr(),
                frames.len(),
                radius_mm,
                i32::from(capped),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Loft exact circular sections with radii linearly interpolated by polyline
/// length. For a straight rail this is an exact conical frustum. A curved
/// rail is represented by the caller's polyline and the loft is approximate.
pub fn pipe_linear(
    path: &[Point3], start_radius_mm: f64, end_radius_mm: f64, capped: bool,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, CreationError> {
    positive(start_radius_mm, "pipe start radius")?;
    positive(end_radius_mm, "pipe end radius")?;
    let path = validate_rail(path, "pipe path")?;
    let frames = polyline_frames(&path)?;
    let total: f64 = path.windows(2).map(|p| distance(p[0], p[1])).sum();
    if !total.is_finite() || total <= GEOMETRIC_TOLERANCE_MM {
        return Err(CreationError::Invalid("pipe path length".into()));
    }
    let mut stations = Vec::with_capacity(frames.len() * 9);
    let mut radii = Vec::with_capacity(frames.len());
    let mut travelled = 0.0;
    for (index, frame) in frames.iter().enumerate() {
        if index > 0 { travelled += distance(path[index - 1], path[index]); }
        stations.extend_from_slice(&frame.origin);
        stations.extend_from_slice(&frame.tangent);
        stations.extend_from_slice(&frame.x);
        radii.push(start_radius_mm + (end_radius_mm - start_radius_mm) * (travelled / total));
    }
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the two buffers contain one frame and radius per station,
        // and remain alive for the synchronous native call.
        unsafe { om9_occt_tube_radii(stations.as_ptr(), radii.as_ptr(), frames.len(),
            i32::from(capped), 0, deflection, budget.points, budget.nodes, budget.bytes,
            sink, context, message, capacity) }
    }).map_err(CreationError::from)
}

/// Loft a radius at each point of an already sampled path, using ruled sections.
pub fn pipe_sampled_sections(
    path: &[Point3], radii_mm: &[f64], capped: bool,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, CreationError> {
    if path.len()!=radii_mm.len() || path.len() as u64>limits.max_points {
        return Err(CreationError::Invalid("pipe section count".into()));
    }
    for &radius in radii_mm {positive(radius,"pipe section radius")?;}
    let path=validate_rail(path,"pipe section path")?;
    let frames=polyline_frames(&path)?;
    let capacity=frames.len().checked_mul(9).ok_or_else(||CreationError::Invalid("pipe section count overflow".into()))?;
    let mut flat=Vec::new(); flat.try_reserve_exact(capacity).map_err(|_|CreationError::Invalid("pipe section allocation".into()))?;
    for frame in &frames {flat.extend_from_slice(&frame.origin); flat.extend_from_slice(&frame.tangent); flat.extend_from_slice(&frame.x);}
    let deflection=checked_deflection(deflection_mm)?; let budget=snapshot_budget(limits);
    snapshot_via(deflection,limits,|sink,context,message,capacity| {
        // SAFETY: frame and radius buffers remain alive throughout the synchronous bridge call.
        unsafe {om9_occt_tube_radii(flat.as_ptr(),radii_mm.as_ptr(),frames.len(),i32::from(capped),1,
            deflection,budget.points,budget.nodes,budget.bytes,sink,context,message,capacity)}
    }).map_err(CreationError::from)
}

/// Straight rail with explicit ordered `(fraction, radius)` stations and
/// piecewise linear interpolation. Fractions include both endpoints.
pub fn pipe_stations_straight(
    endpoints: [Point3; 2], stations: &[(f64, f64)], capped: bool,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, CreationError> {
    validate_rail(&endpoints, "pipe path")?;
    if stations.len() < 2 || stations.len() > MAX_PROFILE_POINTS || stations[0].0 != 0.0
        || stations[stations.len()-1].0 != 1.0 {
        return Err(CreationError::Invalid("pipe stations require ordered endpoints".into()));
    }
    for (index, &(fraction, radius)) in stations.iter().enumerate() {
        positive(radius, "pipe station radius")?;
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction)
            || (index > 0 && fraction <= stations[index-1].0) {
            return Err(CreationError::Invalid("pipe stations must be strictly ordered".into()));
        }
    }
    let points: Vec<Point3> = stations.iter().map(|(t, _)| std::array::from_fn(|axis|
        endpoints[0][axis] + t * (endpoints[1][axis] - endpoints[0][axis]))).collect();
    validate_rail(&points, "pipe station path")?;
    let frames = polyline_frames(&points)?;
    let mut flat = Vec::with_capacity(frames.len() * 9);
    let mut radii = Vec::with_capacity(frames.len());
    for (frame, (_, radius)) in frames.iter().zip(stations) {
        flat.extend_from_slice(&frame.origin);
        flat.extend_from_slice(&frame.tangent);
        flat.extend_from_slice(&frame.x);
        radii.push(*radius);
    }
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the frame and radius arrays stay alive for the synchronous call.
        unsafe { om9_occt_tube_radii(flat.as_ptr(), radii.as_ptr(), frames.len(),
            i32::from(capped), 1, deflection, budget.points, budget.nodes, budget.bytes,
            sink, context, message, capacity) }
    }).map_err(CreationError::from)
}

/// Sweep a circular profile along an open polyline with mitered corners.
pub fn pipe_miter_polyline(path: &[Point3], radius_mm: f64, capped: bool,
    deflection_mm: f64, limits: &Limits) -> Result<Brep, CreationError> {
    positive(radius_mm, "pipe radius")?;
    let path = validate_rail(path, "pipe path")?;
    if distance(path[0], path[path.len() - 1]) <= GEOMETRIC_TOLERANCE_MM {
        return Err(CreationError::Invalid("closed pipe path is not supported".into()));
    }
    for points in path.windows(3) {
        let a = unit(sub(points[1], points[0]), "pipe segment")?;
        let b = unit(sub(points[2], points[1]), "pipe segment")?;
        if dot(a, b) <= -1.0 + 1.0e-9 { return Err(CreationError::Invalid("pipe path reverses direction".into())); }
    }
    let flat = flatten(&path);
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the validated path and callback context live through the synchronous call.
        unsafe { om9_occt_pipe_miter(flat.as_ptr(), path.len(), radius_mm, i32::from(capped),
            deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(CreationError::from)
}

/// An oriented exact hemisphere with its flat face through `center`.
pub fn hemisphere(center: Point3, direction: Point3, radius_mm: f64, forward: bool,
    deflection_mm: f64, limits: &Limits) -> Result<Brep, CreationError> {
    positive(radius_mm, "hemisphere radius")?;
    finite(center,"hemisphere center")?;
    let direction=unit(direction,"hemisphere direction")?;
    let frame=[center[0],center[1],center[2],direction[0],direction[1],direction[2]];
    let deflection=checked_deflection(deflection_mm)?; let budget=snapshot_budget(limits);
    snapshot_via(deflection,limits,|sink,context,message,capacity| {
        // SAFETY: frame and callback context outlive the synchronous native call.
        unsafe {om9_occt_hemisphere(frame.as_ptr(),radius_mm,i32::from(forward),deflection,
            budget.points,budget.nodes,budget.bytes,sink,context,message,capacity)}
    }).map_err(CreationError::from)
}

/// Exact straight conical pipe with a hemispherical cap at each end.
pub fn pipe_round_straight(
    endpoints: [Point3; 2], start_radius_mm: f64, end_radius_mm: f64,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, CreationError> {
    positive(start_radius_mm, "pipe start radius")?;
    positive(end_radius_mm, "pipe end radius")?;
    validate_rail(&endpoints, "pipe path")?;
    let flat = flatten(&endpoints);
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the endpoint buffer and callback context outlive the synchronous call.
        unsafe { om9_occt_round_pipe(flat.as_ptr(), start_radius_mm, end_radius_mm,
            deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(CreationError::from)
}

fn profile_local(profile: &Profile, frame: &PathFrame) -> Vec<[f64; 3]> {
    profile
        .points
        .iter()
        .map(|point| {
            let offset = sub(*point, frame.origin);
            [
                dot(offset, frame.x),
                dot(offset, frame.y),
                dot(offset, frame.tangent),
            ]
        })
        .collect()
}

fn section_from_local(
    frame: &PathFrame,
    local: &[[f64; 3]],
    scale_x: f64,
    scale_y: f64,
) -> Vec<Point3> {
    local
        .iter()
        .map(|value| {
            let x = value[0] * scale_x;
            let y = value[1] * scale_y;
            let z = value[2];
            [
                frame.origin[0] + frame.x[0] * x + frame.y[0] * y + frame.tangent[0] * z,
                frame.origin[1] + frame.x[1] * x + frame.y[1] * y + frame.tangent[1] * z,
                frame.origin[2] + frame.x[2] * x + frame.y[2] * y + frame.tangent[2] * z,
            ]
        })
        .collect()
}

fn check_profile_perpendicular(local: &[[f64; 3]]) -> Result<(), CreationError> {
    let first = local[0][2];
    if local
        .iter()
        .any(|value| (value[2] - first).abs() > GEOMETRIC_TOLERANCE_MM)
    {
        return Err(CreationError::Invalid(
            "sweep profile plane is not perpendicular to the rail".into(),
        ));
    }
    Ok(())
}

fn loft_sections(
    sections: &[Vec<Point3>],
    solid: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let mut points: Vec<Point3> = Vec::new();
    let mut offsets = Vec::with_capacity(sections.len());
    for section in sections {
        offsets.push((points.len(), section.len()));
        points.extend_from_slice(section);
    }
    let deflection = checked_deflection(deflection_mm)?;
    let flat = flatten(&points);
    let section_table = table(&offsets);
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers and the callback context live through this synchronous call.
        unsafe {
            om9_occt_loft(
                flat.as_ptr(),
                points.len(),
                section_table.as_ptr(),
                offsets.len(),
                i32::from(solid),
                0,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

/// Sweep one profile along one rail with rotation-minimizing frames (Sweep1).
pub fn sweep1(
    profile: &Profile,
    rail: &[Point3],
    capped: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let rail = validate_rail(rail, "sweep rail")?;
    let frames = polyline_frames(&rail)?;
    let local = profile_local(profile, &frames[0]);
    check_profile_perpendicular(&local)?;
    let sections: Vec<Vec<Point3>> = frames
        .iter()
        .map(|frame| section_from_local(frame, &local, 1.0, 1.0))
        .collect();
    loft_sections(&sections, capped, deflection_mm, limits)
}

/// Sweep two rails (Sweep2): the profile spans the rail separation, `x` follows that separation,
/// the height is preserved when `maintain_height` is set and scaled with the width otherwise.
pub fn sweep2(
    profile: &Profile,
    rail_a: &[Point3],
    rail_b: &[Point3],
    capped: bool,
    maintain_height: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let rail_a = validate_rail(rail_a, "sweep rail a")?;
    let rail_b = validate_rail(rail_b, "sweep rail b")?;
    if rail_a.len() != rail_b.len() {
        return Err(CreationError::Invalid(
            "sweep rails must have the same station count".into(),
        ));
    }
    let frames_a = polyline_frames(&rail_a)?;
    let frames_b = polyline_frames(&rail_b)?;
    let width0 = distance(rail_a[0], rail_b[0]);
    if !width0.is_finite() || width0 <= GEOMETRIC_TOLERANCE_MM {
        return Err(CreationError::Invalid(
            "sweep2 rails coincide at the first station".into(),
        ));
    }
    let x0 = unit(sub(rail_b[0], rail_a[0]), "sweep2 rail separation")?;
    let normal0 = unit(cross(x0, frames_a[0].tangent), "sweep2 frame")?;
    let tangent0 = unit(cross(normal0, x0), "sweep2 frame")?;
    let mut local = Vec::with_capacity(profile.points.len());
    let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
    for point in &profile.points {
        let offset = sub(*point, rail_a[0]);
        let coordinates = [dot(offset, x0), dot(offset, normal0), dot(offset, tangent0)];
        min_x = min_x.min(coordinates[0]);
        max_x = max_x.max(coordinates[0]);
        local.push(coordinates);
    }
    if min_x > GEOMETRIC_TOLERANCE_MM || max_x < width0 - GEOMETRIC_TOLERANCE_MM {
        return Err(CreationError::Invalid(
            "sweep2 profile does not span both rail starts".into(),
        ));
    }
    if local
        .iter()
        .any(|value| value[2].abs() > GEOMETRIC_TOLERANCE_MM)
    {
        return Err(CreationError::Invalid(
            "sweep2 profile plane is not perpendicular to the primary rail".into(),
        ));
    }
    let mut sections = Vec::with_capacity(frames_a.len());
    for index in 0..frames_a.len() {
        let origin_a = frames_a[index].origin;
        let origin_b = frames_b[index].origin;
        let width = distance(origin_a, origin_b);
        if !width.is_finite() || width <= GEOMETRIC_TOLERANCE_MM {
            return Err(CreationError::Invalid(
                "sweep2 rails coincide at a station".into(),
            ));
        }
        let scale = width / width0;
        let x = unit(sub(origin_b, origin_a), "sweep2 rail separation")?;
        let normal = unit(cross(x, frames_a[index].tangent), "sweep2 frame")?;
        let tangent = unit(cross(normal, x), "sweep2 frame")?;
        let frame = PathFrame {
            origin: origin_a,
            x,
            y: normal,
            tangent,
        };
        let height = if maintain_height { 1.0 } else { scale };
        sections.push(section_from_local(&frame, &local, scale, height));
    }
    loft_sections(&sections, capped, deflection_mm, limits)
}

/// A filled surface patch and the measured maximum deviation from its boundary curves.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePatch {
    pub brep: Brep,
    pub max_deviation_mm: f64,
}

/// Surface from two to four boundary curves (a Coons-style filling patch).
///
/// The curves must form a closed boundary within the geometric tolerance; a gap is refused rather
/// than silently bridged.
pub fn surface_from_edges(
    curves: &[Vec<Point3>],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<SurfacePatch, CreationError> {
    if !(2..=4).contains(&curves.len()) {
        return Err(CreationError::Invalid(
            "surface filling needs two to four boundary curves".into(),
        ));
    }
    positive(tolerance_mm, "surface tolerance")?;
    let tolerance = tolerance_mm.max(GEOMETRIC_TOLERANCE_MM);
    let mut points: Vec<Point3> = Vec::new();
    let mut offsets = Vec::with_capacity(curves.len());
    for curve in curves {
        let curve = validate_rail(curve, "surface boundary curve")?;
        offsets.push((points.len(), curve.len()));
        points.extend(curve);
    }
    // Boundary continuity: each curve's end must meet the next curve's start, and the last meets
    // the first.
    for index in 0..curves.len() {
        let end = *curves[index].last().expect("validated curve");
        let next = *curves[(index + 1) % curves.len()]
            .first()
            .expect("validated curve");
        let gap = distance(end, next);
        if !gap.is_finite() || gap > tolerance {
            return Err(CreationError::Invalid(format!(
                "surface boundary gap {gap} mm exceeds the tolerance {tolerance} mm"
            )));
        }
    }
    let deflection = checked_deflection(deflection_mm)?;
    let flat = flatten(&points);
    let curve_table = table(&offsets);
    let mut deviation = 0.0_f64;
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all buffers, the deviation output and the callback context live through this
        // synchronous call.
        unsafe {
            om9_occt_surface(
                flat.as_ptr(),
                points.len(),
                curve_table.as_ptr(),
                offsets.len(),
                tolerance,
                &mut deviation,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })?;
    if !deviation.is_finite() || deviation > tolerance {
        return Err(CreationError::Invalid(format!(
            "surface deviation {deviation} mm exceeds the tolerance {tolerance} mm"
        )));
    }
    Ok(SurfacePatch {
        brep,
        max_deviation_mm: deviation,
    })
}

/* ------------------------- measurement ------------------------- */

/// Measured facts about one exact B-Rep, computed by OCCT from the canonical source (never from the
/// display mesh or a bounding box).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measures {
    pub volume_mm3: f64,
    pub area_mm2: f64,
    pub faces: usize,
    pub edges: usize,
    pub vertices: usize,
    pub solids: usize,
    pub shells: usize,
    /// Free boundary edges (no second face occurrence): exact B-Rep topology, never a mesh boundary.
    pub naked_edges: usize,
    pub closed: bool,
    pub bounds_mm: [f64; 6],
}

impl Measures {
    pub fn size_mm(&self) -> [f64; 3] {
        [
            self.bounds_mm[3] - self.bounds_mm[0],
            self.bounds_mm[4] - self.bounds_mm[1],
            self.bounds_mm[5] - self.bounds_mm[2],
        ]
    }
}

/// Measure an exact B-Rep: volume, area, topology counts, closedness and optimal bounds.
pub fn measure(brep: &Brep, limits: &Limits) -> Result<Measures, CreationError> {
    brep.validate(limits)
        .map_err(|error| CreationError::Invalid(error.to_string()))?;
    let source = brep.source();
    let mut out = MeasuresOut {
        volume_mm3: 0.0,
        area_mm2: 0.0,
        faces: 0,
        edges: 0,
        vertices: 0,
        solids: 0,
        shells: 0,
        naked_edges: 0,
        closed: 0,
        bounds_mm: [0.0; 6],
    };
    let mut message = [0_i8; 512];
    // SAFETY: the source buffer and every scalar output live through this synchronous call.
    let status = unsafe {
        om9_occt_brep_measure(
            source.as_ptr(),
            source.len(),
            &mut out.volume_mm3,
            &mut out.area_mm2,
            &mut out.faces,
            &mut out.edges,
            &mut out.vertices,
            &mut out.solids,
            &mut out.shells,
            &mut out.naked_edges,
            &mut out.closed,
            out.bounds_mm.as_mut_ptr(),
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(CreationError::Kernel(native_message(&message)));
    }
    Ok(Measures {
        volume_mm3: out.volume_mm3,
        area_mm2: out.area_mm2,
        faces: usize::try_from(out.faces).unwrap_or(0),
        edges: usize::try_from(out.edges).unwrap_or(0),
        vertices: usize::try_from(out.vertices).unwrap_or(0),
        solids: usize::try_from(out.solids).unwrap_or(0),
        shells: usize::try_from(out.shells).unwrap_or(0),
        naked_edges: usize::try_from(out.naked_edges).unwrap_or(0),
        closed: out.closed != 0,
        bounds_mm: out.bounds_mm,
    })
}

/* ------------------------- handle session ------------------------- */

/// The kind of a named entity published by the general profile-to-B-Rep adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Face,
    Edge,
    Vertex,
}

/// One named entity of a published B-Rep: a stable, revision-bound publication name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityName {
    pub kind: EntityKind,
    pub index: usize,
    pub id: Uuid,
}

impl EntityName {
    pub fn name(&self) -> String {
        let kind = match self.kind {
            EntityKind::Face => "face",
            EntityKind::Edge => "edge",
            EntityKind::Vertex => "vertex",
        };
        format!("{kind}[{}]", self.index)
    }
}

/// Owns created B-Rep bodies behind generational handles, so a released handle is stale instead of
/// resolving to the next body that reuses its slot. This is the general profile-to-B-Rep adapter's
/// ownership half: the document stores [`KernelHandle`], never a native pointer.
#[derive(Debug)]
pub struct CreationSession {
    bodies: HandleStore<Brep>,
    limits: Limits,
    deflection_mm: f64,
}

impl CreationSession {
    pub fn new(limits: Limits, deflection_mm: f64) -> Result<Self, CreationError> {
        Ok(Self {
            bodies: HandleStore::new(),
            limits,
            deflection_mm: checked_deflection(deflection_mm)?,
        })
    }

    pub fn with_capacity(
        limits: Limits,
        deflection_mm: f64,
        expected: usize,
    ) -> Result<Self, CreationError> {
        Ok(Self {
            bodies: HandleStore::with_capacity(expected),
            limits,
            deflection_mm: checked_deflection(deflection_mm)?,
        })
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    pub fn deflection_mm(&self) -> f64 {
        self.deflection_mm
    }

    pub fn live(&self) -> usize {
        self.bodies.len()
    }

    pub fn insert(&mut self, brep: Brep) -> KernelHandle {
        self.bodies.insert(brep)
    }

    pub fn build_primitive(&mut self, value: Primitive) -> Result<KernelHandle, CreationError> {
        let brep = primitive(value, self.deflection_mm, &self.limits)?;
        Ok(self.bodies.insert(brep))
    }

    pub fn build_profile(&mut self, profile: &Profile) -> Result<KernelHandle, CreationError> {
        let brep = profile_to_brep(profile, self.deflection_mm, &self.limits)?;
        Ok(self.bodies.insert(brep))
    }

    pub fn body(&self, handle: KernelHandle) -> Result<&Brep, CreationError> {
        self.bodies.get(handle).ok_or(CreationError::StaleHandle)
    }

    pub fn release(&mut self, handle: KernelHandle) -> Result<Brep, CreationError> {
        self.bodies
            .remove(handle)
            .map_err(|_| CreationError::StaleHandle)
    }

    /// Publication names for every face, edge and vertex, in OCCT map order.
    pub fn names(&self, handle: KernelHandle) -> Result<Vec<EntityName>, CreationError> {
        let brep = self.body(handle)?;
        let mut names = Vec::with_capacity(
            brep.display.faces.len() + brep.display.edges.len() + brep.display.vertex_ids.len(),
        );
        for (index, id) in brep.display.faces.iter().enumerate() {
            names.push(EntityName {
                kind: EntityKind::Face,
                index,
                id: *id,
            });
        }
        for (index, id) in brep.display.edges.iter().enumerate() {
            names.push(EntityName {
                kind: EntityKind::Edge,
                index,
                id: *id,
            });
        }
        for (index, id) in brep.display.vertex_ids.iter().enumerate() {
            names.push(EntityName {
                kind: EntityKind::Vertex,
                index,
                id: *id,
            });
        }
        Ok(names)
    }
}
