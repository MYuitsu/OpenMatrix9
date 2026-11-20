// SPDX-License-Identifier: LGPL-2.1-or-later
//! Versioned user density catalog. This does not contain Gemvision materials.
use om9_core::{Error, Result, metal_weight::{MetalMaterial, MetalMaterialRegistry}};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

const SCHEMA: u32 = 1;
const MAX_BYTES: u64 = 1024 * 1024;
const MAX_MATERIALS: usize = 4096;
const MAX_BINDINGS: usize = 8192;
const MAX_FIELD_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserMetalCatalog {
    pub schema_version: u32,
    pub materials: Vec<MetalMaterial>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<MetalDensityBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetalDensityBinding {
    pub document: om9_core::identity::DocumentId,
    pub render_material: om9_core::identity::MaterialId,
    pub density_id: String,
}
impl Default for UserMetalCatalog {
    fn default() -> Self { Self { schema_version: SCHEMA, materials: Vec::new(), bindings: Vec::new() } }
}
impl UserMetalCatalog {
    /// Validate the complete candidate before it replaces the active catalog.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA { return Err(Error::Invalid("metal catalog schema")); }
        if self.materials.len() > MAX_MATERIALS { return Err(Error::Budget("metal catalog materials")); }
        if self.bindings.len() > MAX_BINDINGS { return Err(Error::Budget("metal catalog bindings")); }
        let references = MetalMaterialRegistry::builtin();
        let mut seen = BTreeSet::new();
        let mut validated = MetalMaterialRegistry::empty();
        for material in &self.materials {
            if material.test_only { return Err(Error::Invalid("test-only metal catalog material")); }
            if [&material.id, &material.name, &material.provenance].iter().any(|v| v.len() > MAX_FIELD_BYTES) {
                return Err(Error::Budget("metal catalog field"));
            }
            let id = material.id.trim().to_ascii_lowercase();
            if references.contains(&id) || !seen.insert(id) { return Err(Error::Invalid("duplicate metal catalog id")); }
            validated.insert(material.clone())?;
        }
        let mut bindings = BTreeSet::new();
        for binding in &self.bindings {
            if binding.document.is_nil() || binding.render_material.is_nil() {
                return Err(Error::Invalid("metal binding identity"));
            }
            if binding.density_id.len() > MAX_FIELD_BYTES { return Err(Error::Budget("metal catalog field")); }
            if !references.contains(&binding.density_id) && !validated.contains(&binding.density_id) {
                return Err(Error::Missing("metal material density binding"));
            }
            if !bindings.insert((binding.document, binding.render_material)) {
                return Err(Error::Invalid("duplicate metal material binding"));
            }
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<UserMetalCatalog> {
    let metadata = std::fs::metadata(path).map_err(|e| Error::Io(e.to_string()))?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES { return Err(Error::Invalid("metal catalog file")); }
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|e| Error::Io(e.to_string()))?;
    if bytes.len() as u64 > MAX_BYTES { return Err(Error::Budget("metal catalog file")); }
    let catalog: UserMetalCatalog = serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("metal catalog json"))?;
    catalog.validate()?;
    Ok(catalog)
}

pub fn save(path: &Path, catalog: &UserMetalCatalog) -> Result<()> {
    catalog.validate()?;
    let bytes = serde_json::to_vec_pretty(catalog).map_err(|_| Error::Invalid("metal catalog json"))?;
    if bytes.len() as u64 > MAX_BYTES { return Err(Error::Budget("metal catalog file")); }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    let name = path.file_name().ok_or(Error::Invalid("metal catalog filename"))?.to_string_lossy();
    let staging = parent.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&staging)
            .map_err(|e| Error::Io(e.to_string()))?;
        file.write_all(&bytes).and_then(|_| file.sync_all()).map_err(|e| Error::Io(e.to_string()))?;
        drop(file);
        std::fs::rename(&staging, path).map_err(|e| Error::Io(e.to_string()))?;
        Ok(())
    })();
    if result.is_err() { let _ = std::fs::remove_file(staging); }
    result
}
