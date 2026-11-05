// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use om9_addon_geom::mesh::cuboid;
use serde_json::Value;

fn event(command: &str, input: Input, selection: Vec<Selected>, settings: Value) -> Event {
    Event {
        command: command.into(),
        input,
        state: Value::Null,
        plane: Plane::default(),
        selection,
        pick: None,
        settings: serde_json::from_value(settings).unwrap(),
    }
}

fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection, json!({}));
    let mut r = Top11::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Top11::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}

fn polyline(id: &str, points: Vec<Vec3>, closed: bool) -> Selected {
    let dense = om9_surfaces::resample(
        &{
            let mut p = points.clone();
            if closed {
                p.push(p[0]);
            }
            p
        },
        false,
        64,
    );
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: id.into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points: dense,
        closed,
        exact: Some(Geo::Polyline { points, closed }),
        ..Default::default()
    }
}

fn solid(id: &str, m: Mesh) -> Selected {
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: id.into(),
        bbox: m.bbox(),
        mesh: Some(SelectedMesh {
            vertices: m.vertices,
            triangles: m.triangles,
        }),
        ..Default::default()
    }
}

#[test]
fn control_points_move_one_point() {
    let c = polyline(
        "p",
        vec![[0.0; 3], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        false,
    );
    let r = run(
        CONTROL_POINTS,
        vec![c],
        vec![pt([5.1, 0.1, 0.0]), pt([5.0, 3.0, 0.0]), Input::Enter],
    );
    match &r.replace[0].geo {
        Geo::Polyline { points, .. } => assert_eq!(points[1], [5.0, 3.0, 0.0]),
        g => panic!("{g:?}"),
    }
    // A surface's control net.
    let rows: Vec<Vec<Vec3>> = (0..3)
        .map(|i| (0..3).map(|j| [i as f64, j as f64, 0.0]).collect())
        .collect();
    let g = om9_addon_geom::fit::surface_on_grid(&rows, [2, 2]).unwrap();
    let s = Selected {
        id: "s".into(),
        kind: "surface".into(),
        exact: Some(g),
        ..Default::default()
    };
    let r = run(
        CONTROL_POINTS,
        vec![s],
        vec![
            pt([1.0, 1.0, 0.0]),
            Input::Text {
                text: "1,1,2".into(),
            },
            Input::Enter,
        ],
    );
    assert!(surface_points(&r.replace[0].geo).contains(&[1.0, 1.0, 2.0]));
}

#[test]
fn explode_and_join() {
    let sq = polyline(
        "q",
        vec![[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        true,
    );
    let r = run(EXPLODE, vec![sq], vec![Input::Enter]);
    assert_eq!(r.create.len(), 4);
    assert_eq!(r.delete.len(), 1);
    let mut two = cuboid([0.0; 3], [1.0; 3]);
    two.append(&cuboid([5.0; 3], [6.0; 3]));
    let r = run(EXPLODE, vec![solid("m", two)], vec![Input::Enter]);
    assert_eq!(r.create.len(), 2);
    let a = polyline("a", vec![[0.0; 3], [1.0, 0.0, 0.0]], false);
    let b = polyline("b", vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]], false);
    let r = run(JOIN, vec![a, b], vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    assert_eq!(r.delete.len(), 2);
}

#[test]
fn split_a_curve_and_a_mesh() {
    let l = polyline("l", vec![[0.0; 3], [10.0, 0.0, 0.0]], false);
    let r = run(SPLIT, vec![l], vec![pt([4.0, 0.0, 0.0])]);
    assert_eq!(r.replace.len(), 1);
    assert_eq!(r.create.len(), 1);
    let b = solid("b", cuboid([0.0; 3], [4.0, 4.0, 4.0]));
    let r = run(
        SPLIT,
        vec![b],
        vec![pt([1.0, -5.0, 0.0]), pt([1.0, 5.0, 0.0])],
    );
    let (m1, m2) = match (&r.replace[0].geo, &r.create[0]) {
        (Geo::Mesh { vertices: a, .. }, Geo::Mesh { vertices: b, .. }) => (a.clone(), b.clone()),
        _ => panic!(),
    };
    let xs = |v: &[Vec3]| {
        (
            v.iter().map(|p| p[0]).fold(f64::MAX, f64::min),
            v.iter().map(|p| p[0]).fold(f64::MIN, f64::max),
        )
    };
    let (a, b) = (xs(&m1), xs(&m2));
    assert!(
        (a.1 - 1.0).abs() < 1e-9 || (b.1 - 1.0).abs() < 1e-9,
        "{a:?} {b:?}"
    );
}

#[test]
fn trim_a_cross() {
    let h = polyline("h", vec![[-5.0, 0.0, 0.0], [5.0, 0.0, 0.0]], false);
    let v = polyline("v", vec![[0.0, -5.0, 0.0], [0.0, 5.0, 0.0]], false);
    let r = run(TRIM, vec![h, v], vec![pt([3.0, 0.0, 0.0]), Input::Enter]);
    assert_eq!(r.replace.len(), 1);
    match &r.replace[0].geo {
        Geo::Polyline { points, .. } => {
            let xmax = points.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
            assert!(xmax.abs() < 1e-9, "the right part is gone: {xmax}");
        }
        g => panic!("{g:?}"),
    }
}

#[test]
fn ring_rail_and_every_command_starts() {
    let r = run(RING_RAIL, vec![], vec![Input::Enter]);
    assert!(matches!(r.create[0], Geo::Circle { .. }));
    for c in Top11::SESSION_COMMANDS {
        let r = Top11::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.prompt.is_empty(), "{c}");
    }
}
