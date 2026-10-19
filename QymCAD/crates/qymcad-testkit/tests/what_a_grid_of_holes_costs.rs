//! A PLATE WITH A GRID OF HOLES - the case the parallel boolean is about.
//!
//! A plate of 200x200 with two hundred holes cut in ONE operation. One boolean over many tools is where OCCT's
//! own parallelism has something to divide; a document of many small cuts would measure something else
//! entirely.
//!
//! `QYM_HOLES` sets how many (200 by default), `QYM_ROUNDS` how many times to repeat the measurement.
use qymcad_core::model::Project;

/// HOW MUCH PROCESSOR TIME THIS PROCESS HAS SPENT, in milliseconds - user plus system, over all threads.
///
/// The question "is the boolean really running on several cores" cannot be answered by the clock on the wall:
/// a slower machine and a single-threaded run look alike. Processor time greater than wall time can only come
/// from several cores having worked at once.
fn cpu_ms() -> f64 {
    let Ok(stat) = std::fs::read_to_string("/proc/self/stat") else { return 0.0 };
    // fields 14 and 15 after the executable name in brackets: utime and stime, in clock ticks
    let Some(rest) = stat.rsplit(')').next() else { return 0.0 };
    let f: Vec<&str> = rest.split_whitespace().collect();
    let ticks: f64 = f.get(11).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) + f.get(12).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    ticks / 100.0 * 1000.0 // the usual 100 ticks per second
}

/// The plate with `n` holes cut in one node, and how long the rebuild took in milliseconds.
fn plate_with_holes(n: usize) -> (Project, f64) {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("plate");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "plate");
    p.add_rect_entity(si, 0.0, 0.0, 200.0, 200.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let body = p.add_extrude(sid, 10.0);

    // the holes: a square grid inside the plate, all in one sketch, so the cut is a single node
    let si2 = p.new_sketch("holes");
    let sid2 = p.sketches[si2].id;
    p.add_sketch_node(sid2, "holes");
    let side = (n as f64).sqrt().ceil() as usize;
    let step = 180.0 / side as f64;
    let mut made = 0usize;
    for row in 0..side {
        for col in 0..side {
            if made == n {
                break;
            }
            p.add_circle_entity(si2, 10.0 + col as f64 * step, 10.0 + row as f64 * step, step * 0.3, qymcad_core::feature::Purpose::Real);
            made += 1;
        }
    }
    p.regen_sketch(si2);
    let profiles: Vec<qymcad_core::model::Id> = p.sketches[si2].contour_ids.clone();
    assert_eq!(profiles.len(), n, "setup: the sketch gave {} contours instead of {n}", profiles.len());
    let span = qymcad_core::model::CombineSpan { height: 20.0, down: 0.0, extent: qymcad_core::feature::Extent::default(), fill: &[] };
    let cut = p.add_combine_multi_op(body, sid2, profiles, span, 0);
    p.finish_base_body(cut, 1);

    let (t, cpu0) = (std::time::Instant::now(), cpu_ms());
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    let (took, cpu) = (t.elapsed().as_secs_f64() * 1000.0, cpu_ms() - cpu0);
    assert!(report.errors.is_empty(), "the plate with {n} holes did not build: {:?}", report.errors);
    let cut_us: u128 = report.spent.iter().filter(|(_, name, _)| name.contains("combine")).map(|(_, _, us)| us).sum();
    eprintln!("  {n} holes: the whole rebuild {took:.0} ms, of it the cut {:.0} ms; processor time {cpu:.0} ms, that is {:.2} cores", cut_us as f64 / 1000.0, cpu / took.max(1.0));
    (p, took)
}

#[test]
#[ignore = "a measurement, not a check: prints numbers"]
fn what_a_grid_of_holes_costs() {
    let n: usize = std::env::var("QYM_HOLES").ok().and_then(|s| s.parse().ok()).unwrap_or(200);
    let rounds: usize = std::env::var("QYM_ROUNDS").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    eprintln!("MEASURED a plate of 200x200x10 with a grid of holes cut in one node");
    for _ in 0..rounds {
        let (p, _) = plate_with_holes(n);
        let body = p.bodies.iter().max_by_key(|b| b.mesh.tris.len()).expect("a body");
        eprintln!("    the result: {} triangles, {} faces", body.mesh.tris.len(), body.faces.len());
    }
}
