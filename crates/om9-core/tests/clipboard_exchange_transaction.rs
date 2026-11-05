// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    exchange::{ExchangeMetadata, RenderMaterial},
    identity::MaterialId,
    model::Document,
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

#[test]
fn exchange_insert_rejects_stale_duplicate_and_mismatched_identity_without_publication() {
    let limits = Limits::default();
    let mut doc = Document::new("exchange");
    let original = RenderMaterial {
        id: MaterialId::new(),
        name: "gold".into(),
        diffuse: [1, 2, 3, 255],
    };
    doc.exchange.materials.insert(original.id, original.clone());
    let document = doc.id;
    let revision = doc.revision;
    let c = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let request = |expected_revision, records| {
        TransactionRequest::new(
            "paste",
            vec![Mutation::ExchangeInsert {
                document,
                expected_revision,
                records: Box::new(records),
            }],
        )
    };
    let mut duplicate = ExchangeMetadata::default();
    duplicate.materials.insert(original.id, original.clone());
    assert!(c.prepare(request(revision, duplicate)).is_err());
    let mut mismatched = ExchangeMetadata::default();
    mismatched
        .materials
        .insert(MaterialId::new(), original.clone());
    assert!(c.prepare(request(revision, mismatched)).is_err());
    let mut stale = ExchangeMetadata::default();
    let fresh = RenderMaterial {
        id: MaterialId::new(),
        ..original
    };
    stale.materials.insert(fresh.id, fresh);
    assert!(c.prepare(request(revision + 1, stale)).is_err());
    assert_eq!(c.undo_len(), 0);
    assert_eq!(c.world().documents[&document].exchange.materials.len(), 1);
}

#[test]
fn history_with_pending_input_rechecks_stamp_and_port_before_publication() {
    use om9_core::{
        associative::FeaturePort,
        geometry::{Geometry, GeometryKind, Transform, Unit},
        history::{Recipe, ReplayEnvironment, Tolerance},
        identity::SourceStamp,
        model::Entity,
        request::{Selection, SourceBinding},
    };
    let limits = Limits::default();
    let doc = Document::new("pending history");
    let id = doc.id;
    let input = Entity::new("input", Geometry::Point([0., 0., 0.]), None);
    let output = Entity::new("output", Geometry::Point([1., 0., 0.]), None);
    let stamp = SourceStamp {
        document: id,
        document_epoch: doc.epoch,
        session_generation: doc.session_generation,
        owner_session: doc.owner_session,
        context_revision: doc.context_revision,
        object: input.id,
        generation: 99,
        geometry_revision: 1,
        attributes_revision: 1,
    };
    let recipe = Recipe::new(
        id,
        "fixture",
        vec![SourceBinding {
            role: "source".into(),
            source: stamp,
            selection: Selection::Whole,
        }],
        vec![output.id],
        ReplayEnvironment {
            command_version: 1,
            solver: "fixture".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance::om9_template(),
        },
    )
    .with_ports(
        vec![FeaturePort::geometry("source", &[GeometryKind::Point])],
        vec![FeaturePort::geometry("output", &[GeometryKind::Point])],
    );
    let c = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let request = TransactionRequest::new(
        "paste",
        vec![
            Mutation::Insert {
                document: id,
                entity: input.clone(),
            },
            Mutation::Insert {
                document: id,
                entity: output.clone(),
            },
            Mutation::History {
                document: id,
                expected_revision: 1,
                recipe: Box::new(recipe.clone()),
                published: vec![],
            },
        ],
    );
    assert!(c.prepare(request).is_err());
    let mut wrong_port = recipe;
    wrong_port.inputs[0].source.generation = 1;
    wrong_port.input_ports[0] = FeaturePort::geometry("source", &[GeometryKind::Curve]);
    let request = TransactionRequest::new(
        "paste",
        vec![
            Mutation::Insert {
                document: id,
                entity: input,
            },
            Mutation::Insert {
                document: id,
                entity: output,
            },
            Mutation::History {
                document: id,
                expected_revision: 1,
                recipe: Box::new(wrong_port),
                published: vec![],
            },
        ],
    );
    assert!(c.prepare(request).is_err());
    assert!(c.world().documents[&id].entities.is_empty());
    assert_eq!(c.undo_len(), 0);
}
