// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    layers::{LayerAction, LayerPlan, PresetSlot},
    model::{Document, Entity, Layer},
    transaction::*,
};
#[test]
fn create_and_assign_new_preset_is_one_atomic_undo_in_either_request_order() {
    for reverse in [false, true] {
        let limits = Limits::default();
        let mut d = Document::new("lazy target assignment");
        let e = Entity::new("root", Geometry::Point([1., 2., 3.]), None);
        let eid = e.id;
        d.entities.insert(eid, e);
        let did = d.id;
        let source = d.stamp(eid).unwrap();
        let layer = Layer::preset(PresetSlot::new(1).unwrap());
        let lid = layer.id;
        let mut c =
            TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
        let before = c.snapshot();
        let mut mutations = vec![
            Mutation::Layers(Box::new(
                LayerPlan::capture(
                    &c,
                    did,
                    LayerAction::Create {
                        layer,
                        make_current: false,
                    },
                    None,
                )
                .unwrap(),
            )),
            Mutation::Layers(Box::new(
                LayerPlan::capture(
                    &c,
                    did,
                    LayerAction::Assign {
                        layer: lid,
                        sources: vec![source],
                    },
                    None,
                )
                .unwrap(),
            )),
        ];
        if reverse {
            mutations.reverse();
        }
        let p = c
            .prepare(TransactionRequest::new("lazy Assign", mutations))
            .unwrap();
        c.commit(p).unwrap();
        assert_eq!(
            c.world().documents[&did].entities[&eid].attributes.layer,
            Some(lid)
        );
        assert_eq!(c.world().documents[&did].current_layer, None);
        assert_eq!(c.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.world(), &before.world);
        c.redo().unwrap();
        assert_eq!(
            c.world().documents[&did].entities[&eid].attributes.layer,
            Some(lid)
        );
    }
}

#[test]
fn created_layer_ancestry_rejects_cycles_and_locked_output_atomically() {
    for cycle in [false, true] {
        let limits = Limits::default();
        let document = Document::new("new layer ancestry");
        let did = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut parent = Layer::new("parent", [1, 2, 3, 255]);
        let mut child = Layer::new("child", [4, 5, 6, 255]);
        child.parent = Some(parent.id);
        if cycle { parent.parent = Some(child.id); } else { parent.locked = true; }
        let entity = Entity::new("output", Geometry::Point([1., 2., 3.]), Some(child.id));
        let mutations = vec![parent, child].into_iter().map(|layer| {
            Mutation::Layers(Box::new(LayerPlan::capture(&coordinator, did,
                LayerAction::Create { layer, make_current: false }, None).unwrap()))
        }).chain(std::iter::once(Mutation::Insert { document: did, entity })).collect();
        let before = coordinator.snapshot();
        assert!(coordinator.prepare(TransactionRequest::new("invalid layers", mutations)).is_err());
        assert_eq!(coordinator.world(), &before.world);
        assert_eq!(coordinator.undo_len(), 0);
    }
}
