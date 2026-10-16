//! EVERY TARGET IS FOUND THE WAY A PERSON FINDS IT: a button by its hint, a menu item and a row by their words, a
//! field by its caption, a checkbox by its words, a point of the sketch and a face, an edge and a vertex of a body
//! where the window drew them - brought into view as a person brings them.
use qymcad::{vec2, Kind, Machine, Modifiers, Session, SketchPick};
use qymcad_acceptance::{build, refusal};
use qymcad_acceptance::probe;

probe! {
    /// A BUTTON WITH NO WORDS IS FOUND BY ITS HINT, and pressing it takes the tool; a hint no button has is not found.
    fn a_button_is_found_by_its_hint() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        let xy = s.word("plane-xy-table");
        s.press_word(&xy);
        let two_corners = s.word("opt-rect-2corners");
        assert!(!s.shows(&two_corners), "the rectangle's options are up before the rectangle was taken");
        let hint = s.word("tb-rect-hint");
        s.press_hint(&hint);
        assert!(s.shows(&two_corners), "the button with the rectangle's hint did not take the rectangle; on screen: {:?}", s.words());
        assert_eq!(s.find_hint("a hint no button has"), None, "a hint no button has was found");
    }
}

probe! {
    /// A BUTTON BELOW THE BOTTOM OF A LOW WINDOW IS REACHED WITH THE WHEEL over its panel.
    fn a_button_below_the_window_is_reached_with_the_wheel() {
        let mut s = Session::start_on(Machine { screen: (1280.0, 480.0), ..Machine::default() });
        build::into_the_first_part(&mut s);
        let xy = s.word("plane-xy-table");
        s.press_word(&xy);
        let text_hint = s.word("g-text-hint");
        let hint = s.word("tb-text-hint");
        s.press_hint(&hint);
        assert!(s.shows(&text_hint), "the text tool, below the bottom of a window 480 points high, was not taken; on screen: {:?}", s.words());
    }
}

probe! {
    /// A MENU IS GONE DOWN BY ITS WORDS.
    fn a_menu_is_gone_down_by_its_words() {
        let mut s = Session::start();
        let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
        s.menu(&[&windows, &settings]);
        let autosave = s.word("settings-autosave");
        assert!(s.shows(&autosave), "Windows, Settings did not open the settings; on screen: {:?}", s.words());
    }
}

probe! {
    /// A FIELD IS FOUND BY ITS CAPTION, and what is typed into it is what it holds.
    fn a_field_is_found_by_its_caption_and_filled() {
        let mut s = Session::start();
        let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
        s.menu(&[&windows, &settings]);
        let (autosave, undo) = (s.word("settings-autosave"), s.word("settings-undo-cap"));
        let field = s.field(&autosave);
        assert_eq!((field.kind, field.value.as_str()), (Kind::Number, "180"), "the field under {autosave:?} is not the autosave period: {field:?}");
        assert_eq!(s.field(&undo).value, "40", "the field under {undo:?} is not the number of undo steps");
        s.fill(&autosave, "60").key(qymcad::Key::Enter);
        assert_eq!(s.field(&autosave).value, "60", "60 typed into the autosave period did not arrive");
        assert_eq!(s.field(&undo).value, "40", "typing into one field changed another");
    }
}

probe! {
    /// A CHECKBOX IS TICKED BY ITS WORDS, and says so.
    fn a_checkbox_is_ticked_by_its_words() {
        let mut s = Session::start();
        let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
        s.menu(&[&windows, &settings]);
        let open_last = s.word("settings-open-last");
        let state = |s: &mut Session| s.widgets().into_iter().find(|w| w.kind == Kind::CheckBox && w.label == open_last).and_then(|w| w.checked);
        // ticked at the factory (decided 25.09): a click unticks it
        assert_eq!(state(&mut s), Some(true), "the checkbox {open_last:?} is not ticked at the factory");
        s.toggle(&open_last);
        assert_eq!(state(&mut s), Some(false), "a click on {open_last:?} did not untick it");
        s.toggle(&open_last);
        assert_eq!(state(&mut s), Some(true), "a second click on {open_last:?} did not tick it again");
    }
}

probe! {
    /// POINTS OF THE SKETCH ARE CLICKED WHERE THE WINDOW DREW THEM: the rectangle's corners land where they were
    /// clicked, and what lies under a place is what a click there takes.
    fn a_sketch_is_clicked_where_the_window_drew_it() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        build::rectangle_on_xy(&mut s);
        assert_eq!(s.sketch_under(20.0, 0.0), Some(SketchPick::Line { from: (0.0, 0.0), to: (40.0, 0.0) }), "the bottom side of the rectangle is not under its middle");
        assert_eq!(s.sketch_under(40.0, 15.0), Some(SketchPick::Line { from: (40.0, 0.0), to: (40.0, 30.0) }), "the right side of the rectangle is not under its middle");
        assert_eq!(s.sketch_under(40.0, 30.0), Some(SketchPick::Point { at: (40.0, 30.0) }), "the far corner is not under the place it was clicked");
        assert_eq!(s.sketch_under(20.0, 15.0), Some(SketchPick::Point { at: (20.0, 15.0) }), "the centre of the rectangle is not under its middle");
        assert_eq!(s.sketch_under(10.0, 8.0), None, "the empty inside of the rectangle takes something");
    }
}

probe! {
    /// A PLACE OF THE SKETCH OUT OF VIEW IS BROUGHT INTO IT by dragging the sheet, and clicked there.
    fn a_place_of_the_sketch_out_of_view_is_brought_into_it() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        let xy = s.word("plane-xy-table");
        s.press_word(&xy);
        let canvas = s.canvas();
        let far = (900.0, -700.0);
        let hint = s.word("tb-rect-hint");
        s.press_hint(&hint);
        let a = s.on_sketch(far.0, far.1);
        assert!(canvas.contains(a), "the far corner was not brought onto the canvas {canvas:?}: {a:?}");
        s.click(a).click_on_sketch(far.0 + 40.0, far.1 + 30.0);
        assert_eq!(s.sketch_under(far.0 + 20.0, far.1), Some(SketchPick::Line { from: far, to: (far.0 + 40.0, far.1) }), "a rectangle drawn far away did not land where it was clicked");
    }
}

probe! {
    /// A FACE, AN EDGE AND A VERTEX ARE CLICKED WHERE THE WINDOW DREW THEM, and a point behind the body is refused.
    fn faces_edges_and_vertices_are_found_where_the_window_drew_them() {
        let mut s = Session::start();
        build::block(&mut s);
        let canvas = s.canvas();
        for p in [s.face_at([20.0, 15.0, 10.0]), s.edge_at([20.0, 0.0, 10.0]), s.vertex_at([40.0, 30.0, 10.0])] {
            assert!(canvas.contains(p), "a place on the block is off the canvas {canvas:?}: {p:?}");
        }
        let hidden = refusal(|| {
            s.face_at([20.0, 15.0, 0.0]);
        });
        assert!(hidden.contains("hidden"), "the bottom face, behind the block, was taken as in view: {hidden:?}");
        let wrong = refusal(|| {
            s.edge_at([20.0, 15.0, 10.0]);
        });
        assert!(wrong.contains("no edge"), "the middle of the top face was taken for an edge: {wrong:?}");
    }
}

probe! {
    /// A VERTEX OUT OF THE 3D VIEW IS BROUGHT INTO IT with the gesture that moves the view.
    fn a_vertex_out_of_the_3d_view_is_brought_into_it() {
        let mut s = Session::start();
        build::block(&mut s);
        let canvas = s.canvas();
        let top = s.face_at([20.0, 15.0, 10.0]);
        for _ in 0..12 {
            s.wheel(top, vec2(0.0, 240.0), Modifiers::default());
        }
        let corner = s.vertex_at([0.0, 0.0, 10.0]);
        assert!(canvas.contains(corner), "the corner was not brought onto the canvas {canvas:?}: {corner:?}");
    }
}
