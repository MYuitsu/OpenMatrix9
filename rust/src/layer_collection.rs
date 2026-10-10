//! Geometry-free scope over native, verified object-role facts. Native adapters
//! prove each role; this module owns graph validation, scope and duplication.
use crate::layer_state::{LayerError, MAX_DEPTH, MAX_METADATA_BYTES, MAX_OBJECTS, valid_identity};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Geometry,
    Group,
    Aggregate,
    Definition,
    Instance,
    Metadata,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub native_type: String,
    pub role: Role,
    pub children: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Selected,
    Session,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UnsupportedObject {
    pub id: String,
    pub label: String,
    pub native_type: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Collection {
    pub version: u32,
    pub objects: Vec<String>,
    pub dependencies: Vec<String>,
    pub unsupported: Vec<UnsupportedObject>,
}
fn height(
    at: usize,
    depth: usize,
    edges: &[Vec<usize>],
    colors: &mut [u8],
    heights: &mut [usize],
) -> Result<usize, LayerError> {
    if depth > MAX_DEPTH {
        return Err(LayerError::LimitExceeded);
    }
    if colors[at] == 1 {
        return Err(LayerError::Cycle);
    }
    if colors[at] == 2 {
        if depth + heights[at] - 1 > MAX_DEPTH {
            return Err(LayerError::LimitExceeded);
        }
        return Ok(heights[at]);
    }
    colors[at] = 1;
    let mut longest = 1;
    for &child in &edges[at] {
        longest = longest.max(height(child, depth + 1, edges, colors, heights)? + 1)
    }
    colors[at] = 2;
    heights[at] = longest;
    Ok(longest)
}
fn closure(at: usize, edges: &[Vec<usize>], seen: &mut HashSet<usize>, order: &mut Vec<usize>) {
    for &child in &edges[at] {
        if seen.insert(child) {
            order.push(child);
            closure(child, edges, seen, order)
        }
    }
}
fn selected_roots(
    at: usize,
    nodes: &[Node],
    edges: &[Vec<usize>],
    seen: &mut HashSet<usize>,
    roots: &mut Vec<usize>,
    unsupported: &mut HashSet<usize>,
) -> Result<(), LayerError> {
    match nodes[at].role {
        Role::Group => {
            for &child in &edges[at] {
                selected_roots(child, nodes, edges, seen, roots, unsupported)?
            }
        }
        Role::Geometry | Role::Aggregate | Role::Instance => {
            if !seen.insert(at) {
                return Err(LayerError::DuplicateId);
            }
            roots.push(at)
        }
        Role::Unsupported => {
            unsupported.insert(at);
        }
        Role::Metadata | Role::Definition => return Err(LayerError::InvalidSnapshot),
    }
    Ok(())
}
pub fn collect(
    nodes: &[Node],
    scope: Scope,
    selected: &[String],
) -> Result<Collection, LayerError> {
    if nodes.len() > MAX_OBJECTS || selected.len() > MAX_OBJECTS {
        return Err(LayerError::LimitExceeded);
    }
    if scope == Scope::Session && !selected.is_empty() {
        return Err(LayerError::InvalidSnapshot);
    }
    let mut ids = HashMap::new();
    let mut charge = 0usize;
    for (i, node) in nodes.iter().enumerate() {
        if !valid_identity(&node.id)
            || node.label.contains('\0')
            || node.native_type.is_empty()
            || node.native_type.contains('\0')
        {
            return Err(LayerError::InvalidSnapshot);
        }
        if node.label.len() > 4096
            || node.native_type.len() > 1024
            || node.children.len() > MAX_OBJECTS
        {
            return Err(LayerError::LimitExceeded);
        }
        charge = charge
            .checked_add(
                std::mem::size_of::<Node>()
                    + node.id.len()
                    + node.label.len()
                    + node.native_type.len(),
            )
            .ok_or(LayerError::LimitExceeded)?;
        for child in &node.children {
            if !valid_identity(child) {
                return Err(LayerError::InvalidSnapshot);
            }
            charge = charge
                .checked_add(std::mem::size_of::<String>() + child.len())
                .ok_or(LayerError::LimitExceeded)?;
        }
        if charge > MAX_METADATA_BYTES {
            return Err(LayerError::LimitExceeded);
        }
        if ids.insert(node.id.as_str(), i).is_some() {
            return Err(LayerError::DuplicateId);
        }
    }
    let mut edges = Vec::with_capacity(nodes.len());
    for node in nodes {
        if matches!(
            node.role,
            Role::Geometry | Role::Metadata | Role::Unsupported
        ) && !node.children.is_empty()
        {
            return Err(LayerError::InvalidSnapshot);
        }
        let mut unique = HashSet::new();
        let mut children = Vec::new();
        for child in &node.children {
            let &at = ids.get(child.as_str()).ok_or(LayerError::MissingObject)?;
            if !unique.insert(at) {
                return Err(LayerError::DuplicateId);
            }
            children.push(at);
        }
        if node.role == Role::Instance
            && (children.len() != 1 || nodes[children[0]].role != Role::Definition)
        {
            return Err(LayerError::InvalidSnapshot);
        }
        edges.push(children);
    }
    let mut colors = vec![0; nodes.len()];
    let mut heights = vec![0; nodes.len()];
    for at in 0..nodes.len() {
        height(at, 1, &edges, &mut colors, &mut heights)?;
    }
    let mut aggregates = HashSet::new();
    let mut internal = HashSet::new();
    for (at, node) in nodes.iter().enumerate() {
        if matches!(node.role, Role::Aggregate | Role::Definition) {
            let mut descendants = Vec::new();
            let mut found = HashSet::new();
            closure(at, &edges, &mut found, &mut descendants);
            internal.extend(found.iter().copied());
            if node.role == Role::Aggregate {
                aggregates.extend(found)
            }
        }
    }
    let mut roots = Vec::new();
    let mut unsupported = HashSet::new();
    match scope {
        Scope::Session => {
            for (at, node) in nodes.iter().enumerate() {
                if matches!(node.role, Role::Geometry | Role::Aggregate | Role::Instance)
                    && !internal.contains(&at)
                {
                    roots.push(at)
                }
                if node.role == Role::Unsupported && !aggregates.contains(&at) {
                    unsupported.insert(at);
                }
            }
        }
        Scope::Selected => {
            let mut selection = HashSet::new();
            let mut emitted = HashSet::new();
            for id in selected {
                let &at = ids.get(id.as_str()).ok_or(LayerError::MissingObject)?;
                if !selection.insert(at) {
                    return Err(LayerError::DuplicateId);
                }
                selected_roots(
                    at,
                    nodes,
                    &edges,
                    &mut emitted,
                    &mut roots,
                    &mut unsupported,
                )?;
            }
            let root_set: HashSet<_> = roots.iter().copied().collect();
            for &at in &roots {
                if nodes[at].role == Role::Aggregate {
                    let mut descendants = Vec::new();
                    let mut seen = HashSet::new();
                    closure(at, &edges, &mut seen, &mut descendants);
                    if descendants.iter().any(|n| root_set.contains(n)) {
                        return Err(LayerError::DuplicateId);
                    }
                }
            }
        }
    }
    let mut dependencies = Vec::new();
    let mut seen_dependencies = HashSet::new();
    for &at in &roots {
        if nodes[at].role == Role::Instance {
            let mut found = Vec::new();
            closure(at, &edges, &mut seen_dependencies, &mut found);
            for child in found {
                if nodes[child].role == Role::Unsupported {
                    unsupported.insert(child);
                }
                if nodes[child].role != Role::Metadata {
                    dependencies.push(nodes[child].id.clone());
                }
            }
        }
    }
    Ok(Collection {
        version: 1,
        objects: roots.into_iter().map(|at| nodes[at].id.clone()).collect(),
        dependencies,
        unsupported: nodes
            .iter()
            .enumerate()
            .filter(|(at, _)| unsupported.contains(at))
            .map(|(_, n)| UnsupportedObject {
                id: n.id.clone(),
                label: n.label.clone(),
                native_type: n.native_type.clone(),
            })
            .collect(),
    })
}
