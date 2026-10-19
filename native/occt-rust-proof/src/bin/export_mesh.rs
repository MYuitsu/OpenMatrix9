//! M00-L4-03-02-01 — export the OCCT tessellation so the GPU probe can upload exactly that mesh.
use om9_occt_rust_proof::sphere_mesh_dump;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "mesh.txt".to_string());
    let (vertices, triangles) = sphere_mesh_dump(5.0, 0.01, &path).expect("dump");
    println!(
        "{{\"path\": \"{}\", \"vertices\": {}, \"triangles\": {}, \"axis\": \"OCCT 8.0.1 BRepMesh_IncrementalMesh\"}}",
        path, vertices, triangles
    );
}
