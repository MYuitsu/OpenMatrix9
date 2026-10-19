//! GPU lines and point markers: the grid, curves and points of a viewport, drawn with depth testing so
//! the scene occludes them exactly like the meshes.
//!
//! The CPU painter used to draw these with egui, which cannot depth-test against the GPU meshes, so the
//! grid showed through solid objects. This pass shares the depth buffer with [`crate::scene::SceneBatch`]
//! and therefore sits behind the same geometry.
//!
//! A point is drawn as three short world-space crosses, sized from the camera so it stays about the same
//! number of pixels at any zoom; a 1-pixel point primitive would disappear on a high-DPI screen.

use crate::Camera;

/// One vertex of the line list.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 3],
    colour: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_projection: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

/// A line list that is rebuilt whenever the scene or the view changes.
pub struct OverlayBatch {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    batch: LineBatch,
    buffer: Option<wgpu::Buffer>,
    capacity: usize,
    model: [[f32; 4]; 4],
}

impl OverlayBatch {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self::with_samples(device, queue, 1)
    }

    /// A line pass whose pipeline matches a multisampled colour/depth pair: the grid is one pixel wide, so
    /// multisampling is what turns its diagonal edges into partial coverage instead of a staircase.
    pub fn with_samples(device: wgpu::Device, queue: wgpu::Queue, samples: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay lines"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct Uniforms { view_projection: mat4x4<f32>, model: mat4x4<f32> };
                @group(0) @binding(0) var<uniform> uniforms: Uniforms;

                struct VertexIn {
                    @location(0) position: vec3<f32>,
                    @location(1) colour: vec4<f32>,
                };
                struct VertexOut {
                    @builtin(position) clip: vec4<f32>,
                    @location(0) colour: vec4<f32>,
                };

                @vertex
                fn vs_main(input: VertexIn) -> VertexOut {
                    var out: VertexOut;
                    out.clip = uniforms.view_projection * uniforms.model * vec4<f32>(input.position, 1.0);
                    out.colour = input.colour;
                    return out;
                }

                @fragment
                fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
                    return input.colour;
                }
                "#
                .into(),
            ),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<LineVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 12, shader_location: 1 },
                    ],
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay uniform"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
        });
        Self {
            device,
            queue,
            pipeline,
            bind_group,
            uniform,
            batch: LineBatch::default(),
            buffer: None,
            capacity: 0,
            model: [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]],
        }
    }

    /// The pure geometry of the frame.
    pub fn batch(&mut self) -> &mut LineBatch {
        &mut self.batch
    }

    /// Drops every line, before rebuilding for the current frame.
    pub fn clear(&mut self) {
        self.batch.clear();
    }

    pub fn line(&mut self, a: [f64; 3], b: [f64; 3], colour: [f32; 4]) {
        self.batch.line(a, b, colour);
    }

    pub fn point(&mut self, point: [f64; 3], size: f64, colour: [f32; 4]) {
        self.batch.point(point, size, colour);
    }

    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.batch.line_count()
    }
    /// An affine drag updates uniforms while retaining the uploaded line geometry.
    pub fn set_transform(&mut self, row_major: [[f32;4];4]) {
        self.model=std::array::from_fn(|column| std::array::from_fn(|row| row_major[row][column]));
    }

    /// Uploads the current lines. Reuses the buffer until it has to grow.
    pub fn upload(&mut self) {
        if self.batch.vertices.is_empty() {
            return;
        }
        let bytes: Vec<u8> = bytemuck::cast_slice(&self.batch.vertices).to_vec();
        let needed = self.batch.vertices.len();
        if self.buffer.is_none() || self.capacity < needed {
            let capacity = needed.next_power_of_two();
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("overlay vertices"),
                size: (capacity * std::mem::size_of::<LineVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&buffer, 0, &bytes);
            self.buffer = Some(buffer);
            self.capacity = capacity;
        } else if let Some(buffer) = &self.buffer {
            self.queue.write_buffer(buffer, 0, &bytes);
        }
    }

    /// Draws the lines into an existing pass that already holds the meshes' depth.
    pub fn render(
        &mut self,
        colour_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) {
        self.render_pass(colour_view, None, depth_view, camera, width, height);
    }

    /// Draws into a multisampled colour attachment and resolves it into `resolve_view`.
    #[allow(clippy::too_many_arguments)]
    pub fn render_resolved(
        &mut self,
        colour_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) {
        self.render_pass(colour_view, Some(resolve_view), depth_view, camera, width, height);
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
    ) {
        let Some(buffer) = self.buffer.as_ref() else {
            return;
        };
        let uniforms = Uniforms { view_projection: matrix_to_columns(camera.view_projection(width as f32 / height.max(1) as f32)), model:self.model };
        self.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniforms));
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("overlay") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: colour_view,
                    depth_slice: None,
                    resolve_target: resolve_view,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..(self.batch.vertices.len() as u32), 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }


}

fn matrix_to_columns(matrix: [f32; 16]) -> [[f32; 4]; 4] {
    [
        [matrix[0], matrix[1], matrix[2], matrix[3]],
        [matrix[4], matrix[5], matrix[6], matrix[7]],
        [matrix[8], matrix[9], matrix[10], matrix[11]],
        [matrix[12], matrix[13], matrix[14], matrix[15]],
    ]
}

/// The lines of one frame, without any GPU object: this is what the tests exercise.
#[derive(Default)]
pub struct LineBatch {
    vertices: Vec<LineVertex>,
}

impl LineBatch {
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn line(&mut self, a: [f64; 3], b: [f64; 3], colour: [f32; 4]) {
        self.vertices.push(LineVertex { position: [a[0] as f32, a[1] as f32, a[2] as f32], colour });
        self.vertices.push(LineVertex { position: [b[0] as f32, b[1] as f32, b[2] as f32], colour });
    }

    /// A point marker: three short crosses around `point`.
    pub fn point(&mut self, point: [f64; 3], size: f64, colour: [f32; 4]) {
        for axis in 0..3 {
            let mut a = point;
            let mut b = point;
            a[axis] -= size;
            b[axis] += size;
            self.line(a, b, colour);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.vertices.len() / 2
    }

    #[cfg(test)]
    fn positions(&self) -> Vec<[f32; 3]> {
        self.vertices.iter().map(|vertex| vertex.position).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_is_three_crosses_centred_on_it() {
        let mut batch = LineBatch::default();
        batch.point([1.0, 2.0, 3.0], 0.5, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(batch.line_count(), 3);
        let positions = batch.positions();
        assert_eq!(positions[0], [0.5, 2.0, 3.0]);
        assert_eq!(positions[1], [1.5, 2.0, 3.0]);
        // The second axis row starts at vertex 2: y -/+ around the point.
        assert_eq!(positions[2], [1.0, 1.5, 3.0]);
        assert_eq!(positions[3], [1.0, 2.5, 3.0]);
    }

    #[test]
    fn clearing_drops_every_line() {
        let mut batch = LineBatch::default();
        batch.line([0.0; 3], [1.0; 3], [1.0; 4]);
        assert_eq!(batch.line_count(), 1);
        batch.clear();
        assert!(batch.is_empty());
    }
}
