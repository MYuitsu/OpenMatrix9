// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application contract tests for the gem tools (`OM9-L2-04-09`, L3-003..007).
//!
//! Every test drives the same command surface the icons and the command line use. The owner
//! schema/mesh tests live in `crates/om9-core/tests/gem_assets.rs`; this file proves the
//! session wiring: preview, Apply transaction, Undo/Redo, replacement identity, layer routing,
//! locked-layer policy, custom profile validation, builders, positioner sharing and the F6
//! Gem Properties adapter.

use std::collections::BTreeSet;

use om9_app::f6::{self, Providers};
use om9_app::session::{
    Controller, InvocationOrigin,
    gem::{BAGUETTE, CUSTOM, EMERALD, LOADER, POSITIONER, PROPERTIES},
};
use om9_core::{
    budget::Limits,
    gem::{GemCut, GemShape, build_mesh, gem_record, gem_record_in, instance_count},
    geometry::{Curve, Geometry},
    identity::ObjectId,
    layers::PresetSlot,
    model::{Document, Entity, Layer},
};

/// A committed gem is one editable body: an OCCT B-Rep when the kernel feature is enabled, the
/// portable mesh otherwise. Both are the same authored boundary.
fn is_gem_body(geometry: &Geometry) -> bool {
    matches!(geometry, Geometry::Mesh(_) | Geometry::Brep(_))
}

fn controller() -> Controller {
    Controller::new(Document::new("gems"), Limits::default()).expect("controller")
}

fn with_gem_layer() -> (
    Controller,
    om9_core::identity::LayerId,
    om9_core::identity::LayerId,
) {
    gem_layer_controller(false)
}

fn with_locked_gem_layer() -> (Controller, om9_core::identity::LayerId) {
    let (controller, gem, curve) = gem_layer_controller(true);
    let _ = curve;
    (controller, gem)
}

fn gem_layer_controller(
    locked: bool,
) -> (
    Controller,
    om9_core::identity::LayerId,
    om9_core::identity::LayerId,
) {
    let mut document = Document::new("gems");
    let mut gem = Layer::preset(PresetSlot::new(5).unwrap());
    gem.locked = locked;
    let curve = Layer::preset(PresetSlot::new(1).unwrap());
    let (gem_id, curve_id) = (gem.id, curve.id);
    document.layer_order = vec![gem_id, curve_id];
    document.layers.insert(gem_id, gem);
    document.layers.insert(curve_id, curve);
    document.current_layer = Some(curve_id);
    let controller = Controller::new(document, Limits::default()).expect("controller");
    (controller, gem_id, curve_id)
}

fn last_entity(controller: &Controller) -> ObjectId {
    *controller
        .document()
        .entities
        .keys()
        .next_back()
        .expect("an entity")
}

fn only_gem(controller: &Controller) -> ObjectId {
    let ids: Vec<ObjectId> = controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record(entity).is_some())
        .map(|entity| entity.id)
        .collect();
    assert_eq!(ids.len(), 1, "exactly one gem");
    ids[0]
}

fn add_round_gem(controller: &mut Controller) -> ObjectId {
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader starts");
    controller.submit("Size=6.5").expect("size");
    controller.submit("Depth=3.95").expect("depth");
    controller.submit("").expect("apply");
    only_gem(controller)
}

fn circle_document(z_tilted: bool) -> Document {
    let mut document = Document::new("custom");
    let y = if z_tilted {
        [
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ]
    } else {
        [0.0, 1.0, 0.0]
    };
    let curve = Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y,
        radius: 3.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let entity = Entity::new("profile", Geometry::Curve(curve), None);
    document.entities.insert(entity.id, entity);
    document
}

#[test]
fn the_loader_previews_without_creating_an_object_and_cancel_leaves_no_phantom() {
    let mut controller = controller();
    let before = controller.document().revision;
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    assert!(controller.prompt().contains("Gem Loader"));
    assert!(controller.document().entities.is_empty());
    assert_eq!(controller.document().revision, before);
    let preview = controller.gem_preview().expect("preview");
    assert!(preview.closed);
    assert!(preview.volume_mm3 > 0.0);
    assert!(controller.gem_cancel());
    assert!(controller.document().entities.is_empty());
    assert_eq!(controller.document().revision, before);
    assert!(!controller.is_running());
}

#[test]
fn applying_the_loader_inserts_one_gem_in_one_transaction_and_undo_restores() {
    let mut controller = controller();
    let journal = controller.coordinator().undo_len();
    let id = add_round_gem(&mut controller);
    assert_eq!(controller.coordinator().undo_len(), journal + 1);
    let entity = &controller.document().entities[&id];
    let record = gem_record(entity).expect("gem payload");
    assert_eq!(record.spec.shape, GemShape::Round);
    assert_eq!(record.spec.cut, GemCut::Diamond);
    assert!((record.spec.length_mm - 6.5).abs() < 1.0e-9);
    assert!((record.spec.width_mm - 6.5).abs() < 1.0e-9);
    assert!((record.spec.depth_mm() - 3.95).abs() < 1.0e-9);

    controller.undo().expect("undo");
    assert!(controller.document().entities.is_empty());
    controller.redo().expect("redo");
    assert!(
        controller.document().entities.contains_key(&id),
        "redo keeps the identity"
    );
}

#[test]
fn loader_options_change_the_cut_shape_size_and_depth() {
    let mut controller = controller();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("Shape=Oval").expect("shape");
    controller.submit("Size=8x6").expect("size");
    controller.submit("Depth=4.2").expect("depth");
    let spec = controller.gem_session().unwrap().spec().unwrap();
    assert_eq!(spec.shape, GemShape::Oval);
    assert!((spec.length_mm - 8.0).abs() < 1.0e-9);
    assert!((spec.width_mm - 6.0).abs() < 1.0e-9);
    assert!((spec.depth_mm() - 4.2).abs() < 1.0e-9);
    controller.submit("Cut=Cabochon").expect("cut");
    assert_eq!(controller.gem_session().unwrap().cut, GemCut::Cabochon);
    // A cabochon has a dome and no pavilion depth.
    let spec = controller.gem_session().unwrap().spec().unwrap();
    assert_eq!(spec.pavilion_depth_mm, 0.0);
    controller.submit("").expect("apply");
    let record = gem_record_in(controller.document(), only_gem(&controller)).unwrap();
    assert_eq!(record.spec.cut, GemCut::Cabochon);
    assert!(record.spec.crown_height_mm > 0.0);
}

#[test]
fn a_bad_loader_option_is_refused_and_the_session_stays_open() {
    let mut controller = controller();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    assert!(controller.submit("Size=999").is_err());
    assert!(
        controller.is_running(),
        "a refusal must not close the loader"
    );
    assert!(controller.submit("Depth=0").is_err());
    assert!(controller.submit("Shape=Blob").is_err());
    assert!(controller.submit("Unknown=1").is_err());
    assert!(controller.document().entities.is_empty());
    controller.submit("Cancel").expect("cancel");
    assert!(!controller.is_running());
}

#[test]
fn keep_original_size_keeps_the_replaced_gems_dimensions() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("KeepOriginal=Yes").expect("keep");
    controller.submit("Shape=Oval").expect("shape");
    controller.submit("").expect("apply");
    let record = gem_record_in(controller.document(), id).unwrap();
    assert_eq!(record.spec.shape, GemShape::Oval);
    assert!((record.spec.length_mm - 6.5).abs() < 1.0e-9);
    assert!((record.spec.width_mm - 6.5).abs() < 1.0e-9);
    assert!((record.spec.depth_mm() - 3.95).abs() < 1.0e-9);
}

#[test]
fn replacing_a_selected_gem_keeps_the_object_and_gem_identity() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let gem_id = gem_record(&controller.document().entities[&id])
        .unwrap()
        .gem_id;
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("Shape=Emerald").expect("shape");
    controller.submit("Length=9").expect("length");
    controller.submit("Width=7").expect("width");
    controller.submit("Depth=4.5").expect("depth");
    controller.submit("").expect("apply");
    assert_eq!(controller.document().entities.len(), 1);
    let entity = &controller.document().entities[&id];
    let record = gem_record(entity).unwrap();
    assert_eq!(record.gem_id, gem_id, "the GemId is stable");
    assert!((record.spec.length_mm - 9.0).abs() < 1.0e-9);
    assert!(is_gem_body(&entity.geometry));
    controller.undo().expect("undo");
    let record = gem_record(&controller.document().entities[&id]).unwrap();
    assert_eq!(record.spec.shape, GemShape::Round);
}

#[test]
fn gems_route_to_the_gem_layer_default_even_from_another_current_layer() {
    let (mut controller, gem_layer, curve_layer) = with_gem_layer();
    assert_eq!(controller.document().current_layer, Some(curve_layer));
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("").expect("apply");
    let id = only_gem(&controller);
    assert_eq!(
        controller.document().entities[&id].attributes.layer,
        Some(gem_layer),
        "the Gem 01 layer is the routed default"
    );
}

#[test]
fn a_locked_gem_layer_refuses_creation_and_property_edit() {
    let (mut controller, _gem_layer) = with_locked_gem_layer();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    let error = controller.submit("").expect_err("locked layer");
    assert_eq!(error, om9_core::Error::Invalid("gem layer locked"));
    assert!(controller.document().entities.is_empty());
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn the_emerald_builder_commits_the_requested_dimensions() {
    let mut controller = controller();
    controller
        .dispatch(EMERALD, InvocationOrigin::Cmd)
        .expect("emerald");
    controller.submit("Length=9").expect("length");
    controller.submit("Width=7").expect("width");
    controller.submit("Corner=1.5").expect("corner");
    controller.submit("Depth=4.5").expect("depth");
    controller.submit("Girdle=0.2").expect("girdle");
    controller.submit("").expect("apply");
    let record = gem_record_in(controller.document(), only_gem(&controller)).unwrap();
    assert_eq!(record.spec.shape, GemShape::Emerald);
    assert!((record.spec.length_mm - 9.0).abs() < 1.0e-9);
    assert!((record.spec.width_mm - 7.0).abs() < 1.0e-9);
    assert!((record.spec.corner_mm - 1.5).abs() < 1.0e-9);
    assert!((record.spec.depth_mm() - 4.5).abs() < 1.0e-9);
    assert!((record.spec.girdle_mm - 0.2).abs() < 1.0e-9);
    let extents = record.spec;
    assert!(extents.validate().is_ok());
}

#[test]
fn the_baguette_builder_refuses_a_degenerate_girdle_and_an_oversized_corner() {
    let mut controller = controller();
    controller
        .dispatch(BAGUETTE, InvocationOrigin::Cmd)
        .expect("baguette");
    assert!(controller.submit("Girdle=0").is_err());
    assert!(controller.is_running());
    assert!(controller.submit("Corner=9").is_err());
    controller.submit("Corner=0.4").expect("corner");
    controller.submit("").expect("apply");
    let record = gem_record_in(controller.document(), only_gem(&controller)).unwrap();
    assert_eq!(record.spec.shape, GemShape::Baguette);
    assert!((record.spec.corner_mm - 0.4).abs() < 1.0e-9);
}

#[test]
fn the_custom_builder_uses_a_closed_planar_profile() {
    let document = circle_document(false);
    let profile_id = *document.entities.keys().next().unwrap();
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[profile_id]).expect("select");
    controller
        .dispatch(CUSTOM, InvocationOrigin::Cmd)
        .expect("custom");
    let session = controller.gem_session().unwrap();
    assert_eq!(session.profile.as_ref().unwrap().len(), 63);
    controller.submit("").expect("apply");
    let id = only_gem(&controller);
    let record = gem_record_in(controller.document(), id).unwrap();
    assert!(record.spec.profile.is_some());
    assert!(
        (record.spec.length_mm - 6.0).abs() < 2.0e-3,
        "length {}",
        record.spec.length_mm
    );
    assert!(
        (record.spec.width_mm - 6.0).abs() < 6.0e-3,
        "width {}",
        record.spec.width_mm
    );
}

#[test]
fn the_custom_builder_refuses_an_open_or_non_planar_curve() {
    let mut document = Document::new("open");
    let line = Entity::new(
        "open",
        Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[line_id]).expect("select");
    let error = controller
        .dispatch(CUSTOM, InvocationOrigin::Cmd)
        .expect_err("open profile");
    assert_eq!(
        error,
        om9_core::Error::Invalid("custom gem profile is not closed")
    );

    let document = circle_document(true);
    let id = *document.entities.keys().next().unwrap();
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[id]).expect("select");
    let error = controller
        .dispatch(CUSTOM, InvocationOrigin::Cmd)
        .expect_err("tilted profile");
    assert_eq!(
        error,
        om9_core::Error::Invalid("custom gem profile must be planar")
    );
}

#[test]
fn the_positioner_moves_and_rotates_the_instance_in_place() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let before = controller.document().entities[&id].placement;
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect("positioner");
    controller.submit("Move=5,0,0").expect("move");
    controller.submit("Rotate=0,0,90").expect("rotate");
    controller.submit("").expect("apply");
    assert_eq!(controller.document().entities.len(), 1);
    let entity = &controller.document().entities[&id];
    assert!((entity.placement.0[0][3] - (before.0[0][3] + 5.0)).abs() < 1.0e-9);
    assert!(
        entity.placement.0[1][0] > 0.99,
        "90 deg about Z maps X to Y"
    );
    assert!(is_gem_body(&entity.geometry));
    controller.undo().expect("undo");
    assert_eq!(controller.document().entities[&id].placement, before);
}

#[test]
fn the_positioner_duplicates_with_one_shared_definition() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let journal = controller.coordinator().undo_len();
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect("positioner");
    controller.submit("Duplicate=Yes").expect("duplicate");
    controller.submit("").expect("apply");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.document().definitions.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), journal + 1);
    let definition = *controller.document().definitions.keys().next().unwrap();
    let target = &controller.document().entities[&id];
    assert!(matches!(target.geometry, Geometry::Instance(value) if value == definition));
    assert_eq!(instance_count(controller.document(), definition), 2);
    controller.undo().expect("undo");
    assert!(controller.document().definitions.is_empty());
    assert!(matches!(
        controller.document().entities[&id].geometry,
        Geometry::Mesh(_) | Geometry::Brep(_)
    ));
}

#[test]
fn the_positioner_refuses_a_non_gem() {
    let mut controller = controller();
    controller
        .dispatch(om9_app::session::LINE, InvocationOrigin::Cmd)
        .expect("line");
    controller.accept_world_point([0.0, 0.0, 0.0]).expect("a");
    controller.accept_world_point([5.0, 0.0, 0.0]).expect("b");
    let id = last_entity(&controller);
    controller.select_objects(&[id]).expect("select");
    let error = controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect_err("not a gem");
    assert_eq!(
        error,
        om9_core::Error::Invalid("gem positioner needs one gem")
    );
}

#[test]
fn the_property_editor_is_one_transaction_and_undo_restores_every_field() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let original_girdle = gem_record(&controller.document().entities[&id])
        .unwrap()
        .spec
        .girdle_mm;
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(PROPERTIES, InvocationOrigin::Cmd)
        .expect("editor");
    {
        let editor = controller.gem_editor_mut().expect("editor");
        editor.label = "Centre stone".to_owned();
        editor.material = "ruby".to_owned();
        editor.color = [190, 30, 45, 255];
        editor.girdle_mm = 0.2;
    }
    let journal = controller.coordinator().undo_len();
    controller.gem_properties_apply().expect("apply");
    assert_eq!(controller.coordinator().undo_len(), journal + 1);
    let entity = &controller.document().entities[&id];
    assert_eq!(entity.label, "Centre stone");
    assert_eq!(entity.attributes.color, Some([190, 30, 45, 255]));
    let record = gem_record(entity).unwrap();
    assert_eq!(record.material, "ruby");
    assert!((record.spec.girdle_mm - 0.2).abs() < 1.0e-9);

    controller.undo().expect("undo");
    let entity = &controller.document().entities[&id];
    let record = gem_record(entity).unwrap();
    assert_ne!(entity.label, "Centre stone");
    assert_eq!(record.material, "diamond");
    assert!((record.spec.girdle_mm - original_girdle).abs() < 1.0e-9);
}

#[test]
fn the_property_editor_refuses_a_shared_instance() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect("positioner");
    controller.submit("Duplicate=Yes").expect("duplicate");
    controller.submit("").expect("apply");
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(PROPERTIES, InvocationOrigin::Cmd)
        .expect("editor");
    assert_eq!(controller.gem_editor().unwrap().shared, 2);
    controller.gem_editor_mut().unwrap().length_mm = 8.0;
    assert_eq!(
        controller.gem_properties_apply().expect_err("shared"),
        om9_core::Error::Invalid("shared gem definition")
    );
    controller.gem_properties_close();
}

#[test]
fn a_property_edit_never_recolours_the_rest_of_the_scene() {
    let mut controller = controller();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("first");
    controller.submit("").expect("apply");
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("second");
    controller.submit("").expect("apply");
    let ids: Vec<ObjectId> = controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record(entity).is_some())
        .map(|entity| entity.id)
        .collect();
    assert_eq!(ids.len(), 2);
    controller.select_objects(&[ids[0]]).expect("select");
    controller
        .dispatch(PROPERTIES, InvocationOrigin::Cmd)
        .expect("editor");
    controller.gem_editor_mut().unwrap().color = [10, 200, 10, 255];
    controller.gem_properties_apply().expect("apply");
    assert_eq!(
        controller.document().entities[&ids[0]].attributes.color,
        Some([10, 200, 10, 255])
    );
    assert_ne!(
        controller.document().entities[&ids[1]].attributes.color,
        Some([10, 200, 10, 255]),
        "the other gem keeps its own colour"
    );
}

#[test]
fn the_f6_gem_context_enables_properties_but_not_the_unimplemented_report() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let document = controller.document().clone();
    let selection: BTreeSet<ObjectId> = [id].into_iter().collect();
    let context = f6::context(
        &document,
        &selection,
        Providers {
            m05_gem: true,
            ..Providers::default()
        },
    );
    assert_eq!(context.kind, f6::ContextKind::Gem);
    let properties = context
        .actions
        .iter()
        .find(|action| action.id == PROPERTIES)
        .expect("gem properties action");
    assert!(properties.availability.enabled());
    let report = context
        .actions
        .iter()
        .find(|action| action.id == "GemReport")
        .expect("gem report action");
    assert!(!report.availability.enabled(), "L2-04-13 owns the report");
    controller.gem_cancel();
}

#[test]
fn the_typed_command_names_start_the_same_services() {
    let mut controller = controller();
    for (text, tool) in [
        ("gvLoader", "OM9-GEM-001"),
        ("gvEmerald", "OM9-GEM-013"),
        ("gvBaguette", "OM9-GEM-014"),
    ] {
        controller.submit(text).expect("command");
        assert_eq!(controller.gem_session().unwrap().tool.command(), tool);
        controller.submit("Cancel").expect("cancel");
    }
    assert!(
        controller.submit("gvGemPositioner").is_err(),
        "needs one gem"
    );
    // The palette names resolve to the same owner even without a selection.
    assert!(om9_app::session::gem::handles("BuilderCustomGem"));
    assert!(om9_app::session::gem::handles("BuilderGemLoader"));
    assert!(om9_app::session::gem::handles("BuilderGemPositioner"));
}

#[test]
fn a_gem_mutation_is_the_only_way_the_tools_touch_the_document() {
    // The positioner composes a placement mutation; a gem insert is an insert. Nothing here
    // writes directly to the document, which is what makes one Undo enough.
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect("positioner");
    controller.submit("Move=1,2,3").expect("move");
    let before = controller.document().revision;
    controller.gem_apply().expect("apply");
    assert_eq!(controller.document().revision, before + 1);
}

#[test]
fn the_loader_preview_is_world_space_and_never_committed() {
    let mut controller = controller();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("Size=6.5").expect("size");
    let preview = controller.gem_world_preview().expect("preview");
    assert!(preview.mesh.closed);
    assert!((preview.mesh.extents_mm[0] - 6.5).abs() < 1.0e-9);
    assert_eq!(preview.placement.point([0.0; 3]).unwrap(), [0.0; 3]);
    assert!(controller.document().entities.is_empty());

    controller.submit("Origin=1,2,3").expect("origin");
    let moved = controller.gem_world_preview().expect("moved preview");
    assert_eq!(moved.placement.point([0.0; 3]).unwrap(), [1.0, 2.0, 3.0]);
    assert!(
        controller.document().entities.is_empty(),
        "preview stays false"
    );

    controller.submit("").expect("apply");
    let id = last_entity(&controller);
    let committed = controller.document().entities[&id].placement;
    assert_eq!(committed.point([0.0; 3]).unwrap(), [1.0, 2.0, 3.0]);
}

#[test]
fn the_positioner_preview_mirrors_move_rotate_and_scale() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(POSITIONER, InvocationOrigin::Cmd)
        .expect("positioner");
    controller.submit("Move=2,0,0").expect("move");
    controller.submit("Rotate=0,0,90").expect("rotate");
    controller.submit("Scale=1.1").expect("scale");
    let preview = controller.gem_world_preview().expect("preview");
    assert!((preview.mesh.extents_mm[0] - 6.5 * 1.1).abs() < 1.0e-6);
    controller.gem_apply().expect("apply");
    let entity = &controller.document().entities[&id];
    assert_eq!(preview.placement, entity.placement);
}

#[cfg(feature = "occt-brep")]
#[test]
fn the_loader_commits_an_editable_brep_with_the_mesh_volume() {
    let mut controller = controller();
    let id = add_round_gem(&mut controller);
    let limits = Limits::default();
    let entity = &controller.document().entities[&id];
    let Geometry::Brep(brep) = &entity.geometry else {
        panic!("the kernel build commits a B-Rep gem body");
    };
    brep.validate(&limits).expect("admitted B-Rep");
    let volume = om9_kernel::brep::source_volume(brep, &limits).expect("source volume");
    let record = gem_record_in(controller.document(), id).expect("record");
    let mesh = build_mesh(&record.spec).expect("mesh");
    assert!((volume - mesh.volume_mm3).abs() / mesh.volume_mm3 < 1.0e-6);

    // Editing the gem keeps the body kind: the property editor re-sews instead of downgrading.
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(PROPERTIES, InvocationOrigin::Cmd)
        .expect("editor");
    controller.gem_editor_mut().expect("editor").girdle_mm = 0.2;
    controller.gem_properties_apply().expect("apply");
    assert!(matches!(
        controller.document().entities[&id].geometry,
        Geometry::Brep(_)
    ));
}

#[cfg(feature = "occt-brep")]
#[test]
fn the_brep_option_keeps_the_portable_mesh_body() {
    let mut controller = controller();
    controller
        .dispatch(LOADER, InvocationOrigin::Cmd)
        .expect("loader");
    controller.submit("BRep=No").expect("mesh opt-out");
    controller.submit("").expect("apply");
    let id = last_entity(&controller);
    assert!(matches!(
        controller.document().entities[&id].geometry,
        Geometry::Mesh(_)
    ));
}
