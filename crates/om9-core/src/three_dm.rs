// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 3DM exchange command policy (`OM9-FILE-012`).
//!
//! Ported from the Phase2 Rust owner `rust/src/core_3dm.rs`. It is command metadata only: the
//! import/export/copy work itself lives in `om9-exchange` and the native adapters.

/// Menu/icon identifiers that dispatch the 3DM exchange commands.
pub const ICONS: [&str; 7] = [
    "FileImport3dm",
    "FileExport3dm",
    "FileCopy3dm",
    "FilePaste3dm",
    "FileCopySession3dm",
    "FileExportSession3dm",
    "FileExportLayerSelection3dm",
];

/// The command a menu icon dispatches, or `None` for an unknown icon.
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "FileImport3dm" => Some("Import3dm"),
        "FileExport3dm" => Some("Export3dm"),
        "FileCopy3dm" => Some("Copy3dm"),
        "FilePaste3dm" => Some("Paste3dm"),
        "FileCopySession3dm" => Some("CopySession3dm"),
        "FileExportSession3dm" => Some("ExportSession3dm"),
        "FileExportLayerSelection3dm" => Some("ExportLayerSelection3dm"),
        _ => None,
    }
}

/// The user-facing caption for a menu icon, or `None` for an unknown icon.
pub fn caption(icon: &str) -> Option<&'static str> {
    match icon {
        "FileImport3dm" => Some("Import 3DM..."),
        "FileExport3dm" => Some("Export Selected 3DM..."),
        "FileCopy3dm" => Some("Copy Rhino Geometry"),
        "FilePaste3dm" => Some("Paste Rhino Geometry"),
        "FileCopySession3dm" => Some("Copy Session 3DM"),
        "FileExportSession3dm" => Some("Export Session 3DM..."),
        "FileExportLayerSelection3dm" => Some("Export Selected with Layers 3DM..."),
        _ => None,
    }
}

/// The native operation code for a command name. Zero means "not a 3DM command".
pub fn operation(name: &str) -> u32 {
    match name {
        "Import3dm" => 1,
        "Export3dm" => 2,
        "Copy3dm" => 3,
        "Paste3dm" => 4,
        "CopySession3dm" => 5,
        "ExportSession3dm" => 6,
        "ExportLayerSelection3dm" => 7,
        _ => 0,
    }
}

/// The import scale: the document unit when valid, otherwise the explicit override.
///
/// A non-finite or non-positive value never becomes a scale; the caller gets `0.0` and must
/// refuse the import instead of guessing millimetres.
pub fn scale(unit_mm: f64, override_mm: f64) -> f64 {
    let value = if unit_mm.is_finite() && unit_mm > 0.0 {
        unit_mm
    } else {
        override_mm
    };
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

/// Geometry classes the 3DM exchange understands: 1 point, 2 curve, 3 surface, 4 mesh/B-Rep,
/// 5 instance/block. Unknown codes are refused rather than coerced.
pub fn type_supported(kind: u32) -> bool {
    (1..=5).contains(&kind)
}

/// Whether the command writes the open document. Export/copy commands only read it.
pub fn mutates_document(command: &str) -> bool {
    matches!(command, "Import3dm" | "Paste3dm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_caption_are_one_table() {
        for icon in ICONS {
            assert!(command(icon).is_some(), "{icon}");
            assert!(caption(icon).is_some(), "{icon}");
        }
        assert_eq!(command("FileCopy3dm"), Some("Copy3dm"));
        assert_eq!(caption("FilePaste3dm"), Some("Paste Rhino Geometry"));
        assert_eq!(command("FileUnknown"), None);
        assert_eq!(caption("FileUnknown"), None);
        assert_eq!(operation("ExportLayerSelection3dm"), 7);
        assert_eq!(operation("FileExport3dm"), 0);
    }

    #[test]
    fn units_never_guess_or_accept_invalid_scale() {
        assert_eq!(scale(25.4, 0.0), 25.4);
        assert_eq!(scale(1.0, 0.0), 1.0);
        assert_eq!(scale(0.0, 2.0), 2.0);
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(scale(value, 0.0), 0.0);
        }
        assert_eq!(scale(0.0, f64::NAN), 0.0);
    }

    #[test]
    fn only_declared_geometry_classes_are_supported() {
        for kind in 1..=5 {
            assert!(type_supported(kind));
        }
        assert!(!type_supported(0));
        assert!(!type_supported(6));
        assert!(!type_supported(99));
    }

    #[test]
    fn only_import_and_paste_mutate_the_document() {
        assert!(mutates_document("Import3dm"));
        assert!(mutates_document("Paste3dm"));
        for command in [
            "Export3dm",
            "Copy3dm",
            "CopySession3dm",
            "ExportSession3dm",
            "ExportLayerSelection3dm",
        ] {
            assert!(!mutates_document(command), "{command}");
        }
        assert!(!mutates_document("NotA3dmCommand"));
    }
}
