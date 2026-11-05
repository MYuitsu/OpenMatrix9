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
        Self {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
        }
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
            if delta == 0.0 {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return false;
                }
                continue;
            }
            let mut t0 = (self.min[axis] - origin[axis]) / delta;
            let mut t1 = (self.max[axis] - origin[axis]) / delta;
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
    sort_scratch: Vec<usize>,
}

impl SpatialIndex {
    fn node_capacity(objects: usize) -> om9_core::Result<usize> {
        if objects == 0 { return Ok(0); }
        objects.div_ceil(8).checked_next_power_of_two()
            .and_then(|leaves| leaves.checked_mul(2)).and_then(|nodes| nodes.checked_sub(1))
            .ok_or(om9_core::Error::Overflow("picking nodes"))
    }
    pub(crate) fn allocation_usage(objects: usize) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::budget::{Limits, Usage};
        if objects > u32::MAX as usize { return Err(om9_core::Error::Overflow("picking object index")); }
        let mut usage = Usage::array::<Node>(Self::node_capacity(objects)?)?;
        usage.add(Usage::array::<usize>(objects)?, &Limits { max_bytes: u64::MAX, ..Default::default() })?;
        usage.add(Usage::array::<usize>(objects)?, &Limits { max_bytes: u64::MAX, ..Default::default() })?;
        Ok(usage)
    }
    pub(crate) fn try_with_capacity(objects: usize) -> om9_core::Result<Self> {
        use om9_core::Error;
        if objects > u32::MAX as usize { return Err(Error::Overflow("picking object index")); }
        let mut index = Self { nodes: Vec::new(), order: Vec::new(), sort_scratch: Vec::new() };
        index.nodes.try_reserve_exact(Self::node_capacity(objects)?).map_err(|_| Error::Budget("picking nodes allocation"))?;
        index.order.try_reserve_exact(objects).map_err(|_| Error::Budget("picking order allocation"))?;
        index.sort_scratch.try_reserve_exact(objects).map_err(|_| Error::Budget("picking sort allocation"))?;
        index.sort_scratch.resize(objects, 0);
        Ok(index)
    }
    pub(crate) fn rebuild_from_spheres(&mut self, spheres: &[([f64; 3], f64)]) -> om9_core::Result<()> {
        self.rebuild_with(spheres.len(), |index| spheres[index])
    }
    fn rebuild_with(&mut self, n: usize, sphere: impl Fn(usize) -> ([f64; 3], f64)) -> om9_core::Result<()> {
        if self.nodes.capacity() < Self::node_capacity(n)? || self.order.capacity() < n || self.sort_scratch.len() < n {
            return Err(om9_core::Error::Budget("picking index capacity"));
        }
        self.nodes.clear();
        self.order.clear();
        self.order.extend(0..n);
        if n != 0 { build_node(&mut self.nodes, &mut self.order, &mut self.sort_scratch, &sphere, 0, n); }
        Ok(())
    }
    /// Conservative logical estimate for BVH build and one ray query. This is
    /// not an allocator-enforced bound or a bound on sort implementation scratch.
    pub fn estimated_build_and_query_usage(objects: usize) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::{Error, budget::{Limits, Usage}};
        let n = objects.max(1);
        let four_n = n.checked_mul(4).ok_or(Error::Overflow("picking index estimate"))?;
        let limits = Limits { max_bytes: u64::MAX, ..Default::default() };
        let mut usage = Usage::array::<Node>(four_n)?;
        // Ordering, ray candidates and conservative sorting/query workspace.
        usage.add(Usage::array::<usize>(four_n)?, &limits)?;
        usage.add(Usage::array::<u32>(four_n)?, &limits)?;
        Ok(usage)
    }
    /// Managed heap capacity retained by the picking index, excluding allocator overhead.
    pub fn storage_usage(&self) -> om9_core::Result<om9_core::budget::Usage> {
        let mut usage = om9_core::budget::Usage::array::<Node>(self.nodes.capacity())?;
        usage.add(
            om9_core::budget::Usage::array::<usize>(self.order.capacity())?,
            &om9_core::budget::Limits { max_bytes: u64::MAX, ..Default::default() },
        )?;
        usage.add(om9_core::budget::Usage::array::<usize>(self.sort_scratch.capacity())?,
            &om9_core::budget::Limits { max_bytes: u64::MAX, ..Default::default() })?;
        Ok(usage)
    }
    pub fn build(centres: &[[f64; 3]], radii: &[f64]) -> Self {
        Self::build_with(centres.len(), |index| (centres[index], radii[index]))
    }

    /// Build directly from a scene's already retained occurrence spheres.
    pub fn build_from_spheres(spheres: &[([f64; 3], f64)]) -> Self {
        Self::build_with(spheres.len(), |index| spheres[index])
    }

    fn build_with(n: usize, sphere: impl Fn(usize) -> ([f64; 3], f64)) -> Self {
        let mut index = Self::try_with_capacity(n).expect("picking index allocation");
        index.rebuild_with(n, sphere).expect("preallocated picking index");
        index
    }

    /// Object indices whose sphere may be hit by the ray, in arbitrary order.
    /// Conservative query for a pixel-tolerance pick: the ray is widened by `padding` in world units.
    pub fn ray_candidates_fat(
        &self,
        origin: [f64; 3],
        direction: [f64; 3],
        padding: f64,
    ) -> Vec<usize> {
        let mut out = Vec::new();
        if self.nodes.is_empty() {
            return out;
        }
        let mut stack = vec![0u32];
        while let Some(index) = stack.pop() {
            let node = self.nodes[index as usize];
            if !node
                .bounds
                .hit_with_padding(origin, direction, padding.max(0.0))
            {
                continue;
            }
            if node.count > 0 {
                out.extend(
                    self.order[node.start as usize..(node.start + node.count) as usize]
                        .iter()
                        .copied(),
                );
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
                out.extend(
                    self.order[node.start as usize..(node.start + node.count) as usize]
                        .iter()
                        .copied(),
                );
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
fn build_node<F: Fn(usize) -> ([f64; 3], f64)>(
    nodes: &mut Vec<Node>,
    order: &mut [usize],
    scratch: &mut [usize],
    sphere: &F,
    start: usize,
    end: usize,
) -> u32 {
    let mut bounds = Aabb::empty();
    for index in start..end {
        let object = order[index];
        let (centre, radius) = sphere(object);
        bounds.add_sphere(centre, radius);
    }
    let slot = nodes.len() as u32;
    nodes.push(Node {
        bounds,
        left: 0,
        right: 0,
        start: start as u32,
        count: (end - start) as u32,
    });
    if end - start <= 8 {
        return slot;
    }
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for index in start..end {
        let c = sphere(order[index]).0;
        for axis in 0..3 {
            lo[axis] = lo[axis].min(c[axis]);
            hi[axis] = hi[axis].max(c[axis]);
        }
    }
    let axis = (0..3)
        .max_by(|a, b| (hi[*a] - lo[*a]).total_cmp(&(hi[*b] - lo[*b])))
        .unwrap_or(0);
    if hi[axis] - lo[axis] < 1e-9 {
        return slot;
    }
    // Stable merge sorting preserves the previous tie order while using the
    // admitted reusable workspace instead of std's hidden sort allocation.
    stable_order_sort(&mut order[start..end], &mut scratch[start..end], sphere, axis);
    let mid = (start + end) / 2;
    let left = build_node(nodes, order, scratch, sphere, start, mid);
    let right = build_node(nodes, order, scratch, sphere, mid, end);
    nodes[slot as usize].left = left;
    nodes[slot as usize].right = right;
    nodes[slot as usize].count = 0;
    slot
}

fn stable_order_sort<F: Fn(usize) -> ([f64; 3], f64)>(order: &mut [usize], scratch: &mut [usize], sphere: &F, axis: usize) {
    let n = order.len();
    let mut width = 1usize;
    while width < n {
        for start in (0..n).step_by(width.saturating_mul(2)) {
            let mid = start.saturating_add(width).min(n);
            let end = mid.saturating_add(width).min(n);
            let (mut left, mut right) = (start, mid);
            for slot in &mut scratch[start..end] {
                if left < mid && (right == end || !sphere(order[left]).0[axis].total_cmp(&sphere(order[right]).0[axis]).is_gt()) {
                    *slot = order[left]; left += 1;
                } else { *slot = order[right]; right += 1; }
            }
        }
        order.copy_from_slice(&scratch[..n]);
        width = width.saturating_mul(2);
    }
}

#[cfg(test)]
mod tests {
    use super::{Aabb, SpatialIndex};

    #[test]
    fn reusable_stable_sort_matches_std_ties_and_node_capacity_bound() {
        for n in 0usize..512 {
            let sphere = |index: usize| ([(index % 11) as f64, (index % 7) as f64, 0.], 1.);
            let mut expected: Vec<_> = (0..n).rev().collect();
            let mut actual = expected.clone();
            expected.sort_by(|a, b| sphere(*a).0[0].total_cmp(&sphere(*b).0[0]));
            super::stable_order_sort(&mut actual, &mut vec![0; n], &sphere, 0);
            assert_eq!(actual, expected, "stable ties at {n}");
            let centres: Vec<_> = (0..n).map(|index| sphere(index).0).collect();
            let index = SpatialIndex::build(&centres, &vec![1.; n]);
            assert!(index.node_count() <= SpatialIndex::node_capacity(n).unwrap());
        }
    }

    #[test]
    fn picking_scratch_estimate_rejects_overflow() {
        assert!(SpatialIndex::estimated_build_and_query_usage(usize::MAX).is_err());
        assert!(SpatialIndex::estimated_build_and_query_usage(100).unwrap().bytes > 0);
    }

    #[test]
    fn direct_sphere_input_matches_separate_arrays() {
        let spheres: Vec<_> = (0..257).map(|i| {
            ([f64::from(i % 17), f64::from(i / 17), f64::from(i % 3)], 0.1 + f64::from(i % 7) * 0.1)
        }).collect();
        let centres: Vec<_> = spheres.iter().map(|s| s.0).collect();
        let radii: Vec<_> = spheres.iter().map(|s| s.1).collect();
        let original = SpatialIndex::build(&centres, &radii);
        let direct = SpatialIndex::build_from_spheres(&spheres);
        assert_eq!(original.node_count(), direct.node_count());
        for y in 0..17 {
            let origin = [-5., f64::from(y) + 0.2, 1.];
            assert_eq!(original.ray_candidates(origin, [1., 0., 0.]), direct.ray_candidates(origin, [1., 0., 0.]));
            assert_eq!(original.ray_candidates_fat(origin, [1., 0., 0.], 0.2), direct.ray_candidates_fat(origin, [1., 0., 0.], 0.2));
        }
    }

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
        for x in 0..side {
            for y in 0..side {
                for z in 0..side {
                    centres.push([x as f64, y as f64, z as f64]);
                }
            }
        }
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
            if bounds.hit(origin, direction) {
                brute.push(i);
            }
        }
        assert_eq!(candidates, brute);
        assert!(build_ms < 1000.0, "BVH build {build_ms:.1} ms");
        assert!(query_ms < 16.0, "BVH query {query_ms:.1} ms");
        eprintln!("BVH 27000 objects: build {build_ms:.1} ms, query {query_ms:.3} ms");
    }
}
