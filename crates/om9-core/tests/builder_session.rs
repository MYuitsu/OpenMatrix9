// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder session (`OM9-L2-03-11`, `OM9-L2-04-02`): typed fields, live preview, handles and
//! the one-transaction plan the panel applies.
use om9_core::{
    Error,
    budget::Limits,
    builder::{
        BuilderKind, BuilderSession, field_values, producer_name, register_reference_producers,
    },
    geometry::{Curve, Geometry},
    history::EvaluationState,
    model::{Document, Entity},
    producers::ProducerRegistry,
    result::ResultStatus,
    transaction::*,
};

fn limits() -> Limits {
    Limits::default()
}

fn curve(start: [f64; 3], end: [f64; 3]) -> Geometry {
    Geometry::Curve(Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    })
}

fn registry() -> ProducerRegistry {
    let mut registry = ProducerRegistry::new();
    register_reference_producers(&mut registry).unwrap();
    registry
}

fn document_with_rail() -> (
    Document,
    om9_core::identity::ObjectId,
    om9_core::identity::ObjectId,
) {
    let mut document = Document::new("builder");
    let rail = Entity::new(
        "Rail",
        Geometry::Curve(Curve::Circle {
            center: [0.0, 0.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 9.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let profile = Entity::new("Profile", curve([9.0, -1.0, 0.0], [9.0, 1.0, 0.0]), None);
    let (rail_id, profile_id) = (rail.id, profile.id);
    document.entities.insert(rail_id, rail);
    document.entities.insert(profile_id, profile);
    (document, rail_id, profile_id)
}

#[test]
fn a_ring_rail_builder_previews_its_field_and_exposes_a_diameter_handle() {
    let (document, _, _) = document_with_rail();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    assert_eq!(session.title(), "Ring Rail");
    assert_eq!(producer_name(BuilderKind::RingRail), "RingRailProducer");
    assert!(!session.is_dirty());
    // No preview before the panel asks for one.
    assert!(session.plan(&document).is_err());
    session.refresh(&document, &registry).unwrap();
    let preview = session.preview().unwrap();
    let Geometry::Curve(Curve::Circle { radius, .. }) = &preview.geometry else {
        panic!("ring rail preview must be a circle");
    };
    assert_eq!(*radius, 9.0);
    assert!(preview.summary.contains("diameter 18.00"));
    assert_eq!(preview.handles.len(), 1);
    assert_eq!(preview.handles[0].field, "diameter_mm");
    assert_eq!(preview.handles[0].position, [9.0, 0.0, 0.0]);
    // A handle drag and a panel edit use the same field.
    session.adjust_field("diameter_mm", 2.0).unwrap();
    assert!(session.is_dirty());
    session.refresh(&document, &registry).unwrap();
    assert_eq!(session.preview().unwrap().handles[0].value, 20.0);
    let Geometry::Curve(Curve::Circle { radius, .. }) = &session.preview().unwrap().geometry else {
        panic!("circle");
    };
    assert_eq!(*radius, 10.0);
    // A value outside the domain is refused instead of clamped silently.
    assert!(
        session
            .set_field(
                "diameter_mm",
                om9_core::request::Value::length(om9_core::units::LengthMm::try_new(0.5).unwrap())
            )
            .is_err()
    );
    // A drag is clamped to the domain.
    session.adjust_field("diameter_mm", 100.0).unwrap();
    assert_eq!(
        session.field("diameter_mm").unwrap().numeric().unwrap(),
        40.0
    );
    session.reset(&document, &[]).unwrap();
    assert!(!session.is_dirty());
    assert!(session.preview().is_none());
}

#[test]
fn a_rail_profile_builder_reads_the_selected_rail_and_exposes_two_handles() {
    let (document, rail, _) = document_with_rail();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let preview = session.preview().unwrap();
    assert!(matches!(preview.geometry, Geometry::Curve(_)));
    assert_eq!(preview.handles.len(), 3);
    let fields: Vec<&str> = preview.handles.iter().map(|handle| handle.field).collect();
    assert_eq!(fields, vec!["width_mm", "height_mm", "position"]);
    assert!(preview.summary.contains("2.00 x 1.50"));
    // A rail selection is mandatory.
    assert_eq!(
        BuilderSession::new(BuilderKind::RailProfile, &document, &[]).unwrap_err(),
        Error::Invalid("builder selection count")
    );
    // A point is not a rail.
    let mut points = document.clone();
    let point = Entity::new("Point", Geometry::Point([0.0; 3]), None);
    let point_id = point.id;
    points.entities.insert(point_id, point);
    assert_eq!(
        BuilderSession::new(BuilderKind::RailProfile, &points, &[point_id]).unwrap_err(),
        Error::Invalid("builder input kind")
    );
}

#[test]
fn a_rail_sweep_builder_previews_a_surface_and_keeps_the_selection_roles() {
    let (document, rail, profile) = document_with_rail();
    let registry = registry();
    let mut session =
        BuilderSession::new(BuilderKind::RailSweep, &document, &[rail, profile]).unwrap();
    session.refresh(&document, &registry).unwrap();
    assert!(matches!(
        session.preview().unwrap().geometry,
        Geometry::Surface(_)
    ));
    let roles: Vec<&str> = session
        .inputs()
        .iter()
        .map(|(role, _)| role.as_str())
        .collect();
    assert_eq!(roles, vec!["rail", "profile"]);
    // The analytic sweep publishes one face.
    assert_eq!(session.published_subshapes()[0].subshapes[0].name, "face:0");
    assert!(
        BuilderSession::new(BuilderKind::RailSweep, &document, &[rail]).is_err(),
        "two curves are required"
    );
}

#[test]
fn a_committed_builder_plan_inserts_entity_recipe_and_result_in_one_undoable_step() {
    let (document, _, _) = document_with_rail();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let plan = session.plan(&document).unwrap();
    assert_eq!(plan.entity.id, session.output());
    assert_eq!(plan.recipe.id, session.recipe());
    assert_eq!(plan.recipe.command, "RingRail");
    assert_eq!(plan.recipe.state, EvaluationState::Clean);
    assert_eq!(plan.recipe.input_ports.len(), 0);
    assert_eq!(plan.recipe.output_ports.len(), 1);
    assert_eq!(plan.published.status, ResultStatus::Current);
    assert_eq!(plan.published.recipe, Some(plan.recipe.id));
    let output = plan.entity.id;
    let recipe_id = plan.recipe.id;
    let limits = limits();
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let document_id = coordinator
        .world()
        .documents
        .keys()
        .next()
        .copied()
        .unwrap();
    let expected_revision = coordinator.world().documents[&document_id].revision;
    let prepared = coordinator
        .prepare(TransactionRequest::new(
            "RingRail",
            vec![
                Mutation::Insert {
                    document: document_id,
                    entity: plan.entity.clone(),
                },
                Mutation::History {
                    document: document_id,
                    expected_revision,
                    recipe: Box::new(plan.recipe.clone()),
                    published: vec![(output, plan.published.clone())],
                },
            ],
        ))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    let applied = &coordinator.world().documents[&document_id];
    assert!(applied.entities.contains_key(&output));
    assert_eq!(applied.history.recipes[&recipe_id].command, "RingRail");
    assert_eq!(
        applied.results[&output].geometry_snapshot,
        plan.entity.geometry
    );
    coordinator.undo().unwrap();
    let undone = &coordinator.world().documents[&document_id];
    assert!(!undone.entities.contains_key(&output));
    assert!(!undone.history.recipes.contains_key(&recipe_id));
    assert!(!undone.results.contains_key(&output));
    coordinator.redo().unwrap();
    assert!(
        coordinator.world().documents[&document_id]
            .entities
            .contains_key(&output)
    );
}

#[test]
fn the_field_list_is_typed_and_the_panel_never_sees_a_bare_number() {
    let (document, _, _) = document_with_rail();
    let session = BuilderSession::new(
        BuilderKind::RailProfile,
        &document,
        &[document.entities.keys().next().copied().unwrap()],
    )
    .unwrap();
    let values = field_values(&session);
    assert!(matches!(
        values["width_mm"],
        om9_core::request::Value::Quantity {
            dimension: om9_core::request::Dimension::Length,
            ..
        }
    ));
    // A bare number is refused by the field domain, never treated as millimetres.
    let mut session = session;
    assert!(
        session
            .set_field("width_mm", om9_core::request::Value::Number(3.0))
            .is_err()
    );
}

#[test]
fn an_outside_rail_offsets_a_closed_circular_rail_only() {
    let (document, rail, _) = document_with_rail();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::OutsideRail, &document, &[rail]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let Geometry::Curve(Curve::Circle { radius, center, .. }) = &session.preview().unwrap().geometry
    else {
        panic!("outside rail must stay a circle");
    };
    assert_eq!(*radius, 11.0);
    assert_eq!(*center, [0.0, 0.0, 0.0]);
    assert!(session.preview().unwrap().summary.contains("outer radius 11.00"));
    assert_eq!(session.preview().unwrap().handles.len(), 1);
    assert_eq!(session.preview().unwrap().handles[0].field, "offset_mm");
    assert_eq!(session.preview().unwrap().handles[0].value, 2.0);
    // A wider offset is still concentric.
    session.adjust_field("offset_mm", 1.0).unwrap();
    session.refresh(&document, &registry).unwrap();
    let Geometry::Curve(Curve::Circle { radius, .. }) = &session.preview().unwrap().geometry else {
        panic!("circle");
    };
    assert_eq!(*radius, 12.0);
    // A negative offset is outside the field domain and is refused before the producer.
    assert!(
        session
            .set_field(
                "offset_mm",
                om9_core::request::Value::length(
                    om9_core::units::LengthMm::try_new(-1.0).unwrap()
                )
            )
            .is_err()
    );
    // A non-circular rail has no defined outward direction: refused, not approximated.
    let mut line_document = Document::new("line-rail");
    let line = Entity::new("Line", curve([0.0; 3], [10.0, 0.0, 0.0]), None);
    let line_id = line.id;
    line_document.entities.insert(line_id, line);
    let mut line_session =
        BuilderSession::new(BuilderKind::OutsideRail, &line_document, &[line_id]).unwrap();
    assert_eq!(
        line_session.refresh(&line_document, &registry).unwrap_err(),
        Error::Unsupported("outside rail non-circular rail")
    );
}

#[test]
fn a_ring_resizer_creates_an_independent_ring_and_refuses_a_shrink_below_thickness() {
    let (document, rail, _) = document_with_rail();
    let registry = registry();
    let original = document.entities[&rail].geometry.clone();
    let mut session = BuilderSession::new(BuilderKind::RingResizer, &document, &[rail]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let Geometry::Curve(Curve::Circle { radius, .. }) = &session.preview().unwrap().geometry else {
        panic!("resizer output is a ring circle");
    };
    assert_eq!(*radius, 9.5);
    assert!(session.preview().unwrap().summary.contains("inside diameter 19.00"));
    assert_eq!(session.preview().unwrap().handles[0].field, "target_diameter_mm");
    // The source ring is untouched by the preview.
    assert_eq!(document.entities[&rail].geometry, original);
    // The plan is an independent new object with its own typed recipe.
    let plan = session.plan(&document).unwrap();
    assert_eq!(plan.recipe.command, "RingResizer");
    assert_eq!(plan.recipe.input_ports[0].role, "ring");
    assert_ne!(plan.entity.id, rail);
    // Shrinking below twice the band thickness is refused.
    session
        .set_field(
            "thickness_mm",
            om9_core::request::Value::length(om9_core::units::LengthMm::try_new(9.6).unwrap()),
        )
        .unwrap();
    assert_eq!(
        session.refresh(&document, &registry).unwrap_err(),
        Error::Invalid("ring resizer target below thickness")
    );
    // A non-circular ring is unsupported instead of being scaled silently.
    let mut line_document = Document::new("line-ring");
    let line = Entity::new("Ring", curve([0.0; 3], [10.0, 0.0, 0.0]), None);
    let line_id = line.id;
    line_document.entities.insert(line_id, line);
    let mut line_session =
        BuilderSession::new(BuilderKind::RingResizer, &line_document, &[line_id]).unwrap();
    assert_eq!(
        line_session.refresh(&line_document, &registry).unwrap_err(),
        Error::Unsupported("ring resizer non-circular ring")
    );
}

#[test]
fn the_profile_library_switches_shape_and_refuses_a_removed_preset() {
    let (document, rail, _) = document_with_rail();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail]).unwrap();
    session.refresh(&document, &registry).unwrap();
    assert_eq!(session.selected_preset(), Some(om9_core::producers::ProfilePreset::Rectangle));
    assert_eq!(session.presets().len(), 3);
    // Choosing a library shape changes the preview only.
    session.set_preset("circle").unwrap();
    session.refresh(&document, &registry).unwrap();
    assert!(matches!(
        session.preview().unwrap().geometry,
        Geometry::Curve(Curve::Circle { .. })
    ));
    // The rounded rectangle is a closed sampled curve with more than four segments.
    session.set_preset("rounded_rectangle").unwrap();
    session.refresh(&document, &registry).unwrap();
    let Geometry::Curve(Curve::Polycurve { segments, .. }) = &session.preview().unwrap().geometry
    else {
        panic!("rounded rectangle is a polycurve");
    };
    assert!(segments.len() > 4, "segments {}", segments.len());
    // Removing the selected preset drops the preview and refuses both refresh and apply.
    assert!(session.remove_preset("rounded_rectangle").unwrap());
    assert!(session.preview().is_none());
    assert_eq!(
        session.refresh(&document, &registry).unwrap_err(),
        Error::Missing("profile preset")
    );
    assert_eq!(
        session.set_preset("rounded_rectangle").unwrap_err(),
        Error::Missing("profile preset")
    );
    assert!(!session.presets().iter().any(|preset| preset.name() == "rounded_rectangle"));
    assert!(session.plan(&document).is_err());
}
