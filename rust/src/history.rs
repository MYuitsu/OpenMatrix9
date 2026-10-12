// SPDX-License-Identifier: LGPL-2.1-or-later
//! Persistent links live in native document properties; Rust owns graph and policy semantics.
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr,c_char};
pub const ICONS:[&str;5]=["RhinoHistoryON","InfoSettingsGVClearHistory","GVHistoryRecordON","GVHistoryUpdateON","gvJoinHistory"];
pub fn command(icon:&str)->Option<&'static str>{match icon{"RhinoHistoryON"=>Some("History"),"InfoSettingsGVClearHistory"=>Some("MatrixClearObjectHistory"),"GVHistoryRecordON"=>Some("MatrixHistoryRecord"),"GVHistoryUpdateON"=>Some("MatrixHistoryUpdate"),"gvJoinHistory"=>Some("OM9_gvJoinHistory"),_=>None}}
pub fn caption(icon:&str)->Option<&'static str>{match icon{"RhinoHistoryON"=>Some("History"),"InfoSettingsGVClearHistory"=>Some("Clear Object History"),"GVHistoryRecordON"=>Some("History Record"),"GVHistoryUpdateON"=>Some("History Update"),"gvJoinHistory"=>Some("Join History"),_=>None}}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Policy {pub record:bool,pub update:bool,pub lock:bool,pub warning:bool}
impl Default for Policy {fn default()->Self{Self {record:true,update:true,lock:false,warning:true}}}
impl Policy {
    // Record controls creation of new relationships; Update controls existing ones.
    pub fn can_update(self)->bool{self.update}
    pub fn can_edit(self,recorded_child:bool)->bool{!self.lock||!recorded_child}
}
#[derive(Clone,Default,Debug)]
pub struct HistoryGraph {parents:BTreeMap<u64,BTreeSet<u64>>}
impl HistoryGraph {
    pub fn parents(&self,child:u64)->Vec<u64>{self.parents.get(&child).map(|p|p.iter().copied().collect()).unwrap_or_default()}
    pub fn detach(&mut self,child:u64){self.parents.remove(&child);}
    pub fn record(&mut self,child:u64,parents:&[u64],record:bool)->Result<(), &'static str>{
        if !record {self.detach(child);return Ok(());}
        let set:BTreeSet<_>=parents.iter().copied().collect();
        if child==0||parents.is_empty()||set.len()!=parents.len()||set.contains(&child)||set.contains(&0){return Err("Invalid History parents");}
        let mut candidate=self.clone();candidate.parents.insert(child,set);
        candidate.topological()?;*self=candidate;Ok(())
    }
    fn topological(&self)->Result<Vec<u64>, &'static str>{
        let mut counts=BTreeMap::new();let mut descendants:BTreeMap<u64,Vec<u64>>=BTreeMap::new();
        for (&child,parents) in &self.parents {
            let mut count=0;
            for &parent in parents {if self.parents.contains_key(&parent){count+=1;descendants.entry(parent).or_default().push(child);}}
            counts.insert(child,count);
        }
        let mut ready:BTreeSet<_>=counts.iter().filter(|(_,n)|**n==0).map(|(&id,_)|id).collect();
        let mut ordered=Vec::with_capacity(counts.len());
        while let Some(id)=ready.pop_first(){
            ordered.push(id);
            if let Some(children)=descendants.get(&id){for child in children {let count=counts.get_mut(child).expect("known child");*count-=1;if *count==0{ready.insert(*child);}}}
        }
        if ordered.len()!=self.parents.len(){return Err("History cycle");}Ok(ordered)
    }
    pub fn schedule(&self,changed:&[u64],policy:Policy)->Result<Vec<u64>, &'static str>{
        let ordered=self.topological()?;
        if !policy.can_update(){return Ok(Vec::new());}
        let mut dirty:BTreeSet<_>=changed.iter().copied().collect();let mut result=Vec::new();
        for child in ordered{if self.parents[&child].iter().any(|p|dirty.contains(p)){dirty.insert(child);result.push(child);}}
        Ok(result)
    }
}
unsafe fn input<'a>(p:*const c_char)->Option<&'a str>{if p.is_null(){None}else{unsafe{CStr::from_ptr(p)}.to_str().ok()}}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_history_command_kind(p:*const c_char)->u32{
    let name=unsafe{input(p)}.unwrap_or("").to_ascii_lowercase();
    match name.strip_prefix("om9_").unwrap_or(&name){
        "om9-info-015"|"history"|"rhinohistoryon"=>1,
        "om9-info-016"|"matrixclearobjecthistory"|"infosettingsgvclearhistory"=>2,
        "om9-info-017"|"matrixhistoryrecord"|"gvhistoryrecordon"=>3,
        "om9-info-018"|"matrixhistoryupdate"|"gvhistoryupdateon"=>4,
        "om9-tools-017"|"gvjoinhistory"|"joinhistory"=>5,_=>0
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_history_option(p:*const c_char)->u32{
    match unsafe{input(p)}.unwrap_or("").to_ascii_lowercase().as_str(){"record"=>1,"update"=>2,"lock"=>3,"brokenhistorywarning"=>4,_=>0}
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_history_parse_boolean(p:*const c_char)->i32{
    match unsafe{input(p)}.unwrap_or("").to_ascii_lowercase().as_str(){"yes"|"on"|"1"|"true"=>1,"no"|"off"|"0"|"false"=>0,_=>-1}
}
#[unsafe(no_mangle)] pub extern "C" fn om9_history_can_update(record:bool,update:bool)->bool{Policy {record,update,..Policy::default()}.can_update()}
#[unsafe(no_mangle)] pub extern "C" fn om9_history_can_edit(lock:bool,recorded_child:bool)->bool{Policy {lock,..Policy::default()}.can_edit(recorded_child)}
/// Stateless document scheduler: native persistent links are supplied for each call.
/// Returns required output length, or usize::MAX for invalid buffers/graphs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_history_schedule(parents:*const u64,children:*const u64,edge_count:usize,changed:*const u64,changed_count:usize,record:bool,update:bool,out:*mut u64,capacity:usize)->usize{
    // Reject hostile lengths before creating borrowed native slices. IDs are local
    // document IDs supplied for this call; Rust retains no pointers or identities.
    const MAX_ITEMS:usize=1_000_000;
    if edge_count>MAX_ITEMS||changed_count>MAX_ITEMS||capacity>MAX_ITEMS{return usize::MAX;}
    if (edge_count>0&&(parents.is_null()||children.is_null()))||(changed_count>0&&changed.is_null())||(capacity>0&&out.is_null()){return usize::MAX;}
    let ps=if edge_count==0{&[]}else{unsafe{std::slice::from_raw_parts(parents,edge_count)}};
    let cs=if edge_count==0{&[]}else{unsafe{std::slice::from_raw_parts(children,edge_count)}};
    let dirty=if changed_count==0{&[]}else{unsafe{std::slice::from_raw_parts(changed,changed_count)}};
    let mut graph=HistoryGraph::default();
    if dirty.contains(&0){return usize::MAX;}
    for (&p,&c) in ps.iter().zip(cs){if p==0||c==0||p==c||!graph.parents.entry(c).or_default().insert(p){return usize::MAX;}}
    let Ok(schedule)=graph.schedule(dirty,Policy {record,update,..Policy::default()})else{return usize::MAX;};
    if capacity>=schedule.len()&&!schedule.is_empty(){unsafe{std::ptr::copy_nonoverlapping(schedule.as_ptr(),out,schedule.len())};}
    schedule.len()
}
