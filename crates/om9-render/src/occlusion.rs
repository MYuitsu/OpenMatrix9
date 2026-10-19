//! Occlusion culling with a depth pyramid (Hi-Z), the technique that pays off in crowded scenes.
//!
//! The main pass leaves a depth buffer. A **max** pyramid is built from it: each level stores the farthest
//! depth of the four texels below, so a cluster is culled only when it is certainly hidden. A cluster's
//! screen rectangle picks a pyramid level whose texel roughly matches the rectangle, and the test is
//!
//! ```text
//! nearest_depth(cluster) > max_depth_in_rect  ->  fully occluded
//! ```
//!
//! Using the farthest depth of the region keeps the test conservative: a cluster is never culled because a
//! neighbour happened to be closer.
//!
//! The pyramid maths is kept CPU-only and tested here; the GPU path samples the same numbers.

/// A max-depth pyramid over a depth buffer, one level per halving.
#[derive(Debug, Clone, PartialEq)]
pub struct DepthPyramid {
    levels: Vec<Vec<f32>>,
    widths: Vec<usize>,
    heights: Vec<usize>,
}

impl DepthPyramid {
    /// Builds the pyramid from a row-major depth buffer (`0..=1`, smaller is nearer).
    pub fn build(depth: &[f32], width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || depth.len() != width * height {
            return None;
        }
        let mut levels = Vec::new();
        let mut widths = Vec::new();
        let mut heights = Vec::new();
        levels.push(depth.to_vec());
        widths.push(width);
        heights.push(height);
        let (mut current_width, mut current_height) = (width, height);
        while current_width > 1 || current_height > 1 {
            let next_width = current_width.div_ceil(2);
            let next_height = current_height.div_ceil(2);
            let mut next = vec![0.0f32; next_width * next_height];
            let previous = levels.last().expect("level");
            for y in 0..next_height {
                for x in 0..next_width {
                    let mut maximum = 0.0f32;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let source_x = (x * 2 + dx).min(current_width - 1);
                            let source_y = (y * 2 + dy).min(current_height - 1);
                            maximum = maximum.max(previous[source_y * current_width + source_x]);
                        }
                    }
                    next[y * next_width + x] = maximum;
                }
            }
            levels.push(next);
            widths.push(next_width);
            heights.push(next_height);
            current_width = next_width;
            current_height = next_height;
        }
        Some(Self { levels, widths, heights })
    }

    pub fn levels(&self) -> usize {
        self.levels.len()
    }

    /// Picks the level whose texel is about as large as the rectangle, so one lookup covers it.
    pub fn level_for(&self, rect_width: f32, rect_height: f32) -> usize {
        let extent = rect_width.max(rect_height).max(1.0);
        let level = extent.log2().floor().max(0.0) as usize;
        level.min(self.levels.len() - 1)
    }

    /// The farthest depth under a pixel rectangle at one level.
    pub fn max_depth(&self, level: usize, x: f32, y: f32, width: f32, height: f32) -> f32 {
        let level = level.min(self.levels.len() - 1);
        let level_width = self.widths[level];
        let level_height = self.heights[level];
        let base_level = 0;
        let scale = (self.widths[base_level] as f32 / level_width as f32).max(1.0);
        let left = (x / scale).floor().max(0.0) as usize;
        let top = (y / scale).floor().max(0.0) as usize;
        let right = ((x + width) / scale).ceil().max(left as f32 + 1.0) as usize;
        let bottom = ((y + height) / scale).ceil().max(top as f32 + 1.0) as usize;
        let mut maximum = 0.0f32;
        for row in top..bottom.min(level_height) {
            for column in left..right.min(level_width) {
                maximum = maximum.max(self.levels[level][row * level_width + column]);
            }
        }
        maximum
    }

    /// Conservative test: the cluster is culled only when its nearest point is behind everything in its
    /// screen rectangle.
    pub fn is_occluded(
        &self,
        rect_x: f32,
        rect_y: f32,
        rect_width: f32,
        rect_height: f32,
        nearest_depth: f32,
    ) -> bool {
        let level = self.level_for(rect_width, rect_height);
        let farthest = self.max_depth(level, rect_x, rect_y, rect_width, rect_height);
        // A tiny epsilon keeps a cluster touching the occluder visible, which is the safe side.
        farthest > 0.0 && farthest < 1.0 && nearest_depth > farthest + 1e-5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pyramid_keeps_the_farthest_depth_of_each_quad() {
        // 4x4 where the top-left 2x2 is near (0.1) and the rest is far (0.9).
        let mut depth = vec![0.9f32; 16];
        for y in 0..2 {
            for x in 0..2 {
                depth[y * 4 + x] = 0.1;
            }
        }
        let pyramid = DepthPyramid::build(&depth, 4, 4).expect("pyramid");
        assert_eq!(pyramid.levels(), 3, "4x4 -> 2x2 -> 1x1");
        assert!((pyramid.max_depth(1, 0.0, 0.0, 2.0, 2.0) - 0.1).abs() < 1e-6);
        assert!((pyramid.max_depth(1, 2.0, 2.0, 2.0, 2.0) - 0.9).abs() < 1e-6);
        assert!((pyramid.max_depth(2, 0.0, 0.0, 4.0, 4.0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn a_cluster_behind_a_near_object_is_occluded() {
        let mut depth = vec![1.0f32; 16];
        for y in 0..2 {
            for x in 0..2 {
                depth[y * 4 + x] = 0.2;
            }
        }
        let pyramid = DepthPyramid::build(&depth, 4, 4).expect("pyramid");
        // Covering the near corner and far away: occluded.
        assert!(pyramid.is_occluded(0.0, 0.0, 2.0, 2.0, 0.8));
        // Same depth but nearer than the occluder: visible.
        assert!(!pyramid.is_occluded(0.0, 0.0, 2.0, 2.0, 0.1));
        // Outside the occluder, nothing is in front: visible.
        assert!(!pyramid.is_occluded(2.0, 2.0, 2.0, 2.0, 0.8));
    }

    #[test]
    fn an_empty_depth_buffer_culls_nothing() {
        let depth = vec![1.0f32; 16];
        let pyramid = DepthPyramid::build(&depth, 4, 4).expect("pyramid");
        assert!(!pyramid.is_occluded(0.0, 0.0, 4.0, 4.0, 0.99));
    }

    #[test]
    fn a_bad_buffer_is_rejected() {
        assert!(DepthPyramid::build(&[], 0, 0).is_none());
        assert!(DepthPyramid::build(&[0.5, 0.5, 0.5], 4, 4).is_none());
    }
}
