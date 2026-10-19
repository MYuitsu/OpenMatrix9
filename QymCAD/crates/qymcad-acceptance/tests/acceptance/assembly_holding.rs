//! WHAT ELSE HOLDS PARTS TOGETHER: the anchor that grounds one, the group that holds several where they stand, the
//! width that sits a part between two walls, the tangent that rests a round part on a flat one, and the relation
//! that ties two degrees of freedom.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

/// An assembly of `n` parts, each holding a 40 x 30 x 10 block, the second and the third set 60 and 120 along X.
/// The session stands in the assembly.
fn blocks(n: usize) -> Session {
    let mut s = Session::start();
    s.key(Key::Escape);
    build::block(&mut s);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    for i in 1..n {
        let new_part = s.word("tb-new-part-hint");
        s.press_hint(&new_part);
        let made = s.document().parts.last().cloned().expect("the new part").name;
        build::rectangle_on_xy(&mut s);
        let finish = s.word("wb-finish");
        s.press_word(&finish);
        let extrude = s.word("tb-extrude-hint");
        s.press_hint(&extrude);
        s.key(Key::Enter);
        s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
        let row = s.find(&made, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {made:?} is not in the tree"));
        s.click(row.center());
        s.fill("X", &(60 * i).to_string()).key(Key::Enter);
    }
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

/// Put the part `name` at `x` along X by the numbers of its placement.
fn put_at(s: &mut Session, name: &str, x: f64) {
    let row = s.find(name, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {name:?} is not in the tree"));
    s.click(row.center());
    s.fill("X", &x.to_string()).key(Key::Enter);
}

probe! {
    /// GROUNDING FIXES A PART AND LETS IT GO AGAIN, and the program says which it was.
    fn a_part_is_grounded_and_released() {
        let mut s = blocks(2);
        let second = parts(&mut s)[1].name.clone();
        assert!(!parts(&mut s)[1].grounded, "the second part is fixed before anything was done to it");
        take(&mut s, "tb-ground-hint");
        let at = s.face_at([80.0, 15.0, 10.0]);
        s.click(at);
        assert!(parts(&mut s)[1].grounded, "the part was not fixed by the anchor tool: the program says {:?}", s.status());
        let _ = &second;
        s.click(at);
        assert!(!parts(&mut s)[1].grounded, "the part was not let go by a second click: the program says {:?}", s.status());
    }
}

probe! {
    /// GROUNDING SAYS WHICH PART IT FIXED: the catalogue holds the words for it, and a person needs to know which of
    /// the parts is now the anchor.
    fn grounding_says_which_part_it_fixed() {
        let mut s = blocks(2);
        let second = parts(&mut s)[1].name.clone();
        take(&mut s, "tb-ground-hint");
        let at = s.face_at([80.0, 15.0, 10.0]);
        s.click(at);
        assert!(s.status().contains(&second), "nothing says which part was fixed: the program says {:?}", s.status());
    }
}

probe! {
    /// A GROUP HOLDS THE PARTS WHERE THEY STAND: moving one carries the other with it. The first part of an assembly
    /// is its anchor and does not move, so the group is made of the second and the third.
    fn a_group_holds_the_parts_together() {
        let mut s = blocks(3);
        let (second, third) = (parts(&mut s)[1].name.clone(), parts(&mut s)[2].name.clone());
        take(&mut s, "j-group-tip");
        for p in [[80.0, 15.0, 10.0], [140.0, 15.0, 10.0]] {
            let at = s.face_at(p);
            s.click(at);
        }
        s.key(Key::Enter);
        let said = s.status();
        let before = parts(&mut s)[2].at;
        // the second part is pulled by the arm of its gizmo, as a person moves a part
        let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
        s.click(row.center());
        let (origin, along_x) = (s.in_space([60.0, 0.0, 0.0]), s.in_space([61.0, 0.0, 0.0]));
        let dir = (along_x - origin).normalized();
        s.drag(origin + dir * 30.0, origin + dir * 90.0, qymcad::PointerButton::Primary, qymcad::Modifiers::default());
        let moved = parts(&mut s)[1].at[0] - 60.0;
        assert!(moved.abs() > 1.0, "the second part did not move at all: it stands at {:?}", parts(&mut s)[1].at);
        let now = parts(&mut s)[2].at;
        assert!((now[0] - before[0] - moved).abs() < 0.5, "the group did not carry {third:?} the {moved} the second went: {before:?} became {now:?} (on making the group the program said {said:?})");
    }
}

probe! {
    /// A TANGENT RESTS A ROUND PART ON A FLAT ONE: the cylinder sits on the top of the block, touching it.
    fn a_tangent_rests_a_cylinder_on_a_plane() {
        let mut s = blocks(1);
        // a second part holding a cylinder of radius 10, set aside
        let assembly = s.word("wb-assembly");
        let new_part = s.word("tb-new-part-hint");
        s.press_hint(&new_part);
        let second = parts(&mut s).last().cloned().expect("the second part").name;
        take(&mut s, "tb-cylinder-hint");
        s.key(Key::Enter);
        s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
        put_at(&mut s, &second, 80.0);
        take(&mut s, "j-tangent-tip");
        let flat = s.face_at([20.0, 15.0, 10.0]);
        s.click(flat);
        let round = the_side_of_a_shaft(&mut s, 10.0, 10.0, 80.0);
        s.click(round);
        s.key(Key::Enter);
        // the origin of the cylinder is on its axis: lying on the top at 10, the axis is one radius above, at 20, and
        // runs along the face
        let cyl = parts(&mut s).last().cloned().expect("the cylinder");
        let (at, axis) = (cyl.at, cyl.axes[2]);
        assert!(axis[2].abs() < 1e-3 && (at[2] - 20.0).abs() < 0.5, "the cylinder of radius 10 does not lie on the top of the block at 10: its axis runs {axis:?} through {at:?} and the program says {:?}", s.status());
    }
}

/// WHERE THE ROUND SIDE OF A SHAFT ABOUT (`cx`, 0) CAN BE CLICKED: only the facets turned towards the eye can be.
fn the_side_of_a_shaft(s: &mut Session, r: f64, z: f64, cx: f64) -> qymcad::Pos2 {
    for k in 0..(72 * 3) {
        let a = (k % 72) as f64 * std::f64::consts::TAU / 72.0;
        let r = r - 0.05 * (k / 72) as f64;
        let p = [cx + r * a.cos(), r * a.sin(), z];
        let mut seen = None;
        let message = qymcad_acceptance::refusal(|| seen = Some(s.face_at(p)));
        if message.is_empty() {
            if let Some(at) = seen {
                return at;
            }
        }
    }
    panic!("no place on the round side of the shaft can be seen from here");
}

/// TURN THE VIEW UNTIL THE PLACE `p` CAN BE CLICKED: the eye sees one side of a body at a time, and a check that
/// needs both walls of a part has to walk round it, as a person does with the mouse.
fn turn_until_it_shows(s: &mut Session, p: [f64; 3]) -> qymcad::Pos2 {
    for _ in 0..12 {
        let mut seen = None;
        let message = qymcad_acceptance::refusal(|| seen = Some(s.face_at(p)));
        if message.is_empty() {
            if let Some(at) = seen {
                return at;
            }
        }
        let middle = s.canvas().center();
        s.drag(middle, middle + qymcad::vec2(160.0, 0.0), qymcad::PointerButton::Primary, qymcad::Modifiers::default());
    }
    panic!("the view could not be turned to show {p:?}");
}

probe! {
    /// WIDTH SITS A PART HALFWAY BETWEEN TWO WALLS: the small block ends up in the middle of the 40 between the sides
    /// of the big one.
    fn width_sits_a_part_halfway_between_two_walls() {
        let mut s = blocks(1);
        // a second part holding a 10 by 10 by 10 block, set aside
        let assembly = s.word("wb-assembly");
        let new_part = s.word("tb-new-part-hint");
        s.press_hint(&new_part);
        let second = parts(&mut s).last().cloned().expect("the second part").name;
        let xy = s.word("plane-xy-table");
        s.press_word(&xy);
        build::draw(&mut s, "tb-rect-hint", &[(0.0, 0.0), (10.0, 10.0)]);
        let finish = s.word("wb-finish");
        s.press_word(&finish);
        let extrude = s.word("tb-extrude-hint");
        s.press_hint(&extrude);
        s.key(Key::Enter);
        s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
        put_at(&mut s, &second, 80.0);
        take(&mut s, "j-width-tip");
        let right = s.face_at([40.0, 15.0, 5.0]);
        s.click(right);
        let left = turn_until_it_shows(&mut s, [0.0, 15.0, 5.0]);
        s.click(left);
        let between = turn_until_it_shows(&mut s, [85.0, 5.0, 10.0]);
        s.click(between);
        s.key(Key::Enter);
        let at = parts(&mut s).last().cloned().expect("the small block").at;
        assert!((at[0] + at[0] + 10.0 - 40.0).abs() < 0.5, "the block of 10 between walls 40 apart sits with its middle at 20, so it stands at 15: it stands at {at:?} and the program says {:?}", s.status());
    }
}

probe! {
    /// A CONNECTOR IS MADE ON ITS OWN, ahead of any mate: the tool asks where it goes, and the part keeps it.
    fn a_connector_is_made_on_its_own() {
        let mut s = blocks(2);
        let second = parts(&mut s)[1].name.clone();
        let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree"));
        s.click(row.center());
        let make = s.word("j-conn-new");
        s.press_word_near(&make, qymcad::pos2(1100.0, 500.0));
        assert!(s.status() == s.word("j-conn-pick"), "the connector tool does not ask where the connector goes: the program says {:?}", s.status());
        let at = s.face_at([80.0, 15.0, 10.0]);
        s.click(at);
        assert!(s.status() == s.word("j-conn-made-ok"), "nothing says the connector was made: the program says {:?}", s.status());
        let made = s.word("j-conn-made");
        assert!(s.shows(&made), "the connector is not in the panel of the part; on screen: {:?}", s.words());
    }
}

/// A revolute mate held as it stands, between the part whose corner is at `a` and the one whose corner is at `b`.
fn a_hinge_between(s: &mut Session, a: [f64; 3], b: [f64; 3]) -> Option<qymcad::JointInfo> {
    let before = s.document().joints.len();
    let start = s.word("jp-start-joint");
    s.press_word_near(&start, qymcad::pos2(1100.0, 400.0));
    let caption = s.word("j-kind");
    let list = s.field(&caption);
    s.click(list.rect.center());
    let want = s.word("joint-kind-revolute");
    s.press_word_near(&want, list.rect.center());
    let as_built = s.word("j-as-built");
    s.press_word_near(&as_built, qymcad::pos2(640.0, 24.0)); // the parts keep their places
    let at = s.vertex_at(a);
    s.click(at);
    let at = s.vertex_at(b);
    s.click(at);
    s.key(Key::Enter).key(Key::Escape); // Enter keeps the mate the second pick made, Esc puts the tool down
    (s.document().joints.len() > before).then(|| s.document().joints.last().cloned().expect("the mate"))
}

probe! {
    /// A RELATION TIES TWO MATES: driving one degree of freedom turns the part on the other. Two mates are needed for
    /// it, and the second one is where this stops.
    fn a_relation_ties_two_mates() {
        let mut s = blocks(3);
        let first = a_hinge_between(&mut s, [40.0, 0.0, 10.0], [100.0, 0.0, 10.0]).unwrap_or_else(|| panic!("the first mate was not made: the program says {:?}", s.status()));
        let second = a_hinge_between(&mut s, [40.0, 0.0, 0.0], [160.0, 0.0, 10.0]);
        assert!(second.is_some(), "the second mate of the assembly was not made: the corner of the third part was under the pointer and the click was not taken - the program still says {:?}", s.status());
        let second = second.expect("the second mate");
        take(&mut s, "j-relation-tip");
        assert!(s.status() == s.word("j-relation-pick"), "the relation tool does not ask for the mates: the program says {:?}", s.status());
        let rows: Vec<qymcad::Rect> = [first.name.clone(), second.name.clone()]
            .into_iter()
            .map(|name| s.find(&name, qymcad::pos2(1100.0, 600.0)).unwrap_or_else(|| panic!("the mate {name:?} is not in the list; on screen: {:?}", s.words())))
            .collect();
        for row in rows {
            s.click(row.center());
        }
        s.key(Key::Enter);
        assert!(s.status() == s.word("j-relation-made-ok"), "nothing says the relation was made: the program says {:?}", s.status());
        let before = parts(&mut s)[2].axes;
        let row = s.find(&first.name, qymcad::pos2(1100.0, 600.0)).unwrap_or_else(|| panic!("the mate {:?} is not in the list", first.name));
        s.click(row.center());
        let angle = s.word("j-angle-lower");
        s.fill(&angle, "45").key(Key::Enter);
        let now = parts(&mut s)[2].axes;
        assert!(now != before, "the relation did not carry the turn to the other mate: the third part looks {now:?} as before");
    }
}
