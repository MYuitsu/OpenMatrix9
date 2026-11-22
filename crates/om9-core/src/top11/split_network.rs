// SPDX-License-Identifier: LGPL-2.1-or-later
//! Collect both parameters of each intersection before changing any member of a curve network.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    curve_ops, curve_surface_ops,
    geometry::{Curve, Geometry, Point3},
    identity::ObjectId,
    model::Entity,
    utilities::{Bounds, bounds},
};

pub struct SplitMember {
    pub source: ObjectId,
    pub pieces: Vec<Curve>,
    /// Distinct actual cut positions on this member, in world coordinates.
    pub points: Vec<Point3>,
}

struct Member<'a> {
    source: &'a Entity,
    local: &'a Curve,
    world: Curve,
    bounds: Bounds,
    fractions: Vec<f64>,
}

// Baking a sheared or nonuniformly scaled arc uses rational quadratic spans.
// Their Bézier parameter is not the original angular fraction. Recover that
// angle inside the corresponding span before cutting the local source curve.
fn source_fraction(local: &Curve, world: &Curve, t: f64) -> f64 {
    match (local, world) {
        (
            Curve::Circle {
                domain,
                angle_domain,
                ..
            },
            Curve::Nurbs(n),
        ) => {
            let [start, end] = angle_domain.unwrap_or(*domain);
            let spans = (n.poles.len() - 1) / 2;
            let scaled = t * spans as f64;
            let index = (scaled as usize).min(spans - 1);
            let u = scaled - index as f64;
            let half = (end - start) / (2. * spans as f64);
            let angle = (u * half.sin()).atan2(1. - u + u * half.cos());
            (index as f64 + angle / half) / spans as f64
        }
        (
            Curve::Polycurve { segments, reversed },
            Curve::Polycurve {
                segments: placed, ..
            },
        ) => {
            let n = segments.len();
            let scaled = if *reversed { 1. - t } else { t } * n as f64;
            let index = (scaled as usize).min(n - 1);
            let u = (index as f64
                + source_fraction(&segments[index], &placed[index], scaled - index as f64))
                / n as f64;
            if *reversed { 1. - u } else { u }
        }
        _ => t,
    }
    .clamp(0., 1.)
}

/// Tolerance and geometry share document model units. Curves in the result
/// retain their own frame. A member with no interior cuts is omitted.
pub fn split<'a>(
    entities: impl IntoIterator<Item = &'a Entity>,
    tolerance: f64,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
) -> Result<Vec<SplitMember>> {
    split_admitted(entities, tolerance, limits, canceled, &mut |_| Ok(()))
}

pub fn split_admitted<'a>(
    entities: impl IntoIterator<Item = &'a Entity>,
    tolerance: f64,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
    admit: &mut dyn FnMut(u64) -> Result<()>,
) -> Result<Vec<SplitMember>> {
    if !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("split tolerance"));
    }
    let mut members = Vec::new();
    let mut usage = Usage::default();
    for entity in entities {
        if canceled() {
            return Err(Error::Canceled);
        }
        let Geometry::Curve(local) = &entity.geometry else {
            return Err(Error::Invalid("select curves"));
        };
        usage.add(local.validate(limits)?, limits)?;
        usage.add(Usage::array::<Member>(1)?, limits)?;
        admit(usage.bytes)?;
        let geometry = Geometry::Curve(super::bake_curve(local, entity.placement)?);
        let area = bounds(&geometry).ok_or(Error::Invalid("curve bounds"))?;
        let Geometry::Curve(world) = geometry else {
            unreachable!()
        };
        members.push(Member {
            source: entity,
            local,
            world,
            bounds: area,
            fractions: Vec::new(),
        });
    }
    members.sort_by(|a, b| a.bounds.min[0].total_cmp(&b.bounds.min[0]));
    for index in 0..members.len() {
        if canceled() {
            return Err(Error::Canceled);
        }
        let (left, rest) = members.split_at_mut(index + 1);
        let a = &mut left[index];
        for b in rest {
            if b.bounds.min[0] - a.bounds.max[0] > tolerance {
                break;
            }
            if canceled() {
                return Err(Error::Canceled);
            }
            if (1..3).any(|i| {
                b.bounds.min[i] - a.bounds.max[i] > tolerance
                    || a.bounds.min[i] - b.bounds.max[i] > tolerance
            }) {
                continue;
            }
            let hits = curve_surface_ops::curve_curve_intersections_cancellable(
                &a.world, &b.world, tolerance, limits, canceled,
            )?;
            usage.add(Usage::array::<[f64; 4]>(hits.len())?, limits)?;
            admit(usage.bytes)?;
            for hit in hits {
                a.fractions
                    .push(source_fraction(a.local, &a.world, hit.first_fraction));
                b.fractions
                    .push(source_fraction(b.local, &b.world, hit.second_fraction));
            }
        }
    }
    let mut result = Vec::new();
    for m in members {
        if canceled() {
            return Err(Error::Canceled);
        }
        if m.fractions.is_empty() {
            continue;
        }
        let closed = super::curve_closed_for_cut(m.local, limits)?;
        let mut fractions = super::normalize_cut_fractions(
            m.local,
            m.source.placement,
            m.fractions,
            tolerance,
            closed,
            limits,
        )?;
        if closed {
            if fractions.last() == Some(&1.) {
                fractions.pop();
                if fractions.first() != Some(&0.) {
                    fractions.insert(0, 0.);
                }
            }
            if fractions.len() < 2 {
                continue;
            }
        } else if fractions.is_empty() {
            continue;
        }
        // The trimming routine may hold a copy of the input while building each interval.
        let mut peak = usage;
        let count = fractions
            .len()
            .checked_add(1)
            .ok_or(Error::Overflow("network cut count"))?;
        let per_piece = m.local.validate(limits)?.bytes.saturating_add(1024);
        peak.add(
            Usage {
                bytes: per_piece.saturating_mul(count as u64),
                ..Usage::default()
            },
            limits,
        )?;
        admit(peak.bytes)?;
        let points = fractions
            .iter()
            .map(|&f| {
                m.source
                    .placement
                    .point(curve_ops::point_at(m.local, f, limits)?)
            })
            .collect::<Result<_>>()?;
        let pieces = if closed {
            super::split_closed_at_fractions(m.local, &fractions, limits)?
        } else {
            super::split_at_fractions(m.local, &fractions, limits)?
        };
        for piece in &pieces {
            let mut cost = piece.validate(limits)?;
            cost.outputs = 1;
            usage.add(cost, limits)?;
        }
        result.push(SplitMember {
            source: m.source.id,
            pieces,
            points,
        });
    }
    Ok(result)
}
