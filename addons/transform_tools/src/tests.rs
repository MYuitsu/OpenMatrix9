// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::{Plane, SelectedMesh};
use serde_json::Value;

fn event(command: &str, input: Input, selection: Vec<Selected>) -> Event {
    Event {
        command: command.into(),
        input,
        state: Value::Null,
        plane: Plane::default(),
        selection,
        pick: None,
        settings: Default::default(),
    }
}

/// Run a command: start, then the inputs; returns the last reply.
fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection);
    let mut r = TransformTools::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = TransformTools::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}
fn num(v: f64) -> Input {
    Input::Text { text: fmt(v) }
}

fn line(id: &str, a: Vec3, b: Vec3) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: "Line".into(),
        bbox: [
            [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])],
            [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])],
        ],
        points: vec![a, b],
        exact: Some(Geo::Line { start: a, end: b }),
        ..Default::default()
    }
}

fn polyline(id: &str, pts: Vec<Vec3>, closed: bool) -> Selected {
    let mut b = [[f64::MAX; 3], [f64::MIN; 3]];
    for p in &pts {
        for i in 0..3 {
            b[0][i] = b[0][i].min(p[i]);
            b[1][i] = b[1][i].max(p[i]);
        }
    }
    let mut poles = pts.clone();
    if closed {
        poles.push(pts[0]);
    }
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: "Polyline".into(),
        bbox: b,
        points: pts.clone(),
        closed,
        exact: Some(Geo::Nurbs {
            degree: 1,
            knots: om9_addon_geom::clamped_uniform(poles.len(), 1),
            points: poles,
            weights: Vec::new(),
        }),
        ..Default::default()
    }
}

fn box_mesh(id: &str, lo: Vec3, hi: Vec3) -> Selected {
    let m = mesh::cuboid(lo, hi);
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: "Box".into(),
        bbox: [lo, hi],
        mesh: Some(SelectedMesh {
            vertices: m.vertices,
            triangles: m.triangles,
        }),
        ..Default::default()
    }
}

fn first_matrix(r: &Reply) -> [f64; 16] {
    let op = r.transform.as_ref().unwrap();
    op.matrices
        .first()
        .copied()
        .unwrap_or_else(|| op.per_object[0].matrix)
}

fn close(a: Vec3, b: Vec3) -> bool {
    dist(a, b) < 1e-6
}

#[test]
fn matrices() {
    let sel = vec![line("a", [0.0; 3], [1.0, 0.0, 0.0])];
    let r = run(
        NONUNIFORM,
        sel.clone(),
        vec![
            Input::Selection,
            pt([1.0, 1.0, 0.0]),
            num(2.0),
            num(3.0),
            num(1.0),
        ],
    );
    assert!(r.done);
    assert!(close(
        apply(&first_matrix(&r), [2.0, 2.0, 1.0]),
        [3.0, 4.0, 1.0]
    ));
    let r = run(
        ORIENT2,
        sel.clone(),
        vec![
            Input::Selection,
            pt([0.0; 3]),
            pt([1.0, 0.0, 0.0]),
            pt([5.0, 5.0, 0.0]),
            pt([5.0, 7.0, 0.0]),
        ],
    );
    assert!(close(
        apply(&first_matrix(&r), [1.0, 0.0, 0.0]),
        [5.0, 6.0, 0.0]
    ));
    let r = run(
        ORIENT2,
        sel.clone(),
        vec![
            Input::Selection,
            Input::Option {
                name: "Scale".into(),
                value: None,
            },
            pt([0.0; 3]),
            pt([1.0, 0.0, 0.0]),
            pt([5.0, 5.0, 0.0]),
            pt([5.0, 7.0, 0.0]),
        ],
    );
    assert!(
        close(apply(&first_matrix(&r), [1.0, 0.0, 0.0]), [5.0, 7.0, 0.0]),
        "scaled"
    );
    let r = run(
        ORIENT3,
        sel.clone(),
        vec![
            Input::Selection,
            pt([0.0; 3]),
            pt([1.0, 0.0, 0.0]),
            pt([0.0, 1.0, 0.0]),
            pt([0.0; 3]),
            pt([0.0, 0.0, 1.0]),
            pt([1.0, 0.0, 0.0]),
        ],
    );
    assert!(close(
        apply(&first_matrix(&r), [0.0, 1.0, 0.0]),
        [1.0, 0.0, 0.0]
    ));
    let r = run(
        SHEAR,
        sel.clone(),
        vec![
            Input::Selection,
            pt([0.0; 3]),
            pt([1.0, 0.0, 0.0]),
            num(45.0),
        ],
    );
    assert!(close(
        apply(&first_matrix(&r), [0.0, 2.0, 0.0]),
        [2.0, 2.0, 0.0]
    ));
    let r = run(
        MOVE_UVN,
        sel.clone(),
        vec![Input::Selection, num(1.0), num(2.0), num(3.0)],
    );
    assert!(close(apply(&first_matrix(&r), [0.0; 3]), [1.0, 2.0, 3.0]));
    let mut e = event(SYMMETRY, Input::Selection, sel.clone());
    let r = TransformTools::step(&e).unwrap();
    e.state = r.state;
    e.input = Input::Enter;
    let r = TransformTools::step(&e).unwrap();
    assert!(r.transform.as_ref().unwrap().keep_original);
    assert!(close(
        apply(&first_matrix(&r), [2.0, 1.0, 0.0]),
        [-2.0, 1.0, 0.0]
    ));
}

#[test]
fn stones_turn_down_about_their_own_girdle() {
    let gem = om9_addon_geom::gems::Gem::new(om9_addon_geom::gems::Cut::Round, 2.0, 2.0);
    let f = Frame::new([5.0, 0.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let s = Selected {
        id: "g".into(),
        kind: "mesh".into(),
        gem: Some(gem.record(&f, "t", "diamond")),
        ..Default::default()
    };
    let r = TransformTools::step(&event(DOWN_FACING, Input::Enter, vec![s])).unwrap();
    let m = first_matrix(&r);
    assert!(
        close(apply(&m, [5.0, 0.0, 2.0]), [5.0, 0.0, 0.0]),
        "table flips below the girdle"
    );
    assert!(
        close(apply(&m, [5.0, 0.0, 1.0]), [5.0, 0.0, 1.0]),
        "girdle center stays"
    );
}

#[test]
fn bend_keeps_the_spine_length() {
    // A 10 mm bar bent 180° toward +z becomes a half circle of radius 10/π.
    let p = bend_point(
        [10.0, 0.0, 0.0],
        [0.0; 3],
        [10.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        180.0,
    );
    let r = 10.0 / PI;
    assert!(close(p, [0.0, 0.0, 2.0 * r]), "{p:?}");
    let mid = bend_point(
        [5.0, 0.0, 0.0],
        [0.0; 3],
        [10.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        180.0,
    );
    assert!(close(mid, [r, 0.0, r]), "{mid:?}");
    let sel = vec![line("a", [0.0; 3], [10.0, 0.0, 0.0])];
    let r = run(
        BEND,
        sel,
        vec![
            Input::Selection,
            pt([0.0; 3]),
            pt([10.0, 0.0, 0.0]),
            num(90.0),
        ],
    );
    assert!(r.done);
    let Geo::Nurbs { points, .. } = &r.replace[0].geo else {
        panic!("{:?}", r.replace[0].geo)
    };
    assert!(
        points.last().unwrap()[2] > 1.0,
        "the end lifted toward the normal"
    );
}

#[test]
fn taper_twist_maelstrom_soft_move() {
    let p = taper_point([10.0, 2.0, 0.0], [0.0; 3], [10.0, 0.0, 0.0], 1.0, 0.5);
    assert!(close(p, [10.0, 1.0, 0.0]));
    let p = twist_point([10.0, 1.0, 0.0], [0.0; 3], [10.0, 0.0, 0.0], 90.0);
    assert!(close(p, [10.0, 0.0, 1.0]), "{p:?}");
    let p = maelstrom_point([1.0, 0.0, 0.0], [0.0; 3], [0.0, 0.0, 1.0], 2.0, 90.0);
    let expected = 90.0f64.to_radians() * 0.25;
    assert!(close(p, [expected.cos(), expected.sin(), 0.0]));
    assert_eq!(
        maelstrom_point([3.0, 0.0, 0.0], [0.0; 3], [0.0, 0.0, 1.0], 2.0, 90.0),
        [3.0, 0.0, 0.0]
    );
    let p = soft_move_point([0.0; 3], [0.0; 3], [0.0, 0.0, 2.0], 5.0);
    assert!(close(p, [0.0, 0.0, 2.0]));
    let far = soft_move_point([6.0, 0.0, 0.0], [0.0; 3], [0.0, 0.0, 2.0], 5.0);
    assert_eq!(far, [6.0, 0.0, 0.0]);
    // Meshes deform through their triangles.
    let r = run(
        TAPER,
        vec![box_mesh("b", [0.0, -1.0, -1.0], [10.0, 1.0, 1.0])],
        vec![
            Input::Selection,
            pt([0.0; 3]),
            pt([10.0, 0.0, 0.0]),
            num(1.0),
            num(0.5),
        ],
    );
    let Geo::Mesh { vertices, .. } = &r.replace[0].geo else {
        panic!()
    };
    assert!(vertices.iter().any(|v| close(*v, [10.0, 0.5, 0.5])));
}

#[test]
fn set_points_and_project() {
    let sel = vec![line("a", [0.0, 0.0, 1.0], [10.0, 0.0, 5.0])];
    let r = run(
        SET_POINTS,
        sel.clone(),
        vec![Input::Selection, pt([0.0, 0.0, 3.0])],
    );
    let Geo::Nurbs { points, .. } = &r.replace[0].geo else {
        panic!()
    };
    assert!(points.iter().all(|p| (p[2] - 3.0).abs() < 1e-9));
    let r = TransformTools::step(&event(PROJECT_CPLANE, Input::Enter, sel)).unwrap();
    let Geo::Nurbs { points, .. } = &r.replace[0].geo else {
        panic!()
    };
    assert!(points.iter().all(|p| p[2].abs() < 1e-9));
}

#[test]
fn smart_flow_wraps_a_band_around_the_ring() {
    // A straight 10 mm band along x on the CPlane, and a Ø20 ring rail.
    let band = polyline("band", vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], false);
    let ring: Vec<Vec3> = (0..720)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / 720.0;
            [10.0 * a.cos(), 10.0 * a.sin(), 0.0]
        })
        .collect();
    let rail = polyline("rail", ring, true);
    let sel = vec![band.clone(), rail.clone()];
    let mut e = event(SMART_FLOW, Input::Selection, sel.clone());
    let r = TransformTools::step(&e).unwrap();
    assert!(r.prompt.contains("1 objects"), "{}", r.prompt);
    e.state = r.state;
    e.input = Input::Enter;
    let r = TransformTools::step(&e).unwrap();
    assert!(r.done, "{:?}", r.error);
    assert_eq!(r.replace.len(), 1);
    let Geo::Nurbs { points, .. } = &r.replace[0].geo else {
        panic!()
    };
    // Stretched around the whole ring: every point lies on the ring.
    for p in points {
        assert!(
            ((p[0] * p[0] + p[1] * p[1]).sqrt() - 10.0).abs() < 0.01,
            "{p:?}"
        );
    }
    // Rigid: one placement per object, onto the ring.
    let mut e = event(SMART_FLOW_RIGID, Input::Enter, sel);
    e.state = json!({"selected": true, "answers": [], "options": {}});
    let r = TransformTools::step(&e).unwrap();
    let op = r.transform.unwrap();
    assert_eq!(op.per_object.len(), 1);
    let c = apply(&op.per_object[0].matrix, [5.0, 0.0, 0.0]);
    assert!(
        ((c[0] * c[0] + c[1] * c[1]).sqrt() - 10.0).abs() < 0.01,
        "{c:?}"
    );
}

#[test]
fn arrays_along_curves_and_surfaces() {
    let obj = box_mesh("o", [-0.5, -0.5, 0.0], [0.5, 0.5, 1.0]);
    let curve = polyline("c", vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], false);
    let r = run(
        ARRAY_CURVE,
        vec![obj.clone(), curve.clone()],
        vec![Input::Selection, num(5.0)],
    );
    let op = r.transform.unwrap();
    assert!(op.keep_original);
    assert_eq!(op.per_object.len(), 5);
    let last = apply(&op.per_object[4].matrix, [0.0, 0.0, 0.5]);
    assert!((last[0] - 10.0).abs() < 1e-6, "{last:?}");
    // A tilted plane: stones land on it, upright to it.
    let plane = Selected {
        id: "s".into(),
        kind: "surface".into(),
        bbox: [[-20.0, -20.0, 0.0], [20.0, 20.0, 4.0]],
        mesh: Some(SelectedMesh {
            vertices: vec![
                [-20.0, -20.0, 0.0],
                [20.0, -20.0, 4.0],
                [20.0, 20.0, 4.0],
                [-20.0, 20.0, 0.0],
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
        }),
        ..Default::default()
    };
    let r = run(
        ARRAY_SURFACE,
        vec![obj.clone(), plane.clone()],
        vec![Input::Selection, num(2.0), num(2.0)],
    );
    let op = r.transform.unwrap();
    assert_eq!(op.per_object.len(), 4);
    let p = apply(&op.per_object[0].matrix, [0.0, 0.0, 0.0]);
    assert!(
        (p[2] - (p[0] + 20.0) / 10.0).abs() < 1e-6,
        "on the plane: {p:?}"
    );
    // Orient on Surface, Copy=Yes (default): every click places a copy until Enter.
    let r = run(
        ORIENT_SURFACE,
        vec![obj.clone(), plane.clone()],
        vec![Input::Selection, pt([0.0, 0.0, 0.0])],
    );
    assert!(!r.done, "still placing: {}", r.prompt);
    assert!(!r.preview.is_empty(), "the placed copy is drawn");
    let r = run(
        ORIENT_SURFACE,
        vec![obj.clone(), plane.clone()],
        vec![
            Input::Selection,
            pt([0.0, 0.0, 0.0]),
            pt([10.0, 0.0, 0.0]),
            pt([-10.0, 5.0, 0.0]),
            Input::Undo,
            Input::Enter,
        ],
    );
    assert!(r.done);
    let op = r.transform.as_ref().unwrap();
    assert!(op.keep_original);
    assert_eq!(op.per_object.len(), 2, "two copies after one Undo");
    let p = apply(&op.per_object[0].matrix, [0.0, 0.0, 0.0]);
    assert!(close(p, [0.0, 0.0, 2.0]), "{p:?}");
    let p = apply(&op.per_object[1].matrix, [0.0, 0.0, 0.0]);
    assert!(close(p, [10.0, 0.0, 3.0]), "{p:?}");
    // Copy=No: one click moves the original and ends; Flip puts it under the surface.
    let r = run(
        ORIENT_SURFACE,
        vec![obj, plane],
        vec![
            Input::Selection,
            Input::Option {
                name: "Copy".into(),
                value: Some("No".into()),
            },
            Input::Option {
                name: "Flip".into(),
                value: None,
            },
            pt([0.0, 0.0, 0.0]),
        ],
    );
    assert!(r.done);
    let op = r.transform.as_ref().unwrap();
    assert!(!op.keep_original);
    let top = apply(&first_matrix(&r), [0.0, 0.0, 1.0]);
    assert!(top[2] < 2.0, "flipped below the surface: {top:?}");
}

#[test]
fn cage_edit_follows_moved_corners() {
    let obj = box_mesh("o", [0.0; 3], [2.0, 2.0, 2.0]);
    let r = TransformTools::step(&event(CREATE_CAGE, Input::Enter, vec![obj.clone()])).unwrap();
    let Geo::Mesh {
        vertices,
        triangles,
        meta,
        ..
    } = &r.create[0]
    else {
        panic!()
    };
    // Lift the top four cage points by 1.
    let mut moved = vertices.clone();
    for v in moved.iter_mut().skip(4) {
        v[2] += 1.0;
    }
    let cage = Selected {
        id: "cage".into(),
        kind: "mesh".into(),
        bbox: [[0.0; 3], [2.0, 2.0, 3.0]],
        gem: meta.clone(),
        mesh: Some(SelectedMesh {
            vertices: moved,
            triangles: triangles.clone(),
        }),
        ..Default::default()
    };
    let r = TransformTools::step(&event(CAGE_EDIT, Input::Enter, vec![obj, cage])).unwrap();
    assert!(r.done, "{:?}", r.error);
    let Geo::Mesh { vertices, .. } = &r.replace[0].geo else {
        panic!()
    };
    let top = vertices.iter().map(|v| v[2]).fold(f64::MIN, f64::max);
    assert!(top > 2.5, "the top of the object rose with the cage: {top}");
}
