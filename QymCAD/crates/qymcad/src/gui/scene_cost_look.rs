//! WHAT THE SCENE COSTS: how many vertices, how many bytes, how long to assemble and to hand over.
//!
//! A measurement, not a check. Optimising what nobody measured is belief rather than work: a number comes
//! before every change to the scene, and a change the number does not justify is not made.
//!
//! By default it measures the scenario document; `QYM_SCENE_DOC` points it at any other file - the heavy
//! reference STEP included, once it has been saved as ours.
#[cfg(test)]
mod tests {
    use super::super::App;

    #[test]
    #[ignore = "a measurement, not a check: prints numbers"]
    fn what_the_scene_costs() {
        let path = std::env::var("QYM_SCENE_DOC").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/user-case.qcad").to_string());
        let t0 = std::time::Instant::now();
        let project = qymcad_io::load_project(&path).expect("the document opens");
        let opened = t0.elapsed();

        let mut app = App::default();
        app.finish_project_load(path.clone(), project, Vec::new());
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());

        // the first assembly: every block is built
        let t1 = std::time::Instant::now();
        let scene = crate::gui::render_scene::gpu_scene(&app.painting());
        let built = t1.elapsed();
        let verts: usize = scene.pieces.iter().map(|p| p.verts.len()).sum();
        let idx: usize = scene.pieces.iter().map(|p| p.idx.len()).sum();
        let bytes = verts * std::mem::size_of::<qymcad_ui_state::GpuVert>() + idx * 4;

        // the second: nothing changed, every block is taken ready-made
        let t2 = std::time::Instant::now();
        let _ = crate::gui::render_scene::gpu_scene(&app.painting());
        let again = t2.elapsed();

        // WHAT EVERY FRAME PAYS, even when nothing has changed: the key of the scene and the look table. The
        // geometry above is touched only when the key moves; these two are computed always, so a slow one
        // here is felt as "the whole program is sluggish", settings windows included.
        let t3 = std::time::Instant::now();
        for _ in 0..10 {
            let _ = qymcad_ui_state::gpu_scene_key(&app.painting());
        }
        let key_ms = t3.elapsed().as_secs_f64() * 100.0;
        let t4 = std::time::Instant::now();
        for _ in 0..10 {
            let _ = crate::gui::render_scene::scene_looks(&app.painting());
        }
        let looks_ms = t4.elapsed().as_secs_f64() * 100.0;
        let rss = std::fs::read_to_string("/proc/self/statm").ok().and_then(|s| s.split_whitespace().nth(1).and_then(|p| p.parse::<u64>().ok())).map(|pages| pages * 4096 / 1048576).unwrap_or(0);

        let stats = app.cache.scene_stats.get();
        eprintln!(
            "MEASURED {path}\n  bodies {}, pieces {}, vertices {verts}, indices {idx} ({:.1} triangles per vertex), {:.1} MB ({} bytes per vertex plus 4 per index)\n  \
             opening {:.0} ms, first assembly {:.0} ms, second {:.1} ms\n  \
             blocks: built {}, shifted {}, taken ready-made {}",
            app.project.bodies.len(),
            scene.pieces.len(),
            idx as f64 / 3.0 / verts.max(1) as f64,
            bytes as f64 / 1048576.0,
            std::mem::size_of::<qymcad_ui_state::GpuVert>(),
            opened.as_secs_f64() * 1000.0,
            built.as_secs_f64() * 1000.0,
            again.as_secs_f64() * 1000.0,
            stats[0],
            stats[1],
            stats[2],
        );
        eprintln!("  rows in the look table {}", scene.looks.len());
        eprintln!("  EVERY FRAME: the scene key {key_ms:.1} ms, the look table {looks_ms:.1} ms; resident {rss} MB");
    }
}
