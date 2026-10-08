use crate::{
    menu::{MenuCatalog, MenuKind},
    state::UiState,
};
use std::{
    collections::HashMap,
    ffi::{CString, c_char},
    ptr,
    sync::{Mutex, OnceLock},
};

struct Command {
    id: CString,
    icon: CString,
    native: Option<CString>,
    caption: CString,
}
struct Bundle {
    catalog: MenuCatalog,
    titles: Vec<CString>,
    colors: Vec<CString>,
    commands: Vec<Command>,
    groups: Vec<Vec<usize>>,
    quick: Vec<usize>,
}
static DATA: OnceLock<Option<Bundle>> = OnceLock::new();
static STATE: OnceLock<Mutex<UiState>> = OnceLock::new();
fn state() -> &'static Mutex<UiState> {
    STATE.get_or_init(|| Mutex::new(UiState::new()))
}
fn native(icon: &str) -> Option<&'static str> {
    match icon {
        "FileNew" => Some("Std_New"),
        "FileOpen" => Some("Std_Open"),
        "FileSave" => Some("Std_Save"),
        "FileSaveAs" => Some("Std_SaveAs"),
        "ViewZoomZoomExtents" | "FitAll" => Some("Std_ViewFitAll"),
        "Undo" => Some("Std_Undo"),
        "Redo" => Some("Std_Redo"),
        "ViewFront" => Some("Std_ViewFront"),
        "ViewTop" => Some("Std_ViewTop"),
        "ViewRight" => Some("Std_ViewRight"),
        _ => crate::workspace::native(icon),
    }
}
fn data() -> Option<&'static Bundle> {
    DATA.get_or_init(|| {
        let catalog = MenuCatalog::parse(include_str!("../../Resources/menu/MainMenu.ini")).ok()?;
        let mut commands = Vec::new();
        let mut index = HashMap::new();
        for icon in catalog
            .groups
            .iter()
            .flat_map(|g| g.icons.iter())
            .chain(catalog.quick_icons.iter())
            .map(String::as_str)
            .chain([
                "Undo",
                "Redo",
                "ViewFront",
                "ViewTop",
                "ViewRight",
                "FitAll",
            ])
            .chain(crate::workspace::ICONS)
            .chain(crate::core_views::ICONS)
            .chain(crate::core_notes::ICONS)
            .chain(crate::core_snaps::ICONS)
            .chain(crate::core_keyboard::ICONS)
            .chain(crate::core_3dm::ICONS)
        {
            if index.contains_key(icon) {
                continue;
            }
            index.insert(icon.to_owned(), commands.len());
            commands.push(Command {
                id: CString::new(
                    crate::curve::name(icon)
                        .or_else(|| crate::core_3dm::command(icon))
                        .or_else(|| crate::core_views::command(icon))
                        .or_else(|| crate::core_distance::command(icon))
                        .or_else(|| crate::core_angle::command(icon))
                        .or_else(|| crate::core_snaps::command(icon))
                        .or_else(|| crate::core_keyboard::command(icon))
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("OM9_{icon}")),
                )
                .ok()?,
                icon: CString::new(icon).ok()?,
                native: native(icon).and_then(|v| CString::new(v).ok()),
                caption: CString::new(
                    crate::curve::name(icon)
                        .or_else(|| crate::edit::Kind::from_name(icon).map(|k| k.caption()))
                        .or_else(|| crate::surface::Kind::from_name(icon).map(|k| k.caption()))
                        .or_else(|| crate::solid::Kind::from_name(icon).map(|k| k.caption()))
                        .or_else(|| crate::core_3dm::caption(icon))
                        .or_else(|| crate::core_views::command(icon))
                        .or_else(|| crate::core_distance::command(icon))
                        .or_else(|| crate::core_angle::command(icon))
                        .or_else(|| crate::core_snaps::command(icon))
                        .or_else(|| crate::core_keyboard::command(icon))
                        .or_else(|| crate::core_notes::caption(icon))
                        .unwrap_or_else(|| crate::workspace::caption(icon)),
                )
                .ok()?,
            });
        }
        let groups = catalog
            .groups
            .iter()
            .map(|g| {
                g.icons
                    .iter()
                    .filter_map(|i| index.get(i).copied())
                    .collect()
            })
            .collect();
        let quick = catalog
            .quick_icons
            .iter()
            .filter_map(|i| index.get(i).copied())
            .collect();
        let titles = catalog
            .groups
            .iter()
            .map(|g| CString::new(g.title.as_str()).ok())
            .collect::<Option<Vec<_>>>()?;
        let colors = catalog
            .groups
            .iter()
            .map(|g| CString::new(g.color.as_str()).ok())
            .collect::<Option<Vec<_>>>()?;
        Some(Bundle {
            catalog,
            titles,
            colors,
            commands,
            groups,
            quick,
        })
    })
    .as_ref()
}
fn command(i: usize) -> Option<&'static Command> {
    data()?.commands.get(i)
}
fn ptr(s: Option<&CString>) -> *const c_char {
    s.map_or(ptr::null(), |v| v.as_ptr())
}
macro_rules! string_getter {
    ($name:ident,$field:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(i: usize) -> *const c_char {
            ptr(command(i).map(|c| &c.$field))
        }
    };
}
string_getter!(om9_command_id, id);
string_getter!(om9_command_menu_text, caption);
string_getter!(om9_command_tooltip, caption);
string_getter!(om9_command_icon, icon);
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_count() -> usize {
    data().map_or(0, |d| d.commands.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_workspace_command_count() -> usize {
    crate::workspace::ICONS.len()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_workspace_command(i: usize) -> usize {
    crate::workspace::ICONS
        .get(i)
        .and_then(|icon| {
            data()?
                .commands
                .iter()
                .position(|c| c.icon.as_bytes() == icon.as_bytes())
        })
        .unwrap_or(usize::MAX)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_success(i: usize, effects: u32) -> bool {
    command(i)
        .and_then(|c| c.native.as_ref())
        .and_then(|s| s.to_str().ok())
        .is_some_and(|native| crate::workspace::success(native, effects))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_permissions(i: usize) -> u32 {
    if command(i).and_then(|c| c.icon.to_str().ok()).and_then(crate::edit::Kind::from_name).is_some() {
        return 1;
    }
    if command(i).and_then(|c| c.icon.to_str().ok()).and_then(crate::solid::Kind::from_name).is_some() {
        return 1;
    }
    if command(i).and_then(|c| c.icon.to_str().ok()).and_then(crate::curve::name).is_some() {
        return 1;
    }
    if command(i).and_then(|c| c.icon.to_str().ok()).and_then(crate::surface::Kind::from_name).is_some() {
        return 1;
    }
    match crate::core_3dm::om9_3dm_operation(i) {
        1 => return 1,
        2 => return 0,
        _ => (),
    }
    if command(i).and_then(|c| c.icon.to_str().ok()) == Some("ViewBackgroundBitmapPictureFrame") {
        return 1;
    }
    if command(i)
        .and_then(|c| c.icon.to_str().ok())
        .is_some_and(|icon| icon == "InfoSettingsViewportTabsToggle")
    {
        return 2;
    }
    if command(i)
        .and_then(|c| c.icon.to_str().ok())
        .is_some_and(|icon| crate::core_notes::ICONS.contains(&icon))
    {
        return 1;
    }
    command(i)
        .and_then(|c| c.native.as_ref())
        .and_then(|s| s.to_str().ok())
        .map_or(0, crate::workspace::permissions)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_native_id(i: usize) -> *const c_char {
    ptr(command(i).and_then(|c| c.native.as_ref()))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_execute(i: usize) -> *const c_char {
    om9_command_native_id(i)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_command_is_active(i: usize) -> bool {
    !om9_command_native_id(i).is_null()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_count() -> usize {
    data().map_or(0, |d| d.catalog.groups.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_title(g: usize) -> *const c_char {
    ptr(data().and_then(|d| d.titles.get(g)))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_color(g: usize) -> *const c_char {
    ptr(data().and_then(|d| d.colors.get(g)))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_kind(g: usize) -> u32 {
    data()
        .and_then(|d| d.catalog.groups.get(g))
        .map_or(u32::MAX, |g| match g.kind {
            MenuKind::IconList => 0,
            MenuKind::Custom => 1,
            MenuKind::Reset => 2,
        })
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_item_count(g: usize) -> usize {
    data().and_then(|d| d.groups.get(g)).map_or(0, Vec::len)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_group_item_command(g: usize, i: usize) -> usize {
    data()
        .and_then(|d| d.groups.get(g))
        .and_then(|v| v.get(i))
        .copied()
        .unwrap_or(usize::MAX)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_quick_count() -> usize {
    data().map_or(0, |d| d.quick.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_quick_command(i: usize) -> usize {
    data()
        .and_then(|d| d.quick.get(i))
        .copied()
        .unwrap_or(usize::MAX)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_selected_group() -> usize {
    state().lock().ok().map_or(0, |s| s.selected_group)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_select_group(g: usize) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut s| s.select_group(g, om9_sidebar_group_count()))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_section_flags(i: usize) -> u32 {
    state()
        .lock()
        .ok()
        .and_then(|s| s.sections.get(i).copied())
        .map_or(0, |s| u32::from(s.visible) | (u32::from(s.collapsed) << 1))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_set_section(i: usize, visible: bool, collapsed: bool) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut s| s.set_section(i, visible, collapsed))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_reset() {
    if let Ok(mut s) = state().lock() {
        s.reset();
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_record_execution(i: usize, success: bool) {
    if command(i).is_some()
        && let Ok(mut s) = state().lock()
    {
        s.record_execution(i, success);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_history_count() -> usize {
    state().lock().ok().map_or(0, |s| s.history.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_sidebar_history_command(i: usize) -> usize {
    state()
        .lock()
        .ok()
        .and_then(|s| s.history.get(i).copied())
        .unwrap_or(usize::MAX)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_menu_group_count() -> usize {
    om9_sidebar_group_count()
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_menu_group_title(g: usize) -> *const c_char {
    om9_sidebar_group_title(g)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_menu_group_command_count(g: usize) -> usize {
    om9_sidebar_group_item_count(g)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_menu_group_command_id(g: usize, i: usize) -> *const c_char {
    om9_command_id(om9_sidebar_group_item_command(g, i))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_toolbar_group_count() -> usize {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_toolbar_group_title(g: usize) -> *const c_char {
    if g == 0 {
        c"OpenMatrix9".as_ptr()
    } else {
        ptr::null()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_toolbar_group_command_count(g: usize) -> usize {
    if g == 0 { om9_sidebar_quick_count() } else { 0 }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_toolbar_group_command_id(g: usize, i: usize) -> *const c_char {
    if g == 0 {
        om9_command_id(om9_sidebar_quick_command(i))
    } else {
        ptr::null()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_workbench_activated() {}
#[unsafe(no_mangle)]
pub extern "C" fn om9_workbench_deactivated() {}
