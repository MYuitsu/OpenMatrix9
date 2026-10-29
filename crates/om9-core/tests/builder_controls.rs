// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder parameter controls (`OM9-L3-03-11-003`, `-005`, `-006`): step quantisation, slider
//! positions, stale panel events and locked inputs.
//!
//! These tests exercise the core session the panel and the viewport handles share: the field
//! domain is the one authority for range, unit and step, so a slider, a typed value and a
//! handle drag all produce the same stored number.
use om9_core::{
    Error,
    builder::{BuilderKind, BuilderSession, register_reference_producers},
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::{Document, Entity, Layer},
    producers::ProducerRegistry,
    request::{Dimension, Value},
    units::LengthMm,
};

fn registry() -> ProducerRegistry {
    let mut registry = ProducerRegistry::new();
    register_reference_producers(&mut registry).unwrap();
    registry
}

fn rail_curve() -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 9.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn profile_curve() -> Geometry {
    Geometry::Curve(Curve::Line {
        start: [9.0, -1.0, 0.0],
        end: [9.0, 1.0, 0.0],
        domain: [0.0, 1.0],
    })
}

/// A document whose two curves live on one unlocked layer.
fn document_with_rail() -> (Document, ObjectId, ObjectId, om9_core::identity::LayerId) {
    let mut document = Document::new("builder-controls");
    let layer = Layer::new("Rails", [200, 180, 120, 255]);
    let layer_id = layer.id;
    document.layers.insert(layer_id, layer);
    document.layer_order.push(layer_id);
    let rail = Entity::new("Rail", rail_curve(), Some(layer_id));
    let profile = Entity::new("Profile", profile_curve(), Some(layer_id));
    let (rail_id, profile_id) = (rail.id, profile.id);
    document.entities.insert(rail_id, rail);
    document.entities.insert(profile_id, profile);
    (document, rail_id, profile_id, layer_id)
}

fn length(value: f64) -> Value {
    Value::length(LengthMm::try_new(value).unwrap())
}

#[test]
fn a_slider_move_lands_exactly_on_the_field_step() {
    let (document, rail, _, _) = document_with_rail();
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    let field = session.field("width_mm").unwrap();
    assert_eq!(field.step(), Some(0.1));
    assert_eq!(field.numeric(), Some(2.0));
    assert!(field.slider_ratio().unwrap() >= 0.0 && field.slider_ratio().unwrap() <= 1.0);

    // 2.0 + 0.46 is not on the step: the stored value is the nearest step, 2.5 mm.
    session.adjust_field("width_mm", 0.46).unwrap();
    let stored = session.field("width_mm").unwrap().numeric().unwrap();
    assert_eq!(stored, 2.5);
    assert_eq!(
        format!("{stored}"),
        "2.5",
        "no binary tail leaks into the field"
    );

    // A slider position means a quantised value as well, so the widget, the model and the
    // preview agree on the same number.
    let ratio = (2.5 - 0.1) / (20.0 - 0.1);
    session.set_slider("width_mm", ratio).unwrap();
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(2.5));

    // A slider at either end stays inside the declared range.
    session.set_slider("width_mm", -3.0).unwrap();
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(0.1));
    session.set_slider("width_mm", 3.0).unwrap();
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(20.0));

    // The preview is computed from the quantised field, not from the raw gesture.
    let registry = registry();
    session.set_slider("width_mm", ratio).unwrap();
    session.refresh(&document, &registry).unwrap();
    assert!(session.preview().unwrap().summary.contains("2.50 x 1.50"));
}

#[test]
fn a_nan_or_negative_value_is_refused_before_it_reaches_the_preview() {
    let (document, rail, _, _) = document_with_rail();
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    assert!(session.set_field("width_mm", length(-1.0)).is_err());
    assert!(session.adjust_field("width_mm", f64::NAN).is_err());
    assert!(session.adjust_field("width_mm", f64::INFINITY).is_err());
    assert!(session.set_slider("width_mm", f64::NAN).is_err());
    // A bare number is still refused: the field is a length with a unit.
    assert!(session.set_field("width_mm", Value::Number(2.5)).is_err());
    // A NaN quantity cannot pass the domain either.
    assert!(
        session
            .set_field(
                "width_mm",
                Value::Quantity {
                    value: f64::NAN,
                    dimension: Dimension::Length,
                },
            )
            .is_err()
    );
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(2.0));
}

#[test]
fn a_panel_event_captured_before_a_handle_drag_is_refused() {
    let (document, _, _, _) = document_with_rail();
    let mut session = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    assert_eq!(session.revision(), 0);
    let painted_at = session.revision();

    // The handle drag wins: it moves the shared field to 20 mm first.
    session.adjust_field("diameter_mm", 2.0).unwrap();
    assert_eq!(session.revision(), 1);
    assert_eq!(session.field("diameter_mm").unwrap().numeric(), Some(20.0));

    // The stale panel event must not overwrite the value the user now sees.
    assert_eq!(
        session
            .set_field_at("diameter_mm", length(18.0), painted_at)
            .unwrap_err(),
        Error::Stale("builder field")
    );
    assert_eq!(session.field("diameter_mm").unwrap().numeric(), Some(20.0));

    // An event painted at the current revision is accepted.
    let painted_at = session.revision();
    session
        .set_field_at("diameter_mm", length(18.0), painted_at)
        .unwrap();
    assert_eq!(session.field("diameter_mm").unwrap().numeric(), Some(18.0));
}

#[test]
fn a_locked_input_paints_no_handle_and_reports_itself_read_only() {
    let (mut document, rail, _, layer) = document_with_rail();
    document.layers.get_mut(&layer).unwrap().locked = true;
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    assert!(!session.editable(&document));
    session.refresh(&document, &registry()).unwrap();
    // The preview still describes the geometry, but it offers no handle the document
    // would refuse to move.
    assert!(session.preview().is_some());
    assert!(session.preview().unwrap().handles.is_empty());

    // Unlocking the layer restores the handles without recreating the session.
    document.layers.get_mut(&layer).unwrap().locked = false;
    assert!(session.editable(&document));
    session.refresh(&document, &registry()).unwrap();
    assert_eq!(session.preview().unwrap().handles.len(), 3);
}

#[test]
fn a_builder_without_inputs_is_always_editable() {
    let (document, _, _, _) = document_with_rail();
    let session = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    // No input is read, so no layer permission can make the ring rail builder read-only.
    assert!(session.editable(&document));
}

#[test]
fn a_typed_input_slot_refuses_the_wrong_geometry_kind_and_keeps_the_entity_id() {
    let (mut document, rail, _profile, layer) = document_with_rail();
    // A surface dropped into the rail slot is refused before any field is shown.
    let surface = Entity::new(
        "Plane",
        Geometry::Surface(
            om9_core::surface_kernel::plane(
                [0.0; 3],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                10.0,
                10.0,
                &om9_core::budget::Limits::default(),
            )
            .unwrap(),
        ),
        Some(layer),
    );
    let surface_id = surface.id;
    document.entities.insert(surface_id, surface);
    assert_eq!(
        BuilderSession::new(BuilderKind::RailProfile, &document, &[surface_id])
            .err()
            .unwrap(),
        Error::Invalid("builder input kind")
    );

    // The accepted slot keeps the document entity id, not a copy or a new identity.
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    assert_eq!(session.inputs(), &[("rail".to_string(), rail)]);
    session.refresh(&document, &registry()).unwrap();
    let plan = session.plan(&document).unwrap();
    let binding = &plan.recipe.inputs[0];
    assert_eq!(binding.role, "rail");
    assert_eq!(binding.source.object, rail);
    assert_ne!(plan.entity.id, rail, "the output is a new entity");
}

#[test]
fn switching_tabs_keeps_each_modes_parameters_in_the_session_layer() {
    // The retainer lives in the controller (`OM9-L3-03-11-002`); the core contract it relies
    // on is that a fresh session starts at its own defaults while an existing one keeps its
    // edited fields, so a tab can restore the exact session it stored.
    let (document, rail, profile, _) = document_with_rail();
    let mut profile_session =
        BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    profile_session.set_field("width_mm", length(3.0)).unwrap();
    let fresh = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    assert_eq!(
        profile_session.field("width_mm").unwrap().numeric(),
        Some(3.0)
    );
    assert_eq!(fresh.field("width_mm").unwrap().numeric(), Some(2.0));

    // Two modes describe different schemas and different inputs.
    let sweep = BuilderSession::new(BuilderKind::RailSweep, &document, &[rail, profile]).unwrap();
    assert_eq!(sweep.kind(), BuilderKind::RailSweep);
    assert_eq!(sweep.inputs().len(), 2);
    assert!(sweep.field("solid").is_some());
    assert!(sweep.field("width_mm").is_none());
}
