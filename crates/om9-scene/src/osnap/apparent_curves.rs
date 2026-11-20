// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::camera::{Camera, PointProjector, Projection, Viewport};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    curve_ops::split_curve,
    curve_surface_ops::{closest_point_on_curve, curve_curve_intersections_cancellable},
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, NurbsCurve, Transform},
    top11::{affine_circle, bake_curve},
};

use super::{
    SnapCandidate, SnapKind, SnapSource, apparent_segment_intersection_with_parameter_map,
    apparent_source_precedes, closest_on_segment, distance, source_intersection_witness,
};

const IMAGE_TOLERANCE_PX: f64 = 1e-6;
const MAX_PROJECTION_PIECES: usize = 127;
const MAX_IMAGE_PAIRS: usize = 128;

struct ProjectedCurve {
    world: Curve,
    pieces: Vec<ProjectedPiece>,
}

struct ProjectedPiece {
    image: Curve,
    source_range: [f64; 2],
    line_depths: Option<[f64; 2]>,
}

impl ProjectedPiece {
    fn source_fraction(&self, projection: Projection, fraction: f64) -> Result<f64> {
        let local = match self.line_depths {
            Some(depths) => projection.segment_source_fraction(fraction, depths)?,
            None => fraction,
        };
        let [start, end] = self.source_range;
        Ok(start + local * (end - start))
    }
}

impl ProjectedCurve {
    fn capture(
        source: &Curve,
        placement: Transform,
        camera: &Camera,
        projector: &PointProjector<'_>,
        limits: &Limits,
        canceled: &dyn Fn() -> bool,
    ) -> Result<Option<Self>> {
        source.validate(limits)?;
        let world = match source {
            Curve::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => {
                let angles = angle_domain.unwrap_or(*domain);
                if angles[1] - angles[0] > std::f64::consts::TAU + 1e-10 {
                    return Ok(None);
                }
                affine_circle(*center, *x, *y, *radius, angles, placement)?
            }
            Curve::Nurbs(nurbs) if nurbs.knots.len() > 383 => {
                return Err(Error::Budget("apparent intersection source spans"));
            }
            Curve::Nurbs(_) | Curve::Line { .. } => bake_curve(source, placement)?,
            Curve::Polycurve { .. } => {
                return Err(Error::Invalid("unflattened apparent intersection source"));
            }
        };
        let as_point = |screen: [f64; 2]| [screen[0], screen[1], 0.];
        let pieces = match &world {
            Curve::Line { start, end, .. } => {
                let Some([a, b]) = camera.clip_segment(*start, *end)? else {
                    return Ok(None);
                };
                let (Some(first), Some(last)) = (projector.project(a)?, projector.project(b)?)
                else {
                    return Ok(None);
                };
                if first.screen == last.screen {
                    return Ok(None);
                }
                vec![ProjectedPiece {
                    image: Curve::Line {
                        start: as_point(first.screen),
                        end: as_point(last.screen),
                        domain: [0., 1.],
                    },
                    source_range: [
                        closest_on_segment(a, *start, *end).1,
                        closest_on_segment(b, *start, *end).1,
                    ],
                    line_depths: Some([first.depth, last.depth]),
                }]
            }
            Curve::Nurbs(nurbs) => {
                if nurbs.weights.iter().any(|weight| *weight <= 0.) {
                    return Ok(None);
                }
                let perspective = matches!(camera.projection, Projection::Perspective { .. });
                if perspective && nurbs.poles.iter().any(|&p| projector.depth(p) <= 0.) {
                    project_visible_pieces(nurbs, camera.near_mm, projector, limits, canceled)?
                } else {
                    let Some(piece) = project_nurbs(
                        nurbs.clone(),
                        [0., 1.],
                        perspective,
                        projector,
                        limits,
                        canceled,
                    )?
                    else {
                        return Ok(None);
                    };
                    vec![piece]
                }
            }
            _ => unreachable!("world curve is a segment or a rational curve"),
        };
        Ok((!pieces.is_empty()).then_some(Self { world, pieces }))
    }

    fn distance_to(
        &self,
        point: [f64; 3],
        limits: &Limits,
        canceled: &dyn Fn() -> bool,
    ) -> Result<f64> {
        self.pieces
            .iter()
            .try_fold(f64::INFINITY, |nearest, piece| {
                if canceled() {
                    return Err(Error::Canceled);
                }
                Ok(nearest.min(closest_point_on_curve(&piece.image, point, limits)?.2))
            })
    }
}

fn project_nurbs(
    mut image: NurbsCurve,
    source_range: [f64; 2],
    perspective: bool,
    projector: &PointProjector<'_>,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
) -> Result<Option<ProjectedPiece>> {
    let mut maximum_depth = 0_f64;
    for pole in &mut image.poles {
        if canceled() {
            return Err(Error::Canceled);
        }
        let Some(projected) = projector.project_unclipped(*pole)? else {
            return Ok(None);
        };
        *pole = [projected.screen[0], projected.screen[1], projected.depth];
        maximum_depth = maximum_depth.max(projected.depth);
    }
    // Multiplying each source weight by its depth gives the exact rational
    // image under Perspective, with the same parameterization.
    for (pole, weight) in image.poles.iter_mut().zip(&mut image.weights) {
        if perspective {
            *weight *= pole[2] / maximum_depth;
        }
        pole[2] = 0.;
    }
    let image = Curve::Nurbs(image);
    image.validate(limits)?;
    Ok(Some(ProjectedPiece {
        image,
        source_range,
        line_depths: None,
    }))
}

fn project_visible_pieces(
    nurbs: &NurbsCurve,
    near: f64,
    projector: &PointProjector<'_>,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
) -> Result<Vec<ProjectedPiece>> {
    let mut base = nurbs.clone();
    // Split a reseamed curve at the existing seam before subdividing. Each
    // piece then maps affinely to the original traversal fraction.
    let offset = base.seam.take().map_or(0., |seam| {
        let [start, end] = base.domain;
        if base.reversed {
            (end - seam) / (end - start)
        } else {
            (seam - start) / (end - start)
        }
    });
    let base = Curve::Nurbs(base);
    let mut pending = if offset > 0. && offset < 1. {
        let (before, after) = split_curve(&base, offset, limits)?;
        vec![(before, [1. - offset, 1.]), (after, [0., 1. - offset])]
    } else {
        vec![(base, [0., 1.])]
    };
    let mut pieces = Vec::new();
    let mut usage = Usage::default();
    let mut visited = 0;
    while let Some((piece, range)) = pending.pop() {
        if canceled() {
            return Err(Error::Canceled);
        }
        visited += 1;
        if visited > MAX_PROJECTION_PIECES || visited as u64 > limits.max_nodes {
            return Err(Error::Budget("apparent intersection projection pieces"));
        }
        let Curve::Nurbs(nurbs) = &piece else {
            unreachable!("splitting a NURBS keeps its curve type");
        };
        let (low, high) = nurbs
            .poles
            .iter()
            .map(|&p| projector.depth(p))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), depth| {
                (low.min(depth), high.max(depth))
            });
        // Positive source weights bound every curve depth by the control
        // depths. The overlapping thresholds let subdivision finish away
        // from the eye without guessing where the curve clips at the near plane.
        if high < near * 0.75 {
            continue;
        }
        if low > near * 0.5 {
            usage.add(Usage::array::<ProjectedPiece>(1)?, limits)?;
            usage.add(Usage::array::<[f64; 4]>(nurbs.poles.len())?, limits)?;
            usage.add(Usage::array::<f64>(nurbs.knots.len())?, limits)?;
            let Curve::Nurbs(nurbs) = piece else {
                unreachable!()
            };
            if let Some(image) = project_nurbs(nurbs, range, true, projector, limits, canceled)? {
                pieces.push(image);
            }
            continue;
        }
        let [start, end] = range;
        let middle = start.midpoint(end);
        if middle == start || middle == end {
            return Err(Error::Budget("apparent intersection source interval"));
        }
        let (left, right) = split_curve(&piece, 0.5, limits)?;
        pending.extend([(right, [middle, end]), (left, [start, middle])]);
    }
    Ok(pieces)
}

/// Resolve view intersections onto their original placed curves.
pub fn apparent_curve_intersections(
    first: (&Curve, Transform, &SnapSource),
    second: (&Curve, Transform, &SnapSource),
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    radius_px: f64,
    tolerance_mm: f64,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
) -> Result<Vec<SnapCandidate>> {
    apparent_curve_intersections_with_parameter_map(
        first,
        second,
        camera,
        viewport,
        cursor,
        radius_px,
        tolerance_mm,
        limits,
        canceled,
        &|_, parameter| parameter,
    )
}

/// Resolve apparent intersections while mapping a constituent curve's parameter
/// to its parent. The mapper receives the selected input index (0 or 1), even
/// when both constituents have the same source identity.
pub fn apparent_curve_intersections_with_parameter_map(
    (first, first_transform, first_source): (&Curve, Transform, &SnapSource),
    (second, second_transform, second_source): (&Curve, Transform, &SnapSource),
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    radius_px: f64,
    tolerance_mm: f64,
    limits: &Limits,
    canceled: &dyn Fn() -> bool,
    map_parameter: &dyn Fn(usize, f64) -> f64,
) -> Result<Vec<SnapCandidate>> {
    if canceled() {
        return Err(Error::Canceled);
    }
    if !cursor.iter().all(|coordinate| coordinate.is_finite())
        || !radius_px.is_finite()
        || radius_px < 0.
        || !tolerance_mm.is_finite()
        || tolerance_mm <= 0.
    {
        return Err(Error::Invalid("apparent intersection parameters"));
    }
    if let (
        Curve::Line {
            start: a, end: b, ..
        },
        Curve::Line {
            start: c, end: d, ..
        },
    ) = (first, second)
    {
        return Ok(apparent_segment_intersection_with_parameter_map(
            (
                [first_transform.point(*a)?, first_transform.point(*b)?],
                first_source,
            ),
            (
                [second_transform.point(*c)?, second_transform.point(*d)?],
                second_source,
            ),
            camera,
            viewport,
            cursor,
            radius_px,
            tolerance_mm,
            map_parameter,
        )?
        .into_iter()
        .collect());
    }
    let projector = camera.projector(viewport)?;
    let Some(a) =
        ProjectedCurve::capture(first, first_transform, camera, &projector, limits, canceled)?
    else {
        return Ok(Vec::new());
    };
    let Some(b) = ProjectedCurve::capture(
        second,
        second_transform,
        camera,
        &projector,
        limits,
        canceled,
    )?
    else {
        return Ok(Vec::new());
    };
    let pointer = [cursor[0], cursor[1], 0.];
    let evaluator_a = CurveEvaluator::capture(&a.world, limits)?;
    let evaluator_b = CurveEvaluator::capture(&b.world, limits)?;
    let pair_count = a.pieces.len() * b.pieces.len();
    if pair_count > MAX_IMAGE_PAIRS || pair_count as u64 > limits.max_nodes {
        return Err(Error::Budget("apparent intersection projected pairs"));
    }
    let mut candidates: Vec<SnapCandidate> = Vec::new();
    let mut source_fractions: Vec<[f64; 2]> = Vec::new();
    let mut distances = None;
    let mut usage = Usage::default();
    for (piece_a, piece_b) in a
        .pieces
        .iter()
        .flat_map(|a| b.pieces.iter().map(move |b| (a, b)))
    {
        if canceled() {
            return Err(Error::Canceled);
        }
        let hits = match curve_curve_intersections_cancellable(
            &piece_a.image,
            &piece_b.image,
            IMAGE_TOLERANCE_PX,
            limits,
            canceled,
        ) {
            Err(Error::Unsupported(
                "overlapping line segments"
                | "overlapping line and NURBS"
                | "line and NURBS may overlap"
                | "overlapping NURBS curves",
            )) => continue,
            result => result?,
        };
        for hit in hits {
            if canceled() {
                return Err(Error::Canceled);
            }
            if distance(hit.point, pointer) > radius_px + IMAGE_TOLERANCE_PX {
                continue;
            }
            let fa = piece_a.source_fraction(camera.projection, hit.first_fraction)?;
            let fb = piece_b.source_fraction(camera.projection, hit.second_fraction)?;
            let wa = evaluator_a.evaluate(Parameter::Fraction(fa), 0, Transform::identity())?[0];
            let wb = evaluator_b.evaluate(Parameter::Fraction(fb), 0, Transform::identity())?[0];
            let (pa, ap) =
                source_intersection_witness(first, first_transform, fa, wa, tolerance_mm, limits)?;
            let (pb, bp) = source_intersection_witness(
                second,
                second_transform,
                fb,
                wb,
                tolerance_mm,
                limits,
            )?;
            let separation = distance(pa, pb);
            if !separation.is_finite() || separation <= tolerance_mm {
                continue;
            }
            let (Some(sa), Some(sb)) = (projector.project(pa)?, projector.project(pb)?) else {
                continue;
            };
            if (sa.screen[0] - sb.screen[0]).hypot(sa.screen[1] - sb.screen[1])
                > IMAGE_TOLERANCE_PX * 2.
            {
                continue;
            }
            let [distance_a, distance_b] = match distances {
                Some(distances) => distances,
                None => {
                    let computed = [
                        a.distance_to(pointer, limits, canceled)?,
                        b.distance_to(pointer, limits, canceled)?,
                    ];
                    distances = Some(computed);
                    computed
                }
            };
            let (world, screen, parameter, source, other) = if apparent_source_precedes(
                (distance_a, sa.depth),
                (distance_b, sb.depth),
                IMAGE_TOLERANCE_PX,
            ) {
                (
                    pa,
                    sa.screen,
                    map_parameter(0, ap),
                    first_source,
                    second_source,
                )
            } else {
                (
                    pb,
                    sb.screen,
                    map_parameter(1, bp),
                    second_source,
                    first_source,
                )
            };
            let distance_px = (screen[0] - cursor[0]).hypot(screen[1] - cursor[1]);
            if distance_px > radius_px {
                continue;
            }
            let candidate = SnapCandidate {
                kind: SnapKind::Intersection,
                root: Some(source.root),
                world,
                screen,
                distance_px,
                apparent: true,
                normal: None,
                parameter: Some(parameter),
                tracking_tangent: None,
                source: Some(source.clone()),
                second_source: Some(other.clone()),
                surface_source: None,
            };
            // A split endpoint can be returned by both image pieces and can
            // also have an adjacent numerical root in either piece.
            if let Some(index) =
                source_fractions
                    .iter()
                    .zip(&candidates)
                    .position(|([a, b], prior)| {
                        (fa - a).abs() <= 1e-7
                            && (fb - b).abs() <= 1e-7
                            && prior.root == candidate.root
                            && distance(prior.world, candidate.world) <= tolerance_mm
                            && (prior.screen[0] - screen[0]).hypot(prior.screen[1] - screen[1])
                                <= IMAGE_TOLERANCE_PX * 2.
                    })
            {
                if distance_px < candidates[index].distance_px {
                    candidates[index] = candidate;
                    source_fractions[index] = [fa, fb];
                }
                continue;
            }
            let mut needed = Usage::array::<SnapCandidate>(1)?;
            needed.outputs = 1;
            if pair_count > 1 {
                needed.add(Usage::array::<[f64; 2]>(1)?, limits)?;
                source_fractions
                    .try_reserve(1)
                    .map_err(|_| Error::Budget("apparent intersection witnesses"))?;
                source_fractions.push([fa, fb]);
            }
            usage.add(needed, limits)?;
            candidates
                .try_reserve(1)
                .map_err(|_| Error::Budget("apparent intersection candidates"))?;
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}
