use openmatrix9_rust::gumball_associative::scale_factors;
use openmatrix9_rust::gumball::Frame;
use openmatrix9_rust::geometry_frame::IDENTITY;

#[test]
fn recovers_signed_scales_in_rotated_pivot_frame() {
    let h=0.5_f64.sqrt();
    let f=Frame::new([3.,-7.,2.],[[h,h,0.],[-h,h,0.],[0.,0.,1.]]).unwrap();
    let mut a=IDENTITY;
    let factors=[-2.,1.5,0.75];
    for r in 0..3 {for c in 0..3 {
        a[4*r+c]=(0..3).map(|k|f.axes[k][r]*factors[k]*f.axes[k][c]).sum();
    } a[4*r+3]=f.origin[r]-(0..3).map(|c|a[4*r+c]*f.origin[c]).sum::<f64>();}
    let got=scale_factors(a,f).unwrap();
    for i in 0..3 {assert!((got[i]-factors[i]).abs()<1e-12);}
}

#[test]
fn rejects_shear_projective_zero_nonfinite_and_wrong_pivot() {
    let f=Frame::default();
    for (index,value) in [(1,0.2),(12,0.1),(0,0.),(0,f64::NAN),(3,1.)] {
        let mut a=IDENTITY;a[index]=value;
        assert!(scale_factors(a,f).is_err(),"{index} {value}");
    }
}
