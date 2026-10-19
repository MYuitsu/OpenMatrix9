//! G00-03 probe — a real OCCT tessellation rendered through wgpu.
//!
//! `L4-03-02-01` uploads the mesh that OCCT's `BRepMesh_IncrementalMesh` produced (written by
//! `native/occt-rust-proof --bin export_mesh`) and renders it with a perspective camera.
//! `L4-03-02-02` sweeps orbit, pan, zoom and DPI through the same pipeline and reads each frame back, so
//! the transform path is exercised, not just the shader.
//!
//! Feasibility only: this lives under `native/` and is not part of the product workspace.

use std::time::Instant;
use wgpu::util::DeviceExt;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FRAMES: u32 = 60;

fn parse_mesh(path: &str) -> (Vec<[f32; 3]>, Vec<u32>) {
    let text = std::fs::read_to_string(path).expect("mesh file");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            let mut parts = rest.split_whitespace();
            let x: f32 = parts.next().expect("x").parse().expect("x");
            let y: f32 = parts.next().expect("y").parse().expect("y");
            let z: f32 = parts.next().expect("z").parse().expect("z");
            vertices.push([x, y, z]);
        } else if let Some(rest) = line.strip_prefix("f ") {
            let mut parts = rest.split_whitespace();
            let a: u32 = parts.next().expect("a").parse::<u32>().expect("a") - 1;
            let b: u32 = parts.next().expect("b").parse::<u32>().expect("b") - 1;
            let c: u32 = parts.next().expect("c").parse::<u32>().expect("c") - 1;
            indices.extend_from_slice(&[a, b, c]);
        }
    }
    (vertices, indices)
}

fn perspective(aspect: f32, fovy: f32, znear: f32, zfar: f32) -> [f32; 16] {
    let f = 1.0 / (fovy * 0.5).tan();
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, zfar / (znear - zfar), -1.0,
        0.0, 0.0, znear * zfar / (znear - zfar), 0.0,
    ]
}

fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [f32; 16] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let normalize = |a: [f32; 3]| {
        let length = dot(a, a).sqrt().max(f32::MIN_POSITIVE);
        [a[0] / length, a[1] / length, a[2] / length]
    };
    let forward = normalize(sub(center, eye));
    let right = normalize(cross(forward, up));
    let true_up = cross(right, forward);
    [
        right[0], true_up[0], -forward[0], 0.0,
        right[1], true_up[1], -forward[1], 0.0,
        right[2], true_up[2], -forward[2], 0.0,
        -dot(right, eye), -dot(true_up, eye), dot(forward, eye), 1.0,
    ]
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

struct Frame {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mesh_path = args.get(1).cloned().unwrap_or_else(|| "mesh.txt".to_string());
    let (vertices, indices) = parse_mesh(&mesh_path);

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
    .expect("no adapter");
    let info = adapter.get_info();
    let started = Instant::now();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("om9-gpu-probe"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: Default::default(),
        experimental_features: Default::default(),
        trace: wgpu::Trace::Off,
    }))
    .expect("no device");
    let device_seconds = started.elapsed().as_secs_f64();

    let vertex_data: Vec<u8> = vertices
        .iter()
        .flat_map(|position| position.iter().flat_map(|value| value.to_le_bytes()))
        .collect();
    let index_data: Vec<u8> = indices.iter().flat_map(|value| value.to_le_bytes()).collect();
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("brep vertices"),
        contents: &vertex_data,
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("brep indices"),
        contents: &index_data,
        usage: wgpu::BufferUsages::INDEX,
    });
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mvp"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mvp"),
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mvp"),
        layout: &bind_layout,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mesh"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            struct Mvp { matrix: mat4x4<f32> };
            @group(0) @binding(0) var<uniform> mvp: Mvp;
            @vertex
            fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
                return mvp.matrix * vec4<f32>(position, 1.0);
            }
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(0.85, 0.55, 0.2, 1.0);
            }
            "#
            .into(),
        ),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("layout"),
        bind_group_layouts: &[Some(&bind_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh"),
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
            })],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
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

    let make_frame = |width: u32, height: u32| -> Frame {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(width) * u64::from(height) * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Frame { texture, view, readback, width, height }
    };

    let camera_eye = |orbit_deg: f32, zoom: f32, pan: f32| -> [f32; 3] {
        let angle = orbit_deg.to_radians();
        let distance = 14.0 / zoom;
        [pan + distance * angle.sin(), -2.0, distance * angle.cos()]
    };

    let draw = |frame: &Frame, eye: [f32; 3], target: [f32; 3]| -> (f64, u64) {
        let mvp = multiply(
            perspective(frame.width as f32 / frame.height as f32, 45f32.to_radians(), 0.1, 100.0),
            look_at(eye, target, [0.0, 1.0, 0.0]),
        );
        let matrix_bytes: Vec<u8> = mvp.iter().flat_map(|value| value.to_le_bytes()).collect();
        queue.write_buffer(&uniform, 0, &matrix_bytes);
        let started = Instant::now();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &frame.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &frame.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(frame.width * 4),
                    rows_per_image: Some(frame.height),
                },
            },
            wgpu::Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("poll");
        let elapsed = started.elapsed().as_secs_f64();
        let slice = frame.readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("map");
        let data = slice.get_mapped_range().expect("mapped range");
        let mut painted = 0u64;
        for pixel in data.chunks_exact(4) {
            // The clear colour is (12,12,20); the mesh is (216,140,51).
            if pixel[0] > 150 && pixel[1] > 90 {
                painted += 1;
            }
        }
        drop(data);
        frame.readback.unmap();
        (elapsed, painted)
    };

    let base = make_frame(WIDTH, HEIGHT);
    let mut frame_times = Vec::new();
    let mut painted_base = 0;
    for _ in 0..FRAMES {
        let (seconds, painted) = draw(&base, camera_eye(0.0, 1.0, 0.0), [0.0, 0.0, 0.0]);
        frame_times.push(seconds);
        painted_base = painted;
    }
    let mean = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
    let mut sorted = frame_times.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99 = sorted[(sorted.len() as f64 * 0.99) as usize - 1];

    let mut cases = Vec::new();
    for (name, eye, target, width, height) in [
        ("orbit_y_plus_30", camera_eye(30.0, 1.0, 0.0), [0.0, 0.0, 0.0], WIDTH, HEIGHT),
        ("orbit_y_minus_30", camera_eye(-30.0, 1.0, 0.0), [0.0, 0.0, 0.0], WIDTH, HEIGHT),
        ("pan_x_plus_2", camera_eye(0.0, 1.0, 2.0), [0.0, 0.0, 0.0], WIDTH, HEIGHT),
        ("zoom_2x", camera_eye(0.0, 2.0, 0.0), [0.0, 0.0, 0.0], WIDTH, HEIGHT),
        ("zoom_0_5x", camera_eye(0.0, 0.5, 0.0), [0.0, 0.0, 0.0], WIDTH, HEIGHT),
        (
            "dpi_1_5x",
            camera_eye(0.0, 1.0, 0.0),
            [0.0, 0.0, 0.0],
            (f64::from(WIDTH) * 1.5) as u32,
            (f64::from(HEIGHT) * 1.5) as u32,
        ),
        ("dpi_2x", camera_eye(0.0, 1.0, 0.0), [0.0, 0.0, 0.0], WIDTH * 2, HEIGHT * 2),
    ] {
        let frame = make_frame(width, height);
        let (seconds, painted) = draw(&frame, eye, target);
        cases.push(format!(
            "{{\"name\": \"{name}\", \"width\": {width}, \"height\": {height}, \"frame_seconds\": {seconds:.6}, \"painted_pixels\": {painted}, \"painted_fraction\": {:.5}}}",
            painted as f64 / (u64::from(width) * u64::from(height)) as f64
        ));
    }

    let mesh_bytes = (vertex_data.len() + index_data.len()) as u64;
    let target_bytes = u64::from(WIDTH) * u64::from(HEIGHT) * 4;
    let gpu_bytes = mesh_bytes + target_bytes + target_bytes / 2;
    println!(
        "{{\"adapter\": {{\"name\": \"{}\", \"backend\": \"{:?}\", \"device_type\": \"{:?}\"}}, \
         \"device_seconds\": {:.4}, \"mesh\": {{\"path\": \"{}\", \"vertices\": {}, \"triangles\": {}, \"bytes\": {}}}, \
         \"base\": {{\"frames\": {}, \"mean_frame_seconds\": {:.6}, \"p99_frame_seconds\": {:.6}, \"fps\": {:.1}, \"painted_pixels\": {}, \"total_pixels\": {}}}, \
         \"cases\": [{}], \"gpu_bytes_estimate\": {}}}",
        info.name,
        info.backend,
        info.device_type,
        device_seconds,
        mesh_path,
        vertices.len(),
        indices.len() / 3,
        mesh_bytes,
        FRAMES,
        mean,
        p99,
        1.0 / mean,
        painted_base,
        u64::from(WIDTH) * u64::from(HEIGHT),
        cases.join(", "),
        gpu_bytes
    );
}
