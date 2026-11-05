//! Preferences của người dùng (`userprefs.toml`), tương đương phần add-on trong userpref.blend.
//!
//! ```toml
//! online_access = false
//! dev_paths = ["D:/om9-dev/addons"]
//! [keymap]
//! BuilderGemOnCrv = "Ctrl+Shift+G"
//! [addons.gem_on_curve]
//! enabled = true
//! granted = ["files"]
//! [addons.gem_on_curve.settings]
//! gap = 0.2
//! ```

use crate::settings::SettingValue;
use crate::toml::{self, Table, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AddonPrefs {
    pub enabled: bool,
    pub granted: BTreeSet<String>,
    pub settings: BTreeMap<String, SettingValue>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Prefs {
    pub addons: BTreeMap<String, AddonPrefs>,
    /// mã lệnh → phím tắt người dùng đặt; "" = người dùng bỏ phím mặc định
    pub keymap: BTreeMap<String, String>,
    pub online_access: bool,
    pub dev_paths: Vec<String>,
}

impl Prefs {
    pub fn from_toml(t: &Table) -> Prefs {
        let mut p = Prefs {
            online_access: matches!(t.get("online_access"), Some(Value::Bool(true))),
            ..Default::default()
        };
        if let Some(Value::Array(a)) = t.get("dev_paths") {
            p.dev_paths = a
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        if let Some(Value::Table(k)) = t.get("keymap") {
            for (code, v) in k {
                if let Some(s) = v.as_str() {
                    p.keymap.insert(code.clone(), s.to_string());
                }
            }
        }
        if let Some(Value::Table(ads)) = t.get("addons") {
            for (id, v) in ads {
                let Some(at) = v.as_table() else { continue };
                let mut ap = AddonPrefs {
                    enabled: matches!(at.get("enabled"), Some(Value::Bool(true))),
                    ..Default::default()
                };
                if let Some(Value::Array(g)) = at.get("granted") {
                    ap.granted = g
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect();
                }
                if let Some(Value::Table(s)) = at.get("settings") {
                    for (k, v) in s {
                        if let Some(sv) = SettingValue::from_toml(v) {
                            ap.settings.insert(k.clone(), sv);
                        }
                    }
                }
                p.addons.insert(id.clone(), ap);
            }
        }
        p
    }

    pub fn to_toml(&self) -> Table {
        let mut t = Table::new();
        t.insert("online_access".into(), Value::Bool(self.online_access));
        t.insert(
            "dev_paths".into(),
            Value::Array(self.dev_paths.iter().cloned().map(Value::Str).collect()),
        );
        t.insert(
            "keymap".into(),
            Value::Table(
                self.keymap
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::Str(v.clone())))
                    .collect(),
            ),
        );
        let mut ads = Table::new();
        for (id, a) in &self.addons {
            let mut at = Table::new();
            at.insert("enabled".into(), Value::Bool(a.enabled));
            at.insert(
                "granted".into(),
                Value::Array(a.granted.iter().cloned().map(Value::Str).collect()),
            );
            at.insert(
                "settings".into(),
                Value::Table(
                    a.settings
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_toml()))
                        .collect(),
                ),
            );
            ads.insert(id.clone(), Value::Table(at));
        }
        t.insert("addons".into(), Value::Table(ads));
        t
    }

    /// Đọc file; file hỏng được đổi tên `.bad` và trả về prefs rỗng kèm cảnh báo (không mất dữ liệu).
    pub fn load(path: &Path) -> (Prefs, Option<String>) {
        let Ok(text) = fs::read_to_string(path) else {
            return (Prefs::default(), None);
        };
        match toml::parse(&text) {
            Ok(t) => (Prefs::from_toml(&t), None),
            Err(e) => {
                let bad = path.with_extension("toml.bad");
                let _ = fs::rename(path, &bad);
                (
                    Prefs::default(),
                    Some(format!(
                        "userprefs.toml hỏng ({e}); đã lưu bản cũ thành {}",
                        bad.display()
                    )),
                )
            }
        }
    }

    /// Ghi nguyên tử: file tạm cùng thư mục → fsync → rename.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("toml.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(
                b"# OpenMatrix9 user preferences - OM9 tu ghi, co the sua tay khi OM9 dang tat\n",
            )?;
            f.write_all(toml::to_string(&self.to_toml()).as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_corrupt_recovery() {
        let dir = std::env::temp_dir().join(format!("om9-prefs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("userprefs.toml");
        let mut p = Prefs::default();
        p.keymap
            .insert("BuilderGemOnCrv".into(), "Ctrl+Shift+G".into());
        let mut a = AddonPrefs {
            enabled: true,
            ..Default::default()
        };
        a.granted.insert("files".into());
        a.settings.insert("gap".into(), SettingValue::Float(0.25));
        a.settings
            .insert("name".into(), SettingValue::Str("Đá \"tròn\"".into()));
        p.addons.insert("gem_on_curve".into(), a);
        p.save(&path).unwrap();
        let (q, w) = Prefs::load(&path);
        assert!(w.is_none());
        assert_eq!(p, q);
        fs::write(&path, "addons = [[[").unwrap();
        let (r, w) = Prefs::load(&path);
        assert_eq!(r, Prefs::default());
        assert!(w.unwrap().contains("hỏng"));
        assert!(path.with_extension("toml.bad").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
