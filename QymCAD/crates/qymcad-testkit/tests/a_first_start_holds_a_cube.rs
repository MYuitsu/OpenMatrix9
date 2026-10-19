//! THE DOCUMENT OF A FIRST START IS A CUBE: a part holding a sketch of a square and its extrusion, built through the
//! real kernel - a person opening the program for the first time steps into it and sees how the work goes.
use qymcad_core::model::Project;

#[test]
fn the_sample_of_a_first_start_is_a_cube_of_a_sketch_and_an_extrusion() {
    let mut p = Project::default();
    let part = p.cube_sample();
    let report = qymcad_testkit::open_like_the_app(&mut p);
    assert!(report.errors.is_empty(), "the sample does not build: {:?}", report.errors);
    let parts: Vec<&str> = p.components.iter().filter(|c| c.parent.is_some()).map(|c| c.name.as_str()).collect();
    assert!(parts == ["name-sample-cube"], "a first start holds the parts {parts:?}, not one cube");
    let kinds: Vec<String> = p.timeline.iter().map(|n| format!("{:?}", n.kind).split([' ', '{', '(']).next().unwrap_or_default().to_string()).collect();
    assert!(kinds == ["Sketch", "Extrude"], "the cube is made of {kinds:?}, not a sketch and its extrusion");
    let body = p.timeline.iter().rev().find_map(|n| n.kind.body()).expect("the body of the cube");
    assert!(p.body_owner(body) == Some(part), "the body of the cube stands in no part of it");
    let mi = p.mesh_index(body).expect("the mesh of the cube");
    let v = p.bodies[mi].mesh.volume();
    assert!((v - 8000.0).abs() < 1.0, "the cube of 20 holds {v} mm^3, not 8000");
    assert!(p.active_ctx() == p.root, "a first start does not stand in the assembly");
}
