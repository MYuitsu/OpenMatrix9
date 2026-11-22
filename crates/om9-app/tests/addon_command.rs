// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix command and menu slots use the same host-owned services with or
//! without the legacy gem placement add-on installed.
use om9_addon_abi::sdk::{Addon, CurveData, Output, Request, Value};
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{CommandInput, CommandOutput, LoadRequest, MeshOutput, Runtime};
use om9_addons::settings::SettingValue;
use om9_app::addons::FeatureName;
use om9_app::file_lifecycle::{FileKind, FileOutcome, FilePhase, UnsavedDecision};
use om9_app::session::{Controller, InvocationOrigin, gem_placement::GemPlacementTool};
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
fn gem_builders_use_the_host_with_and_without_the_legacy_add_on() {
    let tmp = std::env::temp_dir().join(format!("om9-app-addon-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let feats = features();
    let names: Vec<FeatureName> = feats
        .iter()
        .map(|(id, command)| FeatureName { id, command: command.as_deref() })
        .collect();

    let (doc, rail) = ring_document();
    let mut controller = Controller::new(doc, Limits::default()).unwrap();
    for with_addon in [false, true] {
        if with_addon {
            controller.attach_addons(host(&tmp), &names);
        }
        for (feature, tool) in [
            ("OM9-GEM-002", GemPlacementTool::OnCurve),
            ("OM9-GEM-011", GemPlacementTool::OnSurface),
        ] {
            assert!(controller.is_available(feature), "{feature}");
            controller.dispatch(feature, InvocationOrigin::Menu).unwrap();
            assert_eq!(controller.gem_placement_session().unwrap().tool, tool);
            assert!(controller.addon_builder().is_none());
            controller.cancel();
        }
        for command in ["_gvGemOnCrv", "OM9-GEM-002"] {
            controller.select_objects(&[rail]).unwrap();
            controller.submit(command).unwrap();
            controller.submit("Size=2").unwrap();
            let preview_count = controller.gem_placement_session().unwrap().preview_count();
            assert!(preview_count > 0);
            assert_eq!(controller.document().entities.len(), 1);
            let undo_before = controller.coordinator().undo_len();
            controller.submit("").unwrap();
            let gems = controller.document().entities.values()
                .filter(|entity| entity.extension(GEM_EXTENSION_SLOT).is_some()).count();
            assert_eq!(gems, preview_count);
            assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
            assert!(controller.gem_placement_session().is_none());
            controller.undo().unwrap();
            assert_eq!(controller.document().entities.len(), 1);
        }
    }

    let saved = tmp.join("ring.om9");
    let (ring, _) = ring_document();
    om9_archive::Archive::save(
        &saved,
        &ring,
        &Limits::default(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
    controller.begin_file_operation(FileKind::Open).unwrap();
    let op = controller.file_operation().unwrap();
    let token = op.token();
    if op.phase() == FilePhase::Unsaved {
        controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
    }
    assert_eq!(controller.accept_file_path(token, &saved).unwrap(), FileOutcome::Replaced);
    assert!(controller.addon_host().is_some(), "add-on survives File > Open");
    controller.addon_host().unwrap().lock().unwrap().disable("gem_on_curve").unwrap();
    controller.refresh_addon_names(&names);
    assert!(controller.is_available("OM9-GEM-002"));
    assert!(controller.is_available("OM9-GEM-011"));
    fs::remove_dir_all(&tmp).unwrap();
}
