// SPDX-License-Identifier: LGPL-2.1-or-later
//! Fallible, bounded decoding of captured native curve values. No native handles.
use crate::builder_history::Json;
use crate::prepared_result::{object,field,text,number,array};
use crate::support_snapshot::{self as support,Curve,BezierCurve};
use crate::analytic_support::{Analytic,AnalyticInput};
use crate::derived_support::{Tree,DerivedNode,LeafInput,MAX_DEPTH};
use crate::nurbs_basis::{Axis,CurveBasis};
use crate::geometry_frame::{Frame,IDENTITY};

fn rows(v:&Json)->Result<&[Json],u32>{if let Json::Array(a)=v{Ok(a)}else{Err(1)}}
fn integer(v:&Json,max:usize)->Result<usize,u32>{
 let n=number(v)?;if !n.is_finite()||n.fract()!=0.||n<0.||n>max as f64{return Err(2);}Ok(n as usize)
}
fn boolean(v:&Json)->Result<bool,u32>{if let Json::Boolean(b)=v{Ok(*b)}else{Err(1)}}
fn header(data:&std::collections::BTreeMap<String,Json>)->Result<(),u32>{
 if number(field(data,"schema_version")?)?!=1.{return Err(1);}Ok(())
}
enum OwnedCurve{Nurbs(Curve),Bezier(BezierCurve),Analytic(Analytic),Derived(Tree)}
impl OwnedCurve{
 fn evaluate(&self,u:f64)->Result<([f64;3],[f64;3]),u32>{
  match self{Self::Nurbs(c)=>c.evaluate(u),Self::Bezier(c)=>c.evaluate(u),Self::Analytic(c)=>c.evaluate_curve(u),Self::Derived(c)=>c.evaluate_curve(u)}.map_err(|_|4u32)
 }
 fn decode(mut value:&Json,edge_domain:[f64;2])->Result<Self,u32>{
  let mut nodes=Vec::new();
  loop{
   let data=object(value)?;header(data)?;
   let kind=match text(field(data,"kind")?)?{"offset_curve"=>1,"trimmed_curve"=>5,_=>break};
   if nodes.len()>=MAX_DEPTH{return Err(2);}
   if integer(field(data,"dimension")?,3)?!=3||text(field(data,"parameter_convention")?)?!="occt_native"{return Err(1);}
   let domains=rows(field(data,"natural_domains")?)?;if domains.len()!=1{return Err(2);}
   let(mut bounds,mut infinite_mask)=([0.;4],0u32);
   if domains[0]==Json::String("unbounded".into()){infinite_mask=3;}else{
    let domain=rows(&domains[0])?;if domain.len()!=2{return Err(2);}
    for j in 0..2{match &domain[j]{Json::Number(n)=>bounds[j]=*n,Json::String(s)if s==if j==0{"-infinity"}else{"infinity"}=>infinite_mask|=1<<j,_=>return Err(2)}}
   }
   nodes.push(DerivedNode{kind,dimension:3,direction:if kind==1{array(field(data,"direction_xyz")?)?}else{[0.;3]},origin:[0.;3],offset:if kind==1{number(field(data,"offset")?)?}else{0.},bounds,infinite_mask,reserved:0});
   value=field(data,"basis")?;
  }
  // Validate wrapper bounds/category before allocating any supporting basis.
  if !nodes.is_empty(){crate::derived_support::validate_chain(false,3,&nodes).map_err(|_|2u32)?;}
  for n in &nodes{
   if n.infinite_mask&1==0&&edge_domain[0]<n.bounds[0]||n.infinite_mask&2==0&&edge_domain[1]>n.bounds[1]{return Err(2);}
  }
  let data=object(value)?;header(data)?;
  let kind=text(field(data,"kind")?)?;
  if kind=="analytic_curve"{
   if integer(field(data,"dimension")?,3)?!=3||text(field(data,"parameter_convention")?)?!="occt_native"{return Err(1);}
   let analytic_type=match text(field(data,"analytic_type")?)?{"line"=>1,"circle"=>2,"ellipse"=>3,"hyperbola"=>4,"parabola"=>5,_=>return Err(1)};
   let params=object(field(data,"parameters")?)?;
   let radii=match analytic_type{2=>[number(field(params,"radius")?)?,0.],3|4=>[number(field(params,"major_radius")?)?,number(field(params,"minor_radius")?)?],5=>[number(field(params,"focal")?)?,0.],_=>[0.;2]};
   let axes=rows(field(data,"axes_xyz")?)?;if axes.len()!=3{return Err(2);}
   let input=AnalyticInput{kind:analytic_type,dimension:3,origin:array(field(data,"origin_xyz")?)?,axes:[array(&axes[0])?,array(&axes[1])?,array(&axes[2])?],radii,extra:0.};
   return if nodes.is_empty(){Ok(Self::Analytic(Analytic::capture(input).map_err(|_|2u32)?))}else{Ok(Self::Derived(Tree::capture(LeafInput::Analytic(input),3,&nodes).map_err(|_|2u32)?))};
  }
  let bezier=match kind{"bezier_curve"=>true,"nurbs_curve"=>false,_=>return Err(1)};
  let policy=support::policy();let pole_rows=rows(field(data,"poles_xyz")?)?;
  if !(2..=policy.max_poles).contains(&pole_rows.len()){return Err(2);}
  let weight_rows=match field(data,"weights")?{Json::Other=>None,v=>Some(rows(v)?)};
  if weight_rows.is_some_and(|w|w.len()!=pole_rows.len()){return Err(2);}
  let axis=if bezier{data}else{object(field(data,"axis")?)?};
  let degree=integer(field(axis,"degree")?,policy.max_degree)?;
  if degree==0{return Err(2);}
  let domain=array(field(axis,"full_domain")?)?;
  if edge_domain[0]<domain[0]||edge_domain[1]>domain[1]{return Err(2);}
  let knot_rows=if bezier{None}else{Some(rows(field(axis,"knots")?)?)};
  let multiplicity_rows=if bezier{None}else{Some(rows(field(axis,"multiplicities")?)?)};
  let knot_count=knot_rows.map_or(2,|k|k.len());
  if !(2..=policy.max_knots).contains(&knot_count)||multiplicity_rows.is_some_and(|m|m.len()!=knot_count){return Err(2);}
  let bytes=pole_rows.len().checked_mul(24).and_then(|n|n.checked_add(weight_rows.map_or(0,|w|w.len()*8))).and_then(|n|knot_count.checked_mul(16).and_then(|k|n.checked_add(k))).ok_or(2u32)?;
  if bytes>policy.max_bytes{return Err(2);}
  let periodic=if bezier{false}else{boolean(field(axis,"periodic")?)?};
  let knots=if let Some(k)=knot_rows{k.iter().map(number).collect::<Result<Vec<_>,_>>()?}else{vec![0.,1.]};
  let multiplicities=if let Some(m)=multiplicity_rows{m.iter().map(|v|integer(v,degree+1)).collect::<Result<Vec<_>,_>>()?}else{vec![degree+1;2]};
  let poles=pole_rows.iter().map(array).collect::<Result<Vec<_>,_>>()?;
  let weights=weight_rows.map(|w|w.iter().map(number).collect::<Result<Vec<_>,_>>()).transpose()?;
  let basis=CurveBasis{poles:&poles,weights:weights.as_deref(),axis:Axis{degree,knots:&knots,multiplicities:&multiplicities,periodic,domain}};
  if nodes.is_empty(){if bezier{Ok(Self::Bezier(BezierCurve::capture(basis).map_err(|_|2u32)?))}else{Ok(Self::Nurbs(Curve::capture(basis).map_err(|_|2u32)?))}}
  else{Ok(Self::Derived(Tree::capture(if bezier{LeafInput::BezierCurve(basis)}else{LeafInput::NurbsCurve(basis)},3,&nodes).map_err(|_|2u32)?))}
 }
}

/// FitPoints reads complete direct control poles, independent of the selected
/// edge interval. Validate its basis without requiring evaluator continuation.
pub(crate) fn validate_control_curve(value:&Json)->Result<(),u32>{
 let data=object(value)?;
 let axis=match text(field(data,"kind")?)?{"bezier_curve"=>data,"nurbs_curve"=>object(field(data,"axis")?)?,_=>return Err(1)};
 let domain=array(field(axis,"full_domain")?)?;
 OwnedCurve::decode(value,domain)?;Ok(())
}
pub(crate) struct SelectedEdge{curve:OwnedCurve,domain:[f64;2],frame:Frame}
pub(crate) fn selected_reference(reference:&std::collections::BTreeMap<String,Json>,snapshot:&std::collections::BTreeMap<String,Json>)->Result<SelectedEdge,u32>{
 if text(field(reference,"snapshot_frame")?)?!="document_world"{return Err(6);}
 let edges=rows(field(object(field(snapshot,"topology_data")?)?,"edges_3d")?)?;
 let sub=text(field(reference,"subelement")?)?;
 let edge=if sub.is_empty(){if edges.len()!=1{return Err(3);} &edges[0]}else{
  let index=sub.strip_prefix("Edge").ok_or(1u32)?;let n=index.parse::<usize>().map_err(|_|1u32)?;
  if n==0||n.to_string()!=index{return Err(1);}
  let mut found=edges.iter().filter(|v|object(v).ok().and_then(|v|v.get("native_topology_index")).and_then(|v|text(v).ok())==Some(index));
  let edge=found.next().ok_or(3u32)?;if found.next().is_some(){return Err(3);}edge
 };
 SelectedEdge::capture(edge)
}
impl SelectedEdge{
 pub(crate) fn capture(value:&Json)->Result<Self,u32>{
  let edge=object(value)?;
  if field(edge,"curve_present")?!=&Json::Boolean(true)||field(edge,"degenerate")?!=&Json::Boolean(false)
   ||text(field(edge,"geometry_coordinates")?)?!="native_local"{return Err(3);}
  let domain=array(field(edge,"domain")?)?;
  if !crate::topology_snapshot::domain_valid(domain[0],domain[1]){return Err(2);}
  let rows=rows(field(edge,"native_location")?)?;if rows.len()!=4{return Err(6);}
  let mut matrix=[0.;16];for(i,row)in rows.iter().enumerate(){matrix[i*4..i*4+4].copy_from_slice(&array::<4>(row)?);}
  // A document_world CAD snapshot already embeds its source/instance placement
  // in each native location. SourceRef.world must not be multiplied a second time.
  let frame=Frame::compose(IDENTITY,IDENTITY,IDENTITY,matrix).map_err(|_|6u32)?;
  Ok(Self{curve:OwnedCurve::decode(field(edge,"geometry_data")?,domain)?,domain,frame})
 }
 pub(crate) fn at_fraction(&self,fraction:f64)->Result<([f64;3],[f64;3]),u32>{
  if !fraction.is_finite()||!(0. ..=1.).contains(&fraction){return Err(2);}
  let [lo,hi]=self.domain;let u=if fraction==1.{hi}else{lo+(hi-lo)*fraction};
  let(p,d)=self.curve.evaluate(u)?;let point=self.frame.transform(p).map_err(|_|6u32)?;
  let m=self.frame.matrix();let d:[f64;3]=std::array::from_fn(|r|(0..3).map(|c|m[4*r+c]*d[c]).sum());
  let length=d[0].hypot(d[1]).hypot(d[2]);
  if !length.is_finite()||length==0.||d.iter().any(|v|!v.is_finite()){return Err(4);}
  Ok((point,d.map(|v|v/length)))
 }
}
