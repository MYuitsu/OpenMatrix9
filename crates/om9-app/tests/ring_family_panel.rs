// SPDX-License-Identifier: LGPL-2.1-or-later
//! The jewelry ring builder family through the controller (`OM9-L2-04-03`): the command
//! line opens each mode, Apply commits the body in one undoable step, and a preset or an
//! invalid parameter goes through the same typed path the panel uses. The product build
//! commits an OCCT B-Rep; the portable build (no `occt-brep`) commits the mesh of the same
//! sections, and both go through the one feasibility gate.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    builder::BuilderKind,
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::{Document, Entity},
    ring_builder,
};

fn limits() -> Limits {
    Limits::default()
}

fn document() -> Document {
    let mut document = Document::new("ring-family-ui");
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
    let id = rail.id;
    document.entities.insert(id, rail);
    document
}

fn controller() -> Controller {
    Controller::new(document(), limits()).unwrap()
}

/// The committed ring measured by the shared feasibility gate. The product build registers
/// the OCCT producer, so a ring reaches the document as a B-Rep body, not as a preview mesh.
fn applied_report(controller: &Controller, output: ObjectId) -> ring_builder::SolidReport {
    let geometry = &controller.document().entities[&output].geometry;
    let occt = cfg!(feature = "occt-brep");
    match geometry {
        Geometry::Brep(_) if occt => {}
        Geometry::Mesh(_) if !occt => {}
        other => panic!(
            "unexpected ring body with occt-brep={occt}: {}",
            other.kind().tag()
        ),
    }
    ring_builder::inspect_geometry(geometry, &limits()).unwrap()
}

#[test]
fn the_ring_family_opens_from_the_command_line_and_lists_its_parameters() {
    let mut controller = controller();
    for (command, kind, expected) in [
        ("Eternity Band", BuilderKind::EternityBand, "stone_count"),
        ("Signet Ring", BuilderKind::SignetRing, "top_width_mm"),
        ("Raised Band", BuilderKind::RaisedBand, "raised_height_mm"),
        ("Award Ring", BuilderKind::AwardRing, "top_length_mm"),
    ] {
        controller.submit(command).unwrap();
        let session = controller.builder().unwrap();
        assert_eq!(session.kind(), kind);
        assert!(
            session.field(expected).is_some(),
            "{command} must expose {expected}"
        );
        assert!(session.field("diameter_mm").is_some());
        controller.builder_cancel();
    }
    let tabs: Vec<BuilderKind> = controller
        .builder_tabs()
        .into_iter()
        .map(|(kind, _)| kind)
        .collect();
    for kind in [
        BuilderKind::EternityBand,
        BuilderKind::SignetRing,
        BuilderKind::RaisedBand,
        BuilderKind::AwardRing,
    ] {
        assert!(tabs.contains(&kind));
    }
}

#[test]
fn an_eternity_band_applies_a_closed_solid_in_one_undoable_step() {
    let mut controller = controller();
    controller.submit("EternityBand").unwrap();
    let output = controller.builder().unwrap().output();
    controller.builder_apply().unwrap();
    assert!(controller.builder().is_none());
    let document = controller.document();
    assert_eq!(document.entities.len(), 2);
    let report = applied_report(&controller, output);
    assert!(report.closed);
    assert!(report.volume_mm3 > 0.0);
    // The stone ladder is a frame list, not twelve gem bodies.
    let frames =
        ring_builder::stone_frames(&om9_core::ring_builder::RingBuilderKind::Eternity.defaults())
            .unwrap();
    assert_eq!(frames.len(), 12);
    assert_eq!(document.history.recipes.len(), 1);
    controller.undo().unwrap();
    assert!(!controller.document().entities.contains_key(&output));
}

#[test]
fn a_signet_style_preset_applies_to_the_signet_builder() {
    let mut controller = controller();
    controller.submit("Signet Ring").unwrap();
    let rows = controller.builder_styles();
    let signet = rows
        .iter()
        .find(|row| row.name == "Signet 3.0")
        .expect("the builtin signet style is listed");
    assert!(signet.enabled);
    let summary = controller.builder_apply_style("Signet 3.0").unwrap();
    assert!(!summary.is_empty());
    let session = controller.builder().unwrap();
    // The band-family builtin carries the width/height profile vocabulary: the signet mode
    // applies the width it owns and ignores the rail-profile height it does not.
    assert_eq!(session.field("width_mm").unwrap().numeric(), Some(3.0));
    controller.builder_apply().unwrap();
    let output = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.label.starts_with("Signet"))
        .map(|entity| entity.id)
        .unwrap();
    assert!(applied_report(&controller, output).closed);
}

#[test]
fn an_invalid_ring_parameter_is_refused_and_the_document_stays_empty() {
    let mut controller = controller();
    controller.submit("Signet Ring").unwrap();
    let before = controller.document().entities.len();
    // A top longer than half the circumference folds the shoulders: the preview refuses it.
    // Both values are inside their input domains, so the refusal comes from the geometry gate,
    // not from the slider range.
    controller.submit("Diameter=8").unwrap();
    let error = controller.submit("Top Length=30").unwrap_err();
    assert!(error.to_string().contains("signet top"), "{error}");
    assert_eq!(
        controller.builder().unwrap().kind(),
        BuilderKind::SignetRing
    );
    // The refused edit never reached the document, and the session stays open.
    assert_eq!(controller.document().entities.len(), before);
    // Correcting the value restores a preview, so Apply commits the corrected body.
    controller.submit("Top Length=10").unwrap();
    controller.builder_apply().unwrap();
    assert!(controller.document().entities.len() > before);
    let output = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.label.starts_with("Signet"))
        .map(|entity| entity.id)
        .unwrap();
    assert!(applied_report(&controller, output).closed);
}

#[test]
fn the_ring_handles_move_the_same_fields_the_panel_edits() {
    let mut controller = controller();
    controller.submit("Signet Ring").unwrap();
    let handles = controller.builder_handles();
    let names: Vec<&str> = handles.iter().map(|handle| handle.name).collect();
    for name in [
        "diameter",
        "width",
        "wall",
        "shoulder",
        "top_width",
        "top_length",
    ] {
        assert!(names.contains(&name), "the signet must paint {name}");
    }
    // A two-millimetre diameter drag is one millimetre of radius, and the field follows it.
    let before = handles
        .iter()
        .find(|handle| handle.name == "diameter")
        .unwrap()
        .value;
    controller.builder_adjust_field("diameter_mm", 2.0).unwrap();
    let after = controller
        .builder_handles()
        .into_iter()
        .find(|handle| handle.name == "diameter")
        .unwrap()
        .value;
    assert!((after - (before + 2.0)).abs() < 1e-6);
    // The drag rebuilds the preview through the same producer; it stays a closed body.
    let report = controller
        .builder()
        .and_then(|session| session.preview())
        .map(|preview| ring_builder::inspect_geometry(&preview.geometry, &limits()).unwrap())
        .unwrap();
    assert!(report.closed && report.volume_mm3 > 0.0);
    // The wall grip cannot drag the section below the family minimum.
    controller
        .builder_adjust_field("thickness_mm", -10.0)
        .unwrap();
    let wall = controller
        .builder_handles()
        .into_iter()
        .find(|handle| handle.name == "wall")
        .unwrap()
        .value;
    assert!((wall - om9_core::ring_builder::MIN_WALL_MM).abs() < 1e-9);
}
