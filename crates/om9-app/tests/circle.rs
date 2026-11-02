#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("circle"), Limits::default()).unwrap()
}

fn radius_of(c: &Controller) -> f64 {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("circle curve");
    };
    *radius
}

#[test]
fn circle_center_and_radius_match() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([1.0, 2.0, 3.0]).unwrap();
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert!((radius_of(&c) - 5.0).abs() < 1e-9);
}

#[test]
fn circle_two_point_uses_the_diameter() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("2Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-9);
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle {
        center, x, radius, ..
    }) = &c.document().entities[&id].geometry
    else {
        panic!("circle");
    };
    let endpoint = [
        center[0] + x[0] * radius,
        center[1] + x[1] * radius,
        center[2] + x[2] * radius,
    ];
    assert!((endpoint[0] - 10.0).abs() < 1e-9);
    assert!(endpoint[1].abs() < 1e-9);
}

#[test]
fn circle_three_point_uses_the_circumcircle() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    assert!((radius_of(&c) - 1.0).abs() < 1e-9);
}

#[test]
fn circle_preview_and_tokens_follow_the_pending_points() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    assert!(
        c.circle_option_tokens()
            .iter()
            .any(|(_, command)| *command == "2Point")
    );
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.circle_preview([5.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        assert!((point[0].powi(2) + point[1].powi(2) + point[2].powi(2)).sqrt() - 5.0 < 1e-9);
    }
}

#[test]
fn circle_center_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { x, y, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("circle");
    };
    assert!((x[1]).abs() < 1e-9);
    assert!((y[1]).abs() < 1e-9);
    assert!((y[2]).abs() > 0.9);
}

#[test]
fn circle_accepts_diameter_and_area_sizes() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Size=Diameter").unwrap();
    c.submit("20").unwrap();
    assert!((radius_of(&c) - 10.0).abs() < 1e-9);

    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Size=Area").unwrap();
    c.submit(&(std::f64::consts::PI * 25.0).to_string())
        .unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-9);
}

#[test]
fn circle_accepts_circumference_size_and_exposes_size_tokens() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert!(
        c.circle_option_tokens()
            .iter()
            .any(|(_, command)| *command == "Size=Diameter")
    );
    c.submit("Size=Circumference").unwrap();
    c.submit(&(std::f64::consts::TAU * 5.0).to_string())
        .unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-9);
}

#[test]
fn circle_vertical_mode_uses_a_vertical_plane() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { x, y, radius, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("circle");
    };
    let normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    assert!((radius - 5.0).abs() < 1e-9);
    assert!(normal[1].abs() > 0.99, "{normal:?}");
}

#[test]
fn circle_vertical_mode_pick_sets_the_plane_from_the_point() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { x, y, radius, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("circle");
    };
    let normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    assert!((radius - 5.0).abs() < 1e-9);
    assert!(normal[1].abs() > 0.99, "{normal:?}");
}

#[test]
fn circle_vertical_preview_is_a_closed_ring() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.circle_preview([5.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9);
    }
}

#[test]
fn circle_center_preview_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.circle_preview([5.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        assert!(point[1].abs() < 1e-9);
        let radius = (point[0].powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9);
    }
}

#[test]
fn circle_two_point_preview_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Circle").unwrap();
    c.submit("2Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.circle_preview([10.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        assert!(point[1].abs() < 1e-9);
        let radius = ((point[0] - 5.0).powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9);
    }
}

#[test]
fn circle_fit_points_fits_a_circle() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("FitPoints").unwrap();
    for point in [
        [5.0, 0.0, 0.0],
        [0.0, 5.0, 0.0],
        [-5.0, 0.0, 0.0],
        [0.0, -5.0, 0.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-6, "{}", radius_of(&c));
}

#[test]
fn circle_fit_points_preview_is_a_ring() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("FitPoints").unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let wire = c.circle_preview([-5.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-6);
    }
}

#[test]
fn circle_accepts_diameter_and_area_prefixes() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Diameter=20").unwrap();
    assert!((radius_of(&c) - 10.0).abs() < 1e-9);

    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Circumference=31.41592653589793").unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-6);
}

#[test]
fn circle_around_curve_uses_the_path() {
    let mut doc = Document::new("circle around");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Circle").unwrap();
    c.submit("AroundCurve").unwrap();
    assert!(
        c.circle_option_tokens()
            .iter()
            .any(|(_, command)| *command == "OnCurve=")
    );
    c.submit("OnCurve=0.5").unwrap();
    c.submit("5").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { center, x, y, .. }) = &circle.geometry
    else {
        panic!("circle");
    };
    assert!((center[0] - 5.0).abs() < 1e-9);
    let normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    assert!(normal[0].abs() > 0.99, "{normal:?}");
}

#[test]
fn circle_around_curve_preview_uses_the_path_normal() {
    let mut doc = Document::new("around preview");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Circle").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    let wire = c.circle_preview([5.0, 5.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        assert!((point[0] - 5.0).abs() < 1e-9);
        let radius = ((point[1]).powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9);
    }
}

#[test]
fn circle_tangent_to_two_lines() {
    let mut doc = Document::new("circle tangent");
    let mut ids = Vec::new();
    for (start, end) in [
        ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
    ] {
        let curve = om9_core::geometry::Curve::Line {
            start,
            end,
            domain: [0.0, 1.0],
        };
        let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Circle").unwrap();
    c.submit("Tangent").unwrap();
    let tokens = c.circle_option_tokens();
    for expected in ["Radius=", "Solution="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
    c.submit("Radius=5").unwrap();
    c.submit("Solution=1").unwrap();
    c.submit("").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { center, radius, .. }) =
        &circle.geometry
    else {
        panic!("circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    assert!((center[0].abs() - 5.0).abs() < 1e-6);
    assert!((center[1].abs() - 5.0).abs() < 1e-6);
}

#[test]
fn circle_tangent_preview_is_a_ring() {
    let mut doc = Document::new("circle tangent preview");
    let mut ids = Vec::new();
    for (start, end) in [
        ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
    ] {
        let curve = om9_core::geometry::Curve::Line {
            start,
            end,
            domain: [0.0, 1.0],
        };
        let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Circle").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Radius=5").unwrap();
    let wire = c.circle_preview([0.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
}

#[test]
fn circle_tangent_to_a_line_and_point() {
    let mut doc = Document::new("circle tangent point");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Circle").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Radius=5").unwrap();
    c.submit("Point=5,5,0").unwrap();
    c.submit("").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { center, radius, .. }) =
        &circle.geometry
    else {
        panic!("circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    assert!((center[1].abs() - 5.0).abs() < 1e-6);
    let distance = ((center[0] - 5.0).powi(2) + (center[1] - 5.0).powi(2)).sqrt();
    assert!((distance - 5.0).abs() < 1e-6, "{center:?}");
}

#[test]
fn circle_tangent_point_preview_is_a_ring() {
    let mut doc = Document::new("circle tangent point preview");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Circle").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Radius=5").unwrap();
    c.submit("Point=5,5,0").unwrap();
    assert_eq!(c.circle_preview([0.0, 0.0, 0.0]).len(), 64);
}

#[test]
fn circle_tangent_accepts_a_bare_radius() {
    let mut doc = Document::new("circle tangent bare");
    let mut ids = Vec::new();
    for (start, end) in [
        ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 10.0, 0.0]),
    ] {
        let curve = om9_core::geometry::Curve::Line {
            start,
            end,
            domain: [0.0, 1.0],
        };
        let entity = om9_core::model::Entity::new("line", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Circle").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("5").unwrap();
    c.submit("").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) = &circle.geometry else {
        panic!("circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
}

#[test]
fn circle_deformable_rebuilds_to_point_count() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Deformable=on").unwrap();
    c.submit("Degree=3").unwrap();
    c.submit("PointCount=16").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &circle.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 16);
    assert_eq!(curve.degree, 3);
}

#[test]
fn circle_deformable_rejects_point_count_not_above_degree() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Deformable=on").unwrap();
    c.submit("Degree=5").unwrap();
    c.submit("PointCount=4").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert!(c.accept_world_point([5.0, 0.0, 0.0]).is_err());
}

#[test]
fn circle_accepts_approx_alias() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    assert!(
        c.circle_option_tokens()
            .iter()
            .any(|(_, command)| *command == "Approx")
    );
    c.submit("Approx=on").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &circle.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 12);
}

#[test]
fn circle_accepts_bare_deformable_option() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &circle.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 12);
}

#[test]
fn circle_deformable_token_toggles_off() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("Deformable").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Circle")
        .unwrap();
    assert!(matches!(
        circle.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Circle { .. })
    ));
}

#[test]
fn circle_deformable_preview_uses_the_rebuilt_curve() {
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.circle_preview([5.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 64);
    for [point, _] in wire {
        let radius = (point[0].powi(2) + point[1].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-6);
    }
}

#[test]
fn circle_area_size_uses_the_shared_choice_option() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = controller();
    c.submit("Circle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let area = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Size=Area")
        .expect("Size=Area option");
    assert_eq!(
        c.activate_command_option(area.id).unwrap(),
        CommandOptionEffect::Applied
    );
    c.submit(&(std::f64::consts::PI * 25.0).to_string())
        .unwrap();
    assert!((radius_of(&c) - 5.0).abs() < 1e-9);
}
