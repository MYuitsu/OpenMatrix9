// SPDX-License-Identifier: LGPL-2.1-or-later
//! End-to-end regression for `OM9-L2-04-16`: the cutter family of the Cutters chapter.
//!
//! One gem row feeds the Gem Cutter (Gem/Azure/Advanced), the Gem Cutter Library presets, the
//! dimensional Bezel Cutter Builder that the shared `gvBezelCutter` command carries, and the
//! Interactive Boolean with its Preserve/Remove answer. Every body must be a closed solid that
//! clears the stone, and every unsupported control must be a typed refusal rather than a
//! silently different cutter.
use om9_core::{
    Error,
    cutters::{
        CUTTER_SEGMENTS, EdgeKind, EdgeShape, GemCutterReference, Ring,
        bezel::{BezelCutterDims, build as build_bezel_cutters},
        boolean::{BooleanMode, CutterKeep, apply as apply_boolean},
        gem::{
            BottomDirection, CutterMode, GemCutterParams, build as build_gem_cutter,
            build_for_stones,
        },
        library::{CutterPreset, LibraryParams, build as build_library_cutter, build_for_stones as build_library_for_stones},
        loft,
    },
    gem::{GemCut, GemShape, GemSpec},
    geometry::{Mesh, Transform},
    mesh_csg::CsgOptions,
    settings::{
        bezel::{ScallopParams, build_scallop},
        head::GemGirdle,
        inspect_solid,
    },
};

fn round_brilliant() -> GemSpec {
    GemSpec {
        shape: GemShape::Round,
        cut: GemCut::Diamond,
        length_mm: 6.5,
        width_mm: 6.5,
        girdle_mm: 0.4,
        crown_height_mm: 1.0,
        pavilion_depth_mm: 2.6,
        crown_angle_deg: 34.5,
        facet_count: 57,
        pavilion_rows: 2,
        bulge: 0.0,
        corner_mm: 0.0,
        profile: None,
    }
}

fn identity() -> Transform {
    Transform::identity()
}

fn girdle() -> GemGirdle {
    GemGirdle::from_spec(&round_brilliant(), &identity()).expect("girdle")
}

fn assert_closed(mesh: &Mesh, what: &str) {
    let report = inspect_solid(mesh).expect(what);
    assert!(report.closed, "{what} must be closed");
    assert!(report.volume_mm3 > 0.0, "{what} must have volume");
}

#[test]
fn the_three_builder_modes_build_closed_cutters_that_clear_the_stone() {
    let spec = round_brilliant();
    let mut heights = Vec::new();
    for mode in CutterMode::ALL {
        let params = GemCutterParams::for_gem(&spec).expect("params").with_mode(mode);
        let plan = build_gem_cutter(&spec, &identity(), "gem:1", &params).expect("plan");
        assert_eq!(plan.bodies.len(), 1, "{mode:?}");
        assert_closed(&plan.bodies[0].mesh, &format!("{mode:?} cutter"));
        assert!(
            plan.min_clearance_mm >= 0.0,
            "{mode:?} must clear the stone, got {}",
            plan.min_clearance_mm
        );
        heights.push(plan.bodies[0].volume_mm3);
    }
    // Azure drops the upper seat, so it removes less metal than Gem.
    assert!(heights[1] < heights[0], "{heights:?}");
    assert!(heights[2] > 0.0, "{heights:?}");
}

#[test]
fn a_cutter_that_would_cut_into_the_stone_is_refused() {
    let spec = round_brilliant();
    let mut params = GemCutterParams::for_gem(&spec).expect("params");
    params.clearance_mm = 0.0;
    params.girdle_scale_percent = 60.0;
    assert_eq!(
        build_gem_cutter(&spec, &identity(), "gem:1", &params).err(),
        Some(Error::Invalid("cutter pavilion clearance"))
    );
}

#[test]
fn the_lower_seat_depth_is_bounded_by_the_cutter_bottom() {
    let spec = round_brilliant();
    let mut params = GemCutterParams::for_gem(&spec).expect("params");
    params.lower_depth_percent = 100.0;
    params.lower_seat_depth_percent = 100.0;
    let error = build_gem_cutter(&spec, &identity(), "gem:1", &params).expect_err("refused");
    assert!(
        format!("{error:?}").contains("cutter"),
        "unexpected refusal {error:?}"
    );
}

#[test]
fn every_edge_profile_stays_a_closed_cutter_and_changes_the_hole() {
    let spec = round_brilliant();
    let plain = {
        let mut params = GemCutterParams::for_gem(&spec).expect("params");
        params.mode = CutterMode::Azure;
        params.edge = EdgeShape::default();
        build_gem_cutter(&spec, &identity(), "gem:1", &params).expect("plain")
    };
    for kind in EdgeKind::ALL.into_iter().filter(|kind| *kind != EdgeKind::Round) {
        let mut params = GemCutterParams::for_gem(&spec).expect("params");
        params.mode = CutterMode::Azure;
        params.edge = EdgeShape {
            kind,
            grooves: 6,
        };
        let plan = build_gem_cutter(&spec, &identity(), "gem:1", &params).expect("plan");
        assert_closed(&plan.bodies[0].mesh, kind.label());
        assert!(plan.min_clearance_mm >= 0.0, "{}", kind.label());
        assert!(
            (plan.bodies[0].volume_mm3 - plain.bodies[0].volume_mm3).abs() > 1.0e-6,
            "{} must change the removed volume",
            kind.label()
        );
    }
}

#[test]
fn the_bottom_direction_mirrors_the_cutter_above_the_girdle() {
    let spec = round_brilliant();
    let mut down = GemCutterParams::for_gem(&spec).expect("params");
    down.mode = CutterMode::Gem;
    let below = down.rings(&spec).expect("below rings");
    let below_bottom = below.first().expect("ring").z;
    assert!(below_bottom < 0.0);
    let below_span = (
        below.iter().map(|ring| ring.z).fold(f64::MAX, f64::min),
        below.iter().map(|ring| ring.z).fold(f64::MIN, f64::max),
    );
    // Flipped up, every ring is mirrored about the girdle plane and the order reversed, so the
    // cutter still spans the girdle but reaches the other way.
    down.bottom = BottomDirection::Up;
    let above = down.rings(&spec).expect("above rings");
    let above_span = (
        above.iter().map(|ring| ring.z).fold(f64::MAX, f64::min),
        above.iter().map(|ring| ring.z).fold(f64::MIN, f64::max),
    );
    assert!(
        (above_span.0 + below_span.1).abs() < 1.0e-3,
        "below {below_span:?} above {above_span:?}"
    );
    assert!(
        (above_span.1 + below_span.0).abs() < 1.0e-3,
        "below {below_span:?} above {above_span:?}"
    );
    assert!(above.first().expect("ring").z <= above.last().expect("ring").z);
    // The same seat would bite the crown, so the gate refuses until the girdle scale opens.
    let built = build_gem_cutter(&spec, &identity(), "gem:1", &down);
    assert_eq!(
        built.err(),
        Some(Error::Invalid("cutter pavilion clearance"))
    );
}

#[test]
fn multiple_stones_get_one_cutter_each() {
    let spec = round_brilliant();
    let stones = vec![
        ("gem:a".to_owned(), spec.clone(), identity()),
        (
            "gem:b".to_owned(),
            spec.clone(),
            Transform([
                [1.0, 0.0, 0.0, 10.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]),
        ),
    ];
    let params = GemCutterParams::for_gem(&spec).expect("params");
    let plans = build_for_stones(&stones, &params).expect("plans");
    assert_eq!(plans.len(), 2);
    assert!(plans.iter().all(|plan| plan.bodies.len() == 1));
    let first = plans[0].bodies[0].mesh.vertices[0];
    let second = plans[1].bodies[0].mesh.vertices[0];
    assert!((second[0] - first[0] - 10.0).abs() < 1.0e-6);
    let empty: Vec<(String, GemSpec, Transform)> = Vec::new();
    assert_eq!(
        build_for_stones(&empty, &params).err(),
        Some(Error::Missing("cutter gems"))
    );
}

#[test]
fn every_library_preset_builds_a_closed_cutter() {
    let spec = round_brilliant();
    for preset in CutterPreset::ALL {
        let params = LibraryParams {
            preset,
            ..LibraryParams::default()
        };
        let cutter = build_library_cutter(&spec, &identity(), "gem:1", &params).expect("cutter");
        assert_closed(&cutter.mesh, preset.label());
        assert!(cutter.min_clearance_mm >= 0.0, "{}", preset.label());
    }
}

#[test]
fn the_library_2d_scale_is_a_typed_control() {
    let spec = round_brilliant();
    let plain = build_library_cutter(&spec, &identity(), "gem:1", &LibraryParams::default())
        .expect("plain");
    let scaled = build_library_cutter(
        &spec,
        &identity(),
        "gem:1",
        &LibraryParams {
            scale_x_percent: 120.0,
            scale_y_percent: 120.0,
            ..LibraryParams::default()
        },
    )
    .expect("scaled");
    assert!((scaled.volume_mm3 - plain.volume_mm3).abs() > 1.0e-6);
    let too_small = LibraryParams {
        preset: CutterPreset::SettingAzure,
        scale_x_percent: 5.0,
        scale_y_percent: 5.0,
        ..LibraryParams::default()
    };
    assert_eq!(
        build_library_cutter(&spec, &identity(), "gem:1", &too_small).err(),
        Some(Error::Invalid("cutter library 2d scale"))
    );
}

#[test]
fn the_library_builds_one_cutter_per_stone_and_refuses_an_empty_group() {
    let spec = round_brilliant();
    let stones = vec![
        ("gem:a".to_owned(), spec.clone(), identity()),
        ("gem:b".to_owned(), spec, identity()),
    ];
    let cutters =
        build_library_for_stones(&stones, &LibraryParams::default()).expect("cutters");
    assert_eq!(cutters.len(), 2);
    assert_eq!(
        build_library_for_stones(&[], &LibraryParams::default()).err(),
        Some(Error::Missing("cutter gems"))
    );
}

#[test]
fn the_dimensional_bezel_builder_makes_a_closed_block_per_cutter() {
    let girdle = girdle();
    let dims = BezelCutterDims::default();
    let set = build_bezel_cutters(&girdle, &dims).expect("bezel cutters");
    assert_eq!(set.count(), dims.count as usize);
    for cutter in &set.cutters {
        assert_closed(cutter, "bezel cutter");
    }
    // The same command (`gvBezelCutter`) carries the block contract through the scallop owner.
    let params = ScallopParams {
        dims: Some(dims.clone()),
        ..ScallopParams::default()
    };
    let scallop = build_scallop(&girdle, &params).expect("scallop with dims");
    assert_eq!(scallop.count(), dims.count as usize);
    assert!(scallop.profile.starts_with("Bezel Cutter"));
}
#[test]
fn the_bezier_profile_still_wins_when_no_dimensional_block_is_asked_for() {
    let girdle = girdle();
    let scallop = build_scallop(&girdle, &ScallopParams::default()).expect("swept scallop");
    assert_eq!(scallop.count(), 12);
    assert_eq!(scallop.profile, "Round");
}

#[test]
fn a_negative_cutter_length_is_refused() {
    let girdle = girdle();
    let mut dims = BezelCutterDims::default();
    dims.cutter_length_mm = 0.0;
    assert_eq!(
        build_bezel_cutters(&girdle, &dims).err(),
        Some(Error::Invalid("bezel cutter cutter length"))
    );
    let mut dims = BezelCutterDims::default();
    dims.cutter_angle_deg = 120.0;
    assert_eq!(
        build_bezel_cutters(&girdle, &dims).err(),
        Some(Error::Invalid("bezel cutter angle"))
    );
}

#[test]
fn the_interactive_boolean_keeps_or_removes_the_cutter_bodies() {
    // Large enough that every block of the pattern bites the body.
    let target = box_mesh(22.0, 8.0, -5.0);
    let girdle = girdle();
    let set = build_bezel_cutters(&girdle, &BezelCutterDims::default()).expect("cutters");
    let kept = apply_boolean(
        &target,
        &set.cutters,
        BooleanMode::On,
        CutterKeep::Preserve,
        CsgOptions::default(),
    )
    .expect("cut");
    assert!(kept.keep_cutters);
    assert!(kept.removed_mm3 > 0.0);
    assert_closed(&kept.result, "cut target");
    let removed = apply_boolean(
        &target,
        &set.cutters,
        BooleanMode::One,
        CutterKeep::Remove,
        CsgOptions::default(),
    )
    .expect("single cut");
    assert!(!removed.keep_cutters);
    assert_eq!(removed.applied, 1);
    assert!(removed.removed_mm3 <= kept.removed_mm3);
}

#[test]
fn the_cutter_reference_is_measured_from_the_stone_profile() {
    let spec = round_brilliant();
    let reference = GemCutterReference::from_spec(&spec).expect("reference");
    assert!(reference.radius_x > 3.0 && reference.radius_x < 3.3);
    assert!(reference.pavilion_depth_mm > 2.5);
    let profile = om9_core::gem::profile_points(&spec).expect("profile");
    let at_girdle = reference.radius_at(&profile, -reference.girdle_half_mm, 1.0);
    assert!((at_girdle - reference.min_radius_mm()).abs() < 1.0e-9);
    assert!(reference.validate().is_ok());
}

#[test]
fn the_loft_refuses_ring_orders_it_cannot_close() {
    let rings = [Ring::round(1.0, 1.0), Ring::round(0.0, 0.5)];
    assert_eq!(
        loft(&rings, CUTTER_SEGMENTS).err(),
        Some(Error::Invalid("cutter ring order"))
    );
    assert_eq!(
        loft(&[], CUTTER_SEGMENTS).err(),
        Some(Error::Invalid("cutter rings"))
    );
}

fn box_mesh(size: f64, height: f64, z: f64) -> Mesh {
    let half = size * 0.5;
    let vertices = vec![
        [-half, -half, z],
        [half, -half, z],
        [half, half, z],
        [-half, half, z],
        [-half, -half, z + height],
        [half, -half, z + height],
        [half, half, z + height],
        [-half, half, z + height],
    ];
    let faces = vec![
        vec![0, 3, 2, 1],
        vec![4, 5, 6, 7],
        vec![0, 1, 5, 4],
        vec![1, 2, 6, 5],
        vec![2, 3, 7, 6],
        vec![3, 0, 4, 7],
    ];
    om9_core::cutters::solid(vertices, faces).expect("box")
}
