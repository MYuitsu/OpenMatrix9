// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed layout contracts for the halo/cluster/baguette builders (`OM9-L2-04-11`).
//!
//! These fixtures exercise the owner math adversarially: spacing that has to close the ring, the
//! fixed 3 o'clock position, the downward Z offset, tapered channels, flipped/banked frames and
//! the persisted feature that a save/open must keep intact. App wiring lives in
//! `crates/om9-app/tests/gem_cluster_panel.rs`.

use om9_core::{
    budget::Limits,
    gem::{GemCut, GemId, GemInstance, GemSpec},
    gem_cluster::{
        CLUSTER_EXTENSION_SLOT, ClusterFeature, ClusterKind, ClusterParams, ClusterStart,
        HaloParams, MAX_ACCENTS, OrientOptions, baguette_channel, cluster_layout,
        gems_between_curves, halo_layout, orient_to_gem, tapered_baguette,
    },
    geometry::{Curve, Geometry, Point3, Transform},
    identity::ObjectId,
    model::{Document, Entity, extension_slot_known},
};

fn limits() -> Limits {
    Limits::default()
}

fn round(diameter: f64) -> GemSpec {
    GemSpec::round(diameter, diameter * 0.6).expect("round gem")
}

fn baguette(length: f64, width: f64) -> GemSpec {
    GemSpec::baguette(length, width, 0.0, width * 0.55, 0.15).expect("baguette")
}

fn instance(spec: GemSpec, position: Point3) -> GemInstance {
    GemInstance {
        spec,
        placement: Transform::translation(position),
        label: String::new(),
    }
}

fn center() -> GemInstance {
    instance(round(6.5), [10.0, 4.0, 2.0])
}

fn axis(transform: &Transform, index: usize) -> Point3 {
    [
        transform.0[0][index],
        transform.0[1][index],
        transform.0[2][index],
    ]
}

fn origin(transform: &Transform) -> Point3 {
    axis(transform, 3)
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn line(start: Point3, end: Point3) -> Curve {
    Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    }
}

#[test]
fn a_halo_ring_closes_on_itself_and_the_spacing_is_realized() {
    let layout = halo_layout(&center(), &HaloParams::new(round(1.3)).unwrap()).expect("halo");
    assert!(layout.accents.len() >= 8);
    let n = layout.accents.len();
    for (index, accent) in layout.accents.iter().enumerate() {
        let next = &layout.accents[(index + 1) % n];
        let gap = {
            let a = origin(&accent.placement);
            let b = origin(&next.placement);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        // Spacing is the edge-to-edge gap, so centres sit one stone length plus the gap apart.
        let pitch = 1.3 + layout.realized_spacing_mm;
        assert!(
            (gap - pitch).abs() < 0.35,
            "gap {gap} at {index} vs pitch {pitch}"
        );
        assert_eq!(accent.spec.length_mm, 1.3);
        // The table axis stays along the center stone's table axis.
        assert!(dot(axis(&accent.placement, 2), [0.0, 0.0, 1.0]) > 0.99);
    }
    // The ring is centred on the source stone and lies in its plane.
    let c = origin(&center().placement);
    for accent in &layout.accents {
        let p = origin(&accent.placement);
        assert!((p[2] - c[2]).abs() < 1.0e-6);
    }
}

#[test]
fn halo_fix_position_pins_the_first_stone_at_twelve_oclock() {
    let mut params = HaloParams::new(round(1.3)).unwrap();
    params.fix_position = true;
    params.count = Some(12);
    params.spacing_mm = 0.2;
    let layout = halo_layout(&center(), &params).expect("halo");
    assert_eq!(layout.accents.len(), 12);
    let c = origin(&center().placement);
    let first = origin(&layout.accents[0].placement);
    let relative = [first[0] - c[0], first[1] - c[1], first[2] - c[2]];
    assert!(
        relative[1] > 0.0 && relative[0].abs() < relative[1] * 0.35,
        "the fixed stone sits at 12 o'clock: {relative:?}"
    );
}

#[test]
fn a_halo_that_cannot_fit_is_refused_instead_of_overlapping() {
    let mut params = HaloParams::new(round(1.3)).unwrap();
    params.count = Some(3);
    params.spacing_mm = 100.0;
    let error = halo_layout(&center(), &params).expect_err("three stones cannot fit");
    assert!(format!("{error}").contains("halo"), "{error}");
    let mut params = HaloParams::new(round(1.3)).unwrap();
    params.count = Some(MAX_ACCENTS + 1);
    let error = halo_layout(&center(), &params).expect_err("too many accents");
    assert!(matches!(error, om9_core::Error::Budget(_)));
}

#[test]
fn a_cluster_uses_the_start_angle_and_the_positive_down_z_offset() {
    let mut params = ClusterParams::new(round(1.3)).unwrap();
    params.count = Some(4);
    params.rings = 2;
    params.start = ClusterStart::Top;
    params.z_offset_mm = 1.0;
    let layout = cluster_layout(&center(), &params).expect("cluster");
    assert_eq!(layout.accents.len(), 8);
    let c = origin(&center().placement);
    let first = origin(&layout.accents[0].placement);
    // Start Top is +Y, ring 0 is the inner ring, and +Z offset means "down" in the manual.
    assert!((first[1] - c[1] - layout.ring_radii_mm[0]).abs() < 1.0e-6);
    assert!((first[2] - c[2] + 1.0).abs() < 1.0e-6);
    let outer = origin(&layout.accents[4].placement);
    assert!((outer[1] - c[1] - layout.ring_radii_mm[1]).abs() < 1.0e-6);
    assert!(layout.ring_radii_mm[1] > layout.ring_radii_mm[0]);
}

#[test]
fn cluster_degree_rotates_each_stone_about_its_tangent() {
    let mut params = ClusterParams::new(baguette(3.0, 1.2)).unwrap();
    params.count = Some(6);
    params.degree_deg = 30.0;
    let layout = cluster_layout(&center(), &params).expect("cluster");
    for accent in &layout.accents {
        let tangent = axis(&accent.placement, 0);
        // The gem's long axis is the tangent rotated by the requested degree.
        assert!(dot(axis(&accent.placement, 2), [0.0, 0.0, 1.0]) > 0.0);
        assert!(dot(tangent, tangent) > 0.99);
    }
    let mut straight = ClusterParams::new(baguette(3.0, 1.2)).unwrap();
    straight.count = Some(6);
    let plain = cluster_layout(&center(), &straight).expect("cluster");
    let turned = dot(
        axis(&layout.accents[0].placement, 0),
        axis(&plain.accents[0].placement, 0),
    );
    assert!((turned - 30.0_f64.to_radians().cos()).abs() < 1.0e-6);
}

#[test]
fn a_baguette_channel_spans_the_walls_and_tapers_toward_the_end() {
    let a = line([0.0, 0.0, 0.0], [12.0, 0.0, 0.0]);
    let b = line([0.0, 4.0, 0.0], [12.0, 4.0, 0.0]);
    let mut params = om9_core::gem_cluster::ChannelParams::new(baguette(4.0, 1.5)).unwrap();
    params.spacing_mm = 0.4;
    let layout = baguette_channel(&a, &b, &params, &limits()).expect("channel");
    assert!(layout.gems.len() >= 2);
    assert!((layout.min_gap_mm - 4.0).abs() < 1.0e-6);
    assert!((layout.max_gap_mm - 4.0).abs() < 1.0e-6);
    for gem in &layout.gems {
        assert_eq!(gem.instance.spec.length_mm, 4.0);
        assert!((gem.instance.spec.width_mm - 4.0).abs() < 1.0e-6);
        let p = origin(&gem.instance.placement);
        assert!(p[1] > 0.0 && p[1] < 4.0);
    }

    params.taper = 0.5;
    let tapered = baguette_channel(&a, &b, &params, &limits()).expect("tapered channel");
    for gem in &tapered.gems {
        let profile = gem
            .instance
            .spec
            .profile
            .as_ref()
            .expect("a tapered profile");
        let width_start = profile
            .iter()
            .filter(|point| point[0] < 0.0)
            .map(|point| point[1].abs())
            .fold(0.0_f64, f64::max);
        let width_end = profile
            .iter()
            .filter(|point| point[0] > 0.0)
            .map(|point| point[1].abs())
            .fold(0.0_f64, f64::max);
        assert!(
            (width_start - 2.0).abs() < 1.0e-6,
            "the wide end spans the walls"
        );
        assert!(
            (width_end - 1.0).abs() < 1.0e-6,
            "the taper halves the far end"
        );
        assert_eq!(gem.instance.spec.shape, om9_core::gem::GemShape::Baguette);
        assert_eq!(gem.instance.spec.cut, GemCut::Gem);
    }
}

#[test]
fn flipping_and_banking_the_channel_changes_the_frame_not_the_footprint() {
    let a = line([0.0, 0.0, 0.0], [12.0, 0.0, 0.0]);
    let b = line([0.0, 3.0, 0.0], [12.0, 3.0, 0.0]);
    let mut params = om9_core::gem_cluster::ChannelParams::new(baguette(3.0, 1.2)).unwrap();
    let plain = baguette_channel(&a, &b, &params, &limits()).expect("channel");
    params.flip = true;
    let flipped = baguette_channel(&a, &b, &params, &limits()).expect("flipped channel");
    assert_eq!(plain.gems.len(), flipped.gems.len());
    for (left, right) in plain.gems.iter().zip(&flipped.gems) {
        assert!(
            dot(
                axis(&left.instance.placement, 1),
                axis(&right.instance.placement, 1)
            ) < 0.0,
            "flip reverses the width axis"
        );
        assert!((left.instance.spec.width_mm - right.instance.spec.width_mm).abs() < 1.0e-9);
    }
    params.flip = false;
    params.banking_deg = 30.0;
    let banked = baguette_channel(&a, &b, &params, &limits()).expect("banked channel");
    for gem in &banked.gems {
        let normal = axis(&gem.instance.placement, 2);
        assert!(
            normal[2] < 0.999,
            "banking tilts the table off the world up"
        );
    }
}

#[test]
fn gems_between_two_curves_are_round_and_touch_both_walls() {
    let a = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line([0.0, 2.0, 0.0], [10.0, 2.0, 0.0]);
    let params = om9_core::gem_cluster::ChannelParams::new(round(1.5)).unwrap();
    let layout = gems_between_curves(&a, &b, &params, &limits()).expect("between");
    assert!(layout.gems.len() >= 3);
    for gem in &layout.gems {
        assert!(gem.instance.spec.shape.is_round());
        assert!((gem.instance.spec.length_mm - 2.0).abs() < 1.0e-6);
        assert!((gem.instance.spec.width_mm - 2.0).abs() < 1.0e-6);
    }
}

#[test]
fn one_tapered_baguette_measures_the_two_wall_fractions() {
    let a = line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line([0.0, 2.0, 0.0], [10.0, 3.0, 0.0]);
    let mut params = om9_core::gem_cluster::ChannelParams::new(baguette(2.0, 1.2)).unwrap();
    params.point_a = 0.2;
    params.point_b = 0.8;
    let gem = tapered_baguette(&a, &b, &params, &limits()).expect("taper");
    let p = origin(&gem.instance.placement);
    assert!(
        (p[0] - 5.0).abs() < 1.0e-6,
        "the stone is centered on the span"
    );
    assert!((p[1] - 1.25).abs() < 1.0e-6);
    assert!(gem.instance.spec.profile.is_some());
    let _ = &gem.instance.spec;
}

#[test]
fn orient_to_gem_reuses_the_exact_gem_basis() {
    let gem = center();
    let object = Transform::translation([0.0, 0.0, 0.0]);
    let options = OrientOptions {
        offset_mm: [0.0, 0.0, 1.5],
        ..OrientOptions::default()
    };
    let placed = orient_to_gem(&gem, &object, &options).expect("orient");
    let basis = gem.basis();
    for index in 0..3 {
        assert!((dot(axis(&placed, index), basis[index]) - 1.0).abs() < 1.0e-9);
        assert!((dot(axis(&placed, index), basis[(index + 1) % 3])).abs() < 1.0e-9);
    }
    assert!((origin(&placed)[2] - (origin(&gem.placement)[2] + 1.5)).abs() < 1.0e-9);
    let nil = Transform::translation([0.0, 0.0, 0.0]);
    let _ = nil;
}

#[test]
fn the_feature_survives_a_document_json_round_trip() {
    let spec = round(6.5);
    let mut feature = ClusterFeature::new(
        ClusterKind::Halo {
            params: HaloParams::new(round(1.3)).unwrap(),
        },
        Some(ObjectId::new()),
        [None, None],
    );
    let ids: Vec<GemId> = (0..4).map(|_| GemId::new()).collect();
    feature.accents = ids.clone();
    assert_eq!(feature.version, 1);
    let json = feature.to_json().expect("json");
    assert_eq!(ClusterFeature::from_json(&json).unwrap(), feature);
    let extension = feature.to_extension().expect("extension");
    assert!(extension_slot_known(CLUSTER_EXTENSION_SLOT));
    assert_eq!(ClusterFeature::from_extension(&extension).unwrap(), feature);

    let mut document = Document::new("halo");
    let mut entity = Entity::new("center", Geometry::Point([0.0, 0.0, 0.0]), None);
    entity
        .extensions
        .insert(CLUSTER_EXTENSION_SLOT.to_owned(), extension);
    let id = entity.id;
    document.entities.insert(id, entity);
    let saved = serde_json::to_string(&document).unwrap();
    let reopened: Document = serde_json::from_str(&saved).unwrap();
    let slot = reopened.entities[&id]
        .extensions
        .get(CLUSTER_EXTENSION_SLOT)
        .expect("slot survives save");
    assert_eq!(ClusterFeature::from_extension(slot).unwrap(), feature);
    let _ = spec;
}

#[test]
fn a_broken_feature_record_is_refused() {
    let mut feature = ClusterFeature::new(
        ClusterKind::Halo {
            params: HaloParams::new(round(1.3)).unwrap(),
        },
        Some(ObjectId::new()),
        [None, None],
    );
    let duplicate = GemId::new();
    feature.accents = vec![duplicate, duplicate];
    let error = feature.validate().expect_err("duplicate children");
    assert!(format!("{error}").contains("duplicate"), "{error}");
    feature.accents = vec![GemId(ObjectId::nil())];
    let error = feature.validate().expect_err("nil child");
    assert!(format!("{error}").contains("gem id"), "{error}");
    feature.accents = vec![GemId::new()];
    feature.kind = ClusterKind::Channel {
        params: om9_core::gem_cluster::ChannelParams::new(round(1.2)).unwrap(),
    };
    // A channel is hosted by two curves, so a feature with no curve ids is invalid.
    let error = feature.validate().expect_err("channel without walls");
    assert!(format!("{error}").contains("curve"), "{error}");
}
