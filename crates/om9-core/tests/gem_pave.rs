// SPDX-License-Identifier: LGPL-2.1-or-later
//! Acceptance tests for the pavé/azure/prong/sphere/springs owner (`OM9-L2-04-12`).

use om9_core::{
    Error,
    budget::Limits,
    gem::{GemId, GemInstance, GemSpec},
    gem_pave::{
        AzureParams, MAX_PAVE_GEMS, PaveFeature, PaveParams, PavePattern, PavePreview, PaveStart,
        PaveStyle, PaveStyleFile, PaveStyleLibrary, ProngKind, ProngParams, RemovalPattern,
        SphereParams, SpringParams, closed_polygon, pave_azures, pave_framework, pave_prongs,
        pave_sphere, pave_surface, refresh_pave_layout, spring_relax,
    },
    identity::ObjectId,
    surface_kernel,
};

fn plane(size: f64) -> om9_core::geometry::NurbsSurface {
    surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        size,
        size,
        &Limits::default(),
    )
    .unwrap()
}

fn gem(size: f64) -> GemSpec {
    GemSpec::round(size, size * 0.65).unwrap()
}

fn center(gem: &GemInstance) -> [f64; 3] {
    [
        gem.placement.0[0][3],
        gem.placement.0[1][3],
        gem.placement.0[2][3],
    ]
}

#[test]
fn auto_pave_fills_a_plane_and_respects_the_region() {
    let surface = plane(20.0);
    let mut params = PaveParams::new(gem(1.5)).unwrap();
    params.spacing_mm = 2.0;
    params.margin_mm = 0.5;
    params.relax_iterations = 6;
    let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
    assert!(layout.stations.len() >= 30);
    assert_eq!(layout.instances().len(), layout.stations.len());
    assert!(layout.stations.iter().all(|station| {
        let [x, y, z] = station.point;
        x > 0.0 && x < 20.0 && y > 0.0 && y < 20.0 && z.abs() < 1.0e-9
    }));
    assert!(layout.stations.iter().all(|station| {
        station.spec.length_mm >= params.min_size_mm && station.spec.length_mm <= params.max_size_mm
    }));
}

#[test]
fn every_pattern_is_distinct_and_never_empty() {
    let surface = plane(14.0);
    let mut params = PaveParams::new(gem(1.2)).unwrap();
    params.margin_mm = 0.0;
    params.relax_iterations = 0;
    let mut signatures = std::collections::BTreeSet::new();
    for pattern in [
        PavePattern::Hex,
        PavePattern::Square,
        PavePattern::Tri,
        PavePattern::HexPent,
        PavePattern::AllDirections,
        PavePattern::HexReverse,
    ] {
        params.pattern = pattern;
        let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
        assert!(!layout.stations.is_empty(), "{pattern:?}");
        signatures.insert((
            layout.stations.len(),
            (layout.stations[0].point[0] * 1000.0).round() as i64,
        ));
    }
    assert!(signatures.len() >= 4);
}

#[test]
fn angle_rotates_the_lattice_and_bad_parameters_are_refused() {
    let surface = plane(12.0);
    let mut params = PaveParams::new(gem(1.0)).unwrap();
    params.margin_mm = 0.0;
    params.relax_iterations = 0;
    let straight = pave_surface(&surface, &params, &Limits::default()).unwrap();
    params.angle_deg = 30.0;
    let turned = pave_surface(&surface, &params, &Limits::default()).unwrap();
    assert_ne!(straight.stations[0].uv, turned.stations[0].uv);
    params.flip = true;
    assert!(pave_surface(&surface, &params, &Limits::default()).is_ok());
    params.min_spacing_mm = params.spacing_mm + 1.0;
    assert!(pave_surface(&surface, &params, &Limits::default()).is_err());
    params.min_spacing_mm = 0.1;
    params.start_uv = [99.0, 99.0];
    assert!(pave_surface(&surface, &params, &Limits::default()).is_err());
}

#[test]
fn seeds_and_start_options_are_recorded() {
    let surface = plane(10.0);
    let mut params = PaveParams::new(gem(1.0)).unwrap();
    params.margin_mm = 0.0;
    params.seeds = vec![[0.2, 0.2], [0.8, 0.8]];
    params.start = PaveStart::SeedGems;
    let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
    assert!(
        layout
            .stations
            .iter()
            .any(|station| station.uv == [0.2, 0.2])
    );
    assert_eq!(params.start, PaveStart::SeedGems);
}

#[test]
fn azure_round_and_hex_profiles_are_closed_and_ordered() {
    let gems = vec![
        GemInstance::at(gem(2.0), [0.0, 0.0, 0.0]).unwrap(),
        GemInstance::at(gem(2.0), [2.8, 0.0, 0.0]).unwrap(),
    ];
    let mut params = AzureParams::new().unwrap();
    let plan = pave_azures(&gems, &params).unwrap();
    assert_eq!(plan.azures.len(), 2);
    for azure in &plan.azures {
        assert_eq!(azure.top.len(), 48);
        assert_eq!(azure.top.len(), azure.seat.len());
        assert_eq!(azure.bottom.len(), 48);
    }
    params.hexagonal = true;
    let hex = pave_azures(&gems, &params).unwrap();
    assert_eq!(hex.azures[0].top.len(), 6);
    params.seat_angle_deg = 45.0;
    let steep = pave_azures(&gems, &params).unwrap();
    let radius = |points: &[[f64; 3]]| {
        points
            .iter()
            .map(|point| point[0].hypot(point[1]))
            .fold(0.0_f64, f64::max)
    };
    assert!(radius(&steep.azures[0].bottom) < radius(&hex.azures[0].bottom));
}

#[test]
fn prongs_cover_shared_center_and_unshared_cases() {
    let gems = vec![
        GemInstance::at(gem(2.0), [0.0, 0.0, 0.0]).unwrap(),
        GemInstance::at(gem(2.0), [1.8, 0.0, 0.0]).unwrap(),
        GemInstance::at(gem(2.0), [0.9, 1.55, 0.0]).unwrap(),
    ];
    let plan = pave_prongs(&gems, &ProngParams::new().unwrap()).unwrap();
    assert!(
        plan.prongs
            .iter()
            .any(|prong| prong.kind == ProngKind::Shared)
    );
    assert!(
        plan.prongs
            .iter()
            .any(|prong| prong.kind == ProngKind::Center)
    );
    assert!(
        plan.prongs
            .iter()
            .any(|prong| prong.kind == ProngKind::Unshared)
    );
    assert!(plan.prongs.iter().all(|prong| !prong.gems.is_empty()));
}

#[test]
fn sphere_layout_uses_count_removal_and_radius() {
    let mut params = SphereParams::new(gem(1.0)).unwrap();
    params.count = Some(60);
    params.relax_amount = 3;
    let gems = pave_sphere(&params).unwrap();
    assert_eq!(gems.len(), 60);
    assert!(gems.iter().all(|gem| {
        let p = center(gem);
        (p[0].powi(2) + p[1].powi(2) + p[2].powi(2)).sqrt() - params.radius_mm < 1.0e-6
    }));
    params.removal = RemovalPattern::PatternB;
    assert!(pave_sphere(&params).unwrap().len() < 60);
    params.radius_mm = 0.1;
    assert!(pave_sphere(&params).is_err());
}

#[test]
fn springs_relax_overlaps_and_refuse_empty_input() {
    let surface = plane(12.0);
    let gems = vec![
        GemInstance::at(gem(1.0), [2.0, 2.0, 0.0]).unwrap(),
        GemInstance::at(gem(1.0), [2.15, 2.0, 0.0]).unwrap(),
        GemInstance::at(gem(1.0), [2.1, 2.1, 0.0]).unwrap(),
    ];
    let params = SpringParams::new().unwrap();
    let relaxed = spring_relax(&surface, &gems, &params, &Limits::default()).unwrap();
    assert_eq!(relaxed.len(), 3);
    for a in 0..3 {
        for b in (a + 1)..3 {
            let delta = [
                center(&relaxed[a])[0] - center(&relaxed[b])[0],
                center(&relaxed[a])[1] - center(&relaxed[b])[1],
            ];
            assert!(delta[0].hypot(delta[1]) > 1.0);
        }
    }
    assert!(spring_relax(&surface, &[], &params, &Limits::default()).is_err());
}

#[test]
fn feature_roundtrip_and_refresh_preserve_ids() {
    let surface = plane(10.0);
    let mut params = PaveParams::new(gem(1.0)).unwrap();
    params.margin_mm = 0.0;
    let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
    let mut feature = PaveFeature::new(ObjectId::new(), params);
    feature.gems = layout.stations.iter().map(|_| GemId::new()).collect();
    feature.uvs = layout.stations.iter().map(|station| station.uv).collect();
    let extension = feature.to_extension().unwrap();
    assert_eq!(PaveFeature::from_extension(&extension).unwrap(), feature);
    let refresh = refresh_pave_layout(&surface, &feature, &Limits::default()).unwrap();
    assert_eq!(refresh.stations.len(), feature.gems.len());
    assert_eq!(refresh.stations[0].gem, Some(feature.gems[0]));
    let mut broken = feature.clone();
    broken.uvs.pop();
    assert!(broken.to_json().is_err());
}

#[test]
fn framework_preview_is_bounded_and_polygons_close() {
    let surface = plane(8.0);
    let mut params = PaveParams::new(gem(1.0)).unwrap();
    params.margin_mm = 0.0;
    let layout = pave_surface(&surface, &params, &Limits::default()).unwrap();
    let edges = pave_framework(&layout);
    assert!(!edges.is_empty());
    assert!(edges.len() <= layout.stations.len() * 3);
    let points = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
    let curve = closed_polygon(&points).unwrap();
    assert!(matches!(curve, om9_core::geometry::Curve::Polycurve { .. }));
    assert!(PavePreview::Stones != PavePreview::Framework);
}

#[test]
fn budget_caps_are_explicit() {
    let surface = plane(200.0);
    let mut params = PaveParams::new(gem(0.5)).unwrap();
    params.spacing_mm = 0.2;
    params.min_spacing_mm = 0.1;
    params.margin_mm = 0.0;
    assert!(matches!(
        pave_surface(&surface, &params, &Limits::default()),
        Err(Error::Budget(_))
    ));
    let mut sphere = SphereParams::new(gem(1.0)).unwrap();
    sphere.count = Some(MAX_PAVE_GEMS + 1);
    assert!(matches!(pave_sphere(&sphere), Err(Error::Budget(_))));
}

#[test]
fn attractor_and_closed_edges_constrain_the_lattice() {
    let surface = plane(10.0);
    let mut params = PaveParams::new(gem(1.0)).unwrap();
    params.margin_mm = 0.0;
    params.relax_iterations = 0;
    let base = pave_surface(&surface, &params, &Limits::default()).unwrap();

    params.attractor = Some(om9_core::geometry::Curve::Line {
        start: [5.0, 0.0, 0.0],
        end: [5.0, 10.0, 0.0],
        domain: [0.0, 1.0],
    });
    params.start = PaveStart::Curve;
    params.attractor_strength = 1.0;
    params.attractor_radius_mm = 1.5;
    let attracted = pave_surface(&surface, &params, &Limits::default()).unwrap();
    assert!(attracted.stations.len() >= base.stations.len());
    assert!(attracted.stations.iter().any(|station| {
        let centre = station.point;
        (centre[0] - 5.0).abs() < 1.0 && centre[1] > 1.0 && centre[1] < 9.0
    }));

    let edge = closed_polygon(&[
        [2.0, 2.0, 0.0],
        [8.0, 2.0, 0.0],
        [8.0, 8.0, 0.0],
        [2.0, 8.0, 0.0],
    ])
    .unwrap();
    params.attractor = None;
    params.edge_curves = vec![edge];
    params.start = PaveStart::Edges;
    let clipped = pave_surface(&surface, &params, &Limits::default()).unwrap();
    assert!(!clipped.stations.is_empty());
    assert!(clipped.stations.iter().all(|station| {
        let [x, y, _] = station.point;
        x > 2.0 && x < 8.0 && y > 2.0 && y < 8.0
    }));

    params.edge_curves = vec![om9_core::geometry::Curve::Line {
        start: [1.0, 1.0, 0.0],
        end: [9.0, 9.0, 0.0],
        domain: [0.0, 1.0],
    }];
    assert!(pave_surface(&surface, &params, &Limits::default()).is_err());
}

#[test]
fn styles_roundtrip_and_reject_duplicate_names() {
    let mut params = PaveParams::new(gem(1.2)).unwrap();
    params.pattern = PavePattern::Square;
    params.angle_deg = 30.0;
    let style = PaveStyle::new("Shop square", params, Some(24)).unwrap();
    assert_eq!(style.quantity, Some(24));
    let file = PaveStyleFile::new(vec![style.clone()]).unwrap();
    let text = file.to_json().unwrap();
    assert_eq!(PaveStyleFile::from_json(&text).unwrap(), file);
    let path = std::env::temp_dir().join(format!("om9-pave-style-{}.json", std::process::id()));
    file.save(&path).unwrap();
    let loaded = PaveStyleFile::load(&path).unwrap();
    assert_eq!(loaded, file);
    let _ = std::fs::remove_file(path);

    let mut library = PaveStyleLibrary::builtin();
    assert!(library.get("Classic Hex").is_some());
    library.add_user(style).unwrap();
    assert!(
        library
            .add_user(
                PaveStyle::new("classic HEX", PaveParams::new(gem(1.0)).unwrap(), None).unwrap()
            )
            .is_err()
    );
    assert!(library.remove_user("shop square"));
    assert!(PaveStyleFile::from_json(r#"{"schema":"wrong","styles":[]}"#).is_err());
}
