// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ported from `rust/tests/core_3dm.rs`: the 3DM command/icon policy and unit scale.

use om9_core::three_dm;

#[test]
fn units_never_guess_or_accept_invalid_scale() {
    assert_eq!(three_dm::scale(25.4, 0.0), 25.4);
    assert_eq!(three_dm::scale(1.0, 0.0), 1.0);
    assert_eq!(three_dm::scale(0.0, 2.0), 2.0);
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(three_dm::scale(value, 0.0), 0.0);
    }
    assert_eq!(three_dm::scale(0.0, f64::NAN), 0.0);
}

#[test]
fn import_export_and_clipboard_commands_share_one_table() {
    for (icon, command) in [
        ("FileImport3dm", "Import3dm"),
        ("FileExport3dm", "Export3dm"),
        ("FileCopy3dm", "Copy3dm"),
        ("FilePaste3dm", "Paste3dm"),
        ("FileCopySession3dm", "CopySession3dm"),
        ("FileExportSession3dm", "ExportSession3dm"),
        ("FileExportLayerSelection3dm", "ExportLayerSelection3dm"),
    ] {
        assert_eq!(three_dm::command(icon), Some(command));
        assert!(three_dm::caption(icon).is_some());
        assert_ne!(three_dm::operation(command), 0);
    }
    assert_eq!(three_dm::operation("Import3dm"), 1);
    assert_eq!(three_dm::operation("Export3dm"), 2);
    assert_eq!(three_dm::operation("CopySession3dm"), 5);
    assert_eq!(three_dm::operation("ExportSession3dm"), 6);
    assert_eq!(three_dm::operation("ExportLayerSelection3dm"), 7);
    assert_eq!(three_dm::operation("NotA3dmCommand"), 0);
}

#[test]
fn only_declared_geometry_classes_are_supported() {
    for kind in 1..=5 {
        assert!(three_dm::type_supported(kind));
    }
    assert!(!three_dm::type_supported(0));
    assert!(!three_dm::type_supported(6));
    assert!(!three_dm::type_supported(99));
}

#[test]
fn only_import_and_paste_mutate_the_document() {
    assert!(three_dm::mutates_document("Import3dm"));
    assert!(three_dm::mutates_document("Paste3dm"));
    for command in [
        "Export3dm",
        "Copy3dm",
        "CopySession3dm",
        "ExportSession3dm",
        "ExportLayerSelection3dm",
    ] {
        assert!(!three_dm::mutates_document(command), "{command}");
    }
    assert!(!three_dm::mutates_document("NotA3dmCommand"));
}
