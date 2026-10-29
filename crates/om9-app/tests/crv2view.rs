#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_views() -> Controller {
    let mut doc = Document::new("crv2view");
    let top = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let front = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 10.0]]).unwrap();
    for (name, curve) in [("top", top), ("front", front)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    Controller::new(doc, Limits::default()).unwrap()
}

#[test]
fn crv2view_combines_two_projection_curves() {
    let mut c = controller_with_views();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Crv2View").unwrap();
    c.submit("").unwrap();
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Crv2View")
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(curve)) = &curve.geometry else {
        panic!("nurbs");
    };
    let max_z = curve
        .poles
        .iter()
        .map(|point| point[2])
        .fold(f64::MIN, f64::max);
    assert!((max_z - 10.0).abs() < 0.5, "{max_z}");
}

/// `Curve 2 Views with History` (OM9-L3-03-16-014): the builder row runs the same
/// two-view construction, records one history recipe with both view curves as inputs,
/// and replays that recipe onto the same output.
#[test]
fn crv2view_history_records_one_replayable_recipe() {
    let mut c = controller_with_views();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    assert_eq!(
        Controller::resolve_alias("Curve 2 Views with History"),
        Some("OM9-CURVE-014")
    );
    assert_eq!(
        Controller::resolve_alias("Crv2ViewHistory"),
        Some("OM9-CURVE-014")
    );
    c.submit("Curve 2 Views with History").unwrap();
    c.submit("").unwrap();

    let (output, curve) = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Crv2ViewHistory")
        .map(|entity| (entity.id, entity.geometry.clone()))
        .expect("history curve");
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(nurbs)) = &curve else {
        panic!("nurbs");
    };
    let max_z = nurbs
        .poles
        .iter()
        .map(|point| point[2])
        .fold(f64::MIN, f64::max);
    assert!((max_z - 10.0).abs() < 0.5, "{max_z}");

    assert_eq!(c.coordinator().undo_len(), 1);
    assert_eq!(c.document().history.recipes.len(), 1);
    let (recipe_id, recipe) = c
        .document()
        .history
        .recipes
        .iter()
        .next()
        .map(|(id, recipe)| (*id, recipe.clone()))
        .expect("history recipe");
    assert_eq!(recipe.command, "Crv2ViewHistory");
    assert_eq!(recipe.outputs, vec![output]);
    assert_eq!(recipe.inputs.len(), 2);
    assert!(recipe.inputs.iter().all(|input| input.role == "view"));
    let mut sources: Vec<_> = recipe
        .inputs
        .iter()
        .map(|input| input.source.object)
        .collect();
    sources.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(sources, expected);

    c.replay_history(recipe_id).expect("replay history");
    assert_eq!(c.coordinator().undo_len(), 2);
    let replayed = c
        .document()
        .entities
        .get(&output)
        .expect("history output")
        .geometry
        .clone();
    let Geometry::Curve(om9_core::geometry::Curve::Nurbs(replayed)) = replayed else {
        panic!("nurbs replay");
    };
    assert_eq!(replayed.poles.len(), nurbs.poles.len());
    assert!(
        replayed
            .poles
            .iter()
            .zip(&nurbs.poles)
            .all(|(a, b)| (a[0] - b[0]).abs() < 1e-9
                && (a[1] - b[1]).abs() < 1e-9
                && (a[2] - b[2]).abs() < 1e-9),
        "a replay without a source edit must reproduce the same curve"
    );
}
