// Host choices for the OM9-IFACE-001 workspace slice and OM9-VIEW-006/007.
// This is not a recovered Matrix9 object-selection/deletion implementation.
pub const ICONS: [&str; 12] = [
    "SelectAll",
    "SelectNone",
    "Delete",
    "FitAll",
    "ViewZoomZoomSelected",
    "ViewIsometric",
    "ViewTop",
    "ViewFront",
    "ViewRight",
    "ViewLeft",
    "ViewRear",
    "ViewBottom",
];

pub fn native(icon: &str) -> Option<&'static str> {
    match icon {
        "SelectAll" => Some("OM9_SelectAllObjects"),
        "SelectNone" => Some("OM9_ClearSelection"),
        "Delete" => Some("Std_Delete"),
        "ViewZoomZoomSelected" => Some("Std_ViewFitSelection"),
        "ViewLeft" => Some("Std_ViewLeft"),
        "ViewRear" => Some("Std_ViewRear"),
        "ViewBottom" => Some("Std_ViewBottom"),
        "ViewIsometric" => Some("Std_ViewIsometric"),
        _ => None,
    }
}

pub fn caption(icon: &str) -> &str {
    match icon {
        "SelectAll" => "Select all objects in the active document",
        "SelectNone" => "Clear selection in the active document",
        "Delete" => "Delete selected objects (Undo available)",
        "FitAll" => "Fit all",
        "ViewZoomZoomSelected" => "Fit selected objects",
        "ViewIsometric" => "Isometric",
        "ViewTop" => "Top",
        "ViewFront" => "Front",
        "ViewRight" => "Right",
        "ViewLeft" => "Left",
        "ViewRear" => "Rear",
        "ViewBottom" => "Bottom",
        _ => icon,
    }
}

// Bitmask of observed host effects, independent of UI/history presentation.
// 1=document set changed; 2=save completed; 4=undo/redo changed;
// 8=selection changed; 16=objects removed; 32=view command dispatched.
pub fn success(native: &str, effects: u32) -> bool {
    let required = match native {
        "Std_New" | "Std_Open" => 1,
        "Std_Save" | "Std_SaveAs" => 2,
        "Std_Undo" | "Std_Redo" => 4,
        "OM9_SelectAllObjects" | "OM9_ClearSelection" => 8,
        "Std_Delete" => 16,
        "Std_ViewFitAll"
        | "Std_ViewFitSelection"
        | "Std_ViewFront"
        | "Std_ViewTop"
        | "Std_ViewRight"
        | "Std_ViewLeft"
        | "Std_ViewRear"
        | "Std_ViewBottom"
        | "Std_ViewIsometric" => 32,
        _ => return false,
    };
    effects & required != 0
}

// Host permission categories: document=1, view=2, selection=4.
pub fn permissions(native: &str) -> u32 {
    match native {
        "OM9_SelectAllObjects" | "OM9_ClearSelection" => 4,
        "Std_Delete" => 1,
        // File and undo/redo commands have special native task/edit policies.
        // Their own isActive checks decide availability; do not add AlterDoc gates.
        "Std_New" | "Std_Open" | "Std_Save" | "Std_SaveAs" | "Std_Undo" | "Std_Redo" => 0,
        id if id.starts_with("Std_View") => 2,
        _ => 0,
    }
}
