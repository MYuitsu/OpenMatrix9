// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-09-*`: F6 context classification, custom-menu persistence and provider gates.
use om9_app::{f6::ContextKind, ui::MatrixApp};
use om9_core::model::Document;

#[test]
fn every_factory_row_has_a_group_and_a_dispatch_decision() {
    use om9_app::f6::catalog::CATALOG;
    use std::collections::{BTreeMap, BTreeSet};
    let app = MatrixApp::new(Document::new("catalog")).unwrap();
    assert_eq!(CATALOG.groups.len(), 250);
    let mut used = BTreeSet::new();
    let mut audit = BTreeMap::new();
    let mut total = 0;
    let mut categories = BTreeMap::new();
    let mut split_categories = BTreeSet::new();
    for group in &CATALOG.groups {
        assert_eq!(group.rows.len(), group.categories.len());
        for (&index, category) in group.rows.iter().zip(&group.categories) {
            *categories.entry(category.label()).or_insert(0usize) += 1;
            if CATALOG.actions[index].0 == "Split" {
                split_categories.insert(category.label());
            }
            let (label, script) = &CATALOG.actions[index];
            used.insert(index);
            if script.is_empty() {
                continue;
            }
            total += 1;
            let action = app.resolve_f6_action(script, label);
            assert!(!label.is_empty());
            let enabled = action.availability.enabled();
            let route = if action.availability.enabled() {
                action.id
            } else {
                action
                    .availability
                    .reason()
                    .expect("disabled reason")
                    .to_owned()
            };
            audit.insert((script, label), (enabled, route));
        }
    }
    assert_eq!(used.len(), CATALOG.actions.len());
    assert_eq!(total, 1822);
    assert_eq!(categories.values().sum::<usize>(), 2281);
    assert_eq!(categories["Curves"], 246);
    assert_eq!(categories["None"], 461);
    assert_eq!(
        split_categories,
        BTreeSet::from(["Art", "Curves", "Menu", "None", "Surface"])
    );
    assert_eq!(audit.len(), 424);
    assert_eq!(
        audit.keys().map(|(s, _)| *s).collect::<BTreeSet<_>>().len(),
        371
    );
    // Optional human-readable snapshot of the current built-in registry.
    if let Ok(path) = std::env::var("OM9_F6_AUDIT") {
        let mut report = String::from(
            "Factory command\tMenu label\tEnabled with empty selection\tOM9 route or disabled reason\n",
        );
        for ((script, label), (enabled, route)) in audit {
            report.push_str(&format!("{script}\t{label}\t{enabled}\t{route}\n"));
        }
        std::fs::write(path, report).unwrap();
    }
}

#[test]
fn points_off_selected_keeps_other_grip_owners_and_preserves_mixed_modes() {
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
        top11::GripMode,
    };
    let mut doc = Document::with_template_tolerance("Selective grips");
    let mut ids = Vec::new();
    let plane = om9_core::surface_kernel::plane(
        [0., 0., 3.],
        [1., 0., 0.],
        [0., 1., 0.],
        8.,
        6.,
        &Default::default(),
    )
    .unwrap();
    for geometry in [
        Geometry::Curve(Curve::Line {
            domain: [0., 1.],
            start: [0.; 3],
            end: [4., 0., 0.],
        }),
        Geometry::Curve(Curve::Line {
            domain: [0., 1.],
            start: [0., 2., 0.],
            end: [4., 2., 0.],
        }),
        Geometry::Surface(plane),
        Geometry::Point([2., 2., 2.]),
    ] {
        let entity = Entity::new("source", geometry, None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let c = &mut app.controller;
    let before = c.document().clone();
    c.select_objects(&ids[..2]).unwrap();
    c.submit("PointsOn").unwrap();
    c.select_objects(&ids[2..3]).unwrap();
    c.submit("EditPtOn").unwrap();
    c.select_objects(&ids[3..]).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert!(
        handles
            .iter()
            .any(|g| g.object == ids[2] && g.mode == GripMode::Edit)
    );
    assert_eq!(
        handles
            .iter()
            .filter(|g| g.mode == GripMode::Control)
            .count(),
        4
    );
    c.select_grips(&handles[..1], om9_app::session::GripPick::Replace);
    c.submit("_ptoffselected").unwrap();
    assert!(c.prompt().contains("turn points off"));
    assert!(c.top11_grip_handles().unwrap().is_empty());
    let scene = c.scene().unwrap();
    assert!(
        scene
            .objects
            .iter()
            .all(|o| o.visible == (o.root != ids[3]))
    );
    assert!(c.select_objects(&ids[3..]).is_err());
    c.select_objects(&ids[..1]).unwrap();
    c.submit("Esc").unwrap();
    assert_eq!(c.selected_ordered(), ids[3..]);
    assert_eq!(handles, c.top11_grip_handles().unwrap());
    assert_eq!(c.selected_grips().len(), 1);
    c.submit("_ptoffselected").unwrap();
    c.select_objects(&ids[..1]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.selected_ordered(), ids[3..]);
    let remaining = c.top11_grip_handles().unwrap();
    assert_eq!(
        remaining,
        handles
            .into_iter()
            .filter(|h| h.object != ids[0])
            .collect::<Vec<_>>()
    );
    c.submit("PointsOff").unwrap();
    assert!(c.top11_grip_handles().unwrap().is_empty());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    app.f6_mode = om9_app::f6::catalog::Mode::Curve;
    for (owner, command) in [(ids[1], "EditPtOn"), (ids[2], "PointsOn")] {
        app.controller.select_objects(&[owner]).unwrap();
        app.controller.submit(command).unwrap();
        let grip = app.controller.top11_grip_handles().unwrap()[0];
        app.controller
            .select_grips(&[grip], om9_app::session::GripPick::Replace);
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|r| r.label == "Points Off Selected")
            .unwrap();
        assert!(row.availability.enabled());
        app.controller
            .dispatch(&row.id, om9_app::session::InvocationOrigin::F6)
            .unwrap();
        app.controller.select_objects(&[owner]).unwrap();
        app.controller.submit("").unwrap();
        assert!(app.controller.top11_grip_handles().unwrap().is_empty());
    }
}

#[test]
fn factory_split_point_spellings_use_the_existing_postselection_command() {
    use om9_app::{f6::catalog::CATALOG, session::InvocationOrigin};
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("Split Point from F6");
    let line = Entity::new(
        "target",
        Geometry::Curve(Curve::Line {
            start: [0.; 3],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let id = line.id;
    doc.entities.insert(id, line);
    let mut app = MatrixApp::new(doc).unwrap();
    let original = app.controller.document().clone();
    let mut count = 0;
    for (label, script) in &CATALOG.actions {
        let words: Vec<_> = script
            .trim_start_matches(['!', '_'])
            .split_whitespace()
            .map(|w| w.trim_start_matches('_').to_ascii_lowercase())
            .collect();
        if words.len() != 2 || words[0] != "split" || !matches!(words[1].as_str(), "p" | "point") {
            continue;
        }
        app.controller.clear_selection();
        let row = app.resolve_f6_action(script, label);
        assert!(row.availability.enabled(), "{script}");
        app.controller
            .dispatch(&row.id, InvocationOrigin::F6)
            .unwrap();
        assert!(app.controller.top11_needs_object(), "{script}");
        app.controller.top11_accept_object(id).unwrap();
        app.controller.submit("").unwrap();
        app.controller.submit("W4,0,0").unwrap();
        let mut lengths: Vec<_> = app
            .controller
            .document()
            .entities
            .values()
            .map(|e| {
                let Geometry::Curve(ref curve) = e.geometry else {
                    panic!("split output")
                };
                om9_core::curve_ops::arc_length(curve, &Default::default()).unwrap()
            })
            .collect();
        lengths.sort_by(f64::total_cmp);
        assert!((lengths[0] - 4.).abs() < 1e-8 && (lengths[1] - 6.).abs() < 1e-8);
        assert_eq!(lengths.len(), 2);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &original);
        count += 1;
    }
    assert_eq!(count, 7);
}

#[test]
fn cage_grips_route_to_the_factory_group_and_share_cage_edit_deformation() {
    use om9_app::{
        f6::catalog::Mode,
        session::{GripPick, InvocationOrigin},
    };
    use om9_core::{
        cage::{box_corners, box_mesh},
        geometry::Geometry,
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("Cage edit point");
    let child = Entity::new(
        "captured",
        Geometry::Mesh(box_mesh(box_corners([0.; 3], [4., 5., 6.]))),
        None,
    );
    let id = child.id;
    doc.entities.insert(id, child);
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("Cage").unwrap();
    let before = app.controller.document().clone();
    let owner = *before.entities.keys().find(|key| **key != id).unwrap();
    app.controller.select_objects(&[owner]).unwrap();
    app.controller.submit("PointsOn").unwrap();
    let handles = app.controller.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 8);
    app.controller
        .select_grips(&[handles[0]], GripPick::Replace);
    app.f6_mode = Mode::Curve;
    let group = app.f6_context();
    let row = group
        .actions
        .iter()
        .find(|row| row.label == "Points Off Selected")
        .unwrap();
    assert!(row.availability.enabled());
    let undo = app.controller.coordinator().undo_len();
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    let scene = app.controller.scene().unwrap();
    assert!(scene.objects.iter().all(|o| o.visible == (o.root == owner)));
    app.controller.select_objects(&[owner]).unwrap();
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    assert!(app.controller.top11_grip_handles().unwrap().is_empty());
    assert_eq!(app.controller.document(), &before);
    app.controller.submit("PointsOn").unwrap();
    let h = app.controller.top11_grip_handles().unwrap()[7];
    let mut moved = h.point;
    moved[2] += 2.;
    let wires = app.controller.grip_edit_preview(&[(h,moved)]);
    assert_eq!(wires.len(), 12);
    assert_eq!(wires.iter().filter(|segment| segment.contains(&moved)).count(), 3);
    assert_eq!(app.controller.document(), &before);
    app.controller.top11_move_grip(h, moved).unwrap();
    let changed = app.controller.document();
    assert_ne!(
        changed.entities[&id].geometry,
        before.entities[&id].geometry
    );
    assert_ne!(
        changed.entities[&owner].geometry,
        before.entities[&owner].geometry
    );
    assert_eq!(app.controller.top11_grip_handles().unwrap()[7].point, moved);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn show_selected_presents_hidden_roots_and_restores_selection_on_enter_and_esc() {
    use om9_core::{geometry::Geometry, model::Entity};
    let mut doc = Document::with_template_tolerance("ShowSelected");
    let mut ids = Vec::new();
    let mut layer = om9_core::model::Layer::new("off", [160; 4]);
    layer.visible = false;
    let layer_id = layer.id;
    doc.layers.insert(layer.id, layer);
    doc.layer_order.push(layer_id);
    for (x, name) in [
        "visible",
        "hidden first",
        "hidden second",
        "explicitly invisible",
        "locked",
        "layer off",
    ]
    .iter()
    .enumerate()
    {
        let mut entity = Entity::new(*name, Geometry::Point([x as f64, 0., 0.]), None);
        entity.attributes.hidden = x > 0;
        if x == 5 {
            entity.attributes.layer = Some(layer_id);
        }
        entity.attributes.visible = x != 3;
        entity.attributes.locked = x == 4;
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let before = app.controller.document().clone();
    let initial_scene = app.controller.scene().unwrap().publication_id();
    app.controller.select_objects(&ids[..1]).unwrap();
    let action = app.resolve_f6_action("_ShowSelected", "ShowSelected");
    assert!(action.availability.enabled());
    app.controller
        .dispatch(&action.id, om9_app::session::InvocationOrigin::F6)
        .unwrap();
    assert!(app.controller.prompt().contains("objects to show"));
    let scene = app.controller.scene().unwrap();
    assert_ne!(scene.publication_id(), initial_scene);
    for (index, id) in ids.iter().enumerate() {
        let object = scene.objects.iter().find(|o| o.root == *id).unwrap();
        assert_eq!(object.visible, matches!(index, 1 | 2 | 4));
        assert_eq!(object.selectable, matches!(index, 1 | 2));
    }
    assert_eq!(app.controller.document(), &before);
    assert!(app.controller.select_objects(&ids[..1]).is_err());
    app.controller.select_objects(&ids[1..2]).unwrap();
    app.controller.cancel();
    assert_eq!(
        app.controller.scene().unwrap().publication_id(),
        initial_scene
    );
    assert_eq!(app.controller.selected_ordered(), ids[..1]);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.submit("_ShowSelected").unwrap();
    app.controller.select_objects(&ids[2..3]).unwrap();
    app.controller.submit("").unwrap();
    assert!(!app.controller.is_running());
    assert!(
        !app.controller.document().entities[&ids[2]]
            .attributes
            .hidden
    );
    for index in [1, 3, 4, 5] {
        assert!(
            app.controller.document().entities[&ids[index]]
                .attributes
                .hidden
        );
    }
    assert_eq!(app.controller.selected_ordered(), ids[..1]);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn mesh_outline_matches_measured_rhino5_world_contours() {
    use om9_core::{
        geometry::{Geometry, Mesh, Point3, Transform},
        model::Entity,
    };
    use om9_scene::{
        camera::{CPlane, Camera},
        mesh_outline::outlines,
    };
    #[derive(serde::Deserialize)]
    struct Trial {
        mode: u8,
        perspective: bool,
        eye: Point3,
        direction: Point3,
        near: f64,
        far: f64,
        shifts: Vec<Point3>,
        success: bool,
        contours: Vec<Vec<Point3>>,
    }
    #[derive(serde::Deserialize)]
    struct Fixture {
        vertices: Vec<Point3>,
        triangles: Vec<Vec<u32>>,
        plane_origin: Point3,
        trials: Vec<Trial>,
    }
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../implementation/fixtures/M02/OM9-L3-03-09-003/mesh-outline-rhino5.json"
    ))
    .unwrap();
    let plane = CPlane::world_xy()
        .with_origin(fixture.plane_origin)
        .unwrap();
    let mesh = Mesh {
        vertices: fixture.vertices,
        faces: fixture.triangles,
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    fn squared_distance_to_path(p: Point3, path: &[Point3]) -> f64 {
        path.iter()
            .zip(path.iter().cycle().skip(1))
            .map(|(a, b)| {
                let d: Point3 = std::array::from_fn(|i| b[i] - a[i]);
                let v: Point3 = std::array::from_fn(|i| p[i] - a[i]);
                let length = d.iter().map(|x| x * x).sum::<f64>();
                let t = if length == 0. {
                    0.
                } else {
                    ((0..3).map(|i| d[i] * v[i]).sum::<f64>() / length).clamp(0., 1.)
                };
                (0..3).map(|i| (v[i] - t * d[i]).powi(2)).sum::<f64>()
            })
            .fold(f64::INFINITY, f64::min)
    }
    for trial in fixture.trials {
        let mut doc = Document::with_template_tolerance("Rhino 5 oracle");
        for shift in trial.shifts {
            let mut e = Entity::new("sample", Geometry::Mesh(mesh.clone()), None);
            e.placement = Transform::translation(shift);
            doc.entities.insert(e.id, e);
        }
        let app = MatrixApp::new(doc).unwrap();
        let objects = app
            .controller
            .scene()
            .unwrap()
            .objects
            .iter()
            .collect::<Vec<_>>();
        let target = std::array::from_fn(|i| trial.eye[i] + trial.direction[i]);
        let mut camera = if trial.perspective {
            Camera::perspective(trial.eye, target, [0., 1., 0.], 1.)
        } else {
            Camera::orthographic(trial.eye, target, [0., 1., 0.], 4.)
        }
        .unwrap();
        camera.near_mm = trial.near;
        camera.far_mm = trial.far;
        let result = outlines(&camera, plane, &objects, &Default::default());
        if !trial.success {
            assert!(result.is_err(), "trial {}", trial.mode);
            continue;
        }
        let paths = result.unwrap();
        assert_eq!(paths.len(), trial.contours.len(), "trial {}", trial.mode);
        for expected in trial.contours {
            assert!(
                paths.iter().any(|path| {
                    [
                        (path.as_slice(), expected.as_slice()),
                        (expected.as_slice(), path.as_slice()),
                    ]
                    .into_iter()
                    .all(|(a, b)| {
                        a.iter().zip(a.iter().cycle().skip(1)).all(|(p, q)| {
                            let mid = std::array::from_fn(|i| (p[i] + q[i]) * 0.5);
                            squared_distance_to_path(*p, b) < 1e-12
                                && squared_distance_to_path(mid, b) < 1e-12
                        })
                    })
                }),
                "trial {} expected {expected:?}, got {paths:?}",
                trial.mode
            );
        }
    }
}

#[test]
fn mesh_outline_f6_and_cmd_preserve_holes_concavity_and_source_identity() {
    use om9_app::session::InvocationOrigin;
    use om9_core::{
        geometry::{Curve, Geometry, Mesh, Transform},
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("MeshOutline selection");
    let xy = [
        [0., 0.],
        [4., 0.],
        [4., 4.],
        [0., 4.],
        [1., 1.],
        [3., 1.],
        [3., 3.],
        [1., 3.],
        [7., 1.],
        [7., 2.],
        [6., 2.],
        [6., 0.],
        [8., 0.],
        [8., 1.],
    ];
    let mesh = Mesh {
        vertices: xy.map(|[x, y]| [x, y, 2.]).to_vec(),
        faces: vec![
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
            vec![4, 5, 1, 0],
            vec![8, 9, 10, 11],
            vec![8, 11, 12, 13],
        ],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let mut entity = Entity::new("ring and concave island", Geometry::Mesh(mesh), None);
    entity.placement = Transform::translation([12., -4., 6.]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let plane = om9_core::surface_kernel::plane(
        [12., -4., 1.],
        [1., 0., 0.],
        [0., 1., 0.],
        2.,
        2.,
        &Default::default(),
    )
    .unwrap();
    let surface = Entity::new(
        "overlapping separate surface",
        Geometry::Surface(plane),
        None,
    );
    let surface_id = surface.id;
    doc.entities.insert(surface_id, surface);
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller.activate_view("Looking Down").unwrap();
    let before = app.controller.document().clone();
    app.controller.select_objects(&[id]).unwrap();
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Mesh Outline")
        .unwrap();
    assert!(row.availability.enabled());
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    let measure = |app: &MatrixApp| {
        let mut areas = vec![];
        for obj in app
            .controller
            .document()
            .entities
            .values()
            .filter(|o| !before.entities.contains_key(&o.id))
        {
            let Geometry::Curve(Curve::Nurbs(c)) = &obj.geometry else {
                panic!("expected a polyline");
            };
            assert_eq!(c.degree, 1);
            assert_eq!(c.poles.first(), c.poles.last());
            assert!(
                c.poles
                    .iter()
                    .all(|p| p[2].abs() < 1e-9 && p[0] >= 12. - 1e-9 && p[1] >= -4. - 1e-9)
            );
            areas.push(
                c.poles
                    .windows(2)
                    .map(|p| p[0][0] * p[1][1] - p[1][0] * p[0][1])
                    .sum::<f64>()
                    / 2.,
            );
        }
        areas.sort_by(f64::total_cmp);
        areas
    };
    let areas = measure(&app);
    assert_eq!(areas.len(), 3);
    assert_eq!(app.controller.selected().len(), 3);
    assert!(!app.controller.selected().contains(&id));
    for (area, n) in areas.into_iter().zip([-4., 3., 16.]) {
        assert!((area - n).abs() < 1e-7, "{area} vs {n}");
    }
    assert_eq!(
        app.controller.document().entities[&id],
        before.entities[&id]
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.select_objects(&[]).unwrap();
    app.controller.submit("_MeshOutline").unwrap();
    assert!(app.controller.addon_collects_objects());
    app.controller.cancel();
    assert_eq!(app.controller.document(), &before);
    app.controller.submit("_MeshOutline").unwrap();
    app.controller.select_objects(&[id, surface_id]).unwrap();
    app.controller.submit("").unwrap();
    let areas = measure(&app);
    assert_eq!(areas.len(), 3, "{areas:?}");
    for (area, n) in areas.into_iter().zip([-3., 3., 16.]) {
        assert!((area - n).abs() < 1e-7, "{area} vs {n}");
    }
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
#[cfg(feature = "occt-brep")]
fn mesh_outline_of_brep_uses_active_side_plane_and_document_units() {
    use om9_core::geometry::{Curve, Geometry, Unit};
    let mut doc = Document::with_template_tolerance("BRep outline in inches");
    doc.page_unit = Unit::Inch;
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller.activate_view("Looking Down").unwrap();
    for input in ["Box", "1,2,3", "3,5,3", "4"] {
        app.controller.submit(input).unwrap();
    }
    let id = *app.controller.document().entities.keys().next().unwrap();
    assert!(matches!(
        app.controller.document().entities[&id].geometry,
        Geometry::Brep(_)
    ));
    let original = app.controller.document().clone();
    app.controller.activate_view("Side View").unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("_MeshOutline").unwrap();
    let c = app
        .controller
        .document()
        .entities
        .values()
        .find(|o| o.id != id)
        .unwrap();
    let Geometry::Curve(Curve::Nurbs(curve)) = &c.geometry else {
        panic!("side polyline");
    };
    assert!(curve.poles.iter().all(|p| p[0].abs() < 1e-8));
    let area = curve
        .poles
        .windows(2)
        .map(|p| p[0][1] * p[1][2] - p[0][2] * p[1][1])
        .sum::<f64>()
        .abs()
        / 2.;
    let square_inches = area / Unit::Inch.factor().powi(2);
    assert!(
        (square_inches - 12.).abs() < 1e-7,
        "{square_inches} square inches"
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
}

#[test]
fn factory_reduce_mesh_uses_preview_geometry_and_preserves_selection_sources() {
    use om9_app::{f6::catalog::Mode, session::InvocationOrigin};
    use om9_core::{
        budget::Limits, geometry::Geometry, mesh_repair::reduction::triangle_count, model::Entity,
        surface_kernel, utilities::mesh_from_surface,
    };
    let limits = Limits::default();
    let plane =
        surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 4., 4., &limits).unwrap();
    let mut doc = Document::with_template_tolerance("Reduce Mesh");
    let ids = [4, 2].map(|n| {
        let entity = Entity::new(
            "open mesh",
            Geometry::Mesh(mesh_from_surface(&plane, n, n, &limits).unwrap()),
            None,
        );
        let id = entity.id;
        doc.entities.insert(id, entity);
        id
    });
    let mut app = MatrixApp::new(doc).unwrap();
    let before = app.controller.document().entities.clone();
    let wait = |app: &mut MatrixApp| {
        let limit = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while app.controller.reduce_mesh_worker_pending() {
            if let Some(result) = app.controller.poll_reduce_mesh() {
                result.unwrap();
            }
            assert!(std::time::Instant::now() < limit);
            std::thread::yield_now();
        }
    };
    app.f6_mode = Mode::Surface;
    app.controller.select_objects(&ids).unwrap();
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|r| r.label == "Reduce Mesh")
        .unwrap();
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    assert!(
        app.controller
            .addon_dialog()
            .unwrap()
            .footer
            .iter()
            .any(|s| s.contains("40"))
    );
    app.controller
        .set_addon_dialog_value("Polygons", "20")
        .unwrap();
    app.controller.submit("PlanarOnly=Yes").unwrap();
    app.controller.submit("Accuracy=10").unwrap();
    app.controller
        .set_addon_dialog_value("Preview", "")
        .unwrap();
    wait(&mut app);
    let preview = app
        .controller
        .addon_session()
        .unwrap()
        .reply
        .preview
        .clone();
    assert_eq!(preview.len(), 2);
    assert_eq!(before, app.controller.document().entities);
    assert!(!app.controller.addon_session().unwrap().lines.is_empty());
    app.controller.submit("").unwrap();
    assert!(app.controller.addon_session().is_none());
    let mut counts = Vec::new();
    for id in ids {
        let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else {
            panic!("mesh")
        };
        counts.push(triangle_count(mesh));
        assert!(preview.iter().any(|g|matches!(g,om9_addon_abi::session::Geo::Mesh{vertices,triangles,..}
            if vertices==&mesh.vertices && triangles.iter().map(|f|f.to_vec()).collect::<Vec<_>>()==mesh.faces)));
    }
    assert!(counts.iter().sum::<usize>() <= 20 && counts[0] > counts[1]);
    app.controller.undo().unwrap();
    assert_eq!(before, app.controller.document().entities);
    app.controller.clear_selection();
    app.controller.submit("_ReduceMesh").unwrap();
    assert!(app.controller.addon_collects_objects());
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("").unwrap();
    app.controller.submit("Percent=25").unwrap();
    assert!(app.controller.submit("Percent=101").is_err());
    app.controller.submit("Preview").unwrap();
    app.controller.cancel();
    assert!(!app.controller.reduce_mesh_worker_pending());
    assert_eq!(before, app.controller.document().entities);
    app.controller.submit("_ReduceMesh").unwrap();
    app.controller.submit("Percent=25").unwrap();
    app.controller.submit("").unwrap();
    wait(&mut app);
    assert!(app.controller.addon_session().is_none());
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities[id].geometry != before[id].geometry)
    );
    app.controller.undo().unwrap();
    assert_eq!(before, app.controller.document().entities);
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("_ReduceMesh").unwrap();
    app.controller.submit("Preview").unwrap();
    wait(&mut app);
    let stamp = app.controller.document().stamp(ids[0]).unwrap();
    app.controller
        .set_objects_hidden(vec![stamp], true)
        .unwrap();
    assert!(app.controller.submit("").is_err());
    assert!(app.controller.addon_session().is_none());
    assert_eq!(
        app.controller.document().entities[&ids[0]].geometry,
        before[&ids[0]].geometry
    );
    assert!(
        app.controller.document().entities[&ids[0]]
            .attributes
            .hidden
    );
}

#[test]
fn factory_group_and_visibility_commands_share_cmd_and_undo() {
    use om9_app::session::InvocationOrigin;
    let mut app = MatrixApp::new(Document::with_template_tolerance("groups")).unwrap();
    for input in ["Point", "0,0,0", "Point", "1,2,0"] {
        app.controller.submit(input).unwrap();
    }
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&ids).unwrap();
    let group = app.resolve_f6_action("_group", "Group");
    app.controller
        .dispatch(&group.id, InvocationOrigin::F6)
        .unwrap();
    assert_eq!(app.controller.document().groups_of(ids[0]).len(), 1);
    app.controller.submit("_ungroup").unwrap();
    assert!(app.controller.document().groups_of(ids[0]).is_empty());
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().groups_of(ids[0]).len(), 1);
    app.controller.select_objects(&ids).unwrap();
    app.controller
        .dispatch("Lock", InvocationOrigin::F6)
        .unwrap();
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities[id].attributes.locked)
    );
    let doc = app.controller.document();
    let source = doc.stamp(ids[0]).unwrap();
    let mut attributes = doc.entities[&ids[0]].attributes.clone();
    attributes.locked = false;
    attributes.color = Some([13, 21, 34, 255]);
    let request = om9_core::transaction::TransactionRequest::new(
        "Invalid unlock",
        vec![om9_core::transaction::Mutation::Attributes { source, attributes }],
    );
    assert!(app.controller.coordinator().prepare(request).is_err());
    app.controller.submit("_Unlock").unwrap();
    assert!(
        ids.iter()
            .all(|id| !app.controller.document().entities[id].attributes.locked)
    );
    app.controller.select_objects(&ids).unwrap();
    app.controller
        .dispatch("Hide", InvocationOrigin::F6)
        .unwrap();
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities[id].attributes.hidden)
    );
    assert!(app.controller.selected().is_empty());
    app.controller.submit("_Show").unwrap();
    assert!(
        ids.iter()
            .all(|id| !app.controller.document().entities[id].attributes.hidden)
    );
    app.controller.undo().unwrap();
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities[id].attributes.hidden)
    );
}

#[test]
fn legacy_menu_import_survives_reload_and_rejects_partial_input_atomically() {
    use om9_app::f6::{CustomMenu, catalog::Mode};
    let fixture = "MatrixObjectAction v:1.0\nDialogName:Curve\nGroup:Empty\nActions:\nn:Draw a line: t:Curves: c:_Line; b: :\n";
    let mut app = MatrixApp::new(Document::new("legacy")).unwrap();
    app.f6_mode = Mode::Curve;
    assert_eq!(
        app.f6_custom.import_legacy(Mode::Curve, fixture).unwrap(),
        (1, 0)
    );
    app.stage_project_state().unwrap();
    app.f6_custom = CustomMenu::default();
    app.reload_project_state().unwrap();
    let actions = app.f6_context().actions;
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].label, "Draw a line");
    assert_eq!(actions[0].category, om9_app::f6::catalog::Category::Curves);
    let old: om9_app::f6::CustomEntry = serde_json::from_value(serde_json::json!({
        "mode":"Curve", "group":"Empty", "global":false, "label":"Line", "command":"_Line"
    }))
    .unwrap();
    assert_eq!(old.category, om9_app::f6::catalog::Category::Menu);
    assert!(actions[0].availability.enabled());
    let before = app.f6_custom.clone();
    assert!(
        app.f6_custom
            .import_legacy(
                Mode::Curve,
                &format!("{fixture}Group:Empty\nActions:\nn:Broken")
            )
            .is_err()
    );
    assert_eq!(before, app.f6_custom);
    if let Ok(directory) = std::env::var("OM9_F6_LEGACY_FIXTURES") {
        for (mode, file) in [
            (Mode::General, "objectActions"),
            (Mode::Curve, "curveLayout"),
            (Mode::Gem, "gemLayout"),
            (Mode::Surface, "surfaceModeling"),
            (Mode::TSplines, "tsplines"),
        ] {
            let text = std::fs::read_to_string(
                std::path::Path::new(&directory).join(format!("{file}.menu")),
            )
            .unwrap();
            let (count, skipped) = app.f6_custom.import_legacy(mode, &text).unwrap();
            assert!(count > 0);
            println!(
                "{} legacy groups imported {count}, unmatched {skipped}",
                mode.label()
            );
        }
    }
}

#[test]
fn factory_set_point_z_moves_only_picked_grips_and_undo_restores_the_curve() {
    use om9_app::{
        f6::catalog::Mode,
        session::{GripPick, InvocationOrigin},
    };
    use om9_core::top11::Top11Action;
    let mut app = MatrixApp::new(Document::with_template_tolerance("Set Point Z")).unwrap();
    for s in ["Polyline", "1,2,3", "4,5,6", "7,8,9", ""] {
        app.controller.submit(s).unwrap();
    }
    let id = *app.controller.document().entities.keys().next().unwrap();
    let before = app.controller.document().entities[&id].geometry.clone();
    app.controller.select_objects(&[id]).unwrap();
    app.controller
        .top11_invoke(Top11Action::ControlPoints)
        .unwrap();
    let handles = app.controller.top11_grip_handles().unwrap();
    app.controller
        .select_grips(&handles[1..], GripPick::Replace);
    app.f6_mode = Mode::Curve;
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Set Point Z")
        .unwrap();
    assert!(row.availability.enabled());
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    let handles = app.controller.top11_grip_handles().unwrap();
    assert_eq!(
        handles.iter().map(|h| h.point).collect::<Vec<_>>(),
        vec![[1., 2., 3.], [4., 5., 0.], [7., 8., 0.]]
    );
    app.controller.undo().unwrap();
    assert_eq!(before, app.controller.document().entities[&id].geometry);
}

#[test]
fn factory_gem_groups_follow_the_compiled_one_two_and_three_stone_branches() {
    use om9_app::f6::catalog::{CATALOG, Mode};
    use om9_core::{
        gem::{GemRecord, GemSpec, build_mesh},
        geometry::Geometry,
        model::{Entity, GEM_EXTENSION_SLOT},
    };
    let mut doc = Document::with_template_tolerance("F6 gems");
    let mut ids = Vec::new();
    for diameter in [2., 3., 4.] {
        let spec = GemSpec::round(diameter, diameter * 0.65).unwrap();
        let mut entity = Entity::new(
            "Round",
            Geometry::Mesh(build_mesh(&spec).unwrap().mesh),
            None,
        );
        entity.extensions.insert(
            GEM_EXTENSION_SLOT.into(),
            GemRecord::new(spec).as_extension().unwrap(),
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    for (count, expected) in [
        (0, "Empty"),
        (1, "ObjGem"),
        (2, "ObjGemsAll3"),
        (3, "ObjGemsAll3"),
    ] {
        let group = CATALOG.group(Mode::Gem, &doc, &ids[..count]).unwrap();
        assert_eq!(group.name, expected);
    }
}

#[test]
fn two_curves_join_through_the_factory_row_and_undo_together() {
    use om9_app::{f6::catalog::Mode, session::InvocationOrigin};
    let mut app = MatrixApp::new(Document::with_template_tolerance("join")).unwrap();
    for (a, b) in [("0,0,0", "2,0,0"), ("2,0,0", "2,3,0")] {
        for text in ["Line", a, b] {
            app.controller.submit(text).unwrap();
        }
    }
    let sources: Vec<_> = app.controller.document().entities.keys().copied().collect();
    app.controller.select_objects(&sources).unwrap();
    app.f6_mode = Mode::Curve;
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Join")
        .unwrap();
    assert!(row.availability.enabled());
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    let doc = app.controller.document();
    assert_eq!(doc.entities.len(), 1);
    let om9_core::geometry::Geometry::Curve(curve) =
        &doc.entities.values().next().unwrap().geometry
    else {
        panic!("joined curve expected")
    };
    let pts = om9_core::surface_kernel::sample_curve(curve, 17, &Default::default()).unwrap();
    let endpoints = [*pts.first().unwrap(), *pts.last().unwrap()];
    assert!(endpoints.contains(&[0., 0., 0.]) && endpoints.contains(&[2., 3., 0.]));
    app.controller.undo().unwrap();
    assert!(
        sources
            .iter()
            .all(|id| app.controller.document().entities.contains_key(id))
    );
}

#[test]
fn f6_uses_the_selected_class_and_empty_general_context() {
    let mut app = MatrixApp::new(Document::new("f6 context")).unwrap();
    assert_eq!(app.f6_context().kind, ContextKind::General);
    assert!(app.f6_context().enabled_count() >= 1);

    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.controller.submit("10,0,0").unwrap();
    let line = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[line]).unwrap();
    assert_eq!(app.f6_context().kind, ContextKind::Curve);
    let context = app.f6_context();
    let enabled: Vec<_> = context
        .actions
        .iter()
        .filter(|action| action.availability.enabled())
        .map(|action| action.id.as_str())
        .collect();
    assert!(enabled.contains(&om9_app::session::gem_placement::ON_CURVE));
    app.f6_mode = om9_app::f6::catalog::Mode::Curve;
    let one_curve = app.f6_context();
    assert!(
        one_curve
            .actions
            .iter()
            .any(|a| a.id == "OM9-CURVE-018" && a.label == "Offset" && a.availability.enabled())
    );
    assert!(
        context
            .actions
            .iter()
            .filter(|action| !action.availability.enabled())
            .all(|action| action.availability.reason().is_some())
    );
}

#[test]
fn a_custom_menu_persists_in_the_project_with_stable_order_and_stale_disable() {
    let mut app = MatrixApp::new(Document::new("f6 custom")).unwrap();
    app.f6_custom.command_ids = vec![
        om9_app::session::POINT.to_owned(),
        "DeletedCommand".to_owned(),
        om9_app::session::LINE.to_owned(),
    ];
    app.stage_project_state().unwrap();
    app.f6_custom.command_ids.clear();
    app.reload_project_state().unwrap();
    assert_eq!(
        app.f6_custom.command_ids,
        vec![
            om9_app::session::POINT.to_owned(),
            "DeletedCommand".to_owned(),
            om9_app::session::LINE.to_owned(),
        ]
    );
    let context = app.f6_custom.context();
    assert_eq!(context.kind, ContextKind::Custom);
    assert!(context.actions[0].availability.enabled());
    assert!(context.actions[1].availability.reason().is_some());
    assert!(context.actions[2].availability.enabled());
}

#[test]
fn tspline_box_face_from_both_display_triangles_deletes_one_quad_via_f6_and_cmd() {
    use om9_app::{
        f6::catalog::Mode,
        session::{InvocationOrigin, SelectedSubobject},
    };
    use om9_core::geometry::Geometry;
    use om9_scene::{
        camera::{Camera, Viewport},
        scene::{PickIntent, PickKind, Subelement},
    };
    let mut app = MatrixApp::new(Document::with_template_tolerance("T-Spline face pick")).unwrap();
    let c = &mut app.controller;
    c.submit("tsPlane").unwrap();
    c.submit("OutputType=Box").unwrap();
    c.submit("XFaces=3").unwrap();
    c.submit("YFaces=2").unwrap();
    c.session_point([0.; 3]).unwrap();
    c.session_point([12., 8., 0.]).unwrap();
    let id = *c.selected().first().unwrap();
    assert_eq!(
        om9_app::f6::catalog::CATALOG
            .group(Mode::General, c.document(), &[id])
            .unwrap()
            .name,
        "TSplinesFlat"
    );
    let archive = om9_archive::Archive::encode(c.document(), &Default::default()).unwrap();
    let reopened = om9_archive::Archive::decode(&archive, &Default::default()).unwrap();
    let first = reopened.entities[&id].clone();
    let Geometry::Mesh(original) = &first.geometry else {
        panic!("expected cage");
    };
    assert_eq!(original.faces.len(), 6);
    assert!(original.faces.iter().all(|f| f.len() == 4));
    let mut app = MatrixApp::new(reopened).unwrap();
    let camera = Camera::orthographic([6., 4., 30.], [6., 4., 0.], [0., 1., 0.], 16.).unwrap();
    let viewport = Viewport {
        origin: [0.; 2],
        size: [400.; 2],
    };
    for (component, expected) in [
        (Subelement::MeshVertex(0), "Grow Selection"),
        (Subelement::MeshEdge { a: 0, b: 1 }, "Ring Select"),
    ] {
        app.controller.select_subobject(id, component).unwrap();
        for mode in [Mode::General, Mode::TSplines] {
            app.f6_mode = mode;
            let menu = app.f6_context();
            assert!(menu.actions.iter().any(|a| a.label == expected));
            assert!(!menu.actions.iter().any(|a| a.label == "Delete Face(s)"));
        }
        app.controller.clear_subselection();
    }
    let mut old_ref = None;
    for world in [[3., 1., 0.], [1., 3., 0.]] {
        let at = camera.project(world, viewport).unwrap().unwrap().screen;
        let hit = app
            .controller
            .scene()
            .unwrap()
            .pick_kind(
                at,
                &camera,
                viewport,
                1.,
                PickIntent::Select,
                PickKind::Face,
            )
            .unwrap()
            .unwrap();
        assert_eq!((hit.root, &hit.subset), (id, &Subelement::MeshFace(0)));
        let reference = SelectedSubobject::from_hit(&hit);
        app.controller
            .select_subobject_ref(reference.clone())
            .unwrap();
        old_ref = Some(reference);
    }
    assert_eq!(app.controller.subselection().len(), 1);
    for mode in [Mode::General, Mode::TSplines] {
        app.f6_mode = mode;
        assert!(
            app.f6_context()
                .actions
                .iter()
                .any(|a| a.label == "Delete Face(s)" && a.availability.enabled())
        );
    }
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Delete Face(s)")
        .unwrap();
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    let after = app.controller.document().entities[&id].geometry.clone();
    let Geometry::Mesh(part) = &after else {
        panic!("expected cage");
    };
    assert_eq!(part.faces.len(), 5);
    assert!(!part.vertices.contains(&[0.; 3]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert!(app.controller.subselection().is_empty());
    assert!(
        app.controller
            .select_subobject_ref(old_ref.unwrap())
            .is_err()
    );
    assert!(
        app.controller
            .dispatch(&row.id, InvocationOrigin::F6)
            .is_err()
    );
    app.controller.undo().unwrap();
    assert_eq!(
        app.controller.document().entities[&id].geometry,
        first.geometry
    );
    app.controller
        .select_subobject(id, Subelement::MeshFace(0))
        .unwrap();
    app.controller.submit("_Delete").unwrap();
    assert_eq!(app.controller.document().entities[&id].geometry, after);
    app.controller.undo().unwrap();
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities[&id].geometry, after);
}

#[test]
fn ordinary_quad_selection_does_not_enable_the_tspline_face_action() {
    use om9_core::{
        geometry::{Geometry, Mesh},
        model::Entity,
    };
    use om9_scene::scene::Subelement;
    let mut doc = Document::with_template_tolerance("ordinary quad");
    let entity = Entity::new(
        "T-Spline Plane",
        Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.]],
            faces: vec![vec![0, 1, 2, 3]],
            normals: vec![],
            colors: vec![],
            corner_uvs: vec![],
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller
        .select_subobject(id, Subelement::MeshFace(0))
        .unwrap();
    assert!(
        !app.resolve_f6_action("_Delete", "Delete Face(s)")
            .availability
            .enabled()
    );
    assert!(
        !app.f6_context()
            .actions
            .iter()
            .any(|a| a.label == "Delete Face(s)")
    );
}

#[test]
fn duplicate_tspline_faces_selects_new_cages_and_preserves_both_sources() {
    use om9_app::session::InvocationOrigin;
    use om9_core::geometry::Geometry;
    use om9_scene::scene::Subelement;
    let mut app = MatrixApp::new(Document::with_template_tolerance("Duplicate faces")).unwrap();
    let mut ids = Vec::new();
    for x in [0., 20.] {
        app.controller
            .dispatch("OM9-TSPLINE-048", InvocationOrigin::Menu)
            .unwrap();
        app.controller.session_point([x, 0., 0.]).unwrap();
        app.controller.session_point([x + 10., 10., 0.]).unwrap();
        ids.push(*app.controller.selected().first().unwrap());
    }
    let before = app.controller.document().entities.clone();
    for &id in &ids {
        app.controller
            .select_subobject(id, Subelement::MeshFace(0))
            .unwrap();
        app.controller
            .select_subobject(id, Subelement::MeshFace(1))
            .unwrap();
    }
    let row = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Duplicate Face(s)")
        .unwrap();
    assert!(row.availability.enabled());
    app.controller
        .dispatch(&row.id, InvocationOrigin::F6)
        .unwrap();
    assert_eq!(app.controller.document().entities.len(), 4);
    assert_eq!(app.controller.selected().len(), 2);
    for id in app.controller.selected() {
        assert!(!ids.contains(id));
        let Geometry::Mesh(part) = &app.controller.document().entities[id].geometry else {
            panic!("expected cage");
        };
        assert_eq!((part.vertices.len(), part.faces.len()), (6, 2));
        assert!(part.faces.iter().all(|f| f.len() == 4));
    }
    for id in &ids {
        assert_eq!(app.controller.document().entities[id], before[id]);
    }
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities, before);
    for &id in &ids {
        for face in 0..4 {
            app.controller
                .select_subobject(id, Subelement::MeshFace(face))
                .unwrap();
        }
    }
    app.controller.submit("_Delete").unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities, before);
}

#[test]
fn ring_select_follows_opposite_cage_edges_to_both_boundaries() {
    use om9_app::{f6::catalog::Mode, session::InvocationOrigin};
    use om9_scene::scene::Subelement;
    use std::collections::BTreeSet;
    let mut app = MatrixApp::new(Document::with_template_tolerance("T-Spline edge ring")).unwrap();
    let c = &mut app.controller;
    c.submit("tsPlane").unwrap();
    c.submit("XFaces=3").unwrap();
    c.submit("YFaces=2").unwrap();
    c.session_point([0.; 3]).unwrap();
    c.session_point([12., 8., 0.]).unwrap();
    let id = *c.selected().first().unwrap();
    let original = c.document().clone();
    let history = c.coordinator().undo_len();
    for (mode, seed, cmd) in [
        (Mode::General, (3, 4), false),
        (Mode::TSplines, (9, 10), true),
    ] {
        app.f6_mode = mode;
        app.controller.clear_subselection();
        app.controller
            .select_subobject(
                id,
                Subelement::MeshEdge {
                    a: seed.0,
                    b: seed.1,
                },
            )
            .unwrap();
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|a| a.label == "Ring Select")
            .unwrap();
        assert!(row.availability.enabled());
        if cmd {
            app.controller.submit("-_tsSelection _SelectRing").unwrap();
        } else {
            app.controller
                .dispatch(&row.id, InvocationOrigin::F6)
                .unwrap();
        }
        let selected = app
            .controller
            .subselection()
            .iter()
            .map(|s| {
                assert_eq!(s.root, id);
                let Subelement::MeshEdge { a, b } = s.subset else {
                    panic!("expected edge");
                };
                (a, b)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(selected, BTreeSet::from([(0, 1), (3, 4), (6, 7), (9, 10)]));
        assert_eq!(app.controller.document(), &original);
        assert_eq!(app.controller.coordinator().undo_len(), history);
    }
}

#[test]
fn cage_loop_grow_and_shrink_follow_factory_components_without_document_history() {
    use om9_addon_geom::subd::selection::Component::{self, Edge, Face, Vertex};
    use om9_app::{f6::catalog::Mode, session::InvocationOrigin};
    use om9_scene::scene::Subelement;
    use std::collections::BTreeSet;
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("T-Spline neighborhood")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    app.controller.submit("XFaces=6").unwrap();
    app.controller.submit("YFaces=6").unwrap();
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([24., 24., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    let original = app.controller.document().clone();
    let history = app.controller.coordinator().undo_len();
    let pick = |app: &mut MatrixApp, keys: &BTreeSet<Component>| {
        app.controller.clear_subselection();
        for &key in keys {
            let part = match key {
                Vertex(v) => Subelement::MeshVertex(v),
                Face(f) => Subelement::MeshFace(f as u32),
                Edge((a, b)) => Subelement::MeshEdge { a, b },
            };
            app.controller.select_subobject(id, part).unwrap();
        }
    };
    let keys = |app: &MatrixApp| {
        app.controller
            .subselection()
            .iter()
            .map(|p| {
                assert_eq!(p.root, id);
                match p.subset {
                    Subelement::MeshVertex(v) => Vertex(v),
                    Subelement::MeshFace(f) => Face(f as usize),
                    Subelement::MeshEdge { a, b } => Edge((a, b)),
                    _ => unreachable!(),
                }
            })
            .collect::<BTreeSet<_>>()
    };
    let run = |app: &mut MatrixApp, label: &str| {
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|r| r.label == label)
            .unwrap();
        assert!(row.availability.enabled(), "{label}");
        app.controller
            .dispatch(&row.id, InvocationOrigin::F6)
            .unwrap();
    };
    let v = |x: u32, y: u32| x * 7 + y;
    let edge = |x, y, xx, yy| Edge(om9_addon_geom::subd::edge(v(x, y), v(xx, yy)));
    let faces = |lo: usize, hi: usize| {
        (lo..=hi)
            .flat_map(|i| (lo..=hi).map(move |j| Face(i * 6 + j)))
            .collect::<BTreeSet<_>>()
    };
    let verts = |lo: u32, hi: u32| {
        (lo..=hi)
            .flat_map(|i| (lo..=hi).map(move |j| Vertex(v(i, j))))
            .collect::<BTreeSet<_>>()
    };
    for (seeds, expected) in [
        (
            BTreeSet::from([edge(2, 2, 2, 3)]),
            (0..6).map(|j| edge(2, j, 2, j + 1)).collect(),
        ),
        (
            BTreeSet::from([Vertex(v(2, 2)), Vertex(v(3, 2))]),
            (0..7).map(|i| Vertex(v(i, 2))).collect(),
        ),
        (
            BTreeSet::from([Face(2 * 6 + 2), Face(3 * 6 + 2)]),
            (0..6).map(|i| Face(i * 6 + 2)).collect(),
        ),
    ] {
        app.f6_mode = Mode::General;
        pick(&mut app, &seeds);
        run(&mut app, "Loop Select");
        assert_eq!(keys(&app), expected);
        pick(&mut app, &seeds);
        app.controller.submit("-_tsSelection _SelectLoop").unwrap();
        assert_eq!(keys(&app), expected);
    }
    app.f6_mode = Mode::TSplines;
    pick(&mut app, &faces(2, 3));
    run(&mut app, "Grow Selection");
    assert_eq!(keys(&app), faces(1, 4));
    app.controller
        .submit("-_tsSelection _ShrinkSelection")
        .unwrap();
    assert_eq!(keys(&app), faces(2, 3));
    pick(&mut app, &verts(1, 4));
    run(&mut app, "Shrink Selection");
    assert_eq!(keys(&app), verts(2, 3));
    let square = BTreeSet::from([
        edge(2, 2, 3, 2),
        edge(2, 3, 3, 3),
        edge(2, 2, 2, 3),
        edge(3, 2, 3, 3),
    ]);
    pick(&mut app, &square);
    run(&mut app, "Grow Selection");
    let mut expanded = square.clone();
    for x in [2, 3] {
        expanded.extend([edge(x, 1, x, 2), edge(x, 3, x, 4)]);
    }
    for y in [2, 3] {
        expanded.extend([edge(1, y, 2, y), edge(3, y, 4, y)]);
    }
    assert_eq!(keys(&app), expanded);
    run(&mut app, "Shrink Selection");
    assert_eq!(keys(&app), square);
    for (label, macro_text, dx, dy) in [
        ("Select Next U", "-_tsSelection _NextU", 1isize, 0isize),
        ("Select Previous U", "-_tsSelection _PrevU", -1, 0),
        ("Select Next V", "_-tsSelection _NextV", 0, 1),
        ("Select Previous V", "_-tsSelection _PrevV", 0, -1),
    ] {
        let (x, y) = ((2 + dx) as u32, (2 + dy) as u32);
        for (seed, next) in [
            (Face(14), Face((x * 6 + y) as usize)),
            (edge(2, 2, 3, 2), edge(x, y, x + 1, y)),
            (edge(2, 2, 2, 3), edge(x, y, x, y + 1)),
        ] {
            let seed = BTreeSet::from([seed]);
            pick(&mut app, &seed);
            run(&mut app, label);
            assert_eq!(keys(&app), BTreeSet::from([next]));
            pick(&mut app, &seed);
            app.controller.submit(macro_text).unwrap();
            assert_eq!(keys(&app), BTreeSet::from([next]));
        }
    }
    pick(&mut app, &BTreeSet::from([Face(0)]));
    run(&mut app, "Select Previous U");
    assert_eq!(keys(&app), BTreeSet::from([Face(0)]));
    let separate = BTreeSet::from([Face(0), Face(35)]);
    pick(&mut app, &separate);
    assert!(app.controller.submit("-_tsSelection _SelectLoop").is_err());
    assert_eq!(keys(&app), separate);
    assert_eq!(app.controller.document(), &original);
    assert_eq!(app.controller.coordinator().undo_len(), history);
}

#[test]
fn set_pivot_places_the_component_manipulator_in_document_units_and_preserves_history() {
    use om9_app::session::InvocationOrigin;
    use om9_core::{
        geometry::Unit,
        gumball::{Axis, Drag, Handle, Session},
    };
    use om9_scene::scene::Subelement;
    let mut doc = Document::with_template_tolerance("pivot in inches");
    doc.page_unit = Unit::Inch;
    let mut app = MatrixApp::new(doc).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let c = &mut app.controller;
    c.submit("tsPlane").unwrap();
    c.session_point([0.; 3]).unwrap();
    c.submit("2,2,0").unwrap();
    let id = *c.selected().first().unwrap();
    let original = c.document().clone();
    c.select_subobject(id, Subelement::MeshFace(0)).unwrap();
    let action = app
        .f6_context()
        .actions
        .into_iter()
        .find(|a| a.label == "Set Pivot")
        .unwrap();
    let c = &mut app.controller;
    c.dispatch(&action.id, InvocationOrigin::F6).unwrap();
    assert!(c.session_wants_point());
    c.session_point([0.5, 0., 0.]).unwrap();
    assert_eq!(c.gumball_pivot().unwrap().origin, [0.5, 0., 0.]);
    assert_eq!(c.subselection().len(), 1);
    c.submit("-_tsManip _Enabled=_On _SetPivot").unwrap();
    c.cancel();
    assert_eq!(c.gumball_origin(), Some([0.5, 0., 0.]));
    assert_eq!(c.coordinator().undo_len(), 1);
    c.submit("-_tsManip _Enabled=_On _SetPivot").unwrap();
    c.submit("1,0.5,0").unwrap();
    assert_eq!(c.gumball_origin(), Some([1.,0.5,0.]));
    c.submit("-_tsManip _Enabled=_On _SetPivot").unwrap();
    c.submit("0,0,0").unwrap();
    c.submit("-_tsManip _Enabled=_On _Manipulator=_Rotate _SelectionMode=_Face _Enter")
        .unwrap();
    let mut gesture =
        Session::begin(c.gumball_pivot().unwrap(), c.gumball_targets(), false).unwrap();
    gesture
        .drag(
            Handle::Rotate(Axis::Z),
            Drag::Angle(std::f64::consts::FRAC_PI_2),
        )
        .unwrap();
    c.commit_gumball(&gesture, "Rotate").unwrap();
    let om9_core::geometry::Geometry::Mesh(m) = &c.document().entities[&id].geometry else {
        panic!("mesh")
    };
    assert!((m.vertices[3][0]).abs() < 1e-9 && (m.vertices[3][1] - 1.).abs() < 1e-9);
    c.undo().unwrap();
    assert_eq!(c.document(), &original);
    c.select_objects(&[id]).unwrap();
    c.submit("-_tsManip _Enabled=_On _SetPivot").unwrap();
    c.clear_selection();
    assert!(c.session_point([12., 0., 0.]).is_err());
    c.cancel();
    assert_ne!(c.gumball_origin(), Some([12., 0., 0.]));
}

#[test]
fn extract_edge_curves_use_one_limit_span_and_offset_on_both_sides() {
    use om9_app::session::InvocationOrigin;
    use om9_core::{
        budget::Limits,
        curve_ops::point_at,
        geometry::{Geometry, Transform},
        gumball,
    };
    use om9_scene::scene::Subelement::MeshEdge;
    let mut app = MatrixApp::new(Document::with_template_tolerance("Extract Edge Curves")).unwrap();
    for command in ["tsPlane", "XFaces=6", "YFaces=6", "0,0,0", "6,6,0"] {
        app.controller.submit(command).unwrap();
    }
    let id = *app.controller.selected().first().unwrap();
    let mut doc = app.controller.document().clone();
    let entity = doc.entities.get_mut(&id).unwrap();
    let placement = Transform::translation([31., -7., 2.])
        .compose(gumball::rotation_about([0.; 3], [0., 0., 1.], 0.71).unwrap())
        .unwrap();
    entity.placement = placement;
    let Geometry::Mesh(m) = &mut entity.geometry else {
        panic!("cage")
    };
    for p in &mut m.vertices {
        p[2] = p[0] * p[0] + 2. * p[1] * p[1];
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let pick = |app: &mut MatrixApp| {
        app.controller
            .select_subobject(id, MeshEdge { a: 17, b: 24 })
            .unwrap();
    };
    let check = |app: &MatrixApp, ys: &[f64]| {
        let mut curves: Vec<_> = app
            .controller
            .document()
            .entities
            .values()
            .filter_map(|e| {
                if let Geometry::Curve(c) = &e.geometry {
                    assert_eq!(e.placement, placement);
                    Some(c)
                } else {
                    None
                }
            })
            .collect();
        let limits = Limits::default();
        curves.sort_by(|a, b| {
            point_at(a, 0., &limits).unwrap()[1].total_cmp(&point_at(b, 0., &limits).unwrap()[1])
        });
        assert_eq!(curves.len(), ys.len());
        for (curve, &y) in curves.iter().zip(ys) {
            for t in [0., 0.25, 0.5, 0.75, 1.] {
                let p = point_at(curve, t, &limits).unwrap();
                let x = 2. + t;
                // Uniform cubic basis reproduces the quadratic with second moment 1/3 in each axis.
                for (got, want) in p.into_iter().zip([x, y, x * x + 2. * y * y + 1.]) {
                    assert!((got - want).abs() < 1e-8, "{p:?} at {t} expected y={y}");
                }
            }
        }
        assert_eq!(app.controller.document().entities[&id], doc.entities[&id]);
    };
    for cmd in [false, true] {
        pick(&mut app);
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|a| a.label == "Extract Edges")
            .unwrap();
        assert!(row.availability.enabled());
        if cmd {
            app.controller
                .submit("-_tsExtractEdges _Enter _selnone _selLast")
                .unwrap();
        } else {
            app.controller
                .dispatch(&row.id, InvocationOrigin::F6)
                .unwrap();
        }
        check(&app, &[3.]);
        assert_eq!(app.controller.selected().len(), 1);
        assert!(!app.controller.selected().contains(&id));
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &doc);
    }
    for edge in [MeshEdge { a: 17, b: 24 }, MeshEdge { a: 18, b: 25 }] {
        app.controller.select_subobject(id, edge).unwrap();
    }
    app.controller
        .submit("-_tsExtractEdges _Enter _selnone _selLast")
        .unwrap();
    check(&app, &[3., 4.]);
    assert_eq!(app.controller.selected().len(), 2);
    assert!(!app.controller.selected().contains(&id));
    app.controller.undo().unwrap();
    for mouse in [false, true] {
        pick(&mut app);
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|a| a.label == "Extract Edges Offset")
            .unwrap();
        if mouse {
            app.controller
                .dispatch(&row.id, InvocationOrigin::F6)
                .unwrap();
        } else {
            app.controller.submit("-_tsExtractEdges _Offset").unwrap();
        }
        app.controller.submit("BothSides=Yes").unwrap();
        assert!(app.controller.submit("1.1").is_err());
        if mouse {
            // Halfway up the negative-side perpendicular control edge, in world coordinates.
            let at = placement.point([2., 2.5, 17.]).unwrap();
            app.controller.session_hover(at);
            assert!(
                !app.controller
                    .addon_session()
                    .unwrap()
                    .hover_lines
                    .is_empty(),
                "hover on projected perpendicular edge"
            );
            app.controller.session_point(at).unwrap();
        } else {
            app.controller.submit("-0.5").unwrap();
        }
        assert_eq!(app.controller.document(), &doc);
        app.controller.submit("").unwrap();
        check(&app, &[2.5, 3.5]);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
    }
    pick(&mut app);
    app.controller.submit("-_tsExtractEdges _Offset").unwrap();
    app.controller.submit("0.5").unwrap();
    app.controller.cancel();
    assert_eq!(app.controller.document(), &doc);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.clear_subselection();
    app.controller
        .select_subobject(id, MeshEdge { a: 0, b: 7 })
        .unwrap();
    app.controller.submit("-_tsExtractEdges _Offset").unwrap();
    app.controller.submit("BothSides=Yes").unwrap();
    assert!(app.controller.submit("0.5").is_err());
    assert_eq!(app.controller.document(), &doc);
    app.controller.clear_subselection();
    assert!(app.controller.submit("0").is_err());
    app.controller.cancel();
}

#[test]
fn extracted_open_border_and_corner_agree_with_independent_catmull_clark_refinement() {
    use om9_addon_geom::subd;
    use om9_core::{
        budget::Limits,
        cage_isocurve::CagePatches,
        curve_ops::point_at,
        geometry::{Curve, Mesh},
    };
    use std::collections::BTreeSet;
    let rows = (0..5)
        .map(|x| {
            (0..5)
                .map(|y| [x as f64, y as f64, ((2 * x + 3 * y) as f64).sin()])
                .collect()
        })
        .collect::<Vec<_>>();
    let coarse = subd::grid(&rows, false, false);
    let fine = subd::catmull_clark(&coarse, &BTreeSet::new());
    let mesh = |p: &subd::Poly| Mesh {
        vertices: p.verts.clone(),
        faces: p.faces.clone(),
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let a = mesh(&coarse);
    let b = mesh(&fine);
    let l = Limits::default();
    let coarse_patches = CagePatches::new(&a, &l).unwrap();
    let fine_patches = CagePatches::new(&b, &l).unwrap();
    let edges = coarse.edge_faces();
    for seed in [(0, 1), (1, 2), (0, 5), (5, 10), (6, 11), (12, 13)] {
        let mid = coarse.verts.len() as u32 + edges.keys().position(|e| *e == seed).unwrap() as u32;
        let span = coarse_patches.edge_span([seed.0, seed.1]).unwrap();
        let edge = Curve::Nurbs(span.curve(0., &l).unwrap());
        for (first, vertex) in [(true, seed.0), (false, seed.1)] {
            let half = Curve::Nurbs(
                fine_patches
                    .edge_span([vertex, mid])
                    .unwrap()
                    .curve(0., &l)
                    .unwrap(),
            );
            for s in [0., 0.25, 0.5, 1.] {
                let t = if first { s / 2. } else { 1. - s / 2. };
                let lhs = point_at(&edge, t, &l).unwrap();
                let rhs = point_at(&half, s, &l).unwrap();
                assert!(
                    (0..3).all(|i| (lhs[i] - rhs[i]).abs() < 1e-9),
                    "edge={seed:?} at {t}: {lhs:?} vs {rhs:?}"
                );
            }
        }
        if edges[&seed].len() == 1 {
            assert!(span.curve(-0.25, &l).is_err());
            assert!(span.curve(1., &l).is_ok());
        }
    }
    let mut broken = a.clone();
    broken.faces.push(vec![6, 3, 24]);
    assert!(
        CagePatches::new(&broken, &l)
            .unwrap()
            .edge_span([6, 11])
            .is_err()
    );
}

#[test]
fn split_curve_network_uses_document_or_custom_tolerance_and_one_undo() {
    use om9_app::{
        f6::catalog::Mode,
        session::{Controller, InvocationOrigin},
    };
    use om9_core::{
        geometry::{Curve, Geometry, Transform, Unit},
        history::Tolerance,
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("inch curve network");
    doc.page_unit = Unit::Inch;
    doc.tolerance = Some(Tolerance::try_new(0.0254, 0.01, 0.01).unwrap());
    let mut ids = vec![];
    for (name, a, b, offset) in [
        ("rail", [-4., 0., 0.], [4., 0., 0.], [0.; 3]),
        ("left", [0., -3., 0.], [0., 3., 0.], [-1., 0., 0.003]),
        ("right", [0., -3., 0.], [0., 3., 0.], [2., 0., 0.003]),
    ] {
        let mut e = Entity::new(
            name,
            Geometry::Curve(Curve::Line {
                start: a,
                end: b,
                domain: [0., 1.],
            }),
            doc.current_layer,
        );
        e.placement = Transform::translation(offset);
        e.attributes.color = Some([10, 20, 30, 255]);
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let wait = |c: &mut Controller| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while c.split_curves_pending() {
            if let Some(r) = c.poll_split_curves() {
                r.unwrap();
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    };
    {
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.f6_mode = Mode::TSplines;
        app.controller.select_objects(&ids).unwrap();
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|r| r.label == "ts Split Curves")
            .unwrap();
        assert!(row.availability.enabled());
        let c = &mut app.controller;
        c.dispatch(&row.id, InvocationOrigin::F6).unwrap();
        let dialog = c.addon_dialog().unwrap();
        assert_eq!(dialog.title, "tsSplitCurves");
        assert!(!dialog.enabled("Tolerance"));
        assert_eq!(
            dialog
                .rows
                .iter()
                .find(|r| r.name == "Tolerance")
                .unwrap()
                .value
                .parse::<f64>()
                .unwrap(),
            0.001
        );
        c.submit("Display").unwrap();
        wait(c);
        assert!(c.addon_session().unwrap().lines.is_empty());
        assert_eq!(c.document(), &doc);
        c.submit("").unwrap();
        assert_eq!(c.coordinator().undo_len(), 0);
        c.select_objects(&[]).unwrap();
        c.submit("_tsSplitCurves").unwrap();
        assert!(c.addon_collects_objects());
        c.select_objects(&ids).unwrap();
        c.submit("").unwrap();
        c.submit("UseFileTolerance=No").unwrap();
        assert!(c.submit("Tolerance=0").is_err());
        c.submit("Tolerance=0.005").unwrap();
        c.submit("Display").unwrap();
        wait(c);
        assert_eq!(c.addon_session().unwrap().reply.preview.len(), 4);
        assert_eq!(c.document(), &doc);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 7);
        assert_eq!(c.selected().len(), 7);
        assert_eq!(c.coordinator().undo_len(), 1);
        let mut spans = vec![];
        for e in c.document().entities.values() {
            let source = doc.entities.values().find(|s| s.label == e.label).unwrap();
            assert_eq!(e.attributes, source.attributes);
            assert_eq!(e.placement, source.placement);
            let Geometry::Curve(Curve::Line { start, end, .. }) = &e.geometry else {
                panic!("line piece")
            };
            let axis = usize::from(e.label != "rail");
            spans.push((e.label.clone(), start[axis] as i32, end[axis] as i32));
            assert_eq!(start[2], 0.);
            assert_eq!(end[2], 0.);
        }
        spans.sort();
        assert_eq!(
            spans,
            [
                ("left", -3, 0),
                ("left", 0, 3),
                ("rail", -4, -1),
                ("rail", -1, 2),
                ("rail", 2, 4),
                ("right", -3, 0),
                ("right", 0, 3)
            ]
            .map(|(s, a, b)| (s.to_string(), a, b))
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &doc);
        c.select_objects(&ids).unwrap();
        c.dispatch("OM9-TSPLINE-069", InvocationOrigin::Menu)
            .unwrap();
        c.submit("Display").unwrap();
        c.cancel();
        assert!(!c.split_curves_pending());
        assert_eq!(c.document(), &doc);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("_tsSplitCurves").unwrap();
        c.submit("UseFileTolerance=No").unwrap();
        c.submit("Tolerance=0.005").unwrap();
        c.submit("Display").unwrap();
        wait(c);
        c.set_current_layer(om9_app::layer_command::LayerChoice::Slot(
            om9_core::layers::PresetSlot::new(21).unwrap(),
        ))
        .unwrap();
        c.refresh_addon_session();
        assert!(c.addon_session().is_none());
        assert_eq!(c.document().entities, doc.entities);
    }
}

#[test]
fn split_curve_network_deduplicates_crossings_and_reconciles_affine_arc_parameters() {
    use om9_core::{
        budget::Limits,
        curve_ops::point_at,
        geometry::{Curve, Geometry, Transform},
        model::Entity,
        top11::split_network,
    };
    let line = |s, e| {
        Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: s,
                end: e,
                domain: [0., 1.],
            }),
            None,
        )
    };
    let entities = [
        line([-1., 0., 0.], [1., 0., 0.]),
        line([0., -1., 0.], [0., 1., 0.]),
        line([-1., -1., 0.], [1., 1., 0.]),
        line([1., 0., 0.], [2., 0., 0.]),
    ];
    let l = Limits::default();
    let split = split_network::split(&entities, 1e-6, &l, &|| false).unwrap();
    assert_eq!(split.len(), 3);
    assert!(
        split
            .iter()
            .all(|m| m.points == vec![[0.; 3]] && m.pieces.len() == 2)
    );
    assert!(split_network::split(&entities, 1e-6, &l, &|| true).is_err());
    // Manual p91: a closed curve crossed by two chords which also cross each other.
    let circle = Entity::new(
        "circle",
        Geometry::Curve(Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 1.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let a = line([0.6, 0.8, 0.], [-0.6, -0.8, 0.]);
    let b = line([-0.8, 0.6, 0.], [0.8, -0.6, 0.]);
    let network = split_network::split([&circle, &a, &b], 1e-6, &l, &|| false).unwrap();
    assert_eq!(network.iter().map(|m| m.pieces.len()).sum::<usize>(), 8);
    assert_eq!(
        network
            .iter()
            .find(|m| m.source == circle.id)
            .unwrap()
            .pieces
            .len(),
        4
    );
    assert!(
        split_network::split_admitted([&circle, &a], 1e-6, &l, &|| false, &mut |_| Err(
            om9_core::Error::Budget("test")
        ))
        .is_err()
    );
    let mut ellipse = Entity::new(
        "ellipse",
        Geometry::Curve(Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 1.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    ellipse.placement = Transform([
        [2., 0., 0., 3.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let cutter = line([3.8, -2., 0.], [3.8, 2., 0.]);
    let split = split_network::split([&ellipse, &cutter], 1e-6, &l, &|| false).unwrap();
    let result = split.iter().find(|s| s.source == ellipse.id).unwrap();
    assert_eq!(result.pieces.len(), 2);
    assert_eq!(result.points.len(), 2);
    for curve in &result.pieces {
        for t in [0., 1.] {
            let p = ellipse
                .placement
                .point(point_at(curve, t, &l).unwrap())
                .unwrap();
            assert!(
                (p[0] - 3.8).abs() < 1e-6 && (p[1].abs() - 0.84_f64.sqrt()).abs() < 1e-6,
                "{p:?}"
            );
        }
    }
}

// The normal archive decoder hands From Lines trimmed/placed model curves, before any UI state.
#[test]
fn from_lines_prepares_archived_curves_at_world_tolerance_and_preserves_sources() {
    use om9_app::session::from_lines::prepare;
    use om9_core::{
        Error,
        curve_build::polyline,
        geometry::{Geometry, Transform},
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("From Lines source");
    let mut ids = vec![];
    for points in [
        vec![[0., 0., 0.], [8., 0., 0.], [8., 8., 0.]],
        vec![[8.002, 8., 0.], [0., 8., 0.], [0., 0., 0.]],
        vec![
            [2., 2., 0.],
            [4., 2., 0.],
            [4., 4., 0.],
            [2., 4., 0.],
            [2., 2., 0.],
        ],
    ] {
        let mut e = Entity::new(
            "input",
            Geometry::Curve(polyline(&points).unwrap()),
            doc.current_layer,
        );
        e.placement = Transform::translation([11., 20., 3.]);
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let limits = Default::default();
    let bytes = om9_archive::Archive::encode(&doc, &limits).unwrap();
    let source = om9_archive::Archive::decode(&bytes, &limits).unwrap();
    ids.push(ids[0]);
    let tight = prepare(&source, &ids, 0.001, 4, &limits, &mut |_| Ok(())).unwrap();
    assert_eq!(tight.unused_edges.len(), 4);
    let layout = prepare(&source, &ids, 0.003, 4, &limits, &mut |_| Ok(())).unwrap();
    assert_eq!(layout.faces().len(), 1);
    assert_eq!(layout.faces()[0].holes.len(), 1);
    assert!(layout.unused_edges.is_empty());
    assert_eq!(layout.pick_face([14., 23., 8.], [0., 0., -1.]), None);
    assert_eq!(layout.pick_face([12., 21., 8.], [0., 0., -1.]), Some(0));
    let display = layout
        .cage(&mut |_| Ok(()))
        .unwrap()
        .display(0, &mut |_| Ok(()))
        .unwrap();
    assert!(display.face_ids().iter().all(|&id| id == 0));
    let fail = prepare(&source, &ids, 0.003, 4, &limits, &mut |_| {
        Err(Error::Canceled)
    });
    assert!(matches!(fail, Err(Error::Canceled)));
    assert_eq!(source, doc);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    let wait = |c: &mut om9_app::session::Controller| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while c.from_lines_pending() {
            if let Some(result) = c.poll_from_lines() {
                result.unwrap();
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    };
    c.submit_effect("tsFromLines").unwrap();
    assert_eq!(
        c.addon_session().unwrap().reply.want,
        om9_addon_abi::session::Want::Objects
    );
    c.select_objects(&ids[..3]).unwrap();
    c.submit_effect("").unwrap();
    wait(c);
    c.submit_effect("UseFileTolerance=No").unwrap();
    c.submit_effect("Tolerance=0.003").unwrap();
    wait(c);
    assert_eq!(c.from_lines_layout().unwrap().faces()[0].holes.len(), 1);
    c.submit_effect("Smooth=No").unwrap();
    c.submit_effect("").unwrap();
    wait(c);
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 4);
    let id = c.selected_ordered()[0];
    let bytes = om9_archive::Archive::encode(c.document(), &limits).unwrap();
    let reopened = om9_archive::Archive::decode(&bytes, &limits).unwrap();
    let cage = om9_app::session::from_lines::CageRecord::read(&reopened.entities[&id]).unwrap();
    assert!(!cage.smooth);
    assert!(cage.display_face_ids.iter().all(|&f| f == 0));
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
    c.select_objects(&ids[..3]).unwrap();
    c.submit_effect("tsFromLines").unwrap();
    c.cancel();
    assert!(!c.from_lines_pending());
    assert_eq!(c.document(), &doc);
    c.submit_effect("tsFromLines").unwrap();
    wait(c);
    c.submit_effect("").unwrap();
    c.set_current_layer(om9_app::layer_command::LayerChoice::Slot(
        om9_core::layers::PresetSlot::new(21).unwrap(),
    ))
    .unwrap();
    c.refresh_addon_session();
    assert!(c.addon_session().is_none());
    assert!(c.poll_from_lines().is_none());
    assert_eq!(c.document().entities, doc.entities);
}

#[test]
fn from_lines_large_smooth_cage_round_trips_and_edits_each_logical_face_once() {
    use om9_app::{
        f6::catalog::Mode,
        session::{InvocationOrigin, from_lines::CageRecord},
    };
    use om9_core::{
        geometry::{Curve, Geometry},
        model::{Entity, TSPLINE_FROM_LINES_SLOT},
    };
    use om9_scene::scene::Subelement;
    let mut doc = Document::with_template_tolerance("From Lines archive");
    let mut sources = vec![];
    for x in 0..=20 {
        for y in 0..20 {
            for (start, end) in [
                ([x as f64, y as f64, 0.], [x as f64, (y + 1) as f64, 0.]),
                ([y as f64, x as f64, 0.], [(y + 1) as f64, x as f64, 0.]),
            ] {
                let e = Entity::new(
                    "source",
                    Geometry::Curve(Curve::Line {
                        start,
                        end,
                        domain: [0., 1.],
                    }),
                    doc.current_layer,
                );
                sources.push(e.id);
                doc.entities.insert(e.id, e);
            }
        }
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let c = &mut app.controller;
    c.select_objects(&sources).unwrap();
    c.dispatch("OM9-TSPLINE-072", InvocationOrigin::Menu)
        .unwrap();
    let wait = |c: &mut om9_app::session::Controller| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while c.from_lines_pending() {
            if let Some(result) = c.poll_from_lines() {
                result.unwrap();
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
    };
    wait(c);
    assert_eq!(c.from_lines_layout().unwrap().faces().len(), 400);
    c.submit_effect("").unwrap();
    wait(c);
    let id = c.selected_ordered()[0];
    let before = c.document().clone();
    assert!(
        before.entities[&id]
            .extension(TSPLINE_FROM_LINES_SLOT)
            .unwrap()
            .payload
            .len()
            > 16 * 1024
    );
    let limits = Default::default();
    let archive = om9_archive::Archive::encode(&before, &limits).unwrap();
    let restored = om9_archive::Archive::decode(&archive, &limits).unwrap();
    assert_eq!(before, restored);
    let mut damaged=restored.clone();
    let mut bad=CageRecord::read(&damaged.entities[&id]).unwrap();
    bad.display_face_ids[0]=usize::MAX;
    damaged.entities.get_mut(&id).unwrap().extensions.get_mut(TSPLINE_FROM_LINES_SLOT).unwrap().payload=serde_json::to_string(&bad).unwrap();
    assert!(om9_archive::Archive::encode(&damaged,&limits).is_err());
    drop(app);
    let mut app = MatrixApp::new(restored).unwrap();
    app.f6_mode = Mode::TSplines;
    for (label, expected) in [("Duplicate Face(s)", 1), ("Delete Face(s)", 399)] {
        app.controller.select_objects(&[id]).unwrap();
        app.controller
            .select_subobject(id, Subelement::MeshFace(2))
            .unwrap();
        assert_eq!(app.controller.subselection().len(), 4);
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|a| a.label == label)
            .unwrap();
        assert!(row.availability.enabled());
        app.controller
            .dispatch(&row.id, InvocationOrigin::F6)
            .unwrap();
        let output = if expected == 1 {
            app.controller.selected_ordered()[0]
        } else {
            id
        };
        let record = CageRecord::read(&app.controller.document().entities[&output]).unwrap();
        assert_eq!(record.polygons.len(), expected);
        assert_eq!(record.display_face_ids.len(), 4 * expected);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn extrude_lines_placed_control_polygon_feeds_from_lines_without_duplicate_wires() {
    use om9_addon_geom::subd::from_lines::LineNetwork;
    use om9_core::{
        Error,
        budget::Limits,
        curve_build::polyline,
        curve_ops::extrude_lines::ControlProfiles,
        geometry::{Curve, Transform},
    };

    let points = [
        [0., 0., 0.],
        [2., 0., 0.],
        [2., 3., 0.],
        [0., 3., 0.],
        [0., 0., 0.],
    ];
    let profile = polyline(&points).unwrap();
    let world = Transform::translation([10., -2., 7.]);
    let supplied = points.map(|p| world.point(p).unwrap());
    let a = supplied[0];
    let b = supplied[1];
    let backwards = Curve::Line {
        start: b,
        end: a,
        domain: [0., 1.],
    };
    let limits = Limits::default();
    let dir = [0., -1., -3.];
    let polygon = ControlProfiles::capture(
        [(&profile, world), (&backwards, Transform::identity())],
        0.001,
        &limits,
        &mut || Ok(()),
    )
    .unwrap();
    assert_eq!(polygon.start, a);
    for (both, keep, created, total_edges, face_count) in [
        (false, true, 8, 12, 6),
        (false, false, 12, 12, 6),
        (true, true, 16, 20, 10),
        (true, false, 12, 12, 6),
    ] {
        let segments = polygon
            .wires(dir, both, keep, &limits, &mut || Ok(()))
            .unwrap();
        assert_eq!(segments.len(), created);
        let mut paths: Vec<_> = segments.into_iter().map(|line| line.to_vec()).collect();
        if keep {
            paths.extend(supplied.windows(2).map(|edge| edge.to_vec()));
        }
        let graph = LineNetwork::from_paths(&paths, 0.001, &mut |_| Ok(())).unwrap();
        assert_eq!(graph.edges.len(), total_edges);
        let faces = graph.automatic_faces(4, &mut |_| Ok(())).unwrap();
        assert_eq!(faces.len(), face_count);
        let cap_heights: Vec<_> = faces
            .iter()
            .filter_map(|face| {
                let z = graph.vertices[face.outer[0] as usize][2];
                face.outer
                    .iter()
                    .all(|id| graph.vertices[*id as usize][2] == z)
                    .then_some(z)
            })
            .collect();
        assert_eq!(cap_heights.len(), 2);
        assert!(cap_heights.contains(&4.));
        assert!(cap_heights.contains(&if both { 10. } else { 7. }));
        assert!(graph.vertices.contains(&[10., -3., 4.]));
        assert_eq!(graph.vertices.contains(&[10., -1., 10.]), both);
    }
    let run = |offset, limits: &Limits| polygon.wires(offset, false, true, limits, &mut || Ok(()));
    assert!(matches!(run([0.; 3], &limits), Err(Error::Invalid(_))));
    assert!(matches!(
        run(
            dir,
            &Limits {
                max_outputs: 7,
                ..limits
            }
        ),
        Err(Error::Budget(_))
    ));
    assert!(matches!(
        polygon.wires(dir, false, true, &limits, &mut || Err(Error::Canceled)),
        Err(Error::Canceled)
    ));
}

#[test]
fn extrude_lines_cmd_postselection_direction_and_base_in_inches() {
    use om9_app::session::InvocationOrigin;
    use om9_core::{
        geometry::{Curve, Geometry, Transform, Unit},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Extrude Lines inch");
    doc.page_unit = Unit::Inch;
    let mut e = Entity::new(
        "profile",
        Geometry::Curve(Curve::Line {
            start: [0.; 3],
            end: [2., 0., 0.],
            domain: [0., 1.],
        }),
        doc.current_layer,
    );
    e.placement = Transform::translation([3., 2., 1.]);
    let id = e.id;
    doc.entities.insert(id, e);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    c.workspace.active = ViewId::LookingDown;
    c.submit_effect("_tsExtrudeLines").unwrap();
    assert!(c.addon_collects_objects());
    c.select_objects(&[id]).unwrap();
    c.submit_effect("").unwrap();
    c.submit_effect("SetBasePoint").unwrap();
    c.submit_effect("w1,2,20").unwrap();
    c.session_hover([1., 2., 18.]);
    let preview = c.addon_session().unwrap().hover_lines.clone();
    assert_eq!(preview.len(), 3);
    assert!(preview.iter().flatten().any(|p| (p[2] + 1.).abs() < 1e-9));
    assert_eq!(c.document(), &doc);
    assert!(c.submit_effect("0").is_err());
    assert!(c.addon_session().is_some());
    c.submit_effect("-50.8 mm").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    for [a, b] in preview {
        assert!(c.document().entities.values().any(|e|matches!(&e.geometry,Geometry::Curve(Curve::Line{start,end,..}) if (0..3).all(|i|(start[i]-a[i]).abs()<1e-9 && (end[i]-b[i]).abs()<1e-9))));
    }
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
    c.select_objects(&[id]).unwrap();
    c.dispatch("OM9-TSPLINE-073", InvocationOrigin::Menu)
        .unwrap();
    for t in [
        "BothSides=Yes",
        "DeleteInput=Yes",
        "Direction",
        "w1,1,1",
        "w1,4,5",
        "5",
    ] {
        c.submit_effect(t).unwrap();
    }
    assert_eq!(c.document().entities.len(), 4);
    assert!(!c.document().entities.contains_key(&id));
    let points: Vec<_> = c
        .document()
        .entities
        .values()
        .flat_map(|e| match &e.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => [*start, *end],
            _ => panic!("expected straight control wires"),
        })
        .collect();
    for p in [[3., -1., -3.], [5., -1., -3.], [3., 5., 5.], [5., 5., 5.]] {
        assert!(
            points
                .iter()
                .any(|q| (0..3).all(|i| (p[i] - q[i]).abs() < 1e-9))
        );
    }
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
    c.select_objects(&[id]).unwrap();
    c.submit_effect("tsExtrudeLines").unwrap();
    let repeated = &c.addon_session().unwrap().reply;
    assert!(repeated.prompt.contains("<5>"), "{}", repeated.prompt);
    assert!(
        repeated
            .options
            .iter()
            .any(|opt| opt.name == "DeleteInput" && opt.value.as_deref() == Some("Yes"))
    );
    c.submit_effect("Direction").unwrap();
    c.session_point([0.; 3]).unwrap();
    assert!(c.session_point([0.; 3]).is_err());
    c.cancel();
    assert_eq!(c.document(), &doc);
    c.select_objects(&[id]).unwrap();
    c.submit_effect("tsExtrudeLines").unwrap();
    c.set_current_layer(om9_app::layer_command::LayerChoice::Slot(
        om9_core::layers::PresetSlot::new(21).unwrap(),
    ))
    .unwrap();
    c.refresh_addon_session();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities, doc.entities);
}

#[test]
fn extrude_lines_preserves_placed_circle_and_rational_nurbs_after_reload() {
    use om9_app::session::InvocationOrigin;
    use om9_core::{
        curve_build::circle_on_frame,
        geometry::{Curve, Geometry, NurbsCurve, Transform},
        model::Entity,
    };
    let circle = circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let spline = Curve::Nurbs(NurbsCurve {
        degree: 2,
        poles: vec![[5., 0., 0.], [6., 2., 0.], [7., 0., 0.]],
        weights: vec![1., 0.4, 1.],
        knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0.125, 0.875],
        periodic: false,
        reversed: true,
        seam: None,
    });
    let placement = Transform::translation([7., 8., 9.])
        .compose(Transform::scale([2., 3., 1.]).unwrap())
        .unwrap();
    let mut doc = Document::with_template_tolerance("curved wire rails");
    let mut ids = Vec::new();
    for curve in [&circle, &spline] {
        let mut entity = Entity::new(
            "original",
            Geometry::Curve(curve.clone()),
            doc.current_layer,
        );
        entity.placement = placement;
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    c.select_objects(&ids).unwrap();
    c.dispatch("OM9-TSPLINE-073", InvocationOrigin::Menu)
        .unwrap();
    for t in ["BothSides=Yes", "DeleteInput=Yes", "Direction", "w1,1,1"] {
        c.submit_effect(t).unwrap();
    }
    c.submit_effect("w1,3,4").unwrap();
    assert_eq!(c.document(), &doc);
    let base = placement.point([2., 0., 0.]).unwrap();
    c.session_hover([base[0], base[1] + 2., base[2] + 3.]);
    assert!(c.addon_session().unwrap().hover_lines.len() > 200);
    c.submit_effect(&13f64.sqrt().to_string()).unwrap();
    let limits = Default::default();
    let bytes = om9_archive::Archive::encode(c.document(), &limits).unwrap();
    let restored = om9_archive::Archive::decode(&bytes, &limits).unwrap();
    assert_eq!(restored.entities.len(), 7);
    for curve in [&circle, &spline] {
        for offset in [[0., 2., 3.], [0., -2., -3.]] {
            let expected = Transform::translation(offset).compose(placement).unwrap();
            assert_eq!(
                restored
                    .entities
                    .values()
                    .filter(|e| e
                        .placement
                        .0
                        .iter()
                        .flatten()
                        .zip(expected.0.iter().flatten())
                        .all(|(a, b)| (a - b).abs() < 1e-10)
                        && e.geometry == Geometry::Curve(curve.clone()))
                    .count(),
                1
            );
        }
    }
    let lines: Vec<_> = restored
        .entities
        .values()
        .filter_map(|e| match &e.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some([*start, *end]),
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 3);
    let mut stations = om9_core::curve_ops::curve_endpoints(&spline, &limits)
        .unwrap()
        .to_vec();
    stations.push([2., 0., 0.]);
    for point in stations {
        let expected = placement.point(point).unwrap();
        assert!(lines.iter().any(|[p, q]| {
            (0..3).all(|i| {
                ((p[i] + q[i]) * 0.5 - expected[i]).abs() < 1e-9
                    && (q[i] - p[i] - [0., 4., 6.][i]).abs() < 1e-9
            })
        }));
    }
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_to_boundary_keeps_sources_on_cancel_and_builds_on_either_side() {
    use om9_core::{curve_build::{polyline,circle_on_frame},geometry::{Curve,Geometry,Transform},model::Entity,surface_kernel};
    use om9_scene::workspace::ViewId;
    let limits=Default::default();
    for height in [3.,-3.] {
        let mut doc=Document::with_template_tolerance("ToBoundary");
        let profile=polyline(&[[0.,0.,0.],[2.,0.,0.]]).unwrap();
        let mut e=Entity::new("line",Geometry::Curve(profile),doc.current_layer);
        let place=Transform::translation([3.,4.,1.]);
        e.placement=place;
        let line=e.id;
        doc.entities.insert(line,e);
        let circle=circle_on_frame([1.,1.,0.],[1.,0.,0.],[0.,1.,0.],0.4).unwrap();
        let mut e=Entity::new("circle",Geometry::Curve(circle),doc.current_layer); e.placement=place;
        let circle=e.id; doc.entities.insert(circle,e);
        let mut face=surface_kernel::plane([-2.,-2.,height],[1.,0.,0.],[0.,1.,0.],6.,6.,&limits).unwrap();
        for p in &mut face.poles {p[2]+=p[0]*0.5;}
        let mut e=Entity::new("stop",Geometry::Surface(face),doc.current_layer); e.placement=place;
        let stop=e.id;doc.entities.insert(stop,e);
        let mut app=MatrixApp::new(doc.clone()).unwrap();
        let c=&mut app.controller;c.workspace.active=ViewId::LookingDown;
        for retry in [false,true] {
            c.select_objects(&[line,circle]).unwrap();
            for text in ["tsExtrudeLines","Solid=Yes","BothSides=Yes","DeleteInput=Yes","ToBoundary"]{c.submit_effect(text).unwrap();}
            assert!(c.addon_collects_objects());
            assert!(c.selected().is_empty());
            assert_eq!(c.document(),&doc);
            if !retry {c.cancel();assert_eq!(c.selected_ordered(),[line,circle]);continue;}
            // Direction is available while the chooser is open and returns to that chooser.
            for text in ["Direction","w0,0,0","w0,0,1"] {c.submit_effect(text).unwrap();}
            assert!(c.addon_collects_objects());
            c.select_objects(&[line]).unwrap();c.refresh_addon_session();
            assert!(c.addon_collects_objects());assert!(c.selected().is_empty());
            c.select_objects(&[stop]).unwrap();c.refresh_addon_session();
            assert!(c.addon_session().is_none(),"boundary selection failed");
            assert_eq!(c.document().entities[&stop],doc.entities[&stop]);
            assert!(!c.document().entities.contains_key(&line));
            assert!(!c.document().entities.contains_key(&circle));
            let mut station_count=0; let mut cap_count=0;
            for id in c.selected_ordered(){
                let e=&c.document().entities[&id];
                match &e.geometry {
                    Geometry::Curve(Curve::Line{start,end,..}) if (start[0]-end[0]).abs()<1e-8=>{
                        let expected=height+(start[0]-3.)*0.5;
                        assert!((end[2]-start[2]-expected).abs()<1e-6);
                        station_count+=1;
                    }
                    Geometry::Brep(body)=>{
                        let vol=om9_kernel::brep::source_volume(body,&limits).unwrap();
                        let expected=std::f64::consts::PI*0.16*(height+0.5).abs();
                        assert!((vol-expected).abs()<1e-6,"{vol} vs {expected}");
                        cap_count+=1;
                    }
                    Geometry::Curve(_)=>{}
                    _=>panic!("unexpected boundary result"),
                }
            }
            assert_eq!((station_count,cap_count),(2,1));
            c.capture_brep_save_sources(Default::default()).unwrap();
            assert_eq!(c.coordinator().undo_len(),1);
            c.undo().unwrap();assert_eq!(c.document(),&doc);
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_solid_retries_a_mixed_plane_selection_without_partial_results() {
    use om9_core::{curve_build::circle_on_frame, geometry::Geometry, model::Entity};
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Mixed-plane extrusion");
    let mut ids = Vec::new();
    for y in [[0., 1., 0.], [0., 0., 1.]] {
        let curve = circle_on_frame([0.; 3], [1., 0., 0.], y, 1.).unwrap();
        let entity = Entity::new("circle", Geometry::Curve(curve), doc.current_layer);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    c.workspace.active = ViewId::LookingDown;
    c.select_objects(&ids).unwrap();
    for token in ["tsExtrudeLines", "Solid=Yes", "DeleteInput=Yes"] {
        c.submit_effect(token).unwrap();
    }
    assert!(c.submit_effect("1").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.document(), &doc);
    assert_eq!(c.selected_ordered(), ids);
    for token in ["Direction", "w0,0,0", "w0,1,1", "2"] {
        c.submit_effect(token).unwrap();
    }
    assert_eq!(c.document().entities.len(), 2);
    for id in c.selected_ordered() {
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else {
            panic!("expected capped circle");
        };
        let volume = om9_kernel::brep::source_volume(body, &Default::default()).unwrap();
        assert!((volume - std::f64::consts::PI * 2_f64.sqrt()).abs() < 1e-8);
    }
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_solid_caps_each_planar_profile_and_preserves_mixed_open_rails() {
    use om9_core::{
        curve_build::{circle_on_frame, polyline},
        geometry::{Curve, Geometry, Transform},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let rect = polyline(&[[0.; 3], [3., 0., 0.], [3., 2., 0.], [0., 2., 0.], [0.; 3]]).unwrap();
    let circle = circle_on_frame([12., 0., 0.], [1., 0., 0.], [0., 1., 0.], 1.).unwrap();
    let mut arc = circle.clone();
    if let Curve::Circle {
        domain,
        angle_domain,
        ..
    } = &mut arc
    {
        *domain = [0., std::f64::consts::PI];
        *angle_domain = Some(*domain);
    }
    let placement = Transform::translation([2., -1., 8.]);
    let mut doc = Document::with_template_tolerance("Extrude Lines mixed cap test");
    let mut inputs = Vec::new();
    for curve in [rect, arc.clone(), circle] {
        let mut e = Entity::new("profile", Geometry::Curve(curve), doc.current_layer);
        e.placement = placement;
        inputs.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let limits = Default::default();
    for both in [false, true] {
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.select_objects(&inputs).unwrap();
        let action = app.resolve_f6_action("_tsExtrudeLines", "Extrude Lines");
        assert!(action.availability.enabled());
        app.controller
            .dispatch(&action.id, om9_app::session::InvocationOrigin::F6)
            .unwrap();
        let c = &mut app.controller;
        c.submit_effect("Solid=Yes").unwrap();
        if both {
            c.submit_effect("BothSides=Yes").unwrap();
            c.submit_effect("DeleteInput=Yes").unwrap();
        }
        c.session_hover([0., 0., 6.]);
        assert!(!c.addon_session().unwrap().hover_lines.is_empty());
        assert_eq!(c.document(), &doc);
        assert!(c.submit_effect("0").is_err());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit_effect("-2").unwrap();
        c.capture_brep_save_sources(Default::default()).unwrap();
        #[cfg(feature = "test-fixtures")]
        let loaded = {
            let path =
                std::env::temp_dir().join(format!("om9-f6-extrude-{}.om9", uuid::Uuid::new_v4()));
            c.save_archive_for_batch(&path).unwrap();
            let reopened = om9_app::session::Controller::open_archive_for_batch(&path, limits);
            std::fs::remove_file(&path).unwrap();
            reopened.unwrap()
        };
        #[cfg(feature = "test-fixtures")]
        let restored = loaded.document();
        #[cfg(not(feature = "test-fixtures"))]
        let restored = c.document();
        let mut faces = Vec::new();
        let mut exact_arcs = 0;
        let length = if both { 4. } else { 2. };
        for id in c.selected_ordered() {
            let e = &restored.entities[&id];
            match &e.geometry {
                Geometry::Brep(body) => {
                    let n = body.topology().faces.len();
                    let area = match n {
                        3 => std::f64::consts::PI,
                        6 => 6.,
                        _ => panic!("unexpected faces: {n}"),
                    };
                    assert!(
                        (om9_kernel::brep::source_volume(body, &limits).unwrap() - area * length)
                            .abs()
                            < 1e-7
                    );
                    let bounds =
                        om9_kernel::brep::world_bounds(body, &e.placement, &limits).unwrap();
                    assert!(
                        (bounds[2] - 6.).abs() < 1e-5 && (bounds[5] - (6. + length)).abs() < 1e-5,
                        "{bounds:?}"
                    );
                    faces.push(n);
                }
                Geometry::Curve(Curve::Circle { .. }) => {
                    assert_eq!(e.geometry, Geometry::Curve(arc.clone()));
                    exact_arcs += 1;
                }
                Geometry::Curve(Curve::Line { .. }) => {}
                _ => panic!("unexpected extrusion result"),
            }
        }
        faces.sort();
        assert_eq!(faces, [3, 6]);
        assert_eq!(exact_arcs, if both { 2 } else { 1 });
        assert!(
            inputs
                .iter()
                .all(|id| restored.entities.contains_key(id) != both)
        );
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &doc);
    }
}

#[test]
fn extrude_lines_default_height_uses_placed_curve_plane_and_spatial_fallback() {
    use om9_core::{
        curve_build::{circle_on_frame, polyline},
        geometry::{Curve, Geometry, Transform},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let circle = circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    // Slab projects its preferred normal perpendicular to a collinear profile.
    // Extrude Lines keeps the CPlane direction for that same ambiguous profile.
    let collinear = [[0.;3],[1.,0.,1.],[2.,0.,2.]];
    let z=[0.,0.,1.];
    let slab=om9_core::slab::source_plane_normal(&collinear,z).unwrap();
    assert!((slab[0]+std::f64::consts::FRAC_1_SQRT_2).abs()<1e-12);
    assert!((slab[2]-std::f64::consts::FRAC_1_SQRT_2).abs()<1e-12);
    let linear = polyline(&collinear).unwrap();
    assert_eq!(om9_core::curve_ops::extrude_lines::default_axis([(&linear, Transform::identity())],z,&Default::default()).unwrap(),z);
    let shear = Transform([
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 1., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let spatial = polyline(&[[0.; 3], [1., 0., 0.], [1., 2., 0.], [1., 2., 3.]]).unwrap();
    for (input, world, expected) in [
        (circle.clone(), shear, [0., -2_f64.sqrt(), 2_f64.sqrt()]),
        (
            circle,
            Transform([
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 2., 1., 0.],
                [0., 0., 0., 1.],
            ]),
            [0., 4. / 5f64.sqrt(), -2. / 5f64.sqrt()],
        ),
        (spatial, Transform::identity(), [0., 0., 2.]),
    ] {
        let mut doc = Document::with_template_tolerance("Extrude plane");
        let mut entity = Entity::new("source", Geometry::Curve(input.clone()), doc.current_layer);
        let id = entity.id;
        entity.placement = world;
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc.clone()).unwrap();
        let c = &mut app.controller;
        c.workspace.active = ViewId::LookingDown;
        c.select_objects(&[id]).unwrap();
        c.submit_effect("tsExtrudeLines").unwrap();
        c.submit_effect("2").unwrap();
        let vectors: Vec<_> = c
            .selected_ordered()
            .iter()
            .filter_map(|id| match c.document().entities[id].geometry {
                Geometry::Curve(Curve::Line { start, end, .. }) => {
                    Some(std::array::from_fn::<_, 3, _>(|i| end[i] - start[i]))
                }
                _ => None,
            })
            .collect();
        assert!(
            vectors
                .iter()
                .any(|v| (0..3).all(|i| (v[i] - expected[i]).abs() < 1e-9)),
            "{vectors:?}, expected {expected:?}"
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &doc);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_split_at_tangents_matches_rhino5_stadium() {
    use om9_core::{
        curve_build::arc_through,
        geometry::{Curve, Geometry},
        model::Entity,
        surface_kernel,
    };
    let line = |a, b| Curve::Line {
        start: a,
        end: b,
        domain: [0., 1.],
    };
    let [a, b, c, d] = [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]];
    let curve = Curve::Polycurve {
        segments: vec![
            line(a, b),
            arc_through(b, [2., 0., 0.], c).unwrap(),
            line(c, d),
            arc_through(d, [-2., 0., 0.], a).unwrap(),
        ],
        reversed: false,
    };
    let limits = Default::default();
    let mut doc = Document::with_template_tolerance("SplitAtTangents");
    let source = Entity::new("stadium", Geometry::Curve(curve), doc.current_layer);
    let stop = Entity::new(
        "stop",
        Geometry::Surface(
            surface_kernel::plane([-3., -3., 3.], [1., 0., 0.], [0., 1., 0.], 6., 6., &limits)
                .unwrap(),
        ),
        doc.current_layer,
    );
    let (id, target) = (source.id, stop.id);
    doc.entities.insert(id, source);
    doc.entities.insert(target, stop);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    for (setting, boundary, expected) in [
        (None, false, 3),
        (Some("SplitAtTangents=Yes"), false, 6),
        (None, true, 6),
        (Some("SplitAtTangents=No"), true, 3),
    ] {
        c.select_objects(&[id]).unwrap();
        c.submit("tsExtrudeLines").unwrap();
        c.submit("Solid=Yes").unwrap();
        if let Some(option) = setting {
            c.submit(option).unwrap();
        }
        if boundary {
            c.submit("ToBoundary").unwrap();
            c.select_objects(&[target]).unwrap();
            c.refresh_addon_session();
        } else {
            c.submit("3").unwrap();
        }
        assert!(
            c.addon_session().is_none(),
            "{:?}",
            c.addon_session().map(|s| &s.reply.messages)
        );
        let ids = c.selected_ordered();
        assert_eq!(ids.len(), 1);
        let Geometry::Brep(body) = &c.document().entities[&ids[0]].geometry else {
            panic!("expected closed solid")
        };
        assert_eq!(
            body.topology().faces.len(),
            expected,
            "{setting:?} boundary={boundary}"
        );
        assert_eq!(
            body.topology().edges.len(),
            if expected == 3 { 3 } else { 12 },
            "{setting:?} boundary={boundary}"
        );
        let volume = om9_kernel::brep::source_volume(body, &limits).unwrap();
        assert!(
            (volume - 3. * (4. + std::f64::consts::PI)).abs() < 1e-6,
            "{volume}"
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &doc);
    }
}

#[test]
fn dir_on_placed_circles_shares_factory_cmd_flipall_and_undo() {
    use om9_app::{f6::catalog::Mode, session::InvocationOrigin};
    use om9_core::{
        budget::Limits,
        curve_ops::{point_at, tangent_at},
        geometry::{Curve, Geometry, Transform},
        model::Entity,
    };
    let limits = Limits::default();
    let mut doc = Document::with_template_tolerance("Dir signed domain");
    let mut ids = Vec::new();
    for dx in [-7., 7.] {
        let curve = Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 4.,
            domain: [2., 8.],
            angle_domain: Some([0., std::f64::consts::TAU]),
        };
        let mut e = Entity::new("circle", Geometry::Curve(curve), doc.current_layer);
        e.placement = Transform::translation([dx, 3., 1.]);
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.f6_mode = Mode::TSplines;
    for selection in [&ids[..1], &ids[..]] {
        app.controller.select_objects(selection).unwrap();
        let row = app
            .f6_context()
            .actions
            .into_iter()
            .find(|row| row.label == "Curve Direction")
            .unwrap();
        assert!(row.availability.enabled());
        app.controller
            .dispatch(&row.id, InvocationOrigin::F6)
            .unwrap();
        let toggle = if selection.len() == 1 {
            "Flip"
        } else {
            "FlipAll"
        };
        app.controller.submit(toggle).unwrap();
        app.controller.submit(toggle).unwrap();
        app.controller.submit("").unwrap();
        assert_eq!(app.controller.document(), &doc);
        app.controller.submit("_Dir").unwrap();
        app.controller.submit(toggle).unwrap();
        app.controller.submit("").unwrap();
        for id in selection {
            let e = &app.controller.document().entities[id];
            assert_eq!(e.placement, doc.entities[id].placement);
            let Geometry::Curve(curve) = &e.geometry else {
                panic!("curve expected")
            };
            assert!(matches!(curve,Curve::Circle{domain,..} if *domain==[-8.,-2.]));
            let start = point_at(curve, 0., &limits).unwrap();
            let tangent = tangent_at(curve, 0., &limits).unwrap();
            assert!(
                (start[0] - 4.).abs() < 1e-10
                    && start[1].abs() < 1e-10
                    && (tangent[1] + 1.).abs() < 1e-10
            );
        }
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &doc);
    }
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("Line").unwrap();
    app.controller.submit("noecho !_Dir").unwrap();
    assert!(app.controller.prompt().contains("FlipAll"));
    app.controller.cancel();
    let toolbar=app.controller.icon_target("AnalyzeDirection",None).unwrap();
    assert!(toolbar.enabled);
    app.controller.dispatch(&toolbar.command,InvocationOrigin::Menu).unwrap();
    assert!(app.controller.prompt().contains("FlipAll"));
    app.controller.submit("FlipAll").unwrap();
    app.controller.submit("Esc").unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_to_point_cmd_postselection_boundary_cancel_and_atomic_undo() {
    use om9_core::{
        curve_build::{circle_on_frame, polyline},
        geometry::{Geometry, Transform},
        model::Entity,
        surface_kernel,
    };
    let limits = Default::default();
    let mut doc = Document::with_template_tolerance("ExtrudeCrvToPoint");
    let world = Transform::translation([2., 4., 1.]);
    let circle = circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let line = polyline(&[[-2., 0., 0.], [2., 0., 0.]]).unwrap();
    let mut sources = Vec::new();
    for curve in [circle, line] {
        let mut e = Entity::new("profile", Geometry::Curve(curve), doc.current_layer);
        e.placement = world;
        sources.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let face = surface_kernel::plane(
        [-8., -8., 4.],
        [1., 0., 0.],
        [0., 1., 0.],
        24.,
        24.,
        &limits,
    )
    .unwrap();
    let stop = Entity::new("target", Geometry::Surface(face), doc.current_layer);
    let target = stop.id;
    doc.entities.insert(target, stop);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    c.select_objects(&sources).unwrap();
    c.submit("ExtrudeCrvToPoint").unwrap();
    c.submit("ToBoundary").unwrap();
    c.cancel();
    assert_eq!(c.selected_ordered(), sources);
    c.submit("ExtrudeCrvToPoint").unwrap();
    c.submit("DeleteInput=Yes").unwrap();
    // The second source would make a zero-area ruled panel. Keep both sources and the prompt.
    assert!(c.submit("w2,4,1").is_err());
    assert!(c.addon_session().is_some());
    assert_eq!(c.document(), &doc);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
    c.clear_selection();
    c.submit("_ExtrudeCrvToPoint").unwrap();
    assert!(c.addon_collects_objects());
    c.select_objects(&sources).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    for text in ["Solid=Yes", "DeleteInput=Yes", "ToBoundary"] {
        c.submit(text).unwrap();
    }
    c.select_objects(&[target]).unwrap();
    c.refresh_addon_session();
    assert_eq!(
        c.addon_session().unwrap().reply.prompt,
        "Point to extrude to"
    );
    assert!(
        !c.addon_session()
            .unwrap()
            .reply
            .options
            .iter()
            .any(|opt| opt.name == "ToBoundary")
    );
    assert_eq!(c.document(), &doc);
    c.submit("w3,6,7").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.selected_ordered().len(), 2);
    let mut face_counts = Vec::new();
    for id in c.selected_ordered() {
        let e = &c.document().entities[&id];
        assert_eq!(e.placement, world);
        let Geometry::Brep(b) = &e.geometry else {
            panic!("expected BRep")
        };
        face_counts.push(b.topology().faces.len());
        if b.topology().faces.len() == 3 {
            assert!(
                (om9_kernel::brep::source_volume(b, &limits).unwrap() - 7. * std::f64::consts::PI)
                    .abs()
                    < 1e-5
            );
        }
    }
    face_counts.sort_unstable();
    assert_eq!(face_counts, [1, 3]);
    assert!(
        sources
            .iter()
            .all(|id| !c.document().entities.contains_key(id))
    );
    assert_eq!(c.document().entities[&target], doc.entities[&target]);
    c.capture_brep_save_sources(Default::default()).unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn tapered_cmd_options_boundary_postselection_and_undo() {
    use om9_core::{
        curve_build,
        geometry::{Geometry, Transform},
        model::Entity,
        surface_kernel,
    };
    let limits = Default::default();
    let mut doc = Document::with_template_tolerance("Tapered");
    let rect = curve_build::rectangle([-2., -1., 0.], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    let mut e = Entity::new("profile", Geometry::Curve(rect), doc.current_layer);
    e.placement = Transform::translation([4., 1., 2.]);
    let id = e.id;
    doc.entities.insert(id, e);
    let plane = surface_kernel::plane(
        [-8., -8., 5.],
        [1., 0., 0.],
        [0., 1., 0.],
        20.,
        20.,
        &limits,
    )
    .unwrap();
    let e = Entity::new("boundary", Geometry::Surface(plane), doc.current_layer);
    let stop = e.id;
    doc.entities.insert(stop, e);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let c = &mut app.controller;
    c.submit("_ExtrudeCrvTapered").unwrap();
    assert!(c.addon_collects_objects());
    c.select_objects(&[id]).unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    let reply = &c.addon_session().unwrap().reply;
    assert!(reply.prompt.contains("<1>"));
    assert!(
        reply
            .options
            .iter()
            .any(|o| o.name == "DraftAngle" && o.value.as_deref() == Some("5"))
    );
    for t in [
        "DraftAngle",
        "10",
        "FlipAngle",
        "Corners=Smooth",
        "Solid=Yes",
        "DeleteInput=Yes",
        "ToBoundary",
    ] {
        c.submit(t).unwrap();
    }
    c.select_objects(&[id]).unwrap();
    c.refresh_addon_session();
    assert!(c.addon_collects_objects());
    c.cancel();
    assert_eq!(c.selected_ordered(), [id]);
    assert_eq!(c.document(), &doc);
    c.submit("ExtrudeCrvTapered").unwrap();
    assert!(c.submit("DraftAngle=0").is_err());
    assert!(c.submit("DraftAngle=91").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    for t in [
        "DraftAngle=-10",
        "Corners=Smooth",
        "Solid=Yes",
        "DeleteInput=Yes",
        "ToBoundary",
    ] {
        c.submit(t).unwrap();
    }
    c.select_objects(&[stop]).unwrap();
    c.refresh_addon_session();
    assert!(c.addon_session().is_none());
    let out = c.selected_ordered()[0];
    let Geometry::Brep(b) = &c.document().entities[&out].geometry else {
        panic!("expected taper")
    };
    assert_eq!(b.topology().faces.len(), 10);
    let v = om9_kernel::brep::source_volume(b, &limits).unwrap();
    assert!((v - 34.47222409).abs() < 1e-5, "{v}");
    assert_eq!(c.document().entities[&stop], doc.entities[&stop]);
    assert!(!c.document().entities.contains_key(&id));
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
    c.select_objects(&[id]).unwrap();
    c.submit("ExtrudeCrvTapered").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .reply
            .options
            .iter()
            .any(|o| o.name == "Corners" && o.value.as_deref() == Some("Smooth"))
    );
    c.submit("-3").unwrap();
    let Geometry::Brep(b) = &c.document().entities[&c.selected_ordered()[0]].geometry else {
        panic!("expected taper")
    };
    assert!((om9_kernel::brep::source_volume(b, &limits).unwrap() - 34.47222409).abs() < 1e-5);
    c.undo().unwrap();
    assert_eq!(c.document(), &doc);
}
