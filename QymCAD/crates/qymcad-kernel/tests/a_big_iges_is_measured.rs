//! HOW LONG A BIG IGES TAKES - a measurement on a file of this machine, named by `QYM_IGES`.
//!
//! Reported behaviour: a 145 MB assembly from another CAD kept the import spinner going with no end in sight.
#[test]
#[ignore = "a file on this machine"]
fn how_long_a_big_iges_takes() {
    let Ok(path) = std::env::var("QYM_IGES") else { return };
    let t = std::time::Instant::now();
    match qymcad_kernel::read_exact(qymcad_kernel::ExactFormat::Iges, &path, 0.5) {
        Ok((bodies, shapes)) => {
            let tris: usize = bodies.iter().map(|qymcad_core::geom::Built { mesh: m, .. }| m.tris.len()).sum();
            eprintln!("PROBE read_exact: {} bodies, {} live solids, {tris} triangles in {:.1} s", bodies.len(), shapes.len(), t.elapsed().as_secs_f64());
        }
        Err(e) => eprintln!("PROBE read_exact: {e} after {:.1} s", t.elapsed().as_secs_f64()),
    }
}
