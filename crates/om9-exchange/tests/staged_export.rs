// SPDX-License-Identifier: LGPL-2.1-or-later
//! These tests run the real pinned native conversion executable. They remain
//! explicitly ignored in portable Rust-only runs; validation invokes --ignored.
#![cfg(windows)]
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform, Unit},
    model::{Document, Entity},
    transaction::*,
};
use om9_exchange::{ExportCapture, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn record(label: &str, receipt: &om9_exchange::ExportReceipt) {
    let stats = |s: &om9_native_owner::ProcessStats| serde_json::json!({"pid":s.pid,"peak_committed_bytes":s.peak_committed_bytes,"active_processes":s.active_processes,"exited":s.exited,"elapsed_ms":s.elapsed.as_millis()});
    println!(
        "OM9_EXCHANGE_RECEIPT {}",
        serde_json::json!({"case":label,"sha256":receipt.sha256.iter().map(|b|format!("{b:02x}")).collect::<String>(),"bytes":receipt.bytes,"root_objects":receipt.root_objects,"definition_members":receipt.definition_members,"dropped_history_recipes":receipt.dropped_history_recipes,"dropped_results":receipt.dropped_results,"dropped_om9_hidden_distinctions":receipt.dropped_om9_hidden_distinctions,"dropped_om9_preset_slots":receipt.dropped_om9_preset_slots,"diagnostics":receipt.diagnostics,"write":stats(&receipt.write),"inspect":stats(&receipt.inspect)})
    );
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("om9-export-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn target(&self) -> PathBuf {
        self.0.join("Nhẫn hiện tại.3dm")
    }
    fn assert_clean(&self) {
        assert_eq!(fs::read_dir(&self.0).unwrap().count(), 1);
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.target());
        let _ = fs::remove_dir(&self.0);
    }
}
fn native() -> NativeExchange {
    let path = PathBuf::from(
        std::env::var_os("OM9_EXCHANGE_NATIVE")
            .expect("set OM9_EXCHANGE_NATIVE to the built pinned adapter"),
    );
    let hash: [u8; 32] = Sha256::digest(fs::read(&path).unwrap()).into();
    NativeExchange::new(
        NativeProgram::admit(&path, hash).unwrap(),
        ProcessLimits::default(),
    )
    .unwrap()
}
fn source() -> (
    TransactionCoordinator,
    om9_core::identity::DocumentId,
    om9_core::identity::ObjectId,
) {
    let mut d = Document::new("Nhẫn");
    d.page_unit = Unit::Inch;
    let mut e = Entity::new("Điểm A", Geometry::Point([1., 2., 3.]), None);
    e.placement = Transform::translation([5., 0., 0.]);
    let oid = e.id;
    d.entities.insert(oid, e);
    let did = d.id;
    let limits = Limits::default();
    (
        TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap(),
        did,
        oid,
    )
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn native_packet_stream_rejects_malformed_duplicate_trailing_and_oversized_input() {
    let folder = Directory::new();
    let packet = folder.0.join("packet.json");
    let output = folder.0.join("private.3dm");
    let executable = std::env::var_os("OM9_EXCHANGE_NATIVE").expect("native executable");
    for bytes in [b"".as_slice(), b"{\"x\":1,\"x\":2}", b"{} {}", b"{\"x\":", b"{\"x\":\"\xff\"}"] {
        fs::write(&packet, bytes).unwrap();
        let result = std::process::Command::new(&executable).arg("--write").arg(&packet).arg(&output).output().unwrap();
        assert!(!result.status.success(), "invalid packet unexpectedly accepted");
        assert!(!output.exists(), "invalid packet created output");
    }
    let file = fs::OpenOptions::new().write(true).truncate(true).open(&packet).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let result = std::process::Command::new(&executable).arg("--write").arg(&packet).arg(&output).output().unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    fs::remove_file(packet).unwrap();
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn native_packet_stream_rejects_file_with_existing_writer() {
    use std::os::windows::fs::OpenOptionsExt;
    let folder = Directory::new();
    let packet = folder.0.join("packet.json");
    let output = folder.0.join("private.3dm");
    fs::write(&packet, b"{}").unwrap();
    // Existing writer permits read/write sharing; the adapter must still
    // refuse because its own two-pass read requires writers to be absent.
    let writer = fs::OpenOptions::new().read(true).write(true).share_mode(3).open(&packet).unwrap();
    let executable = std::env::var_os("OM9_EXCHANGE_NATIVE").expect("native executable");
    let result = std::process::Command::new(executable).arg("--write").arg(&packet).arg(&output).output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("open held native archive"));
    assert!(!output.exists());
    drop(writer);
    fs::remove_file(packet).unwrap();
}

#[test]
#[ignore = "requires two pinned native adapter builds for isolated comparison"]
fn native_packet_stream_compares_write_peak_on_same_cloud_fixture() {
    let old = PathBuf::from(std::env::var_os("OM9_NATIVE_BASELINE").expect("baseline adapter"));
    let new = PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").expect("updated adapter"));
    let adapter = |path: &PathBuf| {
        let hash: [u8; 32] = Sha256::digest(fs::read(path).unwrap()).into();
        NativeExchange::new(NativeProgram::admit(path, hash).unwrap(), ProcessLimits::default()).unwrap()
    };
    let adapters = [("baseline", adapter(&old)), ("streamed", adapter(&new))];
    let limits = Limits::default();
    let cloud_count: usize = std::env::var("OM9_BENCH_CLOUD_COUNT").ok().map(|s| s.parse().unwrap()).unwrap_or(4);
    assert!((1..=8).contains(&cloud_count));
    let definitions = std::env::var_os("OM9_BENCH_DEFINITIONS").is_some();
    let document = profile_cloud_document(cloud_count, definitions);
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let packet_bytes = ExportCapture::all(&coordinator, id).unwrap().encode(&limits).unwrap().len();
    println!("packet_bytes={packet_bytes}, cloud_count={cloud_count}, points_per_cloud=32000, definitions={definitions}");
    let folder = Directory::new();
    for repetition in 0..3 {
        for (label, exchange) in &adapters {
            let capture = ExportCapture::all(&coordinator, id).unwrap();
            let receipt = exchange.prepare(&coordinator, capture, &folder.target(), &Cancellation::default()).unwrap()
                .publish(&coordinator, &Cancellation::default()).unwrap();
            assert_eq!(receipt.root_objects, cloud_count);
            println!("comparison={label}, repetition={repetition}, write_peak_bytes={}, write_ms={}, output_bytes={}",
                receipt.write.peak_committed_bytes, receipt.write.elapsed.as_millis(), receipt.bytes);
        }
    }
    folder.assert_clean();
}

fn profile_cloud_document(cloud_count: usize, definitions: bool) -> Document {
    use om9_core::{geometry::Cloud, model::Definition};
    let mut document = Document::with_template_tolerance("native packet benchmark");
    for cloud_index in 0..cloud_count {
        let points = (0..32_000).map(|i| [i as f64 * 0.012345, cloud_index as f64, (i % 17) as f64 * 0.125]).collect();
        let cloud = Entity::new("cloud", Geometry::Cloud(Cloud { points, normals: vec![], colors: vec![], hidden: vec![] }), None);
        let entity = if definitions {
            let definition = Definition::new(format!("cloud block {cloud_index}"), vec![cloud]);
            let id = definition.id;
            document.definitions.insert(id, definition);
            Entity::new("instance", Geometry::Instance(id), None)
        } else { cloud };
        document.entities.insert(entity.id, entity);
    }
    document
}

#[test]
#[ignore = "requires native adapter built with -ProfileMemory"]
fn native_memory_profile_reports_packet_and_model_phases() {
    struct Cleanup(PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }
    let path = PathBuf::from(std::env::var_os("OM9_NATIVE_PROFILE").expect("profile adapter"));
    let hash: [u8; 32] = Sha256::digest(fs::read(&path).unwrap()).into();
    let program = NativeProgram::admit(&path, hash).unwrap();
    let limits = Limits::default();
    let definitions = std::env::var_os("OM9_BENCH_DEFINITIONS").is_some();
    let document = profile_cloud_document(8, definitions);
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let packet = ExportCapture::all(&coordinator, id).unwrap().encode(&limits).unwrap();
    let folder = Directory::new();
    let packet_path = Cleanup(folder.0.join("input.json"));
    fs::write(&packet_path.0, &packet).unwrap();
    println!("packet_bytes={}", packet.len());
    drop(packet);
    for repetition in 0..3 {
        let output = Cleanup(folder.0.join(format!("profile-{repetition}.3dm")));
        let result = program.run(&["--write".into(), packet_path.0.as_os_str().to_owned(), output.0.as_os_str().to_owned()],
            &ProcessLimits::default(), &Cancellation::default()).unwrap();
        let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(response["status"], "written");
        assert_eq!(response["objects"], if definitions { 16 } else { 8 });
        let samples = String::from_utf8(result.stderr).unwrap();
        for stage in ["packet_open", "preflight_done", "dom_ready", "native_model_ready", "archive_written"] {
            assert!(samples.contains(&format!("stage={stage} ")), "missing profile stage: {samples}");
        }
        println!("repetition={repetition}, job_peak={}, elapsed_ms={}\n{samples}", result.stats.peak_committed_bytes, result.stats.elapsed.as_millis());
    }
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn model_tolerance_is_cold_checked_and_unrepresentable_values_preserve_destination() {
    for (absolute, relative, angle) in [(0.02, 0.005, 0.1), (0.00001, 0.0001, 0.02)] {
        let folder = Directory::new();
        let path = folder.target();
        let mut document = Document::new("tolerance metadata");
        document.tolerance = Some(om9_core::history::Tolerance::try_new(absolute, relative, angle).unwrap());
        let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(entity.id, entity);
        let id = document.id;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let cancel = Cancellation::default();
        let receipt = native().prepare(&coordinator, ExportCapture::all(&coordinator, id).unwrap(), &path, &cancel)
            .unwrap().publish(&coordinator, &cancel).unwrap();
        assert!(receipt.bytes > 0);
    }
    for (relative, angle) in [(0., 0.1), (1., 0.1), (0.01, 0.), (0.01, 4.)] {
        let folder = Directory::new();
        let path = folder.target();
        fs::write(&path, b"original destination").unwrap();
        let mut document = Document::new("unrepresentable tolerance");
        document.tolerance = Some(om9_core::history::Tolerance::try_new(0.01, relative, angle).unwrap());
        let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(entity.id, entity);
        let id = document.id;
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        assert!(native().prepare(&coordinator, ExportCapture::all(&coordinator, id).unwrap(), &path, &Cancellation::default()).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original destination");
    }
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn selected_export_writes_only_selected_current_geometry() {
    let folder = Directory::new();
    let path = folder.target();
    let mut d = Document::new("selected");
    let a = Entity::new("A", Geometry::Point([1., 2., 3.]), None);
    let id = a.id;
    let b = Entity::new("B", Geometry::Point([4., 5., 6.]), None);
    d.entities.insert(id, a);
    d.entities.insert(b.id, b);
    let did = d.id;
    let limits = Limits::default();
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let cancel = Cancellation::default();
    let receipt = native().prepare(&c, ExportCapture::selected(&c, did, &[id]).unwrap(), &path, &cancel)
        .unwrap().publish(&c, &cancel).unwrap();
    assert_eq!(receipt.root_objects, 1);
    assert_eq!(c.world().documents[&did].entities.len(), 2);
    assert_eq!(c.undo_len(), 0);
    folder.assert_clean();
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn mesh_corner_uv_export_refuses_without_replacing_target() {
    use om9_core::geometry::Mesh;
    let folder=Directory::new(); let path=folder.target(); fs::write(&path,b"previous good").unwrap();
    let mut d=Document::new("UV mesh");
    let mesh=Mesh{vertices:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![[0.,0.],[1.,0.],[0.,1.]]};
    let entity=Entity::new("UV mesh",Geometry::Mesh(mesh),None); d.entities.insert(entity.id,entity);
    let id=d.id; let limits=Limits::default();
    let c=TransactionCoordinator::new(World::new(vec![d],&limits).unwrap(),limits).unwrap();
    let result=native().prepare(&c,ExportCapture::all(&c,id).unwrap(),&path,&Cancellation::default());
    assert!(result.is_err()); assert_eq!(fs::read(&path).unwrap(),b"previous good"); folder.assert_clean();
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn hidden_preset_native_projection_is_cold_verified_with_explicit_om9_losses() {
    use om9_core::{layers::PresetSlot, model::Layer};
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"previous good").unwrap();
    let limits = Limits::default();
    let mut d = Document::new("schema4 native projection");
    let layer = Layer::preset(PresetSlot::new(17).unwrap());
    let lid = layer.id;
    d.layer_order.push(lid);
    d.layers.insert(lid, layer);
    let mut e = Entity::new("hidden point", Geometry::Point([1., 2., 3.]), Some(lid));
    e.attributes.hidden = true;
    d.entities.insert(e.id, e);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &limits).unwrap(), limits).unwrap();
    let before = c.snapshot();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"previous good");
    let receipt = staged.publish(&c, &cancel).unwrap();
    assert_eq!(receipt.dropped_om9_hidden_distinctions, 1);
    assert_eq!(receipt.dropped_om9_preset_slots, 1);
    assert!(
        receipt
            .diagnostics
            .iter()
            .any(|message| message.contains("OM9 Hidden"))
    );
    assert!(
        receipt
            .diagnostics
            .iter()
            .any(|message| message.contains("preset slot"))
    );
    assert_eq!(c.snapshot(), before);
    record("hidden_preset_loss_projection", &receipt);
    folder.assert_clean();
}
#[test]
#[ignore = "requires built openNURBS operation executable"]
fn verified_current_geometry_replaces_destination_and_preserves_document_and_undo() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"previous good file").unwrap();
    let (c, did, _) = source();
    let before = c.snapshot();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"previous good file");
    let receipt = staged.publish(&c, &cancel).unwrap();
    record("accepted_current_point", &receipt);
    assert_eq!(receipt.root_objects, 1);
    assert_eq!(receipt.definition_members, 0);
    assert_eq!(receipt.dropped_history_recipes, 0);
    assert!(receipt.write.exited && receipt.inspect.exited);
    assert_eq!(receipt.write.active_processes, 0);
    assert_eq!(receipt.inspect.active_processes, 0);
    assert_eq!(
        receipt.sha256,
        <[u8; 32]>::from(Sha256::digest(fs::read(&path).unwrap()))
    );
    assert_eq!(c.snapshot(), before);
    folder.assert_clean();
}
#[test]
#[ignore = "requires built openNURBS operation executable"]
fn stale_after_commit_and_undo_cannot_publish_even_when_source_content_matches_again() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"good").unwrap();
    let (mut c, did, oid) = source();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    record("stale_commit_undo_rejected", staged.receipt());
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source: c.world().documents[&did].stamp(oid).unwrap(),
                placement: Transform::translation([9., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let before = c.snapshot();
    assert!(staged.publish(&c, &cancel).is_err());
    assert_eq!(c.snapshot(), before);
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
}
#[test]
#[ignore = "requires built openNURBS operation executable"]
fn cancel_after_validation_preserves_existing_file_and_releases_staging() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"good").unwrap();
    let (c, did, _) = source();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    record("late_cancel_rejected", staged.receipt());
    cancel.cancel();
    // The original operation owner remains authoritative even if publication
    // supplies a fresh token. Otherwise the canceled worker result revives.
    assert!(staged.publish(&c, &Cancellation::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
}
#[test]
#[ignore = "requires built openNURBS operation executable"]
fn unsupported_native_attribute_and_prelaunch_budget_failure_preserve_destination() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"good").unwrap();
    let (mut c, did, oid) = source();
    let mut attrs = c.world().documents[&did].entities[&oid].attributes.clone();
    attrs.selectable = false;
    let p = c
        .prepare(TransactionRequest::new(
            "Attribute",
            vec![Mutation::Attributes {
                source: c.world().documents[&did].stamp(oid).unwrap(),
                attributes: attrs,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let before = c.snapshot();
    assert!(
        native()
            .prepare(
                &c,
                ExportCapture::all(&c, did).unwrap(),
                &path,
                &Cancellation::default()
            )
            .is_err()
    );
    assert_eq!(c.snapshot(), before);
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
    let (c, did, _) = source();
    let mut limits = *c.limits();
    limits.max_bytes = 64 * 1024;
    let c = TransactionCoordinator::new(
        World::new(vec![c.world().documents[&did].as_ref().clone()], &limits).unwrap(),
        limits,
    )
    .unwrap();
    assert!(
        native()
            .prepare(
                &c,
                ExportCapture::all(&c, did).unwrap(),
                &path,
                &Cancellation::default()
            )
            .is_err()
    );
    folder.assert_clean();
}
#[test]
#[ignore = "requires built openNURBS operation executable"]
fn readonly_destination_after_validation_preserves_original_file() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"good").unwrap();
    let (c, did, _) = source();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    record("readonly_rejected", staged.receipt());
    let original = fs::metadata(&path).unwrap().permissions();
    let mut permissions = original.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).unwrap();
    assert!(staged.publish(&c, &cancel).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
    fs::set_permissions(&path, original).unwrap();
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn all_owned_master_kinds_shared_nested_instances_and_current_edits_survive_cold_native_validation()
{
    use om9_core::{
        geometry::{Cloud, Curve, Mesh, NurbsCurve, NurbsSurface},
        model::{Definition, Layer},
    };
    let mut d = Document::new("Current native masters");
    d.page_unit = Unit::Inch;
    let parent = Layer::new("Parent", [100, 110, 120, 200]);
    let mut child = Layer::new("Con", [200, 210, 220, 255]);
    child.parent = Some(parent.id);
    child.visible = false;
    child.locked = true;
    d.current_layer = Some(parent.id);
    d.layer_order = vec![parent.id, child.id];
    d.layers.insert(parent.id, parent.clone());
    d.layers.insert(child.id, child);
    let mut member = Entity::new("Member", Geometry::Point([1., 2., 3.]), parent.id);
    member.placement = Transform::translation([1., 0., 0.]);
    let master = Definition::new("Master", vec![member]);
    let mid = master.id;
    let mut nested_member = Entity::new("Nested member", Geometry::Instance(mid), parent.id);
    nested_member.placement = Transform::translation([0., 5., 0.]);
    let nested = Definition::new("Nested master", vec![nested_member]);
    let nid = nested.id;
    d.definitions.insert(mid, master);
    d.definitions.insert(nid, nested);
    let normal = [
        std::f64::consts::FRAC_1_SQRT_2,
        std::f64::consts::FRAC_1_SQRT_2,
        0.,
    ];
    let kinds = vec![
        Geometry::Point([1., 2., 3.]),
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [3., 4., 0.],
            domain: [7., 9.],
        }),
        Geometry::Curve(Curve::Circle {
            center: [2., 3., 4.],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 6.,
            domain: [0., 8.],
            angle_domain: Some([0., std::f64::consts::TAU]),
        }),
        Geometry::Curve(Curve::Nurbs(NurbsCurve {
            degree: 2,
            poles: vec![[10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
            weights: vec![1., std::f64::consts::FRAC_1_SQRT_2, 1.],
            knots: vec![2., 2., 2., 4., 4., 4.],
            domain: [2., 4.],
            periodic: false,
            reversed: false,
            seam: None,
        })),
        Geometry::Surface(NurbsSurface {
            degree: [1, 1],
            size: [2, 3],
            poles: vec![
                [0., 0., 0.],
                [0., 1., 1.],
                [0., 2., 0.],
                [5., 0., 2.],
                [5., 1., 3.],
                [5., 2., 2.],
            ],
            weights: vec![1., 2., 1., 1., 3., 1.],
            knots: [vec![0., 0., 2., 2.], vec![3., 3., 4., 5., 5.]],
            domain: [[0., 2.], [3., 5.]],
            periodic: [false, false],
            reversed: false,
        }),
        Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.]],
            faces: vec![vec![0, 1, 2], vec![0, 1, 2, 3]],
            normals: vec![normal.map(|n| n * 2.); 4],
            colors: vec![[50, 100, 150, 210]; 4],

            corner_uvs: Vec::new(),
        }),
        Geometry::Cloud(Cloud {
            points: vec![[2., 3., 4.], [5., 6., 7.]],
            normals: vec![normal, [0., 1., 0.]],
            colors: vec![[255, 0, 0, 255], [0, 255, 0, 128]],
            hidden: vec![false; 2],
        }),
        Geometry::Curve(Curve::Polycurve {
            segments: vec![
                Curve::Line {
                    start: [0., 0., 0.],
                    end: [1., 0., 0.],
                    domain: [0., 1.],
                },
                Curve::Line {
                    start: [1., 0., 0.],
                    end: [1., 1., 0.],
                    domain: [1., 2.],
                },
            ],
            reversed: false,
        }),
        Geometry::Instance(nid),
        Geometry::Instance(mid),
        Geometry::Curve(Curve::Nurbs(NurbsCurve {
            degree: 2,
            poles: vec![
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                [0., 0., 0.],
                [1., 0., 0.],
            ],
            weights: vec![1.; 5],
            knots: vec![-1., -1., 0., 1., 2., 3., 4., 4.],
            domain: [0., 3.],
            periodic: true,
            reversed: false,
            seam: None,
        })),
    ];
    let mut ids = vec![];
    for (index, g) in kinds.into_iter().enumerate() {
        let mut e = Entity::new(format!("Master {index}"), g, parent.id);
        if index == 0 {
            e.attributes.layer = None;
            e.placement = Transform::translation([10., 20., 30.]);
        }
        if index == 1 {
            e.attributes.locked = true;
        }
        if index == 2 {
            e.attributes.visible = false;
            e.attributes.color = None;
            e.placement = Transform([
                [-2., 0., 0., 10.],
                [0., 2., 0., 20.],
                [0., 0., 2., 30.],
                [0., 0., 0., 1.],
            ]);
        }
        if index == 5 || index == 6 {
            e.placement = Transform([
                [0., -3., 0., 10.],
                [2., 0., 0., 20.],
                [0., 0., 4., 30.],
                [0., 0., 0., 1.],
            ]);
        }
        if index == 8 {
            e.placement = Transform([
                [-2., 0., 0., 10.],
                [0., 3., 0., 20.],
                [0., 0., 4., 30.],
                [0., 0., 0., 1.],
            ]);
        }
        ids.push(e.id);
        d.entities.insert(e.id, e);
    }
    let did = d.id;
    let l = Limits::default();
    let mut c = TransactionCoordinator::new(World::new(vec![d], &l).unwrap(), l).unwrap();
    let folder = Directory::new();
    let path = folder.target();
    let cancel = Cancellation::default();
    let adapter = native();
    let receipt = adapter
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap()
        .publish(&c, &cancel)
        .unwrap();
    assert_eq!(receipt.root_objects, 11);
    assert_eq!(receipt.definition_members, 2);
    assert!(
        receipt
            .diagnostics
            .iter()
            .any(|d| d.contains("normalized inverse-transpose"))
    );
    assert!(
        receipt
            .diagnostics
            .iter()
            .any(|d| d.contains("single precision"))
    );
    record("accepted_all_masters", &receipt);
    folder.assert_clean();
    let old = ids[0];
    let replacement = Entity::new(
        "Created after delete",
        Geometry::Point([19., 18., 17.]),
        None,
    );
    let new = replacement.id;
    let p = c
        .prepare(TransactionRequest::new(
            "Delete B / Create C",
            vec![
                Mutation::Delete {
                    source: c.world().documents[&did].stamp(old).unwrap(),
                },
                Mutation::Insert {
                    document: did,
                    entity: replacement,
                },
                Mutation::Placement {
                    source: c.world().documents[&did].stamp(ids[5]).unwrap(),
                    placement: Transform([
                        [-2., 0., 0., 10.],
                        [0., 3., 0., 20.],
                        [0., 0., 4., 30.],
                        [0., 0., 0., 1.],
                    ]),
                },
            ],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let capture = ExportCapture::all(&c, did).unwrap();
    assert!(!capture.document().entities.contains_key(&old));
    assert!(capture.document().entities.contains_key(&new));
    let before = c.snapshot();
    let receipt = adapter
        .prepare(&c, capture, &path, &cancel)
        .unwrap()
        .publish(&c, &cancel)
        .unwrap();
    record("accepted_current_edit_delete_new", &receipt);
    assert_eq!(receipt.root_objects, 11);
    assert_eq!(receipt.definition_members, 2);
    assert_eq!(c.snapshot(), before);
    folder.assert_clean();
}

#[test]
#[ignore = "requires built openNURBS operation executable"]
fn close_and_staged_file_tampering_reject_before_destination_replacement() {
    let folder = Directory::new();
    let path = folder.target();
    fs::write(&path, b"good").unwrap();
    let (mut c, did, _) = source();
    let cancel = Cancellation::default();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    record("closed_owner_rejected", staged.receipt());
    c.close_document(did).unwrap();
    assert!(staged.publish(&c, &cancel).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
    let (c, did, _) = source();
    let staged = native()
        .prepare(&c, ExportCapture::all(&c, did).unwrap(), &path, &cancel)
        .unwrap();
    record("changed_staging_rejected", staged.receipt());
    let staging = fs::read_dir(&folder.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.is_dir())
        .unwrap();
    fs::write(
        staging.join("current.3dm"),
        b"changed after native cold validation",
    )
    .unwrap();
    assert!(staged.publish(&c, &cancel).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"good");
    folder.assert_clean();
}
