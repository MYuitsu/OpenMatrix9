// SPDX-License-Identifier: LGPL-2.1-or-later
//! Conservative BVH: traversal only rejects disjoint bounds; predicates stay in
//! the owning kernel. Indices are stable and query output order is deterministic.
use super::{Budget, Point3, Result};

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub lo: Point3,
    pub hi: Point3,
}
impl Bounds {
    pub fn of(points: impl IntoIterator<Item = Point3>) -> Self {
        let mut bounds = Self {
            lo: [f64::INFINITY; 3],
            hi: [f64::NEG_INFINITY; 3],
        };
        for p in points {
            for (axis, value) in p.into_iter().enumerate() {
                bounds.lo[axis] = bounds.lo[axis].min(value);
                bounds.hi[axis] = bounds.hi[axis].max(value);
            }
        }
        bounds
    }
    pub fn expanded(self, eps: f64) -> Self {
        Self {
            lo: self.lo.map(|x| x - eps),
            hi: self.hi.map(|x| x + eps),
        }
    }
    fn overlaps(self, other: Self) -> bool {
        (0..3).all(|i| self.lo[i] <= other.hi[i] && other.lo[i] <= self.hi[i])
    }
    fn ray(self, p: Point3, direction: Point3) -> bool {
        let mut lo = 0.0_f64;
        let mut hi = f64::INFINITY;
        for axis in 0..3 {
            let a = (self.lo[axis] - p[axis]) / direction[axis];
            let b = (self.hi[axis] - p[axis]) / direction[axis];
            lo = lo.max(a.min(b));
            hi = hi.min(a.max(b));
            if lo > hi {
                return false;
            }
        }
        true
    }
}
struct Node {
    bounds: Bounds,
    start: usize,
    end: usize,
    children: Option<[usize; 2]>,
}
pub(super) struct Index {
    boxes: Vec<Bounds>,
    ids: Vec<usize>,
    nodes: Vec<Node>,
}
impl Index {
    pub fn new(boxes: Vec<Bounds>, budget: &mut Budget) -> Result<Self> {
        let len = boxes.len();
        let mut index = Self {
            boxes,
            ids: (0..len).collect(),
            nodes: Vec::new(),
        };
        if len > 0 {
            index.build(0, len, budget)?;
        }
        Ok(index)
    }
    fn build(&mut self, start: usize, end: usize, budget: &mut Budget) -> Result<usize> {
        budget.spend(end - start)?;
        let bounds = Bounds::of(
            self.ids[start..end]
                .iter()
                .flat_map(|&i| [self.boxes[i].lo, self.boxes[i].hi]),
        );
        let node = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            start,
            end,
            children: None,
        });
        if end - start > 8 {
            let axis = (0..3)
                .max_by(|&a, &b| {
                    (bounds.hi[a] - bounds.lo[a]).total_cmp(&(bounds.hi[b] - bounds.lo[b]))
                })
                .unwrap();
            let middle = (end - start) / 2;
            self.ids[start..end].select_nth_unstable_by(middle, |&a, &b| {
                (self.boxes[a].lo[axis] + self.boxes[a].hi[axis])
                    .total_cmp(&(self.boxes[b].lo[axis] + self.boxes[b].hi[axis]))
                    .then(a.cmp(&b))
            });
            let mid = start + middle;
            let left = self.build(start, mid, budget)?;
            let right = self.build(mid, end, budget)?;
            self.nodes[node].children = Some([left, right]);
        }
        Ok(node)
    }
    fn query(&self, accepts: impl Fn(Bounds) -> bool, budget: &mut Budget) -> Result<Vec<usize>> {
        let mut out = Vec::new();
        let mut stack = if self.nodes.is_empty() {
            vec![]
        } else {
            vec![0]
        };
        while let Some(id) = stack.pop() {
            budget.spend(1)?;
            let node = &self.nodes[id];
            if !accepts(node.bounds) {
                continue;
            }
            if let Some([a, b]) = node.children {
                stack.extend([b, a]);
            } else {
                for &i in &self.ids[node.start..node.end] {
                    budget.spend(1)?;
                    if accepts(self.boxes[i]) {
                        out.push(i);
                    }
                }
            }
        }
        out.sort_unstable();
        Ok(out)
    }
    pub fn overlapping(&self, bounds: Bounds, budget: &mut Budget) -> Result<Vec<usize>> {
        self.query(|b| b.overlaps(bounds), budget)
    }
    pub fn along_ray(&self, p: Point3, d: Point3, budget: &mut Budget) -> Result<Vec<usize>> {
        self.query(|b| b.ray(p, d), budget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_candidates_match_exhaustive_queries() {
        let boxes: Vec<_> = (0..137)
            .map(|i| {
                let x = (i * 31 % 137) as f64 / 10.;
                let y = (i * 17 % 89) as f64 / 10.;
                Bounds {
                    lo: [x, y, -0.2],
                    hi: [x + 0.3, y + 0.7, 0.2],
                }
            })
            .collect();
        let mut budget = Budget::new(1_000_000);
        let index = Index::new(boxes.clone(), &mut budget).unwrap();
        for k in 0..31 {
            let query = Bounds {
                lo: [k as f64 * 0.3, 1., -1.],
                hi: [k as f64 * 0.3 + 2., 4., 1.],
            };
            let expected: Vec<_> = boxes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.overlaps(query))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(index.overlapping(query, &mut budget).unwrap(), expected);
            let p = [k as f64 * 0.3, 0., -1.];
            let d = [0.31, 0.43, 1.];
            let expected: Vec<_> = boxes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.ray(p, d))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(index.along_ray(p, d, &mut budget).unwrap(), expected);
        }
    }
}
