// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned initial solved contacts, independently checked before state publication.
use super::{CurveSession,Json};
use crate::prepared_result::{object,text,number};
pub(super) fn validate(s:&CurveSession,plan:&crate::circle::Plan,values:&Json,indexing:&Json)->Result<(),u32>{
 let Json::Array(contacts)=values else{return Err(1);};let constraints=&s.circle_options.constraints;
 if contacts.len()!=constraints.len()||contacts.is_empty()||contacts.len()>3||text(indexing)?!="current_constraint_points_zero_based"{return Err(1);}
 let dot=|a:[f64;3],b:[f64;3]|a.into_iter().zip(b).map(|(a,b)|a*b).sum::<f64>();
 for(index,((probe,fixed),fraction))in constraints.iter().zip(contacts).enumerate(){
  if *fixed{
   if fraction!=&Json::Other||!crate::circle_history::tangent_point_matches(plan,*probe){return Err(5);}continue;
  }
  let(reference,snapshot)=s.accepted_input_log.native_constraint(index)?;
  let curve=crate::owned_curve_input::selected_reference(object(reference)?,object(snapshot)?)?;
  let(point,tangent)=curve.at_fraction(number(fraction)?)?;
  if !crate::circle_history::tangent_point_matches(plan,point)||dot(tangent,plan.normal).abs()>1e-8{return Err(5);}
  let radial=std::array::from_fn(|j|point[j]-plan.center[j]);
  if dot(radial,tangent).abs()/plan.radius>1e-8{return Err(5);}
  if index==0&&s.circle_options.from_first{
   let d:[f64;3]=std::array::from_fn(|j|point[j]-probe[j]);if d[0].hypot(d[1]).hypot(d[2])>1e-6{return Err(5);}
  }
 }
 Ok(())
}
