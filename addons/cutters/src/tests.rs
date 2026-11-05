// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::{Plane, Selected, apply};
use om9_addon_geom::gems::{Cut, Gem};
use serde_json::Value;

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

fn stone(id: &str, cut: Cut, l: f64, w: f64, at: Vec3) -> Selected {
    let gem = Gem::new(cut, l, w);
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

fn line(n: usize, d: f64, gap: f64) -> Vec<Selected> {
    (0..n)
        .map(|i| stone(&format!("g{i}"), Cut::Round, d, d, [i as f64 * (d + gap), 0.0, 0.0]))
        .collect()
}

fn meshes(r: &Reply) -> Vec<(Mesh, Option<String>)> {
    r.create
        .iter()
        .filter_map(|g| match g {
            Geo::Mesh { vertices, triangles, layer, .. } => Some((
                Mesh { vertices: vertices.clone(), triangles: triangles.clone() },
                layer.clone(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn library_styles_and_fancy_cutters() {
    let sel = vec![
        stone("r", Cut::Round, 2.0, 2.0, [0.0; 3]),
        stone("o", Cut::Oval, 6.0, 4.0, [10.0, 0.0, 0.0]),
    ];
    let e = event(LIBRARY, Input::Enter, json!({"lib_style": "azure"}), sel);
    let r = Cutters::step(&e).unwrap();
    let m = meshes(&r);
    assert_eq!(m.len(), 2);
    for (mesh, layer) in &m {
        assert!(mesh.is_closed() && mesh.volume() > 0.0);
        assert_eq!(layer.as_deref(), Some(CUTTING));
    }
    // The oval cutter follows the oval: 105 % of 6 × 4 mm at the girdle.
    let b = m[1].0.bbox();
    assert!(((b[1][0] - b[0][0]) - 6.3).abs() < 0.02, "{b:?}");
    assert!(((b[1][1] - b[0][1]) - 4.2).abs() < 0.05, "{b:?}");
    // Azure goes deeper than the default setting cutter.
    let deep = |style: &str| -library_params(style, 1.0, 1.0).lower_depth;
    assert!(deep("azure") < deep("setting"));
}

#[test]
fn scallops_around_a_bezel() {
    let e = event(BEZEL_CUTTER, Input::Enter, json!({"bzc_count": 8, "bzc_size": 1.0}), vec![stone("a", Cut::Round, 4.0, 4.0, [0.0; 3])]);
    let r = Cutters::step(&e).unwrap();
    let m = meshes(&r);
    assert_eq!(m.len(), 8);
    for (mesh, _) in &m {
        assert!(mesh.is_closed());
        let b = mesh.bbox();
        assert!((b[1][2] - 1.1).abs() < 1e-6 && (b[0][2] - 0.1).abs() < 1e-6, "axis 0.6 above, Ø1: {b:?}");
    }
}

#[test]
fn channel_follows_the_line_of_stones() {
    let e = event(CHANNEL, Input::Enter, json!({"ch_width": 100, "ch_extension": 0.5}), line(5, 1.5, 0.1));
    let r = Cutters::step(&e).unwrap();
    let (m, layer) = &meshes(&r)[0];
    assert_eq!(layer.as_deref(), Some(CUTTING));
    assert!(m.is_closed() && m.volume() > 0.0);
    let b = m.bbox();
    // From 0.75 + 0.5 before the first stone to 0.75 + 0.5 after the last (at x = 6.4).
    assert!((b[0][0] + 1.25).abs() < 1e-6 && (b[1][0] - 7.65).abs() < 1e-6, "{b:?}");
    assert!((b[1][1] - 0.75).abs() < 1e-6, "half the stone width: {b:?}");
    let e = event(BRIGHT_CHANNEL, Input::Enter, json!({}), line(3, 1.5, 0.1));
    assert!(meshes(&Cutters::step(&e).unwrap())[0].0.is_closed());
}

#[test]
fn micro_prong_notches_sit_in_the_gaps() {
    let e = event(MICRO_PRONG, Input::Enter, json!({}), line(4, 2.0, 0.2));
    let r = Cutters::step(&e).unwrap();
    let m = meshes(&r);
    assert_eq!(m.len(), 3);
    let b = m[0].0.bbox();
    let cx = (b[0][0] + b[1][0]) / 2.0;
    assert!((cx - 1.1).abs() < 1e-6, "between stones 0 and 1: {cx}");
}

#[test]
fn bright_grooves_on_both_sides() {
    let e = event(BRIGHT_CUTTER, Input::Enter, json!({"bc_offset": 0.1, "bc_width": 0.6}), line(3, 1.5, 0.1));
    let r = Cutters::step(&e).unwrap();
    let m = meshes(&r);
    assert_eq!(m.len(), 2);
    let ys: Vec<f64> = m.iter().map(|(x, _)| (x.bbox()[0][1] + x.bbox()[1][1]) / 2.0).collect();
    assert!((ys[0] + ys[1]).abs() < 1e-6 && ys[0].abs() > 0.75, "{ys:?}");
}

#[test]
fn cubes_where_clicked_and_plane_slab() {
    let mut e = event(PLANE_CUBE, Input::Point { point: [1.0, 2.0, 0.0] }, json!({"cube_x": 2, "cube_y": 2, "cube_z": 2}), vec![]);
    let r = Cutters::step(&e).unwrap();
    e.state = r.state;
    e.input = Input::Enter;
    let r = Cutters::step(&e).unwrap();
    let b = meshes(&r)[0].0.bbox();
    assert_eq!(b, [[0.0, 1.0, -1.0], [2.0, 3.0, 1.0]]);
    let e = event(PLANE_CUBE, Input::Enter, json!({"cube_mode": "plane", "cube_z": 3}), vec![]);
    let b = meshes(&Cutters::step(&e).unwrap())[0].0.bbox();
    assert_eq!((b[0][2], b[1][2]), (-3.0, 0.0));
}

#[test]
fn finger_rail_and_quad_flip() {
    let ring: Vec<Vec3> = (0..64)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / 64.0;
            [8.675 * a.cos(), 0.0, 8.675 * a.sin()]
        })
        .collect();
    let sel = Selected {
        id: "rail".into(),
        kind: "curve".into(),
        name: "Ring Rail".into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points: ring.clone(),
        closed: true,
        gem: None,
        mesh: None,
        exact: None,
    };
    let (c, r, n) = fit_circle(&ring).unwrap();
    assert!(norm(c) < 1e-9 && (r - 8.675).abs() < 1e-9 && n[1].abs() > 0.999);
    let e = event(FINGER_RAIL, Input::Enter, json!({"ftr_length": 20}), vec![sel.clone()]);
    let res = Cutters::step(&e).unwrap();
    let b = meshes(&res)[0].0.bbox();
    assert!((b[1][1] - b[0][1] - 20.0).abs() < 1e-6, "along the finger axis {b:?}");
    let e = event(QUAD_FLIP, Input::Enter, json!({}), vec![sel]);
    let r = Cutters::step(&e).unwrap();
    let op = r.transform.unwrap();
    assert!(op.keep_original);
    let p = [1.0, 2.0, 3.0];
    let images: Vec<Vec3> = op.matrices.iter().map(|m| apply(m, p)).collect();
    assert_eq!(images, vec![[-1.0, 2.0, 3.0], [1.0, -2.0, 3.0], [-1.0, -2.0, 3.0]]);
}
