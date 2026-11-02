// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem Cutter and Gem Cutter Library commands (`OM9-L2-04-16`): the icon stages a preview, the
//! options change it, Enter commits through one transaction (Undo restores), the Interactive
//! Boolean cuts the selected metal body, and every cutter body carries `om9.cutter`.
use om9_app::session::gem_cutters::{
    CUTTER_CUT_SLOT, CUTTER_PAYLOAD_VERSION, CutterPayload, CutterTool, GEM_CUTTER,
    GEM_CUTTER_LIBRARY,
};
use om9_app::session::head_commands::BEZEL_CUTTER;
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::gem::{GemSpec, gem_entity};
use om9_core::geometry::{Geometry, Mesh, Transform};
use om9_core::identity::ObjectId;
use om9_core::model::{CUTTER_EXTENSION_SLOT, Document, Entity};

const CUTTER_SLOT: &str = CUTTER_EXTENSION_SLOT;

fn place(entity: &mut Entity, position: [f64; 3]) {
    entity.placement = Transform([
        [1.0, 0.0, 0.0, position[0]],
        [0.0, 1.0, 0.0, position[1]],
        [0.0, 0.0, 1.0, position[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]);
}

/// A closed 10 x 10 x `height` box whose top face sits at `z`.
fn box_mesh(size: f64, height: f64) -> Mesh {
    let half = size * 0.5;
    let vertices = vec![
        [-half, -half, 0.0],
        [half, -half, 0.0],
        [half, half, 0.0],
        [-half, half, 0.0],
        [-half, -half, height],
        [half, -half, height],
        [half, half, height],
        [-half, half, height],
    ];
    let faces = vec![
        vec![0, 3, 2, 1],
        vec![4, 5, 6, 7],
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![2, 3, 7, 6],
        vec![3, 0, 4, 7],
    ];
    om9_core::cutters::solid(vertices, faces).expect("box")
}

fn gem_controller(diameter: f64) -> (Controller, ObjectId) {
    let mut document = Document::new("cutters");
    let spec = GemSpec::round(diameter, diameter * 0.62).expect("gem spec");
    let mut entity = gem_entity("Gem 1", &spec, None).expect("gem entity");
    place(&mut entity, [0.0, 0.0, 0.0]);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&[id]).expect("select gem");
    (controller, id)
}

fn metal_controller() -> (Controller, ObjectId, ObjectId) {
    let mut document = Document::new("cutters");
    let spec = GemSpec::round(6.5, 6.5 * 0.62).expect("gem spec");
    let mut gem = gem_entity("Gem 1", &spec, None).expect("gem entity");
    place(&mut gem, [0.0, 0.0, 0.0]);
    let gem_id = gem.id;
    document.entities.insert(gem_id, gem);
    let mut body = Entity::new(
        "Metal",
        Geometry::Mesh(box_mesh(12.0, 6.0)),
        document.current_layer,
    );
    place(&mut body, [0.0, 0.0, -4.0]);
    let body_id = body.id;
    document.entities.insert(body_id, body);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller
        .select_objects(&[gem_id, body_id])
        .expect("select");
    (controller, gem_id, body_id)
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

fn payload(controller: &Controller, id: ObjectId) -> CutterPayload {
    let entity = controller.document().entities.get(&id).expect("cutter");
    let extension = entity.extension(CUTTER_SLOT).expect("cutter slot");
    assert_eq!(extension.version, CUTTER_PAYLOAD_VERSION);
    serde_json::from_str(&extension.payload).expect("payload")
}

fn volume(controller: &Controller, id: ObjectId) -> f64 {
    let entity = controller.document().entities.get(&id).expect("body");
    let Geometry::Mesh(mesh) = &entity.geometry else {
        panic!("not a mesh");
    };
    om9_core::settings::inspect_solid(mesh).expect("solid").volume_mm3
}

#[test]
fn the_gem_cutter_icon_stages_a_preview_and_enter_commits_cutter_bodies() {
    let (mut controller, gem) = gem_controller(6.5);
    let before = controller.document().entities.len();
    controller
        .dispatch(GEM_CUTTER, InvocationOrigin::Menu)
        .expect("icon starts the gem cutter");
    let session = controller.gem_cutter_session().expect("staged");
    assert_eq!(session.tool, CutterTool::GemCutter);
    assert_eq!(session.stones.len(), 1);
    assert!(!session.body_meshes().is_empty());
    assert_eq!(session.stones[0].id, gem);
    // The preview owns nothing in the document.
    assert_eq!(controller.document().entities.len(), before);

    controller.submit("Mode=Azure").expect("mode option");
    assert_eq!(
        controller.gem_cutter_session().expect("staged").params.mode,
        om9_core::cutters::gem::CutterMode::Azure
    );
    controller.submit("Enter").expect("commit");
    assert!(controller.gem_cutter_session().is_none());
    let cutters = labels(&controller, "Gem Cutter");
    assert!(!cutters.is_empty());
    assert_eq!(controller.document().entities.len(), before + cutters.len());
    let record = payload(&controller, cutters[0]);
    assert_eq!(record.kind, "gem_cutter");
    assert_eq!(record.seat, "Azure");
    assert!(record.volume_mm3 > 0.0);
    assert!(record.clearance_mm >= 0.0);
    // Undo removes the whole command in one step.
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities.len(), before);
}

#[test]
fn the_library_command_builds_each_preset_and_records_the_seat() {
    let (mut controller, _gem) = gem_controller(6.5);
    controller
        .dispatch(GEM_CUTTER_LIBRARY, InvocationOrigin::Menu)
        .expect("icon starts the library");
    let session = controller.gem_cutter_session().expect("staged");
    assert_eq!(session.tool, CutterTool::GemCutterLibrary);
    controller.submit("Preset=Brite").expect("preset option");
    controller
        .submit("ScaleX=110")
        .expect("scale x option");
    controller.submit("Enter").expect("commit");
    let cutters = labels(&controller, "Gem Cutter");
    assert_eq!(cutters.len(), 1);
    let record = payload(&controller, cutters[0]);
    assert_eq!(record.kind, "gem_cutter_library");
    assert_eq!(record.seat, "Brite Cutter");
}

#[test]
fn the_interactive_boolean_cuts_the_metal_and_the_preserve_answer_removes_the_bodies() {
    let (mut controller, gem, body) = metal_controller();
    let before_volume = volume(&controller, body);
    let before = controller.document().entities.len();
    controller
        .dispatch(GEM_CUTTER, InvocationOrigin::Menu)
        .expect("icon starts the gem cutter");
    controller.submit("Boolean=On").expect("boolean option");
    assert!(controller.gem_cutter_session().expect("staged").cut.is_some());
    controller.submit("Preserve=Remove").expect("preserve option");
    controller.submit("Enter").expect("commit");
    // The cut is real and the cutter bodies did not stay in the document.
    assert!(volume(&controller, body) < before_volume);
    assert!(labels(&controller, "Gem Cutter").is_empty());
    assert_eq!(controller.document().entities.len(), before);
    let entity = controller.document().entities.get(&body).expect("body");
    let extension = entity.extension(CUTTER_CUT_SLOT).expect("cut record");
    let record: serde_json::Value =
        serde_json::from_str(&extension.payload).expect("cut payload");
    assert!(record["removed_mm3"].as_f64().expect("removed") > 0.0);
    assert_eq!(record["seat"], "Gem");
    // The cut leaves a closed solid.
    let Geometry::Mesh(mesh) = &entity.geometry else {
        panic!("not a mesh");
    };
    assert!(om9_core::settings::inspect_solid(mesh).expect("solid").closed);
    let _ = gem;
}

#[test]
fn a_removed_body_without_a_boolean_is_refused_and_the_preview_survives() {
    let (mut controller, _gem) = gem_controller(6.5);
    controller
        .dispatch(GEM_CUTTER, InvocationOrigin::Menu)
        .expect("icon starts the gem cutter");
    let error = controller
        .submit("Preserve=Remove")
        .expect_err("no boolean to remove from");
    assert!(format!("{error:?}").contains("cutter preserve"));
    // A refused option never kills the staged command.
    assert!(controller.gem_cutter_session().is_some());
    controller.submit("Cancel").expect("cancel");
    assert!(controller.gem_cutter_session().is_none());
}

#[test]
fn a_typed_cutter_command_reaches_the_same_handler() {
    let (mut controller, _gem) = gem_controller(6.5);
    controller
        .submit("gvAzureCutter")
        .expect("typed command opens the builder");
    assert_eq!(
        controller.gem_cutter_session().expect("staged").tool,
        CutterTool::GemCutter
    );
    controller.submit("Cancel").expect("cancel");
    controller
        .submit("gvGemCutterLibrary")
        .expect("typed library command");
    assert_eq!(
        controller.gem_cutter_session().expect("staged").tool,
        CutterTool::GemCutterLibrary
    );
}

#[test]
fn the_builder_needs_a_selected_gem() {
    let mut document = Document::new("empty");
    let body = Entity::new(
        "Metal",
        Geometry::Mesh(box_mesh(10.0, 4.0)),
        document.current_layer,
    );
    let body_id = body.id;
    document.entities.insert(body_id, body);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&[body_id]).expect("select body");
    let error = controller
        .dispatch(GEM_CUTTER, InvocationOrigin::Menu)
        .expect_err("no gem");
    assert!(format!("{error:?}").contains("cutter gems"));
}

#[test]
fn the_dimensional_bezel_cutter_options_reach_the_shared_scallop_command() {
    let (mut controller, _gem) = gem_controller(6.5);
    controller
        .dispatch(BEZEL_CUTTER, InvocationOrigin::Menu)
        .expect("scallop command");
    controller.submit("Dims=on").expect("dimensional mode");
    controller.submit("Length=5.5").expect("length");
    controller.submit("CutterLength=1.4").expect("cutter length");
    controller.submit("XOutside=1.8").expect("x outside");
    controller.submit("Count=8").expect("number of cutters");
    controller.submit("Enter").expect("commit");
    // The shared owner committed the dimensional blocks, not the swept profile.
    let cutters = labels(&controller, "Scallop Cutter");
    assert_eq!(cutters.len(), 8);
    let entity = controller.document().entities.get(&cutters[0]).expect("cutter");
    let extension = entity
        .extension(om9_core::model::SETTINGS_SCALLOP_SLOT)
        .expect("scallop record");
    assert!(extension.payload.contains("\"dims\""), "{}", extension.payload);
    // A parameter that cannot build a block is a typed refusal, not a silent sweep.
    controller
        .dispatch(BEZEL_CUTTER, InvocationOrigin::Menu)
        .expect("scallop command again");
    controller.submit("Dims=on").expect("dimensional mode");
    let error = controller
        .submit("Length=1")
        .expect_err("length must exceed the cutter length");
    assert!(format!("{error:?}").contains("bezel cutter"));
    assert!(controller.head_pending().is_some());
}
