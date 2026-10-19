// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01 wave W5: layer clipboard, exchange and codec ported from the legacy bridge.
//!
//! Oracles are structural: a copy carries the layer subtree and its entities, a paste gives
//! every record a fresh identity and re-parents the subtree, an exchange only changes
//! membership, and the wire form round-trips.
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    identity::LayerId,
    layer_ops::{CLIPBOARD_SCHEMA, copy_layer, decode, encode, exchange_contents, paste_layer},
    model::{Document, Entity, Layer},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};

fn limits() -> Limits {
    Limits::default()
}

struct Fixture {
    document: Document,
    root: LayerId,
    child: LayerId,
}

fn fixture() -> Fixture {
    let mut document = Document::new("layers");
    let root = Layer::new("Jewellery", [10, 20, 30, 255]);
    let root_id = root.id;
    document.layers.insert(root_id, root.clone());
    document.layer_order.push(root_id);
    let mut child = Layer::new("Prongs", [40, 50, 60, 255]);
    child.parent = Some(root_id);
    let child_id = child.id;
    document.layers.insert(child_id, child.clone());
    document.layer_order.push(child_id);
    let mut other = Layer::new("Other", [70, 80, 90, 255]);
    other.parent = None;
    let other_id = other.id;
    document.layers.insert(other_id, other.clone());
    document.layer_order.push(other_id);
    document.current_layer = Some(root_id);
    for (name, layer, point) in [
        ("Ring", root_id, [1.0, 0.0, 0.0]),
        ("Prong A", child_id, [2.0, 0.0, 0.0]),
        ("Loose", other_id, [3.0, 0.0, 0.0]),
    ] {
        let entity = Entity::new(name, Geometry::Point(point), Some(layer));
        document.entities.insert(entity.id, entity);
    }
    Fixture {
        document,
        root: root_id,
        child: child_id,
    }
}

#[test]
fn a_copy_carries_the_layer_subtree_and_only_its_entities() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    assert_eq!(clipboard.schema, CLIPBOARD_SCHEMA);
    assert_eq!(clipboard.source, fixture.root);
    // The layer and its child, but not the unrelated top-level layer.
    assert_eq!(clipboard.layers.len(), 2);
    let ids: Vec<LayerId> = clipboard.layers.iter().map(|layer| layer.id).collect();
    assert!(ids.contains(&fixture.root));
    assert!(ids.contains(&fixture.child));
    // Only the two entities that live on the copied layers.
    assert_eq!(clipboard.entities.len(), 2);
    let labels: Vec<&str> = clipboard
        .entities
        .iter()
        .map(|entity| entity.label.as_str())
        .collect();
    assert!(labels.contains(&"Ring"));
    assert!(labels.contains(&"Prong A"));
    assert!(!labels.contains(&"Loose"));
    // The clipboard is deterministic: entity order is by id.
    let mut sorted = clipboard.entities.clone();
    sorted.sort_by_key(|entity| entity.id);
    assert_eq!(clipboard.entities, sorted);
}

#[test]
fn a_copy_of_an_unknown_layer_is_refused() {
    let fixture = fixture();
    assert_eq!(
        copy_layer(&fixture.document, LayerId::new(), &limits()),
        Err(Error::Missing("layer"))
    );
}

#[test]
fn a_paste_gives_every_record_a_fresh_identity() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let plan = paste_layer(&fixture.document, &clipboard, None, &limits()).unwrap();
    assert_eq!(plan.layers.len(), 2);
    assert_eq!(plan.entities.len(), 2);
    // Layer ids are new; the root stays top-level and the child keeps its new parent chain.
    let original_layers: Vec<LayerId> = clipboard.layers.iter().map(|layer| layer.id).collect();
    for layer in &plan.layers {
        assert!(!original_layers.contains(&layer.id), "layer id was reused");
        assert_eq!(layer.revision, 1);
    }
    let pasted_root = plan
        .layers
        .iter()
        .find(|layer| layer.name == "Jewellery")
        .expect("root pasted");
    assert!(pasted_root.parent.is_none());
    let pasted_child = plan
        .layers
        .iter()
        .find(|layer| layer.name == "Prongs")
        .expect("child pasted");
    assert_eq!(pasted_child.parent, Some(pasted_root.id));
    // Entity ids are new too, and they point at the pasted layers.
    let remap: Vec<LayerId> = plan.layers.iter().map(|layer| layer.id).collect();
    for mutation in &plan.entities {
        let Mutation::Insert { entity, .. } = mutation else {
            panic!("a paste inserts entities");
        };
        assert!(
            !clipboard
                .entities
                .iter()
                .any(|source| source.id == entity.id)
        );
        assert!(
            entity
                .attributes
                .layer
                .is_some_and(|layer| remap.contains(&layer)),
            "pasted entity must point at a pasted layer"
        );
    }
}

#[test]
fn a_paste_can_attach_the_subtree_under_an_existing_layer() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let target = fixture
        .document
        .layers
        .values()
        .find(|layer| layer.name == "Other")
        .unwrap()
        .id;
    let plan = paste_layer(&fixture.document, &clipboard, Some(target), &limits()).unwrap();
    let pasted_root = plan
        .layers
        .iter()
        .find(|layer| layer.name == "Jewellery")
        .unwrap();
    assert_eq!(pasted_root.parent, Some(target));
    assert_eq!(
        paste_layer(
            &fixture.document,
            &clipboard,
            Some(LayerId::new()),
            &limits()
        )
        .err(),
        Some(Error::Missing("layer"))
    );
}

#[test]
fn a_paste_requires_the_layers_to_exist_first() {
    // The entity inserts point at the pasted layers, so committing them before the layer plan
    // created those layers must fail instead of inserting orphans.
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let plan = paste_layer(&fixture.document, &clipboard, None, &limits()).unwrap();
    let coordinator = TransactionCoordinator::new(
        World::new(vec![fixture.document.clone()], &limits()).unwrap(),
        limits(),
    )
    .unwrap();
    // Admission already refuses: the inserts name layers the document does not have.
    let prepared = coordinator.prepare(TransactionRequest::new("paste layer", plan.entities));
    assert_eq!(prepared.err(), Some(Error::Missing("layer")));
}

#[test]
fn a_paste_plan_commits_as_one_undoable_step() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let plan = paste_layer(&fixture.document, &clipboard, None, &limits()).unwrap();
    // Apply the layer half of the plan first (the caller does this through `LayerPlan`).
    let mut with_layers = fixture.document.clone();
    for layer in &plan.layers {
        with_layers.layers.insert(layer.id, layer.clone());
        with_layers.layer_order.push(layer.id);
    }
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![with_layers], &limits()).unwrap(), limits())
            .unwrap();
    let document_id = fixture.document.id;
    let before = coordinator.world().documents[&document_id].entities.len();
    let before_undo = coordinator.undo_len();
    let prepared = coordinator
        .prepare(TransactionRequest::new("paste layer", plan.entities))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    let after = coordinator.world().documents[&document_id].entities.len();
    assert_eq!(after, before + 2);
    assert_eq!(coordinator.undo_len(), before_undo + 1);
    // The originals are untouched.
    assert_eq!(
        coordinator.world().documents[&document_id]
            .entities
            .values()
            .filter(|entity| entity.label == "Ring")
            .count(),
        2,
        "the pasted copy sits beside the original"
    );
}

#[test]
fn an_exchange_only_changes_membership() {
    let fixture = fixture();
    let other = fixture
        .document
        .layers
        .values()
        .find(|layer| layer.name == "Other")
        .unwrap()
        .id;
    let mutations = exchange_contents(&fixture.document, fixture.root, other, &limits()).unwrap();
    assert_eq!(
        mutations.len(),
        1,
        "only the entity on the root layer moves"
    );
    let Mutation::Move { layer, .. } = &mutations[0] else {
        panic!("an exchange moves objects");
    };
    assert_eq!(*layer, Some(other));
    // The layers themselves are untouched by the plan.
    assert!(fixture.document.layers.contains_key(&fixture.root));
    assert_eq!(
        exchange_contents(&fixture.document, fixture.root, fixture.root, &limits()).err(),
        Some(Error::Invalid("layer exchange"))
    );
    assert_eq!(
        exchange_contents(&fixture.document, fixture.root, LayerId::new(), &limits()).err(),
        Some(Error::Missing("layer"))
    );
}

#[test]
fn the_wire_form_round_trips_and_refuses_a_foreign_schema() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let bytes = encode(&clipboard, &limits()).unwrap();
    let decoded = decode(&bytes, &limits()).unwrap();
    assert_eq!(decoded, clipboard);
    // A payload from another schema is refused, not reinterpreted.
    let mut other = clipboard.clone();
    other.schema = CLIPBOARD_SCHEMA + 1;
    let other_bytes = serde_json::to_vec(&other).unwrap();
    assert_eq!(
        decode(&other_bytes, &limits()),
        Err(Error::Unsupported("layer clipboard schema"))
    );
    assert_eq!(
        encode(&other, &limits()),
        Err(Error::Unsupported("layer clipboard schema"))
    );
    // Garbage is an invalid payload.
    assert_eq!(
        decode(b"not a clipboard", &limits()),
        Err(Error::Invalid("layer clipboard wire"))
    );
}

#[test]
fn the_wire_form_respects_the_byte_budget() {
    let fixture = fixture();
    let clipboard = copy_layer(&fixture.document, fixture.root, &limits()).unwrap();
    let tight = Limits {
        max_bytes: 8,
        ..limits()
    };
    assert_eq!(
        encode(&clipboard, &tight),
        Err(Error::Budget("layer clipboard bytes"))
    );
    let bytes = encode(&clipboard, &limits()).unwrap();
    assert_eq!(
        decode(&bytes, &tight),
        Err(Error::Budget("layer clipboard bytes"))
    );
}

#[test]
fn a_copy_of_an_empty_layer_is_an_empty_clipboard() {
    let mut fixture = fixture();
    let empty = Layer::new("Empty", [0, 0, 0, 255]);
    fixture.document.layers.insert(empty.id, empty.clone());
    fixture.document.layer_order.push(empty.id);
    let clipboard = copy_layer(&fixture.document, empty.id, &limits()).unwrap();
    assert_eq!(clipboard.layers.len(), 1);
    assert!(clipboard.entities.is_empty());
    // Pasting an empty layer still re-creates the layer record.
    let plan = paste_layer(&fixture.document, &clipboard, None, &limits()).unwrap();
    assert_eq!(plan.layers.len(), 1);
    assert!(plan.entities.is_empty());
}
