use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    IconList,
    Custom,
    Reset,
}
#[derive(Debug)]
pub struct MenuGroup {
    pub id: usize,
    pub title: String,
    pub kind: MenuKind,
    pub color: String,
    pub icons: Vec<String>,
}
#[derive(Debug)]
pub struct MenuCatalog {
    pub groups: Vec<MenuGroup>,
    pub quick_icons: Vec<String>,
}
#[derive(Debug)]
pub struct MenuError(String);
impl fmt::Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for MenuError {}
type Sections = BTreeMap<String, BTreeMap<String, String>>;
fn error(section: &str, key: &str, reason: &str) -> MenuError {
    MenuError(format!("[{section}] {key}: {reason}"))
}
fn value<'a>(sections: &'a Sections, section: &str, key: &str) -> Result<&'a str, MenuError> {
    sections
        .get(section)
        .and_then(|s| s.get(key))
        .filter(|v| !v.is_empty())
        .map(String::as_str)
        .ok_or_else(|| error(section, key, "missing or empty"))
}
fn count(sections: &Sections, section: &str, key: &str) -> Result<usize, MenuError> {
    value(sections, section, key)?
        .parse::<usize>()
        .ok()
        .filter(|n| *n > 0 && *n <= 4096)
        .ok_or_else(|| error(section, key, "expected count in 1..4096"))
}
fn icons(sections: &Sections, section: &str, n: usize) -> Result<Vec<String>, MenuError> {
    if let Some(s) = sections.get(section) {
        for key in s.keys() {
            if let Some(i) = key
                .strip_prefix("Icon")
                .and_then(|v| v.parse::<usize>().ok())
                && (i == 0 || i > n)
            {
                return Err(error(
                    section,
                    "IconCount",
                    "extra icon outside declared count",
                ));
            }
        }
    }
    (1..=n)
        .map(|i| value(sections, section, &format!("Icon{i}")).map(str::to_owned))
        .collect()
}
impl MenuCatalog {
    pub fn parse(text: &str) -> Result<Self, MenuError> {
        let mut sections = Sections::new();
        let mut current = String::new();
        for line in text.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current = line[1..line.len() - 1].trim().to_owned();
                if current.is_empty() || sections.insert(current.clone(), BTreeMap::new()).is_some()
                {
                    return Err(error(&current, "section", "duplicate or empty"));
                }
            } else {
                let (key, val) = line
                    .split_once('=')
                    .ok_or_else(|| error(&current, "syntax", "expected key=value"))?;
                let section = sections
                    .get_mut(&current)
                    .ok_or_else(|| error(&current, key, "no section"))?;
                let key = key.trim();
                if key.is_empty()
                    || val.contains('\0')
                    || section
                        .insert(key.to_owned(), val.trim().to_owned())
                        .is_some()
                {
                    return Err(error(&current, key, "duplicate key or invalid value"));
                }
            }
        }
        let n = count(&sections, "Settings", "MenuCount")?;
        for section in sections.keys() {
            if let Some(i) = section
                .strip_prefix("Menu")
                .and_then(|v| v.parse::<usize>().ok())
                && (i == 0 || i > n)
            {
                return Err(error("Settings", "MenuCount", "extra menu section"));
            }
        }
        let mut groups = Vec::new();
        for id in 0..n {
            let section = format!("Menu{}", id + 1);
            let title = value(&sections, &section, "Name")?.to_owned();
            let color = value(&sections, &section, "Color")?.to_owned();
            let kind = match value(&sections, &section, "Type")? {
                "IconList" => MenuKind::IconList,
                "Custom" => MenuKind::Custom,
                "Reset" => MenuKind::Reset,
                _ => return Err(error(&section, "Type", "unknown menu kind")),
            };
            let items = if kind == MenuKind::IconList {
                icons(
                    &sections,
                    &section,
                    count(&sections, &section, "IconCount")?,
                )?
            } else {
                Vec::new()
            };
            groups.push(MenuGroup {
                id,
                title,
                kind,
                color,
                icons: items,
            });
        }
        Ok(Self {
            groups,
            quick_icons: icons(&sections, "Top11", 11)?,
        })
    }
}
