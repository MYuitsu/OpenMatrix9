// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::{Geometry, GeometryKind, Point3, Transform},
    identity::*,
    model::{Document, Entity},
    transaction::World,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    Length,
    Angle,
    Ratio,
    Count,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    Quantity { value: f64, dimension: Dimension },
    Choice(String),
    Boolean(bool),
    Text(String),
}
pub type Options = BTreeMap<String, Value>;
pub(crate) fn options_usage(options: &Options, l: &Limits) -> Result<Usage> {
    let mut usage = Usage::btree::<String, Value>(options.len())?;
    for (key, value) in options {
        name(key, l)?;
        let text = match value {
            Value::Number(n) | Value::Quantity { value: n, .. } => {
                if !n.is_finite() {
                    return Err(Error::Invalid("nonfinite stored option"));
                }
                0
            }
            Value::Text(s) | Value::Choice(s) => {
                name(s, l)?;
                s.capacity()
            }
            Value::Boolean(_) => 0,
        };
        let bytes = key
            .capacity()
            .checked_add(text)
            .and_then(|n| n.checked_add(std::mem::size_of::<(String, Value)>()))
            .ok_or(Error::Overflow("stored option bytes"))?;
        usage.add(
            Usage {
                bytes: bytes as u64,
                ..Usage::default()
            },
            l,
        )?;
    }
    Ok(usage)
}
#[derive(Debug, Clone)]
pub enum OptionDomain {
    Number {
        dimension: Dimension,
        min: Option<f64>,
        min_inclusive: bool,
        max: Option<f64>,
    },
    Choice(Vec<String>),
    Boolean,
    Text {
        max_bytes: usize,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Reference,
    Edit,
}
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub kinds: Vec<GeometryKind>,
    pub min: usize,
    pub max: usize,
    pub access: Access,
    pub allow_reuse: bool,
}
#[derive(Debug, Clone)]
pub struct RequestSchema {
    pub feature: String,
    pub roles: Vec<Role>,
    pub options: BTreeMap<String, OptionDomain>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexSpan {
    pub start: usize,
    pub count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Selection {
    Whole,
    Indices(Vec<usize>),
    Spans(Vec<IndexSpan>),
    Member(Vec<ObjectId>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBinding {
    pub role: String,
    pub source: SourceStamp,
    pub selection: Selection,
}
impl SourceBinding {
    pub(crate) fn owned_usage(&self, l: &Limits) -> Result<Usage> {
        name(&self.role, l)?;
        let mut u = Usage {
            bytes: self.role.capacity() as u64,
            ..Usage::default()
        };
        u.add(
            match &self.selection {
                Selection::Whole => Usage::default(),
                Selection::Indices(v) => Usage::array::<usize>(v.capacity())?,
                Selection::Spans(v) => Usage::array::<IndexSpan>(v.capacity())?,
                Selection::Member(v) => Usage::array::<ObjectId>(v.capacity())?,
            },
            l,
        )?;
        Ok(u)
    }
}
#[derive(Debug, Clone)]
pub struct BoundInput {
    pub binding: SourceBinding,
    document: Arc<Document>,
    transform: Transform,
    indices: Option<Vec<usize>>,
    definition_revisions: Vec<(DefinitionId, u64)>,
    member_revisions: Vec<(ObjectId, u64, u64, u64)>,
    access: Access,
}
impl BoundInput {
    pub fn entity(&self) -> Result<&Entity> {
        resolve_member(&self.document, &self.binding).map(|v| v.0)
    }
    pub fn transform(&self) -> Transform {
        self.transform
    }
    pub fn selected_points(&self) -> Result<Vec<(usize, Point3)>> {
        let entity = self.entity()?;
        let points: &[Point3] = match &entity.geometry {
            Geometry::Point(p) => std::slice::from_ref(p),
            Geometry::Cloud(c) => &c.points,
            Geometry::Mesh(m) => &m.vertices,
            Geometry::Curve(crate::geometry::Curve::Nurbs(c)) => &c.poles,
            Geometry::Surface(s) => &s.poles,
            _ => return Err(Error::Unsupported("direct control/point selection")),
        };
        let mut out = Vec::new();
        let count = self.indices.as_ref().map_or(points.len(), Vec::len);
        out.try_reserve_exact(count)
            .map_err(|_| Error::Budget("selected point allocation"))?;
        match &self.indices {
            Some(indices) => {
                for i in indices {
                    out.push((*i, self.transform.point(points[*i])?));
                }
            }
            None => {
                for (i, p) in points.iter().enumerate() {
                    out.push((i, self.transform.point(*p)?));
                }
            }
        }
        Ok(out)
    }
    pub fn revalidate(&self, world: &World) -> Result<()> {
        let document = world
            .documents
            .get(&self.binding.source.document)
            .ok_or(Error::Missing("source document"))?;
        document.resolve(&self.binding.source)?;
        let (_, transform, definitions, members) = resolve_member(document, &self.binding)?;
        if transform != self.transform
            || definitions != self.definition_revisions
            || members != self.member_revisions
        {
            return Err(Error::Stale("occurrence path"));
        }
        permissions(document, &self.binding, self.access)?;
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct AdmittedRequest {
    pub feature: String,
    pub inputs: Vec<BoundInput>,
    pub options: Options,
}
impl AdmittedRequest {
    pub fn revalidate(&self, world: &World) -> Result<()> {
        for input in &self.inputs {
            input.revalidate(world)?;
        }
        Ok(())
    }
}

type ResolvedMember<'a> = (
    &'a Entity,
    Transform,
    Vec<(DefinitionId, u64)>,
    Vec<(ObjectId, u64, u64, u64)>,
);
fn resolve_member<'a>(
    document: &'a Document,
    binding: &SourceBinding,
) -> Result<ResolvedMember<'a>> {
    let mut entity = document.resolve(&binding.source)?;
    let mut matrix = entity.placement;
    let mut definitions = vec![];
    let mut members = vec![];
    if let Selection::Member(path) = &binding.selection {
        let mut visited = BTreeSet::new();
        for id in path {
            if !visited.insert(*id) {
                return Err(Error::Invalid("duplicate member path"));
            }
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("non-instance member path"));
            };
            let d = document
                .definitions
                .get(&def)
                .ok_or(Error::Missing("definition"))?;
            definitions.push((def, d.revision));
            entity = d
                .members
                .iter()
                .find(|e| e.id == *id)
                .ok_or(Error::Missing("member"))?;
            members.push((
                entity.id,
                entity.generation,
                entity.geometry_revision,
                entity.attributes_revision,
            ));
            matrix = matrix.compose(entity.placement)?;
        }
    }
    Ok((entity, matrix, definitions, members))
}
fn permissions(d: &Document, b: &SourceBinding, access: Access) -> Result<()> {
    let p = d.permissions(b.source.object, true)?;
    if !(match access {
        Access::Reference => p.reference_snappable,
        Access::Edit => p.editable,
    }) {
        return Err(Error::Permission("bound source"));
    }
    // Member permissions are inherited from the occurrence and each member's
    // own layer/attributes; do not flatten them into selectable.
    if let Selection::Member(path) = &b.selection {
        let mut entity = d.resolve(&b.source)?;
        for id in path {
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("member path"));
            };
            entity = d
                .definitions
                .get(&def)
                .ok_or(Error::Missing("definition"))?
                .members
                .iter()
                .find(|e| e.id == *id)
                .ok_or(Error::Missing("member"))?;
            let p = d.layer_permissions(entity.attributes.layer, true)?;
            if !p.visible
                || !entity.attributes.visible
                || (access == Access::Edit && (!p.editable || entity.attributes.locked))
            {
                return Err(Error::Permission("bound member"));
            }
        }
    }
    Ok(())
}
fn point_count(g: &Geometry) -> Result<usize> {
    match g {
        Geometry::Point(_) => Ok(1),
        Geometry::Cloud(c) => Ok(c.points.len()),
        Geometry::Mesh(m) => Ok(m.vertices.len()),
        Geometry::Curve(crate::geometry::Curve::Nurbs(c)) => Ok(c.poles.len()),
        Geometry::Surface(s) => Ok(s.poles.len()),
        _ => Err(Error::Unsupported("indexed source family")),
    }
}
fn selected_indices(g: &Geometry, s: &Selection, l: &Limits) -> Result<Option<Vec<usize>>> {
    match s {
        Selection::Whole | Selection::Member(_) => Ok(None),
        Selection::Indices(indices) => {
            Usage {
                points: indices.len() as u64,
                ..Usage::array::<usize>(indices.len())?
            }
            .check(l)?;
            let count = point_count(g)?;
            let mut seen = BTreeSet::new();
            if indices.is_empty() || indices.iter().any(|i| *i >= count || !seen.insert(*i)) {
                return Err(Error::Invalid("source indices"));
            }
            Ok(Some(indices.clone()))
        }
        Selection::Spans(spans) => {
            let count = point_count(g)?;
            let mut total = 0usize;
            let mut ends = vec![];
            let mut ranges = BTreeMap::<usize, usize>::new();
            for span in spans {
                let end = span
                    .start
                    .checked_add(span.count)
                    .ok_or(Error::Overflow("span end"))?;
                if span.count == 0 || end > count {
                    return Err(Error::Invalid("span bounds"));
                }
                if ranges
                    .range(..=span.start)
                    .next_back()
                    .is_some_and(|(_, old_end)| *old_end > span.start)
                    || ranges
                        .range(span.start..)
                        .next()
                        .is_some_and(|(start, _)| *start < end)
                {
                    return Err(Error::Invalid("overlapping spans"));
                }
                ranges.insert(span.start, end);
                total = total
                    .checked_add(span.count)
                    .ok_or(Error::Overflow("span count"))?;
                ends.push((span.start, end));
            }
            if total == 0 {
                return Err(Error::Invalid("empty subset"));
            }
            Usage {
                points: total as u64,
                ..Usage::array::<usize>(total)?
            }
            .check(l)?;
            let mut indices = Vec::new();
            indices
                .try_reserve_exact(total)
                .map_err(|_| Error::Budget("span allocation"))?;
            for (start, end) in ends {
                indices.extend(start..end);
            }
            Ok(Some(indices))
        }
    }
}
fn option(domain: &OptionDomain, value: &Value, l: &Limits) -> Result<()> {
    match (domain, value) {
        (
            OptionDomain::Number {
                dimension,
                min,
                min_inclusive,
                max,
            },
            value,
        ) => {
            let n = match value {
                Value::Number(n) => *n,
                Value::Quantity {
                    value,
                    dimension: d,
                } if d == dimension => *value,
                _ => return Err(Error::Invalid("option dimension/type")),
            };
            if !n.is_finite()
                || min.is_some_and(|v| !v.is_finite() || n < v || (!min_inclusive && n == v))
                || max.is_some_and(|v| !v.is_finite() || n > v)
                || (*dimension == Dimension::Count && (n.fract() != 0. || n < 0.))
            {
                return Err(Error::Invalid("numeric option domain"));
            }
        }
        (OptionDomain::Choice(choices), Value::Choice(s)) if choices.contains(s) => name(s, l)?,
        (OptionDomain::Boolean, Value::Boolean(_)) => {}
        (OptionDomain::Text { max_bytes }, Value::Text(s)) if s.len() <= *max_bytes => name(s, l)?,
        _ => return Err(Error::Invalid("option value/type")),
    }
    Ok(())
}
pub fn admit(
    world: &World,
    schema: &RequestSchema,
    bindings: Vec<SourceBinding>,
    options: Options,
    limits: &Limits,
) -> Result<AdmittedRequest> {
    name(&schema.feature, limits)?;
    if schema.feature.is_empty() {
        return Err(Error::Invalid("feature ID"));
    }
    let mut usage = world.validate(limits)?;
    usage.add(Usage::array::<BoundInput>(bindings.len())?, limits)?;
    let mut roles = BTreeMap::new();
    for role in &schema.roles {
        if role.name.is_empty()
            || role.max < role.min
            || role.kinds.is_empty()
            || roles.insert(role.name.as_str(), role).is_some()
        {
            return Err(Error::Invalid("request role schema"));
        }
    }
    for (key, value) in &options {
        name(key, limits)?;
        option(
            schema
                .options
                .get(key)
                .ok_or(Error::Invalid("unknown option"))?,
            value,
            limits,
        )?;
        let text_bytes = match value {
            Value::Text(s) | Value::Choice(s) => s.len(),
            _ => 0,
        };
        let bytes = key
            .len()
            .checked_add(text_bytes)
            .and_then(|n| n.checked_add(std::mem::size_of::<(String, Value)>()))
            .ok_or(Error::Overflow("option bytes"))?;
        usage.add(
            Usage {
                bytes: bytes as u64,
                ..Usage::default()
            },
            limits,
        )?;
    }
    let mut counts = BTreeMap::<&str, usize>::new();
    let mut inputs = Vec::<BoundInput>::new();
    let mut reuse = vec![];
    inputs
        .try_reserve_exact(bindings.len())
        .map_err(|_| Error::Budget("binding allocation"))?;
    for binding in bindings {
        let role = roles
            .get(binding.role.as_str())
            .ok_or(Error::Invalid("unknown role"))?;
        let count = counts.entry(role.name.as_str()).or_default();
        *count = count.checked_add(1).ok_or(Error::Overflow("role count"))?;
        if *count > role.max {
            return Err(Error::Invalid("role cardinality"));
        }
        if let Selection::Member(path) = &binding.selection
            && (path.is_empty() || path.len() > limits.max_depth)
        {
            return Err(Error::Invalid("member path depth"));
        }
        let document = world
            .documents
            .get(&binding.source.document)
            .ok_or(Error::Missing("source document"))?;
        permissions(document, &binding, role.access)?;
        let (entity, transform, definitions, members) = resolve_member(document, &binding)?;
        if !role.kinds.contains(&entity.geometry.kind()) {
            return Err(Error::Unsupported("role representation"));
        }
        // Reserve selected-point output and temporary index-validation storage
        // in the SAME budget as the shared source, before constructing either.
        let (selected_count, index_count, span_count) = match &binding.selection {
            Selection::Whole | Selection::Member(_) => {
                (point_count(&entity.geometry).unwrap_or(0), 0, 0)
            }
            Selection::Indices(indices) => (indices.len(), indices.len(), 0),
            Selection::Spans(spans) => {
                let count = spans.iter().try_fold(0usize, |n, s| {
                    n.checked_add(s.count)
                        .ok_or(Error::Overflow("selected span count"))
                })?;
                (count, count, spans.len())
            }
        };
        let mut planned = Usage::array::<(usize, Point3)>(selected_count)?;
        planned.points = selected_count as u64;
        planned.add(Usage::array::<usize>(index_count)?, limits)?;
        // Conservative BTree node overhead for duplicate/range checking.
        planned.add(
            Usage::array::<[u8; 64]>(
                index_count
                    .checked_add(span_count)
                    .ok_or(Error::Overflow("selection validation nodes"))?,
            )?,
            limits,
        )?;
        planned.add(
            Usage {
                bytes: binding.role.len() as u64,
                nodes: span_count as u64,
                ..Usage::default()
            },
            limits,
        )?;
        usage.add(planned, limits)?;
        let indices = selected_indices(&entity.geometry, &binding.selection, limits)?;
        for (i, previous) in inputs.iter().enumerate() {
            if previous.binding.source.document == binding.source.document
                && previous.binding.source.object == binding.source.object
                && !(role.allow_reuse && reuse[i])
            {
                let a = match &previous.binding.selection {
                    Selection::Member(p) => p.as_slice(),
                    _ => &[],
                };
                let b = match &binding.selection {
                    Selection::Member(p) => p.as_slice(),
                    _ => &[],
                };
                let overlapping = if a == b {
                    match (&previous.indices, &indices) {
                        (Some(a), Some(b)) => {
                            let set: BTreeSet<_> = a.iter().collect();
                            b.iter().any(|i| set.contains(i))
                        }
                        _ => true,
                    }
                } else {
                    a.starts_with(b) || b.starts_with(a)
                };
                if overlapping {
                    return Err(Error::Invalid("duplicate/overlapping source path"));
                }
            }
        }
        inputs.push(BoundInput {
            binding,
            document: document.clone(),
            transform,
            indices,
            definition_revisions: definitions,
            member_revisions: members,
            access: role.access,
        });
        reuse.push(role.allow_reuse);
    }
    for role in &schema.roles {
        if counts.get(role.name.as_str()).copied().unwrap_or(0) < role.min {
            return Err(Error::Invalid("missing required role"));
        }
    }
    Ok(AdmittedRequest {
        feature: schema.feature.clone(),
        inputs,
        options,
    })
}
