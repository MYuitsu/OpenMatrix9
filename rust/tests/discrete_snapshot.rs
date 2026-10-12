use openmatrix9_rust::discrete_snapshot::{Geometry,Error};
use openmatrix9_rust::discrete_snapshot_ffi::{Input,om9_discrete_counts,om9_discrete_json};
#[test]
fn mesh_and_cloud_own_local_arrays_and_inspected_empty_bounds() {
    let mut points=[[1.,2.,3.],[4.,2.,3.],[1.,6.,3.]];
    let mut faces=[[0,1,2]];
    let mesh=Geometry::capture(8,&points,&faces).unwrap();
    let cloud=Geometry::capture(9,&points,&[]).unwrap();
    points[0]=[99.;3];faces[0]=[2,2,2];
    assert_eq!(points[0],[99.;3]);assert_eq!(faces[0],[2,2,2]);
    assert_eq!(mesh.points()[0],[1.,2.,3.]);assert_eq!(mesh.faces(),&[[0,1,2]]);
    assert_eq!(cloud.points(),mesh.points());
    assert!(mesh.json().unwrap().contains("\"bounds_xyz\":[[1,2,3],[4,6,3]]"));
    assert!(cloud.json().unwrap().contains("\"triangles\":null"));
    for kind in [8,9]{let json=Geometry::capture(kind,&[],&[]).unwrap().json().unwrap();assert!(json.contains("\"points_xyz\":[]"));assert!(json.contains("\"bounds_xyz\":null"));}
}
#[test]
fn malformed_discrete_values_indices_kinds_and_budgets_reject() {
    let points=[[0.;3],[1.,0.,0.],[0.,1.,0.]];
    for value in [f64::NAN,f64::INFINITY,1e151]{let mut bad=points;bad[1][0]=value;assert_eq!(Geometry::capture(8,&bad,&[[0,1,2]]).unwrap_err(),Error::Coordinate);}
    for face in [[0,1,3],[0,0,2],[u64::MAX,1,2]]{assert_eq!(Geometry::capture(8,&points,&[face]).unwrap_err(),Error::Face);}
    assert_eq!(Geometry::capture(9,&points,&[[0,1,2]]).unwrap_err(),Error::Kind);
    for kind in [0,1,7,10,11,u32::MAX]{assert_eq!(Geometry::capture(kind,&points,&[]).unwrap_err(),Error::Kind);}
    assert_eq!(Geometry::capture(9,&vec![[0.;3];100001],&[]).unwrap_err(),Error::Count);
}
#[test]
fn discrete_abi_live_envelopes_and_short_output_are_atomic() {
    assert_eq!(std::mem::size_of::<Input>(),40);assert_eq!(std::mem::align_of::<Input>(),8);
    assert_eq!(om9_discrete_counts(8,100000,100000),0);assert_eq!(om9_discrete_counts(9,0,0),0);
    assert_ne!(om9_discrete_counts(8,100001,1),0);assert_ne!(om9_discrete_counts(8,3,100001),0);assert_ne!(om9_discrete_counts(9,3,1),0);
    let points=[[0.;3],[1.,0.,0.],[0.,1.,0.]];let faces=[[0,1,2]];
    let input=Input{representation:8,reserved:0,points:points.as_ptr(),point_count:3,faces:faces.as_ptr(),face_count:1};
    let call=|i,out,cap|unsafe{om9_discrete_json(i,out,cap)};
    let query=call(input,std::ptr::null_mut(),0);assert_eq!(query.status,0);
    let mut bytes=vec![0x5a;query.needed+1];assert_eq!(call(input,bytes.as_mut_ptr(),query.needed).status,0);let saved=bytes.clone();
    assert_eq!(bytes[query.needed],0x5a);assert_eq!(call(input,bytes.as_mut_ptr(),query.needed-1).status,1001);assert_eq!(bytes,saved);
    for bad in [Input{point_count:usize::MAX,..input},Input{points:std::ptr::null(),..input},Input{points:(points.as_ptr() as *const u8).wrapping_add(1) as _,..input},Input{reserved:1,..input},Input{faces:std::ptr::null(),..input}]{assert_ne!(call(bad,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,saved);}
    assert_eq!(call(Input{representation:9,points:std::ptr::null(),point_count:0,faces:std::ptr::null(),face_count:0,reserved:0},std::ptr::null_mut(),0).status,0);
}
#[test]
fn invalid_values_and_serialized_overflow_preserve_live_ffi_output() {
    let points=[[0.;3],[1.,0.,0.],[0.,1.,0.]];
    let faces=[[0,1,2]];
    let mut output=[0x5a;256];let saved=output;
    let input=Input{representation:8,reserved:0,points:points.as_ptr(),point_count:3,faces:faces.as_ptr(),face_count:1};
    for value in [f64::NAN,f64::INFINITY,1e151]{
        let mut invalid=points;invalid[1][0]=value;
        let result=unsafe{om9_discrete_json(Input{points:invalid.as_ptr(),..input},output.as_mut_ptr(),output.len())};
        assert_eq!(result.status,7001);assert_eq!(result.needed,0);assert_eq!(output,saved);
    }
    for face in [[0,1,3],[0,0,2],[u64::MAX,1,2]]{
        let invalid=[face];
        let result=unsafe{om9_discrete_json(Input{faces:invalid.as_ptr(),..input},output.as_mut_ptr(),output.len())};
        assert_eq!(result.status,7001);assert_eq!(result.needed,0);assert_eq!(output,saved);
    }
    let large=vec![[1e149;3];100_000];
    let large_input=Input{representation:9,points:large.as_ptr(),point_count:large.len(),faces:std::ptr::null(),face_count:0,..input};
    assert_eq!(om9_discrete_counts(9,large.len(),0),0);
    let result=unsafe{om9_discrete_json(large_input,output.as_mut_ptr(),output.len())};
    assert_eq!(result.status,1000);assert_eq!(result.needed,0);assert_eq!(output,saved);
}
