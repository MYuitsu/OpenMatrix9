// SPDX-License-Identifier: LGPL-2.1-or-later
//! Configurable OM9 implementation limits, not recovered Rhino defaults.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_bytes: u64,
    pub max_objects: u64,
    pub max_nodes: u64,
    pub max_points: u64,
    pub max_history: u64,
    pub max_outputs: u64,
    pub max_depth: usize,
    pub max_degree: usize,
    pub max_name_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024,
            max_objects: 100_000,
            max_nodes: 4_000_000,
            max_points: 2_000_000,
            max_history: 10_000,
            max_outputs: 100_000,
            max_depth: 32,
            max_degree: 32,
            max_name_bytes: 16 * 1024,
        }
    }
}
impl Limits {
    /// Remaining aggregate counters while an already-owned allocation stays
    /// live. Depth/degree/name constraints apply per value and are not summed.
    pub fn remaining_after(self, held: Usage) -> Result<Self> {
        held.check(&self)?;
        Ok(Self {
            max_bytes: self.max_bytes - held.bytes,
            max_objects: self.max_objects - held.objects,
            max_nodes: self.max_nodes - held.nodes,
            max_points: self.max_points - held.points,
            max_history: self.max_history - held.history,
            max_outputs: self.max_outputs - held.outputs,
            ..self
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub bytes: u64,
    pub objects: u64,
    pub nodes: u64,
    pub points: u64,
    pub history: u64,
    pub outputs: u64,
}
impl Usage {
    pub fn add(&mut self, other: Self, limits: &Limits) -> Result<()> {
        let sum = Self {
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(Error::Overflow("bytes"))?,
            objects: self
                .objects
                .checked_add(other.objects)
                .ok_or(Error::Overflow("objects"))?,
            nodes: self
                .nodes
                .checked_add(other.nodes)
                .ok_or(Error::Overflow("nodes"))?,
            points: self
                .points
                .checked_add(other.points)
                .ok_or(Error::Overflow("points"))?,
            history: self
                .history
                .checked_add(other.history)
                .ok_or(Error::Overflow("history"))?,
            outputs: self
                .outputs
                .checked_add(other.outputs)
                .ok_or(Error::Overflow("outputs"))?,
        };
        sum.check(limits)?;
        *self = sum;
        Ok(())
    }
    pub fn check(self, l: &Limits) -> Result<()> {
        for (value, max, name) in [
            (self.bytes, l.max_bytes, "bytes"),
            (self.objects, l.max_objects, "objects"),
            (self.nodes, l.max_nodes, "nodes"),
            (self.points, l.max_points, "points"),
            (self.history, l.max_history, "history"),
            (self.outputs, l.max_outputs, "outputs"),
        ] {
            if value > max {
                return Err(Error::Budget(name));
            }
        }
        Ok(())
    }
    pub fn array<T>(len: usize) -> Result<Self> {
        let bytes = len
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(Error::Overflow("array bytes"))?;
        Ok(Self {
            bytes: u64::try_from(bytes).map_err(|_| Error::Overflow("array bytes"))?,
            ..Self::default()
        })
    }
    /// Conservative BTree storage for the pinned Rust profile: eleven key/value
    /// slots per node, at least five entries in each non-root node, and a full
    /// internal-node header/link/alignment allowance. Empty maps may retain a
    /// previously allocated root. This is admission accounting, not physical RAM.
    /// Layout reference: https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html
    pub fn btree<K, V>(len: usize) -> Result<Self> {
        let nodes = len.saturating_sub(1) / 5 + 1;
        let node_bytes = std::mem::size_of::<(K, V)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(16 * std::mem::size_of::<usize>()))
            .ok_or(Error::Overflow("BTree node bytes"))?;
        let bytes = nodes
            .checked_mul(node_bytes)
            .ok_or(Error::Overflow("BTree allocation bytes"))?;
        Ok(Self {
            bytes: u64::try_from(bytes).map_err(|_| Error::Overflow("BTree address space"))?,
            ..Self::default()
        })
    }
    pub fn spare<T>(len: usize, capacity: usize) -> Result<Self> {
        Self::array::<T>(
            capacity
                .checked_sub(len)
                .ok_or(Error::Invalid("owned buffer capacity"))?,
        )
    }
}
pub fn name(value: &str, limits: &Limits) -> Result<()> {
    if value.len() > limits.max_name_bytes {
        return Err(Error::Budget("name bytes"));
    }
    if value.contains('\0') {
        return Err(Error::Invalid("NUL in name"));
    }
    Ok(())
}
