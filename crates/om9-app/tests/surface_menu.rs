// SPDX-License-Identifier: LGPL-2.1-or-later
//! Menu-binding acceptance for `OM9-L2-03-17`: every documented Surface Tools row
//! has one stable command id and an explicit visible disabled reason.
use om9_app::{
    menu_catalog,
    session::{Controller, InvocationOrigin},
    surface_menu::{self, BindingState},
};
use om9_core::{
    budget::Limits,
    curve_build,
    geometry::Geometry,
    identity::ObjectId,
    model::{Document, Entity},
    rhino_semantics::{
        FourRailProfile, TrimmedSurface, four_rail_profile_of, set_four_rail_profile,
        set_trimmed_surface,
    },
    surface_kernel,
};

#[test]
fn every_documented_surface_menu_row_has_one_stable_id_and_icon() {
    assert_eq!(surface_menu::BINDINGS.len(), 38);
    let mut l3 = std::collections::BTreeSet::new();
    let mut features = std::collections::BTreeSet::new();
    for (index, binding) in surface_menu::BINDINGS.iter().enumerate() {
        assert_eq!(
            binding.l3_id,
            format!("OM9-L3-03-17-{:03}", index + 1),
            "L3 order"
        );
        assert!(l3.insert(binding.l3_id), "duplicate L3 {}", binding.l3_id);
        assert!(
            features.insert(binding.feature),
            "duplicate command id {}",
            binding.feature
        );
        assert!(
            menu_catalog::SURFACE.contains(&binding.icon),
            "{} is not in the authored Surface palette",
            binding.icon
        );
        assert_eq!(
            surface_menu::by_l3(binding.l3_id).map(|row| row.feature),
            Some(binding.feature)
        );
        assert_eq!(
            surface_menu::by_feature(binding.feature).map(|row| row.l3_id),
            Some(binding.l3_id)
        );
        assert!(!binding.label.is_empty());
        assert!(!binding.source_command.is_empty());
    }
    assert_eq!(l3.len(), 38);
    assert_eq!(features.len(), 38);
}

#[test]
fn enabled_rows_dispatch_and_the_remaining_rows_stay_visible_disabled() {
    let mut enabled = 0;
    for binding in surface_menu::BINDINGS
        .iter()
        .filter(|binding| binding.enabled())
    {
        enabled += 1;
        assert_eq!(binding.state, BindingState::Implemented);
        assert_eq!(binding.disabled_reason, "");
        assert!(
            Controller::availability(binding.feature).is_ok(),
            "{} should dispatch",
            binding.feature
        );
        assert!(surface_menu::enabled(binding.feature));
        assert_eq!(surface_menu::disabled_reason(binding.feature), None);
    }
    assert_eq!(enabled, 38);
    let mut disabled = 0;
    for binding in surface_menu::BINDINGS
        .iter()
        .filter(|binding| !binding.enabled())
    {
        disabled += 1;
        assert!(!binding.disabled_reason.is_empty(), "{}", binding.l3_id);
        assert_eq!(
            surface_menu::disabled_reason(binding.feature),
            Some(binding.disabled_reason)
        );
        assert!(
            Controller::availability(binding.feature).is_err(),
            "{} unexpectedly dispatches",
            binding.feature
        );
        if binding.state == BindingState::OwnerMissing {
            assert!(
                binding
                    .disabled_reason
                    .contains("no verified backend owner")
            );
        } else {
            assert!(binding.disabled_reason.contains(binding.owner));
        }
    }
    assert_eq!(disabled, 0);
}

#[test]
fn shared_history_icons_keep_distinct_command_ids() {
    // Sweep 1/History share the authored icon but not the command id.
    let sweep = surface_menu::by_l3("OM9-L3-03-17-001").expect("sweep");
    let history = surface_menu::by_l3("OM9-L3-03-17-002").expect("history");
    assert_eq!(sweep.icon, history.icon);
    assert_ne!(sweep.feature, history.feature);
    assert_eq!(
        surface_menu::by_icon(sweep.icon).map(|row| row.l3_id),
        Some(sweep.l3_id)
    );
    assert_eq!(
        surface_menu::by_feature(history.feature).map(|row| row.l3_id),
        Some(history.l3_id)
    );
}

#[test]
fn the_surface_catalog_covers_every_l3_owner() {
    let ids: Vec<&str> = surface_menu::BINDINGS.iter().map(|row| row.l3_id).collect();
    assert_eq!(ids.first(), Some(&"OM9-L3-03-17-001"));
    assert_eq!(ids.last(), Some(&"OM9-L3-03-17-038"));
    let session_missing = surface_menu::BINDINGS
        .iter()
        .filter(|row| row.state == BindingState::SessionMissing)
        .count();
    let owner_missing = surface_menu::BINDINGS
        .iter()
        .filter(|row| row.state == BindingState::OwnerMissing)
        .count();
    let implemented = surface_menu::BINDINGS
        .iter()
        .filter(|row| row.state == BindingState::Implemented)
        .count();
    assert_eq!(implemented, 38);
    assert_eq!(session_missing + owner_missing + implemented, 38);
    for row in surface_menu::BINDINGS {
        assert!(
            row.owner.starts_with("OM9-L3-") || row.owner == "OM9-L2-03-17",
            "{} has no owner candidate",
            row.l3_id
        );
    }
}

#[test]
fn surface_edit_without_selection_or_with_a_curve_is_refused_without_a_transaction() {
    let limits = Limits::default();
    let mut document = Document::new("surface edit refusal");
    let curve = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [1.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let curve_id = curve.id;
    document.entities.insert(curve_id, curve);
    let mut controller = Controller::new(document, limits).unwrap();
    assert!(
        controller
            .dispatch("OM9-SURFACE-006", InvocationOrigin::Menu)
            .is_err()
    );
    assert_eq!(controller.coordinator().undo_len(), 0);
    controller.select_objects(&[curve_id]).unwrap();
    assert!(
        controller
            .dispatch("OM9-SURFACE-006", InvocationOrigin::Menu)
            .is_err()
    );
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn rebuild_and_unroll_run_from_a_selected_surface_and_commit_one_transaction() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface edit");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();

    controller
        .dispatch("OM9-SURFACE-006", InvocationOrigin::Menu)
        .expect("rebuild");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);

    controller
        .dispatch("OM9-SURFACE-134", InvocationOrigin::Menu)
        .expect("unroll");
    assert_eq!(controller.document().entities.len(), 2);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn offset_and_extrude_normal_wait_for_distance_and_commit_one_surface() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface distance");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();

    controller
        .dispatch("OM9-SURFACE-025", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Offset distance"));
    controller.submit("2").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();

    controller
        .dispatch("OM9-SURFACE-133", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Extrude distance"));
    controller.submit("1.5").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn a_zero_offset_is_refused_and_cancel_clears_the_prompt() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface distance refusal");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-025", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.submit("0").is_err());
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 0);
    controller.cancel();
    assert!(!controller.is_running());
}

#[test]
fn plane_from_corners_collects_three_points_and_refuses_collinear_input() {
    let mut controller =
        Controller::new(Document::new("plane corners"), Limits::default()).unwrap();
    controller
        .dispatch("OM9-SURFACE-015", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("First corner"));
    controller.submit("0,0,0").unwrap();
    assert!(controller.prompt().contains("Second corner"));
    controller.submit("10,0,0").unwrap();
    assert!(controller.prompt().contains("Third corner"));
    controller.submit("0,10,0").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let entity = controller.document().entities.values().next().unwrap();
    assert!(matches!(entity.geometry, Geometry::Surface(_)));

    let mut collinear =
        Controller::new(Document::new("plane corners refusal"), Limits::default()).unwrap();
    collinear
        .dispatch("OM9-SURFACE-015", InvocationOrigin::Menu)
        .unwrap();
    collinear.submit("0,0,0").unwrap();
    collinear.submit("1,0,0").unwrap();
    assert!(collinear.submit("2,0,0").is_err());
    assert_eq!(collinear.document().entities.len(), 0);
    assert_eq!(collinear.coordinator().undo_len(), 0);
}

#[test]
fn blend_and_loft_combine_two_selected_curves_into_one_surface() {
    let limits = Limits::default();
    let mut document = Document::new("blend and loft");
    let first = Entity::new(
        "first",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 10.0],
        }),
        None,
    );
    let second = Entity::new(
        "second",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 5.0, 5.0],
            end: [10.0, 5.0, 5.0],
            domain: [0.0, 10.0],
        }),
        None,
    );
    let first_id = first.id;
    let second_id = second.id;
    document.entities.insert(first_id, first);
    document.entities.insert(second_id, second);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[first_id, second_id]).unwrap();

    controller
        .dispatch("OM9-SURFACE-007", InvocationOrigin::Menu)
        .expect("blend");
    assert_eq!(controller.document().entities.len(), 3);

    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != first_id && entity.id != second_id)
        .expect("blend surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
    controller.undo().unwrap();

    controller
        .dispatch("OM9-SURFACE-109", InvocationOrigin::Menu)
        .expect("loft");
    assert_eq!(controller.document().entities.len(), 3);

    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn network_sweep1_and_sweep2_build_surfaces_from_ordered_curves() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }

    let limits = Limits::default();
    let mut document = Document::new("network and sweeps");
    let u0 = line("u0", [0.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
    let u1 = line("u1", [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let v0 = line("v0", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let v1 = line("v1", [0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [u0.id, u1.id, v0.id, v1.id];
    for entity in [u0, u1, v0, v1] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-010", InvocationOrigin::Menu)
        .expect("network");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);

    let mut sweep1 = Document::new("sweep 1");
    let profile = line("profile", [0.0, 0.0, 0.0], [0.0, 5.0, 0.0]);
    let rail = line("rail", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let ids = [profile.id, rail.id];
    for entity in [profile, rail] {
        sweep1.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(sweep1, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-101", InvocationOrigin::Menu)
        .expect("sweep 1");
    assert_eq!(controller.document().entities.len(), 3);

    let mut sweep2 = Document::new("sweep 2");
    let profile = line("profile", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let rail_a = line("rail a", [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
    let rail_b = line("rail b", [10.0, 0.0, 0.0], [10.0, 0.0, 10.0]);
    let ids = [profile.id, rail_a.id, rail_b.id];
    for entity in [profile, rail_a, rail_b] {
        sweep2.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(sweep2, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-003", InvocationOrigin::Menu)
        .expect("sweep 2");
    assert_eq!(controller.document().entities.len(), 4);
}

#[test]
fn rail_revolve_waits_for_an_axis_and_commits_one_surface() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("rail revolve");
    let profile = line("profile", [5.0, 0.0, 0.0], [6.0, 0.0, 0.0]);
    let rail = line("rail", [5.0, 0.0, 0.0], [5.0, 0.0, 10.0]);
    let ids = [profile.id, rail.id];
    for entity in [profile, rail] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-019", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("axis"));
    controller.submit("0,0,0,0,0,1").unwrap();
    assert_eq!(controller.document().entities.len(), 3);

    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn match_surfaces_waits_for_edge_names_and_commits_one_surface() {
    let limits = Limits::default();
    let first = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let second = surface_kernel::plane(
        [10.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("match surfaces");
    let first = Entity::new("first", Geometry::Surface(first), None);
    let second = Entity::new("second", Geometry::Surface(second), None);
    let ids = [first.id, second.id];
    document.entities.insert(first.id, first);
    document.entities.insert(second.id, second);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-128", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Match source edge"));
    controller.submit("U1,U0").unwrap();
    assert!(
        controller.prompt().contains("Continuity=Position"),
        "{}",
        controller.prompt()
    );
    controller.submit("").unwrap();
    assert_eq!(controller.document().entities.len(), 3);

    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("matched surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
    assert!(controller.submit("U1,U0").is_err(), "no active match");
}

#[test]
fn adjust_surface_seam_waits_for_axis_and_tolerance() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface seam");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-038", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Seam axis"));
    assert!(controller.submit("W,1").is_err(), "bad axis is refused");
    assert!(controller.is_running(), "the prompt survives a refusal");
    controller.submit("U,20").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("seamed surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn soft_edit_moves_nearby_poles_and_commits_one_surface() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface soft edit");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Soft edit"));
    assert!(
        controller.submit("").is_err(),
        "Enter before the base point is refused"
    );
    assert!(controller.is_running());
    controller.submit("U=0.5,V=0.5").unwrap();
    assert!(
        controller.prompt().contains("Dx"),
        "{}",
        controller.prompt()
    );
    controller.submit("0,0,1").unwrap();
    assert!(
        controller.prompt().contains("UDistance=1"),
        "{}",
        controller.prompt()
    );
    controller.submit("UDistance=0.5,VDistance=0.5").unwrap();
    controller.submit("").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("soft edited surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn soft_edit_copy_no_edits_the_selected_surface_in_place() {
    let limits = Limits::default();
    let source_surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let before = source_surface.poles.clone();
    let mut document = Document::new("surface soft edit in place");
    let entity = Entity::new("plane", Geometry::Surface(source_surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    controller.submit("U=0.5,V=0.5,Dx=0,Dy=0,Dz=2").unwrap();
    controller.submit("Copy=No").unwrap();
    controller.submit("").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let Geometry::Surface(after) = &controller
        .document()
        .entities
        .get(&source)
        .expect("edited surface")
        .geometry
    else {
        panic!("surface kept");
    };
    assert_ne!(before, after.poles, "the selected surface moved");
}

#[test]
fn soft_edit_fix_edges_keeps_the_boundary_poles() {
    let limits = Limits::default();
    // A 4x4 bicubic patch: the interior poles are distinct from the boundary rows.
    let mut poles = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            poles.push([i as f64 * 10.0, j as f64 * 10.0, 0.0]);
        }
    }
    let surface = om9_core::geometry::NurbsSurface {
        degree: [3, 3],
        size: [4, 4],
        poles,
        weights: vec![1.0; 16],
        knots: [
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let boundary = [0usize, 3, 12, 15, 4, 8, 1, 2];
    let interior = 5;
    let before = surface.poles.clone();
    let mut document = Document::new("surface soft edit fixed edges");
    let entity = Entity::new("patch", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    controller
        .submit("U=0.5,V=0.5,Dx=0,Dy=0,Dz=2,FixEdges=Yes")
        .unwrap();
    controller.submit("").unwrap();
    let Geometry::Surface(edited) = &controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("soft edited surface")
        .geometry
    else {
        panic!("surface");
    };
    for index in boundary {
        assert_eq!(before[index], edited.poles[index], "boundary pole {index}");
    }
    assert_ne!(
        before[interior], edited.poles[interior],
        "interior pole moved"
    );
}

#[test]
fn soft_edit_direction_constraint_projects_the_move() {
    let limits = Limits::default();
    let source_surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let before = source_surface.poles.clone();
    let mut document = Document::new("surface soft edit constraint");
    let entity = Entity::new("plane", Geometry::Surface(source_surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    controller
        .submit("U=0.5,V=0.5,Dx=2,Dy=3,Dz=5,DirConstraint=SrfNormal")
        .unwrap();
    controller.submit("").unwrap();
    let Geometry::Surface(edited) = &controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("soft edited surface")
        .geometry
    else {
        panic!("surface");
    };
    let index = edited
        .poles
        .iter()
        .zip(&before)
        .position(|(after, before)| after != before)
        .expect("a moved pole");
    assert_eq!(
        [edited.poles[index][0], edited.poles[index][1]],
        [before[index][0], before[index][1]],
        "SrfNormal keeps the move along +Z"
    );
    assert!(edited.poles[index][2] > before[index][2]);
}

#[test]
fn soft_edit_rejects_an_unknown_option_and_keeps_running() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface soft edit invalid option");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.submit("UDistance=0").is_err());
    assert!(controller.submit("Copy=maybe").is_err());
    assert!(controller.submit("DirConstraint=Sideways").is_err());
    assert!(controller.submit("RadiusU=1").is_err());
    assert!(
        controller.submit("U=0.5").is_ok(),
        "the refused options keep the command running"
    );
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn tween_surfaces_waits_for_a_fraction_and_commits_one_surface() {
    let limits = Limits::default();
    let first = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let second = surface_kernel::plane(
        [0.0, 0.0, 10.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("tween surfaces");
    let first = Entity::new("first", Geometry::Surface(first), None);
    let second = Entity::new("second", Geometry::Surface(second), None);
    let ids = [first.id, second.id];
    document.entities.insert(first.id, first);
    document.entities.insert(second.id, second);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-027", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Tween fraction"));
    assert!(
        controller.submit("1.5").is_err(),
        "fraction outside 0..1 is refused"
    );
    assert!(controller.is_running());
    controller.submit("0.5").unwrap();
    assert_eq!(controller.document().entities.len(), 3);

    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("tween surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn extend_surface_waits_for_edge_and_distance() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface extend");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-033", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Extend edge"));
    assert!(controller.submit("W1,5").is_err(), "bad edge is refused");
    assert!(controller.is_running());
    controller.submit("U1,5").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("extended surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn profile_sweep_uses_the_sweep1_service() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("profile sweep");
    let profile = line("profile", [0.0, 0.0, 0.0], [0.0, 5.0, 0.0]);
    let rail = line("rail", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let ids = [profile.id, rail.id];
    for entity in [profile, rail] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-005", InvocationOrigin::Menu)
        .expect("profile sweep");
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn sweep_multi_creates_one_surface_per_profile_in_one_transaction() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep multi");
    let rail = line("rail", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let first = line("first", [0.0, 0.0, 0.0], [0.0, 5.0, 0.0]);
    let second = line("second", [0.0, 1.0, 0.0], [0.0, 1.0, 5.0]);
    let ids = [rail.id, first.id, second.id];
    for entity in [rail, first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-032", InvocationOrigin::Menu)
        .expect("sweep multi");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let surfaces = controller
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count();
    assert_eq!(surfaces, 2);
}

#[test]
fn merge_surfaces_joins_two_matching_boundaries() {
    let limits = Limits::default();
    let first = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let second = surface_kernel::plane(
        [10.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("merge surfaces");
    let first = Entity::new("first", Geometry::Surface(first), None);
    let second = Entity::new("second", Geometry::Surface(second), None);
    let ids = [first.id, second.id];
    document.entities.insert(first.id, first);
    document.entities.insert(second.id, second);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-127", InvocationOrigin::Menu)
        .expect("merge");
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("merged surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn patch_builds_a_four_boundary_surface() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("patch");
    let u0 = line("u0", [0.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
    let u1 = line("u1", [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let v0 = line("v0", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let v1 = line("v1", [0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [u0.id, u1.id, v0.id, v1.id];
    for entity in [u0, u1, v0, v1] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-011", InvocationOrigin::Menu)
        .expect("patch");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("patch surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn cutting_plane_collects_three_points_and_commits_a_plane() {
    let mut controller =
        Controller::new(Document::new("cutting plane"), Limits::default()).unwrap();
    controller
        .dispatch("OM9-SURFACE-039", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("First corner"));
    controller.submit("0,0,0").unwrap();
    controller.submit("10,0,0").unwrap();
    controller.submit("0,10,0").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let entity = controller.document().entities.values().next().unwrap();
    assert!(matches!(entity.geometry, Geometry::Surface(_)));
}

#[test]
fn split_at_isocurve_creates_two_surfaces_in_one_transaction() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("split isocurve");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-034", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Split axis"));
    assert!(
        controller.submit("U,0").is_err(),
        "boundary parameter is refused"
    );
    assert!(controller.is_running());
    controller.submit("U,0.5").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let surfaces = controller
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count();
    assert_eq!(surfaces, 3);
}

#[test]
fn drape_builds_a_surface_over_selected_points() {
    let limits = Limits::default();
    let mut document = Document::new("drape");
    let mut ids = Vec::new();
    for (name, point) in [
        ("a", [0.0, 0.0, 0.0]),
        ("b", [10.0, 0.0, 2.0]),
        ("c", [0.0, 10.0, 3.0]),
        ("d", [10.0, 10.0, 5.0]),
    ] {
        let entity = Entity::new(name, Geometry::Point(point), None);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-030", InvocationOrigin::Menu)
        .expect("drape");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("drape surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn set_surface_tangent_waits_for_edge_and_direction() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("surface tangent");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-040", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Tangent edge"));
    assert!(
        controller.submit("U0,0,0").is_err(),
        "incomplete direction is refused"
    );
    assert!(controller.is_running());
    controller.submit("U0,0,0,1").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("tangent surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn extrude_all_creates_one_surface_per_curve() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("extrude all");
    let first = line("first", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line("second", [0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-117", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Extrude all height"));
    assert!(controller.submit("0").is_err(), "zero height is refused");
    assert!(controller.is_running());
    controller.submit("5").unwrap();
    assert_eq!(controller.document().entities.len(), 4);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let surfaces = controller
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count();
    assert_eq!(surfaces, 2);
}

#[test]
fn edge_curves_build_a_surface_from_two_or_four_edges() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("edge curves");
    let u0 = line("u0", [0.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
    let u1 = line("u1", [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let v0 = line("v0", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let v1 = line("v1", [0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [u0.id, u1.id, v0.id, v1.id];
    for entity in [u0, u1, v0, v1] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-014", InvocationOrigin::Menu)
        .expect("edge curves");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn planar_curves_build_a_surface_from_coplanar_curves() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("planar curves");
    let a = line("a", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line("b", [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let c = line("c", [10.0, 10.0, 0.0], [0.0, 10.0, 0.0]);
    let d = line("d", [0.0, 10.0, 0.0], [0.0, 0.0, 0.0]);
    let ids = [a.id, b.id, c.id, d.id];
    for entity in [a, b, c, d] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-013", InvocationOrigin::Menu)
        .expect("planar curves");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("planar surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn variable_offset_waits_for_two_distances_and_an_axis() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("variable offset");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-026", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Variable offset start"));
    assert!(controller.submit("1,5,W").is_err(), "bad axis is refused");
    assert!(controller.is_running());
    controller.submit("1,5,U").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("offset surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn shrink_surface_waits_for_a_uv_subdomain() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("shrink surface");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-012", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Shrink u0"));
    assert!(
        controller.submit("0.75,0.25,0,1").is_err(),
        "empty range is refused"
    );
    assert!(controller.is_running());
    controller.submit("0.25,0.75,0.25,0.75").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("shrunken surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn variable_blend_waits_for_tangent_scales() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("variable blend");
    let first = line("first", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line("second", [0.0, 5.0, 5.0], [10.0, 5.0, 5.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-008", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Blend start scale"));
    assert!(controller.submit("1").is_err(), "one scale is refused");
    assert!(controller.is_running());
    controller.submit("1,2").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("variable blend surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn untrim_removes_a_trim_extension_and_refuses_an_untrimmed_surface() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("untrim");
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface.clone()), None);
    let trimmed = TrimmedSurface {
        underlying: surface,
        outer: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        inners: Vec::new(),
        reversed: false,
    };
    set_trimmed_surface(&mut entity, &trimmed, &limits).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller
        .dispatch("OM9-SURFACE-017", InvocationOrigin::Menu)
        .expect("untrim");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .expect("untrimmed");
    assert!(added.extensions.is_empty());

    controller.undo().unwrap();
    let plain = Entity::new(
        "plain",
        Geometry::Surface(
            surface_kernel::plane(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                10.0,
                10.0,
                &limits,
            )
            .unwrap(),
        ),
        None,
    );
    let plain_id = plain.id;
    let mut plain_document = Document::new("untrim refusal");
    plain_document.entities.insert(plain_id, plain);
    let mut plain_controller = Controller::new(plain_document, limits).unwrap();
    plain_controller.select_objects(&[plain_id]).unwrap();
    assert!(
        plain_controller
            .dispatch("OM9-SURFACE-017", InvocationOrigin::Menu)
            .is_err()
    );
}

#[test]
fn chamfer_surfaces_waits_for_a_setback() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("chamfer");
    let first = line("first", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line("second", [0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-023", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.prompt().contains("Chamfer setback"));
    assert!(controller.submit("0").is_err(), "zero setback is refused");
    assert!(controller.is_running());
    controller.submit("1").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("chamfer surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn fillet_and_variable_chamfer_strips_commit_surfaces() {
    fn line(name: &str, start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let limits = Limits::default();
    let build = || {
        let mut document = Document::new("fillet chamfer");
        let first = line("first", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
        let second = line("second", [0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
        let ids = [first.id, second.id];
        document.entities.insert(first.id, first);
        document.entities.insert(second.id, second);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&ids).unwrap();
        (controller, ids)
    };

    let (mut fillet, ids) = build();
    fillet
        .dispatch("OM9-SURFACE-020", InvocationOrigin::Menu)
        .unwrap();
    assert!(fillet.prompt().contains("Fillet radius"));
    assert!(
        fillet.submit("1").is_err(),
        "radius smaller than half the span is refused"
    );
    assert!(fillet.is_running());
    fillet.submit("6").unwrap();
    assert_eq!(fillet.document().entities.len(), 3);
    assert!(matches!(
        fillet
            .document()
            .entities
            .values()
            .find(|entity| !ids.contains(&entity.id))
            .unwrap()
            .geometry,
        Geometry::Surface(_)
    ));

    let (mut variable, _ids) = build();
    variable
        .dispatch("OM9-SURFACE-022", InvocationOrigin::Menu)
        .unwrap();
    variable.submit("5,7").unwrap();
    assert_eq!(variable.document().entities.len(), 3);
    assert_eq!(variable.coordinator().undo_len(), 1);

    let (mut chamfer, _) = build();
    chamfer
        .dispatch("OM9-SURFACE-024", InvocationOrigin::Menu)
        .unwrap();
    chamfer.submit("1,3").unwrap();
    assert_eq!(chamfer.document().entities.len(), 3);
    assert_eq!(chamfer.coordinator().undo_len(), 1);
}

#[test]
fn sweep1_history_builds_a_multi_profile_brep_and_records_history() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep 1 history");
    let rail = polyline("rail", vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
    let first = polyline(
        "first",
        vec![
            [0.0, -2.0, -2.0],
            [0.0, 2.0, -2.0],
            [0.0, 2.0, 2.0],
            [0.0, -2.0, 2.0],
            [0.0, -2.0, -2.0],
        ],
    );
    let second = polyline(
        "second",
        vec![
            [10.0, -2.0, -2.0],
            [10.0, 2.0, -2.0],
            [10.0, 2.0, 2.0],
            [10.0, -2.0, 2.0],
            [10.0, -2.0, -2.0],
        ],
    );
    let ids = [rail.id, first.id, second.id];
    for entity in [rail, first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-102", InvocationOrigin::Menu)
        .expect("sweep 1 history");
    assert!(controller.prompt().contains("Sweep options"));
    {
        use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
        let closed = controller
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == "Flip")
            .expect("Flip option");
        assert_eq!(
            controller.activate_command_option(closed.id).unwrap(),
            CommandOptionEffect::Applied
        );
        assert!(controller.prompt().contains("Flip=Yes"));
    }
    controller.submit("").expect("sweep options");
    assert_eq!(controller.document().entities.len(), 4);
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(controller.document().history.recipes.len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("history surface");
    assert!(matches!(added.geometry, Geometry::Brep(_)));
    let recipe = *controller
        .document()
        .history
        .recipes
        .keys()
        .next()
        .expect("history recipe");
    controller.replay_history(recipe).expect("replay history");
    assert_eq!(controller.coordinator().undo_len(), 2);
}

#[test]
fn sweep2_history_builds_a_two_rail_brep_and_records_history() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep 2 history");
    let rail_a = polyline("rail a", vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
    let rail_b = polyline("rail b", vec![[0.0, 10.0, 0.0], [10.0, 10.0, 0.0]]);
    let first = polyline("first", vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]]);
    let second = polyline("second", vec![[10.0, 0.0, 0.0], [10.0, 10.0, 0.0]]);
    let ids = [rail_a.id, rail_b.id, first.id, second.id];
    for entity in [rail_a, rail_b, first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-104", InvocationOrigin::Menu)
        .expect("sweep 2 history");
    assert!(controller.prompt().contains("Sweep options"));
    controller.submit("simple,flip").expect("sweep options");
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(controller.document().history.recipes.len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("history surface");
    assert!(matches!(added.geometry, Geometry::Brep(_)));
    let recipe = *controller
        .document()
        .history
        .recipes
        .keys()
        .next()
        .expect("history recipe");
    controller.replay_history(recipe).expect("replay history");
    assert_eq!(controller.coordinator().undo_len(), 2);
}

#[test]
fn sweep4_sews_four_rail_patches_into_a_polysurface() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    fn rectangle(x: f64) -> Vec<[f64; 3]> {
        vec![
            [x, -2.0, -2.0],
            [x, 2.0, -2.0],
            [x, 2.0, 2.0],
            [x, -2.0, 2.0],
            [x, -2.0, -2.0],
        ]
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep 4");
    let rails = [
        polyline("rail 0", vec![[0.0, -2.0, -2.0], [10.0, -2.0, -2.0]]),
        polyline("rail 1", vec![[0.0, 2.0, -2.0], [10.0, 2.0, -2.0]]),
        polyline("rail 2", vec![[0.0, 2.0, 2.0], [10.0, 2.0, 2.0]]),
        polyline("rail 3", vec![[0.0, -2.0, 2.0], [10.0, -2.0, 2.0]]),
    ];
    let first = polyline("first", rectangle(0.0));
    let second = polyline("second", rectangle(10.0));
    let ids = [
        rails[0].id,
        rails[1].id,
        rails[2].id,
        rails[3].id,
        first.id,
        second.id,
    ];
    for entity in [
        rails[0].clone(),
        rails[1].clone(),
        rails[2].clone(),
        rails[3].clone(),
        first,
        second,
    ] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-119", InvocationOrigin::Menu)
        .expect("sweep 4");
    assert_eq!(controller.document().entities.len(), 7);
    assert_eq!(controller.coordinator().undo_len(), 1);
    assert_eq!(controller.document().history.recipes.len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("polysurface");
    assert!(matches!(added.geometry, Geometry::Brep(_)));
    let recipe = *controller
        .document()
        .history
        .recipes
        .keys()
        .next()
        .expect("sweep 4 recipe");
    controller.replay_history(recipe).expect("replay sweep 4");
    assert_eq!(controller.coordinator().undo_len(), 2);
}

#[test]
fn four_rail_profile_metadata_round_trips_on_a_curve() {
    let limits = Limits::default();
    let curve = Entity::new(
        "profile",
        Geometry::Curve(curve_build::polyline(&[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).unwrap()),
        None,
    );
    let mut entity = curve;
    let rails = [
        ObjectId::new(),
        ObjectId::new(),
        ObjectId::new(),
        ObjectId::new(),
    ];
    let metadata = FourRailProfile {
        rails,
        origin: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        x_axis: [1.0, 0.0, 0.0],
    };
    set_four_rail_profile(&mut entity, &metadata, &limits).unwrap();
    assert_eq!(
        four_rail_profile_of(&entity, &limits).unwrap(),
        Some(metadata)
    );
}

#[test]
fn sweep4_resolves_rails_from_four_rail_profile_metadata() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    fn rectangle(x: f64) -> Vec<[f64; 3]> {
        vec![
            [x, -2.0, -2.0],
            [x, 2.0, -2.0],
            [x, 2.0, 2.0],
            [x, -2.0, 2.0],
            [x, -2.0, -2.0],
        ]
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep 4 metadata");
    let rails = [
        polyline("rail 0", vec![[0.0, -2.0, -2.0], [10.0, -2.0, -2.0]]),
        polyline("rail 1", vec![[0.0, 2.0, -2.0], [10.0, 2.0, -2.0]]),
        polyline("rail 2", vec![[0.0, 2.0, 2.0], [10.0, 2.0, 2.0]]),
        polyline("rail 3", vec![[0.0, -2.0, 2.0], [10.0, -2.0, 2.0]]),
    ];
    let rail_ids = [rails[0].id, rails[1].id, rails[2].id, rails[3].id];
    let mut first = polyline("first", rectangle(0.0));
    let mut second = polyline("second", rectangle(10.0));
    for profile in [&mut first, &mut second] {
        set_four_rail_profile(
            profile,
            &FourRailProfile {
                rails: rail_ids,
                origin: [0.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
                x_axis: [0.0, 1.0, 0.0],
            },
            &limits,
        )
        .unwrap();
    }
    let profile_ids = [first.id, second.id];
    for entity in [
        rails[0].clone(),
        rails[1].clone(),
        rails[2].clone(),
        rails[3].clone(),
        first,
        second,
    ] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&profile_ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-119", InvocationOrigin::Menu)
        .expect("sweep 4 metadata");
    assert_eq!(controller.document().entities.len(), 7);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !rail_ids.contains(&entity.id) && !profile_ids.contains(&entity.id))
        .expect("polysurface");
    assert!(matches!(added.geometry, Geometry::Brep(_)));
}

#[test]
fn sweep2_slash_collects_two_alignment_points() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("sweep 2 slash");
    let rail_a = polyline("rail a", vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
    let rail_b = polyline("rail b", vec![[0.0, 10.0, 0.0], [10.0, 10.0, 0.0]]);
    let first = polyline("first", vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]]);
    let second = polyline("second", vec![[10.0, 0.0, 0.0], [10.0, 10.0, 0.0]]);
    let ids = [rail_a.id, rail_b.id, first.id, second.id];
    for entity in [rail_a, rail_b, first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-104", InvocationOrigin::Menu)
        .expect("sweep 2 history");
    controller.submit("slash").expect("slash option");
    assert!(controller.prompt().contains("Slash point"));
    {
        use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
        let first_point = controller
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == "First")
            .expect("First option");
        assert_eq!(
            controller.activate_command_option(first_point.id).unwrap(),
            CommandOptionEffect::Draft("First=".into())
        );
        controller.submit("First=5,0,0").unwrap();
    }
    assert!(controller.prompt().contains("second rail"));
    {
        use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
        let second_point = controller
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == "Second")
            .expect("Second option");
        assert_eq!(
            controller.activate_command_option(second_point.id).unwrap(),
            CommandOptionEffect::Draft("Second=".into())
        );
        controller.submit("Second=5,10,0").unwrap();
    }
    assert_eq!(controller.document().entities.len(), 5);
    assert_eq!(controller.document().history.recipes.len(), 1);
}

#[test]
fn four_rail_profile_command_tags_profiles_then_sweep4_uses_them() {
    fn polyline(name: &str, points: Vec<[f64; 3]>) -> Entity {
        Entity::new(
            name,
            Geometry::Curve(curve_build::polyline(&points).expect("polyline")),
            None,
        )
    }
    fn rectangle(x: f64) -> Vec<[f64; 3]> {
        vec![
            [x, -2.0, -2.0],
            [x, 2.0, -2.0],
            [x, 2.0, 2.0],
            [x, -2.0, 2.0],
            [x, -2.0, -2.0],
        ]
    }
    let limits = Limits::default();
    let mut document = Document::new("four rail profile command");
    let rails = [
        polyline("rail 0", vec![[0.0, -2.0, -2.0], [10.0, -2.0, -2.0]]),
        polyline("rail 1", vec![[0.0, 2.0, -2.0], [10.0, 2.0, -2.0]]),
        polyline("rail 2", vec![[0.0, 2.0, 2.0], [10.0, 2.0, 2.0]]),
        polyline("rail 3", vec![[0.0, -2.0, 2.0], [10.0, -2.0, 2.0]]),
    ];
    let rail_ids = [rails[0].id, rails[1].id, rails[2].id, rails[3].id];
    let first = polyline("first", rectangle(0.0));
    let second = polyline("second", rectangle(10.0));
    let profile_ids = [first.id, second.id];
    let all = [
        rail_ids[0],
        rail_ids[1],
        rail_ids[2],
        rail_ids[3],
        profile_ids[0],
        profile_ids[1],
    ];
    for entity in [
        rails[0].clone(),
        rails[1].clone(),
        rails[2].clone(),
        rails[3].clone(),
        first,
        second,
    ] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&all).unwrap();
    controller
        .dispatch("OM9-TOOLS-004", InvocationOrigin::Menu)
        .expect("four rail profile");
    for id in profile_ids {
        assert!(
            controller.document().entities[&id]
                .extensions
                .contains_key("om9.four-rail-profile")
        );
    }
    controller.select_objects(&profile_ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-119", InvocationOrigin::Menu)
        .expect("sweep 4 metadata");
    assert_eq!(controller.document().entities.len(), 7);
}

#[test]
fn extrude_all_height_uses_the_shared_option_entry() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let mut document = Document::new("extrude all option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-117", InvocationOrigin::Menu)
        .unwrap();
    let option = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Height")
        .expect("Height option");
    assert_eq!(
        controller.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("Height=".into())
    );
    controller.submit("Height=5").unwrap();
    assert_eq!(controller.document().entities.len(), 4);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn chamfer_setback_uses_the_shared_option_entry() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let mut document = Document::new("chamfer option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 5.0, 0.0], [10.0, 5.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-023", InvocationOrigin::Menu)
        .unwrap();
    let option = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Setback")
        .expect("Setback option");
    assert_eq!(
        controller.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("Setback=".into())
    );
    controller.submit("Setback=1").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn fillet_radius_options_use_the_shared_entry() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let mut document = Document::new("fillet option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-020", InvocationOrigin::Menu)
        .unwrap();
    let radius = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Radius")
        .expect("Radius option");
    assert_eq!(
        controller.activate_command_option(radius.id).unwrap(),
        CommandOptionEffect::Draft("Radius=".into())
    );
    controller.submit("Radius=6").unwrap();
    assert_eq!(controller.document().entities.len(), 3);

    let mut document = Document::new("variable fillet option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut variable = Controller::new(document, Limits::default()).unwrap();
    variable.select_objects(&ids).unwrap();
    variable
        .dispatch("OM9-SURFACE-022", InvocationOrigin::Menu)
        .unwrap();
    let start = variable
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Start")
        .expect("Start option");
    assert_eq!(
        variable.activate_command_option(start.id).unwrap(),
        CommandOptionEffect::Draft("Start=".into())
    );
    variable.submit("Start=5").unwrap();
    let end = variable
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "End")
        .expect("End option");
    assert_eq!(
        variable.activate_command_option(end.id).unwrap(),
        CommandOptionEffect::Draft("End=".into())
    );
    variable.submit("End=7").unwrap();
    assert_eq!(variable.document().entities.len(), 3);
}

#[test]
fn variable_chamfer_options_stage_start_then_end() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let mut document = Document::new("variable chamfer option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 10.0, 0.0], [10.0, 10.0, 0.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-024", InvocationOrigin::Menu)
        .unwrap();
    let start = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Start")
        .expect("Start option");
    assert_eq!(
        controller.activate_command_option(start.id).unwrap(),
        CommandOptionEffect::Draft("Start=".into())
    );
    controller.submit("Start=1").unwrap();
    assert!(controller.is_running());
    let end = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "End")
        .expect("End option");
    assert_eq!(
        controller.activate_command_option(end.id).unwrap(),
        CommandOptionEffect::Draft("End=".into())
    );
    controller.submit("End=3").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn variable_blend_options_stage_start_then_end() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    fn line(start: [f64; 3], end: [f64; 3]) -> Entity {
        Entity::new(
            "line",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        )
    }
    let mut document = Document::new("variable blend option");
    let first = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let second = line([0.0, 5.0, 5.0], [10.0, 5.0, 5.0]);
    let ids = [first.id, second.id];
    for entity in [first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-008", InvocationOrigin::Menu)
        .unwrap();
    let start = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Start")
        .expect("Start option");
    assert_eq!(
        controller.activate_command_option(start.id).unwrap(),
        CommandOptionEffect::Draft("Start=".into())
    );
    controller.submit("Start=1").unwrap();
    assert!(controller.is_running());
    let end = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "End")
        .expect("End option");
    assert_eq!(
        controller.activate_command_option(end.id).unwrap(),
        CommandOptionEffect::Draft("End=".into())
    );
    controller.submit("End=2").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn variable_offset_options_stage_start_end_axis() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut document = Document::new("variable offset option");
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-026", InvocationOrigin::Menu)
        .unwrap();
    for (name, draft, value) in [
        ("Start", "Start=", "Start=1"),
        ("End", "End=", "End=5"),
        ("Axis", "Axis=", "Axis=U"),
    ] {
        let option = controller
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == name)
            .unwrap_or_else(|| panic!("{name} option"));
        assert_eq!(
            controller.activate_command_option(option.id).unwrap(),
            CommandOptionEffect::Draft(draft.into())
        );
        controller.submit(value).unwrap();
    }
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn soft_edit_options_stage_the_base_point_and_documented_options() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut document = Document::new("soft edit option");
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller
        .dispatch("OM9-SURFACE-037", InvocationOrigin::Menu)
        .unwrap();
    for (name, value) in [
        ("U", "U=0.5"),
        ("V", "V=0.5"),
        ("Dx", "Dx=0"),
        ("Dy", "Dy=0"),
        ("Dz", "Dz=1"),
        ("UDistance", "UDistance=0.5"),
        ("VDistance", "VDistance=0.5"),
    ] {
        let option = controller
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == name)
            .unwrap_or_else(|| panic!("{name} option"));
        assert_eq!(
            controller.activate_command_option(option.id).unwrap(),
            CommandOptionEffect::Draft(format!("{name}="))
        );
        controller.submit(value).unwrap();
    }
    // The documented toggles and the direction constraint stay rows after staging.
    let copy = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Copy")
        .expect("Copy option");
    assert_eq!(copy.label, "Copy=Yes");
    assert_eq!(
        controller.activate_command_option(copy.id).unwrap(),
        CommandOptionEffect::Applied
    );
    controller.submit("Copy=Yes").unwrap();
    let fix_edges = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "FixEdges")
        .expect("FixEdges option");
    assert_eq!(
        controller.activate_command_option(fix_edges.id).unwrap(),
        CommandOptionEffect::Applied
    );
    controller.submit("FixEdges=No").unwrap();
    let constraint = controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "DirConstraint")
        .expect("DirConstraint option");
    assert_eq!(constraint.label, "DirConstraint=None");
    assert_eq!(
        controller.activate_command_option(constraint.id).unwrap(),
        CommandOptionEffect::Applied
    );
    controller.submit("DirConstraint=None").unwrap();
    controller.submit("").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
fn every_surface_row_declares_its_matrix_command_line_name() {
    let mut names = std::collections::BTreeSet::new();
    for binding in surface_menu::BINDINGS {
        assert!(
            !binding.command_line.is_empty(),
            "{} has no command name",
            binding.l3_id
        );
        assert!(
            names.insert(binding.command_line.to_ascii_lowercase()),
            "duplicate command name {}",
            binding.command_line
        );
        assert_eq!(
            surface_menu::by_command_line(binding.command_line).map(|row| row.l3_id),
            Some(binding.l3_id)
        );
        // The authored scripts read `! _Sweep1`; a pasted script must reach the row.
        assert_eq!(
            surface_menu::by_command_line(&format!("! _{}", binding.command_line))
                .map(|row| row.l3_id),
            Some(binding.l3_id)
        );
        assert_eq!(
            surface_menu::by_label(binding.label).map(|row| row.l3_id),
            Some(binding.l3_id)
        );
    }
    assert_eq!(names.len(), 38);
}

#[test]
fn every_surface_command_line_name_resolves_like_its_menu_row() {
    for binding in surface_menu::BINDINGS {
        if binding.command_line.eq_ignore_ascii_case("Rebuild") {
            // Edit ▸ Rebuild and Others ▸ Rebuild share the Matrix script `_Rebuild`; the
            // registry keeps the curve command and a surface selection picks the surface row.
            assert_eq!(
                Controller::resolve_alias(binding.command_line),
                Some("OM9-CURVE-009")
            );
            continue;
        }
        assert_eq!(
            Controller::resolve_alias(binding.command_line),
            Some(binding.feature),
            "{} ({})",
            binding.label,
            binding.command_line
        );
        if binding.label != "Soft Edit" {
            // `Soft Edit` is also a Curve menu row; the selection disambiguates it.
            assert_eq!(
                Controller::resolve_alias(binding.label),
                Some(binding.feature),
                "{} label",
                binding.label
            );
        }
    }
    let resolved = surface_menu::BINDINGS
        .iter()
        .filter(|binding| Controller::resolve_alias(binding.command_line) == Some(binding.feature))
        .count();
    assert_eq!(resolved, 37);
    assert_eq!(
        Controller::resolve_alias("Soft Edit"),
        Some("OM9-CURVE-055"),
        "the Curve row keeps the shared label without a surface selection"
    );
}

#[test]
fn typed_shared_labels_follow_the_selected_object() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("shared labels");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    assert_eq!(
        controller.resolve_typed_command("Soft Edit"),
        Some("OM9-SURFACE-037")
    );
    assert_eq!(
        controller.resolve_typed_command("Rebuild"),
        Some("OM9-SURFACE-006")
    );
    controller.submit("Soft Edit").expect("surface soft edit");
    assert!(controller.prompt().contains("Soft edit"));
    controller.cancel();
    assert!(!controller.is_running());

    let mut curve_document = Document::new("shared labels curve");
    let curve = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [1.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let curve_id = curve.id;
    curve_document.entities.insert(curve_id, curve);
    let mut curve_controller = Controller::new(curve_document, limits).unwrap();
    curve_controller.select_objects(&[curve_id]).unwrap();
    assert_eq!(
        curve_controller.resolve_typed_command("Soft Edit"),
        Some("OM9-CURVE-055")
    );
    assert_eq!(
        curve_controller.resolve_typed_command("Rebuild"),
        Some("OM9-CURVE-009")
    );
}

#[test]
fn menu_and_command_line_dispatch_the_same_surface_command() {
    for binding in surface_menu::BINDINGS {
        let mut menu = Controller::new(Document::new("parity menu"), Limits::default()).unwrap();
        let mut typed = Controller::new(Document::new("parity cmd"), Limits::default()).unwrap();
        let menu_result = menu.dispatch(binding.feature, InvocationOrigin::Menu);
        let typed_result = typed.dispatch(binding.feature, InvocationOrigin::Cmd);
        assert_eq!(
            menu_result.is_err(),
            typed_result.is_err(),
            "{} dispatch parity",
            binding.label
        );
        assert_eq!(menu.prompt(), typed.prompt(), "{} prompt", binding.label);
        assert_eq!(menu.is_running(), typed.is_running(), "{}", binding.label);
        assert_eq!(
            menu.document().entities.len(),
            typed.document().entities.len(),
            "{}",
            binding.label
        );
        assert_eq!(
            menu.coordinator().undo_len(),
            typed.coordinator().undo_len(),
            "{}",
            binding.label
        );
    }
}

#[test]
fn typed_rebuild_uses_the_surface_owner_when_a_surface_is_selected() {
    let limits = Limits::default();
    let surface = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("typed rebuild surface");
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let source = entity.id;
    document.entities.insert(source, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[source]).unwrap();
    controller.submit("Rebuild").expect("typed surface rebuild");
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| entity.id != source)
        .expect("rebuilt surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
}

#[test]
fn typed_offset_srf_runs_the_same_service_as_the_menu_row() {
    let limits = Limits::default();
    let build = || {
        let surface = surface_kernel::plane(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            &limits,
        )
        .unwrap();
        let mut document = Document::new("typed offset parity");
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        let source = entity.id;
        document.entities.insert(source, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[source]).unwrap();
        controller
    };

    let mut menu = build();
    menu.dispatch("OM9-SURFACE-025", InvocationOrigin::Menu)
        .unwrap();
    menu.submit("2").unwrap();
    let mut typed = build();
    typed.submit("OffsetSrf").expect("typed offset");
    assert!(typed.prompt().contains("Offset distance"));
    typed.submit("2").unwrap();

    assert_eq!(
        typed.document().entities.len(),
        menu.document().entities.len()
    );
    assert_eq!(
        typed.coordinator().undo_len(),
        menu.coordinator().undo_len()
    );
    assert_eq!(typed.document().entities.len(), 2);
    let added = typed
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Surface(_)))
        .count();
    assert_eq!(added, 2);
}

#[test]
fn sweep_history_offers_closed_only_for_a_closed_rail_and_two_profiles() {
    fn closed_square(x: f64) -> Entity {
        Entity::new(
            "profile",
            Geometry::Curve(
                curve_build::polyline(&[
                    [x, -1.0, -1.0],
                    [x, 1.0, -1.0],
                    [x, 1.0, 1.0],
                    [x, -1.0, 1.0],
                    [x, -1.0, -1.0],
                ])
                .expect("profile"),
            ),
            None,
        )
    }
    let limits = Limits::default();
    let mut document = Document::new("closed sweep options");
    let rail = Entity::new(
        "rail",
        Geometry::Curve(
            curve_build::circle_on_frame([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0)
                .expect("circle rail"),
        ),
        None,
    );
    let first = closed_square(10.0);
    let second = closed_square(-10.0);
    let ids = [rail.id, first.id, second.id];
    for entity in [rail, first, second] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-102", InvocationOrigin::Menu)
        .expect("sweep 1 history");
    // Matrix p183: Closed=Yes is the default when the rail is closed and two profiles exist.
    assert!(
        controller.prompt().contains("Closed=Yes"),
        "{}",
        controller.prompt()
    );
    let closed_token = controller
        .history_options_tokens()
        .into_iter()
        .find(|(name, _)| *name == "Closed")
        .expect("Closed token");
    assert_eq!(closed_token.1, "Closed=No");
    controller.submit("Closed=No").expect("toggle closed");
    assert!(
        controller.prompt().contains("Closed=No"),
        "{}",
        controller.prompt()
    );
    controller.submit("").expect("open history sweep");
    assert_eq!(controller.document().entities.len(), 4);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("history surface");
    assert!(matches!(added.geometry, Geometry::Brep(_)));
}

#[test]
fn sweep_history_hides_closed_for_an_open_rail() {
    let limits = Limits::default();
    let mut document = Document::new("open rail sweep options");
    let entities = [
        Entity::new(
            "rail",
            Geometry::Curve(
                curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).expect("rail"),
            ),
            None,
        ),
        Entity::new(
            "first",
            Geometry::Curve(
                curve_build::polyline(&[
                    [0.0, -1.0, -1.0],
                    [0.0, 1.0, -1.0],
                    [0.0, 1.0, 1.0],
                    [0.0, -1.0, 1.0],
                    [0.0, -1.0, -1.0],
                ])
                .expect("profile"),
            ),
            None,
        ),
        Entity::new(
            "second",
            Geometry::Curve(
                curve_build::polyline(&[
                    [10.0, -1.0, -1.0],
                    [10.0, 1.0, -1.0],
                    [10.0, 1.0, 1.0],
                    [10.0, -1.0, 1.0],
                    [10.0, -1.0, -1.0],
                ])
                .expect("profile"),
            ),
            None,
        ),
    ];
    let ids: Vec<_> = entities.iter().map(|entity| entity.id).collect();
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-102", InvocationOrigin::Menu)
        .expect("sweep 1 history");
    assert!(
        !controller.prompt().contains("Closed="),
        "{}",
        controller.prompt()
    );
    assert!(
        controller
            .history_options_tokens()
            .iter()
            .all(|(name, _)| *name != "Closed")
    );
    assert!(controller.submit("Closed=Yes").is_err());
    assert!(
        controller.submit("").is_ok(),
        "the default open sweep still builds"
    );
}

#[test]
fn match_surfaces_accepts_the_documented_continuity_option() {
    let limits = Limits::default();
    let first = surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let second = surface_kernel::plane(
        [10.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut document = Document::new("match surfaces continuity");
    let first = Entity::new("first", Geometry::Surface(first), None);
    let second = Entity::new("second", Geometry::Surface(second), None);
    let ids = [first.id, second.id];
    document.entities.insert(first.id, first);
    document.entities.insert(second.id, second);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    controller
        .dispatch("OM9-SURFACE-128", InvocationOrigin::Menu)
        .unwrap();
    assert!(controller.submit("Continuity=Sideways").is_err());
    assert!(controller.prompt().contains("Match source edge"));
    controller.submit("Continuity=Tangency").unwrap();
    controller.submit("U1,U0").unwrap();
    assert!(
        controller.prompt().contains("Continuity=Tangency"),
        "{}",
        controller.prompt()
    );
    controller.submit("").unwrap();
    assert_eq!(controller.document().entities.len(), 3);
    assert_eq!(controller.coordinator().undo_len(), 1);
    let added = controller
        .document()
        .entities
        .values()
        .find(|entity| !ids.contains(&entity.id))
        .expect("matched surface");
    assert!(matches!(added.geometry, Geometry::Surface(_)));
    assert!(controller.submit("U1,U0").is_err(), "no active match");
}

/// The authored Surface menu (`[Menu8]` of `Resources/menu/MainMenu.ini`) carries 40 icons while
/// the milestone plans 38 L3 rows: 36 menu commands plus the Sweep 1/2 History sub-commands. Every
/// one of the 40 icons must reach a command: the catalog rows for the 38 L3 commands plus the four
/// menu-only commands with their own owner sessions (Revolve, Heightfield from Image, Smash and
/// Refit to Tolerance).
#[test]
fn every_authored_surface_menu_icon_maps_to_an_available_command() {
    assert_eq!(menu_catalog::SURFACE.len(), 40);
    let mut wired = 0;
    for &icon in menu_catalog::SURFACE {
        let feature = surface_menu::by_icon(icon)
            .map(|binding| binding.feature)
            .or_else(|| om9_app::icons::feature_for(icon))
            .unwrap_or_else(|| panic!("{icon} has no command"));
        assert!(
            Controller::availability(feature).is_ok(),
            "{icon} -> {feature} is unavailable"
        );
        wired += 1;
    }
    assert_eq!(wired, 40);
    // The four menu-only commands answer on the typed command line too.
    assert_eq!(
        Controller::resolve_alias("Revolve"),
        Some("OM9-SURFACE-018")
    );
    assert_eq!(
        Controller::resolve_alias("Heightfield"),
        Some("OM9-SURFACE-031")
    );
    assert_eq!(Controller::resolve_alias("Smash"), Some("OM9-SURFACE-141"));
    assert_eq!(
        Controller::resolve_alias("RefitSrf"),
        Some("OM9-SURFACE-142")
    );
}
