// SPDX-License-Identifier: LGPL-2.1-or-later
//! Subobject selection set: faces, edges and vertices kept beside the object selection.
use om9_app::session::Controller;
use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
use om9_scene::scene::Subelement;

/// Real polygon topology; arbitrary keys on a point are not valid subobjects.
fn controller() -> (Controller, om9_core::identity::ObjectId) {
    let mut document = Document::new("subobjects");
    let entity = Entity::new("box", Geometry::Mesh(Mesh {
        vertices: vec![[-1.,-1.,-1.], [1.,-1.,-1.], [1.,1.,-1.], [-1.,1.,-1.], [-1.,-1.,1.], [1.,-1.,1.], [1.,1.,1.], [-1.,1.,1.]],
        faces: vec![vec![0,3,2,1],vec![4,5,6,7],vec![0,1,5,4],vec![1,2,6,5],vec![2,3,7,6],vec![3,0,4,7]],
        normals: vec![], colors: vec![],
    }), None);
    let root = entity.id;
    document.entities.insert(root, entity);
    let controller = Controller::new(document, Limits::default()).expect("controller");
    (controller, root)
}

#[test]
fn subobjects_are_added_in_pick_order_and_duplicates_are_ignored() {
    let (mut controller, root) = controller();
    controller.select_subobject(root, Subelement::MeshVertex(3)).expect("add");
    controller.select_subobject(root, Subelement::MeshFace(2)).expect("add");
    controller.select_subobject(root, Subelement::MeshVertex(3)).expect("duplicate");
    assert_eq!(controller.subselection().len(), 2, "one key per subobject");
    assert_eq!(controller.subselection()[0].subset, Subelement::MeshVertex(3));
    assert_eq!(controller.subselection()[1].subset, Subelement::MeshFace(2));
    assert!(controller.sub_revision() > 1);
}

#[test]
fn toggling_the_same_subobject_removes_it() {
    let (mut controller, root) = controller();
    controller.toggle_subobject(root, Subelement::MeshEdge { a: 1, b: 2 }).expect("add");
    assert_eq!(controller.subselection().len(), 1);
    controller.toggle_subobject(root, Subelement::MeshEdge { a: 1, b: 2 }).expect("remove");
    assert!(controller.subselection().is_empty());
}

#[test]
fn an_unknown_owner_is_refused_and_nothing_is_stored() {
    let (mut controller, _root) = controller();
    let unknown = om9_core::identity::ObjectId::new();
    assert!(controller.select_subobject(unknown, Subelement::MeshFace(1)).is_err());
    assert!(controller.subselection().is_empty());
}

#[test]
fn the_object_selection_is_not_disturbed_by_subobjects() {
    let (mut controller, root) = controller();
    controller.select_objects(&[root]).expect("select object");
    let before = controller.selection_state().objects.clone();
    controller.select_subobject(root, Subelement::MeshFace(4)).expect("subobject");
    assert_eq!(controller.selection_state().objects, before);
    controller.clear_subselection();
    assert!(controller.subselection().is_empty());
    assert_eq!(controller.selection_state().objects, before);
}

#[test]
fn a_subobject_whose_object_is_gone_is_pruned() {
    let (mut controller, root) = controller();
    controller.select_subobject(root, Subelement::MeshFace(0)).expect("add");
    // Simulate a document change that removes the object: delete through one transaction.
    controller.delete_objects(&[root]).expect("delete");
    controller.prune_subselection();
    assert!(
        controller.subselection().is_empty(),
        "no stale subobject key survives its object"
    );
}

#[test]
fn the_geometry_kind_is_checked() {
    let mut document = Document::new("point");
    let entity = Entity::new("point", Geometry::Point([0.;3]), None);
    let root = entity.id;
    document.entities.insert(root, entity);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    assert!(controller.select_subobject(root, Subelement::MeshFace(0)).is_err());
    assert!(controller.select_subobject(root, Subelement::MeshVertex(3)).is_err());
    assert!(controller.subselection().is_empty());
}

#[test]
fn invalid_topology_and_a_tessellation_diagonal_are_rejected() {
    let (mut controller, root) = controller();
    for subset in [Subelement::MeshFace(900), Subelement::MeshVertex(900), Subelement::MeshEdge { a: 0, b: 2 }] {
        assert!(controller.select_subobject(root, subset).is_err());
    }
    assert!(controller.subselection().is_empty());
}

#[test]
fn publication_prunes_subobjects_and_undo_never_rematches_them() {
    let (mut controller, root) = controller();
    controller.select_objects(&[root]).unwrap();
    controller.select_subobject(root, Subelement::MeshFace(2)).unwrap();
    controller.numeric_move([3.,0.,0.]).unwrap();
    assert!(controller.subselection().is_empty(), "changed source invalidates the picked key");
    controller.undo().unwrap();
    assert!(controller.subselection().is_empty(), "Undo must not guess a replacement key");
}
