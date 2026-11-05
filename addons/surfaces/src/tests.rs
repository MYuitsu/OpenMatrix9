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

fn run(command: &str, selection: Vec<Selected>, settings: Value, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection, settings);
    let mut r = Surfaces::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Surfaces::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}

fn curve(id: &str, points: Vec<Vec3>, closed: bool) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: id.into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points,
        closed,
        ..Default::default()
    }
}

fn circle_pts(c: Vec3, r: f64, plane: usize, n: usize) -> Vec<Vec3> {
    (0..n)
        .map(|k| {
            let a = TAU * k as f64 / n as f64;
            match plane {
                0 => add(c, [r * a.cos(), r * a.sin(), 0.0]),
                1 => add(c, [r * a.cos(), 0.0, r * a.sin()]),
                _ => add(c, [0.0, r * a.cos(), r * a.sin()]),
            }
        })
        .collect()
}

fn line(id: &str, a: Vec3, b: Vec3) -> Selected {
    curve(
        id,
        (0..=20)
            .map(|k| add(a, scale(sub(b, a), k as f64 / 20.0)))
            .collect(),
        false,
    )
}

fn srf_sel(id: &str, g: Geo) -> Selected {
    let s = Srf::of(&g).unwrap();
    let m = grid_mesh(&grid(&s, 20, 20));
    Selected {
        id: id.into(),
        kind: "surface".into(),
        name: id.into(),
        bbox: m.bbox(),
        mesh: Some(SelectedMesh {
            vertices: m.vertices,
            triangles: m.triangles,
        }),
        exact: Some(g),
        ..Default::default()
    }
}

fn flat(id: &str, x0: f64, x1: f64, y0: f64, y1: f64, z: f64) -> Selected {
    let rows: Vec<Vec<Vec3>> = (0..4)
        .map(|i| {
            (0..4)
                .map(|j| {
                    [
                        x0 + (x1 - x0) * i as f64 / 3.0,
                        y0 + (y1 - y0) * j as f64 / 3.0,
                        z,
                    ]
                })
                .collect()
        })
        .collect();
    srf_sel(id, surface_on_grid(&rows, [3, 3]).unwrap())
}

#[test]
fn sweep_a_tube_around_a_ring() {
    // Rail: a circle of radius 10 in the XZ plane; profile: a circle of radius 1 around the
    // top of the rail, across it.
    let rail = curve("r", circle_pts([0.0; 3], 10.0, 1, 128), true);
    let prof = curve(
        "p",
        circle_pts([10.0, 0.0, 0.0], 1.0, 0, 64)
            .iter()
            .map(|p| [10.0 + (p[0] - 10.0), p[1], 0.0])
            .map(|p| [p[0], p[1], 0.0])
            .collect(),
        true,
    );
    // The profile stands across the rail at (10, 0, 0): in the plane x-y? the rail runs along
    // z there, so the profile lies in z = 0.
    let r = run(SWEEP1, vec![rail, prof], json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=20 {
        for j in 0..=20 {
            let p = s.at_st(i as f64 / 20.0, j as f64 / 20.0);
            // Distance from the rail circle.
            let rho = (p[0] * p[0] + p[2] * p[2]).sqrt();
            let d = ((rho - 10.0).powi(2) + p[1] * p[1]).sqrt();
            assert!((d - 1.0).abs() < 0.02, "{p:?} {d}");
        }
    }
    // A window selection also catches the gem layout circle inside the ring: it does not touch
    // the rail, so it is not a profile (found testing Profile Placer + Profile Sweep on Windows).
    let rail = curve("r", circle_pts([0.0; 3], 10.0, 1, 128), true);
    let prof = curve("p", circle_pts([10.0, 0.0, 0.0], 1.0, 0, 64), true);
    let stray = curve("g", circle_pts([0.0; 3], 5.0, 0, 64), true);
    let r = run(
        PROFILE_SWEEP,
        vec![rail, prof, stray],
        json!({}),
        vec![Input::Enter],
    );
    assert!(
        r.messages.iter().any(|m| m.contains("1 profiles")),
        "{:?}",
        r.messages
    );
}

#[test]
fn loft_patch_and_edge_curves() {
    let a = curve("a", circle_pts([0.0; 3], 2.0, 0, 64), true);
    let b = curve("b", circle_pts([0.0, 0.0, 5.0], 3.0, 0, 64), true);
    let r = run(LOFT, vec![a, b], json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    let mid = s.at_st(0.5, 0.3);
    assert!((mid[2] - 2.5).abs() < 0.05, "{mid:?}");
    assert!(((mid[0].powi(2) + mid[1].powi(2)).sqrt() - 2.5).abs() < 0.05);
    // Four edges of a unit square (a flat patch).
    let edges = vec![
        line("1", [0.0; 3], [1.0, 0.0, 0.0]),
        line("2", [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
        line("3", [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]),
        line("4", [0.0, 1.0, 0.0], [0.0; 3]),
    ];
    let r = run(EDGE_CURVES, edges, json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(
        dist(s.at_st(0.5, 0.5), [0.5, 0.5, 0.0]) < 1e-6,
        "{:?}",
        s.at_st(0.5, 0.5)
    );
    // Two rails: a ruled surface.
    let r = run(
        EDGE_CURVES,
        vec![
            line("1", [0.0; 3], [4.0, 0.0, 0.0]),
            line("2", [0.0, 2.0, 1.0], [4.0, 2.0, 1.0]),
        ],
        json!({}),
        vec![Input::Enter],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    let corners = [s.at_st(0.0, 0.0), s.at_st(1.0, 1.0)];
    assert!(
        corners.iter().any(|c| dist(*c, [0.0; 3]) < 1e-6),
        "{corners:?}"
    );
    let r = run(
        PATCH,
        vec![curve("c", circle_pts([0.0; 3], 3.0, 0, 64), true)],
        json!({}),
        vec![Input::Enter],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(norm(s.at_st(0.5, 0.5)) < 0.3, "{:?}", s.at_st(0.5, 0.5));
}

#[test]
fn revolve_is_an_exact_cylinder() {
    let prof = line("p", [5.0, 0.0, 0.0], [5.0, 0.0, 10.0]);
    let r = run(
        REVOLVE,
        vec![prof],
        json!({}),
        vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0])],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=40 {
        for j in 0..=4 {
            let p = s.at_st(i as f64 / 40.0, j as f64 / 4.0);
            assert!(
                ((p[0] * p[0] + p[1] * p[1]).sqrt() - 5.0).abs() < 1e-9,
                "{p:?}"
            );
        }
    }
    let r = run(
        EXTRUDE_ALL,
        vec![curve("c", circle_pts([0.0; 3], 2.0, 0, 64), true)],
        json!({"ea_distance": 3.0}),
        vec![Input::Enter],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!((s.at_st(1.0, 0.3)[2] - 3.0).abs() < 1e-9);
}

#[test]
fn planar_with_a_hole_and_plane() {
    let outer = curve("o", circle_pts([0.0; 3], 5.0, 0, 64), true);
    let hole = curve("h", circle_pts([0.0; 3], 2.0, 0, 32), true);
    let r = run(PLANAR, vec![outer, hole], json!({}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    if let Geo::Mesh {
        vertices,
        triangles,
        ..
    } = &r.create[0]
    {
        let area: f64 = triangles
            .iter()
            .map(|t| {
                norm(cross(
                    sub(vertices[t[1] as usize], vertices[t[0] as usize]),
                    sub(vertices[t[2] as usize], vertices[t[0] as usize]),
                )) / 2.0
            })
            .sum();
        let want = PI * 25.0 - PI * 4.0;
        assert!((area / want - 1.0).abs() < 0.02, "{area} vs {want}");
    }
    let r = run(
        PLANE,
        vec![],
        json!({}),
        vec![pt([0.0; 3]), pt([2.0, 3.0, 0.0])],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(dist(s.at_st(1.0, 1.0), [2.0, 3.0, 0.0]) < 1e-12);
}

#[test]
fn offset_tween_rebuild_and_split() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let r = run(
        OFFSET,
        vec![a.clone()],
        json!({"os_distance": 1.5}),
        vec![Input::Enter],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!((s.at_st(0.3, 0.7)[2].abs() - 1.5).abs() < 1e-6);
    let b = flat("b", 0.0, 4.0, 0.0, 4.0, 2.0);
    let r = run(
        TWEEN,
        vec![a.clone(), b],
        json!({"tw_count": 1}),
        vec![Input::Enter],
    );
    assert!((Srf::of(&r.create[0]).unwrap().at_st(0.5, 0.5)[2] - 1.0).abs() < 1e-6);
    let r = run(
        REBUILD,
        vec![a.clone()],
        json!({"sr_u_points": 6, "sr_v_points": 5}),
        vec![Input::Enter],
    );
    if let Geo::Surface { size, .. } = &r.replace[0].geo {
        assert_eq!(*size, [6, 5]);
    }
    let r = run(
        SPLIT_ISO,
        vec![a.clone()],
        json!({}),
        vec![pt([1.0, 2.0, 0.0])],
    );
    let first = Srf::of(&r.replace[0].geo).unwrap();
    let second = Srf::of(&r.create[0]).unwrap();
    assert!(
        (first.at_st(1.0, 0.5)[0] - 1.0).abs() < 0.01
            && (second.at_st(0.0, 0.5)[0] - 1.0).abs() < 0.01
    );
    let r = run(
        EXTEND,
        vec![a],
        json!({}),
        vec![num(2.0), pt([4.0, 2.0, 0.0])],
    );
    let ext = Srf::of(&r.replace[0].geo).unwrap();
    let xmax = (0..=10)
        .map(|k| ext.at_st(1.0, k as f64 / 10.0)[0])
        .fold(f64::MIN, f64::max)
        .max(
            (0..=10)
                .map(|k| ext.at_st(0.0, k as f64 / 10.0)[0])
                .fold(f64::MIN, f64::max),
        );
    assert!((xmax - 6.0).abs() < 1e-3, "{xmax}");
}

fn num(v: f64) -> Input {
    Input::Text { text: fmt(v) }
}

#[test]
fn blend_merge_and_match_edges() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let b = flat("b", 6.0, 10.0, 0.0, 4.0, 1.0);
    let r = run(
        BLEND,
        vec![a.clone(), b.clone()],
        json!({}),
        vec![Input::Enter],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    let ends = [s.at_st(0.0, 0.5), s.at_st(1.0, 0.5)];
    assert!(
        ends.iter().any(|p| (p[0] - 4.0).abs() < 1e-6)
            && ends.iter().any(|p| (p[0] - 6.0).abs() < 1e-6),
        "{ends:?}"
    );
    let c = flat("c", 4.0, 8.0, 0.0, 4.0, 0.0);
    let r = run(MERGE, vec![a.clone(), c], json!({}), vec![Input::Enter]);
    let m = Srf::of(&r.replace[0].geo).unwrap();
    let xs: Vec<f64> = [
        m.at_st(0.0, 0.0)[0],
        m.at_st(1.0, 0.0)[0],
        m.at_st(0.0, 1.0)[0],
        m.at_st(1.0, 1.0)[0],
    ]
    .to_vec();
    assert!(
        xs.iter().any(|x| x.abs() < 1e-6) && xs.iter().any(|x| (x - 8.0).abs() < 1e-6),
        "{xs:?}"
    );
    assert_eq!(r.delete.len(), 1);
    let r = run(
        MATCH,
        vec![a, b],
        json!({"sm_continuity": "position"}),
        vec![Input::Enter],
    );
    let m = Srf::of(&r.replace[0].geo).unwrap();
    let pts: Vec<Vec3> = (0..=4)
        .flat_map(|i| (0..=4).map(move |j| (i, j)))
        .map(|(i, j)| m.at_st(i as f64 / 4.0, j as f64 / 4.0))
        .collect();
    assert!(
        pts.iter()
            .any(|p| (p[0] - 6.0).abs() < 1e-3 && (p[2] - 1.0).abs() < 1e-3),
        "the matched edge reaches b"
    );
}

#[test]
fn fillet_and_chamfer_two_planes() {
    // A floor and a wall meeting along y at x = 0 (the corner region is x > 0, z > 0).
    let floor = Selected {
        id: "f".into(),
        kind: "mesh".into(),
        mesh: Some(
            om9_addon_geom::fit::grid_mesh(
                &(0..3)
                    .map(|i| {
                        (0..3)
                            .map(|j| [-5.0 + 7.5 * i as f64, -5.0 + 5.0 * j as f64, 0.0])
                            .collect()
                    })
                    .collect::<Vec<_>>(),
            )
            .into_sel(),
        ),
        ..Default::default()
    };
    let wall = Selected {
        id: "w".into(),
        kind: "mesh".into(),
        mesh: Some(
            om9_addon_geom::fit::grid_mesh(
                &(0..3)
                    .map(|i| {
                        (0..3)
                            .map(|j| [0.0, -5.0 + 5.0 * j as f64, -5.0 + 7.5 * i as f64])
                            .collect()
                    })
                    .collect::<Vec<_>>(),
            )
            .into_sel(),
        ),
        ..Default::default()
    };
    let fm = floor.mesh.clone().unwrap();
    let wm = wall.mesh.clone().unwrap();
    // Choose the sides so the ball sits in x > 0, z > 0.
    let (_, n1) = mesh_point(&fm, [1.0, 0.0, 0.0]).unwrap();
    let (_, n2) = mesh_point(&wm, [0.0, 0.0, 1.0]).unwrap();
    let flip = [n1[2] > 0.0, n2[0] > 0.0];
    let rows = fillet_rows(&fm, &wm, &|_| 1.0, flip, false, 16).unwrap();
    for r in &rows {
        for p in r {
            // On the circle of radius 1 around (1, y, 1).
            assert!(
                ((p[0] - 1.0).powi(2) + (p[2] - 1.0).powi(2)).sqrt() - 1.0 < 1e-6,
                "{p:?}"
            );
        }
        assert!(
            dist(r[0], [1.0, r[0][1], 0.0]) < 1e-6 || dist(r[0], [0.0, r[0][1], 1.0]) < 1e-6,
            "{:?}",
            r[0]
        );
    }
    let r = run(
        CHAMFER,
        vec![floor, wall],
        json!({"fs_radius": 1.0, "fs_flip_first": flip[0], "fs_flip_second": flip[1]}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 1);
}

trait IntoSel {
    fn into_sel(self) -> SelectedMesh;
}

impl IntoSel for Mesh {
    fn into_sel(self) -> SelectedMesh {
        SelectedMesh {
            vertices: self.vertices,
            triangles: self.triangles,
        }
    }
}

#[test]
fn drape_unroll_and_cutting_plane() {
    let block = Selected {
        id: "b".into(),
        kind: "mesh".into(),
        mesh: Some(cuboid([-2.0, -2.0, 0.0], [2.0, 2.0, 3.0]).into_sel()),
        bbox: [[-2.0, -2.0, 0.0], [2.0, 2.0, 3.0]],
        ..Default::default()
    };
    let r = run(
        DRAPE,
        vec![block.clone()],
        json!({"dr_density": 12}),
        vec![pt([-1.0, -1.0, 0.0]), pt([1.0, 1.0, 0.0])],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!((s.at_st(0.5, 0.5)[2] - 3.0).abs() < 1e-6);
    // A half cylinder unrolls to a rectangle π r × length.
    let rows: Vec<Vec<Vec3>> = (0..2)
        .map(|i| {
            (0..=32)
                .map(|j| {
                    let a = PI * j as f64 / 32.0;
                    [2.0 * a.cos(), 2.0 * a.sin(), 5.0 * i as f64]
                })
                .collect()
        })
        .collect();
    let cyl = srf_sel("c", surface_through(&rows, [1, 3]).unwrap());
    let r = run(UNROLL, vec![cyl], json!({}), vec![Input::Enter]);
    if let Geo::Mesh { vertices, .. } = &r.create[0] {
        let xs = vertices.iter().map(|v| v[0]).fold(f64::MIN, f64::max)
            - vertices.iter().map(|v| v[0]).fold(f64::MAX, f64::min);
        let ys = vertices.iter().map(|v| v[1]).fold(f64::MIN, f64::max)
            - vertices.iter().map(|v| v[1]).fold(f64::MAX, f64::min);
        let (a, b) = (xs.max(ys), xs.min(ys));
        assert!(
            (a - 2.0 * PI).abs() < 0.05 && (b - 5.0).abs() < 0.05,
            "{xs} × {ys}"
        );
    }
    let r = run(
        CUTTING_PLANE,
        vec![block],
        json!({}),
        vec![pt([-5.0, 0.0, 0.0]), pt([5.0, 0.0, 0.0])],
    );
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(s.at_st(0.5, 0.5)[1].abs() < 1e-9);
}

#[test]
fn soft_edit_seam_tangent_refit() {
    let a = flat("a", 0.0, 10.0, 0.0, 10.0, 0.0);
    let r = run(
        SOFT_EDIT,
        vec![a.clone()],
        json!({}),
        vec![num(0.3), pt([5.0, 5.0, 0.0]), pt([5.0, 5.0, 2.0])],
    );
    let s = Srf::of(&r.replace[0].geo).unwrap();
    assert!((s.at_st(0.5, 0.5)[2] - 2.0).abs() < 0.05 && s.at_st(0.0, 0.0)[2].abs() < 1e-6);
    let rows: Vec<Vec<Vec3>> = (0..=24)
        .map(|i| {
            let a = TAU * i as f64 / 24.0;
            (0..3)
                .map(|j| [3.0 * a.cos(), 3.0 * a.sin(), j as f64])
                .collect()
        })
        .collect();
    let tube = srf_sel("t", surface_through(&rows, [3, 2]).unwrap());
    let r = run(SEAM, vec![tube], json!({}), vec![pt([-3.0, 0.0, 1.0])]);
    let s = Srf::of(&r.replace[0].geo).unwrap();
    assert!(
        dist(s.at_st(0.0, 0.5), [-3.0, 0.0, 1.0]) < 0.05,
        "{:?}",
        s.at_st(0.0, 0.5)
    );
    let r = run(
        SET_TANGENT,
        vec![a.clone()],
        json!({}),
        vec![pt([10.0, 5.0, 0.0]), pt([9.0, 5.0, -1.0])],
    );
    assert_eq!(r.replace.len(), 1);
    let r = run(
        REFIT,
        vec![a],
        json!({"srf_tolerance": 0.001}),
        vec![Input::Enter],
    );
    assert_eq!(r.replace.len(), 1);
}

#[test]
fn every_command_starts() {
    for c in Surfaces::SESSION_COMMANDS {
        let r = Surfaces::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
