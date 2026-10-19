// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::{Limits, Usage},
    geometry::Geometry,
    model::{Definition, Document, Entity},
    transaction::{TransactionCoordinator, World},
};
use om9_scene::scene::Scene;
use std::sync::Arc;

#[test]
fn empty_shared_instances_charge_each_occurrence_before_descent() {
    let mut document = Document::new("empty shared instance expansion");
    let empty = Definition::new("empty", vec![]);
    let mut definition_id = empty.id;
    document.definitions.insert(empty.id, empty);
    for level in 0..7 {
        let definition = Definition::new(
            format!("level {level}"),
            (0..2)
                .map(|branch| {
                    Entity::new(
                        format!("branch {branch}"),
                        Geometry::Instance(definition_id),
                        None,
                    )
                })
                .collect(),
        );
        definition_id = definition.id;
        document.definitions.insert(definition.id, definition);
    }
    for index in 0..100 {
        let root = Entity::new(
            format!("root {index}"),
            Geometry::Instance(definition_id),
            None,
        );
        document.entities.insert(root.id, root);
    }
    let limits = Limits {
        max_nodes: 2_000,
        max_bytes: 10_000_000,
        ..Limits::default()
    };
    // Catalog validation counts shared definitions once; scene expansion must
    // also bound 25,500 occurrences even though none has drawable geometry.
    assert!(document.validate(&limits).unwrap().nodes < limits.max_nodes);
    assert!(matches!(
        Scene::capture(Arc::new(document.clone()), 1, &limits),
        Err(Error::Budget(_))
    ));
    let coordinator =
        TransactionCoordinator::new(World::new(vec![document.clone()], &limits).unwrap(), limits)
            .unwrap();
    assert!(matches!(
        Scene::capture_in_world(&coordinator, document.id, Usage::default()),
        Err(Error::Budget(_))
    ));
    let generous = Limits {
        max_nodes: 30_000,
        ..limits
    };
    assert!(
        Scene::capture(Arc::new(document), 1, &generous)
            .unwrap()
            .objects
            .is_empty()
    );
}
