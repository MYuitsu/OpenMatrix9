// SPDX-License-Identifier: LGPL-2.1-or-later
//! Actual serialized/reference/cold-SDK output identities, not decoded intent.
use crate::{HostValue, PreparedMergeInputs, RecordAction, RecordOverlay, ScopedNativeKey};
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::SourceStamp,
    retained::{HostRef, NativeRecordKind},
    transaction::{Cancellation, TransactionCoordinator},
};
use serde::Serialize;
use std::collections::BTreeMap;
use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OutputWireIdentity {
    pub native_component_type: u32,
    pub id: Uuid,
    pub serialized_index: Option<i32>,
    pub archive_reference_index: Option<i32>,
    pub sdk_destination_index: Option<i32>,
}
pub struct WireOutputPlan<'a> {
    inputs: &'a PreparedMergeInputs<'a>,
    sources: BTreeMap<ScopedNativeKey, Option<OutputWireIdentity>>,
    hosts: BTreeMap<HostRef, OutputWireIdentity>,
    tables: BTreeMap<u32, Vec<Uuid>>,
    default_layer: Option<OutputWireIdentity>,
    default_layer_name: Option<String>,
    owned: Usage,
}
impl WireOutputPlan<'_> {
    pub(crate) fn inputs(&self) -> &PreparedMergeInputs<'_> {
        self.inputs
    }
    pub fn source(&self, key: ScopedNativeKey) -> Option<Option<OutputWireIdentity>> {
        self.sources.get(&key).copied()
    }
    pub fn host(&self, key: HostRef) -> Option<OutputWireIdentity> {
        self.hosts.get(&key).copied()
    }
    pub fn table_order(&self, ty: u32) -> &[Uuid] {
        self.tables.get(&ty).map_or(&[], Vec::as_slice)
    }
    pub fn default_layer(&self) -> Option<OutputWireIdentity> {
        self.default_layer
    }
    pub fn default_layer_name(&self) -> Option<&str> {
        self.default_layer_name.as_deref()
    }
    pub fn owned_usage(&self) -> Usage {
        self.owned
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        self.inputs.revalidate(c, cancel)
    }
}
impl PreparedMergeInputs<'_> {
    pub fn output_wire_plan<'a>(
        &'a self,
        c: &TransactionCoordinator,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<WireOutputPlan<'a>> {
        self.revalidate(c, cancel)?;
        let intent = self.plan();
        let limits = c.limits();
        let r = intent.source_identities.len();
        let h = intent.host_identities.len();
        let entries = r
            .checked_add(h)
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Overflow("wire table entries"))?;
        let mut owned = Usage::array::<WireOutputPlan>(1)?;
        owned.add(
            Usage::btree::<ScopedNativeKey, Option<OutputWireIdentity>>(r)?,
            limits,
        )?;
        owned.add(Usage::btree::<HostRef, OutputWireIdentity>(h)?, limits)?;
        owned.add(Usage::btree::<u32, Vec<Uuid>>(13)?, limits)?;
        owned.add(
            Usage::array::<Uuid>(
                entries
                    .checked_mul(4)
                    .ok_or(Error::Overflow("wire table vector growth"))?,
            )?,
            limits,
        )?;
        owned.add(
            Usage {
                bytes: 256,
                ..Usage::default()
            },
            limits,
        )?;
        let mut scratch = Usage::btree::<ScopedNativeKey, &RecordOverlay>(r)?;
        scratch.add(
            Usage::array::<SourceStamp>(self.capture().sources.capacity())?,
            limits,
        )?;
        external.add(self.owned_usage(), limits)?;
        external.add(owned, limits)?;
        external.add(scratch, limits)?;
        // Charge already-held prepared inputs and every prospective map/table
        // allocation before coordinator/document validation scratch too.
        c.retained_usage_with_held_usage(external)?;
        let mut rows = BTreeMap::new();
        for ns in &intent.overlay.namespaces {
            for row in &ns.records {
                self.check_cancel(cancel)?;
                if rows
                    .insert(
                        ScopedNativeKey {
                            namespace: ns.namespace.id,
                            key: row.source.key,
                        },
                        row,
                    )
                    .is_some()
                {
                    return Err(Error::Invalid("duplicate wire source action"));
                }
            }
        }
        let mut sources = BTreeMap::new();
        let mut hosts = BTreeMap::new();
        let mut tables = BTreeMap::<u32, Vec<Uuid>>::new();
        let mut layer_serial: Option<i32> = None;
        let mut layer_reference = 0i32;
        let mut definition_reference = 0i32;
        let primary = intent.primary_namespace;
        let namespaces = intent
            .overlay
            .namespaces
            .iter()
            .filter(|n| Some(n.namespace.id) == primary)
            .chain(
                intent
                    .overlay
                    .namespaces
                    .iter()
                    .filter(|n| Some(n.namespace.id) != primary),
            );
        let mut default_needed = false;
        for ns in namespaces {
            self.check_cancel(cancel)?;
            let preparation = self
                .source(ns.namespace.id)
                .ok_or(Error::Missing("wire source preparation"))?;
            if preparation.source_sha256() != ns.source.sha256 {
                return Err(Error::Invalid("wire source preparation identity"));
            }
            let is_primary = Some(ns.namespace.id) == primary;
            let mut original_layers = 0i32;
            for original in preparation.records() {
                self.check_cancel(cancel)?;
                let key = ScopedNativeKey {
                    namespace: ns.namespace.id,
                    key: original.key()?,
                };
                let row = *rows
                    .get(&key)
                    .ok_or(Error::Missing("wire current source action"))?;
                let logical = *intent
                    .source_identities
                    .get(&key)
                    .ok_or(Error::Missing("wire logical source identity"))?;
                let ty = crate::output_identity::component_type(key.key.kind)?;
                if is_primary && ty == 5 {
                    if original.wire.archive_reference_index != Some(original_layers)
                        || original.wire.sdk_destination_index != Some(original_layers)
                    {
                        return Err(Error::Invalid("source layer reference ordinal"));
                    }
                    original_layers = original_layers
                        .checked_add(1)
                        .ok_or(Error::Overflow("source layer ordinal"))?;
                    layer_serial = maximum(layer_serial, original.wire.serialized_index);
                    if matches!(row.action, RecordAction::Delete | RecordAction::Omit) {
                        return Err(Error::Unsupported(
                            "layer removal requires closed settings/light/opaque reference remapping",
                        ));
                    }
                }
                let output = match logical {
                    None => None,
                    Some(logical) => {
                        if logical.native_component_type != ty {
                            return Err(Error::Invalid("wire component identity kind"));
                        }
                        let mut output = OutputWireIdentity {
                            native_component_type: ty,
                            id: logical.id,
                            serialized_index: None,
                            archive_reference_index: None,
                            sdk_destination_index: None,
                        };
                        if is_primary {
                            if logical.id != original.wire.id {
                                return Err(Error::Invalid("primary wire identity changed"));
                            }
                            output.serialized_index = original.wire.serialized_index;
                            output.archive_reference_index = original.wire.archive_reference_index;
                            output.sdk_destination_index = original.wire.sdk_destination_index;
                        } else if !matches!(ty, 5 | 11 | 12) {
                            return Err(Error::Unsupported(
                                "secondary native table requires an admitted remap serializer",
                            ));
                        }
                        match ty {
                            5 => {
                                if !is_primary {
                                    output.serialized_index =
                                        Some(append_layer(&mut layer_serial)?);
                                }
                                output.archive_reference_index = Some(layer_reference);
                                output.sdk_destination_index = Some(layer_reference);
                                layer_reference = layer_reference
                                    .checked_add(1)
                                    .ok_or(Error::Overflow("output layer ordinal"))?;
                            }
                            11 => {
                                output.serialized_index = None;
                                output.archive_reference_index = Some(definition_reference);
                                output.sdk_destination_index = Some(definition_reference);
                                definition_reference = definition_reference
                                    .checked_add(1)
                                    .ok_or(Error::Overflow("output definition ordinal"))?;
                            }
                            12 => {
                                output.serialized_index = None;
                                output.archive_reference_index = None;
                                output.sdk_destination_index = None;
                            }
                            _ if is_primary && row.action == RecordAction::Retain => {}
                            _ => {
                                return Err(Error::Unsupported(
                                    "selected output wire table action",
                                ));
                            }
                        }
                        tables.entry(ty).or_default().push(output.id);
                        if let Some(current) = row.current {
                            if hosts.insert(current.host(), output).is_some() {
                                return Err(Error::Invalid("duplicate output wire host"));
                            }
                            if let HostValue::Entity { entity, .. } = current {
                                default_needed |= entity.attributes.layer.is_none()
                                    && (row.action == RecordAction::Regenerate
                                        || row.changed.attributes);
                            }
                        }
                        Some(output)
                    }
                };
                if sources.insert(key, output).is_some() {
                    return Err(Error::Invalid("duplicate prepared source key"));
                }
            }
        }
        for current in &intent.overlay.additions {
            self.check_cancel(cancel)?;
            let logical = intent
                .host_identities
                .get(&current.host())
                .ok_or(Error::Missing("addition logical identity"))?;
            let (serialized, reference, sdk) = match logical.native_component_type {
                5 => {
                    let reference = layer_reference;
                    layer_reference = reference
                        .checked_add(1)
                        .ok_or(Error::Overflow("new layer ordinal"))?;
                    (
                        Some(append_layer(&mut layer_serial)?),
                        Some(reference),
                        Some(reference),
                    )
                }
                11 => {
                    let reference = definition_reference;
                    definition_reference = reference
                        .checked_add(1)
                        .ok_or(Error::Overflow("new definition ordinal"))?;
                    (None, Some(reference), Some(reference))
                }
                12 => (None, None, None),
                _ => return Err(Error::Unsupported("new wire native family")),
            };
            let output = OutputWireIdentity {
                native_component_type: logical.native_component_type,
                id: logical.id,
                serialized_index: serialized,
                archive_reference_index: reference,
                sdk_destination_index: sdk,
            };
            if hosts.insert(current.host(), output).is_some() {
                return Err(Error::Invalid("duplicate new wire host"));
            }
            tables
                .entry(output.native_component_type)
                .or_default()
                .push(output.id);
            if let HostValue::Entity { entity, .. } = current {
                default_needed |= entity.attributes.layer.is_none();
            }
        }
        if sources.len() != r || hosts.len() != h {
            return Err(Error::Invalid("incomplete output wire ownership"));
        }
        default_needed |= layer_reference == 0;
        let (default_layer, default_layer_name) = if default_needed {
            let (id, name) = fresh_default(self, &sources, &hosts, limits.max_name_bytes, cancel)?;
            let serial = append_layer(&mut layer_serial)?;
            let layer = OutputWireIdentity {
                native_component_type: 5,
                id,
                serialized_index: Some(serial),
                archive_reference_index: Some(layer_reference),
                sdk_destination_index: Some(layer_reference),
            };
            tables.entry(5).or_default().push(id);
            (Some(layer), Some(name))
        } else {
            (None, None)
        };
        self.revalidate(c, cancel)?;
        Ok(WireOutputPlan {
            inputs: self,
            sources,
            hosts,
            tables,
            default_layer,
            default_layer_name,
            owned,
        })
    }
}
fn maximum(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}
fn append_layer(last: &mut Option<i32>) -> Result<i32> {
    let next = match *last {
        None => 0,
        Some(i) => i
            .checked_add(1)
            .ok_or(Error::Overflow("serialized layer append index"))?,
    };
    *last = Some(next);
    Ok(next)
}
fn fresh_default(
    inputs: &PreparedMergeInputs<'_>,
    sources: &BTreeMap<ScopedNativeKey, Option<OutputWireIdentity>>,
    hosts: &BTreeMap<HostRef, OutputWireIdentity>,
    max_name_bytes: usize,
    cancel: &Cancellation,
) -> Result<(Uuid, String)> {
    for attempt in 0..16 {
        inputs.check_cancel(cancel)?;
        let name_bytes = "OM9 Unassigned".len() + if attempt == 0 { 0 } else { 1 + 36 };
        if name_bytes > max_name_bytes {
            return Err(Error::Budget("name bytes"));
        }
        let id = Uuid::new_v4();
        let mut used = inputs.capture().document().id.0 == id;
        for (key, value) in sources {
            inputs.check_cancel(cancel)?;
            used |= key.key.id == id || value.is_some_and(|v| v.id == id);
        }
        for (key, value) in hosts {
            inputs.check_cancel(cancel)?;
            used |= host_uuid(*key) == id || value.id == id;
        }
        let name = if attempt == 0 {
            "OM9 Unassigned".to_owned()
        } else {
            format!("OM9 Unassigned {id}")
        };
        let mut occupied = false;
        for layer in inputs.capture().document().layers.values() {
            inputs.check_cancel(cancel)?;
            occupied |= layer.name.eq_ignore_ascii_case(&name);
        }
        for ns in &inputs.plan().overlay.namespaces {
            inputs.check_cancel(cancel)?;
            let preparation = inputs
                .source(ns.namespace.id)
                .ok_or(Error::Missing("generated layer source preparation"))?;
            for record in preparation.records() {
                inputs.check_cancel(cancel)?;
                occupied |= record.key()?.kind == NativeRecordKind::Layer
                    && record.name.eq_ignore_ascii_case(&name);
            }
        }
        if !used && !occupied {
            return Ok((id, name));
        }
    }
    Err(Error::Invalid("generated native layer identity collision"))
}
fn host_uuid(host: HostRef) -> Uuid {
    match host {
        HostRef::Object(id) => id.0,
        HostRef::Layer(id) => id.0,
        HostRef::Definition(id) => id.0,
        HostRef::DefinitionMember { object, .. } => object.0,
    }
}
