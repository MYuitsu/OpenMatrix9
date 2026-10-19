// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::scene::Scene;
use om9_core::{
    Error, Result,
    geometry::Transform,
    identity::*,
    transaction::{Mutation, PreparedTransaction, TransactionCoordinator, TransactionRequest},
};
use std::collections::BTreeSet;
pub struct AffineGesture {
    document: DocumentId,
    epoch: u64,
    sources: Vec<(SourceStamp, Transform)>,
    delta: Transform,
    active: bool,
}
impl AffineGesture {
    pub fn begin(
        c: &TransactionCoordinator,
        document: DocumentId,
        objects: &[ObjectId],
    ) -> Result<Self> {
        let d = c
            .world()
            .documents
            .get(&document)
            .ok_or(Error::Missing("gesture document"))?;
        let mut seen = BTreeSet::new();
        let mut sources = vec![];
        if objects.is_empty() {
            return Err(Error::Invalid("empty gesture selection"));
        }
        for id in objects {
            if !seen.insert(*id) {
                return Err(Error::Invalid("duplicate gesture source"));
            }
            if !d.permissions(*id, false)?.editable || d.history.locked(*id) {
                return Err(Error::Permission("gesture source"));
            }
            let source = d.stamp(*id)?;
            sources.push((source, d.resolve(&source)?.placement));
        }
        Ok(Self {
            document,
            epoch: c.epoch(),
            sources,
            delta: Transform::identity(),
            active: true,
        })
    }
    fn revalidate(&self, c: &TransactionCoordinator) -> Result<()> {
        if !self.active {
            return Err(Error::Canceled);
        }
        if c.epoch() != self.epoch {
            return Err(Error::Stale("gesture publication epoch"));
        }
        let d = c
            .world()
            .documents
            .get(&self.document)
            .ok_or(Error::Missing("gesture owner"))?;
        for (stamp, _) in &self.sources {
            d.resolve(stamp)?;
            if !d.permissions(stamp.object, false)?.editable {
                return Err(Error::Permission("gesture source"));
            }
        }
        Ok(())
    }
    pub fn update(
        &mut self,
        c: &TransactionCoordinator,
        scene: &mut Scene,
        delta: Transform,
    ) -> Result<()> {
        self.revalidate(c)?;
        delta.validate()?;
        if scene.epoch != self.epoch || scene.document.id != self.document {
            return Err(Error::Stale("gesture scene"));
        }
        for object in &scene.objects {
            if self.sources.iter().any(|(s, _)| s.object == object.root) {
                delta.compose(object.transform)?;
            }
        }
        for object in &mut scene.objects {
            if self.sources.iter().any(|(s, _)| s.object == object.root) {
                object.preview = delta;
            }
        }
        self.delta = delta;
        Ok(())
    }
    pub fn cancel(&mut self, scene: &mut Scene) -> Result<()> {
        if scene.document.id != self.document {
            return Err(Error::Stale("gesture scene owner"));
        }
        for object in &mut scene.objects {
            if self.sources.iter().any(|(s, _)| s.object == object.root) {
                object.preview = Transform::identity();
            }
        }
        self.active = false;
        Ok(())
    }
    pub fn prepare(&self, c: &TransactionCoordinator) -> Result<PreparedTransaction> {
        self.revalidate(c)?;
        let mut mutations = vec![];
        for (source, original) in &self.sources {
            let placement = self.delta.compose(*original)?;
            if placement != *original {
                mutations.push(Mutation::Placement {
                    source: *source,
                    placement,
                });
            }
        }
        c.prepare(TransactionRequest::new("Gumball", mutations))
    }
}
