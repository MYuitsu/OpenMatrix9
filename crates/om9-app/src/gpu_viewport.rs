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

/// One viewport's GPU target and its registration in egui.
pub struct GpuViewport {
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
    last_camera: Option<om9_scene::camera::Camera>,
    /// Scene epoch and CPlane revision of the last rendered frame: new geometry must repaint even when the
    /// camera did not move.
    last_epoch: Option<u64>,
    last_cplane: Option<u64>,
    /// Selection revision of the last render: the highlight is part of the GPU image.
    last_selection: Option<u64>,
    /// Revision of the transient gumball preview: a drag must repaint every viewport.
    last_preview: Option<u64>,
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
    overlays: Option<ViewOverlays>,
    overlay_job: Option<(OverlayKey, PrepareReceiver<ViewOverlays>)>,
    published_id: Option<egui::TextureId>,
    published_colour: Option<wgpu::Texture>,
    last_profile: Option<(u32, bool)>,
    rendered_final: bool,
    preparing: bool,
    preparation_error: Option<String>,
    last_document: Option<om9_core::identity::DocumentId>,
    displayed_camera: Option<om9_scene::camera::Camera>,
    refinement: Option<(RefinementKey, std::time::Instant)>,
    refinement_ms: Option<f64>,
    timer: Option<om9_render::timing::GpuTimer>,
    timer_frame: Option<u64>,
    pub gpu_sample: Option<(u64, f64)>,
    pub encoded_frame: Option<u64>,
    render_stats: Option<om9_render::scene::FrameStats>,
    pub published_frame: Option<u64>,
    pending_publication: Option<PendingPublication>,
}
struct PendingPublication {
    key: RefinementKey,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    submitted: bool,
    texture: egui::TextureId,
    colour: wgpu::Texture,
    camera: om9_scene::camera::Camera,
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
    size: (u32, u32),
    generation: u64,
}

/// The one document scene the four viewports render with their own cameras.
pub type SharedScene = std::sync::Arc<std::sync::Mutex<Option<GpuScene>>>;
pub type SharedResourcePool = std::sync::Arc<std::sync::Mutex<ResourcePool>>;
#[derive(Default)]
pub struct ResourcePool {
    encoder: Option<wgpu::CommandEncoder>,
    budget: Option<UploadBudget>,
    signature: Option<(om9_core::identity::DocumentId, u64, u64)>,
    entries: std::collections::BTreeMap<u32, ResourceEntry>,
}
pub fn submit_window(pool: &SharedResourcePool, state: &RenderState) -> bool {
    let encoder = pool
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .encoder
        .take();
    if let Some(encoder) = encoder {
        state.queue.submit([encoder.finish()]);
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
#[derive(Clone, Copy, Debug)]
pub struct ViewportRenderRequest {
    pub frame_id: u64,
    pub generation: u64,
    pub moving: bool,
    pub render_due: bool,
    pub quality: crate::frame_scheduler::PreviewQuality,
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
type OverlayKey = (om9_core::identity::DocumentId, u64, u64, u64, u32, u64);
struct ViewOverlays {
    key: OverlayKey,
    fixed: om9_render::overlay::OverlayBatch,
    moving: om9_render::overlay::OverlayBatch,
    ghost: om9_render::overlay::OverlayBatch,
}
type PrepareTask = Box<dyn FnOnce() + Send>;
struct PrepareReceiver<T> {
    receiver: std::sync::mpsc::Receiver<T>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl<T> PrepareReceiver<T> {
    fn try_recv(&self) -> Result<T, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
impl<T> Drop for PrepareReceiver<T> {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
fn prepare_task<T: Send + 'static>(
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
    worker
        .try_send(Box::new(move || {
            if obsolete.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }
            let result = task();
            if !obsolete.load(std::sync::atomic::Ordering::Acquire) {
                let _ = tx.send(result);
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
    /// Only the selected objects, drawn over the scene with alpha blending. Selection changes rebuild this
    /// small batch instead of the whole scene.
    highlight: om9_render::scene::SceneBatch,
    overlay: om9_render::overlay::OverlayBatch,
    moving_lines: om9_render::overlay::OverlayBatch,
    ghost_lines: om9_render::overlay::OverlayBatch,
    point_size: f64,
    mesh_objects: std::sync::Arc<Vec<(om9_core::identity::ObjectId, usize)>>,
    epoch: u64,
    cplane: u64,
    selection: u64,
    /// Batch object indices of the selected meshes and of the highlight copies. A drag preview only rewrites
    /// their model matrix, which is a handful of uniform writes instead of a scene rebuild.
    selected_objects: Vec<usize>,
    selected_models: Vec<[[f32; 4]; 4]>,
    highlight_objects: Vec<usize>,
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
            last_camera: None,
            last_epoch: None,
            last_cplane: None,
            last_selection: None,
            last_preview: None,
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
            last_profile: None,
            rendered_final: false,
            preparing: false,
            preparation_error: None,
            last_document: None,
            displayed_camera: None,
            refinement: None,
            refinement_ms: None,
            timer: om9_render::timing::GpuTimer::new(&state.device),
            timer_frame: None,
            gpu_sample: None,
            encoded_frame: None,
            render_stats: None,
            published_frame: None,
            pending_publication: None,
        }
    }

    /// Samples of the document viewport's targets; the stress paths keep one sample and their own depth.
    fn sample_count(&self) -> u32 {
        if self.batch.is_none() && self.cluster.is_none() {
            if self.request.moving {
                self.request.quality.samples()
            } else {
                4
            }
        } else {
            1
        }
    }

    /// Recreates the targets when the rect changed, and registers the new view with egui.
    fn ensure_targets(&mut self, state: &RenderState, width: u32, height: u32) {
        if self.size == (width, height)
            && self.colour_view.is_some()
            && self.last_profile.map(|profile| profile.0) == Some(self.sample_count())
        {
            return;
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
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

    /// Rebuilds the GPU copy of the document scene when the scene epoch or the CPlane revision changed.
    #[allow(clippy::too_many_arguments)]
    fn refresh_scene(
        state: &RenderState,
        slot: &mut Option<GpuScene>,
        camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
        cplane: om9_scene::camera::CPlane,
        viewport: om9_scene::camera::Viewport,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection_revision: u64,
        samples: u32,
        moving: bool,
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
            let point_size = world_per_pixel(camera, viewport) * 5.0;
            if current.selection == selection_revision {
                // Only world-space point markers depend on the zoom; the grid and the curve lines do
                // not. A zoom used to rebuild every line (13,000 curves for 1,000 earrings) because the
                // point size changed, which cost about 700 ms per frame. Rebuild the overlay only when
                // the scene actually has point objects and their size moved by more than 5%.
                let has_points = scene.is_some_and(|scene| {
                    scene.objects.iter().any(|object| {
                        matches!(
                            object.buffers.as_ref(),
                            om9_scene::scene::RenderBuffers::Points { .. }
                        )
                    })
                });
                let resized = (current.point_size - point_size).abs() > point_size * 0.05;
                if has_points && resized && !moving {
                    if let Some(current) = slot.as_mut() {
                        let (overlay, moving, ghost) =
                            build_lines(state, scene, cplane, selected, point_size, samples);
                        current.overlay = overlay;
                        current.moving_lines = moving;
                        current.ghost_lines = ghost;
                        current.point_size = point_size;
                    }
                } else if let Some(current) = slot.as_mut() {
                    current.point_size = point_size;
                }
                return;
            }
            if !current.merged {
                if let Some(current) = slot.as_mut() {
                    current.apply_preview(&om9_core::geometry::Transform::identity());
                    current.highlight =
                        build_highlight(&state.device, &state.queue, scene, selected, samples);
                    current.highlight_objects = (0..current.highlight.object_count()).collect();
                    current.selected_objects = current
                        .mesh_objects
                        .iter()
                        .filter(|(id, _)| selected.contains(id))
                        .map(|(_, index)| *index)
                        .collect();
                    let point_size = world_per_pixel(camera, viewport) * 5.0;
                    let (overlay, moving, ghost) =
                        build_lines(state, scene, cplane, selected, point_size, samples);
                    current.overlay = overlay;
                    current.moving_lines = moving;
                    current.ghost_lines = ghost;
                    current.point_size = point_size;
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
        let point_size = world_per_pixel(camera, viewport) * 5.0;
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
            let spheres = scene.object_spheres();
            let focus_radius = if close { focus } else { f64::INFINITY };
            for (object_index, object) in scene.objects.iter().enumerate() {
                if !object.visible {
                    continue;
                }
                if close
                    && !selected.contains(&object.root)
                    && let Some((centre, radius)) = spheres.get(object_index)
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
        // Keep the selected line geometry separate so preview transforms never rebuild its buffer.
        let make = || {
            om9_render::overlay::OverlayBatch::with_samples(
                state.device.clone(),
                state.queue.clone(),
                samples,
            )
        };
        let (overlay, moving_lines, ghost_lines) = (make(), make(), make());
        let highlight = build_highlight(&state.device, &state.queue, scene, selected, samples);
        let highlight_objects = (0..highlight.object_count()).collect();
        *slot = Some(GpuScene {
            batch,
            highlight,
            overlay,
            moving_lines,
            ghost_lines,
            point_size,
            mesh_objects: std::sync::Arc::new(mesh_objects),
            epoch,
            cplane: cplane_revision,
            selection: selection_revision,
            selected_objects: selected_indices,
            selected_models,
            highlight_objects,
            ghost: Some(build_ghost(
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
            let spheres = scene.object_spheres();
            if spheres.is_empty() {
                return false;
            }
            let mut centre = [0.0f64; 3];
            for (point, _) in spheres {
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
            self.pending_publication = None;
        }
        self.moving = request.moving;
        self.request = request;
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
    pub fn displayed_camera(&self) -> Option<&om9_scene::camera::Camera> {
        self.displayed_camera.as_ref()
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
                s.batch.pending_upload_bytes() + s.highlight.pending_upload_bytes()
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
        scene: Option<&om9_scene::scene::Scene>,
        camera: &om9_scene::camera::Camera,
        cplane: om9_scene::camera::CPlane,
        viewport: om9_scene::camera::Viewport,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection: u64,
        samples: u32,
    ) -> bool {
        let Some(scene) = scene else {
            return false;
        };
        let signature = (scene.document.id, scene.epoch, selection);
        let mut pool = self.pool.lock().unwrap_or_else(|e| e.into_inner());
        if pool.signature != Some(signature) {
            pool.signature = Some(signature);
            pool.entries.clear();
        }
        if !pool.entries.contains_key(&samples) {
            let source = pool.entries.values().find_map(|e| {
                let slot = e.scene.lock().unwrap_or_else(|e| e.into_inner());
                let gpu = slot.as_ref()?;
                (gpu.batch.pending_upload_bytes() == 0
                    && gpu.highlight.pending_upload_bytes() == 0
                    && gpu
                        .ghost
                        .as_ref()
                        .is_none_or(|g| g.pending_upload_bytes() == 0))
                .then(|| gpu.fork_samples_task(samples))
            });
            if let Some(task) = source {
                pool.entries.insert(
                    samples,
                    ResourceEntry {
                        scene: Default::default(),
                        job: prepare_task(task),
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
            let snapshot = scene.render_snapshot();
            let state = state.clone();
            let camera = camera.clone();
            let selected = selected.clone();
            entry.job = prepare_task(move || {
                let mut slot = None;
                Self::refresh_scene(
                    &state,
                    &mut slot,
                    &camera,
                    Some(&snapshot),
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
                Ok(scene) => {
                    *entry.scene.lock().unwrap_or_else(|e| e.into_inner()) = Some(scene);
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
        );
        // Orbit/pan preserve point size; moving zoom reuses its last point markers until settling.
        let overlay_valid = self.overlays.as_ref().is_some_and(|o| {
            o.key == key
                || (self.moving
                    && o.key.0 == key.0
                    && o.key.1 == key.1
                    && o.key.2 == key.2
                    && o.key.3 == key.3
                    && o.key.4 == key.4)
        });
        if !overlay_valid {
            if self.overlay_job.as_ref().is_some_and(|(k, _)| {
                *k != key
                    && !(self.moving
                        && k.0 == key.0
                        && k.1 == key.1
                        && k.2 == key.2
                        && k.3 == key.3
                        && k.4 == key.4)
            }) {
                self.overlay_job = None;
            }
            if self.overlay_job.is_none() {
                let shared = self.overlays.as_ref().filter(|o| {
                    o.key.0 == key.0
                        && o.key.1 == key.1
                        && o.key.2 == key.2
                        && o.key.3 == key.3
                        && o.key.5 == key.5
                        && o.fixed.pending_upload_bytes() == 0
                        && o.moving.pending_upload_bytes() == 0
                        && o.ghost.pending_upload_bytes() == 0
                });
                if let Some(source) = shared {
                    let fixed = source.fixed.fork_samples_task(samples);
                    let moving = source.moving.fork_samples_task(samples);
                    let ghost = source.ghost.fork_samples_task(samples);
                    self.overlay_job = prepare_task(move || ViewOverlays {
                        key,
                        fixed: fixed(),
                        moving: moving(),
                        ghost: ghost(),
                    })
                    .map(|rx| (key, rx));
                } else {
                    let snapshot = scene.render_snapshot();
                    let state = state.clone();
                    let selected = selected.clone();
                    self.overlay_job = prepare_task(move || {
                        let (fixed, moving, ghost) = build_lines(
                            &state,
                            Some(&snapshot),
                            cplane,
                            &selected,
                            point_size,
                            samples,
                        );
                        ViewOverlays {
                            key,
                            fixed,
                            moving,
                            ghost,
                        }
                    })
                    .map(|rx| (key, rx));
                }
            }
        }
        if let Some((job_key, job)) = self.overlay_job.as_ref() {
            match job.try_recv() {
                Ok(overlays) => {
                    if *job_key == key
                        || (self.moving
                            && job_key.0 == key.0
                            && job_key.1 == key.1
                            && job_key.2 == key.2
                            && job_key.3 == key.3
                            && job_key.4 == key.4)
                    {
                        self.overlays = Some(overlays);
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
        let overlay_ready = self.overlays.as_ref().is_some_and(|o| {
            o.key == key
                || (self.moving
                    && o.key.0 == key.0
                    && o.key.1 == key.1
                    && o.key.2 == key.2
                    && o.key.3 == key.3
                    && o.key.4 == key.4)
        });
        if !geometry_ready || !overlay_ready {
            return false;
        }
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
        for batch in [&mut gpu.batch, &mut gpu.highlight] {
            budget.left -= batch.flush_uploads_budget(budget.left, budget.deadline);
        }
        if let Some(ghost) = gpu.ghost.as_mut() {
            budget.left -= ghost.flush_uploads_budget(budget.left, budget.deadline);
        }
        let overlays = self.overlays.as_mut().unwrap();
        for batch in [
            &mut overlays.fixed,
            &mut overlays.moving,
            &mut overlays.ghost,
        ] {
            budget.left -= batch.flush_uploads_budget(budget.left, budget.deadline);
        }
        gpu.batch.pending_upload_bytes() == 0
            && gpu.highlight.pending_upload_bytes() == 0
            && gpu
                .ghost
                .as_ref()
                .is_none_or(|g| g.pending_upload_bytes() == 0)
            && overlays.fixed.pending_upload_bytes() == 0
            && overlays.moving.pending_upload_bytes() == 0
            && overlays.ghost.pending_upload_bytes() == 0
    }

    /// Draws the scene into the GPU target and paints it into `rect`.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        state: &RenderState,
        camera: Camera,
        revision: u64,
        document_camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
        cplane: om9_scene::camera::CPlane,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection_revision: u64,
        preview_revision: u64,
        _close: bool,
    ) -> bool {
        let pixels_per_point = ui.ctx().pixels_per_point();
        if self.pending_publication.is_some() {
            let _ = state.device.poll(wgpu::PollType::Poll);
        }
        if let Some(timer) = self.timer.as_mut()
            && let Some(sample) = timer.poll(&state.device, &state.queue)
        {
            self.gpu_sample = Some(sample);
        }
        let scale = if self.request.moving {
            self.request.quality.scale()
        } else {
            1.
        };
        let width = (rect.width() * pixels_per_point * scale).round().max(1.0) as u32;
        let height = (rect.height() * pixels_per_point * scale).round().max(1.0) as u32;
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
            self.rendered_final = false;
            self.last_document = document_id;
            self.render_stats = None;
            self.last_revision = None;
            self.refinement = None;
            self.pending_publication = None;
            self.preparation_error = None;
        }
        let unchanged = self.last_revision == Some(revision)
            && self.last_camera.as_ref() == Some(document_camera)
            && self.size == (width, height)
            && self.last_epoch == Some(epoch)
            && self.last_cplane == Some(cplane.revision)
            && self.last_selection == Some(selection_revision)
            && self.last_preview == Some(preview_revision);
        let document = self.batch.is_none() && self.cluster.is_none();
        if document {
            self.preparing = false;
            if !unchanged || self.request.moving {
                self.rendered_final = false;
            }
            let key = document_id.map(|document| RefinementKey {
                document,
                scene: epoch,
                camera: revision,
                selection: selection_revision,
                cplane: cplane.revision,
                preview: preview_revision,
                size: (width, height),
                generation: self.request.generation,
            });
            if !self.request.moving
                && !self.rendered_final
                && let Some(key) = key
                && self.refinement.as_ref().is_none_or(|(old, _)| *old != key)
            {
                self.refinement = Some((key, std::time::Instant::now()));
            }
            if let Some(pending) = self.pending_publication.take()
                && Some(pending.key) == key
                && pending.camera == *document_camera
                && !self.request.moving
            {
                if pending.done.load(std::sync::atomic::Ordering::Acquire) {
                    if let Some(old) = self.published_id.replace(pending.texture)
                        && old != pending.texture
                    {
                        state.renderer.write().free_texture(&old);
                    }
                    self.published_colour = Some(pending.colour);
                    self.displayed_camera = Some(pending.camera);
                    self.rendered_final = true;
                    self.published_frame = Some(self.request.frame_id);
                    if let Some((_, started)) = self.refinement.take() {
                        self.refinement_ms = Some(started.elapsed().as_secs_f64() * 1000.);
                    }
                } else {
                    self.pending_publication = Some(pending);
                    self.preparing = true;
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(2));
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
                scene,
                document_camera,
                cplane,
                viewport,
                selected,
                selection_revision,
                samples,
            ) {
                self.preparing = true;
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(2));
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
                    });
                    self.preparing = true;
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(2));
                    return self.paint_published(ui, rect);
                }
            }
            let changed = !unchanged || self.published_id != self.texture_id;
            if changed {
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
                if gpu
                    .batch
                    .encode_pass(
                        encoder,
                        &render_colour,
                        resolve,
                        &depth,
                        camera,
                        width,
                        height,
                        true,
                    )
                    .is_err()
                {
                    return false;
                }
                gpu.highlight
                    .encode_pass(
                        encoder,
                        &render_colour,
                        resolve,
                        &depth,
                        camera,
                        width,
                        height,
                        false,
                    )
                    .ok();
                if delta.0 != om9_core::geometry::Transform::identity().0
                    && let Some(ghost) = gpu.ghost.as_mut()
                {
                    ghost
                        .encode_pass(
                            encoder,
                            &render_colour,
                            resolve,
                            &depth,
                            camera,
                            width,
                            height,
                            false,
                        )
                        .ok();
                }
                overlays.fixed.encode_pass(
                    encoder,
                    &render_colour,
                    resolve,
                    &depth,
                    camera,
                    width,
                    height,
                );
                if delta.0 != om9_core::geometry::Transform::identity().0 {
                    overlays.ghost.encode_pass(
                        encoder,
                        &render_colour,
                        resolve,
                        &depth,
                        camera,
                        width,
                        height,
                    );
                }
                overlays.moving.encode_pass(
                    encoder,
                    &render_colour,
                    resolve,
                    &depth,
                    camera,
                    width,
                    height,
                );
                if timed {
                    let timer = self.timer.as_ref().unwrap();
                    timer.marker(encoder, &render_colour, true);
                    timer.resolve(encoder);
                    self.timer_frame = Some(self.request.frame_id);
                }
                drop(pool);
                self.last_draw_ms = started.elapsed().as_secs_f64() * 1000.;
                self.encoded_frame = Some(self.request.frame_id);
                self.render_stats = Some(gpu.batch.stats());
                self.last_revision = Some(revision);
                self.last_camera = Some(document_camera.clone());
                self.last_epoch = Some(epoch);
                self.last_cplane = Some(cplane.revision);
                self.last_selection = Some(selection_revision);
                self.last_preview = Some(preview_revision);
                drop(slot);
                if self.request.moving {
                    if let Some(old) = self.published_id.replace(texture_id)
                        && old != texture_id
                    {
                        state.renderer.write().free_texture(&old);
                    }
                    self.published_colour = self.colour.clone();
                    self.displayed_camera = Some(document_camera.clone());
                    self.published_frame = Some(self.request.frame_id);
                } else if let (Some(key), Some(colour)) = (key, self.colour.clone()) {
                    self.pending_publication = Some(PendingPublication {
                        key,
                        done: Default::default(),
                        submitted: false,
                        texture: texture_id,
                        colour,
                        camera: document_camera.clone(),
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
        self.last_camera = Some(document_camera.clone());
        self.last_epoch = Some(epoch);
        self.last_cplane = Some(cplane.revision);
        self.last_selection = Some(selection_revision);
        self.last_preview = Some(preview_revision);
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

/// Matrix9's selection pink (`#FF79C6`): the manual's "pink highlighted color" is reserved for "this
/// object is selected", so only curve/point/mesh-edge lines use it.
const SELECTION_PINK: [f32; 4] = [1.0, 0.475, 0.776, 1.0];

/// One object's triangles in world space, or `None` when it has no mesh or no transform.
fn build_lines(
    state: &RenderState,
    scene: Option<&om9_scene::scene::Scene>,
    cplane: om9_scene::camera::CPlane,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    point_size: f64,
    samples: u32,
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
            let tint = if picked {
                SELECTION_PINK
            } else {
                object.color.map(|x| x as f32 / 255.)
            };
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
                } if picked => {
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

/// Rebuilds only the selection overlay: one batch with the selected meshes, which is what changes when the
/// user clicks. The scene batch stays untouched.
fn build_highlight(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    _scene: Option<&om9_scene::scene::Scene>,
    _selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    samples: u32,
) -> om9_render::scene::SceneBatch {
    // Selected solids keep their face colours; `build_lines` draws their B-Rep edges in
    // the selection pink instead of tinting every triangle.
    om9_render::scene::SceneBatch::with_samples(device.clone(), queue.clone(), samples)
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
        let highlight = self.highlight.fork_samples_task(samples);
        let ghost = self.ghost.as_ref().map(|g| g.fork_samples_task(samples));
        let overlay = self.overlay.fork_samples_task(samples);
        let moving = self.moving_lines.fork_samples_task(samples);
        let ghost_lines = self.ghost_lines.fork_samples_task(samples);
        let point_size = self.point_size;
        let mesh_objects = self.mesh_objects.clone();
        let epoch = self.epoch;
        let cplane = self.cplane;
        let selection = self.selection;
        let selected_objects = self.selected_objects.clone();
        let selected_models = self.selected_models.clone();
        let highlight_objects = self.highlight_objects.clone();
        let merged = self.merged;
        let close = self.close;
        Box::new(move || Self {
            batch: batch(),
            highlight: highlight(),
            ghost: ghost.map(|task| task()),
            overlay: overlay(),
            moving_lines: moving(),
            ghost_lines: ghost_lines(),
            point_size,
            mesh_objects,
            epoch,
            cplane,
            selection,
            selected_objects,
            selected_models,
            highlight_objects,
            merged,
            close,
        })
    }
    /// Applies a drag preview without rebuilding any buffer.
    fn apply_preview(&mut self, delta: &om9_core::geometry::Transform) {
        let model = preview_model(delta);
        self.moving_lines.set_transform(model);
        for (&index, base) in self.selected_objects.iter().zip(&self.selected_models) {
            let composed = std::array::from_fn(|row| {
                std::array::from_fn(|col| (0..4).map(|k| model[row][k] * base[k][col]).sum())
            });
            self.batch.set_transform(index, composed);
        }
        for index in &self.highlight_objects {
            self.highlight.set_transform(*index, model);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::preview_model;
    use om9_core::geometry::Transform;

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
    use super::SELECTION_PINK;

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
