use openmatrix9_rust::analytic_support::{Analytic,AnalyticInput,Error};
use openmatrix9_rust::support_snapshot_ffi::om9_support_analytic_json;
fn input(kind:u32)->AnalyticInput {
    AnalyticInput{kind,dimension:3,origin:[1.,2.,3.],axes:[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]],radii:[0.;2],extra:0.}
}
#[test]
fn owned_model_keeps_every_native_kind_and_unbounded_domains() {
    for kind in 1..=10 {
        let mut value=input(kind);
        match kind {1=>{value.axes[1]=[0.;3];value.axes[2]=[0.;3];},2|7|9=>value.radii[0]=3.,
            3|4|10=>value.radii=[3.,2.],5=>value.radii[0]=2.,8=>{value.radii[0]=0.;value.extra=-0.3;},_=>{}}
        let data=Analytic::capture(value).unwrap();
        let json=data.json().unwrap();
        assert!(json.contains("\"parameter_convention\":\"occt_native\""));
        assert!(!json.contains("nurbs"));
        if kind==1 || kind==6 {assert!(json.contains("\"unbounded\""));}
    }
}
#[test]
fn indirect_frames_and_uv_embedding_are_preserved_not_normalized() {
    let mut value=input(2);value.dimension=2;value.origin[2]=0.;value.axes[1][1]=-1.;value.axes[2][2]=-1.;value.radii[0]=3.;
    let json=Analytic::capture(value).unwrap().json().unwrap();
    assert!(json.contains("\"axes_xyz\":[[1,0,0],[0,-1,0],[0,0,-1]]"));
    assert!(json.contains("\"frame_handedness\":1"));
    assert!(json.contains("\"uv_sense\":-1"));
    value.axes[1][1]=1.;value.axes[2][2]=1.;
    assert!(Analytic::capture(value).unwrap().json().unwrap().contains("\"uv_sense\":1"));
    let mut plane=input(6);plane.axes[1][1]=-1.;
    let json=Analytic::capture(plane).unwrap().json().unwrap();
    assert!(json.contains("\"frame_handedness\":-1"));
    assert!(json.contains("\"uv_sense\":null"));
}
#[test]
fn malformed_kind_frame_dimension_and_parameters_reject() {
    let mut v=input(6);v.kind=0;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Kind);
    v=input(6);v.origin[0]=f64::NAN;assert_eq!(Analytic::capture(v).unwrap_err(),Error::NonFinite);
    v=input(6);v.axes[1]=v.axes[0];assert_eq!(Analytic::capture(v).unwrap_err(),Error::Frame);
    v=input(6);v.dimension=2;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Dimension);
    v=input(2);v.radii[0]=-1.;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Parameters);
    v=input(6);v.extra=1.;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Parameters);
    v=input(2);v.dimension=2;v.origin[2]=0.;v.axes[0][2]=0.1;
    assert_eq!(Analytic::capture(v).unwrap_err(),Error::Dimension);
    v=input(2);v.axes[0][0]=2.;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Frame);
    v=input(6);v.radii[0]=1.;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Parameters);
    v=input(3);v.radii=[2.,3.];assert_eq!(Analytic::capture(v).unwrap_err(),Error::Parameters);
    for angle in [0.,std::f64::consts::FRAC_PI_2,-std::f64::consts::FRAC_PI_2] {
        v=input(8);v.extra=angle;assert_eq!(Analytic::capture(v).unwrap_err(),Error::Parameters);
    }
}
#[test]
fn value_abi_has_exact_output_atomic_errors_and_no_retained_input() {
    assert_eq!((std::mem::size_of::<AnalyticInput>(),std::mem::align_of::<AnalyticInput>()),(128,8));
    assert_eq!(std::mem::offset_of!(AnalyticInput,axes),32);
    let mut v=input(6);let q=unsafe{om9_support_analytic_json(v,std::ptr::null_mut(),0)};
    assert_eq!(q.status,0);let mut bytes=vec![0x5a;q.needed+1];
    let copied=unsafe{om9_support_analytic_json(v,bytes.as_mut_ptr(),q.needed)};
    assert_eq!((copied.status,copied.needed),(0,q.needed));assert_eq!(bytes[q.needed],0x5a);
    let before=bytes.clone();v.origin[0]=f64::INFINITY;
    assert_ne!(unsafe{om9_support_analytic_json(v,bytes.as_mut_ptr(),q.needed)}.status,0);
    assert_eq!(bytes,before);
    assert_eq!(unsafe{om9_support_analytic_json(input(6),bytes.as_mut_ptr(),q.needed-1)}.status,1001);
    assert_eq!(bytes,before);
}
