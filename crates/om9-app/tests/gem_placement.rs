// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application contract tests for gems on curves and surfaces (`OM9-L2-04-10`).
//!
//! Every test drives the same command surface the palette icons and the command line use. The
//! geometry oracle lives in `crates/om9-core/tests/gem_placement.rs`; this file proves the
//! session wiring: preview without an object, one-transaction Apply, Undo/Redo, Cancel without a
//! phantom, follow conflicts, flow/splop retargeting, profile/guide insertion and the persisted
//! `om9.gem.placement` link.

use std::collections::BTreeSet;

use om9_app::session::{
    Controller, InvocationOrigin,
    gem_placement::{
        COUNT_ON_CURVE, FLOW, FOLLOW, GUIDES, LIST_ON_CURVE, ON_CURVE, ON_CURVE_ADVANCED,
        ON_CURVE_MULTI, ON_SURFACE, PROFILE, SPLOP, command_name,
    },
};
use om9_core::{
    budget::Limits,
    gem::{GemId, GemRecord, gem_record},
    gem_placement::{PlacementLink, PlacementSource},
    geometry::{Curve, Geometry, NurbsSurface, Transform},
    identity::ObjectId,
    model::{Document, Entity, GEM_PLACEMENT_EXTENSION_SLOT},
};

fn controller_with(entities: Vec<Entity>) -> Controller {
    let mut document = Document::with_template_tolerance("placement");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).expect("controller")
}

fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
    let curve = Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    };
    Entity::new(name, Geometry::Curve(curve), None)
}

fn plane(name: &str, z: f64) -> Entity {
    let limits = Limits::default();
    let surface: NurbsSurface = om9_core::surface_kernel::plane(
        [0.0, 0.0, z],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        100.0,
        100.0,
        &limits,
    )
    .expect("plane");
    Entity::new(name, Geometry::Surface(surface), None)
}

fn gem_ids(controller: &Controller) -> Vec<ObjectId> {
    controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record(entity).is_some())
        .map(|entity| entity.id)
        .collect()
}

fn record(controller: &Controller, id: ObjectId) -> GemRecord {
    gem_record(&controller.document().entities[&id]).expect("a gem record")
}

fn link(controller: &Controller, id: ObjectId) -> PlacementLink {
    let extension = controller.document().entities[&id]
        .extensions
        .get(GEM_PLACEMENT_EXTENSION_SLOT)
        .expect("a placement link");
    PlacementLink::from_extension(extension).expect("a readable link")
}

fn axis_z(transform: Transform) -> [f64; 3] {
    [transform.0[2][0], transform.0[2][1], transform.0[2][2]]
}

/// Place three round gems on a 100 mm line at a chosen spacing.
fn place_on_line(controller: &mut Controller, line_id: ObjectId, spacing: &str) -> Vec<ObjectId> {
    controller.select_objects(&[line_id]).expect("select line");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    controller.submit(&spacing.replace("Spacing=", "Step=")).expect("spacing");
    controller.submit("").expect("apply");
    gem_ids(controller)
}

#[test]
fn the_row_previews_without_an_object_and_applies_in_one_transaction() {
    let line = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    assert!(controller.is_running());
    controller.submit("Size=6").unwrap();
    controller.submit("Step=8").unwrap();
    let preview = controller.gem_placement_session().expect("session").preview_count();
    assert_eq!(preview, 13, "explicit 8 mm centre step starts at 0 and ends at 96");
    assert_eq!(
        controller.document().entities.len(),
        1,
        "a preview never inserts an object"
    );
    controller.submit("Step=10").expect("spacing");
    assert_eq!(
        controller.gem_placement_session().expect("session").preview_count(),
        11
    );
    controller.submit("").expect("apply");
    assert!(!controller.is_running());
    let gems = gem_ids(&controller);
    assert_eq!(gems.len(), 11);
    for id in &gems {
        let entity = &controller.document().entities[id];
        assert!(matches!(entity.geometry, Geometry::Mesh(_)));
        let link = link(&controller, *id);
        assert_eq!(link.source, PlacementSource::Curve { id: line_id });
        assert_eq!(link.state, om9_core::gem_placement::LinkState::Active);
        assert!(link.arc_length_mm >= 0.0 && link.arc_length_mm <= 100.0);
    }
    // One transaction: a single Undo removes the whole row and a single Redo restores it.
    controller.undo().expect("undo");
    assert!(gem_ids(&controller).is_empty());
    assert_eq!(controller.document().entities.len(), 1);
    controller.redo().expect("redo");
    assert_eq!(gem_ids(&controller).len(), 11);
}

#[test]
fn the_advanced_tool_tapers_the_row_from_the_start_size_to_the_end_size() {
    let line = line("path", [0.0, 0.0, 0.0], [90.0, 0.0, 0.0]);
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    controller
        .dispatch(ON_CURVE_ADVANCED, InvocationOrigin::Cmd)
        .expect("advanced");
    controller.submit("Spacing=30").expect("spacing");
    controller.submit("Size=6").expect("start size");
    controller.submit("EndSize=2").expect("end size");
    controller.submit("").expect("apply");
    let gems = gem_ids(&controller);
    assert_eq!(gems.len(), 4, "0/30/60/90 mm");
    let mut row: Vec<(f64, f64)> = gems
        .iter()
        .map(|id| {
            (
                link(&controller, *id).arc_length_mm,
                record(&controller, *id).spec.length_mm,
            )
        })
        .collect();
    row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!((row[0].1 - 6.0).abs() < 1.0e-9, "the first stone keeps the start size");
    assert!((row[3].1 - 2.0).abs() < 1.0e-9, "the last stone reaches the end size");
    assert!(
        row.windows(2).all(|pair| pair[0].1 >= pair[1].1 - 1.0e-9),
        "the taper is monotonic: {row:?}"
    );
}

#[test]
fn cancel_drops_the_session_and_leaves_no_phantom() {
    let line = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    controller.submit("Cancel").expect("cancel");
    assert!(!controller.is_running());
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.prompt(), "Command: ");
}

#[test]
fn the_command_can_pick_a_curve_after_starting() {
    let path = line("a", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    controller.dispatch(ON_CURVE, InvocationOrigin::Cmd).unwrap();
    assert!(controller.gem_placement_needs_target().is_some());
    assert!(controller.submit("").is_err());
    controller.gem_placement_pick_target(path_id).unwrap();
    assert!(controller.gem_placement_needs_target().is_none());
    assert!(controller.gem_placement_session().unwrap().preview_count() > 0);
}

#[test]
fn the_count_tool_fits_the_row_between_the_curve_ends() {
    let line = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    controller
        .dispatch(COUNT_ON_CURVE, InvocationOrigin::Cmd)
        .expect("count on curve");
    controller.submit("Size=6").expect("size");
    controller.submit("Gap=2").expect("gap");
    controller.submit("").expect("apply");
    assert_eq!(gem_ids(&controller).len(), 12, "6 mm stones with a 2 mm gap fit 12");
    let mut arcs: Vec<f64> = gem_ids(&controller)
        .iter()
        .map(|id| link(&controller, *id).arc_length_mm)
        .collect();
    arcs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((arcs[0] - 3.0).abs() < 1.0e-6, "the first stone clears the end");
    assert!((arcs[11] - 91.0).abs() < 1.0e-6, "the last stone clears the end");
}

#[test]
fn the_list_tool_places_heterogeneous_stones_in_order() {
    let line = line("path", [0.0, 0.0, 0.0], [30.0, 0.0, 0.0]);
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    controller
        .dispatch(LIST_ON_CURVE, InvocationOrigin::Cmd)
        .expect("list on curve");
    controller.submit("Sizes=6,5,4").expect("sizes");
    controller.submit("").expect("apply");
    let mut diameters: Vec<f64> = gem_ids(&controller)
        .iter()
        .map(|id| record(&controller, *id).spec.length_mm)
        .collect();
    diameters.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(diameters, vec![4.0, 5.0, 6.0]);
}

#[test]
fn the_multi_tool_places_one_run_per_curve_in_one_transaction() {
    let a = line("a", [0.0, 0.0, 0.0], [40.0, 0.0, 0.0]);
    let b = line("b", [0.0, 20.0, 0.0], [40.0, 20.0, 0.0]);
    let (a_id, b_id) = (a.id, b.id);
    let mut controller = controller_with(vec![a, b]);
    controller.select_objects(&[a_id, b_id]).expect("select");
    controller
        .dispatch(ON_CURVE_MULTI, InvocationOrigin::Cmd)
        .expect("multi");
    controller.submit("Spacing=10").expect("spacing");
    controller.submit("").expect("apply");
    let gems = gem_ids(&controller);
    assert!(gems.len() >= 8, "one run per curve: {}", gems.len());
    let sources: BTreeSet<ObjectId> = gems
        .iter()
        .map(|id| link(&controller, *id).source.object())
        .collect();
    assert_eq!(sources, BTreeSet::from([a_id, b_id]));
    controller.undo().expect("undo");
    assert!(gem_ids(&controller).is_empty(), "multi is atomic");
}

#[test]
fn the_surface_tool_stands_the_gem_on_the_surface_normal() {
    let surface = plane("floor", 5.0);
    let surface_id = surface.id;
    let mut controller = controller_with(vec![surface]);
    controller.select_objects(&[surface_id]).expect("select");
    controller
        .dispatch(ON_SURFACE, InvocationOrigin::Cmd)
        .expect("on surface");
    controller.submit("Size=6").expect("size");
    controller.submit("Grid=2").expect("grid");
    controller.submit("").expect("apply");
    let gems = gem_ids(&controller);
    assert_eq!(gems.len(), 4, "a 2x2 UV grid is four stations");
    for id in &gems {
        let entity = &controller.document().entities[id];
        let axis = axis_z(entity.placement);
        assert!(
            axis[0].abs() < 1.0e-9 && axis[1].abs() < 1.0e-9 && (axis[2] - 1.0).abs() < 1.0e-9,
            "the table axis follows +Z: {axis:?}"
        );
        let z = entity.placement.point([0.0, 0.0, 0.0]).expect("origin")[2];
        assert!((z - 5.0).abs() < 1.0e-9);
        let link = link(&controller, *id);
        assert!(matches!(link.source, PlacementSource::Surface { id } if id == surface_id));
        assert!(link.uv.is_some(), "a surface link keeps its UV");
    }
}

#[test]
fn flow_carries_the_same_stones_onto_a_target_curve() {
    let source = line("source", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let target = line("target", [0.0, 50.0, 0.0], [100.0, 50.0, 0.0]);
    let (source_id, target_id) = (source.id, target.id);
    let mut controller = controller_with(vec![source, target]);
    let gems = place_on_line(&mut controller, source_id, "Spacing=25");
    assert_eq!(gems.len(), 5);
    let mut selection = gems.clone();
    selection.push(target_id);
    controller.select_objects(&selection).expect("select");
    controller
        .dispatch(FLOW, InvocationOrigin::Cmd)
        .expect("flow");
    controller.submit("").expect("apply");
    for id in &gems {
        let link = link(&controller, *id);
        assert_eq!(link.source, PlacementSource::Curve { id: target_id });
        let y = controller.document().entities[id]
            .placement
            .point([0.0, 0.0, 0.0])
            .expect("origin")[1];
        assert!((y - 50.0).abs() < 1.0e-6, "the stone moved to the target curve");
        assert_eq!(record(&controller, *id).gem_id, GemId(*id));
    }
}

#[test]
fn count_places_an_exact_number_of_stones_between_the_range_ends() {
    let path = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    controller.select_objects(&[path_id]).expect("select");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    controller.submit("Count=5").expect("count");
    assert_eq!(
        controller.gem_placement_session().expect("session").preview_count(),
        5,
        "Count= is an exact station count, not a spacing suggestion"
    );
    controller.submit("").expect("apply");
    let mut arcs: Vec<f64> = gem_ids(&controller)
        .iter()
        .map(|id| link(&controller, *id).arc_length_mm)
        .collect();
    arcs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(arcs.len(), 5);
    for (index, arc) in arcs.iter().enumerate() {
        let expected = 25.0 * index as f64;
        assert!((arc - expected).abs() < 1.0e-6, "{arcs:?} vs {expected}");
    }
}

#[test]
fn an_option_the_tool_does_not_use_is_a_typed_refusal() {
    let path = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    controller.select_objects(&[path_id]).expect("select");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    assert_eq!(
        controller.submit("Sizes=6,5").expect_err("sizes is the list tool"),
        om9_core::Error::Invalid("gem placement option")
    );
    assert!(controller.is_running(), "a refusal keeps the session editable");
    controller.submit("Count=3").expect("count still works");
    assert_eq!(
        controller.gem_placement_session().expect("session").preview_count(),
        3
    );
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn flow_carries_surface_stones_onto_another_surface() {
    let base = plane("base", 0.0);
    let target = plane("target", 10.0);
    let (base_id, target_id) = (base.id, target.id);
    let mut controller = controller_with(vec![base, target]);
    controller.select_objects(&[base_id]).expect("select base");
    controller
        .dispatch(ON_SURFACE, InvocationOrigin::Cmd)
        .expect("gem on surface");
    controller.submit("Grid=2").expect("grid");
    controller.submit("").expect("apply");
    let gems = gem_ids(&controller);
    assert_eq!(gems.len(), 4);
    for id in &gems {
        let z = controller.document().entities[id]
            .placement
            .point([0.0, 0.0, 0.0])
            .expect("origin")[2];
        assert!(z.abs() < 1.0e-9);
    }
    let mut selection = gems.clone();
    selection.push(target_id);
    controller.select_objects(&selection).expect("select");
    controller
        .dispatch(FLOW, InvocationOrigin::Cmd)
        .expect("flow to a surface target");
    controller.submit("").expect("apply");
    for id in &gems {
        let entity = &controller.document().entities[id];
        let z = entity.placement.point([0.0, 0.0, 0.0]).expect("origin")[2];
        assert!((z - 10.0).abs() < 1.0e-6, "the stone landed on the target surface");
        let link = link(&controller, *id);
        assert_eq!(link.source, PlacementSource::Surface { id: target_id });
        assert!(link.uv.is_some(), "a surface flow keeps a usable UV link");
    }
    controller.undo().expect("undo");
    for id in &gems {
        assert_eq!(
            link(&controller, *id).source,
            PlacementSource::Surface { id: base_id }
        );
    }
}

#[test]
fn splop_projects_a_gem_onto_a_surface_and_records_the_uv_link() {
    let path = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let path_id = path.id;
    let floor = plane("floor", 5.0);
    let floor_id = floor.id;
    let mut controller = controller_with(vec![path, floor]);
    let gems = place_on_line(&mut controller, path_id, "Spacing=50");
    assert_eq!(gems.len(), 3);
    let mut selection = gems.clone();
    selection.push(floor_id);
    controller.select_objects(&selection).expect("select");
    controller
        .dispatch(SPLOP, InvocationOrigin::Cmd)
        .expect("splop");
    controller.submit("ZOffset=1").expect("offset");
    controller.submit("").expect("apply");
    for id in &gems {
        let entity = &controller.document().entities[id];
        let origin = entity.placement.point([0.0, 0.0, 0.0]).expect("origin");
        assert!((origin[2] - 6.0).abs() < 1.0e-6, "surface plus offset");
        let axis = axis_z(entity.placement);
        assert!((axis[2] - 1.0).abs() < 1.0e-9);
        let link = link(&controller, *id);
        assert!(matches!(link.source, PlacementSource::Surface { id } if id == floor_id));
        assert!(link.uv.is_some());
    }
}

#[test]
fn follow_marks_gems_whose_source_is_gone_as_conflicts_instead_of_moving_them() {
    let path = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    let gems = place_on_line(&mut controller, path_id, "Spacing=30");
    assert_eq!(gems.len(), 4);
    let before: Vec<[f64; 3]> = gems
        .iter()
        .map(|id| {
            controller.document().entities[id]
                .placement
                .point([0.0, 0.0, 0.0])
                .expect("origin")
        })
        .collect();
    controller.delete_objects(&[path_id]).expect("delete source");
    controller.select_objects(&gems).expect("select gems");
    controller
        .dispatch(FOLLOW, InvocationOrigin::Cmd)
        .expect("follow");
    controller.submit("").expect("apply");
    for (index, id) in gems.iter().enumerate() {
        let link = link(&controller, *id);
        assert_eq!(
            link.state,
            om9_core::gem_placement::LinkState::Conflict,
            "a missing source is a visible conflict"
        );
        let after = controller.document().entities[id]
            .placement
            .point([0.0, 0.0, 0.0])
            .expect("origin");
        assert_eq!(before[index], after, "a conflict keeps the last place");
        let _ = record(&controller, *id);
    }
    // The conflict is one transaction too, so Undo restores the ACTIVE links.
    controller.undo().expect("undo");
    for id in &gems {
        assert_eq!(link(&controller, *id).state, om9_core::gem_placement::LinkState::Active);
    }
}

#[test]
fn profile_and_guides_insert_construction_curves() {
    let path = line("path", [0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    let gems = place_on_line(&mut controller, path_id, "Spacing=30");
    assert_eq!(gems.len(), 4);
    controller.select_objects(&gems).expect("select gems");
    controller
        .dispatch(PROFILE, InvocationOrigin::Cmd)
        .expect("profile");
    assert_eq!(
        controller.gem_placement_session().expect("session").preview_count(),
        6,
        "profile, girdle and one guide per stone"
    );
    controller.submit("").expect("apply");
    let labels: Vec<String> = controller
        .document()
        .entities
        .values()
        .map(|entity| entity.label.clone())
        .collect();
    assert!(labels.iter().any(|label| label == "Gem profile"));
    assert!(labels.iter().any(|label| label == "Gem girdle"));
    controller.select_objects(&gems).expect("select gems");
    controller
        .dispatch(GUIDES, InvocationOrigin::Cmd)
        .expect("guides");
    assert!(controller.gem_placement_session().expect("session").preview_count() > 0);
    controller.submit("").expect("apply guides");
    let guides = controller
        .document()
        .entities
        .values()
        .filter(|entity| entity.label.starts_with("Gem guide"))
        .count();
    assert!(guides > 0, "at least one guide curve");
}

#[test]
fn the_links_report_and_the_typed_command_reach_the_same_service() {
    let path = line("path", [0.0, 0.0, 0.0], [40.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    controller.select_objects(&[path_id]).expect("select");
    // The typed Matrix command opens the same tool as the palette icon.
    controller.submit("gvGemOnCrv").expect("typed command");
    assert!(controller.is_running());
    controller.submit("Step=20").expect("spacing");
    controller.submit("").expect("apply");
    let gems = gem_ids(&controller);
    assert_eq!(gems.len(), 3);
    controller.select_objects(&gems).expect("select gems");
    let report = controller.gem_placement_links_report().expect("report");
    assert_eq!(report.lines().count(), 3);
    assert!(report.contains("curve"));
    assert!(report.contains("arc"));
    // The typed names of the whole family resolve to the same commands.
    assert_eq!(command_name("gvGemFlow"), Some(FLOW));
    assert_eq!(command_name("gvGemOnSurface"), Some(ON_SURFACE));
    assert!(om9_core::model::extension_slot_known(
        om9_core::model::GEM_PLACEMENT_EXTENSION_SLOT
    ));
}


#[test]
fn matrix_macro_fits_clearance_and_f6_edits_the_same_row() {
    let path = line("row", [0., 0., 0.], [30., 0., 0.]);
    let path_id = path.id;
    let mut controller = controller_with(vec![path]);
    controller
        .submit("_gvGemOnCrv _SpacingMethod=_Fixed _Size=4 _Spacing=2")
        .unwrap();
    controller.gem_placement_pick_target(path_id).unwrap();
    let first = controller.gem_placement_session().unwrap().plans[0].stations.as_slice();
    assert_eq!(first.len(), 6);
    assert!((first[1].arc_length_mm - first[0].arc_length_mm - 6.0).abs() < 1e-8);
    controller.submit("").unwrap();
    let before = gem_ids(&controller);
    let filename = std::env::temp_dir().join(format!("om9-gem-row-{}.om9", ObjectId::new()));
    controller.save_archive_for_batch(&filename).unwrap();
    let reopened = Controller::from_file(&filename, Limits::default());
    std::fs::remove_file(filename).unwrap();
    let mut controller = reopened.unwrap();
    controller.select_objects(&[before[0]]).unwrap();
    controller.dispatch("EditGemOnCurve", InvocationOrigin::F6).unwrap();
    controller.submit("End=60%").unwrap();
    controller.submit("").unwrap();
    let after = gem_ids(&controller);
    assert_eq!(after.len(), 4);
    assert!(after.iter().all(|id| before.contains(id)));
    controller.undo().unwrap();
    assert_eq!(gem_ids(&controller), before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn gem_on_surface_uses_forward_source_ray_and_trim_and_commits_one_batch() {
    use om9_core::rhino_semantics::{TrimmedSurface, set_trimmed_surface};
    use om9_scene::camera::Ray;
    let limits = Limits::default();
    let mut entity = plane("source with a hole", 8.0);
    let Geometry::Surface(surface) = &entity.geometry else { unreachable!() };
    let [u, v] = surface.domain;
    let uv = |a: f64, b: f64| [u[0] + a * (u[1] - u[0]), v[0] + b * (v[1] - v[0])];
    let trim = TrimmedSurface {
        underlying: surface.clone(),
        outer: vec![uv(0., 0.), uv(1., 0.), uv(1., 1.), uv(0., 1.)],
        inners: vec![vec![uv(0.4, 0.4), uv(0.4, 0.6), uv(0.6, 0.6), uv(0.6, 0.4)]],
        reversed: false,
    };
    let hole = om9_core::surface_kernel::evaluate(surface, uv(0.5, 0.5)[0], uv(0.5, 0.5)[1], &limits).unwrap();
    let target = om9_core::surface_kernel::evaluate(surface, uv(0.25, 0.75)[0], uv(0.25, 0.75)[1], &limits).unwrap();
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    let source = entity.id;
    let mut controller = controller_with(vec![entity]);
    controller.submit("_gvOrientGems").unwrap();
    controller.gem_placement_pick_target(source).unwrap();
    assert!(controller.gem_surface_manual());
    let ray = |point: [f64; 3], direction: [f64; 3]| Ray {
        origin: std::array::from_fn(|axis| point[axis] - 20.0 * direction[axis]),
        direction,
    };
    assert!(controller.gem_surface_click(ray(hole, [0.,0.,-1.]), None).is_err());
    for direction in [[0.,0.,-1.], [0.6,0.,-0.8]] {
        controller.gem_surface_click(ray(target, direction), None).unwrap();
    }
    assert!(gem_ids(&controller).is_empty());
    controller.submit("").unwrap();
    let ids = gem_ids(&controller);
    assert_eq!(ids.len(), 2);
    for id in ids {
        let point = controller.document().entities[&id].placement.point([0.;3]).unwrap();
        assert!(point.iter().zip(target).all(|(actual, expected)| (*actual - expected).abs() < 1e-6));
        assert_eq!(link(&controller, id).source, PlacementSource::Surface { id: source });
    }
    controller.undo().unwrap();
    assert!(gem_ids(&controller).is_empty());
}

#[test]
fn loader_uses_world_f4_and_placement_points_across_aliases() {
    let mut controller = controller_with(Vec::new());
    controller.set_cplane_origin_rotation([10.,15.,20.], 0.5).unwrap();
    controller.submit("_gvBuilderGemLoader _BRep=No _Cut=oval _Width=0 _Percentage=125").unwrap();
    controller.submit("").unwrap();
    let id = gem_ids(&controller)[0];
    assert_eq!(controller.document().entities[&id].placement.point([0.;3]).unwrap(), [0.;3]);
    controller.undo().unwrap();
    controller.dispatch("BuilderGemLoader", InvocationOrigin::F6).unwrap();
    controller.submit("BRep=No").unwrap();
    controller.gem_loader_point([2.,4.,6.]).unwrap();
    controller.gem_loader_point([3.,5.,7.]).unwrap();
    controller.submit("U").unwrap();
    controller.submit("").unwrap();
    let ids = gem_ids(&controller);
    assert_eq!(ids.len(), 1);
    assert_eq!(controller.document().entities[&ids[0]].placement.point([0.;3]).unwrap(), [2.,4.,6.]);
}


#[cfg(feature = "occt-brep")]
#[test]
fn viewport_click_selects_and_places_on_the_nurbs_source() {
    use om9_app::{ui::MatrixApp, ui_replay::replay_app};
    use serde_json::json;
    let mut document = Document::with_template_tolerance("gem viewport placement");
    let surface = om9_core::surface_kernel::plane(
        [-20., -20., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        40.,
        40.,
        &Limits::default(),
    ).unwrap();
    let entity = Entity::new("pick plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut app = MatrixApp::new(document).unwrap();
    let size = [1280., 800.];
    let probe = replay_app(&mut app, size, 1.0, &[]).unwrap();
    let view = probe["steps"][0]["views"].as_array().unwrap().iter()
        .find(|view| view["view"] == om9_scene::workspace::ViewId::LookingDown.title())
        .unwrap();
    let rect: [f64; 4] = serde_json::from_value(view["rect"].clone()).unwrap();
    let viewport = om9_scene::camera::Viewport { origin: [rect[0], rect[1]], size: [rect[2], rect[3]] };
    let camera = &app.controller.workspace.views[&om9_scene::workspace::ViewId::LookingDown].camera;
    let target = [1.0, 2.0, 0.0];
    let screen = camera.project(target, viewport).unwrap().unwrap().screen;
    app.controller.dispatch(ON_SURFACE, InvocationOrigin::F6).unwrap();
    let replay = replay_app(&mut app, size, 1.0, &[json!({"click": screen}), json!({"click": screen})]).unwrap();
    let final_frame = &replay["steps"][2];
    let final_view = final_frame["views"].as_array().unwrap().iter()
        .find(|view| view["view"] == final_frame["active_view"]).unwrap();
    let rect: [f64; 4] = serde_json::from_value(final_view["rect"].clone()).unwrap();
    let viewport = om9_scene::camera::Viewport { origin: [rect[0], rect[1]], size: [rect[2], rect[3]] };
    let camera = app.controller.workspace.views
        .values().find(|view| view.id.title() == final_frame["active_view"].as_str().unwrap()).unwrap().camera.clone();
    let ray = camera.ray(screen, viewport).unwrap();
    let t = -ray.origin[2] / ray.direction[2];
    let expected: [f64; 3] = std::array::from_fn(|axis| ray.origin[axis] + t * ray.direction[axis]);
    assert_eq!(app.controller.gem_placement_session().unwrap().preview_count(), 1);
    assert!(gem_ids(&app.controller).is_empty());
    app.controller.submit("").unwrap();
    let gem = gem_ids(&app.controller)[0];
    let entity = &app.controller.document().entities[&gem];
    let point = entity.placement.point([0.; 3]).unwrap();
    assert!(point.iter().zip(expected).all(|(actual, expected)| (*actual - expected).abs() < 1e-4),
        "placement {point:?} differs from camera ray {expected:?}");
    assert_eq!(link(&app.controller, gem).source, PlacementSource::Surface { id: source });
}

#[cfg(feature = "occt-brep")]
#[test]
fn gem_builders_keep_source_ids_and_stations_across_hosts_at_three_scales() {
    use eframe::egui;
    use om9_app::{dock_layout::{DockSide, FloatRect, SectionId}, session::gem_placement::GemPlacementTool,
        tool_panel::ToolPanel, ui::MatrixApp};
    for scale in [1., 1.25, 1.5] {
        for (tool, source) in [
            (GemPlacementTool::OnCurve, line("path", [0.; 3], [30., 0., 0.])),
            (GemPlacementTool::OnSurface, plane("mount", 8.)),
        ] {
            let id = source.id;
            let mut doc = Document::with_template_tolerance("placement and dock");
            doc.entities.insert(id, source);
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&[id]).unwrap();
            app.controller.dispatch(tool.command(), InvocationOrigin::Cmd).unwrap();
            if tool == GemPlacementTool::OnSurface {
                app.controller.gem_surface_click(om9_scene::camera::Ray {
                    origin: [12., 12., 50.], direction: [0., 0., -1.],
                }, None).unwrap();
            }
            let ctx = egui::Context::default();
            let frame = |app: &mut MatrixApp, width| {
                let mut raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO, egui::vec2(width, 850.))), ..Default::default() };
                raw.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().native_pixels_per_point = Some(scale);
                ctx.run_ui(raw, |ui| app.show(ui)).textures_delta.clear();
            };
            let section = SectionId::Tool(ToolPanel::Placement(tool));
            let preview = |app: &MatrixApp| {
                let session = app.controller.gem_placement_session().unwrap();
                if tool == GemPlacementTool::OnSurface { assert_eq!(session.surface.as_ref().unwrap().0, id); }
                else { assert_eq!(session.curves[0].0, id); }
                session.preview_stations().cloned().collect::<Vec<_>>()
            };
            let before = preview(&app);
            assert!(!before.is_empty());
            frame(&mut app, 1500.);
            assert!(app.dock_menu_section(section, DockSide::Right));
            for width in [1500., 1199., 1200.] {
                frame(&mut app, width);
                assert!(app.section_rect(section.key()).is_some());
                assert_eq!(preview(&app), before);
            }
            assert!(app.float_menu_section(section, FloatRect {x: 420., y: 110., width: 390., height: 400.}));
            frame(&mut app, 1500.);
            assert_eq!(preview(&app), before);
            assert!(app.dock_menu_section(section, DockSide::Left));
            frame(&mut app, 1500.);
            assert_eq!(preview(&app), before);
            assert_eq!(app.controller.document().entities.len(), 1);
            app.controller.submit("").unwrap();
            assert_eq!(gem_ids(&app.controller).len(), before.len());
            assert_eq!(app.controller.coordinator().undo_len(), 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document().entities.len(), 1);
            app.controller.select_objects(&[id]).unwrap();
            app.controller.dispatch(tool.command(), InvocationOrigin::Menu).unwrap();
            app.controller.gem_placement_cancel();
            assert!(app.controller.gem_placement_session().is_none());
            assert_eq!(app.controller.document().entities.len(), 1);
        }
    }
}
