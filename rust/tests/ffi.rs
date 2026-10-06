use openmatrix9_rust::ffi::*;
use std::{ffi::CStr, sync::Mutex};
static LOCK: Mutex<()> = Mutex::new(());
#[test]
fn workspace_commands_are_supported_without_enabling_subd_selection() {
    let _lock = LOCK.lock().unwrap();
    for icon in [
        "SelectAll",
        "SelectNone",
        "Delete",
        "ViewLeft",
        "ViewRear",
        "ViewBottom",
        "ViewIsometric",
        "ViewZoomZoomSelected",
    ] {
        let index = (0..om9_command_count()).find(|&i| {
            unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes()
        });
        assert!(index.is_some(), "missing workspace command {icon}");
        assert!(
            om9_command_is_active(index.unwrap()),
            "unsupported workspace command {icon}"
        );
    }
    for icon in ["ClayooSelectionSelectAll", "ClayooSelectionSelectNone"] {
        let index = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert!(!om9_command_is_active(index));
    }
}
#[test]
fn command_history_requires_the_observed_effect_of_that_command() {
    let _lock = LOCK.lock().unwrap();
    for (icon, expected) in [
        ("SelectAll", 8),
        ("SelectNone", 8),
        ("Delete", 16),
        ("Undo", 4),
        ("Redo", 4),
        ("FileSave", 2),
        ("FileNew", 1),
        ("ViewZoomZoomSelected", 32),
    ] {
        let index = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert!(!om9_command_success(index, 0), "no-op/cancel {icon}");
        assert!(
            !om9_command_success(index, 63 ^ expected),
            "unrelated effects {icon}"
        );
        assert!(
            om9_command_success(index, expected),
            "successful effect {icon}"
        );
    }
    assert!(!om9_command_success(usize::MAX, 63));
    assert_eq!(om9_command_permissions(usize::MAX), 0);
    for icon in [
        "Undo",
        "Redo",
        "FileNew",
        "FileOpen",
        "FileSave",
        "FileSaveAs",
    ] {
        let index = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert_eq!(
            om9_command_permissions(index),
            0,
            "preserve native task/edit policy for {icon}"
        );
    }
    for (icon, expected) in [
        ("SelectAll", 4),
        ("SelectNone", 4),
        ("Delete", 1),
        ("ViewTop", 2),
        ("ViewZoomZoomSelected", 2),
    ] {
        let index = (0..om9_command_count())
            .find(|&i| unsafe { CStr::from_ptr(om9_command_icon(i)) }.to_bytes() == icon.as_bytes())
            .unwrap();
        assert_eq!(om9_command_permissions(index), expected);
    }
    assert_eq!(om9_workspace_command_count(), 12);
    assert_eq!(om9_workspace_command(12), usize::MAX);
    let workspace: Vec<_> = (0..12).map(|i| om9_workspace_command(i)).collect();
    assert!(workspace.iter().all(|&i| i < om9_command_count()));
    assert_eq!(
        workspace
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        12
    );
}
#[test]
fn catalog_indices_strings_and_lifetimes_are_valid() {
    let _lock = LOCK.lock().unwrap();
    assert_eq!(om9_sidebar_group_count(), 18);
    assert_eq!(om9_sidebar_quick_count(), 11);
    for g in 0..18 {
        let p = om9_sidebar_group_title(g);
        assert!(!p.is_null());
        assert!(unsafe { CStr::from_ptr(p) }.to_str().is_ok());
        for i in 0..om9_sidebar_group_item_count(g) {
            assert!(om9_sidebar_group_item_command(g, i) < om9_command_count());
        }
    }
    for i in 0..11 {
        assert!(om9_sidebar_quick_command(i) < om9_command_count());
    }
    let p = om9_sidebar_group_title(0);
    om9_sidebar_select_group(7);
    om9_sidebar_reset();
    assert_eq!(p, om9_sidebar_group_title(0));
    for f in [
        om9_command_id,
        om9_command_menu_text,
        om9_command_tooltip,
        om9_command_icon,
        om9_command_native_id,
        om9_command_execute,
    ] {
        assert!(f(usize::MAX).is_null());
    }
    for f in [
        om9_sidebar_group_title,
        om9_sidebar_group_color,
        om9_menu_group_title,
        om9_toolbar_group_title,
    ] {
        assert!(f(usize::MAX).is_null());
    }
    assert_eq!(om9_sidebar_group_kind(usize::MAX), u32::MAX);
    assert_eq!(om9_sidebar_group_item_count(usize::MAX), 0);
    assert_eq!(om9_sidebar_group_item_command(0, usize::MAX), usize::MAX);
    assert_eq!(om9_sidebar_quick_command(usize::MAX), usize::MAX);
    assert_eq!(om9_sidebar_history_command(usize::MAX), usize::MAX);
    assert!(om9_menu_group_command_id(0, usize::MAX).is_null());
    assert!(om9_toolbar_group_command_id(0, usize::MAX).is_null());
    assert!(!om9_command_is_active(usize::MAX));
}
#[test]
fn state_bounds_failure_history_and_reset() {
    let _lock = LOCK.lock().unwrap();
    om9_sidebar_reset();
    assert!(!om9_sidebar_select_group(18));
    assert!(!om9_sidebar_set_section(7, false, true));
    assert_eq!(om9_sidebar_section_flags(7), 0);
    assert!(om9_sidebar_set_section(0, false, true));
    assert_eq!(om9_sidebar_section_flags(0), 2);
    let n = om9_sidebar_history_count();
    om9_sidebar_record_execution(0, false);
    om9_sidebar_record_execution(usize::MAX, true);
    assert_eq!(om9_sidebar_history_count(), n);
    for _ in 0..21 {
        om9_sidebar_record_execution(0, true);
    }
    assert_eq!(om9_sidebar_history_count(), 20);
    om9_sidebar_reset();
    assert_eq!(om9_sidebar_section_flags(0), 1);
}
