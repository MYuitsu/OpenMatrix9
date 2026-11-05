// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
    transaction::Cancellation,
};
use std::{fs, path::PathBuf};
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("om9-archive-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn successful_same_volume_replacement_reopens_current_document() {
    let dir = Directory::new();
    let path = dir.0.join("model.om9");
    let mut d = Document::new("before");
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    d.name = "after".into();
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    assert_eq!(Archive::open(&path, &Limits::default()).unwrap(), d);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
#[test]
fn canceled_and_invalid_save_preserve_good_destination_and_remove_staging() {
    let dir = Directory::new();
    let path = dir.0.join("model.om9");
    let mut d = Document::new("good");
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    let baseline = fs::read(&path).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(Archive::save(&path, &d, &Limits::default(), &cancel).is_err());
    assert_eq!(fs::read(&path).unwrap(), baseline);
    let e = Entity::new(
        "invalid",
        Geometry::Point([f64::NAN, 0., 0.]),
        d.current_layer,
    );
    d.entities.insert(e.id, e);
    assert!(Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), baseline);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
#[test]
fn destination_permission_failure_preserves_existing_file_and_removes_staging() {
    let dir = Directory::new();
    let path = dir.0.join("model.om9");
    let d = Document::new("good");
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    let baseline = fs::read(&path).unwrap();
    let original_permissions = fs::metadata(&path).unwrap().permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions.clone()).unwrap();
    let result = Archive::save(
        &path,
        &Document::new("changed"),
        &Limits::default(),
        &Cancellation::default(),
    );
    fs::set_permissions(&path, original_permissions).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), baseline);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[test]
fn validation_copy_keeps_per_document_point_limit_and_preserves_invalid_destination() {
    let dir = Directory::new();
    let path = dir.0.join("model.om9");
    let mut d = Document::new("point");
    let e = Entity::new("point", Geometry::Point([1., 2., 3.]), d.current_layer);
    d.entities.insert(e.id, e);
    Archive::save(&path, &d, &Limits::default(), &Cancellation::default()).unwrap();
    let baseline = fs::read(&path).unwrap();
    let limits = Limits {
        max_points: 1,
        ..Limits::default()
    };
    assert!(d.validate(&limits).is_ok());
    Archive::save(&path, &d, &limits, &Cancellation::default()).unwrap();
    let extra = Entity::new("extra", Geometry::Point([4., 5., 6.]), d.current_layer);
    d.entities.insert(extra.id, extra);
    assert!(Archive::save(&path, &d, &limits, &Cancellation::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), baseline);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[test]
fn save_verification_keeps_single_document_node_limit() {
    use om9_core::geometry::Mesh;
    let dir = Directory::new();
    let path = dir.0.join("nodes.om9");
    let mut document = Document::new("node limit");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: (0..100).map(|_| vec![0, 1, 2]).collect(),
        normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), document.current_layer);
    document.entities.insert(entity.id, entity);
    let bytes = Archive::encode(&document, &Limits::default()).unwrap();
    let (mut low, mut high) = (1, 10_000);
    while low < high {
        let middle = (low + high) / 2;
        let candidate = Limits { max_nodes: middle, ..Limits::default() };
        if Archive::decode(&bytes, &candidate).is_ok() { high = middle; } else { low = middle + 1; }
    }
    let limits = Limits { max_nodes: low, ..Limits::default() };
    assert_eq!(Archive::decode(&bytes, &limits).unwrap(), document);
    Archive::save(&path, &document, &limits, &Cancellation::default()).expect("a valid document must fit its own structural limit during verification");
    assert_eq!(Archive::open(&path, &limits).unwrap(), document);
}

#[test]
fn stage_admission_reports_overlap_and_releases_before_readback_publication() {
    use om9_core::Error;
    let dir = Directory::new();
    let path = dir.0.join("phases.om9");
    let document = Document::new("phase admission");
    let limits = Limits::default();
    let encoded = Archive::encode(&document, &limits).unwrap();
    let mut saw_overlap = false;
    let mut saw_readback = false;
    let mut previous_live = 0;
    let mut saw_release = false;
    Archive::save_with_admission(&path, &document, &limits, &Cancellation::default(), |live, pending| {
        assert!(live + pending < 1024 * 1024, "small archive must not reserve the entire default headroom");
        saw_release |= live < previous_live;
        previous_live = live;
        if live > 0 && pending > 0 { saw_overlap = true; }
        if fs::read_dir(&dir.0).unwrap().count() > 0 && live >= encoded.len() as u64 {
            saw_readback = true;
        }
        Ok(())
    }).unwrap();
    assert!(saw_overlap && saw_readback && saw_release);
    assert_eq!(fs::read(&path).unwrap(), encoded);

    // Refuse specifically after the staging file exists. Destination and
    // staging cleanup must remain atomic even on a late phase-growth refusal.
    let result = Archive::save_with_admission(&path, &Document::new("replacement"), &limits,
        &Cancellation::default(), |_, pending| {
            if pending > 0 && fs::read_dir(&dir.0).unwrap().count() > 1 {
                Err(Error::Budget("injected readback admission"))
            } else { Ok(()) }
        });
    assert!(matches!(result, Err(Error::Budget("injected readback admission"))));
    assert_eq!(fs::read(&path).unwrap(), encoded);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
