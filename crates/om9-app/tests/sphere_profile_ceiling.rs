// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]
//! Profile-size diagnostic only: this does not create product history.
use om9_core::{brep::BrepDisplay, budget::{Limits, Usage}};
use om9_kernel::creation::{self,Primitive};

fn display_bytes(d: &BrepDisplay) -> u64 {
    Usage::array::<BrepDisplay>(1).unwrap().bytes + Usage::array::<usize>(2).unwrap().bytes
        + Usage::array::<[f64;3]>(d.points.capacity()).unwrap().bytes
        + Usage::array::<[u32;4]>(d.triangles.capacity()).unwrap().bytes
        + Usage::array::<[u32;3]>(d.segments.capacity()).unwrap().bytes
        + Usage::array::<u32>(d.vertices.capacity()).unwrap().bytes
}

#[test]
fn sphere_profiles_same_analytic_source_and_display_upper_bound() {
    let limits=Limits::default();
    let mut sources=Vec::new();
    for deflection in [0.1,0.01] {
        let capture=creation::primitive_with_topology(Primitive::Sphere { radius_mm: 5. },deflection,&limits,|_,_| Ok(())).unwrap();
        let (brep,proof)=capture.into_parts();
        let volume=creation::measure(&brep,&limits).unwrap().volume_mm3;
        assert!((volume-4.*std::f64::consts::PI*125./3.).abs()<1e-6);
        let bytes=display_bytes(brep.display());
        println!("deflection_mm={deflection} display_bytes={bytes} source_text_bytes={} source_topology_display_bytes={} topology_proof_bytes={} 128_independent_displays_upper_bound_bytes={} points={} triangles={}",brep.source().len(),brep.storage_usage(&limits).unwrap().bytes,proof.storage_bytes().unwrap(),bytes*128,brep.display().points.len(),brep.display().triangles.len());
        sources.push(brep.source().to_owned());
    }
    assert_eq!(sources[0],sources[1]);
}
