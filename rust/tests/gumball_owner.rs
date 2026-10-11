// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{geometry_frame::IDENTITY,gumball_owner::owner_scale};
#[test]
fn owner_placement_must_be_a_rigid_right_handed_frame() {
    for kind in 1..=4 {
        let mut world = IDENTITY;
        world[0] = 2.;
        assert!(owner_scale(IDENTITY,world,[1.;3],kind).is_err());
        world = IDENTITY;
        world[0] = -1.;
        assert!(owner_scale(IDENTITY,world,[1.;3],kind).is_err());
    }
    assert!(owner_scale(IDENTITY,IDENTITY,[1e12;3],3).is_ok());
    let mut affine=IDENTITY;affine[0]=2.;
    assert!(owner_scale(affine,IDENTITY,[1e12;3],3).is_err());
}
#[test]
fn radial_primitives_scale_radius_and_axial_height_independently() {
    let mut affine=IDENTITY;
    affine[0]=2.;affine[5]=2.;affine[10]=3.;affine[3]=-5.;affine[11]=7.;
    let scale=owner_scale(affine,IDENTITY,[1.;3],4).unwrap();
    assert_eq!(scale.factors,[2.,2.,3.]);
    assert_eq!(scale.placement[3],-5.);
    assert_eq!(scale.placement[11],7.);
    assert_eq!([scale.placement[0],scale.placement[5],scale.placement[10]],[1.;3]);
    affine[0]=1.;affine[5]=1.;
    assert_eq!(owner_scale(affine,IDENTITY,[1.;3],4).unwrap().factors,[1.,1.,3.]);
}
#[test]
fn radial_primitives_reject_elliptical_bases_shear_and_reflection() {
    let mut affine=IDENTITY;affine[0]=2.;
    assert!(owner_scale(affine,IDENTITY,[1.;3],4).is_err());
    affine[5]=2.;affine[1]=0.5;
    assert!(owner_scale(affine,IDENTITY,[1.;3],4).is_err());
    affine=IDENTITY;affine[10]=-2.;
    assert!(owner_scale(affine,IDENTITY,[1.;3],4).is_err());
    affine=IDENTITY;affine[10]=0.;
    assert!(owner_scale(affine,IDENTITY,[1.;3],4).is_err());
}
#[test]
fn parametric_and_occurrence_scales_require_exact_native_representation(){
    let mut affine=IDENTITY;affine[0]=2.;affine[3]=-6.;
    let box_scale=owner_scale(affine,IDENTITY,[1.;3],1).unwrap();
    assert_eq!(box_scale.factors,[2.,1.,1.]);assert_eq!(box_scale.placement[3],-6.);
    assert!(owner_scale(affine,IDENTITY,[1.;3],2).is_err());
    let link=owner_scale(affine,IDENTITY,[3.,1.,1.],3).unwrap();assert_eq!(link.factors,[6.,1.,1.]);
    let mut rotated=IDENTITY;let c=2f64.sqrt()/2.;rotated[0]=c;rotated[1]=-c;rotated[4]=c;rotated[5]=c;
    assert!(owner_scale(affine,rotated,[1.;3],3).is_err());
    assert!(owner_scale(affine,rotated,[1.;3],1).is_err());
    affine[5]=2.;affine[10]=2.;let sphere=owner_scale(affine,IDENTITY,[1.;3],2).unwrap();assert_eq!(sphere.factors,[2.;3]);
}
