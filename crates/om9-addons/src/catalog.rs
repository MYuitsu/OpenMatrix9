//! Catalog lệnh OM9 (đọc `Resources/menu/MainMenu.ini`): mã lệnh → nhóm menu.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub codes: BTreeSet<String>,
    /// nhóm → mã theo thứ tự menu
    pub groups: BTreeMap<String, Vec<String>>,
}

impl Catalog {
    pub fn from_main_menu_ini(text: &str) -> Result<Catalog, String> {
        let mut c = Catalog::default();
        let mut section = String::new();
        let mut name: Option<String> = None;
        let mut icons: Vec<String> = Vec::new();
        let flush =
            |c: &mut Catalog, section: &str, name: &Option<String>, icons: &mut Vec<String>| {
                let g = if section == "Top11" {
                    Some("Quick".to_string())
                } else if section.starts_with("Menu") {
                    name.clone()
                } else {
                    None
                };
                if let Some(g) = g
                    && !icons.is_empty()
                {
                    c.groups.entry(g).or_default().append(icons);
                }
                icons.clear();
            };
        for raw in text.trim_start_matches('\u{feff}').lines() {
            let line = raw.trim();
            if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                flush(&mut c, &section, &name, &mut icons);
                section = s.to_string();
                name = None;
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            if k == "Name" {
                name = Some(v.to_string());
            } else if k.starts_with("Icon")
                && k[4..].bytes().all(|b| b.is_ascii_digit())
                && !k[4..].is_empty()
                && !v.is_empty()
            {
                c.codes.insert(v.to_string());
                icons.push(v.to_string());
            }
        }
        flush(&mut c, &section, &name, &mut icons);
        if c.codes.is_empty() {
            return Err("catalog rỗng hoặc sai định dạng".into());
        }
        Ok(c)
    }

    pub fn group_of(&self, code: &str) -> Option<&str> {
        self.groups
            .iter()
            .find(|(_, v)| v.iter().any(|c| c == code))
            .map(|(k, _)| k.as_str())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_groups() {
        let ini = "[Top11]\nIcon1=TopIconMove\n[Settings]\nMenuCount=2\n[Menu1]\nName=Gems\nIconCount=2\nIcon1=BuilderGemOnCrv\nIcon2=BuilderGemOnSurface\n[Menu2]\nName=Custom\n";
        let c = super::Catalog::from_main_menu_ini(ini).unwrap();
        assert_eq!(c.codes.len(), 3);
        assert_eq!(c.group_of("BuilderGemOnSurface"), Some("Gems"));
        assert_eq!(c.group_of("TopIconMove"), Some("Quick"));
        assert!(!c.groups.contains_key("Custom"));
    }
}
