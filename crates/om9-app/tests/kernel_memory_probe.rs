// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in, one-process-per-case release measurements; not a memory limit.
#![cfg(all(feature = "occt-brep", windows))]

use om9_core::{brep::Brep, budget::Limits, geometry::Transform};
use om9_kernel::{brep, creation::{self, Primitive, Profile}, editing::{self, BooleanOp, BooleanOutcome}, transform_ops};

enum Fixture {
    Boolean(Brep, Brep, BooleanOp),
    Fillet(Brep, Vec<usize>),
    Loft(Vec<Profile>),
}

#[test]
#[ignore = "isolated release kernel memory diagnostic"]
fn kernel_memory_probe() {
    let case = std::env::var("OM9_KERNEL_MEMORY_CASE").expect("choose boolean, boolean_drilled, fillet, fillet_all, loft, loft32 or loft32_wave");
    let limits = Limits::default();
    let deflection = 0.01;
    let fixture = match case.as_str() {
        "boolean" => {
            let a = brep::box_brep([10.; 3], deflection, &limits).unwrap();
            let b = transform_ops::transform_brep(&a, &Transform::translation([5., 0., 0.]), deflection, &limits).unwrap();
            Fixture::Boolean(a, b, BooleanOp::Union)
        }
        "boolean_drilled" => {
            let profile = Profile::new(vec![[-10., -10., 0.], [10., -10., 0.], [10., 10., 0.], [-10., 10., 0.]]).unwrap();
            let mut body = creation::extrude(&profile, [0., 0., 10.], true, deflection, &limits).unwrap();
            let cutter = creation::primitive(Primitive::Cylinder { radius_mm: 0.7, height_mm: 10. }, deflection, &limits).unwrap();
            for x in [-6., -2., 2., 6.] { for y in [-6., -2., 2., 6.] {
                let tool = transform_ops::transform_brep(&cutter, &Transform::translation([x, y, 0.]), deflection, &limits).unwrap();
                body = editing::boolean(&body, &tool, BooleanOp::Difference, deflection, &limits).unwrap().body().unwrap().clone();
            }}
            Fixture::Boolean(body, cutter, BooleanOp::Difference)
        }
        "fillet" => Fixture::Fillet(brep::box_brep([10.; 3], deflection, &limits).unwrap(), vec![0]),
        "fillet_all" => {
            let body = brep::box_brep([10.; 3], deflection, &limits).unwrap();
            assert_eq!(body.display.edges.len(), 12);
            Fixture::Fillet(body, (0..12).collect())
        }
        "loft" => Fixture::Loft([0., 5., 10.].into_iter().map(|z| Profile::new(vec![
            [0., 0., z], [10., 0., z], [10., 10., z], [0., 10., z],
        ]).unwrap()).collect()),
        "loft32" => Fixture::Loft((0..32).map(|i| {
            let z = 10. * (i as f64) / 31.;
            Profile::new(vec![[0., 0., z], [10., 0., z], [10., 10., z], [0., 10., z]]).unwrap()
        }).collect()),
        "loft32_wave" => Fixture::Loft((0..32).map(|i| {
            let z = 10. * (i as f64) / 31.;
            let x = 2. * (std::f64::consts::TAU * z / 10.).sin();
            Profile::new(vec![[x, 0., z], [x + 10., 0., z], [x + 10., 10., z], [x, 10., z]]).unwrap()
        }).collect()),
        _ => panic!("unknown kernel fixture"),
    };
    let before = om9_native_owner::process_memory().expect("Windows process counters");
    let start = std::time::Instant::now();
    let result = match &fixture {
        Fixture::Boolean(a, b, operation) => match editing::boolean(a, b, *operation, deflection, &limits).unwrap() {
            BooleanOutcome::Body(body) => body,
            BooleanOutcome::Empty => panic!("overlapping boxes must produce a body"),
        },
        Fixture::Fillet(body, edges) => editing::fillet(body, edges, 1., deflection, &limits).unwrap(),
        Fixture::Loft(sections) => creation::loft(sections, true, false, deflection, &limits).unwrap(),
    };
    let elapsed = start.elapsed();
    let after = om9_native_owner::process_memory().expect("Windows process counters");
    let measures = creation::measure(&result, &limits).unwrap();
    let expected = match case.as_str() {
        "boolean" => 1500.,
        "boolean_drilled" => 4000. - 17. * std::f64::consts::PI * 0.7f64.powi(2) * 10.,
        "fillet" => 1000. - (1. - std::f64::consts::PI / 4.) * 10.,
        "fillet_all" => 8f64.powi(3) + 6. * 8f64.powi(2) + 3. * std::f64::consts::PI * 8. + 4. * std::f64::consts::PI / 3.,
        "loft" | "loft32" | "loft32_wave" => 1000.,
        _ => unreachable!(),
    };
    assert!((measures.volume_mm3 - expected).abs() < 1e-5, "volume {}", measures.volume_mm3);
    assert!(measures.closed);
    assert_eq!(measures.naked_edges, 0);
    if case == "loft32_wave" {
        let min_x = result.display.points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let max_x = result.display.points.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        assert!(min_x < -1.9 && max_x > 11.9, "loft should follow the curved centreline");
    }
    println!("KERNEL_MEMORY case={case} deflection_mm={deflection} volume_mm3={} faces={} elapsed_ms={:.3} before_working={} before_peak_working={} before_private={} before_peak_pagefile={} after_working={} after_peak_working={} after_private={} after_peak_pagefile={}",
        measures.volume_mm3, measures.faces, elapsed.as_secs_f64() * 1000., before.working_set_bytes,
        before.peak_working_set_bytes, before.private_bytes, before.peak_pagefile_bytes,
        after.working_set_bytes, after.peak_working_set_bytes, after.private_bytes, after.peak_pagefile_bytes);
}
