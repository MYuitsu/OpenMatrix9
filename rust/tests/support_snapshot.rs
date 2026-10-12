use openmatrix9_rust::support_snapshot::*;
use openmatrix9_rust::nurbs_basis::{Axis,CurveBasis,SurfaceBasis,BasisError};

fn axis<'a>(knots:&'a[f64],multiplicities:&'a[usize])->Axis<'a> {
    Axis{degree:1,knots,multiplicities,periodic:false,domain:[0.,1.]}
}
#[test]
fn curve_json_is_owned_and_retains_xyz_weights_basis_domain() {
    let mut p=[[1.,0.,0.],[1.,1.,0.],[0.,1.,0.]];
    let w=[1.,std::f64::consts::FRAC_1_SQRT_2,1.];
    let a=Axis{degree:2,knots:&[0.,1.],multiplicities:&[3,3],periodic:false,domain:[0.,1.]};
    let s=Curve::capture(CurveBasis{poles:&p,weights:Some(&w),axis:a}).unwrap();
    p[0][0]=77.;
    assert_eq!(p[0][0],77.);
    let json=s.json().unwrap();
    assert_eq!(json,"{\"schema_version\":1,\"kind\":\"nurbs_curve\",\"poles_xyz\":[[1,0,0],[1,1,0],[0,1,0]],\"weights\":[1,0.7071067811865476,1],\"axis\":{\"degree\":2,\"knots\":[0,1],\"multiplicities\":[3,3],\"periodic\":false,\"full_domain\":[0,1]}}");
}
#[test]
fn rational_surface_json_keeps_u_major_and_separate_axes() {
    let p=[[0.,0.,0.],[0.,1.,0.],[1.,0.,0.],[1.,1.,1.]];
    let s=Surface::capture(SurfaceBasis{poles:&p,weights:Some(&[1.,2.,3.,4.]),u_count:2,v_count:2,
       axes:[axis(&[0.,1.],&[2,2]),axis(&[0.,1.],&[2,2])]}).unwrap();
    let j=s.json().unwrap();assert!(j.contains("\"pole_layout\":\"u_major\""));
    assert!(j.contains("\"poles_xyz\":[[0,0,0],[0,1,0],[1,0,0],[1,1,1]]"));
    assert!(j.contains("\"weights\":[1,2,3,4]"));
}
#[test]
fn periodic_basis_is_not_expanded_or_normalized() {
    let p=[[0.,0.,0.],[1.,0.,0.],[1.,1.,0.]];
    let s=Curve::capture(CurveBasis{poles:&p,weights:None,axis:Axis{degree:1,knots:&[0.,1.,2.,3.],multiplicities:&[1,1,1,1],periodic:true,domain:[0.,3.]}}).unwrap();
    assert!(s.json().unwrap().contains("\"periodic\":true,\"full_domain\":[0,3]"));
    assert!(s.json().unwrap().contains("\"weights\":null"));
}
#[test]
fn malformed_support_rejects_before_owned_clone() {
    let p=[[0.,0.,0.],[1.,0.,0.]];
    let b=CurveBasis{poles:&p,weights:Some(&[1.,f64::NAN]),axis:axis(&[0.,1.],&[2,2])};
    assert_eq!(Curve::capture(b).unwrap_err(),BasisError::Weights);
    assert!(support_counts_allowed(2,0,2,0,false));
    assert!(!support_counts_allowed(usize::MAX,0,2,0,false));
    assert!(!support_counts_allowed(10000,0,2,0,false));
    assert!(serialized_payload_allowed(1024,2048));
    assert!(!serialized_payload_allowed(usize::MAX,1));
}
#[test]
fn valid_basis_with_long_subnormal_decimals_hits_serialized_limit() {
    let n=8192;
    let poles=vec![[1e-307;3];n];
    let weights=vec![1e-307;n];
    let knots:Vec<_>=(0..n).map(|i|i as f64).collect();
    let mut mults=vec![1;n];mults[0]=2;mults[n-1]=2;
    let snapshot=Curve::capture(CurveBasis{poles:&poles,weights:Some(&weights),
        axis:Axis{degree:1,knots:&knots,multiplicities:&mults,periodic:false,domain:[0.,(n-1) as f64]}}).unwrap();
    assert_eq!(snapshot.json(),Err(SerializedLimit));
}
