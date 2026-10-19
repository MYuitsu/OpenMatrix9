//! THE MENU BAR IS WALKED BY HOVER once it has been opened, in the program's own window.
//!
//! Reported behaviour: "when I click a menu and move the mouse to the next menu, it does not expand". The first menu
//! of the bar opens by a click; while it is open the pointer coming to another title of the bar opens that one in its
//! place; a click outside ends the walk. The menus are told apart by an item of each: "Open project" lies in File,
//! "Undo" in Edit.
use qymcad::{pos2, Session};
use qymcad_acceptance::probe;

probe! {
    /// AN OPEN MENU MOVES TO THE TITLE UNDER THE POINTER, back again, and a click outside ends the walk.
    fn an_open_menu_moves_to_the_title_under_the_pointer_and_a_click_outside_ends_it() {
        let mut s = Session::start();
        let (file, edit) = (s.word("menu-file"), s.word("menu-edit"));
        let (open, undo) = (s.word("file-open"), s.word("menu-undo"));
        let title = |s: &mut Session, word: &str| s.find(word, pos2(0.0, 0.0)).unwrap_or_else(|| panic!("the title {word:?} is not on the bar; on screen: {:?}", s.words())).center();
        let (file_at, edit_at) = (title(&mut s, &file), title(&mut s, &edit));

        s.click(file_at);
        assert!(s.shows(&open) && !s.shows(&undo), "a click on {file:?} opens it alone; on screen: {:?}", s.words());
        s.move_to(edit_at);
        assert!(s.shows(&undo), "with {file:?} open the pointer came to {edit:?} and {edit:?} did not open; on screen: {:?}", s.words());
        assert!(!s.shows(&open), "{edit:?} opened by hover and {file:?} stayed open beside it; on screen: {:?}", s.words());
        s.move_to(file_at);
        assert!(s.shows(&open) && !s.shows(&undo), "the pointer came back to {file:?} and the menu did not follow it; on screen: {:?}", s.words());

        // a click on the empty canvas closes the menu, and hover over the bar opens nothing after it
        let empty = s.canvas().center();
        s.click(empty);
        assert!(!s.shows(&open), "a click outside left {file:?} open; on screen: {:?}", s.words());
        s.move_to(edit_at);
        assert!(!s.shows(&undo), "after a click outside the pointer over {edit:?} opened it without a click; on screen: {:?}", s.words());
    }
}
