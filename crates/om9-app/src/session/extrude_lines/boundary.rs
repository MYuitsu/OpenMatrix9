// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_core::{brep::CanonicalBrep, curve_ops, geometry::NurbsCurve};
use om9_kernel::{
    brep, creation::DEFAULT_DEFLECTION_MM, editing, extrusion::boundary::BoundaryProjection,
    measurement, transform_ops,
};
use std::collections::BTreeSet;

fn kernel(error: impl std::fmt::Display) -> Error {
    Error::Io(format!("Extrude Lines: {error}"))
}

pub(in crate::session) fn boundary_face(c: &Controller) -> Result<CanonicalBrep> {
    let selection = c.selected_ordered();
    let [id] = selection.as_slice() else {
        return Err(Error::Invalid("Select one boundary surface"));
    };
    let entity = &c.document().entities[id];
    let limits = c.coordinator.limits();
    let (face, world) = match &entity.geometry {
        Geometry::Surface(surface) => {
            let trim = om9_core::rhino_semantics::trimmed_surface_of(entity, limits)?;
            let (source, world) = if let Some(trim) = trim {
                (
                    measurement::trimmed_surface_to_face_source_admitted(&trim, limits, |_, _| {
                        Ok(())
                    })
                    .map_err(kernel)?,
                    entity.placement,
                )
            } else {
                (
                    measurement::surface_to_face_source(surface, entity.placement, limits)
                        .map_err(kernel)?,
                    Transform::identity(),
                )
            };
            let face = brep::read_brep(&source, DEFAULT_DEFLECTION_MM, limits).map_err(kernel)?;
            (face.into_canonical_and_representation().0, world)
        }
        Geometry::NativeBrep(native) => {
            let tolerance = c.document().effective_tolerance()?.absolute_mm;
            let body = brep::from_3dm(&native.payload, tolerance, DEFAULT_DEFLECTION_MM, limits)
                .map_err(kernel)?;
            let face = if body.topology().faces.len() == 1 {
                body
            } else {
                c.extract_picked_native_boundary(&body, entity)?
            };
            (face.into_canonical_and_representation().0, entity.placement)
        }
        Geometry::Brep(body) if body.topology().faces.len() == 1 => {
            (body.clone(), entity.placement)
        }
        Geometry::Brep(body) => {
            let [reference] = c.subselection() else {
                return Err(Error::Invalid("Pick one face on the boundary polysurface"));
            };
            let om9_scene::scene::Subelement::BrepFace(key) = reference.subset else {
                return Err(Error::Invalid("Select a boundary face"));
            };
            if reference.root != *id || !reference.path.is_empty() {
                return Err(Error::Invalid("Select a document boundary face"));
            }
            let index = body
                .topology()
                .faces
                .iter()
                .position(|id| *id == key)
                .ok_or(Error::Stale("boundary face"))?;
            let triangle = c.solid_face_display_triangle(reference)?;
            let scene = c.scene().ok_or(Error::Missing("boundary scene"))?;
            let witness = scene
                .root_brep_pick_display(&reference.source)?
                .deflection_mm;
            let tolerance = c.document().effective_tolerance()?.absolute_mm.min(1e-6);
            let face = editing::extract_verified_face_with_witness_deflection(
                body,
                index,
                triangle,
                tolerance,
                witness,
                DEFAULT_DEFLECTION_MM,
                limits,
            )
            .map_err(kernel)?;
            (face.into_canonical_and_representation().0, entity.placement)
        }
        _ => return Err(Error::Invalid("Select a surface or polysurface face")),
    };
    if world == Transform::identity() {
        return Ok(face);
    }
    Ok(
        transform_ops::transform_brep(&face, &world, DEFAULT_DEFLECTION_MM, limits)
            .map_err(kernel)?
            .into_canonical_and_representation()
            .0,
    )
}

fn line(start: Point3, end: Point3) -> Curve {
    Curve::Line {
        start,
        end,
        domain: [0., 1.],
    }
}
type EdgeKey = [[u64; 3]; 2];
fn edge_key(start: Point3, end: Point3) -> EdgeKey {
    let mut bits = [start, end].map(|p| p.map(|v| if v == 0. { 0 } else { v.to_bits() }));
    bits.sort_unstable();
    bits
}

#[derive(Default)]
struct Wires {
    changes: Vec<Mutation>,
    ids: Vec<ObjectId>,
    lines: BTreeSet<EdgeKey>,
    usage: Usage,
}
impl Wires {
    fn add(&mut self, c: &Controller, curve: Curve, placement: Transform) -> Result<()> {
        let endpoints = match &curve {
            Curve::Line { start, end, .. } => Some([*start, *end]),
            Curve::Nurbs(NurbsCurve {
                degree: 1, poles, ..
            }) if poles.len() == 2 => {
                Some(curve_ops::curve_endpoints(&curve, c.coordinator.limits())?)
            }
            _ => None,
        };
        let (curve, placement) = if let Some([a, b]) = endpoints {
            let a = placement.point(a)?;
            let b = placement.point(b)?;
            if !self.lines.insert(edge_key(a, b)) {
                return Ok(());
            }
            (line(a, b), Transform::identity())
        } else {
            (curve, placement)
        };
        let limits = c.coordinator.limits();
        let cost = curve.validate(limits)?;
        self.usage.add(cost, limits)?;
        self.usage.add(Usage::array::<Mutation>(1)?, limits)?;
        if self.ids.len() as u64 >= limits.max_outputs {
            return Err(Error::Budget("Extrude Lines output count"));
        }
        let mut entity = Entity::new(LABEL, Geometry::Curve(curve), c.document().current_layer);
        entity.placement = placement;
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        self.ids.push(entity.id);
        self.changes.push(Mutation::Insert {
            document: c.document,
            entity,
        });
        Ok(())
    }
}

impl ExtrudeLines {
    pub(super) fn finish_boundary(&self, c: &mut Controller) -> Result<Reply> {
        let limits = *c.coordinator.limits();
        let tol = c.document().effective_tolerance()?.absolute_mm / c.document().page_unit.factor();
        let target = boundary_face(c)?;
        let available = limits.remaining_after(target.storage_usage(&limits)?)?;
        let projector = BoundaryProjection::new(
            target,
            self.axis,
            self.profiles.start,
            tol,
            DEFAULT_DEFLECTION_MM,
            &available,
        )?;
        let mut wires = Wires::default();
        let mut solid_ids = Vec::new();
        let mut processed = BTreeSet::new();
        for id in &self.ids {
            let source = &c.document().entities[id];
            let Geometry::Curve(curve) = &source.geometry else {
                return Err(Error::Stale("Extrude Lines source"));
            };
            let world = source.placement;
            let remaining = available.remaining_after(wires.usage)?;
            if self.solid
                && curve_ops::is_closed(curve, 1e-7, &remaining)?
                && curve_ops::extrude_lines::profile_plane([(curve, world)], &remaining)?.is_some()
            {
                solid_ids.push(*id);
                continue;
            }
            let shape =
                ControlProfiles::capture([(curve, world)], tol, &remaining, &mut || Ok(()))?;
            if let Some(rail) = shape.curved.first() {
                let mut sections = Vec::new();
                let mut held = Usage::default();
                for part in curve_ops::extrude_lines::profile_segments(curve, &remaining)? {
                    let room = remaining.remaining_after(held)?;
                    let projected = projector.rail(&part, world, &room)?;
                    held.add(projected.validate(&room)?, &remaining)?;
                    sections.push(projected);
                }
                wires.add(
                    c,
                    curve_ops::join_curves(&sections, tol, &remaining)?,
                    Transform::identity(),
                )?;
                for &p in &rail.stations {
                    let q = projector.project_station(p, &remaining)?;
                    wires.add(c, line(p, q), Transform::identity())?;
                }
                if self.delete {
                    wires.add(c, curve.clone(), world)?;
                }
            } else {
                for [a, b] in shape.source_wires() {
                    if !processed.insert(edge_key(a, b)) {
                        continue;
                    }
                    let room = available.remaining_after(wires.usage)?;
                    let section = projector.rail(&line(a, b), Transform::identity(), &room)?;
                    let [p, q] = curve_ops::curve_endpoints(&section, &room)?;
                    for (from, to) in [(a, p), (b, q)] {
                        wires.add(c, line(from, to), Transform::identity())?;
                    }
                    wires.add(c, section, Transform::identity())?;
                    if self.delete {
                        wires.add(c, line(a, b), Transform::identity())?;
                    }
                }
            }
        }
        let count = wires
            .ids
            .len()
            .checked_add(solid_ids.len())
            .ok_or(Error::Overflow("Extrude Lines outputs"))?;
        if count as u64 > limits.max_outputs {
            return Err(Error::Budget("Extrude Lines outputs"));
        }
        if self.delete {
            wires
                .usage
                .add(Usage::array::<Mutation>(self.ids.len())?, &limits)?;
            for id in &self.ids {
                wires.changes.push(Mutation::Delete {
                    source: c.document().stamp(*id)?,
                });
            }
        }
        if solid_ids.is_empty() {
            c.publish_request_with_selection(
                TransactionRequest::new(LABEL, wires.changes),
                Some(&wires.ids),
            )?;
        } else {
            c.publish_extrude_solids(
                (CODE, LABEL),
                &solid_ids,
                wires.changes,
                wires.ids,
                wires.usage,
                |c, id, room| {
                    let entity = &c.document().entities[id];
                    let Geometry::Curve(curve) = &entity.geometry else {
                        return Err(Error::Stale("Extrude Lines source"));
                    };
                    let profile = self.solid_profile(c, curve, entity.placement, room)?;
                    Ok((
                        projector.solid(&profile, entity.placement, room)?,
                        entity.placement,
                    ))
                },
            )?;
        }
        c.extrude_lines_defaults.both = false;
        c.extrude_lines_defaults.delete = self.delete;
        c.extrude_lines_defaults.solid = self.solid;
        c.extrude_lines_defaults.split_at_tangents = self.split_at_tangents;
        Ok(Reply {
            done: true,
            state: serde_json::json!(count),
            ..Default::default()
        })
    }
}
