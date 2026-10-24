#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::{Geometry, Mesh};
use om9_core::model::{Document, Entity};

#[test]
fn mesh_curve_projects_points_onto_the_mesh() {
    let mut doc = Document::new("mesh curve");
    let mesh = Mesh {
        vertices: vec![
            [-20.0, -20.0, 0.0],
            [20.0, -20.0, 0.0],
            [20.0, 20.0, 0.0],
            [-20.0, 20.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        normals: vec![[0.0, 0.0, 1.0]; 4],
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let mesh = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let mut ids = vec![mesh.id];
    doc.entities.insert(mesh.id, mesh);
    for point in [[-5.0, 0.0, 5.0], [0.0, 5.0, 5.0], [5.0, 0.0, 5.0]] {
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Mesh").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Mesh")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    assert!(curve.poles.iter().all(|point| point[2].abs() < 1e-6));
}
