// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::SelectedMesh;
use om9_addon_geom::eval_geo;
use om9_addon_geom::mesh::{Mesh, cuboid, sphere};
use serde_json::Value;

fn event(command: &str, input: Input, selection: Vec<Selected>) -> Event {
    Event {
        command: command.into(),
        input,
        state: Value::Null,
        plane: Plane::default(),
        selection,
        pick: None,
        settings: serde_json::from_value(json!({})).unwrap(),
    }
}

fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection);
    let mut r = step_rec(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command} did not finish: {}", r.prompt);
    r
}

/// `CurveTools::step`, recording the shapes of every finished run.
fn step_rec(e: &Event) -> Result<Reply, String> {
    let r = CurveTools::step(e).map_err(|x| format!("{x:?}"))?;
    if r.done {
        audit(&e.command, &e.selection, &r);
    }
    Ok(r)
}

/// Shape audit: record the run's input and output curves (see `om9_addon_geom::audit`).
fn audit(command: &str, selection: &[Selected], r: &Reply) {
    let inputs: Vec<Geo> = selection.iter().filter_map(|s| s.exact.clone()).collect();
    let mut outputs = r.create.clone();
    outputs.extend(r.replace.iter().map(|x| x.geo.clone()));
    let test = std::thread::current().name().unwrap_or("").to_owned();
    om9_addon_geom::audit::record("curve_tools", &test, command, &inputs, &outputs);
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}
fn num(v: f64) -> Input {
    Input::Text { text: fmt(v) }
}

fn bbox(p: &[Vec3]) -> [Vec3; 2] {
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    for q in p {
        for i in 0..3 {
            lo[i] = lo[i].min(q[i]);
            hi[i] = hi[i].max(q[i]);
        }
    }
    [lo, hi]
}

fn curve(id: &str, geo: Geo, closed: bool) -> Selected {
    let points = nurbs::samples(&geo, 200);
    let mut points = points;
    if closed && dist(points[0], *points.last().unwrap()) < 1e-9 {
        points.pop();
    }
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: id.into(),
        bbox: bbox(&points),
        points,
        closed,
        exact: Some(geo),
        ..Default::default()
    }
}

fn line(id: &str, a: Vec3, b: Vec3) -> Selected {
    curve(id, Geo::Line { start: a, end: b }, false)
}

fn circle(id: &str, c: Vec3, r: f64) -> Selected {
    curve(
        id,
        Geo::Circle {
            center: c,
            radius: r,
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
        },
        true,
    )
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

fn length(g: &Geo) -> f64 {
    if let Geo::Polyline { points, closed } = g {
        let mut l: f64 = points.windows(2).map(|w| dist(w[0], w[1])).sum();
        if *closed {
            l += dist(points[0], *points.last().unwrap());
        }
        return l;
    }
    let p = nurbs::samples(g, 2000);
    p.windows(2).map(|w| dist(w[0], w[1])).sum()
}

#[test]
fn rebuild_and_refit_keep_the_shape() {
    let c = circle("c", [0.0; 3], 5.0);
    let start = step_rec(&event(REBUILD, Input::Start, vec![])).unwrap();
    assert_eq!(start.prompt, "Select curves to rebuild");
    let r = run(REBUILD, vec![c.clone()], vec![Input::Option { name: "PointCount".into(), value: Some("12".into()) }, Input::Enter]);
    let g = &r.replace[0].geo;
    assert!(matches!(g, Geo::Nurbs { degree: 3, points, .. } if points.len() == 12 + 3), "periodic 12 poles, clamped: {g:?}");
    for p in nurbs::samples(g, 100) {
        assert!((norm(p) - 5.0).abs() < 0.01, "{p:?}");
    }
    assert!(r.messages.iter().any(|m| m.contains("maximum deviation")), "{:?}", r.messages);
    // Open curve: the ends stay, the count is exact; DeleteInput=No adds a new curve.
    let s = curve("s", om9_addon_geom::interpolate(&[[0.0; 3], [3.0, 2.0, 0.0], [6.0, -1.0, 0.0], [10.0, 1.0, 0.0]], 3, false).unwrap(), false);
    let r = run(REBUILD, vec![s.clone()], vec![Input::Option { name: "PointCount".into(), value: Some("6".into()) }, Input::Option { name: "DeleteInput".into(), value: Some("No".into()) }, Input::Enter]);
    assert!(r.replace.is_empty());
    let Geo::Nurbs { points, .. } = &r.create[0] else { panic!() };
    assert_eq!(points.len(), 6);
    assert!(dist(points[0], [0.0; 3]) < 1e-12 && dist(points[5], [10.0, 1.0, 0.0]) < 1e-12);
    let r = run(REFIT, vec![c], vec![Input::Enter]);
    for p in nurbs::samples(&r.replace[0].geo, 300) {
        assert!((norm(p) - 5.0).abs() < 0.002, "{}", norm(p));
    }
}

#[test]
fn divide_by_count_length_and_split() {
    let l = line("l", [0.0; 3], [10.0, 0.0, 0.0]);
    let start = step_rec(&event(DIVIDE, Input::Start, vec![l.clone()])).unwrap();
    assert_eq!(start.prompt, "Number of segments <10>");
    let r = run(DIVIDE, vec![l.clone()], vec![num(5.0)]);
    assert_eq!(r.create.len(), 6);
    let r = run(DIVIDE, vec![l.clone()], vec![Input::Option { name: "MarkEnds".into(), value: None }, num(5.0)]);
    assert_eq!(r.create.len(), 4);
    let r = run(DIVIDE, vec![l.clone()], vec![Input::Option { name: "Length".into(), value: None }, num(3.0)]);
    let xs: Vec<f64> = r.create.iter().map(|g| match g { Geo::Point { point } => point[0], _ => f64::NAN }).collect();
    assert_eq!(xs.len(), 5);
    assert!((xs[3] - 9.0).abs() < 1e-6 && (xs[4] - 10.0).abs() < 1e-9, "{xs:?}");
    let r = run(DIVIDE, vec![l.clone()], vec![Input::Option { name: "Split".into(), value: Some("Yes".into()) }, num(4.0)]);
    assert_eq!(r.create.len(), 4);
    assert_eq!(r.delete, ["l"]);
    assert!(r.create.iter().all(|g| (length(g) - 2.5).abs() < 1e-6));
    let c = circle("c", [0.0; 3], 2.0);
    let r = run(DIVIDE, vec![c], vec![num(4.0)]);
    assert_eq!(r.create.len(), 4, "closed: no repeated end");
}

#[test]
fn convert_to_lines_and_arcs() {
    let c = circle("c", [0.0; 3], 5.0);
    let r = run(TO_LINES, vec![c.clone()], vec![Input::Enter]);
    let g = &r.replace[0].geo;
    let Geo::Polyline { points, closed: true } = g else { panic!("{g:?}") };
    // Chord deviation 0.01 and 5° turns: at least 72 sides.
    assert!(points.len() >= 72, "{}", points.len());
    for w in points.windows(2) {
        let mid = scale(add(w[0], w[1]), 0.5);
        assert!(5.0 - norm(mid) <= 0.0101, "{}", 5.0 - norm(mid));
    }
    let s = curve("s", om9_addon_geom::interpolate(&[[0.0; 3], [3.0, 2.0, 0.0], [6.0, -1.0, 0.0], [10.0, 1.0, 0.0]], 3, false).unwrap(), false);
    let r = run(TO_LINES, vec![s.clone()], vec![Input::Option { name: "Output".into(), value: Some("Arcs".into()) }, Input::Enter]);
    let src = om9_addon_geom::pick::CurveRef::of(&s).unwrap();
    for p in nurbs::samples(&r.replace[0].geo, 300) {
        assert!(dist(src.closest(p).1, p) < 0.011);
    }
}

#[test]
fn tween_picks_two_curves() {
    let a = circle("a", [0.0; 3], 2.0);
    let b = circle("b", [0.0; 3], 4.0);
    let picks = [(a.clone(), [2.0, 0.0, 0.0]), (b.clone(), [4.0, 0.0, 0.0])];
    let r = picking(TWEEN, &[("Number", "3")], &picks);
    assert_eq!(r.prompt, "Adjust the tween. Press Enter when done");
    assert_eq!(r.preview.len(), 3);
    let r = enter(TWEEN, r, &picks);
    for (k, g) in r.create.iter().enumerate() {
        let want = 2.0 + 0.5 * (k + 1) as f64;
        for p in nurbs::samples(g, 50) {
            assert!((norm(p) - want).abs() < 0.02, "{}", norm(p));
        }
    }
    // Opposite directions are matched start to start.
    let l1 = line("l1", [0.0; 3], [10.0, 0.0, 0.0]);
    let l2 = line("l2", [10.0, 4.0, 0.0], [0.0, 4.0, 0.0]);
    let picks = [(l1, [5.0, 0.0, 0.0]), (l2, [5.0, 4.0, 0.0])];
    let r = enter(TWEEN, picking(TWEEN, &[], &picks), &picks);
    let (p, q) = ends_of(&r.create[0]);
    assert!(dist(p, [0.0, 2.0, 0.0]) < 1e-6 && dist(q, [10.0, 2.0, 0.0]) < 1e-6, "{p:?} {q:?}");
}

fn enter(command: &str, mut r: Reply, picks: &[(Selected, Vec3)]) -> Reply {
    let mut e = event(command, Input::Enter, picks.iter().map(|p| p.0.clone()).collect());
    e.state = r.state.clone();
    e.pick = picks.last().map(|p| p.1);
    r = step_rec(&e).unwrap();
    assert!(r.error.is_none(), "{command}: {:?}", r.error);
    r
}

#[test]
fn blends_leave_the_picked_ends_tangentially() {
    let a = line("a", [-10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    let b = line("b", [5.0, 5.0, 0.0], [5.0, 15.0, 0.0]);
    let picks = [(a.clone(), [-1.0, 0.0, 0.0]), (b.clone(), [5.0, 6.0, 0.0])];
    for cmd in [BLEND, ARC_BLEND, BLEND_ADJ] {
        let mut r = picking(cmd, &[], &picks);
        if cmd != BLEND {
            assert!(!r.done && r.want == Want::Nothing, "{cmd}: {}", r.prompt);
            assert!(!r.preview.is_empty(), "{cmd} previews");
            r = enter(cmd, r, &picks);
        }
        assert!(r.done, "{cmd}: {}", r.prompt);
        let mut p = Vec::new();
        for g in &r.create {
            p.extend(nurbs::samples(g, 400));
        }
        assert!(dist(p[0], [0.0; 3]) < 1e-6, "{cmd} starts at the end of a: {:?}", p[0]);
        assert!(dist(*p.last().unwrap(), [5.0, 5.0, 0.0]) < 1e-6, "{cmd}");
        let n = p.len();
        let t0 = unit(sub(p[1], p[0]));
        let t1 = unit(sub(p[n - 1], p[n - 2]));
        assert!(dot(t0, [1.0, 0.0, 0.0]) > 0.999, "{cmd} leaves along +x: {t0:?}");
        assert!(dot(t1, [0.0, 1.0, 0.0]) > 0.999, "{cmd} arrives along +y: {t1:?}");
    }
    // Picking the far ends blends those instead.
    let r = picking(BLEND, &[], &[(a.clone(), [-9.0, 0.0, 0.0]), (b.clone(), [5.0, 14.0, 0.0])]);
    let (p, q) = ends_of(&r.create[0]);
    assert!(dist(p, [-10.0, 0.0, 0.0]) < 1e-6 && dist(q, [5.0, 15.0, 0.0]) < 1e-6, "{p:?} {q:?}");
    // Adjustable blend with Join: one curve.
    let r = picking(BLEND_ADJ, &[("Join", "Yes"), ("Continuity1", "Tangency")], &picks);
    let r = enter(BLEND_ADJ, r, &picks);
    assert!(r.create.is_empty() && r.delete == ["b"] && r.replace.len() == 1, "{r:?}");
    // Arc blend: two arcs, equal radii by default, a difference when asked.
    // (Ends on one circle make both arcs that circle: use ends that need two.)
    let b2 = line("b2", [7.0, 4.0, 0.0], [7.0, 14.0, 0.0]);
    let picks = [(a.clone(), [-1.0, 0.0, 0.0]), (b2, [7.0, 5.0, 0.0])];
    let r = picking(ARC_BLEND, &[("RadiusDifference", "2")], &picks);
    let r = enter(ARC_BLEND, r, &picks);
    let radii: Vec<f64> = r.create.iter().map(|g| match g { Geo::Arc { radius, .. } => *radius, _ => f64::NAN }).collect();
    assert!((radii[0] - radii[1] - 2.0).abs() < 1e-6, "{radii:?}");
}

#[test]
fn match_moves_the_picked_end_onto_the_other_curve() {
    let a = line("a", [-10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    let arc = curve("b", Geo::Arc { center: [3.0, 6.0, 0.0], x: [-1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 2.0, angle: 1.5 }, false);
    let picks = [(a.clone(), [-1.0, 0.0, 0.0]), (arc.clone(), eval_geo(&arc.exact.clone().unwrap(), 0.05).unwrap())];
    let r = picking(MATCH, &[], &picks);
    assert_eq!(r.options.iter().map(|o| o.name.as_str()).filter(|n| *n != "Undo").collect::<Vec<_>>(), ["Continuity", "PreserveOtherEnd", "AverageCurves", "Join", "Merge"]);
    let r = enter(MATCH, r, &picks);
    let g = replaced(&r, "a");
    let (p, q) = ends_of(g);
    assert!(dist(p, [-10.0, 0.0, 0.0]) < 1e-3, "other end kept: {p:?}");
    assert!(dist(q, [1.0, 6.0, 0.0]) < 1e-9, "{q:?}");
    // Tangent and curvature continue the arc (leaving its start backwards: -y).
    let c = om9_addon_geom::pick::CurveRef::of(&curve("m", g.clone(), false)).unwrap();
    let end = om9_addon_geom::blend::end_of(&c, 1.0, true);
    assert!(dist(end.d[0], [0.0, 1.0, 0.0]) < 1e-4, "{:?}", end.d[0]);
    assert!(dist(end.d[1], [0.5, 0.0, 0.0]) < 2e-2, "{:?}", end.d[1]);
}

#[test]
fn offsets_keep_their_distance() {
    let c = circle("c", [0.0; 3], 5.0);
    let r = run(OFFSET, vec![c.clone()], vec![num(1.0), pt([10.0, 0.0, 0.0])]);
    assert!(matches!(r.create[0], Geo::Circle { radius, .. } if (radius - 6.0).abs() < 1e-12), "outside: {:?}", r.create[0]);
    let r = run(OFFSET, vec![c], vec![num(1.0), pt([0.5, 0.0, 0.0])]);
    assert!(matches!(r.create[0], Geo::Circle { radius, .. } if (radius - 4.0).abs() < 1e-12), "inside");
    let l = line("l", [0.0; 3], [10.0, 0.0, 0.0]);
    let r = run(OFFSET, vec![l], vec![num(2.0), pt([5.0, 3.0, 0.0])]);
    assert_eq!(r.create[0], Geo::Line { start: [0.0, 2.0, 0.0], end: [10.0, 2.0, 0.0] });
}

#[test]
fn offset_prompts_and_polyline_corners() {
    let r = picking(OFFSET, &[], &[]);
    assert_eq!(r.prompt, "Select curve to offset");
    let names: Vec<String> = r.options.iter().map(|o| format!("{}={}", o.name, o.value.clone().unwrap_or_default())).collect();
    assert_eq!(names, ["Distance=1", "Corner=Sharp", "ThroughPoint=No", "Tolerance=0.001", "BothSides=No", "InCPlane=No", "Cap=None"]);
    // An open L: outside corner at (10, 0) when offsetting below/right.
    let poly = curve("p", Geo::Polyline { points: vec![[0.0; 3], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]], closed: false }, false);
    let side = [5.0, -3.0, 0.0];
    let go = |opts: &[(&str, &str)]| {
        let r = picking(OFFSET, opts, &[(poly.clone(), [5.0, 0.0, 0.0])]);
        assert_eq!(r.prompt, "Side to offset");
        let mut e = event(OFFSET, pt(side), vec![poly.clone()]);
        e.state = r.state.clone();
        let r = step_rec(&e).unwrap();
        assert!(r.done, "{:?}", r.error);
        r.create
    };
    let sharp = go(&[("Distance", "2")]);
    assert!(matches!(&sharp[0], Geo::Polyline { points, .. } if points == &vec![[0.0, -2.0, 0.0], [12.0, -2.0, 0.0], [12.0, 10.0, 0.0]]), "{sharp:?}");
    let round = go(&[("Distance", "2"), ("Corner", "Round")]);
    let rc = om9_addon_geom::pick::CurveRef::of(&curve("r", round[0].clone(), false)).unwrap();
    assert!((dist(rc.closest([12.0, -2.0, 0.0]).1, [10.0, 0.0, 0.0]) - 2.0).abs() < 1e-6, "round corner at distance 2");
    assert!((length(&round[0]) - (10.0 + 10.0 + std::f64::consts::PI)).abs() < 1e-2, "{}", length(&round[0]));
    let chamfer = go(&[("Distance", "2"), ("Corner", "Chamfer")]);
    assert!((length(&chamfer[0]) - (20.0 + 8f64.sqrt())).abs() < 1e-9, "{}", length(&chamfer[0]));
    let smooth = go(&[("Distance", "2"), ("Corner", "Smooth")]);
    assert!(length(&smooth[0]) > 20.0 && length(&smooth[0]) < 24.0);
    // Inside a corner the segments are trimmed whatever the corner style.
    let mut e = event(OFFSET, Input::Start, vec![]);
    let mut rr = picking(OFFSET, &[("Corner", "Round")], &[(poly.clone(), [5.0, 0.0, 0.0])]);
    e.selection = vec![poly.clone()];
    e.state = rr.state.clone();
    e.input = pt([5.0, 3.0, 0.0]);
    rr = step_rec(&e).unwrap();
    assert!(matches!(&rr.create[0], Geo::Polyline { points, .. } if dist(points[1], [9.0, 1.0, 0.0]) < 1e-12), "{:?}", rr.create);
    // BothSides with a flat cap: one closed outline.
    let both = go(&[("BothSides", "Yes"), ("Cap", "Flat")]);
    assert_eq!(both.len(), 1);
    let ends = ends_of(&both[0]);
    assert!(dist(ends.0, ends.1) < 1e-9, "closed: {ends:?}");
}

#[test]
fn offset_of_a_free_curve_stays_within_tolerance() {
    let g = om9_addon_geom::interpolate(&[[0.0; 3], [3.0, 2.0, 0.0], [6.0, -1.0, 0.0], [10.0, 1.0, 0.0]], 3, false).unwrap();
    let c = curve("s", g, false);
    let r = run(OFFSET, vec![c.clone()], vec![num(0.5), pt([3.0, 5.0, 0.0])]);
    let src = om9_addon_geom::pick::CurveRef::of(&c).unwrap();
    for p in nurbs::samples(&r.create[0], 200) {
        let d = dist(src.closest(p).1, p);
        assert!((d - 0.5).abs() < 2e-3, "{d}");
    }
}

/// Run a command that picks curves by clicking: each pick adds the curve to the selection
/// with the click point; `options` are typed first (`Radius=2`).
fn picking(command: &str, options: &[(&str, &str)], picks: &[(Selected, Vec3)]) -> Reply {
    let mut e = event(command, Input::Start, vec![]);
    let mut r = step_rec(&e).unwrap();
    for (name, value) in options {
        e.state = r.state.clone();
        e.input = Input::Option { name: (*name).into(), value: Some((*value).into()) };
        r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?}", r.error);
    }
    for (sel, at) in picks {
        e.state = r.state.clone();
        e.selection.push(sel.clone());
        e.pick = Some(*at);
        e.input = Input::Selection;
        r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    r
}

fn ends_of(g: &Geo) -> (Vec3, Vec3) {
    (eval_geo(g, 0.0).unwrap(), eval_geo(g, 1.0).unwrap())
}

fn replaced<'a>(r: &'a Reply, id: &str) -> &'a Geo {
    &r.replace.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("{id} not replaced")).geo
}

#[test]
fn fillet_keeps_the_picked_sides() {
    let a = line("a", [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line("b", [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    let r = picking(FILLET, &[], &[]);
    assert_eq!(r.prompt, "Select first curve to fillet");
    let names: Vec<String> = r.options.iter().map(|o| format!("{}={}", o.name, o.value.clone().unwrap_or_default())).collect();
    assert_eq!(names, ["Radius=1", "Join=No", "Trim=Yes", "ExtendArcsBy=Arc"]);
    let r = picking(FILLET, &[], &[(a.clone(), [5.0, 0.0, 0.0])]);
    assert_eq!(r.prompt, "Select second curve to fillet");
    // Picks on +x and +y: the corner in that quadrant, radius 2 around (2, 2).
    let r = picking(FILLET, &[("Radius", "2")], &[(a.clone(), [5.0, 0.0, 0.0]), (b.clone(), [0.0, 6.0, 0.0])]);
    assert!(r.done, "{}", r.prompt);
    for p in nurbs::samples(&r.create[0], 50) {
        assert!((dist(p, [2.0, 2.0, 0.0]) - 2.0).abs() < 1e-6, "{p:?}");
    }
    assert!((length(&r.create[0]) - std::f64::consts::PI).abs() < 1e-3);
    let (p, q) = ends_of(replaced(&r, "a"));
    assert!(dist(p, [2.0, 0.0, 0.0]) < 1e-6 && dist(q, [10.0, 0.0, 0.0]) < 1e-6, "{p:?} {q:?}");
    let (p, q) = ends_of(replaced(&r, "b"));
    assert!(dist(p, [0.0, 2.0, 0.0]) < 1e-6 && dist(q, [0.0, 10.0, 0.0]) < 1e-6, "{p:?} {q:?}");
    // The other quadrant: picks on -x and -y.
    let r = picking(FILLET, &[], &[(a.clone(), [-5.0, 0.0, 0.0]), (b.clone(), [0.0, -6.0, 0.0])]);
    let (p, _) = ends_of(&r.create[0]);
    assert!(dist(p, [-1.0, 0.0, 0.0]) < 1e-6, "{p:?}");
    // Join=Yes: one curve replaces both.
    let r = picking(FILLET, &[("Join", "Yes")], &[(a.clone(), [5.0, 0.0, 0.0]), (b.clone(), [0.0, 6.0, 0.0])]);
    assert!(r.create.is_empty() && r.delete == ["b"], "{r:?}");
    let j = replaced(&r, "a");
    assert!((length(j) - (9.0 + 9.0 + std::f64::consts::FRAC_PI_2)).abs() < 1e-2, "{}", length(j));
    // Trim=No leaves the lines alone.
    let r = picking(FILLET, &[("Trim", "No")], &[(a.clone(), [5.0, 0.0, 0.0]), (b.clone(), [0.0, 6.0, 0.0])]);
    assert!(r.replace.is_empty() && r.create.len() == 1);
}

#[test]
fn fillet_extends_short_lines_and_radius_zero_makes_a_corner() {
    let a = line("a", [3.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line("b", [0.0, 4.0, 0.0], [0.0, 10.0, 0.0]);
    let r = picking(FILLET, &[], &[(a.clone(), [6.0, 0.0, 0.0]), (b.clone(), [0.0, 6.0, 0.0])]);
    let (p, _) = ends_of(replaced(&r, "a"));
    assert!(dist(p, [1.0, 0.0, 0.0]) < 1e-6, "{p:?}");
    let (p, _) = ends_of(replaced(&r, "b"));
    assert!(dist(p, [0.0, 1.0, 0.0]) < 1e-6, "{p:?}");
    let r = picking(FILLET, &[("Radius", "0")], &[(a.clone(), [6.0, 0.0, 0.0]), (b.clone(), [0.0, 6.0, 0.0])]);
    assert!(r.create.is_empty());
    let (p, _) = ends_of(replaced(&r, "a"));
    assert!(dist(p, [0.0; 3]) < 1e-6, "{p:?}");
    let (p, _) = ends_of(replaced(&r, "b"));
    assert!(dist(p, [0.0; 3]) < 1e-6, "{p:?}");
}

#[test]
fn fillet_a_line_and_a_circle() {
    let a = line("a", [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let c = circle("c", [0.0, 4.0, 0.0], 1.0);
    let r = picking(FILLET, &[("Radius", "2")], &[(a, [5.0, 0.0, 0.0]), (c, [1.0, 4.0, 0.0])]);
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    let center = [5f64.sqrt(), 2.0, 0.0];
    for p in nurbs::samples(&r.create[0], 30) {
        assert!((dist(p, center) - 2.0).abs() < 1e-5, "{p:?}");
    }
    let (p, q) = ends_of(&r.create[0]);
    let on_line = if p[1].abs() < 1e-6 { p } else { q };
    assert!(dist(on_line, [5f64.sqrt(), 0.0, 0.0]) < 1e-5, "{p:?} {q:?}");
}

#[test]
fn chamfer_measures_from_the_corner() {
    let a = line("a", [-10.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    let b = line("b", [0.0, 0.0, 0.0], [0.0, 10.0, 0.0]);
    let r = picking(CHAMFER, &[], &[]);
    assert_eq!(r.prompt, "Select first curve to chamfer");
    assert_eq!(r.options[0].value.as_deref(), Some("1,1"));
    let r = picking(CHAMFER, &[("Distances", "1,3")], &[(a, [-5.0, 0.0, 0.0]), (b, [0.0, 5.0, 0.0])]);
    let (p, q) = ends_of(&r.create[0]);
    assert!(dist(p, [-1.0, 0.0, 0.0]) < 1e-9 && dist(q, [0.0, 3.0, 0.0]) < 1e-9, "{p:?} {q:?}");
    let (_, q) = ends_of(replaced(&r, "a"));
    assert!(dist(q, [-1.0, 0.0, 0.0]) < 1e-9, "{q:?}");
}

#[test]
fn clicking_a_number_option_asks_for_it() {
    let mut e = event(FILLET, Input::Start, vec![]);
    let mut r = step_rec(&e).unwrap();
    e.state = r.state.clone();
    e.input = Input::Option { name: "Radius".into(), value: None };
    r = step_rec(&e).unwrap();
    assert_eq!(r.prompt, "Radius <1>");
    e.state = r.state.clone();
    e.input = num(0.5);
    r = step_rec(&e).unwrap();
    assert_eq!(r.prompt, "Select first curve to fillet");
    assert_eq!(r.options[0].value.as_deref(), Some("0.5"));
    // A number typed while picking sets Radius too (Rhino habit).
    e.state = r.state.clone();
    e.input = num(3.0);
    r = step_rec(&e).unwrap();
    assert_eq!(r.options[0].value.as_deref(), Some("3"));
}

#[test]
fn fillet_corners_rounds_a_square() {
    let sq = vec![
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ];
    let mut pts = sq.clone();
    pts.push(sq[0]);
    let g = om9_addon_geom::control_curve(&pts, 1).unwrap();
    let s = curve("sq", g, true);
    let r = run(FILLET_CORNERS, vec![s], vec![num(1.0)]);
    // Perimeter: 4·(10 − 2) + 2π·1.
    let l = length(&r.replace[0].geo);
    assert!(
        (l - (32.0 + 2.0 * std::f64::consts::PI)).abs() < 1e-2,
        "{l}"
    );
}

/// One Extend step.
fn ext_step(r: &Reply, input: Input, sel: Vec<Selected>, pick: Option<Vec3>) -> Reply {
    let mut e = event(EXTEND, input, sel);
    e.state = r.state.clone();
    e.pick = pick;
    let r = step_rec(&e).unwrap();
    assert!(r.error.is_none(), "{:?} ({})", r.error, r.prompt);
    r
}

#[test]
fn extend_by_length_to_boundaries_and_dynamically() {
    let l = line("l", [0.0; 3], [10.0, 0.0, 0.0]);
    let start = step_rec(&event(EXTEND, Input::Start, vec![])).unwrap();
    assert_eq!(start.prompt, "Select boundary objects or enter extension length. Press Enter for dynamic extend");
    assert_eq!(start.options[0].value.as_deref(), Some("Natural"));
    // A length, then a pick near the start: the start grows.
    let r = ext_step(&start, num(2.0), vec![], None);
    assert_eq!(r.prompt, "Select curve to extend");
    let r = ext_step(&r, Input::Selection, vec![l.clone()], Some([1.0, 0.0, 0.0]));
    let r = ext_step(&r, Input::Enter, vec![l.clone()], None);
    assert!(r.done);
    let (p, q) = ends_of(&r.replace[0].geo);
    assert!(dist(p, [-2.0, 0.0, 0.0]) < 1e-9 && dist(q, [10.0, 0.0, 0.0]) < 1e-9, "{p:?} {q:?}");
    // A boundary circle: the end runs to it.
    let b = circle("b", [15.0, 0.0, 0.0], 2.0);
    let r = ext_step(&start, Input::Selection, vec![b.clone()], Some([13.0, 0.0, 0.0]));
    let r = ext_step(&r, Input::Enter, vec![b.clone()], None);
    assert_eq!(r.prompt, "Select curve to extend");
    let r = ext_step(&r, Input::Selection, vec![b.clone(), l.clone()], Some([9.0, 0.0, 0.0]));
    let r = ext_step(&r, Input::Enter, vec![b.clone(), l.clone()], None);
    let (_, q) = ends_of(&r.replace[0].geo);
    assert!(dist(q, [13.0, 0.0, 0.0]) < 1e-6, "{q:?}");
    // An arc keeps its circle with Type=Natural, up to a boundary line.
    let arc = curve("a", Geo::Arc { center: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], radius: 5.0, angle: 1.0 }, false);
    let wall = line("w", [-10.0, -5.0, 0.0], [10.0, 15.0, 0.0]);
    let r = ext_step(&start, Input::Selection, vec![wall.clone()], Some([0.0, 10.0, 0.0]));
    let r = ext_step(&r, Input::Enter, vec![wall.clone()], None);
    let r = ext_step(&r, Input::Selection, vec![wall.clone(), arc.clone()], Some([5.0 * 0.9f64.cos(), 5.0 * 0.9f64.sin(), 0.0]));
    let r = ext_step(&r, Input::Enter, vec![wall.clone(), arc.clone()], None);
    let g = &r.replace[0].geo;
    assert!(matches!(g, Geo::Arc { radius, .. } if (*radius - 5.0).abs() < 1e-12), "{g:?}");
    let (_, q) = ends_of(g);
    assert!(dist(q, [0.0, 5.0, 0.0]) < 1e-6, "first crossing with y = x + 5: {q:?}");
    // Dynamic: Enter with nothing, pick the curve, then the end point.
    let r = ext_step(&start, Input::Enter, vec![], None);
    let r = ext_step(&r, Input::Selection, vec![l.clone()], Some([9.0, 0.0, 0.0]));
    assert_eq!(r.prompt, "End of extension");
    let r = ext_step(&r, pt([14.0, 1.0, 0.0]), vec![l.clone()], None);
    let r = ext_step(&r, Input::Enter, vec![l.clone()], None);
    let (_, q) = ends_of(&r.replace[0].geo);
    assert!(dist(q, [14.0, 0.0, 0.0]) < 1e-6, "{q:?}");
}

#[test]
fn seam_moves_to_the_pick() {
    let a = circle("a", [0.0; 3], 2.0);
    let r = run(SEAM, vec![a], vec![pt([0.0, 3.0, 0.0])]);
    let p = nurbs::samples(&r.replace[0].geo, 10)[0];
    assert!(dist(p, [0.0, 2.0, 0.0]) < 0.05, "{p:?}");
}

#[test]
fn csec_makes_profiles_through_the_rails() {
    // Four rails around the x axis, a profile at x = 5.
    let rails: Vec<Selected> = (0..4)
        .map(|k| {
            let a = k as f64 * std::f64::consts::FRAC_PI_2;
            let (y, z) = (2.0 * a.cos(), 2.0 * a.sin());
            line(&format!("r{k}"), [0.0, y, z], [10.0, y, z])
        })
        .collect();
    let start = step_rec(&event(CROSS_SECTION, Input::Start, vec![])).unwrap();
    assert_eq!(start.prompt, "Select longitudinal curves in order. Press Enter when done");
    let r = run(CROSS_SECTION, rails.clone(), vec![pt([5.0, -5.0, 0.0]), pt([5.0, 5.0, 0.0]), Input::Enter]);
    assert_eq!(r.create.len(), 1);
    let g = &r.create[0];
    let (p, q) = ends_of(g);
    assert!(dist(p, q) < 1e-9, "closed");
    for p in nurbs::samples(g, 50) {
        assert!((p[0] - 5.0).abs() < 1e-9, "in the plane x = 5: {p:?}");
    }
}

#[test]
fn curves_from_solids() {
    let b = solid("b", cuboid([0.0; 3], [2.0, 2.0, 2.0]));
    // An open box (no top): its border is the 8 mm square rim.
    let mut open = cuboid([0.0; 3], [2.0, 2.0, 2.0]);
    let top: Vec<[u32; 3]> = open
        .triangles
        .iter()
        .copied()
        .filter(|t| t.iter().all(|&i| open.vertices[i as usize][2] > 1.0))
        .collect();
    open.triangles.retain(|t| !top.contains(t));
    let r = run(DUP_BORDER, vec![solid("o", open)], vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    assert!(
        (length(&r.create[0]) - 8.0).abs() < 1e-6
            || matches!(&r.create[0], Geo::Polyline { closed: true, .. })
    );
    let r = run(DUP_EDGE, vec![b.clone()], vec![Input::Enter]);
    let total: f64 = r.create.iter().map(length).sum();
    assert!((total - 24.0).abs() < 1e-6, "12 edges of 2 mm: {total}");
    let ball = solid("s", sphere(3.0, 48));
    let r = run(SILHOUETTE, vec![ball.clone()], vec![Input::Enter]);
    assert!(!r.create.is_empty());
    let r = run(
        SECTION,
        vec![ball.clone()],
        vec![pt([-5.0, 0.0, 0.0]), pt([5.0, 0.0, 0.0]), pt([-5.0, 1.0, 0.0]), pt([5.0, 1.0, 0.0]), Input::Enter],
    );
    assert_eq!(r.create.len(), 2, "two sections, then Enter");
    let c = solid("c", cuboid([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
    let d = solid("d", cuboid([0.0; 3], [2.0, 2.0, 2.0]));
    let r = run(INTERSECTION, vec![c.clone(), d], vec![Input::Enter]);
    let total: f64 = r.create.iter().map(length).sum();
    assert!((total - 6.0).abs() < 1e-2, "{total}");
    // Project a circle down onto the top of the box.
    let top = solid("top", cuboid([-10.0, -10.0, 0.0], [10.0, 10.0, 3.0]));
    let r = run(
        PROJECT,
        vec![circle("k", [0.0; 3], 2.0), top.clone()],
        vec![Input::Enter, Input::Enter],
    );
    for p in nurbs::samples(&r.create[0], 30) {
        assert!((p[2] - 3.0).abs() < 1e-6, "{p:?}");
    }
    let r = run(
        PULLBACK,
        vec![circle("k", [0.0, 0.0, 5.0], 2.0), top],
        vec![Input::Enter, Input::Enter],
    );
    for p in nurbs::samples(&r.create[0], 30) {
        assert!((p[2] - 3.0).abs() < 1e-3, "{p:?}");
    }
}

#[test]
fn curve_boolean_of_two_squares() {
    let sq = |c: Vec3| {
        let p = vec![
            add(c, [-1.0, -1.0, 0.0]),
            add(c, [1.0, -1.0, 0.0]),
            add(c, [1.0, 1.0, 0.0]),
            add(c, [-1.0, 1.0, 0.0]),
            add(c, [-1.0, -1.0, 0.0]),
        ];
        om9_addon_geom::control_curve(&p, 1).unwrap()
    };
    let a = curve("a", sq([0.0; 3]), true);
    let b = curve("b", sq([1.0, 0.0, 0.0]), true);
    let sel = vec![a.clone(), b.clone()];
    let step = |r: &Reply, input: Input| {
        let mut e = event(CURVE_BOOLEAN, input, sel.clone());
        e.state = r.state.clone();
        step_rec(&e).unwrap()
    };
    let r = step_rec(&event(CURVE_BOOLEAN, Input::Start, sel.clone())).unwrap();
    assert_eq!(r.prompt, "Click inside the regions to keep. Press Enter when done");
    assert_eq!(r.options.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), ["DeleteInput", "CombineRegions", "AllRegions"]);
    let perimeter = |g: &Geo| length(g);
    // The left part only: a 1 × 2 rectangle.
    let r1 = step(&r, pt([-0.5, 0.0, 0.0]));
    assert!(r1.error.is_none(), "{:?}", r1.error);
    assert_eq!(r1.preview.len(), 1);
    let done = step(&r1, Input::Enter);
    assert!(done.done && done.delete.is_empty());
    assert!((perimeter(&done.create[0]) - 6.0).abs() < 1e-6, "{}", perimeter(&done.create[0]));
    let (p, q) = ends_of(&done.create[0]);
    assert!(dist(p, q) < 1e-9, "closed");
    // All three regions combined: the union outline, 3 × 2 → perimeter 10.
    let all = step(&step(&r, Input::Option { name: "AllRegions".into(), value: None }), Input::Enter);
    assert_eq!(all.create.len(), 1);
    assert!((perimeter(&all.create[0]) - 10.0).abs() < 1e-6, "{}", perimeter(&all.create[0]));
    // Not combined: three separate outlines; clicking twice drops a region again.
    let sep = step(&r, Input::Option { name: "CombineRegions".into(), value: Some("No".into()) });
    let sep = step(&sep, Input::Option { name: "AllRegions".into(), value: None });
    assert_eq!(step(&sep, Input::Enter).create.len(), 3);
    let twice = step(&step(&r1, pt([-0.6, 0.1, 0.0])), Input::Enter);
    assert!(twice.error.is_some(), "the region was dropped again");
    // DeleteInput=Used with the middle part: both squares are used.
    let used = step(&r, Input::Option { name: "DeleteInput".into(), value: Some("Used".into()) });
    let used = step(&step(&used, pt([0.5, 0.0, 0.0])), Input::Enter);
    assert_eq!(used.delete.len(), 2);
    assert!((perimeter(&used.create[0]) - 6.0).abs() < 1e-6);
}

#[test]
fn curve_boolean_keeps_circles_exact_and_finds_rings() {
    let a = circle("a", [0.0; 3], 3.0);
    let b = circle("b", [0.0; 3], 1.0);
    let line_x = line("l", [-5.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
    let sel = vec![a, b, line_x];
    let start = step_rec(&event(CURVE_BOOLEAN, Input::Start, sel.clone())).unwrap();
    // The upper half of the ring between the circles.
    let mut e = event(CURVE_BOOLEAN, pt([0.0, 2.0, 0.0]), sel.clone());
    e.state = start.state.clone();
    let r = step_rec(&e).unwrap();
    let mut e = event(CURVE_BOOLEAN, Input::Enter, sel.clone());
    e.state = r.state.clone();
    let r = step_rec(&e).unwrap();
    assert_eq!(r.create.len(), 1);
    let g = &r.create[0];
    let want = std::f64::consts::PI * (3.0 + 1.0) + 2.0 * 2.0;
    assert!((length(g) - want).abs() < 5e-3, "{} vs {want}", length(g));
    for p in nurbs::samples(g, 400) {
        let d = norm(p);
        assert!(p[1] > -1e-9 && ((d - 3.0).abs() < 1e-6 || (d - 1.0).abs() < 1e-6 || p[1].abs() < 1e-9), "{p:?}");
    }
    // Two circles alone: the ring has the small circle as a hole.
    let sel = vec![circle("a", [0.0; 3], 3.0), circle("b", [0.0; 3], 1.0)];
    let start = step_rec(&event(CURVE_BOOLEAN, Input::Start, sel.clone())).unwrap();
    let mut e = event(CURVE_BOOLEAN, pt([2.0, 0.0, 0.0]), sel.clone());
    e.state = start.state.clone();
    let r = step_rec(&e).unwrap();
    let mut e = event(CURVE_BOOLEAN, Input::Enter, sel.clone());
    e.state = r.state.clone();
    let r = step_rec(&e).unwrap();
    assert_eq!(r.create.len(), 2, "outer circle and the hole");
}

#[test]
fn clicked_curves_and_two_views() {
    let r = run(
        SKETCH,
        vec![],
        vec![
            pt([0.0; 3]),
            Input::Hover { point: [1.0, 1.0, 0.0] },
            Input::Hover { point: [2.0, 0.0, 0.0] },
            pt([3.0, 1.0, 0.0]),
        ],
    );
    assert_eq!(r.create.len(), 1);
    let floor = solid("f", cuboid([-10.0, -10.0, -1.0], [10.0, 10.0, 2.0]));
    let r = run(
        POLY_ON_SRF,
        vec![floor],
        vec![pt([0.0; 3]), pt([3.0, 0.0, 0.0]), Input::Enter],
    );
    for p in nurbs::samples(&r.create[0], 10) {
        assert!((p[2] - 2.0).abs() < 1e-9);
    }
    // Top view: a line along x; front view: a slope z = x / 2.
    let top = line("t", [0.0; 3], [10.0, 0.0, 0.0]);
    let front = curve(
        "f",
        Geo::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 5.0],
        },
        false,
    );
    let r = run(CRV2VIEW, vec![top, front], vec![]);
    let p = nurbs::samples(&r.create[0], 10);
    assert!(dist(*p.last().unwrap(), [10.0, 0.0, 5.0]) < 1e-6, "{p:?}");
}

#[test]
fn soft_edit_moves_only_near_the_pick() {
    let l = line("l", [0.0; 3], [20.0, 0.0, 0.0]);
    let r = picking(SOFT_EDIT, &[("Distance", "4")], &[(l.clone(), [10.0, 0.0, 0.0])]);
    assert_eq!(r.prompt, "Point on curve to edit");
    assert_eq!(r.options.iter().map(|o| o.name.as_str()).filter(|n| *n != "Undo").collect::<Vec<_>>(), ["Distance", "Copy", "FixEnds"]);
    let mut e = event(SOFT_EDIT, pt([10.0, 0.0, 0.0]), vec![l.clone()]);
    e.state = r.state.clone();
    let r = step_rec(&e).unwrap();
    let mut e = event(SOFT_EDIT, pt([10.0, 3.0, 0.0]), vec![l.clone()]);
    e.state = r.state.clone();
    let r = step_rec(&e).unwrap();
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    let g = &r.replace[0].geo;
    let p = nurbs::samples(g, 400);
    let peak = p.iter().map(|q| q[1]).fold(f64::MIN, f64::max);
    assert!((peak - 3.0).abs() < 0.02, "{peak}");
    assert!(p[0][1].abs() < 1e-6 && p[399][1].abs() < 1e-6, "ends stay");
    // Far from the pick nothing moves (Distance plus the reach of a cubic's control points).
    assert!(p.iter().filter(|q| (q[0] - 10.0).abs() > 7.0).all(|q| q[1].abs() < 1e-6));
}

#[test]
fn every_command_answers_start() {
    for c in CurveTools::SESSION_COMMANDS {
        let r = step_rec(&event(c, Input::Start, vec![])).unwrap();
        assert!(!r.done, "{c}");
        assert!(!r.prompt.is_empty(), "{c}");
    }
}

/// A flat NURBS patch at height `z` over [-10, 10]², with its render mesh.
fn patch(id: &str, z: f64) -> Selected {
    let c = [[-10.0, -10.0, z], [-10.0, 10.0, z], [10.0, -10.0, z], [10.0, 10.0, z]];
    Selected {
        id: id.into(),
        kind: "surface".into(),
        name: id.into(),
        bbox: [[-10.0, -10.0, z], [10.0, 10.0, z]],
        points: c.to_vec(),
        exact: Some(Geo::Surface { degree: [1, 1], size: [2, 2], points: c.to_vec(), weights: vec![], knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]] }),
        mesh: Some(SelectedMesh { vertices: c.to_vec(), triangles: vec![[0, 2, 3], [0, 3, 1]] }),
        ..Default::default()
    }
}

#[test]
fn project_loose_splits_and_deletes() {
    let top = patch("p", 3.0);
    // A circle half outside the patch: two pieces would come back if it left twice; here it
    // leaves once on the right, so one open piece.
    let c = circle("k", [8.0, 0.0, 0.0], 4.0);
    let r = run(PROJECT, vec![c.clone(), top.clone()], vec![Input::Enter, Input::Enter]);
    assert_eq!(r.create.len(), 1);
    let (p, q) = ends_of(&r.create[0]);
    assert!(dist(p, q) > 1.0, "open where it leaves the patch");
    for s in nurbs::samples(&r.create[0], 50) {
        assert!((s[2] - 3.0).abs() < 1e-9 && s[0] <= 10.0 + 1e-6);
    }
    // Loose: the control structure stays.
    let inner = circle("k2", [0.0; 3], 2.0);
    let r = run(PROJECT, vec![inner.clone(), top.clone()], vec![Input::Option { name: "Loose".into(), value: None }, Input::Option { name: "DeleteInput".into(), value: Some("Yes".into()) }, Input::Enter, Input::Enter]);
    assert!(matches!(&r.create[0], Geo::Nurbs { weights, .. } if !weights.is_empty()), "rational poles kept: {:?}", r.create[0]);
    assert_eq!(r.delete, ["k2"]);
    for s in nurbs::samples(&r.create[0], 50) {
        assert!((s[2] - 3.0).abs() < 1e-9 && (norm([s[0], s[1], 0.0]) - 2.0).abs() < 1e-9);
    }
}

#[test]
fn offsets_on_and_normal_to_a_surface() {
    let srf = patch("p", 0.0);
    let l = line("l", [-5.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
    let step = |cmd: &str, st: &Value, input: Input, sel: Vec<Selected>, pick: Option<Vec3>| {
        let mut e = event(cmd, input, sel);
        e.state = st.clone();
        e.pick = pick;
        let r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{cmd}: {:?} ({})", r.error, r.prompt);
        r
    };
    let r = step(OFFSET_ON_SRF, &Value::Null, Input::Start, vec![], None);
    assert_eq!(r.prompt, "Select curve on surface to offset");
    let r = step(OFFSET_ON_SRF, &r.state, Input::Selection, vec![l.clone()], Some([0.0; 3]));
    assert_eq!(r.prompt, "Select base surface");
    let r = step(OFFSET_ON_SRF, &r.state, Input::Selection, vec![l.clone(), srf.clone()], Some([7.0, 7.0, 0.0]));
    assert_eq!(r.prompt, "Side to offset");
    let r = step(OFFSET_ON_SRF, &r.state, Input::Option { name: "Distance".into(), value: Some("2".into()) }, vec![l.clone(), srf.clone()], None);
    let r = step(OFFSET_ON_SRF, &r.state, pt([0.0, 5.0, 0.0]), vec![l.clone(), srf.clone()], None);
    assert!(r.done);
    for s in nurbs::samples(&r.create[0], 30) {
        assert!((s[1] - 2.0).abs() < 1e-5 && s[2].abs() < 1e-9, "{s:?}");
    }
    let r = step(OFFSET_NORMAL, &Value::Null, Input::Start, vec![], None);
    let r = step(OFFSET_NORMAL, &r.state, Input::Selection, vec![l.clone()], Some([0.0; 3]));
    let r = step(OFFSET_NORMAL, &r.state, Input::Selection, vec![l.clone(), srf.clone()], Some([7.0, 7.0, 0.0]));
    assert_eq!(r.prompt, "Offset distance <1>");
    let r = step(OFFSET_NORMAL, &r.state, num(1.5), vec![l.clone(), srf.clone()], None);
    for s in nurbs::samples(&r.create[0], 30) {
        assert!((s[2].abs() - 1.5).abs() < 1e-6, "{s:?}");
    }
}

#[test]
fn extract_isocurves_by_clicking() {
    let srf = patch("p", 0.0);
    let sel = vec![srf.clone()];
    let mut e = event(ISOCURVE, Input::Start, sel.clone());
    let r = step_rec(&e).unwrap();
    assert_eq!(r.prompt, "Select isocurve location on surface. Press Enter when done");
    assert_eq!(r.options.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), ["Direction", "Toggle", "ExtractAll"]);
    let mut state = r.state.clone();
    for input in [pt([2.0, 3.0, 0.0]), Input::Option { name: "Direction".into(), value: Some("Both".into()) }, pt([-4.0, 1.0, 0.0]), Input::Enter] {
        e.state = state.clone();
        e.input = input;
        let r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{:?}", r.error);
        state = r.state.clone();
        if r.done {
            assert_eq!(r.create.len(), 3);
            return;
        }
    }
    panic!("did not finish");
}

fn point_obj(id: &str, p: Vec3) -> Selected {
    Selected { id: id.into(), kind: "point".into(), bbox: [p, p], points: vec![p], exact: Some(Geo::Point { point: p }), ..Default::default() }
}

#[test]
fn points_marks_continue_and_sketch_prompts() {
    let pts: Vec<Selected> = [[0.0, 0.0, 0.0], [2.0, 2.0, 0.0], [4.0, 0.0, 0.0], [6.0, 2.0, 0.0]]
        .iter()
        .enumerate()
        .map(|(i, p)| point_obj(&format!("p{i}"), *p))
        .collect();
    let r = run(THROUGH_POINTS, pts, vec![Input::Enter]);
    let Geo::Nurbs { points, degree, .. } = &r.create[0] else { panic!("{:?}", r.create) };
    assert_eq!((*degree, points.len()), (3, 4), "the points are the control points");
    let l = line("l", [0.0; 3], [10.0, 0.0, 0.0]);
    let r = run(MARK_END, vec![l.clone()], vec![Input::Enter]);
    assert_eq!(r.create, vec![Geo::Point { point: [10.0, 0.0, 0.0] }]);
    // Continue from the start when the start was picked.
    let mut e = event(CONTINUE_CURVE, Input::Selection, vec![l.clone()]);
    e.pick = Some([1.0, 0.0, 0.0]);
    let r = step_rec(&e).unwrap();
    assert_eq!(r.prompt, "Next point. Press Enter when done");
    assert_eq!(r.rubber_from, Some([0.0; 3]));
    e.state = r.state.clone();
    e.input = pt([-5.0, 3.0, 0.0]);
    let r = step_rec(&e).unwrap();
    e.state = r.state.clone();
    e.input = Input::Enter;
    let r = step_rec(&e).unwrap();
    let (_, q) = ends_of(&r.replace[0].geo);
    assert!(dist(q, [-5.0, 3.0, 0.0]) < 1e-9, "{q:?}");
    let r = step_rec(&event(SKETCH, Input::Start, vec![])).unwrap();
    assert_eq!(r.prompt, "Start of sketch");
    assert_eq!(r.options.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), ["Closed", "Planar", "OnSurface", "OnMesh"]);
    let mut e = event(SKETCH, Input::Option { name: "OnSurface".into(), value: None }, vec![]);
    e.state = r.state.clone();
    assert_eq!(step_rec(&e).unwrap().prompt, "Select surface to sketch on");
}

#[test]
fn project_lands_on_every_layer() {
    let ball = solid("s", sphere(3.0, 48));
    let r = run(PROJECT, vec![circle("k", [0.0, 0.0, 10.0], 1.0), ball], vec![Input::Enter, Input::Enter]);
    assert_eq!(r.create.len(), 2, "top and bottom of the ball");
    let z: Vec<f64> = r.create.iter().map(|g| nurbs::samples(g, 10)[0][2]).collect();
    let want = 8f64.sqrt();
    assert!((z[0] - want).abs() < 0.05 && (z[1] + want).abs() < 0.05, "{z:?}");
}

#[test]
fn sketch_follows_the_pointer() {
    let mut e = event(SKETCH, Input::Start, vec![]);
    let mut r = step_rec(&e).unwrap();
    let path = |x: f64| [x, (x * 0.5).sin(), 0.0];
    let mut steps = vec![pt(path(0.0))];
    steps.extend((1..200).map(|i| Input::Hover { point: path(i as f64 * 0.05) }));
    steps.push(pt(path(10.0)));
    for input in steps {
        e.state = r.state.clone();
        e.input = input;
        r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{:?}", r.error);
        if !r.done {
            assert!(r.track, "the host keeps hover states");
        }
    }
    assert!(r.done, "{}", r.prompt);
    for p in nurbs::samples(&r.create[0], 100) {
        assert!((p[1] - (p[0] * 0.5).sin()).abs() < 0.02, "{p:?}");
    }
}

#[test]
fn project_keeps_corners_and_only_the_part_on_the_surface() {
    let top = patch("p", 3.0);
    // A square half over the patch's right edge (x = 10): only the part over the patch comes
    // back, open, ending on the edge, with its corners sharp.
    let sq = Selected {
        id: "q".into(),
        kind: "curve".into(),
        closed: true,
        points: vec![[4.0, -4.0, 0.0], [16.0, -4.0, 0.0], [16.0, 4.0, 0.0], [4.0, 4.0, 0.0]],
        exact: Some(Geo::Polyline {
            points: vec![[4.0, -4.0, 0.0], [16.0, -4.0, 0.0], [16.0, 4.0, 0.0], [4.0, 4.0, 0.0]],
            closed: true,
        }),
        bbox: [[4.0, -4.0, 0.0], [16.0, 4.0, 0.0]],
        ..Default::default()
    };
    let r = run(PROJECT, vec![sq, top.clone()], vec![Input::Enter, Input::Enter]);
    assert_eq!(r.create.len(), 1, "one piece through the seam: {:?}", r.create);
    let Geo::Polyline { points, closed } = &r.create[0] else { panic!("{:?}", r.create[0]) };
    assert!(!closed, "open where it leaves the surface");
    let ends = [points[0], points[points.len() - 1]];
    for e in ends {
        assert!((e[0] - 10.0).abs() < 1e-6 && (e[2] - 3.0).abs() < 1e-9, "ends on the edge: {e:?}");
    }
    for corner in [[4.0, -4.0, 3.0], [4.0, 4.0, 3.0]] {
        assert!(points.iter().any(|p| dist(*p, corner) < 1e-9), "corner {corner:?} in {points:?}");
    }
    assert_eq!(points.len(), 4, "edge, corner, corner, edge: {points:?}");
    // A pentagon all on the surface comes back closed with its five corners.
    let pent: Vec<Vec3> = (0..5)
        .map(|i| {
            let a = i as f64 * std::f64::consts::TAU / 5.0;
            [5.0 * a.cos(), 5.0 * a.sin(), 0.0]
        })
        .collect();
    let pg = Selected {
        id: "g".into(),
        kind: "curve".into(),
        closed: true,
        points: pent.clone(),
        exact: Some(Geo::Polyline { points: pent.clone(), closed: true }),
        bbox: [[-5.0, -5.0, 0.0], [5.0, 5.0, 0.0]],
        ..Default::default()
    };
    let r = run(PROJECT, vec![pg, top], vec![Input::Enter, Input::Enter]);
    let Geo::Polyline { points, closed } = &r.create[0] else { panic!("{:?}", r.create[0]) };
    assert!(closed);
    assert_eq!(points.len(), 5, "{points:?}");
    for p in &pent {
        assert!(points.iter().any(|q| dist(*q, [p[0], p[1], 3.0]) < 1e-9));
    }
}

#[test]
fn through_points_interpolates_extend_to_point_and_rebuild_to_master() {
    // CurveThroughPt CurveType=Interpolated: the curve passes through the points.
    let pts: Vec<Selected> = [[0.0, 0.0, 0.0], [2.0, 2.0, 0.0], [4.0, 0.0, 0.0], [6.0, 2.0, 0.0]]
        .iter()
        .enumerate()
        .map(|(i, p)| point_obj(&format!("p{i}"), *p))
        .collect();
    let r = run(THROUGH_POINTS, pts, vec![Input::Option { name: "CurveType".into(), value: Some("Interpolated".into()) }, Input::Enter]);
    let c = curve("c", r.create[0].clone(), false);
    let cr = om9_addon_geom::pick::CurveRef::of(&c).unwrap();
    for p in [[2.0, 2.0, 0.0], [4.0, 0.0, 0.0]] {
        assert!(dist(cr.closest(p).1, p) < 1e-6, "passes through {p:?}");
    }
    // Extend Type=Arc ToPoint: a tangent arc ending exactly at the pick.
    let l = line("l", [0.0; 3], [10.0, 0.0, 0.0]);
    let mut e = event(EXTEND, Input::Start, vec![]);
    let mut state = Value::Null;
    let mut finished = false;
    for input in [Input::Option { name: "Type".into(), value: Some("Arc".into()) }, Input::Enter, Input::Selection, Input::Option { name: "ToPoint".into(), value: None }, pt([14.0, 4.0, 0.0]), Input::Enter] {
        e.state = state.clone();
        e.input = input.clone();
        if matches!(input, Input::Selection) {
            e.pick = Some([9.0, 0.0, 0.0]);
            e.selection = vec![l.clone()];
        }
        let r = step_rec(&e).unwrap();
        assert!(r.error.is_none(), "{:?} {}", r.error, r.prompt);
        state = r.state.clone();
        if r.done {
            let g = &r.replace[0].geo;
            let (_, q) = ends_of(g);
            assert!(dist(q, [14.0, 4.0, 0.0]) < 1e-6, "ends at the pick: {q:?}");
            let s = nurbs::samples(g, 400);
            // Tangent at the joint: no kink, the arc leaves along +x.
            assert!(s.iter().all(|p| p[1] >= -1e-9), "bends toward the point");
            finished = true;
        }
    }
    assert!(finished, "Extend finished");
    // Rebuild SelectMasterCurve: the curves take the master's point count and degree.
    let master = curve("m", om9_addon_geom::control_curve(&[[0.0, 5.0, 0.0], [1.0, 6.0, 0.0], [2.0, 5.0, 0.0], [3.0, 6.0, 0.0], [4.0, 5.0, 0.0]], 2).unwrap(), false);
    let mut e = event(REBUILD, Input::Start, vec![l.clone()]);
    let r = step_rec(&e).unwrap();
    assert!(r.options.iter().any(|o| o.name == "SelectMasterCurve"));
    e.state = r.state.clone();
    e.input = Input::Option { name: "SelectMasterCurve".into(), value: None };
    let r = step_rec(&e).unwrap();
    assert_eq!(r.prompt, "Select master curve");
    e.state = r.state.clone();
    e.input = Input::Selection;
    e.selection = vec![l.clone(), master];
    let r = step_rec(&e).unwrap();
    assert!(r.messages.iter().any(|m| m.contains("5 points, degree 2")), "{:?}", r.messages);
    e.state = r.state.clone();
    e.input = Input::Enter;
    let r = step_rec(&e).unwrap();
    assert!(r.done, "{}", r.prompt);
    assert_eq!(r.replace.len(), 1, "the master is not rebuilt");
    let Geo::Nurbs { degree, points, .. } = &r.replace[0].geo else { panic!() };
    assert_eq!((*degree, points.len()), (2, 5));
}

#[test]
fn rhino5_sketch_closed_and_match_average() {
    // Sketch Closed=Yes: the stroke comes back closed.
    let mut e = event(SKETCH, Input::Start, vec![]);
    let mut r = step_rec(&e).unwrap();
    let mut steps = vec![Input::Option { name: "Closed".into(), value: Some("Yes".into()) }, pt([5.0, 0.0, 0.0])];
    steps.extend((1..100).map(|i| {
        let a = i as f64 * 0.06;
        Input::Hover { point: [5.0 * a.cos(), 5.0 * a.sin(), 0.0] }
    }));
    steps.push(pt([5.0 * 5.94f64.cos(), 5.0 * 5.94f64.sin(), 0.0]));
    for input in steps {
        e.state = r.state.clone();
        e.input = input;
        r = step_rec(&e).unwrap();
    }
    assert!(r.done, "{}", r.prompt);
    let (p, q) = ends_of(&r.create[0]);
    assert!(dist(p, q) < 1e-6, "closed: {p:?} {q:?}");
    // Match AverageCurves: both curves meet halfway between the picked ends.
    let a = line("a", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line("b", [12.0, 2.0, 0.0], [20.0, 2.0, 0.0]);
    let mut e = event(MATCH, Input::Selection, vec![a.clone()]);
    e.pick = Some([9.0, 0.0, 0.0]);
    let r = step_rec(&e).unwrap();
    e.state = r.state.clone();
    e.selection = vec![a.clone(), b.clone()];
    e.pick = Some([13.0, 2.0, 0.0]);
    let r = step_rec(&e).unwrap();
    e.state = r.state.clone();
    e.input = Input::Option { name: "AverageCurves".into(), value: Some("Yes".into()) };
    let r = step_rec(&e).unwrap();
    e.state = r.state.clone();
    e.input = Input::Enter;
    let r = step_rec(&e).unwrap();
    assert!(r.done, "{} {:?}", r.prompt, r.error);
    assert_eq!(r.replace.len(), 2, "both curves change");
    let (_, ea) = ends_of(&r.replace[0].geo);
    let (sb, _) = ends_of(&r.replace[1].geo);
    assert!(dist(ea, [11.0, 1.0, 0.0]) < 1e-6 && dist(sb, [11.0, 1.0, 0.0]) < 1e-6, "{ea:?} {sb:?}");
}

/// Shape audit of the commands the other tests do not finish: each runs as a user would
/// and must make the documented shape.
#[test]
fn audit_remaining_commands() {
    let l = line("l", [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    // Mark Curve Start: a point at the start.
    let r = run(MARK_START, vec![l.clone()], vec![Input::Enter]);
    assert_eq!(r.create, vec![Geo::Point { point: [0.0; 3] }]);
    // Intersect Two Sets: a crossing line in the second set gives one point.
    let m = line("m", [5.0, -5.0, 0.0], [5.0, 5.0, 0.0]);
    let mut e = event(INTERSECT_TWO, Input::Start, vec![l.clone()]);
    let mut r = step_rec(&e).unwrap();
    for (input, sel) in [(Input::Enter, vec![l.clone()]), (Input::Selection, vec![l.clone(), m.clone()]), (Input::Enter, vec![l.clone(), m.clone()])] {
        e.state = r.state.clone();
        e.input = input;
        e.selection = sel;
        r = step_rec(&e).unwrap();
    }
    assert!(r.done, "{} {:?}", r.prompt, r.error);
    let hit = r.create.iter().find_map(|g| match g { Geo::Point { point } => Some(*point), _ => None });
    assert!(hit.is_some_and(|p| dist(p, [5.0, 0.0, 0.0]) < 1e-6), "{:?}", r.create);
    // Extract Wireframe of a patch: its four border lines at least.
    let srf = patch("p", 0.0);
    let r = run(WIREFRAME, vec![srf.clone()], vec![Input::Enter]);
    assert!(!r.create.is_empty());
    // Interpolate on Surface: the curve lies on the patch.
    let mut e = event(INTERP_ON_SRF, Input::Start, vec![srf.clone()]);
    let mut r = step_rec(&e).unwrap();
    for input in [pt([-5.0, 0.0, 3.0]), pt([0.0, 4.0, 3.0]), pt([5.0, 0.0, 3.0]), Input::Enter] {
        e.state = r.state.clone();
        e.input = input;
        r = step_rec(&e).unwrap();
    }
    assert!(r.done, "{}", r.prompt);
    for p in nurbs::samples(&r.create[0], 50) {
        assert!(p[2].abs() < 1e-6, "on the surface: {p:?}");
    }
    // Continue Interpolated Curve from the end of a line.
    let mut e = event(CONTINUE_INTERP, Input::Selection, vec![l.clone()]);
    e.pick = Some([9.0, 0.0, 0.0]);
    let mut r = step_rec(&e).unwrap();
    for input in [pt([15.0, 5.0, 0.0]), pt([20.0, 0.0, 0.0]), Input::Enter] {
        e.state = r.state.clone();
        e.input = input;
        r = step_rec(&e).unwrap();
    }
    assert!(r.done, "{}", r.prompt);
    let g = r.replace.first().map(|x| x.geo.clone()).or_else(|| r.create.first().cloned()).unwrap();
    let (p, q) = ends_of(&g);
    assert!(dist(p, [0.0; 3]) < 1e-6 && dist(q, [20.0, 0.0, 0.0]) < 1e-6, "{p:?} {q:?}");
}

#[test]
fn intersect_makes_points_for_curves_like_rhino5() {
    // A vertical line through a patch: one point on the surface; two crossing lines: one point.
    let srf = patch("p", 3.0);
    let v = line("v", [1.0, 2.0, -5.0], [1.0, 2.0, 5.0]);
    let r = run(INTERSECTION, vec![srf, v], vec![Input::Enter]);
    assert_eq!(r.create.len(), 1, "{:?}", r.create);
    let Geo::Point { point } = r.create[0] else { panic!("{:?}", r.create) };
    assert!(dist(point, [1.0, 2.0, 3.0]) < 1e-6, "{point:?}");
    let a = line("a", [0.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let b = circle("b", [5.0, 5.0, 0.0], 2.0);
    let r = run(INTERSECTION, vec![a, b], vec![Input::Enter]);
    assert_eq!(r.create.len(), 2, "a line through a circle crosses it twice");
}
