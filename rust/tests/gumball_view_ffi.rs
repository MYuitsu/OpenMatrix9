// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{gumball_ffi::*,gumball_view::{ViewSettings,ViewProjection,ViewLayout},gumball_owner::OwnerScale,geometry_frame::IDENTITY};
#[test]
fn settings_layout_and_owner_outputs_are_atomic_and_bounded() {
    unsafe {
        assert_eq!(std::mem::size_of::<ViewSettings>(),200);
        assert_eq!(std::mem::size_of::<ViewProjection>(),80);
        assert_eq!(std::mem::size_of::<ViewLayout>(),152);
        assert_eq!(std::mem::size_of::<OwnerScale>(),152);
        let id=om9_gumball_create();assert_ne!(id,0);
        let mut settings=ViewSettings::default();settings.strength=50.;
        assert_eq!(om9_gumball_settings(id,&settings,std::ptr::null_mut()),0);
        let saved=settings;settings.strength=f64::NAN;
        let mut output=ViewSettings::default();output.strength=77.;
        assert_eq!(om9_gumball_settings(id,&settings,&mut output),2);
        assert_eq!(output.strength,77.);
        assert_eq!(om9_gumball_settings(id,std::ptr::null(),&mut output),0);assert_eq!(output,saved);
        assert_eq!(om9_gumball_extents(id,std::ptr::null(),usize::MAX),1);
        let p=ViewProjection{right:[1.,0.,0.],up:[0.,1.,0.],forward:[0.,0.,-1.],world_per_pixel:0.1};
        let points=[[-6.,-4.,-3.],[6.,4.,3.]];
        let mut layout=ViewLayout::default();layout.visible=[99;14];
        assert_eq!(om9_gumball_layout(id,&p,points.as_ptr(),usize::MAX,&mut layout),1);
        assert_eq!(layout.visible,[99;14]);
        let mut invalid=p;invalid.right=[0.;3];
        assert_eq!(om9_gumball_layout(id,&invalid,points.as_ptr(),2,&mut layout),2);
        assert_eq!(layout.visible,[99;14]);
        assert_eq!(om9_gumball_layout(id,&p,points.as_ptr(),2,&mut layout),0);assert_eq!(layout.reference,[-6.,-4.,-3.]);
        let mut owner=OwnerScale{placement:[77.;16],factors:[88.;3]};
        assert_eq!(om9_gumball_owner_scale(std::ptr::null(),IDENTITY.as_ptr(),[1.;3].as_ptr(),3,&mut owner),1);
        let mut world=IDENTITY;world[0]=2.;
        assert_ne!(om9_gumball_owner_scale(IDENTITY.as_ptr(),world.as_ptr(),[1.;3].as_ptr(),3,&mut owner),0);
        assert_eq!(owner.placement,[77.;16]);assert_eq!(owner.factors,[88.;3]);
        assert_eq!(om9_gumball_restore_defaults(id),0);
        assert_eq!(om9_gumball_settings(id,std::ptr::null(),&mut output),0);assert_eq!(output,ViewSettings::default());
        assert_eq!(om9_gumball_destroy(id),0);
        let triangle=[[0.,0.,0.],[0.,2.,0.],[0.,0.,3.]];let mut directions=[77.;6];
        assert_eq!(om9_gumball_mesh_directions(triangle.as_ptr(),usize::MAX,directions.as_mut_ptr()),1);
        assert_eq!(directions,[77.;6]);
        assert_eq!(om9_gumball_mesh_directions([[0.;3];3].as_ptr(),3,directions.as_mut_ptr()),2);
        assert_eq!(directions,[77.;6]);
        assert_eq!(om9_gumball_mesh_directions(triangle.as_ptr(),3,directions.as_mut_ptr()),0);
        assert_eq!(directions,[0.,1.,0.,1.,0.,0.]);
    }
}
