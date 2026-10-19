// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::camera::{CPlane, Camera};
use om9_core::Result;
use om9_core::{Error, identity::SessionId};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
// Cloned layout snapshots share this runtime clock; restoring a snapshot never
// restores the authority of a popup opened before an intervening layout action.
#[derive(Debug, Clone, Default)]
struct LayoutClock(Arc<AtomicU64>, SessionId);
impl PartialEq for LayoutClock {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViewId {
    LookingDown,
    Perspective,
    SideView,
    ThroughFinger,
}
impl ViewId {
    pub const ALL: [Self; 4] = [
        Self::LookingDown,
        Self::Perspective,
        Self::ThroughFinger,
        Self::SideView,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::LookingDown => "Looking Down",
            Self::Perspective => "Perspective",
            Self::SideView => "Side View",
            Self::ThroughFinger => "Through Finger",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Wireframe,
    Shaded,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewState {
    pub id: ViewId,
    pub camera: Camera,
    pub cplane: CPlane,
    pub mode: DisplayMode,
    pub revision: u64,
    #[serde(skip)]
    runtime_owner: SessionId,
}
impl ViewState {
    pub fn new(id: ViewId, camera: Camera, cplane: CPlane) -> Self {
        Self {
            id,
            camera,
            cplane,
            mode: DisplayMode::Wireframe,
            revision: 1,
            runtime_owner: SessionId::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub views: BTreeMap<ViewId, ViewState>,
    pub active: ViewId,
    pub maximized: Option<ViewId>,
    #[serde(skip)]
    layout_revision: LayoutClock,
}
impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
impl Workspace {
    /// Heap owners used by the app's aggregate scene/lifecycle admission.
    pub fn storage_usage(&self) -> Result<om9_core::budget::Usage> {
        Self::usage_for(self.views.len())
    }
    pub fn startup_usage() -> Result<om9_core::budget::Usage> {
        Self::usage_for(ViewId::ALL.len())
    }
    fn usage_for(views: usize) -> Result<om9_core::budget::Usage> {
        use om9_core::budget::Usage;
        let mut usage = Usage::btree::<ViewId, ViewState>(views)?;
        let clock = std::mem::size_of::<AtomicU64>() + 2 * std::mem::size_of::<usize>();
        usage.bytes = usage
            .bytes
            .checked_add(clock as u64)
            .ok_or(Error::Overflow("workspace storage"))?;
        Ok(usage)
    }
    pub fn new() -> Self {
        // Explicit initial OM9 numerical camera choices; view names/planes and
        // finite grid follow Matrix9. Pixel/preset parity remains a UI gate.
        let views = [
            ViewState::new(
                ViewId::LookingDown,
                Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 41.)
                    .expect("constant camera"),
                CPlane::world_xy(),
            ),
            ViewState::new(
                ViewId::Perspective,
                Camera::perspective([35., -35., 35.], [0.; 3], [0., 0., 1.], 35f64.to_radians())
                    .expect("constant camera"),
                CPlane::world_xy(),
            ),
            ViewState::new(
                ViewId::ThroughFinger,
                Camera::orthographic([0., -100., 0.], [0.; 3], [0., 0., 1.], 41.)
                    .expect("constant camera"),
                CPlane::new([0.; 3], [1., 0., 0.], [0., 0., 1.]).expect("constant CPlane"),
            ),
            ViewState::new(
                ViewId::SideView,
                Camera::orthographic([100., 0., 0.], [0.; 3], [0., 0., 1.], 41.)
                    .expect("constant camera"),
                CPlane::new([0.; 3], [0., 1., 0.], [0., 0., 1.]).expect("constant CPlane"),
            ),
        ]
        .into_iter()
        .map(|v| (v.id, v))
        .collect();
        Self {
            views,
            active: ViewId::Perspective,
            maximized: None,
            layout_revision: LayoutClock::default(),
        }
    }
    pub fn toggle_maximize(&mut self, id: ViewId) -> Result<()> {
        let view = self
            .views
            .get(&id)
            .ok_or(Error::Missing("workspace view"))?;
        if view.id != id {
            return Err(Error::Invalid("workspace view key/identity"));
        }
        self.layout_revision
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| Error::Overflow("view layout revision"))?;
        self.active = id;
        self.maximized = if self.maximized == Some(id) {
            None
        } else {
            Some(id)
        };
        Ok(())
    }
    pub fn layout_revision(&self) -> u64 {
        self.layout_revision.0.load(Ordering::Relaxed)
    }
    pub fn layout_owner(&self) -> SessionId {
        self.layout_revision.1
    }
    pub fn visible_views(&self) -> Vec<ViewId> {
        self.maximized
            .map_or_else(|| ViewId::ALL.to_vec(), |id| vec![id])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_overflow_rejects_before_active_maximized_or_view_state_changes() {
        let mut w = Workspace::new();
        w.layout_revision.0.store(u64::MAX, Ordering::Relaxed);
        let before = w.clone();
        assert!(matches!(
            w.toggle_maximize(ViewId::SideView),
            Err(Error::Overflow(_))
        ));
        assert_eq!(w, before);
    }
}
