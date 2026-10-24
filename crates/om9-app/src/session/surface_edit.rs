// SPDX-License-Identifier: LGPL-2.1-or-later
//! Immediate selected-surface edit commands: `Rebuild` and `Unroll Surface`.
//!
//! These commands need no interactive points. They read exactly one selected
//! `Geometry::Surface`, call the owning `om9_core::surface_ops` service and return
//! the replacement geometry; the session commits it as one transaction.
use om9_core::{
    budget::Limits, error::Error, geometry::Geometry, identity::ObjectId, model::Document,
};

pub const REBUILD: &str = "OM9-SURFACE-006";
pub const UNROLL: &str = "OM9-SURFACE-134";
pub const OFFSET: &str = "OM9-SURFACE-025";
pub const EXTRUDE_NORMAL: &str = "OM9-SURFACE-133";
pub const PLANE_CORNERS: &str = "OM9-SURFACE-015";
pub const BLEND: &str = "OM9-SURFACE-007";
pub const LOFT: &str = "OM9-SURFACE-109";
pub const NETWORK: &str = "OM9-SURFACE-010";
pub const SWEEP1: &str = "OM9-SURFACE-101";
pub const SWEEP2: &str = "OM9-SURFACE-003";
pub const PROFILE_SWEEP: &str = "OM9-SURFACE-005";
pub const RAIL_REVOLVE: &str = "OM9-SURFACE-019";
pub const MATCH: &str = "OM9-SURFACE-128";
pub const SEAM: &str = "OM9-SURFACE-038";
pub const SOFT_EDIT: &str = "OM9-SURFACE-037";
pub const TWEEN: &str = "OM9-SURFACE-027";
pub const EXTEND: &str = "OM9-SURFACE-033";
pub const SWEEP_MULTI: &str = "OM9-SURFACE-032";
pub const MERGE: &str = "OM9-SURFACE-127";
pub const PATCH: &str = "OM9-SURFACE-011";
pub const CUTTING_PLANE: &str = "OM9-SURFACE-039";
pub const SPLIT_ISO: &str = "OM9-SURFACE-034";
pub const DRAPE: &str = "OM9-SURFACE-030";
pub const TANGENT: &str = "OM9-SURFACE-040";
pub const EXTRUDE_ALL: &str = "OM9-SURFACE-117";
pub const EDGE_CURVES: &str = "OM9-SURFACE-014";
pub const PLANAR_CURVES: &str = "OM9-SURFACE-013";
pub const VARIABLE_OFFSET: &str = "OM9-SURFACE-026";
pub const SHRINK: &str = "OM9-SURFACE-012";
pub const SWEEP1_HISTORY: &str = "OM9-SURFACE-102";
pub const SWEEP2_HISTORY: &str = "OM9-SURFACE-104";
#[allow(dead_code)]
pub const SWEEP4: &str = "OM9-SURFACE-119";
pub const FOUR_RAIL_PROFILE: &str = "OM9-TOOLS-004";
pub const VARIABLE_BLEND: &str = "OM9-SURFACE-008";
pub const UNTRIM: &str = "OM9-SURFACE-017";
pub const CHAMFER: &str = "OM9-SURFACE-023";
pub const FILLET: &str = "OM9-SURFACE-020";
pub const VARIABLE_FILLET: &str = "OM9-SURFACE-022";
pub const VARIABLE_CHAMFER: &str = "OM9-SURFACE-024";

pub fn handles(feature: &str) -> bool {
    matches!(feature, REBUILD | UNROLL)
}

/// Commands that combine selected curves into one surface.
pub fn handles_curves(feature: &str) -> bool {
    matches!(
        feature,
        BLEND
            | LOFT
            | NETWORK
            | PATCH
            | SWEEP1
            | SWEEP2
            | PROFILE_SWEEP
            | RAIL_REVOLVE
            | EDGE_CURVES
    )
}

/// Commands that need a profile, a rail and one axis prompt.
pub fn handles_curve_pair(feature: &str) -> bool {
    feature == RAIL_REVOLVE
}

/// Commands that need two surfaces and an edge prompt.
pub fn handles_surface_pair(feature: &str) -> bool {
    feature == MATCH
}

/// Commands that close a surface seam at the document tolerance.
pub fn handles_seam(feature: &str) -> bool {
    feature == SEAM
}

/// Commands that move poles near one UV point with a smooth falloff.
pub fn handles_soft_edit(feature: &str) -> bool {
    feature == SOFT_EDIT
}

/// Commands that tween two surfaces.
pub fn handles_tween(feature: &str) -> bool {
    feature == TWEEN
}

/// Commands that extend one surface boundary.
pub fn handles_extend(feature: &str) -> bool {
    feature == EXTEND
}

/// Commands that sweep multiple profiles along the first selected rail.
pub fn handles_sweep_multi(feature: &str) -> bool {
    feature == SWEEP_MULTI
}

/// Commands that merge two surfaces sharing a boundary.
pub fn handles_merge(feature: &str) -> bool {
    feature == MERGE
}

/// Commands that build a plane/rectangle from three points.
pub fn plane_like(feature: &str) -> bool {
    matches!(feature, PLANE_CORNERS | CUTTING_PLANE)
}

/// Commands that split a surface at one isocurve.
pub fn handles_split(feature: &str) -> bool {
    feature == SPLIT_ISO
}

/// Commands that drape a surface over selected points.
pub fn handles_drape(feature: &str) -> bool {
    feature == DRAPE
}

/// Commands that set one edge's tangent direction.
pub fn handles_tangent(feature: &str) -> bool {
    feature == TANGENT
}

/// Commands that extrude all selected curves.
pub fn handles_extrude_all(feature: &str) -> bool {
    feature == EXTRUDE_ALL
}

/// Commands that build a planar surface from coplanar curves.
pub fn handles_planar(feature: &str) -> bool {
    feature == PLANAR_CURVES
}

/// Commands that offset a surface by a distance varying along one parameter.
pub fn handles_variable_offset(feature: &str) -> bool {
    feature == VARIABLE_OFFSET
}

/// Commands that shrink a surface to a UV sub-domain.
pub fn handles_shrink(feature: &str) -> bool {
    feature == SHRINK
}

/// Commands that blend two curves with variable tangent scales.
pub fn handles_variable_blend(feature: &str) -> bool {
    feature == VARIABLE_BLEND
}

/// Commands that remove a trim extension.
pub fn handles_untrim(feature: &str) -> bool {
    feature == UNTRIM
}

/// Commands that build a chamfer strip between two curves.
pub fn handles_chamfer(feature: &str) -> bool {
    feature == CHAMFER
}

/// Commands that build a fillet strip between two curves.
pub fn handles_fillet(feature: &str) -> bool {
    matches!(feature, FILLET | VARIABLE_FILLET)
}

/// Commands that build a variable chamfer strip.
pub fn handles_variable_chamfer(feature: &str) -> bool {
    feature == VARIABLE_CHAMFER
}

/// Multi-profile sweep history commands.
pub fn handles_history(feature: &str) -> bool {
    matches!(feature, SWEEP1_HISTORY | SWEEP2_HISTORY)
}

/// Four-rail Sweep 4.
pub fn handles_sweep4(feature: &str) -> bool {
    feature == SWEEP4
}

/// Four Rail Profile metadata command.
pub fn handles_four_rail_profile(feature: &str) -> bool {
    feature == FOUR_RAIL_PROFILE
}

/// Commands that collect one numeric distance after the selected surface.
pub fn handles_numeric(feature: &str) -> bool {
    matches!(feature, OFFSET | EXTRUDE_NORMAL)
}

pub fn label(feature: &str) -> &'static str {
    match feature {
        REBUILD => "Rebuild Surface",
        UNROLL => "Unroll Surface",
        OFFSET => "Offset Surface",
        EXTRUDE_NORMAL => "Extrude Normal to Surface",
        PLANE_CORNERS => "Plane from Corners",
        BLEND => "Blend Surface",
        LOFT => "Loft",
        NETWORK => "Curve Network",
        PATCH => "Patch",
        EDGE_CURVES => "Edge Curves",
        PLANAR_CURVES => "Planar Curves",
        VARIABLE_OFFSET => "Variable Offset Surface",
        SHRINK => "Shrink Trimmed Surface",
        SWEEP1_HISTORY => "Sweep 1 History",
        SWEEP2_HISTORY => "Sweep 2 History",
        SWEEP4 => "Sweep 4",
        VARIABLE_BLEND => "Variable Blend Surfaces",
        UNTRIM => "Untrim",
        CHAMFER => "Chamfer Surfaces",
        FILLET => "Fillet Surfaces",
        VARIABLE_FILLET => "Variable Fillet Surfaces",
        VARIABLE_CHAMFER => "Variable Chamfer Surfaces",
        CUTTING_PLANE => "Cutting Plane",
        SPLIT_ISO => "Split at Isocurve",
        DRAPE => "Drape",
        TANGENT => "Set Surface Tangent",
        EXTRUDE_ALL => "Surface Extrude All",
        SWEEP1 => "Sweep 1",
        SWEEP2 => "Sweep 2",
        PROFILE_SWEEP => "Profile Sweep",
        RAIL_REVOLVE => "Rail Revolve",
        MATCH => "Match Surfaces",
        SEAM => "Adjust Closed Surface Seam",
        SOFT_EDIT => "Soft Edit",
        TWEEN => "Tween Surfaces",
        EXTEND => "Extend Surface",
        SWEEP_MULTI => "Sweep Multi",
        MERGE => "Merge Surfaces",
        _ => "Surface edit",
    }
}

/// Canonical command id for the session event/recent slot.
pub fn static_feature(feature: &str) -> &'static str {
    match feature {
        UNROLL => UNROLL,
        OFFSET => OFFSET,
        EXTRUDE_NORMAL => EXTRUDE_NORMAL,
        PLANE_CORNERS => PLANE_CORNERS,
        BLEND => BLEND,
        LOFT => LOFT,
        NETWORK => NETWORK,
        PATCH => PATCH,
        EDGE_CURVES => EDGE_CURVES,
        PLANAR_CURVES => PLANAR_CURVES,
        VARIABLE_OFFSET => VARIABLE_OFFSET,
        SHRINK => SHRINK,
        SWEEP1_HISTORY => SWEEP1_HISTORY,
        SWEEP2_HISTORY => SWEEP2_HISTORY,
        SWEEP4 => SWEEP4,
        VARIABLE_BLEND => VARIABLE_BLEND,
        UNTRIM => UNTRIM,
        CHAMFER => CHAMFER,
        FILLET => FILLET,
        VARIABLE_FILLET => VARIABLE_FILLET,
        VARIABLE_CHAMFER => VARIABLE_CHAMFER,
        CUTTING_PLANE => CUTTING_PLANE,
        SPLIT_ISO => SPLIT_ISO,
        DRAPE => DRAPE,
        TANGENT => TANGENT,
        EXTRUDE_ALL => EXTRUDE_ALL,
        SWEEP1 => SWEEP1,
        SWEEP2 => SWEEP2,
        PROFILE_SWEEP => PROFILE_SWEEP,
        RAIL_REVOLVE => RAIL_REVOLVE,
        MATCH => MATCH,
        SEAM => SEAM,
        SOFT_EDIT => SOFT_EDIT,
        TWEEN => TWEEN,
        EXTEND => EXTEND,
        SWEEP_MULTI => SWEEP_MULTI,
        MERGE => MERGE,
        _ => REBUILD,
    }
}

/// Apply a numeric selected-surface command.
pub fn apply_numeric(
    feature: &str,
    surface: &om9_core::geometry::NurbsSurface,
    distance_mm: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    if !distance_mm.is_finite() || distance_mm == 0.0 {
        return Err(Error::Invalid("surface distance"));
    }
    match feature {
        OFFSET => {
            om9_core::surface_ops::offset_surface(surface, distance_mm, [8, 8], [3, 3], limits)
                .map(Geometry::Surface)
        }
        EXTRUDE_NORMAL => {
            om9_core::surface_ops::normal_extrude(surface, distance_mm, [8, 8], [3, 3], limits)
                .map(Geometry::Surface)
        }
        _ => Err(Error::Unsupported("surface numeric edit")),
    }
}

/// Build the exact planar surface through three points. The first point is the
/// origin; the second fixes +X and the width; the third fixes +Y and the height.
/// Coincident or collinear corners are refused.
pub fn plane_from_corners(
    points: [om9_core::geometry::Point3; 3],
    limits: &Limits,
) -> Result<om9_core::geometry::NurbsSurface, Error> {
    let [origin, second, third] = points;
    let v1 = [
        second[0] - origin[0],
        second[1] - origin[1],
        second[2] - origin[2],
    ];
    let width = v1[0].hypot(v1[1]).hypot(v1[2]);
    if !width.is_finite() || width <= 1e-12 {
        return Err(Error::Invalid("plane corner width"));
    }
    let x = [v1[0] / width, v1[1] / width, v1[2] / width];
    let v2 = [
        third[0] - origin[0],
        third[1] - origin[1],
        third[2] - origin[2],
    ];
    let projection = v2[0] * x[0] + v2[1] * x[1] + v2[2] * x[2];
    let y_raw = [
        v2[0] - x[0] * projection,
        v2[1] - x[1] * projection,
        v2[2] - x[2] * projection,
    ];
    let height = y_raw[0].hypot(y_raw[1]).hypot(y_raw[2]);
    if !height.is_finite() || height <= 1e-12 {
        return Err(Error::Invalid("plane corner height"));
    }
    let y = [y_raw[0] / height, y_raw[1] / height, y_raw[2] / height];
    om9_core::surface_kernel::plane(origin, x, y, width, height, limits)
}

/// Combine exactly two selected curves into a surface. `Blend` is G0 (ruled)
/// and `Loft` interpolates the two sections. Selection order is the stable
/// `BTreeSet` order; two-section results are order-insensitive for geometry.
pub fn apply_curves(
    feature: &str,
    document: &Document,
    selection: &[ObjectId],
    limits: &Limits,
) -> Result<Geometry, Error> {
    let curves: Vec<om9_core::geometry::Curve> = selection
        .iter()
        .filter_map(|id| document.entities.get(id))
        .filter_map(|entity| match &entity.geometry {
            Geometry::Curve(curve) => Some(curve.clone()),
            _ => None,
        })
        .collect();
    match feature {
        LOFT if curves.len() >= 2 => {
            om9_core::surface_kernel::loft(&curves, [3, 1], 16, limits).map(Geometry::Surface)
        }
        BLEND if curves.len() == 2 => om9_core::surface_ops::blend_surface(
            &curves[0],
            &curves[1],
            om9_core::surface_ops::SurfaceContinuity::G0,
            None,
            [16, 8],
            [3, 2],
            limits,
        )
        .map(Geometry::Surface),
        NETWORK | PATCH if curves.len() == 4 => om9_core::surface_ops::network(
            [&curves[0], &curves[1], &curves[2], &curves[3]],
            [3, 3],
            [16, 16],
            limits,
        )
        .map(Geometry::Surface),
        EDGE_CURVES if curves.len() == 4 => om9_core::surface_ops::network(
            [&curves[0], &curves[1], &curves[2], &curves[3]],
            [3, 3],
            [16, 16],
            limits,
        )
        .map(Geometry::Surface),
        EDGE_CURVES if curves.len() == 2 => om9_core::surface_ops::blend_surface(
            &curves[0],
            &curves[1],
            om9_core::surface_ops::SurfaceContinuity::G0,
            None,
            [16, 8],
            [3, 2],
            limits,
        )
        .map(Geometry::Surface),
        SWEEP1 | PROFILE_SWEEP if curves.len() == 2 => {
            om9_core::surface_kernel::sweep1(&curves[0], &curves[1], [17, 33], [3, 3], limits)
                .map(Geometry::Surface)
        }
        SWEEP2 if curves.len() == 3 => om9_core::surface_kernel::sweep2(
            &curves[0],
            &curves[1],
            &curves[2],
            om9_core::surface_kernel::Sweep2Options {
                maintain_height: true,
                ..Default::default()
            },
            limits,
        )
        .map(Geometry::Surface),
        _ => Err(Error::Invalid("curve selection for surface command")),
    }
}

/// Revolve a profile along a rail around an explicit axis.
pub fn apply_rail_revolve(
    profile: &om9_core::geometry::Curve,
    rail: &om9_core::geometry::Curve,
    axis_origin: om9_core::geometry::Point3,
    axis_direction: om9_core::geometry::Point3,
    limits: &Limits,
) -> Result<Geometry, Error> {
    let length = axis_direction[0]
        .hypot(axis_direction[1])
        .hypot(axis_direction[2]);
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("rail revolve axis"));
    }
    om9_core::surface_ops::rail_revolve(
        profile,
        rail,
        axis_origin,
        axis_direction,
        [17, 33],
        [3, 3],
        limits,
    )
    .map(Geometry::Surface)
}

/// Extend one boundary of a surface by a distance.
pub fn apply_extend(
    surface: &om9_core::geometry::NurbsSurface,
    edge: om9_core::surface_ops::SurfaceEdge,
    distance_mm: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::extend_surface(surface, edge, distance_mm, limits).map(Geometry::Surface)
}

/// Interpolate a new surface between two selected surfaces.
pub fn apply_tween(
    first: &om9_core::geometry::NurbsSurface,
    second: &om9_core::geometry::NurbsSurface,
    fraction: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::tween_surface(first, second, fraction, [9, 9], [3, 3], limits)
        .map(Geometry::Surface)
}

/// Move the surface poles near one UV point by `delta`.
pub fn apply_soft_edit(
    surface: &om9_core::geometry::NurbsSurface,
    at: [f64; 2],
    radius: [f64; 2],
    delta: om9_core::geometry::Point3,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::softedit(surface, at, delta, radius, limits).map(Geometry::Surface)
}

/// Parse `U` or `V` for a surface seam.
pub fn parse_iso(text: &str) -> Result<om9_core::surface_ops::SurfaceIso, Error> {
    match text.trim().to_ascii_uppercase().as_str() {
        "U" => Ok(om9_core::surface_ops::SurfaceIso::U),
        "V" => Ok(om9_core::surface_ops::SurfaceIso::V),
        _ => Err(Error::Invalid("surface seam axis")),
    }
}

/// Close the U or V seam at the given tolerance.
pub fn apply_seam(
    surface: &om9_core::geometry::NurbsSurface,
    iso: om9_core::surface_ops::SurfaceIso,
    tolerance_mm: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
        return Err(Error::Invalid("surface seam tolerance"));
    }
    om9_core::surface_ops::seam_surface(surface, iso, tolerance_mm, limits).map(Geometry::Surface)
}

/// Parse `U0`, `U1`, `V0` or `V1` for a surface-match edge.
pub fn parse_surface_edge(text: &str) -> Result<om9_core::surface_ops::SurfaceEdge, Error> {
    match text.trim().to_ascii_uppercase().as_str() {
        "U0" => Ok(om9_core::surface_ops::SurfaceEdge::U0),
        "U1" => Ok(om9_core::surface_ops::SurfaceEdge::U1),
        "V0" => Ok(om9_core::surface_ops::SurfaceEdge::V0),
        "V1" => Ok(om9_core::surface_ops::SurfaceEdge::V1),
        _ => Err(Error::Invalid("surface edge")),
    }
}

/// Match `source` at `source_edge` to `target` at `target_edge` at G0.
pub fn apply_match(
    source: &om9_core::geometry::NurbsSurface,
    target: &om9_core::geometry::NurbsSurface,
    source_edge: om9_core::surface_ops::SurfaceEdge,
    target_edge: om9_core::surface_ops::SurfaceEdge,
    limits: &Limits,
) -> Result<Geometry, Error> {
    let report = om9_core::surface_ops::match_surface(
        source,
        source_edge,
        target,
        target_edge,
        om9_core::surface_ops::SurfaceContinuity::G0,
        limits,
    )?;
    Ok(Geometry::Surface(report.surface))
}

/// Build a circular fillet strip between two curves.
pub fn apply_fillet(
    first: &om9_core::geometry::Curve,
    second: &om9_core::geometry::Curve,
    start_radius: f64,
    end_radius: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::fillet_strip(
        first,
        second,
        start_radius,
        end_radius,
        [16, 9],
        [3, 3],
        limits,
    )
    .map(Geometry::Surface)
}

/// Build a chamfer strip with linearly varying setback.
pub fn apply_variable_chamfer(
    first: &om9_core::geometry::Curve,
    second: &om9_core::geometry::Curve,
    start_setback: f64,
    end_setback: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::variable_chamfer_strip(
        first,
        second,
        start_setback,
        end_setback,
        [16, 8],
        [3, 2],
        limits,
    )
    .map(Geometry::Surface)
}

/// Build a chamfer strip between two curves at a setback.
pub fn apply_chamfer(
    first: &om9_core::geometry::Curve,
    second: &om9_core::geometry::Curve,
    setback_mm: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::chamfer_strip(first, second, setback_mm, [16, 8], [3, 2], limits)
        .map(Geometry::Surface)
}

/// Remove the trim extension from an entity and return the untrimmed copy.
pub fn untrim_entity(
    entity: &om9_core::model::Entity,
    limits: &Limits,
) -> Result<om9_core::model::Entity, Error> {
    if om9_core::rhino_semantics::trimmed_surface_of(entity, limits)?.is_none() {
        return Err(Error::Invalid("surface is not trimmed"));
    }
    let mut untrimmed = om9_core::model::Entity::new(
        entity.label.clone(),
        entity.geometry.clone(),
        entity.attributes.layer,
    );
    untrimmed.attributes = entity.attributes.clone();
    untrimmed.placement = entity.placement;
    Ok(untrimmed)
}

/// Blend two curves with variable tangent scales.
pub fn apply_variable_blend(
    first: &om9_core::geometry::Curve,
    second: &om9_core::geometry::Curve,
    start_scale: f64,
    end_scale: f64,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::variable_blend_surface(
        first,
        second,
        start_scale,
        end_scale,
        [16, 8],
        [3, 2],
        limits,
    )
    .map(Geometry::Surface)
}

/// Shrink a surface to a UV sub-domain.
pub fn apply_shrink(
    surface: &om9_core::geometry::NurbsSurface,
    u: [f64; 2],
    v: [f64; 2],
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::shrink_surface_domain(surface, u, v, [12, 12], [3, 3], limits)
        .map(Geometry::Surface)
}

/// Offset a surface by a distance varying from start to end.
pub fn apply_variable_offset(
    surface: &om9_core::geometry::NurbsSurface,
    start_mm: f64,
    end_mm: f64,
    axis: om9_core::surface_ops::SurfaceIso,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::variable_offset_surface(
        surface,
        start_mm,
        end_mm,
        axis,
        [8, 8],
        [3, 3],
        limits,
    )
    .map(Geometry::Surface)
}

/// Build a planar surface spanning selected coplanar curves.
pub fn apply_planar_curves(
    curves: &[om9_core::geometry::Curve],
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::planar_surface_from_curves(curves, limits).map(Geometry::Surface)
}

/// Extrude every selected curve along +Z by a height.
pub fn apply_extrude_all(
    curves: &[om9_core::geometry::Curve],
    height_mm: f64,
    limits: &Limits,
) -> Result<Vec<Geometry>, Error> {
    om9_core::surface_ops::extrude_curves(curves, height_mm, limits)
        .map(|surfaces| surfaces.into_iter().map(Geometry::Surface).collect())
}

/// Set the cross-boundary tangent on one surface edge.
pub fn apply_tangent(
    surface: &om9_core::geometry::NurbsSurface,
    edge: om9_core::surface_ops::SurfaceEdge,
    direction: om9_core::geometry::Point3,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::set_surface_tangent(surface, edge, direction, limits)
        .map(Geometry::Surface)
}

/// Drape a surface over selected point positions.
pub fn apply_drape(
    points: &[om9_core::geometry::Point3],
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::drape_points(points, [12; 2], [3, 3], limits).map(Geometry::Surface)
}

/// Split a surface at an interior isocurve into two independent half-surfaces.
pub fn apply_split(
    surface: &om9_core::geometry::NurbsSurface,
    iso: om9_core::surface_ops::SurfaceIso,
    parameter: f64,
    limits: &Limits,
) -> Result<Vec<Geometry>, Error> {
    let (first, second) = om9_core::surface_ops::split_surface(surface, iso, parameter, limits)?;
    let first = om9_core::surface_ops::shrink_surface(&first, [3, 3], [8, 8], limits)?.underlying;
    let second = om9_core::surface_ops::shrink_surface(&second, [3, 3], [8, 8], limits)?.underlying;
    Ok(vec![Geometry::Surface(first), Geometry::Surface(second)])
}

/// Merge two surfaces sharing a boundary.
pub fn apply_merge(
    first: &om9_core::geometry::NurbsSurface,
    second: &om9_core::geometry::NurbsSurface,
    limits: &Limits,
) -> Result<Geometry, Error> {
    om9_core::surface_ops::merge_surfaces(first, second, limits).map(Geometry::Surface)
}

/// Sweep every profile after the first selected curve along that first rail.
pub fn apply_sweep_multi(
    rail: &om9_core::geometry::Curve,
    profiles: &[om9_core::geometry::Curve],
    limits: &Limits,
) -> Result<Vec<Geometry>, Error> {
    if profiles.is_empty() {
        return Err(Error::Invalid("sweep multi profiles"));
    }
    profiles
        .iter()
        .map(|profile| {
            om9_core::surface_kernel::sweep1(profile, rail, [17, 33], [3, 3], limits)
                .map(Geometry::Surface)
        })
        .collect()
}

/// Apply the command to the single selected surface. Missing selection, a
/// non-surface selection and a service refusal are explicit errors; nothing is
/// mutated here.
pub fn apply(
    feature: &str,
    document: &Document,
    selection: &std::collections::BTreeSet<ObjectId>,
    limits: &Limits,
) -> Result<Geometry, Error> {
    let id = selection
        .iter()
        .next()
        .ok_or(Error::Missing("surface selection"))?;
    let entity = document
        .entities
        .get(id)
        .ok_or(Error::Missing("selected surface"))?;
    let Geometry::Surface(surface) = &entity.geometry else {
        return Err(Error::Invalid("surface selection"));
    };
    match feature {
        REBUILD => om9_core::surface_ops::rebuild_surface(surface, [3, 3], [8, 8], limits)
            .map(Geometry::Surface),
        UNROLL => om9_core::surface_ops::unroll(surface, [8, 8], limits).map(Geometry::Surface),
        _ => Err(Error::Unsupported("surface edit")),
    }
}
