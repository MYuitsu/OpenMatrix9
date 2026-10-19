// SPDX-License-Identifier: LGPL-2.1-or-later
//! Workbench docking regions (plan module `OM9-L2-01-01`, task `OM9-L3-01-01-002`).
//!
//! A pure, egui-free model of the Matrix9 window: left rail, a center holding
//! the four viewports (or one maximized viewport), an optional right
//! properties dock and the bars that frame the center (menu bar, command input,
//! toolbar, status bar).
//!
//! Boundaries:
//! - Sizes and constants come from [`DockLayout::resolve`]; this module adds
//!   geometry, explicit failures and the four-viewport tiling.
//! - It takes `&DockLayout` and plain values only. It holds no document,
//!   transaction, scene or render-cache reference, so a failure cannot change
//!   them by construction.
//! - Persistence (L3-005), restore default (L3-004), reorder/float (L3-003)
//!   and the Builder slot (L3-008) are out of scope; [`RegionPlan`] is only
//!   `serde`-ready for them.
//! - `OM9-L2-01-02` (command dispatch) and `OM9-L2-01-03` (document/scene)
//!   are not implemented. [`CommandInputHost`] and [`ViewportSceneHost`] are
//!   the minimal typed seams toward them and are tested against stubs.
//!
//! The place-kind vocabulary (left, right, centre, top, bottom) and the "host
//! fills the container" seam were read in the pinned QymCAD shell
//! (`qymcad-shell`, revision 746a17f). No upstream text is copied; legal reuse
//! approval is pending the human M00 review.
use crate::dock_layout::{
    CANVAS_MIN_HEIGHT, COMMAND_MAX, COMMAND_MIN, DockLayout, LEFT_RAIL_DEFAULT, LEFT_RAIL_MAX,
    LEFT_RAIL_MIN, NARROW_WINDOW_WIDTH, PanelId, STATUS_BAR_HEIGHT, TOOLBAR_HEIGHT,
};

use om9_scene::workspace::ViewId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Height of the egui "menu" panel in `ui.rs`.
pub const MENU_BAR_HEIGHT: f32 = 21.;
/// Gap between viewports in the 2x2 grid.
pub const VIEW_GAP: f32 = 2.;
/// Larger windows are rejected as [`RegionError::InvalidWindow`].
pub const MAX_WINDOW_DIMENSION: f32 = 100_000.;
/// Smallest window the product supports, in logical points (user decision
/// 2026-10-16: the smallest supported device screen is 1366x768). The real
/// window enforces it (capped to the usable screen on smaller logical screens,
/// see `startup::effective_min_size`).
pub const SUPPORTED_MIN_WINDOW: WindowSize = WindowSize::new(1280., 720.);
/// Planner floor in logical points. It sits below every size the window level
/// can produce (a 1366x768 screen at 150% DPI caps the window at about
/// 895x432), so the capped case still plans. Below it the plan is refused with
/// [`RegionError::WindowTooSmall`] and the last good plan is kept.
pub const PLAN_FLOOR_WINDOW: WindowSize = WindowSize::new(800., 400.);
/// Tolerance for float comparisons in [`RegionPlan::validate`].
const EPS: f32 = 0.01;

/// Window size in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl WindowSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<[f32; 2]> for WindowSize {
    fn from(size: [f32; 2]) -> Self {
        Self::new(size[0], size[1])
    }
}

/// Axis-aligned rectangle in window pixels, origin top-left.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RegionRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
    }

    /// True when the interiors intersect. Empty rectangles never overlap.
    pub fn overlaps(&self, other: &Self) -> bool {
        if self.width <= EPS || self.height <= EPS || other.width <= EPS || other.height <= EPS {
            return false;
        }
        self.x < other.right() - EPS
            && other.x < self.right() - EPS
            && self.y < other.bottom() - EPS
            && other.y < self.bottom() - EPS
    }

    /// True when `inner` lies inside `self` (within tolerance).
    pub fn contains(&self, inner: &Self) -> bool {
        inner.x >= self.x - EPS
            && inner.y >= self.y - EPS
            && inner.right() <= self.right() + EPS
            && inner.bottom() <= self.bottom() + EPS
    }
}

/// The regions of the workbench window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegionKind {
    MenuBar,
    LeftRail,
    Center,
    RightDock,
    CommandInput,
    Toolbar,
    StatusBar,
}

/// One viewport and the rectangle it is drawn into.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewSlot {
    pub view: ViewId,
    pub rect: RegionRect,
}

/// Why the optional right dock did not appear although a menu is docked there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YieldReason {
    /// The window is narrower than [`NARROW_WINDOW_WIDTH`].
    NarrowWindow,
    /// The window is wide enough but left rail + dock + canvas minimum do not fit.
    /// Kept for API and fixture stability: since the dock threshold is 1200 and
    /// the planner floor is 800 this cannot happen at any plannable size (see the
    /// `dead_notes_stay_unreachable` test).
    NotEnoughRoom,
}

/// A visible, non-silent record of a defined fallback.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RegionNote {
    RightDockYielded {
        reason: YieldReason,
    },
    LeftRailShrunk {
        from: f32,
        to: f32,
    },
    /// The layout marked the command input hidden; it is critical, so it is kept.
    CommandInputKept,
}

impl fmt::Display for RegionNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RightDockYielded {
                reason: YieldReason::NarrowWindow,
            } => f.write_str("right dock hidden: window is narrow"),
            Self::RightDockYielded {
                reason: YieldReason::NotEnoughRoom,
            } => f.write_str("right dock hidden: not enough room"),
            Self::LeftRailShrunk { from, to } => {
                write!(f, "left rail narrowed from {from} to {to} px")
            }
            Self::CommandInputKept => f.write_str("command input kept visible"),
        }
    }
}

/// One resolved arrangement of the window. A value: equal inputs give equal plans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionPlan {
    pub window: WindowSize,
    pub menu_bar: RegionRect,
    /// `None` when the left rail is hidden.
    pub left_rail: Option<RegionRect>,
    pub center: RegionRect,
    /// 1..=4 slots tiling `center`.
    pub views: Vec<ViewSlot>,
    /// `None` when hidden or yielded (see `notes`).
    pub right_dock: Option<RegionRect>,
    /// Always present, never shorter than `COMMAND_MIN`.
    pub command_input: RegionRect,
    pub toolbar: RegionRect,
    pub status_bar: RegionRect,
    pub notes: Vec<RegionNote>,
}

/// Why a plan could not be produced or is inconsistent.
#[derive(Debug, Clone, PartialEq)]
pub enum RegionError {
    /// Non-finite, not positive, or larger than [`MAX_WINDOW_DIMENSION`].
    InvalidWindow {
        width: f32,
        height: f32,
    },
    WindowTooSmall {
        width: f32,
        height: f32,
        min_width: f32,
        min_height: f32,
    },
    CenterTooSmall {
        width: f32,
        height: f32,
    },
    /// The command-input host seam reported not ready.
    CommandInputUnavailable,
    /// Zero views, more than four, or a duplicate.
    ViewSetInvalid {
        count: usize,
    },
    Overlap {
        a: RegionKind,
        b: RegionKind,
    },
    OutOfBounds {
        kind: RegionKind,
    },
    NonFinite {
        kind: RegionKind,
    },
    CommandInputTooSmall {
        height: f32,
    },
    ViewSlotInvalid {
        view: ViewId,
    },
}

impl RegionError {
    /// Static label used when the error crosses into `om9_core::Error`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::InvalidWindow { .. } => "workbench window size",
            Self::WindowTooSmall { .. } => "workbench window (below minimum size)",
            Self::CenterTooSmall { .. } => "viewport area size",
            Self::CommandInputUnavailable => "command input (not ready)",
            Self::ViewSetInvalid { .. } => "viewport set",
            Self::Overlap { .. } => "workbench regions (overlap)",
            Self::OutOfBounds { .. } => "workbench regions (outside window)",
            Self::NonFinite { .. } => "workbench region size",
            Self::CommandInputTooSmall { .. } => "command input (too small)",
            Self::ViewSlotInvalid { .. } => "viewport slot",
        }
    }
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindow { width, height } => {
                write!(f, "window size {width}x{height} is not usable")
            }
            Self::WindowTooSmall {
                width,
                height,
                min_width,
                min_height,
            } => write!(
                f,
                "window {width}x{height} is below the minimum {min_width}x{min_height}; layout kept"
            ),
            Self::CenterTooSmall { width, height } => {
                write!(f, "viewport area {width}x{height} is too small to tile")
            }
            Self::CommandInputUnavailable => f.write_str("command input is not ready"),
            Self::ViewSetInvalid { count } => {
                write!(f, "{count} viewports given; expected 1 to 4 distinct views")
            }
            Self::Overlap { a, b } => write!(f, "regions {a:?} and {b:?} overlap"),
            Self::OutOfBounds { kind } => write!(f, "region {kind:?} lies outside the window"),
            Self::NonFinite { kind } => write!(f, "region {kind:?} has a non-finite size"),
            Self::CommandInputTooSmall { height } => {
                write!(f, "command input {height} px is below {COMMAND_MIN} px")
            }
            Self::ViewSlotInvalid { view } => {
                write!(
                    f,
                    "viewport {view:?} lies outside the viewport area or overlaps"
                )
            }
        }
    }
}

impl std::error::Error for RegionError {}

impl From<RegionError> for om9_core::Error {
    fn from(error: RegionError) -> Self {
        om9_core::Error::Invalid(error.label())
    }
}

fn valid_dimension(value: f32) -> bool {
    value.is_finite() && value > 0. && value <= MAX_WINDOW_DIMENSION
}

fn check_view_set(views: &[ViewId]) -> Result<(), RegionError> {
    let count = views.len();
    if count == 0 || count > ViewId::ALL.len() {
        return Err(RegionError::ViewSetInvalid { count });
    }
    for (index, view) in views.iter().enumerate() {
        if views[..index].contains(view) {
            return Err(RegionError::ViewSetInvalid { count });
        }
    }
    Ok(())
}

impl RegionPlan {
    /// Assert the invariants of a plan: finite, inside the window, no overlap,
    /// a command input of at least `COMMAND_MIN`, and view slots tiling the center.
    pub fn validate(&self) -> Result<(), RegionError> {
        let (width, height) = (self.window.width, self.window.height);
        if !valid_dimension(width) || !valid_dimension(height) {
            return Err(RegionError::InvalidWindow { width, height });
        }
        let window = RegionRect::new(0., 0., width, height);
        let mut named: Vec<(RegionKind, RegionRect)> = vec![
            (RegionKind::MenuBar, self.menu_bar),
            (RegionKind::Center, self.center),
            (RegionKind::CommandInput, self.command_input),
            (RegionKind::Toolbar, self.toolbar),
            (RegionKind::StatusBar, self.status_bar),
        ];
        if let Some(rect) = self.left_rail {
            named.push((RegionKind::LeftRail, rect));
        }
        if let Some(rect) = self.right_dock {
            named.push((RegionKind::RightDock, rect));
        }
        for (kind, rect) in &named {
            if !rect.is_finite() {
                return Err(RegionError::NonFinite { kind: *kind });
            }
            if rect.width < 0. || rect.height < 0. || !window.contains(rect) {
                return Err(RegionError::OutOfBounds { kind: *kind });
            }
        }
        for (index, (a, rect_a)) in named.iter().enumerate() {
            for (b, rect_b) in &named[index + 1..] {
                if rect_a.overlaps(rect_b) {
                    return Err(RegionError::Overlap { a: *a, b: *b });
                }
            }
        }
        if self.command_input.height < COMMAND_MIN - EPS {
            return Err(RegionError::CommandInputTooSmall {
                height: self.command_input.height,
            });
        }
        let views: Vec<ViewId> = self.views.iter().map(|slot| slot.view).collect();
        check_view_set(&views)?;
        for (index, slot) in self.views.iter().enumerate() {
            let sound = slot.rect.is_finite()
                && slot.rect.width >= 0.
                && slot.rect.height >= 0.
                && self.center.contains(&slot.rect);
            if !sound
                || self.views[..index]
                    .iter()
                    .any(|other| other.rect.overlaps(&slot.rect))
            {
                return Err(RegionError::ViewSlotInvalid { view: slot.view });
            }
        }
        Ok(())
    }
}

/// Place 1..=4 distinct views in `center`: one view fills it; two to four use a
/// 2x2 grid in the order given with a [`VIEW_GAP`] pixel gap.
pub fn tile_views(center: RegionRect, views: &[ViewId]) -> Result<Vec<ViewSlot>, RegionError> {
    check_view_set(views)?;
    let too_small = RegionError::CenterTooSmall {
        width: center.width,
        height: center.height,
    };
    if !center.is_finite() || center.width < 0. || center.height < 0. {
        return Err(too_small);
    }
    if let [view] = views {
        return Ok(vec![ViewSlot {
            view: *view,
            rect: center,
        }]);
    }
    if center.width < VIEW_GAP || center.height < VIEW_GAP {
        return Err(too_small);
    }
    let width = (center.width - VIEW_GAP) / 2.;
    let height = (center.height - VIEW_GAP) / 2.;
    Ok(views
        .iter()
        .enumerate()
        .map(|(index, view)| {
            let column = (index % 2) as f32;
            let row = (index / 2) as f32;
            ViewSlot {
                view: *view,
                rect: RegionRect::new(
                    center.x + column * (width + VIEW_GAP),
                    center.y + row * (height + VIEW_GAP),
                    width,
                    height,
                ),
            }
        })
        .collect())
}

/// Resolve the window into regions. Pure: nothing in `layout` is mutated.
///
/// Widths come from [`DockLayout::resolve`]. Heights are stacked in the order
/// the egui panels are declared (menu, command, toolbars, canvas, status), so
/// the plan matches the frame. Every successful plan has passed
/// [`RegionPlan::validate`].
pub fn compute_regions(
    layout: &DockLayout,
    window: WindowSize,
    views: &[ViewId],
) -> Result<RegionPlan, RegionError> {
    let (width, height) = (window.width, window.height);
    if !valid_dimension(width) || !valid_dimension(height) {
        return Err(RegionError::InvalidWindow { width, height });
    }
    check_view_set(views)?;

    let left_state = layout.state(PanelId::LeftRail);
    // The floor does not depend on the left rail: every size the window level
    // allows already holds the rail, the command input and the canvas minimum.
    let (min_width, min_height) = (PLAN_FLOOR_WINDOW.width, PLAN_FLOOR_WINDOW.height);
    if width < min_width || height < min_height {
        return Err(RegionError::WindowTooSmall {
            width,
            height,
            min_width,
            min_height,
        });
    }

    let resolved = layout.resolve([width, height]);
    if !resolved.left_rail.is_finite() {
        return Err(RegionError::NonFinite {
            kind: RegionKind::LeftRail,
        });
    }
    if !resolved.right_dock.is_finite() {
        return Err(RegionError::NonFinite {
            kind: RegionKind::RightDock,
        });
    }
    let command_state = layout.state(PanelId::CommandLine);
    if !command_state.size.is_finite() {
        return Err(RegionError::NonFinite {
            kind: RegionKind::CommandInput,
        });
    }

    let mut notes = Vec::new();
    let left = resolved.left_rail;
    if left_state.visible {
        let requested = if left_state.size.is_finite() {
            left_state.size.clamp(LEFT_RAIL_MIN, LEFT_RAIL_MAX)
        } else {
            LEFT_RAIL_DEFAULT
        };
        if left + EPS < requested {
            notes.push(RegionNote::LeftRailShrunk {
                from: requested,
                to: left,
            });
        }
    }
    let right = resolved.right_dock;
    let right_requested = layout
        .right_sections()
        .into_iter()
        .any(|id| layout.section(id).visible);
    if right_requested && right <= 0. {
        let reason = if width < NARROW_WINDOW_WIDTH {
            YieldReason::NarrowWindow
        } else {
            YieldReason::NotEnoughRoom
        };
        notes.push(RegionNote::RightDockYielded { reason });
    }
    if !command_state.visible {
        notes.push(RegionNote::CommandInputKept);
    }

    // The command input yields height to the canvas only down to COMMAND_MIN.
    let reserved = MENU_BAR_HEIGHT + TOOLBAR_HEIGHT + STATUS_BAR_HEIGHT + CANVAS_MIN_HEIGHT;
    let command_height = command_state
        .size
        .clamp(COMMAND_MIN, COMMAND_MAX)
        .min(height - reserved)
        .max(COMMAND_MIN);

    let canvas_top = MENU_BAR_HEIGHT + command_height + TOOLBAR_HEIGHT;
    let canvas_height = height - STATUS_BAR_HEIGHT - canvas_top;
    let center = RegionRect::new(left, canvas_top, width - left - right, canvas_height);
    let plan = RegionPlan {
        window,
        menu_bar: RegionRect::new(0., 0., width, MENU_BAR_HEIGHT),
        left_rail: (left > 0.)
            .then(|| RegionRect::new(0., MENU_BAR_HEIGHT, left, height - MENU_BAR_HEIGHT)),
        center,
        views: tile_views(center, views)?,
        right_dock: (right > 0.)
            .then(|| RegionRect::new(width - right, canvas_top, right, height - canvas_top)),
        command_input: RegionRect::new(left, MENU_BAR_HEIGHT, width - left, command_height),
        toolbar: RegionRect::new(
            left,
            MENU_BAR_HEIGHT + command_height,
            width - left,
            TOOLBAR_HEIGHT,
        ),
        status_bar: RegionRect::new(
            left,
            height - STATUS_BAR_HEIGHT,
            width - left - right,
            STATUS_BAR_HEIGHT,
        ),
        notes,
    };
    plan.validate()?;
    Ok(plan)
}

/// What the docking regions need to know from the command owner
/// (`OM9-L2-01-02`, not implemented here).
pub trait CommandInputHost {
    /// True when the CMD frame is attached and can take input.
    fn command_input_ready(&self) -> bool;
}

/// What the docking regions need to know from the document/scene owner
/// (`OM9-L2-01-03`, not implemented here).
pub trait ViewportSceneHost {
    /// The views to place, in drawing order.
    fn visible_views(&self) -> Vec<ViewId>;
}

impl ViewportSceneHost for om9_scene::workspace::Workspace {
    fn visible_views(&self) -> Vec<ViewId> {
        om9_scene::workspace::Workspace::visible_views(self)
    }
}

/// [`compute_regions`] fed from the two host seams. The command input is checked
/// first: with no usable command input there is nothing to protect or place.
pub fn compute_regions_for(
    layout: &DockLayout,
    window: WindowSize,
    scene: &impl ViewportSceneHost,
    command: &impl CommandInputHost,
) -> Result<RegionPlan, RegionError> {
    if !command.command_input_ready() {
        return Err(RegionError::CommandInputUnavailable);
    }
    let views = scene.visible_views();
    compute_regions(layout, window, &views)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock_layout::{CANVAS_MIN_WIDTH, DockSide, SectionId};
    use std::cell::Cell;

    const ALL_VIEWS: [ViewId; 4] = ViewId::ALL;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> RegionRect {
        RegionRect::new(x, y, width, height)
    }

    fn window(width: f32, height: f32) -> WindowSize {
        WindowSize::new(width, height)
    }

    fn layout_with_right_dock() -> DockLayout {
        let mut layout = DockLayout::matrix_default();
        layout
            .dock_section(SectionId::MainMenu, DockSide::Right)
            .unwrap();
        layout
    }

    fn json(layout: &DockLayout) -> String {
        layout.to_json().unwrap()
    }

    fn view_rect(plan: &RegionPlan, view: ViewId) -> RegionRect {
        plan.views
            .iter()
            .find(|slot| slot.view == view)
            .map(|slot| slot.rect)
            .expect("view slot present")
    }

    struct StubScene {
        views: Vec<ViewId>,
        calls: Cell<u32>,
    }

    impl ViewportSceneHost for StubScene {
        fn visible_views(&self) -> Vec<ViewId> {
            self.calls.set(self.calls.get() + 1);
            self.views.clone()
        }
    }

    struct StubCommand {
        ready: bool,
        calls: Cell<u32>,
    }

    impl CommandInputHost for StubCommand {
        fn command_input_ready(&self) -> bool {
            self.calls.set(self.calls.get() + 1);
            self.ready
        }
    }

    // (a) valid input ------------------------------------------------------

    #[test]
    fn valid_layout_exposes_left_center_four_views_and_optional_right_dock() {
        let layout = layout_with_right_dock();
        let plan = compute_regions(&layout, window(1600., 1000.), &ALL_VIEWS).unwrap();
        // Hand-computed from the documented constants (see positive.json).
        assert_eq!(plan.menu_bar, rect(0., 0., 1600., 21.));
        assert_eq!(plan.left_rail, Some(rect(0., 21., 300., 979.)));
        assert_eq!(plan.right_dock, Some(rect(1300., 178., 300., 822.)));
        assert_eq!(plan.command_input, rect(300., 21., 1300., 90.));
        assert_eq!(plan.toolbar, rect(300., 111., 1300., 67.));
        assert_eq!(plan.status_bar, rect(300., 980., 1000., 20.));
        assert_eq!(plan.center, rect(300., 178., 1000., 802.));
        assert_eq!(plan.views.len(), 4);
        assert_eq!(
            view_rect(&plan, ViewId::LookingDown),
            rect(300., 178., 499., 400.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::Perspective),
            rect(801., 178., 499., 400.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::ThroughFinger),
            rect(300., 580., 499., 400.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::SideView),
            rect(801., 580., 499., 400.)
        );
        assert!(plan.notes.is_empty());
        plan.validate().unwrap();
    }

    #[test]
    fn without_a_docked_menu_the_right_dock_is_absent() {
        let plan = compute_regions(
            &DockLayout::matrix_default(),
            window(1280., 900.),
            &ALL_VIEWS,
        )
        .unwrap();
        assert_eq!(plan.right_dock, None);
        assert_eq!(plan.center, rect(300., 178., 980., 702.));
        assert_eq!(
            view_rect(&plan, ViewId::SideView),
            rect(791., 530., 489., 350.)
        );
    }

    #[test]
    fn one_maximized_view_fills_the_center() {
        let plan = compute_regions(
            &DockLayout::matrix_default(),
            window(1280., 900.),
            &[ViewId::Perspective],
        )
        .unwrap();
        assert_eq!(plan.views.len(), 1);
        assert_eq!(plan.views[0].view, ViewId::Perspective);
        assert_eq!(plan.views[0].rect, plan.center);
        plan.validate().unwrap();
    }

    #[test]
    fn floor_800x400_keeps_every_minimum() {
        let plan = compute_regions(
            &DockLayout::matrix_default(),
            window(800., 400.),
            &ALL_VIEWS,
        )
        .unwrap();
        assert_eq!(plan.menu_bar, rect(0., 0., 800., 21.));
        assert_eq!(plan.left_rail, Some(rect(0., 21., 300., 379.)));
        assert_eq!(plan.command_input, rect(300., 21., 500., 90.));
        assert_eq!(plan.toolbar, rect(300., 111., 500., 67.));
        assert_eq!(plan.center, rect(300., 178., 500., 202.));
        assert_eq!(plan.status_bar, rect(300., 380., 500., 20.));
        assert_eq!(
            view_rect(&plan, ViewId::LookingDown),
            rect(300., 178., 249., 100.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::Perspective),
            rect(551., 178., 249., 100.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::ThroughFinger),
            rect(300., 280., 249., 100.)
        );
        assert_eq!(
            view_rect(&plan, ViewId::SideView),
            rect(551., 280., 249., 100.)
        );
        // 300 + 320 fits in 800, so the rail keeps its stored width.
        assert!(plan.notes.is_empty());
        plan.validate().unwrap();
    }

    #[test]
    fn supported_minimum_and_floor_are_the_documented_numbers() {
        assert_eq!(SUPPORTED_MIN_WINDOW, WindowSize::new(1280., 720.));
        assert_eq!(PLAN_FLOOR_WINDOW, WindowSize::new(800., 400.));
        // The relations between the numbers are compile-time assertions below.
    }

    // The floor holds every region at its own minimum with the rail at its
    // maximum, the supported minimum holds the floor, and the dock shows at the
    // supported minimum.
    const _: () = assert!(
        PLAN_FLOOR_WINDOW.height
            >= MENU_BAR_HEIGHT
                + COMMAND_MIN
                + TOOLBAR_HEIGHT
                + STATUS_BAR_HEIGHT
                + CANVAS_MIN_HEIGHT
    );
    const _: () = assert!(PLAN_FLOOR_WINDOW.width >= LEFT_RAIL_MAX + CANVAS_MIN_WIDTH);
    const _: () = assert!(SUPPORTED_MIN_WINDOW.width >= PLAN_FLOOR_WINDOW.width);
    const _: () = assert!(SUPPORTED_MIN_WINDOW.height >= PLAN_FLOOR_WINDOW.height);
    const _: () = assert!(SUPPORTED_MIN_WINDOW.width >= NARROW_WINDOW_WIDTH);
    // The two notes below can never be produced from `compute_regions`.
    // `>=`, not `>`: with the widest rail (460) and the widest dock (420) the sum
    // at the 1200 threshold is exactly 1200. `DockLayout::resolve` only yields
    // when the sum is strictly greater than the width, so nothing yields there.
    const _: () = assert!(
        NARROW_WINDOW_WIDTH
            >= LEFT_RAIL_MAX + crate::dock_layout::RIGHT_DOCK_MAX + CANVAS_MIN_WIDTH
    );

    #[test]
    fn plans_for_normal_sizes_never_overlap_and_tile_the_center() {
        let sizes = [(1920., 1080.), (1600., 1000.), (1280., 900.), (1024., 768.)];
        for layout in [DockLayout::matrix_default(), layout_with_right_dock()] {
            for (width, height) in sizes {
                let first = compute_regions(&layout, window(width, height), &ALL_VIEWS).unwrap();
                first.validate().unwrap();
                assert_eq!(first.views.len(), 4);
                for slot in &first.views {
                    assert!(first.center.contains(&slot.rect), "{width}x{height}");
                }
                // The tiles plus the gap fill the center exactly.
                let tiled_width = view_rect(&first, ViewId::Perspective).right() - first.center.x;
                let tiled_height = view_rect(&first, ViewId::SideView).bottom() - first.center.y;
                assert!((tiled_width - first.center.width).abs() < 0.01);
                assert!((tiled_height - first.center.height).abs() < 0.01);
                // A plan is a value: no stale reference between calls.
                let second = compute_regions(&layout, window(width, height), &ALL_VIEWS).unwrap();
                assert_eq!(first, second);
            }
        }
    }

    #[test]
    fn plan_is_serde_ready() {
        let layout = layout_with_right_dock();
        let plan = compute_regions(&layout, window(1600., 1000.), &ALL_VIEWS).unwrap();
        let text = serde_json::to_string(&plan).unwrap();
        let back: RegionPlan = serde_json::from_str(&text).unwrap();
        assert_eq!(plan, back);
    }

    // (b) invalid / empty / degenerate -------------------------------------

    #[test]
    fn zero_negative_nonfinite_and_huge_windows_are_invalid() {
        let layout = DockLayout::matrix_default();
        let before = json(&layout);
        for (width, height) in [
            (0., 0.),
            (-10., 500.),
            (800., -1.),
            (f32::NAN, 500.),
            (800., f32::NAN),
            (f32::INFINITY, 500.),
            (800., f32::NEG_INFINITY),
            (1_000_000., 500.),
        ] {
            let error = compute_regions(&layout, window(width, height), &ALL_VIEWS).unwrap_err();
            assert!(
                matches!(error, RegionError::InvalidWindow { .. }),
                "{width}x{height}: {error}"
            );
        }
        assert_eq!(json(&layout), before);
    }

    #[test]
    fn window_below_the_minimum_returns_window_too_small() {
        let layout = DockLayout::matrix_default();
        let before = json(&layout);
        for (width, height) in [(300., 150.), (799., 400.), (800., 399.), (1., 1.)] {
            let error = compute_regions(&layout, window(width, height), &ALL_VIEWS).unwrap_err();
            assert_eq!(
                error,
                RegionError::WindowTooSmall {
                    width,
                    height,
                    min_width: 800.,
                    min_height: 400.
                }
            );
        }
        assert!(compute_regions(&layout, window(800., 400.), &ALL_VIEWS).is_ok());
        assert_eq!(json(&layout), before);
    }

    #[test]
    fn hidden_left_rail_does_not_change_the_floor() {
        let mut layout = DockLayout::matrix_default();
        layout.set_visible(PanelId::LeftRail, false);
        let error = compute_regions(&layout, window(799., 400.), &ALL_VIEWS).unwrap_err();
        assert!(matches!(
            error,
            RegionError::WindowTooSmall {
                min_width: 800.,
                min_height: 400.,
                ..
            }
        ));
        let plan = compute_regions(&layout, window(800., 400.), &ALL_VIEWS).unwrap();
        assert_eq!(plan.left_rail, None);
        assert_eq!(plan.command_input, rect(0., 21., 800., 90.));
    }

    #[test]
    fn narrow_window_yields_right_dock_with_note() {
        let layout = layout_with_right_dock();
        let plan = compute_regions(&layout, window(800., 600.), &ALL_VIEWS).unwrap();
        assert_eq!(plan.right_dock, None);
        assert_eq!(
            plan.notes,
            vec![RegionNote::RightDockYielded {
                reason: YieldReason::NarrowWindow
            }]
        );
        assert_eq!(plan.left_rail, Some(rect(0., 21., 300., 579.)));
        assert_eq!(plan.command_input, rect(300., 21., 500., 90.));
        assert_eq!(plan.center, rect(300., 178., 500., 402.));
        plan.validate().unwrap();
        // The old 910 px case (NotEnoughRoom under the 900 px rule) is simply
        // narrow now that the threshold is 1200.
        let plan = compute_regions(&layout, window(910., 700.), &ALL_VIEWS).unwrap();
        assert_eq!(
            plan.notes,
            vec![RegionNote::RightDockYielded {
                reason: YieldReason::NarrowWindow
            }]
        );
    }

    #[test]
    fn right_dock_threshold_boundary() {
        assert_eq!(NARROW_WINDOW_WIDTH, 1200.);
        let layout = layout_with_right_dock();
        for width in [1199.0_f32, 1199.99] {
            let plan = compute_regions(&layout, window(width, 720.), &ALL_VIEWS).unwrap();
            assert_eq!(plan.right_dock, None, "{width}: just below the threshold");
            assert_eq!(
                plan.notes,
                vec![RegionNote::RightDockYielded {
                    reason: YieldReason::NarrowWindow
                }],
                "{width}"
            );
        }
        let plan = compute_regions(&layout, window(1200., 720.), &ALL_VIEWS).unwrap();
        let dock = plan.right_dock.expect("dock shows at the threshold");
        assert_eq!(dock.width, 300.);
        assert!(plan.notes.is_empty());
        // Viewport width at the threshold: (1200 - 300 - 300 - 2) / 2.
        assert_eq!(view_rect(&plan, ViewId::LookingDown).width, 299.);
        plan.validate().unwrap();
    }

    #[test]
    fn dead_notes_stay_unreachable() {
        // These two notes cannot occur at any plannable size. If a later
        // constant change reopens the gap, this test must fail first.
        // (the inequalities themselves are compile-time assertions above)
        let mut widest = layout_with_right_dock();
        widest.set_size(PanelId::LeftRail, LEFT_RAIL_MAX);
        widest.set_size(PanelId::RightDock, crate::dock_layout::RIGHT_DOCK_MAX);
        for layout in [layout_with_right_dock(), widest] {
            // Includes the exact 1200 threshold, where rail 460 + dock 420 +
            // canvas 320 fits with equality.
            for width in (800..=1400).step_by(25).chain([1199, 1200, 1201]) {
                let plan =
                    compute_regions(&layout, window(width as f32, 500.), &ALL_VIEWS).unwrap();
                for note in &plan.notes {
                    assert!(
                        !matches!(
                            note,
                            RegionNote::LeftRailShrunk { .. }
                                | RegionNote::RightDockYielded {
                                    reason: YieldReason::NotEnoughRoom
                                }
                        ),
                        "{width}: {note:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn hidden_right_dock_is_a_defined_state() {
        let layout = DockLayout::matrix_default();
        let plan = compute_regions(&layout, window(1600., 1000.), &ALL_VIEWS).unwrap();
        assert_eq!(plan.right_dock, None);
        assert!(plan.notes.is_empty());
        // The panel flag alone does not create a dock without a docked menu.
        let mut flagged = DockLayout::matrix_default();
        flagged.set_visible(PanelId::RightDock, true);
        let plan = compute_regions(&flagged, window(1600., 1000.), &ALL_VIEWS).unwrap();
        assert_eq!(plan.right_dock, None);
    }

    #[test]
    fn hidden_command_line_is_kept_with_note() {
        let mut layout = DockLayout::matrix_default();
        layout.set_visible(PanelId::CommandLine, false);
        let before = json(&layout);
        let plan = compute_regions(&layout, window(1280., 900.), &ALL_VIEWS).unwrap();
        assert_eq!(plan.notes, vec![RegionNote::CommandInputKept]);
        assert_eq!(plan.command_input, rect(300., 21., 980., 90.));
        plan.validate().unwrap();
        assert_eq!(json(&layout), before);
    }

    #[test]
    fn view_set_must_be_one_to_four_distinct_views() {
        let layout = DockLayout::matrix_default();
        let size = window(1280., 900.);
        let five = [
            ViewId::LookingDown,
            ViewId::Perspective,
            ViewId::ThroughFinger,
            ViewId::SideView,
            ViewId::SideView,
        ];
        let duplicate = [ViewId::Perspective, ViewId::Perspective];
        for (views, count) in [(&[][..], 0), (&five[..], 5), (&duplicate[..], 2)] {
            assert_eq!(
                compute_regions(&layout, size, views).unwrap_err(),
                RegionError::ViewSetInvalid { count }
            );
        }
    }

    #[test]
    fn nonfinite_stored_size_is_an_explicit_error() {
        let mut layout = DockLayout::matrix_default();
        layout.panels.get_mut("left-rail").unwrap().size = f32::NAN;
        let error = compute_regions(&layout, window(1280., 900.), &ALL_VIEWS).unwrap_err();
        assert_eq!(
            error,
            RegionError::NonFinite {
                kind: RegionKind::LeftRail
            }
        );
    }

    #[test]
    fn tile_views_rejects_a_degenerate_center() {
        let center = rect(0., 0., 1., 100.);
        assert!(matches!(
            tile_views(center, &ALL_VIEWS),
            Err(RegionError::CenterTooSmall { .. })
        ));
        assert!(matches!(
            tile_views(rect(0., 0., f32::NAN, 100.), &ALL_VIEWS),
            Err(RegionError::CenterTooSmall { .. })
        ));
        // A single view keeps the whole (even tiny) rectangle.
        let single = tile_views(center, &[ViewId::Perspective]).unwrap();
        assert_eq!(single[0].rect, center);
    }

    #[test]
    fn validate_detects_overlap_and_a_short_command_input() {
        let layout = layout_with_right_dock();
        let good = compute_regions(&layout, window(1600., 1000.), &ALL_VIEWS).unwrap();

        let mut overlapping = good.clone();
        overlapping.command_input = rect(300., 21., 1300., 200.);
        assert!(matches!(
            overlapping.validate(),
            Err(RegionError::Overlap { .. })
        ));

        let mut short = good.clone();
        short.command_input.height = 20.;
        assert_eq!(
            short.validate().unwrap_err(),
            RegionError::CommandInputTooSmall { height: 20. }
        );

        let mut outside = good.clone();
        outside.status_bar.y = 995.;
        assert!(matches!(
            outside.validate(),
            Err(RegionError::OutOfBounds { .. })
        ));

        let mut bad_view = good.clone();
        bad_view.views[1].rect = bad_view.views[0].rect;
        assert!(matches!(
            bad_view.validate(),
            Err(RegionError::ViewSlotInvalid { .. })
        ));

        let mut not_finite = good;
        not_finite.toolbar.width = f32::NAN;
        assert!(matches!(
            not_finite.validate(),
            Err(RegionError::NonFinite { .. })
        ));
    }

    #[test]
    fn errors_are_visible_text_and_convert_to_the_core_error() {
        let error = RegionError::WindowTooSmall {
            width: 300.,
            height: 150.,
            min_width: PLAN_FLOOR_WINDOW.width,
            min_height: PLAN_FLOOR_WINDOW.height,
        };
        assert!(error.to_string().contains(&format!(
            "below the minimum {}x{}",
            PLAN_FLOOR_WINDOW.width, PLAN_FLOOR_WINDOW.height
        )));
        let core: om9_core::Error = error.into();
        assert!(matches!(core, om9_core::Error::Invalid(_)));
        assert!(core.to_string().contains("below minimum"));
    }

    // negative oracle ------------------------------------------------------

    #[test]
    fn narrow_window_never_overlaps_or_hides_command_input() {
        let default = DockLayout::matrix_default();
        let right = layout_with_right_dock();
        let mut hidden_command = DockLayout::matrix_default();
        hidden_command.set_visible(PanelId::CommandLine, false);
        let mut ok_plans = 0;
        let mut errors = 0;
        for layout in [&default, &right, &hidden_command] {
            for width in (200..=1600).step_by(20) {
                for height in (100..=1000).step_by(20) {
                    let size = window(width as f32, height as f32);
                    match compute_regions(layout, size, &ALL_VIEWS) {
                        Ok(plan) => {
                            ok_plans += 1;
                            plan.validate().unwrap();
                            let command = plan.command_input;
                            assert!(command.height >= COMMAND_MIN, "{width}x{height}");
                            assert!(command.width > 0., "{width}x{height}");
                            assert!(
                                RegionRect::new(0., 0., size.width, size.height).contains(&command)
                            );
                            for other in [
                                Some(plan.menu_bar),
                                plan.left_rail,
                                Some(plan.center),
                                plan.right_dock,
                                Some(plan.toolbar),
                                Some(plan.status_bar),
                            ]
                            .into_iter()
                            .flatten()
                            {
                                assert!(!command.overlaps(&other), "{width}x{height}");
                            }
                        }
                        Err(error) => {
                            errors += 1;
                            assert!(
                                matches!(error, RegionError::WindowTooSmall { .. }),
                                "{width}x{height}: {error}"
                            );
                            assert!(
                                size.width < PLAN_FLOOR_WINDOW.width
                                    || size.height < PLAN_FLOOR_WINDOW.height
                            );
                        }
                    }
                }
            }
        }
        assert!(ok_plans > 0 && errors > 0);
    }

    // (c) each operation changes only its documented state ----------------

    #[test]
    fn compute_regions_changes_nothing_but_its_return_value() {
        let layout = layout_with_right_dock();
        let snapshot = layout.clone();
        let _ = compute_regions(&layout, window(1600., 1000.), &ALL_VIEWS);
        let _ = compute_regions(&layout, window(0., 0.), &ALL_VIEWS);
        let _ = compute_regions(&layout, window(300., 150.), &ALL_VIEWS);
        assert_eq!(layout, snapshot);
        assert_eq!(json(&layout), json(&snapshot));
    }

    #[test]
    fn hosts_are_asked_only_what_the_contract_documents() {
        let layout = DockLayout::matrix_default();
        let before = json(&layout);
        let scene = StubScene {
            views: ALL_VIEWS.to_vec(),
            calls: Cell::new(0),
        };
        let command = StubCommand {
            ready: true,
            calls: Cell::new(0),
        };
        let plan = compute_regions_for(&layout, window(1280., 900.), &scene, &command).unwrap();
        assert_eq!(plan.views.len(), 4);
        assert_eq!((scene.calls.get(), command.calls.get()), (1, 1));

        let not_ready = StubCommand {
            ready: false,
            calls: Cell::new(0),
        };
        let scene = StubScene {
            views: ALL_VIEWS.to_vec(),
            calls: Cell::new(0),
        };
        let error =
            compute_regions_for(&layout, window(1280., 900.), &scene, &not_ready).unwrap_err();
        assert_eq!(error, RegionError::CommandInputUnavailable);
        // Not ready: the scene is not even consulted.
        assert_eq!((scene.calls.get(), not_ready.calls.get()), (0, 1));
        assert_eq!(json(&layout), before);
    }

    #[test]
    fn real_workspace_feeds_the_scene_seam() {
        let mut workspace = om9_scene::workspace::Workspace::new();
        assert_eq!(
            ViewportSceneHost::visible_views(&workspace),
            ViewId::ALL.to_vec()
        );
        workspace.maximized = Some(ViewId::SideView);
        assert_eq!(
            ViewportSceneHost::visible_views(&workspace),
            vec![ViewId::SideView]
        );
    }
}
