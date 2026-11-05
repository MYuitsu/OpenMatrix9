// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::{Input, Plane, Selected};
use om9_addon_geom::gems::{Cut, Gem};
use om9_addon_geom::mesh::Frame;
use serde_json::{Value, json};

fn event(command: &str, input: Input, settings: Value, selection: Vec<Selected>) -> Event {
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

fn stone(id: &str, d: f64, at: Vec3) -> Selected {
    let gem = Gem::new(Cut::Round, d, d);
    let f = Frame::new(at, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: gem.name(),
        bbox: [[0.0; 3], [1.0; 3]],
        points: Vec::new(),
        closed: false,
        gem: Some(gem.record(&f, "test", "diamond")),
        mesh: None,
        exact: None,
    }
}

fn line_of_stones(n: usize, d: f64, gap: f64) -> Vec<Selected> {
    (0..n)
        .map(|i| stone(&format!("g{i}"), d, [i as f64 * (d + gap), 0.0, 0.0]))
        .collect()
}

fn meshes(r: &Reply) -> Vec<Mesh> {
    r.create
        .iter()
        .filter_map(|g| match g {
            Geo::Mesh { vertices, triangles, .. } => Some(Mesh {
                vertices: vertices.clone(),
                triangles: triangles.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn centroid_xy(m: &Mesh) -> [f64; 2] {
    let b = m.bbox();
    [(b[0][0] + b[1][0]) / 2.0, (b[0][1] + b[1][1]) / 2.0]
}

#[test]
fn four_prongs_hug_each_stone() {
    let e = event(PRONG_BUILDER, Input::Enter, json!({"prong_count": 4, "prong_angle": 45}), vec![stone("a", 2.0, [0.0; 3])]);
    let r = Settings::step(&e).unwrap();
    assert!(r.done);
    let m = meshes(&r);
    assert_eq!(m.len(), 4);
    // 25 % of 2 mm = 0.5 mm prongs, 30 % inset: centers at 1 + 0.25 × 0.4 = 1.1 mm.
    for p in &m {
        assert!(p.is_closed());
        let c = centroid_xy(p);
        let r = (c[0] * c[0] + c[1] * c[1]).sqrt();
        assert!((r - 1.1).abs() < 0.02, "{r}");
        assert!((c[0].abs() - c[1].abs()).abs() < 0.02, "at 45°: {c:?}");
        let b = p.bbox();
        assert!((b[1][2] - 0.5).abs() < 1e-6, "height 25 % above the girdle: {}", b[1][2]);
        assert!((b[0][2] + 1.4).abs() < 1e-6, "drop 70 % below");
    }
}

#[test]
fn shared_prongs_sit_between_neighbours() {
    let line = line_of_stones(5, 1.5, 0.15);
    let e = event(PRONG_BUILDER, Input::Enter, json!({"prong_mode": "shared2"}), line.clone());
    let r = Settings::step(&e).unwrap();
    // 4 gaps × 2 shared + 2 ends × 2 = 12 prongs.
    assert_eq!(meshes(&r).len(), 12, "{:?}", r.messages);
    let e = event(PRONG_BUILDER, Input::Enter, json!({"prong_mode": "shared1"}), line);
    let r = Settings::step(&e).unwrap();
    assert_eq!(meshes(&r).len(), 4 + 2);
    // A shared prong between stones 0 and 1 sits at x = 0.75 + 0.075.
    let xs: Vec<f64> = meshes(&r).iter().map(|m| centroid_xy(m)[0]).collect();
    assert!(xs.iter().any(|x| (x - 0.825).abs() < 0.02), "{xs:?}");
}

#[test]
fn a_ring_of_stones_has_no_end_prongs() {
    let n = 12;
    let radius = 5.0;
    let ring: Vec<Selected> = (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            stone(&format!("r{i}"), 2.0, [radius * a.cos(), radius * a.sin(), 0.0])
        })
        .collect();
    let e = event(PRONG_BUILDER, Input::Enter, json!({"prong_mode": "shared2"}), ring);
    assert_eq!(meshes(&Settings::step(&e).unwrap()).len(), 2 * n);
}

#[test]
fn bezel_wraps_the_stone() {
    let e = event(BEZEL, Input::Enter, json!({"bezel_thickness": 0.5}), vec![stone("a", 4.0, [0.0; 3])]);
    let r = Settings::step(&e).unwrap();
    let m = &meshes(&r)[0];
    assert!(m.is_closed() && m.volume() > 0.0);
    let b = m.bbox();
    assert!((b[1][0] - 2.5).abs() < 0.01, "outer radius = girdle + thickness: {}", b[1][0]);
    assert!((b[1][2] - 0.3).abs() < 1e-9, "lip above the girdle");
    // The stone's culet sits inside the bezel depth.
    let gem = Gem::new(Cut::Round, 4.0, 4.0);
    assert!(b[0][2] < gem.top_bottom().1);
}

#[test]
fn head_has_prongs_and_rails() {
    let e = event(HEAD, Input::Enter, json!({"head_count": 6, "head_rails": 2}), vec![stone("a", 6.5, [0.0; 3])]);
    let r = Settings::step(&e).unwrap();
    assert!(r.done);
    let m = &meshes(&r)[0];
    assert!(m.volume() > 0.0);
    assert!(r.messages[0].contains("6 prongs, 2 rails"), "{:?}", r.messages);
}

#[test]
fn beads_follow_the_curve() {
    let line: Vec<Vec3> = vec![[0.0; 3], [10.0, 0.0, 0.0]];
    assert_eq!(bead_positions(&line, false, 0.6, 0.05, 0.0, 1.0).len(), 15);
    let circle: Vec<Vec3> = (0..360)
        .map(|i| {
            let a = TAU * i as f64 / 360.0;
            [5.0 * a.cos(), 5.0 * a.sin(), 0.0]
        })
        .collect();
    let n = bead_positions(&circle, true, 0.6, 0.05, 0.0, 1.0).len();
    assert_eq!(n, (TAU * 5.0 / 0.65).floor() as usize);
    let sel = Selected {
        id: "c".into(),
        kind: "curve".into(),
        name: "c".into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points: line,
        closed: false,
        gem: None,
        mesh: None,
        exact: None,
    };
    let e = event(BEAD_ON_CURVE, Input::Enter, json!({}), vec![sel]);
    let r = Settings::step(&e).unwrap();
    assert_eq!(meshes(&r).len(), 15);
}

#[test]
fn channel_borders_run_both_sides() {
    let e = event(CHANNEL_BORDER, Input::Enter, json!({"cbc_offset": 0.2}), line_of_stones(4, 1.5, 0.1));
    let r = Settings::step(&e).unwrap();
    assert_eq!(r.create.len(), 2);
    let Geo::Nurbs { points, .. } = &r.create[0] else { panic!("{:?}", r.create[0]) };
    assert!((points[0][1].abs() - 0.95).abs() < 1e-6, "{:?}", points[0]);
}
