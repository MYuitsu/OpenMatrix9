//! A BOOLEAN ON SEVERAL CORES MUST GIVE THE SAME BODY AS ON ONE.
//!
//! The low-level boolean algorithms lie when run in parallel - not by crashing but by returning geometry that
//! is almost right. A part that builds "almost right" is worse than one that refuses.
//!
//! So every test document is built twice, on one core and on all, and the results are compared: the volume,
//! the surface area, the bounding box, the number of faces AND THEIR NAMES. The names matter as much as the
//! volume: references (a fillet on an edge, a sketch on a face, a mate) are kept by name, and a parallel pass
//! that numbers the faces in another order would move every reference in the document.
use qymcad_core::model::Project;
use std::collections::BTreeMap;

/// What a document is judged by: for every body, the numbers a difference would show up in.
type Fingerprint = BTreeMap<u64, (i64, i64, [i64; 6], usize, Vec<u32>)>;

fn fingerprint(p: &Project, refused: &[u64]) -> Fingerprint {
    assert!(refused.len() < 3, "the document refused {} nodes, which is too many for the comparison to be about the threads: {refused:?}", refused.len());
    let r = |v: f64| (v * 1e6).round() as i64; // a micron-cubed grid: numbers are compared, not bits
    p.bodies
        .iter()
        .map(|b| {
            let bb = b.mesh.bounds().map(|x| [r(x.min.x), r(x.min.y), r(x.min.z), r(x.max.x), r(x.max.y), r(x.max.z)]).unwrap_or([0; 6]);
            let mut names: Vec<u32> = b.faces.iter().map(|f| f.id).collect();
            names.sort_unstable();
            // the surface area as the sum over the triangles: a second number that a boolean gone wrong moves
            let area: f64 = (0..b.mesh.tris.len())
                .map(|t| {
                    let q = b.mesh.triangle(t);
                    let (u, v) = ([q[1].x - q[0].x, q[1].y - q[0].y, q[1].z - q[0].z], [q[2].x - q[0].x, q[2].y - q[0].y, q[2].z - q[0].z]);
                    let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                    (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5
                })
                .sum();
            (b.id, (r(b.mesh.volume()), r(area), bb, b.mesh.tris.len(), names))
        })
        // THE REFUSALS ARE PART OF THE ANSWER: a rebuild that quietly gave up on a different node would
        // otherwise pass, having simply left that body as it was.
        .chain(std::iter::once((0u64, (refused.len() as i64, refused.iter().map(|n| *n as i64).sum(), [0; 6], 0, Vec::new()))))
        .collect()
}

/// ONE AT A TIME, because the setting is the kernel's and the kernel is one.
///
/// The checks in this file each set the number of threads and then measure what came out. Run side by side -
/// which is how a test binary runs its checks - they overwrite one another's setting, and a check comparing
/// "one thread against many" compares two runs that were both given the same number. Caught by the full run:
/// alone each check passed, together two of them failed.
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The same document, built with the kernel set one way or the other.
fn built(make: &dyn Fn() -> Project, parallel: bool) -> (Fingerprint, f64) {
    let _one_at_a_time = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    qymcad_kernel::set_parallel(parallel, if parallel { 0 } else { 1 });
    assert_eq!(qymcad_kernel::workers() > 1, parallel, "setup: the kernel was not told how many threads to take");
    let mut p = make();
    let t = std::time::Instant::now();
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    let took = t.elapsed().as_secs_f64() * 1000.0;
    // A DOCUMENT MAY HOLD A NODE THAT REFUSES, and that is not what is being compared here: what matters is
    // that BOTH rebuilds refuse the same nodes. A test document of the machine has one such (a mirrored part
    // whose source is gone), and it refuses in one thread exactly as it does in eight.
    let refused: Vec<u64> = report.errors.iter().map(|(n, _)| *n).collect();
    (fingerprint(&p, &refused), took)
}

/// A plate with a grid of holes cut in one node - many tools in one boolean, which is what the flag divides.
fn plate_with_holes(n: usize) -> Project {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("plate");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "plate");
    p.add_rect_entity(si, 0.0, 0.0, 200.0, 200.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let body = p.add_extrude(sid, 10.0);
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
    let span = qymcad_core::model::CombineSpan { height: 20.0, down: 0.0, extent: qymcad_core::feature::Extent::default(), fill: &[] };
    let cut = p.add_combine_multi_op(body, sid2, profiles, span, 0);
    p.finish_base_body(cut, 1);
    p
}

/// A part whose chain goes through several kinds of boolean: a cut, a union, a fillet and a chamfer on top.
fn a_part_with_a_chain() -> Project {
    let mut p = Project::default();
    p.new_document();
    let si = p.new_sketch("base");
    let sid = p.sketches[si].id;
    p.add_sketch_node(sid, "base");
    p.add_rect_entity(si, 0.0, 0.0, 60.0, 40.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si);
    let body = p.add_extrude(sid, 20.0);
    let si2 = p.new_sketch("pocket");
    let sid2 = p.sketches[si2].id;
    p.add_sketch_node(sid2, "pocket");
    p.add_rect_entity(si2, 10.0, 10.0, 50.0, 30.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si2);
    let cut = p.add_combine(body, sid2, 10.0, 0);
    let si3 = p.new_sketch("boss");
    let sid3 = p.sketches[si3].id;
    p.add_sketch_node(sid3, "boss");
    p.add_circle_entity(si3, 30.0, 20.0, 8.0, qymcad_core::feature::Purpose::Real);
    p.regen_sketch(si3);
    let boss = p.add_combine(cut, sid3, 25.0, 1);
    p.finish_base_body(boss, 1);
    p
}

#[test]
fn a_plate_of_two_hundred_holes_comes_out_the_same_on_one_core_and_on_all() {
    // twenty holes rather than two hundred: the same path through the kernel, a check that runs in a second
    let (one, t1) = built(&|| plate_with_holes(20), false);
    let (many, t8) = built(&|| plate_with_holes(20), true);
    eprintln!("  one core {t1:.0} ms, all cores {t8:.0} ms");
    assert_eq!(one, many, "a boolean on several cores gave a different body");
    assert!(!one.is_empty(), "setup: nothing was built at all");
}

#[test]
fn a_chain_of_booleans_comes_out_the_same_on_one_core_and_on_all() {
    let (one, _) = built(&a_part_with_a_chain, false);
    let (many, _) = built(&a_part_with_a_chain, true);
    assert_eq!(one, many, "a chain of booleans on several cores gave a different body");
}

/// TEN TIMES OVER: a race need not show itself the first time.
#[test]
fn the_answer_does_not_wander_between_runs() {
    let (first, _) = built(&|| plate_with_holes(20), true);
    for round in 2..=10 {
        let (again, _) = built(&|| plate_with_holes(20), true);
        assert_eq!(first, again, "run {round} on several cores gave a different body from the first");
    }
}

/// A WHOLE DOCUMENT, REBUILT IN ONE THREAD AND IN MANY, must come out the same to the last face name.
///
/// The parcels of a wave are computed side by side, and the danger is not a
/// crash but a part that is almost right. `QYM_DOCS` names the documents; with none given the scenario one is
/// taken, which holds 178 nodes of every kind - threads, splits, patches, arrays.
#[test]
#[ignore = "needs a document on this machine; the plain checks above run always"]
fn a_whole_document_comes_out_the_same_in_one_thread_and_in_many() {
    let docs = std::env::var("QYM_DOCS").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/user-case.qcad").to_string());
    for path in docs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let load = || qymcad_io::load_project(path).expect("the document opens");
        let (one, t1) = built(&load, false);
        let (many, t8) = built(&load, true);
        eprintln!("  {path}: one thread {t1:.0} ms, all the cores {t8:.0} ms");
        assert!(!one.is_empty(), "setup: the document built nothing");
        assert_eq!(one.len(), many.len(), "the two rebuilds left a different number of bodies");
        for (id, mine) in &one {
            let theirs = many.get(id).expect("the same body is in both rebuilds");
            assert_eq!(mine, theirs, "body {id} came out different when the rebuild was divided between threads");
        }
    }
}

/// THE ORDER OF THE FACE NAMES, checked apart from the volumes.
///
/// References are kept by name (a fillet on an edge, a sketch on a face, a mate), and a name is minted while
/// the node is prepared. Were the preparation to move into the workers, the same part would get its names in
/// whatever order the threads happened to finish - and every reference in the document would point somewhere
/// else. The volumes would still match, which is why this is asked separately.
#[test]
fn the_faces_are_named_in_the_same_order_however_many_threads_there_are() {
    let order = |parallel: bool| -> Vec<Vec<u32>> {
        let _one_at_a_time = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        qymcad_kernel::set_parallel(parallel, if parallel { 0 } else { 1 });
        let mut p = plate_with_holes(20);
        let (report, _) = qymcad_testkit::regenerate(&mut p);
        assert!(report.errors.is_empty(), "the plate did not build: {:?}", report.errors);
        // AS BUILT, not sorted: the question is the order, and a sorted list would answer another one
        p.bodies.iter().map(|b| b.faces.iter().map(|f| f.id).collect()).collect()
    };
    assert_eq!(order(false), order(true), "the faces were named in another order when the rebuild was divided between threads");
}

/// THE SWITCH REALLY SWITCHES: at one thread nothing is computed side by side.
///
/// A setting a person turns to when a part comes out strange has to do what it says, and "the geometry is the
/// same either way" - which every check above asserts - would be equally true if the setting did nothing at
/// all. So this asks the rebuild itself: with one thread the report shows no batch, with all the cores it
/// shows one.
#[test]
fn one_thread_means_nothing_is_computed_side_by_side() {
    let batches = |parallel: bool| -> Vec<usize> {
        let _one_at_a_time = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        qymcad_kernel::set_parallel(parallel, if parallel { 0 } else { 1 });
        // three parts that know nothing of one another: there IS something to compute at once
        let mut p = Project::default();
        p.new_document();
        for k in 0..3 {
            let root = p.root;
            p.set_active_component(Some(root));
            let part = p.add_part(format!("Part {k}"));
            p.set_active_component(Some(part));
            let si = p.new_sketch("base");
            let sid = p.sketches[si].id;
            p.add_sketch_node(sid, "base");
            p.add_rect_entity(si, k as f64 * 40.0, 0.0, k as f64 * 40.0 + 20.0, 20.0, qymcad_core::feature::Purpose::Real);
            p.regen_sketch(si);
            p.add_extrude(sid, 10.0);
        }
        let (report, _) = qymcad_testkit::regenerate(&mut p);
        assert!(report.errors.is_empty(), "the three parts did not build: {:?}", report.errors);
        report.waves
    };
    assert!(batches(false).is_empty(), "at one thread something was still computed side by side");
    assert!(!batches(true).is_empty(), "at all the cores nothing was computed side by side - then the switch is about nothing");
}

/// EVERY NODE IS BUILT ONCE PER REBUILD, however the batches fall.
///
/// A batch is written into the document as soon as it is computed, ahead of the walk down the timeline. The
/// next batch then starts from a place ABOVE those nodes, and each of them still looks as if it has to be
/// rebuilt - its input is dirty. Measured on the scenario document: 783 timings over 134 nodes, the same five
/// nodes rebuilt 74 times each, once per batch, and the rebuild slower than on one thread.
///
/// Three parts with a chain of cuts each is the smallest thing that does it: the heads go together, then the
/// first cuts, and the walk returns to the second cut of the first part with the others already written.
#[test]
fn every_node_is_built_once_however_the_batches_fall() {
    let _one_at_a_time = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    qymcad_kernel::set_parallel(true, 0);
    let mut p = Project::default();
    p.new_document();
    for k in 0..3 {
        let x = k as f64 * 40.0;
        let root = p.root;
        p.set_active_component(Some(root));
        let part = p.add_part(format!("Part {k}"));
        p.set_active_component(Some(part));
        let si = p.new_sketch("base");
        let sid = p.sketches[si].id;
        p.add_sketch_node(sid, "base");
        p.add_rect_entity(si, x, 0.0, x + 20.0, 20.0, qymcad_core::feature::Purpose::Real);
        p.regen_sketch(si);
        let mut body = p.add_extrude(sid, 10.0);
        for c in 0..3 {
            let si = p.new_sketch("cut");
            let sid = p.sketches[si].id;
            p.add_sketch_node(sid, "cut");
            let at = x + 2.0 + c as f64 * 5.0;
            p.add_rect_entity(si, at, 2.0, at + 3.0, 8.0, qymcad_core::feature::Purpose::Real);
            p.regen_sketch(si);
            body = p.add_combine(body, sid, 20.0, 0);
        }
    }
    let (report, _) = qymcad_testkit::regenerate(&mut p);
    assert!(report.errors.is_empty(), "the parts did not build: {:?}", report.errors);
    assert!(!report.waves.is_empty(), "nothing was computed side by side, so this check is about nothing");
    let mut times: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for (id, _, _) in &report.spent {
        *times.entry(*id).or_insert(0) += 1;
    }
    let again: Vec<(u64, usize)> = times.into_iter().filter(|(_, n)| *n > 1).collect();
    assert!(again.is_empty(), "nodes were built more than once in one rebuild (node, times): {again:?}");
}

/// AND THE CHECK ITSELF IS NOT BLIND: a fingerprint that notices nothing would pass every test above.
#[test]
fn the_fingerprint_notices_a_difference() {
    let (holes20, _) = built(&|| plate_with_holes(20), true);
    let (holes21, _) = built(&|| plate_with_holes(21), true);
    assert_ne!(holes20, holes21, "the fingerprint did not tell twenty holes from twenty-one");
}
