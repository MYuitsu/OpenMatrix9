// SPDX-License-Identifier: LGPL-2.1-or-later
use std::{fs, process::Command};
fn command() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_om9"));
    c.env_clear();
    for key in ["SystemRoot", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(key) {
            c.env(key, value);
        }
    }
    c
}
fn cold_output(option: &str, path: &std::path::Path) -> (u32, std::process::Output) {
    use std::process::Stdio;
    let child = command()
        .arg(option)
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    (pid, child.wait_with_output().unwrap())
}
fn receipt(case: &str, details: serde_json::Value) {
    println!(
        "OM9_RESOURCE_RECEIPT {}",
        serde_json::json!({
            "case": case, "binary": env!("CARGO_BIN_EXE_om9"), "details": details
        })
    );
}
#[test]
fn actual_standalone_process_creates_and_cold_reopens_owned_om9_document() {
    let folder = std::env::temp_dir().join(format!("om9-cold-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let path = folder.join("fixture.om9");
    let created = command()
        .args(["--create-fixture"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    assert!(
        path.is_file(),
        "standalone executable must write its own document"
    );
    let before: serde_json::Value = serde_json::from_slice(&created.stdout).unwrap();
    let reopened = command().args(["--inspect"]).arg(&path).output().unwrap();
    assert!(
        reopened.status.success(),
        "{}",
        String::from_utf8_lossy(&reopened.stderr)
    );
    let after: serde_json::Value = serde_json::from_slice(&reopened.stdout).unwrap();
    assert_eq!(after, before);
    assert_eq!(after["target"], "OM9Standalone");
    assert_eq!(after["objects"].as_array().unwrap().len(), 4);
    fs::remove_file(&path).unwrap();
    fs::remove_dir(&folder).unwrap();
}
#[test]
fn corrupt_archive_causes_real_process_failure_without_success_json() {
    let path = std::env::temp_dir().join(format!("om9-corrupt-{}.om9", uuid::Uuid::new_v4()));
    fs::write(&path, b"corrupt archive").unwrap();
    let output = command().arg("--inspect").arg(&path).output().unwrap();
    fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn fresh_process_hydrates_embedded_resource_after_original_file_is_removed() {
    use om9_archive::Archive;
    use om9_core::{
        budget::Limits,
        model::Document,
        resource::{Resource, ResourceKind},
        transaction::Cancellation,
    };
    use sha2::{Digest, Sha256};
    let folder = std::env::temp_dir().join(format!("om9-embedded-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let original = folder.join("source-resource.dat");
    let bytes: Vec<u8> = (0..=255).cycle().take(32 * 1024).collect();
    fs::write(&original, &bytes).unwrap();
    let resource = Resource::from_bytes(
        "authored opaque bytes",
        ResourceKind::OpaqueBinary,
        &fs::read(&original).unwrap(),
        &Limits::default(),
    )
    .unwrap();
    let id = resource.id;
    let mut d = Document::new("embedded cold restore");
    d.resources.insert(id, resource);
    let path = folder.join("resource.om9");
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    fs::remove_file(original).unwrap();
    let (pid, output) = cold_output("--inspect", &path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["restoration"], "ready");
    assert_eq!(report["resources"][0]["id"], id.to_string());
    assert_eq!(report["resources"][0]["bytes"], bytes.len());
    assert_eq!(
        report["resources"][0]["payload_sha256"],
        format!("{:x}", Sha256::digest(&bytes))
    );
    receipt(
        "embedded_resource_cold_reopen",
        serde_json::json!({
            "pid":pid, "exited":output.status.success(), "document_id":d.id.to_string(),
            "resource":report["resources"][0], "original_removed":true,
            "archive_sha256":format!("{:x}",Sha256::digest(fs::read(&path).unwrap()))
        }),
    );
    fs::remove_file(path).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
fn actual_process_requires_explicit_v1_upgrade_and_preserves_durable_identity() {
    use om9_core::{
        geometry::Geometry,
        model::{Document, Entity},
    };
    use sha2::{Digest, Sha256};
    let mut d = Document::new("legacy migration fixture");
    let entity = Entity::new("old point", Geometry::Point([1., 2., 3.]), None);
    let object_id = entity.id;
    d.entities.insert(entity.id, entity);
    use om9_core::{
        geometry::{Transform, Unit},
        history::{Recipe, ReplayEnvironment, Tolerance},
    };
    let recipe = Recipe::new(
        d.id,
        "migration replay fixture",
        vec![],
        vec![object_id],
        ReplayEnvironment {
            command_version: 1,
            solver: "absent-module".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 0.,
                angular_rad: 0.001,
            },
        },
    );
    d.history.recipes.insert(recipe.id, recipe);
    let json = serde_json::to_vec(&d).unwrap();
    let mut bytes = vec![0u8; 88];
    bytes[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[40..48].copy_from_slice(&88u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    bytes[56..88].copy_from_slice(&Sha256::digest(&json));
    bytes.extend_from_slice(&json);
    let path = std::env::temp_dir().join(format!("om9-migrate-{}.om9", uuid::Uuid::new_v4()));
    fs::write(&path, &bytes).unwrap();
    let (rejected_pid, rejected) = cold_output("--inspect", &path);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let (upgraded_pid, upgraded) = cold_output("--migrate-v1", &path);
    assert!(
        upgraded.status.success(),
        "{}",
        String::from_utf8_lossy(&upgraded.stderr)
    );
    let migration_report: serde_json::Value = serde_json::from_slice(&upgraded.stdout).unwrap();
    assert_eq!(
        migration_report["restoration"], "unresolved",
        "migration cannot declare missing replay ready"
    );
    let after = fs::read(&path).unwrap();
    assert_eq!(u32::from_le_bytes(after[8..12].try_into().unwrap()), 4);
    let (loaded_pid, loaded) = cold_output("--inspect", &path);
    assert!(loaded.status.success());
    let report: serde_json::Value = serde_json::from_slice(&loaded.stdout).unwrap();
    assert_eq!(report["document_id"], d.id.to_string());
    assert_eq!(report["objects"][0]["id"], object_id.to_string());
    assert_eq!(
        report["objects"][0]["geometry"]["Point"],
        serde_json::json!([1., 2., 3.])
    );
    assert!(report["resources"].as_array().unwrap().is_empty());
    receipt(
        "explicit_schema1_migration",
        serde_json::json!({
            "pids":[rejected_pid,upgraded_pid,loaded_pid], "exited":true,
            "source_sha256":format!("{:x}",Sha256::digest(&bytes)),
            "destination_sha256":format!("{:x}",Sha256::digest(&after)),
            "from_schema":1, "to_schema":4, "document_id":report["document_id"],
            "object_id":report["objects"][0]["id"], "geometry":report["objects"][0]["geometry"],
            "restoration":migration_report["restoration"], "diagnostics":migration_report["diagnostics"],
            "default_open_rejected":true, "resources_empty":true
        }),
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn runtime_migration_overflow_rejects_before_replacing_original_schema1_file() {
    use om9_core::model::Document;
    use sha2::{Digest, Sha256};
    let mut d = Document::new("counter overflow");
    d.session_generation = u64::MAX;
    let json = serde_json::to_vec(&d).unwrap();
    let mut bytes = vec![0u8; 88];
    bytes[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[40..48].copy_from_slice(&88u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&(json.len() as u64).to_le_bytes());
    bytes[56..88].copy_from_slice(&Sha256::digest(&json));
    bytes.extend_from_slice(&json);
    let folder =
        std::env::temp_dir().join(format!("om9-failed-migration-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let path = folder.join("old.om9");
    fs::write(&path, &bytes).unwrap();
    let (pid, failed) = cold_output("--migrate-v1", &path);
    assert!(!failed.status.success());
    assert!(failed.stdout.is_empty());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
    receipt(
        "migration_counter_overflow_preserves_file",
        serde_json::json!({
            "pid":pid, "exited":true, "exit_success":failed.status.success(),
            "stdout_empty":failed.stdout.is_empty(), "original_preserved":true,
            "directory_entries":1, "source_sha256":format!("{:x}",Sha256::digest(&bytes))
        }),
    );
    fs::remove_file(path).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
fn persistent_report_hash_keeps_recipe_parameters_named_like_session_fields() {
    use om9_archive::Archive;
    use om9_core::{
        budget::Limits,
        geometry::{Geometry, Transform, Unit},
        history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
        model::{Document, Entity},
        request::Value,
        transaction::Cancellation,
    };
    let mut d = Document::new("report fingerprint");
    let e = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let mut recipe = Recipe::new(
        d.id,
        "fixture",
        vec![],
        vec![id],
        ReplayEnvironment {
            command_version: 1,
            solver: "fixture".into(),
            solver_version: "1".into(),
            schema_version: 1,
            length_unit: Unit::Millimeter,
            frame: Transform::identity(),
            tolerance: Tolerance {
                absolute_mm: 0.001,
                relative: 0.,
                angular_rad: 0.001,
            },
        },
    );
    recipe.state = EvaluationState::Detached;
    recipe.parameters.insert("epoch".into(), Value::Number(11.));
    let recipe_id = recipe.id;
    d.history.recipes.insert(recipe_id, recipe);
    let folder = std::env::temp_dir().join(format!("om9-fingerprint-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&folder).unwrap();
    let mut hashes = vec![];
    let mut pids = vec![];
    for number in [11., 42.] {
        d.history
            .recipes
            .get_mut(&recipe_id)
            .unwrap()
            .parameters
            .insert("epoch".into(), Value::Number(number));
        let path = folder.join("current.om9");
        Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
        let (pid, output) = cold_output("--inspect", &path);
        pids.push(pid);
        assert!(output.status.success());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        hashes.push(report["document_sha256"].clone());
    }
    assert_ne!(
        hashes[0], hashes[1],
        "logical option data cannot be erased by runtime-owner normalization"
    );
    receipt(
        "persistent_parameter_fingerprint",
        serde_json::json!({
            "pids":pids, "exited":true, "parameter":"epoch", "values":[11,42],
            "document_id":d.id.to_string(), "recipe_id":recipe_id.to_string(),
            "persistent_sha256":hashes
        }),
    );
    fs::remove_file(folder.join("current.om9")).unwrap();
    fs::remove_dir(folder).unwrap();
}

#[test]
fn audit_process_reports_loaded_document_and_waits_for_its_owner_to_release() {
    use std::io::{BufRead, Write};
    use std::process::Stdio;
    let path = std::env::temp_dir().join(format!("om9-audit-{}.om9", uuid::Uuid::new_v4()));
    let created = command()
        .arg("--create-fixture")
        .arg(&path)
        .output()
        .unwrap();
    assert!(created.status.success());
    let mut child = command()
        .arg("--audit")
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(!line.is_empty());
    let report: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(report["target"], "OM9Standalone");
    assert!(child.try_wait().unwrap().is_none());
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"release\n")
        .unwrap();
    assert!(child.wait().unwrap().success());
    fs::remove_file(path).unwrap();
}

#[test]
fn actual_schema2_upgrade_preserves_embedded_bytes_and_does_not_invent_native_sources() {
    use om9_archive::Archive;
    use om9_core::{
        budget::Limits,
        model::Document,
        resource::{Resource, ResourceKind},
        transaction::Cancellation,
    };
    use sha2::{Digest, Sha256};
    let mut doc = Document::new("legacy embedded bytes");
    let resource = Resource::from_bytes(
        "original.3dm",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored legacy resource; resource kind was never native source verification",
        &Limits::default(),
    )
    .unwrap();
    let resource_id = resource.id;
    let payload_sha = resource.sha256();
    doc.resources.insert(resource.id, resource);
    let mut bytes = Archive::encode(&doc, &Limits::default()).unwrap();
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    let path = std::env::temp_dir().join(format!("om9-migrate2-{}.om9", uuid::Uuid::new_v4()));
    fs::write(&path, &bytes).unwrap();
    let (rejected_pid, rejected) = cold_output("--inspect", &path);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let (pid, upgraded) = cold_output("--migrate-v2", &path);
    assert!(
        upgraded.status.success(),
        "{}",
        String::from_utf8_lossy(&upgraded.stderr)
    );
    let (cold_pid, reopened) = cold_output("--inspect", &path);
    assert!(reopened.status.success());
    let report: serde_json::Value = serde_json::from_slice(&reopened.stdout).unwrap();
    assert_eq!(report["document_id"], doc.id.to_string());
    assert_eq!(report["resources"][0]["id"], resource_id.to_string());
    assert_eq!(
        report["resources"][0]["payload_sha256"],
        payload_sha
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert!(report["retained_sources"].as_object().unwrap().is_empty());
    assert!(report["import_namespaces"].as_object().unwrap().is_empty());
    let after = fs::read(&path).unwrap();
    assert_eq!(u32::from_le_bytes(after[8..12].try_into().unwrap()), 4);
    Archive::save(&path, &doc, &Limits::default(), &Cancellation::default()).unwrap();
    receipt(
        "explicit_schema2_migration",
        serde_json::json!({"pids":[rejected_pid,pid,cold_pid],"exited":true,
        "source_sha256":format!("{:x}",Sha256::digest(&bytes)),"destination_sha256":format!("{:x}",Sha256::digest(&after)),
        "from_schema":2,"to_schema":4,"document_id":report["document_id"],"resource":report["resources"][0],
        "empty_provenance":true,"default_open_rejected":true}),
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn schema4_cold_process_preserves_namespaces_current_edits_tombstones_and_original_unit_context() {
    use om9_archive::Archive;
    use om9_core::{
        budget::Limits,
        geometry::Geometry,
        identity::*,
        model::{Document, Entity},
        resource::{Resource, ResourceKind},
        retained::{
            HostRef, ImportBundle, ImportNamespace, NativeRecordKey, NativeRecordKind, NativeUnits,
            ResolvedBy, RetainedSource, SourceRecord, SourceUnits, fingerprint,
        },
        transaction::*,
    };
    let resource = Resource::from_bytes(
        "embedded original",
        ResourceKind::RetainedThreeDm {
            archive_version: 50,
        },
        b"authored source ledger fixture; native correspondence is validated separately",
        &Limits::default(),
    )
    .unwrap();
    let u = NativeUnits {
        native_enum: 8,
        name: "Inches".into(),
        declared_mm_per_unit: Some(25.4),
        millimeters_per_unit: 25.4,
        resolved_by: ResolvedBy::Declared,
        absolute_tolerance: 0.001,
        relative_tolerance: 0.,
        angular_tolerance_radians: 0.001,
    };
    let source = RetainedSource {
        id: SourceId::new(),
        resource: resource.id,
        sha256: resource.sha256(),
        archive_version: 50,
        units: SourceUnits {
            definitions: Default::default(),
            model: u.clone(),
            page: u,
        },
    };
    let a = Entity::new("A", Geometry::Point([25.4, 0., 0.]), None);
    let b = Entity::new("B", Geometry::Point([0., 25.4, 0.]), None);
    let a_id = a.id;
    let b_id = b.id;
    let records = [&a, &b]
        .iter()
        .enumerate()
        .map(|(index, e)| {
            SourceRecord::bound(
                NativeRecordKey {
                    kind: NativeRecordKind::Geometry,
                    id: uuid::Uuid::new_v4(),
                    native_index: Some(index as i32),
                },
                "ON_Point",
                HostRef::Object(e.id),
                fingerprint(e).unwrap(),
            )
        })
        .collect();
    let namespace = ImportNamespace {
        id: ImportNamespaceId::new(),
        source: source.id,
        import_mode: None,
        records,
    };
    let ns = namespace.id;
    let source_id = source.id;
    let bundle = ImportBundle {
        resource,
        source,
        namespace,
        entities: vec![a, b],
        layers: vec![],
        definitions: vec![],
        groups: vec![],
        exchange: Default::default(),
    };
    let doc = Document::new("owned cold source ledger");
    let did = doc.id;
    let mut coordinator = TransactionCoordinator::new(
        World::new(vec![doc], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let prepared = coordinator
        .prepare(TransactionRequest::new(
            "Import",
            vec![Mutation::Import {
                document: did,
                expected_revision: 1,
                bundle: Box::new(bundle),
            }],
        ))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    let before = &coordinator.world().documents[&did];
    let new = Entity::new("C", Geometry::Point([3., 4., 5.]), None);
    let c_id = new.id;
    let prepared = coordinator
        .prepare(TransactionRequest::new(
            "A edit/B delete/C new",
            vec![
                Mutation::Geometry {
                    source: before.stamp(a_id).unwrap(),
                    geometry: Geometry::Point([50.8, 0., 0.]),
                },
                Mutation::Delete {
                    source: before.stamp(b_id).unwrap(),
                },
                Mutation::Insert {
                    document: did,
                    entity: new,
                },
            ],
        ))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    let path = std::env::temp_dir().join(format!("om9-ledger-{}.om9", uuid::Uuid::new_v4()));
    Archive::save(
        &path,
        &coordinator.world().documents[&did],
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    drop(coordinator);
    let (pid, output) = cold_output("--inspect", &path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["import_namespaces"][ns.to_string()]["records"][0]["disposition"],
        "Edited"
    );
    assert_eq!(
        report["import_namespaces"][ns.to_string()]["records"][1]["disposition"],
        "Deleted"
    );
    assert_eq!(
        report["retained_sources"][source_id.to_string()]["units"]["model"]["millimeters_per_unit"],
        25.4
    );
    let objects = report["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 2);
    assert!(objects.iter().all(|e| e["id"] != b_id.to_string()));
    assert!(objects.iter().any(|e| e["id"] == a_id.to_string()
        && e["geometry"] == serde_json::json!({"Point":[50.8,0.,0.]})));
    assert!(objects.iter().any(|e| e["id"] == c_id.to_string()));
    receipt(
        "schema3_namespace_tombstone_cold_reopen",
        serde_json::json!({"pid":pid,"exited":true,
        "document_id":report["document_id"],"source_id":source_id,"namespace_id":ns,
        "dispositions":["Edited","Deleted"],"objects":objects,"original_model_mm_per_unit":25.4,
        "resource":report["resources"][0]}),
    );
    fs::remove_file(path).unwrap();
}
