//! WHAT A PERSON SEES IS READ BACK: the document as the tree, the properties and a measure show it, the status
//! line, the windows open, the tool in hand by its bar of options, the words and where they stand, a picture of
//! the window.
use qymcad::{pos2, Key, Session};
use qymcad_acceptance::build;
use qymcad_acceptance::probe;

/// Close enough, for numbers measured on a body.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * b.abs().max(1.0)
}

probe! {
    /// THE DOCUMENT IS READ AS BUILT: one part, its sketch, its timeline, its body with the numbers of a 40 x 30 x 10
    /// block, and a step of undo per step taken.
    fn the_document_is_read_as_it_was_built() {
        let mut s = Session::start();
        let empty = s.document();
        assert!(empty.bodies.is_empty() && empty.sketches.is_empty() && empty.undo.is_empty(), "a first start holds something already: {empty:?}");
        build::block(&mut s);
        let doc = s.document();
        let part = s.document().parts.first().map(|p| p.name.clone()).unwrap_or_default();
        assert_eq!(doc.parts.len(), 1, "one part was built in: {:?}", doc.parts);
        assert!(!part.contains('#') && !part.contains("name-"), "the part is read by a catalogue key, not by the name a person sees: {part:?}");
        let [sketch] = doc.sketches.as_slice() else { panic!("one sketch was drawn, the document holds {:?}", doc.sketches) };
        // four corners and the centre of the rectangle, four sides
        assert_eq!((sketch.points, sketch.lines, sketch.part.as_deref()), (5, 4, Some(part.as_str())), "the rectangle's sketch: {sketch:?}");
        let kinds: Vec<&str> = doc.features.iter().map(|f| f.kind.as_str()).collect();
        assert_eq!(kinds, ["Sketch", "Extrude"], "the timeline of a sketch and an extrusion: {:?}", doc.features);
        assert!(doc.features.iter().all(|f| f.error.is_none()), "a node of the block did not build: {:?}", doc.features);
        let [body] = doc.bodies.as_slice() else { panic!("one body was built, the document holds {:?}", doc.bodies) };
        assert!(near(body.volume, 12000.0) && near(body.area, 3800.0), "the block holds {} mm^3 over {} mm^2, not 12000 over 3800", body.volume, body.area);
        assert_eq!((body.faces, body.edges), (6, Some(12)), "the block's faces and edges");
        assert_eq!((body.min, body.max), ([0.0, 0.0, 0.0], [40.0, 30.0, 10.0]), "the box around the block");
        assert!(!doc.undo.is_empty() && doc.redo.is_empty(), "the steps of building are not steps of undo: {:?} / {:?}", doc.undo, doc.redo);
    }
}

probe! {
    /// WHAT A BODY IS MADE OF IS READ: the block is one correct solid of six planes, and a rounded edge adds a
    /// cylinder to them.
    fn what_a_body_is_made_of_is_read() {
        let mut s = Session::start();
        build::block(&mut s);
        let block = s.inspect(0).unwrap_or_else(|| panic!("the block has no exact body to read"));
        assert!(block.valid && block.solids == 1 && block.shells == 1, "the block is not one correct solid: {block:?}");
        assert_eq!(block.kinds, qymcad::FaceKinds { plane: 6, ..Default::default() }, "the block's faces by kind");
        let hint = s.word("tb-fillet-body-hint");
        s.press_hint(&hint);
        let edge = s.edge_at([20.0, 0.0, 10.0]);
        s.click(edge);
        let radius = s.word("f-radius");
        s.fill(&radius, "2");
        s.key(Key::Enter);
        let body = s.document().bodies.iter().rposition(|b| !b.consumed).unwrap_or_else(|| panic!("the fillet left no body"));
        let rounded = s.inspect(body).unwrap_or_else(|| panic!("the rounded block has no exact body to read"));
        assert_eq!(rounded.kinds, qymcad::FaceKinds { plane: 6, cylinder: 1, ..Default::default() }, "the rounded block's faces by kind");
    }
}

probe! {
    /// THE STATUS LINE IS READ: what it says is what the program set, and it is written at the bottom of the window.
    fn the_status_line_is_read() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        let xy = s.word("plane-xy-table");
        s.press_word(&xy);
        let hint = s.word("tb-rect-hint");
        s.press_hint(&hint);
        let status = s.status();
        assert!(!status.is_empty(), "the status line says nothing with the rectangle in hand");
        let bottom = s.screen().y;
        let at = s.find(&status, pos2(0.0, bottom)).unwrap_or_else(|| panic!("the status {status:?} is not written on screen"));
        assert!(at.center().y > bottom - 40.0, "the status {status:?} is written at {at:?}, not at the bottom of the window");
    }
}

probe! {
    /// THE WINDOWS OPEN INSIDE THE PROGRAM ARE READ BY THEIR TITLES.
    fn open_windows_are_read_by_their_titles() {
        let mut s = Session::start();
        let start = s.word("start-title");
        assert_eq!(s.windows(), std::slice::from_ref(&start), "a first start shows the start screen alone");
        s.key(Key::Escape);
        assert!(s.windows().is_empty(), "Esc left a window open: {:?}", s.windows());
        let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
        s.menu(&[&windows, &settings]);
        assert_eq!(s.windows(), [s.word("win-settings")], "Windows, Settings");
    }
}

probe! {
    /// THE TOOL IN HAND IS READ OFF ITS BAR OF OPTIONS, and nothing is in hand without one.
    fn the_tool_in_hand_is_read_off_its_bar() {
        let mut s = Session::start();
        assert!(s.in_hand().is_empty(), "a first start has something in hand: {:?}", s.in_hand());
        build::into_the_first_part(&mut s);
        build::rectangle_on_xy(&mut s);
        let two_corners = s.word("opt-rect-2corners");
        assert!(s.in_hand().contains(&two_corners), "the rectangle in hand is not read off its bar: {:?}", s.in_hand());
        let (file, finish_word) = (s.word("menu-file"), s.word("wb-finish"));
        assert!(!s.in_hand().contains(&file) && !s.in_hand().contains(&finish_word), "words from outside the bar are read as in hand: {:?}", s.in_hand());
        let finish = s.word("wb-finish");
        s.press_word(&finish);
        assert!(s.in_hand().is_empty(), "after Finish something is still in hand: {:?}", s.in_hand());
        let extrude = s.word("tb-extrude-hint");
        s.press_hint(&extrude);
        let apply = s.word("cmd-apply-enter");
        assert!(s.in_hand().contains(&apply), "the extrusion in hand is not read off its bar: {:?}", s.in_hand());
        let cancel = s.word("cmd-cancel-esc");
        s.press_word_near(&cancel, pos2(0.0, 0.0));
        assert!(s.in_hand().is_empty(), "Cancel left the extrusion in hand: {:?}", s.in_hand());
    }
}

probe! {
    /// THE WORDS ARE READ WITH WHERE THEY STAND, the same places a click finds them at.
    fn words_are_read_with_where_they_stand() {
        let mut s = Session::start();
        let words = s.words_at();
        let file = s.word("menu-file");
        let (_, at) = words.iter().find(|(w, _)| *w == file).unwrap_or_else(|| panic!("{file:?} is not among the words: {words:?}"));
        assert_eq!(s.find(&file, pos2(0.0, 0.0)), Some(*at), "the place a word is read at is not the place it is found at");
        assert!(at.min.y < 30.0 && at.min.x < 30.0, "File is read at {at:?}, not in the top left corner of the window");
    }
}

probe! {
    /// A PICTURE OF THE WINDOW IS TAKEN, as large as the window, and it shows what changed on screen.
    fn a_picture_of_the_window_is_taken() {
        let mut s = Session::start();
        let before = s.snapshot();
        assert_eq!((before.width, before.height), (1280, 800), "the picture is not the size of the window");
        let distinct: std::collections::HashSet<&[u8]> = before.rgba.chunks(4).collect();
        assert!(distinct.len() > 16, "the picture of a window full of words holds {} colours", distinct.len());
        s.key(Key::Escape);
        let after = s.snapshot();
        let changed = before.rgba.chunks(4).zip(after.rgba.chunks(4)).filter(|(a, b)| a != b).count();
        assert!(changed > 10_000, "the start screen went away and {changed} pixels of the picture changed");
        let png = after.png();
        assert_eq!(&png[1..4], b"PNG", "the picture does not write as a PNG");
    }
}
