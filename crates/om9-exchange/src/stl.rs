//! Binary STL export (`OM9-L3-01-12-008`, `OM9-L3-01-12-009`).
//!
//! STL carries no unit and no topology, so two things must be decided on the way out and written down:
//! the **scale** applied to every coordinate (OM9 stores millimetres; a slicer that expects inches gets
//! `1/25.4`), and the **winding**, which is the only statement of which side is outside. Both are
//! explicit here rather than implicit in a caller.
//!
//! A mesh with an open edge is not watertight, and most slicers print it wrong instead of saying so. The
//! strict entry point refuses such a mesh and reports how many edges are open.

use om9_core::Error;
use std::io::{self, Write};

/// A triangle soup: positions plus counter-clockwise triangles that index into them.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleSoup {
    pub positions: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

impl TriangleSoup {
    pub fn new(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            positions: positions.into_iter().map(|p| p.map(f64::from)).collect(),
            triangles,
        }
    }

    pub fn new_f64(positions: Vec<[f64; 3]>, triangles: Vec<[u32; 3]>) -> Self {
        Self { positions, triangles }
    }
}

/// Why an export was refused. Every variant is a defect of the geometry or of the request, never of the
/// writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StlError {
    /// The mesh has no triangles; an empty STL file is never useful.
    Empty,
    /// A triangle indexes a vertex that does not exist.
    IndexOutOfRange { triangle: usize, index: u32 },
    /// A coordinate is NaN or infinite.
    NonFinite { vertex: usize },
    /// The scale factor is not a finite, strictly positive number.
    BadScale,
    /// A triangle has no area, so it has no normal.
    Degenerate { triangle: usize },
    /// Conversion to binary-STL coordinates exceeds the requested error budget.
    Quantization { vertex: usize },
    /// A triangle collapses or reverses during f32 conversion.
    QuantizedTriangle { triangle: usize },
    /// Different input positions become the same encoded STL position.
    QuantizationCollision { first: usize, second: usize },
    /// The encoder could not allocate its quantized-position index.
    Allocation,
    /// `strict` was asked for and the surface is not closed.
    OpenSurface { open_edges: usize },
    /// Edge incidence, winding or vertex-fan defects prevent a manifold claim.
    InvalidTopology { non_manifold_edges: usize, inconsistent_edges: usize, disconnected_vertices: usize, duplicate_faces: usize },
    /// The binary format stores the triangle count in a `u32`.
    TooManyTriangles { count: usize },
}

impl std::fmt::Display for StlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "the mesh has no triangles"),
            Self::IndexOutOfRange { triangle, index } => {
                write!(
                    formatter,
                    "triangle {triangle} indexes vertex {index}, which does not exist"
                )
            }
            Self::NonFinite { vertex } => {
                write!(formatter, "vertex {vertex} is not a finite coordinate")
            }
            Self::BadScale => write!(
                formatter,
                "the export scale must be finite and greater than zero"
            ),
            Self::Degenerate { triangle } => write!(formatter, "triangle {triangle} has no area"),
            Self::Quantization { vertex } => write!(formatter, "vertex {vertex} exceeds STL quantization tolerance"),
            Self::QuantizedTriangle { triangle } => write!(formatter, "triangle {triangle} collapses or reverses after STL quantization"),
            Self::QuantizationCollision { first, second } => write!(formatter, "STL quantization merges distinct vertices {first} and {second}"),
            Self::Allocation => write!(formatter, "not enough memory for STL validation"),
            Self::OpenSurface { open_edges } => {
                write!(
                    formatter,
                    "the surface is not watertight: {open_edges} open edges"
                )
            }
            Self::InvalidTopology { non_manifold_edges, inconsistent_edges, disconnected_vertices, duplicate_faces } => write!(formatter, "invalid STL topology: {non_manifold_edges} non-manifold edges, {inconsistent_edges} inconsistent edges, {disconnected_vertices} disconnected vertices, {duplicate_faces} duplicate faces"),
            Self::TooManyTriangles { count } => {
                write!(formatter, "binary STL cannot hold {count} triangles")
            }
        }
    }
}

impl std::error::Error for StlError {}

#[derive(Debug)]
pub enum StlStreamError {
    Geometry(StlError),
    Io(io::Error),
}
impl From<StlError> for StlStreamError { fn from(error: StlError) -> Self { Self::Geometry(error) } }
impl From<io::Error> for StlStreamError { fn from(error: io::Error) -> Self { Self::Io(error) } }
impl std::fmt::Display for StlStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Geometry(error) => error.fmt(f), Self::Io(error) => error.fmt(f) }
    }
}
impl std::error::Error for StlStreamError {}

impl From<StlError> for Error {
    fn from(value: StlError) -> Self {
        match value {
            StlError::IndexOutOfRange { .. }
            | StlError::NonFinite { .. }
            | StlError::BadScale
            | StlError::Degenerate { .. }
            | StlError::Quantization { .. }
            | StlError::QuantizedTriangle { .. } => Error::Invalid("stl geometry or quantization"),
            StlError::QuantizationCollision { .. } => Error::Invalid("STL quantization merges distinct vertices"),
            StlError::Allocation => Error::Budget("STL validation"),
            StlError::Empty => Error::Invalid("empty mesh"),
            StlError::OpenSurface { .. } => Error::Invalid("stl surface not watertight"),
            StlError::InvalidTopology { .. } => Error::Invalid("stl non-manifold topology"),
            StlError::TooManyTriangles { .. } => Error::Unsupported("stl triangle count"),
        }
    }
}

/// How to write the file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StlOptions {
    /// Multiplied into every coordinate. `1.0` writes millimetres, `1.0 / 25.4` writes inches.
    pub scale: f64,
    /// Swap the triangle order, for a mesh whose winding was authored the other way round.
    pub flip_winding: bool,
    /// Refuse a surface with open edges instead of writing a file a slicer will misread.
    pub strict: bool,
}

impl Default for StlOptions {
    fn default() -> Self {
        Self {
            scale: 1.0,
            flip_winding: false,
            strict: true,
        }
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normal_of(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<[f32; 3]> {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = cross(ab, ac);
    let length = n[0].hypot(n[1]).hypot(n[2]);
    if !length.is_finite() || length == 0.0 {
        return None;
    }
    Some([
        (n[0] / length) as f32,
        (n[1] / length) as f32,
        (n[2] / length) as f32,
    ])
}

/// Topological checks on indices. These do not detect geometric self-intersections.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EdgeTopology {
    pub boundary: usize,
    pub non_manifold: usize,
    pub inconsistent_winding: usize,
    pub disconnected_vertices: usize,
    /// Triangle components connected through shared indexed edges, not geometric contact.
    pub edge_connected_components: usize,
    /// Repeated indexed triangles after their first occurrence, ignoring winding.
    pub duplicate_faces: usize,
}

pub fn edge_topology(soup: &TriangleSoup) -> EdgeTopology {
    edge_topology_with_check(soup, || Ok::<(), std::convert::Infallible>(())).unwrap()
}

/// Check cancellation periodically while building and traversing topology.
pub fn edge_topology_with_check<E>(soup: &TriangleSoup, mut check: impl FnMut() -> Result<(), E>) -> Result<EdgeTopology, E> {
    try_edge_topology_with_check(soup, &mut check, || panic!("STL topology allocation failed"))
}

/// Fallible topology validation for callers that must preserve the destination
/// when scratch allocations cannot be satisfied.
pub fn try_edge_topology_with_check<E>(soup: &TriangleSoup, check: impl FnMut() -> Result<(), E>, allocation_error: impl FnMut() -> E) -> Result<EdgeTopology, E> {
    try_edge_topology_with_admission(soup, check, allocation_error, |_| Ok(()))
}

/// Check known edge, fan and traversal storage at allocation boundaries.
/// Hash-table metadata and allocator overhead are excluded.
pub fn try_edge_topology_with_admission<E>(
    soup: &TriangleSoup, mut check: impl FnMut() -> Result<(), E>, mut allocation_error: impl FnMut() -> E,
    mut admit: impl FnMut(u64) -> Result<(), E>,
) -> Result<EdgeTopology, E> {
    use std::collections::HashMap;
    fn known(edges: usize, faces: usize, parents: usize, ranks: usize, extra: usize) -> Option<u64> {
        let bytes = edges.checked_mul(std::mem::size_of::<((u32, u32), (usize, usize, usize))>())?
            .checked_add(faces.checked_mul(std::mem::size_of::<[u32; 3]>())?)?
            .checked_add(parents.checked_mul(std::mem::size_of::<usize>())?)?
            .checked_add(ranks.checked_mul(std::mem::size_of::<u8>())?)?
            .checked_add(extra)?;
        u64::try_from(bytes).ok()
    }
    fn fan_known(table: usize, links: usize, extra: usize) -> Option<u64> {
        let bytes = table.checked_mul(std::mem::size_of::<(u32, Vec<(u32, u32)>)>())?
            .checked_add(links)?.checked_add(extra)?;
        u64::try_from(bytes).ok()
    }
    check()?;
    let mut edges: HashMap<(u32, u32), (usize, usize, usize)> = HashMap::new();
    let mut faces = std::collections::HashSet::new();
    let mut duplicates = 0;
    let mut parents = Vec::new();
    admit(known(0, 0, 0, 0, soup.triangles.len().checked_mul(std::mem::size_of::<usize>()).ok_or_else(&mut allocation_error)?).ok_or_else(&mut allocation_error)?)?;
    parents.try_reserve_exact(soup.triangles.len()).map_err(|_| allocation_error())?;
    parents.extend(0..soup.triangles.len());
    let mut ranks = Vec::new();
    admit(known(0, 0, parents.capacity(), 0, soup.triangles.len()).ok_or_else(&mut allocation_error)?)?;
    ranks.try_reserve_exact(soup.triangles.len()).map_err(|_| allocation_error())?;
    ranks.resize(soup.triangles.len(), 0u8);
    admit(known(0, 0, parents.capacity(), ranks.capacity(), 0).ok_or_else(&mut allocation_error)?)?;
    let mut components = soup.triangles.len();
    fn root(parents: &mut [usize], mut node: usize) -> usize {
        while parents[node] != node {
            parents[node] = parents[parents[node]];
            node = parents[node];
        }
        node
    }
    for (index, triangle) in soup.triangles.iter().enumerate() {
        if index % 1024 == 0 { check()?; }
        let mut face = *triangle;
        face.sort_unstable();
        let face_target = faces.len().checked_add(1).ok_or_else(&mut allocation_error)?;
        let face_grows = face_target > faces.capacity();
        if face_grows {
            let extra = face_target.checked_mul(std::mem::size_of::<[u32; 3]>()).ok_or_else(&mut allocation_error)?;
            admit(known(edges.capacity(), faces.capacity(), parents.capacity(), ranks.capacity(), extra).ok_or_else(&mut allocation_error)?)?;
        }
        faces.try_reserve(1).map_err(|_| allocation_error())?;
        if face_grows { admit(known(edges.capacity(), faces.capacity(), parents.capacity(), ranks.capacity(), 0).ok_or_else(&mut allocation_error)?)?; }
        if !faces.insert(face) { duplicates += 1; }
        let edge_target = edges.len().checked_add(3).ok_or_else(&mut allocation_error)?;
        let edge_grows = edge_target > edges.capacity();
        if edge_grows {
            let extra = edge_target.checked_mul(std::mem::size_of::<((u32, u32), (usize, usize, usize))>()).ok_or_else(&mut allocation_error)?;
            admit(known(edges.capacity(), faces.capacity(), parents.capacity(), ranks.capacity(), extra).ok_or_else(&mut allocation_error)?)?;
        }
        edges.try_reserve(3).map_err(|_| allocation_error())?;
        if edge_grows { admit(known(edges.capacity(), faces.capacity(), parents.capacity(), ranks.capacity(), 0).ok_or_else(&mut allocation_error)?)?; }
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let key = if a <= b { (a, b) } else { (b, a) };
            match edges.entry(key) {
                std::collections::hash_map::Entry::Vacant(entry) => { entry.insert((1, usize::from(a < b), index)); }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let edge = entry.get_mut();
                    edge.0 += 1;
                    if a < b { edge.1 += 1; }
                    let mut first = root(&mut parents, edge.2);
                    let mut second = root(&mut parents, index);
                    if first != second {
                        if ranks[first] < ranks[second] { std::mem::swap(&mut first, &mut second); }
                        parents[second] = first;
                        if ranks[first] == ranks[second] { ranks[first] += 1; }
                        components -= 1;
                    }
                }
            }
        }
    }
    let mut result = EdgeTopology { duplicate_faces: duplicates, edge_connected_components: components, ..EdgeTopology::default() };
    for (index, &(count, forward, _)) in edges.values().enumerate() {
        if index % 1024 == 0 { check()?; }
        match count {
            1 => result.boundary += 1,
            2 if forward != 1 => result.inconsistent_winding += 1,
            2 => {},
            _ => result.non_manifold += 1,
        }
    }
    // The face/edge maps and union-find arrays are no longer needed while
    // inspecting vertex fans. Avoid keeping both large scratch sets alive.
    drop(faces);
    drop(edges);
    drop(parents);
    drop(ranks);
    let mut fans: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
    let mut links_bytes = 0usize;
    for (index, triangle) in soup.triangles.iter().enumerate() {
        if index % 1024 == 0 { check()?; }
        let target = fans.len().checked_add(3).ok_or_else(&mut allocation_error)?;
        let grows = target > fans.capacity();
        if grows {
            let extra = target.checked_mul(std::mem::size_of::<(u32, Vec<(u32, u32)>)>()).ok_or_else(&mut allocation_error)?;
            admit(fan_known(fans.capacity(), links_bytes, extra).ok_or_else(&mut allocation_error)?)?;
        }
        fans.try_reserve(3).map_err(|_| allocation_error())?;
        if grows { admit(fan_known(fans.capacity(), links_bytes, 0).ok_or_else(&mut allocation_error)?)?; }
        let table_capacity = fans.capacity();
        for (v, a, b) in [(triangle[0], triangle[1], triangle[2]), (triangle[1], triangle[2], triangle[0]), (triangle[2], triangle[0], triangle[1])] {
            let links = fans.entry(v).or_default();
            let old_capacity = links.capacity();
            let target = links.len().checked_add(1).ok_or_else(&mut allocation_error)?;
            let grows = target > old_capacity;
            if grows {
                let extra = target.checked_mul(std::mem::size_of::<(u32, u32)>()).ok_or_else(&mut allocation_error)?;
                admit(fan_known(table_capacity, links_bytes, extra).ok_or_else(&mut allocation_error)?)?;
            }
            links.try_reserve(1).map_err(|_| allocation_error())?;
            if grows {
                let old_bytes = old_capacity.checked_mul(std::mem::size_of::<(u32, u32)>()).ok_or_else(&mut allocation_error)?;
                let new_bytes = links.capacity().checked_mul(std::mem::size_of::<(u32, u32)>()).ok_or_else(&mut allocation_error)?;
                links_bytes = links_bytes.checked_sub(old_bytes).and_then(|bytes| bytes.checked_add(new_bytes)).ok_or_else(&mut allocation_error)?;
                admit(fan_known(table_capacity, links_bytes, 0).ok_or_else(&mut allocation_error)?)?;
            }
            links.push((a, b));
        }
    }
    for (index, links) in fans.values().enumerate() {
        if index % 1024 == 0 { check()?; }
        let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();
        let fan_bytes = fan_known(fans.capacity(), links_bytes, 0).ok_or_else(&mut allocation_error)?;
        let mut adjacency_links = 0usize;
        for (link_index, &(a, b)) in links.iter().enumerate() {
            if link_index % 1024 == 0 { check()?; }
            let target = adjacency.len().checked_add(2).ok_or_else(&mut allocation_error)?;
            let grows = target > adjacency.capacity();
            let entry_size = std::mem::size_of::<(u32, Vec<u32>)>();
            let table_bytes = adjacency.capacity().checked_mul(entry_size).ok_or_else(&mut allocation_error)?;
            let base = fan_bytes.checked_add(u64::try_from(table_bytes.checked_add(adjacency_links).ok_or_else(&mut allocation_error)?).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?;
            if grows {
                let extra = target.checked_mul(entry_size).ok_or_else(&mut allocation_error)?;
                admit(base.checked_add(u64::try_from(extra).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
            }
            adjacency.try_reserve(2).map_err(|_| allocation_error())?;
            let table_bytes = adjacency.capacity().checked_mul(entry_size).ok_or_else(&mut allocation_error)?;
            if grows {
                admit(fan_bytes.checked_add(u64::try_from(table_bytes.checked_add(adjacency_links).ok_or_else(&mut allocation_error)?).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
            }
            for (key, value) in [(a, b), (b, a)] {
                let adjacent = adjacency.entry(key).or_default();
                let old = adjacent.capacity();
                let target = adjacent.len().checked_add(1).ok_or_else(&mut allocation_error)?;
                let grows = target > old;
                if grows {
                    let extra = target.checked_mul(std::mem::size_of::<u32>()).ok_or_else(&mut allocation_error)?;
                    let bytes = table_bytes.checked_add(adjacency_links).and_then(|n| n.checked_add(extra)).ok_or_else(&mut allocation_error)?;
                    admit(fan_bytes.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
                }
                adjacent.try_reserve(1).map_err(|_| allocation_error())?;
                if grows {
                    adjacency_links = adjacency_links.checked_sub(old.checked_mul(4).ok_or_else(&mut allocation_error)?)
                        .and_then(|n| n.checked_add(adjacent.capacity().checked_mul(4)?)).ok_or_else(&mut allocation_error)?;
                    let bytes = table_bytes.checked_add(adjacency_links).ok_or_else(&mut allocation_error)?;
                    admit(fan_bytes.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
                }
                adjacent.push(value);
            }
        }
        let Some(&start) = adjacency.keys().next() else { continue };
        let adjacency_bytes = adjacency.capacity().checked_mul(std::mem::size_of::<(u32, Vec<u32>)>())
            .and_then(|n| n.checked_add(adjacency_links)).ok_or_else(&mut allocation_error)?;
        let base = fan_bytes.checked_add(u64::try_from(adjacency_bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?;
        let mut visited = std::collections::HashSet::new();
        let mut pending = Vec::new();
        admit(base.checked_add(4).ok_or_else(&mut allocation_error)?)?;
        pending.try_reserve(1).map_err(|_| allocation_error())?;
        admit(base.checked_add(u64::try_from(pending.capacity().checked_mul(4).ok_or_else(&mut allocation_error)?).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
        pending.push(start);
        let mut visited_count = 0usize;
        while let Some(vertex) = pending.pop() {
            visited_count += 1;
            if visited_count % 1024 == 0 { check()?; }
            let target = visited.len().checked_add(1).ok_or_else(&mut allocation_error)?;
            let grows = target > visited.capacity();
            if grows {
                let bytes = visited.capacity().checked_add(pending.capacity()).and_then(|n: usize| n.checked_add(target)).and_then(|n| n.checked_mul(4)).ok_or_else(&mut allocation_error)?;
                admit(base.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
            }
            visited.try_reserve(1).map_err(|_| allocation_error())?;
            if grows {
                let bytes = visited.capacity().checked_add(pending.capacity()).and_then(|n| n.checked_mul(4)).ok_or_else(&mut allocation_error)?;
                admit(base.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
            }
            if visited.insert(vertex) {
                let additional = adjacency[&vertex].len();
                let target = pending.len().checked_add(additional).ok_or_else(&mut allocation_error)?;
                let grows = target > pending.capacity();
                if grows {
                    let bytes = visited.capacity().checked_add(pending.capacity()).and_then(|n| n.checked_add(target)).and_then(|n| n.checked_mul(4)).ok_or_else(&mut allocation_error)?;
                    admit(base.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
                }
                pending.try_reserve(additional).map_err(|_| allocation_error())?;
                if grows {
                    let bytes = visited.capacity().checked_add(pending.capacity()).and_then(|n| n.checked_mul(4)).ok_or_else(&mut allocation_error)?;
                    admit(base.checked_add(u64::try_from(bytes).map_err(|_| allocation_error())?).ok_or_else(&mut allocation_error)?)?;
                }
                pending.extend(adjacency[&vertex].iter().copied());
            }
        }
        if visited.len() != adjacency.len() { result.disconnected_vertices += 1; }
    }
    check()?;
    Ok(result)
}

/// Number of boundary edges. Use `edge_topology` for other defects.
pub fn open_edges(soup: &TriangleSoup) -> usize {
    edge_topology(soup).boundary
}

/// Writes a binary STL. `strict` refuses open surfaces, degenerate triangles and bad indices; the lenient
/// path still refuses non-finite coordinates because no reader can use them.
pub fn write_binary_stl(
    soup: &TriangleSoup,
    options: StlOptions,
) -> std::result::Result<Vec<u8>, StlError> {
    write_binary_stl_with_tolerance(soup, options, None)
}

/// `max_quantization_error` is Euclidean distance in exported units.
pub fn write_binary_stl_with_tolerance(
    soup: &TriangleSoup,
    options: StlOptions,
    max_quantization_error: Option<f64>,
) -> std::result::Result<Vec<u8>, StlError> {
    let mut bytes = Vec::new();
    match write_binary_stl_to(&mut bytes, soup, options, max_quantization_error) {
        Ok(()) => Ok(bytes),
        Err(StlStreamError::Geometry(error)) => Err(error),
        Err(StlStreamError::Io(_)) => unreachable!("Vec writes do not return I/O errors"),
    }
}

/// Stream an STL to a writer. On failure, the writer can contain a partial file;
/// callers publishing to disk must use a temporary destination.
pub fn write_binary_stl_to(
    writer: &mut impl Write,
    soup: &TriangleSoup,
    options: StlOptions,
    max_quantization_error: Option<f64>,
) -> std::result::Result<(), StlStreamError> {
    write_binary_stl_to_with_check(writer, soup, options, max_quantization_error, || Ok(()))
}

/// As `write_binary_stl_to`, with periodic cancellation checks during preflight,
/// strict topology validation, and encoding. A failed writer may be partial.
pub fn write_binary_stl_to_with_check(
    writer: &mut impl Write,
    soup: &TriangleSoup,
    options: StlOptions,
    max_quantization_error: Option<f64>,
    check: impl FnMut() -> io::Result<()>,
) -> std::result::Result<(), StlStreamError> {
    write_binary_stl_to_with_report(writer, soup, options, max_quantization_error, check).map(|_| ())
}

/// Encoding observations, without an encoded-topology or self-intersection check.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StlWriteReport {
    /// Distinct used source indices have the same encoded position. STL does
    /// not preserve their identity, even when their input positions are equal.
    pub coincident_distinct_indices: bool,
}

/// As `write_binary_stl_to_with_check`, also reporting identity ambiguity.
pub fn write_binary_stl_to_with_report(
    writer: &mut impl Write,
    soup: &TriangleSoup,
    options: StlOptions,
    max_quantization_error: Option<f64>,
    check: impl FnMut() -> io::Result<()>,
) -> std::result::Result<StlWriteReport, StlStreamError> {
    write_binary_stl_to_with_report_and_admission(writer, soup, options, max_quantization_error, check, |_| Ok(()))
}

/// Admit known topology and quantized-position table slots. Hash control bytes,
/// allocator overhead and temporary over-allocation are not included.
pub fn write_binary_stl_to_with_report_and_admission(
    writer: &mut impl Write,
    soup: &TriangleSoup,
    options: StlOptions,
    max_quantization_error: Option<f64>,
    mut check: impl FnMut() -> io::Result<()>,
    mut admit: impl FnMut(u64) -> std::result::Result<(), StlError>,
) -> std::result::Result<StlWriteReport, StlStreamError> {
    check()?;
    if soup.triangles.is_empty() {
        return Err(StlError::Empty.into());
    }
    if !options.scale.is_finite() || options.scale <= 0.0 {
        return Err(StlError::BadScale.into());
    }
    if max_quantization_error.is_some_and(|t| !t.is_finite() || t <= 0.0) {
        return Err(StlError::BadScale.into());
    }
    if soup.triangles.len() > u32::MAX as usize {
        return Err(StlError::TooManyTriangles {
            count: soup.triangles.len(),
        }.into());
    }
    for (index, position) in soup.positions.iter().enumerate() {
        if index % 1024 == 0 { check()?; }
        if position.iter().any(|value| !value.is_finite()) {
            return Err(StlError::NonFinite { vertex: index }.into());
        }
    }
    // Indices are checked before the watertight rule so a malformed mesh reports the real defect.
    for (index, triangle) in soup.triangles.iter().enumerate() {
        if index % 1024 == 0 { check()?; }
        for vertex_index in triangle {
            if *vertex_index as usize >= soup.positions.len() {
                return Err(StlError::IndexOutOfRange {
                    triangle: index,
                    index: *vertex_index,
                }.into());
            }
        }
    }
    if options.strict {
        let topology = try_edge_topology_with_admission(soup, || check().map_err(StlStreamError::Io), || StlStreamError::Geometry(StlError::Allocation), |bytes| admit(bytes).map_err(StlStreamError::Geometry))?;
        if topology.boundary > 0 {
            return Err(StlError::OpenSurface { open_edges: topology.boundary }.into());
        }
        if topology.non_manifold > 0 || topology.inconsistent_winding > 0 || topology.disconnected_vertices > 0 || topology.duplicate_faces > 0 {
            return Err(StlError::InvalidTopology { non_manifold_edges: topology.non_manifold, inconsistent_edges: topology.inconsistent_winding, disconnected_vertices: topology.disconnected_vertices, duplicate_faces: topology.duplicate_faces }.into());
        }
    }
    // The header is free text; say what the coordinates mean so a reader can check.
    let header = format!(
        "OM9 binary STL, {} triangles, scale {} mm-per-unit",
        soup.triangles.len(),
        options.scale
    );
    let mut header_bytes = [0u8; 80];
    let header_bytes_written = header.len().min(80);
    header_bytes[..header_bytes_written]
        .copy_from_slice(&header.as_bytes()[..header_bytes_written]);
    writer.write_all(&header_bytes)?;
    writer.write_all(&(soup.triangles.len() as u32).to_le_bytes())?;
    let mut encoded_positions = std::collections::HashMap::<[u32; 3], ([f64; 3], usize)>::new();
    let mut report = StlWriteReport::default();
    for (index, triangle) in soup.triangles.iter().enumerate() {
        if index % 1024 == 0 { check()?; }
        let mut vertices = [[0.0f64; 3]; 3];
        for (slot, vertex_index) in triangle.iter().enumerate() {
            let position =
                soup.positions
                    .get(*vertex_index as usize)
                    .ok_or(StlError::IndexOutOfRange {
                        triangle: index,
                        index: *vertex_index,
                    })?;
            for axis in 0..3 {
                vertices[slot][axis] = position[axis] * options.scale;
            }
        }
        let mut quantized = [[0.0f64; 3]; 3];
        for (slot, vertex) in vertices.iter().enumerate() {
            let mut distance_sq = 0.0;
            for axis in 0..3 {
                let value = vertex[axis] as f32;
                if !value.is_finite() { return Err(StlError::Quantization { vertex: triangle[slot] as usize }.into()); }
                quantized[slot][axis] = f64::from(value);
                distance_sq += (quantized[slot][axis] - vertex[axis]).powi(2);
            }
            if max_quantization_error.is_some_and(|t| distance_sq.sqrt() > t) {
                return Err(StlError::Quantization { vertex: triangle[slot] as usize }.into());
            }
        }
        let source_normal = normal_of(vertices[0], vertices[1], vertices[2]).ok_or(StlError::Degenerate { triangle: index })?;
        let encoded_normal = normal_of(quantized[0], quantized[1], quantized[2]).ok_or(StlError::QuantizedTriangle { triangle: index })?;
        if (0..3).map(|i| f64::from(source_normal[i]) * f64::from(encoded_normal[i])).sum::<f64>() <= 0.0 {
            return Err(StlError::QuantizedTriangle { triangle: index }.into());
        }
        let entry_size = std::mem::size_of::<([u32; 3], ([f64; 3], usize))>();
        let target = encoded_positions.len().checked_add(3).ok_or(StlError::Allocation)?;
        let grows = target > encoded_positions.capacity();
        if grows {
            let peak = encoded_positions.capacity().checked_add(target).and_then(|n| n.checked_mul(entry_size)).ok_or(StlError::Allocation)?;
            admit(u64::try_from(peak).map_err(|_| StlError::Allocation)?)?;
        }
        encoded_positions.try_reserve(3).map_err(|_| StlError::Allocation)?;
        if grows {
            let bytes = encoded_positions.capacity().checked_mul(entry_size).ok_or(StlError::Allocation)?;
            admit(u64::try_from(bytes).map_err(|_| StlError::Allocation)?)?;
        }
        for slot in 0..3 {
            let key = quantized[slot].map(|value| {
                let value = value as f32;
                if value == 0.0 { 0 } else { value.to_bits() }
            });
            match encoded_positions.entry(key) {
                std::collections::hash_map::Entry::Occupied(entry) if entry.get().0 != vertices[slot] => {
                    return Err(StlError::QuantizationCollision { first: entry.get().1, second: triangle[slot] as usize }.into());
                }
                std::collections::hash_map::Entry::Occupied(entry) => {
                    if entry.get().1 != triangle[slot] as usize { report.coincident_distinct_indices = true; }
                }
                std::collections::hash_map::Entry::Vacant(entry) => { entry.insert((vertices[slot], triangle[slot] as usize)); }
            }
        }
        let (a, b, c) = if options.flip_winding {
            (quantized[0], quantized[2], quantized[1])
        } else {
            (quantized[0], quantized[1], quantized[2])
        };
        let normal = normal_of(a, b, c).ok_or(StlError::Degenerate { triangle: index })?;
        for value in normal {
            writer.write_all(&value.to_le_bytes())?;
        }
        for vertex in [a, b, c] {
            for value in vertex {
                writer.write_all(&(value as f32).to_le_bytes())?;
            }
        }
        writer.write_all(&0u16.to_le_bytes())?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_representable_triangle_is_not_mistaken_for_zero_area() {
        let soup = TriangleSoup::new_f64(
            vec![[0., 0., 0.], [1e-16, 0., 0.], [0., 1e-16, 0.]],
            vec![[0, 1, 2]],
        );
        let options = StlOptions { strict: false, ..Default::default() };
        let encoded = write_binary_stl_with_tolerance(&soup, options, Some(1e-20)).unwrap();
        assert_eq!(encoded.len(), 134);
        let read = |offset| f32::from_le_bytes(encoded[offset..offset + 4].try_into().unwrap());
        assert_eq!(read(92), 1.0, "encoded normal must point along +Z");
        assert!(read(108) > 0.0 && read(124) > 0.0, "encoded vertices must remain distinct");
        let collapsed = TriangleSoup::new_f64(vec![[0., 0., 0.]; 3], vec![[0, 1, 2]]);
        assert!(write_binary_stl_with_tolerance(&collapsed, options, Some(1e-20)).is_err());
    }

    #[test]
    fn cancellation_interrupts_vertex_preflight_before_any_output() {
        let soup = TriangleSoup::new_f64(vec![[0., 0., 0.]; 4096], vec![[0, 1, 2]]);
        let mut checks = 0;
        let mut output = Vec::new();
        let result = write_binary_stl_to_with_check(&mut output, &soup, StlOptions::default(), None, || {
            checks += 1;
            if checks == 3 { Err(io::Error::other("canceled")) } else { Ok(()) }
        });
        assert!(matches!(result, Err(StlStreamError::Io(_))));
        assert_eq!(checks, 3);
        assert!(output.is_empty());
    }

    #[test]
    fn cancellation_interrupts_strict_topology_before_any_output() {
        let soup = cube();
        let mut checks = 0;
        let mut output = Vec::new();
        let result = write_binary_stl_to_with_check(&mut output, &soup, StlOptions::default(), None, || {
            checks += 1;
            // Initial check, vertex preflight, index preflight, then topology.
            if checks == 4 { Err(io::Error::other("canceled")) } else { Ok(()) }
        });
        assert!(matches!(result, Err(StlStreamError::Io(_))));
        assert_eq!(checks, 4);
        assert!(output.is_empty());
    }

    #[test]
    fn quantization_cannot_merge_distinct_surfaces_even_when_triangles_survive() {
        let soup = TriangleSoup::new_f64(
            vec![[0., 0., 1.], [1., 0., 1.], [0., 1., 1.],
                 [0., 0., 1.00000002], [1., 0., 1.00000002], [0., 1., 1.00000002]],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let options = StlOptions { strict: false, ..Default::default() };
        assert!(write_binary_stl_with_tolerance(&soup, options, Some(0.0001)).is_err(),
            "distinct faces must not silently become coincident in the encoded STL");
        let identical = TriangleSoup::new_f64(
            vec![[0., 0., 1.], [1., 0., 1.], [0., 1., 1.],
                 [-0., 0., 1.], [1., 0., 1.], [0., 1., 1.]],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert!(write_binary_stl_with_tolerance(&identical, options, Some(0.0001)).is_ok(),
            "pre-existing identical positions, including signed zero, are not new quantization collisions");
    }

    #[test]
    fn stream_reports_io_failure_without_materializing_the_file() {
        struct FailsAfter(usize);
        impl Write for FailsAfter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.0 == 0 { return Err(io::Error::other("simulated disk failure")); }
                let written = self.0.min(bytes.len());
                self.0 -= written;
                Ok(written)
            }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        let mut writer = FailsAfter(90);
        assert!(matches!(write_binary_stl_to(&mut writer, &cube(), StlOptions::default(), None), Err(StlStreamError::Io(_))));
    }

    #[test]
    fn topology_traversal_stops_when_check_requests_cancellation() {
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]; 4_000]);
        let mut checks = 0;
        let result = try_edge_topology_with_check(&soup, || {
            checks += 1;
            if checks == 3 { Err("cancelled") } else { Ok(()) }
        }, || "allocation");
        assert_eq!(result, Err("cancelled"));
        assert_eq!(checks, 3);
    }

    #[test]
    fn fallible_topology_matches_legacy_result() {
        let soup = cube();
        let result = try_edge_topology_with_check(&soup, || Ok::<(), StlError>(()), || StlError::Allocation).unwrap();
        assert_eq!(result, edge_topology(&soup));
    }

    /// A closed unit cube, wound counter-clockwise seen from outside.
    fn cube() -> TriangleSoup {
        let positions = vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ];
        let triangles = vec![
            [0, 2, 1],
            [0, 3, 2], // -Z
            [4, 5, 6],
            [4, 6, 7], // +Z
            [0, 1, 5],
            [0, 5, 4], // -Y
            [2, 3, 7],
            [2, 7, 6], // +Y
            [1, 2, 6],
            [1, 6, 5], // +X
            [0, 4, 7],
            [0, 7, 3], // -X
        ];
        TriangleSoup::new(positions, triangles)
    }

    fn triangles_of(bytes: &[u8]) -> Vec<([f32; 3], [[f32; 3]; 3])> {
        let count = u32::from_le_bytes(bytes[80..84].try_into().expect("count")) as usize;
        let mut out = Vec::with_capacity(count);
        for index in 0..count {
            let start = 84 + index * 50;
            let float = |offset: usize| {
                f32::from_le_bytes(
                    bytes[start + offset..start + offset + 4]
                        .try_into()
                        .expect("float"),
                )
            };
            let normal = [float(0), float(4), float(8)];
            let vertices = [
                [float(12), float(16), float(20)],
                [float(24), float(28), float(32)],
                [float(36), float(40), float(44)],
            ];
            out.push((normal, vertices));
        }
        out
    }

    #[test]
    fn a_closed_cube_writes_the_binary_layout() {
        let soup = cube();
        let bytes = write_binary_stl(&soup, StlOptions::default()).expect("cube exports");
        assert_eq!(bytes.len(), 84 + 12 * 50);
        let count = u32::from_le_bytes(bytes[80..84].try_into().expect("count"));
        assert_eq!(count, 12);
        let triangles = triangles_of(&bytes);
        assert_eq!(triangles.len(), 12);
        for (normal, _) in &triangles {
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            assert!(
                (length - 1.0).abs() < 1e-5,
                "normals must be unit: {length}"
            );
        }
    }

    #[test]
    fn the_normal_points_out_of_the_solid() {
        let soup = cube();
        let bytes = write_binary_stl(&soup, StlOptions::default()).expect("cube exports");
        // The +Z face is the only one whose vertices all have z = 0.5.
        let top = triangles_of(&bytes)
            .into_iter()
            .find(|(_, vertices)| vertices.iter().all(|vertex| vertex[2] > 0.49))
            .expect("top face");
        assert!(
            top.0[2] > 0.99,
            "the +Z face normal must point up: {:?}",
            top.0
        );
    }

    #[test]
    fn flipping_the_winding_reverses_the_normals() {
        let soup = cube();
        let straight = write_binary_stl(&soup, StlOptions::default()).expect("cube exports");
        let flipped = write_binary_stl(
            &soup,
            StlOptions {
                flip_winding: true,
                ..StlOptions::default()
            },
        )
        .expect("cube exports");
        let a = triangles_of(&straight)[0].0;
        let b = triangles_of(&flipped)[0].0;
        assert!(
            (a[0] + b[0]).abs() < 1e-5 && (a[1] + b[1]).abs() < 1e-5 && (a[2] + b[2]).abs() < 1e-5
        );
    }

    #[test]
    fn the_scale_is_applied_to_every_coordinate() {
        let soup = cube();
        let options = StlOptions {
            scale: 1.0 / 25.4,
            ..StlOptions::default()
        };
        let bytes = write_binary_stl(&soup, options).expect("cube exports");
        let (_, vertices) = triangles_of(&bytes)[0];
        let expected = (-0.5f64 / 25.4) as f32;
        assert!(
            (vertices[0][0] - expected).abs() < 1e-6,
            "{:?} vs {expected}",
            vertices[0]
        );
    }

    #[test]
    fn an_open_surface_is_refused_in_strict_mode() {
        let mut soup = cube();
        soup.triangles.truncate(10);
        assert_eq!(open_edges(&soup), 4, "removing one face opens four edges");
        let error = write_binary_stl(&soup, StlOptions::default()).expect_err("must refuse");
        assert_eq!(error, StlError::OpenSurface { open_edges: 4 });
        let bytes = write_binary_stl(
            &soup,
            StlOptions {
                strict: false,
                ..StlOptions::default()
            },
        )
        .expect("lenient export writes it");
        assert_eq!(bytes.len(), 84 + 10 * 50);
    }

    #[test]
    fn a_degenerate_triangle_is_refused() {
        let soup = TriangleSoup::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let error = write_binary_stl(
            &soup,
            StlOptions {
                strict: false,
                ..StlOptions::default()
            },
        )
        .expect_err("no area");
        assert_eq!(error, StlError::Degenerate { triangle: 0 });
    }

    #[test]
    fn bad_input_is_rejected_with_a_specific_reason() {
        let soup = TriangleSoup::new(vec![[0.0, 0.0, 0.0]], vec![[0, 1, 2]]);
        assert_eq!(
            write_binary_stl(&soup, StlOptions::default()),
            Err(StlError::IndexOutOfRange {
                triangle: 0,
                index: 1
            })
        );
        let not_finite = TriangleSoup::new(
            vec![[f32::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(
            write_binary_stl(&not_finite, StlOptions::default()),
            Err(StlError::NonFinite { vertex: 0 })
        );
        assert_eq!(
            write_binary_stl(
                &cube(),
                StlOptions {
                    scale: 0.0,
                    ..StlOptions::default()
                }
            ),
            Err(StlError::BadScale)
        );
        assert_eq!(
            write_binary_stl(
                &TriangleSoup::new(vec![[0.0; 3]], Vec::new()),
                StlOptions::default()
            ),
            Err(StlError::Empty)
        );
    }
}
