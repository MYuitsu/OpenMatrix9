use om9_spec_examples::*;

const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DEMO-001", effect: Effect::DocumentWrite,
    roles: &[Role { name: "endpoints", kind: InputKind::Point, min: 2, max: Some(2) }],
    parameters: &[Parameter { name: "distance_mm", kind: ParameterKind::Number, required: true }],
};
fn request() -> Request {
    let mut r = Request { document_revision: Some(7), ..Request::default() };
    for i in 0..2 { r.inputs.push(Input { role: "endpoints".into(), kind: InputKind::Point, key: format!("P{i}"), point: Some([i as f64, 0., 0.]) }); }
    r.options.insert("distance_mm".into(), Value::Number(1.)); r
}
#[test] fn required_role_rejects_incomplete_point_pair() {
    let mut r=request();r.inputs.pop();
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::MissingRole("endpoints".into())));
}
#[test] fn numeric_nan_is_not_a_valid_option() {
    let mut r=request();r.options.insert("distance_mm".into(),Value::Number(f64::NAN));
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::InvalidNumber("distance_mm".into())));
}
#[test] fn point_nan_is_rejected_before_planning() {
    let mut r=request();r.inputs[0].point=Some([0.,f64::NAN,0.]);
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::InvalidPoint));
}
#[test] fn unknown_option_is_not_silently_ignored() {
    let mut r=request();r.options.insert("typo".into(),Value::Boolean(true));
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::UnknownOption("typo".into())));
}
#[test] fn mutation_plan_requires_document_revision() {
    let mut r=request();r.document_revision=None;
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::MissingDocumentRevision));
}
#[test] fn stale_document_is_rejected_before_commit() {
    let p=validate_request(&CONTRACT,&request()).unwrap();
    assert_eq!(p.check_revision(Some(8)),Err(PlanError::StaleDocument));
}
#[test] fn same_reference_is_not_repeated_in_one_role() {
    let mut r=request();r.inputs[1].key="P0".into();
    assert_eq!(validate_request(&CONTRACT,&r),Err(PlanError::DuplicateInput));
}
#[test] fn operation_plan_owns_a_snapshot_not_the_mutable_request() {
    let mut r=request();let p=validate_request(&CONTRACT,&r).unwrap();
    r.inputs[0].point=Some([100.,0.,0.]);
    assert_eq!(p.request.inputs[0].point,Some([0.,0.,0.]));
}
