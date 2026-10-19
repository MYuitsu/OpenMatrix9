// SPDX-License-Identifier: LGPL-2.1-or-later
//! Borrowed diagnostic projection. Neither hashing nor printing builds a JSON
//! DOM or a serialized document copy; the sink is owned by the caller.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    history::Recipe,
    identity::*,
    model::{Document, Entity},
    request::SourceBinding,
    resource::Resource,
    result::PublishedResult,
};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{self, BufWriter, Write},
};

const BUFFER: usize = 16 * 1024;
fn hex(hash: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(64);
    for byte in hash {
        write!(result, "{byte:02x}").expect("reserved hash string");
    }
    result
}
struct Stamp<'a>(&'a SourceStamp);
impl Serialize for Stamp<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let x = self.0;
        let mut s = s.serialize_struct("SourceStamp", 9)?;
        s.serialize_field("document", &x.document)?;
        s.serialize_field("document_epoch", &0u64)?;
        s.serialize_field("session_generation", &0u64)?;
        s.serialize_field("owner_session", &0u64)?;
        s.serialize_field("context_revision", &x.context_revision)?;
        s.serialize_field("object", &x.object)?;
        s.serialize_field("generation", &x.generation)?;
        s.serialize_field("geometry_revision", &x.geometry_revision)?;
        s.serialize_field("attributes_revision", &x.attributes_revision)?;
        s.end()
    }
}
struct Stamps<'a>(&'a [SourceStamp]);
impl Serialize for Stamps<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut s = s.serialize_seq(Some(self.0.len()))?;
        for x in self.0 {
            s.serialize_element(&Stamp(x))?;
        }
        s.end()
    }
}
struct Bindings<'a>(&'a [SourceBinding]);
impl Serialize for Bindings<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Binding<'a> {
            role: &'a str,
            source: Stamp<'a>,
            selection: &'a om9_core::request::Selection,
        }
        let mut s = s.serialize_seq(Some(self.0.len()))?;
        for x in self.0 {
            s.serialize_element(&Binding {
                role: &x.role,
                source: Stamp(&x.source),
                selection: &x.selection,
            })?;
        }
        s.end()
    }
}
struct RecipeProjection<'a>(&'a Recipe);
impl Serialize for RecipeProjection<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let x = self.0;
        let mut s = s.serialize_struct("Recipe", 10)?;
        s.serialize_field("id", &x.id)?;
        s.serialize_field("document", &x.document)?;
        s.serialize_field("command", &x.command)?;
        s.serialize_field("environment", &x.environment)?;
        s.serialize_field("inputs", &Bindings(&x.inputs))?;
        s.serialize_field("parameters", &x.parameters)?;
        s.serialize_field("outputs", &x.outputs)?;
        s.serialize_field("policy", &x.policy)?;
        s.serialize_field("state", &x.state)?;
        s.serialize_field("revision", &x.revision)?;
        s.end()
    }
}
struct Recipes<'a>(&'a BTreeMap<HistoryId, Recipe>);
impl Serialize for Recipes<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let mut s = s.serialize_map(Some(self.0.len()))?;
        for (id, r) in self.0 {
            s.serialize_entry(id, &RecipeProjection(r))?;
        }
        s.end()
    }
}
struct Results<'a>(&'a BTreeMap<ObjectId, PublishedResult>);
impl Serialize for Results<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Published<'a> {
            operation: &'a str,
            geometry_revision: u64,
            geometry_snapshot: &'a om9_core::geometry::Geometry,
            source_stamps: Stamps<'a>,
            conversion: &'a om9_core::result::ConversionReport,
            recipe: Option<HistoryId>,
            status: om9_core::result::ResultStatus,
        }
        let mut s = s.serialize_map(Some(self.0.len()))?;
        for (id, x) in self.0 {
            s.serialize_entry(
                id,
                &Published {
                    operation: &x.operation,
                    geometry_revision: x.geometry_revision,
                    geometry_snapshot: &x.geometry_snapshot,
                    source_stamps: Stamps(&x.source_stamps),
                    conversion: &x.conversion,
                    recipe: x.recipe,
                    status: x.status,
                },
            )?;
        }
        s.end()
    }
}
struct Persistent<'a>(&'a Document);
impl Serialize for Persistent<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct History<'a> {
            recipes: Recipes<'a>,
        }
        let d = self.0;
        let mut s = s.serialize_struct("Document", 18)?;
        s.serialize_field("id", &d.id)?;
        s.serialize_field("name", &d.name)?;
        s.serialize_field("epoch", &0u64)?;
        s.serialize_field("session_generation", &0u64)?;
        s.serialize_field("owner_session", &0u64)?;
        s.serialize_field("revision", &d.revision)?;
        s.serialize_field("context_revision", &d.context_revision)?;
        s.serialize_field("page_unit", &d.page_unit)?;
        s.serialize_field("current_layer", &d.current_layer)?;
        s.serialize_field("layer_order", &d.layer_order)?;
        s.serialize_field("layers", &d.layers)?;
        s.serialize_field("entities", &d.entities)?;
        s.serialize_field("definitions", &d.definitions)?;
        s.serialize_field(
            "history",
            &History {
                recipes: Recipes(&d.history.recipes),
            },
        )?;
        s.serialize_field("results", &Results(&d.results))?;
        if !d.resources.is_empty() {
            s.serialize_field("resources", &d.resources)?;
        }
        if !d.retained_sources.is_empty() {
            s.serialize_field("retained_sources", &d.retained_sources)?;
        }
        if !d.import_namespaces.is_empty() {
            s.serialize_field("import_namespaces", &d.import_namespaces)?;
        }
        s.end()
    }
}
struct HashWriter(Sha256);
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Objects<'a>(&'a BTreeMap<ObjectId, Entity>);
impl Serialize for Objects<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Object<'a> {
            id: ObjectId,
            label: &'a str,
            generation: u64,
            geometry_revision: u64,
            geometry: &'a om9_core::geometry::Geometry,
            placement: om9_core::geometry::Transform,
            attributes: &'a om9_core::model::Attributes,
        }
        let mut s = s.serialize_seq(Some(self.0.len()))?;
        for x in self.0.values() {
            s.serialize_element(&Object {
                id: x.id,
                label: &x.label,
                generation: x.generation,
                geometry_revision: x.geometry_revision,
                geometry: &x.geometry,
                placement: x.placement,
                attributes: &x.attributes,
            })?;
        }
        s.end()
    }
}
struct Resources<'a>(&'a BTreeMap<ResourceId, Resource>);
impl Serialize for Resources<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Payload<'a> {
            id: ResourceId,
            kind: &'a om9_core::resource::ResourceKind,
            name: &'a str,
            bytes: usize,
            payload_sha256: String,
        }
        let mut s = s.serialize_seq(Some(self.0.len()))?;
        for x in self.0.values() {
            let bytes = x.payload().map_err(serde::ser::Error::custom)?;
            s.serialize_element(&Payload {
                id: x.id,
                kind: &x.kind,
                name: &x.name,
                bytes: bytes.len(),
                payload_sha256: hex(Sha256::digest(bytes).into()),
            })?;
        }
        s.end()
    }
}
/// Deterministic typed projection v2. Runtime owner locations alone are
/// normalized; arbitrary recipe parameters keep every key and value. This
/// algorithm supersedes the earlier DOM-key-order diagnostic fingerprint.
pub fn write<W: Write>(
    sink: W,
    document: &Document,
    diagnostics: &[String],
    limits: &Limits,
) -> Result<()> {
    let mut held = Usage {
        bytes: (BUFFER + 256) as u64,
        ..Usage::default()
    };
    held.check(limits)?;
    held.add(
        Usage::array::<String>(document.history.recipes.len().max(diagnostics.len()))?,
        limits,
    )?;
    for diagnostic in diagnostics {
        held.add(
            Usage {
                bytes: diagnostic.capacity() as u64,
                ..Usage::default()
            },
            limits,
        )?;
    }
    // Diagnostics remain owned by the caller while document validation grows
    // its temporary sets. Subtract them before that validation starts.
    let remaining = limits.remaining_after(held)?;
    let document_usage = document.validate(&remaining)?;
    held.add(document_usage, limits)?;
    let mut hash = HashWriter(Sha256::new());
    serde_json::to_writer(&mut hash, &Persistent(document))
        .map_err(|_| Error::Invalid("persistent report serialization"))?;
    #[derive(Serialize)]
    struct Report<'a> {
        target: &'static str,
        version: &'static str,
        backend_profile: &'static str,
        document_id: DocumentId,
        page_unit: om9_core::geometry::Unit,
        current_layer: Option<LayerId>,
        layer_order: &'a [LayerId],
        layers: &'a BTreeMap<LayerId, om9_core::model::Layer>,
        document_sha256: String,
        document_hash_algorithm: &'static str,
        objects: Objects<'a>,
        resources: Resources<'a>,
        retained_sources: &'a BTreeMap<SourceId, om9_core::retained::RetainedSource>,
        import_namespaces: &'a BTreeMap<ImportNamespaceId, om9_core::retained::ImportNamespace>,
        restoration: &'static str,
        diagnostics: &'a [String],
    }
    let report = Report {
        target: "OM9Standalone",
        version: env!("CARGO_PKG_VERSION"),
        backend_profile: "rust-owned-foundation",
        document_id: document.id,
        page_unit: document.page_unit,
        current_layer: document.current_layer,
        layer_order: &document.layer_order,
        layers: &document.layers,
        document_sha256: hex(hash.0.finalize().into()),
        document_hash_algorithm: "om9-persistent-projection-v2",
        objects: Objects(&document.entities),
        resources: Resources(&document.resources),
        retained_sources: &document.retained_sources,
        import_namespaces: &document.import_namespaces,
        restoration: if diagnostics.is_empty() {
            "ready"
        } else {
            "unresolved"
        },
        diagnostics,
    };
    let mut output = BufWriter::with_capacity(BUFFER, sink);
    serde_json::to_writer(&mut output, &report)
        .map_err(|_| Error::Io("report output failed".into()))?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|_| Error::Io("report output failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_admission_failure_writes_no_partial_success() {
        let document = Document::new("bounded output");
        let owned = document.validate(&Limits::default()).unwrap().bytes;
        let limits = Limits {
            max_bytes: owned + 4096,
            ..Limits::default()
        };
        let mut bytes = vec![];
        assert!(matches!(
            write(&mut bytes, &document, &[], &limits),
            Err(Error::Budget(_))
        ));
        assert!(bytes.is_empty());
    }
}
