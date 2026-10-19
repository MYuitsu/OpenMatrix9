// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Mesh},
    model::{Document, Entity},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, Scene, Subelement},
    selection::SelectionRect,
};

fn fixture(geometry: Geometry) -> (Scene, Camera, Viewport) {
    let mut d = Document::new("selection regressions");
    let e = Entity::new("subject", geometry, None);
    d.entities.insert(e.id, e);
    (
        Scene::capture(std::sync::Arc::new(d), 1, &Limits::default()).unwrap(),
        Camera::orthographic([0., 0., 10.], [0.; 3], [0., 1., 0.], 20.).unwrap(),
        Viewport {
            origin: [40., 70.],
            size: [400., 400.],
        },
    )
}
fn band() -> SelectionRect {
    SelectionRect {
        min: [230., 260.],
        max: [250., 280.],
    }
}
fn line(a: [f64; 3], b: [f64; 3]) -> Geometry {
    Geometry::Curve(Curve::Line {
        start: a,
        end: b,
        domain: [0., 1.],
    })
}
fn quad() -> Geometry {
    Geometry::Mesh(Mesh {
        vertices: vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]],
        faces: vec![vec![0, 1, 2, 3]],
        normals: vec![],
        colors: vec![],
    })
}

#[test]
fn crossing_takes_a_line_with_both_endpoints_outside_but_window_does_not() {
    let (s, c, v) = fixture(line([-8., 0., 0.], [8., 0., 0.]));
    assert_eq!(s.select_band(&c, v, band(), false).unwrap().len(), 1);
    assert!(s.select_band(&c, v, band(), true).unwrap().is_empty());
    let miss = SelectionRect {
        min: [230., 100.],
        max: [250., 120.],
    };
    assert!(s.select_band(&c, v, miss, false).unwrap().is_empty());
}
#[test]
fn crossing_inside_a_large_face_does_not_need_a_vertex_or_edge_inside() {
    let (s, c, v) = fixture(quad());
    let rect = SelectionRect {
        min: [235., 240.],
        max: [245., 250.],
    };
    assert_eq!(s.select_band(&c, v, rect, false).unwrap().len(), 1);
    assert!(s.select_band(&c, v, rect, true).unwrap().is_empty());
}
#[test]
fn a_clipped_segment_still_crosses_but_cannot_be_window_selected() {
    let (s, mut c, v) = fixture(line([-5., 0., 11.], [5., 0., 0.]));
    c.near_mm = 1.;
    let rect = SelectionRect {
        min: [230., 250.],
        max: [250., 290.],
    };
    assert_eq!(s.select_band(&c, v, rect, false).unwrap().len(), 1);
    assert!(
        s.select_band(
            &c,
            v,
            SelectionRect {
                min: [0., 0.],
                max: [500., 500.]
            },
            true
        )
        .unwrap()
        .is_empty()
    );
}
#[test]
fn hidden_and_locked_primitives_are_excluded_from_both_band_modes() {
    let (mut s, c, v) = fixture(quad());
    s.objects[0].visible = false;
    assert!(s.select_band(&c, v, band(), false).unwrap().is_empty());
    s.objects[0].visible = true;
    s.objects[0].selectable = false;
    assert!(s.select_band(&c, v, band(), false).unwrap().is_empty());
    assert!(
        s.select_band(
            &c,
            v,
            SelectionRect {
                min: [0., 0.],
                max: [500., 500.]
            },
            true
        )
        .unwrap()
        .is_empty()
    );
}
#[test]
fn a_quad_diagonal_is_a_face_hit_but_never_a_document_edge() {
    let (s, c, v) = fixture(quad());
    let screen = c.project([0.; 3], v).unwrap().unwrap().screen;
    assert!(
        s.pick_kind(screen, &c, v, 1., PickIntent::Select, PickKind::Edge)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        s.pick_kind(screen, &c, v, 1., PickIntent::Select, PickKind::Face)
            .unwrap()
            .unwrap()
            .subset,
        Subelement::MeshFace(0)
    );
}
#[test]
fn perspective_edge_hit_reports_a_world_point_on_the_pointer_ray() {
    let (s, _, v) = fixture(Geometry::Mesh(Mesh {
        vertices: vec![[-2., 0., 2.], [2., 0., -4.], [0., 3., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
    }));
    let c = Camera::perspective([0., 0., 10.], [0.; 3], [0., 1., 0.], 45_f64.to_radians()).unwrap();
    let pointer = c.project([0., 0., -1.], v).unwrap().unwrap().screen;
    let hit = s
        .pick_kind(pointer, &c, v, 0.001, PickIntent::Select, PickKind::Edge)
        .unwrap()
        .unwrap();
    assert_eq!(hit.subset, Subelement::MeshEdge { a: 0, b: 1 });
    assert!(
        hit.world_point[0].abs() < 1e-9 && (hit.world_point[2] + 1.).abs() < 1e-9,
        "{:?}",
        hit.world_point
    );
}

#[test]
fn an_edge_with_one_clipped_endpoint_keeps_its_document_key() {
    let (s, mut c, v) = fixture(Geometry::Mesh(Mesh {
        vertices: vec![[-5., 0., 11.], [5., 0., 0.], [5., 4., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
    }));
    c.near_mm = 1.;
    let pointer = c.project([0., 0., 5.5], v).unwrap().unwrap().screen;
    let hit = s
        .pick_kind(pointer, &c, v, 0.001, PickIntent::Select, PickKind::Edge)
        .unwrap()
        .unwrap();
    assert_eq!(hit.subset, Subelement::MeshEdge { a: 0, b: 1 });
    assert!((hit.world_point[2] - 5.5).abs() < 1e-9);
}
