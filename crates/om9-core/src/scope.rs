// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{identity::TransactionId, transaction::World};
use std::thread::ThreadId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeToken {
    pub id: TransactionId,
    pub(crate) generation: u64,
    pub(crate) thread: ThreadId,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Scope {
    pub token: ScopeToken,
    pub label: String,
    pub recording: bool,
    pub active: bool,
    pub baseline: World,
    pub staged: World,
    pub version: u64,
    pub savepoints: Vec<World>,
}
