// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::Controller;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity, Extension},
};

#[test]
fn selected_om9_payload_pastes_independent_objects_in_one_undo() {
    let mut document = Document::new("clipboard");
    let a = Entity::new(
        "A",
        Geometry::Curve(Curve::Line {
            start: [1., 2., 3.],
            end: [4., 5., 6.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = a.id;
    let b = Entity::new("B", Geometry::Point([8., 9., 10.]), None);
    document.entities.insert(id, a.clone());
    document.entities.insert(b.id, b);
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let data = controller.copy_selected_om9().unwrap();
    let created = controller.paste_om9(&data).unwrap();
    assert_eq!(created.len(), 1);
    assert_ne!(created[0], id);
    assert_eq!(
        controller.document().entities[&created[0]].geometry,
        a.geometry
    );
    assert_eq!(controller.document().entities.len(), 3);
    assert!(controller.selected().contains(&created[0]));
    assert!(!controller.selected().contains(&id));
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 3);
}

#[test]
fn malformed_payload_does_not_modify_document_or_history() {
    let mut controller = Controller::new(Document::new("clipboard"), Default::default()).unwrap();
    assert!(controller.paste_om9(b"not an OM9 archive").is_err());
    assert!(controller.paste_3dm(b"not a 3DM archive", None).is_err());
    assert!(controller.document().entities.is_empty());
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn dependent_extension_is_refused_without_changing_document() {
    let mut document = Document::new("clipboard");
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [1., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    entity.extensions.insert(
        "example.test".into(),
        Extension::new(1, "{\"ref\":\"unknown\"}").unwrap(),
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    assert!(controller.copy_selected_om9().is_err());
    assert_eq!(controller.document().entities.len(), 1);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn selected_feature_ownership_is_remapped_atomically() {
    use om9_core::model::{Feature, FeatureKind};
    let mut document = Document::new("feature source");
    let a = Entity::new("first", Geometry::Point([0., 0., 0.]), None);
    let b = Entity::new("second", Geometry::Point([1., 0., 0.]), None);
    let (a_id, b_id) = (a.id, b.id);
    document.entities.insert(a_id, a);
    document.entities.insert(b_id, b);
    let feature = Feature::new("annotation", FeatureKind::Annotation, vec![a_id, b_id]);
    let old_feature = feature.id;
    document.features.insert(old_feature, feature);
    let mut from = Controller::new(document, Default::default()).unwrap();
    from.select_objects(&[a_id]).unwrap();
    assert!(from.copy_selected_om9().is_err());
    from.select_objects(&[a_id, b_id]).unwrap();
    let bytes = from.copy_selected_om9().unwrap();
    let mut target = Controller::new(Document::new("target"), Default::default()).unwrap();
    let ids = target.paste_om9(&bytes).unwrap();
    assert_eq!(target.document().features.len(), 1);
    let copied = target.document().features.values().next().unwrap();
    assert_ne!(copied.id, old_feature);
    assert_eq!(copied.label, "annotation");
    assert_eq!(copied.kind, FeatureKind::Annotation);
    assert!(copied.outputs.iter().all(|id| ids.contains(id)));
    assert_eq!(copied.outputs.len(), 2);
    assert_eq!(
        target.document().entities[&copied.outputs[0]].label,
        "first"
    );
    assert_eq!(
        target.document().entities[&copied.outputs[1]].label,
        "second"
    );
    let new_feature = copied.id;
    assert_eq!(target.coordinator().undo_len(), 1);
    target.undo().unwrap();
    assert!(target.document().features.is_empty());
    assert!(target.document().entities.is_empty());
    target.redo().unwrap();
    assert_eq!(target.document().features[&new_feature].outputs.len(), 2);
}

#[test]
fn detached_history_is_not_silently_discarded() {
    use om9_core::{
        geometry::{Transform, Unit},
        history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    };
    let mut document = Document::new("detached history");
    let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut recipe = Recipe::new(
        document.id,
        "fixture",
        vec![],
        vec![id],
        ReplayEnvironment {
            command_version: 1,
            solver: "fixture".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 0.,
                angular_rad: 0.001,
            },
        },
    );
    recipe.state = EvaluationState::Detached;
    document.history.recipes.insert(recipe.id, recipe);
    let mut source = Controller::new(document, Default::default()).unwrap();
    source.select_objects(&[id]).unwrap();
    assert!(source.copy_selected_om9().is_err());
    assert_eq!(source.document().history.recipes.len(), 1);
    assert_eq!(source.coordinator().undo_len(), 0);
}

#[test]
fn selected_clean_history_and_result_remap_and_survive_undo_redo() {
    use om9_core::{
        associative::FeaturePort,
        geometry::{GeometryKind, Transform, Unit},
        history::{Recipe, ReplayEnvironment, Tolerance},
        request::{Selection, SourceBinding},
        result::{ConversionReport, PublishedResult, ResultStatus},
    };
    let mut doc = Document::new("history source");
    let input = Entity::new("input", Geometry::Point([1., 0., 0.]), None);
    let output = Entity::new("output", Geometry::Point([2., 0., 0.]), None);
    let (input_id, output_id) = (input.id, output.id);
    doc.entities.insert(input.id, input);
    doc.entities.insert(output.id, output.clone());
    let stamp = doc.stamp(input_id).unwrap();
    let recipe = Recipe::new(
        doc.id,
        "fixture",
        vec![SourceBinding {
            role: "source".into(),
            source: stamp,
            selection: Selection::Whole,
        }],
        vec![output_id],
        ReplayEnvironment {
            command_version: 1,
            solver: "fixture".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance::om9_template(),
        },
    )
    .with_ports(
        vec![FeaturePort::geometry("source", &[GeometryKind::Point])],
        vec![FeaturePort::geometry("result", &[GeometryKind::Point])],
    );
    let old_recipe = recipe.id;
    doc.history.recipes.insert(recipe.id, recipe);
    doc.results.insert(
        output_id,
        PublishedResult {
            operation: "fixture".into(),
            geometry_revision: output.geometry_revision,
            geometry_snapshot: output.geometry,
            source_stamps: vec![stamp],
            conversion: ConversionReport::identity(GeometryKind::Point),
            recipe: Some(old_recipe),
            status: ResultStatus::Current,
        },
    );
    let mut from = Controller::new(doc, Default::default()).unwrap();
    from.select_objects(&[output_id]).unwrap();
    assert!(from.copy_selected_om9().is_err());
    from.select_objects(&[input_id, output_id]).unwrap();
    let bytes = from.copy_selected_om9().unwrap();
    let mut target =
        Controller::new(Document::new("history destination"), Default::default()).unwrap();
    let ids = target.paste_om9(&bytes).unwrap();
    assert_eq!(ids.len(), 2);
    let copied = target.document().history.recipes.values().next().unwrap();
    assert_ne!(copied.id, old_recipe);
    assert_eq!(copied.document, target.document().id);
    let new_input = copied.inputs[0].source.object;
    let new_output = copied.outputs[0];
    assert_eq!(target.document().entities[&new_input].label, "input");
    assert_eq!(target.document().entities[&new_output].label, "output");
    assert_eq!(
        target.document().stamp(new_input).unwrap(),
        copied.inputs[0].source
    );
    let result = &target.document().results[&new_output];
    assert_eq!(result.recipe, Some(copied.id));
    assert_eq!(result.source_stamps, vec![copied.inputs[0].source]);
    let edges = om9_core::associative::typed_edges(
        target.document(), &target.document().history, &Default::default(),
    ).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source, new_input);
    assert_eq!(target.coordinator().undo_len(), 1);
    target.undo().unwrap();
    assert!(target.document().entities.is_empty());
    assert!(target.document().history.recipes.is_empty());
    assert!(target.document().results.is_empty());
    target.redo().unwrap();
    assert_eq!(target.document().history.recipes.len(), 1);
    assert_eq!(target.document().results.len(), 1);
    let restored = target.document().history.recipes.values().next().unwrap();
    target.document().resolve(&restored.inputs[0].source).unwrap();
    target.document().results[&restored.outputs[0]]
        .validate(restored.outputs[0], target.document(), &Default::default())
        .unwrap();
}

#[test]
fn four_rail_profile_remaps_selected_dependencies_and_undo_redo() {
    use om9_core::rhino_semantics::{FourRailProfile, four_rail_profile_of, set_four_rail_profile};
    let mut document = Document::new("four rails");
    let mut rails = Vec::new();
    for i in 0..4 {
        let rail = Entity::new(
            format!("rail {i}"),
            Geometry::Curve(Curve::Line {
                start: [i as f64, 0., 0.],
                end: [i as f64, 1., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        rails.push(rail.id);
        document.entities.insert(rail.id, rail);
    }
    let mut profile = Entity::new(
        "profile",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [3., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let old_profile = profile.id;
    set_four_rail_profile(
        &mut profile,
        &FourRailProfile {
            rails: rails.clone().try_into().unwrap(),
            origin: [0., 0., 0.],
            normal: [0., 0., 1.],
            x_axis: [1., 0., 0.],
        },
        &Default::default(),
    )
    .unwrap();
    document.entities.insert(profile.id, profile);
    let mut source = Controller::new(document, Default::default()).unwrap();
    source.select_objects(&[old_profile]).unwrap();
    assert!(source.copy_selected_om9().is_err());
    let mut selected = rails.clone();
    selected.push(old_profile);
    source.select_objects(&selected).unwrap();
    let bytes = source.copy_selected_om9().unwrap();
    let mut target = Controller::new(Document::new("target"), Default::default()).unwrap();
    let ids = target.paste_om9(&bytes).unwrap();
    assert_eq!(ids.len(), 5);
    let copied_profile = target
        .document()
        .entities
        .values()
        .find(|e| e.label == "profile")
        .unwrap();
    let metadata = four_rail_profile_of(copied_profile, &Default::default())
        .unwrap()
        .unwrap();
    for (index, rail) in metadata.rails.iter().enumerate() {
        assert_ne!(*rail, rails[index]);
        assert_eq!(
            target.document().entities[rail].label,
            format!("rail {index}")
        );
    }
    target.undo().unwrap();
    assert!(target.document().entities.is_empty());
    target.redo().unwrap();
    assert_eq!(target.document().entities.len(), 5);
    let restored = target
        .document()
        .entities
        .values()
        .find(|e| e.label == "profile")
        .unwrap();
    assert_eq!(
        four_rail_profile_of(restored, &Default::default())
            .unwrap()
            .unwrap(),
        metadata
    );
}

#[test]
fn material_and_line_pattern_dependencies_transfer_atomically_and_avoid_collisions() {
    use om9_core::exchange::{LinePattern, LinePatternSegment, RenderMaterial};
    use om9_core::identity::{LinePatternId, MaterialId};
    let mut doc = Document::new("attributes");
    let material = RenderMaterial {
        id: MaterialId::new(),
        name: "gold".into(),
        diffuse: [200, 180, 0, 255],
    };
    let pattern = LinePattern {
        id: LinePatternId::new(),
        name: "dash".into(),
        segments: vec![LinePatternSegment {
            length_mm: 2.,
            segment_type: 1,
        }],
    };
    doc.exchange.materials.insert(material.id, material.clone());
    doc.exchange
        .line_patterns
        .insert(pattern.id, pattern.clone());
    let mut layer = om9_core::model::Layer::new("styled layer", [10, 20, 30, 255]);
    layer.material = Some(material.id);
    layer.line_pattern = Some(pattern.id);
    let layer_id = layer.id;
    doc.layers.insert(layer.id, layer);
    doc.layer_order.push(layer_id);
    let mut entity = Entity::new("styled", Geometry::Point([0., 0., 0.]), Some(layer_id));
    entity.attributes.material = Some(material.id);
    entity.attributes.line_pattern = Some(pattern.id);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut source = Controller::new(doc, Default::default()).unwrap();
    source.select_objects(&[id]).unwrap();
    let bytes = source.copy_selected_om9().unwrap();
    let mut empty = Controller::new(Document::new("empty"), Default::default()).unwrap();
    let pasted = empty.paste_om9(&bytes).unwrap();
    let attrs = &empty.document().entities[&pasted[0]].attributes;
    let new_material = attrs.material.unwrap();
    let new_pattern = attrs.line_pattern.unwrap();
    assert_ne!(new_material, material.id);
    assert_ne!(new_pattern, pattern.id);
    assert_eq!(
        empty.document().exchange.materials[&new_material].diffuse,
        material.diffuse
    );
    assert_eq!(
        empty.document().exchange.line_patterns[&new_pattern].segments,
        pattern.segments
    );
    let new_layer = attrs.layer.unwrap();
    assert_eq!(
        empty.document().layers[&new_layer].material,
        Some(new_material)
    );
    assert_eq!(
        empty.document().layers[&new_layer].line_pattern,
        Some(new_pattern)
    );
    assert_eq!(empty.coordinator().undo_len(), 1);
    empty.undo().unwrap();
    assert!(empty.document().entities.is_empty());
    assert!(empty.document().exchange.materials.is_empty());
    assert!(empty.document().exchange.line_patterns.is_empty());
    assert!(empty.document().layers.is_empty());
    empty.redo().unwrap();
    assert_eq!(
        empty.document().entities[&pasted[0]].attributes.material,
        Some(new_material)
    );
    let second = empty.paste_om9(&bytes).unwrap();
    assert_eq!(
        empty.document().entities[&second[0]].attributes.material,
        Some(new_material)
    );
    assert_eq!(empty.document().exchange.materials.len(), 1);
    assert_eq!(empty.document().exchange.line_patterns.len(), 1);
    empty.undo().unwrap();
    assert!(empty.document().entities.contains_key(&pasted[0]));
    assert_eq!(empty.document().exchange.materials.len(), 1);
    empty.redo().unwrap();
    assert!(empty.document().entities.contains_key(&second[0]));
    let mut changed = Document::new("changed");
    let mut altered = material.clone();
    altered.diffuse = [0, 0, 0, 255];
    changed
        .exchange
        .materials
        .insert(material.id, altered.clone());
    changed
        .exchange
        .line_patterns
        .insert(pattern.id, pattern.clone());
    let mut target = Controller::new(changed, Default::default()).unwrap();
    let pasted = target.paste_om9(&bytes).unwrap();
    let attrs = &target.document().entities[&pasted[0]].attributes;
    let replacement = attrs.material.unwrap();
    assert_ne!(replacement, material.id);
    assert_eq!(target.document().exchange.materials[&material.id], altered);
    assert_eq!(
        target.document().exchange.materials[&replacement].diffuse,
        material.diffuse
    );
    assert_eq!(attrs.line_pattern, Some(pattern.id));
    let pasted = source.paste_om9(&bytes).unwrap();
    let attrs = &source.document().entities[&pasted[0]].attributes;
    assert_eq!(attrs.material, Some(material.id));
    assert_eq!(attrs.line_pattern, Some(pattern.id));
}

#[test]
fn standalone_gem_copy_preserves_properties_and_remaps_gem_identity() {
    use om9_core::gem::{GemSpec, gem_entity, gem_record};
    let mut source = Document::new("gem clipboard");
    let gem = gem_entity("round", &GemSpec::round(6., 3.72).unwrap(), None).unwrap();
    let id = gem.id;
    let original = gem_record(&gem).unwrap();
    source.entities.insert(id, gem.clone());
    let mut from = Controller::new(source, Default::default()).unwrap();
    from.select_objects(&[id]).unwrap();
    let bytes = from.copy_selected_om9().unwrap();
    let mut to = Controller::new(Document::new("target"), Default::default()).unwrap();
    let ids = to.paste_om9(&bytes).unwrap();
    let copied = &to.document().entities[&ids[0]];
    let record = gem_record(copied).unwrap();
    assert_ne!(copied.id, id);
    assert_eq!(record.gem_id.0, copied.id);
    assert_eq!(record.spec, original.spec);
    assert_eq!(record.material, original.material);
    assert_eq!(record.color, original.color);
    assert_eq!(copied.geometry, gem.geometry);
    to.undo().unwrap();
    assert!(to.document().entities.is_empty());
    to.redo().unwrap();
    assert_eq!(
        gem_record(&to.document().entities[&ids[0]])
            .unwrap()
            .gem_id
            .0,
        ids[0]
    );
}

#[test]
fn inconsistent_gem_identity_is_refused_before_copy() {
    use om9_core::gem::{GemId, GemSpec, gem_entity, gem_record};
    use om9_core::identity::ObjectId;
    let mut document = Document::new("invalid gem clipboard");
    let mut gem = gem_entity("round", &GemSpec::round(6., 3.72).unwrap(), None).unwrap();
    let id = gem.id;
    let mut record = gem_record(&gem).unwrap();
    record.gem_id = GemId(ObjectId::new());
    gem.extensions.insert(
        om9_core::model::GEM_EXTENSION_SLOT.into(),
        record.as_extension().unwrap(),
    );
    document.entities.insert(id, gem);
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    assert!(controller.copy_selected_om9().is_err());
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn gem_in_block_keeps_geometry_and_gets_new_member_identity() {
    use om9_core::gem::{GemSpec, gem_entity, gem_record};
    use om9_core::model::Definition;
    let gem = gem_entity("round", &GemSpec::round(4., 2.48).unwrap(), None).unwrap();
    let original = gem.clone();
    let definition = Definition::new("gem block", vec![gem]);
    let definition_id = definition.id;
    let root = Entity::new("instance", Geometry::Instance(definition_id), None);
    let root_id = root.id;
    let mut document = Document::new("gem block");
    document.definitions.insert(definition_id, definition);
    document.entities.insert(root_id, root);
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&[root_id]).unwrap();
    let payload = controller.copy_selected_om9().unwrap();
    let copied = controller.paste_om9(&payload).unwrap();
    let Geometry::Instance(new_definition) = controller.document().entities[&copied[0]].geometry
    else {
        panic!("expected instance")
    };
    let member = &controller.document().definitions[&new_definition].members[0];
    assert_ne!(member.id, original.id);
    assert_eq!(member.geometry, original.geometry);
    assert_eq!(gem_record(member).unwrap().gem_id.0, member.id);
    controller.undo().unwrap();
    assert_eq!(controller.document().definitions.len(), 1);
}

#[test]
fn mesh_uv_payload_survives_copy_and_external_texture_is_refused() {
    use om9_core::geometry::Mesh;
    use om9_core::rhino_semantics::{MeshUv, PbrMaterial, set_mesh_uv};
    let mut source = Document::new("mesh clipboard");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let mut entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let mut uv = MeshUv {
        uvs: vec![[0., 0.], [1., 0.], [0., 1.]],
        corners: vec![],
        material: "gold".into(),
        pbr: PbrMaterial::default(),
    };
    set_mesh_uv(&mut entity, &uv, &Default::default()).unwrap();
    let id = entity.id;
    source.entities.insert(id, entity.clone());
    let mut from = Controller::new(source, Default::default()).unwrap();
    from.select_objects(&[id]).unwrap();
    let payload = from.copy_selected_om9().unwrap();
    let mut target = Controller::new(Document::new("target"), Default::default()).unwrap();
    let pasted = target.paste_om9(&payload).unwrap();
    assert_eq!(
        target.document().entities[&pasted[0]].extensions,
        entity.extensions
    );
    assert_eq!(
        target.document().entities[&pasted[0]].geometry,
        entity.geometry
    );

    uv.pbr.texture = Some("unresolved-texture".into());
    set_mesh_uv(&mut entity, &uv, &Default::default()).unwrap();
    let mut textured = Document::new("textured");
    textured.entities.insert(id, entity);
    let mut controller = Controller::new(textured, Default::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    assert!(controller.copy_selected_om9().is_err());
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn trimmed_surface_and_extrusion_metadata_survive_copy() {
    use om9_core::{
        curve_build,
        rhino_semantics::{Extrusion, TrimmedSurface, set_extrusion, set_trimmed_surface},
        surface_kernel,
    };
    let limits = Default::default();
    let surface =
        surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface {
        underlying: surface.clone(),
        outer: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        inners: vec![],
        reversed: false,
    };
    let mut patch = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut patch, &trim, &limits).unwrap();
    let profile =
        curve_build::rectangle([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 20.).unwrap();
    let extrusion = Extrusion {
        profile: profile.clone(),
        direction: [0., 0., 1.],
        height: 5.,
        capped: true,
    };
    let mut extruded = Entity::new("extrusion", Geometry::Curve(profile), None);
    set_extrusion(&mut extruded, &extrusion, &limits).unwrap();
    let ids = [patch.id, extruded.id];
    let expected = [patch.clone(), extruded.clone()];
    let mut doc = Document::new("surface semantics");
    doc.entities.insert(patch.id, patch);
    doc.entities.insert(extruded.id, extruded);
    let mut controller = Controller::new(doc, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    let bytes = controller.copy_selected_om9().unwrap();
    let created = controller.paste_om9(&bytes).unwrap();
    for original in expected {
        let copy = created
            .iter()
            .map(|id| &controller.document().entities[id])
            .find(|entity| entity.label == original.label)
            .unwrap();
        assert_eq!(copy.geometry, original.geometry);
        assert_eq!(copy.extensions, original.extensions);
    }
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
}

#[test]
fn multiple_copies_are_undone_together() {
    let mut document = Document::new("multiple");
    let mut originals = Vec::new();
    for x in [1., 4.] {
        let entity = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [x, 0., 0.],
                end: [x, 2., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        originals.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&originals).unwrap();
    let payload = controller.copy_selected_om9().unwrap();
    let pasted = controller.paste_om9(&payload).unwrap();
    assert_eq!(pasted.len(), 2);
    assert_eq!(controller.document().entities.len(), 4);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 4);
}

#[test]
fn unlayered_cross_document_paste_is_one_transaction() {
    let mut source = Document::new("source");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [1., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = line.id;
    source.entities.insert(id, line);
    let mut from = Controller::new(source, Default::default()).unwrap();
    from.select_objects(&[id]).unwrap();
    let bytes = from.copy_selected_om9().unwrap();
    let mut to = Controller::new(Document::new("target"), Default::default()).unwrap();
    assert_eq!(to.paste_om9(&bytes).unwrap().len(), 1);
    assert_eq!(to.document().entities.len(), 1);
    assert_eq!(to.coordinator().undo_len(), 1);
    to.undo().unwrap();
    assert!(to.document().entities.is_empty());
}

#[test]
fn cross_document_paste_recreates_nested_layers_in_same_undo() {
    use om9_core::model::Layer;
    let mut source = Document::new("layered source");
    let parent = Layer::new("parent", [10, 20, 30, 255]);
    let parent_id = parent.id;
    let mut child = Layer::new("child", [40, 50, 60, 255]);
    child.parent = Some(parent_id);
    let child_id = child.id;
    source.layers.insert(parent_id, parent);
    source.layers.insert(child_id, child);
    source.layer_order.extend([parent_id, child_id]);
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [1., 0., 0.],
            domain: [0., 1.],
        }),
        Some(child_id),
    );
    let id = line.id;
    source.entities.insert(id, line);
    let mut from = Controller::new(source, Default::default()).unwrap();
    from.select_objects(&[id]).unwrap();
    let bytes = from.copy_selected_om9().unwrap();
    let mut to = Controller::new(Document::new("target"), Default::default()).unwrap();
    let pasted = to.paste_om9(&bytes).unwrap();
    let child = to
        .document()
        .layers
        .get(&to.document().entities[&pasted[0]].attributes.layer.unwrap())
        .unwrap();
    assert_eq!(child.color, [40, 50, 60, 255]);
    assert_eq!(
        to.document().layers[&child.parent.unwrap()].color,
        [10, 20, 30, 255]
    );
    assert_eq!(to.document().layers.len(), 2);
    to.undo().unwrap();
    assert!(to.document().layers.is_empty());
    assert!(to.document().entities.is_empty());
    to.redo().unwrap();
    assert_eq!(to.document().layers.len(), 2);
}

#[test]
fn nested_block_paste_creates_independent_definitions_and_undoes_together() {
    use om9_core::model::Definition;
    let mut doc = Document::new("blocks");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [2., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let inner = Definition::new("inner", vec![line]);
    let inner_id = inner.id;
    let outer = Definition::new(
        "outer",
        vec![Entity::new("nested", Geometry::Instance(inner_id), None)],
    );
    let outer_id = outer.id;
    doc.definitions.insert(inner_id, inner);
    doc.definitions.insert(outer_id, outer);
    let root = Entity::new("root", Geometry::Instance(outer_id), None);
    let root_id = root.id;
    doc.entities.insert(root_id, root);
    let mut controller = Controller::new(doc, Default::default()).unwrap();
    controller.select_objects(&[root_id]).unwrap();
    let payload = controller.copy_selected_om9().unwrap();
    let copied = controller.paste_om9(&payload).unwrap();
    let Geometry::Instance(new_outer) = controller.document().entities[&copied[0]].geometry else {
        panic!("instance")
    };
    assert_ne!(new_outer, outer_id);
    let Geometry::Instance(new_inner) =
        controller.document().definitions[&new_outer].members[0].geometry
    else {
        panic!("nested instance")
    };
    assert_ne!(new_inner, inner_id);
    assert_eq!(controller.document().definitions.len(), 4);
    controller.undo().unwrap();
    assert_eq!(controller.document().definitions.len(), 2);
    controller.redo().unwrap();
    assert_eq!(controller.document().definitions.len(), 4);
}

#[test]
fn copied_group_references_only_new_objects() {
    use om9_core::model::Group;
    let mut doc = Document::new("groups");
    let mut originals = Vec::new();
    for x in [1., 2.] {
        let entity = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [x, 0., 0.],
                end: [x, 1., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        originals.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let group = Group::new("group", originals.clone());
    doc.groups.insert(group.id, group);
    let mut controller = Controller::new(doc, Default::default()).unwrap();
    controller.select_objects(&originals).unwrap();
    let bytes = controller.copy_selected_om9().unwrap();
    let copied = controller.paste_om9(&bytes).unwrap();
    assert_eq!(controller.document().groups.len(), 2);
    assert!(
        controller
            .document()
            .groups
            .values()
            .any(|group| group.members.len() == copied.len()
                && group.members.iter().all(|id| copied.contains(id)))
    );
    controller.undo().unwrap();
    assert_eq!(controller.document().groups.len(), 1);
    controller.redo().unwrap();
    assert_eq!(controller.document().groups.len(), 2);
}

#[test]
#[ignore = "requires configured native 3DM exchange executable"]
fn rhino_archive_copy_and_paste_uses_one_undo() {
    let mut document = Document::new("3dm clipboard");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [2., 3., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = line.id;
    document.entities.insert(id, line);
    let mut controller = Controller::new(document, Default::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let payload = controller.copy_selected_3dm().unwrap();
    controller.paste_3dm(&payload, None).unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires configured native 3DM exchange executable"]
fn editable_brep_copy_and_paste_with_nondefault_document_tolerance() {
    let limits = om9_core::budget::Limits::default();
    for tolerance in [0.02, 0.00001] {
        let mut document = Document::new("3dm B-rep tolerance");
        document.tolerance = Some(om9_core::history::Tolerance::try_new(tolerance, 0.01, 0.1).unwrap());
        let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
        let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let payload = controller.copy_selected_3dm().unwrap();
        assert!(matches!(controller.document().entities[&id].geometry, Geometry::Brep(_)));
        assert_eq!(controller.coordinator().undo_len(), 0);
        controller.paste_3dm(&payload, None).unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.coordinator().undo_len(), 1);
    }
}

#[test]
#[ignore = "requires configured native 3DM exchange executable"]
fn equivalent_3dm_layer_sidecar_is_verified_and_conflicts_are_atomic() {
    use om9_core::model::Layer;
    use om9_exchange::{
        three_dm::{ColorSource, ThreeDmLayerRow, ThreeDmLayerSnapshot, ThreeDmObjectRow},
        three_dm_clipboard::{LayerClipboardScope, prepare_clipboard},
    };
    let mut source = Document::new("source");
    let layer = Layer::new("Gem", [10, 20, 30, 255]);
    let layer_id = layer.id;
    source.layers.insert(layer_id, layer);
    source.layer_order.push(layer_id);
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [2., 3., 0.],
            domain: [0., 1.],
        }),
        Some(layer_id),
    );
    let id = line.id;
    source.entities.insert(id, line);
    let mut colored = Entity::new(
        "colored",
        Geometry::Curve(Curve::Line {
            start: [0., 1., 0.],
            end: [2., 4., 0.],
            domain: [0., 1.],
        }),
        Some(layer_id),
    );
    colored.attributes.color = Some([100, 90, 80, 255]);
    let colored_id = colored.id;
    source.entities.insert(colored_id, colored);
    let snapshot = ThreeDmLayerSnapshot {
        document_id: source.id.to_string(),
        generation: 1,
        layers: vec![ThreeDmLayerRow {
            source_id: layer_id.to_string(),
            parent_id: None,
            name: "Gem".into(),
            path_components: vec!["Gem".into()],
            rgb: [10, 20, 30],
            locked: false,
            visible: true,
            persistent_locked: None,
            persistent_visible: None,
        }],
        objects: vec![
            ThreeDmObjectRow {
                id: id.to_string(),
                layer_id: layer_id.to_string(),
                locked: false,
                visible: true,
                color_source: ColorSource::ByLayer,
                rgb: [10, 20, 30],
            },
            ThreeDmObjectRow {
                id: colored_id.to_string(),
                layer_id: layer_id.to_string(),
                locked: false,
                visible: true,
                color_source: ColorSource::ByObject,
                rgb: [100, 90, 80],
            },
        ],
        active_layer: None,
    };
    let mut from = Controller::new(source, Default::default()).unwrap();
    from.select_objects(&[id, colored_id]).unwrap();
    let geometry = from.copy_selected_3dm().unwrap();
    let sidecar = prepare_clipboard(&snapshot, LayerClipboardScope::Selected, &geometry).unwrap();
    let mut target = Controller::new(Document::new("target"), Default::default()).unwrap();
    target.paste_3dm(&geometry, Some(&sidecar)).unwrap();
    assert_eq!(target.document().entities.len(), 2);
    let imported = target.document().entities.values().next().unwrap();
    assert_eq!(
        target.document().layers[&imported.attributes.layer.unwrap()].color[..3],
        [10, 20, 30]
    );
    assert_eq!(target.coordinator().undo_len(), 1);
    target.undo().unwrap();
    assert!(target.document().entities.is_empty());

    let mut conflicting = snapshot.clone();
    conflicting.layers[0].rgb = [99, 20, 30];
    let sidecar =
        prepare_clipboard(&conflicting, LayerClipboardScope::Selected, &geometry).unwrap();
    let before = target.coordinator().snapshot();
    assert!(target.paste_3dm(&geometry, Some(&sidecar)).is_err());
    assert_eq!(target.coordinator().world(), &before.world);
    assert_eq!(target.coordinator().redo_len(), 1);

    let mut wrong_color = snapshot.clone();
    wrong_color.objects[1].rgb = [101, 90, 80];
    let sidecar =
        prepare_clipboard(&wrong_color, LayerClipboardScope::Selected, &geometry).unwrap();
    assert!(target.paste_3dm(&geometry, Some(&sidecar)).is_err());

    let original_sidecar =
        prepare_clipboard(&snapshot, LayerClipboardScope::Selected, &geometry).unwrap();
    let mut altered_geometry = geometry.clone();
    let last = altered_geometry.len() - 1;
    altered_geometry[last] ^= 1;
    assert!(
        target
            .paste_3dm(&altered_geometry, Some(&original_sidecar))
            .is_err()
    );

    let mut active = snapshot.clone();
    active.active_layer = Some(layer_id.to_string());
    let sidecar = prepare_clipboard(&active, LayerClipboardScope::Selected, &geometry).unwrap();
    assert!(target.paste_3dm(&geometry, Some(&sidecar)).is_err());
    let mut foreign = snapshot;
    foreign.objects[0].id = om9_core::identity::ObjectId::new().to_string();
    let sidecar = prepare_clipboard(&foreign, LayerClipboardScope::Selected, &geometry).unwrap();
    assert!(target.paste_3dm(&geometry, Some(&sidecar)).is_err());
    assert_eq!(target.coordinator().world(), &before.world);
    assert_eq!(target.coordinator().redo_len(), 1);
}
