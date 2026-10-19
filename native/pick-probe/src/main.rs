//! M00-L4-03-03-01 — hit testing on the OCCT tessellation: object, face, edge and vertex.
//!
//! The probe reads the mesh that OCCT produced (`native/occt-rust-proof --bin export_mesh`), builds a
//! camera identical to the GPU probe's, and casts rays through screen points:
//!
//! * **object**: the bounding sphere of the mesh;
//! * **face**: Möller–Trumbore against every triangle, nearest positive hit wins;
//! * **edge / vertex**: distance from the ray to a mesh edge or vertex, accepted within a world-space
//!   tolerance derived from a pixel tolerance (the same rule a viewport uses).
//!
//! Feasibility only, no product code: `native/`.

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;
const FOVY: f32 = 0.7853982; // 45 degrees

#[derive(Clone, Copy)]
struct Vec3([f32; 3]);

impl Vec3 {
    fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
    fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
    fn scale(self, factor: f32) -> Self {
        Self([self.0[0] * factor, self.0[1] * factor, self.0[2] * factor])
    }
    fn dot(self, other: Self) -> f32 {
        self.0[0] * other.0[0] + self.0[1] * other.0[1] + self.0[2] * other.0[2]
    }
    fn cross(self, other: Self) -> Self {
        Self([
            self.0[1] * other.0[2] - self.0[2] * other.0[1],
            self.0[2] * other.0[0] - self.0[0] * other.0[2],
            self.0[0] * other.0[1] - self.0[1] * other.0[0],
        ])
    }
    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn normalized(self) -> Self {
        self.scale(1.0 / self.length().max(f32::MIN_POSITIVE))
    }
}

fn parse_mesh(path: &str) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let text = std::fs::read_to_string(path).expect("mesh file");
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            let mut parts = rest.split_whitespace();
            let x: f32 = parts.next().expect("x").parse().expect("x");
            let y: f32 = parts.next().expect("y").parse().expect("y");
            let z: f32 = parts.next().expect("z").parse().expect("z");
            vertices.push(Vec3([x, y, z]));
        } else if let Some(rest) = line.strip_prefix("f ") {
            let mut parts = rest.split_whitespace();
            let a: u32 = parts.next().expect("a").parse::<u32>().expect("a") - 1;
            let b: u32 = parts.next().expect("b").parse::<u32>().expect("b") - 1;
            let c: u32 = parts.next().expect("c").parse::<u32>().expect("c") - 1;
            faces.push([a, b, c]);
        }
    }
    (vertices, faces)
}

/// A camera looking at `target` from `eye`, matching the GPU probe's perspective and up vector.
struct Camera {
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}

impl Camera {
    fn look_at(eye: Vec3, target: Vec3) -> Self {
        let forward = target.sub(eye).normalized();
        let right = forward.cross(Vec3([0.0, 1.0, 0.0])).normalized();
        let up = right.cross(forward);
        Self { eye, forward, right, up }
    }

    /// Ray through a pixel, in the same convention as the renderer (y down).
    fn ray(&self, screen_x: f32, screen_y: f32) -> (Vec3, Vec3) {
        let ndc_x = 2.0 * screen_x / WIDTH - 1.0;
        let ndc_y = 1.0 - 2.0 * screen_y / HEIGHT;
        let tan = (FOVY * 0.5).tan();
        let aspect = WIDTH / HEIGHT;
        let direction = self
            .forward
            .add(self.right.scale(ndc_x * tan * aspect))
            .add(self.up.scale(ndc_y * tan))
            .normalized();
        (self.eye, direction)
    }

    /// World units per pixel at a given depth, for turning a pixel tolerance into a world tolerance.
    fn world_per_pixel(&self, depth: f32) -> f32 {
        2.0 * (FOVY * 0.5).tan() * depth / HEIGHT
    }
}

fn ray_triangle(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let edge1 = b.sub(a);
    let edge2 = c.sub(a);
    let pvec = direction.cross(edge2);
    let determinant = edge1.dot(pvec);
    if determinant.abs() < 1e-8 {
        return None;
    }
    let inverse = 1.0 / determinant;
    let tvec = origin.sub(a);
    let u = tvec.dot(pvec) * inverse;
    if !(-1e-6..=1.000001).contains(&u) {
        return None;
    }
    let qvec = tvec.cross(edge1);
    let v = direction.dot(qvec) * inverse;
    if v < -1e-6 || u + v > 1.000001 {
        return None;
    }
    let t = edge2.dot(qvec) * inverse;
    (t > 0.0).then_some(t)
}

fn ray_point_distance(origin: Vec3, direction: Vec3, point: Vec3) -> f32 {
    let to_point = point.sub(origin);
    let along = to_point.dot(direction).max(0.0);
    to_point.sub(direction.scale(along)).length()
}

fn ray_segment_distance(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3) -> f32 {
    // Closest approach between the ray and the segment, sampled at both ends and refined by the
    // standard two-line formula.
    let u = direction;
    let v = b.sub(a);
    let w = origin.sub(a);
    let a_dot = u.dot(u);
    let b_dot = u.dot(v);
    let c_dot = v.dot(v);
    let d_dot = u.dot(w);
    let e_dot = v.dot(w);
    let denominator = a_dot * c_dot - b_dot * b_dot;
    let (mut ray_t, mut segment_s) = if denominator.abs() < 1e-9 {
        (0.0, if c_dot.abs() < 1e-9 { 0.0 } else { (e_dot / c_dot).clamp(0.0, 1.0) })
    } else {
        let t = (b_dot * e_dot - c_dot * d_dot) / denominator;
        let s = (a_dot * e_dot - b_dot * d_dot) / denominator;
        (t.max(0.0), s.clamp(0.0, 1.0))
    };
    if ray_t < 0.0 {
        ray_t = 0.0;
    }
    let ray_point = origin.add(u.scale(ray_t));
    let segment_point = a.add(v.scale(segment_s));
    ray_point.sub(segment_point).length()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "mesh.txt".to_string());
    let (vertices, faces) = parse_mesh(&path);
    let mut checks = Vec::new();
    let mut failures = 0usize;

    // The camera the GPU probe renders with: eye (0,-2,14), looking at the origin.
    let camera = Camera::look_at(Vec3([0.0, -2.0, 14.0]), Vec3([0.0, 0.0, 0.0]));

    let mut push = |name: &str, expected: bool, hit: bool, detail: String| {
        let pass = expected == hit;
        if !pass {
            failures += 1;
        }
        checks.push(format!(
            "{{\"name\": \"{name}\", \"expected_hit\": {expected}, \"hit\": {hit}, \"pass\": {pass}, {detail}}}"
        ));
    };

    // --- object: bounding sphere ------------------------------------------
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for vertex in &vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex.0[axis]);
            max[axis] = max[axis].max(vertex.0[axis]);
        }
    }
    let centre = Vec3([
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ]);
    let radius = vertex_max_distance(&vertices, centre);
    let (origin, direction) = camera.ray(WIDTH * 0.5, HEIGHT * 0.5);
    let to_centre = centre.sub(origin);
    let along = to_centre.dot(direction);
    let perpendicular = to_centre.sub(direction.scale(along)).length();
    push(
        "object_centre_ray",
        true,
        perpendicular <= radius,
        format!("\"radius\": {radius:.4}, \"perpendicular\": {perpendicular:.4}"),
    );
    let (corner_origin, corner_direction) = camera.ray(WIDTH - 1.0, HEIGHT - 1.0);
    let to_centre = centre.sub(corner_origin);
    let along = to_centre.dot(corner_direction);
    let perpendicular = to_centre.sub(corner_direction.scale(along)).length();
    push(
        "object_corner_ray",
        false,
        perpendicular <= radius,
        format!("\"perpendicular\": {perpendicular:.4}"),
    );

    // --- face: nearest triangle -------------------------------------------
    let centre_ray = camera.ray(WIDTH * 0.5, HEIGHT * 0.5);
    let mut nearest: Option<(f32, usize)> = None;
    for (index, face) in faces.iter().enumerate() {
        let a = vertices[face[0] as usize];
        let b = vertices[face[1] as usize];
        let c = vertices[face[2] as usize];
        if let Some(t) = ray_triangle(centre_ray.0, centre_ray.1, a, b, c) {
            if nearest.map(|(best, _)| t < best).unwrap_or(true) {
                nearest = Some((t, index));
            }
        }
    }
    match nearest {
        Some((t, index)) => {
            let expected_distance = 14.0 - 5.0; // eye z minus sphere radius, along a near-centre ray
            let close = (t - expected_distance).abs() < 0.5;
            push(
                "face_centre_ray_hits_front",
                true,
                close,
                format!("\"triangle\": {index}, \"t\": {t:.4}, \"expected_t\": {expected_distance:.4}"),
            );
        }
        None => push("face_centre_ray_hits_front", true, false, "\"triangle\": null".to_string()),
    }
    let corner_ray = camera.ray(WIDTH - 1.0, HEIGHT - 1.0);
    let mut corner_hit: Option<(f32, usize)> = None;
    for (index, face) in faces.iter().enumerate() {
        let a = vertices[face[0] as usize];
        let b = vertices[face[1] as usize];
        let c = vertices[face[2] as usize];
        if let Some(t) = ray_triangle(corner_ray.0, corner_ray.1, a, b, c) {
            if corner_hit.map(|(best, _)| t < best).unwrap_or(true) {
                corner_hit = Some((t, index));
            }
        }
    }
    push(
        "face_corner_ray_misses",
        false,
        corner_hit.is_some(),
        format!("\"triangle\": {}", corner_hit.map(|(_, index)| index.to_string()).unwrap_or_else(|| "null".to_string())),
    );

    // --- vertex and edge picking with a pixel tolerance -------------------
    let first_face = faces[0];
    let first_vertex_index = first_face[0];
    let vertex = vertices[first_vertex_index as usize];
    let vertex_camera = Camera::look_at(vertex.add(Vec3([0.0, -2.0, 9.0])), vertex);
    let (origin, direction) = vertex_camera.ray(WIDTH * 0.5, HEIGHT * 0.5);
    let mut nearest_vertex: Option<(f32, usize)> = None;
    for (index, candidate) in vertices.iter().enumerate() {
        let distance = ray_point_distance(origin, direction, *candidate);
        if nearest_vertex.map(|(best, _)| distance < best).unwrap_or(true) {
            nearest_vertex = Some((distance, index));
        }
    }
    let tolerance_pixels = 2.0f32;
    let tolerance_world = vertex_camera.world_per_pixel(9.0) * tolerance_pixels;
    let (distance, index) = nearest_vertex.expect("vertex");
    push(
        "vertex_pick_with_tolerance",
        true,
        distance <= tolerance_world && index == first_vertex_index as usize,
        format!("\"vertex\": {index}, \"distance\": {distance:.6}, \"tolerance_world\": {tolerance_world:.6}"),
    );

    let edge_a = vertices[first_face[0] as usize];
    let edge_b = vertices[first_face[1] as usize];
    let midpoint = edge_a.add(edge_b).scale(0.5);
    let edge_camera = Camera::look_at(midpoint.add(Vec3([0.0, -2.0, 9.0])), midpoint);
    let (origin, direction) = edge_camera.ray(WIDTH * 0.5, HEIGHT * 0.5);
    let mut nearest_edge: Option<(f32, usize, usize)> = None;
    for face in &faces {
        for pair in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
            let a = vertices[pair[0] as usize];
            let b = vertices[pair[1] as usize];
            let distance = ray_segment_distance(origin, direction, a, b);
            if nearest_edge.map(|(best, _, _)| distance < best).unwrap_or(true) {
                nearest_edge = Some((distance, pair[0] as usize, pair[1] as usize));
            }
        }
    }
    let tolerance_world = edge_camera.world_per_pixel(9.0) * tolerance_pixels;
    let (distance, from, to) = nearest_edge.expect("edge");
    let same_edge = [from, to].contains(&(first_face[0] as usize)) && [from, to].contains(&(first_face[1] as usize));
    push(
        "edge_pick_with_tolerance",
        true,
        distance <= tolerance_world && same_edge,
        format!("\"edge\": [{from}, {to}], \"distance\": {distance:.6}, \"tolerance_world\": {tolerance_world:.6}"),
    );

    // A ray 20 px off the vertex must miss once the tolerance is 2 px.
    let (offset_origin, offset_direction) = vertex_camera.ray(WIDTH * 0.5 + 20.0, HEIGHT * 0.5);
    let mut nearest_vertex: Option<(f32, usize)> = None;
    for (index, candidate) in vertices.iter().enumerate() {
        let distance = ray_point_distance(offset_origin, offset_direction, *candidate);
        if nearest_vertex.map(|(best, _)| distance < best).unwrap_or(true) {
            nearest_vertex = Some((distance, index));
        }
    }
    let (distance, index) = nearest_vertex.expect("vertex");
    push(
        "vertex_pick_respects_tolerance",
        false,
        distance <= tolerance_world,
        format!("\"vertex\": {index}, \"distance\": {distance:.6}, \"tolerance_world\": {tolerance_world:.6}"),
    );

    println!(
        "{{\"mesh\": {{\"path\": \"{}\", \"vertices\": {}, \"triangles\": {}}}, \"checks\": [{}], \"failures\": {}, \"decision\": \"{}\"}}",
        path.replace('\\', "/"),
        vertices.len(),
        faces.len(),
        checks.join(", "),
        failures,
        if failures == 0 { "VIABLE" } else { "NOT_VIABLE" }
    );
}

fn vertex_max_distance(vertices: &[Vec3], centre: Vec3) -> f32 {
    vertices
        .iter()
        .map(|vertex| vertex.sub(centre).length())
        .fold(0.0f32, f32::max)
}
