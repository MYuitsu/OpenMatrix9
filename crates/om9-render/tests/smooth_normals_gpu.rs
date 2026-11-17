use om9_render::{Camera, Projection, normals, scene::SceneBatch, shapes};

#[test]
fn live_batch_interpolates_normals_and_retains_the_same_silhouette() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .expect("GPU adapter required for this validation");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .unwrap();
    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("normals proof"),
        size: wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("normals depth"),
        size: colour.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let sphere = shapes::sphere(12, 16, 1.);
    let faces: Vec<_> = sphere
        .indices
        .chunks_exact(3)
        .map(|t| ([t[0], t[1], t[2]], 0))
        .collect();
    let smooth = normals::smooth_faces(sphere.positions.clone(), &faces);
    let model = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let camera = Camera {
        eye: [0., -4., 1.],
        target: [0.; 3],
        up: [0., 0., 1.],
        projection: Projection::Perspective {
            vertical_fov_rad: 0.7,
        },
        near: 0.1,
        far: 100.,
    };
    let mut results = Vec::new();
    for (label, mesh) in [("flat", sphere), ("smooth", smooth)] {
        let mut batch = SceneBatch::new(device.clone(), queue.clone());
        batch.add_mesh(&mesh, model, [0.8, 0.8, 0.8, 1.]);
        batch
            .render_into(
                &colour.create_view(&Default::default()),
                &depth.create_view(&Default::default()),
                camera,
                256,
                256,
            )
            .unwrap();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("normals pixels"),
            size: 256 * 256 * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &colour,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(1024),
                    rows_per_image: Some(256),
                },
            },
            colour.size(),
        );
        queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();
        let data = slice.get_mapped_range().unwrap();
        let pixels = data.to_vec();
        drop(data);
        buffer.unmap();
        if let Some(directory) = std::env::var_os("OM9_NORMALS_PROOF_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let mut ppm = b"P6\n256 256\n255\n".to_vec();
            ppm.extend(pixels.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]));
            std::fs::write(directory.join(format!("{label}.ppm")), ppm).unwrap();
        }
        results.push(pixels);
    }
    let painted = |bytes: &Vec<u8>| bytes.chunks_exact(4).map(|p| p[0] > 30).collect::<Vec<_>>();
    assert_eq!(
        painted(&results[0]),
        painted(&results[1]),
        "shading cannot alter the silhouette"
    );
    let distinct = |pixels: &Vec<u8>| {
        pixels
            .chunks_exact(4)
            .filter(|p| p[0] > 30)
            .map(|p| p[0])
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    assert!(
        distinct(&results[1]) > distinct(&results[0]) + 30,
        "interpolation should remove flat triangle bands"
    );
    assert!(
        results[0]
            .iter()
            .zip(&results[1])
            .filter(|(a, b)| a != b)
            .count()
            > 3000
    );
}
