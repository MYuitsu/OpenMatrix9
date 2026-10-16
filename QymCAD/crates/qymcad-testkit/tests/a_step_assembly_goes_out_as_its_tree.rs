//! A STEP ASSEMBLY GOES OUT AS ITS TREE and reads back the same: the parts as products under their names and
//! colours, every component in its place, and the plate - one product placed twice in the file - once again.
//!
//! The export used to write every body baked into the world, as "Open CASCADE STEP translator 7.9 N" - no tree, no
//! names, no colours, a clone written out twice.
use qymcad_core::model::Project;
use qymcad_kernel::{ExactTree, document_tree, read_exact_tree, write_step_tree, ExactFormat, ImportNode};

const STEP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qymcad-kernel/tests/data/assembly.step");

/// The names the reference was written with: the assembly, the plate, the subassembly, the pin.
const NAMES: &str = include_str!("../../qymcad-kernel/tests/data/assembly.names");

fn name(k: usize) -> &'static str {
    NAMES.lines().nth(k).expect("a name of the reference")
}

/// Where node `i` stands in the world: its placement composed with every parent's.
fn world(nodes: &[ImportNode], i: usize) -> [f64; 3] {
    let mut m = nodes[i].place;
    let mut at = nodes[i].parent;
    while let Some(p) = at {
        let a = nodes[p].place;
        let mut out = [0.0; 12];
        for r in 0..3 {
            for c in 0..4 {
                out[r * 4 + c] = (0..3).map(|k| a[r * 4 + k] * m[k * 4 + c]).sum::<f64>() + if c == 3 { a[r * 4 + 3] } else { 0.0 };
            }
        }
        m = out;
        at = nodes[p].parent;
    }
    [m[3], m[7], m[11]]
}

#[test]
fn a_step_assembly_goes_out_as_its_tree() {
    let ExactTree { bodies, shapes, nodes } = read_exact_tree(ExactFormat::Step, STEP, 0.5).expect("the reference reads");
    let mut p = Project::default();
    p.new_empty_document();
    let source = p.add_source("assembly.step", std::fs::read(STEP).expect("the file reads"));
    let ids: Vec<u64> = bodies.into_iter().map(|qymcad_core::geom::Built { mesh: m, .. }| p.add_mesh(m)).collect();
    let root = p.import_tree_as_parts(document_tree(&nodes, &ids, "assembly"), source, "assembly").expect("it came in");
    // the live bodies: the ones read, and the clone's, built by a rebuild from its original's
    let (_, live) = qymcad_testkit::regenerate_with_shapes(&mut p, ids.iter().copied().zip(shapes).collect());
    let tree = p.export_tree(root, |b| live.contains_key(&b));
    let dir = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/step-export"));
    std::fs::create_dir_all(&dir).expect("a folder for the check");
    let out = dir.join("assembly-out.step").to_string_lossy().into_owned();
    let pairs: Vec<(u64, &qymcad_kernel::Shape)> = live.iter().map(|(k, s)| (*k, s)).collect();
    write_step_tree(&tree, &pairs, &out).expect("the tree is written");

    let ExactTree { bodies: back_bodies, nodes: back, .. } = read_exact_tree(ExactFormat::Step, &out, 0.5).expect("the file written reads back");
    let named = |n: &str| (0..back.len()).filter(|&i| back[i].name == n).collect::<Vec<_>>();
    let roots: Vec<usize> = (0..back.len()).filter(|&i| back[i].parent.is_none()).collect();
    assert_eq!(roots.iter().map(|&i| back[i].name.as_str()).collect::<Vec<_>>(), [name(0)], "the root does not go out under its name");
    let under = |p: usize| back.iter().filter(|n| n.parent == Some(p)).map(|n| n.name.as_str()).collect::<Vec<_>>();
    assert_eq!(under(roots[0]), [name(1), name(1), name(2)], "the tree does not go out as it stands");
    assert_eq!(under(named(name(2))[0]), [name(3)]);
    assert_eq!(back_bodies.len(), 2, "the plate goes out twice, not as one product placed twice");
    let plates = named(name(1));
    assert!(plates.iter().any(|&i| world(&back, i)[0].abs() < 1e-6) && plates.iter().any(|&i| (world(&back, i)[0] - 30.0).abs() < 1e-6), "the plates do not stand at x 0 and 30");
    let pin = named(name(3))[0];
    assert!(world(&back, pin).iter().zip([5.0, 10.0, 5.0]).all(|(g, w)| (g - w).abs() < 1e-6), "the pin does not stand at (5, 10, 5): {:?}", world(&back, pin));
    let near = |c: Option<[f32; 3]>, want: [f32; 3]| c.is_some_and(|c| c.iter().zip(want).all(|(g, w)| (g - w).abs() < 0.01));
    assert!(near(back[plates[0]].color, [0.8, 0.1, 0.1]), "the plate goes out coloured {:?}, not red", back[plates[0]].color);
    assert!(near(back[pin].color, [0.1, 0.2, 0.9]), "the pin goes out coloured {:?}, not blue", back[pin].color);
    // A FACE OF A COLOUR OF ITS OWN GOES OUT WITH IT: the plate's top face, green where the plate is red
    let faces = &back[plates[0]].faces;
    assert_eq!(faces.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [6], "the plate's top face does not go out in a colour of its own: {faces:?}");
    assert!(near(Some(faces[0].1), [0.1, 0.8, 0.1]), "the top face goes out {:?}, not green", faces[0].1);
    // THE NAMES SPELLED AS THE FORMAT HOLDS THEM: a STEP string is ISO 8859-1, and what lies past it is written as
    // \X2\ ... \X0\ in UTF-16 - raw UTF-8 comes up as mojibake in a reader that keeps to the format
    let text = std::fs::read(&out).expect("the file written");
    assert!(text.iter().all(|b| b.is_ascii()), "the names go out as raw UTF-8, not spelled as \\X2\\");
}
