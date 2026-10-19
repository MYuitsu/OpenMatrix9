//! AN INNER THREAD TAKES THE ISO GROOVE: M10 in a hole drilled 8.5, with no fit clearance and no run-out, takes the room
//! of the bolt's tooth by ISO 68-1 - 3/4 of the pitch wide at the basic minor diameter narrowing to P/8 at the major -
//! and in proportion to its length.
//!
//! Reported behaviour: 136.4 mm^3 on a length of 8 (1.93 times the standard's 70.8) and 54.7 on 4.
use qymcad_core::model::Project;
use qymcad_core::thread::{ThreadSpec, ThreadStandard};
use std::f64::consts::PI;

/// The groove of an inner thread of major diameter `d`, pitch `p` in a hole of `hole` over `length` (ISO 68-1), by
/// Pappus radius by radius.
fn iso_inner(d: f64, p: f64, hole: f64, length: f64) -> f64 {
    let minor = d - 1.082_532 * p;
    let width = |r: f64| p * (0.75 - 0.625 * (r - minor / 2.0) / ((d - minor) / 2.0));
    let (from, to, n) = (hole / 2.0, d / 2.0, 4000);
    let dr = (to - from) / n as f64;
    let per_pitch: f64 = (0..n).map(|i| from + (i as f64 + 0.5) * dr).map(|r| width(r).max(0.0) * 2.0 * PI * r * dr).sum();
    per_pitch * length / p
}

/// What M10 inside a through hole of 8.5 in a sleeve of 30 x 20 takes over `length`.
fn taken(length: f64) -> (f64, Vec<String>) {
    let mut p = Project::default();
    p.new_document();
    let outer = p.add_cylinder(15.0, 20.0);
    let hole = p.add_cylinder(4.25, 30.0);
    let sleeve = p.add_body_boolean(outer, hole, 0);
    let (_, shapes) = qymcad_testkit::regenerate(&mut p);
    let before = shapes.get(&sleeve).map(|s| s.volume()).expect("the sleeve");
    let rim = p.regen_edges.get(&sleeve).and_then(|es| es.iter().find(|e| (e.radius - 4.25).abs() < 0.05).map(|e| e.id)).expect("the rim of the hole");
    let spec = ThreadSpec { standard: ThreadStandard::MetricIso, nominal_d: 10.0, pitch: 1.5, internal: true, fit: 0.0, ..Default::default() };
    let t = p.add_thread(sleeve, rim, spec, length, 0.0, 0.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    let after = shapes.get(&t).map(|s| s.volume()).unwrap_or(before);
    (before - after, report.errors.iter().map(|(_, e)| format!("{e:?}")).collect())
}

#[test]
fn m10_inside_takes_the_iso_groove_in_proportion_to_its_length() {
    for length in [8.0, 4.0] {
        let (got, errors) = taken(length);
        let want = iso_inner(10.0, 1.5, 8.5, length);
        assert!(errors.is_empty(), "M10 inside over {length} was refused: {errors:?}");
        assert!((got - want).abs() < want * 0.02, "M10 inside over {length} took {got:.1} mm^3, the ISO groove is {want:.1}");
    }
}

/// A THREAD OF ANOTHER SIZE THAN ITS FACE is refused in words: M10 on a shaft of 20. Reported behaviour: it stood green,
/// 519 mm^3 cut off the cylinder in 76 faces.
#[test]
fn m10_on_a_shaft_of_20_is_refused_in_words() {
    let mut p = Project::default();
    p.new_document();
    let shaft = p.add_cylinder(10.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let rim = p.regen_edges.get(&shaft).and_then(|es| es.iter().find(|e| (e.radius - 10.0).abs() < 0.05).map(|e| e.id)).expect("the rim of the shaft");
    let spec = ThreadSpec { standard: ThreadStandard::MetricIso, nominal_d: 10.0, pitch: 1.5, ..Default::default() };
    let t = p.add_thread(shaft, rim, spec, 10.0, 0.0, 0.0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.iter().any(|(id, e)| *id == t && matches!(e, qymcad_core::errors::CoreError::ThreadNotItsSize { .. })), "M10 on a shaft of 20 was not refused in words: {:?}", report.errors);
}

/// IN A BLIND HOLE DRILLED FROM THE TOP, as the hole tool drills it: M10 in 8.5 x 8 over the whole 8 and over 4. Reported
/// behaviour: the whole 8 took nothing at all and stood green - the groove was set in the void short of the wall.
#[test]
fn m10_inside_a_blind_hole_takes_the_iso_groove() {
    use qymcad_core::feature::FaceKey;
    use qymcad_core::model::HoleTool;
    for length in [8.0, 4.0] {
        let mut p = Project::default();
        p.new_document();
        let block = p.add_box(40.0, 30.0, 10.0);
        let _ = qymcad_testkit::regenerate(&mut p);
        let f = p.regen_faces.get(&block).and_then(|fs| fs.iter().find(|f| f.normal[2] > 0.9)).cloned().expect("the top");
        let key = FaceKey { index: 0, centroid: [f.centroid.x, f.centroid.y, f.centroid.z], normal: f.normal, id: f.id };
        let h = p.add_hole_at(block, key, key.centroid, HoleTool { kind: 0, diameter: 8.5, depth: 8.0, dia2: 0.0, depth2: 0.0 });
        let (_, shapes) = qymcad_testkit::regenerate(&mut p);
        let before = shapes.get(&h).map(|s| s.volume()).expect("the drilled block");
        let rim =
            p.regen_edges.get(&h).and_then(|es| es.iter().filter(|e| (e.radius - 4.25).abs() < 0.05).max_by(|a, b| a.center[2].total_cmp(&b.center[2])).map(|e| e.id)).expect("the rim at the top");
        let spec = ThreadSpec { standard: ThreadStandard::MetricIso, nominal_d: 10.0, pitch: 1.5, internal: true, fit: 0.0, ..Default::default() };
        let t = p.add_thread(h, rim, spec, length, 0.0, 0.0);
        let (report, shapes) = qymcad_testkit::regenerate(&mut p);
        assert!(report.errors.is_empty(), "M10 in the blind hole over {length} was refused: {:?}", report.errors);
        let got = before - shapes.get(&t).map(|s| s.volume()).unwrap_or(before);
        let want = iso_inner(10.0, 1.5, 8.5, length);
        assert!((got - want).abs() < want * 0.02, "M10 in the blind hole over {length} took {got:.1} mm^3, the ISO groove is {want:.1}");
    }
}
