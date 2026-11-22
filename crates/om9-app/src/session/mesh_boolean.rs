// SPDX-License-Identifier: LGPL-2.1-or-later
//! Selection and options for Rhino mesh Booleans. Geometry and publication use the
//! same CSG worker and transaction path as Boolean Builder.
use om9_addon_abi::session::{Event, Input, OptionToken, Reply, Want};
use om9_core::{Error, Result};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshBoolean {
    Union,
    Difference,
    Intersection,
}

impl MeshBoolean {
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Union, Self::Difference, Self::Intersection]
            .into_iter()
            .find(|op| name.eq_ignore_ascii_case(op.name()))
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Union => "MeshBooleanUnion",
            Self::Difference => "MeshBooleanDifference",
            Self::Intersection => "MeshBooleanIntersection",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Union => "Mesh Union",
            Self::Difference => "Mesh Difference",
            Self::Intersection => "Mesh Intersection",
        }
    }

    /// A completed selection supplies an event to the existing Boolean worker.
    pub(super) fn step(self, event: &mut Event) -> Result<Option<Reply>> {
        let mut state = std::mem::take(&mut event.state);
        if state.is_null() {
            state = json!({});
        }
        let mut keep = state.get("keep").and_then(Value::as_bool).unwrap_or(false);
        if let Input::Option { name, value } = &event.input {
            if !name.eq_ignore_ascii_case("DeleteInput") {
                return Err(Error::Invalid("mesh Boolean option"));
            }
            keep = match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                Some("no" | "n") => true,
                Some("yes" | "y") => false,
                None => !keep,
                _ => return Err(Error::Invalid("DeleteInput expects Yes or No")),
            };
            state["keep"] = keep.into();
        }

        let union = self == Self::Union;
        let mut first = state.get("objects").is_none();
        if matches!(event.input, Input::Enter) {
            if event.selection.is_empty() {
                return Err(Error::Missing("Boolean selection"));
            }
            if union && event.selection.len() < 2 {
                return Err(Error::Missing("select two objects to unite"));
            }
            if first {
                let count = if union { 1 } else { event.selection.len() };
                let group = event.selection[..count]
                    .iter()
                    .map(|s| [&s.id])
                    .collect::<Vec<_>>();
                state["objects"] = serde_json::to_value(group)
                    .map_err(|_| Error::Invalid("mesh Boolean first set"))?;
                first = false;
                if !union {
                    return Ok(Some(self.prompt(state, first, keep)));
                }
            }
            let objects = state["objects"]
                .as_array()
                .ok_or(Error::Invalid("mesh Boolean first set"))?;
            let accepted: std::collections::BTreeSet<_> =
                objects.iter().filter_map(|v| v[0].as_str()).collect();
            if event
                .selection
                .iter()
                .all(|s| accepted.contains(s.id.as_str()))
            {
                return Err(Error::Missing("select the second Boolean set"));
            }
            event.settings.insert(
                "bb_mode".into(),
                Value::from(match self {
                    Self::Union => "union",
                    Self::Difference => "difference",
                    Self::Intersection => "intersection",
                }),
            );
            event.settings.insert("bb_keep_cutters".into(), keep.into());
            event.state = state;
            return Ok(None);
        }
        Ok(Some(self.prompt(state, first, keep)))
    }

    fn prompt(self, state: Value, first: bool, keep: bool) -> Reply {
        let text = match (self, first) {
            (Self::Union, _) => "Select objects to unite, Enter",
            (Self::Difference, true) => "Select objects to subtract from, Enter",
            (Self::Difference, false) => "Select objects to subtract with, Enter",
            (Self::Intersection, true) => "Select first set to intersect, Enter",
            (Self::Intersection, false) => "Select second set to intersect, Enter",
        };
        Reply::ask(format!("{}: {text}", self.label()), state)
            .want(Want::Objects)
            .with_options(vec![OptionToken::toggle("DeleteInput", !keep)])
    }
}

pub(super) struct Operands {
    first: Vec<om9_core::identity::ObjectId>,
    second: Vec<om9_core::identity::ObjectId>,
}

impl Operands {
    pub(super) fn read(event: &Event) -> Result<Self> {
        use om9_core::identity::ObjectId;
        let id = |s: &str| {
            uuid::Uuid::parse_str(s)
                .map(ObjectId)
                .map_err(|_| Error::Invalid("Boolean object ID"))
        };
        let entries = event.state["objects"]
            .as_array()
            .ok_or(Error::Missing("first Boolean set"))?;
        let first: Vec<_> = entries
            .iter()
            .map(|v| id(v[0].as_str().ok_or(Error::Invalid("Boolean ID"))?))
            .collect::<Result<_>>()?;
        let accepted: std::collections::BTreeSet<_> = first.iter().copied().collect();
        let second = event
            .selection
            .iter()
            .map(|s| id(&s.id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter(|v| !accepted.contains(v))
            .collect();
        Ok(Self { first, second })
    }

    pub(super) fn ids(&self) -> impl Iterator<Item = &om9_core::identity::ObjectId> {
        self.first.iter().chain(&self.second)
    }

    pub(super) fn compute(
        &self,
        document: &om9_core::model::Document,
        mut event: Event,
        limits: &om9_core::budget::Limits,
        cancel: &om9_core::transaction::Cancellation,
    ) -> Result<Reply> {
        let tolerance = document.effective_tolerance()?.absolute_mm;
        let k = document.page_unit.factor();
        let mut triangles = 0usize;
        let mut load = |id: &om9_core::identity::ObjectId| {
            if cancel.is_canceled() {
                return Err(Error::Canceled);
            }
            let e = document
                .entities
                .get(id)
                .ok_or(Error::Missing("Boolean source"))?;
            let mesh = if let Some((mesh, placement, _)) =
                super::addon_session::boolean_mesh_source(document, e, limits.max_depth)?
            {
                super::addon_session::boolean_source_mesh(
                    mesh,
                    &placement,
                    k,
                    tolerance,
                    Some(cancel),
                )?
            } else {
                surface_mesh(e, k, tolerance, limits, cancel)?
            };
            triangles = triangles
                .checked_add(mesh.triangles.len())
                .ok_or(Error::Overflow("mesh Boolean triangles"))?;
            if triangles > om9_core::mesh_csg::CsgOptions::default().max_triangles {
                return Err(Error::Budget("mesh Boolean triangles"));
            }
            Ok(mesh)
        };
        let first = self
            .first
            .iter()
            .map(|id| Ok((id.to_string(), load(id)?)))
            .collect::<Result<Vec<_>>>()?;
        event.selection = self
            .second
            .iter()
            .map(|id| {
                Ok(om9_addon_abi::session::Selected {
                    id: id.to_string(),
                    kind: "mesh".into(),
                    mesh: Some(load(id)?),
                    ..Default::default()
                })
            })
            .collect::<Result<_>>()?;
        super::boolean_builder_worker::calculate(&event, cancel, false, Some(first))
    }
}

#[cfg(feature = "occt-brep")]
fn surface_mesh(
    entity: &om9_core::model::Entity,
    k: f64,
    tolerance: f64,
    limits: &om9_core::budget::Limits,
    cancel: &om9_core::transaction::Cancellation,
) -> Result<om9_addon_abi::session::SelectedMesh> {
    use om9_core::geometry::{Geometry, Mesh, Transform};
    let max_triangles = om9_core::mesh_csg::CsgOptions::default().max_triangles;
    let bounded = om9_core::budget::Limits {
        max_points: limits.max_points.min(3 * max_triangles as u64),
        ..*limits
    };
    let limits = &bounded;
    let chord = tolerance / k;
    let surface = match &entity.geometry {
        Geometry::Brep(source) => om9_kernel::brep::read_brep(source.source(), chord, limits),
        Geometry::NativeBrep(source) => {
            om9_kernel::brep::from_3dm(source.payload(), chord, chord, limits)
        }
        Geometry::Surface(source) => {
            om9_kernel::measurement::surface_to_face(source, Transform::identity(), chord, limits)
        }
        _ => return Err(Error::Invalid("select a mesh, surface or polysurface")),
    }
    .map_err(|e| Error::Io(format!("mesh Boolean tessellation: {e:?}")))?;
    if cancel.is_canceled() {
        return Err(Error::Canceled);
    }
    let display = surface.display();
    if display.triangles.len() > om9_core::mesh_csg::CsgOptions::default().max_triangles {
        return Err(Error::Budget("Boolean surface triangles"));
    }
    let mesh = Mesh {
        vertices: display.points.clone(),
        faces: display.triangles.iter().map(|t| t[..3].to_vec()).collect(),
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let (welded, _) = om9_core::mesh_repair::repair_mesh(&mesh, chord * 0.01)?;
    super::addon_session::boolean_source_mesh(
        &welded,
        &entity.placement,
        k,
        tolerance,
        Some(cancel),
    )
}

#[cfg(not(feature = "occt-brep"))]
fn surface_mesh(
    _: &om9_core::model::Entity,
    _: f64,
    _: f64,
    _: &om9_core::budget::Limits,
    _: &om9_core::transaction::Cancellation,
) -> Result<om9_addon_abi::session::SelectedMesh> {
    Err(Error::Unsupported(
        "surface mesh Boolean requires the OCCT adapter",
    ))
}
