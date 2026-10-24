// SPDX-License-Identifier: LGPL-2.1-or-later
//! Immutable actual-source admission; native data never chooses a program.
use crate::{ImportCapture, NativeExchange, import_dto as dto, staged::native};
use om9_core::{
    Error, Result,
    brep::Brep,
    budget::{Limits, Usage, name},
    exchange::{
        DimStyle, ExchangeMetadata, HatchPattern, HatchPatternLine, LinePattern,
        LinePatternSegment, RenderLight, RenderMaterial, TextStyle,
    },
    geometry::Geometry,
    identity::*,
    model::{Definition, Entity, Group, Layer},
    resource::{Resource, ResourceKind},
    retained::{
        HostRef, ImportBundle, ImportNamespace, NativeRecordKey, NativeRelation, RetainedSource,
        SourceRecord, fingerprint,
    },
    transaction::{Cancellation, PreparedTransaction, TransactionCoordinator},
};
use om9_native_owner::ProcessStats;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportMode {
    #[default]
    Preserve,
    Geometry,
}
/// BRep policy for a native import. `EditableStrict` is the release path when
/// an OCCT bridge is admitted; `RetainOpaque` keeps the current NativeBrep path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrepMode {
    #[default]
    RetainOpaque,
    EditableStrict,
    EditableBestEffort,
}
impl BrepMode {
    fn editable(self) -> bool {
        !matches!(self, Self::RetainOpaque)
    }
    fn strict(self) -> bool {
        matches!(self, Self::EditableStrict)
    }
}
/// Complete conversion operation for a projected 3DM BRep. The exchange layer
/// owns the transaction and budget; the implementation owns only OCCT.
pub trait BrepBridge {
    fn import(
        &self,
        payload: &[u8],
        tolerance_mm: f64,
        deflection_mm: f64,
        limits: &Limits,
    ) -> Result<Brep>;
}
/// Complete OCCT -> 3DM payload operation used only for an export copy.
pub trait BrepExport {
    fn export(&self, brep: &Brep, limits: &Limits) -> Result<om9_core::geometry::NativeBrep>;
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct UnitResolution {
    pub model_mm_per_unit: Option<f64>,
    pub page_mm_per_unit: Option<f64>,
    pub definition_mm_per_unit: BTreeMap<Uuid, f64>,
}
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub mode: ImportMode,
    pub brep: BrepMode,
    pub units: UnitResolution,
    pub max_file_bytes: u64,
}
impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Preserve,
            brep: BrepMode::RetainOpaque,
            units: UnitResolution::default(),
            max_file_bytes: 64 * 1024 * 1024,
        }
    }
}
impl ImportOptions {
    fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.max_file_bytes == 0 || self.max_file_bytes > 64 * 1024 * 1024 {
            return Err(Error::Invalid("source file byte limit"));
        }
        for f in self
            .units
            .model_mm_per_unit
            .into_iter()
            .chain(self.units.page_mm_per_unit)
            .chain(self.units.definition_mm_per_unit.values().copied())
        {
            if !f.is_finite() || f <= 0. {
                return Err(Error::Invalid("explicit source unit factor"));
            }
        }
        if self.units.definition_mm_per_unit.keys().any(Uuid::is_nil) {
            return Err(Error::Invalid("definition resolution identity"));
        }
        let u = Usage::btree::<Uuid, f64>(self.units.definition_mm_per_unit.len())?;
        u.check(l)?;
        Ok(u)
    }
}
#[derive(Debug)]
pub struct ImportReceipt {
    pub source: SourceId,
    pub resource: ResourceId,
    pub namespace: ImportNamespaceId,
    pub source_sha256: [u8; 32],
    pub root_objects: usize,
    pub definition_members: usize,
    pub definitions: usize,
    pub layers: usize,
    pub groups: usize,
    pub line_patterns: usize,
    pub page_space_objects: usize,
    pub text_styles: usize,
    pub materials: usize,
    pub dim_styles: usize,
    pub hatch_patterns: usize,
    pub breps_editable: usize,
    pub breps_opaque: usize,
    pub breps_displayable: usize,
    pub breps_display_unavailable: usize,
    pub annotations: usize,
    pub hatches: usize,
    pub lights: usize,
    pub retained_records: usize,
    pub diagnostics: Vec<String>,
    pub native: ProcessStats,
}
impl ImportReceipt {
    fn usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage::array::<Self>(1)?;
        u.add(Usage::array::<String>(self.diagnostics.capacity())?, l)?;
        for s in &self.diagnostics {
            u.add(
                Usage {
                    bytes: s.capacity() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
        Ok(u)
    }
}
pub struct PreparedNativeImport {
    capture: ImportCapture,
    bundle: ImportBundle,
    receipt: ImportReceipt,
    cancel: Cancellation,
}
impl PreparedNativeImport {
    pub fn receipt(&self) -> &ImportReceipt {
        &self.receipt
    }
    pub fn prepare(
        mut self,
        c: &TransactionCoordinator,
        cancel: &Cancellation,
    ) -> Result<(PreparedTransaction, ImportReceipt)> {
        let combined = self.cancel.either(cancel)?;
        check(&combined)?;
        let held = self.receipt.usage(c.limits())?;
        let target = self.capture.document().id;
        let mut p = self
            .capture
            .prepare_with_held_usage(c, self.bundle, &combined, held)?;
        // Validate display preparation while the document is still private. Old
        // documents/Undo, the candidate, and receipt remain charged together.
        let mut peak = c.retained_usage_with_held_usage(held)?;
        peak.add(p.candidate_world().validate(c.limits())?, c.limits())?;
        let document = p
            .candidate_world()
            .documents
            .get(&target)
            .ok_or(Error::Missing("import scene target"))?;
        let scene = om9_scene::scene::Scene::capture(
            document.clone(),
            c.epoch(),
            &c.limits().remaining_after(peak)?,
        )?;
        let scene_diagnostics = scene.diagnostics;
        drop(scene.objects);
        drop(scene.document);
        let additional = scene_diagnostics.len();
        peak.add(
            Usage::array::<String>(scene_diagnostics.capacity())?,
            c.limits(),
        )?;
        let mut receipt_peak = self.receipt.usage(c.limits())?;
        receipt_peak.add(
            Usage::array::<String>(self.receipt.diagnostics.len() + additional)?,
            c.limits(),
        )?;
        for message in &scene_diagnostics {
            receipt_peak.add(
                Usage {
                    bytes: message.capacity() as u64,
                    ..Usage::default()
                },
                c.limits(),
            )?;
        }
        peak.add(receipt_peak, c.limits())?;
        c.reserve_prepared_held_usage(&mut p, receipt_peak)?;
        self.receipt
            .diagnostics
            .try_reserve_exact(additional)
            .map_err(|_| Error::Budget("import scene diagnostics"))?;
        self.receipt.diagnostics.extend(scene_diagnostics);
        self.capture.revalidate(c)?;
        check(&combined)?;
        Ok((p, self.receipt))
    }
}
fn io(e: std::io::Error) -> Error {
    Error::Io(e.to_string())
}
fn check(cancel: &Cancellation) -> Result<()> {
    if cancel.is_canceled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}
fn locked_read(path: &Path) -> Result<File> {
    let mut o = OpenOptions::new();
    o.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        o.share_mode(1);
    }
    o.open(path).map_err(io)
}
struct Snapshot {
    dir: PathBuf,
    bytes: Vec<u8>,
    sha256: [u8; 32],
    source_name: String,
    input: Option<File>,
    original: Option<File>,
    request: Option<File>,
}
impl Snapshot {
    fn capture(
        source: &Path,
        parent: &Path,
        max: u64,
        held: Usage,
        l: &Limits,
        cancel: &Cancellation,
    ) -> Result<Self> {
        check(cancel)?;
        if !fs::symlink_metadata(source).map_err(io)?.is_file() {
            return Err(Error::Invalid("regular source file"));
        }
        let mut original = locked_read(source)?;
        let meta = original.metadata().map_err(io)?;
        if !meta.is_file() || meta.len() == 0 || meta.len() > max {
            return Err(Error::Budget("source file bytes"));
        }
        if source.as_os_str().len() > 8192 || parent.as_os_str().len() > 8192 {
            return Err(Error::Budget("source staging path bytes"));
        }
        let source_name = source
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(Error::Invalid("source filename"))?;
        name(source_name, l)?;
        let mut peak = held;
        peak.add(
            Usage {
                bytes: meta
                    .len()
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(16384))
                    .ok_or(Error::Overflow("source snapshot copies"))?,
                ..Usage::default()
            },
            l,
        )?;
        let source_name = source_name.to_owned();
        let capacity = usize::try_from(meta.len())
            .map_err(|_| Error::Overflow("source snapshot address space"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| Error::Budget("source snapshot allocation"))?;
        let mut hash = Sha256::new();
        let mut buffer = [0; 8192];
        loop {
            check(cancel)?;
            let n = original.read(&mut buffer).map_err(io)?;
            if n == 0 {
                break;
            }
            if bytes.len().checked_add(n).is_none_or(|n| n > capacity) {
                return Err(Error::Invalid("changing source file"));
            }
            hash.update(&buffer[..n]);
            bytes.extend_from_slice(&buffer[..n]);
        }
        if bytes.len() != capacity || original.metadata().map_err(io)?.len() != meta.len() {
            return Err(Error::Invalid("changing source length"));
        }
        let parent = parent.canonicalize().map_err(io)?;
        if !parent.is_dir() {
            return Err(Error::Invalid("source staging directory"));
        }
        let dir = parent.join(format!(".om9-import-{}", Uuid::new_v4()));
        fs::create_dir(&dir).map_err(io)?;
        let mut snapshot = Self {
            dir,
            bytes,
            sha256: hash.finalize().into(),
            source_name,
            input: None,
            original: Some(original),
            request: None,
        };
        let path = snapshot.source();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(io)?;
        file.write_all(&snapshot.bytes).map_err(io)?;
        file.flush().map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        snapshot.input = Some(locked_read(&path)?);
        Ok(snapshot)
    }
    fn source(&self) -> PathBuf {
        self.dir.join("source.3dm")
    }
    fn resolution(&self) -> PathBuf {
        self.dir.join("resolution.json")
    }
    fn write_resolution(&mut self, units: &UnitResolution) -> Result<()> {
        #[derive(Serialize)]
        struct Packet<'a> {
            schema_version: u32,
            model_mm_per_unit: Option<f64>,
            page_mm_per_unit: Option<f64>,
            definition_mm_per_unit: &'a BTreeMap<Uuid, f64>,
        }
        let p = self.resolution();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&p)
            .map_err(io)?;
        serde_json::to_writer(
            &mut file,
            &Packet {
                schema_version: 1,
                model_mm_per_unit: units.model_mm_per_unit,
                page_mm_per_unit: units.page_mm_per_unit,
                definition_mm_per_unit: &units.definition_mm_per_unit,
            },
        )
        .map_err(|_| Error::Invalid("source resolution serialization"))?;
        file.flush().map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        self.request = Some(locked_read(&p)?);
        Ok(())
    }
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        self.input.take();
        self.request.take();
        self.original.take();
        let _ = fs::remove_file(self.source());
        let _ = fs::remove_file(self.resolution());
        let _ = fs::remove_dir(&self.dir);
    }
}
fn hex(hash: [u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}
impl NativeExchange {
    pub fn prepare_import(
        &self,
        c: &TransactionCoordinator,
        capture: ImportCapture,
        path: &Path,
        staging_parent: &Path,
        options: &ImportOptions,
        cancel: &Cancellation,
    ) -> Result<PreparedNativeImport> {
        self.prepare_import_with_brep(c, capture, path, staging_parent, options, None, cancel)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_import_with_brep(
        &self,
        c: &TransactionCoordinator,
        capture: ImportCapture,
        path: &Path,
        staging_parent: &Path,
        options: &ImportOptions,
        bridge: Option<&dyn BrepBridge>,
        cancel: &Cancellation,
    ) -> Result<PreparedNativeImport> {
        check(cancel)?;
        capture.revalidate(c)?;
        let options_usage = options.validate(c.limits())?;
        let base = c.retained_usage_with_held_usage(options_usage)?;
        let mut running = base;
        running.add(
            Usage {
                bytes: self.limits.reservation_bytes().map_err(native)?,
                ..Usage::default()
            },
            c.limits(),
        )?;
        let mut snapshot = Snapshot::capture(
            path,
            staging_parent,
            options.max_file_bytes,
            running,
            c.limits(),
            cancel,
        )?;
        snapshot.write_resolution(&options.units)?;
        check(cancel)?;
        let output = self
            .program
            .run(
                &[
                    "--project-source".into(),
                    snapshot.source().into_os_string(),
                    snapshot.resolution().into_os_string(),
                ],
                &self.limits,
                cancel,
            )
            .map_err(native)?;
        if !output.stats.exited || output.stats.active_processes != 0 {
            return Err(Error::Invalid("native source owner not reaped"));
        }
        let mut held = base;
        held.add(
            Usage {
                bytes: snapshot.bytes.capacity() as u64 + 16384,
                ..Usage::default()
            },
            c.limits(),
        )?;
        held.add(
            Usage {
                bytes: output.stdout.capacity() as u64 + output.stderr.capacity() as u64,
                ..Usage::default()
            },
            c.limits(),
        )?;
        let response_limits = c.limits().remaining_after(held)?;
        let inventory = dto::Inventory::decode(&output.stdout, &response_limits)?;
        if inventory.archive.file_bytes != snapshot.bytes.len() as u64
            || inventory.archive.source_sha256 != hex(snapshot.sha256)
        {
            return Err(Error::Invalid("native source byte digest"));
        }
        capture.revalidate(c)?;
        check(cancel)?;
        held.add(
            inventory.owned_usage(&c.limits().remaining_after(held)?)?,
            c.limits(),
        )?;
        let (bundle, mut receipt) = convert(
            inventory,
            &snapshot,
            options,
            bridge,
            held,
            c.limits(),
            cancel,
        )?;
        receipt.native = output.stats;
        drop(output.stdout);
        drop(output.stderr);
        drop(snapshot);
        capture.revalidate(c)?;
        check(cancel)?;
        Ok(PreparedNativeImport {
            capture,
            bundle,
            receipt,
            cancel: cancel.clone(),
        })
    }
}

struct Catalog {
    records: Vec<SourceRecord>,
    by_id: BTreeMap<(u32, Uuid), NativeRecordKey>,
    by_index: BTreeMap<(u32, i32), NativeRecordKey>,
    slots: BTreeMap<NativeRecordKey, usize>,
}
impl Catalog {
    fn insert(&mut self, ty: u32, id: Uuid, index: Option<i32>, family: &str) -> Result<()> {
        let key = dto::key(ty, id, index)?;
        if !id.is_nil() && self.by_id.insert((ty, id), key).is_some() {
            return Err(Error::Invalid("duplicate native component UUID"));
        }
        if let Some(index) = index
            && self.by_index.insert((ty, index), key).is_some()
        {
            return Err(Error::Invalid("duplicate native component index"));
        }
        if self.slots.insert(key, self.records.len()).is_some() {
            return Err(Error::Invalid("duplicate native key"));
        }
        self.records.push(SourceRecord::retained(key, family));
        Ok(())
    }
    fn key(&self, ty: u32, id: Uuid) -> Result<NativeRecordKey> {
        self.by_id
            .get(&(ty, id))
            .copied()
            .ok_or(Error::Missing("source component key"))
    }
    fn relations(
        &self,
        relations: &[dto::Relation],
        limits: &Limits,
    ) -> Result<Vec<NativeRelation>> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(relations.len())
            .map_err(|_| Error::Budget("native relations"))?;
        for r in relations {
            if let Some(target) = self.resolve(r, limits)? {
                result.push(NativeRelation {
                    role: r.role.clone(),
                    target,
                });
            }
        }
        Ok(result)
    }
    fn resolve(&self, r: &dto::Relation, limits: &Limits) -> Result<Option<NativeRecordKey>> {
        name(&r.role, limits)?;
        dto::type_number(&r.component_type)?;
        if let Some(transform) = r.transform {
            transform.validate()?;
        }
        if r.scope == "system_or_unset" {
            if r.id.is_some_and(|id| !id.is_nil()) || r.native_index.is_some_and(|index| index >= 0)
            {
                return Err(Error::Invalid("native system relation identity"));
            }
            return Ok(None);
        }
        if r.scope != "source" {
            return Err(Error::Invalid("native relation scope"));
        }
        let ty = dto::type_number(&r.component_type)?;
        let by_id =
            r.id.filter(|id| !id.is_nil())
                .map(|id| self.key(ty, id))
                .transpose()?;
        let by_index = r
            .native_index
            .filter(|i| *i >= 0)
            .map(|i| {
                self.by_index
                    .get(&(ty, i))
                    .copied()
                    .ok_or(Error::Missing("native relation index"))
            })
            .transpose()?;
        let target = match (by_id, by_index) {
            (Some(a), Some(b)) if a == b => a,
            (Some(a), None) | (None, Some(a)) => a,
            _ => {
                return Err(Error::Invalid(
                    "native relation identity/index correspondence",
                ));
            }
        };
        Ok(Some(target))
    }
    fn bind(&mut self, key: NativeRecordKey, host: HostRef, hash: [u8; 32]) -> Result<()> {
        let r = &mut self.records[*self
            .slots
            .get(&key)
            .ok_or(Error::Missing("source record"))?];
        if r.host.is_some() {
            return Err(Error::Invalid("native source record bound twice"));
        }
        r.host = Some(host);
        r.baseline_sha256 = Some(hash);
        r.disposition = om9_core::retained::RecordDisposition::Unchanged;
        Ok(())
    }
    fn bind_entity(&mut self, key: NativeRecordKey, host: HostRef, entity: &Entity) -> Result<()> {
        let slot = *self
            .slots
            .get(&key)
            .ok_or(Error::Missing("source record"))?;
        let r = &self.records[slot];
        if r.host.is_some() {
            return Err(Error::Invalid("native source record bound twice"));
        }
        let bound = SourceRecord::bound_entity(key, &r.family, host, entity)?;
        let r = &mut self.records[slot];
        r.host = bound.host;
        r.baseline_sha256 = bound.baseline_sha256;
        r.entity_baseline = bound.entity_baseline;
        r.disposition = bound.disposition;
        Ok(())
    }
    fn one(
        &self,
        relations: &[dto::Relation],
        role: &str,
        ty: u32,
    ) -> Result<Option<NativeRecordKey>> {
        let mut found = None;
        for r in relations.iter().filter(|r| r.role == role) {
            if found.is_some() || dto::type_number(&r.component_type)? != ty {
                return Err(Error::Invalid("native relation role/type multiplicity"));
            }
            found = Some(r);
        }
        found
            .map(|r| self.resolve(r, &Limits::default()))
            .transpose()
            .map(Option::flatten)
    }
    fn projection_relations(
        &self,
        key: NativeRecordKey,
        relations: &[dto::Relation],
        l: &Limits,
    ) -> Result<()> {
        let mut declared = self.records[self.slots[&key]].relations.iter();
        for relation in relations {
            if let Some(target) = self.resolve(relation, l)?
                && declared
                    .next()
                    .is_none_or(|r| r.role != relation.role || r.target != target)
            {
                return Err(Error::Invalid("native projection relation correspondence"));
            }
        }
        if declared.next().is_some() {
            return Err(Error::Invalid("native projection relation coverage"));
        }
        Ok(())
    }
}
fn default_preservation(p: &dto::Preservation, represented: &[&str]) -> bool {
    p.admission_required
        && p.nondefault_fields
            .keys()
            .all(|k| represented.contains(&k.as_str()))
}
fn loss(
    diagnostics: &mut Vec<String>,
    id: Uuid,
    family: &str,
    p: &dto::Preservation,
    represented: &[&str],
) {
    use std::fmt::Write as _;
    let mut keys = p
        .nondefault_fields
        .keys()
        .filter(|key| !represented.contains(&key.as_str()))
        .peekable();
    if keys.peek().is_none() {
        return;
    }
    let capacity = 256 + keys.clone().map(|key| key.len() + 2).sum::<usize>();
    let mut message = String::with_capacity(capacity);
    let _ = write!(
        message,
        "{id} ({family}): geometry import omits native fields "
    );
    let mut first = true;
    for key in keys {
        if !first {
            message.push_str(", ");
        }
        first = false;
        message.push_str(key);
    }
    message.push_str(" from the host representation; original bytes remain retained.");
    diagnostics.push(message);
}
fn eligible_object(o: &dto::Object, mode: ImportMode, layers: &BTreeMap<Uuid, LayerId>) -> bool {
    let page_space = o.attributes.space == 2;
    o.admission_required
        && (o.preservation.admission_required || page_space)
        && o.native_component_type == 12
        && o.capability == "geometry_projection_exact"
        && o.geometry.is_some()
        && (o.attributes.space == 1 || page_space)
        && o.attributes
            .layer
            .is_some_and(|id| layers.contains_key(&id))
        && (page_space
            || mode == ImportMode::Geometry
            || (o.userdata.is_empty()
                && o.unsupported_fields.is_empty()
                && o.attributes.color_source <= 1
                && default_preservation(&o.preservation, &["color_source", "native_color"])))
}
#[derive(Default)]
struct BrepCounters {
    editable: usize,
    opaque: usize,
    displayable: usize,
    display_unavailable: usize,
}
fn native_display_available(native: &om9_core::geometry::NativeBrep) -> bool {
    !native.display.vertices.is_empty() && !native.display.faces.is_empty()
}
#[allow(clippy::too_many_arguments)]
fn entity(
    mut object: dto::Object,
    layers: &BTreeMap<Uuid, LayerId>,
    line_patterns: &BTreeMap<i32, LinePatternId>,
    materials: &BTreeMap<i32, MaterialId>,
    defs: &BTreeMap<Uuid, DefinitionId>,
    bridge: Option<&dyn BrepBridge>,
    brep_mode: BrepMode,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
    counters: &mut BrepCounters,
    diagnostics: &mut Vec<String>,
) -> Result<Entity> {
    if brep_mode.editable() {
        let conversion = if let Some(Geometry::NativeBrep(native)) = &object.geometry {
            let payload = native.payload.clone();
            Some((payload, object.id))
        } else {
            None
        };
        if let Some((payload, id)) = conversion {
            if let Some(bridge) = bridge {
                match bridge.import(&payload, tolerance_mm, deflection_mm, limits) {
                    Ok(brep) => {
                        object.geometry = Some(Geometry::Brep(brep));
                        counters.editable = counters
                            .editable
                            .checked_add(1)
                            .ok_or(Error::Overflow("editable BRep count"))?;
                        counters.displayable = counters
                            .displayable
                            .checked_add(1)
                            .ok_or(Error::Overflow("displayable BRep count"))?;
                    }
                    Err(error) if brep_mode.strict() => {
                        return Err(Error::Io(format!("3DM BRep {id}: {error}")));
                    }
                    Err(error) => {
                        counters.opaque = counters
                            .opaque
                            .checked_add(1)
                            .ok_or(Error::Overflow("opaque BRep count"))?;
                        if let Some(Geometry::NativeBrep(native)) = &object.geometry
                            && native_display_available(native)
                        {
                            counters.displayable = counters
                                .displayable
                                .checked_add(1)
                                .ok_or(Error::Overflow("displayable BRep count"))?;
                        } else {
                            counters.display_unavailable = counters
                                .display_unavailable
                                .checked_add(1)
                                .ok_or(Error::Overflow("display-unavailable BRep count"))?;
                        }
                        diagnostics.push(format!("{id}: BREP_CONVERSION_FAILED: {error}"));
                    }
                }
            } else if brep_mode.strict() {
                return Err(Error::Missing("BRep conversion backend"));
            } else {
                counters.opaque = counters
                    .opaque
                    .checked_add(1)
                    .ok_or(Error::Overflow("opaque BRep count"))?;
                if let Some(Geometry::NativeBrep(native)) = &object.geometry
                    && native_display_available(native)
                {
                    counters.displayable = counters
                        .displayable
                        .checked_add(1)
                        .ok_or(Error::Overflow("displayable BRep count"))?;
                } else {
                    counters.display_unavailable = counters
                        .display_unavailable
                        .checked_add(1)
                        .ok_or(Error::Overflow("display-unavailable BRep count"))?;
                }
                diagnostics.push(format!(
                    "{id}: BREP_CONVERSION_UNAVAILABLE: no editable backend; retained opaque"
                ));
            }
        }
    } else if let Some(Geometry::NativeBrep(native)) = &object.geometry {
        counters.opaque = counters
            .opaque
            .checked_add(1)
            .ok_or(Error::Overflow("opaque BRep count"))?;
        if native_display_available(native) {
            counters.displayable = counters
                .displayable
                .checked_add(1)
                .ok_or(Error::Overflow("displayable BRep count"))?;
        } else {
            counters.display_unavailable = counters
                .display_unavailable
                .checked_add(1)
                .ok_or(Error::Overflow("display-unavailable BRep count"))?;
            diagnostics.push(format!(
                "{}: BREP_DISPLAY_UNAVAILABLE: retained BRep has no display mesh; use a build with occt-brep",
                object.id
            ));
        }
    }
    let mut geometry = object
        .geometry
        .take()
        .ok_or(Error::Missing("native projected geometry"))?;
    if let Geometry::Instance(id) = &mut geometry {
        *id = *defs
            .get(&id.0)
            .ok_or(Error::Missing("admitted native definition"))?;
    }
    let layer = object
        .attributes
        .layer
        .map(|id| {
            layers
                .get(&id)
                .copied()
                .ok_or(Error::Missing("admitted object layer"))
        })
        .transpose()?;
    let mut e = Entity::new(object.label, geometry, layer);
    e.placement = object
        .placement
        .ok_or(Error::Missing("native projected placement"))?;
    e.attributes.visible = object.attributes.visible;
    e.attributes.locked = object.attributes.locked;
    e.attributes.selectable = true;
    e.attributes.color = if object.attributes.color_source == 1 {
        object.attributes.color
    } else {
        None
    };
    e.attributes.line_pattern = line_patterns
        .get(&object.attributes.native_line_pattern_index)
        .copied();
    e.attributes.line_pattern_source = object.attributes.line_pattern_source;
    e.attributes.material = materials
        .get(&object.attributes.native_material_index)
        .copied();
    e.attributes.material_source = object.attributes.material_source;
    e.attributes.plot_color = object.attributes.plot_color;
    e.attributes.plot_weight_mm =
        (object.attributes.plot_weight_mm >= 0.).then_some(object.attributes.plot_weight_mm);
    e.attributes.plot_color_source = Some(object.attributes.plot_color_source);
    e.attributes.plot_weight_source = Some(object.attributes.plot_weight_source);
    e.attributes.display_order = object.attributes.display_order;
    e.attributes.space = object.attributes.space;
    e.attributes.viewport_id = object.attributes.viewport_id.map(ViewportId);
    Ok(e)
}
fn convert(
    mut d: dto::Inventory,
    snapshot: &Snapshot,
    options: &ImportOptions,
    bridge: Option<&dyn BrepBridge>,
    mut held: Usage,
    l: &Limits,
    cancel: &Cancellation,
) -> Result<(ImportBundle, ImportReceipt)> {
    let count = d
        .components
        .len()
        .checked_add(d.objects.len())
        .ok_or(Error::Overflow("source record count"))?;
    let annotation_count = d
        .objects
        .iter()
        .filter(|object| {
            object.native_kind.contains("Annotation")
                || object.native_kind.contains("Dimension")
                || object.native_kind.contains("Text")
        })
        .count();
    let hatch_count = d
        .objects
        .iter()
        .filter(|object| object.native_kind.contains("Hatch"))
        .count();
    let light_count = d
        .components
        .iter()
        .filter(|record| record.native_kind.contains("Light"))
        .count()
        + d.objects
            .iter()
            .filter(|record| record.native_kind.contains("Light"))
            .count();
    // Preflight conservative simultaneous DTO/conversion/map/resource copies.
    held.add(
        Usage {
            nodes: count as u64,
            ..Usage::default()
        },
        l,
    )?;
    held.add(
        Usage {
            bytes: d.units.model.name.len() as u64 + d.units.page.name.len() as u64,
            ..Usage::default()
        },
        l,
    )?;
    held.add(Usage::array::<SourceRecord>(count)?, l)?;
    for usage in [
        Usage::btree::<(u32, Uuid), NativeRecordKey>(count)?,
        Usage::btree::<(u32, i32), NativeRecordKey>(count)?,
        Usage::btree::<NativeRecordKey, usize>(count)?,
        Usage::btree::<Uuid, LayerId>(d.layers.len())?,
        Usage::btree::<Uuid, DefinitionId>(d.definitions.len())?,
        Usage::btree::<Uuid, usize>(d.objects.len())?,
        Usage::btree::<Uuid, dto::Layer>(d.layers.len())?,
        Usage::btree::<Uuid, dto::Definition>(d.definitions.len())?,
        Usage::btree::<Uuid, Uuid>(d.objects.len())?,
        Usage::btree::<Uuid, Vec<Uuid>>(d.layers.len())?,
        Usage::btree::<Uuid, Vec<Uuid>>(d.definitions.len())?,
        Usage::btree::<Uuid, ()>(d.definitions.len())?,
        Usage::btree::<Uuid, ()>(d.definitions.len())?,
        Usage::array::<Uuid>(d.layers.len() + d.objects.len())?,
        Usage::array::<Option<dto::Object>>(d.objects.len())?,
        Usage::array::<Vec<NativeRelation>>(count)?,
        Usage::array::<Entity>(d.objects.len())?,
        Usage::array::<Layer>(d.layers.len())?,
        Usage::array::<Definition>(d.definitions.len())?,
    ] {
        held.add(usage, l)?;
    }
    for usage in [
        crate::import_graph::workspace_usage(d.layers.len())?,
        crate::import_graph::workspace_usage(d.definitions.len())?,
    ] {
        held.add(usage, l)?;
    }
    for (family, relations) in d
        .components
        .iter()
        .map(|c| (&c.native_kind, &c.relations))
        .chain(d.objects.iter().map(|o| (&o.native_kind, &o.relations)))
    {
        name(family, l)?;
        held.add(
            Usage {
                bytes: family.len() as u64,
                ..Usage::default()
            },
            l,
        )?;
        held.add(Usage::array::<NativeRelation>(relations.len())?, l)?;
        for r in relations {
            held.add(
                Usage {
                    bytes: r.role.len() as u64,
                    ..Usage::default()
                },
                l,
            )?;
        }
    }
    held.add(
        Usage::btree::<Uuid, om9_core::retained::DefinitionUnits>(d.definitions.len())?,
        l,
    )?;
    for def in &d.definitions {
        name(&def.units.name, l)?;
        held.add(
            Usage {
                bytes: def.units.name.len() as u64 + def.label.len() as u64,
                ..Usage::default()
            },
            l,
        )?;
    }
    for layer in &d.layers {
        held.add(
            Usage {
                bytes: layer.name.len() as u64,
                ..Usage::default()
            },
            l,
        )?;
    }
    let mut source_units = d.source_units(
        options.units.model_mm_per_unit,
        options.units.page_mm_per_unit,
    )?;
    for def in &d.definitions {
        let units = d.definition_units(
            def,
            options.units.definition_mm_per_unit.get(&def.id).copied(),
        )?;
        if source_units.definitions.insert(def.id, units).is_some() {
            return Err(Error::Invalid("duplicate native definition unit context"));
        }
    }
    let brep_tolerance_mm =
        (source_units.model.absolute_tolerance * source_units.model.millimeters_per_unit).max(1e-7);
    let brep_deflection_mm = 0.02_f64.max(brep_tolerance_mm * 2.);
    let mut brep_counters = BrepCounters::default();
    if options.brep.strict() && bridge.is_none() {
        return Err(Error::Missing("BRep conversion backend"));
    }
    let mut catalog = Catalog {
        records: Vec::new(),
        by_id: BTreeMap::new(),
        by_index: BTreeMap::new(),
        slots: BTreeMap::new(),
    };
    catalog
        .records
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("source record allocation"))?;
    for c in &d.components {
        check(cancel)?;
        if !c.admission_required || dto::type_number(&c.component_type)? != c.native_component_type
        {
            return Err(Error::Invalid("native component type"));
        }
        catalog.insert(
            c.native_component_type,
            c.id,
            c.native_index,
            &c.native_kind,
        )?;
    }
    for o in &d.objects {
        check(cancel)?;
        if o.native_component_type != 12 || o.component_type.is_some() {
            return Err(Error::Invalid("native geometry component type"));
        }
        catalog.insert(
            o.native_component_type,
            o.id,
            o.native_index,
            &o.native_kind,
        )?;
        if let Some(placement) = o.placement {
            placement.validate()?;
        }
    }
    let mut relation_sets = Vec::new();
    relation_sets
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("source relation sets"))?;
    for relations in d
        .components
        .iter()
        .map(|c| &c.relations)
        .chain(d.objects.iter().map(|o| &o.relations))
    {
        relation_sets.push(catalog.relations(relations, l)?);
    }
    for (record, relations) in catalog.records.iter_mut().zip(relation_sets) {
        record.relations = relations;
    }
    let mut diagnostics = std::mem::take(&mut d.diagnostics);
    let mut diagnostic_count = diagnostics
        .len()
        .checked_add(2 + count + d.layers.len() + d.definitions.len())
        .ok_or(Error::Overflow("import diagnostic count"))?;
    for object in &d.objects {
        diagnostic_count = diagnostic_count
            .checked_add(object.diagnostics.len())
            .ok_or(Error::Overflow("import object diagnostic count"))?;
    }
    held.add(Usage::array::<String>(diagnostic_count)?, l)?;
    for (p, extras, native_diagnostics) in d
        .objects
        .iter()
        .map(|o| {
            (
                &o.preservation,
                o.unsupported_fields.as_slice(),
                o.diagnostics.as_slice(),
            )
        })
        .chain(
            d.layers
                .iter()
                .map(|layer| (&layer.preservation, &[][..], &[][..])),
        )
        .chain(
            d.definitions
                .iter()
                .map(|def| (&def.preservation, &[][..], &[][..])),
        )
    {
        let bytes = p
            .nondefault_fields
            .keys()
            .map(|s| s.len() as u64 + 2)
            .chain(extras.iter().map(|s| s.len() as u64 + 2))
            .try_fold(512u64, |sum, n| {
                sum.checked_add(n)
                    .ok_or(Error::Overflow("import diagnostic bytes"))
            })?;
        held.add(
            Usage {
                bytes,
                ..Usage::default()
            },
            l,
        )?;
        for message in native_diagnostics {
            held.add(
                Usage {
                    bytes: message.len() as u64 + 40,
                    ..Usage::default()
                },
                l,
            )?;
        }
    }
    held.add(
        Usage {
            bytes: (count as u64)
                .checked_mul(192)
                .and_then(|n| n.checked_add(1024))
                .ok_or(Error::Overflow("import relation diagnostics"))?,
            ..Usage::default()
        },
        l,
    )?;
    diagnostics
        .try_reserve_exact(diagnostic_count - diagnostics.len())
        .map_err(|_| Error::Budget("import diagnostics"))?;
    for object in &d.objects {
        for message in &object.diagnostics {
            diagnostics.push(format!("{}: {message}", object.id));
        }
    }
    for (id, relations) in d
        .components
        .iter()
        .map(|c| (c.id, &c.relations))
        .chain(d.objects.iter().map(|o| (o.id, &o.relations)))
    {
        if relations.iter().any(|r| r.scope == "system_or_unset") {
            diagnostics.push(format!("{id}: native system/default component references remain in original source bytes; they are not namespace-local host bindings."));
        }
    }
    if !d.user_tables.is_empty() {
        diagnostics.push(format!("{} native user tables remain in the immutable original source; attachment remapping is not an import capability.",d.user_tables.len()));
    }
    let mut layer_ids = BTreeMap::new();
    let mut layer_data = BTreeMap::new();
    let layer_order: Vec<_> = d.layers.iter().map(|layer| layer.id).collect();
    for layer in d.layers {
        if !layer.admission_required
            || layer.native_component_type != 5
            || catalog.key(5, layer.id)?.native_index != Some(layer.native_index)
        {
            return Err(Error::Invalid("native layer projection key"));
        }
        let key = catalog.key(5, layer.id)?;
        catalog.projection_relations(key, &layer.relations, l)?;
        if catalog
            .one(&layer.relations, "parent_layer", 5)?
            .map(|r| r.id)
            != layer.parent
        {
            return Err(Error::Invalid("native layer parent correspondence"));
        }
        if layer_data.insert(layer.id, layer).is_some() {
            return Err(Error::Invalid("duplicate layer projection"));
        }
    }
    if layer_data.len()
        != d.components
            .iter()
            .filter(|c| c.native_component_type == 5)
            .count()
    {
        return Err(Error::Invalid("native layer projection coverage"));
    }
    let layer_graph: BTreeMap<_, _> = layer_data
        .iter()
        .map(|(id, layer)| (*id, layer.parent.into_iter().collect()))
        .collect();
    let ordered_layers = crate::import_graph::order(&layer_graph, &l.remaining_after(held)?)?;
    for id in &ordered_layers {
        let layer = &layer_data[id];
        if (options.mode == ImportMode::Geometry
            || (layer.userdata.is_empty() && default_preservation(&layer.preservation, &[])))
            && layer
                .parent
                .is_none_or(|parent| layer_ids.contains_key(&parent))
        {
            layer_ids.insert(*id, LayerId::new());
        }
    }
    let mut line_patterns_by_index = BTreeMap::new();
    for component in &d.components {
        if component.native_component_type == 4
            && let Some(index) = component.native_index
        {
            line_patterns_by_index.insert(index, LinePatternId(component.id));
        }
    }
    let mut materials_by_index = BTreeMap::new();
    for component in &d.components {
        if component.native_component_type == 3
            && let Some(index) = component.native_index
        {
            materials_by_index.insert(index, MaterialId(component.id));
        }
    }
    let mut layers = Vec::new();
    layers
        .try_reserve_exact(layer_ids.len())
        .map_err(|_| Error::Budget("imported layers"))?;
    for id in layer_order {
        let layer = &layer_data[&id];
        if let Some(host) = layer_ids.get(&id) {
            let mut l_ = Layer::new(&layer.name, layer.color);
            l_.id = *host;
            l_.parent = layer.parent.map(|p| layer_ids[&p]);
            l_.visible = layer.visible;
            l_.locked = layer.locked;
            l_.plot_color = Some(layer.plot_color);
            l_.plot_weight_mm = Some(layer.plot_weight_mm);
            for relation in &layer.relations {
                if relation.role == "line_pattern"
                    && let Some(index) = relation.native_index
                {
                    l_.line_pattern = line_patterns_by_index.get(&index).copied();
                }
                if relation.role == "material"
                    && let Some(index) = relation.native_index
                {
                    l_.material = materials_by_index.get(&index).copied();
                }
            }
            catalog.bind(
                catalog.key(5, id)?,
                HostRef::Layer(*host),
                fingerprint(&l_)?,
            )?;
            layers.push(l_);
            if options.mode == ImportMode::Geometry {
                loss(&mut diagnostics, id, "layer", &layer.preservation, &[]);
            }
        }
    }
    let mut object_indices = BTreeMap::new();
    for (index, o) in d.objects.iter().enumerate() {
        if object_indices.insert(o.id, index).is_some() {
            return Err(Error::Invalid("duplicate native geometry identity"));
        }
    }
    let mut definitions = BTreeMap::new();
    let mut member_owner = BTreeMap::new();
    for def in d.definitions {
        if def.native_component_type != 11 {
            return Err(Error::Invalid("definition component type"));
        }
        let key = catalog.key(11, def.id)?;
        catalog.projection_relations(key, &def.relations, l)?;
        let related = &catalog.records[catalog.slots[&key]].relations;
        if related.len() != def.members.len()
            || !related.iter().zip(&def.members).all(|(r, id)| {
                r.role == "definition_member"
                    && r.target.kind == om9_core::retained::NativeRecordKind::Geometry
                    && r.target.id == *id
            })
        {
            return Err(Error::Invalid("native definition member correspondence"));
        }
        for member in &def.members {
            if !object_indices.contains_key(member)
                || member_owner.insert(*member, def.id).is_some()
            {
                return Err(Error::Invalid("native member ownership"));
            }
        }
        if definitions.insert(def.id, def).is_some() {
            return Err(Error::Invalid("duplicate definition projection"));
        }
    }
    if definitions.len()
        != d.components
            .iter()
            .filter(|c| c.native_component_type == 11)
            .count()
    {
        return Err(Error::Invalid("native definition projection coverage"));
    }
    let mut definition_graph = BTreeMap::new();
    for (id, def) in &definitions {
        let mut children = Vec::new();
        children
            .try_reserve_exact(def.members.len())
            .map_err(|_| Error::Budget("native definition children"))?;
        for member in &def.members {
            if let Some(child) = catalog.one(
                &d.objects[object_indices[member]].relations,
                "instance_definition",
                11,
            )? {
                children.push(child.id);
            }
        }
        definition_graph.insert(*id, children);
    }
    let ordered_defs = crate::import_graph::order(&definition_graph, &l.remaining_after(held)?)?;
    for o in &d.objects {
        if o.is_definition_member != member_owner.contains_key(&o.id) {
            return Err(Error::Invalid("native definition member flag"));
        }
        let layer = catalog.one(&o.relations, "layer", 5)?;
        if layer.map(|r| r.id) != o.attributes.layer
            || layer.map(|r| r.native_index)
                != (o.attributes.native_layer_index >= 0)
                    .then_some(Some(o.attributes.native_layer_index))
        {
            return Err(Error::Invalid("native object layer correspondence"));
        }
        let target = catalog.one(&o.relations, "instance_definition", 11)?;
        if let Some(geometry) = &o.geometry {
            if o.capability != "geometry_projection_exact"
                || o.placement.is_none()
                || o.projection_units_basis.as_deref()
                    != Some(if o.is_definition_member {
                        "resolved_definition_units_to_canonical_mm"
                    } else {
                        "resolved_model_units_to_canonical_mm"
                    })
                || match geometry {
                    Geometry::Instance(id) => target.map(|r| r.id) != Some(id.0),
                    _ => target.is_some(),
                }
            {
                return Err(Error::Invalid("native geometry projection correspondence"));
            }
        } else if o.capability != "retained_only"
            || o.placement.is_some()
            || o.projection_units_basis.is_some()
        {
            return Err(Error::Invalid("native retained geometry classification"));
        }
        o.native_placement.validate()?;
        if (o.attributes.color_source == 1) != o.attributes.color.is_some() {
            return Err(Error::Invalid("native object color correspondence"));
        }
    }
    if d.unit_resolution.definitions.len() != definitions.len() {
        return Err(Error::Invalid("definition unit receipt coverage"));
    }
    for id in definitions.keys() {
        if !d
            .unit_resolution
            .definitions
            .contains_key(&id.hyphenated().to_string())
        {
            return Err(Error::Missing("definition unit receipt"));
        }
    }
    for id in options.units.definition_mm_per_unit.keys() {
        if !definitions.contains_key(id) {
            return Err(Error::Invalid("unknown explicit definition unit"));
        }
    }
    let mut admitted = BTreeSet::new();
    // Children precede parents; every native dependency was validated above.
    for id in &ordered_defs {
        let def = &definitions[id];
        if def.linked
            || !def.admission_required
            || !def.preservation.admission_required
            || source_units.definitions[id].millimeters_per_unit.is_none()
        {
            continue;
        }
        if options.mode == ImportMode::Preserve
            && (!def.userdata.is_empty() || !default_preservation(&def.preservation, &[]))
        {
            continue;
        }
        let ok = def.members.iter().all(|m| {
            let o = &d.objects[object_indices[m]];
            eligible_object(o, options.mode, &layer_ids)
                && match &o.geometry {
                    Some(Geometry::Instance(i)) => admitted.contains(&i.0),
                    _ => true,
                }
        });
        if ok {
            admitted.insert(*id);
        }
    }
    let mut used = BTreeSet::new();
    for o in &d.objects {
        if !o.is_definition_member
            && eligible_object(o, options.mode, &layer_ids)
            && let Some(Geometry::Instance(i)) = &o.geometry
            && admitted.contains(&i.0)
        {
            used.insert(i.0);
        }
    }
    for id in ordered_defs.iter().rev() {
        if used.contains(id) {
            used.extend(&definition_graph[id]);
        }
    }
    let def_ids: BTreeMap<_, _> = used.iter().map(|id| (*id, DefinitionId::new())).collect();
    let mut objects: Vec<Option<dto::Object>> = d.objects.into_iter().map(Some).collect();
    let mut host_defs = Vec::new();
    host_defs
        .try_reserve_exact(used.len())
        .map_err(|_| Error::Budget("import definitions"))?;
    for id in &used {
        check(cancel)?;
        let def = &definitions[id];
        let mut members = Vec::new();
        members
            .try_reserve_exact(def.members.len())
            .map_err(|_| Error::Budget("definition members"))?;
        for member in &def.members {
            let o = objects[object_indices[member]]
                .take()
                .ok_or(Error::Invalid("duplicate bound native member"))?;
            let key = catalog.key(o.native_component_type, o.id)?;
            if options.mode == ImportMode::Geometry {
                loss(
                    &mut diagnostics,
                    o.id,
                    "definition member",
                    &o.preservation,
                    if o.attributes.color_source <= 1 {
                        &["color_source", "native_color"]
                    } else {
                        &[]
                    },
                );
            }
            let e = entity(
                o,
                &layer_ids,
                &line_patterns_by_index,
                &materials_by_index,
                &def_ids,
                bridge,
                options.brep,
                brep_tolerance_mm,
                brep_deflection_mm,
                l,
                &mut brep_counters,
                &mut diagnostics,
            )?;
            catalog.bind_entity(
                key,
                HostRef::DefinitionMember {
                    definition: def_ids[id],
                    object: e.id,
                },
                &e,
            )?;
            members.push(e);
        }
        let mut h = Definition::new(&def.label, members);
        h.id = def_ids[id];
        catalog.bind(
            catalog.key(11, *id)?,
            HostRef::Definition(h.id),
            fingerprint(&h)?,
        )?;
        host_defs.push(h);
        if options.mode == ImportMode::Geometry {
            loss(&mut diagnostics, *id, "definition", &def.preservation, &[]);
        }
    }
    let mut exchange = ExchangeMetadata::default();
    for component in &d.components {
        if component.native_component_type != 4 {
            continue;
        }
        let mut segments = Vec::new();
        if let Some(values) = component
            .settings
            .as_ref()
            .and_then(|settings| settings.get("segments"))
            .and_then(serde_json::Value::as_array)
        {
            segments
                .try_reserve_exact(values.len())
                .map_err(|_| Error::Budget("line pattern segments"))?;
            for value in values {
                let length_mm = value
                    .get("length_mm")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or(Error::Invalid("line pattern segment length"))?;
                let segment_type = value
                    .get("segment_type")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or(Error::Invalid("line pattern segment type"))?;
                segments.push(LinePatternSegment {
                    length_mm,
                    segment_type,
                });
            }
        }
        let id = LinePatternId(component.id);
        if let Some(index) = component.native_index {
            line_patterns_by_index.insert(index, id);
        }
        if exchange
            .line_patterns
            .insert(
                id,
                LinePattern {
                    id,
                    name: component.name.clone(),
                    segments,
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native line pattern id"));
        }
    }
    let mut entities = Vec::new();
    let mut page_space_objects = 0usize;
    entities
        .try_reserve_exact(objects.len())
        .map_err(|_| Error::Budget("import root objects"))?;
    for component in &d.components {
        if component.native_component_type != 7 {
            continue;
        }
        let id = TextStyleId(component.id);
        if exchange
            .text_styles
            .insert(
                id,
                TextStyle {
                    id,
                    name: component.name.clone(),
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native text style id"));
        }
    }
    for component in &d.components {
        if component.native_component_type != 3 {
            continue;
        }
        let diffuse = component
            .settings
            .as_ref()
            .and_then(|settings| settings.get("diffuse"))
            .cloned()
            .ok_or(Error::Invalid("native material diffuse"))?;
        let diffuse: [u8; 4] =
            serde_json::from_value(diffuse).map_err(|_| Error::Invalid("material diffuse"))?;
        let id = MaterialId(component.id);
        if exchange
            .materials
            .insert(
                id,
                RenderMaterial {
                    id,
                    name: component.name.clone(),
                    diffuse,
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native material id"));
        }
    }
    for component in &d.components {
        if component.native_component_type != 8 {
            continue;
        }
        let settings = component
            .settings
            .as_ref()
            .ok_or(Error::Invalid("native dimension style settings"))?;
        let text_height = settings
            .get("text_height")
            .and_then(serde_json::Value::as_f64)
            .ok_or(Error::Invalid("dimension style height"))?;
        let length_resolution = settings
            .get("length_resolution")
            .and_then(serde_json::Value::as_f64)
            .ok_or(Error::Invalid("dimension style resolution"))?;
        let id = DimStyleId(component.id);
        if exchange
            .dim_styles
            .insert(
                id,
                DimStyle {
                    id,
                    name: component.name.clone(),
                    text_height,
                    length_resolution,
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native dimension style id"));
        }
    }
    for component in &d.components {
        if component.native_component_type != 10 {
            continue;
        }
        let fill_type = component
            .settings
            .as_ref()
            .and_then(|settings| settings.get("fill_type"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(Error::Invalid("hatch pattern fill type"))?;
        let mut lines = Vec::new();
        if let Some(values) = component
            .settings
            .as_ref()
            .and_then(|settings| settings.get("lines"))
            .and_then(serde_json::Value::as_array)
        {
            lines
                .try_reserve_exact(values.len())
                .map_err(|_| Error::Budget("hatch pattern lines"))?;
            for value in values {
                let angle_radians = value
                    .get("angle_radians")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or(Error::Invalid("hatch line angle"))?;
                let base = serde_json::from_value::<[f64; 2]>(
                    value
                        .get("base")
                        .cloned()
                        .ok_or(Error::Invalid("hatch line base"))?,
                )
                .map_err(|_| Error::Invalid("hatch line base"))?;
                let offset = serde_json::from_value::<[f64; 2]>(
                    value
                        .get("offset")
                        .cloned()
                        .ok_or(Error::Invalid("hatch line offset"))?,
                )
                .map_err(|_| Error::Invalid("hatch line offset"))?;
                let dashes = serde_json::from_value::<Vec<f64>>(
                    value
                        .get("dashes")
                        .cloned()
                        .ok_or(Error::Invalid("hatch line dashes"))?,
                )
                .map_err(|_| Error::Invalid("hatch line dashes"))?;
                lines.push(HatchPatternLine {
                    angle_radians,
                    base,
                    offset,
                    dashes,
                });
            }
        }
        let id = HatchPatternId(component.id);
        if exchange
            .hatch_patterns
            .insert(
                id,
                HatchPattern {
                    id,
                    name: component.name.clone(),
                    fill_type,
                    lines,
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native hatch pattern id"));
        }
    }
    for component in &d.components {
        if component.native_component_type != 9 {
            continue;
        }
        let settings = component
            .settings
            .as_ref()
            .ok_or(Error::Invalid("native light settings"))?;
        let style = settings
            .get("style")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(Error::Invalid("light style"))?;
        let location = serde_json::from_value::<[f64; 3]>(
            settings
                .get("location")
                .cloned()
                .ok_or(Error::Invalid("light location"))?,
        )
        .map_err(|_| Error::Invalid("light location"))?;
        let direction = serde_json::from_value::<[f64; 3]>(
            settings
                .get("direction")
                .cloned()
                .ok_or(Error::Invalid("light direction"))?,
        )
        .map_err(|_| Error::Invalid("light direction"))?;
        let diffuse = serde_json::from_value::<[u8; 4]>(
            settings
                .get("diffuse")
                .cloned()
                .ok_or(Error::Invalid("light diffuse"))?,
        )
        .map_err(|_| Error::Invalid("light diffuse"))?;
        let intensity = settings
            .get("intensity")
            .and_then(serde_json::Value::as_f64)
            .ok_or(Error::Invalid("light intensity"))?;
        let id = LightId(component.id);
        if exchange
            .lights
            .insert(
                id,
                RenderLight {
                    id,
                    name: component.name.clone(),
                    style,
                    location,
                    direction,
                    diffuse,
                    intensity,
                },
            )
            .is_some()
        {
            return Err(Error::Invalid("duplicate native light id"));
        }
    }
    let mut groups = Vec::new();
    let mut group_positions = BTreeMap::new();
    for component in &d.components {
        if component.native_component_type != 6 {
            continue;
        }
        let native_index = component
            .native_index
            .ok_or(Error::Invalid("native group index"))?;
        if group_positions.insert(native_index, groups.len()).is_some() {
            return Err(Error::Invalid("duplicate native group index"));
        }
        groups.push(Group {
            id: GroupId(component.id),
            name: component.name.clone(),
            members: Vec::new(),
            revision: 1,
        });
    }
    for o in objects.into_iter().flatten() {
        check(cancel)?;
        if o.is_definition_member
            || !eligible_object(&o, options.mode, &layer_ids)
            || matches!(&o.geometry,Some(Geometry::Instance(i)) if !def_ids.contains_key(&i.0))
        {
            continue;
        }
        let native_group_indices = o.attributes.groups.clone();
        if options.mode == ImportMode::Geometry {
            loss(
                &mut diagnostics,
                o.id,
                "object",
                &o.preservation,
                if o.attributes.color_source <= 1 {
                    &["color_source", "native_color"]
                } else {
                    &[]
                },
            );
        }
        let page_space = o.attributes.space == 2;
        let key = catalog.key(o.native_component_type, o.id)?;
        let e = entity(
            o,
            &layer_ids,
            &line_patterns_by_index,
            &materials_by_index,
            &def_ids,
            bridge,
            options.brep,
            brep_tolerance_mm,
            brep_deflection_mm,
            l,
            &mut brep_counters,
            &mut diagnostics,
        )?;
        catalog.bind_entity(key, HostRef::Object(e.id), &e)?;
        if page_space {
            page_space_objects = page_space_objects
                .checked_add(1)
                .ok_or(Error::Overflow("page-space object count"))?;
        }
        for index in native_group_indices {
            if let Some(position) = group_positions.get(&index) {
                groups[*position].members.push(e.id);
            }
        }
        entities.push(e);
    }
    let retained_records = catalog.records.iter().filter(|r| r.host.is_none()).count();
    if retained_records > 0 {
        diagnostics.push(format!("{retained_records} native source records remain retained-only; no fabricated host geometry is created."));
    }
    let resource = Resource::from_bytes(
        &snapshot.source_name,
        ResourceKind::RetainedThreeDm {
            archive_version: d.archive.version,
        },
        &snapshot.bytes,
        &l.remaining_after(held)?,
    )?;
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: d.archive.version,
        units: source_units,
    };
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: Some(match options.mode {
            ImportMode::Preserve => om9_core::retained::ImportMode::Preserve,
            ImportMode::Geometry => om9_core::retained::ImportMode::Geometry,
        }),
        records: catalog.records,
    };
    let receipt = ImportReceipt {
        source: source.id,
        resource: resource.id,
        namespace: namespace.id,
        source_sha256: source.sha256,
        root_objects: entities.len(),
        definition_members: host_defs.iter().map(|d| d.members.len()).sum(),
        definitions: host_defs.len(),
        layers: layers.len(),
        groups: groups.len(),
        line_patterns: exchange.line_patterns.len(),
        page_space_objects,
        text_styles: exchange.text_styles.len(),
        materials: exchange.materials.len(),
        dim_styles: exchange.dim_styles.len(),
        hatch_patterns: exchange.hatch_patterns.len(),
        breps_editable: brep_counters.editable,
        breps_opaque: brep_counters.opaque,
        breps_displayable: brep_counters.displayable,
        breps_display_unavailable: brep_counters.display_unavailable,
        annotations: annotation_count,
        hatches: hatch_count,
        lights: light_count + exchange.lights.len(),
        retained_records,
        diagnostics,
        native: ProcessStats {
            pid: 0,
            peak_committed_bytes: 0,
            active_processes: 0,
            exited: false,
            elapsed: std::time::Duration::ZERO,
        },
    };
    Ok((
        ImportBundle {
            resource,
            source,
            namespace,
            entities,
            layers,
            definitions: host_defs,
            groups,
            exchange,
        },
        receipt,
    ))
}
