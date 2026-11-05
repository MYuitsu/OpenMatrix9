// SPDX-License-Identifier: LGPL-2.1-or-later
//! Every command of the OM9 Matrix package, through the real controller, on a small scene
//! (an open polyline, a circle, a box, a stone, a NURBS plane): start it, hover, click a few
//! points, type a number, press Enter twice, cancel what is still open and undo what it made.
//! Nothing may panic, and the document must come back to the scene each time.
use om9_addon_abi::session::session_json;
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{LoadRequest, Runtime};
use om9_app::addons::FeatureName;
use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::model::Document;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

static PANICS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct InProcess;
impl Runtime for InProcess {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn session(&mut self, _id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        let (code, out) = session_json::<om9_matrix::Matrix>(event);
        if code == om9_addon_abi::INVOKE_PANIC {
            let ev: serde_json::Value = serde_json::from_slice(event).unwrap_or_default();
            PANICS
                .lock()
                .unwrap()
                .push(format!("{} on {}", ev["command"], ev["input"]));
        }
        if code == 0 {
            Ok(out)
        } else {
            Err(String::from_utf8_lossy(&out).into())
        }
    }
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn host(tmp: &Path) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(
        om9_app::addons::om9_version(),
        catalog.clone(),
        tmp.join("config"),
    ));
    h.register_runtime(RuntimeKind::Python, Box::new(InProcess));
    h.startup(&Startup::default());
    let stage = tmp.join("stage");
    fs::create_dir_all(stage.join("icons")).unwrap();
    let addon = repo().join("addons/matrix/addon");
    let manifest = fs::read_to_string(addon.join("om9_addon.toml"))
        .unwrap()
        .replace("@PLATFORM@", "any")
        .replace("runtime = \"native\"", "runtime = \"python\"")
        .replace("@ENTRY@", "entry.bin");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::write(stage.join("entry.bin"), b"in-process").unwrap();
    for e in fs::read_dir(addon.join("icons")).unwrap() {
        let e = e.unwrap().path();
        fs::copy(&e, stage.join("icons").join(e.file_name().unwrap())).unwrap();
    }
    // The interface tables (lang/vi.toml) travel with the package.
    fs::create_dir_all(stage.join("lang")).unwrap();
    for e in fs::read_dir(addon.join("lang")).unwrap() {
        let e = e.unwrap().path();
        fs::copy(&e, stage.join("lang").join(e.file_name().unwrap())).unwrap();
    }
    let (_, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    let path = tmp.join("matrix.zip");
    fs::write(&path, zip).unwrap();
    let r = h
        .install_zip(
            &path,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    Arc::new(Mutex::new(h))
}

fn features() -> Vec<(String, Option<String>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../assets/features.json")).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["id"].as_str().unwrap().to_owned(),
                f["command"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

/// Run a command to its end with clicks / typed values.
fn make(c: &mut Controller, code: &str, points: &[[f64; 3]], texts: &[&str]) {
    c.run_addon_command(code).unwrap();
    for t in texts.iter().take_while(|t| t.starts_with('#')) {
        c.submit(&t[1..]).unwrap();
    }
    for p in points {
        c.session_point(*p).unwrap();
    }
    for t in texts.iter().filter(|t| !t.starts_with('#')) {
        c.submit(t).unwrap();
    }
    assert!(
        c.addon_session().is_none(),
        "{code} did not finish: {}",
        c.prompt_text()
    );
}

#[test]
fn every_package_command_runs_in_om9_without_panics() {
    let tmp = std::env::temp_dir().join(format!("om9-app-smoke-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("smoke"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    // The scene.
    make(
        &mut c,
        "CurvePolylinePolyline",
        &[
            [-8.0, -2.0, 0.0],
            [-4.0, 2.0, 0.0],
            [0.0, -2.0, 0.0],
            [4.0, 2.0, 0.0],
        ],
        &[""],
    );
    make(
        &mut c,
        "CurveCircleCenterRadius",
        &[[0.0, 0.0, 0.0]],
        &["5"],
    );
    make(
        &mut c,
        "SolidBoxCornertoCornerHeight",
        &[[-3.0, -3.0, 0.0], [3.0, 3.0, 0.0]],
        &["3"],
    );
    make(&mut c, "BuilderGemLoader", &[[6.0, 6.0, 3.0]], &[""]);
    make(
        &mut c,
        "SurfacePlaneCornertoCorner",
        &[[-10.0, -10.0, -1.0], [10.0, 10.0, -1.0]],
        &[],
    );
    let scene: Vec<_> = c.document().entities.keys().copied().collect();
    assert_eq!(scene.len(), 5);
    let baseline = c.coordinator().undo_len();

    let text = fs::read_to_string(repo().join("addons/matrix/addon/om9_addon.toml")).unwrap();
    let codes: Vec<String> = text
        .split("[[commands]]")
        .skip(1)
        .filter(|b| b.contains("interactive = true"))
        .filter_map(|b| {
            b.lines()
                .find_map(|l| l.trim().strip_prefix("id = "))
                .map(|v| v.trim_matches('"').to_owned())
        })
        .collect();
    assert!(codes.len() > 330, "{}", codes.len());
    let mut open_with_scene = Vec::new();
    for code in &codes {
        for (k, selection) in [scene.clone(), Vec::new()].into_iter().enumerate() {
            c.select_objects(&selection).unwrap();
            if c.run_addon_command(code).is_err() {
                continue;
            }
            assert!(
                !c.prompt_text().is_empty() || c.addon_session().is_none(),
                "{code}: no prompt"
            );
            c.refresh_addon_session();
            c.session_hover([1.0, 1.0, 0.0]);
            for p in [[1.0, 1.0, 0.0], [2.0, 0.5, 0.0], [3.0, 3.0, 0.0]] {
                if c.addon_session().is_some() {
                    let _ = c.session_point(p);
                }
            }
            for t in ["1", "", ""] {
                if c.addon_session().is_some() {
                    let _ = c.submit(t);
                }
            }
            if k == 0 && c.addon_session().is_some() {
                open_with_scene.push(format!("{code}: {}", c.prompt_text()));
            }
            c.cancel_addon_session();
            c.cancel_addon_builder();
            while c.coordinator().undo_len() > baseline {
                c.undo().unwrap();
            }
            assert_eq!(
                c.document().entities.len(),
                5,
                "{code}: the scene came back"
            );
        }
    }
    let panics = PANICS.lock().unwrap().clone();
    assert!(
        panics.is_empty(),
        "{} panics:\n{}",
        panics.len(),
        panics.join("\n")
    );
    // Commands that need other picks than this script's stay open (reported, not an error).
    eprintln!(
        "{} of {} commands still open after the script:\n{}",
        open_with_scene.len(),
        codes.len(),
        open_with_scene.join("\n")
    );
    fs::remove_dir_all(&tmp).unwrap();
}

/// A builder command started on a preselected stone (icon click with the stone selected):
/// its panel must offer ✔ at once, because the session already previews the head.
#[test]
fn a_builder_started_on_a_selection_can_be_accepted() {
    let tmp = std::env::temp_dir().join(format!("om9-app-presel-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, c)| FeatureName {
            id,
            command: c.as_deref(),
        })
        .collect();
    let mut c = Controller::new(Document::new("presel"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    make(&mut c, "BuilderGemLoader", &[[6.0, 6.0, 0.0]], &[""]);
    let stone: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&stone).unwrap();
    c.run_addon_command("BuilderHead").unwrap();
    // The UI refreshes every frame; nothing else changes between frames here.
    c.refresh_addon_session();
    c.refresh_builder_preview();
    c.refresh_builder_preview();
    let count = c.addon_builder().map(|b| b.preview.count).unwrap_or(0);
    assert!(count > 0, "✔ must be enabled: {}", c.prompt_text());
    c.accept_addon_builder().unwrap();
    assert!(c.addon_builder().is_none() && c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 2, "stone + head");
    fs::remove_dir_all(&tmp).unwrap();
}
