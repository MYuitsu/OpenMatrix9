// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    layers::{LayerAction, LayerPlan},
    model::{Document, Entity, Layer},
    transaction::*,
};
fn coordinator(d: Document) -> TransactionCoordinator {
    let limits = Limits::default();
    TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap()
}
fn request(
    c: &TransactionCoordinator,
    id: om9_core::identity::DocumentId,
    action: LayerAction,
) -> TransactionRequest {
    TransactionRequest::new(
        "Layers",
        vec![Mutation::Layers(Box::new(
            LayerPlan::capture(c, id, action, None).unwrap(),
        ))],
    )
}
fn apply(c: &mut TransactionCoordinator, id: om9_core::identity::DocumentId, action: LayerAction) {
    let r = request(c, id, action);
    let p = c.prepare(r).unwrap();
    c.commit(p).unwrap();
}
#[test]
fn lazy_create_and_current_are_one_undo_and_retired_uuid_cannot_be_reused() {
    let d = Document::new("owned layers");
    let did = d.id;
    let mut c = coordinator(d);
    let before = c.snapshot();
    let layer = Layer::new("Metal 01", [34, 153, 135, 255]);
    let lid = layer.id;
    apply(
        &mut c,
        did,
        LayerAction::Create {
            layer: layer.clone(),
            make_current: true,
        },
    );
    assert_eq!(c.world().documents[&did].current_layer, Some(lid));
    assert_eq!(c.world().documents[&did].layer_order, vec![lid]);
    assert_eq!(c.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.world(), &before.world);
    let saved = c.snapshot();
    let r = request(
        &c,
        did,
        LayerAction::Create {
            layer,
            make_current: false,
        },
    );
    assert!(c.prepare(r).is_err());
    assert_eq!(c.snapshot(), saved);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].current_layer, Some(lid));
}
#[test]
fn current_only_preserves_existing_entities_and_noop_keeps_executable_redo() {
    let mut d = Document::new("current only");
    let layer = Layer::new("Gem 01", [89, 139, 194, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
    d.entities.insert(e.id, e);
    let did = d.id;
    let entities = d.entities.clone();
    let mut c = coordinator(d);
    apply(&mut c, did, LayerAction::SetCurrent { layer: Some(lid) });
    assert_eq!(c.world().documents[&did].entities, entities);
    c.undo().unwrap();
    let before = c.snapshot();
    apply(&mut c, did, LayerAction::SetCurrent { layer: None });
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].current_layer, Some(lid));
}
#[test]
fn visibility_lock_and_unlock_preserve_explicit_object_preferences() {
    let mut d = Document::new("permissions");
    let layer = Layer::new("User 01", [226, 52, 52, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut e = Entity::new("explicit unselectable", Geometry::Point([0.; 3]), Some(lid));
    e.attributes.selectable = false;
    let eid = e.id;
    let explicit = e.attributes.clone();
    d.entities.insert(eid, e);
    let did = d.id;
    let mut c = coordinator(d);
    for (visible, locked) in [(true, true), (false, true), (false, false), (true, false)] {
        let mut layer = c.world().documents[&did].layers[&lid].clone();
        layer.visible = visible;
        layer.locked = locked;
        apply(&mut c, did, LayerAction::Update { layer });
        let d = &c.world().documents[&did];
        assert_eq!(d.entities[&eid].attributes, explicit);
        let p = d.permissions(eid, false).unwrap();
        assert_eq!(p.visible, visible);
        assert!(!p.selectable);
        assert_eq!(p.reference_snappable, visible && !locked);
        assert_eq!(
            d.permissions(eid, true).unwrap().reference_snappable,
            visible
        );
    }
}
#[test]
fn foreign_coordinator_and_reopened_equal_counter_document_reject_layer_capture() {
    let d = Document::new("owner");
    let did = d.id;
    let mut c = coordinator(d.clone());
    let action = LayerAction::SetCurrent { layer: None };
    let r = request(&c, did, action);
    let second = coordinator(d.clone());
    assert!(second.prepare(r.clone()).is_err());
    let mut reopened = d;
    reopened.renew_session(&Limits::default()).unwrap();
    let second = coordinator(reopened);
    assert!(second.prepare(r.clone()).is_err());
    let other = request(
        &c,
        did,
        LayerAction::Create {
            layer: Layer::new("Lights", [255; 4]),
            make_current: false,
        },
    );
    let p = c.prepare(other).unwrap();
    c.commit(p).unwrap();
    let before = c.snapshot();
    assert!(c.prepare(r).is_err());
    assert_eq!(c.snapshot(), before);
}
#[test]
fn assign_changes_membership_and_color_atomically_without_changing_current_or_placement() {
    let mut d = Document::new("assign");
    let mut from = Layer::new("source off", [20, 30, 40, 255]);
    from.visible = false;
    let target = Layer::new("target", [90, 80, 70, 255]);
    let tid = target.id;
    let fid = from.id;
    d.layer_order.extend([fid, tid]);
    d.layers.insert(fid, from);
    d.layers.insert(tid, target);
    d.current_layer = Some(fid);
    let e = Entity::new("off root", Geometry::Point([2., 3., 4.]), Some(fid));
    let eid = e.id;
    let placement = e.placement;
    d.entities.insert(eid, e);
    let stamp = d.stamp(eid).unwrap();
    let did = d.id;
    let mut c = coordinator(d);
    apply(
        &mut c,
        did,
        LayerAction::Assign {
            layer: tid,
            sources: vec![stamp],
        },
    );
    let d = &c.world().documents[&did];
    assert_eq!(d.current_layer, Some(fid));
    assert_eq!(d.entities[&eid].attributes.layer, Some(tid));
    assert_eq!(d.entities[&eid].attributes.color, None);
    assert_eq!(d.layers[&tid].color, [90, 80, 70, 255]);
    assert_eq!(d.entities[&eid].placement, placement);
    assert_eq!(c.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(
        c.world().documents[&did].entities[&eid].attributes.layer,
        Some(fid)
    );
}
#[test]
fn cycle_locked_current_and_cancel_reject_before_any_publication() {
    let mut d = Document::new("layer rejection");
    let layer = Layer::new("lock", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let did = d.id;
    let mut c = coordinator(d);
    let mut cycle = c.world().documents[&did].layers[&lid].clone();
    cycle.parent = Some(lid);
    let before = c.snapshot();
    assert!(
        c.prepare(request(&c, did, LayerAction::Update { layer: cycle }))
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let mut locked = c.world().documents[&did].layers[&lid].clone();
    locked.locked = true;
    apply(&mut c, did, LayerAction::Update { layer: locked });
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(request(
            &c,
            did,
            LayerAction::SetCurrent { layer: Some(lid) }
        )),
        Err(Error::Permission(_))
    ));
    let r = request(
        &c,
        did,
        LayerAction::Create {
            layer: Layer::new("cancel", [0; 4]),
            make_current: false,
        },
    );
    r.cancel.cancel();
    assert!(matches!(c.prepare(r), Err(Error::Canceled)));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn one_batch_cannot_lock_a_target_then_assign_using_its_old_permission() {
    let mut d = Document::new("permission batch");
    let layer = Layer::new("target", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layers.insert(lid, layer);
    d.layer_order.push(lid);
    let e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
    let eid = e.id;
    d.entities.insert(eid, e);
    let did = d.id;
    let c = coordinator(d);
    let d = &c.world().documents[&did];
    let mut locked = d.layers[&lid].clone();
    locked.locked = true;
    let update = LayerPlan::capture(&c, did, LayerAction::Update { layer: locked }, None).unwrap();
    let assign = LayerPlan::capture(
        &c,
        did,
        LayerAction::Assign {
            layer: lid,
            sources: vec![d.stamp(eid).unwrap()],
        },
        None,
    )
    .unwrap();
    let before = c.snapshot();
    assert!(
        c.prepare(TransactionRequest::new(
            "mixed permissions",
            vec![
                Mutation::Layers(Box::new(update)),
                Mutation::Layers(Box::new(assign))
            ]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
    let mut off = c.world().documents[&did].layers[&lid].clone();
    off.visible = false;
    // An output prepared with explicit target cannot be accepted under an
    // intervening target metadata write in the same batch either.
    let update = LayerPlan::capture(&c, did, LayerAction::Update { layer: off }, None).unwrap();
    let mut output = Entity::new("output", Geometry::Point([4., 5., 6.]), Some(lid));
    output.attributes.locked = false;
    assert!(
        c.prepare(TransactionRequest::new(
            "mixed output",
            vec![
                Mutation::Layers(Box::new(update)),
                Mutation::Insert {
                    document: did,
                    entity: output
                }
            ]
        ))
        .is_err()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn mixed_assignment_locked_source_refuses_whole_batch_and_preserves_redo() {
    let mut d = Document::new("mixed assignment");
    let layer = Layer::new("target", [7, 8, 9, 255]);
    let lid = layer.id;
    d.layers.insert(lid, layer);
    d.layer_order.push(lid);
    let first = Entity::new("first", Geometry::Point([1., 2., 3.]), None);
    let first_id = first.id;
    let mut last = Entity::new("locked last", Geometry::Point([4., 5., 6.]), None);
    last.attributes.locked = true;
    let last_id = last.id;
    d.entities.insert(first_id, first);
    d.entities.insert(last_id, last);
    let did = d.id;
    let mut c = coordinator(d);
    apply(&mut c, did, LayerAction::SetCurrent { layer: Some(lid) });
    c.undo().unwrap();
    let d = &c.world().documents[&did];
    let r = request(
        &c,
        did,
        LayerAction::Assign {
            layer: lid,
            sources: vec![d.stamp(first_id).unwrap(), d.stamp(last_id).unwrap()],
        },
    );
    let before = c.snapshot();
    assert!(matches!(c.prepare(r), Err(Error::Permission(_))));
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].current_layer, Some(lid));
    assert_eq!(
        c.world().documents[&did].entities[&first_id]
            .attributes
            .layer,
        None
    );
}

#[test]
fn caller_savepoint_rollback_retires_created_layer_and_keeps_global_redo() {
    let d = Document::new("caller layers");
    let did = d.id;
    let mut c = coordinator(d);
    apply(
        &mut c,
        did,
        LayerAction::Create {
            layer: Layer::new("global", [1, 2, 3, 255]),
            make_current: false,
        },
    );
    c.undo().unwrap();
    let global = c.world().clone();
    let token = c.book("caller", true).unwrap();
    c.begin(&token).unwrap();
    c.savepoint(&token).unwrap();
    let layer = Layer::new("private", [4, 5, 6, 255]);
    let lid = layer.id;
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer: layer.clone(),
            make_current: true,
        },
        Some(&token),
    )
    .unwrap();
    let mut r = TransactionRequest::new("private layer", vec![Mutation::Layers(Box::new(plan))]);
    r.owner = Some(token.clone());
    let p = c.prepare(r).unwrap();
    assert!(!c.commit(p).unwrap().published);
    assert_eq!(c.world(), &global);
    assert_eq!(
        c.editing_world(&token).unwrap().documents[&did].current_layer,
        Some(lid)
    );
    c.rollback_savepoint(&token).unwrap();
    let before = c.snapshot();
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer,
            make_current: false,
        },
        Some(&token),
    )
    .unwrap();
    let mut r = TransactionRequest::new(
        "retired private layer",
        vec![Mutation::Layers(Box::new(plan))],
    );
    r.owner = Some(token.clone());
    assert!(c.prepare(r).is_err());
    assert_eq!(c.snapshot(), before);
    c.abort(&token).unwrap();
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].layers.len(), 1);
    assert!(!c.world().documents[&did].layers.contains_key(&lid));
}

#[test]
fn layer_revision_overflow_and_new_layer_node_budget_preserve_immediate_world() {
    let mut d = Document::new("layer boundaries");
    let mut layer = Layer::new("limit", [1, 2, 3, 255]);
    layer.revision = u64::MAX;
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let did = d.id;
    let c = coordinator(d);
    let mut layer = c.world().documents[&did].layers[&lid].clone();
    layer.visible = false;
    let before = c.snapshot();
    assert!(matches!(
        c.prepare(request(&c, did, LayerAction::Update { layer })),
        Err(Error::Overflow(_))
    ));
    assert_eq!(c.snapshot(), before);
    let limits = Limits {
        max_nodes: 0,
        ..Limits::default()
    };
    let d = Document::new("no new nodes");
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    let r = request(
        &c,
        did,
        LayerAction::Create {
            layer: Layer::new("new", [1, 2, 3, 255]),
            make_current: false,
        },
    );
    assert!(matches!(c.prepare(r), Err(Error::Budget(_))));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn current_noop_at_full_history_budget_succeeds_and_keeps_prior_redo() {
    let limits = Limits {
        max_history: 1,
        ..Limits::default()
    };
    let d = Document::new("full history noop");
    let did = d.id;
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    apply(
        &mut c,
        did,
        LayerAction::Create {
            layer: Layer::new("one", [1, 2, 3, 255]),
            make_current: true,
        },
    );
    c.undo().unwrap();
    let before = c.snapshot();
    apply(&mut c, did, LayerAction::SetCurrent { layer: None });
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].layers.len(), 1);
}

#[test]
fn oversized_assignment_capture_and_late_cancel_keep_world_and_old_redo() {
    let mut d = Document::new("bounded layer operations");
    let did = d.id;
    let layer = Layer::new("target", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
    let eid = e.id;
    d.entities.insert(eid, e);
    let limits = Limits {
        max_outputs: 2,
        ..Limits::default()
    };
    let mut c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    apply(&mut c, did, LayerAction::SetCurrent { layer: Some(lid) });
    c.undo().unwrap();
    let before = c.snapshot();
    let source = c.world().documents[&did].stamp(eid).unwrap();
    assert!(matches!(
        LayerPlan::capture(
            &c,
            did,
            LayerAction::Assign {
                layer: lid,
                sources: vec![source; 3]
            },
            None
        ),
        Err(Error::Budget(_))
    ));
    assert_eq!(c.snapshot(), before);
    let r = request(&c, did, LayerAction::SetCurrent { layer: Some(lid) });
    let cancel = r.cancel.clone();
    let p = c.prepare(r).unwrap();
    cancel.cancel();
    assert!(matches!(c.commit(p), Err(Error::Canceled)));
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
    assert_eq!(c.world().documents[&did].current_layer, Some(lid));
}

#[test]
fn created_layer_and_its_output_are_one_transaction_and_one_undo() {
    let d = Document::new("create and add");
    let did = d.id;
    let mut c = coordinator(d);
    let before = c.snapshot();
    let layer = Layer::new("Gems", [1, 2, 3, 255]);
    let lid = layer.id;
    let entity = Entity::new("gem", Geometry::Point([1., 2., 3.]), lid);
    let eid = entity.id;
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer,
            make_current: true,
        },
        None,
    )
    .unwrap();
    let r = TransactionRequest::new(
        "create and add",
        vec![
            Mutation::Layers(Box::new(plan)),
            Mutation::Insert {
                document: did,
                entity,
            },
        ],
    );
    let prepared = c.prepare(r).unwrap();
    c.commit(prepared).unwrap();
    let updated = &c.world().documents[&did];
    assert!(updated.layers.contains_key(&lid));
    assert_eq!(updated.current_layer, Some(lid));
    assert_eq!(updated.entities[&eid].attributes.layer, Some(lid));
    assert_eq!(c.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.world(), &before.world);
}

#[test]
fn output_into_a_created_locked_layer_publishes_nothing() {
    let d = Document::new("created locked");
    let did = d.id;
    let c = coordinator(d);
    let before = c.snapshot();
    let mut layer = Layer::new("Locked", [1, 2, 3, 255]);
    layer.locked = true;
    let lid = layer.id;
    let entity = Entity::new("blocked", Geometry::Point([0.; 3]), lid);
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer,
            make_current: false,
        },
        None,
    )
    .unwrap();
    let r = TransactionRequest::new(
        "locked output",
        vec![
            Mutation::Layers(Box::new(plan)),
            Mutation::Insert {
                document: did,
                entity,
            },
        ],
    );
    assert!(matches!(
        c.prepare(r),
        Err(Error::Permission("output layer"))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn output_into_a_created_layer_honours_the_captured_parent_chain() {
    let mut d = Document::new("created child");
    let mut parent = Layer::new("Locked parent", [9, 9, 9, 255]);
    parent.locked = true;
    let pid = parent.id;
    d.layer_order.push(pid);
    d.layers.insert(pid, parent);
    let did = d.id;
    let c = coordinator(d);
    let before = c.snapshot();
    let mut child = Layer::new("Child", [2, 2, 2, 255]);
    child.parent = Some(pid);
    let cid = child.id;
    let entity = Entity::new("blocked", Geometry::Point([0.; 3]), cid);
    let plan = LayerPlan::capture(
        &c,
        did,
        LayerAction::Create {
            layer: child,
            make_current: false,
        },
        None,
    )
    .unwrap();
    let r = TransactionRequest::new(
        "child output",
        vec![
            Mutation::Layers(Box::new(plan)),
            Mutation::Insert {
                document: did,
                entity,
            },
        ],
    );
    assert!(matches!(
        c.prepare(r),
        Err(Error::Permission("output layer"))
    ));
    assert_eq!(c.snapshot(), before);
}

#[test]
fn output_into_a_layer_whose_flags_change_in_the_same_request_is_refused() {
    let mut d = Document::new("stale permission");
    let layer = Layer::new("Gems", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let did = d.id;
    let c = coordinator(d);
    let before = c.snapshot();
    let mut updated = c.world().documents[&did].layers[&lid].clone();
    updated.visible = false;
    let plan = LayerPlan::capture(&c, did, LayerAction::Update { layer: updated }, None).unwrap();
    let entity = Entity::new("gem", Geometry::Point([1., 2., 3.]), lid);
    let r = TransactionRequest::new(
        "stale permission",
        vec![
            Mutation::Layers(Box::new(plan)),
            Mutation::Insert {
                document: did,
                entity,
            },
        ],
    );
    assert!(matches!(
        c.prepare(r),
        Err(Error::Invalid(
            "layer read/write conflict within transaction"
        ))
    ));
    assert_eq!(c.snapshot(), before);
}
