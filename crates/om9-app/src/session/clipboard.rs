// SPDX-License-Identifier: LGPL-2.1-or-later
//! Detached OM9 clipboard payloads. Unsupported dependencies fail closed.
use super::*;
use om9_core::identity::ObjectId;
#[cfg(feature = "occt-brep")]
use om9_core::budget::Usage;

#[cfg(feature = "occt-brep")]
pub(crate) struct PreparedBrepJobbag {
    transaction: om9_core::transaction::PreparedTransaction,
    scene: ReservedCandidateScene,
    ids: Vec<ObjectId>,
    context: om9_core::transaction::OperationContext,
    _owners: crate::brep_restore_host::BoundBrepOwners,
    _projection: Option<om9_core::transaction::ExternalMemoryReservation>,
    _working: om9_core::transaction::ExternalMemoryReservation,
    staging: om9_core::transaction::ExternalMemoryReservation,
    peer: Option<om9_core::transaction::TransactionPreparationPeer>,
    cache: Option<Arc<super::files::BrepArchiveCache>>,
}

#[cfg(feature = "occt-brep")]
impl PreparedBrepJobbag {
    pub(crate) fn retain_peer(mut self, peer: om9_core::transaction::TransactionPreparationPeer) -> Self {
        self.peer = Some(peer);
        self
    }
    pub(crate) fn retain_cache(mut self, cache: Arc<super::files::BrepArchiveCache>) -> Self {
        self.cache = Some(cache);
        self
    }
}

/// Explicit embedded-resource URI; other texture strings remain unresolved
/// external references and are not safe to copy as owned assets.
fn texture_resource_id(value: &str) -> Result<om9_core::identity::ResourceId> {
    om9_core::rhino_semantics::embedded_texture_id(value)
}

fn texture_resources(document: &Document, limits: &Limits) -> Result<std::collections::BTreeSet<om9_core::identity::ResourceId>> {
    let mut ids = std::collections::BTreeSet::new();
    for entity in document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members)) {
        if let Some(uv) = om9_core::rhino_semantics::mesh_uv_of(entity, limits)? {
            if let Some(texture) = &uv.pbr.texture {
                let id = texture_resource_id(texture)?;
                let resource = document.resources.get(&id).ok_or(Error::Missing("clipboard texture resource"))?;
                if !matches!(resource.kind, om9_core::resource::ResourceKind::OpaqueBinary) {
                    return Err(Error::Invalid("clipboard texture resource kind"));
                }
                resource.validate(limits)?;
                ids.insert(id);
            }
        }
    }
    Ok(ids)
}

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
                if let Some(texture) = &uv.pbr.texture { texture_resource_id(texture)?; }
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

/// Expand Job Bag roots to include inputs and sibling outputs needed by clean
/// history, features, FourRail profiles, and bound top-level native dependencies. Clipboard Copy retains its normal
/// explicit-selection behavior; this closure is specific to saved bags.
pub(crate) fn jobbag_selection_closure(document: &Document, roots: &[ObjectId], limits: &Limits) -> Result<Vec<ObjectId>> {
    use std::collections::BTreeSet;
    let mut selected: BTreeSet<_> = roots.iter().copied().collect();
    if selected.len() != roots.len() || selected.is_empty() {
        return Err(Error::Invalid("jobbag selection"));
    }
    loop {
        let before = selected.len();
        for recipe in document.history.recipes.values() {
            if recipe.outputs.iter().any(|id| selected.contains(id)) {
                selected.extend(recipe.outputs.iter().copied());
                selected.extend(recipe.inputs.iter().map(|input| input.source.object));
            }
        }
        for feature in document.features.values() {
            if feature.outputs.iter().any(|id| selected.contains(id)) {
                selected.extend(feature.outputs.iter().copied());
            }
        }
        let mut definitions = BTreeSet::new();
        let mut pending_definitions = Vec::new();
        let mut rails = Vec::new();
        for id in &selected {
            let entity = document.entities.get(id).ok_or(Error::Missing("jobbag dependency object"))?;
            if let Some(profile) = om9_core::rhino_semantics::four_rail_profile_of(entity, limits)? {
                rails.extend(profile.rails);
            }
            if let Geometry::Instance(id) = entity.geometry { pending_definitions.push(id); }
        }
        while let Some(id) = pending_definitions.pop() {
            if !definitions.insert(id) { continue; }
            let definition = document.definitions.get(&id).ok_or(Error::Missing("jobbag definition"))?;
            for entity in &definition.members {
                if let Some(profile) = om9_core::rhino_semantics::four_rail_profile_of(entity, limits)? {
                    rails.extend(profile.rails);
                }
                if let Geometry::Instance(id) = entity.geometry { pending_definitions.push(id); }
            }
        }
        selected.extend(rails);
        let hosts = jobbag_hosts(document, &selected, limits)?;
        for host in &hosts {
            if let om9_core::retained::HostRef::Definition(id) = host {
                let definition = document.definitions.get(id).ok_or(Error::Missing("jobbag native definition"))?;
                for member in &definition.members {
                    if let Some(profile) = om9_core::rhino_semantics::four_rail_profile_of(member, limits)? {
                        selected.extend(profile.rails);
                    }
                }
            }
        }
        let native = om9_core::retained::selected_retained_closure(document, &hosts, limits)?;
        for (namespace, keys) in native {
            for record in &document.import_namespaces[&namespace].records {
                if keys.binary_search(&record.key).is_ok() {
                    if let Some(om9_core::retained::HostRef::Object(id)) = record.host {
                        if document.entities.contains_key(&id) { selected.insert(id); }
                    }
                }
            }
        }
        if selected.len() as u64 > limits.max_objects || selected.len() as u64 > limits.max_nodes {
            return Err(Error::Budget("jobbag dependencies"));
        }
        if selected.len() == before { return Ok(selected.into_iter().collect()); }
    }
}

/// Collect selected root, definition, member and layer hosts for native closure.
fn jobbag_hosts(document: &Document, roots: &std::collections::BTreeSet<ObjectId>, limits: &Limits) -> Result<std::collections::BTreeSet<om9_core::retained::HostRef>> {
    use om9_core::retained::HostRef;
    use std::collections::BTreeSet;
    let mut definitions = BTreeSet::new();
    let mut layers = BTreeSet::new();
    let mut pending_definitions = Vec::new();
    let mut pending_layers = Vec::new();
    for id in roots {
        let entity = document.entities.get(id).ok_or(Error::Missing("jobbag dependency object"))?;
        pending_layers.extend(entity.attributes.layer);
        if let Geometry::Instance(id) = entity.geometry { pending_definitions.push(id); }
    }
    while let Some(id) = pending_definitions.pop() {
        if !definitions.insert(id) { continue; }
        let definition = document.definitions.get(&id).ok_or(Error::Missing("jobbag definition"))?;
        for member in &definition.members {
            pending_layers.extend(member.attributes.layer);
            if let Geometry::Instance(id) = member.geometry { pending_definitions.push(id); }
        }
    }
    while let Some(id) = pending_layers.pop() {
        if layers.insert(id) {
            pending_layers.extend(document.layers.get(&id).ok_or(Error::Missing("jobbag layer"))?.parent);
        }
    }
    let mut hosts = BTreeSet::new();
    hosts.extend(roots.iter().copied().map(HostRef::Object));
    hosts.extend(layers.into_iter().map(HostRef::Layer));
    for definition in definitions {
        hosts.insert(HostRef::Definition(definition));
        for member in &document.definitions[&definition].members {
            hosts.insert(HostRef::DefinitionMember { definition, object: member.id });
        }
    }
    loop {
        let before = hosts.len();
        let closure = om9_core::retained::selected_retained_closure(document, &hosts, limits)?;
        for (namespace, keys) in closure {
            for record in &document.import_namespaces[&namespace].records {
                if keys.binary_search(&record.key).is_ok() {
                    match record.host {
                        Some(HostRef::Layer(id)) => pending_layers.push(id),
                        Some(HostRef::Definition(id) | HostRef::DefinitionMember { definition: id, .. }) => pending_definitions.push(id),
                        _ => {}
                    }
                }
            }
        }
        while let Some(id) = pending_definitions.pop() {
            if hosts.insert(HostRef::Definition(id)) {
                let definition = document.definitions.get(&id).ok_or(Error::Missing("jobbag native definition"))?;
                for member in &definition.members {
                    hosts.insert(HostRef::DefinitionMember { definition: id, object: member.id });
                    pending_layers.extend(member.attributes.layer);
                    if let Geometry::Instance(child) = member.geometry { pending_definitions.push(child); }
                }
            }
        }
        while let Some(id) = pending_layers.pop() {
            if hosts.insert(HostRef::Layer(id)) {
                pending_layers.extend(document.layers.get(&id).ok_or(Error::Missing("jobbag native layer"))?.parent);
            }
        }
        if hosts.len() as u64 > limits.max_nodes { return Err(Error::Budget("jobbag native hosts")); }
        if hosts.len() == before { break; }
    }
    Ok(hosts)
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
    pending.extend(source.import_namespaces.values().flat_map(|namespace| &namespace.records).filter_map(|record| {
        match record.host { Some(om9_core::retained::HostRef::Layer(id)) => Some(id), _ => None }
    }));
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
        self.copy_selected_om9_inner(false, None)
    }

    /// A Job Bag must not recursively capture the manager that contains it.
    pub fn copy_selected_jobbag_om9(&self) -> Result<Vec<u8>> {
        self.copy_selected_om9_inner(true, None)
    }

    pub fn copy_jobbag_objects_om9(&self, ids: &[ObjectId]) -> Result<Vec<u8>> {
        self.copy_selected_om9_inner(true, Some(ids))
    }

    fn copy_selected_om9_inner(&self, omit_project_state: bool, explicit_roots: Option<&[ObjectId]>) -> Result<Vec<u8>> {
        let capture = self.prepare_om9_selection(omit_project_state, explicit_roots)?;
        let bytes = om9_archive::Archive::encode(capture.document(), self.coordinator.limits())?;
        capture.revalidate(&self.coordinator)?;
        Ok(bytes)
    }

    pub fn prepare_jobbag_capture(&self, roots: &[ObjectId]) -> Result<om9_exchange::ExportCapture> {
        self.prepare_om9_selection(true, Some(roots))
    }

    fn prepare_om9_selection(&self, omit_project_state: bool, explicit_roots: Option<&[ObjectId]>) -> Result<om9_exchange::ExportCapture> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let document = self.document();
        let roots: Vec<_> = explicit_roots.map_or_else(|| self.selected().iter().copied().collect(), |ids| ids.to_vec());
        let ids = if omit_project_state {
            jobbag_selection_closure(document, &roots, self.coordinator.limits())?
        } else {
            roots
        };
        for id in &ids {
            let entity = document
                .entities
                .get(id)
                .ok_or(Error::Missing("selected object"))?;
            validate_extensions(entity, self.coordinator.limits())?;
        }
        let mut capture =
            om9_exchange::ExportCapture::selected_om9(&self.coordinator, self.document, &ids)?;
        if omit_project_state {
            let roots = ids.iter().copied().collect();
            capture.retain_selected_sources(&jobbag_hosts(document, &roots, self.coordinator.limits())?, self.coordinator.limits())?;
            capture.omit_resource_named(super::project_state::PROJECT_STATE_RESOURCE, self.coordinator.limits())?;
            capture.omit_resources_with_prefix(super::project_state::PROJECT_ASSET_PREFIX, self.coordinator.limits())?;
        }
        for definition in capture.document().definitions.values() {
            for member in &definition.members {
                validate_extensions(member, self.coordinator.limits())?;
            }
        }
        validate_rail_dependencies(capture.document(), self.coordinator.limits())?;
        exchange_dependencies(capture.document())?;
        texture_resources(capture.document(), self.coordinator.limits())?;
        Ok(capture)
    }

    /// Paste the archive as one transaction; unsupported dependencies never partially commit.
    pub fn paste_om9(&mut self, bytes: &[u8]) -> Result<Vec<ObjectId>> {
        self.paste_om9_checked(bytes, None, false)
    }

    pub fn insert_jobbag_archive(&mut self, bytes: &[u8], snapshot: &crate::project_manager::Snapshot) -> Result<Vec<ObjectId>> {
        self.paste_om9_checked(bytes, Some(snapshot), false)
    }

    pub(crate) fn insert_jobbag_archive_accounted(&mut self, bytes: &[u8], snapshot: &crate::project_manager::Snapshot, input_accounted: bool) -> Result<Vec<ObjectId>> {
        self.paste_om9_checked(bytes, Some(snapshot), input_accounted)
    }

    /// Classify and project a detached bag without publishing to the live document.
    #[cfg(feature = "occt-brep")]
    pub(crate) fn prepare_bound_jobbag_document(coordinator: &om9_core::transaction::TransactionCoordinator,
        bound: crate::brep_restore_host::BoundBrepDocument, snapshot: &crate::project_manager::Snapshot,
        filter: crate::project_manager::BagFilter, target: DocumentId, held: Usage, auxiliary: Usage,
        cancel: &om9_core::transaction::Cancellation) -> Result<PreparedBrepJobbag> {
        let (document, owners, context) = bound.into_replacement_parts();
        let result = (|| {
            let check = || if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) };
            check()?;
            if coordinator.capture_context(None)?.0 != context { return Err(Error::Stale("BRep Job Bag context")); }
            let limits = *coordinator.limits();
            let (document, projection) = if filter == crate::project_manager::BagFilter::All {
                snapshot.validate_archive_roots(&document, &limits)?;
                check()?;
                (document, None)
            } else {
            let roots = snapshot.filtered_ids_in_document(filter, &document, &limits)?;
            if roots.is_empty() { return Err(Error::Invalid("jobbag filter matches no objects")); }
            check()?;
            let ids = jobbag_selection_closure(&document, &roots, &limits)?;
            check()?;
            let id = document.id;
            let world = om9_core::transaction::World::new(vec![document], &limits)?;
            let mut source = om9_core::transaction::TransactionCoordinator::new_with_held(world, limits, held)?;
            source.adopt_external_captures(coordinator)?;
            source.share_brep_representations_for_replacement(coordinator, held, check)?;
            let mut capture = om9_exchange::ExportCapture::selected_om9(&source, id, &ids)?;
            check()?;
            let selected: std::collections::BTreeSet<_> = ids.iter().copied().collect();
            let original = source.world().documents.get(&id).ok_or(Error::Missing("jobbag source document"))?;
            capture.retain_selected_sources(&jobbag_hosts(original, &selected, &limits)?, &limits)?;
            capture.omit_resource_named(super::project_state::PROJECT_STATE_RESOURCE, &limits)?;
            capture.omit_resources_with_prefix(super::project_state::PROJECT_ASSET_PREFIX, &limits)?;
            capture.revalidate(&source)?;
            let (document, projection) = capture.into_projected_document()?;
            check()?;
            (document, Some(projection))
            };
            let estimate = document.validate(&limits)?.bytes.checked_mul(4).and_then(|n| n.checked_add(16 * 1024 * 1024))
                .ok_or(Error::Overflow("BRep Job Bag paste preparation"))?;
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            let working = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
                estimate.max(1), physical, auxiliary)?;
            let (mutations, ids) = Self::build_clipboard_mutations(coordinator, target, document, None)?;
            check()?;
            let mut request = TransactionRequest::new("Paste", mutations);
            request.cancel = cancel.clone();
            let actual = request.worker_input_bytes(&limits)?.checked_add(Usage::array::<ObjectId>(ids.capacity())?.bytes)
                .ok_or(Error::Overflow("BRep Job Bag paste input"))?;
            if actual > working.bytes() { return Err(Error::Budget("BRep Job Bag paste preparation")); }
            let mut paste_held = auxiliary;
            paste_held.add(Usage::array::<ObjectId>(ids.len())?, &limits)?;
            paste_held.add(Usage::btree::<ObjectId, ()>(ids.len())?, &limits)?;
            let mut transaction = coordinator.prepare_with_held_usage(request, paste_held)?;
            coordinator.register_prepared_document(&transaction, target)?;
            let staging = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
                transaction.worker_staging_allowance_bytes().max(1), physical, paste_held)?;
            coordinator.reserve_prepared_held_usage(&mut transaction, paste_held)?;
            check()?;
            let epoch = coordinator.epoch().checked_add(1).ok_or(Error::Overflow("jobbag scene epoch"))?;
            let scene = Self::prepare_history_scene(coordinator, transaction.candidate_world(), epoch, target,
                paste_held, Usage::default(), None)?;
            let scene = super::topology_remesh::prepare_missing_scene_wires(coordinator, scene, paste_held)?;
            check()?;
            Ok(PreparedBrepJobbag { transaction, scene, ids, context, _owners: owners,
                _projection: projection, _working: working, staging, peer: None, cache: None })
        })();
        if result.is_err() { coordinator.prune_orphaned_brep_representations(); coordinator.prune_orphaned_native_representations(); }
        result
    }

    /// Publish one already validated projection through the existing atomic paste.
    #[cfg(feature = "occt-brep")]
    pub(crate) fn insert_prepared_brep_jobbag(&mut self, prepared: PreparedBrepJobbag,
        cancel: &om9_core::transaction::Cancellation) -> Result<Vec<ObjectId>> {
        let result = (|| {
            if self.is_running() { return Err(Error::Busy); }
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if self.coordinator.capture_context(None)?.0 != prepared.context { return Err(Error::Stale("BRep Job Bag context")); }
            let PreparedBrepJobbag { mut transaction, scene, ids, _owners, _projection, _working, staging, peer, mut cache, .. } = prepared;
            let limits = *self.coordinator.limits();
            let mut held = self.scene_cache_usage()?;
            held.add(Usage::array::<ObjectId>(ids.len())?, &limits)?;
            held.add(Usage::btree::<ObjectId, ()>(ids.len())?, &limits)?;
            self.coordinator.reserve_prepared_held_usage(&mut transaction, held)?;
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if let Some(candidate) = &cache {
                if std::iter::successors(self.brep_archive_cache.as_deref(), |entry| entry.previous.as_deref())
                    .any(|entry| entry.same_shared_archive(candidate)) {
                    cache = None;
                }
            }
            if let Some(entry) = &mut cache {
                Arc::get_mut(entry).ok_or(Error::Invalid("shared staged BRep archive cache"))?.previous = self.brep_archive_cache.clone();
            }
            drop((staging, peer));
            let receipt = self.coordinator.commit(transaction)?;
            let ReservedCandidateScene { scene, buffer_owner, reservation } = scene;
            if receipt.published {
                if let Some(cache) = cache { self.brep_archive_cache = Some(cache); }
                self.publish_owned_scene(scene, buffer_owner);
                self.sync_published(om9_core::changes::ChangeCause::Edit);
                self.select_clipboard_roots(&ids);
            }
            drop((reservation, _owners, _projection, _working));
            Ok(ids)
        })();
        if result.is_err() { self.coordinator.prune_orphaned_brep_representations(); self.coordinator.prune_orphaned_native_representations(); }
        result
    }

    #[cfg(all(feature = "occt-brep", feature = "test-fixtures"))]
    pub fn test_insert_bound_jobbag_document(&mut self, bound: crate::brep_restore_host::BoundBrepDocument,
        snapshot: &crate::project_manager::Snapshot)
        -> Result<Vec<ObjectId>> {
        let cancel = om9_core::transaction::Cancellation::default();
        let held = self.replacement_hold(Usage::default())?;
        let auxiliary = self.scene_cache_usage()?;
        let peer = self.coordinator.capture_transaction_preparation_peer(auxiliary)?;
        let prepared = Self::prepare_bound_jobbag_document(peer.coordinator(), bound, snapshot,
            crate::project_manager::BagFilter::All, self.document, held, auxiliary, &cancel)?.retain_peer(peer);
        self.insert_prepared_brep_jobbag(prepared, &cancel)
    }

    fn paste_om9_checked(&mut self, bytes: &[u8], snapshot: Option<&crate::project_manager::Snapshot>, input_accounted: bool) -> Result<Vec<ObjectId>> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let (_source_reservation, source) = super::project_state::decode_project_archive_reserved(
            &self.coordinator, bytes, self.scene_cache_usage()?, None, input_accounted,
        )?;
        self.paste_decoded_om9(source, snapshot)
    }

    /// Paste an already admitted detached document through the same dependency
    /// remapping and atomic scene publication used by the ordinary clipboard.
    fn paste_decoded_om9(&mut self, source: om9_core::model::Document,
        snapshot: Option<&crate::project_manager::Snapshot>) -> Result<Vec<ObjectId>> {
        let (mutations, ids) = Self::build_clipboard_mutations(&self.coordinator, self.document, source, snapshot)?;
        self.commit_clipboard_paste(mutations, &ids)?;
        Ok(ids)
    }

    fn build_clipboard_mutations(coordinator: &TransactionCoordinator, document_id: DocumentId,
        source: om9_core::model::Document, snapshot: Option<&crate::project_manager::Snapshot>) -> Result<(Vec<Mutation>, Vec<ObjectId>)> {
        if let Some(snapshot) = snapshot { snapshot.validate_archive_roots(&source, coordinator.limits())?; }
        let target = coordinator.world().documents.get(&document_id).ok_or(Error::Missing("clipboard target document"))?;
        if source.page_unit != target.page_unit {
            return Err(Error::Unsupported("clipboard document units differ"));
        }
        if source.entities.is_empty() {
            return Err(Error::Unsupported(
                "clipboard object has dependent metadata or history",
            ));
        }
        let mut retained = om9_core::retained::RetainedSelection {
            resources: std::collections::BTreeMap::new(),
            sources: source.retained_sources.clone(),
            namespaces: source.import_namespaces.clone(),
        };
        for retained_source in retained.sources.values() {
            if !retained.namespaces.values().any(|namespace| namespace.source == retained_source.id) {
                return Err(Error::Unsupported("clipboard unbound retained source"));
            }
            let resource = source.resources.get(&retained_source.resource).ok_or(Error::Missing("clipboard retained resource"))?;
            retained.resources.insert(resource.id, resource.clone());
        }
        let retained_layers: std::collections::BTreeSet<_> = retained.namespaces.values().flat_map(|namespace| &namespace.records)
            .filter_map(|record| match record.host { Some(om9_core::retained::HostRef::Layer(id)) => Some(id), _ => None }).collect();
        let textures = texture_resources(&source, coordinator.limits())?;
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
            validate_extensions(entity, coordinator.limits())?;
        }
        validate_rail_dependencies(&source, coordinator.limits())?;
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
        pending.extend(retained_layers.iter().copied());
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
        let mut texture_ids = std::collections::BTreeMap::new();
        for old in textures {
            let mut resource = source.resources.get(&old).ok_or(Error::Missing("clipboard texture resource"))?.clone();
            resource.id = om9_core::identity::ResourceId::new();
            texture_ids.insert(old, resource.id);
            mutations.push(Mutation::ResourceInsert { document: document_id, expected_revision: target.revision, resource });
        }
        if !new_records.line_patterns.is_empty() || !new_records.materials.is_empty() {
            mutations.push(Mutation::ExchangeInsert {
                document: document_id,
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
            let existing = target.layers.values().filter(|_| !retained_layers.contains(&ready)).find(|candidate| {
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
                    coordinator,
                    document_id,
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
                    coordinator.limits(),
                )?;
                for rail in &mut profile.rails {
                    *rail = *object_ids
                        .get(rail)
                        .ok_or(Error::Missing("clipboard rail identity"))?;
                }
                *extension = profile.to_extension()?;
            }
            if let Some(extension) = entity.extensions.get_mut(om9_core::model::MESH_UV_EXTENSION_SLOT) {
                let mut uv = om9_core::rhino_semantics::MeshUv::from_extension(extension, coordinator.limits())?;
                if let Some(texture) = &mut uv.pbr.texture {
                    let old = texture_resource_id(texture)?;
                    let id = texture_ids.get(&old).ok_or(Error::Missing("clipboard texture remap"))?;
                    *texture = om9_core::rhino_semantics::embedded_texture_uri(*id)?;
                    *extension = uv.to_extension()?;
                }
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
                document: document_id,
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
            document: document_id,
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
                document: document_id,
                expected_revision: target.revision,
                recipe: Box::new(recipe),
                published,
            });
        }
        if !source.history.references.is_empty() {
            let mut references = Vec::new();
            references.try_reserve_exact(source.history.references.len()).map_err(|_| Error::Budget("clipboard history references"))?;
            for mut reference in source.history.references.into_values() {
                reference.id = om9_core::identity::ReferenceId::new();
                reference.recipe = *history_ids.get(&reference.recipe).ok_or(Error::Missing("clipboard semantic recipe"))?;
                reference.output = *object_ids.get(&reference.output).ok_or(Error::Missing("clipboard semantic output"))?;
                references.push(reference);
            }
            mutations.push(Mutation::HistoryReferencesInsert { document: document_id,
                expected_revision: target.revision, references });
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
                document: document_id,
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
                document: document_id,
                expected_revision: target.revision,
                group: Box::new(group),
            });
        }
        if !retained.namespaces.is_empty() {
            let maps = om9_core::retained::RetainedHostRemap { objects: object_ids, definitions, layers };
            let selection = retained.remap_for_transaction(&maps, coordinator.limits())?;
            mutations.push(Mutation::RetainedSelectionInsert { document: document_id,
                expected_revision: target.revision, selection: Box::new(selection) });
        }
        Ok((mutations, ids))
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
            self.select_clipboard_roots(ids);
        }
        Ok(())
    }

    fn select_clipboard_roots(&mut self, ids: &[ObjectId]) {
        let selectable: Vec<_> = ids.iter().copied().filter(|id| {
            self.scene.as_ref().is_some_and(|scene| scene.objects.iter().any(|object| {
                object.root == *id && object.selectable && object.has_pick_geometry()
            }))
        }).collect();
        if selectable.is_empty() {
            self.clear_selection();
        } else if let Err(error) = self.select_objects(&selectable) {
            self.clear_selection();
            self.events.emit(crate::command_events::EventKind::Warn, "Paste",
                &format!("Committed; result selection was cleared: {error}"), None);
        }
    }
}
