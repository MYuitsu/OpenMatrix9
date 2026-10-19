// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed panel operations for the Matrix menu column: move, collapse, dock and
//! float, with off-screen recovery.
//!
//! Owner: plan module `OM9-L2-01-01` L3-003 ("Reorder Collapse And Float
//! Panels"). The module is pure: no egui, no IO, no document access. The UI has
//! no other mutation path for menu placement, which is what makes the recovery
//! and "layout unchanged on error" rules checkable.
//!
//! Documented behaviour comes from Matrix8 book 1, "A Tour of the Interface"
//! (the in-repo reference of this module): a gray title bar rolls a menu up;
//! dragging a title bar above or below another menu changes the order; dragging
//! it into the viewports floats it over the model; dragging it to the far right
//! docks it in the mirror column; **Reset Menus** and **Reload Menus** put the
//! menus back. A menu that ends up outside the window must be recoverable.
//!
//! Design: `implementation/design/M01/OM9-L3-01-01-003/api.md` (§3 API, §4 rules)
//! and the fixtures `implementation/fixtures/M01/OM9-L3-01-01-003/`.
use crate::dock_layout::{self, DockLayout, DockSide, FloatRect, SectionId};
use crate::workbench_regions::RegionRect;
use om9_core::Error;
use serde::{Deserialize, Serialize};

/// Size of a menu floated by a drop (the title bar is grabbed at its centre).
pub const FLOAT_DROP_SIZE: [f32; 2] = [300., 240.];
/// Pointer offset inside that rectangle: the centre of the title bar.
pub const FLOAT_DROP_GRAB: [f32; 2] = [150., 7.];
/// Dock-right target width while the right column is not shown.
pub const RIGHT_DROP_STRIP: f32 = 48.;
/// Work-area margins inside the window: left/right 8, top 29 (menu bar plus the
/// command box header), bottom 28 (status line).
pub const WORK_AREA_MARGIN_X: f32 = 8.;
pub const WORK_AREA_MARGIN_TOP: f32 = 29.;
pub const WORK_AREA_MARGIN_BOTTOM: f32 = 28.;
/// Smallest floating rectangle accepted by the clamp, and the reason a work
/// area smaller than this makes the window invalid for floating menus.
pub const FLOAT_MIN_WIDTH: f32 = dock_layout::FLOAT_MIN_WIDTH;
pub const FLOAT_MIN_HEIGHT: f32 = dock_layout::FLOAT_MIN_HEIGHT;
/// Largest window dimension accepted when deriving a work area.
pub const MAX_WINDOW_DIMENSION: f32 = 100_000.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColumnSide {
    Left,
    Right,
}

impl ColumnSide {
    pub fn dock_side(self) -> DockSide {
        match self {
            ColumnSide::Left => DockSide::Left,
            ColumnSide::Right => DockSide::Right,
        }
    }

    pub fn from_dock_side(side: DockSide) -> Option<Self> {
        match side {
            DockSide::Left => Some(ColumnSide::Left),
            DockSide::Right => Some(ColumnSide::Right),
            _ => None,
        }
    }
}

/// Where a menu returns to when it is not floating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionHome {
    pub side: ColumnSide,
    pub index: u32,
}

/// The rectangle a floating menu may live in, derived from the window size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WorkArea {
    /// `1600x1000 -> [8, 29, 1584, 943]`, `1280x720 -> [8, 29, 1264, 663]`.
    pub fn for_window(window: [f32; 2]) -> Result<Self, PanelOpError> {
        let [width, height] = window;
        if !width.is_finite() || !height.is_finite() {
            return Err(PanelOpError::InvalidWindow { width, height });
        }
        if width <= 0.
            || height <= 0.
            || width > MAX_WINDOW_DIMENSION
            || height > MAX_WINDOW_DIMENSION
        {
            return Err(PanelOpError::InvalidWindow { width, height });
        }
        let area = Self {
            x: WORK_AREA_MARGIN_X,
            y: WORK_AREA_MARGIN_TOP,
            width: width - 2. * WORK_AREA_MARGIN_X,
            height: height - WORK_AREA_MARGIN_TOP - WORK_AREA_MARGIN_BOTTOM,
        };
        if area.width < FLOAT_MIN_WIDTH || area.height < FLOAT_MIN_HEIGHT {
            return Err(PanelOpError::InvalidWindow { width, height });
        }
        Ok(area)
    }

    /// Raise a too-small rectangle to the float minimum, then keep it inside.
    pub fn clamp(&self, rect: FloatRect) -> FloatRect {
        let width = rect
            .width
            .clamp(FLOAT_MIN_WIDTH, self.width.max(FLOAT_MIN_WIDTH));
        let height = rect
            .height
            .clamp(FLOAT_MIN_HEIGHT, self.height.max(FLOAT_MIN_HEIGHT));
        let max_x = (self.x + self.width - width).max(self.x);
        let max_y = (self.y + self.height - height).max(self.y);
        FloatRect {
            x: rect.x.clamp(self.x, max_x),
            y: rect.y.clamp(self.y, max_y),
            width,
            height,
        }
    }

    pub fn contains(&self, rect: FloatRect) -> bool {
        rect.width <= self.width
            && rect.height <= self.height
            && rect.x >= self.x
            && rect.y >= self.y
            && rect.x + rect.width <= self.x + self.width
            && rect.y + rect.height <= self.y + self.height
    }
}

impl std::fmt::Display for WorkArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.x, self.y, self.width, self.height
        )
    }
}

/// The rectangle a drop produces before clamping.
pub fn float_drop_rect(pointer: [f32; 2]) -> FloatRect {
    FloatRect {
        x: pointer[0] - FLOAT_DROP_GRAB[0],
        y: pointer[1] - FLOAT_DROP_GRAB[1],
        width: FLOAT_DROP_SIZE[0],
        height: FLOAT_DROP_SIZE[1],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FloatOrigin {
    Drop,
    OutsideMainWindow,
    Button,
    Api,
}

/// One placement operation. Every mutation of menu placement is one of these.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelOp {
    ToggleCollapsed {
        section: SectionId,
    },
    Move {
        section: SectionId,
        delta: i32,
    },
    Place {
        section: SectionId,
        side: ColumnSide,
        index: usize,
    },
    Dock {
        section: SectionId,
        side: ColumnSide,
    },
    DockHome {
        section: SectionId,
    },
    Float {
        section: SectionId,
        rect: FloatRect,
        work: WorkArea,
        origin: FloatOrigin,
    },
    SetFloatRect {
        section: SectionId,
        rect: FloatRect,
        work: WorkArea,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelOutcome {
    Collapsed {
        section: SectionId,
        collapsed: bool,
    },
    Moved {
        section: SectionId,
        side: ColumnSide,
        index: usize,
    },
    Docked {
        section: SectionId,
        side: ColumnSide,
        index: usize,
        restored_home: bool,
    },
    Floated {
        section: SectionId,
        rect: FloatRect,
    },
    FloatRectSet {
        section: SectionId,
        rect: FloatRect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelNote {
    Recovered {
        section: SectionId,
        from: FloatRect,
        to: FloatRect,
    },
    OutsideMainWindow {
        section: SectionId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelReport {
    pub outcome: PanelOutcome,
    pub notes: Vec<PanelNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CancelReason {
    NoTarget,
    Escape,
    PointerLost,
    LayoutChanged,
}

impl ColumnSide {
    /// The `DockSide` wrappers map here; a non-column side is an explicit error.
    pub fn try_from_dock_side(side: DockSide) -> Result<Self, PanelOpError> {
        match side {
            DockSide::Left => Ok(ColumnSide::Left),
            DockSide::Right => Ok(ColumnSide::Right),
            other => Err(PanelOpError::InvalidTarget { side: other }),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ColumnSide::Left => "left",
            ColumnSide::Right => "right",
        }
    }
}

/// One rendered menu column and how far its content reaches this frame.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ColumnZone {
    pub rect: RegionRect,
    pub content_bottom: f32,
}

/// Everything a drop can hit, collected from the frame being drawn.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DropZones {
    pub window: Option<RegionRect>,
    pub left: Option<ColumnZone>,
    pub right: Option<ColumnZone>,
    /// The rightmost [`RIGHT_DROP_STRIP`] of the views, when the right column is
    /// not shown and the window is wide enough to accept one.
    pub right_strip: Option<RegionRect>,
    pub bars: Vec<(SectionId, ColumnSide, RegionRect)>,
    pub views: Vec<RegionRect>,
}

impl DropZones {
    fn contains(rect: &RegionRect, pointer: [f32; 2]) -> bool {
        pointer[0] >= rect.x
            && pointer[1] >= rect.y
            && pointer[0] <= rect.x + rect.width
            && pointer[1] <= rect.y + rect.height
    }

    /// The index rule of the design: the position, among the sections that stay
    /// in the column, of the first rendered bar whose centre is at or below the
    /// pointer. Hidden sections keep their slot; a drop on a menu body works.
    fn column_index(
        &self,
        layout: &DockLayout,
        dragged: SectionId,
        side: ColumnSide,
        pointer: [f32; 2],
    ) -> usize {
        let others: Vec<SectionId> = layout
            .section_order(side.dock_side())
            .into_iter()
            .filter(|id| *id != dragged)
            .collect();
        let mut index = others.len();
        for (position, id) in others.iter().enumerate() {
            let Some((_, _, bar)) = self
                .bars
                .iter()
                .find(|(bar_id, bar_side, _)| bar_id == id && *bar_side == side)
            else {
                continue;
            };
            if bar.y + bar.height / 2. >= pointer[1] {
                index = position;
                break;
            }
        }
        index
    }
}

/// What a release at the current pointer position would do.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DropTarget {
    Place { side: ColumnSide, index: usize },
    Float { pointer: [f32; 2] },
    FloatOutside { pointer: [f32; 2] },
    Cancel { reason: CancelReason },
}

/// The visual feedback while a drag is pressed: a 3 px line, or the float rect.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DropIndicator {
    pub rect: RegionRect,
    pub line: bool,
}

/// The drop target for the last finite pointer position, in the documented
/// precedence: lost pointer, outside the window, a column, the right strip, a
/// viewport, and finally "nothing here".
pub fn resolve_drop(
    layout: &DockLayout,
    dragged: SectionId,
    pointer: Option<[f32; 2]>,
    zones: &DropZones,
) -> DropTarget {
    let Some(pointer) = pointer.filter(|p| p[0].is_finite() && p[1].is_finite()) else {
        return DropTarget::Cancel {
            reason: CancelReason::PointerLost,
        };
    };
    if let Some(window) = &zones.window
        && !DropZones::contains(window, pointer)
    {
        return DropTarget::FloatOutside { pointer };
    }
    if let Some(right) = &zones.right
        && DropZones::contains(&right.rect, pointer)
    {
        return DropTarget::Place {
            side: ColumnSide::Right,
            index: zones.column_index(layout, dragged, ColumnSide::Right, pointer),
        };
    }
    if let Some(left) = &zones.left
        && DropZones::contains(&left.rect, pointer)
    {
        return DropTarget::Place {
            side: ColumnSide::Left,
            index: zones.column_index(layout, dragged, ColumnSide::Left, pointer),
        };
    }
    if let Some(strip) = &zones.right_strip
        && DropZones::contains(strip, pointer)
    {
        let index = layout
            .section_order(DockSide::Right)
            .into_iter()
            .filter(|id| *id != dragged)
            .count();
        return DropTarget::Place {
            side: ColumnSide::Right,
            index,
        };
    }
    if zones
        .views
        .iter()
        .any(|rect| DropZones::contains(rect, pointer))
    {
        return DropTarget::Float { pointer };
    }
    DropTarget::Cancel {
        reason: CancelReason::NoTarget,
    }
}

/// The indicator for a target, or `None` when the drop would be cancelled.
pub fn drop_indicator(
    target: DropTarget,
    zones: &DropZones,
    work: Option<WorkArea>,
) -> Option<DropIndicator> {
    match target {
        DropTarget::Place { side, index } => {
            let bars: Vec<&(SectionId, ColumnSide, RegionRect)> = zones
                .bars
                .iter()
                .filter(|(_, bar_side, _)| *bar_side == side)
                .collect();
            let line_y = |fallback: f32| match bars.get(index) {
                Some((_, _, bar)) => bar.y - 1.5,
                None => fallback,
            };
            let (x, width, y) = match side {
                ColumnSide::Left => {
                    let zone = zones.left.as_ref()?;
                    (
                        zone.rect.x,
                        zone.rect.width,
                        line_y(zone.content_bottom - 1.5),
                    )
                }
                ColumnSide::Right => match zones.right.as_ref() {
                    Some(zone) => (
                        zone.rect.x,
                        zone.rect.width,
                        line_y(zone.content_bottom - 1.5),
                    ),
                    // The column does not exist yet: the strip stands in for it.
                    None => {
                        let strip = zones.right_strip.as_ref()?;
                        (strip.x, strip.width, line_y(strip.y))
                    }
                },
            };
            Some(DropIndicator {
                rect: RegionRect::new(x, y, width, 3.),
                line: true,
            })
        }
        DropTarget::Float { pointer } | DropTarget::FloatOutside { pointer } => {
            let work = work?;
            let rect = work.clamp(float_drop_rect(pointer));
            Some(DropIndicator {
                rect: RegionRect::new(rect.x, rect.y, rect.width, rect.height),
                line: false,
            })
        }
        DropTarget::Cancel { .. } => None,
    }
}

/// The operation a release performs. A cancelled drag is an explicit error.
pub fn drop_op(
    section: SectionId,
    target: DropTarget,
    work: Option<WorkArea>,
) -> Result<PanelOp, PanelOpError> {
    match target {
        DropTarget::Place { side, index } => Ok(PanelOp::Place {
            section,
            side,
            index,
        }),
        DropTarget::Float { pointer } => Ok(PanelOp::Float {
            section,
            rect: float_drop_rect(pointer),
            work: work.ok_or(PanelOpError::NoWorkArea)?,
            origin: FloatOrigin::Drop,
        }),
        DropTarget::FloatOutside { pointer } => Ok(PanelOp::Float {
            section,
            rect: float_drop_rect(pointer),
            work: work.ok_or(PanelOpError::NoWorkArea)?,
            origin: FloatOrigin::OutsideMainWindow,
        }),
        DropTarget::Cancel { reason } => Err(PanelOpError::DragCancelled { section, reason }),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PanelOpError {
    NotPermitted {
        section: SectionId,
    },
    UnknownSection {
        key: String,
    },
    Hidden {
        section: SectionId,
    },
    InvalidTarget {
        side: DockSide,
    },
    InvalidRect {
        section: SectionId,
    },
    InvalidWindow {
        width: f32,
        height: f32,
    },
    NoWorkArea,
    LayoutBlocked,
    NotDocked {
        section: SectionId,
    },
    NotFloating {
        section: SectionId,
    },
    AlreadyFloating {
        section: SectionId,
    },
    AlreadyThere {
        section: SectionId,
        side: ColumnSide,
        index: usize,
    },
    DragCancelled {
        section: SectionId,
        reason: CancelReason,
    },
}

impl std::fmt::Display for PanelOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelOpError::NotPermitted { section } => {
                write!(f, "{} cannot be moved", section.title())
            }
            PanelOpError::UnknownSection { key } => {
                write!(f, "menu section '{key}' is not recognized")
            }
            PanelOpError::Hidden { section } => write!(f, "{} is hidden", section.title()),
            PanelOpError::InvalidTarget { side } => write!(f, "invalid column target {side:?}"),
            PanelOpError::InvalidRect { section } => {
                write!(f, "{} has an invalid floating rectangle", section.title())
            }
            PanelOpError::InvalidWindow { width, height } => {
                write!(f, "invalid window {width}x{height}")
            }
            PanelOpError::NoWorkArea => f.write_str("no usable window work area yet"),
            PanelOpError::LayoutBlocked => f.write_str("layout is blocked by a region error"),
            PanelOpError::NotDocked { section } => {
                write!(f, "{} is floating, not docked", section.title())
            }
            PanelOpError::NotFloating { section } => {
                write!(f, "{} is docked, not floating", section.title())
            }
            PanelOpError::AlreadyFloating { section } => {
                write!(f, "{} is already floating", section.title())
            }
            PanelOpError::AlreadyThere { section, index, .. } => write!(
                f,
                "{} is already at menu position {}",
                section.title(),
                index + 1
            ),
            PanelOpError::DragCancelled { section, reason } => {
                write!(f, "{} drag cancelled ({reason:?})", section.title())
            }
        }
    }
}

impl From<PanelOpError> for Error {
    fn from(value: PanelOpError) -> Self {
        Error::Invalid(match value {
            PanelOpError::NotPermitted { .. } => "menu section is not movable",
            PanelOpError::UnknownSection { .. } => "unknown menu section",
            PanelOpError::Hidden { .. } => "menu section is hidden",
            PanelOpError::InvalidTarget { .. } => "invalid column target",
            PanelOpError::InvalidRect { .. } => "invalid floating rectangle",
            PanelOpError::InvalidWindow { .. } => "invalid window size",
            PanelOpError::NoWorkArea => "no usable work area",
            PanelOpError::LayoutBlocked => "layout blocked",
            PanelOpError::NotDocked { .. } => "menu section is not docked",
            PanelOpError::NotFloating { .. } => "menu section is not floating",
            PanelOpError::AlreadyFloating { .. } => "menu section is already floating",
            PanelOpError::AlreadyThere { .. } => "menu section is already there",
            PanelOpError::DragCancelled { .. } => "menu drag cancelled",
        })
    }
}

impl SectionId {
    /// `BuilderSlot` is reserved by M01 (D4): no operation may move it.
    pub fn is_movable(self) -> bool {
        !self.is_reserved()
    }
}

impl DockLayout {
    /// Apply one placement operation. On `Err` the layout is untouched, so the
    /// serialized form is byte-identical by construction.
    pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError> {
        let section = op.section();
        if !section.is_movable() {
            return Err(PanelOpError::NotPermitted { section });
        }
        if !self.section(section).visible {
            return Err(PanelOpError::Hidden { section });
        }
        let mut staged = self.clone();
        let report = staged.apply_checked(op)?;
        // Every successful operation leaves all three columns dense, including
        // the floating column that has no explicit index of its own.
        for side in [ColumnSide::Left, ColumnSide::Right] {
            let order = staged.section_order(side.dock_side());
            write_order(&mut staged, side, &order);
        }
        let floating = staged.section_order(DockSide::Floating);
        for (index, id) in floating.iter().enumerate() {
            state_mut(&mut staged, *id).order = index as u32;
        }
        staged.assert_invariants();
        *self = staged;
        Ok(report)
    }

    fn apply_checked(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError> {
        match op {
            PanelOp::ToggleCollapsed { section } => {
                let state = self.toggle_section(section);
                Ok(PanelReport {
                    outcome: PanelOutcome::Collapsed {
                        section,
                        collapsed: state.collapsed,
                    },
                    notes: Vec::new(),
                })
            }
            PanelOp::Move { section, delta } => self.move_within_column(section, delta),
            PanelOp::Place {
                section,
                side,
                index,
            } => self.place_in_column(section, side, index),
            PanelOp::Dock { section, side } => self.dock_to(section, side),
            PanelOp::DockHome { section } => self.dock_home(section),
            PanelOp::Float {
                section,
                rect,
                work,
                origin,
            } => self.float_from(section, rect, work, origin),
            PanelOp::SetFloatRect {
                section,
                rect,
                work,
            } => self.set_float_rect(section, rect, work),
        }
    }

    /// ▲ ▼ and the API: reorder inside the menu's own column.
    fn move_within_column(
        &mut self,
        section: SectionId,
        delta: i32,
    ) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        let side =
            ColumnSide::from_dock_side(state.dock).ok_or(PanelOpError::NotDocked { section })?;
        let order = self.section_order(state.dock);
        let index = order
            .iter()
            .position(|id| *id == section)
            .ok_or(PanelOpError::NotDocked { section })?;
        let target = index
            .saturating_add_signed(delta as isize)
            .min(order.len().saturating_sub(1));
        if target == index {
            return Err(PanelOpError::AlreadyThere {
                section,
                side,
                index,
            });
        }
        let mut reordered = order;
        let moved = reordered.remove(index);
        reordered.insert(target, moved);
        write_order(self, side, &reordered);
        Ok(PanelReport {
            outcome: PanelOutcome::Moved {
                section,
                side,
                index: target,
            },
            notes: Vec::new(),
        })
    }

    /// Drop a menu at a column position, in the same column or the other one.
    fn place_in_column(
        &mut self,
        section: SectionId,
        side: ColumnSide,
        index: usize,
    ) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        let target = side.dock_side();
        match ColumnSide::from_dock_side(state.dock) {
            Some(current_side) if current_side == side => {
                let order = self.section_order(state.dock);
                let current = order
                    .iter()
                    .position(|id| *id == section)
                    .ok_or(PanelOpError::NotDocked { section })?;
                // `index` is the position among the sections that stay, which is
                // exactly what `resolve_drop` reports (design §5).
                let mut reordered = order;
                reordered.remove(current);
                let insert_at = index.min(reordered.len());
                if insert_at == current {
                    return Err(PanelOpError::AlreadyThere {
                        section,
                        side,
                        index: current,
                    });
                }
                reordered.insert(insert_at, section);
                write_order(self, side, &reordered);
                Ok(PanelReport {
                    outcome: PanelOutcome::Moved {
                        section,
                        side,
                        index: insert_at,
                    },
                    notes: Vec::new(),
                })
            }
            other => {
                if let Some(from) = other {
                    let order = self.section_order(state.dock);
                    let current = order.iter().position(|id| *id == section).unwrap_or(0);
                    let mut remaining = order;
                    remaining.retain(|id| *id != section);
                    write_order(self, from, &remaining);
                    let inserted = insert_at(self, section, side, index);
                    let state_mut = state_mut(self, section);
                    state_mut.dock = target;
                    state_mut.home = state.home.or(Some(SectionHome {
                        side: from,
                        index: current as u32,
                    }));
                    return Ok(PanelReport {
                        outcome: PanelOutcome::Docked {
                            section,
                            side,
                            index: inserted,
                            restored_home: false,
                        },
                        notes: Vec::new(),
                    });
                }
                // A floating menu placed into a column is simply docked there.
                let inserted = insert_at(self, section, side, index);
                state_mut(self, section).dock = target;
                Ok(PanelReport {
                    outcome: PanelOutcome::Docked {
                        section,
                        side,
                        index: inserted,
                        restored_home: false,
                    },
                    notes: Vec::new(),
                })
            }
        }
    }

    /// ◀ and "Dock right ▶": dock to a column, restoring the home position when
    /// the menu returns to the side it came from.
    fn dock_to(
        &mut self,
        section: SectionId,
        side: ColumnSide,
    ) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        let target = side.dock_side();
        if ColumnSide::from_dock_side(state.dock) == Some(side) {
            let index = self
                .section_order(target)
                .iter()
                .position(|id| *id == section)
                .unwrap_or(0);
            return Err(PanelOpError::AlreadyThere {
                section,
                side,
                index,
            });
        }
        let restored_home = state.home.is_some_and(|home| home.side == side);
        let index = match state.home.filter(|home| home.side == side) {
            Some(home) => home.index as usize,
            None => self.section_order(target).len(),
        };
        let leaving = ColumnSide::from_dock_side(state.dock).map(|from| {
            let order = self.section_order(state.dock);
            SectionHome {
                side: from,
                index: order.iter().position(|id| *id == section).unwrap_or(0) as u32,
            }
        });
        if let Some(from) = ColumnSide::from_dock_side(state.dock) {
            let order = self.section_order(state.dock);
            let mut remaining = order;
            remaining.retain(|id| *id != section);
            write_order(self, from, &remaining);
        }
        let inserted = insert_at(self, section, side, index);
        let state_mut = state_mut(self, section);
        state_mut.dock = target;
        state_mut.visible = true;
        state_mut.home = if restored_home {
            None
        } else {
            state.home.or(leaving)
        };
        Ok(PanelReport {
            outcome: PanelOutcome::Docked {
                section,
                side,
                index: inserted,
                restored_home,
            },
            notes: Vec::new(),
        })
    }

    /// Closing a floating menu, and "Reset/Reload": back to the home column.
    fn dock_home(&mut self, section: SectionId) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        let side = state.home.map_or(ColumnSide::Left, |home| home.side);
        if ColumnSide::from_dock_side(state.dock) == Some(side) {
            let index = self
                .section_order(side.dock_side())
                .iter()
                .position(|id| *id == section)
                .unwrap_or(0);
            return Err(PanelOpError::AlreadyThere {
                section,
                side,
                index,
            });
        }
        let restored_home = state.home.is_some_and(|home| home.side == side);
        let index = match state.home.filter(|home| home.side == side) {
            Some(home) => home.index as usize,
            // D6: a menu without a home returns to its documented position.
            None => documented_index(section),
        };
        if let Some(from) = ColumnSide::from_dock_side(state.dock) {
            let order = self.section_order(state.dock);
            let mut remaining = order;
            remaining.retain(|id| *id != section);
            write_order(self, from, &remaining);
        }
        let inserted = insert_at(self, section, side, index);
        state_mut(self, section).dock = side.dock_side();
        state_mut(self, section).home = None;
        Ok(PanelReport {
            outcome: PanelOutcome::Docked {
                section,
                side,
                index: inserted,
                restored_home,
            },
            notes: Vec::new(),
        })
    }

    /// Float a menu over the model, recovering a rectangle that left the window.
    fn float_from(
        &mut self,
        section: SectionId,
        rect: FloatRect,
        work: WorkArea,
        origin: FloatOrigin,
    ) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        if state.dock == DockSide::Floating {
            return Err(PanelOpError::AlreadyFloating { section });
        }
        if !rect_is_valid(rect) {
            return Err(PanelOpError::InvalidRect { section });
        }
        let to = work.clamp(rect);
        let mut notes = Vec::new();
        if to != rect {
            notes.push(PanelNote::Recovered {
                section,
                from: rect,
                to,
            });
        }
        if origin == FloatOrigin::OutsideMainWindow {
            notes.push(PanelNote::OutsideMainWindow { section });
        }
        let leaving = ColumnSide::from_dock_side(state.dock).map(|from| {
            let order = self.section_order(state.dock);
            SectionHome {
                side: from,
                index: order.iter().position(|id| *id == section).unwrap_or(0) as u32,
            }
        });
        if let Some(from) = ColumnSide::from_dock_side(state.dock) {
            let order = self.section_order(state.dock);
            let mut remaining = order;
            remaining.retain(|id| *id != section);
            write_order(self, from, &remaining);
        }
        let state_mut = state_mut(self, section);
        state_mut.dock = DockSide::Floating;
        state_mut.float = Some(to);
        state_mut.visible = true;
        state_mut.home = state.home.or(leaving);
        Ok(PanelReport {
            outcome: PanelOutcome::Floated { section, rect: to },
            notes,
        })
    }

    /// The floating window was moved or resized by the user: read the rectangle
    /// back, and only report recovery when the window had to be pulled inside.
    fn set_float_rect(
        &mut self,
        section: SectionId,
        rect: FloatRect,
        work: WorkArea,
    ) -> Result<PanelReport, PanelOpError> {
        let state = self.section(section);
        if state.dock != DockSide::Floating {
            return Err(PanelOpError::NotFloating { section });
        }
        if !rect_is_valid(rect) {
            return Err(PanelOpError::InvalidRect { section });
        }
        let to = work.clamp(rect);
        let notes = if to != rect {
            vec![PanelNote::Recovered {
                section,
                from: rect,
                to,
            }]
        } else {
            Vec::new()
        };
        state_mut(self, section).float = Some(to);
        Ok(PanelReport {
            outcome: PanelOutcome::FloatRectSet { section, rect: to },
            notes,
        })
    }

    fn assert_invariants(&self) {
        let mut seen: Vec<&'static str> = Vec::new();
        for side in [DockSide::Left, DockSide::Right, DockSide::Floating] {
            let order = self.section_order(side);
            for (index, id) in order.iter().enumerate() {
                debug_assert_eq!(
                    self.section(*id).order,
                    index as u32,
                    "column orders stay dense"
                );
                seen.push(id.key());
            }
        }
        seen.sort_unstable();
        let mut expected: Vec<&'static str> = dock_layout::SectionId::ALL
            .iter()
            .map(|id| id.key())
            .collect();
        expected.sort_unstable();
        debug_assert_eq!(seen, expected, "every menu has exactly one column");
    }
}

/// The documented position of a menu in the left column (D6).
fn documented_index(section: SectionId) -> usize {
    dock_layout::SectionId::ALL
        .iter()
        .position(|id| *id == section)
        .unwrap_or(0)
}

fn rect_is_valid(rect: FloatRect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(f32::is_finite)
        && rect.width > 0.
        && rect.height > 0.
}

fn state_mut(layout: &mut DockLayout, id: SectionId) -> &mut dock_layout::SectionState {
    layout
        .sections
        .entry(id.key().to_owned())
        .or_insert_with(|| id.default_state())
}

fn write_order(layout: &mut DockLayout, side: ColumnSide, order: &[SectionId]) {
    for (index, id) in order.iter().enumerate() {
        state_mut(layout, *id).order = index as u32;
        state_mut(layout, *id).dock = side.dock_side();
    }
}

fn insert_at(layout: &mut DockLayout, section: SectionId, side: ColumnSide, index: usize) -> usize {
    let mut order = layout.section_order(side.dock_side());
    order.retain(|id| *id != section);
    let insert_at = index.min(order.len());
    order.insert(insert_at, section);
    write_order(layout, side, &order);
    insert_at
}

impl PanelOp {
    pub fn section(self) -> SectionId {
        match self {
            PanelOp::ToggleCollapsed { section }
            | PanelOp::Move { section, .. }
            | PanelOp::Place { section, .. }
            | PanelOp::Dock { section, .. }
            | PanelOp::DockHome { section }
            | PanelOp::Float { section, .. }
            | PanelOp::SetFloatRect { section, .. } => section,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_area_matches_the_documented_numbers() {
        let wide = WorkArea::for_window([1600., 1000.]).unwrap();
        assert_eq!(
            wide,
            WorkArea {
                x: 8.,
                y: 29.,
                width: 1584.,
                height: 943.
            }
        );
        let small = WorkArea::for_window([1280., 720.]).unwrap();
        assert_eq!(
            small,
            WorkArea {
                x: 8.,
                y: 29.,
                width: 1264.,
                height: 663.
            }
        );
    }

    #[test]
    fn work_area_refuses_unusable_windows() {
        for window in [
            [0., 1000.],
            [-10., 500.],
            [f32::NAN, 500.],
            [800., f32::INFINITY],
            [0.5e6, 500.],
            [200., 100.],
        ] {
            assert!(
                WorkArea::for_window(window).is_err(),
                "window {window:?} must be refused"
            );
        }
    }

    #[test]
    fn float_drop_rect_is_the_pointer_minus_the_grab_offset() {
        let rect = float_drop_rect([1700., 500.]);
        assert_eq!(
            rect,
            FloatRect {
                x: 1550.,
                y: 493.,
                width: 300.,
                height: 240.
            }
        );
    }
}
