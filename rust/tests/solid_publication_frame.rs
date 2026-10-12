// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{geometry_frame::IDENTITY, solid::{SolidPublicationFrame, om9_solid_publication_frame}};

fn compare(expected: [f64; 16], actual: [f64; 16]) -> u32 {
    om9_solid_publication_frame(SolidPublicationFrame { expected, actual })
}

#[test]
fn published_frame_matches_frozen_local_and_world_observations() {
    assert_eq!(compare(IDENTITY, IDENTITY), 0);
    let mut frame=IDENTITY; frame[3]=20.;frame[7]=30.;frame[11]=40.;
    assert_eq!(compare(frame, frame), 0);
    assert_eq!(compare(IDENTITY, frame), 3, "a placed parent changes world geometry");
    let mut rotation=IDENTITY;rotation[0]=0.;rotation[1]=-1.;rotation[4]=1.;rotation[5]=0.;
    assert_eq!(compare(rotation,rotation),0);
    assert_eq!(compare(IDENTITY,rotation),3);
    let mut roundtrip=rotation;roundtrip[0]=f64::EPSILON;
    assert_eq!(compare(rotation,roundtrip),0,"native quaternion roundtrip noise");
    roundtrip[3]=1e-9;
    assert_eq!(compare(rotation,roundtrip),3,"do not use CAD tolerance to accept moved output");
}

#[test]
fn invalid_native_placement_is_rejected_before_shape_normalization() {
    for index in 0..16 {
        for invalid in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            let mut actual=IDENTITY;actual[index]=invalid;
            assert_eq!(compare(IDENTITY,actual),1);
            assert_eq!(compare(actual,IDENTITY),1);
        }
    }
    let mut singular=IDENTITY;singular[0]=0.;assert_eq!(compare(IDENTITY,singular),2);
    let mut projective=IDENTITY;projective[12]=0.1;assert_eq!(compare(IDENTITY,projective),2);
    let mut oversized=IDENTITY;oversized[3]=1e151;assert_eq!(compare(IDENTITY,oversized),2);
}
