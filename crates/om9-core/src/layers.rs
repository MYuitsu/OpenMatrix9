// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned layer mutations use the coordinator's session and publication barrier.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    geometry::Geometry,
    identity::{DocumentId, LayerId, ObjectId, SessionId, SourceStamp},
    model::{Document, Layer},
    retained::HostRef,
    transaction::{Cancellation, OperationContext, ScopeToken, TransactionCoordinator},
};
use std::collections::BTreeSet;
mod category;
mod preset;
pub use category::{LayerCategory, PaletteGroup};
pub use preset::PresetSlot;
pub(crate) use preset::same_name;
pub(crate) use preset::valid_label;

#[derive(Debug, Clone)]
pub enum LayerAction {
    Create {
        layer: Layer,
        make_current: bool,
    },
    Update {
        layer: Layer,
    },
    SetCurrent {
        layer: Option<LayerId>,
    },
    Assign {
        layer: LayerId,
        sources: Vec<SourceStamp>,
    },
    Hidden {
        sources: Vec<SourceStamp>,
        hidden: bool,
    },
}
#[derive(Debug, Clone)]
pub struct LayerPlan {
    context: OperationContext,
    document: DocumentId,
    owner: (SessionId, u64, u64, u64, u64),
    action: LayerAction,
    changed: bool,
}
impl LayerPlan {
    pub fn capture(
        c: &TransactionCoordinator,
        document: DocumentId,
        action: LayerAction,
        caller: Option<&ScopeToken>,
    ) -> Result<Self> {
        if let LayerAction::Assign { sources, .. } | LayerAction::Hidden { sources, .. } = &action
            && sources.len() as u64 > c.limits().max_outputs
        {
            return Err(Error::Budget("layer assignment count"));
        }
        let (context, world) = c.capture_context(caller)?;
        let d = world
            .documents
            .get(&document)
            .ok_or(Error::Missing("layer document"))?;
        let changed = match &action {
            LayerAction::Create { .. } => true,
            LayerAction::Update { layer } => d.layers.get(&layer.id) != Some(layer),
            LayerAction::SetCurrent { layer } => d.current_layer != *layer,
            LayerAction::Assign { layer, sources } => sources.iter().any(|s| {
                d.entities.get(&s.object).is_none_or(|e| {
                    e.attributes.layer != Some(*layer) || e.attributes.color.is_some()
                })
            }),
            LayerAction::Hidden { sources, hidden } => sources.iter().any(|s| {
                d.entities
                    .get(&s.object)
                    .is_none_or(|e| e.attributes.hidden != *hidden)
            }),
        };
        Ok(Self {
            context,
            document,
            owner: Self::owner(d),
            action,
            changed,
        })
    }
    fn owner(d: &Document) -> (SessionId, u64, u64, u64, u64) {
        (
            d.owner_session,
            d.epoch,
            d.session_generation,
            d.revision,
            d.context_revision,
        )
    }
    pub(crate) fn document(&self) -> DocumentId {
        self.document
    }
    pub(crate) fn changed(&self) -> bool {
        self.changed
    }
    pub(crate) fn created_id(&self) -> Option<LayerId> {
        if let LayerAction::Create { layer, .. } = &self.action {
            Some(layer.id)
        } else {
            None
        }
    }
    pub(crate) fn created_layer(&self) -> Option<&Layer> {
        if let LayerAction::Create { layer, .. } = &self.action {
            Some(layer)
        } else {
            None
        }
    }
    pub(crate) fn assignment_target(&self) -> Option<LayerId> {
        if let LayerAction::Assign { layer, .. } = &self.action {
            Some(*layer)
        } else {
            None
        }
    }
    pub(crate) fn proposed_layer(&self) -> Option<&Layer> {
        match &self.action {
            LayerAction::Create { layer, .. } | LayerAction::Update { layer } => Some(layer),
            _ => None,
        }
    }
    pub(crate) fn has_custom_label(&self) -> bool {
        self.proposed_layer().is_some_and(|layer| {
            layer
                .preset_slot
                .is_some_and(|slot| slot.is_custom() && layer.name != slot.name())
        })
    }
    pub(crate) fn storage_usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage::array::<Self>(1)?;
        match &self.action {
            LayerAction::Create { layer, .. } | LayerAction::Update { layer } => u.add(
                Usage {
                    bytes: layer.name.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?,
            LayerAction::Assign { sources, .. } | LayerAction::Hidden { sources, .. } => {
                u.add(Usage::array::<SourceStamp>(sources.capacity())?, l)?;
                u.add(
                    Usage {
                        outputs: sources.len() as u64,
                        ..Usage::default()
                    },
                    l,
                )?;
            }
            _ => {}
        }
        Ok(u)
    }
    pub(crate) fn admit(
        &self,
        context: OperationContext,
        d: &Document,
        new_target: Option<&Layer>,
        l: &Limits,
        peak: &mut Usage,
        cancel: &Cancellation,
    ) -> Result<()> {
        if self.context != context || self.document != d.id || self.owner != Self::owner(d) {
            return Err(Error::Stale("layer document/session/context"));
        }
        match &self.action {
            LayerAction::Create {
                layer,
                make_current,
            } => {
                if layer.id.is_nil() || layer.revision != 1 || d.layers.contains_key(&layer.id) {
                    return Err(Error::Invalid("new layer identity/revision"));
                }
                name(&layer.name, l)?;
                if let Some(slot) = layer.preset_slot {
                    if !valid_label(slot, &layer.name)
                        || (!slot.is_custom() && layer.color != slot.color())
                    {
                        return Err(Error::Invalid("OM9 preset label/palette"));
                    }
                    for other in d.layers.values() {
                        check_cancel(cancel)?;
                        if other.preset_slot == Some(slot) {
                            return Err(Error::Invalid("duplicate OM9 preset slot"));
                        }
                        if slot.is_custom()
                            && layer.name != slot.name()
                            && same_name(&layer.name, &other.name)
                        {
                            return Err(Error::Invalid("duplicate layer label"));
                        }
                    }
                }
                let editable = editable_layer(d, layer.parent)?;
                if *make_current && (!editable || layer.locked) {
                    return Err(Error::Permission("new current layer"));
                }
                peak.add(
                    Usage::btree::<LayerId, Layer>(
                        d.layers
                            .len()
                            .checked_add(1)
                            .ok_or(Error::Overflow("layer count"))?,
                    )?,
                    l,
                )?;
                peak.add(
                    Usage::array::<LayerId>(
                        d.layer_order
                            .len()
                            .checked_add(1)
                            .and_then(|n| n.checked_mul(2))
                            .ok_or(Error::Overflow("layer order"))?
                            .max(4),
                    )?,
                    l,
                )?;
                peak.add(
                    Usage {
                        bytes: std::mem::size_of::<Layer>() as u64 + layer.name.capacity() as u64,
                        nodes: 1,
                        ..Usage::default()
                    },
                    l,
                )?;
            }
            LayerAction::Update { layer } => {
                let old = d
                    .layers
                    .get(&layer.id)
                    .ok_or(Error::Missing("updated layer"))?;
                if old.revision != layer.revision {
                    return Err(Error::Stale("layer revision"));
                }
                if old.preset_slot != layer.preset_slot {
                    return Err(Error::Invalid("stable preset identity"));
                }
                if let Some(slot) = layer.preset_slot {
                    if !slot.is_custom() && layer.color != old.color {
                        return Err(Error::Permission(
                            "only Advanced preset color customization",
                        ));
                    }
                    if layer.name != old.name {
                        if !slot.is_custom() {
                            return Err(Error::Permission(
                                "only Advanced preset name customization",
                            ));
                        }
                        for (id, other) in &d.layers {
                            check_cancel(cancel)?;
                            if *id != layer.id && preset::same_name(&layer.name, &other.name) {
                                return Err(Error::Invalid("duplicate layer label"));
                            }
                        }
                    }
                }
                name(&layer.name, l)?;
                if self.changed {
                    layer
                        .revision
                        .checked_add(1)
                        .ok_or(Error::Overflow("layer revision"))?;
                }
                if let Some(parent) = layer.parent {
                    d.layers
                        .get(&parent)
                        .ok_or(Error::Missing("layer parent"))?;
                }
            }
            LayerAction::SetCurrent { layer } => {
                if !editable_layer(d, *layer)? {
                    return Err(Error::Permission("current layer"));
                }
            }
            LayerAction::Assign { layer, sources } => {
                let editable = if let Some(created) =
                    new_target.filter(|_| !d.layers.contains_key(layer))
                {
                    created.id == *layer && !created.locked && editable_layer(d, created.parent)?
                } else {
                    editable_layer(d, Some(*layer))?
                };
                if !editable {
                    return Err(Error::Permission("assigned layer"));
                }
                peak.add(Usage::btree::<ObjectId, ()>(sources.len())?, l)?;
                let mut written = BTreeSet::new();
                for source in sources {
                    check_cancel(cancel)?;
                    let e = d.resolve(source)?;
                    if !written.insert(e.id) {
                        return Err(Error::Invalid("duplicate layer assignment"));
                    }
                    if e.attributes.locked || !editable_layer(d, e.attributes.layer)? {
                        return Err(Error::Permission("assigned source"));
                    }
                    if matches!(e.geometry, Geometry::Instance(_)) {
                        return Err(Error::Unsupported(
                            "structural/retained layer assignment requires proven ownership policy",
                        ));
                    }
                    if e.attributes.layer != Some(*layer) || e.attributes.color.is_some() {
                        e.attributes_revision
                            .checked_add(1)
                            .ok_or(Error::Overflow("attributes revision"))?;
                    }
                }
                // One pass over retained bindings, using the already admitted
                // source set. Do not rescan every retained row for each source.
                for record in d.import_namespaces.values().flat_map(|ns| &ns.records) {
                    check_cancel(cancel)?;
                    if let Some(HostRef::Object(id)) = record.host
                        && written.contains(&id)
                    {
                        return Err(Error::Unsupported(
                            "structural/retained layer assignment requires proven ownership policy",
                        ));
                    }
                }
            }
            LayerAction::Hidden { sources, hidden } => {
                peak.add(Usage::btree::<ObjectId, ()>(sources.len())?, l)?;
                let mut written = BTreeSet::new();
                for source in sources {
                    check_cancel(cancel)?;
                    let e = d.resolve(source)?;
                    if !written.insert(e.id) {
                        return Err(Error::Invalid("duplicate Hidden write"));
                    }
                    if e.attributes.hidden != *hidden {
                        e.attributes_revision
                            .checked_add(1)
                            .ok_or(Error::Overflow("attributes revision"))?;
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn assigned_sources(&self) -> &[SourceStamp] {
        if let LayerAction::Assign { sources, .. } | LayerAction::Hidden { sources, .. } =
            &self.action
        {
            sources
        } else {
            &[]
        }
    }
    pub(crate) fn updated_id(&self) -> Option<LayerId> {
        if let LayerAction::Update { layer } = &self.action {
            Some(layer.id)
        } else {
            None
        }
    }
    pub(crate) fn changes_current(&self) -> bool {
        matches!(
            self.action,
            LayerAction::SetCurrent { .. }
                | LayerAction::Create {
                    make_current: true,
                    ..
                }
        )
    }
    pub(crate) fn check_permission_reads(
        &self,
        d: &Document,
        new_target: Option<&Layer>,
        written: &BTreeSet<(DocumentId, LayerId)>,
        cancel: &Cancellation,
    ) -> Result<()> {
        if written.is_empty() {
            return Ok(());
        }
        match &self.action {
            LayerAction::SetCurrent { layer } => check_layer_reads(d, *layer, written)?,
            LayerAction::Create {
                layer,
                make_current: true,
            } => check_layer_reads(d, layer.parent, written)?,
            LayerAction::Assign { layer, sources } => {
                if let Some(created) = new_target.filter(|_| !d.layers.contains_key(layer)) {
                    check_layer_reads(d, created.parent, written)?;
                } else {
                    check_layer_reads(d, Some(*layer), written)?;
                }
                for source in sources {
                    check_cancel(cancel)?;
                    check_layer_reads(d, d.resolve(source)?.attributes.layer, written)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub(crate) fn apply(self, d: &mut Document, cancel: &Cancellation) -> Result<()> {
        match self.action {
            LayerAction::Hidden { sources, hidden } => {
                for source in sources {
                    check_cancel(cancel)?;
                    if d.entities
                        .get(&source.object)
                        .ok_or(Error::Missing("hidden object"))?
                        .attributes
                        .hidden
                        != hidden
                    {
                        d.invalidate_dependents(source.object, false)?;
                        let e = d
                            .entities
                            .get_mut(&source.object)
                            .ok_or(Error::Missing("hidden object"))?;
                        e.attributes_revision = e
                            .attributes_revision
                            .checked_add(1)
                            .ok_or(Error::Overflow("attributes revision"))?;
                        e.attributes.hidden = hidden;
                    }
                }
            }
            LayerAction::Create {
                layer,
                make_current,
            } => {
                let id = layer.id;
                d.layer_order
                    .try_reserve(1)
                    .map_err(|_| Error::Budget("layer order allocation"))?;
                d.layer_order.push(id);
                if make_current {
                    d.current_layer = Some(id);
                }
                if d.layers.insert(id, layer).is_some() {
                    return Err(Error::Invalid("duplicate layer"));
                }
            }
            LayerAction::Update { mut layer } => {
                if d.layers.get(&layer.id) != Some(&layer) {
                    layer.revision = layer
                        .revision
                        .checked_add(1)
                        .ok_or(Error::Overflow("layer revision"))?;
                    d.layers.insert(layer.id, layer);
                }
            }
            LayerAction::SetCurrent { layer } => d.current_layer = layer,
            LayerAction::Assign { layer, sources } => {
                for source in sources {
                    check_cancel(cancel)?;
                    let changed = d
                        .entities
                        .get(&source.object)
                        .ok_or(Error::Missing("assigned object"))?
                        .attributes
                        .layer
                        != Some(layer)
                        || d.entities[&source.object].attributes.color.is_some();
                    if changed {
                        d.invalidate_dependents(source.object, false)?;
                        let e = d
                            .entities
                            .get_mut(&source.object)
                            .ok_or(Error::Missing("assigned object"))?;
                        e.attributes_revision = e
                            .attributes_revision
                            .checked_add(1)
                            .ok_or(Error::Overflow("attributes revision"))?;
                        e.attributes.layer = Some(layer);
                        e.attributes.color = None;
                    }
                }
            }
        }
        Ok(())
    }
}

fn check_cancel(cancel: &Cancellation) -> Result<()> {
    if cancel.is_canceled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}
// Base document ownership/cycles were admitted by the coordinator. This
// bounded borrowed walk avoids allocating a layer-chain set for each source.
fn editable_layer(d: &Document, mut layer: Option<LayerId>) -> Result<bool> {
    for _ in 0..=d.layers.len() {
        let Some(id) = layer else { return Ok(true) };
        let value = d
            .layers
            .get(&id)
            .ok_or(Error::Missing("layer permission chain"))?;
        if value.locked {
            return Ok(false);
        }
        layer = value.parent;
    }
    Err(Error::Invalid("layer permission cycle"))
}

pub(crate) fn check_layer_reads(
    d: &Document,
    layer: Option<LayerId>,
    written: &BTreeSet<(DocumentId, LayerId)>,
) -> Result<()> {
    check_layer_reads_with(d, layer, written, |id| d.layers.get(&id))
}

/// The same guard resolved through a caller lookup, so a chain that includes a
/// layer proposed by this batch can be walked and its captured writes excluded.
pub(crate) fn check_layer_reads_with<'l>(
    d: &Document,
    mut layer: Option<LayerId>,
    written: &BTreeSet<(DocumentId, LayerId)>,
    resolve: impl Fn(LayerId) -> Option<&'l Layer>,
) -> Result<()> {
    if written.is_empty() {
        return Ok(());
    }
    while let Some(id) = layer {
        if written.contains(&(d.id, id)) {
            return Err(Error::Invalid(
                "layer read/write conflict within transaction",
            ));
        }
        layer = resolve(id)
            .ok_or(Error::Missing("layer permission chain"))?
            .parent;
    }
    Ok(())
}
