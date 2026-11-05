//! `om9_addon.toml` — schema 1.x.
//!
//! Trường bám theo `blender_manifest.toml` (schema_version, id, version, name, tagline,
//! maintainer, type, license, *_version_min/max, platforms, permissions, build.paths_exclude_pattern)
//! cộng phần riêng của OM9: runtime, entry, [[commands]] (mã catalog), [[settings]], [[assets]].

use crate::catalog::Catalog;
use crate::keymap;
use crate::semver::Version;
use crate::settings::SettingSpec;
use crate::toml::{self, Table, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const FILE_NAME: &str = "om9_addon.toml";
pub const SCHEMA_MAJOR: u64 = 1;
pub const PLATFORMS: &[&str] = &["windows-x64", "linux-x64", "macos-arm64", "any"];
pub const PERMISSIONS: &[&str] = &["files", "network", "clipboard"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddonType {
    /// Có mã lệnh chạy được.
    AddOn,
    /// Chỉ dữ liệu: thư viện đá, bảng size nhẫn, mật độ kim loại, preset...
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeKind {
    Python,
    Wasm,
    Native,
    Data,
}

impl RuntimeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeKind::Python => "python",
            RuntimeKind::Wasm => "wasm",
            RuntimeKind::Native => "native",
            RuntimeKind::Data => "data",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSpec {
    pub id: String,
    pub label: String,
    pub handler: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    /// Lệnh mới không có trong catalog Matrix 9 (phải ghi rõ nhóm menu).
    pub is_new: bool,
    pub group: Option<String>,
    /// `builder = true`: the command opens its builder (command-line options, builder panel and
    /// live preview) and only creates geometry on Enter/Accept. Otherwise it runs at once.
    pub builder: bool,
    /// `interactive = true`: the command runs as a step-by-step session (points, command-line
    /// sub-options, preview) through `om9_addon_session`.
    pub interactive: bool,
    /// Extra names the command can be typed as (`Rectangle`, `Circle` …), besides the catalog
    /// command name.
    pub aliases: Vec<String>,
    /// `mesh_data = true`: an interactive command receives the triangles of selected meshes
    /// and solids (Metal Weights, Object Checker …). Off by default: it is heavy.
    pub mesh_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetSpec {
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub schema_version: Version,
    pub id: String,
    pub version: Version,
    pub name: String,
    pub tagline: Option<String>,
    pub maintainer: Option<String>,
    pub addon_type: AddonType,
    pub license: Vec<String>,
    pub om9_version_min: Version,
    /// Không bao gồm (giống blender_version_max).
    pub om9_version_max: Option<Version>,
    pub platforms: Vec<String>,
    pub runtime: RuntimeKind,
    pub entry: Option<String>,
    /// quyền → lý do hiển thị cho người dùng
    pub permissions: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub commands: Vec<CommandSpec>,
    pub settings: Vec<SettingSpec>,
    pub assets: Vec<AssetSpec>,
    pub exclude: Vec<String>,
}

/// Ngữ cảnh kiểm tra: danh sách file trong gói và catalog (nếu có).
#[derive(Default)]
pub struct Check<'a> {
    pub files: Option<&'a BTreeSet<String>>,
    pub catalog: Option<&'a Catalog>,
}

pub fn valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    (2..=64).contains(&b.len())
        && b[0].is_ascii_lowercase()
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

fn valid_cmd(id: &str) -> bool {
    let b = id.as_bytes();
    (1..=96).contains(&b.len())
        && b[0].is_ascii_alphabetic()
        && b.iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b' ')
}

/// Đường dẫn tương đối an toàn kiểu POSIX (dùng cho file trong gói).
pub fn safe_rel_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains('\\')
        && !p.contains('\0')
        && !(p.len() >= 2 && p.as_bytes()[1] == b':')
        && p.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
}

struct R<'t> {
    t: &'t Table,
    errs: Vec<String>,
}

impl<'t> R<'t> {
    fn str_opt(&mut self, k: &str) -> Option<String> {
        match self.t.get(k) {
            None => None,
            Some(Value::Str(s)) => Some(s.clone()),
            Some(v) => {
                self.errs
                    .push(format!("{k} phải là chuỗi, không phải {}", v.type_name()));
                None
            }
        }
    }
    fn str_req(&mut self, k: &str) -> String {
        if !self.t.contains_key(k) {
            self.errs.push(format!("thiếu trường bắt buộc: {k}"));
            return String::new();
        }
        self.str_opt(k).unwrap_or_default()
    }
    fn ver(&mut self, k: &str, req: bool) -> Option<Version> {
        let s = if req {
            Some(self.str_req(k))
        } else {
            self.str_opt(k)
        }?;
        if s.is_empty() && req {
            return None;
        }
        match Version::parse(&s) {
            Ok(v) => Some(v),
            Err(e) => {
                self.errs.push(format!("{k}: {e}"));
                None
            }
        }
    }
    fn str_list(&mut self, k: &str) -> Option<Vec<String>> {
        match self.t.get(k) {
            None => None,
            Some(Value::Array(a)) => {
                let mut out = Vec::new();
                for v in a {
                    match v {
                        Value::Str(s) => out.push(s.clone()),
                        _ => {
                            self.errs.push(format!("{k} chỉ chứa chuỗi"));
                            return None;
                        }
                    }
                }
                Some(out)
            }
            Some(v) => {
                self.errs
                    .push(format!("{k} phải là mảng, không phải {}", v.type_name()));
                None
            }
        }
    }
    fn tables(&mut self, k: &str) -> Vec<&'t Table> {
        match self.t.get(k) {
            None => Vec::new(),
            Some(Value::Array(a)) if a.iter().all(|v| matches!(v, Value::Table(_))) => {
                a.iter().filter_map(Value::as_table).collect()
            }
            Some(_) => {
                self.errs.push(format!("{k} phải là [[{k}]]"));
                Vec::new()
            }
        }
    }
}

pub fn parse(src: &str, chk: &Check) -> Result<Manifest, Vec<String>> {
    let t = toml::parse(src).map_err(|e| vec![format!("{FILE_NAME} lỗi cú pháp, {e}")])?;
    from_table(&t, chk)
}

pub fn from_table(t: &Table, chk: &Check) -> Result<Manifest, Vec<String>> {
    let mut r = R {
        t,
        errs: Vec::new(),
    };
    let has_file = |p: &str| chk.files.is_none_or(|f| f.contains(p));

    let schema_version = r.ver("schema_version", true).unwrap_or(Version(0, 0, 0));
    if schema_version.0 != SCHEMA_MAJOR && schema_version != Version(0, 0, 0) {
        r.errs.push(format!(
            "schema_version {schema_version} không tương thích (host hỗ trợ {SCHEMA_MAJOR}.x)"
        ));
    }
    let id = r.str_req("id");
    if !id.is_empty() && !valid_id(&id) {
        r.errs.push(format!(
            "id không hợp lệ: {id:?} (2–64 ký tự: chữ thường, số, _; bắt đầu bằng chữ)"
        ));
    }
    let version = r.ver("version", true).unwrap_or(Version(0, 0, 0));
    let name = r.str_req("name");
    if name.chars().count() > 64 {
        r.errs.push("name dài quá 64 ký tự".into());
    }
    let tagline = r.str_opt("tagline");
    if tagline.as_ref().is_some_and(|s| s.chars().count() > 64) {
        r.errs.push("tagline dài quá 64 ký tự".into());
    }
    let maintainer = r.str_opt("maintainer");

    let addon_type = match r.str_opt("type").as_deref() {
        None | Some("add-on") => AddonType::AddOn,
        Some("library") => AddonType::Library,
        Some(o @ ("theme" | "keymap")) => {
            r.errs
                .push(format!("type {o:?} dự kiến cho bản sau, chưa hỗ trợ"));
            AddonType::AddOn
        }
        Some(o) => {
            r.errs
                .push(format!("type {o:?} không hợp lệ (add-on | library)"));
            AddonType::AddOn
        }
    };

    let license = r.str_list("license").unwrap_or_default();
    if license.is_empty() {
        r.errs
            .push("license phải là danh sách \"SPDX:<mã>\" không rỗng".into());
    } else if let Some(bad) = license
        .iter()
        .find(|l| l.strip_prefix("SPDX:").is_none_or(str::is_empty))
    {
        r.errs
            .push(format!("license {bad:?} phải có dạng \"SPDX:<mã>\""));
    }

    let om9_version_min = r.ver("om9_version_min", true).unwrap_or(Version(0, 0, 0));
    let om9_version_max = r.ver("om9_version_max", false);
    if om9_version_max.is_some_and(|m| m <= om9_version_min) {
        r.errs
            .push("om9_version_max phải lớn hơn om9_version_min".into());
    }

    let platforms = r
        .str_list("platforms")
        .unwrap_or_else(|| vec!["any".into()]);
    if platforms.is_empty() {
        r.errs.push("platforms không được rỗng".into());
    }
    for p in &platforms {
        if !PLATFORMS.contains(&p.as_str()) {
            r.errs.push(format!(
                "platform {p:?} không hợp lệ ({})",
                PLATFORMS.join(", ")
            ));
        }
    }

    let default_rt = if addon_type == AddonType::Library {
        "data"
    } else {
        ""
    };
    let rt = r
        .str_opt("runtime")
        .unwrap_or_else(|| default_rt.to_string());
    let runtime = match rt.as_str() {
        "python" => RuntimeKind::Python,
        "wasm" => RuntimeKind::Wasm,
        "native" => RuntimeKind::Native,
        "data" => RuntimeKind::Data,
        "" => {
            r.errs.push("thiếu trường bắt buộc: runtime".into());
            RuntimeKind::Data
        }
        o => {
            r.errs.push(format!(
                "runtime {o:?} không hợp lệ (python | wasm | native | data)"
            ));
            RuntimeKind::Data
        }
    };
    if runtime == RuntimeKind::Native && platforms.iter().any(|p| p == "any") {
        r.errs
            .push("runtime native phải khai báo nền tảng cụ thể, không dùng \"any\"".into());
    }
    match (addon_type, runtime) {
        (AddonType::Library, rt) if rt != RuntimeKind::Data => {
            r.errs.push("type library chỉ dùng runtime \"data\"".into())
        }
        (AddonType::AddOn, RuntimeKind::Data) if !rt.is_empty() => r
            .errs
            .push("type add-on cần runtime chạy mã (python | wasm | native)".into()),
        _ => {}
    }

    let entry = r.str_opt("entry");
    if runtime != RuntimeKind::Data {
        match &entry {
            None => r.errs.push("thiếu trường bắt buộc: entry".into()),
            Some(e) if !safe_rel_path(e) => r.errs.push(format!(
                "entry {e:?} không phải đường dẫn tương đối an toàn"
            )),
            Some(e) if !has_file(e) => r.errs.push(format!("entry {e:?} không có trong gói")),
            _ => {}
        }
    }

    let mut permissions = BTreeMap::new();
    match t.get("permissions") {
        None => {}
        Some(Value::Table(p)) => {
            for (k, v) in p {
                if !PERMISSIONS.contains(&k.as_str()) {
                    r.errs.push(format!(
                        "quyền {k:?} không hợp lệ ({})",
                        PERMISSIONS.join(", ")
                    ));
                    continue;
                }
                match v.as_str().map(str::trim) {
                    Some(reason) if !reason.is_empty() && reason.chars().count() <= 80 => {
                        permissions.insert(k.clone(), reason.to_string());
                    }
                    _ => r.errs.push(format!(
                        "quyền {k}: cần lý do (chuỗi 1–80 ký tự) để hiển thị cho người dùng"
                    )),
                }
            }
        }
        Some(_) => r
            .errs
            .push("permissions phải là bảng { quyền = \"lý do\" }".into()),
    }

    let tags = r.str_list("tags").unwrap_or_default();

    // [[commands]]
    let mut commands = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, c) in r.tables("commands").into_iter().enumerate() {
        let mut cr = R {
            t: c,
            errs: Vec::new(),
        };
        let cid = cr.str_opt("id").unwrap_or_default();
        let ctx = if cid.is_empty() {
            format!("commands[{i}]")
        } else {
            format!("lệnh {cid}")
        };
        let label = cr.str_opt("label").unwrap_or_else(|| cid.clone());
        let handler = cr.str_opt("handler");
        let icon = cr.str_opt("icon");
        let shortcut = cr.str_opt("shortcut");
        let group = cr.str_opt("group");
        let is_new = matches!(c.get("new"), Some(Value::Bool(true)));
        let builder = matches!(c.get("builder"), Some(Value::Bool(true)));
        let interactive = matches!(c.get("interactive"), Some(Value::Bool(true)));
        let mesh_data = matches!(c.get("mesh_data"), Some(Value::Bool(true)));
        let aliases = cr.str_list("aliases").unwrap_or_default();
        let mut errs: Vec<String> = cr.errs.into_iter().map(|e| format!("{ctx}: {e}")).collect();
        for a in &aliases {
            if a.is_empty()
                || a.len() > 48
                || !a.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                errs.push(format!("{ctx}: alias {a:?} phải là một từ chữ/số"));
            }
        }
        if !valid_cmd(&cid) {
            errs.push(format!("{ctx}: mã lệnh không hợp lệ {cid:?}"));
        } else if !seen.insert(cid.clone()) {
            errs.push(format!("{ctx}: mã lệnh lặp"));
        }
        if let Some(cat) = chk.catalog {
            if is_new {
                if cat.codes.contains(&cid) {
                    errs.push(format!("{ctx}: đã có trong catalog, bỏ new = true"));
                }
                if group.as_ref().is_none_or(|g| !cat.groups.contains_key(g)) {
                    errs.push(format!(
                        "{ctx}: lệnh mới cần group là một nhóm menu có sẵn ({})",
                        cat.groups.keys().cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            } else if !cat.codes.contains(&cid) && valid_cmd(&cid) {
                errs.push(format!("{ctx}: mã {cid:?} không có trong catalog (đặt new = true và group nếu là lệnh mới)"));
            }
        }
        if runtime == RuntimeKind::Data {
            errs.push(format!("{ctx}: add-on runtime data không có lệnh"));
        } else if handler.as_ref().is_none_or(|h| h.trim().is_empty()) {
            errs.push(format!("{ctx}: thiếu handler"));
        }
        if let Some(ic) = &icon
            && (!safe_rel_path(ic) || !has_file(ic))
        {
            errs.push(format!("{ctx}: icon {ic:?} không có trong gói"));
        }
        let shortcut = match shortcut {
            Some(s) => match keymap::normalize(&s) {
                Ok(n) => Some(n),
                Err(e) => {
                    errs.push(format!("{ctx}: {e}"));
                    None
                }
            },
            None => None,
        };
        r.errs.extend(errs);
        commands.push(CommandSpec {
            id: cid,
            label,
            handler,
            icon,
            shortcut,
            is_new,
            group,
            builder,
            interactive,
            aliases,
            mesh_data,
        });
    }
    if addon_type == AddonType::AddOn && commands.is_empty() {
        r.errs
            .push("add-on phải khai báo ít nhất một [[commands]]".into());
    }

    // [[settings]]
    let mut settings = Vec::new();
    let mut skeys = BTreeSet::new();
    for (i, s) in r.tables("settings").into_iter().enumerate() {
        let mut e = Vec::new();
        if let Some(spec) = SettingSpec::from_table(s, i, &mut e) {
            if !skeys.insert(spec.key.clone()) {
                e.push(format!("settings.{}: key lặp", spec.key));
            }
            for c in &spec.commands {
                if !commands.iter().any(|cmd| &cmd.id == c) {
                    e.push(format!("settings.{}: lệnh {c:?} không có trong [[commands]]", spec.key));
                }
            }
            // One command line cannot have two options with the same name.
            if let Some(other) = settings.iter().find(|o: &&SettingSpec| {
                o.option.eq_ignore_ascii_case(&spec.option)
                    && commands.iter().any(|c| o.applies_to(&c.id) && spec.applies_to(&c.id))
            }) {
                e.push(format!(
                    "settings.{}: option {:?} trùng với setting {}",
                    spec.key, spec.option, other.key
                ));
            }
            settings.push(spec);
        }
        r.errs.extend(e);
    }

    // [[assets]]
    let mut assets = Vec::new();
    for (i, a) in r.tables("assets").into_iter().enumerate() {
        let kind = a
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let path = a
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim_end_matches('/')
            .to_string();
        if kind.is_empty() || !kind.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            r.errs.push(format!(
                "assets[{i}]: kind phải là chữ thường/_ (vd gem_cut, ring_size_table)"
            ));
        }
        let exists = chk.files.is_none_or(|f| {
            f.contains(&path) || f.iter().any(|x| x.starts_with(&format!("{path}/")))
        });
        if !safe_rel_path(&path) || !exists {
            r.errs
                .push(format!("assets[{i}]: path {path:?} không có trong gói"));
        }
        assets.push(AssetSpec { kind, path });
    }
    if addon_type == AddonType::Library && assets.is_empty() {
        r.errs
            .push("library phải khai báo ít nhất một [[assets]]".into());
    }

    let exclude = match t.get("build").and_then(Value::as_table) {
        Some(b) => {
            let mut br = R {
                t: b,
                errs: Vec::new(),
            };
            let v = br.str_list("paths_exclude_pattern").unwrap_or_default();
            r.errs
                .extend(br.errs.into_iter().map(|e| format!("build.{e}")));
            v
        }
        None => Vec::new(),
    };

    if !r.errs.is_empty() {
        return Err(r.errs);
    }
    Ok(Manifest {
        schema_version,
        id,
        version,
        name,
        tagline,
        maintainer,
        addon_type,
        license,
        om9_version_min,
        om9_version_max,
        platforms,
        runtime,
        entry,
        permissions,
        tags,
        commands,
        settings,
        assets,
        exclude,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const GOOD: &str = r#"
schema_version = "1.0.0"
id = "gem_on_curve"
version = "0.2.0"
name = "Gem On Curve"
license = ["SPDX:LGPL-2.1-or-later"]
om9_version_min = "0.1.0"
runtime = "python"
entry = "__init__.py"

[[commands]]
id = "BuilderGemOnCrv"
handler = "gem_on_curve"
shortcut = "shift+ctrl+g"

[[settings]]
key = "gap"
type = "length"
default = 0.2
min = 0
max = 2
"#;

    fn cat() -> Catalog {
        Catalog::from_main_menu_ini(
            "[Menu1]\nName=Gems\nIcon1=BuilderGemOnCrv\nIcon2=BuilderGemOnSurface\n",
        )
        .unwrap()
    }

    #[test]
    fn good_manifest() {
        let files: BTreeSet<String> = ["__init__.py".to_string()].into();
        let c = cat();
        let m = parse(
            GOOD,
            &Check {
                files: Some(&files),
                catalog: Some(&c),
            },
        )
        .unwrap();
        assert_eq!(m.id, "gem_on_curve");
        assert_eq!(m.runtime, RuntimeKind::Python);
        assert_eq!(m.commands[0].shortcut.as_deref(), Some("Ctrl+Shift+G"));
        assert_eq!(m.platforms, vec!["any"]);
        assert_eq!(m.settings.len(), 1);
    }

    #[test]
    fn collects_all_errors() {
        let bad = GOOD
            .replace("id = \"gem_on_curve\"", "id = \"Gem-On\"")
            .replace(
                "license = [\"SPDX:LGPL-2.1-or-later\"]",
                "license = [\"LGPL\"]",
            )
            .replace("id = \"BuilderGemOnCrv\"", "id = \"NotInCatalog\"")
            .replace("default = 0.2", "default = 9");
        let files: BTreeSet<String> = BTreeSet::new();
        let c = cat();
        let errs = parse(
            &bad,
            &Check {
                files: Some(&files),
                catalog: Some(&c),
            },
        )
        .unwrap_err();
        let all = errs.join("\n");
        for needle in [
            "id không hợp lệ",
            "SPDX",
            "không có trong catalog",
            "entry",
            "default",
        ] {
            assert!(all.contains(needle), "thiếu lỗi {needle:?}:\n{all}");
        }
    }

    #[test]
    fn new_command_needs_group() {
        let c = cat();
        let src = GOOD.replace(
            "id = \"BuilderGemOnCrv\"",
            "id = \"OM9GemAutoFill\"\nnew = true",
        );
        let e = parse(
            &src,
            &Check {
                files: None,
                catalog: Some(&c),
            },
        )
        .unwrap_err();
        assert!(e[0].contains("group"), "{e:?}");
        let ok = src.replace("new = true", "new = true\ngroup = \"Gems\"");
        assert!(
            parse(
                &ok,
                &Check {
                    files: None,
                    catalog: Some(&c)
                }
            )
            .is_ok()
        );
    }

    #[test]
    fn library_rules() {
        let lib = r#"
schema_version = "1.0.0"
id = "ring_sizes"
version = "1.0.0"
name = "Ring Sizes"
type = "library"
license = ["SPDX:CC0-1.0"]
om9_version_min = "0.1.0"
[[assets]]
kind = "ring_size_table"
path = "tables"
"#;
        let files: BTreeSet<String> = ["tables/us.json".to_string()].into();
        let m = parse(
            lib,
            &Check {
                files: Some(&files),
                catalog: None,
            },
        )
        .unwrap();
        assert_eq!(m.runtime, RuntimeKind::Data);
        let e = parse(
            &lib.replace(
                "type = \"library\"",
                "type = \"library\"\nruntime = \"python\"",
            ),
            &Check::default(),
        )
        .unwrap_err();
        assert!(e.iter().any(|x| x.contains("library chỉ dùng")), "{e:?}");
    }

    #[test]
    fn permissions_need_reasons() {
        let src = GOOD.replace(
            "[[commands]]",
            "permissions = { files = \"\", camera = \"x\" }\n[[commands]]",
        );
        let e = parse(&src, &Check::default()).unwrap_err().join("\n");
        assert!(e.contains("quyền files"), "{e}");
        assert!(e.contains("\"camera\""), "{e}");
    }

    #[test]
    fn safe_paths() {
        for ok in ["a.py", "icons/x.svg"] {
            assert!(safe_rel_path(ok));
        }
        for bad in ["", "/a", "../a", "a/../b", "a\\b", "C:/x", "a//b", "./a"] {
            assert!(!safe_rel_path(bad), "{bad}");
        }
    }
}
