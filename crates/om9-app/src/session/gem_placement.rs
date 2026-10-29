// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gems on curves and surfaces (`OM9-L2-04-10`).
//!
//! This module is the Matrix command surface of the placement family: `Gem on Curve`
//! (`gvGemOnCrv`), its advanced/taper and count/list/multi variants, `Gem on Surface`,
//! `Gem Follow`, `Gem Flow`, `Gem Splop`, `Gem Profile` and `Gem Guides`. It is a
//! dispatcher over the typed owner `om9_core::gem_placement`:
//!
//! * the domain (arc-length stations, surface frames, follow/flow/splop and the persisted
//!   link) lives in `om9_core::gem_placement`;
//! * this module turns a command into session state, computes the preview from the same
//!   service and commits ONE transaction on Apply, so Undo restores exactly the previous
//!   document;
//! * a preview never inserts an object; Cancel drops the session and leaves no phantom.
//!
//! The original Matrix command names are preserved and both the palette icons and the command
//! line reach the same handler.

use om9_core::{
    Error, Result,
    gem::{GemId, GemRecord, GemSpec, build_mesh, gem_record},
    gem_placement::{
        CurveFit, CurveParams, CurvePlacement, GemStation, PlacementGuides, PlacementLink,
        PlacementSource, SplopParams, SurfaceParams, count_on_curve, flow_station,
        flow_surface_placement, place_on_curve, place_on_curve_variable, place_on_curves,
        place_on_surface, profile_and_guides, splop_station, station_at_arc, taper_specs,
    },
    geometry::{Curve, Geometry, NurbsSurface, Transform},
    identity::ObjectId,
    model::{Entity, GEM_EXTENSION_SLOT, GEM_PLACEMENT_EXTENSION_SLOT},
    transaction::Mutation,
};

use super::Controller;

/// `Builder Gem on Curve` palette icon / `gvGemOnCrv` (`OM9-L3-04-10-001`).
pub const ON_CURVE: &str = "BuilderGemOnCrv";
/// `Builder Gem on Curve Advanced` (`OM9-L3-04-10-002`).
pub const ON_CURVE_ADVANCED: &str = "BuilderGemOnCrvAdvanced";
/// `Builder Gem Count on Curve` (`OM9-L3-04-10-003`).
pub const COUNT_ON_CURVE: &str = "BuilderGemCountOnCrv";
/// `Builder Gem List on Curve` (`OM9-L3-04-10-004`).
pub const LIST_ON_CURVE: &str = "BuilderGemListOnCrv";
/// `Builder Gem on Curve Multi` (`OM9-L3-04-10-005`).
pub const ON_CURVE_MULTI: &str = "BuilderGemOnCrvMulti";
/// `Builder Gem on Surface` (`OM9-L3-04-10-007`).
pub const ON_SURFACE: &str = "BuilderGemOnSurface";
/// `Builder Gem Profile` (`OM9-L3-04-10-006`).
pub const PROFILE: &str = "BuilderGemProfile";
/// `Builder Gem Guides` (`OM9-L3-04-10-006`).
pub const GUIDES: &str = "BuilderGemGuides";
/// `Builder Gem Follow` (`OM9-L3-04-10-008`).
pub const FOLLOW: &str = "BuilderGemFollow";
/// `Builder GVD Gem Flow` (`OM9-L3-04-10-009`).
pub const FLOW: &str = "BuilderGVDGemFlow";
/// `Builder GVD Gem Splop` (`OM9-L3-04-10-010`).
pub const SPLOP: &str = "BuilderGVDGemSplop";
/// Command-line report of the placement links (`OM9-L3-04-10-011`).
pub const LINKS: &str = "GemPlacementLinks";

const DEFAULT_MATERIAL: &str = "diamond";
const DEFAULT_COLOR: [u8; 4] = [180, 180, 180, 255];
const INITIAL_SPACING_MM: f64 = 8.0;
const INITIAL_DIAMETER_MM: f64 = 6.0;

/// The placement tool an open session represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemPlacementTool {
    OnCurve,
    OnCurveAdvanced,
    CountOnCurve,
    ListOnCurve,
    OnCurveMulti,
    OnSurface,
    Follow,
    Flow,
    Splop,
    Profile,
    Guides,
}

impl GemPlacementTool {
    pub fn command(self) -> &'static str {
        match self {
            Self::OnCurve => ON_CURVE,
            Self::OnCurveAdvanced => ON_CURVE_ADVANCED,
            Self::CountOnCurve => COUNT_ON_CURVE,
            Self::ListOnCurve => LIST_ON_CURVE,
            Self::OnCurveMulti => ON_CURVE_MULTI,
            Self::OnSurface => ON_SURFACE,
            Self::Follow => FOLLOW,
            Self::Flow => FLOW,
            Self::Splop => SPLOP,
            Self::Profile => PROFILE,
            Self::Guides => GUIDES,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::OnCurve => "Gem on Curve",
            Self::OnCurveAdvanced => "Gem on Curve Advanced",
            Self::CountOnCurve => "Gem Count on Curve",
            Self::ListOnCurve => "Gem List on Curve",
            Self::OnCurveMulti => "Gem on Curve Multi",
            Self::OnSurface => "Gem on Surface",
            Self::Follow => "Gem Follow",
            Self::Flow => "Gem Flow",
            Self::Splop => "Gem Splop",
            Self::Profile => "Gem Profile",
            Self::Guides => "Gem Guides",
        }
    }
    /// Tools that edit existing gems instead of inserting new ones.
    pub fn edits_existing(self) -> bool {
        matches!(self, Self::Follow | Self::Flow | Self::Splop)
    }
    /// Tools that insert gems.
    pub fn inserts_gems(self) -> bool {
        matches!(
            self,
            Self::OnCurve
                | Self::OnCurveAdvanced
                | Self::CountOnCurve
                | Self::ListOnCurve
                | Self::OnCurveMulti
                | Self::OnSurface
        )
    }
}

/// Whether this module owns the feature id.
pub fn handles(feature: &str) -> bool {
    matches!(
        feature,
        ON_CURVE
            | ON_CURVE_ADVANCED
            | COUNT_ON_CURVE
            | LIST_ON_CURVE
            | ON_CURVE_MULTI
            | ON_SURFACE
            | PROFILE
            | GUIDES
            | FOLLOW
            | FLOW
            | SPLOP
            | LINKS
    )
}

/// The typed command names that reach the same services.
pub fn command_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "gvgemoncrv" | "gemoncrv" | "gem on curve" | "gemoncurve" => Some(ON_CURVE),
        "gvgemoncrvadvanced" | "gemoncrvadvanced" | "gem on curve advanced" => {
            Some(ON_CURVE_ADVANCED)
        }
        "gvgemcountoncrv" | "gemcountoncrv" | "gem count on curve" => Some(COUNT_ON_CURVE),
        "gvgemlistoncrv" | "gemlistoncrv" | "gem list on curve" => Some(LIST_ON_CURVE),
        "gvgemoncrvmulti" | "gemoncrvmulti" | "gem on curve multi" => Some(ON_CURVE_MULTI),
        "gvgemonsurface" | "gvgemonsrf" | "gem on surface" => Some(ON_SURFACE),
        "gvgemprofile" | "gemprofile" | "gem profile" => Some(PROFILE),
        "gvgemguides" | "gemguides" | "gem guides" => Some(GUIDES),
        "gvgemfollow" | "gemfollow" | "gem follow" => Some(FOLLOW),
        "gvgemflow" | "gemflow" | "gem flow" => Some(FLOW),
        "gvgemsplop" | "gemsplop" | "gem splop" => Some(SPLOP),
        "gemplacementlinks" | "gem placement links" | "gemlinks" => Some(LINKS),
        _ => None,
    }
}

/// One gem edited by Follow/Flow/Splop.
#[derive(Debug, Clone)]
struct GemTarget {
    id: ObjectId,
    record: GemRecord,
    link: PlacementLink,
}

/// One recomputed station for an existing gem. `station == None` means the source no longer
/// resolves: the link becomes a visible conflict while the gem keeps its last place.
#[derive(Debug, Clone)]
struct GemUpdate {
    gem: ObjectId,
    station: Option<GemStation>,
    link: PlacementLink,
}

/// One open placement session. The session is the whole preview: nothing exists in the
/// document until Apply commits one transaction.
#[derive(Debug, Clone)]
pub struct GemPlacementSession {
    pub tool: GemPlacementTool,
    pub spec: GemSpec,
    pub end_spec: Option<GemSpec>,
    pub params: CurveParams,
    pub surface_params: SurfaceParams,
    pub splop: SplopParams,
    /// The run's curves with their world geometry, in selection order.
    pub curves: Vec<(ObjectId, Curve)>,
    /// Every selected surface, before a tool claims its target.
    pub surfaces: Vec<(ObjectId, NurbsSurface)>,
    /// The flow target curve (the selected curve).
    pub target: Option<(ObjectId, Curve)>,
    /// The flow target surface (the selected surface).
    pub target_surface: Option<(ObjectId, NurbsSurface)>,
    /// The surface a surface tool claims.
    pub surface: Option<(ObjectId, NurbsSurface)>,
    /// Gems edited by Follow/Flow/Splop.
    gems: Vec<GemTarget>,
    /// Curve plans of the preview (curve tools).
    pub plans: Vec<CurvePlacement>,
    /// Surface stations of the preview (surface tool).
    pub surface_stations: Vec<GemStation>,
    /// Profile/girdle/guide curves of the preview.
    pub guides: Option<PlacementGuides>,
    /// Recomputed stations for existing gems.
    updates: Vec<GemUpdate>,
    /// Explicit conflicts of a follow/flow/splop, as human-readable lines.
    pub conflicts: Vec<String>,
    /// Surface UV grid used by the surface tool.
    pub grid: usize,
    /// Exact station count requested with `Count=` (`OM9-L3-04-10-001`).
    pub count: Option<usize>,
    /// Gap and end allowance of the count tool.
    pub gap_mm: f64,
    pub allowance_mm: f64,
    /// Heterogeneous diameters of the list tool.
    pub diameters: Vec<f64>,
    /// Latest feasible count of the count tool.
    pub fit: Option<CurveFit>,
}

impl GemPlacementSession {
    /// Number of gems this session would insert or move.
    pub fn preview_count(&self) -> usize {
        if self.tool.edits_existing() {
            return self
                .updates
                .iter()
                .filter(|update| update.station.is_some())
                .count();
        }
        if self.tool == GemPlacementTool::Profile || self.tool == GemPlacementTool::Guides {
            return self.guides.as_ref().map_or(0, |guides| {
                guides.guides.len()
                    + if self.tool == GemPlacementTool::Profile {
                        2
                    } else {
                        0
                    }
            });
        }
        self.plans.iter().map(CurvePlacement::count).sum::<usize>() + self.surface_stations.len()
    }

    /// Open a session for `tool` from the current selection and compute its preview.
    fn open(tool: GemPlacementTool, controller: &Controller) -> Result<Self> {
        let limits = *controller.coordinator.limits();
        let selection = controller.selected_ordered();
        let mut curves = Vec::new();
        let mut surfaces = Vec::new();
        let mut gems = Vec::new();
        for id in &selection {
            let entity = controller
                .document()
                .entities
                .get(id)
                .ok_or(Error::Stale("gem placement selection"))?;
            match &entity.geometry {
                Geometry::Curve(curve) => {
                    curves.push((*id, bake_curve(curve, entity.placement, &limits)?));
                }
                Geometry::Surface(surface) => {
                    surfaces.push((*id, bake_surface(surface, entity.placement, &limits)?));
                }
                _ => {}
            }
            if let Some(record) = super::gem::entity_record(controller.document(), entity) {
                let link = entity
                    .extensions
                    .get(GEM_PLACEMENT_EXTENSION_SLOT)
                    .and_then(|extension| PlacementLink::from_extension(extension).ok());
                if let Some(link) = link {
                    gems.push(GemTarget {
                        id: *id,
                        record,
                        link,
                    });
                }
            }
        }
        // A selected gem's own size seeds the new row, so a row matches the stones around it.
        let spec = match gems.first() {
            Some(gem) => gem.record.spec.clone(),
            None => GemSpec::round(INITIAL_DIAMETER_MM, INITIAL_DIAMETER_MM * 0.65)?,
        };
        let mut session = Self {
            tool,
            spec,
            end_spec: None,
            params: CurveParams::spacing(INITIAL_SPACING_MM),
            surface_params: SurfaceParams::new(vec![[0.5, 0.5]]),
            splop: SplopParams::default(),
            curves,
            surfaces,
            target: None,
            target_surface: None,
            surface: None,
            gems,
            plans: Vec::new(),
            surface_stations: Vec::new(),
            guides: None,
            updates: Vec::new(),
            conflicts: Vec::new(),
            grid: 1,
            count: None,
            gap_mm: 1.0,
            allowance_mm: 0.0,
            diameters: Vec::new(),
            fit: None,
        };
        // The selection contract per tool: the wrong input is a typed refusal before a
        // session exists.
        session.claim_selection()?;
        session.recompute(controller)?;
        Ok(session)
    }

    fn claim_selection(&mut self) -> Result<()> {
        match self.tool {
            GemPlacementTool::OnCurve
            | GemPlacementTool::OnCurveAdvanced
            | GemPlacementTool::CountOnCurve
            | GemPlacementTool::ListOnCurve => {
                if self.curves.len() != 1 || !self.surfaces.is_empty() {
                    return Err(Error::Invalid("gem on curve needs one curve"));
                }
                if self.gems.len() > 1 {
                    return Err(Error::Invalid("gem on curve needs one curve"));
                }
            }
            GemPlacementTool::OnCurveMulti => {
                if self.curves.len() < 2 {
                    return Err(Error::Invalid("gem on curve multi needs two curves"));
                }
            }
            GemPlacementTool::OnSurface => {
                if self.surfaces.len() != 1 || !self.curves.is_empty() {
                    return Err(Error::Invalid("gem on surface needs one surface"));
                }
                self.surface = self.surfaces.first().cloned();
            }
            GemPlacementTool::Follow => {
                if self.gems.is_empty() {
                    return Err(Error::Invalid("gem follow needs placed gems"));
                }
            }
            GemPlacementTool::Flow => {
                if self.gems.is_empty() {
                    return Err(Error::Invalid("gem flow needs placed gems"));
                }
                match (self.curves.len(), self.surfaces.len()) {
                    (1, 0) => {
                        self.target = self.curves.first().cloned();
                        self.curves.clear();
                    }
                    (0, 1) => self.target_surface = self.surfaces.first().cloned(),
                    _ => {
                        return Err(Error::Invalid("gem flow needs one target curve or surface"));
                    }
                }
            }
            GemPlacementTool::Splop => {
                if self.gems.is_empty() || self.surfaces.len() != 1 {
                    return Err(Error::Invalid("gem splop needs gems and one surface"));
                }
                self.surface = self.surfaces.first().cloned();
            }
            GemPlacementTool::Profile | GemPlacementTool::Guides => {
                if self.gems.len() < 2 {
                    return Err(Error::Invalid("gem profile needs two placed gems"));
                }
            }
        }
        Ok(())
    }

    /// Full recompute from the current parameters and the document state.
    fn recompute(&mut self, controller: &Controller) -> Result<()> {
        self.plans.clear();
        self.surface_stations.clear();
        self.guides = None;
        self.updates.clear();
        self.conflicts.clear();
        self.fit = None;
        let limits = *controller.coordinator.limits();
        match self.tool {
            GemPlacementTool::OnCurve => {
                if let Some((id, curve)) = self.curves.first().cloned() {
                    let params = self.params_with_count(&curve, &limits)?;
                    let mut plan = place_on_curve(&curve, &self.spec, &params, &limits)?;
                    if self.count.is_some_and(|count| plan.count() != count) {
                        return Err(Error::Invalid("gem count placement"));
                    }
                    plan.curve = Some(id);
                    self.plans.push(plan);
                }
            }
            GemPlacementTool::OnCurveAdvanced => {
                if let Some((id, curve)) = self.curves.first().cloned() {
                    let params = self.params_with_count(&curve, &limits)?;
                    let base = place_on_curve(&curve, &self.spec, &params, &limits)?;
                    let end = match &self.end_spec {
                        Some(spec) => spec.clone(),
                        None => self
                            .spec
                            .clone()
                            .set_girdle_mm(self.spec.length_mm * 0.6, self.spec.width_mm * 0.6)?,
                    };
                    let specs = taper_specs(&self.spec, &end, base.count())?;
                    let mut plan = place_on_curve_variable(&curve, &specs, &params, &limits)?;
                    plan.curve = Some(id);
                    self.plans.push(plan);
                }
            }
            GemPlacementTool::CountOnCurve => {
                if let Some((id, curve)) = self.curves.first().cloned() {
                    let fit = count_on_curve(
                        &curve,
                        &self.spec,
                        self.gap_mm,
                        self.allowance_mm,
                        &limits,
                    )?;
                    self.fit = Some(fit.clone());
                    if !fit.fits {
                        return Err(Error::Invalid("gem count on curve does not fit"));
                    }
                    let params = CurveParams::spacing(fit.spacing_mm)
                        .with_range(fit.first_center_mm, Some(fit.last_center_mm))
                        .with_z_offset(self.params.z_offset_mm)
                        .with_rotation(self.params.rotation_deg)
                        .with_orientation(self.params.orientation);
                    let mut plan = place_on_curve(&curve, &self.spec, &params, &limits)?;
                    plan.curve = Some(id);
                    self.plans.push(plan);
                }
            }
            GemPlacementTool::ListOnCurve => {
                if let Some((id, curve)) = self.curves.first().cloned() {
                    let diameters = if self.diameters.is_empty() {
                        vec![self.spec.length_mm; 3]
                    } else {
                        self.diameters.clone()
                    };
                    let specs: Vec<GemSpec> = diameters
                        .iter()
                        .map(|diameter| GemSpec::round(*diameter, (*diameter * 0.65).max(0.1)))
                        .collect::<Result<_>>()?;
                    let total = om9_core::curve_ops::arc_length(&curve, &limits)?;
                    let start = self.params.start_mm.min(total);
                    let end = self.params.end_mm.unwrap_or(total).min(total).max(start);
                    let available = end - start;
                    let count = specs.len();
                    let spacing = if count > 1 {
                        available / (count - 1) as f64
                    } else {
                        (available + self.spec.length_mm).max(1.0)
                    };
                    let params = CurveParams::spacing(spacing)
                        .with_range(start, Some(end))
                        .with_z_offset(self.params.z_offset_mm)
                        .with_rotation(self.params.rotation_deg)
                        .with_orientation(self.params.orientation)
                        .with_overlap(self.params.allow_overlap);
                    let mut plan = place_on_curve_variable(&curve, &specs, &params, &limits)?;
                    plan.curve = Some(id);
                    self.plans.push(plan);
                }
            }
            GemPlacementTool::OnCurveMulti => {
                let curves: Vec<Curve> =
                    self.curves.iter().map(|(_, curve)| curve.clone()).collect();
                let ids: Vec<ObjectId> = self.curves.iter().map(|(id, _)| *id).collect();
                let multi = place_on_curves(&curves, &self.spec, &self.params, &limits)?;
                for (mut plan, id) in multi.groups.into_iter().zip(ids) {
                    plan.curve = Some(id);
                    self.plans.push(plan);
                }
            }
            GemPlacementTool::OnSurface => {
                if let Some((_, surface)) = self.surface.clone() {
                    let params = SurfaceParams {
                        uv: self.uv_grid(),
                        z_offset_mm: self.surface_params.z_offset_mm,
                        rotation_deg: self.surface_params.rotation_deg,
                    };
                    let placement = place_on_surface(
                        &surface,
                        std::slice::from_ref(&self.spec),
                        &params,
                        &limits,
                    )?;
                    self.surface_stations = placement.stations;
                }
            }
            GemPlacementTool::Follow => {
                for gem in self.gems.clone() {
                    let PlacementSource::Curve { id } = gem.link.source else {
                        self.conflicts
                            .push(format!("{}: the link is not on a curve", gem.id));
                        continue;
                    };
                    let source =
                        controller.document().entities.get(&id).map(|entity| {
                            match &entity.geometry {
                                Geometry::Curve(curve) => {
                                    bake_curve(curve, entity.placement, &limits).ok()
                                }
                                _ => None,
                            }
                        });
                    let result = source
                        .flatten()
                        .ok_or(Error::Stale("gem follow source"))
                        .and_then(|curve| {
                            station_at_arc(
                                &curve,
                                gem.link.arc_length_mm,
                                &gem.record.spec,
                                &self.link_params(&gem.link),
                                &limits,
                            )
                        });
                    match result {
                        Ok(station) => {
                            let mut link = gem.link.clone();
                            link.refresh(
                                self.source_revision(controller, id),
                                station.arc_length_mm,
                                station.fraction,
                            )?;
                            self.updates.push(GemUpdate {
                                gem: gem.id,
                                station: Some(station),
                                link,
                            });
                        }
                        Err(_) => {
                            self.conflicts
                                .push(format!("{}: the source station is gone", gem.id));
                            self.updates.push(GemUpdate {
                                gem: gem.id,
                                station: None,
                                link: conflicted(&gem.link),
                            });
                        }
                    }
                }
            }
            GemPlacementTool::Flow => {
                if let Some((target_id, target)) = self.target.clone() {
                    self.flow_onto_curve(controller, target_id, &target, &limits)?;
                } else if let Some((target_id, target)) = self.target_surface.clone() {
                    self.flow_onto_surface(controller, target_id, &target, &limits)?;
                } else {
                    return Err(Error::Missing("gem flow target"));
                }
            }
            GemPlacementTool::Splop => {
                let Some((surface_id, surface)) = self.surface.clone() else {
                    return Err(Error::Missing("gem splop surface"));
                };
                for gem in self.gems.clone() {
                    let point = controller
                        .document()
                        .entities
                        .get(&gem.id)
                        .and_then(|entity| entity.placement.point([0.0, 0.0, 0.0]).ok());
                    let result = point
                        .ok_or(Error::Stale("gem splop gem"))
                        .and_then(|point| {
                            splop_station(&surface, point, &gem.record.spec, &self.splop, &limits)
                        });
                    match result {
                        Ok(station) => {
                            let uv = station.uv.unwrap_or([0.5, 0.5]);
                            let link = PlacementLink::new(
                                gem.record.gem_id,
                                PlacementSource::Surface { id: surface_id },
                                0.0,
                                0.0,
                                self.source_revision(controller, surface_id),
                                station.spec.length_mm.max(1.0e-6),
                                station.z_offset_mm,
                            )?
                            .with_uv(uv)?;
                            self.updates.push(GemUpdate {
                                gem: gem.id,
                                station: Some(station),
                                link,
                            });
                        }
                        Err(_) => {
                            self.conflicts
                                .push(format!("{}: the splop could not be resolved", gem.id));
                            self.updates.push(GemUpdate {
                                gem: gem.id,
                                station: None,
                                link: conflicted(&gem.link),
                            });
                        }
                    }
                }
            }
            GemPlacementTool::Profile | GemPlacementTool::Guides => {
                let mut stations: Vec<GemStation> = Vec::new();
                for gem in &self.gems {
                    let PlacementSource::Curve { id } = gem.link.source else {
                        continue;
                    };
                    let Some(entity) = controller.document().entities.get(&id) else {
                        continue;
                    };
                    let Geometry::Curve(curve) = &entity.geometry else {
                        continue;
                    };
                    let curve = bake_curve(curve, entity.placement, &limits)?;
                    if let Ok(station) = station_at_arc(
                        &curve,
                        gem.link.arc_length_mm,
                        &gem.record.spec,
                        &self.link_params(&gem.link),
                        &limits,
                    ) {
                        stations.push(station);
                    }
                }
                if stations.len() < 2 {
                    return Err(Error::Invalid("gem profile needs two placed gems"));
                }
                stations.sort_by(|a, b| {
                    a.arc_length_mm
                        .partial_cmp(&b.arc_length_mm)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (index, station) in stations.iter_mut().enumerate() {
                    station.index = index;
                }
                let line = CurvePlacement {
                    curve: None,
                    params: CurveParams::spacing(self.params.spacing_mm.max(1.0e-6)),
                    total_length_mm: stations.last().map_or(0.0, |station| station.arc_length_mm),
                    range: [0.0, 0.0],
                    closed: false,
                    stations,
                    gaps_mm: Vec::new(),
                };
                self.guides = Some(profile_and_guides(&line, &limits)?);
            }
        }
        Ok(())
    }

    fn uv_grid(&self) -> Vec<[f64; 2]> {
        let count = self.grid.clamp(1, 10);
        let mut uvs = Vec::new();
        for i in 0..count {
            for j in 0..count {
                let u = (i as f64 + 1.0) / (count as f64 + 1.0);
                let v = (j as f64 + 1.0) / (count as f64 + 1.0);
                uvs.push([u, v]);
            }
        }
        uvs
    }

    /// Derive the spacing of an exact `Count=` request over the requested range
    /// (`OM9-L3-04-10-001`). The end is recomputed from the spacing so the last station lands
    /// exactly on it instead of drifting past the curve.
    fn params_with_count(
        &self,
        curve: &Curve,
        limits: &om9_core::budget::Limits,
    ) -> Result<CurveParams> {
        let Some(count) = self.count else {
            return Ok(self.params);
        };
        if count == 0 {
            return Err(Error::Invalid("gem count"));
        }
        let total = om9_core::curve_ops::arc_length(curve, limits)?;
        let start = self.params.start_mm.clamp(0.0, total);
        let end = self
            .params
            .end_mm
            .unwrap_or(total)
            .clamp(start, total)
            .max(start);
        let spacing = if count > 1 {
            ((end - start) / (count - 1) as f64).max(1.0e-9)
        } else {
            (end - start).max(1.0)
        };
        let end = start + spacing * (count.saturating_sub(1)) as f64;
        Ok(CurveParams {
            spacing_mm: spacing,
            ..self.params.with_range(start, Some(end))
        })
    }

    /// Flow curve-linked gems from their base curve onto the target curve.
    fn flow_onto_curve(
        &mut self,
        controller: &Controller,
        target_id: ObjectId,
        target: &Curve,
        limits: &om9_core::budget::Limits,
    ) -> Result<()> {
        for gem in self.gems.clone() {
            let PlacementSource::Curve { id } = gem.link.source else {
                self.conflicts
                    .push(format!("{}: the link is not on a curve", gem.id));
                continue;
            };
            let source =
                controller
                    .document()
                    .entities
                    .get(&id)
                    .and_then(|entity| match &entity.geometry {
                        Geometry::Curve(curve) => bake_curve(curve, entity.placement, limits).ok(),
                        _ => None,
                    });
            let result = source
                .ok_or(Error::Stale("gem flow source"))
                .and_then(|source| {
                    let station = station_at_arc(
                        &source,
                        gem.link.arc_length_mm,
                        &gem.record.spec,
                        &self.link_params(&gem.link),
                        limits,
                    )?;
                    flow_station(&source, target, &station, limits)
                });
            match result {
                Ok(station) => {
                    let link = PlacementLink::new(
                        gem.record.gem_id,
                        PlacementSource::Curve { id: target_id },
                        station.arc_length_mm,
                        station.fraction,
                        self.source_revision(controller, target_id),
                        self.params.spacing_mm.max(1.0e-6),
                        station.z_offset_mm,
                    )?;
                    self.updates.push(GemUpdate {
                        gem: gem.id,
                        station: Some(station),
                        link,
                    });
                }
                Err(_) => {
                    self.conflicts
                        .push(format!("{}: the flow could not be resolved", gem.id));
                    self.updates.push(GemUpdate {
                        gem: gem.id,
                        station: None,
                        link: conflicted(&gem.link),
                    });
                }
            }
        }
        Ok(())
    }

    /// Flow surface-linked gems from their base surface onto the target surface.
    ///
    /// The frame is warped by the shared `deform::rigid_flow` owner, so the stone keeps its
    /// proportions; the new UV is read back from the target so the link stays usable.
    fn flow_onto_surface(
        &mut self,
        controller: &Controller,
        target_id: ObjectId,
        target: &NurbsSurface,
        limits: &om9_core::budget::Limits,
    ) -> Result<()> {
        for gem in self.gems.clone() {
            let PlacementSource::Surface { id } = gem.link.source else {
                self.conflicts
                    .push(format!("{}: the link is not on a surface", gem.id));
                continue;
            };
            let base =
                controller
                    .document()
                    .entities
                    .get(&id)
                    .and_then(|entity| match &entity.geometry {
                        Geometry::Surface(surface) => {
                            bake_surface(surface, entity.placement, limits).ok()
                        }
                        _ => None,
                    });
            let entity = controller.document().entities.get(&gem.id).cloned();
            let result = match (base, entity) {
                (Some(base), Some(entity)) => entity
                    .placement
                    .point([0.0, 0.0, 0.0])
                    .map_err(|_| Error::Invalid("gem flow frame"))
                    .and_then(|point| {
                        let matrix = entity.placement.0;
                        let station = GemStation {
                            index: 0,
                            fraction: 0.0,
                            arc_length_mm: 0.0,
                            point,
                            tangent: [matrix[0][0], matrix[1][0], matrix[2][0]],
                            axis: [matrix[0][2], matrix[1][2], matrix[2][2]],
                            uv: gem.link.uv,
                            transform: entity.placement,
                            spec: gem.record.spec.clone(),
                            z_offset_mm: gem.link.z_offset_mm,
                            spin_deg: 0.0,
                            gem: None,
                        };
                        let line = CurvePlacement {
                            curve: None,
                            params: CurveParams::spacing(gem.record.spec.length_mm.max(1.0e-6)),
                            total_length_mm: 0.0,
                            range: [0.0, 0.0],
                            closed: false,
                            stations: vec![station],
                            gaps_mm: Vec::new(),
                        };
                        flow_surface_placement(&base, target, &line, limits)
                    }),
                _ => Err(Error::Stale("gem flow source")),
            };
            match result {
                Ok(flowed) => {
                    let Some(station) = flowed.stations.into_iter().next() else {
                        return Err(Error::Invalid("gem flow station"));
                    };
                    let uv = om9_core::curve_surface_ops::closest_point_on_surface(
                        target,
                        station.point,
                        limits,
                    )
                    .map(|projection| projection.parameters)
                    .ok()
                    .or(gem.link.uv);
                    let mut link = PlacementLink::new(
                        gem.record.gem_id,
                        PlacementSource::Surface { id: target_id },
                        0.0,
                        0.0,
                        self.source_revision(controller, target_id),
                        station.spec.length_mm.max(1.0e-6),
                        station.z_offset_mm,
                    )?;
                    if let Some(uv) = uv {
                        link = link.with_uv(uv)?;
                    }
                    self.updates.push(GemUpdate {
                        gem: gem.id,
                        station: Some(station),
                        link,
                    });
                }
                Err(_) => {
                    self.conflicts
                        .push(format!("{}: the flow could not be resolved", gem.id));
                    self.updates.push(GemUpdate {
                        gem: gem.id,
                        station: None,
                        link: conflicted(&gem.link),
                    });
                }
            }
        }
        Ok(())
    }

    /// The parameters a link recorded, so a follow repeats the same frame rule.
    fn link_params(&self, link: &PlacementLink) -> CurveParams {
        CurveParams {
            start_mm: 0.0,
            end_mm: None,
            spacing_mm: link.spacing_mm.max(1.0e-6),
            z_offset_mm: link.z_offset_mm,
            rotation_deg: self.params.rotation_deg,
            orientation: self.params.orientation,
            allow_overlap: true,
            skip_seam: true,
        }
    }

    fn source_revision(&self, controller: &Controller, id: ObjectId) -> u64 {
        controller
            .document()
            .entities
            .get(&id)
            .map_or(1, |entity| entity.geometry_revision)
    }
}

fn conflicted(link: &PlacementLink) -> PlacementLink {
    let mut link = link.clone();
    let _ = link.mark_conflict();
    link
}

/// The session prompt of the command line.
pub fn prompt(session: &GemPlacementSession) -> String {
    let options = match session.tool {
        GemPlacementTool::OnCurve => "Start= End= Spacing= Count= ZOffset= Rotate= Size= Flip=",
        GemPlacementTool::OnCurveAdvanced => {
            "Start= End= Spacing= Count= ZOffset= Rotate= Size= EndSize= Flip="
        }
        GemPlacementTool::CountOnCurve => "Gap= Allowance= ZOffset= Rotate= Size=",
        GemPlacementTool::ListOnCurve => "Start= End= Sizes= ZOffset= Rotate= Flip=",
        GemPlacementTool::OnCurveMulti => "Start= End= Spacing= ZOffset= Rotate= Size=",
        GemPlacementTool::OnSurface => "U= V= Grid= ZOffset= Rotate= Size=",
        GemPlacementTool::Profile => "Start= End= Spacing= Size=",
        GemPlacementTool::Guides => "Start= End= Spacing= Size=",
        GemPlacementTool::Follow => "",
        GemPlacementTool::Flow => "",
        GemPlacementTool::Splop => "ZOffset= Rotate=",
    };
    if options.is_empty() {
        return format!(
            "{}: {} gem(s) — (Enter=Apply, Esc=Cancel): ",
            session.tool.title(),
            session.preview_count()
        );
    }
    format!(
        "{}: {} gem(s), spacing {:.3} mm — {} (Enter=Apply, Esc=Cancel): ",
        session.tool.title(),
        session.preview_count(),
        session.params.spacing_mm,
        options
    )
}

/// Whether a tool consumes an option key. Advertised keys are enforced so a typed option is
/// never accepted and silently ignored.
fn option_used(tool: GemPlacementTool, key: &str) -> bool {
    match key {
        "start" | "end" => matches!(
            tool,
            GemPlacementTool::OnCurve
                | GemPlacementTool::OnCurveAdvanced
                | GemPlacementTool::ListOnCurve
                | GemPlacementTool::OnCurveMulti
                | GemPlacementTool::Profile
                | GemPlacementTool::Guides
        ),
        "spacing" => matches!(
            tool,
            GemPlacementTool::OnCurve
                | GemPlacementTool::OnCurveAdvanced
                | GemPlacementTool::OnCurveMulti
                | GemPlacementTool::Profile
                | GemPlacementTool::Guides
        ),
        "count" => matches!(
            tool,
            GemPlacementTool::OnCurve | GemPlacementTool::OnCurveAdvanced
        ),
        "zoffset" | "rotate" | "rotation" => matches!(
            tool,
            GemPlacementTool::OnCurve
                | GemPlacementTool::OnCurveAdvanced
                | GemPlacementTool::CountOnCurve
                | GemPlacementTool::ListOnCurve
                | GemPlacementTool::OnCurveMulti
                | GemPlacementTool::OnSurface
                | GemPlacementTool::Splop
                | GemPlacementTool::Profile
                | GemPlacementTool::Guides
        ),
        "size" => matches!(
            tool,
            GemPlacementTool::OnCurve
                | GemPlacementTool::OnCurveAdvanced
                | GemPlacementTool::CountOnCurve
                | GemPlacementTool::ListOnCurve
                | GemPlacementTool::OnCurveMulti
                | GemPlacementTool::OnSurface
                | GemPlacementTool::Profile
                | GemPlacementTool::Guides
        ),
        "endsize" => tool == GemPlacementTool::OnCurveAdvanced,
        "sizes" => tool == GemPlacementTool::ListOnCurve,
        "gap" | "allowance" => tool == GemPlacementTool::CountOnCurve,
        "u" | "v" | "grid" => tool == GemPlacementTool::OnSurface,
        "flip" | "overlap" => matches!(
            tool,
            GemPlacementTool::OnCurve
                | GemPlacementTool::OnCurveAdvanced
                | GemPlacementTool::CountOnCurve
                | GemPlacementTool::ListOnCurve
                | GemPlacementTool::OnCurveMulti
        ),
        _ => false,
    }
}

fn bake_curve(
    curve: &Curve,
    placement: Transform,
    limits: &om9_core::budget::Limits,
) -> Result<Curve> {
    if placement == Transform::identity() {
        return Ok(curve.clone());
    }
    match om9_core::top11::bake_geometry(&Geometry::Curve(curve.clone()), placement, limits)? {
        Geometry::Curve(curve) => Ok(curve),
        _ => Err(Error::Invalid("gem placement curve")),
    }
}

fn bake_surface(
    surface: &NurbsSurface,
    placement: Transform,
    limits: &om9_core::budget::Limits,
) -> Result<NurbsSurface> {
    if placement == Transform::identity() {
        return Ok(surface.clone());
    }
    match om9_core::top11::bake_geometry(&Geometry::Surface(surface.clone()), placement, limits)? {
        Geometry::Surface(surface) => Ok(surface),
        _ => Err(Error::Invalid("gem placement surface")),
    }
}

impl Controller {
    /// Open a placement tool. The caller has already checked `availability`; this method does
    /// the selection and preview work.
    pub fn dispatch_gem_placement(&mut self, feature: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let feature = command_name(feature).unwrap_or(feature);
        if feature == LINKS {
            return self.report_gem_placement_links();
        }
        let tool = match feature {
            ON_CURVE => GemPlacementTool::OnCurve,
            ON_CURVE_ADVANCED => GemPlacementTool::OnCurveAdvanced,
            COUNT_ON_CURVE => GemPlacementTool::CountOnCurve,
            LIST_ON_CURVE => GemPlacementTool::ListOnCurve,
            ON_CURVE_MULTI => GemPlacementTool::OnCurveMulti,
            ON_SURFACE => GemPlacementTool::OnSurface,
            FOLLOW => GemPlacementTool::Follow,
            FLOW => GemPlacementTool::Flow,
            SPLOP => GemPlacementTool::Splop,
            PROFILE => GemPlacementTool::Profile,
            GUIDES => GemPlacementTool::Guides,
            _ => return Err(Error::Unsupported("gem placement command")),
        };
        let session = GemPlacementSession::open(tool, self)?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            tool.command(),
            tool.title(),
            None,
        );
        self.gem_placement = Some(session);
        Ok(())
    }

    pub fn gem_placement_session(&self) -> Option<&GemPlacementSession> {
        self.gem_placement.as_ref()
    }

    /// Cancel: the session is dropped and the document was never touched.
    pub fn gem_placement_cancel(&mut self) -> bool {
        let Some(session) = self.gem_placement.take() else {
            return false;
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            session.tool.command(),
            "gem placement canceled",
            Some(crate::command_events::EventOutcome::Cancelled),
        );
        true
    }

    /// Command-line / Enter handling for an open placement tool. `None` means no tool is open.
    pub fn gem_placement_text(&mut self, text: &str) -> Option<Result<()>> {
        self.gem_placement.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.is_empty() || lower == "apply" || lower == "enter" {
            return Some(self.gem_placement_apply());
        }
        if lower == "cancel" || lower == "esc" || lower == "exit" {
            self.gem_placement_cancel();
            return Some(Ok(()));
        }
        Some(self.gem_placement_option(trimmed))
    }

    pub fn gem_placement_prompt(&self) -> Option<String> {
        self.gem_placement.as_ref().map(prompt)
    }

    fn gem_placement_option(&mut self, text: &str) -> Result<()> {
        let mut session = self
            .gem_placement
            .clone()
            .ok_or(Error::Missing("gem placement session"))?;
        let (key, value) = match text.split_once('=') {
            Some(pair) => pair,
            // A bare number is the Matrix spacing shortcut.
            None => ("spacing", text),
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if !option_used(session.tool, &key) {
            return Err(Error::Invalid("gem placement option"));
        }
        let number = |value: &str| -> Result<f64> {
            value
                .trim_end_matches("mm")
                .trim()
                .parse::<f64>()
                .map_err(|_| Error::Invalid("gem placement option"))
        };
        match key.as_str() {
            "start" => session.params.start_mm = number(value)?,
            "end" => {
                if value.eq_ignore_ascii_case("none") {
                    session.params.end_mm = None;
                } else {
                    session.params.end_mm = Some(number(value)?);
                }
            }
            "spacing" => session.params.spacing_mm = number(value)?,
            "count" => {
                let count = number(value)?;
                if !(count.is_finite() && (1.0..=1_000.0).contains(&count)) {
                    return Err(Error::Invalid("gem count"));
                }
                session.count = Some(count.round() as usize);
            }
            "zoffset" => {
                let value = number(value)?;
                match session.tool {
                    GemPlacementTool::Splop => session.splop.z_offset_mm = value,
                    GemPlacementTool::OnSurface => session.surface_params.z_offset_mm = value,
                    _ => session.params.z_offset_mm = value,
                }
            }
            "rotate" | "rotation" => {
                let value = number(value)?;
                match session.tool {
                    GemPlacementTool::Splop => session.splop.rotation_deg = value,
                    GemPlacementTool::OnSurface => session.surface_params.rotation_deg = value,
                    _ => session.params.rotation_deg = value,
                }
            }
            "size" => {
                let diameter = number(value)?;
                session.spec = GemSpec::round(diameter, (diameter * 0.65).max(0.1))?;
            }
            "endsize" => {
                let diameter = number(value)?;
                session.end_spec = Some(GemSpec::round(diameter, (diameter * 0.65).max(0.1))?);
            }
            "sizes" => {
                let mut diameters = Vec::new();
                for part in value.split(',') {
                    diameters.push(number(part)?);
                }
                session.diameters = diameters;
            }
            "gap" => session.gap_mm = number(value)?,
            "allowance" => session.allowance_mm = number(value)?,
            "u" => session.surface_params.uv[0][0] = number(value)?,
            "v" => session.surface_params.uv[0][1] = number(value)?,
            "grid" => session.grid = number(value)?.clamp(1.0, 10.0) as usize,
            "flip" => {
                let flip = matches!(
                    value.to_ascii_lowercase().as_str(),
                    "yes" | "true" | "1" | "on"
                );
                session.params.orientation = if flip {
                    om9_core::gem_placement::Orientation::Axis([0.0, 0.0, -1.0])
                } else {
                    om9_core::gem_placement::Orientation::Upright
                };
            }
            "overlap" => {
                session.params.allow_overlap = matches!(
                    value.to_ascii_lowercase().as_str(),
                    "yes" | "true" | "1" | "on"
                );
            }
            _ => return Err(Error::Invalid("gem placement option")),
        }
        session.recompute(self)?;
        self.gem_placement = Some(session);
        Ok(())
    }

    /// Apply the open tool as ONE transaction (Insert, or Placement+Extensions for an edit).
    /// A failed apply keeps the session open so the parameters can be corrected.
    pub fn gem_placement_apply(&mut self) -> Result<()> {
        let session = self
            .gem_placement
            .clone()
            .ok_or(Error::Missing("gem placement session"))?;
        match apply(&session, self) {
            Ok(()) => {
                self.gem_placement = None;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    session.tool.command(),
                    "gem placement applied",
                    None,
                );
                Ok(())
            }
            Err(error) => {
                self.gem_placement = Some(session);
                Err(error)
            }
        }
    }

    /// Human-readable report of the placement links on the selected gems
    /// (`OM9-L3-04-10-011`). A gem without a link is reported as unlinked, never invented.
    pub fn gem_placement_links_report(&self) -> Result<String> {
        let selection = self.selected_ordered();
        let ids = if selection.is_empty() {
            self.document()
                .entities
                .values()
                .map(|entity| entity.id)
                .collect::<Vec<_>>()
        } else {
            selection
        };
        let mut lines = Vec::new();
        for id in ids {
            let Some(entity) = self.document().entities.get(&id) else {
                continue;
            };
            if gem_record(entity).is_none() {
                continue;
            }
            match entity
                .extensions
                .get(GEM_PLACEMENT_EXTENSION_SLOT)
                .map(PlacementLink::from_extension)
            {
                Some(Ok(link)) => lines.push(format!(
                    "{id}: {} {} · arc {:.3} mm · station {:.4} · rev {} · uv {}",
                    link.source.tag(),
                    link.source.object(),
                    link.arc_length_mm,
                    link.station,
                    link.source_revision,
                    link.uv
                        .map_or_else(|| "—".to_owned(), |uv| format!("{:.4},{:.4}", uv[0], uv[1]))
                )),
                Some(Err(_)) => lines.push(format!("{id}: placement link is not readable")),
                None => lines.push(format!("{id}: no placement link")),
            }
        }
        if lines.is_empty() {
            return Err(Error::Missing("gem placement links"));
        }
        Ok(lines.join("\n"))
    }

    fn report_gem_placement_links(&mut self) -> Result<()> {
        let report = self.gem_placement_links_report()?;
        self.events.emit(
            crate::command_events::EventKind::Finished,
            LINKS,
            report.as_str(),
            None,
        );
        Ok(())
    }
}

/// Turn the session into one transaction.
fn apply(session: &GemPlacementSession, controller: &mut Controller) -> Result<()> {
    if session.tool.inserts_gems() {
        return apply_inserts(session, controller);
    }
    if session.tool.edits_existing() {
        return apply_edits(session, controller);
    }
    apply_guides(session, controller)
}

fn apply_inserts(session: &GemPlacementSession, controller: &mut Controller) -> Result<()> {
    let layer = controller.gem_layer()?;
    controller.ensure_layer_writable(layer)?;
    let mut mutations = Vec::new();
    for plan in &session.plans {
        let source = plan.curve.ok_or(Error::Missing("gem placement source"))?;
        let revision = revision_of(controller, source);
        for station in &plan.stations {
            let entity = gem_entity_for(station, layer, controller)?;
            let record = gem_record(&entity).ok_or(Error::Invalid("gem record"))?;
            let link = PlacementLink::new(
                record.gem_id,
                PlacementSource::Curve { id: source },
                station.arc_length_mm,
                station.fraction,
                revision,
                plan.params.spacing_mm.max(1.0e-6),
                station.z_offset_mm,
            )?;
            let mut entity = entity;
            entity.extensions.insert(
                GEM_PLACEMENT_EXTENSION_SLOT.to_owned(),
                link.to_extension()?,
            );
            mutations.push(Mutation::Insert {
                document: controller.document,
                entity,
            });
        }
    }
    let Some((source, _)) = session.surface.clone() else {
        if mutations.is_empty() {
            return Err(Error::Missing("gem placement stations"));
        }
        return controller.commit_gem(session.tool.title(), mutations);
    };
    let revision = revision_of(controller, source);
    for station in &session.surface_stations {
        let entity = gem_entity_for(station, layer, controller)?;
        let record = gem_record(&entity).ok_or(Error::Invalid("gem record"))?;
        let uv = station.uv.unwrap_or([0.5, 0.5]);
        let link = PlacementLink::new(
            record.gem_id,
            PlacementSource::Surface { id: source },
            0.0,
            0.0,
            revision,
            station.spec.length_mm.max(1.0e-6),
            station.z_offset_mm,
        )?
        .with_uv(uv)?;
        let mut entity = entity;
        entity.extensions.insert(
            GEM_PLACEMENT_EXTENSION_SLOT.to_owned(),
            link.to_extension()?,
        );
        mutations.push(Mutation::Insert {
            document: controller.document,
            entity,
        });
    }
    if mutations.is_empty() {
        return Err(Error::Missing("gem placement stations"));
    }
    controller.commit_gem(session.tool.title(), mutations)
}

/// Build the gem entity (mesh + `om9.gem` record) for one station, at the station's frame.
fn gem_entity_for(
    station: &GemStation,
    layer: Option<om9_core::identity::LayerId>,
    controller: &Controller,
) -> Result<Entity> {
    let mesh = build_mesh(&station.spec)?;
    if !mesh.closed {
        return Err(Error::Invalid("gem mesh open"));
    }
    let mut entity = Entity::new(
        format!(
            "{} {}",
            station.spec.shape.label(),
            station.spec.cut.label()
        ),
        controller.gem_geometry(mesh, false)?,
        layer,
    );
    entity.placement = station.transform;
    let mut record = GemRecord::new(station.spec.clone()).with_material(
        &station.spec.asset_key(),
        DEFAULT_MATERIAL,
        DEFAULT_COLOR,
    );
    record.gem_id = GemId(entity.id);
    entity
        .extensions
        .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
    Ok(entity)
}

fn revision_of(controller: &Controller, id: ObjectId) -> u64 {
    controller
        .document()
        .entities
        .get(&id)
        .map_or(1, |entity| entity.geometry_revision)
}

fn apply_edits(session: &GemPlacementSession, controller: &mut Controller) -> Result<()> {
    let mut mutations = Vec::new();
    for update in &session.updates {
        let entity = controller
            .document()
            .entities
            .get(&update.gem)
            .ok_or(Error::Stale("gem placement target"))?
            .clone();
        controller.ensure_layer_writable(entity.attributes.layer)?;
        let source = controller.document().stamp(update.gem)?;
        if let Some(station) = &update.station {
            mutations.push(Mutation::Placement {
                source,
                placement: station.transform,
            });
        }
        let mut extensions = entity.extensions.clone();
        extensions.insert(
            GEM_PLACEMENT_EXTENSION_SLOT.to_owned(),
            update.link.to_extension()?,
        );
        mutations.push(Mutation::Extensions { source, extensions });
    }
    if mutations.is_empty() {
        return Err(Error::Missing("gem placement updates"));
    }
    controller.commit_gem(session.tool.title(), mutations)?;
    for reason in &session.conflicts {
        controller.events.emit(
            crate::command_events::EventKind::Warn,
            session.tool.command(),
            reason.as_str(),
            None,
        );
    }
    Ok(())
}

fn apply_guides(session: &GemPlacementSession, controller: &mut Controller) -> Result<()> {
    let guides = session
        .guides
        .as_ref()
        .ok_or(Error::Missing("gem placement guides"))?;
    let layer = controller.gem_layer()?;
    controller.ensure_layer_writable(layer)?;
    let mut curves: Vec<(String, Curve)> = Vec::new();
    if session.tool == GemPlacementTool::Profile {
        curves.push(("Gem profile".to_owned(), guides.profile.clone()));
        curves.push(("Gem girdle".to_owned(), guides.girdle.clone()));
    }
    for (index, guide) in guides.guides.iter().enumerate() {
        curves.push((format!("Gem guide {index}"), guide.clone()));
    }
    let mut mutations = Vec::new();
    for (label, curve) in curves {
        let entity = Entity::new(label, Geometry::Curve(curve), layer);
        mutations.push(Mutation::Insert {
            document: controller.document,
            entity,
        });
    }
    controller.commit_gem(session.tool.title(), mutations)
}
