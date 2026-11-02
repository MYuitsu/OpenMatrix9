// SPDX-License-Identifier: LGPL-2.1-or-later
//! Setting-parts commands (`OM9-L2-04-15`): the icon path stages a preview, the options change
//! it, Enter commits through the transaction (Undo restores), and every prong carries the
//! `om9.settings.prong` payload the editor and the settings mass read-out consume.
use om9_app::session::settings_commands::{
    BEAD_ON_CURVE, BEAD_ON_SURFACE, CHANNEL_BORDER, METAL_PIECE, PRONG_BUILDER, PRONG_EDITOR,
    PRONG_ON_SURFACE, PULL_TO_RAIL, ProngPayload, SETTINGS_MASS,
};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::gem::{gem_entity, GemSpec};
use om9_core::geometry::{Curve, Geometry, Transform};
use om9_core::identity::ObjectId;
use om9_core::model::Document;

const PRONG_SLOT: &str = "om9.settings.prong";

fn place(entity: &mut om9_core::model::Entity, position: [f64; 3]) {
    entity.placement = Transform([
        [1.0, 0.0, 0.0, position[0]],
        [0.0, 1.0, 0.0, position[1]],
        [0.0, 0.0, 1.0, position[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]);
}

fn gem_document(positions: &[[f64; 3]]) -> (Document, Vec<ObjectId>) {
    let mut document = Document::new("settings");
    let mut ids = Vec::new();
    for (index, position) in positions.iter().enumerate() {
        let spec = GemSpec::round(2.0, 1.6).expect("gem spec");
        let mut entity = gem_entity(&format!("Gem {index}"), &spec, None).expect("gem entity");
        place(&mut entity, *position);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    (document, ids)
}

fn gem_controller(positions: &[[f64; 3]]) -> Controller {
    let (document, ids) = gem_document(positions);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&ids).expect("select gems");
    controller
}

fn insert(controller: &mut Controller, entity: om9_core::model::Entity) -> ObjectId {
    let id = entity.id;
    let mut document = controller.document().clone();
    document.entities.insert(entity.id, entity);
    *controller = Controller::new(document, Default::default()).expect("controller");
    id
}

fn labels(controller: &Controller, label: &str) -> Vec<ObjectId> {
    controller
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == label)
        .map(|entity| entity.id)
        .collect()
}

fn line_curve(length: f64) -> Curve {
    Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [length, 0.0, 0.0],
        domain: [0.0, 1.0],
    }
}

#[test]
fn the_icon_stages_a_prong_set_and_enter_commits_one_body_per_prong() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    let before = controller.document().entities.len();
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("icon starts the builder");
    let staged = controller.settings_pending().expect("staged command");
    assert!(staged.message.starts_with("Prong Builder:"), "{}", staged.message);
    let expected = match &staged.draft {
        om9_app::session::settings_commands::SettingsDraft::Prongs { set, .. } => set.count(),
        other => panic!("unexpected draft {other:?}"),
    };
    assert!(expected > 0);

    controller.submit("Enter").expect("commit");
    assert!(controller.settings_pending().is_none());
    let prongs = labels(&controller, "Prong");
    assert_eq!(prongs.len(), expected);
    assert_eq!(controller.document().entities.len(), before + expected);
    assert_eq!(controller.settings_features().len(), 1);
    for id in prongs {
        let entity = controller.document().entities.get(&id).expect("prong");
        let extension = entity.extension(PRONG_SLOT).expect("prong payload");
        let payload = ProngPayload::from_extension(extension).expect("payload parses");
        assert!(payload.feature_id.starts_with(PRONG_BUILDER));
        assert!(!payload.prong_id.is_empty());
    }
}

#[test]
fn an_option_changes_the_preview_before_the_commit() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("start");
    let first = match &controller.settings_pending().expect("pending").draft {
        om9_app::session::settings_commands::SettingsDraft::Prongs { set, .. } => set.count(),
        _ => panic!("wrong draft"),
    };
    controller
        .submit("Mode=4 Prongs 0 Degrees")
        .expect("mode option");
    let second = match &controller.settings_pending().expect("pending").draft {
        om9_app::session::settings_commands::SettingsDraft::Prongs { set, .. } => set.count(),
        _ => panic!("wrong draft"),
    };
    assert!(second > first, "{second} is not more than {first}");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Prong").len(), second);
}

#[test]
fn a_refused_option_keeps_the_command_and_its_last_preview() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("start");
    let error = controller.submit("Diameter=-2").expect_err("refused");
    assert!(matches!(error, om9_core::Error::Invalid(_)));
    assert!(controller.settings_pending().is_some());
    controller.submit("Enter").expect("commit");
    assert!(!labels(&controller, "Prong").is_empty());
}

#[test]
fn the_editor_edits_a_committed_prong_in_place() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Enter").expect("commit");
    let prong = labels(&controller, "Prong")[0];
    let prong_count = labels(&controller, "Prong").len();
    let before = controller.document().entities.get(&prong).expect("prong").clone();

    controller.select_objects(&[prong]).expect("select prong");
    controller
        .dispatch(PRONG_EDITOR, InvocationOrigin::Menu)
        .expect("editor starts");
    controller.submit("Diameter=1.6").expect("diameter option");
    controller.submit("Enter").expect("commit edit");

    let after = controller.document().entities.get(&prong).expect("prong").clone();
    assert_eq!(after.id, before.id);
    assert_ne!(after.geometry, before.geometry);
    let payload = ProngPayload::from_extension(
        after.extension(PRONG_SLOT).expect("payload"),
    )
    .expect("payload parses");
    assert!((payload.site.diameter_mm - 1.6).abs() < 1.0e-9);
    assert_eq!(labels(&controller, "Prong").len(), prong_count);
}

#[test]
fn the_editor_refuses_a_non_prong_selection() {
    let (document, _) = gem_document(&[[0.0, 0.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let id = labels(&controller, "Gem 0")[0];
    controller.select_objects(&[id]).expect("select gem");
    assert_eq!(
        controller
            .dispatch(PRONG_EDITOR, InvocationOrigin::Menu)
            .err(),
        Some(om9_core::Error::Invalid("prong editor needs prong bodies"))
    );
}

#[test]
fn metal_piece_top_mode_stages_a_cutter_and_commits_bodies() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Mode=Top").expect("top mode");
    let message = controller.settings_pending().expect("pending").message.clone();
    assert!(message.contains("cutter"), "{message}");
    controller.submit("Enter").expect("commit");
    assert!(!labels(&controller, "Metal Piece").is_empty());
    // A cutter is committed as a body but never counted as weighed metal.
    assert_eq!(controller.settings_features().len(), 0);
    assert_eq!(controller.settings_cutter_count(), 1);
}

#[test]
fn the_channel_border_commits_two_guide_curves() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    controller
        .dispatch(CHANNEL_BORDER, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Offset=0.3").expect("offset option");
    controller.submit("Enter").expect("commit");
    let left = labels(&controller, "Channel Border Left");
    let right = labels(&controller, "Channel Border Right");
    assert_eq!(left.len(), 1);
    assert_eq!(right.len(), 1);
    for id in [left[0], right[0]] {
        assert!(matches!(
            controller.document().entities.get(&id).expect("border").geometry,
            Geometry::Curve(_)
        ));
    }
}

#[test]
fn bead_on_curve_commits_the_run_along_the_selected_curve() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let guide = insert(
        &mut controller,
        om9_core::model::Entity::new("Guide", Geometry::Curve(line_curve(10.0)), None),
    );
    controller.select_objects(&[guide]).expect("select guide");
    controller
        .dispatch(BEAD_ON_CURVE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Spacing=1").expect("spacing option");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Bead").len(), 11);
    assert_eq!(controller.settings_features().len(), 1);
}

#[test]
fn bead_count_drives_the_run_and_spacing_returns_to_auto() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let guide = insert(
        &mut controller,
        om9_core::model::Entity::new("Guide", Geometry::Curve(line_curve(10.0)), None),
    );
    controller.select_objects(&[guide]).expect("select guide");
    controller
        .dispatch(BEAD_ON_CURVE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Count=5").expect("count option");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Bead").len(), 5);
    // The next run is driven by spacing again; the count does not leak between commands.
    controller.select_objects(&[guide]).expect("select guide");
    controller
        .dispatch(BEAD_ON_CURVE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Spacing=2.5").expect("spacing option");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Bead").len(), 10);
}

#[test]
fn a_refused_bead_count_keeps_the_staged_command() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let guide = insert(
        &mut controller,
        om9_core::model::Entity::new("Guide", Geometry::Curve(line_curve(10.0)), None),
    );
    controller.select_objects(&[guide]).expect("select guide");
    controller
        .dispatch(BEAD_ON_CURVE, InvocationOrigin::Menu)
        .expect("start");
    assert_eq!(
        controller.submit("Count=1").err(),
        Some(om9_core::Error::Invalid("bead count"))
    );
    assert!(controller.settings_pending().is_some());
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn cutter_features_stay_out_of_the_mass() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Mode=Top").expect("top mode");
    controller.submit("Enter").expect("commit cutter");
    // A cutter alone leaves nothing to weigh.
    assert_eq!(
        controller.submit(SETTINGS_MASS).err(),
        Some(om9_core::Error::Missing("settings mass features"))
    );
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Mode=Outside").expect("outside mode");
    controller.submit("Enter").expect("commit metal");
    controller.submit(SETTINGS_MASS).expect("mass command");
    let report = controller.settings_mass_report().expect("mass report");
    assert_eq!(report.features, 1);
    assert_eq!(report.excluded_cutters, 1);
    assert!(report.text.contains("1 cutter excluded"), "{}", report.text);
}

#[test]
fn metal_piece_can_follow_a_committed_channel_border() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    controller
        .dispatch(CHANNEL_BORDER, InvocationOrigin::Menu)
        .expect("border start");
    controller.submit("Enter").expect("border commit");
    assert!(!labels(&controller, "Channel Border Left").is_empty());
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("metal start");
    controller.submit("Border=On").expect("border option");
    controller.submit("Enter").expect("metal commit");
    assert_eq!(labels(&controller, "Metal Piece").len(), 2);
    let features = controller.settings_features();
    assert_eq!(features.len(), 1);
    assert!(features[0].1 > 0.0);
}

#[test]
fn metal_piece_curve_mode_uses_the_committed_border() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    controller
        .dispatch(CHANNEL_BORDER, InvocationOrigin::Menu)
        .expect("border start");
    controller.submit("Enter").expect("border commit");
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("metal start");
    controller.submit("Border=On").expect("border option");
    controller.submit("Mode=Curve").expect("curve mode");
    let message = controller.settings_pending().expect("pending").message.clone();
    assert!(message.contains("Curve"), "{message}");
    controller.submit("Enter").expect("curve commit");
    assert_eq!(labels(&controller, "Metal Piece").len(), 2);
}

#[test]
fn metal_piece_curve_mode_without_a_border_is_refused() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("start");
    assert_eq!(
        controller.submit("Mode=Curve").err(),
        Some(om9_core::Error::Missing("metal piece curve"))
    );
    assert!(controller.settings_pending().is_some());
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn metal_piece_curve_mode_uses_the_picked_curve() {
    // The sheet's Curve mode prompts for "a curve on the target surface"; the picked curve is
    // part of the same selection as the gems and orients the metal walls.
    let (mut document, gems) = gem_document(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
    let guide = om9_core::model::Entity::new("Guide", Geometry::Curve(line_curve(6.0)), None);
    let guide_id = guide.id;
    document.entities.insert(guide_id, guide);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let mut selection = gems;
    selection.push(guide_id);
    controller
        .select_objects(&selection)
        .expect("select row and curve");
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("metal start");
    controller.submit("Mode=Curve").expect("curve mode");
    let message = controller.settings_pending().expect("pending").message.clone();
    assert!(message.contains("picked curve"), "{message}");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Metal Piece").len(), 2);
}

#[test]
fn metal_piece_border_without_a_border_is_refused() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(METAL_PIECE, InvocationOrigin::Menu)
        .expect("start");
    assert_eq!(
        controller.submit("Border=On").err(),
        Some(om9_core::Error::Missing("metal piece channel border"))
    );
    assert!(controller.settings_pending().is_some());
    controller.submit("Cancel").expect("cancel");
}

fn plane_entity(controller: &mut Controller) -> ObjectId {
    let limits = Default::default();
    let plane = om9_core::surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .expect("plane");
    insert(
        controller,
        om9_core::model::Entity::new("Plane", Geometry::Surface(plane), None),
    )
}

#[test]
fn prong_on_surface_stages_from_a_surface_and_commits_one_prong() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let surface = plane_entity(&mut controller);
    controller.select_objects(&[surface]).expect("select plane");
    controller
        .dispatch(PRONG_ON_SURFACE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Diameter=1.2").expect("diameter option");
    controller.submit("UV=0.4,0.6").expect("uv option");
    controller.submit("Enter").expect("commit");
    let prongs = labels(&controller, "Prong");
    assert_eq!(prongs.len(), 1);
    let payload = ProngPayload::from_extension(
        controller.document().entities.get(&prongs[0]).expect("prong").extension(PRONG_SLOT).expect("payload"),
    )
    .expect("payload parses");
    // The plane is z=0, so the surface prong sits on it.
    assert!(payload.site.frame.origin[2].abs() < 1.0e-9);
    assert_eq!(controller.settings_features().len(), 1);
}

#[test]
fn the_prong_builder_stages_the_documented_option_row() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("start");
    let tokens: Vec<&str> = controller
        .settings_option_tokens()
        .into_iter()
        .map(|(token, _)| token)
        .collect();
    for expected in ["Mode=", "Diameter=", "Height=", "Nudge=", "Ends=", "Spacing=", "Enter", "Cancel"] {
        assert!(tokens.contains(&expected), "{expected} missing from {tokens:?}");
    }
    controller.submit("Cancel").expect("cancel");
    assert!(controller.settings_pending().is_none());
    assert!(labels(&controller, "Prong").is_empty());
}

#[test]
fn bead_on_surface_places_one_bead_on_the_picked_uv() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let limits = Default::default();
    let plane = om9_core::surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .expect("plane");
    let surface = insert(
        &mut controller,
        om9_core::model::Entity::new("Plane", Geometry::Surface(plane), None),
    );
    controller.select_objects(&[surface]).expect("select plane");
    controller
        .dispatch(BEAD_ON_SURFACE, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Scale=2").expect("scale option");
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Bead").len(), 1);
}

#[test]
fn pull_to_rail_needs_a_selected_circle_rail() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let id = labels(&controller, "Gem 0")[0];
    controller.select_objects(&[id]).expect("select");
    assert_eq!(
        controller.dispatch(PULL_TO_RAIL, InvocationOrigin::Menu).err(),
        Some(om9_core::Error::Missing("pull needs a selected circle rail"))
    );
}

#[test]
fn pull_to_rail_moves_a_selected_mesh_onto_the_rail() {
    let (document, _) = gem_document(&[[10.0, 10.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let rail = insert(
        &mut controller,
        om9_core::model::Entity::new(
            "Rail",
            Geometry::Curve(Curve::Circle {
                center: [0.0, 0.0, 0.0],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius: 10.0,
                domain: [0.0, 1.0],
                angle_domain: None,
            }),
            None,
        ),
    );
    let mesh = om9_core::settings::sphere([14.0, 0.0, 0.0], 2.0, 16).expect("sphere");
    let mut object = om9_core::model::Entity::new("Ball", Geometry::Mesh(mesh), None);
    place(&mut object, [14.0, 0.0, 0.0]);
    let ball = insert(&mut controller, object);
    controller.select_objects(&[rail, ball]).expect("select both");

    controller
        .dispatch(PULL_TO_RAIL, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Enter").expect("commit");

    let placement = controller.document().entities.get(&ball).expect("ball").placement.0;
    let x = placement[0][3];
    let y = placement[1][3];
    let radial = (x * x + y * y).sqrt();
    assert!((radial - 10.0).abs() < 1.0e-6, "radial {radial}");
    let _ = SETTINGS_MASS;
}

#[test]
fn settings_mass_reports_grams_from_the_committed_features() {
    let mut controller = gem_controller(&[[0.0, 0.0, 0.0], [2.5, 0.0, 0.0]]);
    assert_eq!(
        controller.submit(SETTINGS_MASS).err(),
        Some(om9_core::Error::Missing("settings mass features"))
    );
    controller
        .dispatch(PRONG_BUILDER, InvocationOrigin::Menu)
        .expect("start");
    controller.submit("Enter").expect("commit");
    controller.submit(SETTINGS_MASS).expect("mass command");
    let report = controller.settings_mass_report().expect("mass report");
    assert!(report.grams > 0.0, "{}", report.text);
    assert!(report.dwt > 0.0);
    assert_eq!(report.features, 1);
    assert!(report.text.starts_with("Settings Mass:"), "{}", report.text);
}

#[test]
fn every_icon_alias_resolves_to_its_own_feature() {
    for (alias, feature) in [
        ("ProngBuilder", PRONG_BUILDER),
        ("Prong Editor", PRONG_EDITOR),
        ("MetalPiece", METAL_PIECE),
        ("ChannelBorder", CHANNEL_BORDER),
        ("BeadOnCrv", BEAD_ON_CURVE),
        ("BeadOnSurface", BEAD_ON_SURFACE),
        ("PullToRail", PULL_TO_RAIL),
    ] {
        assert_eq!(Controller::resolve_alias(alias), Some(feature), "{alias}");
    }
}

#[test]
fn a_command_without_a_gem_selection_is_refused() {
    let (document, _) = gem_document(&[[0.0, 0.0, 0.0]]);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    assert_eq!(
        controller.dispatch(PRONG_BUILDER, InvocationOrigin::Menu).err(),
        Some(om9_core::Error::Missing("gem row selection"))
    );
    assert_eq!(
        controller.dispatch(METAL_PIECE, InvocationOrigin::Menu).err(),
        Some(om9_core::Error::Missing("gem row selection"))
    );
}
