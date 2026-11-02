// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03-20 measure read-outs: golden values, refusals and the Command-Line label.
//!
//! Every measurement is read-only: the document, its revision and the Undo journal
//! must be identical before and after a read-out, which is asserted by the golden
//! cases below (OM9-L3-03-20-001 … -005).
use om9_app::session::{Controller, InvocationOrigin, measure};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Mesh, NativeBrep, NurbsCurve, Point3},
    model::{Definition, Document, Entity, Extension, GEM_EXTENSION_SLOT},
    surface_kernel,
};

fn controller(entities: Vec<Entity>) -> Controller {
    let mut document = Document::new("measure");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).expect("controller")
}

fn line(start: Point3, end: Point3) -> Curve {
    let length =
        ((end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2) + (end[2] - start[2]).powi(2))
            .sqrt();
    Curve::Line {
        start,
        end,
        domain: [0.0, length],
    }
}

fn circle(center: Point3, radius: f64) -> Curve {
    Curve::Circle {
        center,
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    }
}

/// A closed box as a native solid: the display mesh carries the exact corners, so
/// the reported bounding box has the box's exact size and center.
fn box_solid(origin: Point3, size: Point3) -> Geometry {
    let vertex = |index: usize| -> Point3 {
        std::array::from_fn(|axis| {
            origin[axis]
                + if index & (1 << axis) == 0 {
                    0.0
                } else {
                    size[axis]
                }
        })
    };
    let vertices: Vec<Point3> = (0..8).map(vertex).collect();
    let faces = vec![
        vec![0, 1, 3],
        vec![0, 3, 2],
        vec![4, 6, 7],
        vec![4, 7, 5],
        vec![0, 4, 5],
        vec![0, 5, 1],
        vec![2, 3, 7],
        vec![2, 7, 6],
        vec![0, 2, 6],
        vec![0, 6, 4],
        vec![1, 5, 7],
        vec![1, 7, 3],
    ];
    Geometry::NativeBrep(NativeBrep {
        display: Mesh {
            vertices,
            faces,
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        },
        payload: vec![1, 2, 3, 4],
        crc: 0,
    })
}

/// The document fingerprint a read-only command must never change.
fn fingerprint(controller: &Controller) -> (u64, u64, usize, usize) {
    let document = controller.document();
    (
        document.revision,
        document.context_revision,
        document.entities.len(),
        controller.coordinator().undo_len(),
    )
}

#[test]
fn m03_03_20_001_distance_and_length_report_millimetres() {
    let entity = Entity::new(
        "line",
        Geometry::Curve(line([0.0, 0.0, 0.0], [3.0, 4.0, 0.0])),
        None,
    );
    let id = entity.id;
    let mut controller = controller(vec![entity]);
    let before = fingerprint(&controller);
    let document = controller.document().clone();

    controller.submit("Distance").expect("Distance by name");
    assert_eq!(controller.active_feature(), Some(measure::DISTANCE));
    assert_eq!(controller.prompt(), "First distance point: ");
    controller.submit("0,0,0").expect("first point");
    assert_eq!(controller.prompt(), "Second distance point: ");
    controller.submit("3,4,0").expect("second point");
    assert!(!controller.is_running(), "the read-out closes normally");
    assert_eq!(
        controller.take_measurement_label().as_deref(),
        Some("Distance = 5 mm")
    );
    assert_eq!(
        controller.events.run_len(measure::DISTANCE),
        4,
        "Started, one Progress per point, Finished"
    );

    controller.select_objects(&[id]).expect("selection");
    controller.submit("Length").expect("Length by name");
    assert_eq!(
        controller.last_measurement(),
        Some("Length = 5 mm"),
        "the exact arc length of the 3-4-5 line"
    );

    controller.clear_selection();
    assert!(
        controller.submit("Length").is_err(),
        "Length without a curve selection is refused"
    );
    assert_eq!(
        fingerprint(&controller),
        before,
        "no document or Undo change"
    );
    assert_eq!(controller.document(), &document);
}

#[test]
fn m03_03_20_002_angle_and_radius_report_degrees_and_millimetres() {
    let entity = Entity::new(
        "circle",
        Geometry::Curve(circle([0.0, 0.0, 0.0], 5.0)),
        None,
    );
    let id = entity.id;
    let spline = Entity::new(
        "spline",
        Geometry::Curve(Curve::Nurbs(NurbsCurve {
            degree: 2,
            poles: vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]],
            weights: vec![1.0, 1.0, 1.0],
            knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            domain: [0.0, 1.0],
            periodic: false,
            reversed: false,
            seam: None,
        })),
        None,
    );
    let spline_id = spline.id;
    let mut controller = controller(vec![entity, spline]);
    let before = fingerprint(&controller);

    controller.submit("Angle").expect("Angle by name");
    assert_eq!(controller.active_feature(), Some(measure::ANGLE));
    for prompt in [
        "First angle line start: ",
        "First angle line end: ",
        "Second angle line start: ",
        "Second angle line end: ",
    ] {
        assert_eq!(controller.prompt(), prompt);
        controller
            .submit(match prompt {
                "First angle line start: " => "0,0,0",
                "First angle line end: " => "1,0,0",
                "Second angle line start: " => "0,0,0",
                _ => "0,1,0",
            })
            .expect("angle point");
    }
    assert!(!controller.is_running());
    assert_eq!(
        controller.take_measurement_label().as_deref(),
        Some("Angle = 90 deg")
    );

    controller.select_objects(&[id]).expect("selection");
    controller.submit("Radius").expect("Radius by name");
    assert_eq!(controller.last_measurement(), Some("Radius = 5 mm"));

    // A zero-length direction and a spline have no single answer: both are refused,
    // never guessed.
    assert!(
        controller
            .measure_angle([0.0; 3], [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0])
            .is_err()
    );
    controller.select_objects(&[spline_id]).expect("selection");
    controller.submit("Radius").unwrap();
    assert!(controller.is_running(), "a spline needs an explicit curvature station");
    controller.submit("Station=0.5").unwrap();
    assert!(controller.last_measurement().unwrap().starts_with("Local curvature radius"));
    assert_eq!(fingerprint(&controller).3, before.3, "no Undo entry");
}

#[test]
fn m03_03_20_003_bounding_box_reports_size_and_center() {
    let entity = Entity::new("box", box_solid([3.0, 4.0, 5.0], [10.0, 20.0, 30.0]), None);
    let id = entity.id;
    let mut controller = controller(vec![entity]);
    let before = fingerprint(&controller);

    controller.select_objects(&[id]).expect("selection");
    controller
        .dispatch(measure::BOUNDING_BOX, InvocationOrigin::Menu)
        .expect("Bounding Box menu row");
    assert_eq!(
        controller.take_measurement_label().as_deref(),
        Some("Bounding Box = 10 x 20 x 30 mm, center (8, 14, 20)")
    );
    assert_eq!(fingerprint(&controller), before, "no box was created");

    controller.clear_selection();
    assert!(controller.measure_selected_bounds().is_err());
}

#[test]
fn m03_03_20_004_object_summary_counts_types_without_duplicating_definitions() {
    let gem = {
        let mut entity = Entity::new("gem", Geometry::Point([0.0, 0.0, 0.0]), None);
        entity.extensions.insert(
            GEM_EXTENSION_SLOT.to_owned(),
            Extension::new(1, r#"{"kind":"round"}"#).expect("gem extension"),
        );
        entity
    };
    let placeholder = {
        let mut entity = Entity::new("placeholder", Geometry::Point([1.0, 0.0, 0.0]), None);
        entity.extensions.insert(
            "om9.unknown".to_owned(),
            Extension::new(1, r#"{"note":"not a gem"}"#).expect("unknown extension"),
        );
        entity
    };
    let solid = Entity::new("solid", box_solid([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), None);
    let surface = Entity::new(
        "surface",
        Geometry::Surface(
            surface_kernel::plane(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                4.0,
                4.0,
                &Limits::default(),
            )
            .expect("plane"),
        ),
        None,
    );
    let first = Entity::new(
        "first",
        Geometry::Curve(line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])),
        None,
    );
    let second = Entity::new(
        "second",
        Geometry::Curve(circle([0.0, 0.0, 0.0], 1.0)),
        None,
    );
    let mut controller = controller(vec![first, second, surface, solid, gem, placeholder]);
    let before = fingerprint(&controller);

    let summary = controller.document_summary();
    assert_eq!(summary.total, 6);
    assert_eq!(summary.by_kind["Curve"], 2);
    assert_eq!(summary.by_kind["Surface"], 1);
    assert_eq!(summary.by_kind["Brep"], 1);
    assert_eq!(summary.by_kind["Point"], 2);
    assert_eq!(summary.gems, 1, "one gem slot");
    assert_eq!(
        summary.unknown_slots, 1,
        "the unknown slot is counted apart, never as a gem"
    );
    assert_eq!(summary.definitions, 0);

    controller.submit("ObjectSummary").expect("typed name");
    let label = controller.take_measurement_label().expect("label");
    assert!(label.contains("Objects: 6 total"), "{label}");
    assert!(label.contains("gems 1"), "{label}");
    assert!(label.contains("unknown 1"), "{label}");

    // A block definition carrying copies of the same entities must not add to the
    // object count: the placed instance is the counted object.
    let mut document = controller.document().clone();
    let members = document.entities.values().cloned().collect::<Vec<_>>();
    let definition = Definition::new("block", members);
    document.definitions.insert(definition.id, definition);
    let summary = measure::summarize(&document);
    assert_eq!(summary.total, 6, "members are not objects");
    assert_eq!(summary.definitions, 1);

    assert_eq!(fingerprint(&controller), before);
}

#[test]
fn m03_03_20_005_the_label_is_transient_and_never_stale() {
    let entity = Entity::new(
        "line",
        Geometry::Curve(line([0.0, 0.0, 0.0], [3.0, 4.0, 0.0])),
        None,
    );
    let id = entity.id;
    let mut controller = controller(vec![entity]);
    let before = fingerprint(&controller);
    let document = controller.document().clone();

    controller.select_objects(&[id]).expect("selection");
    controller.submit("Length").expect("Length");
    assert_eq!(controller.last_measurement(), Some("Length = 5 mm"));
    assert_eq!(
        controller.take_measurement_label().as_deref(),
        Some("Length = 5 mm"),
        "the status line consumes the label once"
    );
    assert!(
        controller.take_measurement_label().is_none(),
        "an old read-out is never repeated"
    );

    controller.submit("Length").expect("Length again");
    assert!(controller.last_measurement().is_some());
    controller.clear_selection();
    assert!(
        controller.last_measurement().is_none(),
        "a selection change invalidates the read-out"
    );

    assert_eq!(fingerprint(&controller), before, "read-only, no Undo entry");
    assert_eq!(controller.document(), &document);
}
