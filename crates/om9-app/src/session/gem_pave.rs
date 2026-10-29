// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pavé, azure, prong, sphere and springs sessions (`OM9-L2-04-12`).
//!
//! Each builder is one session: `open` validates the selection, `recompute` rebuilds the pure plan
//! from the typed parameters, and `apply` commits the whole plan in one transaction. Cancel only
//! drops the session, so no preview stone or marker ever leaks into the document.

use crate::session::Controller;
use om9_core::{
    Error, Result,
    budget::Limits,
    gem::{GemId, GemInstance, GemMesh, GemRecord, GemSpec, build_mesh, gem_record_in},
    gem_pave::{
        AzureParams, AzurePlan, PaveCorrection, PaveFeature, PaveLayout, PaveParams, PavePattern,
        PaveStart, PaveStyle, ProngParams, ProngPlan, SphereParams, SpringParams, marker_axes,
        pave_azures, pave_framework, pave_prongs, pave_sphere, pave_surface, refresh_pave_layout,
        spring_relax,
    },
    geometry::{Curve, Geometry, NurbsSurface, Point3, Transform},
    identity::ObjectId,
    model::{Entity, GEM_EXTENSION_SLOT, GEM_PAVE_EXTENSION_SLOT},
    transaction::Mutation,
};

/// The command family owned by this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaveTool {
    Auto,
    Manual,
    Azure,
    Prong,
    Sphere,
    Springs,
}

impl PaveTool {
    pub const ALL: [Self; 6] = [
        Self::Auto,
        Self::Manual,
        Self::Azure,
        Self::Prong,
        Self::Sphere,
        Self::Springs,
    ];
    pub fn command(self) -> &'static str {
        match self {
            Self::Auto => "BuilderPaveDialog",
            Self::Manual => "BuilderPaveBuilder",
            Self::Azure => "BuilderPaveAzureBuilder",
            Self::Prong => "BuilderPaveProngBuilder",
            Self::Sphere => "BuilderPaveSphere",
            Self::Springs => "BuilderGemSprings",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Auto => "Auto Pavé Builder",
            Self::Manual => "Pavé Builder",
            Self::Azure => "Pavé Azure Builder",
            Self::Prong => "Pavé Prong Builder",
            Self::Sphere => "Pavé Sphere",
            Self::Springs => "Gem Springs",
        }
    }
    pub fn uses_surface(self) -> bool {
        matches!(
            self,
            Self::Auto | Self::Manual | Self::Azure | Self::Springs
        )
    }
    pub fn uses_gems(self) -> bool {
        matches!(self, Self::Azure | Self::Prong | Self::Springs)
    }
    pub fn handles(feature: &str) -> bool {
        Self::ALL.iter().any(|tool| tool.command() == feature)
    }
    pub fn command_name(text: &str) -> Option<&'static str> {
        match text.trim().to_ascii_lowercase().as_str() {
            "gvpavedialog" | "autopavebuilder" | "paveauto" => Some(Self::Auto.command()),
            "gvpavebuilder" | "pavebuilder" | "pavemanual" => Some(Self::Manual.command()),
            "gvpaveazurebuilder" | "paveazurebuilder" | "azure" => Some(Self::Azure.command()),
            "gvpaveprongbuilder" | "paveprongbuilder" | "paveprongs" => Some(Self::Prong.command()),
            "gvpavesphere" | "pavesphere" => Some(Self::Sphere.command()),
            "gvgemsprings" | "gemsprings" => Some(Self::Springs.command()),
            _ => None,
        }
    }
}

/// The typed parameters of one open builder. Stone stays inline: the session is replaced
/// as a whole on every edit, and boxing would add an allocation to the preview path.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum PaveKind {
    Stone(PaveParams),
    Azure(AzureParams),
    Prong(ProngParams),
    Sphere(SphereParams),
    Spring(SpringParams),
}

impl PaveKind {
    pub fn stone_pattern(&self) -> Option<PavePattern> {
        match self {
            Self::Stone(params) => Some(params.pattern),
            _ => None,
        }
    }
}

/// One live pavé/azure/prong builder.
#[derive(Debug, Clone)]
pub struct GemPaveSession {
    pub tool: PaveTool,
    pub surface: Option<ObjectId>,
    pub surface_geometry: Option<NurbsSurface>,
    pub gems: Vec<ObjectId>,
    pub gem_instances: Vec<GemInstance>,
    pub kind: PaveKind,
    pub layout: Option<PaveLayout>,
    pub azures: Option<AzurePlan>,
    pub prongs: Option<ProngPlan>,
    pub sphere: Option<Vec<GemInstance>>,
    pub springs: Option<Vec<GemInstance>>,
    pub edit: Option<PaveFeature>,
    /// Document curve that supplied the attractor, when selected.
    pub attractor_source: Option<ObjectId>,
    /// Document curves that supplied the closed region edges, when selected.
    pub edge_sources: Vec<ObjectId>,
    /// Stations whose stored UV left the surface domain after a refresh.
    pub conflicts: Vec<GemId>,
    /// Manual Pavé exact quantity; `None` fills the region.
    pub quantity: Option<usize>,
    /// Name shown in the style browser; edits are local to the window.
    pub style_name: String,
    pub preview: om9_core::gem_pave::PavePreview,
    pub brep: bool,
}

fn selected_surface(
    controller: &Controller,
    selection: &[ObjectId],
) -> Result<(ObjectId, NurbsSurface)> {
    let limits = *controller.coordinator.limits();
    for id in selection {
        let entity = controller
            .document()
            .entities
            .get(id)
            .ok_or(Error::Stale("pave surface selection"))?;
        if let Geometry::Surface(surface) = &entity.geometry {
            let baked = if entity.placement == Transform::identity() {
                surface.clone()
            } else {
                match om9_core::top11::bake_geometry(
                    &Geometry::Surface(surface.clone()),
                    entity.placement,
                    &limits,
                )? {
                    Geometry::Surface(surface) => surface,
                    _ => return Err(Error::Invalid("pave surface")),
                }
            };
            return Ok((*id, baked));
        }
    }
    Err(Error::Missing("pave surface"))
}

fn selected_gems(controller: &Controller, selection: &[ObjectId]) -> Vec<ObjectId> {
    selection
        .iter()
        .copied()
        .filter(|id| gem_record_in(controller.document(), *id).is_some())
        .collect()
}

fn selected_curves(
    controller: &Controller,
    selection: &[ObjectId],
) -> Result<Vec<(ObjectId, Curve)>> {
    let limits = *controller.coordinator.limits();
    let mut curves = Vec::new();
    for id in selection {
        let entity = controller
            .document()
            .entities
            .get(id)
            .ok_or(Error::Stale("pave curve selection"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            continue;
        };
        let baked = if entity.placement == Transform::identity() {
            curve.clone()
        } else {
            match om9_core::top11::bake_geometry(
                &Geometry::Curve(curve.clone()),
                entity.placement,
                &limits,
            )? {
                Geometry::Curve(curve) => curve,
                _ => return Err(Error::Invalid("pave curve")),
            }
        };
        curves.push((*id, baked));
    }
    Ok(curves)
}

/// Split selected curves into one open attractor and closed region edges.
fn document_curve(controller: &Controller, id: ObjectId, limits: &Limits) -> Result<Option<Curve>> {
    let Some(entity) = controller.document().entities.get(&id) else {
        return Ok(None);
    };
    let Geometry::Curve(curve) = &entity.geometry else {
        return Ok(None);
    };
    if entity.placement == Transform::identity() {
        return Ok(Some(curve.clone()));
    }
    match om9_core::top11::bake_geometry(&Geometry::Curve(curve.clone()), entity.placement, limits)?
    {
        Geometry::Curve(curve) => Ok(Some(curve)),
        _ => Err(Error::Invalid("pave curve")),
    }
}

fn guide_curves(
    curves: Vec<(ObjectId, Curve)>,
    limits: &Limits,
) -> Result<(Option<(ObjectId, Curve)>, Vec<(ObjectId, Curve)>)> {
    let mut attractor = None;
    let mut edges = Vec::new();
    for (id, curve) in curves {
        if om9_core::curve_ops::is_closed(&curve, 1.0e-6, limits)? {
            edges.push((id, curve));
        } else if attractor.is_none() {
            attractor = Some((id, curve));
        } else {
            return Err(Error::Invalid("pave has more than one attractor curve"));
        }
    }
    Ok((attractor, edges))
}

fn instance_for(controller: &Controller, id: ObjectId) -> Result<GemInstance> {
    let record =
        gem_record_in(controller.document(), id).ok_or(Error::Invalid("pave needs a gem"))?;
    let entity = controller
        .document()
        .entities
        .get(&id)
        .ok_or(Error::Stale("pave gem"))?;
    Ok(GemInstance {
        spec: record.spec,
        placement: entity.placement,
        label: record.asset,
    })
}

fn parse_pair(text: &str) -> Option<(&str, &str)> {
    let (key, value) = text.split_once('=')?;
    Some((key.trim(), value.trim()))
}

impl GemPaveSession {
    /// Open one builder for the current selection.
    pub fn open(controller: &Controller, tool: PaveTool) -> Result<Self> {
        let selection = controller.selected_ordered();
        let mut surface = None;
        let mut surface_geometry = None;
        let mut gems = Vec::new();
        let mut instances = Vec::new();
        if tool.uses_surface() {
            let (id, geometry) = selected_surface(controller, &selection)?;
            surface = Some(id);
            surface_geometry = Some(geometry);
        }
        if tool.uses_gems() {
            gems = selected_gems(controller, &selection);
            if gems.is_empty() {
                return Err(Error::Missing("pave gems"));
            }
            for id in &gems {
                instances.push(instance_for(controller, *id)?);
            }
        }
        if tool == PaveTool::Prong && !gems.is_empty() && surface.is_none() {
            // Prongs can be derived from the gems alone; the surface is optional.
        }
        let limits = *controller.coordinator.limits();
        let guides = if matches!(tool, PaveTool::Auto | PaveTool::Manual) {
            guide_curves(selected_curves(controller, &selection)?, &limits)?
        } else {
            (None, Vec::new())
        };
        let mut kind = match tool {
            PaveTool::Auto | PaveTool::Manual => {
                let spec = GemSpec::round(1.5, 1.0)?;
                let mut params = PaveParams::new(spec)?;
                params.relax_iterations = if tool == PaveTool::Auto { 4 } else { 0 };
                PaveKind::Stone(params)
            }
            PaveTool::Azure => PaveKind::Azure(AzureParams::new()?),
            PaveTool::Prong => PaveKind::Prong(ProngParams::new()?),
            PaveTool::Sphere => PaveKind::Sphere(SphereParams::new(GemSpec::round(1.5, 1.0)?)?),
            PaveTool::Springs => PaveKind::Spring(SpringParams::new()?),
        };
        let edit = if matches!(tool, PaveTool::Auto | PaveTool::Manual) {
            surface
                .and_then(|id| controller.document().entities.get(&id))
                .and_then(|entity| entity.extensions.get(GEM_PAVE_EXTENSION_SLOT))
                .and_then(|extension| PaveFeature::from_extension(extension).ok())
        } else {
            None
        };
        if let (PaveKind::Stone(params), Some(feature)) = (&mut kind, edit.as_ref()) {
            params.clone_from(&feature.params);
        }
        if let PaveKind::Stone(params) = &mut kind
            && (guides.0.is_some() || !guides.1.is_empty())
        {
            params.attractor = guides.0.as_ref().map(|(_, curve)| curve.clone());
            params.edge_curves = guides.1.iter().map(|(_, curve)| curve.clone()).collect();
            params.start = if params.attractor.is_some() {
                PaveStart::Curve
            } else {
                PaveStart::Edges
            };
        }
        let attractor_source = guides
            .0
            .as_ref()
            .map(|(id, _)| *id)
            .or_else(|| edit.as_ref().and_then(|feature| feature.attractor_source));
        let edge_sources = if guides.1.is_empty() {
            edit.as_ref()
                .map(|feature| feature.edge_sources.clone())
                .unwrap_or_default()
        } else {
            guides.1.iter().map(|(id, _)| *id).collect()
        };
        let quantity = edit.as_ref().and_then(|feature| feature.quantity);
        let mut session = Self {
            tool,
            surface,
            surface_geometry,
            gems,
            gem_instances: instances,
            kind,
            layout: None,
            azures: None,
            prongs: None,
            sphere: None,
            springs: None,
            edit,
            attractor_source,
            edge_sources,
            conflicts: Vec::new(),
            quantity,
            style_name: "Classic Hex".to_owned(),
            preview: om9_core::gem_pave::PavePreview::Stones,
            brep: cfg!(feature = "occt-brep"),
        };
        session.recompute(controller.coordinator.limits())?;
        Ok(session)
    }

    pub fn recompute(&mut self, limits: &Limits) -> Result<()> {
        match &self.kind {
            PaveKind::Stone(params) => {
                let surface = self
                    .surface_geometry
                    .as_ref()
                    .ok_or(Error::Missing("pave surface"))?;
                let mut layout = pave_surface(surface, params, limits)?;
                if let Some(quantity) = self.quantity {
                    layout.stations.truncate(quantity.max(1));
                }
                self.layout = Some(layout);
                self.azures = None;
                self.prongs = None;
                self.sphere = None;
                self.springs = None;
            }
            PaveKind::Azure(params) => {
                self.azures = Some(pave_azures(&self.gem_instances, params)?);
                self.layout = None;
            }
            PaveKind::Prong(params) => {
                self.prongs = Some(pave_prongs(&self.gem_instances, params)?);
                self.layout = None;
            }
            PaveKind::Sphere(params) => {
                self.sphere = Some(pave_sphere(params)?);
                self.layout = None;
            }
            PaveKind::Spring(params) => {
                let surface = self
                    .surface_geometry
                    .as_ref()
                    .ok_or(Error::Missing("spring surface"))?;
                self.springs = Some(spring_relax(surface, &self.gem_instances, params, limits)?);
                self.layout = None;
            }
        }
        Ok(())
    }

    pub fn refresh(&mut self, limits: &Limits) -> Result<()> {
        let surface = self
            .surface_geometry
            .as_ref()
            .ok_or(Error::Missing("pave surface"))?;
        let feature = self.edit.as_ref().ok_or(Error::Missing("pave feature"))?;
        let refresh = refresh_pave_layout(surface, feature, limits)?;
        self.layout = Some(PaveLayout {
            params: feature.params.clone(),
            stations: refresh.stations,
        });
        self.conflicts = refresh.conflicts;
        Ok(())
    }

    pub fn preview_count(&self) -> usize {
        if let Some(layout) = &self.layout {
            return layout.stations.len();
        }
        if let Some(plan) = &self.azures {
            return plan.azures.len();
        }
        if let Some(plan) = &self.prongs {
            return plan.prongs.len();
        }
        if let Some(gems) = &self.sphere {
            return gems.len();
        }
        if let Some(gems) = &self.springs {
            return gems.len();
        }
        0
    }

    pub fn summary(&self) -> String {
        match &self.kind {
            PaveKind::Stone(params) => format!(
                "{} stones · {} pattern · {:.2} mm spacing",
                self.preview_count(),
                params.pattern.label(),
                params.spacing_mm
            ),
            PaveKind::Azure(params) => format!(
                "{} azures · {:.2} mm deep · {}",
                self.preview_count(),
                params.azure_depth_mm,
                if params.hexagonal { "hex" } else { "round" }
            ),
            PaveKind::Prong(params) => format!(
                "{} prongs · {:.2} mm high · fillet {:.2}",
                self.preview_count(),
                params.height_mm,
                params.fillet
            ),
            PaveKind::Sphere(params) => format!(
                "{} stones · R {:.2} mm",
                self.preview_count(),
                params.radius_mm
            ),
            PaveKind::Spring(params) => format!(
                "{} stones · {:.2} mm spacing",
                self.preview_count(),
                params.spacing_mm
            ),
        }
    }

    pub fn prompt(&self) -> String {
        format!("{}: {} · Apply/Cancel", self.tool.title(), self.summary())
    }

    pub fn world_preview(&self) -> Vec<(GemMesh, Transform)> {
        if let Some(layout) = &self.layout {
            return layout
                .stations
                .iter()
                .filter_map(|station| {
                    build_mesh(&station.spec)
                        .ok()
                        .map(|mesh| (mesh, station.transform))
                })
                .collect();
        }
        let instances: Vec<&GemInstance> = if let Some(gems) = &self.sphere {
            gems.iter().collect()
        } else if let Some(gems) = &self.springs {
            gems.iter().collect()
        } else {
            Vec::new()
        };
        instances
            .into_iter()
            .filter_map(|instance| {
                build_mesh(&instance.spec)
                    .ok()
                    .map(|mesh| (mesh, instance.placement))
            })
            .collect()
    }
}

impl Controller {
    pub fn dispatch_gem_pave(&mut self, feature: &str) -> Result<()> {
        let tool = PaveTool::ALL
            .into_iter()
            .find(|tool| tool.command() == feature)
            .ok_or(Error::Invalid("pave command"))?;
        if self.is_running() {
            return Err(Error::Busy);
        }
        let session = GemPaveSession::open(self, tool)?;
        self.gem_pave = Some(session);
        self.command_run = self.command_run.wrapping_add(1);
        Ok(())
    }

    pub fn gem_pave_session(&self) -> Option<&GemPaveSession> {
        self.gem_pave.as_ref()
    }

    pub fn gem_pave_session_mut(&mut self) -> Option<&mut GemPaveSession> {
        self.gem_pave.as_mut()
    }

    pub fn gem_pave_cancel(&mut self) -> bool {
        self.gem_pave.take().is_some()
    }

    /// Built-in plus user styles shown in the Auto/Manual window.
    pub fn gem_pave_style_names(&self) -> Vec<String> {
        self.pave_style_library.names()
    }

    pub fn gem_pave_style_error(&self) -> Option<&str> {
        self.pave_style_error.as_deref()
    }

    /// Viewport handles for the sphere builder: radius at the equator and stone count at
    /// the north pole. They feed the same fields as the panel, so drag and typed edit agree.
    pub fn gem_pave_handles(&self) -> Vec<om9_core::builder::BuilderHandle> {
        let Some(session) = &self.gem_pave else {
            return Vec::new();
        };
        let PaveKind::Sphere(params) = &session.kind else {
            return Vec::new();
        };
        let radius = params.radius_mm;
        let mut handles = vec![om9_core::builder::BuilderHandle {
            name: "pave_sphere_radius",
            field: "radius",
            label: "Radius",
            position: [radius, 0.0, 0.0],
            axis: [1.0, 0.0, 0.0],
            value: radius,
            min: 1.0,
            max: 100.0,
            scale: 1.0,
        }];
        if let Some(count) = params.count {
            handles.push(om9_core::builder::BuilderHandle {
                name: "pave_sphere_count",
                field: "count",
                label: "Count",
                position: [0.0, 0.0, radius],
                axis: [0.0, 0.0, 1.0],
                value: count as f64,
                min: 4.0,
                max: 1024.0,
                scale: 0.5,
            });
        }
        handles
    }

    pub fn gem_pave_adjust_handle(&mut self, field: &str, delta: f64) -> Result<()> {
        let mut session = self.gem_pave.take().ok_or(Error::Missing("pave session"))?;
        let PaveKind::Sphere(params) = &mut session.kind else {
            self.gem_pave = Some(session);
            return Err(Error::Invalid("pave sphere handle"));
        };
        match field {
            "radius" => params.radius_mm = (params.radius_mm + delta).clamp(1.0, 100.0),
            "count" => {
                let Some(current) = params.count else {
                    self.gem_pave = Some(session);
                    return Err(Error::Missing("pave sphere count"));
                };
                let next = (current as f64 + delta).round().clamp(4.0, 1024.0) as usize;
                params.count = Some(next);
            }
            _ => {
                self.gem_pave = Some(session);
                return Err(Error::Invalid("pave sphere handle field"));
            }
        }
        let limits = *self.coordinator.limits();
        let result = session.recompute(&limits);
        self.gem_pave = Some(session);
        result
    }

    /// Replace the open stone session with a stored style. The style chooses pattern,
    /// spacing, corrections and the manual count; Apply still owns the document.
    pub fn gem_pave_apply_style(&mut self, name: &str) -> Result<()> {
        self.pave_style_error = None;
        let style = self
            .pave_style_library
            .get(name)
            .cloned()
            .ok_or(Error::Missing("pave style"))?;
        let mut session = self.gem_pave.take().ok_or(Error::Missing("pave session"))?;
        if !matches!(session.kind, PaveKind::Stone(_)) {
            self.gem_pave = Some(session);
            return Err(Error::Invalid("pave style needs a stone builder"));
        }
        session.kind = PaveKind::Stone(style.params);
        // A style saved from Auto carries no count; applying it to Manual keeps the
        // builder's explicit count instead of silently reverting to a region fill.
        if session.tool == PaveTool::Manual
            && let Some(quantity) = style.quantity
        {
            session.quantity = Some(quantity);
        }
        session.style_name = style.name;
        let limits = *self.coordinator.limits();
        let result = session.recompute(&limits);
        self.gem_pave = Some(session);
        result
    }

    /// Snapshot the current stone parameters under `name`. The file write is best effort:
    /// the in-memory library always keeps the style and a disk error is reported separately.
    pub fn gem_pave_save_style(&mut self, name: &str) -> Result<()> {
        let session = self
            .gem_pave
            .as_ref()
            .ok_or(Error::Missing("pave session"))?;
        let PaveKind::Stone(params) = &session.kind else {
            return Err(Error::Invalid("pave style needs a stone builder"));
        };
        let style = PaveStyle::new(name, params.clone(), session.quantity)?;
        self.pave_style_library.add_user(style)?;
        if let Some(path) = &self.pave_style_path
            && let Err(error) = self.pave_style_library.save_user(path)
        {
            self.pave_style_error = Some(error.to_string());
        }
        if let Some(session) = self.gem_pave.as_mut() {
            session.style_name = name.trim().to_owned();
        }
        Ok(())
    }

    pub fn gem_pave_set(&mut self, mut session: GemPaveSession) -> Result<()> {
        session.recompute(self.coordinator.limits())?;
        self.gem_pave = Some(session);
        Ok(())
    }

    /// Re-evaluate a persisted feature after its host surface changed (`OM9-L3-04-12-008`).
    pub fn gem_pave_refresh(&mut self) -> Result<()> {
        let mut session = self.gem_pave.take().ok_or(Error::Missing("pave session"))?;
        let limits = *self.coordinator.limits();
        let prepared = (|| -> Result<()> {
            let Some(feature) = session.edit.as_mut() else {
                return Ok(());
            };
            if let Some(source) = feature.attractor_source {
                feature.params.attractor = Some(
                    document_curve(self, source, &limits)?
                        .ok_or(Error::Stale("pave attractor source"))?,
                );
            }
            if !feature.edge_sources.is_empty() {
                let mut edges = Vec::with_capacity(feature.edge_sources.len());
                for source in &feature.edge_sources {
                    edges.push(
                        document_curve(self, *source, &limits)?
                            .ok_or(Error::Stale("pave edge source"))?,
                    );
                }
                feature.params.edge_curves = edges;
            }
            Ok(())
        })();
        let result = prepared.and_then(|()| session.refresh(&limits));
        self.gem_pave = Some(session);
        result
    }

    pub fn gem_pave_apply(&mut self) -> Result<()> {
        let session = self
            .gem_pave
            .clone()
            .ok_or(Error::Missing("pave session"))?;
        let result = match session.tool {
            PaveTool::Auto | PaveTool::Manual => self.apply_pave_stones(&session),
            PaveTool::Azure => self.apply_pave_azures(&session),
            PaveTool::Prong => self.apply_pave_prongs(&session),
            PaveTool::Sphere => self.apply_pave_sphere(&session),
            PaveTool::Springs => self.apply_gem_springs(&session),
        };
        if result.is_ok() {
            self.gem_pave = None;
        }
        result
    }

    fn apply_pave_stones(&mut self, session: &GemPaveSession) -> Result<()> {
        if !session.conflicts.is_empty() {
            return Err(Error::Invalid(
                "pave refresh has stations outside the surface",
            ));
        }
        let layout = session
            .layout
            .as_ref()
            .ok_or(Error::Missing("pave layout"))?;
        let host = session.surface.ok_or(Error::Missing("pave surface"))?;
        let layer = self.gem_layer()?;
        self.ensure_layer_writable(layer)?;
        let host_entity = self
            .document()
            .entities
            .get(&host)
            .ok_or(Error::Stale("pave surface"))?
            .clone();
        let existing = session
            .edit
            .as_ref()
            .map(|feature| feature.gems.clone())
            .unwrap_or_default();
        let document = self.document;
        let mut mutations = Vec::new();
        let mut gems = Vec::with_capacity(layout.stations.len());
        let mut uvs = Vec::with_capacity(layout.stations.len());
        let mut used = std::collections::BTreeSet::new();
        for (index, station) in layout.stations.iter().enumerate() {
            let existing_id = station
                .gem
                .map(GemId::object)
                .filter(|id| existing.iter().any(|gem| gem.object() == *id))
                .or_else(|| existing.get(index).map(|id| GemId::object(*id)));
            let geometry = self.pave_child_geometry(&station.spec, session.brep)?;
            if let Some(id) = existing_id {
                used.insert(id);
                let source = self.document().stamp(id)?;
                let mut record = gem_record_in(self.document(), id)
                    .unwrap_or_else(|| GemRecord::new(station.spec.clone()));
                record.spec = station.spec.clone();
                record.asset = station.spec.asset_key();
                let extensions = {
                    let entity = self
                        .document()
                        .entities
                        .get(&id)
                        .ok_or(Error::Stale("pave child"))?;
                    let mut extensions = entity.extensions.clone();
                    extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
                    extensions
                };
                mutations.push(Mutation::Geometry { source, geometry });
                mutations.push(Mutation::Placement {
                    source,
                    placement: station.transform,
                });
                mutations.push(Mutation::Extensions { source, extensions });
                gems.push(GemId(id));
            } else {
                let mut record = GemRecord::new(station.spec.clone());
                let mut entity = Entity::new(
                    format!(
                        "{} {}",
                        station.spec.shape.label(),
                        station.spec.cut.label()
                    ),
                    geometry,
                    layer,
                );
                record.gem_id = GemId(entity.id);
                entity.placement = station.transform;
                entity
                    .extensions
                    .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
                gems.push(record.gem_id);
                mutations.push(Mutation::Insert { document, entity });
            }
            uvs.push(station.uv);
        }
        for id in existing.iter().filter(|id| !used.contains(&id.object())) {
            let source = self.document().stamp(id.object())?;
            mutations.push(Mutation::Delete { source });
        }
        let mut feature = PaveFeature::new(host, layout.params.clone());
        feature.quantity = session.quantity;
        feature.attractor_source = session.attractor_source;
        feature.edge_sources = session.edge_sources.clone();
        feature.gems = gems;
        feature.uvs = uvs;
        let source = self.document().stamp(host)?;
        let mut extensions = host_entity.extensions.clone();
        extensions.insert(GEM_PAVE_EXTENSION_SLOT.to_owned(), feature.to_extension()?);
        mutations.push(Mutation::Extensions { source, extensions });
        self.commit_gem(session.tool.title(), mutations)
    }

    fn apply_pave_azures(&mut self, session: &GemPaveSession) -> Result<()> {
        let plan = session
            .azures
            .as_ref()
            .ok_or(Error::Missing("azure plan"))?;
        let layer = self.document().current_layer;
        self.ensure_layer_writable(layer)?;
        let document = self.document;
        let mut mutations = Vec::new();
        for (index, azure) in plan.azures.iter().enumerate() {
            for (role, points) in [
                ("Top", &azure.top),
                ("Through", &azure.through),
                ("Bottom", &azure.bottom),
                ("Seat", &azure.seat),
            ] {
                let curve = om9_core::gem_pave::closed_polygon(points)?;
                let entity = Entity::new(
                    format!("Azure {role} {:02}", index + 1),
                    Geometry::Curve(curve),
                    layer,
                );
                mutations.push(Mutation::Insert { document, entity });
            }
        }
        self.commit_gem("Pavé Azure", mutations)
    }

    fn apply_pave_prongs(&mut self, session: &GemPaveSession) -> Result<()> {
        let plan = session
            .prongs
            .as_ref()
            .ok_or(Error::Missing("prong plan"))?;
        let layer = self.document().current_layer;
        self.ensure_layer_writable(layer)?;
        let document = self.document;
        let mut mutations = Vec::new();
        for (index, prong) in plan.prongs.iter().enumerate() {
            let z = prong.normal;
            let (x, y) = marker_axes(z)?;
            let curve = Geometry::Curve(om9_core::geometry::Curve::Circle {
                center: prong.position,
                x,
                y,
                radius: prong.top_radius_mm.max(0.05),
                domain: [0.0, std::f64::consts::TAU],
                angle_domain: None,
            });
            let entity = Entity::new(format!("Pavé Prong {:03}", index + 1), curve, layer);
            mutations.push(Mutation::Insert { document, entity });
        }
        self.commit_gem("Pavé Prongs", mutations)
    }

    fn apply_pave_sphere(&mut self, session: &GemPaveSession) -> Result<()> {
        let gems = session
            .sphere
            .as_ref()
            .ok_or(Error::Missing("pave sphere"))?;
        let layer = self.gem_layer()?;
        self.ensure_layer_writable(layer)?;
        let document = self.document;
        let mut mutations = Vec::new();
        for instance in gems {
            let geometry = self.pave_child_geometry(&instance.spec, session.brep)?;
            let mut record = GemRecord::new(instance.spec.clone());
            let mut entity = Entity::new(
                format!(
                    "{} {}",
                    instance.spec.shape.label(),
                    instance.spec.cut.label()
                ),
                geometry,
                layer,
            );
            record.gem_id = GemId(entity.id);
            entity.placement = instance.placement;
            entity
                .extensions
                .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
            mutations.push(Mutation::Insert { document, entity });
        }
        self.commit_gem("Pavé Sphere", mutations)
    }

    fn apply_gem_springs(&mut self, session: &GemPaveSession) -> Result<()> {
        let gems = session
            .springs
            .as_ref()
            .ok_or(Error::Missing("spring layout"))?;
        if gems.len() != session.gems.len() {
            return Err(Error::Invalid("spring gem count"));
        }
        let mut mutations = Vec::new();
        for (id, instance) in session.gems.iter().zip(gems) {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Stale("spring gem"))?;
            self.ensure_layer_writable(entity.attributes.layer)?;
            let source = self.document().stamp(*id)?;
            let geometry = self.pave_child_geometry(&instance.spec, session.brep)?;
            let mut record = gem_record_in(self.document(), *id)
                .unwrap_or_else(|| GemRecord::new(instance.spec.clone()));
            record.spec = instance.spec.clone();
            record.asset = instance.spec.asset_key();
            let mut extensions = entity.extensions.clone();
            extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
            mutations.push(Mutation::Geometry { source, geometry });
            mutations.push(Mutation::Placement {
                source,
                placement: instance.placement,
            });
            mutations.push(Mutation::Extensions { source, extensions });
        }
        self.commit_gem("Gem Springs", mutations)
    }

    fn pave_child_geometry(&self, spec: &GemSpec, brep: bool) -> Result<Geometry> {
        let mesh = build_mesh(spec)?;
        if !mesh.closed {
            return Err(Error::Invalid("pave gem mesh open"));
        }
        self.gem_geometry(mesh, brep)
    }

    pub fn gem_pave_prompt(&self) -> Option<String> {
        self.gem_pave.as_ref().map(GemPaveSession::prompt)
    }

    /// The open builder owns the command line: `key=value` options, Apply/Enter or Cancel/Esc.
    pub fn gem_pave_text(&mut self, text: &str) -> Option<Result<()>> {
        let mut session = self.gem_pave.as_ref()?.clone();
        let mut apply = false;
        let mut cancel = false;
        let mut options = false;
        let outcome: Result<()> = (|| {
            let tokens: Vec<&str> = text.split_whitespace().collect();
            if tokens.is_empty() {
                apply = true;
            }
            for token in tokens {
                let lower = token.trim().to_ascii_lowercase();
                if matches!(lower.as_str(), "apply" | "enter" | "commit") {
                    apply = true;
                    continue;
                }
                if matches!(lower.as_str(), "cancel" | "esc" | "abort") {
                    cancel = true;
                    continue;
                }
                if lower == "reset" {
                    session = reset_session(&session)?;
                    options = true;
                    continue;
                }
                let Some((key, value)) = parse_pair(&lower) else {
                    return Err(Error::Invalid("pave option"));
                };
                options = true;
                let number = || -> Result<f64> {
                    value
                        .parse::<f64>()
                        .map_err(|_| Error::Invalid("pave number"))
                };
                let flag = value == "on" || value == "yes" || value == "true";
                match &mut session.kind {
                    PaveKind::Stone(params) => match key {
                        "pattern" => {
                            params.pattern = match value {
                                "hex" => PavePattern::Hex,
                                "square" => PavePattern::Square,
                                "tri" => PavePattern::Tri,
                                "hexpent" | "hex_pent" => PavePattern::HexPent,
                                "alldirections" | "all" => PavePattern::AllDirections,
                                "hexr" | "hexreverse" => PavePattern::HexReverse,
                                _ => return Err(Error::Invalid("pave pattern")),
                            }
                        }
                        "start" => {
                            params.start = match value {
                                "base" | "basedirection" => PaveStart::BaseDirection,
                                "curve" => PaveStart::Curve,
                                "edges" => PaveStart::Edges,
                                "seeds" | "seedgems" => PaveStart::SeedGems,
                                _ => return Err(Error::Invalid("pave start")),
                            }
                        }
                        "spacing" => params.spacing_mm = number()?,
                        "minspacing" => params.min_spacing_mm = number()?,
                        "base" => params.base_size_mm = number()?,
                        "min" => params.min_size_mm = number()?,
                        "max" => params.max_size_mm = number()?,
                        "margin" => params.margin_mm = number()?,
                        "border" => params.border_mm = number()?,
                        "angle" => params.angle_deg = number()?,
                        "flip" => params.flip = flag,
                        "taper" => params.taper = number()?,
                        "equalize" => params.equalize = number()?,
                        "perturb" => params.perturb = number()?,
                        "strength" => params.relax_strength = number()?,
                        "relax" => params.relax_iterations = number()?.max(0.0) as u32,
                        "pre" => params.pre = correction(value)?,
                        "post" => params.post = correction(value)?,
                        "count" => {
                            let count = number()?;
                            session.quantity = (count > 0.0).then_some(count as usize);
                        }
                        "seed" => {
                            let (u, v) =
                                value.split_once(',').ok_or(Error::Invalid("pave seed"))?;
                            params.seeds.push([
                                u.parse::<f64>().map_err(|_| Error::Invalid("pave seed"))?,
                                v.parse::<f64>().map_err(|_| Error::Invalid("pave seed"))?,
                            ]);
                        }
                        _ => return Err(Error::Invalid("pave option")),
                    },
                    PaveKind::Azure(params) => match key {
                        "hex" => params.hexagonal = flag,
                        "angle" => params.angle_deg = number()?,
                        "depth" => params.azure_depth_mm = number()?,
                        "size" => params.azure_size_mm = number()?,
                        "through" => params.through_hole_mm = number()?,
                        "seat" => params.seat_angle_deg = number()?,
                        "minspacing" => params.min_spacing_mm = number()?,
                        "scale" => params.scale = number()?,
                        _ => return Err(Error::Invalid("azure option")),
                    },
                    PaveKind::Prong(params) => match key {
                        "fillet" => params.fillet = number()?,
                        "height" => params.height_mm = number()?,
                        "drop" => params.drop_mm = number()?,
                        "taper" => params.taper = number()?,
                        "scale" => params.scale = number()?,
                        "shared" => params.shared_size_mm = number()?,
                        "unshared" => params.unshared_size_mm = number()?,
                        "centermin" => params.center_min_mm = number()?,
                        "centermax" => params.center_max_mm = number()?,
                        "touch" => params.touch_fraction = number()?,
                        _ => return Err(Error::Invalid("prong option")),
                    },
                    PaveKind::Sphere(params) => match key {
                        "radius" => params.radius_mm = number()?,
                        "count" => {
                            let count = number()?;
                            params.count = (count > 0.0).then_some(count as usize);
                        }
                        "x" => params.x = number()?.max(0.0) as u32,
                        "t" => params.t = number()?.max(0.0) as u32,
                        "removal" => {
                            params.removal = match value {
                                "none" => om9_core::gem_pave::RemovalPattern::None,
                                "a" | "patterna" => om9_core::gem_pave::RemovalPattern::PatternA,
                                "b" | "patternb" => om9_core::gem_pave::RemovalPattern::PatternB,
                                _ => return Err(Error::Invalid("pave removal")),
                            }
                        }
                        "min" => params.min_size_mm = number()?,
                        "strength" => params.relax_strength = number()?,
                        _ => return Err(Error::Invalid("sphere option")),
                    },
                    PaveKind::Spring(params) => match key {
                        "spacing" => params.spacing_mm = number()?,
                        "minspacing" => params.min_spacing_mm = number()?,
                        "iterations" => params.iterations = number()?.max(0.0) as u32,
                        "strength" => params.strength = number()?,
                        "resize" => params.resize = number()?,
                        _ => return Err(Error::Invalid("spring option")),
                    },
                }
            }
            if cancel {
                return Ok(());
            }
            if options {
                self.gem_pave_set(session)?;
            } else {
                self.gem_pave = Some(session);
            }
            if apply {
                self.gem_pave_apply()?;
            }
            Ok(())
        })();
        if outcome.is_ok() && cancel {
            self.gem_pave_cancel();
        }
        Some(outcome)
    }

    pub fn gem_pave_world_preview(&self) -> Vec<(GemMesh, Transform)> {
        self.gem_pave
            .as_ref()
            .map(GemPaveSession::world_preview)
            .unwrap_or_default()
    }

    pub fn gem_pave_world_framework(&self) -> Vec<[Point3; 2]> {
        self.gem_pave
            .as_ref()
            .and_then(|session| session.layout.as_ref())
            .map(pave_framework)
            .unwrap_or_default()
    }

    pub fn gem_pave_world_curves(&self) -> Vec<Vec<Point3>> {
        let Some(session) = &self.gem_pave else {
            return Vec::new();
        };
        if let Some(plan) = &session.azures {
            let mut out = Vec::new();
            for azure in &plan.azures {
                out.extend([
                    azure.top.clone(),
                    azure.through.clone(),
                    azure.bottom.clone(),
                    azure.seat.clone(),
                ]);
            }
            return out;
        }
        Vec::new()
    }
}

fn correction(value: &str) -> Result<PaveCorrection> {
    match value {
        "normal" => Ok(PaveCorrection::Normal),
        "basic" => Ok(PaveCorrection::Basic),
        "grow" => Ok(PaveCorrection::Grow),
        "shrink" => Ok(PaveCorrection::Shrink),
        _ => Err(Error::Invalid("pave correction")),
    }
}

/// Reset every parameter of an open builder while keeping its host selection.
pub fn reset_session(session: &GemPaveSession) -> Result<GemPaveSession> {
    let mut session = session.clone();
    session.quantity = None;
    session.kind = match &session.kind {
        PaveKind::Stone(params) => PaveKind::Stone(PaveParams::new(params.spec.clone())?),
        PaveKind::Azure(_) => PaveKind::Azure(AzureParams::new()?),
        PaveKind::Prong(_) => PaveKind::Prong(ProngParams::new()?),
        PaveKind::Sphere(params) => PaveKind::Sphere(SphereParams::new(params.spec.clone())?),
        PaveKind::Spring(_) => PaveKind::Spring(SpringParams::new()?),
    };
    session.recompute(&Limits::default())?;
    Ok(session)
}
