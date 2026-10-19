// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rust-authored retained writer request, separate from native admission.
use crate::{HostValue, OutputWireIdentity, ScopedNativeKey, WireOutputPlan};
use om9_core::{
    Error, Result,
    budget::Usage,
    geometry::{Geometry, Transform, Unit},
    identity::SourceStamp,
    model::{Definition, Entity},
    retained::{HostRef, NativeRecordKind},
    transaction::{Cancellation, TransactionCoordinator},
};
use serde::{
    Serialize, Serializer,
    ser::{Error as _, SerializeMap, SerializeSeq, SerializeStruct},
};
use std::{
    fmt,
    io::{self, Write},
};
use uuid::Uuid;

struct Hex<'a>(&'a [u8; 32]);
impl fmt::Display for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
impl Serialize for Hex<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}
#[derive(Serialize)]
struct CurrentLayer {
    policy: &'static str,
    id: Option<Uuid>,
    archive_reference_index: Option<i32>,
}
fn current_layer(wire: &WireOutputPlan<'_>, cancel: &Cancellation) -> Result<CurrentLayer> {
    let document = wire.inputs().capture().document();
    let explicit = if let Some(id) = document.current_layer {
        Some(
            wire.host(HostRef::Layer(id))
                .ok_or(Error::Missing("current output layer"))?,
        )
    } else if wire.inputs().plan().primary_namespace.is_some() {
        return Ok(CurrentLayer {
            policy: "preserve_primary",
            id: None,
            archive_reference_index: None,
        });
    } else if let Some(default) = wire.default_layer() {
        Some(default)
    } else {
        let id = wire
            .table_order(5)
            .first()
            .ok_or(Error::Missing("native current layer policy"))?;
        let mut found = None;
        for ns in &wire.inputs().plan().overlay.namespaces {
            for row in &ns.records {
                wire.inputs().check_cancel(cancel)?;
                let key = ScopedNativeKey {
                    namespace: ns.namespace.id,
                    key: row.source.key,
                };
                if let Some(Some(output)) = wire.source(key)
                    && output.id == *id
                {
                    found = Some(output);
                }
            }
        }
        for host in &wire.inputs().plan().overlay.additions {
            wire.inputs().check_cancel(cancel)?;
            if let Some(output) = wire.host(host.host())
                && output.id == *id
            {
                found = Some(output);
            }
        }
        found
    };
    let output = explicit.ok_or(Error::Missing("output current layer"))?;
    if output.native_component_type != 5 || output.archive_reference_index.is_none() {
        return Err(Error::Invalid("current layer wire context"));
    }
    Ok(CurrentLayer {
        policy: "explicit_output",
        id: Some(output.id),
        archive_reference_index: output.archive_reference_index,
    })
}
#[derive(Serialize)]
struct Attributes {
    layer_index: i32,
    visible: bool,
    selectable: bool,
    locked: bool,
    color: Option<[u8; 4]>,
}
struct NativeGeometry<'a> {
    geometry: &'a Geometry,
    instance: Option<Uuid>,
}
impl Serialize for NativeGeometry<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        if let Some(id) = self.instance {
            let mut map = s.serialize_map(Some(1))?;
            map.serialize_entry("Instance", &id)?;
            map.end()
        } else {
            self.geometry.serialize(s)
        }
    }
}
#[derive(Serialize)]
struct NativeEntity<'a> {
    id: Uuid,
    label: &'a str,
    geometry: NativeGeometry<'a>,
    placement: Transform,
    attributes: Attributes,
    definition_member: bool,
}
#[derive(Serialize)]
struct NativeLayer<'a> {
    id: Uuid,
    name: &'a str,
    parent: Option<Uuid>,
    visible: bool,
    selectable: bool,
    locked: bool,
    color: [u8; 4],
}
struct MemberIds<'a> {
    wire: &'a WireOutputPlan<'a>,
    definition: &'a Definition,
}
impl Serialize for MemberIds<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.definition.members.len()))?;
        for member in &self.definition.members {
            let host = HostRef::DefinitionMember {
                definition: self.definition.id,
                object: member.id,
            };
            let output = self
                .wire
                .host(host)
                .ok_or_else(|| S::Error::custom("output definition member"))?;
            seq.serialize_element(&output.id)?;
        }
        seq.end()
    }
}
#[derive(Serialize)]
struct NativeDefinition<'a> {
    id: Uuid,
    label: &'a str,
    mm_per_unit: f64,
    members: MemberIds<'a>,
}
#[derive(Serialize)]
enum Current<'a> {
    Entity(NativeEntity<'a>),
    Layer(NativeLayer<'a>),
    Definition(NativeDefinition<'a>),
}
fn current<'a>(wire: &'a WireOutputPlan<'a>, host: HostValue<'a>) -> Result<Current<'a>> {
    let output = wire
        .host(host.host())
        .ok_or(Error::Missing("current payload output"))?;
    Ok(match host {
        HostValue::Entity { host, entity } => {
            let layer = match entity.attributes.layer {
                Some(id) => wire.host(HostRef::Layer(id)),
                None => wire.default_layer(),
            };
            let layer_index = layer
                .and_then(|layer| layer.archive_reference_index)
                .ok_or(Error::Missing("current entity native layer"))?;
            let instance = if let Geometry::Instance(id) = entity.geometry {
                Some(
                    wire.host(HostRef::Definition(id))
                        .ok_or(Error::Missing("current instance native definition"))?
                        .id,
                )
            } else {
                None
            };
            Current::Entity(NativeEntity {
                id: output.id,
                label: &entity.label,
                geometry: NativeGeometry {
                    geometry: &entity.geometry,
                    instance,
                },
                placement: wire.inputs().plan().native_placement(host, entity)?,
                attributes: Attributes {
                    layer_index,
                    visible: entity.attributes.visible && !entity.attributes.hidden,
                    selectable: entity.attributes.selectable,
                    locked: entity.attributes.locked,
                    color: entity.attributes.color,
                },
                definition_member: matches!(host, HostRef::DefinitionMember { .. }),
            })
        }
        HostValue::Layer(layer) => Current::Layer(NativeLayer {
            id: output.id,
            name: &layer.name,
            parent: layer
                .parent
                .map(|id| {
                    wire.host(HostRef::Layer(id))
                        .map(|out| out.id)
                        .ok_or(Error::Missing("current layer native parent"))
                })
                .transpose()?,
            visible: layer.visible,
            selectable: layer.selectable,
            locked: layer.locked,
            color: layer.color,
        }),
        HostValue::Definition(definition) => Current::Definition(NativeDefinition {
            id: output.id,
            label: &definition.label,
            mm_per_unit: *wire
                .inputs()
                .plan()
                .definition_mm_per_unit
                .get(&definition.id)
                .ok_or(Error::Missing("current definition units"))?,
            members: MemberIds { wire, definition },
        }),
    })
}
#[derive(Serialize)]
struct Correlation<'a> {
    id: om9_core::identity::DocumentId,
    name: &'a str,
    page_unit: Unit,
    current_layer: CurrentLayer,
}
struct ExpectedWire<'a>(&'a crate::WireIdentity);
impl Serialize for ExpectedWire<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let w = self.0;
        let mut st = s.serialize_struct("ExpectedWire", 9)?;
        st.serialize_field("id", &w.id)?;
        st.serialize_field("serialized_index", &w.serialized_index)?;
        st.serialize_field("archive_reference_index", &w.archive_reference_index)?;
        st.serialize_field("sdk_destination_index", &w.sdk_destination_index)?;
        st.serialize_field("table_type", &w.table_type)?;
        st.serialize_field("record_type", &w.record_type)?;
        st.serialize_field("offset", &w.offset)?;
        st.serialize_field("byte_length", &w.byte_length)?;
        st.serialize_field("sha256", &Hex(&w.sha256))?;
        st.end()
    }
}
#[derive(Serialize)]
struct SourceKey {
    native_component_type: u32,
    id: Uuid,
    native_index: Option<i32>,
}
fn action(action: crate::RecordAction) -> &'static str {
    use crate::RecordAction::*;
    match action {
        Retain => "retain",
        Delete => "delete",
        Omit => "omit",
        Regenerate => "regenerate",
        PatchEntity => "patch_entity",
        PatchLayer => "patch_layer",
        PatchDefinition => "patch_definition",
    }
}
struct Records<'a> {
    wire: &'a WireOutputPlan<'a>,
    ns: &'a crate::NamespaceOverlay<'a>,
    cancel: &'a Cancellation,
}
impl Serialize for Records<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let preparation = self
            .wire
            .inputs()
            .source(self.ns.namespace.id)
            .ok_or_else(|| S::Error::custom("source preparation"))?;
        let mut seq = s.serialize_seq(Some(self.ns.records.len()))?;
        for row in &self.ns.records {
            self.wire
                .inputs()
                .check_cancel(self.cancel)
                .map_err(S::Error::custom)?;
            let source = row.source.key;
            let original = preparation
                .record(source)
                .ok_or_else(|| S::Error::custom("prepared original record"))?;
            let ty =
                crate::output_identity::component_type(source.kind).map_err(S::Error::custom)?;
            let output = self
                .wire
                .source(ScopedNativeKey {
                    namespace: self.ns.namespace.id,
                    key: source,
                })
                .ok_or_else(|| S::Error::custom("source wire output"))?;
            let current = row
                .current
                .map(|host| current(self.wire, host))
                .transpose()
                .map_err(S::Error::custom)?;
            let definition_mm_per_unit =
                if source.kind == NativeRecordKind::Definition && output.is_some() {
                    if let Some(HostValue::Definition(definition)) = row.current {
                        self.wire
                            .inputs()
                            .plan()
                            .definition_mm_per_unit
                            .get(&definition.id)
                            .copied()
                    } else {
                        self.ns
                            .source
                            .units
                            .definitions
                            .get(&source.id)
                            .and_then(|units| units.millimeters_per_unit)
                    }
                } else {
                    None
                };
            #[derive(Serialize)]
            struct Changed {
                geometry: bool,
                placement: bool,
                attributes: bool,
                label: bool,
            }
            #[derive(Serialize)]
            struct Row<'a> {
                source: SourceKey,
                expected_wire: ExpectedWire<'a>,
                action: &'static str,
                changed: Changed,
                needs_cache_refresh: bool,
                output: Option<OutputWireIdentity>,
                definition_mm_per_unit: Option<f64>,
                current: Option<Current<'a>>,
            }
            let row = Row {
                source: SourceKey {
                    native_component_type: ty,
                    id: source.id,
                    native_index: source.native_index,
                },
                expected_wire: ExpectedWire(&original.wire),
                action: action(row.action),
                changed: Changed {
                    geometry: row.changed.geometry,
                    placement: row.changed.placement,
                    attributes: row.changed.attributes,
                    label: row.changed.label,
                },
                needs_cache_refresh: row.needs_cache_refresh,
                output,
                definition_mm_per_unit,
                current,
            };
            seq.serialize_element(&row)?;
        }
        seq.end()
    }
}
struct Sources<'a> {
    wire: &'a WireOutputPlan<'a>,
    cancel: &'a Cancellation,
}
impl Serialize for Sources<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.wire.inputs().plan().overlay.namespaces.len()))?;
        for ns in &self.wire.inputs().plan().overlay.namespaces {
            self.wire
                .inputs()
                .check_cancel(self.cancel)
                .map_err(S::Error::custom)?;
            #[derive(Serialize)]
            struct Source<'a> {
                namespace: om9_core::identity::ImportNamespaceId,
                source_id: om9_core::identity::SourceId,
                sha256: Hex<'a>,
                model_mm_per_unit: f64,
                page_mm_per_unit: f64,
                records: Records<'a>,
            }
            seq.serialize_element(&Source {
                namespace: ns.namespace.id,
                source_id: ns.source.id,
                sha256: Hex(&ns.source.sha256),
                model_mm_per_unit: ns.source.units.model.millimeters_per_unit,
                page_mm_per_unit: ns.source.units.page.millimeters_per_unit,
                records: Records {
                    wire: self.wire,
                    ns,
                    cancel: self.cancel,
                },
            })?;
        }
        seq.end()
    }
}
struct Additions<'a> {
    wire: &'a WireOutputPlan<'a>,
    cancel: &'a Cancellation,
}
impl Serialize for Additions<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.wire.inputs().plan().overlay.additions.len()))?;
        for host in &self.wire.inputs().plan().overlay.additions {
            self.wire
                .inputs()
                .check_cancel(self.cancel)
                .map_err(S::Error::custom)?;
            let output = self
                .wire
                .host(host.host())
                .ok_or_else(|| S::Error::custom("addition wire output"))?;
            let current = current(self.wire, *host).map_err(S::Error::custom)?;
            let needs_cache_refresh = matches!(host, HostValue::Definition(_))
                || matches!(
                    host,
                    HostValue::Entity {
                        entity: Entity {
                            geometry: Geometry::Instance(_),
                            ..
                        },
                        ..
                    }
                );
            #[derive(Serialize)]
            struct Addition<'a> {
                output: OutputWireIdentity,
                needs_cache_refresh: bool,
                current: Current<'a>,
            }
            seq.serialize_element(&Addition {
                output,
                needs_cache_refresh,
                current,
            })?;
        }
        seq.end()
    }
}
struct Tables<'a>(&'a WireOutputPlan<'a>);
impl Serialize for Tables<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        for ty in 1..=13 {
            let order = self.0.table_order(ty);
            if !order.is_empty() {
                map.serialize_entry(&ty, order)?;
            }
        }
        map.end()
    }
}
struct Packet<'a> {
    wire: &'a WireOutputPlan<'a>,
    cancel: &'a Cancellation,
}
impl Serialize for Packet<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.wire
            .inputs()
            .check_cancel(self.cancel)
            .map_err(S::Error::custom)?;
        let d = self.wire.inputs().capture().document();
        let primary = self.wire.inputs().plan().primary_namespace;
        #[derive(Serialize)]
        struct Default<'a> {
            output: OutputWireIdentity,
            needs_cache_refresh: bool,
            current: Current<'a>,
        }
        let default = self.wire.default_layer().map(|output| Default {
            output,
            needs_cache_refresh: false,
            current: Current::Layer(NativeLayer {
                id: output.id,
                name: self.wire.default_layer_name().unwrap_or(""),
                parent: None,
                visible: true,
                selectable: true,
                locked: false,
                color: [128, 128, 128, 255],
            }),
        });
        let mut st = s.serialize_struct("RetainedRequest", 11)?;
        st.serialize_field("schema_version", &4u32)?;
        st.serialize_field("writer_policy", "om9-retained-current-v1")?;
        st.serialize_field(
            "metadata_policy",
            if primary.is_some() {
                "primary_archive"
            } else {
                "new_mm"
            },
        )?;
        st.serialize_field("bounds_policy", "cold_leaf_hull_v1")?;
        st.serialize_field("primary_namespace", &primary)?;
        st.serialize_field(
            "model_mm_per_unit",
            &self.wire.inputs().plan().model_mm_per_unit,
        )?;
        st.serialize_field(
            "document",
            &Correlation {
                id: d.id,
                name: &d.name,
                page_unit: d.page_unit,
                current_layer: current_layer(self.wire, self.cancel).map_err(S::Error::custom)?,
            },
        )?;
        st.serialize_field(
            "sources",
            &Sources {
                wire: self.wire,
                cancel: self.cancel,
            },
        )?;
        st.serialize_field(
            "additions",
            &Additions {
                wire: self.wire,
                cancel: self.cancel,
            },
        )?;
        st.serialize_field("default_layer", &default)?;
        st.serialize_field("table_order", &Tables(self.wire))?;
        st.end()
    }
}
struct Counter<'a> {
    bytes: usize,
    limit: usize,
    wire: &'a WireOutputPlan<'a>,
    cancel: &'a Cancellation,
}
impl Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.wire
            .inputs()
            .check_cancel(self.cancel)
            .map_err(|_| io::Error::other("canceled retained request"))?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("retained packet budget"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Encoder<'a> {
    writer: crate::BoundedWriter,
    wire: &'a WireOutputPlan<'a>,
    cancel: &'a Cancellation,
}
impl Write for Encoder<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.wire
            .inputs()
            .check_cancel(self.cancel)
            .map_err(|_| io::Error::other("canceled retained request"))?;
        self.writer.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl WireOutputPlan<'_> {
    pub fn encode_retained_request(
        &self,
        coordinator: &TransactionCoordinator,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<Vec<u8>> {
        self.revalidate(coordinator, cancel)?;
        let limits = coordinator.limits();
        external.add(self.inputs().owned_usage(), limits)?;
        external.add(self.owned_usage(), limits)?;
        external.add(
            Usage::array::<SourceStamp>(self.inputs().capture().sources.capacity())?,
            limits,
        )?;
        // serde_json formats bounded numeric/UUID values without a DOM. Keep
        // its small owned error/formatter allocations admitted on refusal too.
        external.add(
            Usage {
                bytes: 8192,
                ..Usage::default()
            },
            limits,
        )?;
        let held = coordinator.retained_usage_with_held_usage(external)?;
        let limit = usize::try_from(
            (limits
                .max_bytes
                .checked_sub(held.bytes)
                .ok_or(Error::Budget("retained request aggregate"))?
                / 2)
            .min(16 * 1024 * 1024),
        )
        .map_err(|_| Error::Overflow("retained request address space"))?;
        current_layer(self, cancel)?;
        for ns in &self.inputs().plan().overlay.namespaces {
            for row in &ns.records {
                self.inputs().check_cancel(cancel)?;
                if let Some(host) = row.current {
                    current(self, host)?;
                }
                if row.source.key.kind == NativeRecordKind::Definition
                    && self
                        .source(ScopedNativeKey {
                            namespace: ns.namespace.id,
                            key: row.source.key,
                        })
                        .flatten()
                        .is_some()
                    && row.current.is_none()
                {
                    let factor = ns
                        .source
                        .units
                        .definitions
                        .get(&row.source.key.id)
                        .and_then(|units| units.millimeters_per_unit)
                        .ok_or(Error::Missing("retained definition resolved units"))?;
                    if !factor.is_finite() || factor <= 0. {
                        return Err(Error::Invalid("retained definition unit factor"));
                    }
                }
            }
        }
        for host in &self.inputs().plan().overlay.additions {
            self.inputs().check_cancel(cancel)?;
            current(self, *host)?;
        }
        let packet = Packet { wire: self, cancel };
        let mut count = Counter {
            bytes: 0,
            limit,
            wire: self,
            cancel,
        };
        if serde_json::to_writer(&mut count, &packet).is_err() {
            self.inputs().check_cancel(cancel)?;
            return Err(Error::Budget("retained request bytes"));
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(count.bytes)
            .map_err(|_| Error::Budget("retained request allocation"))?;
        let mut encoder = Encoder {
            writer: crate::BoundedWriter {
                bytes,
                limit: count.bytes,
                failed: false,
            },
            wire: self,
            cancel,
        };
        if serde_json::to_writer(&mut encoder, &packet).is_err() {
            self.inputs().check_cancel(cancel)?;
            return Err(Error::Invalid("retained request serialization"));
        }
        if encoder.writer.bytes.len() != count.bytes {
            return Err(Error::Invalid("retained request changed length"));
        }
        self.revalidate(coordinator, cancel)?;
        Ok(encoder.writer.bytes)
    }
}
