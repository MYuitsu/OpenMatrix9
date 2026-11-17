// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5's selection commands typed on the command line: `SelCrv`, `SelSrf`, `SelPt`,
//! `SelPolyline`, `SelPolygon`, `SelLine`, `SelClosedCrv`, `SelAll`, `SelNone`. Each adds the
//! objects of its type to the selection (it keeps what was selected) and says how many.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, NurbsCurve},
    identity::ObjectId,
    model::{Document, Entity},
};

fn polyline(points: &[[f64; 3]]) -> Geometry {
    let n = points.len();
    let mut knots = vec![0.0];
    knots.extend((0..n).map(|i| i as f64));
    knots.push((n - 1) as f64);
    Geometry::Curve(Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: points.to_vec(),
        weights: vec![1.0; n],
        knots,
        domain: [0.0, (n - 1) as f64],
        periodic: false,
        reversed: false,
        seam: None,
    }))
}

fn scene() -> (Controller, Vec<(&'static str, ObjectId)>) {
    let mut document = Document::new("select-kind");
    let objects = [
        ("line", Geometry::Curve(Curve::Line { start: [0.0; 3], end: [5.0, 0.0, 0.0], domain: [0.0, 1.0] })),
        (
            "circle",
            Geometry::Curve(Curve::Circle {
                center: [0.0, 10.0, 0.0],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius: 2.0,
                domain: [0.0, std::f64::consts::TAU],
                angle_domain: None,
            }),
        ),
        ("open polyline", polyline(&[[0.0, 20.0, 0.0], [3.0, 22.0, 0.0], [6.0, 20.0, 0.0]])),
        ("triangle", polyline(&[[10.0, 0.0, 0.0], [14.0, 0.0, 0.0], [12.0, 3.0, 0.0], [10.0, 0.0, 0.0]])),
        ("point", Geometry::Point([20.0, 0.0, 0.0])),
    ];
    let mut ids = Vec::new();
    for (name, g) in objects {
        let e = Entity::new(name, g, None);
        ids.push((name, e.id));
        document.entities.insert(e.id, e);
    }
    (Controller::new(document, Limits::default()).unwrap(), ids)
}

fn names(c: &Controller, ids: &[(&'static str, ObjectId)]) -> Vec<&'static str> {
    ids.iter().filter(|(_, id)| c.selected().contains(id)).map(|(n, _)| *n).collect()
}

#[test]
fn sel_commands_add_objects_of_their_type() {
    let (mut c, ids) = scene();
    c.submit("SelCrv").unwrap();
    assert_eq!(names(&c, &ids), ["line", "circle", "open polyline", "triangle"]);
    c.submit("SelNone").unwrap();
    assert!(c.selected().is_empty());
    c.submit("SelPt").unwrap();
    assert_eq!(names(&c, &ids), ["point"]);
    // Adds to what is selected (Rhino), it does not replace it.
    c.submit("SelPolygon").unwrap();
    assert_eq!(names(&c, &ids), ["triangle", "point"]);
    c.submit("SelNone").unwrap();
    c.submit("SelPolyline").unwrap();
    assert_eq!(names(&c, &ids), ["open polyline", "triangle"]);
    c.submit("SelNone").unwrap();
    c.submit("_SelLine").unwrap();
    assert_eq!(names(&c, &ids), ["line"]);
    c.submit("SelNone").unwrap();
    c.submit("SelClosedCrv").unwrap();
    assert_eq!(names(&c, &ids), ["circle", "triangle"]);
    c.submit("SelNone").unwrap();
    c.submit("SelSrf").unwrap();
    assert!(c.selected().is_empty(), "no surfaces here");
    c.submit("SelAll").unwrap();
    assert_eq!(c.selected().len(), 5);
}
