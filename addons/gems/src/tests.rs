// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::{Plane, Selected};
use serde_json::Value;

pub fn event(command: &str, input: Input, settings: Value, selection: Vec<Selected>) -> Event {
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

/// A selected stone placed with `frame`.
pub fn stone(id: &str, gem: &Gem, frame: &Frame) -> Selected {
    let m = gem.mesh().placed(frame);
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: gem.name(),
        bbox: m.bbox(),
        points: Vec::new(),
        closed: false,
        gem: Some(gem.record(frame, "test", "diamond")),
        mesh: None,
        exact: None,
    }
}

pub fn curve(id: &str, points: Vec<Vec3>, closed: bool) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: "Curve".into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points,
        closed,
        gem: None,
        mesh: None,
        exact: None,
    }
}

fn run(mut e: Event, inputs: Vec<Input>) -> Reply {
    let mut r = Gems::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Gems::step(&e).unwrap();
    }
    r
}

fn meshes(r: &Reply) -> Vec<(&Vec<Vec3>, Option<&Value>)> {
    r.create
        .iter()
        .filter_map(|g| match g {
            Geo::Mesh { vertices, meta, .. } => Some((vertices, meta.as_ref())),
            _ => None,
        })
        .collect()
}

#[test]
fn loader_places_stones_where_clicked_with_records() {
    let e = event(
        GEM_LOADER,
        Input::Start,
        json!({"loader_cut": "oval", "loader_length": 7.0, "loader_width": 5.0}),
        vec![],
    );
    let r = Gems::step(&e).unwrap();
    assert!(r.prompt.starts_with("Gem Loader: Oval 7.00 × 5.00 mm"), "{}", r.prompt);
    assert_eq!(r.want, Want::Point);
    let r = run(
        e,
        vec![
            Input::Hover { point: [1.0, 0.0, 0.0] },
            Input::Point { point: [0.0, 0.0, 0.0] },
            Input::Text { text: "10,0".into() },
            Input::Undo,
            Input::Text { text: "20,5".into() },
            Input::Enter,
        ],
    );
    assert!(r.done);
    let m = meshes(&r);
    assert_eq!(m.len(), 2);
    let rec = m[1].1.unwrap();
    assert_eq!(rec["cut"], "oval");
    assert_eq!(rec["center"], json!([20.0, 5.0, 0.0]));
    assert_eq!(rec["length_mm"], 7.0);
    // The stone is 7 mm long along the CPlane x axis.
    let xs = m[1].0.iter().map(|v| v[0]);
    let (lo, hi) = xs.fold((f64::MAX, f64::MIN), |(l, h), x| (l.min(x), h.max(x)));
    assert!((hi - lo - 7.0).abs() < 0.05);
}

#[test]
fn loader_replaces_selected_stones() {
    let old = Gem::new(Cut::Round, 1.5, 1.5);
    let f = Frame::new([3.0, 4.0, 1.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let e = event(
        GEM_LOADER,
        Input::Enter,
        json!({"loader_cut": "princess", "loader_length": 2.0}),
        vec![stone("a", &old, &f)],
    );
    let r = Gems::step(&e).unwrap();
    assert!(r.done);
    assert_eq!(r.delete, vec!["a".to_string()]);
    let rec = meshes(&r)[0].1.unwrap();
    assert_eq!(rec["cut"], "princess");
    assert_eq!(rec["center"], json!([3.0, 4.0, 1.0]));
}

#[test]
fn emerald_and_baguette_builders_use_their_own_sizes() {
    let e = event(EMERALD, Input::Enter, json!({"emerald_length": 8.0, "emerald_width": 6.0}), vec![]);
    let r = Gems::step(&e).unwrap();
    let rec = meshes(&r)[0].1.unwrap();
    assert_eq!((rec["cut"].as_str(), rec["width_mm"].as_f64()), (Some("emerald"), Some(6.0)));
    let e = event(BAGUETTE, Input::Enter, json!({}), vec![]);
    let rec = meshes(&Gems::step(&e).unwrap())[0].1.unwrap().clone();
    assert_eq!(rec["cut"], "baguette");
}

#[test]
fn halo_rings_the_center_stone() {
    let center = Gem::new(Cut::Round, 6.5, 6.5);
    let f = Frame::new([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let settings = json!({"halo_size": 1.0, "halo_spacing": 0.15, "halo_offset": 0.3});
    let e = event(HALO, Input::Selection, settings.clone(), vec![stone("c", &center, &f)]);
    let r = Gems::step(&e).unwrap();
    assert!(!r.preview.is_empty());
    // Accent ring radius 3.25 + 0.3 + 0.5 = 4.05 → ⌊2π·4.05 / 1.15⌋ = 22 stones.
    assert!(r.prompt.contains("22 accent stones"), "{}", r.prompt);
    let e = event(HALO, Input::Enter, settings, vec![stone("c", &center, &f)]);
    let r = Gems::step(&e).unwrap();
    assert!(r.done);
    let accents: Vec<_> = meshes(&r).into_iter().filter(|m| m.1.is_some()).collect();
    assert_eq!(accents.len(), 22);
    for (_, rec) in &accents {
        let c: Vec<f64> = serde_json::from_value(rec.unwrap()["center"].clone()).unwrap();
        let radius = (c[0] * c[0] + c[1] * c[1]).sqrt();
        assert!((radius - 4.05).abs() < 0.01, "{radius}");
    }
    // Neighbouring accents keep at least the spacing between girdles.
    let centers: Vec<[f64; 3]> = accents
        .iter()
        .map(|(_, r)| serde_json::from_value(r.unwrap()["center"].clone()).unwrap())
        .collect();
    for w in centers.windows(2) {
        assert!(dist(w[0], w[1]) >= 1.15 - 1e-6, "{}", dist(w[0], w[1]));
    }
    // One metal frame, closed.
    let metal = r
        .create
        .iter()
        .find_map(|g| match g {
            Geo::Mesh { name, vertices, triangles, .. } if name == "Halo metal" => {
                Some(Mesh { vertices: vertices.clone(), triangles: triangles.clone() })
            }
            _ => None,
        })
        .unwrap();
    assert!(metal.is_closed() && metal.volume() > 0.0);
}

#[test]
fn reporter_totals_and_assigns_materials() {
    let f = |x: f64| Frame::new([x, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let big = Gem::new(Cut::Round, 6.5, 6.5);
    let small = Gem::new(Cut::Round, 1.5, 1.5);
    let sel = vec![stone("a", &big, &f(0.0)), stone("b", &small, &f(5.0)), stone("c", &small, &f(7.0))];
    let (lines, total) = report(
        &sel.iter().map(|s| PlacedGem::from_selected(s, [0.0, 0.0, 1.0]).unwrap()).collect::<Vec<_>>(),
        None,
    );
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("2 × Round 1.50 mm diamond"), "{lines:?}");
    assert!((total - (1.0219 + 2.0 * 0.01256)).abs() < 0.002, "{total}");
    let e = event(REPORTER, Input::Option { name: "Material".into(), value: Some("Ruby".into()) }, json!({}), sel.clone());
    let r = Gems::step(&e).unwrap();
    assert!(r.prompt.contains("3 stones"));
    let mut e2 = event(REPORTER, Input::Enter, json!({}), sel);
    e2.state = r.state;
    let r = Gems::step(&e2).unwrap();
    assert!(r.done && r.create.is_empty());
    assert_eq!(r.records.len(), 3);
    assert_eq!(r.records[0].record["material"], "ruby");
    assert!(r.messages.iter().any(|m| m.contains("ruby")), "{:?}", r.messages);
}

#[test]
fn guides_and_gem_map() {
    let g = Gem::new(Cut::Round, 2.0, 2.0);
    let f = Frame::new([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let e = event(GUIDES, Input::Enter, json!({"guides_offset": 0.5}), vec![stone("a", &g, &f)]);
    let r = Gems::step(&e).unwrap();
    let Geo::Polyline { points, closed } = &r.create[0] else { panic!() };
    assert!(closed);
    assert!((dist(points[0], [1.0, 1.0, 0.0]) - 1.5).abs() < 1e-3);
    let e = event(GEM_MAP, Input::Enter, json!({}), vec![stone("a", &g, &f)]);
    let r = Gems::step(&e).unwrap();
    assert!(matches!(r.create[0], Geo::Point { point } if point == [1.0, 1.0, 0.0]));
    assert!(r.messages.iter().any(|m| m.contains("#1 Round 2.00 mm at 1, 1, 0")), "{:?}", r.messages);
}

#[test]
fn stones_fill_the_gap_between_two_curves() {
    // Two lines 2 mm apart, then widening to 3 mm.
    let a: Vec<Vec3> = (0..=100).map(|i| [i as f64 * 0.2, 0.0, 0.0]).collect();
    let b: Vec<Vec3> = (0..=100).map(|i| [i as f64 * 0.2, 2.0 + i as f64 * 0.01, 0.0]).collect();
    let e = event(
        BETWEEN_CURVES,
        Input::Enter,
        json!({"g2c_margin": 0.0, "g2c_spacing": 0.1}),
        vec![curve("a", a, false), curve("b", b, false)],
    );
    let r = Gems::step(&e).unwrap();
    assert!(r.done, "{:?}", r.error);
    let stones = meshes(&r);
    assert!(stones.len() > 5);
    let sizes: Vec<f64> = stones.iter().map(|m| m.1.unwrap()["diameter_mm"].as_f64().unwrap()).collect();
    assert!(sizes.windows(2).all(|w| w[1] >= w[0] - 1e-9), "sizes grow with the gap {sizes:?}");
    assert!((sizes[0] - 2.0).abs() < 0.05, "{}", sizes[0]);
    // Only one curve: refused with the reason.
    let e = event(BETWEEN_CURVES, Input::Enter, json!({}), vec![curve("a", vec![[0.0; 3], [1.0, 0.0, 0.0]], false)]);
    assert!(Gems::step(&e).unwrap().error.unwrap().contains("exactly two"));
}

#[test]
fn custom_gem_and_update() {
    let square: Vec<Vec3> = vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 3.0, 0.0], [0.0, 3.0, 0.0]];
    let e = event(CUSTOM_GEM, Input::Enter, json!({}), vec![curve("s", square, true)]);
    let r = Gems::step(&e).unwrap();
    let rec = meshes(&r)[0].1.unwrap();
    assert_eq!(rec["cut"], "custom");
    let c: Vec<f64> = serde_json::from_value(rec["center"].clone()).unwrap();
    assert!((c[0] - 2.0).abs() < 1e-6 && (c[1] - 1.5).abs() < 1e-6, "{c:?}");
    let g = Gem::new(Cut::Pear, 6.0, 4.0);
    let f = Frame::new([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    let e = event(GEM_UPDATE, Input::Enter, json!({}), vec![stone("p", &g, &f)]);
    let r = Gems::step(&e).unwrap();
    assert_eq!(r.replace.len(), 1);
    assert_eq!(r.replace[0].id, "p");
}
