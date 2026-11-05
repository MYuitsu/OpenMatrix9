// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use om9_addon_abi::session::{apply, dist};
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
    let mut r = Builders::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Builders::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn curve(points: Vec<Vec3>, closed: bool) -> Selected {
    Selected {
        id: "c".into(),
        kind: "curve".into(),
        name: "c".into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points,
        closed,
        ..Default::default()
    }
}

fn inner_radius(m: &Mesh) -> f64 {
    // The smallest distance from the ring axis (world Y) of any vertex.
    m.vertices
        .iter()
        .map(|v| (v[0] * v[0] + v[2] * v[2]).sqrt())
        .fold(f64::MAX, f64::min)
}

#[test]
fn ring_sizes() {
    assert!((inner_diameter("us", 7.0) - 17.3196).abs() < 1e-9);
    assert!((inner_diameter("iso", 54.0) - 54.0 / PI).abs() < 1e-12);
    assert_eq!(inner_diameter("diameter", 18.0), 18.0);
}

#[test]
fn rings_are_closed_and_fit_the_finger() {
    for (cmd, set) in [
        (
            SUBD_RING,
            json!({"sdr_ring_size": 7, "sdr_profile": "comfort"}),
        ),
        (SUBD_RING, json!({"sdr_ring_size": 7, "sdr_profile": "d"})),
        (
            SUBD_RING,
            json!({"sdr_ring_size": 7, "sdr_profile": "knife"}),
        ),
        (SIGNET, json!({"sg_ring_size": 7})),
        (SUBD_SIGNET, json!({"sds_ring_size": 7})),
        (RAISED, json!({"rb_ring_size": 7})),
    ] {
        let r = run(cmd, vec![], set, vec![Input::Enter]);
        let m = mesh_of(&r.create[0]);
        assert!(m.is_closed(), "{cmd}");
        assert!(m.volume() > 0.0, "{cmd} {}", m.volume());
        let ir = inner_radius(&m);
        assert!((ir - 17.3196 / 2.0).abs() < 0.2, "{cmd}: inner radius {ir}");
    }
    // The signet's top is flat at the top height.
    let r = run(
        SIGNET,
        vec![],
        json!({"sg_ring_size": 7, "sg_top_height": 4.0}),
        vec![Input::Enter],
    );
    let b = mesh_of(&r.create[0]).bbox();
    assert!((b[1][2] - (17.3196 / 2.0 + 4.0)).abs() < 1e-6, "{b:?}");
}

#[test]
fn eternity_band_fills_the_circle() {
    let r = run(
        ETERNITY,
        vec![],
        json!({"et_ring_size": 7, "et_sizes": "2.0", "et_spacing": 0.1}),
        vec![Input::Enter],
    );
    let stones: Vec<&Geo> = r
        .create
        .iter()
        .filter(|g| matches!(g, Geo::Mesh { meta: Some(_), .. }))
        .collect();
    let cutters = r
        .create
        .iter()
        .filter(|g| matches!(g, Geo::Mesh { layer: Some(l), .. } if l == "Cutting"))
        .count();
    assert!(stones.len() > 25, "{}", stones.len());
    assert_eq!(cutters, stones.len());
    // Neighbouring girdles keep the gap.
    let c: Vec<Vec3> = stones
        .iter()
        .map(|g| match g {
            Geo::Mesh { meta: Some(m), .. } => {
                let a = m["center"].as_array().unwrap();
                [
                    a[0].as_f64().unwrap(),
                    a[1].as_f64().unwrap(),
                    a[2].as_f64().unwrap(),
                ]
            }
            _ => unreachable!(),
        })
        .collect();
    let d = dist(c[0], c[1]);
    assert!(d > 2.0 + 0.09, "{d}");
    // Alternating sizes close with whole repeats.
    let a = around_circle(10.0, &[2.0, 1.0], 0.1);
    assert_eq!(a.len() % 2, 0);
    assert_eq!(a[0].1, 2.0);
    assert_eq!(a[1].1, 1.0);
}

#[test]
fn text_rope_jump_and_knots() {
    let line = curve(
        (0..=100).map(|k| [k as f64 * 0.3, 0.0, 0.0]).collect(),
        false,
    );
    let r = run(
        TEXT_ON_CURVE,
        vec![line.clone()],
        json!({"toc_text": "AB", "toc_solid": false, "toc_height": 2.0}),
        vec![Input::Enter],
    );
    assert_eq!(
        r.create.len(),
        shapes::glyph('A').len() + shapes::glyph('B').len()
    );
    let r = run(
        TEXT_ON_CURVE,
        vec![line],
        json!({"toc_text": "OM9"}),
        vec![Input::Enter],
    );
    assert!(r.create.iter().all(|g| mesh_of(g).is_closed()));
    let r = run(ROPE, vec![], json!({"rope_strands": 3}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 3);
    assert!(r.create.iter().all(|g| mesh_of(g).is_closed()));
    let r = run(
        JUMP,
        vec![],
        json!({"jr_diameter": 5.0, "jr_wire": 1.0}),
        vec![Input::Enter],
    );
    let b = mesh_of(&r.create[0]).bbox();
    assert!(
        (b[1][0] - b[0][0] - 7.0).abs() < 0.05,
        "outside = inside + 2 wires: {b:?}"
    );
    // A figure-eight crosses itself once: one strand goes over, the other under.
    let eight: Vec<Vec3> = (0..200)
        .map(|k| {
            let t = TAU * k as f64 / 200.0;
            [10.0 * t.sin(), 5.0 * (2.0 * t).sin(), 0.0]
        })
        .collect();
    let flat: Vec<[f64; 2]> = eight.iter().map(|p| [p[0], p[1]]).collect();
    let z = weave(&flat, true, 1.0, 2.0);
    assert!(
        z.iter().cloned().fold(f64::MIN, f64::max) > 0.45
            && z.iter().cloned().fold(f64::MAX, f64::min) < -0.45
    );
    let r = run(
        FREEFORM_KNOT,
        vec![curve(eight, true)],
        json!({}),
        vec![Input::Enter],
    );
    assert!(mesh_of(&r.create[0]).is_closed());
    let r = run(
        KNOT,
        vec![],
        json!({}),
        vec![Input::Point { point: [0.0; 3] }],
    );
    assert!(mesh_of(&r.create[0]).volume() > 0.0);
}

#[test]
fn patterns_and_nautilus_copy_the_selection() {
    let obj = Selected {
        id: "o".into(),
        kind: "mesh".into(),
        bbox: [[0.0; 3], [1.0; 3]],
        ..Default::default()
    };
    let r = run(
        PATTERN,
        vec![obj.clone()],
        json!({"pat_x_repeat": 2, "pat_y_repeat": 2, "pat_size": 5.0}),
        vec![Input::Enter],
    );
    let op = r.transform.unwrap();
    assert!(op.keep_original);
    assert_eq!(op.matrices.len(), 2 * 2 * 8 - 1);
    // The diagonal mirror swaps x and y.
    let m = pattern_matrices(
        &event(PATTERN, Input::Enter, vec![], json!({})),
        5.0,
        1,
        1,
        true,
    );
    let p = apply(&m[4], [2.0, 1.0, 0.0]);
    assert!(dist(p, [1.0, 2.0, 0.0]) < 1e-9, "{p:?}");
    let r = run(
        NAUTILUS,
        vec![obj],
        json!({"nau_amount": 10}),
        vec![Input::Enter],
    );
    assert_eq!(r.transform.unwrap().matrices.len(), 10);
}

#[test]
fn mill_work_cutters() {
    let circle = curve(
        (0..128)
            .map(|k| {
                let t = TAU * k as f64 / 128.0;
                [10.0 * t.cos(), 10.0 * t.sin(), 0.0]
            })
            .collect(),
        true,
    );
    let r = run(
        MILL_WORK,
        vec![circle.clone()],
        json!({"mw_size": 0.5, "mw_spacing": 0.1}),
        vec![Input::Enter],
    );
    let n = (TAU * 10.0 / 0.6f64).floor() as usize;
    assert!(
        (r.create.len() as i64 - n as i64).abs() <= 1,
        "{} vs {n}",
        r.create.len()
    );
    let r = run(
        MILL_WORK_C,
        vec![circle.clone()],
        json!({}),
        vec![Input::Enter],
    );
    assert!(!r.create.is_empty());
    let r = run(
        MILL_AREA,
        vec![circle],
        json!({"mla_cross": false, "mla_spacing": 1.0}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 20);
    assert!(r.create.iter().all(|g| mesh_of(g).is_closed()));
}

#[test]
fn award_top_and_every_command_starts() {
    let r = run(AWARD, vec![], json!({"aw_stone": 4.0}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 2);
    assert!(mesh_of(&r.create[0]).volume() > 0.0);
    for c in Builders::SESSION_COMMANDS {
        let r = Builders::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
