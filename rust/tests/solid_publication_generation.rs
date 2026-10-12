use openmatrix9_rust::solid::*;

#[test]
fn publication_token_rejects_same_kind_restart_and_cancel() {
    om9_solid_cancel();
    let inactive = om9_solid_generation();
    unsafe { assert!(om9_solid_start(c"Box".as_ptr())); }
    let original = om9_solid_generation();
    assert!(original > inactive);
    unsafe { assert!(om9_solid_start(c"Box".as_ptr())); }
    let restarted = om9_solid_generation();
    assert!(restarted > original, "same-kind replacement must invalidate old publication");
    unsafe { assert!(!om9_solid_start(c"invalid command".as_ptr())); }
    assert_eq!(om9_solid_generation(), restarted, "rejected start leaves the live request alone");
    unsafe { assert_eq!(om9_solid_input(c"Cancel".as_ptr()), 3); }
    let cancelled = om9_solid_generation();
    assert!(cancelled > restarted, "direct Rust cancellation invalidates publication");
    unsafe { assert!(om9_solid_start(c"Sphere".as_ptr())); }
    assert!(om9_solid_generation() > cancelled);
    let sphere = om9_solid_generation();
    om9_solid_cancel();
    assert!(om9_solid_generation() > sphere);
    assert_eq!(om9_solid_phase(), 0);
}
