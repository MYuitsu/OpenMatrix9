use openmatrix9_rust::native_surface_capability::{SurfaceCapabilityInput,om9_native_surface_capability as gate,om9_native_surface_declared as declared};
use std::mem::{align_of,offset_of,size_of};
fn input(kind:u32)->SurfaceCapabilityInput{
    SurfaceCapabilityInput{kind,representation:2,stage:3,flags:128,style:0,section_mode:0,
        point_count:16,continuity_a:0,continuity_b:0,rail_closed:0,
        input_count:if kind==2{3}else{2},tolerance:0.01,slash_pairs:std::ptr::null(),slash_count:0}
}
#[test]
fn surface_value_layout_and_current_kind_defaults(){
    assert_eq!(size_of::<SurfaceCapabilityInput>(),72);assert_eq!(align_of::<SurfaceCapabilityInput>(),8);
    assert_eq!(offset_of!(SurfaceCapabilityInput,input_count),40);
    assert_eq!(offset_of!(SurfaceCapabilityInput,tolerance),48);
    assert_eq!(offset_of!(SurfaceCapabilityInput,slash_pairs),56);
    assert_eq!(offset_of!(SurfaceCapabilityInput,slash_count),64);
    for kind in 1..=3{assert_eq!(unsafe{gate(input(kind))},4);}
}
#[test]
fn surface_scalar_rejection_and_declarations_never_authorize_unknown(){
    for kind in [0,4,u32::MAX]{assert_eq!(unsafe{gate(input(kind))},100);assert!(!declared(kind));}
    for kind in 1..=3{
        assert!(declared(kind));
        for representation in [0,1,4,5,6,8,9,10,12,13]{
            let mut r=input(kind);r.representation=representation;
            assert_eq!(unsafe{gate(r)},101);
        }
        for representation in [2,3,7]{
            let mut r=input(kind);r.representation=representation;
            assert_eq!(unsafe{gate(r)},4);
        }
        let mut r=input(kind);r.stage=1;r.input_count=0;
        assert_eq!(unsafe{gate(r)},4);
        r.stage=2;assert_eq!(unsafe{gate(r)},4);
        r.stage=3;assert_eq!(unsafe{gate(r)},102);
    }
}
#[test]
fn malformed_masks_stages_and_numbers_are_checked(){
    for field in 0..8{
        let mut r=input(3);match field{
            0=>r.flags=256,1=>r.rail_closed=4,2=>r.stage=0,3=>r.stage=4,
            4=>r.point_count=1,5=>r.tolerance=f64::NAN,6=>r.continuity_a=3,_=>r.section_mode=3,
        }
        assert_eq!(unsafe{gate(r)},102);
    }
    let mut r=input(2);r.flags=128|4;
    assert_eq!(unsafe{gate(r)},4);r.input_count=4;assert_eq!(unsafe{gate(r)},102);
    let mut r=input(2);r.input_count=4;r.continuity_a=1;r.continuity_b=2;
    assert_eq!(unsafe{gate(r)},4);r.flags|=8;assert_eq!(unsafe{gate(r)},102);
}
#[test]
fn bounded_borrowed_slashes_are_read_only_and_rejected_before_invalid_reads(){
    let pairs=[0.2,0.3,0.8,0.9];let before=pairs;
    let mut r=input(2);r.slash_pairs=pairs.as_ptr();r.slash_count=2;
    assert_eq!(unsafe{gate(r)},4);assert_eq!(pairs,before);
    r.rail_closed=1;assert_eq!(unsafe{gate(r)},102);
    r.rail_closed=0;r.slash_count=65;r.slash_pairs=std::ptr::null();
    assert_eq!(unsafe{gate(r)},102);
    r.slash_count=usize::MAX;assert_eq!(unsafe{gate(r)},102);
    r.slash_count=1;assert_eq!(unsafe{gate(r)},102);
    r.slash_pairs=(pairs.as_ptr() as usize+1) as *const f64;
    assert_eq!(unsafe{gate(r)},102);
    let invalid=[0.2,f64::INFINITY];r.slash_pairs=invalid.as_ptr();
    assert_eq!(unsafe{gate(r)},102);
    let decreasing=[0.8,0.3,0.2,0.9];r.slash_pairs=decreasing.as_ptr();r.slash_count=2;
    assert_eq!(unsafe{gate(r)},102);assert_eq!(pairs,before);
}
