//! EXTERNAL REFERENCES ACROSS CHANGES OF THE NEIGHBOUR: a second part built in context on the top face of the first -
//! a rectangle on it extruded 10 - then the first part changed: its block made taller, the part moved; the second
//! part must follow each, as a live reference does. The same, the reference broken first: the second part stays.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

/// The block in the first part and, built in context, a second part: a rectangle from (10, 10) to (30, 20) sketched
/// on the top face of the block and extruded 10. The session left in the assembly.
fn built_on_the_neighbour() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    build::into_a_new_part(&mut s);
    let context = s.word("wb-in-context");
    let switch = s.widgets().into_iter().find(|w| w.label.ends_with(&context)).unwrap_or_else(|| panic!("there is no switch for working in context"));
    if switch.checked == Some(false) {
        s.click(switch.rect.center());
    }
    build::sketch_on_face(&mut s, [20.0, 15.0, 10.0]);
    build::draw(&mut s, "tb-rect-hint", &[(10.0, 10.0), (30.0, 20.0)]);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude).key(Key::Enter);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    s
}

/// The lowest corner of the second part's body.
fn second_body_min(s: &mut Session) -> [f64; 3] {
    let doc = s.document();
    let key = doc.parts.get(1).map(|p| p.key).expect("the second part");
    doc.bodies.iter().find(|b| b.part_key == Some(key) && !b.consumed && !b.sheet).map(|b| b.min).unwrap_or_else(|| panic!("the second part holds no body"))
}

/// The first part's block made `h` tall: into the part by its row, the extrusion reopened by its row, the length typed.
fn first_made_taller(s: &mut Session, h: &str) {
    let first = s.document().parts[0].name.clone();
    let row = s.find(&first, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {first:?} is not in the tree"));
    s.double_click(row.center());
    let lead = s.word("cmd-extrude");
    let left = s.canvas().min.x;
    let rows: Vec<qymcad::Rect> = s.words_at().into_iter().filter(|(w, r)| r.max.x < left && w.starts_with(&lead)).map(|(_, r)| r).collect();
    let [row] = rows.as_slice() else { panic!("the row of the extrusion is not one row of the tree: {rows:?}") };
    s.double_click(row.center());
    let caption = s.word("f-length");
    s.fill(&caption, h).key(Key::Enter);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
}

/// The first part moved to `x` along X by the numbers of its placement.
fn first_moved(s: &mut Session, x: &str) {
    let first = s.document().parts[0].name.clone();
    let row = s.find(&first, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {first:?} is not in the tree"));
    s.click(row.center());
    s.fill("X", x).key(Key::Enter);
}

/// The reference of the second part broken by the cross beside it in the part's properties.
fn broken(s: &mut Session) {
    let second = s.document().parts[1].name.clone();
    let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
    s.click(row.center());
    let cross = s.word("comp-break-ref-hint");
    s.press_hint(&cross);
}

probe! {
    budget = 1800;
    /// THE SECOND PART FOLLOWS ITS NEIGHBOUR while the reference lives - the block made 15 tall lifts it to 15, the
    /// first part moved 25 along X carries it 25 - and stays where it is once the reference is broken.
    fn a_reference_follows_its_neighbour_until_broken() {
        let mut failed = Vec::new();
        let mut s = built_on_the_neighbour();
        let was = second_body_min(&mut s);
        if (was[2] - 10.0).abs() > 1e-3 {
            failed.push(format!("setup: the second part stands on the top at 10, its body starts at {was:?}"));
        }
        first_made_taller(&mut s, "15");
        let now = second_body_min(&mut s);
        if (now[2] - 15.0).abs() > 1e-3 {
            failed.push(format!("the block made 15 tall: the second part should start at 15 up, it starts at {now:?}"));
        }
        let mut s = built_on_the_neighbour();
        first_moved(&mut s, "25");
        let now = second_body_min(&mut s);
        if (now[0] - (was[0] + 25.0)).abs() > 1e-3 {
            failed.push(format!("the first part moved 25 along X: the second part should start at {} along X, it starts at {now:?}", was[0] + 25.0));
        }
        let mut s = built_on_the_neighbour();
        broken(&mut s);
        first_made_taller(&mut s, "15");
        let now = second_body_min(&mut s);
        if (now[2] - 10.0).abs() > 1e-3 {
            failed.push(format!("the reference broken, the block made 15 tall: the second part should stay at 10 up, it starts at {now:?}; the program says {:?}", s.status()));
        }
        assert!(failed.is_empty(), "{} of 3 changes went wrong:\n{}", failed.len(), failed.join("\n"));
    }
}
