// SPDX-License-Identifier: LGPL-2.1-or-later
//! Portable options for the existing native Cage services; no host owners.
use crate::{cage_commands::{Kind,Phase},geometry_contract::*};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u32)]
pub enum Stage{Declare=1,Capture=2,Construct=3,Recompute=4}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u32)]
pub enum Role{Geometry,NativeControl,RetainedControl,BindingStorage}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Request{
    pub kind:Kind,pub stage:Stage,pub phase:Phase,pub role:Role,
    pub input_count:usize,pub has_control:bool,pub retained:bool,pub explicit_bounds:bool,
    pub counts:[usize;3],pub degrees:[usize;3],pub lo:[f64;3],pub hi:[f64;3],
    pub local:bool,pub falloff:f64,
}
/// Fixed, initialized scalar envelope. Passed by value: no host storage, array
/// pointer, or session handle is retained. Integer/flag decoding precedes policy.
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Envelope{
    pub kind:u32,pub stage:u32,pub phase:u32,pub role:u32,pub flags:u32,pub reserved:u32,
    pub input_count:usize,pub counts:[usize;3],pub degrees:[usize;3],
    pub lo:[f64;3],pub hi:[f64;3],pub falloff:f64,
}
impl From<Request> for Envelope{
    fn from(r:Request)->Self{Self{kind:r.kind as u32,stage:r.stage as u32,phase:r.phase as u32,role:r.role as u32,
        flags:u32::from(r.local)|(u32::from(r.has_control)<<1)|(u32::from(r.retained)<<2)|(u32::from(r.explicit_bounds)<<3),
        reserved:0,input_count:r.input_count,counts:r.counts,degrees:r.degrees,lo:r.lo,hi:r.hi,falloff:r.falloff}}
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_cage_capability(representation:u32,envelope:Envelope)->u32{
    std::panic::catch_unwind(||match decode(envelope){Ok(r)=>query(representation,r),Err(status)=>status}).unwrap_or(103)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_cage_declared(kind:u32)->bool{
    std::panic::catch_unwind(||match decode_kind(kind){Ok(k)=>query(7,Request::new(k,Stage::Declare))==7,Err(_)=>false}).unwrap_or(false)
}
/// # Safety
/// `session` must be a live aligned Rust Cage Session handle, borrowed only for
/// this call. Caller serializes access/mutation/free on the GUI thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_session_capability(session:*const crate::cage_commands::Session,
    representation:u32,stage:u32,role:u32)->u32{
    std::panic::catch_unwind(||{
        if session.is_null()||!(session as usize).is_multiple_of(std::mem::align_of::<crate::cage_commands::Session>()){return 103;}
        let (Ok(stage),Ok(role))=(decode_stage(stage),decode_role(role))else{return 102;};
        match unsafe{&*session}.capability_request(stage,role){Ok(r)=>query(representation,r),Err(_)=>100}
    }).unwrap_or(103)
}
fn decode_kind(k:u32)->Result<Kind,u32>{match k{1=>Ok(Kind::Create),2=>Ok(Kind::Capture),3=>Ok(Kind::Release),_=>Err(100)}}
fn decode_stage(s:u32)->Result<Stage,u32>{match s{1=>Ok(Stage::Declare),2=>Ok(Stage::Capture),3=>Ok(Stage::Construct),4=>Ok(Stage::Recompute),_=>Err(102)}}
fn decode_role(r:u32)->Result<Role,u32>{match r{0=>Ok(Role::Geometry),1=>Ok(Role::NativeControl),2=>Ok(Role::RetainedControl),3=>Ok(Role::BindingStorage),_=>Err(102)}}
fn decode(e:Envelope)->Result<Request,u32>{
    let kind=decode_kind(e.kind)?;let stage=decode_stage(e.stage)?;let role=decode_role(e.role)?;
    let phase=match e.phase{1=>Phase::Inputs,2=>Phase::Control,3=>Phase::Options,4=>Phase::Ready,_=>return Err(102)};
    if e.reserved!=0||e.flags&!15!=0{return Err(102);}
    Ok(Request{kind,stage,phase,role,input_count:e.input_count,has_control:e.flags&2!=0,retained:e.flags&4!=0,
        explicit_bounds:e.flags&8!=0,local:e.flags&1!=0,counts:e.counts,degrees:e.degrees,lo:e.lo,hi:e.hi,falloff:e.falloff})
}
fn query(code:u32,r:Request)->u32{
    use RepresentationKind::*;
    let input=match code{1=>Point,2=>Curve,3=>Polycurve,4=>TrimmedFace,5=>Shell,6=>Solid,7=>Brep,
        8=>Mesh,9=>PointCloud,10=>Instance,11=>Group,12=>Annotation,13=>Retained,_=>return 101};
    match operation_capability(OperationId::NativeCage,input,OperationOptions::NativeCage(r)){
        Ok(c)if c.backend==Backend::NativeCageAdapters=>7,
        Ok(_)|Err(CapabilityError::UnsupportedOperation(_))=>100,
        Err(CapabilityError::UnsupportedRepresentation(_))=>101,Err(CapabilityError::UnsupportedOptions)=>102,
    }
}
impl Request{
    pub fn new(kind:Kind,stage:Stage)->Self{Self{kind,stage,phase:Phase::Inputs,role:Role::Geometry,
        input_count:0,has_control:false,retained:false,explicit_bounds:false,
        counts:[2;3],degrees:[1;3],lo:[0.;3],hi:[0.;3],local:false,falloff:0.}}
}
pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::NativeCage||r.kind==Kind::None{return Err(CapabilityError::UnsupportedOperation(operation));}
    use RepresentationKind::*;
    let cad=matches!(input,Point|Curve|Polycurve|TrimmedFace|Shell|Solid|Brep);
    let allowed=match r.role{Role::Geometry|Role::BindingStorage=>cad||input==Mesh,
        Role::NativeControl=>cad,Role::RetainedControl=>input==Retained};
    if !allowed{return Err(CapabilityError::UnsupportedRepresentation(input));}
    let unsupported=||Err(CapabilityError::UnsupportedOptions);
    if !r.falloff.is_finite()||r.falloff<0.
        ||r.lo.iter().chain(&r.hi).any(|x|!x.is_finite())||(0..3).any(|a|r.lo[a]>r.hi[a]){return unsupported();}
    if r.explicit_bounds&&(r.kind!=Kind::Create||r.role!=Role::Geometry){return unsupported();}
    match r.kind{
        Kind::Create=>{
            if r.role!=Role::Geometry||r.has_control||r.retained||r.local||r.falloff!=0.
                ||crate::cage::box_dimensions(r.counts,r.degrees).is_err(){return unsupported();}
            if r.stage==Stage::Construct&&crate::cage::box_reference(r.lo,r.hi).is_err(){return unsupported();}
        }
        Kind::Capture=>{
            if r.role==Role::BindingStorage||(r.role==Role::RetainedControl&&!r.retained){return unsupported();}
        }
        Kind::Release=>{
            if !matches!(r.role,Role::Geometry|Role::BindingStorage)||r.has_control||r.retained||r.local||r.falloff!=0.{return unsupported();}
        }
        Kind::None=>unreachable!(),
    }
    match r.stage{
        Stage::Declare=>if r.phase!=Phase::Inputs||r.role!=Role::Geometry{return unsupported();},
        Stage::Capture=>{
            let phase=if matches!(r.role,Role::NativeControl|Role::RetainedControl){Phase::Control}else{Phase::Inputs};
            if r.phase!=phase{return unsupported();}
        }
        Stage::Construct=>{
            if !matches!(r.phase,Phase::Options|Phase::Ready)
                ||(r.input_count==0&&!r.explicit_bounds&&r.role!=Role::RetainedControl)
                ||(r.kind==Kind::Capture&&!r.has_control){return unsupported();}
        }
        Stage::Recompute=>{
            if r.kind!=Kind::Capture||r.phase!=Phase::Ready||r.role!=Role::Geometry
                ||r.input_count==0||!r.has_control||r.retained{return unsupported();}
        }
    }
    let id=match r.kind{Kind::Create=>"OM9-TRANSFORM-041",Kind::Capture=>"OM9-TRANSFORM-020",Kind::Release=>"OM9-TRANSFORM-022",Kind::None=>unreachable!()};
    Ok((Backend::NativeCageAdapters,&[OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
        OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts],id))
}
