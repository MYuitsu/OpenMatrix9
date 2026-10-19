// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compile-time palette of existing authored OpenMatrix9 SVGs. Source identity
//! follows Resources/menu/icons.ini and MainMenu.ini, with feature aliases from
//! the v2 FEATURES.json and tools/named-icon-bindings.json. No screenshot pixels,
//! manual/reference files, filesystem loading, or command dispatch live here.
use eframe::egui;
use std::collections::BTreeMap;

struct Icon {
    name: &'static str,
    command_key: &'static str,
    feature: &'static str,
    svg: &'static [u8],
}
macro_rules! icon {
    ($name:literal, $key:literal, $feature:literal, $source:literal) => {
        Icon {
            name: $name,
            command_key: $key,
            feature: $feature,
            svg: include_bytes!(concat!("../../../Resources/icons/", $source)),
        }
    };
}
const PALETTE: &[Icon] = &[
    icon!(
        "Line",
        "CurveLineSingleLine",
        "OM9-CURVE-002",
        "curve-minimal/CurveLineSingleLine.svg"
    ),
    icon!(
        "Point",
        "CurvePointObjectSinglePoint",
        "OM9-CURVE-034",
        "curve-minimal/CurvePointObjectSinglePoint.svg"
    ),
    icon!(
        "Circle",
        "CurveCircleCenterRadius",
        "OM9-CURVE-005",
        "curve-minimal/CurveCircleCenterRadius.svg"
    ),
    icon!(
        "Rectangle",
        "CurveRectangleCornertoCorner",
        "OM9-CURVE-004",
        "curve-minimal/CurveRectangleCornertoCorner.svg"
    ),
    icon!(
        "Polyline",
        "CurvePolylinePolyline",
        "OM9-CURVE-001",
        "curve-minimal/CurvePolylinePolyline.svg"
    ),
    icon!(
        "Interpolate",
        "CurveFreeFormInterpolatePoints",
        "",
        "curve-minimal/CurveFreeFormInterpolatePoints.svg"
    ),
    icon!(
        "Ellipse",
        "CurveEllipseFromCenter",
        "",
        "curve-minimal/CurveEllipseFromCenter.svg"
    ),
    icon!(
        "Offset",
        "CurveOffsetCurve",
        "",
        "curve-minimal/CurveOffsetCurve.svg"
    ),
    icon!(
        "ProjectCurve",
        "CurveCurveFromObjectsProject",
        "",
        "curve-minimal/CurveCurveFromObjectsProject.svg"
    ),
    icon!(
        "Box",
        "SolidBoxCornertoCornerHeight",
        "OM9-SOLID-012",
        "solid-minimal/SolidBoxCornertoCornerHeight.svg"
    ),
    icon!(
        "Sphere",
        "SolidSphereCenterRadius",
        "OM9-SOLID-014",
        "solid-minimal/SolidSphereCenterRadius.svg"
    ),
    icon!("Union", "SolidUnion", "", "solid-minimal/SolidUnion.svg"),
    icon!(
        "Difference",
        "SolidDifference",
        "",
        "solid-minimal/SolidDifference.svg"
    ),
    icon!(
        "Intersection",
        "SolidIntersection",
        "",
        "solid-minimal/SolidIntersection.svg"
    ),
    icon!(
        "Sweep",
        "SurfaceSweepSweep1Rail",
        "",
        "surface-minimal/SurfaceSweepSweep1Rail.svg"
    ),
    icon!("Loft", "SurfaceLoft", "", "surface-minimal/SurfaceLoft.svg"),
    icon!(
        "Revolve",
        "SurfaceRevolve",
        "",
        "surface-minimal/SurfaceRevolve.svg"
    ),
    icon!(
        "Move",
        "TopIconMove",
        "OM9-TOP11-006",
        "core-minimal/TopIconMove.svg"
    ),
    icon!(
        "Rotate",
        "TopIconRotate",
        "OM9-TOP11-007",
        "core-minimal/TopIconRotate.svg"
    ),
    icon!(
        "Scale",
        "TransformScaleScale3D",
        "OM9-TRANSFORM-001",
        "transform-minimal/TransformScaleScale3D.svg"
    ),
    icon!(
        "Mirror",
        "TopIconMirror",
        "OM9-TOP11-004",
        "core-minimal/TopIconMirror.svg"
    ),
    icon!(
        "Join",
        "TopIconJoin",
        "OM9-TOP11-008",
        "core-minimal/TopIconJoin.svg"
    ),
    icon!(
        "Split",
        "TopIconSplit",
        "OM9-TOP11-009",
        "core-minimal/TopIconSplit.svg"
    ),
    icon!("Trim", "TopIconTrim", "", "core-minimal/TopIconTrim.svg"),
    icon!(
        "Duplicate",
        "TopIconDuplicate",
        "",
        "core-minimal/TopIconDuplicate.svg"
    ),
    icon!(
        "EditPoints",
        "TopIconEditPointsOn",
        "",
        "core-minimal/TopIconEditPointsOn.svg"
    ),
    icon!("Undo", "Undo", "", "core-minimal/Undo.svg"),
    icon!("Redo", "Redo", "", "core-minimal/Redo.svg"),
    icon!(
        "Save",
        "FileSave",
        "OM9-FILE-005",
        "core-minimal/FileSave.svg"
    ),
    icon!(
        "Open",
        "FileOpen",
        "OM9-FILE-003",
        "core-minimal/FileOpen.svg"
    ),
    icon!("New", "FileNew", "OM9-FILE-002", "core-minimal/FileNew.svg"),
    icon!(
        "Wireframe",
        "Dial_Wireframe",
        "",
        "core-minimal/Dial_Wireframe.svg"
    ),
    icon!(
        "Shaded",
        "Dial_Shaded",
        "OM9-DISPLAY-008",
        "core-minimal/Dial_Shaded.svg"
    ),
    icon!(
        "SnapEnd",
        "ToolsObjectSnapEnd",
        "OM9-SNAP-002",
        "core-minimal/ToolsObjectSnapEnd.svg"
    ),
    icon!(
        "SnapMid",
        "ToolsObjectSnapMidpoint",
        "OM9-SNAP-003",
        "core-minimal/ToolsObjectSnapMidpoint.svg"
    ),
    icon!(
        "SnapCenter",
        "ToolsObjectSnapCenter",
        "OM9-SNAP-001",
        "core-minimal/ToolsObjectSnapCenter.svg"
    ),
    icon!(
        "SnapPoint",
        "ToolsObjectSnapPoint",
        "OM9-SNAP-007",
        "core-minimal/ToolsObjectSnapPoint.svg"
    ),
    icon!("Grid", "GridON", "", "core-minimal/GridON.svg"),
    icon!(
        "GridSnap",
        "GridSnapON",
        "OM9-SNAP-013",
        "core-minimal/GridSnapON.svg"
    ),
    icon!("Ortho", "OrthoSnapON", "", "core-minimal/OrthoSnapON.svg"),
    icon!(
        "Planar",
        "PlanarSnapON",
        "",
        "core-minimal/PlanarSnapON.svg"
    ),
    icon!(
        "Settings",
        "RhinoOptions",
        "",
        "core-minimal/RhinoOptions.svg"
    ),
    icon!(
        "Properties",
        "ObjectProperties",
        "",
        "core-minimal/ObjectProperties.svg"
    ),
    icon!(
        "Layers",
        "LayerVisibility",
        "OM9-LAYER-001",
        "core-minimal/LayerVisibility.svg"
    ),
    icon!("LayerLock", "LayerLock", "", "core-minimal/LayerLock.svg"),
    icon!("LayerShow", "LayerShow", "", "core-minimal/LayerShow.svg"),
    icon!("LayerHide", "LayerHide", "", "core-minimal/LayerHide.svg"),
    icon!(
        "Gem",
        "BuilderGemLoader",
        "OM9-GEM-001",
        "gems-minimal/BuilderGemLoader.svg"
    ),
    icon!(
        "Emerald",
        "BuilderEmerald",
        "OM9-GEM-013",
        "gems-minimal/BuilderEmerald.svg"
    ),
    icon!(
        "Builder",
        "BuilderEternity",
        "OM9-BUILDER-001",
        "builder-minimal/BuilderEternity.svg"
    ),
    icon!(
        "RingRail",
        "BuilderRingRail",
        "OM9-TOOLS-001",
        "tools-minimal/BuilderRingRail.svg"
    ),
    icon!(
        "Project",
        "ProjectManager",
        "OM9-PROJECT-001",
        "core-minimal/ProjectManager.svg"
    ),
    icon!("ProjectIn", "ProjectIn", "", "core-minimal/ProjectIn.svg"),
    icon!(
        "ProjectSave",
        "ProjectSave",
        "",
        "core-minimal/ProjectSave.svg"
    ),
    icon!(
        "Cutters",
        "BuilderGemCutter",
        "",
        "cutters-minimal/BuilderGemCutter.svg"
    ),
    icon!(
        "Render",
        "BuilderRenderVrayRenderStyler",
        "",
        "render-minimal/BuilderRenderVrayRenderStyler.svg"
    ),
    icon!("Delete", "Delete", "", "core-minimal/Delete.svg"),
    icon!(
        "Fit",
        "ViewZoomZoomExtents",
        "",
        "core-minimal/ViewZoomZoomExtents.svg"
    ),
];

fn lookup(name: &str) -> Option<&'static Icon> {
    let name = match name {
        "Curve" => "Polyline",
        "Surface" => "Sweep",
        "Solid" => "Box",
        "Transform" => "Move",
        "Gems" => "Gem",
        "Projects" => "Project",
        "SnapMidpoint" => "SnapMid",
        "End" => "SnapEnd",
        "Mid" => "SnapMid",
        "Center" => "SnapCenter",
        "Scale3D" => "Scale",
        other => other,
    };
    PALETTE.iter().find(|icon| {
        icon.name.eq_ignore_ascii_case(name)
            || icon.command_key.eq_ignore_ascii_case(name)
            || (!icon.feature.is_empty() && icon.feature.eq_ignore_ascii_case(name))
    })
}

/// Resolve a real menu command key, feature ID, or short palette label to its
/// decorative icon name. This makes no statement about command availability.
pub fn icon_name(name: &str) -> Option<&'static str> {
    lookup(name).map(|icon| icon.name)
}

/// Lazily owns a bounded palette of texture handles. A successful icon raster
/// is reused on every subsequent frame; failed authored SVGs cache a fallback.
#[derive(Default)]
pub struct Icons {
    textures: BTreeMap<&'static str, Option<egui::TextureHandle>>,
}
impl Icons {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn paint(&mut self, ui: &egui::Ui, rect: egui::Rect, name: &str) {
        if !rect.is_finite() || rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let Some(icon) = lookup(name) else {
            fallback(ui, rect);
            return;
        };
        let texture = self.textures.entry(icon.name).or_insert_with(|| {
            raster(icon.svg).map(|image| {
                ui.ctx().load_texture(
                    format!("om9-authored-icon-{}", icon.name),
                    image,
                    egui::TextureOptions::LINEAR,
                )
            })
        });
        if let Some(texture) = texture {
            let side = rect.width().min(rect.height());
            let square = egui::Rect::from_center_size(rect.center(), egui::vec2(side, side));
            ui.painter().image(
                texture.id(),
                square,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            fallback(ui, rect);
        }
    }
}
fn raster(svg: &[u8]) -> Option<egui::ColorImage> {
    const SIDE: u32 = 64;
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(SIDE, SIDE)?;
    let size = tree.size();
    let transform = resvg::tiny_skia::Transform::from_scale(
        SIDE as f32 / size.width(),
        SIDE as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(egui::ColorImage::from_rgba_premultiplied(
        [SIDE as usize, SIDE as usize],
        pixmap.data(),
    ))
}
fn fallback(ui: &egui::Ui, rect: egui::Rect) {
    // Neutral authored placeholder, deliberately without a command glyph.
    let side = rect.width().min(rect.height()) * 0.56;
    let square = egui::Rect::from_center_size(rect.center(), egui::vec2(side, side));
    ui.painter().rect_stroke(
        square,
        2.0,
        egui::Stroke::new(1.2, egui::Color32::from_gray(155)),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .circle_filled(rect.center(), 1.0, egui::Color32::from_gray(155));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_authored_svg_renders_visible_pixels() {
        for icon in PALETTE {
            let source = std::str::from_utf8(icon.svg).unwrap();
            assert!(!source.contains("<image") && !source.contains("<script"));
            let image = raster(icon.svg).unwrap_or_else(|| panic!("icon {}", icon.name));
            assert_eq!(image.size, [64, 64]);
            assert!(
                image.pixels.iter().any(|pixel| pixel.a() > 0),
                "{}",
                icon.name
            );
        }
    }
    #[test]
    fn feature_and_menu_aliases_share_cached_rasters_and_unknowns_are_neutral() {
        let context = egui::Context::default();
        let mut icons = Icons::new();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(24.0, 24.0));
            icons.paint(ui, rect, "Line");
            icons.paint(ui, rect, "OM9-CURVE-002");
            icons.paint(ui, rect, "CurveLineSingleLine");
            icons.paint(ui, rect, "not-an-icon");
        });
        output.textures_delta.clear(); // Headless fixture has no GPU texture consumer.
        assert_eq!(icons.textures.len(), 1);
        assert!(icons.textures["Line"].is_some());
        assert_eq!(icon_name("OM9-GEM-013"), Some("Emerald"));
        assert_eq!(icon_name("not-an-icon"), None);
    }
}
