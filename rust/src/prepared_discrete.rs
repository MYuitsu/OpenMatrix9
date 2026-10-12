// SPDX-License-Identifier: LGPL-2.1-or-later
//! Prepared mesh/current-cloud admission; no native or Python owners enter here.
use crate::builder_history::Json;
use crate::discrete_snapshot::{self,Geometry,Error};
use crate::geometry_contract::{operation_capability,OperationId,OperationOptions,ConversionPolicy,PreservationPolicy,RepresentationKind};
use std::collections::BTreeMap;
fn field<'a>(o:&'a BTreeMap<String,Json>,key:&str)->Result<&'a Json,u32>{o.get(key).ok_or(1)}
fn object(v:&Json)->Result<&BTreeMap<String,Json>,u32>{if let Json::Object(o)=v{Ok(o)}else{Err(1)}}
fn array(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn number(v:&Json)->Result<f64,u32>{if let Json::Number(n)=v{if n.is_finite(){Ok(*n)}else{Err(4)}}else{Err(1)}}
fn boolean(v:&Json)->Result<bool,u32>{if let Json::Boolean(b)=v{Ok(*b)}else{Err(1)}}
fn points(rows:&[Json])->Result<Vec<[f64;3]>,u32>{
 if rows.len()>discrete_snapshot::MAX_POINTS{return Err(2);}
 rows.iter().map(|v|{let row=array(v)?;if row.len()!=3{return Err(2);}
  let p=[number(&row[0])?,number(&row[1])?,number(&row[2])?];
  if p.iter().any(|n|n.abs()>1e150){return Err(4);}Ok(p)
 }).collect()
}
fn index(value:&Json,count:usize)->Result<u64,u32>{let n=number(value)?;
 if n<0.||n.fract()!=0.||n>=count as f64{return Err(3);}Ok(n as u64)
}
fn native(kind:u32,points:&[[f64;3]],faces:&[[u64;3]])->Result<(),u32>{
 Geometry::capture(kind,points,faces).map(|_|()).map_err(|e|match e{Error::Kind=>1,Error::Count=>2,Error::Coordinate=>4,Error::Face=>3})
}
// Compare oriented triangle multisets at the host's declared float32 storage
// precision. Cyclic rotations/reindexing are harmless; reversal and duplicate
// multiplicity are significant. Original float64 vertices/quads remain owned.
fn triangle_key(points:[[f64;3];3])->Result<[u32;9],u32>{
 let mut keys=[[0u32;3];3];
 for (key,point) in keys.iter_mut().zip(points) {for (bits,n) in key.iter_mut().zip(point) {
  let n=n as f32;if !n.is_finite(){return Err(4);}*bits=if n==0.{0}else{n.to_bits()};
 }}
 let first=(0..3).min_by_key(|&i|[keys[i],keys[(i+1)%3],keys[(i+2)%3]]).unwrap();
 Ok(std::array::from_fn(|i|keys[(first+i/3)%3][i%3]))
}
fn mesh(data:&BTreeMap<String,Json>)->Result<(),u32>{
 let rows=array(field(data,"points_xyz")?)?;let triangles=array(field(data,"triangles")?)?;
 if !discrete_snapshot::counts_allowed(8,rows.len(),triangles.len()){return Err(2);}
 let xyz=points(rows)?;let faces=triangles.iter().map(|v|{let row=array(v)?;if row.len()!=3{return Err(3);}
  Ok([index(&row[0],xyz.len())?,index(&row[1],xyz.len())?,index(&row[2],xyz.len())?])}).collect::<Result<Vec<_>,u32>>()?;
 native(8,&xyz,&faces)?;
 let archive=object(field(data,"source_archive_mesh")?)?;let original=points(array(field(archive,"vertices")?)?)?;
 let quads=array(field(archive,"faces")?)?;if quads.len()>discrete_snapshot::MAX_FACES{return Err(2);}
 let mut archive_triangles=Vec::with_capacity(quads.len());
 for quad in quads {let row=array(quad)?;if row.len()!=4{return Err(3);}
  let i=[index(&row[0],original.len())?,index(&row[1],original.len())?,index(&row[2],original.len())?,index(&row[3],original.len())?];
  if i[0]==i[1]||i[1]==i[2]||i[0]==i[2]||(i[3]!=i[2]&&(i[3]==i[0]||i[3]==i[1])){return Err(3);}
  for indices in [[i[0],i[1],i[2]], [i[0],i[2],i[3]]].into_iter().take(if i[3]==i[2]{1}else{2}) {
   if archive_triangles.len()==discrete_snapshot::MAX_FACES{return Err(2);}
   archive_triangles.push(triangle_key(indices.map(|n|original[n as usize]))?);
  }
 }
 let matrix=array(field(data,"native_discrete_location")?)?;if matrix.len()!=4{return Err(6);}
 let mut location=[[0.;4];4];
 for(row,values)in matrix.iter().enumerate(){let values=array(values)?;if values.len()!=4{return Err(6);}
  for(column,v)in values.iter().enumerate(){let n=number(v)?;if n.abs()>1e150{return Err(6);}
   if row==3&&n!=if column==3{1.}else{0.}{return Err(6);}
   location[row][column]=n;
  }
 }
 let mut current_triangles=Vec::with_capacity(faces.len());
 for indices in faces {let transformed=indices.map(|n|{
  let p=xyz[n as usize];std::array::from_fn(|r|location[r][0]*p[0]+location[r][1]*p[1]+location[r][2]*p[2]+location[r][3])
 });current_triangles.push(triangle_key(transformed)?);}
 archive_triangles.sort_unstable();current_triangles.sort_unstable();
 if archive_triangles!=current_triangles{return Err(8);}
 Ok(())
}
fn cloud(data:&BTreeMap<String,Json>,validation:&BTreeMap<String,Json>)->Result<(),u32>{
 let current=object(field(data,"current_fields")?)?;
 let keys=["schema_version","points","normals","colors","values","plane","ordered","has_plane"];
 if current.len()!=keys.len()||keys.iter().any(|k|!current.contains_key(*k))||number(field(current,"schema_version")?)?!=1.{return Err(1);}
 let rows=array(field(current,"points")?)?;if rows.is_empty()||!discrete_snapshot::counts_allowed(9,rows.len(),0){return Err(2);}
 let xyz=points(rows)?;native(9,&xyz,&[])?;
 let normals=array(field(current,"normals")?)?;let colors=array(field(current,"colors")?)?;let values=array(field(current,"values")?)?;
 if (!normals.is_empty()&&normals.len()!=xyz.len())||(!colors.is_empty()&&colors.len()!=4*xyz.len())||(!values.is_empty()&&values.len()!=xyz.len()){return Err(2);}
 points(normals)?;
 for color in colors{let n=number(color)?;if !(0. ..=255.).contains(&n)||n.fract()!=0.{return Err(2);}}
 for value in values{number(value)?;}
 let plane=array(field(current,"plane")?)?;if plane.len()!=16{return Err(2);}for value in plane{number(value)?;}
 boolean(field(current,"ordered")?)?;
 if boolean(field(current,"has_plane")?)?&&!boolean(field(validation,"native_plane_valid")?)?{return Err(2);}
 Ok(())
}
pub(crate) fn validate(data:&BTreeMap<String,Json>,ownership:&str,validation:&BTreeMap<String,Json>)->Result<&'static str,u32>{
 let Json::String(representation)=field(data,"representation")? else{return Err(1);};
 let (name,kind)=match representation.as_str(){"mesh"=>("mesh",RepresentationKind::Mesh),"point_cloud"=>("point_cloud",RepresentationKind::PointCloud),_=>return Err(1)};
 let options=OperationOptions::WorkingExchange{archive_version:5,conversion:ConversionPolicy::ExistingConverters,preservation:PreservationPolicy::ReportObservedFacts};
 operation_capability(OperationId::ImportWorkingV5,kind,options).map_err(|_|1u32)?;
 if name=="mesh" {if ownership!="private_native_copy"{return Err(1);}mesh(data)?;}
 else{if ownership!="private_plain_copy"{return Err(1);}cloud(data,validation)?;}
 Ok(name)
}
