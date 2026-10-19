//! THE CONTENT OF A WINDOW KEEPS CLEAR OF THE LINE THAT DIVIDES IT FROM ITS SIDE PANEL.
//!
//! Reported behaviour (issue #16): in the settings window the text of a section was pressed against the line
//! between the list of sections and the section itself - "General", "Language", the highlight of "English"
//! all started on the line, with not a pixel between.
//!
//! The cause is how `egui::Panel` behaves INSIDE a window: the panel keeps a margin of its own, then moves the
//! cursor of the window exactly to its outer edge and draws the divider there. Whatever is added next starts
//! on that line unless it brings a margin of its own - a `CentralPanel`, as the parts library does. The
//! settings and the help drew their right side bare.
//!
//! Checked BY GEOMETRY, where the letters and the highlights actually landed, over every window that has a
//! side panel: a margin set up in one place and bypassed by the first `ui.label` outside it would pass a guard
//! over the source. The windows are opened as a person opens them.
use qymcad::{Key, Session};
use qymcad_acceptance::probe;

/// How much room the content must leave between itself and the divider. The side panel keeps 8 points on
/// its side of the line; a little less on the other side is still a gap, nothing at all is not.
const CLEAR: f32 = 6.0;

/// The leftmost captions and filled backgrounds painted to the right of the side panel `panel`, within its height, as
/// distances from the divider - the panel's outer right edge.
fn gaps(s: &mut Session, panel: &str) -> Vec<(f32, String)> {
    let side = s.panel(panel).unwrap_or_else(|| panic!("the panel \"{panel}\" was not drawn"));
    let divider = side.max.x;
    // A shape that starts a point or two left of the line still belongs to the right side: a highlight is
    // expanded around its widget.
    s.painted()
        .into_iter()
        .filter(|(r, text)| r.min.x > divider - 2.0 && r.min.y >= side.min.y && r.max.y <= side.max.y && !text.trim().is_empty())
        .map(|(r, text)| (r.min.x - divider, text))
        .collect()
}

fn assert_clear(name: &str, gaps: Vec<(f32, String)>) {
    assert!(gaps.len() > 3, "{name}: suspiciously little is painted to the right of the side panel: {}", gaps.len());
    let (gap, what) = gaps.into_iter().min_by(|a, b| a.0.total_cmp(&b.0)).expect("checked above");
    assert!(gap >= CLEAR, "{name}: {what:?} is {gap:.1} points from the divider of the side panel, and there must be at least {CLEAR}");
}

probe! {
    /// THE SETTINGS: a section keeps clear of the list of sections.
    fn the_settings_section_keeps_clear_of_the_list_of_sections() {
        let mut s = Session::start();
        let (windows, settings) = (s.word("menu-windows"), s.word("win-settings"));
        s.menu(&[&windows, &settings]);
        assert_clear("the settings", gaps(&mut s, "settings_sections"));
    }
}

probe! {
    /// THE HELP: the article keeps clear of the contents. F1 with nothing in hand opens the contents; the menu says
    /// "Help" twice, the menu and its first item.
    fn the_help_article_keeps_clear_of_the_contents() {
        let mut s = Session::start();
        s.key(Key::F1);
        assert_clear("the help", gaps(&mut s, "help_toc"));
    }
}

probe! {
    /// THE PARTS LIBRARY, which kept its gap already: the parts a search lists keep clear of the catalogue.
    fn the_parts_keep_clear_of_the_catalogue() {
        let mut s = Session::start();
        let (windows, library) = (s.word("menu-windows"), s.word("win-parts-library"));
        s.menu(&[&windows, &library]);
        // a search lists the matching parts on the right; with no category chosen there is one hint and nothing more
        let search = s.word("pl-search");
        s.fill_empty(&search, "e");
        assert_clear("the parts library", gaps(&mut s, "parts_lib_tree"));
    }
}
