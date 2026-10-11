// SPDX-License-Identifier: LGPL-2.1-or-later
//! Inputs must be initialized, immutable, live, aligned single allocations.
//! Output must be writable and nonoverlapping. No native pointer escapes.
use crate::support_snapshot_ffi::JsonSummary;
use crate::support_snapshot_ffi::{output_valid,copy_json};
use crate::source_attributes::{self as model,Input,Material,SourceAttributes};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Text {pub bytes:*const u8,pub len:usize}
#[repr(C)]
#[derive(Clone,Copy)]
pub struct NativeMaterial {pub colors:[[f64;4];4],pub shininess:f64,pub transparency:f64,pub text:[Text;3]}
#[repr(C)]
#[derive(Clone,Copy)]
pub struct AttributesInput {pub text:[Text;6],pub known:u32,pub values:u32,pub materials:*const NativeMaterial,pub material_count:usize}
fn error(status:u32)->JsonSummary {JsonSummary{status,..Default::default()}}
fn envelope<T>(p:*const T,count:usize)->bool {
    if count==0{return true;}
    !p.is_null() && (p as usize)%std::mem::align_of::<T>()==0 && count.checked_mul(std::mem::size_of::<T>())
        .is_some_and(|n|n<=isize::MAX as usize&&(p as usize).checked_add(n).is_some())
}
fn text_envelope(t:Text)->bool {t.len<=model::MAX_TEXT_BYTES&&envelope(t.bytes,t.len)}
unsafe fn text<'a>(t:Text)->Result<&'a str,()> {
    let bytes=if t.len==0 {&[]}else{unsafe{std::slice::from_raw_parts(t.bytes,t.len)}};
    std::str::from_utf8(bytes).map_err(|_|())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_source_attribute_counts(materials:usize,text_bytes:usize)->u32 {
    u32::from(!model::counts_allowed(materials,text_bytes))
}
/// # Safety
/// Caller supplies the live initialized input and writable output contract above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_source_attributes_json(i:AttributesInput,output:*mut u8,capacity:usize)->JsonSummary {
    if !output_valid(output,capacity){return error(1002);}
    if !model::flags_valid(i.known,i.values){return error(5000);}
    if !model::counts_allowed(i.material_count,0)||!envelope(i.materials,i.material_count){return error(5001);}
    // SAFETY: bounded/aligned envelope; caller guarantees initialized immutable storage.
    let materials=if i.material_count==0{&[]}else{unsafe{std::slice::from_raw_parts(i.materials,i.material_count)}};
    let mut total=0usize;
    for t in i.text.iter().copied().chain(materials.iter().flat_map(|m|m.text)) {
        if !text_envelope(t){return error(5002);}
        total=match total.checked_add(t.len){Some(v)=>v,None=>return error(5001)};
        if !model::counts_allowed(i.material_count,total){return error(5001);}
    }
    // No text buffer is borrowed until every envelope and aggregate budget passes.
    let mut root=["";6];
    for (out,t) in root.iter_mut().zip(i.text){*out=match unsafe{text(t)}{Ok(s)=>s,Err(())=>return error(5003)};}
    let mut borrowed=Vec::with_capacity(materials.len());
    for m in materials {
        let mut fields=["";3];
        for (out,t) in fields.iter_mut().zip(m.text){*out=match unsafe{text(t)}{Ok(s)=>s,Err(())=>return error(5003)};}
        borrowed.push(Material{colors:m.colors,shininess:m.shininess,transparency:m.transparency,text:fields});
    }
    let owned=match SourceAttributes::capture(Input{text:root,known:i.known,values:i.values,materials:&borrowed}){
        Ok(s)=>s,Err(_)=>return error(5004)
    };
    unsafe{copy_json(owned.json(),output,capacity)}
}
