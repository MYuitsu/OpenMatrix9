// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in large allocation regression; uses real retained resource bytes.
#[test]
#[ignore = "allocates over 256 MiB; run in an isolated process with sufficient free RAM"]
fn interactive_document_accepts_real_resource_above_old_256_mib_limit() {
    use om9_core::{budget::Limits, model::Document, resource::{Resource, ResourceKind}};
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 400 * MIB, "fixture needs a managed ceiling above 400 MiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 768 * MIB, "fixture needs enough available physical RAM");
    let input = vec![0x5a; 270 * MIB as usize];
    let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
    let resource = Resource::from_bytes("memory fixture", ResourceKind::OpaqueBinary, &input, &limits).unwrap();
    drop(input);
    let mut document = Document::with_template_tolerance("large managed memory fixture");
    document.resources.insert(resource.id, resource);
    let app = om9_app::ui::MatrixApp::new(document).expect("interactive startup must admit >256 MiB when RAM permits");
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
    assert!(app.controller.coordinator().limits().max_bytes > 256 * MIB);
}

fn large_mesh_document(width: usize, height: usize, triangles_per_cell: usize) -> (om9_core::model::Document, om9_core::identity::ObjectId) {
    use om9_core::{geometry::{Geometry, Mesh}, model::{Document, Entity}};
    assert!(matches!(triangles_per_cell, 1 | 3));
    let vertices: Vec<_> = (0..width * height).map(|i| [(i % width) as f64, (i / width) as f64, 0.]).collect();
    // Three triangles per cell deliberately overlap; one is an open mesh.
    let mut faces = Vec::with_capacity((width - 1) * (height - 1) * triangles_per_cell);
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let i = (y * width + x) as u32;
            faces.push(vec![i, i + 1, i + width as u32]);
            if triangles_per_cell == 3 {
                faces.push(vec![i + 1, i + width as u32 + 1, i + width as u32]);
                faces.push(vec![i, i + 1, i + width as u32 + 1]);
            }
        }
    }
    let mesh = Mesh {
        normals: vec![[0., 0., 1.]; vertices.len()],
        colors: vec![[200, 200, 200, 255]; vertices.len()],
        corner_uvs: vec![[0., 0.]; faces.len() * 3],
        vertices, faces,
    };
    let mut document = Document::with_template_tolerance("large authored mesh fixture");
    let entity = Entity::new("grid mesh", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    (document, id)
}

#[test]
fn incremental_point_edit_does_not_sum_mesh_nodes_across_history() {
    use om9_app::session::Controller;
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
    let limits = Limits { max_nodes: 100, ..Limits::default() };
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
        faces: vec![vec![0,1,2]; 70], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let mut document = Document::with_template_tolerance("node history");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("1,2,3").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
}

#[test]
#[ignore = "large real mesh; run in an isolated process with ample free RAM"]
fn interactive_document_accepts_real_mesh_above_old_256_mib_limit() {
    use om9_core::geometry::Geometry;
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 2_048 * MIB, "fixture needs a managed ceiling above 2 GiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 3_072 * MIB, "fixture needs at least 3 GiB above the physical reserve");
    let (document, id) = large_mesh_document(1_000, 1_000, 3);
    let app = om9_app::ui::MatrixApp::new(document).expect("interactive startup must admit a >256 MiB mesh when RAM permits");
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else { panic!("expected mesh") };
    assert_eq!(mesh.vertices.len(), 1_000_000);
    assert_eq!(mesh.faces.len(), 2_994_003);
}

#[test]
#[ignore = "2 million-point mesh; run in an isolated process with ample free RAM"]
fn interactive_two_million_source_and_display_points_are_not_summed() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::geometry::Geometry;
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 2_048 * MIB, "fixture needs a managed ceiling above 2 GiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 3_072 * MIB, "fixture needs at least 3 GiB above the physical reserve");
    let (document, id) = large_mesh_document(2_000, 1_000, 1);
    let mut app = om9_app::ui::MatrixApp::new(document).expect("source and display point quotas must not be added together");
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else { panic!("expected mesh") };
    assert_eq!(mesh.vertices.len(), 2_000_000);
    assert_eq!(mesh.faces.len(), 1_997_001);
    app.controller.select_objects(&[id]).unwrap();
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    app.controller.cancel_file_operation(token).unwrap();
}

#[test]
#[ignore = "large mesh archive roundtrip; run with ample RAM and disk space"]
fn interactive_large_mesh_survives_save_new_and_open() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::geometry::Geometry;
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 3_072 * MIB, "fixture needs a managed ceiling above 3 GiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 4_096 * MIB, "fixture needs at least 4 GiB above the physical reserve");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-large-mesh-roundtrip-{}.om9", uuid::Uuid::new_v4())));
    let (document, id) = large_mesh_document(1_000, 1_000, 3);
    let mut app = om9_app::ui::MatrixApp::new(document).unwrap();
    let initial = *app.controller.coordinator().limits();
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Saved);
    println!("large_mesh_archive_bytes={}", std::fs::metadata(&target.0).unwrap().len());
    assert_eq!(app.controller.begin_file_operation(FileKind::New).unwrap(), FileOutcome::Replaced);
    assert_eq!(app.controller.begin_file_operation(FileKind::Open).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else { panic!("expected mesh") };
    assert_eq!(mesh.vertices.len(), 1_000_000);
    assert_eq!(mesh.faces.len(), 2_994_003);
    assert_eq!(mesh.vertices[1_001], [1., 1., 0.]);
    assert_eq!(mesh.faces[0], [0, 1, 1_000]);
    assert_eq!(mesh.corner_uvs.len(), 2_994_003 * 3);
}

#[test]
#[ignore = "writes and reads a >256 MiB geometry archive; run with ample RAM and disk space"]
fn interactive_geometry_file_above_256_mib_survives_save_new_and_open() {
    use om9_app::{file_lifecycle::{FileKind, FileOutcome, UnsavedDecision}, session::Controller};
    use om9_core::{budget::Limits, geometry::Geometry, model::Document};
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 4_096 * MIB, "fixture needs a managed ceiling above 4 GiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 6_144 * MIB, "fixture needs at least 6 GiB above the physical reserve");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-large-geometry-file-{}.om9", uuid::Uuid::new_v4())));
    let (mut document, id) = large_mesh_document(1_000, 1_000, 3);
    let Geometry::Mesh(mesh) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("expected mesh") };
    // Nonzero UVs add real mesh attributes to the archive without exceeding the
    // independent structural limits by adding millions more faces.
    mesh.corner_uvs.fill([0.123456789, 0.25]);
    let mut app = om9_app::ui::MatrixApp::new(document).unwrap();
    let initial = *app.controller.coordinator().limits();
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
    // Exercise the ordinary single-entity transaction while a genuinely large
    // mesh and its display scene remain live; the edit must undo and redo.
    app.controller.submit("Point").unwrap();
    app.controller.submit("1234,2345,3456").unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Saved);
    let bytes = std::fs::metadata(&target.0).unwrap().len();
    println!("geometry_archive_bytes={bytes}");
    assert!(bytes > 256 * MIB, "the file itself must exceed the former limit");
    // Exercise refusal against the real large archive while an unrelated small
    // document has both Undo and Redo, without exhausting physical RAM.
    let mut limited = Controller::new(Document::with_template_tolerance("preserved"), Limits::default()).unwrap();
    for point in ["1,2,3", "4,5,6"] { limited.submit("Point").unwrap(); limited.submit(point).unwrap(); }
    limited.undo().unwrap();
    let before = limited.coordinator().snapshot();
    let undo_before = limited.coordinator().undo_len();
    let redo_before = limited.coordinator().redo_len();
    assert!(undo_before > 0 && redo_before > 0);
    limited.begin_file_operation(FileKind::Open).unwrap();
    let token = limited.file_operation().unwrap().token();
    limited.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    let token = limited.file_operation().unwrap().token();
    let refused = limited.accept_file_path(token, &target.0);
    assert!(matches!(refused, Err(om9_core::Error::Budget(_))), "large file must be refused for budget: {refused:?}");
    assert_eq!(limited.coordinator().snapshot(), before);
    assert_eq!(limited.coordinator().undo_len(), undo_before);
    assert_eq!(limited.coordinator().redo_len(), redo_before);
    assert_eq!(std::fs::metadata(&target.0).unwrap().len(), bytes);
    limited.redo().expect("Redo must remain usable after refusal");
    limited.undo().expect("Undo must remain usable after refusal");
    drop(limited);
    assert_eq!(app.controller.begin_file_operation(FileKind::New).unwrap(), FileOutcome::Replaced);
    assert_eq!(app.controller.begin_file_operation(FileKind::Open).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else { panic!("expected mesh") };
    assert_eq!(mesh.vertices.len(), 1_000_000);
    assert_eq!(mesh.faces.len(), 999 * 999 * 3);
    assert_eq!(mesh.vertices[1_001], [1., 1., 0.]);
    assert_eq!(mesh.faces[0], [0, 1, 1_000]);
    assert_eq!(mesh.corner_uvs.len(), mesh.faces.len() * 3);
    assert_eq!(mesh.corner_uvs[0], [0.123456789, 0.25]);
    drop(app);
    use std::io::Read;
    let mut inspected = std::process::Command::new(env!("CARGO_BIN_EXE_om9"))
        .arg("--inspect").arg(&target.0).stdout(std::process::Stdio::piped()).spawn().unwrap();
    let mut stdout = inspected.stdout.take().unwrap();
    let mut prefix = Vec::new();
    (&mut stdout).take(65_536).read_to_end(&mut prefix).unwrap();
    std::io::copy(&mut stdout, &mut std::io::sink()).unwrap();
    assert!(inspected.wait().unwrap().success(), "large inspect failed");
    assert!(String::from_utf8_lossy(&prefix).contains("grid mesh"));
}

#[test]
#[ignore = "writes and reads a >256 MiB archive; run with ample RAM and disk space"]
fn interactive_large_resource_survives_save_new_and_open() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::{budget::Limits, model::Document, resource::{Resource, ResourceKind}};
    const MIB: u64 = 1024 * 1024;
    let sample = om9_native_owner::physical_memory().expect("physical RAM sample required");
    assert!(sample.total_bytes * 3 / 5 > 2_048 * MIB, "fixture needs a managed ceiling above 2 GiB");
    assert!(sample.available_bytes > (sample.total_bytes / 10).max(256 * MIB) + 2_048 * MIB, "fixture needs at least 2 GiB above the physical reserve");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let target = Cleanup(std::env::temp_dir().join(format!("om9-large-ram-roundtrip-{}.om9", uuid::Uuid::new_v4())));
    let input = vec![0x5a; 270 * MIB as usize];
    let limits = Limits { max_bytes: 1024 * MIB, ..Limits::default() };
    let resource = Resource::from_bytes("memory fixture", ResourceKind::OpaqueBinary, &input, &limits).unwrap();
    let id = resource.id;
    let digest = resource.sha256();
    drop(input);
    let mut document = Document::with_template_tolerance("large roundtrip");
    document.resources.insert(id, resource);
    let mut app = om9_app::ui::MatrixApp::new(document).unwrap();
    let initial = *app.controller.coordinator().limits();
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Saved);
    assert!(std::fs::metadata(&target.0).unwrap().len() > 256 * MIB);
    assert_eq!(app.controller.begin_file_operation(FileKind::New).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    assert_eq!(app.controller.begin_file_operation(FileKind::Open).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &target.0).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    let restored = &app.controller.document().resources[&id];
    assert_eq!(restored.byte_length(), 270 * MIB);
    assert_eq!(restored.sha256(), digest);
    assert_eq!(restored.payload().unwrap()[0], 0x5a);
    assert!(app.controller.coordinator().retained_usage().unwrap().bytes > 256 * MIB);
}

#[test]
fn interactive_new_and_open_keep_the_same_session_limit() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::{budget::Limits, model::Document, transaction::Cancellation};
    let path = std::env::temp_dir().join(format!("om9-memory-policy-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &Document::with_template_tolerance("fixture"), &Limits::default(), &Cancellation::default()).unwrap();
    let mut app = om9_app::ui::MatrixApp::new(Document::with_template_tolerance("original")).unwrap();
    let initial = *app.controller.coordinator().limits();
    assert_eq!(app.controller.begin_file_operation(FileKind::New).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    assert_eq!(app.controller.begin_file_operation(FileKind::Open).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &path).unwrap(), FileOutcome::Replaced);
    assert_eq!(*app.controller.coordinator().limits(), initial);
    assert_eq!(app.controller.document().name, path.file_name().unwrap().to_string_lossy());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn file_operation_does_not_double_count_source_and_scene_points() {
    use om9_app::{file_lifecycle::{FileKind, FileOutcome}, session::Controller};
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
    let mut document = Document::with_template_tolerance("source and scene points");
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(entity.id, entity);
    let limits = Limits { max_points: 3, ..Limits::default() };
    let mut controller = Controller::new(document, limits).unwrap();
    assert_eq!(controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = controller.file_operation().unwrap().token();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
    let path = Cleanup(std::env::temp_dir().join(format!("om9-points-limit-{}.om9", uuid::Uuid::new_v4())));
    assert_eq!(controller.accept_file_path(token, &path.0).unwrap(), FileOutcome::Saved);
    let stamp = controller.document().stamp(id).unwrap();
    controller.set_objects_hidden(vec![stamp], true).unwrap();
    controller.undo().unwrap();
}

#[test]
fn inserting_geometry_at_document_point_limit_keeps_scene_budget_independent() {
    use om9_app::session::Controller;
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
    let mut document = Document::with_template_tolerance("source and scene insertion");
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    let limits = Limits { max_points: 4, ..Limits::default() };
    let mut controller = Controller::new(document, limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("2,2,2").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
}

#[test]
fn paste_at_document_point_limit_keeps_scene_budget_independent() {
    use om9_app::session::Controller;
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
    let mut document = Document::with_template_tolerance("paste source and scene points");
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let limits = Limits { max_points: 6, ..Limits::default() };
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let data = controller.copy_selected_om9().unwrap();
    assert_eq!(controller.paste_om9(&data).unwrap().len(), 1);
    assert_eq!(controller.document().entities.len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 2);
}

#[test]
fn paste_over_document_point_limit_keeps_document_and_history() {
    use om9_app::session::Controller;
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}};
    let mut document = Document::with_template_tolerance("paste point refusal");
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits { max_points: 5, ..Limits::default() }).unwrap();
    controller.select_objects(&[id]).unwrap();
    let data = controller.copy_selected_om9().unwrap();
    let before = controller.coordinator().snapshot();
    assert!(controller.paste_om9(&data).is_err());
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.document().entities.len(), 1);
}

#[test]
fn undo_and_redo_refused_under_memory_pressure_keep_the_scene_and_history() {
    use om9_app::session::Controller;
    use om9_core::{budget::{Limits, memory::MemoryClass}, model::Document};
    let limits = Limits { max_bytes: 2 * 1024 * 1024, ..Limits::default() };
    let mut controller = Controller::new(Document::with_template_tolerance("history pressure"), limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("1,2,3").unwrap();
    for redo in [false, true] {
        let held = controller.scene_cache_usage().unwrap();
        let mut low = 0u64;
        let mut high = limits.max_bytes;
        while low < high {
            let mid = low + (high - low + 1) / 2;
            let trial = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, mid, None, held);
            if trial.is_ok() { low = mid; } else { high = mid - 1; }
        }
        let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, low, None, held).unwrap();
        let snapshot = controller.coordinator().snapshot();
        let scene_epoch = controller.scene().unwrap().epoch;
        let scene_objects = controller.scene().unwrap().objects.len();
        let result = if redo { controller.redo() } else { controller.undo() };
        assert!(result.is_err());
        assert_eq!(controller.coordinator().snapshot(), snapshot);
        assert_eq!(controller.scene().unwrap().epoch, scene_epoch);
        assert_eq!(controller.scene().unwrap().objects.len(), scene_objects);
        drop(reservation);
        if redo { controller.redo().unwrap(); } else { controller.undo().unwrap(); }
        assert_eq!(controller.scene().unwrap().epoch, controller.coordinator().epoch());
    }
}

#[test]
fn numeric_gumball_refusal_under_pressure_preserves_document_scene_and_redo() {
    use om9_app::session::Controller;
    use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::Geometry, model::{Document, Entity}};
    let limits = Limits { max_bytes: 2 * 1024 * 1024, ..Limits::default() };
    let mut document = Document::with_template_tolerance("gumball pressure");
    let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.numeric_move([1., 0., 0.]).unwrap();
    controller.undo().unwrap();
    assert_eq!(controller.coordinator().redo_len(), 1);
    let held = controller.scene_cache_usage().unwrap();
    let (mut low, mut high) = (0u64, limits.max_bytes);
    while low < high {
        let mid = low + (high - low + 1) / 2;
        let trial = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, mid, None, held);
        if trial.is_ok() { low = mid; } else { high = mid - 1; }
    }
    let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, low, None, held).unwrap();
    let before = controller.coordinator().snapshot();
    let scene_epoch = controller.scene().unwrap().epoch;
    assert!(controller.numeric_move([2., 0., 0.]).is_err());
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.scene().unwrap().epoch, scene_epoch);
    assert_eq!(controller.coordinator().redo_len(), 1);
    drop(reservation);
    controller.numeric_move([2., 0., 0.]).unwrap();
    assert_eq!(controller.coordinator().redo_len(), 0);
    assert_eq!(controller.scene().unwrap().epoch, controller.coordinator().epoch());
}

#[test]
fn delete_refusal_under_pressure_preserves_selection_scene_and_redo() {
    use om9_app::session::Controller;
    use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::Geometry, model::{Document, Entity}};
    let limits = Limits { max_bytes: 2 * 1024 * 1024, ..Limits::default() };
    let mut document = Document::with_template_tolerance("delete pressure");
    let point = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    let id = point.id;
    document.entities.insert(id, point);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.numeric_move([1., 0., 0.]).unwrap();
    controller.undo().unwrap();
    let held = controller.scene_cache_usage().unwrap();
    let (mut low, mut high) = (0u64, limits.max_bytes);
    while low < high {
        let mid = low + (high - low + 1) / 2;
        let trial = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, mid, None, held);
        if trial.is_ok() { low = mid; } else { high = mid - 1; }
    }
    let reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Export, low, None, held).unwrap();
    let before = controller.coordinator().snapshot();
    let scene_epoch = controller.scene().unwrap().epoch;
    assert!(controller.delete_objects(&[id]).is_err());
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.scene().unwrap().epoch, scene_epoch);
    assert_eq!(controller.selected_ordered(), vec![id]);
    assert_eq!(controller.coordinator().redo_len(), 1);
    drop(reservation);
    assert_eq!(controller.delete_objects(&[id]).unwrap(), 1);
    assert!(controller.selected_ordered().is_empty());
    assert_eq!(controller.coordinator().redo_len(), 0);
}
