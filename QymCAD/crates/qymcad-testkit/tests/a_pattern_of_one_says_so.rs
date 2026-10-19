//! A PATTERN OF ONE COPY IS THE BODY ALONE and says so. Reported behaviour: a linear and a circular pattern of 1 stood
//! green, the body unchanged to the mm^3.
use qymcad_core::errors::CoreError;
use qymcad_core::model::Project;

#[test]
fn a_pattern_of_one_copy_is_refused_in_words() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let lin = p.add_linear_array(block, 50.0, 0.0, 0.0, 1);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.iter().any(|(id, e)| *id == lin && matches!(e, CoreError::ArrayOfOne)), "a linear pattern of 1 was not refused in words: {:?}", report.errors);
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let circ = p.add_circular_array(block, 1, 360.0);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.iter().any(|(id, e)| *id == circ && matches!(e, CoreError::ArrayOfOne)), "a circular pattern of 1 was not refused in words: {:?}", report.errors);
}
