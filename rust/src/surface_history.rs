// SPDX-License-Identifier: LGPL-2.1-or-later
//! Independent bounded saved Surface History requests, before native source capture.
use crate::builder_history::{Json,parse_document_at_depth};
use crate::native_surface_capability::{Request,Stage};
use crate::geometry_contract::{operation_capability,OperationId,OperationOptions,RepresentationKind};
use crate::surface::Kind;
use std::collections::BTreeMap;
const BYTES:usize=1_048_576;
const GROUPS:usize=256;
const LINKS:usize=GROUPS*4096;
const SLASHES:usize=64;

#[derive(Clone,Debug,PartialEq)]
pub struct SavedInput {pub count:usize,pub chain:bool,pub closed:bool,pub reverse:bool,pub seam:f64}
#[derive(Clone,Debug,PartialEq)]
pub struct SavedRequest {pub request:Request,pub inputs:Vec<SavedInput>}
fn object(value:&Json)->Result<&BTreeMap<String,Json>,&'static str>{
    match value {Json::Object(m)=>Ok(m),_=>Err("saved object required")}
}
fn number(m:&BTreeMap<String,Json>,key:&str,lo:f64,hi:f64)->Result<f64,&'static str>{
    match m.get(key){Some(Json::Number(n)) if n.is_finite()&&*n>=lo&&*n<=hi=>Ok(*n),_=>Err("invalid saved number")}
}
fn integer(m:&BTreeMap<String,Json>,key:&str,lo:u32,hi:u32)->Result<u32,&'static str>{
    let n=number(m,key,lo as f64,hi as f64)?;
    if n.fract()!=0. {return Err("invalid saved integer");}Ok(n as u32)
}
fn boolean(m:&BTreeMap<String,Json>,key:&str)->Result<bool,&'static str>{
    match m.get(key){Some(Json::Boolean(b))=>Ok(*b),_=>Err("invalid saved boolean")}
}
fn array<'a>(m:&'a BTreeMap<String,Json>,key:&str,limit:usize)->Result<&'a [Json],&'static str>{
    match m.get(key){Some(Json::Array(a)) if a.len()<=limit=>Ok(a),_=>Err("invalid saved array")}
}
/// Only scalar/group metadata is read. The native adapter still verifies actual
/// source ownership, geometry, rail closure and source stamps before its solver.
/// Unknown bounded fields remain compatible; ambiguous JSON is rejected.
pub fn decode_saved(raw:&str,source_count:usize)->Result<SavedRequest,&'static str>{
    let saved=decode_metadata(raw,source_count)?;
    if !saved_allowed(&saved.request){return Err("unsupported saved Surface request");}
    Ok(saved)
}
fn saved_allowed(r:&Request)->bool {
    operation_capability(OperationId::EditSurface,RepresentationKind::Polycurve,OperationOptions::NativeSurface(r.clone())).is_ok()
}
fn decode_metadata(raw:&str,source_count:usize)->Result<SavedRequest,&'static str>{
    if source_count>LINKS {return Err("source reference budget");}
    let value=parse_document_at_depth(raw,BYTES,16384,32)?;let root=object(&value)?;
    integer(root,"version",1,1)?;
    let kind=match integer(root,"kind",1,3)?{1=>Kind::Sweep1,2=>Kind::Sweep2,_=>Kind::Loft};
    let rows=array(root,"inputs",GROUPS)?;
    let mut r=Request::new(kind,Stage::Saved,rows.len());
    r.style=integer(root,"style",0,5)?;
    r.section_mode=integer(root,"sectionMode",0,2)?;r.point_count=integer(root,"pointCount",2,256)?;
    r.tolerance=number(root,"tolerance",1e-7,1e6)?;
    r.continuity_a=integer(root,"continuityA",0,2)?;r.continuity_b=integer(root,"continuityB",0,2)?;
    r.frenet=boolean(root,"frenet")?;r.closed=boolean(root,"closed")?;
    r.maintain_height=boolean(root,"maintainHeight")?;r.preview=boolean(root,"preview")?;
    r.history=boolean(root,"history")?;r.history_command=boolean(root,"historyCommand")?;
    r.match_start=boolean(root,"matchStart")?;r.match_end=boolean(root,"matchEnd")?;
    for slash in array(root,"slashes",SLASHES)? {
        let Json::Array(pair)=slash else{return Err("invalid saved slash");};
        if pair.len()!=2{return Err("invalid saved slash");}
        let scalar=|v:&Json|match v {Json::Number(n) if n.is_finite()&&(0.0..=1.0).contains(n)=>Ok(*n),_=>Err("invalid saved slash fraction")};
        r.slashes.push((scalar(&pair[0])?,scalar(&pair[1])?));
    }
    let mut inputs=Vec::with_capacity(rows.len());let mut count=0usize;
    for row in rows {
        let m=object(row)?;
        let input=SavedInput{count:integer(m,"count",1,4096)? as usize,chain:boolean(m,"chain")?,
            closed:boolean(m,"closed")?,reverse:boolean(m,"reverse")?,seam:number(m,"seam",0.,1.)?};
        if (!input.chain&&input.count!=1)||input.seam>=1. {return Err("invalid saved input grouping or seam");}
        count=count.checked_add(input.count).ok_or("source reference overflow")?;
        if count>source_count{return Err("source reference mismatch");}inputs.push(input);
    }
    if count!=source_count {return Err("source reference mismatch");}
    for (i,input) in inputs.iter().take(kind.rails()).enumerate(){r.rail_closed[i]=input.closed;}
    Ok(SavedRequest{request:r,inputs})
}

#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct NativeSavedInput {pub count:u32,pub flags:u32,pub seam:f64}
/// Fixed caller-owned storage; no native or Rust pointers survive the call.
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct NativeSavedSurface {
    pub kind:u32,pub style:u32,pub section_mode:u32,pub point_count:u32,
    pub continuity_a:u32,pub continuity_b:u32,pub flags:u32,pub group_count:u32,
    pub tolerance:f64,pub slash_count:usize,
    pub inputs:[NativeSavedInput;GROUPS],pub slashes:[[f64;2];SLASHES],
}
impl Default for NativeSavedSurface {
    fn default()->Self{Self{kind:0,style:0,section_mode:0,point_count:0,continuity_a:0,continuity_b:0,
        flags:0,group_count:0,tolerance:0.,slash_count:0,inputs:[NativeSavedInput::default();GROUPS],slashes:[[0.;2];SLASHES]}}
}
/// # Safety
/// For a bounded nonempty length, raw must name live initialized immutable bytes
/// for this synchronous call. out must name one exclusive aligned writable
/// NativeSavedSurface, disjoint from raw. Address checks cannot prove allocation
/// lifetime. Failure leaves out untouched; successful output contains values only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_surface_history_decode(raw:*const u8,length:usize,source_count:usize,out:*mut NativeSavedSurface)->u32{
    std::panic::catch_unwind(||{
        if length==0||length>BYTES||source_count>LINKS||raw.is_null()||out.is_null()
            ||(raw as usize).checked_add(length).is_none()
            ||out as usize%std::mem::align_of::<NativeSavedSurface>()!=0
            ||(out as usize).checked_add(std::mem::size_of::<NativeSavedSurface>()).is_none(){return 102;}
        // SAFETY: caller guarantees both disjoint live allocations for the call.
        let bytes=unsafe{std::slice::from_raw_parts(raw,length)};
        let Ok(text)=std::str::from_utf8(bytes) else{return 102;};
        let Ok(saved)=decode_metadata(text,source_count) else{return 102;};
        // Valid complete metadata can identify the concrete operation even when
        // the shared options/cardinality policy rejects it. No output is written.
        if !saved_allowed(&saved.request){return 200+saved.request.kind as u32;}
        let r=saved.request;let mut output=NativeSavedSurface{kind:r.kind as u32,style:r.style,section_mode:r.section_mode,
            point_count:r.point_count,continuity_a:r.continuity_a,continuity_b:r.continuity_b,
            tolerance:r.tolerance,group_count:saved.inputs.len() as u32,slash_count:r.slashes.len(),..NativeSavedSurface::default()};
        output.flags=(r.frenet as u32)|((r.closed as u32)<<1)|((r.maintain_height as u32)<<2)
            |((r.match_start as u32)<<3)|((r.match_end as u32)<<4)|((r.history as u32)<<5)
            |((r.history_command as u32)<<6)|((r.preview as u32)<<7);
        for (target,input) in output.inputs.iter_mut().zip(saved.inputs){*target=NativeSavedInput{count:input.count as u32,
            flags:(input.chain as u32)|((input.closed as u32)<<1)|((input.reverse as u32)<<2),seam:input.seam};}
        for (target,(a,b)) in output.slashes.iter_mut().zip(r.slashes){*target=[a,b];}
        // SAFETY: bounded output owner was checked above and guaranteed by caller.
        unsafe{out.write(output)};0
    }).unwrap_or(103)
}
