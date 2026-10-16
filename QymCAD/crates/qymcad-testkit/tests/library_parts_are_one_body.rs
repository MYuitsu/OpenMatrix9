//! THE PARTS THAT COME WITH THE PROGRAM ARE PARTS OF TODAY: one body each, built from the timeline as it reads now.
//!
//! Reported behaviour: a part taken from the built-in library did not load at all, and once it loaded it came in as
//! several bodies - it was saved before a part became one body, every extrusion a body of its own.
//!
//! `every_library_part_is_one_body` holds that. `translate_the_library_parts` is the one-off translation that was run
//! to bring them up to it, kept beside it so it can be read and run again: the first extrusion of a part stays its
//! base, every later one is added to the body, and what worked on a body works on the body carried along. The
//! fillet of the charger named its edges by numbers of the old naming; they are taken again by where they lie - the
//! long edges of the connector on top, the way the thumbnail shows them rounded.
use qymcad_core::feature::FeatureKind;
use qymcad_core::model::{Id, Project};

/// Every part of the built-in library, walked from its folder.
fn parts() -> Vec<String> {
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "qpart") {
                out.push(p.to_string_lossy().into_owned());
            }
        }
    }
    let mut out = Vec::new();
    walk(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../library/parts")), &mut out);
    assert!(!out.is_empty(), "the built-in library holds no part");
    out
}

/// The bodies of the part that are not used up by a later step.
fn live(p: &Project) -> Vec<Id> {
    let gone = p.consumed_bodies();
    p.timeline.iter().flat_map(|n| n.kind.bodies()).filter(|b| !gone.contains(b)).collect()
}

#[test]
fn every_library_part_is_one_body() {
    for path in parts() {
        let path = path.as_str();
        let mut p = qymcad_io::load_part(path).unwrap_or_else(|e| panic!("{path} does not load: {e}")).project;
        let (rep, _) = qymcad_testkit::regenerate(&mut p);
        assert!(rep.errors.is_empty(), "{path} rebuilds with errors: {:?}", rep.errors);
        assert_eq!(live(&p).len(), 1, "{path} is not one body: {:?}", live(&p));
    }
}

#[test]
#[ignore = "a one-off translation of the files in library/parts; run by hand"]
fn translate_the_library_parts() {
    for path in parts() {
        let path = path.as_str();
        let loaded = qymcad_io::load_part(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut p = loaded.project;
        // every later extrusion is added to the body; every step on a body takes the body carried along
        let mut cur: Option<Id> = None;
        for n in p.timeline.iter_mut() {
            match n.kind.clone() {
                FeatureKind::Extrude { sketch, profiles, height, down, fill, body, .. } => {
                    if let Some(src) = cur {
                        n.kind = FeatureKind::Combine { src, sketch, profiles, height, op: 1, extent: Default::default(), down, fill, body, pieces: Vec::new() };
                    }
                    cur = Some(body);
                }
                ref k => {
                    if let (Some(src), Some(out)) = (k.consumed_body(), k.body()) {
                        if let Some(c) = cur.filter(|c| *c != src) {
                            n.kind.remap_body_input(src, c);
                        }
                        cur = Some(out);
                    }
                }
            }
            n.dirty = true;
        }
        let (rep, _) = qymcad_testkit::regenerate(&mut p);
        // the fillet named its edges in the old naming: they are taken again by where they lie
        let fillets: Vec<(Id, Id)> = p
            .timeline
            .iter()
            .filter_map(|n| match n.kind {
                FeatureKind::Fillet { src, body, .. } => Some((src, body)),
                _ => None,
            })
            .collect();
        for (src, body) in fillets {
            let edges = &p.regen_edges[&src];
            let top = edges.iter().map(|e| e.a[2].max(e.b[2])).fold(f64::MIN, f64::max);
            let len = |e: &qymcad_core::geom::MeshEdge| ((e.a[0] - e.b[0]).powi(2) + (e.a[1] - e.b[1]).powi(2) + (e.a[2] - e.b[2]).powi(2)).sqrt();
            let longest = edges.iter().filter(|e| (e.a[2] - top).abs() < 1e-6 && (e.b[2] - top).abs() < 1e-6).map(len).fold(0.0, f64::max);
            let picked: Vec<u32> = edges.iter().filter(|e| (e.a[2] - top).abs() < 1e-6 && (e.b[2] - top).abs() < 1e-6 && (len(e) - longest).abs() < 1e-6).map(|e| e.id).collect();
            assert_eq!(picked.len(), 2, "{path}: the long top edges of the connector: {picked:?}");
            if let Some(n) = p.timeline.iter_mut().find(|n| n.kind.body() == Some(body)) {
                if let FeatureKind::Fillet { edges, .. } = &mut n.kind {
                    *edges = qymcad_core::refs::Ref::picks(&picked);
                }
                n.dirty = true;
            }
        }
        let _ = rep;
        let (rep, _) = qymcad_testkit::regenerate(&mut p);
        assert!(rep.errors.is_empty(), "{path}: {:?}", rep.errors);
        assert_eq!(live(&p).len(), 1, "{path}: {:?}", live(&p));
        qymcad_io::save_part(&p, &loaded.manifest, &[], loaded.thumb_png.as_deref(), path).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}
