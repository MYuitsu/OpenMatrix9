use openmatrix9_rust::{circle::{Circle,Mode as CircleMode},ellipse::{Ellipse,Mode as EllipseMode},
    rectangle::Rectangle,spline::Knots,native_curve_capability::{Options,Request,Stage}};
use openmatrix9_rust::geometry_contract::*;
fn request(options:Options)->Request{
    let output_count=match options{Options::Line{..}=>2,Options::Polyline{..}|Options::InterpCrv{..}=>3,
        Options::Rectangle(_)=>5,_=>65};
    Request{options,stage:Stage::Construct,input_count:2,output_count,has_plan:true,mm_per_unit:1.}
}
fn capability(input:RepresentationKind,r:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::ConstructCurve,input,OperationOptions::NativeCurve(r))
}
fn families()->Vec<Options>{vec![Options::Line{both_sides:true,length:None},
    Options::Polyline{close:true,length:Some(2.)},Options::InterpCrv{degree:3,knots:Knots::Chord,close:true},
    Options::Rectangle(Rectangle::default()),Options::Circle(Circle::default()),Options::Ellipse(Ellipse::default())]}
#[test]
fn evidence_ids_match_original_feature_specs(){
    let specs=[
        ("OM9-CURVE-002",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-002-line.md")),
        ("OM9-CURVE-001",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-001-polyline.md")),
        ("OM9-CURVE-003",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-003-interp-curve.md")),
        ("OM9-CURVE-004",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-004-rectangle.md")),
        ("OM9-CURVE-005",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-005-circle.md")),
        ("OM9-CURVE-006",include_str!("../../OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-006-ellipse.md")),
    ];
    for (options,(id,spec)) in families().into_iter().zip(specs){
        assert!(spec.lines().any(|line|line==format!("id: {id}")));
        assert_eq!(capability(RepresentationKind::Point,request(options)).unwrap().evidence_id,id);
    }
}
#[test]
fn current_typed_families_retain_actual_options_and_backend(){
    for options in families(){let r=request(options.clone());let c=capability(RepresentationKind::Point,r.clone()).unwrap();
        assert_eq!(c.backend,Backend::OcctEdit);assert_eq!(c.evidence_id,options.feature_id());
        assert_eq!(c.options,OperationOptions::NativeCurve(r));
        for guard in [OperationConstraint::ValidateNativeInput,OperationConstraint::PrivateOwnedInput,
            OperationConstraint::ValidateSourceStamp,OperationConstraint::ReportConversionFacts]{assert!(c.constraints.contains(&guard));}
    }
}
#[test]
fn declaration_and_incomplete_preview_prompt_without_authorizing_unknown_or_empty_construct(){
    for options in families(){for stage in [Stage::Declare,Stage::Capture,Stage::Preview]{
        let r=Request{stage,input_count:0,output_count:0,has_plan:false,..request(options.clone())};
        assert!(capability(RepresentationKind::Point,r.clone()).is_ok());
        assert_eq!(capability(RepresentationKind::Unknown,r),Err(CapabilityError::UnsupportedRepresentation(RepresentationKind::Unknown)));
    }
        assert_eq!(capability(RepresentationKind::Point,Request{has_plan:false,..request(options)}),Err(CapabilityError::UnsupportedOptions));
    }
}
#[test]
fn native_reference_types_depend_on_actual_circle_and_ellipse_modes(){
    for mode in [CircleMode::AroundCurve,CircleMode::Tangent,CircleMode::FitPoints]{
        let r=request(Options::Circle(Circle{mode,..Circle::default()}));
        assert!(capability(RepresentationKind::Curve,r.clone()).is_ok());
        assert!(capability(RepresentationKind::Polycurve,r.clone()).is_ok());
        for kind in [RepresentationKind::Mesh,RepresentationKind::PointCloud,RepresentationKind::TrimmedFace,RepresentationKind::Solid]{
            assert_eq!(capability(kind,r.clone()).is_ok(),mode==CircleMode::FitPoints);
        }
    }
    let mut ellipse=Ellipse::default();ellipse.mode=EllipseMode::AroundCurve;
    let r=request(Options::Ellipse(ellipse));
    assert!(capability(RepresentationKind::Curve,r).is_ok());
    for options in families(){assert_eq!(capability(RepresentationKind::Curve,request(options)),
        Err(CapabilityError::UnsupportedRepresentation(RepresentationKind::Curve)));}
}
#[test]
fn current_basis_numeric_limits_and_foci_constraints_are_reused(){
    for degree in [1,3,5,7,9,11]{
        assert!(capability(RepresentationKind::Point,request(Options::InterpCrv{degree,knots:Knots::SqrtChord,close:false})).is_ok());
    }
    for degree in [0,2,12,usize::MAX]{assert_eq!(capability(RepresentationKind::Point,
        request(Options::InterpCrv{degree,knots:Knots::Uniform,close:false})),Err(CapabilityError::UnsupportedOptions));}
    let mut circle=Circle::default();circle.degree=8;circle.point_count=8;
    assert_eq!(capability(RepresentationKind::Point,request(Options::Circle(circle))),Err(CapabilityError::UnsupportedOptions));
    let mut ellipse=Ellipse::default();ellipse.mark_foci=true;
    assert_eq!(capability(RepresentationKind::Point,request(Options::Ellipse(ellipse))),Err(CapabilityError::UnsupportedOptions));
    for length in [f64::NAN,f64::INFINITY,0.,1e10]{assert_eq!(capability(RepresentationKind::Point,
        request(Options::Line{both_sides:false,length:Some(length)})),Err(CapabilityError::UnsupportedOptions));}
    for scale in [f64::NAN,0.,-1.]{assert_eq!(capability(RepresentationKind::Point,Request{mm_per_unit:scale,..request(Options::Circle(Circle::default()))}),Err(CapabilityError::UnsupportedOptions));}
}
#[test]
fn counts_and_other_operations_do_not_gain_permission(){
    assert_eq!(capability(RepresentationKind::Point,Request{input_count:4097,..request(Options::Polyline{close:false,length:None})}),Err(CapabilityError::UnsupportedOptions));
    assert_eq!(capability(RepresentationKind::Point,Request{input_count:1025,..request(Options::Circle(Circle{mode:CircleMode::FitPoints,..Circle::default()}))}),Err(CapabilityError::UnsupportedOptions));
    assert_eq!(capability(RepresentationKind::Point,Request{input_count:257,..request(Options::InterpCrv{degree:3,knots:Knots::Uniform,close:false})}),Err(CapabilityError::UnsupportedOptions));
    assert!(operation_capability(OperationId::Inspect,RepresentationKind::Point,OperationOptions::InspectDescriptor).is_ok());
    assert!(operation_capability(OperationId::Boolean,RepresentationKind::Point,OperationOptions::NativeCurve(request(Options::Circle(Circle::default())))).is_err());
}
#[test]
fn actual_sessions_capture_current_options_and_cancellation_state(){
    use openmatrix9_rust::curve::CurveSession;
    for name in ["Line","Polyline","InterpCrv","Rectangle","Circle","Ellipse"]{
        let mut session=CurveSession::default();session.start(name).unwrap();
        let r=session.capability_request(Stage::Declare).unwrap();
        assert_eq!(r.input_count,0);assert_eq!(r.output_count,0);assert!(!r.has_plan);
        assert!(capability(RepresentationKind::Point,r).is_ok());
        session.cancel();assert!(session.capability_request(Stage::Construct).is_none());
    }
    let mut session=CurveSession::default();session.start("Circle").unwrap();session.set_input_scale(25.4).unwrap();
    session.input("FitPoints").unwrap();session.input("PointCount=12").unwrap();session.input("Degree=5").unwrap();session.input("History=Yes").unwrap();
    let r=session.capability_request(Stage::Capture).unwrap();assert_eq!(r.mm_per_unit,25.4);
    let Options::Circle(options)=&r.options else{panic!("expected actual Circle options")};
    assert_eq!(options.mode,CircleMode::FitPoints);assert_eq!(options.point_count,12);assert_eq!(options.degree,5);assert!(options.history);
    session.input("PointCount=16").unwrap();
    let Options::Circle(previous)=r.options else{unreachable!()};assert_eq!(previous.point_count,12);
}
#[test]
fn current_completed_geometry_and_latent_ellipse_foci_use_effective_options(){
    use openmatrix9_rust::curve::{CurveSession,Effect};
    let mut s=CurveSession::default();s.start("Line").unwrap();s.point([0.;3]).unwrap();
    assert_eq!(s.point([2.,0.,0.]).unwrap(),Effect::Commit);
    let r=s.capability_request(Stage::Construct).unwrap();assert_eq!(r.output_count,2);assert!(r.has_plan);
    assert!(capability(RepresentationKind::Point,r).is_ok());
    s.start("Ellipse").unwrap();s.input("FromFoci").unwrap();s.input("MarkFoci=Yes").unwrap();s.input("Center").unwrap();
    let r=s.capability_request(Stage::Capture).unwrap();
    let Options::Ellipse(e)=&r.options else{panic!("actual Ellipse request")};
    assert_eq!(e.mode,EllipseMode::Center);assert!(!e.mark_foci);
    assert!(capability(RepresentationKind::Point,r).is_ok());
}
