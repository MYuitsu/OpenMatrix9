// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix ring/profile icon dispatch (`OM9-L2-04-02`). The same controller entry points the
//! sidebar, the menu and the command line use.
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{builder::BuilderKind, 
    budget::Limits,
    curve_build,
    curve_ops,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

fn controller_with(entities: Vec<Entity>) -> Controller {
    let mut document = Document::new("ring-tools");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).unwrap()
}

fn circle(radius: f64) -> Geometry {
    Geometry::Curve(Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn rectangle(x: f64) -> Entity {
    Entity::new(
        "profile",
        Geometry::Curve(
            curve_build::polyline(&[
                [x, -2.0, -2.0],
                [x, 2.0, -2.0],
                [x, 2.0, 2.0],
                [x, -2.0, 2.0],
                [x, -2.0, -2.0],
            ])
            .unwrap(),
        ),
        None,
    )
}

fn active_builder(controller: &Controller) -> Option<BuilderKind> {
    controller
        .builder_tabs()
        .into_iter()
        .find_map(|(kind, active)| active.then_some(kind))
}

#[test]
fn the_profile_placer_and_outside_rail_icons_open_the_existing_builder_modes() {
    let rail = Entity::new("rail", circle(9.0), None);
    let rail_id = rail.id;
    let mut controller = controller_with(vec![rail]);
    controller.select_objects(&[rail_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-002", InvocationOrigin::Menu)
        .expect("profile placer");
    assert_eq!(active_builder(&controller), Some(BuilderKind::RailProfile));
    assert!(controller.builder_is_open());

    controller.builder_cancel();
    controller
        .dispatch("OM9-TOOLS-003", InvocationOrigin::Menu)
        .expect("outside ring rail");
    assert_eq!(active_builder(&controller), Some(BuilderKind::OutsideRail));
}

#[test]
fn the_custom_rail_icon_records_a_validated_closed_rail_and_refuses_an_open_one() {
    let rail = Entity::new("rail", circle(9.0), None);
    let rail_id = rail.id;
    let mut controller = controller_with(vec![rail]);
    controller.select_objects(&[rail_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-004", InvocationOrigin::Menu)
        .expect("custom rail");
    assert_eq!(controller.document().entities.len(), 2);
    assert!(
        controller
            .document()
            .entities
            .values()
            .any(|entity| entity.label == "CustomRail")
    );

    let open = Entity::new(
        "open",
        Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let open_id = open.id;
    let mut controller = controller_with(vec![open]);
    controller.select_objects(&[open_id]).unwrap();
    assert!(
        controller
            .dispatch("OM9-TOOLS-035", InvocationOrigin::Menu)
            .is_err(),
        "an open rail cannot become a custom ring"
    );
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn the_profile_merge_icon_joins_compatible_sections_and_refuses_one_section() {
    let limits = Limits::default();
    let rail = Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 1.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let (head, tail) = curve_ops::split_curve(&rail, 0.5, &limits).unwrap();
    let first = Entity::new("head", Geometry::Curve(head), None);
    let second = Entity::new("tail", Geometry::Curve(tail), None);
    let (first_id, second_id) = (first.id, second.id);
    let mut controller = controller_with(vec![first, second]);
    controller.select_objects(&[first_id, second_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-034", InvocationOrigin::Cmd)
        .expect("profile merge");
    let merged = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ProfileMerge")
        .expect("merged curve");
    let Geometry::Curve(curve) = &merged.geometry else {
        panic!("merged curve geometry");
    };
    assert!(curve_ops::is_closed(curve, 1.0e-6, &limits).unwrap());

    let single = Entity::new("single", circle(1.0), None);
    let single_id = single.id;
    let mut controller = controller_with(vec![single]);
    controller.select_objects(&[single_id]).unwrap();
    assert!(
        controller
            .dispatch("OM9-TOOLS-034", InvocationOrigin::Menu)
            .is_err()
    );
}

#[test]
fn the_profile_end_cap_icon_builds_a_planar_cap_from_closed_sections() {
    let profile = rectangle(0.0);
    let profile_id = profile.id;
    let mut controller = controller_with(vec![profile]);
    controller.select_objects(&[profile_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-033", InvocationOrigin::Menu)
        .expect("profile end cap");
    assert_eq!(controller.document().entities.len(), 2);
    assert!(matches!(
        controller
            .document()
            .entities
            .values()
            .find(|entity| entity.label == "ProfileEndCap")
            .expect("end cap")
            .geometry,
        Geometry::Surface(_)
    ));
}

#[test]
fn the_four_profile_icon_reaches_the_existing_four_rail_metadata_command() {
    let rails = [
        Entity::new(
            "rail 0",
            Geometry::Curve(curve_build::polyline(&[[0.0, -2.0, -2.0], [10.0, -2.0, -2.0]]).unwrap()),
            None,
        ),
        Entity::new(
            "rail 1",
            Geometry::Curve(curve_build::polyline(&[[0.0, 2.0, -2.0], [10.0, 2.0, -2.0]]).unwrap()),
            None,
        ),
        Entity::new(
            "rail 2",
            Geometry::Curve(curve_build::polyline(&[[0.0, 2.0, 2.0], [10.0, 2.0, 2.0]]).unwrap()),
            None,
        ),
        Entity::new(
            "rail 3",
            Geometry::Curve(curve_build::polyline(&[[0.0, -2.0, 2.0], [10.0, -2.0, 2.0]]).unwrap()),
            None,
        ),
    ];
    let first = rectangle(0.0);
    let second = rectangle(10.0);
    let (first_id, second_id) = (first.id, second.id);
    let mut ids: Vec<_> = rails.iter().map(|rail| rail.id).collect();
    ids.extend([first_id, second_id]);
    let mut controller = controller_with(
        rails
            .into_iter()
            .chain([first, second])
            .collect::<Vec<_>>(),
    );
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-TOOLS-032", InvocationOrigin::Menu)
        .expect("four profile icon");
    for id in [first_id, second_id] {
        assert!(
            controller.document().entities[&id]
                .extensions
                .contains_key("om9.four-rail-profile")
        );
    }
}

#[test]
fn the_profile_end_cap_refuses_an_open_curve_without_changing_the_document() {
    let open = Entity::new(
        "open",
        Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [5.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let open_id = open.id;
    let mut controller = controller_with(vec![open]);
    controller.select_objects(&[open_id]).unwrap();
    assert!(
        controller
            .dispatch("OM9-TOOLS-033", InvocationOrigin::Menu)
            .is_err(),
        "an open curve has no cap boundary"
    );
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn the_four_profile_icon_refuses_an_inconsistent_selection() {
    let mut entities = Vec::new();
    for index in 0..7 {
        entities.push(Entity::new(
            format!("rail {index}"),
            Geometry::Curve(
                curve_build::polyline(&[[0.0, index as f64, 0.0], [10.0, index as f64, 0.0]])
                    .unwrap(),
            ),
            None,
        ));
    }
    let ids: Vec<_> = entities.iter().map(|entity| entity.id).collect();
    let mut controller = controller_with(entities);
    controller.select_objects(&ids).unwrap();
    assert!(
        controller
            .dispatch("OM9-TOOLS-032", InvocationOrigin::Menu)
            .is_err(),
        "five-plus rails is not a four-boundary profile selection"
    );
}

#[test]
fn the_ring_size_command_switches_the_builder_to_the_us_table() {
    let rail = Entity::new("rail", circle(9.0), None);
    let rail_id = rail.id;
    let mut controller = controller_with(vec![rail]);
    controller.select_objects(&[rail_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-002", InvocationOrigin::Cmd)
        .expect("profile placer");
    controller.builder_cancel();
    controller
        .submit("RingRail")
        .expect("ring rail command");
    controller.submit("standard=us").expect("US table");
    controller.submit("size=9").expect("US size 9");
    let summary = controller.builder_summary().expect("summary");
    assert!(summary.contains("US size 9"), "{summary}");
    assert!(summary.contains("diameter 18.95 mm"), "{summary}");
    assert!(
        controller.submit("standard=UK").is_err(),
        "an unverified table must be refused by the command line too"
    );
}

#[test]
fn a_ring_size_edit_rebuilds_the_profile_and_one_dirty_sweep() {
    let rail = Entity::new("rail", circle(9.0), None);
    let rail_id = rail.id;
    let mut controller = controller_with(vec![rail]);

    controller.begin_builder(BuilderKind::RingRail).unwrap();
    controller.builder_apply().unwrap();
    let rail_id = controller
        .document()
        .entities
        .iter()
        .find_map(|(id, entity)| (entity.label == "Ring Rail").then_some(*id))
        .unwrap_or(rail_id);

    controller.select_objects(&[rail_id]).unwrap();
    controller.begin_builder(BuilderKind::RailProfile).unwrap();
    controller.builder_apply().unwrap();
    let profile_id = *controller
        .document()
        .entities
        .iter()
        .find_map(|(id, entity)| (entity.label == "Rail Profile").then_some(id))
        .expect("profile");

    controller.select_objects(&[rail_id, profile_id]).unwrap();
    controller.begin_builder(BuilderKind::RailSweep).unwrap();
    controller.builder_apply().unwrap();
    let sweep_id = *controller
        .document()
        .entities
        .iter()
        .find_map(|(id, entity)| (entity.label == "Rail Sweep").then_some(id))
        .expect("sweep");
    let before = controller.document().entities[&sweep_id]
        .geometry_revision;
    let ring_recipe = controller
        .document()
        .history
        .recipes
        .iter()
        .find_map(|(id, recipe)| (recipe.command == "RingRail").then_some(*id))
        .expect("ring recipe");
    controller
        .set_history_parameter(
            ring_recipe,
            "diameter_mm",
            om9_core::request::Value::length(om9_core::units::LengthMm::try_new(19.0).unwrap()),
        )
        .unwrap();
    controller.recompute_history(ring_recipe).unwrap();
    let after = controller.document().entities[&sweep_id]
        .geometry_revision;
    assert!(after > before, "the dependent sweep must be rebuilt once");
    let sweep = &controller.document().entities[&sweep_id];
    assert_eq!(sweep.label, "Rail Sweep");
    // This build uses the OCCT registry, so the sweep lands as a B-Rep; the reference
    // backend test in `om9-core` covers the analytic surface variant.
    assert!(
        matches!(&sweep.geometry, Geometry::Brep(_) | Geometry::Surface(_)),
        "sweep geometry variant: {:?}",
        std::mem::discriminant(&sweep.geometry)
    );
    assert_eq!(
        om9_core::history_ops::plan_recompute(&controller.document().history)
            .unwrap()
            .len(),
        0,
        "the chain A -> B -> C is clean after one edit"
    );

    // A failed downstream rebuild keeps the last committed sweep: the profile width becomes
    // invalid, so the sweep must not commit a partial or empty B-Rep.
    let profile_recipe = controller
        .document()
        .history
        .recipes
        .iter()
        .find_map(|(id, recipe)| (recipe.command == "RailProfile").then_some(*id))
        .expect("profile recipe");
    let committed = controller.document().entities[&sweep_id]
        .geometry_revision;
    controller
        .set_history_parameter(
            profile_recipe,
            "width_mm",
            om9_core::request::Value::length(om9_core::units::LengthMm::try_new(0.0).unwrap()),
        )
        .unwrap();
    let outcome = controller.recompute_history(profile_recipe).unwrap();
    assert!(
        matches!(outcome, om9_core::associative::RecomputeOutcome::Failed { .. }),
        "an invalid profile must fail explicitly: {outcome:?}"
    );
    assert_eq!(
        controller.document().entities[&sweep_id].geometry_revision,
        committed,
        "a failed sweep leaves the committed B-Rep unchanged"
    );
}

#[test]
fn the_rail_profile_handles_edit_width_and_commit_exactly_once() {
    let rail = Entity::new("rail", circle(9.0), None);
    let rail_id = rail.id;
    let mut controller = controller_with(vec![rail]);
    controller.select_objects(&[rail_id]).unwrap();
    controller.begin_builder(BuilderKind::RailProfile).unwrap();
    assert_eq!(controller.builder_handles().len(), 3);
    controller.builder_adjust_field("width_mm", 1.0).unwrap();
    let summary = controller.builder_summary().expect("summary");
    assert!(summary.contains("3.00 x 1.50"), "{summary}");
    controller.builder_apply().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.document().history.recipes.len(), 1);

    controller.select_objects(&[rail_id]).unwrap();
    controller.begin_builder(BuilderKind::RailProfile).unwrap();
    controller.builder_adjust_field("width_mm", 1.0).unwrap();
    controller.builder_cancel();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.document().history.recipes.len(), 1);
}



