//! Gumball interaction on the controller side (`OM9-L3-01-01-*`, viewport selection and transform).
//!
//! `om9_core::gumball` owns the maths and `om9_scene::gumball_view` owns the pointer conversation; this
//! module is the only place allowed to turn a finished gesture into document changes, so a drag is one
//! transaction and therefore one Undo entry, exactly like the legacy gumball.
use super::Controller;
use om9_core::{
    Error, Result,
    gumball::{self, Frame},
    identity::ObjectId,
    transaction::{Mutation, TransactionRequest},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) struct BoundsCache {
    document: om9_core::identity::DocumentId,
    epoch: u64,
    selection: std::collections::BTreeSet<ObjectId>,
    centre: Option<om9_core::geometry::Point3>,
}

/// Persisted Gumball preferences (`OM9-INFO-020`). The selection pivot is intentionally not
/// saved: the guide keeps a relocated pivot for the session only.
pub const GUMBALL_SETTINGS_SCHEMA: u32 = 1;
pub const GUMBALL_SETTINGS_ENV: &str = "OM9_GUMBALL_SETTINGS";
const GUMBALL_SETTINGS_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GumballSettings {
    pub schema_version: u32,
    pub snappy: bool,
    pub drag_strength: f64,
    pub alignment: GumballAlignment,
}

impl Default for GumballSettings {
    fn default() -> Self {
        Self {
            schema_version: GUMBALL_SETTINGS_SCHEMA,
            snappy: false,
            drag_strength: 1.0,
            alignment: GumballAlignment::World,
        }
    }
}

impl GumballSettings {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != GUMBALL_SETTINGS_SCHEMA {
            return Err(Error::Invalid("gumball settings schema"));
        }
        if !self.drag_strength.is_finite() || !(0.01..=2.0).contains(&self.drag_strength) {
            return Err(Error::Invalid("gumball settings strength"));
        }
        Ok(())
    }
}

/// Application-scoped settings location; `OM9_GUMBALL_SETTINGS` overrides it for apps/tests.
pub fn default_settings_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(GUMBALL_SETTINGS_ENV) {
        return Some(PathBuf::from(path));
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("OpenMatrix9").join("gumball.json"))
}

pub fn load_settings(path: &Path) -> Result<GumballSettings> {
    let metadata = std::fs::metadata(path).map_err(|error| Error::Io(error.to_string()))?;
    if !metadata.is_file() || metadata.len() > GUMBALL_SETTINGS_BYTES {
        return Err(Error::Invalid("gumball settings file"));
    }
    let bytes = std::fs::read(path).map_err(|error| Error::Io(error.to_string()))?;
    let settings: GumballSettings =
        serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("gumball settings json"))?;
    settings.validate()?;
    Ok(settings)
}

/// Atomic replacement; a failed write leaves the previous file untouched.
pub fn save_settings(path: &Path, settings: &GumballSettings) -> Result<()> {
    settings.validate()?;
    let bytes =
        serde_json::to_vec(settings).map_err(|_| Error::Invalid("gumball settings json"))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
    let name = path
        .file_name()
        .ok_or(Error::Invalid("gumball settings filename"))?
        .to_string_lossy()
        .into_owned();
    let staging = parent.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&staging, bytes).map_err(|error| Error::Io(error.to_string()))?;
    if let Err(error) = std::fs::rename(&staging, path) {
        let _ = std::fs::remove_file(&staging);
        return Err(Error::Io(error.to_string()));
    }
    Ok(())
}

/// Typed Gumball drag mode: `Gumball Snappy` / `Gumball Smooth`, also accepted
/// without the prefix. The Matrix guide names the two modes; the default stays
/// Smooth so existing pointer behaviour does not change silently.
pub fn parse_snappy(text: &str) -> Option<bool> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("Gumball Snappy") || text.eq_ignore_ascii_case("Snappy") {
        Some(true)
    } else if text.eq_ignore_ascii_case("Gumball Smooth") || text.eq_ignore_ascii_case("Smooth") {
        Some(false)
    } else {
        None
    }
}

/// `Gumball DragStrength=50` (percent, guide wording) or `DragStrength=50`.
pub fn parse_drag_strength(text: &str) -> Option<Result<f64>> {
    let lower = text.trim().to_ascii_lowercase();
    let rest = lower.strip_prefix("gumball ").unwrap_or(&lower);
    let value = rest
        .strip_prefix("dragstrength")
        .or_else(|| rest.strip_prefix("drag strength"))?;
    let value = value
        .trim()
        .strip_prefix('=')
        .unwrap_or(value.trim())
        .trim();
    let percent = match value.parse::<f64>() {
        Ok(percent) if percent.is_finite() && (1.0..=200.0).contains(&percent) => percent,
        _ => return Some(Err(Error::Invalid("gumball drag strength"))),
    };
    Some(Ok(percent / 100.0))
}

pub fn parse_restore_defaults(text: &str) -> bool {
    let text = text.trim();
    text.eq_ignore_ascii_case("Gumball RestoreDefaults")
        || text.eq_ignore_ascii_case("Gumball Restore Defaults")
        || text.eq_ignore_ascii_case("RestoreDefaults")
}

impl Controller {
    /// The objects the gumball acts on: the published selection, in document order.
    pub fn gumball_targets(&self) -> Vec<ObjectId> {
        self.selection.iter().copied().collect()
    }

    /// The gumball sits at the centre of the selection, computed from the geometry the scene holds rather
    /// than from `placement`: a Point or a Line carries its coordinates in its buffers, so its placement is
    /// often identity and would put every pivot at the origin.
    ///
    /// `None` when nothing is selected, so the viewport draws no gumball at all.
    pub fn gumball_pivot(&self) -> Option<Frame> {
        use om9_scene::scene::RenderBuffers;
        if self.grips_selected() {
            // Grips on and some picked: the gumball sits at the picked grips (Rhino/Matrix).
            let grips = self.selected_grips();
            if grips.is_empty() {
                return None;
            }
            let (mut min, mut max) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
            for grip in &grips {
                for i in 0..3 {
                    min[i] = min[i].min(grip.point[i]);
                    max[i] = max[i].max(grip.point[i]);
                }
            }
            let origin = self
                .gumball_origin
                .unwrap_or_else(|| std::array::from_fn(|i| min[i] * 0.5 + max[i] * 0.5));
            let axes = match self.gumball_alignment {
                GumballAlignment::CPlane => cplane_frame(self),
                _ => Frame::world(),
            };
            return Some(Frame {
                origin,
                x: axes.x,
                y: axes.y,
                z: axes.z,
            });
        }
        let scene = self.scene.as_ref()?;
        let cached = self.gumball_bounds_cache.borrow();
        let centre = if let Some(cache) = cached.as_ref().filter(|cache| {
            cache.document == scene.document.id
                && cache.epoch == scene.epoch
                && cache.selection == self.selection
        }) {
            cache.centre
        } else {
            drop(cached);
            let (mut min, mut max) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
            let mut found = false;
            for object in &scene.objects {
                if !self.selection.contains(&object.root) || !object.visible || !object.editable {
                    continue;
                }
                let points = match object.buffers.as_ref() {
                    RenderBuffers::Points { points, .. } | RenderBuffers::Curve { points, .. } => {
                        points
                    }
                    RenderBuffers::Mesh { vertices, .. } => vertices,
                };
                let Ok(transform) = object.display_transform() else {
                    continue;
                };
                for point in points {
                    let Ok(world) = transform.point(*point) else {
                        continue;
                    };
                    for i in 0..3 {
                        min[i] = min[i].min(world[i]);
                        max[i] = max[i].max(world[i]);
                    }
                    found = true;
                }
            }
            let centre = found.then(|| std::array::from_fn(|i| min[i] * 0.5 + max[i] * 0.5));
            *self.gumball_bounds_cache.borrow_mut() = Some(BoundsCache {
                document: scene.document.id,
                epoch: scene.epoch,
                selection: self.selection.clone(),
                centre,
            });
            centre
        };
        let origin = self.gumball_origin.unwrap_or(centre?);
        let axes = match self.gumball_alignment {
            GumballAlignment::World => Frame::world(),
            GumballAlignment::CPlane => cplane_frame(self),
            GumballAlignment::Object => {
                object_frame(self, &scene.objects).unwrap_or_else(Frame::world)
            }
        };
        Some(Frame {
            origin,
            x: axes.x,
            y: axes.y,
            z: axes.z,
        })
    }

    /// Moves the gumball to `origin`, or back to the selection centre with `None`.
    pub fn relocate_gumball(&mut self, origin: Option<om9_core::geometry::Point3>) {
        self.gumball_origin = origin;
    }

    /// The origin the user set, if any.
    pub fn gumball_origin(&self) -> Option<om9_core::geometry::Point3> {
        self.gumball_origin
    }

    /// Chooses between world axes and the first selected object's axes.
    pub fn set_gumball_alignment(&mut self, alignment: GumballAlignment) {
        self.gumball_alignment = alignment;
    }

    /// The alignment in use.
    pub fn gumball_alignment(&self) -> GumballAlignment {
        self.gumball_alignment
    }

    /// Snappy drag follows the active object-snap candidate; Smooth follows the
    /// raw pointer ray. Smooth is the default so an ordinary drag is unchanged.
    pub fn gumball_snappy(&self) -> bool {
        self.gumball_snappy
    }

    pub fn set_gumball_snappy(&mut self, snappy: bool) {
        self.gumball_snappy = snappy;
    }

    pub fn gumball_drag_strength(&self) -> f64 {
        self.gumball_drag_strength
    }

    /// Percentage-style drag strength: 1.0 is normal, 0.5 moves half the pointer
    /// distance. Values outside 1%..=200% are refused instead of clamped.
    pub fn set_gumball_drag_strength(&mut self, strength: f64) -> Result<()> {
        if !strength.is_finite() || !(0.01..=2.0).contains(&strength) {
            return Err(Error::Invalid("gumball drag strength"));
        }
        self.gumball_drag_strength = strength;
        Ok(())
    }

    /// Restore the documented gumball defaults: world axes, selection-centre
    /// pivot, smooth drag and 100% strength.
    pub fn restore_gumball_defaults(&mut self) {
        self.gumball_origin = None;
        self.gumball_alignment = GumballAlignment::World;
        self.gumball_snappy = false;
        self.gumball_drag_strength = 1.0;
    }

    pub fn gumball_settings(&self) -> GumballSettings {
        GumballSettings {
            schema_version: GUMBALL_SETTINGS_SCHEMA,
            snappy: self.gumball_snappy,
            drag_strength: self.gumball_drag_strength,
            alignment: self.gumball_alignment,
        }
    }

    pub fn apply_gumball_settings(&mut self, settings: GumballSettings) -> Result<()> {
        settings.validate()?;
        self.set_gumball_drag_strength(settings.drag_strength)?;
        self.gumball_snappy = settings.snappy;
        self.gumball_alignment = settings.alignment;
        Ok(())
    }

    /// Commits a finished gesture as one transaction.
    ///
    /// A gesture that produced no change (a click without motion) writes nothing, so clicking the centre
    /// of the gumball cannot fill the journal with empty entries.
    pub fn commit_gumball(&mut self, session: &gumball::Session, name: &str) -> Result<()> {
        if session.drag_count() == 0
            || session.accumulated() == om9_core::geometry::Transform::identity()
        {
            // A click on a handle that never moved must not become an empty journal entry.
            return Ok(());
        }
        self.validate_gumball_targets(session)?;
        if self.grips_selected() {
            return self.commit_gumball_grips(session);
        }
        let mutations = session.move_mutations(self.document())?;
        self.commit_gumball_mutations(name, mutations)
    }

    /// Copy variant of [`Controller::commit_gumball`]: the selection stays where it is and each target is
    /// duplicated with the gesture's transform applied (the `copy` flag of the legacy gumball).
    pub fn commit_gumball_copy(&mut self, session: &gumball::Session, name: &str) -> Result<()> {
        if session.drag_count() == 0
            || session.accumulated() == om9_core::geometry::Transform::identity()
        {
            return Ok(());
        }
        self.validate_gumball_targets(session)?;
        if self.grips_selected() {
            // Grips cannot be copied: the gesture moves them.
            return self.commit_gumball_grips(session);
        }
        let mutations = session.copy_mutations(self.document())?;
        self.commit_gumball_mutations(name, mutations)
    }

    /// A gumball gesture on picked grips: every grip takes the gesture's transform, and the
    /// edited curves/surfaces commit as one transaction.
    fn commit_gumball_grips(&mut self, session: &gumball::Session) -> Result<()> {
        let delta = session.accumulated();
        let moves = self
            .selected_grips()
            .into_iter()
            .map(|grip| Ok((grip, delta.point(grip.point)?)))
            .collect::<Result<Vec<_>>>()?;
        self.grip_preview = None;
        self.gumball_origin = None;
        self.top11_move_grips(&moves).map(|_| ())
    }

    /// The shared transaction path used by both gumball modes.
    ///
    /// A placement change is a geometry-visible edit, so the scene is **rebuilt** afterwards: the cached
    /// scene only allows an in-place update when placements are unchanged, and refusing that here would be
    /// the wrong trade (a transform must be visible immediately).
    fn commit_gumball_mutations(&mut self, name: &str, mutations: Vec<Mutation>) -> Result<()> {
        if mutations.is_empty() {
            return Ok(());
        }
        if self.scene.is_none() {
            return Err(Error::Missing("gumball scene"));
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    fn validate_gumball_targets(&self, session: &gumball::Session) -> Result<()> {
        if session.targets().len() != self.selection.len()
            || session
                .targets()
                .iter()
                .any(|id| !self.selection.contains(id))
        {
            return Err(Error::Stale("gumball selection"));
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("gumball scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("gumball scene"));
        }
        for id in session.targets() {
            if !scene
                .objects
                .iter()
                .any(|o| o.root == *id && o.visible && o.selectable && o.editable)
            {
                return Err(Error::Permission("gumball target"));
            }
        }
        Ok(())
    }
}

/// A gumball can point along the world axes or along the axes of one selected object, which is what
/// "align to object" means in Matrix9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GumballAlignment {
    #[default]
    World,
    /// The active view's construction plane axes, which is Matrix9's "CPlane" alignment.
    CPlane,
    Object,
}

/// The active view's construction-plane axes, normalised, with `z = x cross y`. The cplane origin is not used
/// here: the gumball pivot stays at the selection, only the axes follow the plane.
fn cplane_frame(controller: &Controller) -> Frame {
    let plane = &controller.workspace.views[&controller.workspace.active].cplane;
    let normal = [
        plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
        plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
        plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
    ];
    Frame {
        origin: [0.0; 3],
        x: plane.x,
        y: plane.y,
        z: normal,
    }
}

/// The axes of the first selected object's placement, normalised, falling back to the world when the
/// placement is degenerate.
fn object_frame(
    controller: &Controller,
    objects: &[om9_scene::scene::SceneObject],
) -> Option<Frame> {
    let object = objects
        .iter()
        .find(|object| controller.selection.contains(&object.root))?;
    let placement = object.display_transform().ok()?.0;
    let axis = |row: [f64; 3]| {
        let length = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
        (length > 1e-12).then(|| [row[0] / length, row[1] / length, row[2] / length])
    };
    let x = axis([placement[0][0], placement[1][0], placement[2][0]])?;
    let y = axis([placement[0][1], placement[1][1], placement[2][1]])?;
    let z = axis([placement[0][2], placement[1][2], placement[2][2]])?;
    Some(Frame {
        origin: [0.0; 3],
        x,
        y,
        z,
    })
}
