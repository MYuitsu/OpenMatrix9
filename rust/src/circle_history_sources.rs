// SPDX-License-Identifier: LGPL-2.1-or-later
//! Reconstruct replay inputs from immutable storage and current source evidence.
//! Selection follows native control-pole precedence and native topology indices.
use super::{Json,BTreeMap,rows,xyz,integer,selected_curve,source_point_matches,near};
use crate::prepared_result::{object,field,text,number,array};

pub(super) fn validate(c:&BTreeMap<String,Json>,current:&[[f64;3]])->Result<(),u32>{
 let refs=rows(field(c,"source_refs")?)?;
 let snapshots=rows(field(c,"source_snapshots")?)?;
 let stored=rows(field(c,"recorded_points_world")?)?;
 let roles=rows(field(c,"recorded_source_roles")?)?;
 let parameters=rows(field(c,"recorded_source_parameters")?)?;
 let config=array::<24>(field(c,"recipe")?)?;
 if text(field(c,"storage")?)?=="legacy_around_curve"{
  // The old native storage has no Recipe/Points/Sources. Its current path and
  // fraction are independently evaluated by the parent validator.
  if !rows(field(c,"recorded_recipe")?)?.is_empty()||!stored.is_empty()||!roles.is_empty()||!parameters.is_empty()
   ||refs.len()!=1||text(field(object(&refs[0])?,"role")?)?!="history_path"{return Err(1);}
  return Ok(());
 }
 let mut expected=array::<24>(field(c,"recorded_recipe")?)?;
 super::Recipe::decode(&expected).map_err(|_|2u32)?;
 if stored.is_empty()||stored.len()>1024{return Err(2);}
 let mut points=stored.iter().map(xyz).collect::<Result<Vec<_>,_>>()?;
 let source_count=refs.iter().try_fold(0usize,|n,r|->Result<usize,u32>{Ok(n+usize::from(text(field(object(r)?,"role")?)?=="history_source"))})?;
 if roles.len()!=source_count.checked_mul(3).ok_or(2u32)?||parameters.len()!=source_count{return Err(1);}
 let mut bound=vec![false;points.len()];
 let mut replacements=Vec::<(usize,usize,Vec<[f64;3]>)>::new();
 let mut replacement_point_count=0usize;
 let mut direction=None;let mut radius=None;
 for reference in refs{
  let reference=object(reference)?;
  if text(field(reference,"role")?)?=="history_path"{continue;}
  let index=integer(field(reference,"role_index")?,source_count)?.checked_mul(3).ok_or(2u32)?;
  let role=number(roles.get(index).ok_or(1u32)?)?;
  let offset=integer(roles.get(index+1).ok_or(1u32)?,points.len())?;
  let count=integer(roles.get(index+2).ok_or(1u32)?,points.len())?;
  if offset>=points.len()||count>points.len()-offset{return Err(1);}
  let parameter=number(parameters.get(index/3).ok_or(1u32)?)?;
  if !parameter.is_finite()||role!=2.&&parameter!=0.{return Err(2);}
  let slot=integer(field(reference,"input_snapshot_index")?,snapshots.len())?.checked_sub(1).ok_or(1u32)?;
  let snapshot=object(snapshots.get(slot).ok_or(1u32)?)?;
  if matches!(role,0.|1.|2.|3.){
   if count==0||bound[offset..offset+count].iter().any(|b|*b){return Err(1);}
   bound[offset..offset+count].fill(true);
  }
  if role==2.{
   if expected[1]!=6.||count!=1||!matches!(number(field(reference,"selected_representation")?)?,2.|3.){return Err(1);}
   points[offset]=selected_curve(reference,snapshot)?.at_fraction(parameter)?.0;
   continue;
  }
  let selected=rows(field(snapshot,"selected_points_world")?)?;
  if selected.is_empty()||selected.len()>1024{return Err(2);}
  crate::owned_point_selection::validate(reference,snapshot,role==3.)?;
  match role{
   0.=>{if count!=1||selected.len()!=1{return Err(1);}points[offset]=xyz(&selected[0])?;},
   1.|3.=>{
    if expected[1]!=5.||role==3.&&selected.len()!=count{return Err(1);}
    replacement_point_count=replacement_point_count.checked_add(selected.len()).ok_or(2u32)?;
    if replacement_point_count>1024{return Err(2);}
    replacements.push((offset,count,selected.iter().map(xyz).collect::<Result<Vec<_>,_>>()?));
   },
   -1.=>{
    if count!=0||offset!=0||selected.len()!=1||direction.is_some(){return Err(1);}
    direction=Some((offset,xyz(&selected[0])?));
   },
   4.=>{
    if count!=0||selected.len()!=1||!matches!(expected[1],2.|6.)||radius.is_some(){return Err(1);}
    radius=Some((offset,xyz(&selected[0])?));
   },
   _=>return Err(1),
  }
 }
 replacements.sort_by_key(|r|r.0);
 if !replacements.is_empty(){
  // Spans address original storage. Preserve unbound numeric gaps and suffixes
  // while each whole source can grow/shrink; never zip by old point count.
  let mut rebuilt=Vec::new();let mut cursor=0;
  for(offset,count,selected)in replacements{
   if offset<cursor{return Err(1);}
   let size=rebuilt.len()+offset-cursor+selected.len();if size>1024{return Err(2);}
   rebuilt.extend_from_slice(&points[cursor..offset]);rebuilt.extend(selected);cursor=offset+count;
  }
  if rebuilt.len()+points.len()-cursor>1024{return Err(2);}
  rebuilt.extend_from_slice(&points[cursor..]);points=rebuilt;
 }
 if points.len()!=current.len()||points.iter().zip(current).any(|(a,b)|!source_point_matches(*a,*b)){return Err(5);}
 let mut derived=[false;24];
 if let Some((offset,endpoint))=direction{
  let anchor=points.get(offset).ok_or(1u32)?;
  expected[5]=1.;for j in 0..3{expected[6+j]=endpoint[j]-anchor[j];derived[6+j]=true;}
 }
 if let Some((offset,endpoint))=radius{
  let anchor=points.get(offset).ok_or(1u32)?;
  let d:[f64;3]=std::array::from_fn(|j|endpoint[j]-anchor[j]);
  expected[4]=d[0].hypot(d[1]).hypot(d[2]);derived[4]=true;
  if !(1e-7..=1e9).contains(&expected[4]){return Err(2);}
 }
 for j in 0..24{
  if !config[j].is_finite()||if derived[j]{!near(config[j],expected[j])}else{config[j]!=expected[j]}{return Err(5);}
 }
 Ok(())
}
