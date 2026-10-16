// SPDX-License-Identifier: LGPL-2.1-or-later
//! Borrowed scene expansion and heap peak admission before cache allocation.
use super::*;
use std::collections::BTreeSet;
type Key = (ObjectId, u64, u64);
pub(super) struct AllocationPlan {
    pub objects: usize,
    pub diagnostics: usize,
}
fn buffer_bound(geometry: &Geometry, limits: &Limits) -> Result<Option<Usage>> {
    let mut usage = Usage {
        bytes: std::mem::size_of::<RenderBuffers>() as u64 + 16,
        ..Usage::default()
    };
    match geometry {
        Geometry::Point(_) => {
            usage.add(Usage::array::<Point3>(1)?, limits)?;
            usage.add(Usage::array::<u32>(1)?, limits)?;
            usage.points = 1;
        }
        Geometry::Cloud(cloud) => {
            let visible = cloud
                .points
                .iter()
                .enumerate()
                .filter(|(i, _)| !cloud.hidden.get(*i).copied().unwrap_or(false))
                .count();
            usage.add(Usage::array::<Point3>(visible)?, limits)?;
            usage.add(Usage::array::<u32>(visible)?, limits)?;
            usage.points = cloud.points.len() as u64;
        }
        Geometry::Curve(curve) => {
            let segments = match curve {
                Curve::Line { .. } => 1,
                Curve::Nurbs(n) if !n.periodic => 256,
                Curve::Circle {
                    radius,
                    domain,
                    angle_domain,
                    ..
                } => {
                    let angle = 2. * (1. - (0.005 / radius).min(1.)).acos();
                    let angles = angle_domain.unwrap_or(*domain);
                    let estimate = ((angles[1] - angles[0]) / angle).ceil().max(16.);
                    if !estimate.is_finite() || estimate > 65535. {
                        return Err(Error::Budget("circle display chord samples"));
                    }
                    estimate as usize
                }
                _ => return Ok(None),
            };
            let count = segments + 1;
            usage.add(Usage::array::<Point3>(count)?, limits)?;
            usage.add(Usage::array::<f64>(count)?, limits)?;
            usage.points = count as u64;
        }
        Geometry::Mesh(mesh) => {
            let mut triangles = 0usize;
            let mut edges = 0usize;
            for face in &mesh.faces {
                triangles = triangles
                    .checked_add(face.len() - 2)
                    .ok_or(Error::Overflow("scene triangles"))?;
                edges = edges
                    .checked_add(face.len())
                    .ok_or(Error::Overflow("scene edges"))?;
            }
            usage.add(Usage::array::<Point3>(mesh.vertices.len())?, limits)?;
            usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, limits)?;
            usage.add(Usage::array::<[u32; 2]>(edges)?, limits)?;
            usage.points = mesh.vertices.len() as u64;
        }
        _ => return Ok(None),
    }
    usage.check(limits)?;
    Ok(Some(usage))
}
struct Planner<'a> {
    document: &'a Document,
    limits: &'a Limits,
    usage: Usage,
    seen: BTreeSet<Key>,
    objects: usize,
    diagnostics: usize,
    paths: usize,
    max_degree: usize,
    max_depth: usize,
}
impl Planner<'_> {
    fn entity(&mut self, entity: &Entity, depth: usize) -> Result<()> {
        if depth > self.limits.max_depth {
            return Err(Error::Budget("scene instance depth"));
        }
        // Instance occurrences, including empty definitions, consume traversal
        // work independently of the shared catalog and drawable leaf count.
        self.usage.add(
            Usage {
                nodes: 1,
                ..Usage::default()
            },
            self.limits,
        )?;
        self.max_depth = self.max_depth.max(depth);
        if let Geometry::Instance(id) = entity.geometry {
            let definition = self
                .document
                .definitions
                .get(&id)
                .ok_or(Error::Missing("scene definition"))?;
            for member in &definition.members {
                self.entity(member, depth + 1)?;
            }
            return Ok(());
        }
        let key = (entity.id, entity.generation, entity.geometry_revision);
        let binding = if self.seen.contains(&key) {
            Some(Usage::default())
        } else {
            buffer_bound(&entity.geometry, self.limits)?
        };
        if let Some(buffer) = binding {
            if self
                .seen
                .insert((entity.id, entity.generation, entity.geometry_revision))
            {
                self.usage.add(buffer, self.limits)?;
            }
            self.objects = self
                .objects
                .checked_add(1)
                .ok_or(Error::Overflow("scene objects"))?;
            let capacity = if depth == 0 {
                0
            } else {
                depth
                    .checked_mul(2)
                    .ok_or(Error::Overflow("scene path capacity"))?
                    .max(4)
            };
            self.paths = self
                .paths
                .checked_add(capacity)
                .ok_or(Error::Overflow("scene paths"))?;
            if let Geometry::Curve(Curve::Nurbs(n)) = &entity.geometry {
                self.max_degree = self.max_degree.max(n.degree);
            }
            if matches!(
                &entity.geometry,
                Geometry::Curve(Curve::Circle {
                    angle_domain: Some(_),
                    ..
                })
            ) {
                self.diagnostics = self
                    .diagnostics
                    .checked_add(1)
                    .ok_or(Error::Overflow("scene diagnostics"))?;
            }
        } else {
            self.diagnostics = self
                .diagnostics
                .checked_add(1)
                .ok_or(Error::Overflow("scene diagnostics"))?;
        }
        Ok(())
    }
}
pub(super) fn admit(document: &Document, limits: &Limits, held: Usage) -> Result<AllocationPlan> {
    let members = document
        .definitions
        .values()
        .try_fold(document.entities.len(), |n, d| {
            n.checked_add(d.members.len())
                .ok_or(Error::Overflow("scene unique sources"))
        })?;
    let mut usage = held;
    // The preflight seen set and actual capture's shared cache map have
    // separately admitted lifetimes; conservatively reserve both here.
    usage.add(Usage::btree::<Key, ()>(members)?, limits)?;
    usage.add(Usage::btree::<Key, Arc<RenderBuffers>>(members)?, limits)?;
    usage.add(Usage::btree::<LayerId, ()>(document.layers.len())?, limits)?;
    let mut planner = Planner {
        document,
        limits,
        usage,
        seen: BTreeSet::new(),
        objects: 0,
        diagnostics: 0,
        paths: 0,
        max_degree: 0,
        max_depth: 0,
    };
    for entity in document.entities.values() {
        planner.entity(entity, 0)?;
    }
    let mut usage = planner.usage;
    let path_scratch = planner
        .max_depth
        .checked_add(1)
        .and_then(|n| n.checked_mul(n))
        .and_then(|n| n.checked_mul(4))
        .ok_or(Error::Overflow("scene path scratch"))?;
    usage.add(Usage::array::<ObjectId>(path_scratch)?, limits)?;
    for allocation in [
        Usage::array::<Scene>(1)?,
        Usage::array::<SceneObject>(planner.objects)?,
        Usage::array::<ObjectId>(planner.paths)?,
        Usage::array::<String>(planner.diagnostics)?,
        Usage {
            bytes: (planner.diagnostics as u64)
                .checked_mul(384)
                .ok_or(Error::Overflow("scene diagnostic strings"))?,
            ..Usage::default()
        },
        Usage::array::<[[f64; 4]; 4]>(
            planner
                .max_degree
                .checked_add(1)
                .ok_or(Error::Overflow("scene evaluator degree"))?,
        )?,
        Usage::array::<Point3>(8)?,
    ] {
        usage.add(allocation, limits)?;
    }
    Ok(AllocationPlan {
        objects: planner.objects,
        diagnostics: planner.diagnostics,
    })
}
