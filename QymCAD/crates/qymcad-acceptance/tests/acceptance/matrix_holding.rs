//! HOLDING PARTS ACROSS PLACES: grounding a part wherever it stands, and letting it go; a group of two parts at
//! different places, held where they are; a width that sits a tab halfway between two walls at different spacings.
//! Each is read back from the document - which part is grounded, which mates stand, where the tab ended up.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

/// A part of the assembly holding the rectangle from `a` to `b` on XY extruded 10, put at `at`; the session left in
/// the assembly.
fn part(s: &mut Session, a: (f64, f64), b: (f64, f64), at: [f64; 3]) {
    let assembly = s.word("wb-assembly");
    let new_part = s.word("tb-new-part-hint");
    s.press_hint(&new_part);
    let made = s.document().parts.last().cloned().expect("the new part").name;
    let xy = s.word("plane-xy-table");
    s.press_word(&xy);
    build::draw(s, "tb-rect-hint", &[a, b]);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude);
    s.key(Key::Enter);
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    let row = s.find(&made, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {made:?} is not in the tree"));
    s.click(row.center());
    for (caption, v) in ["X", "Y", "Z"].into_iter().zip(at) {
        s.fill(caption, &format!("{v}"));
    }
    s.key(Key::Enter);
}

/// The block in the first part, a second block put at `at`, the view fitted.
fn two_blocks(at: [f64; 3]) -> Session {
    let mut s = Session::start();
    s.key(Key::Escape);
    build::block(&mut s);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    part(&mut s, (0.0, 0.0), (40.0, 30.0), at);
    fit(&mut s);
    s
}

fn fit(s: &mut Session) {
    let (view, fit) = (s.word("menu-view"), s.word("menu-fit-view"));
    s.menu(&[&view, &fit]);
}

fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

/// Every case, each in a session of its own; all that go wrong reported at once.
fn run(cases: Vec<(String, Box<dyn Fn()>)>) {
    let n = cases.len();
    let failed: Vec<String> = cases
        .into_iter()
        .filter_map(|(what, c)| {
            let problem = qymcad_acceptance::refusal(&c);
            (!problem.is_empty()).then(|| format!("{what}: {problem}"))
        })
        .collect();
    assert!(failed.is_empty(), "{} of {n} cases went wrong:\n{}", failed.len(), failed.join("\n"));
}

const PLACES: [[f64; 3]; 3] = [[60.0, 0.0, 0.0], [0.0, 50.0, 0.0], [60.0, 40.0, 20.0]];

probe! {
    budget = 1800;
    /// GROUNDING wherever the part stands: the second block fixed by a click on its top, let go by a second click; and
    /// the first block fixed too.
    fn grounding_across_places() {
        run(PLACES.iter().map(|&at| {
            (format!("the second block at {at:?}"), Box::new(move || {
                let mut s = two_blocks(at);
                take(&mut s, "tb-ground-hint");
                let top = s.face_at([at[0] + 20.0, at[1] + 15.0, at[2] + 10.0]);
                s.click(top);
                let grounded: Vec<bool> = s.document().parts.iter().map(|p| p.grounded).collect();
                assert!(grounded == [false, true], "a click on the second block grounds it alone: {grounded:?}; the program says {:?}", s.status());
                s.click(top);
                let grounded: Vec<bool> = s.document().parts.iter().map(|p| p.grounded).collect();
                assert!(grounded == [false, false], "a second click lets it go: {grounded:?}");
                let first = s.face_at([20.0, 15.0, 10.0]);
                s.click(first);
                let grounded: Vec<bool> = s.document().parts.iter().map(|p| p.grounded).collect();
                assert!(grounded == [true, false], "a click on the first block grounds it alone: {grounded:?}");
            }) as Box<dyn Fn()>)
        }).collect());
    }
}

probe! {
    budget = 1800;
    /// A GROUP holds two blocks where they stand, wherever that is: a mate of the kind "Group", and neither block moved.
    fn groups_across_places() {
        run(PLACES.iter().map(|&at| {
            (format!("the second block at {at:?}"), Box::new(move || {
                let mut s = two_blocks(at);
                take(&mut s, "j-group-tip");
                for p in [[20.0, 15.0, 10.0], [at[0] + 20.0, at[1] + 15.0, at[2] + 10.0]] {
                    let f = s.face_at(p);
                    s.click(f);
                }
                s.key(Key::Enter);
                let doc = s.document();
                assert!(doc.mates == ["Group"], "the two blocks are held by {:?}, not one group; the program says {:?}", doc.mates, s.status());
                let places: Vec<[f64; 3]> = doc.parts.iter().map(|p| p.at).collect();
                assert!(places == [[0.0, 0.0, 0.0], at], "a group holds the parts where they stand, and they stand at {places:?}");
            }) as Box<dyn Fn()>)
        }).collect());
    }
}

probe! {
    budget = 1800;
    /// A WIDTH sits a tab 10 wide halfway between the right faces of two walls, whatever their spacing: walls from 0 to
    /// 10 and from `d` to `d + 10`, the tab from 20 to 30 moved so its middle is at (10 + d + 10) / 2.
    fn widths_across_spacings() {
        run([50.0, 90.0, 30.0].into_iter().map(|d: f64| {
            (format!("walls {d} apart"), Box::new(move || {
                let mut s = Session::start();
                s.key(Key::Escape);
                build::into_the_first_part(&mut s);
                let xy = s.word("plane-xy-table");
                s.press_word(&xy);
                build::draw(&mut s, "tb-rect-hint", &[(0.0, 0.0), (10.0, 30.0)]);
                let finish = s.word("wb-finish");
                s.press_word(&finish);
                let extrude = s.word("tb-extrude-hint");
                s.press_hint(&extrude);
                s.key(Key::Enter);
                let assembly = s.word("wb-assembly");
                s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
                part(&mut s, (d, 0.0), (d + 10.0, 30.0), [0.0; 3]);
                part(&mut s, (20.0, 10.0), (30.0, 20.0), [0.0; 3]);
                fit(&mut s);
                take(&mut s, "j-width-tip");
                for p in [[10.0, 15.0, 5.0], [d + 10.0, 15.0, 5.0], [25.0, 15.0, 10.0]] {
                    let f = s.face_at(p);
                    s.click(f);
                }
                s.key(Key::Enter);
                let doc = s.document();
                assert!(doc.mates == ["Width"], "the tab is held by {:?}, not one width; the program says {:?}", doc.mates, s.status());
                let shift = doc.parts.last().map(|p| p.at[0]).expect("the tab");
                let want = (10.0 + d + 10.0) / 2.0 - 25.0;
                assert!((shift - want).abs() < 1e-3, "the tab moved {shift} along X, it should move {want} to sit halfway between the walls");
            }) as Box<dyn Fn()>)
        }).collect());
    }
}

probe! {
    budget = 1800;
    /// THE CHECK OF INTERFERENCE across places: the second block run 20 into the first sideways, 5 into it from above -
    /// one interference each; touching it face to face and standing apart - none.
    fn interference_across_places() {
        run([([20.0, 0.0, 0.0], true), ([0.0, 0.0, 5.0], true), ([40.0, 0.0, 0.0], false), ([60.0, 0.0, 0.0], false)]
            .into_iter()
            .map(|(at, meets): ([f64; 3], bool)| {
                (format!("the second block at {at:?}"), Box::new(move || {
                    let mut s = two_blocks(at);
                    let check = s.word("tree-interference");
                    s.press_word(&check);
                    let one = s.shows("Interferences: 1");
                    assert!(one == meets, "{} blocks {} on screen \"Interferences: 1\"; on screen: {:?}", if meets { "overlapping" } else { "not overlapping" }, if one { "put" } else { "do not put" }, s.words());
                }) as Box<dyn Fn()>)
            })
            .collect());
    }
}
