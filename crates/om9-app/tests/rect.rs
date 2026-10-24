#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::Document;

fn controller() -> Controller {
    Controller::new(Document::new("rect"), Limits::default()).unwrap()
}

fn bounds(c: &Controller) -> ([f64; 3], [f64; 3]) {
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("polycurve");
    };
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for segment in segments {
        let om9_core::geometry::Curve::Line { start, end, .. } = segment else {
            panic!("line");
        };
        for point in [start, end] {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
    }
    (min, max)
}

#[test]
fn rectangle_two_corners_match() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert_eq!(min, [0.0, 0.0, 0.0]);
    assert_eq!(max, [10.0, 20.0, 0.0]);
}

#[test]
fn rectangle_width_and_height_commit_after_enter() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Width=10").unwrap();
    c.submit("Height=20").unwrap();
    c.submit("").unwrap();
    let (_, max) = bounds(&c);
    assert_eq!(max, [10.0, 20.0, 0.0]);
}

#[test]
fn rectangle_preview_has_four_edges() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.rect_preview([10.0, 20.0, 0.0]).len(), 4);
}


#[test]
fn rectangle_square_option_uses_the_longer_side() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("Square").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 20.0).abs() < 1e-9);
    assert!((max[1] - 20.0).abs() < 1e-9);
}


#[test]
fn rectangle_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 20.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[2] - 20.0).abs() < 1e-9);
    assert!(min[1].abs() < 1e-9 && max[1].abs() < 1e-9);
}


#[test]
fn rectangle_preview_follows_the_active_cplane() {
    let mut c = controller();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.rect_preview([10.0, 0.0, 20.0]);
    assert_eq!(wire.len(), 4);
    let max_z = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[2])
        .fold(f64::MIN, f64::max);
    assert!((max_z - 20.0).abs() < 1e-9, "{max_z}");
    assert!(wire
        .iter()
        .flat_map(|segment| segment.iter())
        .all(|point| point[1].abs() < 1e-9));
}


#[test]
fn rectangle_center_mode_uses_the_center_and_doubles_the_extent() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([5.0, 10.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((min[0] + 5.0).abs() < 1e-9, "{min:?}");
    assert!((min[1] + 10.0).abs() < 1e-9, "{min:?}");
    assert!((max[0] - 5.0).abs() < 1e-9, "{max:?}");
    assert!((max[1] - 10.0).abs() < 1e-9, "{max:?}");
}

#[test]
fn rectangle_center_preview_is_centered() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.rect_preview([5.0, 10.0, 0.0]);
    assert_eq!(wire.len(), 4);
    let max_x = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[0])
        .fold(f64::MIN, f64::max);
    let min_x = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[0])
        .fold(f64::MAX, f64::min);
    assert!((max_x - 5.0).abs() < 1e-9, "max_x={max_x} wire={wire:?}");
    assert!((min_x + 5.0).abs() < 1e-9, "min_x={min_x}");
}


#[test]
fn rectangle_three_point_mode_uses_edge_then_width() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((min[0]).abs() < 1e-9);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}


#[test]
fn rectangle_three_point_preview_uses_the_edge_frame() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let wire = c.rect_preview([0.0, 5.0, 0.0]);
    assert_eq!(wire.len(), 4);
    let max_y = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[1])
        .fold(f64::MIN, f64::max);
    assert!((max_y - 5.0).abs() < 1e-9, "{max_y}");
}


#[test]
fn rectangle_vertical_mode_uses_a_vertical_edge() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([2.0, 0.0, 5.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 5.0]).unwrap();
    let (min, max) = bounds(&c);
    assert!((min[0]).abs() < 1e-9);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[2] - 5.0).abs() < 1e-9);
}


#[test]
fn rectangle_vertical_preview_uses_the_vertical_edge() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([2.0, 0.0, 5.0]).unwrap();
    let wire = c.rect_preview([10.0, 0.0, 5.0]);
    assert_eq!(wire.len(), 4);
    let max_x = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[0])
        .fold(f64::MIN, f64::max);
    let max_z = wire
        .iter()
        .flat_map(|segment| segment.iter())
        .map(|point| point[2])
        .fold(f64::MIN, f64::max);
    assert!((max_x - 10.0).abs() < 1e-9);
    assert!((max_z - 5.0).abs() < 1e-9);
}


#[test]
fn rectangle_accepts_length_option() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Length=10").unwrap();
    c.submit("Height=5").unwrap();
    c.submit("").unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}


#[test]
fn rectangle_three_point_accepts_numeric_width() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Width=5").unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}


#[test]
fn rectangle_three_point_accepts_numeric_height() {
    let mut c = controller();
    c.submit("Rectangle").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Height=5").unwrap();
    let (_, max) = bounds(&c);
    assert!((max[0] - 10.0).abs() < 1e-9);
    assert!((max[1] - 5.0).abs() < 1e-9);
}
