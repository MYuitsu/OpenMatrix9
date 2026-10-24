#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller() -> Controller {
    Controller::new(Document::new("ellipse"), Limits::default()).unwrap()
}

fn bounds(c: &Controller) -> ([f64; 3], [f64; 3]) {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for point in &curve.poles {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

#[test]
fn ellipse_center_major_and_minor_points_match() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    for expected in ["RadiusX=", "RadiusY=", "MarkFoci"] {
        assert!(c
            .ellipse_option_tokens()
            .iter()
            .any(|(_, command)| *command == expected));
    }
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((min[0] + 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}

#[test]
fn ellipse_radius_options_commit_on_enter() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("RadiusX=10").unwrap();
    c.submit("RadiusY=5").unwrap();
    c.submit("").unwrap();
    let (_min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}

#[test]
fn ellipse_preview_is_a_closed_ring() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.ellipse_preview([10.0, 0.0, 0.0]).len(), 64);
}


#[test]
fn ellipse_corner_mode_uses_the_bounding_box() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Corner").unwrap();
    c.accept_world_point([-10.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 5.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((min[0] + 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
    assert!((min[1] + 5.0).abs() < 1e-9);
}


#[test]
fn ellipse_diameter_mode_uses_major_diameter() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Diameter").unwrap();
    c.accept_world_point([-10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
    assert!((min[0] + 10.0).abs() < 1e-9);
}

#[test]
fn ellipse_vertical_mode_swaps_the_radii() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}

#[test]
fn ellipse_from_foci_uses_the_focal_sum() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("FromFoci").unwrap();
    c.accept_world_point([-5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 4.0, 0.0]).unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 41.0_f64.sqrt()).abs() < 1e-9);
    assert!((max[1] - 4.0).abs() < 1e-9);
}


#[test]
fn ellipse_center_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Ellipse").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 5.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[2] - 5.0).abs() < 1e-9);
    assert!(min[1].abs() < 1e-9 && max[1].abs() < 1e-9);
}


#[test]
fn ellipse_mark_foci_creates_two_points() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("FromFoci").unwrap();
    c.submit("MarkFoci=on").unwrap();
    c.accept_world_point([-3.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([3.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 4.0, 0.0]).unwrap();
    let points = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "EllipseFoci")
        .count();
    assert_eq!(points, 2);
}


#[test]
fn ellipse_deformable_rebuilds_to_the_requested_point_count() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Deformable=on").unwrap();
    for expected in ["Degree=", "PointCount="] {
        assert!(c
            .ellipse_option_tokens()
            .iter()
            .any(|(_, command)| *command == expected));
    }
    c.submit("Degree=3").unwrap();
    c.submit("PointCount=32").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) =
        &c.document().entities[&id].geometry
    else {
        panic!("nurbs");
    };
    assert_eq!(curve.degree, 3);
    assert_eq!(curve.poles.len(), 32);
}


#[test]
fn ellipse_exposes_mode_tokens() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    let tokens = c.ellipse_option_tokens();
    for expected in ["Center", "Corner", "Diameter", "Vertical", "FromFoci"] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}


#[test]
fn ellipse_corner_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Ellipse").unwrap();
    c.submit("Corner").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 20.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9, "{max:?}");
    assert!((max[2] - 20.0).abs() < 1e-9, "{max:?}");
    assert!(min[1].abs() < 1e-9 && max[1].abs() < 1e-9);
}


#[test]
fn ellipse_vertical_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Ellipse").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 5.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9, "{max:?}");
    assert!((max[2] - 5.0).abs() < 1e-9, "{max:?}");
    assert!(min[1].abs() < 1e-9 && max[1].abs() < 1e-9);
}


#[test]
fn ellipse_around_curve_uses_the_path() {
    let mut doc = Document::new("ellipse around");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipse").unwrap();
    c.submit("AroundCurve").unwrap();
    assert!(c
        .ellipse_option_tokens()
        .iter()
        .any(|(_, command)| *command == "OnCurve="));
    c.submit("OnCurve=0.5").unwrap();
    c.submit("RadiusX=5").unwrap();
    c.submit("RadiusY=3").unwrap();
    c.submit("").unwrap();
    let entity = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Ellipse")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &entity.geometry else {
        panic!("nurbs");
    };
    let center = curve.poles.iter().fold([0.0; 3], |sum, point| {
        [sum[0] + point[0], sum[1] + point[1], sum[2] + point[2]]
    });
    let count = curve.poles.len() as f64;
    assert!((center[0] / count - 5.0).abs() < 0.1);
}


#[test]
fn ellipse_around_curve_preview_uses_the_path_normal() {
    let mut doc = Document::new("ellipse around preview");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipse").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    let wire = c.ellipse_preview([5.0, 5.0, 0.0]);
    assert!(!wire.is_empty());
    for [point, _] in wire {
        assert!((point[0] - 5.0).abs() < 1e-9, "{point:?}");
        let radius = (point[1].powi(2) + point[2].powi(2)).sqrt();
        assert!((radius - 5.0).abs() < 1e-9);
    }
}


#[test]
fn ellipse_around_curve_accepts_a_path_pick() {
    let mut doc = Document::new("ellipse around pick");
    let curve = om9_core::geometry::Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let entity = Entity::new("line", Geometry::Curve(curve), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipse").unwrap();
    c.submit("AroundCurve").unwrap();
    c.accept_world_point([5.0, 5.0, 0.0]).unwrap();
    c.submit("RadiusX=5").unwrap();
    c.submit("RadiusY=3").unwrap();
    c.submit("").unwrap();
    let entity = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Ellipse")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &entity.geometry else {
        panic!("nurbs");
    };
    let min_x = curve.poles.iter().map(|point| point[0]).fold(f64::MAX, f64::min);
    let max_x = curve.poles.iter().map(|point| point[0]).fold(f64::MIN, f64::max);
    assert!(((min_x + max_x) * 0.5 - 5.0).abs() < 1e-6, "{min_x} {max_x}");
}


#[test]
fn ellipse_from_foci_exposes_undo_token() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("FromFoci").unwrap();
    c.accept_world_point([-3.0, 0.0, 0.0]).unwrap();
    assert!(c
        .ellipse_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Undo"));
}


#[test]
fn ellipse_center_mode_can_mark_foci() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("MarkFoci=on").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let foci: Vec<_> = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "EllipseFoci")
        .collect();
    assert_eq!(foci.len(), 2);
    let Geometry::Point(point) = foci[0].geometry else {
        panic!("point");
    };
    assert!((point[0].abs() - 75.0f64.sqrt()).abs() < 1e-6);
}


#[test]
fn ellipse_accepts_approx_alias() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    assert!(c
        .ellipse_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Approx"));
    c.submit("Approx=on").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let ellipse = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Ellipse")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &ellipse.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 12);
}


#[test]
fn ellipse_accepts_bare_deformable_option() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let ellipse = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Ellipse")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &ellipse.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 12);
}


#[test]
fn ellipse_deformable_token_toggles_off() {
    let mut c = controller();
    c.submit("Ellipse").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("Deformable").unwrap();
    c.submit("PointCount=12").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let ellipse = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Ellipse")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &ellipse.geometry else {
        panic!("nurbs");
    };
    assert_eq!(curve.poles.len(), 9);
}
