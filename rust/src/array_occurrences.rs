// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned metadata for bounded revision-scoped native array occurrences.
use crate::core_objects::Stamp;
use crate::geometry_frame::{Frame,IDENTITY};
use crate::support_snapshot::{Json,SerializedLimit};
use std::fmt::Write;
pub const MAX_ELEMENTS:usize=(crate::instance_graph::MAX_NODES-1)/2;
pub const MAX_RETAINED_BYTES:usize=1024*1024;
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct Occurrence{pub index:u64,pub element_present:u32,pub reserved:u32,pub element:Stamp,pub definition:Stamp,pub object_to_world:[f64;16]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Count,Index,Flag,Identity,Frame}
#[derive(Debug)]
pub struct Snapshot{total:usize,rows:Vec<Occurrence>}
pub fn counts_allowed(total:usize,count:usize)->bool{
 total>0&&total<=MAX_ELEMENTS&&count>0&&count<=total&&count.checked_mul(std::mem::size_of::<Occurrence>()).and_then(|n|n.checked_add(std::mem::size_of::<Snapshot>())).is_some_and(|n|n<=MAX_RETAINED_BYTES)
}
fn values(s:Stamp)->[u64;5]{[s.document,s.object,s.generation,s.revision,s.epoch]}
fn stamp(j:&mut Json,s:Stamp)->std::fmt::Result{
 write!(j,"{{\"document\":\"{}\",\"object\":\"{}\",\"generation\":\"{}\",\"revision\":\"{}\",\"epoch\":\"{}\"}}",s.document,s.object,s.generation,s.revision,s.epoch)
}
impl Snapshot{
 pub fn capture(total:usize,rows:&[Occurrence])->Result<Self,Error>{
  if !counts_allowed(total,rows.len()){return Err(Error::Count);}
  let mut previous=None;
  for row in rows{
   if row.index>=total as u64||previous.is_some_and(|p|row.index<=p){return Err(Error::Index);}previous=Some(row.index);
   if row.reserved!=0||row.element_present>1{return Err(Error::Flag);}
   if values(row.definition).contains(&0)||(row.element_present==1&&values(row.element).contains(&0))||(row.element_present==0&&values(row.element)!=[0;5]){return Err(Error::Identity);}
   Frame::compose(IDENTITY,row.object_to_world,IDENTITY,IDENTITY).map_err(|_|Error::Frame)?;
  }
  Ok(Self{total,rows:rows.to_vec()})
 }
 pub fn json(&self)->Result<String,SerializedLimit>{
  let mut j=Json(String::new());let result=(||{
   write!(j,"{{\"schema_version\":1,\"kind\":\"array_occurrences\",\"identity_scope\":\"revision_scoped_array_owner\",\"total_count\":{},\"elements\":[",self.total)?;
   for (i,row) in self.rows.iter().enumerate(){if i!=0{j.write_char(',')?;}
    write!(j,"{{\"array_index\":{},\"native_path\":\"{}.\",\"native_element_ref\":",row.index,row.index)?;
    if row.element_present==0{j.write_str("null")?;}else{stamp(&mut j,row.element)?;}
    j.write_str(",\"definition_ref\":")?;stamp(&mut j,row.definition)?;
    j.write_str(",\"definition_object_to_world\":[")?;
    for r in 0..4{if r!=0{j.write_char(',')?;}crate::support_snapshot::numbers(&mut j,&row.object_to_world[r*4..r*4+4])?;}
    let frame=Frame::compose(IDENTITY,row.object_to_world,IDENTITY,IDENTITY).map_err(|_|std::fmt::Error)?;
    write!(j,"],\"world_orientation_sign\":{}}}",frame.orientation_sign())?;
   }
   j.write_str("]}")
  })();result.map(|_|j.0).map_err(|_|SerializedLimit)
 }
}
