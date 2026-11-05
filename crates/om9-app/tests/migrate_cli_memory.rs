// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::Archive;
use om9_core::{budget::Limits, geometry::{Cloud, Geometry}, model::{Document, Entity}};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

struct TempFile(PathBuf);
impl Drop for TempFile { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }

#[test]
fn legacy_cli_respects_ram_override_and_preserves_source_before_retry() {
    let limits = Limits::default();
    let mut document = Document::new("legacy memory fixture");
    let id = document.id;
    let points = (0..100_000).map(|i| [i as f64, i as f64, i as f64]).collect();
    let entity = Entity::new("cloud", Geometry::Cloud(Cloud { points, normals: vec![], colors: vec![], hidden: vec![] }), None);
    let entity_id = entity.id;
    document.entities.insert(entity_id, entity);
    let json = serde_json::to_vec(&document).unwrap();
    let mut archive = vec![0; 88];
    archive[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    archive[8..12].copy_from_slice(&3u32.to_le_bytes());
    archive[12..16].copy_from_slice(&1u32.to_le_bytes());
    archive[16..20].copy_from_slice(&1u32.to_le_bytes());
    archive[40..48].copy_from_slice(&88u64.to_le_bytes());
    archive[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    archive[56..88].copy_from_slice(&Sha256::digest(&json));
    archive.extend(json);
    let path = TempFile(std::env::temp_dir().join(format!("om9-migrate-memory-{}.om9", uuid::Uuid::new_v4())));
    fs::write(&path.0, &archive).unwrap();
    let run = |max: &str| Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--migrate-v3").arg(&path.0).env("OM9_MAX_BYTES", max).output().unwrap();
    assert!(!run("1024").status.success());
    assert_eq!(fs::read(&path.0).unwrap(), archive);
    let result = run("536870912");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let migrated = fs::read(&path.0).unwrap();
    assert_eq!(u32::from_le_bytes(migrated[8..12].try_into().unwrap()), 4);
    let restored = Archive::decode(&migrated, &limits).unwrap();
    assert_eq!(restored.id, id);
    let Geometry::Cloud(cloud) = &restored.entities[&entity_id].geometry else { panic!("expected cloud") };
    assert_eq!(cloud.points.len(), 100_000);
    assert_eq!(cloud.points[99_999], [99_999.0; 3]);
}
