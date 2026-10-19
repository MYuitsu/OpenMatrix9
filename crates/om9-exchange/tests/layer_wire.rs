// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    layers::PresetSlot,
    model::{Definition, Document, Entity, Layer},
    transaction::{TransactionCoordinator, World},
};
use om9_exchange::ExportCapture;
#[test]
fn geometry_wire_projects_hidden_to_visibility_and_keeps_om9_slots_out_of_native_abi() {
    let limits = Limits::default();
    let mut d = Document::new("native visibility projection");
    let layer = Layer::preset(PresetSlot::new(17).unwrap());
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut e = Entity::new("hidden", Geometry::Point([1., 2., 3.]), Some(lid));
    let eid = e.id;
    e.attributes.hidden = true;
    d.entities.insert(eid, e);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    let packet = ExportCapture::all(&c, did)
        .unwrap()
        .encode(&limits)
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(
        json["document"]["entities"][eid.to_string()]["attributes"]["visible"],
        false
    );
    assert!(
        json["document"]["entities"][eid.to_string()]["attributes"]
            .get("hidden")
            .is_none()
    );
    assert!(
        json["document"]["layers"][lid.to_string()]
            .get("preset_slot")
            .is_none()
    );
    assert_eq!(c.snapshot(), before);
}

#[test]
fn definition_hidden_is_projected_without_leaking_new_fields_into_native_geometry_wire() {
    let limits = Limits::default();
    let mut d = Document::new("hidden member native projection");
    let mut member = Entity::new("member", Geometry::Point([1., 2., 3.]), None);
    member.attributes.hidden = true;
    let definition = Definition::new("definition", vec![member]);
    let id = definition.id;
    d.definitions.insert(id, definition);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &ExportCapture::all(&c, did)
            .unwrap()
            .encode(&limits)
            .unwrap(),
    )
    .unwrap();
    let attributes = &json["document"]["definitions"][id.to_string()]["members"][0]["attributes"];
    assert_eq!(attributes["visible"], false);
    assert_eq!(attributes["selectable"], true);
    assert!(attributes.get("hidden").is_none());
    assert!(
        c.world().documents[&did].definitions[&id].members[0]
            .attributes
            .hidden
    );
}
