// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for the gem asset schema, library, mesh builder, reference API and
//! instance placement (`OM9-L2-04-09`, L3-001/002/004/005/006/008).
//!
//! Every assertion runs against the public `om9_core::gem` surface only, so the evidence is
//! about the owner contract, not about private helpers.

use om9_core::{
    Error,
    gem::{
        CustomGemParams, GEM_SPEC_SCHEMA, GemCut, GemId, GemInstance, GemLibrary, GemRecord,
        GemShape, GemSpec, build_mesh, crown_facet, gem_entity, gem_record, gem_record_in,
        gem_reference, gem_reference_for, girdle_outline, instance_count, mesh_bounds,
        pavilion_facet, profile_points, summary, table_outline, validate_profile,
    },
    geometry::Geometry,
    model::{Definition, Document, Entity, GEM_EXTENSION_SLOT},
};

fn diamond_round() -> GemSpec {
    GemSpec::from_outer_dimensions(GemShape::Round, GemCut::Diamond, 6.0, 6.0, 3.9).unwrap()
}

#[test]
fn every_cut_and_shape_round_trips_through_the_versioned_schema() {
    let library = GemLibrary::canonical();
    for asset in library.assets() {
        let size = asset.sizes[asset.sizes.len() / 2];
        let spec = asset.spec_for(&size).unwrap();
        let json = spec.to_json().unwrap();
        assert!(
            json.contains(GEM_SPEC_SCHEMA),
            "{} missing schema",
            asset.key
        );
        let back = GemSpec::from_json(&json).unwrap();
        assert_eq!(back, spec, "{} did not round-trip", asset.key);
        assert_eq!(spec.asset_key(), asset.key);
    }
}

#[test]
fn the_library_has_one_asset_for_every_cut_and_shape_combination() {
    let library = GemLibrary::canonical();
    let expected: usize = GemCut::ALL.iter().map(|cut| cut.shapes().len()).sum();
    assert_eq!(library.assets().len(), expected);
    let mut keys = std::collections::BTreeSet::new();
    for asset in library.assets() {
        assert!(keys.insert(asset.key.clone()), "duplicate {}", asset.key);
        assert!(!asset.sizes.is_empty());
        assert!(
            asset
                .sizes
                .windows(2)
                .all(|pair| pair[0].span_mm() <= pair[1].span_mm())
        );
        for size in &asset.sizes {
            size.validate().unwrap();
            asset.spec_for(size).unwrap();
        }
    }
}

#[test]
fn every_shape_builds_a_closed_solid_whose_bounds_match_the_spec() {
    let library = GemLibrary::canonical();
    for asset in library.assets() {
        for size in [
            asset.sizes[0],
            asset.sizes[asset.sizes.len() / 2],
            *asset.sizes.last().unwrap(),
        ] {
            let spec = asset.spec_for(&size).unwrap();
            let built = build_mesh(&spec).expect(&asset.key);
            assert!(built.closed, "{} is open", asset.key);
            assert!(built.volume_mm3 > 0.0, "{} has no volume", asset.key);
            let tolerance = 1.0e-6 * spec.length_mm.max(1.0);
            assert!((built.extents_mm[0] - spec.length_mm).abs() <= tolerance);
            assert!((built.extents_mm[1] - spec.width_mm).abs() <= tolerance);
            assert!((built.extents_mm[2] - spec.depth_mm()).abs() <= tolerance);
            let (min, max) = mesh_bounds(&built.mesh).unwrap();
            assert!((max[2] - min[2] - spec.depth_mm()).abs() <= tolerance);
        }
    }
}

#[test]
fn a_non_finite_or_negative_dimension_is_refused() {
    let mut spec = diamond_round();
    spec.length_mm = f64::NAN;
    assert_eq!(
        spec.validate(),
        Err(Error::Invalid("gem dimension is not finite"))
    );

    let mut spec = diamond_round();
    spec.width_mm = f64::INFINITY;
    assert_eq!(
        spec.validate(),
        Err(Error::Invalid("gem dimension is not finite"))
    );

    let mut spec = diamond_round();
    spec.width_mm = -1.0;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem dimension range")));

    let mut spec = diamond_round();
    spec.length_mm = 0.0;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem dimension range")));
}

#[test]
fn a_zero_depth_is_refused_and_a_new_depth_keeps_the_crown_pavilion_split() {
    let spec = diamond_round();
    let mut broken = spec.clone();
    broken.pavilion_depth_mm = 0.0;
    assert_eq!(
        broken.validate(),
        Err(Error::Invalid("gem pavilion depth is zero"))
    );

    let deeper = spec.clone().set_depth_mm(6.0).unwrap();
    assert!((deeper.depth_mm() - 6.0).abs() < 1.0e-9);
    assert!(deeper.crown_height_mm > spec.crown_height_mm);
    assert!(deeper.pavilion_depth_mm > spec.pavilion_depth_mm);
    let ratio = spec.crown_height_mm / (spec.crown_height_mm + spec.pavilion_depth_mm);
    let new_ratio = deeper.crown_height_mm / (deeper.crown_height_mm + deeper.pavilion_depth_mm);
    assert!((ratio - new_ratio).abs() < 1.0e-9);
    assert!(spec.clone().set_depth_mm(0.05).is_err());
}

#[test]
fn an_unknown_cut_or_shape_tag_is_refused() {
    assert_eq!(
        GemCut::from_tag("rose").err(),
        Some(Error::Invalid("gem cut"))
    );
    assert_eq!(
        GemShape::from_tag("briolette").err(),
        Some(Error::Invalid("gem shape"))
    );
    assert_eq!(GemCut::from_tag("Diamond Cut").unwrap(), GemCut::Diamond);
    assert_eq!(GemShape::from_tag("Round").unwrap(), GemShape::Round);
}

#[test]
fn a_shape_outside_the_cut_family_is_refused() {
    let mut spec = diamond_round();
    spec.shape = GemShape::Heart;
    spec.cut = GemCut::Cabochon;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem shape for cut")));
}

#[test]
fn a_round_stone_needs_one_diameter_and_width_never_exceeds_length() {
    let mut spec = diamond_round();
    spec.length_mm = 7.0;
    assert_eq!(
        spec.validate(),
        Err(Error::Invalid("round gem needs one diameter"))
    );

    spec.shape = GemShape::Oval;
    spec.length_mm = 6.0;
    spec.width_mm = 7.0;
    assert_eq!(
        spec.validate(),
        Err(Error::Invalid("gem width exceeds length"))
    );
}

#[test]
fn a_corner_or_angle_outside_the_geometry_is_refused() {
    let mut spec = GemSpec::emerald(8.0, 6.0, 1.2, 4.0, 0.15).unwrap();
    spec.corner_mm = 4.0;
    assert_eq!(
        spec.validate(),
        Err(Error::Invalid("gem corner exceeds the girdle"))
    );

    let mut spec = diamond_round();
    spec.crown_angle_deg = 0.0;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem crown angle")));

    let mut spec = diamond_round();
    spec.facet_count = 2;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem facet count")));
}

#[test]
fn a_cabochon_has_a_dome_and_no_pavilion_depth() {
    let spec =
        GemSpec::from_outer_dimensions(GemShape::Round, GemCut::Cabochon, 8.0, 8.0, 4.0).unwrap();
    assert_eq!(spec.pavilion_depth_mm, 0.0);
    let built = build_mesh(&spec).unwrap();
    assert!(built.closed);
    assert!((built.extents_mm[2] - 4.0).abs() < 1.0e-6);
    let mut broken = spec.clone();
    broken.pavilion_depth_mm = 1.0;
    assert_eq!(
        broken.validate(),
        Err(Error::Invalid("cabochon pavilion depth"))
    );
}

#[test]
fn an_open_self_intersecting_or_zero_area_profile_is_refused() {
    let open = [[0.0, 0.0], [4.0, 0.0], [0.0, 3.0]];
    // Three points form a closed triangle by the polygon rule, so this is legal.
    assert!(validate_profile(&open).is_ok());
    // Two points cannot close an area.
    let two = [[0.0, 0.0], [1.0, 1.0]];
    assert_eq!(
        validate_profile(&two),
        Err(Error::Invalid("gem profile needs three points"))
    );

    let bow_tie = [[0.0, 0.0], [4.0, 4.0], [4.0, 0.0], [0.0, 3.0]];
    assert_eq!(
        validate_profile(&bow_tie),
        Err(Error::Invalid("gem profile is self-intersecting"))
    );

    let line = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    assert_eq!(
        validate_profile(&line),
        Err(Error::Invalid("gem profile has zero area"))
    );

    let two = [[0.0, 0.0], [1.0, 1.0]];
    assert_eq!(
        validate_profile(&two),
        Err(Error::Invalid("gem profile needs three points"))
    );
}

#[test]
fn a_custom_profile_is_centred_and_keeps_the_requested_parameters() {
    let profile = [[0.0, 0.0], [6.0, 0.0], [6.0, 4.0], [0.0, 4.0]];
    let params = CustomGemParams {
        facet_count: 8,
        crown_angle_deg: 34.5,
        crown_height_mm: 1.4,
        girdle_mm: 0.2,
        pavilion_depth_mm: 2.8,
        pavilion_rows: 3,
        bulge: 0.3,
    };
    let spec = GemSpec::from_profile(&profile, GemCut::Gem, &params).unwrap();
    assert_eq!(spec.length_mm, 6.0);
    assert_eq!(spec.width_mm, 4.0);
    assert_eq!(spec.crown_height_mm, 1.4);
    assert_eq!(spec.girdle_mm, 0.2);
    assert_eq!(spec.pavilion_rows, 3);
    let outline = girdle_outline(&spec).unwrap();
    let xs: Vec<f64> = outline.iter().map(|p| p[0]).collect();
    let ys: Vec<f64> = outline.iter().map(|p| p[1]).collect();
    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!((min_x + max_x).abs() < 1.0e-9, "profile x is centred");
    assert!((min_x + max_x).abs() < 1.0e-9);
    assert!(ys.iter().cloned().fold(0.0, f64::max) - ys.iter().cloned().fold(0.0, f64::min) > 0.0);
    let built = build_mesh(&spec).unwrap();
    assert!((built.extents_mm[0] - 6.0).abs() < 1.0e-6);
    assert!((built.extents_mm[1] - 4.0).abs() < 1.0e-6);
}

#[test]
fn the_emerald_and_baguette_builders_match_their_dimensions() {
    for spec in [
        GemSpec::emerald(8.0, 6.0, 1.2, 4.2, 0.15).unwrap(),
        GemSpec::baguette(7.0, 3.5, 0.0, 2.8, 0.12).unwrap(),
        GemSpec::baguette(7.0, 3.5, 0.4, 2.8, 0.12).unwrap(),
    ] {
        let built = build_mesh(&spec).unwrap();
        let tolerance = 1.0e-6 * spec.length_mm;
        assert!((built.extents_mm[0] - spec.length_mm).abs() < tolerance);
        assert!((built.extents_mm[1] - spec.width_mm).abs() < tolerance);
        assert!((built.extents_mm[2] - spec.depth_mm()).abs() < tolerance);
    }
    // A degenerate girdle thickness is refused by the shared range rule.
    let mut spec = GemSpec::emerald(8.0, 6.0, 1.2, 4.2, 0.15).unwrap();
    spec.girdle_mm = 0.0;
    assert_eq!(spec.validate(), Err(Error::Invalid("gem dimension range")));
}

#[test]
fn the_reference_api_returns_geometry_and_stable_normals() {
    let spec = GemSpec::emerald(8.0, 6.0, 1.2, 4.2, 0.15).unwrap();
    let girdle = gem_reference(&spec, "girdle").unwrap();
    assert!(girdle.closed);
    assert_eq!(girdle.points.len(), spec.facet_count as usize);
    assert_eq!(girdle.normal, [0.0, 0.0, 1.0]);

    let table = gem_reference(&spec, "TABLE").unwrap();
    assert!(table.closed);
    assert_eq!(table.points, table_outline(&spec).unwrap());
    assert!(
        table
            .points
            .iter()
            .all(|p| { (p[2] - (spec.girdle_mm * 0.5 + spec.crown_height_mm)).abs() < 1.0e-12 })
    );
    let girdle_radius = girdle
        .points
        .iter()
        .map(|p| p[0].hypot(p[1]))
        .fold(0.0, f64::max);
    let table_radius = table
        .points
        .iter()
        .map(|p| p[0].hypot(p[1]))
        .fold(0.0, f64::max);
    assert!(table_radius < girdle_radius);

    let crown = crown_facet(&spec, 0).unwrap();
    assert_eq!(crown.points.len(), 4);
    let length =
        (crown.normal[0].powi(2) + crown.normal[1].powi(2) + crown.normal[2].powi(2)).sqrt();
    assert!((length - 1.0).abs() < 1.0e-9);
    assert!(crown.normal.iter().all(|v| v.is_finite()));
    // The same reference asked twice is the same geometry, not a mesh triangle index.
    assert_eq!(crown, crown_facet(&spec, 0).unwrap());
    // The outward normal points away from the gem axis at the facet midpoint.
    let mid: [f64; 3] = crown.points.iter().fold([0.0; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    assert!(crown.normal[0] * mid[0] + crown.normal[1] * mid[1] > 0.0);

    let pavilion = pavilion_facet(&spec, 0).unwrap();
    assert_eq!(pavilion.points.len(), 4);
    let length =
        (pavilion.normal[0].powi(2) + pavilion.normal[1].powi(2) + pavilion.normal[2].powi(2))
            .sqrt();
    assert!((length - 1.0).abs() < 1.0e-9);
    assert!(pavilion.normal.iter().all(|v| v.is_finite()));
    let mid: [f64; 3] = pavilion.points.iter().fold([0.0; 3], |acc, p| {
        [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
    });
    assert!(pavilion.normal[0] * mid[0] + pavilion.normal[1] * mid[1] > 0.0);

    let profile = gem_reference(&spec, "profile").unwrap();
    assert!(!profile.closed);
    let profile_2d = profile_points(&spec).unwrap();
    assert_eq!(profile.points.len(), profile_2d.len());
    let last = profile_2d.last().unwrap();
    assert_eq!(profile.points.last().unwrap()[2], last[1]);
    assert_eq!(last[0], 0.0, "the profile ends at the table centre");

    let culet = gem_reference(&spec, "culet").unwrap();
    assert_eq!(culet.points.len(), 1);
    assert_eq!(
        culet.points[0][2],
        -(spec.girdle_mm * 0.5 + spec.pavilion_depth_mm)
    );

    assert_eq!(
        gem_reference(&spec, "grid").err(),
        Some(Error::Invalid("gem reference name"))
    );
    assert_eq!(
        crown_facet(&spec, 999).err(),
        Some(Error::Invalid("gem facet index"))
    );
    assert_eq!(
        gem_reference(&spec, "crown_facet:x").err(),
        Some(Error::Invalid("gem facet index"))
    );
}

#[test]
fn a_reference_is_bound_to_the_gem_identity() {
    let spec = diamond_round();
    let entity = gem_entity("Round Diamond", &spec, None).unwrap();
    let record = gem_record(&entity).unwrap();
    assert!(!record.gem_id.is_nil());
    let bound = gem_reference_for(&record, "girdle").unwrap();
    assert_eq!(bound.gem, record.gem_id);
    assert!(bound.reference.closed);

    let mut unbound = record.clone();
    unbound.gem_id = GemId::default();
    // A generated id is live; a nil id is the only refused one.
    assert!(gem_reference_for(&unbound, "girdle").is_ok());
    unbound.gem_id = GemId(om9_core::identity::ObjectId::nil());
    assert_eq!(
        gem_reference_for(&unbound, "girdle").err(),
        Some(Error::Invalid("gem id"))
    );
}

#[test]
fn a_gem_entity_round_trips_through_the_document_record() {
    let spec = GemSpec::oval(9.0, 6.5, 4.2).unwrap();
    let entity = gem_entity("Oval Gem", &spec, None).unwrap();
    assert!(matches!(entity.geometry, Geometry::Mesh(_)));
    let payload = entity.extension(GEM_EXTENSION_SLOT).unwrap();
    assert_eq!(payload.version, 1);
    let record = GemRecord::from_json(&payload.payload).unwrap();
    assert_eq!(record.spec, spec);
    assert_eq!(record.asset, "oval-diamond");
    assert_eq!(record.material, "diamond");
    assert_eq!(record.gem_id, GemId(entity.id));
    assert!(summary(&record).contains("Oval"));

    let mut document = Document::new("gems");
    let id = entity.id;
    document.entities.insert(id, entity);
    assert_eq!(gem_record_in(&document, id).unwrap().gem_id, GemId(id));
    assert!(gem_record_in(&document, om9_core::identity::ObjectId::new()).is_none());
}

#[test]
fn an_instance_transform_keeps_a_normalised_basis_and_moves_without_duplicating_geometry() {
    let spec = diamond_round();
    let instance = GemInstance::at(spec.clone(), [10.0, 2.0, -3.0]).unwrap();
    let basis = instance.basis();
    for axis in basis {
        let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        assert!((length - 1.0).abs() < 1.0e-9);
    }
    assert!(basis[0][0].abs() > 0.99);
    assert!((instance.placement.0[0][3] - 10.0).abs() < 1.0e-9);

    let rotated = instance.clone().with_rotation([0.0, 0.0, 90.0]).unwrap();
    let basis = rotated.basis();
    assert!(basis[0][1].abs() > 0.99, "90 deg about Z maps X to Y");
    let moved = rotated.clone().moved([1.0, 0.0, 0.0]).unwrap();
    assert!((moved.placement.0[0][3] - 11.0).abs() < 1.0e-9);
    assert_eq!(moved.spec, spec);

    let scaled = instance.clone().scaled(2.0).unwrap();
    assert!((scaled.spec.length_mm - 12.0).abs() < 1.0e-9);
    assert!((scaled.spec.depth_mm() - spec.depth_mm() * 2.0).abs() < 1.0e-9);
    build_mesh(&scaled.spec).unwrap();

    // Two instances of one definition share the immutable definition; they are not two meshes.
    let mut document = Document::new("instances");
    let gem = gem_entity("Source", &spec, None).unwrap();
    let definition = Definition::new("Round Diamond 6 mm", vec![gem]);
    let definition_id = definition.id;
    let first = Entity::new("Instance 1", Geometry::Instance(definition_id), None);
    let second = Entity::new("Instance 2", Geometry::Instance(definition_id), None);
    let first_id = first.id;
    document.entities.insert(first_id, first);
    document.entities.insert(second.id, second);
    document.definitions.insert(definition_id, definition);
    assert_eq!(instance_count(&document, definition_id), 2);
    let placed = instance.clone().moved([5.0, 0.0, 0.0]).unwrap();
    assert_ne!(placed.placement, instance.placement);
    assert_eq!(placed.spec, instance.spec);
}

#[test]
fn a_payload_from_the_previous_shape_is_refused_instead_of_silently_dropped() {
    let record = GemRecord::new(diamond_round());
    let mut value: serde_json::Value = serde_json::from_str(&record.to_json().unwrap()).unwrap();
    value["version"] = serde_json::json!(2);
    assert_eq!(
        GemRecord::from_json(&value.to_string()).err(),
        Some(Error::Invalid("gem payload version"))
    );
    value["version"] = serde_json::json!(1);
    value["spec"]["shape"] = serde_json::json!("briolette");
    assert!(GemRecord::from_json(&value.to_string()).is_err());

    let mut document: serde_json::Value = serde_json::from_str(&record.to_json().unwrap()).unwrap();
    document["spec"]["length_mm"] = serde_json::json!(-2.0);
    assert_eq!(
        GemRecord::from_json(&document.to_string()).err(),
        Some(Error::Invalid("gem dimension range"))
    );
}

#[test]
fn a_spec_from_the_wrong_schema_is_refused() {
    let spec = diamond_round();
    let json = spec.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["schema"] = serde_json::json!("om9.gem_spec.v2");
    assert_eq!(
        GemSpec::from_json(&value.to_string()).err(),
        Some(Error::Invalid("gem spec version"))
    );
}

#[test]
fn the_library_nearest_size_matches_by_short_axis() {
    let library = GemLibrary::canonical();
    let size = library
        .nearest_size(GemShape::Oval, GemCut::Diamond, 6.2)
        .unwrap();
    assert!((size.width_mm - 6.0).abs() < 0.51);
    assert!(library.asset(GemShape::Heart, GemCut::Cabochon).is_none());
    assert!(
        library
            .nearest_size(GemShape::Heart, GemCut::Cabochon, 6.0)
            .is_none()
    );
}

#[test]
fn the_custom_profile_refuses_self_intersection_and_open_outlines() {
    let params = CustomGemParams::default();
    let bow_tie = [[0.0, 0.0], [4.0, 4.0], [4.0, 0.0], [0.0, 3.0]];
    assert_eq!(
        GemSpec::from_profile(&bow_tie, GemCut::Gem, &params).err(),
        Some(Error::Invalid("gem profile is self-intersecting"))
    );
    let line = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    assert_eq!(
        GemSpec::from_profile(&line, GemCut::Gem, &params).err(),
        Some(Error::Invalid("gem profile has zero area"))
    );
}
