//! WHAT A REBUILD COSTS, node by node - a measurement, not a check.
//!
//! Every speed-up starts from this number. Without it "the booleans are slow" is a belief, and a day spent on a
//! parallel flag may buy 5 % of the time.
//!
//! `QYM_DOCS` names the documents to measure, separated by commas; with none given the scenario document is
//! taken. The table goes to the terminal and to `target/rebuild-cost/<document>.txt`.
use std::collections::HashMap;

/// One line of the table: what kind of node it was, how many of them, how long they took together.
fn by_kind(spent: &[(u64, String, u128)]) -> Vec<(String, usize, u128)> {
    let mut sum: HashMap<&str, (usize, u128)> = HashMap::new();
    for (_, name, us) in spent {
        let e = sum.entry(name.as_str()).or_insert((0, 0));
        e.0 += 1;
        e.1 += us;
    }
    let mut out: Vec<(String, usize, u128)> = sum.into_iter().map(|(k, (n, us))| (k.to_string(), n, us)).collect();
    out.sort_by_key(|(_, _, us)| std::cmp::Reverse(*us));
    out
}

/// The kinds that go through a boolean in the kernel - the ones the parallel flag is about.
fn is_boolean(name: &str) -> bool {
    ["cut", "fuse", "common", "boolean", "hole", "combine", "pocket", "slot", "thread", "trim", "split"].iter().any(|k| name.contains(k))
}

#[test]
#[ignore = "a measurement, not a check: prints numbers"]
fn what_a_rebuild_costs() {
    // `QYM_THREADS` says how many threads the rebuild may take: 1 is single-threaded, 0 all the cores but one
    if let Ok(n) = std::env::var("QYM_THREADS").map(|s| s.parse::<i32>().unwrap_or(0)) {
        qymcad_kernel::set_parallel(n != 1, n);
        eprintln!("THREADS {n} (the kernel gives {})", qymcad_kernel::workers());
    }
    let docs = std::env::var("QYM_DOCS").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/user-case.qcad").to_string());
    for path in docs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let t0 = std::time::Instant::now();
        let Ok(mut project) = qymcad_io::load_project(path) else {
            eprintln!("MEASURED {path}: the document does not open");
            continue;
        };
        let opened = t0.elapsed();

        // THE WAVES ARE ASKED BEFORE THE REBUILD, and of a document marked dirty throughout - that is the
        // full recount. A freshly opened document
        // carries no dirty marks, and the plan would then hold two nodes instead of all of them.
        //
        // The marks are put on a COPY: setting them on the document being measured changes the rebuild that
        // follows, and the measurement would then be of something else (measured: 139 bodies became 129).
        let waves = {
            let mut all_dirty = project.clone();
            for n in &mut all_dirty.timeline {
                n.dirty = true;
            }
            all_dirty.rebuild_waves()
        };

        let t1 = std::time::Instant::now();
        let (report, _shapes) = qymcad_testkit::regenerate(&mut project);
        let rebuilt = t1.elapsed();

        let total: u128 = report.spent.iter().map(|(_, _, us)| us).sum();
        let booleans: u128 = report.spent.iter().filter(|(_, n, _)| is_boolean(n)).map(|(_, _, us)| us).sum();
        let mut lines = vec![format!(
            "MEASURED {path}\n  nodes {}, bodies {}, opened in {:.0} ms, rebuilt in {:.0} ms (of that inside the nodes {:.0} ms)\n  \
             through a boolean: {:.0} ms, that is {:.0} % of the rebuild\n  errors {}",
            project.timeline.len(),
            project.bodies.len(),
            opened.as_secs_f64() * 1000.0,
            rebuilt.as_secs_f64() * 1000.0,
            total as f64 / 1000.0,
            booleans as f64 / 1000.0,
            booleans as f64 / total.max(1) as f64 * 100.0,
            report.errors.len(),
        )];
        lines.push(format!("    computed side by side: {} batches, sizes {:?}", report.waves.len(), &report.waves[..report.waves.len().min(20)]));
        // A NODE TIMED TWICE was rebuilt twice in one pass, and that is work thrown away rather than a cost.
        let mut times_of: HashMap<u64, usize> = HashMap::new();
        for (id, _, _) in &report.spent {
            *times_of.entry(*id).or_insert(0) += 1;
        }
        let again = times_of.values().filter(|n| **n > 1).count();
        lines.push(format!("    timed {} times over {} distinct nodes; rebuilt more than once: {again}", report.spent.len(), times_of.len()));
        let mut repeated: Vec<(u64, usize)> = times_of.iter().filter(|(_, n)| **n > 1).map(|(id, n)| (*id, *n)).collect();
        repeated.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        for (id, n) in repeated.iter().take(6) {
            let at = project.timeline.iter().position(|nd| nd.id == *id).unwrap_or(usize::MAX);
            let name = project.timeline.get(at).map(|nd| nd.name.clone()).unwrap_or_default();
            lines.push(format!("      {name} (node {id}, place {at}) rebuilt {n} times"));
        }
        // THE CEILING OF THE SPEED-UP, COUNTED IN TIME RATHER THAN IN NODES.
        //
        // "A wave of 1296 nodes" promises 648x only while every node costs the same. They do not: on one
        // document a single node is 94 % of the rebuild, and no number of cores moves it. So the waves are
        // weighed by what each node actually took, and a wave costs at least its heaviest node.
        let spent_of: HashMap<u64, u128> = report.spent.iter().map(|(id, _, us)| (*id, *us)).collect();
        // Only a rebuild with nothing computed side by side knows what each node costs: a node from a batch is
        // timed while it is written down, its computing having happened in the batch (a thread of 2.1 s shows
        // as 0.2 s). So the ceiling is counted on one thread, and elsewhere the line says why it is absent.
        if report.waves.is_empty() {
            let hands = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).saturating_sub(1).max(1) as f64;
            let mut ideal = 0.0f64;
            let mut widest = (0usize, 0u128);
            for w in &waves {
                let times: Vec<u128> = w.iter().filter_map(|id| spent_of.get(id).copied()).collect();
                let sum: u128 = times.iter().sum();
                let top = times.iter().copied().max().unwrap_or(0);
                if sum > widest.1 {
                    widest = (w.len(), sum);
                }
                ideal += (sum as f64 / hands).max(top as f64);
            }
            lines.push(format!(
                "    waves {}, the dearest holds {} nodes and {:.0} ms; on {hands:.0} hands the whole would take at best {:.0} ms - a ceiling of {:.2}x, counted in time",
                waves.len(),
                widest.0,
                widest.1 as f64 / 1000.0,
                ideal / 1000.0,
                total as f64 / ideal.max(1.0)
            ));
        } else {
            lines.push("    the ceiling is counted on one thread (QYM_THREADS=1): here a batched node is timed only while it is written down".to_string());
        }
        for (node, e) in report.errors.iter().take(6) {
            let name = project.timeline.iter().find(|n| n.id == *node).map(|n| n.name.clone()).unwrap_or_default();
            lines.push(format!("    failed: {name} (node {node}): {e:?}"));
        }
        for (name, n, us) in by_kind(&report.spent).into_iter().take(12) {
            lines.push(format!("    {name}: {n} of them, {:.0} ms together, {:.1} ms each", us as f64 / 1000.0, us as f64 / 1000.0 / n as f64));
        }
        // and the heaviest single nodes, because one node of six seconds and six hundred of ten milliseconds
        // call for different work
        let mut worst = report.spent.clone();
        worst.sort_by_key(|(_, _, us)| std::cmp::Reverse(*us));
        for (id, name, us) in worst.iter().take(5) {
            lines.push(format!("    the heaviest: {name} (node {id}) {:.0} ms", *us as f64 / 1000.0));
        }
        let text = lines.join("\n");
        eprintln!("{text}");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/rebuild-cost");
        std::fs::create_dir_all(&dir).ok();
        let stem = std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "document".into());
        std::fs::write(dir.join(format!("{stem}.txt")), &text).ok();
    }
}
