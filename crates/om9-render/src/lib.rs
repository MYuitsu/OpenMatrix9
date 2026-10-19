//! GPU render loop for the OM9 viewport (M01-L2-01-06).
//!
//! A headless renderer: it owns the wgpu device, a colour+depth target, a mesh cache and three passes —
//! shaded faces, wireframe edges and a ground grid — plus a tint used by the selection highlight. It
//! returns pixels, so the loop itself is tested automatically; the look of the window stays a manual
//! check (all nine L3 of this module declare `requires_manual_test = true`).
//!
//! Shading is computed in the fragment shader from the world-space derivatives of the position, so the
//! renderer needs no normal attribute and still shades curved faces correctly.

use std::collections::{HashMap, HashSet};
pub mod cluster;
pub mod lod;
pub mod meshlet;
pub mod occlusion;
pub mod overlay;
pub mod pyramid;
pub mod qem;
pub mod scene;
pub mod shapes;
pub mod streaming;
pub mod tests_support;

use wgpu::util::DeviceExt;

/// A triangle mesh ready to upload. The kernel produces this shape; the renderer never sees OCCT types.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Self {
        Self { positions, indices }
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Unique triangle edges as line-list positions, for the wireframe pass.
    pub fn edge_lines(&self) -> Vec<[f32; 3]> {
        let mut seen: HashSet<(u32, u32)> = HashSet::new();
        let mut lines = Vec::new();
        for triangle in self.indices.chunks(3) {
            if triangle.len() < 3 {
                continue;
            }
            for (a, b) in [(triangle[0], triangle[1]), (triangle[1], triangle[2]), (triangle[2], triangle[0])] {
                let key = if a < b { (a, b) } else { (b, a) };
                if !seen.insert(key) {
                    continue;
                }
                let (Some(from), Some(to)) = (self.positions.get(a as usize), self.positions.get(b as usize)) else {
                    continue;
                };
                lines.push(*from);
                lines.push(*to);
            }
        }
        lines
    }
}

/// A camera in the same convention as the viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub projection: Projection,
    pub near: f32,
    pub far: f32,
}

/// How the camera turns view space into clip space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Perspective { vertical_fov_rad: f32 },
    /// Parallel projection; `vertical_span` is the world height the viewport covers at any depth.
    Orthographic { vertical_span: f32 },
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: [0.0, -2.0, 14.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective { vertical_fov_rad: 45f32.to_radians() },
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Camera {
    /// A perspective camera.
    pub fn perspective(eye: [f32; 3], target: [f32; 3], up: [f32; 3], vertical_fov_rad: f32) -> Self {
        Self { eye, target, up, projection: Projection::Perspective { vertical_fov_rad }, near: 0.1, far: 10_000.0 }
    }

    /// An orthographic camera: no vanishing point, `vertical_span` covers the viewport height.
    pub fn orthographic(eye: [f32; 3], target: [f32; 3], up: [f32; 3], vertical_span: f32) -> Self {
        Self { eye, target, up, projection: Projection::Orthographic { vertical_span }, near: 0.1, far: 20_000.0 }
    }

    /// `tan(fov/2)` for a perspective camera; 1 for an orthographic one, where the span is used instead.
    pub fn tan_half_fov(&self) -> f32 {
        match self.projection {
            Projection::Perspective { vertical_fov_rad } => (vertical_fov_rad * 0.5).tan(),
            Projection::Orthographic { .. } => 1.0,
        }
    }

    /// Half the world height the viewport covers.
    pub fn half_span(&self) -> f32 {
        match self.projection {
            Projection::Orthographic { vertical_span } => vertical_span * 0.5,
            Projection::Perspective { .. } => 0.0,
        }
    }

    pub fn is_orthographic(&self) -> bool {
        matches!(self.projection, Projection::Orthographic { .. })
    }

    pub fn view_projection(&self, aspect: f32) -> [f32; 16] {
        let forward = normalize(sub(self.target, self.eye));
        let right = normalize(cross(forward, self.up));
        let true_up = cross(right, forward);
        let view = [
            right[0], true_up[0], -forward[0], 0.0,
            right[1], true_up[1], -forward[1], 0.0,
            right[2], true_up[2], -forward[2], 0.0,
            -dot(right, self.eye), -dot(true_up, self.eye), dot(forward, self.eye), 1.0,
        ];
        let projection = match self.projection {
            Projection::Perspective { vertical_fov_rad } => {
                let f = 1.0 / (vertical_fov_rad * 0.5).tan();
                [
                    f / aspect, 0.0, 0.0, 0.0,
                    0.0, f, 0.0, 0.0,
                    0.0, 0.0, self.far / (self.near - self.far), -1.0,
                    0.0, 0.0, self.near * self.far / (self.near - self.far), 0.0,
                ]
            }
            Projection::Orthographic { vertical_span } => {
                let half_height = (vertical_span * 0.5).max(1e-6);
                [
                    1.0 / (half_height * aspect), 0.0, 0.0, 0.0,
                    0.0, 1.0 / half_height, 0.0, 0.0,
                    0.0, 0.0, 1.0 / (self.near - self.far), 0.0,
                    0.0, 0.0, self.near / (self.near - self.far), 1.0,
                ]
            }
        };
        multiply(projection, view)
    }
}

/// What to draw on top of the shaded faces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderStyle {
    /// Base colour of the shaded faces; the selection highlight uses a different tint.
    pub tint: [f32; 4],
    pub wireframe: bool,
    pub grid: bool,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self { tint: [0.82, 0.52, 0.2, 1.0], wireframe: false, grid: false }
    }
}

impl RenderStyle {
    pub fn highlighted() -> Self {
        Self { tint: [0.2, 0.9, 0.35, 1.0], ..Self::default() }
    }
}

/// A rendered frame: RGBA8 pixels, row-major, tightly packed.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Frame {
    pub fn painted_pixels(&self) -> usize {
        self.pixels
            .chunks(4)
            .filter(|pixel| pixel[0] > 60 || pixel[1] > 60 || pixel[2] > 60)
            .count()
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * self.width + x) * 4) as usize;
        [self.pixels[index], self.pixels[index + 1], self.pixels[index + 2], self.pixels[index + 3]]
    }

    /// Counts pixels whose green channel leads red, which is what the green highlight produces.
    pub fn green_pixels(&self) -> usize {
        self.pixels
            .chunks(4)
            .filter(|pixel| pixel[1] > 120 && pixel[1] > pixel[0] + 30)
            .count()
    }
}

/// Why a frame could not be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    Lost,
    NoAdapter,
}

struct GpuMesh {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    index_count: u32,
    wire: wgpu::Buffer,
    wire_vertices: u32,
}

/// Owner of the wgpu device, the offscreen target and the mesh cache.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    faces: wgpu::RenderPipeline,
    lines: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    cache: HashMap<u64, GpuMesh>,
    uploads: usize,
    lost: bool,
}

const UNIFORM_SIZE: u64 = 64 + 16;

impl Renderer {
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("om9-render"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        Some(Self::build(device, queue))
    }

    /// Builds the renderer on a caller-owned device, for example eframe's wgpu render state.
    pub fn from_device(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self::build(device, queue)
    }

    fn build(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });

        let face_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shaded"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct Uniforms { mvp: mat4x4<f32>, tint: vec4<f32> };
                @group(0) @binding(0) var<uniform> u: Uniforms;

                struct VertexOut {
                    @builtin(position) clip: vec4<f32>,
                    @location(0) world: vec3<f32>,
                };

                @vertex
                fn vs_main(@location(0) position: vec3<f32>) -> VertexOut {
                    var out: VertexOut;
                    out.clip = u.mvp * vec4<f32>(position, 1.0);
                    out.world = position;
                    return out;
                }

                @fragment
                fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
                    // Face normal from the screen-space derivatives of the world position: no normal
                    // attribute is uploaded, and the shading follows the geometry's own triangles.
                    let normal = normalize(cross(dpdx(input.world), dpdy(input.world)));
                    let light = normalize(vec3<f32>(0.4, -0.7, 0.6));
                    let lambert = clamp(abs(dot(normal, light)), 0.0, 1.0);
                    let shade = 0.25 + 0.75 * lambert;
                    return vec4<f32>(u.tint.rgb * shade, 1.0);
                }
                "#
                .into(),
            ),
        });
        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lines"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct Uniforms { mvp: mat4x4<f32>, tint: vec4<f32> };
                @group(0) @binding(0) var<uniform> u: Uniforms;

                @vertex
                fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
                    return u.mvp * vec4<f32>(position, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return u.tint;
                }
                "#
                .into(),
            ),
        });

        let faces = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shaded"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &face_shader,
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
                })],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &face_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let lines = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lines"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
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
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &line_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            device,
            queue,
            faces,
            lines,
            bind_group,
            uniform,
            cache: HashMap::new(),
            uploads: 0,
            lost: false,
        }
    }

    /// Draws the mesh with the default style.
    pub fn render(&mut self, key: u64, mesh: &MeshData, camera: Camera, width: u32, height: u32) -> Result<Frame, RenderError> {
        self.render_with(key, mesh, camera, width, height, RenderStyle::default())
    }

    /// Uploads and caches the mesh under `key`; a no-op when the key is already cached.
    fn ensure_mesh(&mut self, key: u64, mesh: &MeshData) {
        if self.cache.contains_key(&key) {
            return;
        }
        let vertex_data: Vec<u8> = mesh.positions.iter().flat_map(|p| p.iter().flat_map(|v| v.to_le_bytes())).collect();
        let index_data: Vec<u8> = mesh.indices.iter().flat_map(|v| v.to_le_bytes()).collect();
        let edges = mesh.edge_lines();
        let wire_data: Vec<u8> = edges.iter().flat_map(|p| p.iter().flat_map(|v| v.to_le_bytes())).collect();
        let placeholder = [0u8; 12];
        let vertex = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh vertices"),
            contents: if vertex_data.is_empty() { &placeholder } else { &vertex_data },
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh indices"),
            contents: if index_data.is_empty() { &placeholder } else { &index_data },
            usage: wgpu::BufferUsages::INDEX,
        });
        let wire = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh wire"),
            contents: if wire_data.is_empty() { &placeholder } else { &wire_data },
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.cache.insert(
            key,
            GpuMesh { vertex, index, index_count: mesh.indices.len() as u32, wire, wire_vertices: edges.len() as u32 },
        );
        self.uploads += 1;
    }

    /// Writes the uniforms, draws the cached mesh into the given views and submits.
    #[allow(clippy::too_many_arguments)]
    fn draw_pass(
        &mut self,
        colour_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        key: u64,
        camera: Camera,
        width: u32,
        height: u32,
        style: RenderStyle,
    ) {
        let matrix = camera.view_projection(width as f32 / height as f32);
        let mut uniform_bytes: Vec<u8> = matrix.iter().flat_map(|v| v.to_le_bytes()).collect();
        uniform_bytes.extend(style.tint.iter().flat_map(|v| v.to_le_bytes()));
        self.queue.write_buffer(&self.uniform, 0, &uniform_bytes);
        let grid = if style.grid { grid_lines() } else { Vec::new() };
        let grid_bytes: Vec<u8> = grid.iter().flat_map(|p| p.iter().flat_map(|v| v.to_le_bytes())).collect();
        let grid_vertices = grid.len() as u32;
        let grid_buffer = if grid.is_empty() {
            None
        } else {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("grid"),
                contents: &grid_bytes,
                usage: wgpu::BufferUsages::VERTEX,
            }))
        };
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shaded"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: colour_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let cached = self.cache.get(&key).expect("cached mesh");
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_pipeline(&self.faces);
            pass.set_vertex_buffer(0, cached.vertex.slice(..));
            pass.set_index_buffer(cached.index.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..cached.index_count, 0, 0..1);
            if style.wireframe && cached.wire_vertices > 0 {
                pass.set_pipeline(&self.lines);
                pass.set_vertex_buffer(0, cached.wire.slice(..));
                pass.draw(0..cached.wire_vertices, 0..1);
            }
            if let Some(grid) = &grid_buffer {
                pass.set_pipeline(&self.lines);
                pass.set_vertex_buffer(0, grid.slice(..));
                pass.draw(0..grid_vertices, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Draws into caller-owned targets, so the viewport can show the result without a CPU readback.
    #[allow(clippy::too_many_arguments)]
    pub fn render_into(
        &mut self,
        colour_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        key: u64,
        mesh: &MeshData,
        camera: Camera,
        width: u32,
        height: u32,
        style: RenderStyle,
    ) -> Result<(), RenderError> {
        assert!(width > 0 && height > 0, "the target must have a size");
        if self.lost {
            return Err(RenderError::Lost);
        }
        self.ensure_mesh(key, mesh);
        self.draw_pass(colour_view, depth_view, key, camera, width, height, style);
        Ok(())
    }

    /// Uploads the mesh on first use, caches the face and wire buffers under `key`, then draws.
    #[allow(clippy::too_many_arguments)]
    pub fn render_with(
        &mut self,
        key: u64,
        mesh: &MeshData,
        camera: Camera,
        width: u32,
        height: u32,
        style: RenderStyle,
    ) -> Result<Frame, RenderError> {
        assert!(width > 0 && height > 0, "the target must have a size");
        if self.lost {
            return Err(RenderError::Lost);
        }
        self.ensure_mesh(key, mesh);
        let colour = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("colour"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        // A texture copy requires each row to start on a 256-byte boundary.
        let row_bytes = width * 4;
        let padded_row = row_bytes.div_ceil(256) * 256;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.draw_pass(&colour_view, &depth_view, key, camera, width, height, style);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("readback") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &colour,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("map");
        let data = slice.get_mapped_range().expect("mapped range");
        // Strip the row padding so callers always get tightly packed RGBA8 rows.
        let mut pixels = Vec::with_capacity((row_bytes * height) as usize);
        for row in 0..height {
            let start = (row * padded_row) as usize;
            pixels.extend_from_slice(&data[start..start + row_bytes as usize]);
        }
        drop(data);
        readback.unmap();
        Ok(Frame { width, height, pixels })
    }

    pub fn uploads(&self) -> usize {
        self.uploads
    }

    pub fn invalidate(&mut self, key: u64) {
        self.cache.remove(&key);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn simulate_device_loss(&mut self) {
        self.device.destroy();
        self.cache.clear();
        self.lost = true;
    }

    pub fn recover(&mut self) -> bool {
        match Self::new() {
            Some(fresh) => {
                *self = fresh;
                true
            }
            None => false,
        }
    }
}

/// Ground grid on the z=0 plane, as a line list.
fn grid_lines() -> Vec<[f32; 3]> {
    let mut lines = Vec::new();
    for step in -10..=10 {
        let value = step as f32;
        lines.push([value, -10.0, 0.0]);
        lines.push([value, 10.0, 0.0]);
        lines.push([-10.0, value, 0.0]);
        lines.push([10.0, value, 0.0]);
    }
    lines
}

/// Redraw bookkeeping: the viewport only redraws when something it depends on changed.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderScheduler {
    camera_revision: u64,
    scene_revision: u64,
    dirty: bool,
    drawn: bool,
}

impl RenderScheduler {
    pub fn should_redraw(&mut self, camera_revision: u64, scene_revision: u64) -> bool {
        let changed = self.dirty
            || !self.drawn
            || camera_revision != self.camera_revision
            || scene_revision != self.scene_revision;
        if changed {
            self.camera_revision = camera_revision;
            self.scene_revision = scene_revision;
            self.dirty = false;
        }
        changed
    }

    pub fn mark_drawn(&mut self) {
        self.drawn = true;
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let length = dot(a, a).sqrt().max(f32::MIN_POSITIVE);
    [a[0] / length, a[1] / length, a[2] / length]
}

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
