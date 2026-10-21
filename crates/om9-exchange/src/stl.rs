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

/// A triangle soup: positions plus counter-clockwise triangles that index into them.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleSoup {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

impl TriangleSoup {
    pub fn new(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            positions,
            triangles,
        }
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
    /// `strict` was asked for and the surface is not closed.
    OpenSurface { open_edges: usize },
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
            Self::OpenSurface { open_edges } => {
                write!(
                    formatter,
                    "the surface is not watertight: {open_edges} open edges"
                )
            }
            Self::TooManyTriangles { count } => {
                write!(formatter, "binary STL cannot hold {count} triangles")
            }
        }
    }
}

impl std::error::Error for StlError {}

impl From<StlError> for Error {
    fn from(value: StlError) -> Self {
        match value {
            StlError::IndexOutOfRange { .. }
            | StlError::NonFinite { .. }
            | StlError::BadScale
            | StlError::Degenerate { .. } => Error::Invalid("stl geometry"),
            StlError::Empty => Error::Invalid("empty mesh"),
            StlError::OpenSurface { .. } => Error::Invalid("stl surface not watertight"),
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
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if !length.is_finite() || length <= 1e-30 {
        return None;
    }
    Some([
        (n[0] / length) as f32,
        (n[1] / length) as f32,
        (n[2] / length) as f32,
    ])
}

/// Counts edges used by an odd number of triangles: zero means a closed surface.
pub fn open_edges(soup: &TriangleSoup) -> usize {
    use std::collections::HashMap;
    let mut edges: HashMap<(u32, u32), usize> = HashMap::new();
    for triangle in &soup.triangles {
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let key = if a <= b { (a, b) } else { (b, a) };
            *edges.entry(key).or_insert(0) += 1;
        }
    }
    edges.values().filter(|count| **count % 2 != 0).count()
}

/// Writes a binary STL. `strict` refuses open surfaces, degenerate triangles and bad indices; the lenient
/// path still refuses non-finite coordinates because no reader can use them.
pub fn write_binary_stl(
    soup: &TriangleSoup,
    options: StlOptions,
) -> std::result::Result<Vec<u8>, StlError> {
    if soup.triangles.is_empty() {
        return Err(StlError::Empty);
    }
    if !options.scale.is_finite() || options.scale <= 0.0 {
        return Err(StlError::BadScale);
    }
    if soup.triangles.len() > u32::MAX as usize {
        return Err(StlError::TooManyTriangles {
            count: soup.triangles.len(),
        });
    }
    for (index, position) in soup.positions.iter().enumerate() {
        if position.iter().any(|value| !value.is_finite()) {
            return Err(StlError::NonFinite { vertex: index });
        }
    }
    // Indices are checked before the watertight rule so a malformed mesh reports the real defect.
    for (index, triangle) in soup.triangles.iter().enumerate() {
        for vertex_index in triangle {
            if *vertex_index as usize >= soup.positions.len() {
                return Err(StlError::IndexOutOfRange {
                    triangle: index,
                    index: *vertex_index,
                });
            }
        }
    }
    if options.strict {
        let open = open_edges(soup);
        if open > 0 {
            return Err(StlError::OpenSurface { open_edges: open });
        }
    }
    let mut bytes = Vec::with_capacity(84 + soup.triangles.len() * 50);
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
    bytes.extend_from_slice(&header_bytes);
    bytes.extend_from_slice(&(soup.triangles.len() as u32).to_le_bytes());
    for (index, triangle) in soup.triangles.iter().enumerate() {
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
                vertices[slot][axis] = f64::from(position[axis]) * options.scale;
            }
        }
        let (a, b, c) = if options.flip_winding {
            (vertices[0], vertices[2], vertices[1])
        } else {
            (vertices[0], vertices[1], vertices[2])
        };
        let normal = normal_of(a, b, c).ok_or(StlError::Degenerate { triangle: index })?;
        for value in normal {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for vertex in [a, b, c] {
            for value in vertex {
                bytes.extend_from_slice(&(value as f32).to_le_bytes());
            }
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

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
