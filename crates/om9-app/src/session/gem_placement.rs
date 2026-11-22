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
    gem::{GemId, GemRecord, GemShape, GemSpec, build_mesh, gem_record},
    gem_placement::{
        CurveFit, CurveParams, CurvePlacement, GemStation, Orientation, PlacementGuides, PlacementLink, RowFit,
        PlacementSource, SplopParams, SurfaceParams, count_on_curve, flow_station,
        flow_surface_placement, place_on_curve, place_on_curve_variable, place_on_curves,
        fit_gem_row, place_on_surface, profile_and_guides, scale_gem_spec, splop_station, station_at_arc, taper_specs,
    },
    geometry::{Curve, Geometry, NurbsSurface, Point3, Transform},
    identity::{ObjectId, SourceStamp},
    model::{Entity, Extension, GEM_EXTENSION_SLOT, GEM_PLACEMENT_EXTENSION_SLOT},
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

pub const EDIT_CURVE_ROW: &str = "EditGemOnCurve";
pub use om9_core::model::GEM_CURVE_ROW_SLOT as CURVE_ROW_SLOT;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CurveRowOptions {
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) gap: f64,
    pub(crate) fit: RowFit,
    pub(crate) taper_gap: bool,
    pub(crate) mirror: bool,
    pub(crate) tops: bool,
    pub(crate) toward_origin: bool,
    pub(crate) flipped: bool,
    #[serde(default)]
    pub(crate) guide: Option<ObjectId>,
    #[serde(default)]
    pub(crate) snap_corners: bool,
    #[serde(default)]
    pub(crate) show_prongs: bool,
    #[serde(default)]
    pub(crate) sphere: bool,
    pub(crate) count: Option<usize>,
    pub(crate) center_step: Option<f64>,
}

impl Default for CurveRowOptions {
    fn default() -> Self {
        Self {
            start: 0.0, end: 100.0, gap: 0.2, fit: RowFit::Spacing,
            taper_gap: false, mirror: false, tops: false, toward_origin: false,
            flipped: false, guide: None, snap_corners: false,
            show_prongs: false, sphere: false, count: None, center_step: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CurveRowRecipe {
    id: ObjectId,
    source: ObjectId,
    index: usize,
    #[serde(default)]
    prong: bool,
    options: CurveRowOptions,
    spec: GemSpec,
    end_spec: Option<GemSpec>,
    z_offset: f64,
    roll: f64,
    material: String,
}

fn curve_row_recipe(entity: &Entity) -> Option<CurveRowRecipe> {
    let extension = entity.extension(CURVE_ROW_SLOT)?;
    (extension.version == 1).then(|| serde_json::from_str(&extension.payload).ok()).flatten()
}

const DEFAULT_MATERIAL: &str = "diamond";
const DEFAULT_COLOR: [u8; 4] = [180, 180, 180, 255];
const INITIAL_SPACING_MM: f64 = 8.0;
const INITIAL_DIAMETER_MM: f64 = 6.0;

/// The placement tool an open session represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
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
        "gvgemoncrv" | "gemoncrv" | "gem on curve" | "gemoncurve" | "buildergemoncrv" | "om9-gem-002" => Some(ON_CURVE),
        "gvgemoncrvadvanced" | "gemoncrvadvanced" | "gem on curve advanced" => {
            Some(ON_CURVE_ADVANCED)
        }
        "gvgemcountoncrv" | "gemcountoncrv" | "gem count on curve" => Some(COUNT_ON_CURVE),
        "gvgemlistoncrv" | "gemlistoncrv" | "gem list on curve" => Some(LIST_ON_CURVE),
        "gvgemoncrvmulti" | "gemoncrvmulti" | "gem on curve multi" => Some(ON_CURVE_MULTI),
        "gvorientgems" | "gvgemonsurface" | "gvgemonsrf" | "gemonsurface" | "gem on surface"
        | "buildergemonsurface" | "om9-gem-011" => Some(ON_SURFACE),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GemBuilderPick {
    RowCurve,
    Surface,
    CuletGuide,
    SurfaceGuide,
    Gem,
    ReferenceGem,
}

impl GemBuilderPick {
    pub(crate) fn prompt(self) -> &'static str {
        match self {
            Self::RowCurve => "Pick the row curve in the viewport",
            Self::Surface => "Pick a NURBS surface in the viewport",
            Self::CuletGuide => "Pick a point, curve or surface to orient the culets",
            Self::SurfaceGuide => "Pick a guide curve on the surface",
            Self::Gem => "Pick a gemstone to use as the pattern",
            Self::ReferenceGem => "Pick a gemstone for the spacing guides",
        }
    }

    pub(crate) fn accepts(self, document: &om9_core::model::Document, entity: &Entity) -> bool {
        match self {
            Self::RowCurve | Self::SurfaceGuide => matches!(entity.geometry, Geometry::Curve(_)),
            Self::Surface => matches!(entity.geometry, Geometry::Surface(_)),
            Self::CuletGuide => matches!(entity.geometry, Geometry::Point(_) | Geometry::Curve(_) | Geometry::Surface(_)),
            Self::Gem | Self::ReferenceGem => super::gem::entity_record(document, entity).is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GemPlacementSession {
    pub tool: GemPlacementTool,
    source_stamp: Option<SourceStamp>,
    guide_stamp: Option<SourceStamp>,
    pub(crate) curve_row: CurveRowOptions,
    pub(crate) builder_pick: Option<GemBuilderPick>,
    pub(crate) builder_styles: Vec<String>,
    pub(crate) edit_row: Option<ObjectId>,
    pub curve_prongs: Vec<om9_core::geometry::Mesh>,
    surface_hover: Option<(om9_scene::camera::Ray, Option<Point3>, Option<GemStation>)>,
    pub(crate) surface_flip: bool,
    pub surface_copy: bool,
    pub surface_snap: bool,
    pub(crate) surface_curve: bool,
    pub surface_spacing: f64,
    pub surface_guides: Vec<(Transform, f64)>,
    pub material: String,
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
        let mut selected_gem = None;
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
                if selected_gem.is_none() {
                    selected_gem = Some(record.clone());
                }
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
        let diameter = match tool {
            GemPlacementTool::OnCurve => 1.5,
            GemPlacementTool::OnSurface => 1.3,
            _ => INITIAL_DIAMETER_MM,
        };
        let mut material = DEFAULT_MATERIAL.to_owned();
        let spec = match selected_gem {
            Some(gem) if matches!(tool, GemPlacementTool::OnCurve | GemPlacementTool::OnSurface) => {
                material = gem.material.clone();
                gem.spec
            }
            _ => match gems.first() {
                Some(gem) => gem.record.spec.clone(),
                None => GemSpec::round(diameter, diameter * 0.65)?,
            },
        };
        let mut session = Self {
            tool,
            source_stamp: None,
            guide_stamp: None,
            curve_row: CurveRowOptions::default(),
            builder_pick: None,
            builder_styles: if tool == GemPlacementTool::OnCurve {
                om9_core::gem_edits::GemStyleStore::load_or_empty(&super::gem_edits::gem_style_path())
                    .map(|store| store.styles.into_iter().map(|style| style.name).collect()).unwrap_or_default()
            } else { Vec::new() },
            edit_row: None,
            curve_prongs: Vec::new(),
            surface_hover: None,
            surface_flip: false,
            surface_copy: true,
            surface_snap: false,
            surface_curve: false,
            surface_spacing: 0.15,
            surface_guides: Vec::new(),
            material,
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
            grid: 0,
            count: None,
            gap_mm: 1.0,
            allowance_mm: 0.0,
            diameters: Vec::new(),
            fit: None,
        };
        // The selection contract per tool: the wrong input is a typed refusal before a
        // session exists.
        session.claim_selection()?;
        session.update_source_stamp(controller)?;
        session.recompute(controller)?;
        Ok(session)
    }

    fn claim_selection(&mut self) -> Result<()> {
        match self.tool {
            GemPlacementTool::OnCurve => {
                if self.curves.is_empty() && !self.surfaces.is_empty() {
                    return Err(Error::Invalid("gem on curve needs one curve"));
                }
            }
            GemPlacementTool::OnCurveAdvanced
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
                if self.surfaces.len() > 1 {
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
        self.curve_prongs.clear();
        if self.tool != GemPlacementTool::OnSurface || self.grid > 0 {
            self.surface_stations.clear();
        }
        self.surface_hover = None;
        self.guides = None;
        self.updates.clear();
        self.conflicts.clear();
        self.fit = None;
        let limits = *controller.coordinator.limits();
        match self.tool {
            GemPlacementTool::OnCurve => {
                if let Some((id, curve)) = self.curves.first().cloned() {
                    let total = om9_core::curve_ops::arc_length(&curve, &limits)?;
                    let mut params = self.params;
                    params.start_mm = self.curve_row.start / 100.0 * total;
                    params.end_mm = Some(if self.curve_row.mirror {
                        (2.0 * self.curve_row.end - self.curve_row.start).min(100.0)
                    } else { self.curve_row.end } / 100.0 * total);
                    if let Some(step) = self.curve_row.center_step {
                        params.spacing_mm = step;
                    }
                    let mut plan = if let Some(count) = self.curve_row.count {
                        let mut setup = self.clone();
                        setup.params = params;
                        setup.count = Some(count);
                        let plan = place_on_curve(&curve, &self.spec, &setup.params_with_count(&curve, &limits)?, &limits)?;
                        if plan.count() != count {
                            return Err(Error::Invalid("gem count placement"));
                        }
                        plan
                    } else if self.curve_row.center_step.is_some() {
                        place_on_curve(&curve, &self.spec, &params, &limits)?
                    } else {
                        fit_gem_row(&curve, &self.spec, self.end_spec.as_ref(), &params,
                            self.curve_row.gap, self.curve_row.fit, self.curve_row.taper_gap, &limits)?
                    };
                    if self.curve_row.snap_corners && let Curve::Polycurve { segments, .. } = &curve {
                        let mut used = std::collections::BTreeSet::new();
                        for i in 1..segments.len() {
                            let fraction = i as f64 / segments.len() as f64;
                            let arc = om9_core::curve_ops::length_between(&curve, 0.0, fraction, &limits)?;
                            if arc < params.start_mm || arc > params.end_mm.unwrap_or(total) { continue; }
                            let nearest = plan.stations.iter_mut()
                                .filter(|station| !used.contains(&station.index))
                                .min_by(|left, right| (left.arc_length_mm - arc).abs()
                                    .total_cmp(&(right.arc_length_mm - arc).abs()));
                            if let Some(station) = nearest {
                                let index = station.index;
                                *station = station_at_arc(&curve, arc, &station.spec, &params, &limits)?;
                                station.index = index;
                                used.insert(index);
                            }
                        }
                        plan.stations.sort_by(|left, right| left.arc_length_mm.total_cmp(&right.arc_length_mm));
                        for (index, station) in plan.stations.iter_mut().enumerate() {
                            station.index = index;
                        }
                    }
                    if self.curve_row.toward_origin || self.curve_row.tops || self.curve_row.flipped
                        || self.curve_row.guide.is_some()
                    {
                        for station in &mut plan.stations {
                            let point = om9_core::curve_ops::point_at(&curve, station.fraction, &limits)?;
                            let axis = if let Some(id) = self.curve_row.guide {
                                let guide = controller.document().entities.get(&id).ok_or(Error::Stale("gem orientation guide"))?;
                                let target = match &guide.geometry {
                                    Geometry::Point(target) => guide.placement.point(*target)?,
                                    Geometry::Curve(curve) => om9_core::curve_surface_ops::closest_point_on_placed_curve(
                                        curve, guide.placement, point, &limits)?.1,
                                    Geometry::Surface(surface) => om9_core::curve_surface_ops::closest_point_on_surface(
                                        &bake_surface(surface, guide.placement, &limits)?, point, &limits)?.point,
                                    _ => return Err(Error::Invalid("gem orientation guide")),
                                };
                                std::array::from_fn(|index| point[index] - target[index])
                            } else if self.curve_row.toward_origin && point != [0.0; 3] {
                                point
                            } else { [0., 0., 1.] };
                            let sign = if self.curve_row.flipped { -1.0 } else { 1.0 };
                            let mut station_params = params;
                            station_params.orientation = Orientation::Axis(axis.map(|value| value * sign));
                            if self.curve_row.tops {
                                station_params.z_offset_mm -= station.spec.crown_height_mm;
                            }
                            let index = station.index;
                            *station = station_at_arc(&curve, station.arc_length_mm, &station.spec, &station_params, &limits)?;
                            station.index = index;
                        }
                    }
                    if self.curve_row.show_prongs {
                        use om9_core::settings::{gems::{GemRow, GemStone}, prong::{ProngParams, build_prong_set}};
                        let stones = plan.stations.iter().map(|station| GemStone::round(
                            station.index.to_string(), station.point, station.spec.width_mm, station.axis))
                            .collect::<Result<Vec<_>>>()?;
                        let row = if plan.closed && plan.range == [0.0, plan.total_length_mm] {
                            GemRow::closed(stones)?
                        } else { GemRow::open(stones)? };
                        self.curve_prongs = build_prong_set(&row, &ProngParams::default())?.bodies;
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
                if self.grid > 0 && let Some((id, surface)) = &self.surface {
                    let entity = controller.document().entities.get(id).ok_or(Error::Stale("gem surface"))?;
                    let mut uv = self.uv_grid(surface);
                    if let Some(trim) = om9_core::rhino_semantics::trimmed_surface_of(entity, &limits)? {
                        uv.retain(|point| om9_core::rhino_semantics::classify_trim_uv(&trim, *point).ok() == Some(Some(true)));
                    }
                    if uv.is_empty() {
                        return Ok(());
                    }
                    let params = SurfaceParams {
                        uv,
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

    fn update_source_stamp(&mut self, controller: &Controller) -> Result<()> {
        let source = match self.tool {
            GemPlacementTool::OnCurve => self.curves.first().map(|(id, _)| *id),
            GemPlacementTool::OnSurface => self.surface.as_ref().map(|(id, _)| *id),
            _ => None,
        };
        self.source_stamp = source.map(|id| controller.document().stamp(id)).transpose()?;
        self.update_guide_stamp(controller)
    }

    fn update_guide_stamp(&mut self, controller: &Controller) -> Result<()> {
        let guide = match self.tool {
            GemPlacementTool::OnCurve => self.curve_row.guide,
            GemPlacementTool::OnSurface if self.surface_curve => self.curves.first().map(|(id, _)| *id),
            _ => None,
        };
        self.guide_stamp = guide.map(|id| controller.document().stamp(id)).transpose()?;
        Ok(())
    }

    fn check_source(&self, controller: &Controller) -> Result<()> {
        for stamp in [self.source_stamp, self.guide_stamp].into_iter().flatten() {
            if controller.document().stamp(stamp.object).ok() != Some(stamp) {
                return Err(Error::Stale("gem placement source changed"));
            }
        }
        Ok(())
    }

    pub fn previews_spheres(&self) -> bool {
        self.tool == GemPlacementTool::OnCurve && self.curve_row.sphere
    }

    pub fn preview_stations(&self) -> impl Iterator<Item = &GemStation> {
        self.plans.iter().flat_map(|plan| &plan.stations)
            .chain(&self.surface_stations)
            .chain(self.surface_hover.iter().filter_map(|(_, _, station)| station.as_ref()))
    }

    fn uv_grid(&self, surface: &NurbsSurface) -> Vec<[f64; 2]> {
        let count = self.grid.clamp(1, 10);
        if count == 1 {
            return vec![self.surface_params.uv[0]];
        }
        let mut uvs = Vec::new();
        for i in 0..count {
            for j in 0..count {
                let u = (i as f64 + 1.0) / (count as f64 + 1.0);
                let v = (j as f64 + 1.0) / (count as f64 + 1.0);
                uvs.push(std::array::from_fn(|axis| {
                    let [min, max] = surface.domain[axis];
                    min + (max - min) * [u, v][axis]
                }));
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
    if session.tool == GemPlacementTool::OnCurve && session.curves.is_empty() {
        return "Gem on Curve: select a curve".to_owned();
    }
    if session.tool == GemPlacementTool::OnSurface {
        if session.surface.is_none() {
            return "Gem on Surface: select a surface".to_owned();
        }
        if session.grid == 0 {
            return format!("Gem on Surface: pick a point ({} placed) (Angle= VerticalOffset= Scale= Spacing= Copy= Snap= Flip= Curve= Gems= Start Reset)", session.surface_stations.len());
        }
    }
    let options = match session.tool {
        GemPlacementTool::OnCurve => "Start= End= Spacing= SpacingMethod= Count= ZOffset= Roll= Size= EndSize= CuletDir= Flip= GemTops= Mirror= TaperSpacing=",
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
        if session.tool == GemPlacementTool::OnCurve { session.curve_row.gap } else { session.params.spacing_mm },
        options
    )
}

/// Whether a tool consumes an option key. Advertised keys are enforced so a typed option is
/// never accepted and silently ignored.
fn option_used(tool: GemPlacementTool, key: &str) -> bool {
    if tool == GemPlacementTool::OnCurve && matches!(key,
        "step" | "spacingmethod" | "layout" | "roll" | "startsize" | "endsize" | "tapergems" |
        "taperspacing" | "mirror" | "gemtops" | "culetdir" | "orient" | "objecttype" |
        "snaptocorners" | "showprongs" | "material" | "style")
    {
        return true;
    }
    if tool == GemPlacementTool::OnSurface && matches!(key,
        "angle" | "verticaloffset" | "scale" | "spacing" | "copy" | "snap" | "flip" | "curve" |
        "cut" | "shape" | "width" | "material" | "gems")
    {
        return true;
    }
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
    /// The same host-owned recipe is used by menu, F6 and CMD even with Matrix enabled.
    pub(super) fn dispatch_shared_gem(&mut self, input: &str) -> Option<Result<()>> {
        let spelling = input.trim().trim_start_matches('_');
        let phrase = spelling.to_ascii_lowercase();
        let input = match phrase.as_str() {
            "gem loader" => "BuilderGemLoader",
            "gem on curve" => ON_CURVE,
            "gem on surface" => ON_SURFACE,
            _ => input,
        };
        let mut tokens = input.split_whitespace();
        let command = tokens.next()?.trim_start_matches('_');
        if command.eq_ignore_ascii_case(EDIT_CURVE_ROW) {
            return Some(self.edit_curve_row());
        }
        let tool = match command.to_ascii_lowercase().as_str() {
            "gem" | "gvloader" | "gemloader" | "loadgem" | "gvbuildergemloader"
            | "buildergemloader" | "om9-gem-001" => None,
            _ => Some(command_name(command).filter(|feature| matches!(*feature, ON_CURVE | ON_SURFACE))?),
        };
        Some((|| {
            if self.is_running() {
                return Err(Error::Busy);
            }
            match tool {
                Some(feature) => self.dispatch_gem_placement(feature)?,
                None => self.dispatch_gem(super::gem::LOADER)?,
            }
            for token in tokens {
                let token = token.trim_start_matches('_');
                let option = token.split_once('=').map_or_else(
                    || token.to_owned(),
                    |(key, value)| format!("{key}={}", value.trim_start_matches('_')),
                );
                let result = match tool {
                    Some(_) => self.gem_placement_text(&option),
                    None => self.gem_text(&option),
                };
                if let Err(error) = result.unwrap_or(Err(Error::Invalid("gem macro option"))) {
                    self.gem_placement_cancel();
                    self.gem_cancel();
                    return Err(error);
                }
            }
            self.invoked_feature = self.is_running().then(|| tool.unwrap_or(super::gem::LOADER).to_owned());
            Ok(())
        })())
    }

    fn edit_curve_row(&mut self) -> Result<()> {
        if self.is_running() { return Err(Error::Busy); }
        let selection = self.selected_ordered();
        let document = self.document();
        let mut recipes = selection.iter().filter_map(|id| document.entities.get(id).and_then(curve_row_recipe));
        let recipe = recipes.next().ok_or(Error::Missing("select a Gem on Curve row"))?;
        if recipes.any(|item| item.id != recipe.id) {
            return Err(Error::Invalid("select stones from one curve row"));
        }
        let entity = document.entities.get(&recipe.source).ok_or(Error::Missing("gem curve source"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Invalid("gem curve source")); };
        let curve = bake_curve(curve, entity.placement, self.coordinator.limits())?;
        let mut session = GemPlacementSession::open(GemPlacementTool::OnCurve, self)?;
        session.curves = vec![(recipe.source, curve)];
        session.edit_row = Some(recipe.id);
        session.curve_row = recipe.options;
        session.spec = recipe.spec;
        session.end_spec = recipe.end_spec;
        session.material = recipe.material;
        session.params.z_offset_mm = recipe.z_offset;
        session.params.rotation_deg = recipe.roll;
        session.update_source_stamp(self)?;
        session.recompute(self)?;
        self.gem_placement = Some(session);
        self.events.emit(crate::command_events::EventKind::Started, EDIT_CURVE_ROW, "Edit Gem on Curve", None);
        Ok(())
    }

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

    pub fn gem_surface_set_gem(&mut self, spec: GemSpec, material: String) {
        if let Some(session) = self.gem_placement.as_mut() {
            session.spec = spec;
            session.material = material;
            session.surface_hover = None;
        }
    }

    pub(crate) fn gem_builder_pick(&self) -> Option<GemBuilderPick> {
        let session = self.gem_placement.as_ref()?;
        session.builder_pick.or_else(|| match self.gem_placement_needs_target() {
            Some(GemPlacementTool::OnCurve) => Some(GemBuilderPick::RowCurve),
            Some(GemPlacementTool::OnSurface) => Some(GemBuilderPick::Surface),
            _ => None,
        })
    }

    pub(crate) fn request_gem_builder_pick(&mut self, pick: GemBuilderPick) -> Result<()> {
        let session = self.gem_placement.as_mut().ok_or(Error::Missing("gem placement builder"))?;
        let compatible = match (session.tool, pick) {
            (GemPlacementTool::OnCurve, GemBuilderPick::RowCurve) => session.edit_row.is_none(),
            (GemPlacementTool::OnCurve, GemBuilderPick::CuletGuide | GemBuilderPick::Gem) => true,
            (GemPlacementTool::OnSurface, GemBuilderPick::Surface) => session.surface_stations.is_empty(),
            (GemPlacementTool::OnSurface, GemBuilderPick::SurfaceGuide | GemBuilderPick::Gem |
                GemBuilderPick::ReferenceGem) => session.surface.is_some(),
            _ => false,
        };
        if !compatible {
            return Err(Error::Invalid("gem builder pick at this step"));
        }
        session.builder_pick = Some(pick);
        session.surface_hover = None;
        Ok(())
    }

    pub(crate) fn cancel_gem_builder_pick(&mut self) {
        if let Some(session) = self.gem_placement.as_mut() {
            session.builder_pick = None;
        }
    }

    pub fn gem_placement_needs_target(&self) -> Option<GemPlacementTool> {
        let session = self.gem_placement.as_ref()?;
        match session.tool {
            GemPlacementTool::OnCurve if session.curves.is_empty() => Some(session.tool),
            GemPlacementTool::OnSurface if session.surface.is_none() => Some(session.tool),
            _ => None,
        }
    }

    pub fn gem_placement_pick_target(&mut self, id: ObjectId) -> Result<()> {
        let pick = self.gem_builder_pick().ok_or(Error::Invalid("gem placement already has a target"))?;
        let document = self.document();
        let entity = document.entities.get(&id).ok_or(Error::Missing("gem target"))?;
        if !pick.accepts(document, entity) {
            return Err(Error::Invalid("object does not match the gem builder input"));
        }
        let limits = self.coordinator.limits();
        let mut session = self.gem_placement.clone().expect("active gem command");
        match pick {
            GemBuilderPick::RowCurve | GemBuilderPick::SurfaceGuide => {
                let Geometry::Curve(curve) = &entity.geometry else { unreachable!() };
                session.curves = vec![(id, bake_curve(curve, entity.placement, limits)?)];
                if pick == GemBuilderPick::SurfaceGuide {
                    session.surface_curve = true;
                }
            }
            GemBuilderPick::Surface => {
                let Geometry::Surface(surface) = &entity.geometry else { unreachable!() };
                session.surface = Some((id, bake_surface(surface, entity.placement, limits)?));
            }
            GemBuilderPick::CuletGuide => {
                if session.curves.first().is_some_and(|(source, _)| *source == id) {
                    return Err(Error::Invalid("choose a guide different from the row curve"));
                }
                session.curve_row.guide = Some(id);
                session.curve_row.toward_origin = false;
            }
            GemBuilderPick::Gem => {
                let record = super::gem::entity_record(document, entity).ok_or(Error::Missing("gem pattern"))?;
                session.spec = record.spec;
                session.material = record.material;
                session.curve_row.sphere = false;
            }
            GemBuilderPick::ReferenceGem => {
                let record = super::gem::entity_record(document, entity).ok_or(Error::Missing("reference gem"))?;
                let guide = (entity.placement, record.spec.length_mm);
                if !session.surface_guides.contains(&guide) {
                    session.surface_guides.push(guide);
                }
            }
        }
        session.builder_pick = None;
        session.update_source_stamp(self)?;
        session.recompute(self)?;
        self.gem_placement = Some(session);
        Ok(())
    }

    pub fn gem_surface_hover(&mut self, ray: Option<om9_scene::camera::Ray>, snap: Option<Point3>) {
        let Some(session) = self.gem_placement.as_ref() else { return; };
        if let Some(ray) = ray && session.surface_hover.as_ref()
            .is_some_and(|(current_ray, current_snap, station)| station.is_some() && *current_ray == ray && *current_snap == snap)
            && session.check_source(self).is_ok()
        {
            return;
        }
        let hover = ray.map(|ray| (ray, snap, self.gem_surface_station(ray, snap).ok()));
        if let Some(session) = self.gem_placement.as_mut() {
            session.surface_hover = hover;
        }
    }

    pub fn gem_surface_manual(&self) -> bool {
        self.gem_placement.as_ref().is_some_and(|session|
            session.tool == GemPlacementTool::OnSurface && session.surface.is_some()
                && session.builder_pick.is_none() && session.grid == 0)
    }

    pub fn gem_surface_click(&mut self, ray: om9_scene::camera::Ray, snap: Option<Point3>) -> Result<()> {
        let station = self.gem_surface_station(ray, snap)?;
        let limit = self.coordinator.limits().max_outputs;
        let session = self.gem_placement.as_mut().expect("active gem placement");
        if session.surface_stations.len() as u64 >= limit {
            return Err(Error::Budget("gem surface placement"));
        }
        session.surface_stations.push(station);
        if !session.surface_copy && let Err(error) = self.gem_placement_apply() {
            if let Some(session) = self.gem_placement.as_mut() {
                session.surface_stations.pop();
            }
            return Err(error);
        }
        Ok(())
    }

    #[cfg(not(feature = "occt-brep"))]
    fn gem_surface_station(&self, _: om9_scene::camera::Ray, _: Option<Point3>) -> Result<GemStation> {
        Err(Error::Unsupported("surface ray placement requires OCCT"))
    }

    #[cfg(feature = "occt-brep")]
    fn gem_surface_station(&self, ray: om9_scene::camera::Ray, snap: Option<Point3>) -> Result<GemStation> {
        use om9_core::curve_surface_ops::{closest_point_on_curve, closest_point_on_surface};
        use om9_core::rhino_semantics::{classify_trim_uv, trimmed_surface_of};

        let session = self.gem_placement.as_ref().ok_or(Error::Missing("gem surface command"))?;
        session.check_source(self)?;
        let (id, surface) = session.surface.as_ref().ok_or(Error::Missing("gem surface"))?;
        let document = self.document();
        let entity = document.entities.get(id).ok_or(Error::Stale("gem surface"))?;
        let limits = self.coordinator.limits();
        let tolerance = document.effective_tolerance()?.absolute_mm;
        let _permit = self.coordinator.external_memory_account().try_acquire_surface_snap()?;
        let trim = trimmed_surface_of(entity, limits)?;
        let inside = |uv| -> Result<bool> {
            Ok(match &trim {
                Some(trim) => classify_trim_uv(trim, uv)? == Some(true),
                None => true,
            })
        };
        let mut hits = om9_kernel::measurement::surface_ray_hits_uv(
            &entity.geometry, entity.placement, ray.origin, ray.direction, tolerance, limits,
        ).map_err(|_| Error::Invalid("gem surface ray"))?;
        hits.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        let mut source = None;
        for hit in hits {
            if inside(hit.uv)? {
                source = Some(hit);
                break;
            }
        }
        let hit = source.ok_or(Error::Missing("point on the selected face"))?;
        let mut uv = hit.uv;
        let mut target = snap.filter(|_| session.surface_snap);
        if session.surface_curve {
            let (_, curve) = session.curves.first().ok_or(Error::Missing("select a guide curve with the surface"))?;
            target = Some(closest_point_on_curve(curve, target.unwrap_or(hit.point), limits)?.1);
        }
        if let Some(target) = target {
            let projected = closest_point_on_surface(surface, target, limits)?;
            if projected.distance > tolerance || !inside(projected.parameters)? {
                return Err(Error::Invalid("snap or guide is off the selected face"));
            }
            uv = projected.parameters;
        }
        let reversed = trim.as_ref().is_some_and(|trim| trim.reversed);
        let flip_basis = reversed ^ session.surface_flip;
        let offset = session.surface_params.z_offset_mm * if reversed { -1.0 } else { 1.0 };
        let rotation = session.surface_params.rotation_deg * if flip_basis { -1.0 } else { 1.0 };
        let mut station = place_on_surface(surface, std::slice::from_ref(&session.spec),
            &SurfaceParams { uv: vec![uv], z_offset_mm: offset, rotation_deg: rotation },
            limits)?.stations.remove(0);
        if flip_basis {
            for row in &mut station.transform.0[..3] {
                row[1] = -row[1];
                row[2] = -row[2];
            }
            station.axis = station.axis.map(|component| -component);
        }
        station.z_offset_mm = session.surface_params.z_offset_mm;
        station.spin_deg = session.surface_params.rotation_deg;
        Ok(station)
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
        if self.gem_surface_manual() && matches!(lower.as_str(), "start" | "reset" | "undo" | "u" | "a" | "d" | "w" | "s" | "q" | "e") {
            let session = self.gem_placement.as_mut().expect("active gem placement");
            session.surface_hover = None;
            match lower.as_str() {
                "start" => session.surface_hover = None,
                "reset" => {
                    session.surface_params = SurfaceParams::new(vec![[0.5, 0.5]]);
                    session.surface_spacing = 0.15;
                    session.surface_flip = false;
                    session.surface_copy = true;
                    session.surface_snap = false;
                    session.surface_curve = false;
                    session.surface_guides.clear();
                    session.guide_stamp = None;
                    session.surface_hover = None;
                }
                "u" | "undo" => { session.surface_stations.pop(); }
                "a" => session.surface_params.rotation_deg -= 5.0,
                "d" => session.surface_params.rotation_deg += 5.0,
                "w" => session.surface_params.z_offset_mm += 0.1,
                "s" => session.surface_params.z_offset_mm -= 0.1,
                "q" | "e" => {
                    return Some(scale_gem_spec(&session.spec, if lower == "q" { 0.95 } else { 1.05 })
                        .map(|spec| session.spec = spec));
                }
                _ => unreachable!(),
            }
            return Some(Ok(()));
        }
        Some(self.gem_placement_option(trimmed))
    }

    pub fn gem_placement_prompt(&self) -> Option<String> {
        let session = self.gem_placement.as_ref()?;
        Some(match session.builder_pick {
            Some(pick) => pick.prompt().to_owned(),
            None => prompt(session),
        })
    }

    fn gem_placement_option(&mut self, text: &str) -> Result<()> {
        let mut session = self
            .gem_placement
            .clone()
            .ok_or(Error::Missing("gem placement session"))?;
        session.check_source(self)?;
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
            let number = value.trim_end_matches("mm").trim_end_matches('%').trim()
                .parse::<f64>().map_err(|_| Error::Invalid("gem placement option"))?;
            if !number.is_finite() { return Err(Error::Invalid("gem placement option")); }
            Ok(number)
        };
        let flag = || match value.to_ascii_lowercase().as_str() {
            "yes" | "true" | "1" | "on" => Ok(true),
            "no" | "false" | "0" | "off" => Ok(false),
            _ => Err(Error::Invalid("gem option flag")),
        };
        let pct = || {
            let fraction = number(value)?;
            if !(0.0..=100.0).contains(&fraction) { return Err(Error::Invalid("gem curve percent")); }
            Ok(fraction)
        };
        match key.as_str() {
            "start" if session.tool == GemPlacementTool::OnCurve => session.curve_row.start = pct()?,
            "end" if session.tool == GemPlacementTool::OnCurve => session.curve_row.end = pct()?,
            "spacing" if session.tool == GemPlacementTool::OnCurve => {
                let gap = number(value)?;
                if gap < 0.0 { return Err(Error::Invalid("gem clearance")); }
                session.curve_row.gap = gap;
                session.curve_row.center_step = None;
            }
            "spacing" if session.tool == GemPlacementTool::OnSurface => {
                let gap = number(value)?;
                if gap < 0.0 { return Err(Error::Invalid("gem spacing guide")); }
                session.surface_spacing = gap;
            }
            "step" => session.curve_row.center_step = Some(number(value)?),
            "spacingmethod" | "layout" => {
                session.curve_row.fit = match value.to_ascii_lowercase().as_str() {
                    "spacing" | "fit" => RowFit::Spacing,
                    "sizegems" | "size" => RowFit::SizeGems,
                    "fixed" | "fixed_gap" => RowFit::Fixed,
                    _ => return Err(Error::Invalid("gem curve spacing method")),
                };
                session.curve_row.center_step = None;
            }
            "mirror" => session.curve_row.mirror = flag()?,
            "gemtops" => session.curve_row.tops = flag()?,
            "snaptocorners" => session.curve_row.snap_corners = flag()?,
            "showprongs" => session.curve_row.show_prongs = flag()?,
            "taperspacing" => session.curve_row.taper_gap = flag()?,
            "tapergems" => {
                session.end_spec = flag()?.then(|| scale_gem_spec(&session.spec, 0.6)).transpose()?;
            }
            "culetdir" | "orient" => {
                session.curve_row.guide = None;
                session.curve_row.toward_origin = match value.to_ascii_lowercase().as_str() {
                    "f4" | "outward" => true,
                    "down" | "plane" => false,
                    "object" => {
                        let host = session.curves.first().map(|(id, _)| *id);
                        session.curve_row.guide = Some(self.selected_ordered().into_iter()
                            .find(|id| Some(*id) != host && self.document().entities.get(id)
                                .is_some_and(|entity| matches!(entity.geometry,
                                    Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Point(_))))
                            .ok_or(Error::Missing("select an orientation curve, surface or point"))?);
                        false
                    }
                    _ => return Err(Error::Invalid("select a supported culet direction")),
                };
            }
            "objecttype" => match value.to_ascii_lowercase().as_str() {
                "roundgem" | "sphere" => {
                    session.spec = GemSpec::round(session.spec.length_mm, session.spec.depth_mm())?;
                    session.curve_row.sphere = value.eq_ignore_ascii_case("sphere");
                }
                "selectgem" => {
                    let record = self.selected_ordered().iter().find_map(|id|
                        self.document().entities.get(id).and_then(|entity|
                            super::gem::entity_record(self.document(), entity)))
                        .ok_or(Error::Missing("select a gemstone"))?;
                    session.spec = record.spec;
                    session.material = record.material;
                    session.curve_row.sphere = false;
                }
                _ => return Err(Error::Unsupported("Gem on Curve object type")),
            },
            "style" => {
                let store = om9_core::gem_edits::GemStyleStore::load_or_empty(&super::gem_edits::gem_style_path())?;
                let style = store.get(value).ok_or(Error::Missing("gem style"))?;
                let styled = style.apply(&GemRecord::new(session.spec.clone()))?;
                session.spec = styled.spec;
                session.material = styled.material;
            }
            "copy" => session.surface_copy = flag()?,
            "snap" => session.surface_snap = flag()?,
            "curve" => {
                session.surface_curve = flag()?;
                if session.surface_curve && session.curves.is_empty() {
                    return Err(Error::Missing("select a guide curve with the surface"));
                }
            }
            "gems" => {
                session.surface_guides.clear();
                if flag()? {
                    session.surface_guides.extend(self.selected_ordered().iter().filter_map(|id| {
                        let document = self.document();
                        let entity = document.entities.get(id)?;
                        let record = super::gem::entity_record(document, entity)?;
                        Some((entity.placement, record.spec.length_mm))
                    }));
                    if session.surface_guides.is_empty() {
                        return Err(Error::Missing("select reference stones"));
                    }
                }
            }
            "material" => {
                if super::gem::material_color(value).is_none() { return Err(Error::Invalid("gem material")); }
                session.material = value.to_ascii_lowercase();
            }
            "shape" | "cut" => {
                let (cut, shape) = if key == "shape" {
                    (session.spec.cut, GemShape::from_tag(value)?)
                } else {
                    super::gem::loader_cut(value, session.spec.cut, session.spec.shape)?
                };
                let size = session.spec.length_mm;
                session.spec = GemSpec::from_outer_dimensions(shape, cut, size, size / shape.default_aspect(), session.spec.depth_mm())?;
            }
            "width" => {
                let width = number(value)?;
                let length = session.spec.length_mm;
                let width = if width == 0.0 { length / session.spec.shape.default_aspect() } else { width };
                session.spec = session.spec.clone().set_girdle_mm(length, width)?;
            }
            "start" => session.params.start_mm = number(value)?,
            "end" => {
                if value.eq_ignore_ascii_case("none") {
                    session.params.end_mm = None;
                } else {
                    session.params.end_mm = Some(number(value)?);
                }
            }
            "spacing" => session.params.spacing_mm = number(value)?,
            "count" if session.tool == GemPlacementTool::OnCurve && value.eq_ignore_ascii_case("auto") => {
                session.curve_row.count = None;
            }
            "count" => {
                let count = number(value)?;
                if !(count.is_finite() && (1.0..=1_000.0).contains(&count)) {
                    return Err(Error::Invalid("gem count"));
                }
                if session.tool == GemPlacementTool::OnCurve {
                    session.curve_row.count = Some(count.round() as usize);
                } else {
                    session.count = Some(count.round() as usize);
                }
            }
            "zoffset" | "verticaloffset" => {
                let value = number(value)?;
                match session.tool {
                    GemPlacementTool::Splop => session.splop.z_offset_mm = value,
                    GemPlacementTool::OnSurface => session.surface_params.z_offset_mm = value,
                    _ => session.params.z_offset_mm = value,
                }
            }
            "rotate" | "rotation" | "roll" | "angle" => {
                let value = number(value)?;
                match session.tool {
                    GemPlacementTool::Splop => session.splop.rotation_deg = value,
                    GemPlacementTool::OnSurface => session.surface_params.rotation_deg = value,
                    _ => session.params.rotation_deg = value,
                }
            }
            "size" | "scale" | "startsize" => {
                let diameter = number(value)?;
                session.spec = if matches!(session.tool, GemPlacementTool::OnCurve | GemPlacementTool::OnSurface) {
                    scale_gem_spec(&session.spec, diameter / session.spec.length_mm)?
                } else {
                    GemSpec::round(diameter, (diameter * 0.65).max(0.1))?
                };
            }
            "endsize" => {
                let diameter = number(value)?;
                session.end_spec = Some(if session.tool == GemPlacementTool::OnCurve {
                    scale_gem_spec(&session.spec, diameter / session.spec.length_mm)?
                } else {
                    GemSpec::round(diameter, (diameter * 0.65).max(0.1))?
                });
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
            "u" | "v" => {
                session.grid = 1;
                session.surface_params.uv[0][usize::from(key == "v")] = number(value)?;
            }
            "grid" => {
                if !session.surface_stations.is_empty() && session.grid == 0 {
                    return Err(Error::Invalid("finish the manual surface placement first"));
                }
                session.grid = number(value)?.clamp(0.0, 10.0) as usize;
                session.surface_stations.clear();
            }
            "flip" if session.tool == GemPlacementTool::OnSurface => session.surface_flip = flag()?,
            "flip" if session.tool == GemPlacementTool::OnCurve => session.curve_row.flipped = flag()?,
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
        session.update_guide_stamp(self)?;
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
        if session.builder_pick.is_some() {
            return Err(Error::Missing("complete the gem builder pick"));
        }
        session.check_source(self)?;
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
    if session.previews_spheres() || !session.curve_prongs.is_empty() {
        controller.ensure_layer_writable(layer)?;
    }
    let mut mutations = Vec::new();
    let row_id = session.edit_row.unwrap_or_else(ObjectId::new);
    let mut old_row = std::collections::BTreeMap::new();
    let mut old_prongs = Vec::new();
    if let Some(id) = session.edit_row {
        for entity in controller.document().entities.values() {
            if let Some(recipe) = curve_row_recipe(entity) && recipe.id == id {
                if recipe.prong {
                    old_prongs.push(entity.id);
                    continue;
                }
                if old_row.insert(recipe.index, entity.id).is_some() {
                    return Err(Error::Invalid("duplicate Gem on Curve row index"));
                }
            }
        }
        if old_row.is_empty() { return Err(Error::Stale("gem curve row")); }
    }
    for plan in &session.plans {
        let source = plan.curve.ok_or(Error::Missing("gem placement source"))?;
        let revision = revision_of(controller, source);
        for station in &plan.stations {
            let existing = old_row.remove(&station.index);
            let mut entity = gem_entity_for(station, layer, controller, &session.material)?;
            let mut record = gem_record(&entity).ok_or(Error::Invalid("gem record"))?;
            if session.previews_spheres() {
                entity.geometry = Geometry::Mesh(om9_core::settings::sphere([0.; 3], station.spec.length_mm, 24)?);
                entity.label = "Gem on Curve sphere".to_owned();
            }
            if let Some(id) = existing {
                let old = &controller.document().entities[&id];
                controller.ensure_layer_writable(old.attributes.layer)?;
                record.gem_id = gem_record(old).map_or(GemId(id), |stone| stone.gem_id);
                entity.extensions = old.extensions.clone();
                entity.extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
            }
            if session.previews_spheres() {
                entity.extensions.remove(GEM_EXTENSION_SLOT);
            }
            let link = PlacementLink::new(
                record.gem_id,
                PlacementSource::Curve { id: source },
                station.arc_length_mm,
                station.fraction,
                revision,
                plan.params.spacing_mm.max(1.0e-6),
                station.z_offset_mm,
            )?;
            entity.extensions.insert(GEM_PLACEMENT_EXTENSION_SLOT.to_owned(), link.to_extension()?);
            if session.tool == GemPlacementTool::OnCurve {
                let recipe = CurveRowRecipe {
                    id: row_id, source, index: station.index, prong: false, options: session.curve_row.clone(),
                    spec: session.spec.clone(), end_spec: session.end_spec.clone(),
                    z_offset: session.params.z_offset_mm, roll: session.params.rotation_deg,
                    material: session.material.clone(),
                };
                entity.extensions.insert(CURVE_ROW_SLOT.to_owned(), Extension::new(1,
                    serde_json::to_string(&recipe).map_err(|_| Error::Invalid("gem row recipe"))?)?);
            }
            if let Some(id) = existing {
                let source = controller.document().stamp(id)?;
                mutations.push(Mutation::Geometry { source, geometry: entity.geometry });
                mutations.push(Mutation::Placement { source, placement: entity.placement });
                mutations.push(Mutation::Extensions { source, extensions: entity.extensions });
            } else {
                mutations.push(Mutation::Insert { document: controller.document, entity });
            }
        }
    }
    if !session.curve_prongs.is_empty() {
        let source = session.curves.first().ok_or(Error::Missing("gem row curve"))?.0;
        for (index, mesh) in session.curve_prongs.iter().enumerate() {
            let recipe = CurveRowRecipe {
                id: row_id, source, index, prong: true, options: session.curve_row.clone(),
                spec: session.spec.clone(), end_spec: session.end_spec.clone(),
                z_offset: session.params.z_offset_mm, roll: session.params.rotation_deg,
                material: session.material.clone(),
            };
            let mut entity = Entity::new("Gem on Curve prong", Geometry::Mesh(mesh.clone()), layer);
            entity.extensions.insert(CURVE_ROW_SLOT.to_owned(), Extension::new(1,
                serde_json::to_string(&recipe).map_err(|_| Error::Invalid("gem row recipe"))?)?);
            mutations.push(Mutation::Insert { document: controller.document, entity });
        }
    }
    for id in old_row.into_values().chain(old_prongs) {
        controller.ensure_layer_writable(controller.document().entities[&id].attributes.layer)?;
        mutations.push(Mutation::Delete { source: controller.document().stamp(id)? });
    }
    let Some((source, _)) = session.surface.as_ref() else {
        if mutations.is_empty() {
            return Err(Error::Missing("gem placement stations"));
        }
        return controller.commit_gem(session.tool.title(), mutations);
    };
    let source = *source;
    let revision = revision_of(controller, source);
    for station in &session.surface_stations {
        let entity = gem_entity_for(station, layer, controller, &session.material)?;
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
    material: &str,
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
        material,
        super::gem::material_color(material).unwrap_or(DEFAULT_COLOR),
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
