//! A STEP ASSEMBLY SAVED AND OPENED AGAIN KEEPS ITS TREE, and every body comes back from the embedded file in the
//! coordinates of its own part, where the tree places it - the way the program opens a document.
//!
//! The reference is the kernel's `tests/data/assembly.step`: an assembly holding a plate twice (at x 0 and at x 30)
//! and a subassembly at (5, 10, 0) holding a pin at z 5.
use qymcad_core::model::Project;
use qymcad_kernel::{ExactTree, document_tree, read_exact_tree, ExactFormat};

const STEP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qymcad-kernel/tests/data/assembly.step");
const IGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qymcad-kernel/tests/data/assembly.igs");

/// Every component as (name, its parent's name, its placement), in the document's order.
fn tree(p: &Project) -> Vec<(String, String, [f64; 12])> {
    let name = |id: Option<u64>| id.and_then(|id| p.components.iter().find(|c| c.id == id)).map(|c| c.name.clone()).unwrap_or_default();
    p.components.iter().map(|c| (c.name.clone(), name(c.parent), c.transform)).collect()
}

/// Where each part stands in the world, sorted: the second plate is a clone and brings no body of its own from the
/// file, so parts are counted, not the bodies read.
fn places(p: &Project) -> Vec<[f64; 3]> {
    let mut out: Vec<[f64; 3]> = p
        .components
        .iter()
        .filter(|c| c.kind == qymcad_core::feature::ComponentKind::Part && !p.component_bodies(c.id).is_empty())
        .map(|c| p.world_transform(c.id))
        .map(|w| [w[3], w[7], w[11]])
        .collect();
    out.sort_by(|a, b| a.partial_cmp(b).expect("numbers"));
    out
}

#[test]
fn a_step_assembly_reopens_as_its_tree() {
    reopens(ExactFormat::Step, STEP, "assembly.step");
}

/// THE SAME FOR IGES: its bodies come back from the embedded file by the same walk of its subfigures, in the same
/// order, so every part finds its own.
#[test]
fn an_iges_assembly_reopens_as_its_tree() {
    reopens(ExactFormat::Iges, IGES, "assembly.igs");
}

fn reopens(format: ExactFormat, file: &str, stem: &str) {
    let ExactTree { bodies, nodes, .. } = read_exact_tree(format, file, 0.5).expect("the reference reads");
    let mut p = Project::default();
    p.new_empty_document();
    let source = p.add_source(stem, std::fs::read(file).expect("the file reads"));
    let ids: Vec<u64> = bodies.into_iter().map(|qymcad_core::geom::Built { mesh: m, .. }| p.add_mesh(m)).collect();
    p.import_tree_as_parts(document_tree(&nodes, &ids, "assembly"), source, "assembly").expect("it came in");
    let boxes: Vec<[f64; 6]> = ids.iter().map(|&b| p.bodies[p.mesh_index(b).expect("a body")].mesh.bounds().expect("a mesh")).map(|b| [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z]).collect();

    let dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/step-reopen"));
    std::fs::create_dir_all(&dir).expect("a folder for the check");
    let path = dir.join(format!("{stem}.qcad")).to_string_lossy().into_owned();
    qymcad_io::save_project(&p, &path).expect("saved");
    let mut q = qymcad_io::load_project(&path).expect("opened");
    assert_eq!(tree(&q), tree(&p), "the tree did not survive saving");
    assert_eq!(places(&q), [[0.0, 0.0, 0.0], [5.0, 10.0, 5.0], [30.0, 0.0, 0.0]], "the parts do not stand where the file puts them");

    // every body comes back from the embedded file in its own part's coordinates: a body baked into the world would
    // be placed twice. The kernel's box of a curved body is looser than its mesh by some 0.011 mm.
    let shapes = qymcad_testkit::restore_import_shapes(&q);
    for (k, &b) in ids.iter().enumerate() {
        let got = shapes.get(&b).unwrap_or_else(|| panic!("body {k} came back with no live body")).bbox().expect("a box");
        assert!(got.iter().zip(boxes[k]).all(|(g, w)| (g - w).abs() < 0.05), "body {k} came back at {got:?}, not at {:?} as it was read", boxes[k]);
    }
    let report = qymcad_testkit::open_like_the_app(&mut q);
    assert!(report.errors.is_empty(), "the reopened document does not rebuild: {:?}", report.errors);
    assert_eq!(places(&q), [[0.0, 0.0, 0.0], [5.0, 10.0, 5.0], [30.0, 0.0, 0.0]], "rebuilding moved the parts");
}
