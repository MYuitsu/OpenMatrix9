// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT producer chain for `OM9-L2-01-04`: ring rail -> profile -> real B-Rep sweep.
//!
//! The rail and profile are the pure-Rust producers; the sweep runs through
//! `creation::pipe_shell` and the topology names come from the OCCT B-Rep snapshot. This is
//! the end-to-end proof the graph-kernel tests alone could not give: real OCCT geometry,
//! real subshapes, a semantic reference that resolves after the rebuild.
#![cfg(feature = "3dm-brep")]

use om9_core::{
    associative::{
        FeaturePort, RecomputeCache, RecomputeOutcome, ReferenceAnchor, SubshapeKind,
        bind_reference, resolve_semantic_reference,
    },
    budget::Limits,
    geometry::{Curve, Geometry, GeometryKind, Transform, Unit},
    history::{HistoryGraph, Recipe, ReplayEnvironment, Tolerance},
    identity::{HistoryId, ObjectId},
    model::{Document, Entity},
    producers::{SnapshotTopology, TopologyProvider, recompute_registered},
    request::{Selection, SourceBinding, Value},
};
use om9_kernel::{
    associative::{OcctTopology, history_registry},
    creation::{self, DEFAULT_DEFLECTION_MM},
};

fn limits() -> Limits {
    Limits::default()
}

fn environment() -> ReplayEnvironment {
    ReplayEnvironment {
        command_version: 1,
        solver: "om9-occt".into(),
        solver_version: "1".into(),
        schema_version: 1,
        length_unit: Unit::Millimeter,
        frame: Transform::identity(),
        tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
    }
}

fn length(value: f64) -> Value {
    Value::length(om9_core::units::LengthMm::try_new(value).unwrap())
}

fn add(document: &mut Document, label: &str, geometry: Geometry) -> ObjectId {
    let entity = Entity::new(label, geometry, None);
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

fn binding(document: &Document, id: ObjectId, role: &str) -> SourceBinding {
    SourceBinding {
        role: role.into(),
        source: document.stamp(id).unwrap(),
        selection: Selection::Whole,
    }
}

fn curve_port(role: &str) -> FeaturePort {
    FeaturePort::geometry(role, &[GeometryKind::Curve])
}

fn typed_feature(
    document: &Document,
    command: &str,
    inputs: Vec<(SourceBinding, FeaturePort)>,
    outputs: Vec<ObjectId>,
) -> Recipe {
    let input_ports = inputs.iter().map(|(_, port)| port.clone()).collect();
    let output_ports = outputs
        .iter()
        .map(|id| FeaturePort::geometry("result", &[document.entities[id].geometry.kind()]))
        .collect();
    Recipe::new(
        document.id,
        command,
        inputs.into_iter().map(|(binding, _)| binding).collect(),
        outputs,
        environment(),
    )
    .with_ports(input_ports, output_ports)
}

fn placeholder_brep() -> Geometry {
    let spine = [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
    let section: Vec<[f64; 3]> = (0..16)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / 16.0;
            [0.0, angle.cos(), angle.sin()]
        })
        .collect();
    Geometry::Brep(
        creation::pipe_shell(
            &spine,
            None,
            &[section],
            false,
            DEFAULT_DEFLECTION_MM,
            &limits(),
        )
        .expect("placeholder pipe shell"),
    )
}

struct Chain {
    document: Document,
    rail: HistoryId,
    profile: HistoryId,
    sweep: HistoryId,
    out_sweep: ObjectId,
}

fn chain(width_mm: f64) -> Chain {
    let mut document = Document::new("occt-ring-chain");
    let rail_curve = Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 9.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let profile_curve = Curve::Circle {
        center: [9.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 0.0, 1.0],
        radius: 1.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let out_rail = add(&mut document, "Rail", Geometry::Curve(rail_curve));
    let out_profile = add(&mut document, "Profile", Geometry::Curve(profile_curve));
    let out_sweep = add(&mut document, "Sweep", placeholder_brep());
    let mut rail = typed_feature(&document, "RingRail", vec![], vec![out_rail]);
    rail.parameters.insert("diameter_mm".into(), length(18.0));
    let mut profile = typed_feature(
        &document,
        "RailProfile",
        vec![(binding(&document, out_rail, "rail"), curve_port("rail"))],
        vec![out_profile],
    );
    profile
        .parameters
        .insert("width_mm".into(), length(width_mm));
    profile.parameters.insert("height_mm".into(), length(1.5));
    let sweep = typed_feature(
        &document,
        "RailSweep",
        vec![
            (binding(&document, out_rail, "rail"), curve_port("rail")),
            (
                binding(&document, out_profile, "profile"),
                curve_port("profile"),
            ),
        ],
        vec![out_sweep],
    );
    let ids = (rail.id, profile.id, sweep.id);
    let mut graph = HistoryGraph::default();
    for recipe in [rail, profile, sweep] {
        graph.recipes.insert(recipe.id, recipe);
    }
    document.history = graph;
    Chain {
        document,
        rail: ids.0,
        profile: ids.1,
        sweep: ids.2,
        out_sweep,
    }
}

#[test]
fn the_occt_registry_sweeps_a_real_brep_and_publishes_its_topology() {
    let mut chain = chain(2.0);
    let registry = history_registry().unwrap();
    let mut cache = RecomputeCache::default();
    let mut outcomes = Vec::new();
    for id in [chain.rail, chain.profile, chain.sweep] {
        outcomes
            .push(recompute_registered(&mut chain.document, &mut cache, id, &registry).unwrap());
    }
    assert!(matches!(outcomes[2], RecomputeOutcome::Rebuilt { .. }));
    let Geometry::Brep(brep) = &chain.document.entities[&chain.out_sweep].geometry else {
        panic!("OCCT sweep must produce a B-Rep");
    };
    assert!(
        brep.source().contains("CASCADE Topology"),
        "the Brep must carry the OCCT source"
    );
    let RecomputeOutcome::Rebuilt { subshapes, .. } = &outcomes[2] else {
        panic!("sweep must rebuild");
    };
    let set = subshapes
        .iter()
        .find(|set| set.output == chain.out_sweep)
        .unwrap();
    let names: Vec<&str> = set
        .subshapes
        .iter()
        .map(|subshape| subshape.name.as_str())
        .collect();
    assert!(names.iter().any(|name| name.starts_with("edge:")));
    assert!(names.iter().any(|name| name.starts_with("face:")));
    assert!(names.iter().any(|name| name.starts_with("vertex:")));
    // The OCCT provider and the generic snapshot policy agree on the OCCT snapshot.
    let via_occt = OcctTopology
        .subshapes(
            &chain.document.entities[&chain.out_sweep].geometry,
            &limits(),
        )
        .unwrap();
    let via_snapshot = SnapshotTopology
        .snapshot_subshapes(
            &chain.document.entities[&chain.out_sweep].geometry,
            &limits(),
        )
        .unwrap();
    assert_eq!(via_occt, via_snapshot);
}

#[test]
fn an_occt_sweep_reference_resolves_after_a_profile_edit() {
    let mut chain = chain(2.0);
    let registry = history_registry().unwrap();
    let mut cache = RecomputeCache::default();
    let mut outcomes = Vec::new();
    for id in [chain.rail, chain.profile, chain.sweep] {
        outcomes
            .push(recompute_registered(&mut chain.document, &mut cache, id, &registry).unwrap());
    }
    let RecomputeOutcome::Rebuilt { subshapes, .. } = &outcomes[2] else {
        panic!("sweep must rebuild");
    };
    let set = subshapes
        .iter()
        .find(|set| set.output == chain.out_sweep)
        .unwrap();
    let edge = set
        .subshapes
        .iter()
        .find(|subshape| subshape.kind == SubshapeKind::Edge)
        .expect("the B-Rep publishes edges");
    let reference = bind_reference(
        &mut chain.document.history,
        &limits(),
        chain.sweep,
        chain.out_sweep,
        edge.name.clone(),
        ReferenceAnchor {
            kind: SubshapeKind::Edge,
            fingerprint: edge.signature,
            tolerance_mm: 100.0,
        },
    )
    .unwrap();
    // A profile edit changes the swept B-Rep; the reference must resolve against the new
    // snapshot (a wide tolerance makes this an allowed edit).
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.profile)
        .unwrap()
        .parameters
        .insert("width_mm".into(), length(3.0));
    recompute_registered(&mut chain.document, &mut cache, chain.profile, &registry).unwrap();
    let outcome =
        recompute_registered(&mut chain.document, &mut cache, chain.sweep, &registry).unwrap();
    let RecomputeOutcome::Rebuilt { subshapes, .. } = &outcome else {
        panic!("the OCCT rebuild must succeed, got {outcome:?}");
    };
    let published = subshapes
        .iter()
        .find(|set| set.output == chain.out_sweep)
        .unwrap();
    let resolved = resolve_semantic_reference(
        &chain.document.history,
        &chain.document,
        &limits(),
        reference,
        published,
    )
    .unwrap();
    assert_eq!(resolved.name, edge.name);
}
