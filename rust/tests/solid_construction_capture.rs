// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::solid::{Session,Kind,Effect,om9_solid_construction_json,
    om9_solid_start,om9_solid_input,om9_solid_generation,om9_solid_cancel};

#[test]
fn accepted_box_capture_owns_normalized_output_and_exact_generation() {
    let mut s=Session::new(Kind::Box);
    assert!(s.construction_evidence(1).is_err());
    for input in ["Center","10,20,30","4","6"] {assert_eq!(s.input(input).unwrap(),Effect::Waiting);}
    assert_eq!(s.input("8").unwrap(),Effect::Commit);
    let owned=s.construction_evidence(u64::MAX).unwrap();
    assert!(owned.contains(r#""generation":"18446744073709551615""#));
    assert!(owned.contains(r#""output":[8,17,30,1,0,0,0,1,0,0,0,1,4,6,8]"#));
    assert!(owned.contains(r#""operation":"ConstructSolid""#));
    assert!(owned.contains(r#""feature_id":"OM9-SOLID-012""#));
    assert!(owned.contains(r#""mode":"Center""#));
    s.cancel();
    assert!(s.construction_evidence(2).is_err());
    assert!(owned.contains(r#""output":[8,17,30,"#));
}

#[test]
fn fit_input_undo_preserves_only_the_accepted_owned_world_points() {
    let mut s=Session::new(Kind::Sphere);
    s.input("FitPoints").unwrap();
    s.append_fit_points(&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]]).unwrap();
    s.point([0.,-2.,0.]).unwrap();
    s.input("Undo").unwrap();
    assert_eq!(s.input("").unwrap(),Effect::Commit);
    let evidence=s.construction_evidence(17).unwrap();
    assert!(evidence.contains(r#""points":[[1,0,0],[0,1,0],[-1,0,0]]"#));
    assert!(evidence.contains(r#""feature_id":"OM9-SOLID-014""#));
    assert!(evidence.contains(r#""coordinates":"document_world_mm""#));
    assert!(!evidence.contains("upstream_archive_conversion"));
    assert!(!evidence.contains("parameterization_preserved"));
}

#[test]
fn tangent_constraints_and_captured_frame_are_not_replaced_by_output_axes() {
    let mut s=Session::new(Kind::Sphere);
    s.input("Tangent").unwrap();
    s.set_frame([2.,3.,4.],[[0.,1.,0.],[-1.,0.,0.],[0.,0.,1.]]).unwrap();
    s.accept_reference([1.,0.,0.],[0.,0.,1.]).unwrap();
    s.input("Point").unwrap();s.point([0.,1.,0.]).unwrap();
    s.input("Radius=2").unwrap();
    assert_eq!(s.resolve([1.,1.,0.],2.).unwrap(),Effect::Commit);
    let evidence=s.construction_evidence(2).unwrap();
    assert!(evidence.contains(r#""constraint_types":[false,true]"#));
    assert!(evidence.contains(r#""frame_origin":[2,3,4]"#));
    assert!(evidence.contains(r#""frame_axes":[[0,1,0],[-1,0,0],[0,0,1]]"#));
    assert!(evidence.contains(r#""points":[[1,0,0],[0,1,0]]"#));
}

#[test]
fn four_point_and_curve_pick_evidence_keeps_points_reduced_by_the_solver() {
    let mut s=Session::new(Kind::Sphere);s.input("4Point").unwrap();
    for p in [[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]]{s.point(p).unwrap();}
    assert_eq!(s.point([0.,0.,1.]).unwrap(),Effect::Commit);
    let evidence=s.construction_evidence(3).unwrap();
    assert!(evidence.contains(r#""accepted_points":[[1,0,0],[0,1,0],[-1,0,0],[0,0,1]]"#));
    let mut s=Session::new(Kind::Sphere);s.input("AroundCurve").unwrap();
    s.accept_reference([7.,8.,9.],[0.,0.,1.]).unwrap();
    s.accept_reference([1.,2.,3.],[0.,0.,1.]).unwrap();s.input("Radius=2").unwrap();
    assert!(s.construction_evidence(4).unwrap().contains(r#""accepted_points":[[7,8,9],[1,2,3]]"#));
}

#[test]
fn construction_copy_rejects_stale_generations_without_writing_caller_memory() {
    unsafe {
        assert!(om9_solid_start(c"Box".as_ptr()));let generation=om9_solid_generation();
        let mut output=vec![0xA5;4096];
        let waiting=om9_solid_construction_json(generation,output.as_mut_ptr(),output.len());
        assert_ne!(waiting.status,0);assert!(output.iter().all(|&b|b==0xA5));
        for input in [c"Diagonal",c"0,0,0",c"2,3,0"] {assert_eq!(om9_solid_input(input.as_ptr()),1);}
        assert_eq!(om9_solid_input(c"4".as_ptr()),2);
        let query=om9_solid_construction_json(generation,std::ptr::null_mut(),0);
        assert_eq!(query.status,0);assert_eq!(query.reserved,0);assert!(query.needed>100);
        let small=om9_solid_construction_json(generation,output.as_mut_ptr(),1);
        assert_eq!(small.status,2);assert_eq!(small.needed,query.needed);assert!(output.iter().all(|&b|b==0xA5));
        let invalid=om9_solid_construction_json(generation,std::ptr::null_mut(),1);
        assert_eq!(invalid.status,1);
        output.resize(query.needed+16,0xA5);
        let copied=om9_solid_construction_json(generation,output.as_mut_ptr(),query.needed);
        assert_eq!(copied.status,0);assert_eq!(copied.needed,query.needed);
        assert!(output[query.needed..].iter().all(|&b|b==0xA5));
        let owned=output[..query.needed].to_vec();
        assert!(om9_solid_start(c"Sphere".as_ptr()));
        output.fill(0xA5);
        assert_eq!(om9_solid_construction_json(generation,output.as_mut_ptr(),output.len()).status,5);
        assert!(output.iter().all(|&b|b==0xA5));om9_solid_cancel();
        assert!(std::str::from_utf8(&owned).unwrap().contains(r#""output":[0,0,0,1,0,0,0,1,0,0,0,1,2,3,4]"#));
        assert_eq!(om9_solid_construction_json(om9_solid_generation(),output.as_mut_ptr(),output.len()).status,5);
    }
}
