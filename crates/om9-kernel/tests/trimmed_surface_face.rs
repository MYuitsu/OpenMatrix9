// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "3dm-brep")]
use om9_core::{budget::Limits,rhino_semantics::TrimmedSurface,surface_kernel};
use om9_kernel::{brep,creation,measurement};

#[test]
fn many_edge_face_requests_native_headroom_before_conversion() {
    let limits=Limits::default();
    let surface=surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..129).map(|i|{let a=std::f64::consts::TAU*i as f64/129.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface,outer,inners:vec![],reversed:false};
    let mut requested=false;
    let result=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,pending|{
        if pending>1024*1024 {requested=true;Err(om9_core::Error::Budget("native headroom"))} else {Ok(())}
    });
    assert!(requested,"large native headroom was not requested");assert!(result.is_err());
}

#[test]
fn temporary_trimmed_face_preserves_planar_outer_and_hole() {
    let limits=Limits::default();
    let surface=surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    for reverse in [false,true] {
        let mut outer=vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
        let mut hole=vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]];
        if reverse {outer.reverse();hole.reverse();}
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![hole],reversed:false};
        let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
        let face=brep::read_brep(&source,0.01,&limits).unwrap();let measure=creation::measure(&face,&limits).unwrap();
        assert!((measure.area_mm2-32.).abs()<1e-6,"{}",measure.area_mm2);
        let target=om9_core::geometry::Geometry::Brep(face);let identity=om9_core::geometry::Transform::identity();
        let ray=|p|measurement::try_surface_ray_hit(&target,identity,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
        assert!(ray([3.,3.,10.]).is_some());
        assert!(ray([5.,5.,10.]).is_none());
        assert!(ray([9.,9.,10.]).is_none());
        let mut denied=false;
        let result=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_| {denied=true;Err(om9_core::Error::Budget("test trim admission"))});
        assert!(denied && result.is_err());
    }
}

#[test]
fn trimmed_planar_face_survives_native_3dm_brep_roundtrip() {
    let limits=Limits::default();
    let surface=surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let mut denied=false;
    assert!(brep::source_to_3dm_payload_admitted(&source,0.001,&limits,|_,_| {denied=true;Err(om9_core::Error::Budget("test native payload"))}).is_err());
    assert!(denied);
    let (payload,_)=brep::source_to_3dm_payload_admitted(&source,0.001,&limits,|_,_|Ok(())).unwrap();
    let restored=brep::from_3dm(&payload,0.001,0.01,&limits).unwrap();
    let geometry=om9_core::geometry::Geometry::Brep(restored);let transform=om9_core::geometry::Transform::identity();
    let ray=|p|measurement::try_surface_ray_hit(&geometry,transform,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
    assert!(ray([3.,3.,10.]).is_some());assert!(ray([5.,5.,10.]).is_none());assert!(ray([9.,9.,10.]).is_none());
}

#[test]
fn trimmed_cylinder_face_survives_native_3dm_brep_roundtrip() {
    let limits=Limits::default();
    let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.3,0.2],[0.3,0.8],[0.1,0.8]],
        inners:vec![vec![[0.15,0.4],[0.2,0.4],[0.2,0.6],[0.15,0.6]]],reversed:false};
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let face=brep::read_brep(&source,0.01,&limits).unwrap();let native=brep::to_3dm(&face,0.001,&limits).unwrap();
    let restored=brep::from_3dm(&native.payload,0.001,0.01,&limits).unwrap();
    let target=om9_core::geometry::Geometry::Brep(restored);let transform=om9_core::geometry::Transform::identity();
    let ray=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();let n=[p[0]/2.,p[1]/2.,0.];
        let origin=[p[0]-0.5*n[0],p[1]-0.5*n[1],p[2]];
        measurement::try_surface_ray_hit(&target,transform,origin,n,origin,1e-8,&limits).unwrap()};
    assert!(ray(0.12).is_some());assert!(ray(0.175).is_none());assert!(ray(0.35).is_none());
}

#[test]
fn temporary_trimmed_face_rejects_unverified_closed_seam() {
    let limits=Limits::default();let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.2],[1.,0.2],[0.5,0.8]],inners:vec![],reversed:false};
    assert!(measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).is_err());
}

#[test]
fn temporary_full_seam_cylinder_preserves_area_and_hole() {
    let limits=Limits::default();let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],
        inners:vec![vec![[0.3,0.4],[0.4,0.4],[0.4,0.6],[0.3,0.6]]],reversed:false};
    let angle=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();p[1].atan2(p[0])};
    let expected=12.*std::f64::consts::PI-2.*(angle(0.4)-angle(0.3));
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let face=brep::read_brep(&source,0.01,&limits).unwrap();let result=creation::measure(&face,&limits).unwrap();
    assert!((result.area_mm2-expected).abs()<1e-5,"expected {expected}, got {}",result.area_mm2);
    let target=om9_core::geometry::Geometry::Brep(face);let identity=om9_core::geometry::Transform::identity();
    let ray=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();let n=[p[0]/2.,p[1]/2.,0.];
        let origin=[p[0]-0.5*n[0],p[1]-0.5*n[1],p[2]];
        measurement::try_surface_ray_hit(&target,identity,origin,n,origin,1e-8,&limits).unwrap()};
    assert!(ray(0.2).is_some());assert!(ray(0.35).is_none());
}

#[test]
fn temporary_cylinder_patch_touching_one_seam_preserves_area_and_hole() {
    let limits=Limits::default();let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    for (a,b,h0,h1) in [(0.,0.4,0.15,0.2),(0.6,1.,0.8,0.85)] {
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[a,0.2],[b,0.2],[b,0.8],[a,0.8]],
            inners:vec![vec![[h0,0.4],[h1,0.4],[h1,0.6],[h0,0.6]]],reversed:false};
        let angle=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();p[1].atan2(p[0])};
        let delta=|x,y| (angle(y)-angle(x)).rem_euclid(std::f64::consts::TAU);
        let expected=6.*delta(a,b)-2.*delta(h0,h1);
        let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
        let face=brep::read_brep(&source,0.01,&limits).unwrap();let result=creation::measure(&face,&limits).unwrap();
        assert!((result.area_mm2-expected).abs()<1e-5,"expected {expected}, got {}",result.area_mm2);
        let target=om9_core::geometry::Geometry::Brep(face);
        let ray=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();let n=[p[0]/2.,p[1]/2.,0.];
            let origin=[p[0]-0.5*n[0],p[1]-0.5*n[1],p[2]];
            measurement::try_surface_ray_hit(&target,om9_core::geometry::Transform::identity(),origin,n,origin,1e-8,&limits).unwrap()};
        assert!(ray((h0+h1)/2.).is_none());assert!(ray((a+h0)/2.).is_some());
    }
}

#[test]
fn temporary_trimmed_cylinder_face_preserves_curved_area_and_hole() {
    let limits=Limits::default();let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.3,0.2],[0.3,0.8],[0.1,0.8]],
        inners:vec![vec![[0.15,0.4],[0.2,0.4],[0.2,0.6],[0.15,0.6]]],reversed:false};
    let angle=|u| {let p=surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();p[1].atan2(p[0])};
    let expected=2.*3.*(angle(0.3)-angle(0.1))-2.*1.*(angle(0.2)-angle(0.15));
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let face=brep::read_brep(&source,0.01,&limits).unwrap();let result=creation::measure(&face,&limits).unwrap();
    assert!((result.area_mm2-expected).abs()<1e-6,"expected {expected}, got {}",result.area_mm2);
}

#[test]
fn temporary_periodic_trimmed_face_preserves_uv_corner_positions() {
    use om9_core::geometry::NurbsSurface;
    let limits=Limits::default();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let weights=[1.,0.7,1.2,0.9,1.,0.7];let mut poles=Vec::new();let mut w=Vec::new();
    for (i,[x,y]) in rows.into_iter().enumerate() {for z in [0.,2.] {poles.push([x,y,z]);w.push(weights[i]);}}
    let surface=NurbsSurface {degree:[2,1],size:[6,2],poles,weights:w,
        knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],
        domain:[[0.3,3.7],[0.,1.]],periodic:[true,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.6,0.2],[2.5,0.2],[2.5,0.8],[0.6,0.8]],inners:vec![],reversed:false};
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let face=brep::read_brep(&source,0.01,&limits).unwrap();let vertices=brep::source_vertices(&face,&limits).unwrap();
    for uv in &trim.outer {let expected=surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();
        assert!(vertices.iter().any(|p|p.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-6)),"missing {expected:?}");}
    let target=om9_core::geometry::Geometry::Brep(face);
    for uv in [[0.9,0.35],[1.5,0.5],[2.2,0.65]] {
        let p=surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();let n=surface_kernel::normal(&surface,uv[0],uv[1],&limits).unwrap();
        let origin=std::array::from_fn(|i|p[i]+3.*n[i]);let direction=n.map(|v|-v);
        let hit=measurement::try_surface_ray_hit(&target,om9_core::geometry::Transform::identity(),origin,direction,p,1e-8,&limits).unwrap().unwrap();
        assert!(hit.iter().zip(p).all(|(a,b)|(a-b).abs()<1e-6),"{hit:?} vs {p:?}");
    }
}

#[test]
fn temporary_periodic_full_seam_matches_source_at_sampled_points() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=Limits::default();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let weights=[1.,0.7,1.2,0.9,1.,0.7];let mut poles=Vec::new();let mut w=Vec::new();
    for (i,[x,y]) in rows.into_iter().enumerate() {for z in [0.,2.] {poles.push([x,y,z]);w.push(weights[i]);}}
    let surface=NurbsSurface {degree:[2,1],size:[6,2],poles,weights:w,
        knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],
        domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[4.,0.2],[4.,0.8],[0.,0.8]],
        inners:vec![vec![[1.2,0.4],[1.6,0.4],[1.6,0.6],[1.2,0.6]]],reversed:false};
    let source=measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let face=brep::read_brep(&source,0.01,&limits).unwrap();let target=Geometry::Brep(face);
    let ray=|uv:[f64;2]| {let p=surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();
        let n=surface_kernel::normal(&surface,uv[0],uv[1],&limits).unwrap();
        let origin=std::array::from_fn(|i|p[i]-0.1*n[i]);
        (p,measurement::try_surface_ray_hit(&target,Transform::identity(),origin,n,origin,1e-8,&limits).unwrap())};
    for uv in [[0.2,0.35],[2.2,0.65],[3.8,0.5]] {let (p,hit)=ray(uv);let hit=hit.unwrap();
        assert!(hit.iter().zip(p).all(|(a,b)|(a-b).abs()<1e-6),"{hit:?} vs {p:?}");}
    assert!(ray([1.4,0.5]).1.is_none(),"hole center should not intersect a local outward ray");
}
