// SPDX-License-Identifier: LGPL-2.1-or-later
//! AppKit dialogs supply paths. The application owns validation and document changes.
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSButton, NSControlStateValueOff, NSControlStateValueOn, NSModalResponseCancel,
    NSModalResponseOK, NSOpenPanel, NSSavePanel,
};
use objc2_foundation::{NSArray, NSString, NSURL};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Document,
    ThreeDm,
    Stl,
    ProjectArchive,
    ProjectFolder,
    Addon,
    ViewImage,
}

pub fn choose(
    kind: Kind,
    save: bool,
    current: Option<&Path>,
    alpha: Option<&mut bool>,
) -> Result<Option<PathBuf>, String> {
    let mtm = MainThreadMarker::new().ok_or("AppKit file picker requires the main thread")?;
    let (title, extensions, default_name): (_, &[_], _) = match (kind, save) {
        (Kind::Document, false) => ("Open OM9 document", &["om9"], "Untitled"),
        (Kind::Document, true) => ("Save OM9 document", &["om9"], "Untitled"),
        (Kind::ThreeDm, false) => ("Import Rhino 3DM", &["3dm"], "Untitled"),
        (Kind::ThreeDm, true) => ("Export Rhino 3DM", &["3dm"], "Untitled"),
        (Kind::Stl, _) => ("Export STL", &["stl"], "Untitled"),
        (Kind::ProjectArchive, false) => ("Import OM9 project", &["om9project"], "Project"),
        (Kind::ProjectArchive, true) => ("Export OM9 project", &["om9project"], "Project"),
        (Kind::ProjectFolder, _) => ("Select project folder", &[], ""),
        (Kind::Addon, _) => ("Install OM9 add-on", &["zip"], ""),
        (Kind::ViewImage, _) => (
            "Save Viewport Image",
            &["png", "jpg", "jpeg", "bmp", "pcx", "tif", "tiff", "tga"],
            "Viewport",
        ),
    };

    let open_panel = (!save).then(|| NSOpenPanel::openPanel(mtm));
    let save_panel = save.then(|| NSSavePanel::savePanel(mtm));
    let panel: &NSSavePanel = if let Some(open) = &open_panel {
        let folders = kind == Kind::ProjectFolder;
        open.setCanChooseDirectories(folders);
        open.setCanChooseFiles(!folders);
        open.setAllowsMultipleSelection(false);
        open
    } else {
        save_panel.as_deref().expect("saving has a panel")
    };
    panel.setTitle(Some(&NSString::from_str(title)));
    if let Some(directory) = current.and_then(Path::parent).and_then(NSURL::from_file_path) {
        panel.setDirectoryURL(Some(&directory));
    }
    if !extensions.is_empty() {
        let types = extensions.iter().map(|ext| NSString::from_str(ext)).collect::<Vec<_>>();
        // AppKit resolves the file extensions, including OM9's custom formats.
        #[allow(deprecated)]
        panel.setAllowedFileTypes(Some(&NSArray::from_retained_slice(&types)));
        panel.setAllowsOtherFileTypes(false);
    }
    if save {
        let stem = current.and_then(Path::file_stem).and_then(|stem| stem.to_str()).unwrap_or(default_name);
        let filename = format!("{stem}.{}", extensions.first().ok_or("missing save file type")?);
        panel.setNameFieldStringValue(&NSString::from_str(&filename));
    }
    let checkbox = alpha.as_deref().map(|enabled| {
        // SAFETY: There is no target or callback. AppKit owns the intrinsic toggle.
        let button = unsafe {
            NSButton::checkboxWithTitle_target_action(&NSString::from_str("Transparent background"), None, None, mtm)
        };
        button.setState(if *enabled { NSControlStateValueOn } else { NSControlStateValueOff });
        panel.setAccessoryView(Some(&button));
        button
    });
    match panel.runModal() {
        result if result == NSModalResponseCancel => Ok(None),
        result if result == NSModalResponseOK => {
            let path = panel.URL().and_then(|url| url.to_file_path()).ok_or("AppKit returned no file path")?;
            if let (Some(alpha), Some(checkbox)) = (alpha, checkbox) {
                *alpha = checkbox.state() == NSControlStateValueOn;
            }
            Ok(Some(path))
        }
        _ => Err("AppKit could not present the file picker".into()),
    }
}
