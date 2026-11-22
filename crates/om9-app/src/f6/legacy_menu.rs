// SPDX-License-Identifier: LGPL-2.1-or-later
//! Read MatrixObjectAction 1.0 text menus into a customization draft.
use super::{
    CustomEntry, CustomMenu,
    catalog::{CATALOG, Mode},
};
use om9_core::{Error, Result};

impl CustomMenu {
    pub fn import_legacy(&mut self, mode: Mode, text: &str) -> Result<(usize, usize)> {
        let mut lines = text.trim_start_matches('\u{feff}').lines().peekable();
        if lines.next() != Some("MatrixObjectAction v:1.0") {
            return Err(Error::Invalid("F6 menu header"));
        }
        let mut replacements = Vec::new();
        let mut skipped = 0;
        while let Some(line) = lines.next() {
            let line = line.trim();
            if let Some(name) = line.strip_prefix("DialogName:") {
                let expected = if mode == Mode::TSplines {
                    "TSplines"
                } else {
                    mode.label()
                };
                if !name.trim().eq_ignore_ascii_case(expected) {
                    return Err(Error::Invalid("file belongs to a different F6 mode"));
                }
            }
            let Some(name) = line.strip_prefix("Group:") else {
                continue;
            };
            let mut fields = std::collections::BTreeMap::new();
            let mut rows = Vec::new();
            let mut in_actions = false;
            while lines
                .peek()
                .is_some_and(|line| !line.trim().starts_with("Group:"))
            {
                let line = lines.next().unwrap().trim();
                if line.is_empty() || line.starts_with("//") || line.starts_with("--") {
                    continue;
                }
                if line == "Actions:" {
                    in_actions = true;
                    continue;
                }
                if !in_actions {
                    if let Some((key, value)) = line.split_once(':') {
                        fields.insert(key.trim(), value.trim().trim_end_matches(':').trim());
                    }
                    continue;
                }
                let Some(tail) = line.strip_prefix("n:") else {
                    continue;
                };
                let (label, tail) = tail
                    .split_once(':')
                    .ok_or(Error::Invalid("F6 menu action name"))?;
                let (kind, tail) = tail
                    .split_once("c:")
                    .ok_or(Error::Invalid("F6 menu command field"))?;
                let (command, _) = tail
                    .split_once(';')
                    .ok_or(Error::Invalid("F6 menu command terminator"))?;
                let command = command.trim();
                let category_name = kind
                    .trim()
                    .strip_prefix("t:")
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(':')
                    .trim();
                let category = super::catalog::Category::ALL
                    .into_iter()
                    .find(|c| c.label().eq_ignore_ascii_case(category_name))
                    .unwrap_or_default();
                // These legacy buttons now live in the mode strip and footer.
                if ["_gvObjectActionsMode", "_gvObjectActionsEdit"]
                    .iter()
                    .any(|c| command.eq_ignore_ascii_case(c))
                {
                    continue;
                }
                rows.push(if label.trim() == "NewLine" {
                    (String::new(), String::new(), super::catalog::Category::None)
                } else {
                    (label.trim().to_owned(), command.to_owned(), category)
                });
            }
            if !in_actions {
                return Err(Error::Invalid("F6 group has no Actions section"));
            }
            let candidates = CATALOG.groups.iter().filter(|g| {
                g.mode == mode
                    && (name.trim().is_empty()
                        || g.name.trim_end_matches(':') == name.trim().trim_end_matches(':'))
            });
            let Some(group) = candidates
                .into_iter()
                .find(|g| g.legacy_fields_match(&fields))
            else {
                skipped += 1;
                continue;
            };
            replacements.push((group.scope_key(), rows));
        }
        if replacements.is_empty() {
            return Err(Error::Invalid("no matching F6 groups"));
        }
        let count = replacements.len();
        for (group, rows) in replacements {
            self.entries
                .retain(|entry| entry.mode != mode || entry.group != group);
            if !self
                .replaced_groups
                .iter()
                .any(|(m, g)| *m == mode && *g == group)
            {
                self.replaced_groups.push((mode, group.clone()));
            }
            self.entries.extend(
                rows.into_iter()
                    .map(|(label, command, category)| CustomEntry {
                        category,
                        mode,
                        group: group.clone(),
                        global: false,
                        label,
                        command,
                    }),
            );
        }
        Ok((count, skipped))
    }
}
