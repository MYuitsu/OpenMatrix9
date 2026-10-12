// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::reference_path::{self as model,Plan,Step};
#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct Summary{pub status:u32,pub count:u32,pub selector_start:u32,pub selector_len:u32}
#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct AliasSummary{pub status:u32,pub matched:u32,pub index:u32,pub reserved:u32}
/// # Safety
/// Both byte spans stay initialized, immutable and live for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_reference_array_index(owner:*const u8,owner_len:usize,token:*const u8,token_len:usize,allow_collapsed:u32)->AliasSummary{
 let error=|status|AliasSummary{status,..Default::default()};
 if allow_collapsed>1{return error(6201);}
 for (p,n) in [(owner,owner_len),(token,token_len)]{
  if n>model::MAX_BYTES||(n>0&&(p.is_null()||(p as usize).checked_add(n).is_none())){return error(6200);}
 }
 // SAFETY: both bounded envelopes passed; the caller guarantees initialized,
 // live immutable single-allocation buffers. Zero lengths borrow no pointer.
 let owner=if owner_len==0{&[]}else{unsafe{std::slice::from_raw_parts(owner,owner_len)}};
 let token=if token_len==0{&[]}else{unsafe{std::slice::from_raw_parts(token,token_len)}};
 let (Ok(owner),Ok(token))=(std::str::from_utf8(owner),std::str::from_utf8(token)) else{return error(6201)};
 match model::array_index(owner,token,allow_collapsed==1){Ok(Some(index))=>AliasSummary{status:0,matched:1,index,reserved:0},Ok(None)=>AliasSummary::default(),Err(_)=>error(6201)}
}
/// # Safety
/// Input stays initialized, immutable and live; output is live writable and
/// nonoverlapping for this synchronous call. Neither pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_reference_path_parse(input:*const u8,len:usize,output:*mut Step,capacity:usize)->Summary{
 let error=|status|Summary{status,..Default::default()};
 if capacity>model::MAX_STEPS||(capacity>0&&(output.is_null()||(output as usize)%std::mem::align_of::<Step>()!=0||capacity.checked_mul(std::mem::size_of::<Step>()).is_none_or(|n|n>isize::MAX as usize||(output as usize).checked_add(n).is_none()))){return error(1002);}
 if len>model::MAX_BYTES||(len>0&&(input.is_null()||(input as usize).checked_add(len).is_none())){return error(6200);}
 // SAFETY: checked bounded envelope; caller guarantees live initialized,
 // immutable bytes within one allocation. Zero input borrows no pointer.
 let bytes=if len==0{&[]}else{unsafe{std::slice::from_raw_parts(input,len)}};
 let plan=match Plan::parse(bytes){Ok(p)=>p,Err(_)=>return error(6201)};
 let mut summary=Summary{status:0,count:plan.steps().len() as u32,selector_start:plan.selector_start() as u32,selector_len:plan.selector().len() as u32};
 if capacity==0{return summary;}
 if capacity<plan.steps().len(){summary.status=1001;return summary;}
 if !plan.steps().is_empty(){
  // SAFETY: checked output envelope/capacity; caller supplies live writable
  // output, nonoverlapping with the input. Owned rows are never retained.
  unsafe{std::ptr::copy_nonoverlapping(plan.steps().as_ptr(),output,plan.steps().len())};
 }
 summary
}
