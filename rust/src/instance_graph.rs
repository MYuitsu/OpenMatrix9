// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded owned definition occurrences; shared identities are not deduplicated.
use crate::core_objects::Stamp;
use crate::support_snapshot::{Json,SerializedLimit};
use crate::geometry_frame::{Frame,IDENTITY};
use std::fmt::Write;
pub const MAX_NODES:usize=4096;
pub const MAX_DEPTH:usize=64;
pub const ROOT:usize=usize::MAX;
#[repr(C)]
#[derive(Clone,Copy,Debug)]
pub struct Node {pub stamp:Stamp,pub parent:usize,pub representation:u32,pub reserved:u32,pub local_to_world:[f64;16]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Count,Identity,Structure,Depth,Frame}
#[derive(Clone,Debug)]
pub struct Graph {nodes:Vec<Node>}
pub fn counts_allowed(nodes:usize,depth:usize)->bool {nodes>0&&nodes<=MAX_NODES&&depth>0&&depth<=MAX_DEPTH}
impl Graph {
    pub fn capture(nodes:&[Node])->Result<Self,Error>{
        if !counts_allowed(nodes.len(),1){return Err(Error::Count);}
        let mut depths=Vec::with_capacity(nodes.len());
        let mut children=vec![0usize;nodes.len()];
        for (i,node) in nodes.iter().enumerate() {
            let s=node.stamp;
            if [s.document,s.object,s.generation,s.revision,s.epoch].contains(&0){return Err(Error::Identity);}
            if node.reserved!=0||!(1..=11).contains(&node.representation){return Err(Error::Structure);}
            let depth=if i==0 {
                if node.parent!=ROOT||!matches!(node.representation,10|11){return Err(Error::Structure);}
                1usize
            } else {
                if node.parent>=i||!matches!(nodes[node.parent].representation,10|11){return Err(Error::Structure);}
                children[node.parent]+=1;depths[node.parent]+1
            };
            if !counts_allowed(nodes.len(),depth){return Err(Error::Depth);}
            Frame::compose(IDENTITY,node.local_to_world,IDENTITY,IDENTITY).map_err(|_|Error::Frame)?;
            depths.push(depth);
        }
        if nodes.iter().zip(children).any(|(n,count)|n.representation==10&&count!=1){return Err(Error::Structure);}
        Ok(Self{nodes:nodes.to_vec()})
    }
    pub fn json(&self)->Result<String,SerializedLimit>{
        let mut j=Json(String::new());
        let result=(||{
            j.write_str("{\"schema_version\":1,\"identity_scope\":\"revision_scoped_definition_occurrences\",\"nodes\":[")?;
            let names=["unknown","point","curve","polycurve","trimmed_face","shell","solid","brep","mesh","point_cloud","instance","group"];
            for (index,n) in self.nodes.iter().enumerate(){if index!=0{j.write_char(',')?;}
                let s=n.stamp;
                write!(j,"{{\"index\":{index},\"parent\":")?;
                if n.parent==ROOT{j.write_str("null")?;}else{write!(j,"{}",n.parent)?;}
                write!(j,",\"representation\":\"{}\",\"stamp\":{{\"document\":\"{}\",\"object\":\"{}\",\"generation\":\"{}\",\"revision\":\"{}\",\"epoch\":\"{}\"}},\"local_to_world\":[",names[n.representation as usize],s.document,s.object,s.generation,s.revision,s.epoch)?;
                for r in 0..4 {if r!=0{j.write_char(',')?;}crate::support_snapshot::numbers(&mut j,&n.local_to_world[r*4..r*4+4])?;}
                let f=Frame::compose(IDENTITY,n.local_to_world,IDENTITY,IDENTITY).map_err(|_|std::fmt::Error)?;
                write!(j,"],\"world_orientation_sign\":{}}}",f.orientation_sign())?;
            }
            j.write_str("]}")
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
