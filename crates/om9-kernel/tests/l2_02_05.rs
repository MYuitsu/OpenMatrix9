// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused OCCT acceptance for `OM9-L2-02-05`: patch, join, offset, fillet/chamfer and
//! normal extrusion through the shared kernel. The portable surface services are covered by
//! `crates/om9-core/tests/surface_ops.rs`; this file proves the exact B-Rep side.
#![cfg(feature = "3dm-brep")]

use om9_core::budget::Limits;
use om9_kernel::{
    creation::{self, DEFAULT_DEFLECTION_MM, Primitive},
    editing,
};

fn limits() -> Limits {
    Limits::default()
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn rectangle_patch(x0: f64, x1: f64) -> creation::SurfacePatch {
    let boundary = [
        vec![[x0, 0.0, 0.0], [(x0 + x1) / 2.0, 0.0, 0.0], [x1, 0.0, 0.0]],
        vec![[x1, 0.0, 0.0], [x1, 1.0, 0.0], [x1, 2.0, 0.0]],
        vec![[x1, 2.0, 0.0], [(x0 + x1) / 2.0, 2.0, 0.0], [x0, 2.0, 0.0]],
        vec![[x0, 2.0, 0.0], [x0, 1.0, 0.0], [x0, 0.0, 0.0]],
    ];
    creation::surface_from_edges(&boundary, 1e-6, DEFAULT_DEFLECTION_MM, &limits())
        .expect("planar boundary patch")
}

#[test]
fn m03_02_05_patch_and_normal_extrude_are_exact_breps() {
    let patch = rectangle_patch(0.0, 4.0);
    assert!(patch.max_deviation_mm <= 1e-6);
    let measured = creation::measure(&patch.brep, &limits()).expect("patch measure");
    assert!(
        close(measured.area_mm2, 8.0, 1e-5),
        "patch area {}",
        measured.area_mm2
    );
    assert!(!measured.closed, "a single patch is an open sheet");

    // Normal extrusion / thickness through the shared OCCT kernel.
    let solid = editing::thicken(&patch.brep, 0.5, DEFAULT_DEFLECTION_MM, &limits())
        .expect("thicken patch");
    let measured = creation::measure(&solid, &limits()).expect("solid measure");
    assert!(measured.closed, "thickened patch must be closed");
    assert!(
        close(measured.volume_mm3, 8.0 * 0.5, 0.05),
        "thickened volume {}",
        measured.volume_mm3
    );
    assert_eq!(measured.solids, 1);
}

#[test]
fn m03_02_05_offset_fillet_and_chamfer_use_the_shared_kernel() {
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("box");
    let boxed_volume = creation::measure(&boxed, &limits()).unwrap().volume_mm3;
    let offset =
        editing::offset_shape(&boxed, 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("offset box");
    let offset_volume = creation::measure(&offset, &limits()).unwrap().volume_mm3;
    assert!(
        offset_volume > boxed_volume,
        "outward offset {offset_volume} <= {boxed_volume}"
    );
    assert!(creation::measure(&offset, &limits()).unwrap().closed);

    let filleted = editing::fillet(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits())
        .expect("fillet box edge");
    let fillet_measures = creation::measure(&filleted, &limits()).unwrap();
    assert!(fillet_measures.closed);
    assert_eq!(fillet_measures.naked_edges, 0);
    assert!(fillet_measures.faces > 6, "fillet adds a face");

    let chamfered = editing::chamfer(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits())
        .expect("chamfer box edge");
    let chamfer_measures = creation::measure(&chamfered, &limits()).unwrap();
    assert!(chamfer_measures.closed);
    assert_eq!(chamfer_measures.naked_edges, 0);
    assert!(chamfer_measures.faces > 6, "chamfer adds a face");
    assert!(
        chamfer_measures.volume_mm3 < boxed_volume,
        "chamfer removes material"
    );
}

#[test]
fn m03_02_05_join_surfaces_sews_two_sheets() {
    let left = rectangle_patch(0.0, 2.0);
    let right = rectangle_patch(2.0, 4.0);
    let joined = editing::sew_all(
        &[left.brep, right.brep],
        1e-6,
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("sew two surface sheets");
    let measured = creation::measure(&joined, &limits()).expect("joined measure");
    assert!(measured.faces >= 2, "joined faces {}", measured.faces);
    assert!(measured.shells >= 1, "joined shells {}", measured.shells);
    assert!(!measured.closed, "two open sheets form an open shell");
    // Both authored boundaries remain in the exact B-Rep.
    assert!(measured.area_mm2 > 8.0 - 1e-6);
}

#[test]
fn m03_02_05_surface_services_refuse_degenerate_bodies() {
    let patch = rectangle_patch(0.0, 4.0);
    assert!(editing::thicken(&patch.brep, 0.0, DEFAULT_DEFLECTION_MM, &limits()).is_err());
    assert!(editing::offset_shape(&patch.brep, 0.0, DEFAULT_DEFLECTION_MM, &limits()).is_err());
    // A gap in the boundary loop must be refused, never bridged silently.
    let open = [
        vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
        vec![[4.0, 0.0, 0.0], [4.0, 2.0, 0.0]],
        vec![[4.0, 2.0, 0.0], [0.0, 2.0, 0.0]],
        vec![[0.0, 2.0, 0.0], [0.0, 0.5, 0.0]],
    ];
    assert!(creation::surface_from_edges(&open, 1e-6, DEFAULT_DEFLECTION_MM, &limits()).is_err());
}

#[test]
fn m03_02_05_kernel_receipt() {
    let patch = rectangle_patch(0.0, 4.0);
    let patch_measure = creation::measure(&patch.brep, &limits()).expect("patch measure");
    let solid = editing::thicken(&patch.brep, 0.5, DEFAULT_DEFLECTION_MM, &limits())
        .expect("thicken patch");
    let solid_measure = creation::measure(&solid, &limits()).expect("solid measure");
    let boxed = creation::primitive(
        Primitive::Box {
            x_mm: 10.0,
            y_mm: 20.0,
            z_mm: 30.0,
        },
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("box");
    let offset =
        editing::offset_shape(&boxed, 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("offset box");
    let offset_measure = creation::measure(&offset, &limits()).expect("offset measure");
    let filleted =
        editing::fillet(&boxed, &[0], 1.0, DEFAULT_DEFLECTION_MM, &limits()).expect("fillet");
    let fillet_measure = creation::measure(&filleted, &limits()).expect("fillet measure");
    let joined = editing::sew_all(
        &[
            rectangle_patch(0.0, 2.0).brep,
            rectangle_patch(2.0, 4.0).brep,
        ],
        1e-6,
        DEFAULT_DEFLECTION_MM,
        &limits(),
    )
    .expect("sew sheets");
    let join_measure = creation::measure(&joined, &limits()).expect("join measure");
    let receipt = serde_json::json!({
        "status": "measured",
        "patch_area_mm2": patch_measure.area_mm2,
        "patch_deviation_mm": patch.max_deviation_mm,
        "thicken_volume_mm3": solid_measure.volume_mm3,
        "thicken_closed": solid_measure.closed,
        "offset_volume_mm3": offset_measure.volume_mm3,
        "offset_closed": offset_measure.closed,
        "fillet_faces": fillet_measure.faces,
        "fillet_naked_edges": fillet_measure.naked_edges,
        "join_faces": join_measure.faces,
        "join_shells": join_measure.shells,
    });
    println!("{receipt}");
}

#[test]
fn m03_02_05_kernel_stress_joins_and_offsets_repeatedly() {
    let mut parts = Vec::new();
    for index in 0..10 {
        parts.push(rectangle_patch(index as f64 * 2.0, index as f64 * 2.0 + 2.0).brep);
    }
    let joined =
        editing::sew_all(&parts, 1e-6, DEFAULT_DEFLECTION_MM, &limits()).expect("stress sew");
    let measured = creation::measure(&joined, &limits()).expect("stress join measure");
    assert!(measured.faces >= 10);
    assert!(measured.shells >= 1);
    let mut shape = joined;
    for _ in 0..3 {
        shape = editing::offset_shape(&shape, 0.1, DEFAULT_DEFLECTION_MM, &limits())
            .expect("stress offset");
        let measured = creation::measure(&shape, &limits()).expect("stress offset measure");
        assert!(measured.faces >= 10);
        assert!(measured.volume_mm3.is_finite());
    }
}

#[test]
fn bounded_nurbs_surface_to_face_preserves_area_and_refuses_invalid_input() {
    use om9_core::geometry::Transform;
    let limits=limits();let mut surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::translation([8.,2.,3.]),DEFAULT_DEFLECTION_MM,&limits).unwrap();let m=creation::measure(&face,&limits).unwrap();assert_eq!(m.faces,1);assert_eq!(m.naked_edges,4);assert!((m.area_mm2-6.).abs()<1e-8);
    assert!(om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),0.,&limits).is_err());surface.knots[0].clear();assert!(om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).is_err());
}

#[test]
fn rational_curved_surface_face_matches_rust_samples() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=limits();let surface=NurbsSurface{degree:[2,1],size:[3,2],poles:vec![[0.,0.,0.],[0.,1.,0.],[1.,0.,1.],[1.,1.,1.],[2.,0.,0.],[2.,1.,0.]],weights:vec![1.,1.,0.7,0.7,1.2,1.2],knots:[vec![0.,0.,0.,1.,1.,1.],vec![0.,0.,1.,1.]],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    for (u,v) in [(0.2,0.3),(0.5,0.5),(0.8,0.7)] {let p=om9_core::surface_kernel::evaluate(&surface,u,v,&limits).unwrap();let origin=[p[0],p[1],p[2]+3.];let hit=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face.clone()),Transform::identity(),origin,[0.,0.,-1.],p,1e-8,&limits).unwrap();assert!(p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7),"{p:?} {hit:?}");}
}

#[test]
fn reversed_surface_face_retains_parameter_geometry_and_normal_direction() {
    use om9_core::{geometry::Transform,surface_kernel::SurfaceIso};
    let limits=limits();let original=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    for axis in [SurfaceIso::U,SurfaceIso::V] {
        let surface=om9_core::surface_ops::reverse_surface(&original,axis,&limits).unwrap();assert!(surface.reversed);
        let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
        let expected=om9_core::surface_kernel::normal(&surface,0.5,0.5,&limits).unwrap();let t=face.display.triangles[0];let p=face.display.points[t[0] as usize];let q=face.display.points[t[1] as usize];let r=face.display.points[t[2] as usize];let u=[q[0]-p[0],q[1]-p[1],q[2]-p[2]];let v=[r[0]-p[0],r[1]-p[1],r[2]-p[2]];let normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];assert!(normal.iter().zip(expected).map(|(a,b)|a*b).sum::<f64>()>0.);
    }
}

#[test]
fn sewing_preserves_each_underlying_surface_on_a_toleranced_gap() {
    use om9_core::geometry::Transform;
    let limits=limits();let s=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    let a=om9_kernel::measurement::surface_to_face(&s,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();let b=om9_kernel::measurement::surface_to_face(&s,Transform::translation([2.0005,0.,0.]),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let joined=editing::sew_preserving_surfaces(&a,&b,0.001,DEFAULT_DEFLECTION_MM,&limits).unwrap();let measured=creation::measure(&joined,&limits).unwrap();assert_eq!(measured.faces,2);assert_eq!(measured.shells,1);
}

#[test]
fn repeated_periodic_surface_face_matches_rust_evaluation() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=limits();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];let mut poles=Vec::new();let mut weights=Vec::new();for (i,[x,y]) in rows.into_iter().enumerate(){for z in [0.,2.] {poles.push([x,y,z]);weights.push([1.,0.7,1.2,0.9,1.,0.7][i]);}}
    let surface=NurbsSurface{degree:[2,1],size:[6,2],poles,weights,knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();let m=creation::measure(&face,&limits).unwrap();assert_eq!(m.faces,1);assert_eq!(om9_kernel::measurement::face_periodicity(&face,&limits).unwrap(),[true,false]);
    for u in [0.3,1.5,2.7] {let p=om9_core::surface_kernel::evaluate(&surface,u,0.4,&limits).unwrap();let dx=p[0]-1.;let dy=p[1]-1.;let origin=[p[0]+3.*dx,p[1]+3.*dy,p[2]];let hit=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face.clone()),Transform::identity(),origin,[-dx,-dy,0.],p,1e-8,&limits).unwrap();assert!(p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7),"{p:?} {hit:?}");}
}

#[test]
fn periodic_surface_v_axis_and_invalid_basis_are_checked() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=limits();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];let mut poles=Vec::new();let mut weights=Vec::new();for z in [0.,2.] {for (i,[x,y]) in rows.iter().copied().enumerate(){poles.push([x,y,z]);weights.push([1.,0.7,1.2,0.9,1.,0.7][i]);}}
    let mut s=NurbsSurface{degree:[1,2],size:[2,6],poles,weights,knots:[vec![0.,0.,1.,1.],vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.]],domain:[[0.,1.],[0.,4.]],periodic:[false,true],reversed:false};
    s.certify_repeated_periodic_basis(&limits).unwrap();let face=om9_kernel::measurement::surface_to_face(&s,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();assert_eq!(om9_kernel::measurement::face_periodicity(&face,&limits).unwrap(),[false,true]);
    for v in [0.3,1.5,2.7] {let p=om9_core::surface_kernel::evaluate(&s,0.4,v,&limits).unwrap();let dx=p[0]-1.;let dy=p[1]-1.;let hit=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face.clone()),Transform::identity(),[p[0]+3.*dx,p[1]+3.*dy,p[2]],[-dx,-dy,0.],p,1e-8,&limits).unwrap();assert!(p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7));}
    s.poles[5][0]+=0.01;assert!(s.certify_repeated_periodic_basis(&limits).is_err());assert!(om9_kernel::measurement::surface_to_face(&s,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).is_err());
}

#[test]
fn trimmed_repeated_periodic_surface_preserves_shape_and_support() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=limits();
    let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let mut poles=Vec::new(); let mut weights=Vec::new();
    for (i,[x,y]) in rows.into_iter().enumerate() {
        for z in [0.,2.] { poles.push([x,y,z]); weights.push([1.,0.7,1.2,0.9,1.,0.7][i]); }
    }
    let surface=NurbsSurface{degree:[2,1],size:[6,2],poles,weights,
        knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],
        domain:[[0.3,3.7],[0.,1.]],periodic:[true,false],reversed:false};
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    assert_eq!(om9_kernel::measurement::face_periodicity(&face,&limits).unwrap(),[true,false]);
    for u in [0.35,0.5,1.5,2.7,3.65] {
        let p=om9_core::surface_kernel::evaluate(&surface,u,0.4,&limits).unwrap();
        let dx=p[0]-1.; let dy=p[1]-1.;
        let hit=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face.clone()),Transform::identity(),
            [p[0]+3.*dx,p[1]+3.*dy,p[2]],[-dx,-dy,0.],p,1e-8,&limits).unwrap();
        assert!(p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7),"{p:?} {hit:?}");
    }
}

#[test]
fn doubly_periodic_repeated_surface_preserves_shape_and_axes() {
    use om9_core::geometry::{Geometry,NurbsSurface,Transform};
    let limits=limits();
    let circle=[[1.,0.],[0.,1.],[-1.,0.],[0.,-1.],[1.,0.],[0.,1.]];
    let w=[1.,0.7,1.2,0.9,1.,0.7]; let mut poles=Vec::new(); let mut weights=Vec::new();
    for (i,[x,y]) in circle.into_iter().enumerate() {
        for (j,[radial,z]) in circle.into_iter().enumerate() {
            poles.push([(3.+0.5*radial)*x,(3.+0.5*radial)*y,0.5*z]); weights.push(w[i]*w[j]);
        }
    }
    let knots=vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.];
    let surface=NurbsSurface{degree:[2,2],size:[6,6],poles,weights,
        knots:[knots.clone(),knots],domain:[[0.,4.],[0.,4.]],periodic:[true,true],reversed:false};
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    assert_eq!(om9_kernel::measurement::face_periodicity(&face,&limits).unwrap(),[true,true]);
    for (u,v) in [(0.3,0.4),(1.5,1.2),(2.7,2.5)] {
        let p=om9_core::surface_kernel::evaluate(&surface,u,v,&limits).unwrap();
        let n=om9_core::surface_kernel::normal(&surface,u,v,&limits).unwrap();
        let origin=[p[0]+n[0]*0.2,p[1]+n[1]*0.2,p[2]+n[2]*0.2];
        let hit=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face.clone()),Transform::identity(),
            origin,[-n[0],-n[1],-n[2]],p,1e-8,&limits).unwrap();
        assert!(p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7),"{p:?} {hit:?}");
    }
}
