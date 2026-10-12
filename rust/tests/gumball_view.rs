// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::gumball::Frame;
use openmatrix9_rust::gumball_view::{layout, ViewProjection, ViewSettings};

fn projection(right: [f64;3], up: [f64;3], forward: [f64;3]) -> ViewProjection {
    ViewProjection { right, up, forward, world_per_pixel: 0.1 }
}
fn handles(p: ViewProjection) -> Vec<usize> {
    let l = layout(Frame::default(), &[[-6.,-4.,-3.],[6.,4.,3.]],p,ViewSettings::default()).unwrap();
    l.visible.iter().enumerate().filter_map(|(i,&v)| (v==1).then_some(i)).collect()
}
#[test]
fn orthographic_visibility_comes_from_projection() {
    assert_eq!(handles(projection([1.,0.,0.],[0.,1.,0.],[0.,0.,-1.])),vec![0,1,3,6,9,10,11,13]);
    assert_eq!(handles(projection([1.,0.,0.],[0.,0.,1.],[0.,1.,0.])),vec![0,2,5,6,8,10,12,13]);
    assert_eq!(handles(projection([0.,1.,0.],[0.,0.,1.],[-1.,0.,0.])),vec![1,2,4,6,7,11,12,13]);
}
#[test]
fn scales_follow_geometry_and_relocated_basis() {
    let mut f=Frame::default();f.origin=[6.,4.,3.];
    let p=projection([1.,0.,0.],[0.,1.,0.],[0.,0.,-1.]);
    let l=layout(f,&[[0.,0.,0.],[12.,8.,6.]],p,ViewSettings::default()).unwrap();
    assert_eq!(l.anchors,[[0.,4.,3.],[6.,0.,3.],[6.,4.,0.]]);
    assert_eq!(l.reference,[-6.,-4.,-3.]);
    f.origin=[-10.,4.,3.];
    let outside=layout(f,&[[0.,0.,0.],[12.,8.,6.]],p,ViewSettings::default()).unwrap();
    assert!(outside.anchors[0][0]<=-11.2);
    assert_eq!(outside.reference[0],-22.);
    f.axes=[[0.,1.,0.],[-1.,0.,0.],[0.,0.,1.]];f.origin=[6.,4.,3.];
    let rotated=layout(f,&[[0.,0.,0.],[12.,8.,6.]],p,ViewSettings::default()).unwrap();
    assert_eq!(rotated.anchors[0],[6.,0.,3.]);
    assert_eq!(rotated.anchors[1],[12.,4.,3.]);
    let tiny=layout(Frame::default(),&[[-0.1;3],[0.1;3]],p,ViewSettings::default()).unwrap();
    assert_eq!(tiny.anchors[0],[-0.1,0.,0.]);
}
#[test]
fn pixel_defaults_and_zero_groups_are_explicit() {
    let mut s=ViewSettings::default();
    assert_eq!((s.move_length,s.rotate_radius,s.arrow_length,s.arrow_width),(80.,80.,18.,9.));
    assert_eq!((s.scale_size,s.origin_size,s.plane_size,s.plane_offset),(8.,4.,20.,50.));
    assert_eq!((s.menu_size,s.menu_x,s.menu_y,s.pick_radius,s.strength),(10.,-30.,30.,5.,100.));
    assert_eq!((s.snappy,s.orbit),(0,0));
    s.move_thickness=0.;s.rotate_thickness=0.;s.plane_size=0.;s.scale_size=0.;s.menu_size=0.;
    let l=layout(Frame::default(),&[[-1.;3],[1.;3]],projection([1.,0.,0.],[0.,1.,0.],[0.,0.,-1.]),s).unwrap();
    assert_eq!(l.visible.iter().enumerate().filter_map(|(i,&v)|(v==1).then_some(i)).collect::<Vec<_>>(),vec![6]);
}
#[test]
fn invalid_observations_and_settings_are_rejected() {
    for value in [0.,-1.,f64::NAN,f64::INFINITY] {
        let mut s=ViewSettings::default();s.strength=value;assert!(s.validate().is_err());
    }
    let p=projection([1.,0.,0.],[0.,1.,0.],[0.,0.,-1.]);
    assert!(layout(Frame::default(),&[[f64::NAN,0.,0.]],p,ViewSettings::default()).is_err());
    assert!(layout(Frame::default(),&[],p,ViewSettings::default()).is_err());
    let l=layout(Frame::default(),&[[0.;3]],p,ViewSettings::default()).unwrap();
    assert!((l.anchors[0][0]+1.2).abs()<1e-12);
    assert_eq!(l.reference,[0.;3]); // no geometry extent: Begin Scale must reject
}

#[test]
fn mesh_component_directions_require_one_oriented_plane() {
    use openmatrix9_rust::gumball_view::mesh_directions;
    let triangles=[[0.,0.,0.],[0.,2.,0.],[0.,2.,3.],[0.,0.,0.],[0.,2.,3.],[0.,0.,3.]];
    let (x,n)=mesh_directions(&triangles).unwrap();
    assert_eq!(x,[0.,1.,0.]);assert_eq!(n,[1.,0.,0.]);
    let mut bent=triangles;bent[5][0]=1.;assert!(mesh_directions(&bent).is_err());
    let reversed=[triangles[0],triangles[1],triangles[2],triangles[3],triangles[5],triangles[4]];
    assert!(mesh_directions(&reversed).is_err());
    assert!(mesh_directions(&[[0.;3],[1.,0.,0.],[2.,0.,0.]]).is_err());
    assert!(mesh_directions(&triangles[..5]).is_err());
    assert!(mesh_directions(&[[f64::NAN;3];3]).is_err());
}
