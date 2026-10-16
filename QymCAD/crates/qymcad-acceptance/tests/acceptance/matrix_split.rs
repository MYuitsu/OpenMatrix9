//! SPLIT BODY ACROSS BODIES: the plane of a face taken, its offset typed, Enter - the body falls into two pieces that
//! stay bodies of the one part (the named exception to "a part is one body"), each holding what the geometry gives;
//! a plane that does not cross the body refused in words. Then what is done with the pieces: undone and redone, saved
//! and opened, the base changed above them, joined, cut and met again.
use qymcad::{Key, Session};
use qymcad_acceptance::{bodies, build, oracles, probe};
use std::f64::consts::PI;

/// The solid bodies of the part, in the order they were made.
fn solids(s: &mut Session) -> Vec<qymcad::Solid> {
    s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).collect()
}

/// The volumes of the solids, smallest first.
fn volumes(s: &mut Session) -> Vec<f64> {
    let mut v: Vec<f64> = solids(s).iter().map(|b| b.volume).collect();
    v.sort_by(f64::total_cmp);
    v
}

/// Split the body by the plane of the face through `face`, offset by `offset`.
fn split(s: &mut Session, face: [f64; 3], offset: &str) {
    let hint = s.word("tb-split-body-hint");
    s.press_hint(&hint);
    let at = s.face_at(face);
    s.click(at);
    let caption = s.word("f-offset");
    s.fill(&caption, offset);
    s.key(Key::Enter);
}

/// Are the volumes `got` those of `want`, each within a thousandth?
fn same(got: &[f64], want: &[f64]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(g, w)| (g - w).abs() <= w.abs() * 1e-3 + 0.5)
}

/// ONE CASE: the body `make` builds, split through `face` at `offset`, gives the pieces `want` (smallest first); an
/// empty `want` is a refusal - in words, the body whole.
fn case(what: &str, make: fn(&mut Session), face: [f64; 3], offset: &str, want: &[f64]) -> Option<String> {
    let mut s = Session::start();
    make(&mut s);
    let whole = volumes(&mut s);
    let status = s.status();
    split(&mut s, face, offset);
    let got = volumes(&mut s);
    if want.is_empty() {
        let red = s.document().features.iter().rfind(|f| f.kind == "SplitBody").is_some_and(|f| f.error.is_some());
        let said = red || s.status() != status;
        let mut problems = Vec::new();
        if !said {
            problems.push(format!("refused without a word: the status line says {:?}", s.status()));
        }
        if !same(&got, &whole) {
            problems.push(format!("the body changed: {got:?}, it was {whole:?}"));
        }
        return (!problems.is_empty()).then(|| format!("{what}: {}", problems.join("; ")));
    }
    if !same(&got, want) {
        return Some(format!("{what}: the pieces hold {got:?}, they should hold {want:?}; the status line says {:?}", s.status()));
    }
    None
}

probe! {
    budget = 900;
    /// THE PIECES ACROSS BODIES AND PLANES: a block and a cylinder cut across the middle and near the top; a plane
    /// below the body, in the plane of its top and above it, refused.
    fn split_body_across_bodies_and_planes() {
        let block = |s: &mut Session| build::block(s);
        let cylinder = PI * 100.0 * 10.0;
        let problems: Vec<String> = [
            case("the block across its middle", block, [20.0, 15.0, 10.0], "-5", &[6000.0, 6000.0]),
            case("the block near its top", block, [20.0, 15.0, 10.0], "-2.5", &[3000.0, 9000.0]),
            case("the block through its front", block, [20.0, 0.0, 5.0], "-10", &[4000.0, 8000.0]),
            case("the cylinder across its middle", bodies::cylinder, [20.0, 15.0, 10.0], "-5", &[cylinder / 2.0, cylinder / 2.0]),
            case("the block by a plane below it", block, [20.0, 15.0, 10.0], "-12", &[]),
            case("the block by the plane of its top", block, [20.0, 15.0, 10.0], "0", &[]),
            case("the block by a plane above it", block, [20.0, 15.0, 10.0], "3", &[]),
        ]
        .into_iter()
        .flatten()
        .collect();
        assert!(problems.is_empty(), "{} of 7 cases went wrong:\n{}", problems.len(), problems.join("\n"));
    }
}

/// The block split across its middle: two pieces of 6000.
fn halves() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    split(&mut s, [20.0, 15.0, 10.0], "-5");
    assert!(same(&volumes(&mut s), &[6000.0, 6000.0]), "setup: the block was not split in two: {:?}", volumes(&mut s));
    s
}

probe! {
    /// THE PIECES LIVE THROUGH UNDO, REDO, SAVING, OPENING AND REBUILDING: each gives back the same two pieces.
    fn split_pieces_survive_the_round_trips() {
        let mut s = halves();
        let path = std::env::temp_dir().join(format!("qymcad-split-{}.qcad", std::process::id())).to_string_lossy().into_owned();
        let problems: Vec<String> = [oracles::undo_redo(&mut s), oracles::save_open(&mut s, &path), oracles::rebuild_everything(&mut s)].into_iter().filter_map(Result::err).collect();
        assert!(problems.is_empty(), "the pieces did not come back the same:\n{}", problems.join("\n"));
    }
}

probe! {
    /// THE PIECES FOLLOW THE BASE: the extrusion made 20 tall above the split, the plane 5 below the top moves with
    /// the top - pieces of 6000 above and 18000 below.
    fn split_pieces_follow_a_change_above() {
        let mut s = halves();
        let lead = s.word("cmd-extrude");
        let left = s.canvas().min.x;
        let rows: Vec<qymcad::Rect> = s.words_at().into_iter().filter(|(w, r)| r.max.x < left && w.starts_with(&lead)).map(|(_, r)| r).collect();
        let [row] = rows.as_slice() else { panic!("the row of the extrusion is not one row of the tree: {rows:?}") };
        s.double_click(row.center());
        let caption = s.word("f-length");
        s.fill(&caption, "20").key(Key::Enter);
        let got = volumes(&mut s);
        assert!(same(&got, &[6000.0, 18000.0]), "the extrusion made 20 tall above the split: the pieces hold {got:?}, they should hold [6000, 18000]; the status line says {:?}", s.status());
    }
}

/// Do the boolean `kind` between the piece whose face is at `a` and the piece whose face is at `b`.
fn boolean(s: &mut Session, a: [f64; 3], kind: &str, b: [f64; 3]) {
    let at = s.face_at(a);
    s.double_click(at); // body A: a double click takes the body, one click only its face
    let hint = s.word("tb-bool-bodies-hint");
    s.press_hint(&hint);
    let word = s.word(kind);
    s.press_word_near(&word, qymcad::pos2(0.0, 0.0));
    let at = s.face_at(b);
    s.click(at); // body B taken
    s.key(qymcad::Key::Enter); // the boolean made
}

probe! {
    /// THE PIECES OF A CYLINDER JOINED AND CUT AGAIN: a union makes the whole cylinder back; a cut of the lower from
    /// the upper would remove nothing, the pieces only touching, so it is refused and both halves stay.
    fn booleans_between_the_pieces_of_a_cylinder() {
        let half = PI * 100.0 * 5.0;
        let problems: Vec<String> = [("f-union", vec![2.0 * half]), ("f-cut-ab", vec![half, half])]
            .into_iter()
            .filter_map(|(kind, want)| {
                let mut s = Session::start();
                bodies::cylinder(&mut s);
                split(&mut s, [20.0, 15.0, 10.0], "-5");
                boolean(&mut s, [20.0, 15.0, 10.0], kind, [20.0, 5.0, 2.0]);
                let got = volumes(&mut s);
                (!same(&got, &want)).then(|| format!("{kind}: the part holds {got:?}, it should hold {want:?}; the status line says {:?}", s.status()))
            })
            .collect();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
