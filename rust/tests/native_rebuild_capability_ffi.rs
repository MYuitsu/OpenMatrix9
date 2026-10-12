use openmatrix9_rust::native_rebuild_capability::{om9_native_rebuild_capability as gate,om9_native_rebuild_declared};
#[test]
fn scalar_bridge_matches_current_types_options_and_stages(){
    assert!(om9_native_rebuild_declared());
    assert_eq!(gate(2,1,0,4,3,1),4);
    for kind in [2,3,7]{for flags in [0,1,4,5,8,9,12,13]{
        assert_eq!(gate(kind,2,1,4,3,flags),4);
        assert_eq!(gate(kind,3,16,256,11,flags),4);
    }}
}
#[test]
fn malformed_envelopes_reject_without_host_storage(){
    for kind in [0,1,4,5,6,8,9,10,11,12,13,u32::MAX]{assert_eq!(gate(kind,3,1,4,3,1),101);}
    for stage in [0,4,u32::MAX]{assert_eq!(gate(2,stage,1,4,3,1),102);}
    for count in [0,17,usize::MAX]{assert_eq!(gate(2,3,count,4,3,1),102);}
    for flags in [2,3,15,16,u32::MAX]{assert_eq!(gate(2,3,1,4,3,flags),102);}
    for (poles,degree) in [(1,1),(4,4),(257,3),(8,usize::MAX)]{assert_eq!(gate(2,3,1,poles,degree,1),102);}
}
