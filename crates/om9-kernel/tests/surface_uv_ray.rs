// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "3dm-brep")]
use om9_core::{budget::Limits,geometry::{Geometry,Transform},surface_kernel};
use om9_kernel::measurement::surface_ray_hits_uv;

#[test]
fn surface_uv_ray_reports_all_hits_in_source_parameters() {
    let l=Limits::default();let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&l).unwrap();
    let affine=Transform([[2.,0.5,0.,4.],[0.25,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);
    for placement in [Transform::identity(),Transform::translation([4.,-3.,7.]),affine] {
        let origin=placement.point([5.,0.4,2.5]).unwrap();
        let direction=[-placement.0[0][0],-placement.0[1][0],-placement.0[2][0]];
        let hits=surface_ray_hits_uv(&Geometry::Surface(surface.clone()),placement,origin,direction,1e-8,&l).unwrap();
        assert_eq!(hits.len(),2,"{hits:?}");
        for hit in &hits {
            let local=surface_kernel::evaluate(&surface,hit.uv[0],hit.uv[1],&l).unwrap();
            let expected=placement.point(local).unwrap();
            assert!(expected.iter().zip(hit.point).all(|(a,b)|(a-b).abs()<1e-7),"{expected:?} {hit:?}");
            assert!(hit.distance>=0.);
        }
        assert!((hits[0].distance-hits[1].distance).abs()>1.);
        let miss=surface_ray_hits_uv(&Geometry::Surface(surface.clone()),placement,placement.point([5.,4.,2.5]).unwrap(),direction,1e-8,&l).unwrap();
        assert!(miss.is_empty());
    }
    let limited=Limits {max_outputs:1,..l};
    assert!(surface_ray_hits_uv(&Geometry::Surface(surface),Transform::identity(),[5.,0.4,2.5],[-1.,0.,0.],1e-8,&limited).is_err());
}
