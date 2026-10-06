//! OM9-VIEWPORT-001: default host coordinate frames (XY, YZ, XZ).
#[derive(Clone, Copy)]
pub struct View {
    pub title: &'static str,
    pub rotation: [f64; 4],
    pub plane_rotation: [f64; 4],
    pub perspective: bool,
}
const XY: [f64; 4] = [0.0, 0.0, 0.0, 1.0];
const YZ: [f64; 4] = [0.5, 0.5, 0.5, 0.5];
const XZ: [f64; 4] = [
    std::f64::consts::FRAC_1_SQRT_2,
    0.0,
    0.0,
    std::f64::consts::FRAC_1_SQRT_2,
];
pub const VIEWS: [View; 4] = [
    View {
        title: "Looking Down",
        rotation: XY,
        plane_rotation: XY,
        perspective: false,
    },
    View {
        title: "Perspective",
        rotation: [0.424708, 0.17592, 0.339851, 0.820473],
        plane_rotation: XY,
        perspective: true,
    },
    View {
        title: "Side View",
        rotation: YZ,
        plane_rotation: YZ,
        perspective: false,
    },
    View {
        title: "Through Finger",
        rotation: XZ,
        plane_rotation: XZ,
        perspective: false,
    },
];
pub const ICONS: [&str; 2] = ["GridON", "InfoSettingsViewportTabsToggle"];
/// Minor lines every millimeter, major lines every five; zero is a colored axis.
pub fn grid_line_kind(index: i32) -> u32 {
    if index == 0 {
        2
    } else if index % 5 == 0 {
        1
    } else {
        0
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_grid_line_kind(index: i32) -> u32 {
    grid_line_kind(index)
}
pub fn command(icon: &str) -> Option<&'static str> {
    match icon {
        "ViewRestoreViewports" => Some("RestoreViewports"),
        "OthersViewSynchronizeViews" => Some("SynchronizeViews"),
        "ViewSetCameraCenterViewport" => Some("CenterViewport"),
        "ShowGrid" | "GridON" => Some("ShowGrid"),
        "ViewZoomZoomWindow" => Some("Zoom_Window"),
        "ViewZoomZoomDynamic" => Some("Zoom_Dynamic"),
        "ViewZoomZoomExtents" => Some("Zoom_Extents"),
        "ViewZoomZoomSelected" => Some("Zoom_Selected"),
        "ViewCaptureToFile" => Some("ViewCaptureToFile"),
        "ViewBackgroundBitmapPictureFrame" => Some("PictureFrame"),
        "OthersSetCrosshairs" => Some("Crosshairs"),
        "InfoSettingsViewportTabsToggle" => Some("ViewportTabs"),
        _ => None,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_view_rotation(slot: usize, axis: usize, plane: bool) -> f64 {
    VIEWS
        .get(slot)
        .and_then(|v| {
            if plane {
                v.plane_rotation.get(axis)
            } else {
                v.rotation.get(axis)
            }
        })
        .copied()
        .unwrap_or(f64::NAN)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_core_view_perspective(slot: usize) -> bool {
    VIEWS.get(slot).is_some_and(|v| v.perspective)
}
