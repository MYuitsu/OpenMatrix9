// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{circle::{Circle,Mode},circle_history::{Recipe,replay,om9_circle_history_capability as gate}};
use openmatrix9_rust::geometry_contract::*;

#[test]
fn replay_rejects_options_the_shared_circle_policy_rejects(){
    for options in [Circle{from_first:true,..Circle::default()},
        Circle{solution:Some(0),..Circle::default()},
        Circle{tangent_vertical:true,..Circle::default()}]{
        let recipe=Recipe::new(&options,Some(2.));
        assert!(replay(&recipe,&[[0.;3]],None).is_err(),"History ignored unsupported stored Circle options");
    }
}
#[test]
fn tangent_replay_uses_the_same_solution_budget_as_current_construction(){
    let options=Circle{mode:Mode::Tangent,solution:Some(10_000),radius_constraint:Some(2.),..Circle::default()};
    let recipe=Recipe::new(&options,None);
    assert!(replay(&recipe,&[[0.;3],[2.,0.,0.]],Some([2.,2.,0.,0.,0.,1.,2.])).is_err(),"History bypassed the shared solution budget");
}
#[test]
fn independent_query_uses_stored_modes_and_concrete_source_types(){unsafe{
    for mode in [Mode::Center,Mode::AroundCurve,Mode::Tangent,Mode::FitPoints]{
        let r=Recipe::new(&Circle{mode,..Circle::default()},None).encode();
        for kind in 1..=13{
            let allowed=kind==1||((kind==2||kind==3)&&mode!=Mode::Center)||(kind<=9&&mode==Mode::FitPoints);
            assert_eq!(gate(r.as_ptr(),r.len(),kind,2),if allowed{4}else{101},"{mode:?} type{kind}");
        }
        assert_eq!(gate(r.as_ptr(),r.len(),0,2),101);
    }
}}
#[test]
fn invalid_headers_and_budgets_are_rejected_before_foreign_array_access(){unsafe{
    for count in [0,23,usize::MAX]{assert_eq!(gate(std::ptr::null(),count,1,1),102);}
    let r=Recipe::new(&Circle::default(),Some(2.)).encode();
    assert_eq!(gate(r.as_ptr(),r.len(),1,0),102);
    assert_eq!(gate(r.as_ptr(),r.len(),1,1025),102);
    let mut corrupt=r;corrupt[18]=f64::NAN;assert_eq!(gate(corrupt.as_ptr(),corrupt.len(),1,1),102);
    let bytes=[0u8;193];let misaligned=bytes.as_ptr().add(1).cast();
    // The address is deliberately not aligned to a double.
    if misaligned as usize%8!=0{assert_eq!(gate(misaligned,24,1,1),102);}
}}
#[test]
fn replay_request_is_owned_and_does_not_modify_current_line(){
    unsafe extern "C"{fn om9_curve_start(text:*const std::ffi::c_char)->bool;fn om9_curve_cancel();}
    unsafe{om9_curve_cancel();assert!(om9_curve_start(c"Line".as_ptr()));}
    let mut r=Recipe::new(&Circle{mode:Mode::AroundCurve,..Circle::default()},Some(2.));
    let cap=r.construction_capability(RepresentationKind::Curve,1).unwrap();
    assert_eq!(cap.backend,Backend::OcctEdit);assert_eq!(cap.evidence_id,"OM9-CURVE-005");
    r.options.mode=Mode::Center;
    let OperationOptions::NativeCurve(request)=cap.options else{panic!("wrong shared request")};
    assert!(matches!(request.options,openmatrix9_rust::native_curve_capability::Options::Circle(Circle{mode:Mode::AroundCurve,..})));
    let encoded=Recipe::new(&Circle{mode:Mode::AroundCurve,..Circle::default()},Some(2.)).encode();
    assert_eq!(unsafe{gate(encoded.as_ptr(),encoded.len(),2,1)},4);
    assert_eq!(openmatrix9_rust::native_curve_capability::om9_native_curve_capability(2,2),101);
    assert_eq!(openmatrix9_rust::native_curve_capability::om9_native_curve_capability(1,2),4);
    unsafe{om9_curve_cancel();}
}
