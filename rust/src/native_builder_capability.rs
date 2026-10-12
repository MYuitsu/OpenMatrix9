// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent existing affine-template requests; native geometry stays native.
use crate::geometry_contract::*;
pub use crate::native_surface_capability::Stage;

#[derive(Clone,Debug,PartialEq)]
pub struct Request {
    pub stage:Stage,
    pub feature_id:String,
    pub recipe:String,
    pub scale_to_gem:bool,
    pub record_count:usize,
    pub template_count:usize,
    pub target_count:usize,
    pub dimensions:Option<([f64;3],[f64;3])>,
}

fn counts_valid(records:usize,templates:usize,targets:usize)->bool{
    targets<=1024 && crate::builder_history::valid_replay_budget(records,templates,targets)
        && records.checked_mul(1024).is_some_and(|n|templates<=n)
}
/// Scalar-only traversal preflight; the full request still requires admission.
#[unsafe(no_mangle)]
pub extern "C" fn om9_native_builder_counts(records:usize,templates:usize,targets:usize)->bool{
    counts_valid(records,templates,targets)
}
fn readable_header<T>(p:*const T,count:usize)->bool{
    if count==0{return true;}
    let Some(bytes)=count.checked_mul(std::mem::size_of::<T>()) else{return false;};
    let address=p as usize;
    !p.is_null() && address%std::mem::align_of::<T>()==0
        && bytes<=isize::MAX as usize && address.checked_add(bytes).is_some()
}

/// Synchronous status query; Rust retains no native pointer or owner.
///
/// # Safety
/// Nonempty byte buffers and optional six-double dimensions must be initialized,
/// immutable and live in their respective single caller-owned allocations for
/// this call. Headers are checked before reads; they cannot prove provenance or
/// initialization. Caller frees its buffers. Rust-owned requests and parser
/// allocations remain on this side. Unwinds return103; release panic=abort and
/// native dependencies are not certified memory-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_native_builder_capability(
    representation:u32,stage:u32,id:*const u8,id_len:usize,
    raw:*const u8,raw_len:usize,records:usize,templates:usize,targets:usize,
    flags:u32,dimensions:*const f64,dimension_count:usize)->u32{
    std::panic::catch_unwind(||{
        let input=match representation{1=>RepresentationKind::Point,2=>RepresentationKind::Curve,
            3=>RepresentationKind::Polycurve,4=>RepresentationKind::TrimmedFace,5=>RepresentationKind::Shell,
            6=>RepresentationKind::Solid,7=>RepresentationKind::Brep,8=>RepresentationKind::Mesh,
            9=>RepresentationKind::PointCloud,10=>RepresentationKind::Instance,11=>RepresentationKind::Group,
            12=>RepresentationKind::Annotation,13=>RepresentationKind::Retained,_=>return 101};
        let stage=match stage{1=>Stage::Declare,2=>Stage::Capture,3=>Stage::Construct,_=>return 102};
        if id_len==0||id_len>24||raw_len==0||raw_len>1_048_576||flags&!1!=0
            ||!counts_valid(records,templates,targets)
            ||!matches!(dimension_count,0|6)
            ||!readable_header(id,id_len)||!readable_header(raw,raw_len)
            ||!readable_header(dimensions,dimension_count){return 102;}
        let (Ok(id),Ok(raw))=(std::str::from_utf8(unsafe{std::slice::from_raw_parts(id,id_len)}),
            std::str::from_utf8(unsafe{std::slice::from_raw_parts(raw,raw_len)})) else{return 102;};
        let dimensions=if dimension_count==6{
            let values=unsafe{std::slice::from_raw_parts(dimensions,6)};
            Some(([values[0],values[1],values[2]],[values[3],values[4],values[5]]))
        }else{None};
        let request=Request{stage,feature_id:id.into(),recipe:raw.into(),scale_to_gem:flags&1!=0,
            record_count:records,template_count:templates,target_count:targets,dimensions};
        match operation_capability(OperationId::NativeBuilder,input,OperationOptions::NativeBuilder(request)){
            Ok(c) if c.backend==Backend::OcctEdit=>4,
            Ok(_)|Err(CapabilityError::UnsupportedOperation(_))=>100,
            Err(CapabilityError::UnsupportedRepresentation(_))=>101,
            Err(CapabilityError::UnsupportedOptions)=>102,
        }
    }).unwrap_or(103)
}

pub(crate) fn policy(operation:OperationId,input:RepresentationKind,r:&Request)
    ->Result<(Backend,&'static [OperationConstraint],&'static str),CapabilityError>{
    if operation!=OperationId::NativeBuilder{return Err(CapabilityError::UnsupportedOperation(operation));}
    // Existing Builder API consumes native Part shapes. A surface support or
    // retained source payload is not implicitly converted into a Part template.
    if !matches!(input,RepresentationKind::Point|RepresentationKind::Curve|RepresentationKind::Polycurve|
        RepresentationKind::TrimmedFace|RepresentationKind::Shell|RepresentationKind::Solid|RepresentationKind::Brep){
        return Err(CapabilityError::UnsupportedRepresentation(input));
    }
    if !crate::builder_history::valid_feature_id(&r.feature_id)
        ||!counts_valid(r.record_count,r.template_count,r.target_count){
        return Err(CapabilityError::UnsupportedOptions);
    }
    let recipe=crate::builder_history::validate_recipe(&r.recipe)
        .map_err(|_|CapabilityError::UnsupportedOptions)?;
    if let Some((initial,current))=r.dimensions{
        crate::builder_history::affine_plan(&recipe,initial,current,r.scale_to_gem)
            .map_err(|_|CapabilityError::UnsupportedOptions)?;
    }else if r.stage==Stage::Construct{return Err(CapabilityError::UnsupportedOptions);}
    Ok((Backend::OcctEdit,&[OperationConstraint::ValidateSourceStamp,
        OperationConstraint::ValidateNativeInput,OperationConstraint::PrivateOwnedInput,
        OperationConstraint::ReportConversionFacts],"OM9-HISTORY-001"))
}
