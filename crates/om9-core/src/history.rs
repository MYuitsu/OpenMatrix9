// SPDX-License-Identifier: LGPL-2.1-or-later
//! Durable recipe graph, separate from transaction Undo and Icon History.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Transform, Unit},
    identity::*,
    model::Document,
    request::{Options, SourceBinding, options_usage},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tolerance {
    pub absolute_mm: f64,
    pub relative: f64,
    pub angular_rad: f64,
}
/// The tolerance comparison rule of OM9-L3-01-08-003: two lengths are equal within `tolerance`
/// when their absolute difference is at most it. The comparison is exact (no epsilon fuzz), so a
/// value one ULP past the tolerance is outside it; a non-finite input is never within.
pub fn within_tolerance(a: f64, b: f64, tolerance: f64) -> bool {
    let difference = (a - b).abs();
    difference.is_finite() && tolerance.is_finite() && tolerance >= 0.0 && difference <= tolerance
}

impl Tolerance {
    /// The document's tolerance policy (OM9-L3-01-08-003). Labels are the recorded
    /// contract, so a refusal names the exact field.
    pub fn try_new(absolute_mm: f64, relative: f64, angular_rad: f64) -> Result<Self> {
        let tolerance = Self {
            absolute_mm,
            relative,
            angular_rad,
        };
        if !absolute_mm.is_finite() || absolute_mm <= 0. {
            return Err(Error::Invalid("absolute tolerance"));
        }
        if !relative.is_finite() || relative < 0. {
            return Err(Error::Invalid("relative tolerance"));
        }
        if !angular_rad.is_finite() || angular_rad < 0. {
            return Err(Error::Invalid("angular tolerance"));
        }
        Ok(tolerance)
    }
    /// The tolerance a **new** OM9 document starts with (template policy, not a hidden
    /// default): millimetre template values, 0.001 mm absolute, 0.01% relative and one
    /// degree angular. No accessible Matrix page confirms these numbers, so they are
    /// recorded as an OM9 template decision that a document template may override;
    /// `Document::new` itself still states no tolerance.
    pub fn om9_template() -> Self {
        Self {
            absolute_mm: 0.001,
            relative: 1.0e-4,
            angular_rad: std::f64::consts::PI / 180.0,
        }
    }
    /// Geometry at or below the absolute tolerance is unresolvable for this document.
    pub fn admit_feature(&self, size: crate::units::LengthMm) -> Result<()> {
        self.validate()?;
        if size.mm().abs() <= self.absolute_mm {
            return Err(Error::Invalid(
                "feature at or below document absolute tolerance",
            ));
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        if ![self.absolute_mm, self.relative, self.angular_rad]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
            || self.absolute_mm == 0.
        {
            return Err(Error::Invalid("effective tolerance"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayEnvironment {
    pub command_version: u32,
    pub solver: String,
    pub solver_version: String,
    pub schema_version: u32,
    pub length_unit: Unit,
    pub frame: Transform,
    pub tolerance: Tolerance,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryPolicy {
    pub record: bool,
    pub update: bool,
    pub lock: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationState {
    Clean,
    Dirty(String),
    Pending,
    Unresolved(String),
    Failed(String),
    Detached,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: HistoryId,
    pub document: DocumentId,
    pub command: String,
    pub environment: ReplayEnvironment,
    pub inputs: Vec<SourceBinding>,
    pub parameters: Options,
    pub outputs: Vec<ObjectId>,
    pub policy: HistoryPolicy,
    pub state: EvaluationState,
    pub revision: u64,
    /// Declared type of every input role (`OM9-L3-01-04-001`). A recipe recorded
    /// before typed ports existed carries none and is reported as `Untyped`; a type
    /// is never inferred from the geometry it happens to be bound to today.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_ports: Vec<crate::associative::FeaturePort>,
    /// Declared geometry kind of every output, aligned with `outputs`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_ports: Vec<crate::associative::FeaturePort>,
}
impl Recipe {
    pub fn new(
        document: DocumentId,
        command: impl Into<String>,
        inputs: Vec<SourceBinding>,
        outputs: Vec<ObjectId>,
        environment: ReplayEnvironment,
    ) -> Self {
        Self {
            id: HistoryId::new(),
            document,
            command: command.into(),
            environment,
            inputs,
            parameters: Options::new(),
            outputs,
            // These are explicit creation policy for this constructor, not UI
            // defaults recovered from Rhino. Callers set their session policy.
            policy: HistoryPolicy {
                record: true,
                update: true,
                lock: false,
            },
            state: EvaluationState::Clean,
            revision: 1,
            input_ports: vec![],
            output_ports: vec![],
        }
    }
    /// Attach typed ports to this recipe. The two lists must line up with `inputs`
    /// and `outputs`; `validate` refuses a mismatch, so a caller can never end up
    /// with a type that silences an unrelated edge.
    pub fn with_ports(
        mut self,
        input_ports: Vec<crate::associative::FeaturePort>,
        output_ports: Vec<crate::associative::FeaturePort>,
    ) -> Self {
        self.input_ports = input_ports;
        self.output_ports = output_ports;
        self
    }
    pub fn validate(&self, d: &Document, l: &Limits) -> Result<Usage> {
        self.validate_with_pending(d, l, &BTreeSet::new())
    }
    /// Validate a recipe whose outputs may be inserted by the same transaction. A Clean
    /// recipe normally requires its outputs to exist; a builder that computes the geometry
    /// before the transaction records a Clean recipe whose output entity arrives with it, so
    /// the transaction passes the ids it is about to insert.
    pub fn validate_with_pending(
        &self,
        d: &Document,
        l: &Limits,
        pending: &BTreeSet<ObjectId>,
    ) -> Result<Usage> {
        if self.id.is_nil()
            || self.document != d.id
            || self.command.is_empty()
            || self.environment.command_version == 0
            || self.environment.schema_version != 1
            || self.revision == 0
            || self.outputs.is_empty()
        {
            return Err(Error::Invalid("recipe identity/schema"));
        }
        for s in [
            &self.command,
            &self.environment.solver,
            &self.environment.solver_version,
        ] {
            name(s, l)?;
            if s.is_empty() {
                return Err(Error::Invalid("recipe replay module"));
            }
        }
        self.environment.frame.validate()?;
        self.environment.tolerance.validate()?;
        if self.environment.length_unit != Unit::Millimeter {
            return Err(Error::Invalid("canonical recipe lengths"));
        }
        let mut ids = BTreeSet::new();
        for id in &self.outputs {
            if id.is_nil() || !ids.insert(id) {
                return Err(Error::Invalid("recipe output identity"));
            }
        }
        if self.state == EvaluationState::Clean {
            for input in &self.inputs {
                d.resolve(&input.source)?;
            }
            if self
                .outputs
                .iter()
                .any(|id| !pending.contains(id) && !d.entities.contains_key(id))
            {
                return Err(Error::Missing("clean recipe output"));
            }
        }
        let mut u = Usage {
            bytes: std::mem::size_of::<Self>() as u64
                + self.command.capacity() as u64
                + self.environment.solver.capacity() as u64
                + self.environment.solver_version.capacity() as u64,
            history: 1,
            ..Usage::default()
        };
        u.add(Usage::array::<SourceBinding>(self.inputs.capacity())?, l)?;
        u.add(Usage::array::<ObjectId>(self.outputs.capacity())?, l)?;
        for input in &self.inputs {
            u.add(input.owned_usage(l)?, l)?;
        }
        u.add(options_usage(&self.parameters, l)?, l)?;
        if let EvaluationState::Dirty(s)
        | EvaluationState::Unresolved(s)
        | EvaluationState::Failed(s) = &self.state
        {
            name(s, l)?;
            u.add(
                Usage {
                    bytes: s.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        u.add(
            crate::associative::validate_typed_ports_with_pending(self, d, l, pending)?,
            l,
        )?;
        Ok(u)
    }
}
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryGraph {
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    pub recipes: BTreeMap<HistoryId, Recipe>,
    /// Semantic subshape references that survive a rebuild (`OM9-L3-01-04-004`).
    /// Kept on the graph, not on the entity, so a reference is owned by the feature
    /// that produced the subshape and is dropped with it.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub references: BTreeMap<ReferenceId, crate::associative::SemanticReference>,
}
impl HistoryGraph {
    pub fn locked(&self, id: ObjectId) -> bool {
        self.recipes.values().any(|r| {
            r.policy.lock && r.state != EvaluationState::Detached && r.outputs.contains(&id)
        })
    }
    pub fn owner(&self, id: ObjectId) -> Option<HistoryId> {
        self.recipes
            .values()
            .find(|r| r.outputs.contains(&id) && r.state != EvaluationState::Detached)
            .map(|r| r.id)
    }
    pub fn execution_order(&self) -> Result<Vec<HistoryId>> {
        self.execution_order_inner()
    }
    /// Conservative temporary storage for topological traversal. Compute this
    /// without building maps, sets or vectors, before allocating the traversal.
    pub fn execution_scratch_usage(&self) -> Result<Usage> {
        let unlimited = Limits {
            max_bytes: u64::MAX,
            ..Limits::default()
        };
        let count = self.recipes.len();
        let mut outputs = 0usize;
        let mut inputs = 0usize;
        let mut most_inputs = 0usize;
        for recipe in self.recipes.values() {
            if recipe.state != EvaluationState::Detached {
                outputs = outputs
                    .checked_add(recipe.outputs.len())
                    .ok_or(Error::Overflow("History outputs"))?;
            }
            inputs = inputs
                .checked_add(recipe.inputs.len())
                .ok_or(Error::Overflow("History inputs"))?;
            most_inputs = most_inputs.max(recipe.inputs.len());
        }
        let mut u = Usage::btree::<ObjectId, HistoryId>(outputs)?;
        u.add(Usage::btree::<HistoryId, usize>(count)?, &unlimited)?;
        u.add(
            Usage::btree::<HistoryId, BTreeSet<HistoryId>>(count)?,
            &unlimited,
        )?;
        // Child sets can be distributed over every recipe. Include one root
        // per bucket as well as all edge slots and the current dependency set.
        let roots = Usage::btree::<HistoryId, ()>(0)?
            .bytes
            .checked_mul(count as u64)
            .ok_or(Error::Overflow("History set roots"))?;
        u.add(
            Usage {
                bytes: roots,
                ..Usage::default()
            },
            &unlimited,
        )?;
        u.add(Usage::btree::<HistoryId, ()>(inputs)?, &unlimited)?;
        u.add(Usage::btree::<HistoryId, ()>(most_inputs)?, &unlimited)?;
        u.add(Usage::array::<HistoryId>(count)?, &unlimited)?;
        u.add(Usage::array::<HistoryId>(count)?, &unlimited)?;
        Ok(u)
    }
    pub fn execution_order_bounded(
        &self,
        limits: &Limits,
        mut held: Usage,
    ) -> Result<Vec<HistoryId>> {
        held.add(self.execution_scratch_usage()?, limits)?;
        self.execution_order_inner()
    }
    fn execution_order_inner(&self) -> Result<Vec<HistoryId>> {
        let mut owners = BTreeMap::new();
        for recipe in self
            .recipes
            .values()
            .filter(|r| r.state != EvaluationState::Detached)
        {
            for output in &recipe.outputs {
                if owners.insert(*output, recipe.id).is_some() {
                    return Err(Error::Invalid("multiple recipe owners"));
                }
            }
        }
        let mut degrees = BTreeMap::new();
        let mut children = BTreeMap::<HistoryId, BTreeSet<HistoryId>>::new();
        for recipe in self.recipes.values() {
            let dependencies: BTreeSet<_> = recipe
                .inputs
                .iter()
                .filter_map(|i| owners.get(&i.source.object).copied())
                .collect();
            degrees.insert(recipe.id, dependencies.len());
            for dep in dependencies {
                children.entry(dep).or_default().insert(recipe.id);
            }
        }
        let mut ready = VecDeque::new();
        ready
            .try_reserve_exact(self.recipes.len())
            .map_err(|_| Error::Budget("History ready queue"))?;
        ready.extend(degrees.iter().filter(|(_, n)| **n == 0).map(|(id, _)| *id));
        let mut order = Vec::new();
        order
            .try_reserve_exact(self.recipes.len())
            .map_err(|_| Error::Budget("History execution order"))?;
        while let Some(id) = ready.pop_front() {
            order.push(id);
            if let Some(dependents) = children.get(&id) {
                for child in dependents {
                    let n = degrees
                        .get_mut(child)
                        .ok_or(Error::Missing("recipe dependency"))?;
                    *n = n.checked_sub(1).ok_or(Error::Invalid("recipe indegree"))?;
                    if *n == 0 {
                        ready.push_back(*child);
                    }
                }
            }
        }
        if order.len() != self.recipes.len() {
            return Err(Error::Invalid("History cycle"));
        }
        Ok(order)
    }
    pub fn validate(&self, d: &Document, l: &Limits) -> Result<Usage> {
        let u = self.owned_usage(d, l)?;
        self.execution_order_bounded(l, u)?;
        Ok(u)
    }
    pub(crate) fn owned_usage(&self, d: &Document, l: &Limits) -> Result<Usage> {
        let mut u = Usage::btree::<HistoryId, Recipe>(self.recipes.len())?;
        for (id, r) in &self.recipes {
            if *id != r.id {
                return Err(Error::Invalid("recipe map key"));
            }
            u.add(r.validate(d, l)?, l)?;
        }
        u.add(
            crate::associative::references_usage(&self.references, l)?,
            l,
        )?;
        Ok(u)
    }
    pub(crate) fn invalidate(
        &mut self,
        changed: ObjectId,
        missing: bool,
    ) -> Result<BTreeSet<ObjectId>> {
        let mut queue = VecDeque::from([(changed, missing)]);
        let mut affected = BTreeSet::new();
        let mut seen = BTreeSet::new();
        while let Some((source, missing)) = queue.pop_front() {
            for r in self.recipes.values_mut() {
                if r.state == EvaluationState::Detached
                    || seen.contains(&r.id)
                    || !r.inputs.iter().any(|i| i.source.object == source)
                {
                    continue;
                }
                seen.insert(r.id);
                r.revision = r
                    .revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("recipe revision"))?;
                // Missing sources stay unresolved until explicit, valid replay.
                if missing {
                    r.state =
                        EvaluationState::Unresolved("source was deleted or transferred".into());
                } else if !matches!(r.state, EvaluationState::Unresolved(_)) {
                    r.state = EvaluationState::Dirty("source revision changed".into());
                }
                for output in &r.outputs {
                    affected.insert(*output);
                    queue.push_back((*output, false));
                }
            }
        }
        Ok(affected)
    }
}
