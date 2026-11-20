// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit native plugin loading and the shared CLI/CMD transaction boundary.

use super::Controller;
use om9_core::{
    Error, Result, changes::ChangeCause, identity::ObjectId, transaction::TransactionRequest,
};
use om9_plugin_host::{LoadedPlugin, ProjectState};
use om9_scene::scene::Scene;
use om9_sdk::{MessageLevel, PluginDescriptor};
use serde_json::Value;
use std::{path::Path, rc::Rc};

fn project_change(batch: &om9_core::changes::ChangeBatch) -> om9_plugin_host::ProjectChange {
    use om9_core::changes::Change;
    let changes = batch
        .changes
        .iter()
        .map(|change| {
            let (kind, id) = match change {
                Change::CurrentLayerChanged(id) => {
                    ("current_layer_changed", id.map(|id| id.to_string()))
                }
                Change::LayerOrderChanged => ("layer_order_changed", None),
                Change::LayerCreated(id) => ("layer_created", Some(id.to_string())),
                Change::LayerModified(id) => ("layer_modified", Some(id.to_string())),
                Change::LayerRemoved(id) => ("layer_removed", Some(id.to_string())),
                Change::EntityAdded(id) => ("entity_added", Some(id.to_string())),
                Change::EntityRemoved(id) => ("entity_removed", Some(id.to_string())),
                Change::EntityModified(id) => ("entity_modified", Some(id.to_string())),
                Change::ExtensionChanged(id) => ("extension_changed", Some(id.to_string())),
                Change::GroupCreated(id) => ("group_created", Some(id.to_string())),
                Change::GroupModified(id) => ("group_modified", Some(id.to_string())),
                Change::GroupRemoved(id) => ("group_removed", Some(id.to_string())),
                Change::FeatureAdded(id) => ("feature_added", Some(id.to_string())),
                Change::FeatureModified(id) => ("feature_modified", Some(id.to_string())),
                Change::FeatureRemoved(id) => ("feature_removed", Some(id.to_string())),
            };
            om9_plugin_host::ChangeEntry {
                kind: kind.into(),
                id,
            }
        })
        .collect();
    om9_plugin_host::ProjectChange {
        from_revision: batch.from_revision,
        to_revision: batch.to_revision,
        cause: match batch.cause {
            om9_core::changes::ChangeCause::Edit => "edit",
            om9_core::changes::ChangeCause::Undo => "undo",
            om9_core::changes::ChangeCause::Redo => "redo",
            om9_core::changes::ChangeCause::Replace => "replace",
            om9_core::changes::ChangeCause::Restore => "restore",
        }
        .into(),
        changes,
    }
}
fn plugin_error(error: impl std::fmt::Display) -> Error {
    Error::Io(format!("plugin: {error}"))
}

fn token(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    if text.is_empty() {
        return None;
    }
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    Some((&text[..end], text[end..].trim_start()))
}

#[cfg(feature = "occt-brep")]
struct PluginTrackAnchor {
    token: uuid::Uuid,
    plugin_id: String,
    publication: uuid::Uuid,
    plane: om9_scene::camera::CPlane,
    point: [f64; 3],
    tangent: Option<[f64; 3]>,
}

#[cfg(feature = "occt-brep")]
struct IssuedEndDirection {
    invocation: uuid::Uuid,
    hit: om9_sdk::SnapHit,
    tangent: [f64; 3],
}

/// Invocation-independent, source-stamped work retained across SDK retries.
#[cfg(feature = "occt-brep")]
#[derive(Default)]
pub(super) struct PluginSnapWorkers {
    publication: Option<uuid::Uuid>,
    bounds: super::edge_snap::EdgeBoundsCache,
    sources: super::edge_snap::EdgeSnapSourceCache,
    views: super::edge_snap::EdgeSnapViewScheduler,
    intersections: super::intersection_snap::IntersectionScheduler,
    surface: super::surface_snap::SurfaceSnapScheduler,
    surface_receipt: Option<(om9_scene::workspace::DisplayMode, super::surface_snap::SurfaceSnapReceipt)>,
    track_anchors: std::collections::VecDeque<PluginTrackAnchor>,
    issued_end_directions: Vec<IssuedEndDirection>,
}

#[cfg(feature = "occt-brep")]
impl PluginSnapWorkers {
    fn cancel_edge_work(&mut self) {
        // Drop receivers as well as queued work so abandoned jobs can release
        // reservations without another call to poll the caches.
        self.bounds = Default::default();
        self.sources = Default::default();
        self.views = Default::default();
        self.intersections = Default::default();
    }

    fn cancel_surface(&mut self) {
        self.surface_receipt = None;
        // Dropping the scheduler cancels its job and receiver. The worker owns
        // its reservation and permit until the native call actually returns,
        // even if the plugin is never invoked again.
        self.surface = super::surface_snap::SurfaceSnapScheduler::default();
    }

    fn surface_query(&mut self, controller: &Controller, candidates: Vec<om9_scene::osnap::SnapCandidate>,
        camera: &om9_scene::camera::Camera, viewport: om9_scene::camera::Viewport, cursor: [f64; 2],
        mode: om9_scene::workspace::DisplayMode) -> Result<Option<Option<om9_scene::osnap::SnapCandidate>>> {
        use om9_core::budget::memory::MemoryClass;
        if !self.surface_receipt.as_ref().is_some_and(|(saved_mode, receipt)|
            *saved_mode == mode && receipt.matches_request(controller, &candidates, camera, viewport, cursor)) {
            self.surface_receipt = None;
        }
        if self.surface_receipt.is_none() && !self.surface.matches_request(controller, &candidates, camera, viewport, cursor) {
            let floor = super::surface_snap::SurfaceSnapJob::source_only_minimum(controller, &candidates, camera, viewport, cursor)?
                .ok_or(Error::Unsupported("SDK surface snap source"))?;
            let bytes = floor.checked_add(8 * 1024 * 1024).ok_or(Error::Overflow("SDK surface snap admission"))?;
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            let guard = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, bytes,
                physical, controller.scene_cache_usage()?)?;
            self.surface.submit_with_occlusion(controller, candidates, camera.clone(), viewport, cursor, guard)?;
        }
        if self.surface_receipt.is_none() {
            if let Some(result) = self.surface.poll(controller) { self.surface_receipt = Some((mode, result?)); }
        }
        let Some((_, receipt)) = &self.surface_receipt else { return Ok(None) };
        let hit = receipt.resolve(controller, camera, viewport, cursor)?;
        if mode == om9_scene::workspace::DisplayMode::Shaded && hit.is_some() {
            if receipt.display_occluded().ok_or(Error::Missing("SDK surface snap occlusion"))? {
                return Err(Error::Unsupported("surface snap is occluded by shaded geometry"));
            }
        }
        Ok(Some(hit))
    }

    fn query(&mut self, controller: &Controller, input: super::edge_snap::EdgeSnapViewInput)
        -> Result<Option<Vec<om9_scene::osnap::SnapCandidate>>> {
        use om9_core::{budget::memory::MemoryClass, geometry::Geometry};
        use om9_scene::osnap::{SnapKind, occurrence_source};
        let settings = input.settings;
        let edge_query = [SnapKind::End, SnapKind::Mid, SnapKind::Near, SnapKind::Quadrant, SnapKind::Center]
            .into_iter().any(|kind| settings.is_effective(kind))
            || input.from.is_some() && (settings.is_effective(SnapKind::Tangent) || settings.is_effective(SnapKind::Perpendicular));
        let intersections = settings.is_effective(SnapKind::Intersection);
        let tracking = input.guides.iter().any(Option::is_some);
        if !edge_query && !intersections && !tracking {
            self.cancel_edge_work();
            return Ok(Some(Vec::new()));
        }
        let scene = controller.scene().ok_or(Error::Missing("SDK snap scene"))?;
        if self.publication != Some(scene.publication_id()) {
            self.cancel_edge_work();
            self.cancel_surface();
            self.publication = Some(scene.publication_id());
        }
        let mut sources = Vec::new();
        for (index, object) in scene.objects.iter().enumerate() {
            if !object.visible || !(object.selectable || object.reference_snappable) { continue; }
            let entity = scene.entity_for(&scene.document, object)?;
            let needs_source = matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_) | Geometry::Surface(_))
                || matches!(&entity.geometry, Geometry::Curve(curve) if om9_scene::osnap::contains_nurbs(curve)
                    || (settings.is_effective(SnapKind::Mid) || settings.is_effective(SnapKind::Near)
                        || input.from.is_some() && settings.is_effective(SnapKind::Perpendicular))
                        && om9_scene::osnap::contains_circle(curve));
            if !needs_source { continue; }
            if sources.len() as u64 >= controller.coordinator().limits().max_outputs { return Err(Error::Budget("SDK snap sources")); }
            sources.try_reserve(1).map_err(|_| Error::Budget("SDK snap sources"))?;
            sources.push(occurrence_source(scene, index)?);
        }
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        self.bounds.poll(controller)?;
        for source in &sources {
            if self.bounds.may_hit(controller, source, &input.camera, input.viewport, input.cursor, settings.radius_px)?.is_some() { continue; }
            if !self.bounds.is_busy() {
                let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,
                    32 * 1024 * 1024, physical, controller.scene_cache_usage()?)?;
                self.bounds.request(controller, source.clone(), reservation)?;
            }
            return Ok(None);
        }
        let mut eligible = Vec::new();
        for source in sources {
            if settings.is_effective(SnapKind::Center)
                || self.bounds.may_hit(controller, &source, &input.camera, input.viewport, input.cursor, settings.radius_px)? == Some(true) {
                eligible.try_reserve(1).map_err(|_| Error::Budget("SDK snap sources"))?;
                eligible.push(source);
            }
        }
        self.sources.poll(controller)?;
        for source in &eligible {
            if self.sources.ready(controller, source).is_some() { continue; }
            if !self.sources.is_busy() {
                let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,
                    32 * 1024 * 1024, physical, controller.scene_cache_usage()?)?;
                self.sources.request(controller, source.clone(), reservation)?;
            }
            return Ok(None);
        }
        let mut result = Vec::new();
        if edge_query {
            self.views.poll(controller)?;
            for source in &eligible {
                let receipt = self.sources.ready(controller, source).ok_or(Error::Stale("SDK snap source cache"))?;
                if !settings.is_effective(SnapKind::Center)
                    && !receipt.may_hit(controller, &input.camera, input.viewport, input.cursor, settings.radius_px)? { continue; }
                if let Some(candidates) = self.views.ready(controller, &receipt, &input) {
                    if result.len().saturating_add(candidates.len()) as u64 > controller.coordinator().limits().max_outputs {
                        return Err(Error::Budget("SDK snap candidates"));
                    }
                    result.try_reserve(candidates.len()).map_err(|_| Error::Budget("SDK snap candidates"))?;
                    result.extend_from_slice(candidates);
                } else {
                    if self.views.is_busy() {
                        if !self.views.matches_active(&receipt, &input) { self.views.cancel(); }
                    } else {
                        let bytes = super::edge_snap::EdgeSnapViewJob::required_bytes(&receipt, &input, controller.coordinator().limits())?;
                        let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::CpuCache,
                            bytes, physical, controller.scene_cache_usage()?)?;
                        self.views.request(controller, receipt, input.clone(), reservation)?;
                    }
                    return Ok(None);
                }
            }
        }
        if intersections || tracking {
            let mut occurrences = Vec::new();
            for (index, object) in scene.objects.iter().enumerate() {
                if !object.visible || !(object.selectable || object.reference_snappable) { continue; }
                let entity = scene.entity_for(&scene.document, object)?;
                if !matches!(entity.geometry, Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_) | Geometry::NativeBrep(_)) { continue; }
                let source = occurrence_source(scene, index)?;
                let edges = if matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_))
                    || matches!(entity.geometry, Geometry::Surface(_)) && entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                    if self.bounds.may_hit(controller, &source, &input.camera, input.viewport, input.cursor, settings.radius_px)? != Some(true) { continue; }
                    let receipt = self.sources.ready(controller, &source).ok_or(Error::Stale("SDK intersection edge cache"))?;
                    if !receipt.may_hit(controller, &input.camera, input.viewport, input.cursor, settings.radius_px)? { continue; }
                    Some(receipt)
                } else { None };
                if occurrences.len() as u64 >= controller.coordinator().limits().max_outputs { return Err(Error::Budget("SDK snap occurrences")); }
                occurrences.try_reserve(1).map_err(|_| Error::Budget("SDK snap occurrences"))?;
                occurrences.push(super::intersection_snap::IntersectionOccurrence { source, transform: object.display_transform()?, edges });
            }
            if !occurrences.is_empty() {
                self.intersections.poll(controller)?;
                if let Some(candidates) = self.intersections.ready(controller, &occurrences, &input) {
                    if result.len().saturating_add(candidates.len()) as u64 > controller.coordinator().limits().max_outputs {
                        return Err(Error::Budget("SDK intersection candidates"));
                    }
                    result.try_reserve(candidates.len()).map_err(|_| Error::Budget("SDK intersection candidates"))?;
                    result.extend_from_slice(candidates);
                } else {
                    if !self.intersections.matches(controller, &occurrences, &input) {
                        let bytes = super::intersection_snap::IntersectionJob::required_bytes(&occurrences)?;
                        let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::CpuCache,
                            bytes, physical, controller.scene_cache_usage()?)?;
                        self.intersections.request(controller, occurrences, input, reservation)?;
                    }
                    return Ok(None);
                }
            }
        }
        Ok(Some(result))
    }
}

impl Controller {
    fn plugin_snap_request(&self, _plugin_id: &str, invocation: uuid::Uuid, request: om9_sdk::HostRequest) -> om9_sdk::Result<serde_json::Value> {
        use om9_scene::osnap::{SnapKind, SnapSettings};
        use om9_sdk::{HostRequest, SdkError};
        let fail = |code: &str, message: String| SdkError::new(code, message);
        let scene = self.scene().ok_or_else(|| fail("missing", "snap scene unavailable".into()))?;
        if scene.epoch != self.coordinator().epoch() {
            return Err(fail("stale", "snap scene changed".into()));
        }
        let view = self.workspace.views.get(&self.workspace.active)
            .ok_or_else(|| fail("missing", "active view unavailable".into()))?;
        let token = format!("{}:{}:{:?}:{}:{}:{}", invocation, scene.publication_id(), self.workspace.active,
            view.revision, view.camera.revision, view.cplane.revision);
        #[cfg(not(feature = "occt-brep"))]
        if matches!(request, HostRequest::SnapTrackAnchor { .. } | HostRequest::SnapQueryTracked { .. }) {
            return Err(fail("unsupported", "SmartTrack queries need source workers".into()));
        }
        #[cfg(feature = "occt-brep")]
        let mut tracked_directions = [None; 2];
        #[cfg(feature = "occt-brep")]
        let request = match request {
            HostRequest::SnapTrackAnchor { context_token, hit } => {
                if context_token != token { return Err(fail("stale", "snap context changed".into())); }
                let mut workers = self.sdk_snap_workers.try_borrow_mut()
                    .map_err(|_| fail("busy", "snap query is already active".into()))?;
                let tangent = workers.issued_end_directions.iter().rev()
                    .find(|issued| issued.invocation == invocation && issued.hit == hit)
                    .map(|issued| issued.tangent);
                if workers.track_anchors.len() == 32 { workers.track_anchors.pop_front(); }
                workers.track_anchors.try_reserve(1).map_err(|e| fail("budget", e.to_string()))?;
                let anchor = uuid::Uuid::new_v4();
                workers.track_anchors.push_back(PluginTrackAnchor {
                    token: anchor,
                    plugin_id: _plugin_id.to_owned(),
                    publication: scene.publication_id(),
                    plane: view.cplane,
                    point: hit.point_mm,
                    tangent,
                });
                return Ok(serde_json::Value::String(anchor.to_string()));
            }
            HostRequest::SnapQueryTracked { mut query, anchors } => {
                if query.context_token != token { return Err(fail("stale", "snap context changed".into())); }
                if !(1..=2).contains(&anchors.len()) || !query.tracking_anchors_mm.is_empty() {
                    return Err(fail("invalid", "expected one or two SmartTrack anchor tokens".into()));
                }
                let workers = self.sdk_snap_workers.try_borrow()
                    .map_err(|_| fail("busy", "snap query is already active".into()))?;
                for (anchor, direction) in anchors.into_iter().zip(&mut tracked_directions) {
                    let id = uuid::Uuid::parse_str(&anchor).map_err(|e| fail("invalid", e.to_string()))?;
                    let anchor = workers.track_anchors.iter().find(|anchor| {
                        anchor.token == id && anchor.plugin_id == _plugin_id
                            && anchor.publication == scene.publication_id() && anchor.plane == view.cplane
                    }).ok_or_else(|| fail("stale", "SmartTrack anchor expired or belongs to another plugin".into()))?;
                    query.tracking_anchors_mm.push(anchor.point);
                    *direction = anchor.tangent;
                }
                HostRequest::SnapQuery { query }
            }
            request => request,
        };
        let encode = |value: serde_json::Value| Ok(value);
        match request {
            HostRequest::SnapContext => {
                let tolerance = self.document().effective_tolerance().map_err(|e| fail("invalid", e.to_string()))?;
                serde_json::to_value(om9_sdk::SnapContext { token, view: self.workspace.active.title().to_owned(),
                    absolute_tolerance_mm: tolerance.absolute_mm, plane_origin_mm: view.cplane.origin,
                    plane_x: view.cplane.x, plane_y: view.cplane.y }).map_err(|e| fail("json", e.to_string()))
            }
            HostRequest::SnapConstrain { request } => {
                if request.context_token != token { return Err(fail("stale", "snap context changed".into())); }
                let base = request.base_mm.map(om9_core::coords::WorldPoint::try_new).transpose()
                    .map_err(|e| fail("invalid", e.to_string()))?;
                let point = if request.object_snap {
                    om9_core::geometry::finite(request.point_mm).map_err(|e| fail("invalid", e.to_string()))?;
                    if self.project() { self.project_snap_with_base(request.point_mm, base).map_err(|e| fail("invalid", e.to_string()))? }
                    else { request.point_mm }
                } else {
                    self.constrain_point_with_base(request.point_mm, request.shift, base)
                        .map_err(|e| fail("invalid", e.to_string()))?
                };
                serde_json::to_value(point).map_err(|e| fail("json", e.to_string()))
            }
            HostRequest::SnapQuery { query } => {
                if query.context_token != token { return Err(fail("stale", "snap context changed".into())); }
                if query.kinds.is_empty() || query.kinds.len() > 11 || query.tracking_anchors_mm.len() > 2 || !query.radius_px.is_finite()
                    || !(0.0..=128.0).contains(&query.radius_px) || !query.cursor.iter().all(|v| v.is_finite()) {
                    return Err(fail("invalid", "invalid snap query".into()));
                }
                #[cfg(not(feature = "occt-brep"))]
                if !query.tracking_anchors_mm.is_empty() {
                    return Err(fail("unsupported", "SmartTrack queries need source workers".into()));
                }
                let viewport = om9_scene::camera::Viewport { origin: query.viewport_origin, size: query.viewport_size };
                viewport.validate().map_err(|e| fail("invalid", e.to_string()))?;
                if viewport.size.iter().any(|size| *size > 32768.) { return Err(fail("budget", "snap viewport too large".into())); }
                let mut settings = SnapSettings { enabled: 0, radius_px: query.radius_px, ..Default::default() };
                for name in &query.kinds {
                    let kind = match name.to_ascii_lowercase().as_str() {
                        "end" => SnapKind::End, "mid" => SnapKind::Mid, "center" => SnapKind::Center,
                        "quadrant" => SnapKind::Quadrant, "point" => SnapKind::Point, "near" => SnapKind::Near,
                        "tangent" => SnapKind::Tangent, "perpendicular" => SnapKind::Perpendicular,
                        "intersection" => SnapKind::Intersection,
                        "apparent_intersection" => {
                            settings.apparent_intersections = true;
                            SnapKind::Intersection
                        }
                        #[cfg(feature = "occt-brep")]
                        "surface" => SnapKind::Surface,
                        #[cfg(feature = "occt-brep")]
                        "smarttrack" => SnapKind::SmartTrack,
                        _ => return Err(fail("unsupported", format!("snap kind {name} requires a source-aware host query"))),
                    };
                    settings.set(kind, true);
                }
                #[cfg(feature = "occt-brep")]
                let tracking = if settings.is_effective(SnapKind::SmartTrack) {
                    if query.tracking_anchors_mm.is_empty() { return Err(fail("invalid", "SmartTrack needs at least one anchor".into())); }
                    let mut tracking = om9_scene::osnap::SmartTrack::new();
                    for (&world, tracking_tangent) in query.tracking_anchors_mm.iter().zip(tracked_directions) {
                        om9_core::geometry::finite(world).map_err(|e| fail("invalid", e.to_string()))?;
                        let kind = if tracking_tangent.is_some() { SnapKind::End } else { SnapKind::Point };
                        let anchor = om9_scene::osnap::SnapCandidate { kind, root: None, world,
                            screen: [0.; 2], distance_px: 0., normal: None, parameter: None,
                            tracking_tangent, source: None, second_source: None, surface_source: None, apparent: false };
                        tracking.acquire(&anchor, view.cplane, self.document().effective_tolerance()
                            .map_err(|e| fail("invalid", e.to_string()))?.absolute_mm)
                            .map_err(|e| fail("invalid", e.to_string()))?;
                    }
                    Some(tracking)
                } else {
                    if !query.tracking_anchors_mm.is_empty() { return Err(fail("invalid", "tracking anchors require SmartTrack".into())); }
                    None
                };
                #[cfg(feature = "occt-brep")]
                if !settings.is_effective(SnapKind::Surface) {
                    self.sdk_snap_workers.try_borrow_mut().map_err(|_| fail("busy", "snap query is already active".into()))?
                        .cancel_surface();
                }
                if let Some(point) = query.from_mm { om9_core::geometry::finite(point).map_err(|e| fail("invalid", e.to_string()))?; }
                #[cfg(not(feature = "occt-brep"))]
                {
                // Without source workers do not claim a complete ranking.
                // The synchronous scene query does not include the viewport's
                // source-edge and general-intersection workers. Do not return a
                // lower-ranked hit (or a miss) as though the query were complete.
                let source_edges = [SnapKind::End, SnapKind::Mid, SnapKind::Center, SnapKind::Quadrant,
                    SnapKind::Near, SnapKind::Intersection].into_iter().any(|kind| settings.is_effective(kind))
                    || query.from_mm.is_some() && [SnapKind::Tangent, SnapKind::Perpendicular]
                        .into_iter().any(|kind| settings.is_effective(kind));
                if source_edges {
                    for object in &scene.objects {
                        if !object.visible || !(object.selectable || object.reference_snappable) { continue; }
                        let entity = scene.entity_for(&scene.document, object).map_err(|e| fail("stale", e.to_string()))?;
                        let missing = match &entity.geometry {
                            om9_core::geometry::Geometry::Brep(_) | om9_core::geometry::Geometry::NativeBrep(_) => true,
                            om9_core::geometry::Geometry::Surface(_) =>
                                entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
                                    || settings.is_effective(SnapKind::Intersection),
                            om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Nurbs(_)
                                | om9_core::geometry::Curve::Polycurve { .. }) => settings.is_effective(SnapKind::Intersection),
                             om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Circle { .. }) =>
                                 settings.apparent_intersections,
                            _ => false,
                        };
                        if missing {
                            return Err(fail("unsupported", "snap query needs asynchronous source-edge or intersection evaluation".into()));
                        }
                    }
                }
                }
                #[cfg(not(feature = "occt-brep"))]
                let candidates = om9_scene::osnap::candidates_in_plane(scene, &view.camera, viewport, query.cursor,
                    settings, query.from_mm, view.cplane).map_err(|e| fail("snap", e.to_string()))?;
                #[cfg(feature = "occt-brep")]
                let candidates = {
                    let mut candidates = om9_scene::osnap::candidates_in_plane_deferred(scene, &view.camera, viewport, query.cursor,
                        settings, query.from_mm, view.cplane).map_err(|e| fail("snap", e.to_string()))?;
                    let mut guides = [None; om9_scene::osnap::MAX_SMARTTRACK_GUIDES];
                    if let Some(tracking) = &tracking {
                        for (slot, guide) in guides.iter_mut().zip(tracking.guides()) { *slot = Some(*guide); }
                    }
                    let input = super::edge_snap::EdgeSnapViewInput { camera: view.camera.clone(), viewport, cursor: query.cursor,
                        settings, from: query.from_mm, cplane: view.cplane, mode: view.mode, guides };
                    let mut workers = self.sdk_snap_workers.try_borrow_mut()
                        .map_err(|_| fail("busy", "snap query is already active".into()))?;
                    let extra = workers.query(self, input).map_err(|e| fail("snap", e.to_string()))?
                        .ok_or_else(|| fail("pending", "source snap evaluation is pending; retry this invocation".into()))?;
                    if candidates.len().saturating_add(extra.len()) as u64 > self.coordinator().limits().max_outputs {
                        return Err(fail("budget", "snap candidate count".into()));
                    }
                    candidates.try_reserve(extra.len()).map_err(|_| fail("budget", "snap candidate allocation".into()))?;
                    candidates.extend(extra);
                    candidates
                };
                let first = om9_scene::osnap::choose(&candidates);
                #[cfg(feature = "occt-brep")]
                let guide_perpendicular = match (&tracking, query.from_mm) {
                    (Some(tracking), Some(base)) if settings.is_effective(SnapKind::Perpendicular)
                        && first.as_ref().is_none_or(|hit| hit.kind == SnapKind::SmartTrack) =>
                    {
                        tracking.infer_perpendicular_in_view(base, &view.camera, viewport, query.cursor,
                            query.radius_px).map_err(|e| fail("snap", e.to_string()))?
                    }
                    _ => None,
                };
                #[cfg(not(feature = "occt-brep"))]
                let guide_perpendicular: Option<om9_scene::osnap::SnapCandidate> = None;
                let from_tracking_guide = guide_perpendicular.is_some();
                let mut first = guide_perpendicular.or(first);
                #[cfg(feature = "occt-brep")]
                if first.is_none() && let Some(tracking) = &tracking {
                    let tolerance = self.document().effective_tolerance().map_err(|e| fail("invalid", e.to_string()))?;
                    first = tracking.infer_in_view(&view.camera, viewport, query.cursor,
                        query.radius_px, tolerance.absolute_mm).map_err(|e| fail("snap", e.to_string()))?;
                }
                #[cfg(feature = "occt-brep")]
                if first.as_ref().is_none_or(|hit| hit.kind != SnapKind::Surface) && settings.is_effective(SnapKind::Surface) {
                    self.sdk_snap_workers.try_borrow_mut().map_err(|_| fail("busy", "snap query is already active".into()))?
                        .cancel_surface();
                }
                let Some(mut hit) = first else { return encode(serde_json::Value::Null); };
                #[cfg(feature = "occt-brep")]
                if hit.kind == SnapKind::Surface {
                    let surfaces = candidates.into_iter().filter(|candidate| candidate.kind == SnapKind::Surface).collect();
                    let mut workers = self.sdk_snap_workers.try_borrow_mut()
                        .map_err(|_| fail("busy", "snap query is already active".into()))?;
                    hit = match workers.surface_query(self, surfaces, &view.camera, viewport, query.cursor, view.mode)
                        .map_err(|error| fail("snap", error.to_string()))? {
                        None => return Err(fail("pending", "source surface evaluation is pending; retry this invocation".into())),
                        Some(None) => return encode(serde_json::Value::Null),
                        Some(Some(hit)) => hit,
                    };
                }
                let identity = |source: &om9_scene::osnap::SnapSource| -> om9_sdk::Result<om9_sdk::SnapSourceIdentity> {
                    source.validate(scene).map_err(|e| fail("stale", e.to_string()))?;
                    Ok(om9_sdk::SnapSourceIdentity { root_id: source.root.to_string(),
                        member_path: source.path.iter().map(ToString::to_string).collect(), leaf_id: source.leaf.to_string(),
                        source_generation: source.stamp.generation, geometry_revision: source.stamp.geometry_revision,
                        leaf_generation: source.leaf_generation, leaf_geometry_revision: source.leaf_geometry_revision })
                };
                let source = hit.source.as_ref().map(identity).transpose()?;
                if source.is_none() && hit.kind != SnapKind::SmartTrack && !from_tracking_guide {
                    return Err(fail("unsupported", "snap has no source binding".into()));
                }
                let second_source = hit.second_source.as_ref().map(|source| identity(source)).transpose()?;
                if hit.kind == SnapKind::Intersection && second_source.is_none() {
                    return Err(fail("unsupported", "intersection has no second source binding".into()));
                }
                let kind = if hit.apparent { "Apparent Intersection" } else { hit.kind.label() };
                let result = om9_sdk::SnapHit { kind: kind.to_owned(), point_mm: hit.world,
                    root_id: source.as_ref().map(|s| s.root_id.clone()), member_path: source.as_ref().map(|s| s.member_path.clone()).unwrap_or_default(),
                    leaf_id: source.as_ref().map(|s| s.leaf_id.clone()), source_generation: source.as_ref().map(|s| s.source_generation),
                    geometry_revision: source.as_ref().map(|s| s.geometry_revision), leaf_generation: source.as_ref().map(|s| s.leaf_generation),
                    leaf_geometry_revision: source.as_ref().map(|s| s.leaf_geometry_revision), second_source };
                let value = serde_json::to_value(&result).map_err(|e| fail("json", e.to_string()))?;
                #[cfg(feature = "occt-brep")]
                if let Some(tangent) = hit.tracking_tangent {
                    let mut workers = self.sdk_snap_workers.try_borrow_mut()
                        .map_err(|_| fail("busy", "snap query is already active".into()))?;
                    workers.issued_end_directions.try_reserve(1).map_err(|e| fail("budget", e.to_string()))?;
                    workers.issued_end_directions.push(IssuedEndDirection { invocation, hit: result, tangent });
                }
                Ok(value)
            }
            _ => Err(fail("unsupported", "not a snap request".into())),
        }
    }
    /// Load a package only on an explicit request. Its library remains live for
    /// this controller; duplicate plugin IDs are refused.
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginDescriptor> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let plugin = LoadedPlugin::open(path).map_err(plugin_error)?;
        let descriptor = plugin.descriptor().clone();
        if self
            .plugins
            .iter()
            .any(|loaded| loaded.descriptor().id == descriptor.id)
        {
            return Err(Error::Invalid("duplicate plugin ID"));
        }
        self.plugins
            .try_reserve(1)
            .map_err(|_| Error::Budget("loaded plugins"))?;
        self.plugins.push(Rc::new(plugin));
        Ok(descriptor)
    }

    pub fn plugin_descriptors(&self) -> Vec<PluginDescriptor> {
        self.plugins
            .iter()
            .map(|plugin| plugin.descriptor().clone())
            .collect()
    }

    /// The same service used by live CMD invocation and the headless CLI.
    pub fn run_loaded_plugin(
        &mut self,
        id: &str,
        command: &str,
        arguments: Value,
    ) -> Result<Value> {
        let plugin = self
            .plugins
            .iter()
            .find(|plugin| plugin.descriptor().id == id)
            .cloned()
            .ok_or(Error::Missing("loaded plugin"))?;
        self.run_plugin(&plugin, command, arguments)
    }

    /// Plugins stage owned mutations. A successful command publishes a fully
    /// prepared scene and document together, with one ordinary OM9 Undo entry.
    pub fn run_plugin(
        &mut self,
        plugin: &LoadedPlugin,
        command: &str,
        arguments: Value,
    ) -> Result<Value> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let event_command = format!("{}:{command}", plugin.descriptor().id);
        self.events.emit(
            crate::command_events::EventKind::Started,
            &event_command,
            "plugin started",
            None,
        );
        let result = (|| {
            let selection = self.selected().iter().copied().collect::<Vec<_>>();
            let last_change = self.plugin_change.as_ref().map(project_change);
            let project = ProjectState {
                path: self.file.path.clone(),
                dirty: self.is_dirty()?,
            };
            let invocation = uuid::Uuid::new_v4();
            let snap_service = |request| self.plugin_snap_request(&plugin.descriptor().id, invocation, request);
            let run = plugin
                .prepare_with_snap_service(
                    &self.coordinator,
                    self.document,
                    &selection,
                    project,
                    last_change,
                    command,
                    arguments,
                    Some(&snap_service),
                );
            #[cfg(feature = "occt-brep")]
            self.sdk_snap_workers.get_mut().issued_end_directions.clear();
            let run = run.map_err(plugin_error)?;
            // The plugin reports progress/warnings through the same bounded event log
            // the Command line and its observers read (OM9-L3-01-02-010).
            for message in &run.messages {
                self.events.emit(
                    match message.level {
                        MessageLevel::Progress => crate::command_events::EventKind::Progress,
                        MessageLevel::Warning => crate::command_events::EventKind::Warn,
                    },
                    &event_command,
                    &message.text,
                    None,
                );
            }
            let staged_selection = run.selection.clone();
            let before = self.coordinator.capture_retained_document(self.document)?;
            if let Some(request) = run.request {
                self.publish_plugin_request(request, staged_selection.as_deref())?;
            } else if let Some(ids) = staged_selection.as_deref() {
                self.apply_plugin_selection_now(ids)?;
            }
            if run.save_as
                && let Err(error) =
                    self.begin_file_operation(crate::file_lifecycle::FileKind::SaveAs)
            {
                // Save As is a host flow: the commit already happened, the path dialog is
                // the user's next step, and a busy file operation can only warn.
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    &event_command,
                    &format!("project save-as failed: {error}"),
                    None,
                );
            }
            if run.save
                && let Err(error) = self.begin_file_operation(crate::file_lifecycle::FileKind::Save)
            {
                // The command edits are committed; a failed save must not turn the
                // command into a failure. The document stays dirty and the user sees why.
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    &event_command,
                    &format!("project save failed: {error}"),
                    None,
                );
            }
            // Record the committed difference of this plugin command for the next one.
            if let Some(batch) = om9_core::changes::ChangeBatch::between(
                &before,
                self.document(),
                om9_core::changes::ChangeCause::Edit,
            )
            .ok()
            .flatten()
            {
                self.plugin_change = Some(batch);
            }
            Ok(run.result)
        })();
        self.events.emit(
            crate::command_events::EventKind::Finished,
            &event_command,
            if result.is_ok() {
                "plugin finished"
            } else {
                "plugin refused"
            },
            Some(if result.is_ok() {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            }),
        );
        result
    }

    /// Validate one staged plugin selection against a scene with the same rule
    /// `select_objects` uses: a canonical, selectable root with pick geometry.
    fn prepare_plugin_selection(&self, scene: &Scene, ids: &[ObjectId])
        -> Result<super::selection_memory::ReservedRootSelection> {
        self.prepare_root_selection(scene, ids, true).map_err(|error| match error {
            Error::Budget("root selection count") => Error::Budget("plugin selection"),
            Error::Permission("selected root") => Error::Permission("plugin selected root"),
            other => other,
        })
    }

    fn apply_plugin_selection(&mut self, selection: super::selection_memory::ReservedRootSelection) {
        self.publish_root_selection(selection);
        self.clear_subselection();
    }

    /// Apply a selection from a command that changed no document state.
    fn apply_plugin_selection_now(&mut self, ids: &[ObjectId]) -> Result<()> {
        let next = {
            let scene = self
                .scene
                .as_ref()
                .ok_or(Error::Missing("plugin selection scene"))?;
            if scene.epoch != self.coordinator.epoch() {
                return Err(Error::Stale("plugin selection scene"));
            }
            self.prepare_plugin_selection(scene, ids)?
        };
        self.apply_plugin_selection(next);
        Ok(())
    }

    /// Prepare the candidate, validate the staged selection against the scene that will
    /// be published, then commit. A bad selection refuses the command before publication.
    fn publish_plugin_request(
        &mut self,
        request: TransactionRequest,
        selection: Option<&[ObjectId]>,
    ) -> Result<()> {
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("plugin scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("plugin scene"));
        }
        let old_usage = self.scene_cache_usage()?;
        let mut prepared = self
            .coordinator
            .prepare_with_held_usage(request, old_usage)?;
        let candidate = prepared
            .candidate_world()
            .documents
            .get(&self.document)
            .ok_or(Error::Missing("plugin candidate document"))?
            .clone();
        if std::sync::Arc::ptr_eq(
            &candidate,
            &self.coordinator.world().documents[&self.document],
        ) {
            let staged = match selection {
                Some(ids) => Some(self.prepare_plugin_selection(old_scene, ids)?),
                None => None,
            };
            self.coordinator.commit(prepared)?;
            if let Some(selection) = staged {
                self.apply_plugin_selection(selection);
            }
            return Ok(());
        }
        // Keep the construction reservation alive through validation and
        // publication, so concurrent jobs see this scene while it is built.
        let candidate_scene = self.prepare_candidate_scene_guard(
            &mut prepared, old_usage, None,
            || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            |_| Ok(()),
        )?;
        let staged = match selection {
            Some(ids) => Some(self.prepare_plugin_selection(&candidate_scene.scene, ids)?),
            None => None,
        };
        let receipt = self.coordinator.commit(prepared)?;
        let super::ReservedCandidateScene { scene, buffer_owner, reservation } = candidate_scene;
        if receipt.published {
            self.publish_owned_scene(scene, buffer_owner);
            self.prune_selection();
            self.clear_subselection();
            self.clear_gumball_preview();
            self.sync_published(ChangeCause::Edit);
        } else {
            drop(scene);
            drop(buffer_owner);
        }
        if let Some(selection) = staged {
            self.apply_plugin_selection(selection);
        }
        drop(reservation);
        Ok(())
    }

    pub(super) fn submit_plugin_text(&mut self, text: &str) -> Option<Result<()>> {
        let (name, rest) = token(text)?;
        if name.eq_ignore_ascii_case("PluginLoad") {
            return Some((|| {
                let path: String = serde_json::from_str(rest).map_err(plugin_error)?;
                let descriptor = self.load_plugin(Path::new(&path))?;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    "PluginLoad",
                    &format!("loaded {}", descriptor.id),
                    None,
                );
                Ok(())
            })());
        }
        if !name.eq_ignore_ascii_case("PluginRun") {
            return None;
        }
        Some((|| {
            let (id, rest) = token(rest).ok_or(Error::Invalid("plugin ID required"))?;
            let (command, json) = token(rest).ok_or(Error::Invalid("plugin command required"))?;
            let arguments: Value = serde_json::from_str(json).map_err(plugin_error)?;
            if !arguments.is_object() {
                return Err(Error::Invalid("plugin arguments must be a JSON object"));
            }
            self.run_loaded_plugin(id, command, arguments).map(|_| ())
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        budget::Limits,
        geometry::{Curve, Geometry},
        identity::ObjectId,
        model::{Document, Entity},
        transaction::Mutation,
    };

    #[cfg(not(feature = "occt-brep"))]
    #[test]
    fn sdk_snap_query_binds_source_and_rejects_stale_context() {
        use om9_sdk::{HostRequest, SnapConstrain, SnapContext, SnapHit, SnapQuery};
        use om9_scene::camera::Viewport;
        let mut document = Document::with_template_tolerance("SDK line snap");
        let entity = Entity::new("line", Geometry::Curve(Curve::Line {
            start: [0., 0., 0.], end: [4., 0., 0.], domain: [0., 1.],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        let before = controller.coordinator().snapshot();
        let invocation = uuid::Uuid::new_v4();
        let context: SnapContext = serde_json::from_value(controller.plugin_snap_request("test", invocation, HostRequest::SnapContext).unwrap()).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
        let view = &controller.workspace.views[&controller.workspace.active];
        let cursor = view.camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
        let query = SnapQuery { context_token: context.token.clone(), viewport_origin: viewport.origin,
            viewport_size: viewport.size, cursor, radius_px: 8., kinds: vec!["end".into()], from_mm: None, tracking_anchors_mm: vec![] };
        let hit: SnapHit = serde_json::from_value(controller.plugin_snap_request("test", invocation, HostRequest::SnapQuery { query: query.clone() }).unwrap()).unwrap();
        assert_eq!(hit.root_id.as_deref(), Some(id.to_string().as_str()));
        assert_eq!(hit.point_mm, [0., 0., 0.]);
        assert_eq!(controller.plugin_snap_request("test", uuid::Uuid::new_v4(), HostRequest::SnapQuery { query: query.clone() }).unwrap_err().code, "stale");
        controller.set_grid_snap(true);
        controller.set_grid_snap_step_mm(0.5);
        let request = SnapConstrain { context_token: context.token.clone(), point_mm: [0.3, 0.7, 0.],
            base_mm: None, shift: false, object_snap: false };
        assert_eq!(controller.plugin_snap_request("test", uuid::Uuid::new_v4(), HostRequest::SnapConstrain { request: request.clone() }).unwrap_err().code, "stale");
        let point: [f64; 3] = serde_json::from_value(controller.plugin_snap_request("test", invocation, HostRequest::SnapConstrain { request: request.clone() }).unwrap()).unwrap();
        assert_eq!(point, [0.5, 0.5, 0.]);
        let point: [f64; 3] = serde_json::from_value(controller.plugin_snap_request("test", invocation, HostRequest::SnapConstrain {
            request: SnapConstrain { object_snap: true, ..request.clone() },
        }).unwrap()).unwrap();
        assert_eq!(point, [0.3, 0.7, 0.]);
        controller.set_planar(true);
        controller.set_project(true);
        for (base_mm, height) in [(None, 0.), (Some([0., 0., 3.]), 3.)] {
            let point: [f64; 3] = serde_json::from_value(controller.plugin_snap_request("test", invocation,
                HostRequest::SnapConstrain { request: SnapConstrain {
                    point_mm: [0.3, 0.7, 9.], object_snap: true, base_mm, ..request.clone()
                } },
            ).unwrap()).unwrap();
            assert_eq!(point, [0.3, 0.7, height]);
        }
        controller.set_cplane(om9_scene::camera::CPlane::preset(om9_scene::camera::CPlanePreset::Front)).unwrap();
        assert_eq!(controller.plugin_snap_request("test", invocation, HostRequest::SnapQuery { query }).unwrap_err().code, "stale");
        assert_eq!(controller.coordinator().snapshot(), before);
    }

    #[cfg(not(feature = "occt-brep"))]
    #[test]
    #[ignore = "requires OM9_TEST_PLUGIN_PACKAGE pointing to a freshly built hello-plugin package"]
    fn sdk_snap_demo_runs_through_loaded_plugin() {
        use om9_scene::camera::Viewport;
        let mut document = Document::with_template_tolerance("SDK loaded snap");
        let entity = Entity::new("line", Geometry::Curve(Curve::Line {
            start: [0., 0., 0.], end: [4., 0., 0.], domain: [0., 1.],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        let package = std::env::var_os("OM9_TEST_PLUGIN_PACKAGE").expect("plugin package path");
        controller.load_plugin(std::path::Path::new(&package)).unwrap();
        let before = controller.coordinator().snapshot();
        let before_document = controller.document().clone();
        let undo_before = controller.coordinator().undo_len();
        let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
        let view = &controller.workspace.views[&controller.workspace.active];
        let cursor = view.camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
        let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo",
            serde_json::json!({"cursor_x": cursor[0], "cursor_y": cursor[1]})).unwrap();
        assert_eq!(result["hit"]["root_id"], id.to_string());
        assert_eq!(result["point_mm"], serde_json::json!([0., 0., 0.]));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().undo_len(), undo_before);
        let created = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo",
            serde_json::json!({"cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true})).unwrap();
        let new_id = created["created"].as_str().expect("created point id");
        let point = controller.document().entities.values().find(|entity| entity.id.to_string() == new_id).unwrap();
        assert!(matches!(&point.geometry, Geometry::Point(p) if *p == [0., 0., 0.]));
        assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
        controller.undo().unwrap();
        assert_eq!(controller.document(), &before_document);
        assert_eq!(controller.coordinator().undo_len(), undo_before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.redo().unwrap();
        assert!(controller.document().entities.values().any(|entity| entity.id.to_string() == new_id));
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sdk_snap_query_retries_deferred_source_without_mutating_document() {
        use om9_core::geometry::Transform;
        use om9_sdk::{HostRequest, SnapContext, SnapHit, SnapQuery};
        use om9_scene::camera::Viewport;
        let curve = Curve::Circle { center: [0.; 3], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2.,
            domain: [0., std::f64::consts::FRAC_PI_2], angle_domain: None };
        let mut entity = Entity::new("arc", Geometry::Curve(curve), None);
        entity.placement = Transform::scale([3., 1., 1.]).unwrap();
        let id = entity.id;
        let mut document = Document::with_template_tolerance("SDK source snap");
        document.entities.insert(id, entity);
        let controller = Controller::new(document, Limits::default()).unwrap();
        let before = controller.coordinator().snapshot();
        let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
        let view = &controller.workspace.views[&controller.workspace.active];
        let cursor = view.camera.project([4., 1., 0.], viewport).unwrap().unwrap().screen;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut pending = false;
        let hit: SnapHit = loop {
            // Each retry models a new plugin invocation with its own context.
            let invocation = uuid::Uuid::new_v4();
            let context: SnapContext = serde_json::from_value(controller.plugin_snap_request("test", invocation, HostRequest::SnapContext).unwrap()).unwrap();
            let query = SnapQuery { context_token: context.token, viewport_origin: viewport.origin, viewport_size: viewport.size,
                cursor, radius_px: 128., kinds: vec!["mid".into()], from_mm: None, tracking_anchors_mm: vec![] };
            match controller.plugin_snap_request("test", invocation, HostRequest::SnapQuery { query }) {
                Ok(value) => break serde_json::from_value(value).unwrap(),
                Err(error) if error.code == "pending" => {
                    pending = true;
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => panic!("SDK snap query failed: {error}"),
            }
        };
        assert!(pending);
        assert_eq!(hit.root_id, Some(id.to_string()));
        assert_eq!(controller.coordinator().snapshot(), before);
    }

    #[test]
    fn staged_plugin_selection_transfers_its_owner_before_publication() {
        let mut controller = Controller::new(Document::with_template_tolerance("plugin selection owner"), Limits::default()).unwrap();
        let first = Entity::new("point", Geometry::Point([1., 0., 0.]), None);
        let second = Entity::new("point", Geometry::Point([2., 0., 0.]), None);
        let mut ids = vec![first.id, second.id];
        let request = TransactionRequest::new("plugin selection", vec![
            Mutation::Insert { document: controller.document, entity: first },
            Mutation::Insert { document: controller.document, entity: second },
        ]);
        controller.publish_plugin_request(request, Some(&ids)).unwrap();
        ids.sort_unstable();
        assert_eq!(controller.selected_ordered(), ids);
        let bytes = om9_core::budget::Usage::btree::<ObjectId, ()>(2).unwrap().bytes
            + om9_core::budget::Usage::array::<ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        assert_eq!(controller.selection_reservation.as_ref().unwrap().bytes(), bytes);
        assert_eq!(controller.selection_reservation.as_ref().unwrap().pending_bytes(), 0);
        controller.undo().unwrap();
        assert!(controller.selected().is_empty());
        let order_bytes = om9_core::budget::Usage::array::<ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        assert_eq!(controller.selection_reservation.as_ref().unwrap().bytes(), order_bytes);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        assert!(controller.selected().is_empty());
    }

    #[test]
    fn plugin_budget_refusal_preserves_scene_and_document() {
        let limits = Limits {
            max_points: 1,
            ..Limits::default()
        };
        let mut controller =
            Controller::new(Document::with_template_tolerance("budget"), limits).unwrap();
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let mutations = (0..2)
            .map(|n| Mutation::Insert {
                document: controller.document,
                entity: Entity::new(
                    "point",
                    Geometry::Point([n as f64, 0.0, 0.0]),
                    controller.document().current_layer,
                ),
            })
            .collect();
        assert!(matches!(
            controller
                .publish_plugin_request(TransactionRequest::new("over budget", mutations), None),
            Err(Error::Budget(_))
        ));
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert!(controller.scene().unwrap().objects.is_empty());
        assert_eq!(controller.coordinator().undo_len(), 0);
        assert_eq!(controller.coordinator().redo_len(), 0);
    }

    #[test]
    fn staged_plugin_batch_publishes_scene_in_one_undo_unit() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let line = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0.0; 3],
                end: [10.0, 0.0, 0.0],
                domain: [0.0, 10.0],
            }),
            controller.document().current_layer,
        );
        let polyline = Entity::new(
            "polyline",
            Geometry::Curve(
                om9_core::curve_build::polyline(&[[0.0; 3], [0.0, 5.0, 0.0], [5.0, 5.0, 0.0]])
                    .unwrap(),
            ),
            controller.document().current_layer,
        );
        let ids = [line.id, polyline.id];
        controller
            .publish_plugin_request(
                TransactionRequest::new(
                    "plugin batch",
                    vec![
                        Mutation::Insert {
                            document: controller.document,
                            entity: line,
                        },
                        Mutation::Insert {
                            document: controller.document,
                            entity: polyline,
                        },
                    ],
                ),
                None,
            )
            .unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.scene().unwrap().objects.len(), 2);
        assert_eq!(
            controller.scene().unwrap().epoch,
            controller.coordinator().epoch()
        );
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
        assert!(controller.scene().unwrap().objects.is_empty());
        controller.redo().unwrap();
        assert!(
            ids.iter()
                .all(|id| controller.document().entities.contains_key(id))
        );
        assert_eq!(controller.scene().unwrap().objects.len(), 2);
    }

    #[test]
    fn refused_plugin_batch_keeps_scene_document_and_redo() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let point = Entity::new(
            "point",
            Geometry::Point([0.0; 3]),
            controller.document().current_layer,
        );
        controller
            .publish_plugin_request(
                TransactionRequest::new(
                    "seed",
                    vec![Mutation::Insert {
                        document: controller.document,
                        entity: point,
                    }],
                ),
                None,
            )
            .unwrap();
        controller.undo().unwrap();
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let valid = Entity::new(
            "valid",
            Geometry::Point([1.0; 3]),
            controller.document().current_layer,
        );
        let invalid = Entity::new(
            "invalid",
            Geometry::Point([f64::NAN, 0.0, 0.0]),
            controller.document().current_layer,
        );
        assert!(
            controller
                .publish_plugin_request(
                    TransactionRequest::new(
                        "invalid batch",
                        vec![
                            Mutation::Insert {
                                document: controller.document,
                                entity: valid
                            },
                            Mutation::Insert {
                                document: controller.document,
                                entity: invalid
                            },
                        ]
                    ),
                    None
                )
                .is_err()
        );
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert!(controller.scene().unwrap().objects.is_empty());
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
    }

    #[test]
    fn plugin_selection_is_validated_before_publication_and_applied_after() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("selection"),
            Limits::default(),
        )
        .unwrap();
        let first = Entity::new(
            "first",
            Geometry::Point([0.0; 3]),
            controller.document().current_layer,
        );
        let second = Entity::new(
            "second",
            Geometry::Point([1.0; 3]),
            controller.document().current_layer,
        );
        let ids = [first.id, second.id];
        let request = TransactionRequest::new(
            "insert pair",
            vec![
                Mutation::Insert {
                    document: controller.document,
                    entity: first,
                },
                Mutation::Insert {
                    document: controller.document,
                    entity: second,
                },
            ],
        );
        controller
            .publish_plugin_request(request, Some(&ids))
            .unwrap();
        assert_eq!(
            controller.selected(),
            &ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert_eq!(controller.coordinator().undo_len(), 1);

        // A selection that names an unknown root refuses the whole command.
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let live_buffers = controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
        let stranger = Entity::new(
            "stranger",
            Geometry::Point([2.0; 3]),
            controller.document().current_layer,
        );
        let stranger_id = stranger.id;
        let request = TransactionRequest::new(
            "unknown selection",
            vec![Mutation::Insert {
                document: controller.document,
                entity: stranger,
            }],
        );
        let bogus = [ObjectId(uuid::Uuid::new_v4())];
        assert!(
            controller
                .publish_plugin_request(request, Some(&bogus))
                .is_err()
        );
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert_eq!(controller.coordinator().undo_len(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(
            om9_core::budget::memory::MemoryClass::CpuCache), live_buffers);
        assert!(!controller.document().entities.contains_key(&stranger_id));
        assert_eq!(
            controller.selected(),
            &ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        );
    }
    #[test]
    fn cmd_plugin_errors_preserve_document_and_existing_command() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let before = controller.document().clone();
        for command in [
            "PluginRun",
            "PluginRun missing command []",
            "PluginLoad not-json",
            "PluginRun missing command {}",
        ] {
            assert!(controller.submit(command).is_err());
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
        controller.submit("Line").unwrap();
        assert!(controller.submit("PluginLoad \"missing.om9p\"").is_err());
        assert!(controller.is_running());
        assert_eq!(controller.document(), &before);
    }
}
