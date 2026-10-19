// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L3-01-08-003 document tolerance policy.
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    history::Tolerance,
    identity::*,
    model::{Document, Entity, Layer},
    transaction::*,
    units::LengthMm,
};

fn document() -> (Document, LayerId) {
    let mut doc = Document::new("tolerance");
    let layer = Layer::new("Metals", [1, 2, 3, 255]);
    let id = layer.id;
    doc.layers.insert(id, layer);
    doc.layer_order.push(id);
    doc.current_layer = Some(id);
    (doc, id)
}

fn coordinator(doc: Document) -> (TransactionCoordinator, DocumentId) {
    let id = doc.id;
    (
        TransactionCoordinator::new(
            World::new(vec![doc], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        id,
    )
}

#[test]
fn l3_003_a_tolerance_names_the_field_it_refuses() {
    assert_eq!(
        Tolerance::try_new(0., 1e-6, 0.),
        Err(Error::Invalid("absolute tolerance"))
    );
    assert_eq!(
        Tolerance::try_new(f64::NAN, 1e-6, 0.),
        Err(Error::Invalid("absolute tolerance"))
    );
    assert_eq!(
        Tolerance::try_new(0.001, -1e-6, 0.),
        Err(Error::Invalid("relative tolerance"))
    );
    assert_eq!(
        Tolerance::try_new(0.001, 1e-6, f64::INFINITY),
        Err(Error::Invalid("angular tolerance"))
    );
    assert_eq!(
        Tolerance::try_new(0.001, 1e-6, 0.).unwrap().absolute_mm,
        0.001
    );
}

#[test]
fn l3_003_geometry_at_or_below_the_absolute_tolerance_is_refused() {
    let tolerance = Tolerance::try_new(0.001, 1e-6, 0.).unwrap();
    assert_eq!(
        tolerance.admit_feature(LengthMm::try_new(0.001).unwrap()),
        Err(Error::Invalid(
            "feature at or below document absolute tolerance"
        ))
    );
    assert!(
        tolerance
            .admit_feature(LengthMm::try_new(0.01).unwrap())
            .is_ok()
    );
}

#[test]
fn l3_003_a_document_without_a_tolerance_reports_missing_not_a_default() {
    let (doc, _layer) = document();
    assert!(doc.tolerance().is_none());
    assert_eq!(
        doc.effective_tolerance(),
        Err(Error::Missing("document tolerance"))
    );
}

#[test]
fn l3_003_the_tolerance_mutation_is_one_undo_entry_and_keeps_context_revision() {
    let (doc, layer) = document();
    let (mut c, d) = coordinator(doc);
    let entity = Entity::new("A", Geometry::Point([0.; 3]), Some(layer));
    let id = entity.id;
    let p = c
        .prepare(TransactionRequest::new(
            "insert",
            vec![Mutation::Insert {
                document: d,
                entity,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let before_revision = c.world().documents[&d].revision;
    let before_context = c.world().documents[&d].context_revision;
    let before_undo = c.undo_len();
    let tolerance = Tolerance::try_new(0.001, 1e-6, 0.).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "tolerance",
            vec![Mutation::Tolerance {
                document: d,
                expected_revision: before_revision,
                tolerance,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let doc = &c.world().documents[&d];
    assert_eq!(doc.revision, before_revision + 1);
    assert_eq!(
        doc.context_revision, before_context,
        "a tolerance is not a unit-context change: stamps stay valid"
    );
    assert_eq!(c.undo_len(), before_undo + 1);
    assert_eq!(doc.effective_tolerance().unwrap().absolute_mm, 0.001);
    assert!(doc.entities.contains_key(&id));
    // Undo restores the previous (absent) policy in one step.
    c.undo().unwrap();
    assert!(c.world().documents[&d].tolerance().is_none());
}

#[test]
fn l3_003_a_stale_tolerance_revision_is_refused_without_mutation() {
    let (doc, _layer) = document();
    let (c, d) = coordinator(doc);
    let before = c.snapshot();
    let refused = c.prepare(TransactionRequest::new(
        "tolerance",
        vec![Mutation::Tolerance {
            document: d,
            expected_revision: 7,
            tolerance: Tolerance::try_new(0.001, 1e-6, 0.).unwrap(),
        }],
    ));
    assert_eq!(
        refused.err(),
        Some(Error::Stale("document tolerance revision"))
    );
    assert_eq!(c.snapshot(), before);
    assert!(c.world().documents[&d].tolerance().is_none());
}

#[test]
fn l3_003_a_tolerance_document_survives_validation_and_json() {
    let (mut doc, _layer) = document();
    doc.tolerance = Some(Tolerance::try_new(0.001, 1e-6, 1e-9).unwrap());
    doc.validate(&Limits::default()).unwrap();
    let text = serde_json::to_string(&doc).unwrap();
    assert!(text.contains("tolerance"));
    let reopened: Document = serde_json::from_str(&text).unwrap();
    assert_eq!(reopened.tolerance().unwrap().absolute_mm, 0.001);
    // A document without a policy writes no tolerance key at all.
    let (plain, _) = document();
    let plain_json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&plain).unwrap()).unwrap();
    assert!(
        plain_json.get("tolerance").is_none(),
        "a document that states no tolerance must not write the key: {plain_json}"
    );
}

#[test]
fn l3_003_a_new_document_states_the_template_policy_explicitly() {
    // `Document::new` still states no tolerance: the template policy is a decision
    // written into the document, never a hidden fallback.
    assert!(Document::new("plain").tolerance().is_none());
    let templated = Document::with_template_tolerance("ring");
    let tolerance = templated.tolerance().expect("template policy");
    assert_eq!(tolerance.absolute_mm, 0.001);
    assert_eq!(tolerance.relative, 1.0e-4);
    assert!((tolerance.angular_rad - std::f64::consts::PI / 180.).abs() < 1e-15);
    templated.validate(&Limits::default()).unwrap();
    assert_eq!(templated.effective_tolerance().unwrap(), *tolerance);
    let reopened: Document =
        serde_json::from_str(&serde_json::to_string(&templated).unwrap()).unwrap();
    assert_eq!(reopened.tolerance(), Some(tolerance));
}
