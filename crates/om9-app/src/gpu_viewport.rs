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
    /// The document scene drawn on the GPU: meshes, curves, points and the grid.
    scene: Option<GpuScene>,
}

/// Everything the document viewport needs on the GPU. It is rebuilt when the scene or the CPlane changes,
/// never per frame, so painting costs one draw pass.
/// Above this many mesh objects the unselected ones are merged into static ranges: one draw instead of
/// one per object, at the cost of one colour for the merged group.
const MERGE_THRESHOLD: usize = 300;

/// Samples per pixel of the document viewport. Four is the multisample count WebGPU guarantees for a
/// renderable format, and it is what keeps the one-pixel grid from turning into a staircase: a diagonal
/// segment now covers part of a pixel and resolves to a grey level instead of a hard step.
const SAMPLES: u32 = 4;

struct GpuScene {
    batch: om9_render::scene::SceneBatch,
    /// Only the selected objects, drawn over the scene with alpha blending. Selection changes rebuild this
    /// small batch instead of the whole scene.
    highlight: om9_render::scene::SceneBatch,
    overlay: om9_render::overlay::OverlayBatch,
    moving_lines: om9_render::overlay::OverlayBatch,
    ghost_lines: om9_render::overlay::OverlayBatch,
    point_size: f64,
    mesh_objects: Vec<(om9_core::identity::ObjectId, usize)>,
    epoch: u64,
    cplane: u64,
    selection: u64,
    /// Batch object indices of the selected meshes and of the highlight copies. A drag preview only rewrites
    /// their model matrix, which is a handful of uniform writes instead of a scene rebuild.
    selected_objects: Vec<usize>,
    highlight_objects: Vec<usize>,
    /// Grey copy of the objects at the position a drag started from; `None` when no drag is running.
    ghost: Option<om9_render::scene::SceneBatch>,
    /// True when the batch merged unselected meshes into static ranges, so a selection change has to rebuild
    /// the batch (the selected mesh is no longer an object of its own).
    merged: bool,
}


impl GpuViewport {
    /// Builds the renderer on eframe's device. With `OM9_GPU_STRESS=N` the viewport instead shows a
    /// batch of N boxes through the shared-arena scene renderer, which is the architecture chosen for
    /// 1000-2000 objects (`implementation/design/M01/VIEWPORT-PERFORMANCE.md`).
    pub fn new(state: &RenderState) -> Self {
        let stress = std::env::var("OM9_GPU_STRESS").ok().and_then(|value| value.trim().parse::<usize>().ok()).unwrap_or(0);
        // `OM9_GPU_STRESS_SHAPE=sphere` draws 1024-triangle spheres instead of the 12-triangle cubes, so
        // the demo can answer "how many spheres" rather than "how many boxes".
        let unit = if std::env::var("OM9_GPU_STRESS_SHAPE").is_ok_and(|value| value.eq_ignore_ascii_case("sphere")) {
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
                eprintln!("gpu: stress {stress}, side {side}, clusters {}", clusters.len());
            }
            cluster = om9_render::cluster::ClusterRenderer::new(state.device.clone(), state.queue.clone(), &clusters);
            if std::env::var("OM9_GPU_DEBUG").is_ok() {
                eprintln!("gpu: cluster renderer built: {}", cluster.is_some());
            }
            if let Some(renderer) = cluster.as_mut() {
                // The manual checklist can turn occlusion on; it takes one frame to build the pyramid.
                if std::env::var("OM9_GPU_OCCLUSION").is_ok() {
                    renderer.set_occlusion(true);
                }
                if let Ok(megabytes) = std::env::var("OM9_GPU_STREAM_MB")
                    && let Ok(value) = megabytes.trim().parse::<u64>()
                {
                    renderer.set_budget_bytes(value * 1024 * 1024);
                }
            }
        }
        if stress > 0 && cluster.is_none() {
            let mut scene = om9_render::scene::SceneBatch::new(state.device.clone(), state.queue.clone());
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
            last_epoch: None,
            last_cplane: None,
            last_selection: None,
            last_preview: None,
            batch,
            cluster,
            scene: None,
        }
    }

    /// Samples of the document viewport's targets; the stress paths keep one sample and their own depth.
    fn sample_count(&self) -> u32 {
        if self.batch.is_none() && self.cluster.is_none() { SAMPLES } else { 1 }
    }

    /// Recreates the targets when the rect changed, and registers the new view with egui.
    fn ensure_targets(&mut self, state: &RenderState, width: u32, height: u32) {
        if self.size == (width, height) && self.colour_view.is_some() {
            return;
        }
        if let Some(id) = self.texture_id.take() {
            state.renderer.write().free_texture(&id);
        }
        let samples = self.sample_count();
        let colour = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("om9 viewport colour"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
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
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let depth = state.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("om9 viewport depth msaa"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
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
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }))
        };
        self.depth_view = depth.as_ref().map(|depth| depth.create_view(&wgpu::TextureViewDescriptor::default()));
        self.colour_view = Some(colour_view);
        self.colour = Some(colour);
        self.depth = depth;
        self.msaa_colour = msaa_colour;
        self.msaa_colour_view = msaa_colour_view;
        self.msaa_depth = msaa_depth;
        self.msaa_depth_view = msaa_depth_view;
        self.size = (width, height);
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

    /// Rebuilds the GPU copy of the document scene when the scene epoch or the CPlane revision changed.
    #[allow(clippy::too_many_arguments)]
    fn refresh_scene(
        &mut self,
        state: &RenderState,
        camera: &om9_scene::camera::Camera,
        scene: Option<&om9_scene::scene::Scene>,
        cplane: om9_scene::camera::CPlane,
        viewport: om9_scene::camera::Viewport,
        selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
        selection_revision: u64,
        samples: u32,
    ) {
        let epoch = scene.map(|scene| scene.epoch).unwrap_or(0);
        let cplane_revision = cplane.revision;
        if let Some(current) = &self.scene
            && current.epoch == epoch
            && current.cplane == cplane_revision
        {
            if current.selection == selection_revision
                && current.point_size == world_per_pixel(camera, viewport) * 5.0 { return; }
            if !current.merged {
                if let Some(current) = self.scene.as_mut() {
                    current.apply_preview(&om9_core::geometry::Transform::identity());
                    current.highlight = build_highlight(&state.device, &state.queue, scene, selected, samples);
                    current.highlight_objects = (0..current.highlight.object_count()).collect();
                    current.selected_objects = current.mesh_objects.iter().filter(|(id,_)|selected.contains(id)).map(|(_,index)|*index).collect();
                    let point_size=world_per_pixel(camera,viewport)*5.0;
                    let (overlay,moving,ghost)=build_lines(state,scene,cplane,selected,point_size,samples);
                    current.overlay=overlay;
                    current.moving_lines=moving;
                    current.ghost_lines=ghost;
                    current.point_size=point_size;
                    current.ghost=None;
                    current.selection = selection_revision;
                }
                return;
            }
            // Merged scenes hold the selected mesh inside a static range; only a rebuild takes it out.
        }
        let mut batch = om9_render::scene::SceneBatch::with_samples(state.device.clone(), state.queue.clone(), samples);
        let mut selected_indices: Vec<usize> = Vec::new();
        let mut mesh_objects=Vec::new();
        let mesh_count = scene
            .map(|scene| {
                scene
                    .objects
                    .iter()
                    .filter(|object| {
                        matches!(object.buffers.as_ref(), om9_scene::scene::RenderBuffers::Mesh { .. })
                    })
                    .count()
            })
            .unwrap_or(0);
        let merged = mesh_count > MERGE_THRESHOLD;
        let merge = merged;
        if let Some(scene) = scene
            && let Some(object) = scene.objects.iter().find(|object| {
                matches!(object.buffers.as_ref(), om9_scene::scene::RenderBuffers::Mesh { .. })
                    && !selected.contains(&object.root)
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
        if let Some(scene) = scene {
            for object in &scene.objects {
                if !object.visible {
                    continue;
                }
                let transform = object.transform;
                let colour = [
                object.color[0] as f32 / 255.0,
                object.color[1] as f32 / 255.0,
                object.color[2] as f32 / 255.0,
                1.0,
            ];

                if let om9_scene::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() {
                        let mut positions = Vec::with_capacity(vertices.len());
                        for vertex in vertices {
                            let Ok(point) = transform.point(*vertex) else {
                                continue;
                            };
                            positions.push([point[0] as f32, point[1] as f32, point[2] as f32]);
                        }
                        if positions.is_empty() || triangles.is_empty() {
                            continue;
                        }
                        let mut indices = Vec::with_capacity(triangles.len() * 3);
                        for (corners, _) in triangles {
                            indices.extend_from_slice(corners);
                        }
                        let mesh = om9_render::MeshData::new(positions, indices);
                        if merge && !selected.contains(&object.root) {
                            // Merged ranges share one draw and the tint set above; the selected objects stay
                            // separate so their highlight keeps its own colour.
                            batch.add_static(&mesh, identity_model());
                        } else {
                            let index = batch.add_mesh(&mesh, identity_model(), colour);
                            mesh_objects.push((object.root,index));
                            if selected.contains(&object.root) {
                                selected_indices.push(index);
                            }
                        }

                }
            }
        }
        // Keep the selected line geometry separate so preview transforms never rebuild its buffer.
        let (overlay,moving_lines,ghost_lines)=build_lines(state,scene,cplane,selected,point_size,samples);
        let highlight = build_highlight(&state.device, &state.queue, scene, selected, samples);
        let highlight_objects = (0..highlight.object_count()).collect();
        self.scene = Some(GpuScene {
            batch,
            highlight,
            overlay,
            moving_lines,
            ghost_lines,
            point_size,
            mesh_objects,
            epoch,
            cplane: cplane_revision,
            selection: selection_revision,
            selected_objects: selected_indices,
            highlight_objects,
            ghost: None,
            merged,
        });
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
    ) -> bool {
        let pixels_per_point = ui.ctx().pixels_per_point();
        let width = (rect.width() * pixels_per_point).round().max(1.0) as u32;
        let height = (rect.height() * pixels_per_point).round().max(1.0) as u32;
        self.ensure_targets(state, width, height);
        let (Some(colour_view), Some(texture_id)) = (self.colour_view.clone(), self.texture_id) else {
            return false;
        };
        // The camera revision is the identity of the rendered frame: while it and the target size are
        // unchanged, redrawing would produce the same pixels, so the cached texture is reused.
        let epoch = scene.map(|scene| scene.epoch).unwrap_or(0);
        let unchanged = self.last_revision == Some(revision)
            && self.size == (width, height)
            && self.last_epoch == Some(epoch)
            && self.last_cplane == Some(cplane.revision)
            && self.last_selection == Some(selection_revision)
            && self.last_preview == Some(preview_revision);
        let document = self.batch.is_none() && self.cluster.is_none();
        if document {
            // Every document pass draws into the multisampled pair and resolves into the texture egui
            // paints, so the grid lines and the mesh silhouettes are antialiased.
            let (Some(render_colour), Some(depth_view)) = (
                self.msaa_colour_view.clone(),
                self.msaa_depth_view.clone(),
            ) else {
                return false;
            };
            let viewport = om9_scene::camera::Viewport {
                origin: [rect.min.x as f64, rect.min.y as f64],
                size: [rect.width() as f64, rect.height() as f64],
            };
            self.refresh_scene(
                state,
                document_camera,
                scene,
                cplane,
                viewport,
                selected,
                selection_revision,
                SAMPLES,
            );
            if let Some(gpu) = self.scene.as_mut()
                && !unchanged
            {
                // A drag preview is a model matrix on the moving objects: no buffer is rebuilt, so the frame
                // cost stays flat while the pointer moves.
                let delta = scene
                    .and_then(|scene| {
                        scene
                            .objects
                            .iter()
                            .find(|object| selected.contains(&object.root))
                            .map(|object| object.preview)
                    })
                    .unwrap_or_else(om9_core::geometry::Transform::identity);
                gpu.apply_preview(&delta);
                let dragging = delta.0 != om9_core::geometry::Transform::identity().0;
                if dragging && gpu.ghost.is_none() {
                    gpu.ghost = Some(build_ghost(&state.device, &state.queue, scene, selected, SAMPLES));
                } else if !dragging {
                    gpu.ghost = None;
                }
                if gpu
                    .batch
                    .render_into_resolved(&render_colour, &colour_view, &depth_view, camera, width, height)
                    .is_err()
                {
                    return false;
                }
                if let Some(ghost) = gpu.ghost.as_mut() {
                    // Over the scene, depth-tested: the ghost shows where the object is not covering it.
                    ghost
                        .render_over_into_resolved(&render_colour, &colour_view, &depth_view, camera, width, height)
                        .ok();
                }
                // The highlight is small, so it is drawn every frame over the scene rather than merged.
                gpu.highlight
                    .render_over_into_resolved(&render_colour, &colour_view, &depth_view, camera, width, height)
                    .ok();
                gpu.overlay.render_resolved(&render_colour, &colour_view, &depth_view, camera, width, height);
                if dragging {
                    gpu.ghost_lines.render_resolved(&render_colour, &colour_view, &depth_view, camera, width, height);
                }
                gpu.moving_lines.render_resolved(&render_colour, &colour_view, &depth_view, camera, width, height);
                self.last_revision = Some(revision);
                self.last_epoch = Some(epoch);
                self.last_cplane = Some(cplane.revision);
                self.last_selection = Some(selection_revision);
                self.last_preview = Some(preview_revision);
            }
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture_id, rect, uv, egui::Color32::WHITE);
            return true;
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
            && batch.render_into(&colour_view, &depth_view, camera, width, height).is_err()
        {
            return false;
        }
        self.last_revision = Some(revision);
        self.last_epoch = Some(epoch);
        self.last_cplane = Some(cplane.revision);
        self.last_selection = Some(selection_revision);
        self.last_preview = Some(preview_revision);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(texture_id, rect, uv, egui::Color32::WHITE);
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
            projection: om9_render::Projection::Perspective { vertical_fov_rad: vertical_fov_rad as f32 },
            near: 0.01,
            far: 10_000.0,
        },
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => {
            // A real parallel projection: no vanishing point, so a top view shows squares, not depth.
            let span = (vertical_span_mm as f32).max(1e-3);
            let eye = camera.eye.map(|value| value as f32);
            let distance = ((eye[0] - target[0]).powi(2) + (eye[1] - target[1]).powi(2) + (eye[2] - target[2]).powi(2))
                .sqrt()
                .max(span);
            Camera {
                eye,
                target,
                up,
                projection: om9_render::Projection::Orthographic { vertical_span: span },
                near: (distance - span * 4.0).max(0.01),
                far: distance + span * 4.0,
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
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5, 0, 4, 7,
        0, 7, 3,
    ];
    MeshData::new(positions, indices)
}

/// World units per screen pixel for the document camera.
fn world_per_pixel(camera: &om9_scene::camera::Camera, viewport: om9_scene::camera::Viewport) -> f64 {
    let height = viewport.size[1].max(1.0);
    match camera.projection {
        om9_scene::camera::Projection::Perspective { vertical_fov_rad } => {
            let offset = [
                camera.eye[0] - camera.target[0],
                camera.eye[1] - camera.target[1],
                camera.eye[2] - camera.target[2],
            ];
            let distance = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
            2.0 * (vertical_fov_rad * 0.5).tan() * distance / height
        }
        om9_scene::camera::Projection::Orthographic { vertical_span_mm } => vertical_span_mm / height,
    }
}

/// One object's triangles in world space, or `None` when it has no mesh or no transform.
fn build_lines(
    state:&RenderState,
    scene:Option<&om9_scene::scene::Scene>,
    cplane:om9_scene::camera::CPlane,
    selected:&std::collections::BTreeSet<om9_core::identity::ObjectId>,
    point_size:f64,
    samples:u32,
) -> (om9_render::overlay::OverlayBatch,om9_render::overlay::OverlayBatch,om9_render::overlay::OverlayBatch) {
    use om9_scene::scene::RenderBuffers;
    let make=||om9_render::overlay::OverlayBatch::with_samples(state.device.clone(),state.queue.clone(),samples);
    let (mut fixed,mut moving,mut ghost)=(make(),make(),make());
    // Grid first, so coplanar curves are visible over it.
    if let Ok(grid)=om9_scene::grid::Grid::new(cplane) {
        for line in grid.lines {fixed.line(line.start,line.end,line.color.map(|x|x as f32/255.));}
    }
    if let Some(scene)=scene {
        for object in &scene.objects {
            if !object.visible {continue;}
            let picked=selected.contains(&object.root);
            let tint=if picked {[1.,0.86,0.,1.]} else {object.color.map(|x|x as f32/255.)};
            let out=if picked {&mut moving} else {&mut fixed};
            let draw=|batch:&mut om9_render::overlay::OverlayBatch,color| {
                match object.buffers.as_ref() {
                    RenderBuffers::Curve {points,..}=> {
                        for pair in points.windows(2) {
                            if let (Ok(a),Ok(b))=(object.transform.point(pair[0]),object.transform.point(pair[1])) {batch.line(a,b,color);}
                        }
                    }
                    RenderBuffers::Points {points,..}=> {
                        for p in points {if let Ok(p)=object.transform.point(*p) {batch.point(p,point_size,color);}}
                    }
                    _=>{}
                }
            };
            draw(out,tint);
            if picked {draw(&mut ghost,[0.5,0.5,0.5,1.]);}
        }
    }
    fixed.upload();moving.upload();ghost.upload();
    (fixed,moving,ghost)
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
    let om9_scene::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else {
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
    let mut ghost = om9_render::scene::SceneBatch::with_samples(device.clone(), queue.clone(), samples);
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
    scene: Option<&om9_scene::scene::Scene>,
    selected: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    samples: u32,
) -> om9_render::scene::SceneBatch {
    let mut highlight = om9_render::scene::SceneBatch::with_samples(device.clone(), queue.clone(), samples);
    let Some(scene) = scene else {
        return highlight;
    };
    for object in &scene.objects {
        if !selected.contains(&object.root) {
            continue;
        }
        if let Some(mesh) = world_mesh_at(object, object.transform) {
            highlight.add_mesh(&mesh, identity_model(), [0.25, 0.80, 0.50, 0.55]);
        }
    }
    highlight
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
    /// Applies a drag preview without rebuilding any buffer.
    fn apply_preview(&mut self, delta: &om9_core::geometry::Transform) {
        let model = preview_model(delta);
        self.moving_lines.set_transform(model);
        for index in &self.selected_objects {
            self.batch.set_transform(*index, model);
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
        assert!((ghost.positions[0][0] - 1.5).abs() < 1e-5, "ghost at the placement: {:?}", ghost.positions[0]);
        assert!((live.positions[0][0] - 6.5).abs() < 1e-5, "live mesh at the preview: {:?}", live.positions[0]);
    }
}
