// SPDX-License-Identifier: LGPL-2.1-or-later
//! The planar, box-mode T-Spline control cage stores its actual source polygons in the document.
use super::Controller;
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Plane, Reply, Vec3};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Geometry, Mesh},
    identity::ObjectId,
    model::{Entity, Extension, MESH_UV_EXTENSION_SLOT, TSPLINE_BOX_CAGE_SLOT},
    transaction::{Mutation, TransactionRequest},
};
use om9_scene::scene::Subelement;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use om9_addon_geom::subd::selection::{Component, Operation};

fn cage_component(subset: &Subelement) -> Option<Component> {
    match *subset {
        Subelement::MeshFace(f) => Some(Component::Face(f as usize)),
        Subelement::MeshEdge { a, b } => Some(Component::Edge((a, b))),
        Subelement::MeshVertex(v) => Some(Component::Vertex(v)),
        _ => None,
    }
}

#[derive(Serialize, Deserialize)]
struct PlaneInput {
    plane: Plane,
    first: Option<Vec3>,
    opposite: Option<Vec3>,
    center: bool,
    x_faces: usize,
    y_faces: usize,
}

impl PlaneInput {
    fn admit(&self, limits: &Limits) -> Result<()> {
        let count = self
            .x_faces
            .checked_add(1)
            .and_then(|u| self.y_faces.checked_add(1).and_then(|v| u.checked_mul(v)))
            .ok_or(Error::Overflow("T-Spline plane grid"))?;
        if self.x_faces == 0 || self.y_faces == 0 || count > u32::MAX as usize {
            return Err(Error::Invalid("T-Spline face counts"));
        }
        let mut usage = Usage::array::<[u8; 128]>(count)?;
        usage.points = count as u64;
        usage.nodes = (self.x_faces * self.y_faces) as u64;
        usage.check(limits)
    }

    fn corners(&self, opposite: Vec3) -> Result<(Plane, f64, f64, f64, f64)> {
        let plane = self
            .plane
            .moved(self.first.ok_or(Error::Missing("first corner"))?);
        let [u, v, _] = plane.local(opposite);
        if !u.is_finite() || !v.is_finite() || u == 0. || v == 0. {
            return Err(Error::Invalid(
                "pick two different corners with nonzero width and height",
            ));
        }
        let (x, y) = if self.center { (-u, -v) } else { (0., 0.) };
        Ok((plane, x.min(u), x.max(u), y.min(v), y.max(v)))
    }
}

pub(super) fn plane_step(event: &Event, limits: &Limits) -> Result<Reply> {
    let mut state = if matches!(event.input, Input::Start) {
        PlaneInput {
            plane: event.plane.clone(),
            first: None,
            opposite: None,
            center: false,
            x_faces: 2,
            y_faces: 2,
        }
    } else {
        serde_json::from_value(event.state.clone())
            .map_err(|_| Error::Invalid("T-Spline plane session"))?
    };
    let mut reply = Reply::default();
    match &event.input {
        Input::Point { point } if state.first.is_none() => {
            state.first = Some(*point);
            state.plane = event.plane.clone();
        }
        Input::Point { point } => {
            state.corners(*point)?;
            state.opposite = Some(*point);
            reply.done = true;
        }
        Input::Hover { point } if state.first.is_some() => {
            if let Ok((plane, x0, x1, y0, y1)) = state.corners(*point) {
                let corners = [
                    plane.at(x0, y0, 0.),
                    plane.at(x1, y0, 0.),
                    plane.at(x1, y1, 0.),
                    plane.at(x0, y1, 0.),
                ];
                for i in 0..4 {
                    reply.preview.push(Geo::Line {
                        start: corners[i],
                        end: corners[(i + 1) % 4],
                    });
                }
            }
        }
        Input::Option { name, value } => match name.as_str() {
            "Center" if state.first.is_none() => state.center = true,
            "2Point" if state.first.is_none() => state.center = false,
            "XFaces" | "YFaces" => {
                let n = value
                    .as_deref()
                    .and_then(|v| v.parse().ok())
                    .ok_or(Error::Invalid("enter a positive face count"))?;
                if name == "XFaces" {
                    state.x_faces = n;
                } else {
                    state.y_faces = n;
                }
                state.admit(limits)?;
            }
            "OutputType"
                if value
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case("Box")) => {}
            _ => {
                return Err(Error::Unsupported(
                    "T-Spline plane supports Box control cages",
                ));
            }
        },
        Input::Undo => state.first = None,
        Input::Start | Input::Hover { .. } | Input::Selection => {}
        _ => {
            return Err(Error::Invalid(
                "pick a plane corner or choose a plane option",
            ));
        }
    }
    reply.prompt = match (state.first, state.center) {
        (Some(_), _) => "Opposite corner of T-Spline plane",
        (None, true) => "Center of T-Spline plane",
        _ => "First corner of T-Spline plane",
    }
    .into();
    reply.options = vec![
        OptionToken::choice("OutputType", "Box", &["Box"]),
        OptionToken::number("XFaces", Some(state.x_faces as f64)),
        OptionToken::number("YFaces", Some(state.y_faces as f64)),
    ];
    if state.first.is_none() {
        reply.options.push(OptionToken::action(if state.center {
            "2Point"
        } else {
            "Center"
        }));
    }
    reply.state =
        serde_json::to_value(state).map_err(|_| Error::Invalid("T-Spline plane state"))?;
    Ok(reply)
}

impl Controller {
    pub(super) fn start_tspline_plane(&mut self) -> Result<()> {
        self.start_interactive_session("tsPlane", "T-Spline Plane", "om9".to_owned(), false, false)
    }

    pub(super) fn commit_tspline_plane(&mut self, reply: &Reply) -> Result<()> {
        let state: PlaneInput = serde_json::from_value(reply.state.clone())
            .map_err(|_| Error::Invalid("T-Spline plane result"))?;
        let limits = self.coordinator.limits();
        state.admit(limits)?;
        let (plane, x0, x1, y0, y1) =
            state.corners(state.opposite.ok_or(Error::Missing("opposite corner"))?)?;
        let k = self.document().page_unit.factor();
        let rows = (0..=state.x_faces)
            .map(|i| {
                let u = x0 + (x1 - x0) * i as f64 / state.x_faces as f64;
                (0..=state.y_faces)
                    .map(|j| {
                        let v = y0 + (y1 - y0) * j as f64 / state.y_faces as f64;
                        plane.at(u, v, 0.).map(|x| x / k)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let grid = om9_addon_geom::subd::grid(&rows, false, false);
        let mesh = Mesh {
            vertices: grid.verts,
            faces: grid.faces,
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        };
        mesh.validate(limits)?;
        let mut entity = Entity::new(
            "T-Spline Plane",
            Geometry::Mesh(mesh),
            self.document().current_layer,
        );
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        entity.extensions.insert(
            TSPLINE_BOX_CAGE_SLOT.into(),
            Extension::new(1, om9_core::model::TSPLINE_BOX_METADATA)?,
        );
        let id = entity.id;
        self.publish_request_with_selection(
            TransactionRequest::new(
                "T-Spline Plane",
                vec![Mutation::Insert {
                    document: self.document,
                    entity,
                }],
            ),
            Some(&[id]),
        )
    }

    pub(crate) fn tspline_component_kind(&self) -> Option<om9_scene::scene::PickKind> {
        use om9_scene::scene::PickKind;
        let epoch = self.coordinator.epoch();
        let kind = |p: &super::SelectedSubobject| {
            if !p.path.is_empty()
                || p.world_epoch != epoch
                || !self
                    .document()
                    .entities
                    .get(&p.root)
                    .is_some_and(om9_core::model::is_tspline_box_cage)
            {
                return None;
            }
            match p.subset {
                Subelement::MeshVertex(_) => Some(PickKind::Vertex),
                Subelement::MeshEdge { .. } => Some(PickKind::Edge),
                Subelement::MeshFace(_) => Some(PickKind::Face),
                _ => None,
            }
        };
        let (first, rest) = self.subselection().split_first()?;
        let first = kind(first)?;
        rest.iter().all(|p| kind(p) == Some(first)).then_some(first)
    }

    pub fn tspline_faces_selected(&self) -> bool {
        self.tspline_component_kind() == Some(om9_scene::scene::PickKind::Face)
            || self.from_lines_faces_selected()
    }

    pub(crate) fn from_lines_faces_selected(&self)->bool {
        !self.subselection().is_empty() && self.subselection().iter().all(|p|{
            p.path.is_empty() && p.world_epoch==self.coordinator.epoch() && matches!(p.subset,Subelement::MeshFace(_))
                && self.document().entities.get(&p.root).is_some_and(|e|e.extension(om9_core::model::TSPLINE_FROM_LINES_SLOT).is_some() || om9_core::model::is_tspline_box_cage(e))
        })
    }

    pub fn cage_paint_mode(&self) -> Option<om9_scene::scene::PickKind> {
        self.cage_paint_mode
            .filter(|(doc, _)| *doc == self.document)
            .map(|(_, kind)| kind)
    }

    pub(super) fn toggle_cage_paint(&mut self) -> Result<()> {
        if self.cage_paint_mode().is_some() {
            self.cage_paint_mode = None;
            self.addon_messages.push("Paint Selection off".into());
        } else {
            let kind = self
                .tspline_component_kind()
                .ok_or(Error::Invalid("Select T-Spline components to paint"))?;
            self.cage_paint_mode = Some((self.document, kind));
            self.addon_messages.push(
                "Paint Selection on. Drag across components to add them. Esc finishes.".into(),
            );
        }
        Ok(())
    }

    pub(super) fn cage_selection_reason(&self, op: Operation) -> Option<&'static str> {
        use om9_scene::scene::PickKind;
        match (op, self.tspline_component_kind()) {
            (Operation::Ring, Some(PickKind::Edge)) => None,
            (Operation::Ring, _) => Some("Select T-Spline edges"),
            (Operation::Loop, Some(PickKind::Face | PickKind::Vertex))
                if self.subselection().len() < 2 =>
            {
                Some("Select two adjacent T-Spline faces or vertices")
            }
            (_, Some(_)) => None,
            _ => Some("Select T-Spline components"),
        }
    }

    pub(crate) fn select_cage_components(&mut self, operation: Operation) -> Result<()> {
        if let Some(reason) = self.cage_selection_reason(operation) {
            return Err(Error::Invalid(reason));
        }
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("T-Spline scene"))?;
        let mut owners = BTreeMap::new();
        for pick in self.subselection() {
            let key = cage_component(&pick.subset).ok_or(Error::Invalid("T-Spline component"))?;
            // All keys were validated on entry. For the same immutable source publication
            // a second visibility/path lookup or linear search of display edges is redundant.
            match owners.entry(pick.root) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    pick.validate(scene, self.coordinator.epoch())?;
                    entry.insert((pick.clone(), BTreeSet::from([key])));
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let (seed, selected) = entry.get_mut();
                    if seed.source != pick.source || seed.world_epoch != pick.world_epoch {
                        return Err(Error::Stale("T-Spline selection"));
                    }
                    selected.insert(key);
                }
            }
        }
        let mut additions = Vec::new();
        let mut results = BTreeMap::new();
        for (id, (seed, original)) in owners {
            let Geometry::Mesh(mesh) = &self.document().entities[&id].geometry else {
                return Err(Error::Invalid("T-Spline cage"));
            };
            let corners = mesh
                .faces
                .len()
                .checked_mul(4)
                .ok_or(Error::Overflow("cage adjacency"))?;
            Usage::array::<[u8; 192]>(corners)?.check(self.coordinator.limits())?;
            let selected =
                om9_addon_geom::subd::selection::select(&mesh.faces, &original, operation)
                    .map_err(Error::Invalid)?;
            additions
                .try_reserve(selected.difference(&original).count())
                .map_err(|_| Error::Budget("cage selection"))?;
            for key in selected.difference(&original) {
                let subset = match *key {
                    Component::Face(f) => Subelement::MeshFace(
                        u32::try_from(f).map_err(|_| Error::Overflow("face index"))?,
                    ),
                    Component::Vertex(v) => Subelement::MeshVertex(v),
                    Component::Edge((a, b)) => Subelement::MeshEdge { a, b },
                };
                additions.push(super::SelectedSubobject {
                    subset,
                    ..seed.clone()
                });
            }
            results.insert(id, selected);
        }
        self.update_cage_subselection(additions, |pick| {
            cage_component(&pick.subset).is_some_and(|key| {
                results
                    .get(&pick.root)
                    .is_some_and(|set| set.contains(&key))
            })
        })
    }

    pub(super) fn edit_tspline_faces(&mut self, side: om9_core::top11::MeshFaceSide) -> Result<()> {
        use om9_core::top11::{MeshFaceSide, mesh_face_subset};
        if !self.tspline_faces_selected() {
            return Err(Error::Invalid("select T-Spline control faces"));
        }
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("T-Spline scene"))?;
        let mut by_owner = BTreeMap::<ObjectId, BTreeSet<u32>>::new();
        for pick in self.subselection() {
            pick.validate(scene, self.coordinator.epoch())?;
            let Subelement::MeshFace(face) = pick.subset else {
                return Err(Error::Invalid("T-Spline face"));
            };
            by_owner.entry(pick.root).or_default().insert(face);
        }
        let mut mutations = Vec::new();
        let mut selected = Vec::new();
        for (id, faces) in by_owner {
            let entity = &self.document().entities[&id];
            if !self
                .document()
                .permissions_for_entity(entity, false)?
                .editable
            {
                return Err(Error::Permission("T-Spline cage"));
            }
            let Geometry::Mesh(mesh) = &entity.geometry else {
                return Err(Error::Invalid("T-Spline cage"));
            };
            let limits = self.coordinator.limits();
            let source = self.document().stamp(id)?;
            let mut extensions = entity.extensions.clone();
            let new_mesh=if entity.extension(om9_core::model::TSPLINE_FROM_LINES_SLOT).is_some() {
                super::from_lines::face_subset(entity,&faces,side,limits)?.map(|(mesh,ext)|{
                    extensions.insert(om9_core::model::TSPLINE_FROM_LINES_SLOT.into(),ext);
                    mesh
                })
            } else {
                let texture = om9_core::rhino_semantics::mesh_uv_of(entity, limits)?;
                mesh_face_subset(mesh,&faces,side,texture,limits)?.map(|part|->Result<Mesh>{
                    if let Some(uv)=part.uv {extensions.insert(MESH_UV_EXTENSION_SLOT.into(),uv.to_extension()?);}
                    Ok(part.mesh)
                }).transpose()?
            };
            let Some(new_mesh)=new_mesh else {
                mutations.push(Mutation::Delete { source });
                continue;
            };
            let geometry = Geometry::Mesh(new_mesh);
            match side {
                MeshFaceSide::Remaining => {
                    if extensions != entity.extensions {
                        mutations.push(Mutation::Extensions { source, extensions });
                    }
                    mutations.push(Mutation::Geometry { source, geometry });
                }
                MeshFaceSide::Selected => {
                    let mut copy = Entity::new(&entity.label, geometry, entity.attributes.layer);
                    copy.placement = entity.placement;
                    copy.attributes = entity.attributes.clone();
                    copy.extensions = extensions;
                    selected.push(copy.id);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: copy,
                    });
                }
            }
        }
        let label = match side {
            MeshFaceSide::Selected => "Duplicate Face(s)",
            MeshFaceSide::Remaining => "Delete Face(s)",
        };
        self.publish_request_with_selection(
            TransactionRequest::new(label, mutations),
            Some(&selected),
        )?;
        self.clear_subselection();
        Ok(())
    }

    pub(crate) fn delete_current_selection(&mut self) -> Result<()> {
        if self.subselection().iter().any(|pick| {
            self.document()
                .entities
                .get(&pick.root)
                .is_some_and(|e|om9_core::model::is_tspline_box_cage(e) || e.extension(om9_core::model::TSPLINE_FROM_LINES_SLOT).is_some())
        }) {
            return self.edit_tspline_faces(om9_core::top11::MeshFaceSide::Remaining);
        }
        self.delete_objects(&self.selected_ordered()).map(|_| ())
    }
}
