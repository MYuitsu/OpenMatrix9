use openmatrix9_rust::support_snapshot_ffi::*;
use openmatrix9_rust::nurbs_basis_ffi::{NativeAxis,NativeCurve,NativeSurface};
static KNOTS:[f64;2]=[0.,1.];
static MULTS:[usize;2]=[2,2];
fn curve(p:&[[f64;3]])->NativeCurve {
    NativeCurve{poles:p.as_ptr(),pole_count:p.len(),weights:std::ptr::null(),weight_count:0,
      axis:NativeAxis{degree:1,knots:KNOTS.as_ptr(),knot_count:2,multiplicities:MULTS.as_ptr(),multiplicity_count:2,periodic:0,domain:[0.,1.]}}
}
#[test]
fn exact_sized_output_and_query_copy_complete_valid_json() {
    assert_eq!(std::mem::size_of::<JsonSummary>(),16);
    let p=[[0.,0.,0.],[1.,2.,3.]];let c=curve(&p);
    let query=unsafe{om9_support_curve_json(c,std::ptr::null_mut(),0)};
    assert_eq!(query.status,0);assert!(query.needed>0);
    let mut output=vec![0x5a;query.needed];
    let copied=unsafe{om9_support_curve_json(c,output.as_mut_ptr(),output.len())};
    assert_eq!((copied.status,copied.needed),(0,query.needed));
    assert_eq!(String::from_utf8(output).unwrap(),"{\"schema_version\":1,\"kind\":\"nurbs_curve\",\"poles_xyz\":[[0,0,0],[1,2,3]],\"weights\":null,\"axis\":{\"degree\":1,\"knots\":[0,1],\"multiplicities\":[2,2],\"periodic\":false,\"full_domain\":[0,1]}}");
}
#[test]
fn invalid_basis_output_storage_and_short_capacity_are_atomic() {
    let p=[[0.,0.,0.],[1.,2.,3.]];let c=curve(&p);let mut output=[0x5a;16];
    assert_ne!(unsafe{om9_support_curve_json(c,output.as_mut_ptr(),output.len())}.status,0);
    let mut bad=c;bad.poles=std::ptr::null();
    assert_ne!(unsafe{om9_support_curve_json(bad,output.as_mut_ptr(),output.len())}.status,0);
    bad=c;bad.axis.domain=[1.,0.];
    assert_ne!(unsafe{om9_support_curve_json(bad,output.as_mut_ptr(),output.len())}.status,0);
    assert_ne!(unsafe{om9_support_curve_json(c,std::ptr::null_mut(),16)}.status,0);
    assert_ne!(unsafe{om9_support_curve_json(c,output.as_mut_ptr(),usize::MAX)}.status,0);
    assert_eq!(output,[0x5a;16]);
}
#[test]
fn counts_and_aggregate_serialized_budget_use_checked_arithmetic() {
    assert_eq!(om9_support_counts(2,0,2,0,0),0);
    assert_ne!(om9_support_counts(usize::MAX,2,2,2,1),0);
    assert_ne!(om9_support_counts(2,0,2,0,2),0);
    assert_eq!(om9_support_payload(100,200),0);
    assert_ne!(om9_support_payload(8*1024*1024,1),0);
    assert_ne!(om9_support_payload(usize::MAX,1),0);
}
#[test]
fn surface_json_copies_rational_u_major_data_and_rejects_atomically() {
    let poles=[[0.,0.,0.],[0.,1.,0.],[1.,0.,0.],[1.,1.,1.]];
    let weights=[1.,2.,3.,4.];
    let surface=NativeSurface{poles:poles.as_ptr(),pole_count:4,weights:weights.as_ptr(),weight_count:4,
        u_count:2,v_count:2,axes:[curve(&poles).axis;2]};
    let query=unsafe{om9_support_surface_json(surface,std::ptr::null_mut(),0)};
    assert_eq!(query.status,0);
    let mut output=vec![0x5a;query.needed+1];
    let copied=unsafe{om9_support_surface_json(surface,output.as_mut_ptr(),query.needed)};
    assert_eq!((copied.status,copied.needed),(0,query.needed));
    assert_eq!(output[query.needed],0x5a);
    let json=String::from_utf8(output[..query.needed].to_vec()).unwrap();
    assert!(json.contains("\"pole_layout\":\"u_major\",\"poles_xyz\":[[0,0,0],[0,1,0],[1,0,0],[1,1,1]],\"weights\":[1,2,3,4]"));
    let mut sentinel=vec![0x5a;query.needed];
    let mut malformed=surface;malformed.v_count=usize::MAX;
    assert_ne!(unsafe{om9_support_surface_json(malformed,sentinel.as_mut_ptr(),sentinel.len())}.status,0);
    malformed=surface;malformed.axes[1].periodic=2;
    assert_ne!(unsafe{om9_support_surface_json(malformed,sentinel.as_mut_ptr(),sentinel.len())}.status,0);
    malformed=surface;let invalid_weights=[1.,2.,f64::NAN,4.];malformed.weights=invalid_weights.as_ptr();
    assert_ne!(unsafe{om9_support_surface_json(malformed,sentinel.as_mut_ptr(),sentinel.len())}.status,0);
    assert_eq!(unsafe{om9_support_surface_json(surface,sentinel.as_mut_ptr(),query.needed-1)}.status,1001);
    assert_eq!(sentinel,vec![0x5a;query.needed]);
}
