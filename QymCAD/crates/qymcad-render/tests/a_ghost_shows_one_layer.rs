//! A GHOST SHOWS ONE LAYER: a translucent body is seen as its nearest surface over what stands behind it, not as every
//! face of it laid over the others.
//!
//! Reported behaviour: a neighbour's part shown translucent in context, a hexagonal hole through it - the walls of the
//! hole and the faces behind them heaped up darker than the rest, a mess of layers where one pane of glass was expected.
use egui::Color32;
use qymcad_render::{raster_band_blend, RasterTri};

/// A square over the whole 4 x 4 band at depth `z`, flat in colour `c`.
fn square(z: f32, c: Color32) -> [RasterTri; 2] {
    let (a, b, cc, d) = ([0.0, 0.0, z], [4.0, 0.0, z], [4.0, 4.0, z], [0.0, 4.0, z]);
    [RasterTri { v: [a, b, cc], cols: [c; 3] }, RasterTri { v: [a, cc, d], cols: [c; 3] }]
}

#[test]
fn two_ghost_faces_one_behind_the_other_blend_once() {
    let back = Color32::from_rgba_premultiplied(20, 20, 20, 255);
    let ghost = Color32::from_rgba_premultiplied(60, 60, 60, 128); // half-translucent, premultiplied
                                                                   // one pane over the background: what the eye expects of a ghost
    let mut once = vec![back; 16];
    raster_band_blend(&mut once, &[1.0; 16], 4, 0, 4, &square(0.5, ghost));
    // the same ghost with a second face of it behind the first - a wall of a hole behind the front face
    let mut two = vec![back; 16];
    let tris: Vec<RasterTri> = square(0.7, ghost).into_iter().chain(square(0.5, ghost)).collect();
    raster_band_blend(&mut two, &[1.0; 16], 4, 0, 4, &tris);
    assert_eq!(two[5], once[5], "a ghost with a face behind its front one came out {:?}, one pane over the background is {:?}", two[5], once[5]);
}

/// A GHOST BEHIND A SOLID BODY IS NOT SEEN: the solid's depth closes the pixel to it.
#[test]
fn a_ghost_behind_a_solid_body_leaves_the_pixel_alone() {
    let solid = Color32::from_rgba_premultiplied(200, 10, 10, 255);
    let ghost = Color32::from_rgba_premultiplied(60, 60, 60, 128);
    let mut px = vec![solid; 16];
    raster_band_blend(&mut px, &[0.3; 16], 4, 0, 4, &square(0.5, ghost));
    assert_eq!(px[5], solid, "a ghost behind a solid body showed through it");
}
