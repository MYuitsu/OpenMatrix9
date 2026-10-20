// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix-oriented dock layout for the standalone application shell.
//!
//! Owner: plan module `OM9-L2-01-01` (L3-002 docking regions, L3-003
//! reorder/collapse/float, L3-004 default restore, L3-005 persistence, L3-008
//! builder slot). This module owns panel/menu placement, size and visibility
//! only: it never reads or mutates document geometry, and it never creates a
//! second model for a panel owner such as the Builder.
//!
//! Documented Matrix layout (Matrix8 book 1, "A Tour of the Interface", the
//! in-repo reference for this module): the left column is divided by gray title
//! bars into **Icon History, Main Menu, Display, Snaps, Info & Settings,
//! Layers, Projects**, and "in the space below the Project Manager, the Builder
//! interface for Matrix tools will appear". Titles can be rolled up
//! (collapsed), dragged to reorder, floated over the model, or docked to the far
//! right in a ~300-pixel column that mirrors the left one. The configuration is
//! kept with View ▸ Matrix Menus ▸ Set as Default, restored with Reset Menus and
//! reloaded with Reload Menus.
//!
//! The Matrix9 screenshot shows no right dock: by default the far-right column
//! is hidden and only appears after the user drags a menu there. Layers always
//! sits in the left column directly below Info & Settings; no audit, fixture or
//! test docks it right ([`RIGHT_DOCK_FIXTURE`] is the neutral stand-in). Rail
//! widths are OUTER widths; the content is [`LEFT_RAIL_CHROME_PX`] narrower, and
//! the icons of a column follow its width ([`rail_icon_edge`]).
//!
//! Loading is deliberately total: a missing, corrupt, oversized or
//! future-versioned layout file resets to the Matrix default and reports a
//! diagnostic instead of failing or panicking.
use om9_core::{Error, Result, budget::Limits};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Schema 2 introduces the documented left-column menu sections and drops the
/// invented right-dock Project Manager/Properties panels.
pub const SCHEMA_VERSION: u32 = 2;

/// Rail widths are OUTER widths (frame margins included), the number the manual
/// and the user mean. The content inside is [`LEFT_RAIL_CHROME_PX`] narrower.
pub const LEFT_RAIL_MIN: f32 = 240.;
/// 460 is the largest rail that keeps both layout guarantees: the plan floor
/// (800 >= 460 + `CANVAS_MIN_WIDTH`) and the unreachable "not enough room" note
/// (`NARROW_WINDOW_WIDTH` 1200 >= 460 + `RIGHT_DOCK_MAX` 420 + `CANVAS_MIN_WIDTH`,
/// with equality). `resolve` yields only when the sum is strictly greater.
pub const LEFT_RAIL_MAX: f32 = 460.;
pub const LEFT_RAIL_DEFAULT: f32 = 300.;
/// Outer minus inner width of the left rail (frame margin and panel stroke),
/// measured from a real frame (see `left-dock-measurements.md`).
pub const LEFT_RAIL_CHROME_PX: f32 = 5.;
/// Icon box edge (points) in the left column at the default rail width.
pub const ICON_BASE_EDGE: f32 = 27.5;
/// Smallest icon box edge a narrow rail uses.
pub const ICON_MIN_EDGE: f32 = 23.;
/// Largest icon box edge a widened rail uses.
pub const ICON_MAX_EDGE: f32 = 42.;
/// Neutral section used wherever an audit, fixture or test needs something in
/// the right dock. It is never Layers: Matrix9 shows no right dock and Layers
/// lives in the left column.
pub const RIGHT_DOCK_FIXTURE: SectionId = SectionId::Snaps;
// The drop strip while a menu is dragged and the right dock is still hidden is
// owned by the panel operations: see `panel_ops::RIGHT_DROP_STRIP` (48 points).
// It stays the rightmost strip of the views' union, not a fixed edge overlay.

/// Icon box edge for a column of the given OUTER width: proportional to the
/// default 300 px rail, clamped to 23..42 and rounded to whole points.
/// Non-finite widths give the base edge.
pub fn rail_icon_edge(outer_width: f32) -> f32 {
    if !outer_width.is_finite() {
        return ICON_BASE_EDGE;
    }
    (ICON_BASE_EDGE * outer_width / LEFT_RAIL_DEFAULT)
        .clamp(ICON_MIN_EDGE, ICON_MAX_EDGE)
        .round()
}
pub const RIGHT_DOCK_MIN: f32 = 200.;
pub const RIGHT_DOCK_MAX: f32 = 420.;
/// The manual documents the right-hand column as a ~300-pixel mirror of the left.
pub const RIGHT_DOCK_DEFAULT: f32 = 300.;
pub const COMMAND_MIN: f32 = 58.;
pub const COMMAND_MAX: f32 = 400.;
pub const COMMAND_DEFAULT: f32 = 90.;
pub const TOOLBAR_HEIGHT: f32 = 67.;
pub const STATUS_BAR_HEIGHT: f32 = 20.;
pub const CANVAS_MIN_WIDTH: f32 = 320.;
pub const CANVAS_MIN_HEIGHT: f32 = 120.;
/// Below this window width (logical points) the optional right column yields
/// instead of squeezing the command line or the modelling canvas.
///
/// 1200 (was 900, which first showed the dock at 915): with the default rails
/// (left 300, right 300, gap 2) each of the four viewports is `(W - 602) / 2`
/// wide, so 1200 gives 299 pt, about the smallest width a viewport stays useful
/// at. The old rule left roughly 160 pt per viewport at 915. The supported
/// window minimum is 1280x720, so the dock shows at every supported size on a
/// normal screen and yields on capped high-DPI laptop screens (1366x768 at
/// 125%/150%).
pub const NARROW_WINDOW_WIDTH: f32 = 1200.;
pub const SECTION_MIN_HEIGHT: f32 = 24.;
pub const SECTION_MAX_HEIGHT: f32 = 2000.;
pub const SECTION_DEFAULT_HEIGHT: f32 = 120.;
pub const FLOAT_MIN_WIDTH: f32 = 180.;
pub const FLOAT_MIN_HEIGHT: f32 = 120.;
pub const FLOAT_MARGIN: f32 = 8.;
pub const MAX_SECTION_BYTES: usize = 128;

/// Where a panel or menu section lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DockSide {
    Top,
    Bottom,
    Left,
    Right,
    Floating,
}

/// Region-level panels. Keys are persisted, so they are part of the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PanelId {
    LeftRail,
    CommandLine,
    Toolbar,
    RightDock,
    CommandSearch,
}

impl PanelId {
    pub const ALL: [PanelId; 5] = [
        PanelId::LeftRail,
        PanelId::CommandLine,
        PanelId::Toolbar,
        PanelId::RightDock,
        PanelId::CommandSearch,
    ];

    pub fn key(self) -> &'static str {
        match self {
            PanelId::LeftRail => "left-rail",
            PanelId::CommandLine => "command-line",
            PanelId::Toolbar => "toolbar",
            PanelId::RightDock => "right-dock",
            PanelId::CommandSearch => "command-search",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.key() == key)
    }

    pub fn title(self) -> &'static str {
        match self {
            PanelId::LeftRail => "Matrix Interface",
            PanelId::CommandLine => "Command Line",
            PanelId::Toolbar => "Toolbars",
            PanelId::RightDock => "Right Menu Column",
            PanelId::CommandSearch => "Command Search",
        }
    }

    pub fn default_state(self) -> PanelState {
        let mut state = PanelState::region(true, 0.);
        match self {
            PanelId::LeftRail => state.size = LEFT_RAIL_DEFAULT,
            PanelId::CommandLine => state.size = COMMAND_DEFAULT,
            PanelId::Toolbar => state.size = TOOLBAR_HEIGHT,
            // Docking menus to the right is a user action, so the column starts
            // hidden and shows nothing until a menu is docked there.
            PanelId::RightDock => {
                state.visible = false;
                state.size = RIGHT_DOCK_DEFAULT;
            }
            PanelId::CommandSearch => {
                state.visible = false;
                state.float = Some(FloatRect::command_search_default());
            }
        }
        state
    }
}

/// The documented left-column menu sections plus the reserved Builder area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SectionId {
    IconHistory,
    MainMenu,
    Display,
    Snaps,
    InfoSettings,
    Layers,
    Projects,
    /// Reserved by M01: builders open in the space below the Project Manager.
    BuilderSlot,
}

impl SectionId {
    /// Documented order, with the reserved Builder area directly below Projects.
    pub const ALL: [SectionId; 8] = [
        SectionId::IconHistory,
        SectionId::MainMenu,
        SectionId::Display,
        SectionId::Snaps,
        SectionId::InfoSettings,
        SectionId::Layers,
        SectionId::Projects,
        SectionId::BuilderSlot,
    ];

    pub fn key(self) -> &'static str {
        match self {
            SectionId::IconHistory => "icon-history",
            SectionId::MainMenu => "main-menu",
            SectionId::Display => "display",
            SectionId::Snaps => "snaps",
            SectionId::InfoSettings => "info-settings",
            SectionId::Layers => "layers",
            SectionId::Projects => "projects",
            SectionId::BuilderSlot => "builder-slot",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.key() == key)
    }

    /// Title bar text exactly as the manual writes it.
    pub fn title(self) -> &'static str {
        match self {
            SectionId::IconHistory => "Icon History",
            SectionId::MainMenu => "Main Menu",
            SectionId::Display => "Display",
            SectionId::Snaps => "Snaps",
            SectionId::InfoSettings => "Info & Settings",
            SectionId::Layers => "Layers",
            SectionId::Projects => "Projects",
            SectionId::BuilderSlot => "Builders",
        }
    }

    /// Uppercase title bar used by the rendered chrome.
    pub fn bar_title(self) -> &'static str {
        match self {
            SectionId::IconHistory => "ICON HISTORY",
            SectionId::MainMenu => "MAIN MENU",
            SectionId::Display => "DISPLAY",
            SectionId::Snaps => "SNAPS",
            SectionId::InfoSettings => "INFO & SETTINGS",
            SectionId::Layers => "LAYERS",
            SectionId::Projects => "PROJECTS",
            SectionId::BuilderSlot => "BUILDERS",
        }
    }

    /// `Snaps` and `Layers` keep important controls on their closed title bars.
    pub fn keeps_controls_while_collapsed(self) -> bool {
        matches!(self, SectionId::Snaps | SectionId::Layers)
    }

    /// Reserved by M01 for M04; the slot never owns a builder model itself.
    pub fn is_reserved(self) -> bool {
        matches!(self, SectionId::BuilderSlot)
    }

    pub fn default_state(self) -> SectionState {
        SectionState {
            visible: true,
            collapsed: false,
            dock: DockSide::Left,
            order: Self::ALL
                .iter()
                .position(|id| *id == self)
                .unwrap_or_default() as u32,
            height: SECTION_DEFAULT_HEIGHT,
            float: None,
            home: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloatRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl FloatRect {
    pub fn command_search_default() -> Self {
        Self {
            x: 480.,
            y: 160.,
            width: 520.,
            height: 320.,
        }
    }

    fn finite(&self) -> bool {
        [self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f32::is_finite)
    }

    fn normalized(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width: self.width.max(FLOAT_MIN_WIDTH),
            height: self.height.max(FLOAT_MIN_HEIGHT),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelState {
    pub visible: bool,
    pub collapsed: bool,
    pub size: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub float: Option<FloatRect>,
}

impl PanelState {
    fn region(visible: bool, size: f32) -> Self {
        Self {
            visible,
            collapsed: false,
            size,
            float: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionState {
    pub visible: bool,
    pub collapsed: bool,
    pub dock: DockSide,
    pub order: u32,
    #[serde(default = "default_section_height")]
    pub height: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub float: Option<FloatRect>,
    /// Where this menu returns to when it is docked home again (L3-003). The
    /// field is optional and defaults to `None`, so layouts written before the
    /// panel operations existed still load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home: Option<crate::panel_ops::SectionHome>,
}

fn default_section_height() -> f32 {
    SECTION_DEFAULT_HEIGHT
}

/// Persisted layout document. Unknown fields are ignored so a newer writer does
/// not brick an older reader; the `schema_version` gate decides compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockLayout {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub panels: BTreeMap<String, PanelState>,
    #[serde(default)]
    pub sections: BTreeMap<String, SectionState>,
}

impl Default for DockLayout {
    fn default() -> Self {
        Self::matrix_default()
    }
}

impl DockLayout {
    /// Documented Matrix baseline: region panels plus the seven menu sections in
    /// manual order and the reserved Builder area below Projects.
    pub fn matrix_default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            panels: PanelId::ALL
                .into_iter()
                .map(|id| (id.key().to_owned(), id.default_state()))
                .collect(),
            sections: SectionId::ALL
                .into_iter()
                .map(|id| (id.key().to_owned(), id.default_state()))
                .collect(),
        }
    }

    pub fn state(&self, id: PanelId) -> PanelState {
        self.panels
            .get(id.key())
            .cloned()
            .unwrap_or_else(|| id.default_state())
    }

    fn state_mut(&mut self, id: PanelId) -> &mut PanelState {
        self.panels
            .entry(id.key().to_owned())
            .or_insert_with(|| id.default_state())
    }

    pub fn section(&self, id: SectionId) -> SectionState {
        self.sections
            .get(id.key())
            .cloned()
            .unwrap_or_else(|| id.default_state())
    }

    fn section_mut(&mut self, id: SectionId) -> &mut SectionState {
        self.sections
            .entry(id.key().to_owned())
            .or_insert_with(|| id.default_state())
    }

    pub fn set_size(&mut self, id: PanelId, size: f32) {
        let (min, max) = size_bounds(id);
        let size = if size.is_finite() {
            size.clamp(min, max)
        } else {
            id.default_state().size
        };
        self.state_mut(id).size = size;
    }

    pub fn set_visible(&mut self, id: PanelId, visible: bool) {
        self.state_mut(id).visible = visible;
    }

    pub fn set_float(&mut self, id: PanelId, rect: Option<FloatRect>) {
        let rect = rect.filter(FloatRect::finite).map(FloatRect::normalized);
        self.state_mut(id).float = rect;
    }

    pub fn set_section_visible(&mut self, id: SectionId, visible: bool) {
        self.section_mut(id).visible = visible;
    }

    /// Roll a title bar up or down; Snaps/Layers keep controls on the closed bar,
    /// which the renderer owns.
    pub fn set_section_collapsed(&mut self, id: SectionId, collapsed: bool) {
        self.section_mut(id).collapsed = collapsed;
    }

    pub fn toggle_section(&mut self, id: SectionId) -> SectionState {
        let collapsed = !self.section(id).collapsed;
        self.set_section_collapsed(id, collapsed);
        self.section(id)
    }

    pub fn set_section_height(&mut self, id: SectionId, height: f32) {
        let height = if height.is_finite() {
            height.clamp(SECTION_MIN_HEIGHT, SECTION_MAX_HEIGHT)
        } else {
            SECTION_DEFAULT_HEIGHT
        };
        self.section_mut(id).height = height;
    }

    /// Reorder a menu inside its own column, exactly like dragging a title bar
    /// above or below another menu. Returns the moved section and its new index.
    pub fn move_section(&mut self, id: SectionId, delta: i32) -> Result<(SectionId, usize)> {
        let side = self.section(id).dock;
        if side == DockSide::Floating {
            return Err(Error::Invalid("floating menu order"));
        }
        let mut ordered: Vec<SectionId> = self.section_order(side);
        let index = ordered
            .iter()
            .position(|section| *section == id)
            .ok_or(Error::Missing("menu section"))?;
        let target = index
            .saturating_add_signed(delta as isize)
            .min(ordered.len().saturating_sub(1));
        if target != index {
            let moved = ordered.remove(index);
            ordered.insert(target, moved);
            for (order, section) in ordered.iter().enumerate() {
                self.section_mut(*section).order = order as u32;
            }
        }
        Ok((id, target))
    }

    /// Dock a menu to the left column or to the far-right mirror column.
    pub fn dock_section(&mut self, id: SectionId, side: DockSide) -> Result<()> {
        if !matches!(side, DockSide::Left | DockSide::Right) {
            return Err(Error::Invalid("menu dock side"));
        }
        let next = self.section_order(side).len() as u32;
        let state = self.section_mut(id);
        state.dock = side;
        state.visible = true;
        state.order = next;
        state.collapsed = false;
        self.dense_section_orders();
        Ok(())
    }

    /// Float a menu over the model at an owned rectangle.
    pub fn float_section(&mut self, id: SectionId, rect: FloatRect) {
        let rect = if rect.finite() {
            rect.normalized()
        } else {
            FloatRect::command_search_default()
        };
        let state = self.section_mut(id);
        state.dock = DockSide::Floating;
        state.visible = true;
        state.float = Some(rect);
        self.dense_section_orders();
    }

    /// Menu order for one column: dense, complete and stable.
    pub fn section_order(&self, side: DockSide) -> Vec<SectionId> {
        let mut members: Vec<(u32, SectionId)> = SectionId::ALL
            .into_iter()
            .filter(|id| self.section(*id).dock == side)
            .map(|id| (self.section(id).order, id))
            .collect();
        members.sort_by_key(|(order, id)| (*order, *id));
        members.into_iter().map(|(_, id)| id).collect()
    }

    /// The documented left column order.
    pub fn left_sections(&self) -> Vec<SectionId> {
        self.section_order(DockSide::Left)
    }

    /// Menus the user docked to the far-right mirror column.
    pub fn right_sections(&self) -> Vec<SectionId> {
        self.section_order(DockSide::Right)
    }

    pub fn floating_sections(&self) -> Vec<SectionId> {
        self.section_order(DockSide::Floating)
    }

    /// Renumber every column to a dense order. Returns whether anything changed,
    /// so `normalize` can report the repair instead of renumbering silently.
    fn dense_section_orders(&mut self) -> bool {
        let mut changed = false;
        for side in [DockSide::Left, DockSide::Right, DockSide::Floating] {
            for (order, section) in self.section_order(side).into_iter().enumerate() {
                let state = self.section_mut(section);
                if state.order != order as u32 {
                    state.order = order as u32;
                    changed = true;
                }
            }
        }
        changed
    }

    /// Drop unknown keys, restore missing entries, clamp sizes/heights/orders and
    /// normalize float rectangles. Diagnostics are user-visible warnings, never
    /// silent data loss.
    pub fn normalize(&mut self) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let known_panels: BTreeSet<&'static str> =
            PanelId::ALL.into_iter().map(PanelId::key).collect();
        let unknown: Vec<String> = self
            .panels
            .keys()
            .filter(|key| !known_panels.contains(key.as_str()))
            .cloned()
            .collect();
        for key in unknown {
            self.panels.remove(&key);
            diagnostics.push(format!("layout panel '{key}' is not recognized; removed"));
        }
        for id in PanelId::ALL {
            let state = self
                .panels
                .entry(id.key().to_owned())
                .or_insert_with(|| id.default_state());
            let default = id.default_state().size;
            let original = state.size;
            let (min, max) = size_bounds(id);
            let mut fixed = if original.is_finite() {
                original
            } else {
                default
            };
            fixed = if min == max {
                min
            } else {
                fixed.clamp(min, max)
            };
            if fixed != original {
                diagnostics.push(format!(
                    "panel '{}' size {original} is not usable; using {fixed}",
                    id.key()
                ));
            }
            state.size = fixed;
            state.float = match state.float.take() {
                Some(rect) if !rect.finite() => {
                    diagnostics.push(format!(
                        "panel '{}' floating rectangle is not finite; the stored position was dropped",
                        id.key()
                    ));
                    None
                }
                Some(rect) => {
                    let fixed = rect.normalized();
                    if fixed != rect {
                        diagnostics.push(format!(
                            "panel '{}' floating rectangle {}x{} is smaller than the minimum; raised to {}x{}",
                            id.key(),
                            rect.width,
                            rect.height,
                            fixed.width,
                            fixed.height
                        ));
                    }
                    Some(fixed)
                }
                None => None,
            };
        }
        let known_sections: BTreeSet<&'static str> =
            SectionId::ALL.into_iter().map(SectionId::key).collect();
        let unknown: Vec<String> = self
            .sections
            .keys()
            .filter(|key| !known_sections.contains(key.as_str()))
            .cloned()
            .collect();
        for key in unknown {
            self.sections.remove(&key);
            diagnostics.push(format!("menu section '{key}' is not recognized; removed"));
        }
        for id in SectionId::ALL {
            let state = self
                .sections
                .entry(id.key().to_owned())
                .or_insert_with(|| id.default_state());
            let original = state.height;
            let mut fixed = if original.is_finite() {
                original
            } else {
                SECTION_DEFAULT_HEIGHT
            };
            fixed = fixed.clamp(SECTION_MIN_HEIGHT, SECTION_MAX_HEIGHT);
            if fixed != original {
                diagnostics.push(format!(
                    "{} height {original} is not usable; using {fixed}",
                    id.title()
                ));
            }
            state.height = fixed;
            state.float = match state.float.take() {
                Some(rect) if !rect.finite() => {
                    diagnostics.push(format!(
                        "{} floating rectangle is not finite; the stored position was dropped",
                        id.title()
                    ));
                    None
                }
                Some(rect) => {
                    let fixed = rect.normalized();
                    if fixed != rect {
                        diagnostics.push(format!(
                            "{} floating rectangle {}x{} is smaller than the minimum; raised to {}x{}",
                            id.title(),
                            rect.width,
                            rect.height,
                            fixed.width,
                            fixed.height
                        ));
                    }
                    Some(fixed)
                }
                None => None,
            };
            if state.dock == DockSide::Floating && state.float.is_none() {
                state.float = Some(FloatRect::command_search_default());
                diagnostics.push(format!(
                    "{} was floating without a rectangle; placed at the default position",
                    id.title()
                ));
            }
        }
        // The reserved BUILDERS area is not user state: a document cannot move,
        // hide, float or roll it up (OM9-L3-01-01-008), and it stays below the
        // Project Manager. The check runs on the DOCUMENT's own values, before the
        // density pass renumbers the columns for their current membership.
        {
            let reserved = SectionId::BuilderSlot;
            let order = self.section(reserved).order;
            let projects = self.section(SectionId::Projects);
            let below_projects = projects.dock != DockSide::Left || order > projects.order;
            let state = self
                .sections
                .entry(reserved.key().to_owned())
                .or_insert_with(|| reserved.default_state());
            let broken = state.dock != DockSide::Left
                || !state.visible
                || state.collapsed
                || state.float.is_some()
                || !below_projects;
            if broken {
                state.dock = DockSide::Left;
                state.visible = true;
                state.collapsed = false;
                state.float = None;
                // Last in the left column, whatever the rest of the document says:
                // the density pass below turns this into the documented index.
                state.order = u32::MAX;
                diagnostics.push(
                    "Builders is reserved for builder tools; the area was restored below the Project Manager"
                        .into(),
                );
            }
        }
        if self.dense_section_orders() {
            diagnostics
                .push("menu orders were not dense; the documented order was restored".into());
        }
        // A stored home must be a slot its column can have. A document may carry
        // an index from a different arrangement; it is repaired to the documented
        // position with a diagnostic instead of surprising the user on DockHome.
        let left_len = self.left_sections().len() as u32;
        let right_len = self.right_sections().len() as u32;
        for id in SectionId::ALL {
            let Some(home) = self.section(id).home else {
                continue;
            };
            let len = match home.side {
                crate::panel_ops::ColumnSide::Left => left_len,
                crate::panel_ops::ColumnSide::Right => right_len,
            };
            if home.index <= len {
                continue;
            }
            let documented = SectionId::ALL
                .iter()
                .position(|candidate| *candidate == id)
                .unwrap_or(0) as u32;
            if let Some(state) = self.sections.get_mut(id.key()) {
                state.home = Some(crate::panel_ops::SectionHome {
                    side: crate::panel_ops::ColumnSide::Left,
                    index: documented,
                });
            }
            diagnostics.push(format!(
                "{} home {} is outside the {} menu column; the documented position is used",
                id.title(),
                home.index + 1,
                home.side.label()
            ));
        }
        diagnostics
    }

    /// Reset every panel and menu to the documented Matrix baseline while leaving
    /// the document, the command draft and the undo journal untouched.
    pub fn reset_to_default(&mut self) {
        *self = Self::matrix_default();
    }

    /// Keep floating menus reachable: an off-screen or oversized rectangle is
    /// moved and shrunk back inside the usable window area.
    pub fn clamp_floats(&mut self, window: [f32; 2]) -> Vec<String> {
        let mut diagnostics = Vec::new();
        if !window[0].is_finite() || !window[1].is_finite() || window[0] <= 0. || window[1] <= 0. {
            return diagnostics;
        }
        let max_width = (window[0] - 2. * FLOAT_MARGIN).max(FLOAT_MIN_WIDTH);
        let max_height = (window[1] - 2. * FLOAT_MARGIN).max(FLOAT_MIN_HEIGHT);
        let clamp = |rect: FloatRect| {
            let width = rect.width.clamp(FLOAT_MIN_WIDTH, max_width);
            let height = rect.height.clamp(FLOAT_MIN_HEIGHT, max_height);
            let x = rect.x.clamp(
                FLOAT_MARGIN,
                (window[0] - width - FLOAT_MARGIN).max(FLOAT_MARGIN),
            );
            let y = rect.y.clamp(
                FLOAT_MARGIN,
                (window[1] - height - FLOAT_MARGIN).max(FLOAT_MARGIN),
            );
            FloatRect {
                x,
                y,
                width,
                height,
            }
        };
        for id in PanelId::ALL {
            let state = self.state(id);
            let Some(rect) = state.float else {
                continue;
            };
            let clamped = clamp(rect);
            if clamped != rect {
                self.state_mut(id).float = Some(clamped);
                diagnostics.push(format!("{} was moved back inside the window", id.title()));
            }
        }
        for id in SectionId::ALL {
            let state = self.section(id);
            let Some(rect) = state.float else {
                continue;
            };
            let clamped = clamp(rect);
            if clamped != rect {
                self.section_mut(id).float = Some(clamped);
                diagnostics.push(format!("{} was moved back inside the window", id.title()));
            }
        }
        diagnostics
    }

    /// Effective region sizes for a window of `window` pixel size. Pure: the same
    /// input always resolves to the same regions and never returns a negative or
    /// overlapping size.
    pub fn resolve(&self, window: [f32; 2]) -> Regions {
        let width = if window[0].is_finite() {
            window[0].max(0.)
        } else {
            0.
        };
        let height = if window[1].is_finite() {
            window[1].max(0.)
        } else {
            0.
        };
        let toolbar = TOOLBAR_HEIGHT.min(height);
        let status = STATUS_BAR_HEIGHT.min((height - toolbar).max(0.));
        let room = (height - toolbar - status).max(0.);
        let command = if self.state(PanelId::CommandLine).visible {
            self.state(PanelId::CommandLine)
                .size
                .clamp(COMMAND_MIN, COMMAND_MAX)
                .min(room)
        } else {
            0.
        };

        let right_wanted = self
            .right_sections()
            .iter()
            .any(|id| self.section(*id).visible)
            && width >= NARROW_WINDOW_WIDTH;
        let mut right = if right_wanted {
            self.state(PanelId::RightDock)
                .size
                .clamp(RIGHT_DOCK_MIN, RIGHT_DOCK_MAX)
        } else {
            0.
        };
        let mut left = if self.state(PanelId::LeftRail).visible {
            self.state(PanelId::LeftRail)
                .size
                .clamp(LEFT_RAIL_MIN, LEFT_RAIL_MAX)
        } else {
            0.
        };
        if left + right + CANVAS_MIN_WIDTH > width {
            // The optional right column yields first, then the rail shrinks to
            // its minimum, and only then below it as a last resort.
            right = 0.;
            if left + CANVAS_MIN_WIDTH > width {
                left = (width - CANVAS_MIN_WIDTH).clamp(0., LEFT_RAIL_MAX);
            }
        }
        let canvas_width = (width - left - right).max(0.);
        Regions {
            left_rail: left,
            right_dock: right,
            command_line: command,
            toolbar,
            status_bar: status,
            canvas: [canvas_width, (room - command).max(0.)],
            right_dock_visible: right > 0.,
            builder_slot: self.left_sections().contains(&SectionId::BuilderSlot),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|_| Error::Invalid("layout serialization"))
    }

    /// Parse a bounded layout document. A version mismatch resets to the Matrix
    /// default with a diagnostic; malformed bytes are the caller's decision, so
    /// `load` below folds them into the same total behavior.
    ///
    /// The version gate reads the declared version from the raw document, before
    /// the typed parse: a layout written by another version does not have to
    /// satisfy today's field set, and the user must be told which version was
    /// refused instead of seeing "layout document".
    pub fn from_json(bytes: &[u8], limits: &Limits) -> Result<(Self, Vec<String>)> {
        if bytes.len() as u64 > limits.max_bytes {
            return Err(Error::Budget("layout file bytes"));
        }
        om9_archive::admit_json(bytes, limits)?;
        let declared = serde_json::from_slice::<serde_json::Value>(bytes)
            .map_err(|_| Error::Invalid("layout document"))?
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .map(|version| version as u32);
        if declared != Some(SCHEMA_VERSION) {
            let mut diagnostics = vec![format!(
                "layout version {} is not supported; restored the Matrix default",
                declared.unwrap_or(0)
            )];
            let mut default = Self::matrix_default();
            diagnostics.extend(default.normalize());
            return Ok((default, diagnostics));
        }
        let mut layout: Self =
            serde_json::from_slice(bytes).map_err(|_| Error::Invalid("layout document"))?;
        let diagnostics = layout.normalize();
        Ok((layout, diagnostics))
    }

    /// Parse layout bytes with the built-in limits: the entry point tests and
    /// tools use, so the version gate and the diagnostics are reachable without
    /// owning a file path.
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)> {
        Self::from_json(bytes, &Limits::default())
    }

    /// Total load: never fails, never panics. Missing file is a clean default.
    /// Floating rectangles are clamped later against the real window size.
    pub fn load(path: &Path, limits: &Limits) -> (Self, Vec<String>) {
        let unreadable = |error: std::io::Error| {
            (
                Self::matrix_default(),
                vec![format!(
                    "layout unreadable ({error}); restored the Matrix default"
                )],
            )
        };
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return (Self::matrix_default(), Vec::new());
            }
            Err(error) => return unreadable(error),
        };
        if !metadata.is_file() || metadata.len() > limits.max_bytes {
            return (
                Self::matrix_default(),
                vec!["layout ignored (layout file bytes); restored the Matrix default".into()],
            );
        }
        match std::fs::read(path) {
            Ok(bytes) => match Self::from_json(&bytes, limits) {
                Ok(loaded) => loaded,
                Err(error) => (
                    Self::matrix_default(),
                    vec![format!(
                        "layout ignored ({error}); restored the Matrix default"
                    )],
                ),
            },
            Err(error) => unreadable(error),
        }
    }

    /// Replace-on-write through a private staging file. The layout is app-level
    /// preference data, so a failed write is a reported diagnostic, never a
    /// document failure.
    pub fn save(&self, path: &Path) -> Result<()> {
        let bytes = self.to_json()?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
        let name = path
            .file_name()
            .ok_or(Error::Invalid("layout destination filename"))?
            .to_string_lossy()
            .into_owned();
        let staging = parent.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(&staging, bytes.as_bytes()).map_err(|error| Error::Io(error.to_string()))?;
        let published = std::fs::rename(&staging, path);
        if published.is_err() {
            // Windows refuses rename over an existing file; the copy keeps the
            // previous layout replaceable without ever deleting it first.
            let copied = std::fs::copy(&staging, path);
            let _ = std::fs::remove_file(&staging);
            copied.map_err(|error| Error::Io(error.to_string()))?;
        }
        Ok(())
    }

    /// Application-scoped layout location. `OM9_UI_LAYOUT` overrides it so the
    /// app, audits and tests can point at an owned directory.
    pub fn default_store_path() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("OM9_UI_LAYOUT") {
            return Some(PathBuf::from(path));
        }
        let base = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("OpenMatrix9").join("ui-layout.json"))
    }
}

fn size_bounds(id: PanelId) -> (f32, f32) {
    match id {
        PanelId::LeftRail => (LEFT_RAIL_MIN, LEFT_RAIL_MAX),
        PanelId::RightDock => (RIGHT_DOCK_MIN, RIGHT_DOCK_MAX),
        PanelId::CommandLine => (COMMAND_MIN, COMMAND_MAX),
        PanelId::Toolbar => (TOOLBAR_HEIGHT, TOOLBAR_HEIGHT),
        PanelId::CommandSearch => (0., 0.),
    }
}

/// Resolved pixel sizes for one frame. `right_dock` is zero when no menu is
/// docked right, or when the window is too narrow for the mirror column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regions {
    pub left_rail: f32,
    pub right_dock: f32,
    pub command_line: f32,
    pub toolbar: f32,
    pub status_bar: f32,
    pub canvas: [f32; 2],
    pub right_dock_visible: bool,
    pub builder_slot: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            max_bytes: 64 * 1024,
            ..Limits::default()
        }
    }

    #[test]
    fn default_left_column_matches_the_documented_matrix_order() {
        let layout = DockLayout::matrix_default();
        assert_eq!(layout.schema_version, SCHEMA_VERSION);
        assert_eq!(
            layout.left_sections(),
            vec![
                SectionId::IconHistory,
                SectionId::MainMenu,
                SectionId::Display,
                SectionId::Snaps,
                SectionId::InfoSettings,
                SectionId::Layers,
                SectionId::Projects,
                SectionId::BuilderSlot,
            ]
        );
        assert_eq!(SectionId::Projects.title(), "Projects");
        assert_eq!(SectionId::BuilderSlot.title(), "Builders");
        assert!(SectionId::BuilderSlot.is_reserved());
        assert!(SectionId::Snaps.keeps_controls_while_collapsed());
        assert!(layout.right_sections().is_empty());
        assert!(!layout.state(PanelId::RightDock).visible);
        assert_eq!(layout.state(PanelId::LeftRail).size, LEFT_RAIL_DEFAULT);
    }

    #[test]
    fn builder_area_defaults_to_the_space_below_projects() {
        let layout = DockLayout::matrix_default();
        let order = layout.left_sections();
        let projects = order
            .iter()
            .position(|id| *id == SectionId::Projects)
            .unwrap();
        let builders = order
            .iter()
            .position(|id| *id == SectionId::BuilderSlot)
            .unwrap();
        assert_eq!(builders, projects + 1);
        assert!(layout.resolve([1600., 1000.]).builder_slot);
    }

    #[test]
    fn reorder_moves_only_the_requested_menu_and_keeps_other_state() {
        let mut layout = DockLayout::matrix_default();
        layout.set_section_collapsed(SectionId::Layers, true);
        assert_eq!(
            layout.move_section(SectionId::Projects, -1).unwrap(),
            (SectionId::Projects, 5)
        );
        assert_eq!(
            layout.left_sections(),
            vec![
                SectionId::IconHistory,
                SectionId::MainMenu,
                SectionId::Display,
                SectionId::Snaps,
                SectionId::InfoSettings,
                SectionId::Projects,
                SectionId::Layers,
                SectionId::BuilderSlot,
            ]
        );
        assert!(layout.section(SectionId::Layers).collapsed);
        assert_eq!(
            layout.move_section(SectionId::Projects, -99).unwrap(),
            (SectionId::Projects, 0)
        );
        assert_eq!(layout.left_sections()[0], SectionId::Projects);
    }

    #[test]
    fn dock_right_mirrors_the_left_column_and_float_needs_a_rectangle() {
        let mut layout = DockLayout::matrix_default();
        layout
            .dock_section(SectionId::Projects, DockSide::Right)
            .unwrap();
        assert_eq!(layout.right_sections(), vec![SectionId::Projects]);
        assert!(!layout.left_sections().contains(&SectionId::Projects));
        assert!(layout.resolve([1600., 1000.]).right_dock_visible);
        assert_eq!(
            layout.resolve([1600., 1000.]).right_dock,
            RIGHT_DOCK_DEFAULT
        );

        layout.float_section(
            SectionId::Display,
            FloatRect {
                x: 40.,
                y: 60.,
                width: 300.,
                height: 200.,
            },
        );
        assert_eq!(layout.floating_sections(), vec![SectionId::Display]);
        assert!(layout.move_section(SectionId::Display, 1).is_err());

        // A floating menu without a rectangle is repaired, not dropped.
        layout.sections.insert(
            SectionId::Snaps.key().into(),
            SectionState {
                visible: true,
                collapsed: false,
                dock: DockSide::Floating,
                order: 0,
                height: SECTION_DEFAULT_HEIGHT,
                float: None,
                home: None,
            },
        );
        let diagnostics = layout.normalize();
        assert!(diagnostics.iter().any(|d| d.contains("floating without")));
        assert!(layout.section(SectionId::Snaps).float.is_some());
    }

    const _: () = assert!(LEFT_RAIL_MAX == 460.);
    const _: () = assert!(RIGHT_DOCK_MAX == 420.);
    const _: () = assert!(800. >= LEFT_RAIL_MAX + CANVAS_MIN_WIDTH);
    const _: () = assert!(NARROW_WINDOW_WIDTH >= LEFT_RAIL_MAX + RIGHT_DOCK_MAX + CANVAS_MIN_WIDTH);

    #[test]
    fn rail_icon_edge_hits_its_anchors_and_stays_monotone_and_clamped() {
        assert_eq!(rail_icon_edge(240.), 23.);
        assert_eq!(rail_icon_edge(300.), 28.);
        assert_eq!(rail_icon_edge(460.), 42.);
        assert_eq!(rail_icon_edge(0.), ICON_MIN_EDGE);
        assert_eq!(rail_icon_edge(10_000.), ICON_MAX_EDGE);
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(rail_icon_edge(bad), ICON_BASE_EDGE);
        }
        let mut previous = rail_icon_edge(0.);
        let mut width = 0.;
        while width <= 600. {
            let edge = rail_icon_edge(width);
            assert!((ICON_MIN_EDGE..=ICON_MAX_EDGE).contains(&edge), "{width}");
            assert!(edge >= previous, "{width}: {edge} < {previous}");
            previous = edge;
            width += 0.5;
        }
    }

    #[test]
    fn widest_rail_and_widest_right_dock_still_leave_the_canvas_minimum() {
        let mut layout = DockLayout::matrix_default();
        layout
            .dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
            .unwrap();
        layout.set_size(PanelId::LeftRail, LEFT_RAIL_MAX);
        layout.set_size(PanelId::RightDock, RIGHT_DOCK_MAX);
        let exact = layout.resolve([NARROW_WINDOW_WIDTH, 800.]);
        assert!(exact.right_dock_visible);
        assert_eq!(exact.left_rail, 460.);
        assert_eq!(exact.right_dock, 420.);
        assert_eq!(exact.canvas[0], CANVAS_MIN_WIDTH);
        let below = layout.resolve([NARROW_WINDOW_WIDTH - 1., 800.]);
        assert!(!below.right_dock_visible);
        assert!(below.canvas[0] >= CANVAS_MIN_WIDTH);
    }

    #[test]
    fn narrow_window_yields_the_right_column_and_keeps_command_line() {
        let mut layout = DockLayout::matrix_default();
        layout
            .dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
            .unwrap();
        let wide = layout.resolve([1600., 1000.]);
        assert!(wide.right_dock_visible);
        assert_eq!(wide.command_line, COMMAND_DEFAULT);
        let narrow = layout.resolve([800., 600.]);
        assert!(!narrow.right_dock_visible);
        assert_eq!(narrow.right_dock, 0.);
        assert!(narrow.canvas[0] >= CANVAS_MIN_WIDTH);
        let tiny = layout.resolve([260., 200.]);
        assert_eq!(
            tiny.canvas[1],
            200. - TOOLBAR_HEIGHT - STATUS_BAR_HEIGHT - COMMAND_DEFAULT
        );
    }

    #[test]
    fn right_dock_threshold_boundary_is_1200_logical_points() {
        let mut layout = DockLayout::matrix_default();
        layout
            .dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
            .unwrap();
        assert_eq!(NARROW_WINDOW_WIDTH, 1200.);
        assert_eq!(layout.resolve([1199., 800.]).right_dock, 0.);
        assert_eq!(layout.resolve([1199.99, 800.]).right_dock, 0.);
        assert_eq!(layout.resolve([1200., 800.]).right_dock, RIGHT_DOCK_DEFAULT);
    }

    #[test]
    fn resolve_is_total_for_degenerate_and_nonfinite_windows() {
        let layout = DockLayout::matrix_default();
        for window in [
            [0., 0.],
            [-10., -10.],
            [f32::NAN, 100.],
            [100., f32::INFINITY],
        ] {
            let regions = layout.resolve(window);
            assert!(regions.left_rail >= 0.);
            assert!(regions.command_line >= 0.);
            assert!(regions.canvas[0] >= 0.);
            assert!(regions.canvas[1] >= 0.);
        }
    }

    #[test]
    fn unknown_panel_and_section_keys_are_reported_and_repaired() {
        let mut layout = DockLayout::matrix_default();
        layout.panels.insert(
            "right-dock.project-manager".into(),
            PanelState::region(true, 100.),
        );
        layout
            .sections
            .insert("left-dock.legacy".into(), SectionId::Snaps.default_state());
        layout.set_size(PanelId::LeftRail, f32::NAN);
        layout.set_section_height(SectionId::Projects, f32::NAN);
        let diagnostics = layout.normalize();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.contains("right-dock.project-manager"))
        );
        assert!(diagnostics.iter().any(|d| d.contains("left-dock.legacy")));
        assert_eq!(layout.state(PanelId::LeftRail).size, LEFT_RAIL_DEFAULT);
        assert_eq!(
            layout.section(SectionId::Projects).height,
            SECTION_DEFAULT_HEIGHT
        );
        assert_eq!(layout.left_sections().len(), SectionId::ALL.len());
    }

    #[test]
    fn removed_section_is_restored_and_orders_stay_dense() {
        let mut layout = DockLayout::matrix_default();
        layout.sections.remove(SectionId::BuilderSlot.key());
        let diagnostics = layout.normalize();
        assert!(diagnostics.is_empty());
        let order = layout.left_sections();
        assert_eq!(order.last(), Some(&SectionId::BuilderSlot));
        let mut orders: Vec<u32> = order.iter().map(|id| layout.section(*id).order).collect();
        orders.sort_unstable();
        assert_eq!(orders, (0..SectionId::ALL.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn round_trip_persists_menu_identity_visibility_and_order() {
        let mut layout = DockLayout::matrix_default();
        layout.set_section_collapsed(SectionId::Snaps, true);
        layout.set_section_visible(SectionId::InfoSettings, false);
        layout
            .dock_section(SectionId::Projects, DockSide::Right)
            .unwrap();
        layout.set_size(PanelId::LeftRail, 355.5);
        layout.set_float(
            PanelId::CommandSearch,
            Some(FloatRect {
                x: 120.,
                y: 90.,
                width: 640.,
                height: 400.,
            }),
        );
        let json = layout.to_json().unwrap();
        let (restored, diagnostics) = DockLayout::from_json(json.as_bytes(), &limits()).unwrap();
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(restored, layout);
        assert!(restored.section(SectionId::Snaps).collapsed);
        assert!(!restored.section(SectionId::InfoSettings).visible);
        assert_eq!(restored.right_sections(), vec![SectionId::Projects]);
    }

    #[test]
    fn schema_one_and_unknown_versions_reset_to_the_documented_default() {
        // Schema 1 carried the invented right-dock Project Manager panels.
        let v1 = br#"{"schema_version": 1, "panels": {"right-dock.project-manager": {"visible": true, "collapsed": false, "size": 0.0, "order": 1}}}"#;
        let (layout, diagnostics) = DockLayout::from_json(v1, &limits()).unwrap();
        assert_eq!(layout, DockLayout::matrix_default());
        assert!(diagnostics.iter().any(|d| d.contains("version 1")));

        let future = br#"{"schema_version": 99}"#;
        let (layout, diagnostics) = DockLayout::from_json(future, &limits()).unwrap();
        assert_eq!(layout, DockLayout::matrix_default());
        assert!(diagnostics.iter().any(|d| d.contains("99")));
    }

    #[test]
    fn load_reports_missing_corrupt_and_oversized_files_as_default() {
        let directory = std::env::temp_dir().join(format!("om9-layout-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let missing = directory.join("absent.json");
        let (layout, diagnostics) = DockLayout::load(&missing, &limits());
        assert_eq!(layout, DockLayout::matrix_default());
        assert!(diagnostics.is_empty());

        let corrupt = directory.join("corrupt.json");
        std::fs::write(&corrupt, b"{not json").unwrap();
        let (layout, diagnostics) = DockLayout::load(&corrupt, &limits());
        assert_eq!(layout, DockLayout::matrix_default());
        assert!(diagnostics.iter().any(|d| d.contains("layout ignored")));

        let oversized = directory.join("big.json");
        std::fs::write(&oversized, vec![b' '; 70 * 1024]).unwrap();
        let (layout, diagnostics) = DockLayout::load(&oversized, &limits());
        assert_eq!(layout, DockLayout::matrix_default());
        assert!(diagnostics.iter().any(|d| d.contains("layout ignored")));

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn save_reload_and_reset_menus_keep_document_scope_out() {
        let directory = std::env::temp_dir().join(format!("om9-layout-{}", uuid::Uuid::new_v4()));
        let path = directory.join("ui-layout.json");
        let mut layout = DockLayout::matrix_default();
        layout.set_section_collapsed(SectionId::Projects, true);
        layout
            .dock_section(SectionId::Snaps, DockSide::Right)
            .unwrap();
        layout.save(&path).unwrap();

        // Set as Default kept the configuration; Reload Menus reads it back.
        let (mut restored, diagnostics) = DockLayout::load(&path, &limits());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(restored.section(SectionId::Projects).collapsed);
        assert_eq!(restored.right_sections(), vec![SectionId::Snaps]);

        // Reset Menus returns to the documented default in place.
        restored.reset_to_default();
        assert_eq!(restored, DockLayout::matrix_default());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn clamp_floats_recovers_menus_dragged_outside_the_window() {
        let mut layout = DockLayout::matrix_default();
        layout.float_section(
            SectionId::Display,
            FloatRect {
                x: 100_000.,
                y: 100_000.,
                width: 100_000.,
                height: 100_000.,
            },
        );
        let diagnostics = layout.clamp_floats([1280., 800.]);
        assert_eq!(diagnostics.len(), 1);
        let rect = layout.section(SectionId::Display).float.unwrap();
        assert!(rect.width <= 1280. && rect.height <= 800.);
        assert!(rect.x + rect.width <= 1280. && rect.y + rect.height <= 800.);
    }
}
