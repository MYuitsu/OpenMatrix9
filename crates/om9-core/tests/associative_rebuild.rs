// SPDX-License-Identifier: LGPL-2.1-or-later
//! M04 `OM9-L2-01-04` — associative dependency rebuild.
//!
//! Each test names the L3 it proves. The fixtures are deliberately small (curves and one
//! point) because the behaviour under test is the *graph*: typed edges, provenance,
//! propagation order, semantic references, cache invalidation, preview isolation, failed
//! rebuild rollback, delete policy and deterministic baselines.
use om9_core::{
    Error, Result,
    associative::{
        BrokenInputPolicy, FeatureGraphBaseline, FeaturePort, PortKind, PreviewSession,
        PublishedSubshapes, RebuildActivity, RebuildRequest, RebuiltOutput, RecomputeCache,
        RecomputeOutcome, ReferenceAnchor, Subshape, SubshapeKind, SubshapeSignature,
        bind_reference, delete_source_input, drop_references_of, feature_graph_baseline,
        feature_provenance, load_baseline, recompute_feature, record_feature, reopen_feature,
        resolve_semantic_reference, typed_edges,
    },
    budget::Limits,
    geometry::{Curve, Geometry, GeometryKind, Transform, Unit},
    history::{EvaluationState, HistoryGraph, Recipe, ReplayEnvironment, Tolerance},
    history_ops::{detach, mark_document_stale, plan_recompute, stale_recipes, state_counts},
    identity::{HistoryId, ObjectId},
    model::{Document, Entity},
    request::{Options, Selection, SourceBinding, Value},
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

/// Build a typed feature record. Output ports take the output's current geometry kind, so
/// the fixture can never declare a type the produced geometry does not have.
fn feature(
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

fn curve_port(role: &str) -> FeaturePort {
    FeaturePort::geometry(role, &[GeometryKind::Curve])
}

struct RailChain {
    document: Document,
    rail: HistoryId,
    profile: HistoryId,
    sweep: HistoryId,
    out_rail: ObjectId,
}

/// `rail -> profile -> sweep`, three typed nodes and two typed edges (`L3-001` oracle).
fn rail_chain() -> RailChain {
    let mut document = Document::new("rail-chain");
    let out_rail = add(&mut document, "Rail", curve([0., 0., 0.], [10., 0., 0.]));
    let out_profile = add(&mut document, "Profile", curve([0., 0., 0.], [0., 5., 0.]));
    let out_sweep = add(&mut document, "Sweep", curve([0., 0., 0.], [0., 0., 5.]));
    let rail = feature(&document, "RingRail", vec![], vec![out_rail]);
    let profile = feature(
        &document,
        "RailProfile",
        vec![(binding(&document, out_rail, "rail"), curve_port("rail"))],
        vec![out_profile],
    );
    let sweep = feature(
        &document,
        "RailSweep",
        vec![(
            binding(&document, out_profile, "profile"),
            curve_port("profile"),
        )],
        vec![out_sweep],
    );
    let mut graph = HistoryGraph::default();
    let ids = (rail.id, profile.id, sweep.id);
    for recipe in [rail, profile, sweep] {
        graph.recipes.insert(recipe.id, recipe);
    }
    document.history = graph;
    RailChain {
        document,
        rail: ids.0,
        profile: ids.1,
        sweep: ids.2,
        out_rail,
    }
}

struct Fanout {
    document: Document,
    source: ObjectId,
    a: HistoryId,
    b: HistoryId,
    c: HistoryId,
    d: HistoryId,
}

/// `source -> A`, then `A -> B -> C` and `A -> D` (`L3-003` oracle).
fn fanout() -> Fanout {
    let mut document = Document::new("fanout");
    let source = add(&mut document, "Source", curve([0., 0., 0.], [10., 0., 0.]));
    let out_a = add(&mut document, "A-out", curve([0., 0., 0.], [1., 0., 0.]));
    let out_b = add(&mut document, "B-out", curve([0., 0., 0.], [2., 0., 0.]));
    let out_c = add(&mut document, "C-out", curve([0., 0., 0.], [3., 0., 0.]));
    let out_d = add(&mut document, "D-out", curve([0., 0., 0.], [4., 0., 0.]));
    let a = feature(
        &document,
        "A",
        vec![(binding(&document, source, "rail"), curve_port("rail"))],
        vec![out_a],
    );
    let b = feature(
        &document,
        "B",
        vec![(binding(&document, out_a, "rail"), curve_port("rail"))],
        vec![out_b],
    );
    let c = feature(
        &document,
        "C",
        vec![(binding(&document, out_b, "profile"), curve_port("profile"))],
        vec![out_c],
    );
    let d = feature(
        &document,
        "D",
        vec![(binding(&document, out_a, "rail"), curve_port("rail"))],
        vec![out_d],
    );
    let mut graph = HistoryGraph::default();
    let ids = (a.id, b.id, c.id, d.id);
    for recipe in [a, b, c, d] {
        graph.recipes.insert(recipe.id, recipe);
    }
    document.history = graph;
    Fanout {
        document,
        source,
        a: ids.0,
        b: ids.1,
        c: ids.2,
        d: ids.3,
    }
}

/* --------------------------- L3-001 typed edges --------------------------- */

#[test]
fn typed_edges_follow_the_declared_io_and_link_producers() {
    let chain = rail_chain();
    let graph = &chain.document.history;
    assert_eq!(graph.execution_order().unwrap().len(), 3, "three nodes");
    let edges = typed_edges(&chain.document, graph, &limits()).unwrap();
    assert_eq!(edges.len(), 2, "rail->profile and profile->sweep");
    let profile = edges
        .iter()
        .find(|edge| edge.consumer == chain.profile)
        .unwrap();
    assert_eq!(profile.producer, Some(chain.rail));
    assert_eq!(profile.role, "rail");
    assert_eq!(
        profile.kind,
        PortKind::Geometry {
            kinds: vec![GeometryKind::Curve]
        }
    );
    let sweep = edges
        .iter()
        .find(|edge| edge.consumer == chain.sweep)
        .unwrap();
    assert_eq!(sweep.producer, Some(chain.profile));
    assert_eq!(sweep.role, "profile");
}

#[test]
fn a_curve_in_a_scalar_input_and_a_wrong_output_type_are_refused() {
    let base = rail_chain();
    // A curve fed into a `ScalarMm` (Length) input is not converted.
    let mut scalar = base.document.clone();
    scalar
        .history
        .recipes
        .get_mut(&base.profile)
        .unwrap()
        .input_ports = vec![FeaturePort::length("rail")];
    assert_eq!(
        scalar.history.validate(&scalar, &limits()).unwrap_err(),
        Error::Invalid("object bound to scalar input")
    );
    // A port whose role does not name the binding is a different error, not a silent skip.
    let mut role = base.document.clone();
    role.history
        .recipes
        .get_mut(&base.profile)
        .unwrap()
        .input_ports = vec![curve_port("not-the-role")];
    assert_eq!(
        role.history.validate(&role, &limits()).unwrap_err(),
        Error::Invalid("typed input port role")
    );
    // The output port must describe the produced kind.
    let mut output = base.document.clone();
    output
        .history
        .recipes
        .get_mut(&base.rail)
        .unwrap()
        .output_ports = vec![FeaturePort::geometry("result", &[GeometryKind::Surface])];
    assert_eq!(
        output.history.validate(&output, &limits()).unwrap_err(),
        Error::Invalid("typed feature output")
    );
    // A count mismatch is refused before any of the above is considered.
    let mut count = base.document.clone();
    count
        .history
        .recipes
        .get_mut(&base.sweep)
        .unwrap()
        .input_ports = vec![];
    assert_eq!(
        count.history.validate(&count, &limits()).unwrap_err(),
        Error::Invalid("typed input port count")
    );
}

#[test]
fn a_legacy_recipe_without_ports_is_reported_untyped_never_guessed() {
    let mut chain = rail_chain();
    let rail = chain.document.history.recipes.get_mut(&chain.rail).unwrap();
    rail.input_ports.clear();
    rail.output_ports.clear();
    let edges = typed_edges(&chain.document, &chain.document.history, &limits()).unwrap();
    // The rail has no input, so the untyped recipe appears as the consumer of nothing; the
    // profile still carries its own typed port.
    assert_eq!(
        edges
            .iter()
            .find(|edge| edge.consumer == chain.profile)
            .unwrap()
            .kind,
        PortKind::Geometry {
            kinds: vec![GeometryKind::Curve]
        }
    );
    // Removing the *profile's* ports shows an `Untyped` edge rather than a guess.
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.profile)
        .unwrap()
        .input_ports
        .clear();
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.profile)
        .unwrap()
        .output_ports
        .clear();
    let edges = typed_edges(&chain.document, &chain.document.history, &limits()).unwrap();
    let profile = edges
        .iter()
        .find(|edge| edge.consumer == chain.profile)
        .unwrap();
    assert_eq!(profile.kind, PortKind::Untyped);
    assert_eq!(profile.role, "rail");
}

#[test]
fn record_feature_enforces_identity_document_and_output_ownership() {
    let mut chain = rail_chain();
    let output = add(
        &mut chain.document,
        "Extra",
        curve([0., 0., 0.], [1., 1., 0.]),
    );
    let extra = feature(&chain.document, "Extra", vec![], vec![output]);
    let mut graph = chain.document.history.clone();
    let id = record_feature(&chain.document, &mut graph, extra.clone(), &limits()).unwrap();
    assert_eq!(id, extra.id);
    assert_eq!(
        record_feature(&chain.document, &mut graph, extra.clone(), &limits()).unwrap_err(),
        Error::Invalid("duplicate feature record")
    );
    let mut twin = extra.clone();
    twin.id = HistoryId::new();
    assert_eq!(
        record_feature(&chain.document, &mut graph, twin, &limits()).unwrap_err(),
        Error::Invalid("feature output already owned")
    );
    let mut other = extra.clone();
    other.id = HistoryId::new();
    other.document = om9_core::identity::DocumentId::new();
    assert_eq!(
        record_feature(&chain.document, &mut graph, other, &limits()).unwrap_err(),
        Error::Invalid("feature record document")
    );
}

/* --------------------------- L3-002 provenance ---------------------------- */

#[test]
fn provenance_reopens_the_exact_command_parameters_inputs_and_unit() {
    let mut chain = rail_chain();
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.rail)
        .unwrap()
        .parameters
        .insert("diameter_mm".into(), length(18.0));
    let recipe = &chain.document.history.recipes[&chain.rail];
    let provenance = feature_provenance(recipe).unwrap();
    assert_eq!(provenance.command, "RingRail");
    assert_eq!(provenance.command_version, 1);
    assert_eq!(provenance.unit, Unit::Millimeter);
    assert_eq!(
        provenance.parameters["diameter_mm"]
            .as_length()
            .unwrap()
            .mm(),
        18.0
    );
    assert!(provenance.inputs.is_empty());
    let reopened = reopen_feature(&provenance).unwrap();
    assert_eq!(reopened.command, "RingRail");
    assert_eq!(reopened.parameters, provenance.parameters);
    assert_eq!(reopened.unit, Unit::Millimeter);
    // The profile recipe's input is reopened with role, stamp and selection intact.
    let profile = feature_provenance(&chain.document.history.recipes[&chain.profile]).unwrap();
    assert_eq!(profile.inputs.len(), 1);
    assert_eq!(profile.inputs[0].role, "rail");
    assert_eq!(profile.inputs[0].stamp.object, chain.out_rail);
    assert_eq!(profile.inputs[0].kind, curve_port("rail").kind);
    let reopened = reopen_feature(&profile).unwrap();
    assert_eq!(
        reopened.inputs,
        chain.document.history.recipes[&chain.profile].inputs
    );
}

#[test]
fn provenance_refuses_a_bare_number_a_missing_input_id_and_a_duplicate_role() {
    let base = rail_chain();
    // A recorded parameter without a unit cannot be replayed as millimetres.
    let mut bare = base.document.history.recipes[&base.rail].clone();
    bare.parameters
        .insert("diameter_mm".into(), Value::Number(18.0));
    assert_eq!(
        feature_provenance(&bare).unwrap_err(),
        Error::Invalid("feature parameter unit/type")
    );
    // A payload whose required input id is nil is not a reopenable feature.
    let mut missing = base.document.history.recipes[&base.profile].clone();
    missing.inputs[0].source.object = ObjectId::nil();
    assert_eq!(
        feature_provenance(&missing).unwrap_err(),
        Error::Invalid("feature input identity")
    );
    // Two inputs with the same role would make the reopen ambiguous.
    let mut duplicate = base.document.history.recipes[&base.profile].clone();
    let mut second = duplicate.inputs[0].clone();
    second.selection = Selection::Indices(vec![0]);
    duplicate.inputs.push(second);
    duplicate.input_ports.push(curve_port("rail"));
    assert_eq!(
        feature_provenance(&duplicate).unwrap_err(),
        Error::Invalid("feature input role")
    );
    // An empty explicit selection names nothing.
    let mut empty = base.document.history.recipes[&base.profile].clone();
    empty.inputs[0].selection = Selection::Indices(vec![]);
    assert_eq!(
        feature_provenance(&empty).unwrap_err(),
        Error::Invalid("feature input selection")
    );
}

/* ------------------- L3-003 dirty propagation and order -------------------- */

#[test]
fn editing_the_source_dirties_b_and_d_then_c_in_topological_order() {
    let mut fixture = fanout();
    let source = fixture.document.entities.get_mut(&fixture.source).unwrap();
    source.geometry_revision += 1;
    let stale = stale_recipes(&fixture.document, &fixture.document.history);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].id, fixture.a);
    assert_eq!(stale[0].reason, "source changed");
    let changed = mark_document_stale(&mut fixture.document).unwrap();
    assert_eq!(changed[0], fixture.a, "producer first");
    let dirty: std::collections::BTreeSet<_> = changed.iter().copied().collect();
    for expected in [fixture.a, fixture.b, fixture.c, fixture.d] {
        assert!(dirty.contains(&expected));
    }
    let position = |id: HistoryId| changed.iter().position(|changed| *changed == id).unwrap();
    assert!(position(fixture.b) < position(fixture.c), "B before C");
    assert_eq!(
        fixture.document.history.recipes[&fixture.c].state,
        EvaluationState::Dirty("downstream of dirty recipe".into())
    );
    let plan = plan_recompute(&fixture.document.history).unwrap();
    let unique: std::collections::BTreeSet<_> = plan.iter().copied().collect();
    assert_eq!(unique.len(), plan.len(), "no duplicate evaluation");
    assert_eq!(unique.len(), 4);
    assert_eq!(state_counts(&fixture.document.history)["dirty"], 4);
}

#[test]
fn a_cycle_is_refused_instead_of_being_ordered_or_planned() {
    let mut document = Document::new("cycle");
    let out_a = add(&mut document, "A", curve([0., 0., 0.], [1., 0., 0.]));
    let out_b = add(&mut document, "B", curve([0., 0., 0.], [2., 0., 0.]));
    let a = feature(
        &document,
        "A",
        vec![(binding(&document, out_b, "rail"), curve_port("rail"))],
        vec![out_a],
    );
    let b = feature(
        &document,
        "B",
        vec![(binding(&document, out_a, "rail"), curve_port("rail"))],
        vec![out_b],
    );
    for recipe in [a, b] {
        document.history.recipes.insert(recipe.id, recipe);
    }
    assert_eq!(
        document.history.execution_order().unwrap_err(),
        Error::Invalid("History cycle")
    );
    assert_eq!(
        plan_recompute(&document.history).unwrap_err(),
        Error::Invalid("History cycle")
    );
    assert_eq!(
        typed_edges(&document, &document.history, &limits()).unwrap_err(),
        Error::Invalid("History cycle")
    );
}

/* ---------------------- L3-004 semantic reference links -------------------- */

fn signature(start: [f64; 3], end: [f64; 3]) -> SubshapeSignature {
    SubshapeSignature {
        start,
        end,
        measure_mm: 10.0,
    }
}

/// The named rail-seam edge at a 0.05 mm drift tolerance.
fn edge_anchor(start: [f64; 3], end: [f64; 3]) -> ReferenceAnchor {
    ReferenceAnchor {
        kind: SubshapeKind::Edge,
        fingerprint: signature(start, end),
        tolerance_mm: 0.05,
    }
}

#[test]
fn a_semantic_reference_survives_an_allowed_edit_and_refuses_ambiguity() {
    let mut chain = rail_chain();
    let limits = limits();
    let id = bind_reference(
        &mut chain.document.history,
        &limits,
        chain.rail,
        chain.out_rail,
        "rail-seam",
        edge_anchor([0., 0., 0.], [10., 0., 0.]),
    )
    .unwrap();
    let revision = chain.document.history.recipes[&chain.rail].revision;
    let published = PublishedSubshapes {
        recipe: chain.rail,
        output: chain.out_rail,
        revision,
        subshapes: vec![
            // Allowed edit: the same named edge moved by 0.01 mm.
            Subshape {
                name: "rail-seam".into(),
                kind: SubshapeKind::Edge,
                signature: signature([0.01, 0., 0.], [10.01, 0., 0.]),
            },
            Subshape {
                name: "rail-other".into(),
                kind: SubshapeKind::Edge,
                signature: signature([0., 1., 0.], [10., 1., 0.]),
            },
        ],
    };
    let resolved = resolve_semantic_reference(
        &chain.document.history,
        &chain.document,
        &limits,
        id,
        &published,
    )
    .unwrap();
    assert_eq!(resolved.name, "rail-seam");
    assert_eq!(resolved.kind, SubshapeKind::Edge);

    // Two subshapes with the same name after a topology change: never a silent relink.
    let mut ambiguous = published.clone();
    ambiguous.subshapes.push(Subshape {
        name: "rail-seam".into(),
        kind: SubshapeKind::Edge,
        signature: signature([0., 0., 0.], [5., 0., 0.]),
    });
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &ambiguous
        )
        .unwrap_err(),
        Error::Invalid("ambiguous semantic reference")
    );
    // A different kind is a different subshape.
    let mut wrong_kind = published.clone();
    wrong_kind.subshapes[0].kind = SubshapeKind::Face;
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &wrong_kind
        )
        .unwrap_err(),
        Error::Invalid("semantic subshape kind")
    );
    // Beyond the recorded tolerance the reference moved too far: explicit relink only.
    let mut moved = published.clone();
    moved.subshapes[0].signature = signature([0., 0., 0.], [20., 0., 0.]);
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &moved
        )
        .unwrap_err(),
        Error::Stale("semantic reference moved")
    );
    om9_core::associative::relink_reference(
        &mut chain.document.history,
        &limits,
        id,
        signature([0., 0., 0.], [20., 0., 0.]),
    )
    .unwrap();
    assert!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &moved
        )
        .is_ok()
    );
    // A missing name and a stale revision are distinct answers.
    let mut missing = published.clone();
    missing
        .subshapes
        .retain(|subshape| subshape.name != "rail-seam");
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &missing
        )
        .unwrap_err(),
        Error::Missing("semantic subshape")
    );
    let mut stale = moved;
    stale.revision += 1;
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &stale
        )
        .unwrap_err(),
        Error::Stale("semantic reference revision")
    );
    // Dropping the owning feature drops the reference.
    assert_eq!(
        drop_references_of(&mut chain.document.history, chain.rail),
        1
    );
    assert!(chain.document.history.references.is_empty());
}

#[test]
fn a_detached_feature_cannot_keep_a_semantic_reference() {
    let mut chain = rail_chain();
    let limits = limits();
    let id = bind_reference(
        &mut chain.document.history,
        &limits,
        chain.rail,
        chain.out_rail,
        "rail-seam",
        edge_anchor([0., 0., 0.], [10., 0., 0.]),
    )
    .unwrap();
    detach(&mut chain.document.history, chain.rail).unwrap();
    let published = PublishedSubshapes {
        recipe: chain.rail,
        output: chain.out_rail,
        revision: chain.document.history.recipes[&chain.rail].revision,
        subshapes: vec![Subshape {
            name: "rail-seam".into(),
            kind: SubshapeKind::Edge,
            signature: signature([0., 0., 0.], [10., 0., 0.]),
        }],
    };
    assert_eq!(
        resolve_semantic_reference(
            &chain.document.history,
            &chain.document,
            &limits,
            id,
            &published
        )
        .unwrap_err(),
        Error::Invalid("semantic reference detached feature")
    );
    // A duplicate binding to the same role is refused at bind time.
    let mut duplicate = rail_chain();
    let first = bind_reference(
        &mut duplicate.document.history,
        &limits,
        duplicate.rail,
        duplicate.out_rail,
        "rail-seam",
        edge_anchor([0., 0., 0.], [10., 0., 0.]),
    )
    .unwrap();
    assert!(!first.is_nil());
    assert_eq!(
        bind_reference(
            &mut duplicate.document.history,
            &limits,
            duplicate.rail,
            duplicate.out_rail,
            "rail-seam",
            edge_anchor([0., 0., 0.], [10., 0., 0.]),
        )
        .unwrap_err(),
        Error::Invalid("duplicate semantic reference")
    );
}

/* --------------------- L3-005 incremental cache --------------------------- */

struct Incremental {
    document: Document,
    source: ObjectId,
    a: HistoryId,
    b: HistoryId,
    x: HistoryId,
    y: HistoryId,
}

fn incremental() -> Incremental {
    let mut document = Document::new("incremental");
    let source = add(&mut document, "Source", curve([0., 0., 0.], [10., 0., 0.]));
    let out_a = add(&mut document, "A", curve([0., 0., 0.], [1., 0., 0.]));
    let out_b = add(&mut document, "B", curve([0., 0., 0.], [2., 0., 0.]));
    let out_x = add(&mut document, "X", curve([0., 0., 0.], [3., 0., 0.]));
    let out_y = add(&mut document, "Y", curve([0., 0., 0.], [4., 0., 0.]));
    let a = feature(
        &document,
        "A",
        vec![(binding(&document, source, "rail"), curve_port("rail"))],
        vec![out_a],
    );
    let b = feature(
        &document,
        "B",
        vec![(binding(&document, out_a, "rail"), curve_port("rail"))],
        vec![out_b],
    );
    let x = feature(&document, "X", vec![], vec![out_x]);
    let y = feature(&document, "Y", vec![], vec![out_y]);
    let mut graph = HistoryGraph::default();
    let ids = (a.id, b.id, x.id, y.id);
    for recipe in [a, b, x, y] {
        graph.recipes.insert(recipe.id, recipe);
    }
    document.history = graph;
    Incremental {
        document,
        source,
        a: ids.0,
        b: ids.1,
        x: ids.2,
        y: ids.3,
    }
}

#[test]
fn only_the_affected_chain_recomputes_and_independent_cache_entries_are_untouched() {
    let mut fixture = incremental();
    let mut cache = RecomputeCache::default();
    for id in fixture.document.history.execution_order().unwrap() {
        cache
            .record(&fixture.document, &fixture.document.history.recipes[&id])
            .unwrap();
    }
    let x_before = cache.entries[&fixture.x].clone();
    let y_before = cache.entries[&fixture.y].clone();
    assert!(cache.is_valid(
        &fixture.document,
        &fixture.document.history.recipes[&fixture.x]
    ));

    fixture
        .document
        .entities
        .get_mut(&fixture.source)
        .unwrap()
        .geometry_revision += 1;
    mark_document_stale(&mut fixture.document).unwrap();
    let plan = cache
        .plan(&fixture.document, &fixture.document.history)
        .unwrap();
    assert_eq!(
        plan.iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        [fixture.a, fixture.b].into_iter().collect(),
        "only A and B recalculate"
    );
    assert!(cache.is_valid(
        &fixture.document,
        &fixture.document.history.recipes[&fixture.x]
    ));
    assert!(cache.is_valid(
        &fixture.document,
        &fixture.document.history.recipes[&fixture.y]
    ));
    assert_eq!(cache.entries[&fixture.x], x_before);
    assert_eq!(cache.entries[&fixture.y], y_before);
    assert!(!cache.is_valid(
        &fixture.document,
        &fixture.document.history.recipes[&fixture.a]
    ));
}

#[test]
fn a_cache_entry_with_an_outdated_input_stamp_is_a_miss_even_when_the_graph_is_clean() {
    let fixture = incremental();
    let mut cache = RecomputeCache::default();
    for id in fixture.document.history.execution_order().unwrap() {
        cache
            .record(&fixture.document, &fixture.document.history.recipes[&id])
            .unwrap();
    }
    let entry = cache.entries.get_mut(&fixture.a).unwrap();
    entry.inputs[0].geometry_revision = entry.inputs[0].geometry_revision.saturating_sub(1);
    assert!(
        !cache.is_valid(
            &fixture.document,
            &fixture.document.history.recipes[&fixture.a]
        ),
        "stale cache after a source version change"
    );
    assert_eq!(
        cache
            .plan(&fixture.document, &fixture.document.history)
            .unwrap(),
        vec![fixture.a]
    );
    // A parameter edit invalidates the entry even though the revision did not move yet.
    let mut cache = RecomputeCache::default();
    for id in fixture.document.history.execution_order().unwrap() {
        cache
            .record(&fixture.document, &fixture.document.history.recipes[&id])
            .unwrap();
    }
    let mut edited = fixture.document.clone();
    edited
        .history
        .recipes
        .get_mut(&fixture.a)
        .unwrap()
        .parameters
        .insert("diameter_mm".into(), length(19.0));
    assert!(!cache.is_valid(&edited, &edited.history.recipes[&fixture.a]));
}

/* --------------------- L3-006 preview versus committed -------------------- */

#[test]
fn preview_edits_never_touch_the_committed_document_until_one_apply() {
    let mut chain = rail_chain();
    chain
        .document
        .history
        .recipes
        .get_mut(&chain.rail)
        .unwrap()
        .parameters
        .insert("diameter_mm".into(), length(18.0));
    let committed_before = serde_json::to_vec(&chain.document).unwrap();

    let mut session = PreviewSession::begin(&chain.document);
    session
        .set_parameter(&chain.document, chain.rail, "diameter_mm", length(19.0))
        .unwrap();
    session
        .set_parameter(&chain.document, chain.rail, "diameter_mm", length(20.0))
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&chain.document).unwrap(),
        committed_before,
        "committed document and its geometry hash are unchanged"
    );
    assert_eq!(session.plan(&chain.document).unwrap(), vec![chain.rail]);
    assert_eq!(
        session.staged_recipe(chain.rail).unwrap().parameters["diameter_mm"]
            .as_length()
            .unwrap()
            .mm(),
        20.0
    );
    // The preview stores the one edited recipe, not a second document.
    let (recipes, entities, results) = session.staged_counts();
    assert_eq!((recipes, entities, results), (1, 0, 0));
    // A transient node exists only in the preview.
    let transient_entity = Entity::new("PreviewOnly", curve([0., 0., 0.], [1., 1., 0.]), None);
    let transient_output = transient_entity.id;
    session.stage_entity(transient_entity).unwrap();
    let transient = Recipe::new(
        chain.document.id,
        "PreviewOnly",
        vec![],
        vec![transient_output],
        environment(),
    );
    let transient_id = transient.id;
    session.stage_recipe(transient).unwrap();
    let (recipes, entities, results) = session.staged_counts();
    assert_eq!((recipes, entities, results), (2, 1, 0));
    assert!(!chain.document.history.recipes.contains_key(&transient_id));
    let applied = session.commit(&chain.document).unwrap();
    assert_eq!(
        applied.history.recipes[&chain.rail].parameters["diameter_mm"]
            .as_length()
            .unwrap()
            .mm(),
        20.0,
        "only the final value is applied once"
    );
    assert!(applied.history.recipes.contains_key(&transient_id));
    assert_eq!(
        serde_json::to_vec(&chain.document).unwrap(),
        committed_before
    );
}

#[test]
fn escape_discards_transient_nodes_and_a_moved_base_is_refused() {
    let chain = rail_chain();
    let mut session = PreviewSession::begin(&chain.document);
    let entity = Entity::new("Transient", curve([0., 0., 0.], [1., 1., 0.]), None);
    let out = entity.id;
    session.stage_entity(entity).unwrap();
    let transient = Recipe::new(
        chain.document.id,
        "Transient",
        vec![],
        vec![out],
        environment(),
    );
    let transient_id = transient.id;
    session.stage_recipe(transient).unwrap();
    session.discard();
    assert!(!chain.document.history.recipes.contains_key(&transient_id));

    let mut stale_base = rail_chain();
    stale_base
        .document
        .history
        .recipes
        .get_mut(&stale_base.rail)
        .unwrap()
        .parameters
        .insert("diameter_mm".into(), length(18.0));
    let session = PreviewSession::begin(&stale_base.document);
    let mut current = stale_base.document.clone();
    current.revision += 1;
    assert_eq!(
        session.commit(&current).unwrap_err(),
        Error::Stale("preview base document")
    );
}

/* ---------------------- L3-007 failed rebuild rollback -------------------- */

struct Single {
    document: Document,
    recipe: HistoryId,
    output: ObjectId,
    source: ObjectId,
}

fn single() -> Single {
    let mut document = Document::new("single");
    let source = add(&mut document, "Source", curve([0., 0., 0.], [10., 0., 0.]));
    let output = add(&mut document, "Sweep", curve([0., 0., 0.], [10., 0., 0.]));
    let mut recipe = feature(
        &document,
        "RailSweep",
        vec![(binding(&document, source, "rail"), curve_port("rail"))],
        vec![output],
    );
    recipe.parameters.insert("length_mm".into(), length(10.0));
    let id = recipe.id;
    document.history.recipes.insert(id, recipe);
    Single {
        document,
        recipe: id,
        output,
        source,
    }
}

fn sweep_producer(request: &RebuildRequest) -> Result<Vec<RebuiltOutput>> {
    let length = request
        .parameters
        .get("length_mm")
        .ok_or(Error::Missing("length parameter"))?
        .as_length()?
        .mm();
    if length <= 0.0 {
        return Err(Error::Invalid("sweep length"));
    }
    Ok(vec![RebuiltOutput {
        object: request.outputs[0],
        geometry: curve([0., 0., 0.], [length, 0., 0.]),
    }])
}

fn set_length(document: &mut Document, id: HistoryId, value: f64) {
    document
        .history
        .recipes
        .get_mut(&id)
        .unwrap()
        .parameters
        .insert("length_mm".into(), length(value));
}

#[test]
fn a_successful_rebuild_updates_geometry_published_result_and_cache() {
    let mut fixture = single();
    let mut cache = RecomputeCache::default();
    let outcome = recompute_feature(
        &mut fixture.document,
        &mut cache,
        fixture.recipe,
        sweep_producer,
    )
    .unwrap();
    let RecomputeOutcome::Rebuilt { sha256, .. } = outcome else {
        panic!("expected a rebuild, got {outcome:?}");
    };
    assert_eq!(
        fixture.document.entities[&fixture.output].geometry,
        curve([0., 0., 0.], [10., 0., 0.])
    );
    let published = &fixture.document.results[&fixture.output];
    assert_eq!(published.status, om9_core::result::ResultStatus::Current);
    assert_eq!(
        published.geometry_snapshot,
        fixture.document.entities[&fixture.output].geometry
    );
    assert_eq!(cache.entries[&fixture.recipe].sha256, sha256);
    assert!(cache.is_valid(
        &fixture.document,
        &fixture.document.history.recipes[&fixture.recipe]
    ));
    // A cached recompute does not call the producer at all.
    let cached = recompute_feature(&mut fixture.document, &mut cache, fixture.recipe, |_| {
        panic!("cache hit must not rebuild")
    })
    .unwrap();
    assert!(matches!(cached, RecomputeOutcome::Cached { sha256: hit, .. } if hit == sha256));
}

#[test]
fn a_failed_rebuild_keeps_the_last_good_geometry_result_and_cache_entry() {
    let mut fixture = single();
    let mut cache = RecomputeCache::default();
    set_length(&mut fixture.document, fixture.recipe, 20.0);
    recompute_feature(
        &mut fixture.document,
        &mut cache,
        fixture.recipe,
        sweep_producer,
    )
    .unwrap();
    let good_geometry = fixture.document.entities[&fixture.output].geometry.clone();
    let good_revision = fixture.document.entities[&fixture.output].geometry_revision;
    let good_result = fixture.document.results[&fixture.output].clone();
    let good_cache = cache.entries[&fixture.recipe].clone();

    set_length(&mut fixture.document, fixture.recipe, -1.0);
    let outcome = recompute_feature(
        &mut fixture.document,
        &mut cache,
        fixture.recipe,
        sweep_producer,
    )
    .unwrap();
    let RecomputeOutcome::Failed {
        reason,
        previous_retained,
        ..
    } = outcome
    else {
        panic!("expected a failed rebuild");
    };
    assert_eq!(reason, "invalid sweep length");
    assert!(previous_retained);
    assert_eq!(
        fixture.document.entities[&fixture.output].geometry, good_geometry,
        "previous good geometry is retained"
    );
    assert_eq!(
        fixture.document.entities[&fixture.output].geometry_revision,
        good_revision
    );
    assert_eq!(fixture.document.results[&fixture.output], good_result);
    assert_eq!(cache.entries[&fixture.recipe], good_cache);
    assert_eq!(
        fixture.document.history.recipes[&fixture.recipe].state,
        EvaluationState::Failed("invalid sweep length".into())
    );
    // A failed feature is not planned; the result is visible as an error path.
    assert!(
        plan_recompute(&fixture.document.history)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_producer_that_breaks_the_recorded_output_set_creates_no_corrupt_node() {
    let mut fixture = single();
    let mut cache = RecomputeCache::default();
    let before = fixture.document.entities[&fixture.output].clone();
    let empty = recompute_feature(&mut fixture.document, &mut cache, fixture.recipe, |_| {
        Ok(vec![])
    })
    .unwrap();
    assert!(matches!(
        empty,
        RecomputeOutcome::Failed { ref reason, .. } if reason == "invalid rebuilt output set"
    ));
    assert_eq!(fixture.document.entities[&fixture.output], before);
    // Wrong geometry kind for a recorded Curve output port.
    let wrong = recompute_feature(
        &mut fixture.document,
        &mut cache,
        fixture.recipe,
        |request| {
            Ok(vec![RebuiltOutput {
                object: request.outputs[0],
                geometry: Geometry::Point([0., 0., 0.]),
            }])
        },
    )
    .unwrap();
    assert!(matches!(
        wrong,
        RecomputeOutcome::Failed { ref reason, .. } if reason == "invalid typed feature output"
    ));
    assert_eq!(fixture.document.entities[&fixture.output], before);
    // An unresolved feature is refused, not rebuilt from a missing source.
    fixture.document.entities.remove(&fixture.source);
    mark_document_stale(&mut fixture.document).unwrap();
    assert_eq!(
        recompute_feature(
            &mut fixture.document,
            &mut cache,
            fixture.recipe,
            sweep_producer
        )
        .unwrap_err(),
        Error::Invalid("unresolved feature recompute")
    );
    // A detached feature refuses recompute too.
    let mut detached = single();
    detach(&mut detached.document.history, detached.recipe).unwrap();
    assert_eq!(
        recompute_feature(
            &mut detached.document,
            &mut RecomputeCache::default(),
            detached.recipe,
            sweep_producer
        )
        .unwrap_err(),
        Error::Invalid("detached feature recompute")
    );
}

/* ----------------- L3-008 delete parent / change input -------------------- */

fn reference_on(
    document: &mut Document,
    recipe: HistoryId,
    output: ObjectId,
) -> om9_core::identity::ReferenceId {
    bind_reference(
        &mut document.history,
        &limits(),
        recipe,
        output,
        "rail-seam",
        edge_anchor([0., 0., 0.], [10., 0., 0.]),
    )
    .unwrap()
}

#[test]
fn deleting_the_parent_marks_the_child_broken_and_reports_the_policy_choices() {
    let mut fixture = fanout();
    let child_output = fixture.document.history.recipes[&fixture.a].outputs[0];
    let child_reference = reference_on(&mut fixture.document, fixture.a, child_output);
    let report = delete_source_input(
        &mut fixture.document,
        fixture.source,
        None,
        &RebuildActivity::new(),
    )
    .unwrap();
    assert_eq!(report.source, fixture.source);
    assert!(report.policy.is_none());
    assert_eq!(report.affected.len(), 4, "A, B, C and D are affected");
    let first = report
        .affected
        .iter()
        .find(|broken| broken.recipe == fixture.a)
        .unwrap();
    assert_eq!(first.missing, vec![fixture.source]);
    assert_eq!(first.policies, BrokenInputPolicy::options().to_vec());
    assert!(!fixture.document.entities.contains_key(&fixture.source));
    assert_eq!(
        fixture.document.history.recipes[&fixture.a].state,
        EvaluationState::Unresolved("source was deleted or transferred".into())
    );
    assert!(
        fixture
            .document
            .history
            .references
            .contains_key(&child_reference)
    );
    // The reference cannot resolve while the feature is unresolved.
    let published = PublishedSubshapes {
        recipe: fixture.a,
        output: fixture.document.history.recipes[&fixture.a].outputs[0],
        revision: fixture.document.history.recipes[&fixture.a].revision,
        subshapes: vec![],
    };
    assert!(
        resolve_semantic_reference(
            &fixture.document.history,
            &fixture.document,
            &limits(),
            child_reference,
            &published
        )
        .is_err()
    );
}

#[test]
fn detach_and_remove_policies_leave_no_dangling_reference() {
    // Detach keeps the outputs as plain objects and drops the history edge.
    let mut detach = fanout();
    let detach_output = detach.document.history.recipes[&detach.b].outputs[0];
    let reference = reference_on(&mut detach.document, detach.b, detach_output);
    let report = delete_source_input(
        &mut detach.document,
        detach.source,
        Some(BrokenInputPolicy::DetachDependents),
        &RebuildActivity::new(),
    )
    .unwrap();
    assert_eq!(report.detached.len(), 4);
    for id in [detach.a, detach.b, detach.c, detach.d] {
        assert_eq!(
            detach.document.history.recipes[&id].state,
            EvaluationState::Detached
        );
    }
    assert!(detach.document.history.references.is_empty());
    assert!(!detach.document.history.references.contains_key(&reference));
    for id in [detach.a, detach.b, detach.c, detach.d] {
        for output in &detach.document.history.recipes[&id].outputs {
            assert!(detach.document.entities.contains_key(output));
        }
    }

    // Remove deletes the dependents and their outputs; no recorded edge mentions them.
    let mut remove = fanout();
    let removed_outputs: Vec<_> = [remove.a, remove.b, remove.c, remove.d]
        .into_iter()
        .flat_map(|id| remove.document.history.recipes[&id].outputs.clone())
        .collect();
    let report = delete_source_input(
        &mut remove.document,
        remove.source,
        Some(BrokenInputPolicy::RemoveDependents),
        &RebuildActivity::new(),
    )
    .unwrap();
    assert_eq!(report.removed_recipes.len(), 4);
    assert_eq!(report.removed_objects.len(), 4);
    for output in &removed_outputs {
        assert!(!remove.document.entities.contains_key(output));
        assert!(!remove.document.results.contains_key(output));
    }
    for recipe in remove.document.history.recipes.values() {
        assert!(
            recipe
                .inputs
                .iter()
                .all(|input| input.source.object != remove.source),
            "no recipe still records the deleted parent"
        );
        assert!(
            recipe
                .outputs
                .iter()
                .all(|output| !removed_outputs.contains(output))
        );
    }
    assert!(remove.document.history.references.is_empty());
}

#[test]
fn a_delete_during_an_async_rebuild_is_refused_not_raced() {
    let mut fixture = fanout();
    let mut activity = RebuildActivity::new();
    activity.begin(fixture.b).unwrap();
    assert!(activity.is_busy(fixture.b));
    assert_eq!(
        delete_source_input(&mut fixture.document, fixture.source, None, &activity).unwrap_err(),
        Error::Busy
    );
    // The refused delete changed nothing.
    assert!(fixture.document.entities.contains_key(&fixture.source));
    activity.finish(fixture.b);
    assert!(activity.is_empty());
    delete_source_input(&mut fixture.document, fixture.source, None, &activity).unwrap();
}

/* ------------------- L3-009 deterministic baselines ---------------------- */

struct RingGraph {
    document: Document,
    sweep: HistoryId,
    sweep_output: ObjectId,
}

/// `rail -> profile -> sweep` with a gem placeholder, all at one rail length.
fn ring_graph(length_mm: f64) -> RingGraph {
    let mut document = Document::new("ring");
    let out_rail = add(&mut document, "Rail", curve([0., 0., 0.], [10., 0., 0.]));
    let out_profile = add(&mut document, "Profile", curve([0., 0., 0.], [0., 5., 0.]));
    let out_sweep = add(
        &mut document,
        "Sweep",
        curve([0., 0., 0.], [length_mm, 0., 0.]),
    );
    let out_gem = add(&mut document, "Gem", Geometry::Point([0., 0., 2.]));
    let rail = feature(&document, "RingRail", vec![], vec![out_rail]);
    let profile = feature(
        &document,
        "RailProfile",
        vec![(binding(&document, out_rail, "rail"), curve_port("rail"))],
        vec![out_profile],
    );
    let sweep = feature(
        &document,
        "RailSweep",
        vec![(
            binding(&document, out_profile, "profile"),
            curve_port("profile"),
        )],
        vec![out_sweep],
    );
    let gem = feature(
        &document,
        "GemPlaceholder",
        vec![(binding(&document, out_sweep, "sweep"), curve_port("sweep"))],
        vec![out_gem],
    );
    let mut graph = HistoryGraph::default();
    for recipe in [rail, profile, sweep, gem] {
        graph.recipes.insert(recipe.id, recipe);
    }
    let sweep_id = graph
        .recipes
        .values()
        .find(|recipe| recipe.command == "RailSweep")
        .unwrap()
        .id;
    graph
        .recipes
        .get_mut(&sweep_id)
        .unwrap()
        .parameters
        .insert("length_mm".into(), length(length_mm));
    document.history = graph;
    RingGraph {
        document,
        sweep: sweep_id,
        sweep_output: out_sweep,
    }
}

#[test]
fn the_baseline_is_deterministic_across_sessions_and_round_trips() {
    let first = ring_graph(10.0);
    let second = ring_graph(10.0);
    let a = feature_graph_baseline(&first.document).unwrap();
    let b = feature_graph_baseline(&second.document).unwrap();
    assert_eq!(a.nodes.len(), 4);
    assert_eq!(a.edges.len(), 3);
    assert_eq!(
        a.sha256, b.sha256,
        "same design and parameters, same baseline"
    );
    assert_eq!(a, b);
    assert_eq!(
        a,
        feature_graph_baseline(&first.document).unwrap(),
        "stable within one document"
    );
    let bytes = serde_json::to_vec(&a).unwrap();
    assert_eq!(load_baseline(&bytes, &first.document).unwrap(), a);
}

#[test]
fn editing_a_parameter_and_rebuilding_matches_a_fresh_build() {
    let mut edited = ring_graph(10.0);
    let fresh = ring_graph(20.0);
    // The stale baseline is refused once the document changes.
    let stale_baseline = feature_graph_baseline(&edited.document).unwrap();
    edited
        .document
        .history
        .recipes
        .get_mut(&edited.sweep)
        .unwrap()
        .parameters
        .insert("length_mm".into(), length(20.0));
    assert_eq!(
        stale_baseline.verify_against(&edited.document).unwrap_err(),
        Error::Stale("feature baseline document")
    );
    // The fixture geometry must actually change with the parameter; the producer is the
    // same function a fresh build uses.
    let mut cache = RecomputeCache::default();
    let outcome = recompute_feature(
        &mut edited.document,
        &mut cache,
        edited.sweep,
        sweep_producer,
    )
    .unwrap();
    assert!(matches!(outcome, RecomputeOutcome::Rebuilt { .. }));
    assert_eq!(
        edited.document.entities[&edited.sweep_output].geometry,
        curve([0., 0., 0.], [20., 0., 0.])
    );
    assert_eq!(
        feature_graph_baseline(&edited.document).unwrap().sha256,
        feature_graph_baseline(&fresh.document).unwrap().sha256,
        "an edited rebuild equals a fresh build"
    );
}

#[test]
fn corrupt_or_missing_fixture_data_fails_to_load() {
    let fixture = ring_graph(10.0);
    let baseline = feature_graph_baseline(&fixture.document).unwrap();
    // Corrupt JSON.
    assert!(matches!(
        load_baseline(b"{ this is not a baseline", &fixture.document),
        Err(Error::Io(_))
    ));
    // A stored node order that does not match its index.
    let mut corrupt: FeatureGraphBaseline = baseline.clone();
    corrupt.nodes[0].index = 1;
    assert_eq!(
        corrupt.verify_structure().unwrap_err(),
        Error::Invalid("feature baseline node order")
    );
    // An edge that names a node outside the record.
    let mut out_of_range = baseline.clone();
    out_of_range.edges[0].to = 99;
    assert_eq!(
        out_of_range.verify_structure().unwrap_err(),
        Error::Invalid("feature baseline edge")
    );
    // A baseline whose recorded hash was edited no longer describes itself.
    let mut tampered = baseline.clone();
    tampered.sha256 = "00".repeat(32);
    assert_eq!(
        tampered.verify_structure().unwrap_err(),
        Error::Stale("feature baseline hash")
    );
}

#[test]
fn options_are_hashed_by_content_not_by_allocation() {
    // Two option maps with the same values but different insertion order serialize as the
    // same canonical BTreeMap, so a baseline never depends on edit order.
    let mut first: Options = Options::new();
    first.insert("a".into(), length(1.0));
    first.insert("b".into(), Value::Boolean(true));
    let mut second: Options = Options::new();
    second.insert("b".into(), Value::Boolean(true));
    second.insert("a".into(), length(1.0));
    assert_eq!(first, second);
}
