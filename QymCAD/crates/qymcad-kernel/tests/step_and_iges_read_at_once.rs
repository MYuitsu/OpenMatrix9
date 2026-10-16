//! STEP AND IGES READ AT ONCE, from threads side by side, give what they give one after the other - and the process
//! lives.
//!
//! Measured on the check that reopens a STEP and an IGES assembly side by side: 1 run of 25 died with SIGSEGV, and once
//! in a whole run it grew to 24 GB and took the machine into swap; one after the other it passed 25 of 25. Both crashes
//! stood in the same place: the first IGES reader made (`IGESControl_Controller::Init`, dying in `MoniTool_TypedValue`)
//! while another thread made the first STEP one (`STEPControl_Controller::Init`). Each kind sets OCCT's data exchange up
//! the first time it is made, in process-wide state, and each was held by a lock of its own. An import runs on a worker
//! thread, so a document holding both can bring them together.
use qymcad_kernel::{ExactTree, read_exact_tree, ExactFormat};

const STEP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/assembly.step");
const IGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/assembly.igs");

/// What a reading gives, told by its counts: the bodies, the nodes of the tree, the faces of a colour of their own.
fn read(format: ExactFormat, path: &str) -> (usize, usize, usize) {
    let ExactTree { bodies, nodes, .. } = read_exact_tree(format, path, 0.5).expect("the reference reads");
    (bodies.len(), nodes.len(), nodes.iter().map(|n| n.faces.len()).sum())
}

#[test]
fn step_and_iges_read_at_once_as_one_after_the_other() {
    // THE FIRST READINGS AT ONCE: that is where they met, so nothing is read before the threads start, and they start
    // together
    let start = std::sync::Arc::new(std::sync::Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|k| {
            let start = start.clone();
            std::thread::spawn(move || {
                let (format, path) = if k % 2 == 0 { (ExactFormat::Step, STEP) } else { (ExactFormat::Iges, IGES) };
                start.wait();
                (0..20).map(|_| read(format, path)).collect::<Vec<_>>()
            })
        })
        .collect();
    let got: Vec<Vec<(usize, usize, usize)>> = threads.into_iter().map(|t| t.join().expect("a reading thread")).collect();
    let alone = [read(ExactFormat::Step, STEP), read(ExactFormat::Iges, IGES)];
    for (k, g) in got.iter().enumerate() {
        let want = alone[k % 2];
        assert!(g.iter().all(|x| *x == want), "a reading beside the others gave {g:?}; alone it gives {want:?}");
    }
}
