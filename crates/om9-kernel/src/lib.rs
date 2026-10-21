//! OCCT 8.0.1 bridge for OpenMatrix9 (M01-L2-01-05).
//!
//! Promoted from the M00-L2-02 gate proof (`native/occt-rust-proof`), which measured the version, the
//! analytic volumes, the lifetime behaviour, the tessellation and the failure contract before the
//! product crate existed. The rules that proof established still hold here:
//!
//! * every function is a **complete operation**: no `TopoDS_Shape` and no OCCT `Handle` crosses into
//!   safe Rust;
//! * a kernel exception is caught in C++ and returned as [`KernelError`];
//! * the version is part of the contract: the tests assert the linked OCCT is the pinned 8.0.x.
//!
//! The pin lives in `native/occt/pin.json`; `implementation/tools/fetch_occt.py` materialises the SDK and
//! `implementation/tools/occt_upgrade_gate.py` decides whether the local `TKG3d` accuracy patch is still
//! needed after an upgrade.

use std::ffi::{CStr, c_char, c_double, c_int};
use std::fmt;
pub mod brep;

/// A kernel call that did not produce a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// The caller passed a value the kernel cannot accept (non-finite, non-positive, …).
    InvalidInput(String),
    /// The kernel raised a failure; the message is OCCT's `what()`.
    Failure(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid kernel input: {message}"),
            Self::Failure(message) => write!(formatter, "kernel failure: {message}"),
        }
    }
}

impl std::error::Error for KernelError {}

const OK: c_int = 0;
const INVALID_INPUT: c_int = 1;

unsafe extern "C" {
    fn om9_occt_version() -> *const c_char;
    fn om9_occt_box_volume(
        width: c_double,
        height: c_double,
        depth: c_double,
        volume: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_drilled_volume(
        width: c_double,
        height: c_double,
        depth: c_double,
        radius: c_double,
        volume: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_invalid_cylinder(radius: c_double, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_working_set(bytes: *mut c_double) -> c_int;
    fn om9_occt_handle_refcount(
        before: *mut c_int,
        copied: *mut c_int,
        after: *mut c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sphere_triangles(
        radius: c_double,
        tolerance: c_double,
        vertex_count: *mut i64,
        index_count: *mut i64,
        positions: *mut f32,
        indices: *mut u32,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_topology(
        kind: c_int,
        vertices: *mut i64,
        edges: *mut i64,
        faces: *mut i64,
        wires: *mut i64,
        valid: *mut c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_transform_probe(
        dx: c_double,
        dy: c_double,
        dz: c_double,
        rotate_deg: c_double,
        scale: c_double,
        min_x: *mut c_double,
        max_x: *mut c_double,
        min_z: *mut c_double,
        max_z: *mut c_double,
        volume: *mut c_double,
        valid: *mut c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_curve_eval(
        u: c_double,
        px: *mut c_double,
        py: *mut c_double,
        pz: *mut c_double,
        dx: *mut c_double,
        dy: *mut c_double,
        dz: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_surface_eval(
        u: c_double,
        v: c_double,
        px: *mut c_double,
        py: *mut c_double,
        pz: *mut c_double,
        nx: *mut c_double,
        ny: *mut c_double,
        nz: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_line_face_hits(
        hits: *mut i64,
        first_x: *mut c_double,
        first_y: *mut c_double,
        first_z: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_box_measures(
        width: c_double,
        height: c_double,
        depth: c_double,
        volume: *mut c_double,
        area: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sphere_measures(
        radius: c_double,
        volume: *mut c_double,
        area: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_cylinder_measures(
        radius: c_double,
        height: c_double,
        volume: *mut c_double,
        area: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_fillet_volume(
        width: c_double,
        height: c_double,
        depth: c_double,
        radius: c_double,
        volume: *mut c_double,
        valid: *mut c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_offset_volume(
        width: c_double,
        height: c_double,
        depth: c_double,
        distance: c_double,
        volume: *mut c_double,
        valid: *mut c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sphere_mesh(
        radius: c_double,
        tolerance: c_double,
        max_deviation: *mut c_double,
        triangles: *mut i64,
        boundary_edges: *mut i64,
        non_manifold_edges: *mut i64,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sphere_mesh_dump(
        radius: c_double,
        tolerance: c_double,
        path: *const c_char,
        vertices: *mut i64,
        triangles: *mut i64,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_degenerate(
        kind: c_int,
        valid: *mut c_int,
        volume: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_create_destroy(
        count: i64,
        elapsed_seconds: *mut c_double,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}

/// Version of the linked OCCT, e.g. `8.0.1`.
pub fn version() -> &'static str {
    // The bridge returns a static string literal, valid for the program lifetime.
    unsafe {
        let pointer = om9_occt_version();
        CStr::from_ptr(pointer).to_str().unwrap_or("unknown")
    }
}

fn status_message(code: c_int, message: &[c_char; 512]) -> KernelError {
    let text = unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    if code == INVALID_INPUT {
        KernelError::InvalidInput(text)
    } else {
        KernelError::Failure(text)
    }
}

/// Volume of a `width x height x depth` box at the origin.
pub fn box_volume(width: f64, height: f64, depth: f64) -> Result<f64, KernelError> {
    let mut volume = 0.0f64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_box_volume(
            width,
            height,
            depth,
            &mut volume,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(volume)
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume of the box after drilling a `radius` cylinder through it along Z (a boolean operation).
pub fn drilled_volume(
    width: f64,
    height: f64,
    depth: f64,
    radius: f64,
) -> Result<f64, KernelError> {
    let mut volume = 0.0f64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_drilled_volume(
            width,
            height,
            depth,
            radius,
            &mut volume,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(volume)
    } else {
        Err(status_message(code, &message))
    }
}

/// Calls the kernel with input it must reject, and reports the failure. Used by the gate's exception
/// contract: a kernel fault must arrive as an error, never as a crash.
pub fn reject_invalid_cylinder(radius: f64) -> Result<(), KernelError> {
    let mut message = [0 as c_char; 512];
    let code =
        unsafe { om9_occt_invalid_cylinder(radius, message.as_mut_ptr(), message.len() as c_int) };
    if code == OK {
        Ok(())
    } else {
        Err(status_message(code, &message))
    }
}

/// Working set of the process in bytes.
pub fn working_set_bytes() -> Result<f64, KernelError> {
    let mut bytes = 0.0f64;
    let code = unsafe { om9_occt_working_set(&mut bytes) };
    if code == OK {
        Ok(bytes)
    } else {
        Err(KernelError::Failure("working set unavailable".to_string()))
    }
}

/// Reference count of a kernel handle before a copy, while copied, and after `Nullify()`.
pub fn handle_refcount() -> Result<(i32, i32, i32), KernelError> {
    let (mut before, mut copied, mut after) = (0i32, 0i32, 0i32);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_handle_refcount(
            &mut before,
            &mut copied,
            &mut after,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((before, copied, after))
    } else {
        Err(status_message(code, &message))
    }
}

/// Creates and destroys `count` boxes; returns the elapsed seconds.
pub fn create_destroy(count: i64) -> Result<f64, KernelError> {
    let mut elapsed = 0.0f64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_create_destroy(
            count,
            &mut elapsed,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(elapsed)
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume and area of a box, in millimetres.
pub fn box_measures(width: f64, height: f64, depth: f64) -> Result<(f64, f64), KernelError> {
    let (mut volume, mut area) = (0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_box_measures(
            width,
            height,
            depth,
            &mut volume,
            &mut area,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((volume, area))
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume and area of a sphere, in millimetres.
pub fn sphere_measures(radius: f64) -> Result<(f64, f64), KernelError> {
    let (mut volume, mut area) = (0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_sphere_measures(
            radius,
            &mut volume,
            &mut area,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((volume, area))
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume and area of a cylinder, in millimetres.
pub fn cylinder_measures(radius: f64, height: f64) -> Result<(f64, f64), KernelError> {
    let (mut volume, mut area) = (0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_cylinder_measures(
            radius,
            height,
            &mut volume,
            &mut area,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((volume, area))
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume of a box after filleting one vertical edge.
pub fn fillet_volume(width: f64, height: f64, depth: f64, radius: f64) -> Result<f64, KernelError> {
    let mut volume = 0.0f64;
    let mut valid = 0i32;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_fillet_volume(
            width,
            height,
            depth,
            radius,
            &mut volume,
            &mut valid,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK && valid == 1 {
        Ok(volume)
    } else if code == OK {
        Err(KernelError::Failure(
            "fillet result is not a valid solid".to_string(),
        ))
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume of a box after an inward offset.
pub fn offset_volume(
    width: f64,
    height: f64,
    depth: f64,
    distance: f64,
) -> Result<f64, KernelError> {
    let mut volume = 0.0f64;
    let mut valid = 0i32;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_offset_volume(
            width,
            height,
            depth,
            distance,
            &mut volume,
            &mut valid,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK && valid == 1 {
        Ok(volume)
    } else if code == OK {
        Err(KernelError::Failure(
            "offset result is not a valid solid".to_string(),
        ))
    } else {
        Err(status_message(code, &message))
    }
}

/// A tessellated sphere: largest node deviation, triangle count, watertightness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshReport {
    pub max_deviation: f64,
    pub triangles: i64,
    /// Edges used by one triangle only: any of these is a hole.
    pub boundary_edges: i64,
    /// Edges used by more than two triangles (OCCT leaves one at each sphere pole).
    pub non_manifold_edges: i64,
}

impl MeshReport {
    /// Closed means "no boundary edge"; manifoldness is reported separately because a closed mesh can
    /// still carry a non-manifold edge.
    pub fn closed(&self) -> bool {
        self.boundary_edges == 0
    }
}

pub fn sphere_mesh(radius: f64, tolerance: f64) -> Result<MeshReport, KernelError> {
    let mut max_deviation = 0.0f64;
    let mut triangles = 0i64;
    let mut boundary_edges = 0i64;
    let mut non_manifold_edges = 0i64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_sphere_mesh(
            radius,
            tolerance,
            &mut max_deviation,
            &mut triangles,
            &mut boundary_edges,
            &mut non_manifold_edges,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(MeshReport {
            max_deviation,
            triangles,
            boundary_edges,
            non_manifold_edges,
        })
    } else {
        Err(status_message(code, &message))
    }
}

/// A degenerate case: whether the kernel produced a valid shape and its volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DegenerateReport {
    pub valid: bool,
    pub volume: f64,
}

pub fn degenerate(kind: i32) -> Result<DegenerateReport, KernelError> {
    let mut valid = 0i32;
    let mut volume = 0.0f64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_degenerate(
            kind,
            &mut valid,
            &mut volume,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(DegenerateReport {
            valid: valid == 1,
            volume,
        })
    } else {
        Err(status_message(code, &message))
    }
}

/// Writes the OCCT tessellation of a sphere to a text mesh file and reports its size.
pub fn sphere_mesh_dump(
    radius: f64,
    tolerance: f64,
    path: &str,
) -> Result<(i64, i64), KernelError> {
    let c_path =
        std::ffi::CString::new(path).map_err(|_| KernelError::InvalidInput("path".to_string()))?;
    let mut vertices = 0i64;
    let mut triangles = 0i64;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_sphere_mesh_dump(
            radius,
            tolerance,
            c_path.as_ptr(),
            &mut vertices,
            &mut triangles,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((vertices, triangles))
    } else {
        Err(status_message(code, &message))
    }
}

/// Topology counts and validity for a primitive (0 box, 1 sphere, 2 cylinder).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Topology {
    pub vertices: i64,
    pub edges: i64,
    pub faces: i64,
    pub wires: i64,
    pub valid: bool,
}

pub fn topology(kind: i32) -> Result<Topology, KernelError> {
    let (mut vertices, mut edges, mut faces, mut wires) = (0i64, 0i64, 0i64, 0i64);
    let mut valid = 0i32;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_topology(
            kind,
            &mut vertices,
            &mut edges,
            &mut faces,
            &mut wires,
            &mut valid,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(Topology {
            vertices,
            edges,
            faces,
            wires,
            valid: valid == 1,
        })
    } else {
        Err(status_message(code, &message))
    }
}

/// Result of translating, rotating about Z and scaling a 10x10x4 box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformReport {
    pub min_x: f64,
    pub max_x: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub volume: f64,
    pub valid: bool,
}

pub fn transform_box(
    dx: f64,
    dy: f64,
    dz: f64,
    rotate_deg: f64,
    scale: f64,
) -> Result<TransformReport, KernelError> {
    let (mut min_x, mut max_x, mut min_z, mut max_z) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut volume = 0.0f64;
    let mut valid = 0i32;
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_transform_probe(
            dx,
            dy,
            dz,
            rotate_deg,
            scale,
            &mut min_x,
            &mut max_x,
            &mut min_z,
            &mut max_z,
            &mut volume,
            &mut valid,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(TransformReport {
            min_x,
            max_x,
            min_z,
            max_z,
            volume,
            valid: valid == 1,
        })
    } else {
        Err(status_message(code, &message))
    }
}

/// A point and its first derivative on the rational NURBS quarter circle of radius 5.
pub fn curve_point_and_derivative(u: f64) -> Result<([f64; 3], [f64; 3]), KernelError> {
    let (mut px, mut py, mut pz) = (0.0f64, 0.0f64, 0.0f64);
    let (mut dx, mut dy, mut dz) = (0.0f64, 0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_curve_eval(
            u,
            &mut px,
            &mut py,
            &mut pz,
            &mut dx,
            &mut dy,
            &mut dz,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(([px, py, pz], [dx, dy, dz]))
    } else {
        Err(status_message(code, &message))
    }
}

/// A point and its unit normal on the radius-5 sphere.
pub fn surface_point_and_normal(u: f64, v: f64) -> Result<([f64; 3], [f64; 3]), KernelError> {
    let (mut px, mut py, mut pz) = (0.0f64, 0.0f64, 0.0f64);
    let (mut nx, mut ny, mut nz) = (0.0f64, 0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_surface_eval(
            u,
            v,
            &mut px,
            &mut py,
            &mut pz,
            &mut nx,
            &mut ny,
            &mut nz,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok(([px, py, pz], [nx, ny, nz]))
    } else {
        Err(status_message(code, &message))
    }
}

/// A line along +Z through the box: the face hits and the first hit point.
pub fn line_face_hits() -> Result<(i64, Option<[f64; 3]>), KernelError> {
    let mut hits = 0i64;
    let (mut x, mut y, mut z) = (0.0f64, 0.0f64, 0.0f64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_line_face_hits(
            &mut hits,
            &mut x,
            &mut y,
            &mut z,
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code == OK {
        Ok((hits, (hits > 0).then_some([x, y, z])))
    } else {
        Err(status_message(code, &message))
    }
}

/// A tessellated sphere as GPU-ready arrays.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshGeometry {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Tessellate a sphere into positions and triangle indices.
pub fn sphere_triangles(radius: f64, tolerance: f64) -> Result<MeshGeometry, KernelError> {
    let (mut vertex_count, mut index_count) = (0i64, 0i64);
    let mut message = [0 as c_char; 512];
    let code = unsafe {
        om9_occt_sphere_triangles(
            radius,
            tolerance,
            &mut vertex_count,
            &mut index_count,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code != OK {
        return Err(status_message(code, &message));
    }
    let mut positions = vec![0.0f32; vertex_count as usize * 3];
    let mut indices = vec![0u32; index_count as usize];
    let code = unsafe {
        om9_occt_sphere_triangles(
            radius,
            tolerance,
            &mut vertex_count,
            &mut index_count,
            positions.as_mut_ptr(),
            indices.as_mut_ptr(),
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if code != OK {
        return Err(status_message(code, &message));
    }
    let positions = positions
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();
    Ok(MeshGeometry { positions, indices })
}
