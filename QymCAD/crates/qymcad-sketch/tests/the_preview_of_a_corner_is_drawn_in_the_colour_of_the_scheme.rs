//! THE PREVIEW OF A CORNER IS DRAWN IN THE COLOUR OF THE SCHEME IN HAND.
//!
//! Reported: with the corner tool, holding Shift and naming a second corner, the lines of the preview were not
//! drawn at all. The two colours were kept in the state of the tool (`CornerInput`) and written by the one window
//! that had a scheme to hand - the bar that takes the tool - so every other way of opening the field left them at
//! the transparent default of a `Color32`, and `CornerInput::clear()` (which every applied corner and every mode
//! change calls) took them away again for the rest of the set.
//!
//! So the check here is not that a colour is a colour: it is that the colour is READ FROM THE SCHEME THE DRAWING
//! IS DONE IN, every frame, and that the two states of a corner are told apart by that scheme rather than by a
//! number written into the drawing. A scheme that is put down and picked up again changes the preview with it.
use qymcad_ui_state::SchemeUi;

/// A COLOUR THAT IS DRAWN WITH MUST BE SEEN: transparent, or the same as the background it stands on, is a
/// preview that is not drawn whatever its shape is right.
#[test]
fn the_colour_of_a_corner_preview_is_read_from_the_scheme_in_hand() {
    let scheme = SchemeUi::default();
    let new = qymcad_sketch::corner_colour(&scheme, false);
    let fixed = qymcad_sketch::corner_colour(&scheme, true);
    assert_eq!(new.a(), 255, "the newest corner is drawn transparent, so it is not drawn at all");
    assert_eq!(fixed.a(), 255, "a fixed corner is drawn transparent, so it is not drawn at all");
    assert_ne!(new, fixed, "a corner a further pick can still move and one that is remembered are drawn in one colour: which is which cannot be seen");
    // AND IT IS THE SCHEME'S OWN ROLE, so that a scheme in hand decides it and the code names no colour of its own.
    assert_eq!(new, scheme.pal.preview_corner_new(), "the newest corner is not drawn in the colour the scheme gives that role");
    assert_eq!(fixed, scheme.pal.preview_corner_fixed(), "a fixed corner is not drawn in the colour the scheme gives that role");
}

/// A SCHEME PUT DOWN AND TAKEN UP AGAIN CHANGES THE PREVIEW WITH IT, which is what reading it where the drawing is
/// done buys: the colours are not remembered from the moment the tool was taken.
#[test]
fn another_scheme_draws_the_preview_in_its_own_colours() {
    let dark = SchemeUi::default();
    let mut light = SchemeUi::default();
    light.pal = qymcad_scheme::light();
    assert_ne!(qymcad_sketch::corner_colour(&dark, false), qymcad_sketch::corner_colour(&light, false), "the preview of a corner is drawn in the colour of the scheme the tool was taken in");
    assert_eq!(qymcad_sketch::corner_colour(&light, false), light.pal.preview_corner_new(), "the preview of a corner does not follow the scheme it is drawn in");
}
