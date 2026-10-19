//! GPU depth pyramid built with render passes, the Hi-Z structure cluster culling reads.
//!
//! The compute attempt in `cluster.rs` did not land level 0, so this module takes the route that has
//! already been proven in this repository: draw the pyramid. The depth buffer is sampled into a layered
//! `R32Float` texture, one layer per halving, and each layer is rendered with its own viewport. Culling
//! then reads a layer with `textureLoad`, whose layer index may be a runtime value.

use wgpu::util::DeviceExt;

/// The render-pass pyramid: one array layer per level, level 0 at full resolution.
pub struct DepthPyramidGpu {
    layout: wgpu::BindGroupLayout,
    from_depth: wgpu::RenderPipeline,
    downsample: wgpu::RenderPipeline,
    dims: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    array_view: Option<wgpu::TextureView>,
    /// A one-texel stand-in for the layer a level-0 pass does not read.
    dummy: wgpu::Texture,
    levels: Vec<(u32, u32)>,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Dims {
    /// x: source layer, y: destination size x, z: destination size y, w: unused
    values: [u32; 4],
}

impl DepthPyramidGpu {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pyramid layers"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pyramid"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pyramid"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                @group(0) @binding(0) var depth_texture: texture_depth_2d;
                @group(0) @binding(1) var previous: texture_2d_array<f32>;
                @group(0) @binding(2) var<uniform> dims: vec4<u32>;

                struct VertexOut {
                    @builtin(position) clip: vec4<f32>,
                };

                @vertex
                fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
                    // A full-screen triangle; the viewport limits it to the level's size.
                    var positions = array<vec2<f32>, 3>(
                        vec2<f32>(-1.0, -3.0),
                        vec2<f32>(-1.0, 1.0),
                        vec2<f32>(3.0, 1.0),
                    );
                    var out: VertexOut;
                    out.clip = vec4<f32>(positions[index], 0.0, 1.0);
                    return out;
                }

                @fragment
                fn from_depth(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
                    let coordinate = vec2<i32>(position.xy);
                    return vec4<f32>(textureLoad(depth_texture, coordinate, 0), 0.0, 0.0, 0.0);
                }

                @fragment
                fn downsample(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
                    let source = vec2<i32>(position.xy) * 2;
                    var maximum = 0.0;
                    for (var dy = 0; dy < 2; dy = dy + 1) {
                        for (var dx = 0; dx < 2; dx = dx + 1) {
                            maximum = max(maximum, textureLoad(previous, source + vec2<i32>(dx, dy), 0, 0).x);
                        }
                    }
                    return vec4<f32>(maximum, 0.0, 0.0, 0.0);
                }
                "#
                .into(),
            ),
        });
        let make_pipeline = |entry: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("pyramid"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let from_depth = make_pipeline("from_depth");
        let downsample = make_pipeline("downsample");
        let dims = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("pyramid dims"),
            contents: bytemuck::bytes_of(&Dims::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pyramid dummy"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Self { layout, from_depth, downsample, dims, texture: None, array_view: None, dummy, levels: Vec::new() }
    }

    /// Creates the layered texture for a target size; a no-op when the size is unchanged.
    pub fn ensure(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let mut levels = Vec::new();
        let (mut level_width, mut level_height) = (width.max(1), height.max(1));
        levels.push((level_width, level_height));
        while level_width > 1 || level_height > 1 {
            level_width = level_width.div_ceil(2);
            level_height = level_height.div_ceil(2);
            levels.push((level_width, level_height));
            if levels.len() >= 16 {
                break;
            }
        }
        if self.texture.is_some() && self.levels == levels {
            return;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pyramid layers"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: levels.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.array_view = Some(texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("pyramid array"),
            format: Some(wgpu::TextureFormat::R32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(levels.len() as u32),
        }));
        self.texture = Some(texture);
        self.levels = levels;
    }

    pub fn array_view(&self) -> Option<&wgpu::TextureView> {
        self.array_view.as_ref()
    }

    pub fn levels(&self) -> usize {
        self.levels.len()
    }

    /// Copies one layer back for diagnostics: the maximum and the first few values.
    pub fn debug_layer(&self, device: &wgpu::Device, queue: &wgpu::Queue, level: usize) -> Option<(f32, Vec<f32>)> {
        let texture = self.texture.as_ref()?;
        let (width, height) = *self.levels.get(level)?;
        let padded = (width * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pyramid readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("pyramid read") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: level as u32 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("poll");
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("map");
        let data = slice.get_mapped_range().expect("mapped");
        let mut values = Vec::with_capacity(width as usize * height as usize);
        for row in 0..height {
            let start = (row * padded) as usize;
            let row_values: &[f32] = bytemuck::cast_slice(&data[start..start + (width * 4) as usize]);
            values.extend_from_slice(row_values);
        }
        drop(data);
        readback.unmap();
        let maximum = values.iter().cloned().fold(0.0f32, f32::max);
        Some((maximum, values))
    }

    /// Renders every level from the depth buffer; level 0 from depth, the rest from the level below.
    pub fn build(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        depth_view: &wgpu::TextureView,
    ) {
        let Some(texture) = self.texture.as_ref() else {
            return;
        };
        if self.levels.len() < 2 {
            return;
        }
        for (level, (width, height)) in self.levels.iter().copied().enumerate() {
            let layer_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("pyramid layer"),
                format: Some(wgpu::TextureFormat::R32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: level as u32,
                array_layer_count: Some(1),
            });
            queue.write_buffer(
                &self.dims,
                0,
                bytemuck::bytes_of(&Dims { values: [(level.saturating_sub(1)) as u32, width, height, 0] }),
            );
            // Reading the whole array while writing one of its layers aliases; bind the source layer
            // alone, and a dummy texel for level 0 which reads no previous layer.
            let previous_view = if level == 0 {
                self.dummy.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("pyramid dummy layer"),
                    format: Some(wgpu::TextureFormat::R32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: Some(1),
                })
            } else {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("pyramid source layer"),
                    format: Some(wgpu::TextureFormat::R32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: (level - 1) as u32,
                    array_layer_count: Some(1),
                })
            };
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("pyramid"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(depth_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&previous_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: self.dims.as_entire_binding() },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pyramid"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &layer_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(0.0, 0.0, width as f32, height as f32, 0.0, 1.0);
            pass.set_pipeline(if level == 0 { &self.from_depth } else { &self.downsample });
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
