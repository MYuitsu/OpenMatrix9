use openmatrix9_rust::solid::*;
#[test]
fn owned_wire_buffers_are_bounded_and_hover_does_not_advance_native_session() {
    unsafe {
        om9_solid_cancel();assert!(om9_solid_start(c"Box".as_ptr()));
        assert_eq!(om9_solid_point(0.,0.,0.,false),1);
        let mut edges=[1234567.;73];let mut marks=[7654321.;10];
        assert_eq!(om9_solid_wire_preview(edges.as_mut_ptr(),marks.as_mut_ptr(),10.,20.,0.,false),4);
        assert_eq!(edges[72],1234567.);assert_eq!(marks[9],7654321.);assert_eq!(om9_solid_phase(),2);
        assert_eq!(&marks[3..6],&[10.,20.,0.]);
        assert_eq!(om9_solid_wire_preview(std::ptr::null_mut(),marks.as_mut_ptr(),10.,20.,0.,false),0);
        assert_eq!(om9_solid_point(10.,20.,0.,false),1);let mut axis=[0.;6];assert!(om9_solid_height_axis(axis.as_mut_ptr()));assert_eq!(&axis[..3],&[10.,20.,0.]);
        let mut result=[0.;3];assert!(om9_solid_height_drag(result.as_mut_ptr(),[10.,25.,0.].as_ptr(),[0.,1.,0.].as_ptr()));assert_eq!(result,[10.,20.,5.]);
        assert_eq!(om9_solid_wire_preview(edges.as_mut_ptr(),marks.as_mut_ptr(),result[0],result[1],result[2],false),12);
        assert_eq!(edges[72],1234567.);assert_eq!(marks[9],7654321.);assert_eq!(om9_solid_phase(),4);
        om9_solid_cancel();assert_eq!(om9_solid_wire_preview(edges.as_mut_ptr(),marks.as_mut_ptr(),1.,2.,3.,false),0);
    }
}
