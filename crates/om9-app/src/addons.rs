// SPDX-License-Identifier: LGPL-2.1-or-later
//! Add-on integration (Preferences › Add-ons).
//!
//! `om9-addons` owns packages, manifests, enable state, settings and keymap. This module
//! opens the host at startup and derives the name table the controller uses so menu,
//! sidebar icons, F6 and CMD light up the commands add-ons provide. A typed Matrix command
//! name (`gvGemOnCrv`), its feature id (`OM9-GEM-002`) and its catalog code
//! (`BuilderGemOnCrv`) all resolve to the same code. Commands no add-on provides keep the
//! existing rule: visible and disabled.

use eframe::egui;
use om9_addons::catalog::Catalog;
use om9_addons::host::{Host, HostConfig, Startup, default_config_dir};
use om9_addons::manifest::RuntimeKind;
use om9_addons::native::NativeRuntime;
use om9_addons::semver::Version;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub type SharedHost = Arc<Mutex<Host>>;

/// A feature row's id and typed command name (from `assets/features.json`).
#[derive(Clone, Copy)]
pub struct FeatureName<'a> {
    pub id: &'a str,
    pub command: Option<&'a str>,
}

/// lowercase name (catalog code, feature id or typed command) → catalog code.
pub fn provided_names<'a>(host: &Host, features: &[FeatureName<'a>]) -> BTreeMap<String, String> {
    let bound = host.bound_commands();
    // A command's own code always names it, whatever feature or alias another command has.
    let mut map: BTreeMap<String, String> = bound
        .iter()
        .map(|(code, _, _)| (code.to_ascii_lowercase(), code.clone()))
        .collect();
    let codes: std::collections::BTreeSet<String> = map.keys().cloned().collect();
    for (code, _owner, _label) in &bound {
        if let Some(info) = host.command_info(code) {
            for alias in &info.command.aliases {
                map.entry(alias.to_ascii_lowercase())
                    .or_insert_with(|| code.clone());
            }
        }
        if let Some(feature) = crate::icons::feature_for(code) {
            let mut names = vec![feature.to_ascii_lowercase()];
            for f in features {
                if f.id == feature
                    && let Some(cmd) = f.command
                    && !cmd.trim().is_empty()
                {
                    names.push(cmd.trim().to_ascii_lowercase());
                }
            }
            for name in names {
                if !codes.contains(&name) {
                    map.insert(name, code.clone());
                }
            }
        }
    }
    map
}

/// The OM9 command catalog shipped in the binary.
pub fn catalog() -> Result<Catalog, String> {
    Catalog::from_main_menu_ini(include_str!("../../../Resources/menu/MainMenu.ini"))
}

pub fn om9_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version(0, 1, 0))
}

/// A host with the runtimes this build supports.
pub fn new_host(config: HostConfig) -> Host {
    let mut host = Host::new(config);
    host.register_runtime(RuntimeKind::Native, Box::new(NativeRuntime::new()));
    host
}

/// Open the add-on host for the interactive application.
///
/// - user repository and `userprefs.toml`: `OM9_CONFIG_DIR` or the per-version config folder;
/// - system repository: `addons/` beside `om9.exe` (read-only, shipped with the package);
/// - `OM9_SAFE_MODE=1` starts without enabling any add-on and without rewriting preferences.
pub fn open_host() -> Result<(SharedHost, Vec<String>), String> {
    let version = om9_version();
    let mut config = HostConfig::new(version, catalog()?, default_config_dir(version));
    config.system_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("addons")))
        .filter(|dir| dir.is_dir());
    let mut host = new_host(config);
    let safe_mode = std::env::var("OM9_SAFE_MODE").is_ok_and(|v| v.trim() == "1");
    host.startup(&Startup {
        safe_mode,
        only: None,
    });
    let log = std::mem::take(&mut host.log);
    Ok((Arc::new(Mutex::new(host)), log))
}

/// Parse a normalized OM9 shortcut (`Ctrl+Shift+G`) into egui terms.
pub fn egui_shortcut(keys: &str) -> Option<(egui::Modifiers, egui::Key)> {
    let mut mods = egui::Modifiers::NONE;
    let mut key = None;
    for part in keys.split('+') {
        match part {
            "Ctrl" => mods.ctrl = true,
            "Alt" => mods.alt = true,
            "Shift" => mods.shift = true,
            "Meta" => mods.mac_cmd = true,
            other => key = egui::Key::from_name(if other == "Esc" { "Escape" } else { other }),
        }
    }
    key.map(|k| (mods, k))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipped_catalog_has_gem_commands() {
        let c = catalog().unwrap();
        assert_eq!(c.group_of("BuilderGemOnCrv"), Some("Gems"));
        assert_eq!(c.group_of("BuilderGemCountOnCrv"), Some("Gems"));
    }
    #[test]
    fn shortcuts_map_to_egui() {
        let (m, k) = egui_shortcut("Ctrl+Shift+G").unwrap();
        assert!(m.ctrl && m.shift && !m.alt);
        assert_eq!(k, egui::Key::G);
        assert_eq!(egui_shortcut("Alt+F4").unwrap().1, egui::Key::F4);
        assert!(egui_shortcut("Ctrl+Nope").is_none());
    }
}
