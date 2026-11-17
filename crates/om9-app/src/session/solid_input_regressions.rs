// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use super::{Controller, InvocationOrigin};
use om9_core::geometry::Geometry;
use om9_core::{budget::Limits, model::Document};

fn controller() -> Controller {
    Controller::new(Document::new("Solid input regression"), Limits::default()).unwrap()
}

#[test]
fn pipe_chain_reopened_pick_does_not_assume_native_edge_order() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    display.edges.swap(0, 1);
    for entry in &mut display.face_edges { if entry[1] == 0 { entry[1] = 1; } else if entry[1] == 1 { entry[1] = 0; } }
    for entry in &mut display.segments { if entry[2] == 0 { entry[2] = 1; } else if entry[2] == 1 { entry[2] = 0; } }
    let segment = *display.segments.iter().find(|entry| entry[2] == 0).unwrap();
    let expected = [display.points[segment[0] as usize], display.points[segment[1] as usize]];
    let edge = display.edges[0];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Pipe reordered edge");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-pipe-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    let original = c.document().clone();
    c.submit("PipeChain").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    c.solid_pipe_pick_edge(reference).unwrap();
    let super::solid_pending::PipeChainItem::Edge(_, actual) = &c.solid_pending.as_ref().unwrap().pipe_chain_order[0] else { panic!() };
    let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-8);
    assert!((near(actual[0], expected[0]) && near(actual[1], expected[1])) ||
        (near(actual[0], expected[1]) && near(actual[1], expected[0])), "PipeChain chose a different source edge");
    let length = (0..3).map(|i| (expected[1][i]-expected[0][i]).powi(2)).sum::<f64>().sqrt();
    c.submit("").unwrap(); c.submit("0.5").unwrap(); c.submit("0.5").unwrap(); c.submit("").unwrap();
    let output = c.document().entities.values().find(|entity| entity.id != id).unwrap();
    let Geometry::Brep(pipe) = &output.geometry else { panic!("Pipe output") };
    let measures = om9_kernel::creation::measure(pipe, &limits).unwrap();
    assert!((measures.volume_mm3 - length * std::f64::consts::PI * 0.25).abs() < 1e-5);
    c.undo().unwrap(); assert_eq!(c.document(), &original);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn pipe_chain_rejects_mixed_display_segments_after_reopen() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let selected = display.segments[0][2];
    let mut foreign = *display.segments.iter().find(|segment| segment[2] != selected).unwrap();
    foreign[2] = selected;
    display.segments.push(foreign);
    let edge = display.edges[selected as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Pipe mixed display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-pipe-mixed-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.submit("PipeChain").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let before = c.coordinator().snapshot();
    assert!(c.solid_pipe_pick_edge(reference).is_err(), "a mixed display edge must be refused");
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_pending.as_ref().unwrap().pipe_chain_order.is_empty());
}

#[test]
fn pipe_chain_refuses_unverified_display_edge_and_preserves_redo() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let segment = display.segments[0];
    display.points[segment[0] as usize][2] += 100.;
    display.points[segment[1] as usize][2] += 100.;
    let edge = display.edges[segment[2] as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Pipe stale display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.submit("PipeChain").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let before = c.coordinator().snapshot();
    assert!(c.solid_pipe_pick_edge(reference).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_pending.as_ref().unwrap().pipe_chain_order.is_empty());
}

#[test]
fn sphere_aroundcurve_reopened_edge_tracks_selected_source() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    display.edges.swap(0, 1);
    for entry in &mut display.face_edges { if entry[1] == 0 { entry[1] = 1; } else if entry[1] == 1 { entry[1] = 0; } }
    for entry in &mut display.segments { if entry[2] == 0 { entry[2] = 1; } else if entry[2] == 1 { entry[2] = 0; } }
    let segment = *display.segments.iter().find(|entry| entry[2] == 0).unwrap();
    let a = display.points[segment[0] as usize]; let b = display.points[segment[1] as usize];
    let midpoint = std::array::from_fn(|i| (a[i]+b[i])*0.5);
    let edge = display.edges[0];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Sphere reordered edge");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-sphere-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for route in 0..3 {
        let mut c = Controller::from_file(&path, limits).unwrap();
        c.submit("Sphere AroundCurve").unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        if route == 2 { c.submit("Path=box.Edge1").unwrap(); }
        else { c.solid_sphere_pick_brep_edge(reference.clone(), midpoint).unwrap(); }
        if route == 0 { c.solid_sphere_pick_brep_edge(reference, midpoint).unwrap(); }
        else { c.submit("OnCurve=0.5").unwrap(); }
        c.submit("1").unwrap();
        let sphere = c.document().entities.values().find(|entity| entity.id != id).unwrap();
        let center = sphere.placement.point([0.,0.,0.]).unwrap();
        assert!((0..3).all(|i| (center[i]-midpoint[i]).abs() < 1e-7), "route {route}: Sphere center {center:?}, selected midpoint {midpoint:?}");
        c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn sphere_aroundcurve_source_circle_edge_survives_reopen() {
    use om9_core::{geometry::Curve, model::Entity};
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.;3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shape = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let (index, _) = (0..shape.topology().edges.len()).filter_map(|index| {
        let curve = om9_kernel::editing::edge_nurbs(&shape, index, &limits).ok()?;
        let Curve::Nurbs(n) = &curve else { return None };
        (n.degree > 1).then_some((index, curve))
    }).next().unwrap();
    let edge = shape.topology().edges[index];
    let analytic = om9_kernel::editing::circular_edge(&shape, index, &limits).unwrap();
    let expected = om9_core::curve_ops::point_at(&analytic, 0.25, &limits).unwrap();
    let mut doc = Document::new("Sphere circle edge");
    let entity = Entity::new("shell", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-sphere-circle-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.submit("Sphere AroundCurve").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    c.solid_sphere_pick_brep_edge(reference, [2.,0.,0.]).unwrap();
    c.submit("OnCurve=0.25").unwrap(); c.submit("1").unwrap();
    let sphere = c.document().entities.values().find(|entity| entity.id != id).unwrap();
    let center = sphere.placement.point([0.,0.,0.]).unwrap();
    assert!((center[0].hypot(center[1])-2.).abs() < 1e-7, "{center:?}");
    assert!(center[2].abs() < 1e-7 || (center[2]-5.).abs() < 1e-7, "{center:?}");
    assert!((0..3).all(|i| (center[i]-expected[i]).abs() < 1e-7), "{center:?} != {expected:?}");
    c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn sphere_aroundcurve_rejects_mixed_display_segments_after_reopen() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let selected = display.segments[0][2];
    let mut foreign = *display.segments.iter().find(|segment| segment[2] != selected).unwrap();
    let midpoint = std::array::from_fn(|i| (display.points[foreign[0] as usize][i] + display.points[foreign[1] as usize][i]) * 0.5);
    foreign[2] = selected;
    display.segments.push(foreign);
    let edge = display.edges[selected as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Sphere mixed display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-sphere-mixed-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for numeric in [false, true] {
        let mut c = Controller::from_file(&path, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
        c.submit("Sphere AroundCurve").unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        let before = c.coordinator().snapshot();
        let result = if numeric { c.submit(&format!("Path=box.Edge{}", selected + 1)) }
            else { c.solid_sphere_pick_brep_edge(reference, midpoint) };
        assert!(result.is_err(), "a mixed display edge must be refused, numeric={numeric}");
        assert_eq!(c.coordinator().snapshot(), before);
        assert_eq!(c.coordinator().redo_len(), 1);
        assert_eq!(c.solid_sphere_brep_edge_stage(), 1);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn sphere_aroundcurve_refuses_unverified_edge_without_losing_redo() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let segment = display.segments[0];
    display.points[segment[0] as usize][2] += 100.;
    display.points[segment[1] as usize][2] += 100.;
    let edge = display.edges[segment[2] as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Sphere stale display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.submit("Sphere AroundCurve").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let before = c.coordinator().snapshot();
    assert!(c.solid_sphere_pick_brep_edge(reference, [0.,0.,0.]).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert_eq!(c.solid_sphere_brep_edge_stage(), 1);
}

#[test]
fn fillet_centimeter_radius_matches_source_in_direct_and_picked_paths() {
    use crate::command_options::CommandOptionInterface;
    use om9_core::{geometry::Unit, model::Entity};
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let source = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 40., y_mm: 50., z_mm: 60.
    }, 0.1, &limits).unwrap();
    let expected: Vec<_> = [1., 2.].into_iter().map(|radius| {
        let body = om9_kernel::editing::fillet_with_radii(&source, &[(0, radius)], 0.1, &limits).unwrap();
        om9_kernel::creation::measure(&body, &limits).unwrap().volume_mm3
    }).collect();
    let edge = source.topology().edges[0];
    let mut doc = Document::new("Fillet centimeters"); doc.page_unit = Unit::Centimeter;
    let entity = Entity::new("box", Geometry::Brep(source), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fillet-units-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for mode in 0..7 {
        let mut c = Controller::from_file(&path, limits).unwrap();
        let before = c.document().clone();
        if mode < 2 {
            c.select_objects(&[id]).unwrap();
            if mode == 0 { c.submit("FilletEdge 0 0.1").unwrap(); }
            else { c.dispatch("OM9-SOLID-007", super::InvocationOrigin::Menu).unwrap(); c.submit("0 0.1").unwrap(); }
        } else {
            c.submit("FilletEdge").unwrap(); c.submit("NextRadius=0.1").unwrap();
            assert!(c.command_options().iter().any(|option| option.label == "NextRadius=0.1"));
            let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
                source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_fillet_pick_edge(reference.clone()).unwrap(); c.submit("").unwrap();
            if mode == 3 {
                let preview = c.solid_fillet_preview().unwrap().0.clone();
                assert!(c.submit("SetAll=1e308").is_err());
                assert_eq!(c.solid_fillet_preview().unwrap().0, &preview);
                c.submit("SetAll=0.2").unwrap();
            }
            if mode == 4 {
                c.solid_fillet_select_handle(0).unwrap(); c.submit("CurrentRadius=0.2").unwrap();
                assert!(c.command_options().iter().any(|option| option.label == "CurrentRadius=0.2"));
                c.solid_fillet_select_handle(1).unwrap(); c.submit("CurrentRadius=0.2").unwrap();
            }
            if mode == 5 {
                c.solid_fillet_select_handle(0).unwrap(); c.submit("CurrentRadius=0.2").unwrap();
                c.submit("CopyHandle").unwrap(); c.solid_fillet_select_handle(0).unwrap();
                c.solid_fillet_copy_to_hit(&[(1, 0.)]).unwrap();
                assert!((c.solid_fillet_handles()[1].2 - 2.).abs() < 1e-10);
                c.submit("").unwrap();
            }
            if mode == 6 {
                c.submit("AddHandle").unwrap(); c.submit("CurrentRadius=0.1").unwrap();
                let (a, b) = c.solid_fillet_edge_endpoints(&reference).unwrap().unwrap();
                c.solid_fillet_add_handle(reference, std::array::from_fn(|i| (a[i] + b[i]) * 0.5)).unwrap();
                assert!((c.solid_fillet_handles()[2].2 - 1.).abs() < 1e-10);
                c.submit("").unwrap();
            }
            c.submit("").unwrap();
        }
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let volume = om9_kernel::creation::measure(body, &limits).unwrap().volume_mm3;
        let expected = expected[usize::from((3..=5).contains(&mode))];
        assert!((volume - expected).abs() < 1e-6, "mode {mode}: volume {volume}, expected {expected}");
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
        let output = std::env::temp_dir().join(format!("om9-fillet-units-output-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&output, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let reopened = Controller::from_file(&output, limits).unwrap(); std::fs::remove_file(output).unwrap();
        let Geometry::Brep(body) = &reopened.document().entities[&id].geometry else { panic!() };
        let reopened_volume = om9_kernel::creation::measure(body, &limits).unwrap().volume_mm3;
        assert!((reopened_volume - expected).abs() < 1e-6, "mode {mode}: reopened volume {reopened_volume}");
    }
    for picked in [false, true] {
        let mut c = Controller::from_file(&path, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("100,100,100").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); let before = c.coordinator().snapshot(); let redo = c.coordinator().redo_len();
        if picked {
            c.submit("FilletEdge").unwrap();
            assert!(c.submit("NextRadius=1e308").is_err());
            assert!(c.solid_pending.is_some());
        } else { assert!(c.submit("FilletEdge 0 1e308").is_err()); }
        assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), redo);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn fillet_reopened_edge_matches_selected_display_edge() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let oracle = om9_kernel::editing::fillet_with_radii(&shape, &[(1, 0.5)], 0.1, &limits).unwrap();
    let oracle_volume = om9_kernel::creation::measure(&oracle, &limits).unwrap().volume_mm3;
    let mut display = shape.display().clone();
    display.edges.swap(0, 1);
    for entry in &mut display.face_edges { if entry[1] == 0 { entry[1] = 1; } else if entry[1] == 1 { entry[1] = 0; } }
    for entry in &mut display.segments { if entry[2] == 0 { entry[2] = 1; } else if entry[2] == 1 { entry[2] = 0; } }
    let segment = *display.segments.iter().find(|entry| entry[2] == 0).unwrap();
    let expected = [display.points[segment[0] as usize], display.points[segment[1] as usize]];
    let edge = display.edges[0];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Fillet reordered edge");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fillet-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    let original = c.document().clone();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let (a,b) = c.solid_fillet_edge_endpoints(&reference).unwrap().unwrap();
    let midpoint = std::array::from_fn(|i| (expected[0][i] + expected[1][i]) * 0.5);
    let (fraction, projected) = c.solid_fillet_project_pick(&reference, om9_core::geometry::Transform::identity(), midpoint).unwrap();
    assert!((fraction - 0.5).abs() < 1e-8 && (0..3).all(|i| (projected[i]-midpoint[i]).abs() < 1e-8));
    c.submit("FilletEdge").unwrap(); c.submit("NextRadius=0.5").unwrap();
    c.solid_fillet_pick_edge(reference).unwrap(); c.submit("").unwrap(); c.submit("").unwrap();
    let Geometry::Brep(actual) = &c.document().entities[&id].geometry else { panic!() };
    let actual_volume = om9_kernel::creation::measure(actual, &limits).unwrap().volume_mm3;
    assert!((actual_volume-oracle_volume).abs() < 1e-7,
        "Fillet modified the wrong edge: volume {actual_volume}, expected {oracle_volume}");
    let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-8);
    assert!((near(a, expected[0]) && near(b, expected[1])) || (near(a, expected[1]) && near(b, expected[0])),
        "Fillet endpoints {a:?} {b:?} do not match selected edge {expected:?}");
    c.undo().unwrap(); assert_eq!(c.document(), &original);
    c.redo().unwrap();
    let Geometry::Brep(actual) = &c.document().entities[&id].geometry else { panic!() };
    assert!((om9_kernel::creation::measure(actual, &limits).unwrap().volume_mm3-oracle_volume).abs() < 1e-7);
    let mut numeric = Controller::new(original.clone(), limits).unwrap();
    numeric.select_objects(&[id]).unwrap();
    numeric.submit("FilletEdge 0 0.5").unwrap();
    let Geometry::Brep(actual) = &numeric.document().entities[&id].geometry else { panic!() };
    assert!((om9_kernel::creation::measure(actual, &limits).unwrap().volume_mm3-oracle_volume).abs() < 1e-7);
    numeric.undo().unwrap(); assert_eq!(numeric.document(), &original);
}

#[test]
fn fillet_rejects_mixed_display_segments_after_reopen() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let selected = display.segments[0][2];
    let mut foreign = *display.segments.iter().find(|segment| segment[2] != selected).unwrap();
    foreign[2] = selected;
    display.segments.push(foreign);
    let edge = display.edges[selected as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Fillet mixed display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fillet-mixed-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.submit("FilletEdge").unwrap(); c.submit("NextRadius=0.5").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let before = c.coordinator().snapshot();
    assert!(c.solid_fillet_pick_edge(reference).is_err(), "a mixed display edge must be refused");
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    c.submit("Cancel").unwrap();
    c.select_objects(&[id]).unwrap();
    assert!(c.submit(&format!("FilletEdge {selected} 0.5")).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn fillet_refuses_unverified_edge_and_preserves_redo() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let segment = display.segments[0];
    display.points[segment[0] as usize][2] += 100.;
    display.points[segment[1] as usize][2] += 100.;
    let edge = display.edges[segment[2] as usize];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Fillet stale display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.submit("FilletEdge").unwrap(); c.submit("NextRadius=0.5").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    let before = c.coordinator().snapshot();
    assert!(c.solid_fillet_pick_edge(reference).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn fillet_reopened_multiple_edges_preserves_per_edge_radii() {
    use om9_core::model::Entity;
    use om9_scene::scene::Subelement;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 10., y_mm: 10., z_mm: 10.
    }, 0.1, &limits).unwrap();
    let vertical: Vec<_> = (0..shape.topology().edges.len()).filter(|&i| {
        om9_kernel::editing::straight_edge_endpoints(&shape, i, &limits).is_ok_and(|(a,b)|
            (a[0]-b[0]).abs() < 1e-8 && (a[1]-b[1]).abs() < 1e-8 && (a[2]-b[2]).abs() > 9.)
    }).take(2).collect();
    assert_eq!(vertical.len(), 2);
    let keys = [shape.topology().edges[vertical[0]], shape.topology().edges[vertical[1]]];
    let swap = (0..shape.topology().edges.len()).find(|i| !vertical.contains(i)).unwrap();
    let mut display = shape.display().clone();
    display.edges.swap(vertical[0], swap);
    for entry in &mut display.face_edges {
        if entry[1] as usize == vertical[0] { entry[1] = swap as u32; }
        else if entry[1] as usize == swap { entry[1] = vertical[0] as u32; }
    }
    for entry in &mut display.segments {
        if entry[2] as usize == vertical[0] { entry[2] = swap as u32; }
        else if entry[2] as usize == swap { entry[2] = vertical[0] as u32; }
    }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Fillet reordered multiple edges");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fillet-multi-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    for (key, radius) in keys.into_iter().zip([1.,2.]) {
        c.submit(&format!("NextRadius={radius}")).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(key),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    c.submit("").unwrap(); c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
    let volume = om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3;
    let expected = 1000. - (1. - std::f64::consts::PI / 4.) * 50.;
    assert!((volume - expected).abs() < 1e-5, "{volume} != {expected}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
    assert!((om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3 - expected).abs() < 1e-5);
}

#[test]
fn extract_surface_reopened_numeric_face_matches_selected_display_face() {
    use om9_core::model::Entity;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let areas: Vec<_> = (0..shape.topology().faces.len()).map(|i| {
        let face = om9_kernel::editing::extract_face(&shape, i, 0.1, &limits).unwrap();
        om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2
    }).collect();
    let other = (1..areas.len()).find(|i| (areas[*i]-areas[0]).abs() > 1.).unwrap();
    let expected_area = areas[other];
    let mut display = shape.display().clone();
    display.faces.swap(0, other);
    let face_key = display.faces[0];
    let second_key = display.faces[1];
    for entry in &mut display.face_edges { if entry[0] == 0 { entry[0] = other as u32; } else if entry[0] as usize == other { entry[0] = 0; } }
    for entry in &mut display.triangles { if entry[3] == 0 { entry[3] = other as u32; } else if entry[3] as usize == other { entry[3] = 0; } }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("ExtractSurface reordered face");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-extract-face-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for (copy, picked) in [(true,false), (false,false), (true,true), (false,true)] {
        let mut c = Controller::from_file(&path, limits).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        if picked {
            let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(face_key),
                source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_extract_picks(&[reference], copy, false).unwrap();
        } else { c.solid_extract("OM9-SOLID-006", 0, copy, false).unwrap(); }
        let Geometry::Brep(face) = &c.document().entities.values().find(|entity| entity.id != id).unwrap().geometry else { panic!() };
        let actual_area = om9_kernel::creation::measure(face, &limits).unwrap().area_mm2;
        assert!((actual_area-expected_area).abs() < 1e-7, "wrong face area {actual_area}, expected {expected_area}");
        let Geometry::Brep(remainder) = &c.document().entities[&id].geometry else { panic!() };
        let expected_remainder = 148. - if copy { 0. } else { expected_area };
        assert!((om9_kernel::creation::measure(remainder, &limits).unwrap().area_mm2 - expected_remainder).abs() < 1e-7);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
    }
    let mut c = Controller::from_file(&path, limits).unwrap();
    let before = c.document().clone();
    let source = c.document().stamp(id).unwrap();
    let picks: Vec<_> = [face_key,second_key].into_iter().map(|key| super::SelectedSubobject {
        root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key), source, world_epoch: c.coordinator().epoch(),
    }).collect();
    c.solid_extract_picks(&picks, false, false).unwrap();
    let total_area: f64 = c.document().entities.values().map(|entity| {
        let Geometry::Brep(shape) = &entity.geometry else { panic!() };
        om9_kernel::creation::measure(shape, &limits).unwrap().area_mm2
    }).sum();
    assert!((total_area-148.).abs() < 1e-7);
    assert_eq!(c.document().entities.len(), 3);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 3);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn extract_surface_refuses_selected_mixed_face_triangles() {
    use om9_core::model::Entity;
    use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, Subelement}};
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let top_triangle = display.triangles.iter().position(|triangle| triangle[..3].iter()
        .all(|vertex| (display.points[*vertex as usize][2] - 6.).abs() < 1e-8)).unwrap();
    let triangle = display.triangles[top_triangle];
    let top = triangle[3];
    let other = display.triangles.iter().find(|triangle| triangle[3] != top && triangle[..3].iter()
        .any(|vertex| display.points[*vertex as usize][2] < 1.)).unwrap()[3];
    let target_key = display.faces[other as usize];
    let cursor_world: [f64; 3] = std::array::from_fn(|i| triangle[..3].iter().map(|v| display.points[*v as usize][i]).sum::<f64>() / 3.);
    let mut misplaced = display.triangles.remove(top_triangle);
    misplaced[3] = other;
    // Check valid points first and the bad triangle last, so a reused OCCT
    // distance object must not retain a prior successful result.
    display.triangles.push(misplaced);
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::with_template_tolerance("mixed face display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 10.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project(cursor_world, viewport).unwrap().unwrap().screen;
    let hit = c.scene().unwrap().pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
    assert_eq!(hit.subset, Subelement::BrepFace(target_key));
    let reference = super::SelectedSubobject::from_hit(&hit);
    let before = c.coordinator().snapshot();
    assert!(c.solid_extract_picks(&[reference.clone()], true, false).is_err(), "a clicked triangle from another source face must not produce a face");
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    c.select_subobject(id, Subelement::BrepFace(target_key)).unwrap();
    assert!(c.solid_shell_picks(&[reference], 0.5).is_err(), "Shell must also refuse the mixed face");
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn extract_surface_unverified_face_preserves_document_and_redo() {
    use om9_core::model::Entity;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let triangle = *display.triangles.iter().find(|entry| entry[3] == 0).unwrap();
    for vertex in &triangle[..3] { display.points[*vertex as usize][2] += 100.; }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("ExtractSurface stale display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.solid_extract("OM9-SOLID-006", 0, false, false).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn extract_surface_reopened_cylindrical_face_keeps_source_area() {
    use om9_core::model::Entity;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {
        radius_mm: 2., height_mm: 5.
    }, 0.1, &limits).unwrap();
    let side = (0..shape.topology().faces.len()).find(|&i| {
        let face = om9_kernel::editing::extract_face(&shape, i, 0.1, &limits).unwrap();
        (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 20. * std::f64::consts::PI).abs() < 1e-6
    }).unwrap();
    let display = shape.display();
    let tri = display.triangles.iter().find(|tri| tri[3] as usize == side).unwrap();
    let witness = [display.points[tri[0] as usize], display.points[tri[1] as usize], display.points[tri[2] as usize]];
    assert!(om9_kernel::editing::verify_source_face_triangles(&shape, &[witness,witness], 1e-6, &limits).is_err());
    let mut doc = Document::new("ExtractSurface cylinder");
    let entity = Entity::new("cylinder", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-extract-cylinder-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap();
    c.solid_extract("OM9-SOLID-006", side, true, false).unwrap();
    let Geometry::Brep(face) = &c.document().entities.values().find(|entity| entity.id != id).unwrap().geometry else { panic!() };
    let area = om9_kernel::creation::measure(face, &limits).unwrap().area_mm2;
    assert!((area-20.*std::f64::consts::PI).abs() < 1e-6, "{area}");
    c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn shell_centimeter_thickness_matches_source_in_direct_and_picked_paths() {
    use om9_core::{geometry::Unit, model::Entity};
    let limits = Limits::default();
    let source = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 40., y_mm: 50., z_mm: 60.
    }, 0.1, &limits).unwrap();
    let expected_shape = om9_kernel::editing::shell(&source, 0, 1., 0.1, &limits).unwrap();
    let expected = om9_kernel::creation::measure(&expected_shape, &limits).unwrap().volume_mm3;
    let key = source.topology().faces[0];
    let mut doc = Document::new("Shell centimeters"); doc.page_unit = Unit::Centimeter;
    let entity = Entity::new("box", Geometry::Brep(source), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-shell-units-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for mode in 0..4 {
        let mut c = Controller::from_file(&path, limits).unwrap();
        let before = c.document().clone();
        if mode < 2 {
            c.select_objects(&[id]).unwrap();
            if mode == 0 { c.submit("Shell 0 0.1").unwrap(); }
            else { c.dispatch("OM9-SOLID-029", super::InvocationOrigin::Menu).unwrap(); c.submit("0 0.1").unwrap(); }
        } else {
            c.submit("Shell").unwrap();
            if mode == 2 { c.submit("Thickness=0.1").unwrap(); }
            let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
                source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_extract_pick_face(reference).unwrap(); c.submit("").unwrap();
            if mode == 3 { c.submit("0.1").unwrap(); }
        }
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
        let volume = om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3;
        assert!((volume - expected).abs() < 1e-6, "mode {mode}: volume {volume}, expected {expected}");
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
    let mut c = Controller::from_file(&path, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("100,100,100").unwrap(); c.undo().unwrap();
    let before = c.coordinator().snapshot(); let redo = c.coordinator().redo_len();
    c.submit("Shell").unwrap(); c.submit("Thickness=1e308").unwrap();
    let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
    c.solid_extract_pick_face(reference).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), redo);
    assert!(c.solid_pending.is_some());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn shell_reopened_face_matches_selected_display_face() {
    use om9_core::model::Entity;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let areas: Vec<_> = (0..shape.topology().faces.len()).map(|i| {
        let face = om9_kernel::editing::extract_face(&shape, i, 0.1, &limits).unwrap();
        om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2
    }).collect();
    let other = (1..areas.len()).find(|i| (areas[*i]-areas[0]).abs() > 1.).unwrap();
    let oracle = om9_kernel::editing::shell(&shape, other, 0.5, 0.1, &limits).unwrap();
    let expected = om9_kernel::creation::measure(&oracle, &limits).unwrap().volume_mm3;
    let mut display = shape.display().clone();
    display.faces.swap(0, other);
    let selected_key = display.faces[0];
    for entry in &mut display.face_edges { if entry[0] == 0 { entry[0] = other as u32; } else if entry[0] as usize == other { entry[0] = 0; } }
    for entry in &mut display.triangles { if entry[3] == 0 { entry[3] = other as u32; } else if entry[3] as usize == other { entry[3] = 0; } }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Shell reordered face");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-shell-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for picked in [false,true] {
        let mut c = Controller::from_file(&path, limits).unwrap();
        let before = c.document().clone();
        if picked {
            c.submit("Shell").unwrap(); c.submit("Thickness=0.5").unwrap();
            let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(selected_key),
                source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_extract_pick_face(reference).unwrap(); c.submit("").unwrap();
        } else {
            c.select_objects(&[id]).unwrap(); c.submit("Shell 0 0.5").unwrap();
        }
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
        let volume = om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3;
        assert!((volume-expected).abs() < 1e-7, "wrong opening: volume {volume}, expected {expected}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap();
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
        assert!((om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3-expected).abs() < 1e-7);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn shell_refuses_unverified_face_and_preserves_redo() {
    use om9_core::model::Entity;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
        x_mm: 4., y_mm: 5., z_mm: 6.
    }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let triangle = *display.triangles.iter().find(|entry| entry[3] == 0).unwrap();
    for vertex in &triangle[..3] { display.points[*vertex as usize][2] += 100.; }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("Shell stale display");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id; doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.solid_shell("OM9-SOLID-029", 0, 0.5).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn pyramid_rejects_fractional_side_count_without_mutation() {
    let mut c = controller();
    assert!(c.submit("Pyramid 4.5 3 5").is_err());
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn pyramid_prompt_retries_invalid_complete_input() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-018", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("4.5 3 5").is_err());
    assert_eq!(c.prompt(), "Center of pyramid base");
    assert!(c.document().entities.is_empty());
    c.submit("4 3 5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn pyramid_rejects_invalid_side_count_before_advancing() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-018", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("NumSides=4.5").is_err());
    assert_eq!(c.prompt(), "Center of pyramid base");
    c.submit("NumSides=4").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    assert_eq!(c.prompt(), "Base corner or radius");
    c.submit("3").unwrap();
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn numeric_prompt_preserves_previous_inputs_on_parse_and_geometry_errors() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-016", InvocationOrigin::Menu).unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("3").unwrap();
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.submit("5 nope").is_err());
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.submit("0").is_err());
    assert_eq!(c.prompt(), "Height or point");
    assert!(c.document().entities.is_empty());
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn boolean_operation_failure_keeps_prompt_and_document_unchanged() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
    assert!(c.submit("U").is_err());
    assert!(c.prompt().contains("Operation"));
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
}

#[test]
fn boolean_difference_uses_operand_pick_order_for_asymmetric_boxes() {
    for route in 0..4 {
        for method in 0..4 {
            for reverse in [false, true] {
                let limits = Limits::default();
                let large = om9_core::identity::ObjectId(uuid::Uuid::from_u128(1));
                let small = om9_core::identity::ObjectId(uuid::Uuid::from_u128(2));
                let mut doc = Document::new("Asymmetric Boolean");
                for (id, side) in [(large, 10.0), (small, 5.0)] {
                    let shape = om9_kernel::creation::primitive(
                        om9_kernel::creation::Primitive::Box {
                            x_mm: side,
                            y_mm: side,
                            z_mm: side,
                        },
                        0.01,
                        &limits,
                    )
                    .unwrap();
                    let mut entity =
                        om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
                    entity.id = id;
                    doc.entities.insert(id, entity);
                }
                let mut c = Controller::new(doc, limits).unwrap();
                c.select_objects(&[small]).unwrap();
                c.submit("Move").unwrap();
                c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
                c.accept_world_point([7.5, 0.0, 0.0]).unwrap();
                c.clear_selection();
                let ids = if reverse {
                    [small, large]
                } else {
                    [large, small]
                };
                match method {
                    0 => c.select_objects(&ids).unwrap(),
                    1 => {
                        c.select_root(Some(ids[0]), false).unwrap();
                        c.select_root(Some(ids[1]), true).unwrap();
                    }
                    2 => {
                        c.add_objects(&ids[..1]).unwrap();
                        c.add_objects(&ids[1..]).unwrap();
                    }
                    _ => {
                        c.toggle_selection(ids[0]).unwrap();
                        c.toggle_selection(ids[1]).unwrap();
                    }
                }
                if route < 2 {
                    c.remove_objects(&[ids[1]]).unwrap();
                }
                match route {
                    0 => c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap(),
                    1 => c.submit("BooleanDifference").unwrap(),
                    2 => {
                        c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
                        c.submit("DeleteInput=Yes").unwrap();
                        c.submit("D").unwrap();
                        c.submit("").unwrap();
                    }
                    _ => {
                        c.submit("Boolean2Objects").unwrap();
                        c.submit("DeleteInput=Yes").unwrap();
                        c.submit("D").unwrap();
                        c.submit("").unwrap();
                    }
                }
                if route < 2 {
                    c.submit("DeleteInput=Yes").unwrap();
                    c.submit("").unwrap();
                    c.solid_difference_pick(ids[1], false).unwrap();
                    c.submit("").unwrap();
                }
                let body = c
                    .document()
                    .entities
                    .values()
                    .find_map(|e| match &e.geometry {
                        Geometry::Brep(b) => Some(b),
                        _ => None,
                    })
                    .unwrap();
                let volume = om9_kernel::creation::measure(body, c.coordinator().limits())
                    .unwrap()
                    .volume_mm3;
                let expected = if reverse { 62.5 } else { 937.5 };
                assert!(
                    (volume - expected).abs() < 1e-6,
                    "route={route} method={method} reverse={reverse} volume={volume}"
                );
                c.undo().unwrap();
                assert_eq!(c.document().entities.len(), 2);
                c.redo().unwrap();
                assert_eq!(c.document().entities.len(), 1);
            }
        }
    }
}

#[test]
fn selection_order_follows_remove_reselect_and_replacement() {
    let mut c = controller();
    c.submit("Box 2 2 2").unwrap();
    c.submit("Box 3 3 3").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&[ids[1], ids[0]]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
    c.remove_objects(&[ids[1]]).unwrap();
    c.add_objects(&[ids[1]]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    c.toggle_selection(ids[0]).unwrap();
    c.toggle_selection(ids[0]).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
    c.clear_selection();
    c.select_root(Some(ids[0]), false).unwrap();
    c.select_root(Some(ids[1]), true).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    let revision = c.selection_revision;
    c.select_objects(&[ids[1], ids[0]]).unwrap();
    assert!(c.selection_revision > revision);
    assert_eq!(c.selected_ordered(), vec![ids[1], ids[0]]);
}

#[test]
fn boolean_union_of_three_breps_is_atomic_and_preserves_disconnected_solids() {
    for via_menu in [true, false] {
        let limits = Limits::default();
        let mut doc = Document::new("Three-way union");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0, 30.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        if via_menu {
            c.select_objects(&ids).unwrap();
        } else {
            c.select_objects(&[ids[0], ids[2], ids[1]]).unwrap();
        }
        if via_menu {
            c.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("BooleanUnion").unwrap();
        }
        c.submit("DeleteInput=Yes").unwrap();
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), 1);
        let Geometry::Brep(result) = &c.document().entities.values().next().unwrap().geometry
        else {
            panic!("expected BRep");
        };
        let measures = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
        assert!((measures.volume_mm3 - 2500.0).abs() < 1e-6);
        assert_eq!(measures.solids, 2);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 3);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
        let result_id = *c.document().entities.keys().next().unwrap();
        let path = std::env::temp_dir().join(format!("om9-union-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(
            &path,
            c.document(),
            c.coordinator().limits(),
            &om9_core::transaction::Cancellation::default(),
        )
        .unwrap();
        let restored = om9_archive::Archive::open_restored(
            &path,
            c.coordinator().limits(),
            &Default::default(),
        )
        .unwrap();
        std::fs::remove_file(&path).unwrap();
        let Geometry::Brep(restored_body) = &restored.document.entities[&result_id].geometry else {
            panic!("restored BRep");
        };
        let restored_measures =
            om9_kernel::creation::measure(restored_body, c.coordinator().limits()).unwrap();
        assert!((restored_measures.volume_mm3 - 2500.0).abs() < 1e-6);
        assert_eq!(restored_measures.solids, 2);
    }
}

#[test]
fn boolean_union_invalid_late_operand_preserves_document_and_redo() {
    let limits = Limits::default();
    let mut doc = Document::new("Invalid union operand");
    let mut ids = Vec::new();
    for (index, side) in [2.0, 3.0].into_iter().enumerate() {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut point =
        om9_core::model::Entity::new("not a solid", Geometry::Point([10.0, 0.0, 0.0]), None);
    point.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(3));
    ids.push(point.id);
    doc.entities.insert(point.id, point);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn boolean_union_late_open_brep_failure_is_atomic() {
    let limits = Limits::default();
    let mut doc = Document::new("Late union failure");
    let mut ids = Vec::new();
    for index in 0..3 {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let body = if index == 2 {
            om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap()
        } else {
            body
        };
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement =
            om9_core::geometry::Transform::translation([index as f64 * 5.0, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn boolean_union_staged_selection_and_delete_input_are_atomic() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Union selection and input retention");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        if delete_input {
            c.select_objects(&[ids[0]]).unwrap();
            c.submit("BooleanUnion DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap();
            c.solid_boolean_pick(ids[0], false).unwrap();
        }
        assert!(c.solid_boolean_needs_object());
        assert_eq!(c.active_feature(), Some("OM9-SOLID-003"));
        assert_eq!(
            c.command_options()
                .into_iter()
                .find(|o| o.id.name() == "DeleteInput")
                .unwrap()
                .value,
            CommandValue::Flag(delete_input)
        );
        let before = c.document().clone();
        assert!(c.submit("").is_err());
        assert_eq!(c.document(), &before);
        assert!(c.submit("DeleteInput=invalid").is_err());
        c.solid_boolean_pick(ids[1], false).unwrap();
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let output = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &output.geometry else {
            panic!("BRep result")
        };
        let measured = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert!((measured.volume_mm3 - 1500.0).abs() < 1e-6);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
    }
}

#[test]
fn boolean_union_pending_undo_and_cancel_preserve_document() {
    let mut c = controller();
    c.submit("Box 2 2 2").unwrap();
    c.submit("Box 3 3 3").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.clear_selection();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.submit("BooleanUnion").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    assert!(c.submit("BooleanUnion unexpected").is_err());
    assert!(c.solid_boolean_needs_object());
    assert_eq!(c.document(), &before);
    c.cancel();
}

#[test]
fn boolean_intersection_fuses_two_sets_and_honors_delete_input() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Intersection sets");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0, 8.0, 12.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids[..2]).unwrap();
        let before = c.document().clone();
        assert!(c.dispatch_solid_args("OM9-SOLID-002", &[]).is_err());
        if delete_input {
            c.submit("BooleanIntersection DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("OM9-SOLID-002", InvocationOrigin::Menu).unwrap();
        }
        assert!(c.solid_boolean_needs_object());
        assert_eq!(c.active_feature(), Some("OM9-SOLID-002"));
        assert!(c.prompt().contains("first set"));
        assert_eq!(c.document(), &before);
        let option = c
            .command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap();
        assert_eq!(option.value, CommandValue::Flag(delete_input));
        assert!(c.submit("DeleteInput=invalid").is_err());
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert!(c.prompt().contains("second set"));
        assert!(c.solid_boolean_pick(ids[0], false).is_err());
        for id in &ids[2..] {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 5 }
        );
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let entity = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &entity.geometry else {
            panic!("BRep result")
        };
        let measured = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(measured.solids, 1);
        assert!(
            (measured.volume_mm3 - 700.0).abs() < 1e-6,
            "{}",
            measured.volume_mm3
        );
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 5 }
        );
    }
}

#[test]
fn boolean_intersection_empty_result_preserves_document_and_redo() {
    let limits = Limits::default();
    let mut doc = Document::new("Disjoint intersection");
    let mut ids = Vec::new();
    for (index, x) in [0.0, 30.0].into_iter().enumerate() {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("BooleanIntersection DeleteInput=Yes").unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_boolean_needs_object());
    c.cancel();
}

#[test]
fn boolean_intersection_pending_undo_and_cancel_preserve_document() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanIntersection").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[2], false).unwrap();
    c.submit("Undo").unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().contains("first set"));
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn boolean_intersection_open_late_input_is_atomic_and_retryable() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep")
    };
    entity.geometry = Geometry::Brep(
        om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap(),
    );
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("BooleanIntersection").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[2], false).unwrap();
    c.solid_boolean_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert!(c.solid_boolean_needs_object());
    c.solid_boolean_pick(ids[3], true).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn boolean_two_objects_still_rejects_three_operands() {
    let mut c = controller();
    for side in [2, 3, 4] {
        c.submit(&format!("Box {side} {side} {side}")).unwrap();
    }
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.submit("Boolean2Objects").unwrap();
    assert!(c.submit("U").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
}

#[test]
fn boolean_split_stages_same_inputs_and_commits_separate_pieces_atomically() {
    for delete in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Boolean Split sets");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        if delete {
            c.submit("BooleanSplit DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("SolidBooleanSplit", InvocationOrigin::Menu)
                .unwrap();
        }
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 5 });
        for id in &ids {
            assert_eq!(c.document().entities.contains_key(id), !delete);
        }
        let mut positions = Vec::new();
        for (id, entity) in &c.document().entities {
            if ids.contains(id) {
                continue;
            }
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("BRep piece")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 500.0).abs() < 1e-6);
            positions.push(m.bounds_mm[0]);
        }
        positions.sort_by(f64::total_cmp);
        assert_eq!(positions.len(), 3);
        for (actual, expected) in positions.into_iter().zip([0.0, 5.0, 10.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 5 });
    }
}

#[test]
fn boolean_split_rejects_stale_or_open_input_without_partial_commit() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut stale, ids) = difference_group_controller();
    stale.submit("BooleanSplit").unwrap();
    stale.solid_boolean_pick(ids[0], false).unwrap();
    stale.submit("").unwrap();
    let source = stale.document().stamp(ids[0]).unwrap();
    let edit = stale
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    stale.coordinator.commit(edit).unwrap();
    stale.refresh_scene().unwrap();
    stale.solid_boolean_pick(ids[2], false).unwrap();
    let before = stale.document().clone();
    let undo = stale.coordinator.undo_len();
    assert!(matches!(stale.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(stale.document(), &before);
    assert_eq!(stale.coordinator.undo_len(), undo);
    assert!(stale.is_running());

    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep cutter")
    };
    entity.geometry = Geometry::Brep(
        om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap(),
    );
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("BooleanSplit DeleteInput=Yes").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.cancel();
}

#[test]
fn boolean_split_pending_undo_and_cancel_do_not_publish() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanSplit").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("Undo").unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
}

#[test]
fn create_solid_forms_regions_from_faces_and_honors_delete_input() {
    for delete in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Create solid faces");
        let mut ids = Vec::new();
        for (body_index, x) in [0.0, 5.0].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            for face_index in 0..6 {
                let face =
                    om9_kernel::editing::extract_face(&body, face_index, 0.01, &limits).unwrap();
                let mut entity = om9_core::model::Entity::new("face", Geometry::Brep(face), None);
                entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(
                    (body_index * 6 + face_index + 1) as u128,
                ));
                entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
                ids.push(entity.id);
                doc.entities.insert(entity.id, entity);
            }
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        c.select_objects(&ids).unwrap();
        if delete {
            c.submit("CreateSolid DeleteInput=Yes").unwrap();
        } else {
            c.dispatch("SolidCreateSolid", InvocationOrigin::Menu)
                .unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator.undo_len(), 0);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 15 });
        let mut starts = Vec::new();
        for (id, entity) in &c.document().entities {
            if ids.contains(id) {
                continue;
            }
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("solid region")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 500.0).abs() < 1e-6);
            starts.push(m.bounds_mm[0]);
        }
        starts.sort_by(f64::total_cmp);
        for (actual, expected) in starts.into_iter().zip([0.0, 5.0, 10.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), if delete { 3 } else { 15 });
    }
}

#[test]
fn create_solid_open_set_and_cancel_preserve_document_and_redo() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let body = match &doc.entities[&ids[0]].geometry {
        Geometry::Brep(body) => body,
        _ => panic!("BRep"),
    };
    let face = om9_kernel::editing::extract_face(body, 0, 0.01, &Limits::default()).unwrap();
    doc.entities.get_mut(&ids[0]).unwrap().geometry = Geometry::Brep(face);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("CreateSolid DeleteInput=Yes").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    assert!(c.selected().is_empty());
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), 1);
}

#[test]
fn extrude_all_inline_and_menu_use_document_units_for_every_source() {
    use om9_core::geometry::{Transform, Unit};
    let limits=Limits::default(); let mut document=Document::new("extrude all centimeters");
    document.page_unit=Unit::Centimeter;
    let curve=om9_core::curve_build::rectangle([0.;3],[1.,0.,0.],[0.,1.,0.],4.,2.).unwrap();
    let mut ids=Vec::new();
    for x in [0.,10.] {
        let mut entity=om9_core::model::Entity::new("rectangle",Geometry::Curve(curve.clone()),None);
        entity.placement=Transform::translation([x,0.,0.]); ids.push(entity.id); document.entities.insert(entity.id,entity);
    }
    for menu in [false,true] {
    let mut c=Controller::new(document.clone(),limits).unwrap(); c.select_objects(&ids).unwrap(); let before=c.document().clone();
    if menu { c.dispatch("OM9-SOLID-010",InvocationOrigin::Menu).unwrap(); c.submit("2").unwrap(); }
    else { c.submit("ExtrudeAll 2").unwrap(); }
    let bodies:Vec<_>=c.document().entities.values().filter_map(|e| match &e.geometry {
        Geometry::Brep(body)=>Some((body,e.placement)), _=>None,
    }).collect();
    assert_eq!(bodies.len(),2);
    for (body,placement) in bodies {
        let bounds=om9_kernel::brep::world_bounds(body,&placement,&limits).unwrap();
        assert!((bounds[2]).abs()<1e-6 && (bounds[5]-20.).abs()<1e-6,"{bounds:?}");
    }
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);
    c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn extrude_inline_and_menu_respect_centimeter_document_units_after_reopen() {
    use om9_core::geometry::Unit;
    let limits = Limits::default();
    for (height,menu) in [(2.,false),(-2.,false),(2.,true),(-2.,true)] {
        let curve = om9_core::curve_build::rectangle([0.;3],[1.,0.,0.],[0.,1.,0.],4.,2.).unwrap();
        let mut entity = om9_core::model::Entity::new("rectangle",Geometry::Curve(curve),None);
        entity.placement = om9_core::geometry::Transform::translation([10.,20.,30.]);
        let id=entity.id; let mut document=Document::new("extrusion centimeters");
        document.page_unit=Unit::Centimeter; document.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-extrude-cm-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.document().clone();
        if menu { c.dispatch("OM9-SOLID-009",InvocationOrigin::Menu).unwrap(); c.submit(&height.to_string()).unwrap(); }
        else { c.submit(&format!("ExtrudeCrv {height}")).unwrap(); }
        let output=c.document().entities.values().find(|entity|entity.id!=id).unwrap();
        let Geometry::Brep(body)=&output.geometry else {panic!("BRep")};
        let bounds=om9_kernel::brep::world_bounds(body,&output.placement,&limits).unwrap();
        let (bottom,top)=if height>0. {(30.,50.)} else {(10.,30.)};
        assert!((bounds[2]-bottom).abs()<1e-6 && (bounds[5]-top).abs()<1e-6,"menu={menu}, height={height}, bounds={bounds:?}");
        let after=c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before);
        c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn extrude_reopened_curve_respects_affine_placement_and_world_height() {
    let limits = Limits::default();
    for (placement, expected, height, menu) in [
        (om9_core::geometry::Transform([[2., 0., 0., 10.], [0., 3., 0., 20.], [0., 0., 2., 30.], [0., 0., 0., 1.]]), [10., 20., 30., 18., 26., 35.], 5., false),
        (om9_core::geometry::Transform([[0., -3., 0., 10.], [2., 0., 0., 20.], [0., 0., 2., 30.], [0., 0., 0., 1.]]), [4., 20., 30., 10., 28., 35.], 5., true),
        (om9_core::geometry::Transform([[1., 0., 0.5, 10.], [0., 1., 0., 20.], [0., 0., 1., 30.], [0., 0., 0., 1.]]), [10., 20., 30., 14., 22., 35.], 5., false),
        (om9_core::geometry::Transform([[2., 0., 0., 10.], [0., 3., 0., 20.], [0., 0., 2., 30.], [0., 0., 0., 1.]]), [10., 20., 25., 18., 26., 30.], -5., false),
    ] {
    let curve = om9_core::curve_build::rectangle([0.; 3], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    let mut entity = om9_core::model::Entity::new("placed rectangle", Geometry::Curve(curve), None);
    entity.placement = placement;
    let id = entity.id;
    let mut document = Document::new("placed extrusion");
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-extrude-placement-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    if menu { c.dispatch("OM9-SOLID-009", InvocationOrigin::Menu).unwrap(); c.submit(&height.to_string()).unwrap(); }
    else { c.submit(&format!("ExtrudeCrv {height}")).unwrap(); }
    let output = c.document().entities.values().find(|entity| entity.id != id).unwrap();
    let Geometry::Brep(body) = &output.geometry else { panic!("extrusion result") };
    let bounds = om9_kernel::brep::world_bounds(body, &output.placement, &limits).unwrap();
    for (actual, expected) in bounds.into_iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-5, "bounds {bounds:?}");
    }
    let after = c.document().clone();
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extrude_reopened_reflected_circle_uses_active_cplane_direction() {
    let limits = Limits::default();
    let curve = om9_core::curve_build::circle_on_frame([0.; 3], [0., 1., 0.], [0., 0., 1.], 2.).unwrap();
    let mut entity = om9_core::model::Entity::new("placed circle", Geometry::Curve(curve), None);
    entity.placement = om9_core::geometry::Transform([[-2., 0., 0., 10.], [0., 3., 0., 20.], [0., 0., 2., 30.], [0., 0., 0., 1.]]);
    let id = entity.id;
    let mut document = Document::new("reflected circle extrusion");
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-extrude-reflected-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.set_cplane(om9_scene::camera::CPlane::new([0.; 3], [0., 1., 0.], [0., 0., 1.]).unwrap()).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("ExtrudeCrv 5").unwrap();
    let output = c.document().entities.values().find(|entity| entity.id != id).unwrap();
    let Geometry::Brep(body) = &output.geometry else { panic!("extrusion result") };
    let bounds = om9_kernel::brep::world_bounds(body, &output.placement, &limits).unwrap();
    for (actual, expected) in bounds.into_iter().zip([10., 14., 26., 15., 26., 34.]) {
        assert!((actual - expected).abs() < 1e-5, "bounds {bounds:?}");
    }
    let local = om9_kernel::creation::measure(body, &limits).unwrap();
    assert!(local.closed && local.solids == 1);
    assert!((local.volume_mm3 * 12. - 120. * std::f64::consts::PI).abs() < 1e-5);
    let after = c.document().clone();
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn extrude_all_uses_each_curve_placement_in_one_transaction() {
    let limits = Limits::default();
    let curve = om9_core::curve_build::rectangle([0.; 3], [1., 0., 0.], [0., 1., 0.], 4., 2.).unwrap();
    let mut document = Document::new("placed extrude all");
    let mut ids = Vec::new();
    for x in [10., 30.] {
        let mut entity = om9_core::model::Entity::new("rectangle", Geometry::Curve(curve.clone()), None);
        entity.placement = om9_core::geometry::Transform::translation([x, 20., 30.]);
        ids.push(entity.id); document.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("ExtrudeAll 5").unwrap();
    let mut bounds = c.document().entities.values().filter(|entity| !before.entities.contains_key(&entity.id)).map(|entity| {
        let Geometry::Brep(body) = &entity.geometry else { panic!("extrusion result") };
        om9_kernel::brep::world_bounds(body, &entity.placement, &limits).unwrap()
    }).collect::<Vec<_>>();
    bounds.sort_by(|a, b| a[0].total_cmp(&b[0]));
    assert_eq!(bounds.len(), 2);
    for (actual, expected) in bounds.iter().zip([[10., 20., 30., 14., 22., 35.], [30., 20., 30., 34., 22., 35.]]) {
        for (a, e) in actual.iter().zip(expected) { assert!((*a - e).abs() < 1e-5, "bounds {actual:?}"); }
    }
    let after = c.document().clone();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn cap_planar_cylinder_source_is_independent_of_display_profile() {
    use om9_kernel::{brep, creation, editing};
    let limits = Limits::default();
    let cylinder = creation::primitive(creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let top = (0..cylinder.display().faces.len()).find(|&face| {
        let triangles: Vec<_> = cylinder.display().triangles.iter().filter(|triangle| triangle[3] as usize == face).collect();
        !triangles.is_empty() && triangles.iter().all(|triangle| triangle[..3].iter()
            .all(|index| (cylinder.display().points[*index as usize][2] - 5.).abs() < 1e-8))
    }).unwrap();
    let shell = editing::remove_face(&cylinder, top, 0.01, &limits).unwrap();
    let source = shell.source().to_owned();
    let mut triangle_counts = Vec::new();
    for (profile, menu) in [(0.5, true), (0.01, false)] {
        let sampled = brep::read_brep(&source, profile, &limits).unwrap();
        triangle_counts.push(sampled.display().triangles.len());
        let mut document = Document::new("cylinder cap profiles");
        let mut entity = om9_core::model::Entity::new("open cylinder", Geometry::Brep(sampled), None);
        entity.placement = om9_core::geometry::Transform::translation([12., -3., 4.]);
        let id = entity.id;
        document.entities.insert(id, entity);
        let path = std::env::temp_dir().join(format!("om9-cap-profile-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(path).unwrap();
        let before = c.document().clone();
        if menu { c.dispatch("OM9-SOLID-005", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Cap").unwrap(); }
        c.solid_boolean_pick(id, false).unwrap();
        c.submit("").unwrap();
        let entity = &c.document().entities[&id];
        assert_eq!(entity.placement, before.entities[&id].placement);
        let Geometry::Brep(result) = &entity.geometry else { panic!("capped cylinder") };
        let measure = creation::measure(result, &limits).unwrap();
        assert!(measure.closed && measure.solids == 1 && measure.naked_edges == 0);
        assert!((measure.volume_mm3 - 20. * std::f64::consts::PI).abs() < 1e-6);
        let after = c.document().clone();
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
    assert_ne!(triangle_counts[0], triangle_counts[1], "profiles must produce distinct displays");
}

#[test]
fn cap_planar_stages_multiple_shells_and_commits_one_transaction() {
    for menu in [true, false] {
        let limits = Limits::default();
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let shell = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
        let mut doc = Document::new("Cap multiple");
        let mut ids = Vec::new();
        for (index, x) in [0.0, 20.0].into_iter().enumerate() {
            let mut entity =
                om9_core::model::Entity::new("open box", Geometry::Brep(shell.clone()), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        if menu {
            c.dispatch("OM9-SOLID-005", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("Cap").unwrap();
        }
        for id in &ids {
            c.solid_boolean_pick(*id, false).unwrap();
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator.undo_len(), 0);
        c.submit("").unwrap();
        for (index, id) in ids.iter().enumerate() {
            let entity = &c.document().entities[id];
            assert_eq!(entity.placement, before.entities[id].placement);
            let Geometry::Brep(body) = &entity.geometry else {
                panic!("capped BRep")
            };
            let m = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(
                m.closed && m.solids == 1 && (m.volume_mm3 - 1000.0).abs() < 1e-6,
                "{index}"
            );
        }
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
    }
}

#[test]
fn cap_planar_late_invalid_input_is_atomic_and_retryable() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let shell = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
    let mut doc = Document::new("Cap failure");
    let mut ids = Vec::new();
    for (index, shape) in [shell, body].into_iter().enumerate() {
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(shape), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    for id in &ids {
        c.solid_boolean_pick(*id, false).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert_eq!(c.coordinator.redo_len(), 1);
}

#[test]
fn cap_planar_multiple_loops_through_command_and_undo() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let once = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
    let shell = (0..once.display().faces.len())
        .filter_map(|i| om9_kernel::editing::remove_face(&once, i, 0.01, &limits).ok())
        .find(|shape| {
            om9_kernel::creation::measure(shape, &limits)
                .unwrap()
                .naked_edges
                == 8
        })
        .expect("opposite faces removed");
    let mut doc = Document::new("Cap two loops");
    let entity = om9_core::model::Entity::new("open box", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        measure.closed && measure.naked_edges == 0 && (measure.volume_mm3 - 1000.0).abs() < 1e-6
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn cap_planar_annular_hole_through_command_and_undo() {
    let limits = Limits::default();
    let cylinder = |radius_mm| {
        om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Cylinder {
                radius_mm,
                height_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap()
    };
    let ring = om9_kernel::editing::boolean(
        &cylinder(5.0),
        &cylinder(2.0),
        om9_kernel::editing::BooleanOp::Difference,
        0.01,
        &limits,
    )
    .unwrap()
    .body()
    .unwrap()
    .clone();
    let top = (0..ring.display().faces.len())
        .find(|&i| {
            let vertices: Vec<_> = ring
                .display()
                .triangles
                .iter()
                .filter(|triangle| triangle[3] as usize == i)
                .flat_map(|triangle| {
                    triangle[..3]
                        .iter()
                        .map(|index| ring.display().points[*index as usize][2])
                })
                .collect();
            !vertices.is_empty() && vertices.iter().all(|z| (*z - 10.0).abs() < 1e-9)
        })
        .expect("top annular face");
    let shell = om9_kernel::editing::remove_face(&ring, top, 0.01, &limits).unwrap();
    let mut doc = Document::new("Cap annulus");
    let entity = om9_core::model::Entity::new("open tube", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        measure.closed
            && measure.naked_edges == 0
            && (measure.volume_mm3 - 210.0 * std::f64::consts::PI).abs() < 1e-5
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn cap_planar_partial_shell_through_command_and_undo() {
    use om9_kernel::{creation, editing};
    let limits = Limits::default();
    let box_at = |x: f64, height: f64| {
        let profile = creation::Profile::new(vec![
            [x, 0.0, 0.0],
            [x + 10.0, 0.0, 0.0],
            [x + 10.0, 10.0, 0.0],
            [x, 10.0, 0.0],
        ])
        .unwrap();
        creation::extrude(&profile, [0.0, 0.0, height], true, 0.01, &limits).unwrap()
    };
    let mut shell = editing::boolean(
        &box_at(0.0, 10.0),
        &box_at(10.0, 5.0),
        editing::BooleanOp::Union,
        0.01,
        &limits,
    )
    .unwrap()
    .body()
    .unwrap()
    .clone();
    let vertices = |brep: &om9_core::brep::Brep, face: usize| -> Vec<[f64; 3]> {
        brep.display()
            .triangles
            .iter()
            .filter(|t| t[3] as usize == face)
            .flat_map(|t| t[..3].iter().map(|i| brep.display().points[*i as usize]))
            .collect()
    };
    for z in [10.0, 5.0] {
        while let Some(face) = (0..shell.display().faces.len()).find(|&i| {
            let points = vertices(&shell, i);
            !points.is_empty() && points.iter().all(|p| (p[2] - z).abs() < 1e-9)
        }) {
            shell = editing::remove_face(&shell, face, 0.01, &limits).unwrap();
        }
    }
    let step = (0..shell.display().faces.len())
        .find(|&i| {
            let points = vertices(&shell, i);
            !points.is_empty()
                && points.iter().all(|p| (p[0] - 10.0).abs() < 1e-9)
                && points.iter().any(|p| (p[2] - 5.0).abs() < 1e-9)
                && points.iter().any(|p| (p[2] - 10.0).abs() < 1e-9)
        })
        .unwrap();
    shell = editing::remove_face(&shell, step, 0.01, &limits).unwrap();
    while let Some(face) = (0..shell.display().faces.len()).find(|&i| {
        let points = vertices(&shell, i);
        !points.is_empty() && points.iter().all(|p| p[2].abs() < 1e-9)
    }) {
        shell = editing::remove_face(&shell, face, 0.01, &limits).unwrap();
    }
    let before_measure = creation::measure(&shell, &limits).unwrap();
    let mut doc = Document::new("Partial cap");
    let entity = om9_core::model::Entity::new("open stepped shell", Geometry::Brep(shell), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Cap").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    c.submit("").unwrap();
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let after = creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(
        !after.closed && after.naked_edges > 0 && after.naked_edges < before_measure.naked_edges
    );
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn extract_surface_separates_face_in_one_undoable_transaction() {
    let mut c = Controller::new(Document::new("Extract"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo_before = c.coordinator.undo_len();
    c.submit("ExtractSrf 0").unwrap();
    let Geometry::Brep(remainder) = &c.document().entities[&id].geometry else {
        panic!("shell")
    };
    let measure = om9_kernel::creation::measure(remainder, c.coordinator().limits()).unwrap();
    assert!(!measure.closed);
    assert_eq!(measure.faces, 5);
    assert_eq!(c.document().entities.len(), 2);
    assert_eq!(c.coordinator.undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn extract_surface_copy_preserves_source_and_placement() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Extract copy");
    let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
    entity.placement = om9_core::geometry::Transform::translation([20.0, 3.0, 2.0]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("ExtractSrf 0 Copy=Yes").unwrap();
    assert_eq!(c.document().entities[&id], before.entities[&id]);
    let extracted = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    assert_eq!(extracted.placement, before.entities[&id].placement);
    let Geometry::Brep(face) = &extracted.geometry else {
        panic!("face")
    };
    assert_eq!(
        om9_kernel::creation::measure(face, c.coordinator().limits())
            .unwrap()
            .faces,
        1
    );
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn extract_surface_failure_leaves_document_and_redo_intact() {
    let mut c = Controller::new(Document::new("Extract failure"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let undo_before = c.coordinator.undo_len();
    let redo_before = c.coordinator.redo_len();
    assert!(c.submit("ExtractSrf 999").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), undo_before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
}

#[test]
fn extract_surface_interactive_options_choose_output_layer() {
    use om9_core::model::Layer;
    for input_layer in [false, true] {
        let limits = Limits::default();
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut doc = Document::new("Extract layers");
        let source_layer = Layer::new("Source", [255, 0, 0, 255]);
        let current_layer = Layer::new("Current", [0, 255, 0, 255]);
        let source_id = source_layer.id;
        let current_id = current_layer.id;
        doc.layers.insert(source_id, source_layer);
        doc.layers.insert(current_id, current_layer);
        doc.layer_order.extend([source_id, current_id]);
        doc.current_layer = Some(current_id);
        let entity = om9_core::model::Entity::new("box", Geometry::Brep(body), Some(source_id));
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        c.submit("ExtractSrf").unwrap();
        c.submit("Copy=Yes").unwrap();
        if input_layer {
            c.submit("OutputLayer=Input").unwrap();
        }
        c.submit("0").unwrap();
        let extracted = c.document().entities.values().find(|e| e.id != id).unwrap();
        assert_eq!(
            extracted.attributes.layer,
            Some(if input_layer { source_id } else { current_id })
        );
        assert_eq!(c.document().entities[&id], before.entities[&id]);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn extract_surface_multiple_faces_and_sources_commit_atomically() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    for copy in [false, true] {
        let mut doc = Document::new("multi extract");
        let mut ids = Vec::new();
        for x in [0.0, 20.0] {
            let mut entity =
                om9_core::model::Entity::new("box", Geometry::Brep(shape.clone()), None);
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.submit("ExtractSrf").unwrap();
        if copy {
            c.submit("Copy=Yes").unwrap();
        }
        for (id, index) in [(ids[0], 0), (ids[0], 2), (ids[1], 1)] {
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            let face = brep.display().faces[index];
            let object = c
                .scene()
                .unwrap()
                .objects
                .iter()
                .find(|o| o.root == id && o.path.is_empty())
                .unwrap();
            let reference = super::SelectedSubobject {
                root: id,
                path: vec![],
                subset: om9_scene::scene::Subelement::BrepFace(face),
                source: object.source,
                world_epoch: c.scene().unwrap().epoch,
            };
            c.solid_extract_pick_face(reference).unwrap();
        }
        assert_eq!(c.subselection().len(), 3);
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert_eq!(c.document().entities.len(), 5);
        assert_eq!(c.coordinator.undo_len(), 1);
        for (id, expected) in [
            (ids[0], if copy { 6 } else { 4 }),
            (ids[1], if copy { 6 } else { 5 }),
        ] {
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            assert_eq!(
                om9_kernel::creation::measure(brep, c.coordinator().limits())
                    .unwrap()
                    .faces,
                expected
            );
        }
        assert_eq!(
            c.document()
                .entities
                .values()
                .filter(|e| !ids.contains(&e.id))
                .count(),
            3
        );
        let mut positions: Vec<_> = c
            .document()
            .entities
            .values()
            .filter(|e| !ids.contains(&e.id))
            .map(|e| e.placement.0[0][3])
            .collect();
        positions.sort_by(f64::total_cmp);
        assert_eq!(positions, vec![0.0, 0.0, 20.0]);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 5);
    }
}

#[test]
fn extract_surface_all_faces_removes_source_once_and_undo_restores_it() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("extract all faces");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("ExtractSrf").unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let faces = brep.display().faces.clone();
    for face in faces {
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepFace(face),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_extract_pick_face(reference).unwrap();
    }
    c.submit("").unwrap();
    assert!(!c.document().entities.contains_key(&id));
    assert_eq!(c.document().entities.len(), 6);
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn shell_all_faces_failure_can_be_revised_or_canceled() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let keys = shape.display().faces.clone();
    let mut doc = Document::new("Shell invalid selection");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Shell").unwrap();
    c.submit("Thickness=1").unwrap();
    for key in keys {
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_extract_pick_face(reference).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Undo").unwrap();
    assert!(c.solid_face_needs_pick());
    c.submit("Undo").unwrap();
    assert_eq!(c.subselection().len(), 5);
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    assert!(c.subselection().is_empty());
}

#[test]
fn shell_stages_two_faces_and_commits_atomically() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let keys = [body.display().faces[0], body.display().faces[1]];
    let mut display = body.display().clone();
    display.faces.swap(0, 2);
    for entry in &mut display.face_edges { if entry[0] == 0 { entry[0] = 2; } else if entry[0] == 2 { entry[0] = 0; } }
    for entry in &mut display.triangles { if entry[3] == 0 { entry[3] = 2; } else if entry[3] == 2 { entry[3] = 0; } }
    let body = om9_core::brep::Brep::from_native(body.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("multi-face shell");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-shell-multi-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    let before = c.document().clone();
    c.submit("Shell").unwrap();
    assert!(c.submit("").is_err());
    assert!(c.submit("Thickness=0").is_err());
    for key in keys {
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_extract_pick_face(reference).unwrap();
    }
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert!(!c.solid_face_needs_pick());
    assert!(c.submit("0").is_err());
    assert_eq!(c.document(), &before);
    c.submit("1").unwrap();
    let Geometry::Brep(shell) = &c.document().entities[&id].geometry else { panic!() };
    let measure = om9_kernel::creation::measure(shell, c.coordinator().limits()).unwrap();
    assert!(measure.closed && (measure.volume_mm3 - 360.0).abs() < 1e-5, "{}", measure.volume_mm3);
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
}

#[test]
fn fillet_closed_edge_has_one_handle_and_can_commit() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder { radius_mm: 5.0, height_mm: 10.0 },
        0.01,
        &limits,
    ).unwrap();
    let index = (0..shape.display().edges.len()).find(|&i|
        om9_kernel::editing::oriented_edge_endpoints_or_closed(&shape, i, &limits).unwrap().is_none()
    ).unwrap();
    let edge = shape.display().edges[index];
    let mut doc = Document::new("closed fillet edge");
    let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1").unwrap();
    let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
    let reference = super::SelectedSubobject {
        root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge),
        source: object.source, world_epoch: c.scene().unwrap().epoch,
    };
    c.solid_fillet_pick_edge(reference).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.solid_fillet_handles().len(), 1);
    assert!(c.solid_fillet_show_radius());
    c.submit("ShowRadius=No").unwrap();
    assert!(!c.solid_fillet_show_radius());
    assert_eq!(c.solid_fillet_handles().len(), 1);
    assert!(c.submit("ShowRadius=invalid").is_err());
    assert!(!c.solid_fillet_show_radius());
    c.submit("ShowRadius=Yes").unwrap();
    assert!(c.solid_fillet_show_radius());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_variable_radius_on_selected_tangent_edges() {
    let limits = Limits::default();
    let profile = om9_kernel::creation::Profile::new(vec![
        [0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0], [0.0, 10.0, 0.0],
    ]).unwrap();
    let shape = om9_kernel::creation::extrude(&profile, [0.0, 0.0, 10.0], true, 0.01, &limits).unwrap();
    let selected: Vec<_> = (0..shape.display().edges.len()).filter_map(|index| {
        let (a, b) = om9_kernel::editing::oriented_edge_endpoints(&shape, index, &limits).ok()?;
        if a[1].abs() < 1e-6 && b[1].abs() < 1e-6 && (a[2]-10.0).abs() < 1e-6
            && (b[2]-10.0).abs() < 1e-6 && ((a[0]-b[0]).abs()-5.0).abs() < 1e-6 {
            Some((index, a, b))
        } else { None }
    }).collect();
    assert_eq!(selected.len(), 2);
    let selected_keys = [shape.topology().edges[selected[0].0], shape.topology().edges[selected[1].0]];
    let swap = (0..shape.topology().edges.len()).find(|i| selected.iter().all(|item| item.0 != *i)).unwrap();
    let mut display = shape.display().clone();
    display.edges.swap(selected[0].0, swap);
    for entry in &mut display.face_edges {
        if entry[1] as usize == selected[0].0 { entry[1] = swap as u32; }
        else if entry[1] as usize == swap { entry[1] = selected[0].0 as u32; }
    }
    for entry in &mut display.segments {
        if entry[2] as usize == selected[0].0 { entry[2] = swap as u32; }
        else if entry[2] as usize == swap { entry[2] = selected[0].0 as u32; }
    }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut doc = Document::new("tangent fillet");
    let entity = om9_core::model::Entity::new("split prism", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fillet-law-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1").unwrap();
    for edge in selected_keys {
        let object = c.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge), source: object.source, world_epoch: c.scene().unwrap().epoch };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    c.submit("").unwrap();
    let outer = if (selected[0].1[0] - 5.0).abs() > 1e-6 { 0 } else { 1 };
    let shared = 1 - outer;
    let other_shared = 2 + if (selected[1].1[0] - 5.0).abs() < 1e-6 { 0 } else { 1 };
    assert!(c.solid_fillet_select_hit(&[(outer, 0.0), (shared, 0.1)]).is_err());
    assert_eq!(c.solid_fillet_current_radius(), None);
    c.solid_fillet_select_hit(&[(shared, 0.0), (other_shared, 0.1)]).unwrap();
    assert_eq!(c.solid_fillet_current_radius(), Some(1.0));
    c.solid_fillet_select_handle(outer).unwrap();
    c.submit("CurrentRadius=1.5").unwrap();
    c.solid_fillet_select_handle(shared).unwrap();
    c.submit("CurrentRadius=1.8").unwrap();
    let handles = c.solid_fillet_handles();
    let shared_radii: Vec<_> = handles.iter().filter(|item| (item.1[0]-5.0).abs() < 1e-6).map(|item| item.2).collect();
    assert_eq!(shared_radii, vec![1.8, 1.8]);
    let before_failure = c.solid_fillet_preview().unwrap().0.clone();
    assert!(c.submit("CurrentRadius=1000").is_err());
    assert_eq!(c.solid_fillet_current_radius(), Some(1.8));
    let shared_radii: Vec<_> = c.solid_fillet_handles().iter().filter(|item| (item.1[0]-5.0).abs() < 1e-6).map(|item| item.2).collect();
    assert_eq!(shared_radii, vec![1.8, 1.8]);
    assert_eq!(c.solid_fillet_preview().unwrap().0, &before_failure);
    assert_eq!(c.document(), &before);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_edge_stages_multiple_edges_preview_and_commit() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("staged fillet");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    assert!(c.submit("NextRadius=0").is_err());
    assert!(c.submit("").is_err());
    c.submit("NextRadius=1").unwrap();
    for index in [0, 2] {
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let edge = brep.display().edges[index];
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepEdge(edge),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    assert_eq!(c.document(), &before);
    c.submit("NextRadius=2").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document(), &before);
    assert!(c.solid_fillet_preview().is_some());
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Undo").unwrap();
    assert!(c.solid_fillet_preview().is_none());
    assert_eq!(c.document(), &before);
    c.submit("").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
    assert!(measure.closed && measure.volume_mm3 < 1000.0 && measure.volume_mm3 > 900.0);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_edge_captures_next_radius_for_each_selected_edge() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let vertical: Vec<_> = (0..shape.display().edges.len())
        .filter(|&index| {
            om9_kernel::editing::edge_endpoints(&shape, index).is_ok_and(|(a, b)| {
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() > 9.0
            })
        })
        .collect();
    assert!(vertical.len() >= 2);
    let mut doc = Document::new("different fillet radii");
    let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("FilletEdge").unwrap();
    for (index, radius) in [(vertical[0], 1.0), (vertical[1], 2.0)] {
        c.submit(&format!("NextRadius={radius}")).unwrap();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let edge = brep.display().edges[index];
        let object = c
            .scene()
            .unwrap()
            .objects
            .iter()
            .find(|o| o.root == id && o.path.is_empty())
            .unwrap();
        let reference = super::SelectedSubobject {
            root: id,
            path: vec![],
            subset: om9_scene::scene::Subelement::BrepEdge(edge),
            source: object.source,
            world_epoch: c.scene().unwrap().epoch,
        };
        c.solid_fillet_pick_edge(reference).unwrap();
    }
    c.submit("").unwrap();
    let (preview, _) = c.solid_fillet_preview().unwrap();
    let m = om9_kernel::creation::measure(preview, c.coordinator().limits()).unwrap();
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 50.0;
    assert!((m.volume_mm3 - expected).abs() < 1e-5, "{}", m.volume_mm3);
    assert_eq!(c.document(), &before);
    c.submit("SetAll=1.5").unwrap();
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("").unwrap();
    assert_eq!(c.coordinator.undo_len(), 1);
    let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!() };
    let volume = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap().volume_mm3;
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 45.0;
    assert!((volume - expected).abs() < 1e-5);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn fillet_invalid_preview_keeps_document_and_redo() {
    let mut c = Controller::new(Document::new("invalid fillet"), Limits::default()).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Box 2 2 2").unwrap();
    c.undo().unwrap();
    let before = c.document().clone();
    let redo_before = c.coordinator.redo_len();
    c.submit("FilletEdge").unwrap();
    c.submit("NextRadius=1000").unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!()
    };
    let edge = brep.display().edges[0];
    let object = c
        .scene()
        .unwrap()
        .objects
        .iter()
        .find(|o| o.root == id && o.path.is_empty())
        .unwrap();
    let reference = super::SelectedSubobject {
        root: id,
        path: vec![],
        subset: om9_scene::scene::Subelement::BrepEdge(edge),
        source: object.source,
        world_epoch: c.scene().unwrap().epoch,
    };
    c.solid_fillet_pick_edge(reference).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
    assert!(c.is_running());
    c.submit("Undo").unwrap();
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.redo_len(), redo_before);
}

#[test]
fn fillet_open_joined_shell_staged_and_numeric_paths() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let top = (0..body.display().faces.len())
        .find(|&index| {
            let points: Vec<_> = body
                .display()
                .triangles
                .iter()
                .filter(|t| t[3] as usize == index)
                .flat_map(|t| t[..3].iter().map(|i| body.display().points[*i as usize][2]))
                .collect();
            !points.is_empty() && points.iter().all(|z| (*z - 10.0).abs() < 1e-9)
        })
        .unwrap();
    let shell = om9_kernel::editing::remove_face(&body, top, 0.01, &limits).unwrap();
    let edge = (0..shell.display().edges.len())
        .find(|&i| {
            om9_kernel::editing::edge_endpoints(&shell, i).is_ok_and(|(a, b)| {
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() > 9.0
            })
        })
        .unwrap();
    for staged in [false, true] {
        let mut doc = Document::new("Open shell fillet");
        let entity = om9_core::model::Entity::new("shell", Geometry::Brep(shell.clone()), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        if staged {
            c.submit("FilletEdge").unwrap();
            c.submit("NextRadius=1").unwrap();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
                panic!()
            };
            let object = c
                .scene()
                .unwrap()
                .objects
                .iter()
                .find(|o| o.root == id && o.path.is_empty())
                .unwrap();
            let reference = super::SelectedSubobject {
                root: id,
                path: vec![],
                subset: om9_scene::scene::Subelement::BrepEdge(brep.display().edges[edge]),
                source: object.source,
                world_epoch: c.scene().unwrap().epoch,
            };
            c.solid_fillet_pick_edge(reference).unwrap();
            c.submit("").unwrap();
            assert_eq!(c.document(), &before);
            c.submit("").unwrap();
        } else {
            c.select_objects(&[id]).unwrap();
            c.submit(&format!("FilletEdge {edge} 1")).unwrap();
        }
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else {
            panic!()
        };
        let measure = om9_kernel::creation::measure(result, c.coordinator().limits()).unwrap();
        let expected =
            500.0 - 20.0 + 5.0 * std::f64::consts::PI - (1.0 - std::f64::consts::PI / 4.0);
        assert!(!measure.closed && (measure.area_mm2 - expected).abs() < 1e-5);
        assert_eq!(c.coordinator.undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn boolean_two_objects_selection_undo_and_retry_do_not_commit() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    assert!(c.solid_boolean_needs_object());
    assert!(c.submit("U").is_err());
    c.solid_boolean_pick(ids[0], false).unwrap();
    c.solid_boolean_pick(ids[1], false).unwrap();
    assert!(c.solid_boolean_pick(ids[2], false).is_err());
    assert_eq!(c.selected_ordered(), vec![ids[0], ids[1]]);
    c.submit("Undo").unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.solid_boolean_pick(ids[2], false).unwrap();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn boolean_two_reopened_sphere_profiles_have_same_analytic_results() {
    use om9_kernel::{brep, creation};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let box_body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let sphere_source = sphere.source().to_owned();
    let mut triangle_counts = Vec::new();
    for (profile, menu, mode) in [(0.5, true, 0), (0.01, false, 0), (0.5, false, 1), (0.01, true, 1), (0.5, false, 2), (0.01, true, 2)] {
        let sphere = brep::read_brep(&sphere_source, profile, &limits).unwrap();
        triangle_counts.push(sphere.display().triangles.len());
        let mut document = Document::new("Boolean two profiles");
        let a = om9_core::model::Entity::new("sphere", Geometry::Brep(sphere), None);
        let box_source = match mode {
            1 => creation::primitive(creation::Primitive::Box { x_mm: 8., y_mm: 4., z_mm: 8. }, 0.01, &limits).unwrap(),
            2 => creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap(),
            _ => box_body.clone(),
        };
        let mut b = om9_core::model::Entity::new("box", Geometry::Brep(box_source), None);
        b.placement = match mode {
            1 => om9_core::geometry::Transform([[0., -1., 0., 4.], [1., 0., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]),
            2 => om9_core::geometry::Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]),
            _ => om9_core::geometry::Transform::translation([0., -4., -4.]),
        };
        let ids = [a.id, b.id];
        document.entities.insert(a.id, a); document.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-boolean2-profile-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(path).unwrap();
        for (id, expected) in ids.into_iter().zip([[-3., -3., -3., 3., 3., 3.], [0., -4., -4., 4., 4., 4.]]) {
            let entity = &c.document().entities[&id];
            let Geometry::Brep(shape) = &entity.geometry else { panic!("input") };
            let bounds = brep::world_bounds(shape, &entity.placement, &limits).unwrap();
            for (actual, want) in bounds.into_iter().zip(expected) { assert!((actual - want).abs() < 1e-6); }
        }
        if mode == 2 {
            c.submit("Box 1 1 1").unwrap();
            c.undo().unwrap();
            assert_eq!(c.coordinator.redo_len(), 1);
        }
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        if menu { c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Boolean2Objects").unwrap(); }
        if mode == 2 {
            assert!(matches!(c.submit(""), Err(om9_core::Error::Unsupported(_))));
            assert!(c.solid_boolean_two_preview().is_none());
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator.redo_len(), 1);
            c.cancel();
            continue;
        }
        c.submit("").unwrap();
        let half = 18. * std::f64::consts::PI;
        for (name, expected) in [("Union", 256. + half), ("Difference", half),
            ("ReverseDifference", 256. - half), ("Intersection", half), ("InversionIntersection", 256.)] {
            let (body, actual_name) = c.solid_boolean_two_preview().unwrap();
            assert_eq!(actual_name, name);
            let actual = creation::measure(body, &limits).unwrap().volume_mm3;
            assert!((actual - expected).abs() < 1e-5, "{name}: {actual} vs {expected}");
            assert_eq!(c.document(), &before);
            c.submit("Next").unwrap();
        }
        c.submit("").unwrap();
        assert_eq!(c.coordinator.undo_len(), 1);
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
    assert_ne!(triangle_counts[0], triangle_counts[1]);
}

#[test]
fn boolean_union_certified_baked_box_is_correct_and_undoable() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let affine = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &affine, 0.01, &limits).unwrap();
    for (profile, two_objects) in [(0.5, false), (0.01, false), (0.5, true), (0.01, true)] {
        let mut document = Document::new("baked affine Boolean");
        let a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
        let b = om9_core::model::Entity::new("baked box", Geometry::Brep(block.clone()), None);
        let ids = [a.id, b.id];
        document.entities.insert(a.id, a);
        document.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-baked-boolean-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap();
        c.undo().unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        if two_objects {
            let preview = c.boolean_two_build_preview(0).unwrap();
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator.redo_len(), 1);
            c.boolean_two_commit(&preview, true).unwrap();
        } else {
            c.submit("BooleanUnion").unwrap();
            c.submit("").unwrap();
        }
        let result = c.document().entities.values().find(|entity| !ids.contains(&entity.id)).expect("union result");
        let Geometry::Brep(body) = &result.geometry else { panic!("union result is not a B-rep") };
        let measured = creation::measure(body, &limits).unwrap();
        assert!(measured.closed && measured.solids == 1 && measured.naked_edges == 0, "{measured:?}");
        let volume = measured.volume_mm3;
        let expected = 256. + 18. * std::f64::consts::PI;
        assert!((volume - expected).abs() < 1e-5, "profile {profile}: volume {volume}, expected {expected}");
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn boolean_union_reopened_placed_compound_keeps_all_solids_and_undo_redo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{creation, editing, transform_ops};
    let limits = Limits::default();
    let cube = creation::primitive(creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.01, &limits).unwrap();
    let second = transform_ops::transform_brep(&cube, &Transform::translation([3., 0., 0.]), 0.01, &limits).unwrap();
    let compound = editing::boolean(&cube, &second, editing::BooleanOp::Union, 0.01, &limits).unwrap().body().unwrap().clone();
    let mut a = om9_core::model::Entity::new("compound", Geometry::Brep(compound), None);
    a.placement = Transform::translation([5., 0., 0.]);
    let mut b = om9_core::model::Entity::new("other", Geometry::Brep(cube), None);
    b.placement = Transform::translation([12., 0., 0.]);
    let ids = [a.id, b.id];
    let mut document = Document::new("placed compound Union");
    document.entities.insert(a.id, a); document.entities.insert(b.id, b);
    let path = std::env::temp_dir().join(format!("om9-placed-compound-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap(); c.submit("").unwrap();
    let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).expect("union result").1;
    let Geometry::Brep(body) = &output.geometry else { panic!("union must produce a B-rep") };
    let measure = creation::measure(body, &limits).unwrap();
    assert_eq!(measure.solids, 3);
    assert!((measure.volume_mm3 - 3.).abs() < 1e-6);
    assert!((measure.bounds_mm[0] - 5.).abs() < 1e-6 && (measure.bounds_mm[3] - 13.).abs() < 1e-6);
    let after = c.document().clone();
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn boolean_two_relative_rotation_curved_modes_commit_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let box_body = creation::primitive(creation::Primitive::Box { x_mm: 8., y_mm: 4., z_mm: 8. }, 0.01, &limits).unwrap();
    let relative = Transform([[0., -1., 0., 4.], [1., 0., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let half = 18. * std::f64::consts::PI;
    let expected = [("Union", 256. + half), ("Difference", half), ("ReverseDifference", 256. - half), ("Intersection", half), ("InversionIntersection", 256.)];
    for base in [
        Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
    ] {
      for profile in [0.5, 0.01] {
        for (index, (mode, volume)) in expected.iter().copied().enumerate() {
            let mut document = Document::new("curved Boolean relative rotation");
            let mut a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
            let mut b = om9_core::model::Entity::new("box", Geometry::Brep(box_body.clone()), None);
            a.placement = base; b.placement = base.compose(relative).unwrap();
            let ids = [a.id, b.id]; document.entities.insert(a.id, a); document.entities.insert(b.id, b);
            let path = std::env::temp_dir().join(format!("om9-curved-relative-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
            c.select_objects(&ids).unwrap(); let before = c.document().clone();
            c.submit("Boolean2Objects").unwrap(); c.submit("").unwrap();
            for _ in 0..index { c.submit("Next").unwrap(); }
            assert_eq!(c.solid_boolean_two_preview().unwrap().1, mode);
            assert_eq!(c.solid_boolean_two_preview_placement(), Some(base));
            c.submit("").unwrap();
            let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).unwrap().1;
            assert_eq!(output.placement, base);
            let Geometry::Brep(body) = &output.geometry else { panic!("B-rep output") };
            let actual = creation::measure(body, &limits).unwrap().volume_mm3;
            assert!((actual - volume).abs() < 1e-5, "{mode}: {actual} vs {volume}");
            let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
      }
    }
}

#[test]
fn boolean_common_affine_with_relative_rotation_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation};
    let limits = Limits::default();
    let body = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    let relative = Transform([[0., -1., 0., 3.], [1., 0., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    for base in [
        Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
    ] {
      for profile in [0.5, 0.01] {
        for (command, feature) in [("BooleanUnion", "OM9-SOLID-003"), ("Boolean2Objects", "OM9-SOLID-004"), ("BooleanDifference", "OM9-SOLID-001"), ("BooleanIntersection", "OM9-SOLID-002"), ("BooleanSplit", "SolidBooleanSplit")] {
            let mut doc = Document::new("rotated shared affine");
            let mut a = om9_core::model::Entity::new("target", Geometry::Brep(brep::read_brep(body.source(), profile, &limits).unwrap()), None);
            let mut b = om9_core::model::Entity::new("tool", Geometry::Brep(body.clone()), None);
            a.placement = base; b.placement = base.compose(relative).unwrap();
            let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
            let path = std::env::temp_dir().join(format!("om9-relative-rotation-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
            let both = matches!(command, "BooleanUnion" | "Boolean2Objects");
            c.select_objects(if both { &ids[..] } else { &ids[..1] }).unwrap(); let before = c.document().clone();
            if profile == 0.5 { c.dispatch(feature, InvocationOrigin::Menu).unwrap(); } else { c.submit(command).unwrap(); }
            c.submit("").unwrap();
            if !both {
                if command == "BooleanDifference" { c.solid_difference_pick(ids[1], false).unwrap(); }
                else { c.solid_boolean_pick(ids[1], false).unwrap(); }
                c.submit("").unwrap();
            } else if command == "Boolean2Objects" { c.submit("").unwrap(); }
            let mut count = 0; let mut volume = 0.;
            for (id, entity) in &c.document().entities {
                if ids.contains(id) { continue; }
                assert_eq!(entity.placement, base, "{command}");
                let Geometry::Brep(shape) = &entity.geometry else { panic!("B-rep output") };
                volume += creation::measure(shape, &limits).unwrap().volume_mm3; count += 1;
            }
            let (expected_count, expected_volume) = match command {
                "BooleanUnion" | "Boolean2Objects" => (1, 12.),
                "BooleanDifference" | "BooleanIntersection" => (1, 4.),
                _ => (2, 8.),
            };
            assert_eq!(count, expected_count, "{command}"); assert!((volume - expected_volume).abs() < 1e-6, "{command}: {volume}");
            let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
      }
    }
}

#[test]
fn boolean_shared_linear_with_different_translations_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation};
    let limits = Limits::default();
    let body = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    for base in [
        Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
    ] {
      for profile in [0.5, 0.01] {
        for (command, feature) in [("BooleanUnion", "OM9-SOLID-003"), ("Boolean2Objects", "OM9-SOLID-004"), ("BooleanDifference", "OM9-SOLID-001"), ("BooleanIntersection", "OM9-SOLID-002"), ("BooleanSplit", "SolidBooleanSplit")] {
            let mut other = base;
            let translated = base.point([1., 0.5, 0.]).unwrap();
            for i in 0..3 { other.0[i][3] = translated[i]; }
            let mut doc = Document::new("translated shared affine");
            let mut a = om9_core::model::Entity::new("target", Geometry::Brep(brep::read_brep(body.source(), profile, &limits).unwrap()), None);
            let mut b = om9_core::model::Entity::new("tool", Geometry::Brep(body.clone()), None);
            a.placement = base; b.placement = other;
            let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
            let path = std::env::temp_dir().join(format!("om9-shared-linear-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
            let both = matches!(command, "BooleanUnion" | "Boolean2Objects");
            c.select_objects(if both { &ids[..] } else { &ids[..1] }).unwrap();
            let before = c.document().clone();
            if profile == 0.5 { c.dispatch(feature, InvocationOrigin::Menu).unwrap(); } else { c.submit(command).unwrap(); }
            c.submit("").unwrap();
            if !both {
                if command == "BooleanDifference" { c.solid_difference_pick(ids[1], false).unwrap(); }
                else { c.solid_boolean_pick(ids[1], false).unwrap(); }
                c.submit("").unwrap();
            } else if command == "Boolean2Objects" { c.submit("").unwrap(); }
            let mut count = 0; let mut volume = 0.;
            for (id, entity) in &c.document().entities {
                if ids.contains(id) { continue; }
                assert_eq!(entity.placement, base, "{command}");
                let Geometry::Brep(shape) = &entity.geometry else { panic!("B-rep output") };
                volume += creation::measure(shape, &limits).unwrap().volume_mm3; count += 1;
            }
            let (expected_count, expected_volume) = match command {
                "BooleanUnion" | "Boolean2Objects" => (1, 13.),
                "BooleanDifference" => (1, 5.),
                "BooleanIntersection" => (1, 3.),
                _ => (2, 8.),
            };
            assert_eq!(count, expected_count, "{command}"); assert!((volume - expected_volume).abs() < 1e-6, "{command}: {volume}");
            let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
      }
    }
}

#[test]
fn grouped_boolean_different_affine_placements_preserve_redo_on_refusal() {
    let limits = Limits::default();
    for command in ["BooleanDifference", "BooleanIntersection", "BooleanSplit"] {
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
        let mut doc = Document::new("different Boolean placements");
        let mut a = om9_core::model::Entity::new("target", Geometry::Brep(body.clone()), None);
        a.placement = om9_core::geometry::Transform([[2., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
        let b = om9_core::model::Entity::new("tool", Geometry::Brep(body), None);
        let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let mut c = Controller::new(doc, limits).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
        c.select_objects(&ids[..1]).unwrap(); let before = c.document().clone();
        c.submit(command).unwrap(); c.submit("").unwrap();
        if command == "BooleanDifference" { c.solid_difference_pick(ids[1], false).unwrap(); }
        else { c.solid_boolean_pick(ids[1], false).unwrap(); }
        assert!(matches!(c.submit(""), Err(om9_core::Error::Unsupported(_))), "{command}");
        assert_eq!(c.document(), &before, "{command}"); assert_eq!(c.coordinator.redo_len(), 1, "{command}");
    }
}

#[test]
fn difference_common_affine_multiple_targets_and_cutters() {
    use om9_core::geometry::Transform;
    use om9_kernel::{creation, transform_ops};
    let limits = Limits::default();
    let target = creation::primitive(creation::Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }, 0.01, &limits).unwrap();
    let cutter = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]);
    for (second_tool, expected) in [([21., 1., 1.], 1984.), ([2., 1., 1.], 1988.)] {
        let local = [target.clone(), transform_ops::transform_brep(&target, &Transform::translation([20., 0., 0.]), 0.01, &limits).unwrap(),
            transform_ops::transform_brep(&cutter, &Transform::translation([1., 1., 1.]), 0.01, &limits).unwrap(),
            transform_ops::transform_brep(&cutter, &Transform::translation(second_tool), 0.01, &limits).unwrap()];
        let mut doc = Document::new("common affine multiple difference");
        let mut ids = Vec::new();
        for body in local { let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(body), None); entity.placement = placement; ids.push(entity.id); doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids[..2]).unwrap(); let before = c.document().clone();
        c.submit("BooleanDifference").unwrap(); c.submit("").unwrap();
        c.solid_difference_pick(ids[2], false).unwrap(); c.solid_difference_pick(ids[3], false).unwrap(); c.submit("").unwrap();
        let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).unwrap().1;
        assert_eq!(output.placement, placement);
        let Geometry::Brep(body) = &output.geometry else { panic!("B-rep result") };
        assert!((creation::measure(body, &limits).unwrap().volume_mm3 - expected).abs() < 1e-6);
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn grouped_boolean_common_affine_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let cube = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    let shifted = transform_ops::transform_brep(&cube, &Transform::translation([1., 0., 0.]), 0.01, &limits).unwrap();
    let placements = [
        Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
        Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]),
    ];
    for placement in placements {
      for profile in [0.5, 0.01] {
        for (command, feature, pieces, total_volume) in [
            ("BooleanDifference", "OM9-SOLID-001", 1, 4.),
            ("BooleanIntersection", "OM9-SOLID-002", 1, 4.),
            ("BooleanSplit", "SolidBooleanSplit", 2, 8.),
        ] {
            let mut doc = Document::new("grouped shared affine");
            let mut a = om9_core::model::Entity::new("target", Geometry::Brep(brep::read_brep(cube.source(), profile, &limits).unwrap()), None);
            let mut b = om9_core::model::Entity::new("tool", Geometry::Brep(shifted.clone()), None);
            a.placement = placement; b.placement = placement;
            let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
            let path = std::env::temp_dir().join(format!("om9-grouped-affine-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
            c.select_objects(&ids[..1]).unwrap(); let before = c.document().clone();
            if profile == 0.5 { c.dispatch(feature, InvocationOrigin::Menu).unwrap(); }
            else { c.submit(command).unwrap(); }
            c.submit("").unwrap();
            if feature == "OM9-SOLID-001" { c.solid_difference_pick(ids[1], false).unwrap(); }
            else { c.solid_boolean_pick(ids[1], false).unwrap(); }
            c.submit("").unwrap();
            let mut outputs = 0; let mut volume = 0.;
            for (id, entity) in &c.document().entities {
                if ids.contains(id) { continue; }
                assert_eq!(entity.placement, placement, "{command}");
                let Geometry::Brep(body) = &entity.geometry else { panic!("B-rep result") };
                let measure = creation::measure(body, &limits).unwrap();
                volume += measure.volume_mm3; outputs += 1;
                let mut expected = [f64::INFINITY, f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
                for x in [measure.bounds_mm[0], measure.bounds_mm[3]] { for y in [measure.bounds_mm[1], measure.bounds_mm[4]] { for z in [measure.bounds_mm[2], measure.bounds_mm[5]] {
                    let p = placement.point([x, y, z]).unwrap();
                    for i in 0..3 { expected[i] = expected[i].min(p[i]); expected[i + 3] = expected[i + 3].max(p[i]); }
                } } }
                for (actual, expected) in brep::world_bounds(body, &placement, &limits).unwrap().into_iter().zip(expected) {
                    assert!((actual - expected).abs() < 1e-6, "{command}: world bound {actual} vs {expected}");
                }
            }
            assert_eq!(outputs, pieces, "{command}");
            assert!((volume - total_volume).abs() < 1e-6, "{command}: {volume}");
            let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
      }
    }
}

#[test]
fn boolean_two_common_affine_preview_and_commit_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    let shifted = transform_ops::transform_brep(&block, &Transform::translation([1., 0., 0.]), 0.01, &limits).unwrap();
    let cases = [
        (Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 2., [4., 2., -3., 10., 4., -1.]),
        (Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 1., [4., 2., -3., 8., 4., -1.]),
        (Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 1., [1., 2., -3., 4., 4., -1.]),
    ];
    for (placement, determinant, expected_bounds) in cases {
      for profile in [0.5, 0.01] {
        let mut doc = Document::new("Boolean Two common affine");
        let mut a = om9_core::model::Entity::new("first", Geometry::Brep(brep::read_brep(block.source(), profile, &limits).unwrap()), None);
        let mut b = om9_core::model::Entity::new("second", Geometry::Brep(shifted.clone()), None);
        a.placement = placement; b.placement = placement;
        let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-boolean-two-affine-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.select_objects(&ids).unwrap(); let before = c.document().clone();
        if profile == 0.5 { c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Boolean2Objects").unwrap(); }
        c.submit("").unwrap();
        for (mode, expected_volume) in [("Union", 12.), ("Difference", 4.), ("ReverseDifference", 4.), ("Intersection", 4.), ("InversionIntersection", 8.)] {
            let (body, actual_mode) = c.solid_boolean_two_preview().unwrap();
            assert_eq!(mode, actual_mode);
            assert!((creation::measure(body, &limits).unwrap().volume_mm3 - expected_volume).abs() < 1e-6);
            assert_eq!(c.solid_boolean_two_preview_placement(), Some(placement));
            assert_eq!(c.document(), &before);
            c.submit("Next").unwrap();
        }
        c.submit("").unwrap();
        let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).unwrap().1;
        assert_eq!(output.placement, placement);
        let Geometry::Brep(body) = &output.geometry else { panic!("B-rep result") };
        assert!((creation::measure(body, &limits).unwrap().volume_mm3 * determinant - 12. * determinant).abs() < 1e-6);
        let bounds = brep::world_bounds(body, &output.placement, &limits).unwrap();
        for (actual, expected) in bounds.into_iter().zip(expected_bounds) { assert!((actual - expected).abs() < 1e-6); }
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
      }
    }
}

#[test]
fn boolean_local_affine_result_reopen_reuse_refusal_preserves_redo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{creation, editing, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let box_body = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let sphere = transform_ops::transform_brep(&sphere, &transform_ops::inverse(&placement).unwrap(), 0.01, &limits).unwrap();
    let initial = editing::boolean_checked(&sphere, &box_body, editing::BooleanOp::Union, 0.01, &limits).unwrap().body().unwrap().clone();
    let mut a = om9_core::model::Entity::new("local result", Geometry::Brep(initial), None);
    let mut b = om9_core::model::Entity::new("local box", Geometry::Brep(box_body), None);
    a.placement = placement; b.placement = placement;
    let ids = [a.id, b.id]; let mut doc = Document::new("local result downstream");
    doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
    let path = std::env::temp_dir().join(format!("om9-local-result-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
    c.select_objects(&ids).unwrap(); let before = c.document().clone();
    c.submit("BooleanUnion").unwrap();
    let error = c.submit("").expect_err("current checked downstream operation must refuse");
    assert!(error.to_string().contains("boolean result failed OCCT validity checks"), "{error}");
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator.redo_len(), 1);
}

#[test]
fn boolean_union_common_affine_placement_after_reopen() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let cube = creation::primitive(creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.01, &limits).unwrap();
    let shifted = transform_ops::transform_brep(&cube, &Transform::translation([0.5, 0., 0.]), 0.01, &limits).unwrap();
    let cases = [
        (Transform([[2., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 2., [4., 2., -3., 7., 3., -2.]),
        (Transform([[1., 0.5, 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 1., [4., 2., -3., 6., 3., -2.]),
        (Transform([[-1., 0., 0., 4.], [0., 1., 0., 2.], [0., 0., 1., -3.], [0., 0., 0., 1.]]), 1., [2.5, 2., -3., 4., 3., -2.]),
    ];
    for (placement, determinant, expected_bounds) in cases {
      for (profile, menu) in [(0.5, true), (0.01, false)] {
        let mut document = Document::new("common affine Union");
        let mut a = om9_core::model::Entity::new("first", Geometry::Brep(brep::read_brep(cube.source(), profile, &limits).unwrap()), None);
        let mut b = om9_core::model::Entity::new("second", Geometry::Brep(shifted.clone()), None);
        a.placement = placement; b.placement = placement;
        let ids = [a.id, b.id];
        document.entities.insert(a.id, a); document.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-common-affine-union-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(&path).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        if menu { c.dispatch("OM9-SOLID-003", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("BooleanUnion").unwrap(); }
        c.submit("").unwrap();
        let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).expect("union result").1;
        let output_id = output.id;
        assert_eq!(output.placement, placement);
        let Geometry::Brep(body) = &output.geometry else { panic!("B-rep result") };
        let volume = creation::measure(body, &limits).unwrap().volume_mm3;
        assert!((volume * determinant - 1.5 * determinant).abs() < 1e-6, "world volume {}", volume * determinant);
        let bounds = brep::world_bounds(body, &output.placement, &limits).unwrap();
        for (actual, expected) in bounds.into_iter().zip(expected_bounds) {
            assert!((actual - expected).abs() < 1e-6, "world bound {actual}, expected {expected}");
        }
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
        om9_archive::Archive::save(&path, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut reopened = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(path).unwrap();
        reopened.select_objects(&[output_id, ids[0]]).unwrap();
        reopened.submit("BooleanUnion").unwrap(); reopened.submit("").unwrap();
        let chained = reopened.document().entities.iter()
            .find(|(id, _)| !ids.contains(id) && **id != output_id).expect("chained union").1;
        assert_eq!(chained.placement, placement);
        let Geometry::Brep(body) = &chained.geometry else { panic!("chained B-rep result") };
        assert!((creation::measure(body, &limits).unwrap().volume_mm3 - 1.5).abs() < 1e-6);
      }
    }
}

#[test]
fn boolean_union_accepts_computed_rigid_rotation() {
    let limits = Limits::default();
    let cube = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.01, &limits).unwrap();
    let mut document = Document::new("rotated Boolean");
    let a = om9_core::model::Entity::new("cube", Geometry::Brep(cube.clone()), None);
    let mut b = om9_core::model::Entity::new("rotated", Geometry::Brep(cube), None);
    b.placement = om9_kernel::transform_ops::rotation([0., 0., 0.], [0., 1., 0.], std::f64::consts::FRAC_PI_4).unwrap();
    b.placement.0[0][3] = 4.;
    let ids = [a.id, b.id];
    document.entities.insert(a.id, a); document.entities.insert(b.id, b);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("BooleanUnion").unwrap(); c.submit("").unwrap();
    let output = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).expect("union result").1;
    let Geometry::Brep(body) = &output.geometry else { panic!("B-rep expected") };
    assert!((om9_kernel::creation::measure(body, &limits).unwrap().volume_mm3 - 2.).abs() < 1e-6);
    let after = c.document().clone();
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn boolean_rejects_nonrigid_placements_without_losing_redo() {
    let limits = Limits::default();
    for placement in [
        om9_core::geometry::Transform([[2., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]),
        om9_core::geometry::Transform([[-1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]),
        om9_core::geometry::Transform([[1. + 4e-11, 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]),
        om9_core::geometry::Transform([[1., 1e-13, 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]),
    ] {
        for command in ["BooleanUnion", "Boolean2Objects"] {
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.1, &limits).unwrap();
            let mut document = Document::new("nonrigid Boolean");
            let a = om9_core::model::Entity::new("box", Geometry::Brep(body.clone()), None);
            let mut b = om9_core::model::Entity::new("placed box", Geometry::Brep(body), None);
            b.placement = placement;
            let ids = [a.id, b.id];
            document.entities.insert(a.id, a); document.entities.insert(b.id, b);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
            c.select_objects(&ids).unwrap();
            let before = c.document().clone();
            assert_eq!(c.coordinator.redo_len(), 1);
            c.submit(command).unwrap();
            assert!(matches!(c.submit(""), Err(om9_core::Error::Unsupported(_))), "{command}");
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator.redo_len(), 1);
            c.cancel();
        }
    }
}

#[test]
fn boolean_two_objects_preview_cycles_all_results_before_commit() {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean2 preview");
    let mut ids = Vec::new();
    for (width, x) in [(10.0, 0.0), (15.0, 5.0)] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: width,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
        entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    for (name, expected) in [
        ("Union", 2000.0),
        ("Difference", 500.0),
        ("ReverseDifference", 1000.0),
        ("Intersection", 500.0),
        ("InversionIntersection", 1500.0),
    ] {
        let (body, mode) = c.solid_boolean_two_preview().unwrap();
        assert_eq!(mode, name);
        let volume = om9_kernel::creation::measure(body, c.coordinator().limits())
            .unwrap()
            .volume_mm3;
        assert!((volume - expected).abs() < 1e-6, "{name}: {volume}");
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("Next").unwrap();
    }
    assert_eq!(c.solid_boolean_two_preview().unwrap().1, "Union");
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert!(ids.iter().all(|id| c.document().entities.contains_key(id)));
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn boolean_two_objects_stale_preview_rejects_cycle_and_commit() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    let source = c.document().stamp(ids[0]).unwrap();
    let prepared = c
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    c.coordinator.commit(prepared).unwrap();
    c.refresh_scene().unwrap();
    let before = c.document().clone();
    let undo = c.coordinator.undo_len();
    assert!(matches!(c.submit("Next"), Err(om9_core::Error::Stale(_))));
    assert!(matches!(c.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), undo);
    c.cancel();
    assert!(c.solid_boolean_two_preview().is_none());
}

#[test]
fn boolean_two_objects_preview_undo_and_cancel_do_not_publish() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects U").unwrap();
    assert!(c.solid_boolean_two_preview().is_some());
    assert_eq!(c.solid_boolean_two_hidden_sources(), Some([ids[0], ids[1]]));
    assert_eq!(c.document(), &before);
    c.submit("Undo").unwrap();
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.solid_boolean_two_hidden_sources().is_none());
    assert!(c.solid_boolean_needs_object());
    c.submit("U").unwrap();
    c.submit("Cancel").unwrap();
    assert!(!c.is_running());
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.solid_boolean_two_hidden_sources().is_none());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
}

#[test]
fn boolean_two_objects_cycles_through_empty_preview_without_committing_it() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    c.submit("Boolean2Objects").unwrap();
    c.submit("").unwrap();
    for _ in 0..3 {
        c.submit("Next").unwrap();
    }
    assert_eq!(c.solid_boolean_two_preview_mode(), Some("Intersection"));
    assert!(c.solid_boolean_two_preview().is_none());
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator.undo_len(), 0);
    c.submit("Next").unwrap();
    assert_eq!(
        c.solid_boolean_two_preview_mode(),
        Some("InversionIntersection")
    );
    assert!(c.solid_boolean_two_preview().is_some());
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
}

#[test]
fn boolean_two_objects_reverse_and_xor_have_measured_results_and_atomic_empty() {
    for (mode, second_side, offset, expected) in [
        ("R", 15.0, 5.0, Some(1000.0)),
        ("X", 15.0, 5.0, Some(1500.0)),
        ("InversionIntersection", 4.0, 3.0, Some(600.0)),
        ("Xor", 10.0, 30.0, Some(2000.0)),
        ("X", 10.0, 0.0, None),
    ] {
        let limits = Limits::default();
        let mut doc = Document::new("Boolean Two Objects additional results");
        let mut ids = Vec::new();
        for (index, (side, x)) in [(10.0, 0.0), (second_side, offset)].into_iter().enumerate() {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: side,
                    y_mm: 10.0,
                    z_mm: 10.0,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        c.submit("Boolean2Objects").unwrap();
        c.submit("DeleteInput=Yes").unwrap();
        let result = c.submit(mode);
        if let Some(expected) = expected {
            result.unwrap();
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 0);
            c.submit("").unwrap();
            let Geometry::Brep(body) = &c.document().entities.values().next().unwrap().geometry
            else {
                panic!("BRep result")
            };
            let measure = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
            assert!(
                (measure.volume_mm3 - expected).abs() < 1e-6,
                "{mode}: {}",
                measure.volume_mm3
            );
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
            c.redo().unwrap();
        } else {
            result.unwrap();
            assert!(c.solid_boolean_two_preview().is_none());
            assert_eq!(
                c.solid_boolean_two_preview_mode(),
                Some("InversionIntersection")
            );
            assert!(c.submit("").is_err());
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().undo_len(), 0);
            assert!(c.is_running());
        }
    }
}

fn difference_group_controller() -> (Controller, Vec<om9_core::identity::ObjectId>) {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean Difference groups");
    let mut ids = Vec::new();
    for (index, (side, placement)) in [
        (10.0, [0.0, 0.0, 0.0]),
        (10.0, [20.0, 0.0, 0.0]),
        (2.0, [1.0, 1.0, 1.0]),
        (2.0, [21.0, 1.0, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let brep = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(brep), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        entity.placement = om9_core::geometry::Transform::translation(placement);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    (Controller::new(doc, limits).unwrap(), ids)
}

#[test]
fn difference_disjoint_tools_rejects_false_empty_without_losing_redo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let affine = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &affine, 0.01, &limits).unwrap();
    let distant = creation::primitive(creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.01, &limits).unwrap();
    for profile in [0.5, 0.01] {
        let mut doc = Document::new("multi-tool false empty");
        let a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
        let b = om9_core::model::Entity::new("baked box", Geometry::Brep(block.clone()), None);
        let mut d = om9_core::model::Entity::new("distant box", Geometry::Brep(distant.clone()), None);
        d.placement = Transform::translation([20., 20., 20.]);
        let ids = [a.id, b.id, d.id]; for entity in [a, b, d] { doc.entities.insert(entity.id, entity); }
        let path = std::env::temp_dir().join(format!("om9-multi-false-empty-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap(); c.select_objects(&ids[..1]).unwrap();
        let before = c.document().clone();
        if profile == 0.5 { c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("BooleanDifference").unwrap(); }
        c.submit("").unwrap(); c.solid_difference_pick(ids[1], false).unwrap(); c.solid_difference_pick(ids[2], false).unwrap();
        let error = c.submit("").expect_err("false empty must be rejected");
        assert!(error.to_string().contains("boolean difference returned empty despite an interior witness"), "{error}");
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator.redo_len(), 1);
    }
}

#[test]
fn intersection_baked_affine_reopened_centered_sphere_is_accurate_and_undoable() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &placement, 0.01, &limits).unwrap();
    for (profile,center) in [(0.5,0.), (0.01,0.), (0.5,-0.001), (0.01,-0.001)] {
        let mut doc = Document::new("baked affine intersection refusal");
        let mut a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
        a.placement = Transform::translation([center,0.,0.]);
        let b = om9_core::model::Entity::new("baked box", Geometry::Brep(block.clone()), None);
        let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-intersection-refusal-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
        c.select_objects(&ids[..1]).unwrap(); let before = c.document().clone();
        if profile == 0.5 { c.dispatch("OM9-SOLID-002", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("BooleanIntersection").unwrap(); }
        c.submit("").unwrap(); c.solid_boolean_pick(ids[1], false).unwrap();
        c.submit("").unwrap();
        let result=c.document().entities.iter().find(|(id,_)|!ids.contains(id)).unwrap().1;
        let Geometry::Brep(body)=&result.geometry else {panic!("B-rep result")};
        let measure=creation::measure(body,&limits).unwrap();
        let expected=std::f64::consts::PI*(3.+center).powi(2)*(6.-center)/3.;
        assert!((measure.volume_mm3-expected).abs()<1e-5 && measure.closed && measure.solids==1 && measure.naked_edges==0,"{measure:?}");
        assert!(ids.iter().all(|id|c.document().entities[id]==before.entities[id]));
        let after=c.document().clone(); assert_eq!(c.coordinator.redo_len(),0);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
        c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn boolean_two_certified_box_preview_and_delete_are_undoable() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep,creation,transform_ops};
    let limits=Limits::default();
    let sphere=creation::primitive(creation::Primitive::Sphere {radius_mm:3.},0.01,&limits).unwrap();
    let block=creation::primitive(creation::Primitive::Box {x_mm:2.,y_mm:8.,z_mm:8.},0.01,&limits).unwrap();
    let transform=Transform([[2.,0.,0.,0.],[0.,1.,0.,-4.],[0.,0.,1.,-4.],[0.,0.,0.,1.]]);
    let block=transform_ops::transform_brep(&block,&transform,0.01,&limits).unwrap();
    for profile in [0.5,0.01] {
        let mut doc=Document::new("false empty Boolean Two");
        let mut a=om9_core::model::Entity::new("sphere",Geometry::Brep(brep::read_brep(sphere.source(),profile,&limits).unwrap()),None);
        a.placement=Transform::translation([-0.001,0.,0.]);
        let b=om9_core::model::Entity::new("box",Geometry::Brep(block.clone()),None);
        let ids=[a.id,b.id]; doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
        let path=std::env::temp_dir().join(format!("om9-intersection-preview-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap(); c.select_objects(&ids).unwrap();
        let before=c.document().clone();
        let preview=c.boolean_two_build_preview(3).unwrap();
        let measure=creation::measure(preview.body.as_ref().expect("nonempty preview"),&limits).unwrap();
        let expected=std::f64::consts::PI*(2.999_f64).powi(2)*6.001/3.;
        assert!((measure.volume_mm3-expected).abs()<1e-5 && measure.closed && measure.solids==1,"{measure:?}");
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator.redo_len(),1);
        c.boolean_two_commit(&preview,true).unwrap(); let after=c.document().clone();
        assert!(ids.iter().all(|id|!after.entities.contains_key(id))); assert_eq!(c.coordinator.redo_len(),0);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
        c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn intersection_baked_affine_reopened_offset_sphere_has_analytic_volume_and_undo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &placement, 0.01, &limits).unwrap();
    for (profile, center) in [(0.5, -0.5_f64), (0.01, 0.5)] {
        let mut doc = Document::new("baked affine intersection");
        let mut a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
        a.placement = Transform::translation([center, 0., 0.]);
        let b = om9_core::model::Entity::new("baked box", Geometry::Brep(block.clone()), None);
        let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-intersection-affine-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.select_objects(&ids[..1]).unwrap(); let before = c.document().clone();
        if profile == 0.5 { c.dispatch("OM9-SOLID-002", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("BooleanIntersection").unwrap(); }
        c.submit("").unwrap(); c.solid_boolean_pick(ids[1], false).unwrap(); c.submit("").unwrap();
        let result = c.document().entities.iter().find(|(id, _)| !ids.contains(id)).expect("intersection output").1;
        let Geometry::Brep(body) = &result.geometry else { panic!("B-rep output") };
        let actual = creation::measure(body, &limits).unwrap().volume_mm3;
        let expected = std::f64::consts::PI * (3. + center).powi(2) * (6. - center) / 3.;
        assert!((actual - expected).abs() < 1e-5, "profile {profile}: {actual} vs {expected}");
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn intersection_reopened_rotated_sphere_must_not_publish_wrong_volume() {
    use om9_core::geometry::Transform;
    use om9_kernel::{creation,transform_ops};
    let limits=Limits::default();
    let sphere=creation::primitive(creation::Primitive::Sphere {radius_mm:3.},0.01,&limits).unwrap();
    let block=creation::primitive(creation::Primitive::Box {x_mm:2.,y_mm:8.,z_mm:8.},0.01,&limits).unwrap();
    let affine=Transform([[2.,0.,0.,0.],[0.,1.,0.,-4.],[0.,0.,1.,-4.],[0.,0.,0.,1.]]);
    let block=transform_ops::transform_brep(&block,&affine,0.01,&limits).unwrap();
    for center in [0.,0.001_f64] {
    let mut rotation=transform_ops::rotation([0.;3],[1.,0.,0.],std::f64::consts::FRAC_PI_4).unwrap();
    rotation.0[0][3]=center;
    let sphere=transform_ops::transform_brep(&sphere,&rotation,0.01,&limits).unwrap();
    let mut doc=Document::new("rotated sphere intersection");
    let a=om9_core::model::Entity::new("sphere",Geometry::Brep(sphere),None);
    let b=om9_core::model::Entity::new("box",Geometry::Brep(block.clone()),None);
    let ids=[a.id,b.id]; doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
    let path=std::env::temp_dir().join(format!("om9-rotated-intersection-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
    c.select_objects(&ids[..1]).unwrap(); let before=c.document().clone(); c.submit("BooleanIntersection").unwrap();
    c.submit("").unwrap(); c.solid_boolean_pick(ids[1],false).unwrap();
    c.submit("").unwrap();
    let (result_id,result)=c.document().entities.iter().find(|(id,_)|!ids.contains(id)).unwrap();
    let Geometry::Brep(body)=&result.geometry else {panic!("B-rep result")};
    if center==0.001 {
        if let Some(path)=std::env::var_os("OM9_CERTIFIED_INTERSECTION_OUTPUT") {
            use std::io::Write;
            let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
            file.write_all(body.source().as_bytes()).unwrap();
        }
    }
    let measure=creation::measure(body,&limits).unwrap();
    let expected=std::f64::consts::PI*(3.+center).powi(2)*(6.-center)/3.;
    assert!((measure.volume_mm3-expected).abs()<1e-5 && measure.closed && measure.solids==1 && measure.naked_edges==0,"{measure:?}");
    let expected_area=std::f64::consts::PI*(6.*(3.+center)+9.-center*center);
    assert!((measure.area_mm2-expected_area).abs()<1e-5,"area={}: expected={expected_area}",measure.area_mm2);
    assert!(ids.iter().all(|id|c.document().entities[id]==before.entities[id]));
    let after=c.document().clone();
    let output_path=std::env::temp_dir().join(format!("om9-rotated-intersection-output-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&output_path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let reopened=Controller::from_file(&output_path,limits).unwrap(); std::fs::remove_file(output_path).unwrap();
    let Geometry::Brep(reopened_body)=&reopened.document().entities[result_id].geometry else {panic!("reopened B-rep")};
    let reopened_volume=creation::measure(reopened_body,&limits).unwrap().volume_mm3;
    assert!((reopened_volume-expected).abs()<1e-5);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn difference_baked_affine_rejects_false_empty_without_losing_redo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &placement, 0.01, &limits).unwrap();
    for profile in [0.5, 0.01] {
        let mut doc = Document::new("false empty Boolean Difference");
        let a = om9_core::model::Entity::new("sphere", Geometry::Brep(brep::read_brep(sphere.source(), profile, &limits).unwrap()), None);
        let b = om9_core::model::Entity::new("baked box", Geometry::Brep(block.clone()), None);
        let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let path = std::env::temp_dir().join(format!("om9-false-empty-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap(); c.select_objects(&ids[..1]).unwrap();
        let before = c.document().clone();
        if profile == 0.5 { c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("BooleanDifference").unwrap(); }
        c.submit("").unwrap(); c.solid_difference_pick(ids[1], false).unwrap();
        let error = c.submit("").expect_err("false empty must be rejected");
        assert!(error.to_string().contains("boolean difference returned empty despite an interior witness"), "{error}");
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator.redo_len(), 1);
    }
}

#[test]
fn difference_disjoint_tools_baked_affine_reopen_rejects_invalid_result_without_losing_redo() {
    use om9_core::geometry::Transform;
    use om9_kernel::{brep, creation, transform_ops};
    let limits = Limits::default();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 2., y_mm: 8., z_mm: 8. }, 0.01, &limits).unwrap();
    let affine = Transform([[2., 0., 0., 0.], [0., 1., 0., -4.], [0., 0., 1., -4.], [0., 0., 0., 1.]]);
    let block = transform_ops::transform_brep(&block, &affine, 0.01, &limits).unwrap();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let far = creation::primitive(creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.01, &limits).unwrap();
    for profile in [0.5, 0.01] {
        let mut document = Document::new("baked affine difference");
        let a = om9_core::model::Entity::new("target", Geometry::Brep(brep::read_brep(block.source(), profile, &limits).unwrap()), None);
        let b = om9_core::model::Entity::new("sphere cutter", Geometry::Brep(sphere.clone()), None);
        let mut d = om9_core::model::Entity::new("distant cutter", Geometry::Brep(far.clone()), None);
        d.placement = Transform::translation([20., 20., 20.]);
        let ids = [a.id, b.id, d.id];
        for entity in [a, b, d] { document.entities.insert(entity.id, entity); }
        let path = std::env::temp_dir().join(format!("om9-baked-difference-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut c = Controller::from_file(&path, limits).unwrap();
        std::fs::remove_file(path).unwrap();
        c.submit("Box 1 1 1").unwrap(); c.undo().unwrap();
        c.select_objects(&ids[..1]).unwrap();
        let before = c.document().clone();
        c.submit("BooleanDifference").unwrap(); c.submit("").unwrap();
        c.solid_difference_pick(ids[1], false).unwrap();
        c.solid_difference_pick(ids[2], false).unwrap();
        let error = c.submit("").expect_err("invalid difference must not be published");
        assert!(error.to_string().contains("invalid BRep source shape"), "profile {profile}: {error}");
        assert_eq!(c.document(), &before, "profile {profile}");
        assert_eq!(c.coordinator.redo_len(), 1, "profile {profile}");
    }
}

#[test]
fn difference_fuses_overlapping_groups_and_honors_delete_input() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    for delete_input in [false, true] {
        let limits = Limits::default();
        let mut doc = Document::new("Overlapping Boolean sets");
        let mut ids = Vec::new();
        for (index, (side, placement)) in [
            (10.0, [0.0, 0.0, 0.0]),
            (10.0, [5.0, 0.0, 0.0]),
            (4.0, [6.0, 1.0, 1.0]),
            (4.0, [8.0, 1.0, 1.0]),
        ]
        .into_iter()
        .enumerate()
        {
            let body = om9_kernel::creation::primitive(
                om9_kernel::creation::Primitive::Box {
                    x_mm: side,
                    y_mm: side,
                    z_mm: side,
                },
                0.01,
                &limits,
            )
            .unwrap();
            let mut entity = om9_core::model::Entity::new("body", Geometry::Brep(body), None);
            entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            entity.placement = om9_core::geometry::Transform::translation(placement);
            ids.push(entity.id);
            doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, limits).unwrap();
        c.select_objects(&ids[..2]).unwrap();
        if delete_input {
            c.submit("BooleanDifference").unwrap();
        } else {
            c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap();
        }
        assert!(c.is_running());
        assert_eq!(c.selected_ordered(), ids[..2]);
        let option = c
            .command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap();
        assert_eq!(option.value, CommandValue::Flag(false));
        assert_eq!(option.shortcut, Some('D'));
        assert!(c.submit("DeleteInput=invalid").is_err());
        assert_eq!(
            c.command_options()
                .into_iter()
                .find(|o| o.id.name() == "DeleteInput")
                .unwrap()
                .value,
            CommandValue::Flag(false)
        );
        if delete_input {
            let id = c.resolve_command_option("d").unwrap();
            c.activate_command_option(id).unwrap();
            assert_eq!(
                c.command_options()
                    .into_iter()
                    .find(|o| o.id.name() == "DeleteInput")
                    .unwrap()
                    .value,
                CommandValue::Flag(true)
            );
        }
        c.submit("").unwrap();
        for id in &ids[2..] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
        for id in &ids[..2] {
            assert!(!c.document().entities.contains_key(id));
        }
        for id in &ids[2..] {
            assert_eq!(c.document().entities.contains_key(id), !delete_input);
        }
        let output = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &output.geometry else {
            panic!("BRep result")
        };
        let measures = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(measures.solids, 1);
        assert!(
            (measures.volume_mm3 - 1404.0).abs() < 1e-6,
            "{}",
            measures.volume_mm3
        );
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(
            c.document().entities.len(),
            if delete_input { 1 } else { 3 }
        );
    }
}

#[test]
fn difference_selects_target_and_cutter_groups_through_menu_and_cmd() {
    for menu in [true, false] {
        let (mut c, ids) = difference_group_controller();
        if menu {
            c.dispatch("OM9-SOLID-001", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("BooleanDifference").unwrap();
        }
        assert!(c.solid_difference_needs_object());
        assert!(c.prompt().contains("subtract from"));
        assert!(c.submit("").is_err());
        for id in &ids[..2] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert!(c.prompt().contains("Select objects to subtract ("));
        assert!(c.solid_difference_pick(ids[0], false).is_err());
        assert!(c.submit("").is_err());
        for id in &ids[2..] {
            c.solid_difference_pick(*id, false).unwrap();
        }
        c.submit("").unwrap();
        assert!(!c.is_running());
        assert_eq!(c.document().entities.len(), 3);
        assert!(c.document().entities.contains_key(&ids[2]));
        assert!(c.document().entities.contains_key(&ids[3]));
        let entity = c
            .document()
            .entities
            .iter()
            .find(|(id, _)| !ids.contains(id))
            .unwrap()
            .1;
        let Geometry::Brep(body) = &entity.geometry else {
            panic!("BRep output")
        };
        let measures = om9_kernel::creation::measure(body, c.coordinator().limits()).unwrap();
        assert!((measures.volume_mm3 - 1984.0).abs() < 1e-6);
        assert_eq!(measures.solids, 2);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 4);
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), 3);
    }
}

#[test]
fn difference_cancel_after_target_selection_leaves_document_unchanged() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.cancel();
    assert!(!c.is_running());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_uses_one_preselected_object_as_target() {
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("BooleanDifference").unwrap();
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
}

#[test]
fn difference_inline_option_cannot_bypass_selection_stages() {
    use crate::command_options::{CommandOptionInterface, CommandValue};
    let (mut c, ids) = difference_group_controller();
    c.select_objects(&ids[..2]).unwrap();
    let before = c.document().clone();
    assert!(c.dispatch_solid_args("OM9-SOLID-001", &[]).is_err());
    c.submit("BooleanDifference DeleteInput=Yes").unwrap();
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.selected_ordered(), ids[..2]);
    assert_eq!(
        c.command_options()
            .into_iter()
            .find(|o| o.id.name() == "DeleteInput")
            .unwrap()
            .value,
        CommandValue::Flag(true)
    );
    c.cancel();
    assert!(c.submit("BooleanDifference unexpected").is_err());
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_empty_result_refuses_without_deleting_inputs() {
    let (mut c, ids) = difference_group_controller();
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.submit("DeleteInput=Yes").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert!(c.solid_difference_needs_object());
}

#[test]
fn difference_pending_undo_revises_selection_and_restores_target_stage() {
    use crate::command_options::CommandOptionInterface;
    let (mut c, ids) = difference_group_controller();
    c.submit("BooleanDifference").unwrap();
    assert_eq!(c.active_feature(), Some("OM9-SOLID-001"));
    c.solid_difference_pick(ids[0], false).unwrap();
    c.solid_difference_pick(ids[1], false).unwrap();
    let undo = c.resolve_command_option("u").expect("Undo shortcut");
    c.activate_command_option(undo).unwrap();
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.submit("Undo").unwrap();
    assert!(c.selected().is_empty());
    c.submit("Undo").unwrap();
    assert!(c.prompt().contains("subtract from"));
    assert_eq!(c.selected_ordered(), vec![ids[0]]);
    let cancel = c.resolve_command_option("Cancel").expect("Cancel option");
    c.activate_command_option(cancel).unwrap();
    assert!(!c.is_running());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn difference_rejects_stale_target_before_publication() {
    use om9_core::transaction::{Mutation, TransactionRequest};
    let (mut c, ids) = difference_group_controller();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.submit("").unwrap();
    let source = c.document().stamp(ids[0]).unwrap();
    let prepared = c
        .coordinator
        .prepare(TransactionRequest::new(
            "external edit",
            vec![Mutation::Placement {
                source,
                placement: om9_core::geometry::Transform::translation([1.0, 0.0, 0.0]),
            }],
        ))
        .unwrap();
    c.coordinator.commit(prepared).unwrap();
    c.refresh_scene().unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    assert!(matches!(c.submit(""), Err(om9_core::Error::Stale(_))));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    assert!(c.solid_difference_needs_object());
    c.cancel();
}

#[test]
fn difference_late_open_cutter_failure_is_atomic_and_retryable() {
    let (initial, ids) = difference_group_controller();
    let mut doc = initial.document().clone();
    let entity = doc.entities.get_mut(&ids[3]).unwrap();
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep cutter")
    };
    let open = om9_kernel::editing::remove_face(body, 0, 0.01, &Limits::default()).unwrap();
    entity.geometry = Geometry::Brep(open);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 4 4 4").unwrap();
    c.undo().unwrap();
    assert_eq!(c.coordinator().redo_len(), 1);
    let before = c.document().clone();
    c.submit("BooleanDifference").unwrap();
    c.solid_difference_pick(ids[0], false).unwrap();
    c.solid_difference_pick(ids[1], false).unwrap();
    c.submit("").unwrap();
    c.solid_difference_pick(ids[2], false).unwrap();
    c.solid_difference_pick(ids[3], false).unwrap();
    assert!(c.submit("").is_err());
    assert!(c.solid_difference_needs_object());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
    c.solid_difference_pick(ids[3], true).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}
