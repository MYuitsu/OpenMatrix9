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
        let scene = self.scene.as_ref()?;
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
        if !found {
            return None;
        }
        let origin = self
            .gumball_origin
            .unwrap_or_else(|| std::array::from_fn(|i| min[i] * 0.5 + max[i] * 0.5));
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
        let mutations = session.move_mutations(self.document())?;
        self.commit_mutations(name, mutations)
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
        let mutations = session.copy_mutations(self.document())?;
        self.commit_mutations(name, mutations)
    }

    /// The shared transaction path used by both gumball modes.
    ///
    /// A placement change is a geometry-visible edit, so the scene is **rebuilt** afterwards: the cached
    /// scene only allows an in-place update when placements are unchanged, and refusing that here would be
    /// the wrong trade (a transform must be visible immediately).
    fn commit_mutations(&mut self, name: &str, mutations: Vec<Mutation>) -> Result<()> {
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
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
