use openmatrix9_rust::{solid,native_solid_capability::{om9_native_solid_capability as gate,om9_native_solid_declared as declared}};
#[test]
fn declaration_uses_existing_solid_kinds_without_new_command_names(){
    assert!(declared(1));assert!(declared(2));for kind in [0,3,u32::MAX]{assert!(!declared(kind));}
}
#[test]
fn actual_session_query_and_preview_geometry_share_policy(){
    solid::om9_solid_cancel();assert_eq!(gate(1,2),100);
    assert_eq!(gate(1,0),102);assert_eq!(gate(0,2),101);
    unsafe{assert!(solid::om9_solid_start(c"Box".as_ptr()));}
    for stage in [1,2,4]{assert_eq!(gate(1,stage),4);}assert_eq!(gate(1,3),102);
    unsafe{assert_eq!(solid::om9_solid_input(c"Diagonal".as_ptr()),1);}
    for stage in [1,2,4]{assert_eq!(gate(1,stage),4);}assert_eq!(gate(1,3),102);assert_eq!(gate(2,2),101);
    assert_eq!(solid::om9_solid_point(0.,0.,0.,false),1);assert_eq!(solid::om9_solid_point(-2.,-3.,0.,false),1);
    let mut preview=[0.;15];unsafe{assert!(solid::om9_solid_geometry(preview.as_mut_ptr(),true,0.,0.,-4.,false));}
    assert_eq!(&preview[..3],&[-2.,-3.,-4.]);assert_eq!(&preview[12..],&[2.,3.,4.]);assert_eq!(gate(1,3),102);
    assert_eq!(solid::om9_solid_point(0.,0.,-4.,false),2);assert_eq!(gate(1,3),4);
    let mut committed=[0.;15];unsafe{assert!(solid::om9_solid_geometry(committed.as_mut_ptr(),false,0.,0.,0.,false));}
    assert_eq!(committed,preview);solid::om9_solid_cancel();assert_eq!(gate(1,4),100);
    unsafe{assert!(solid::om9_solid_start(c"Sphere".as_ptr()));assert_eq!(solid::om9_solid_input(c"FitPoints".as_ptr()),1);}
    for input in 1..=9{assert_eq!(gate(input,2),4);}for input in 10..=13{assert_eq!(gate(input,2),101);}
    solid::om9_solid_cancel();unsafe{assert!(solid::om9_solid_start(c"Sphere".as_ptr()));assert_eq!(solid::om9_solid_input(c"AroundCurve".as_ptr()),1);}
    assert_eq!(gate(2,2),4);assert_eq!(gate(8,2),101);assert_eq!(gate(9,2),101);solid::om9_solid_cancel();
}
