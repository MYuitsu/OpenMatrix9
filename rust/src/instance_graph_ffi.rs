// SPDX-License-Identifier: LGPL-2.1-or-later
//! Live initialized immutable one-allocation input; writable nonoverlapping
//! output. No native pointer, allocator ownership or panic crosses this ABI.
use crate::instance_graph::{self as model,Node};
use crate::support_snapshot_ffi::{JsonSummary,output_valid,copy_json};
#[unsafe(no_mangle)]
pub extern "C" fn om9_instance_graph_counts(nodes:usize,depth:usize)->u32 {u32::from(!model::counts_allowed(nodes,depth))}
/// # Safety
/// Input/output must satisfy the caller storage and lifetime obligations above.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_instance_graph_json(nodes:*const Node,count:usize,output:*mut u8,capacity:usize)->JsonSummary {
    let error=|status|JsonSummary{status,..Default::default()};
    if !output_valid(output,capacity){return error(1002);}
    if !model::counts_allowed(count,1)||nodes.is_null()||(nodes as usize)%std::mem::align_of::<Node>()!=0
        ||count.checked_mul(std::mem::size_of::<Node>()).is_none_or(|n|n>isize::MAX as usize||(nodes as usize).checked_add(n).is_none()) {
        return error(6000);
    }
    // SAFETY: validated envelope; caller still guarantees live initialized immutable nodes.
    let owned=match model::Graph::capture(unsafe{std::slice::from_raw_parts(nodes,count)}){Ok(v)=>v,Err(_)=>return error(6001)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
