use openmatrix9_rust::nurbs_basis::{Axis, BasisError, CurveBasis, SurfaceBasis};
use openmatrix9_rust::nurbs_basis_ffi::{NativeAxis, NativeCurve, NativeSurface};
use openmatrix9_rust::support_snapshot::{BezierCurve, BezierSurface};
use openmatrix9_rust::support_snapshot_ffi::{om9_support_bezier_curve_json, om9_support_bezier_surface_json};

fn axis()->Axis<'static> { Axis {degree:1,knots:&[0.,1.],multiplicities:&[2,2],periodic:false,domain:[0.,1.]} }
#[test]
fn owned_rational_curve_keeps_bezier_identity_weights_and_controls() {
    let mut poles=[[10.,0.,0.],[10.,10.,0.],[0.,10.,0.]];
    let weights=[1.,std::f64::consts::FRAC_1_SQRT_2,1.];
    let mut a=axis();a.degree=2;a.multiplicities=&[3,3];
    let data=BezierCurve::capture(CurveBasis{poles:&poles,weights:Some(&weights),axis:a}).unwrap();
    poles[0][0]=77.;assert_eq!(poles[0][0],77.);
    let json=data.json().unwrap();
    assert!(json.contains("\"kind\":\"bezier_curve\""));
    assert!(json.contains("\"poles_xyz\":[[10,0,0],[10,10,0],[0,10,0]]"));
    assert!(json.contains("\"weights\":[1,0.7071067811865476,1]"));
    assert!(json.contains("\"degree\":2,\"full_domain\":[0,1]"));
    assert!(!json.contains("knots"));
}
#[test]
fn owned_surface_preserves_rectangular_u_major_control_net() {
    let p=[[0.,0.,0.],[0.,1.,1.],[0.,2.,0.],[3.,0.,0.],[3.,1.,2.],[3.,2.,0.]];
    let mut v=axis();v.degree=2;v.multiplicities=&[3,3];
    let b=SurfaceBasis{poles:&p,weights:Some(&[1.,2.,3.,4.,5.,6.]),u_count:2,v_count:3,axes:[axis(),v]};
    let json=BezierSurface::capture(b).unwrap().json().unwrap();
    assert!(json.contains("\"u_count\":2,\"v_count\":3,\"pole_layout\":\"u_major\""));
    assert!(json.contains("\"weights\":[1,2,3,4,5,6]"));
    assert!(json.contains("\"degrees\":[1,2],\"full_domains\":[[0,1],[0,1]]"));
    assert!(!json.contains("nurbs"));
}
#[test]
fn arbitrary_nurbs_basis_cannot_be_mislabeled_as_bezier() {
    let p=[[0.;3];4];
    let multi=Axis{degree:2,knots:&[0.,0.5,1.],multiplicities:&[3,1,3],periodic:false,domain:[0.,1.]};
    assert!(BezierCurve::capture(CurveBasis{poles:&p,weights:None,axis:multi}).is_err());
    let periodic=Axis{degree:1,knots:&[0.,1.,2.,3.,4.],multiplicities:&[1;5],periodic:true,domain:[0.,4.]};
    assert!(BezierCurve::capture(CurveBasis{poles:&p,weights:None,axis:periodic}).is_err());
    let mut a=axis();a.domain=[0.2,0.8];
    assert_eq!(BezierCurve::capture(CurveBasis{poles:&p[..2],weights:None,axis:a}).unwrap_err(),BasisError::Domain);
    a=axis();a.knots=&[2.,3.];a.domain=[2.,3.];
    assert!(BezierCurve::capture(CurveBasis{poles:&p[..2],weights:None,axis:a}).is_err());
    a=axis();a.multiplicities=&[1,3];
    assert!(BezierCurve::capture(CurveBasis{poles:&p[..2],weights:None,axis:a}).is_err());
    assert_eq!(BezierCurve::capture(CurveBasis{poles:&p[..2],weights:Some(&[1.,f64::NAN]),axis:axis()}).unwrap_err(),BasisError::Weights);
    assert!(BezierSurface::capture(SurfaceBasis{poles:&p,weights:None,u_count:2,v_count:2,axes:[axis(),a]}).is_err());
}
#[test]
fn curve_and_surface_abi_share_bounded_atomic_output_without_retaining_inputs() {
    let p=[[0.;3],[1.;3]];let knots=[0.,1.];let mult=[2,2];
    let a=NativeAxis{degree:1,knots:knots.as_ptr(),knot_count:2,multiplicities:mult.as_ptr(),multiplicity_count:2,periodic:0,domain:[0.,1.]};
    let c=NativeCurve{poles:p.as_ptr(),pole_count:2,weights:std::ptr::null(),weight_count:0,axis:a};
    let grid=[[0.,0.,0.],[0.,1.,0.],[1.,0.,0.],[1.,1.,1.]];let w=[1.,2.,3.,4.];
    let s=NativeSurface{poles:grid.as_ptr(),pole_count:4,weights:w.as_ptr(),weight_count:4,u_count:2,v_count:2,axes:[a,a]};
    for surface in [false,true] {
        let call=|out,cap|unsafe{if surface {om9_support_bezier_surface_json(s,out,cap)}else{om9_support_bezier_curve_json(c,out,cap)}};
        let q=call(std::ptr::null_mut(),0);assert_eq!(q.status,0);
        let mut bytes=vec![0x5a;q.needed+1];let copied=call(bytes.as_mut_ptr(),q.needed);
        assert_eq!((copied.status,copied.needed),(0,q.needed));assert_eq!(bytes[q.needed],0x5a);
        let text=std::str::from_utf8(&bytes[..q.needed]).unwrap();assert!(text.contains("\"weights\":"));
        assert!(text.contains(if surface {"\"kind\":\"bezier_surface\""} else {"\"kind\":\"bezier_curve\""}));
        let before=bytes.clone();assert_eq!(call(bytes.as_mut_ptr(),q.needed-1).status,1001);assert_eq!(bytes,before);
        assert_eq!(call(std::ptr::null_mut(),1).status,1002);
        let mut bad_c=c;bad_c.axis.domain=[0.2,0.8];let mut bad_s=s;bad_s.axes[1].domain=[0.2,0.8];
        let bad=unsafe{if surface {om9_support_bezier_surface_json(bad_s,bytes.as_mut_ptr(),q.needed)}else{om9_support_bezier_curve_json(bad_c,bytes.as_mut_ptr(),q.needed)}};
        assert_ne!(bad.status,0);assert_eq!(bytes,before);
    }
}
