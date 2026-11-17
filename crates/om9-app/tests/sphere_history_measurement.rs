// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

//! B0 diagnostic for the current product Sphere and delete routes.
//! The liveness witnesses are weak and do not keep display buffers alive.
use om9_app::session::Controller;
use om9_core::{brep::BrepDisplay, budget::{Limits, Usage}, geometry::Geometry, model::Document};
use std::time::Instant;

fn display_bytes(display: &BrepDisplay) -> u64 {
    Usage::array::<BrepDisplay>(1).unwrap().bytes + Usage::array::<usize>(2).unwrap().bytes
        + Usage::array::<[f64;3]>(display.points.capacity()).unwrap().bytes
        + Usage::array::<[u32;4]>(display.triangles.capacity()).unwrap().bytes
        + Usage::array::<[u32;3]>(display.segments.capacity()).unwrap().bytes
        + Usage::array::<u32>(display.vertices.capacity()).unwrap().bytes
}

#[test]
#[ignore = "manual B0 measurement; produces 32 and 128 product Spheres"]
fn b0_product_sphere_history_default_profile() {
    for count in [32usize,128] {
        let mut c = Controller::new(Document::new("B0 product Sphere"),Limits::default()).unwrap();
        let mut seen = std::collections::BTreeSet::new();
        let mut records = Vec::new();
        let mut sampled_peak = 0u64;
        for _ in 0..count {
            c.submit("Sphere 5").unwrap();
            let (&id, entity) = c.document().entities.iter().find(|(id,_)| !seen.contains(*id)).unwrap();
            seen.insert(id);
            let Geometry::Brep(shape) = &entity.geometry else { panic!("Sphere is not B-rep") };
            assert_eq!(shape.display().deflection_mm,0.01);
            records.push((id,shape.display_liveness(),display_bytes(shape.display()),shape.source().len()));
            sampled_peak = sampled_peak.max(c.coordinator().retained_usage().unwrap().bytes + c.scene_cache_usage().unwrap().bytes);
        }
        for (id,_,_,_) in records.iter().rev() {
            assert_eq!(c.delete_objects(&[*id]).unwrap(),1);
            sampled_peak = sampled_peak.max(c.coordinator().retained_usage().unwrap().bytes + c.scene_cache_usage().unwrap().bytes);
        }
        assert!(c.document().entities.is_empty());
        let live: Vec<_> = records.iter().filter(|(_,weak,_,_)| weak.is_alive()).collect();
        let display: u64 = live.iter().map(|(_,_,bytes,_)| *bytes).sum();
        let source_payload: usize = live.iter().map(|(_,_,_,bytes)| *bytes).sum();
        let history = c.coordinator().undo_memory_bytes().unwrap();
        let steps = c.coordinator().undo_len();
        let start = Instant::now();
        c.undo().unwrap();
        let undo_ms = start.elapsed().as_secs_f64()*1000.;
        assert_eq!(c.document().entities.len(),1);
        println!("count={count} deflection_mm=0.01 remaining_steps={steps} live_displays={} display_bytes={display} source_text_bytes={source_payload} history_bytes={history} sampled_retained_plus_scene_peak_bytes={sampled_peak} first_undo_ms={undo_ms:.3}",live.len());
    }
}
