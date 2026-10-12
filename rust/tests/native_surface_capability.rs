use openmatrix9_rust::geometry_contract::*;
use openmatrix9_rust::native_surface_capability::{Request,Stage};
use openmatrix9_rust::surface::Kind;

fn capability(input:RepresentationKind, request:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::EditSurface,input,OperationOptions::NativeSurface(request))
}
fn request(kind:Kind)->Request{Request::new(kind,Stage::Construct,kind.rails()+kind.minimum_sections())}

#[test]
fn existing_surface_families_use_native_backend_and_retain_request(){
    for kind in [Kind::Sweep1,Kind::Sweep2,Kind::Loft]{
        for input in [RepresentationKind::Curve,RepresentationKind::Polycurve,RepresentationKind::Brep]{
            let r=request(kind);let result=capability(input,r.clone()).unwrap();
            assert_eq!(result.backend,Backend::OcctEdit);
            assert_eq!(result.evidence_id,kind.feature_id());
            assert_eq!(result.options,OperationOptions::NativeSurface(r));
            for constraint in [OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts]{
                assert!(result.constraints.contains(&constraint));
            }
        }
    }
}
#[test]
fn concrete_surface_types_and_operation_must_match(){
    for input in [RepresentationKind::Unknown,RepresentationKind::Mesh,RepresentationKind::PointCloud,RepresentationKind::Solid,RepresentationKind::TrimmedFace,RepresentationKind::Shell,RepresentationKind::Instance,RepresentationKind::Annotation]{
        assert_eq!(capability(input,request(Kind::Loft)),Err(CapabilityError::UnsupportedRepresentation(input)));
    }
    assert!(operation_capability(OperationId::Boolean,RepresentationKind::Curve,OperationOptions::NativeSurface(request(Kind::Loft))).is_err());
}
#[test]
fn declaration_prompts_without_authorizing_unknown_or_empty_solve(){
    for kind in [Kind::Sweep1,Kind::Sweep2,Kind::Loft]{
        for stage in [Stage::Declare,Stage::Capture]{
            assert!(capability(RepresentationKind::Curve,Request::new(kind,stage,0)).is_ok());
            assert_eq!(capability(RepresentationKind::Unknown,Request::new(kind,stage,0)),Err(CapabilityError::UnsupportedRepresentation(RepresentationKind::Unknown)));
        }
        assert_eq!(capability(RepresentationKind::Curve,Request::new(kind,Stage::Construct,0)),Err(CapabilityError::UnsupportedOptions));
    }
}
#[test]
fn styles_closedness_and_existing_cardinality_are_checked(){
    for style in 0..=5{
        let mut r=request(Kind::Loft);r.style=style;
        assert!(capability(RepresentationKind::Curve,r.clone()).is_ok());
        r.closed=true;r.input_count=3;
        assert_eq!(capability(RepresentationKind::Curve,r).is_ok(),style!=5);
    }
    for kind in [Kind::Sweep1,Kind::Sweep2]{
        let mut r=request(kind);r.style=1;
        assert_eq!(capability(RepresentationKind::Curve,r),Err(CapabilityError::UnsupportedOptions));
        let mut r=request(kind);r.closed=true;r.rail_closed=[true,true];
        assert!(capability(RepresentationKind::Curve,r.clone()).is_err());
        r.input_count+=1;assert!(capability(RepresentationKind::Curve,r.clone()).is_ok());
        r.rail_closed[0]=false;assert!(capability(RepresentationKind::Curve,r).is_err());
    }
    let mut r=request(Kind::Loft);r.closed=true;
    assert!(capability(RepresentationKind::Curve,r).is_err());
}
#[test]
fn fitting_numbers_and_all_inert_context_fields_are_retained(){
    for mode in 0..=2{for count in [2,16,256]{
        let mut r=request(Kind::Sweep1);r.section_mode=mode;r.point_count=count;
        r.frenet=true;r.history=true;r.history_command=true;r.preview=false;
        assert!(capability(RepresentationKind::Curve,r).is_ok());
    }}
    for tolerance in [f64::NAN,f64::INFINITY,0.,-1.,1e6+1.]{
        let mut r=request(Kind::Loft);r.tolerance=tolerance;
        assert!(capability(RepresentationKind::Curve,r).is_err());
    }
    for (mode,count) in [(3,16),(0,1),(0,257)]{
        let mut r=request(Kind::Loft);r.section_mode=mode;r.point_count=count;
        assert!(capability(RepresentationKind::Curve,r).is_err());
    }
}
#[test]
fn maintain_height_and_slashes_use_actual_sweep2_context(){
    let mut r=request(Kind::Sweep2);r.maintain_height=true;
    assert!(capability(RepresentationKind::Curve,r.clone()).is_ok());
    r.input_count=4;assert!(capability(RepresentationKind::Curve,r).is_err());
    for kind in [Kind::Sweep1,Kind::Loft]{
        let mut r=request(kind);r.maintain_height=true;
        assert!(capability(RepresentationKind::Curve,r).is_err());
    }
    let mut r=request(Kind::Sweep2);r.slashes=vec![(0.2,0.3),(0.8,0.9)];
    assert!(capability(RepresentationKind::Curve,r.clone()).is_ok());
    for closed in [[true,false],[false,true]]{
        let mut bad=r.clone();bad.rail_closed=closed;
        assert!(capability(RepresentationKind::Curve,bad).is_err());
    }
    for pairs in [vec![(0.,0.3)],vec![(0.2,f64::NAN)],vec![(0.8,0.3),(0.2,0.9)],vec![(0.2,0.3);65]]{
        let mut bad=r.clone();bad.slashes=pairs;
        assert!(capability(RepresentationKind::Curve,bad).is_err());
    }
    r.continuity_a=1;assert!(capability(RepresentationKind::Curve,r).is_err());
}
#[test]
fn continuity_and_match_tangents_preserve_the_current_supported_slice(){
    let mut sweep=request(Kind::Sweep2);sweep.input_count=4;sweep.continuity_a=1;sweep.continuity_b=2;
    assert!(capability(RepresentationKind::Curve,sweep.clone()).is_ok());
    for change in 0..6{
        let mut r=sweep.clone();match change{
            0=>r.closed=true,1=>r.section_mode=1,2=>r.match_start=true,
            3=>r.continuity_a=3,4=>r.input_count=3,_=>r.input_count=35,
        }
        assert!(capability(RepresentationKind::Curve,r).is_err());
    }
    for style in 0..=5{
        let mut r=request(Kind::Loft);r.match_start=true;r.match_end=true;r.style=style;
        assert_eq!(capability(RepresentationKind::Curve,r).is_ok(),matches!(style,0|3));
    }
    let mut r=request(Kind::Sweep1);r.continuity_a=1;
    assert!(capability(RepresentationKind::Curve,r).is_err());
}
#[test]
fn unrelated_inspection_exchange_and_edit_policy_stays_available(){
    assert!(operation_capability(OperationId::Inspect,RepresentationKind::Unknown,OperationOptions::InspectDescriptor).is_ok());
    assert!(operation_capability(OperationId::ImportWorkingV5,RepresentationKind::Solid,OperationOptions::WorkingExchange{archive_version:5,conversion:ConversionPolicy::ExistingConverters,preservation:PreservationPolicy::ReportObservedFacts}).is_ok());
    assert!(operation_capability(OperationId::NativeEdit,RepresentationKind::Curve,OperationOptions::NativeEdit{kind:1,flags:4,mode:0}).is_ok());
}
