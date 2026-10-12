// SPDX-License-Identifier: LGPL-2.1-or-later
//! Existing Rebuild request, portable values only; native shape facts stay native.
use crate::geometry_contract::*;
pub use crate::native_surface_capability::Stage;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Request{
    pub stage:Stage,pub input_count:usize,pub point_count:usize,pub degree:usize,
    pub delete_input:bool,pub current_layer:bool,pub preview:bool,pub closed:bool,
}

/// Value-only synchronous bridge; no native owner, caller buffer or pointer.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_rebuild_capability(representation:u32,stage:u32,
    input_count:usize,point_count:usize,degree:usize,flags:u32)->u32{
    std::panic::catch_unwind(||{
        let input=match representation{2=>RepresentationKind::Curve,3=>RepresentationKind::Polycurve,
            7=>RepresentationKind::Brep,_=>return 101};
        let stage=match stage{1=>Stage::Declare,2=>Stage::Capture,3=>Stage::Construct,_=>return 102};
        if flags&!15!=0{return 102;}
        let request=Request{stage,input_count,point_count,degree,delete_input:flags&1!=0,
            current_layer:flags&2!=0,preview:flags&4!=0,closed:flags&8!=0};
        match operation_capability(OperationId::EditCurve,input,OperationOptions::NativeRebuild(request)){
            Ok(c) if c.backend==Backend::OcctEdit=>4,
            Ok(_)|Err(CapabilityError::UnsupportedOperation(_))=>100,
            Err(CapabilityError::UnsupportedRepresentation(_))=>101,
            Err(CapabilityError::UnsupportedOptions)=>102,
        }
    }).unwrap_or(103)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_rebuild_declared()->bool{
    om9_native_rebuild_capability(2,1,0,4,3,1)==4
}
pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::EditCurve{return Err(CapabilityError::UnsupportedOperation(operation));}
    // Brep permits the existing connected, curve-only Compound reader. Native
    // face/edge/connectivity checks still apply; this does not admit surfaces.
    if !matches!(input,RepresentationKind::Curve|RepresentationKind::Polycurve|RepresentationKind::Brep){
        return Err(CapabilityError::UnsupportedRepresentation(input));
    }
    if r.input_count>16||(r.stage!=Stage::Declare&&r.input_count==0)||r.current_layer
        ||crate::spline::rebuild_options(r.point_count,r.degree).is_err(){
        return Err(CapabilityError::UnsupportedOptions);
    }
    Ok((Backend::OcctEdit,&[OperationConstraint::ValidateSourceStamp,
        OperationConstraint::ValidateNativeInput,OperationConstraint::PrivateOwnedInput,
        OperationConstraint::ReportConversionFacts],"OM9-CURVE-009"))
}
