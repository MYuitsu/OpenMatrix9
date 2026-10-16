//! THE VIEW AND THE MOUSE: turning, moving and scaling the view under every mouse layout the settings offer, the
//! wheel scaling from the cursor or from the middle, the cube, fitting the view, the flat sheet and the space, and
//! the perspective.
//!
//! What the eye reads off the screen is where the corners of the block are seen - the three corners of one corner
//! of it. Moving the view moves all three the same way, scaling pulls them apart from one place, turning changes
//! the shape they make, and nothing of it touches the document.
use qymcad::{pos2, Key, Modifiers, Pos2, Session, Vec2};
use qymcad_acceptance::mouse::{bare, choose_the_layout, close_the_settings, make, open_the_viewport_settings, with, Hold, LEFT, MIDDLE, RIGHT};
use qymcad_acceptance::{build, probe};

/// THE BLOCK IN VIEW, seen in space and with nothing in hand.
fn a_block_in_view() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    s.key(Key::Escape);
    s
}

/// WHERE THE THREE CORNERS OF THE BLOCK ARE SEEN NOW: the origin, the one 40 along and the one 30 across.
fn corners(s: &mut Session) -> [Pos2; 3] {
    [[0.0, 0.0, 0.0], [40.0, 0.0, 0.0], [0.0, 30.0, 0.0]].map(|p| s.seen_at(p).unwrap_or_else(|| panic!("the corner {p:?} of the block is not on the canvas at all")))
}

/// THE SHAPE THE THREE CORNERS MAKE: the angle at the first between the other two, and how much longer one side is
/// than the other. Moving the view leaves both alone, scaling leaves both alone, turning changes them.
fn shape(c: [Pos2; 3]) -> (f32, f32) {
    let (a, b) = (c[1] - c[0], c[2] - c[0]);
    (a.angle() - b.angle(), a.length() / b.length())
}

/// FRAME THE MODEL AGAIN through the View menu, so what comes next starts from a view the whole block fits in.
fn fit_the_view(s: &mut Session) {
    let (view, fit) = (s.word("menu-view"), s.word("menu-fit-view"));
    s.menu(&[&view, &fit]);
}

/// ONE NOTCH OF THE WHEEL at `at`, the way a mouse sends it: a notch is about fifty points.
fn a_turn_of_the_wheel(s: &mut Session, at: Pos2) {
    s.wheel(at, qymcad::vec2(0.0, 50.0), Modifiers::NONE);
}

/// The middle of the canvas, and a place a little to one side of it.
fn across_the_canvas(s: &mut Session) -> (Pos2, Pos2) {
    let canvas = s.canvas();
    let middle = canvas.center();
    (middle, pos2(middle.x + 160.0, middle.y + 60.0))
}

/// TURNING, MOVING AND SCALING UNDER ONE LAYOUT: each gesture does its own and nothing else, and none of them
/// touches the document.
fn a_layout_moves_the_view(code: &str, turn: Hold, slide: Hold, scale: Option<Hold>) {
    let mut s = a_block_in_view();
    choose_the_layout(&mut s, code);
    let paper = s.document();
    // TURNING: the shape the corners make changes - and if they all move the same way instead, the view was
    // moved rather than turned
    let (from, to) = across_the_canvas(&mut s);
    let stood = corners(&mut s);
    make(&mut s, turn, from, to);
    let now = corners(&mut s);
    let (before, after) = (shape(stood), shape(now));
    let moves: Vec<Vec2> = (0..3).map(|i| now[i] - stood[i]).collect();
    let together = moves.iter().all(|m| (*m - moves[0]).length() < 1.0);
    assert!(
        (after.0 - before.0).abs() > 0.05 || (after.1 / before.1 - 1.0).abs() > 0.05,
        "under {code} the model was turned and it is seen as it was: the corners made {before:?} and make {after:?}; they moved {moves:?}{}",
        if together && moves[0].length() > 1.0 { " - all the same way, so the view was moved instead of turned" } else { "" }
    );
    // MOVING: every corner moves the same way, and by something worth seeing
    let before = corners(&mut s);
    make(&mut s, slide, from, to);
    let after = corners(&mut s);
    let moves: Vec<Vec2> = (0..3).map(|i| after[i] - before[i]).collect();
    assert!(moves[0].length() > 20.0, "under {code} the view was moved across the canvas and the model stayed where it was: it moved {:?}", moves[0]);
    assert!(moves.iter().all(|m| (*m - moves[0]).length() < 1.0), "under {code} the view was moved and the model came apart: the corners moved {moves:?}");
    // SCALING: from a framed view again, the corners are pulled apart and the shape they make is left alone
    fit_the_view(&mut s);
    let before = corners(&mut s);
    match scale {
        Some(hold) => make(&mut s, hold, from, pos2(from.x, from.y - 120.0)),
        None => {
            a_turn_of_the_wheel(&mut s, from);
            a_turn_of_the_wheel(&mut s, from);
        }
    }
    let after = corners(&mut s);
    let (was, now) = ((before[1] - before[0]).length(), (after[1] - after[0]).length());
    assert!(now > was * 1.1, "under {code} the view was scaled up and the model grew from {was} to {now}");
    assert!((shape(after).0 - shape(before).0).abs() < 0.02, "under {code} scaling the view turned it as well: the corners made {:?} and make {:?}", shape(before), shape(after));
    assert!(s.document() == paper, "moving the view about changed the document");
}

probe! {
    /// OURS: a drag turns, Shift and a drag moves, the wheel scales.
    fn the_view_moves_under_qymcad() {
        a_layout_moves_the_view("qymcad", bare(&[LEFT]), with(&[LEFT], Modifiers::SHIFT), None);
    }
}

probe! {
    /// CAD: the middle and the left together turn, the middle alone moves.
    fn the_view_moves_under_cad() {
        a_layout_moves_the_view("cad", bare(&[MIDDLE, LEFT]), bare(&[MIDDLE]), None);
    }
}

probe! {
    /// BLENDER: the middle turns, Shift and the middle moves.
    fn the_view_moves_under_blender() {
        a_layout_moves_the_view("blender", bare(&[MIDDLE]), with(&[MIDDLE], Modifiers::SHIFT), None);
    }
}

probe! {
    /// GESTURE: the left turns, the right moves.
    fn the_view_moves_under_gesture() {
        a_layout_moves_the_view("gesture", bare(&[LEFT]), bare(&[RIGHT]), None);
    }
}

probe! {
    /// MAYA: Alt and the left turn, Alt and the middle move.
    fn the_view_moves_under_maya() {
        a_layout_moves_the_view("maya", with(&[LEFT], Modifiers::ALT), with(&[MIDDLE], Modifiers::ALT), None);
    }
}

probe! {
    /// OPEN CASCADE: Ctrl and the right turn, the middle alone moves, Ctrl and the left scale.
    fn the_view_moves_under_opencascade() {
        let ctrl = Modifiers { ctrl: true, command: true, ..Modifiers::NONE };
        a_layout_moves_the_view("opencascade", with(&[RIGHT], ctrl), bare(&[MIDDLE]), Some(with(&[LEFT], ctrl)));
    }
}

probe! {
    /// THE LAYOUT OF THAT NAME: the left turns, the middle moves - the settings describe it under its own line.
    fn the_view_moves_under_openinventor() {
        a_layout_moves_the_view("openinventor", bare(&[LEFT]), bare(&[MIDDLE]), None);
    }
}

probe! {
    /// OPENSCAD: the left turns, the right moves.
    fn the_view_moves_under_openscad() {
        a_layout_moves_the_view("openscad", bare(&[LEFT]), bare(&[RIGHT]), None);
    }
}

probe! {
    /// REVIT: Shift and the middle turn, the middle alone moves.
    fn the_view_moves_under_revit() {
        a_layout_moves_the_view("revit", with(&[MIDDLE], Modifiers::SHIFT), bare(&[MIDDLE]), None);
    }
}

probe! {
    /// TINKERCAD: the right turns, the middle moves.
    fn the_view_moves_under_tinkercad() {
        a_layout_moves_the_view("tinkercad", bare(&[RIGHT]), bare(&[MIDDLE]), None);
    }
}

probe! {
    /// THE TOUCHPAD: no button at all - Alt and a movement turn, Shift and a movement move, Ctrl with Shift scales.
    /// There is no wheel to scale with.
    fn the_view_moves_under_touchpad() {
        let ctrl_shift = Modifiers { ctrl: true, shift: true, command: true, ..Modifiers::NONE };
        a_layout_moves_the_view("touchpad", with(&[], Modifiers::ALT), with(&[], Modifiers::SHIFT), Some(with(&[], ctrl_shift)));
    }
}

/// THE WHEEL SCALES FROM WHERE THE SETTING SAYS: the place named by `key` is chosen, and the world point under the
/// cursor - or the one in the middle of the view - stays where it was.
fn the_wheel_scales_from(key: &str, at_the_cursor: bool) {
    let mut s = a_block_in_view();
    open_the_viewport_settings(&mut s);
    let choice = s.word(key);
    let caption = s.word("settings-zoom-at");
    let near = s.find(&caption, pos2(640.0, 300.0)).unwrap_or_else(|| panic!("the settings do not say where the wheel scales from"));
    s.press_word_near(&choice, near.center());
    close_the_settings(&mut s);
    // the corner 40 along is put under the cursor, the middle of the canvas holds whatever is there
    let corner = [40.0, 0.0, 0.0];
    let at = s.seen_at(corner).expect("the corner of the block is on the canvas");
    let middle = s.canvas().center();
    let before = corners(&mut s);
    a_turn_of_the_wheel(&mut s, at);
    let after = corners(&mut s);
    assert!((after[1] - after[0]).length() > (before[1] - before[0]).length() * 1.1, "the wheel was turned and the model did not grow");
    // what the scaling was made about stands still: the point under it is seen in the same place
    let moved = if at_the_cursor {
        (s.seen_at(corner).expect("the corner after the wheel") - at).length()
    } else {
        // the middle holds: the place a point of the world was seen at scales about the middle, so the corner
        // seen before must now stand where the scaling about the middle puts it
        let k = (after[1] - after[0]).length() / (before[1] - before[0]).length();
        (s.seen_at(corner).expect("the corner after the wheel") - (middle + (at - middle) * k)).length()
    };
    assert!(moved < 2.0, "the wheel scales {}, and the place it was scaled about moved {moved} away", if at_the_cursor { "from the cursor" } else { "from the middle of the view" });
}

probe! {
    /// THE WHEEL SCALES FROM THE CURSOR: what stood under it stays under it.
    fn the_wheel_scales_from_the_cursor() {
        the_wheel_scales_from("settings-zoom-at-cursor", true);
    }
}

probe! {
    /// THE WHEEL SCALES FROM THE MIDDLE OF THE VIEW when that is what is asked for.
    fn the_wheel_scales_from_the_middle() {
        the_wheel_scales_from("settings-zoom-at-centre", false);
    }
}

probe! {
    /// THE CUBE TURNS THE VIEW TO A NAMED SIDE: a click on TOP looks down on the table, where the height of the
    /// block cannot be seen at all and the 40 by 30 of it is seen whole.
    fn the_cube_turns_the_view_to_a_named_side() {
        let mut s = a_block_in_view();
        let top = s.word("view-top");
        let at = s.cube_side(&top).unwrap_or_else(|| panic!("the navigation cube has no side that reads {top:?}"));
        s.click(at);
        let (o, up) = (s.seen_at([0.0, 0.0, 0.0]).expect("the corner of the block"), s.seen_at([0.0, 0.0, 10.0]).expect("the corner above it"));
        assert!((up - o).length() < 1.0, "the view was turned to look down and the 10 mm of height is still seen: {} px of it", (up - o).length());
        let c = corners(&mut s);
        let (along, across) = ((c[1] - c[0]).length(), (c[2] - c[0]).length());
        assert!(((along / across) - 40.0 / 30.0).abs() < 0.02, "looking down on a 40 by 30 block it is seen {along} by {across}");
    }
}

probe! {
    /// THE CUBE TURNS THE VIEW TO A CORNER, AND WHAT WAS HIDDEN IS SEEN: the bottom edge on the left of the block is
    /// behind it as the view opens, and a click on the corner of the cube between front, left and top brings it out.
    /// A corner on the far side of the cube is out of reach until the cube is turned.
    fn the_cube_turns_the_view_to_a_corner() {
        let mut s = a_block_in_view();
        let hidden = [0.0, 15.0, 0.0];
        assert!(!s.sees(hidden), "the bottom edge on the left is seen as the view opens; the check has nothing to bring out");
        let at = s.cube_toward([-1, -1, 1]).unwrap_or_else(|| panic!("the corner of the cube between front, left and top is out of reach"));
        s.click(at);
        assert!(s.sees(hidden), "the view was turned to the corner between front, left and top, and the bottom edge on the left is still hidden");
        assert!(s.cube_toward([1, 1, -1]).is_none(), "the corner of the cube behind it, between back, right and bottom, is offered for a click");
        assert!(s.sees([0.0, 30.0, 5.0]) && !s.sees([40.0, 30.0, 5.0]), "from the corner between front, left and top the back left edge is seen and the back right one is not");
    }
}

probe! {
    /// FITTING THE VIEW BRINGS THE WHOLE MODEL INTO IT, however far the view was scaled in before.
    fn fitting_the_view_brings_the_whole_model_into_it() {
        let mut s = a_block_in_view();
        let middle = s.canvas().center();
        for _ in 0..6 {
            a_turn_of_the_wheel(&mut s, middle);
        }
        let all_in = |s: &mut Session| [[0.0, 0.0, 0.0], [40.0, 30.0, 10.0]].iter().all(|p| s.seen_at(*p).is_some());
        assert!(!all_in(&mut s), "the view was scaled in six turns of the wheel and the whole block still fits in it; there is nothing to fit");
        fit_the_view(&mut s);
        let seen: Vec<Option<Pos2>> = [[0.0, 0.0, 0.0], [40.0, 0.0, 0.0], [0.0, 30.0, 0.0], [40.0, 30.0, 10.0]].iter().map(|p| s.seen_at(*p)).collect();
        assert!(seen.iter().all(|p| p.is_some()), "the view was fitted and the block does not fit in it: its corners are seen at {seen:?}");
        let canvas = s.canvas();
        let across = (seen[1].unwrap() - seen[0].unwrap()).length();
        assert!(across > canvas.width() * 0.2, "the view was fitted and the block of 40 is {across} px on a canvas of {} - it sits in a corner of it", canvas.width());
    }
}

probe! {
    /// THE FLAT SHEET AND THE SPACE SWAP BY THE VIEW MENU, and the model is seen again when the space comes back.
    fn the_flat_sheet_and_the_space_swap() {
        let mut s = a_block_in_view();
        assert!(!s.flat(), "the model is shown in space to begin with");
        let before = corners(&mut s);
        let (view, orbit) = (s.word("menu-view"), s.word("menu-orbit3d"));
        s.menu(&[&view, &orbit]);
        assert!(s.flat(), "the space was turned off in the View menu and the canvas is still the space");
        s.menu(&[&view, &orbit]);
        assert!(!s.flat(), "the space was turned back on and the canvas is still flat");
        let after = corners(&mut s);
        assert!((0..3).all(|i| (after[i] - before[i]).length() < 1.0), "the space came back and the model is seen elsewhere: {before:?} became {after:?}");
    }
}

probe! {
    /// THE PERSPECTIVE MAKES THE FAR EDGE SHORTER than the near one of the same length; the ortho projection draws
    /// them equal.
    fn the_perspective_makes_the_far_edge_shorter() {
        let mut s = a_block_in_view();
        let edges = |s: &mut Session| {
            let mut p = |x: f64, y: f64| s.seen_at([x, y, 10.0]).unwrap_or_else(|| panic!("the corner ({x}, {y}) of the top of the block is not on the canvas"));
            let (near, far) = ((p(40.0, 0.0) - p(0.0, 0.0)).length(), (p(40.0, 30.0) - p(0.0, 30.0)).length());
            near / far
        };
        let flat_on = edges(&mut s);
        assert!((flat_on - 1.0).abs() < 0.005, "the ortho projection draws two edges of 40 as {flat_on} of each other");
        open_the_viewport_settings(&mut s);
        let persp = s.word("settings-projection-persp");
        s.press_word(&persp);
        close_the_settings(&mut s);
        let with_depth = edges(&mut s);
        assert!((with_depth - 1.0).abs() > 0.02, "the perspective was turned on and the near edge of 40 is {with_depth} of the far one - the picture has no depth in it");
    }
}

/// THE VIEW TURNS ABOUT WHAT THE SETTING SAYS: with "the point under the cursor" chosen in the list of the settings, a
/// drag that turns the view keeps the point of the block it began on in its place; with the middle of the view, the
/// factory choice, that point goes round with the rest.
fn a_turn_from_a_corner(choose_the_pointer: bool) -> f32 {
    let mut s = a_block_in_view();
    if choose_the_pointer {
        open_the_viewport_settings(&mut s);
        let (now, pointer) = (s.word("settings-orbit-about-centre"), s.word("settings-orbit-about-pointer"));
        let caption = s.word("settings-orbit-about");
        let near = s.find(&caption, pos2(640.0, 300.0)).unwrap_or_else(|| panic!("the settings do not say what the view turns about")).center();
        s.press_word_near(&now, near);
        s.press_word_near(&pointer, near);
        assert!(s.shows(&pointer) && !s.shows(&now), "\"{pointer}\" was chosen and the list does not show it");
        close_the_settings(&mut s);
    }
    // a point of the top of the block near its corner, away from the middle of the view
    let corner = [4.0, 4.0, 10.0];
    let at = s.face_at(corner);
    s.drag(at, at + qymcad::vec2(80.0, 30.0), qymcad::PointerButton::Primary, Modifiers::NONE);
    (s.seen_at(corner).expect("the corner after the turn") - at).length()
}

probe! {
    /// THE VIEW TURNS ABOUT THE POINT UNDER THE CURSOR when that is chosen: the point the turn began on stays put.
    fn the_view_turns_about_the_point_under_the_cursor() {
        let moved = a_turn_from_a_corner(true);
        assert!(moved < 1.0, "the view turns about the point under the cursor, and that point moved {moved} away");
    }
}

probe! {
    /// THE VIEW TURNS ABOUT ITS MIDDLE by the factory setting: the point the turn began on goes round with the rest.
    fn by_default_the_view_turns_about_its_middle() {
        let moved = a_turn_from_a_corner(false);
        assert!(moved > 5.0, "the view turns about its middle, and the point the turn began on stayed put ({moved})");
    }
}
