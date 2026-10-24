//! openNURBS 3DM mesh import for OpenMatrix9.
//!
//! The gate proved the pinned openNURBS can write and read a version-5 archive (`G00-04`). This crate is
//! the product path: it reads every mesh of a `.3dm` file through a small C ABI, converts the triangles
//! into [`om9_core::geometry::Mesh`], and hands them to the document or the viewport.
//!
//! The C ABI stays narrow on purpose: counts first, then one fill call, so a large file costs two
//! `std::fs` passes and no per-mesh round trip.
#![deny(unsafe_op_in_unsafe_fn)]

use om9_core::{
    Error,
    geometry::{Mesh, Point3},
};
use std::{
    ffi::{CStr, c_char, c_int, c_longlong},
    fmt,
    path::Path,
};

const OK: c_int = 0;
const INVALID_INPUT: c_int = 1;

/// What an import is allowed to cost, in vertices, triangle corners and file bytes.
///
/// A cap is not optional: the archive carries the counts, so an untrusted file can otherwise ask the
/// importer to allocate any amount of memory before a single vertex is read. The defaults are derived
/// from a memory budget instead of being round numbers:
///
/// * peak bytes per vertex ≈ 60 B (positions `f64` in the bridge, the `Mesh` copy, and the `f32` copy
///   used by the viewport);
/// * peak bytes per triangle corner ≈ 8 B (the `u32` written by the bridge plus the `u32` in the `Mesh`);
/// * a closed triangle surface has about two triangles per vertex, so six corners per vertex.
///
/// That is ≈ `60 + 6 * 8 = 108` bytes per vertex, which is where `from_memory` gets its divisor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportBudget {
    pub max_bytes: u64,
    pub max_vertices: usize,
    pub max_corners: usize,
}

impl ImportBudget {
    /// Derives the caps from a total memory budget for one import.
    pub fn from_memory(bytes: u64) -> Self {
        const BYTES_PER_VERTEX: u64 = 108;
        let vertices = (bytes / BYTES_PER_VERTEX).max(1).min(usize::MAX as u64);
        Self {
            max_bytes: bytes,
            max_vertices: vertices as usize,
            max_corners: (vertices as usize).saturating_mul(6),
        }
    }

    /// The default budget: the document limit's 256 MiB, which is what a normal import may use.
    pub fn document() -> Self {
        Self::from_memory(om9_core::budget::Limits::default().max_bytes)
    }

    /// A larger budget for sculpt-scale meshes; callers must ask for it explicitly.
    pub fn sculpt() -> Self {
        Self::from_memory(1024 * 1024 * 1024)
    }
}

impl Default for ImportBudget {
    /// No fixed ceiling: an import may use half of the physical memory that is free right now.
    fn default() -> Self {
        Self::detect()
    }
}

/// Windows reports the free and total physical memory in this layout.
#[repr(C)]
#[derive(Default)]
struct MemoryStatusEx {
    length: u32,
    memory_load: u32,
    total_physical: u64,
    available_physical: u64,
    total_page_file: u64,
    available_page_file: u64,
    total_virtual: u64,
    available_virtual: u64,
    available_extended_virtual: u64,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
}

fn available_physical_bytes() -> Option<u64> {
    let mut status = MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        ..Default::default()
    };
    // SAFETY: the structure is plain data with its own length field, and the call only writes into it.
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    (ok != 0 && status.available_physical > 0).then_some(status.available_physical)
}

impl ImportBudget {
    /// Measures the free physical memory and spends at most **three quarters** of it on one import.
    ///
    /// There is no fixed ceiling: a machine with 32 GiB free may import what fits, and the remaining
    /// quarter is left so the rest of the application keeps running. A 64 MiB floor keeps tiny imports
    /// possible on a loaded system.
    pub fn detect() -> Self {
        match ImportBudget::available_bytes() {
            Some(available) => Self::from_memory((available / 4 * 3).max(64 * 1024 * 1024)),
            None => Self::document(),
        }
    }

    /// The free physical memory this process could use, if the platform reports it.
    pub fn available_bytes() -> Option<u64> {
        available_physical_bytes()
    }
}

/// The measured reason an import was refused, with the numbers the user needs to decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetReport {
    pub needed_bytes: u64,
    pub allowed_bytes: u64,
    pub needed_vertices: usize,
    pub allowed_vertices: usize,
    pub needed_corners: usize,
    pub allowed_corners: usize,
}

impl BudgetReport {
    fn bytes(needed: u64, allowed: u64) -> Self {
        Self {
            needed_bytes: needed,
            allowed_bytes: allowed,
            needed_vertices: 0,
            allowed_vertices: 0,
            needed_corners: 0,
            allowed_corners: 0,
        }
    }

    fn geometry(
        needed_vertices: usize,
        allowed_vertices: usize,
        needed_corners: usize,
        allowed_corners: usize,
    ) -> Self {
        Self {
            needed_bytes: 0,
            allowed_bytes: 0,
            needed_vertices,
            allowed_vertices,
            needed_corners,
            allowed_corners,
        }
    }
}

impl fmt::Display for BudgetReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.needed_bytes > 0 {
            let format = |bytes: u64| {
                if bytes >= 1024 * 1024 {
                    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{bytes} bytes")
                }
            };
            return write!(
                f,
                "the archive is {} but the import budget is {} (75% of the free memory at the time of the import)",
                format(self.needed_bytes),
                format(self.allowed_bytes)
            );
        }
        write!(
            f,
            "the archive needs {} vertices and {} triangle corners, but the budget allows {} vertices and {} corners; close files or raise the budget",
            self.needed_vertices, self.needed_corners, self.allowed_vertices, self.allowed_corners
        )
    }
}

/// Why an import did not produce meshes.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    /// The archive is larger than the memory budget; the report carries both numbers.
    Budget(BudgetReport),
    /// The archive could not be read or parsed.
    Archive(String),
    /// The request itself is not usable.
    Invalid(&'static str),
    /// A core model rule rejected the mesh (limits, identity, geometry).
    Core(Error),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget(report) => write!(f, "3DM import refused: {report}"),
            Self::Archive(detail) => write!(f, "3DM archive could not be read: {detail}"),
            Self::Invalid(detail) => write!(f, "3DM import refused: {detail}"),
            Self::Core(error) => write!(f, "3DM import failed: {error}"),
        }
    }
}

impl std::error::Error for ImportError {}

impl From<Error> for ImportError {
    fn from(error: Error) -> Self {
        Self::Core(error)
    }
}

unsafe extern "C" {
    fn om9_3dm_write_jewelry(
        path: *const c_char,
        rings: c_int,
        diamonds: c_int,
        ring_u: c_int,
        ring_v: c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_3dm_write_fixture(
        path: *const c_char,
        mesh_count: c_int,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_3dm_read(
        path: *const c_char,
        mesh_count: *mut c_longlong,
        vertex_count: *mut c_longlong,
        index_count: *mut c_longlong,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_3dm_fill(
        path: *const c_char,
        positions: *mut f64,
        indices: *mut u32,
        offsets: *mut c_longlong,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}

fn message_of(buffer: &[c_char; 512]) -> String {
    // SAFETY: the bridge always NUL-terminates within the buffer it was given.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn status(code: c_int, buffer: &[c_char; 512]) -> std::result::Result<(), ImportError> {
    match code {
        OK => Ok(()),
        INVALID_INPUT => Err(ImportError::Invalid("3dm import input")),
        _ => Err(ImportError::Archive(message_of(buffer))),
    }
}

/// Writes a fixture with `mesh_count` unit cubes, each translated along +X, for tests and benchmarks.
pub fn write_fixture(path: &Path, mesh_count: usize) -> std::result::Result<(), ImportError> {
    let text = path.to_string_lossy().into_owned();
    let c_path = std::ffi::CString::new(text).map_err(|_| ImportError::Invalid("path"))?;
    let mut buffer = [0 as c_char; 512];
    let code = unsafe {
        om9_3dm_write_fixture(
            c_path.as_ptr(),
            mesh_count.min(i32::MAX as usize) as c_int,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    status(code, &buffer)
}

/// Writes a jewelry-like archive: `rings` torus bands and `diamonds` brilliant proxies on a grid.
///
/// `ring_u` and `ring_v` control the torus tessellation, so a caller can trade bytes for silhouette
/// quality the way a real ring mesh does.
pub fn write_jewelry_fixture(
    path: &Path,
    rings: usize,
    diamonds: usize,
    ring_u: usize,
    ring_v: usize,
) -> std::result::Result<(), ImportError> {
    let text = path.to_string_lossy().into_owned();
    let c_path = std::ffi::CString::new(text).map_err(|_| ImportError::Invalid("path"))?;
    let mut buffer = [0 as c_char; 512];
    let code = unsafe {
        om9_3dm_write_jewelry(
            c_path.as_ptr(),
            rings.min(i32::MAX as usize) as c_int,
            diamonds.min(i32::MAX as usize) as c_int,
            ring_u.min(i32::MAX as usize) as c_int,
            ring_v.min(i32::MAX as usize) as c_int,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    status(code, &buffer)
}

/// Reads every mesh of a `.3dm` file. Each triangle becomes a three-index face; openNURBS quads are
/// split into two triangles, which is what the viewport draws anyway.
pub fn read_meshes(path: &Path) -> std::result::Result<Vec<Mesh>, ImportError> {
    read_meshes_with(path, ImportBudget::default())
}

/// Reads every mesh of a `.3dm` file under an explicit budget.
pub fn read_meshes_with(
    path: &Path,
    budget: ImportBudget,
) -> std::result::Result<Vec<Mesh>, ImportError> {
    let file_bytes = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    if file_bytes > budget.max_bytes {
        return Err(ImportError::Budget(BudgetReport::bytes(
            file_bytes,
            budget.max_bytes,
        )));
    }
    let text = path.to_string_lossy().into_owned();
    let c_path = std::ffi::CString::new(text).map_err(|_| ImportError::Invalid("path"))?;
    let mut buffer = [0 as c_char; 512];
    let (mut mesh_count, mut vertex_count, mut index_count) = (0i64, 0i64, 0i64);
    let code = unsafe {
        om9_3dm_read(
            c_path.as_ptr(),
            &mut mesh_count,
            &mut vertex_count,
            &mut index_count,
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    status(code, &buffer)?;
    if vertex_count < 0 || index_count < 0 || mesh_count < 0 {
        return Err(ImportError::Invalid("3dm negative counts"));
    }
    let vertices_total = vertex_count as usize;
    let indices_total = index_count as usize;
    if vertices_total > budget.max_vertices || indices_total > budget.max_corners {
        return Err(ImportError::Budget(BudgetReport::geometry(
            vertices_total,
            budget.max_vertices,
            indices_total,
            budget.max_corners,
        )));
    }
    if mesh_count == 0 {
        return Ok(Vec::new());
    }
    let mut positions = vec![0.0f64; vertices_total * 3];
    let mut indices = vec![0u32; indices_total];
    let mut offsets = vec![0i64; mesh_count as usize * 4];
    let code = unsafe {
        om9_3dm_fill(
            c_path.as_ptr(),
            positions.as_mut_ptr(),
            indices.as_mut_ptr(),
            offsets.as_mut_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as c_int,
        )
    };
    status(code, &buffer)?;
    let mut meshes = Vec::with_capacity(mesh_count as usize);
    for mesh in 0..mesh_count as usize {
        let vertex_start = offsets[mesh * 4].max(0) as usize;
        let vertex_len = offsets[mesh * 4 + 1].max(0) as usize;
        let index_start = offsets[mesh * 4 + 2].max(0) as usize;
        let index_len = offsets[mesh * 4 + 3].max(0) as usize;
        if vertex_start + vertex_len > vertices_total || index_start + index_len > indices_total {
            return Err(ImportError::Invalid("3dm mesh range"));
        }
        let vertices: Vec<Point3> = (0..vertex_len)
            .map(|offset| {
                let base = (vertex_start + offset) * 3;
                [positions[base], positions[base + 1], positions[base + 2]]
            })
            .collect();
        let faces: Vec<Vec<u32>> = indices[index_start..index_start + index_len]
            .chunks(3)
            .filter(|chunk| chunk.len() == 3)
            .map(|chunk| chunk.to_vec())
            .collect();
        meshes.push(Mesh {
            normals: Vec::new(),
            colors: Vec::new(),
            vertices,
            faces,
            corner_uvs: Vec::new(),
        });
    }
    Ok(meshes)
}

/// Flat vertex/index buffers ready for `SceneBatch`, one mesh per entry.
pub struct FlatMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl FlatMesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Converts one imported mesh to the flat layout the renderer uploads.
pub fn flatten(mesh: &Mesh) -> FlatMesh {
    let positions = mesh
        .vertices
        .iter()
        .map(|point| [point[0] as f32, point[1] as f32, point[2] as f32])
        .collect();
    let mut indices = Vec::with_capacity(mesh.faces.len() * 3);
    for face in &mesh.faces {
        for corner in face.iter().take(3) {
            indices.push(*corner);
        }
    }
    FlatMesh { positions, indices }
}
