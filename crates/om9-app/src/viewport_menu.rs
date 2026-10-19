// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix9 S6 vocabulary and scoped view actions, never renderer certification.
use om9_core::{
    Error, Result,
    identity::{DocumentId, SessionId},
    model::Document,
    transaction::OperationContext,
};
use om9_scene::workspace::{DisplayMode, ViewId, ViewState, Workspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayPreset {
    Wireframe,
    Shaded,
    Rendered,
    Ghosted,
    XRay,
    Technical,
    ArtColor,
    ArtColorWires,
    ClayooSculpt,
    DetectAll,
    DetectBackfaces,
    DetectNakedEdges,
    DetectUvNormals,
    Floorplan,
    Ice,
    Legacy,
    Machine,
    Pastel,
    Plastic,
    Presentation,
    ShinyPlastic,
    SimpleShade,
    TechWhite,
    Vivid,
    WireRender,
    WorkingRender,
    WorkingShade,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Maximize,
    Preset(DisplayPreset),
    PrintPreview,
    FlatShade,
    ShadeSelectedOnly,
    PanZoomRotate,
    SetView,
    SetCPlane,
    SetCamera,
    ActiveViewport,
    ViewportLayout,
    BackgroundBitmap,
    GridOptions,
    Capture,
    CloseViewport,
    ClippingPlane,
    RefreshShade,
    DisplayOptions,
    ViewportProperties,
}
#[derive(Debug, Clone, Copy)]
pub struct MenuEntry {
    pub label: &'static str,
    pub action: Option<MenuAction>,
    pub submenu: bool,
}
const fn item(label: &'static str, action: MenuAction) -> MenuEntry {
    MenuEntry {
        label,
        action: Some(action),
        submenu: false,
    }
}
const fn submenu(label: &'static str, action: MenuAction) -> MenuEntry {
    MenuEntry {
        label,
        action: Some(action),
        submenu: true,
    }
}
const SEP: MenuEntry = MenuEntry {
    label: "",
    action: None,
    submenu: false,
};
use DisplayPreset as P;
use MenuAction as A;
pub const ENTRIES: &[MenuEntry] = &[
    item("Maximize", A::Maximize),
    SEP,
    item("Wireframe", A::Preset(P::Wireframe)),
    item("Shaded", A::Preset(P::Shaded)),
    item("Rendered", A::Preset(P::Rendered)),
    item("Ghosted", A::Preset(P::Ghosted)),
    item("X-Ray", A::Preset(P::XRay)),
    item("Technical", A::Preset(P::Technical)),
    item("Art Color", A::Preset(P::ArtColor)),
    item("Art Color Wires", A::Preset(P::ArtColorWires)),
    item("ClayooSculpt", A::Preset(P::ClayooSculpt)),
    item("Detect All", A::Preset(P::DetectAll)),
    item("Detect Backfaces", A::Preset(P::DetectBackfaces)),
    item("Detect Naked Edges", A::Preset(P::DetectNakedEdges)),
    item("Detect UV Normals", A::Preset(P::DetectUvNormals)),
    item("Floorplan", A::Preset(P::Floorplan)),
    item("Ice", A::Preset(P::Ice)),
    item("Legacy", A::Preset(P::Legacy)),
    item("Machine", A::Preset(P::Machine)),
    item("Pastel", A::Preset(P::Pastel)),
    item("Plastic", A::Preset(P::Plastic)),
    item("Presentation", A::Preset(P::Presentation)),
    item("Shiny Plastic", A::Preset(P::ShinyPlastic)),
    item("Simple Shade", A::Preset(P::SimpleShade)),
    item("Tech White", A::Preset(P::TechWhite)),
    item("Vivid", A::Preset(P::Vivid)),
    item("Wire Render", A::Preset(P::WireRender)),
    item("Working Render", A::Preset(P::WorkingRender)),
    item("Working Shade", A::Preset(P::WorkingShade)),
    SEP,
    item("Print Preview", A::PrintPreview),
    item("Flat Shade", A::FlatShade),
    item("Shade Selected Objects Only", A::ShadeSelectedOnly),
    SEP,
    submenu("Pan, Zoom, and Rotate", A::PanZoomRotate),
    SEP,
    submenu("Set View", A::SetView),
    submenu("Set CPlane", A::SetCPlane),
    submenu("Set Camera", A::SetCamera),
    SEP,
    submenu("Active Viewport", A::ActiveViewport),
    submenu("Viewport Layout", A::ViewportLayout),
    SEP,
    submenu("Background Bitmap", A::BackgroundBitmap),
    item("Grid Options...", A::GridOptions),
    submenu("Capture", A::Capture),
    SEP,
    item("Close Viewport", A::CloseViewport),
    SEP,
    item("Clipping plane", A::ClippingPlane),
    item("Refresh Shade", A::RefreshShade),
    item("Display Options...", A::DisplayOptions),
    item("Viewport Properties...", A::ViewportProperties),
];
impl MenuAction {
    pub fn availability(self) -> Result<()> {
        match self {
            Self::Maximize | Self::Preset(P::Wireframe) => Ok(()),
            Self::Preset(P::Shaded) => Err(Error::Unsupported(
                "Shaded display adapter not yet accepted",
            )),
            Self::Preset(_) => Err(Error::Unsupported(
                "Display preset/backend not yet accepted",
            )),
            _ => Err(Error::Unsupported("Viewport operation not yet accepted")),
        }
    }
    pub fn is_checked(self, mode: DisplayMode) -> bool {
        matches!(
            (self, mode),
            (Self::Preset(P::Wireframe), DisplayMode::Wireframe)
                | (Self::Preset(P::Shaded), DisplayMode::Shaded)
        )
    }
}
#[derive(Debug, Clone)]
pub struct ViewMenuCapture {
    context: OperationContext,
    document: (DocumentId, SessionId, u64, u64),
    view: ViewState,
    maximized: Option<ViewId>,
    layout_revision: u64,
    layout_owner: SessionId,
    action_generation: u64,
}
impl ViewMenuCapture {
    pub(crate) fn new(
        context: OperationContext,
        d: &Document,
        view: &ViewState,
        workspace: &Workspace,
        action_generation: u64,
    ) -> Self {
        Self {
            context,
            document: (d.id, d.owner_session, d.epoch, d.session_generation),
            view: view.clone(),
            maximized: workspace.maximized,
            layout_revision: workspace.layout_revision(),
            layout_owner: workspace.layout_owner(),
            action_generation,
        }
    }
    pub fn target(&self) -> ViewId {
        self.view.id
    }
    pub(crate) fn validate(
        &self,
        context: OperationContext,
        d: &Document,
        workspace: &Workspace,
        action_generation: u64,
    ) -> Result<()> {
        if self.context != context
            || self.document != (d.id, d.owner_session, d.epoch, d.session_generation)
        {
            return Err(Error::Stale("viewport menu document owner/context"));
        }
        if self.action_generation != action_generation
            || workspace.views.get(&self.view.id) != Some(&self.view)
            || workspace.maximized != self.maximized
            || workspace.layout_revision() != self.layout_revision
            || workspace.layout_owner() != self.layout_owner
        {
            return Err(Error::Stale("viewport menu view instance/state"));
        }
        Ok(())
    }
}
