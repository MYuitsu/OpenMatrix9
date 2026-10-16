//! COPIES AND TIES ACROSS AN ASSEMBLY: a part laid out in a pattern, a mirrored copy of one, the check that finds
//! bodies standing inside one another, and the reference a part keeps to its neighbour.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, golden, probe};

/// An assembly of two parts, each holding a 40 x 30 x 10 block: the first at the origin, the second 60 along X. The
/// session stands in the assembly with the second part taken in the tree.
fn two_parts() -> Session {
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
    s.fill("X", "60").key(Key::Enter);
    s
}

/// The parts of the assembly.
fn parts(s: &mut Session) -> Vec<qymcad::Part> {
    s.document().parts
}

/// Take the tool whose hint is `hint`.
fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

probe! {
    /// A LINEAR PATTERN OF COMPONENTS lays the part out in a row, and every copy is a part of the assembly.
    fn a_linear_pattern_lays_the_part_out_in_a_row() {
        let mut s = two_parts();
        let before = parts(&mut s).len();
        take(&mut s, "tb-comp-lin-array-hint");
        let at = s.face_at([80.0, 15.0, 10.0]);
        s.click(at);
        let count = s.word("cmd-copies");
        s.fill(&count, "3");
        s.key(Key::Enter).key(Key::Enter);
        let now = parts(&mut s);
        assert!(now.len() == before + 2, "three in a row means two copies beside the part, and the assembly holds {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
        let places: Vec<f64> = now.iter().map(|p| p.at[0]).collect();
        assert!(places.iter().filter(|x| **x > 60.5).count() == 2, "the copies do not stand beside the part they came from: the parts stand at {places:?}");
    }
}

probe! {
    /// A MIRRORED COPY OF A PART stands on the other side of the plane it was mirrored in.
    fn a_mirrored_copy_stands_on_the_other_side() {
        let mut s = two_parts();
        let before = parts(&mut s).len();
        take(&mut s, "tb-mirror-part-hint");
        let at = s.face_at([80.0, 15.0, 10.0]);
        s.click(at); // the part to mirror
        let plane = s.face_at([40.0, 15.0, 5.0]); // the right side of the first block, at x = 40
        s.click(plane);
        s.key(Key::Enter);
        let now = parts(&mut s);
        assert!(now.len() == before + 1, "the mirrored copy is not in the assembly: it holds {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
        let made = now.last().cloned().expect("the mirrored copy");
        assert!(made.at[0] < 40.0, "the copy mirrored in the face at 40 stands on the same side as the original: at {:?}", made.at);
    }
}

probe! {
    /// THE CHECK OF INTERFERENCE MARKS BODIES STANDING INSIDE ONE ANOTHER: with it on, the picture of two blocks in
    /// the same place is not the picture of the same two with the check off.
    fn the_check_of_interference_marks_bodies_inside_one_another() {
        let mut s = two_parts();
        // the second part is put over the first, so the two blocks share the same place
        let second = parts(&mut s)[1].name.clone();
        let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
        s.click(row.center());
        s.fill("X", "20").key(Key::Enter);
        let canvas = s.canvas();
        s.click(canvas.min + (canvas.max - canvas.min) * 0.02); // nothing taken, so the picture shows no selection
        let quiet = s.snapshot();
        let check = s.word("tree-interference");
        s.toggle(&check);
        assert!(!golden::same(&quiet, &s.snapshot()), "with the check on, two blocks standing in one place look exactly as they did with it off");
    }
}

/// A SECOND PART WHOSE SKETCH SITS ON THE FACE OF THE FIRST - a live reference between the two. The session stands
/// in the assembly with the second part taken, its properties open.
fn a_part_referring_to_its_neighbour() -> (Session, String) {
    let mut s = Session::start();
    s.key(Key::Escape);
    build::block(&mut s);
    let first = s.document().parts[0].name.clone();
    build::into_a_new_part(&mut s);
    let context = s.word("wb-in-context");
    let switch = s.widgets().into_iter().find(|w| w.label.ends_with(&context)).unwrap_or_else(|| panic!("there is no switch for working in context"));
    if switch.checked == Some(false) {
        s.click(switch.rect.center());
    }
    build::sketch_on_face(&mut s, [20.0, 15.0, 10.0]);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    let second = s.document().parts[1].name.clone();
    let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
    s.click(row.center());
    (s, first)
}

/// Where the cross that breaks the reference to `neighbour` stands.
fn the_cross_beside_the_reference(s: &mut Session, neighbour: &str) -> qymcad::Pos2 {
    let head = s.word("comp-on-face-of").split('{').next().unwrap_or_default().trim().to_string();
    let line = s
        .words_at()
        .into_iter()
        .find(|(w, _)| w.starts_with(&head) && w.contains(neighbour))
        .map(|(_, r)| r)
        .unwrap_or_else(|| panic!("the line {head:?} naming the neighbour is not in the properties; on screen: {:?}", s.words()));
    s.widgets()
        .into_iter()
        .filter(|w| w.rect.left() >= line.right() - 2.0 && (w.rect.center().y - line.center().y).abs() < 14.0)
        .min_by(|a, b| a.rect.left().total_cmp(&b.rect.left()))
        .map(|w| w.rect.center())
        .unwrap_or_else(|| qymcad::pos2(line.right() + 12.0, line.center().y))
}

probe! {
    /// BREAKING A REFERENCE SAYS SO: the catalogue holds the words for it - how many sketches were frozen and that
    /// the part no longer follows its neighbour.
    fn breaking_a_reference_says_so() {
        let (mut s, first) = a_part_referring_to_its_neighbour();
        let cross = the_cross_beside_the_reference(&mut s, &first);
        s.click(cross);
        let head = s.word("comp-ref-broken").split(':').next().unwrap_or_default().to_string();
        assert!(s.status().starts_with(&head), "nothing says the reference was broken: the program says {:?}", s.status());
    }
}

probe! {
    /// A REFERENCE TO A NEIGHBOUR IS LISTED AND CAN BE BROKEN: a sketch on a neighbour's face is a live reference,
    /// the part's properties name it, and the cross beside it takes the part off its neighbour.
    fn a_reference_to_a_neighbour_is_listed_and_broken() {
        let (mut s, first) = a_part_referring_to_its_neighbour();
        let refs = s.word("comp-external-refs");
        assert!(s.shows(&refs), "the properties of the part do not name the reference to {first:?}; on screen: {:?}", s.words());
        assert!(s.words().iter().any(|w| w.contains(&first)), "the references do not name the neighbour {first:?} the sketch was put on; on screen: {:?}", s.words());
        let cross = the_cross_beside_the_reference(&mut s, &first);
        s.click(cross);
        assert!(!s.shows(&refs), "the reference is still listed after the cross beside it was pressed: the program says {:?}; on screen: {:?}", s.status(), s.words());
    }
}

probe! {
    /// A CIRCULAR PATTERN OF PARTS ABOUT AN EDGE CLICKED AS ITS AXIS: the upright edge at the far corner of the block
    /// is picked in the view, and the copy turned half a turn about it stands at (80, 60). Reported behaviour: the axis
    /// of a circular pattern of parts could only be X, Y or Z through the origin of the assembly.
    fn a_circular_pattern_of_parts_turns_about_an_edge_clicked() {
        let mut s = qymcad_acceptance::contract::fixtures::Fixture::BlockInAssemblyPicked.start();
        let tool = s.word("tb-comp-circ-array-hint");
        s.press_hint(&tool);
        let copies = s.word("cmd-copies");
        s.fill(&copies, "2");
        let pick = s.word("cmd-pick-axis");
        s.press_word(&pick);
        let edge = s.edge_at([40.0, 30.0, 5.0]);
        s.click(edge);
        s.key(qymcad::Key::Enter);
        let parts: Vec<qymcad::Part> = s.document().parts.into_iter().filter(|p| !p.assembly).collect();
        assert!(parts.len() == 2, "a pattern of two holds two parts, and the assembly holds {}; the status line says {:?}", parts.len(), s.status());
        let at = parts[1].at;
        assert!((at[0] - 80.0).abs() < 1e-3 && (at[1] - 60.0).abs() < 1e-3, "the copy turned half a turn about the edge at (40, 30) stands at {at:?}, not at (80, 60)");
    }
}

probe! {
    /// THE MIRRORED COPY TAKEN WITH NOTHING PICKED WAITS FOR A PART: the bar says what to click, a click on the block
    /// takes its part, a click on its front face is the plane, Enter makes the copy. Reported behaviour: with no part
    /// picked before it the button said "pick a part first" and put nothing in hand.
    fn a_mirrored_copy_taken_with_nothing_picked_waits_for_a_part() {
        let mut s = qymcad_acceptance::contract::fixtures::Fixture::BlockInAssembly.start();
        s.key(Key::Escape);
        let tool = s.word("tb-mirror-part-hint");
        s.press_hint(&tool);
        let wait = s.word("tb-mirror-pick-part");
        assert!(s.words().contains(&wait), "the tool taken with nothing picked shows no bar asking for a part; on screen: {:?}", s.words());
        let body = s.face_at([20.0, 15.0, 10.0]);
        s.click(body); // the part
        let front = s.face_at([20.0, 0.0, 5.0]);
        s.click(front); // the plane
        s.key(Key::Enter);
        let parts = s.document().parts.iter().filter(|p| !p.assembly).count();
        assert!(parts == 2, "the copy was not made: the assembly holds {parts} parts; the status line says {:?}", s.status());
    }
}
