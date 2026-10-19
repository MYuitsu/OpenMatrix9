//! A COMMAND OF THE PART - its frame, whatever the command: Enter applies, Esc cancels, as its buttons say.
use qymcad::{Key, Session};
use qymcad_acceptance::build;
use qymcad_acceptance::probe;

probe! {
    /// THE Esc LADDER OF A COMMAND: with the focus in its field the first Esc takes the focus out of the field and
    /// leaves the command in hand; the next cancels the command and leaves the document as it was. (With a list of
    /// names open, the list goes first.)
    fn escape_leaves_the_field_then_cancels_the_command() {
        let mut s = Session::start();
        build::into_the_first_part(&mut s);
        build::rectangle_on_xy(&mut s);
        let finish = s.word("wb-finish");
        s.press_word(&finish);
        let before = s.document();
        let extrude = s.word("tb-extrude-hint");
        s.press_hint(&extrude);
        let bar = s.in_hand();
        s.key(Key::Escape);
        assert!(!s.in_hand().is_empty(), "the first Esc, with the focus in the field, put the extrusion down: the field should let go first");
        s.key(Key::Escape);
        assert!(s.in_hand().is_empty(), "the second Esc left the extrusion in hand: {:?} (it was {bar:?})", s.in_hand());
        let after = s.document();
        assert_eq!((after.features, after.bodies), (before.features, before.bodies), "Esc on the extrusion changed the document");
    }
}
