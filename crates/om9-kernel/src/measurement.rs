// SPDX-License-Identifier: LGPL-2.1-or-later
//! Read-only canonical measurements. Rust validates/budgets immutable operands;
//! OCCT owns only the synchronous geometric solver, never the document.
use crate::KernelError;
use om9_core::{
    brep::Brep,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Transform},
};
use std::ffi::{CStr, c_char, c_int};

#[repr(C)]
struct Part {
    kind: c_int,
    degree: c_int,
    periodic: c_int,
    source: *const u8,
    source_len: usize,
    points: *const Point3,
    point_count: usize,
    weights: *const f64,
    knots: *const f64,
    knot_count: usize,
    domain: [f64; 2],
    radius: f64,
    matrix: [[f64; 4]; 4],
    size: [c_int;2],
    degree_v:c_int,
    knots_v:*const f64,
    knot_count_v:usize,
    domain_v:[f64;2],
}
unsafe extern "C" {
    fn om9_occt_measure_distance(
        a: *const Part,
        ac: usize,
        b: *const Part,
        bc: usize,
        value: *mut f64,
        witnesses: *mut Point3,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_measure_edge_length(
        source: *const u8,
        length: usize,
        index: c_int,
        matrix: *const f64,
        value: *mut f64,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}
fn error(status: c_int, message: &[c_char]) -> KernelError {
    // SAFETY: the adapter NUL-terminates the fixed buffer on every error.
    let text = unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    if status == 1 {
        KernelError::InvalidInput(text)
    } else {
        KernelError::Failure(text)
    }
}
fn admit(result: om9_core::Result<impl Sized>) -> Result<(), KernelError> {
    result
        .map(|_| ())
        .map_err(|e| KernelError::InvalidInput(e.to_string()))
}
impl Part {
    fn new(transform: Transform) -> Self {
        Self {
            kind: 0,
            degree: 0,
            periodic: 0,
            source: std::ptr::null(),
            source_len: 0,
            points: std::ptr::null(),
            point_count: 0,
            weights: std::ptr::null(),
            knots: std::ptr::null(),
            knot_count: 0,
            domain: [0., 1.],
            radius: 0.,
            matrix: transform.0,
            size:[0;2],degree_v:0,knots_v:std::ptr::null(),knot_count_v:0,domain_v:[0.,1.],
        }
    }
}
fn curve_parts(
    curve: &Curve,
    transform: Transform,
    parts: &mut Vec<Part>,
    owned: &mut Vec<Vec<Point3>>,
) {
    if let Curve::Polycurve { segments, .. } = curve {
        for c in segments {
            curve_parts(c, transform, parts, owned)
        }
        return;
    }
    let mut p = Part::new(transform);
    match curve {
        Curve::Line { start, end, .. } => {
            p.kind = 2;
            owned.push(vec![*start, *end]);
            p.points = owned.last().unwrap().as_ptr();
            p.point_count = 2;
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            p.kind = 3;
            owned.push(vec![*center, *x, *y]);
            p.points = owned.last().unwrap().as_ptr();
            p.point_count = 3;
            p.radius = *radius;
            p.domain = angle_domain.unwrap_or(*domain);
        }
        Curve::Nurbs(n) => {
            p.kind = 4;
            p.points = n.poles.as_ptr();
            p.point_count = n.poles.len();
            p.weights = n.weights.as_ptr();
            p.knots = n.knots.as_ptr();
            p.knot_count = n.knots.len();
            p.degree = n.degree as c_int;
            p.periodic = i32::from(n.periodic);
            p.domain = n.domain;
        }
        Curve::Polycurve { .. } => unreachable!(),
    }
    parts.push(p);
}
fn parts(
    geometry: &Geometry,
    transform: Transform,
    limits: &Limits,
    owned: &mut Vec<Vec<Point3>>,
) -> Result<Vec<Part>, KernelError> {
    admit(geometry.validate(limits))?;
    admit(transform.validate())?;
    let mut out = Vec::new();
    let mut p = Part::new(transform);
    match geometry {
        Geometry::Brep(b) => {
            p.source = b.source().as_ptr();
            p.source_len = b.source().len();
            out.push(p);
        }
        Geometry::Point(point) => {
            p.kind = 1;
            p.points = point;
            p.point_count = 1;
            out.push(p);
        }
        Geometry::Curve(c) => curve_parts(c, transform, &mut out, owned),
        Geometry::Surface(s)=>{
            p.kind=6;p.points=s.poles.as_ptr();p.point_count=s.poles.len();p.weights=s.weights.as_ptr();
            p.degree=s.degree[0] as c_int;p.degree_v=s.degree[1] as c_int;
            p.periodic=i32::from(s.periodic[0])|(i32::from(s.periodic[1])<<1);
            p.size=[c_int::try_from(s.size[0]).map_err(|_|KernelError::InvalidInput("surface U size".into()))?,c_int::try_from(s.size[1]).map_err(|_|KernelError::InvalidInput("surface V size".into()))?];
            p.knots=s.knots[0].as_ptr();p.knot_count=s.knots[0].len();p.knots_v=s.knots[1].as_ptr();p.knot_count_v=s.knots[1].len();p.domain=s.domain[0];p.domain_v=s.domain[1];out.push(p);
        }
        Geometry::Mesh(mesh) => {
            for face in &mesh.faces {
                if face.len()!=3{return Err(KernelError::InvalidInput("distance on polygon mesh needs an explicit triangulation".into()));}
                for i in 1..face.len() - 1 {
                    let mut p = Part::new(transform);
                    p.kind = 5;
                    owned.push(vec![
                        mesh.vertices[face[0] as usize],
                        mesh.vertices[face[i] as usize],
                        mesh.vertices[face[i + 1] as usize],
                    ]);
                    p.points = owned.last().unwrap().as_ptr();
                    p.point_count = 3;
                    out.push(p);
                }
            }
        }
        _ => {
            return Err(KernelError::InvalidInput(
                "object distance expects point, curve, mesh or canonical BRep".into(),
            ));
        }
    }
    if out.is_empty() || out.len() as u64 > limits.max_nodes {
        return Err(KernelError::InvalidInput(
            "object distance operand budget".into(),
        ));
    }
    Ok(out)
}
#[derive(Debug, Clone, Copy)]
pub struct Distance {
    pub millimetres: f64,
    pub points: [Point3; 2],
}
pub fn distance(
    a: &Geometry,
    at: Transform,
    b: &Geometry,
    bt: Transform,
    limits: &Limits,
) -> Result<Distance, KernelError> {
    let mut owned = Vec::new();
    let first = parts(a, at, limits, &mut owned)?;
    let second = parts(b, bt, limits, &mut owned)?;
    let mut value = 0.;
    let mut points = [[0.; 3]; 2];
    let mut message = [0; 512];
    // SAFETY: borrowed geometry and all owned point buffers remain live; no pointer is retained.
    let status = unsafe {
        om9_occt_measure_distance(
            first.as_ptr(),
            first.len(),
            second.as_ptr(),
            second.len(),
            &mut value,
            points.as_mut_ptr(),
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(error(status, &message));
    }
    Ok(Distance {
        millimetres: value,
        points,
    })
}
pub fn edge_length(
    brep: &Brep,
    index: Option<usize>,
    transform: Transform,
    limits: &Limits,
) -> Result<f64, KernelError> {
    admit(brep.validate(limits))?;
    admit(transform.validate())?;
    let index = match index {
        Some(i) if i < brep.display.edges.len() => {
            c_int::try_from(i).map_err(|_| KernelError::InvalidInput("edge index".into()))?
        }
        Some(_) => return Err(KernelError::InvalidInput("edge index".into())),
        None => -1,
    };
    let mut value = 0.;
    let mut message = [0; 512];
    // SAFETY: all inputs/outputs live through this synchronous scalar operation.
    let status = unsafe {
        om9_occt_measure_edge_length(
            brep.source().as_ptr(),
            brep.source().len(),
            index,
            transform.0.as_ptr().cast(),
            &mut value,
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(error(status, &message));
    }
    Ok(value)
}
