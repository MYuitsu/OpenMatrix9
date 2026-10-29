// SPDX-License-Identifier: LGPL-2.1-or-later
//! Halo, cluster and baguette builders (`OM9-L2-04-11`).
//!
//! This module owns the Matrix command surface for the gem-layout family:
//!
//! * `BuilderHalo` / `gvHaloBuilder` — accent ring around one center stone (`-001`).
//! * `BuilderClusterBuilderOld` / `gvClusterBuilder` — edge-gem cluster (`-002`).
//! * `BuilderTaperBagChannel` / `gvTaperBagChannel` — baguette channel (`-003`).
//! * `BuilderTaperBagBetweenTwoCurves` / `gvTaperBagBetweenTwoCurves` — one tapered baguette
//!   between two walls (`-004`).
//! * `BuilderGemBetweenTwoCurves` / `gvGemsBetween2Crvs` — stones touching both curves (`-005`).
//! * `BuilderOrientToGem` / `gvOrientGems` — align a companion object to a gem (`-006`).
//! * every committed layout stores one `om9.gem.cluster` feature, so re-running the builder
//!   edits the existing children instead of stacking a second halo (`-007`).
//!
//! The typed layout owner is `om9_core::gem_cluster`; this module only turns a command or a
//! window edit into one document transaction. The preview is pure session state and Apply is
//! atomic, so Cancel never leaves a phantom gem behind.

use om9_core::{
    Error, Result,
    gem::{GemId, GemInstance, GemRecord, GemSpec, build_mesh, gem_record_in},
    gem_cluster::{
        CLUSTER_EXTENSION_SLOT, ChannelParams, ClusterFeature, ClusterKind, ClusterParams,
        ClusterStart, HaloParams, MAX_ACCENTS, OrientOptions, baguette_channel, cluster_layout,
        gems_between_curves, halo_layout, orient_to_gem, tapered_baguette,
    },
    geometry::{Curve, Geometry, Transform},
    identity::{LayerId, ObjectId},
    model::{Entity, GEM_EXTENSION_SLOT},
    transaction::Mutation,
};

use super::Controller;

/// `Builder Halo` palette icon / `gvHaloBuilder` command.
pub const HALO: &str = "BuilderHalo";
/// `Cluster Builder` palette icon / `gvClusterBuilder` command.
pub const CLUSTER: &str = "BuilderClusterBuilderOld";
/// `Baguette Channel` palette icon / `gvTaperBagChannel` command.
pub const CHANNEL: &str = "BuilderTaperBagChannel";
/// `Tapered Baguette Between Two Curves` palette icon / command.
pub const TAPERED: &str = "BuilderTaperBagBetweenTwoCurves";
/// `Gems Between 2 Curves` palette icon / `gvGemsBetween2Crvs` command.
pub const BETWEEN: &str = "BuilderGemBetweenTwoCurves";
/// `Orient to Gem` palette icon / `gvOrientGems` command.
pub const ORIENT: &str = "BuilderOrientToGem";

/// The builder an open session represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterTool {
    Halo,
    Cluster,
    Channel,
    Tapered,
    Between,
    Orient,
}

impl ClusterTool {
    /// The Matrix command id the window and the command line report.
    pub fn command(self) -> &'static str {
        match self {
            Self::Halo => HALO,
            Self::Cluster => CLUSTER,
            Self::Channel => CHANNEL,
            Self::Tapered => TAPERED,
            Self::Between => BETWEEN,
            Self::Orient => ORIENT,
        }
    }
    /// The Matrix command-line name; the manual quotes these names.
    pub fn typed_name(self) -> &'static str {
        match self {
            Self::Halo => "gvHaloBuilder",
            Self::Cluster => "gvClusterBuilder",
            Self::Channel => "gvTaperBagChannel",
            Self::Tapered => "gvTaperBagBetweenTwoCurves",
            Self::Between => "gvGemsBetween2Crvs",
            Self::Orient => "gvOrientGems",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Halo => "Halo Builder",
            Self::Cluster => "Cluster Builder",
            Self::Channel => "Baguette Channel",
            Self::Tapered => "Tapered Baguette Between Two Curves",
            Self::Between => "Gems Between 2 Curves",
            Self::Orient => "Orient to Gem",
        }
    }
    /// Curve tools take two walls; the two remaining tools take a center gem.
    pub fn uses_curves(self) -> bool {
        matches!(self, Self::Channel | Self::Tapered | Self::Between)
    }
    pub fn uses_center(self) -> bool {
        matches!(self, Self::Halo | Self::Cluster | Self::Orient)
    }
}

/// Whether this module owns the feature id.
pub fn handles(feature: &str) -> bool {
    matches!(
        feature,
        HALO | CLUSTER | CHANNEL | TAPERED | BETWEEN | ORIENT
    )
}

/// The typed command names that reach the same services.
pub fn command_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "gvhalobuilder" | "halobuilder" | "halo builder" | "halo" => Some(HALO),
        "gvclusterbuilder" | "clusterbuilder" | "cluster builder" | "cluster" => Some(CLUSTER),
        "gvtaperbagchannel" | "baguettechannel" | "baguette channel" => Some(CHANNEL),
        "gvtaperbagbetweentwocurves" | "taperbagbetweentwocurves" => Some(TAPERED),
        "gvgemsbetween2crvs" | "gemsbetween2curves" | "gems between 2 curves" => Some(BETWEEN),
        "gvorientgems" | "orienttogem" | "orient to gem" => Some(ORIENT),
        _ => None,
    }
}

/// The typed layout parameters of one session.
#[derive(Debug, Clone)]
pub enum LayoutKind {
    Halo(HaloParams),
    Cluster(ClusterParams),
    Channel(ChannelParams),
}

impl LayoutKind {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Halo(_) => "halo",
            Self::Cluster(_) => "cluster",
            Self::Channel(_) => "channel",
        }
    }
    pub fn to_feature(
        &self,
        center: Option<ObjectId>,
        curves: [Option<ObjectId>; 2],
        accents: Vec<GemId>,
    ) -> ClusterFeature {
        let kind = match self {
            Self::Halo(params) => ClusterKind::Halo {
                params: params.clone(),
            },
            Self::Cluster(params) => ClusterKind::Cluster {
                params: params.clone(),
            },
            Self::Channel(params) => ClusterKind::Channel {
                params: params.clone(),
            },
        };
        let mut feature = ClusterFeature::new(kind, center, curves);
        feature.accents = accents;
        feature
    }
}

/// One open halo/cluster/baguette builder. The session is the whole preview: no document object
/// exists until Apply commits one transaction.
#[derive(Debug, Clone)]
pub struct GemClusterSession {
    pub tool: ClusterTool,
    pub kind: LayoutKind,
    /// The center gem (`-001`, `-002` and `-006`).
    pub center: Option<ObjectId>,
    pub center_instance: Option<GemInstance>,
    /// The two wall curves in selection order.
    pub curves: Vec<(ObjectId, Curve)>,
    /// The current preview accents; this is never written to the document before Apply.
    pub accents: Vec<GemInstance>,
    /// The entity that stores the `om9.gem.cluster` feature: the center gem or the first curve.
    pub host: Option<ObjectId>,
    /// The existing feature when the builder was opened for an edit (`-007`).
    pub edit: Option<ClusterFeature>,
    /// The object a `-006` session orients.
    pub orient_object: Option<ObjectId>,
    pub orient_options: OrientOptions,
    /// Keep the kernel B-Rep body when the kernel is compiled in.
    pub brep: bool,
    pub status: Option<String>,
}

impl GemClusterSession {
    pub fn preview_count(&self) -> usize {
        if self.tool == ClusterTool::Orient {
            return usize::from(self.orient_object.is_some());
        }
        self.accents.len()
    }
    pub fn summary(&self) -> String {
        match &self.kind {
            LayoutKind::Halo(params) => format!(
                "Halo · {} accents · {:.2} mm gap · {:.2} mm from center · {:.1}°",
                self.accents.len(),
                params.spacing_mm,
                params.spacing_from_center_mm,
                params.angle_deg
            ),
            LayoutKind::Cluster(params) => format!(
                "Cluster · {} accents · XY {:.2} · Z {:.2} mm · {:.1}°",
                self.accents.len(),
                params.xy_offset_mm,
                params.z_offset_mm,
                params.degree_deg
            ),
            LayoutKind::Channel(params) => format!(
                "{} · {} stones · gap {:.2} mm · taper {:.0}%",
                if self.tool == ClusterTool::Between {
                    "Gems between curves"
                } else if self.tool == ClusterTool::Tapered {
                    "Tapered baguette"
                } else {
                    "Baguette channel"
                },
                self.accents.len(),
                params.spacing_mm,
                params.taper * 100.0
            ),
        }
    }
    pub fn prompt(&self) -> String {
        match &self.kind {
            LayoutKind::Halo(_) => format!(
                "{}: spacing=0.3 angle=0 count=auto | Apply/Cancel",
                self.tool.typed_name()
            ),
            LayoutKind::Cluster(_) => format!(
                "{}: XY=0 Z=0 count=auto start=bottom | Apply/Cancel",
                self.tool.typed_name()
            ),
            LayoutKind::Channel(_) => format!(
                "{}: spacing=0.2 taper=100 banking=0 points=0..100 | Apply/Cancel",
                self.tool.typed_name()
            ),
        }
    }
}

/// The default accent stone: a small round diamond that reads as a melee.
fn default_accent() -> Result<GemSpec> {
    GemSpec::round(1.3, 0.8)
}

/// The default channel section: a 4 x 1.5 mm baguette.
fn default_baguette() -> Result<GemSpec> {
    GemSpec::baguette(4.0, 1.5, 0.0, 1.2, 0.15)
}

fn default_section(tool: ClusterTool) -> Result<GemSpec> {
    match tool {
        ClusterTool::Between => GemSpec::round(1.5, 1.0),
        ClusterTool::Tapered => GemSpec::baguette(5.0, 2.0, 0.0, 1.6, 0.15),
        _ => default_baguette(),
    }
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
            .ok_or(Error::Stale("gem cluster selection"))?;
        if let Geometry::Curve(curve) = &entity.geometry {
            let baked = if entity.placement == Transform::identity() {
                curve.clone()
            } else {
                match om9_core::top11::bake_geometry(
                    &Geometry::Curve(curve.clone()),
                    entity.placement,
                    &limits,
                )? {
                    Geometry::Curve(curve) => curve,
                    _ => return Err(Error::Invalid("gem cluster curve")),
                }
            };
            curves.push((*id, baked));
        }
    }
    Ok(curves)
}

fn selected_gems(controller: &Controller, selection: &[ObjectId]) -> Vec<ObjectId> {
    selection
        .iter()
        .copied()
        .filter(|id| gem_record_in(controller.document(), *id).is_some())
        .collect()
}

fn instance_for(controller: &Controller, id: ObjectId) -> Result<GemInstance> {
    let record = gem_record_in(controller.document(), id)
        .ok_or(Error::Invalid("gem cluster needs a gem"))?;
    let entity = controller
        .document()
        .entities
        .get(&id)
        .ok_or(Error::Stale("gem cluster center"))?;
    Ok(GemInstance {
        spec: record.spec,
        placement: entity.placement,
        label: record.asset,
    })
}

impl GemClusterSession {
    /// Open the builder for `tool` from the current selection.
    ///
    /// A center gem that already carries the matching `om9.gem.cluster` feature opens as an
    /// edit session, so Halo > Edit regenerates the existing stones instead of adding a second
    /// halo (`-007`). Every other selection opens a fresh preview.
    fn open(tool: ClusterTool, controller: &Controller) -> Result<Self> {
        let selection = controller.selected_ordered();
        let mut session = Self {
            tool,
            kind: LayoutKind::Halo(HaloParams::new(default_accent()?)?),
            center: None,
            center_instance: None,
            curves: Vec::new(),
            accents: Vec::new(),
            host: None,
            edit: None,
            orient_object: None,
            orient_options: OrientOptions::default(),
            brep: cfg!(feature = "occt-brep"),
            status: None,
        };
        if tool.uses_center() {
            let gems = selected_gems(controller, &selection);
            let center = match gems.as_slice() {
                [id] => *id,
                [] => return Err(Error::Missing("gem cluster center gem")),
                _ => return Err(Error::Invalid("gem cluster needs one center gem")),
            };
            session.center = Some(center);
            session.center_instance = Some(instance_for(controller, center)?);
            session.host = Some(center);
            if tool == ClusterTool::Orient {
                let object = selection.iter().copied().find(|id| *id != center);
                session.orient_object = object;
                if object.is_none() {
                    return Err(Error::Missing("orient to gem object"));
                }
            } else if let Some(feature) = controller.cluster_feature(center)?
                && tool_matches(tool, &feature)
            {
                session.kind = LayoutKind::from_feature(&feature)?;
                session.edit = Some(feature);
            } else {
                let params = default_accent()?;
                session.kind = match tool {
                    ClusterTool::Halo => LayoutKind::Halo(HaloParams::new(params)?),
                    _ => LayoutKind::Cluster(ClusterParams::new(params)?),
                };
            }
        } else {
            session.curves = selected_curves(controller, &selection)?;
            if session.curves.len() != 2 {
                return Err(Error::Invalid("gem cluster needs two curves"));
            }
            let host = session.curves[0].0;
            session.host = Some(host);
            if let Some(feature) = controller.cluster_feature(host)?
                && tool_matches(tool, &feature)
            {
                session.kind = LayoutKind::from_feature(&feature)?;
                session.edit = Some(feature);
            } else {
                let mut params = ChannelParams::new(default_section(tool)?)?;
                if tool == ClusterTool::Tapered {
                    params.point_a = 0.2;
                    params.point_b = 0.8;
                    params.straight = false;
                }
                session.kind = LayoutKind::Channel(params);
            }
        }
        session.recompute(controller.coordinator.limits())?;
        Ok(session)
    }

    /// Recompute the whole preview from the typed parameters. Nothing is written.
    pub fn recompute(&mut self, limits: &om9_core::budget::Limits) -> Result<()> {
        self.accents = self.layout(limits)?;
        Ok(())
    }

    fn layout(&self, limits: &om9_core::budget::Limits) -> Result<Vec<GemInstance>> {
        match &self.kind {
            LayoutKind::Halo(params) => {
                let center = self
                    .center_instance
                    .as_ref()
                    .ok_or(Error::Missing("halo center"))?;
                Ok(halo_layout(center, params)?.accents)
            }
            LayoutKind::Cluster(params) => {
                let center = self
                    .center_instance
                    .as_ref()
                    .ok_or(Error::Missing("cluster center"))?;
                Ok(cluster_layout(center, params)?.accents)
            }
            LayoutKind::Channel(params) => {
                let [(_, a), (_, b)] = self.curves.as_slice() else {
                    return Err(Error::Missing("gem cluster curves"));
                };
                let gems = match self.tool {
                    ClusterTool::Tapered => vec![tapered_baguette(a, b, params, limits)?.instance],
                    ClusterTool::Between => gems_between_curves(a, b, params, limits)?
                        .gems
                        .into_iter()
                        .map(|gem| gem.instance)
                        .collect(),
                    _ => baguette_channel(a, b, params, limits)?
                        .gems
                        .into_iter()
                        .map(|gem| gem.instance)
                        .collect(),
                };
                Ok(gems)
            }
        }
    }
}

impl LayoutKind {
    fn from_feature(feature: &ClusterFeature) -> Result<Self> {
        Ok(match &feature.kind {
            ClusterKind::Halo { params } => Self::Halo(params.clone()),
            ClusterKind::Cluster { params } => Self::Cluster(params.clone()),
            ClusterKind::Channel { params } => Self::Channel(params.clone()),
        })
    }
}

fn tool_matches(tool: ClusterTool, feature: &ClusterFeature) -> bool {
    matches!(
        (tool, &feature.kind),
        (ClusterTool::Halo, ClusterKind::Halo { .. })
            | (ClusterTool::Cluster, ClusterKind::Cluster { .. })
            | (
                ClusterTool::Channel | ClusterTool::Tapered | ClusterTool::Between,
                ClusterKind::Channel { .. }
            )
    )
}

impl Controller {
    /// Open a halo/cluster/baguette builder from the current selection.
    pub fn dispatch_gem_cluster(&mut self, feature: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let tool = match feature {
            HALO => ClusterTool::Halo,
            CLUSTER => ClusterTool::Cluster,
            CHANNEL => ClusterTool::Channel,
            TAPERED => ClusterTool::Tapered,
            BETWEEN => ClusterTool::Between,
            ORIENT => ClusterTool::Orient,
            _ => return Err(Error::Unsupported("gem cluster command")),
        };
        if self.gem_session.is_some() || self.gem_editor.is_some() || self.gem_placement.is_some() {
            return Err(Error::Busy);
        }
        let session = GemClusterSession::open(tool, self)?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            tool.command(),
            tool.title(),
            None,
        );
        self.gem_cluster = Some(session);
        Ok(())
    }

    /// The open builder, if any.
    pub fn gem_cluster_session(&self) -> Option<&GemClusterSession> {
        self.gem_cluster.as_ref()
    }

    /// Store an edited session after a successful recompute. Invalid parameters keep the old
    /// preview, so a window edit can never leave a half-updated builder.
    pub fn gem_cluster_set(&mut self, mut session: GemClusterSession) -> Result<()> {
        session.recompute(self.coordinator.limits())?;
        self.gem_cluster = Some(session);
        Ok(())
    }

    /// Cancel: the session is dropped and the document was never touched.
    pub fn gem_cluster_cancel(&mut self) -> bool {
        let Some(session) = self.gem_cluster.take() else {
            return false;
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            session.tool.command(),
            "gem layout canceled",
            Some(crate::command_events::EventOutcome::Cancelled),
        );
        true
    }

    /// The `om9.gem.cluster` feature on an entity, when it parses and validates.
    pub fn cluster_feature(&self, host: ObjectId) -> Result<Option<ClusterFeature>> {
        let Some(entity) = self.document().entities.get(&host) else {
            return Ok(None);
        };
        let Some(extension) = entity.extensions.get(CLUSTER_EXTENSION_SLOT) else {
            return Ok(None);
        };
        ClusterFeature::from_extension(extension)
            .map(Some)
            .map_err(|_| Error::Invalid("gem cluster feature record"))
    }

    /// Apply the open builder as one transaction.
    pub fn gem_cluster_apply(&mut self) -> Result<()> {
        let session = self
            .gem_cluster
            .clone()
            .ok_or(Error::Missing("gem cluster session"))?;
        let result = if session.tool == ClusterTool::Orient {
            self.apply_orient_to_gem(&session)
        } else {
            self.apply_cluster_layout(&session)
        };
        match result {
            Ok(()) => {
                self.gem_cluster = None;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    session.tool.command(),
                    "gem layout applied",
                    None,
                );
                Ok(())
            }
            Err(error) => {
                self.gem_cluster = Some(session);
                Err(error)
            }
        }
    }

    fn cluster_child_geometry(
        &self,
        instance: &GemInstance,
        brep: bool,
    ) -> Result<om9_core::geometry::Geometry> {
        let mesh = build_mesh(&instance.spec)?;
        if !mesh.closed {
            return Err(Error::Invalid("gem mesh open"));
        }
        self.gem_geometry(mesh, brep)
    }

    fn apply_cluster_layout(&mut self, session: &GemClusterSession) -> Result<()> {
        if session.accents.len() > MAX_ACCENTS {
            return Err(Error::Budget("gem cluster accent count"));
        }
        let layer = self.gem_layer()?;
        self.ensure_layer_writable(layer)?;
        let host = session.host.ok_or(Error::Missing("gem cluster host"))?;
        let host_entity = self
            .document()
            .entities
            .get(&host)
            .ok_or(Error::Stale("gem cluster host"))?
            .clone();
        let existing = session
            .edit
            .as_ref()
            .map(|feature| feature.accents.clone())
            .unwrap_or_default();
        let document = self.document;
        let mut mutations = Vec::new();
        let mut gem_ids = Vec::with_capacity(session.accents.len());
        for (index, instance) in session.accents.iter().enumerate() {
            let geometry = self.cluster_child_geometry(instance, session.brep)?;
            if let Some(id) = existing.get(index).copied() {
                let Some(entity) = self.document().entities.get(&id.object()) else {
                    return Err(Error::Stale("gem cluster child"));
                };
                let source = self.document().stamp(id.object())?;
                let mut record = gem_record_in(self.document(), id.object())
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
                gem_ids.push(id);
            } else {
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
                // The gem id is the document entity id, exactly like the loader, so a saved
                // feature points at the real child after save/open.
                record.gem_id = GemId(entity.id);
                entity.placement = instance.placement;
                entity
                    .extensions
                    .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
                gem_ids.push(record.gem_id);
                mutations.push(Mutation::Insert { document, entity });
            }
        }
        // A smaller edit removes the now-unused children; Undo restores them with the same ids.
        for id in existing.iter().skip(session.accents.len()) {
            let source = self.document().stamp(id.object())?;
            mutations.push(Mutation::Delete { source });
        }
        let mut feature = session.kind.to_feature(
            session.center,
            [
                session.curves.first().map(|(id, _)| *id),
                session.curves.get(1).map(|(id, _)| *id),
            ],
            gem_ids,
        );
        if let Some(previous) = &session.edit {
            feature.version = previous.version;
        }
        let source = self.document().stamp(host)?;
        let mut extensions = host_entity.extensions.clone();
        extensions.insert(CLUSTER_EXTENSION_SLOT.to_owned(), feature.to_extension()?);
        mutations.push(Mutation::Extensions { source, extensions });
        self.commit_gem(session.tool.title(), mutations)
    }

    fn apply_orient_to_gem(&mut self, session: &GemClusterSession) -> Result<()> {
        let object = session
            .orient_object
            .ok_or(Error::Missing("orient to gem object"))?;
        let gem = session
            .center_instance
            .as_ref()
            .ok_or(Error::Missing("orient to gem gem"))?;
        let entity = self
            .document()
            .entities
            .get(&object)
            .ok_or(Error::Stale("orient to gem object"))?;
        self.ensure_layer_writable(entity.attributes.layer)?;
        let placement = orient_to_gem(gem, &entity.placement, &session.orient_options)?;
        let source = self.document().stamp(object)?;
        self.commit_gem(
            "Orient to Gem",
            vec![Mutation::Placement { source, placement }],
        )
    }

    /// The live preview of every accent, world space and unwritten.
    pub fn gem_cluster_world_preview(&self) -> Vec<(om9_core::gem::GemMesh, Transform)> {
        let Some(session) = self.gem_cluster.as_ref() else {
            return Vec::new();
        };
        session
            .accents
            .iter()
            .filter_map(|instance| {
                build_mesh(&instance.spec)
                    .ok()
                    .map(|mesh| (mesh, instance.placement))
            })
            .collect()
    }

    /// The command-line entry point of an open builder.
    pub fn gem_cluster_text(&mut self, text: &str) -> Option<Result<()>> {
        let session = self.gem_cluster.as_ref()?;
        let mut session = session.clone();
        let mut apply = false;
        let mut cancel = false;
        let outcome: Result<()> = (|| {
            let tokens: Vec<&str> = text.split_whitespace().collect();
            if tokens.is_empty() {
                // Empty Enter is Apply, exactly like the other gem builders.
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
                    let accent = match &session.kind {
                        LayoutKind::Halo(params) => params.accent.clone(),
                        LayoutKind::Cluster(params) => params.accent.clone(),
                        LayoutKind::Channel(params) => params.section.clone(),
                    };
                    session.kind = match session.tool {
                        ClusterTool::Halo => LayoutKind::Halo(HaloParams::new(accent)?),
                        ClusterTool::Cluster => LayoutKind::Cluster(ClusterParams::new(accent)?),
                        _ => LayoutKind::Channel(ChannelParams::new(accent)?),
                    };
                    continue;
                }
                let Some((key, value)) = lower.split_once('=') else {
                    return Err(Error::Invalid("gem layout option"));
                };
                let number = || -> Result<f64> {
                    value
                        .parse::<f64>()
                        .map_err(|_| Error::Invalid("gem layout number"))
                };
                let flag = value == "on" || value == "yes" || value == "true";
                match key {
                    "length" => {
                        let value = number()?;
                        match &mut session.kind {
                            LayoutKind::Halo(params) => {
                                params.accent = params
                                    .accent
                                    .clone()
                                    .set_girdle_mm(value, params.accent.width_mm.min(value))?
                            }
                            LayoutKind::Cluster(params) => {
                                params.accent = params
                                    .accent
                                    .clone()
                                    .set_girdle_mm(value, params.accent.width_mm.min(value))?
                            }
                            LayoutKind::Channel(params) => {
                                params.section = params
                                    .section
                                    .clone()
                                    .set_girdle_mm(value, params.section.width_mm.min(value))?
                            }
                        }
                    }
                    "width" => {
                        let value = number()?;
                        match &mut session.kind {
                            LayoutKind::Halo(params) => {
                                params.accent = params
                                    .accent
                                    .clone()
                                    .set_girdle_mm(params.accent.length_mm.max(value), value)?
                            }
                            LayoutKind::Cluster(params) => {
                                params.accent = params
                                    .accent
                                    .clone()
                                    .set_girdle_mm(params.accent.length_mm.max(value), value)?
                            }
                            LayoutKind::Channel(params) => {
                                params.section = params
                                    .section
                                    .clone()
                                    .set_girdle_mm(params.section.length_mm.max(value), value)?
                            }
                        }
                    }
                    "depth" => {
                        let value = number()?;
                        match &mut session.kind {
                            LayoutKind::Halo(params) => {
                                params.accent = params.accent.clone().set_depth_mm(value)?
                            }
                            LayoutKind::Cluster(params) => {
                                params.accent = params.accent.clone().set_depth_mm(value)?
                            }
                            LayoutKind::Channel(params) => {
                                params.section = params.section.clone().set_depth_mm(value)?
                            }
                        }
                    }
                    "girdle" => {
                        let value = number()?;
                        match &mut session.kind {
                            LayoutKind::Halo(params) => {
                                params.accent =
                                    params.accent.clone().set_girdle_mm_thickness(value)?
                            }
                            LayoutKind::Cluster(params) => {
                                params.accent =
                                    params.accent.clone().set_girdle_mm_thickness(value)?
                            }
                            LayoutKind::Channel(params) => {
                                params.section =
                                    params.section.clone().set_girdle_mm_thickness(value)?
                            }
                        }
                    }
                    "spacing" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.spacing_mm = number()?,
                        LayoutKind::Channel(params) => params.spacing_mm = number()?,
                        LayoutKind::Cluster(_) => {
                            return Err(Error::Invalid("cluster has no spacing"));
                        }
                    },
                    "innerwall" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.inner_wall_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has an inner wall")),
                    },
                    "outerwall" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.outer_wall_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has an outer wall")),
                    },
                    "profilewidth" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.profile_width_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a profile width")),
                    },
                    "profileheight" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.profile_height_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a profile height")),
                    },
                    "channelspacing" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.channel_spacing_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a channel spacing")),
                    },
                    "wallheight" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.channel_wall_height_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a wall height")),
                    },
                    "blend" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.channel_blend_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a blend")),
                    },
                    "edgenudge" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.edge_nudge_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has an edge nudge")),
                    },
                    "prongheight" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.prong_height_offset_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a prong height")),
                    },
                    "prongdome" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.prong_dome = number()?,
                        _ => return Err(Error::Invalid("only halo has a prong dome")),
                    },
                    "gemnudge" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.gem_nudge_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a gem nudge")),
                    },
                    "prongtaper" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.prong_taper = number()?,
                        _ => return Err(Error::Invalid("only halo has a prong taper")),
                    },
                    "prongvertical" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.prong_vertical_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a prong vertical")),
                    },
                    "prongsmeshed" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.prongs_meshed = flag,
                        _ => return Err(Error::Invalid("only halo meshes prongs")),
                    },
                    "count" => {
                        let count = value
                            .parse::<usize>()
                            .map_err(|_| Error::Invalid("gem layout count"))?;
                        match &mut session.kind {
                            LayoutKind::Halo(params) => params.count = Some(count),
                            LayoutKind::Cluster(params) => params.count = Some(count),
                            LayoutKind::Channel(_) => {
                                return Err(Error::Invalid("channel has no count"));
                            }
                        }
                    }
                    "angle" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.angle_deg = number()?,
                        _ => return Err(Error::Invalid("only halo has an angle")),
                    },
                    "fromcenter" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.spacing_from_center_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a center offset")),
                    },
                    "height" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.height_offset_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a height offset")),
                    },
                    "placement" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.placement_mm = number()?,
                        _ => return Err(Error::Invalid("only halo has a placement")),
                    },
                    "rotation" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.rotation_deg = number()?,
                        LayoutKind::Cluster(params) => params.degree_deg = number()?,
                        LayoutKind::Channel(_) => {
                            return Err(Error::Invalid("channel has no rotation"));
                        }
                    },
                    "xy" => match &mut session.kind {
                        LayoutKind::Cluster(params) => params.xy_offset_mm = number()?,
                        _ => return Err(Error::Invalid("only cluster has an xy offset")),
                    },
                    "z" => match &mut session.kind {
                        LayoutKind::Cluster(params) => params.z_offset_mm = number()?,
                        _ => return Err(Error::Invalid("only cluster has a z offset")),
                    },
                    "start" => match &mut session.kind {
                        LayoutKind::Cluster(params) => {
                            params.start = match value {
                                "top" => ClusterStart::Top,
                                "left" => ClusterStart::Left,
                                "right" => ClusterStart::Right,
                                "bottom" => ClusterStart::Bottom,
                                _ => return Err(Error::Invalid("cluster start point")),
                            }
                        }
                        _ => return Err(Error::Invalid("only cluster has a start point")),
                    },
                    "taper" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.taper = number()?.clamp(0.05, 1.0),
                        _ => return Err(Error::Invalid("only a channel has a taper")),
                    },
                    "banking" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.banking_deg = number()?,
                        _ => return Err(Error::Invalid("only a channel has banking")),
                    },
                    "pointa" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.point_a = number()?,
                        _ => return Err(Error::Invalid("only a channel has points")),
                    },
                    "pointb" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.point_b = number()?,
                        _ => return Err(Error::Invalid("only a channel has points")),
                    },
                    "flip" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.flip = flag || value == "1",
                        _ => return Err(Error::Invalid("only a channel flips")),
                    },
                    "middle" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.middle = flag || value == "1",
                        _ => return Err(Error::Invalid("only a channel has a middle")),
                    },
                    "straight" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.straight = flag || value == "1",
                        _ => return Err(Error::Invalid("only a channel is straight")),
                    },
                    "even" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.even = flag,
                        _ => return Err(Error::Invalid("only halo is even")),
                    },
                    "match" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.match_shape = flag,
                        _ => return Err(Error::Invalid("only halo matches a shape")),
                    },
                    "fix" => match &mut session.kind {
                        LayoutKind::Halo(params) => params.fix_position = flag,
                        _ => return Err(Error::Invalid("only halo fixes a position")),
                    },
                    "scaletogem" => {
                        if session.tool != ClusterTool::Orient {
                            return Err(Error::Invalid("only orient scales to a gem"));
                        }
                        session.orient_options.scale_to_gem = flag;
                    }
                    "autospacing" => match &mut session.kind {
                        LayoutKind::Channel(params) => params.auto_spacing = flag,
                        _ => return Err(Error::Invalid("only a channel auto-spaces")),
                    },
                    _ => return Err(Error::Invalid("gem layout option")),
                }
            }
            Ok(())
        })();
        if let Err(error) = outcome {
            return Some(Err(error));
        }
        if cancel {
            self.gem_cluster_cancel();
            return Some(Ok(()));
        }
        if let Err(error) = session.recompute(self.coordinator.limits()) {
            return Some(Err(error));
        }
        self.gem_cluster = Some(session);
        if apply {
            return Some(self.gem_cluster_apply());
        }
        Some(Ok(()))
    }

    /// The prompt line of the open builder.
    pub fn gem_cluster_prompt(&self) -> Option<String> {
        self.gem_cluster.as_ref().map(GemClusterSession::prompt)
    }

    /// The host layer of a builder edit, used by the window when it reports the target.
    pub fn gem_cluster_layer(&self) -> Result<Option<LayerId>> {
        self.gem_layer()
    }
}
