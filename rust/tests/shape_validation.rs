use openmatrix9_rust::shape_validation::{om9_shape_validation as inspect, Input, Summary};
fn solid() -> Input {
    Input { root_kind: 2, native_valid: 2, has_faces: 1, edge_manifold: 1,
        vertex_manifold: 1, edge_closed: 1, face_oriented: 1,
        native_orientation: 1, shell_count: 1, ..Input::default() }
}
#[test]
fn solid_requires_independent_valid_closed_oriented_manifold_facts() {
    let s=inspect(solid(),1);
    assert_eq!((s.status,s.native_valid,s.manifold,s.closed,s.valid_solid),(0,2,2,2,2));
    for field in 0..5 {
        let mut i=solid();
        match field {0=>i.native_valid=1,1=>i.edge_manifold=0,2=>i.vertex_manifold=0,
            3=>i.edge_closed=0,_=>i.face_oriented=0}
        let s=inspect(i,1);assert_eq!(s.valid_solid,1);assert_ne!(s.reasons,0);
    }
}
#[test]
fn uncertainty_stays_unknown_and_known_negative_wins() {
    let mut i=solid();i.native_valid=0;
    assert_eq!(inspect(i,1).valid_solid,0);
    i.edge_closed=0;assert_eq!(inspect(i,1).valid_solid,1);
    let mut i=solid();i.native_orientation=0;
    assert_eq!(inspect(i,1).valid_solid,0);
    i.root_kind=3;assert_eq!(inspect(i,1).valid_solid,1);
}
#[test]
fn loose_and_repeated_geometry_cannot_certify_one_surface_boundary() {
    let mut i=solid();i.loose_geometry=1;
    let s=inspect(i,1);assert_eq!((s.manifold,s.closed,s.valid_solid),(1,1,1));
    let mut i=solid();i.duplicate_faces=1;
    let s=inspect(i,1);assert_eq!((s.manifold,s.closed,s.valid_solid),(0,0,0));
    i.root_kind=0;i.native_orientation=0;assert_eq!(inspect(i,1).valid_solid,1);
}
#[test]
fn orientation_is_recomputed_from_native_even_after_repeated_reframes() {
    for orientation in [1,2] {
        let mut i=solid();i.native_orientation=orientation;
        assert_eq!(inspect(i,1).world_orientation,orientation);
        assert_eq!(inspect(i,-1).world_orientation,3-orientation);
        assert_eq!(inspect(i,-1).world_orientation,3-orientation);
        assert_eq!(inspect(i,1).world_orientation,orientation);
    }
    let mut i=solid();i.native_valid=1;
    assert_eq!(inspect(i,-1).world_orientation,0);
}
#[test]
fn cavities_use_native_validity_and_lower_dimensions_are_not_surfaces() {
    let mut i=solid();i.shell_count=2;assert_eq!(inspect(i,1).valid_solid,2);
    i.native_valid=1;assert_eq!(inspect(i,1).valid_solid,1);
    let s=inspect(Input{root_kind:6,native_valid:2,..Input::default()},1);
    assert_eq!((s.manifold,s.closed,s.valid_solid),(0,0,1));
}
#[test]
fn invalid_scalar_contract_returns_no_partial_facts() {
    assert_eq!((std::mem::size_of::<Input>(),std::mem::align_of::<Input>()),(44,4));
    assert_eq!((std::mem::size_of::<Summary>(),std::mem::align_of::<Summary>()),(32,4));
    for invalid in [Input{root_kind:8,..solid()},Input{native_valid:3,..solid()},
        Input{has_faces:2,..solid()},Input{native_orientation:3,..solid()},
        Input{shell_count:16385,..solid()},Input{root_kind:4,..solid()}] {
        assert_eq!(inspect(invalid,1),Summary{status:1,..Summary::default()});
    }
    for sign in [0,2,-2] {assert_eq!(inspect(solid(),sign).status,1);}
}
#[test]
fn scalar_orientation_reframe_rejects_invalid_codes_without_native_storage() {
    use openmatrix9_rust::shape_validation::om9_shape_orientation as frame;
    assert_eq!((frame(0,1),frame(0,-1)),(0,0));
    assert_eq!((frame(1,1),frame(1,-1),frame(2,1),frame(2,-1)),(1,2,2,1));
    for (code,sign) in [(3,1),(u32::MAX,-1),(1,0),(0,2)] {assert_eq!(frame(code,sign),3);}
}
