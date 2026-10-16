//! A TORUS NEVER PASSES THROUGH ITSELF: a tube as thick as the ring or thicker is refused in words. Reported behaviour:
//! a ring of 5 and a tube of 5 (or 8) stood green as a spindle of 2467 (6317) mm^3.
use qymcad_core::errors::CoreError;
use qymcad_core::model::Project;

#[test]
fn a_tube_as_thick_as_the_ring_is_refused() {
    for minor in [5.0, 8.0] {
        let mut p = Project::default();
        p.new_document();
        let t = p.add_torus(5.0, minor);
        let (report, _) = qymcad_testkit::regenerate(&mut p);
        assert!(report.errors.iter().any(|(id, e)| *id == t && matches!(e, CoreError::TorusThroughItself)), "a ring of 5 with a tube of {minor} was not refused in words: {:?}", report.errors);
    }
    let mut p = Project::default();
    p.new_document();
    let t = p.add_torus(5.0, 4.0);
    let (report, shapes) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "a ring of 5 with a tube of 4 was refused: {:?}", report.errors);
    let v = shapes.get(&t).map(|s| s.volume()).unwrap_or(0.0);
    let want = 2.0 * std::f64::consts::PI.powi(2) * 5.0 * 16.0;
    assert!((v - want).abs() < 1.0, "a ring of 5 with a tube of 4 is {v:.1} mm^3, 2 pi^2 R r^2 = {want:.1}");
}
