// SPDX-License-Identifier: LGPL-2.1-or-later
//! Add-on commands through the real controller: install a package, enable it, select a
//! ring rail, run `gvGemOnCrv` typed in CMD and from the menu feature id, and check the
//! gems, their `om9.gem` extension, the single Undo and the availability gate.
//!
//! The runtime here calls the real `gem_on_curve` add-on code in-process (the DLL loader
//! itself is exercised by `addons/gem_on_curve/tests/native_load.rs`).
use om9_addon_abi::sdk::{Addon, CurveData, Output, Request, Value};
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{CommandInput, CommandOutput, LoadRequest, MeshOutput, Runtime};
use om9_addons::settings::SettingValue;
use om9_app::addons::FeatureName;
use om9_app::file_lifecycle::{FileKind, FileOutcome, FilePhase, UnsavedDecision};
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry};
use om9_core::model::{Document, Entity, GEM_EXTENSION_SLOT};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

struct InProcessGemRuntime;

#[derive(Default)]
struct Collect(CommandOutput);
impl Output for Collect {
    fn add_mesh(
        &mut self,
        name: &str,
        v: &[[f64; 3]],
        t: &[[u32; 3]],
        meta: &str,
    ) -> Result<(), String> {
        self.0.meshes.push(MeshOutput {
            name: name.into(),
            vertices: v.to_vec(),
            triangles: t.to_vec(),
            meta_json: meta.into(),
        });
        Ok(())
    }
    fn message(&mut self, text: &str) {
        self.0.messages.push(text.into());
    }
}

impl Runtime for InProcessGemRuntime {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
    fn invoke(&mut self, _id: &str, input: &CommandInput) -> Result<CommandOutput, String> {
        let request = Request {
            command: input.command.clone(),
            curves: input
                .curves
                .iter()
                .map(|c| CurveData {
                    points: c.points.clone(),
                    closed: c.closed,
                })
                .collect(),
            settings: input
                .settings
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            SettingValue::Bool(b) => Value::Bool(*b),
                            SettingValue::Int(i) => Value::Number(*i as f64),
                            SettingValue::Float(f) => Value::Number(*f),
                            SettingValue::Str(s) => Value::Text(s.clone()),
                        },
                    )
                })
                .collect(),
            plane_normal: input.plane_normal,
        };
        let mut out = Collect::default();
        gem_on_curve::GemOnCurve::invoke(&request, &mut out)?;
        Ok(out.0)
    }
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn host(tmp: &Path) -> Arc<Mutex<Host>> {
    let stage = tmp.join("stage");
    fs::create_dir_all(stage.join("icons")).unwrap();
    let src = repo().join("addons/gem_on_curve/addon");
    let manifest = fs::read_to_string(src.join("om9_addon.toml"))
        .unwrap()
        .replace("@PLATFORM@", "any")
        .replace("runtime = \"native\"", "runtime = \"python\"")
        .replace("@ENTRY@", "entry.bin");
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::write(stage.join("entry.bin"), b"in-process test entry").unwrap();
    fs::copy(
        src.join("icons/gem_on_crv.svg"),
        stage.join("icons/gem_on_crv.svg"),
    )
    .unwrap();
    let catalog = om9_app::addons::catalog().unwrap();
    let (_, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    fs::write(tmp.join("gem.zip"), zip).unwrap();
    let mut h = Host::new(HostConfig::new(
        om9_app::addons::om9_version(),
        catalog,
        tmp.join("config"),
    ));
    h.register_runtime(RuntimeKind::Python, Box::new(InProcessGemRuntime));
    h.startup(&Startup::default());
    let r = h
        .install_zip(
            &tmp.join("gem.zip"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    Arc::new(Mutex::new(h))
}

fn ring_document() -> (Document, om9_core::identity::ObjectId) {
    let mut doc = Document::new("ring rail");
    let rail = Entity::new(
        "Ring Rail",
        Geometry::Curve(Curve::Circle {
            center: [0.0, 0.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 17.35 / 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        doc.current_layer,
    );
    let id = rail.id;
    doc.entities.insert(id, rail);
    (doc, id)
}

#[test]
fn gem_on_crv_runs_from_cmd_and_menu_with_one_undo() {
    let tmp = std::env::temp_dir().join(format!("om9-app-addon-{}", std::process::id()));
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

    let (doc, rail) = ring_document();
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    // Without an add-on the Matrix command is visible but unavailable.
    assert!(!c.is_available("OM9-GEM-002"));
    assert!(c.submit("gvGemOnCrv").is_err());

    c.attach_addons(host(&tmp), &names);
    assert!(c.is_available("OM9-GEM-002"), "menu/icon feature id");
    assert!(c.is_available("BuilderGemOnCrv"), "catalog code");
    assert_eq!(
        c.addon_code("gvGemOnCrv").as_deref(),
        Some("BuilderGemOnCrv"),
        "typed Matrix name"
    );
    assert_eq!(
        c.addon_code("gvgemcountoncrv").as_deref(),
        Some("BuilderGemCountOnCrv")
    );
    assert!(
        !c.is_available("OM9-GEM-011"),
        "Gem On Surface stays disabled"
    );

    let gems_in = |c: &Controller| {
        c.document()
            .entities
            .values()
            .filter(|e| e.extension(GEM_EXTENSION_SLOT).is_some())
            .count()
    };
    let before = c.document().entities.len();
    assert_eq!(c.addon_typed_name("BuilderGemOnCrv"), Some("gvGemOnCrv"));

    // Gem On Crv opens its builder instead of running: nothing is created yet.
    c.submit("gvGemOnCrv").unwrap();
    let b = c.addon_builder().expect("builder open");
    assert_eq!(b.code, "BuilderGemOnCrv");
    assert!(
        c.prompt_text().contains("chọn đường cong"),
        "{}",
        c.prompt_text()
    );
    let tokens: Vec<String> = b.tokens().into_iter().map(|(_, t)| t).collect();
    assert_eq!(tokens[0], "Size=1.5", "{tokens:?}");
    assert!(tokens.contains(&"Layout=Dàn đều".to_string()), "{tokens:?}");
    assert!(
        b.preview
            .error
            .as_deref()
            .is_some_and(|e| e.contains("Chọn ít nhất một đường cong"))
    );
    // Enter without a curve: nothing to create, the builder stays open.
    assert!(c.submit("").is_err());
    assert!(c.addon_builder().is_some());
    assert_eq!(c.document().entities.len(), before);

    // Picking the curve while the builder is open previews 32 stones; still no document change.
    c.select_objects(&[rail]).unwrap();
    c.refresh_builder_preview();
    let b = c.addon_builder().unwrap();
    assert_eq!(b.preview.count, 32);
    assert!(!b.preview.edges.is_empty(), "preview outline");
    assert!(
        b.preview
            .messages
            .iter()
            .any(|m| m.contains("32 viên") && m.contains("0.203"))
    );
    assert!(
        c.prompt_text().contains("Enter = đặt 32 viên"),
        "{}",
        c.prompt_text()
    );
    assert_eq!(
        c.document().entities.len(),
        before,
        "a preview is never a document object"
    );

    // Command-line options: Size=2 → 24 stones; a bad value is refused and changes nothing.
    c.submit("Size=2").unwrap();
    c.refresh_builder_preview();
    assert_eq!(c.addon_builder().unwrap().preview.count, 24);
    assert!(c.submit("Size=99").is_err());
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "Size=2");
    // A bare choice option steps to the next choice; a label can be typed too.
    c.submit("Layout").unwrap();
    assert!(
        c.addon_builder()
            .unwrap()
            .tokens()
            .iter()
            .any(|(_, t)| t == "Layout=Giữ khe")
    );
    c.submit("layout=Dàn đều").unwrap();
    assert!(
        c.addon_builder()
            .unwrap()
            .tokens()
            .iter()
            .any(|(_, t)| t == "Layout=Dàn đều")
    );
    // Option names match case-insensitively and by unique prefix.
    c.submit("spa=0,2").unwrap();
    assert!(c.submit("Nope=1").is_err());

    // Enter accepts: one transaction, the builder closes, the values become the defaults.
    let undo_before = c.coordinator().undo_len();
    c.submit("").unwrap();
    assert!(c.addon_builder().is_none());
    assert_eq!(gems_in(&c), 24);
    assert_eq!(
        c.coordinator().undo_len(),
        undo_before + 1,
        "one transaction"
    );
    assert!(
        c.take_addon_messages()
            .iter()
            .any(|m| m.contains("đã đặt 24 viên"))
    );
    let gem = c
        .document()
        .entities
        .values()
        .find(|e| e.extension(GEM_EXTENSION_SLOT).is_some())
        .unwrap();
    assert!(
        gem.extension(GEM_EXTENSION_SLOT)
            .unwrap()
            .payload
            .contains("\"diameter_mm\":2")
    );
    let stored = c
        .addon_host()
        .unwrap()
        .lock()
        .unwrap()
        .settings("gem_on_curve")
        .unwrap();
    assert!(
        stored
            .iter()
            .any(|(s, v)| s.key == "diameter" && v.display() == "2")
    );

    // One Undo removes every gem; the rail stays.
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), before);
    assert!(c.document().entities.contains_key(&rail));

    // From the menu/icon path the same builder opens with the remembered Ø2.0;
    // Esc closes it and creates nothing.
    c.dispatch("OM9-GEM-002", InvocationOrigin::Menu).unwrap();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "Size=2");
    c.cancel();
    assert!(c.addon_builder().is_none());
    assert_eq!(c.document().entities.len(), before);

    // Widget path (builder panel): set a value, reset to defaults.
    c.dispatch("OM9-GEM-002", InvocationOrigin::Menu).unwrap();
    c.set_builder_value("diameter", om9_addons::settings::SettingValue::Float(1.3))
        .unwrap();
    c.refresh_builder_preview();
    assert_eq!(c.addon_builder().unwrap().preview.count, 36, "Ø1.3 → 36");
    assert!(
        c.set_builder_value("diameter", om9_addons::settings::SettingValue::Float(0.0))
            .is_err()
    );
    c.reset_builder_defaults();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "Size=1.5");
    // Typing another command closes the builder and runs that command.
    c.submit("gvGemCountOnCrv").unwrap();
    assert!(c.addon_builder().is_none());
    // Gem Count On Crv is not a builder: it reports at once with the stored values (Ø2.0).
    let msgs = c.take_addon_messages();
    assert!(msgs.iter().any(|m| m.contains("24 viên")), "{msgs:?}");
    assert_eq!(c.document().entities.len(), before);

    // File › Open replaces the document controller; installed add-ons stay attached
    // (regression: the first Windows run reported "unknown command" after Open).
    let saved = tmp.join("ring.om9");
    let (ring, _) = ring_document();
    om9_archive::Archive::save(
        &saved,
        &ring,
        &Limits::default(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
    c.begin_file_operation(FileKind::Open).unwrap();
    if c.file_operation()
        .is_some_and(|op| op.phase() == FilePhase::Unsaved)
    {
        let t = c.file_operation().unwrap().token();
        c.decide_unsaved(t, UnsavedDecision::Discard).unwrap();
    }
    let t = c.file_operation().unwrap().token();
    assert_eq!(
        c.accept_file_path(t, &saved).unwrap(),
        FileOutcome::Replaced
    );
    assert!(c.is_available("OM9-GEM-002"), "add-on survives File › Open");
    let opened_rail = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[opened_rail]).unwrap();
    c.submit("gvGemCountOnCrv").unwrap();
    assert!(
        c.take_addon_messages()
            .iter()
            .any(|m| m.contains("24 viên")),
        "Ø2.0 setting kept"
    );

    // Disabling the add-on turns the command off again.
    c.addon_host()
        .unwrap()
        .lock()
        .unwrap()
        .disable("gem_on_curve")
        .unwrap();
    c.refresh_addon_names(&names);
    assert!(!c.is_available("OM9-GEM-002"));
    fs::remove_dir_all(&tmp).unwrap();
}
