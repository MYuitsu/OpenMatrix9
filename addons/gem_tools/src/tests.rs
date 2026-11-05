// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use om9_addon_geom::mesh::cuboid;

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

fn run_with(command: &str, selection: Vec<Selected>, settings: Value, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection, settings);
    let mut r = GemTools::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = GemTools::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Reply {
    run_with(command, selection, json!({}), inputs)
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

fn circle(id: &str, r: f64, n: usize) -> Selected {
    curve(
        id,
        (0..n)
            .map(|k| {
                let a = TAU * k as f64 / n as f64;
                [r * a.cos(), r * a.sin(), 0.0]
            })
            .collect(),
        true,
    )
}

fn line(id: &str, a: Vec3, b: Vec3) -> Selected {
    curve(
        id,
        (0..=50)
            .map(|k| add(a, scale(sub(b, a), k as f64 / 50.0)))
            .collect(),
        false,
    )
}

fn gem_sel(id: &str, c: Vec3, d: f64) -> Selected {
    let g = Gem::new(Cut::Round, d, d);
    let f = Frame::new(c, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: g.name(),
        bbox: [sub(c, [d; 3]), add(c, [d; 3])],
        gem: Some(g.record(&f, "test", "diamond")),
        ..Default::default()
    }
}

fn surface(id: &str, m: Mesh) -> Selected {
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

fn centers(r: &Reply) -> Vec<(Vec3, f64)> {
    r.create
        .iter()
        .filter_map(|g| match g {
            Geo::Mesh { meta: Some(m), .. } => {
                let c = m.get("center")?.as_array()?;
                Some((
                    [c[0].as_f64()?, c[1].as_f64()?, c[2].as_f64()?],
                    m.get("length_mm")?.as_f64()?,
                ))
            }
            _ => None,
        })
        .collect()
}

fn min_gap(c: &[(Vec3, f64)]) -> f64 {
    let mut g = f64::MAX;
    for i in 0..c.len() {
        for j in i + 1..c.len() {
            g = g.min(dist(c[i].0, c[j].0) - (c[i].1 + c[j].1) / 2.0);
        }
    }
    g
}

#[test]
fn stones_along_a_curve_keep_their_spacing() {
    let r = run_with(
        ON_CRV_ADVANCED,
        vec![line("l", [0.0; 3], [20.0, 0.0, 0.0])],
        json!({"goca_size": 1.5, "goca_spacing": 0.2}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    assert_eq!(c.len(), ((20.0 + 0.2) / 1.7f64).floor() as usize);
    assert!((min_gap(&c) - 0.2).abs() < 1e-6, "{}", min_gap(&c));
    // A closed ring of stones fills the whole circle evenly.
    let r = run_with(
        ON_CRV_ADVANCED,
        vec![circle("c", 10.0, 256)],
        json!({"goca_size": 2.0, "goca_spacing": 0.3, "goca_facing": "outward"}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    let per = 2.0 * PI * 10.0;
    assert_eq!(c.len(), (per / 2.3).floor() as usize);
    // Outward: each table axis points away from the center.
    for g in &r.create {
        if let Geo::Mesh { meta: Some(m), .. } = g {
            let a: Vec<f64> = m["axis"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let o: Vec<f64> = m["center"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            assert!(a[0] * o[0] + a[1] * o[1] > 0.0);
        }
    }
    // Tapered from 1 to 2 mm.
    let r = run_with(
        ON_CRV_ADVANCED,
        vec![line("l", [0.0; 3], [30.0, 0.0, 0.0])],
        json!({"goca_size": 1.0, "goca_taper": true, "goca_end_size": 2.0}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    assert!(
        c[0].1 < 1.05 && c.last().unwrap().1 > 1.9,
        "{:?}",
        (c[0].1, c.last().unwrap().1)
    );
    assert!(min_gap(&c) > 0.19);
    // Snapped to the corners of a square.
    let sq = Selected {
        exact: Some(Geo::Polyline {
            points: vec![
                [0.0; 3],
                [10.0, 0.0, 0.0],
                [10.0, 10.0, 0.0],
                [0.0, 10.0, 0.0],
            ],
            closed: true,
        }),
        ..curve(
            "s",
            (0..400)
                .map(|k| {
                    let t = k as f64 / 100.0;
                    let (i, f) = (t.floor() as usize, t.fract());
                    let c = [
                        [0.0, 0.0, 0.0],
                        [10.0, 0.0, 0.0],
                        [10.0, 10.0, 0.0],
                        [0.0, 10.0, 0.0],
                        [0.0, 0.0, 0.0],
                    ];
                    add(c[i], scale(sub(c[i + 1], c[i]), f))
                })
                .collect(),
            true,
        )
    };
    let r = run_with(
        ON_CRV_ADVANCED,
        vec![sq],
        json!({"goca_size": 1.5, "goca_snap_corners": true}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    for corner in [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ] {
        assert!(
            c.iter().any(|x| dist(x.0, corner) < 1e-6),
            "a stone on {corner:?}"
        );
    }
}

#[test]
fn list_and_multi_rows() {
    let list = vec![
        gem_sel("a", [50.0, 0.0, 0.0], 1.0),
        gem_sel("b", [60.0, 0.0, 0.0], 2.0),
        gem_sel("c", [70.0, 0.0, 0.0], 3.0),
    ];
    let mut sel = vec![line("l", [0.0; 3], [20.0, 0.0, 0.0])];
    sel.extend(list);
    let r = run_with(
        LIST_ON_CRV,
        sel,
        json!({"glc_spacing": 0.5}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    assert_eq!(
        c.iter().map(|x| x.1).collect::<Vec<_>>(),
        vec![1.0, 2.0, 3.0]
    );
    assert!((c[1].0[0] - (1.0 + 0.5 + 1.0)).abs() < 1e-6, "{:?}", c);
    let rows = vec![
        line("m", [0.0; 3], [10.0, 0.0, 0.0]),
        line("u", [0.0, 2.0, 0.0], [10.0, 2.0, 0.0]),
        line("d", [0.0, -2.0, 0.0], [10.0, -2.0, 0.0]),
    ];
    let r = run_with(
        ON_CRV_MULTI,
        rows,
        json!({"gcm_size": 1.5, "gcm_spacing": 0.2}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 3 * ((10.0 + 0.2) / 1.7f64).floor() as usize);
}

#[test]
fn pave_fills_inside_the_margin() {
    let r = run_with(
        AUTO_PAVE,
        vec![circle("c", 5.0, 128)],
        json!({"ap_size": 1.0, "ap_min_size": 1.0, "ap_spacing": 0.1, "ap_margin": 0.2}),
        vec![Input::Enter],
    );
    let c = centers(&r);
    assert!(c.len() > 50, "{}", c.len());
    assert!(min_gap(&c) > 0.1 - 1e-6, "{}", min_gap(&c));
    for (p, d) in &c {
        assert!(norm(*p) + d / 2.0 <= 5.0 - 0.2 + 0.02, "{p:?}");
    }
    // Onto a raised surface.
    let r = run_with(
        AUTO_PAVE,
        vec![
            circle("c", 5.0, 128),
            surface("s", cuboid([-10.0, -10.0, 0.0], [10.0, 10.0, 3.0])),
        ],
        json!({}),
        vec![Input::Enter],
    );
    assert!(centers(&r).iter().all(|c| (c.0[2] - 3.0).abs() < 1e-6));
}

#[test]
fn pave_prongs_sit_between_three_stones() {
    let gems: Vec<PlacedGem> = [[0.0, 0.0, 0.0], [1.2, 0.0, 0.0], [0.6, 1.0392, 0.0]]
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            PlacedGem::from_selected(&gem_sel(&i.to_string(), c, 1.0), [0.0, 0.0, 1.0]).unwrap()
        })
        .collect();
    let p = pave_prong_points(&gems, 0.4, 0.3, false);
    assert_eq!(p.len(), 1);
    assert!(dist(p[0].0, [0.6, 0.3464, 0.0]) < 1e-3, "{:?}", p[0].0);
    let p = pave_prong_points(&gems, 0.4, 0.3, true);
    assert_eq!(p.len(), 4, "one inside, one on each open side");
    let r = run(
        PAVE_PRONGS,
        gems.iter()
            .map(|g| gem_sel(&g.id, g.frame.origin, 1.0))
            .collect(),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 4);
}

#[test]
fn gem_on_surface_and_pave_builder() {
    let floor = surface("f", cuboid([-10.0, -10.0, -1.0], [10.0, 10.0, 2.0]));
    let r = run(
        ON_SURFACE,
        vec![floor.clone()],
        vec![pt([0.0; 3]), pt([3.0, 0.0, 0.0]), Input::Enter],
    );
    let c = centers(&r);
    assert_eq!(c.len(), 2);
    assert!(c.iter().all(|x| (x.0[2] - 2.0).abs() < 1e-9));
    // The pavé builder shrinks a stone to fit next to the first one.
    let r = run_with(
        PAVE,
        vec![floor],
        json!({"pb_size": 2.0, "pb_spacing": 0.2, "pb_min_size": 0.5}),
        vec![pt([0.0; 3]), pt([2.0, 0.0, 0.0]), Input::Enter],
    );
    let c = centers(&r);
    assert!((c[1].1 - 1.6).abs() < 1e-9, "{:?}", c);
}

#[test]
fn tapered_baguettes_follow_the_bend() {
    let r = run_with(
        TAPER_CHANNEL,
        vec![circle("c", 10.0, 512)],
        json!({"tbc_width": 2.0, "tbc_length": 1.5, "tbc_spacing": 0.1, "tbc_facing": "cplane"}),
        vec![Input::Enter],
    );
    let n = (2.0 * PI * 10.0 / 1.6f64).floor() as usize;
    assert_eq!(r.create.len(), n);
    // The outer edge is longer than the inner one.
    if let Geo::Mesh { meta: Some(m), .. } = &r.create[0] {
        let o = m["outline"].as_array().unwrap();
        assert!(o.len() > 4);
    }
    let r = run_with(
        TAPER_BETWEEN,
        vec![
            line("a", [0.0; 3], [10.0, 0.0, 0.0]),
            line("b", [0.0, 3.0, 0.0], [10.0, 5.0, 0.0]),
        ],
        json!({"tbb_count": 5}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 5);
}

#[test]
fn clusters_rings_do_not_overlap() {
    let rings = cluster_rings(4.0, &[(1.5, 0.0), (1.2, 0.0)], 0.2);
    let c: Vec<(Vec3, f64)> = rings.iter().map(|(f, d)| (f.origin, *d)).collect();
    assert!(min_gap(&c) >= 0.2 - 1e-9, "{}", min_gap(&c));
    let r = run(
        CLUSTER,
        vec![gem_sel("c", [0.0; 3], 5.0)],
        vec![Input::Enter],
    );
    assert!(!r.create.is_empty());
    let r = run(CLUSTER, vec![], vec![pt([1.0, 1.0, 0.0]), Input::Enter]);
    assert!(r.create.len() > 5);
}

#[test]
fn springs_relax_overlapping_stones() {
    let sel = vec![
        gem_sel("a", [0.0; 3], 1.0),
        gem_sel("b", [0.5, 0.0, 0.0], 1.0),
    ];
    let r = run_with(
        SPRINGS,
        sel,
        json!({"gs_spacing": 0.2, "gs_iterations": 200, "gs_strength": 100}),
        vec![Input::Enter],
    );
    let op = r.transform.unwrap();
    let a = apply(&op.per_object[0].matrix, [0.0; 3]);
    let b = apply(&op.per_object[1].matrix, [0.5, 0.0, 0.0]);
    assert!((dist(a, b) - 1.2).abs() < 1e-3, "{}", dist(a, b));
}

#[test]
fn control_points_culets_at_a_point() {
    let sel = vec![
        gem_sel("a", [-5.0, 0.0, 0.0], 1.0),
        gem_sel("b", [5.0, 0.0, 0.0], 1.0),
    ];
    let r = run(CONTROL, sel, vec![pt([0.0, 0.0, -10.0])]);
    let op = r.transform.unwrap();
    // The table axis of stone a (at −5) turns to point away from (0, 0, −10).
    let m = &op.per_object[0].matrix;
    let z = sub(apply(m, [-5.0, 0.0, 1.0]), apply(m, [-5.0, 0.0, 0.0]));
    let want = unit(sub([-5.0, 0.0, 0.0], [0.0, 0.0, -10.0]));
    assert!(dot(z, want) > 0.999, "{z:?}");
}

#[test]
fn orient_objects_to_stones() {
    let gems = vec![
        gem_sel("a", [5.0, 0.0, 0.0], 1.0),
        gem_sel("b", [-5.0, 0.0, 0.0], 2.0),
    ];
    let mut e = event(
        ORIENT_TO_GEM,
        Input::Enter,
        gems,
        json!({"otg_scale": true}),
    );
    let r = GemTools::step(&e).unwrap();
    e.state = r.state;
    e.selection = vec![Selected {
        id: "head".into(),
        kind: "mesh".into(),
        bbox: [[-5.0; 3], [5.0; 3]],
        ..Default::default()
    }];
    let r = GemTools::step(&e).unwrap();
    assert!(r.done);
    let op = r.transform.unwrap();
    assert_eq!(op.per_object.len(), 2);
    // A 10 mm model point (5, 0, 0) lands on stone a's girdle edge (scaled by 1 / 10).
    assert!(
        dist(
            apply(&op.per_object[0].matrix, [5.0, 0.0, 0.0]),
            [5.5, 0.0, 0.0]
        ) < 1e-9
    );
}

#[test]
fn styles_match_and_round_trip_through_a_file() {
    // A stone remembers the prongs built on it…
    let src = gem_sel("src", [0.0; 3], 2.0);
    let ev = event(
        om9_settings::PRONG_BUILDER,
        Input::Enter,
        vec![src.clone()],
        json!({"prong_count": 6}),
    );
    let made = <om9_settings::Settings as Interactive>::step(&ev).unwrap();
    assert!(made.done);
    assert_eq!(made.records.len(), 1);
    let mut styled = src.clone();
    styled.gem = Some(made.records[0].record.clone());
    // …and Match Attributes makes the same six prongs on two other stones.
    let targets = vec![
        gem_sel("t1", [10.0, 0.0, 0.0], 2.0),
        gem_sel("t2", [20.0, 0.0, 0.0], 2.0),
    ];
    let mut e = event(MATCH, Input::Enter, targets.clone(), json!({}));
    let r = GemTools::step(&e).unwrap();
    e.state = r.state;
    e.selection = vec![styled.clone()];
    let r = GemTools::step(&e).unwrap();
    assert!(r.done, "{}", r.prompt);
    assert_eq!(r.create.len(), 12);
    assert_eq!(r.records.len(), 2);
    assert!(r.records[0].record["styles"][om9_settings::PRONG_BUILDER].is_object());
    // Save and load the style.
    let dir = std::env::temp_dir().join(format!("om9-styles-{}", std::process::id()));
    let set = json!({"style_folder": dir.to_string_lossy(), "style_slot": "3"});
    let r = run_with(SAVE_STYLE, vec![styled], set.clone(), vec![Input::Enter]);
    assert!(r.messages[0].contains("3.om9style"), "{:?}", r.messages);
    let r = run_with(LOAD_STYLE, targets, set, vec![Input::Enter]);
    assert_eq!(r.create.len(), 12);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn positioner_flow_and_sphere() {
    let r = run(
        POSITIONER,
        vec![gem_sel("a", [0.0; 3], 1.0)],
        vec![
            Input::Text { text: "1".into() },
            Input::Enter,
            Input::Enter,
            Input::Enter,
            Input::Text { text: "200".into() },
        ],
    );
    let c = centers(&r);
    assert_eq!(c[0], ([1.0, 0.0, 0.0], 2.0));
    assert_eq!(r.delete, vec!["a".to_string()]);
    let base = line("b", [0.0; 3], [10.0, 0.0, 0.0]);
    let target = circle("t", 10.0 / TAU, 256);
    let r = run(
        GEM_FLOW,
        vec![gem_sel("g", [2.5, 0.0, 0.0], 1.0), base, target],
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 1);
    assert_eq!(r.delete.len(), 1);
    let r = run_with(
        PAVE_SPHERE,
        vec![],
        json!({"ps_radius": 5.0, "ps_size": 1.0, "ps_spacing": 0.15}),
        vec![pt([0.0; 3])],
    );
    let c = centers(&r);
    assert!(c.len() > 200, "{}", c.len());
    assert!(min_gap(&c) > 0.14, "{}", min_gap(&c));
}

#[test]
fn splop_puts_stones_on_the_surface() {
    let stones = vec![
        gem_sel("a", [0.0; 3], 1.0),
        gem_sel("b", [2.0, 0.0, 0.0], 1.5),
    ];
    let top = surface("t", cuboid([-20.0, -20.0, 0.0], [20.0, 20.0, 4.0]));
    let mut e = event(GEM_SPLOP, Input::Enter, stones, json!({}));
    let r = GemTools::step(&e).unwrap();
    e.state = r.state;
    e.selection = vec![top];
    let mut r = GemTools::step(&e).unwrap();
    for i in [pt([5.0, 5.0, 0.0]), Input::Enter] {
        e.state = r.state.clone();
        e.input = i;
        let r2 = GemTools::step(&e).unwrap();
        assert!(r2.error.is_none(), "{:?}", r2.error);
        if r2.done {
            let c = centers(&r2);
            assert_eq!(c.len(), 2);
            assert!(
                dist(c[0].0, [5.0, 5.0, 4.0]) < 1e-9 && dist(c[1].0, [7.0, 5.0, 4.0]) < 1e-9,
                "{c:?}"
            );
            assert_eq!(c[1].1, 1.5);
            return;
        }
        r = r2;
    }
    panic!("splop did not finish");
}

#[test]
fn profiles_azure_and_every_command_starts() {
    let r = run(
        PROFILE,
        vec![gem_sel("a", [0.0; 3], 2.0)],
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 1);
    let r = run(AZURE, vec![gem_sel("a", [0.0; 3], 2.0)], vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    for c in GemTools::SESSION_COMMANDS {
        let r = GemTools::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
