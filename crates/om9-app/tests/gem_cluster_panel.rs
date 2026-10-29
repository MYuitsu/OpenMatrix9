// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application contract tests for the halo, cluster and baguette builders (`OM9-L2-04-11`).
//!
//! The typed layout math lives in `crates/om9-core/src/gem_cluster.rs`; this file drives the same
//! command surface the icons and the command line use and proves the session wiring: preview,
//! one-transaction Apply, Undo/Redo, child id stability on edit, channel creation, orient and the
//! explicit refusals.

use std::collections::BTreeSet;

use om9_app::session::{
    Controller, InvocationOrigin,
    gem_cluster::{BETWEEN, CHANNEL, CLUSTER, HALO, LayoutKind, ORIENT, TAPERED},
};
use om9_core::{
    budget::Limits,
    gem::{GemId, GemRecord, GemSpec, build_mesh, gem_record_in},
    gem_cluster::ClusterKind,
    geometry::{Curve, Geometry, Mesh, Point3, Transform},
    identity::ObjectId,
    model::{Document, Entity, GEM_EXTENSION_SLOT},
};

fn base_document() -> Document {
    let mut document = Document::new("halo");
    let gem = om9_core::model::Layer::preset(om9_core::layers::PresetSlot::new(5).unwrap());
    let curve = om9_core::model::Layer::preset(om9_core::layers::PresetSlot::new(1).unwrap());
    let curve_id = curve.id;
    document.layer_order = vec![gem.id, curve_id];
    document.layers.insert(gem.id, gem);
    document.layers.insert(curve_id, curve);
    document.current_layer = Some(curve_id);
    document
}

fn add_gem(document: &mut Document, center: Point3) -> ObjectId {
    let spec = GemSpec::round(6.5, 3.95).expect("spec");
    let mesh = build_mesh(&spec).expect("mesh");
    let mut record = GemRecord::new(spec);
    record.gem_id = GemId::new();
    let mut entity = Entity::new("Round", Geometry::Mesh(mesh.mesh), None);
    entity.placement = Transform::translation(center);
    entity.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        record.as_extension().unwrap(),
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn add_line(document: &mut Document, start: Point3, end: Point3) -> ObjectId {
    let curve = Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("wall", Geometry::Curve(curve), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn add_cube(document: &mut Document) -> ObjectId {
    let mesh = Mesh {
        vertices: vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ],
        faces: vec![
            vec![0, 1, 2, 3],
            vec![4, 7, 6, 5],
            vec![0, 4, 5, 1],
            vec![1, 5, 6, 2],
            vec![2, 6, 7, 3],
            vec![3, 7, 4, 0],
        ],
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let entity = Entity::new("object", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn gem_ids(controller: &Controller) -> BTreeSet<ObjectId> {
    controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record_in(controller.document(), entity.id).is_some())
        .map(|entity| entity.id)
        .collect()
}

#[test]
fn the_halo_builder_previews_and_commits_one_transaction() {
    let mut document = base_document();
    let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect("halo opens");
    let session = controller.gem_cluster_session().expect("session");
    assert_eq!(session.preview_count(), session.accents.len());
    assert!(session.preview_count() >= 8);

    let before = controller.document().entities.len();
    controller.submit("").expect("apply");
    assert!(controller.document().entities.len() > before);
    let feature = controller
        .cluster_feature(center)
        .expect("feature")
        .expect("halo feature");
    assert!(matches!(feature.kind, ClusterKind::Halo { .. }));
    assert!(feature.accents.len() >= 8);
    for id in &feature.accents {
        assert!(controller.document().entities.contains_key(&id.object()));
    }
}

#[test]
fn cancel_leaves_the_document_untouched() {
    let mut document = base_document();
    let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect("halo opens");
    let before = controller.document().entities.len();
    assert!(controller.gem_cluster_cancel());
    assert_eq!(controller.document().entities.len(), before);
    assert!(controller.cluster_feature(center).unwrap().is_none());
}

#[test]
fn undoing_a_halo_removes_the_accents_and_the_feature() {
    let mut document = base_document();
    let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect("halo opens");
    let before = controller.document().entities.len();
    controller.submit("").expect("apply");
    let after = controller.document().entities.len();
    assert!(after > before);
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities.len(), before);
    assert!(controller.cluster_feature(center).unwrap().is_none());
    controller.redo().expect("redo");
    assert_eq!(controller.document().entities.len(), after);
}

#[test]
fn editing_a_halo_reuses_the_existing_child_ids() {
    let mut document = base_document();
    let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect("halo opens");
    controller.submit("").expect("apply");
    let first = controller
        .cluster_feature(center)
        .unwrap()
        .unwrap()
        .accents
        .clone();
    assert!(!first.is_empty());
    let child_count = controller.document().entities.len();

    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect("halo edit opens");
    let session = controller.gem_cluster_session().expect("session");
    assert!(session.edit.is_some(), "the builder detected the feature");
    let mut edited = session.clone();
    match &mut edited.kind {
        LayoutKind::Halo(params) => params.spacing_mm = 0.8,
        _ => panic!("a halo session"),
    }
    controller.gem_cluster_set(edited).expect("recompute");
    controller.gem_cluster_apply().expect("apply the edit");
    let after = controller
        .cluster_feature(center)
        .unwrap()
        .unwrap()
        .accents
        .clone();
    let shared: BTreeSet<ObjectId> = first
        .iter()
        .map(|id| id.object())
        .filter(|id| after.iter().any(|next| next.object() == *id))
        .collect();
    assert!(
        shared.len() >= after.len().min(3),
        "editing reused the existing children"
    );
    assert!(controller.document().entities.len() < child_count + first.len());
    controller.undo().expect("undo the edit");
    assert_eq!(
        controller.cluster_feature(center).unwrap().unwrap().accents,
        first
    );
}

#[test]
fn the_cluster_builder_places_a_ring_with_offsets() {
    let mut document = base_document();
    let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[center]).expect("select");
    controller
        .dispatch(CLUSTER, InvocationOrigin::Cmd)
        .expect("cluster opens");
    controller
        .gem_cluster_text("xy=1.0")
        .expect("option")
        .expect("ok");
    controller
        .submit("z=0.5 count=10 start=top")
        .expect("options");
    controller.submit("").expect("apply");
    let feature = controller.cluster_feature(center).unwrap().unwrap();
    assert!(matches!(feature.kind, ClusterKind::Cluster { .. }));
    assert_eq!(feature.accents.len(), 10);
}

#[test]
fn the_baguette_channel_commits_between_two_curves() {
    let mut document = base_document();
    let a = add_line(&mut document, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = add_line(&mut document, [0.0, 3.0, 0.0], [10.0, 3.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[a, b]).expect("select");
    controller
        .dispatch(CHANNEL, InvocationOrigin::Cmd)
        .expect("channel opens");
    let session = controller.gem_cluster_session().expect("session");
    assert!(session.accents.len() >= 2);
    controller.submit("").expect("apply");
    let feature = controller
        .cluster_feature(a)
        .unwrap()
        .or_else(|| controller.cluster_feature(b).unwrap())
        .expect("channel feature");
    assert!(matches!(feature.kind, ClusterKind::Channel { .. }));
    assert!(feature.accents.len() >= 2);
    for id in &feature.accents {
        let gem = gem_record_in(controller.document(), id.object()).unwrap();
        assert!(
            (gem.spec.width_mm - 3.0).abs() < 0.05,
            "the stone touches both walls"
        );
    }
}

#[test]
fn the_tapered_baguette_between_two_curves_is_one_stone() {
    let mut document = base_document();
    let a = add_line(&mut document, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = add_line(&mut document, [0.0, 2.0, 0.0], [10.0, 3.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[a, b]).expect("select");
    controller
        .dispatch(TAPERED, InvocationOrigin::Cmd)
        .expect("taper opens");
    let session = controller.gem_cluster_session().expect("session");
    assert_eq!(session.accents.len(), 1);
    controller.submit("pointa=0.2 pointb=0.8").expect("points");
    controller.submit("").expect("apply");
    let feature = controller
        .cluster_feature(a)
        .unwrap()
        .or_else(|| controller.cluster_feature(b).unwrap())
        .expect("taper feature");
    assert!(matches!(feature.kind, ClusterKind::Channel { .. }));
    assert_eq!(feature.accents.len(), 1);
}

#[test]
fn gems_between_curves_uses_the_round_section_and_touches_both_walls() {
    let mut document = base_document();
    let a = add_line(&mut document, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = add_line(&mut document, [0.0, 1.6, 0.0], [10.0, 1.6, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[a, b]).expect("select");
    controller
        .dispatch(BETWEEN, InvocationOrigin::Cmd)
        .expect("between opens");
    controller.submit("").expect("apply");
    let feature = controller
        .cluster_feature(a)
        .unwrap()
        .or_else(|| controller.cluster_feature(b).unwrap())
        .expect("between feature");
    assert!(feature.accents.len() >= 2);
    for id in &feature.accents {
        let gem = gem_record_in(controller.document(), id.object()).unwrap();
        assert!((gem.spec.width_mm - 1.6).abs() < 0.05);
    }
}

#[test]
fn orient_to_gem_aligns_the_object_and_undo_restores_it() {
    let mut document = base_document();
    let center = add_gem(&mut document, [1.0, 2.0, 3.0]);
    let object = add_cube(&mut document);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    let before = controller.document().entities[&object].placement;
    controller
        .select_objects(&[center, object])
        .expect("select");
    controller
        .dispatch(ORIENT, InvocationOrigin::Cmd)
        .expect("orient opens");
    controller.submit("").expect("apply");
    let placement = controller.document().entities[&object].placement;
    assert_eq!(
        [placement.0[0][3], placement.0[1][3], placement.0[2][3]],
        [1.0, 2.0, 3.0]
    );
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities[&object].placement, before);
}

#[test]
fn typed_command_names_open_the_same_builders() {
    for (name, expected) in [
        ("gvHaloBuilder", HALO),
        ("gvClusterBuilder", CLUSTER),
        ("gvTaperBagChannel", CHANNEL),
        ("gvTaperBagBetweenTwoCurves", TAPERED),
        ("gvGemsBetween2Crvs", BETWEEN),
    ] {
        let mut document = base_document();
        if expected == CHANNEL || expected == TAPERED || expected == BETWEEN {
            let a = add_line(&mut document, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
            let b = add_line(&mut document, [0.0, 2.0, 0.0], [10.0, 2.0, 0.0]);
            let mut controller = Controller::new(document, Limits::default()).expect("controller");
            controller.select_objects(&[a, b]).expect("select");
            controller
                .submit(name)
                .expect("command name opens the builder");
            assert_eq!(
                controller
                    .gem_cluster_session()
                    .map(|session| session.tool.command()),
                Some(expected)
            );
            controller.gem_cluster_cancel();
        } else {
            let center = add_gem(&mut document, [0.0, 0.0, 0.0]);
            let mut controller = Controller::new(document, Limits::default()).expect("controller");
            controller.select_objects(&[center]).expect("select");
            controller
                .submit(name)
                .expect("command name opens the builder");
            assert_eq!(
                controller
                    .gem_cluster_session()
                    .map(|session| session.tool.command()),
                Some(expected)
            );
            controller.gem_cluster_cancel();
        }
    }
}

#[test]
fn a_bad_selection_is_refused_explicitly() {
    let mut document = base_document();
    let a = add_line(&mut document, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    let error = controller
        .dispatch(HALO, InvocationOrigin::Cmd)
        .expect_err("no center gem");
    assert!(format!("{error}").contains("center"));
    controller.select_objects(&[a]).expect("select");
    let error = controller
        .dispatch(CHANNEL, InvocationOrigin::Cmd)
        .expect_err("one wall");
    assert!(format!("{error}").contains("two curves"));
    let _ = gem_ids(&controller);
}
