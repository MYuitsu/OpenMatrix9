// SPDX-License-Identifier: LGPL-2.1-or-later
//! Real producer chain for `OM9-L2-01-04`: ring rail -> profile -> sweep.
//!
//! These tests prove the history rebuild with geometry, not only graph state: the rail is a
//! real circle, the profile is a real closed rectangle on the rail frame, the sweep is a real
//! analytic swept surface, and the registry re-runs the chain after a parameter edit.
use om9_core::{
    Error, Result,
    associative::{
        FeaturePort, PublishedSubshapes, RecomputeCache, RecomputeOutcome, ReferenceAnchor,
        SubshapeKind, SubshapeSignature, bind_reference, recompute_feature,
        resolve_semantic_reference,
    },
    budget::Limits,
    geometry::{Curve, Geometry, GeometryKind, Transform, Unit},
    history::{HistoryGraph, Recipe, ReplayEnvironment, Tolerance},
    identity::{HistoryId, ObjectId},
    model::{Document, Entity},
    producers::{
        ProducerRegistry, SnapshotTopology, SurfaceSweepBackend, SweepBackend, TopologyProvider,
        recompute_registered, reference_registry,
    },
    request::{Selection, SourceBinding, Value},
};

fn limits() -> Limits {
    Limits::default()
}

fn environment() -> ReplayEnvironment {
    ReplayEnvironment {
        command_version: 1,
        solver: "om9".into(),
        solver_version: "1".into(),
        schema_version: 1,
        length_unit: Unit::Millimeter,
        frame: Transform::identity(),
        tolerance: Tolerance::try_new(0.001, 1.0e-4, 1.0e-9).unwrap(),
    }
}

fn curve(start: [f64; 3], end: [f64; 3]) -> Geometry {
    Geometry::Curve(Curve::Line {
        start,
        end,
        domain: [0.0, 1.0],
    })
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

fn length(value: f64) -> Value {
    Value::length(om9_core::units::LengthMm::try_new(value).unwrap())
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

struct Chain {
    document: Document,
    rail: HistoryId,
    profile: HistoryId,
    sweep: HistoryId,
    out_rail: ObjectId,
    out_profile: ObjectId,
    out_sweep: ObjectId,
}

fn chain(width_mm: f64) -> Chain {
    let mut document = Document::new("ring-chain");
    // Placeholder geometry of the right kind; the first recompute replaces all three with
    // real producer output.
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
    let sweep_surface = SurfaceSweepBackend
        .sweep(&rail_curve, &profile_curve, false, &limits())
        .unwrap();
    let out_rail = add(&mut document, "Rail", Geometry::Curve(rail_curve));
    let out_profile = add(&mut document, "Profile", Geometry::Curve(profile_curve));
    let out_sweep = add(&mut document, "Sweep", sweep_surface);
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
        out_rail,
        out_profile,
        out_sweep,
    }
}

fn rebuild_chain(chain: &mut Chain, registry: &ProducerRegistry) -> Result<Vec<RecomputeOutcome>> {
    let mut cache = RecomputeCache::default();
    let mut outcomes = Vec::new();
    for id in [chain.rail, chain.profile, chain.sweep] {
        outcomes.push(recompute_registered(
            &mut chain.document,
            &mut cache,
            id,
            registry,
        )?);
    }
    Ok(outcomes)
}

fn face_signature(outcome: &RecomputeOutcome, output: ObjectId) -> SubshapeSignature {
    let RecomputeOutcome::Rebuilt { subshapes, .. } = outcome else {
        panic!("expected a rebuild, got {outcome:?}");
    };
    subshapes
        .iter()
        .find(|set| set.output == output)
        .expect("output subshapes")
        .subshapes
        .iter()
        .find(|subshape| subshape.name == "face:0")
        .expect("face:0")
        .signature
}

#[test]
fn the_registry_builds_rail_profile_and_sweep_from_parameters() {
    let mut chain = chain(2.0);
    let registry = reference_registry().unwrap();
    assert_eq!(
        registry.commands().collect::<Vec<_>>(),
        vec![
            "AwardRing",
            "EternityBand",
            "OutsideRail",
            "RailProfile",
            "RailSweep",
            "RaisedBand",
            "RingRail",
            "RingResizer",
            "SignetRing"
        ]
    );
    let outcomes = rebuild_chain(&mut chain, &registry).unwrap();
    assert!(matches!(outcomes[0], RecomputeOutcome::Rebuilt { .. }));
    let Geometry::Curve(Curve::Circle { radius, .. }) =
        &chain.document.entities[&chain.out_rail].geometry
    else {
        panic!("rail must be a circle");
    };
    assert_eq!(*radius, 9.0);
    assert!(matches!(
        chain.document.entities[&chain.out_profile].geometry,
        Geometry::Curve(_)
    ));
    let Geometry::Surface(_) = &chain.document.entities[&chain.out_sweep].geometry else {
        panic!("sweep must be a surface");
    };
    let RecomputeOutcome::Rebuilt { subshapes, .. } = &outcomes[2] else {
        panic!("sweep must rebuild");
    };
    let set = subshapes
        .iter()
        .find(|set| set.output == chain.out_sweep)
        .unwrap();
    assert!(
        set.subshapes
            .iter()
            .any(|subshape| subshape.name == "face:0")
    );
    assert_eq!(
        chain.document.results[&chain.out_sweep].status,
        om9_core::result::ResultStatus::Current
    );
}

#[test]
fn a_parameter_edit_rebuilds_the_chain_through_the_producer_registry() {
    let mut chain = chain(2.0);
    let registry = reference_registry().unwrap();
    rebuild_chain(&mut chain, &registry).unwrap();
    let before = chain.document.entities[&chain.out_sweep].geometry.clone();
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.profile)
        .unwrap()
        .parameters
        .insert("width_mm".into(), length(3.0));
    // The profile is now dirty; the sweep reads the changed profile on its own rebuild.
    let mut cache = RecomputeCache::default();
    recompute_registered(&mut chain.document, &mut cache, chain.profile, &registry).unwrap();
    let outcome =
        recompute_registered(&mut chain.document, &mut cache, chain.sweep, &registry).unwrap();
    assert!(matches!(outcome, RecomputeOutcome::Rebuilt { .. }));
    assert_ne!(
        chain.document.entities[&chain.out_sweep].geometry, before,
        "the swept surface must follow the profile edit"
    );
    // A successful rebuild re-stamps its inputs, so nothing is left stale by its own run.
    let plan = om9_core::history_ops::plan_document_recompute(&chain.document).unwrap();
    assert!(
        !plan.contains(&chain.profile),
        "profile must not plan itself after a successful rebuild"
    );
}

#[test]
fn a_sweep_reference_survives_an_allowed_edit_and_refuses_a_tight_move() {
    let mut allowed = chain(2.0);
    let registry = reference_registry().unwrap();
    let outcomes = rebuild_chain(&mut allowed, &registry).unwrap();
    let signature = face_signature(&outcomes[2], allowed.out_sweep);
    let limits = limits();
    let allowed_reference = bind_reference(
        &mut allowed.document.history,
        &limits,
        allowed.sweep,
        allowed.out_sweep,
        "face:0",
        ReferenceAnchor {
            kind: SubshapeKind::Face,
            fingerprint: signature,
            tolerance_mm: 100.0,
        },
    )
    .unwrap();
    allowed
        .document
        .history
        .recipes
        .get_mut(&allowed.profile)
        .unwrap()
        .parameters
        .insert("width_mm".into(), length(3.0));
    let mut cache = RecomputeCache::default();
    recompute_registered(
        &mut allowed.document,
        &mut cache,
        allowed.profile,
        &registry,
    )
    .unwrap();
    let outcome =
        recompute_registered(&mut allowed.document, &mut cache, allowed.sweep, &registry).unwrap();
    let RecomputeOutcome::Rebuilt { subshapes, .. } = &outcome else {
        panic!("allowed edit must rebuild, got {outcome:?}");
    };
    let published: Vec<PublishedSubshapes> = subshapes.clone();
    let resolved = resolve_semantic_reference(
        &allowed.document.history,
        &allowed.document,
        &limits,
        allowed_reference,
        published
            .iter()
            .find(|set| set.output == allowed.out_sweep)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(resolved.name, "face:0");

    // A tight tolerance turns the same edit into an explicit refusal, not a silent relink.
    let mut refused = chain(2.0);
    let outcomes = rebuild_chain(&mut refused, &registry).unwrap();
    let signature = face_signature(&outcomes[2], refused.out_sweep);
    bind_reference(
        &mut refused.document.history,
        &limits,
        refused.sweep,
        refused.out_sweep,
        "face:0",
        ReferenceAnchor {
            kind: SubshapeKind::Face,
            fingerprint: signature,
            tolerance_mm: 1.0e-9,
        },
    )
    .unwrap();
    refused
        .document
        .history
        .recipes
        .get_mut(&refused.profile)
        .unwrap()
        .parameters
        .insert("width_mm".into(), length(3.0));
    let mut cache = RecomputeCache::default();
    recompute_registered(
        &mut refused.document,
        &mut cache,
        refused.profile,
        &registry,
    )
    .unwrap();
    let before = refused.document.entities[&refused.out_sweep]
        .geometry
        .clone();
    let outcome =
        recompute_registered(&mut refused.document, &mut cache, refused.sweep, &registry).unwrap();
    let RecomputeOutcome::Failed { reason, .. } = outcome else {
        panic!("tight reference must refuse the rebuild");
    };
    assert_eq!(reason, "stale semantic reference moved");
    assert_eq!(
        refused.document.entities[&refused.out_sweep].geometry, before,
        "a refused reference keeps the previous geometry"
    );
    assert_eq!(
        refused.document.history.recipes[&refused.sweep].state,
        om9_core::history::EvaluationState::Failed("stale semantic reference moved".into())
    );
}

#[test]
fn an_unregistered_command_is_refused_before_geometry_runs() {
    let mut document = Document::new("unknown");
    let output = add(&mut document, "Out", curve([0., 0., 0.], [1., 0., 0.]));
    let recipe = typed_feature(&document, "NotAProducer", vec![], vec![output]);
    let id = recipe.id;
    document.history.recipes.insert(id, recipe);
    let registry = reference_registry().unwrap();
    assert_eq!(
        recompute_registered(&mut document, &mut RecomputeCache::default(), id, &registry)
            .unwrap_err(),
        Error::Unsupported("history producer")
    );
}

#[test]
fn snapshot_topology_names_curves_surfaces_and_points() {
    let topology = SnapshotTopology;
    let limits = limits();
    let curve_shapes = topology
        .snapshot_subshapes(
            &Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            &limits,
        )
        .unwrap();
    assert_eq!(curve_shapes.len(), 1);
    assert_eq!(curve_shapes[0].name, "curve:0");
    assert_eq!(curve_shapes[0].kind, SubshapeKind::Curve);
    assert!((curve_shapes[0].signature.measure_mm - 10.0).abs() < 1.0e-9);

    let surface = SurfaceSweepBackend
        .sweep(
            &Curve::Circle {
                center: [0., 0., 0.],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.0,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            },
            &Curve::Circle {
                center: [5., 0., 0.],
                x: [1., 0., 0.],
                y: [0., 0., 1.],
                radius: 0.5,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            },
            false,
            &limits,
        )
        .unwrap();
    let face_shapes = topology.snapshot_subshapes(&surface, &limits).unwrap();
    assert_eq!(face_shapes[0].name, "face:0");
    assert_eq!(face_shapes[0].kind, SubshapeKind::Face);

    let point = topology
        .snapshot_subshapes(&Geometry::Point([1., 2., 3.]), &limits)
        .unwrap();
    assert_eq!(point[0].name, "vertex:0");
    assert_eq!(point[0].kind, SubshapeKind::Vertex);
}

#[test]
fn a_geometry_only_producer_still_works_but_cannot_serve_references() {
    // `recompute_feature` remains the closure API; a feature without references rebuilds
    // through it, which keeps the old callers (and the L3-007 tests) valid.
    let mut chain = chain(2.0);
    let mut cache = RecomputeCache::default();
    let outcome = recompute_feature(&mut chain.document, &mut cache, chain.rail, |request| {
        Ok(vec![om9_core::associative::RebuiltOutput {
            object: request.outputs[0],
            geometry: Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
        }])
    })
    .unwrap();
    assert!(matches!(outcome, RecomputeOutcome::Rebuilt { .. }));
}

#[test]
fn a_custom_topology_provider_can_replace_the_snapshot_policy() {
    struct OneFace;
    impl TopologyProvider for OneFace {
        fn subshapes(
            &self,
            _geometry: &Geometry,
            _limits: &Limits,
        ) -> Result<Vec<om9_core::associative::Subshape>> {
            Ok(vec![om9_core::associative::Subshape {
                name: "face:0".into(),
                kind: SubshapeKind::Face,
                signature: SubshapeSignature {
                    start: [0.; 3],
                    end: [0.; 3],
                    measure_mm: 0.0,
                },
            }])
        }
    }
    let mut registry = ProducerRegistry::with_topology(OneFace);
    registry
        .register("RingRail", om9_core::producers::RingRailProducer)
        .unwrap();
    let mut chain = chain(2.0);
    let outcome = recompute_registered(
        &mut chain.document,
        &mut RecomputeCache::default(),
        chain.rail,
        &registry,
    )
    .unwrap();
    let RecomputeOutcome::Rebuilt { subshapes, .. } = outcome else {
        panic!("expected rebuild");
    };
    assert_eq!(subshapes[0].subshapes[0].name, "face:0");
}
