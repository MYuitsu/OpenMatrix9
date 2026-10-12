// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{curve::{CurveSession,Effect},builder_history::{Json,parse_document}};
fn evidence(s:&CurveSession)->Json{parse_document(&s.construction_evidence(1).unwrap(),16*1024*1024,1_000_000).unwrap()}
fn field<'a>(v:&'a Json,key:&str)->&'a Json{let Json::Object(o)=v else{panic!("object")};o.get(key).unwrap_or_else(||panic!("missing {key}"))}
fn events(v:&Json)->&[Json]{let Json::Array(a)=field(v,"accepted_input_events")else{panic!("events")};a}
fn text(v:&Json)->&str{let Json::String(s)=v else{panic!("text")};s}
fn point(x:f64,y:f64,z:f64)->Json{Json::Array(vec![Json::Number(x),Json::Number(y),Json::Number(z)])}
#[test]
fn rectangle_retains_original_direction_before_length_normalization(){
 let mut s=CurveSession::default();s.start("Rectangle").unwrap();
 for input in ["3Point","0,0,0","Length=5","10,0,0","0,3,0"]{s.input(input).unwrap();}
 assert_eq!(s.points()[1],[5.,0.,0.]);let v=evidence(&s);let a=events(&v);assert_eq!(a.len(),5);
 assert_eq!(text(field(&a[2],"text")),"Length=5");
 assert_eq!(field(&a[3],"resolved_point_world"),&point(10.,0.,0.));
 assert_eq!(text(field(&a[3],"text")),"10,0,0");
}
#[test]
fn scalar_text_pending_option_units_and_frame_survive_synthetic_axis_points(){
 let mut s=CurveSession::default();s.start("Ellipse").unwrap();s.set_input_scale(10.).unwrap();
 s.set_frame([10.,20.,30.],[[0.,1.,0.],[-1.,0.,0.],[0.,0.,1.]]).unwrap();
 for input in ["Degree","3","0,0,0","0.5cm","3mm"]{s.input(input).unwrap();}
 let v=evidence(&s);let a=events(&v);assert_eq!(a.len(),5);
 assert_eq!(text(field(&a[1],"pending_option")),"Degree");
 assert_eq!(text(field(&a[3],"text")),"0.5cm");assert_eq!(field(&a[3],"input_mm_per_unit"),&Json::Number(10.));
 assert_eq!(field(&a[3],"frame_origin"),&point(10.,20.,30.));assert_eq!(field(&a[3],"resolved_point_world"),&Json::Other);
 let mut s=CurveSession::default();s.start("Circle").unwrap();
 for input in ["0,0,0","Diameter","2cm"]{s.input(input).unwrap();}
 let v=evidence(&s);let a=events(&v);assert_eq!(a.len(),3);assert_eq!(text(field(&a[2],"circle_size")),"Diameter");
 assert_eq!(text(field(&a[2],"text")),"2cm");assert_eq!(s.circle().unwrap().radius,10.);
}
#[test]
fn mouse_and_relative_points_are_retained_before_constraint_projection(){
 let mut s=CurveSession::default();s.start("Polyline").unwrap();s.point([0.,0.,0.]).unwrap();
 s.mouse_point([3.,4.,0.],true).unwrap();s.input("r1,0,0").unwrap();assert_eq!(s.input("").unwrap(),Effect::Commit);
 let v=evidence(&s);let a=events(&v);assert_eq!(a.len(),4);
 assert_eq!(text(field(&a[1],"kind")),"mouse_point");assert_eq!(field(&a[1],"point_world"),&point(3.,4.,0.));
 assert_eq!(field(&a[1],"ortho"),&Json::Boolean(true));assert_eq!(s.points()[1],[0.,4.,0.]);
 assert_eq!(field(&a[2],"previous_point_world"),&point(0.,4.,0.));assert_eq!(field(&a[2],"resolved_point_world"),&point(1.,4.,0.));
}
#[test]
fn circle_undo_removes_an_entire_owned_batch_and_invalid_inputs_add_no_event(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 s.circle_fit_batch(&[[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.]]).unwrap();s.input("Undo").unwrap();
 assert!(s.circle_fit_batch(&[[f64::NAN,0.,0.]]).is_err());
 s.circle_fit_batch(&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]]).unwrap();s.input("").unwrap();
 let v=evidence(&s);let a=events(&v);assert_eq!(a.len(),3);assert_eq!(text(field(&a[1],"kind")),"native_fit_batch");
 assert_eq!(field(&a[1],"points_world"),&Json::Array(vec![point(1.,0.,0.),point(0.,1.,0.),point(-1.,0.,0.)]));
 assert!((s.circle().unwrap().radius-1.).abs()<1e-12);
 assert_eq!(text(field(&a[0],"circle_context_before")),"0");assert_eq!(text(field(&a[0],"circle_context_after")),"1");
 assert_eq!(text(field(&a[1],"circle_context_after")),"3");
}
#[test]
fn accepted_event_budget_rejects_before_mutation_and_cancel_reclaims_owned_log(){
 let mut s=CurveSession::default();s.start("InterpCrv").unwrap();let mut rejected=false;
 for _ in 0..8193{let before=s.prompt();if s.input("Degree=1").is_err(){assert_eq!(s.prompt(),before);rejected=true;break;}}
 assert!(rejected,"repeated accepted options need a bounded owned log");
 s.cancel();s.start("Line").unwrap();s.point([0.,0.,0.]).unwrap();s.point([1.,0.,0.]).unwrap();
 assert_eq!(events(&evidence(&s)).len(),2);
}
#[test]
fn ignored_native_reference_fields_must_be_finite_before_session_or_log_mutation(){
 for name in ["Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(name).unwrap();s.input("AroundCurve").unwrap();
  for invalid in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY]{
   let before_prompt=s.prompt();let before_log=s.accepted_input_log.json();
   let before_keys=s.circle_input_context_keys();
   let result=if name=="Circle"{s.circle_native_reference([0.;3],[invalid,0.,0.],1)}
              else{s.ellipse_native_reference([0.;3],[invalid,0.,0.],1)};
   assert!(result.as_ref().is_err_and(|e|e.starts_with("NonFiniteGeometry:")),"{name} retained nonfinite native reference: {result:?}");
   assert_eq!(s.prompt(),before_prompt);assert_eq!(s.accepted_input_log.json(),before_log);
   assert_eq!(s.circle_input_context_keys(),before_keys);
   if name=="Ellipse"{
    let result=s.ellipse_native_reference([invalid,0.,0.],[0.,0.,1.],1);
    assert!(result.as_ref().is_err_and(|e|e.starts_with("NonFiniteGeometry:")));
    assert_eq!(s.prompt(),before_prompt);assert_eq!(s.accepted_input_log.json(),before_log);
   }
  }
  if name=="Circle"{
   s.circle_native_reference([0.;3],[0.,0.,1.],1).unwrap();
   s.circle_native_reference([0.;3],[0.,0.,1.],2).unwrap();s.input("5mm").unwrap();
   assert_eq!(s.circle().unwrap().radius,5.);
  }else{
   s.ellipse_native_reference([0.;3],[0.,0.,1.],1).unwrap();
   s.ellipse_native_reference([0.;3],[0.,0.,1.],2).unwrap();s.input("5mm").unwrap();s.input("3mm").unwrap();
  }
  let v=evidence(&s);assert_eq!(events(&v).len(),if name=="Circle"{4}else{5});
  assert_eq!(text(field(&events(&v)[1],"kind")),"native_reference");
 }
}
