// SPDX-License-Identifier: LGPL-2.1-or-later
//! Accounted, cancellable source-curve pair evaluation for intersection snaps.
//! The caller owns scheduling and must revalidate every occurrence before use.
use super::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Geometry, Transform},
    transaction::{CapturedDocument, ExternalMemoryReservation}};
use om9_scene::osnap::{SnapCandidate, SnapSource};
use super::edge_snap::{EdgeSnapSourceReceipt, EdgeSnapViewInput};

#[derive(Clone)]
pub(crate) struct IntersectionOccurrence {
    pub source: SnapSource,
    pub transform: Transform,
    /// None means a standalone source curve retained in the captured document.
    pub edges: Option<Arc<EdgeSnapSourceReceipt>>,
}

pub(crate) struct IntersectionReceipt {
    occurrences: Vec<IntersectionOccurrence>,
    input: EdgeSnapViewInput,
    tolerance: f64,
    selection_revision: u64,
    preview_revision: u64,
    candidates: Vec<SnapCandidate>,
    canceled: Arc<AtomicBool>,
    _reservation: ExternalMemoryReservation,
}

impl IntersectionReceipt {
    pub fn resolve<'a>(&'a self, controller: &Controller, input: &EdgeSnapViewInput) -> Result<&'a [SnapCandidate]> {
        if self.canceled.load(Ordering::Acquire) { return Err(Error::Canceled); }
        if &self.input != input || self.selection_revision != controller.selection_state().revision
            || self.preview_revision != controller.preview_revision()
            || self.tolerance.to_bits() != controller.document().effective_tolerance()?.absolute_mm.to_bits() {
            return Err(Error::Stale("intersection snap view"));
        }
        let scene = controller.scene().ok_or(Error::Missing("intersection snap scene"))?;
        for occurrence in &self.occurrences {
            occurrence.source.validate(scene)?;
            if let Some(edges) = &occurrence.edges { edges.validate(controller)?; }
        }
        Ok(&self.candidates)
    }
}

pub(crate) struct IntersectionJob {
    canceled: Arc<AtomicBool>,
    receiver: Option<mpsc::Receiver<Result<IntersectionReceipt>>>,
    delivered: bool,
}

pub(super) fn document_entity<'a>(document: &'a om9_core::model::Document, source: &SnapSource) -> Result<&'a om9_core::model::Entity> {
    let mut entity = document.resolve(&source.stamp)?;
    for member in &source.path {
        let Geometry::Instance(definition) = entity.geometry else { return Err(Error::Stale("intersection member path")); };
        entity = document.definitions.get(&definition).ok_or(Error::Stale("intersection definition"))?
            .members.iter().find(|entry| entry.id == *member).ok_or(Error::Stale("intersection member"))?;
    }
    if entity.id != source.leaf || entity.generation != source.leaf_generation
        || entity.geometry_revision != source.leaf_geometry_revision { return Err(Error::Stale("intersection leaf")); }
    Ok(entity)
}

#[derive(Clone, Copy)]
struct IntersectionSegment<'a> {
    curve: &'a Curve,
    occurrence: &'a IntersectionOccurrence,
    parent_range: Option<[f64; 2]>,
}

impl IntersectionSegment<'_> {
    fn source_parameter(self, parameter: f64) -> f64 {
        let Some([start, end]) = self.parent_range else {
            return parameter;
        };
        let fraction = match self.curve {
            Curve::Circle {
                domain,
                angle_domain,
                ..
            } => {
                let [first, last] = angle_domain.unwrap_or(*domain);
                (parameter - first) / (last - first)
            }
            _ => parameter,
        };
        start + fraction.clamp(0., 1.) * (end - start)
    }

    fn map_parameters(self, hits: &mut [SnapCandidate]) {
        for hit in hits {
            hit.parameter = hit
                .parameter
                .map(|parameter| self.source_parameter(parameter));
        }
    }
}

fn flatten<'a>(
    curve: &'a Curve,
    occurrence: &'a IntersectionOccurrence,
    parent_range: Option<[f64; 2]>,
    depth: usize,
    limits: &om9_core::budget::Limits,
    output: &mut Vec<IntersectionSegment<'a>>,
) -> Result<()> {
    if depth > limits.max_depth {
        return Err(Error::Budget("intersection curve depth"));
    }
    if let Curve::Polycurve { segments, reversed } = curve {
        let ranges = om9_core::curve_ops::polycurve_segment_ranges(
            parent_range.unwrap_or([0., 1.]), segments.len(), *reversed,
        );
        for (segment, range) in segments.iter().zip(ranges) {
            flatten(segment, occurrence, Some(range), depth + 1, limits, output)?;
        }
    } else {
        if output.len() >= 8192 || output.len() as u64 >= limits.max_outputs {
            return Err(Error::Budget("intersection source curves"));
        }
        output
            .try_reserve(1)
            .map_err(|_| Error::Budget("intersection source curves"))?;
        output.push(IntersectionSegment {
            curve,
            occurrence,
            parent_range,
        });
    }
    Ok(())
}

/// Bound an infinite tracking line by the source curve's world-space control
/// hull. The line remains transient and does not acquire a document identity.
fn guide_segment(curve: &Curve, transform: Transform, guide: om9_scene::osnap::SnapGuide,
    tolerance: f64) -> Result<Curve> {
    let origin = std::array::from_fn::<_, 3, _>(|i| guide.start[i] * 0.5 + guide.end[i] * 0.5);
    let axis = std::array::from_fn::<_, 3, _>(|i| guide.end[i] - guide.start[i]);
    let length = axis.iter().map(|v| v * v).sum::<f64>().sqrt();
    if !length.is_finite() || length <= 0. { return Err(Error::Invalid("SmartTrack guide")); }
    let distance = |p: Point3| (0..3).map(|i| (p[i] - origin[i]).powi(2)).sum::<f64>().sqrt();
    let extent = match curve {
        Curve::Line { start, end, .. } => distance(transform.point(*start)?).max(distance(transform.point(*end)?)),
        Curve::Nurbs(nurbs) => {
            if nurbs.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) {
                return Err(Error::Unsupported("SmartTrack NURBS control hull"));
            }
            let mut extent = 0_f64;
            for &pole in &nurbs.poles { extent = extent.max(distance(transform.point(pole)?)); }
            extent
        }
        Curve::Circle { center, x, y, radius, .. } => {
            let c = transform.point(*center)?;
            let px = transform.point(std::array::from_fn(|i| center[i] + x[i] * *radius))?;
            let py = transform.point(std::array::from_fn(|i| center[i] + y[i] * *radius))?;
            let norm = |p: Point3| (0..3).map(|i| (p[i] - c[i]).powi(2)).sum::<f64>().sqrt();
            distance(c) + norm(px) + norm(py)
        }
        Curve::Polycurve { .. } => return Err(Error::Invalid("unflattened SmartTrack curve")),
    } + tolerance;
    if !extent.is_finite() || extent <= 0. { return Err(Error::Invalid("SmartTrack guide extent")); }
    let direction = axis.map(|value| value / length * extent);
    Ok(Curve::Line { start: std::array::from_fn(|i| origin[i] - direction[i]),
        end: std::array::from_fn(|i| origin[i] + direction[i]), domain: [0., 1.] })
}

fn curve_may_hit(curve: &Curve, transform: Transform, input: &EdgeSnapViewInput) -> Result<bool> {
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    fn include(low: &mut [f64; 3], high: &mut [f64; 3], transform: Transform,
        point: om9_core::geometry::Point3) -> Result<()> {
        let point = transform.point(point)?;
        for axis in 0..3 { low[axis] = low[axis].min(point[axis]); high[axis] = high[axis].max(point[axis]); }
        Ok(())
    }
    match curve {
        Curve::Line { start, end, .. } => { include(&mut low, &mut high, transform, *start)?; include(&mut low, &mut high, transform, *end)?; }
        Curve::Nurbs(curve) => {
            if curve.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) { return Ok(true); }
            for &point in &curve.poles { include(&mut low, &mut high, transform, point)?; }
        }
        Curve::Circle { center, x, y, radius, .. } => {
            // The box enclosing the whole affine ellipse also encloses an arc.
            let center = transform.point(*center)?;
            let origin = transform.point([0.; 3])?;
            let tx = transform.point(*x)?;
            let ty = transform.point(*y)?;
            for axis in 0..3 {
                let extent = radius * ((tx[axis] - origin[axis]).abs() + (ty[axis] - origin[axis]).abs());
                low[axis] = center[axis] - extent;
                high[axis] = center[axis] + extent;
            }
        }
        Curve::Polycurve { .. } => return Ok(true),
    }
    if low.iter().chain(high.iter()).any(|value| !value.is_finite()) { return Ok(true); }
    let mut corners = [[0.; 3]; 8];
    for (index, corner) in corners.iter_mut().enumerate() {
        *corner = std::array::from_fn(|axis| if index & (1 << axis) == 0 { low[axis] } else { high[axis] });
    }
    let bounds = match input.camera.projector(input.viewport)?.project_box(corners) {
        Ok(Some(bounds)) => bounds,
        Ok(None) | Err(_) => return Ok(true),
    };
    let radius = input.settings.radius_px + 1e-7;
    Ok((0..2).all(|axis| input.cursor[axis] >= bounds[0][axis] - radius
        && input.cursor[axis] <= bounds[1][axis] + radius))
}

/// Split until each positive-weight control polygon has a strictly monotone
/// projection, then intersect distinct pieces. Unresolved pieces are refused.
fn self_candidates(curve: &Curve, occurrence: &IntersectionOccurrence, input: &EdgeSnapViewInput,
    tolerance: f64, limits: &om9_core::budget::Limits, canceled: &AtomicBool,
    work: &mut usize) -> Result<Vec<SnapCandidate>> {
    let Curve::Nurbs(nurbs) = curve else { return Ok(Vec::new()) };
    let spans = nurbs.knots.windows(2).filter(|pair| pair[1] > pair[0]
        && pair[1] > nurbs.domain[0] && pair[0] < nurbs.domain[1]).count();
    // A single positive-weight linear or quadratic rational span has no
    // isolated self-crossing; avoid manufacturing a continuum of coincident hits.
    if nurbs.degree <= 2 && spans <= 1 && nurbs.weights.iter().all(|weight| *weight > 0.) {
        return Ok(Vec::new());
    }
    let evaluator = om9_core::evaluation::CurveEvaluator::capture(curve, limits)?;
    let endpoint = |f| -> Result<Point3> {
        Ok(evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(f), 0, occurrence.transform)?[0])
    };
    let start = endpoint(0.)?;
    let end = endpoint(1.)?;
    let closed = (0..3).map(|axis| (start[axis] - end[axis]).powi(2)).sum::<f64>().sqrt() <= tolerance;
    let mut pending = vec![(0., 1.)];
    let mut pieces = Vec::new();
    let mut subdivisions = 0usize;
    let piece_bound = curve.validate(limits)?.bytes.checked_mul(4).and_then(|n| n.checked_add(65_536))
        .ok_or(Error::Overflow("self intersection pieces"))?;
    let mut retained_bytes = 0u64;
    while let Some((low, high)) = pending.pop() {
        if canceled.load(Ordering::Acquire) { return Err(Error::Canceled); }
        subdivisions += 1;
        if subdivisions > 128 { return Err(Error::Budget("self intersection subdivision")); }
        if retained_bytes.checked_add(piece_bound).is_none_or(|n| n > limits.max_bytes) {
            return Err(Error::Budget("self intersection pieces"));
        }
        let piece = if low == 0. && high == 1. { curve.clone() }
            else { om9_core::curve_ops::trim_curve_by_parameter(curve, low, high, limits)? };
        if !curve_may_hit(&piece, occurrence.transform, input)? { continue; }
        let Curve::Nurbs(part) = &piece else { return Err(Error::Unsupported("self intersection source piece")); };
        if part.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) {
            return Err(Error::Unsupported("self intersection source weights"));
        }
        let middle = low * 0.5 + high * 0.5;
        let check = om9_core::evaluation::CurveEvaluator::capture(&piece, limits)?
            .evaluate(om9_core::evaluation::Parameter::Fraction(0.5), 0, occurrence.transform)?[0];
        let original_midpoint = endpoint(middle)?;
        if (0..3).map(|i| (check[i] - original_midpoint[i]).powi(2)).sum::<f64>().sqrt() > tolerance.min(1e-6) {
            return Err(Error::Unsupported("self intersection parameterization"));
        }
        let first = occurrence.transform.point(part.poles[0])?;
        let last = occurrence.transform.point(*part.poles.last().ok_or(Error::Invalid("self intersection poles"))?)?;
        let chord: Point3 = std::array::from_fn(|i| last[i] - first[i]);
        let axes = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], chord];
        let mut monotone = false;
        for axis in axes {
            let norm = axis.iter().map(|v| v * v).sum::<f64>().sqrt();
            if !norm.is_finite() || norm == 0. { continue; }
            let unit = axis.map(|v| v / norm);
            let mut values = Vec::new();
            values.try_reserve_exact(part.poles.len()).map_err(|_| Error::Budget("self intersection hull"))?;
            for &pole in &part.poles {
                let point = occurrence.transform.point(pole)?;
                let value = (0..3).map(|i| point[i] * unit[i]).sum::<f64>();
                if !value.is_finite() { return Err(Error::Invalid("self intersection hull")); }
                values.push(value);
            }
            let ordered = values.windows(2).all(|pair| pair[0] <= pair[1])
                || values.windows(2).all(|pair| pair[0] >= pair[1]);
            if !ordered { continue; }
            // Repeated endpoint controls are fine, but no nonempty knot span
            // may have a constant active control window.
            let mut has_span = false;
            let mut strict = true;
            for k in part.degree..part.poles.len() {
                if part.knots[k] >= part.knots[k + 1] || part.knots[k + 1] <= part.domain[0]
                    || part.knots[k] >= part.domain[1] { continue; }
                has_span = true;
                let (a, b) = (values[k - part.degree], values[k]);
                let margin = 128. * f64::EPSILON * a.abs().max(b.abs()).max(1.);
                if (a - b).abs() <= margin { strict = false; break; }
            }
            if has_span && strict { monotone = true; break; }
        }
        if monotone {
            if pieces.len() >= 32 { return Err(Error::Budget("self intersection pieces")); }
            retained_bytes = retained_bytes.checked_add(piece.validate(limits)?.bytes)
                .and_then(|n| n.checked_add(std::mem::size_of::<(f64, f64, Curve)>() as u64 * 2))
                .ok_or(Error::Overflow("self intersection pieces"))?;
            pieces.try_reserve(1).map_err(|_| Error::Budget("self intersection pieces"))?;
            pieces.push((low, high, piece));
        } else {
            if middle == low || middle == high { return Err(Error::Unsupported("self intersection parameter precision")); }
            pending.try_reserve(2).map_err(|_| Error::Budget("self intersection subdivision"))?;
            pending.push((middle, high)); pending.push((low, middle));
        }
    }
    let mut result = Vec::new();
    for first_index in 0..pieces.len() {
        if canceled.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let (first_low, first_high, first) = &pieces[first_index];
        for second_index in first_index + 1..pieces.len() {
            if canceled.load(Ordering::Acquire) { return Err(Error::Canceled); }
            let (second_low, second_high, second) = &pieces[second_index];
            *work = work.checked_add(1).ok_or(Error::Overflow("intersection pair work"))?;
            if *work > 128 { return Err(Error::Budget("intersection pair work")); }
            let first_world = om9_core::top11::bake_curve(first, occurrence.transform)?;
            let second_world = om9_core::top11::bake_curve(second, occurrence.transform)?;
            let hits = om9_core::curve_surface_ops::curve_curve_intersections_cancellable(
                &first_world, &second_world, tolerance, limits, &|| canceled.load(Ordering::Acquire))?;
            for hit in hits {
                let f = *first_low + (first_high - first_low) * hit.first_fraction;
                let g = *second_low + (second_high - second_low) * hit.second_fraction;
                if (g - f).abs() <= 1e-8 || closed && f <= 1e-8 && g >= 1. - 1e-8 { continue; }
                let a = endpoint(f)?;
                let b = endpoint(g)?;
                if first_high == second_low {
                    let joint = endpoint(*first_high)?;
                    let near_joint = |p: Point3| (0..3).map(|i| (p[i] - joint[i]).powi(2)).sum::<f64>().sqrt() <= tolerance * 2.;
                    if near_joint(a) && near_joint(b) { continue; }
                }
                let separation = (0..3).map(|axis| (a[axis] - b[axis]).powi(2)).sum::<f64>().sqrt();
                if !separation.is_finite() || separation > tolerance { continue; }
                let point = std::array::from_fn(|axis| a[axis] * 0.5 + b[axis] * 0.5);
                let Some(projected) = input.camera.project(point, input.viewport)? else { continue };
                let distance_px = (projected.screen[0] - input.cursor[0]).hypot(projected.screen[1] - input.cursor[1]);
                if !distance_px.is_finite() || distance_px > input.settings.radius_px { continue; }
                if result.iter().any(|old: &SnapCandidate| old.parameter.is_some_and(|parameter| (parameter - f).abs() <= 1e-8)
                    && (0..3).map(|axis| (old.world[axis] - point[axis]).powi(2)).sum::<f64>().sqrt() <= tolerance) { continue; }
                if result.len() >= 256 { return Err(Error::Budget("self intersection candidates")); }
                result.try_reserve(1).map_err(|_| Error::Budget("self intersection candidates"))?;
                result.push(SnapCandidate { kind: om9_scene::osnap::SnapKind::Intersection, root: Some(occurrence.source.root),
                    world: point, screen: projected.screen, distance_px, normal: None, parameter: Some(f), tracking_tangent: None,
                    source: Some(occurrence.source.clone()), second_source: Some(occurrence.source.clone()), surface_source: None, apparent: false });
            }
        }
    }
    Ok(result)
}

impl IntersectionJob {
    pub fn required_bytes(occurrences: &Vec<IntersectionOccurrence>) -> Result<u64> {
        // A bounded allocation floor for pair scratch and candidate paths. The
        // caller reserves additional space for its occurrence vector and any
        // retained captured document; the kernel has no native allocations here.
        let mut required = 64u64 * 1024 * 1024;
        let mut max_path = 0u64;
        for occurrence in occurrences {
            max_path = max_path.max(u64::try_from(occurrence.source.path.len())
                .map_err(|_| Error::Overflow("intersection admission"))?);
            required = required.checked_add(u64::try_from(occurrence.source.path.capacity())
                .map_err(|_| Error::Overflow("intersection admission"))?.saturating_mul(32))
                .ok_or(Error::Overflow("intersection admission"))?;
        }
        // Both identities may be copied into up to 1024 retained hits plus
        // one 256-hit pair buffer; allow vector growth during aggregation.
        required = required.checked_add(max_path.checked_mul(2 * 16 * 1280 * 2)
            .ok_or(Error::Overflow("intersection admission"))?)
            .and_then(|bytes| bytes.checked_add((occurrences.capacity() as u64)
                .saturating_mul(std::mem::size_of::<IntersectionOccurrence>() as u64).saturating_mul(2)))
            .ok_or(Error::Overflow("intersection admission"))?;
        Ok(required)
    }

    pub fn start(controller: &Controller, occurrences: Vec<IntersectionOccurrence>, input: EdgeSnapViewInput,
        mut reservation: ExternalMemoryReservation) -> Result<Self> {
        if reservation.class() != MemoryClass::CpuCache
            || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("intersection worker admission"));
        }
        let scene = controller.scene().ok_or(Error::Missing("intersection snap scene"))?;
        let current = controller.coordinator().world().documents.get(&controller.document().id)
            .ok_or(Error::Missing("intersection document"))?;
        if !std::ptr::eq(scene.document.as_ref(), current.as_ref()) {
            return Err(Error::Stale("intersection document snapshot"));
        }
        for occurrence in &occurrences {
            occurrence.source.validate(scene)?;
            let object = scene.objects.iter().find(|object| object.root == occurrence.source.root
                && object.path == occurrence.source.path && object.leaf == occurrence.source.leaf
                && object.source == occurrence.source.stamp).ok_or(Error::Stale("intersection occurrence"))?;
            if object.display_transform()? != occurrence.transform { return Err(Error::Stale("intersection placement")); }
            if let Some(edges) = &occurrence.edges {
                edges.validate(controller)?;
                if edges.source != occurrence.source { return Err(Error::Stale("intersection edge source")); }
            }
        }
        let required = Self::required_bytes(&occurrences)?;
        if reservation.pending_bytes() < required { return Err(Error::Budget("intersection worker admission")); }
        reservation.recheck_pending()?;
        let document: CapturedDocument = controller.coordinator().capture_tracked_document(controller.document().id)?;
        let limits = *controller.coordinator().limits();
        let tolerance = controller.document().effective_tolerance()?.absolute_mm;
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        let occlusion = if input.mode == om9_scene::workspace::DisplayMode::Shaded {
            Some(controller.capture_surface_occlusion()?.ok_or(Error::Unsupported("intersection occlusion worker"))?)
        } else { None };
        let canceled = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&canceled);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-intersection-snap".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<IntersectionReceipt> {
                reservation.recheck_pending()?;
                let mut boundaries: Vec<(usize, Curve)> = Vec::new();
                let mut boundary_bytes = 0u64;
                for (index, occurrence) in occurrences.iter().enumerate() {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    if occurrence.edges.is_some() { continue; }
                    let entity = document_entity(&document, &occurrence.source)?;
                    if let Geometry::Surface(surface) = &entity.geometry {
                        if entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                            let mut items = Vec::new();
                            let retained = std::cell::Cell::new(boundary_bytes);
                            let mut admit = |live: u64, pending: u64| -> Result<()> {
                                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                                let total = retained.get().checked_add(live).and_then(|n| n.checked_add(pending))
                                    .ok_or(Error::Overflow("intersection trim boundaries"))?;
                                if total > 8 * 1024 * 1024 { return Err(Error::Budget("intersection trim boundaries")); }
                                reservation.recheck_pending()
                            };
                            super::wireframe::append_trimmed_surface_edges_at(entity, occurrence.transform, &mut items,
                                "ExtractWire", &limits, tolerance, &retained, &mut admit)?;
                            boundary_bytes = retained.get();
                            boundaries.try_reserve(items.len()).map_err(|_| Error::Budget("intersection trim boundaries"))?;
                            for (_, geometry, _) in items {
                                let Geometry::Curve(curve) = geometry else { return Err(Error::Invalid("intersection trim curve")); };
                                boundaries.push((index, curve));
                            }
                            continue;
                        }
                        use om9_core::surface_kernel::SurfaceIso;
                        for (iso, fixed_axis) in [(SurfaceIso::U, 1usize), (SurfaceIso::V, 0usize)] {
                            for side in 0..if surface.periodic[fixed_axis] { 1 } else { 2 } {
                                let mut extraction_limits = limits;
                                extraction_limits.max_bytes = extraction_limits.max_bytes.min(8 * 1024 * 1024);
                                let curve = Curve::Nurbs(om9_core::surface_kernel::isocurve(
                                    surface, iso, surface.domain[fixed_axis][side], &extraction_limits)?);
                                boundary_bytes = boundary_bytes.checked_add(curve.validate(&extraction_limits)?.bytes)
                                    .ok_or(Error::Overflow("intersection surface boundaries"))?;
                                if boundary_bytes > 8 * 1024 * 1024 { return Err(Error::Budget("intersection surface boundaries")); }
                                boundaries.try_reserve(1).map_err(|_| Error::Budget("intersection surface boundaries"))?;
                                boundaries.push((index, curve));
                            }
                        }
                    }
                }
                let mut curves = Vec::new();
                for (index, occurrence) in occurrences.iter().enumerate() {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    if let Some(edges) = &occurrence.edges {
                        for curve in &edges.curves { flatten(curve, occurrence, None, 0, &limits, &mut curves)?; }
                    } else {
                        match &document_entity(&document, &occurrence.source)?.geometry {
                            Geometry::Curve(curve) => flatten(curve, occurrence, None, 0, &limits, &mut curves)?,
                            Geometry::Surface(_) => for (_, curve) in boundaries.iter().filter(|(owner, _)| *owner == index) {
                                flatten(curve, occurrence, None, 0, &limits, &mut curves)?;
                            },
                            _ => return Err(Error::Stale("intersection source geometry")),
                        }
                    }
                }
                let mut filter_error = None;
                curves.retain(|segment| {
                    if worker_cancel.load(Ordering::Acquire) { filter_error = Some(Error::Canceled); return false; }
                    match curve_may_hit(segment.curve, segment.occurrence.transform, &input) {
                        Ok(keep) => keep,
                        Err(error) => { filter_error = Some(error); false }
                    }
                });
                if let Some(error) = filter_error { return Err(error); }
                let intersections = input.settings.is_effective(om9_scene::osnap::SnapKind::Intersection);
                let pairs = if intersections {
                    curves.len().checked_mul(curves.len().saturating_sub(1)).and_then(|n| n.checked_div(2))
                        .ok_or(Error::Overflow("intersection pairs"))?
                } else { 0 };
                let guides = input.guides.iter().flatten().count();
                let track_pairs = curves.len().checked_mul(guides).ok_or(Error::Overflow("SmartTrack curve pairs"))?;
                if pairs.checked_add(track_pairs).is_none_or(|work| work > 128) {
                    return Err(Error::Budget("intersection and SmartTrack pair work"));
                }
                if pairs > 128 { return Err(Error::Budget("intersection pair work")); }
                let mut pair_limits = limits;
                pair_limits.max_outputs = pair_limits.max_outputs.min(256);
                pair_limits.max_bytes = pair_limits.max_bytes.min(8 * 1024 * 1024);
                let apparent = input.settings.apparent_intersections && intersections;
                if apparent {
                    curves.sort_by_key(|segment| std::cmp::Reverse(
                        document.entities.get(&segment.occurrence.source.root).and_then(|entity| entity.creation_order),
                    ));
                }
                let mut candidates = Vec::new();
                let mut overlap = None;
                let mut accepted = None;
                let record_intersections = |hits: Vec<SnapCandidate>, candidates: &mut Vec<SnapCandidate>,
                    accepted: &mut Option<SnapCandidate>| -> Result<()> {
                    if !apparent {
                        if candidates.len().saturating_add(hits.len()) > 1024 {
                            return Err(Error::Budget("intersection snap candidates"));
                        }
                        candidates.try_reserve(hits.len()).map_err(|_| Error::Budget("intersection snap candidates"))?;
                        candidates.extend(hits);
                        return Ok(());
                    }
                    for hit in hits {
                        if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                        if accepted.as_ref().is_some_and(|best| {
                            !om9_scene::osnap::intersection_pick_is_closer(&hit, best, input.cursor)
                        }) {
                            continue;
                        }
                        if let Some(occlusion) = &occlusion {
                            if occlusion.edge_occluded(&input.camera, input.viewport, &hit)? { continue; }
                        }
                        *accepted = Some(hit);
                    }
                    Ok(())
                };
                let mut pair_work = pairs + track_pairs;
                for index in 0..curves.len() {
                    let segment = curves[index];
                    let IntersectionSegment { curve, occurrence, .. } = segment;
                    for guide in input.guides.iter().flatten() {
                        if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                        let line = guide_segment(curve, occurrence.transform, *guide, tolerance)?;
                        let mut hits = match om9_scene::osnap::source_curve_intersection_candidates(curve, occurrence.transform,
                            &occurrence.source, &line, Transform::identity(), &occurrence.source,
                            &input.camera, input.viewport, input.cursor, input.settings.radius_px, tolerance,
                            &pair_limits, &|| worker_cancel.load(Ordering::Acquire)) {
                            // A coincident guide has no isolated intersection; other snap kinds
                            // and other guide/curve pairs are still valid.
                            Err(Error::Unsupported("overlapping line segments")) => continue,
                            result => result?,
                        };
                        segment.map_parameters(&mut hits);
                        for hit in &mut hits { hit.kind = om9_scene::osnap::SnapKind::SmartTrack; hit.second_source = None; }
                        if candidates.len().saturating_add(hits.len()) > 1024 { return Err(Error::Budget("SmartTrack curve candidates")); }
                        candidates.try_reserve(hits.len()).map_err(|_| Error::Budget("SmartTrack curve candidates"))?;
                        candidates.extend(hits);
                    }
                    if !intersections { continue; }
                    let mut self_hits = self_candidates(curve, occurrence, &input, tolerance, &pair_limits,
                        &worker_cancel, &mut pair_work)?;
                    segment.map_parameters(&mut self_hits);
                    record_intersections(self_hits, &mut candidates, &mut accepted)?;
                    for other in index + 1..curves.len() {
                        if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                        let second = curves[other];
                        let b = second.occurrence;
                        let mut hits = match om9_scene::osnap::source_curve_intersection_candidates(curve, occurrence.transform, &occurrence.source,
                            second.curve, b.transform, &b.source, &input.camera, input.viewport, input.cursor,
                            input.settings.radius_px, tolerance, &pair_limits, &|| worker_cancel.load(Ordering::Acquire)) {
                            Err(error @ Error::Unsupported("overlapping line segments" | "overlapping line and NURBS"
                                | "line and NURBS may overlap" | "overlapping NURBS curves")) => {
                                overlap.get_or_insert(error);
                                continue;
                            }
                            result => result?,
                        };
                        if apparent && accepted.is_none() && hits.is_empty() {
                            hits = om9_scene::osnap::apparent_curve_intersections_with_parameter_map(
                                (curve, occurrence.transform, &occurrence.source), (second.curve, b.transform, &b.source),
                                &input.camera, input.viewport, input.cursor, input.settings.radius_px,
                                tolerance, &pair_limits, &|| worker_cancel.load(Ordering::Acquire),
                                &|source, parameter| [segment, second][source].source_parameter(parameter),
                            )?;
                        } else {
                            segment.map_parameters(&mut hits);
                        }
                        record_intersections(hits, &mut candidates, &mut accepted)?;
                    }
                }
                drop(curves);
                if let Some(occlusion) = &occlusion {
                    let mut failure = None;
                    candidates.retain(|candidate| {
                        if failure.is_some() { return false; }
                        if worker_cancel.load(Ordering::Acquire) { failure = Some(Error::Canceled); return false; }
                        match occlusion.edge_occluded(&input.camera, input.viewport, candidate) {
                            Ok(hidden) => !hidden,
                            Err(error) => { failure = Some(error); false }
                        }
                    });
                    if let Some(error) = failure { return Err(error); }
                }
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                if let Some(hit) = accepted {
                    candidates.try_reserve(1).map_err(|_| Error::Budget("intersection snap candidates"))?;
                    candidates.push(hit);
                }
                if candidates.is_empty() && let Some(error) = overlap {
                    return Err(error);
                }
                // Keep the entire admission while the receipt is live; it covers
                // the retained vector, identities and accounted solver allowance.
                reservation.set_allocated_bytes(required)?;
                Ok(IntersectionReceipt { occurrences, input, tolerance, selection_revision, preview_revision,
                    candidates, canceled: Arc::clone(&worker_cancel), _reservation: reservation })
            })).unwrap_or_else(|_| Err(Error::Io("intersection snap worker panicked".into())));
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("intersection worker: {error}")))?;
        Ok(Self { canceled, receiver: Some(receiver), delivered: false })
    }
    pub fn cancel(&self) { self.canceled.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<IntersectionReceipt>> {
        match self.receiver.as_ref()?.try_recv() {
            Ok(result) => { self.receiver = None; self.delivered = true; Some(result) },
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => { self.receiver = None; Some(Err(Error::Io("intersection worker disconnected".into()))) }
        }
    }
}
impl Drop for IntersectionJob { fn drop(&mut self) { if !self.delivered { self.cancel(); } } }

fn same_occurrences(left: &[IntersectionOccurrence], right: &[IntersectionOccurrence]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| {
        a.source == b.source && a.transform == b.transform
            && match (&a.edges, &b.edges) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
    })
}

#[derive(Default)]
pub(crate) struct IntersectionScheduler {
    active: Option<(Vec<IntersectionOccurrence>, EdgeSnapViewInput, IntersectionJob)>,
    pending: Option<(Vec<IntersectionOccurrence>, EdgeSnapViewInput, ExternalMemoryReservation)>,
    ready: Option<IntersectionReceipt>,
    obsolete: bool,
}

impl IntersectionScheduler {
    pub fn matches(&self, controller: &Controller, occurrences: &[IntersectionOccurrence], input: &EdgeSnapViewInput) -> bool {
        self.ready.as_ref().is_some_and(|receipt| same_occurrences(&receipt.occurrences, occurrences)
            && receipt.resolve(controller, input).is_ok())
            || self.pending.as_ref().is_some_and(|(old, old_input, _)| same_occurrences(old, occurrences) && old_input == input)
            || !self.obsolete && self.pending.is_none() && self.active.as_ref()
                .is_some_and(|(old, old_input, _)| same_occurrences(old, occurrences) && old_input == input)
    }

    pub fn request(&mut self, controller: &Controller, occurrences: Vec<IntersectionOccurrence>,
        input: EdgeSnapViewInput, reservation: ExternalMemoryReservation) -> Result<()> {
        if self.matches(controller, &occurrences, &input) { return Ok(()); }
        self.ready = None;
        self.pending = Some((occurrences, input, reservation));
        if let Some((_, _, active)) = &self.active {
            active.cancel();
            self.obsolete = true;
        } else { self.launch(controller)?; }
        Ok(())
    }

    fn launch(&mut self, controller: &Controller) -> Result<()> {
        if self.active.is_some() { return Ok(()); }
        let Some((occurrences, input, reservation)) = self.pending.take() else { return Ok(()) };
        let job = IntersectionJob::start(controller, occurrences.clone(), input.clone(), reservation)?;
        self.active = Some((occurrences, input, job));
        self.obsolete = false;
        Ok(())
    }

    pub fn poll(&mut self, controller: &Controller) -> Result<()> {
        let result = self.active.as_mut().and_then(|(_, _, job)| job.try_result());
        if let Some(result) = result {
            self.active = None;
            if !self.obsolete {
                let receipt = result?;
                receipt.resolve(controller, &receipt.input)?;
                self.ready = Some(receipt);
            }
            self.obsolete = false;
        }
        self.launch(controller)
    }

    pub fn ready<'a>(&'a self, controller: &Controller, occurrences: &[IntersectionOccurrence], input: &EdgeSnapViewInput)
        -> Option<&'a [SnapCandidate]> {
        let receipt = self.ready.as_ref().filter(|receipt| same_occurrences(&receipt.occurrences, occurrences))?;
        receipt.resolve(controller, input).ok()
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.ready = None;
        if let Some((_, _, active)) = &self.active { active.cancel(); self.obsolete = true; }
    }
    pub fn is_busy(&self) -> bool { self.active.is_some() || self.pending.is_some() }
}
