//! THE WINDOW LOOKS AS SIGNED: a picture taken without a graphics card, against the one a person looked at.
use qymcad::Session;
use qymcad_acceptance::{build, golden, probe};

probe! {
    /// ONE SCREEN IS ONE PICTURE: the start screen of two starts, taken apart, is the same picture.
    fn one_screen_is_one_picture() {
        let first = Session::start().snapshot();
        let second = Session::start().snapshot();
        assert_eq!(golden::difference(&first, &second).map(|(d, _)| d), Ok(0), "two starts of the program draw two different start screens");
    }
}

probe! {
    /// A CHANGED SCREEN IS A DIFFERENT PICTURE: the settings opened over the start screen are seen.
    fn a_changed_screen_is_a_different_picture() {
        let mut s = Session::start();
        let before = s.snapshot();
        let (windows, settings) = (s.word("menu-windows"), s.word("menu-settings"));
        s.menu(&[&windows, &settings]);
        assert!(!golden::same(&before, &s.snapshot()), "the settings opened and the picture of the window stayed the same");
    }
}

probe! {
    /// THE START SCREEN LOOKS AS SIGNED.
    fn the_start_screen_looks_as_signed() {
        golden::looks_as_signed("start_screen", &Session::start().snapshot());
    }
}

probe! {
    /// A BLOCK LOOKS AS SIGNED: the part, its tree and the body in the 3D view.
    fn a_block_looks_as_signed() {
        let mut s = Session::start();
        build::block(&mut s);
        // the pointer rests on the empty tree: what it hovered last is not what the picture signs
        s.move_to(qymcad::pos2(240.0, 520.0));
        golden::looks_as_signed("block", &s.snapshot());
    }
}
