// SPDX-License-Identifier: LGPL-2.1-or-later
//! Select a path through the system GTK dialog. The application validates the result.
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy)]
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
    _alpha: Option<&mut bool>,
) -> Result<Option<PathBuf>, String> {
    let (title, filter, extension, default_name) = match kind {
        Kind::Document => (
            if save {
                "Save OM9 document"
            } else {
                "Open OM9 document"
            },
            "OM9 documents | *.om9",
            "om9",
            "Untitled",
        ),
        Kind::ThreeDm => (
            if save {
                "Export Rhino 3DM"
            } else {
                "Import Rhino 3DM"
            },
            "Rhino 3DM files | *.3dm",
            "3dm",
            "Untitled",
        ),
        Kind::Stl => ("Export STL", "STL files | *.stl", "stl", "Untitled"),
        Kind::ProjectArchive => (
            if save {
                "Export OM9 project"
            } else {
                "Import OM9 project"
            },
            "OM9 projects | *.om9project",
            "om9project",
            "Project",
        ),
        Kind::ProjectFolder => ("Select project folder", "", "", ""),
        Kind::Addon => ("Install OM9 add-on", "OM9 add-ons | *.zip", "zip", ""),
        Kind::ViewImage => (
            "Save Viewport Image",
            "Images | *.png *.jpg *.jpeg *.bmp *.pcx *.tif *.tiff *.tga",
            "png",
            "Viewport",
        ),
    };
    let mut command = Command::new("zenity");
    command.arg("--file-selection").arg("--title").arg(title);
    if matches!(kind, Kind::ProjectFolder) {
        command.arg("--directory");
    } else {
        command.arg("--file-filter").arg(filter);
    }
    if save {
        command.arg("--save");
        let directory = current.and_then(Path::parent).unwrap_or(Path::new(""));
        let stem = current
            .and_then(Path::file_stem)
            .unwrap_or(OsStr::new(default_name));
        let mut name = directory.join(stem);
        name.set_extension(extension);
        command.arg("--filename").arg(name);
    } else if let Some(directory) = current.and_then(Path::parent) {
        command.arg("--filename").arg(directory.join(""));
    }

    let response = command
        .output()
        .map_err(|error| format!("Linux file dialog (zenity): {error}"))?;
    match response.status.code() {
        Some(0) => {
            let path = response
                .stdout
                .strip_suffix(b"\n")
                .unwrap_or(&response.stdout);
            if path.is_empty() {
                return Err("Linux file dialog returned no path".into());
            }
            Ok(Some(PathBuf::from(OsString::from_vec(path.to_vec()))))
        }
        Some(1) => Ok(None),
        _ => Err("Linux file dialog did not complete".into()),
    }
}
