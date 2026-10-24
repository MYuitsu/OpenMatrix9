// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact line/curve/mesh round trip through the native 3DM reader/writer.
//!
//! The product requirement is deliberately narrow: import/export must let the user continue editing
//! curves and lines, distinguish a mesh from a curve, and preserve the mathematics exactly. History,
//! materials and Rhino tables are not part of this oracle.
//!
//! Ignored by default because it needs the admitted `om9-exchange-native.exe`.
#![cfg(windows)]

use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    curve_build,
    geometry::{Curve, Geometry, Mesh, NurbsCurve, NurbsSurface},
    model::{Definition, Document, Entity, Layer},
    transaction::Cancellation,
};
use std::process::Command;

fn entity(label: &str, geometry: Geometry) -> Entity {
    Entity::new(label, geometry, None)
}

#[test]
#[ignore = "requires an admitted om9-exchange-native.exe"]
fn line_circle_nurbs_and_mesh_survive_the_3dm_round_trip_exactly() {
    let folder = std::env::temp_dir().join(format!("om9-curve-roundtrip-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&folder).unwrap();
    let source = folder.join("source.om9");
    let exported = folder.join("roundtrip.3dm");
    let imported = folder.join("imported.om9");
    let limits = Limits::default();
    let cancellation = Cancellation::default();

    let line = Curve::Line {
        start: [1.23456789012345, -2.34567890123456, 3.45678901234567],
        end: [9.87654321098765, 8.76543210987654, -7.65432109876543],
        domain: [0.0, 1.0],
    };
    let circle = Curve::Circle {
        center: [10.0, -20.0, 30.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 12.3456789012345,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let arc = Curve::Circle {
        center: [-4.0, 5.0, 6.0],
        x: [0.0, 1.0, 0.0],
        y: [-1.0, 0.0, 0.0],
        radius: 7.25,
        domain: [0.0, 1.0],
        angle_domain: Some([0.0, std::f64::consts::FRAC_PI_2]),
    };
    let polyline = curve_build::polyline(&[
        [0.0, 0.0, 0.0],
        [std::f64::consts::PI, 0.0, 0.0],
        [std::f64::consts::PI, std::f64::consts::E, 0.0],
    ])
    .unwrap();
    let nurbs_curve = NurbsCurve {
        degree: 3,
        poles: vec![
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ],
        weights: vec![1.0, 0.7, 0.7, 1.0],
        knots: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    let nurbs = Curve::Nurbs(nurbs_curve.clone());
    let surface = NurbsSurface {
        degree: [2, 2],
        size: [3, 3],
        poles: vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.5],
            [0.0, 2.0, 1.0],
            [1.0, 0.0, 0.25],
            [1.0, 1.0, 0.75],
            [1.0, 2.0, 1.25],
            [2.0, 0.0, 0.5],
            [2.0, 1.0, 1.0],
            [2.0, 2.0, 1.5],
        ],
        weights: vec![1.0; 9],
        knots: [
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let mesh = Geometry::Mesh(Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        faces: vec![vec![0, 1, 2], vec![1, 3, 2]],
        normals: vec![],
        colors: vec![],

        corner_uvs: Vec::new(),
    });

    let mut document = Document::new("curve round trip");
    let mut walls = Layer::new("Walls", [12, 34, 56, 255]);
    walls.visible = false;
    walls.locked = true;
    let walls_id = walls.id;
    document.layers.insert(walls_id, walls);
    document.layer_order.push(walls_id);
    document.current_layer = Some(walls_id);
    for (label, geometry) in [
        ("line", Geometry::Curve(line.clone())),
        ("circle", Geometry::Curve(circle.clone())),
        ("arc", Geometry::Curve(arc.clone())),
        ("polyline", Geometry::Curve(polyline.clone())),
        ("nurbs", Geometry::Curve(nurbs.clone())),
        ("surface", Geometry::Surface(surface.clone())),
        ("mesh", mesh.clone()),
    ] {
        let mut e = entity(label, geometry);
        if label == "line" {
            e.attributes.layer = Some(walls_id);
        }
        document.entities.insert(e.id, e);
    }
    let block_member = entity("block-member", Geometry::Curve(line.clone()));
    let block = Definition::new("Block A", vec![block_member]);
    let block_id = block.id;
    document.definitions.insert(block_id, block);
    let block_instance = entity("block-instance", Geometry::Instance(block_id));
    document.entities.insert(block_instance.id, block_instance);
    Archive::save(&source, &document, &limits, &cancellation).unwrap();

    let binary = env!("CARGO_BIN_EXE_om9");
    let output = Command::new(binary)
        .arg("--export-3dm")
        .arg(&source)
        .arg(&exported)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "export: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(binary)
        .arg("--import-3dm")
        .arg(&exported)
        .arg(&imported)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "import: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reopened = Archive::open(&imported, &limits).unwrap();
    assert!(
        reopened.history.recipes.is_empty(),
        "geometry import must not manufacture construction history"
    );
    let by_label = |label: &str| {
        reopened
            .entities
            .values()
            .find(|entity| entity.label == label)
            .unwrap_or_else(|| panic!("imported entity {label} missing"))
            .geometry
            .clone()
    };
    assert_eq!(by_label("line"), Geometry::Curve(line.clone()));
    assert_eq!(by_label("circle"), Geometry::Curve(circle));
    assert_eq!(by_label("arc"), Geometry::Curve(arc));
    assert_eq!(by_label("polyline"), Geometry::Curve(polyline));
    let Geometry::Curve(Curve::Nurbs(imported_nurbs)) = by_label("nurbs") else {
        panic!("NURBS curve was reclassified");
    };
    assert_eq!(imported_nurbs.degree, nurbs_curve.degree);
    assert_eq!(imported_nurbs.weights, nurbs_curve.weights);
    assert_eq!(imported_nurbs.knots, nurbs_curve.knots);
    assert_eq!(imported_nurbs.domain, nurbs_curve.domain);
    assert_eq!(imported_nurbs.periodic, nurbs_curve.periodic);
    // ON stores rational curves in homogeneous form; the homogeneous/Euclidean
    // conversion can move a coordinate by one ULP. No shape approximation is
    // accepted: every pole must agree to double precision.
    assert_eq!(imported_nurbs.poles.len(), nurbs_curve.poles.len());
    for (before, after) in nurbs_curve.poles.iter().zip(&imported_nurbs.poles) {
        for axis in 0..3 {
            assert!(
                (before[axis] - after[axis]).abs() <= 1e-12,
                "NURBS pole drift: {before:?} -> {after:?}"
            );
        }
    }
    assert_eq!(by_label("surface"), Geometry::Surface(surface));
    assert_eq!(by_label("mesh"), mesh);
    // Kinds stay distinct: a mesh is not reclassified as a curve and vice versa.
    assert!(matches!(by_label("mesh"), Geometry::Mesh(_)));
    assert!(matches!(
        by_label("line"),
        Geometry::Curve(Curve::Line { .. })
    ));
    assert!(matches!(
        by_label("circle"),
        Geometry::Curve(Curve::Circle { .. })
    ));
    assert!(matches!(
        by_label("nurbs"),
        Geometry::Curve(Curve::Nurbs(_))
    ));
    let reopened_walls = reopened
        .layers
        .values()
        .find(|layer| layer.name == "Walls")
        .expect("imported Walls layer");
    assert_eq!(reopened_walls.color, [12, 34, 56, 255]);
    assert!(!reopened_walls.visible);
    assert!(reopened_walls.locked);
    let line_entity = reopened
        .entities
        .values()
        .find(|entity| entity.label == "line")
        .expect("imported line");
    assert_eq!(line_entity.attributes.layer, Some(reopened_walls.id));
    let instance = reopened
        .entities
        .values()
        .find(|entity| entity.label == "block-instance")
        .expect("imported block instance");
    let Geometry::Instance(definition_id) = instance.geometry else {
        panic!("block instance changed kind");
    };
    let definition = reopened
        .definitions
        .get(&definition_id)
        .expect("imported block definition");
    assert_eq!(definition.label, "Block A");
    assert_eq!(definition.members.len(), 1);
    assert_eq!(definition.members[0].label, "block-member");
    assert_eq!(
        definition.members[0].geometry,
        Geometry::Curve(line.clone())
    );

    let second_3dm = folder.join("roundtrip-2.3dm");
    let second_om9 = folder.join("roundtrip-2.om9");
    for args in [
        vec![
            "--export-3dm",
            imported.to_str().unwrap(),
            second_3dm.to_str().unwrap(),
        ],
        vec![
            "--import-3dm",
            second_3dm.to_str().unwrap(),
            second_om9.to_str().unwrap(),
        ],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "second round trip: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let second = Archive::open(&second_om9, &limits).unwrap();
    assert_eq!(second.entities.len(), reopened.entities.len());
    assert_eq!(second.definitions.len(), 1);
    assert!(second.history.recipes.is_empty());

    std::fs::remove_file(source).unwrap();
    std::fs::remove_file(exported).unwrap();
    std::fs::remove_file(imported).unwrap();
    std::fs::remove_file(second_3dm).unwrap();
    std::fs::remove_file(second_om9).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
