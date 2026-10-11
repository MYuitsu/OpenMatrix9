// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned native discrete geometry; native owner adapters are separate.
use crate::support_snapshot::{Json,SerializedLimit,numbers};
use std::fmt::Write;
pub const MAX_POINTS:usize=100_000;
pub const MAX_FACES:usize=100_000;
pub const MAX_BYTES:usize=8*1024*1024;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Kind,Count,Coordinate,Face}
#[derive(Debug)]
pub struct Geometry {representation:u32,points:Vec<[f64;3]>,faces:Vec<[u64;3]>}
pub fn counts_allowed(kind:u32,points:usize,faces:usize)->bool {
    matches!(kind,8|9)&&(kind==8||faces==0)&&points<=MAX_POINTS&&faces<=MAX_FACES
        &&points.checked_add(faces).and_then(|n|n.checked_mul(24)).and_then(|n|n.checked_add(std::mem::size_of::<Geometry>())).is_some_and(|n|n<=MAX_BYTES)
}
pub fn triangle_allowed(face:[u64;3],points:usize)->bool{
    face.iter().all(|&i|i<(points as u64))&&face[0]!=face[1]&&face[1]!=face[2]&&face[0]!=face[2]
}
impl Geometry {
    pub fn capture(kind:u32,points:&[[f64;3]],faces:&[[u64;3]])->Result<Self,Error>{
        if !matches!(kind,8|9)||(kind==9&&!faces.is_empty()){return Err(Error::Kind);}
        if !counts_allowed(kind,points.len(),faces.len()){return Err(Error::Count);}
        if points.iter().flatten().any(|v|!v.is_finite()||v.abs()>1e150){return Err(Error::Coordinate);}
        if faces.iter().any(|f|!triangle_allowed(*f,points.len())){return Err(Error::Face);}
        Ok(Self{representation:kind,points:points.to_vec(),faces:faces.to_vec()})
    }
    pub fn points(&self)->&[[f64;3]]{&self.points}
    pub fn faces(&self)->&[[u64;3]]{&self.faces}
    pub fn json(&self)->Result<String,SerializedLimit>{
        let mut j=Json(String::new());
        let result=(||{
            write!(j,"{{\"schema_version\":1,\"kind\":\"{}\",\"coordinate_storage\":\"owned_float64\",\"points_xyz\":[",if self.representation==8{"mesh"}else{"point_cloud"})?;
            for (i,p) in self.points.iter().enumerate(){if i!=0{j.write_char(',')?;}numbers(&mut j,p)?;}
            j.write_str("],\"triangles\":")?;
            if self.representation==9{j.write_str("null")?;}else{
                j.write_char('[')?;for (i,f) in self.faces.iter().enumerate(){if i!=0{j.write_char(',')?;}numbers(&mut j,f)?;}j.write_char(']')?;
            }
            j.write_str(",\"bounds_xyz\":")?;
            if let Some(first)=self.points.first(){let (mut low,mut high)=(*first,*first);
                for p in &self.points{for a in 0..3{low[a]=low[a].min(p[a]);high[a]=high[a].max(p[a]);}}
                j.write_char('[')?;numbers(&mut j,&low)?;j.write_char(',')?;numbers(&mut j,&high)?;j.write_char(']')?;
            }else{j.write_str("null")?;}
            j.write_str(",\"other_native_channels\":\"uninspected\",\"manifold_certificate\":null}")
        })();result.map(|_|j.0).map_err(|_|SerializedLimit)
    }
}
