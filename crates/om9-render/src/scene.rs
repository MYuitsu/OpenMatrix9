//! Batched scene renderer for the viewport: one arena, one pipeline, many objects.
//!
//! The per-object renderer in `lib.rs` uploads a mesh and draws it once; that is the right shape for a
//! single object and the wrong shape for two thousand. This module keeps:
//!
//! * one growing **vertex/index arena** shared by every object, uploaded when the object is added;
//! * one **uniform block per visible object** (mvp + tint) written in a single `write_buffer` call;
//! * **frustum culling** on the CPU from the object's bounding sphere;
//! * a **front-to-back sort** once the object count passes 256, which reduces overdraw;
//! * one `set_bind_group` with a dynamic offset per object and `draw_indexed` over its arena range.
//!
//! The numbers this design is chosen for live in `tests/scene_bench.rs`.

use crate::{Camera, MeshData, RenderError};

const SORT_THRESHOLD: usize = 256;

/// One object in the batch.
#[derive(Debug, Clone)]
pub struct SceneObject {
    base_vertex: u32,
    index_start: u32,
    index_count: u32,
    model: [[f32; 4]; 4],
    tint: [f32; 4],
    centre: [f32; 3],
    radius: f32,
    vertex_count: usize,
    chunk: usize,
}

/// Statistics of the last frame, so the caller can show or assert them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameStats {
    pub objects: usize,
    pub visible: usize,
    /// Objects outside the frustum.
    pub culled: usize,
    /// Objects inside the frustum but smaller than the visible-pixel threshold.
    pub culled_small: usize,
    pub draw_calls: usize,
}

/// A merged range of static meshes: they share a chunk and are drawn with one draw call.
#[derive(Debug, Clone)]
struct StaticRange {
    chunk: usize,
    index_start: u32,
    index_count: u32,
    centre: [f32; 3],
    radius: f32,
}

/// One GPU buffer pair. wgpu caps a single buffer size, so a large scene is split into several chunks;
/// every object remembers which chunk owns its range.
#[derive(Clone)]
struct Chunk {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    vertex_bytes: u64,
    index_bytes: u64,
    vertex_capacity: u64,
    index_capacity: u64,
}

/// Chunk capacities. A scene used to open a new chunk at the device's maximum buffer size (about 2 GB),
/// so four viewports could reserve 8 GB of VRAM for a few hundred megabytes of geometry. Grow in classes
/// instead and only ever allocate the smallest class that holds the next mesh.
const CHUNK_CLASSES: [u64; 5] = [2 << 20, 8 << 20, 32 << 20, 128 << 20, 256 << 20];

fn chunk_capacity_for(needed: u64, device_max: u64) -> u64 {
    let needed = needed.max(2 << 20);
    CHUNK_CLASSES
        .iter()
        .copied()
        .find(|class| *class >= needed)
        .unwrap_or_else(|| needed.next_power_of_two().min(device_max.max(needed)))
        .max(needed)
        .min(device_max.max(needed))
}

fn new_chunk(device: &wgpu::Device, capacity: u64) -> Chunk {
    let vertices = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scene vertices"),
        size: capacity,
        usage: wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let indices = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scene indices"),
        size: capacity,
        usage: wgpu::BufferUsages::INDEX
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    Chunk {
        vertices,
        indices,
        vertex_bytes: 0,
        index_bytes: 0,
        vertex_capacity: capacity,
        index_capacity: capacity,
    }
}

/// The batch: arena, objects and the pipelines that draw them.
pub struct SceneBatch {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    overlay_pipeline: wgpu::RenderPipeline,
    bind_layout: wgpu::BindGroupLayout,
    chunks: Vec<Chunk>,
    max_buffer_size: u64,
    objects: std::sync::Arc<Vec<SceneObject>>,
    static_ranges: Vec<StaticRange>,
    /// Colour of every merged static range: the merge keeps one draw, so it keeps one tint too.
    static_tint: [f32; 4],
    static_meshes: usize,
    culling: bool,
    min_pixel_radius: f32,
    last_stats: FrameStats,
    /// Vertex/index bytes appended since the last flush. One write per chunk instead of two per object,
    /// which is what made a 42,000-object scene spend most of its open time in queue submissions.
    pending_vertices: Vec<PendingWrite>,
    pending_indices: Vec<PendingWrite>,
}

struct PendingWrite {
    chunk: usize,
    offset: u64,
    bytes: Vec<u8>,
    consumed: usize,
}

impl SceneBatch {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self::with_samples(device, queue, 1)
    }

    /// A batch whose pipelines match a multisampled colour/depth pair. `render_into_resolved` then names
    /// the single-sample texture the pass resolves into, so the viewport keeps an ordinary texture for
    /// its image while the grid and the mesh edges are antialiased.
    pub fn with_samples(device: wgpu::Device, queue: wgpu::Queue, samples: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene batch"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct Object { mvp: mat4x4<f32>, tint: vec4<f32> };
                @group(0) @binding(0) var<uniform> object: Object;

                struct VertexOut {
                    @builtin(position) clip: vec4<f32>,
                    @location(0) world: vec3<f32>,
                    @location(1) tint: vec4<f32>,
                };

                @vertex
                fn vs_main(@location(0) position: vec3<f32>,
                    @location(1) m0:vec4<f32>, @location(2) m1:vec4<f32>,
                    @location(3) m2:vec4<f32>, @location(4) m3:vec4<f32>,
                    @location(5) tint:vec4<f32>) -> VertexOut {
                    var out: VertexOut;
                    let world=mat4x4<f32>(m0,m1,m2,m3)*vec4<f32>(position,1.0);
                    out.clip=object.mvp*world;
                    out.world=world.xyz;
                    out.tint=tint;
                    return out;
                }

                @fragment
                fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
                    let normal = normalize(cross(dpdx(input.world), dpdy(input.world)));
                    let light = normalize(vec3<f32>(0.4, -0.7, 0.6));
                    let lambert = clamp(abs(dot(normal, light)), 0.0, 1.0);
                    return vec4<f32>(input.tint.rgb * (0.3 + 0.7 * lambert), input.tint.a);
                }
                "#
                .into(),
            ),
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene object"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(64 + 16),
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene batch"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let make_pipeline = |overlay: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scene batch"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }), Some(wgpu::VertexBufferLayout {
                        array_stride:80, step_mode:wgpu::VertexStepMode::Instance,
                        attributes:&wgpu::vertex_attr_array![1=>Float32x4,2=>Float32x4,3=>Float32x4,4=>Float32x4,5=>Float32x4],
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(!overlay),
                    depth_compare: Some(if overlay {
                        wgpu::CompareFunction::LessEqual
                    } else {
                        wgpu::CompareFunction::Less
                    }),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        // Alpha blending lets a selection highlight be drawn over the object it belongs to;
                        // opaque objects have alpha 1 and are unaffected.
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipeline = make_pipeline(false);
        let overlay_pipeline = make_pipeline(true);
        let max_buffer_size = device.limits().max_buffer_size.max(1 << 20);
        let chunks = vec![new_chunk(&device, chunk_capacity_for(0, max_buffer_size))];
        Self {
            device,
            queue,
            pipeline,
            overlay_pipeline,
            bind_layout,
            chunks,
            max_buffer_size,
            objects: std::sync::Arc::new(Vec::new()),
            static_ranges: Vec::new(),
            static_tint: [0.8, 0.6, 0.4, 1.0],
            static_meshes: 0,
            culling: true,
            min_pixel_radius: 0.5,
            last_stats: FrameStats::default(),
            pending_vertices: Vec::new(),
            pending_indices: Vec::new(),
        }
    }

    /// Appends a mesh whose placement is baked into its vertices, merging it into the chunk's static
    /// range. A thousand imported rings then cost a few draw calls instead of a thousand.
    pub fn add_static(&mut self, mesh: &MeshData, model: [[f32; 4]; 4]) -> usize {
        let (local_centre, local_radius) = bounds(mesh);
        let (centre, radius) = place_bounds(local_centre, local_radius, &model);
        let mut bytes = Vec::with_capacity(mesh.positions.len() * 12);
        for position in &mesh.positions {
            let x = model[0][0] * position[0]
                + model[1][0] * position[1]
                + model[2][0] * position[2]
                + model[3][0];
            let y = model[0][1] * position[0]
                + model[1][1] * position[1]
                + model[2][1] * position[2]
                + model[3][1];
            let z = model[0][2] * position[0]
                + model[1][2] * position[1]
                + model[2][2] * position[2]
                + model[3][2];
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
            bytes.extend_from_slice(&z.to_le_bytes());
        }
        let needed_v = bytes.len() as u64;
        let needed_i = (mesh.indices.len() * 4) as u64;
        let chunk_index = match self.chunks.iter().position(|chunk| {
            chunk.vertex_bytes + needed_v <= chunk.vertex_capacity
                && chunk.index_bytes + needed_i <= chunk.index_capacity
        }) {
            Some(index) => index,
            None => {
                self.chunks.push(new_chunk(
                    &self.device,
                    chunk_capacity_for(needed_v.max(needed_i), self.max_buffer_size),
                ));
                self.chunks.len() - 1
            }
        };
        let chunk = &mut self.chunks[chunk_index];
        let base_vertex = (chunk.vertex_bytes / 12) as u32;
        let vertex_offset = chunk.vertex_bytes;
        let index_offset = chunk.index_bytes;
        let index_bytes: Vec<u8> = mesh
            .indices
            .iter()
            .flat_map(|index| (index + base_vertex).to_le_bytes())
            .collect();
        chunk.vertex_bytes += needed_v;
        chunk.index_bytes += needed_i;
        self.pending_vertices.push(PendingWrite {
            chunk: chunk_index,
            offset: vertex_offset,
            bytes,
            consumed: 0,
        });
        self.pending_indices.push(PendingWrite {
            chunk: chunk_index,
            offset: index_offset,
            bytes: index_bytes,
            consumed: 0,
        });
        let index_start = (index_offset / 4) as u32;
        self.static_meshes += 1;
        if let Some(last) = self.static_ranges.last_mut()
            && last.chunk == chunk_index
            && last.index_start + last.index_count == index_start
        {
            last.index_count += mesh.indices.len() as u32;
            let (merged_centre, merged_radius) =
                union_sphere(last.centre, last.radius, centre, radius);
            last.centre = merged_centre;
            last.radius = merged_radius;
            return self.static_ranges.len() - 1;
        }
        self.static_ranges.push(StaticRange {
            chunk: chunk_index,
            index_start,
            index_count: mesh.indices.len() as u32,
            centre,
            radius,
        });
        self.static_ranges.len() - 1
    }

    /// Adds an occurrence of already uploaded geometry, without duplicating vertex/index bytes.
    pub fn add_instance(
        &mut self,
        source: usize,
        model: [[f32; 4]; 4],
        tint: [f32; 4],
    ) -> Option<usize> {
        let mut object = self.objects.get(source)?.clone();
        object.model = model;
        object.tint = tint;
        std::sync::Arc::make_mut(&mut self.objects).push(object);
        Some(self.objects.len() - 1)
    }

    /// Sets the tint used by merged static ranges, so a merged group can keep the colour of the objects it
    /// replaced.
    pub fn set_static_tint(&mut self, tint: [f32; 4]) {
        self.static_tint = tint;
    }

    /// Number of objects currently in the arena.
    #[allow(dead_code)]
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Turns frustum culling off, for throughput measurements and for scenes that must always be drawn.
    pub fn set_culling(&mut self, enabled: bool) {
        self.culling = enabled;
    }

    /// Objects whose projected radius is below this many pixels are skipped, because a sub-pixel object
    /// contributes nothing a person can see. `0.0` disables the test.
    pub fn set_min_pixel_radius(&mut self, pixels: f32) {
        self.min_pixel_radius = pixels.max(0.0);
    }

    pub fn stats(&self) -> FrameStats {
        self.last_stats
    }

    /// A different MSAA pipeline over the same resident geometry and occurrence records.
    /// Call only after pending uploads are drained; no mesh bytes are copied or uploaded.
    pub fn fork_samples(&self, samples: u32) -> Self {
        self.fork_samples_task(samples)()
    }
    pub fn fork_samples_task(&self, samples: u32) -> Box<dyn FnOnce() -> Self + Send> {
        assert_eq!(
            self.pending_upload_bytes(),
            0,
            "fork requires resident source"
        );
        let device = self.device.clone();
        let queue = self.queue.clone();
        let chunks = self.chunks.clone();
        let objects = self.objects.clone();
        let ranges = self.static_ranges.clone();
        let tint = self.static_tint;
        let meshes = self.static_meshes;
        let culling = self.culling;
        let pixels = self.min_pixel_radius;
        Box::new(move || {
            let mut fork = Self::with_samples(device, queue, samples);
            fork.chunks = chunks;
            fork.objects = objects;
            fork.static_ranges = ranges;
            fork.static_tint = tint;
            fork.static_meshes = meshes;
            fork.culling = culling;
            fork.min_pixel_radius = pixels;
            fork
        })
    }

    pub fn clear(&mut self) {
        std::sync::Arc::make_mut(&mut self.objects).clear();
        self.static_ranges.clear();
        self.static_meshes = 0;
        for chunk in &mut self.chunks {
            chunk.vertex_bytes = 0;
            chunk.index_bytes = 0;
        }
        self.pending_vertices.clear();
        self.pending_indices.clear();
    }

    /// Uploads a mesh into the arena and returns the object index.
    ///
    /// The arena is split into chunks because wgpu caps one buffer size; a mesh goes into the first chunk
    /// that still has room, and a new chunk is opened when none does.
    /// Adds local geometry with a row-major core-style model transform.
    pub fn add_mesh(&mut self, mesh: &MeshData, model: [[f32; 4]; 4], tint: [f32; 4]) -> usize {
        let (centre, radius) = bounds(mesh);
        let mut bytes = Vec::with_capacity(mesh.positions.len() * 12);
        for position in &mesh.positions {
            for value in position {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        let index_bytes: Vec<u8> = mesh
            .indices
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        let needed_v = bytes.len() as u64;
        let needed_i = index_bytes.len() as u64;
        let chunk_index = match self.chunks.iter().position(|chunk| {
            chunk.vertex_bytes + needed_v <= chunk.vertex_capacity
                && chunk.index_bytes + needed_i <= chunk.index_capacity
        }) {
            Some(index) => index,
            None => {
                let chunk = new_chunk(
                    &self.device,
                    chunk_capacity_for(needed_v.max(needed_i), self.max_buffer_size),
                );
                self.chunks.push(chunk);
                self.chunks.len() - 1
            }
        };
        let chunk = &mut self.chunks[chunk_index];
        let base_vertex = (chunk.vertex_bytes / 12) as u32;
        let index_start = (chunk.index_bytes / 4) as u32;
        let vertex_offset = chunk.vertex_bytes;
        let index_offset = chunk.index_bytes;
        chunk.vertex_bytes += needed_v;
        chunk.index_bytes += needed_i;
        self.pending_vertices.push(PendingWrite {
            chunk: chunk_index,
            offset: vertex_offset,
            bytes,
            consumed: 0,
        });
        self.pending_indices.push(PendingWrite {
            chunk: chunk_index,
            offset: index_offset,
            bytes: index_bytes,
            consumed: 0,
        });
        std::sync::Arc::make_mut(&mut self.objects).push(SceneObject {
            base_vertex,
            index_start,
            index_count: mesh.indices.len() as u32,
            model,
            tint,
            centre,
            radius,
            vertex_count: mesh.positions.len(),
            chunk: chunk_index,
        });
        self.objects.len() - 1
    }

    /// Rewrites a contiguous run of one object's vertices, which is what a sculpt stroke does: only the
    /// touched range is uploaded, the rest of the arena stays on the GPU.
    pub fn update_vertices(&mut self, object: usize, first_vertex: u32, positions: &[[f32; 3]]) {
        let Some(entry) = self.objects.get(object) else {
            return;
        };
        if first_vertex as usize + positions.len() > entry.vertex_count {
            return;
        }
        let mut bytes = Vec::with_capacity(positions.len() * 12);
        for position in positions {
            for value in position {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        let offset = entry.base_vertex as u64 * 12 + first_vertex as u64 * 12;
        let chunk = &self.chunks[entry.chunk];
        self.queue.write_buffer(&chunk.vertices, offset, &bytes);
    }

    pub fn set_transform(&mut self, object: usize, model: [[f32; 4]; 4]) {
        if self.objects.get(object).is_none_or(|o| o.model == model) {
            return;
        }
        if let Some(entry) = std::sync::Arc::make_mut(&mut self.objects).get_mut(object) {
            entry.model = model;
        }
    }

    pub fn pending_upload_bytes(&self) -> usize {
        self.pending_vertices
            .iter()
            .chain(self.pending_indices.iter())
            .map(|w| w.bytes.len() - w.consumed)
            .sum()
    }
    /// Service a shared window upload allowance; each write is at most 1MiB and aligned to four bytes.
    pub fn flush_uploads_budget(
        &mut self,
        max_bytes: usize,
        deadline: std::time::Instant,
    ) -> usize {
        let mut written = 0;
        for (vertices, writes) in [
            (true, &mut self.pending_vertices),
            (false, &mut self.pending_indices),
        ] {
            for w in writes.iter_mut() {
                while w.consumed < w.bytes.len()
                    && written + 4 <= max_bytes
                    && std::time::Instant::now() < deadline
                {
                    let n = (w.bytes.len() - w.consumed)
                        .min(max_bytes - written)
                        .min(1 << 20)
                        / 4
                        * 4;
                    if n == 0 {
                        break;
                    }
                    let buffer = if vertices {
                        &self.chunks[w.chunk].vertices
                    } else {
                        &self.chunks[w.chunk].indices
                    };
                    self.queue.write_buffer(
                        buffer,
                        w.offset + w.consumed as u64,
                        &w.bytes[w.consumed..w.consumed + n],
                    );
                    w.consumed += n;
                    written += n;
                }
            }
            writes.retain(|w| w.consumed < w.bytes.len());
        }
        written
    }

    /// Uploads every geometry byte appended since the last flush. Consecutive appends to one chunk are
    /// merged into a single `write_buffer`, so opening a 42,000-object scene is no longer 84,000 submits.
    pub fn flush_uploads(&mut self) {
        while self.pending_upload_bytes() > 0 {
            self.flush_uploads_budget(
                usize::MAX,
                std::time::Instant::now() + std::time::Duration::from_secs(3600),
            );
        }
    }

    /// Draws the batch into the given targets. `None` objects are culled.
    pub fn render_into(
        &mut self,
        colour_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.render_pass(colour_view, None, depth_view, camera, width, height, true)
    }

    /// Draws into a multisampled colour attachment and resolves it into `resolve_view` in the same pass.
    pub fn render_into_resolved(
        &mut self,
        colour_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.render_pass(
            colour_view,
            Some(resolve_view),
            depth_view,
            camera,
            width,
            height,
            true,
        )
    }

    /// Draws over what is already in the targets instead of clearing them.
    ///
    /// This is what a selection highlight needs: the scene stays as it is, and only the selected objects
    /// are drawn again on top, so changing the selection costs a few draws instead of a scene rebuild.
    pub fn render_over_into(
        &mut self,
        colour_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.render_pass(colour_view, None, depth_view, camera, width, height, false)
    }

    /// The multisampled form of [`Self::render_over_into`]: the existing samples stay, and the resolve
    /// target receives the frame after the highlight has been drawn.
    pub fn render_over_into_resolved(
        &mut self,
        colour_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.render_pass(
            colour_view,
            Some(resolve_view),
            depth_view,
            camera,
            width,
            height,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_pass(
        &mut self,
        colour_view: &wgpu::TextureView,
        resolve_view: Option<&wgpu::TextureView>,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
        clear: bool,
    ) -> Result<(), RenderError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scene wrapper"),
            });
        self.encode_pass(
            &mut encoder,
            colour_view,
            resolve_view,
            depth_view,
            camera,
            width,
            height,
            clear,
        )?;
        self.queue.submit([encoder.finish()]);
        Ok(())
    }

    /// Encodes into the window's encoder. Every pass owns immutable camera and instance buffers;
    /// encoding another view before submission cannot overwrite this view's uniforms.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        colour_view: &wgpu::TextureView,
        resolve_view: Option<&wgpu::TextureView>,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
        clear: bool,
    ) -> Result<(), RenderError> {
        self.flush_uploads();
        let view_projection = camera.view_projection(width as f32 / height as f32);
        let frustum = Frustum::from_view_projection(view_projection);
        let mut visible: Vec<usize> = Vec::with_capacity(self.objects.len());
        let mut culled = 0usize;
        let mut culled_small = 0usize;
        let tan_half = camera.tan_half_fov().max(1e-6);
        for (index, object) in self.objects.iter().enumerate() {
            if !self.culling {
                visible.push(index);
                continue;
            }
            let (centre, radius) = world_bounds(object);
            if !frustum.contains(centre, radius) {
                culled += 1;
                continue;
            }
            if self.min_pixel_radius > 0.0 {
                let distance = distance2(&camera.eye, centre).sqrt().max(camera.near);
                let radius_px = radius * (height as f32 * 0.5) / (distance * tan_half);
                if radius_px < self.min_pixel_radius {
                    culled_small += 1;
                    continue;
                }
            }
            visible.push(index);
        }
        // Static ranges are merged meshes: one draw per range, with the same frustum and pixel tests.
        let mut static_visible: Vec<usize> = Vec::with_capacity(self.static_ranges.len());
        for (index, range) in self.static_ranges.iter().enumerate() {
            if self.culling && !frustum.contains(range.centre, range.radius) {
                continue;
            }
            if self.culling && self.min_pixel_radius > 0.0 {
                let distance = distance2(&camera.eye, range.centre).sqrt().max(camera.near);
                let radius_px = range.radius * (height as f32 * 0.5) / (distance * tan_half);
                if radius_px < self.min_pixel_radius {
                    continue;
                }
            }
            static_visible.push(index);
        }
        if self.objects.len() > SORT_THRESHOLD {
            // Front to back: the depth buffer then rejects hidden fragments early.
            visible.sort_by(|left, right| {
                let a = self.objects[*left].centre;
                let b = self.objects[*right].centre;
                distance2(&camera.eye, a).total_cmp(&distance2(&camera.eye, b))
            });
        }
        use wgpu::util::DeviceExt;
        // Opaque occurrences group by shared geometry. Transparent overlays retain input order.
        if clear {
            visible.sort_by_key(|i| {
                let o = &self.objects[*i];
                (o.chunk, o.index_start, o.index_count, o.base_vertex)
            });
        }
        let mut camera_bytes = Vec::with_capacity(80);
        for value in view_projection {
            camera_bytes.extend_from_slice(&value.to_le_bytes());
        }
        camera_bytes.extend_from_slice(&[0u8; 16]);
        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("view camera snapshot"),
                contents: &camera_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let camera_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view camera snapshot"),
            layout: &self.bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(80),
                }),
            }],
        });
        let mut instance_bytes = Vec::with_capacity((visible.len() + 1) * 80);
        let append = |bytes: &mut Vec<u8>, model: [[f32; 4]; 4], tint: [f32; 4]| {
            for col in 0..4 {
                for row in &model {
                    bytes.extend_from_slice(&row[col].to_le_bytes());
                }
            }
            for value in tint {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        };
        append(
            &mut instance_bytes,
            [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
            self.static_tint,
        );
        for index in &visible {
            let o = &self.objects[*index];
            append(&mut instance_bytes, o.model, o.tint);
        }
        let instances = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("view occurrence snapshot"),
                contents: &instance_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });
        let mut draw_calls = 0;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: colour_view,
                    depth_slice: None,
                    resolve_target: resolve_view,
                    ops: wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.05,
                                g: 0.05,
                                b: 0.08,
                                a: 1.0,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: if clear {
                            wgpu::LoadOp::Clear(1.0)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(if clear {
                &self.pipeline
            } else {
                &self.overlay_pipeline
            });
            pass.set_bind_group(0, &camera_group, &[0]);
            pass.set_vertex_buffer(1, instances.slice(..));
            for (chunk_index, chunk) in self.chunks.iter().enumerate() {
                pass.set_vertex_buffer(0, chunk.vertices.slice(..));
                pass.set_index_buffer(chunk.indices.slice(..), wgpu::IndexFormat::Uint32);
                let mut slot = 0;
                while slot < visible.len() {
                    let object = &self.objects[visible[slot]];
                    let mut end = slot + 1;
                    while end < visible.len() {
                        let next = &self.objects[visible[end]];
                        if (
                            next.chunk,
                            next.index_start,
                            next.index_count,
                            next.base_vertex,
                        ) != (
                            object.chunk,
                            object.index_start,
                            object.index_count,
                            object.base_vertex,
                        ) {
                            break;
                        }
                        end += 1;
                    }
                    if object.chunk == chunk_index {
                        pass.draw_indexed(
                            object.index_start..object.index_start + object.index_count,
                            object.base_vertex as i32,
                            (slot as u32 + 1)..(end as u32 + 1),
                        );
                        draw_calls += 1;
                    }
                    slot = end;
                }
                for &index in &static_visible {
                    let range = &self.static_ranges[index];
                    if range.chunk == chunk_index {
                        pass.draw_indexed(
                            range.index_start..range.index_start + range.index_count,
                            0,
                            0..1,
                        );
                        draw_calls += 1;
                    }
                }
            }
        }

        self.last_stats = FrameStats {
            objects: self.objects.len() + self.static_meshes,
            visible: visible.len() + static_visible.len(),
            culled,
            culled_small,
            draw_calls,
        };
        Ok(())
    }
}

/// Six clipping planes as (normal, d): a point is inside when `dot(normal, p) + d >= -radius`.
struct Frustum {
    planes: [[f32; 4]; 6],
}

impl Frustum {
    fn from_view_projection(matrix: [f32; 16]) -> Self {
        // Rows of a column-major matrix.
        let row = |index: usize| {
            [
                matrix[index],
                matrix[4 + index],
                matrix[8 + index],
                matrix[12 + index],
            ]
        };
        let [r0, r1, r2, r3] = [row(0), row(1), row(2), row(3)];
        let combine = |a: [f32; 4], b: [f32; 4], sign: f32| {
            [
                a[0] + sign * b[0],
                a[1] + sign * b[1],
                a[2] + sign * b[2],
                a[3] + sign * b[3],
            ]
        };
        let normalise = |mut plane: [f32; 4]| {
            let length = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2])
                .sqrt()
                .max(1e-9);
            for value in &mut plane {
                *value /= length;
            }
            plane
        };
        Self {
            planes: [
                normalise(combine(r3, r0, 1.0)),
                normalise(combine(r3, r0, -1.0)),
                normalise(combine(r3, r1, 1.0)),
                normalise(combine(r3, r1, -1.0)),
                normalise(r2),
                normalise(combine(r3, r2, -1.0)),
            ],
        }
    }

    fn contains(&self, point: [f32; 3], radius: f32) -> bool {
        self.planes.iter().all(|plane| {
            plane[0] * point[0] + plane[1] * point[1] + plane[2] * point[2] + plane[3] >= -radius
        })
    }
}

#[allow(dead_code)]
fn multiply(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for column in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[column * 4 + k];
            }
            out[column * 4 + row] = sum;
        }
    }
    out
}

/// Frustum test for any caller that has the view-projection: true when the sphere may be visible.
pub fn frustum_contains(view_projection: [f32; 16], centre: [f32; 3], radius: f32) -> bool {
    Frustum::from_view_projection(view_projection).contains(centre, radius)
}

/// Places a local bounding sphere by a model matrix and scales its radius by the largest axis.
fn place_bounds(centre: [f32; 3], radius: f32, model: &[[f32; 4]; 4]) -> ([f32; 3], f32) {
    let [x, y, z] = centre;
    let placed = [
        model[0][0] * x + model[1][0] * y + model[2][0] * z + model[3][0],
        model[0][1] * x + model[1][1] * y + model[2][1] * z + model[3][1],
        model[0][2] * x + model[1][2] * y + model[2][2] * z + model[3][2],
    ];
    // sqrt(||A||1 * ||A||infinity) bounds the spectral norm, including affine shear.
    let columns = (0..3)
        .map(|col| (0..3).map(|row| model[col][row].abs()).sum::<f32>())
        .fold(0., f32::max);
    let rows = (0..3)
        .map(|row| (0..3).map(|col| model[col][row].abs()).sum::<f32>())
        .fold(0., f32::max);
    let frobenius = (0..3)
        .flat_map(|col| (0..3).map(move |row| model[col][row].powi(2)))
        .sum::<f32>()
        .sqrt();
    (placed, radius * (columns * rows).sqrt().min(frobenius))
}

/// Smallest sphere covering both inputs.
fn union_sphere(
    a_centre: [f32; 3],
    a_radius: f32,
    b_centre: [f32; 3],
    b_radius: f32,
) -> ([f32; 3], f32) {
    let mut centre = [0.0f32; 3];
    for axis in 0..3 {
        let low = (a_centre[axis] - a_radius).min(b_centre[axis] - b_radius);
        let high = (a_centre[axis] + a_radius).max(b_centre[axis] + b_radius);
        centre[axis] = (low + high) * 0.5;
    }
    let mut radius: f32 = 0.0;
    for (source, source_radius) in [(a_centre, a_radius), (b_centre, b_radius)] {
        radius = radius.max(distance2(&centre, source).sqrt() + source_radius);
    }
    (centre, radius)
}

/// Places the local bounding sphere in world space and scales the radius by the model's largest axis.
fn world_bounds(object: &SceneObject) -> ([f32; 3], f32) {
    // Dynamic models are row-major (as in core Transform); add_static's legacy API is column-major.
    let columns = std::array::from_fn(|col| std::array::from_fn(|row| object.model[row][col]));
    place_bounds(object.centre, object.radius, &columns)
}

fn distance2(from: &[f32; 3], to: [f32; 3]) -> f32 {
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let dz = to[2] - from[2];
    dx * dx + dy * dy + dz * dz
}

fn bounds(mesh: &MeshData) -> ([f32; 3], f32) {
    if mesh.positions.is_empty() {
        return ([0.0; 3], 0.0);
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for position in &mesh.positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    let centre = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let radius = mesh
        .positions
        .iter()
        .map(|position| distance2(&centre, *position).sqrt())
        .fold(0.0f32, f32::max);
    (centre, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn culling_keeps_an_object_that_is_in_front_of_the_camera() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            }))
            .ok()
        else {
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("cull unit"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: wgpu::Trace::Off,
        }))
        .expect("device");
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("colour"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let mut batch = SceneBatch::new(device, queue);
        let cube = MeshData::new(
            vec![
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1,
                6, 5, 0, 4, 7, 0, 7, 3,
            ],
        );
        let camera = Camera {
            eye: [0., -260., 210.],
            target: [0., 0., 0.],
            up: [0., 0., 1.],
            projection: crate::Projection::Perspective {
                vertical_fov_rad: 40f32.to_radians(),
            },
            near: 0.1,
            far: 2000.0,
        };
        // A 64x64 target puts these cubes under half a pixel, so the pixel test is disabled here: this
        // test is about the frustum alone.
        batch.set_min_pixel_radius(0.0);
        let corners = [
            (0.0, 0.0),
            (38.4, 32.0),
            (-38.4, -32.0),
            (38.4, -32.0),
            (-38.4, 32.0),
        ];
        for (x, y) in corners {
            batch.clear();
            batch.add_mesh(
                &cube,
                [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [x, y, 0., 1.],
                ],
                [1., 1., 1., 1.],
            );
            batch
                .render_into(&colour_view, &depth_view, camera, 64, 64)
                .expect("frame");
            let stats = batch.stats();
            assert_eq!(
                stats.visible, 1,
                "corner ({x}, {y}) should be visible, culled {}",
                stats.culled
            );
        }
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn culling_keeps_a_five_hundred_object_grid() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            }))
            .ok()
        else {
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("grid cull"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: wgpu::Trace::Off,
        }))
        .expect("device");
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("colour"),
            size: wgpu::Extent3d {
                width: 960,
                height: 540,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: 960,
                height: 540,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let mut batch = SceneBatch::new(device, queue);
        let cube = MeshData::new(
            vec![
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1,
                6, 5, 0, 4, 7, 0, 7, 3,
            ],
        );
        for index in 0..500usize {
            let x = (index % 25) as f32 - 12.0;
            let y = (index / 25) as f32 - 10.0;
            batch.add_mesh(
                &cube,
                [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [x * 3.2, y * 3.2, 0., 1.],
                ],
                [1., 1., 1., 1.],
            );
        }
        let camera = Camera {
            eye: [0., -260., 210.],
            target: [0., 0., 0.],
            up: [0., 0., 1.],
            projection: crate::Projection::Perspective {
                vertical_fov_rad: 40f32.to_radians(),
            },
            near: 0.1,
            far: 2000.0,
        };
        batch
            .render_into(&colour_view, &depth_view, camera, 960, 540)
            .expect("frame");
        let stats = batch.stats();
        assert_eq!(stats.objects, 500);
        assert_eq!(
            stats.culled, 0,
            "the whole grid is in front of this camera, culled {}",
            stats.culled
        );
    }
}
