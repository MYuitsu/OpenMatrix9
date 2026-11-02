// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Unit},
    model::Document,
};
use om9_scene::camera::{CPlane, CPlanePreset};

fn controller(plane: CPlane, unit: Unit) -> Controller {
    let mut document = Document::new("rectangle regressions");
    document.page_unit = unit;
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.workspace
        .views
        .get_mut(&c.workspace.active)
        .unwrap()
        .cplane = plane;
    c
}

fn corners(c: &Controller) -> Vec<Point3> {
    assert_eq!(c.document().entities.len(), 1);
    let Geometry::Curve(Curve::Polycurve { segments, .. }) =
        &c.document().entities.values().next().unwrap().geometry
    else {
        panic!("polycurve")
    };
    assert_eq!(segments.len(), 4);
    segments
        .iter()
        .map(|segment| match segment {
            Curve::Line { start, .. } => *start,
            _ => panic!("four exact lines"),
        })
        .collect()
}

fn contains(points: &[Point3], expected: Point3) -> bool {
    points
        .iter()
        .any(|p| (0..3).all(|axis| (p[axis] - expected[axis]).abs() < 1e-8))
}

fn assert_preview_commit(c: &mut Controller, second: Point3, expected: &[Point3]) {
    let preview = c.rect_preview(second);
    assert_eq!(preview.len(), 4);
    for point in expected {
        assert!(
            contains(
                &preview.iter().map(|edge| edge[0]).collect::<Vec<_>>(),
                *point
            ),
            "preview {preview:?} must contain {point:?}"
        );
    }
    assert!(c.document().entities.is_empty());
    c.accept_world_point(second).unwrap();
    let committed = corners(c);
    for point in expected {
        assert!(
            contains(&committed, *point),
            "commit {committed:?} must contain {point:?}"
        );
    }
    for [a, _] in preview {
        assert!(contains(&committed, a));
    }
    let scene = c.scene().unwrap();
    assert_eq!(scene.objects.len(), 1);
    assert!(scene.objects[0].visible);
    assert_eq!(scene.epoch, c.coordinator().epoch());
}

#[test]
fn corners_keep_all_four_drag_directions_in_shifted_and_rotated_cplanes() {
    let planes = [
        CPlane::world_xy(),
        CPlane::preset(CPlanePreset::Front),
        CPlane::world_xy()
            .rotated(0.37)
            .unwrap()
            .with_origin([17., -11., 6.])
            .unwrap(),
    ];
    for plane in planes {
        for menu in [false, true] {
            for (dx, dy) in [(10., 20.), (-10., 20.), (10., -20.), (-10., -20.)] {
                let first = plane.point([3., 7., 4.]).unwrap();
                let second = plane.point([3. + dx, 7. + dy, 4.]).unwrap();
                let mut c = controller(plane, Unit::Millimeter);
                if menu {
                    c.dispatch("OM9-CURVE-004", InvocationOrigin::Menu).unwrap();
                } else {
                    c.submit("Rectangle").unwrap();
                }
                c.accept_world_point(first).unwrap();
                assert_preview_commit(&mut c, second, &[first, second]);
            }
        }
    }
}

#[test]
fn typed_first_and_relative_second_corner_share_the_coordinate_parser() {
    let plane = CPlane::world_xy()
        .rotated(0.37)
        .unwrap()
        .with_origin([17., -11., 6.])
        .unwrap();
    let mut c = controller(plane, Unit::Centimeter);
    c.submit("Rectangle").unwrap();
    c.submit("0.3,0.7,0.4").unwrap();
    c.submit("@-1,-2,0").unwrap();
    let committed = corners(&c);
    assert!(contains(&committed, plane.point([3., 7., 4.]).unwrap()));
    assert!(contains(&committed, plane.point([-7., -13., 4.]).unwrap()));
}

#[test]
fn typed_world_corners_ignore_cplane_origin_and_project_off_plane_height() {
    let mut c = controller(
        CPlane::world_xy().with_origin([100., 200., 0.]).unwrap(),
        Unit::Millimeter,
    );
    c.submit("Rectangle").unwrap();
    c.submit("W3,7,4").unwrap();
    c.submit("W-7,-13,9").unwrap();
    let committed = corners(&c);
    assert!(contains(&committed, [3., 7., 4.]));
    assert!(contains(&committed, [-7., -13., 4.]));
}

#[test]
fn square_and_fixed_length_keep_the_cursor_side() {
    for square in [false, true] {
        let mut c = controller(CPlane::world_xy(), Unit::Millimeter);
        c.submit("Rectangle").unwrap();
        if square {
            c.submit("Square").unwrap();
        }
        c.accept_world_point([3., 7., 0.]).unwrap();
        if !square {
            c.submit("Length=10").unwrap();
        }
        let expected = if square {
            [-27., -23., 0.]
        } else {
            [-7., -13., 0.]
        };
        assert_preview_commit(&mut c, [-27., -13., 0.], &[[3., 7., 0.], expected]);
    }
}

#[test]
fn bad_input_and_degenerate_pick_can_retry_then_undo_redo_and_cancel() {
    let mut c = controller(CPlane::world_xy(), Unit::Millimeter);
    c.submit("Rectangle").unwrap();
    assert!(c.submit("bad,7,0").is_err());
    assert!(c.submit("3,NaN,0").is_err());
    assert!(c.document().entities.is_empty());
    c.submit("3,7,0").unwrap();
    assert!(c.submit("3,-13,0").is_err());
    assert!(c.document().entities.is_empty());
    c.submit("-7,-13,0").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.undo().unwrap();
    assert!(c.document().entities.is_empty());
    c.redo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    let epoch = c.coordinator().epoch();
    c.submit("Rectangle").unwrap();
    c.submit("1,1,0").unwrap();
    c.submit("Cancel").unwrap();
    assert_eq!(c.coordinator().epoch(), epoch);
    assert_eq!(c.document().entities.len(), 1);
    assert!(c.rect_preview([5., 5., 0.]).is_empty());
}
