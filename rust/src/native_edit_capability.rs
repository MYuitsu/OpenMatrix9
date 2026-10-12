// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared policy for the existing native Edit consumers; no native owners.
use crate::edit::Kind;
use crate::geometry_contract::*;
pub(crate) fn kind(code:u32)->Option<Kind>{Some(match code {
 1=>Kind::Join,2=>Kind::Explode,3=>Kind::Trim,4=>Kind::Difference,
 5=>Kind::Intersection,6=>Kind::Union,7=>Kind::TwoObjects,_=>return None,
})}
pub(crate) fn policy(operation:OperationId,input:RepresentationKind,code:u32,flags:u32,mode:u32)
 ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
 let k=kind(code).ok_or(CapabilityError::UnsupportedOperation(operation))?;
 if (code>=4&&operation!=OperationId::Boolean)||(code<4&&operation!=OperationId::NativeEdit){return Err(CapabilityError::UnsupportedOptions);}
 // Existing fixed deletion for Join/Explode/Trim, selectable deletion for
 // Booleans, Trim's two intersection switches and five TwoObjects modes.
 let valid_flags=match k {Kind::Trim=>(4..=7).contains(&flags),Kind::Join|Kind::Explode=>flags==4,_=>matches!(flags,0|4)};
 if !valid_flags||if k==Kind::TwoObjects{mode>4}else{mode!=0}{return Err(CapabilityError::UnsupportedOptions);}
 use RepresentationKind::*;
 let allowed=match k {
  Kind::Join|Kind::Trim=>matches!(input,Curve|Polycurve|TrimmedFace|Shell),
  Kind::Explode=>matches!(input,Polycurve|Shell|Solid|Brep|Mesh|Instance|Annotation|Retained),
  _=>matches!(input,TrimmedFace|Shell|Solid|Brep|Mesh),
 };
 if !allowed{return Err(CapabilityError::UnsupportedRepresentation(input));}
 // Brep compounds still need the existing native homogeneous solid/surface
 // check. Special Explode sources still need their known native adapter.
 let backend=if k==Kind::Explode{Backend::NativeExplodeAdapters}else if input==Mesh{Backend::MeshOcctEdit}else{Backend::OcctEdit};
 Ok((backend,&[OperationConstraint::ValidateSourceStamp,OperationConstraint::ValidateNativeInput,
  OperationConstraint::PrivateOwnedInput,OperationConstraint::ReportConversionFacts],k.feature()))
}
fn input(code:u32)->RepresentationKind{use RepresentationKind::*;match code {
 1=>Point,2=>Curve,3=>Polycurve,4=>TrimmedFace,5=>Shell,6=>Solid,7=>Brep,
 8=>Mesh,9=>PointCloud,10=>Instance,11=>Group,12=>Annotation,13=>Retained,_=>Unknown,
}}
/// Value-only synchronous ABI, no pointers or owners retained. Return values
/// share the existing registry convention: backend4/5/6, errors100/101/102.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_edit_capability(kind:u32,representation:u32,flags:u32,mode:u32)->u32 {
 std::panic::catch_unwind(||{
  let operation=if kind>=4{OperationId::Boolean}else{OperationId::NativeEdit};
  match operation_capability(operation,input(representation),OperationOptions::NativeEdit{kind,flags,mode}) {
   Ok(c)=>match c.backend{Backend::OcctEdit=>4,Backend::MeshOcctEdit=>5,Backend::NativeExplodeAdapters=>6,_=>100},
   Err(CapabilityError::UnsupportedOperation(_))=>100,
   Err(CapabilityError::UnsupportedRepresentation(_))=>101,
   Err(CapabilityError::UnsupportedOptions)=>102,
  }
 }).unwrap_or(103)
}
/// Command declaration before actual input exists. Enumerate supported types;
/// Unknown is never authorized as concrete geometry. Host context gates remain.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_edit_declared(kind:u32,flags:u32,mode:u32)->bool {
 (1..=13).any(|representation|om9_native_edit_capability(kind,representation,flags,mode)<100)
}
