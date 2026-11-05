// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive add-on commands through the real controller: start from CMD, menu code and
//! typed alias; command-line sub-options; clicked and typed points; preview without document
//! objects; one transaction on finish; Esc cancels; another command closes the session.
use om9_addon_abi::session::session_json;
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{LoadRequest, Runtime};
use om9_app::addons::FeatureName;
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry};
use om9_core::model::Document;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Runs the real add-on package code in-process through the same JSON boundary as the DLL.
struct InProcess;
impl Runtime for InProcess {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn session(&mut self, _id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        let (code, out) = session_json::<om9_matrix::Matrix>(event);
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

pub fn features() -> Vec<(String, Option<String>)> {
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

fn host(tmp: &Path) -> Arc<Mutex<Host>> {
    host_with(tmp, &[])
}

/// A host with the OM9 Matrix package installed and enabled (all menus in one add-on), in
/// English: these tests check the command texts as written.
fn host_with(tmp: &Path, _menus: &[&str]) -> Arc<Mutex<Host>> {
    host_lang(tmp, "en")
}

/// The same host in the given interface language (`vi` is the package default).
fn host_lang(tmp: &Path, lang: &str) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(
        om9_app::addons::om9_version(),
        catalog.clone(),
        tmp.join("config"),
    ));
    h.register_runtime(RuntimeKind::Python, Box::new(InProcess));
    h.startup(&Startup::default());
    let zip = stage_zip(tmp, &catalog);
    let r = h
        .install_zip(
            &zip,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    h.set_setting("om9_matrix", "language", lang).unwrap();
    Arc::new(Mutex::new(h))
}

fn stage_zip(tmp: &Path, catalog: &om9_addons::catalog::Catalog) -> PathBuf {
    let stage = tmp.join("stage-matrix");
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
    let (_, zip) = build_package(&stage, Some(catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    let path = tmp.join("matrix.zip");
    fs::write(&path, zip).unwrap();
    path
}


/// Benchmark (ignored): built-in vs add-on Line/Circle, from the command to its commit, in an
/// empty and a large document. `cargo test --release -p om9-app --test addon_bench -- --ignored --nocapture`
fn controller(n: usize, addons: Option<&Arc<Mutex<Host>>>) -> Controller {
    // A large file: `n` lines already in the document when it opens.
    let mut doc = Document::new("bench");
    let layer = doc.current_layer;
    for i in 0..n {
        let y = i as f64 * 0.01;
        let line = Curve::Line { start: [0.0, y, 0.0], end: [3.0, y, 0.0], domain: [0.0, 3.0] };
        let e = om9_core::model::Entity::new("Line", Geometry::Curve(line), layer);
        doc.entities.insert(e.id, e);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    assert_eq!(c.document().entities.len(), n);
    if let Some(tmp) = addons {
        let feats = features();
        let names: Vec<FeatureName> = feats
            .iter()
            .map(|(id, cmd)| FeatureName { id, command: cmd.as_deref() })
            .collect();
        c.attach_addons(tmp.clone(), &names);
    }
    c
}

fn time(c: &mut Controller, steps: &[&str], addon: Option<&str>) -> f64 {
    let before = c.document().entities.len();
    let undo = c.coordinator().undo_len();
    let t0 = std::time::Instant::now();
    match addon {
        Some(code) => c.run_addon_command(code).unwrap(),
        None => { c.submit(steps[0]).unwrap(); }
    }
    for s in &steps[1..] {
        // "@x,y,z" is a viewport click: the built-in Circle takes its centre by click.
        match s.strip_prefix('@') {
            Some(p) if addon.is_none() => {
                let v: Vec<f64> = p.split(',').map(|x| x.parse().unwrap()).collect();
                c.accept_circle_point([v[0], v[1], v[2]]).unwrap();
            }
            Some(p) => {
                let v: Vec<f64> = p.split(',').map(|x| x.parse().unwrap()).collect();
                c.session_point([v[0], v[1], v[2]]).unwrap();
            }
            None => {
                c.submit(s).unwrap();
            }
        }
    }
    let dt = t0.elapsed().as_secs_f64() * 1e3;
    assert_eq!(c.document().entities.len(), before + 1, "one object");
    assert_eq!(c.coordinator().undo_len(), undo + 1, "one Undo entry");
    c.undo().unwrap();
    dt
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

#[test]
#[ignore]
fn bench_builtin_vs_addon() {
    let tmp = std::env::temp_dir().join(format!("om9-bench-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let samples: usize = std::env::var("BENCH_N").ok().and_then(|v| v.parse().ok()).unwrap_or(40);
    let shared = host(&tmp);
    for n in [0usize, 5000, 20000] {
        let mut b = controller(n, None);
        let mut a = controller(n, Some(&shared));
        for (name, steps, asteps, code) in [
            ("Line 10 mm", vec![om9_app::session::LINE, "0,0,0", "10,0,0"], vec!["", "0,0,0", "10,0,0"], om9_curves::LINE),
            ("Circle r5", vec!["OM9-CURVE-005", "@0,0,0", "radius=5"], vec!["", "@0,0,0", "5"], om9_curves::CIRCLE),
        ] {
            let (mut tb, mut ta) = (Vec::new(), Vec::new());
            for _ in 0..samples {
                tb.push(time(&mut b, &steps, None));
                ta.push(time(&mut a, &asteps, Some(code)));
            }
            let (mb, ma) = (median(tb), median(ta));
            eprintln!("BENCH doc={n:>5} {name:<11} built-in {mb:>8.3} ms  add-on {ma:>8.3} ms  x{:.1}", ma / mb);
        }
        // Pointer moves during an add-on Line (after its start point).
        a.run_addon_command(om9_curves::LINE).unwrap();
        a.submit("0,0,0").unwrap();
        let t0 = std::time::Instant::now();
        for i in 0..200 {
            a.session_hover([5.0, i as f64 * 0.01, 0.0]);
        }
        eprintln!("BENCH doc={n:>5} add-on hover  {:>8.3} ms per move", t0.elapsed().as_secs_f64() * 1e3 / 200.0);
        a.cancel_addon_session();
    }
}

/// With the Matrix package (453 settings), the per-command settings asked at every step are
/// exactly the full table filtered to that command.
#[test]
fn matrix_command_settings_match_the_full_table() {
    let tmp = std::env::temp_dir().join(format!("om9-cmdset-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let shared = host(&tmp);
    let h = shared.lock().unwrap();
    let full = h.settings("om9_matrix").unwrap();
    assert!(full.len() > 100, "{}", full.len());
    let mut codes: Vec<String> = full.iter().flat_map(|(s, _)| s.commands.clone()).collect();
    codes.extend([om9_curves::LINE.to_owned(), om9_curves::CIRCLE.to_owned()]);
    codes.sort();
    codes.dedup();
    for code in &codes {
        let want: Vec<_> = full
            .iter()
            .filter(|(s, _)| s.applies_to(code))
            .map(|(s, v)| (s.key.clone(), v.clone()))
            .collect();
        assert_eq!(h.command_settings("om9_matrix", code), want, "{code}");
        if let Some(info) = h.command_info(code) {
            let got: Vec<_> = info.settings.into_iter().map(|(s, v)| (s.key, v)).collect();
            assert_eq!(got, want, "{code}");
        }
    }
}

/// An add-on command that makes one object goes into the scene the incremental way (as OM9's
/// own commands do) in a document that already has objects: it is drawn, one Undo removes it
/// from the document and the scene, Redo brings it back.
#[test]
fn addon_single_object_is_published_and_undone() {
    let tmp = std::env::temp_dir().join(format!("om9-incr-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let shared = host(&tmp);
    let mut c = controller(200, Some(&shared));
    let roots = |c: &Controller| -> std::collections::BTreeSet<_> {
        c.scene().unwrap().objects.iter().map(|o| o.root).collect()
    };
    assert_eq!(roots(&c).len(), 200);
    c.run_addon_command(om9_curves::CIRCLE).unwrap();
    c.session_point([1.0, 2.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    assert_eq!(c.document().entities.len(), 201);
    let after = roots(&c);
    assert_eq!(after.len(), 201, "the new circle is in the scene");
    assert!(c.document().entities.keys().all(|id| after.contains(id)));
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 200);
    assert_eq!(roots(&c).len(), 200, "Undo takes it out of the scene");
    c.redo().unwrap();
    assert_eq!(roots(&c).len(), 201, "Redo puts it back");
    // A second add-on object on top, still one transaction each.
    let undo = c.coordinator().undo_len();
    c.run_addon_command(om9_curves::LINE).unwrap();
    c.submit("0,0,0").unwrap();
    c.submit("10,0,0").unwrap();
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    assert_eq!(roots(&c).len(), 202);
}
