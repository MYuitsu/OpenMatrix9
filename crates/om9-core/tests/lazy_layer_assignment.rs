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
