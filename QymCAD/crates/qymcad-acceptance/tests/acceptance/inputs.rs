//! EVERY INPUT A PERSON HAS REACHES THE PROGRAM: the buttons of the mouse, the wheel, a drag, the keys, typing,
//! the clipboard, the file chooser, the size of the window, the machine the program starts on.
//!
//! Each input is shown to arrive by what the program does with it on screen, not by what it did inside.
use qymcad::{pos2, vec2, Chooser, Key, Machine, Modifiers, PointerButton, Pos2, Rect, Session};
use qymcad_acceptance::{build, probe};

/// Where the catalogue word `key` is written on screen, nearest to `near`.
fn place(s: &mut Session, key: &str, near: Pos2) -> Rect {
    let word = s.word(key);
    match s.find(&word, near) {
        Some(r) => r,
        None => panic!("{word:?} is not on screen; on screen: {:?}", s.words()),
    }
}

/// The top left corner of the window, where the menu bar starts.
const CORNER: Pos2 = pos2(0.0, 0.0);

/// The command search, open and holding the keyboard, with `text` typed into it.
fn search_with(s: &mut Session, text: &str) -> Rect {
    s.chord(Modifiers::COMMAND, Key::K).type_text(text);
    s.find(text, CORNER).unwrap_or_else(|| panic!("the typed {text:?} is not in the search field; on screen: {:?}", s.words()))
}

probe! {
    /// A CLICK OPENS A MENU: its items come up.
    fn a_click_opens_a_menu() {
        let mut s = Session::start();
        let open = s.word("file-open");
        assert!(!s.shows(&open), "the File menu is open before anything was clicked");
        let file = place(&mut s, "menu-file", CORNER);
        s.click(file.center());
        assert!(s.shows(&open), "a click on File opened nothing; on screen: {:?}", s.words());
    }
}

probe! {
    /// A PRESS ALONE IS NOT A CLICK; the release that follows it is.
    fn a_press_is_not_a_click_until_it_is_released() {
        let mut s = Session::start();
        let open = s.word("file-open");
        let file = place(&mut s, "menu-file", CORNER);
        s.move_to(file.center()).press(PointerButton::Primary);
        assert!(!s.shows(&open), "the menu opened on the press, before the button was let go");
        s.release(PointerButton::Primary);
        assert!(s.shows(&open), "a press and a release on File opened nothing");
    }
}

probe! {
    /// THE RIGHT BUTTON OPENS WHAT BELONGS TO A ROW: the menu of a part in the tree.
    fn the_right_button_opens_the_menu_of_a_row() {
        let mut s = Session::start();
        build::a_part_in_the_assembly(&mut s);
        let rename = s.word("act-rename");
        let part = s.find("Part 1", pos2(0.0, 300.0)).unwrap_or_else(|| panic!("no part in the tree; on screen: {:?}", s.words()));
        s.click_with(part.center(), PointerButton::Primary, Modifiers::default());
        assert!(!s.shows(&rename), "a left click opened the menu of the row");
        s.click_with(part.center(), PointerButton::Secondary, Modifiers::default());
        assert!(s.shows(&rename), "a right click on a part in the tree opened no menu; on screen: {:?}", s.words());
    }
}

probe! {
    /// KEYS REACH THE PROGRAM: Ctrl+K opens the command search and Esc closes it.
    fn a_chord_opens_the_command_search_and_a_key_closes_it() {
        let mut s = Session::start();
        let hint = s.word("cs-hint");
        s.key(Key::Escape); // the start screen
        assert!(!s.shows(&hint), "the command search is open before it was asked for");
        s.chord(Modifiers::COMMAND, Key::K);
        assert!(s.shows(&hint), "Ctrl+K opened no command search; on screen: {:?}", s.words());
        s.key(Key::Escape);
        assert!(!s.shows(&hint), "Esc left the command search open");
    }
}

probe! {
    /// TYPING AND PASTING REACH THE FIELD that holds the keyboard, and the program answers them.
    fn typed_and_pasted_text_reach_the_field() {
        let extrude = |s: &Session| s.word("cmd-extrude");
        let mut s = Session::start();
        search_with(&mut s, "extr");
        let word = extrude(&s);
        assert!(s.shows(&word), "typing into the command search found nothing; on screen: {:?}", s.words());

        let mut s = Session::start();
        s.chord(Modifiers::COMMAND, Key::K).paste("extr");
        assert!(s.shows("extr") && s.shows(&word), "text pasted into the command search did not arrive; on screen: {:?}", s.words());
    }
}

probe! {
    /// COPY AND CUT PUT WHAT IS SELECTED ON THE CLIPBOARD; cut takes it out of the field.
    fn copy_and_cut_reach_the_clipboard() {
        let mut s = Session::start();
        search_with(&mut s, "extrude");
        s.chord(Modifiers::COMMAND, Key::A).copy();
        assert_eq!(s.clipboard(), ["extrude"], "Ctrl+A and copy in the command search");
        s.cut();
        assert_eq!(s.clipboard(), ["extrude", "extrude"], "cut in the command search");
        assert!(!s.shows("extrude"), "cut left the text in the field");
    }
}

probe! {
    /// A DOUBLE CLICK SELECTS THE WORD UNDER IT: what is typed next replaces that word alone.
    fn a_double_click_selects_a_word() {
        let mut s = Session::start();
        let field = search_with(&mut s, "alpha beta");
        s.double_click(pos2(field.max.x - 6.0, field.center().y)).type_text("gamma");
        assert!(s.shows("alpha gamma"), "a double click on the second word did not select it; on screen: {:?}", s.words());
    }
}

probe! {
    /// A MODIFIER HELD FOR A CLICK changes the click: Shift extends the selection to where it clicks.
    fn shift_held_for_a_click_extends_the_selection() {
        let mut s = Session::start();
        let field = search_with(&mut s, "alpha beta");
        let (start, end) = (pos2(field.min.x + 1.0, field.center().y), pos2(field.max.x + 2.0, field.center().y));
        s.click(start).click_with(end, PointerButton::Primary, Modifiers::SHIFT).type_text("x");
        assert!(s.shows("x") && !s.shows("alpha beta") && !s.shows("xalpha beta"), "Shift and a click did not select the text; on screen: {:?}", s.words());

        // and held across gestures until let go
        let mut s = Session::start();
        let field = search_with(&mut s, "alpha beta");
        s.click(pos2(field.min.x + 1.0, field.center().y)).hold(Modifiers::SHIFT).click(pos2(field.max.x + 2.0, field.center().y)).let_go().type_text("y");
        assert!(s.shows("y") && !s.shows("alpha beta"), "Shift held over the click did not select the text; on screen: {:?}", s.words());
    }
}

probe! {
    /// A DRAG CARRIES WHAT IT TOOK: a window pulled by its title goes where it is pulled.
    fn a_drag_moves_a_window_by_its_title() {
        let mut s = Session::start();
        let windows = place(&mut s, "menu-windows", CORNER);
        s.click(windows.center());
        let settings = place(&mut s, "menu-settings", windows.center());
        s.click(settings.center());
        let title = place(&mut s, "win-settings", pos2(640.0, 0.0));
        let by = vec2(120.0, 80.0);
        s.drag(title.center(), title.center() + by, PointerButton::Primary, Modifiers::default());
        let moved = place(&mut s, "win-settings", title.center() + by);
        assert!((moved.center() - (title.center() + by)).length() < 8.0, "the window did not follow the drag: title from {title:?} by {by:?} came to {moved:?}");
    }
}

probe! {
    /// THE WHEEL SCROLLS what is under the pointer: the settings, taller than a low window, move up.
    fn the_wheel_scrolls_what_is_under_it() {
        let mut s = Session::start_on(Machine { screen: (1280.0, 420.0), ..Machine::default() });
        let windows = place(&mut s, "menu-windows", CORNER);
        s.click(windows.center());
        let settings = place(&mut s, "menu-settings", windows.center());
        s.click(settings.center());
        let row = place(&mut s, "settings-open-last", pos2(640.0, 400.0));
        s.wheel(row.center(), vec2(0.0, -120.0), Modifiers::default());
        let after = s.find(&s.word("settings-open-last"), row.center());
        assert!(after.is_none_or(|r| r.center().y < row.center().y - 50.0), "the wheel over the settings did not scroll them: the row stood at {row:?} and stands at {after:?}");
    }
}

probe! {
    /// A BIGGER WINDOW IS LAID OUT AGAIN: the panel on the right goes to the new right edge.
    fn a_resized_window_is_laid_out_again() {
        let mut s = Session::start();
        let before = place(&mut s, "props-title", pos2(1280.0, 0.0));
        s.resize(vec2(1600.0, 900.0));
        let after = place(&mut s, "props-title", pos2(1600.0, 0.0));
        assert!((after.center().x - before.center().x - 320.0).abs() < 2.0, "the panel on the right did not move with the window growing by 320 points: from {before:?} to {after:?}");
    }
}

/// A sample project of the repository.
const SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/Filter-v2.qcad");

/// File, Open project: the chooser comes up, past the question about unsaved work if the program asks it.
fn ask_to_open_a_project(s: &mut Session) {
    let file = place(s, "menu-file", CORNER);
    s.click(file.center());
    let open = place(s, "file-open", file.center());
    s.click(open.center());
    let dont_save = s.word("nav-dont-save");
    if let Some(button) = s.find(&dont_save, CORNER) {
        s.click(button.center());
    }
}

probe! {
    /// THE FILE CHOOSER IS ANSWERED as a person answers it, and no chooser of the system comes up.
    fn a_file_chooser_is_answered_and_cancelled() {
        let mut s = Session::start();
        let title = s.title();
        ask_to_open_a_project(&mut s);
        assert_eq!(s.chooser(), Some(Chooser::Open), "Open project put up no chooser for an existing file; on screen: {:?}", s.words());
        s.cancel_file();
        assert_eq!(s.chooser(), None, "a cancelled chooser is still waiting");
        assert_eq!(s.title(), title, "cancelling the chooser changed the document");

        ask_to_open_a_project(&mut s);
        s.answer_file(SAMPLE);
        assert_eq!(s.chooser(), None, "an answered chooser is still waiting");
        assert!(s.title().contains("Filter-v2"), "the chosen project did not open: the title is {:?}", s.title());
    }
}

probe! {
    /// A PROGRAM OF THE SYSTEM IS ASKED FOR, and not started: the folder of the settings goes to the file manager.
    fn the_system_is_asked_to_show_a_folder() {
        let mut s = Session::start();
        let windows = place(&mut s, "menu-windows", CORNER);
        s.click(windows.center());
        let settings = place(&mut s, "menu-settings", windows.center());
        s.click(settings.center());
        let folder = place(&mut s, "settings-open-folder", CORNER);
        s.click(folder.center());
        let started = s.started();
        assert_eq!(started.len(), 1, "Open the folder asked the system for {started:?}");
        assert!(started[0].1.iter().any(|a| a.contains("qymcad-sessions")), "the folder shown is not the program's own: {started:?}");
    }
}

probe! {
    /// THE MACHINE'S LANGUAGE is the language of a first start.
    fn a_first_start_speaks_the_language_of_the_machine() {
        let english = Session::start().word("start-title");
        let mut s = Session::start_on(Machine { locale: "ru-RU".into(), ..Machine::default() });
        let russian = s.word("start-title");
        assert_ne!(russian, english, "a machine set to Russian starts the program in English");
        assert!(s.shows(&russian), "the start screen does not say {russian:?}; on screen: {:?}", s.words());
    }
}

probe! {
    /// WHAT THE PROGRAM KEEPS OUTLIVES IT: a project opened before closing is among the recent ones at the next start.
    fn what_the_program_keeps_is_there_at_the_next_start() {
        let mut s = Session::start();
        ask_to_open_a_project(&mut s);
        s.answer_file(SAMPLE);
        let kept = match s.quit() {
            Ok(kept) => kept,
            // the question on closing is answered as a person answers it; whether it should have been asked is not
            // what this check is about
            Err(mut s) => {
                let dont_save = place(&mut s, "nav-dont-save", CORNER);
                s.click(dont_save.center());
                assert!(s.closed(), "Don't save on closing left the window open");
                s.quit().unwrap_or_else(|_| panic!("the window closed by the program would not give what it keeps"))
            }
        };
        // the project of the last run opens by itself at the next start (on by default, decided 25.09)
        let mut s = Session::start_on(Machine { kept, ..Machine::default() });
        let path = s.document().path;
        assert!(path.as_deref().is_some_and(|p| p.contains("Filter-v2")), "the project opened in the last run is not open at the next start: {path:?}; on screen: {:?}", s.words());
    }
}
