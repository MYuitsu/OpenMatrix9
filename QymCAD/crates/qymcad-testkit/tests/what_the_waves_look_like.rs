//! HOW WIDE THE WAVES OF A REBUILD ARE - a measurement.
//!
//! If the widest wave of a real document holds one or two nodes, there is nothing to compute side by side.
//! The ceiling of any speed-up is the number of nodes divided by the number of waves - not the number of cores.
//!
//! `QYM_DOCS` names the documents, separated by commas; with none given the scenario document is taken.

/// A FULL rebuild is what is measured, so everything is marked dirty first - that is what opening a
/// file without geometry, or changing a parameter at the very top, comes to.
fn waves_of(path: &str) -> Option<(usize, Vec<usize>)> {
    let mut project = qymcad_io::load_project(path).ok()?;
    for n in &mut project.timeline {
        n.dirty = true;
    }
    let waves = project.rebuild_waves();
    let nodes: usize = waves.iter().map(|w| w.len()).sum();
    Some((nodes, waves.iter().map(|w| w.len()).collect()))
}

/// NO NODE STANDS IN A WAVE BEFORE WHAT IT DEPENDS ON - asked of a real document, not of a synthetic one.
#[test]
#[ignore = "a measurement over a document on this machine"]
fn on_a_real_document_no_node_stands_before_its_input() {
    let docs = std::env::var("QYM_DOCS").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/user-case.qcad").to_string());
    for path in docs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Ok(mut project) = qymcad_io::load_project(path) else { continue };
        for n in &mut project.timeline {
            n.dirty = true;
        }
        let waves = project.rebuild_waves();
        let wave_of: std::collections::HashMap<u64, usize> = waves.iter().enumerate().flat_map(|(w, ids)| ids.iter().map(move |id| (*id, w))).collect();
        // who makes what
        let mut maker: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
        for n in &project.timeline {
            for d in n.kind.declares() {
                maker.entry(d).or_insert(n.id);
            }
        }
        let mut bad = Vec::new();
        for n in &project.timeline {
            let Some(&mine) = wave_of.get(&n.id) else { continue };
            for input in n.kind.inputs() {
                let Some(&who) = maker.get(&input) else { continue };
                let Some(&theirs) = wave_of.get(&who) else { continue };
                if theirs >= mine {
                    bad.push(format!("{} (wave {mine}) takes {input} made by {who} (wave {theirs})", n.name));
                }
            }
        }
        eprintln!("MEASURED {path}: {} nodes stand no later than their input", bad.len());
        for b in bad.iter().take(8) {
            eprintln!("    {b}");
        }
    }
}

#[test]
#[ignore = "a measurement, not a check: prints numbers"]
fn what_the_waves_look_like() {
    let docs = std::env::var("QYM_DOCS").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/user-case.qcad").to_string());
    for path in docs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Some((nodes, shape)) = waves_of(path) else {
            eprintln!("MEASURED {path}: the document does not open");
            continue;
        };
        let widest = shape.iter().copied().max().unwrap_or(0);
        let ceiling = nodes as f64 / shape.len().max(1) as f64;
        eprintln!("MEASURED {path}\n  nodes {nodes}, waves {}, the widest {widest}, the ceiling of any speed-up {ceiling:.1}x\n  the waves: {:?}", shape.len(), &shape[..shape.len().min(40)]);
    }
}
