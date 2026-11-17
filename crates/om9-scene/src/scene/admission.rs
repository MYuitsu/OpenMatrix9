// SPDX-License-Identifier: LGPL-2.1-or-later
//! Borrowed scene expansion and heap peak admission before cache allocation.
use super::*;
use std::collections::BTreeSet;
type Key = (ObjectId, u64, u64);
pub(super) fn planning_scratch(document: &Document) -> Result<Usage> {
    let members = document.definitions.values().try_fold(document.entities.len(), |n, definition| {
        n.checked_add(definition.members.len()).ok_or(Error::Overflow("scene unique sources"))
    })?;
    let mut usage=Usage::btree::<Key, ()>(members)?;
    let max_payload=document.entities.values().chain(document.definitions.values().flat_map(|d|d.members.iter()))
        .filter_map(|e|e.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT))
        .map(|e|e.payload.len()).max().unwrap_or(0);
    if max_payload>0 {
        let extra=u64::try_from(max_payload).map_err(|_|Error::Overflow("trim planning scratch"))?
            .checked_mul(64).and_then(|n|n.checked_add(4096)).ok_or(Error::Overflow("trim planning scratch"))?;
        usage.bytes=usage.bytes.checked_add(extra)
            .ok_or(Error::Overflow("trim planning scratch"))?;
    }
    Ok(usage)
}
pub(super) fn planning_scratch_reusing(document: &Document, previous: Option<&Scene>) -> Result<Usage> {
    let mut usage = planning_scratch(document)?;
    if let Some(previous) = previous {
        usage.bytes = usage.bytes.checked_add(Usage::btree::<usize, ()>(previous.objects.len())?.bytes)
            .ok_or(Error::Overflow("reused scene planning scratch"))?;
        usage.bytes = usage.bytes.checked_add(Usage::array::<usize>(previous.objects.len())?.bytes)
            .ok_or(Error::Overflow("reused scene index scratch"))?;
    }
    Ok(usage)
}
pub(super) struct AllocationPlan {
    pub objects: usize,
    pub diagnostics: usize,
    pub peak_bytes: u64,
}
fn buffer_bound(entity: &Entity, limits: &Limits, representations: &BrepRepresentationStore) -> Result<Option<Usage>> {
    let geometry=&entity.geometry;
    let mut usage = Usage {
        bytes: std::mem::size_of::<super::BufferPayload>() as u64 + 16,
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
            if matches!(curve, Curve::Polycurve { .. }) {
                if has_nested_segments(curve) {
                    let count = match nested_display_count(curve, limits) {
                        Ok(count) => count,
                        Err(Error::Unsupported(_)) => return Ok(None),
                        Err(error) => return Err(error),
                    };
                    usage.add(Usage::array::<Point3>(count)?, limits)?;
                    usage.points = count as u64;
                    usage.check(limits)?;
                    return Ok(Some(usage));
                }
                let segments = match connected_polyline_segments(curve, limits) {
                    Ok(segments) => segments,
                    Err(Error::Unsupported(_)) => {
                        if let Curve::Polycurve{segments,reversed:false}=curve
                            && segments.iter().all(|s|matches!(s,Curve::Line{..})) {
                            let count=segments.len().checked_mul(2).ok_or(Error::Overflow("stroke display vertices"))?;
                            usage.add(Usage::array::<Point3>(count)?,limits)?;
                            usage.add(Usage::array::<[u32;2]>(segments.len())?,limits)?;
                            usage.points=count as u64;return Ok(Some(usage));
                        }
                        return Ok(None);
                    },
                    Err(error) => return Err(error),
                };
                let count = segments.iter().try_fold(1usize, |count, segment| {
                    count
                        .checked_add(polyline_display_steps(segment)?)
                        .ok_or(Error::Overflow("polyline display vertices"))
                })?;
                usage.add(Usage::array::<Point3>(count)?, limits)?;
                usage.points = count as u64;
                usage.check(limits)?;
                return Ok(Some(usage));
            }
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
        Geometry::Surface(surface) => {
            match super::trim_decomposition::decode(entity,limits) {
                Ok(Some(trim)) => {
                    let bytes=entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload.len();
                    match super::trim_decomposition::bound(&trim,bytes,limits) {
                        Ok(bound)=>usage.add(bound,limits)?,
                        Err(Error::Invalid(_) | Error::Unsupported(_))=>super::surface_buffer_usage(&mut usage,limits)?,
                        Err(error)=>return Err(error),
                    }
                }
                Ok(None) => { super::surface_buffer_usage_for(surface, true, &mut usage, limits)?; },
                Err(Error::Invalid(_) | Error::Unsupported(_))=>super::surface_buffer_usage(&mut usage,limits)?,
                Err(error)=>return Err(error),
            }
        }
        Geometry::Brep(shape) => {
            let d = representations.display_for(shape)?;
            usage.add(Usage::array::<Point3>(d.points.len())?, limits)?;
            usage.add(Usage::array::<([u32; 3], u32)>(d.triangles.len())?, limits)?;
            usage.add(Usage::array::<[u32; 2]>(d.segments.len())?, limits)?;
            usage.points = d.points.len() as u64;
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
        Geometry::NativeBrep(brep) => {
            let mesh = representations.get_native(brep)?.display();
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
    representations: &'a BrepRepresentationStore,
    document: &'a Document,
    limits: &'a Limits,
    usage: Usage,
    seen: BTreeSet<Key>,
    seen_reused: BTreeSet<usize>,
    objects: usize,
    diagnostics: usize,
    paths: usize,
    max_degree: usize,
    max_depth: usize,
    max_surface_basis: usize,
    previous: Option<&'a Scene>,
    previous_index: Vec<usize>,
}
struct PlannedPath<'a> {
    parent: Option<&'a PlannedPath<'a>>,
    member: ObjectId,
}
impl PlannedPath<'_> {
    fn matches(&self, path: &[ObjectId]) -> bool {
        let mut current = Some(self);
        let mut indices = path.iter().rev();
        while let Some(node) = current {
            if indices.next() != Some(&node.member) { return false; }
            current = node.parent;
        }
        indices.next().is_none()
    }
}
impl Planner<'_> {
    fn entity(&mut self, entity: &Entity, root: ObjectId, path: Option<&PlannedPath<'_>>, depth: usize) -> Result<()> {
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
                let child = PlannedPath { parent: path, member: member.id };
                self.entity(member, root, Some(&child), depth + 1)?;
            }
            return Ok(());
        }
        let key = (entity.id, entity.generation, entity.geometry_revision);
        let reused = match self.previous {
            Some(previous) => reusable_brep_wire_matching(previous, &self.previous_index, root, entity, self.representations,
                |candidate| match path { Some(path) => path.matches(candidate), None => candidate.is_empty() })?,
            None => None,
        };
        let binding = if let Some(old) = reused {
            Some(Usage { points: buffer_usage(old.buffers.as_ref())?.points, ..Usage::default() })
        } else if self.seen.contains(&key) {
            Some(Usage::default())
        } else {
            match buffer_bound(entity, self.limits, self.representations) {
                Err(Error::Unsupported(_)) => None,
                other => other?,
            }
        };
        if let Some(buffer) = binding {
            let first = if let Some(old) = reused {
                self.seen_reused.insert(Arc::as_ptr(&old.buffers.inner) as usize)
            } else {
                self.seen.insert((entity.id, entity.generation, entity.geometry_revision))
            };
            if first {
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
            if let Geometry::Surface(surface) = &entity.geometry {
                let basis = surface.size[0].checked_add(surface.size[1])
                    .ok_or(Error::Overflow("surface evaluator basis"))?;
                self.max_surface_basis = self.max_surface_basis.max(basis);
                if entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                    self.diagnostics=self.diagnostics.checked_add(1).ok_or(Error::Overflow("scene diagnostics"))?;
                }
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
pub(super) fn admit(document: &Document, limits: &Limits, held: Usage, representations: &BrepRepresentationStore) -> Result<AllocationPlan> {
    admit_reusing(document, limits, held, representations, None)
}
pub(super) fn admit_reusing(document: &Document, limits: &Limits, held: Usage,
    representations: &BrepRepresentationStore, previous: Option<&Scene>) -> Result<AllocationPlan> {
    let members = document
        .definitions
        .values()
        .try_fold(document.entities.len(), |n, d| {
            n.checked_add(d.members.len())
                .ok_or(Error::Overflow("scene unique sources"))
        })?;
    let mut usage = held;
    // Document point complexity and derived display-point complexity each
    // have their own cap; their bytes still coexist and remain aggregated.
    usage.points = 0;
    // The preflight seen set and actual capture's shared cache map have
    // separately admitted lifetimes; conservatively reserve both here.
    usage.add(planning_scratch_reusing(document, previous)?, limits)?;
    usage.add(Usage::btree::<Key, SharedBuffer>(members)?, limits)?;
    usage.add(Usage::btree::<LayerId, ()>(document.layers.len())?, limits)?;
    let mut planner = Planner {
        representations,
        document,
        limits,
        usage,
        seen: BTreeSet::new(),
        seen_reused: BTreeSet::new(),
        objects: 0,
        diagnostics: 0,
        paths: 0,
        max_degree: 0,
        max_depth: 0,
        max_surface_basis: 0,
        previous,
        previous_index: previous_occurrence_index(previous)?,
    };
    for entity in document.entities.values() {
        planner.entity(entity, entity.id, None, 0)?;
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
        Usage::array::<Mutex<super::SceneMemory>>(1)?,
        Usage::array::<usize>(2)?,
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
        Usage::array::<f64>(planner.max_surface_basis)?,
    ] {
        usage.add(allocation, limits)?;
    }
    Ok(AllocationPlan {
        objects: planner.objects,
        diagnostics: planner.diagnostics,
        peak_bytes: usage.bytes.checked_sub(held.bytes).ok_or(Error::Overflow("scene preparation bytes"))?,
    })
}
