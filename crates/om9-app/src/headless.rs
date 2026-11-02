// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless viewport capture: render every workspace camera into a PNG without starting the
//! application, a window, egui or a swapchain.
//!
//! Geometry is taken from the same published `Scene` the viewport draws. Curves and points go
//! through the overlay line pass, meshes through the scene batch; the result is the viewport image
//! a reader can open, never an OS screenshot.
use crate::session::Controller;
use om9_archive::Archive;
use om9_core::{Error, Result, budget::Limits};
use om9_render::{Frame, LayerLine, LayerMesh, LayerPoint, MeshData, Renderer};
use om9_scene::scene::{RenderBuffers, Scene};
use std::path::{Path, PathBuf};

/// Renders one PNG per workspace viewport into `out_dir` and returns the written files.
pub fn render_project(
    project: &Path,
    out_dir: &Path,
    width: u32,
    height: u32,
) -> Result<Vec<PathBuf>> {
    let frames = render_frames(project, width, height)?;
    std::fs::create_dir_all(out_dir).map_err(|error| Error::Io(error.to_string()))?;
    let mut written = Vec::new();
    for (index, (title, frame)) in frames.iter().enumerate() {
        let path = out_dir.join(format!("{}-{}.png", index + 1, slug(title)));
        save_png(&path, frame.width, frame.height, &frame.pixels)?;
        written.push(path);
    }
    Ok(written)
}

/// Renders the four workspace viewports into one 2x2 sheet (`2*width` by `2*height`).
pub fn render_sheet(project: &Path, out: &Path, width: u32, height: u32) -> Result<PathBuf> {
    let frames = render_frames(project, width, height)?;
    let sheet_width = width * 2;
    let sheet_height = height * 2;
    let mut pixels = vec![0u8; (sheet_width * sheet_height * 4) as usize];
    for (index, (_, frame)) in frames.iter().take(4).enumerate() {
        let origin_x = (index as u32 % 2) * width;
        let origin_y = (index as u32 / 2) * height;
        let row_bytes = (width * 4) as usize;
        for row in 0..height {
            let source = (row * width * 4) as usize;
            let destination = ((origin_y + row) * sheet_width * 4 + origin_x * 4) as usize;
            pixels[destination..destination + row_bytes]
                .copy_from_slice(&frame.pixels[source..source + row_bytes]);
        }
    }
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| Error::Io(error.to_string()))?;
    }
    save_png(out, sheet_width, sheet_height, &pixels)?;
    Ok(out.to_path_buf())
}

fn save_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<()> {
    image::save_buffer(path, pixels, width, height, image::ColorType::Rgba8)
        .map_err(|error| Error::Io(error.to_string()))
}

fn render_frames(project: &Path, width: u32, height: u32) -> Result<Vec<(&'static str, Frame)>> {
    if width == 0 || height == 0 {
        return Err(Error::Invalid("render size"));
    }
    let limits = Limits::default();
    let restored = Archive::open_restored(project, &limits, &std::collections::BTreeSet::new())?;
    let controller = Controller::new(restored.document, limits)?;
    let scene = controller.scene().ok_or(Error::Missing("render scene"))?;
    let (meshes, lines, points) = layers(scene)?;
    let mut renderer = Renderer::new().ok_or(Error::Unsupported("headless GPU adapter"))?;
    let mut frames = Vec::new();
    for (title, camera, cplane) in controller.view_cameras() {
        // The ruler grid belongs to the view's CPlane, so each tile gets its own grid, drawn
        // first with the scene lines over it — the same order the interactive viewport uses.
        let mut view_lines = Vec::new();
        if let Ok(grid) = om9_scene::grid::Grid::new(cplane) {
            for line in grid.lines {
                view_lines.push(LayerLine {
                    start: line.start,
                    end: line.end,
                    colour: [
                        line.color[0] as f32 / 255.,
                        line.color[1] as f32 / 255.,
                        line.color[2] as f32 / 255.,
                        1.,
                    ],
                });
            }
        }
        view_lines.extend(lines.iter().copied());
        let frame = renderer
            .render_layers(
                &meshes,
                &view_lines,
                &points,
                crate::gpu_viewport::scene_camera(&camera),
                width,
                height,
            )
            .map_err(|error| Error::Io(format!("headless render: {error:?}")))?;
        frames.push((title, frame));
    }
    Ok(frames)
}

/// Parses the optional `WIDTHxHEIGHT` argument.
pub fn parse_size(text: &str) -> Result<(u32, u32)> {
    let (width, height) = text
        .split_once('x')
        .ok_or(Error::Invalid("render size WIDTHxHEIGHT"))?;
    let width = width
        .parse::<u32>()
        .map_err(|_| Error::Invalid("render width"))?;
    let height = height
        .parse::<u32>()
        .map_err(|_| Error::Invalid("render height"))?;
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err(Error::Invalid("render size"));
    }
    Ok((width, height))
}

fn layers(scene: &Scene) -> Result<(Vec<LayerMesh>, Vec<LayerLine>, Vec<LayerPoint>)> {
    let mut meshes = Vec::new();
    let mut lines = Vec::new();
    let mut points = Vec::new();
    for object in &scene.objects {
        if !object.visible {
            continue;
        }
        let tint = [
            object.color[0] as f32 / 255.,
            object.color[1] as f32 / 255.,
            object.color[2] as f32 / 255.,
            1.,
        ];
        let transform = object.display_transform()?;
        match object.buffers.as_ref() {
            RenderBuffers::Mesh {
                vertices,
                triangles,
                ..
            } => {
                let mut positions = Vec::with_capacity(vertices.len());
                for vertex in vertices {
                    let point = transform.point(*vertex)?;
                    positions.push([point[0] as f32, point[1] as f32, point[2] as f32]);
                }
                if positions.is_empty() || triangles.is_empty() {
                    continue;
                }
                let mut indices = Vec::with_capacity(triangles.len() * 3);
                for (corners, _) in triangles {
                    indices.extend_from_slice(corners);
                }
                meshes.push(LayerMesh {
                    mesh: MeshData::new(positions, indices),
                    model: identity(),
                    tint,
                });
            }
            RenderBuffers::Curve { points: curve, .. } => {
                for pair in curve.windows(2) {
                    if let (Ok(start), Ok(end)) =
                        (transform.point(pair[0]), transform.point(pair[1]))
                    {
                        lines.push(LayerLine {
                            start,
                            end,
                            colour: tint,
                        });
                    }
                }
            }
            RenderBuffers::Points { points: dots, .. } => {
                for dot in dots {
                    if let Ok(point) = transform.point(*dot) {
                        points.push(LayerPoint {
                            point,
                            size: 4.,
                            colour: tint,
                        });
                    }
                }
            }
        }
    }
    Ok((meshes, lines, points))
}

fn identity() -> [[f32; 4]; 4] {
    [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]
}

fn slug(title: &str) -> String {
    title
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}
