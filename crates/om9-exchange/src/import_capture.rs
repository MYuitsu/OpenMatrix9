// SPDX-License-Identifier: LGPL-2.1-or-later
//! Frozen target ownership for a separately admitted import operation.
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::DocumentId,
    model::Document,
    retained::ImportBundle,
    transaction::{
        Cancellation, Mutation, OperationContext, PreparedTransaction, ScopeToken,
        TransactionCoordinator, TransactionRequest,
    },
};
use std::sync::Arc;

pub struct ImportCapture {
    document: Arc<Document>,
    context: OperationContext,
    owner: Option<ScopeToken>,
}
impl ImportCapture {
    pub fn target(coordinator: &TransactionCoordinator, document: DocumentId) -> Result<Self> {
        Self::capture(coordinator, document, None)
    }
    pub fn for_scope(
        coordinator: &TransactionCoordinator,
        document: DocumentId,
        owner: &ScopeToken,
    ) -> Result<Self> {
        Self::capture(coordinator, document, Some(owner))
    }
    fn capture(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        owner: Option<&ScopeToken>,
    ) -> Result<Self> {
        let (context, world) = coordinator.capture_context(owner)?;
        let document = world
            .documents
            .get(&id)
            .ok_or(Error::Missing("import document"))?;
        coordinator.retained_usage_with_held_usage(Usage::array::<Self>(1)?)?;
        Ok(Self {
            document: document.clone(),
            context,
            owner: owner.cloned(),
        })
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn revalidate(&self, coordinator: &TransactionCoordinator) -> Result<()> {
        if coordinator.authority() != self.context.authority() {
            return Err(Error::WrongOwner);
        }
        let (context, world) = coordinator.capture_context(self.owner.as_ref())?;
        if context != self.context {
            return Err(Error::Stale("import target operation"));
        }
        let current = world
            .documents
            .get(&self.document.id)
            .ok_or(Error::Missing("import target"))?;
        if !Arc::ptr_eq(current, &self.document)
            || current.owner_session != self.document.owner_session
            || current.session_generation != self.document.session_generation
            || current.epoch != self.document.epoch
            || current.context_revision != self.document.context_revision
            || current.revision != self.document.revision
        {
            return Err(Error::Stale("import target snapshot"));
        }
        Ok(())
    }
    pub fn prepare(
        &self,
        coordinator: &TransactionCoordinator,
        bundle: ImportBundle,
        cancel: &Cancellation,
    ) -> Result<PreparedTransaction> {
        self.prepare_with_held_usage(coordinator, bundle, cancel, Usage::default())
    }
    pub fn prepare_with_held_usage(
        &self,
        coordinator: &TransactionCoordinator,
        bundle: ImportBundle,
        cancel: &Cancellation,
        held: Usage,
    ) -> Result<PreparedTransaction> {
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        self.revalidate(coordinator)?;
        // Admit fixed request storage before boxing the already owned bundle.
        // Core then admits its contents and every prospective clone/Undo owner.
        let mut request_peak = held;
        request_peak.add(Usage::array::<ImportBundle>(1)?, coordinator.limits())?;
        request_peak.add(Usage::array::<Mutation>(1)?, coordinator.limits())?;
        request_peak.add(
            Usage {
                bytes: 6,
                ..Usage::default()
            },
            coordinator.limits(),
        )?;
        coordinator.retained_usage_with_held_usage(request_peak)?;
        let mut request = TransactionRequest::new(
            "Import",
            vec![Mutation::Import {
                document: self.document.id,
                expected_revision: self.document.revision,
                bundle: Box::new(bundle),
            }],
        );
        request.owner = self.owner.clone();
        request.cancel = cancel.clone();
        coordinator.prepare_with_held_usage(request, held)
    }
}
