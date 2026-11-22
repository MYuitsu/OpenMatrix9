// SPDX-License-Identifier: LGPL-2.1-or-later
//! Named presentation snapshots. The root v2 layout mirrors the startup profile for older readers.
use crate::dock_layout::{DockLayout, SCHEMA_VERSION};
use om9_core::{Error, Result, budget::Limits};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutProfiles {
    pub active: String,
    pub startup: String,
    pub layouts: BTreeMap<String, DockLayout>,
}

impl Default for LayoutProfiles {
    fn default() -> Self {
        Self::single(DockLayout::default())
    }
}

pub enum ProfileAction<'a> {
    Create(&'a str),
    Duplicate(&'a str),
    Rename(&'a str),
    Switch(&'a str),
    Startup(&'a str),
    Reset,
}

impl LayoutProfiles {
    fn single(layout: DockLayout) -> Self {
        let name = "Matrix".to_owned();
        Self {
            active: name.clone(),
            startup: name.clone(),
            layouts: [(name, layout)].into(),
        }
    }

    pub fn load(path: &Path) -> (Self, bool, Vec<String>) {
        let limits = Limits::default();
        let read = || -> Result<(Self, bool, Vec<String>)> {
            let file = match std::fs::File::open(path) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Ok((Self::default(), false, vec![]));
                }
                Err(e) => return Err(Error::Io(e.to_string())),
            };
            let mut bytes = Vec::new();
            file.take(limits.max_bytes.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(|e| Error::Io(e.to_string()))?;
            let (layout, mut diagnostics) = DockLayout::from_json(&bytes, &limits)?;
            let mut root: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| Error::Invalid("layout document"))?;
            if root["schema_version"].as_u64() != Some(u64::from(SCHEMA_VERSION))
                || root.get("layout_profiles").is_none()
            {
                return Ok((Self::single(layout), false, diagnostics));
            }
            let mut profiles: Self = serde_json::from_value(root["layout_profiles"].take())
                .map_err(|_| Error::Invalid("layout profiles"))?;
            if !profiles.layouts.contains_key(&profiles.startup)
                || !profiles.layouts.contains_key(&profiles.active)
            {
                return Err(Error::Invalid("missing layout profile"));
            }
            for (name, layout) in &mut profiles.layouts {
                Self::name(name)?;
                if layout.schema_version != SCHEMA_VERSION {
                    return Err(Error::Invalid("profile layout version"));
                }
                diagnostics.extend(layout.normalize());
            }
            profiles.active.clone_from(&profiles.startup);
            Ok((profiles, true, diagnostics))
        };
        read().unwrap_or_else(|error| {
            (
                Self::default(),
                false,
                vec![format!(
                    "layout ignored ({error}); restored the Matrix default"
                )],
            )
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut root = serde_json::to_value(&self.layouts[&self.startup])
            .map_err(|_| Error::Invalid("startup layout"))?;
        root["layout_profiles"] =
            serde_json::to_value(self).map_err(|_| Error::Invalid("layout profiles"))?;
        let json = serde_json::to_string_pretty(&root)
            .map_err(|_| Error::Invalid("layout serialization"))?;
        DockLayout::write_json(path, &json)
    }

    fn name(name: &str) -> Result<&str> {
        let name = name.trim();
        if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
            return Err(Error::Invalid("layout profile name"));
        }
        Ok(name)
    }

    pub fn edit(&mut self, live: &DockLayout, action: ProfileAction<'_>) -> Result<()> {
        use ProfileAction::*;
        match action {
            Create(name) | Duplicate(name) | Rename(name) => {
                let name = Self::name(name)?;
                if self.layouts.contains_key(name) {
                    return Err(Error::Invalid("layout profile name already exists"));
                }
                if matches!(action, Rename(_)) {
                    self.layouts.remove(&self.active);
                    if self.startup == self.active {
                        self.startup = name.to_owned();
                    }
                } else {
                    self.layouts.insert(self.active.clone(), live.clone());
                }
                self.active = name.to_owned();
                self.layouts.insert(self.active.clone(), live.clone());
            }
            Switch(name) | Startup(name) => {
                if !self.layouts.contains_key(name) {
                    return Err(Error::Invalid("unknown layout profile"));
                }
                self.layouts.insert(self.active.clone(), live.clone());
                if matches!(action, Switch(_)) {
                    self.active = name.to_owned();
                } else {
                    self.startup = name.to_owned();
                }
            }
            Reset => {
                self.layouts
                    .insert(self.active.clone(), DockLayout::default());
            }
        }
        Ok(())
    }
}
