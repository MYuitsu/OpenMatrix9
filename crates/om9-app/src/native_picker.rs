// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selected native OS adapter. No document/transaction authority lives here.
use crate::file_lifecycle::FilePhase;
use om9_core::{Error, Result};
use std::path::{Path, PathBuf};
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
    #[cfg(not(windows))]
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
#[cfg(not(windows))]
pub fn pick_3dm(
    _parent: &eframe::Frame,
    _save: bool,
    _current: Option<&Path>,
) -> Result<Option<PathBuf>> {
    Err(Error::Unsupported("native 3DM chooser on this platform"))
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
