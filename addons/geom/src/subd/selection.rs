// SPDX-License-Identifier: LGPL-2.1-or-later
//! Neighborhood selection on the document's polygon cage, before tessellation.
use super::{Edge, polygon_edge_faces, vertex_edges, visit_edge_loop, visit_quad_ring};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component {
    Vertex(u32),
    Edge(Edge),
    Face(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Loop,
    Ring,
    Grow,
    Shrink,
    Step(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    NextU,
    PrevU,
    NextV,
    PrevV,
}

/// Members of an incidence group are immediate neighbors. Read the original
/// selection throughout so one invocation moves the boundary by exactly one step.
fn resize_group(
    group: impl Iterator<Item = Component> + Clone,
    selected: &BTreeSet<Component>,
    output: &mut BTreeSet<Component>,
    grow: bool,
) {
    if grow {
        if group.clone().any(|key| selected.contains(&key)) {
            output.extend(group);
        }
    } else if group.clone().any(|key| !selected.contains(&key)) {
        for key in group {
            output.remove(&key);
        }
    }
}

/// Box cages keep each authored quad in [u0v0, u1v0, u1v1, u0v1] order,
/// including after extracting faces and compacting their vertex IDs.
fn step_components(
    faces: &[Vec<u32>],
    selected: &BTreeSet<Component>,
    direction: Direction,
) -> Result<BTreeSet<Component>, &'static str> {
    use Component::{Edge as Segment, Face, Vertex};
    let incidence = polygon_edge_faces(faces);
    let (side, steps) = match direction {
        Direction::NextU => (1, [(0, 1), (3, 2)]),
        Direction::PrevU => (3, [(1, 0), (2, 3)]),
        Direction::NextV => (2, [(0, 3), (1, 2)]),
        Direction::PrevV => (0, [(3, 0), (2, 1)]),
    };
    let mut next_vertex = BTreeMap::<u32, Option<u32>>::new();
    if !matches!(selected.first(), Some(Face(_))) {
        for face in faces {
            if face.len() != 4 {
                continue;
            }
            for (from, to) in steps {
                let neighbor = face[to];
                next_vertex
                    .entry(face[from])
                    .and_modify(|v| {
                        if *v != Some(neighbor) {
                            *v = None;
                        }
                    })
                    .or_insert(Some(neighbor));
            }
        }
    }
    let neighbor = |v| next_vertex.get(&v).copied().flatten();
    let mut result = BTreeSet::new();
    for &key in selected {
        let next = match key {
            Face(f) => {
                let face = &faces[f];
                if face.len() != 4 {
                    return Err("Directional face selection requires quadrilaterals");
                }
                let side_edge = super::edge(face[side], face[(side + 1) % 4]);
                let adjacent = &incidence[&side_edge];
                if adjacent.len() > 2 {
                    return Err("Ambiguous cage direction at this edge");
                }
                adjacent.iter().copied().find(|i| *i != f).map(Face)
            }
            Vertex(v) => neighbor(v).map(Vertex),
            Segment((a, b)) => neighbor(a).zip(neighbor(b)).and_then(|(c, d)| {
                let target = super::edge(c, d);
                let adjacent = incidence.get(&target)?;
                // A step along an edge shares a vertex. A transverse step crosses
                // one source quad; another nearby strip across a hole is unrelated.
                let connected =
                    a == d || b == c || incidence[&(a, b)].iter().any(|f| adjacent.contains(f));
                connected.then_some(Segment(target))
            }),
        };
        result.insert(next.unwrap_or(key));
    }
    Ok(result)
}

/// The caller supplies validated keys of a single kind on one owner.
pub fn select(
    faces: &[Vec<u32>],
    selected: &BTreeSet<Component>,
    operation: Operation,
) -> Result<BTreeSet<Component>, &'static str> {
    use Component::{Edge as Segment, Face, Vertex};
    let first = *selected.first().ok_or("Select T-Spline components")?;
    if let Operation::Step(direction) = operation {
        return step_components(faces, selected, direction);
    }
    let incidence = polygon_edge_faces(faces);
    let mut output = selected.clone();
    if matches!(operation, Operation::Grow | Operation::Shrink) {
        let grow = operation == Operation::Grow;
        match first {
            Vertex(_) => {
                for &(a, b) in incidence.keys() {
                    resize_group(
                        [Vertex(a), Vertex(b)].into_iter(),
                        selected,
                        &mut output,
                        grow,
                    );
                }
            }
            Segment(_) => {
                for edges in vertex_edges(&incidence).values() {
                    resize_group(
                        edges.iter().copied().map(Segment),
                        selected,
                        &mut output,
                        grow,
                    );
                }
            }
            Face(_) => {
                let mut at_vertex = BTreeMap::<u32, Vec<usize>>::new();
                for (id, face) in faces.iter().enumerate() {
                    for &v in face {
                        at_vertex.entry(v).or_default().push(id);
                    }
                }
                for adjacent in at_vertex.values() {
                    resize_group(
                        adjacent.iter().copied().map(Face),
                        selected,
                        &mut output,
                        grow,
                    );
                }
            }
        }
        return Ok(output);
    }
    let mut covered = BTreeSet::new();
    if operation == Operation::Ring || matches!(first, Face(_)) {
        for (&segment, adjacent) in &incidence {
            let seed = match (operation, first) {
                (Operation::Ring, Segment(_)) => selected.contains(&Segment(segment)),
                (Operation::Loop, Face(_)) => {
                    adjacent.len() == 2 && adjacent.iter().all(|f| selected.contains(&Face(*f)))
                }
                _ => return Err("Select T-Spline edges for Ring Select"),
            };
            if !seed || !covered.insert(segment) {
                continue;
            }
            visit_quad_ring(faces, &incidence, segment, |face, opposite| {
                covered.extend(opposite);
                if operation == Operation::Loop {
                    output.insert(Face(face));
                } else {
                    output.extend(opposite.map(Segment));
                }
            });
        }
    } else {
        let at_vertex = vertex_edges(&incidence);
        for &segment in incidence.keys() {
            let seed = match first {
                Vertex(_) => {
                    selected.contains(&Vertex(segment.0)) && selected.contains(&Vertex(segment.1))
                }
                _ => selected.contains(&Segment(segment)),
            };
            if !seed || covered.contains(&segment) {
                continue;
            }
            visit_edge_loop(&incidence, &at_vertex, segment, |next| {
                covered.insert(next);
                if matches!(first, Vertex(_)) {
                    output.extend([Vertex(next.0), Vertex(next.1)]);
                } else {
                    output.insert(Segment(next));
                }
            });
        }
    }
    if covered.is_empty() {
        return Err("Select an edge, two adjacent vertices or two adjacent faces");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Component::{Edge as Segment, Face};

    #[test]
    fn a_uv_step_cannot_jump_across_a_missing_quad() {
        // 3x3 grid with the center face removed. Both endpoints of the
        // near edge have a +U neighbor, but the strip between them is empty.
        let faces: Vec<_> = (0..3)
            .flat_map(|x| {
                (0..3).filter_map(move |y| {
                    let i = 4 * x + y;
                    (i != 5).then(|| vec![i, i + 4, i + 5, i + 1])
                })
            })
            .collect();
        for seed in [Segment((5, 6)), Face(1)] {
            let keys = BTreeSet::from([seed]);
            assert_eq!(
                select(&faces, &keys, Operation::Step(Direction::NextU)).unwrap(),
                keys
            );
        }
        let keys = BTreeSet::from([Face(0), Face(2)]);
        assert_eq!(
            select(&faces, &keys, Operation::Step(Direction::NextU)).unwrap(),
            BTreeSet::from([Face(3), Face(4)])
        );
    }
}
