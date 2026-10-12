// SPDX-License-Identifier: LGPL-2.1-or-later
//! Value-only ABI; native source lifetime remains the caller's responsibility.
use crate::geometry_frame::Frame;
#[repr(C)]
#[derive(Clone,Copy)]
pub struct FrameInput {pub parent:[f64;16],pub path:[f64;16],pub source:[f64;16],pub location:[f64;16]}
#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct FrameSummary {pub status:u32,pub reserved:u32,pub local_to_world:[f64;16],pub orientation_sign:i32,pub reserved2:u32}
#[unsafe(no_mangle)]
pub extern "C" fn om9_geometry_frame(i:FrameInput)->FrameSummary {
    match Frame::compose(i.parent,i.path,i.source,i.location) {
        Ok(f)=>FrameSummary{status:0,local_to_world:f.matrix(),orientation_sign:f.orientation_sign(),..Default::default()},
        Err(e)=>FrameSummary{status:1+e as u32,..Default::default()}
    }
}
