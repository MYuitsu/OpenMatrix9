use openmatrix9_rust::spline::{self, Knots};
fn near(a: [f64;3], b: [f64;3], tol: f64) {assert!(spline::distance(a,b)<tol,"{a:?} != {b:?}");}
#[test]
fn om9_curve_003_interpolates_irregular_3d_points_in_all_parameterizations() {
    let points=[[0.,0.,0.],[0.1,2.,3.],[5.,-1.,2.],[12.,4.,-3.],[14.,0.,0.],[16.,2.,1.]];
    for degree in [1,3,5] {for mode in [Knots::Uniform,Knots::Chord,Knots::SqrtChord] {
        let curve=spline::interpolate(&points,degree,mode,false).unwrap();
        for (&u,&p) in spline::parameters(&points,mode,false).iter().zip(&points) {near(curve.value(u),p,1e-8);}
        assert_eq!(curve.degree,degree);assert!(!curve.periodic);
    }}
}
#[test]
fn om9_curve_003_smooth_close_interpolates_and_matches_seam_tangents() {
    let points=[[0.,0.,0.],[3.,0.,1.],[4.,4.,0.],[0.,2.,-1.],[-2.,1.,0.]];
    for mode in [Knots::Uniform,Knots::Chord,Knots::SqrtChord] {
        let curve=spline::interpolate(&points,3,mode,true).unwrap();
        for (&u,&p) in spline::parameters(&points,mode,true).iter().zip(&points) {near(curve.value(u),p,1e-8);}
        near(curve.value(0.),curve.value(1.),1e-10);
        let eps=1e-6;
        let a=curve.value(eps);let b=curve.value(1.-eps);let seam=curve.value(0.);
        near(std::array::from_fn(|i| (a[i]-seam[i])/eps),std::array::from_fn(|i| (seam[i]-b[i])/eps),1e-3);
    }
}
#[test]
fn om9_curve_009_has_exact_requested_structure_and_preserves_line_endpoints() {
    let samples=(0..=128).map(|i| {let u=i as f64/128.;[10.+30.*u,4.-12.*u,9.*u]}).collect::<Vec<_>>();
    for degree in [1,2,3,5,11] {
        let curve=spline::rebuild(&samples,16,degree,false).unwrap();
        assert_eq!(curve.degree,degree);assert_eq!(curve.poles.len(),16);
        near(curve.value(0.),[10.,4.,0.],1e-10);near(curve.value(1.),[40.,-8.,9.],1e-10);
        for i in 0..=32 {let u=i as f64/32.;near(curve.value(u),[10.+30.*u,4.-12.*u,9.*u],1e-6);}
        let (knots,mults)=curve.host_knots();assert_eq!(mults[0],degree+1);assert_eq!(*mults.last().unwrap(),degree+1);
        assert!(knots.windows(3).all(|k| ((k[1]-k[0])-(k[2]-k[1])).abs()<1e-12));
    }
}
#[test]
fn om9_curve_009_periodic_rebuild_preserves_a_circle_and_exact_pole_count() {
    let samples=(0..=256).map(|i| {let t=std::f64::consts::TAU*i as f64/256.;[5.*t.cos(),5.*t.sin(),2.]}).collect::<Vec<_>>();
    let curve=spline::rebuild(&samples,16,3,true).unwrap();
    assert_eq!(curve.poles.len(),16);assert!(curve.periodic);
    for i in 0..=128 {let p=curve.value(i as f64/128.);assert!((p[0].hypot(p[1])-5.).abs()<0.001);assert!((p[2]-2.).abs()<1e-8);}
    let (knots,mults)=curve.host_knots();assert_eq!(knots.len(),17);assert!(mults.iter().all(|&m| m==1));
}
#[test]
fn invalid_geometry_and_options_are_rejected_before_fitting() {
    assert!(spline::interpolate(&[[0.;3],[0.;3]],3,Knots::Uniform,false).is_err());
    assert!(spline::interpolate(&[[0.;3],[f64::NAN,1.,2.]],3,Knots::Chord,false).is_err());
    assert!(spline::rebuild_options(3,3).is_err());assert!(spline::rebuild_options(257,3).is_err());
    assert!(spline::rebuild_options(12,12).is_err());
    assert!(spline::rebuild(&[[0.;3],[1.;3]],4,3,false).is_err());
}
