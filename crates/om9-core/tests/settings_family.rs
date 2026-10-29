// SPDX-License-Identifier: LGPL-2.1-or-later
//! End-to-end regression for `OM9-L2-04-15`: one gem row feeds every setting part.
//!
//! The test is the acceptance oracle for the port: the same `GemRow` is consumed by the prong
//! builder/editor, Metal from Gems, the channel border curve, the beads and the associative
//! rebuild, and the mass read-out agrees with the metal owner's density arithmetic. It also
//! pins the documented refusals so an unsupported mode cannot silently build a different body.
use om9_core::{
    Error,
    budget::Limits,
    settings::inspect_solid,
    gem::{GemSpec},
    geometry::Curve,
    metal_weight::{MetalMaterialRegistry, MetalEstimate},
    settings::{
        associative::{settings_mass, FeatureMassPart},
        bead::{BeadDirection, BeadOnCurveParams, BeadOnSurfaceParams, bead_on_curve, bead_on_surface, build_bead_bodies},
        channel_border::{ChannelBorderParams, build_border},
        gems::{GemRow, GemStone, RowShape, straight_row},
        metal_piece::{MetalPieceMode, MetalPieceParams, build as build_metal},
        prong::{
            EndMode, ProngMode, ProngParams, ProngPosition, ProngSiteKind, build_prong_set,
            edit_prong, ProngEdit, place_prongs, prong_on_surface, SurfaceProngParams,
        },
        rail::{Rail, RailPullMode, pull_mesh, pull_to_rail},
    },
    surface_kernel,
};

fn straight(diameters: &[f64], spacing: f64) -> GemRow {
    straight_row(diameters, spacing, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).expect("row")
}

fn closed_row() -> GemRow {
    let stones = (0..8)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / 8.0;
            GemStone::round(
                format!("stone_{index:02}"),
                [10.0 * angle.cos(), 10.0 * angle.sin(), 0.0],
                2.0,
                [0.0, 0.0, 1.0],
            )
            .expect("stone")
        })
        .collect();
    GemRow::new(stones, RowShape::Closed).expect("closed row")
}

#[test]
fn one_row_drives_every_setting_part() {
    let limits = Limits::default();
    let row = straight(&[2.0, 2.0, 2.0, 2.0], 0.1);
    assert_eq!(row.len(), 4);

    // Prongs: the default Gem Flow style is the documented 2 Shared Prongs.
    let params = ProngParams::default();
    assert_eq!(params.mode, ProngMode::TwoShared);
    let set = build_prong_set(&row, &params).expect("prong set");
    assert!(!set.sites.is_empty());
    assert_eq!(set.sites.len(), set.bodies.len());
    assert!(set.min_height_mm >= om9_core::settings::MIN_PRONG_HEIGHT_MM - 1.0e-12);
    let mut prong_volume = 0.0;
    for body in &set.bodies {
        let report = om9_core::settings::inspect_solid(body).expect("closed prong");
        assert!(report.volume_mm3 > 0.0);
        prong_volume += report.volume_mm3;
    }
    assert!((prong_volume - set.volume_mm3).abs() < 1.0e-6);

    // Every prong has a stable id, and the ids are unique.
    let mut ids: Vec<String> = set.sites.iter().map(|site| site.stable_id()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), set.sites.len());

    // Metal from Gems consumes the same row; the channel wall is closed and outside the girdle.
    let metal = build_metal(
        &row,
        &MetalPieceParams {
            mirror: false,
            ..MetalPieceParams::default()
        },
    )
    .expect("metal piece");
    assert_eq!(metal.bodies.len(), 1);
    assert!(metal.volume_mm3 > 0.0);
    for body in &metal.bodies {
        om9_core::settings::inspect_solid(body).expect("closed metal");
    }

    // Channel border: two valid guide curves, longer than the stone chain.
    let border = build_border(&row, &ChannelBorderParams::default(), &limits).expect("border");
    border.left.validate(&limits).expect("left curve");
    border.right.validate(&limits).expect("right curve");
    // The border spans the chain from the first stone centre to the last one.
    let straight_length = 3.0 * (2.0 + 0.1);
    let left_length = om9_core::curve_ops::arc_length(&border.left, &limits).expect("length");
    assert!(left_length >= straight_length - 1.0e-6);

    // Beads: the same row centreline could carry milgrain, and the bead bodies are closed.
    let centreline = Curve::Polycurve {
        segments: row
            .stones
            .windows(2)
            .map(|pair| Curve::Line {
                start: pair[0].center,
                end: pair[1].center,
                domain: [0.0, 1.0],
            })
            .collect(),
        reversed: false,
    };
    centreline.validate(&limits).expect("centreline");
    let beads = bead_on_curve(
        &centreline,
        &BeadOnCurveParams {
            spacing_mm: 1.0,
            direction: BeadDirection::Down,
            ..BeadOnCurveParams::default()
        },
        &limits,
    )
    .expect("beads");
    assert!(beads.beads.len() >= 4);
    let bodies = build_bead_bodies(&beads, 12).expect("bead bodies");
    assert_eq!(bodies.len(), beads.beads.len());

    // A bead on a surface: one body, the surface normal as its axis, the scale applied.
    let plane = surface_kernel::plane([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0, &limits)
        .expect("plane");
    let (surface_bead, surface_body) = bead_on_surface(
        &plane,
        [0.4, 0.6],
        &BeadOnSurfaceParams {
            scale: 1.5,
            ..BeadOnSurfaceParams::default()
        },
        &limits,
    )
    .expect("surface bead");
    assert!((surface_bead.diameter_mm - 1.5).abs() < 1.0e-9);
    om9_core::settings::inspect_solid(&surface_body).expect("closed bead");

    // Pull to rail: the rigid mesh pull lands on the rail and preserves the body shape.
    let rail = Rail::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 10.0).expect("rail");
    let ball = om9_core::settings::sphere([14.0, 0.0, 2.0], 2.0, 16).expect("sphere");
    let (pulled, report) = pull_mesh(&ball, &rail, RailPullMode::Rigid).expect("rigid pull");
    assert!(report.max_displacement_mm > 0.0);
    assert_eq!(pulled.vertices.len(), ball.vertices.len());

    // The pull result itself is on the rail, and the deform mode keeps the axial height.
    let rigid = pull_to_rail([14.0, 0.0, 2.0], [0.0, 0.0, 1.0], &rail, RailPullMode::Rigid)
        .expect("rigid result");
    assert!(rigid.is_on_rail(&rail, 1.0e-9));
    let deform = pull_to_rail([14.0, 0.0, 2.0], [0.0, 0.0, 1.0], &rail, RailPullMode::Deform)
        .expect("deform result");
    assert!((deform.origin[2] - 2.0).abs() < 1.0e-9);
}

#[test]
fn every_documented_prong_preset_builds_or_refuses_by_name() {
    // A 3.0 mm girdle gap keeps the tangent-facing prongs of adjacent stones apart, so a style
    // is refused only for its own layout, never because the test row was too tight.
    let row = straight(&[2.0, 2.0], 3.0);
    let presets = ProngMode::presets();
    assert!(presets.len() >= 20, "{} documented styles", presets.len());
    let mut built = 0usize;
    let mut refused: Vec<(String, Error)> = Vec::new();
    for preset in &presets {
        let params = ProngParams {
            mode: *preset,
            ..ProngParams::default()
        };
        match build_prong_set(&row, &params) {
            Ok(set) => {
                built += 1;
                assert!(!set.bodies.is_empty(), "{} built nothing", preset.name());
                for body in &set.bodies {
                    om9_core::settings::inspect_solid(body).expect("closed body");
                }
                assert_eq!(
                    ProngMode::from_name(&preset.name()),
                    Some(*preset),
                    "{} did not round-trip",
                    preset.name()
                );
            }
            Err(error) => refused.push((preset.name(), error)),
        }
    }
    for (name, error) in &refused {
        assert!(
            matches!(error, Error::Unsupported(_) | Error::Invalid("duplicate prong")),
            "{name} refused with an untyped error: {error}"
        );
    }
    assert!(
        built == presets.len(),
        "{} of {} styles built; refused: {:?}",
        built,
        presets.len(),
        refused.iter().map(|(name, error)| (name.as_str(), error.to_string())).collect::<Vec<_>>()
    );
}

#[test]
fn gem_north_prongs_are_placed_around_the_gem_and_follow_the_girdle_angle() {
    let row = straight(&[2.0, 2.0], 0.1);
    let params = ProngParams {
        mode: ProngMode::FourProngsAt(45.0),
        ..ProngParams::default()
    };
    let sites = place_prongs(&row, &params).expect("sites");
    let gem_sites: Vec<_> = sites
        .iter()
        .filter(|site| matches!(site.kind, ProngSiteKind::Gem { .. }))
        .collect();
    assert_eq!(gem_sites.len(), 8);
    // Four prongs per stone, at -45 / +45 around the requested 45 degree plane, repeated on
    // the opposite side: 45+-r and 135+-r.
    let angles: Vec<f64> = gem_sites
        .iter()
        .filter(|site| matches!(site.kind, ProngSiteKind::Gem { stone: 0 }))
        .filter_map(|site| site.angle_deg)
        .collect();
    assert_eq!(angles.len(), 4);
    let radius = (angles[1] - angles[0]).abs() / 2.0;
    let mid = (angles[0] + angles[1]) / 2.0;
    assert!((mid - 45.0).abs() < 1.0e-9, "mid {mid} radius {radius}");
    // The prong origin sits on the girdle circle of the stone plus the nudge.
    for site in &gem_sites {
        assert!(site.placement_radius_mm > row.stone(0).unwrap().radius_mm());
    }
}

#[test]
fn the_close_cluster_keeps_two_prongs_per_side_closer_than_4_prongs_0() {
    // The sheet defines the style by its spacing: "two prongs on the North side closer together
    // than 4 Prongs 0 Degrees". The test measures that sentence instead of a fixed degree value.
    let row = straight(&[2.0, 2.0], 3.0);
    let close = build_prong_set(
        &row,
        &ProngParams {
            mode: ProngMode::FourProngsClose,
            ..ProngParams::default()
        },
    )
    .expect("close cluster");
    let open = build_prong_set(
        &row,
        &ProngParams {
            mode: ProngMode::FourProngsAt(0.0),
            ..ProngParams::default()
        },
    )
    .expect("4 prongs 0");
    assert_eq!(close.sites.len(), 8, "four prongs on two stones");
    assert_eq!(close.merged_prongs, 0, "a close cluster never merges");
    for body in &close.bodies {
        om9_core::settings::inspect_solid(body).expect("closed close prong");
    }
    /// The smallest distance between two prongs of the first stone: one twin of the close pair.
    fn twin_gap(set: &om9_core::settings::prong::ProngSet) -> f64 {
        let origins: Vec<[f64; 3]> = set
            .sites
            .iter()
            .filter(|site| site.kind == ProngSiteKind::Gem { stone: 0 })
            .map(|site| site.frame.origin)
            .collect();
        let mut best = f64::INFINITY;
        for (index, first) in origins.iter().enumerate() {
            for second in &origins[index + 1..] {
                let dx = first[0] - second[0];
                let dy = first[1] - second[1];
                let dz = first[2] - second[2];
                best = best.min((dx * dx + dy * dy + dz * dz).sqrt());
            }
        }
        best
    }
    let close_gap = twin_gap(&close);
    let open_gap = twin_gap(&open);
    assert!(
        close_gap < open_gap,
        "close {close_gap:.3} mm is not closer than 4 prongs 0 {open_gap:.3} mm"
    );
    assert!(
        close_gap > 0.05,
        "a close pair is still two prongs, not one: {close_gap:.3} mm"
    );
}

#[test]
fn six_prongs_0_builds_on_a_tight_row_and_merges_only_a_shared_point() {
    // The 6-prong style contains the close cluster, so the two prongs of one stone are closer
    // than half a prong and must not be treated as duplicates of each other.
    let tight = straight(&[2.0, 2.0], 0.1);
    let set = build_prong_set(
        &tight,
        &ProngParams {
            mode: ProngMode::SixProngsAt(0.0),
            ..ProngParams::default()
        },
    )
    .expect("six prongs on a tight row");
    assert_eq!(set.sites.len(), 12, "six prongs on each of two stones");
    assert_eq!(set.merged_prongs, 0, "{}", set.summary());
    // When the row spacing equals twice the placement radius, the facing prongs of two stones
    // land on one point: the sheet shows one shared prong there, so the pair merges.
    let coincident = straight(&[2.0, 2.0], 0.8);
    let merged = build_prong_set(
        &coincident,
        &ProngParams {
            mode: ProngMode::SixProngsAt(0.0),
            ..ProngParams::default()
        },
    )
    .expect("six prongs on a coincident row");
    assert_eq!(merged.merged_prongs, 1, "{}", merged.summary());
    assert_eq!(merged.sites.len(), 11);
    let shared = merged
        .sites
        .iter()
        .find(|site| site.angle_deg.is_none())
        .expect("the shared prong has no single girdle angle");
    assert!((shared.frame.origin[0] - 1.4).abs() < 1.0e-9, "{shared:?}");
    for body in &merged.bodies {
        om9_core::settings::inspect_solid(body).expect("closed shared prong");
    }
}

#[test]
fn the_editor_changes_one_prong_and_leaves_its_neighbours_alone() {
    let row = straight(&[2.0, 2.0], 0.1);
    let params = ProngParams::default();
    let mut set = build_prong_set(&row, &params).expect("set");
    let before: Vec<f64> = set.sites.iter().map(|site| site.diameter_mm).collect();
    let edit = ProngEdit {
        diameter_mm: Some(1.4),
        ..ProngEdit::default()
    };
    let body = edit_prong(&mut set.sites, 0, &params, &edit).expect("edit");
    om9_core::settings::inspect_solid(&body).expect("edited body is closed");
    assert_eq!(set.sites[0].diameter_mm, 1.4);
    for (site, previous) in set.sites.iter().skip(1).zip(before.iter().skip(1)) {
        assert_eq!(site.diameter_mm, *previous);
    }
}

#[test]
fn row_end_modes_and_prong_spacing_change_the_placement_only() {
    let row = straight(&[2.0, 2.0], 0.1);
    let base = ProngParams::default();
    let none = build_prong_set(
        &row,
        &ProngParams {
            end_mode: EndMode::None,
            ..base.clone()
        },
    )
    .expect("no ends");
    let two = build_prong_set(
        &row,
        &ProngParams {
            end_mode: EndMode::Two,
            ..base.clone()
        },
    )
    .expect("two ends");
    assert!(two.count() > none.count());
    let even = build_prong_set(
        &row,
        &ProngParams {
            position: ProngPosition::Even,
            ..base.clone()
        },
    )
    .expect("even");
    let odd = build_prong_set(
        &row,
        &ProngParams {
            position: ProngPosition::Odd,
            ..base
        },
    )
    .expect("odd");
    assert_eq!(even.count(), odd.count());
}

#[test]
fn an_unsupported_metal_mode_is_a_typed_refusal_not_a_different_body() {
    let row = straight(&[2.0, 2.0], 0.1);
    let error = build_metal(
        &row,
        &MetalPieceParams {
            mode: MetalPieceMode::Curve,
            ..MetalPieceParams::default()
        },
    )
    .err();
    assert_eq!(error, Some(Error::Unsupported("metal piece curve mode")));
}

#[test]
fn the_closed_row_builds_a_closed_metal_ring_and_keeps_the_prong_order() {
    let row = closed_row();
    let metal = build_metal(
        &row,
        &MetalPieceParams {
            mode: MetalPieceMode::ClosedOutside,
            mirror: false,
            ..MetalPieceParams::default()
        },
    )
    .expect("closed metal");
    assert!(metal.closed_row);
    for body in &metal.bodies {
        om9_core::settings::inspect_solid(body).expect("watertight ring");
    }
    let set = build_prong_set(&row, &ProngParams::default()).expect("closed prongs");
    assert!(set.count() > 0);
    om9_core::settings::prong::verify_row_order(&row, &set.sites).expect("ordered");
}

#[test]
fn a_surface_prong_uses_the_surface_normal_and_the_shared_drop() {
    let limits = Limits::default();
    let plane = surface_kernel::plane([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0, &limits)
        .expect("plane");
    let params = SurfaceProngParams {
        diameter_mm: 1.2,
        height_mm: 1.0,
        drop_mm: 0.2,
        ..SurfaceProngParams::default()
    };
    let (site, body) = prong_on_surface(&plane, [0.5, 0.5], &params, &limits).expect("surface prong");
    om9_core::settings::inspect_solid(&body).expect("closed");
    assert!(site.anchor_normal[2].abs() > 0.99);
    assert!((site.frame.origin[2] + params.drop_mm).abs() < 1.0e-9);
}

#[test]
fn settings_mass_matches_the_metal_owner_for_the_prong_volume() {
    let row = straight(&[2.0, 2.0, 2.0], 0.1);
    let set = build_prong_set(&row, &ProngParams::default()).expect("set");
    let registry = MetalMaterialRegistry::builtin();
    let material = registry.iter().next().expect("builtin material");
    let parts = vec![FeatureMassPart::new("prongs-1", set.volume_mm3, material.id.clone())];
    let breakdown = settings_mass(&parts, &registry).expect("mass");
    let expected = MetalEstimate::new(set.volume_mm3, material).expect("estimate");
    assert!((breakdown.total_grams - expected.grams).abs() < 1.0e-9);
    assert!((breakdown.total_dwt - expected.dwt).abs() < 1.0e-9);
    assert_eq!(breakdown.entries.len(), 1);
}

#[test]
fn the_gem_contract_accepts_the_gem_owner_specs() {
    // The settings port never re-implements gems: a spec the gems owner validates is a legal
    // stone size for the row, and the diameter the row reports is the spec girdle width.
    let spec = GemSpec::round(2.0, 1.6).expect("gem spec");
    let stone = GemStone::round("gem_01", [0.0; 3], spec.width_mm, [0.0, 0.0, 1.0]).expect("stone");
    assert!((stone.diameter_mm - 2.0).abs() < 1.0e-9);
    let row = GemRow::new(vec![stone], RowShape::Open);
    assert!(row.is_ok());
}
#[test]
fn bead_count_and_spacing_are_alternatives_on_the_same_run() {
    let limits = Limits::default();
    let curve = Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    let mut params = BeadOnCurveParams {
        start_size_mm: 0.4,
        end_size_mm: 0.4,
        ..BeadOnCurveParams::default()
    };
    params.count = Some(5);
    let run = bead_on_curve(&curve, &params, &limits).expect("count run");
    assert_eq!(run.beads.len(), 5, "{}", run.summary());
    assert!((run.spacing_mm - 2.5).abs() < 1.0e-9, "{}", run.spacing_mm);
    // Driving the run by spacing keeps the same endpoints and produces the documented 11 beads.
    params.count = None;
    params.spacing_mm = 1.0;
    let spaced = bead_on_curve(&curve, &params, &limits).expect("spaced run");
    assert_eq!(spaced.beads.len(), 11);
    // A count below two is a typed refusal, not a single bead or a silent default.
    params.count = Some(1);
    assert_eq!(
        bead_on_curve(&curve, &params, &limits).err(),
        Some(Error::Invalid("bead count"))
    );
}

#[test]
fn bead_on_surface_uses_the_shared_base_diameter() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &limits,
    )
    .expect("plane");
    let params = BeadOnSurfaceParams {
        scale: 2.0,
        ..BeadOnSurfaceParams::default()
    };
    let (bead, body) = bead_on_surface(&surface, [0.5, 0.5], &params, &limits).expect("bead");
    let report = inspect_solid(&body).expect("solid");
    assert!(report.closed);
    assert!(
        (bead.diameter_mm - params.base_diameter_mm * params.scale).abs() < 1.0e-9,
        "{}",
        bead.diameter_mm
    );
}
