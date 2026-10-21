//! GPU cluster culling: the renderer decides per cluster on the GPU, not on the CPU.
//!
//! Nanite's first two stages in one module: a **storage buffer of cluster records**, a **compute pass** that
//! tests each cluster's bounding sphere against the frustum and its projected size, and a single
//! `multi_draw_indexed_indirect` whose argument list the compute pass wrote. Culled clusters get an index
//! count of zero, which the GPU skips without a second dispatch or a counter readback.
//!
//! Requires `Features::MULTI_DRAW_INDIRECT`; [`ClusterRenderer::available`] reports whether the adapter has
//! it before a caller builds one.

use crate::Camera;
use crate::meshlet::Meshlet;
use wgpu::util::DeviceExt;

/// One GPU buffer pair. wgpu caps a single buffer, so cluster geometry lives in several chunks and each
/// chunk is drawn with its own multi-draw over a contiguous range of the indirect list.
struct ClusterChunk {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    vertex_bytes: u64,
    index_bytes: u64,
}

/// The per-cluster record the compute pass reads: a range plus a bounding sphere.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterRecord {
    pub index_start: u32,
    pub index_count: u32,
    pub base_vertex: i32,
    /// 0 = full detail, 1 = the group's coarse level.
    pub level: u32,
    pub centre_radius: [f32; 4],
    /// Which group this cluster belongs to; the compute pass uses it to pick the level.
    pub group: u32,
    /// Padding so the struct is 48 bytes, matching the WGSL layout (align 16 due to `vec4`).
    pub _pad: [u32; 3],
}

/// `DrawIndexedIndirectArgs`, the layout the GPU expects.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewUniform {
    /// Six frustum planes as (nx, ny, nz, d), each normalised.
    planes: [[f32; 4]; 6],
    /// view_projection, so the vertex stage needs no other data.
    matrix: [[f32; 4]; 4],
    /// x: eye xyz, w: tan(fov/2)
    eye_tan: [f32; 4],
    /// x: half height in pixels, y: minimum projected radius, z: near plane, w: coarse pixel threshold
    params: [f32; 4],
    /// x: width, y: height, z: pyramid levels, w: unused
    screen: [f32; 4],
    /// One (offset, width, height, 0) per pyramid level, in storage-buffer elements.
    levels: [[f32; 4]; 16],
    /// x: 1 for orthographic, y: half vertical span, z: far, w: near
    projection: [f32; 4],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClusterStats {
    pub clusters: usize,
    pub visible: usize,
}

/// Owns the cluster buffers, the compute pipeline and the indirect draw pass.
pub struct ClusterRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compute: wgpu::ComputePipeline,
    render: wgpu::RenderPipeline,
    indirect: wgpu::Buffer,
    view: wgpu::Buffer,
    chunks: Vec<ClusterChunk>,
    /// (first cluster, cluster count) per chunk, in indirect-list order.
    chunk_ranges: Vec<(u32, u32)>,
    /// Hash of the camera the last frame used, to notice a moving view.
    last_view: Option<u64>,
    /// Which chunks were uploaded; only those are drawn and culled.
    chunk_resident: Vec<bool>,
    parse: wgpu::Buffer,
    /// Kept alive because the compute bind group references it.
    #[allow(dead_code)]
    group_radius: wgpu::Buffer,
    /// World-space error of each group's coarse level, used to choose the level.
    #[allow(dead_code)]
    group_error: wgpu::Buffer,
    coarse_pixels: f32,
    level_count: u32,
    levels: Vec<u32>,
    cluster_count: u32,
    last_stats: ClusterStats,
    /// Cluster records; kept as a field because the bind group is rebuilt when targets change.
    records: wgpu::Buffer,
    compute_layout: wgpu::BindGroupLayout,
    compute_bind: Option<wgpu::BindGroup>,
    render_bind: wgpu::BindGroup,
    /// Depth the render pass writes; the pyramid reads it.
    depth: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
    /// Render-pass depth pyramid; rebuilt after each frame for the next one.
    pyramid: crate::pyramid::DepthPyramidGpu,
    /// (offset, width, height) per level, plus the total element count.
    pyramid_table: Vec<(u32, u32, u32)>,
    size: (u32, u32),
    /// True once a pyramid exists, so the first frame culls on frustum only.
    occluding: bool,
    /// True when per-group level selection has error data to work with.
    level_select: bool,
    /// One flag per cluster: 1 = resident in VRAM, 0 = evicted by the budget.
    residency: wgpu::Buffer,
    /// Bytes each cluster costs while resident.
    cluster_bytes: Vec<u64>,
    /// How many clusters the budget keeps resident.
    resident_count: usize,
    /// Flipped by `set_occlusion(true)`; the pyramid build must be trusted first.
    pyramid_verified: bool,
}

impl ClusterRenderer {
    /// Clusters the cull pass must examine: the resident prefix of the list.
    fn cull_count(&self) -> u32 {
        self.chunk_ranges
            .iter()
            .zip(self.chunk_resident.iter())
            .filter(|(_, resident)| **resident)
            .map(|((start, count), _)| start + count)
            .max()
            .unwrap_or(0)
    }

    /// `multi_draw_indexed_indirect` is core in wgpu 30, so an adapter that can render can do this.
    pub fn available(_adapter: &wgpu::Adapter) -> bool {
        true
    }

    /// Uploads clusters with an explicit level and group per entry, and the group radii the compute pass
    /// uses to pick a level.
    #[allow(clippy::too_many_arguments)]
    fn build(
        device: wgpu::Device,
        queue: wgpu::Queue,
        entries: &[(Meshlet, u32, u32)],
        radii: &[f32],
        errors: &[f32],
        coarse_pixels: f32,
        level_count: u32,
        budget_bytes: Option<u64>,
    ) -> Option<Self> {
        let mut used_bytes = 0u64;

        let limit = device.limits().max_buffer_size;
        let mut cluster_bytes: Vec<u64> = Vec::with_capacity(entries.len());
        let mut records = Vec::with_capacity(entries.len());
        let mut args = Vec::with_capacity(entries.len());
        let mut chunk_ranges: Vec<(u32, u32)> = Vec::new();
        // Geometry is packed into chunks that each stay under the wgpu buffer limit.
        let mut chunk_positions: Vec<Vec<[f32; 3]>> = Vec::new();
        let mut chunk_indices: Vec<Vec<u32>> = Vec::new();
        let mut chunk_vertex_bytes: Vec<u64> = Vec::new();
        let mut chunk_index_bytes: Vec<u64> = Vec::new();
        let mut cluster_chunk: Vec<usize> = Vec::with_capacity(entries.len());
        for (cluster, level, group) in entries {
            let vertex_bytes = (cluster.positions.len() as u64) * 12;
            let index_bytes = (cluster.indices.len() as u64) * 4;
            let chunk_limit = budget_bytes
                .map(|budget| (budget / 8).max(64 * 1024).min(limit))
                .unwrap_or(limit);
            let fits = chunk_positions.last().is_some_and(|_| {
                let chunk = chunk_positions.len() - 1;
                chunk_vertex_bytes[chunk] + vertex_bytes <= chunk_limit
                    && chunk_index_bytes[chunk] + index_bytes <= chunk_limit
            });
            if !fits {
                chunk_positions.push(Vec::new());
                chunk_indices.push(Vec::new());
                chunk_vertex_bytes.push(0);
                chunk_index_bytes.push(0);
                chunk_ranges.push((records.len() as u32, 0));
            }
            let chunk = chunk_positions.len() - 1;
            let base_vertex = (chunk_vertex_bytes[chunk] / 12) as u32;
            let index_start = (chunk_index_bytes[chunk] / 4) as u32;
            chunk_positions[chunk].extend(cluster.positions.iter().copied());
            chunk_indices[chunk].extend(cluster.indices.iter().copied());
            chunk_vertex_bytes[chunk] += vertex_bytes;
            chunk_index_bytes[chunk] += index_bytes;
            if let Some(range) = chunk_ranges.last_mut() {
                range.1 += 1;
            }
            cluster_bytes.push(vertex_bytes + index_bytes);
            cluster_chunk.push(chunk);
            records.push(ClusterRecord {
                index_start,
                index_count: cluster.indices.len() as u32,
                base_vertex: base_vertex as i32,
                level: *level,
                centre_radius: [
                    cluster.centre[0],
                    cluster.centre[1],
                    cluster.centre[2],
                    cluster.radius,
                ],
                group: *group,
                _pad: [0; 3],
            });
            args.push(DrawArgs {
                index_count: cluster.indices.len() as u32,
                instance_count: 1,
                first_index: index_start,
                base_vertex: base_vertex as i32,
                first_instance: 0,
            });
        }
        if records.is_empty() {
            return None;
        }
        let mut chunks = Vec::with_capacity(chunk_positions.len());
        let mut chunk_resident: Vec<bool> = Vec::with_capacity(chunk_positions.len());
        for (positions, indices) in chunk_positions.iter().zip(chunk_indices.iter()) {
            let bytes = (positions.len() as u64) * 12 + (indices.len() as u64) * 4;
            let fits = budget_bytes.is_none_or(|budget| used_bytes + bytes <= budget);
            if fits {
                used_bytes += bytes;
            }
            chunk_resident.push(fits);
            let (vertex_data, index_data) = if fits {
                (
                    positions
                        .iter()
                        .flat_map(|p| p.iter().flat_map(|v| v.to_le_bytes()))
                        .collect::<Vec<u8>>(),
                    indices
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect::<Vec<u8>>(),
                )
            } else {
                // Never uploaded: a placeholder keeps the pipeline valid while the budget is spent
                // elsewhere.
                (vec![0u8; 12], vec![0u8; 4])
            };
            chunks.push(ClusterChunk {
                vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("cluster chunk vertices"),
                    contents: &vertex_data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
                indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("cluster chunk indices"),
                    contents: &index_data,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                }),
                vertex_bytes: if fits {
                    (positions.len() as u64) * 12
                } else {
                    0
                },
                index_bytes: if fits { (indices.len() as u64) * 4 } else { 0 },
            });
        }
        let initial_resident: Vec<u32> = cluster_chunk
            .iter()
            .map(|chunk| u32::from(chunk_resident[*chunk]))
            .collect();
        let initial_count = initial_resident.iter().filter(|flag| **flag == 1).count();
        // A cluster whose chunk was never uploaded must not leave a stale draw argument behind.
        for (index, chunk) in cluster_chunk.iter().enumerate() {
            if !chunk_resident[*chunk] {
                args[index].index_count = 0;
                args[index].instance_count = 1;
            }
        }
        let records = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cluster records"),
            contents: bytemuck::cast_slice(&records),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let indirect = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cluster indirect"),
            contents: bytemuck::cast_slice(&args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
        });
        let parse = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cluster indirect readback"),
            size: (args.len() * std::mem::size_of::<DrawArgs>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let residency = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cluster residency"),
            contents: bytemuck::cast_slice(&initial_resident),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let group_radius = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cluster group radius"),
            contents: bytemuck::cast_slice(radii),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let expected = radii.len() * level_count.max(1) as usize;
        let error_data: Vec<f32> = if errors.len() == expected {
            errors.to_vec()
        } else {
            vec![0.0; expected]
        };
        let group_error = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cluster group error"),
            contents: bytemuck::cast_slice(&error_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let view = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cluster view"),
            size: std::mem::size_of::<ViewUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cluster compute"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let render_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cluster render"),
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
        let render_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cluster render"),
            layout: &render_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view.as_entire_binding(),
            }],
        });
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cluster cull"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct Cluster {
                    index_start: u32,
                    index_count: u32,
                    base_vertex: i32,
                    level: u32,
                    centre_radius: vec4<f32>,
                    group: u32,
                    pad0: u32,
                    pad1: u32,
                };
                struct Args {
                    index_count: u32,
                    instance_count: u32,
                    first_index: u32,
                    base_vertex: i32,
                    first_instance: u32,
                };
                struct View {
                    planes: array<vec4<f32>, 6>,
                    matrix: mat4x4<f32>,
                    eye_tan: vec4<f32>,
                    params: vec4<f32>,
                    screen: vec4<f32>,
                    levels: array<vec4<f32>, 16>,
                    projection: vec4<f32>,
                };
                @group(0) @binding(0) var<storage, read> clusters: array<Cluster>;
                @group(0) @binding(1) var<storage, read_write> args: array<Args>;
                @group(0) @binding(2) var<uniform> view: View;
                @group(0) @binding(3) var<storage, read> group_radius: array<f32>;
                @group(0) @binding(4) var<storage, read> group_error: array<f32>;
                @group(0) @binding(5) var pyramid: texture_2d_array<f32>;
                @group(0) @binding(6) var<storage, read> residency: array<u32>;

                @compute @workgroup_size(64)
                fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                    let index = id.x;
                    if (index >= arrayLength(&clusters)) {
                        return;
                    }
                    // A cluster evicted by the streaming budget is skipped before any other work.
                    if (residency[index] == 0u) {
                        args[index].index_count = 0u;
                        args[index].instance_count = 1u;
                        return;
                    }
                    let cluster = clusters[index];
                    let centre = cluster.centre_radius.xyz;
                    let radius = cluster.centre_radius.w;
                    var visible = true;
                    for (var plane = 0; plane < 6; plane = plane + 1) {
                        let p = view.planes[plane];
                        if (dot(p.xyz, centre) + p.w < -radius) {
                            visible = false;
                        }
                    }
                    // Projected radius: the same rule the CPU uses, so both agree.
                    let eye = view.eye_tan.xyz;
                    let distance = max(length(centre - eye), view.params.z);
                    let ortho = view.projection.x > 0.5;
                    let radius_pixels = select(
                        radius * view.params.x / (distance * view.eye_tan.w),
                        radius * view.params.x / view.projection.y,
                        ortho,
                    );
                    if (radius_pixels < view.params.y) {
                        visible = false;
                    }
                    // Level choice by projected error: the coarse level is used when its geometric error
                    // stays under one pixel on screen. `coarse_pixels` is that threshold.
                    let group_distance = max(length(centre - eye), view.params.z);
                    if (view.screen.w > 0.5) {
                        let level_count = u32(view.screen.w);
                        var wanted_level = 0u;
                        let group_base = cluster.group * level_count;
                        for (var level = 1u; level < level_count; level = level + 1u) {
                            let error_pixels = group_error[group_base + level] * view.params.x / (group_distance * view.eye_tan.w);
                            if (error_pixels < view.params.w) {
                                wanted_level = level;
                            } else {
                                break;
                            }
                        }
                        if (cluster.level != wanted_level) {
                            visible = false;
                        }
                    }
                    // Occlusion: compare the cluster's nearest depth with the farthest depth of the
                    // pyramid cell covering its screen rectangle. Conservative, so never culls a visible
                    // cluster.
                    if (visible && view.screen.z > 0.5) {
                        let clip = view.matrix * vec4<f32>(centre, 1.0);
                        if (clip.w > 0.0) {
                            let ndc = clip.xyz / clip.w;
                            let pixels_x = (ndc.x * 0.5 + 0.5) * view.screen.x;
                            let pixels_y = (0.5 - ndc.y * 0.5) * view.screen.y;
                            let extent = radius_pixels * 2.0;
                            var level = u32(clamp(log2(max(extent, 1.0)), 0.0, view.screen.z - 1.0));
                            let info = view.levels[level];
                            let level_width = info.y;
                            let level_height = info.z;
                            let scale = view.screen.x / level_width;
                            // Conservatively cover the cluster's screen rectangle: the farthest depth of
                            // every cell it touches. A single centre sample culls a cluster whose centre
                            // happens to sit behind a neighbour, which leaves holes at silhouettes.
                            let half = max(radius_pixels, 1.0);
                            let cell_x0 = floor((pixels_x - half) / scale);
                            let cell_x1 = floor((pixels_x + half) / scale);
                            let cell_y0 = floor((pixels_y - half) / scale);
                            let cell_y1 = floor((pixels_y + half) / scale);
                            var max_depth = 0.0;
                            for (var row = u32(clamp(cell_y0, 0.0, level_height - 1.0));
                                row <= u32(clamp(cell_y1, 0.0, level_height - 1.0));
                                row = row + 1u) {
                                for (var column = u32(clamp(cell_x0, 0.0, level_width - 1.0));
                                    column <= u32(clamp(cell_x1, 0.0, level_width - 1.0));
                                    column = column + 1u) {
                                    max_depth = max(
                                        max_depth,
                                        textureLoad(pyramid, vec2<i32>(i32(column), i32(row)), i32(level), 0).x,
                                    );
                                }
                            }
                            // Depth is not linear: near the far plane a whole millimetre is less than one
                            // f32 step, so compare real view distances instead of ndc values. The margin is
                            // one tenth of a percent plus a micrometre, which no visible occlusion is.
                            let far = view.projection.z;
                            let near = view.projection.w;
                            let denominator = far - max_depth * (far - near);
                            let covered_distance = select(
                                far * near / max(denominator, 1e-9),
                                1.0e9,
                                denominator <= 1e-9,
                            );
                            let nearest_distance = group_distance - radius;
                            let margin = max(1e-3, covered_distance * 1e-3);
                            if (max_depth > 0.0 && max_depth < 0.9999 && nearest_distance > covered_distance + margin) {
                                visible = false;
                            }
                        }
                    }
                    if (visible) {
                        args[index].index_count = cluster.index_count;
                        args[index].instance_count = 1;
                        args[index].first_index = cluster.index_start;
                        args[index].base_vertex = cluster.base_vertex;
                        args[index].first_instance = 0;
                    } else {
                        args[index].index_count = 0;
                        args[index].instance_count = 1;
                    }
                }
                "#
                .into(),
            ),
        });
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cluster draw"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct View {
                    planes: array<vec4<f32>, 6>,
                    matrix: mat4x4<f32>,
                    eye_tan: vec4<f32>,
                    params: vec4<f32>,
                    screen: vec4<f32>,
                    levels: array<vec4<f32>, 16>,
                    projection: vec4<f32>,
                };
                @group(0) @binding(0) var<uniform> view: View;
                struct VertexOut {
                    @builtin(position) clip: vec4<f32>,
                    @location(0) world: vec3<f32>,
                };
                @vertex
                fn vs_main(@location(0) position: vec3<f32>) -> VertexOut {
                    var out: VertexOut;
                    out.clip = view.matrix * vec4<f32>(position, 1.0);
                    out.world = position;
                    return out;
                }
                @fragment
                fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
                    let normal = normalize(cross(dpdx(input.world), dpdy(input.world)));
                    let light = normalize(vec3<f32>(0.4, -0.7, 0.6));
                    let lambert = clamp(abs(dot(normal, light)), 0.0, 1.0);
                    return vec4<f32>(vec3<f32>(0.82, 0.55, 0.25) * (0.3 + 0.7 * lambert), 1.0);
                }
                "#
                .into(),
            ),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cluster compute"),
                bind_group_layouts: &[Some(&compute_layout)],
                immediate_size: 0,
            });
        let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cluster cull"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cluster render"),
                bind_group_layouts: &[Some(&render_layout)],
                immediate_size: 0,
            });
        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cluster draw"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
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
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
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
        let pyramid = crate::pyramid::DepthPyramidGpu::new(&device);
        Some(Self {
            device,
            queue,
            compute,
            render,
            indirect,
            view,
            chunks,
            chunk_ranges,
            chunk_resident,
            last_view: None,
            parse,
            group_radius,
            coarse_pixels,
            level_count,
            group_error,
            levels: entries.iter().map(|(_, level, _)| *level).collect(),
            cluster_count: args.len() as u32,
            last_stats: ClusterStats::default(),
            records,
            compute_layout,
            compute_bind: None,
            render_bind,
            depth: None,
            depth_view: None,
            pyramid,
            size: (0, 0),
            occluding: false,
            level_select: !errors.is_empty(),
            residency,
            cluster_bytes,
            resident_count: initial_count,
            pyramid_verified: false,
            pyramid_table: Vec::new(),
        })
    }

    /// Builds a renderer that keeps only as many chunks as `budget_bytes` allows; the rest stay on the
    /// "disk" side and are skipped, which is how a 2 GB archive fits in a small VRAM budget.
    pub fn with_budget(
        device: wgpu::Device,
        queue: wgpu::Queue,
        clusters: &[Meshlet],
        budget_bytes: u64,
    ) -> Option<Self> {
        let entries: Vec<(Meshlet, u32, u32)> = clusters
            .iter()
            .enumerate()
            .map(|(index, cluster)| (cluster.clone(), 0, index as u32))
            .collect();
        let radii: Vec<f32> = clusters.iter().map(|cluster| cluster.radius).collect();
        Self::build(
            device,
            queue,
            &entries,
            &radii,
            &[],
            0.0,
            0,
            Some(budget_bytes),
        )
    }

    /// Uploads a flat cluster list; every cluster is level 0 in its own group.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, clusters: &[Meshlet]) -> Option<Self> {
        let entries: Vec<(Meshlet, u32, u32)> = clusters
            .iter()
            .enumerate()
            .map(|(index, cluster)| (cluster.clone(), 0, index as u32))
            .collect();
        let radii: Vec<f32> = clusters.iter().map(|cluster| cluster.radius).collect();
        Self::build(device, queue, &entries, &radii, &[], 0.0, 0, None)
    }

    /// Uploads a cluster LOD hierarchy; the compute pass picks the level per group from its projected
    /// size, using the coarse level below `coarse_pixels` on screen.
    pub fn from_lod(
        device: wgpu::Device,
        queue: wgpu::Queue,
        groups: &[crate::meshlet::ClusterGroup],
        coarse_pixels: f32,
    ) -> Option<Self> {
        let level_count = groups.iter().map(|group| group.levels.len()).max().unwrap_or(1).max(1) as u32;
        let mut entries: Vec<(Meshlet, u32, u32)> = Vec::new();
        let mut radii: Vec<f32> = Vec::with_capacity(groups.len());
        for (group_index, group) in groups.iter().enumerate() {
            let radius = group
                .levels
                .iter()
                .flat_map(|level| level.iter())
                .map(|cluster| cluster.radius)
                .fold(0.0f32, f32::max);
            radii.push(radius);
            for (level_index, level) in group.levels.iter().enumerate() {
                for cluster in level {
                    entries.push((cluster.clone(), level_index as u32, group_index as u32));
                }
            }
        }
        if entries.is_empty() {
            return None;
        }
        let mut errors = vec![0.0f32; groups.len() * level_count as usize];
        for (group_index, group) in groups.iter().enumerate() {
            let last = group.errors_world.last().copied().unwrap_or(0.0);
            for level in 0..level_count as usize {
                errors[group_index * level_count as usize + level] =
                    group.errors_world.get(level).copied().unwrap_or(last);
            }
        }
        Self::build(
            device,
            queue,
            &entries,
            &radii,
            &errors,
            coarse_pixels,
            level_count,
            None,
        )
    }

    /// Creates the depth texture, the pyramid storage and the cull bind group for a target size.
    fn ensure_targets(&mut self, width: u32, height: u32) {
        if self.size == (width, height) && self.depth.is_some() {
            return;
        }
        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cluster depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth_view = Some(depth.create_view(&wgpu::TextureViewDescriptor::default()));
        self.pyramid.ensure(&self.device, width, height);
        let mut table: Vec<(u32, u32, u32)> = Vec::new();
        let (mut level_width, mut level_height) = (width.max(1), height.max(1));
        table.push((0, level_width, level_height));
        while level_width > 1 || level_height > 1 {
            level_width = level_width.div_ceil(2);
            level_height = level_height.div_ceil(2);
            table.push((0, level_width, level_height));
            if table.len() >= 16 {
                break;
            }
        }
        let Some(pyramid_view) = self.pyramid.array_view() else {
            return;
        };
        self.compute_bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cluster compute"),
            layout: &self.compute_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.records.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.indirect.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.view.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.group_radius.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.group_error.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(pyramid_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.residency.as_entire_binding(),
                },
            ],
        }));
        self.depth = Some(depth);
        self.pyramid_table = table;
        self.size = (width, height);
    }

    /// Renders the pyramid from this frame's depth so the next frame can cull against it.
    fn build_pyramid(&mut self) {
        let (Some(depth_view), true) = (self.depth_view.as_ref(), self.pyramid.levels() > 1) else {
            return;
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pyramid"),
            });
        self.pyramid
            .build(&self.device, &self.queue, &mut encoder, depth_view);
        self.queue.submit(std::iter::once(encoder.finish()));
        if self.pyramid_verified {
            self.occluding = true;
        }
    }

    /// Keeps clusters resident while their combined bytes fit the budget, in upload order. Clusters over
    /// the budget are skipped by the cull pass, which is what keeps a 2 GB archive inside a small VRAM
    /// budget.
    pub fn set_budget_bytes(&mut self, budget: u64) {
        // Mark the clusters that fit; uploaded chunks may still contain evicted clusters, which the cull
        // pass skips, so the flag is what controls what is drawn.
        let mut resident = vec![0u32; self.cluster_bytes.len()];
        let mut used = 0u64;
        let mut count = 0usize;
        for (index, bytes) in self.cluster_bytes.iter().enumerate() {
            if used + *bytes <= budget {
                resident[index] = 1;
                used += *bytes;
                count += 1;
            }
        }
        self.queue
            .write_buffer(&self.residency, 0, bytemuck::cast_slice(&resident));
        self.resident_count = count;
    }

    /// How many clusters the budget keeps resident.
    pub fn resident_count(&self) -> usize {
        self.resident_count
    }

    /// Bytes actually uploaded for resident chunks.
    pub fn resident_bytes(&self) -> u64 {
        self.chunks
            .iter()
            .map(|chunk| chunk.vertex_bytes + chunk.index_bytes)
            .sum()
    }

    /// Total bytes the clusters would need if all were resident.
    pub fn total_bytes(&self) -> u64 {
        self.cluster_bytes.iter().sum()
    }

    /// Turns occlusion on or off; it is on by default once a pyramid exists.
    /// Screen-space error budget for the coarse level, in pixels. Jewellery can use 0.5 for gem and
    /// prong clusters while a 1.0 default suits general geometry.
    pub fn set_coarse_pixels(&mut self, pixels: f32) {
        self.coarse_pixels = pixels.max(0.0);
        self.last_view = None;
    }

    pub fn set_occlusion(&mut self, enabled: bool) {
        // A verified pyramid only takes effect after it has actually been built once, so the first frame
        // draws everything into depth and the second can cull against it.
        self.pyramid_verified = enabled;
        if !enabled {
            self.occluding = false;
        }
    }

    /// Whether the cull pass used a pyramid on the last frame.
    pub fn occluding(&self) -> bool {
        self.occluding
    }

    /// Runs the cull pass and the indirect draw, then reports the numbers.
    pub fn render(
        &mut self,
        colour_view: &wgpu::TextureView,
        camera: Camera,
        width: u32,
        height: u32,
    ) -> Result<ClusterStats, crate::RenderError> {
        self.ensure_targets(width, height);
        let Some(depth_view) = self.depth_view.clone() else {
            return Err(crate::RenderError::Lost);
        };
        let view_hash = view_hash(&camera);
        let moving = self.last_view != Some(view_hash);
        self.last_view = Some(view_hash);
        let matrix = camera.view_projection(width as f32 / height.max(1) as f32);
        let uniform = ViewUniform {
            planes: planes_from(matrix),
            matrix: matrix_to_columns(matrix),
            eye_tan: [
                camera.eye[0],
                camera.eye[1],
                camera.eye[2],
                camera.tan_half_fov(),
            ],
            params: [height as f32 * 0.5, 0.5, camera.near, self.coarse_pixels],
            screen: [
                width as f32,
                height as f32,
                if self.occluding && !moving && self.pyramid.levels() > 1 {
                    self.pyramid.levels() as f32
                } else {
                    0.0
                },
                if self.level_select { self.level_count.max(1) as f32 } else { 0.0 },
            ],
            projection: [
                if camera.is_orthographic() { 1.0 } else { 0.0 },
                camera.half_span().max(1e-6),
                camera.far,
                camera.near,
            ],
            levels: {
                let mut table = [[0.0f32; 4]; 16];
                for (level, (offset, level_width, level_height)) in
                    self.pyramid_table.iter().enumerate().take(16)
                {
                    table[level] = [
                        *offset as f32,
                        *level_width as f32,
                        *level_height as f32,
                        0.0,
                    ];
                }
                table
            },
        };
        self.queue
            .write_buffer(&self.view, 0, bytemuck::bytes_of(&uniform));
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("cluster"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cluster cull"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.compute);
            if let Some(bind) = self.compute_bind.as_ref() {
                pass.set_bind_group(0, bind, &[]);
            }
            pass.dispatch_workgroups(self.cull_count().div_ceil(64), 1, 1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cluster draw"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: colour_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.render);
            pass.set_bind_group(0, &self.render_bind, &[]);
            // One multi-draw per chunk: clusters are appended in chunk order, so each chunk owns a
            // contiguous slice of the indirect list.
            for (index, (start, count)) in self.chunk_ranges.iter().enumerate() {
                if !self.chunk_resident[index] {
                    // An evicted chunk is never drawn: its clusters are all marked non-resident.
                    continue;
                }
                let chunk = &self.chunks[index];
                pass.set_vertex_buffer(0, chunk.vertices.slice(..));
                pass.set_index_buffer(chunk.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.multi_draw_indexed_indirect(&self.indirect, u64::from(*start) * 20, *count);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|_| crate::RenderError::Lost)?;
        // Build the pyramid from this frame for the next one to cull against.
        self.build_pyramid();
        Ok(self.last_stats)
    }

    /// Copies the indirect list back and counts the clusters with work, for tests and diagnostics.
    pub fn count_visible(&mut self) -> usize {
        let size = (self.cluster_count as usize * std::mem::size_of::<DrawArgs>()) as u64;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("cluster read"),
            });
        encoder.copy_buffer_to_buffer(&self.indirect, 0, &self.parse, 0, size);
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("poll");
        let slice = self.parse.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("map");
        let data = slice.get_mapped_range().expect("mapped");
        let args: &[DrawArgs] = bytemuck::cast_slice(&data);
        let visible = args.iter().filter(|arg| arg.index_count > 0).count();
        drop(data);
        self.parse.unmap();
        self.last_stats = ClusterStats {
            clusters: self.cluster_count as usize,
            visible,
        };
        visible
    }

    /// Counts the visible clusters per level, for the LOD test.
    pub fn count_visible_levels(&mut self) -> (usize, usize) {
        let size = (self.cluster_count as usize * std::mem::size_of::<DrawArgs>()) as u64;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("cluster read"),
            });
        encoder.copy_buffer_to_buffer(&self.indirect, 0, &self.parse, 0, size);
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("poll");
        let slice = self.parse.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("map");
        let data = slice.get_mapped_range().expect("mapped");
        let args: &[DrawArgs] = bytemuck::cast_slice(&data);
        let mut fine = 0;
        let mut coarse = 0;
        for (index, arg) in args.iter().enumerate() {
            if arg.index_count == 0 {
                continue;
            }
            if self.levels.get(index).copied().unwrap_or(0) == 0 {
                fine += 1;
            } else {
                coarse += 1;
            }
        }
        drop(data);
        self.parse.unmap();
        (fine, coarse)
    }

    /// Reads one pyramid layer back.
    pub fn debug_pyramid(&self, level: usize) -> Option<(f32, Vec<f32>)> {
        self.pyramid.debug_layer(&self.device, &self.queue, level)
    }

    /// The clusters uploaded, culled or not.
    pub fn clusters(&self) -> usize {
        self.cluster_count as usize
    }
}

/// A cheap identity for a camera: a new hash means the depth pyramid is stale.
fn view_hash(camera: &Camera) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    let mut mix = |value: f64| {
        hash ^= value.to_bits();
        hash = hash.wrapping_mul(0x100000001b3);
    };
    for value in camera
        .eye
        .iter()
        .chain(camera.target.iter())
        .chain(camera.up.iter())
        .copied()
        .map(f64::from)
    {
        mix(value);
    }
    match camera.projection {
        crate::Projection::Perspective { vertical_fov_rad } => mix(f64::from(vertical_fov_rad)),
        crate::Projection::Orthographic { vertical_span } => mix(f64::from(vertical_span) + 1.0),
    }
    mix(f64::from(camera.near));
    mix(f64::from(camera.far));
    hash
}

/// Normalised frustum planes from a column-major view-projection.
fn planes_from(matrix: [f32; 16]) -> [[f32; 4]; 6] {
    let row = |index: usize| {
        [
            matrix[index],
            matrix[4 + index],
            matrix[8 + index],
            matrix[12 + index],
        ]
    };
    let (r0, r1, r2, r3) = (row(0), row(1), row(2), row(3));
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
    [
        normalise(combine(r3, r0, 1.0)),
        normalise(combine(r3, r0, -1.0)),
        normalise(combine(r3, r1, 1.0)),
        normalise(combine(r3, r1, -1.0)),
        normalise(r2),
        normalise(combine(r3, r2, -1.0)),
    ]
}

fn matrix_to_columns(matrix: [f32; 16]) -> [[f32; 4]; 4] {
    [
        [matrix[0], matrix[1], matrix[2], matrix[3]],
        [matrix[4], matrix[5], matrix[6], matrix[7]],
        [matrix[8], matrix[9], matrix[10], matrix[11]],
        [matrix[12], matrix[13], matrix[14], matrix[15]],
    ]
}

/// Highest LOD level whose projected geometric error stays below `threshold_px`. Level 0 is always
/// acceptable. This is the CPU oracle for the compute shader and lets tests pin the accuracy policy.
pub fn choose_level(
    errors_world: &[f32],
    distance: f32,
    half_height_px: f32,
    tan_half_fov: f32,
    threshold_px: f32,
) -> usize {
    if errors_world.len() <= 1 || distance <= 0.0 || tan_half_fov <= 0.0 || threshold_px <= 0.0 {
        return 0;
    }
    let mut chosen = 0;
    for (level, error) in errors_world.iter().enumerate().skip(1) {
        let pixels = *error * half_height_px / (distance * tan_half_fov);
        if pixels < threshold_px {
            chosen = level;
        } else {
            break;
        }
    }
    chosen
}

#[cfg(test)]
mod level_choice_tests {
    use super::choose_level;

    #[test]
    fn the_level_is_the_highest_one_under_the_pixel_budget() {
        let errors = [0.0, 0.01, 0.05, 0.2];
        // 1000 px half-height, 35 degree fov, unit distance: error 0.05 -> ~0.08 px per 0.001.
        let tan = (35f32 / 2.0).to_radians().tan();
        assert_eq!(choose_level(&errors, 10.0, 500.0, tan, 32.0), 3);
        assert_eq!(choose_level(&errors, 10.0, 500.0, tan, 8.0), 2);
        assert_eq!(choose_level(&errors, 10.0, 500.0, tan, 2.0), 1);
        assert_eq!(choose_level(&errors, 10.0, 500.0, tan, 1.0), 0);
        assert_eq!(choose_level(&errors, 10.0, 500.0, tan, 0.0), 0);
    }
}

#[cfg(test)]
mod jewellery_threshold_tests {
    use super::choose_level;

    #[test]
    fn gem_and_shank_thresholds_differ_at_the_same_distance() {
        let errors = [0.0, 0.05];
        let tan = 35f32.to_radians().tan() / 2.0;
        let half = 500.0;
        // Distance where 0.05 mm projects to about 0.8 px.
        let distance = 0.05 * half / (0.8 * tan);
        assert_eq!(choose_level(&errors, distance, half, tan, 0.5), 0, "gem keeps LOD 0");
        assert_eq!(choose_level(&errors, distance, half, tan, 1.0), 1, "shank may use coarse");
    }
}
