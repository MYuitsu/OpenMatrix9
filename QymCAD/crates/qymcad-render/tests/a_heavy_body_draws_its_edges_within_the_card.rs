//! A BODY OF HUNDREDS OF THOUSANDS OF EDGES DOES NOT BRING THE WINDOW DOWN under a chamfer. Reported behaviour: a body
//! made from a mesh as it is, a chamfer picked, and the frame asked the graphics card for 402 601 368 bytes of geometry
//! against its limit of 268 435 456. The edges drawn for picking stay within a budget, and a picked edge is always drawn.
use qymcad_render::{edges_to_draw, EDGE_SEGMENTS_DRAWN};

#[test]
fn a_heavy_body_draws_its_edges_within_the_budget_and_every_picked_one() {
    // every side of 178 000 triangles an edge of its own
    let n = 267_000;
    let polys: Vec<Vec<[f32; 3]>> = (0..n).map(|i| vec![[i as f32, 0.0, 0.0], [i as f32, 1.0, 0.0]]).collect();
    let mut picked = vec![false; n];
    for i in [5, 150_000, n - 1] {
        picked[i] = true;
    }
    let drawn = edges_to_draw(&polys, &picked, EDGE_SEGMENTS_DRAWN);
    let pieces: usize = drawn.iter().map(|&i| polys[i].len() - 1).sum();
    assert!(pieces <= EDGE_SEGMENTS_DRAWN, "{pieces} pieces of edges drawn in one frame, over the budget of {EDGE_SEGMENTS_DRAWN}");
    for i in [5, 150_000, n - 1] {
        assert!(drawn.contains(&i), "the picked edge {i} was not drawn");
    }
}

#[test]
fn a_light_body_draws_every_edge() {
    let polys: Vec<Vec<[f32; 3]>> = (0..12).map(|i| vec![[i as f32, 0.0, 0.0], [i as f32, 1.0, 0.0], [i as f32, 2.0, 0.0]]).collect();
    let drawn = edges_to_draw(&polys, &[false; 12], EDGE_SEGMENTS_DRAWN);
    assert_eq!(drawn.len(), 12, "a cube's edges are all drawn");
}
