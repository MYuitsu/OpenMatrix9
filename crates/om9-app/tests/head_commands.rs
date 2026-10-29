// SPDX-License-Identifier: LGPL-2.1-or-later
//! Head Builder / Bezel Builder / Scallop Bezel commands (`OM9-L2-04-14`): the icon path stages a
//! preview, the options change it, Enter commits through the transaction (Undo restores), and
//! every head/bezel body carries the typed `om9.settings.head` / `om9.settings.bezel` payload.
use om9_app::session::head_commands::{
    BEZEL_BUILDER, BEZEL_CUTTER, HEAD_BUILDER, HeadDraft, HeadPayload, HeadPending,
};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::gem::{gem_entity, GemSpec};
use om9_core::geometry::{Geometry, Transform};
use om9_core::identity::ObjectId;
use om9_core::model::Document;

const HEAD_SLOT: &str = "om9.settings.head";
const BEZEL_SLOT: &str = "om9.settings.bezel";
const SCALLOP_SLOT: &str = "om9.settings.scallop";
const HEAD_SEAT_SLOT: &str = "om9.settings.head.seat";

fn place(entity: &mut om9_core::model::Entity, position: [f64; 3]) {
    entity.placement = Transform([
        [1.0, 0.0, 0.0, position[0]],
        [0.0, 1.0, 0.0, position[1]],
        [0.0, 0.0, 1.0, position[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]);
}

fn gem_controller(diameter: f64) -> (Controller, ObjectId) {
    let mut document = Document::new("head");
    let spec = GemSpec::round(diameter, diameter * 0.62).expect("gem spec");
    let mut entity = gem_entity("Gem 1", &spec, None).expect("gem entity");
    place(&mut entity, [0.0, 0.0, 0.0]);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&[id]).expect("select gem");
    (controller, id)
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

fn staged(controller: &Controller) -> &HeadPending {
    controller.head_pending().expect("staged head command")
}

fn gem_volume(controller: &Controller, gem: ObjectId) -> f64 {
    let entity = controller.document().entities.get(&gem).expect("gem");
    let mesh = match &entity.geometry {
        Geometry::Mesh(mesh) => mesh,
        other => panic!("unexpected gem geometry {other:?}"),
    };
    om9_core::settings::inspect_solid(mesh).expect("gem solid").volume_mm3
}

#[test]
fn the_head_icon_stages_a_basket_and_enter_commits_prongs_and_rails() {
    let (mut controller, _gem) = gem_controller(6.0);
    let before = controller.document().entities.len();
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("icon starts the head builder");
    assert!(staged(&controller).message.starts_with("Head Builder"));
    assert!(staged(&controller).message.contains("4 prongs"));
    assert!(staged(&controller).message.contains("2 rails"));
    controller.submit("Enter").expect("commit");
    assert!(controller.head_pending().is_none());
    assert_eq!(labels(&controller, "Prong").len(), 4);
    assert_eq!(labels(&controller, "Gallery Rail").len(), 2);
    assert_eq!(controller.document().entities.len(), before + 6);
    let prong = labels(&controller, "Prong")[0];
    let entity = controller.document().entities.get(&prong).expect("body");
    let payload = entity
        .extension(HEAD_SLOT)
        .and_then(HeadPayload::from_extension)
        .expect("head payload");
    assert!(payload.gem_id.starts_with("gem:"));
    assert_eq!(payload.params.prongs, 4);
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities.len(), before);
}

#[test]
fn the_half_bezel_toggle_replaces_the_prongs_and_keeps_rails() {
    let (mut controller, _gem) = gem_controller(6.0);
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    controller.submit("Mode=HalfBezel").expect("half bezel");
    assert!(staged(&controller).message.contains("half bezel"));
    let draft = match &staged(&controller).draft {
        HeadDraft::Head { setting, .. } => setting.clone(),
        other => panic!("unexpected draft {other:?}"),
    };
    assert!(draft.prongs.is_empty());
    assert!(draft.prong_profile_ignored);
    assert_eq!(draft.rails.len(), 2);
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Half Bezel").len(), 1);
    assert!(labels(&controller, "Prong").is_empty());
}

#[test]
fn a_level_four_edit_changes_only_its_prong_in_the_preview() {
    let (mut controller, _gem) = gem_controller(6.0);
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    controller.submit("Level=4").expect("level");
    controller.submit("Prong=2").expect("element");
    controller.submit("Size=1.3").expect("size");
    let (first, second) = match &staged(&controller).draft {
        HeadDraft::Head { setting, .. } => (
            setting.prongs[0].diameter_mm,
            setting.prongs[1].diameter_mm,
        ),
        other => panic!("unexpected draft {other:?}"),
    };
    assert!((first - 0.9).abs() < 1.0e-9);
    assert!((second - 1.3).abs() < 1.0e-9);
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Prong").len(), 4);
}

#[test]
fn the_three_bezel_modes_build_and_the_boolean_cuts_a_finger_rail() {
    for (option, expected) in [
        ("Mode=Bezel", "bezel"),
        ("Mode=UnderBezel", "below"),
        ("Mode=GalleryRail", "below"),
    ] {
        let (mut controller, _gem) = gem_controller(6.0);
        controller
            .dispatch(BEZEL_BUILDER, InvocationOrigin::Menu)
            .expect("bezel builder");
        assert!(staged(&controller).message.contains("Bezel Builder"));
        controller.submit(option).expect("mode");
        controller.submit("Enter").expect("commit");
        let bodies = labels(&controller, "Bezel");
        assert_eq!(bodies.len(), 1, "{expected}");
        let entity = controller.document().entities.get(&bodies[0]).expect("body");
        assert!(entity.extension(BEZEL_SLOT).is_some());
        let mesh = match &entity.geometry {
            Geometry::Mesh(mesh) => mesh,
            other => panic!("unexpected geometry {other:?}"),
        };
        let lowest = mesh
            .vertices
            .iter()
            .map(|vertex| vertex[2])
            .fold(f64::INFINITY, f64::min);
        match expected {
            "below" => assert!(lowest < 0.0, "{option} sits below the girdle"),
            _ => assert!(lowest <= 0.0),
        }
        // The manual default Boolean=ON subtracts the finger rail, committed as its own body.
        assert_eq!(labels(&controller, "Finger Rail").len(), 1, "{option} rail");
    }

    // Boolean=Off keeps the solid ring: no finger rail body and more metal.
    let (mut controller, _gem) = gem_controller(6.0);
    controller
        .dispatch(BEZEL_BUILDER, InvocationOrigin::Menu)
        .expect("bezel builder");
    controller.submit("Boolean=Off").expect("boolean off");
    controller.submit("Enter").expect("commit");
    assert!(labels(&controller, "Finger Rail").is_empty());
}

#[test]
fn scallop_cutters_commit_and_stay_out_of_the_settings_mass() {
    let (mut controller, gem) = gem_controller(6.0);
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    controller.submit("Enter").expect("head committed");
    controller.select_objects(&[gem]).expect("reselect gem");
    controller
        .dispatch(BEZEL_CUTTER, InvocationOrigin::Menu)
        .expect("scallop bezel");
    assert!(staged(&controller).message.contains("12 cutters"));
    controller
        .submit("Profile=Bezier:0,0;1,0;1,0.5;0.5,1;0,0.5")
        .expect("bezier cutter profile");
    controller.submit("Enter").expect("cutters committed");
    assert_eq!(labels(&controller, "Scallop Cutter").len(), 12);
    assert_eq!(controller.settings_cutter_count(), 1);
    let cutter = labels(&controller, "Scallop Cutter")[0];
    let entity = controller.document().entities.get(&cutter).expect("cutter");
    assert!(entity.extension(SCALLOP_SLOT).is_some());
    let mass = controller
        .calculate_settings_mass(None)
        .expect("mass report");
    assert_eq!(mass.excluded_cutters, 1);
    assert!(mass.cutter_volume_mm3 > 0.0);
    assert!(mass.net_volume_mm3 < mass.volume_mm3);
    assert!((mass.volume_mm3 - mass.cutter_volume_mm3 - mass.net_volume_mm3).abs() < 1.0e-6);
    assert!(mass.text.contains("cutter volume"));
    assert!(mass.text.contains("net metal"));
}

#[test]
fn the_bezier_profile_is_saved_and_resolved_by_name() {
    let (mut controller, _gem) = gem_controller(6.0);
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    let plain = match &staged(&controller).draft {
        HeadDraft::Head { setting, .. } => setting.volume_mm3,
        other => panic!("unexpected draft {other:?}"),
    };
    controller
        .submit("Profile=Bezier:0,0;0.5,0.1;0.55,0.8;0,1")
        .expect("bezier profile");
    let draft = match &staged(&controller).draft {
        HeadDraft::Head { feature, setting, .. } => (feature.params.clone(), setting.volume_mm3),
        other => panic!("unexpected draft {other:?}"),
    };
    assert!(draft.0.profile_section.is_some());
    assert!((draft.1 - plain).abs() > 1.0e-6, "the Bezier section shapes the prongs");
    controller.submit("ProfileSave=Leaf").expect("save profile");
    assert_eq!(controller.head_profile_names(), vec!["Leaf".to_string()]);
    controller.submit("Profile=Round").expect("builtin profile");
    controller.submit("Profile=Leaf").expect("user profile");
    let resolved = match &staged(&controller).draft {
        HeadDraft::Head { feature, .. } => feature.params.clone(),
        other => panic!("unexpected draft {other:?}"),
    };
    assert!(resolved.profile_section.is_some());
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Prong").len(), 4);
}

#[test]
fn the_seat_cut_opens_the_stone_and_undo_restores_it() {
    let (mut controller, gem) = gem_controller(6.0);
    let before = gem_volume(&controller, gem);
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    controller.submit("SeatDepth=0.25").expect("seat depth");
    controller.submit("Enter").expect("commit");
    let after = gem_volume(&controller, gem);
    assert!(after < before, "the seat cut removes stone");
    let entity = controller.document().entities.get(&gem).expect("gem");
    let payload = entity
        .extension(HEAD_SEAT_SLOT)
        .and_then(HeadPayload::from_extension)
        .expect("seat cut payload");
    assert_eq!(payload.seat_count, 4);
    assert!(payload.seat_cut_removed_mm3 > 0.0);
    controller.undo().expect("undo");
    let restored = gem_volume(&controller, gem);
    assert!((restored - before).abs() < 1.0e-6);
    assert!(
        controller
            .document()
            .entities
            .get(&gem)
            .expect("gem")
            .extension(HEAD_SEAT_SLOT)
            .is_none()
    );
}

#[test]
fn a_selection_without_a_gem_is_refused_by_name() {
    let mut document = Document::new("head");
    let entity = om9_core::model::Entity::new(
        "Curve",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        document.current_layer,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&[id]).expect("select");
    let error = controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect_err("no gem");
    assert!(format!("{error:?}").contains("head gem selection"));
}

#[test]
fn cancel_discards_the_preview_without_touching_the_document() {
    let (mut controller, _gem) = gem_controller(6.0);
    let before = controller.document().entities.len();
    controller
        .dispatch(HEAD_BUILDER, InvocationOrigin::Menu)
        .expect("head builder");
    controller.submit("Cancel").expect("cancel");
    assert!(controller.head_pending().is_none());
    assert_eq!(controller.document().entities.len(), before);
    assert!(labels(&controller, "Prong").is_empty());
}
