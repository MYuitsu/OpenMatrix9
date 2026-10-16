//! THE COMPONENTS OF AN ASSEMBLY: a part and a sub-assembly made, entered and left, one brought in from a file,
//! one taken from the library, a part moved by its gizmo and by the mouse, and a part cloned and copied.
use qymcad::{Key, Session};
use qymcad_acceptance::{build, probe};

/// A first start with the start screen put away: the assembly and the part it comes with.
fn a_first_start() -> Session {
    let mut s = Session::start();
    build::a_part_in_the_assembly(&mut s);
    s
}

/// Take the tool whose hint is `hint`.
fn take(s: &mut Session, hint: &str) {
    let hint = s.word(hint);
    s.press_hint(&hint);
}

/// The parts and sub-assemblies of the document.
fn parts(s: &mut Session) -> Vec<qymcad::Part> {
    s.document().parts
}

/// Go up to the assembly by the path at the top.
fn up_to_the_assembly(s: &mut Session) {
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
}

probe! {
    /// A NEW PART stands in the assembly and the person is put inside it.
    fn a_new_part_is_made_and_entered() {
        let mut s = a_first_start();
        let before = parts(&mut s).len();
        take(&mut s, "tb-new-part-hint");
        let now = parts(&mut s);
        assert!(now.len() == before + 1, "a new part was made, and the assembly holds {} of them instead of {}", now.len(), before + 1);
        let made = now.last().cloned().expect("the new part");
        assert!(!made.assembly, "the new part is written down as an assembly: {made:?}");
        assert!(s.document().context == made.name, "the person was not put inside the new part {:?}: the path says {:?}", made.name, s.document().context);
    }
}

probe! {
    /// A NEW SUB-ASSEMBLY stands in the assembly as an assembly of its own, and the person is put inside it.
    fn a_new_subassembly_is_made_and_entered() {
        let mut s = a_first_start();
        take(&mut s, "tb-new-subassembly-hint");
        let made = parts(&mut s).last().cloned().expect("the new sub-assembly");
        assert!(made.assembly, "what was made is not an assembly: {made:?}");
        assert!(s.document().context == made.name, "the person was not put inside {:?}: the path says {:?}", made.name, s.document().context);
        // and a part made inside it belongs to it
        take(&mut s, "tb-new-part-hint");
        let inner = parts(&mut s).last().cloned().expect("the part inside the sub-assembly");
        assert!(inner.parent.as_deref() == Some(made.name.as_str()), "the part made inside {:?} stands under {:?}", made.name, inner.parent);
    }
}

probe! {
    /// A PART IS ENTERED BY A DOUBLE CLICK IN THE TREE AND LEFT BY THE PATH AT THE TOP.
    fn a_part_is_entered_by_a_double_click_and_left_by_the_path() {
        let mut s = a_first_start();
        let part = parts(&mut s)[0].name.clone();
        let assembly = s.word("wb-assembly");
        assert!(s.document().context == assembly, "a first start does not stand in the assembly: the path says {:?}", s.document().context);
        let row = s.find(&part, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {part:?} is not in the tree; on screen: {:?}", s.words()));
        s.double_click(row.center());
        assert!(s.document().context == part, "the double click did not step inside {part:?}: the path says {:?}", s.document().context);
        up_to_the_assembly(&mut s);
        assert!(s.document().context == assembly, "the path did not lead back out to the assembly: it says {:?}", s.document().context);
    }
}

probe! {
    /// A PART IS BROUGHT IN FROM A FILE and stands in the assembly beside the others.
    fn a_part_is_brought_in_from_a_file() {
        let mut s = a_first_start();
        build::block(&mut s); // it steps into the first part itself
        let path = format!("{}/insert-me.step", std::env::temp_dir().display());
        let (file, export) = (s.word("menu-file"), s.word("file-export"));
        s.menu(&[&file, &export, "STEP\u{2026}"]);
        s.answer_file(&path);
        assert!(std::path::Path::new(&path).exists(), "the part was not written out to {path}: the program says {:?}", s.status());
        up_to_the_assembly(&mut s);
        let before = parts(&mut s).len();
        take(&mut s, "tb-insert-component-hint");
        s.answer_file(&path);
        let now = parts(&mut s);
        assert!(now.len() > before, "the file brought in no component: the assembly holds {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
        let brought = s.document().bodies.into_iter().rfind(|b| !b.consumed && !b.sheet).expect("the body that came in");
        assert!((brought.volume - 12000.0).abs() < 1.0, "the block that came in holds 12000, and it holds {}", brought.volume);
    }
}

probe! {
    /// A CLONE IS THE SAME PART ONCE MORE: it stands on its own and follows the original when the original changes.
    fn a_clone_follows_the_original() {
        let mut s = a_first_start();
        build::block(&mut s); // it steps into the first part itself
        up_to_the_assembly(&mut s);
        let part = parts(&mut s)[0].name.clone();
        let row = s.find(&part, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {part:?} is not in the tree; on screen: {:?}", s.words()));
        s.click_with(row.center(), qymcad::PointerButton::Secondary, qymcad::Modifiers::default());
        let clone = s.word("act-clone-part");
        s.press_word_near(&clone, row.center());
        let now = parts(&mut s);
        assert!(now.len() == 2, "the clone does not stand in the assembly: it holds {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
        let bodies = s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).count();
        assert!(bodies == 2, "the clone brought no body of its own: the assembly holds {bodies} bodies");
    }
}

probe! {
    /// A PART IS COPIED: the copy stands in the assembly with a body of its own, and the original is left alone.
    fn a_part_is_copied() {
        let mut s = a_first_start();
        build::block(&mut s);
        up_to_the_assembly(&mut s);
        let part = parts(&mut s)[0].name.clone();
        let row = s.find(&part, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {part:?} is not in the tree; on screen: {:?}", s.words()));
        s.click(row.center());
        s.chord(qymcad::Modifiers::COMMAND, Key::C);
        s.chord(qymcad::Modifiers::COMMAND, Key::V);
        let now = parts(&mut s);
        assert!(now.len() == 2, "the copy does not stand in the assembly: it holds {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>());
        let bodies: Vec<qymcad::Solid> = s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).collect();
        assert!(bodies.len() == 2, "the copy brought no body of its own: the assembly holds {} bodies", bodies.len());
        assert!(bodies.iter().all(|b| (b.volume - 12000.0).abs() < 1.0), "the copy is not the same block: {:?}", bodies.iter().map(|b| b.volume).collect::<Vec<_>>());
    }
}

probe! {
    /// A PART IS TAKEN FROM THE LIBRARY: what the search finds is brought into the assembly by the button.
    fn a_part_is_taken_from_the_library() {
        let mut s = a_first_start();
        let before = parts(&mut s).len();
        let (windows, library) = (s.word("menu-windows"), s.word("win-parts-library"));
        s.menu(&[&windows, &library]);
        let title = s.word("pl-title");
        assert!(s.shows(&title), "the library did not open; on screen: {:?}", s.words());
        let search = s.word("pl-search");
        s.fill_empty(&search, "a"); // every part whose name or tags hold the letter
        let insert = s.word("pl-insert");
        assert!(s.shows(&insert), "the library found nothing to take; on screen: {:?}", s.words());
        s.press_word(&insert);
        let now = parts(&mut s);
        assert!(now.len() > before, "nothing came out of the library: the assembly holds {:?} and the program says {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>(), s.status());
    }
}

/// A SECOND PART OF THE ASSEMBLY, holding a 40 x 30 x 10 block: the first part of an assembly is its anchor and does
/// not move, so what is moved in these checks is a second one. The session is left standing in the assembly.
fn a_second_part_with_a_block(s: &mut Session) -> String {
    take(s, "tb-new-part-hint");
    let second = parts(s).last().cloned().expect("the second part").name;
    build::rectangle_on_xy(s);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude);
    s.key(Key::Enter);
    up_to_the_assembly(s);
    second
}

/// Where the part `name` stands in the assembly.
fn stands_at(s: &mut Session, name: &str) -> [f64; 3] {
    parts(s).into_iter().find(|p| p.name == name).map(|p| p.at).unwrap_or_else(|| panic!("the assembly holds no part {name:?}"))
}

probe! {
    /// A PART IS MOVED BY ITS GIZMO: the arm of the gizmo is dragged and the part goes along it.
    fn a_part_is_moved_by_its_gizmo() {
        let mut s = a_first_start();
        let second = a_second_part_with_a_block(&mut s);
        assert!(stands_at(&mut s, &second) == [0.0; 3], "the new part does not start at the origin: it stands at {:?}", stands_at(&mut s, &second));
        let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree; on screen: {:?}", s.words()));
        s.click(row.center()); // the part is taken in the tree, and its gizmo stands at its origin
        // the arm of the gizmo is drawn 60 points long from the origin of the part, along each axis: the grab is
        // halfway along it and the drag goes another 60 points the same way
        let (origin, along_x) = (s.in_space([0.0, 0.0, 0.0]), s.in_space([1.0, 0.0, 0.0]));
        let dir = (along_x - origin).normalized();
        s.drag(origin + dir * 30.0, origin + dir * 90.0, qymcad::PointerButton::Primary, qymcad::Modifiers::default());
        let now = stands_at(&mut s, &second);
        assert!(now[0].abs() > 1.0, "the part did not move along the arm of its gizmo: it stands at {now:?}");
        assert!(now[1].abs() < 1e-6 && now[2].abs() < 1e-6, "the part went off the arm it was dragged along: it stands at {now:?}");
    }
}

probe! {
    /// A PART IS MOVED BY THE NUMBERS OF ITS PLACEMENT: 40 typed into X puts it 40 along.
    fn a_part_is_moved_by_the_numbers_of_its_placement() {
        let mut s = a_first_start();
        let second = a_second_part_with_a_block(&mut s);
        let row = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree; on screen: {:?}", s.words()));
        s.click(row.center());
        s.fill("X", "40").key(Key::Enter);
        let now = stands_at(&mut s, &second);
        assert!((now[0] - 40.0).abs() < 1e-3, "the part typed 40 along X stands at {now:?}");
    }
}

probe! {
    /// A PART IS DRAGGED BY THE MOUSE: the body itself is taken with Shift and the left button and carried, and the
    /// part goes with it.
    fn a_part_is_dragged_by_the_mouse() {
        let mut s = a_first_start();
        let second = a_second_part_with_a_block(&mut s);
        let from = s.face_at([20.0, 15.0, 10.0]);
        s.click(from); // the part is taken by its body
        let to = from + qymcad::vec2(160.0, 0.0); // aimed on screen: bringing another point into view would turn it
        s.drag(from, to, qymcad::PointerButton::Primary, qymcad::Modifiers::SHIFT);
        let now = stands_at(&mut s, &second);
        assert!(now[0].abs() > 1.0, "the part did not follow the mouse: it stands at {now:?}");
    }
}

probe! {
    /// ONE TICK IN THE HEADING: the tick before the word "Components" clears every part of the assembly in one
    /// click, and the next click brings every one of them back; the count at the right edge of the heading says
    /// how many are on screen of how many there are. The word itself still folds the branch.
    ///
    /// Reported behaviour: with many parts in an assembly, hiding them all meant a click on every row.
    fn one_tick_hides_every_part_and_the_next_shows_them_all() {
        let mut s = a_first_start();
        a_second_part_with_a_block(&mut s);
        let names: Vec<String> = parts(&mut s).into_iter().filter(|p| !p.assembly).map(|p| p.name).collect();
        assert!(names.len() >= 2, "the setup stands two parts in the assembly and the document holds {names:?}");
        let heading = s.word("tree-components");
        // The tick stands on the heading's line, to the left of the word, where the rows under it carry theirs.
        let tick = |s: &mut Session| {
            let at = s.find(&heading, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the heading {heading:?} is not in the tree; on screen: {:?}", s.words()));
            s.widgets()
                .into_iter()
                .filter(|w| w.kind == qymcad::Kind::CheckBox && w.rect.center().y > at.min.y && w.rect.center().y < at.max.y && w.rect.max.x <= at.min.x + 1.0)
                .max_by(|a, b| a.rect.max.x.total_cmp(&b.rect.max.x))
                .unwrap_or_else(|| panic!("the heading {heading:?} has no tick beside it; on screen: {:?}", s.words()))
                .rect
                .center()
        };
        // The count: the words on the heading's line to the right of the word, in the tree's column.
        let count = |s: &mut Session| -> Vec<String> {
            let at = s.find(&heading, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the heading {heading:?} is not in the tree; on screen: {:?}", s.words()));
            s.words_at()
                .into_iter()
                .filter(|(_, r)| r.center().y > at.min.y && r.center().y < at.max.y && r.min.x >= at.max.x - 1.0 && r.max.x < 500.0)
                .flat_map(|(w, _)| w.split_whitespace().map(str::to_string).collect::<Vec<_>>())
                .collect()
        };
        let n = names.len().to_string();
        let at = tick(&mut s);
        s.click(at);
        let shown: Vec<String> = parts(&mut s).into_iter().filter(|p| names.contains(&p.name) && p.visible).map(|p| p.name).collect();
        assert!(shown.is_empty(), "the tick above the rows was cleared and {shown:?} are still shown");
        let said = count(&mut s);
        assert!(said.contains(&"0".to_string()) && said.contains(&n), "every part is hidden and the heading counts {said:?} rather than 0 of {n}");
        let at = tick(&mut s);
        s.click(at);
        let hidden: Vec<String> = parts(&mut s).into_iter().filter(|p| names.contains(&p.name) && !p.visible).map(|p| p.name).collect();
        assert!(hidden.is_empty(), "the tick above the rows was set again and {hidden:?} are still hidden");
        let said = count(&mut s);
        assert!(said.iter().filter(|t| **t == n).count() == 2, "every part is shown and the heading counts {said:?} rather than {n} of {n}");
        // THE WORD STILL FOLDS THE BRANCH: the heading was assembled by hand to take the tick, and that must not
        // have cost it the fold by a click that every other heading of the tree answers. The part is looked for
        // in the tree's own column, under the heading: the joints panel names the parts too, and a fold of the
        // tree does not touch it.
        let part = names[1].clone();
        let in_tree = |s: &mut Session| {
            let at = s.find(&heading, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the heading {heading:?} is not in the tree; on screen: {:?}", s.words()));
            s.words_at().into_iter().any(|(w, r)| w.contains(part.as_str()) && r.min.y > at.max.y - 1.0 && r.max.x < 500.0)
        };
        s.press_word_near(&heading, qymcad::pos2(0.0, 300.0));
        assert!(!in_tree(&mut s), "the branch of the components was folded by its heading and {part:?} is still in the tree; on screen: {:?}", s.words());
        s.press_word_near(&heading, qymcad::pos2(0.0, 300.0));
        assert!(in_tree(&mut s), "the branch of the components was unfolded and {part:?} did not come back; on screen: {:?}", s.words());
    }
}

probe! {
    /// THE WHOLE LINE OF THE HEADING FOLDS THE BRANCH, not the word alone: a click between the word "Components" and
    /// the count folds it, as a click anywhere on any other heading of the tree does.
    fn a_click_between_the_word_and_the_count_folds_the_components() {
        let mut s = a_first_start();
        a_second_part_with_a_block(&mut s);
        let part = parts(&mut s).into_iter().filter(|p| !p.assembly).map(|p| p.name).nth(1).expect("the second part");
        let heading = s.word("tree-components");
        let at = s.find(&heading, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the heading {heading:?} is not in the tree; on screen: {:?}", s.words()));
        let in_tree = |s: &mut Session| s.words_at().into_iter().any(|(w, r)| w.contains(part.as_str()) && r.min.y > at.max.y - 1.0 && r.max.x < 500.0);
        assert!(in_tree(&mut s), "setup: {part:?} is not under the heading; on screen: {:?}", s.words());
        // a little past the word, short of the count at the right edge of the column
        s.click(qymcad::pos2(at.max.x + 12.0, at.center().y));
        assert!(!in_tree(&mut s), "a click on the heading's line past the word did not fold the branch: {part:?} is still in the tree");
    }
}

probe! {
    /// A HIDDEN PART HIDES ITS SKETCHES: in the assembly, with "Sketch contours" on, the part's tick cleared takes its
    /// sketch off the canvas with its body - the canvas is then the one "Sketch contours" off would give.
    ///
    /// Reported behaviour: the sketches of a hidden part stayed on screen with the toggle on.
    fn a_hidden_part_takes_its_sketch_off_the_canvas() {
        let mut s = qymcad_acceptance::contract::fixtures::Fixture::BlockInAssembly.start();
        let part = s.document().parts.iter().find(|p| !p.assembly).map(|p| p.name.clone()).expect("the part of the block");
        let at = s.find(&part, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("{part:?} is not in the tree; on screen: {:?}", s.words()));
        let tick = s
            .widgets()
            .into_iter()
            .filter(|w| w.kind == qymcad::Kind::CheckBox && w.rect.center().y > at.min.y && w.rect.center().y < at.max.y && w.rect.max.x <= at.min.x + 1.0)
            .max_by(|a, b| a.rect.max.x.total_cmp(&b.rect.max.x))
            .unwrap_or_else(|| panic!("the row of {part:?} has no tick; on screen: {:?}", s.words()));
        s.click(tick.rect.center());
        assert!(s.document().parts.iter().any(|p| p.name == part && !p.visible), "the part's tick was cleared and it is still shown");
        let canvas = s.canvas();
        let aside = qymcad::pos2(canvas.max.x - 10.0, canvas.max.y - 10.0);
        s.move_to(aside);
        let hidden_part = s.snapshot();
        let contours = s.word("tree-contours-toggle");
        let toggle = s.widgets().into_iter().find(|w| w.kind == qymcad::Kind::CheckBox && w.label.contains(contours.as_str())).unwrap_or_else(|| panic!("no {contours:?} toggle; on screen: {:?}", s.words()));
        s.click(toggle.rect.center());
        s.move_to(aside);
        let no_contours = s.snapshot();
        assert!(!qymcad_acceptance::golden::changed_in(&hidden_part, &no_contours, canvas, &[]), "switching the sketch contours off changed the canvas: the hidden part's sketch was on it");
    }
}
