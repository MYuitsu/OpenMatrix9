// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native cache invalidation intent. Actual bounds require independent geometry.
use crate::{
    HostValue, NamespaceOverlay, RecordAction, RecordOverlay, ScopedNativeKey,
    current_overlay::{canceled, counts},
};
use om9_core::{
    Error, Result,
    budget::Usage,
    geometry::Geometry,
    identity::{DefinitionId, ImportNamespaceId},
    model::Document,
    retained::{HostRef, NativeRecordKind},
    transaction::Cancellation,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Source(ScopedNativeKey),
    Host(DefinitionId),
}

pub(crate) fn reservation(d: &Document) -> Result<Usage> {
    let limits = om9_core::budget::Limits {
        max_bytes: u64::MAX,
        ..Default::default()
    };
    let (records, hosts) = counts(d)?;
    let nodes = records
        .checked_add(d.definitions.len())
        .ok_or(Error::Overflow("cache graph nodes"))?;
    let mut edges = hosts;
    for ns in d.import_namespaces.values() {
        for record in &ns.records {
            edges = edges
                .checked_add(record.relations.len())
                .ok_or(Error::Overflow("cache graph edges"))?;
        }
    }
    let mut u = Usage::btree::<ScopedNativeKey, (usize, usize)>(records)?;
    u.add(
        Usage::btree::<DefinitionId, Node>(d.definitions.len())?,
        &limits,
    )?;
    u.add(Usage::btree::<Node, Vec<Node>>(nodes)?, &limits)?;
    u.add(Usage::btree::<Node, ()>(nodes)?, &limits)?;
    u.add(Usage::array::<Node>(nodes)?, &limits)?;
    // Each fallible edge Vec growth admits its old/new storage together.
    u.add(
        Usage::array::<Node>(
            edges
                .checked_mul(4)
                .ok_or(Error::Overflow("cache edge storage"))?,
        )?,
        &limits,
    )?;
    Ok(u)
}
fn live(row: &RecordOverlay<'_>) -> bool {
    !matches!(row.action, RecordAction::Delete | RecordAction::Omit)
}
fn source_node(namespace: ImportNamespaceId, row: &RecordOverlay<'_>) -> Node {
    Node::Source(ScopedNativeKey {
        namespace,
        key: row.source.key,
    })
}
fn child(
    row: &RecordOverlay<'_>,
    namespace: ImportNamespaceId,
    aliases: &BTreeMap<DefinitionId, Node>,
) -> Result<Option<Node>> {
    if let Some(HostValue::Entity { entity, .. }) = row.current {
        return if let Geometry::Instance(id) = entity.geometry {
            Ok(Some(
                *aliases
                    .get(&id)
                    .ok_or(Error::Missing("cache current definition"))?,
            ))
        } else {
            Ok(None)
        };
    }
    let mut result = None;
    for relation in row
        .source
        .relations
        .iter()
        .filter(|r| r.role == "instance_definition")
    {
        if row.source.key.kind != NativeRecordKind::Geometry
            || relation.target.kind != NativeRecordKind::Definition
            || result.is_some()
        {
            return Err(Error::Invalid("cache native instance relation"));
        }
        result = Some(Node::Source(ScopedNativeKey {
            namespace,
            key: relation.target,
        }));
    }
    Ok(result)
}
fn seed(node: Node, dirty: &mut BTreeSet<Node>, queue: &mut Vec<Node>) {
    if dirty.insert(node) {
        queue.push(node);
    }
}
fn edge(child: Node, parent: Node, reverse: &mut BTreeMap<Node, Vec<Node>>) -> Result<()> {
    let list = reverse
        .get_mut(&child)
        .ok_or(Error::Missing("cache native child definition"))?;
    list.try_reserve(1)
        .map_err(|_| Error::Budget("cache dependency edges"))?;
    list.push(parent);
    Ok(())
}
pub(crate) fn refresh(
    d: &Document,
    namespaces: &mut [NamespaceOverlay<'_>],
    cancel: &Cancellation,
) -> Result<()> {
    let (records, _) = counts(d)?;
    let nodes = records
        .checked_add(d.definitions.len())
        .ok_or(Error::Overflow("cache graph nodes"))?;
    let mut queue = Vec::new();
    queue
        .try_reserve_exact(nodes)
        .map_err(|_| Error::Budget("cache dependency queue"))?;
    let mut slots = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    let mut reverse = BTreeMap::new();
    let mut dirty = BTreeSet::new();
    for id in d.definitions.keys() {
        aliases.insert(*id, Node::Host(*id));
    }
    for (i, ns) in namespaces.iter().enumerate() {
        for (j, row) in ns.records.iter().enumerate() {
            canceled(cancel)?;
            if slots
                .insert(
                    ScopedNativeKey {
                        namespace: ns.namespace.id,
                        key: row.source.key,
                    },
                    (i, j),
                )
                .is_some()
            {
                return Err(Error::Invalid("cache native key duplicate"));
            }
            if row.source.key.kind == NativeRecordKind::Definition && live(row) {
                let node = source_node(ns.namespace.id, row);
                reverse.entry(node).or_insert_with(Vec::new);
                if let Some(HostValue::Definition(def)) = row.current {
                    aliases.insert(def.id, node);
                }
                if matches!(
                    row.action,
                    RecordAction::PatchDefinition | RecordAction::Regenerate
                ) {
                    seed(node, &mut dirty, &mut queue);
                }
            }
        }
    }
    for node in aliases.values() {
        canceled(cancel)?;
        reverse.entry(*node).or_insert_with(Vec::new);
        if matches!(node, Node::Host(_)) {
            seed(*node, &mut dirty, &mut queue);
        }
    }
    for (id, def) in &d.definitions {
        let parent = aliases[id];
        for member in &def.members {
            canceled(cancel)?;
            if let Geometry::Instance(id) = member.geometry {
                edge(
                    *aliases
                        .get(&id)
                        .ok_or(Error::Missing("cache represented child definition"))?,
                    parent,
                    &mut reverse,
                )?;
            }
        }
    }
    for ns in namespaces.iter() {
        for row in &ns.records {
            canceled(cancel)?;
            if row.source.key.kind == NativeRecordKind::Definition
                && live(row)
                && row.current.is_none()
            {
                let parent = source_node(ns.namespace.id, row);
                for relation in row
                    .source
                    .relations
                    .iter()
                    .filter(|r| r.role == "definition_member")
                {
                    let key = ScopedNativeKey {
                        namespace: ns.namespace.id,
                        key: relation.target,
                    };
                    let (i, j) = *slots
                        .get(&key)
                        .ok_or(Error::Missing("cache retained member"))?;
                    let member = &namespaces[i].records[j];
                    if let Some(target) = child(member, ns.namespace.id, &aliases)? {
                        edge(target, parent, &mut reverse)?;
                    }
                }
            }
            if let Some(HostRef::DefinitionMember { definition, .. }) = row.source.host
                && (matches!(row.action, RecordAction::Delete | RecordAction::Regenerate)
                    || (row.action == RecordAction::PatchEntity
                        && (row.changed.geometry || row.changed.placement)))
                && let Some(parent) = aliases.get(&definition)
            {
                seed(*parent, &mut dirty, &mut queue);
            }
        }
    }
    let mut cursor = 0;
    while cursor < queue.len() {
        canceled(cancel)?;
        let node = queue[cursor];
        cursor += 1;
        for parent in reverse
            .get(&node)
            .ok_or(Error::Missing("cache dirty definition node"))?
        {
            seed(*parent, &mut dirty, &mut queue);
        }
    }
    for ns in namespaces {
        for row in &mut ns.records {
            canceled(cancel)?;
            row.needs_cache_refresh = live(row)
                && if row.source.key.kind == NativeRecordKind::Definition {
                    dirty.contains(&source_node(ns.namespace.id, row))
                } else if let Some(target) = child(row, ns.namespace.id, &aliases)? {
                    dirty.contains(&target)
                        || row.action == RecordAction::Regenerate
                        || (row.action == RecordAction::PatchEntity
                            && (row.changed.geometry || row.changed.placement))
                } else {
                    false
                };
        }
    }
    Ok(())
}
