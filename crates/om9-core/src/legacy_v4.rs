// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed v4 document input. This is not the current document serialization.
//! Old display data stays in this value until a migration owner takes it.
use crate::{
    brep::Brep,
    geometry::{Cloud, Curve, Geometry, Hatch, Light, Mesh, NativeBrep, NurbsSurface, Point3, Transform, Unit},
    identity::*,
    model::{Attributes, Definition, Document, Entity, Extension, Feature, Group, Layer},
    result::{ConversionReport, PublishedResult, ResultStatus},
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
enum GeometryV4 {
    #[serde(alias = "BRep", alias = "B-rep")]
    Brep(Brep),
    NativeBrep(NativeBrep),
    Point(Point3),
    Curve(Curve),
    Surface(NurbsSurface),
    Mesh(Mesh),
    Cloud(Cloud),
    Instance(DefinitionId),
    Hatch(Hatch),
    Light(Light),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityV4 {
    id: ObjectId,
    label: String,
    generation: u64,
    geometry_revision: u64,
    attributes_revision: u64,
    geometry: GeometryV4,
    placement: Transform,
    attributes: Attributes,
    #[serde(default, deserialize_with = "crate::identity::unique_named_slot_map")]
    extensions: BTreeMap<String, Extension>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionV4 {
    id: DefinitionId,
    label: String,
    revision: u64,
    members: Vec<EntityV4>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedResultV4 {
    operation: String,
    geometry_revision: u64,
    geometry_snapshot: GeometryV4,
    source_stamps: Vec<SourceStamp>,
    conversion: ConversionReport,
    recipe: Option<HistoryId>,
    status: ResultStatus,
}

/// An untrusted, fully typed v4 document. Archive preflight and admission must
/// precede deserialization, and normal manifest/resource validation follows it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentV4 {
    id: DocumentId,
    name: String,
    epoch: u64,
    session_generation: u64,
    #[serde(default = "SessionId::legacy_nil")]
    owner_session: SessionId,
    revision: u64,
    context_revision: u64,
    page_unit: Unit,
    #[serde(default)]
    tolerance: Option<crate::history::Tolerance>,
    current_layer: Option<LayerId>,
    layer_order: Vec<LayerId>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    layers: BTreeMap<LayerId, Layer>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    entities: BTreeMap<ObjectId, EntityV4>,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    definitions: BTreeMap<DefinitionId, DefinitionV4>,
    history: crate::history::HistoryGraph,
    #[serde(deserialize_with = "crate::identity::unique_uuid_map")]
    results: BTreeMap<ObjectId, PublishedResultV4>,
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    resources: BTreeMap<ResourceId, crate::resource::Resource>,
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    retained_sources: BTreeMap<SourceId, crate::retained::RetainedSource>,
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    import_namespaces: BTreeMap<ImportNamespaceId, crate::retained::ImportNamespace>,
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    groups: BTreeMap<GroupId, Group>,
    #[serde(default, deserialize_with = "crate::identity::unique_uuid_map")]
    features: BTreeMap<FeatureId, Feature>,
    #[serde(default)]
    exchange: crate::exchange::ExchangeMetadata,
}

/// Display data from v4 stays paired with the exact new source owner. A caller
/// must reserve and validate it before moving it into the representation store.
#[derive(Debug, Default)]
pub struct LegacyDisplays {
    pub occt: Vec<(crate::brep::CanonicalBrep, crate::brep::BrepRepresentation)>,
    pub native: Vec<(crate::geometry::NativeBrepSource, crate::geometry::NativeBrepRepresentation)>,
}
impl LegacyDisplays {
    pub fn vector_storage_bytes(&self) -> crate::Result<u64> {
        use crate::budget::Usage;
        Usage::array::<(crate::brep::CanonicalBrep, crate::brep::BrepRepresentation)>(self.occt.capacity())?.bytes
            .checked_add(Usage::array::<(crate::geometry::NativeBrepSource, crate::geometry::NativeBrepRepresentation)>(self.native.capacity())?.bytes)
            .ok_or(crate::Error::Overflow("legacy display vectors"))
    }
    /// Display and vector allocations retained in addition to the canonical
    /// document. Source and topology allocations are accounted by the document.
    pub fn retained_display_bytes(&self) -> crate::Result<u64> {
        let mut total = self.vector_storage_bytes()?;
        for (_, display) in &self.occt {
            total = total.checked_add(display.display_allocation_bytes()?)
                .ok_or(crate::Error::Overflow("legacy display storage"))?;
        }
        for (_, display) in &self.native {
            total = total.checked_add(display.display_bytes()?)
                .ok_or(crate::Error::Overflow("legacy native display storage"))?;
        }
        Ok(total)
    }
}

impl GeometryV4 {
    fn split(self, displays: &mut LegacyDisplays) -> Geometry {
        match self {
            Self::Brep(value) => {
                let (source, display) = value.into_canonical_and_representation();
                displays.occt.push((source.clone(), display));
                Geometry::Brep(source)
            }
            Self::NativeBrep(value) => {
                let (source, display) = value.into_source_and_representation();
                displays.native.push((source.clone(), display));
                Geometry::NativeBrep(source)
            }
            Self::Point(value) => Geometry::Point(value),
            Self::Curve(value) => Geometry::Curve(value),
            Self::Surface(value) => Geometry::Surface(value),
            Self::Mesh(value) => Geometry::Mesh(value),
            Self::Cloud(value) => Geometry::Cloud(value),
            Self::Instance(value) => Geometry::Instance(value),
            Self::Hatch(value) => Geometry::Hatch(value),
            Self::Light(value) => Geometry::Light(value),
        }
    }
}
impl EntityV4 {
    fn split(self, displays: &mut LegacyDisplays) -> Entity {
        Entity { id: self.id, label: self.label, generation: self.generation,
            geometry_revision: self.geometry_revision, attributes_revision: self.attributes_revision,
            geometry: self.geometry.split(displays), placement: self.placement,
            attributes: self.attributes, extensions: self.extensions }
    }
}

impl DocumentV4 {
    /// Validate each old display before splitting. The callback receives
    /// scratch in addition to the already-admitted v4 document allocation.
    pub fn validate_displays_admitted(&self, limits: &crate::budget::Limits,
        mut admit_scratch: impl FnMut(u64, u64) -> crate::Result<()>) -> crate::Result<()> {
        let geometries = self.entities.values().chain(self.definitions.values().flat_map(|d| &d.members))
            .map(|entity| &entity.geometry).chain(self.results.values().map(|value| &value.geometry_snapshot));
        for geometry in geometries {
            match geometry {
                GeometryV4::Brep(value) => {
                    let baseline = value.storage_usage(limits)?.bytes;
                    value.validate_admitted(limits, &mut |live, pending| {
                        admit_scratch(live.checked_sub(baseline).ok_or(crate::Error::Invalid("legacy BRep validation accounting"))?, pending)
                    })?;
                }
                GeometryV4::NativeBrep(value) => { value.validate(limits)?; }
                _ => {}
            }
        }
        Ok(())
    }

    /// Number of old display owners, including result snapshots and definitions.
    pub fn display_counts(&self) -> (usize, usize) {
        let mut result = (0usize, 0usize);
        let mut count = |geometry: &GeometryV4| match geometry {
            GeometryV4::Brep(_) => result.0 += 1,
            GeometryV4::NativeBrep(_) => result.1 += 1,
            _ => {}
        };
        for entity in self.entities.values().chain(self.definitions.values().flat_map(|d| &d.members)) {
            count(&entity.geometry);
        }
        for value in self.results.values() { count(&value.geometry_snapshot); }
        result
    }

    /// Upper-bound the extra collection storage while the old typed document
    /// and its replacement coexist. Geometry allocations transfer by move.
    pub fn split_allocation_usage(&self, limits: &crate::budget::Limits) -> crate::Result<crate::budget::Usage> {
        use crate::budget::Usage;
        let (occt, native) = self.display_counts();
        let mut usage = Usage::array::<(crate::brep::CanonicalBrep, crate::brep::BrepRepresentation)>(occt)?;
        usage.add(Usage::array::<(crate::geometry::NativeBrepSource, crate::geometry::NativeBrepRepresentation)>(native)?, limits)?;
        usage.add(Usage::btree::<ObjectId, Entity>(self.entities.len())?, limits)?;
        usage.add(Usage::btree::<DefinitionId, Definition>(self.definitions.len())?, limits)?;
        usage.add(Usage::btree::<ObjectId, PublishedResult>(self.results.len())?, limits)?;
        for definition in self.definitions.values() {
            usage.add(Usage::array::<Entity>(definition.members.len())?, limits)?;
        }
        Ok(usage)
    }

    /// Split only after the caller admits map/vector overlap and old displays.
    /// This operation preserves old display bytes; it does not attest topology.
    pub fn split(self, mut displays: LegacyDisplays) -> crate::Result<(Document, LegacyDisplays)> {
        let (occt, native) = self.display_counts();
        displays.occt.try_reserve_exact(occt).map_err(|_| crate::Error::Budget("legacy BRep displays"))?;
        displays.native.try_reserve_exact(native).map_err(|_| crate::Error::Budget("legacy native BRep displays"))?;
        let entities = self.entities.into_iter().map(|(key, value)| (key, value.split(&mut displays))).collect();
        let definitions = self.definitions.into_iter().map(|(key, value)| {
            let members = value.members.into_iter().map(|value| value.split(&mut displays)).collect();
            (key, Definition { id: value.id, label: value.label, revision: value.revision, members })
        }).collect();
        let results = self.results.into_iter().map(|(key, value)| (key, PublishedResult {
            operation: value.operation, geometry_revision: value.geometry_revision,
            geometry_snapshot: value.geometry_snapshot.split(&mut displays), source_stamps: value.source_stamps,
            conversion: value.conversion, recipe: value.recipe, status: value.status,
        })).collect();
        let document = Document { id: self.id, name: self.name, epoch: self.epoch, session_generation: self.session_generation,
            owner_session: self.owner_session, revision: self.revision, context_revision: self.context_revision,
            page_unit: self.page_unit, tolerance: self.tolerance, current_layer: self.current_layer,
            layer_order: self.layer_order, layers: self.layers, entities, definitions, history: self.history,
            results, resources: self.resources, retained_sources: self.retained_sources,
            import_namespaces: self.import_namespaces, groups: self.groups, features: self.features, exchange: self.exchange };
        Ok((document, displays))
    }
}
