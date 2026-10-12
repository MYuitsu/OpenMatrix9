// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned bounded native object-reference syntax, independent of document APIs.
#[repr(C)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Step{pub start:u32,pub len:u32,pub kind:u32,pub index:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Bytes,Utf8,Token,Count,Index}
pub const MAX_BYTES:usize=4096;
pub const MAX_STEPS:usize=64;
pub fn array_index(owner:&str,token:&str,allow_collapsed:bool)->Result<Option<u32>,Error>{
 if owner.len()>MAX_BYTES||token.len()>MAX_BYTES||owner.contains('\0')||token.contains('\0'){return Err(Error::Bytes);}
 if owner.is_empty(){return Err(Error::Token);}
 let numeric=if !token.is_empty()&&token.bytes().all(|c|c.is_ascii_digit()){Some(token)}
 else if allow_collapsed {
  let token=token.strip_prefix('$').unwrap_or(token);
  token.strip_prefix(owner).and_then(|s|s.strip_prefix('_')).and_then(|s|s.rsplit_once('i'))
   .map(|(_,digits)|digits).filter(|s|!s.is_empty()&&s.bytes().all(|c|c.is_ascii_digit()))
 }else{None};
 match numeric{None=>Ok(None),Some(digits)=>{
  let index=digits.parse::<u32>().map_err(|_|Error::Index)?;
  if index>=crate::array_occurrences::MAX_ELEMENTS as u32{return Err(Error::Index);}Ok(Some(index))
 }}
}
pub struct Plan{path:String,steps:Vec<Step>,selector_start:usize}
fn token_step(start:usize,token:&str)->Result<Step,Error>{
 if token.is_empty()||token=="$"{return Err(Error::Token);}
 let (kind,index)=if token.bytes().all(|c|c.is_ascii_digit()){
  let index=token.parse::<u32>().map_err(|_|Error::Index)?;
  if index>=crate::array_occurrences::MAX_ELEMENTS as u32{return Err(Error::Index);}(2,index)
 }else{(u32::from(token.starts_with('$')),0)};
 Ok(Step{start:start as u32,len:token.len() as u32,kind,index})
}
impl Plan{
 pub fn parse(bytes:&[u8])->Result<Self,Error>{
  if bytes.len()>MAX_BYTES||bytes.contains(&0){return Err(Error::Bytes);}
  let path=std::str::from_utf8(bytes).map_err(|_|Error::Utf8)?;
  let mut start=0;let mut count=0;
  // Validate every token and the aggregate step count before cloning or
  // reserving owned text/rows. UTF-8 dot boundaries are safe byte offsets.
  for (end,_) in path.match_indices('.'){
   count+=1;if count>MAX_STEPS{return Err(Error::Count);}
   token_step(start,&path[start..end])?;start=end+1;
  }
  let selector_start=start;let mut steps=Vec::with_capacity(count);start=0;
  for (end,_) in path.match_indices('.'){
   steps.push(token_step(start,&path[start..end])?);start=end+1;
  }
  Ok(Self{path:path.to_owned(),steps,selector_start})
 }
 pub fn path(&self)->&str{&self.path}
 pub fn steps(&self)->&[Step]{&self.steps}
 pub fn selector(&self)->&str{&self.path[self.selector_start..]}
 pub fn selector_start(&self)->usize{self.selector_start}
}
