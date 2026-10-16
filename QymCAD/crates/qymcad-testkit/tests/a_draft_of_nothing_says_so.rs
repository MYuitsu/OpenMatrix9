//! A DRAFT OF 0 DEGREES TILTS NOTHING and says so. Reported behaviour: the front of a block drafted by 0 stood green,
//! the body as it was.
use qymcad_core::errors::CoreError;
use qymcad_core::model::Project;

#[test]
fn a_draft_of_zero_is_refused_in_words() {
    let mut p = Project::default();
    p.new_document();
    let block = p.add_box(40.0, 30.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    let fs = p.regen_faces.get(&block).cloned().expect("the faces of the block");
    let front = fs.iter().find(|f| f.normal[1] < -0.9).expect("the front").id;
    let bottom = fs.iter().find(|f| f.normal[2] < -0.9).expect("the bottom").id;
    let d = p.add_draft(block, vec![front], bottom, 0.0, false);
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.iter().any(|(id, e)| *id == d && matches!(e, CoreError::DraftAngleZero)), "a draft of 0 was not refused in words: {:?}", report.errors);
}
