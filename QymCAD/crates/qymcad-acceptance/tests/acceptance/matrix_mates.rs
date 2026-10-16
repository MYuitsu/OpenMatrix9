//! THE MATES ACROSS KINDS AND PLACES: the second of two blocks of 40 by 30 by 10 put at different places before the
//! mate - along X, along Y, off to a corner and up, to the left - and every one of the eight kinds laid between the
//! top faces. Each must be made of the kind asked for and hold; a rigid mate brings the second block onto the first
//! wherever it stood.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

/// The places the second part is put at before the mate.
const PLACES: [[f64; 3]; 4] = [[60.0, 0.0, 0.0], [0.0, 50.0, 0.0], [60.0, 40.0, 20.0], [-60.0, 0.0, 0.0]];

/// The eight kinds of mate.
const KINDS: [&str; 8] = [
    "joint-kind-rigid",
    "joint-kind-revolute",
    "joint-kind-slider",
    "joint-kind-cylindrical",
    "joint-kind-pin-slot",
    "joint-kind-planar",
    "joint-kind-ball",
    "joint-kind-parallel",
];

/// Two parts, each holding a block; the second put at `at` by the numbers of its placement.
fn two_blocks(at: [f64; 3]) -> Session {
    let mut s = Session::start();
    s.key(Key::Escape);
    build::block(&mut s);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    let new_part = s.word("tb-new-part-hint");
    s.press_hint(&new_part);
    let second = s.document().parts.last().cloned().expect("the second part").name;
    build::rectangle_on_xy(&mut s);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude);
    s.key(Key::Enter);
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
    s.click(row.center());
    for (caption, v) in ["X", "Y", "Z"].into_iter().zip(at) {
        s.fill(caption, &format!("{v}"));
    }
    s.key(Key::Enter);
    let (view, fit) = (s.word("menu-view"), s.word("menu-fit-view"));
    s.menu(&[&view, &fit]);
    s
}

/// ONE CASE: the second block put at `at`, a mate of `kind` between the two tops. What went wrong, if anything.
fn case(kind: &str, at: [f64; 3]) -> Option<String> {
    let problem = qymcad_acceptance::refusal(|| {
        let mut s = two_blocks(at);
        let start = s.word("jp-start-joint");
        s.press_word_near(&start, qymcad::pos2(1100.0, 400.0));
        let caption = s.word("j-kind");
        let list = s.field(&caption);
        s.click(list.rect.center());
        let want = s.word(kind);
        s.press_word_near(&want, list.rect.center());
        let a = s.face_at([20.0, 15.0, 10.0]);
        s.click(a);
        let b = s.face_at([at[0] + 20.0, at[1] + 15.0, at[2] + 10.0]);
        s.click(b);
        let joint = s.document().joints.last().cloned().unwrap_or_else(|| panic!("no mate was made: the program says {:?}", s.status()));
        let squeeze = |w: &str| w.to_lowercase().replace(['-', ' '], "");
        assert!(squeeze(&joint.kind) == squeeze(&want), "a {want} mate was asked for, and the document holds a {:?} one", joint.kind);
        assert!(!joint.violated, "the mate does not hold: {joint:?}");
        if kind == "joint-kind-rigid" {
            let now = s.document().parts.last().map(|p| p.at).expect("the second part");
            assert!(now.iter().all(|v| v.abs() < 1e-3), "the rigid mate did not bring the second block onto the first: it stands at {now:?}");
        }
    });
    (!problem.is_empty()).then(|| format!("{kind} from {at:?}: {problem}"))
}

/// Every kind from every place in `places`.
fn run(places: &[[f64; 3]]) {
    let failed: Vec<String> = places.iter().flat_map(|at| KINDS.iter().filter_map(move |k| case(k, *at))).collect();
    assert!(failed.is_empty(), "{} of {} mates went wrong:\n{}", failed.len(), places.len() * KINDS.len(), failed.join("\n"));
}

probe! {
    budget = 1800;
    /// EVERY KIND OF MATE, the second block along X and along Y.
    fn mates_from_along_the_axes() {
        run(&PLACES[..2]);
    }
}

probe! {
    budget = 1800;
    /// EVERY KIND OF MATE, the second block off to a corner and up, and to the left of the first.
    fn mates_from_a_corner_and_the_left() {
        run(&PLACES[2..]);
    }
}
