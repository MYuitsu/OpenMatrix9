// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selected native OS adapter. No document/transaction authority lives here.
use crate::file_lifecycle::FilePhase;
use om9_core::{Error, Result};
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use om9_native_owner::file_dialog::{self, Kind};
#[cfg(windows)]
pub fn arm_cancel_probe(
    parent: &eframe::Frame,
    phase: FilePhase,
) -> Result<om9_native_owner::dialog_probe::ModalCancelProbe> {
    use om9_native_owner::dialog_probe::{DialogKind, ModalCancelProbe};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(handle) = parent
        .window_handle()
        .map_err(|e| Error::Io(e.to_string()))?
        .as_raw()
    else {
        return Err(Error::Unsupported("Win32 dialog probe parent"));
    };
    let kind = match phase {
        FilePhase::OpenPath => DialogKind::Open,
        FilePhase::SavePath => DialogKind::Save,
        _ => return Err(Error::Invalid("dialog probe phase")),
    };
    ModalCancelProbe::arm(handle.hwnd.get() as usize, kind).map_err(|e| Error::Io(e.to_string()))
}
pub fn pick(
    parent: &eframe::Frame,
    phase: FilePhase,
    current: Option<&Path>,
) -> Result<Option<PathBuf>> {
    #[cfg(windows)]
    {
        use om9_native_owner::file_dialog::{Mode, choose};
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let mode = match phase {
            FilePhase::OpenPath => Mode::Open,
            FilePhase::SavePath => Mode::Save,
            _ => return Err(Error::Invalid("native file picker phase")),
        };
        let RawWindowHandle::Win32(handle) = parent
            .window_handle()
            .map_err(|e| Error::Io(e.to_string()))?
            .as_raw()
        else {
            return Err(Error::Unsupported("Win32 file picker parent"));
        };
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            choose(handle.hwnd.get() as usize, mode, current)
        }))
        .map_err(|_| Error::Io("native file picker panicked".into()))?
        .map_err(|error| Error::Io(format!("Windows file dialog: {error}")))
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let _ = parent;
        let save = match phase {
            FilePhase::OpenPath => false,
            FilePhase::SavePath => true,
            _ => return Err(Error::Invalid("native file picker phase")),
        };
        file_dialog::choose(Kind::Document, save, current, None).map_err(Error::Io)
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = (parent, phase, current);
        Err(Error::Unsupported("native file chooser on this platform"))
    }
}
/// Rhino 3DM picker for the host adapter. `save` chooses the Save dialog; the returned path is
/// still validated by the import/export owner.
#[cfg(windows)]
pub fn pick_3dm(
    parent: &eframe::Frame,
    save: bool,
    current: Option<&Path>,
) -> Result<Option<PathBuf>> {
    use om9_native_owner::file_dialog::{Mode, choose_3dm};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let mode = if save { Mode::Save } else { Mode::Open };
    let RawWindowHandle::Win32(handle) = parent
        .window_handle()
        .map_err(|e| Error::Io(e.to_string()))?
        .as_raw()
    else {
        return Err(Error::Unsupported("Win32 3DM picker parent"));
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        choose_3dm(handle.hwnd.get() as usize, mode, current)
    }))
    .map_err(|_| Error::Io("native 3DM picker panicked".into()))?
    .map_err(|error| Error::Io(format!("Windows 3DM dialog: {error}")))
}
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn pick_3dm(
    _parent: &eframe::Frame,
    save: bool,
    current: Option<&Path>,
) -> Result<Option<PathBuf>> {
    file_dialog::choose(Kind::ThreeDm, save, current, None).map_err(Error::Io)
}
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn pick_3dm(
    _parent: &eframe::Frame,
    _save: bool,
    _current: Option<&Path>,
) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native 3DM chooser on this platform"))
}

/// Binary STL save picker for File ▸ Export Selected. The path is still validated by
/// the export owner, which owns the extension and the atomic write.
#[cfg(windows)]
pub fn pick_stl(parent: &eframe::Frame, current: Option<&Path>) -> Result<Option<PathBuf>> {
    use om9_native_owner::file_dialog::{Mode, choose_stl};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(handle) = parent
        .window_handle()
        .map_err(|e| Error::Io(e.to_string()))?
        .as_raw()
    else {
        return Err(Error::Unsupported("Win32 STL picker parent"));
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        choose_stl(handle.hwnd.get() as usize, Mode::Save, current)
    }))
    .map_err(|_| Error::Io("native STL picker panicked".into()))?
    .map_err(|error| Error::Io(format!("Windows STL dialog: {error}")))
}
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn pick_stl(_parent: &eframe::Frame, current: Option<&Path>) -> Result<Option<PathBuf>> {
    file_dialog::choose(Kind::Stl, true, current, None).map_err(Error::Io)
}
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn pick_stl(_parent: &eframe::Frame, _current: Option<&Path>) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native STL chooser on this platform"))
}

#[cfg(windows)]
pub fn pick_project_archive(parent: &eframe::Frame, save: bool) -> Result<Option<PathBuf>> {
    use om9_native_owner::file_dialog::{Mode, choose_project_archive};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(handle) = parent.window_handle().map_err(|error| Error::Io(error.to_string()))?.as_raw() else {
        return Err(Error::Unsupported("Win32 project picker parent"));
    };
    let mode = if save { Mode::Save } else { Mode::Open };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| choose_project_archive(handle.hwnd.get() as usize, mode)))
        .map_err(|_| Error::Io("native project picker panicked".into()))?
        .map_err(|error| Error::Io(format!("Windows project dialog: {error}")))
}
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn pick_project_archive(_parent: &eframe::Frame, save: bool) -> Result<Option<PathBuf>> {
    file_dialog::choose(Kind::ProjectArchive, save, None, None).map_err(Error::Io)
}
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn pick_project_archive(_parent: &eframe::Frame, _save: bool) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native project chooser on this platform"))
}

#[cfg(windows)]
pub fn pick_project_folder(parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(handle) = parent.window_handle().map_err(|error| Error::Io(error.to_string()))?.as_raw() else {
        return Err(Error::Unsupported("Win32 folder picker parent"));
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| om9_native_owner::file_dialog::choose_project_folder(handle.hwnd.get() as usize)))
        .map_err(|_| Error::Io("native folder picker panicked".into()))?
        .map_err(|error| Error::Io(format!("Windows folder dialog: {error}")))
}
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn pick_project_folder(_parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    file_dialog::choose(Kind::ProjectFolder, false, None, None).map_err(Error::Io)
}
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn pick_project_folder(_parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native project folder chooser on this platform"))
}

#[cfg(windows)]
pub fn pick_addon_zip(parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    use om9_native_owner::file_dialog::choose_addon_zip;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(handle) = parent.window_handle().map_err(|e| Error::Io(e.to_string()))?.as_raw() else {
        return Err(Error::Unsupported("Win32 add-on picker parent"));
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| choose_addon_zip(handle.hwnd.get() as usize)))
        .map_err(|_| Error::Io("native add-on picker panicked".into()))?
        .map_err(|error| Error::Io(format!("Windows add-on dialog: {error}")))
}
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn pick_addon_zip(_parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    file_dialog::choose(Kind::Addon, false, None, None).map_err(Error::Io)
}
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn pick_addon_zip(_parent: &eframe::Frame) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native add-on chooser on this platform"))
}

#[cfg(windows)]
pub fn pick_fixture_for_audit(
    parent: &eframe::Frame,
    phase: FilePhase,
    fixture: &Path,
) -> Result<(Option<PathBuf>, serde_json::Value)> {
    use om9_native_owner::{
        dialog_probe::{DialogKind, ModalCancelProbe},
        file_dialog::{Mode, choose_fixture_for_audit},
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let (mode, kind) = match phase {
        FilePhase::OpenPath => (Mode::Open, DialogKind::Open),
        FilePhase::SavePath => (Mode::Save, DialogKind::Save),
        _ => return Err(Error::Invalid("audit chooser phase")),
    };
    let RawWindowHandle::Win32(handle) = parent
        .window_handle()
        .map_err(|e| Error::Io(e.to_string()))?
        .as_raw()
    else {
        return Err(Error::Unsupported("Win32 audit parent"));
    };
    let probe = ModalCancelProbe::arm_accept(handle.hwnd.get() as usize, kind)
        .map_err(|e| Error::Io(e.to_string()))?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        choose_fixture_for_audit(handle.hwnd.get() as usize, mode, fixture)
    }));
    let receipt = probe.finish().map_err(|e| Error::Io(e.to_string()))?;
    let path = result
        .map_err(|_| Error::Io("audit chooser panicked".into()))?
        .map_err(|e| Error::Io(format!("Windows file dialog: {e}")))?;
    let value = serde_json::json!({"pid":receipt.pid,"ui_thread":receipt.thread,"observed":receipt.observed,"accept_posted":receipt.accept_posted,"close_posted":receipt.close_posted,"destroyed":receipt.destroyed,"elapsed_ms":receipt.elapsed_ms,"returned_path":path,"phase":format!("{phase:?}"),"boundary":"owned PID/thread/root/modal/title; enabled IDOK child WM_COMMAND; real Show/GetResult; joined probe"});
    Ok((path, value))
}

pub(crate) fn pick_view_image(
    parent: &eframe::Frame,
    current: Option<&Path>,
    alpha: &mut bool,
) -> Result<Option<PathBuf>> {
    #[cfg(windows)]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let RawWindowHandle::Win32(handle) = parent
            .window_handle()
            .map_err(|e| Error::Io(e.to_string()))?
            .as_raw()
        else {
            return Err(Error::Unsupported("Win32 capture picker parent"));
        };
        om9_native_owner::file_dialog::choose_view_image(handle.hwnd.get() as usize, current, alpha)
            .map_err(|e| Error::Io(format!("Windows image dialog: {e}")))
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let _ = parent;
        file_dialog::choose(Kind::ViewImage, true, current, Some(alpha)).map_err(Error::Io)
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = (parent, current, alpha);
        Err(Error::Unsupported("native capture dialog on this platform"))
    }
}
