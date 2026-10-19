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
fn validation_copy_shares_point_budget_with_live_model_before_file_mutation() {
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
    assert!(Archive::save(&path, &d, &limits, &Cancellation::default()).is_err());
    assert_eq!(fs::read(&path).unwrap(), baseline);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
