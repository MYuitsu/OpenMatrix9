//! THE PARTS LIBRARY: a part of one's own put into it, found in it by name, and brought back into an assembly.
use qymcad::Session;
use qymcad_acceptance::{build, probe};

/// The name this check gives the part it saves.
const PART: &str = "a-block-of-my-own";

/// A first start holding a 40 x 30 x 10 block, standing in the assembly with the part saved into the library under
/// the name `PART`.
fn a_part_saved_into_the_library() -> Session {
    let mut s = Session::start();
    build::block(&mut s);
    let assembly = s.word("wb-assembly");
    s.press_word_near(&assembly, qymcad::pos2(0.0, 0.0));
    let part = s.document().parts[0].name.clone();
    let row = s.find(&part, qymcad::pos2(0.0, 300.0)).unwrap_or_else(|| panic!("the part {part:?} is not in the tree"));
    s.click_with(row.center(), qymcad::PointerButton::Secondary, qymcad::Modifiers::default());
    let save = s.word("act-save-as-part");
    s.press_word_near(&save, row.center());
    let title = s.word("io-save-as-part");
    assert!(s.shows(&title), "the window for saving a part did not open; on screen: {:?}", s.words());
    let name = s.word("io-name");
    s.fill(&name, PART);
    let go = s.word("io-save");
    s.press_word(&go);
    s
}

/// Open the library and answer whether it offers `what` once the search is given `PART`.
fn the_library_offers(s: &mut Session) -> bool {
    let (windows, library) = (s.word("menu-windows"), s.word("win-parts-library"));
    s.menu(&[&windows, &library]);
    let search = s.word("pl-search");
    s.fill_empty(&search, PART);
    s.words().iter().any(|w| w.contains(PART))
}

probe! {
    /// A PART IS PUT INTO THE LIBRARY and the program says it is there.
    fn a_part_is_put_into_the_library() {
        let mut s = a_part_saved_into_the_library();
        let title = s.word("io-save-as-part");
        assert!(!s.shows(&title), "the window for saving a part is still open after Save: the program says {:?}", s.status());
        assert!(!s.status().is_empty(), "nothing is said about the part that was saved");
    }
}

probe! {
    /// THE PART IS FOUND IN THE LIBRARY BY ITS NAME.
    fn a_part_of_ones_own_is_found_in_the_library() {
        let mut s = a_part_saved_into_the_library();
        assert!(the_library_offers(&mut s), "the part {PART:?} saved into the library is not found by its name; on screen: {:?}", s.words());
    }
}

probe! {
    /// THE PART FOUND IN THE LIBRARY COMES INTO THE ASSEMBLY, holding what it held.
    fn a_part_of_ones_own_comes_out_of_the_library() {
        let mut s = a_part_saved_into_the_library();
        assert!(the_library_offers(&mut s), "the part {PART:?} is not in the library to take");
        let before = s.document().parts.len();
        let insert = s.word("pl-insert");
        s.press_word(&insert);
        let now = s.document().parts;
        assert!(now.len() > before, "nothing came out of the library: the assembly holds {:?} and the program says {:?}", now.iter().map(|p| p.name.clone()).collect::<Vec<_>>(), s.status());
        let bodies: Vec<f64> = s.document().bodies.into_iter().filter(|b| !b.consumed && !b.sheet).map(|b| b.volume).collect();
        assert!(bodies.iter().filter(|v| (**v - 12000.0).abs() < 1.0).count() == 2, "the part that came out of the library does not hold the block of 12000: the assembly holds {bodies:?}");
    }
}
