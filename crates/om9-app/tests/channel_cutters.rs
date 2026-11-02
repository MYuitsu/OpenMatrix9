// SPDX-License-Identifier: LGPL-2.1-or-later
//! Channel / Micro Prong / Bright Cut commands (`OM9-L2-04-17`): the icon stages a preview, the
//! options change it, Enter commits through one transaction (Undo restores), the Interactive
//! Boolean cuts every selected metal body, and every cutter carries `om9.cutter` with its row.
use om9_app::session::channel_cutters::{
    BRIGHT_CUT_CHANNEL, BRIGHT_CUTTER, CHANNEL_BUILDER, MICRO_PRONG, RowCutterPayload, RowTool,
};
use om9_app::session::gem_cutters::CUTTER_CUT_SLOT;
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::gem::{GemSpec, gem_entity};
use om9_core::geometry::{Geometry, Mesh, Transform};
use om9_core::identity::ObjectId;
use om9_core::model::{CUTTER_EXTENSION_SLOT, Document, Entity};

fn place(entity: &mut Entity, position: [f64; 3]) {
    entity.placement = Transform([
        [1.0, 0.0, 0.0, position[0]],
        [0.0, 1.0, 0.0, position[1]],
        [0.0, 0.0, 1.0, position[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]);
}

/// A closed box whose top face sits at `height`.
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

/// A row of `count` stones along +X, 7 mm apart.
fn row_controller(count: usize) -> (Controller, Vec<ObjectId>) {
    let mut document = Document::new("row cutters");
    let mut ids = Vec::new();
    for index in 0..count {
        let spec = GemSpec::round(6.5, 4.03).expect("gem spec");
        let mut entity = gem_entity(&format!("Gem {index}"), &spec, None).expect("gem entity");
        place(&mut entity, [index as f64 * 7.0, 0.0, 0.0]);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&ids).expect("select row");
    (controller, ids)
}

/// A row plus one metal body under it.
fn row_with_metal() -> (Controller, Vec<ObjectId>, ObjectId) {
    let mut document = Document::new("row cutters");
    let mut gems = Vec::new();
    for index in 0..3 {
        let spec = GemSpec::round(6.5, 4.03).expect("gem spec");
        let mut entity = gem_entity(&format!("Gem {index}"), &spec, None).expect("gem entity");
        place(&mut entity, [index as f64 * 7.0, 0.0, 0.0]);
        gems.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut body = Entity::new(
        "Metal",
        Geometry::Mesh(box_mesh(24.0, 6.0)),
        document.current_layer,
    );
    place(&mut body, [-3.5, 0.0, -3.2]);
    let body_id = body.id;
    document.entities.insert(body_id, body);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    let mut selection = gems.clone();
    selection.push(body_id);
    controller.select_objects(&selection).expect("select row and metal");
    (controller, gems, body_id)
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

fn payload(controller: &Controller, id: ObjectId) -> RowCutterPayload {
    let entity = controller.document().entities.get(&id).expect("cutter");
    let extension = entity.extension(CUTTER_EXTENSION_SLOT).expect("cutter slot");
    serde_json::from_str(&extension.payload).expect("payload")
}

fn assert_closed(entity: &Entity, what: &str) {
    let Geometry::Mesh(mesh) = &entity.geometry else {
        panic!("{what} is not a mesh");
    };
    let report = om9_core::settings::inspect_solid(mesh).expect(what);
    assert!(report.closed, "{what} must be closed");
    assert!(report.volume_mm3 > 0.0, "{what} must have a positive volume");
}

#[test]
fn the_channel_icon_stages_a_preview_and_enter_commits_one_transaction() {
    let (mut controller, gems) = row_controller(3);
    let before = controller.document().entities.len();
    controller
        .dispatch(CHANNEL_BUILDER, InvocationOrigin::Menu)
        .expect("channel builder");
    let session = controller.row_cutter_session().expect("staged");
    assert_eq!(session.stones.len(), 3);
    assert_eq!(session.cutters.len(), 1);
    assert_eq!(controller.document().entities.len(), before, "preview owns nothing");
    controller.submit("Enter").expect("commit");
    let cutters = labels(&controller, "Channel Builder");
    assert_eq!(cutters.len(), 1);
    let entity = controller.document().entities.get(&cutters[0]).expect("cutter");
    assert_closed(entity, "channel cutter");
    let record = payload(&controller, cutters[0]);
    assert_eq!(record.kind, "channel_cutter");
    assert_eq!(record.row.len(), 3, "row provenance");
    assert_eq!(record.row[0], "Gem 0");
    // Undo restores the pre-command document in one step.
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities.len(), before);
    assert_eq!(labels(&controller, "Channel Builder").len(), 0);
    assert!(controller.document().entities.contains_key(&gems[0]));
}

#[test]
fn the_channel_options_reach_the_cutter_and_a_bad_width_is_refused() {
    let (mut controller, _) = row_controller(2);
    controller
        .dispatch(CHANNEL_BUILDER, InvocationOrigin::Menu)
        .expect("channel builder");
    controller.submit("Width=7.4").expect("wider");
    controller.submit("Depth=1.2").expect("shallower");
    controller.submit("Start=Taper").expect("tapered start");
    controller.submit("End=Round").expect("rounded end");
    controller.submit("Middle=Wide").expect("middle section");
    let staged = controller.row_cutter_session().expect("staged");
    assert_eq!(staged.channel.width_mm, 7.4);
    assert_eq!(staged.channel.start.label(), "Taper");
    assert_eq!(staged.channel.end.label(), "Round");
    assert_eq!(staged.cutters.len(), 1);
    // A channel narrower than its stone would cut the stone: typed refusal, preview survives.
    let error = controller
        .submit("Width=2.0")
        .expect_err("narrower than the stone");
    assert!(format!("{error:?}").contains("channel width"), "{error:?}");
    assert_eq!(
        controller.row_cutter_session().expect("still staged").channel.width_mm,
        7.4
    );
    controller.submit("Cancel").expect("cancel");
    assert!(controller.row_cutter_session().is_none());
}

#[test]
fn micro_prong_builds_one_scallop_per_gap_and_records_the_count() {
    let (mut controller, _) = row_controller(3);
    controller
        .dispatch(MICRO_PRONG, InvocationOrigin::Menu)
        .expect("micro prong");
    assert_eq!(controller.row_cutter_session().expect("staged").cutters.len(), 2);
    controller.submit("Count=4").expect("four scallops");
    assert_eq!(controller.row_cutter_session().expect("staged").cutters.len(), 4);
    controller.submit("Angle=25").expect("leaning scallops");
    controller.submit("Enter").expect("commit");
    let cutters = labels(&controller, "Micro Prong Cutter");
    assert_eq!(cutters.len(), 4);
    for id in &cutters {
        let entity = controller.document().entities.get(id).expect("cutter");
        assert_closed(entity, "micro prong scallop");
        assert_eq!(payload(&controller, *id).kind, "micro_prong_cutter");
    }
}

#[test]
fn the_bright_cutter_rings_one_stone_and_the_channel_mirrors_the_row() {
    let (mut controller, _) = row_controller(1);
    controller
        .dispatch(BRIGHT_CUTTER, InvocationOrigin::Menu)
        .expect("bright cutter");
    controller.submit("Scale2D=140").expect("scale");
    controller.submit("Height=0.3").expect("height");
    controller.submit("Enter").expect("commit");
    let cutters = labels(&controller, "Bright Cutter");
    assert!(!cutters.is_empty());
    assert_eq!(payload(&controller, cutters[0]).kind, "bright_cutter");
    assert_eq!(labels(&controller, "Bright Cutter").len(), 6);

    let (mut controller, _) = row_controller(2);
    controller
        .dispatch(BRIGHT_CUT_CHANNEL, InvocationOrigin::Menu)
        .expect("bright cut channel");
    let one_side = controller
        .row_cutter_session()
        .expect("staged")
        .cutters
        .len();
    assert_eq!(one_side, 4, "two stones, mirrored to both sides");
    controller.submit("Mirror=off").expect("one side");
    assert_eq!(
        controller.row_cutter_session().expect("staged").cutters.len(),
        2
    );
    controller.submit("Enter").expect("commit");
    assert_eq!(labels(&controller, "Bright Cut Channel").len(), 2);
}

#[test]
fn the_interactive_boolean_cuts_every_selected_metal_body() {
    let (mut controller, _gems, body) = row_with_metal();
    controller
        .dispatch(CHANNEL_BUILDER, InvocationOrigin::Menu)
        .expect("channel builder");
    controller.submit("Boolean=On").expect("boolean on");
    let staged = controller.row_cutter_session().expect("staged");
    assert_eq!(staged.cuts.len(), 1, "one cut per selected metal body");
    assert!(staged.cuts[0].removed_mm3 > 0.0);
    controller.submit("Preserve=Remove").expect("remove bodies");
    controller.submit("Enter").expect("commit");
    // The metal body was replaced and no cutter body was inserted.
    assert_eq!(labels(&controller, "Channel Builder").len(), 0);
    assert_eq!(labels(&controller, "Metal").len(), 1);
    let entity = controller.document().entities.get(&body).expect("metal");
    let extension = entity.extension(CUTTER_CUT_SLOT).expect("cut record");
    assert!(extension.payload.contains("\"kind\":\"channel_cutter\""));
    assert!(extension.payload.contains("\"row\":[\"Gem 0\",\"Gem 1\",\"Gem 2\"]"));
}

#[test]
fn a_boolean_without_a_metal_body_is_refused_and_keeps_the_preview() {
    let (mut controller, _) = row_controller(2);
    controller
        .dispatch(CHANNEL_BUILDER, InvocationOrigin::Menu)
        .expect("channel builder");
    let error = controller.submit("Boolean=On").expect_err("no metal body");
    assert!(format!("{error:?}").contains("cutter boolean surface"), "{error:?}");
    assert!(controller.row_cutter_session().is_some());
    let error = controller
        .submit("Preserve=Remove")
        .expect_err("remove needs a boolean");
    assert!(format!("{error:?}").contains("cutter preserve"), "{error:?}");
}

#[test]
fn every_row_command_resolves_from_the_typed_line_and_needs_a_gem() {
    let (mut controller, _) = row_controller(2);
    for (typed, tool) in [
        ("gvChannelBuilder", RowTool::ChannelBuilder),
        ("micro prong cutter", RowTool::MicroProng),
        ("BrightCutChannel", RowTool::BrightCutChannel),
        ("gvBrightCutter", RowTool::BrightCutter),
    ] {
        controller.submit(typed).expect("typed command");
        assert_eq!(
            controller.row_cutter_session().expect("staged").tool,
            tool,
            "{typed}"
        );
        controller.submit("Cancel").expect("cancel");
    }
    assert_eq!(RowTool::from_feature(CHANNEL_BUILDER), Some(RowTool::ChannelBuilder));

    let mut document = Document::new("metal only");
    let body = Entity::new(
        "Metal",
        Geometry::Mesh(box_mesh(10.0, 4.0)),
        document.current_layer,
    );
    let body_id = body.id;
    document.entities.insert(body_id, body);
    let mut controller = Controller::new(document, Default::default()).expect("controller");
    controller.select_objects(&[body_id]).expect("select");
    for feature in [CHANNEL_BUILDER, MICRO_PRONG, BRIGHT_CUT_CHANNEL, BRIGHT_CUTTER] {
        let error = controller
            .dispatch(feature, InvocationOrigin::Menu)
            .expect_err("no gem");
        assert!(format!("{error:?}").contains("cutter gems"), "{feature}");
    }
}
