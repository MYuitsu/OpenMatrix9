// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for gems on curves and surfaces (`OM9-L2-04-10`, L3-001..011).
//!
//! Every assertion runs against the public `om9_core::gem_placement` surface only, so the
//! evidence is about the owner contract and not about private helpers. The curve/surface
//! evaluators are the shared ones (`curve_ops`, `surface_kernel`), which is exactly the
//! parent-module exclusion rule: no second kernel inside the placement family.

use om9_core::{
    budget::Limits,
    curve_build::circle_through,
    curve_ops::arc_length,
    gem::GemSpec,
    gem_placement::{
        CurveParams, GemStation, LinkState, PlacementLink, PlacementSource, SplopParams,
        SurfaceParams, count_on_curve, flow_placement, flow_surface_placement, follow_placement,
        place_on_curve, place_on_curve_variable, place_on_curves, place_on_surface,
        profile_and_guides, splop_station, taper_specs,
    },
    geometry::{Curve, Geometry, NurbsSurface, Point3},
    identity::ObjectId,
    model::{
        Document, Entity, Extension, GEM_PLACEMENT_EXTENSION_SLOT, extension_slot_known,
    },
};

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

fn round_gem(diameter: f64, depth: f64) -> GemSpec {
    GemSpec::round(diameter, depth).unwrap()
}

fn plane_z(z: f64) -> NurbsSurface {
    let limits = Limits::default();
    om9_core::surface_kernel::plane([0.0, 0.0, z], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 100.0, 100.0, &limits)
        .unwrap()
}

/// A surface whose V tangent vanishes everywhere: it has no normal.
fn degenerate_surface() -> NurbsSurface {
    NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

#[test]
fn even_spacing_uses_arc_length_on_a_straight_curve() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let placement = place_on_curve(
        &curve,
        &round_gem(6.0, 3.9),
        &CurveParams::spacing(10.0),
        &limits,
    )
    .unwrap();
    assert_eq!(placement.count(), 11);
    for (index, station) in placement.stations.iter().enumerate() {
        assert_eq!(station.index, index);
        assert!((station.arc_length_mm - index as f64 * 10.0).abs() <= 1e-9);
        assert!(close(station.point, [index as f64 * 10.0, 0.0, 0.0], 1e-9));
        assert!(station.frame_is_orthonormal(), "station {index}");
    }
    // Upright on a horizontal line: the table faces world +Z.
    assert!(close(placement.stations[0].axis, [0.0, 0.0, 1.0], 1e-12));
    for gap in &placement.gaps_mm {
        assert!((gap - 10.0).abs() <= 1e-9);
    }
}

#[test]
fn start_and_end_bounds_limit_the_row() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let params = CurveParams::spacing(10.0).with_range(20.0, Some(60.0));
    let placement = place_on_curve(&curve, &round_gem(6.0, 3.9), &params, &limits).unwrap();
    assert_eq!(placement.count(), 5);
    assert!((placement.stations[0].arc_length_mm - 20.0).abs() <= 1e-9);
    assert!((placement.stations[4].arc_length_mm - 60.0).abs() <= 1e-9);
    assert_eq!(placement.range, [20.0, 60.0]);

    // A degenerate range still hosts the single station at its start.
    let single = CurveParams::spacing(10.0).with_range(95.0, Some(95.0));
    let single = place_on_curve(&curve, &round_gem(6.0, 3.9), &single, &limits).unwrap();
    assert_eq!(single.count(), 1);
    assert!((single.stations[0].arc_length_mm - 95.0).abs() <= 1e-9);
}

#[test]
fn closed_curve_seam_is_not_duplicated() {
    let limits = Limits::default();
    let circle = circle_through([10.0, 0.0, 0.0], [0.0, 10.0, 0.0], [-10.0, 0.0, 0.0]).unwrap();
    let total = arc_length(&circle, &limits).unwrap();
    let quarter = total / 4.0;
    let placement = place_on_curve(
        &circle,
        &round_gem(3.0, 2.0),
        &CurveParams::spacing(quarter),
        &limits,
    )
    .unwrap();
    assert!(placement.closed);
    assert_eq!(placement.count(), 4, "the seam station must be dropped");
    for station in &placement.stations {
        assert!(station.arc_length_mm < total - 1e-6);
    }
    // The seam gap closes the ring, so the row stays evenly spaced across the seam.
    let seam_gap = placement.gaps_mm.last().copied().unwrap();
    assert!((seam_gap - quarter).abs() <= 1e-6);
    assert!((placement.min_gap_mm().unwrap() - quarter).abs() <= 1e-6);
}

#[test]
fn a_stone_longer_than_the_spacing_is_refused_unless_overlap_is_requested() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let spec = round_gem(6.0, 3.9);
    let refused = place_on_curve(&curve, &spec, &CurveParams::spacing(4.0), &limits);
    assert!(refused.is_err(), "6 mm stone on 4 mm pitch must be refused");
    let allowed = place_on_curve(
        &curve,
        &spec,
        &CurveParams::spacing(4.0).with_overlap(true),
        &limits,
    )
    .unwrap();
    assert_eq!(allowed.count(), 26);
}

#[test]
fn invalid_parameters_are_typed_refusals() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let spec = round_gem(6.0, 3.9);
    assert!(place_on_curve(&curve, &spec, &CurveParams::spacing(0.0), &limits).is_err());
    assert!(place_on_curve(&curve, &spec, &CurveParams::spacing(-2.0), &limits).is_err());
    assert!(
        place_on_curve(
            &curve,
            &spec,
            &CurveParams::spacing(10.0).with_range(-1.0, None),
            &limits
        )
        .is_err()
    );
    assert!(
        place_on_curve(
            &curve,
            &spec,
            &CurveParams::spacing(10.0).with_range(0.0, Some(150.0)),
            &limits
        )
        .is_err()
    );
    assert!(
        place_on_curve(
            &curve,
            &spec,
            &CurveParams::spacing(10.0).with_rotation(f64::NAN),
            &limits
        )
        .is_err()
    );
    // A curve with no length cannot host a row.
    let point_curve = line([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
    assert!(place_on_curve(&point_curve, &spec, &CurveParams::spacing(10.0), &limits).is_err());
}

#[test]
fn taper_changes_diameter_monotonically_and_keeps_every_stone_valid() {
    let start = round_gem(8.0, 5.2);
    let end = round_gem(4.0, 2.6);
    let specs = taper_specs(&start, &end, 5).unwrap();
    assert_eq!(specs.len(), 5);
    assert!((specs[0].length_mm - 8.0).abs() <= 1e-12);
    assert!((specs[4].length_mm - 4.0).abs() <= 1e-12);
    for pair in specs.windows(2) {
        assert!(pair[0].length_mm > pair[1].length_mm);
        assert!(pair[0].width_mm > pair[1].width_mm);
        assert!(pair[0].depth_mm() > pair[1].depth_mm());
    }
    assert_eq!(taper_specs(&start, &end, 0).unwrap_err(), om9_core::Error::Invalid("gem taper count"));

    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let placement = place_on_curve_variable(
        &curve,
        &specs,
        &CurveParams::spacing(25.0),
        &limits,
    )
    .unwrap();
    assert_eq!(placement.count(), 5);
    for (station, spec) in placement.stations.iter().zip(specs.iter()) {
        assert_eq!(&station.spec, spec);
    }
}

#[test]
fn variable_list_places_each_stone_in_order() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let specs = vec![round_gem(6.0, 3.9), round_gem(4.0, 2.6), round_gem(4.0, 2.6)];
    let params = CurveParams::spacing(40.0).with_range(10.0, None);
    let placement = place_on_curve_variable(&curve, &specs, &params, &limits).unwrap();
    assert_eq!(placement.count(), 3);
    assert!((placement.stations[0].arc_length_mm - 10.0).abs() <= 1e-9);
    assert!((placement.stations[2].arc_length_mm - 90.0).abs() <= 1e-9);
    for (station, spec) in placement.stations.iter().zip(specs.iter()) {
        assert_eq!(&station.spec, spec);
    }
    // Empty and mismatched lists are refusals, not silent truncation.
    assert!(place_on_curve_variable(&curve, &[], &params, &limits).is_err());
    let mismatched = vec![round_gem(6.0, 3.9), round_gem(6.0, 3.9)];
    assert!(place_on_curve_variable(&curve, &mismatched, &params, &limits).is_err());
}

#[test]
fn count_matches_an_independent_fit_calculation() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let spec = round_gem(6.0, 3.9);
    let fit = count_on_curve(&curve, &spec, 2.0, 0.0, &limits).unwrap();
    assert!(fit.fits);
    assert_eq!(fit.count, 12);
    assert!((fit.spacing_mm - 8.0).abs() <= 1e-12);
    let occupied = fit.count as f64 * spec.length_mm + (fit.count as f64 - 1.0) * 2.0;
    assert!(occupied <= fit.usable_mm + 1e-9);
    assert!((fit.first_center_mm - 3.0).abs() <= 1e-12);
    assert!((fit.last_center_mm - 91.0).abs() <= 1e-12);

    // The same count is reproduced by an actual placement at the reported pitch.
    let placement = place_on_curve(
        &curve,
        &spec,
        &CurveParams::spacing(fit.spacing_mm).with_range(fit.first_center_mm, Some(fit.last_center_mm)),
        &limits,
    )
    .unwrap();
    assert_eq!(placement.count(), fit.count);

    let short = count_on_curve(&curve, &spec, 2.0, 10.0, &limits).unwrap();
    assert!(short.fits);
    assert_eq!(short.count, 10);
    assert!(
        short.count as f64 * spec.length_mm + (short.count as f64 - 1.0) * 2.0
            <= short.usable_mm + 1e-9
    );

    // A curve that cannot hold one stone reports zero instead of a negative count.
    let tiny = line([0.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
    let none = count_on_curve(&tiny, &spec, 1.0, 0.0, &limits).unwrap();
    assert_eq!(none.count, 0);
    assert!(!none.fits);
    assert!(count_on_curve(&curve, &spec, -1.0, 0.0, &limits).is_err());
    assert!(count_on_curve(&curve, &spec, 1.0, -1.0, &limits).is_err());
}

#[test]
fn multi_curve_run_is_atomic() {
    let limits = Limits::default();
    let curves = vec![
        line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]),
        line([0.0, 20.0, 0.0], [0.0, 80.0, 0.0]),
    ];
    let multi = place_on_curves(&curves, &round_gem(6.0, 3.9), &CurveParams::spacing(10.0), &limits)
        .unwrap();
    assert_eq!(multi.groups.len(), 2);
    assert_eq!(multi.groups[0].count(), 11);
    assert_eq!(multi.groups[1].count(), 7);
    assert_eq!(multi.stations(), 18);
    assert!(!multi.is_empty());

    // One bad path fails the whole call: the caller never sees a partial row.
    let broken = vec![
        line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]),
        line([5.0, 5.0, 5.0], [5.0, 5.0, 5.0]),
    ];
    assert!(
        place_on_curves(&broken, &round_gem(6.0, 3.9), &CurveParams::spacing(10.0), &limits).is_err()
    );
    assert!(place_on_curves(&[], &round_gem(6.0, 3.9), &CurveParams::spacing(10.0), &limits).is_err());
}

#[test]
fn surface_frame_is_orthonormal_and_uses_the_surface_normal() {
    let limits = Limits::default();
    let surface = plane_z(0.0);
    let params = SurfaceParams::new(vec![[0.5, 0.5], [0.25, 0.75]]);
    let placement = place_on_surface(&surface, &[round_gem(6.0, 3.9)], &params, &limits).unwrap();
    assert_eq!(placement.count(), 2);
    let first = &placement.stations[0];
    assert!(close(first.point, [50.0, 50.0, 0.0], 1e-9));
    assert!(close(first.axis, [0.0, 0.0, 1.0], 1e-12));
    assert_eq!(first.uv, Some([0.5, 0.5]));
    assert!(first.frame_is_orthonormal());
    assert!(close(placement.stations[1].point, [25.0, 75.0, 0.0], 1e-9));

    let lifted = SurfaceParams {
        uv: vec![[0.5, 0.5]],
        z_offset_mm: 2.0,
        rotation_deg: 0.0,
    };
    let lifted = place_on_surface(&surface, &[round_gem(6.0, 3.9)], &lifted, &limits).unwrap();
    assert!(close(lifted.stations[0].point, [50.0, 50.0, 2.0], 1e-9));
    // The stone itself is untouched by the offset.
    assert!((lifted.stations[0].spec.length_mm - 6.0).abs() <= 1e-12);
}

#[test]
fn surface_parameter_outside_the_domain_is_refused() {
    let limits = Limits::default();
    let surface = plane_z(0.0);
    let spec = round_gem(6.0, 3.9);
    for uv in [[1.5, 0.5], [0.5, -0.2], [f64::NAN, 0.5]] {
        let params = SurfaceParams::new(vec![uv]);
        assert!(
            place_on_surface(&surface, std::slice::from_ref(&spec), &params, &limits).is_err(),
            "uv {uv:?} must be refused"
        );
    }
    assert!(
        place_on_surface(&surface, &[], &SurfaceParams::new(vec![[0.5, 0.5]]), &limits).is_err()
    );
}

#[test]
fn degenerate_surface_has_no_normal_and_is_refused() {
    let limits = Limits::default();
    let surface = degenerate_surface();
    surface.validate(&limits).unwrap();
    let params = SurfaceParams::new(vec![[0.5, 0.5]]);
    assert!(place_on_surface(&surface, &[round_gem(6.0, 3.9)], &params, &limits).is_err());
    assert!(
        splop_station(
            &surface,
            [5.0, 5.0, 5.0],
            &round_gem(6.0, 3.9),
            &SplopParams::default(),
            &limits
        )
        .is_err()
    );
}

#[test]
fn profile_and_guides_are_built_without_touching_the_placement() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [20.0, 0.0, 0.0]);
    let placement =
        place_on_curve(&curve, &round_gem(4.0, 2.6), &CurveParams::spacing(10.0), &limits).unwrap();
    let before = placement.clone();
    let guides = profile_and_guides(&placement, &limits).unwrap();
    assert_eq!(guides.table_points.len(), 3);
    assert_eq!(guides.guides.len(), 3);
    assert!(arc_length(&guides.profile, &limits).unwrap() > 0.0);
    assert!(arc_length(&guides.girdle, &limits).unwrap() > 0.0);
    for guide in &guides.guides {
        assert!(arc_length(guide, &limits).unwrap() > 0.0);
    }
    // The profile sits on the table plane, one stone depth above the girdle line.
    assert!(close(guides.table_points[0], [0.0, 0.0, 2.6], 1e-9));
    // The placement is pure data and was not modified.
    assert_eq!(placement, before);

    // One station cannot describe a profile.
    let single = place_on_curve(
        &curve,
        &round_gem(4.0, 2.6),
        &CurveParams::spacing(100.0).with_range(0.0, Some(0.0)),
        &limits,
    )
    .unwrap();
    assert_eq!(single.count(), 1);
    assert!(profile_and_guides(&single, &limits).is_err());
}

#[test]
fn follow_recomputes_stations_and_reports_shortened_conflicts() {
    let limits = Limits::default();
    let source = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let mut placement =
        place_on_curve(&source, &round_gem(6.0, 3.9), &CurveParams::spacing(20.0), &limits)
            .unwrap();
    assert_eq!(placement.count(), 6);
    let ids: Vec<_> = placement
        .stations
        .iter_mut()
        .map(|station| {
            let gem = om9_core::gem::GemId::new();
            station.gem = Some(gem);
            gem
        })
        .collect();

    // The source is rotated and shortened: three stations remain, three are conflicts.
    let edited = line([0.0, 0.0, 0.0], [0.0, 50.0, 0.0]);
    let outcome = follow_placement(&placement, &edited, &limits).unwrap();
    assert_eq!(outcome.placement.count(), 3);
    assert_eq!(outcome.conflicts.len(), 3);
    assert_eq!(outcome.conflicts[0].station, 3);
    assert_eq!(outcome.conflicts[0].gem, Some(ids[3]));
    assert_eq!(outcome.conflicts[0].reason, "source curve shortened");
    for (index, station) in outcome.placement.stations.iter().enumerate() {
        assert_eq!(station.gem, Some(ids[index]), "identity must survive");
        assert!(close(station.point, [0.0, index as f64 * 20.0, 0.0], 1e-9));
        assert!(outcome.movement_mm[index] >= 0.0);
    }
    assert!(outcome.movement_mm[1] > 1.0, "the row really moved");
}

#[test]
fn follow_on_a_longer_curve_keeps_existing_ids_and_adds_stations() {
    let limits = Limits::default();
    let source = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let mut placement =
        place_on_curve(&source, &round_gem(6.0, 3.9), &CurveParams::spacing(20.0), &limits)
            .unwrap();
    let ids: Vec<_> = placement
        .stations
        .iter_mut()
        .map(|station| {
            let gem = om9_core::gem::GemId::new();
            station.gem = Some(gem);
            gem
        })
        .collect();
    let longer = line([0.0, 0.0, 0.0], [140.0, 0.0, 0.0]);
    let outcome = follow_placement(&placement, &longer, &limits).unwrap();
    assert!(outcome.conflicts.is_empty());
    assert_eq!(outcome.placement.count(), 8);
    for (index, gem) in ids.iter().enumerate() {
        assert_eq!(outcome.placement.stations[index].gem, Some(*gem));
    }
    for station in outcome.placement.stations.iter().skip(ids.len()) {
        assert_eq!(station.gem, None, "an appended station has no identity yet");
        assert_eq!(&station.spec, &outcome.placement.stations[0].spec);
    }
}

#[test]
fn flow_carries_gem_frames_along_the_target_curve() {
    let limits = Limits::default();
    let source = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let target = line([0.0, 0.0, 0.0], [0.0, 100.0, 0.0]);
    let placement =
        place_on_curve(&source, &round_gem(6.0, 3.9), &CurveParams::spacing(25.0), &limits)
            .unwrap();
    let flowed = flow_placement(&source, &target, &placement, &limits).unwrap();
    assert_eq!(flowed.count(), placement.count());
    for (index, station) in flowed.stations.iter().enumerate() {
        assert!(close(station.point, [0.0, index as f64 * 25.0, 0.0], 1e-6));
        // The frame turned with the tangent; the stone kept its size.
        assert!((station.spec.length_mm - 6.0).abs() <= 1e-12);
        assert!(station.frame_is_orthonormal());
    }
    assert!(close(flowed.stations[0].axis, [0.0, 0.0, 1.0], 1e-9));
}

#[test]
fn flow_between_surfaces_keeps_stone_proportions() {
    let limits = Limits::default();
    let base = plane_z(0.0);
    let target = plane_z(50.0);
    let curve = line([10.0, 50.0, 0.0], [90.0, 50.0, 0.0]);
    let placement =
        place_on_curve(&curve, &round_gem(6.0, 3.9), &CurveParams::spacing(20.0), &limits)
            .unwrap();
    let flowed = flow_surface_placement(&base, &target, &placement, &limits).unwrap();
    assert_eq!(flowed.count(), 5);
    for (index, station) in flowed.stations.iter().enumerate() {
        let x = 10.0 + index as f64 * 20.0;
        assert!(close(station.point, [x, 50.0, 50.0], 1e-6));
        assert!(close(station.axis, [0.0, 0.0, 1.0], 1e-9));
        assert!((station.spec.length_mm - 6.0).abs() <= 1e-12);
        assert!((station.spec.depth_mm() - placement.stations[index].spec.depth_mm()).abs() <= 1e-12);
    }
}

#[test]
fn splop_is_rigid_and_orients_to_the_surface_normal() {
    let limits = Limits::default();
    let surface = plane_z(0.0);
    let spec = round_gem(6.0, 3.9);
    let params = SplopParams {
        z_offset_mm: 1.0,
        rotation_deg: 0.0,
        x_hint: [1.0, 0.0, 0.0],
    };
    let station = splop_station(&surface, [50.0, 50.0, 20.0], &spec, &params, &limits).unwrap();
    assert!(close(station.point, [50.0, 50.0, 1.0], 1e-6));
    assert!(close(station.axis, [0.0, 0.0, 1.0], 1e-9));
    assert!(station.frame_is_orthonormal());
    assert_eq!(&station.spec, &spec, "splop must not deform the stone");
    let uv = station.uv.unwrap();
    assert!((uv[0] - 0.5).abs() <= 1e-6 && (uv[1] - 0.5).abs() <= 1e-6);
    // A non-finite reference point is a typed refusal.
    assert!(
        splop_station(&surface, [f64::NAN, 0.0, 0.0], &spec, &params, &limits).is_err()
    );
}

#[test]
fn placement_link_round_trips_through_the_extension_and_save() {
    let gem = om9_core::gem::GemId::new();
    let source = ObjectId::new();
    let link = PlacementLink::new(
        gem,
        PlacementSource::Curve { id: source },
        12.5,
        0.25,
        7,
        10.0,
        0.5,
    )
    .unwrap();
    assert!(link.is_live());
    assert_eq!(link.state, LinkState::Active);
    assert_eq!(link.source.tag(), "curve");

    let extension = link.to_extension().unwrap();
    let reopened = PlacementLink::from_extension(&extension).unwrap();
    assert_eq!(reopened, link);
    assert_eq!(PlacementLink::from_json(&link.to_json().unwrap()).unwrap(), link);
    assert!(extension_slot_known(GEM_PLACEMENT_EXTENSION_SLOT));

    // Save/reopen: the link survives a full document JSON round trip on the entity.
    let mut document = Document::new("gems");
    let mut entity = Entity::new("gem", Geometry::Point([0.0, 0.0, 0.0]), None);
    entity
        .extensions
        .insert(GEM_PLACEMENT_EXTENSION_SLOT.to_owned(), extension);
    let id = entity.id;
    document.entities.insert(id, entity);
    let saved = serde_json::to_string(&document).unwrap();
    let reopened_document: Document = serde_json::from_str(&saved).unwrap();
    let slot = reopened_document.entities[&id]
        .extensions
        .get(GEM_PLACEMENT_EXTENSION_SLOT)
        .expect("slot survives save");
    assert_eq!(PlacementLink::from_extension(slot).unwrap(), link);
}

#[test]
fn placement_link_rejects_a_broken_record_and_tracks_state() {
    let nil_gem = om9_core::gem::GemId(ObjectId::nil());
    assert!(
        PlacementLink::new(
            nil_gem,
            PlacementSource::Curve {
                id: ObjectId::new()
            },
            0.0,
            0.0,
            1,
            1.0,
            0.0,
        )
        .is_err()
    );
    let link = PlacementLink::new(
        om9_core::gem::GemId::new(),
        PlacementSource::Surface {
            id: ObjectId::new(),
        },
        0.0,
        0.5,
        3,
        5.0,
        0.0,
    )
    .unwrap()
    .with_uv([0.25, 0.5])
    .unwrap();
    let mut broken = link.clone();
    broken.mark_broken().unwrap();
    assert_eq!(broken.state, LinkState::Broken);
    assert!(!broken.is_live());
    let mut conflict = link.clone();
    conflict.mark_conflict().unwrap();
    assert_eq!(conflict.state, LinkState::Conflict);
    let mut refreshed = conflict.clone();
    refreshed.refresh(4, 8.0, 0.4).unwrap();
    assert_eq!(refreshed.state, LinkState::Active);
    assert_eq!(refreshed.source_revision, 4);
    // A stale revision is refused instead of overwriting a newer station.
    assert!(refreshed.clone().refresh(2, 1.0, 0.1).is_err());

    // A payload with an unknown member or a foreign schema is refused.
    let mut value: serde_json::Value = serde_json::from_str(&link.to_json().unwrap()).unwrap();
    value["schema"] = serde_json::Value::String("om9.gem_placement.v0".to_owned());
    assert!(PlacementLink::from_json(&value.to_string()).is_err());
    let extension = Extension::new(1, link.to_json().unwrap()).unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(&extension.payload).unwrap();
    payload["unknown_member"] = serde_json::Value::Bool(true);
    assert!(
        PlacementLink::from_json(&payload.to_string()).is_err(),
        "unknown members must not be silently accepted"
    );
}

#[test]
fn station_frames_are_right_handed_under_every_orientation() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]);
    let spec = round_gem(4.0, 2.6);
    let orientations = [
        om9_core::gem_placement::Orientation::Upright,
        om9_core::gem_placement::Orientation::Axis([0.0, 1.0, 0.0]),
        om9_core::gem_placement::Orientation::Axis([0.0, 0.0, 1.0]),
    ];
    for orientation in orientations {
        let params = CurveParams::spacing(10.0).with_orientation(orientation);
        let placement = place_on_curve(&curve, &spec, &params, &limits).unwrap();
        for station in &placement.stations {
            assert!(station.frame_is_orthonormal(), "{orientation:?}");
        }
    }
    // SurfaceNormal is meaningless on a curve and is refused by name.
    let params = CurveParams::spacing(10.0)
        .with_orientation(om9_core::gem_placement::Orientation::SurfaceNormal);
    assert!(place_on_curve(&curve, &spec, &params, &limits).is_err());
}

#[test]
fn placement_is_pure_data_and_can_be_reused() {
    let limits = Limits::default();
    let curve = line([0.0, 0.0, 0.0], [40.0, 0.0, 0.0]);
    let params = CurveParams::spacing(10.0).with_z_offset(1.5);
    let first =
        place_on_curve(&curve, &round_gem(6.0, 3.9), &params, &limits).unwrap();
    let second =
        place_on_curve(&curve, &round_gem(6.0, 3.9), &params, &limits).unwrap();
    assert_eq!(first, second, "the same input must produce the same plan");
    for station in &first.stations {
        assert!(close(station.point, [station.arc_length_mm, 0.0, 1.5], 1e-9));
    }
    // A station is a value: cloning it does not share state.
    let copy: GemStation = first.stations[0].clone();
    assert_eq!(copy, first.stations[0]);
}
