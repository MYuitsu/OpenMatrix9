use openmatrix9_rust::geometry_contract::*;
use openmatrix9_rust::native_rebuild_capability::{Request,Stage};

fn request()->Request{Request{stage:Stage::Construct,input_count:1,point_count:8,degree:3,
    delete_input:true,current_layer:false,preview:false,closed:false}}
fn capability(input:RepresentationKind,r:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::EditCurve,input,OperationOptions::NativeRebuild(r))
}
#[test]
fn existing_rebuild_types_share_policy_and_retain_real_options(){
    for input in [RepresentationKind::Curve,RepresentationKind::Polycurve,RepresentationKind::Brep]{
        for closed in [false,true]{for preview in [false,true]{for delete_input in [false,true]{
            let r=Request{closed,preview,delete_input,..request()};
            let result=capability(input,r.clone()).unwrap();
            assert_eq!(result.backend,Backend::OcctEdit);
            assert_eq!(result.evidence_id,"OM9-CURVE-009");
            assert_eq!(result.options,OperationOptions::NativeRebuild(r));
            for constraint in [OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
                OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts]{
                assert!(result.constraints.contains(&constraint));
            }
        }}}
    }
}
#[test]
fn wrong_concrete_representations_and_operations_are_rejected(){
    for input in [RepresentationKind::Unknown,RepresentationKind::Point,RepresentationKind::TrimmedFace,
        RepresentationKind::Shell,RepresentationKind::Solid,RepresentationKind::Mesh,
        RepresentationKind::PointCloud,RepresentationKind::Instance,RepresentationKind::Group,
        RepresentationKind::Annotation,RepresentationKind::Retained]{
        assert_eq!(capability(input,request()),Err(CapabilityError::UnsupportedRepresentation(input)));
    }
    assert!(operation_capability(OperationId::EditSurface,RepresentationKind::Curve,
        OperationOptions::NativeRebuild(request())).is_err());
}
#[test]
fn empty_declaration_can_prompt_but_capture_and_solve_require_current_sources(){
    assert!(capability(RepresentationKind::Curve,Request{stage:Stage::Declare,input_count:0,..request()}).is_ok());
    for stage in [Stage::Capture,Stage::Construct]{
        for count in [0,17,usize::MAX]{
            assert_eq!(capability(RepresentationKind::Curve,Request{stage,input_count:count,..request()}),
                Err(CapabilityError::UnsupportedOptions));
        }
        assert!(capability(RepresentationKind::Polycurve,Request{stage,input_count:16,..request()}).is_ok());
    }
}
#[test]
fn actual_rebuild_count_degree_and_disabled_layer_rules_are_reused(){
    for (count,degree) in [(2,1),(4,3),(12,11),(256,11)]{
        assert!(capability(RepresentationKind::Curve,Request{point_count:count,degree,..request()}).is_ok());
    }
    for (count,degree) in [(1,1),(4,4),(257,3),(8,0),(20,12),(usize::MAX,3),(8,usize::MAX)]{
        assert_eq!(capability(RepresentationKind::Curve,Request{point_count:count,degree,..request()}),
            Err(CapabilityError::UnsupportedOptions));
    }
    assert_eq!(capability(RepresentationKind::Curve,Request{current_layer:true,..request()}),
        Err(CapabilityError::UnsupportedOptions));
}
#[test]
fn previous_inspection_exchange_edit_and_surface_policies_remain_available(){
    assert!(operation_capability(OperationId::Inspect,RepresentationKind::Curve,OperationOptions::InspectDescriptor).is_ok());
    assert!(operation_capability(OperationId::ExportWorkingV5,RepresentationKind::Curve,OperationOptions::WorkingExchange{
        archive_version:5,conversion:ConversionPolicy::ExistingConverters,preservation:PreservationPolicy::ReportObservedFacts}).is_ok());
    assert!(operation_capability(OperationId::NativeEdit,RepresentationKind::Curve,
        OperationOptions::NativeEdit{kind:1,flags:4,mode:0}).is_ok());
    assert!(operation_capability(OperationId::EditSurface,RepresentationKind::Curve,
        OperationOptions::NativeSurface(openmatrix9_rust::native_surface_capability::Request::new(
            openmatrix9_rust::surface::Kind::Loft,Stage::Construct,2))).is_ok());
}
