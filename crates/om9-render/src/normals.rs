//! Smooth display normals within CAD faces; never changes the source geometry.
use crate::MeshData;
use std::collections::HashMap;

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn unit(n: [f32; 3]) -> [f32; 3] {
    let length = dot(n, n).sqrt();
    if length.is_finite() && length > 1e-20 {
        n.map(|v| v / length)
    } else {
        [0.; 3]
    }
}

/// Area weighted normals, split at source face boundaries and at sharp creases.
/// Only exactly coincident display vertices within a face share shading. Positions
/// are copied unchanged; duplicate seam vertices can share normals without welding
/// the CAD topology. The 45 degree guard also keeps temporary mesh Boolean edges sharp.
pub fn smooth_faces(positions: Vec<[f32; 3]>, triangles: &[([u32; 3], u32)]) -> MeshData {
    let mut groups: HashMap<([u32; 3], u32), u32> = HashMap::new();
    let mut directions = Vec::<[f32; 3]>::new();
    let mut next = Vec::<u32>::new();
    let mut mesh = MeshData::new(Vec::new(), Vec::with_capacity(triangles.len() * 3));
    for (triangle, face) in triangles {
        let p = triangle.map(|i| *positions.get(i as usize).expect("validated display triangle index"));
        let normal = cross(
            std::array::from_fn(|i| p[1][i] - p[0][i]),
            std::array::from_fn(|i| p[2][i] - p[0][i]),
        );
        let direction = unit(normal);
        for position in p {
            let key = (
                position.map(|v| if v == 0. { 0 } else { v.to_bits() }),
                *face,
            );
            let head = groups.get(&key).copied().unwrap_or(u32::MAX);
            let mut index = head;
            while index != u32::MAX
                && dot(directions[index as usize], direction) < std::f32::consts::FRAC_1_SQRT_2
                && direction != [0.; 3]
                && directions[index as usize] != [0.; 3]
            {
                index = next[index as usize];
            }
            let index = if index != u32::MAX {
                index
            } else {
                let index = mesh.positions.len() as u32;
                mesh.positions.push(position);
                mesh.normals.push([0.; 3]);
                directions.push(direction);
                next.push(head);
                groups.insert(key, index);
                index
            };
            if directions[index as usize] == [0.; 3] {
                directions[index as usize] = direction;
            }
            for axis in 0..3 {
                mesh.normals[index as usize][axis] += normal[axis];
            }
            mesh.indices.push(index);
        }
    }
    for n in &mut mesh.normals {
        *n = unit(*n);
    }
    mesh
}

/// Cofactor transform equals inverse transpose up to a common scale/sign. The
/// shader uses two-sided lighting, so mirrors are supported without dividing by a
/// potentially tiny determinant. A collapsed normal falls back to flat shading.
pub fn transform(normal: [f32; 3], model: [[f32; 4]; 4]) -> [f32; 3] {
    let x = model[0][..3].try_into().unwrap();
    let y = model[1][..3].try_into().unwrap();
    let z = model[2][..3].try_into().unwrap();
    let a = cross(y, z);
    let b = cross(z, x);
    let c = cross(x, y);
    unit(std::array::from_fn(|i| {
        a[i] * normal[0] + b[i] * normal[1] + c[i] * normal[2]
    }))
}
