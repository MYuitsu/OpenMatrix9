// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application contract tests for the pavé/azure/prong/sphere/springs builders (`OM9-L2-04-12`).

use om9_app::session::{
    Controller, InvocationOrigin,
    gem_pave::{PaveKind, PaveTool, reset_session},
};
use om9_core::{
    budget::Limits,
    gem::{GemId, GemRecord, GemSpec, build_mesh, gem_record_in},
    gem_pave::{PaveFeature, PavePattern, PaveStart},
    geometry::{Curve, Geometry, Point3, Transform},
    identity::ObjectId,
    model::{Document, Entity, GEM_EXTENSION_SLOT, GEM_PAVE_EXTENSION_SLOT},
    surface_kernel,
};

fn base_document() -> Document {
    let mut document = Document::new("pave");
    let gem = om9_core::model::Layer::preset(om9_core::layers::PresetSlot::new(5).unwrap());
    let curve = om9_core::model::Layer::preset(om9_core::layers::PresetSlot::new(1).unwrap());
    let curve_id = curve.id;
    document.layer_order = vec![gem.id, curve_id];
    document.layers.insert(gem.id, gem);
    document.layers.insert(curve_id, curve);
    document.current_layer = Some(curve_id);
    document
}

fn add_gem(document: &mut Document, center: Point3) -> ObjectId {
    let spec = GemSpec::round(2.0, 1.3).expect("spec");
    let mesh = build_mesh(&spec).expect("mesh");
    let mut record = GemRecord::new(spec);
    record.gem_id = GemId::new();
    let mut entity = Entity::new("Round", Geometry::Mesh(mesh.mesh), None);
    entity.placement = Transform::translation(center);
    entity.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        record.as_extension().unwrap(),
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn add_surface(document: &mut Document, size: f64) -> ObjectId {
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        size,
        size,
        &Limits::default(),
    )
    .expect("plane");
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn add_curve(document: &mut Document, curve: Curve) -> ObjectId {
    let entity = Entity::new("guide", Geometry::Curve(curve), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn feature(controller: &Controller, surface: ObjectId) -> Option<PaveFeature> {
    controller
        .document()
        .entities
        .get(&surface)?
        .extensions
        .get(GEM_PAVE_EXTENSION_SLOT)
        .and_then(|extension| PaveFeature::from_extension(extension).ok())
}

fn gem_count(controller: &Controller) -> usize {
    controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record_in(controller.document(), entity.id).is_some())
        .count()
}

#[test]
fn auto_pave_previews_and_commits_one_feature() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 20.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    let session = controller.gem_pave_session().expect("session");
    assert!(session.preview_count() > 20);
    let before = controller.document().entities.len();
    controller.submit("spacing=2.2").expect("option");
    controller.submit("").expect("apply");
    let stored = feature(&controller, surface).expect("feature");
    assert!(stored.gems.len() > 20);
    assert_eq!(stored.gems.len(), stored.uvs.len());
    assert!(controller.document().entities.len() > before);
    for id in &stored.gems {
        assert!(controller.document().entities.contains_key(&id.object()));
    }
}

#[test]
fn cancel_and_bad_selection_leave_the_document_untouched() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 10.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    let before = controller.document().entities.len();
    assert!(controller.gem_pave_cancel());
    assert_eq!(controller.document().entities.len(), before);
    assert!(feature(&controller, surface).is_none());

    controller.clear_selection();
    assert!(
        controller
            .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
            .is_err()
    );
}

#[test]
fn undo_redo_restore_the_whole_pave_layout() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 16.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    controller.submit("").expect("apply");
    let count = gem_count(&controller);
    assert!(count > 20);
    controller.undo().expect("undo");
    assert_eq!(gem_count(&controller), 0);
    assert!(feature(&controller, surface).is_none());
    controller.redo().expect("redo");
    assert_eq!(gem_count(&controller), count);
    assert!(feature(&controller, surface).is_some());
}

#[test]
fn editing_the_builder_reuses_existing_child_ids() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 14.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    controller.submit("spacing=2.4").expect("option");
    controller.submit("").expect("apply");
    let first = feature(&controller, surface).unwrap().gems;
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("reopen");
    controller.submit("spacing=2.6").expect("option");
    controller.submit("").expect("apply");
    let second = feature(&controller, surface).unwrap().gems;
    assert!(!second.is_empty());
    assert_eq!(first.len().min(second.len()), second.len().min(first.len()));
    assert_eq!(second[0], first[0]);
    assert_eq!(gem_count(&controller), second.len());
}

#[test]
fn manual_pave_honours_the_exact_quantity() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 16.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Manual.command(), InvocationOrigin::Cmd)
        .expect("manual opens");
    controller.submit("count=12").expect("count");
    let session = controller.gem_pave_session().expect("session");
    assert_eq!(session.preview_count(), 12);
    controller.submit("").expect("apply");
    assert_eq!(feature(&controller, surface).unwrap().gems.len(), 12);
    controller
        .dispatch(PaveTool::Manual.command(), InvocationOrigin::Cmd)
        .expect("reopen manual");
    let reopened = controller.gem_pave_session().expect("session");
    assert_eq!(reopened.quantity, Some(12));
    assert_eq!(reopened.preview_count(), 12);
    controller
        .gem_pave_apply_style("Classic Hex")
        .expect("built-in style");
    assert_eq!(
        controller.gem_pave_session().expect("session").quantity,
        Some(12),
        "an Auto style must not erase the Manual count"
    );
}

#[test]
fn refreshing_a_feature_preserves_the_child_ids() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 16.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    controller.submit("spacing=2.2").expect("option");
    controller.submit("").expect("apply");
    let first = feature(&controller, surface).unwrap().gems;
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("reopen");
    controller.gem_pave_refresh().expect("refresh");
    controller.submit("").expect("apply refreshed");
    let second = feature(&controller, surface).unwrap().gems;
    assert_eq!(first, second);
}

#[test]
fn azure_builder_commits_four_profiles_per_stone() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 10.0);
    let gem = add_gem(&mut document, [2.0, 2.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface, gem]).expect("select");
    controller
        .dispatch(PaveTool::Azure.command(), InvocationOrigin::Cmd)
        .expect("azure opens");
    assert_eq!(controller.gem_pave_session().unwrap().preview_count(), 1);
    let before = controller.document().entities.len();
    controller.submit("hex=on").expect("option");
    controller.submit("").expect("apply");
    assert_eq!(controller.document().entities.len(), before + 4);
    assert!(
        controller
            .document()
            .entities
            .values()
            .any(|entity| entity.label.starts_with("Azure Top"))
    );
}

#[test]
fn prong_builder_commits_markers_and_refuses_no_gem() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 10.0);
    let a = add_gem(&mut document, [2.0, 2.0, 0.0]);
    let b = add_gem(&mut document, [3.7, 2.0, 0.0]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface, a, b]).expect("select");
    controller
        .dispatch(PaveTool::Prong.command(), InvocationOrigin::Cmd)
        .expect("prong opens");
    let before = controller.document().entities.len();
    controller.submit("").expect("apply");
    assert!(controller.document().entities.len() > before);
    assert!(
        controller
            .document()
            .entities
            .values()
            .any(|entity| entity.label.starts_with("Pavé Prong"))
    );
    controller.clear_selection();
    assert!(
        controller
            .dispatch(PaveTool::Prong.command(), InvocationOrigin::Cmd)
            .is_err()
    );
}

#[test]
fn sphere_builder_previews_and_commits_without_a_surface() {
    let document = base_document();
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller
        .dispatch(PaveTool::Sphere.command(), InvocationOrigin::Cmd)
        .expect("sphere opens");
    controller.submit("count=24").expect("count");
    let session = controller.gem_pave_session().expect("session");
    assert_eq!(session.preview_count(), 24);
    controller.submit("").expect("apply");
    assert_eq!(gem_count(&controller), 24);
}

#[test]
fn gem_springs_move_stones_and_undo_restores_them() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 12.0);
    let a = add_gem(&mut document, [2.0, 2.0, 0.0]);
    let b = add_gem(&mut document, [2.3, 2.0, 0.0]);
    let before = controller_centers(&document, &[a, b]);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface, a, b]).expect("select");
    controller
        .dispatch(PaveTool::Springs.command(), InvocationOrigin::Cmd)
        .expect("springs opens");
    controller.submit("").expect("apply");
    let after = controller_centers(controller.document(), &[a, b]);
    assert!(
        distance(after[0], after[1]) > distance(before[0], before[1]),
        "{before:?} -> {after:?}"
    );
    controller.undo().expect("undo");
    let restored = controller_centers(controller.document(), &[a, b]);
    assert!((distance(restored[0], restored[1]) - distance(before[0], before[1])).abs() < 1.0e-9);
}

fn controller_centers(document: &Document, ids: &[ObjectId]) -> Vec<Point3> {
    ids.iter()
        .map(|id| {
            let placement = document.entities.get(id).unwrap().placement;
            [placement.0[0][3], placement.0[1][3], placement.0[2][3]]
        })
        .collect()
}

fn distance(a: Point3, b: Point3) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}

#[test]
fn typed_commands_and_reset_share_the_same_handler() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 12.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller.select_objects(&[surface]).expect("select");
    controller.submit("gvPaveDialog").expect("typed auto");
    assert!(controller.gem_pave_session().is_some());
    controller.submit("cancel").expect("cancel");
    controller.submit("gvPaveBuilder").expect("typed manual");
    assert!(controller.gem_pave_session().is_some());
    controller.submit("cancel").expect("cancel");
    controller.submit("gvPaveSphere").expect("typed sphere");
    assert!(controller.gem_pave_session().is_some());
    controller.submit("cancel").expect("cancel");
    controller.select_objects(&[surface]).expect("select");
    controller.submit("gvPaveDialog").expect("typed auto");
    let mut session = controller.gem_pave_session().unwrap().clone();
    assert_eq!(session.kind.stone_pattern(), Some(PavePattern::Hex));
    session.quantity = Some(4);
    let reset = reset_session(&session).unwrap();
    assert_eq!(reset.quantity, None);
}

#[test]
fn selected_curves_supply_the_attractor_and_closed_edges() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 10.0);
    let attractor = add_curve(
        &mut document,
        Curve::Line {
            start: [5.0, 0.0, 0.0],
            end: [5.0, 10.0, 0.0],
            domain: [0.0, 1.0],
        },
    );
    let edge = add_curve(
        &mut document,
        om9_core::gem_pave::closed_polygon(&[
            [2.0, 2.0, 0.0],
            [8.0, 2.0, 0.0],
            [8.0, 8.0, 0.0],
            [2.0, 8.0, 0.0],
        ])
        .expect("edge"),
    );
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller
        .select_objects(&[surface, attractor, edge])
        .expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    let session = controller.gem_pave_session().expect("session");
    let PaveKind::Stone(params) = &session.kind else {
        panic!("stone session");
    };
    assert!(params.attractor.is_some());
    assert_eq!(params.edge_curves.len(), 1);
    assert_eq!(params.start, PaveStart::Curve);
    assert_eq!(session.attractor_source, Some(attractor));
    assert_eq!(session.edge_sources, vec![edge]);
    assert!(session.preview_count() > 0);
    for station in &session.layout.as_ref().expect("layout").stations {
        let [x, y, _] = station.point;
        assert!(x > 2.0 && x < 8.0 && y > 2.0 && y < 8.0);
    }
    controller.submit("").expect("apply");
    let stored = feature(&controller, surface).expect("feature");
    assert_eq!(stored.attractor_source, Some(attractor));
    assert_eq!(stored.edge_sources, vec![edge]);
}

#[test]
fn pave_style_save_and_apply_restore_the_stone_parameters() {
    let mut document = base_document();
    let surface = add_surface(&mut document, 12.0);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[surface]).expect("select");
    controller
        .dispatch(PaveTool::Auto.command(), InvocationOrigin::Cmd)
        .expect("auto opens");
    controller.submit("spacing=2.6").expect("spacing");
    controller
        .gem_pave_save_style("Tight test")
        .expect("save style");
    assert!(
        controller
            .gem_pave_style_names()
            .iter()
            .any(|name| name == "Tight test")
    );
    controller.submit("spacing=1.2").expect("spacing");
    controller
        .gem_pave_apply_style("Tight test")
        .expect("apply style");
    let session = controller.gem_pave_session().expect("session");
    let PaveKind::Stone(params) = &session.kind else {
        panic!("stone session");
    };
    assert!((params.spacing_mm - 2.6).abs() < 1.0e-9);
    assert!(controller.gem_pave_apply_style("Classic Hex").is_ok());
    assert!(controller.gem_pave_apply_style("missing style").is_err());
}

#[test]
fn sphere_handles_drag_radius_and_count_through_the_shared_builder_route() {
    let mut controller = Controller::new(base_document(), Limits::default()).expect("controller");
    controller.clear_selection();
    controller
        .dispatch(PaveTool::Sphere.command(), InvocationOrigin::Cmd)
        .expect("sphere opens");
    let handles = controller.builder_handles();
    assert_eq!(handles.len(), 2);
    let before = controller
        .gem_pave_session()
        .expect("session")
        .preview_count();
    let radius = handles
        .iter()
        .find(|handle| handle.field == "radius")
        .expect("radius handle");
    let step = radius.max - radius.value;
    controller
        .builder_adjust_field("radius", step)
        .expect("radius drag");
    let session = controller.gem_pave_session().expect("session");
    let PaveKind::Sphere(params) = &session.kind else {
        panic!("sphere session");
    };
    assert!((params.radius_mm - 100.0).abs() < 1.0e-9);
    controller
        .builder_adjust_field("count", 20.0)
        .expect("count drag");
    let session = controller.gem_pave_session().expect("session");
    let PaveKind::Sphere(params) = &session.kind else {
        panic!("sphere session");
    };
    assert_eq!(params.count, Some(140));
    assert!(session.preview_count() >= before);
}
