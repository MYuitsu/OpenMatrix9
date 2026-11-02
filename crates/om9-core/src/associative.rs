// SPDX-License-Identifier: LGPL-2.1-or-later
//! Associative dependency rebuild for the M04 builder history (`OM9-L2-01-04`).
//!
//! The durable graph lives in [`crate::history`]. This module owns the parts that make the
//! graph *associative* rather than a flat undo log:
//!
//! * typed input/output ports, so a curve can never be consumed as a millimetre scalar
//!   (`OM9-L3-01-04-001`);
//! * a command provenance view that reopens the exact command, typed inputs and unit
//!   (`OM9-L3-01-04-002`);
//! * semantic subshape references that survive an allowed rebuild and refuse a silent
//!   relink when the topology becomes ambiguous (`OM9-L3-01-04-004`);
//! * an incremental cache keyed by recipe revision and input stamps
//!   (`OM9-L3-01-04-005`);
//! * a preview session that never touches the committed document
//!   (`OM9-L3-01-04-006`);
//! * a recompute driver that keeps the last good result when a producer fails
//!   (`OM9-L3-01-04-007`);
//! * the delete-parent / change-input policy with an explicit broken-dependency report
//!   (`OM9-L3-01-04-008`);
//! * deterministic baseline fixtures for ring/profile/sweep rebuild regression
//!   (`OM9-L3-01-04-009`).
//!
//! Dirty propagation and topological ordering stay in [`crate::history_ops`]; this module
//! calls them instead of re-deriving the order.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, GeometryKind, Point3, Transform, Unit, finite},
    history::{
        EvaluationState, HistoryGraph, Recipe, ReplayEnvironment, Tolerance, within_tolerance,
    },
    identity::*,
    model::{Document, Entity},
    request::{Dimension, Options, Selection, SourceBinding, Value},
    result::{ConversionReport, PublishedResult, ResultStatus},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/* ------------------------------- typed ports ------------------------------- */

/// The declared type of one feature port. `Geometry` admits only the listed
/// [`GeometryKind`]s; a scalar port is a *parameter*, never an object binding, so binding
/// an entity to it is refused instead of being coerced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PortKind {
    Geometry {
        kinds: Vec<GeometryKind>,
    },
    Length,
    Angle,
    Ratio,
    Count,
    Text,
    /// A recipe recorded before typed ports existed. Reported, never silently
    /// reinterpreted as a concrete type.
    Untyped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeaturePort {
    pub role: String,
    pub kind: PortKind,
}

impl FeaturePort {
    pub fn geometry(role: impl Into<String>, kinds: &[GeometryKind]) -> Self {
        Self {
            role: role.into(),
            kind: PortKind::Geometry {
                kinds: kinds.to_vec(),
            },
        }
    }
    pub fn length(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            kind: PortKind::Length,
        }
    }
    pub fn angle(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            kind: PortKind::Angle,
        }
    }
    pub fn scalar(role: impl Into<String>, dimension: Dimension) -> Self {
        Self {
            role: role.into(),
            kind: match dimension {
                Dimension::Length => PortKind::Length,
                Dimension::Angle => PortKind::Angle,
                Dimension::Ratio => PortKind::Ratio,
                Dimension::Count => PortKind::Count,
            },
        }
    }
    pub fn text(role: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            kind: PortKind::Text,
        }
    }
    /// The port itself must be well formed before anything is bound to it.
    pub fn validate(&self, l: &Limits) -> Result<()> {
        name(&self.role, l)?;
        if self.role.is_empty() {
            return Err(Error::Invalid("feature port role"));
        }
        if let PortKind::Geometry { kinds } = &self.kind {
            let mut seen = BTreeSet::new();
            if kinds.is_empty() {
                return Err(Error::Invalid("feature port kinds"));
            }
            for kind in kinds {
                if !seen.insert(*kind) {
                    return Err(Error::Invalid("feature port kinds"));
                }
            }
        }
        Ok(())
    }
    pub(crate) fn usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage {
            bytes: self.role.capacity() as u64 + std::mem::size_of::<Self>() as u64,
            ..Usage::default()
        };
        if let PortKind::Geometry { kinds } = &self.kind {
            u.add(Usage::array::<GeometryKind>(kinds.capacity())?, l)?;
        }
        Ok(u)
    }
    /// An *object* binding is legal only for a geometry port of an accepted kind.
    pub fn admits_entity(&self, entity: &Entity) -> Result<()> {
        match &self.kind {
            PortKind::Geometry { kinds } => {
                if kinds.contains(&entity.geometry.kind()) {
                    Ok(())
                } else {
                    Err(Error::Invalid("typed feature input"))
                }
            }
            PortKind::Untyped => Ok(()),
            // A scalar parameter is not an object: this is exactly the "CurveRef into
            // ScalarMm" refusal, not an implicit conversion.
            _ => Err(Error::Invalid("object bound to scalar input")),
        }
    }
    /// An output port declares the geometry kind the feature produces.
    pub fn admits_geometry(&self, kind: GeometryKind) -> Result<()> {
        match &self.kind {
            PortKind::Geometry { kinds } => {
                if kinds.contains(&kind) {
                    Ok(())
                } else {
                    Err(Error::Invalid("typed feature output"))
                }
            }
            PortKind::Untyped => Ok(()),
            _ => Err(Error::Invalid("scalar feature output")),
        }
    }
}

fn scalar_port_ok(kind: &PortKind) -> bool {
    !matches!(kind, PortKind::Geometry { .. } | PortKind::Untyped)
}

/// The independent storage cost of a recipe's ports. Validation and the document's
/// storage accounting must agree byte for byte (`Document::validate` asserts it), so both
/// sides call this one function.
pub(crate) fn ports_usage(recipe: &Recipe, l: &Limits) -> Result<Usage> {
    let mut u = Usage::default();
    if recipe.input_ports.is_empty() && recipe.output_ports.is_empty() {
        return Ok(u);
    }
    u.add(
        Usage::array::<FeaturePort>(recipe.input_ports.capacity())?,
        l,
    )?;
    u.add(
        Usage::array::<FeaturePort>(recipe.output_ports.capacity())?,
        l,
    )?;
    for port in recipe.input_ports.iter().chain(&recipe.output_ports) {
        port.validate(l)?;
        u.add(port.usage(l)?, l)?;
    }
    for port in &recipe.output_ports {
        if scalar_port_ok(&port.kind) {
            return Err(Error::Invalid("scalar feature output"));
        }
    }
    Ok(u)
}

/// Validate a recipe's recorded ports against its bindings, its outputs and (for a clean
/// recipe) the current source entities. Called by [`Recipe::validate`], so every document
/// admission re-checks the types instead of trusting the stored declaration.
pub(crate) fn validate_typed_ports(recipe: &Recipe, d: &Document, l: &Limits) -> Result<Usage> {
    validate_typed_ports_with_pending(recipe, d, l, &BTreeSet::new())
}

/// As [`validate_typed_ports`], but ids the same transaction is about to insert are accepted
/// as outputs; their type is checked by the staged document validation after the insert.
pub(crate) fn validate_typed_ports_with_pending(
    recipe: &Recipe,
    d: &Document,
    l: &Limits,
    pending: &BTreeSet<ObjectId>,
) -> Result<Usage> {
    let u = ports_usage(recipe, l)?;
    if recipe.input_ports.is_empty() && recipe.output_ports.is_empty() {
        return Ok(u);
    }
    if recipe.input_ports.len() != recipe.inputs.len() {
        return Err(Error::Invalid("typed input port count"));
    }
    if recipe.output_ports.len() != recipe.outputs.len() {
        return Err(Error::Invalid("typed output port count"));
    }
    let mut roles = BTreeSet::new();
    for (port, input) in recipe.input_ports.iter().zip(&recipe.inputs) {
        if port.role != input.role || !roles.insert(port.role.clone()) {
            return Err(Error::Invalid("typed input port role"));
        }
    }
    if recipe.state == EvaluationState::Clean {
        for (port, input) in recipe.input_ports.iter().zip(&recipe.inputs) {
            port.admits_entity(d.resolve(&input.source)?)?;
        }
        for (port, output) in recipe.output_ports.iter().zip(&recipe.outputs) {
            if pending.contains(output) {
                continue;
            }
            let entity = d
                .entities
                .get(output)
                .ok_or(Error::Missing("clean recipe output"))?;
            port.admits_geometry(entity.geometry.kind())?;
        }
    }
    Ok(u)
}

/// Record a fully typed feature in the graph. The outputs must already exist in the
/// document and no other *live* recipe may own them; this is the same ownership rule the
/// replay path applies, and it keeps a rebuild from silently stealing another feature's
/// result.
pub fn record_feature(
    document: &Document,
    graph: &mut HistoryGraph,
    recipe: Recipe,
    l: &Limits,
) -> Result<HistoryId> {
    if recipe.document != document.id {
        return Err(Error::Invalid("feature record document"));
    }
    if graph.recipes.contains_key(&recipe.id) {
        return Err(Error::Invalid("duplicate feature record"));
    }
    for output in &recipe.outputs {
        if graph.owner(*output).is_some() {
            return Err(Error::Invalid("feature output already owned"));
        }
    }
    recipe.validate(document, l)?;
    let id = recipe.id;
    graph.recipes.insert(id, recipe);
    Ok(id)
}

/// One typed edge of the recorded graph: `consumer` reads `role` of `source`, produced by
/// `producer` when the source is itself a feature output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub producer: Option<HistoryId>,
    pub consumer: HistoryId,
    pub role: String,
    pub kind: PortKind,
    pub source: ObjectId,
}

/// Every recorded edge in execution order. Roles and ports are validated first, so a
/// document whose types no longer match its bindings is refused here rather than
/// producing a misleading edge list.
pub fn typed_edges(
    document: &Document,
    graph: &HistoryGraph,
    l: &Limits,
) -> Result<Vec<DependencyEdge>> {
    let order = graph.execution_order()?;
    let mut owners = BTreeMap::new();
    for recipe in graph
        .recipes
        .values()
        .filter(|recipe| recipe.state != EvaluationState::Detached)
    {
        for output in &recipe.outputs {
            owners.insert(*output, recipe.id);
        }
    }
    let mut edges = Vec::new();
    for id in order {
        let recipe = &graph.recipes[&id];
        validate_typed_ports(recipe, document, l)?;
        for (index, input) in recipe.inputs.iter().enumerate() {
            let kind = recipe
                .input_ports
                .get(index)
                .map(|port| port.kind.clone())
                .unwrap_or(PortKind::Untyped);
            if let PortKind::Geometry { .. } = kind {
                // A typed geometry edge must still resolve: a deleted source is a broken
                // edge, not silently dropped from the report.
                let entity = document.resolve(&input.source)?;
                recipe.input_ports[index].admits_entity(entity)?;
            }
            edges.push(DependencyEdge {
                producer: owners.get(&input.source.object).copied(),
                consumer: id,
                role: input.role.clone(),
                kind,
                source: input.source.object,
            });
        }
    }
    Ok(edges)
}

/* ----------------------------- feature provenance ---------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct RecordedInput {
    pub role: String,
    pub kind: PortKind,
    pub stamp: SourceStamp,
    pub selection: Selection,
}

/// The recorded answer to "which command made this, with which parameters and inputs".
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureProvenance {
    pub recipe: HistoryId,
    pub document: DocumentId,
    pub command: String,
    pub command_version: u32,
    pub schema_version: u32,
    pub solver: String,
    pub solver_version: String,
    pub unit: Unit,
    pub frame: Transform,
    pub tolerance: Tolerance,
    pub parameters: Options,
    pub inputs: Vec<RecordedInput>,
    pub outputs: Vec<ObjectId>,
    pub revision: u64,
}

/// The exact request a "re-open this feature" action must submit again.
#[derive(Debug, Clone, PartialEq)]
pub struct ReopenFeature {
    pub command: String,
    pub parameters: Options,
    pub inputs: Vec<SourceBinding>,
    pub unit: Unit,
}

fn typed_parameter(value: &Value) -> bool {
    match value {
        // A recorded parameter without a unit cannot be replayed in millimetres: it is
        // refused at provenance time instead of being guessed on reopen.
        Value::Number(_) => false,
        Value::Quantity { value, .. } => value.is_finite(),
        Value::Choice(text) => !text.is_empty(),
        Value::Text(text) => !text.contains('\0'),
        Value::Boolean(_) => true,
    }
}

/// Read the provenance of one recipe. This validates the recorded payload, so a feature
/// that could not be reopened exactly is reported as invalid now rather than at reopen.
pub fn feature_provenance(recipe: &Recipe) -> Result<FeatureProvenance> {
    if recipe.command.is_empty()
        || recipe.environment.command_version == 0
        || recipe.environment.schema_version == 0
    {
        return Err(Error::Invalid("feature command provenance"));
    }
    if recipe.environment.length_unit != Unit::Millimeter {
        return Err(Error::Invalid("feature canonical unit"));
    }
    recipe.environment.frame.validate()?;
    recipe.environment.tolerance.validate()?;
    let mut roles = BTreeSet::new();
    let mut inputs = Vec::new();
    for (index, binding) in recipe.inputs.iter().enumerate() {
        if binding.role.is_empty() || !roles.insert(binding.role.clone()) {
            return Err(Error::Invalid("feature input role"));
        }
        // A recorded input that names no object cannot be reopened: this is the
        // "payload missing required input id" refusal.
        if binding.source.object.is_nil() {
            return Err(Error::Invalid("feature input identity"));
        }
        match &binding.selection {
            Selection::Indices(indices) if indices.is_empty() => {
                return Err(Error::Invalid("feature input selection"));
            }
            Selection::Spans(spans) if spans.is_empty() => {
                return Err(Error::Invalid("feature input selection"));
            }
            Selection::Member(members) if members.is_empty() => {
                return Err(Error::Invalid("feature input selection"));
            }
            _ => {}
        }
        inputs.push(RecordedInput {
            role: binding.role.clone(),
            kind: recipe
                .input_ports
                .get(index)
                .map(|port| port.kind.clone())
                .unwrap_or(PortKind::Untyped),
            stamp: binding.source,
            selection: binding.selection.clone(),
        });
    }
    for value in recipe.parameters.values() {
        if !typed_parameter(value) {
            return Err(Error::Invalid("feature parameter unit/type"));
        }
    }
    Ok(FeatureProvenance {
        recipe: recipe.id,
        document: recipe.document,
        command: recipe.command.clone(),
        command_version: recipe.environment.command_version,
        schema_version: recipe.environment.schema_version,
        solver: recipe.environment.solver.clone(),
        solver_version: recipe.environment.solver_version.clone(),
        unit: recipe.environment.length_unit,
        frame: recipe.environment.frame,
        tolerance: recipe.environment.tolerance.clone(),
        parameters: recipe.parameters.clone(),
        inputs,
        outputs: recipe.outputs.clone(),
        revision: recipe.revision,
    })
}

/// Reopen the exact command request. Parameters and inputs are copied, never
/// re-derived; the unit is the recorded canonical millimetre.
pub fn reopen_feature(provenance: &FeatureProvenance) -> Result<ReopenFeature> {
    // A builder feature may legitimately have no object input (a rail built from
    // parameters alone), so only the command and the unit are required here.
    if provenance.command.is_empty() {
        return Err(Error::Invalid("feature reopen payload"));
    }
    if provenance.unit != Unit::Millimeter {
        return Err(Error::Invalid("feature reopen unit"));
    }
    let mut roles = BTreeSet::new();
    let mut inputs = Vec::new();
    for input in &provenance.inputs {
        if input.role.is_empty() || !roles.insert(input.role.clone()) {
            return Err(Error::Invalid("feature reopen role"));
        }
        inputs.push(SourceBinding {
            role: input.role.clone(),
            source: input.stamp,
            selection: input.selection.clone(),
        });
    }
    for value in provenance.parameters.values() {
        if !typed_parameter(value) {
            return Err(Error::Invalid("feature reopen parameter unit/type"));
        }
    }
    Ok(ReopenFeature {
        command: provenance.command.clone(),
        parameters: provenance.parameters.clone(),
        inputs,
        unit: provenance.unit,
    })
}

/// A replay environment is the durable half of provenance; a caller that only has the
/// command request still needs the recorded frame/tolerance/versions to reopen it.
pub fn replay_environment(provenance: &FeatureProvenance) -> ReplayEnvironment {
    ReplayEnvironment {
        command_version: provenance.command_version,
        solver: provenance.solver.clone(),
        solver_version: provenance.solver_version.clone(),
        schema_version: provenance.schema_version,
        length_unit: provenance.unit,
        frame: provenance.frame,
        tolerance: provenance.tolerance.clone(),
    }
}

/* -------------------------- semantic reference links ------------------------ */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubshapeKind {
    Edge,
    Face,
    Vertex,
    Curve,
}

/// A geometric fingerprint of one subshape. The anchors plus a scalar measure are enough
/// to tell "the same intended edge after a legal edit" from "a different edge that
/// happens to carry the same name".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubshapeSignature {
    pub start: Point3,
    pub end: Point3,
    pub measure_mm: f64,
}

impl SubshapeSignature {
    pub fn validate(&self) -> Result<()> {
        finite(self.start)?;
        finite(self.end)?;
        if !self.measure_mm.is_finite() || self.measure_mm < 0. {
            return Err(Error::Invalid("subshape signature measure"));
        }
        Ok(())
    }
    /// The largest coordinate/measure difference between two fingerprints.
    pub fn deviation_from(&self, other: &Self) -> f64 {
        let mut worst: f64 = 0.;
        for (a, b) in self
            .start
            .iter()
            .zip(other.start.iter())
            .chain(self.end.iter().zip(other.end.iter()))
        {
            worst = worst.max((a - b).abs());
        }
        worst.max((self.measure_mm - other.measure_mm).abs())
    }
    pub(crate) fn usage(&self) -> Usage {
        Usage {
            bytes: std::mem::size_of::<Self>() as u64,
            ..Usage::default()
        }
    }
}

/// A named subshape published by a producer for one output revision.
#[derive(Debug, Clone, PartialEq)]
pub struct Subshape {
    pub name: String,
    pub kind: SubshapeKind,
    pub signature: SubshapeSignature,
}

/// The producer's answer for one rebuilt output: the semantic names it still carries.
#[derive(Debug, Clone, PartialEq)]
pub struct PublishedSubshapes {
    pub recipe: HistoryId,
    pub output: ObjectId,
    pub revision: u64,
    pub subshapes: Vec<Subshape>,
}

/// A durable reference to one named subshape of a feature output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticReference {
    pub id: ReferenceId,
    pub recipe: HistoryId,
    pub output: ObjectId,
    pub role: String,
    pub kind: SubshapeKind,
    pub fingerprint: SubshapeSignature,
    pub tolerance_mm: f64,
}

impl SemanticReference {
    pub fn validate(&self, l: &Limits) -> Result<()> {
        name(&self.role, l)?;
        if self.id.is_nil()
            || self.role.is_empty()
            || self.output.is_nil()
            || !self.tolerance_mm.is_finite()
            || self.tolerance_mm < 0.
        {
            return Err(Error::Invalid("semantic reference identity"));
        }
        self.fingerprint.validate()
    }
}

pub(crate) fn references_usage(
    references: &BTreeMap<ReferenceId, SemanticReference>,
    l: &Limits,
) -> Result<Usage> {
    // An empty table is never serialised and owns no node, so it charges nothing;
    // this matches `Document::storage_usage_internal`, which skips every empty
    // optional table for the same reason.
    if references.is_empty() {
        return Ok(Usage::default());
    }
    let mut u = Usage::btree::<ReferenceId, SemanticReference>(references.len())?;
    for (id, reference) in references {
        if *id != reference.id {
            return Err(Error::Invalid("semantic reference map key"));
        }
        reference.validate(l)?;
        u.add(
            Usage {
                bytes: reference.role.capacity() as u64
                    + std::mem::size_of::<SemanticReference>() as u64,
                ..Usage::default()
            },
            l,
        )?;
        u.add(reference.fingerprint.usage(), l)?;
    }
    Ok(u)
}

/// The three values that describe how strictly a reference is matched: the subshape kind,
/// the expected fingerprint and the tolerance the fingerprint may drift within.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReferenceAnchor {
    pub kind: SubshapeKind,
    pub fingerprint: SubshapeSignature,
    pub tolerance_mm: f64,
}

/// Bind a semantic reference to one named subshape of a live recipe output.
pub fn bind_reference(
    graph: &mut HistoryGraph,
    l: &Limits,
    recipe: HistoryId,
    output: ObjectId,
    role: impl Into<String>,
    anchor: ReferenceAnchor,
) -> Result<ReferenceId> {
    let role = role.into();
    let target = graph
        .recipes
        .get(&recipe)
        .ok_or(Error::Missing("semantic reference feature"))?;
    if !target.outputs.contains(&output) {
        return Err(Error::Invalid("semantic reference output"));
    }
    if target.state == EvaluationState::Detached {
        return Err(Error::Invalid("semantic reference detached feature"));
    }
    let reference = SemanticReference {
        id: ReferenceId::new(),
        recipe,
        output,
        role,
        kind: anchor.kind,
        fingerprint: anchor.fingerprint,
        tolerance_mm: anchor.tolerance_mm,
    };
    reference.validate(l)?;
    if graph.references.values().any(|existing| {
        existing.recipe == recipe && existing.output == output && existing.role == reference.role
    }) {
        return Err(Error::Invalid("duplicate semantic reference"));
    }
    let reference_id = reference.id;
    graph.references.insert(reference_id, reference);
    Ok(reference_id)
}

/// Resolve one reference against the subshapes a producer published for the same output
/// revision. A missing name, a duplicated name, a changed kind or a fingerprint outside
/// the recorded tolerance are different refusals; none of them silently relinks.
pub fn resolve_semantic_reference(
    graph: &HistoryGraph,
    document: &Document,
    l: &Limits,
    id: ReferenceId,
    published: &PublishedSubshapes,
) -> Result<Subshape> {
    let reference = graph
        .references
        .get(&id)
        .ok_or(Error::Missing("semantic reference"))?;
    reference.validate(l)?;
    if published.recipe != reference.recipe || published.output != reference.output {
        return Err(Error::Stale("semantic reference output"));
    }
    let recipe = graph
        .recipes
        .get(&reference.recipe)
        .ok_or(Error::Missing("semantic reference feature"))?;
    if recipe.state == EvaluationState::Detached {
        return Err(Error::Invalid("semantic reference detached feature"));
    }
    if recipe.state != EvaluationState::Clean {
        return Err(Error::Stale("semantic reference feature state"));
    }
    if recipe.revision != published.revision {
        return Err(Error::Stale("semantic reference revision"));
    }
    // The output must still be the entity the reference was bound to; a reissued identity
    // is not the same subshape even when the name matches.
    if !document.entities.contains_key(&reference.output) {
        return Err(Error::Missing("semantic reference entity"));
    }
    let mut matches = published
        .subshapes
        .iter()
        .filter(|subshape| subshape.name == reference.role);
    let Some(candidate) = matches.next() else {
        return Err(Error::Missing("semantic subshape"));
    };
    if matches.next().is_some() {
        return Err(Error::Invalid("ambiguous semantic reference"));
    }
    if candidate.kind != reference.kind {
        return Err(Error::Invalid("semantic subshape kind"));
    }
    candidate.signature.validate()?;
    if !within_tolerance(
        candidate.signature.deviation_from(&reference.fingerprint),
        0.,
        reference.tolerance_mm,
    ) {
        return Err(Error::Stale("semantic reference moved"));
    }
    Ok(candidate.clone())
}

/// Accept a user-approved move of a reference (explicit relink). This is the only way a
/// fingerprint changes; resolution itself never writes.
pub fn relink_reference(
    graph: &mut HistoryGraph,
    l: &Limits,
    id: ReferenceId,
    fingerprint: SubshapeSignature,
) -> Result<()> {
    fingerprint.validate()?;
    let reference = graph
        .references
        .get_mut(&id)
        .ok_or(Error::Missing("semantic reference"))?;
    reference.fingerprint = fingerprint;
    reference.validate(l)
}

/// Drop every reference owned by one recipe. Callers use it when a feature is detached,
/// removed or loses the output the reference pointed at.
pub fn drop_references_of(graph: &mut HistoryGraph, recipe: HistoryId) -> usize {
    let before = graph.references.len();
    graph
        .references
        .retain(|_, reference| reference.recipe != recipe);
    before - graph.references.len()
}

/* ------------------------- incremental recompute cache --------------------- */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheEntry {
    pub recipe: HistoryId,
    pub revision: u64,
    pub parameters: Options,
    pub inputs: Vec<SourceStamp>,
    pub outputs: Vec<ObjectId>,
    pub sha256: [u8; 32],
}

/// Content hash of a feature's outputs. Only geometry is hashed, in output order, so two
/// rebuilds that produce the same shapes agree and an unchanged output keeps its hash.
pub fn output_hash(document: &Document, outputs: &[ObjectId]) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    for id in outputs {
        let entity = document
            .entities
            .get(id)
            .ok_or(Error::Missing("cache output"))?;
        let bytes = serde_json::to_vec(&entity.geometry)
            .map_err(|_| Error::Invalid("cache geometry encoding"))?;
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    Ok(out)
}

/// Incremental cache of feature results. An entry is valid only for the exact recipe
/// revision, the exact recorded input stamps and outputs that still resolve; anything
/// else is a cache miss, never a stale hit.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecomputeCache {
    #[serde(default)]
    pub entries: BTreeMap<HistoryId, CacheEntry>,
}

impl RecomputeCache {
    pub fn is_valid(&self, document: &Document, recipe: &Recipe) -> bool {
        let Some(entry) = self.entries.get(&recipe.id) else {
            return false;
        };
        if recipe.state != EvaluationState::Clean
            || entry.revision != recipe.revision
            || entry.parameters != recipe.parameters
            || entry.outputs != recipe.outputs
        {
            return false;
        }
        let stamps: Vec<_> = recipe.inputs.iter().map(|input| input.source).collect();
        if entry.inputs != stamps {
            return false;
        }
        // A source that changed after the cached build invalidates the entry even before
        // the graph is marked dirty.
        recipe
            .inputs
            .iter()
            .all(|input| document.resolve(&input.source).is_ok())
            && entry
                .outputs
                .iter()
                .all(|id| document.entities.contains_key(id))
    }
    /// Record a successful build. Fails when an output disappeared, so the cache can never
    /// claim a result the document does not hold.
    pub fn record(&mut self, document: &Document, recipe: &Recipe) -> Result<[u8; 32]> {
        if recipe.state != EvaluationState::Clean {
            return Err(Error::Invalid("cache record of a dirty feature"));
        }
        let sha256 = output_hash(document, &recipe.outputs)?;
        self.entries.insert(
            recipe.id,
            CacheEntry {
                recipe: recipe.id,
                revision: recipe.revision,
                parameters: recipe.parameters.clone(),
                inputs: recipe.inputs.iter().map(|input| input.source).collect(),
                outputs: recipe.outputs.clone(),
                sha256,
            },
        );
        Ok(sha256)
    }
    pub fn invalidate(&mut self, recipe: HistoryId) -> bool {
        self.entries.remove(&recipe).is_some()
    }
    /// What must run now: recipes the graph marks dirty/pending, plus a clean recipe whose
    /// cache entry is missing or stale. Independent clean features stay out.
    pub fn plan(&self, document: &Document, graph: &HistoryGraph) -> Result<Vec<HistoryId>> {
        let order = graph.execution_order()?;
        let mut plan = Vec::new();
        for id in order {
            let recipe = &graph.recipes[&id];
            let runnable = match recipe.state {
                EvaluationState::Dirty(_) | EvaluationState::Pending => true,
                EvaluationState::Clean => !self.is_valid(document, recipe),
                _ => false,
            };
            if runnable {
                plan.push(id);
            }
        }
        Ok(plan)
    }
}

/* ----------------------- preview versus committed graph -------------------- */

/// A builder preview that stores only the edits, not a second document.
///
/// `begin` copies the base identity/revision and nothing else. A parameter edit stages one
/// recipe clone; preview geometry stages one entity; a transient feature stages one record.
/// The committed document is only replaced by the single [`PreviewSession::commit`], which
/// refuses if the base moved while the preview was open.
#[derive(Debug, Clone)]
pub struct PreviewSession {
    document: DocumentId,
    base_revision: u64,
    base_epoch: u64,
    base_session_generation: u64,
    base_context_revision: u64,
    recipes: BTreeMap<HistoryId, Recipe>,
    entities: BTreeMap<ObjectId, Entity>,
    results: BTreeMap<ObjectId, PublishedResult>,
}

impl PreviewSession {
    pub fn begin(document: &Document) -> Self {
        Self {
            document: document.id,
            base_revision: document.revision,
            base_epoch: document.epoch,
            base_session_generation: document.session_generation,
            base_context_revision: document.context_revision,
            recipes: BTreeMap::new(),
            entities: BTreeMap::new(),
            results: BTreeMap::new(),
        }
    }
    pub fn base_revision(&self) -> u64 {
        self.base_revision
    }
    /// (staged recipes, staged entities, staged results). A preview of one parameter in a
    /// large document stores one recipe and no geometry.
    pub fn staged_counts(&self) -> (usize, usize, usize) {
        (self.recipes.len(), self.entities.len(), self.results.len())
    }
    pub fn staged_recipe(&self, recipe: HistoryId) -> Option<&Recipe> {
        self.recipes.get(&recipe)
    }
    pub fn staged_entity(&self, object: ObjectId) -> Option<&Entity> {
        self.entities.get(&object)
    }
    /// Edit one recorded parameter. The base recipe is cloned once into the staged map.
    pub fn set_parameter(
        &mut self,
        document: &Document,
        recipe: HistoryId,
        key: impl Into<String>,
        value: Value,
    ) -> Result<()> {
        if document.id != self.document {
            return Err(Error::Invalid("preview document"));
        }
        let key = key.into();
        if let std::collections::btree_map::Entry::Vacant(entry) = self.recipes.entry(recipe) {
            let base = document
                .history
                .recipes
                .get(&recipe)
                .ok_or(Error::Missing("preview feature"))?
                .clone();
            entry.insert(base);
        }
        let target = self
            .recipes
            .get_mut(&recipe)
            .ok_or(Error::Missing("preview feature"))?;
        if target.state == EvaluationState::Detached {
            return Err(Error::Invalid("detached preview feature"));
        }
        target.parameters.insert(key, value);
        target.state = EvaluationState::Dirty("preview parameter edit".into());
        Ok(())
    }
    /// Stage a transient recipe that exists only for this preview.
    pub fn stage_recipe(&mut self, recipe: Recipe) -> Result<HistoryId> {
        if recipe.document != self.document {
            return Err(Error::Invalid("preview feature document"));
        }
        if self.recipes.contains_key(&recipe.id) {
            return Err(Error::Invalid("duplicate preview feature"));
        }
        let id = recipe.id;
        self.recipes.insert(id, recipe);
        Ok(id)
    }
    /// Stage preview geometry for one object.
    pub fn stage_entity(&mut self, entity: Entity) -> Result<()> {
        if self.entities.contains_key(&entity.id) {
            return Err(Error::Invalid("duplicate preview entity"));
        }
        self.entities.insert(entity.id, entity);
        Ok(())
    }
    /// Stage the published result that belongs to a staged entity.
    pub fn stage_result(&mut self, object: ObjectId, result: PublishedResult) -> Result<()> {
        if self.results.contains_key(&object) {
            return Err(Error::Invalid("duplicate preview result"));
        }
        self.results.insert(object, result);
        Ok(())
    }
    /// What would recompute if this preview were applied now. The base graph is read from
    /// `document`; only staged recipes override a state.
    pub fn plan(&self, document: &Document) -> Result<Vec<HistoryId>> {
        if document.id != self.document {
            return Err(Error::Invalid("preview document"));
        }
        let order = document.history.execution_order()?;
        let mut plan = Vec::new();
        for id in &order {
            let state = self
                .recipes
                .get(id)
                .map(|recipe| &recipe.state)
                .unwrap_or(&document.history.recipes[id].state);
            if matches!(state, EvaluationState::Dirty(_) | EvaluationState::Pending) {
                plan.push(*id);
            }
        }
        // A transient feature is not in the base execution order; report it after the base
        // features in id order so the caller sees everything it staged.
        for (id, recipe) in &self.recipes {
            if !document.history.recipes.contains_key(id)
                && matches!(
                    recipe.state,
                    EvaluationState::Dirty(_) | EvaluationState::Pending
                )
            {
                plan.push(*id);
            }
        }
        Ok(plan)
    }
    /// Apply the preview once. The caller passes the still-current committed document so a
    /// preview started before someone else's edit is refused instead of overwriting it.
    pub fn commit(self, current: &Document) -> Result<Document> {
        if current.id != self.document
            || current.revision != self.base_revision
            || current.epoch != self.base_epoch
            || current.session_generation != self.base_session_generation
            || current.context_revision != self.base_context_revision
        {
            return Err(Error::Stale("preview base document"));
        }
        // One clone at commit (the caller's document), then only the staged deltas.
        let mut applied = current.clone();
        for (id, recipe) in self.recipes {
            applied.history.recipes.insert(id, recipe);
        }
        for (id, entity) in self.entities {
            applied.entities.insert(id, entity);
        }
        for (id, result) in self.results {
            applied.results.insert(id, result);
        }
        Ok(applied)
    }
    /// Escape: the staged edits are dropped, the committed document was never touched.
    pub fn discard(self) {}
}

/* ----------------------- recompute with failure rollback ------------------- */

/// One resolved input handed to a producer.
#[derive(Debug, Clone, PartialEq)]
pub struct InputSnapshot {
    pub role: String,
    pub object: ObjectId,
    pub entity: Entity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildRequest {
    pub recipe: HistoryId,
    pub command: String,
    pub document: DocumentId,
    pub parameters: Options,
    pub inputs: Vec<InputSnapshot>,
    pub outputs: Vec<ObjectId>,
}

/// One rebuilt output. The producer names the object, never a new identity: a rebuild
/// updates a feature's existing result.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuiltOutput {
    pub object: ObjectId,
    pub geometry: Geometry,
}

/// The semantic names one producer publishes for one rebuilt output. The revision is
/// stamped by [`recompute_with`] once the new recipe revision is known.
#[derive(Debug, Clone, PartialEq)]
pub struct ProducedSubshapes {
    pub output: ObjectId,
    pub subshapes: Vec<Subshape>,
}

/// What a feature producer returns: the rebuilt geometry and the named subshapes a
/// semantic reference may resolve against.
#[derive(Debug, Clone, PartialEq)]
pub struct ProducedFeature {
    pub outputs: Vec<RebuiltOutput>,
    pub subshapes: Vec<ProducedSubshapes>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecomputeOutcome {
    Rebuilt {
        recipe: HistoryId,
        outputs: Vec<ObjectId>,
        sha256: [u8; 32],
        /// The published names for the new revision, so the caller can resolve (or reject)
        /// semantic references without publishing a second time.
        subshapes: Vec<PublishedSubshapes>,
    },
    Cached {
        recipe: HistoryId,
        sha256: [u8; 32],
    },
    Failed {
        recipe: HistoryId,
        reason: String,
        previous_retained: bool,
    },
}

/// The snapshot a failed rebuild must leave untouched: the recipe record and every output
/// with its published result.
struct FeatureSnapshot {
    recipe: Recipe,
    outputs: Vec<(ObjectId, Entity, Option<PublishedResult>)>,
}

impl FeatureSnapshot {
    fn capture(document: &Document, recipe: &Recipe) -> Result<Self> {
        let mut outputs = Vec::new();
        for id in &recipe.outputs {
            let entity = document
                .entities
                .get(id)
                .ok_or(Error::Missing("feature output"))?
                .clone();
            outputs.push((*id, entity, document.results.get(id).cloned()));
        }
        Ok(Self {
            recipe: recipe.clone(),
            outputs,
        })
    }
    fn restore(self, document: &mut Document) {
        document.history.recipes.insert(self.recipe.id, self.recipe);
        for (id, entity, published) in self.outputs {
            document.entities.insert(id, entity);
            match published {
                Some(published) => {
                    document.results.insert(id, published);
                }
                None => {
                    document.results.remove(&id);
                }
            }
        }
    }
}

fn rebuild_request(document: &Document, recipe: &Recipe) -> Result<RebuildRequest> {
    let mut inputs = Vec::new();
    for binding in &recipe.inputs {
        // A recompute reads the source *now*: the recorded stamp is the reason the recipe is
        // out of date, so requiring it to match would make a rebuild impossible. A deleted
        // source is caught by the `Unresolved` state check before this point; an unmarked
        // deletion is still a missing object and is refused here.
        let entity = document
            .entities
            .get(&binding.source.object)
            .ok_or(Error::Missing("feature input"))?;
        inputs.push(InputSnapshot {
            role: binding.role.clone(),
            object: binding.source.object,
            entity: entity.clone(),
        });
    }
    Ok(RebuildRequest {
        recipe: recipe.id,
        command: recipe.command.clone(),
        document: recipe.document,
        parameters: recipe.parameters.clone(),
        inputs,
        outputs: recipe.outputs.clone(),
    })
}

/// Recompute one feature in place with a geometry-only producer. A producer failure is
/// recorded as `Failed(reason)` and the previous geometry, published result and cache entry
/// are kept, so a failed operation never leaves a corrupt committed node behind.
pub fn recompute_feature(
    document: &mut Document,
    cache: &mut RecomputeCache,
    id: HistoryId,
    producer: impl FnOnce(&RebuildRequest) -> Result<Vec<RebuiltOutput>>,
) -> Result<RecomputeOutcome> {
    recompute_with(document, cache, id, |request| {
        Ok(ProducedFeature {
            outputs: producer(request)?,
            subshapes: vec![],
        })
    })
}

/// Recompute one feature and carry the producer's semantic subshapes into the result.
///
/// A reference owned by this feature is checked against the produced subshapes *before*
/// the new geometry is applied. A missing, duplicated, mistyped or moved name fails the
/// rebuild and restores the snapshot: a rebuild never silently relinks a reference.
pub(crate) fn recompute_with(
    document: &mut Document,
    cache: &mut RecomputeCache,
    id: HistoryId,
    producer: impl FnOnce(&RebuildRequest) -> Result<ProducedFeature>,
) -> Result<RecomputeOutcome> {
    let recipe = document
        .history
        .recipes
        .get(&id)
        .ok_or(Error::Missing("history recipe"))?
        .clone();
    match recipe.state {
        EvaluationState::Detached => return Err(Error::Invalid("detached feature recompute")),
        EvaluationState::Unresolved(_) => {
            return Err(Error::Invalid("unresolved feature recompute"));
        }
        _ => {}
    }
    if cache.is_valid(document, &recipe) {
        return Ok(RecomputeOutcome::Cached {
            recipe: id,
            sha256: cache.entries[&id].sha256,
        });
    }
    let snapshot = FeatureSnapshot::capture(document, &recipe)?;
    let request = rebuild_request(document, &recipe)?;
    let produced = match producer(&request) {
        Ok(produced) => produced,
        Err(error) => {
            let reason = error.to_string();
            snapshot.restore(document);
            crate::history_ops::mark_failed(&mut document.history, id, reason.clone())?;
            return Ok(RecomputeOutcome::Failed {
                recipe: id,
                reason,
                previous_retained: true,
            });
        }
    };
    let ProducedFeature {
        outputs: built,
        subshapes: produced_subshapes,
    } = produced;
    // The output set is part of the feature record: a producer that returns different
    // objects, or geometry of the wrong kind, is a failed rebuild, not a new feature.
    let shape_error = if built.len() != recipe.outputs.len()
        || built
            .iter()
            .zip(&recipe.outputs)
            .any(|(output, expected)| output.object != *expected)
    {
        Some(Error::Invalid("rebuilt output set"))
    } else {
        built.iter().enumerate().find_map(|(index, output)| {
            recipe
                .output_ports
                .get(index)
                .and_then(|port| port.admits_geometry(output.geometry.kind()).err())
        })
    };
    if let Some(error) = shape_error {
        let reason = error.to_string();
        snapshot.restore(document);
        crate::history_ops::mark_failed(&mut document.history, id, reason.clone())?;
        return Ok(RecomputeOutcome::Failed {
            recipe: id,
            reason,
            previous_retained: true,
        });
    }
    let new_revision = recipe
        .revision
        .checked_add(1)
        .ok_or(Error::Overflow("recipe revision"))?;
    if let Err(error) = validate_rebuilt_references(&document.history, id, &produced_subshapes) {
        let reason = error.to_string();
        snapshot.restore(document);
        crate::history_ops::mark_failed(&mut document.history, id, reason.clone())?;
        return Ok(RecomputeOutcome::Failed {
            recipe: id,
            reason,
            previous_retained: true,
        });
    }
    let published: Vec<PublishedSubshapes> = produced_subshapes
        .into_iter()
        .map(|set| PublishedSubshapes {
            recipe: id,
            output: set.output,
            revision: new_revision,
            subshapes: set.subshapes,
        })
        .collect();
    // Applying the result mutates the document. If any write fails, the captured snapshot is
    // restored so a failed rebuild cannot leave a half-published node behind.
    let applied = (|| -> Result<[u8; 32]> {
        for output in &built {
            let entity = document
                .entities
                .get_mut(&output.object)
                .ok_or(Error::Missing("feature output"))?;
            entity.geometry = output.geometry.clone();
            entity.geometry_revision = entity
                .geometry_revision
                .checked_add(1)
                .ok_or(Error::Overflow("rebuild geometry revision"))?;
            let revision = entity.geometry_revision;
            let snapshot_geometry = entity.geometry.clone();
            let stamps: Vec<_> = recipe.inputs.iter().map(|input| input.source).collect();
            let operation = recipe.command.clone();
            let kind = snapshot_geometry.kind();
            match document.results.get_mut(&output.object) {
                Some(published) => {
                    published.geometry_revision = revision;
                    published.geometry_snapshot = snapshot_geometry;
                    published.source_stamps = stamps;
                    published.status = ResultStatus::Current;
                }
                None => {
                    document.results.insert(
                        output.object,
                        PublishedResult {
                            operation,
                            geometry_revision: revision,
                            geometry_snapshot: snapshot_geometry,
                            source_stamps: stamps,
                            conversion: ConversionReport::identity(kind),
                            recipe: Some(id),
                            status: ResultStatus::Current,
                        },
                    );
                }
            }
        }
        // The rebuild consumed the sources as they are now; re-stamp the recorded inputs so
        // the next recompute does not see its own successful rebuild as stale input.
        let refreshed: Vec<SourceStamp> = recipe
            .inputs
            .iter()
            .map(|input| document.stamp(input.source.object))
            .collect::<Result<_>>()?;
        let stored = document
            .history
            .recipes
            .get_mut(&id)
            .ok_or(Error::Missing("history recipe"))?;
        for (binding, stamp) in stored.inputs.iter_mut().zip(refreshed) {
            binding.source = stamp;
        }
        crate::history_ops::mark_clean(&mut document.history, id)?;
        // A rebuilt output is a changed source for everything downstream: the graph marks
        // the consumers dirty and advances their revisions, so their caches miss on the
        // next plan.
        for output in &recipe.outputs {
            document.history.invalidate(*output, false)?;
        }
        cache.record(document, &document.history.recipes[&id])
    })();
    match applied {
        Ok(sha256) => Ok(RecomputeOutcome::Rebuilt {
            recipe: id,
            outputs: recipe.outputs,
            sha256,
            subshapes: published,
        }),
        Err(error) => {
            snapshot.restore(document);
            crate::history_ops::mark_failed(&mut document.history, id, error.to_string())?;
            Err(error)
        }
    }
}

/// A rebuild must keep every reference of this feature resolvable. The check is the same
/// one `resolve_semantic_reference` applies, minus the document/revision read, because the
/// new revision exists only after `mark_clean`.
fn validate_rebuilt_references(
    graph: &HistoryGraph,
    recipe: HistoryId,
    produced: &[ProducedSubshapes],
) -> Result<()> {
    for reference in graph
        .references
        .values()
        .filter(|reference| reference.recipe == recipe)
    {
        let set = produced
            .iter()
            .find(|set| set.output == reference.output)
            .ok_or(Error::Missing("semantic subshape set"))?;
        let mut matches = set
            .subshapes
            .iter()
            .filter(|subshape| subshape.name == reference.role);
        let Some(candidate) = matches.next() else {
            return Err(Error::Missing("semantic subshape"));
        };
        if matches.next().is_some() {
            return Err(Error::Invalid("ambiguous semantic reference"));
        }
        if candidate.kind != reference.kind {
            return Err(Error::Invalid("semantic subshape kind"));
        }
        candidate.signature.validate()?;
        if !within_tolerance(
            candidate.signature.deviation_from(&reference.fingerprint),
            0.,
            reference.tolerance_mm,
        ) {
            return Err(Error::Stale("semantic reference moved"));
        }
    }
    Ok(())
}

/* ------------------- delete parent / change-input policy ------------------- */

/// The activity markers that make a delete-during-rebuild an explicit refusal instead of a
/// race. A caller marks a recipe busy for the duration of its recompute.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RebuildActivity {
    busy: BTreeSet<HistoryId>,
}

impl RebuildActivity {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn begin(&mut self, recipe: HistoryId) -> Result<()> {
        if !self.busy.insert(recipe) {
            return Err(Error::Busy);
        }
        Ok(())
    }
    pub fn finish(&mut self, recipe: HistoryId) -> bool {
        self.busy.remove(&recipe)
    }
    pub fn is_busy(&self, recipe: HistoryId) -> bool {
        self.busy.contains(&recipe)
    }
    pub fn is_empty(&self) -> bool {
        self.busy.is_empty()
    }
}

/// What to do with every feature that depended on a deleted input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokenInputPolicy {
    /// Keep the outputs, mark the feature unresolved and report the missing input.
    KeepBroken,
    /// Keep the outputs as plain objects and drop their history.
    DetachDependents,
    /// Remove the dependents and their outputs, leaving no reference at all.
    RemoveDependents,
}

impl BrokenInputPolicy {
    /// The policy menu the report offers for one broken child.
    pub fn options() -> [Self; 3] {
        [
            Self::KeepBroken,
            Self::DetachDependents,
            Self::RemoveDependents,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenDependency {
    pub recipe: HistoryId,
    pub missing: Vec<ObjectId>,
    pub outputs: Vec<ObjectId>,
    pub policies: Vec<BrokenInputPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteSourceReport {
    pub source: ObjectId,
    pub policy: Option<BrokenInputPolicy>,
    pub affected: Vec<BrokenDependency>,
    pub detached: Vec<HistoryId>,
    pub removed_recipes: Vec<HistoryId>,
    pub removed_objects: Vec<ObjectId>,
}

fn dependents(graph: &HistoryGraph, source: ObjectId) -> Vec<HistoryId> {
    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::from([source]);
    while let Some(object) = queue.pop_front() {
        for recipe in graph.recipes.values() {
            if recipe.state == EvaluationState::Detached
                || reached.contains(&recipe.id)
                || !recipe
                    .inputs
                    .iter()
                    .any(|input| input.source.object == object)
            {
                continue;
            }
            reached.insert(recipe.id);
            for output in &recipe.outputs {
                queue.push_back(*output);
            }
        }
    }
    reached.into_iter().collect()
}

/// Delete one input object and apply the chosen policy to every dependent feature.
///
/// With no policy the report is produced and the dependents stay `KeepBroken`: the caller
/// can show repair/removal choices before committing to one. `RebuildActivity` refuses the
/// delete when a dependent is being recomputed, so an asynchronous rebuild never loses its
/// input halfway.
pub fn delete_source_input(
    document: &mut Document,
    source: ObjectId,
    policy: Option<BrokenInputPolicy>,
    activity: &RebuildActivity,
) -> Result<DeleteSourceReport> {
    let affected = dependents(&document.history, source);
    if affected.iter().any(|id| activity.is_busy(*id)) {
        return Err(Error::Busy);
    }
    let direct: BTreeSet<_> = affected
        .iter()
        .copied()
        .filter(|id| {
            document.history.recipes[id]
                .inputs
                .iter()
                .any(|input| input.source.object == source)
        })
        .collect();
    let mut report = DeleteSourceReport {
        source,
        policy,
        affected: Vec::new(),
        detached: Vec::new(),
        removed_recipes: Vec::new(),
        removed_objects: Vec::new(),
    };
    for id in &affected {
        let recipe = &document.history.recipes[id];
        report.affected.push(BrokenDependency {
            recipe: *id,
            missing: if direct.contains(id) {
                vec![source]
            } else {
                vec![]
            },
            outputs: recipe.outputs.clone(),
            policies: BrokenInputPolicy::options().to_vec(),
        });
    }
    // Fail before mutating anything: a cyclic graph cannot be removed dependents-first.
    let removal_order: Option<Vec<HistoryId>> =
        if policy == Some(BrokenInputPolicy::RemoveDependents) {
            let order = document.history.execution_order()?;
            let mut removal: Vec<_> = order
                .into_iter()
                .filter(|id| affected.contains(id))
                .collect();
            // Reverse execution order: children before the producers they consume.
            removal.reverse();
            Some(removal)
        } else {
            None
        };
    document.entities.remove(&source);
    document.results.remove(&source);
    match policy {
        None | Some(BrokenInputPolicy::KeepBroken) => {
            // Direct consumers become unresolved; every downstream feature is dirty. The
            // recorded stamp stays visible in the report, never as a live reference.
            document.history.invalidate(source, true)?;
        }
        Some(BrokenInputPolicy::DetachDependents) => {
            for id in &affected {
                crate::history_ops::detach(&mut document.history, *id)?;
                drop_references_of(&mut document.history, *id);
                if let Some(recipe) = document.history.recipes.get(id) {
                    for output in &recipe.outputs {
                        if let Some(published) = document.results.get_mut(output) {
                            published.status = ResultStatus::Stale;
                        }
                    }
                }
                report.detached.push(*id);
            }
        }
        Some(BrokenInputPolicy::RemoveDependents) => {
            for id in removal_order.unwrap_or_default() {
                drop_references_of(&mut document.history, id);
                let recipe = document
                    .history
                    .recipes
                    .remove(&id)
                    .ok_or(Error::Missing("broken feature"))?;
                for output in recipe.outputs {
                    document.entities.remove(&output);
                    document.results.remove(&output);
                    report.removed_objects.push(output);
                }
                report.removed_recipes.push(id);
            }
        }
    }
    Ok(report)
}

/* ------------------------ deterministic graph baselines -------------------- */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineOutput {
    pub kind: GeometryKind,
    pub geometry: Geometry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineNode {
    pub index: usize,
    pub command: String,
    pub parameters: Options,
    pub input_ports: Vec<FeaturePort>,
    pub outputs: Vec<BaselineOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineEdge {
    pub from: usize,
    pub to: usize,
    pub role: String,
}

/// A deterministic regression record of a feature graph. Identity and revisions are
/// excluded on purpose: two sessions that build the same design from the same parameters
/// must produce the same baseline, while a changed parameter or geometry must not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureGraphBaseline {
    pub schema: String,
    pub nodes: Vec<BaselineNode>,
    pub edges: Vec<BaselineEdge>,
    pub sha256: String,
}

impl FeatureGraphBaseline {
    /// Check the recorded hash and the structural rules without needing the document.
    pub fn verify_structure(&self) -> Result<()> {
        if self.schema != "om9.feature_graph_baseline.v1" || self.nodes.is_empty() {
            return Err(Error::Invalid("feature baseline schema"));
        }
        for (position, node) in self.nodes.iter().enumerate() {
            if node.index != position {
                return Err(Error::Invalid("feature baseline node order"));
            }
            if node.command.is_empty() {
                return Err(Error::Invalid("feature baseline command"));
            }
        }
        for edge in &self.edges {
            if edge.from >= self.nodes.len()
                || edge.to >= self.nodes.len()
                || edge.from == edge.to
                || edge.role.is_empty()
            {
                return Err(Error::Invalid("feature baseline edge"));
            }
        }
        let mut reached = BTreeSet::new();
        let mut remaining: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for node in &self.nodes {
            remaining.insert(node.index, vec![]);
        }
        for edge in &self.edges {
            remaining.entry(edge.to).or_default().push(edge.from);
        }
        loop {
            let ready: Vec<_> = remaining
                .iter()
                .filter(|(index, parents)| {
                    !reached.contains(*index)
                        && parents.iter().all(|parent| reached.contains(parent))
                })
                .map(|(index, _)| *index)
                .collect();
            if ready.is_empty() {
                break;
            }
            for index in ready {
                reached.insert(index);
            }
        }
        if reached.len() != self.nodes.len() {
            return Err(Error::Invalid("feature baseline cycle"));
        }
        if self.sha256 != baseline_hash(&self.nodes, &self.edges)? {
            return Err(Error::Stale("feature baseline hash"));
        }
        Ok(())
    }
    /// Compare a stored baseline with the live document.
    pub fn verify_against(&self, document: &Document) -> Result<()> {
        self.verify_structure()?;
        let live = feature_graph_baseline(document)?;
        if live.nodes != self.nodes || live.edges != self.edges || live.sha256 != self.sha256 {
            return Err(Error::Stale("feature baseline document"));
        }
        Ok(())
    }
}

fn hex(digest: impl AsRef<[u8]>) -> String {
    let mut text = String::with_capacity(64);
    for byte in digest.as_ref() {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

fn baseline_hash(nodes: &[BaselineNode], edges: &[BaselineEdge]) -> Result<String> {
    let bytes = serde_json::to_vec(&(nodes, edges))
        .map_err(|_| Error::Invalid("feature baseline encoding"))?;
    Ok(hex(Sha256::digest(&bytes)))
}

/// Record the graph with the geometry each output currently holds.
///
/// Node order is **content order**, not insertion order: `HistoryGraph` is a `BTreeMap`
/// keyed by random UUIDs, so execution order among independent features is not stable
/// across sessions. Each node gets a Merkle-style label over its own content and its
/// producers' labels, and the baseline sorts by `(command, label)`. Two sessions that
/// build the same design therefore agree even when a document holds two features of the
/// same command, while a changed parameter, port, input producer or geometry does not.
pub fn feature_graph_baseline(document: &Document) -> Result<FeatureGraphBaseline> {
    struct Pending {
        node: BaselineNode,
        label: String,
        inputs: Vec<(ObjectId, String)>,
        outputs: Vec<ObjectId>,
    }
    let order = document.history.execution_order()?;
    let mut producer_of: BTreeMap<ObjectId, HistoryId> = BTreeMap::new();
    for id in &order {
        for output in &document.history.recipes[id].outputs {
            producer_of.insert(*output, *id);
        }
    }
    let mut label_of: BTreeMap<HistoryId, String> = BTreeMap::new();
    let mut pending = Vec::new();
    for id in &order {
        let recipe = &document.history.recipes[id];
        let mut outputs = Vec::new();
        for output in &recipe.outputs {
            let entity = document
                .entities
                .get(output)
                .ok_or(Error::Missing("feature baseline output"))?;
            outputs.push(BaselineOutput {
                kind: entity.geometry.kind(),
                geometry: entity.geometry.clone(),
            });
        }
        // The producer chain is part of the label: two features with identical content but
        // different inputs must not be interchangeable in the record.
        let mut parents: Vec<(String, String)> = recipe
            .inputs
            .iter()
            .map(|input| {
                let producer = producer_of
                    .get(&input.source.object)
                    .and_then(|producer| label_of.get(producer))
                    .cloned()
                    .unwrap_or_default();
                (input.role.clone(), producer)
            })
            .collect();
        parents.sort();
        let content = serde_json::to_vec(&(
            recipe.command.as_str(),
            &recipe.parameters,
            &recipe.input_ports,
            &outputs,
            &parents,
        ))
        .map_err(|_| Error::Invalid("feature baseline node encoding"))?;
        let label = hex(Sha256::digest(&content));
        label_of.insert(*id, label.clone());
        pending.push(Pending {
            node: BaselineNode {
                index: 0,
                command: recipe.command.clone(),
                parameters: recipe.parameters.clone(),
                input_ports: recipe.input_ports.clone(),
                outputs,
            },
            label,
            inputs: recipe
                .inputs
                .iter()
                .map(|input| (input.source.object, input.role.clone()))
                .collect(),
            outputs: recipe.outputs.clone(),
        });
    }
    pending.sort_by(|a, b| {
        (a.node.command.as_str(), a.label.as_str())
            .cmp(&(b.node.command.as_str(), b.label.as_str()))
    });
    let mut producer_index: BTreeMap<ObjectId, usize> = BTreeMap::new();
    let mut nodes = Vec::with_capacity(pending.len());
    for (index, entry) in pending.iter_mut().enumerate() {
        entry.node.index = index;
        for output in &entry.outputs {
            producer_index.insert(*output, index);
        }
        nodes.push(entry.node.clone());
    }
    let mut edges = Vec::new();
    for (index, entry) in pending.iter().enumerate() {
        for (object, role) in &entry.inputs {
            if let Some(from) = producer_index.get(object) {
                edges.push(BaselineEdge {
                    from: *from,
                    to: index,
                    role: role.clone(),
                });
            }
        }
    }
    edges.sort_by(|a, b| (a.to, a.from, &a.role).cmp(&(b.to, b.from, &b.role)));
    let sha256 = baseline_hash(&nodes, &edges)?;
    Ok(FeatureGraphBaseline {
        schema: "om9.feature_graph_baseline.v1".into(),
        nodes,
        edges,
        sha256,
    })
}

/// Load a stored baseline and prove it still describes the live document. Corrupt JSON,
/// a missing output and a hash mismatch are all refusals; a fixture never loads "best
/// effort".
pub fn load_baseline(bytes: &[u8], document: &Document) -> Result<FeatureGraphBaseline> {
    let baseline: FeatureGraphBaseline = serde_json::from_slice(bytes)
        .map_err(|error| Error::Io(format!("feature baseline decode: {error}")))?;
    baseline.verify_against(document)?;
    Ok(baseline)
}
