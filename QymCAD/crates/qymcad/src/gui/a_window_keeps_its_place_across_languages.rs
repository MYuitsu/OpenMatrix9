//! A WINDOW KEEPS ITS PLACE WHEN THE LANGUAGE CHANGES.
//!
//! egui keeps the place, size and folding of a window under the window's identity, and `Window::new` takes that
//! identity from the title. Reported behaviour: after the language was changed in Settings, the Settings window
//! jumped to a new place in the same frame as the click, the other windows the next time they opened, and the
//! earlier language brought every one of them back. A translated title is a new identity with nothing stored under
//! it, so egui placed the window anew; the old place stayed behind under the old title.
//!
//! The place is read as a screen reader is told it: the rectangle of the window itself, whatever its title says.
#[cfg(test)]
mod tests {
    use super::super::hand::Hand;
    use super::super::App;

    /// Where the Settings window stands, titled in the language now chosen: its top-left corner, the next frame drawn.
    fn settings_place(hand: &mut Hand) -> egui::Pos2 {
        let title = crate::i18n::tr("win-settings");
        hand.window_titled(&title).unwrap_or_else(|| panic!("the Settings window, titled {title:?}, is on screen")).min
    }

    /// SETTINGS OPENED FROM THE MENU, DRAGGED BY ITS TITLE, THEN ANOTHER LANGUAGE CHOSEN in it: the window stays where
    /// the person put it.
    #[test]
    fn the_settings_window_stays_where_it_was_put_when_the_language_changes() {
        let mut app = App::default();
        let mut hand = Hand::new(&mut app);
        let first = crate::i18n::language();
        let origin = egui::pos2(0.0, 0.0);
        assert!(hand.press_word(&crate::i18n::tr("menu-windows"), origin), "the menu bar has a Windows menu");
        assert!(hand.press_word(&crate::i18n::tr("win-settings"), origin), "the Windows menu has Settings");
        let title = hand.written_at(&crate::i18n::tr("win-settings")).expect("Settings opens with its title");
        let opened = settings_place(&mut hand);

        let shift = egui::vec2(240.0, 140.0);
        hand.drag_screen(title.center(), title.center() + shift);
        let put = settings_place(&mut hand);
        // GUARD: the drag carried the window far from where egui places it, otherwise a window placed anew would
        // land on the person's place and pass
        assert!((put - opened).length() > 200.0, "GUARD: the drag must carry the window away, and it went from {opened:?} to {put:?}");

        let (code, name) = crate::i18n::available().into_iter().find(|(code, _)| *code != first).expect("a second language in the catalogue");
        assert!(hand.press_word(&name, put + egui::vec2(200.0, 80.0)), "the General section lists {name}");
        assert_eq!(crate::i18n::language(), code, "GUARD: a click on {name} switches the interface to it");
        let after = settings_place(&mut hand);
        let later = settings_place(&mut hand);
        crate::i18n::set_language(&first);
        assert!((after - put).length() < 0.5, "the Settings window must stay where it was put when the language changes to {code}; it went from {put:?} to {after:?}");
        assert!((later - put).length() < 0.5, "and stay there in the frames after; it went from {put:?} to {later:?}");
    }
}
