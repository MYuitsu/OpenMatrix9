use openmatrix9_rust::geometry_frame::{Frame,IDENTITY};
use openmatrix9_rust::geometry_frame_ffi::{FrameInput,om9_geometry_frame};
fn translation(x:f64,y:f64,z:f64)->[f64;16] {let mut m=IDENTITY;m[3]=x;m[7]=y;m[11]=z;m}
#[test]
fn embedded_placement_is_removed_exactly_once_and_parent_rotation_kept() {
    let parent=[0.,-1.,0.,10.,1.,0.,0.,20.,0.,0.,1.,30.,0.,0.,0.,1.];
    let placement=translation(3.,4.,5.);let native=translation(9.,11.,13.);
    let f=Frame::compose(parent,placement,placement,native).unwrap();
    assert_eq!(f.transform([1.,2.,3.]).unwrap(),[-3.,30.,46.]);assert_eq!(f.orientation_sign(),1);
    assert_eq!(f.matrix(),[0.,-1.,0.,-1.,1.,0.,0.,29.,0.,0.,1.,43.,0.,0.,0.,1.]);
}
#[test]
fn instance_affine_path_keeps_mirror_nonuniform_scale_and_ignores_definition_ancestors() {
    let path=[-2.,0.,0.,7.,0.,3.,0.,8.,0.,0.,0.5,9.,0.,0.,0.,1.];
    let f=Frame::compose(translation(10.,20.,30.),path,translation(1.,2.,3.),translation(5.,7.,11.)).unwrap();
    assert_eq!(f.transform([1.,2.,3.]).unwrap(),[7.,49.,44.5]);assert_eq!(f.orientation_sign(),-1);
}
#[test]
fn invalid_native_frames_never_produce_a_matrix() {
    for which in 0..6 {
        let mut parent=IDENTITY;let mut path=IDENTITY;let mut source=IDENTITY;let mut loc=IDENTITY;
        match which {0=>parent[3]=f64::NAN,1=>path[0]=0.,2=>source[0]=2.,3=>loc[15]=2.,4=>{path[0]=1e150;loc[0]=1e150;},_=>path[12]=0.1}
        assert!(Frame::compose(parent,path,source,loc).is_err(),"{which}");
        assert_ne!(om9_geometry_frame(FrameInput{parent,path,source,location:loc}).status,0);
    }
    let result=om9_geometry_frame(FrameInput{parent:IDENTITY,path:IDENTITY,source:IDENTITY,location:translation(1.,2.,3.)});
    assert_eq!(result.status,0);assert_eq!(result.local_to_world,translation(1.,2.,3.));assert_eq!(result.orientation_sign,1);
}
