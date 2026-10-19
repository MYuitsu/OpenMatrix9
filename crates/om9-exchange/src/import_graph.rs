// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded source dependency validation, including disconnected retained records.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(crate) fn order(graph: &BTreeMap<Uuid, Vec<Uuid>>, limits: &Limits) -> Result<Vec<Uuid>> {
    workspace_usage(graph.len())?.check(limits)?;
    let mut states = BTreeMap::<Uuid, Option<usize>>::new();
    let mut result = Vec::new();
    result
        .try_reserve_exact(graph.len())
        .map_err(|_| Error::Budget("native dependency order"))?;
    fn visit(
        id: Uuid,
        graph: &BTreeMap<Uuid, Vec<Uuid>>,
        states: &mut BTreeMap<Uuid, Option<usize>>,
        result: &mut Vec<Uuid>,
        depth: usize,
        limits: &Limits,
    ) -> Result<usize> {
        if depth > limits.max_depth {
            return Err(Error::Budget("native dependency depth"));
        }
        if let Some(previous) = states.get(&id) {
            let height = previous.ok_or(Error::Invalid("native dependency cycle"))?;
            if depth
                .checked_add(height - 1)
                .is_none_or(|n| n > limits.max_depth)
            {
                return Err(Error::Budget("native dependency depth"));
            }
            return Ok(height);
        }
        let children = graph.get(&id).ok_or(Error::Missing("native dependency"))?;
        states.insert(id, None);
        let mut height = 1;
        for child in children {
            height = height.max(visit(*child, graph, states, result, depth + 1, limits)? + 1);
        }
        states.insert(id, Some(height));
        result.push(id);
        Ok(height)
    }
    for id in graph.keys() {
        visit(*id, graph, &mut states, &mut result, 1, limits)?;
    }
    Ok(result)
}

pub(crate) fn workspace_usage(count: usize) -> Result<Usage> {
    let mut usage = Usage::btree::<Uuid, Option<usize>>(count)?;
    usage.add(
        Usage::array::<Uuid>(count)?,
        &Limits {
            max_bytes: u64::MAX,
            ..Limits::default()
        },
    )?;
    Ok(usage)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }
    #[test]
    fn rejects_disconnected_cycles_and_missing_dependencies() {
        let cycle = BTreeMap::from([(id(1), vec![]), (id(2), vec![id(3)]), (id(3), vec![id(2)])]);
        assert!(order(&cycle, &Limits::default()).is_err());
        assert!(order(&BTreeMap::from([(id(1), vec![id(2)])]), &Limits::default()).is_err());
    }
    #[test]
    fn shared_dependency_is_emitted_once_before_all_parents() {
        let graph = BTreeMap::from([(id(1), vec![id(3)]), (id(2), vec![id(3)]), (id(3), vec![])]);
        let ordered = order(&graph, &Limits::default()).unwrap();
        assert_eq!(ordered.len(), 3);
        let at = |i| ordered.iter().position(|value| *value == id(i)).unwrap();
        assert!(at(3) < at(1) && at(3) < at(2));
    }
    #[test]
    fn depth_includes_an_already_visited_shared_subtree() {
        let graph = BTreeMap::from([
            (id(1), vec![id(2)]),
            (id(2), vec![id(3)]),
            (id(3), vec![]),
            (id(4), vec![id(5)]),
            (id(5), vec![id(1)]),
        ]);
        let limits = Limits {
            max_depth: 3,
            ..Limits::default()
        };
        assert!(matches!(
            order(&graph, &limits),
            Err(Error::Budget("native dependency depth"))
        ));
    }
}
