// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive add-on commands through the real controller: start from CMD, menu code and
//! typed alias; command-line sub-options; clicked and typed points; preview without document
//! objects; one transaction on finish; Esc cancels; another command closes the session.
use om9_addon_abi::session::{Interactive, session_json};
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

#[test]
fn curve_commands_run_interactively() {
    assert_eq!(
        <om9_curves::Curves as Interactive>::SESSION_COMMANDS.len(),
        13
    );
    let tmp = std::env::temp_dir().join(format!("om9-app-session-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("curves"), Limits::default()).unwrap();
    assert!(!c.is_available("CurveCircleCenterRadius"));
    c.attach_addons(host(&tmp), &names);
    for code in om9_curves::Curves::SESSION_COMMANDS {
        assert!(c.is_available(code), "{code} is enabled from its menu icon");
    }
    assert_eq!(
        c.addon_code("rectangle").as_deref(),
        Some("CurveRectangleCornertoCorner")
    );
    assert_eq!(
        c.addon_code("Polyline").as_deref(),
        Some("CurvePolylinePolyline")
    );

    // With the package enabled, its Polyline, Circle … take over OM9's own icon and name.
    assert!(
        c.icon_target("Polyline", None)
            .is_some_and(|t| t.addon && t.command == "CurvePolylinePolyline")
    );
    // Polyline: prompt + sub-options, clicks, a typed relative point, Enter.
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    let s = c.addon_session().expect("session open");
    assert!(
        c.prompt_text().starts_with("Start of polyline"),
        "{}",
        c.prompt_text()
    );
    assert!(
        s.tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "PersistentClose=No")
    );
    assert!(c.session_wants_point());
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.session_hover([10.0, 5.0, 0.0]);
    assert!(
        !c.addon_session().unwrap().hover_lines.is_empty(),
        "rubber segment preview"
    );
    assert!(
        c.document().entities.is_empty(),
        "preview is never a document object"
    );
    c.submit("@0,5").unwrap(); // relative to the last point
    c.submit("Undo").unwrap();
    c.submit("10,5").unwrap(); // absolute CPlane coordinates
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(n, _)| n == "Close")
    );
    let undo = c.coordinator().undo_len();
    c.submit("Close").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), undo + 1, "one transaction");
    let e = c.document().entities.values().next().unwrap();
    let Geometry::Curve(Curve::Nurbs(n)) = &e.geometry else {
        panic!("{:?}", e.geometry)
    };
    assert_eq!((n.degree, n.poles.len()), (1, 4), "closed: start repeated");
    assert_eq!(n.poles[2], [10.0, 5.0, 0.0]);
    assert!(c.take_addon_messages().iter().any(|m| m.contains("closed")));

    // Circle from its menu icon: the Diameter sub-option (as in Matrix), typed diameter.
    c.dispatch("CurveCircleCenterRadius", InvocationOrigin::Menu)
        .unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Diameter").unwrap();
    c.submit("17.35").unwrap();
    let circle = c
        .document()
        .entities
        .values()
        .find_map(|e| match &e.geometry {
            Geometry::Curve(Curve::Circle { radius, .. }) => Some(*radius),
            _ => None,
        })
        .unwrap();
    assert!((circle - 8.675).abs() < 1e-12);

    // Esc cancels; nothing is created. Another command closes the session and runs.
    let before = c.document().entities.len();
    c.run_addon_command("CurveRectangleCornertoCorner").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.cancel();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before);
    c.run_addon_command("CurveRectangleCornertoCorner").unwrap();
    c.run_addon_command("CurveCircleCenterRadius").unwrap();
    assert_eq!(c.addon_session().unwrap().code, "CurveCircleCenterRadius");
    c.submit("Circle").unwrap();
    assert_eq!(
        c.addon_session().map(|s| s.code.as_str()),
        Some("CurveCircleCenterRadius"),
        "the typed name runs the add-on's Circle"
    );
    c.cancel();
    // A refused value keeps the session open with the reason.
    c.run_addon_command("CurvePolygonCenterRadius").unwrap();
    let err = c.submit("NumSides=2").unwrap_err().to_string();
    assert!(err.contains("NumSides"), "{err}");
    assert!(c.addon_session().is_some());
    c.submit("NumSides=6").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("3").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before + 1);
    // Undo removes the polygon.
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), before);
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn transform_commands_act_on_the_selection() {
    let tmp = std::env::temp_dir().join(format!("om9-app-transform-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("transform"), Limits::default()).unwrap();
    c.attach_addons(host_with(&tmp, &["curves", "transform"]), &names);
    assert!(c.is_available("OM9-TOP11-006"), "Top 11 Move icon");
    assert_eq!(c.addon_code("move").as_deref(), Some("TopIconMove"));
    // A circle to work on.
    c.run_addon_command("CurveCircleCenterRadius").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("2").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let center = |c: &Controller, id| {
        let e = c.document().entities.get(&id).unwrap();
        e.placement.point([0.0; 3]).unwrap()
    };

    // Move with nothing selected asks for objects first; selecting advances the prompt.
    c.select_objects(&[]).unwrap();
    c.run_addon_command("TopIconMove").unwrap();
    assert!(
        c.prompt_text().starts_with("Select objects to move"),
        "{}",
        c.prompt_text()
    );
    c.select_objects(&[id]).unwrap();
    c.refresh_addon_session();
    assert!(
        c.prompt_text().starts_with("Point to move from"),
        "{}",
        c.prompt_text()
    );
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_hover([5.0, 0.0, 0.0]);
    assert!(
        !c.addon_session().unwrap().hover_lines.is_empty(),
        "moved selection previewed"
    );
    let undo = c.coordinator().undo_len();
    c.session_point([10.0, 5.0, 0.0]).unwrap();
    assert_eq!(center(&c, id), [10.0, 5.0, 0.0]);
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    assert_eq!(c.document().entities.len(), 1, "Move does not copy");

    // Polar array: 6 items → 5 copies, one transaction, one Undo.
    c.run_addon_command("TransformArrayPolar").unwrap();
    c.submit("0,0").unwrap();
    c.submit("6").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);

    // Mirror keeps the original (Copy=Yes) across the CPlane y axis.
    c.run_addon_command("TopIconMirror").unwrap();
    c.submit("YAxis").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let mirrored = c
        .document()
        .entities
        .values()
        .map(|e| e.placement.point([0.0; 3]).unwrap())
        .find(|p| p[0] < 0.0)
        .unwrap();
    assert!((mirrored[0] + 10.0).abs() < 1e-9 && (mirrored[1] - 5.0).abs() < 1e-9);

    // Scale 3D with a typed factor; Rotate with a typed angle.
    c.run_addon_command("TransformScaleScale3D").unwrap();
    c.submit("10,5").unwrap();
    c.submit("2").unwrap();
    let e = c.document().entities.get(&id).unwrap();
    assert!((e.placement.0[0][0] - 2.0).abs() < 1e-12);
    c.run_addon_command("TopIconRotate").unwrap();
    c.submit("10,5").unwrap();
    c.submit("90").unwrap();
    assert!(c.addon_session().is_none());
    fs::remove_dir_all(&tmp).unwrap();
}

#[test]
fn gem_cutter_uses_its_panel_and_command_line() {
    use om9_core::geometry::Mesh;
    use om9_core::model::{Entity, Extension, GEM_EXTENSION_SLOT};
    let tmp = std::env::temp_dir().join(format!("om9-app-cutter-{}", std::process::id()));
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
    let mut doc = Document::new("cutter");
    // A Ø2 gem at (5,0,0), then moved +1 in z by its placement: the cutter must follow.
    let mut gem = Entity::new(
        "Gem 2.00 mm",
        Geometry::Mesh(Mesh {
            vertices: vec![
                [4.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
                [5.0, 0.0, -1.0],
            ],
            faces: vec![vec![0, 1, 2], vec![0, 3, 1], vec![1, 3, 2], vec![2, 3, 0]],
            normals: vec![],
            colors: vec![],
            corner_uvs: Vec::new(),
        }),
        doc.current_layer,
    );
    gem.placement = om9_core::geometry::Transform::translation([0.0, 0.0, 1.0]);
    gem.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(
            1,
            r#"{"cut":"round","diameter_mm":2.0,"center":[5,0,0],"axis":[0,0,1],"x":[1,0,0]}"#,
        )
        .unwrap(),
    );
    let gid = gem.id;
    doc.entities.insert(gid, gem);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.attach_addons(host_with(&tmp, &["gem_cutter"]), &names);
    assert_eq!(
        c.addon_code("gvAzureCutter").as_deref(),
        Some("BuilderGemCutter")
    );
    c.submit("gvAzureCutter").unwrap();
    assert!(
        c.prompt_text().starts_with("Select gems"),
        "{}",
        c.prompt_text()
    );
    assert!(
        c.addon_builder().is_some_and(|b| b.interactive),
        "panel opens with the command"
    );
    c.select_objects(&[gid]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("1 cutters"), "{}", c.prompt_text());
    assert!(
        !c.addon_session().unwrap().lines.is_empty(),
        "cutter preview"
    );
    // Command-line option sets the panel value; the preview follows.
    c.submit("GirdleScale=120").unwrap();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "GirdleScale=120");
    let before = c.document().entities.len();
    c.submit("").unwrap();
    assert!(
        c.addon_session().is_none() && c.addon_builder().is_none(),
        "{} / {:?}",
        c.prompt_text(),
        c.take_addon_messages()
    );
    assert_eq!(c.document().entities.len(), before + 1);
    let cutter = c
        .document()
        .entities
        .values()
        .find(|e| e.label.starts_with("Gem Cutter"))
        .unwrap();
    assert!(
        cutter.extension(GEM_EXTENSION_SLOT).is_none(),
        "a cutter is not a gem"
    );
    let Geometry::Mesh(m) = &cutter.geometry else {
        panic!()
    };
    let top = m.vertices.iter().map(|v| v[2]).fold(f64::MIN, f64::max);
    assert!(
        (top - 2.0).abs() < 1e-9,
        "top 50 % of Ø2 above the moved girdle at z=1: {top}"
    );
    let widest = m
        .vertices
        .iter()
        .map(|v| ((v[0] - 5.0).powi(2) + v[1] * v[1]).sqrt())
        .fold(0.0, f64::max);
    assert!(
        (widest - 1.2).abs() < 1e-9,
        "GirdleScale 120 % of Ø2: {widest}"
    );
    // The values accepted become the panel defaults next time.
    c.submit("gvAzureCutter").unwrap();
    assert_eq!(c.addon_builder().unwrap().tokens()[0].1, "GirdleScale=120");
    c.cancel();
    fs::remove_dir_all(&tmp).unwrap();
}

/// Every command the bundled add-ons provide has an OM9 icon whose catalog code calls it
/// once the add-on is installed (menu palettes, toolbars and icon history share this lookup).
#[test]
fn every_addon_command_has_an_icon_that_calls_it() {
    let tmp = std::env::temp_dir().join(format!("om9-app-icons-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("icons"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let text = fs::read_to_string(repo().join("addons/matrix/addon/om9_addon.toml")).unwrap();
    let codes: Vec<&str> = text
        .split("[[commands]]")
        .skip(1)
        .filter_map(|block| block.lines().find_map(|l| l.trim().strip_prefix("id = ")))
        .map(|v| v.trim_matches('"'))
        .collect();
    assert_eq!(codes.len(), om9_matrix::ALL.len());
    // OM9 draws a few commands with the same icon name (Ring Rail on the Top 11 bar and in
    // Tools): that icon runs one of them; both are the same Matrix command.
    let mut by_icon: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for code in &codes {
        if let Some(icon) = om9_app::icons::icon_for_command(code) {
            by_icon.entry(icon).or_default().push(code);
        }
    }
    for code in codes {
        let icon = om9_app::icons::icon_for_command(code)
            .unwrap_or_else(|| panic!("no OM9 icon for {code}"));
        let target = c
            .icon_target(icon, None)
            .unwrap_or_else(|| panic!("{icon}: no command"));
        assert!(target.enabled && target.addon, "{icon} → {target:?}");
        assert!(by_icon[icon].contains(&target.command.as_str()), "{icon}: {} is not one of {:?}", target.command, by_icon[icon]);
        if by_icon[icon].len() == 1 {
            assert_eq!(target.command, code, "{icon}");
        }
        assert!(c.is_available(code), "{code}");
    }
    fs::remove_dir_all(&tmp).unwrap();
}

fn gem_record(c: &Controller, id: om9_core::identity::ObjectId) -> serde_json::Value {
    let e = c.document().entities.get(&id).unwrap();
    serde_json::from_str(&e.extension(om9_core::model::GEM_EXTENSION_SLOT).unwrap().payload).unwrap()
}

/// Gem Loader → Halo → Gem Reporter (material) → replace by another cut → Gem Update →
/// prongs, each through the controller as from the icons, one Undo step each.
#[test]
fn jewelry_flow_through_the_controller() {
    let tmp = std::env::temp_dir().join(format!("om9-app-jewelry-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("ring"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);

    // Gem Loader: the panel opens with the command; the size is typed on the command line.
    c.run_addon_command("BuilderGemLoader").unwrap();
    assert!(c.addon_builder().is_some_and(|b| b.interactive));
    c.submit("Length=6.5").unwrap();
    assert!(c.prompt_text().contains("Round 6.50 mm"), "{}", c.prompt_text());
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    let undo = c.coordinator().undo_len();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none() && c.addon_builder().is_none());
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    let center = *c.document().entities.keys().next().unwrap();
    assert_eq!(gem_record(&c, center)["diameter_mm"], 6.5);

    // Halo around it.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderHalo").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("22 accent stones"), "{}", c.prompt_text());
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 1 + 22 + 1);

    // Gem Reporter assigns ruby to the center stone.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderGemReporter").unwrap();
    c.submit("Material=ruby").unwrap();
    c.submit("").unwrap();
    assert_eq!(gem_record(&c, center)["material"], "ruby");
    assert!(c.take_addon_messages().iter().any(|m| m.contains("ruby")));

    // The loader replaces the selected stone by an oval in the same place.
    c.select_objects(&[center]).unwrap();
    c.run_addon_command("BuilderGemLoader").unwrap();
    c.submit("Cut=oval").unwrap();
    c.submit("Length=8").unwrap();
    c.submit("").unwrap();
    assert!(!c.document().entities.contains_key(&center), "old stone replaced");
    let oval = c
        .document()
        .entities
        .iter()
        .find(|(_, e)| e.label.starts_with("Gem Oval"))
        .map(|(id, _)| *id)
        .unwrap();
    assert_eq!(gem_record(&c, oval)["center"], serde_json::json!([0.0, 0.0, 0.0]));
    assert_eq!(c.document().entities.len(), 1 + 22 + 1);

    // Gem Update rebuilds the stone in place (same object).
    c.select_objects(&[oval]).unwrap();
    c.run_addon_command("BuilderGemUpdate").unwrap();
    c.submit("").unwrap();
    assert!(c.document().entities.contains_key(&oval));

    // Four prongs on the oval.
    c.select_objects(&[oval]).unwrap();
    c.run_addon_command("BuilderProngAdder").unwrap();
    c.refresh_addon_session();
    c.submit("").unwrap();
    let prongs = c
        .document()
        .entities
        .values()
        .filter(|e| e.label == "Prong")
        .count();
    assert_eq!(prongs, 4);
    // One Undo removes the prongs only.
    c.undo().unwrap();
    assert_eq!(
        c.document().entities.values().filter(|e| e.label == "Prong").count(),
        0
    );
    fs::remove_dir_all(&tmp).unwrap();
}

/// A stone remembers the prongs made on it; Match Attributes makes the same prongs on another
/// stone (two selection groups through the controller).
#[test]
fn styles_follow_stones_through_the_controller() {
    let tmp = std::env::temp_dir().join(format!("om9-app-styles-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("styles"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("BuilderGemLoader").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([20.0, 0.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let mut stones: Vec<_> = c.document().entities.keys().copied().collect();
    stones.sort_by(|a, b| {
        gem_record(&c, *a)["center"][0]
            .as_f64()
            .unwrap()
            .total_cmp(&gem_record(&c, *b)["center"][0].as_f64().unwrap())
    });
    let (a, b) = (stones[0], stones[1]);
    // Six prongs on stone a: a remembers them.
    c.select_objects(&[a]).unwrap();
    c.run_addon_command("BuilderProngAdder").unwrap();
    c.submit("Count=6").unwrap();
    c.submit("").unwrap();
    let prongs = |c: &Controller| c.document().entities.values().filter(|e| e.label == "Prong").count();
    assert_eq!(prongs(&c), 6);
    let style = &gem_record(&c, a)["styles"]["BuilderProngAdder"];
    assert_eq!(style["prong_count"], 6, "{style}");
    assert!(style.get("halo_size").is_none(), "only the prong builder's own values: {style}");
    // Match Attributes: first the stone that gets the style, then the stone that has it.
    c.select_objects(&[b]).unwrap();
    c.run_addon_command("OthersMatchAttributes").unwrap();
    c.submit("").unwrap();
    c.select_objects(&[a]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("BuilderProngAdder"), "{}", c.prompt_text());
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(prongs(&c), 12);
    assert_eq!(gem_record(&c, b)["styles"]["BuilderProngAdder"]["prong_count"], 6);
    fs::remove_dir_all(&tmp).unwrap();
}

/// Surface commands make NURBS surfaces in the document, and later commands see them (their
/// exact definition and their tessellation).
#[test]
fn surfaces_round_trip_through_the_document() {
    let tmp = std::env::temp_dir().join(format!("om9-app-surfaces-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("surfaces"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("SurfacePlaneCornertoCorner").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([4.0, 3.0, 0.0]).unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let plane = *c.document().entities.keys().next().unwrap();
    assert!(matches!(
        c.document().entities[&plane].geometry,
        Geometry::Surface(_)
    ));
    // Offset it 2 mm: the command reads the exact surface back.
    c.select_objects(&[plane]).unwrap();
    c.run_addon_command("SurfaceOffsetSurface").unwrap();
    c.submit("Distance=2").unwrap();
    c.submit("").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(c.document().entities.len(), 2);
    // Extrude Normal to Surface needs the plane's tessellation (mesh_data).
    c.select_objects(&[]).unwrap();
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    c.session_point([1.0, 1.0, 0.0]).unwrap();
    c.session_point([3.0, 1.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let curve = *c
        .document()
        .entities
        .keys()
        .find(|id| matches!(c.document().entities[*id].geometry, Geometry::Curve(_)))
        .unwrap();
    c.select_objects(&[curve, plane]).unwrap();
    c.run_addon_command("SurfaceExtrudeCurveNormalToSurface").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("1 ribbons"), "{}", c.prompt_text());
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    fs::remove_dir_all(&tmp).unwrap();
}

/// SubD: a cage from the add-on keeps its faces through the document, so a face selection
/// made by one command is used by the next (Paint Selection → Extrude → Subdivision Level).
#[test]
fn subd_face_selection_survives_the_document() {
    let tmp = std::env::temp_dir().join(format!("om9-app-subd-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("subd"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    c.run_addon_command("ClayooPrimitivesBox").unwrap();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([4.0, 4.0, 0.0]).unwrap();
    c.submit("4").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let cube = *c.document().entities.keys().next().unwrap();
    let tris = |c: &Controller| match &c.document().entities[&cube].geometry {
        Geometry::Mesh(m) => m.faces.len(),
        g => panic!("{g:?}"),
    };
    assert_eq!(tris(&c), 6 * 4 * 2, "a 2 × 2 cage on every side");
    // Paint one top face, extrude it.
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooSelectionPaintSelection").unwrap();
    c.session_point([1.0, 1.0, 0.0]).unwrap();
    assert!(c.prompt_text().contains("1 faces selected"), "{}", c.prompt_text());
    c.submit("").unwrap();
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooEditExtrude").unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().contains("Distance"), "the selection is used: {}", c.prompt_text());
    c.submit("2").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(tris(&c), (24 + 4) * 2, "four side walls");
    // Smooth it.
    c.select_objects(&[cube]).unwrap();
    c.run_addon_command("ClayooEditSubdivisionLevel").unwrap();
    c.submit("2").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    assert_eq!(tris(&c), 28 * 16 * 2);
    fs::remove_dir_all(&tmp).unwrap();
}

/// A deformation (Taper) rebuilds a moved mesh in place: the object keeps its id and
/// placement, its world shape follows the deformation.
#[test]
fn deformations_rebuild_objects_in_place() {
    let tmp = std::env::temp_dir().join(format!("om9-app-deform-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("deform"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    // A 10 × 2 × 2 box (Cube Cutter) at the origin, then moved +5 in x.
    c.run_addon_command("BuilderPlaneandCubeCutters").unwrap();
    c.submit("X=10").unwrap();
    c.submit("Y=2").unwrap();
    c.submit("Z=2").unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("TopIconMove").unwrap();
    c.refresh_addon_session();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([5.0, 0.0, 0.0]).unwrap();
    let placement = c.document().entities[&id].placement;
    // Taper along x from 0 to 10 (world): the far end shrinks to half.
    c.select_objects(&[id]).unwrap();
    c.run_addon_command("TransformTaper").unwrap();
    c.refresh_addon_session();
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("1").unwrap();
    c.submit("0.5").unwrap();
    assert!(c.addon_session().is_none(), "{}", c.prompt_text());
    let e = &c.document().entities[&id];
    assert_eq!(e.placement, placement, "the object keeps its placement");
    let Geometry::Mesh(m) = &e.geometry else { panic!() };
    let world: Vec<[f64; 3]> = m.vertices.iter().map(|v| e.placement.point(*v).unwrap()).collect();
    let at_end = world.iter().filter(|v| (v[0] - 10.0).abs() < 1e-6).map(|v| v[1].abs()).fold(0.0, f64::max);
    assert!((at_end - 0.5).abs() < 1e-6, "half width at x = 10: {at_end}");
    fs::remove_dir_all(&tmp).unwrap();
}

/// Vietnamese (the package default): menu names, command names, sub-options and prompts are
/// shown in Vietnamese; English and Vietnamese option names both work when typed.
#[test]
fn vietnamese_interface_keeps_english_typing() {
    let tmp = std::env::temp_dir().join(format!("om9-app-session-vi-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("vi"), Limits::default()).unwrap();
    assert_eq!(c.tr().menu("Curve"), "Curve", "English without add-ons");
    let host = host_lang(&tmp, "vi");
    c.attach_addons(host.clone(), &names);
    assert_eq!(c.tr().menu("Curve"), "Đường cong");
    assert_eq!(
        c.tr().command("CurveLineSingleLine", "Single Line"),
        "Đoạn thẳng"
    );

    c.run_addon_command("CurvePolylinePolyline").unwrap();
    assert!(
        c.prompt_text().starts_with("Điểm đầu đường gấp khúc"),
        "{}",
        c.prompt_text()
    );
    let tokens = c.addon_session().unwrap().tokens(c.tr());
    assert!(
        tokens
            .iter()
            .any(|(n, t)| n == "PersistentClose" && t == "Luôn khép kín=Không"),
        "{tokens:?}"
    );
    // Typed in English, in Vietnamese with accents and without.
    c.submit("PersistentClose=Yes").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Có")
    );
    c.submit("luon khep kin=khong").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Không")
    );
    c.submit("Luôn khép kín=Có").unwrap();
    assert!(
        c.addon_session()
            .unwrap()
            .tokens(c.tr())
            .iter()
            .any(|(_, t)| t == "Luôn khép kín=Có")
    );
    c.session_point([0.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 0.0, 0.0]).unwrap();
    c.session_point([10.0, 10.0, 0.0]).unwrap();
    assert!(
        c.prompt_text()
            .starts_with("Điểm tiếp theo của đường gấp khúc"),
        "{}",
        c.prompt_text()
    );
    c.submit("hoan tac").unwrap(); // Undo by its Vietnamese name
    c.submit("").unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), 1);

    // English on request: every text as written.
    host.lock()
        .unwrap()
        .set_setting("om9_matrix", "language", "en")
        .unwrap();
    c.refresh_addon_names(&names);
    assert_eq!(c.tr().menu("Curve"), "Curve");
    c.run_addon_command("CurvePolylinePolyline").unwrap();
    assert!(
        c.prompt_text().starts_with("Start of polyline"),
        "{}",
        c.prompt_text()
    );
    c.cancel_addon_session();
}

#[test]
fn curves_are_picked_by_clicking_and_sketch_follows_the_pointer() {
    let tmp = std::env::temp_dir().join(format!("om9-app-pick-{}", std::process::id()));
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
    let mut c = Controller::new(Document::new("pick"), Limits::default()).unwrap();
    c.attach_addons(host(&tmp), &names);
    let line = |c: &mut Controller, a: [f64; 3], b: [f64; 3]| {
        let before: Vec<_> = c.document().entities.keys().copied().collect();
        c.run_addon_command("CurvePolylinePolyline").unwrap();
        c.session_point(a).unwrap();
        c.session_point(b).unwrap();
        c.submit("").unwrap();
        *c.document().entities.keys().find(|k| !before.contains(k)).unwrap()
    };
    let a = line(&mut c, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let b = line(&mut c, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    assert_ne!(a, b);
    // Fillet: click the first line on +x, the second on +y.
    c.clear_selection();
    c.run_addon_command("CurveFilletCurves").unwrap();
    assert!(c.prompt_text().starts_with("Select first curve to fillet"), "{}", c.prompt_text());
    c.note_session_pick(Some([5.0, 0.0, 0.0]));
    c.select_objects(&[a]).unwrap();
    c.refresh_addon_session();
    assert!(c.prompt_text().starts_with("Select second curve to fillet"), "{}", c.prompt_text());
    c.note_session_pick(Some([0.0, 6.0, 0.0]));
    c.select_objects(&[a, b]).unwrap();
    c.refresh_addon_session();
    assert!(c.addon_session().is_none(), "the fillet is made on the second pick");
    assert_eq!(c.document().entities.len(), 3, "two trimmed lines and the arc");
    // The arc (an OM9 circle with an angle domain): radius 1 around (1, 1).
    let arc = c.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Circle { center, radius, domain, .. }) => Some((*center, *radius, *domain)),
        _ => None,
    });
    let (center, radius, domain) = arc.expect("a fillet arc");
    assert!((center[0] - 1.0).abs() < 1e-6 && (center[1] - 1.0).abs() < 1e-6 && (radius - 1.0).abs() < 1e-9, "{center:?} {radius}");
    assert!((domain[1] - domain[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-6, "a quarter: {domain:?}");
    // Sketch: click, move, click.
    let before = c.document().entities.len();
    c.clear_selection();
    c.run_addon_command("CurveFreeFormSketch").unwrap();
    c.session_point([0.0, 20.0, 0.0]).unwrap();
    for i in 1..60 {
        let x = i as f64 * 0.1;
        c.session_hover([x, 20.0 + (x * 0.8).sin(), 0.0]);
    }
    c.session_point([6.0, 20.0 + (4.8f64).sin(), 0.0]).unwrap();
    assert!(c.addon_session().is_none());
    assert_eq!(c.document().entities.len(), before + 1, "the stroke became a curve");
    fs::remove_dir_all(&tmp).unwrap();
}
