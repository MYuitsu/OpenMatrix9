// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned options from existing Box/Sphere sessions, without host owners.
use crate::{solid::{Kind,Mode},geometry_contract::*};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Stage{Declare,Capture,Preview,Construct}
#[derive(Clone,Debug,PartialEq)]
pub struct Request{
    pub kind:Kind,pub mode:Mode,pub stage:Stage,pub phase:u32,pub point_count:usize,
    pub length:f64,pub width:f64,pub radius:f64,pub point_constraint:bool,
    pub constraint_types:Vec<bool>,pub first:[f64;3],pub axes:[[f64;3];3],
    pub frame_origin:[f64;3],pub frame_axes:[[f64;3];3],
    pub circle:Option<([f64;3],f64,[f64;3])>,pub output:Option<[f64;15]>,
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_solid_capability(representation:u32,stage:u32)->u32{
    std::panic::catch_unwind(||{
        let stage=match stage{1=>Stage::Declare,2=>Stage::Capture,3=>Stage::Construct,4=>Stage::Preview,_=>return 102};
        use RepresentationKind::*;
        let input=match representation{1=>Point,2=>Curve,3=>Polycurve,4=>TrimmedFace,5=>Shell,6=>Solid,7=>Brep,
            8=>Mesh,9=>PointCloud,10=>Instance,11=>Group,12=>Annotation,13=>Retained,_=>return 101};
        match crate::solid::current_construction_capability(input,stage){
            Some(Ok(c)) if c.backend==Backend::OcctEdit=>4,
            Some(Ok(_)|Err(CapabilityError::UnsupportedOperation(_)))=>100,
            Some(Err(CapabilityError::UnsupportedRepresentation(_)))=>101,
            Some(Err(CapabilityError::UnsupportedOptions))=>102,None=>103,
        }
    }).unwrap_or(103)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_solid_declared(kind:u32)->bool{
    std::panic::catch_unwind(||{
        let kind=match kind{1=>Kind::Box,2=>Kind::Sphere,_=>return false};
        crate::solid::Session::new(kind).construction_capability(RepresentationKind::Point,Stage::Declare).is_ok()
    }).unwrap_or(false)
}
pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::ConstructSolid||r.phase==0{return Err(CapabilityError::UnsupportedOperation(operation));}
    use RepresentationKind::*;
    let allowed=input==Point||(r.kind==Kind::Sphere&&match r.mode{
        Mode::FitPoints=>matches!(input,Curve|Polycurve|TrimmedFace|Shell|Solid|Brep|Mesh|PointCloud),
        Mode::AroundCurve|Mode::Tangent=>matches!(input,Curve|Polycurve),_=>false,
    });
    if !allowed{return Err(CapabilityError::UnsupportedRepresentation(input));}
    let unsupported=||Err(CapabilityError::UnsupportedOptions);
    let mode_valid=match r.kind{
        Kind::Box=>matches!(r.mode,Mode::Corners|Mode::Diagonal|Mode::ThreePoint|Mode::Vertical|Mode::Center|Mode::Cube),
        Kind::Sphere=>matches!(r.mode,Mode::Center|Mode::Vertical|Mode::TwoPoint|Mode::ThreePoint|Mode::FourPoint|Mode::FitPoints|Mode::AroundCurve|Mode::Tangent),
    };
    let phase_valid=match r.kind{Kind::Box=>matches!(r.phase,1|2|3|4|6),Kind::Sphere=>matches!(r.phase,1|5|6|7|8|9|10|11|12|15|16|17|20|21|22)};
    if !mode_valid||!phase_valid||r.point_count>crate::fit_point_input::MAX_POINTS
        ||r.constraint_types.len()>3||crate::solid::validate_frame(r.first,r.axes).is_err()
        ||crate::solid::validate_frame(r.frame_origin,r.frame_axes).is_err(){return unsupported();}
    if r.mode==Mode::Tangent{
        if r.point_count>3||r.constraint_types.len()!=r.point_count{return unsupported();}
    }else if r.point_constraint||!r.constraint_types.is_empty(){return unsupported();}
    if [r.length,r.width,r.radius].iter().any(|&n|n!=0.&&!crate::solid::dimension_valid(n))||r.radius<0.
        ||(r.kind==Kind::Box&&r.radius!=0.)||(r.kind==Kind::Sphere&&(r.length!=0.||r.width!=0.)){return unsupported();}
    if let Some((center,radius,normal))=r.circle{
        if r.kind!=Kind::Sphere||crate::circle::Plan::new(center,normal,radius).is_err(){return unsupported();}
    }
    if r.output.as_ref().is_some_and(|b|!crate::solid::native_output_valid(r.kind,b)){return unsupported();}
    if r.stage==Stage::Construct&&(r.phase!=6||r.output.is_none()){return unsupported();}
    let id=if r.kind==Kind::Box{"OM9-SOLID-012"}else{"OM9-SOLID-014"};
    Ok((Backend::OcctEdit,&[OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
        OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts],id))
}
