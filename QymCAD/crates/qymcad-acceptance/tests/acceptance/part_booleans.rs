//! A BOOLEAN BETWEEN TWO BODIES of one part: joined, cut and met.
//!
//! Two bodies in a part come of a split: a box of 20 by 20 by 20 cut across the middle leaves two pieces of 4000
//! each. What the tool then does to them is read as a number.
use qymcad::{Key, Session};
use qymcad_acceptance::build;
use qymcad_acceptance::probe;

/// The solid bodies of the part, in the order they were made.
fn bodies(s: &mut Session) -> Vec<qymcad::Solid> {
    s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).collect()
}

/// Take the tool whose hint is `hint`.
fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

/// Press the word `key` names in the bar of options at the top.
fn bar_word(s: &mut Session, key: &str) {
    let word = s.word(key);
    s.press_word_near(&word, qymcad::pos2(640.0, 24.0));
}

/// A part holding two pieces of 4000 each: a box of 20 by 20 by 20 split across the middle.
fn two_pieces() -> Session {
    let mut s = Session::start();
    build::into_the_first_part(&mut s);
    take(&mut s, "tb-box-hint");
    s.key(Key::Enter);
    take(&mut s, "tb-split-body-hint");
    let top = s.face_at([0.0, 0.0, 20.0]);
    s.click(top);
    let caption = s.word("f-offset");
    s.fill(&caption, "-10");
    s.key(Key::Enter);
    let pieces = bodies(&mut s);
    assert!(pieces.len() == 2, "the box was not split in two: the part holds {:?}", pieces.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
    s
}

/// Take the boolean `kind` between the piece whose face is at `a` and the piece whose face is at `b`: body B clicked,
/// shown as the tool, nothing made yet.
fn boolean_taken(s: &mut Session, a: [f64; 3], kind: &str, b: [f64; 3]) {
    let at = s.face_at(a);
    s.double_click(at); // body A: a double click takes the body, one click only its face
    take(s, "tb-bool-bodies-hint");
    bar_word(s, kind);
    let at = s.face_at(b);
    s.click(at); // body B
}

/// Do the boolean `kind` between the piece whose face is at `a` and the piece whose face is at `b`.
fn boolean(s: &mut Session, a: [f64; 3], kind: &str, b: [f64; 3]) {
    boolean_taken(s, a, kind, b);
    s.key(Key::Enter); // the boolean made, as every command is
}

probe! {
    /// UNION: the two pieces become one body again, holding what they held together.
    fn a_union_makes_the_two_pieces_one_body() {
        let mut s = two_pieces();
        boolean(&mut s, [0.0, 0.0, 20.0], "f-union", [0.0, -10.0, 5.0]);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "the union leaves one body, and the part holds {:?}", now.iter().map(|b| (b.name.clone(), b.volume)).collect::<Vec<_>>());
        assert!((now[0].volume - 8000.0).abs() < 1.0, "the two pieces of 4000 make 8000, and the body holds {}", now[0].volume);
    }
}

probe! {
    /// A CUT OF PIECES THAT ONLY TOUCH REMOVES NOTHING, and the program says so at the click on body B instead of
    /// laying a node that goes red. Reported behaviour: the cut laid a red node "the cut removed nothing".
    fn a_cut_of_pieces_that_only_touch_says_so() {
        let mut s = two_pieces();
        let before: f64 = bodies(&mut s).iter().map(|b| b.volume).sum();
        let nodes = s.document().features.len();
        boolean_taken(&mut s, [0.0, 0.0, 20.0], "f-cut-ab", [0.0, -10.0, 5.0]);
        let said = s.status();
        assert!(said == s.word("vp-bool-cut-removes-nothing"), "nothing says the cut would remove nothing: the program says {said:?}");
        s.key(Key::Enter);
        let now: f64 = bodies(&mut s).iter().map(|b| b.volume).sum();
        assert!((now - before).abs() < 1.0, "the part lost its material to a cut that removes nothing: {before} became {now}");
        assert!(s.document().features.len() == nodes, "a cut that removes nothing laid a node: {} nodes became {}", nodes, s.document().features.len());
    }
}

probe! {
    /// INTERSECTION OF TWO PIECES THAT ONLY TOUCH MAKES NOTHING, and the program says so instead of leaving an empty
    /// part behind.
    fn an_intersection_of_pieces_that_only_touch_says_so() {
        let mut s = two_pieces();
        let before: f64 = bodies(&mut s).iter().map(|b| b.volume).sum();
        // the refusal comes at the click on body B, before anything is laid
        boolean_taken(&mut s, [0.0, 0.0, 20.0], "f-intersect", [0.0, -10.0, 5.0]);
        let said = s.status();
        let now: f64 = bodies(&mut s).iter().map(|b| b.volume).sum();
        assert!(said == s.word("vp-bool-nothing-in-common"), "nothing says the two pieces meet in nothing: the program says {said:?}");
        assert!((now - before).abs() < 1.0, "the part lost its material to an intersection that makes nothing: {before} became {now}");
    }
}

probe! {
    /// A PART IS ONE BODY, AND THE PIECES OF A SPLIT ARE THE NAMED EXCEPTION: the split leaves its two pieces as two
    /// bodies of the one part, as the professional systems leave a body split into several, and a union of the pieces
    /// brings the part back to one body.
    fn a_split_leaves_its_pieces_as_bodies_of_one_part() {
        let mut s = two_pieces();
        let pieces = bodies(&mut s);
        let parts: Vec<Option<u64>> = pieces.iter().map(|b| b.part_key).collect();
        assert!(
            pieces.len() == 2 && pieces.iter().all(|b| (b.volume - 4000.0).abs() < 1.0) && parts[0].is_some() && parts[0] == parts[1],
            "the split leaves two pieces of 4000 in the one part, and the document holds {:?}",
            pieces.iter().map(|b| (b.name.clone(), b.volume, b.part_key)).collect::<Vec<_>>()
        );
        boolean(&mut s, [0.0, 0.0, 20.0], "f-union", [0.0, -10.0, 5.0]);
        let now = bodies(&mut s);
        assert!(now.len() == 1, "a union of the pieces brings the part back to one body, and it holds {}", now.len());
    }
}
