// SPDX-License-Identifier: LGPL-2.1-or-later
//! T-Splines manipulator on authored polygon cages. Gestures edit unique source vertices.
use super::Controller;
use om9_addon_abi::session::{Event, Input, OptionToken, Reply, Vec3, Want};
use om9_core::{
    Error, Result,
    budget::Usage,
    geometry::Geometry,
    gumball::{Axis, Drag, Handle, Session},
    identity::ObjectId,
    transaction::{Mutation, TransactionRequest},
};
use om9_scene::{gumball_view::Mode, scene::Subelement};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn numeric_axis(code: &str) -> Option<Axis> {
    match code {
        "tsManipHandleX" => Some(Axis::X),
        "tsManipHandleY" => Some(Axis::Y),
        "tsManipHandleZ" => Some(Axis::Z),
        _ => None,
    }
}

impl Controller {
    pub(super) fn start_cage_pivot(&mut self) -> Result<()> {
        if !self.cage_object_manipulator_ready() {
            return Err(Error::Invalid("Select a T-Spline cage or its components"));
        }
        self.start_interactive_session("tsManipSetPivot", "Set Pivot", "om9".into(), false, false)
    }

    pub(super) fn cage_pivot_step(&self, event: &Event) -> Result<Reply> {
        let state = serde_json::json!([
            self.document,
            self.coordinator.epoch(),
            self.selection_revision(),
            self.sub_revision()
        ]);
        if event.input != Input::Start && state != event.state {
            return Err(Error::Stale("T-Spline pivot selection changed"));
        }
        let done = match &event.input {
            Input::Point { point } if point.iter().all(|v| v.is_finite()) => true,
            Input::Start | Input::Hover { .. } | Input::Selection => false,
            _ => return Err(Error::Invalid("Pick a pivot point")),
        };
        Ok(Reply {
            prompt: "Pick a point for the manipulator pivot".into(),
            state,
            done,
            ..Default::default()
        })
    }

    pub(super) fn set_cage_pivot(&mut self, point: Vec3) -> Result<()> {
        if self.cage_manipulator_mode().is_none() {
            let kind = self
                .tspline_component_kind()
                .unwrap_or(om9_scene::scene::PickKind::Object);
            self.start_cage_manipulator(om9_scene::gumball_view::Mode::Move, kind)?;
        }
        self.relocate_gumball(Some(point));
        Ok(())
    }

    fn whole_cages_selected(&self) -> bool {
        !self.selected().is_empty()
            && self.selected().iter().all(|id| {
                self.document()
                    .entities
                    .get(id)
                    .is_some_and(om9_core::model::is_tspline_box_cage)
            })
    }

    pub(super) fn cage_object_manipulator_ready(&self) -> bool {
        if self.subselection().is_empty() {
            self.whole_cages_selected()
        } else {
            self.tspline_component_kind().is_some()
        }
    }

    pub(super) fn start_cage_manipulator(
        &mut self,
        mode: om9_scene::gumball_view::Mode,
        kind: om9_scene::scene::PickKind,
    ) -> Result<()> {
        if !self.cage_object_manipulator_ready() {
            return Err(Error::Invalid("Select a T-Spline cage or its components"));
        }
        if !self.subselection().is_empty() {
            let ids: BTreeSet<_> = self.subselection().iter().map(|p| p.root).collect();
            self.select_objects(&ids.into_iter().collect::<Vec<_>>())?;
            if self.tspline_component_kind() != Some(kind) {
                self.clear_subselection();
            }
        }
        self.clear_gumball_preview();
        self.cage_paint_mode = None;
        if self.grips_selected() {
            self.select_grips(&[], super::GripPick::Replace);
        }
        self.cage_manipulator = Some((self.document, mode, kind));
        Ok(())
    }

    pub(crate) fn cage_manipulator_mode(&self) -> Option<om9_scene::gumball_view::Mode> {
        let (doc, mode, kind) = self.cage_manipulator?;
        if doc != self.document {
            return None;
        }
        let active = if self.subselection().is_empty() {
            self.whole_cages_selected()
        } else {
            self.tspline_component_kind() == Some(kind)
        };
        active.then_some(mode)
    }

    pub(crate) fn cage_pick_kind(&self) -> Option<om9_scene::scene::PickKind> {
        self.cage_manipulator.and_then(|(doc, _, kind)| {
            (doc == self.document && kind != om9_scene::scene::PickKind::Object).then_some(kind)
        })
    }

    pub(crate) fn cage_component_mode(&self) -> Option<om9_scene::scene::PickKind> {
        let (doc, _, kind) = self.cage_manipulator?;
        (doc == self.document
            && kind != om9_scene::scene::PickKind::Object
            && self.cage_manipulator_mode().is_some())
        .then_some(kind)
    }

    pub(crate) fn clear_cage_manipulator(&mut self) {
        self.cage_manipulator = None;
        self.clear_gumball_preview();
    }

    fn cage_vertex_indices(&self) -> Result<BTreeMap<ObjectId, BTreeSet<u32>>> {
        let scene = self.scene().ok_or(Error::Missing("T-Spline scene"))?;
        let mut owners = BTreeMap::<_, BTreeSet<_>>::new();
        for pick in self.subselection() {
            use std::collections::btree_map::Entry;
            let vertices = match owners.entry(pick.root) {
                Entry::Vacant(entry) => {
                    if !pick.validate(scene, self.coordinator.epoch())?.editable {
                        return Err(Error::Permission("T-Spline cage"));
                    }
                    entry.insert(BTreeSet::new())
                }
                Entry::Occupied(entry) => {
                    // Every component was validated when picked. A matching stamp keeps
                    // its indices valid without searching the same edge buffer again.
                    if pick.world_epoch != scene.epoch
                        || !pick.path.is_empty()
                        || pick.source != scene.document.stamp(pick.root)?
                    {
                        return Err(Error::Stale("T-Spline component"));
                    }
                    entry.into_mut()
                }
            };
            let Geometry::Mesh(mesh) = &scene.document.entities[&pick.root].geometry else {
                return Err(Error::Invalid("T-Spline cage"));
            };
            match pick.subset {
                Subelement::MeshVertex(v) => {
                    vertices.insert(v);
                }
                Subelement::MeshEdge { a, b } => {
                    vertices.extend([a, b]);
                }
                Subelement::MeshFace(f) => {
                    vertices.extend(&mesh.faces[f as usize]);
                }
                _ => return Err(Error::Invalid("T-Spline component")),
            }
        }
        if owners.is_empty() {
            return Err(Error::Missing("T-Spline components"));
        }
        Ok(owners)
    }

    pub(super) fn cage_component_center(&self) -> Option<[f64; 3]> {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for (id, selected) in self.cage_vertex_indices().ok()? {
            let e = &self.document().entities[&id];
            let Geometry::Mesh(mesh) = &e.geometry else {
                return None;
            };
            for v in selected {
                let p = e.placement.point(mesh.vertices[v as usize]).ok()?;
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
            }
        }
        Some(std::array::from_fn(|i| lo[i] * 0.5 + hi[i] * 0.5))
    }

    pub fn cage_preview_edges(&self) -> &[[Vec3; 2]] {
        &self.cage_preview_edges
    }

    pub(super) fn preview_cage_vertices(
        &self,
        delta: om9_core::geometry::Transform,
    ) -> Result<Vec<[Vec3; 2]>> {
        let mut lines = Vec::new();
        for (id, selected) in self.cage_vertex_indices()? {
            let entity = &self.document().entities[&id];
            let Geometry::Mesh(mesh) = &entity.geometry else {
                return Err(Error::Invalid("T-Spline cage"));
            };
            let count = mesh.faces.iter().try_fold(0usize, |n, f| {
                n.checked_add(f.len())
                    .ok_or(Error::Overflow("cage corners"))
            })?;
            Usage::array::<[u8; 64]>(count)?.check(self.coordinator.limits())?;
            let edges = mesh
                .faces
                .iter()
                .flat_map(|face| {
                    face.iter()
                        .copied()
                        .zip(face.iter().copied().cycle().skip(1))
                })
                .filter(|(a, b)| selected.contains(a) || selected.contains(b))
                .map(|(a, b)| om9_addon_geom::subd::edge(a, b))
                .collect::<BTreeSet<_>>();
            let total = lines
                .len()
                .checked_add(edges.len())
                .ok_or(Error::Overflow("cage preview"))?;
            Usage::array::<[Vec3; 2]>(total)?.check(self.coordinator.limits())?;
            lines
                .try_reserve(edges.len())
                .map_err(|_| Error::Budget("cage preview"))?;
            let world = |v: u32| {
                let p = entity.placement.point(mesh.vertices[v as usize])?;
                if selected.contains(&v) {
                    delta.point(p)
                } else {
                    Ok(p)
                }
            };
            for (a, b) in edges {
                lines.push([world(a)?, world(b)?]);
            }
        }
        Ok(lines)
    }

    pub(super) fn commit_cage_vertices(
        &mut self,
        delta: om9_core::geometry::Transform,
        name: &str,
    ) -> Result<()> {
        let mut mutations = Vec::new();
        for (id, selected) in self.cage_vertex_indices()? {
            let entity = &self.document().entities[&id];
            let Geometry::Mesh(source) = &entity.geometry else {
                return Err(Error::Invalid("T-Spline cage"));
            };
            let inverse = om9_core::top11::inverse_affine(entity.placement)?;
            let mut mesh = source.clone();
            let mut changed = false;
            for v in selected {
                let point = &mut mesh.vertices[v as usize];
                let world = entity.placement.point(*point)?;
                let moved = delta.point(world)?;
                if moved != world {
                    *point = inverse.point(moved)?;
                    changed = true;
                }
            }
            if !changed {
                continue;
            }
            mesh.normals.clear();
            mesh.validate(self.coordinator.limits())?;
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(id)?,
                geometry: Geometry::Mesh(mesh),
            });
        }
        if mutations.is_empty() {
            self.clear_gumball_preview();
            return Ok(());
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        if self.commit_with_candidate_scene(prepared, held)?.published {
            // Connectivity is unchanged by a vertex-only edit. Restamp the same
            // components before normal publication prunes stale references.
            let scene = self.scene.as_ref().expect("published cage scene");
            for pick in &mut self.subselection {
                if let Ok(stamp) = scene.document.stamp(pick.root) {
                    pick.source = stamp;
                    pick.world_epoch = scene.epoch;
                }
            }
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        self.clear_gumball_preview();
        Ok(())
    }

    pub(crate) fn start_cage_handle_entry(&mut self, axis: Axis) -> Result<()> {
        if self.cage_manipulator_mode().is_none() || self.gumball_pivot().is_none() {
            return Err(Error::Missing("T-Spline manipulator selection"));
        }
        let code = match axis {
            Axis::X => "tsManipHandleX",
            Axis::Y => "tsManipHandleY",
            Axis::Z => "tsManipHandleZ",
        };
        self.start_interactive_session(code, "T-Spline manipulator", "om9".into(), false, false)
    }

    pub(super) fn cage_number_step(&self, axis: Axis, event: &Event) -> Result<Reply> {
        let basis = serde_json::json!([
            self.document,
            self.coordinator.epoch(),
            self.selection_revision(),
            self.sub_revision(),
            self.gumball_origin()
        ]);
        let mut state = if matches!(event.input, Input::Start) {
            let style = match self
                .cage_manipulator_mode()
                .ok_or(Error::Missing("T-Spline manipulator"))?
            {
                Mode::Move => "Translate",
                Mode::Rotate => "Rotate",
                Mode::Scale => "Scale",
            };
            serde_json::json!({"basis":basis,"Handle":match axis {Axis::X=>"X",Axis::Y=>"Y",Axis::Z=>"Z"},"Style":style})
        } else {
            if event.state["basis"] != basis {
                return Err(Error::Stale("T-Spline manipulator selection changed"));
            }
            event.state.clone()
        };
        if let Input::Option { name, value } = &event.input {
            let choices: &[&str] = match name.as_str() {
                "Style" => &["Translate", "Rotate", "Scale"],
                "Handle" => &["X", "Y", "Z"],
                _ => return Err(Error::Invalid("manipulator option")),
            };
            let choice = choices
                .iter()
                .find(|v| {
                    value
                        .as_deref()
                        .is_some_and(|value| v.eq_ignore_ascii_case(value))
                })
                .ok_or(Error::Invalid("manipulator choice"))?;
            state[name.as_str()] = serde_json::Value::from(*choice);
        }
        let axis = state["Handle"]
            .as_str()
            .ok_or(Error::Invalid("manipulator axis"))?;
        let style = state["Style"]
            .as_str()
            .ok_or(Error::Invalid("manipulator style"))?;
        let prompt = match style {
            "Translate" => "Distance",
            "Rotate" => "Angle in degrees",
            "Scale" => "Scale factor",
            _ => return Err(Error::Invalid("manipulator style")),
        };
        let options = vec![
            OptionToken::choice("Handle", axis, &["X", "Y", "Z"]),
            OptionToken::choice("Style", style, &["Translate", "Rotate", "Scale"]),
        ];
        let done = match &event.input {
            Input::Text { text } => {
                let n = text
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|n| n.is_finite() && (style != "Scale" || *n > 0.))
                    .ok_or(Error::Invalid("Enter a valid manipulator value"))?;
                state["value"] = serde_json::Value::from(n);
                true
            }
            Input::Start | Input::Hover { .. } | Input::Selection | Input::Option { .. } => false,
            _ => return Err(Error::Invalid("Type a distance, angle or scale factor")),
        };
        Ok(Reply {
            prompt: prompt.into(),
            options,
            state,
            want: Want::Number,
            done,
            ..Default::default()
        })
    }

    pub(super) fn commit_cage_number(&mut self, reply: &Reply) -> Result<()> {
        let axis = match reply.state["Handle"].as_str() {
            Some("X") => Axis::X,
            Some("Y") => Axis::Y,
            Some("Z") => Axis::Z,
            _ => return Err(Error::Invalid("manipulator axis")),
        };
        let n = reply.state["value"]
            .as_f64()
            .ok_or(Error::Invalid("manipulator value"))?;
        let frame = self
            .gumball_pivot()
            .ok_or(Error::Missing("T-Spline pivot"))?;
        let (mode, handle, motion, name) = match reply.state["Style"].as_str() {
            Some("Translate") => (
                Mode::Move,
                Handle::MoveAxis(axis),
                Drag::WorldDelta(frame.axis(axis).map(|v| v * n)),
                "Translate",
            ),
            Some("Rotate") => (
                Mode::Rotate,
                Handle::Rotate(axis),
                Drag::Angle(n.to_radians()),
                "Rotate",
            ),
            Some("Scale") => (Mode::Scale, Handle::Scale(axis), Drag::Factor(n), "Scale"),
            _ => return Err(Error::Invalid("manipulator style")),
        };
        let mut session = Session::begin(frame, self.gumball_targets(), false)?;
        session.drag(handle, motion)?;
        self.commit_gumball(&session, name)?;
        if let Some((_, active, _)) = &mut self.cage_manipulator {
            *active = mode;
        }
        Ok(())
    }
}
