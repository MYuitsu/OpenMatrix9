//! WHAT A CLICK IN SPACE TAKES: a face in a part and the body on a double click, the part in an assembly, a face under a tool that asks for one,
//! an edge, a corner, and a frame drawn round several things; what is under the cursor and what is taken are both
//! lit up, and a row taken in the tree lights up on the picture.
use qymcad::{Key, Modifiers, PointerButton, Pos2, Session};
use qymcad_acceptance::{build, golden, probe};

/// THE BLOCK OF THE FIRST PART, seen in space with nothing in hand.
fn a_block() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    s.key(Key::Escape);
    s
}

/// AN ASSEMBLY OF TWO BLOCKS, the person standing in the assembly: 40 x 30 x 10 at the origin and 20 x 20 x 10
/// beside it at x 60.
fn two_blocks() -> Session {
    let mut s = a_block();
    build::into_a_new_part(&mut s);
    let xy = s.word("plane-xy-table");
    s.press_word(&xy);
    let rect = s.word("tb-rect-hint");
    s.press_hint(&rect);
    s.click_on_sketch(60.0, 0.0).click_on_sketch(80.0, 20.0);
    let finish = s.word("wb-finish");
    s.press_word(&finish);
    let extrude = s.word("tb-extrude-hint");
    s.press_hint(&extrude);
    s.key(Key::Enter);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    s
}

/// What the properties panel calls what is chosen now, of the titles it knows; nothing chosen is `None`.
fn chosen(s: &mut Session) -> Option<String> {
    for key in ["mesh-props-title", "face-props-title", "edge-props-title", "vertex-props-title", "props-component", "props-title"] {
        let title = s.word(key);
        if s.shows(&title) {
            return Some(title);
        }
    }
    None
}

probe! {
    /// A CLICK IN A PART TAKES THE FACE, A DOUBLE CLICK THE BODY: one click is the piece under the cursor, and the
    /// properties tell of the face of 1200 mm^2; two tell of the part's body.
    fn a_click_in_a_part_takes_the_face_and_a_double_click_the_body() {
        let mut s = a_block();
        let at = s.face_at([20.0, 15.0, 10.0]);
        s.click(at);
        assert!(chosen(&mut s) == Some(s.word("face-props-title")), "a click on the top of the block took {:?}", chosen(&mut s));
        assert!(s.words().iter().any(|w| w.contains("1200")), "the top of a 40 by 30 block is 1200 mm^2 and nothing on screen says so: {:?}", s.words());
        s.double_click(at);
        assert!(chosen(&mut s) == Some(s.word("mesh-props-title")), "a double click on the top of the block took {:?}", chosen(&mut s));
        let body = s.document().bodies.first().map(|b| b.name.clone()).expect("the body of the block");
        assert!(s.shows(&body), "the body {body:?} was taken and the properties do not name it; on screen: {:?}", s.words());
    }
}

probe! {
    /// A CLICK IN AN ASSEMBLY TAKES THE PART the body belongs to, by its name.
    fn a_click_in_an_assembly_takes_the_part() {
        let mut s = two_blocks();
        let second = s.document().parts.iter().filter(|p| !p.assembly).map(|p| p.name.clone()).next_back().expect("the second part");
        let at = s.face_at([70.0, 10.0, 10.0]);
        s.click(at);
        assert!(chosen(&mut s) == Some(s.word("props-component")), "a click on the top of the second block took {:?}", chosen(&mut s));
        assert!(s.shows(&second), "the part {second:?} was taken and the properties do not name it; on screen: {:?}", s.words());
    }
}

probe! {
    /// A CLICK UNDER A TOOL THAT ASKS FOR A FACE TAKES THAT FACE: the shell asks for one, and the top of the block
    /// is 1200 mm^2.
    fn a_click_under_a_tool_that_asks_takes_the_face() {
        let mut s = a_block();
        let shell = s.word("tb-shell-hint");
        s.press_hint(&shell);
        let at = s.face_at([20.0, 15.0, 10.0]);
        s.click(at);
        assert!(chosen(&mut s) == Some(s.word("face-props-title")), "the shell asks for a face and the click on the top of the block took {:?}", chosen(&mut s));
        assert!(s.words().iter().any(|w| w.contains("1200")), "the top of a 40 by 30 block is 1200 mm^2 and nothing on screen says so: {:?}", s.words());
    }
}

probe! {
    /// A CLICK ON AN EDGE TAKES THAT EDGE, not the whole body behind it: an edge is a thing of its own, and a
    /// person clicking one means it.
    fn a_click_on_an_edge_takes_the_edge() {
        let mut s = a_block();
        let at = s.edge_at([20.0, 0.0, 10.0]);
        s.click(at);
        let took = chosen(&mut s);
        assert!(took != Some(s.word("mesh-props-title")), "a click on the front top edge of the block took the whole body: the properties say {took:?}");
        assert!(took == Some(s.word("edge-props-title")), "a click on the front top edge of the block took {took:?}");
        assert!(s.words().iter().any(|w| w.contains("40")), "the front top edge of a 40 by 30 block is 40 long and nothing on screen says so: {:?}", s.words());
    }
}

probe! {
    /// A CLICK ON A CORNER TAKES THAT CORNER, not the body behind it.
    fn a_click_on_a_corner_takes_the_corner() {
        let mut s = a_block();
        let at = s.vertex_at([40.0, 0.0, 10.0]);
        s.click(at);
        let took = chosen(&mut s);
        assert!(took != Some(s.word("mesh-props-title")), "a click on the corner of the block took the whole body: the properties say {took:?}");
        assert!(took == Some(s.word("vertex-props-title")), "a click on the corner of the block took {took:?}");
    }
}

probe! {
    /// A FRAME DRAWN ROUND SEVERAL PARTS TAKES THEM ALL: the frame starts on empty space and covers both blocks.
    fn a_frame_takes_what_it_covers() {
        let mut s = two_blocks();
        let canvas = s.canvas();
        let (from, to) = (qymcad::pos2(canvas.min.x + 4.0, canvas.min.y + 4.0), qymcad::pos2(canvas.max.x - 4.0, canvas.max.y - 4.0));
        s.drag(from, to, PointerButton::Primary, Modifiers::default());
        let took = chosen(&mut s);
        assert!(took != Some(s.word("props-title")) && took.is_some(), "a frame was drawn round both parts and nothing was taken: the properties say {took:?}");
        let copy = s.word("act-copy-selected").replace("{ $n }", "2").replace("{$n}", "2");
        let at = s.canvas().center();
        s.click_with(at, PointerButton::Secondary, Modifiers::default());
        // the item carries its icon before the words
        assert!(s.words().iter().any(|w| w.contains(&copy)), "a frame was drawn round two parts and the menu does not offer to copy two of them; on screen: {:?}", s.words());
    }
}

probe! {
    /// WHAT IS UNDER THE CURSOR IS LIT, and putting the cursor away puts the picture back.
    fn what_is_under_the_cursor_is_lit() {
        let mut s = a_block();
        let away = qymcad::pos2(s.canvas().min.x + 4.0, s.canvas().max.y - 4.0);
        s.move_to(away);
        let plain = s.snapshot();
        let at = s.face_at([20.0, 15.0, 10.0]);
        s.move_to(at);
        assert!(!golden::same(&plain, &s.snapshot()), "the cursor stands on the top of the block and nothing on the picture says so");
        s.move_to(away);
        assert!(golden::same(&plain, &s.snapshot()), "the cursor was taken off the block and the picture did not go back to what it was");
    }
}

/// A binary STL of a 40 x 30 x 10 box standing on the origin: twelve triangles, two to a side, wound outwards.
fn box_stl(path: &str) {
    let (x, y, z) = (40.0f32, 30.0f32, 10.0f32);
    let p = |i: usize| [if i & 1 != 0 { x } else { 0.0 }, if i & 2 != 0 { y } else { 0.0 }, if i & 4 != 0 { z } else { 0.0 }];
    // each side as four corners going round counter-clockwise seen from outside
    let sides = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
    let mut out = vec![0u8; 80];
    out.extend(12u32.to_le_bytes());
    for s in sides {
        for t in [[s[0], s[1], s[2]], [s[0], s[2], s[3]]] {
            out.extend([0u8; 12]);
            for c in t {
                for v in p(c) {
                    out.extend(v.to_le_bytes());
                }
            }
            out.extend([0u8; 2]);
        }
    }
    std::fs::write(path, out).expect("the mesh of a box is written");
}

probe! {
    /// THE FACE UNDER THE CURSOR IS LIT ON A BODY OF A MESH, not the first face of that body. Such a body has no
    /// B-rep, so every face it is recognised into carries the persistent id 0; the hover once looked the face up by
    /// that id and lit the first face of the body wherever the cursor stood, while a click took the face pointed at.
    /// Reported behaviour: on a project of meshes the highlight stayed on one face, "especially on the two islands".
    fn the_face_under_the_cursor_is_lit_on_a_body_of_a_mesh() {
        let path = format!("{}/qymcad-box-{}.stl", std::env::temp_dir().display(), std::process::id());
        box_stl(&path);
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        build::import(&mut s, &path);
        let _ = std::fs::remove_file(&path);
        s.key(Key::Escape);
        let away = qymcad::pos2(s.canvas().min.x + 4.0, s.canvas().max.y - 4.0);
        s.move_to(away);
        let plain = s.snapshot();
        let (top, side) = (s.face_at([20.0, 15.0, 10.0]), s.face_at([20.0, 0.0, 5.0]));
        s.move_to(top);
        let on_top = s.snapshot();
        s.move_to(side);
        let on_side = s.snapshot();
        assert!(!golden::same(&plain, &on_top), "the cursor stands on the top of the box and nothing on the picture says so");
        assert!(!golden::same(&on_top, &on_side), "the cursor moved from the top of the box to its side and the same face stayed lit");
    }
}

probe! {
    /// WHAT IS TAKEN IS LIT: the picture of a chosen body is not the picture of the same body untouched.
    fn what_is_taken_is_lit() {
        let mut s = a_block();
        let away = qymcad::pos2(s.canvas().min.x + 4.0, s.canvas().max.y - 4.0);
        let at = s.face_at([20.0, 15.0, 10.0]);
        s.move_to(away);
        let plain = s.snapshot();
        s.click(at);
        s.move_to(away);
        assert!(!golden::same(&plain, &s.snapshot()), "the body was taken and the picture is the same as when nothing was");
    }
}

probe! {
    /// A ROW TAKEN IN THE TREE IS LIT ON THE PICTURE: what is chosen on the left is seen on the right.
    fn a_row_taken_in_the_tree_is_lit_on_the_picture() {
        let mut s = two_blocks();
        let away = qymcad::pos2(s.canvas().min.x + 4.0, s.canvas().max.y - 4.0);
        s.move_to(away);
        let plain = s.snapshot();
        let second = s.document().parts.iter().filter(|p| !p.assembly).map(|p| p.name.clone()).next_back().expect("the second part");
        let row: Pos2 = s.find(&second, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {second:?} is not in the tree; on screen: {:?}", s.words())).center();
        s.click(row);
        s.move_to(away);
        assert!(!golden::same(&plain, &s.snapshot()), "the part {second:?} was taken in the tree and nothing on the picture says which one");
    }
}
