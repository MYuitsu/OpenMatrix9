// SPDX-License-Identifier: LGPL-2.1-or-later
//! Benchmark (ignored): object snap time per pointer move in a large file, NURBS vs lines.
use om9_core::{budget::Limits, geometry::{Curve, Geometry}, model::{Document, Entity}, transaction::{TransactionCoordinator, World}};
use om9_scene::{camera::{Camera, Viewport}, osnap::{SnapSettings, candidates}, scene::Scene};

fn scene(n: usize, nurbs: bool) -> Scene {
    let mut d = Document::new("bench");
    for i in 0..n {
        let y = i as f64 * 0.05;
        let g = if nurbs {
            Curve::Nurbs(om9_core::geometry::NurbsCurve { degree: 3, poles: vec![[0., y, 0.], [1., y + 0.02, 0.], [2., y - 0.02, 0.], [3., y, 0.]],
                weights: vec![1.; 4], knots: vec![0., 0., 0., 0., 1., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None })
        } else {
            Curve::Line { start: [0., y, 0.], end: [3., y, 0.], domain: [0., 3.] }
        };
        let e = Entity::new("c", Geometry::Curve(g), None);
        d.entities.insert(e.id, e);
    }
    let id = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &Limits::default()).unwrap(), Limits::default()).unwrap();
    Scene::capture_in_world(&c, id, om9_core::budget::Usage::default()).unwrap()
}

#[test]
#[ignore]
fn bench_snap_per_pointer_move() {
    let cam = Camera::orthographic([1.5, 25., 100.], [1.5, 25., 0.], [0., 1., 0.], 60.).unwrap();
    let vp = Viewport { origin: [0., 0.], size: [1200., 800.] };
    for (label, nurbs) in [("20000 lines", false), ("4000 nurbs", true)] {
        let s = scene(if nurbs { 4_000 } else { 20_000 }, nurbs);
        let t0 = std::time::Instant::now();
        let mut found = 0;
        for i in 0..50 {
            found += candidates(&s, &cam, vp, [600. + i as f64, 400.], SnapSettings::default(), None).unwrap().len();
        }
        eprintln!("SNAP {label}: {:.3} ms per move ({found} candidates)", t0.elapsed().as_secs_f64() * 1e3 / 50.);
    }
}
