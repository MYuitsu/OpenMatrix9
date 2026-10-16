// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    Error, Result,
    identity::*,
    request::AdmittedRequest,
    transaction::{Cancellation, TransactionCoordinator},
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerPlan {
    pub workers: u64,
    pub reserved_bytes: u64,
}
pub fn plan_workers(
    logical_cpus: u64,
    tasks: u64,
    bytes_per_worker: u64,
    available_bytes: u64,
    shared_bytes: u64,
) -> Result<WorkerPlan> {
    if bytes_per_worker == 0 {
        return Err(Error::Invalid("worker memory estimate"));
    }
    if tasks == 0 {
        return Ok(WorkerPlan {
            workers: 0,
            reserved_bytes: 0,
        });
    }
    let available = available_bytes
        .checked_sub(shared_bytes)
        .ok_or(Error::Budget("shared worker memory"))?;
    let default = (logical_cpus / 5 * 3 + logical_cpus % 5 * 3 / 5).max(1);
    let workers = default.min(tasks).min(available / bytes_per_worker);
    if workers == 0 {
        return Err(Error::Budget("worker memory"));
    }
    let reserved_bytes = workers
        .checked_mul(bytes_per_worker)
        .and_then(|n| n.checked_add(shared_bytes))
        .ok_or(Error::Overflow("worker reservation"))?;
    Ok(WorkerPlan {
        workers,
        reserved_bytes,
    })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobToken {
    pub id: JobId,
    epoch: u64,
}
#[derive(Debug, Clone)]
pub struct WorkerInput {
    pub token: JobToken,
    pub request: Arc<AdmittedRequest>,
    pub cancel: Cancellation,
}
struct Pending {
    input: WorkerInput,
    reserved_bytes: u64,
}
#[derive(Default)]
pub struct JobManager {
    pending: BTreeMap<JobId, Pending>,
}
impl JobManager {
    pub fn start(
        &mut self,
        coordinator: &TransactionCoordinator,
        request: AdmittedRequest,
    ) -> Result<WorkerInput> {
        request.revalidate(coordinator.world())?;
        if self.pending.len() as u64 >= coordinator.limits().max_outputs {
            return Err(Error::Budget("pending jobs"));
        }
        // Conservative admission of independently held job snapshots. Arc
        // sharing avoids copying masters; opaque native memory is not included
        // unless its operation supplies a separate worker reservation.
        let reserved_bytes = coordinator.world().validate(coordinator.limits())?.bytes;
        let total = self.pending.values().try_fold(reserved_bytes, |n, p| {
            n.checked_add(p.reserved_bytes)
                .ok_or(Error::Overflow("job snapshots"))
        })?;
        if total > coordinator.limits().max_bytes {
            return Err(Error::Budget("aggregate job snapshots"));
        }
        let input = WorkerInput {
            token: JobToken {
                id: JobId::new(),
                epoch: coordinator.epoch(),
            },
            request: Arc::new(request),
            cancel: Cancellation::default(),
        };
        self.pending.insert(
            input.token.id,
            Pending {
                input: input.clone(),
                reserved_bytes,
            },
        );
        Ok(input)
    }
    pub fn complete<T>(
        &mut self,
        token: &JobToken,
        coordinator: &TransactionCoordinator,
        value: T,
    ) -> Result<T> {
        let pending = self
            .pending
            .remove(&token.id)
            .ok_or(Error::Stale("retired job"))?;
        if pending.input.token != *token || token.epoch != coordinator.epoch() {
            return Err(Error::Stale("job publication epoch"));
        }
        if pending.input.cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        pending.input.request.revalidate(coordinator.world())?;
        Ok(value)
    }
    pub fn cancel(&mut self, token: &JobToken) -> Result<()> {
        let p = self.pending.get(&token.id).ok_or(Error::Stale("job"))?;
        if p.input.token != *token {
            return Err(Error::Stale("job token"));
        }
        p.input.cancel.cancel();
        Ok(())
    }
    pub fn close_document(&mut self, document: DocumentId) {
        self.pending.retain(|_, p| {
            if p.input
                .request
                .inputs
                .iter()
                .any(|i| i.binding.source.document == document)
            {
                p.input.cancel.cancel();
                false
            } else {
                true
            }
        });
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
