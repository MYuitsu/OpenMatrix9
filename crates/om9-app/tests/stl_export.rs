#![cfg(feature = "occt-brep")]
//! File ▸ Export Selected STL (`OM9-FILE-001`, codec `OM9-L3-01-12-008`).
//! The binary writer's own policy is tested in `om9-exchange`; these tests prove the document
//! route: menu/CMD, selection, B-Rep tessellation, vertex welding, millimetres and atomic write.
use om9_app::session::{Controller, DispatchOutcome, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Geometry, Mesh, Transform};
use om9_core::model::{Document, Entity};
use std::path::PathBuf;

fn controller() -> Controller {
    Controller::new(Document::with_template_tolerance("stl"), Limits::default()).unwrap()
}

#[test]
fn independent_export_profile_controls_sphere_mesh_and_rejects_invalid_values() {
    use om9_app::session::StlExportProfile;
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5.0, 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("profile sphere");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let target = path("export-profile");
    let coarse = c.export_stl_selected_with_profile(&target, StlExportProfile { linear_tolerance_mm: 0.05, angular_tolerance_rad: 0.3 }).unwrap();
    assert_eq!(coarse.requested_tolerance_mm, 0.05);
    let fine = c.export_stl_selected_with_profile(&target, StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
    assert_eq!(fine.requested_tolerance_mm, 0.001);
    assert!(fine.triangles > coarse.triangles, "coarse {} fine {}", coarse.triangles, fine.triangles);
    assert_eq!(*c.document(), before);
    let bytes = std::fs::read(&target).unwrap();
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(c.export_stl_selected_with_profile(&target, StlExportProfile { linear_tolerance_mm: invalid, angular_tolerance_rad: 0.3 }).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
    }
    std::fs::remove_file(target).unwrap();
}

#[test]
fn export_reports_scaled_occt_topology_tolerance_without_claiming_deviation() {
    let limits = Limits::default();
    let brep = om9_kernel::brep::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("scaled source tolerance");
    let mut entity = Entity::new("box", Geometry::Brep(brep), None);
    entity.placement = Transform([[20.,0.,0.,0.], [0.,1.,0.,0.], [0.,0.,1.,0.], [0.,0.,0.,1.]]);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("scaled-source-tolerance");
    let receipt = controller.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile {
        linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3,
    }).unwrap();
    let note = receipt.warnings.iter().find(|note| note.contains("maximum face/edge/vertex tolerance")).expect("source tolerance note");
    assert!(!note.contains("exceeds requested STL tolerance"), "{note}");
    let raw: f64 = note.split("tolerance ").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    let scaled: f64 = note.split("upper bound ").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    assert!(raw >= 1e-7 && (scaled / raw - 20.).abs() < 1e-12, "{note}");
    assert_eq!(receipt.max_occt_topology_tolerance_upper_mm, Some(scaled));
    assert!(note.contains("metadata is not measured mesh deviation"), "{note}");
    assert!(receipt.message().contains("global deviation unverified"));
    assert_eq!(triangles(&std::fs::read(&target).unwrap()).len(), receipt.triangles);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn export_reports_maximum_occt_tolerance_across_occurrences() {
    let limits = Limits::default();
    let brep = om9_kernel::brep::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
    let source_tolerance = om9_kernel::brep::read_brep_with_shell_groups_and_edges(brep.source(), 0.1, 0.3, &limits).unwrap().max_source_tolerance();
    let mut document = Document::with_template_tolerance("multiple tolerance occurrences");
    let mut ids = Vec::new();
    for (scale, x) in [(1., 0.), (20., 30.)] {
        let mut entity = Entity::new("box", Geometry::Brep(brep.clone()), None);
        entity.placement = Transform([[scale,0.,0.,x], [0.,1.,0.,0.], [0.,0.,1.,0.], [0.,0.,0.,1.]]);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    let target = path("maximum-source-tolerance");
    let receipt = controller.export_stl_selected(&target).unwrap();
    let maximum = receipt.max_occt_topology_tolerance_upper_mm.unwrap();
    assert!((maximum - source_tolerance * 20.).abs() < 1e-15);
    assert_eq!(receipt.warnings.iter().filter(|warning| warning.contains("maximum face/edge/vertex tolerance")).count(), 2);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn high_source_tolerance_is_reported_for_real_occt_brep() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(2., 0.1, &limits).unwrap();
    // The generated OCCT text stores each tolerance after its topology marker.
    // Change only that scalar, preserving the sphere's geometry.
    for (kind, old, new) in [
        ("vertex", "Ve\n1e-07\n", "Ve\n0.01\n"),
        ("edge", "Ed\n 1e-07 ", "Ed\n 0.01 "),
        ("face", "Fa\n0  1e-07 ", "Fa\n0  0.01 "),
    ] {
        let source = sphere.source().replacen(old, new, 1);
        assert_ne!(source, sphere.source(), "fixture must change a {kind} tolerance");
        let source_brep = om9_kernel::brep::read_brep(&source, 0.1, &limits).unwrap();
        let snapshot = om9_kernel::brep::read_brep_with_shell_groups_and_edges(source_brep.source(), 0.1, 0.3, &limits).unwrap();
        assert!((snapshot.max_source_tolerance() - 0.01).abs() < 1e-12);
        let mut document = Document::with_template_tolerance("high source tolerance");
        let entity = Entity::new("sphere", Geometry::Brep(source_brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = path(&format!("high-source-tolerance-{kind}"));
        let receipt = controller.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile {
            linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3,
        }).unwrap();
        let note = receipt.warnings.iter().find(|note| note.contains("maximum face/edge/vertex tolerance")).unwrap();
        assert!(note.contains("exceeds requested STL tolerance"), "{note}");
        assert!(note.contains("metadata is not measured mesh deviation"), "{note}");
        assert!(receipt.message().contains("global deviation unverified"));
        let encoded = triangles(&std::fs::read(&target).unwrap());
        assert_eq!(encoded.len(), receipt.triangles);
        for vertex in encoded.iter().flatten() {
            let radius = vertex.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>().sqrt();
            assert!((radius - 2.).abs() < 1e-6, "{kind}: encoded vertex radius {radius}");
        }
        std::fs::remove_file(target).unwrap();
    }
}

#[test]
fn angular_export_profile_controls_canonical_and_retained_sphere() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5.0, 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    for (name, geometry) in [("canonical", Geometry::Brep(sphere)), ("retained", Geometry::NativeBrep(native))] {
        let mut document = Document::with_template_tolerance(name);
        let entity = Entity::new(name, geometry, None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = path(&format!("angle-{name}"));
        let profile = |angle| om9_app::session::StlExportProfile { linear_tolerance_mm: 0.5, angular_tolerance_rad: angle };
        let coarse = controller.export_stl_selected_with_profile(&target, profile(0.6)).unwrap();
        let fine = controller.export_stl_selected_with_profile(&target, profile(0.1)).unwrap();
        assert_eq!(fine.requested_angle_rad, 0.1);
        assert!(fine.triangles > coarse.triangles, "{name}: coarse {} fine {}", coarse.triangles, fine.triangles);
        let before = std::fs::read(&target).unwrap();
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, 4.0] {
            assert!(controller.export_stl_selected_with_profile(&target, profile(invalid)).is_err());
            assert_eq!(std::fs::read(&target).unwrap(), before);
        }
        std::fs::remove_file(target).unwrap();
    }
}

#[test]
fn native_brep_snapshot_roundtrip_preserves_sphere_source_geometry() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let original_payload = native.payload.clone();
    let mut document = Document::with_template_tolerance("native sphere snapshot");
    let entity = Entity::new("sphere", Geometry::NativeBrep(native), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let snapshot = om9_archive::DocumentSnapshot::capture(&document, &limits).unwrap();
    let restored = om9_archive::DocumentSnapshot::from_json(&snapshot.to_json().unwrap(), &limits).unwrap().restore(&limits).unwrap();
    let Geometry::NativeBrep(restored_native) = &restored.entities[&id].geometry else { panic!("native B-rep") };
    assert_eq!(&*restored_native.payload, &*original_payload);
    assert!(!std::sync::Arc::ptr_eq(&restored_native.payload, &original_payload));
    let reimported = om9_kernel::brep::from_3dm(&restored_native.payload, 0.001, 0.1, &limits).unwrap();
    let volume = om9_kernel::brep::source_volume(&reimported, &limits).unwrap();
    let analytic = 4. * std::f64::consts::PI * 5_f64.powi(3) / 3.;
    assert!((volume - analytic).abs() < 1e-5, "restored sphere volume: {volume}");
    let measures = om9_kernel::creation::measure(&reimported, &limits).unwrap();
    assert_eq!((measures.solids, measures.shells), (1, 1));
}

#[test]
fn retained_brep_export_is_independent_of_its_display_mesh() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
    let full = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    assert!(full.display.faces.len() > 1);
    let mut partial = full.clone();
    // An intentionally incomplete but valid display cache is a stronger
    // separation probe than comparing two similarly tessellated caches.
    partial.display.faces.truncate(1);
    partial.display.corner_uvs.clear();
    let mut outputs = Vec::new();
    for (name, native) in [("native-display-full", full), ("native-display-partial", partial)] {
        let mut document = Document::with_template_tolerance(name);
        let entity = Entity::new(name, Geometry::NativeBrep(native), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = path(name);
        let receipt = controller.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile {
            linear_tolerance_mm: 0.05, angular_tolerance_rad: 0.3,
        }).unwrap();
        assert!(receipt.watertight(), "{name}: {receipt:?}");
        assert!(receipt.triangles > 1);
        outputs.push(std::fs::read(&target).unwrap());
        std::fs::remove_file(target).unwrap();
    }
    assert_eq!(outputs[0], outputs[1], "the retained 3DM payload, not its display cache, owns STL geometry");
}

#[test]
fn brep_export_is_independent_of_display_deflection() {
    let limits = Limits::default();
    let coarse = om9_kernel::brep::sphere_brep(5.0, 0.1, &limits).unwrap();
    let finer = om9_kernel::brep::read_brep(coarse.source(), 0.02, &limits).unwrap();
    assert_ne!(coarse.display.deflection_mm, finer.display.deflection_mm);
    let mut outputs = Vec::new();
    for (name, brep) in [("display-coarse", coarse), ("display-finer", finer)] {
        let mut document = Document::with_template_tolerance(name);
        let entity = Entity::new(name, Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = path(name);
        controller.export_stl_selected(&target).unwrap();
        outputs.push(std::fs::read(&target).unwrap());
        std::fs::remove_file(target).unwrap();
    }
    assert_eq!(outputs[0], outputs[1], "display meshing must not change exported STL");
}

#[test]
fn nonuniformly_scaled_sphere_export_sampled_triangle_centers_meet_document_tolerance() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5.0, 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("scaled sphere");
    let mut entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    entity.placement = Transform::scale([3.0, 1.0, 0.5]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("scaled-sphere-centers");
    controller.export_stl_selected(&target).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    let mut maximum = 0.0_f64;
    for triangle in &faces {
        let center: [f64; 3] = std::array::from_fn(|axis| triangle.iter().map(|p| f64::from(p[axis])).sum::<f64>() / 3.0);
        let local = [center[0] / 3.0, center[1], center[2] / 0.5];
        let length = local.iter().map(|v| v * v).sum::<f64>().sqrt();
        let source = [local[0] * 15.0 / length, local[1] * 5.0 / length, local[2] * 2.5 / length];
        let deviation = (0..3).map(|axis| (source[axis] - center[axis]).powi(2)).sum::<f64>().sqrt();
        maximum = maximum.max(deviation);
    }
    assert!(maximum <= 0.001, "sampled maximum deviation: {maximum} mm");
    // Independent source-to-output radial probes on the ellipsoid, including
    // axes, near-pole and near-seam directions. This is sampled coverage only.
    let mut source_max = 0.0_f64;
    let mut probes = 0usize;
    for z in [-1.0_f64, -0.999, 0., 0.999, 1.] {
        let xy = (1. - z*z).sqrt();
        for angle in [-1e-8_f64, 0., 1e-8, std::f64::consts::FRAC_PI_2, std::f64::consts::PI] {
            let source = [15. * xy * angle.cos(), 5. * xy * angle.sin(), 2.5 * z];
            let radius = source[0].hypot(source[1]).hypot(source[2]);
            let direction = source.map(|coordinate| coordinate / radius);
            let encoded_radius = faces.iter()
                .filter_map(|triangle| ray_triangle_distance(direction, triangle.map(|p| p.map(f64::from))))
                .fold(f64::INFINITY, f64::min);
            assert!(encoded_radius.is_finite(), "scaled-sphere ray missed: z={z} angle={angle}");
            source_max = source_max.max((radius - encoded_radius).abs());
            probes += 1;
        }
    }
    assert!(source_max <= 0.001, "source-to-STL sampled radial distance: {source_max} mm");
    println!("scaled_sphere_stl samples={probes} max_triangle_center={maximum:.9} max_source_radial={source_max:.9} mm");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn large_coordinate_three_axis_quantization_uses_euclidean_budget_atomically() {
    use om9_app::session::StlExportProfile;
    let source = [[100000.003, 100000.003, 100000.003],
        [100001.003, 100000.003, 100000.003], [100000.003, 100001.003, 100000.003]];
    let mesh = Mesh { vertices: source.to_vec(), faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![] };
    let mut document = Document::with_template_tolerance("three-axis STL quantization");
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    let target = path("three-axis-quantization");
    std::fs::write(&target, b"previous destination").unwrap();
    let profile = |linear_tolerance_mm| StlExportProfile { linear_tolerance_mm, angular_tolerance_rad: 0.3 };
    assert!(controller.export_stl_selected_with_profile(&target, profile(0.04)).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"previous destination");
    let receipt = controller.export_stl_selected_with_profile(&target, profile(0.06)).unwrap();
    assert_eq!(receipt.triangles, 1);
    let encoded = triangles(&std::fs::read(&target).unwrap());
    for (actual, expected) in encoded[0].iter().zip(source) {
        let delta = std::array::from_fn::<_, 3, _>(|axis| (f64::from(actual[axis]) - expected[axis]).abs());
        assert!(delta.iter().all(|value| *value < 0.004));
        let error = delta[0].hypot(delta[1]).hypot(delta[2]);
        assert!(error > 0.004 && error < 0.006, "Euclidean quantization error {error}");
    }
    assert_eq!(*controller.document(), before);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn tiny_authored_triangle_exports_without_area_floor_and_preserves_document() {
    let mut document = Document::with_template_tolerance("tiny authored triangle");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1e-16, 0., 0.], [0., 1e-16, 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("tiny triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    let target = path("tiny-authored-triangle");
    let receipt = controller.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile {
        linear_tolerance_mm: 1e-20, angular_tolerance_rad: 0.3,
    }).unwrap();
    let bytes = std::fs::read(&target).unwrap();
    let faces = triangles(&bytes);
    assert_eq!(receipt.triangles, 1);
    assert_eq!(faces.len(), 1);
    assert!(faces[0][1][0] > 0. && faces[0][2][1] > 0.);
    assert!((f64::from(faces[0][1][0]) - 1e-16).abs() < 1e-21);
    assert_eq!(f32::from_le_bytes(bytes[92..96].try_into().unwrap()), 1.);
    assert_eq!(*controller.document(), before);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn mirrored_authored_mesh_preserves_world_winding_in_export() {
    let mut document = Document::with_template_tolerance("mirror stl");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    entity.placement = Transform::scale([-1., 1., 1.]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("mirrored-winding");
    c.export_stl_selected(&target).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    let [a, b, c] = faces[0];
    let normal_z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    assert!(normal_z > 0.0, "mirrored triangle winding: {faces:?}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn nested_instances_export_all_occurrences_with_composed_reflections() {
    use om9_core::model::Definition;
    let mut document = Document::with_template_tolerance("nested instances STL");
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
        faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let mut leaf = Entity::new("triangle", Geometry::Mesh(mesh), None);
    leaf.placement = Transform::translation([2., 0., 0.]);
    let inner = Definition::new("inner", vec![leaf]);
    let inner_id = inner.id;
    document.definitions.insert(inner_id, inner);
    let mut nested = Entity::new("nested", Geometry::Instance(inner_id), None);
    nested.placement = Transform::scale([-1., 1., 1.]).unwrap();
    let outer = Definition::new("outer", vec![nested]);
    let outer_id = outer.id;
    document.definitions.insert(outer_id, outer);
    let mut ids = Vec::new();
    for x in [10., 20.] {
        let mut instance = Entity::new("occurrence", Geometry::Instance(outer_id), None);
        instance.placement = Transform::translation([x, 0., 0.]);
        ids.push(instance.id);
        document.entities.insert(instance.id, instance);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    let target = path("nested-instances");
    let receipt = controller.export_stl_selected(&target).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    assert_eq!(receipt.triangles, 2);
    assert_eq!(faces.len(), 2);
    let mut minima: Vec<f32> = faces.iter().map(|face| face.iter().map(|point| point[0]).fold(f32::INFINITY, f32::min)).collect();
    minima.sort_by(f32::total_cmp);
    assert_eq!(minima, vec![7., 17.]);
    for [a, b, c] in faces {
        assert!((b[0]-a[0])*(c[1]-a[1]) - (b[1]-a[1])*(c[0]-a[0]) > 0.);
    }
    assert_eq!(controller.coordinator().undo_len(), 0);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn repeated_brep_instances_export_from_source_at_each_placement() {
    use om9_core::model::Definition;
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("B-rep instances STL");
    let sphere = om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap();
    let definition = Definition::new("sphere", vec![Entity::new("sphere", Geometry::Brep(sphere), None)]);
    let definition_id = definition.id;
    document.definitions.insert(definition_id, definition);
    let mut ids = Vec::new();
    for x in [-10., 10.] {
        let mut instance = Entity::new("occurrence", Geometry::Instance(definition_id), None);
        instance.placement = Transform::translation([x, 0., 0.]);
        ids.push(instance.id);
        document.entities.insert(instance.id, instance);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    let target = path("brep-instances");
    let profile = om9_app::session::StlExportProfile { linear_tolerance_mm: 0.05, angular_tolerance_rad: 0.3 };
    let receipt = controller.export_stl_selected_with_profile(&target, profile).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    assert!(receipt.watertight(), "{receipt:?}");
    assert!(!faces.is_empty());
    let mut sides = [0usize; 2];
    for face in faces {
        let center = face.iter().map(|point| point[0]).sum::<f32>() / 3.;
        assert!(center.abs() > 6. && center.abs() < 14.);
        sides[usize::from(center > 0.)] += 1;
    }
    assert!(sides[0] > 0 && sides[0] == sides[1], "{sides:?}");
    assert_eq!(controller.coordinator().undo_len(), 0);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn hidden_block_geometry_is_included_with_an_explicit_warning() {
    use om9_core::model::Definition;
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
        faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let visible = Entity::new("visible", Geometry::Mesh(mesh.clone()), None);
    let mut hidden = Entity::new("hidden", Geometry::Mesh(mesh), None);
    hidden.attributes.hidden = true;
    hidden.placement = Transform::translation([5., 0., 0.]);
    let definition = Definition::new("mixed block", vec![visible, hidden]);
    let definition_id = definition.id;
    let mut document = Document::with_template_tolerance("hidden block STL");
    document.definitions.insert(definition_id, definition);
    let root = Entity::new("block", Geometry::Instance(definition_id), None);
    let id = root.id;
    document.entities.insert(id, root);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("hidden-block");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 2);
    assert!(receipt.warnings.iter().any(|warning| warning.contains("hidden or invisible")));
    std::fs::remove_file(target).unwrap();
}

#[test]
fn instance_export_capture_keeps_placement_after_edit() {
    use om9_core::model::Definition;
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
        faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let definition = Definition::new("triangle", vec![Entity::new("mesh", Geometry::Mesh(mesh), None)]);
    let definition_id = definition.id;
    let mut document = Document::with_template_tolerance("captured block");
    document.definitions.insert(definition_id, definition);
    let instance = Entity::new("block", Geometry::Instance(definition_id), None);
    let id = instance.id;
    document.entities.insert(id, instance);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    controller.numeric_move([10., 0., 0.]).unwrap();
    let target = path("captured-instance");
    capture.export_with_profile(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    assert_eq!(faces.len(), 1);
    assert!(faces[0].iter().all(|point| point[0] < 2.));
    assert_eq!(controller.document().entities[&id].placement, Transform::translation([10., 0., 0.]));
    std::fs::remove_file(target).unwrap();
}

#[test]
fn concave_quad_export_uses_interior_diagonal() {
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0.5, 0.], [2., 0., 0.], [0., 2., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::with_template_tolerance("concave quad");
    let entity = Entity::new("concave", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let scene = controller.scene().unwrap();
    let object = scene.objects.iter().find(|object| object.root == id).unwrap();
    let om9_scene::scene::RenderBuffers::Mesh { vertices, triangles: display, .. } = object.buffers.as_ref() else { panic!("mesh display") };
    assert_eq!(display.iter().map(|(indices, _)| *indices).collect::<Vec<_>>(), vec![[0, 1, 3], [1, 2, 3]]);
    let displayed: Vec<_> = display.iter().map(|(indices, _)| indices.map(|index| vertices[index as usize].map(|v| v as f32))).collect();
    let camera = om9_scene::camera::Camera::orthographic([0., 0., 10.], [0.; 3], [0., 1., 0.], 6.).unwrap();
    let viewport = om9_scene::camera::Viewport { origin: [0.; 2], size: [400.; 2] };
    let pick = |point| scene.pick(camera.project(point, viewport).unwrap().unwrap().screen,
        &camera, viewport, 0., om9_scene::scene::PickIntent::Select).unwrap();
    assert!(pick([1., 0.1, 0.]).is_none(), "the concavity outside the polygon must not be pickable");
    assert_eq!(pick([0.5, 1., 0.]).unwrap().root, id);
    let screen = camera.project([0.5, 1., 0.], viewport).unwrap().unwrap().screen;
    let face = scene.pick_kind(screen, &camera, viewport, 0., om9_scene::scene::PickIntent::Select,
        om9_scene::scene::PickKind::Face).unwrap().unwrap();
    assert_eq!(face.subset, om9_scene::scene::Subelement::MeshFace(0));
    let target = path("concave-quad");
    controller.export_stl_selected(&target).unwrap();
    let faces = triangles(&std::fs::read(&target).unwrap());
    assert_eq!(faces.len(), 2);
    assert_eq!(faces, displayed, "picking/display triangles and exported triangles must agree");
    let mut area = 0.0;
    for [a, b, c] in faces {
        let signed = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(signed > 0.0, "triangle winding must follow the face");
        area += signed / 2.0;
    }
    assert_eq!(area, 1.5);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn crossed_quad_is_rejected_without_replacing_destination() {
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [2., 2., 0.], [0., 2., 0.], [2., 0., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::with_template_tolerance("crossed quad");
    let entity = Entity::new("crossed", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("crossed-quad");
    std::fs::write(&target, b"old STL").unwrap();
    assert!(controller.export_stl_selected(&target).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"old STL");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn touching_tetrahedra_report_disconnected_vertex_fans_after_encoding() {
    let mesh = Mesh {
        vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.], [-1.,0.,0.], [0.,-1.,0.], [0.,0.,-1.]],
        faces: vec![vec![0,2,1], vec![0,1,3], vec![1,2,3], vec![2,0,3],
            vec![0,5,4], vec![0,4,6], vec![4,5,6], vec![5,0,6]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::with_template_tolerance("disconnected vertex fans");
    let entity = Entity::new("touching tetrahedra", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("touching-tetrahedra-fan");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!((receipt.open_edges, receipt.non_manifold_edges, receipt.inconsistent_edges), (0, 0, 0));
    assert_eq!(receipt.disconnected_vertices, 1);
    assert_eq!(receipt.edge_connected_components, 2);
    assert!(!receipt.watertight());
    assert_eq!(receipt.encoded_closed_by_topology_checks(), Some(false));
    assert!(receipt.message().contains("1 disconnected vertices"));
    let encoded = triangles(&std::fs::read(&target).unwrap());
    assert_eq!(encoded.len(), 8);
    let key = |point: [f32; 3]| point.map(|v| if v == 0. { 0 } else { v.to_bits() });
    let mut vertices = std::collections::BTreeSet::new();
    let mut edges = std::collections::BTreeMap::<([u32; 3], [u32; 3]), usize>::new();
    for face in &encoded {
        let ids = face.map(key);
        vertices.extend(ids);
        for (a, b) in [(ids[0],ids[1]), (ids[1],ids[2]), (ids[2],ids[0])] {
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    assert_eq!(vertices.len(), 7);
    assert!(edges.values().all(|count| *count == 2));
    assert!(!receipt.encoded_vertex_identity_ambiguous);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn intersecting_closed_components_remain_reported_as_self_intersection_unchecked() {
    let tetra = Mesh {
        vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.], [0.,0.,2.]],
        faces: vec![vec![0,2,1], vec![0,1,3], vec![1,2,3], vec![2,0,3]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::with_template_tolerance("intersecting closed tetrahedra");
    let a = Entity::new("first", Geometry::Mesh(tetra.clone()), None);
    let mut b = Entity::new("second", Geometry::Mesh(tetra), None);
    b.placement = Transform::translation([0.5; 3]);
    let ids = [a.id, b.id];
    document.entities.insert(a.id, a);
    document.entities.insert(b.id, b);
    // The second tetrahedron has a vertex strictly inside x+y+z<2 of
    // the first and an adjacent vertex outside it, so their surfaces cross.
    let inside_sum = 0.5 + 0.5 + 0.5;
    let outside_sum = 2.5 + 0.5 + 0.5;
    assert!(inside_sum < 2. && outside_sum > 2.);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&ids).unwrap();
    let target = path("intersecting-closed-components");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 8);
    assert_eq!(receipt.edge_connected_components, 2);
    assert!(receipt.watertight(), "indexed topology is closed: {receipt:?}");
    assert_eq!(receipt.encoded_closed_by_topology_checks(), Some(true));
    assert!(receipt.message().contains("self-intersection unchecked"));
    assert_eq!(triangles(&std::fs::read(&target).unwrap()).len(), 8);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn coincident_open_objects_are_not_welded_into_a_false_closed_mesh() {
    let mut document = Document::with_template_tolerance("separate objects");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let one = Entity::new("one", Geometry::Mesh(mesh.clone()), None);
    let two = Entity::new("two", Geometry::Mesh(mesh), None);
    let ids = [one.id, two.id];
    document.entities.insert(one.id, one);
    document.entities.insert(two.id, two);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let target = path("separate-coincident-objects");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 2);
    assert_eq!(receipt.open_edges, 6);
    assert_eq!(receipt.edge_connected_components, 2);
    assert_eq!(receipt.max_occt_topology_tolerance_upper_mm, None);
    assert!(receipt.encoded_vertex_identity_ambiguous);
    assert_eq!(receipt.encoded_closed_by_topology_checks(), None);
    assert!(!receipt.watertight());
    let message = receipt.message();
    assert!(message.contains("self-intersection unchecked"), "{message}");
    assert!(message.contains("global deviation unverified"), "{message}");
    assert!(message.contains("source-scoped indexed topology"), "{message}");
    assert!(message.contains("2 source-scoped edge-connected triangle components"), "{message}");
    assert!(message.contains("STL cannot preserve their topology identity"), "{message}");
    assert!(message.contains("STL has no unit metadata"), "{message}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn encoded_topology_status_is_unknown_when_closed_sources_coincide() {
    use std::collections::HashMap;
    let tetra = Mesh {
        vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.]],
        faces: vec![vec![0,2,1], vec![0,1,3], vec![1,2,3], vec![2,0,3]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    for count in [1, 2] {
        let mut document = Document::with_template_tolerance("encoded topology");
        let mut ids = Vec::new();
        for _ in 0..count {
            let entity = Entity::new("tetra", Geometry::Mesh(tetra.clone()), None);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&ids).unwrap();
        let target = path(&format!("encoded-topology-{count}"));
        let receipt = controller.export_stl_selected(&target).unwrap();
        let encoded = triangles(&std::fs::read(&target).unwrap());
        let mut edges = HashMap::<([u32; 3], [u32; 3]), usize>::new();
        for triangle in encoded {
            let vertices = triangle.map(|point| point.map(|value| if value == 0.0 { 0 } else { value.to_bits() }));
            for (a, b) in [(vertices[0], vertices[1]), (vertices[1], vertices[2]), (vertices[2], vertices[0])] {
                *edges.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
            }
        }
        assert!(receipt.watertight(), "each source is indexed-closed");
        if count == 1 {
            assert_eq!(receipt.encoded_closed_by_topology_checks(), Some(true));
            assert!(edges.values().all(|incidence| *incidence == 2));
        } else {
            assert_eq!(receipt.encoded_closed_by_topology_checks(), None);
            assert!(edges.values().all(|incidence| *incidence == 4), "coordinate welding makes the encoded edge incidence non-manifold");
        }
        std::fs::remove_file(target).unwrap();
    }
}

#[test]
fn encoded_topology_reports_unambiguous_open_triangle() {
    let mut document = Document::with_template_tolerance("open encoded triangle");
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("open", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("encoded-open-triangle");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.encoded_closed_by_topology_checks(), Some(false));
    assert_eq!(receipt.open_edges, 3);
    assert_eq!(triangles(&std::fs::read(&target).unwrap()).len(), 1);
    std::fs::remove_file(target).unwrap();
}

fn nearby_solids_gap_fixture(case: &str) -> (Option<bool>, Vec<String>) {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let a = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap();
    let b = transform_ops::transform_brep(&a, &Transform::translation([1.0000005, 0., 0.]), 0.1, &limits).unwrap();
    let compound = editing::boolean(&a, &b, BooleanOp::Union, 0.1, &limits).unwrap().body().unwrap().clone();
    assert_eq!(creation::measure(&compound, &limits).unwrap().solids, 2, "the source must contain separate solids");
    let mut document = Document::with_template_tolerance("nearby solids");
    let entity = Entity::new("compound", Geometry::Brep(compound), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path(case);
    std::fs::write(&target, b"existing destination").unwrap();
    let result = c.export_stl_selected(&target);
    if let Err(error) = result {
        assert!(error.to_string().contains("unverified weld"), "{error}");
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        std::fs::remove_file(target).unwrap();
        return (None, Vec::new());
    }
    let receipt = result.unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    let encoded_second_boundary = 1.0000005_f64 as f32;
    let first_boundary_survives = output.iter().flatten().any(|point| point[0] == 1.0_f32);
    let second_boundary_survives = first_boundary_survives && output.iter().flatten().any(|point| point[0] == encoded_second_boundary);
    std::fs::remove_file(target).unwrap();
    (Some(second_boundary_survives), receipt.warnings)
}

#[test]
fn nearby_solids_in_one_brep_report_if_export_loses_the_gap() {
    let (second_boundary_survives, warnings) = nearby_solids_gap_fixture("nearby-solids-report");
    assert_ne!(second_boundary_survives, Some(false), "unsafe export must be refused or preserve the gap: {warnings:?}");
}

#[test]
fn nearby_solids_in_one_brep_preserve_their_gap() {
    let (second_boundary_survives, _) = nearby_solids_gap_fixture("nearby-solids-preserve");
    assert_eq!(second_boundary_survives, Some(true), "two distinct solids must export and retain their gap in STL");
}

#[test]
fn separated_box_solids_without_ambiguous_welds_still_export() {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let a = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap();
    let b = transform_ops::transform_brep(&a, &Transform::translation([5., 0., 0.]), 0.1, &limits).unwrap();
    let compound = editing::boolean(&a, &b, BooleanOp::Union, 0.1, &limits).unwrap().body().unwrap().clone();
    assert_eq!(creation::measure(&compound, &limits).unwrap().solids, 2);
    let mut document = Document::with_template_tolerance("separated solids");
    let entity = Entity::new("compound", Geometry::Brep(compound), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("separated-solids");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 24);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn oriented_volume_oracle_changes_sign_when_triangle_order_reverses() {
    let a = [0., 0., 0.]; let b = [1., 0., 0.]; let c = [0., 1., 0.]; let d = [0., 0., 1.];
    let tetrahedron = vec![[a,c,b], [a,b,d], [a,d,c], [b,c,d]];
    let reversed: Vec<_> = tetrahedron.iter().map(|[a,b,c]| [*a,*c,*b]).collect();
    assert!((oriented_volume(&tetrahedron) - 1. / 6.).abs() < 1e-12);
    assert!((oriented_volume(&reversed) + 1. / 6.).abs() < 1e-12);
}

#[test]
fn mirrored_nonuniform_brep_cavity_preserves_each_encoded_shell_orientation() {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let outer = creation::primitive(Primitive::Box { x_mm: 4., y_mm: 4., z_mm: 4. }, 0.1, &limits).unwrap();
    let inner = creation::primitive(Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.1, &limits).unwrap();
    let inner = transform_ops::transform_brep(&inner, &Transform::translation([1.; 3]), 0.1, &limits).unwrap();
    let body = editing::boolean(&outer, &inner, BooleanOp::Difference, 0.1, &limits).unwrap().body().unwrap().clone();
    assert_eq!(creation::measure(&body, &limits).unwrap().shells, 2);
    let mut document = Document::with_template_tolerance("mirrored cavity");
    let mut entity = Entity::new("cavity", Geometry::Brep(body), None);
    entity.placement = Transform::scale([-2., 1.5, 0.5]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("mirrored-nonuniform-cavity");
    let receipt = controller.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    assert!(receipt.watertight(), "{receipt:?}");
    let (inner, outer): (Vec<_>, Vec<_>) = output.into_iter().partition(|triangle| triangle.iter().all(|p| {
        (-6. ..= -2.).contains(&p[0]) && (1.5..=4.5).contains(&p[1]) && (0.5..=1.5).contains(&p[2])
    }));
    assert_eq!((outer.len(), inner.len()), (12, 12));
    let outer_volume = oriented_volume(&outer);
    let inner_volume = oriented_volume(&inner);
    assert!((outer_volume - 96.).abs() < 1e-5, "outer shell volume {outer_volume}");
    assert!((inner_volume + 12.).abs() < 1e-5, "cavity shell volume {inner_volume}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn thin_cavity_in_one_solid_preserves_its_distinct_boundary() {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let outer = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap();
    let inner = creation::primitive(Primitive::Box { x_mm: 0.8, y_mm: 0.8, z_mm: 0.8 }, 0.1, &limits).unwrap();
    let gap = 0.0000005;
    let inner = transform_ops::transform_brep(&inner, &Transform::translation([gap; 3]), 0.1, &limits).unwrap();
    let body = editing::boolean(&outer, &inner, BooleanOp::Difference, 0.1, &limits).unwrap().body().unwrap().clone();
    let measures = creation::measure(&body, &limits).unwrap();
    assert_eq!((measures.solids, measures.shells), (1, 2), "fixture must have a cavity");
    let mut document = Document::with_template_tolerance("thin cavity");
    let entity = Entity::new("cavity", Geometry::Brep(body), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("thin-cavity");
    let receipt = c.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    let inner_corner = gap as f32;
    assert!(output.iter().flatten().any(|point| point.iter().all(|x| *x == inner_corner)),
        "the cavity corner must survive in STL; warnings: {:?}", receipt.warnings);
    assert!(receipt.watertight(), "indexed topology of both shells must be closed: {receipt:?}");
    let volume = oriented_volume(&output);
    assert!((volume - (1.0 - 0.8_f64.powi(3))).abs() < 1e-6,
        "inner shell must retain cavity winding: volume={volume}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn narrow_slot_in_one_shell_keeps_both_topological_edges() {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let outer = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap();
    let gap = 0.1;
    let x_scale = 0.000005;
    let cutter = creation::primitive(Primitive::Box { x_mm: gap, y_mm: 0.5, z_mm: 1.2 }, 0.1, &limits).unwrap();
    let cutter = transform_ops::transform_brep(&cutter, &Transform::translation([0.5, 0.25, -0.1]), 0.1, &limits).unwrap();
    let body = editing::boolean(&outer, &cutter, BooleanOp::Difference, 0.1, &limits).unwrap().body().unwrap().clone();
    let measures = creation::measure(&body, &limits).unwrap();
    assert_eq!((measures.solids, measures.shells), (1, 1), "slot fixture must be a single shell");
    let mut document = Document::with_template_tolerance("single shell narrow slot");
    let mut entity = Entity::new("slotted block", Geometry::Brep(body), None);
    entity.placement = Transform::scale([x_scale, 1., 1.]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("single-shell-slot");
    let receipt = c.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    let left = (0.5 * x_scale) as f32;
    let right = ((0.5 + gap) * x_scale) as f32;
    assert_ne!(left, right, "fixture gap must be representable in STL");
    assert!(output.iter().flatten().any(|p| p[0] == left && p[1] == 0.25 && p[2].abs() < 1e-10));
    assert!(output.iter().flatten().any(|p| p[0] == right && p[1] == 0.25 && p[2].abs() < 1e-10),
        "both slot walls must survive; receipt: {receipt:?}");
    assert!(receipt.watertight(), "fixture should preserve closed indexed topology: {receipt:?}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn retained_native_cavity_preserves_its_distinct_boundary() {
    use om9_kernel::{creation::{self, Primitive}, editing::{self, BooleanOp}, transform_ops};
    let limits = Limits::default();
    let outer = creation::primitive(Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap();
    let inner = creation::primitive(Primitive::Box { x_mm: 0.8, y_mm: 0.8, z_mm: 0.8 }, 0.1, &limits).unwrap();
    let gap = 0.0000005;
    let inner = transform_ops::transform_brep(&inner, &Transform::translation([gap; 3]), 0.1, &limits).unwrap();
    let body = editing::boolean(&outer, &inner, BooleanOp::Difference, 0.1, &limits).unwrap().body().unwrap().clone();
    let native = om9_kernel::brep::to_3dm(&body, 1e-8, &limits).unwrap();
    let imported = om9_kernel::brep::from_3dm(&native.payload, 1e-8, 0.1, &limits).unwrap();
    let source = om9_kernel::brep::from_3dm_source(&native.payload, 1e-8, &limits).unwrap();
    let (grouped, owners) = om9_kernel::brep::read_brep_with_shell_groups(&source, 0.1, 0.3, &limits).unwrap();
    assert_eq!(owners.iter().copied().collect::<std::collections::BTreeSet<_>>().len(), 2);
    assert_eq!(creation::measure(&grouped, &limits).unwrap().shells, 2);
    assert!(om9_kernel::brep::from_3dm_source(b"not a 3dm payload", 1e-8, &limits).is_err());
    let measures = creation::measure(&imported, &limits).unwrap();
    assert_eq!((measures.solids, measures.shells), (1, 2), "retained source must retain both shells");
    assert!(imported.display.points.iter().any(|point| point.iter().all(|x| (*x - gap).abs() < 1e-10)),
        "converted retained source must retain the cavity corner before STL export");
    let mut document = Document::with_template_tolerance("retained thin cavity");
    document.tolerance.as_mut().unwrap().absolute_mm = 1e-8;
    let entity = Entity::new("native cavity", Geometry::NativeBrep(native), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("native-thin-cavity");
    let receipt = c.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile {
        linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3,
    }).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    assert!(output.iter().flatten().any(|point| point.iter().all(|x| *x == gap as f32)),
        "retained cavity corner must survive in STL; warnings: {:?}", receipt.warnings);
    assert!(receipt.watertight(), "indexed shells must be closed: {receipt:?}");
    assert!((oriented_volume(&output) - (1.0 - 0.8_f64.powi(3))).abs() < 1e-6);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn authored_small_triangle_is_not_erased_by_welding() {
    let mut document = Document::with_template_tolerance("small authored triangle");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [5e-7, 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("small", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("small-authored-triangle");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 1);
    let actual = triangles(&std::fs::read(&target).unwrap());
    assert_eq!(actual.len(), 1);
    assert!(actual[0][1][0] > 0.0);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn duplicate_opposite_faces_are_reported_by_export_receipt() {
    let mut document = Document::with_template_tolerance("duplicate faces");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2], vec![0, 2, 1]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("duplicate", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("duplicate-opposite-faces");
    let receipt = controller.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.open_edges, 0);
    assert_eq!(receipt.duplicate_faces, 1);
    assert!(!receipt.watertight());
    assert!(receipt.message().contains("1 duplicate faces"));
    std::fs::remove_file(target).unwrap();
}

#[test]
fn quantization_failure_preserves_existing_destination() {
    let mut document = Document::with_template_tolerance("precision");
    let mesh = Mesh {
        vertices: vec![[100000.003, 0., 0.], [100001.003, 0., 0.], [100000.003, 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("far triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let directory = std::env::temp_dir().join(format!("om9-stl-failure-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let target = directory.join("target.stl");
    std::fs::write(&target, b"previous destination").unwrap();
    assert!(controller.export_stl_selected(&target).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"previous destination");
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1, "partial temporary file must be removed");
    std::fs::remove_file(target).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

fn path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("om9-stl-{}-{name}.stl", std::process::id()))
}

fn only(controller: &Controller) -> om9_core::identity::ObjectId {
    *controller.document().entities.keys().next().unwrap()
}

/// Count and geometric bounds of a binary STL, read back the way a slicer reads it.
fn triangles(bytes: &[u8]) -> Vec<[[f32; 3]; 3]> {
    assert!(bytes.len() >= 84);
    let count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 84 + count * 50);
    (0..count)
        .map(|index| {
            let mut corners = [[0.0f32; 3]; 3];
            let base = 84 + index * 50;
            for (slot, corner) in corners.iter_mut().enumerate() {
                let at = base + 12 + slot * 12;
                for axis in 0..3 {
                    corner[axis] = f32::from_le_bytes(
                        bytes[at + axis * 4..at + axis * 4 + 4].try_into().unwrap(),
                    );
                }
            }
            corners
        })
        .collect()
}

fn bounds(triangles: &[[[f32; 3]; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for triangle in triangles {
        for corner in triangle {
            for axis in 0..3 {
                lo[axis] = lo[axis].min(f64::from(corner[axis]));
                hi[axis] = hi[axis].max(f64::from(corner[axis]));
            }
        }
    }
    (lo, hi)
}

fn signed_volume(triangles: &[[[f32; 3]; 3]]) -> f64 {
    oriented_volume(triangles).abs()
}

fn oriented_volume(triangles: &[[[f32; 3]; 3]]) -> f64 {
    let mut volume = 0.0;
    for [a, b, c] in triangles {
        let (a, b, c) = (
            [f64::from(a[0]), f64::from(a[1]), f64::from(a[2])],
            [f64::from(b[0]), f64::from(b[1]), f64::from(b[2])],
            [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])],
        );
        volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]))
            / 6.0;
    }
    volume
}

#[test]
fn menu_and_cmd_reach_the_same_stl_export() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = only(&c);
    c.select_objects(&[id]).unwrap();
    let menu = c
        .dispatch_effect("OM9-FILE-001", InvocationOrigin::Menu)
        .unwrap();
    assert!(matches!(menu, DispatchOutcome::StlExport));
    let typed = c.submit_effect("ExportSTL").unwrap();
    assert!(matches!(typed, DispatchOutcome::StlExport));

    let target = path("routes");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.path, std::path::absolute(&target).unwrap());
    let bytes = std::fs::read(&target).unwrap();
    let triangles = triangles(&bytes);
    assert_eq!(triangles.len(), receipt.triangles);
    assert_eq!(bytes.len(), receipt.bytes);
    let (lo, hi) = bounds(&triangles);
    assert!(
        (lo[0]).abs() < 1e-6 && (hi[0] - 10.0).abs() < 1e-6,
        "{lo:?} {hi:?}"
    );
    assert!(
        (lo[2]).abs() < 1e-6 && (hi[2] - 10.0).abs() < 1e-6,
        "{lo:?} {hi:?}"
    );
    assert!((signed_volume(&triangles) - 1000.0).abs() < 1e-3);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn empty_selection_is_an_explicit_error_and_writes_nothing() {
    let c = controller();
    let target = path("empty");
    assert!(c.export_stl_selected(&target).is_err());
    assert!(!target.exists());
    assert!(matches!(Controller::availability("OM9-FILE-001"), Ok(())));
}

#[test]
fn mixed_selection_exports_the_solid_and_names_the_skipped_curve() {
    let mut c = controller();
    c.submit("Line").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Box 10 10 10").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    let target = path("mixed");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.skipped.len(), 1);
    assert!(receipt.skipped[0].contains("Line"), "{:?}", receipt.skipped);
    let triangles = triangles(&std::fs::read(&target).unwrap());
    let (_, hi) = bounds(&triangles);
    assert_eq!(hi, [10.0, 10.0, 10.0]);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn a_sphere_exports_as_one_watertight_millimetre_mesh() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let id = only(&c);
    c.select_objects(&[id]).unwrap();
    let target = path("sphere");
    let receipt = c.export_stl_selected(&target).unwrap();
    if receipt.warnings.iter().any(|warning| warning.contains("lack verified shared topology")) {
        assert!(receipt.message().contains("distinct features may have been joined"));
    }
    assert!(receipt.triangles > 100, "{}", receipt.triangles);
    let triangles = triangles(&std::fs::read(&target).unwrap());
    let (lo, hi) = bounds(&triangles);
    assert!(
        lo[0] < -4.9 && hi[0] > 4.9 && lo[2] < -4.9 && hi[2] > 4.9,
        "{lo:?} {hi:?}"
    );
    let volume = signed_volume(&triangles);
    let expected = 4.0 / 3.0 * std::f64::consts::PI * 125.0;
    assert!(
        (volume / expected - 1.0).abs() < 0.01,
        "{volume} vs {expected}"
    );
    std::fs::remove_file(target).unwrap();
}

#[test]
fn sphere_12mm_export_checks_all_triangles_and_sampled_surface_points() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(12., 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("sphere accuracy");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("sphere-12-analytic-accuracy");
    let receipt = controller.export_stl_selected(&target).unwrap();
    let mesh = triangles(&std::fs::read(&target).unwrap());
    assert!(receipt.watertight(), "{}", receipt.message());
    let mut triangle_max = 0.0_f64;
    for triangle in &mesh {
        let points = triangle.map(|p| p.map(f64::from));
        let minimum_radius = point_to_stl_triangle_distance([0.; 3], points);
        let maximum_radius = points.iter().map(|p| p.iter().map(|v| v*v).sum::<f64>().sqrt()).fold(0.0_f64, f64::max);
        triangle_max = triangle_max.max((12. - minimum_radius).abs()).max((12. - maximum_radius).abs());
    }
    assert!(triangle_max <= receipt.requested_tolerance_mm, "all-triangle radial bound {triangle_max}");
    let mut sampled_max = 0.0_f64;
    for i in 0..48 {
        let z = 1. - 2. * (i as f64 + 0.5) / 48.;
        let angle = i as f64 * std::f64::consts::PI * (3. - 5_f64.sqrt());
        let planar = (1. - z*z).sqrt();
        let direction = [planar * angle.cos(), planar * angle.sin(), z];
        let distance = mesh.iter().filter_map(|triangle| ray_triangle_distance(direction, triangle.map(|p| p.map(f64::from)))).fold(f64::INFINITY, f64::min);
        assert!(distance.is_finite(), "surface sample ray {i} missed the mesh");
        sampled_max = sampled_max.max((12. - distance).abs());
    }
    // Explicitly probe both poles and the periodic seam in addition to the
    // quasi-uniform samples above. Values close to the poles exercise narrow
    // incident triangles without claiming complete source coverage.
    let mut seam_pole_max = 0.0_f64;
    let mut seam_pole_samples = 0usize;
    for z in [-1.0_f64, -0.999999, -0.99, 0., 0.99, 0.999999, 1.] {
        let planar = (1. - z*z).sqrt();
        for angle in [-1e-8_f64, 0., 1e-8, std::f64::consts::PI, std::f64::consts::TAU - 1e-8] {
            let direction = [planar * angle.cos(), planar * angle.sin(), z];
            let distance = mesh.iter().filter_map(|triangle| ray_triangle_distance(direction, triangle.map(|p| p.map(f64::from))))
                .fold(f64::INFINITY, f64::min);
            assert!(distance.is_finite(), "pole/seam ray z={z}, angle={angle} missed the mesh");
            seam_pole_max = seam_pole_max.max((12. - distance).abs());
            seam_pole_samples += 1;
        }
    }
    assert!(sampled_max <= receipt.requested_tolerance_mm, "sampled radial deviation {sampled_max}");
    assert!(seam_pole_max <= receipt.requested_tolerance_mm, "pole/seam sampled radial deviation {seam_pole_max}");
    println!("sphere_stl_accuracy triangles={} max_triangle_radial={triangle_max:.9} max_source_ray_sample={sampled_max:.9}", mesh.len());
    println!("sphere_stl_poles_seam samples={seam_pole_samples} max_source_ray_sample={seam_pole_max:.9} mm");
    std::fs::remove_file(target).unwrap();
}

fn ray_triangle_distance(direction: [f64; 3], triangle: [[f64; 3]; 3]) -> Option<f64> {
    let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i]*b[i]).sum::<f64>();
    let cross = |a: [f64; 3], b: [f64; 3]| [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
    let sub = |a: [f64; 3], b: [f64; 3]| std::array::from_fn::<_, 3, _>(|i| a[i]-b[i]);
    let [a,b,c] = triangle;
    let e1 = sub(b,a); let e2 = sub(c,a); let h = cross(direction,e2);
    let det = dot(e1,h);
    if det.abs() < 1e-18 { return None; }
    let s = a.map(|v| -v); let u = dot(s,h)/det;
    if !(-1e-12..=1. + 1e-12).contains(&u) { return None; }
    let q = cross(s,e1); let v = dot(direction,q)/det;
    if v < -1e-12 || u+v > 1. + 1e-12 { return None; }
    let t = dot(e2,q)/det;
    (t >= 0.).then_some(t)
}

#[test]
fn only_selected_objects_are_exported() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let small = only(&c);
    c.submit("Box 20 20 20").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    assert_eq!(ids.len(), 2);
    c.select_objects(&[small]).unwrap();
    let target = path("selected");
    c.export_stl_selected(&target).unwrap();
    let (_, hi) = bounds(&triangles(&std::fs::read(&target).unwrap()));
    assert_eq!(hi, [10.0, 10.0, 10.0]);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn captured_stl_source_survives_later_document_and_selection_changes() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<om9_app::session::StlExportCapture>();
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let first = only(&controller);
    controller.select_objects(&[first]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    let source_epoch = capture.source_epoch();
    let source_document = capture.document_id();
    controller.submit("Box 20 20 20").unwrap();
    let second = *controller.document().entities.keys().find(|id| **id != first).unwrap();
    controller.select_objects(&[second]).unwrap();
    controller.delete_objects(&[first]).unwrap();
    assert!(!controller.document().entities.contains_key(&first));
    assert_ne!(controller.coordinator().epoch(), source_epoch);
    let target = path("captured-source");
    let receipt = capture.export_with_profile(&target, om9_app::session::StlExportProfile {
        linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3,
    }).unwrap();
    assert_eq!(receipt.source_epoch, source_epoch);
    assert_eq!(receipt.source_document_id, source_document);
    let (_, hi) = bounds(&triangles(&std::fs::read(&target).unwrap()));
    assert_eq!(hi, [10.0, 10.0, 10.0]);
    assert!(controller.selected().contains(&second));
    std::fs::remove_file(target).unwrap();
}

#[test]
fn cancelled_capture_does_not_touch_destination() {
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    let cancel = om9_app::session::StlCancellation::default();
    let target = path("cancelled-capture");
    std::fs::write(&target, b"old").unwrap();
    assert!(cancel.cancel());
    assert!(matches!(capture.export_with_cancellation(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, &cancel), Err(om9_core::Error::Canceled)));
    assert_eq!(std::fs::read(&target).unwrap(), b"old");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn old_capture_remains_accounted_after_new_document_replacement() {
    use om9_app::file_lifecycle::{FileKind, UnsavedDecision};
    use om9_core::budget::memory::MemoryClass;
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    let selection_bytes = controller.coordinator().external_reserved_bytes(MemoryClass::Export);
    assert!(selection_bytes > 0);
    let permit = controller.coordinator().external_memory_account().try_acquire_export().unwrap();
    controller.begin_file_operation(FileKind::New).unwrap();
    let token = controller.file_operation().unwrap().token();
    controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), selection_bytes);
    assert!(controller.coordinator().external_memory_account().try_acquire_export().is_err());
    drop(permit);
    assert!(controller.coordinator().external_memory_account().try_acquire_export().is_ok());
    let with_capture = controller.coordinator().retained_usage().unwrap().bytes;
    drop(capture);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    let after_drop = controller.coordinator().retained_usage().unwrap().bytes;
    assert!(with_capture > after_drop, "held={with_capture}, after={after_drop}");
}

#[test]
fn old_capture_remains_accounted_after_open_document_replacement() {
    use om9_app::file_lifecycle::{FileKind, UnsavedDecision};
    use om9_core::budget::memory::MemoryClass;
    let path = std::env::temp_dir().join(format!("om9-open-capture-{}.om9", uuid::Uuid::new_v4()));
    let mut saved = controller();
    saved.begin_file_operation(FileKind::SaveAs).unwrap();
    let token = saved.file_operation().unwrap().token();
    saved.accept_file_path(token, &path).unwrap();
    let mut active = controller();
    active.submit("Box 10 10 10").unwrap();
    let id = only(&active);
    active.select_objects(&[id]).unwrap();
    let capture = active.capture_stl_selected().unwrap();
    let selection_bytes = active.coordinator().external_reserved_bytes(MemoryClass::Export);
    assert!(selection_bytes > 0);
    let permit = active.coordinator().external_memory_account().try_acquire_export().unwrap();
    active.begin_file_operation(FileKind::Open).unwrap();
    let token = active.file_operation().unwrap().token();
    active.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    let token = active.file_operation().unwrap().token();
    active.accept_file_path(token, &path).unwrap();
    assert_eq!(active.coordinator().external_reserved_bytes(MemoryClass::Export), selection_bytes);
    assert!(active.coordinator().external_memory_account().try_acquire_export().is_err());
    drop(permit);
    assert!(active.coordinator().external_memory_account().try_acquire_export().is_ok());
    let held = active.coordinator().retained_usage().unwrap().bytes;
    drop(capture);
    assert_eq!(active.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    let released = active.coordinator().retained_usage().unwrap().bytes;
    assert!(held > released, "held={held}, released={released}");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn background_job_exports_capture_and_releases_reservation() {
    use om9_core::budget::memory::MemoryClass;
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    let epoch = capture.source_epoch();
    let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, 64 * 1024 * 1024, Some((8 * 1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024))).unwrap();
    let target = path("background-capture");
    let mut job = om9_app::session::StlExportJob::start(capture, target.clone(), om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation).unwrap();
    controller.delete_objects(&[id]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let result = loop {
        if let Some(result) = job.try_result() { break result; }
        assert!(std::time::Instant::now() < deadline, "STL job did not finish");
        std::thread::yield_now();
    };
    let receipt = result.unwrap();
    assert_eq!(receipt.source_epoch, epoch);
    assert_eq!(receipt.triangles, 12);
    assert_eq!(bounds(&triangles(&std::fs::read(&target).unwrap())).1, [10., 10., 10.]);
    assert!(job.try_result().is_none());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    std::fs::remove_file(target).unwrap();
}

#[test]
fn background_job_refuses_a_second_active_export_without_touching_destination() {
    use om9_core::budget::memory::MemoryClass;
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let account = controller.coordinator().external_memory_account();
    let permit = account.try_acquire_export().unwrap();
    let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, 1024, None).unwrap();
    let target = path("busy-export-destination");
    std::fs::write(&target, b"existing destination").unwrap();
    let result = om9_app::session::StlExportJob::start(controller.capture_stl_selected().unwrap(), target.clone(),
        om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation);
    assert!(matches!(result, Err(om9_core::Error::Invalid("export already active"))));
    assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    drop(permit);
    assert!(account.try_acquire_export().is_ok());
    std::fs::remove_file(target).unwrap();
}

#[test]
fn background_job_rejects_missing_or_wrong_reservation() {
    use om9_core::budget::memory::MemoryClass;
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let target = std::env::temp_dir().join(format!("om9-bad-reservation-{}.stl", uuid::Uuid::new_v4()));
    assert!(controller.coordinator().reserve_external_memory(MemoryClass::Export, 0, None).is_err());
    let foreign = Controller::new(Document::with_template_tolerance("foreign"), Limits::default()).unwrap();
    for (owner, class) in [(controller.coordinator(), MemoryClass::Kernel), (foreign.coordinator(), MemoryClass::Export)] {
        let reservation = owner.reserve_external_memory(class, 1024, None).unwrap();
        let result = om9_app::session::StlExportJob::start(controller.capture_stl_selected().unwrap(), target.clone(), om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation);
        assert!(matches!(result, Err(om9_core::Error::Invalid("STL export reservation"))));
        assert_eq!(owner.external_reserved_bytes(class), 0);
    }
    assert!(!target.exists());
}

#[test]
fn background_job_rejects_reservation_without_physical_ram_sample() {
    use om9_core::budget::memory::MemoryClass;
    let mut controller = controller();
    controller.submit("Box 10 10 10").unwrap();
    let id = only(&controller);
    controller.select_objects(&[id]).unwrap();
    let target = path("unsampled-ram-destination");
    std::fs::write(&target, b"existing destination").unwrap();
    let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, 1024, None).unwrap();
    let result = om9_app::session::StlExportJob::start(controller.capture_stl_selected().unwrap(), target.clone(),
        om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation);
    assert!(matches!(result, Err(om9_core::Error::Invalid("STL export requires sampled physical RAM"))));
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
    assert!(controller.coordinator().external_memory_account().try_acquire_export().is_ok());
    std::fs::remove_file(target).unwrap();
}

#[test]
fn a_retained_native_brep_exports_its_canonical_payload() {
    let brep = om9_kernel::brep::box_brep([10.0, 10.0, 10.0], 0.01, &Limits::default()).unwrap();
    let native = om9_kernel::brep::to_3dm(&brep, 0.001, &Limits::default()).unwrap();
    assert_ne!(native.crc, 0);
    let mut document = Document::with_template_tolerance("native stl");
    let entity = Entity::new("native cube", Geometry::NativeBrep(native), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("native");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert_eq!(receipt.triangles, 12);
    assert!(receipt.watertight(), "{:?}", receipt);
    assert!(
        receipt
            .warnings
            .iter()
            .any(|w| w.contains("retained native payload")),
        "{:?}",
        receipt.warnings
    );
    let (lo, hi) = bounds(&triangles(&std::fs::read(&target).unwrap()));
    assert!(
        lo[0].abs() < 1e-6 && (hi[2] - 10.0).abs() < 1e-6,
        "{lo:?} {hi:?}"
    );
    std::fs::remove_file(target).unwrap();
}

#[test]
fn an_open_surface_exports_with_an_honest_open_edge_count() {
    let mut document = Document::with_template_tolerance("surface stl");
    let surface = om9_core::surface_kernel::plane(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("surface");
    let receipt = c.export_stl_selected(&target).unwrap();
    assert!(receipt.triangles >= 2);
    assert!(!receipt.watertight(), "a surface cannot be closed");
    assert!(receipt.open_edges > 0);
    assert!(
        receipt.warnings.iter().any(|w| w.contains("open surface")),
        "{:?}",
        receipt.warnings
    );
    let (lo, hi) = bounds(&triangles(&std::fs::read(&target).unwrap()));
    assert!(lo[0].abs() < 1e-6 && lo[2].abs() < 1e-6, "{lo:?}");
    assert!(
        (hi[0] - 10.0).abs() < 1e-6 && (hi[1] - 10.0).abs() < 1e-6,
        "{hi:?}"
    );
    std::fs::remove_file(target).unwrap();
}

#[test]
fn nurbs_fold_keeps_nearby_unrelated_sheets_after_nonuniform_scale() {
    use om9_core::geometry::NurbsSurface;
    let surface = NurbsSurface {
        degree: [1, 1], size: [4, 2],
        poles: vec![[0., 0., 0.], [0., 1., 0.], [0., 0., 1.], [0., 1., 1.],
                    [0.1, 0., 1.], [0.1, 1., 1.], [0.1, 0., 0.], [0.1, 1., 0.]],
        weights: vec![1.; 8], knots: [vec![0., 0., 0.3, 0.6, 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let mut document = Document::with_template_tolerance("NURBS narrow fold");
    let mut entity = Entity::new("fold", Geometry::Surface(surface), None);
    entity.placement = Transform::scale([0.000005, 1., 1.]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("nurbs-fold");
    let receipt = c.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    assert!(output.iter().flatten().any(|p| p[0] == 0. && p[1] == 0. && p[2] == 0.));
    assert!(output.iter().flatten().any(|p| p[0] == (0.1 * 0.000005) as f32 && p[1] == 0. && p[2] == 0.),
        "both NURBS sheets must survive: {receipt:?}");
    assert!(receipt.open_edges > 0, "the folded sheet is open");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn cylindrical_nurbs_stl_has_only_two_open_boundary_loops() {
    use std::collections::{BTreeMap, BTreeSet};
    for (case, scale) in [("identity", [1., 1., 1.]), ("nonuniform", [2., 0.5, 1.5]), ("mirror", [-1., 1., 1.])] {
    let surface = om9_core::surface_kernel::cylinder([0.; 3], [1., 0., 0.], [0., 1., 0.], 2., 5., &Limits::default()).unwrap();
    let mut document = Document::with_template_tolerance("cylindrical NURBS seam");
    let mut entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    entity.placement = Transform::scale(scale).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path(&format!("nurbs-cylinder-boundaries-{case}"));
    let receipt = c.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    for tri in &output {
        let [a, b, c] = tri.map(|p| p.map(f64::from));
        let u = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
        let v = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
        let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
        let center = [(a[0]+b[0]+c[0])/3., (a[1]+b[1]+c[1])/3.];
        assert!(n[0]*center[0]/scale[0].powi(2) + n[1]*center[1]/scale[1].powi(2) > 0., "outward winding for {case}");
    }
    let key = |p: [f32; 3]| p.map(|v| if v == 0. { 0 } else { v.to_bits() });
    let mut edges = BTreeMap::<([u32; 3], [u32; 3]), usize>::new();
    for tri in &output {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let a = key(a); let b = key(b);
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut adjacency = BTreeMap::<[u32; 3], Vec<[u32; 3]>>::new();
    let mut boundary = 0;
    for ((a, b), count) in edges {
        assert!(count <= 2, "encoded non-manifold edge");
        if count == 1 { boundary += 1; adjacency.entry(a).or_default().push(b); adjacency.entry(b).or_default().push(a); }
    }
    assert_eq!(receipt.open_edges, boundary, "source-index boundary and encoded geometric boundary differ");
    assert!(adjacency.values().all(|neighbors| neighbors.len() == 2), "boundary must contain closed loops");
    let mut seen = BTreeSet::new();
    let mut components = 0;
    for start in adjacency.keys() {
        if seen.contains(start) { continue; }
        components += 1;
        let mut pending = vec![*start];
        while let Some(point) = pending.pop() {
            if seen.insert(point) { pending.extend(adjacency[&point].iter().copied()); }
        }
    }
    assert_eq!(components, 2, "an uncapped cylinder should have exactly two open loops");
    std::fs::remove_file(target).unwrap();
    }
}

#[test]
fn periodic_nurbs_export_preserves_expected_encoded_boundaries() {
    use om9_core::geometry::NurbsSurface;
    use std::collections::{BTreeMap, BTreeSet};
    let rows = [[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let weights = [1.,0.7,1.2,0.9,1.,0.7];
    let knots = vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.];
    let circle = [[1.,0.],[0.,1.],[-1.,0.],[0.,-1.],[1.,0.],[0.,1.]];
    let mut maximum_deviation = 0.0_f64;
    for torus in [false, true] {
      for (scale_name, scale) in [("identity", [1.,1.,1.]), ("nonuniform", [2.,0.5,1.5]), ("mirror", [-1.,1.,1.])] {
       let mut unreversed_volume = None;
       for reversed in [false, true] {
        let mut poles = Vec::new();
        let mut surface_weights = Vec::new();
        if torus {
            for (i, [x, y]) in circle.iter().copied().enumerate() {
                for (j, [radial, z]) in circle.iter().copied().enumerate() {
                    poles.push([(3. + 0.5 * radial) * x, (3. + 0.5 * radial) * y, 0.5 * z]);
                    surface_weights.push(weights[i] * weights[j]);
                }
            }
        } else {
            for (i, [x, y]) in rows.iter().copied().enumerate() {
                for z in [0., 2.] { poles.push([x, y, z]); surface_weights.push(weights[i]); }
            }
        }
        let surface = NurbsSurface {
            degree: if torus { [2, 2] } else { [2, 1] }, size: if torus { [6, 6] } else { [6, 2] },
            poles, weights: surface_weights, knots: [knots.clone(), if torus { knots.clone() } else { vec![0.,0.,1.,1.] }],
            domain: [[0.,4.], if torus { [0.,4.] } else { [0.,1.] }], periodic: [true, torus], reversed,
        };
        surface.certify_repeated_periodic_basis(&Limits::default()).unwrap();
        let source = surface.clone();
        let mut document = Document::with_template_tolerance("periodic surface export");
        let mut entity = Entity::new("periodic", Geometry::Surface(surface), None);
        entity.placement = Transform::scale(scale).unwrap();
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut c = Controller::new(document, Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();
        let target = path(&format!("nurbs-periodic-{torus}-{scale_name}-{reversed}"));
        let receipt = c.export_stl_selected(&target).unwrap();
        let output = triangles(&std::fs::read(&target).unwrap());
        for u in [0.17, 0.81, 1.4, 2.3, 3.17, 3.83] {
            let samples: &[f64] = if torus { &[0.28, 1.09, 2.17, 3.62] } else { &[0.2, 0.5, 0.8] };
            for &v in samples {
                let local = om9_core::surface_kernel::evaluate(&source, u, v, &Limits::default()).unwrap();
                let point = Transform::scale(scale).unwrap().point(local).unwrap();
                let deviation = output.iter().map(|triangle| point_to_stl_triangle_distance(point, triangle.map(|p| p.map(f64::from)))).fold(f64::INFINITY, f64::min);
                maximum_deviation = maximum_deviation.max(deviation);
                assert!(deviation <= receipt.requested_tolerance_mm, "sample deviation={deviation}, tolerance={}, torus={torus}, scale={scale_name}, reversed={reversed}, uv=[{u},{v}]", receipt.requested_tolerance_mm);
            }
        }
        let key = |p: [f32; 3]| p.map(|v| if v == 0. { 0 } else { v.to_bits() });
        let mut edges = BTreeMap::<([u32; 3], [u32; 3]), (usize, i32)>::new();
        let mut signed_volume = 0.;
        for tri in &output {
            let [a,b,c] = tri.map(|point| point.map(f64::from));
            signed_volume += (a[0]*(b[1]*c[2]-b[2]*c[1]) + a[1]*(b[2]*c[0]-b[0]*c[2]) + a[2]*(b[0]*c[1]-b[1]*c[0])) / 6.;
            for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                let a = key(a); let b = key(b);
                let entry = edges.entry((a.min(b), a.max(b))).or_default();
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        let mut adjacency = BTreeMap::<[u32; 3], Vec<[u32; 3]>>::new();
        for ((a,b), (count, orientation)) in edges {
            assert!(count <= 2, "non-manifold encoded edge, torus={torus}");
            if count == 2 { assert_eq!(orientation, 0, "inconsistent encoded winding"); }
            if count == 1 { adjacency.entry(a).or_default().push(b); adjacency.entry(b).or_default().push(a); }
        }
        let boundary = adjacency.values().map(Vec::len).sum::<usize>() / 2;
        assert_eq!(receipt.open_edges, boundary, "indexed and encoded boundaries differ, torus={torus}");
        assert_eq!(receipt.non_manifold_edges, 0);
        assert_eq!(receipt.inconsistent_edges, 0);
        assert!(adjacency.values().all(|neighbors| neighbors.len() == 2), "open boundary chains, torus={torus}");
        let mut seen = BTreeSet::new();
        let mut loops = 0;
        for start in adjacency.keys() {
            if seen.contains(start) { continue; }
            loops += 1;
            let mut pending = vec![*start];
            while let Some(point) = pending.pop() {
                if seen.insert(point) { pending.extend(adjacency[&point].iter().copied()); }
            }
        }
        assert_eq!(loops, if torus { 0 } else { 2 }, "boundary loops, torus={torus}");
        if torus {
            assert!(signed_volume.abs() > 0.01, "degenerate torus volume");
            if let Some(previous) = unreversed_volume {
                assert!(signed_volume * previous < 0., "reversal must reverse encoded signed volume");
            } else { unreversed_volume = Some(signed_volume); }
        }
        std::fs::remove_file(target).unwrap();
       }
      }
    }
    println!("periodic_nurbs_source_to_encoded_stl: 252 samples, maximum sampled distance {maximum_deviation:.9} mm");
}

fn point_to_stl_triangle_distance(p: [f64; 3], triangle: [[f64; 3]; 3]) -> f64 {
    let sub = |a: [f64; 3], b: [f64; 3]| std::array::from_fn::<_, 3, _>(|i| a[i] - b[i]);
    let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
    let [a,b,c] = triangle;
    let ab = sub(b,a); let ac = sub(c,a); let ap = sub(p,a);
    let aa = dot(ab,ab); let bb = dot(ab,ac); let cc = dot(ac,ac);
    let da = dot(ap,ab); let db = dot(ap,ac); let denominator = aa*cc-bb*bb;
    let mut minimum = f64::INFINITY;
    if denominator > 0. {
        let u = (cc*da-bb*db)/denominator; let v = (aa*db-bb*da)/denominator;
        if u >= 0. && v >= 0. && u+v <= 1. {
            let delta = std::array::from_fn(|i| ap[i]-u*ab[i]-v*ac[i]);
            minimum = dot(delta,delta).sqrt();
        }
    }
    for (start,end) in [(a,b),(b,c),(c,a)] {
        let edge = sub(end,start); let offset = sub(p,start); let length2 = dot(edge,edge);
        if length2 > 0. {
            let t = (dot(offset,edge)/length2).clamp(0.,1.);
            let delta = std::array::from_fn(|i| offset[i]-t*edge[i]);
            minimum = minimum.min(dot(delta,delta).sqrt());
        }
    }
    minimum
}

#[test]
fn sampled_deviation_oracle_handles_face_edge_and_vertex() {
    let triangle = [[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]];
    for (point, expected) in [([0.5,0.5,3.], 3.), ([1.,-2.,0.], 2.), ([-3.,-4.,0.], 5.)] {
        let actual = point_to_stl_triangle_distance(point, triangle);
        assert!((actual - expected).abs() < 1e-12, "{point:?}: {actual}");
    }
}

#[test]
fn trimmed_periodic_nurbs_stl_has_one_boundary_loop() {
    use om9_core::geometry::NurbsSurface;
    use std::collections::{BTreeMap, BTreeSet};
    let rows = [[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let weights = [1.,0.7,1.2,0.9,1.,0.7];
    let mut poles = Vec::new();
    let mut w = Vec::new();
    for (i, [x, y]) in rows.into_iter().enumerate() {
        for z in [0., 2.] { poles.push([x,y,z]); w.push(weights[i]); }
    }
    let surface = NurbsSurface { degree: [2,1], size: [6,2], poles, weights: w,
        knots: [vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.], vec![0.,0.,1.,1.]],
        domain: [[0.3,3.7],[0.,1.]], periodic: [true,false], reversed: false };
    surface.certify_repeated_periodic_basis(&Limits::default()).unwrap();
    let mut document = Document::with_template_tolerance("trimmed periodic STL");
    let entity = Entity::new("trimmed periodic", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let target = path("trimmed-periodic-boundary");
    let receipt = controller.export_stl_selected(&target).unwrap();
    let output = triangles(&std::fs::read(&target).unwrap());
    let key = |p: [f32;3]| p.map(|v| if v == 0. { 0 } else { v.to_bits() });
    let mut edges = BTreeMap::<([u32;3], [u32;3]), usize>::new();
    for triangle in output {
        for (a,b) in [(triangle[0],triangle[1]), (triangle[1],triangle[2]), (triangle[2],triangle[0])] {
            let a = key(a); let b = key(b);
            *edges.entry((a.min(b),a.max(b))).or_default() += 1;
        }
    }
    let mut adjacency = BTreeMap::<[u32;3], Vec<[u32;3]>>::new();
    for ((a,b), count) in edges {
        assert!(count <= 2);
        if count == 1 { adjacency.entry(a).or_default().push(b); adjacency.entry(b).or_default().push(a); }
    }
    assert!(!adjacency.is_empty());
    assert!(adjacency.values().all(|neighbors| neighbors.len() == 2));
    assert_eq!(receipt.open_edges, adjacency.values().map(Vec::len).sum::<usize>() / 2);
    let mut seen = BTreeSet::new();
    let mut pending = vec![*adjacency.keys().next().unwrap()];
    while let Some(point) = pending.pop() {
        if seen.insert(point) { pending.extend(adjacency[&point].iter().copied()); }
    }
    assert_eq!(seen.len(), adjacency.len(), "trimmed surface must have one connected boundary loop");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn curved_surface_uses_the_requested_kernel_deflection() {
    use om9_core::geometry::NurbsSurface;
    let mut poles = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            let u = i as f64 / 2.0;
            let v = j as f64 / 2.0;
            poles.push([10.0 * u, 10.0 * v, 8.0 * (u * u + v * v)]);
        }
    }
    let surface = NurbsSurface {
        degree: [2, 2],
        size: [3, 3],
        poles,
        weights: vec![1.0; 9],
        knots: [
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let mut document = Document::with_template_tolerance("curved surface stl");
    let entity = Entity::new("paraboloid", Geometry::Surface(surface), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let target = path("curved");
    let angle_coarse = c.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.5, angular_tolerance_rad: 0.6 }).unwrap();
    let angle_fine = c.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.5, angular_tolerance_rad: 0.1 }).unwrap();
    assert!(angle_fine.triangles > angle_coarse.triangles, "angular coarse {} fine {}", angle_coarse.triangles, angle_fine.triangles);
    let coarse = c.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.1, angular_tolerance_rad: 0.3 }).unwrap();
    let fine = c.export_stl_selected_with_profile(&target, om9_app::session::StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
    assert!(fine.triangles > coarse.triangles, "coarse {} fine {}", coarse.triangles, fine.triangles);
    assert!(!fine.watertight());
    let mut max_sample_error = 0.0_f64;
    for triangle in triangles(&std::fs::read(&target).unwrap()) {
        let center: [f64; 3] = std::array::from_fn(|axis| triangle.iter().map(|p| f64::from(p[axis])).sum::<f64>() / 3.0);
        let (u, v) = (center[0] / 10.0, center[1] / 10.0);
        let source_z = 4.0 * (u * u + u + v * v + v);
        max_sample_error = max_sample_error.max((center[2] - source_z).abs());
    }
    assert!(max_sample_error <= 0.001, "sampled vertical error {max_sample_error}");
    std::fs::remove_file(target).unwrap();
}

#[test]
fn nurbs_surface_reversal_and_mirror_preserve_orientation() {
    for (reversed, mirror, expected) in [(false, false, 1.0), (true, false, -1.0), (false, true, 1.0), (true, true, -1.0)] {
        let mut surface = om9_core::surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 10., 10., &Limits::default()).unwrap();
        surface.reversed = reversed;
        let mut entity = Entity::new("plane", Geometry::Surface(surface), None);
        if mirror { entity.placement = Transform::scale([-1., 1., 1.]).unwrap(); }
        let id = entity.id;
        let mut document = Document::with_template_tolerance("surface orientation");
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = path(&format!("surface-orientation-{reversed}-{mirror}"));
        controller.export_stl_selected(&target).unwrap();
        for [a, b, c] in triangles(&std::fs::read(&target).unwrap()) {
            let normal_z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            assert!(normal_z * expected > 0.0, "reversed={reversed} mirror={mirror} z={normal_z}");
        }
        std::fs::remove_file(target).unwrap();
    }
}

#[test]
fn a_wrong_extension_is_refused_without_creating_a_file() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = only(&c);
    c.select_objects(&[id]).unwrap();
    let target = std::env::temp_dir().join(format!("om9-stl-{}.obj", std::process::id()));
    assert!(c.export_stl_selected(&target).is_err());
    assert!(!target.exists());
}

#[test]
fn a_second_export_replaces_the_destination() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = only(&c);
    c.select_objects(&[id]).unwrap();
    let target = path("replace");
    let first = c.export_stl_selected(&target).unwrap();
    let second = c.export_stl_selected(&target).unwrap();
    assert_eq!(first.bytes, second.bytes);
    let bytes = std::fs::read(&target).unwrap();
    assert_eq!(bytes.len(), second.bytes);
    assert_eq!(triangles(&bytes).len(), second.triangles);
    std::fs::remove_file(target).unwrap();
}
#[test]
fn occt_snapshot_honors_angular_deflection_and_rejects_invalid_values() {
    let limits = om9_core::budget::Limits::default();
    let source = om9_kernel::brep::sphere_brep(10.0, 0.5, &limits).unwrap();
    let coarse = om9_kernel::brep::read_brep_with_angle(source.source(), 0.5, 0.6, &limits).unwrap();
    let fine = om9_kernel::brep::read_brep_with_angle(source.source(), 0.5, 0.1, &limits).unwrap();
    assert!(fine.display.triangles.len() > coarse.display.triangles.len());
    let default = om9_kernel::brep::read_brep(source.source(), 0.5, &limits).unwrap();
    let explicit_default = om9_kernel::brep::read_brep_with_angle(source.source(), 0.5, 0.3, &limits).unwrap();
    assert_eq!(default.display.triangles, explicit_default.display.triangles);
    for angle in [0.0, -0.1, f64::NAN, f64::INFINITY, 4.0] {
        assert!(om9_kernel::brep::read_brep_with_angle(source.source(), 0.5, angle, &limits).is_err());
    }
}
