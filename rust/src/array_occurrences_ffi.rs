// SPDX-License-Identifier: LGPL-2.1-or-later
//! Input stays live, initialized, immutable and within one allocation for the
//! call. Output is live writable and nonoverlapping. No retained native pointer.
use crate::array_occurrences::{self as model,Occurrence};
use crate::support_snapshot_ffi::{JsonSummary,output_valid,copy_json};
#[unsafe(no_mangle)]
pub extern "C" fn om9_array_occurrences_counts(total:usize,count:usize)->u32{u32::from(!model::counts_allowed(total,count))}
/// # Safety
/// Caller provides live immutable initialized rows and nonoverlapping writable output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_array_occurrences_json(rows:*const Occurrence,count:usize,total:usize,output:*mut u8,capacity:usize)->JsonSummary{
 let error=|status|JsonSummary{status,..Default::default()};
 if !output_valid(output,capacity){return error(1002);}
 if !model::counts_allowed(total,count)||rows.is_null()||(rows as usize)%std::mem::align_of::<Occurrence>()!=0
  ||count.checked_mul(std::mem::size_of::<Occurrence>()).is_none_or(|n|n>isize::MAX as usize||(rows as usize).checked_add(n).is_none()) {return error(6100);}
 // SAFETY: checked envelope; initialized live immutable storage remains a caller obligation.
 let owned=match model::Snapshot::capture(total,unsafe{std::slice::from_raw_parts(rows,count)}){Ok(v)=>v,Err(_)=>return error(6101)};
 unsafe{copy_json(owned.json(),output,capacity)}
}
