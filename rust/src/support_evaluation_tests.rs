use crate::{builder_history::{Json,parse_document},nurbs_basis::{Axis,CurveBasis},support_snapshot::{Curve,BezierCurve}};
fn field<'a>(v:&'a Json,key:&str)->&'a Json{let Json::Object(m)=v else{panic!("object")};m.get(key).unwrap()}
fn rows(v:&Json)->&[Json]{let Json::Array(v)=v else{panic!("array")};v}
fn number(v:&Json)->f64{let Json::Number(v)=v else{panic!("number")};*v}
fn point(v:&Json)->[f64;3]{rows(v).iter().map(number).collect::<Vec<_>>().try_into().unwrap()}
fn near(actual:[f64;3],expected:[f64;3],name:&Json,u:f64,quantity:&str){
 for(a,b)in actual.into_iter().zip(expected){assert!((a-b).abs()<=2e-9*b.abs().max(1.),"{name:?} parameter {u} {quantity}: {actual:?} versus native {expected:?}");}
}
#[test]
fn owned_basis_value_and_derivative_match_nine_actual_occt_families(){
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_oracle.json"),1024*1024,100000).unwrap();
 let mut query_count=0;
 for case in rows(&cases){
  let data=field(case,"geometry_data");let poles:Vec<_>=rows(field(data,"poles_xyz")).iter().map(point).collect();
  let weights=match field(data,"weights"){Json::Other=>None,v=>Some(rows(v).iter().map(number).collect::<Vec<_>>())};
  let bezier=field(data,"kind")==&Json::String("bezier_curve".into());
  let(degree,knots,multiplicities,periodic,domain)=if bezier{
   let degree=number(field(data,"degree"))as usize;(degree,vec![0.,1.],vec![degree+1;2],false,[0.,1.])
  }else{
   let axis=field(data,"axis");let domain=rows(field(axis,"full_domain"));
   (number(field(axis,"degree"))as usize,rows(field(axis,"knots")).iter().map(number).collect(),rows(field(axis,"multiplicities")).iter().map(|v|number(v)as usize).collect(),field(axis,"periodic")==&Json::Boolean(true),[number(&domain[0]),number(&domain[1])])
  };
  let basis=CurveBasis{poles:&poles,weights:weights.as_deref(),axis:Axis{degree,knots:&knots,multiplicities:&multiplicities,periodic,domain}};
  let snapshot=Curve::capture(basis).unwrap();let before=snapshot.json().unwrap();
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let(p,d)=snapshot.evaluate(u).unwrap();
   near(p,point(field(query,"value")),field(case,"name"),u,"value");near(d,point(field(query,"derivative")),field(case,"name"),u,"D1");
   if bezier{let(p,d)=BezierCurve::capture(basis).unwrap().evaluate(u).unwrap();near(p,point(field(query,"value")),field(case,"name"),u,"Bezier value");near(d,point(field(query,"derivative")),field(case,"name"),u,"Bezier D1");}
   query_count+=1;
  }
  assert_eq!(snapshot.json().unwrap(),before,"evaluation preserves the original compact owned basis");
 }
 assert_eq!(query_count,93);
}
#[test]
fn owned_basis_evaluation_uses_absolute_parameters_and_rejects_outside_domain(){
 let poles=[[1.,2.,3.],[7.,11.,15.]];
 let c=Curve::capture(CurveBasis{poles:&poles,weights:None,axis:Axis{degree:1,knots:&[2.,5.],multiplicities:&[2,2],periodic:false,domain:[2.,5.]}}).unwrap();
 assert_eq!(c.evaluate(3.5).unwrap(),([4.,6.5,9.],[2.,3.,4.]));
 for u in [0.5,1.999,5.001,f64::NAN,f64::INFINITY]{assert!(c.evaluate(u).is_err(),"invalid absolute parameter {u}");}
}
#[test]
fn owned_basis_evaluation_retains_extreme_equal_weights_without_underflow(){
 let poles=[[1.,2.,3.],[7.,11.,15.]];let weights=[1e-307,1e-307];
 let c=Curve::capture(CurveBasis{poles:&poles,weights:Some(&weights),axis:Axis{degree:1,knots:&[2.,5.],multiplicities:&[2,2],periodic:false,domain:[2.,5.]}}).unwrap();
 assert_eq!(c.evaluate(3.5).unwrap(),([4.,6.5,9.],[2.,3.,4.]));
}
#[test]
fn owned_analytic_value_and_derivative_match_all_five_native_curve_kinds(){
 use crate::analytic_support::{Analytic,AnalyticInput};
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_analytic_oracle.json"),1024*1024,100000).unwrap();
 let mut count=0;
 for case in rows(&cases){
  let data=field(case,"geometry_data");if field(data,"kind")!=&Json::String("analytic_curve".into()){continue;}
  let kind=["line","circle","ellipse","hyperbola","parabola"].iter().position(|name|field(data,"analytic_type")==&Json::String((*name).into())).unwrap()+1;
  let parameters=field(data,"parameters");let optional=|key|if let Json::Object(m)=parameters{m.get(key).map_or(0.,number)}else{panic!("parameters")};
  let radii=match kind{2=>[optional("radius"),0.],3|4=>[optional("major_radius"),optional("minor_radius")],5=>[optional("focal"),0.],_=>[0.;2]};
  let axes=rows(field(data,"axes_xyz"));
  let snapshot=Analytic::capture(AnalyticInput{kind:kind as u32,dimension:3,origin:point(field(data,"origin_xyz")),axes:[point(&axes[0]),point(&axes[1]),point(&axes[2])],radii,extra:0.}).unwrap();
  let before=snapshot.json().unwrap();
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let(p,d)=snapshot.evaluate_curve(u).unwrap();
   near(p,point(field(query,"value")),field(case,"name"),u,"analytic value");near(d,point(field(query,"derivative")),field(case,"name"),u,"analytic D1");count+=1;
  }
  assert_eq!(snapshot.json().unwrap(),before);
 }
 assert_eq!(count,45);
}
#[test]
fn owned_analytic_evaluation_preserves_indirect_uv_axes(){
 use crate::analytic_support::{Analytic,AnalyticInput};
 let c=Analytic::capture(AnalyticInput{kind:3,dimension:2,origin:[4.,6.,0.],axes:[[0.6,0.8,0.],[0.8,-0.6,0.],[0.,0.,-1.]],radii:[5.,2.],extra:0.}).unwrap();
 assert_eq!(c.evaluate_curve(0.).unwrap(),([7.,10.,0.],[1.6,-1.2,0.]));
 assert!(c.evaluate_curve(f64::NAN).is_err());
 assert!(c.evaluate_curve(f64::INFINITY).is_err());
}
#[test]
fn owned_basis_evaluation_retains_restricted_domain_without_renormalization(){
 let poles=[[1.,2.,3.],[7.,11.,15.]];
 let c=Curve::capture(CurveBasis{poles:&poles,weights:None,axis:Axis{degree:1,knots:&[2.,5.],multiplicities:&[2,2],periodic:false,domain:[2.5,4.5]}}).unwrap();
 assert_eq!(c.evaluate(3.5).unwrap(),([4.,6.5,9.],[2.,3.,4.]));
 assert!(c.evaluate(2.).is_err());assert!(c.evaluate(5.).is_err());
}

enum OwnedCurve {Nurbs(Curve),Bezier(BezierCurve),Analytic(crate::analytic_support::Analytic),Derived(crate::derived_support::Tree)}
impl OwnedCurve{
 fn taylor(&self,u:f64,order:usize)->Result<Vec<[f64;3]>,super::EvaluationError>{match self{
  Self::Nurbs(c)=>c.taylor(u,order),Self::Bezier(c)=>c.taylor(u,order),
  Self::Analytic(c)=>c.taylor_curve(u,order),Self::Derived(c)=>c.taylor_curve(u,order),
 }}
}
fn capture_owned(mut data:&Json)->OwnedCurve{
 use crate::derived_support::{Tree,DerivedNode,LeafInput};
 use crate::analytic_support::{Analytic,AnalyticInput};
 let mut nodes=Vec::new();let mut dimension=3;
 while matches!(field(data,"kind"),Json::String(k)if k=="offset_curve"||k=="trimmed_curve"){
  let kind=if field(data,"kind")==&Json::String("offset_curve".into()){1}else{5};
  dimension=number(field(data,"dimension"))as u32;
  let domains=rows(field(data,"natural_domains"));assert_eq!(domains.len(),1);
  let(mut bounds,mut infinite_mask)=([0.;4],0);
  if domains[0]==Json::String("unbounded".into()){infinite_mask=3;}else{
   let d=rows(&domains[0]);for j in 0..2{if let Json::Number(v)=d[j]{bounds[j]=v;}else{infinite_mask|=1<<j;}}
  }
  let direction=if kind==1&&dimension==3{point(field(data,"direction_xyz"))}else{[0.;3]};
  nodes.push(DerivedNode{kind,dimension,direction,origin:[0.;3],offset:if kind==1{number(field(data,"offset"))}else{0.},bounds,infinite_mask,reserved:0});
  data=field(data,"basis");
 }
 if field(data,"kind")==&Json::String("analytic_curve".into()){
  let kind=["line","circle","ellipse","hyperbola","parabola"].iter().position(|n|field(data,"analytic_type")==&Json::String((*n).into())).unwrap()+1;
  let parameters=field(data,"parameters");let optional=|key|if let Json::Object(m)=parameters{m.get(key).map_or(0.,number)}else{panic!("parameters")};
  let radii=match kind{2=>[optional("radius"),0.],3|4=>[optional("major_radius"),optional("minor_radius")],5=>[optional("focal"),0.],_=>[0.;2]};
  let axes=rows(field(data,"axes_xyz"));
  let input=AnalyticInput{kind:kind as u32,dimension,origin:point(field(data,"origin_xyz")),axes:[point(&axes[0]),point(&axes[1]),point(&axes[2])],radii,extra:0.};
  return if nodes.is_empty(){OwnedCurve::Analytic(Analytic::capture(input).unwrap())}else{OwnedCurve::Derived(Tree::capture(LeafInput::Analytic(input),dimension,&nodes).unwrap())};
 }
 let poles:Vec<_>=rows(field(data,"poles_xyz")).iter().map(point).collect();
 let weights=match field(data,"weights"){Json::Other=>None,v=>Some(rows(v).iter().map(number).collect::<Vec<_>>())};
 let bezier=field(data,"kind")==&Json::String("bezier_curve".into());
 let(degree,knots,multiplicities,periodic,domain)=if bezier{
  let degree=number(field(data,"degree"))as usize;(degree,vec![0.,1.],vec![degree+1;2],false,[0.,1.])
 }else{
  let a=field(data,"axis");let d=rows(field(a,"full_domain"));
  (number(field(a,"degree"))as usize,rows(field(a,"knots")).iter().map(number).collect(),rows(field(a,"multiplicities")).iter().map(|v|number(v)as usize).collect(),field(a,"periodic")==&Json::Boolean(true),[number(&d[0]),number(&d[1])])
 };
 let basis=CurveBasis{poles:&poles,weights:weights.as_deref(),axis:Axis{degree,knots:&knots,multiplicities:&multiplicities,periodic,domain}};
 if nodes.is_empty(){if bezier{OwnedCurve::Bezier(BezierCurve::capture(basis).unwrap())}else{OwnedCurve::Nurbs(Curve::capture(basis).unwrap())}}
 else{OwnedCurve::Derived(Tree::capture(if bezier{LeafInput::BezierCurve(basis)}else{LeafInput::NurbsCurve(basis)},dimension,&nodes).unwrap())}
}
#[test]
fn selected_owned_edge_fraction_and_world_d1_match_nineteen_native_families(){
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"),1024*1024,150000).unwrap();
 let mut count=0;
 for case in rows(&cases){
  // Synthetic placement around genuine captured native-local geometry. Expected
  // values are the native oracle rotated + translated, never the Rust evaluator.
  let edge=crate::prepared_result::map([
   ("native_topology_index",Json::String("1".into())),("domain",field(case,"domain").clone()),
   ("curve_present",Json::Boolean(true)),("degenerate",Json::Boolean(false)),
   ("geometry_coordinates",Json::String("native_local".into())),
   ("native_location",parse_document("[[0,-1,0,11],[1,0,0,-7],[0,0,1,3],[0,0,0,1]]",1000,100).unwrap()),
   ("geometry_data",field(case,"geometry_data").clone())]);
  let owned=crate::owned_curve_input::SelectedEdge::capture(&edge).unwrap();
  let domain=rows(field(case,"domain"));let(lo,hi)=(number(&domain[0]),number(&domain[1]));
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let(p,d)=owned.at_fraction((u-lo)/(hi-lo)).unwrap();
   let q=point(field(query,"value"));let t=point(field(query,"derivative"));let length=t[0].hypot(t[1]).hypot(t[2]);
   near(p,[11.-q[1],q[0]-7.,q[2]+3.],field(case,"name"),u,"world point");
   near(d,[-t[1]/length,t[0]/length,t[2]/length],field(case,"name"),u,"world unit D1");count+=1;
  }
  for fraction in [-0.01,1.01,f64::NAN,f64::INFINITY]{assert!(owned.at_fraction(fraction).is_err());}
 }
 assert_eq!(count,183);
}
fn native_oracle_edge(index:usize)->Json{
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"),1024*1024,150000).unwrap();let case=&rows(&cases)[index];
 crate::prepared_result::map([("domain",field(case,"domain").clone()),("curve_present",Json::Boolean(true)),("degenerate",Json::Boolean(false)),("geometry_coordinates",Json::String("native_local".into())),("native_location",parse_document("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]",1000,100).unwrap()),("geometry_data",field(case,"geometry_data").clone())])
}
#[test]
fn selected_owned_edge_rejects_an_edge_domain_outside_the_retained_basis(){
 let mut edge=native_oracle_edge(0);let Json::Object(e)=&mut edge else{panic!("edge")};e.insert("domain".into(),Json::Array(vec![Json::Number(-1.),Json::Number(2.)]));
 assert!(crate::owned_curve_input::SelectedEdge::capture(&edge).is_err(),"an individually evaluable middle query must not conceal a false edge restriction");
}
#[test]
fn selected_owned_edge_rejects_malformed_basis_before_evaluation(){
 for key in ["degree","weights","multiplicities","knots","frame","kind","count"]{
  let mut edge=native_oracle_edge(0);let Json::Object(e)=&mut edge else{panic!("edge")};
  if key=="frame"{e.insert("native_location".into(),parse_document("[[1,0,0,0],[0,0,0,0],[0,0,1,0],[0,0,0,1]]",1000,100).unwrap());}
  else{
   let Json::Object(data)=e.get_mut("geometry_data").unwrap()else{panic!("data")};
   match key{
    "weights"=>{data.insert("weights".into(),Json::Array(vec![Json::Number(1.)]));},
    "count"=>{data.insert("poles_xyz".into(),Json::Array(vec![Json::Array(vec![Json::Number(0.);3]);8193]));},
    "kind"=>{data.insert("kind".into(),Json::String("analytic_surface".into()));},
    _=>{let Json::Object(axis)=data.get_mut("axis").unwrap()else{panic!("axis")};axis.insert(key.into(),match key{"degree"=>Json::Number(2.5),"multiplicities"=>Json::Array(vec![Json::Number(3.),Json::Number(2.5)]),_=>Json::Array(vec![Json::Number(0.),Json::Number(0.)])});},
   }
  }
  assert!(crate::owned_curve_input::SelectedEdge::capture(&edge).is_err(),"malformed {key}");
 }
}
#[test]
fn owned_curve_jets_match_native_value_and_three_derivatives_for_nineteen_families(){
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"),1024*1024,150000).unwrap();
 let mut count=0;
 for case in rows(&cases){
  let owned=capture_owned(field(case,"geometry_data"));
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let coefficients=owned.taylor(u,3).unwrap();assert_eq!(coefficients.len(),4);
   for(order,(key,factorial))in [("value",1.),("derivative",1.),("second",2.),("third",6.)].iter().enumerate(){
    near(coefficients[order].map(|v|v*factorial),point(field(query,key)),field(case,"name"),u,key);
   }
   count+=1;
  }
 }
 assert_eq!(rows(&cases).len(),19);assert_eq!(count,183);
}
#[test]
fn owned_offset_chain_evaluates_full_depth_and_rejects_singular_normal_and_outside_trim(){
 use crate::derived_support::{Tree,DerivedNode,LeafInput,MAX_DEPTH};use crate::analytic_support::AnalyticInput;
 let circle=AnalyticInput{kind:2,dimension:3,origin:[0.;3],axes:[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]],radii:[10.,0.],extra:0.};
 let offset=DerivedNode{kind:1,dimension:3,direction:[0.,0.,1.],origin:[0.;3],offset:0.1,bounds:[0.,std::f64::consts::TAU,0.,0.],infinite_mask:0,reserved:0};
 let tree=Tree::capture(LeafInput::Analytic(circle),3,&vec![offset;MAX_DEPTH]).unwrap();let before=tree.json().unwrap();
 let(p,d)=tree.evaluate_curve(0.).unwrap();near(p,[11.6,0.,0.],&Json::String("depth16".into()),0.,"point");near(d,[0.,11.6,0.],&Json::String("depth16".into()),0.,"D1");assert_eq!(tree.json().unwrap(),before);
 assert!(tree.taylor_curve(0.,usize::MAX).is_err());
 let trim=DerivedNode{kind:5,direction:[0.;3],offset:0.,bounds:[0.2,0.8,0.,0.],..offset};
 let tree=Tree::capture(LeafInput::Analytic(circle),3,&[trim,offset]).unwrap();assert!(tree.evaluate_curve(0.1).is_err());assert!(tree.evaluate_curve(0.5).is_ok());
 let line=AnalyticInput{kind:1,axes:[[0.,0.,1.],[0.;3],[0.;3]],radii:[0.;2],..circle};
 let singular=Tree::capture(LeafInput::Analytic(line),3,&[offset]).unwrap();assert!(singular.evaluate_curve(0.5).is_err());
}
#[test]
fn owned_derived_value_first_and_second_derivatives_match_all_native_queries(){
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"),1024*1024,150000).unwrap();let mut count=0;
 for case in rows(&cases){
  let owned=capture_owned(field(case,"geometry_data"));
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let c=owned.taylor(u,2).unwrap();
   for(k,(key,factorial))in [("value",1.),("derivative",1.),("second",2.)].iter().enumerate(){near(c[k].map(|v|v*factorial),point(field(query,key)),field(case,"name"),u,key);}
   count+=1;
  }
 }
 assert_eq!(count,183);
}
#[test]
fn owned_offset_third_derivative_matches_independent_high_precision_definition(){
 let cases=parse_document(include_str!("../tests/fixtures/owned_curve_evaluation/independent_offset_third.json"),1024*1024,150000).unwrap();let mut count=0;let mut native_mismatches=0;
 for case in rows(&cases){let owned=capture_owned(field(case,"geometry_data"));
  for query in rows(field(case,"queries")){
   let u=number(field(query,"parameter"));let c=owned.taylor(u,3).unwrap();near(c[3].map(|v|v*6.),point(field(query,"third")),field(case,"name"),u,"independent third");count+=1;
   native_mismatches+=usize::from(field(query,"native_third_matches")==&Json::Boolean(false));
  }
 }
 assert_eq!(count,45);assert_eq!(native_mismatches,36);
}
#[test]
fn owned_uv_offset_preserves_native_clockwise_normal_and_rejects_surface_chain(){
 use crate::derived_support::{Tree,DerivedNode,LeafInput};use crate::analytic_support::AnalyticInput;
 let line=AnalyticInput{kind:1,dimension:2,origin:[0.;3],axes:[[1.,0.,0.],[0.;3],[0.;3]],radii:[0.;2],extra:0.};
 let node=DerivedNode{kind:1,dimension:2,direction:[0.;3],origin:[0.;3],offset:2.,bounds:[0.;4],infinite_mask:3,reserved:0};
 let uv=Tree::capture(LeafInput::Analytic(line),2,&[node]).unwrap();assert_eq!(uv.evaluate_curve(3.).unwrap(),([3.,-2.,0.],[1.,0.,0.]));
 let extrusion=DerivedNode{kind:3,dimension:3,direction:[0.,0.,1.],offset:0.,infinite_mask:15,..node};
 let surface=Tree::capture(LeafInput::Analytic(AnalyticInput{dimension:3,..line}),3,&[extrusion]).unwrap();assert!(surface.evaluate_curve(3.).is_err());
}
