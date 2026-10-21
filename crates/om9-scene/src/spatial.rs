//! CPU spatial index for picking: a bounding-sphere BVH.
//!
//! Picking stays on the CPU because it must use the full-resolution geometry and return immediately,
//! but testing 27,000 objects per click is avoidable. The BVH only rejects objects whose sphere the ray
//! misses; the exact geometry test is unchanged.

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[allow(dead_code, clippy::needless_range_loop)]
impl Aabb {
    fn empty() -> Self {
        Self { min: [f64::INFINITY; 3], max: [f64::NEG_INFINITY; 3] }
    }
    fn add_sphere(&mut self, centre: [f64; 3], radius: f64) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(centre[axis] - radius);
            self.max[axis] = self.max[axis].max(centre[axis] + radius);
        }
    }
    fn union(&mut self, other: &Aabb) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(other.min[axis]);
            self.max[axis] = self.max[axis].max(other.max[axis]);
        }
    }
    fn hit_with_padding(&self, origin: [f64; 3], direction: [f64; 3], padding: f64) -> bool {
        let mut expanded = *self;
        for axis in 0..3 {
            expanded.min[axis] -= padding;
            expanded.max[axis] += padding;
        }
        expanded.hit(origin, direction)
    }

    fn hit(&self, origin: [f64; 3], direction: [f64; 3]) -> bool {
        let mut near = 0.0f64;
        let mut far = f64::INFINITY;
        for axis in 0..3 {
            let delta = direction[axis];
            if delta.abs() < 1e-12 {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return false;
                }
                continue;
            }
            let inv = 1.0 / delta;
            let mut t0 = (self.min[axis] - origin[axis]) * inv;
            let mut t1 = (self.max[axis] - origin[axis]) * inv;
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }
            near = near.max(t0);
            far = far.min(t1);
            if near > far {
                return false;
            }
        }
        far >= 0.0
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    bounds: Aabb,
    left: u32,
    right: u32,
    start: u32,
    count: u32,
}

pub struct SpatialIndex {
    nodes: Vec<Node>,
    order: Vec<usize>,
}

impl SpatialIndex {
    pub fn build(centres: &[[f64; 3]], radii: &[f64]) -> Self {
        let n = centres.len();
        let mut order: Vec<usize> = (0..n).collect();
        let mut nodes = Vec::new();
        if n > 0 {
            build_node(&mut nodes, &mut order, centres, radii, 0, n);
        }
        Self { nodes, order }
    }

    /// Object indices whose sphere may be hit by the ray, in arbitrary order.
    /// Conservative query for a pixel-tolerance pick: the ray is widened by `padding` in world units.
    pub fn ray_candidates_fat(&self, origin: [f64; 3], direction: [f64; 3], padding: f64) -> Vec<usize> {
        let mut out = Vec::new();
        if self.nodes.is_empty() {
            return out;
        }
        let mut stack = vec![0u32];
        while let Some(index) = stack.pop() {
            let node = self.nodes[index as usize];
            if !node.bounds.hit_with_padding(origin, direction, padding.max(0.0)) {
                continue;
            }
            if node.count > 0 {
                out.extend(self.order[node.start as usize..(node.start + node.count) as usize].iter().copied());
            } else {
                stack.push(node.left);
                stack.push(node.right);
            }
        }
        out
    }

    pub fn ray_candidates(&self, origin: [f64; 3], direction: [f64; 3]) -> Vec<usize> {
        let mut out = Vec::new();
        if self.nodes.is_empty() {
            return out;
        }
        let mut stack = vec![0u32];
        while let Some(index) = stack.pop() {
            let node = self.nodes[index as usize];
            if !node.bounds.hit(origin, direction) {
                continue;
            }
            if node.count > 0 {
                out.extend(self.order[node.start as usize..(node.start + node.count) as usize].iter().copied());
            } else {
                stack.push(node.left);
                stack.push(node.right);
            }
        }
        out
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[allow(clippy::needless_range_loop)]
fn build_node(
    nodes: &mut Vec<Node>,
    order: &mut [usize],
    centres: &[[f64; 3]],
    radii: &[f64],
    start: usize,
    end: usize,
) -> u32 {
    let mut bounds = Aabb::empty();
    for index in start..end {
        let object = order[index];
        bounds.add_sphere(centres[object], radii[object]);
    }
    let slot = nodes.len() as u32;
    nodes.push(Node { bounds, left: 0, right: 0, start: start as u32, count: (end - start) as u32 });
    if end - start <= 8 {
        return slot;
    }
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for index in start..end {
        let c = centres[order[index]];
        for axis in 0..3 {
            lo[axis] = lo[axis].min(c[axis]);
            hi[axis] = hi[axis].max(c[axis]);
        }
    }
    let axis = (0..3).max_by(|a, b| (hi[*a] - lo[*a]).total_cmp(&(hi[*b] - lo[*b]))).unwrap_or(0);
    if hi[axis] - lo[axis] < 1e-9 {
        return slot;
    }
    order[start..end].sort_by(|a, b| centres[*a][axis].total_cmp(&centres[*b][axis]));
    let mid = (start + end) / 2;
    let left = build_node(nodes, order, centres, radii, start, mid);
    let right = build_node(nodes, order, centres, radii, mid, end);
    nodes[slot as usize].left = left;
    nodes[slot as usize].right = right;
    nodes[slot as usize].count = 0;
    slot
}

#[cfg(test)]
mod tests {
    use super::{Aabb, SpatialIndex};

    #[test]
    fn a_ray_only_returns_objects_near_its_path() {
        let centres: Vec<[f64; 3]> = (0..100).map(|i| [i as f64, 0.0, 0.0]).collect();
        let radii = vec![0.4; 100];
        let index = SpatialIndex::build(&centres, &radii);
        let along = index.ray_candidates([-10.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(along.len(), 100);
        let away = index.ray_candidates([-10.0, 10.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(away.is_empty());
        assert!(index.node_count() > 1);
    }

    #[test]
    fn the_fat_ray_includes_pixel_tolerance_hits() {
        let centres = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let radii = vec![0.4, 0.4];
        let index = SpatialIndex::build(&centres, &radii);
        let origin = [-1.0, 0.5, 0.0];
        let direction = [1.0, 0.0, 0.0];
        assert_eq!(index.ray_candidates(origin, direction).len(), 0);
        assert_eq!(index.ray_candidates_fat(origin, direction, 0.2).len(), 2);
        assert_eq!(index.ray_candidates_fat(origin, direction, 0.05).len(), 0);
    }

    #[test]
    fn a_27k_object_index_matches_brute_force_and_stays_fast() {
        let side = 30usize;
        let mut centres = Vec::new();
        for x in 0..side { for y in 0..side { for z in 0..side {
            centres.push([x as f64, y as f64, z as f64]);
        }}}
        let radii = vec![0.4; centres.len()];
        let started = std::time::Instant::now();
        let index = SpatialIndex::build(&centres, &radii);
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        let origin = [-5.0, 14.5, 14.5];
        let direction = [1.0, 0.0, 0.0];
        let started = std::time::Instant::now();
        let mut candidates = index.ray_candidates(origin, direction);
        let query_ms = started.elapsed().as_secs_f64() * 1000.0;
        candidates.sort_unstable();
        let mut brute = Vec::new();
        for (i, centre) in centres.iter().enumerate() {
            let mut bounds = Aabb::empty();
            bounds.add_sphere(*centre, radii[i]);
            if bounds.hit(origin, direction) { brute.push(i); }
        }
        assert_eq!(candidates, brute);
        assert!(build_ms < 1000.0, "BVH build {build_ms:.1} ms");
        assert!(query_ms < 16.0, "BVH query {query_ms:.1} ms");
        eprintln!("BVH 27000 objects: build {build_ms:.1} ms, query {query_ms:.3} ms");
    }
}
