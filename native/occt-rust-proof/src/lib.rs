//! M00-L2-02 / L4-04 proof harness — **feasibility only**, not the product bridge.
//!
//! M00 is a gate: `planning_state` stays `NOT_STARTED` and no CAD code lives here. What the gate needs
//! is a *minimal* Rust application that calls the pinned OCCT, which is why this crate sits under
//! `native/` as its own workspace and is not a member of the OM9 workspace. The product bridge
//! (`crates/om9-kernel`) is created in **M01-L2-01-05** from this proof.
//!
//! The ABI is a C ABI of complete operations: no `TopoDS_Shape` or OCCT `Handle` crosses into Rust, so
//! no kernel lifetime escapes. A kernel exception is caught inside `bridge.cpp` and returned as a status
//! code plus a message.

use std::ffi::{CStr, c_char, c_double, c_int};
use std::fmt;

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
    let code = unsafe {
        om9_occt_invalid_cylinder(radius, message.as_mut_ptr(), message.len() as c_int)
    };
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
        om9_occt_create_destroy(count, &mut elapsed, message.as_mut_ptr(), message.len() as c_int)
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
        Err(KernelError::Failure("fillet result is not a valid solid".to_string()))
    } else {
        Err(status_message(code, &message))
    }
}

/// Volume of a box after an inward offset.
pub fn offset_volume(width: f64, height: f64, depth: f64, distance: f64) -> Result<f64, KernelError> {
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
        Err(KernelError::Failure("offset result is not a valid solid".to_string()))
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

/// Placeholder kept in the numeric probe so the symbol list stays explicit.
pub const create_destroy_placeholder: () = ();

/// Writes the OCCT tessellation of a sphere to a text mesh file and reports its size.
pub fn sphere_mesh_dump(radius: f64, tolerance: f64, path: &str) -> Result<(i64, i64), KernelError> {
    let c_path = std::ffi::CString::new(path).map_err(|_| KernelError::InvalidInput("path".to_string()))?;
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
