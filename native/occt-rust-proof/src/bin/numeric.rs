//! M00-L2-02 / L3-03 gate probe — geometry numeric invariants.
//!
//! Covers the four L4s of `OM9-M00-L3-02-03`:
//!
//! * `L4-02-03-01` unit-pinned fixtures: box, sphere and cylinder measured against analytic volume and
//!   area in millimetres;
//! * `L4-02-03-02` boolean/fillet/offset plus degenerate errors: the cut volume against `V − πr²h`, a
//!   fillet against `V − (1 − π/4)r²L`, an inward offset against the shrunken box, and three degenerate
//!   cases that must be reported instead of crashing;
//! * `L4-02-03-03` tessellation against B-Rep exactness: the largest mesh-node deviation from the exact
//!   sphere at two deflection values, with the triangle count and a watertightness check;
//! * `L4-02-03-04` the viability decision, summarised from the checks above.
//!
//! Feasibility only: the harness lives under `native/`, M00 owns no product code.
use om9_occt_rust_proof::{
    box_measures, cylinder_measures, degenerate, fillet_volume, offset_volume, sphere_measures,
    sphere_mesh,
};
use std::f64::consts::PI;

fn check(
    checks: &mut Vec<String>,
    failures: &mut usize,
    name: &str,
    expected: f64,
    measured: f64,
    tolerance: f64,
) {
    let error = (measured - expected).abs();
    let pass = error <= tolerance;
    if !pass {
        *failures += 1;
    }
    checks.push(format!(
        "{{\"name\": \"{name}\", \"expected\": {expected:.12}, \"measured\": {measured:.12}, \"error\": {error:.3e}, \"tolerance\": {tolerance:.1e}, \"pass\": {pass}}}"
    ));
}

fn main() {
    let mut checks: Vec<String> = Vec::new();
    let mut failures = 0usize;

    // --- L4-02-03-01: unit-pinned fixtures ---------------------------------
    match box_measures(10.0, 10.0, 4.0) {
        Ok((volume, area)) => {
            check(&mut checks, &mut failures, "box_volume_mm3", 400.0, volume, 1e-9);
            check(&mut checks, &mut failures, "box_area_mm2", 360.0, area, 1e-9);
        }
        Err(error) => {
            failures += 1;
            checks.push(format!("{{\"name\": \"box_measures\", \"error\": \"{error}\"}}"));
        }
    }
    match sphere_measures(5.0) {
        Ok((volume, area)) => {
            check(
                &mut checks,
                &mut failures,
                "sphere_volume_mm3",
                4.0 / 3.0 * PI * 125.0,
                volume,
                1e-7,
            );
            check(
                &mut checks,
                &mut failures,
                "sphere_area_mm2",
                4.0 * PI * 25.0,
                area,
                1e-7,
            );
        }
        Err(error) => {
            failures += 1;
            checks.push(format!("{{\"name\": \"sphere_measures\", \"error\": \"{error}\"}}"));
        }
    }
    match cylinder_measures(2.0, 4.0) {
        Ok((volume, area)) => {
            check(
                &mut checks,
                &mut failures,
                "cylinder_volume_mm3",
                PI * 4.0 * 4.0,
                volume,
                1e-9,
            );
            check(
                &mut checks,
                &mut failures,
                "cylinder_area_mm2",
                24.0 * PI,
                area,
                1e-7,
            );
        }
        Err(error) => {
            failures += 1;
            checks.push(format!("{{\"name\": \"cylinder_measures\", \"error\": \"{error}\"}}"));
        }
    }

    // --- L4-02-03-02: boolean, fillet, offset, degenerate ------------------
    match fillet_volume(10.0, 10.0, 4.0, 1.0) {
        Ok(volume) => {
            let expected = 400.0 - (1.0 - PI / 4.0) * 1.0 * 1.0 * 4.0;
            check(&mut checks, &mut failures, "fillet_volume_mm3", expected, volume, 1e-6);
        }
        Err(error) => {
            failures += 1;
            checks.push(format!("{{\"name\": \"fillet_volume\", \"error\": \"{error}\"}}"));
        }
    }
    match offset_volume(10.0, 10.0, 4.0, -1.0) {
        Ok(volume) => check(
            &mut checks,
            &mut failures,
            "offset_volume_mm3",
            8.0 * 8.0 * 2.0,
            volume,
            1e-6,
        ),
        Err(error) => {
            failures += 1;
            checks.push(format!("{{\"name\": \"offset_volume\", \"error\": \"{error}\"}}"));
        }
    }

    let mut degenerate_reports = Vec::new();
    for (kind, name) in [
        (0, "cut_with_touching_box"),
        (1, "offset_past_the_wall"),
        (2, "box_at_kernel_resolution"),
    ] {
        match degenerate(kind) {
            Ok(report) => degenerate_reports.push(format!(
                "{{\"case\": \"{name}\", \"valid\": {}, \"volume\": {:.12}}}",
                report.valid, report.volume
            )),
            Err(error) => degenerate_reports.push(format!(
                "{{\"case\": \"{name}\", \"reported_error\": \"{}\"}}",
                error.to_string().replace('"', "\\\"")
            )),
        }
    }

    // --- L4-02-03-03: tessellation versus the exact sphere -----------------
    let mut mesh_reports = Vec::new();
    for tolerance in [0.01, 0.001] {
        match sphere_mesh(5.0, tolerance) {
            Ok(report) => {
                if report.max_deviation > tolerance {
                    failures += 1;
                }
                mesh_reports.push(format!(
                    "{{\"tolerance\": {tolerance}, \"max_node_deviation\": {:.3e}, \"triangles\": {}, \"boundary_edges\": {}, \"non_manifold_edges\": {}, \"closed\": {}}}",
                    report.max_deviation,
                    report.triangles,
                    report.boundary_edges,
                    report.non_manifold_edges,
                    report.closed()
                ));
            }
            Err(error) => {
                failures += 1;
                mesh_reports.push(format!(
                    "{{\"tolerance\": {tolerance}, \"error\": \"{}\"}}",
                    error.to_string().replace('"', "\\\"")
                ));
            }
        }
    }

    // --- L4-02-03-04: viability decision -----------------------------------
    let decision = if failures == 0 { "VIABLE" } else { "NOT_VIABLE" };
    println!(
        "{{\"occt_version\": \"{}\", \"checks\": [{}], \"degenerate\": [{}], \"mesh\": [{}], \"failures\": {}, \"decision\": \"{}\"}}",
        om9_occt_rust_proof::version(),
        checks.join(", "),
        degenerate_reports.join(", "),
        mesh_reports.join(", "),
        failures,
        decision
    );
}
