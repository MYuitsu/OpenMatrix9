use openmatrix9_rust::{
    menu::{MenuCatalog, MenuKind},
    state::UiState,
};
const REFERENCE: &str = include_str!("../../Resources/menu/MainMenu.ini");

#[test]
fn reference_catalog_has_18_groups_and_11_quick_icons() {
    let c = MenuCatalog::parse(REFERENCE).unwrap();
    assert_eq!(c.groups.len(), 18);
    assert_eq!(c.quick_icons.len(), 11);
    assert_eq!(c.groups[0].title, "File");
    assert_eq!(c.groups[17].title, "Render");
    assert_eq!(c.groups[4].kind, MenuKind::Custom);
    assert_eq!(c.groups[5].kind, MenuKind::Reset);
    assert_eq!(c.groups[0].icons[0], "FileNew");
    assert_eq!(c.groups[0].icons[12], "FilePrint");
    assert_eq!(c.quick_icons[0], "TopIconDuplicate");
    assert_eq!(c.quick_icons[10], "TopIconRingRail");
}

#[test]
fn bom_crlf_and_unicode_titles_are_preserved() {
    let input = format!(
        "\u{feff}{}",
        REFERENCE
            .replace("Name=File", "Name=Tệp")
            .replace('\n', "\r\n")
    );
    assert_eq!(MenuCatalog::parse(&input).unwrap().groups[0].title, "Tệp");
}

#[test]
fn malformed_catalog_has_section_and_key_diagnostics() {
    for (input, section, key) in [
        (
            REFERENCE.replace("MenuCount=18", "MenuCount=18\nMenuCount=18"),
            "Settings",
            "MenuCount",
        ),
        (
            REFERENCE.replace("MenuCount=18", "MenuCount=19"),
            "Menu19",
            "Name",
        ),
        (
            REFERENCE.replace("MenuCount=18", "MenuCount=17"),
            "Settings",
            "MenuCount",
        ),
        (
            REFERENCE.replace("IconCount=13", "IconCount=14"),
            "Menu1",
            "Icon14",
        ),
        (
            REFERENCE.replace("IconCount=13", "IconCount=12"),
            "Menu1",
            "IconCount",
        ),
        (
            REFERENCE.replace("Icon1=FileNew", "Icon1="),
            "Menu1",
            "Icon1",
        ),
        (
            REFERENCE.replace("Type=Custom", "Type=Unknown"),
            "Menu5",
            "Type",
        ),
    ] {
        let error = MenuCatalog::parse(&input).unwrap_err().to_string();
        assert!(error.contains(section) && error.contains(key), "{error}");
    }
}

#[test]
fn section_visibility_and_collapse_are_independent_and_resettable() {
    let mut s = UiState::new();
    assert!(s.select_group(17, 18));
    assert!(!s.select_group(18, 18));
    assert!(s.set_section(2, false, true));
    assert!(s.sections[0].visible && !s.sections[0].collapsed);
    assert!(!s.set_section(7, false, false));
    s.reset();
    assert_eq!(s.selected_group, 0);
    assert!(s.sections.iter().all(|v| v.visible && !v.collapsed));
}

#[test]
fn history_is_successful_only_latest_first_and_bounded() {
    let mut s = UiState::new();
    for i in 0..21 {
        s.record_execution(i, true);
    }
    s.record_execution(99, false);
    assert_eq!(s.history.len(), 20);
    assert_eq!(s.history[0], 20);
    assert_eq!(s.history[19], 1);
    s.reset();
    assert_eq!(s.history.len(), 20, "UI reset preserves execution history");
}
