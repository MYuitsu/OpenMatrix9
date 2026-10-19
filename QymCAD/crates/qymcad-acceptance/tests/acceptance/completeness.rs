//! EVERYTHING A PERSON CAN REACH HAS A DESCRIPTION, and so a contract that runs: the buttons, the menus, the commands of
//! the search, the articles of help about a function.
use qymcad::Session;
use qymcad_acceptance::completeness::{articles, button_hints, command_names, described, menu_paths};
use qymcad_acceptance::probe;
use qymcad_acceptance::tools::ALL;

/// Both lists at once: what the descriptions name and the window does not have, and what the window has without a
/// description. One of them stopping the other hid how many were missing.
fn judge(what: &str, stale: &[&String], missing: &[&String]) {
    let mut problems = Vec::new();
    if !stale.is_empty() {
        problems.push(format!("{} {what} named by descriptions are not found - gone, renamed, or not read:\n{}", stale.len(), stale.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")));
    }
    if !missing.is_empty() {
        problems.push(format!("{} {what} have no description:\n{}", missing.len(), missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

probe! {
    /// EVERY TOOL BUTTON HAS A DESCRIPTION.
    fn every_button_has_a_description() {
        let s = Session::start();
        let (hints, ..) = described(&s, ALL);
        let found = button_hints();
        let stale: Vec<&String> = hints.iter().filter(|h| !found.contains(h)).collect();
        let missing: Vec<&String> = found.iter().filter(|h| !hints.contains(h)).collect();
        judge("buttons", &stale, &missing);
    }
}

probe! {
    /// EVERY ITEM OF THE MENUS HAS A DESCRIPTION.
    fn every_menu_item_has_a_description() {
        let s = Session::start();
        let (_, menus, ..) = described(&s, ALL);
        let found = menu_paths();
        assert!(found.len() > 10, "the menus were not read: {found:?}");
        let stale: Vec<String> = menus.iter().filter(|m| !found.contains(m)).map(|p| p.join(" > ")).collect();
        let missing: Vec<String> = found.iter().filter(|p| !menus.contains(p)).map(|p| p.join(" > ")).collect();
        judge("menu items", &stale.iter().collect::<Vec<_>>(), &missing.iter().collect::<Vec<_>>());
    }
}

probe! {
    budget = 900;
    /// EVERY COMMAND THE SEARCH FINDS HAS A DESCRIPTION.
    fn every_command_has_a_description() {
        let s = Session::start();
        let (_, _, commands, _) = described(&s, ALL);
        let found = command_names();
        let stale: Vec<&String> = commands.iter().filter(|c| !found.contains(c)).collect();
        let missing: Vec<&String> = found.iter().filter(|c| !commands.contains(c)).collect();
        judge("commands", &stale, &missing);
    }
}

probe! {
    /// EVERY ARTICLE OF HELP ABOUT A FUNCTION HAS A DESCRIPTION.
    fn every_article_has_a_description() {
        let s = Session::start();
        let (.., help) = described(&s, ALL);
        let found = articles(&s.language());
        let stale: Vec<&String> = help.iter().filter(|a| !found.contains(a)).collect();
        let missing: Vec<&String> = found.iter().filter(|a| !help.contains(a)).collect();
        judge("articles", &stale, &missing);
    }
}

probe! {
    /// EVERY TOOL HAS ITS ARTICLE OF HELP: F1 with it in hand, or the index of help, leads a person to what to press and
    /// what comes of it. A description that names no article is a tool the help does not write about.
    fn every_tool_has_an_article() {
        let without: Vec<&str> = ALL.iter().filter(|t| t.help.is_empty()).map(|t| t.id).collect();
        assert!(without.is_empty(), "{} tools have no article of help: {without:?}", without.len());
    }
}
