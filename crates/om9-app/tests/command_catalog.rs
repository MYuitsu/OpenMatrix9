// SPDX-License-Identifier: LGPL-2.1-or-later
//! `docs/matrix9-commands.md` lists every command of the Matrix-style menus with its menu
//! position, icon, name and what runs it today. It is a checked artifact: the commands come
//! from `Resources/menu/MainMenu.ini` and the status from the same resolution an icon click
//! uses (`Controller::icon_target`) with the bundled add-ons installed.
//!
//! Regenerate after adding commands:
//! `OM9_UPDATE_CATALOG=1 cargo test -p om9-app --test command_catalog`
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{LoadRequest, Runtime};
use om9_app::addons::FeatureName;
use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::model::Document;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The groups the catalog covers, in menu order (`Clayoo` is the SubD menu).
const MENUS: &[(&str, &str)] = &[
    ("Curve", "Curve"),
    ("Surface", "Surface"),
    ("Solid", "Solid"),
    ("Transform", "Transform"),
    ("Clayoo", "SubD (Clayoo)"),
    ("Builder", "Builder"),
    ("Tools", "Tools"),
    ("Gems", "Gems"),
    ("Settings", "Settings"),
    ("Cutters", "Cutters"),
];
/// Bundled add-ons whose commands count as available.
const ADDONS: &[&str] = &["matrix"];

/// Declares the manifest's commands; the catalog never runs them.
struct Declared;
impl Runtime for Declared {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        Ok(req.manifest.commands.iter().map(|c| c.id.clone()).collect())
    }
    fn unload(&mut self, _id: &str) {}
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ini(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current = None;
    for line in text.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = Some(name.to_owned());
            out.entry(name.to_owned()).or_default();
        } else if let (Some(section), Some((k, v))) = (&current, line.split_once('=')) {
            out.get_mut(section)
                .unwrap()
                .insert(k.trim().to_owned(), v.trim().to_owned());
        }
    }
    out
}

fn host(tmp: &Path) -> Arc<Mutex<Host>> {
    let catalog = om9_app::addons::catalog().unwrap();
    let mut h = Host::new(HostConfig::new(
        om9_app::addons::om9_version(),
        catalog.clone(),
        tmp.join("config"),
    ));
    h.register_runtime(RuntimeKind::Python, Box::new(Declared));
    h.startup(&Startup::default());
    for name in ADDONS {
        let stage = tmp.join(name);
        fs::create_dir_all(&stage).unwrap();
        let manifest =
            fs::read_to_string(repo().join(format!("addons/{name}/addon/om9_addon.toml")))
                .unwrap()
                .replace("@PLATFORM@", "any")
                .replace("runtime = \"native\"", "runtime = \"python\"")
                .replace("@ENTRY@", "entry.bin");
        fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
        fs::write(stage.join("entry.bin"), b"declared").unwrap();
        copy_tree(&repo().join(format!("addons/{name}/addon/icons")), &stage.join("icons"));
        copy_tree(&repo().join(format!("addons/{name}/addon/commands")), &stage.join("commands"));
        copy_tree(&repo().join(format!("addons/{name}/addon/lang")), &stage.join("lang"));
        let (_, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
        let path = tmp.join(format!("{name}.zip"));
        fs::write(&path, zip).unwrap();
        let r = h
            .install_zip(
                &path,
                &InstallOptions {
                    enable: true,
                    grant_permissions: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(r.enabled, "{name}: {:?}", r.warnings);
    }
    Arc::new(Mutex::new(h))
}

fn copy_tree(from: &Path, to: &Path) {
    if !from.is_dir() {
        return;
    }
    fs::create_dir_all(to).unwrap();
    for f in fs::read_dir(from).unwrap() {
        let f = f.unwrap().path();
        let dst = to.join(f.file_name().unwrap());
        if f.is_dir() {
            copy_tree(&f, &dst);
        } else {
            fs::copy(&f, dst).unwrap();
        }
    }
}

/// `Resources/icons/<group>-minimal/<code>.svg`, relative to `Resources/icons`.
fn svg(code: &str) -> Option<String> {
    let root = repo().join("Resources/icons");
    let mut dirs: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    dirs.sort();
    dirs.into_iter().find_map(|d| {
        let f = d.join(format!("{code}.svg"));
        f.is_file()
            .then(|| format!("{}/{code}.svg", d.file_name().unwrap().to_string_lossy()))
    })
}

fn render() -> String {
    let tmp = std::env::temp_dir().join(format!("om9-catalog-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let features: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../assets/features.json")).unwrap();
    let names: Vec<FeatureName> = features
        .iter()
        .map(|f| FeatureName {
            id: f["id"].as_str().unwrap(),
            command: f["command"].as_str(),
        })
        .collect();
    let by_id: BTreeMap<&str, &str> = features
        .iter()
        .filter_map(|f| Some((f["id"].as_str()?, f["name"].as_str()?)))
        .collect();
    let host = host(&tmp);
    let mut c = Controller::new(Document::new("catalog"), Limits::default()).unwrap();
    c.attach_addons(host.clone(), &names);
    // OM9 alone: what an icon runs without the add-on.
    let plain = Controller::new(Document::new("catalog"), Limits::default()).unwrap();
    let menu = ini(&fs::read_to_string(repo().join("Resources/menu/MainMenu.ini")).unwrap());
    let menus: BTreeMap<&str, &BTreeMap<String, String>> = menu
        .iter()
        .filter(|(k, _)| k.starts_with("Menu"))
        .filter_map(|(_, s)| Some((s.get("Name")?.as_str(), s)))
        .collect();

    let mut groups = Vec::new();
    let mut top11 = menu["Top11"].clone();
    top11.insert("IconCount".into(), "11".into());
    top11.insert("Color".into(), "—".into());
    groups.push(("Top 11 buttons", &top11));
    for (key, title) in MENUS {
        groups.push((*title, menus[key]));
    }

    let mut sections = Vec::new();
    let mut summary = Vec::new();
    for (title, section) in groups {
        let count: usize = section["IconCount"].parse().unwrap();
        let (mut core, mut addon, mut neither) = (0, 0, 0);
        let mut rows = Vec::new();
        for i in 1..=count {
            let code = section
                .get(&format!("Icon{i}"))
                .map(String::as_str)
                .unwrap_or("");
            let icon = om9_app::icons::icon_for_command(code).unwrap_or(code);
            let target = c.icon_target(icon, None);
            let feature = om9_app::icons::feature_for(icon);
            let name = om9_app::curve_menu::by_icon(icon)
                .map(|b| b.label)
                .or_else(|| om9_app::surface_menu::by_icon(icon).map(|b| b.label))
                .or_else(|| feature.and_then(|f| by_id.get(f).copied()))
                .filter(|n| !n.is_empty())
                .unwrap_or(icon);
            // The Top 11 buttons run OM9's own Top 11 actions (Move, Join, Ring Rail …).
            let top11 = [
                icon,
                code.trim_start_matches("TopIcon").trim_end_matches("On"),
            ]
            .into_iter()
            .any(|n| om9_core::top11::Top11Action::from_icon(n).is_some());
            let builtin = top11 || plain.icon_target(icon, None).is_some_and(|t| t.enabled);
            let by_addon = target.as_ref().is_some_and(|t| t.addon);
            if builtin {
                core += 1;
            }
            if by_addon {
                addon += 1;
            }
            if !builtin && !by_addon {
                neither += 1;
            }
            let status = match (&target, builtin) {
                (Some(t), b) if t.addon => {
                    let owner = host
                        .lock()
                        .unwrap()
                        .command_info(&t.command)
                        .map(|i| i.owner)
                        .unwrap_or_default();
                    if b {
                        format!("add-on `{owner}` (OM9 also has it)")
                    } else {
                        format!("add-on `{owner}`")
                    }
                }
                (_, true) => if top11 { "OM9 (Top 11)".to_owned() } else { "OM9".to_owned() },
                _ => "—".to_owned(),
            };
            let svg = svg(code).map_or("—".to_owned(), |s| format!("`{s}`"));
            rows.push(format!("| {i} | `{code}` | {name} | {svg} | {status} |"));
        }
        summary.push(format!(
            "| {title} | {core} | {addon} | {neither} | {count} |"
        ));
        sections.push(format!(
            "## {title}\n\nMenu colour: {}. OM9 built in: {core}, OM9 Matrix add-on: {addon}, neither: {neither} of {count}.\n\n\
             | Position | Code | Name | Icon | Runs |\n|---|---|---|---|---|\n{}\n",
            section.get("Color").map_or("—", String::as_str),
            rows.join("\n")
        ));
    }
    fs::remove_dir_all(&tmp).unwrap();
    format!(
        "# Matrix-style command catalog\n\n\
         Generated by `crates/om9-app/tests/command_catalog.rs` from `Resources/menu/MainMenu.ini`,\n\
         the icon table and the bundled add-on manifests. Do not edit by hand; regenerate with\n\
         `OM9_UPDATE_CATALOG=1 cargo test -p om9-app --test command_catalog`.\n\n\
         - **Position**: the slot in the menu palette, counted left to right and top to bottom.\n\
         - **Icon**: the authored SVG under `Resources/icons/`.\n\
         - **Runs**: what a click on the icon runs.\n\
         \x20 - add-on `om9_matrix`: the OM9 Matrix add-on package runs it while the package is\n\
         \x20   installed and enabled (icon, typed name and F6 alike).\n\
         \x20 - `OM9`: OM9's own command, when the add-on is not enabled (or does not have it).\n\
         \x20 - `—`: neither has it yet.\n\
         - With the add-on enabled, its command takes the icon; disabling the add-on gives the\n\
         \x20 icon back to OM9's own command.\n\n\
         ## Summary\n\n| Group | OM9 built in | OM9 Matrix add-on | Neither | Commands |\n|---|---|---|---|---|\n{}\n\n{}",
        summary.join("\n"),
        sections.join("\n")
    )
}

#[test]
fn catalog_matches_menus_icons_and_addons() {
    let path = repo().join("docs/matrix9-commands.md");
    let text = render();
    if std::env::var_os("OM9_UPDATE_CATALOG").is_some() {
        fs::write(&path, &text).unwrap();
        return;
    }
    let current = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == text,
        "docs/matrix9-commands.md is out of date: OM9_UPDATE_CATALOG=1 cargo test -p om9-app --test command_catalog"
    );
}
