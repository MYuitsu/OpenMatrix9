//! The viewport's GPU surface bridge (M01-L2-01-06, surface integration).
//!
//! `om9-render` draws into textures; this module gives those textures to egui so the running application
//! shows the real GPU frame inside a viewport rect, without a CPU readback: the renderer draws into a
//! colour+depth pair, the colour view is registered with egui's wgpu renderer once, and every frame the
//! viewport paints that texture as an image.
//!
//! Always active in the desktop app. Native BRep construction remains an independent optional capability.

use eframe::egui;
use eframe::egui_wgpu::RenderState;
use eframe::wgpu;
use om9_render::{Camera, MeshData};
use om9_scene::workspace::DisplayMode;

type HiddenSources = Option<[om9_core::identity::ObjectId; 2]>;

struct TrackedPresentationSnapshot {
    // Scene has its own allocation/lease; an expired Weak retains only this
    // small presentation block, not the Scene body.
    scene: Box<om9_scene::scene::Scene>,
    _document: om9_core::transaction::CapturedDocument,
    _buffers: std::sync::Arc<crate::session::SceneBufferLease>,
    _allocation: PresentationAllocation,
}
type PresentationKey = (om9_core::identity::DocumentId, usize, u64, HiddenSources);
type PresentationAllocation = std::sync::Arc<om9_core::transaction::ExternalMemoryReservation>;
// Weak must drop before its shared allocation owner when an entry retires.
type PresentationCache = std::sync::Arc<std::sync::Mutex<Option<(PresentationKey, std::sync::Weak<TrackedPresentationSnapshot>, PresentationAllocation)>>>;

fn presentation_snapshot_overhead() -> u64 {
    // Scene storage is independent. The presentation allocation and the Arc
    // owning its allowance stay charged through the last cached Weak.
    (std::mem::size_of::<TrackedPresentationSnapshot>()
        + std::mem::size_of::<om9_core::transaction::ExternalMemoryReservation>()
        + 8 * std::mem::size_of::<usize>()) as u64
}

fn presentation_snapshot(
    cache: &PresentationCache,
    controller: &crate::session::Controller,
    scene: &om9_scene::scene::Scene,
    _preview: u64,
    hidden: HiddenSources,
) -> om9_core::Result<std::sync::Arc<TrackedPresentationSnapshot>> {
    if !controller.coordinator().world().documents.get(&scene.document.id)
        .is_some_and(|document| std::sync::Arc::ptr_eq(document, &scene.document)) {
        return Err(om9_core::Error::Stale("viewport scene document"));
    }
    let key = (scene.document.id, std::sync::Arc::as_ptr(&scene.document) as usize, scene.epoch, hidden);
    let mut cached = cache.lock().unwrap_or_else(|e| e.into_inner());
    if cached.as_ref().is_some_and(|(_, weak, _)| weak.strong_count() == 0) {
        // Retire the actual expired allocation before admitting its successor.
        drop(cached.take());
    }
    if let Some((previous, weak, _)) = &*cached
        && *previous == key
        && let Some(snapshot) = weak.upgrade()
        && snapshot._document.matches_weak(&std::sync::Arc::downgrade(&scene.document)) {
        return Ok(snapshot);
    }
    let coordinator = controller.coordinator();
    let capture = coordinator.capture_tracked_document(scene.document.id)?;
    if !capture.matches_weak(&std::sync::Arc::downgrade(&scene.document)) {
        return Err(om9_core::Error::Stale("viewport scene document"));
    }
    // The live scene and its snapshots share one lease for render buffers.
    // Snapshot metadata is independent and has its own lease.
    let buffers = controller.share_scene_buffer_owner(scene)?;
    let overhead = presentation_snapshot_overhead();
    let arena = scene.picking_arena_bytes()?;
    let metadata_bytes = scene.render_snapshot_metadata_estimate()?.bytes.checked_add(overhead).and_then(|bytes| bytes.checked_add(arena))
        .ok_or(om9_core::Error::Overflow("viewport snapshot accounting"))?;
    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
    let mut reservation = coordinator.reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, metadata_bytes.max(1),
        physical, controller.scene_cache_usage()?,
    )?;
    let mut snapshot = Box::new(scene.try_render_snapshot()?);
    // GPU preparation builds the untransformed scene. The live preview is
    // applied separately at draw time and must not create a second snapshot.
    for object in &mut snapshot.objects { object.preview = om9_core::geometry::Transform::identity(); }
    if let Some(hidden) = hidden {
        snapshot.objects.retain(|object| !hidden.contains(&object.root));
    }
    let arena = snapshot.picking_arena_bytes()?;
    let actual = snapshot.metadata_storage_usage()?.bytes.checked_add(overhead).and_then(|bytes| bytes.checked_add(arena))
        .ok_or(om9_core::Error::Overflow("viewport snapshot accounting"))?;
    if actual > reservation.bytes() {
        let additional = actual - reservation.bytes();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        coordinator.grow_external_memory_with_held(&mut reservation, additional, physical, controller.scene_cache_usage()?)?;
    }
    reservation.shrink_to(actual.max(1))?;
    snapshot.transfer_memory_allowance(&mut reservation, coordinator, controller.scene_cache_usage()?)?;
    let allocation = std::sync::Arc::new(reservation);
    let mut snapshot = std::sync::Arc::new(TrackedPresentationSnapshot { scene: snapshot, _document: capture, _buffers: buffers, _allocation: allocation });
    let allocation = &mut std::sync::Arc::get_mut(&mut snapshot).expect("new presentation snapshot")._allocation;
    let reservation = std::sync::Arc::get_mut(allocation).expect("new presentation allocation");
    reservation.mark_allocated(reservation.pending_bytes())?;
    *cached = Some((key, std::sync::Arc::downgrade(&snapshot), snapshot._allocation.clone()));
    Ok(snapshot)
}

/// One viewport's GPU target and its registration in egui.
pub struct GpuViewport {
    diagnostics: Option<ViewportDiagnostics>,
    #[cfg(test)]
    target_ceiling_override: Option<u64>,
    #[cfg(test)]
    legacy_world_lines: bool,
    size: (u32, u32),
    colour: Option<wgpu::Texture>,
    depth: Option<wgpu::Texture>,
    colour_view: Option<wgpu::TextureView>,
    depth_view: Option<wgpu::TextureView>,
    /// Multisampled pair of the document viewport. Every pass draws into these and resolves into
    /// `colour_view`, so egui keeps one ordinary texture while the grid and the mesh edges are antialiased.
    msaa_colour: Option<wgpu::Texture>,
    msaa_colour_view: Option<wgpu::TextureView>,
    msaa_depth: Option<wgpu::Texture>,
    msaa_depth_view: Option<wgpu::TextureView>,
    texture_id: Option<egui::TextureId>,
    /// Camera revision of the last rendered frame; the pass is skipped when it did not change.
    last_revision: Option<u64>,
    display_mode: DisplayMode,
    last_mode: Option<DisplayMode>,
    last_camera: Option<om9_scene::camera::Camera>,
    /// Scene epoch and CPlane revision of the last rendered frame: new geometry must repaint even when the
    /// camera did not move.
    last_epoch: Option<u64>,
    last_cplane: Option<u64>,
    /// Selection revision of the last render: the highlight is part of the GPU image.
    last_selection: Option<u64>,
    /// Revision of the transient gumball preview: a drag must repaint every viewport.
    last_preview: Option<u64>,
    last_hidden_sources: Option<HiddenSources>,
    /// Stress scene (`OM9_GPU_STRESS=N`): a batch of N boxes sharing one arena, instead of the sphere.
    batch: Option<om9_render::scene::SceneBatch>,
    /// The same stress scene through the GPU cluster culler, when `OM9_GPU_CLUSTER` is set.
    cluster: Option<om9_render::cluster::ClusterRenderer>,
    /// The document scene drawn on the GPU, shared by all four viewports so its geometry is uploaded once.
    shared_scene: SharedScene,
    /// CPU time of the last scene sync (prep/upload) and the last draw submission, for the split log.
    last_prepare_ms: f64,
    last_draw_ms: f64,
    /// True while a manual gesture is animating; avoids rebuilding the screen-sized point overlay
    /// on every zoom frame.
    moving: bool,
    request: ViewportRenderRequest,
    pool: SharedResourcePool,
    /// Last nonblocking inventory supplied by the window before this pane draws.
    window_known_gpu_bytes: Option<Option<u64>>,
    shared_world_lines: Option<SharedWorldLines>,
    overlays: Option<ViewOverlays>,
    overlay_job: Option<(OverlayKey, PrepareReceiver<ViewOverlays>)>,
    published_id: Option<egui::TextureId>,
    published_colour: Option<wgpu::Texture>,
    spare_colour: Option<(egui::TextureId, wgpu::Texture)>,
    last_profile: Option<(u32, bool)>,
    rendered_final: bool,
    preparing: bool,
    preparation_error: Option<String>,
    snapshot_retry: Option<(PresentationKey, std::time::Instant)>,
    last_document: Option<om9_core::identity::DocumentId>,
    displayed_camera: Option<om9_scene::camera::Camera>,
    /// Document and scene that produced the published texture, not the latest encoded frame.
    displayed_scene: Option<(om9_core::identity::DocumentId, u64)>,
    displayed_key: Option<RefinementKey>,
    refinement: Option<(RefinementKey, std::time::Instant)>,
    refinement_ms: Option<f64>,
    timer: Option<om9_render::timing::GpuTimer>,
    timer_frame: Option<u64>,
    pub gpu_sample: Option<(u64, f64)>,
    pub encoded_frame: Option<u64>,
    render_stats: Option<om9_render::scene::FrameStats>,
    pub published_frame: Option<u64>,
    pending_publication: Option<PendingPublication>,
    hold_publication_on_press: bool,
    /// One encoded document frame per viewport until the queue completes it.
    /// Queue completion does not establish desktop presentation.
    frame_submission: Option<FrameSubmission>,
}

/// Per-viewport counters, enabled only for an explicit diagnostic session.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct ViewportDiagnostics {
    pub target_rebuilds: u64,
    pub scene_preparations: u64,
    pub overlay_preparations: u64,
    pub geometry_upload_bytes: u64,
    pub overlay_upload_bytes: u64,
    /// Cumulative buffer initialization for the shaded main scene pass only.
    pub main_scene_camera_snapshot_bytes: u64,
    /// Unique scene camera uniform buffers encoded/submitted by the window.
    pub encoded_camera_bytes: Option<u64>,
    pub inflight_camera_bytes: Option<u64>,
    pub tracked_camera_bytes: Option<u64>,
    /// Excludes retained mesh geometry, ghost passes and line overlays.
    pub main_scene_instance_snapshot_bytes: u64,
    pub encoded_frames: u64,
    pub cached_frames: u64,
    pub readiness_checks: u64,
    /// Logical sizes of known live colour/depth targets; excludes driver padding and retired in-flight targets.
    pub known_target_logical_bytes: u64,
    /// Scene buffers are shared across views; do not sum this value across viewports.
    pub shared_scene_buffer_bytes: Option<u64>,
    /// Logical visible-index bytes retained by scene snapshot caches; shared across views.
    pub shared_scene_instance_snapshot_cpu_bytes: Option<u64>,
    /// Unique instance buffers in submitted work; shared across viewports and
    /// may overlap shared_scene_buffer_bytes, so the two must not be summed.
    pub inflight_instance_bytes: Option<u64>,
    pub inflight_instance_buffers: Option<usize>,
    /// Encoded but not yet submitted buffers and the deduplicated union of
    /// encoded and submitted buffers; both may overlap the live scene cache.
    pub encoded_instance_bytes: Option<u64>,
    pub tracked_instance_bytes: Option<u64>,
    /// Unique live cache, encoded and submitted instance buffers in this
    /// window pool. None means unavailable; excludes all other GPU resources.
    pub unique_instance_bytes: Option<u64>,
    /// Unique main/ghost geometry arenas across live, encoded and submitted
    /// scenes; excludes overlays, upload staging and other GPU owners.
    pub unique_geometry_bytes: Option<u64>,
    pub inflight_geometry_bytes: Option<u64>,
    pub inflight_geometry_buffers: Option<usize>,
    /// Logical target sizes held by viewport render commands in this pool;
    /// they overlap live targets and exclude later eframe compositing work.
    pub encoded_target_bytes: Option<u64>,
    pub inflight_target_bytes: Option<u64>,
    pub tracked_target_bytes: Option<u64>,
    pub inflight_target_count: Option<usize>,
    pub overlay_buffer_bytes: u64,
    /// Vertex/uniform buffers encoded or submitted by the window; may overlap live overlays.
    pub encoded_overlay_bytes: Option<u64>,
    pub inflight_overlay_bytes: Option<u64>,
    pub tracked_overlay_bytes: Option<u64>,
}
struct PendingPublication {
    key: RefinementKey,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    submitted: bool,
    texture: egui::TextureId,
    colour: wgpu::Texture,
    camera: om9_scene::camera::Camera,
    frame_id: u64,
}
struct FrameSubmission {
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    submitted: bool,
}

/// An image is complete only for this exact source and camera snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RefinementKey {
    document: om9_core::identity::DocumentId,
    scene: u64,
    camera: u64,
    selection: u64,
    cplane: u64,
    preview: u64,
    hidden_sources: HiddenSources,
    mode: DisplayMode,
    size: (u32, u32),
    generation: u64,
}

impl RefinementKey {
    fn same_content(self, other: Self) -> bool {
        self.document == other.document && self.scene == other.scene
            && self.selection == other.selection && self.cplane == other.cplane
            && self.preview == other.preview && self.hidden_sources == other.hidden_sources
            && self.size == other.size && self.mode == other.mode
    }
    pub(crate) fn same_input_basis(self, other: Self) -> bool {
        self.document == other.document && self.scene == other.scene && self.camera == other.camera
            && self.selection == other.selection && self.cplane == other.cplane
            && self.preview == other.preview && self.hidden_sources == other.hidden_sources
            && self.size == other.size && self.mode == other.mode
    }
}

/// The one document scene the four viewports render with their own cameras.
pub type SharedScene = std::sync::Arc<std::sync::Mutex<Option<GpuScene>>>;
pub type SharedResourcePool = std::sync::Arc<std::sync::Mutex<ResourcePool>>;
type SharedWorldLines = std::sync::Arc<std::sync::Mutex<WorldLines>>;
/// Logical target sizes for one window, deduplicated by texture handle.
/// These are not physical VRAM allocations or presentation ownership.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct WindowTargetDiagnostics {
    pub live_bytes: u64,
    pub live_count: usize,
    pub retired_command_bytes: u64,
    pub retired_command_count: usize,
    pub unique_bytes: u64,
    pub unique_count: usize,
}

/// Call only for an explicitly requested diagnostic snapshot. Every viewport
/// must belong to this pool; unavailable locks or unknown formats return None.
pub fn window_target_diagnostics<'a>(
    pool: &SharedResourcePool,
    ports: impl IntoIterator<Item = &'a GpuViewport>,
) -> Option<WindowTargetDiagnostics> {
    let mut textures: Vec<wgpu::Texture> = Vec::new();
    let mut add = |texture: &wgpu::Texture| -> Option<()> {
        if window_target_bytes(texture).is_none() {
            return None;
        }
        if !textures.contains(texture) { textures.push(texture.clone()); }
        Some(())
    };
    for port in ports {
        if !std::sync::Arc::ptr_eq(pool, &port.pool) { return None; }
        for texture in [port.colour.as_ref(), port.depth.as_ref(), port.msaa_colour.as_ref(),
            port.msaa_depth.as_ref(), port.published_colour.as_ref(),
            port.spare_colour.as_ref().map(|(_, texture)| texture),
            port.pending_publication.as_ref().map(|pending| &pending.colour)].into_iter().flatten() {
            add(texture)?;
        }
    }
    drop(add);
    let live_count = textures.len();
    let live_bytes = textures.iter().try_fold(0u64, |sum, texture| sum.checked_add(window_target_bytes(texture)?))?;
    let pool = pool.try_lock().ok()?;
    let inflight = pool.inflight_targets.try_lock().ok()?;
    for texture in pool.encoded_targets.iter().chain(inflight.resources.iter().map(|(texture, _)| texture)) {
        if window_target_bytes(texture).is_none() { return None; }
        if !textures.contains(texture) { textures.push(texture.clone()); }
    }
    let unique_bytes = textures.iter().try_fold(0u64, |sum, texture| sum.checked_add(window_target_bytes(texture)?))?;
    Some(WindowTargetDiagnostics {
        live_bytes, live_count,
        retired_command_bytes: unique_bytes.checked_sub(live_bytes)?,
        retired_command_count: textures.len().saturating_sub(live_count),
        unique_bytes, unique_count: textures.len(),
    })
}

/// Deduplicate known scene, overlay and camera buffers across live viewports,
/// encoded commands and in-flight submissions. Worker-owned buffers are not included.
pub fn window_known_buffer_bytes<'a>(pool: &SharedResourcePool, ports: impl IntoIterator<Item = &'a GpuViewport>) -> Option<u64> {
    let account = pool;
    let pool = account.try_lock().ok()?;
    let geometry = pool.inflight_geometry.try_lock().ok()?;
    let instances = pool.inflight_instances.try_lock().ok()?;
    let cameras = pool.inflight_cameras.try_lock().ok()?;
    let overlays_inflight = pool.inflight_overlays.try_lock().ok()?;
    if pool.preparations.active.load(std::sync::atomic::Ordering::Acquire) != 0 { return None; }
    let pending = pool.preparations.completed.try_lock().ok()?;
    let mut buffers = Vec::<wgpu::Buffer>::new();
    let mut add = |buffer: &wgpu::Buffer| {
        if !buffers.contains(buffer) { buffers.push(buffer.clone()); }
    };
    for weak in &pool.known_scenes {
        let Some(scene) = weak.upgrade() else { continue };
        let guard = scene.try_lock().ok()?;
        if let Some(gpu) = guard.as_ref() {
            for batch in std::iter::once(&gpu.batch).chain(gpu.ghost.iter()) {
                batch.for_each_geometry_buffer(&mut add);
                if let Some(buffer) = batch.instance_snapshot_buffer() { add(&buffer); }
            }
        }
    }
    for entry in pool.world_lines.values() {
        if let Some(world) = &entry.batch {
            let world = world.try_lock().ok()?;
            for batch in world.batches() { for buffer in batch.gpu_buffer_handles() { add(buffer); } }
        }
    }
    for buffer in pool.encoded_geometry.iter().chain(&pool.encoded_instances)
        .chain(&pool.encoded_cameras).chain(&pool.encoded_overlays)
        .chain(geometry.resources.iter().map(|(buffer, _)| buffer))
        .chain(instances.resources.iter().map(|(buffer, _)| buffer))
        .chain(cameras.resources.iter().map(|(buffer, _)| buffer))
        .chain(overlays_inflight.resources.iter().map(|(buffer, _)| buffer)) { add(buffer); }
    for weak in pending.iter() {
        if let Some(held) = weak.upgrade() { for buffer in held.iter() { add(buffer); } }
    }
    for port in ports {
        if !std::sync::Arc::ptr_eq(account, &port.pool) { return None; }
        if let Some(overlays) = &port.overlays {
            for batch in [&overlays.fixed, &overlays.moving, &overlays.ghost, &overlays.selected_guides] {
                for buffer in batch.gpu_buffer_handles() { add(buffer); }
            }
        }
        if let Some(world) = &port.shared_world_lines {
            let world = world.try_lock().ok()?;
            for batch in world.batches() { for buffer in batch.gpu_buffer_handles() { add(buffer); } }
        }
    }
    buffers.iter().try_fold(0u64, |sum, buffer| sum.checked_add(buffer.size()))
}

/// Deduplicated overlay vertex buffers and encoded/submitted overlay buffers.
/// Supply every viewport in the window. Worker-held buffers are not included.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct WindowOverlayDiagnostics {
    pub live_bytes: u64,
    pub live_count: usize,
    pub retired_command_bytes: u64,
    pub retired_command_count: usize,
    pub unique_bytes: u64,
    pub unique_count: usize,
}

pub fn window_overlay_diagnostics<'a>(
    pool: &SharedResourcePool,
    ports: impl IntoIterator<Item = &'a GpuViewport>,
) -> Option<WindowOverlayDiagnostics> {
    let mut buffers = Vec::<wgpu::Buffer>::new();
    for port in ports {
        if !std::sync::Arc::ptr_eq(pool, &port.pool) { return None; }
        if let Some(overlays) = &port.overlays {
            for batch in [&overlays.fixed, &overlays.moving, &overlays.ghost, &overlays.selected_guides] {
                for buffer in batch.gpu_buffer_handles() {
                    if !buffers.contains(buffer) { buffers.push(buffer.clone()); }
                }
            }
        }
        if let Some(world) = &port.shared_world_lines {
            let world = world.try_lock().ok()?;
            for batch in world.batches() { for buffer in batch.gpu_buffer_handles() {
                if !buffers.contains(buffer) { buffers.push(buffer.clone()); }
            } }
        }
    }
    let live_count = buffers.len();
    let live_bytes = buffers.iter().try_fold(0u64, |sum, buffer| sum.checked_add(buffer.size()))?;
    let pool = pool.try_lock().ok()?;
    let inflight = pool.inflight_overlays.try_lock().ok()?;
    for buffer in pool.encoded_overlays.iter().chain(inflight.resources.iter().map(|(buffer, _)| buffer)) {
        if !buffers.contains(buffer) { buffers.push(buffer.clone()); }
    }
    let unique_bytes = buffers.iter().try_fold(0u64, |sum, buffer| sum.checked_add(buffer.size()))?;
    Some(WindowOverlayDiagnostics {
        live_bytes, live_count,
        retired_command_bytes: unique_bytes.checked_sub(live_bytes)?,
        retired_command_count: buffers.len().checked_sub(live_count)?,
        unique_bytes, unique_count: buffers.len(),
    })
}

#[derive(Default)]
pub struct ResourcePool {
    encoder: Option<wgpu::CommandEncoder>,
    budget: Option<UploadBudget>,
    signature: Option<(om9_core::identity::DocumentId, u64, u64, HiddenSources)>,
    presentation: PresentationCache,
    entries: std::collections::BTreeMap<u32, ResourceEntry>,
    world_lines: std::collections::BTreeMap<u32, WorldLinesEntry>,
    encoded_instances: Vec<wgpu::Buffer>,
    inflight_instances: std::sync::Arc<std::sync::Mutex<InFlightInstances>>,
    encoded_cameras: Vec<wgpu::Buffer>,
    inflight_cameras: std::sync::Arc<std::sync::Mutex<InFlightInstances>>,
    encoded_geometry: Vec<wgpu::Buffer>,
    inflight_geometry: std::sync::Arc<std::sync::Mutex<InFlightInstances>>,
    encoded_overlays: Vec<wgpu::Buffer>,
    inflight_overlays: std::sync::Arc<std::sync::Mutex<InFlightInstances>>,
    encoded_targets: Vec<wgpu::Texture>,
    inflight_targets: std::sync::Arc<std::sync::Mutex<InFlightTargets>>,
    known_scenes: Vec<std::sync::Weak<std::sync::Mutex<Option<GpuScene>>>>,
    preparations: std::sync::Arc<PreparationInventory>,
}

struct InFlightResources<T> {
    resources: Vec<(T, usize)>,
}
impl<T> Default for InFlightResources<T> {
    fn default() -> Self { Self { resources: Vec::new() } }
}
impl<T: Clone + PartialEq> InFlightResources<T> {
    fn retain_submission(&mut self, resources: &[T]) {
        for resource in resources {
            if let Some((_, count)) = self.resources.iter_mut().find(|(held, _)| held == resource) {
                *count += 1;
            } else {
                self.resources.push((resource.clone(), 1));
            }
        }
    }
    fn complete_submission(&mut self, resources: &[T]) {
        for resource in resources {
            if let Some(index) = self.resources.iter().position(|(held, _)| held == resource) {
                self.resources[index].1 -= 1;
                if self.resources[index].1 == 0 { self.resources.swap_remove(index); }
            }
        }
    }
}
type InFlightInstances = InFlightResources<wgpu::Buffer>;
type InFlightTargets = InFlightResources<wgpu::Texture>;
/// Minimum logical bytes for two published/drawing colours and the depth/MSAA
/// attachments of one viewport. Excludes geometry, staging and driver overhead.
fn viewport_target_minimum_bytes(width: u32, height: u32, samples: u32) -> Option<u64> {
    if width == 0 || height == 0 || samples == 0 { return None; }
    let attachments = if samples > 1 { 2u64.checked_add(u64::from(samples).checked_mul(2)?)? } else { 3 };
    u64::from(width).checked_mul(u64::from(height))?.checked_mul(4)?.checked_mul(attachments)
}
// Only this renderer's RGBA8 and Depth32Float 2D targets are registered here.
fn is_known_window_target(texture: &wgpu::Texture) -> bool {
    matches!(texture.format(), wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Depth32Float)
        && texture.dimension() == wgpu::TextureDimension::D2
        && texture.depth_or_array_layers() == 1
        && texture.mip_level_count() == 1
}
fn window_target_bytes(texture: &wgpu::Texture) -> Option<u64> {
    if !is_known_window_target(texture) { return None; }
    u64::from(texture.width()).checked_mul(u64::from(texture.height()))?
        .checked_mul(u64::from(texture.sample_count()))?.checked_mul(4)
}
fn target_logical_bytes(texture: &wgpu::Texture) -> u64 {
    u64::from(texture.width()).saturating_mul(u64::from(texture.height()))
        .saturating_mul(u64::from(texture.sample_count())).saturating_mul(4)
}
impl InFlightInstances {
    fn totals(&self) -> (u64, usize) {
        (self.resources.iter().fold(0u64, |sum, (buffer, _)| sum.saturating_add(buffer.size())), self.resources.len())
    }
}
impl InFlightTargets {
    fn totals(&self) -> (u64, usize) {
        (self.resources.iter().fold(0u64, |sum, (texture, _)| sum.saturating_add(target_logical_bytes(texture))), self.resources.len())
    }
}
impl ResourcePool {
    fn track_encoded_instance(&mut self, buffer: wgpu::Buffer) {
        if !self.encoded_instances.contains(&buffer) { self.encoded_instances.push(buffer); }
    }
    fn track_encoded_camera(&mut self, buffer: wgpu::Buffer) {
        if !self.encoded_cameras.contains(&buffer) { self.encoded_cameras.push(buffer); }
    }
    fn track_encoded_overlay(&mut self, buffer: wgpu::Buffer) {
        if !self.encoded_overlays.contains(&buffer) { self.encoded_overlays.push(buffer); }
    }
    fn track_encoded_target(&mut self, texture: wgpu::Texture) {
        if !self.encoded_targets.contains(&texture) { self.encoded_targets.push(texture); }
    }
    fn unique_instance_bytes(&self, inflight: &InFlightInstances) -> Option<u64> {
        let mut buffers: Vec<wgpu::Buffer> = Vec::new();
        let mut add = |buffer: wgpu::Buffer| {
            if !buffers.contains(&buffer) { buffers.push(buffer); }
        };
        for weak in &self.known_scenes {
            let Some(scene) = weak.upgrade() else { continue };
            let guard = scene.try_lock().ok()?;
            if let Some(gpu) = guard.as_ref() {
                if let Some(buffer) = gpu.batch.instance_snapshot_buffer() { add(buffer); }
                if let Some(buffer) = gpu.ghost.as_ref().and_then(|ghost| ghost.instance_snapshot_buffer()) { add(buffer); }
            }
        }
        for buffer in &self.encoded_instances { add(buffer.clone()); }
        for (buffer, _) in &inflight.resources { add(buffer.clone()); }
        Some(buffers.iter().fold(0u64, |sum, buffer| sum.saturating_add(buffer.size())))
    }
    fn unique_geometry_bytes(&self, inflight: &InFlightInstances) -> Option<u64> {
        let mut buffers: Vec<wgpu::Buffer> = Vec::new();
        let mut add = |buffer: wgpu::Buffer| {
            if !buffers.contains(&buffer) { buffers.push(buffer); }
        };
        for weak in &self.known_scenes {
            let Some(scene) = weak.upgrade() else { continue };
            let guard = scene.try_lock().ok()?;
            if let Some(gpu) = guard.as_ref() {
                for buffer in gpu.batch.geometry_buffer_handles() { add(buffer); }
                if let Some(ghost) = &gpu.ghost {
                    for buffer in ghost.geometry_buffer_handles() { add(buffer); }
                }
            }
        }
        for buffer in &self.encoded_geometry { add(buffer.clone()); }
        for (buffer, _) in &inflight.resources { add(buffer.clone()); }
        Some(buffers.iter().fold(0u64, |sum, buffer| sum.saturating_add(buffer.size())))
    }
}
pub fn submit_window(pool: &SharedResourcePool, state: &RenderState) -> bool {
    let (encoder, buffers, inflight, cameras, inflight_cameras, geometry, inflight_geometry, overlays, inflight_overlays, targets, inflight_targets) = {
        let mut pool = pool.lock().unwrap_or_else(|e| e.into_inner());
        (pool.encoder.take(), std::mem::take(&mut pool.encoded_instances), pool.inflight_instances.clone(),
            std::mem::take(&mut pool.encoded_cameras), pool.inflight_cameras.clone(),
            std::mem::take(&mut pool.encoded_geometry), pool.inflight_geometry.clone(),
            std::mem::take(&mut pool.encoded_overlays), pool.inflight_overlays.clone(),
            std::mem::take(&mut pool.encoded_targets), pool.inflight_targets.clone())
    };
    if let Some(encoder) = encoder {
        if !buffers.is_empty() {
            inflight.lock().unwrap_or_else(|e| e.into_inner()).retain_submission(&buffers);
        }
        if !cameras.is_empty() {
            inflight_cameras.lock().unwrap_or_else(|e| e.into_inner()).retain_submission(&cameras);
        }
        if !geometry.is_empty() {
            inflight_geometry.lock().unwrap_or_else(|e| e.into_inner()).retain_submission(&geometry);
        }
        if !overlays.is_empty() {
            inflight_overlays.lock().unwrap_or_else(|e| e.into_inner()).retain_submission(&overlays);
        }
        if !targets.is_empty() {
            inflight_targets.lock().unwrap_or_else(|e| e.into_inner()).retain_submission(&targets);
        }
        state.queue.submit([encoder.finish()]);
        if !buffers.is_empty() || !cameras.is_empty() || !geometry.is_empty() || !overlays.is_empty() || !targets.is_empty() {
            state.queue.on_submitted_work_done(move || {
                inflight.lock().unwrap_or_else(|e| e.into_inner()).complete_submission(&buffers);
                inflight_cameras.lock().unwrap_or_else(|e| e.into_inner()).complete_submission(&cameras);
                inflight_geometry.lock().unwrap_or_else(|e| e.into_inner()).complete_submission(&geometry);
                inflight_overlays.lock().unwrap_or_else(|e| e.into_inner()).complete_submission(&overlays);
                inflight_targets.lock().unwrap_or_else(|e| e.into_inner()).complete_submission(&targets);
            });
        }
        true
    } else {
        false
    }
}
struct UploadBudget {
    frame: u64,
    left: usize,
    deadline: std::time::Instant,
}
struct ResourceEntry {
    scene: SharedScene,
    job: Option<PrepareReceiver<GpuScene>>,
    failed: bool,
}
#[derive(Default)]
struct WorldLinesEntry {
    batch: Option<SharedWorldLines>,
    job: Option<PrepareReceiver<WorldLines>>,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ViewportRenderRequest {
    pub frame_id: u64,
    pub generation: u64,
    pub moving: bool,
    pub render_due: bool,
    pub quality: crate::frame_scheduler::PreviewQuality,
}
fn target_size(rect: egui::Rect, pixels_per_point: f32, request: ViewportRenderRequest) -> (u32, u32) {
    let scale = if request.moving { request.quality.scale() } else { 1. };
    ((rect.width() * pixels_per_point * scale).round().max(1.0) as u32,
     (rect.height() * pixels_per_point * scale).round().max(1.0) as u32)
}
impl Default for ViewportRenderRequest {
    fn default() -> Self {
        Self {
            frame_id: 0,
            generation: 0,
            moving: false,
            render_due: true,
            quality: Default::default(),
        }
    }
}
type OverlayKey = (om9_core::identity::DocumentId, u64, u64, u64, u32, u64, HiddenSources);

#[cfg(test)]
mod presentation_snapshot_tests {
    use super::*;
    use om9_core::{budget::{Limits, memory::MemoryClass}, model::Document};

    #[test]
    fn expired_presentation_weak_keeps_its_allocation_charged_until_cache_prunes_it() {
        let controller = crate::session::Controller::new(Document::with_template_tolerance("expired frame body"),
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        let cache = PresentationCache::default();
        controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let snapshot = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        let live = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        drop(snapshot);
        assert_eq!(cache.lock().unwrap().as_ref().unwrap().1.strong_count(), 0);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline + presentation_snapshot_overhead(),
            "Weak still owns the presentation allocation after scene data has dropped");
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) < live,
            "the expired header must not retain scene data or its arena quota");
        drop(cache.lock().unwrap().take());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline);
    }

    #[test]
    fn presentation_allocation_survives_cache_drop_until_the_last_reader() {
        let controller = crate::session::Controller::new(Document::with_template_tolerance("reader allocation"),
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let cache = PresentationCache::default();
        let snapshot = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        let live = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        drop(cache);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), live);
        assert_eq!(snapshot.scene.document.id, controller.document().id);
        drop(snapshot);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline);
    }

    #[test]
    fn presentation_successor_prunes_expired_allocation_before_reusing_the_cache() {
        let controller = crate::session::Controller::new(Document::with_template_tolerance("successor allocation"),
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        let cache = PresentationCache::default();
        let first = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        let live = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        drop(first);
        let second = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), live,
            "an expired predecessor must not leave a second allocation charge");
        assert_eq!(cache.lock().unwrap().as_ref().unwrap().1.strong_count(), 1);
        drop(second);
    }

    #[test]
    fn first_incremental_edit_registers_buffers_without_viewport_work() {
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut controller = crate::session::Controller::new(Document::with_template_tolerance("first edit"), limits).unwrap();
        assert_eq!(controller.scene().unwrap().memory_allowance_bytes(), controller.scene().unwrap().managed_memory_allowance().unwrap());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene().unwrap().memory_allowance_bytes());
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap();
        assert_eq!(controller.scene().unwrap().memory_allowance_bytes(), controller.scene().unwrap().managed_memory_allowance().unwrap());
        let overhead = std::mem::size_of::<crate::session::SceneBufferLease>() as u64 + 4 * std::mem::size_of::<usize>() as u64;
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            controller.scene().unwrap().shared_buffer_bytes() + overhead + controller.scene().unwrap().memory_allowance_bytes());
        assert_eq!(controller.coordinator().external_pending_bytes(MemoryClass::CpuCache), controller.scene().unwrap().picking_arena_bytes().unwrap(),
            "published render buffers must not be charged as future physical allocations");
    }

    #[test]
    fn concurrent_preparations_share_one_lease_and_stale_snapshots_remain_charged() {
        let mut controller = crate::session::Controller::new(Document::with_template_tolerance("viewport lease"),
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        let cache = PresentationCache::default();
        let scene = controller.scene().unwrap();
        let first = presentation_snapshot(&cache, &controller, scene, 1, None).unwrap();
        let charged = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        assert!(charged > 0);
        let second = presentation_snapshot(&cache, &controller, scene, 1, None).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &second));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), charged);
        let changed_preview = presentation_snapshot(&cache, &controller, scene, 2, None).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &changed_preview));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), charged);
        drop(changed_preview);
        controller.submit("Point").unwrap();
        controller.submit("1,2,3").unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) >= charged);
        assert!(matches!(presentation_snapshot(&cache, &controller, &first.scene, 1, None), Err(om9_core::Error::Stale(_))));
        drop(first);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) >= charged);
        drop(second);
        // The incrementally updated live scene still shares the old buffers.
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) > 0);
    }

    #[test]
    fn preparation_refusal_releases_all_leases_and_can_retry() {
        let controller = crate::session::Controller::new(Document::with_template_tolerance("viewport pressure"),
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        let cache = PresentationCache::default();
        let held = controller.scene_cache_usage().unwrap();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
        assert!(presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).is_err());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline);
        drop(pressure);
        let snapshot = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        drop(snapshot);
        // The live scene retains the small lease-owner allocation until replacement.
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) > 0);
    }

    #[test]
    fn scene_and_worker_share_buffer_charge_until_last_owner() {
        use om9_core::{geometry::{Geometry, Mesh}, model::Entity};
        let mut document = Document::with_template_tolerance("shared scene buffers");
        let point = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = point.id;
        document.entities.insert(id, point);
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut controller = crate::session::Controller::new(document, limits).unwrap();
        let cache = PresentationCache::default();
        let buffer_bytes = controller.scene().unwrap().shared_buffer_bytes();
        assert!(buffer_bytes > 0);
        let before = controller.scene_cache_usage().unwrap().bytes;
        let initial_charge = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        assert!(initial_charge >= buffer_bytes, "startup must register buffers before viewport work");
        let metadata = controller.scene().unwrap().render_snapshot_metadata_estimate().unwrap().bytes + presentation_snapshot_overhead()
            + controller.scene().unwrap().picking_arena_bytes().unwrap();
        let snapshot = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        assert_eq!(controller.scene_cache_usage().unwrap().bytes, before);
        let with_snapshot = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        assert_eq!(with_snapshot, initial_charge + metadata.max(1));
        assert!(with_snapshot > buffer_bytes + metadata.max(1)); // includes the lease owner
        // A captured STL source must accept the separately registered buffers.
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        drop(capture);
        controller.submit("Point").unwrap();
        controller.submit("4,5,6").unwrap();
        let after_edit = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let new_scene = controller.scene().unwrap();
        let added_buffer_bytes = new_scene.shared_buffer_bytes() - buffer_bytes;
        let new_metadata = new_scene.render_snapshot_metadata_estimate().unwrap().bytes + presentation_snapshot_overhead() + new_scene.picking_arena_bytes().unwrap();
        let next = presentation_snapshot(&cache, &controller, new_scene, 0, None).unwrap();
        let with_next = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        // The edit already registered the new buffer; the next worker adds metadata only.
        assert!(after_edit > with_snapshot + added_buffer_bytes);
        assert!(after_edit < with_snapshot + added_buffer_bytes + 1024);
        assert_eq!(with_next, after_edit + new_metadata.max(1));
        assert_eq!(controller.coordinator().external_pending_bytes(MemoryClass::CpuCache),
            controller.scene().unwrap().picking_arena_bytes().unwrap() + snapshot.scene.picking_arena_bytes().unwrap() + next.scene.picking_arena_bytes().unwrap());
        drop(next);
        drop(snapshot);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) >= buffer_bytes + added_buffer_bytes);
    }

    #[test]
    fn new_document_keeps_old_snapshot_buffers_charged_until_worker_releases() {
        use om9_core::{geometry::Geometry, model::Entity};
        let mut document = Document::with_template_tolerance("old scene");
        let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(point.id, point);
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut controller = crate::session::Controller::new(document, limits).unwrap();
        let cache = PresentationCache::default();
        let snapshot = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        let charged = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let old_main = controller.scene().unwrap().memory_allowance_bytes();
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(controller.document().entities.is_empty());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), charged - old_main + controller.scene().unwrap().memory_allowance_bytes());
        drop(snapshot);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene().unwrap().memory_allowance_bytes() + presentation_snapshot_overhead());
        drop(cache.lock().unwrap().take());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene().unwrap().memory_allowance_bytes());
    }

    #[test]
    fn shared_buffers_do_not_require_a_second_full_buffer_budget() {
        use om9_core::{geometry::{Geometry, Mesh}, model::Entity};
        let mut document = Document::with_template_tolerance("buffer budget");
        let mut vertices = vec![[0., 0., 0.]; 40_000];
        vertices[1] = [1., 0., 0.];
        vertices[2] = [0., 1., 0.];
        let mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices, faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(mesh.id, mesh);
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let controller = crate::session::Controller::new(document, limits).unwrap();
        let scene = controller.scene().unwrap();
        let bytes = scene.shared_buffer_bytes();
        let metadata = scene.cache_usage().unwrap().bytes - bytes;
        assert!(bytes > metadata * 4 + 65_536);
        let held = controller.scene_cache_usage().unwrap();
        let mut pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, held,
        ).unwrap();
        let slack = bytes / 2;
        pressure.shrink_to(pressure.bytes() - slack).unwrap();
        let cache = PresentationCache::default();
        let snapshot = presentation_snapshot(&cache, &controller, scene, 0, None).unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) < bytes + slack);
        drop(snapshot);
        drop(pressure);
    }

    #[test]
    fn open_registers_new_buffers_before_viewport_and_preserves_old_worker_owner() {
        use om9_core::{geometry::Geometry, model::Entity};
        let make = |name: &str, position| {
            let mut document = Document::with_template_tolerance(name);
            let entity = Entity::new("point", Geometry::Point(position), None);
            document.entities.insert(entity.id, entity);
            document
        };
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let path = std::env::temp_dir().join(format!("om9-buffer-owner-{}.om9", uuid::Uuid::new_v4()));
        let target = make("opened", [4., 5., 6.]);
        om9_archive::Archive::save(&path, &target, &limits, &Default::default()).unwrap();
        let mut controller = crate::session::Controller::new(make("old", [1., 2., 3.]), limits).unwrap();
        let old_snapshot = controller.scene().unwrap().try_render_snapshot().unwrap();
        let old_buffers = controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        let old_charge = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let old_main = controller.scene().unwrap().memory_allowance_bytes();
        controller.begin_file_operation(crate::file_lifecycle::FileKind::Open).unwrap();
        let token = controller.file_operation().unwrap().token();
        controller.accept_file_path(token, &path).unwrap();
        assert_eq!(controller.document().id, target.id);
        assert_eq!(controller.active_file_path(), Some(path.as_path()));
        let overhead = std::mem::size_of::<crate::session::SceneBufferLease>() as u64
            + 4 * std::mem::size_of::<usize>() as u64;
        // Open also retains the absolute path alongside its scene allocations.
        let path_charge = std::path::absolute(&path).unwrap().capacity() as u64;
        let new_charge = controller.scene().unwrap().shared_buffer_bytes() + overhead
            + controller.scene().unwrap().memory_allowance_bytes() + path_charge;
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), old_charge - old_main + new_charge);
        drop(old_snapshot);
        drop(old_buffers);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), new_charge);
        let before = controller.coordinator().snapshot();
        controller.begin_file_operation(crate::file_lifecycle::FileKind::Open).unwrap();
        let token = controller.file_operation().unwrap().token();
        assert!(controller.accept_file_path(token, &path.with_extension("missing.om9")).is_err());
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), new_charge);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn undo_redo_register_buffers_before_viewport_and_keep_old_snapshot_alive() {
        use om9_core::{geometry::Geometry, model::Entity};
        let mut document = Document::with_template_tolerance("history owners");
        let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(point.id, point);
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut controller = crate::session::Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap();
        controller.submit("4,5,6").unwrap();
        let cache = PresentationCache::default();
        let old = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
        let old_charge = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let old_main = controller.scene().unwrap().memory_allowance_bytes();
        let overhead = std::mem::size_of::<crate::session::SceneBufferLease>() as u64 + 4 * std::mem::size_of::<usize>() as u64;
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            old_charge - old_main + controller.scene().unwrap().shared_buffer_bytes() + overhead + controller.scene().unwrap().memory_allowance_bytes());
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        let current = controller.scene().unwrap().shared_buffer_bytes() + overhead + controller.scene().unwrap().memory_allowance_bytes();
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), old_charge - old_main + current);
        drop(old);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), current + presentation_snapshot_overhead());
        drop(cache.lock().unwrap().take());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), current);
    }

    #[test]
    fn snapshot_and_master_restore_keep_old_worker_and_register_new_scene() {
        use om9_core::{geometry::Geometry, model::Entity};
        for master in [false, true] {
            let mut document = Document::with_template_tolerance("restore owners");
            let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
            document.entities.insert(point.id, point);
            let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
            let mut controller = crate::session::Controller::new(document, limits).unwrap();
            let source = controller.capture_snapshot().unwrap();
            let master_document = controller.document().clone();
            controller.submit("Point").unwrap(); controller.submit("4,5,6").unwrap();
            let cache = PresentationCache::default();
            let old = presentation_snapshot(&cache, &controller, controller.scene().unwrap(), 0, None).unwrap();
            let old_charge = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
            let old_main = controller.scene().unwrap().memory_allowance_bytes();
            if master { controller.restore_master_document(master_document).unwrap(); }
            else { controller.restore_snapshot(&source).unwrap(); }
            assert_eq!(controller.document().entities.len(), 1);
            let overhead = std::mem::size_of::<crate::session::SceneBufferLease>() as u64 + 4 * std::mem::size_of::<usize>() as u64;
            let current = controller.scene().unwrap().shared_buffer_bytes() + overhead + controller.scene().unwrap().memory_allowance_bytes();
            assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), old_charge - old_main + current);
            drop(old);
            assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), current + presentation_snapshot_overhead());
            drop(cache.lock().unwrap().take());
            assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), current);
        }
    }

    #[test]
    fn presentation_metadata_excludes_source_pick_cache_and_stays_allocated_until_last_owner() {
        use om9_core::{geometry::Geometry, model::Entity};
        let mut document = Document::with_template_tolerance("metadata phase");
        let point = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(point.id, point);
        let mut controller = crate::session::Controller::new(document,
            Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() }).unwrap();
        let scene = controller.scene().unwrap();
        scene.spatial_index().unwrap();
        assert!(scene.cache_usage().unwrap().bytes > scene.metadata_storage_usage().unwrap().bytes + scene.shared_buffer_bytes());
        let cache = PresentationCache::default();
        let snapshot = presentation_snapshot(&cache, &controller, scene, 0, None).unwrap();
        let actual = presentation_snapshot_overhead();
        assert_eq!(snapshot._allocation.bytes(), actual);
        assert_eq!(snapshot._allocation.pending_bytes(), 0);
        assert_eq!(snapshot.scene.cache_usage().unwrap().bytes,
            snapshot.scene.metadata_storage_usage().unwrap().bytes + snapshot.scene.shared_buffer_bytes());
        let retained = snapshot.clone();
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(controller.document().entities.is_empty());
        let charge = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        drop(snapshot);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), charge);
        drop(retained);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene().unwrap().memory_allowance_bytes() + actual);
        drop(cache.lock().unwrap().take());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene().unwrap().memory_allowance_bytes());
        assert_eq!(controller.coordinator().external_pending_bytes(MemoryClass::CpuCache), controller.scene().unwrap().picking_arena_bytes().unwrap(),
            "only the new scene's not-yet-built arena remains pending");
    }
}

fn overlay_matches(actual: OverlayKey, wanted: OverlayKey, moving: bool) -> bool {
    actual == wanted || (moving
        && actual.0 == wanted.0 && actual.1 == wanted.1 && actual.2 == wanted.2
        && actual.3 == wanted.3 && actual.4 == wanted.4 && actual.6 == wanted.6)
}
#[cfg(test)]
mod overlay_cache_tests {
    use super::overlay_matches;
    use om9_core::identity::{DocumentId, ObjectId};

    #[test]
    fn moving_overlay_can_reuse_point_scale_but_never_stale_hidden_sources() {
        let key = (DocumentId::new(), 1, 2, 3, 4, 5, None);
        let mut zoomed = key;
        zoomed.5 += 1;
        assert!(overlay_matches(key, zoomed, true));
        assert!(!overlay_matches(key, zoomed, false));
        zoomed.6 = Some([ObjectId::new(), ObjectId::new()]);
        assert!(!overlay_matches(key, zoomed, true));
    }
}
struct ViewOverlays {
    key: OverlayKey,
    fixed: om9_render::overlay::OverlayBatch,
    moving: om9_render::overlay::OverlayBatch,
    ghost: om9_render::overlay::OverlayBatch,
    selected_guides: om9_render::overlay::OverlayBatch,
}
struct WorldLines {
    fixed: om9_render::overlay::OverlayBatch,
    moving: om9_render::overlay::OverlayBatch,
    ghost: om9_render::overlay::OverlayBatch,
    selected_guides: om9_render::overlay::OverlayBatch,
}
impl WorldLines {
    fn batches(&self) -> [&om9_render::overlay::OverlayBatch; 4] {
        [&self.fixed, &self.moving, &self.ghost, &self.selected_guides]
    }
    fn pending_upload_bytes(&self) -> usize {
        self.batches().iter().fold(0usize, |sum, batch| sum.saturating_add(batch.pending_upload_bytes()))
    }
}
#[derive(Default)]
struct PreparationInventory {
    active: std::sync::atomic::AtomicUsize,
    completed: std::sync::Mutex<Vec<std::sync::Weak<Vec<wgpu::Buffer>>>>,
}
struct ActivePreparation(std::sync::Arc<PreparationInventory>);
impl Drop for ActivePreparation {
    fn drop(&mut self) { self.0.active.fetch_sub(1, std::sync::atomic::Ordering::Release); }
}
trait PreparedGpuBuffers {
    fn prepared_buffers(&self) -> Vec<wgpu::Buffer>;
}
impl PreparedGpuBuffers for GpuScene {
    fn prepared_buffers(&self) -> Vec<wgpu::Buffer> {
        let mut buffers = Vec::new();
        for batch in std::iter::once(&self.batch).chain(self.ghost.iter()) {
            batch.for_each_geometry_buffer(|buffer| buffers.push(buffer.clone()));
            if let Some(buffer) = batch.instance_snapshot_buffer() { buffers.push(buffer); }
        }
        buffers
    }
}
impl PreparedGpuBuffers for ViewOverlays {
    fn prepared_buffers(&self) -> Vec<wgpu::Buffer> {
        [&self.fixed, &self.moving, &self.ghost, &self.selected_guides]
            .into_iter().flat_map(|batch| batch.gpu_buffer_handles().iter().cloned()).collect()
    }
}
impl PreparedGpuBuffers for WorldLines {
    fn prepared_buffers(&self) -> Vec<wgpu::Buffer> {
        self.batches().into_iter().flat_map(|batch| batch.gpu_buffer_handles().iter().cloned()).collect()
    }
}
struct PreparedResult<T> {
    value: T,
    _buffers: std::sync::Arc<Vec<wgpu::Buffer>>,
}
type PrepareTask = Box<dyn FnOnce() + Send>;
struct PrepareReceiver<T> {
    receiver: std::sync::mpsc::Receiver<PreparedResult<T>>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl<T> PrepareReceiver<T> {
    fn try_recv(&self) -> Result<PreparedResult<T>, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
impl<T> Drop for PrepareReceiver<T> {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
fn prepare_task<T: PreparedGpuBuffers + Send + 'static>(
    inventory: &std::sync::Arc<PreparationInventory>,
    task: impl FnOnce() -> T + Send + 'static,
) -> Option<PrepareReceiver<T>> {
    static WORKER: std::sync::OnceLock<std::sync::mpsc::SyncSender<PrepareTask>> =
        std::sync::OnceLock::new();
    let worker = WORKER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::sync_channel::<PrepareTask>(32);
        std::thread::Builder::new()
            .name("om9-viewport-prepare".into())
            .spawn(move || {
                while let Ok(task) = rx.recv() {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
                }
            })
            .expect("viewport preparation worker");
        tx
    });
    let (tx, rx) = std::sync::mpsc::channel();
    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let obsolete = cancelled.clone();
    inventory.active.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    let active = ActivePreparation(inventory.clone());
    worker
        .try_send(Box::new(move || {
            if obsolete.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }
            let result = task();
            if !obsolete.load(std::sync::atomic::Ordering::Acquire) {
                let buffers = std::sync::Arc::new(result.prepared_buffers());
                let mut completed = active.0.completed.lock().unwrap_or_else(|e| e.into_inner());
                completed.retain(|weak| weak.strong_count() > 0);
                completed.push(std::sync::Arc::downgrade(&buffers));
                drop(completed);
                let _ = tx.send(PreparedResult { value: result, _buffers: buffers });
            }
        }))
        .ok()?;
    Some(PrepareReceiver {
        receiver: rx,
        cancelled,
    })
}

/// Everything the document viewport needs on the GPU. It is rebuilt when the scene or the CPlane changes,
/// never per frame, so painting costs one draw pass.
/// Above this many mesh objects the unselected ones are merged into static ranges: one draw instead of
/// one per object, at the cost of one colour for the merged group.
/// Samples per pixel of the document viewport. Four is the multisample count WebGPU guarantees for a
/// renderable format, and it is what keeps the one-pixel grid from turning into a staircase: a diagonal
/// segment now covers part of a pixel and resolves to a grey level instead of a hard step.
pub struct GpuScene {
    batch: om9_render::scene::SceneBatch,
    mesh_objects: std::sync::Arc<Vec<(om9_core::identity::ObjectId, usize)>>,
    epoch: u64,
    cplane: u64,
    selection: u64,
    /// Batch object indices of the selected meshes. A drag preview only rewrites
    /// their model matrix, which is a handful of uniform writes instead of a scene rebuild.
    selected_objects: Vec<usize>,
    selected_models: Vec<[[f32; 4]; 4]>,
    /// Grey copy of the objects at the position a drag started from; `None` when no drag is running.
    ghost: Option<om9_render::scene::SceneBatch>,
    /// True when the batch merged unselected meshes into static ranges, so a selection change has to rebuild
    /// the batch (the selected mesh is no longer an object of its own).
    merged: bool,
    /// True when the camera is close enough that per-object culling beats one merged draw.
    close: bool,
}

impl GpuViewport {
    /// Builds the renderer on eframe's device. With `OM9_GPU_STRESS=N` the viewport instead shows a
    /// batch of N boxes through the shared-arena scene renderer, which is the architecture chosen for
    /// 1000-2000 objects (`implementation/design/M01/VIEWPORT-PERFORMANCE.md`).
    pub fn new(state: &RenderState, pool: SharedResourcePool) -> Self {
        let shared_scene = std::sync::Arc::new(std::sync::Mutex::new(None));
        let stress = std::env::var("OM9_GPU_STRESS")
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        // `OM9_GPU_STRESS_SHAPE=sphere` draws 1024-triangle spheres instead of the 12-triangle cubes, so
        // the demo can answer "how many spheres" rather than "how many boxes".
        let unit = if std::env::var("OM9_GPU_STRESS_SHAPE")
            .is_ok_and(|value| value.eq_ignore_ascii_case("sphere"))
        {
            om9_render::shapes::sphere(32, 16, 0.95)
        } else {
            unit_cube()
        };
        let mut batch = None;
        let mut cluster = None;
        if stress > 0 && std::env::var("OM9_GPU_CLUSTER").is_ok() {
            // One cluster per box: the meshlet path needs world-space vertices, so each instance is a
            // translated copy of the unit cube.
            let cube = unit.clone();
            let side = (stress as f32).cbrt().ceil() as i32;
            let spacing = 1.6;
            let mut clusters = Vec::with_capacity(stress);
            for index in 0..stress {
                let [x, y, z] = lattice_position(index, side, spacing);
                let translated = om9_render::MeshData::new(
                    cube.positions
                        .iter()
                        .map(|p| [p[0] + x, p[1] + y, p[2] + z])
                        .collect(),
                    cube.indices.clone(),
                );
                clusters.extend(om9_render::meshlet::build_meshlets(&translated));
            }
            if std::env::var("OM9_GPU_DEBUG").is_ok() {
                eprintln!(
                    "gpu: stress {stress}, side {side}, clusters {}",
                    clusters.len()
                );
            }
            cluster = om9_render::cluster::ClusterRenderer::new(
                state.device.clone(),
                state.queue.clone(),
                &clusters,
            );
            if std::env::var("OM9_GPU_DEBUG").is_ok() {
                eprintln!("gpu: cluster renderer built: {}", cluster.is_some());
            }
            if let Some(renderer) = cluster.as_mut() {
                // The manual checklist can turn occlusion on; it takes one frame to build the pyramid.
                if std::env::var("OM9_GPU_OCCLUSION").is_ok() {
                    renderer.set_occlusion(true);
                }
                if let Ok(value) = std::env::var("OM9_CLUSTER_PIXELS")
                    && let Ok(pixels) = value.trim().parse::<f32>()
                {
                    renderer.set_coarse_pixels(pixels);
                }
                if let Ok(megabytes) = std::env::var("OM9_GPU_STREAM_MB")
                    && let Ok(value) = megabytes.trim().parse::<u64>()
                {
                    renderer.set_budget_bytes(value * 1024 * 1024);
                }
            }
        }
        if stress > 0 && cluster.is_none() {
            let mut scene =
                om9_render::scene::SceneBatch::new(state.device.clone(), state.queue.clone());
            let cube = unit.clone();
            let side = (stress as f32).cbrt().ceil() as i32;
            let spacing = 1.6;
            for index in 0..stress {
                let [x, y, z] = lattice_position(index, side, spacing);
                scene.add_mesh(
                    &cube,
                    [
                        [1., 0., 0., 0.],
                        [0., 1., 0., 0.],
                        [0., 0., 1., 0.],
                        [x, y, z, 1.],
                    ],
                    [0.82, 0.52, 0.2, 1.0],
                );
            }
            batch = Some(scene);
        }
        Self {
            diagnostics: std::env::var_os("OM9_INTERACTION_LOG").map(|_| ViewportDiagnostics::default()),
            #[cfg(test)]
            target_ceiling_override: None,
            #[cfg(test)]
            legacy_world_lines: false,
            window_known_gpu_bytes: None,
            shared_world_lines: None,
            size: (0, 0),
            colour: None,
            depth: None,
            colour_view: None,
            depth_view: None,
            msaa_colour: None,
            msaa_colour_view: None,
            msaa_depth: None,
            msaa_depth_view: None,
            texture_id: None,
            last_revision: None,
            display_mode: DisplayMode::Shaded,
            last_mode: None,
            last_camera: None,
            last_epoch: None,
            last_cplane: None,
            last_selection: None,
            last_preview: None,
            last_hidden_sources: None,
            batch,
            cluster,
            shared_scene,
            last_prepare_ms: 0.0,
            last_draw_ms: 0.0,
            moving: false,
            request: Default::default(),
            pool,
            overlays: None,
            overlay_job: None,
            published_id: None,
            published_colour: None,
            spare_colour: None,
            last_profile: None,
            rendered_final: false,
            preparing: false,
            preparation_error: None,
            snapshot_retry: None,
            last_document: None,
            displayed_camera: None,
            displayed_scene: None,
            displayed_key: None,
            refinement: None,
            refinement_ms: None,
            timer: om9_render::timing::GpuTimer::new(&state.device),
            timer_frame: None,
            gpu_sample: None,
            encoded_frame: None,
            render_stats: None,
            published_frame: None,
            pending_publication: None,
            hold_publication_on_press: false,
            frame_submission: None,
        }
    }

    /// Samples of the document viewport's targets; the stress paths keep one sample and their own depth.
    fn sample_count(&self) -> u32 {
        self.sample_count_for(self.request)
    }
    fn sample_count_for(&self, request: ViewportRenderRequest) -> u32 {
        if self.batch.is_none() && self.cluster.is_none() {
            if request.moving {
                request.quality.samples()
            } else {
                4
            }
        } else {
            1
        }
    }

    #[cfg(test)]
    pub(crate) fn set_target_ceiling_for_test(&mut self, ceiling: Option<u64>) {
        self.target_ceiling_override = ceiling;
    }

    #[cfg(test)]
    pub(crate) fn use_legacy_world_lines_for_test(&mut self) { self.legacy_world_lines = true; }

    fn share_world_lines(&self) -> bool {
        #[cfg(test)]
        { !self.legacy_world_lines }
        #[cfg(not(test))]
        { true }
    }

    #[cfg(test)]
    pub(crate) fn shared_world_test_info(&self) -> Option<(usize, u64)> {
        let world = self.shared_world_lines.as_ref()?;
        let bytes = world.try_lock().ok()?.fixed.gpu_buffer_capacity_bytes();
        Some((std::sync::Arc::as_ptr(world) as usize, bytes))
    }

    #[cfg(test)]
    pub(crate) fn shared_world_test_weak_owner(&self) -> Option<std::sync::Weak<dyn std::any::Any + Send + Sync>> {
        let owner: std::sync::Arc<dyn std::any::Any + Send + Sync> = self.shared_world_lines.as_ref()?.clone();
        Some(std::sync::Arc::downgrade(&owner))
    }

    #[cfg(test)]
    pub(crate) fn shared_world_test_capacities(&self) -> Option<(usize, [u64; 4])> {
        let world = self.shared_world_lines.as_ref()?;
        let guard = world.try_lock().ok()?;
        Some((std::sync::Arc::as_ptr(world) as usize, guard.batches().map(|batch| batch.gpu_buffer_capacity_bytes())))
    }

    pub(crate) fn set_window_known_gpu_bytes(&mut self, bytes: Option<u64>) {
        self.window_known_gpu_bytes = Some(bytes);
    }

    pub(crate) fn needs_target_inventory(&self, rect: egui::Rect, pixels_per_point: f32, request: ViewportRenderRequest) -> bool {
        let size = target_size(rect, pixels_per_point, request);
        self.size != size || self.colour_view.is_none()
            || self.last_profile.map(|profile| profile.0) != Some(self.sample_count_for(request))
    }

    /// Recreates the targets when the rect changed, and registers the new view with egui.
    fn ensure_targets(&mut self, state: &RenderState, width: u32, height: u32) {
        if self.size == (width, height)
            && self.colour_view.is_some()
            && self.last_profile.map(|profile| profile.0) == Some(self.sample_count())
        {
            return;
        }
        if let Some(counters) = self.diagnostics.as_mut() {
            counters.target_rebuilds += 1;
        }
        if let Some((id, _)) = self.spare_colour.take() {
            state.renderer.write().free_texture(&id);
        }
        if let Some(id) = self.texture_id.take()
            && Some(id) != self.published_id
        {
            state.renderer.write().free_texture(&id);
        }
        let samples = self.sample_count();
        let colour = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("om9 viewport colour"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
                | if cfg!(test) { wgpu::TextureUsages::COPY_SRC } else { wgpu::TextureUsages::empty() },
            view_formats: &[],
        });
        let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
        self.texture_id = Some(state.renderer.write().register_native_texture(
            &state.device,
            &colour_view,
            wgpu::FilterMode::Linear,
        ));
        // A multisampled target needs a multisampled depth attachment too: a render pass takes one
        // sample count for all of its attachments.
        let mut msaa_colour = None;
        let mut msaa_colour_view = None;
        let mut msaa_depth = None;
        let mut msaa_depth_view = None;
        let depth = if samples > 1 {
            let colour = state.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("om9 viewport colour msaa"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let depth = state.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("om9 viewport depth msaa"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            msaa_colour_view = Some(colour.create_view(&wgpu::TextureViewDescriptor::default()));
            msaa_depth_view = Some(depth.create_view(&wgpu::TextureViewDescriptor::default()));
            msaa_colour = Some(colour);
            msaa_depth = Some(depth);
            None
        } else {
            Some(state.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("om9 viewport depth"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }))
        };
        self.depth_view = depth
            .as_ref()
            .map(|depth| depth.create_view(&wgpu::TextureViewDescriptor::default()));
        self.colour_view = Some(colour_view);
        self.colour = Some(colour);
        self.depth = depth;
        self.msaa_colour = msaa_colour;
        self.msaa_colour_view = msaa_colour_view;
        self.msaa_depth = msaa_depth;
        self.msaa_depth_view = msaa_depth_view;
        self.size = (width, height);
        self.last_profile = Some((samples, self.request.moving));
        self.last_revision = None;
        self.last_epoch = None;
        self.last_cplane = None;
        self.last_selection = None;
        self.last_preview = None;
        self.last_hidden_sources = None;
    }

    /// Never render into the texture whose pixels are still published.
    fn ensure_unpublished_colour(&mut self, state: &RenderState) {
        if self.texture_id != self.published_id || self.published_id.is_none() {
            return;
        }
        let (id, texture) = self.spare_colour.take().unwrap_or_else(|| {
            let texture = state.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("om9 viewport alternate colour"),
                size: wgpu::Extent3d { width: self.size.0, height: self.size.1, depth_or_array_layers: 1 },
                mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
                    | if cfg!(test) { wgpu::TextureUsages::COPY_SRC } else { wgpu::TextureUsages::empty() },
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let id = state.renderer.write().register_native_texture(&state.device, &view, wgpu::FilterMode::Linear);
            (id, texture)
        });
        self.colour_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.colour = Some(texture);
        self.texture_id = Some(id);
    }

    fn publish_colour(&mut self, state: &RenderState, id: egui::TextureId, texture: wgpu::Texture) {
        let old_id = self.published_id.replace(id);
        let old_colour = self.published_colour.replace(texture);
        if let (Some(old_id), Some(old_colour)) = (old_id, old_colour)
            && old_id != id
        {
            if let Some((spare_id, _)) = self.spare_colour.take() {
                state.renderer.write().free_texture(&spare_id);
            }
            if old_colour.width() == self.size.0 && old_colour.height() == self.size.1 {
                self.spare_colour = Some((old_id, old_colour));
            } else {
                state.renderer.write().free_texture(&old_id);
            }
        }
    }

    /// The cluster renderer, when this viewport draws through the cluster path.
    pub fn cluster_mut(&mut self) -> Option<&mut om9_render::cluster::ClusterRenderer> {
        self.cluster.as_mut()
    }

    /// Counters of the last rendered frame, for the interaction log: how much of the document reached
    /// the GPU and how much the frustum and the pixel threshold removed.
    pub fn last_prepare_ms(&self) -> f64 {
        self.last_prepare_ms
    }
    pub fn last_draw_ms(&self) -> f64 {
        self.last_draw_ms
    }

    pub fn frame_stats(&self) -> Option<om9_render::scene::FrameStats> {
        self.render_stats
            .or_else(|| self.batch.as_ref().map(|batch| batch.stats()))
    }

    pub fn diagnostics(&self) -> Option<ViewportDiagnostics> {
        let mut result = self.diagnostics?;
        let textures = [self.colour.as_ref(), self.depth.as_ref(), self.msaa_colour.as_ref(), self.msaa_depth.as_ref(), self.published_colour.as_ref(), self.spare_colour.as_ref().map(|(_, texture)| texture), self.pending_publication.as_ref().map(|p| &p.colour)];
        for (i, texture) in textures.iter().enumerate() {
            let Some(texture) = texture else { continue };
            if textures[..i].iter().flatten().any(|previous| *previous == *texture) { continue; }
            if matches!(texture.format(), wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Depth32Float) {
                let bytes = u64::from(texture.width()).saturating_mul(u64::from(texture.height()))
                    .saturating_mul(u64::from(texture.sample_count())).saturating_mul(4);
                result.known_target_logical_bytes = result.known_target_logical_bytes.saturating_add(bytes);
            }
        }
        result.shared_scene_buffer_bytes = if let Some(batch) = &self.batch {
            Some(batch.gpu_buffer_capacity_bytes())
        } else if self.cluster.is_some() {
            None
        } else {
            self.shared_scene.try_lock().ok().and_then(|scene| scene.as_ref().map(|scene| {
                scene.batch.gpu_buffer_capacity_bytes().saturating_add(scene.ghost.as_ref().map_or(0, |ghost| ghost.gpu_buffer_capacity_bytes()))
            }))
        };
        result.shared_scene_instance_snapshot_cpu_bytes = if let Some(batch) = &self.batch {
            Some(batch.instance_snapshot_cpu_bytes())
        } else if self.cluster.is_some() {
            None
        } else {
            self.shared_scene.try_lock().ok().and_then(|scene| scene.as_ref().map(|scene| {
                scene.batch.instance_snapshot_cpu_bytes().saturating_add(scene.ghost.as_ref().map_or(0, |ghost| ghost.instance_snapshot_cpu_bytes()))
            }))
        };
        if let Ok(pool) = self.pool.try_lock()
            && let Ok(inflight) = pool.inflight_instances.try_lock()
        {
            let (bytes, count) = inflight.totals();
            result.inflight_instance_bytes = Some(bytes);
            result.inflight_instance_buffers = Some(count);
            result.encoded_instance_bytes = Some(pool.encoded_instances.iter()
                .fold(0u64, |sum, buffer| sum.saturating_add(buffer.size())));
            result.tracked_instance_bytes = Some(pool.encoded_instances.iter()
                .filter(|buffer| !inflight.resources.iter().any(|(held, _)| held == *buffer))
                .fold(bytes, |sum, buffer| sum.saturating_add(buffer.size())));
            if let Ok(cameras) = pool.inflight_cameras.try_lock() {
                let (bytes, _) = cameras.totals();
                result.inflight_camera_bytes = Some(bytes);
                result.encoded_camera_bytes = Some(pool.encoded_cameras.iter()
                    .fold(0u64, |sum, buffer| sum.saturating_add(buffer.size())));
                result.tracked_camera_bytes = Some(pool.encoded_cameras.iter()
                    .filter(|buffer| !cameras.resources.iter().any(|(held, _)| held == *buffer))
                    .fold(bytes, |sum, buffer| sum.saturating_add(buffer.size())));
            }
            if self.batch.is_none() && self.cluster.is_none() {
                result.unique_instance_bytes = pool.unique_instance_bytes(&inflight);
                if let Ok(geometry) = pool.inflight_geometry.try_lock() {
                    let (bytes, count) = geometry.totals();
                    result.inflight_geometry_bytes = Some(bytes);
                    result.inflight_geometry_buffers = Some(count);
                    result.unique_geometry_bytes = pool.unique_geometry_bytes(&geometry);
                }
            }
            if let Ok(targets) = pool.inflight_targets.try_lock() {
                let (bytes, count) = targets.totals();
                result.inflight_target_bytes = Some(bytes);
                result.inflight_target_count = Some(count);
                result.encoded_target_bytes = Some(pool.encoded_targets.iter()
                    .fold(0u64, |sum, texture| sum.saturating_add(target_logical_bytes(texture))));
                result.tracked_target_bytes = Some(pool.encoded_targets.iter()
                    .filter(|texture| !targets.resources.iter().any(|(held, _)| held == *texture))
                    .fold(bytes, |sum, texture| sum.saturating_add(target_logical_bytes(texture))));
            }
            if let Ok(overlays) = pool.inflight_overlays.try_lock() {
                let (bytes, _) = overlays.totals();
                result.inflight_overlay_bytes = Some(bytes);
                result.encoded_overlay_bytes = Some(pool.encoded_overlays.iter()
                    .fold(0u64, |sum, buffer| sum.saturating_add(buffer.size())));
                result.tracked_overlay_bytes = Some(pool.encoded_overlays.iter()
                    .filter(|buffer| !overlays.resources.iter().any(|(held, _)| held == *buffer))
                    .fold(bytes, |sum, buffer| sum.saturating_add(buffer.size())));
            }
        }
        if let Some(overlays) = &self.overlays {
            for batch in [&overlays.fixed, &overlays.moving, &overlays.ghost, &overlays.selected_guides] {
                result.overlay_buffer_bytes = result.overlay_buffer_bytes.saturating_add(batch.gpu_buffer_capacity_bytes());
            }
        }
        if let Some(world) = &self.shared_world_lines {
            let world = world.try_lock().ok()?;
            for batch in world.batches() {
                result.overlay_buffer_bytes = result.overlay_buffer_bytes.saturating_add(batch.gpu_buffer_capacity_bytes());
            }
        }
        Some(result)
    }

    /// Enable counters for an explicitly requested GPU test.
    pub fn enable_diagnostics(&mut self) {
        self.diagnostics.get_or_insert_default();
    }

    /// Rebuilds the GPU copy of the document scene when the scene epoch or the CPlane revision changed.
    #[allow(clippy::too_many_arguments)]
    fn refresh_scene(
        state: &RenderState,
        slot: &mut Option<GpuScene>,
        camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
        cplane: om9_scene::camera::CPlane,
        _viewport: om9_scene::camera::Viewport,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection_revision: u64,
        samples: u32,
        _moving: bool,
        _close: bool,
    ) {
        let epoch = scene.map(|scene| scene.epoch).unwrap_or(0);
        let cplane_revision = cplane.revision;
        // A fixed close-range focus: rebuilding the batch every time the orbit radius changes costs
        // hundreds of ms. Three millimetres around the target is the visible prong/stone region.
        let close = false; // Full-frustum geometry is retained; no 3mm focus omission.
        let focus = 0.0;
        if let Some(current) = &*slot
            && current.epoch == epoch
            && current.cplane == cplane_revision
            && current.close == close
        {
            if current.selection == selection_revision {
                return;
            }
            if !current.merged {
                if let Some(current) = slot.as_mut() {
                    current.apply_preview(&om9_core::geometry::Transform::identity());
                    current.selected_objects = current
                        .mesh_objects
                        .iter()
                        .filter(|(id, _)| selected.contains(id))
                        .map(|(_, index)| *index)
                        .collect();
                    current.ghost = None;
                    current.selection = selection_revision;
                }
                return;
            }
            // Merged scenes hold the selected mesh inside a static range; only a rebuild takes it out.
        }
        let mut batch = om9_render::scene::SceneBatch::with_samples(
            state.device.clone(),
            state.queue.clone(),
            samples,
        );
        batch.set_min_pixel_radius(0.0);
        let mut geometry_instances = std::collections::BTreeMap::new();
        let mut selected_indices: Vec<usize> = Vec::new();
        let mut selected_models = Vec::new();
        let mut mesh_objects = Vec::new();
        let mesh_count = scene
            .map(|scene| {
                scene
                    .objects
                    .iter()
                    .filter(|object| {
                        matches!(
                            object.buffers.as_ref(),
                            om9_scene::scene::RenderBuffers::Mesh { .. }
                        )
                    })
                    .count()
            })
            .unwrap_or(0);
        let merged = false;
        let _ = mesh_count;

        if let Some(scene) = scene
            && let Some(object) = scene.objects.iter().find(|object| {
                matches!(
                    object.buffers.as_ref(),
                    om9_scene::scene::RenderBuffers::Mesh { .. }
                ) && !selected.contains(&object.root)
            })
        {
            // The merged group keeps the document colour of the objects it stands for.
            batch.set_static_tint([
                object.color[0] as f32 / 255.0,
                object.color[1] as f32 / 255.0,
                object.color[2] as f32 / 255.0,
                1.0,
            ]);
        }
        // Diagnostic display LOD (`OM9_VIEWPORT_LOD=<triangles>`): each unique source mesh is decimated
        // once and every occurrence reuses it. The BRep source and the document are untouched; this only
        // changes the display triangles the viewport uploads, which is what interaction cost follows.
        let requested = std::env::var("OM9_VIEWPORT_LOD")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase());
        let _requested_manual_lod = requested
            .as_deref()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|target| *target >= 4);
        let auto_lod = false; // Retained LOD0 has zero screen error in every preview profile.
        let manual_lod = None::<usize>;
        // Automatic display LOD: keep the whole scene near a triangle budget, scaled by the number of
        // occurrences. 1,000 earrings (72.5M triangles) become ~12M; 8,000 become ~12M as well. The
        // budget is shared by the four viewports because the geometry is shared.
        let mut lod_targets: std::collections::BTreeMap<om9_core::identity::ObjectId, usize> =
            std::collections::BTreeMap::new();
        if manual_lod.is_none()
            && auto_lod
            && let Some(scene) = scene
        {
            let budget = std::env::var("OM9_VIEWPORT_TRI_BUDGET")
                .ok()
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(12_000_000);
            let mut unique: std::collections::BTreeMap<om9_core::identity::ObjectId, usize> =
                std::collections::BTreeMap::new();
            let mut roots = std::collections::BTreeSet::new();
            for object in &scene.objects {
                roots.insert(object.root);
                if let om9_scene::scene::RenderBuffers::Mesh { triangles, .. } =
                    object.buffers.as_ref()
                {
                    *unique.entry(object.leaf).or_default() += triangles.len();
                }
            }
            let instances = roots.len().max(1);
            let min_ratio = std::env::var("OM9_LOD_MIN_RATIO")
                .ok()
                .and_then(|value| value.trim().parse::<f64>().ok())
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            let total: usize = unique.values().sum();
            if total > 0 {
                let ratio = budget as f64 / (instances as f64 * total as f64);
                if ratio < 0.9 {
                    for (leaf, triangles) in unique {
                        lod_targets.insert(
                            leaf,
                            ((triangles as f64 * ratio) as usize)
                                .max((triangles as f64 * min_ratio) as usize)
                                .max(64),
                        );
                    }
                }
            }
        }
        let mut local_meshes: std::collections::BTreeMap<
            (om9_core::identity::ObjectId, Option<usize>),
            LocalDisplayMesh,
        > = std::collections::BTreeMap::new();
        if let Some(scene) = scene {
            let spheres = if close { scene.cached_object_spheres() } else { None };
            let focus_radius = if close { focus } else { f64::INFINITY };
            for (object_index, object) in scene.objects.iter().enumerate() {
                if !object.visible {
                    continue;
                }
                if close
                    && !selected.contains(&object.root)
                    && let Some((centre, radius)) = spheres.as_ref().and_then(|s| s.get(object_index))
                {
                    let distance = ((centre[0] - camera.target[0]).powi(2)
                        + (centre[1] - camera.target[1]).powi(2)
                        + (centre[2] - camera.target[2]).powi(2))
                    .sqrt();
                    if distance - radius > focus_radius {
                        continue;
                    }
                }

                let colour = [
                    object.color[0] as f32 / 255.0,
                    object.color[1] as f32 / 255.0,
                    object.color[2] as f32 / 255.0,
                    1.0,
                ];

                if let om9_scene::scene::RenderBuffers::Mesh {
                    vertices,
                    triangles,
                    ..
                } = object.buffers.as_ref()
                {
                    let target = display_lod_target(
                        selected.contains(&object.root),
                        manual_lod,
                        lod_targets.get(&object.leaf).copied(),
                    );
                    let local = local_meshes
                        .entry((object.leaf, target))
                        .or_insert_with(|| {
                            std::sync::Arc::new(local_display_mesh(vertices, triangles, target))
                        })
                        .clone();
                    if local.0.is_empty() || local.1.is_empty() {
                        continue;
                    }
                    let model = preview_model(&object.transform);
                    let key = (object.leaf, target);
                    let index = if let Some(&source) = geometry_instances.get(&key) {
                        batch
                            .add_instance(source, model, colour)
                            .expect("registered geometry")
                    } else {
                        let mesh = om9_render::MeshData::new(local.0.clone(), local.1.clone());
                        let index = batch.add_mesh(&mesh, model, colour);
                        geometry_instances.insert(key, index);
                        index
                    };
                    mesh_objects.push((object.root, index));
                    if selected.contains(&object.root) {
                        selected_indices.push(index);
                        selected_models.push(model);
                    }
                }
            }
        }
        *slot = Some(GpuScene {
            batch,
            mesh_objects: std::sync::Arc::new(mesh_objects),
            epoch,
            cplane: cplane_revision,
            selection: selection_revision,
            selected_objects: selected_indices,
            selected_models,
            ghost: (!selected.is_empty()).then(|| build_ghost(
                &state.device,
                &state.queue,
                scene,
                selected,
                samples,
            )),
            merged,
            close,
        });
    }

    /// Whether the active camera is close enough to the scene that per-object culling beats a
    /// merged draw. Computed once from the active view so all four panes share one batch state.
    pub fn close_mode(
        camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
    ) -> bool {
        scene.is_some_and(|scene| {
            let Some(spheres) = scene.cached_object_spheres() else { return false; };
            if spheres.is_empty() {
                return false;
            }
            let mut centre = [0.0f64; 3];
            for (point, _) in spheres.iter() {
                for axis in 0..3 {
                    centre[axis] += point[axis] / spheres.len() as f64;
                }
            }
            let extent = spheres
                .iter()
                .map(|(point, radius)| {
                    ((point[0] - centre[0]).powi(2)
                        + (point[1] - centre[1]).powi(2)
                        + (point[2] - centre[2]).powi(2))
                    .sqrt()
                        + radius
                })
                .fold(0.0f64, f64::max);
            let camera_radius = ((camera.eye[0] - camera.target[0]).powi(2)
                + (camera.eye[1] - camera.target[1]).powi(2)
                + (camera.eye[2] - camera.target[2]).powi(2))
            .sqrt();
            extent > 0.0 && camera_radius < extent * 0.35
        })
    }

    /// Tells the viewport whether a manual camera motion is in flight; overlay point sizes are only
    /// rebuilt after it settles.
    pub fn set_moving(&mut self, moving: bool) {
        self.moving = moving;
    }

    pub fn set_render_request(&mut self, request: ViewportRenderRequest) {
        if request.moving || request.generation != self.request.generation {
            self.refinement = None;
            self.rendered_final = false;
        }
        self.moving = request.moving;
        self.request = request;
    }
    /// Preserve the pressed image for this UI frame while capture is pending.
    pub(crate) fn hold_publication_on_press(&mut self, hold: bool) {
        self.hold_publication_on_press = hold;
    }
    pub fn final_complete(&self) -> bool {
        self.rendered_final
    }
    pub fn preparing(&self) -> bool {
        self.preparing
    }
    pub fn preparation_error(&self) -> Option<&str> {
        self.preparation_error.as_deref()
    }
    #[cfg(test)]
    pub(crate) fn snapshot_retry_deadline(&self) -> Option<std::time::Instant> {
        self.snapshot_retry.map(|(_, deadline)| deadline)
    }
    #[cfg(test)]
    pub(crate) fn delay_snapshot_retry_for_test(&mut self, delay: std::time::Duration) {
        if let Some((_, deadline)) = &mut self.snapshot_retry {
            *deadline = std::time::Instant::now() + delay;
        }
    }
    pub fn displayed_camera(&self) -> Option<&om9_scene::camera::Camera> {
        self.displayed_camera.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn published_texture(&self) -> Option<&wgpu::Texture> {
        self.published_colour.as_ref()
    }
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }
    pub fn displayed_scene_stamp(&self) -> Option<(om9_core::identity::DocumentId, u64)> {
        self.published_id.and(self.displayed_scene)
    }
    pub fn displays_scene(&self, scene: &om9_scene::scene::Scene) -> bool {
        self.displayed_scene_stamp() == Some((scene.document.id, scene.epoch))
    }
    pub(crate) fn displayed_key(&self) -> Option<RefinementKey> {
        self.published_id.and(self.displayed_key)
    }
    pub(crate) fn displayed_mode(&self) -> Option<DisplayMode> {
        self.displayed_key().map(|key| key.mode)
    }
    pub(crate) fn displays_content(&self, scene: &om9_scene::scene::Scene, selection: u64, preview: u64,
        cplane: u64, hidden_sources: HiddenSources, mode: DisplayMode) -> bool {
        self.displayed_key().is_some_and(|key| {
            key.document == scene.document.id && key.scene == scene.epoch
                && key.selection == selection && key.preview == preview
                && key.cplane == cplane && key.hidden_sources == hidden_sources && key.size == self.size
                && key.mode == mode
        })
    }
    pub fn refinement_ms(&self) -> Option<f64> {
        self.refinement_ms
    }
    pub fn window_submitted(&mut self, queue: &wgpu::Queue) {
        if let Some(frame) = self.timer_frame.take()
            && let Some(timer) = self.timer.as_mut()
        {
            timer.submitted(frame);
        }
        if let Some(pending) = self.pending_publication.as_mut()
            && !pending.submitted
        {
            let done = pending.done.clone();
            queue.on_submitted_work_done(move || {
                done.store(true, std::sync::atomic::Ordering::Release)
            });
            pending.submitted = true;
        }
        if let Some(frame) = self.frame_submission.as_mut()
            && !frame.submitted
        {
            let done = frame.done.clone();
            queue.on_submitted_work_done(move || {
                done.store(true, std::sync::atomic::Ordering::Release)
            });
            frame.submitted = true;
        }
    }
    pub fn refinement_delayed(&self) -> bool {
        self.refinement
            .as_ref()
            .is_some_and(|(_, t)| t.elapsed() >= std::time::Duration::from_millis(250))
    }
    pub fn pending_geometry_bytes(&self) -> usize {
        self.shared_scene
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map_or(0, |s| {
                s.batch.pending_upload_bytes()
            })
    }
    fn paint_published(&self, ui: &mut egui::Ui, rect: egui::Rect) -> bool {
        if let Some(id) = self.published_id {
            ui.painter().image(
                id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
            true
        } else {
            false
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn resources_ready(
        &mut self,
        state: &RenderState,
        controller: &crate::session::Controller,
        scene: Option<&om9_scene::scene::Scene>,
        camera: &om9_scene::camera::Camera,
        cplane: om9_scene::camera::CPlane,
        viewport: om9_scene::camera::Viewport,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection: u64,
        preview: u64,
        hidden_sources: HiddenSources,
        samples: u32,
    ) -> bool {
        let Some(scene) = scene else {
            return false;
        };
        let snapshot_key = (scene.document.id, std::sync::Arc::as_ptr(&scene.document) as usize, scene.epoch, hidden_sources);
        if let Some((key, deadline)) = self.snapshot_retry {
            if key == snapshot_key && std::time::Instant::now() < deadline {
                return false;
            }
            self.snapshot_retry = None;
        }
        if let Some(counters) = self.diagnostics.as_mut() {
            counters.readiness_checks += 1;
        }
        let signature = (scene.document.id, scene.epoch, selection, hidden_sources);
        let share_world = self.share_world_lines();
        let mut pool = self.pool.lock().unwrap_or_else(|e| e.into_inner());
        let preparations = pool.preparations.clone();
        let presentation = pool.presentation.clone();
        if pool.signature != Some(signature) {
            // A viewport may still own an old scene while the replacement is
            // preparing. Drop its optional cache without assuming in-flight
            // GPU references have already released the underlying allocation.
            for entry in pool.entries.values() {
                if let Some(old) = entry.scene.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                    old.batch.evict_instance_snapshot();
                    if let Some(ghost) = old.ghost.as_mut() {
                        ghost.evict_instance_snapshot();
                    }
                }
            }
            pool.signature = Some(signature);
            pool.entries.clear();
            pool.world_lines.clear();
        }
        if !pool.entries.contains_key(&samples) {
            let source = pool.entries.values().find_map(|e| {
                let slot = e.scene.lock().unwrap_or_else(|e| e.into_inner());
                let gpu = slot.as_ref()?;
                (gpu.batch.pending_upload_bytes() == 0
                    && gpu
                        .ghost
                        .as_ref()
                        .is_none_or(|g| g.pending_upload_bytes() == 0))
                .then(|| gpu.fork_samples_task(samples))
            });
            if let Some(task) = source {
                if let Some(counters) = self.diagnostics.as_mut() {
                    counters.scene_preparations += 1;
                }
                pool.entries.insert(
                    samples,
                    ResourceEntry {
                        scene: Default::default(),
                        job: prepare_task(&preparations, task),
                        failed: false,
                    },
                );
            }
        }
        let entry = pool
            .entries
            .entry(samples)
            .or_insert_with(|| ResourceEntry {
                scene: Default::default(),
                job: None,
                failed: false,
            });
        if entry
            .scene
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_none()
            && entry.job.is_none()
            && !entry.failed
        {
            if let Some(counters) = self.diagnostics.as_mut() {
                counters.scene_preparations += 1;
            }
            let snapshot = match presentation_snapshot(&presentation, controller, scene, preview, hidden_sources) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    if matches!(error, om9_core::Error::Budget(_)) {
                        self.snapshot_retry = Some((snapshot_key, std::time::Instant::now() + std::time::Duration::from_millis(250)));
                    }
                    self.preparation_error = Some(error.to_string()); return false;
                }
            };
            let state = state.clone();
            let camera = camera.clone();
            let selected = selected.clone();
            entry.job = prepare_task(&preparations, move || {
                let mut slot = None;
                Self::refresh_scene(
                    &state,
                    &mut slot,
                    &camera,
                    Some(&snapshot.scene),
                    cplane,
                    viewport,
                    &selected,
                    selection,
                    samples,
                    true,
                    false,
                );
                slot.expect("prepared scene")
            });
        }
        if let Some(job) = entry.job.as_ref() {
            match job.try_recv() {
                Ok(prepared) => {
                    *entry.scene.lock().unwrap_or_else(|e| e.into_inner()) = Some(prepared.value);
                    entry.job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    entry.job = None;
                    entry.failed = true;
                    self.preparation_error = Some("GPU scene preparation failed".into());
                }
                _ => {}
            }
        }
        self.shared_scene = entry.scene.clone();
        let geometry_ready = entry
            .scene
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        let current_scene = std::sync::Arc::downgrade(&self.shared_scene);
        pool.known_scenes.retain(|scene| scene.strong_count() > 0);
        if !pool.known_scenes.iter().any(|scene| scene.ptr_eq(&current_scene)) {
            pool.known_scenes.push(current_scene);
        }
        let world = pool.world_lines.entry(samples).or_default();
        if world.batch.is_none() && world.job.is_none() && !world.failed {
            let snapshot = match presentation_snapshot(&presentation, controller, scene, preview, hidden_sources) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    if matches!(error, om9_core::Error::Budget(_)) {
                        self.snapshot_retry = Some((snapshot_key, std::time::Instant::now() + std::time::Duration::from_millis(250)));
                    }
                    self.preparation_error = Some(error.to_string()); return false;
                }
            };
            let state = state.clone();
            let selected = selected.clone();
            world.job = prepare_task(&preparations, move || {
                if share_world { build_shared_world_lines(&state, &snapshot.scene, &selected, samples) }
                else { empty_world_lines(&state, samples) }
            });
        }
        if let Some(job) = world.job.as_ref() {
            match job.try_recv() {
                Ok(prepared) => {
                    world.batch = Some(std::sync::Arc::new(std::sync::Mutex::new(prepared.value)));
                    world.job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    world.job = None;
                    world.failed = true;
                    self.preparation_error = Some("GPU world-line preparation failed".into());
                }
                _ => {}
            }
        }
        self.shared_world_lines = world.batch.clone();
        let world_ready = self.shared_world_lines.is_some();
        drop(pool);
        // Mesh edges, curves and grid do not depend on camera scale. Only point crosses do.
        let has_points=scene.objects.iter().any(|o|matches!(o.buffers.as_ref(),om9_scene::scene::RenderBuffers::Points{points,..} if !points.is_empty()));
        let point_size = if has_points {
            world_per_pixel(camera, viewport) * 5.
        } else {
            0.
        };
        let key = (
            scene.document.id,
            scene.epoch,
            selection,
            cplane.revision,
            samples,
            point_size.to_bits(),
            hidden_sources,
        );
        // Orbit/pan preserve point size; moving zoom reuses its last point markers until settling.
        let overlay_valid = self.overlays.as_ref().is_some_and(|o| overlay_matches(o.key, key, self.moving));
        if !overlay_valid {
            if self.overlay_job.as_ref().is_some_and(|(k, _)| !overlay_matches(*k, key, self.moving)) {
                self.overlay_job = None;
            }
            if self.overlay_job.is_none() {
                if let Some(counters) = self.diagnostics.as_mut() {
                    counters.overlay_preparations += 1;
                }
                let shared = self.overlays.as_ref().filter(|o| {
                    o.key.0 == key.0
                        && o.key.1 == key.1
                        && o.key.2 == key.2
                        && o.key.3 == key.3
                        && o.key.5 == key.5
                        && o.key.6 == key.6
                        && o.fixed.pending_upload_bytes() == 0
                        && o.moving.pending_upload_bytes() == 0
                        && o.ghost.pending_upload_bytes() == 0
                        && o.selected_guides.pending_upload_bytes() == 0
                });
                if let Some(source) = shared {
                    let fixed = source.fixed.fork_samples_task(samples);
                    let moving = source.moving.fork_samples_task(samples);
                    let ghost = source.ghost.fork_samples_task(samples);
                    let selected_guides = source.selected_guides.fork_samples_task(samples);
                    self.overlay_job = prepare_task(&preparations, move || ViewOverlays {
                        key,
                        fixed: fixed(),
                        moving: moving(),
                        ghost: ghost(),
                        selected_guides: selected_guides(),
                    })
                    .map(|rx| (key, rx));
                } else {
                    let snapshot = match presentation_snapshot(&presentation, controller, scene, preview, hidden_sources) {
                        Ok(snapshot) => snapshot,
                        Err(error) => {
                            if matches!(error, om9_core::Error::Budget(_)) {
                                self.snapshot_retry = Some((snapshot_key, std::time::Instant::now() + std::time::Duration::from_millis(250)));
                            }
                            self.preparation_error = Some(error.to_string()); return false;
                        }
                    };
                    let state = state.clone();
                    let selected = selected.clone();
                    self.overlay_job = prepare_task(&preparations, move || {
                        let (fixed, moving, ghost) = build_lines(
                            &state,
                            Some(&snapshot.scene),
                            cplane,
                            &selected,
                            point_size,
                            samples,
                            share_world,
                        );
                        let selected_guides = if share_world {
                            om9_render::overlay::OverlayBatch::on_top(state.device.clone(), state.queue.clone(), samples)
                        } else {
                            build_selected_shape_lines(&state, &snapshot.scene, &selected, samples)
                        };
                        ViewOverlays {
                            key,
                            fixed,
                            moving,
                            ghost,
                            selected_guides,
                        }
                    })
                    .map(|rx| (key, rx));
                }
            }
        }
        if let Some((job_key, job)) = self.overlay_job.as_ref() {
            match job.try_recv() {
                Ok(prepared) => {
                    if overlay_matches(*job_key, key, self.moving) {
                        self.overlays = Some(prepared.value);
                    }
                    self.overlay_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.overlay_job = None;
                    self.preparation_error = Some("GPU overlay preparation failed".into());
                }
                _ => {}
            }
        }
        let overlay_ready = self.overlays.as_ref().is_some_and(|o| overlay_matches(o.key, key, self.moving));
        if !geometry_ready || !overlay_ready || !world_ready {
            return false;
        }
        // A prior admission failure is transient; all required resources have
        // now been prepared successfully for this scene.
        self.preparation_error = None;
        let mut pool = self.pool.lock().unwrap_or_else(|e| e.into_inner());
        if pool
            .budget
            .as_ref()
            .is_none_or(|b| b.frame != self.request.frame_id)
        {
            pool.budget = Some(UploadBudget {
                frame: self.request.frame_id,
                left: 8 << 20,
                deadline: std::time::Instant::now() + std::time::Duration::from_millis(2),
            });
        }
        let budget = pool.budget.as_mut().unwrap();
        let mut slot = self.shared_scene.lock().unwrap_or_else(|e| e.into_inner());
        let gpu = slot.as_mut().unwrap();
        let mut geometry_uploaded = gpu.batch.flush_uploads_budget(budget.left, budget.deadline);
        budget.left -= geometry_uploaded;
        if let Some(ghost) = gpu.ghost.as_mut() {
            let uploaded = ghost.flush_uploads_budget(budget.left, budget.deadline);
            budget.left -= uploaded;
            geometry_uploaded += uploaded;
        }
        let mut overlay_uploaded = 0;
        let mut world = self.shared_world_lines.as_ref().unwrap().lock().unwrap_or_else(|e| e.into_inner());
        let WorldLines { fixed, moving, ghost, selected_guides } = &mut *world;
        for batch in [fixed, moving, ghost, selected_guides] {
            let uploaded = batch.flush_uploads_budget(budget.left, budget.deadline);
            budget.left -= uploaded;
            overlay_uploaded += uploaded;
        }
        let overlays = self.overlays.as_mut().unwrap();
        for batch in [
            &mut overlays.fixed,
            &mut overlays.moving,
            &mut overlays.ghost,
            &mut overlays.selected_guides,
        ] {
            let uploaded = batch.flush_uploads_budget(budget.left, budget.deadline);
            budget.left -= uploaded;
            overlay_uploaded += uploaded;
        }
        if let Some(counters) = self.diagnostics.as_mut() {
            counters.geometry_upload_bytes += geometry_uploaded as u64;
            counters.overlay_upload_bytes += overlay_uploaded as u64;
        }
        gpu.batch.pending_upload_bytes() == 0
            && gpu
                .ghost
                .as_ref()
                .is_none_or(|g| g.pending_upload_bytes() == 0)
            && world.pending_upload_bytes() == 0
            && overlays.fixed.pending_upload_bytes() == 0
            && overlays.moving.pending_upload_bytes() == 0
            && overlays.ghost.pending_upload_bytes() == 0
            && overlays.selected_guides.pending_upload_bytes() == 0
    }

    /// Draws the scene into the GPU target and paints it into `rect`.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        state: &RenderState,
        controller: &crate::session::Controller,
        camera: Camera,
        revision: u64,
        document_camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
        cplane: om9_scene::camera::CPlane,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection_revision: u64,
        preview_revision: u64,
        hidden_sources: HiddenSources,
        _close: bool,
    ) -> bool {
        let hold_publication_on_press = std::mem::take(&mut self.hold_publication_on_press);
        let pixels_per_point = ui.ctx().pixels_per_point();
        if self.pending_publication.is_some() || self.frame_submission.is_some() {
            let _ = state.device.poll(wgpu::PollType::Poll);
        }
        if let Some(timer) = self.timer.as_mut()
            && let Some(sample) = timer.poll(&state.device, &state.queue)
        {
            self.gpu_sample = Some(sample);
        }
        let (width, height) = target_size(rect, pixels_per_point, self.request);
        let limit = state.device.limits().max_texture_dimension_2d;
        if width > limit || height > limit {
            self.preparation_error = Some(format!(
                "Viewport {width}×{height} exceeds GPU texture limit {limit}; image is not complete"
            ));
            self.rendered_final = false;
            self.preparing = false;
            self.pending_publication = None;
            return self.paint_published(ui, rect);
        }
        if self
            .preparation_error
            .as_deref()
            .is_some_and(|e| e.starts_with("Viewport "))
        {
            self.preparation_error = None;
        }
        let samples = self.sample_count();
        let needs_targets = self.size != (width, height) || self.colour_view.is_none()
            || self.last_profile.map(|profile| profile.0) != Some(samples);
        if needs_targets && self.batch.is_none() && self.cluster.is_none() {
            if self.window_known_gpu_bytes == Some(None) {
                self.preparation_error = Some("Viewport GPU allocation inventory unavailable; retrying".into());
                self.rendered_final = false;
                self.preparing = false;
                self.pending_publication = None;
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
                return if self.last_document == scene.map(|s| s.document.id) {
                    self.paint_published(ui, rect)
                } else { false };
            }
            let preview = crate::gpu_budget::selected_adapter_preview(&state.adapter, &state.device);
            let configured = preview.configured_ceiling_bytes;
            #[cfg(test)]
            let configured = self.target_ceiling_override.unwrap_or(configured);
            let ceiling = configured.saturating_sub(self.window_known_gpu_bytes.flatten().unwrap_or(0))
                .min(preview.os_remaining_after_headroom_bytes.unwrap_or(u64::MAX));
            let minimum = viewport_target_minimum_bytes(width, height, samples);
            if minimum.is_none_or(|bytes| bytes > ceiling) {
                self.preparation_error = Some(format!(
                    "Viewport {width}×{height} needs at least {} bytes for full-resolution targets; GPU ceiling and sampled headroom permit {ceiling} bytes{}",
                    minimum.map_or_else(|| "more than addressable".to_owned(), |bytes| bytes.to_string()),
                    if preview.hardware_ceiling_verified { "" } else { " (hardware ceiling unverified)" },
                ));
                self.rendered_final = false;
                self.preparing = false;
                self.pending_publication = None;
                return if self.last_document == scene.map(|s| s.document.id) {
                    self.paint_published(ui, rect)
                } else { false };
            }
        }
        self.ensure_targets(state, width, height);
        let (Some(colour_view), Some(texture_id)) = (self.colour_view.clone(), self.texture_id)
        else {
            return false;
        };
        // The camera revision is the identity of the rendered frame: while it and the target size are
        // unchanged, redrawing would produce the same pixels, so the cached texture is reused.
        let epoch = scene.map(|scene| scene.epoch).unwrap_or(0);
        let document_id = scene.map(|s| s.document.id);
        if self.last_document != document_id {
            // An old document's image cannot serve as the new document's placeholder.
            if let Some(id) = self.published_id.take()
                && Some(id) != self.texture_id
            {
                state.renderer.write().free_texture(&id);
            }
            self.published_colour = None;
            self.displayed_camera = None;
            self.displayed_scene = None;
            self.displayed_key = None;
            self.rendered_final = false;
            self.last_document = document_id;
            self.render_stats = None;
            self.last_revision = None;
            self.refinement = None;
            self.pending_publication = None;
            self.preparation_error = None;
        }
        let unchanged = self.last_revision == Some(revision)
            && self.last_mode == Some(self.display_mode)
            && self.last_camera.as_ref() == Some(document_camera)
            && self.size == (width, height)
            && self.last_epoch == Some(epoch)
            && self.last_cplane == Some(cplane.revision)
            && self.last_selection == Some(selection_revision)
            && self.last_preview == Some(preview_revision)
            && self.last_hidden_sources == Some(hidden_sources);
        let document = self.batch.is_none() && self.cluster.is_none();
        if document {
            self.preparing = false;
            if !unchanged || self.request.moving {
                self.rendered_final = false;
            }
            if let Some(frame) = &self.frame_submission {
                if !frame.done.load(std::sync::atomic::Ordering::Acquire) {
                    self.preparing = true;
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(2));
                    return self.paint_published(ui, rect);
                }
                self.frame_submission = None;
            }
            let key = document_id.map(|document| RefinementKey {
                document,
                scene: epoch,
                camera: revision,
                selection: selection_revision,
                cplane: cplane.revision,
                preview: preview_revision,
                hidden_sources,
                mode: self.display_mode,
                size: (width, height),
                generation: self.request.generation,
            });
            // A press must be evaluated against the exact image visible before
            // this UI frame. Never hold an image whose content is stale.
            if hold_publication_on_press && self.pending_publication.is_some() && self.published_id.is_some()
                && key.zip(self.displayed_key).is_some_and(|(current, displayed)| current.same_input_basis(displayed))
                && self.displayed_camera.as_ref() == Some(document_camera) {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
                return self.paint_published(ui, rect);
            }
            if !self.request.moving
                && !self.rendered_final
                && let Some(key) = key
                && self.refinement.as_ref().is_none_or(|(old, _)| *old != key)
            {
                self.refinement = Some((key, std::time::Instant::now()));
            }
            if let Some(pending) = self.pending_publication.take()
                && key.is_some_and(|current| {
                    if self.request.moving { pending.key.same_content(current) }
                    else { pending.key == current && pending.camera == *document_camera }
                }) {
                if pending.done.load(std::sync::atomic::Ordering::Acquire) {
                    let current_camera = pending.camera == *document_camera;
                    self.publish_colour(state, pending.texture, pending.colour);
                    self.displayed_camera = Some(pending.camera);
                    self.displayed_scene = Some((pending.key.document, pending.key.scene));
                    self.displayed_key = Some(pending.key);
                    self.rendered_final = !self.request.moving;
                    self.published_frame = Some(pending.frame_id);
                    if let Some((_, started)) = self.refinement.take() {
                        self.refinement_ms = Some(started.elapsed().as_secs_f64() * 1000.);
                    }
                    if !self.request.moving || current_camera {
                        return self.paint_published(ui, rect);
                    }
                } else {
                    self.pending_publication = Some(pending);
                    self.preparing = true;
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(2));
                    return self.paint_published(ui, rect);
                }
            }
            // A complete image for this exact state needs no scene/overlay preparation.
            if unchanged
                && self.published_id.is_some()
                && self.published_id == self.texture_id
                && self.displayed_camera.as_ref() == Some(document_camera)
                && (self.request.moving || self.rendered_final)
            {
                if let Some(counters) = self.diagnostics.as_mut() {
                    counters.cached_frames += 1;
                }
                return self.paint_published(ui, rect);
            }
            let samples = self.sample_count();
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            if !self.request.render_due && self.published_id.is_some() {
                return self.paint_published(ui, rect);
            }
            let prepare_started = std::time::Instant::now();
            if !self.resources_ready(
                state,
                controller,
                scene,
                document_camera,
                cplane,
                viewport,
                selected,
                selection_revision,
                preview_revision,
                hidden_sources,
                samples,
            ) {
                self.preparing = true;
                let delay = self.snapshot_retry.map(|(_, deadline)| deadline.saturating_duration_since(std::time::Instant::now()))
                    .unwrap_or(std::time::Duration::from_millis(2));
                ui.ctx().request_repaint_after(delay);
                return self.paint_published(ui, rect);
            }
            self.last_prepare_ms = prepare_started.elapsed().as_secs_f64() * 1000.;
            if !self.request.moving
                && !self.rendered_final
                && unchanged
                && self.published_id == self.texture_id
            {
                // Q0 already rendered every LOD0 pass at full size/4x. Reuse that complete image;
                // still wait for its GPU queue tail before certifying the new generation.
                if let (Some(key), Some(colour)) = (key, self.colour.clone()) {
                    self.pending_publication = Some(PendingPublication {
                        key,
                        done: Default::default(),
                        submitted: false,
                        texture: texture_id,
                        colour,
                        camera: document_camera.clone(),
                        frame_id: self.request.frame_id,
                    });
                    self.preparing = true;
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(2));
                    return self.paint_published(ui, rect);
                }
            }
            let changed = !unchanged || self.published_id != self.texture_id;
            if changed {
                self.ensure_unpublished_colour(state);
                let (Some(colour_view), Some(texture_id)) = (self.colour_view.clone(), self.texture_id) else {
                    return self.paint_published(ui, rect);
                };
                let render_colour = if samples > 1 {
                    self.msaa_colour_view.clone()
                } else {
                    self.colour_view.clone()
                };
                let depth = if samples > 1 {
                    self.msaa_depth_view.clone()
                } else {
                    self.depth_view.clone()
                };
                let (Some(render_colour), Some(depth), Some(overlays)) =
                    (render_colour, depth, self.overlays.as_mut())
                else {
                    return self.paint_published(ui, rect);
                };
                let Some(shared_world_lines) = self.shared_world_lines.as_ref() else {
                    return self.paint_published(ui, rect);
                };
                let mut world = shared_world_lines.lock().unwrap_or_else(|e| e.into_inner());
                let mut slot = self.shared_scene.lock().unwrap_or_else(|e| e.into_inner());
                let Some(gpu) = slot.as_mut() else {
                    return false;
                };
                let delta = scene
                    .and_then(|s| {
                        s.objects
                            .iter()
                            .find(|o| selected.contains(&o.root))
                            .map(|o| o.preview)
                    })
                    .unwrap_or_else(om9_core::geometry::Transform::identity);
                gpu.apply_preview(&delta);
                overlays.moving.set_transform(preview_model(&delta));
                overlays.selected_guides.set_transform(preview_model(&delta));
                world.moving.set_transform(preview_model(&delta));
                world.selected_guides.set_transform(preview_model(&delta));
                let started = std::time::Instant::now();
                let mut pool = self.pool.lock().unwrap_or_else(|e| e.into_inner());
                let encoder = pool.encoder.get_or_insert_with(|| {
                    state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("OM9 window frame"),
                        })
                });
                let timed = self.timer.as_ref().is_some_and(|t| t.available())
                    && self.timer_frame.is_none();
                if timed {
                    self.timer
                        .as_ref()
                        .unwrap()
                        .marker(encoder, &render_colour, false);
                }
                let resolve = (samples > 1).then_some(&colour_view);
                let mut ghost_instance = None;
                let mut main_camera = None;
                let mut ghost_camera = None;
                if self.display_mode == DisplayMode::Wireframe {
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("wireframe clear"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_colour, depth_slice: None, resolve_target: resolve,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth,
                            depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
                    });
                } else {
                    main_camera = match gpu.batch.encode_pass(
                        encoder,
                        &render_colour,
                        resolve,
                        &depth,
                        camera,
                        width,
                        height,
                        true,
                    ) {
                        Ok(buffer) => Some(buffer),
                        Err(_) => return false,
                    };
                }
                if self.display_mode == DisplayMode::Shaded
                    && delta.0 != om9_core::geometry::Transform::identity().0
                    && let Some(ghost) = gpu.ghost.as_mut()
                {
                    if let Ok(buffer) = ghost.encode_pass(
                            encoder,
                            &render_colour,
                            resolve,
                            &depth,
                            camera,
                            width,
                            height,
                            false,
                        ) {
                        ghost_camera = Some(buffer);
                        ghost_instance = ghost.instance_snapshot_buffer();
                    }
                }
                let fixed_uniform = overlays.fixed.encode_pass(
                    encoder,
                    &render_colour,
                    resolve,
                    &depth,
                    camera,
                    width,
                    height,
                );
                let world_uniform = world.fixed.encode_pass(
                    encoder, &render_colour, resolve, &depth, camera, width, height,
                );
                let world_ghost_uniform = if delta.0 != om9_core::geometry::Transform::identity().0 {
                    world.ghost.encode_pass(encoder, &render_colour, resolve, &depth, camera, width, height)
                } else { None };
                let ghost_uniform = if delta.0 != om9_core::geometry::Transform::identity().0 {
                    overlays.ghost.encode_pass(
                        encoder,
                        &render_colour,
                        resolve,
                        &depth,
                        camera,
                        width,
                        height,
                    )
                } else { None };
                let world_moving_uniform = world.moving.encode_pass(
                    encoder, &render_colour, resolve, &depth, camera, width, height,
                );
                let moving_uniform = overlays.moving.encode_pass(
                    encoder,
                    &render_colour,
                    resolve,
                    &depth,
                    camera,
                    width,
                    height,
                );
                let world_guides_uniform = world.selected_guides.encode_pass(encoder, &render_colour, resolve, &depth, camera, width, height);
                let guides_uniform = overlays.selected_guides.encode_pass(encoder, &render_colour, resolve, &depth, camera, width, height);
                if timed {
                    let timer = self.timer.as_ref().unwrap();
                    timer.marker(encoder, &render_colour, true);
                    timer.resolve(encoder);
                    self.timer_frame = Some(self.request.frame_id);
                }
                if self.display_mode == DisplayMode::Shaded
                    && let Some(instance) = gpu.batch.instance_snapshot_buffer()
                {
                    pool.track_encoded_instance(instance);
                }
                if let Some(instance) = ghost_instance {
                    pool.track_encoded_instance(instance);
                }
                if self.display_mode == DisplayMode::Shaded {
                    gpu.batch.for_each_geometry_buffer(|buffer| {
                        if !pool.encoded_geometry.contains(buffer) { pool.encoded_geometry.push(buffer.clone()); }
                    });
                    if delta.0 != om9_core::geometry::Transform::identity().0
                        && let Some(ghost) = gpu.ghost.as_ref()
                    {
                        ghost.for_each_geometry_buffer(|buffer| {
                            if !pool.encoded_geometry.contains(buffer) { pool.encoded_geometry.push(buffer.clone()); }
                        });
                    }
                }
                if self.display_mode == DisplayMode::Shaded {
                    for buffer in [main_camera, ghost_camera].into_iter().flatten() {
                        pool.track_encoded_camera(buffer);
                    }
                }
                {
                    for buffer in [fixed_uniform, world_uniform, world_ghost_uniform, ghost_uniform,
                        world_moving_uniform, moving_uniform, world_guides_uniform, guides_uniform].into_iter().flatten() {
                        pool.track_encoded_overlay(buffer);
                    }
                    for batch in [&world.fixed, &world.moving, &world.selected_guides] {
                        for buffer in batch.gpu_buffer_handles() { pool.track_encoded_overlay(buffer.clone()); }
                    }
                    if delta.0 != om9_core::geometry::Transform::identity().0 {
                        for buffer in world.ghost.gpu_buffer_handles() { pool.track_encoded_overlay(buffer.clone()); }
                    }
                    for batch in [&overlays.fixed, &overlays.moving, &overlays.selected_guides] {
                        for buffer in batch.gpu_buffer_handles() { pool.track_encoded_overlay(buffer.clone()); }
                    }
                    if delta.0 != om9_core::geometry::Transform::identity().0 {
                        for buffer in overlays.ghost.gpu_buffer_handles() { pool.track_encoded_overlay(buffer.clone()); }
                    }
                }
                if let Some(colour) = self.colour.as_ref() {
                    pool.track_encoded_target(colour.clone());
                }
                if samples > 1 {
                    if let Some(colour) = self.msaa_colour.as_ref() { pool.track_encoded_target(colour.clone()); }
                    if let Some(depth) = self.msaa_depth.as_ref() { pool.track_encoded_target(depth.clone()); }
                } else if let Some(depth) = self.depth.as_ref() {
                    pool.track_encoded_target(depth.clone());
                }
                drop(pool);
                self.last_draw_ms = started.elapsed().as_secs_f64() * 1000.;
                self.encoded_frame = Some(self.request.frame_id);
                if let Some(counters) = self.diagnostics.as_mut() {
                    counters.encoded_frames += 1;
                    if self.display_mode == DisplayMode::Shaded {
                        let stats = gpu.batch.stats();
                        counters.main_scene_camera_snapshot_bytes = counters.main_scene_camera_snapshot_bytes.saturating_add(stats.camera_snapshot_bytes);
                        counters.main_scene_instance_snapshot_bytes = counters.main_scene_instance_snapshot_bytes.saturating_add(stats.instance_snapshot_bytes);
                    }
                }
                self.render_stats = (self.display_mode == DisplayMode::Shaded).then(|| gpu.batch.stats());
                self.last_revision = Some(revision);
                self.last_mode = Some(self.display_mode);
                self.last_camera = Some(document_camera.clone());
                self.last_epoch = Some(epoch);
                self.last_cplane = Some(cplane.revision);
                self.last_selection = Some(selection_revision);
                self.last_preview = Some(preview_revision);
                self.last_hidden_sources = Some(hidden_sources);
                drop(slot);
                self.frame_submission = Some(FrameSubmission {
                    done: Default::default(),
                    submitted: false,
                });
                if let (Some(key), Some(colour)) = (key, self.colour.clone()) {
                    self.pending_publication = Some(PendingPublication {
                        key,
                        done: Default::default(),
                        submitted: false,
                        texture: texture_id,
                        colour,
                        camera: document_camera.clone(),
                        frame_id: self.request.frame_id,
                    });
                    self.preparing = true;
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(2));
                }
            }
            self.rendered_final =
                !self.request.moving && self.pending_publication.is_none() && unchanged;
            return self.paint_published(ui, rect);
        }
        let Some(depth_view) = self.depth_view.clone() else {
            return false;
        };
        if !unchanged
            && let Some(cluster) = self.cluster.as_mut()
            && cluster.render(&colour_view, camera, width, height).is_err()
        {
            return false;
        }
        if !unchanged
            && self.cluster.is_none()
            && let Some(batch) = self.batch.as_mut()
            && batch
                .render_into(&colour_view, &depth_view, camera, width, height)
                .is_err()
        {
            return false;
        }
        self.last_revision = Some(revision);
        self.last_mode = Some(self.display_mode);
        self.last_camera = Some(document_camera.clone());
        self.last_epoch = Some(epoch);
        self.last_cplane = Some(cplane.revision);
        self.last_selection = Some(selection_revision);
        self.last_preview = Some(preview_revision);
        self.last_hidden_sources = Some(hidden_sources);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(texture_id, rect, uv, egui::Color32::WHITE);
        true
    }
}

/// Maps the document camera onto the renderer camera.
///
/// Keeps the document's perspective/orthographic projection and parallel-view scale.
pub fn scene_camera(camera: &om9_scene::camera::Camera) -> Camera {
    let target = camera.target.map(|value| value as f32);
    let up = camera.up.map(|value| value as f32);
    match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => Camera {
            eye: camera.eye.map(|value| value as f32),
            target,
            up,
            projection: om9_render::Projection::Perspective {
                vertical_fov_rad: vertical_fov_rad as f32,
            },
            near: camera.near_mm as f32,
            far: camera.far_mm as f32,
        },
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            // A real parallel projection: no vanishing point, so a top view shows squares, not depth.
            let span = (vertical_span_mm as f32).max(1e-3);
            let eye = camera.eye.map(|value| value as f32);
            Camera {
                eye,
                target,
                up,
                projection: om9_render::Projection::Orthographic {
                    vertical_span: span,
                },
                near: camera.near_mm as f32,
                far: camera.far_mm as f32,
            }
        }
    }
}

/// A cubic lattice, so every one of the four viewports sees a block of boxes rather than a flat sheet.
fn lattice_position(index: usize, side: i32, spacing: f32) -> [f32; 3] {
    let index = index as i32;
    let x = index % side;
    let y = (index / side) % side;
    let z = index / (side * side);
    let centre = side as f32 * 0.5;
    [
        (x as f32 - centre) * spacing,
        (y as f32 - centre) * spacing,
        (z as f32 - centre) * spacing,
    ]
}

/// A unit cube: 8 vertices and 12 triangles, the cheapest real object for the stress scene.
fn unit_cube() -> MeshData {
    let positions = vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let indices = vec![
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5,
        0, 4, 7, 0, 7, 3,
    ];
    MeshData::new(positions, indices)
}

/// Editing accuracy: a selected object always uses LOD 0, otherwise the manual override wins over the
/// per-mesh automatic target.
fn display_lod_target(
    selected: bool,
    manual: Option<usize>,
    automatic: Option<usize>,
) -> Option<usize> {
    if selected { None } else { manual.or(automatic) }
}

/// Local display mesh shared by every occurrence of one source entity (positions, triangle indices).
type LocalDisplayMesh = std::sync::Arc<(Vec<[f32; 3]>, Vec<u32>)>;

/// Local display positions and indices for one source mesh, optionally decimated once by `OM9_VIEWPORT_LOD`.
fn local_display_mesh(
    vertices: &[[f64; 3]],
    triangles: &[([u32; 3], u32)],
    lod_target: Option<usize>,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let positions: Vec<[f32; 3]> = vertices
        .iter()
        .map(|point| [point[0] as f32, point[1] as f32, point[2] as f32])
        .collect();
    let mut indices = Vec::with_capacity(triangles.len() * 3);
    for (corners, _) in triangles {
        indices.extend_from_slice(corners);
    }
    if let Some(target) = lod_target
        && indices.len() / 3 > target
    {
        let mesh = om9_render::MeshData::new(positions, indices);
        // Jewellery accuracy: open boundaries and sharp facet/prong edges never collapse in the LOD.
        let sharp_degrees = std::env::var("OM9_LOD_SHARP_DEGREES")
            .ok()
            .and_then(|value| value.trim().parse::<f64>().ok())
            .unwrap_or(30.0);
        let locked = om9_render::qem::feature_locks(&mesh, sharp_degrees);
        let (simplified, _) =
            om9_render::qem::simplify_with_error_locked(&mesh, target, Some(&locked));
        return (simplified.positions, simplified.indices);
    }
    (positions, indices)
}

/// World units per screen pixel for the document camera.
fn world_per_pixel(
    camera: &om9_scene::camera::Camera,
    viewport: om9_scene::camera::Viewport,
) -> f64 {
    let height = viewport.size[1].max(1.0);
    match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            let offset = [
                camera.eye[0] - camera.target[0],
                camera.eye[1] - camera.target[1],
                camera.eye[2] - camera.target[2],
            ];
            let distance =
                (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
            2.0 * (vertical_fov_rad * 0.5).tan() * distance / height
        }
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            vertical_span_mm / height
        }
    }
}

/// Viewport-only line colours; source and layer colours are not modified.
const SELECTION_PINK: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
const OBJECT_GREEN: [f32; 4] = [0.0, 130.0 / 255.0, 85.0 / 255.0, 1.0];

fn object_line_color(selected: bool) -> [f32; 4] {
    if selected { SELECTION_PINK } else { OBJECT_GREEN }
}

// Display-only midpoint guides for the six rectangular faces of a tessellated box.
// Face IDs keep the triangle diagonals out of the result and out of picking.
fn box_face_guides(vertices: &[[f64; 3]], triangles: &[([u32; 3], u32)]) -> Option<Vec<[[f64; 3]; 2]>> {
    let mut faces = std::collections::BTreeMap::<u32, Vec<u32>>::new();
    if triangles.len() != 12 { return None; }
    for (indices, face) in triangles {
        let points = faces.entry(*face).or_default();
        for index in indices { if !points.contains(index) { points.push(*index); } }
    }
    if faces.len() != 6 || faces.values().any(|face| face.len() != 4) { return None; }
    let mut guides = Vec::with_capacity(12);
    for indices in faces.values() {
        let p: Vec<[f64; 3]> = indices.iter().map(|&i| vertices.get(i as usize).copied()).collect::<Option<_>>()?;
        let dist2 = |a: [f64;3], b: [f64;3]| (0..3).map(|i| (a[i]-b[i]).powi(2)).sum::<f64>();
        let opposite = (1..4).max_by(|&a,&b| dist2(p[0],p[a]).total_cmp(&dist2(p[0],p[b])))?;
        let others: Vec<usize> = (1..4).filter(|&i| i != opposite).collect();
        let (a,b,c,d)=(p[0],p[others[0]],p[opposite],p[others[1]]);
        let scale=dist2(a,c);
        if !scale.is_finite() || scale <= 1e-20 { return None; }
        let midpoint_error: f64 = (0..3).map(|i| (a[i]+c[i]-b[i]-d[i]).powi(2)).sum();
        let dot: f64 = (0..3).map(|i| (b[i]-a[i])*(d[i]-a[i])).sum();
        if midpoint_error > scale*1e-16 || dot.abs() > scale*1e-8 { return None; }
        let mid=|x:[f64;3],y:[f64;3]| std::array::from_fn(|i|(x[i]+y[i])*0.5);
        guides.push([mid(a,b),mid(c,d)]);
        guides.push([mid(a,d),mid(b,c)]);
    }
    Some(guides)
}

// Recognize a full, single-face spherical display mesh by fitting a sphere to
// four well-separated vertices and checking the entire mesh and its extent.
// This is presentation data only and never changes or identifies model geometry.
fn sphere_guides(vertices: &[[f64; 3]], triangles: &[([u32; 3], u32)]) -> Option<Vec<[[f64; 3]; 2]>> {
    if vertices.len() < 8 || triangles.len() < 8 || triangles.iter().any(|t| t.1 != triangles[0].1) { return None; }
    let p = *vertices.first()?;
    let sub = |a: [f64;3], b: [f64;3]| std::array::from_fn(|i| a[i]-b[i]);
    let dot = |a: [f64;3], b: [f64;3]| (0..3).map(|i| a[i]*b[i]).sum::<f64>();
    let cross = |a: [f64;3], b: [f64;3]| [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
    let furthest = |score: &dyn Fn([f64;3]) -> f64| vertices.iter().copied().max_by(|a,b| score(*a).total_cmp(&score(*b)));
    let a = sub(furthest(&|v| dot(sub(v,p),sub(v,p)))?,p);
    let b = sub(furthest(&|v| { let c=cross(a,sub(v,p)); dot(c,c) })?,p);
    let ab = cross(a,b);
    let c = sub(furthest(&|v| dot(ab,sub(v,p)).abs())?,p);
    let det = dot(ab,c);
    let scale = dot(a,a).sqrt();
    if !scale.is_finite() || scale <= 1e-10 || !det.is_finite() || det.abs() < scale.powi(3)*1e-10 { return None; }
    let bc=cross(b,c); let ca=cross(c,a);
    let center: [f64;3]=std::array::from_fn(|i| p[i]+(dot(a,a)*bc[i]+dot(b,b)*ca[i]+dot(c,c)*ab[i])/(2.*det));
    let radius=dot(sub(p,center),sub(p,center)).sqrt();
    if !radius.is_finite() || radius <= 1e-10 { return None; }
    let mut min=[f64::INFINITY;3]; let mut max=[f64::NEG_INFINITY;3];
    for &v in vertices {
        let d=sub(v,center); let r=dot(d,d).sqrt();
        if !r.is_finite() || (r-radius).abs() > radius*1e-6 { return None; }
        for i in 0..3 { min[i]=min[i].min(d[i]); max[i]=max[i].max(d[i]); }
    }
    if (0..3).any(|i| min[i] > -0.95*radius || max[i] < 0.95*radius) { return None; }
    // A trimmed spherical patch may span all three axes too. Its triangles do
    // not subtend the full solid angle, so do not draw guides across a hole.
    let mut solid_angle=0.;
    for (indices,_) in triangles {
        let [i,j,k]=*indices;
        let a=sub(*vertices.get(i as usize)?,center);
        let b=sub(*vertices.get(j as usize)?,center);
        let c=sub(*vertices.get(k as usize)?,center);
        let numerator=dot(a,cross(b,c)).abs();
        let denominator=radius.powi(3)+radius*(dot(a,b)+dot(b,c)+dot(c,a));
        solid_angle+=2.*numerator.atan2(denominator);
    }
    // Allow for floating point accumulation, not a fixed angular area that can
    // hide an entire small missing triangle. This remains a display heuristic,
    // not a topological proof of a complete untrimmed spherical face.
    let angle_roundoff = 128. * f64::EPSILON * triangles.len() as f64;
    if !solid_angle.is_finite() || (solid_angle-4.*std::f64::consts::PI).abs()>angle_roundoff { return None; }
    const SEGMENTS: usize = 96;
    let mut guides=Vec::with_capacity(3*SEGMENTS);
    for axis in 0..3 {
        let (u,v)=((axis+1)%3,(axis+2)%3);
        let point=|i:usize| { let angle=std::f64::consts::TAU*i as f64/SEGMENTS as f64; let mut q=center; q[u]+=radius*angle.cos(); q[v]+=radius*angle.sin(); q };
        for i in 0..SEGMENTS { guides.push([point(i),point(i+1)]); }
    }
    Some(guides)
}

fn build_selected_shape_lines(
    state: &RenderState,
    scene: &om9_scene::scene::Scene,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    samples: u32,
) -> om9_render::overlay::OverlayBatch {
    use om9_scene::scene::RenderBuffers;
    let mut out = om9_render::overlay::OverlayBatch::on_top(state.device.clone(), state.queue.clone(), samples);
    for object in scene.objects.iter().filter(|o| o.visible && selected.contains(&o.root)) {
        let RenderBuffers::Mesh { vertices, triangles, edges } = object.buffers.as_ref() else { continue };
        let box_guides = box_face_guides(vertices, triangles);
        let sphere_guides = if box_guides.is_none() { sphere_guides(vertices, triangles) } else { None };
        let Some(guides) = box_guides.as_ref().or(sphere_guides.as_ref()) else { continue };
        let color = object_line_color(true);
        if box_guides.is_some() {
            for [a,b] in edges {
                if let (Some(a),Some(b))=(vertices.get(*a as usize),vertices.get(*b as usize))
                    && let (Ok(a),Ok(b))=(object.transform.point(*a),object.transform.point(*b)) { out.line(a,b,color); }
            }
        }
        for [a,b] in guides {
            if let (Ok(a),Ok(b))=(object.transform.point(*a),object.transform.point(*b)) { out.line(a,b,color); }
        }
    }
    out.upload_deferred();
    out
}

fn empty_world_lines(state: &RenderState, samples: u32) -> WorldLines {
    let make = || om9_render::overlay::OverlayBatch::with_samples(state.device.clone(), state.queue.clone(), samples);
    WorldLines { fixed: make(), moving: make(), ghost: make(),
        selected_guides: om9_render::overlay::OverlayBatch::on_top(state.device.clone(), state.queue.clone(), samples) }
}

/// World-space mesh and curve lines do not depend on viewport camera or CPlane.
fn build_shared_world_lines(
    state: &RenderState,
    scene: &om9_scene::scene::Scene,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    samples: u32,
) -> WorldLines {
    use om9_scene::scene::RenderBuffers;
    let make = || om9_render::overlay::OverlayBatch::with_samples(state.device.clone(), state.queue.clone(), samples);
    let mut world = WorldLines { fixed: make(), moving: make(), ghost: make(),
        selected_guides: build_selected_shape_lines(state, scene, selected, samples) };
    for object in scene.objects.iter().filter(|object| object.visible) {
        let picked = selected.contains(&object.root);
        let color = object_line_color(picked);
        let out = if picked { &mut world.moving } else { &mut world.fixed };
        match object.buffers.as_ref() {
            RenderBuffers::Curve { points, .. } => {
                for pair in points.windows(2) {
                    if let (Ok(a), Ok(b)) = (object.transform.point(pair[0]), object.transform.point(pair[1])) {
                        out.line(a, b, color);
                        if picked { world.ghost.line(a, b, [0.5, 0.5, 0.5, 1.]); }
                    }
                }
            }
            RenderBuffers::Mesh { vertices, triangles, edges } => {
                let box_guides = box_face_guides(vertices, triangles);
                let sphere_guides = if box_guides.is_none() { sphere_guides(vertices, triangles) } else { None };
                if sphere_guides.is_none() {
                    for [a, b] in edges {
                        if let (Some(a), Some(b)) = (vertices.get(*a as usize), vertices.get(*b as usize))
                            && let (Ok(a), Ok(b)) = (object.transform.point(*a), object.transform.point(*b)) {
                            out.line(a, b, color);
                            if picked { world.ghost.line(a, b, [0.5, 0.5, 0.5, 1.]); }
                        }
                    }
                }
                if let Some(guides) = box_guides.as_ref().or(sphere_guides.as_ref()) {
                    for [a, b] in guides {
                        if let (Ok(a), Ok(b)) = (object.transform.point(*a), object.transform.point(*b)) {
                            out.line(a, b, color);
                        }
                    }
                }
            }
            RenderBuffers::Points { .. } => {}
        }
    }
    world.fixed.upload_deferred();
    world.moving.upload_deferred();
    world.ghost.upload_deferred();
    world
}

/// One object's triangles in world space, or `None` when it has no mesh or no transform.
fn build_lines(
    state: &RenderState,
    scene: Option<&om9_scene::scene::Scene>,
    cplane: om9_scene::camera::CPlane,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    point_size: f64,
    samples: u32,
    share_world: bool,
) -> (
    om9_render::overlay::OverlayBatch,
    om9_render::overlay::OverlayBatch,
    om9_render::overlay::OverlayBatch,
) {
    use om9_scene::scene::RenderBuffers;
    let make = || {
        om9_render::overlay::OverlayBatch::with_samples(
            state.device.clone(),
            state.queue.clone(),
            samples,
        )
    };
    let (mut fixed, mut moving, mut ghost) = (make(), make(), make());
    // Grid first, so coplanar curves are visible over it.
    if let Ok(grid) = om9_scene::grid::Grid::new(cplane) {
        for line in grid.lines {
            fixed.line(line.start, line.end, line.color.map(|x| x as f32 / 255.));
        }
    }
    if let Some(scene) = scene {
        for object in &scene.objects {
            if !object.visible {
                continue;
            }
            let picked = selected.contains(&object.root);
            if share_world && !matches!(object.buffers.as_ref(), RenderBuffers::Points { .. }) {
                continue;
            }
            let box_guides = match object.buffers.as_ref() {
                RenderBuffers::Mesh { vertices, triangles, .. } => box_face_guides(vertices, triangles),
                _ => None,
            };
            let sphere_guides = match object.buffers.as_ref() {
                RenderBuffers::Mesh { vertices, triangles, .. } if box_guides.is_none() => sphere_guides(vertices, triangles),
                _ => None,
            };
            let tint = object_line_color(picked);
            let out = if picked { &mut moving } else { &mut fixed };
            let draw = |batch: &mut om9_render::overlay::OverlayBatch, color| match object
                .buffers
                .as_ref()
            {
                RenderBuffers::Curve { points, .. } => {
                    for pair in points.windows(2) {
                        if let (Ok(a), Ok(b)) = (
                            object.transform.point(pair[0]),
                            object.transform.point(pair[1]),
                        ) {
                            batch.line(a, b, color);
                        }
                    }
                }
                RenderBuffers::Points { points, .. } => {
                    for p in points {
                        if let Ok(p) = object.transform.point(*p) {
                            batch.point(p, point_size, color);
                        }
                    }
                }
                RenderBuffers::Mesh {
                    vertices, edges, ..
                } if sphere_guides.is_none() => {
                    for [a, b] in edges {
                        if let (Some(a), Some(b)) =
                            (vertices.get(*a as usize), vertices.get(*b as usize))
                            && let (Ok(a), Ok(b)) =
                                (object.transform.point(*a), object.transform.point(*b))
                        {
                            batch.line(a, b, color);
                        }
                    }
                }
                _ => {}
            };
            draw(out, tint);
            if let Some(guides) = box_guides.as_ref().or(sphere_guides.as_ref()) {
                for [a,b] in guides {
                    if let (Ok(a),Ok(b))=(object.transform.point(*a),object.transform.point(*b)) {
                        out.line(a,b,tint);
                    }
                }
            }
            if picked {
                draw(&mut ghost, [0.5, 0.5, 0.5, 1.]);
            }
        }
    }
    fixed.upload_deferred();
    moving.upload_deferred();
    ghost.upload_deferred();
    (fixed, moving, ghost)
}

#[cfg(test)]
fn world_mesh(object: &om9_scene::scene::SceneObject) -> Option<om9_render::MeshData> {
    let transform = object.display_transform().ok()?;
    world_mesh_at(object, transform)
}

/// The same mesh under an explicit transform. The drag ghost uses `object.transform`, the placement without
/// the preview, so it stays where the object was when the gesture started.
fn world_mesh_at(
    object: &om9_scene::scene::SceneObject,
    transform: om9_core::geometry::Transform,
) -> Option<om9_render::MeshData> {
    let om9_scene::scene::RenderBuffers::Mesh {
        vertices,
        triangles,
        ..
    } = object.buffers.as_ref()
    else {
        return None;
    };
    let mut positions = Vec::with_capacity(vertices.len());
    for vertex in vertices {
        let point = transform.point(*vertex).ok()?;
        positions.push([point[0] as f32, point[1] as f32, point[2] as f32]);
    }
    if positions.is_empty() || triangles.is_empty() {
        return None;
    }
    let mut indices = Vec::with_capacity(triangles.len() * 3);
    for (corners, _) in triangles {
        indices.extend_from_slice(corners);
    }
    Some(om9_render::MeshData::new(positions, indices))
}

/// The grey copy left where the objects were when a drag started.
///
/// It is built from the document placement (`object.transform`), not from the previewed display transform,
/// so move, rotate and scale all leave one grey object at the starting position while the live object moves.
fn build_ghost(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: Option<&om9_scene::scene::Scene>,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    samples: u32,
) -> om9_render::scene::SceneBatch {
    let mut ghost =
        om9_render::scene::SceneBatch::with_samples(device.clone(), queue.clone(), samples);
    let Some(scene) = scene else {
        return ghost;
    };
    for object in &scene.objects {
        if !selected.contains(&object.root) {
            continue;
        }
        if let Some(mesh) = world_mesh_at(object, object.transform) {
            // Grey and translucent: readable as "the old position", never mistaken for geometry.
            ghost.add_mesh(&mesh, identity_model(), [0.45, 0.45, 0.45, 0.55]);
        }
    }
    ghost
}

/// The model matrix of geometry that is already in world space.
fn identity_model() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Row-major `f64` transform to the `f32` model matrix the renderer takes. The meshes hold world-space
/// vertices, so a model matrix is exactly the transient transform a drag needs.
fn preview_model(delta: &om9_core::geometry::Transform) -> [[f32; 4]; 4] {
    let mut model = [[0.0f32; 4]; 4];
    for (row, values) in delta.0.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            model[row][column] = *value as f32;
        }
    }
    model
}

impl GpuScene {
    fn fork_samples_task(&self, samples: u32) -> Box<dyn FnOnce() -> Self + Send> {
        let batch = self.batch.fork_samples_task(samples);
        let ghost = self.ghost.as_ref().map(|g| g.fork_samples_task(samples));
        let mesh_objects = self.mesh_objects.clone();
        let epoch = self.epoch;
        let cplane = self.cplane;
        let selection = self.selection;
        let selected_objects = self.selected_objects.clone();
        let selected_models = self.selected_models.clone();
        let merged = self.merged;
        let close = self.close;
        Box::new(move || Self {
            batch: batch(),
            ghost: ghost.map(|task| task()),
            mesh_objects,
            epoch,
            cplane,
            selection,
            selected_objects,
            selected_models,
            merged,
            close,
        })
    }
    /// Applies a drag preview without rebuilding any buffer.
    fn apply_preview(&mut self, delta: &om9_core::geometry::Transform) {
        let model = preview_model(delta);
        for (&index, base) in self.selected_objects.iter().zip(&self.selected_models) {
            let composed = std::array::from_fn(|row| {
                std::array::from_fn(|col| (0..4).map(|k| model[row][k] * base[k][col]).sum())
            });
            self.batch.set_transform(index, composed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InFlightInstances, preview_model, viewport_target_minimum_bytes, wgpu};
    use om9_core::geometry::Transform;

    #[test]
    fn worker_inventory_covers_active_completed_and_canceled_preparation() {
        use super::{prepare_task, window_known_buffer_bytes, PreparedGpuBuffers, SharedResourcePool};
        use std::{sync::mpsc, time::{Duration, Instant}};
        fn block_on<F: std::future::Future>(future: F) -> F::Output {
            let mut future = std::pin::pin!(future);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            loop { match future.as_mut().poll(&mut context) {
                std::task::Poll::Ready(value) => return value,
                std::task::Poll::Pending => std::thread::yield_now(),
            }}
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), flags: Default::default(),
            memory_budget_thresholds: Default::default(), backend_options: Default::default(), display: Default::default(),
        });
        let Some(adapter) = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
            compatible_surface: None, apply_limit_buckets: false,
        })).ok() else { eprintln!("SKIP: no GPU adapter"); return };
        let (device, _) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pending preparation"), size: 4096, usage: wgpu::BufferUsages::VERTEX, mapped_at_creation: false,
        });
        struct Fixture(wgpu::Buffer);
        impl PreparedGpuBuffers for Fixture {
            fn prepared_buffers(&self) -> Vec<wgpu::Buffer> { vec![self.0.clone()] }
        }
        let pool = SharedResourcePool::default();
        let inventory = pool.lock().unwrap().preparations.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let job = prepare_task(&inventory, move || {
            started_tx.send(()).unwrap(); release_rx.recv().unwrap(); Fixture(buffer)
        }).unwrap();
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), None, "active worker cannot be inventoried yet");
        release_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while inventory.active.load(std::sync::atomic::Ordering::Acquire) != 0 {
            assert!(Instant::now() < deadline); std::thread::yield_now();
        }
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), Some(4096));
        let completed = job.try_recv().unwrap();
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), Some(4096));
        drop(completed);
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), Some(0));
        drop(job);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canceled preparation"), size: 4096, usage: wgpu::BufferUsages::VERTEX, mapped_at_creation: false,
        });
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let canceled = prepare_task(&inventory, move || {
            started_tx.send(()).unwrap(); release_rx.recv().unwrap(); Fixture(buffer)
        }).unwrap();
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(canceled);
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), None, "canceled worker remains active until it exits");
        release_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while inventory.active.load(std::sync::atomic::Ordering::Acquire) != 0 {
            assert!(Instant::now() < deadline); std::thread::yield_now();
        }
        assert_eq!(window_known_buffer_bytes(&pool, std::iter::empty()), Some(0));
    }

    #[test]
    fn target_minimum_includes_two_colours_and_msaa_depth() {
        assert_eq!(viewport_target_minimum_bytes(320, 240, 1), Some(320 * 240 * 12));
        assert_eq!(viewport_target_minimum_bytes(320, 240, 4), Some(320 * 240 * 40));
        assert_eq!(viewport_target_minimum_bytes(0, 240, 4), None);
        assert_eq!(viewport_target_minimum_bytes(u32::MAX, u32::MAX, u32::MAX), None);
    }

    #[test]
    fn the_preview_matrix_keeps_the_row_major_origin() {
        let delta = Transform::translation([2.0, -1.0, 0.5]);
        let model = preview_model(&delta);
        assert_eq!(model[0][3], 2.0);
        assert_eq!(model[1][3], -1.0);
        assert_eq!(model[2][3], 0.5);
        assert_eq!(model[3], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(model[0][0], 1.0);
    }

    #[test]
    fn overlapping_submissions_keep_one_charge_until_both_finish() {
        fn block_on<F: std::future::Future>(future: F) -> F::Output {
            let mut future = std::pin::pin!(future);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            loop {
                match future.as_mut().poll(&mut context) {
                    std::task::Poll::Ready(value) => return value,
                    std::task::Poll::Pending => std::thread::yield_now(),
                }
            }
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), flags: Default::default(),
            memory_budget_thresholds: Default::default(), backend_options: Default::default(), display: Default::default(),
        });
        let Some(adapter) = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
            compatible_surface: None, apply_limit_buckets: false,
        })).ok() else { eprintln!("SKIP: no GPU adapter"); return };
        let (device, _) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default()
        })).unwrap();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlapping instance submissions"), size: 160,
            usage: wgpu::BufferUsages::VERTEX, mapped_at_creation: false,
        });
        let mut tracker = InFlightInstances::default();
        tracker.retain_submission(std::slice::from_ref(&buffer));
        tracker.retain_submission(std::slice::from_ref(&buffer));
        assert_eq!(tracker.totals(), (160, 1));
        tracker.complete_submission(std::slice::from_ref(&buffer));
        assert_eq!(tracker.totals(), (160, 1));
        tracker.complete_submission(std::slice::from_ref(&buffer));
        assert_eq!(tracker.totals(), (0, 0));
    }
}

#[cfg(test)]
mod ghost_tests {
    use super::{world_mesh, world_mesh_at};
    use om9_core::{
        budget::Limits,
        geometry::{Geometry, Mesh, Transform},
        model::{Document, Entity},
        transaction::{TransactionCoordinator, World},
    };
    use om9_scene::scene::Scene;

    fn cube() -> Mesh {
        Mesh {
            vertices: vec![
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            faces: vec![
                vec![0, 2, 1],
                vec![0, 3, 2],
                vec![4, 5, 6],
                vec![4, 6, 7],
                vec![0, 1, 5],
                vec![0, 5, 4],
                vec![2, 3, 7],
                vec![2, 7, 6],
                vec![1, 2, 6],
                vec![1, 6, 5],
                vec![0, 4, 7],
                vec![0, 7, 3],
            ],
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        }
    }

    #[test]
    fn the_ghost_uses_the_placement_and_the_live_mesh_uses_the_preview() {
        let mut document = Document::new("ghost");
        let mut entity = Entity::new("cube", Geometry::Mesh(cube()), None);
        entity.placement = Transform::translation([2.0, 0.0, 0.0]);
        let id = entity.id;
        document.entities.insert(id, entity);
        let document_id = document.id;
        let coordinator = TransactionCoordinator::new(
            World::new(vec![document], &Limits::default()).expect("world"),
            Limits::default(),
        )
        .expect("coordinator");
        let mut scene = Scene::capture_in_world(
            &coordinator,
            document_id,
            om9_core::budget::Usage::default(),
        )
        .expect("scene");
        let object = scene.objects.first_mut().expect("object");
        object.preview = Transform::translation([5.0, 0.0, 0.0]);

        let ghost = world_mesh_at(object, object.transform).expect("ghost mesh");
        let live = world_mesh(object).expect("live mesh");
        // Placement puts the cube at x = 2, the preview adds 5 on top of it for the drawn object.
        assert!(
            (ghost.positions[0][0] - 1.5).abs() < 1e-5,
            "ghost at the placement: {:?}",
            ghost.positions[0]
        );
        assert!(
            (live.positions[0][0] - 6.5).abs() < 1e-5,
            "live mesh at the preview: {:?}",
            live.positions[0]
        );
    }
}

#[cfg(test)]
mod display_lod_tests {
    use super::local_display_mesh;

    #[test]
    fn a_display_lod_reduces_the_triangles_and_keeps_the_full_mesh_without_a_target() {
        let sphere = om9_render::shapes::sphere(32, 16, 1.0);
        let vertices: Vec<[f64; 3]> = sphere
            .positions
            .iter()
            .map(|point| [point[0] as f64, point[1] as f64, point[2] as f64])
            .collect();
        let triangles: Vec<([u32; 3], u32)> = sphere
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
            .map(|(face, corners)| ([corners[0], corners[1], corners[2]], face as u32))
            .collect();
        let full = triangles.len();

        let (positions, indices) = local_display_mesh(&vertices, &triangles, Some(64));
        assert!(!positions.is_empty());
        assert!(
            indices.len() / 3 < full,
            "LOD kept {} of {full} triangles",
            indices.len() / 3
        );
        assert!(
            indices.len() / 3 <= 80,
            "LOD overshot its target: {} triangles",
            indices.len() / 3
        );

        let (_, untouched) = local_display_mesh(&vertices, &triangles, None);
        assert_eq!(
            untouched.len(),
            full * 3,
            "without a target every triangle stays"
        );
    }
}

#[cfg(test)]
mod selected_lod_tests {
    use super::display_lod_target;

    #[test]
    fn a_selected_object_never_receives_a_lod_target() {
        assert_eq!(display_lod_target(true, Some(100), Some(50)), None);
        assert_eq!(display_lod_target(false, Some(100), Some(50)), Some(100));
        assert_eq!(display_lod_target(false, None, Some(50)), Some(50));
        assert_eq!(display_lod_target(false, None, None), None);
    }
}

#[cfg(test)]
mod selection_colour_tests {
    use super::{object_line_color, SELECTION_PINK};

    #[test]
    fn object_line_colours_are_green_or_magenta() {
        assert_eq!(object_line_color(false), [0., 130. / 255., 85. / 255., 1.]);
        assert_eq!(object_line_color(true), [1., 0., 1., 1.]);
    }

    /// Matrix9 marks a selected object with its pink highlight; a yellow selection reads as unselected
    /// geometry from the manual's own colours, so the rule is pinned here.
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn the_selection_colour_is_pink_not_yellow() {
        let [red, green, blue, alpha] = SELECTION_PINK;
        assert!(red > 0.9, "the selection keeps a full red channel: {red}");
        assert!(
            blue > 0.7 && blue > green + 0.25,
            "the selection is pink, not yellow: {SELECTION_PINK:?}"
        );
        assert!(
            green < 0.5,
            "a yellow selection has its green channel near red: {green}"
        );
        assert_eq!(alpha, 1.0, "lines and points stay opaque");
    }

    /// The tint the app hands the GPU must actually reach the image as pink; a hue mistake in either
    /// constant would otherwise only show up in the running viewport.
    #[test]
    fn the_renderer_paints_the_selection_pink() {
        let Some(mut renderer) = om9_render::Renderer::new() else {
            eprintln!("no wgpu adapter on this machine; the manual viewport check covers it");
            return;
        };
        // Big enough to cover a large part of the texture, as the sphere does in the render loop test.
        let mesh = om9_render::MeshData::new(
            vec![[-6.0, -5.0, 0.0], [6.0, -5.0, 0.0], [0.0, 6.0, 0.0]],
            vec![0, 1, 2],
        );
        let style = om9_render::RenderStyle {
            tint: SELECTION_PINK,
            ..Default::default()
        };
        let frame = renderer
            .render_with(1, &mesh, om9_render::Camera::default(), 320, 180, style)
            .expect("selection frame");
        let mut pink = 0;
        for y in 0..frame.height {
            for x in 0..frame.width {
                let [red, green, blue, _] = frame.pixel(x, y);
                if red > 60 && blue > 50 && blue as u32 * 2 > red as u32 && green < red {
                    pink += 1;
                }
            }
        }
        assert!(
            pink > 5_000,
            "the selected object must be pink, counted {pink} pink pixels"
        );
    }
}

#[cfg(all(test, feature = "occt-brep"))]
mod box_guide_tests {
    #[test]
    fn rectangular_brep_gets_two_midlines_per_face_without_tessellation_diagonals() {
        use om9_kernel::creation::{primitive, Primitive, DEFAULT_DEFLECTION_MM};
        let b = primitive(Primitive::Box { x_mm: 4., y_mm: 6., z_mm: 8. }, DEFAULT_DEFLECTION_MM, &om9_core::budget::Limits::default()).unwrap();
        let triangles: Vec<_> = b.display.triangles.iter().map(|[a,b,c,f]| ([*a,*b,*c],*f)).collect();
        let guides = super::box_face_guides(&b.display.points, &triangles).expect("six rectangular faces");
        assert_eq!(guides.len(), 12);
        for [a,b] in guides {
            assert_eq!((0..3).filter(|&i| (a[i]-b[i]).abs() > 1e-9).count(),1);
        }
        assert!(super::box_face_guides(&b.display.points, &triangles[..10]).is_none());
    }

    #[test]
    fn sphere_gets_three_great_circles_and_ellipsoid_is_not_misidentified() {
        use om9_kernel::creation::{primitive, Primitive, DEFAULT_DEFLECTION_MM};
        let b=primitive(Primitive::Sphere {radius_mm: 5.},DEFAULT_DEFLECTION_MM,&om9_core::budget::Limits::default()).unwrap();
        let triangles:Vec<_>=b.display.triangles.iter().map(|[a,b,c,f]| ([*a,*b,*c],*f)).collect();
        let guides=super::sphere_guides(&b.display.points,&triangles).expect("full sphere");
        assert_eq!(guides.len(),288);
        for axis in 0..3 { assert!(guides[axis*96..(axis+1)*96].iter().flatten().all(|p|p[axis].abs()<1e-8)); }
        for [a,b] in &guides { for p in [a,b] {assert!((p.iter().map(|v|v*v).sum::<f64>().sqrt()-5.).abs()<1e-8);} }
        let ellipsoid:Vec<_>=b.display.points.iter().map(|p|[p[0]*2.,p[1],p[2]]).collect();
        assert!(super::sphere_guides(&ellipsoid,&triangles).is_none());
        let translated:Vec<_>=b.display.points.iter().map(|p|[p[0]+12.,p[1]-7.,p[2]+3.]).collect();
        let moved=super::sphere_guides(&translated,&triangles).expect("translated sphere");
        assert!(moved.iter().flatten().all(|p| ((p[0]-12.).powi(2)+(p[1]+7.).powi(2)+(p[2]-3.).powi(2)-25.).abs()<1e-7));
        assert!(super::sphere_guides(&b.display.points,&triangles[..triangles.len()-10]).is_none());
        assert!(super::sphere_guides(&b.display.points,&[]).is_none());
    }

    #[test]
    fn spherical_mesh_missing_a_small_triangle_has_no_full_sphere_guides() {
        use om9_kernel::creation::{primitive, Primitive};
        let b = primitive(Primitive::Sphere { radius_mm: 5. }, 0.1, &om9_core::budget::Limits::default()).unwrap();
        let mut points = b.display.points.clone();
        let mut triangles: Vec<_> = b.display.triangles.iter().map(|[a,b,c,f]| ([*a,*b,*c],*f)).collect();
        let area = |indices: [u32; 3]| {
            let [a,b,c] = indices.map(|i| b.display.points[i as usize]);
            let u = std::array::from_fn::<_, 3, _>(|i| b[i]-a[i]);
            let v = std::array::from_fn::<_, 3, _>(|i| c[i]-a[i]);
            let cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
            cross.iter().map(|x| x*x).sum::<f64>()
        };
        let largest = triangles.iter().enumerate().max_by(|(_,a),(_,b)| area(a.0).total_cmp(&area(b.0))).unwrap().0;
        let mut child = triangles.swap_remove(largest);
        for _ in 0..7 {
            let [a,b,c] = child.0;
            let mut midpoint = |i: u32, j: u32| {
                let p = std::array::from_fn::<_, 3, _>(|k| (points[i as usize][k] + points[j as usize][k]) * 0.5);
                let scale = 5. / p.iter().map(|v| v*v).sum::<f64>().sqrt();
                points.push(p.map(|v| v*scale));
                (points.len() - 1) as u32
            };
            let ab = midpoint(a,b); let bc = midpoint(b,c); let ca = midpoint(c,a);
            triangles.extend([([a,ab,ca],child.1), ([ab,b,bc],child.1), ([ca,bc,c],child.1)]);
            child = ([ab,bc,ca],child.1);
        }
        triangles.push(child);
        assert!(super::sphere_guides(&points, &triangles).is_some(), "refined full sphere must retain its guides");
        triangles.pop();
        assert!(super::sphere_guides(&points, &triangles).is_none(), "a missing nondegenerate triangle must not be covered by full-sphere guides");
    }
}
