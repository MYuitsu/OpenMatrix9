// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selected Win32 common file dialog. Rust retains all document authority.
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FILEOPENDIALOGOPTIONS, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
            FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST, FOS_STRICTFILETYPES, FileOpenDialog,
            FileSaveDialog, IFileDialog, IShellItem, SHCreateItemFromParsingName,
            SIGDN_FILESYSPATH,
        },
    },
    core::{HRESULT, PCWSTR, PWSTR, w},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Save,
}
/// Native dialog filter. The extension is part of the owned selection contract; the
/// returned path is still re-validated by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogFilter {
    Om9,
    ThreeDm,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogError {
    pub stage: &'static str,
    pub hresult: i32,
}
impl std::fmt::Display for DialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} failed (HRESULT 0x{:08X})",
            self.stage, self.hresult as u32
        )
    }
}
impl std::error::Error for DialogError {}
fn native(stage: &'static str, error: windows::core::Error) -> DialogError {
    DialogError {
        stage,
        hresult: error.code().0,
    }
}
fn invalid(stage: &'static str) -> DialogError {
    DialogError {
        stage,
        hresult: 0x80070057u32 as i32,
    }
}
fn show_result(result: windows::core::Result<()>) -> Result<bool, DialogError> {
    match result {
        Ok(()) => Ok(true),
        Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => Ok(false),
        Err(error) => Err(native("Show", error)),
    }
}
fn options(default: FILEOPENDIALOGOPTIONS, mode: Mode) -> FILEOPENDIALOGOPTIONS {
    let base = default | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_NOCHANGEDIR;
    if mode == Mode::Save {
        (base & !FOS_OVERWRITEPROMPT) | FOS_STRICTFILETYPES
    } else {
        base
    }
}
// !Send/!Sync: successful COM initialization must be balanced on this thread.
struct Apartment(std::marker::PhantomData<std::rc::Rc<()>>);
impl Apartment {
    fn initialize() -> Result<Self, DialogError> {
        // SAFETY: Enter STA on this calling UI thread, never silently reuse MTA.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) }
            .ok()
            .map_err(|e| native("Initialize COM", e))?;
        Ok(Self(std::marker::PhantomData))
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: One balanced uninitialize for every successful S_OK/S_FALSE init.
        unsafe {
            CoUninitialize();
        }
    }
}
fn wide(value: &OsStr) -> Result<Vec<u16>, DialogError> {
    let mut value: Vec<_> = value.encode_wide().take(32768).collect();
    if value.len() == 32768 || value.contains(&0) {
        return Err(invalid("UTF-16 path/name admission"));
    }
    value.push(0);
    Ok(value)
}
fn admit_fixture_reply(expected: &Path, selected: &Path) -> Result<(), DialogError> {
    if expected != selected {
        return Err(invalid("audit selection differs from owned fixture"));
    }
    Ok(())
}
struct CoTaskWide(PWSTR);
impl Drop for CoTaskWide {
    fn drop(&mut self) {
        // SAFETY: GetDisplayName allocates this string with the COM task allocator.
        unsafe {
            CoTaskMemFree(Some(self.0.0.cast()));
        }
    }
}
impl CoTaskWide {
    fn path(&self) -> Result<PathBuf, DialogError> {
        if self.0.0.is_null() {
            return Err(invalid("GetDisplayName null path"));
        }
        // SAFETY: IShellItem's documented result is a NUL-terminated task-allocated
        // string. Read under that native contract, with an explicit Windows path cap.
        // A safe caller does not certify a COM server violating that contract.
        unsafe {
            for length in 0..32768 {
                if *self.0.0.add(length) == 0 {
                    if length == 0 {
                        return Err(invalid("GetDisplayName empty path"));
                    }
                    return Ok(PathBuf::from(OsString::from_wide(
                        std::slice::from_raw_parts(self.0.0, length),
                    )));
                }
            }
        }
        Err(invalid("GetDisplayName path length"))
    }
}
/// Synchronous UI-thread adapter. Only a verified own-window handle is accepted;
/// document/transaction authority stays in the safe Rust caller.
pub fn choose(
    parent: usize,
    mode: Mode,
    current: Option<&Path>,
) -> Result<Option<PathBuf>, DialogError> {
    choose_impl(parent, mode, current, DialogFilter::Om9, false)
}
/// The same owned dialog with the Rhino 3DM filter; used by the 3DM host adapter.
pub fn choose_3dm(
    parent: usize,
    mode: Mode,
    current: Option<&Path>,
) -> Result<Option<PathBuf>, DialogError> {
    choose_impl(parent, mode, current, DialogFilter::ThreeDm, false)
}
/// Explicit own-app audit prefill. Show and GetResult remain the native
/// selection boundary; this hint never becomes a returned selection itself.
pub fn choose_fixture_for_audit(
    parent: usize,
    mode: Mode,
    fixture: &Path,
) -> Result<Option<PathBuf>, DialogError> {
    choose_impl(parent, mode, Some(fixture), DialogFilter::Om9, true)
}
fn choose_impl(
    parent: usize,
    mode: Mode,
    current: Option<&Path>,
    filter: DialogFilter,
    audit_prefill: bool,
) -> Result<Option<PathBuf>, DialogError> {
    use windows_sys::Win32::{
        System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
        UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    let mut pid = 0;
    // SAFETY: Win32 validates HWND. Require this process and this UI thread.
    let thread = unsafe { GetWindowThreadProcessId(parent as _, &mut pid) };
    if parent == 0
        || thread == 0
        || pid != unsafe { GetCurrentProcessId() }
        || thread != unsafe { GetCurrentThreadId() }
    {
        return Err(invalid("owner HWND"));
    }
    let _apartment = Apartment::initialize()?;
    // SAFETY: A live STA, generated interface bindings, borrowed bounded setter
    // strings, verified parent HWND, RAII COM interfaces. No raw pointer escapes.
    unsafe {
        let dialog: IFileDialog = CoCreateInstance(
            if mode == Mode::Open {
                &FileOpenDialog
            } else {
                &FileSaveDialog
            },
            None,
            CLSCTX_INPROC_SERVER,
        )
        .map_err(|e| native("Create dialog", e))?;
        let defaults = dialog.GetOptions().map_err(|e| native("GetOptions", e))?;
        dialog
            .SetOptions(options(defaults, mode))
            .map_err(|e| native("SetOptions", e))?;
        let (title, filter_name, filter_spec, extension, default_name) = match filter {
            DialogFilter::Om9 => (
                if mode == Mode::Open {
                    w!("Open OpenMatrix9 document")
                } else {
                    w!("Save OpenMatrix9 document")
                },
                w!("OpenMatrix9 document"),
                w!("*.om9"),
                w!("om9"),
                "Untitled.om9",
            ),
            DialogFilter::ThreeDm => (
                if mode == Mode::Open {
                    w!("Import Rhino 3DM")
                } else {
                    w!("Export Rhino 3DM")
                },
                w!("Rhino 3DM"),
                w!("*.3dm"),
                w!("3dm"),
                "Untitled.3dm",
            ),
        };
        dialog.SetTitle(title).map_err(|e| native("SetTitle", e))?;
        dialog
            .SetFileTypes(&[COMDLG_FILTERSPEC {
                pszName: filter_name,
                pszSpec: filter_spec,
            }])
            .map_err(|e| native("SetFileTypes", e))?;
        dialog
            .SetDefaultExtension(extension)
            .map_err(|e| native("SetDefaultExtension", e))?;
        if let Some(directory) = current
            .and_then(Path::parent)
            .filter(|directory| directory.is_dir())
        {
            let directory = wide(directory.as_os_str())?;
            let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(directory.as_ptr()), None)
                .map_err(|e| native("Resolve initial directory", e))?;
            dialog
                .SetFolder(&item)
                .map_err(|e| native("SetFolder", e))?;
        }
        if mode == Mode::Save || audit_prefill {
            let extension_text = if matches!(filter, DialogFilter::ThreeDm) {
                "3dm"
            } else {
                "om9"
            };
            let suggested = if audit_prefill {
                current
                    .and_then(Path::file_name)
                    .map(OsStr::to_os_string)
                    .unwrap_or_else(|| OsString::from(default_name))
            } else {
                current
                    .and_then(Path::file_name)
                    .map(|name| {
                        let mut path = PathBuf::from(name);
                        path.set_extension(extension_text);
                        path.into_os_string()
                    })
                    .unwrap_or_else(|| OsString::from(default_name))
            };
            let name = wide(&suggested)?;
            dialog
                .SetFileName(PCWSTR(name.as_ptr()))
                .map_err(|e| native("SetFileName", e))?;
        }
        if !show_result(dialog.Show(Some(HWND(parent as _))))? {
            return Ok(None);
        }
        let item = dialog.GetResult().map_err(|e| native("GetResult", e))?;
        let path = CoTaskWide(
            item.GetDisplayName(SIGDN_FILESYSPATH)
                .map_err(|e| native("GetDisplayName", e))?,
        );
        let path = path.path()?;
        if audit_prefill {
            admit_fixture_reply(current.expect("audit fixture"), &path)?;
        }
        Ok(Some(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Com::COINIT_MULTITHREADED;
    #[test]
    fn only_native_show_cancel_is_a_cancel_and_other_hresult_is_visible() {
        assert_eq!(
            show_result(Err(windows::core::Error::from_hresult(
                HRESULT::from_win32(ERROR_CANCELLED.0)
            ))),
            Ok(false)
        );
        let error = show_result(Err(windows::core::Error::from_hresult(HRESULT(
            0x80004005u32 as i32,
        ))))
        .unwrap_err();
        assert_eq!(error.stage, "Show");
        assert_eq!(error.hresult as u32, 0x80004005);
        assert!(error.to_string().contains("0x80004005"));
    }
    #[test]
    fn save_leaves_overwrite_to_owned_confirmation_and_requires_filesystem_path() {
        let value = options(FOS_OVERWRITEPROMPT, Mode::Save);
        assert_eq!(value.0 & FOS_OVERWRITEPROMPT.0, 0);
        assert_ne!(value.0 & FOS_FORCEFILESYSTEM.0, 0);
        assert_ne!(value.0 & FOS_PATHMUSTEXIST.0, 0);
    }
    #[test]
    fn audit_prefill_is_bounded_and_never_bypasses_the_native_owner() {
        let bad = OsString::from_wide(&[b'a' as u16, 0, b'b' as u16]);
        assert!(wide(&bad).is_err());
        assert!(wide(&OsString::from("x".repeat(32768))).is_err());
        let error =
            choose_fixture_for_audit(0, Mode::Open, Path::new("H:/owned/fixture.om9")).unwrap_err();
        assert_eq!(error.stage, "owner HWND");
        let expected = Path::new("H:/owned/fixture.om9");
        assert!(admit_fixture_reply(expected, expected).is_ok());
        assert!(admit_fixture_reply(expected, Path::new("C:/unexpected.om9")).is_err());
        assert!(admit_fixture_reply(expected, Path::new("H:/owned/other.om9")).is_err());
    }
    #[test]
    fn real_com_apartment_conflict_is_error_and_does_not_uninitialize_the_owner() {
        std::thread::spawn(|| {
            // SAFETY: This test owns this fresh thread's COM initialization and
            // balances both successful calls, including S_FALSE.
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED).ok().unwrap();
                let result = Apartment::initialize();
                assert!(result.is_err(), "MTA cannot be silently reused as STA");
                assert_eq!(result.err().unwrap().hresult as u32, 0x80010106);
                let second = CoInitializeEx(None, COINIT_MULTITHREADED);
                assert_eq!(second.0, 1);
                CoUninitialize();
                CoUninitialize();
            }
        })
        .join()
        .unwrap();
    }
}
