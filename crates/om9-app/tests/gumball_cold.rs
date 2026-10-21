// SPDX-License-Identifier: LGPL-2.1-or-later
//! A committed generic transform survives an archive round trip and a fresh executable.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::Transform,
    gumball::{Drag, Handle, Session},
    model::Document,
    transaction::Cancellation,
};

#[test]
fn a_gumball_commit_reopens_in_a_fresh_process_with_the_same_id_and_placement() {
    let limits = Limits::default();
    let mut c = Controller::new(Document::new("gumball cold"), limits).unwrap();
    c.submit("Line").unwrap();
    c.submit("-5,-3,0").unwrap();
    c.submit("5,3,0").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let mut gesture =
        Session::begin(c.gumball_pivot().unwrap(), c.gumball_targets(), false).unwrap();
    gesture
        .drag(Handle::MoveFree, Drag::WorldDelta([5., 2., 1.]))
        .unwrap();
    c.commit_gumball(&gesture, "Move").unwrap();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/standalone-validation")
        .join(format!("m02-cold-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("fixture.om9");
    om9_archive::Archive::save(&path, c.document(), &limits, &Cancellation::default()).unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_om9"));
    command.env_clear();
    for key in ["SystemRoot", "TEMP", "TMP", "PATH"] {
        if let Some(v) = std::env::var_os(key) {
            command.env(key, v);
        }
    }
    let output = command.arg("--inspect").arg(&path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["objects"][0]["id"], id.to_string());
    assert_eq!(
        report["objects"][0]["placement"],
        serde_json::to_value(Transform::translation([5., 2., 1.])).unwrap()
    );
    let restored = Controller::from_file(&path, limits).unwrap();
    assert_eq!(
        restored.document().entities[&id].placement,
        Transform::translation([5., 2., 1.])
    );
    assert!(
        restored.selected().is_empty(),
        "selection is a session state"
    );
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
