// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use std::f64::consts::{PI, TAU};
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
    // Rhino 5 Patch: a fitted surface trimmed to the closed curve.
    let Geo::Trimmed { surface, loops, .. } = &r.create[0] else { panic!("trimmed patch") };
    assert_eq!(loops.len(), 1);
    let s = Srf::of(surface).unwrap();
    assert!(norm(s.at_st(0.5, 0.5)) < 0.3, "{:?}", s.at_st(0.5, 0.5));
}

#[test]
fn revolve_is_an_exact_cylinder() {
    let prof = line("p", [5.0, 0.0, 0.0], [5.0, 0.0, 10.0]);
    let r = run(
        REVOLVE,
        vec![prof],
        json!({}),
        vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0]), Input::Enter],
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
    let Geo::Trimmed { loops, display, .. } = &r.create[0] else {
        panic!("a trimmed plane: {:?}", r.create[0]);
    };
    assert_eq!(loops.len(), 2, "outer and hole");
    if let Some(Geo::Mesh {
        vertices,
        triangles,
        ..
    }) = display.as_deref()
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
    // Rhino 5 order: the edge (click near it), then the extension.
    let r = run(
        EXTEND,
        vec![a],
        json!({}),
        vec![pt([4.0, 2.0, 0.0]), num(2.0)],
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
    let ends = [s.at_st(0.0, 0.5), s.at_st(1.0, 0.5), s.at_st(0.5, 0.0), s.at_st(0.5, 1.0)];
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
        vec![pt([-5.0, 0.0, 0.0]), pt([5.0, 0.0, 0.0]), Input::Enter],
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
        vec![pt([5.0, 5.0, 0.0]), pt([5.0, 5.0, 2.0])],
    );
    // Rhino 5 order: base point, point to move to; the base point lands exactly.
    let s = Srf::of(&r.replace[0].geo).unwrap();
    assert!((s.at_st(0.5, 0.5)[2] - 2.0).abs() < 1e-9 && s.at_st(0.0, 0.0)[2].abs() < 1e-9, "{:?}", s.at_st(0.5, 0.5));
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
        vec![pt([10.0, 5.0, 0.0]), pt([0.0, 0.0, 0.0]), pt([-1.0, 0.0, 1.0])],
    );
    // Rhino 5 order: near the edge, base point, direction point; leaves x = 10 at 45° up.
    assert_eq!(r.replace.len(), 1);
    let s = Srf::of(&r.replace[0].geo).unwrap();
    let ends = [(1.0, 0.5, 1.0 - 1e-5), (0.0, 0.5, 1e-5)];
    assert!(ends.iter().any(|(u, v, u2)| {
        let (a, b) = (s.at_st(*u, *v), s.at_st(*u2, *v));
        (a[0] - 10.0).abs() < 1e-9 && ((b[2] - a[2]) / (a[0] - b[0]) - 1.0).abs() < 1e-3
    }) || [(0.5, 1.0, 1.0 - 1e-5), (0.5, 0.0, 1e-5)].iter().any(|(v, u, u2)| {
        let (a, b) = (s.at_st(*v, *u), s.at_st(*v, *u2));
        (a[0] - 10.0).abs() < 1e-9 && ((b[2] - a[2]) / (a[0] - b[0]) - 1.0).abs() < 1e-3
    }));
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

fn exact_circle(id: &str, c: Vec3, r: f64, x: Vec3, y: Vec3) -> Selected {
    let g = Geo::Circle { center: c, x, y, radius: r };
    let pts: Vec<Vec3> = (0..256).map(|k| om9_addon_geom::eval_geo(&g, k as f64 / 256.0).unwrap()).collect();
    Selected { exact: Some(g), ..curve(id, pts, true) }
}

fn loft_with(sel: Vec<Selected>, style: &str, closed: bool) -> Geo {
    run(LOFT, sel, json!({"loft_style": style, "loft_closed": closed}), vec![Input::Enter]).create.remove(0)
}

/// Rhino 5 Loft keeps its sections: two equal circles make an exact cylinder, and every
/// section of a normal, tight, uniform or straight loft lies exactly on the surface.
#[test]
fn loft_is_exact_through_its_sections_like_rhino5() {
    let (x, y) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let t0 = std::time::Instant::now();
    let g = loft_with(vec![exact_circle("a", [0.0; 3], 5.0, x, y), exact_circle("b", [0.0, 0.0, 10.0], 5.0, x, y)], "normal", false);
    let s = Srf::of(&g).unwrap();
    for i in 0..=100 {
        for j in 0..=100 {
            let p = s.at_st(i as f64 / 100.0, j as f64 / 100.0);
            assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - 5.0).abs() < 1e-9, "{p:?}");
        }
    }
    // Three sections, the middle one wound the other way and started elsewhere: still on the
    // surface, no twist (the middle of the surface keeps a circle's radius 8 there).
    let secs = || vec![
        exact_circle("a", [0.0; 3], 5.0, x, y),
        exact_circle("b", [0.0, 0.0, 5.0], 8.0, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
        exact_circle("c", [0.0, 0.0, 10.0], 5.0, x, y),
    ];
    for style in ["normal", "tight", "uniform", "straight"] {
        let g = loft_with(secs(), style, false);
        let s = Srf::of(&g).unwrap();
        for (z, r) in [(0.0, 5.0), (5.0, 8.0), (10.0, 5.0)] {
            // The isocurve through that section: find u where z matches.
            let u = (0..=2000).map(|i| i as f64 / 2000.0).min_by(|a, b| (s.at_st(*a, 0.3)[2] - z).abs().total_cmp(&(s.at_st(*b, 0.3)[2] - z).abs())).unwrap();
            for j in 0..=40 {
                let p = s.at_st(u, j as f64 / 40.0);
                assert!((p[2] - z).abs() < 1e-6 && ((p[0] * p[0] + p[1] * p[1]).sqrt() - r).abs() < 1e-6, "{style} {p:?}");
            }
        }
        // No twist: a straight line of the surface keeps its angle (x/y sign) from bottom to top.
        let (a, b) = (s.at_st(0.0, 0.1), s.at_st(1.0, 0.1));
        assert!(a[0] * b[0] + a[1] * b[1] > 0.0, "{style} twisted {a:?} {b:?}");
    }
    // Loose: through the first and last only, inside the others' hull.
    let g = loft_with(secs(), "loose", false);
    let s = Srf::of(&g).unwrap();
    assert!((s.at_st(0.0, 0.2)[2]).abs() < 1e-9 && (s.at_st(1.0, 0.2)[2] - 10.0).abs() < 1e-9);
    // Closed loft of four sections: through all, and the same at both ends of u.
    let ring = vec![
        exact_circle("a", [10.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        exact_circle("b", [0.0, 10.0, 0.0], 1.5, [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
        exact_circle("c", [-10.0, 0.0, 0.0], 1.0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        exact_circle("d", [0.0, -10.0, 0.0], 1.5, [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]),
    ];
    let g = loft_with(ring, "normal", true);
    let s = Srf::of(&g).unwrap();
    for j in 0..=20 {
        let v = j as f64 / 20.0;
        assert!(dist(s.at_st(0.0, v), s.at_st(1.0, v)) < 1e-9);
        let p = s.at_st(0.0, v);
        assert!((dist(p, [10.0, 0.0, 0.0]) - 1.0).abs() < 1e-9, "{p:?}");
    }
    // Mixed open sections (line, arc, cubic) are made compatible.
    let line = Selected { exact: Some(Geo::Line { start: [0.0; 3], end: [4.0, 0.0, 0.0] }), ..line("l", [0.0; 3], [4.0, 0.0, 0.0]) };
    let arcg = Geo::Arc { center: [2.0, 0.0, 3.0], x: [-1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 2.0, angle: PI };
    let arc_pts: Vec<Vec3> = (0..=64).map(|k| om9_addon_geom::eval_geo(&arcg, k as f64 / 64.0).unwrap()).collect();
    let arc = Selected { exact: Some(arcg), ..curve("a", arc_pts, false) };
    let g = loft_with(vec![line, arc], "normal", false);
    let s = Srf::of(&g).unwrap();
    for j in 0..=40 {
        let p = s.at_st(1.0, j as f64 / 40.0);
        assert!((dist(p, [2.0, 0.0, 3.0]) - 2.0).abs() < 1e-9, "{p:?}");
    }
    assert!(t0.elapsed().as_secs_f64() < 1.0, "{:?}", t0.elapsed());
}

/// Revolving an exact circle (Rhino 5: rational) gives an exact torus.
#[test]
fn revolve_of_a_circle_is_an_exact_torus() {
    let prof = exact_circle("p", [10.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let r = run(REVOLVE, vec![prof], json!({}), vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0]), Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=60 {
        for j in 0..=60 {
            let p = s.at_st(i as f64 / 60.0, j as f64 / 60.0);
            let rho = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!((((rho - 10.0).powi(2) + p[2] * p[2]).sqrt() - 2.0).abs() < 1e-9, "{p:?}");
        }
    }
}

fn exact_sel(id: &str, g: Geo, closed: bool) -> Selected {
    let pts: Vec<Vec3> = (0..=128).map(|k| om9_addon_geom::eval_geo(&g, k as f64 / 128.0).unwrap()).collect();
    Selected { exact: Some(g), ..curve(id, pts, closed) }
}

/// Rhino 5 EdgeSrf: the surface's edges are exactly the picked curves (any order and
/// direction); two curves are opposite edges.
#[test]
fn edge_curves_keep_their_edges_exactly() {
    // A square of side 10 whose bottom edge is a half circle bulging down, others straight.
    let arc = Geo::Arc { center: [5.0, 0.0, 0.0], x: [-1.0, 0.0, 0.0], y: [0.0, -1.0, 0.0], radius: 5.0, angle: PI };
    let sel = vec![
        exact_sel("top", Geo::Line { start: [0.0, 10.0, 0.0], end: [10.0, 10.0, 0.0] }, false),
        exact_sel("arc", arc.clone(), false),
        exact_sel("left", Geo::Line { start: [0.0, 10.0, 0.0], end: [0.0, 0.0, 0.0] }, false),
        exact_sel("right", Geo::Line { start: [10.0, 0.0, 0.0], end: [10.0, 10.0, 0.0] }, false),
    ];
    let r = run(EDGE_CURVES, sel, json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    // One edge is the arc: every point of it is 5 from (5,0,0), below y = 0.
    let edges = [(0, 0.0), (0, 1.0), (1, 0.0), (1, 1.0)];
    let mut found_arc = false;
    for (dir, at) in edges {
        let pts: Vec<Vec3> = (0..=50).map(|k| { let t = k as f64 / 50.0; if dir == 0 { s.at_st(at, t) } else { s.at_st(t, at) } }).collect();
        if pts.iter().all(|p| (dist(*p, [5.0, 0.0, 0.0]) - 5.0).abs() < 1e-9) {
            found_arc = true;
        }
    }
    assert!(found_arc, "the arc is an edge");
    // Flat inside (all on z = 0).
    for i in 0..=10 { for j in 0..=10 { assert!(s.at_st(i as f64 / 10.0, j as f64 / 10.0)[2].abs() < 1e-12); } }
    // Two curves: a ruled surface between them.
    let r = run(EDGE_CURVES, vec![
        exact_sel("a", Geo::Line { start: [0.0; 3], end: [4.0, 0.0, 0.0] }, false),
        exact_sel("b", Geo::Line { start: [4.0, 2.0, 1.0], end: [0.0, 2.0, 1.0] }, false),
    ], json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    let mid = s.at_st(0.5, 0.5);
    assert!(dist(mid, [2.0, 1.0, 0.5]) < 1e-12, "{mid:?}");
}

fn opt(name: &str, value: Option<&str>) -> Input {
    Input::Option { name: name.into(), value: value.map(str::to_owned) }
}

/// Rhino 5 Revolve: curves, axis start, axis end, revolution angle (Enter = 360, typed, or a
/// click around the axis); FullCircle, StartAngle, Deformable/PointCount, DeleteInput.
#[test]
fn revolve_follows_rhino5_prompts_and_options() {
    let prof = || line("p", [5.0, 0.0, 0.0], [5.0, 0.0, 10.0]);
    let axis = || vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0])];
    let angle_of = |g: &Geo| -> f64 {
        // The end of the turn (u = 1) at the profile's start: its angle around z.
        let s = Srf::of(g).unwrap();
        let p = s.at_st(1.0, 0.0);
        let a = p[1].atan2(p[0]).to_degrees();
        if a <= 1e-6 { a + 360.0 } else { a }
    };
    // The prompt asks for the angle after the axis, with 360 as default.
    let mut e = event(REVOLVE, Input::Start, vec![prof()], json!({}));
    let mut r = Surfaces::step(&e).unwrap();
    for i in axis() {
        e.state = r.state.clone();
        e.input = i;
        r = Surfaces::step(&e).unwrap();
    }
    assert!(!r.done && r.prompt.contains("Revolution angle") && r.prompt.contains("360"), "{}", r.prompt);
    // The pointer turns the preview: at (0, 5, 0) it shows a quarter turn.
    e.state = r.state.clone();
    e.input = Input::Hover { point: [0.0, 5.0, 0.0] };
    let h = Surfaces::step(&e).unwrap();
    assert!((angle_of(&h.preview[0]) - 90.0).abs() < 1e-6);
    let mut steps = axis();
    steps.push(text("90"));
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    assert!((angle_of(&r.create[0]) - 90.0).abs() < 1e-9);
    // A click at (0, 5, 0) around the z axis: 90 degrees.
    let mut steps = axis();
    steps.push(pt([0.0, 5.0, 0.0]));
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    assert!((angle_of(&r.create[0]) - 90.0).abs() < 1e-6);
    // FullCircle wins over a typed angle; StartAngle turns the start.
    let mut steps = axis();
    steps.insert(0, opt("FullCircle", Some("Yes")));
    steps.push(text("45"));
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(dist(s.at_st(0.0, 0.0), s.at_st(1.0, 0.0)) < 1e-9, "full turn");
    let mut steps = axis();
    steps.insert(0, opt("StartAngle", Some("30")));
    steps.push(text("60"));
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    let s = Srf::of(&r.create[0]).unwrap();
    let p = s.at_st(0.0, 0.0);
    assert!((p[1].atan2(p[0]).to_degrees() - 30.0).abs() < 1e-6, "{p:?}");
    assert!((angle_of(&r.create[0]) - 90.0).abs() < 1e-6);
    // Deformable: degree 3, not rational, PointCount points around; still near the cylinder.
    let mut steps = axis();
    steps.insert(0, opt("Deformable", Some("Yes")));
    steps.insert(1, opt("PointCount", Some("12")));
    steps.push(Input::Enter);
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    match &r.create[0] {
        Geo::Surface { degree, weights, size, .. } => {
            assert_eq!((degree[0], size[0]), (3, 12));
            assert!(weights.iter().all(|w| (*w - 1.0).abs() < 1e-12));
        }
        other => panic!("{other:?}"),
    }
    // DeleteInput removes the profile in the same step.
    let mut steps = axis();
    steps.insert(0, opt("DeleteInput", Some("Yes")));
    steps.push(Input::Enter);
    let r = run(REVOLVE, vec![prof()], json!({}), steps);
    assert_eq!(r.delete, vec!["p".to_string()]);
}

fn text(t: &str) -> Input {
    Input::Text { text: t.into() }
}

/// Rhino 5 Plane draws its rectangle like Rectangle (corner/length/width, 3Point, Vertical,
/// Center) and has Deformable (degrees and point counts) instead of Rounded.
#[test]
fn plane_follows_rhino5_rectangle_and_deformable() {
    let corners = |g: &Geo| {
        let s = Srf::of(g).unwrap();
        [s.at_st(0.0, 0.0), s.at_st(1.0, 0.0), s.at_st(1.0, 1.0), s.at_st(0.0, 1.0)]
    };
    let has = |c: &[Vec3; 4], p: Vec3| c.iter().any(|q| dist(*q, p) < 1e-9);
    // Typed length then width.
    let r = run(PLANE, vec![], json!({}), vec![pt([1.0, 1.0, 0.0]), text("4"), text("2")]);
    let c = corners(&r.create[0]);
    let size = (dist(c[0], c[1]), dist(c[1], c[2]));
    assert!(((size.0 - 4.0).abs() < 1e-9 && (size.1 - 2.0).abs() < 1e-9) || ((size.0 - 2.0).abs() < 1e-9 && (size.1 - 4.0).abs() < 1e-9), "{size:?}");
    // Center: corners symmetric about the center.
    let r = run(PLANE, vec![], json!({}), vec![opt("Center", None), pt([0.0; 3]), pt([3.0, 2.0, 0.0])]);
    let c = corners(&r.create[0]);
    assert!(has(&c, [-3.0, -2.0, 0.0]) && has(&c, [3.0, 2.0, 0.0]));
    // Vertical: stands on the edge, up from the CPlane.
    let r = run(PLANE, vec![], json!({}), vec![opt("Vertical", None), pt([0.0; 3]), pt([5.0, 0.0, 0.0]), text("3")]);
    let c = corners(&r.create[0]);
    assert!(c.iter().any(|p| (p[2].abs() - 3.0).abs() < 1e-9), "{c:?}");
    // 3Point: edge then width.
    let r = run(PLANE, vec![], json!({}), vec![opt("3Point", None), pt([0.0; 3]), pt([4.0, 0.0, 0.0]), pt([2.0, 2.0, 0.0])]);
    let c = corners(&r.create[0]);
    assert!(has(&c, [0.0, 2.0, 0.0]) && has(&c, [4.0, 2.0, 0.0]), "{c:?}");
    // Deformable: degree 3 × 2, 6 × 5 poles, still the same flat rectangle.
    let r = run(PLANE, vec![], json!({}), vec![
        opt("Deformable", Some("Yes")), opt("UDegree", Some("3")), opt("VDegree", Some("2")),
        opt("UPointCount", Some("6")), opt("VPointCount", Some("5")),
        pt([0.0; 3]), pt([6.0, 4.0, 0.0]),
    ]);
    match &r.create[0] {
        Geo::Surface { degree, size, .. } => assert_eq!((*degree, *size), ([3, 2], [6, 5])),
        other => panic!("{other:?}"),
    }
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=10 {
        for j in 0..=10 {
            let p = s.at_st(i as f64 / 10.0, j as f64 / 10.0);
            assert!(p[2].abs() < 1e-12 && p[0] > -1e-9 && p[0] < 6.0 + 1e-9 && p[1] > -1e-9 && p[1] < 4.0 + 1e-9);
        }
    }
    assert!(dist(s.at_st(0.5, 0.5), [3.0, 2.0, 0.0]) < 1e-9);
    // Rounded is not a Plane option; the prompt names the plane.
    let e = event(PLANE, opt("Rounded", Some("Yes")), vec![], json!({}));
    let r = Surfaces::step(&e).unwrap();
    assert!(r.error.is_some() && r.prompt.contains("plane") && !r.options.iter().any(|o| o.name == "Rounded"), "{}", r.prompt);
}

/// Rhino 5 PlanarSrf: a plane trimmed by the exact curves (a circle stays a circle), inner
/// curves are holes, and curves that only overlap make separate surfaces.
#[test]
fn planar_curves_are_exact_trims_like_rhino5() {
    let outer = exact_circle("o", [0.0; 3], 5.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let hole = exact_circle("h", [1.0, 0.0, 0.0], 2.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let r = run(PLANAR, vec![outer, hole], json!({}), vec![Input::Enter]);
    let Geo::Trimmed { surface, loops, .. } = &r.create[0] else { panic!() };
    // Each loop is the exact NURBS circle (every point on the circle).
    for (ring, (c, rad)) in loops.iter().zip([([0.0, 0.0, 0.0], 5.0), ([1.0, 0.0, 0.0], 2.0)]) {
        let nc = Nc::from_geo(&ring[0]).unwrap();
        for i in 0..=100 {
            assert!((dist(nc.eval(i as f64 / 100.0), c) - rad).abs() < 1e-12);
        }
    }
    // The sheet covers the outer loop in its plane.
    let s = Srf::of(surface).unwrap();
    assert!(s.at_st(0.0, 0.0)[0] < -5.0 && s.at_st(1.0, 1.0)[0] > 5.0 && s.at_st(0.5, 0.5)[2].abs() < 1e-12);
    // Two overlapping circles: two surfaces, no holes.
    let a = exact_circle("a", [0.0; 3], 3.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let b = exact_circle("b", [4.0, 0.0, 0.0], 3.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let r = run(PLANAR, vec![a, b], json!({}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 2);
    assert!(r.create.iter().all(|g| matches!(g, Geo::Trimmed { loops, .. } if loops.len() == 1)));
}

/// Rhino 5 ExtrudeCrv (Extrude All): curves, then the distance (typed, Enter, or a click; the
/// preview follows the pointer); exact surfaces; BothSides; DeleteInput.
#[test]
fn extrude_all_follows_rhino5_and_is_exact() {
    let circ = || exact_circle("c", [0.0; 3], 4.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let on_cylinder = |g: &Geo, z0: f64, z1: f64| {
        let s = Srf::of(g).unwrap();
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for i in 0..=40 {
            for j in 0..=40 {
                let p = s.at_st(i as f64 / 40.0, j as f64 / 40.0);
                assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - 4.0).abs() < 1e-9, "{p:?}");
                lo = lo.min(p[2]);
                hi = hi.max(p[2]);
            }
        }
        assert!((lo - z0).abs() < 1e-9 && (hi - z1).abs() < 1e-9, "{lo} {hi}");
    };
    let r = run(EXTRUDE_ALL, vec![circ()], json!({}), vec![text("5")]);
    on_cylinder(&r.create[0], 0.0, 5.0);
    // Enter: the default (2); a click 3 above the curve: 3.
    let r = run(EXTRUDE_ALL, vec![circ()], json!({}), vec![Input::Enter]);
    on_cylinder(&r.create[0], 0.0, 2.0);
    let r = run(EXTRUDE_ALL, vec![circ()], json!({}), vec![pt([9.0, 1.0, 3.0])]);
    on_cylinder(&r.create[0], 0.0, 3.0);
    // BothSides and DeleteInput.
    let r = run(EXTRUDE_ALL, vec![circ()], json!({}), vec![opt("BothSides", Some("Yes")), opt("DeleteInput", Some("Yes")), text("1.5")]);
    on_cylinder(&r.create[0], -1.5, 1.5);
    assert_eq!(r.delete, vec!["c".to_string()]);
    // The preview follows the pointer's height.
    let mut e = event(EXTRUDE_ALL, Input::Start, vec![circ()], json!({}));
    let s0 = Surfaces::step(&e).unwrap();
    e.state = s0.state.clone();
    e.input = Input::Hover { point: [0.0, 9.0, 7.0] };
    let h = Surfaces::step(&e).unwrap();
    on_cylinder(&h.preview[0], 0.0, 7.0);
    // Zero is refused, the command stays open.
    let mut e = event(EXTRUDE_ALL, Input::Start, vec![circ()], json!({}));
    let s0 = Surfaces::step(&e).unwrap();
    e.state = s0.state.clone();
    e.input = text("0");
    let z = Surfaces::step(&e).unwrap();
    assert!(!z.done && z.error.is_some());
}

fn point_sel(id: &str, p: Vec3) -> Selected {
    Selected { kind: "point".into(), ..curve(id, vec![p], false) }
}

/// Rhino 5 Loft options: a point may end (or start) the loft, Natural keeps the seams,
/// Rebuild / Refit simplify the sections.
#[test]
fn loft_point_seams_and_simplify_like_rhino5() {
    let (x, y) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // Circle r5 at z 0 to the point (0, 0, 10), straight: an exact cone.
    let r = run(LOFT, vec![exact_circle("a", [0.0; 3], 5.0, x, y), point_sel("p", [0.0, 0.0, 10.0])], json!({"loft_style": "straight"}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=20 {
        for j in 0..=40 {
            let p = s.at_st(i as f64 / 20.0, j as f64 / 40.0);
            let want = 5.0 * (1.0 - p[2] / 10.0);
            assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - want).abs() < 1e-9, "{p:?}");
        }
    }
    assert!(dist(s.at_st(1.0, 0.3), [0.0, 0.0, 10.0]) < 1e-12);
    // A point in the middle is refused.
    let e = event(LOFT, Input::Enter, vec![exact_circle("a", [0.0; 3], 5.0, x, y), point_sel("p", [0.0, 0.0, 5.0]), exact_circle("b", [0.0, 0.0, 10.0], 5.0, x, y)], json!({}));
    let r = Surfaces::step(&e).unwrap();
    assert!(!r.done && r.error.is_some());
    // Natural: the second circle keeps its own start (turned 90°), so the surface's v = 0
    // line ends where that circle starts.
    let b = exact_circle("b", [0.0, 0.0, 10.0], 5.0, y, [-1.0, 0.0, 0.0]);
    let r = run(LOFT, vec![exact_circle("a", [0.0; 3], 5.0, x, y), b.clone()], json!({"loft_seams": "natural"}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(dist(s.at_st(1.0, 0.0), [0.0, 5.0, 10.0]) < 1e-9);
    let r = run(LOFT, vec![exact_circle("a", [0.0; 3], 5.0, x, y), b], json!({}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    assert!(dist(s.at_st(1.0, 0.0), [5.0, 0.0, 10.0]) < 1e-9, "automatic aligns the seam: {:?} {:?}", s.at_st(1.0, 0.0), s.at_st(0.0, 0.0));
    // Rebuild with 12 points: non-rational sections, near the circles.
    let r = run(LOFT, vec![exact_circle("a", [0.0; 3], 5.0, x, y), exact_circle("b", [0.0, 0.0, 10.0], 5.0, x, y)], json!({"loft_simplify": "rebuild", "loft_rebuild_points": 12}), vec![Input::Enter]);
    match &r.create[0] {
        Geo::Surface { weights, size, .. } => {
            assert!(weights.iter().all(|w| (*w - 1.0).abs() < 1e-12));
            assert!((12..=15).contains(&size[1]), "{size:?}");
        }
        other => panic!("{other:?}"),
    }
    // Refit within 0.001: every point of the surface within that of the cylinder.
    let r = run(LOFT, vec![exact_circle("a", [0.0; 3], 5.0, x, y), exact_circle("b", [0.0, 0.0, 10.0], 5.0, x, y)], json!({"loft_simplify": "refit", "loft_refit_tolerance": 0.001}), vec![Input::Enter]);
    let s = Srf::of(&r.create[0]).unwrap();
    for j in 0..=200 {
        let p = s.at_st(0.5, j as f64 / 200.0);
        assert!(((p[0] * p[0] + p[1] * p[1]).sqrt() - 5.0).abs() < 1e-3, "{p:?}");
    }
}

/// Feed one input with a given selection, carrying the state.
fn feed(command: &str, state: &Value, input: Input, selection: Vec<Selected>, settings: Value) -> Reply {
    let mut e = event(command, input, selection, settings);
    e.state = state.clone();
    Surfaces::step(&e).unwrap()
}

/// Rhino 5 Sweep1: rail, then cross sections in order (Point option), Enter, then the
/// options (Style, Closed, GlobalBlend, Simplify) with a preview, Enter creates.
#[test]
fn sweep1_follows_rhino5_prompts_and_options() {
    let rail = || exact_sel("rail", Geo::Line { start: [0.0; 3], end: [0.0, 0.0, 10.0] }, false);
    let ring = || exact_circle("c", [0.0; 3], 2.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let r = feed(SWEEP1, &Value::Null, Input::Start, vec![], json!({}));
    assert_eq!(r.prompt, "Select rail");
    let r = feed(SWEEP1, &r.state, Input::Selection, vec![rail()], json!({}));
    assert!(r.prompt.starts_with("Select cross section curves"), "{}", r.prompt);
    assert!(r.options.iter().any(|o| o.name == "Point"));
    let r = feed(SWEEP1, &r.state, Input::Selection, vec![rail(), ring()], json!({}));
    assert!(r.prompt.contains("1 selected"), "{}", r.prompt);
    // Point option: the sweep ends at the apex.
    let r = feed(SWEEP1, &r.state, opt("Point", None), vec![rail(), ring()], json!({}));
    assert_eq!(r.want, Want::Point);
    let r = feed(SWEEP1, &r.state, pt([0.0, 0.0, 10.0]), vec![rail(), ring()], json!({}));
    // Enter ends the selection: options and a preview, nothing created yet.
    let r = feed(SWEEP1, &r.state, Input::Enter, vec![rail(), ring()], json!({}));
    assert!(!r.done && r.error.is_none(), "{:?} {}", r.error, r.prompt);
    assert!(!r.preview.is_empty());
    for name in ["Style", "GlobalBlend", "Simplify", "RebuildPoints", "RefitTolerance"] {
        assert!(r.options.iter().any(|o| o.name == name), "{name}");
    }
    // Closed is only offered for a closed rail with two or more sections.
    assert!(!r.options.iter().any(|o| o.name == "Closed"));
    // Global shape blending: linear from the circle to the point, a cone.
    let r = feed(SWEEP1, &r.state, opt("GlobalBlend", None), vec![rail(), ring()], json!({}));
    let r = feed(SWEEP1, &r.state, Input::Enter, vec![rail(), ring()], json!({}));
    assert!(r.done, "{}", r.prompt);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=20 {
        for j in 0..=20 {
            let p = s.at_st(i as f64 / 20.0, j as f64 / 20.0);
            let want = 2.0 * (1.0 - p[2] / 10.0);
            assert!((p[0].hypot(p[1]) - want).abs() < 1e-3, "{p:?}");
        }
    }
}

/// A ring: one profile around a closed rail is an exact surface of revolution; Roadlike Top
/// keeps it too (the rail lies flat).
#[test]
fn sweep1_ring_is_exact_like_rhino5() {
    let rail = exact_circle("rail", [0.0; 3], 10.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let prof = exact_circle("p", [10.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    for style in ["freeform", "roadlike_top"] {
        let r = run(SWEEP1, vec![rail.clone(), prof.clone()], json!({"sw1_style": style}), vec![Input::Enter]);
        assert!(r.messages.iter().any(|m| m.contains("exact")), "{:?}", r.messages);
        let s = Srf::of(&r.create[0]).unwrap();
        for i in 0..=40 {
            for j in 0..=40 {
                let p = s.at_st(i as f64 / 40.0, j as f64 / 40.0);
                let d = (p[0].hypot(p[1]) - 10.0).hypot(p[2]);
                assert!((d - 1.0).abs() < 1e-9, "{style}: {d}");
            }
        }
    }
}

/// Rhino 5 Sweep2: first rail, second rail, cross sections, Enter, options (MaintainHeight,
/// Simplify …), Enter. The surface's edges run on the rails.
#[test]
fn sweep2_follows_rhino5_prompts_and_runs_on_the_rails() {
    let ra = || exact_sel("a", Geo::Line { start: [0.0; 3], end: [0.0, 10.0, 0.0] }, false);
    let rb = || exact_sel("b", Geo::Line { start: [4.0, 0.0, 0.0], end: [8.0, 10.0, 0.0] }, false);
    let arch = || exact_sel("s", Geo::Arc { center: [2.0, 0.0, 0.0], x: [-1.0, 0.0, 0.0], y: [0.0, 0.0, 1.0], radius: 2.0, angle: PI }, false);
    let r = feed(SWEEP2, &Value::Null, Input::Start, vec![], json!({}));
    assert_eq!(r.prompt, "Select first rail");
    let r = feed(SWEEP2, &r.state, Input::Selection, vec![ra()], json!({}));
    assert_eq!(r.prompt, "Select second rail");
    let r = feed(SWEEP2, &r.state, Input::Selection, vec![ra(), rb()], json!({}));
    assert!(r.prompt.starts_with("Select cross section curves"), "{}", r.prompt);
    let r = feed(SWEEP2, &r.state, Input::Selection, vec![ra(), rb(), arch()], json!({}));
    let r = feed(SWEEP2, &r.state, Input::Enter, vec![ra(), rb(), arch()], json!({}));
    assert!(!r.done && !r.preview.is_empty(), "{:?}", r.error);
    assert!(r.options.iter().any(|o| o.name == "MaintainHeight"));
    assert!(!r.options.iter().any(|o| o.name == "Style"));
    let r = feed(SWEEP2, &r.state, opt("MaintainHeight", None), vec![ra(), rb(), arch()], json!({}));
    let r = feed(SWEEP2, &r.state, Input::Enter, vec![ra(), rb(), arch()], json!({}));
    assert!(r.done, "{}", r.prompt);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=20 {
        let u = i as f64 / 20.0;
        let (e0, e1, top) = (s.at_st(u, 0.0), s.at_st(u, 1.0), s.at_st(u, 0.5));
        assert!(e0[0].abs() < 1e-6, "edge on rail A {e0:?}");
        assert!((e1[0] - (4.0 + 0.4 * e1[1])).abs() < 2e-3, "edge on rail B {e1:?}");
        assert!((top[2] - 2.0).abs() < 2e-3, "maintain height: {top:?}");
    }
    // Preselected (Matrix way): the two longest curves are the rails.
    let r = run(SWEEP2, vec![ra(), rb(), arch()], json!({}), vec![Input::Enter]);
    assert!(r.done);
}

/// Rhino 5 RailRevolve: profile, rail, axis start, axis end (ScaleHeight); exact.
#[test]
fn rail_revolve_follows_rhino5_and_is_exact() {
    let prof = || exact_sel("p", Geo::Arc { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 0.0, 1.0], radius: 10.0, angle: PI / 2.0 }, false);
    let rail = || exact_circle("r", [0.0; 3], 10.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let r = feed(RAIL_REVOLVE, &Value::Null, Input::Start, vec![], json!({}));
    assert_eq!(r.prompt, "Select profile curve");
    assert!(r.options.iter().any(|o| o.name == "ScaleHeight"));
    let r = feed(RAIL_REVOLVE, &r.state, Input::Selection, vec![prof()], json!({}));
    assert_eq!(r.prompt, "Select rail curve");
    let r = feed(RAIL_REVOLVE, &r.state, Input::Selection, vec![prof(), rail()], json!({}));
    assert!(r.prompt.starts_with("Start of RailRevolve axis"), "{}", r.prompt);
    let r = feed(RAIL_REVOLVE, &r.state, pt([0.0; 3]), vec![prof(), rail()], json!({}));
    assert!(r.prompt.starts_with("End of RailRevolve axis"), "{}", r.prompt);
    let r = feed(RAIL_REVOLVE, &r.state, pt([0.0, 0.0, 5.0]), vec![prof(), rail()], json!({}));
    assert!(r.done, "{} {:?} {:?}", r.prompt, r.error, r.state);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=30 {
        for j in 0..=10 {
            let p = s.at_st(i as f64 / 30.0, j as f64 / 10.0);
            assert!(((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 10.0).abs() < 1e-9, "hemisphere {p:?}");
        }
    }
}

/// Rhino 5 NetworkSrf: curves of both directions in any order; the surface holds them all.
#[test]
fn curve_network_runs_through_every_curve() {
    let h = |x: f64, y: f64| 1.5 * (x * 0.4).sin() * (y * 0.3).cos();
    let mut sel = Vec::new();
    let mut k = 0;
    let mut add = |pts: Vec<Vec3>| {
        k += 1;
        let g = om9_addon_geom::interpolate(&pts, 3, false).unwrap();
        sel.push(exact_sel(&format!("c{k}"), g, false));
    };
    for x in [0.0, 4.0, 9.0] {
        add((0..=8).map(|i| { let y = i as f64; [x, y, h(x, y)] }).collect());
    }
    for y in [0.0, 3.0, 8.0] {
        add((0..=9).map(|i| { let x = i as f64; [x, y, h(x, y)] }).collect());
    }
    let r = run(NETWORK, sel.clone(), json!({}), vec![Input::Enter]);
    assert!(r.done, "{:?}", r.error);
    assert!(r.messages.iter().any(|m| m.contains("3 × 3")), "{:?}", r.messages);
    let s = Srf::of(&r.create[0]).unwrap();
    for c in &sel {
        for p in c.points.iter().step_by(8) {
            let mut best = (f64::MAX, 0.0, 0.0);
            for a in 0..=90 {
                for b in 0..=90 {
                    let (u, v) = (a as f64 / 90.0, b as f64 / 90.0);
                    let d = dist(*p, s.at_st(u, v));
                    if d < best.0 {
                        best = (d, u, v);
                    }
                }
            }
            let mut d = best.0;
            for a in -40..=40 {
                for b in -40..=40 {
                    let (u, v) = ((best.1 + a as f64 / 3600.0).clamp(0.0, 1.0), (best.2 + b as f64 / 3600.0).clamp(0.0, 1.0));
                    d = d.min(dist(*p, s.at_st(u, v)));
                }
            }
            assert!(d < 0.005, "{d}");
        }
    }
}

/// Rhino 5 Patch: a fit through boundary and interior curves, trimmed to the closed boundary.
#[test]
fn patch_fits_and_trims_like_rhino5() {
    let h = |x: f64, y: f64| 0.4 * (x / 3.0).sin() * (y / 4.0).cos();
    let mut sel = Vec::new();
    let mut add = |id: &str, f: &dyn Fn(f64) -> (f64, f64)| {
        let pts: Vec<Vec3> = (0..=12).map(|i| { let (x, y) = f(i as f64 / 12.0); [x, y, h(x, y)] }).collect();
        sel.push(exact_sel(id, om9_addon_geom::interpolate(&pts, 3, false).unwrap(), false));
    };
    add("b", &|t| (10.0 * t, 0.0));
    add("r", &|t| (10.0, 8.0 * t));
    add("t", &|t| (10.0 * (1.0 - t), 8.0));
    add("l", &|t| (0.0, 8.0 * (1.0 - t)));
    add("mid", &|t| (2.0 + 6.0 * t, 4.0));
    let r = run(PATCH, sel.clone(), json!({}), vec![Input::Enter]);
    assert!(r.done, "{:?}", r.error);
    let Geo::Trimmed { surface, loops, display } = &r.create[0] else { panic!("trimmed patch expected") };
    assert_eq!(loops.len(), 1);
    assert_eq!(loops[0].len(), 4, "the four boundary curves");
    assert!(display.is_some());
    let s = Srf::of(surface).unwrap();
    // The interior curve lies on the patch.
    for p in sel[4].points.iter().step_by(6) {
        let mut best = f64::MAX;
        for a in 0..=100 {
            for b in 0..=100 {
                best = best.min(dist(*p, s.at_st(a as f64 / 100.0, b as f64 / 100.0)));
            }
        }
        assert!(best < 0.06, "{best}");
    }
    // Without automatic trim: the plain surface.
    let r = run(PATCH, sel, json!({"patch_auto_trim": false}), vec![Input::Enter]);
    assert!(matches!(r.create[0], Geo::Surface { .. }));
}

/// Rhino 5 OffsetSrf: select, Enter, Offset distance <d> (FlipAll Loose Tolerance BothSides
/// DeleteInput); a plane offsets exactly.
#[test]
fn offset_srf_follows_rhino5() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let r = feed(OFFSET, &Value::Null, Input::Start, vec![a.clone()], json!({}));
    assert!(r.prompt.starts_with("Offset distance"), "{}", r.prompt);
    for name in ["FlipAll", "Loose", "Tolerance", "BothSides", "DeleteInput"] {
        assert!(r.options.iter().any(|o| o.name == name), "{name}");
    }
    assert!(!r.preview.is_empty(), "preview of the default distance");
    let r = feed(OFFSET, &r.state, opt("BothSides", None), vec![a.clone()], json!({}));
    let r = feed(OFFSET, &r.state, opt("DeleteInput", None), vec![a.clone()], json!({}));
    let r = feed(OFFSET, &r.state, text("0.6"), vec![a.clone()], json!({}));
    assert!(r.done, "{}", r.prompt);
    assert_eq!(r.create.len(), 2);
    assert_eq!(r.delete, vec!["a".to_string()]);
    let zs: Vec<f64> = r.create.iter().map(|g| Srf::of(g).unwrap().at_st(0.3, 0.6)[2]).collect();
    assert!(zs.iter().any(|z| (z - 0.6).abs() < 1e-12) && zs.iter().any(|z| (z + 0.6).abs() < 1e-12), "{zs:?}");
    assert!(r.messages.iter().any(|m| m.contains("exact")), "{:?}", r.messages);
}

/// Rhino 5 FilletSrf: first surface (click on the side kept), second surface (click on the
/// side kept); Radius on the command line. Two planes give an exact quarter cylinder in the
/// corner between the kept sides.
#[test]
fn fillet_srf_follows_rhino5_picks() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        name: String::new(),
        bbox: [[0.0; 3]; 2],
        points: vec![],
        closed: false,
        gem: None,
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        sub: Vec::new(),
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 6.0]]);
    let step = |state: &Value, input: Input, sel: Vec<Selected>, pick: Option<Vec3>| {
        let mut e = event(FILLET, input, sel, json!({}));
        e.state = state.clone();
        e.pick = pick;
        Surfaces::step(&e).unwrap()
    };
    let r = step(&Value::Null, Input::Start, vec![], None);
    assert_eq!(r.prompt, "Select first surface to fillet");
    assert!(r.options.iter().any(|o| o.name == "Radius"));
    let r = step(&r.state, opt("Radius", None), vec![], None);
    assert_eq!(r.want, Want::Number);
    let r = step(&r.state, text("1.5"), vec![], None);
    assert_eq!(r.prompt, "Select first surface to fillet");
    let r = step(&r.state, Input::Selection, vec![floor.clone()], Some([6.0, 4.0, 0.0]));
    assert_eq!(r.prompt, "Select second surface to fillet");
    let r = step(&r.state, Input::Selection, vec![floor.clone(), wall.clone()], Some([0.0, 4.0, 4.0]));
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=10 {
        for j in 0..=10 {
            let p = s.at_st(i as f64 / 10.0, j as f64 / 10.0);
            let d = (p[0] - 1.5).hypot(p[2] - 1.5);
            assert!((d - 1.5).abs() < 1e-6 && p[0] <= 1.5 + 1e-9 && p[2] <= 1.5 + 1e-9, "{p:?}");
        }
    }
    // Trim=Yes (Rhino's default): the floor keeps x ≥ 1.5, the wall z ≥ 1.5.
    assert_eq!(r.replace.len(), 2, "{:?}", r.messages);
    for rep in &r.replace {
        let Geo::Trimmed { loops, .. } = &rep.geo else { panic!("trimmed input expected") };
        let pts: Vec<Vec3> = loops[0].iter().flat_map(|c| om9_addon_geom::nurbs::samples(c, 8)).collect();
        let (axis, lo, hi) = if rep.id == "floor" { (0, 1.5, 10.0) } else { (2, 1.5, 6.0) };
        let min = pts.iter().map(|p| p[axis]).fold(f64::MAX, f64::min);
        let max = pts.iter().map(|p| p[axis]).fold(f64::MIN, f64::max);
        assert!((min - lo).abs() < 1e-6 && (max - hi).abs() < 1e-6, "{}: {min} {max}", rep.id);
    }
}

/// Rhino 5 ChamferSrf: Distances (first, then second), then the two surfaces with the sides
/// kept; the chamfer runs from the first distance on the first surface to the second on
/// the second, measured from the intersection.
#[test]
fn chamfer_srf_follows_rhino5() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        name: String::new(),
        bbox: [[0.0; 3]; 2],
        points: vec![],
        closed: false,
        gem: None,
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        sub: Vec::new(),
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 6.0]]);
    let step = |state: &Value, input: Input, sel: Vec<Selected>, pick: Option<Vec3>| {
        let mut e = event(CHAMFER, input, sel, json!({}));
        e.state = state.clone();
        e.pick = pick;
        Surfaces::step(&e).unwrap()
    };
    let r = step(&Value::Null, Input::Start, vec![], None);
    assert_eq!(r.prompt, "Select first surface to chamfer");
    let r = step(&r.state, opt("Distances", None), vec![], None);
    assert!(r.prompt.starts_with("First chamfer distance"), "{}", r.prompt);
    let r = step(&r.state, text("0.5"), vec![], None);
    assert!(r.prompt.starts_with("Second chamfer distance"), "{}", r.prompt);
    let r = step(&r.state, text("2"), vec![], None);
    assert_eq!(r.prompt, "Select first surface to chamfer");
    let r = step(&r.state, Input::Selection, vec![floor.clone()], Some([6.0, 4.0, 0.0]));
    let r = step(&r.state, Input::Selection, vec![floor, wall], Some([0.0, 4.0, 4.0]));
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=8 {
        let t = i as f64 / 8.0;
        let ends = [s.at_st(t, 0.0), s.at_st(t, 1.0), s.at_st(0.0, t), s.at_st(1.0, t)];
        assert!(ends.iter().any(|p| p[2].abs() < 1e-9 && (p[0] - 0.5).abs() < 1e-9), "{ends:?}");
        assert!(ends.iter().any(|p| p[0].abs() < 1e-9 && (p[2] - 2.0).abs() < 1e-9), "{ends:?}");
    }
}

/// Rhino 5 ExtendSrf: Select edge of surface to extend (Type), Extension factor <d> with a
/// preview; Enter takes the default.
#[test]
fn extend_srf_follows_rhino5() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let r = feed(EXTEND, &Value::Null, Input::Start, vec![a.clone()], json!({"es_length": 1.5}));
    assert_eq!(r.prompt, "Select edge of surface to extend");
    assert!(r.options.iter().any(|o| o.name == "Type" && o.value.as_deref() == Some("Smooth")));
    let r = feed(EXTEND, &r.state, opt("Type", None), vec![a.clone()], json!({"es_length": 1.5}));
    assert!(r.options.iter().any(|o| o.name == "Type" && o.value.as_deref() == Some("Line")));
    let r = feed(EXTEND, &r.state, pt([2.0, 4.1, 0.0]), vec![a.clone()], json!({"es_length": 1.5}));
    assert!(r.prompt.starts_with("Extension factor <1.5>"), "{}", r.prompt);
    assert!(!r.preview.is_empty());
    let r = feed(EXTEND, &r.state, Input::Enter, vec![a], json!({"es_length": 1.5}));
    assert!(r.done);
    let s = Srf::of(&r.replace[0].geo).unwrap();
    let ymax = (0..=4).flat_map(|i| [s.at_st(i as f64 / 4.0, 1.0)[1], s.at_st(1.0, i as f64 / 4.0)[1]]).fold(f64::MIN, f64::max);
    assert!((ymax - 5.5).abs() < 1e-9, "{ymax}");
}

/// Rhino 5 BlendSrf: a click near each edge, then Continuity1/2 and Bulge1/2 with a preview;
/// Enter creates a blend meeting both edges tangentially.
#[test]
fn blend_srf_follows_rhino5() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let b = flat("b", 6.0, 10.0, 0.0, 4.0, 1.0);
    let sel = || vec![a.clone(), b.clone()];
    let mut e = event(BLEND, Input::Start, vec![], json!({}));
    e.state = Value::Null;
    let r = Surfaces::step(&e).unwrap();
    assert_eq!(r.prompt, "Select segment for first edge");
    // Both surfaces selected: clicks near the edges x = 4 and x = 6.
    let mut e = event(BLEND, pt([4.05, 2.0, 0.0]), sel(), json!({}));
    e.state = r.state.clone();
    let r = Surfaces::step(&e).unwrap();
    assert_eq!(r.prompt, "Select segment for second edge");
    let r = feed(BLEND, &r.state, pt([5.95, 2.0, 1.0]), sel(), json!({}));
    assert!(!r.done && !r.preview.is_empty(), "{:?}", r.error);
    for name in ["Continuity1", "Continuity2", "Bulge1", "Bulge2"] {
        assert!(r.options.iter().any(|o| o.name == name), "{name}");
    }
    let r = feed(BLEND, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=8 {
        let t = i as f64 / 8.0;
        let ends = [s.at_st(t, 0.0), s.at_st(t, 1.0), s.at_st(0.0, t), s.at_st(1.0, t)];
        assert!(ends.iter().any(|p| (p[0] - 4.0).abs() < 1e-6 && p[2].abs() < 1e-6), "{ends:?}");
        assert!(ends.iter().any(|p| (p[0] - 6.0).abs() < 1e-6 && (p[2] - 1.0).abs() < 1e-6), "{ends:?}");
    }
}

/// Rhino 5 MatchSrf: a click near the edge to change, a click near the target edge, then
/// Continuity with a preview; Enter changes the first surface so its edge lies on the target.
#[test]
fn match_srf_follows_rhino5() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let b = flat("b", 4.5, 8.0, 0.0, 4.0, 1.0);
    let sel = || vec![a.clone(), b.clone()];
    let mut e = event(MATCH, pt([4.6, 2.0, 1.0]), sel(), json!({}));
    e.state = json!({"edges": [], "flow": null});
    let r = Surfaces::step(&e).unwrap();
    assert_eq!(r.prompt, "Select target surface edge");
    let r = feed(MATCH, &r.state, pt([3.9, 2.0, 0.0]), sel(), json!({}));
    assert!(!r.done && !r.preview.is_empty(), "{:?}", r.error);
    assert!(r.options.iter().any(|o| o.name == "Continuity"));
    let r = feed(MATCH, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done);
    assert_eq!(r.replace[0].id, "b");
    let s = Srf::of(&r.replace[0].geo).unwrap();
    let pts = [s.at_st(0.0, 0.5), s.at_st(1.0, 0.5), s.at_st(0.5, 0.0), s.at_st(0.5, 1.0)];
    assert!(pts.iter().any(|p| (p[0] - 4.0).abs() < 1e-6 && p[2].abs() < 1e-6), "the edge moved onto x = 4, z = 0: {pts:?}");
    // Average=Yes: both change, meeting half way (x 4.25, z 0.5).
    let mut e = event(MATCH, pt([4.6, 2.0, 1.0]), sel(), json!({}));
    e.state = json!({"edges": [], "flow": null});
    let r = Surfaces::step(&e).unwrap();
    let r = feed(MATCH, &r.state, pt([3.9, 2.0, 0.0]), sel(), json!({}));
    let r = feed(MATCH, &r.state, opt("Average", None), sel(), json!({}));
    assert_eq!(r.preview.len(), 2);
    let r = feed(MATCH, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done && r.replace.len() == 2, "{:?}", r.error);
    for x in &r.replace {
        let s = Srf::of(&x.geo).unwrap();
        let pts = [s.at_st(0.0, 0.5), s.at_st(1.0, 0.5), s.at_st(0.5, 0.0), s.at_st(0.5, 1.0)];
        assert!(pts.iter().any(|p| (p[0] - 4.25).abs() < 1e-6 && (p[2] - 0.5).abs() < 1e-6), "{}: {pts:?}", x.id);
    }
}

/// Rhino 5 Split (Isocurve) and SrfSeam: exact pieces and seam, the isocurve following the
/// pointer.
#[test]
fn split_isocurve_and_seam_are_exact() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let r = feed(SPLIT_ISO, &Value::Null, Input::Start, vec![a.clone()], json!({}));
    assert_eq!(r.prompt, "Select isocurve location");
    let r = feed(SPLIT_ISO, &r.state, opt("Direction", Some("Both")), vec![a.clone()], json!({}));
    let h = feed(SPLIT_ISO, &r.state, Input::Hover { point: [1.0, 3.0, 0.0] }, vec![a.clone()], json!({}));
    assert_eq!(h.preview.len(), 2, "both isocurves follow the pointer");
    let r = feed(SPLIT_ISO, &r.state, pt([1.0, 3.0, 0.0]), vec![a.clone()], json!({}));
    assert!(r.done);
    assert_eq!(r.create.len() + r.replace.len(), 4);
    let area: f64 = r
        .create
        .iter()
        .chain(r.replace.iter().map(|x| &x.geo))
        .map(|g| {
            let s = Srf::of(g).unwrap();
            let (p, q) = (s.at_st(0.0, 0.0), s.at_st(1.0, 1.0));
            ((q[0] - p[0]) * (q[1] - p[1])).abs()
        })
        .sum();
    assert!((area - 16.0).abs() < 1e-9, "{area}");
}

/// Rhino 5 Fin: curve on the surface, the surface, Pick a location on the curve for a
/// height, the height (typed or clicked), Pick location to set another height (or Enter:
/// uniform). Direction Normal / Tangent. On a plane the fin stands straight up exactly.
#[test]
fn fin_follows_rhino5() {
    let a = flat("a", 0.0, 10.0, 0.0, 10.0, 0.0);
    let c = exact_sel("c", Geo::Line { start: [2.0, 5.0, 0.0], end: [8.0, 5.0, 0.0] }, false);
    let sel = || vec![c.clone(), a.clone()];
    let r = feed(EXTRUDE_NORMAL, &Value::Null, Input::Start, vec![], json!({}));
    assert_eq!(r.prompt, "Select curve on surface");
    let r = feed(EXTRUDE_NORMAL, &r.state, Input::Selection, vec![c.clone()], json!({}));
    assert_eq!(r.prompt, "Select surface");
    let r = feed(EXTRUDE_NORMAL, &r.state, Input::Selection, sel(), json!({}));
    assert_eq!(r.prompt, "Pick a location on the curve for a height");
    let start = r.state.clone();
    // Uniform: a location, a clicked height, Enter.
    let r = feed(EXTRUDE_NORMAL, &start, pt([5.0, 5.0, 0.0]), sel(), json!({}));
    assert!(r.prompt.starts_with("Height"), "{}", r.prompt);
    let h = feed(EXTRUDE_NORMAL, &r.state, Input::Hover { point: [5.0, 5.0, 2.0] }, sel(), json!({}));
    assert!(!h.preview.is_empty());
    let r = feed(EXTRUDE_NORMAL, &r.state, pt([5.0, 5.0, 2.5]), sel(), json!({}));
    assert!(r.prompt.starts_with("Pick location to set another height"), "{}", r.prompt);
    let r = feed(EXTRUDE_NORMAL, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    let zs: Vec<f64> = [s.at_st(0.5, 0.0), s.at_st(0.5, 1.0), s.at_st(0.0, 0.5), s.at_st(1.0, 0.5)].iter().map(|p| p[2].abs()).collect();
    assert!(zs.iter().any(|z| (z - 2.5).abs() < 1e-9) && zs.iter().any(|z| *z < 1e-9), "{zs:?}");
    // A height typed at the first prompt: uniform at once.
    let r = feed(EXTRUDE_NORMAL, &start, text("1.5"), sel(), json!({}));
    assert!(r.done, "{:?}", r.error);
    // Two heights: 1 at the start (x 2), 3 at the end (x 8); 2 half way.
    let r = feed(EXTRUDE_NORMAL, &start, pt([2.0, 5.0, 0.0]), sel(), json!({}));
    let r = feed(EXTRUDE_NORMAL, &r.state, text("1"), sel(), json!({}));
    let r = feed(EXTRUDE_NORMAL, &r.state, pt([8.0, 5.0, 0.0]), sel(), json!({}));
    let r = feed(EXTRUDE_NORMAL, &r.state, text("3"), sel(), json!({}));
    let r = feed(EXTRUDE_NORMAL, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    // The top edge: height at x.
    let top_at = |x: f64| -> f64 {
        let mut best = (f64::MAX, 0.0);
        for i in 0..=600 {
            for v in [0.0, 1.0] {
                let p = s.at_st(i as f64 / 600.0, v);
                if p[2].abs() > 1e-6 && (p[0] - x).abs() < best.0 {
                    best = ((p[0] - x).abs(), p[2].abs());
                }
            }
        }
        best.1
    };
    assert!((top_at(2.0) - 1.0).abs() < 1e-3 && (top_at(8.0) - 3.0).abs() < 1e-3 && (top_at(5.0) - 2.0).abs() < 2e-2, "{} {} {}", top_at(2.0), top_at(5.0), top_at(8.0));
}

/// Rhino 5 VariableFilletSrf: handles at the ends (click near one, type its radius),
/// AddHandle (a point on the edge, then its radius), preview, Enter.
#[test]
fn variable_fillet_follows_rhino5_handles() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        name: String::new(),
        bbox: [[0.0; 3]; 2],
        points: vec![],
        closed: false,
        gem: None,
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        sub: Vec::new(),
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 6.0]]);
    let sel = || vec![floor.clone(), wall.clone()];
    let set = json!({"fs_radius": 1.0, "fs_end_radius": 1.0, "fs_trim": false});
    let r = feed(VAR_FILLET, &Value::Null, Input::Start, sel(), set.clone());
    assert!(r.prompt.starts_with("Select handle to edit"), "{}", r.prompt);
    assert!(!r.preview.is_empty(), "{:?}", r.error);
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 7.9, 0.0]), sel(), set.clone());
    assert!(r.prompt.starts_with("Radius <1>"), "{}", r.prompt);
    let r = feed(VAR_FILLET, &r.state, text("2"), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, opt("AddHandle", None), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 4.0, 0.0]), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, text("0.5"), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, Input::Enter, sel(), set);
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    // The floor contact (z = 0): x is the radius there.
    let floor_x_at = |y: f64| -> f64 {
        let mut best = (f64::MAX, 0.0);
        for i in 0..=400 {
            for v in [0.0, 1.0] {
                let p = s.at_st(i as f64 / 400.0, v);
                if p[2].abs() < 1e-6 && (p[1] - y).abs() < best.0 {
                    best = ((p[1] - y).abs(), p[0]);
                }
            }
        }
        best.1
    };
    assert!((floor_x_at(0.0) - 1.0).abs() < 2e-3, "{}", floor_x_at(0.0));
    assert!((floor_x_at(4.0) - 0.5).abs() < 2e-3, "{}", floor_x_at(4.0));
    assert!((floor_x_at(8.0) - 2.0).abs() < 2e-3, "{}", floor_x_at(8.0));
}

/// Rhino 5 VariableOffsetSrf: corner handles (click near one, type its distance), preview,
/// Enter.
#[test]
fn variable_offset_follows_rhino5_handles() {
    let a = flat("a", 0.0, 4.0, 0.0, 4.0, 0.0);
    let set = json!({"os_distance": 1.0});
    let r = feed(VAR_OFFSET, &Value::Null, Input::Start, vec![a.clone()], set.clone());
    assert!(r.prompt.starts_with("Select edge point to adjust"), "{}", r.prompt);
    assert!(!r.preview.is_empty());
    let r = feed(VAR_OFFSET, &r.state, pt([3.9, 3.9, 0.0]), vec![a.clone()], set.clone());
    assert!(r.prompt.starts_with("Distance <1>"), "{}", r.prompt);
    let r = feed(VAR_OFFSET, &r.state, text("3"), vec![a.clone()], set.clone());
    let r = feed(VAR_OFFSET, &r.state, Input::Enter, vec![a], set);
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    let corners: Vec<f64> = [(0.0, 0.0), (1.0, 1.0)].iter().map(|(u, v)| s.at_st(*u, *v)[2].abs()).collect();
    assert!((corners[0] - 1.0).abs() < 1e-6 && (corners[1] - 3.0).abs() < 1e-6, "{corners:?}");
}

/// Rhino 5 VariableBlendSrf: the same handles; a curvature-continuous blend, Enter.
#[test]
fn variable_blend_follows_rhino5() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        name: String::new(),
        bbox: [[0.0; 3]; 2],
        points: vec![],
        closed: false,
        gem: None,
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        sub: Vec::new(),
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 6.0]]);
    let sel = || vec![floor.clone(), wall.clone()];
    let r = feed(VAR_BLEND, &Value::Null, Input::Start, sel(), json!({"fs_trim": true}));
    assert!(r.prompt.starts_with("Select handle to edit"), "{}", r.prompt);
    let r = feed(VAR_BLEND, &r.state, Input::Enter, sel(), json!({"fs_trim": true}));
    assert!(r.done, "{:?}", r.error);
    assert_eq!(r.replace.len(), 2, "trimmed inputs: {:?}", r.messages);
}

/// Rhino 5 UnrollSrf: a half cylinder unrolls to a flat NURBS rectangle π·r long.
#[test]
fn unroll_gives_a_flat_surface_with_true_lengths() {
    // An exact half cylinder (rational) as the input.
    let half = om9_addon_geom::exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, PI);
    let srf_rows: Vec<Vec<om9_addon_geom::exact::H>> = half.points.iter().map(|c| vec![*c, [c[0], c[1], 4.0 * c[3], c[3]]]).collect();
    let g = om9_addon_geom::exact::surface(&srf_rows, [2, 1], [half.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
    let r = run(UNROLL, vec![srf_sel("c", g.clone())], json!({}), vec![Input::Enter]);
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).expect("a NURBS surface");
    let (a, b) = (s.at_st(0.0, 0.0), s.at_st(1.0, 0.0));
    let (c, d) = (s.at_st(0.0, 1.0), s.at_st(1.0, 1.0));
    let lengths = [dist(a, b), dist(a, c)];
    assert!(lengths.iter().any(|l| (l - 3.0 * PI).abs() < 2e-4) && lengths.iter().any(|l| (l - 4.0).abs() < 1e-6), "{lengths:?}");
    assert!(((dist(c, d) - dist(a, b)).abs()) < 2e-4);
    assert!([a, b, c, d].iter().all(|p| p[2].abs() < 1e-9), "flat");
    // A curve on the surface comes along: the half circle at height 2 unrolls to a straight
    // line 3π long, 2 from the flat piece's edge.
    let ring = om9_addon_geom::exact::arc([0.0, 0.0, 2.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, PI);
    let curve = exact_sel("r", ring.to_geo(), false);
    let r = run(UNROLL, vec![srf_sel("c", g.clone()), curve], json!({}), vec![Input::Enter]);
    assert!(r.done && r.create.len() == 2, "{:?}", r.create.len());
    let pts = om9_addon_geom::nurbs::samples(&r.create[1], 50);
    let (p0, p1) = (pts[0], *pts.last().unwrap());
    assert!((dist(p0, p1) - 3.0 * PI).abs() < 2e-3, "{}", dist(p0, p1));
    let flat_srf = Srf::of(&r.create[0]).unwrap();
    for q in &pts {
        // On the flat piece, straight: every point on the chord p0–p1.
        let t = dot(sub(*q, p0), sub(p1, p0)) / dot(sub(p1, p0), sub(p1, p0));
        let on = add(p0, scale(sub(p1, p0), t));
        assert!(dist(*q, on) < 2e-3 && q[2].abs() < 1e-9, "{q:?}");
        let (a, b) = flat_srf.closest_st(*q);
        assert!(dist(flat_srf.at_st(a, b), *q) < 2e-3);
    }
}

/// Rhino 5 FitSrf: a rational cylinder piece fitted within the tolerance, degrees as set;
/// DeleteInput=No adds the fit instead of replacing.
#[test]
fn fitsrf_meets_the_tolerance_with_the_set_degrees() {
    let arc = om9_addon_geom::exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, PI);
    let rows: Vec<Vec<om9_addon_geom::exact::H>> = arc.points.iter().map(|c| vec![*c, [c[0], c[1], 5.0 * c[3], c[3]]]).collect();
    let g = om9_addon_geom::exact::surface(&rows, [2, 1], [arc.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]]);
    let r = run(REFIT, vec![srf_sel("c", g.clone())], json!({"srf_tolerance": 0.001, "rf_degree_u": 5, "rf_degree_v": 1}), vec![Input::Enter]);
    assert!(r.done, "{:?}", r.error);
    assert_eq!(r.replace.len(), 1);
    let Geo::Surface { degree, .. } = &r.replace[0].geo else { panic!() };
    assert_eq!(*degree, [5, 1]);
    let f = Srf::of(&r.replace[0].geo).unwrap();
    for i in 0..=97 {
        for j in 0..=3 {
            let p = f.at_st(i as f64 / 97.0, j as f64 / 3.0);
            assert!((p[0].hypot(p[1]) - 10.0).abs() <= 0.001, "{p:?}");
        }
    }
    let r = run(REFIT, vec![srf_sel("c", g)], json!({"rf_delete_input": false}), vec![Input::Enter]);
    assert!(r.done && r.replace.is_empty() && r.create.len() == 1);
}

/// Rhino 5 CutPlane: the plane runs through the picked line, square to the CPlane, and
/// reaches past every selected object even when the line is short.
#[test]
fn cutplane_is_large_enough_for_the_selection() {
    let s = flat("p", -20.0, 30.0, -5.0, 5.0, 3.0);
    let sel = || vec![s.clone()];
    let r = feed(CUTTING_PLANE, &Value::Null, Input::Start, sel(), json!({}));
    assert!(r.prompt.starts_with("Start of cut plane"), "{}", r.prompt);
    let r = feed(CUTTING_PLANE, &r.state, pt([0.0, 0.0, 0.0]), sel(), json!({}));
    assert_eq!(r.prompt, "End of cut plane");
    let h = feed(CUTTING_PLANE, &r.state, Input::Hover { point: [1.0, 0.0, 0.0] }, sel(), json!({}));
    assert_eq!(h.preview.len(), 1);
    // The prompts repeat: a second plane across (x = 5), then Enter makes both.
    let r = feed(CUTTING_PLANE, &r.state, pt([1.0, 0.0, 0.0]), sel(), json!({}));
    assert!(r.prompt.starts_with("Start of cut plane") && r.preview.len() == 1, "{}", r.prompt);
    let r = feed(CUTTING_PLANE, &r.state, pt([5.0, -1.0, 0.0]), sel(), json!({}));
    let r = feed(CUTTING_PLANE, &r.state, pt([5.0, 1.0, 0.0]), sel(), json!({}));
    let r = feed(CUTTING_PLANE, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done && r.create.len() == 2, "{:?}", r.error);
    let f = Srf::of(&r.create[0]).unwrap();
    let corners = [f.at_st(0.0, 0.0), f.at_st(1.0, 0.0), f.at_st(0.0, 1.0), f.at_st(1.0, 1.0)];
    assert!(corners.iter().all(|p| p[1].abs() < 1e-9), "through the line: {corners:?}");
    let (x0, x1) = corners.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p[0]), b.max(p[0])));
    let (z0, z1) = corners.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p[2]), b.max(p[2])));
    assert!(x0 < -20.0 && x1 > 30.0 && z0 < 3.0 && z1 > 3.0, "{x0} {x1} {z0} {z1}");
    let g = Srf::of(&r.create[1]).unwrap();
    assert!([g.at_st(0.0, 0.0), g.at_st(1.0, 1.0)].iter().all(|p| (p[0] - 5.0).abs() < 1e-9));
    // 3Point: edge (0,0,0) → (4,0,0), width typed 2: a 4 × 2 rectangle standing up.
    let r = feed(CUTTING_PLANE, &Value::Null, Input::Start, sel(), json!({}));
    let r = feed(CUTTING_PLANE, &r.state, opt("3Point", None), sel(), json!({}));
    assert_eq!(r.prompt, "Start of edge");
    let r = feed(CUTTING_PLANE, &r.state, pt([0.0, 0.0, 0.0]), sel(), json!({}));
    let r = feed(CUTTING_PLANE, &r.state, pt([4.0, 0.0, 0.0]), sel(), json!({}));
    assert!(r.prompt.starts_with("Width"), "{}", r.prompt);
    let r = feed(CUTTING_PLANE, &r.state, text("2"), sel(), json!({}));
    let r = feed(CUTTING_PLANE, &r.state, Input::Enter, sel(), json!({}));
    assert!(r.done && r.create.len() == 1);
    let q = Srf::of(&r.create[0]).unwrap();
    let c = [q.at_st(0.0, 0.0), q.at_st(1.0, 0.0), q.at_st(1.0, 1.0), q.at_st(0.0, 1.0)];
    let xs = c.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
    let zs = c.iter().map(|p| p[2]).fold(f64::MIN, f64::max);
    assert!((xs - 4.0).abs() < 1e-9 && (zs - 2.0).abs() < 1e-9 && c.iter().all(|p| p[1].abs() < 1e-9), "{c:?}");
}

/// Rhino 5 FilletSrf Extend and Split: the wall is shorter (y 2..6 of the floor's 0..8). With
/// Extend=Yes the fillet runs the floor's whole 8 mm; Trim=No, Split=Yes cuts each input into
/// two surfaces along its fillet edge (the wall only along its own 4 mm).
#[test]
fn filletsrf_extend_and_split() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        ..flat("x", 0.0, 1.0, 0.0, 1.0, 0.0)
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 2.0, -1.0], [0.0, 6.0, -1.0], [0.0, 2.0, 6.0], [0.0, 6.0, 6.0]]);
    let run = |set: Value| {
        let mut e = event(FILLET, Input::Start, vec![], set.clone());
        let r = Surfaces::step(&e).unwrap();
        e = event(FILLET, Input::Selection, vec![floor.clone()], set.clone());
        e.state = r.state;
        e.pick = Some([6.0, 4.0, 0.0]);
        let r = Surfaces::step(&e).unwrap();
        e = event(FILLET, Input::Selection, vec![floor.clone(), wall.clone()], set);
        e.state = r.state;
        e.pick = Some([0.0, 4.0, 4.0]);
        Surfaces::step(&e).unwrap()
    };
    let length = |g: &Geo| {
        let s = Srf::of(g).unwrap();
        dist(s.at_st(0.0, 0.0), s.at_st(1.0, 0.0))
    };
    let r = run(json!({"fs_radius": 1.0, "fs_extend": false, "fs_trim": false}));
    assert!(r.done && r.replace.is_empty(), "{:?}", r.error);
    assert!((length(&r.create[0]) - 4.0).abs() < 1e-6);
    let r = run(json!({"fs_radius": 1.0, "fs_trim": false, "fs_split": true}));
    assert!(r.done, "{:?}", r.error);
    assert!((length(&r.create[0]) - 8.0).abs() < 1e-6, "{}", length(&r.create[0]));
    assert_eq!(r.replace.len(), 2, "{:?}", r.messages);
    assert_eq!(r.create.len(), 3, "fillet + the two other pieces");
    // The wall's kept piece: z from 1 (its fillet edge) to 6; its other piece z −1 to 1.
    let span = |g: &Geo, axis: usize| {
        let Geo::Trimmed { loops, .. } = g else { panic!("trimmed piece expected") };
        let pts: Vec<Vec3> = loops[0].iter().flat_map(|c| om9_addon_geom::nurbs::samples(c, 8)).collect();
        (pts.iter().map(|p| p[axis]).fold(f64::MAX, f64::min), pts.iter().map(|p| p[axis]).fold(f64::MIN, f64::max))
    };
    let wall_kept = &r.replace.iter().find(|x| x.id == "wall").unwrap().geo;
    let (lo, hi) = span(wall_kept, 2);
    assert!((lo - 1.0).abs() < 1e-6 && (hi - 6.0).abs() < 1e-6, "{lo} {hi}");
    let others: Vec<(f64, f64)> = r.create[1..].iter().map(|g| span(g, 2)).collect();
    assert!(others.iter().any(|(lo, hi)| (lo + 1.0).abs() < 1e-6 && (hi - 1.0).abs() < 1e-6), "{others:?}");
    let floor_kept = &r.replace.iter().find(|x| x.id == "floor").unwrap().geo;
    let (lo, hi) = span(floor_kept, 0);
    assert!((lo - 1.0).abs() < 1e-6 && (hi - 10.0).abs() < 1e-6, "{lo} {hi}");
    // Extend + Trim: both inputs trimmed (the wall along its own part of the edge).
    let r = run(json!({"fs_radius": 1.0}));
    assert!(r.done && r.replace.len() == 2 && r.create.len() == 1, "{:?}", r.messages);
    let (lo, hi) = span(&r.replace.iter().find(|x| x.id == "wall").unwrap().geo, 2);
    assert!((lo - 1.0).abs() < 1e-6 && (hi - 6.0).abs() < 1e-6, "{lo} {hi}");
}

/// Rhino 5 VariableFilletSrf handle options: CopyHandle, RemoveHandle (not the ends),
/// LinkHandles, FromTwoPoints; RailType=DistFromEdge (between square planes the radius equals
/// the distance from the edge).
#[test]
fn variable_fillet_handle_options_and_rail_type() {
    let surface = |id: &str, pts: Vec<Vec3>| Selected {
        id: id.into(),
        kind: "surface".into(),
        mesh: None,
        exact: Some(Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: pts,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        }),
        ..flat("x", 0.0, 1.0, 0.0, 1.0, 0.0)
    };
    let floor = surface("floor", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]]);
    let wall = surface("wall", vec![[0.0, 0.0, 0.0], [0.0, 8.0, 0.0], [0.0, 0.0, 6.0], [0.0, 8.0, 6.0]]);
    let sel = || vec![floor.clone(), wall.clone()];
    let set = json!({"fs_radius": 1.0, "fs_end_radius": 1.0, "fs_trim": false});
    let state = |st: &Value| -> Vec<(f64, f64)> {
        st["handles"].as_array().unwrap().iter().map(|h| (h[0].as_f64().unwrap(), h[1].as_f64().unwrap())).collect()
    };
    let r = feed(VAR_FILLET, &Value::Null, Input::Start, sel(), set.clone());
    let names: Vec<String> = r.options.iter().map(|o| o.name.clone()).collect();
    for n in ["SetAll", "AddHandle", "CopyHandle", "LinkHandles", "RailType", "Trim"] {
        assert!(names.iter().any(|x| x == n), "{n}: {names:?}");
    }
    // Edit the start handle by two points: 1.5 apart.
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 0.1, 0.0]), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, opt("FromTwoPoints", None), sel(), set.clone());
    assert_eq!(r.prompt, "First point");
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 0.0, 0.0]), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, pt([1.5, 0.0, 0.0]), sel(), set.clone());
    assert!(state(&r.state).iter().filter(|h| (h.1 - 1.5).abs() < 1e-12).count() == 1, "{:?}", state(&r.state));
    // CopyHandle: copy the start (1.5) to the middle.
    let r = feed(VAR_FILLET, &r.state, opt("CopyHandle", None), sel(), set.clone());
    assert_eq!(r.prompt, "Select handle to copy");
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 0.2, 0.0]), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 4.0, 0.0]), sel(), set.clone());
    let h = state(&r.state);
    assert!(h.len() == 3 && (h[2].0 - 0.5).abs() < 1e-3 && (h[2].1 - 1.5).abs() < 1e-12, "{h:?}");
    // RemoveHandle: the end handles stay; the middle one goes.
    let r = feed(VAR_FILLET, &r.state, opt("RemoveHandle", None), sel(), set.clone());
    let refused = feed(VAR_FILLET, &r.state, pt([0.0, 7.9, 0.0]), sel(), set.clone());
    assert!(refused.error.is_some() && state(&refused.state).len() == 3);
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 4.1, 0.0]), sel(), set.clone());
    assert_eq!(state(&r.state).len(), 2);
    // LinkHandles: setting one sets all.
    let r = feed(VAR_FILLET, &r.state, opt("LinkHandles", None), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, pt([0.0, 7.9, 0.0]), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, text("0.8"), sel(), set.clone());
    assert!(state(&r.state).iter().all(|h| (h.1 - 0.8).abs() < 1e-12));
    // RailType DistBetweenRails 0.8 between square planes: the edges 0.8 apart (R = 0.8/√2).
    let r = feed(VAR_FILLET, &r.state, opt("RailType", Some("DistBetweenRails")), sel(), set.clone());
    let r = feed(VAR_FILLET, &r.state, Input::Enter, sel(), set);
    assert!(r.done, "{:?}", r.error);
    let s = Srf::of(&r.create[0]).unwrap();
    let (p, q) = (s.at_st(0.5, 0.0), s.at_st(0.5, 1.0));
    assert!((dist(p, q) - 0.8).abs() < 1e-4, "{}", dist(p, q));
}

/// Rhino 5 NetworkSrf NoAutoSort: the first direction's curves (Enter), then the second's
/// (Enter); the surface runs through them in the order picked — the same surface as the
/// sorted one when they were picked in order.
#[test]
fn curve_network_no_auto_sort() {
    let h = |x: f64, y: f64| 1.5 * (x * 0.4).sin() * (y * 0.3).cos();
    let mk = |id: &str, pts: Vec<Vec3>| exact_sel(id, om9_addon_geom::interpolate(&pts, 3, false).unwrap(), false);
    let first: Vec<Selected> = [0.0, 4.0, 9.0].iter().enumerate().map(|(i, x)| mk(&format!("a{i}"), (0..=8).map(|k| { let y = k as f64; [*x, y, h(*x, y)] }).collect())).collect();
    let second: Vec<Selected> = [0.0, 3.0, 8.0].iter().enumerate().map(|(i, y)| mk(&format!("b{i}"), (0..=9).map(|k| { let x = k as f64; [x, *y, h(x, *y)] }).collect())).collect();
    let set = json!({"nw_no_auto_sort": true});
    let r = feed(NETWORK, &Value::Null, Input::Start, vec![], set.clone());
    assert!(r.prompt.starts_with("Select curves in first direction"), "{}", r.prompt);
    let r = feed(NETWORK, &r.state, Input::Enter, first.clone(), set.clone());
    assert!(r.prompt.starts_with("Select curves in second direction"), "{}", r.prompt);
    let all: Vec<Selected> = first.iter().chain(&second).cloned().collect();
    let r = feed(NETWORK, &r.state, Input::Enter, all.clone(), set);
    assert!(r.done, "{:?}", r.error);
    let auto = run(NETWORK, all, json!({}), vec![Input::Enter]);
    let (a, b) = (Srf::of(&r.create[0]).unwrap(), Srf::of(&auto.create[0]).unwrap());
    for i in 0..=10 {
        for j in 0..=10 {
            let (u, v) = (i as f64 / 10.0, j as f64 / 10.0);
            let p = a.at_st(u, v);
            // Same surface (possibly with u and v named the other way).
            assert!(dist(p, b.at_st(u, v)) < 1e-6 || dist(p, b.at_st(v, u)) < 1e-6, "{p:?}");
        }
    }
}

/// Rhino: surface edges stand for curves. Two strips picked near their facing edges (x 4 on
/// the first, x 6 on the second) loft between those exact edges; EdgeSrf from the same two
/// edges makes the same ruled surface.
#[test]
fn surface_edges_as_curves() {
    let a = flat("a", 0.0, 4.0, 0.0, 5.0, 0.0);
    let b = flat("b", 6.0, 10.0, 0.0, 5.0, 2.0);
    for cmd in [LOFT, EDGE_CURVES] {
        let mut e = event(cmd, Input::Start, vec![], json!({}));
        let r = Surfaces::step(&e).unwrap();
        e.state = r.state;
        e.input = Input::Selection;
        e.selection = vec![a.clone()];
        e.pick = Some([3.9, 2.0, 0.0]);
        let r = Surfaces::step(&e).unwrap();
        e.state = r.state;
        e.selection = vec![a.clone(), b.clone()];
        e.pick = Some([6.1, 3.0, 2.0]);
        let r = Surfaces::step(&e).unwrap();
        e.state = r.state;
        e.input = Input::Enter;
        e.pick = None;
        let mut r = Surfaces::step(&e).unwrap();
        // Loft asks for options after the curves: Enter again.
        if !r.done {
            e.state = r.state;
            r = Surfaces::step(&e).unwrap();
        }
        assert!(r.done, "{cmd}: {:?} {}", r.error, r.prompt);
        assert!(r.delete.is_empty() && r.replace.is_empty());
        let s = Srf::of(&r.create[0]).unwrap();
        // Its edges: the line x 4, z 0 and the line x 6, z 2 (y 0..5), exactly.
        let ends: Vec<Vec3> = [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)].iter().map(|(u, v)| s.at_st(*u, *v)).collect();
        for p in &ends {
            assert!(((p[0] - 4.0).abs() < 1e-9 && p[2].abs() < 1e-9) || ((p[0] - 6.0).abs() < 1e-9 && (p[2] - 2.0).abs() < 1e-9), "{cmd}: {ends:?}");
        }
        let mid = s.at_st(0.5, 0.5);
        assert!((mid[0] - 5.0).abs() < 1e-6 && (mid[2] - 1.0).abs() < 1e-6, "{cmd}: {mid:?}");
    }
}

/// Rhino 5 SplitAtTangents (Loft, Revolve): an L-shaped profile (a kink) makes one surface
/// per leg; without it, one surface.
#[test]
fn split_at_tangents_cuts_at_kinks() {
    let l = |id: &str, z: f64| exact_sel(id, Geo::Polyline { points: vec![[5.0, 0.0, z], [7.0, 0.0, z], [7.0, 0.0, z + 2.0]], closed: false }, false);
    // Revolve the L round z: a flat ring (z 0) and a cylinder (r 7).
    for (split, want) in [(false, 1), (true, 2)] {
        let r = run(REVOLVE, vec![l("p", 0.0)], json!({"rev_split_at_tangents": split}), vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0]), Input::Enter]);
        assert!(r.done, "{:?}", r.error);
        assert_eq!(r.create.len(), want, "revolve split {split}");
        if split {
            let flat = r.create.iter().any(|g| { let s = Srf::of(g).unwrap(); (0..=4).all(|i| s.at_st(i as f64 / 4.0, 0.5)[2].abs() < 1e-9) });
            let cyl = r.create.iter().any(|g| { let s = Srf::of(g).unwrap(); (0..=4).all(|i| { let p = s.at_st(i as f64 / 4.0, 0.5); (p[0].hypot(p[1]) - 7.0).abs() < 1e-9 }) });
            assert!(flat && cyl);
        }
    }
    // Loft two L sections (z 0 and z 5, same shape): two flat pieces.
    let r = run(LOFT, vec![l("a", 0.0), l("b", 5.0)], json!({"loft_split_at_tangents": true}), vec![Input::Enter]);
    assert!(r.done, "{:?}", r.error);
    assert_eq!(r.create.len(), 2, "{:?}", r.messages);
}

/// A trimmed plate as `brep_data` commands receive it: a flat 10 × 10 surface at height `z`,
/// its square outer loop (four lines as NURBS) and a hole of radius 1 round (5, 5) as one
/// exact circle.
fn trimmed_plate(id: &str, z: f64) -> Selected {
    let base = flat(id, 0.0, 10.0, 0.0, 10.0, z);
    let edge = |a: Vec3, b: Vec3| Geo::Nurbs { degree: 1, points: vec![a, b], weights: vec![1.0; 2], knots: vec![0.0, 0.0, 1.0, 1.0] };
    let c = |x: f64, y: f64| [x, y, z];
    let outer = vec![edge(c(0.0, 0.0), c(10.0, 0.0)), edge(c(10.0, 0.0), c(10.0, 10.0)), edge(c(10.0, 10.0), c(0.0, 10.0)), edge(c(0.0, 10.0), c(0.0, 0.0))];
    let hole = vec![exact::arc(c(5.0, 5.0), [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, TAU).to_geo()];
    Selected {
        kind: "surface".into(),
        exact: Some(Geo::Trimmed { surface: Box::new(base.exact.clone().unwrap()), loops: vec![outer, hole], display: None }),
        ..base
    }
}

/// Rhino: the edges of trimmed surfaces are curves too. Loft between the holes of two
/// trimmed plates (z 0 and z 3, picked near their hole edges) is the cylinder r 1 between them.
#[test]
fn trimmed_surface_edges_as_curves() {
    let (a, b) = (trimmed_plate("a", 0.0), trimmed_plate("b", 3.0));
    let mut e = event(LOFT, Input::Start, vec![], json!({}));
    let r = Surfaces::step(&e).unwrap();
    e.state = r.state;
    e.input = Input::Selection;
    e.selection = vec![a.clone()];
    e.pick = Some([6.05, 5.0, 0.0]);
    let r = Surfaces::step(&e).unwrap();
    e.state = r.state;
    e.selection = vec![a.clone(), b.clone()];
    e.pick = Some([5.0, 5.95, 3.0]);
    let r = Surfaces::step(&e).unwrap();
    e.state = r.state;
    e.input = Input::Enter;
    e.pick = None;
    let mut r = Surfaces::step(&e).unwrap();
    if !r.done {
        e.state = r.state;
        r = Surfaces::step(&e).unwrap();
    }
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    assert!(r.delete.is_empty() && r.replace.is_empty(), "the plates stay");
    assert_eq!(r.create.len(), 1);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=12 {
        for j in 0..=4 {
            let p = s.at_st(i as f64 / 12.0, j as f64 / 4.0);
            assert!(((p[0] - 5.0).hypot(p[1] - 5.0) - 1.0).abs() < 1e-9 && p[2] > -1e-9 && p[2] < 3.0 + 1e-9, "{p:?}");
        }
    }
    // A click near the outer border picks that straight edge (x 10), not the hole.
    let mut e = event(EDGE_CURVES, Input::Start, vec![], json!({}));
    let r = Surfaces::step(&e).unwrap();
    e.state = r.state;
    e.input = Input::Selection;
    e.selection = vec![a.clone()];
    e.pick = Some([9.9, 4.0, 0.0]);
    let r = Surfaces::step(&e).unwrap();
    let st: Value = r.state.clone();
    assert_eq!(st["picks"].as_array().map(Vec::len), Some(1), "{st}");
}

/// Rhino 5 Untrim: trimmed surfaces go back to their whole surface; KeepTrimObjects=Yes
/// leaves the trim curves as new objects.
#[test]
fn untrim_restores_the_surface_and_can_keep_the_trims() {
    let a = trimmed_plate("a", 0.0);
    let Some(Geo::Trimmed { surface, .. }) = a.exact.clone() else { unreachable!() };
    let r = run(UNTRIM, vec![a.clone()], json!({}), vec![Input::Enter]);
    assert_eq!(r.replace.len(), 1);
    assert_eq!(r.replace[0].geo, *surface, "the whole surface");
    assert!(r.create.is_empty());
    let keep = Input::Option { name: "KeepTrimObjects".into(), value: None };
    let r = run(UNTRIM, vec![a], json!({}), vec![keep, Input::Enter]);
    assert_eq!(r.replace.len(), 1);
    assert_eq!(r.create.len(), 5, "four border lines and the hole");
}

/// Rhino 5 ExtrudeCrv Solid=Yes: a closed planar curve is capped into a solid (a curve inside
/// it in its plane is a hole); an open curve still makes a surface. The faces are exact: the
/// walls are on the circles r 2 and r 1, the caps are planes at z 0 and z 3 bounded by both
/// circles.
#[test]
fn extrude_crv_solid_caps_closed_planar_curves() {
    let ring = |id: &str, r: f64| exact_sel(id, Geo::Circle { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: r }, true);
    let open = exact_sel("line", Geo::Line { start: [5.0, 0.0, 0.0], end: [8.0, 0.0, 0.0] }, false);
    let r = run(EXTRUDE_ALL, vec![ring("outer", 2.0), ring("hole", 1.0), open], json!({"ea_solid": true}), vec![text("3")]);
    assert_eq!(r.create.len(), 2, "one solid, one surface: {:?}", r.messages);
    let Some(Geo::Brep { faces, solid, display }) = r.create.iter().find(|g| matches!(g, Geo::Brep { .. })) else { panic!() };
    assert!(*solid && display.is_some());
    assert_eq!(faces.len(), 4, "two walls, two caps");
    let walls: Vec<Srf> = faces.iter().filter_map(Srf::of).collect();
    assert_eq!(walls.len(), 2);
    let mut radii: Vec<f64> = walls
        .iter()
        .map(|w| {
            let r0 = { let p = w.at_st(0.3, 0.0); p[0].hypot(p[1]) };
            for i in 0..=8 {
                for j in 0..=2 {
                    let p = w.at_st(i as f64 / 8.0, j as f64 / 2.0);
                    assert!((p[0].hypot(p[1]) - r0).abs() < 1e-9 && p[2] > -1e-9 && p[2] < 3.0 + 1e-9, "{p:?}");
                }
            }
            r0
        })
        .collect();
    radii.sort_by(f64::total_cmp);
    assert!((radii[0] - 1.0).abs() < 1e-9 && (radii[1] - 2.0).abs() < 1e-9, "{radii:?}");
    let caps: Vec<(&Geo, &Vec<Vec<Geo>>)> = faces.iter().filter_map(|f| match f { Geo::Trimmed { surface, loops, .. } => Some((surface.as_ref(), loops)), _ => None }).collect();
    let mut zs = Vec::new();
    for (plane, loops) in caps {
        let s = Srf::of(plane).unwrap();
        let z = s.at_st(0.0, 0.0)[2];
        assert!((0..=4).all(|i| (s.at_st(i as f64 / 4.0, 1.0 - i as f64 / 4.0)[2] - z).abs() < 1e-12), "flat");
        assert_eq!(loops.len(), 2, "outline and hole");
        for (lp, want) in loops.iter().zip([2.0, 1.0]) {
            for p in nurbs::samples(&lp[0], 16) {
                assert!((p[0].hypot(p[1]) - want).abs() < 1e-9 && (p[2] - z).abs() < 1e-12, "{p:?}");
            }
        }
        zs.push(z);
    }
    zs.sort_by(f64::total_cmp);
    assert_eq!(zs, vec![0.0, 3.0]);
    // Solid=No (the default): three surfaces, no solid.
    let r = run(EXTRUDE_ALL, vec![ring("outer", 2.0), ring("hole", 1.0)], json!({}), vec![text("3")]);
    assert_eq!(r.create.len(), 2);
    assert!(r.create.iter().all(|g| matches!(g, Geo::Surface { .. })));
}

/// Rhino subobject picks: edges of a polysurface picked with Ctrl+Shift+click (`sub`) stand
/// for their exact curves. Loft between two picked circular edges is the cylinder between them.
#[test]
fn picked_edges_of_a_polysurface_are_curves() {
    let circle = |z: f64| exact::arc([0.0, 0.0, z], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, TAU).to_geo();
    let body = Selected {
        id: "body".into(),
        kind: "brep".into(),
        name: "body".into(),
        sub: vec![
            om9_addon_abi::session::SubPick { face: None, edge: Some(3), exact: Some(circle(0.0)) },
            om9_addon_abi::session::SubPick { face: None, edge: Some(7), exact: Some(circle(5.0)) },
        ],
        ..Default::default()
    };
    let mut e = event(LOFT, Input::Start, vec![body], json!({}));
    let mut r = Surfaces::step(&e).unwrap();
    for _ in 0..3 {
        if r.done {
            break;
        }
        e.state = r.state.clone();
        e.input = Input::Enter;
        r = Surfaces::step(&e).unwrap();
    }
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    assert!(r.delete.is_empty() && r.replace.is_empty(), "the body stays");
    assert_eq!(r.create.len(), 1);
    let s = Srf::of(&r.create[0]).unwrap();
    for i in 0..=12 {
        for j in 0..=4 {
            let p = s.at_st(i as f64 / 12.0, j as f64 / 4.0);
            assert!((p[0].hypot(p[1]) - 2.0).abs() < 1e-9 && p[2] > -1e-9 && p[2] < 5.0 + 1e-9, "{p:?}");
        }
    }
}
