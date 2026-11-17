// SPDX-License-Identifier: LGPL-2.1-or-later
//! Every Curve, Surface and Solid command, driven with common selections and inputs, either
//! makes or changes something, or says why not: never finishes with nothing and no message.
use om9_addon_abi::session::{Event, Geo, Input, Plane, Reply, Selected, SelectedMesh, Vec3};
use serde_json::Value;

fn sampled(id: &str, g: Geo, pts: Vec<Vec3>, closed: bool) -> Selected {
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    for p in &pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    Selected { id: id.into(), kind: "curve".into(), name: id.into(), bbox: [lo, hi], points: pts, closed, exact: Some(g), ..Default::default() }
}

fn line(id: &str, a: Vec3, b: Vec3) -> Selected {
    let pts = (0..=32).map(|k| { let t = k as f64 / 32.0; [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t] }).collect();
    sampled(id, Geo::Line { start: a, end: b }, pts, false)
}

fn circle(id: &str, c: Vec3, r: f64) -> Selected {
    let pts = (0..128).map(|k| { let a = std::f64::consts::TAU * k as f64 / 128.0; [c[0] + r * a.cos(), c[1] + r * a.sin(), c[2]] }).collect();
    sampled(id, Geo::Circle { center: c, x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: r }, pts, true)
}

fn plane_srf(id: &str) -> Selected {
    let n = 10;
    let mut vertices = vec![];
    let mut triangles = vec![];
    for i in 0..=n {
        for j in 0..=n {
            vertices.push([-10.0 + 20.0 * i as f64 / n as f64, -10.0 + 20.0 * j as f64 / n as f64, 0.0]);
        }
    }
    for i in 0..n {
        for j in 0..n {
            let a = (i * (n + 1) + j) as u32;
            let b = a + (n + 1) as u32;
            triangles.push([a, b, b + 1]);
            triangles.push([a, b + 1, a + 1]);
        }
    }
    let g = Geo::Surface {
        degree: [1, 1],
        size: [2, 2],
        points: vec![[-10.0, -10.0, 0.0], [-10.0, 10.0, 0.0], [10.0, -10.0, 0.0], [10.0, 10.0, 0.0]],
        weights: vec![],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
    };
    Selected { id: id.into(), kind: "surface".into(), name: id.into(), bbox: [[-10.0, -10.0, 0.0], [10.0, 10.0, 0.0]], mesh: Some(SelectedMesh { vertices, triangles }), exact: Some(g), ..Default::default() }
}

fn boxed(id: &str) -> Selected {
    let v: Vec<Vec3> = (0..8).map(|k| [if k & 1 == 0 { 0.0 } else { 10.0 }, if k & 2 == 0 { 0.0 } else { 10.0 }, if k & 4 == 0 { 0.0 } else { 10.0 }]).collect();
    let f = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
    let triangles = f.iter().flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]).collect();
    Selected { id: id.into(), kind: "solid".into(), name: id.into(), bbox: [[0.0; 3], [10.0; 3]], closed: true, mesh: Some(SelectedMesh { vertices: v, triangles }), ..Default::default() }
}

fn made_something(r: &Reply) -> bool {
    !r.create.is_empty() || !r.replace.is_empty() || !r.delete.is_empty() || r.transform.is_some() || !r.records.is_empty()
}

#[test]
#[ignore = "audit: prints every silent finish; run with --ignored --nocapture"]
fn no_command_finishes_silently() {
    let fixtures: Vec<(&str, Vec<Selected>)> = vec![
        ("none", vec![]),
        ("2 lines", vec![line("l1", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]), line("l2", [0.0, 10.0, 0.0], [10.0, 10.0, 0.0])]),
        ("crossing lines", vec![line("l1", [0.0, 0.0, 0.0], [10.0, 10.0, 0.0]), line("l2", [0.0, 10.0, 0.0], [10.0, 0.0, 0.0])]),
        ("circle", vec![circle("c1", [0.0; 3], 5.0)]),
        ("2 circles", vec![circle("c1", [0.0; 3], 5.0), circle("c2", [0.0, 0.0, 10.0], 3.0)]),
        ("plane", vec![plane_srf("s1")]),
        ("plane + circle", vec![plane_srf("s1"), circle("c1", [0.0, 0.0, 5.0], 3.0)]),
        ("box", vec![boxed("b1")]),
    ];
    let scripts: Vec<(&str, Vec<Input>)> = vec![
        ("enter", vec![Input::Enter, Input::Enter, Input::Enter]),
        ("points", vec![
            Input::Point { point: [1.0, 2.0, 0.0] },
            Input::Point { point: [6.0, 3.0, 0.0] },
            Input::Point { point: [7.0, 8.0, 0.0] },
            Input::Enter,
            Input::Point { point: [2.0, 9.0, 0.0] },
            Input::Enter,
            Input::Enter,
        ]),
        ("number", vec![Input::Point { point: [0.0; 3] }, Input::Text { text: "2".into() }, Input::Enter, Input::Text { text: "3".into() }, Input::Enter, Input::Enter]),
    ];
    let mut silent = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut panics = vec![];
    let mut total = 0;
    for cat in om9_matrix::CATEGORIES.iter().filter(|c| matches!(c.name, "Curve" | "Surface" | "Solid")) {
        for &cmd in cat.commands {
            for (fname, sel) in &fixtures {
                for (sname, inputs) in &scripts {
                    total += 1;
                    let res = std::panic::catch_unwind(|| {
                        let mut e = Event { command: cmd.into(), input: Input::Start, state: Value::Null, plane: Plane::default(), selection: sel.clone(), pick: None, settings: Default::default() };
                        let mut r = match om9_matrix::step(&e) { Ok(r) => r, Err(m) => return Some(format!("Err at start: {m}")) };
                        let mut last_prompt = r.prompt.clone();
                        for i in inputs {
                            if r.done || r.error.is_some() {
                                break;
                            }
                            e.state = r.state.clone();
                            e.input = i.clone();
                            r = match om9_matrix::step(&e) { Ok(r) => r, Err(m) => return Some(format!("Err: {m}")) };
                            if !r.prompt.is_empty() {
                                last_prompt = r.prompt.clone();
                            }
                        }
                        (r.done && !made_something(&r) && r.error.is_none() && r.messages.is_empty())
                            .then(|| format!("silent finish after {:?} (last prompt: {last_prompt})", e.input))
                    });
                    match res {
                        Ok(Some(why)) if why.starts_with("silent") => silent.entry(cmd.to_string()).or_default().push(format!("[{fname} / {sname}] {why}")),
                        Ok(_) => {}
                        Err(_) => panics.push(format!("{cmd} [{fname} / {sname}]")),
                    }
                }
            }
        }
    }
    println!("runs: {total}; commands with a silent finish: {}; panics: {}", silent.len(), panics.len());
    for (cmd, cases) in &silent {
        println!("SILENT {cmd}");
        for c in cases.iter().take(4) {
            println!("    {c}");
        }
    }
    for p in &panics {
        println!("PANIC {p}");
    }
}
