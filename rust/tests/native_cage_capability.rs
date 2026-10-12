// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{cage_commands::{Kind,Phase},native_cage_capability::{Request,Stage,Role},geometry_contract::*};
fn cap(input:RepresentationKind,r:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::NativeCage,input,OperationOptions::NativeCage(r))
}
fn complete(kind:Kind)->Request{Request{phase:Phase::Ready,input_count:1,
    has_control:kind==Kind::Capture,lo:[0.;3],hi:[10.;3],..Request::new(kind,Stage::Construct)}}
#[test]
fn existing_cage_services_keep_defaults_types_and_feature_identity(){
    use RepresentationKind::*;
    for (kind,id) in [(Kind::Create,"OM9-TRANSFORM-041"),(Kind::Capture,"OM9-TRANSFORM-020"),(Kind::Release,"OM9-TRANSFORM-022")]{
        let r=complete(kind);assert_eq!(r.counts,[2;3]);assert_eq!(r.degrees,[1;3]);assert!(!r.local);assert_eq!(r.falloff,0.);
        for input in [Point,Curve,Polycurve,TrimmedFace,Shell,Solid,Brep,Mesh]{
            let c=cap(input,r).unwrap();assert_eq!(c.evidence_id,id);assert_eq!(c.backend,Backend::NativeCageAdapters);
            assert_eq!(c.options,OperationOptions::NativeCage(r));assert!(c.constraints.contains(&OperationConstraint::ValidateNativeInput));
        }
        for input in [Unknown,PointCloud,Instance,Group,Annotation,SubD,Mixed,Retained]{
            assert_eq!(cap(input,r),Err(CapabilityError::UnsupportedRepresentation(input)));
        }
    }
}
#[test]
fn declaration_can_prompt_but_construct_needs_real_inputs_and_control(){
    for kind in [Kind::Create,Kind::Capture,Kind::Release]{let r=Request::new(kind,Stage::Declare);
        assert!(cap(RepresentationKind::Brep,r).is_ok());
        assert_eq!(cap(RepresentationKind::Brep,Request{stage:Stage::Construct,..r}),Err(CapabilityError::UnsupportedOptions));
        let c=complete(kind);assert!(cap(RepresentationKind::Brep,c).is_ok());
        assert_eq!(cap(RepresentationKind::Brep,Request{input_count:0,..c}),Err(CapabilityError::UnsupportedOptions));
        if kind==Kind::Capture{assert_eq!(cap(RepresentationKind::Brep,Request{has_control:false,..c}),Err(CapabilityError::UnsupportedOptions));}
    }
    assert_eq!(cap(RepresentationKind::Brep,Request::new(Kind::None,Stage::Declare)),Err(CapabilityError::UnsupportedOperation(OperationId::NativeCage)));
}
#[test]
fn explicit_api_bounds_and_archived_relationships_do_not_require_interactive_selection(){
    let r=Request{explicit_bounds:true,input_count:0,..complete(Kind::Create)};
    assert!(cap(RepresentationKind::Brep,r).is_ok());
    let r=Request{retained:true,role:Role::RetainedControl,input_count:0,..complete(Kind::Capture)};
    assert!(cap(RepresentationKind::Retained,r).is_ok());
    assert_eq!(cap(RepresentationKind::Retained,Request{role:Role::Geometry,..r}),Err(CapabilityError::UnsupportedRepresentation(RepresentationKind::Retained)));
    assert_eq!(cap(RepresentationKind::Retained,Request{retained:false,..r}),Err(CapabilityError::UnsupportedOptions));
    assert!(cap(RepresentationKind::Mesh,Request{role:Role::BindingStorage,..complete(Kind::Release)}).is_ok());
    assert_eq!(cap(RepresentationKind::Brep,Request{role:Role::BindingStorage,..complete(Kind::Create)}),Err(CapabilityError::UnsupportedOptions));
}
#[test]
fn real_create_bounds_counts_degrees_and_budget_are_checked_before_allocation(){
    let r=complete(Kind::Create);
    for (counts,degrees) in [([1,2,2],[1;3]),([129,2,2],[1;3]),([2;3],[2,1,1]),([128;3],[1;3]),([20;3],[17,1,1])]{
        assert_eq!(cap(RepresentationKind::Brep,Request{counts,degrees,..r}),Err(CapabilityError::UnsupportedOptions));
    }
    assert!(cap(RepresentationKind::Brep,Request{counts:[100;3],degrees:[16;3],..r}).is_ok());
    for hi in [[10.,10.,0.],[f64::INFINITY;3],[f64::NAN;3],[1e-310;3]]{
        assert_eq!(cap(RepresentationKind::Brep,Request{hi,..r}),Err(CapabilityError::UnsupportedOptions));
    }
    assert!(cap(RepresentationKind::Brep,Request{lo:[-10.;3],hi:[0.;3],..r}).is_ok());
}
#[test]
fn capture_roles_region_and_independent_recompute_use_actual_options(){
    let r=complete(Kind::Capture);
    assert!(cap(RepresentationKind::Mesh,Request{local:true,falloff:2.5,..r}).is_ok());
    for falloff in [-1.,f64::NAN,f64::INFINITY]{assert_eq!(cap(RepresentationKind::Mesh,Request{falloff,..r}),Err(CapabilityError::UnsupportedOptions));}
    assert!(cap(RepresentationKind::Brep,Request{role:Role::NativeControl,..r}).is_ok());
    assert_eq!(cap(RepresentationKind::Mesh,Request{role:Role::NativeControl,..r}),Err(CapabilityError::UnsupportedRepresentation(RepresentationKind::Mesh)));
    let pick=Request{stage:Stage::Capture,phase:Phase::Control,role:Role::NativeControl,has_control:false,..r};
    assert!(cap(RepresentationKind::Brep,pick).is_ok());
    assert_eq!(cap(RepresentationKind::Brep,Request{phase:Phase::Inputs,..pick}),Err(CapabilityError::UnsupportedOptions));
    assert!(cap(RepresentationKind::Mesh,Request{stage:Stage::Recompute,local:true,falloff:2.5,..r}).is_ok());
    assert_eq!(cap(RepresentationKind::Brep,Request{stage:Stage::Recompute,..complete(Kind::Create)}),Err(CapabilityError::UnsupportedOptions));
    assert!(operation_capability(OperationId::Inspect,RepresentationKind::Brep,OperationOptions::InspectDescriptor).is_ok());
    assert!(operation_capability(OperationId::Boolean,RepresentationKind::Brep,OperationOptions::NativeCage(r)).is_err());
}
#[test]
fn existing_sessions_snapshot_only_owned_options_and_reject_incomplete_prepare(){
    use openmatrix9_rust::cage_commands::{Session,Input};
    for kind in [Kind::Create,Kind::Capture,Kind::Release]{let mut s=Session::new(kind).unwrap();
        let before=s.capability_request(Stage::Declare,Role::Geometry).unwrap();assert_eq!(before.input_count,0);
        assert!(cap(RepresentationKind::Brep,before).is_ok());assert!(s.validate_commit().is_err());
        s.add(Input::new("Source","snapshot",[0.;3],[10.;3]).unwrap()).unwrap();s.advance().unwrap();
        if kind==Kind::Capture{s.set_control(Input::new("Control","basis",[0.;3],[10.;3]).unwrap(),false).unwrap();s.option("Region","Local").unwrap();s.option("Falloff","2.5").unwrap();}
        let ready=s.capability_request(Stage::Construct,Role::Geometry).unwrap();assert_eq!(ready.input_count,1);
        assert!(cap(RepresentationKind::Brep,ready).is_ok());assert!(s.validate_commit().is_ok());
        assert_eq!(before.input_count,0);assert_eq!(before.falloff,0.);
    }
    // Public safe Session data must still be checked at prepare, even if callers
    // bypass normal option transitions. No source/signature buffer is cloned.
    let mut s=Session::new(Kind::Create).unwrap();s.phase=Phase::Ready;s.lo=[0.;3];s.hi=[10.;3];
    assert!(s.validate_commit().is_err());
}
