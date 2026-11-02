// SPDX-License-Identifier: LGPL-2.1-or-later
//! Regression for `OM9-L2-04-17`: the channel, micro-prong and bright-cut cutter family.
//!
//! Every cutter must be a closed solid with positive volume; the row controls (end profiles,
//! A/B flip, Mirror, the middle section, per-end depth) must change the body they name; and a
//! parameter that cannot build a cutter must be a typed refusal instead of a silently different
//! body.
use om9_core::{
    cutters::{
        brightcut::{
            BrightParams, build_channel as build_bright_channel,
            build_single as build_bright_single,
        },
        channel::{
            ChannelEnd, ChannelMiddle, ChannelParams, build_row as build_channel,
        },
        microprong::{MicroProngParams, build_row as build_microprong},
        sweep,
    },
    gem::{GemCut, GemShape, GemSpec},
    geometry::{Mesh, Transform},
    settings::inspect_solid,
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

fn baguette() -> GemSpec {
    GemSpec {
        shape: GemShape::Baguette,
        cut: GemCut::Diamond,
        length_mm: 4.0,
        width_mm: 2.2,
        girdle_mm: 0.35,
        crown_height_mm: 0.7,
        pavilion_depth_mm: 1.6,
        crown_angle_deg: 30.0,
        facet_count: 25,
        pavilion_rows: 2,
        bulge: 0.0,
        corner_mm: 0.2,
        profile: None,
    }
}

/// A row of identical stones along +X, `spacing` millimetres apart.
fn row(count: usize, spacing: f64) -> Vec<(String, GemSpec, Transform)> {
    (0..count)
        .map(|index| {
            (
                format!("stone-{index}"),
                round_brilliant(),
                Transform::translation([index as f64 * spacing, 0.0, 0.0]),
            )
        })
        .collect()
}

fn assert_closed(mesh: &Mesh, what: &str) {
    let report = inspect_solid(mesh).expect(what);
    assert!(report.closed, "{what} must be closed");
    assert!(
        report.volume_mm3 > 0.0,
        "{what} must have positive volume, got {}",
        report.volume_mm3
    );
}

fn stone_specs(stones: &[(String, GemSpec, Transform)]) -> Vec<GemSpec> {
    stones.iter().map(|(_, spec, _)| spec.clone()).collect()
}

#[test]
fn the_channel_builder_sweeps_one_closed_trench_through_the_row() {
    let stones = row(3, 7.0);
    let params = ChannelParams::for_row(&stone_specs(&stones)).expect("defaults");
    let cutter = build_channel(&stones, &params).expect("channel");
    assert_closed(&cutter.mesh, "channel cutter");
    assert_eq!(cutter.gem_ids, vec!["stone-0", "stone-1", "stone-2"]);
    assert!(cutter.length_mm > 14.0, "3 stones span 14 mm");
    assert!(cutter.min_margin_mm >= 0.0);
    // One cap at each end plus one station per stone.
    assert_eq!(cutter.stations, 5);
    assert!(cutter.summary.contains("Channel Cutter"));
}

#[test]
fn the_channel_end_profiles_shape_the_caps() {
    let stones = row(2, 7.0);
    let specs = stone_specs(&stones);
    let mut square = ChannelParams::for_row(&specs).expect("defaults");
    square.start = ChannelEnd::Square;
    square.end = ChannelEnd::Square;
    let flat = build_channel(&stones, &square).expect("square ends");
    let mut taper = square;
    taper.start = ChannelEnd::Taper;
    taper.end = ChannelEnd::Taper;
    let tapered = build_channel(&stones, &taper).expect("taper ends");
    let mut round = square;
    round.start = ChannelEnd::Round;
    round.end = ChannelEnd::Round;
    let rounded = build_channel(&stones, &round).expect("round ends");
    assert_closed(&tapered.mesh, "tapered channel");
    assert_closed(&rounded.mesh, "rounded channel");
    assert!(
        tapered.volume_mm3 < flat.volume_mm3,
        "a tapered cap removes material: {:.3} < {:.3}",
        tapered.volume_mm3,
        flat.volume_mm3
    );
    assert!(
        rounded.volume_mm3 < flat.volume_mm3,
        "a rounded cap removes material"
    );
    // Every end profile keeps the row frame, so the length does not move.
    assert!((flat.length_mm - rounded.length_mm).abs() < 1.0e-9);
}

#[test]
fn the_channel_mirror_and_flip_swap_the_ends() {
    let stones = row(2, 7.0);
    let specs = stone_specs(&stones);
    let mut params = ChannelParams::for_row(&specs).expect("defaults");
    params.start = ChannelEnd::Taper;
    params.end = ChannelEnd::Square;
    params.start_depth_mm = 0.30;
    params.end_depth_mm = 2.00;
    let lopsided = build_channel(&stones, &params).expect("A/B ends");
    let mut flipped = params;
    flipped.flip = true;
    let flipped_cutter = build_channel(&stones, &flipped).expect("flipped");
    assert_closed(&lopsided.mesh, "channel A/B");
    assert_closed(&flipped_cutter.mesh, "channel B/A");
    assert_ne!(
        lopsided.mesh.vertices, flipped_cutter.mesh.vertices,
        "Flip must move the deep end to the other side"
    );
    let mut mirrored = params;
    mirrored.mirror = true;
    let mirrored_cutter = build_channel(&stones, &mirrored).expect("mirrored");
    assert_closed(&mirrored_cutter.mesh, "channel mirrored");
    // Mirror makes the channel symmetric: flipping A/B afterwards cannot change the body, which
    // is exactly what the manual's Mirror toggle promises.
    let mut mirrored_flipped = mirrored;
    mirrored_flipped.flip = true;
    let same = build_channel(&stones, &mirrored_flipped).expect("mirrored and flipped");
    assert_eq!(mirrored_cutter.mesh.vertices, same.mesh.vertices);
}

#[test]
fn the_channel_middle_profile_adds_a_station() {
    let stones = row(3, 7.0);
    let specs = stone_specs(&stones);
    let plain = ChannelParams::for_row(&specs).expect("defaults");
    let plain_cutter = build_channel(&stones, &plain).expect("plain");
    let mut wide = plain;
    wide.middle = ChannelMiddle::Wide;
    let wide_cutter = build_channel(&stones, &wide).expect("wide middle");
    assert_eq!(wide_cutter.stations, plain_cutter.stations + 1);
    assert!(wide_cutter.volume_mm3 > plain_cutter.volume_mm3);
    let mut deep = plain;
    deep.middle = ChannelMiddle::Deep;
    let deep_cutter = build_channel(&stones, &deep).expect("deep middle");
    assert_closed(&deep_cutter.mesh, "deep middle channel");
    assert!(deep_cutter.volume_mm3 > plain_cutter.volume_mm3);
}

#[test]
fn the_channel_refuses_a_narrow_row_or_a_deep_trench() {
    let stones = row(2, 7.0);
    let specs = stone_specs(&stones);
    let mut params = ChannelParams::for_row(&specs).expect("defaults");
    params.width_mm = 2.0;
    let error = build_channel(&stones, &params).expect_err("narrower than the stone");
    assert!(format!("{error:?}").contains("channel width"), "{error:?}");

    let mut deep = ChannelParams::for_row(&specs).expect("defaults");
    deep.depth_mm = 9.0;
    let error = build_channel(&stones, &deep).expect_err("deeper than the stone");
    assert!(format!("{error:?}").contains("channel depth"), "{error:?}");

    let error = build_channel(&[], &ChannelParams::default()).expect_err("empty row");
    assert!(format!("{error:?}").contains("channel gems"), "{error:?}");

    let mut gap = params;
    gap.width_mm = 7.0;
    let too_close = vec![
        ("a".to_owned(), round_brilliant(), Transform::identity()),
        (
            "b".to_owned(),
            round_brilliant(),
            Transform::translation([0.001, 0.0, 0.0]),
        ),
    ];
    let error = build_channel(&too_close, &gap).expect_err("coincident stones");
    assert!(format!("{error:?}").contains("channel row"), "{error:?}");
}

#[test]
fn microprongs_land_in_every_gap_and_stay_closed() {
    let stones = row(3, 7.0);
    let params = MicroProngParams::for_row(&stone_specs(&stones)).expect("defaults");
    let cutters = build_microprong(&stones, &params).expect("microprongs");
    assert_eq!(cutters.count(), 2, "one scallop per gap");
    assert_eq!(cutters.gem_ids.len(), 3);
    for mesh in &cutters.cutters {
        assert_closed(mesh, "micro-prong scallop");
    }
    assert!(cutters.total_volume_mm3 > 0.0);

    let mut exact = params;
    exact.count = 5;
    let five = build_microprong(&stones, &exact).expect("five scallops");
    assert_eq!(five.count(), 5);

    let mut leaned = params;
    leaned.angle_deg = 30.0;
    let leaning = build_microprong(&stones, &leaned).expect("leaning scallop");
    assert_closed(&leaning.cutters[0], "leaned scallop");
    assert_ne!(leaning.cutters[0].vertices, cutters.cutters[0].vertices);

    let mut bad = params;
    bad.radius_mm = 0.0;
    let error = build_microprong(&stones, &bad).expect_err("zero radius");
    assert!(format!("{error:?}").contains("microprong radius"), "{error:?}");
    let error = build_microprong(&[], &params).expect_err("empty row");
    assert!(format!("{error:?}").contains("microprong gems"), "{error:?}");
}

#[test]
fn the_bright_cutter_builds_one_closed_facet_per_count() {
    let spec = round_brilliant();
    let params = BrightParams::for_gem(&spec).expect("defaults");
    let cutter = build_bright_single(&spec, &Transform::identity(), "diamond", &params)
        .expect("bright cutter");
    assert_eq!(cutter.count(), params.count);
    for mesh in &cutter.bodies {
        assert_closed(mesh, "bright facet");
    }
    assert!(cutter.total_volume_mm3 > 0.0);
    assert!(cutter.summary.contains("Bright Cutter"));
    // The count is the facet ring: a different count is a different body set.
    let mut four = params;
    four.count = 4;
    let four_facets = build_bright_single(&spec, &Transform::identity(), "diamond", &four)
        .expect("four facets");
    assert_eq!(four_facets.count(), 4);
    // The facet ring keeps the same total bright area (a wider facet each), so the count changes
    // the body set without shrinking the cutter.
    assert!(four_facets.total_volume_mm3 > 0.0);
    assert_closed(&four_facets.bodies[0], "four-facet bright cutter");
}

#[test]
fn the_bright_2d_scale_and_angle_are_typed_controls() {
    let spec = round_brilliant();
    let params = BrightParams::for_gem(&spec).expect("defaults");
    let baseline = build_bright_single(&spec, &Transform::identity(), "diamond", &params)
        .expect("baseline");
    let mut scaled = params;
    scaled.scale_2d_percent = 180.0;
    let big = build_bright_single(&spec, &Transform::identity(), "diamond", &scaled)
        .expect("scaled");
    assert!(big.total_volume_mm3 > baseline.total_volume_mm3);
    let mut small = params;
    small.scale_2d_percent = 5.0;
    let error = build_bright_single(&spec, &Transform::identity(), "diamond", &small)
        .expect_err("out of range scale");
    assert!(
        format!("{error:?}").contains("bright cutter 2d scale"),
        "{error:?}"
    );

    let mut steep = params;
    steep.angle_deg = 88.0;
    let error = build_bright_single(&spec, &Transform::identity(), "diamond", &steep)
        .expect_err("angle out of range");
    assert!(
        format!("{error:?}").contains("bright cutter angle"),
        "{error:?}"
    );

    let mut few = params;
    few.count = 2;
    let error = build_bright_single(&spec, &Transform::identity(), "diamond", &few)
        .expect_err("count out of range");
    assert!(
        format!("{error:?}").contains("bright cutter count"),
        "{error:?}"
    );
}

#[test]
fn the_bright_channel_mirrors_both_sides_of_the_row() {
    let stones = vec![
        ("a".to_owned(), baguette(), Transform::identity()),
        (
            "b".to_owned(),
            baguette(),
            Transform::translation([4.6, 0.0, 0.0]),
        ),
    ];
    let params = BrightParams::for_gem(&baguette()).expect("defaults");
    let mut one_side = params;
    one_side.mirror = false;
    let one_side = build_bright_channel(&stones, &one_side).expect("one side");
    assert_eq!(one_side.count(), 2, "one facet per stone on one side");
    let mut both = params;
    both.mirror = true;
    let mirrored = build_bright_channel(&stones, &both).expect("both sides");
    assert_eq!(mirrored.count(), 4, "two facets per stone mirrored");
    for mesh in &mirrored.bodies {
        assert_closed(mesh, "mirrored bright facet");
    }
    assert!(mirrored.total_volume_mm3 > one_side.total_volume_mm3);
    assert_eq!(mirrored.gem_ids, vec!["a", "b"]);
    let error = build_bright_channel(&[], &params).expect_err("empty row");
    assert!(
        format!("{error:?}").contains("bright cutter gems"),
        "{error:?}"
    );
}

#[test]
fn the_sweep_helper_refuses_a_degenerate_section() {
    let two = vec![vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]; 2];
    let error = sweep(&two).expect_err("two corners cannot close a section");
    assert!(format!("{error:?}").contains("cutter sweep"), "{error:?}");
    let one = vec![vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]];
    let error = sweep(&one).expect_err("one section cannot sweep");
    assert!(format!("{error:?}").contains("cutter sweep"), "{error:?}");
    let mismatched = vec![
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
    ];
    let error = sweep(&mismatched).expect_err("sections must share a corner count");
    assert!(format!("{error:?}").contains("cutter sweep"), "{error:?}");
}
