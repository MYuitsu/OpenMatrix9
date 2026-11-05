// SPDX-License-Identifier: LGPL-2.1-or-later
//! Detached OM9 clipboard payloads. Unsupported dependencies fail closed.
use super::*;
use om9_core::identity::ObjectId;

/// Admit only understood self-contained metadata. Other payloads may contain
/// object references or ownership contracts that need explicit remapping.
fn validate_extensions(entity: &Entity, limits: &Limits) -> Result<()> {
    use om9_core::model::{
        EXTRUSION_EXTENSION_SLOT, FOUR_RAIL_PROFILE_EXTENSION_SLOT, GEM_EXTENSION_SLOT,
        MESH_UV_EXTENSION_SLOT, TRIM_SURFACE_EXTENSION_SLOT,
    };
    use om9_core::rhino_semantics::{Extrusion, MeshUv, TrimmedSurface};
    for (slot, extension) in &entity.extensions {
        match slot.as_str() {
            GEM_EXTENSION_SLOT => {
                let gem = om9_core::gem::GemRecord::from_json(&extension.payload)?;
                if extension.version != gem.version || gem.gem_id.0 != entity.id {
                    return Err(Error::Invalid("clipboard gem identity"));
                }
            }
            MESH_UV_EXTENSION_SLOT if extension.version == 1 => {
                let uv = MeshUv::from_extension(extension, limits)?;
                if !matches!(entity.geometry, Geometry::Mesh(_)) {
                    return Err(Error::Invalid("clipboard mesh UV geometry"));
                }
                if uv.pbr.texture.is_some() {
                    return Err(Error::Unsupported("clipboard mesh texture resource"));
                }
            }
            TRIM_SURFACE_EXTENSION_SLOT if extension.version == 1 => {
                let trim = TrimmedSurface::from_extension(extension, limits)?;
                if !matches!(&entity.geometry, Geometry::Surface(surface) if surface == &trim.underlying)
                {
                    return Err(Error::Invalid("clipboard trimmed surface geometry"));
                }
            }
            EXTRUSION_EXTENSION_SLOT if extension.version == 1 => {
                let extrusion = Extrusion::from_extension(extension, 1.0e-6, limits)?;
                if !matches!(&entity.geometry, Geometry::Curve(curve) if curve == &extrusion.profile)
                {
                    return Err(Error::Invalid("clipboard extrusion geometry"));
                }
            }
            FOUR_RAIL_PROFILE_EXTENSION_SLOT if extension.version == 1 => {
                om9_core::rhino_semantics::FourRailProfile::from_extension(extension, limits)?;
                if !matches!(entity.geometry, Geometry::Curve(_)) {
                    return Err(Error::Invalid("clipboard four rail profile geometry"));
                }
            }
            _ => {
                return Err(Error::Unsupported(
                    "clipboard object has dependent metadata",
                ));
            }
        }
    }
    Ok(())
}

fn validate_rail_dependencies(document: &Document, limits: &Limits) -> Result<()> {
    for entity in document
        .entities
        .values()
        .chain(document.definitions.values().flat_map(|d| &d.members))
    {
        if let Some(profile) = om9_core::rhino_semantics::four_rail_profile_of(entity, limits)? {
            for id in profile.rails {
                if !matches!(
                    document.entities.get(&id).map(|rail| &rail.geometry),
                    Some(Geometry::Curve(_))
                ) {
                    return Err(Error::Unsupported(
                        "clipboard requires all four rail curves in the selection",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn exchange_dependencies(
    source: &Document,
) -> Result<(
    std::collections::BTreeSet<om9_core::identity::LinePatternId>,
    std::collections::BTreeSet<om9_core::identity::MaterialId>,
)> {
    let mut layers = std::collections::BTreeSet::new();
    let mut pending = Vec::new();
    let mut patterns = std::collections::BTreeSet::new();
    let mut materials = std::collections::BTreeSet::new();
    for entity in source
        .entities
        .values()
        .chain(source.definitions.values().flat_map(|d| &d.members))
    {
        pending.extend(entity.attributes.layer);
        patterns.extend(entity.attributes.line_pattern);
        materials.extend(entity.attributes.material);
    }
    while let Some(id) = pending.pop() {
        if layers.insert(id) {
            let layer = source
                .layers
                .get(&id)
                .ok_or(Error::Missing("clipboard layer"))?;
            pending.extend(layer.parent);
            patterns.extend(layer.line_pattern);
            materials.extend(layer.material);
        }
    }
    for id in &patterns {
        let record = source
            .exchange
            .line_patterns
            .get(id)
            .ok_or(Error::Missing("clipboard line pattern"))?;
        if record.id != *id {
            return Err(Error::Invalid("clipboard line pattern identity"));
        }
    }
    for id in &materials {
        let record = source
            .exchange
            .materials
            .get(id)
            .ok_or(Error::Missing("clipboard material"))?;
        if record.id != *id {
            return Err(Error::Invalid("clipboard material identity"));
        }
    }
    Ok((patterns, materials))
}

struct ExchangeTemp(std::path::PathBuf);
impl ExchangeTemp {
    fn new() -> Result<Self> {
        let path = std::env::temp_dir().join(format!("om9-clipboard-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).map_err(|error| Error::Io(error.to_string()))?;
        Ok(Self(path))
    }
    fn path(&self) -> std::path::PathBuf {
        self.0.join("geometry.3dm")
    }
}
impl Drop for ExchangeTemp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.path());
        let _ = std::fs::remove_dir(&self.0);
    }
}

impl Controller {
    /// Export a current selection into a validated Rhino 5 clipboard archive.
    pub fn copy_selected_3dm(&self) -> Result<Vec<u8>> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let selected: Vec<_> = self.selected().iter().copied().collect();
        let temp = ExchangeTemp::new()?;
        crate::three_dm_host::ThreeDmHost::configured()?.export_selected(
            &self.coordinator,
            self.document,
            &selected,
            &temp.path(),
            &om9_core::transaction::Cancellation::default(),
        )?;
        let path = temp.path();
        let size = std::fs::metadata(&path)
            .map_err(|error| Error::Io(error.to_string()))?
            .len();
        if size > om9_exchange::three_dm_clipboard::MAX_GEOMETRY_BYTES as u64 {
            return Err(Error::Budget("clipboard geometry"));
        }
        self.coordinator
            .retained_usage_with_held_usage(om9_core::budget::Usage {
                bytes: size,
                ..Default::default()
            })?;
        let bytes = std::fs::read(path).map_err(|error| Error::Io(error.to_string()))?;
        om9_exchange::three_dm_clipboard::receive_clipboard(&bytes, None)
            .map_err(|_| Error::Invalid("Rhino 5 clipboard geometry"))?;
        Ok(bytes)
    }

    /// Validate and import clipboard geometry through the existing single transaction.
    pub fn paste_3dm(&mut self, bytes: &[u8], metadata: Option<&[u8]>) -> Result<Vec<String>> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let geometry_bytes =
            u64::try_from(bytes.len()).map_err(|_| Error::Overflow("clipboard geometry size"))?;
        let metadata_bytes = u64::try_from(metadata.map_or(0, <[u8]>::len))
            .map_err(|_| Error::Overflow("clipboard metadata size"))?;
        let held = metadata_bytes
            .checked_mul(4)
            .and_then(|value| value.checked_add(geometry_bytes))
            .ok_or(Error::Overflow("clipboard admission size"))?;
        self.coordinator
            .retained_usage_with_held_usage(om9_core::budget::Usage {
                bytes: held,
                ..Default::default()
            })?;
        let received = om9_exchange::three_dm_clipboard::receive_clipboard(bytes, metadata)
            .map_err(|_| Error::Invalid("Rhino 5 clipboard geometry or metadata"))?;
        use std::io::Write;
        let temp = ExchangeTemp::new()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp.path())
            .map_err(|error| Error::Io(error.to_string()))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| Error::Io(error.to_string()))?;
        drop(file);
        self.import_3dm_with_clipboard_metadata(&temp.path(), received.snapshot.as_ref())
    }

    /// Capture current selected objects into a validated OM9 archive.
    pub fn copy_selected_om9(&self) -> Result<Vec<u8>> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let document = self.document();
        let ids: Vec<_> = self.selected().iter().copied().collect();
        for id in &ids {
            let entity = document
                .entities
                .get(id)
                .ok_or(Error::Missing("selected object"))?;
            validate_extensions(entity, self.coordinator.limits())?;
        }
        let capture =
            om9_exchange::ExportCapture::selected_om9(&self.coordinator, self.document, &ids)?;
        for definition in capture.document().definitions.values() {
            for member in &definition.members {
                validate_extensions(member, self.coordinator.limits())?;
            }
        }
        validate_rail_dependencies(capture.document(), self.coordinator.limits())?;
        exchange_dependencies(capture.document())?;
        let bytes = om9_archive::Archive::encode(capture.document(), self.coordinator.limits())?;
        capture.revalidate(&self.coordinator)?;
        Ok(bytes)
    }

    /// Paste the archive as one transaction; unsupported dependencies never partially commit.
    pub fn paste_om9(&mut self, bytes: &[u8]) -> Result<Vec<ObjectId>> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let source = om9_archive::Archive::decode(bytes, self.coordinator.limits())?;
        let target = self.document();
        if source.page_unit != target.page_unit {
            return Err(Error::Unsupported("clipboard document units differ"));
        }
        if source.entities.is_empty()
            || !source.history.references.is_empty()
            || !source.retained_sources.is_empty()
            || !source.import_namespaces.is_empty()
        {
            return Err(Error::Unsupported(
                "clipboard object has dependent metadata or history",
            ));
        }
        let roots: std::collections::BTreeSet<_> = source.entities.keys().copied().collect();
        for recipe in source.history.recipes.values() {
            if recipe.state != om9_core::history::EvaluationState::Clean
                || recipe.outputs.iter().any(|id| !roots.contains(id))
                || recipe
                    .inputs
                    .iter()
                    .any(|input| !roots.contains(&input.source.object))
            {
                return Err(Error::Unsupported(
                    "clipboard history dependencies or state",
                ));
            }
        }
        for (id, result) in &source.results {
            if !roots.contains(id)
                || result.status != om9_core::result::ResultStatus::Current
                || result
                    .recipe
                    .is_none_or(|r| !source.history.recipes.contains_key(&r))
                || result
                    .source_stamps
                    .iter()
                    .any(|stamp| !roots.contains(&stamp.object))
            {
                return Err(Error::Unsupported("clipboard result dependencies or state"));
            }
        }
        for entity in source.entities.values().chain(
            source
                .definitions
                .values()
                .flat_map(|definition| &definition.members),
        ) {
            validate_extensions(entity, self.coordinator.limits())?;
        }
        validate_rail_dependencies(&source, self.coordinator.limits())?;
        let (needed_patterns, needed_materials) = exchange_dependencies(&source)?;
        let mut pattern_ids = std::collections::BTreeMap::new();
        let mut material_ids = std::collections::BTreeMap::new();
        let mut new_records = om9_core::exchange::ExchangeMetadata::default();
        for old in needed_patterns {
            let record = &source.exchange.line_patterns[&old];
            let existing = target.exchange.line_patterns.values().find(|candidate| {
                let mut comparable = record.clone();
                comparable.id = candidate.id;
                comparable == **candidate
            });
            let id = if let Some(existing) = existing {
                existing.id
            } else {
                let mut copy = record.clone();
                copy.id = om9_core::identity::LinePatternId::new();
                let id = copy.id;
                new_records.line_patterns.insert(id, copy);
                id
            };
            pattern_ids.insert(old, id);
        }
        for old in needed_materials {
            let record = &source.exchange.materials[&old];
            let existing = target.exchange.materials.values().find(|candidate| {
                let mut comparable = record.clone();
                comparable.id = candidate.id;
                comparable == **candidate
            });
            let id = if let Some(existing) = existing {
                existing.id
            } else {
                let mut copy = record.clone();
                copy.id = om9_core::identity::MaterialId::new();
                let id = copy.id;
                new_records.materials.insert(id, copy);
                id
            };
            material_ids.insert(old, id);
        }
        // Reuse an identical layer or recreate its ancestry in the same paste
        // transaction. Never overwrite an existing destination layer.
        let mut needed = std::collections::BTreeSet::new();
        let mut pending: Vec<_> = source
            .entities
            .values()
            .chain(
                source
                    .definitions
                    .values()
                    .flat_map(|definition| &definition.members),
            )
            .filter_map(|entity| entity.attributes.layer)
            .collect();
        while let Some(id) = pending.pop() {
            if needed.insert(id) {
                let layer = source
                    .layers
                    .get(&id)
                    .ok_or(Error::Missing("clipboard layer"))?;
                if let Some(parent) = layer.parent {
                    pending.push(parent);
                }
            }
        }
        let mut layers = std::collections::BTreeMap::new();
        let mut mutations = Vec::new();
        if !new_records.line_patterns.is_empty() || !new_records.materials.is_empty() {
            mutations.push(Mutation::ExchangeInsert {
                document: self.document,
                expected_revision: target.revision,
                records: Box::new(new_records),
            });
        }
        while !needed.is_empty() {
            let ready = needed
                .iter()
                .copied()
                .find(|id| {
                    source.layers[id]
                        .parent
                        .is_none_or(|parent| layers.contains_key(&parent))
                })
                .ok_or(Error::Invalid("clipboard layer hierarchy"))?;
            let source_layer = &source.layers[&ready];
            let parent = source_layer.parent.map(|id| layers[&id]);
            let existing = target.layers.values().find(|candidate| {
                let mut equivalent = source_layer.clone();
                equivalent.id = candidate.id;
                equivalent.revision = candidate.revision;
                equivalent.parent = parent;
                equivalent.line_pattern = equivalent.line_pattern.map(|id| pattern_ids[&id]);
                equivalent.material = equivalent.material.map(|id| material_ids[&id]);
                &equivalent == *candidate
            });
            let id = if let Some(existing) = existing {
                existing.id
            } else {
                let mut layer = source_layer.clone();
                layer.id = om9_core::identity::LayerId::new();
                layer.revision = 1;
                layer.parent = parent;
                layer.line_pattern = layer.line_pattern.map(|id| pattern_ids[&id]);
                layer.material = layer.material.map(|id| material_ids[&id]);
                let id = layer.id;
                let plan = om9_core::layers::LayerPlan::capture(
                    &self.coordinator,
                    self.document,
                    om9_core::layers::LayerAction::Create {
                        layer,
                        make_current: false,
                    },
                    None,
                )?;
                mutations.push(Mutation::Layers(Box::new(plan)));
                id
            };
            layers.insert(ready, id);
            needed.remove(&ready);
        }
        let definitions: std::collections::BTreeMap<_, _> = source
            .definitions
            .keys()
            .map(|id| (*id, om9_core::identity::DefinitionId::new()))
            .collect();
        let history_ids: std::collections::BTreeMap<_, _> = source
            .history
            .recipes
            .keys()
            .map(|id| (*id, om9_core::identity::HistoryId::new()))
            .collect();
        let mut object_ids = std::collections::BTreeMap::new();
        for entity in source.entities.values().chain(
            source
                .definitions
                .values()
                .flat_map(|definition| &definition.members),
        ) {
            if object_ids.insert(entity.id, ObjectId::new()).is_some() {
                return Err(Error::Invalid("duplicate clipboard object identity"));
            }
        }
        let remap_instance = |entity: &mut Entity| -> Result<()> {
            if let Geometry::Instance(id) = &mut entity.geometry {
                *id = *definitions
                    .get(id)
                    .ok_or(Error::Missing("clipboard block definition"))?;
            }
            if let Some(layer) = &mut entity.attributes.layer {
                *layer = *layers
                    .get(layer)
                    .ok_or(Error::Missing("clipboard target layer"))?;
            }
            if let Some(id) = &mut entity.attributes.line_pattern {
                *id = pattern_ids[id];
            }
            if let Some(id) = &mut entity.attributes.material {
                *id = material_ids[id];
            }
            entity.id = *object_ids
                .get(&entity.id)
                .ok_or(Error::Missing("clipboard object identity"))?;
            if let Some(extension) = entity
                .extensions
                .get_mut(om9_core::model::GEM_EXTENSION_SLOT)
            {
                let mut gem = om9_core::gem::GemRecord::from_json(&extension.payload)?;
                gem.gem_id = om9_core::gem::GemId(entity.id);
                *extension = gem.as_extension()?;
            }
            if let Some(extension) = entity
                .extensions
                .get_mut(om9_core::model::FOUR_RAIL_PROFILE_EXTENSION_SLOT)
            {
                let mut profile = om9_core::rhino_semantics::FourRailProfile::from_extension(
                    extension,
                    self.coordinator.limits(),
                )?;
                for rail in &mut profile.rails {
                    *rail = *object_ids
                        .get(rail)
                        .ok_or(Error::Missing("clipboard rail identity"))?;
                }
                *extension = profile.to_extension()?;
            }
            entity.generation = 1;
            entity.geometry_revision = 1;
            entity.attributes_revision = 1;
            Ok(())
        };
        for mut definition in source.definitions.into_values() {
            definition.id = definitions[&definition.id];
            definition.revision = 1;
            for member in &mut definition.members {
                remap_instance(member)?;
            }
            mutations.push(Mutation::DefinitionInsert {
                document: self.document,
                expected_revision: target.revision,
                definition: Box::new(definition),
                instances: Vec::new(),
            });
        }
        let mut copies = Vec::with_capacity(source.entities.len());
        for mut entity in source.entities.into_values() {
            remap_instance(&mut entity)?;
            copies.push(entity);
        }
        let ids: Vec<_> = copies.iter().map(|entity| entity.id).collect();
        mutations.extend(copies.into_iter().map(|entity| Mutation::Insert {
            document: self.document,
            entity,
        }));
        let remap_stamp = |stamp: &mut om9_core::identity::SourceStamp| -> Result<()> {
            if !roots.contains(&stamp.object) {
                return Err(Error::Missing("clipboard history input"));
            }
            *stamp = om9_core::identity::SourceStamp {
                document: target.id,
                document_epoch: target.epoch,
                session_generation: target.session_generation,
                owner_session: target.owner_session,
                context_revision: target.context_revision,
                object: *object_ids
                    .get(&stamp.object)
                    .ok_or(Error::Missing("clipboard history input"))?,
                generation: 1,
                geometry_revision: 1,
                attributes_revision: 1,
            };
            Ok(())
        };
        let mut results = source.results;
        for mut recipe in source.history.recipes.into_values() {
            let old_id = recipe.id;
            recipe.id = history_ids[&old_id];
            recipe.document = target.id;
            recipe.revision = 1;
            for input in &mut recipe.inputs {
                remap_stamp(&mut input.source)?;
                if let om9_core::request::Selection::Member(members) = &mut input.selection {
                    for member in members {
                        *member = *object_ids
                            .get(member)
                            .ok_or(Error::Missing("clipboard history member"))?;
                    }
                }
            }
            let mut published = Vec::new();
            for output in &mut recipe.outputs {
                if let Some(mut result) = results.remove(output) {
                    for stamp in &mut result.source_stamps {
                        remap_stamp(stamp)?;
                    }
                    if let Geometry::Instance(id) = &mut result.geometry_snapshot {
                        *id = *definitions
                            .get(id)
                            .ok_or(Error::Missing("clipboard result definition"))?;
                    }
                    result.recipe = Some(recipe.id);
                    result.geometry_revision = 1;
                    published.push((object_ids[output], result));
                }
                *output = object_ids[output];
            }
            mutations.push(Mutation::History {
                document: self.document,
                expected_revision: target.revision,
                recipe: Box::new(recipe),
                published,
            });
        }
        if !results.is_empty() {
            return Err(Error::Invalid("clipboard unowned result"));
        }
        for mut feature in source.features.into_values() {
            feature.id = om9_core::identity::FeatureId::new();
            feature.revision = 1;
            for output in &mut feature.outputs {
                *output = *object_ids
                    .get(output)
                    .ok_or(Error::Missing("clipboard feature output"))?;
            }
            mutations.push(Mutation::FeatureInsert {
                document: self.document,
                expected_revision: target.revision,
                feature: Box::new(feature),
            });
        }
        for mut group in source.groups.into_values() {
            group.id = om9_core::identity::GroupId::new();
            group.revision = 1;
            for member in &mut group.members {
                *member = *object_ids
                    .get(member)
                    .ok_or(Error::Missing("clipboard group member"))?;
            }
            mutations.push(Mutation::GroupCreate {
                document: self.document,
                expected_revision: target.revision,
                group: Box::new(group),
            });
        }
        self.commit_clipboard_paste(mutations, &ids)?;
        Ok(ids)
    }

    /// Validate the prospective scene before publication, then select any
    /// copied roots that remain selectable without turning a committed edit
    /// into a reported failure if selection cannot be restored.
    fn commit_clipboard_paste(&mut self, mutations: Vec<Mutation>, ids: &[ObjectId]) -> Result<()> {
        let old_scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("clipboard scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("clipboard scene"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<ObjectId>(ids.len())?,
            self.coordinator.limits(),
        )?;
        held.add(
            om9_core::budget::Usage::btree::<ObjectId, ()>(ids.len())?,
            self.coordinator.limits(),
        )?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new("Paste", mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
            let selectable: Vec<_> = ids
                .iter()
                .copied()
                .filter(|id| {
                    self.scene.as_ref().is_some_and(|scene| {
                        scene.objects.iter().any(|object| {
                            object.root == *id && object.selectable && object.has_pick_geometry()
                        })
                    })
                })
                .collect();
            if selectable.is_empty() {
                self.clear_selection();
            } else if let Err(error) = self.select_objects(&selectable) {
                self.clear_selection();
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    "Paste",
                    &format!("Committed; result selection was cleared: {error}"),
                    None,
                );
            }
        }
        Ok(())
    }
}
