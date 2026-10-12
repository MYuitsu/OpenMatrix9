// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::geometry_contract::*;
use openmatrix9_rust::native_builder_capability::{Request,Stage};
fn request()->Request {
    Request{stage:Stage::Capture,feature_id:"OM9-SETTING-001".into(),
        recipe:r#"{"schema":1,"evaluator":"om9.affine-template","version":1,"settings":{},"scale":[1,2,3],"offset":[4,5,6]}"#.into(),
        scale_to_gem:true,record_count:1,template_count:1,target_count:1,dimensions:None}
}
fn capability(input:RepresentationKind,request:Request)->Result<OperationCapability,CapabilityError>{
    operation_capability(OperationId::NativeBuilder,input,OperationOptions::NativeBuilder(request))
}
#[test]
fn existing_affine_requests_require_shared_owned_part_capability_before_copy(){
    for stage in [Stage::Declare,Stage::Capture,Stage::Construct]{
        for input in [RepresentationKind::Point,RepresentationKind::Curve,RepresentationKind::Polycurve,
            RepresentationKind::TrimmedFace,RepresentationKind::Shell,RepresentationKind::Solid,RepresentationKind::Brep]{
            let mut r=request();r.stage=stage;
            if stage==Stage::Construct{r.dimensions=Some(([2.,4.,8.],[4.,4.,4.]));}
            let c=capability(input,r.clone()).expect("existing Part affine-template evaluator needs shared admission");
            assert_eq!(c.backend,Backend::OcctEdit);assert_eq!(c.operation,OperationId::NativeBuilder);
            assert_eq!(c.options,OperationOptions::NativeBuilder(r));
            for guard in [OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
                OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts]{assert!(c.constraints.contains(&guard));}
        }
    }
}
#[test]
fn rejects_actual_nonpart_sources_and_stored_options_before_native_solve(){
    for input in [RepresentationKind::Mesh,RepresentationKind::PointCloud,RepresentationKind::Instance,
        RepresentationKind::Group,RepresentationKind::Annotation,RepresentationKind::Retained,RepresentationKind::Unknown,
        RepresentationKind::Surface,RepresentationKind::LightweightExtrusion,RepresentationKind::SubD,RepresentationKind::Mixed]{
        assert_eq!(capability(input,request()),Err(CapabilityError::UnsupportedRepresentation(input)));
    }
    let mut invalid=Vec::new();
    let mut r=request();r.recipe="{}".into();invalid.push(r);
    let mut r=request();r.feature_id="builder".into();invalid.push(r);
    let mut r=request();r.template_count=1025;invalid.push(r);
    let mut r=request();r.target_count=1025;invalid.push(r);
    let mut r=request();r.record_count=usize::MAX;invalid.push(r);
    let mut r=request();r.stage=Stage::Construct;invalid.push(r);
    let mut r=request();r.dimensions=Some(([2.,4.,8.],[0.,4.,4.]));invalid.push(r);
    for r in invalid{assert_eq!(capability(RepresentationKind::Solid,r),Err(CapabilityError::UnsupportedOptions));}
}
