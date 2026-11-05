//! Host add-on của OM9: kho (system/user/dev), cài từ zip, bật/tắt, gỡ, quyền,
//! settings, phím tắt, safe mode và trạng thái lệnh cho menu.

use crate::catalog::Catalog;
use crate::keymap;
use crate::manifest::{self, Check, CommandSpec, Manifest, RuntimeKind};
use crate::prefs::Prefs;
use crate::runtime::{CommandInput, CommandOutput, DataRuntime, LoadRequest, Runtime};
use crate::semver::Version;
use crate::settings::{self, SettingSpec, SettingValue};
use crate::sha256;
use crate::toml::{self, Table, Value};
use crate::zip::ZipArchive;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_ZIP_BYTES: u64 = 256 << 20;
pub const MAX_FILES: usize = 2000;
pub const MAX_UNPACKED: u64 = 200 << 20;
pub const MAX_RATIO: u64 = 200;
pub const RECEIPT: &str = ".om9_receipt.toml";
const WINDOWS_RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepoKind {
    /// Đi kèm bộ cài OM9, chỉ đọc.
    System,
    /// Người dùng cài từ zip.
    User,
    /// Thư mục phát triển trỏ thẳng tới mã nguồn (không copy), ưu tiên cao nhất.
    Dev,
}

impl RepoKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RepoKind::System => "system",
            RepoKind::User => "user",
            RepoKind::Dev => "dev",
        }
    }
}

pub fn current_platform() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else {
        "unknown"
    }
}

/// Thư mục cấu hình theo phiên bản major.minor (như Blender) để nhiều bản OM9 không đè nhau.
pub fn default_config_dir(v: Version) -> PathBuf {
    if let Some(p) = std::env::var_os("OM9_CONFIG_DIR") {
        return PathBuf::from(p);
    }
    let ver = format!("{}.{}", v.0, v.1);
    let home = || {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(home)
            .join("OpenMatrix9")
            .join(ver)
    } else if cfg!(target_os = "macos") {
        home()
            .join("Library/Application Support/OpenMatrix9")
            .join(ver)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".config"))
            .join("openmatrix9")
            .join(ver)
    }
}

pub struct HostConfig {
    pub om9_version: Version,
    pub platform: String,
    pub catalog: Catalog,
    pub reserved_shortcuts: Vec<String>,
    pub config_dir: PathBuf,
    pub system_dir: Option<PathBuf>,
}

impl HostConfig {
    pub fn new(om9_version: Version, catalog: Catalog, config_dir: PathBuf) -> HostConfig {
        HostConfig {
            om9_version,
            platform: current_platform().into(),
            catalog,
            reserved_shortcuts: keymap::default_reserved(),
            config_dir,
            system_dir: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Installed {
    manifest: Option<Manifest>,
    errors: Vec<String>,
    repo: RepoKind,
    dir: PathBuf,
    shadows: Vec<(RepoKind, PathBuf)>,
}

struct Active {
    runtime: RuntimeKind,
    commands: Vec<String>,
    assets: Vec<(String, PathBuf)>,
    settings: BTreeMap<String, SettingValue>,
}

#[derive(Debug, Clone, Default)]
pub struct Startup {
    /// Không bật add-on nào, không ghi đè preferences (giống `--factory-startup`).
    pub safe_mode: bool,
    /// Chỉ bật các add-on này trong phiên (giống `--addons`), không ghi preferences.
    pub only: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Cho phép cài lại cùng bản hoặc hạ phiên bản.
    pub force: bool,
    /// Người dùng đã đồng ý các quyền add-on yêu cầu.
    pub grant_permissions: bool,
    /// Bật ngay sau khi cài (mặc định của "Install from Disk" trong Blender).
    pub enable: bool,
}

#[derive(Debug, Clone)]
pub struct InstallReport {
    pub id: String,
    pub version: Version,
    pub enabled: bool,
    pub sha256: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddonStatus {
    Enabled,
    Disabled,
    Invalid(Vec<String>),
    Incompatible(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub owner: String,
    pub addon_name: String,
    pub command: CommandSpec,
    pub settings: Vec<(SettingSpec, SettingValue)>,
}

#[derive(Debug, Clone)]
pub struct AddonRow {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub tagline: Option<String>,
    pub repo: RepoKind,
    pub dir: PathBuf,
    pub status: AddonStatus,
    pub commands: Vec<String>,
    pub permissions: BTreeMap<String, String>,
    pub shadowed: Vec<RepoKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandState {
    /// Lệnh chạy được, do add-on này cung cấp.
    Bound {
        addon: String,
        shortcut: Option<String>,
    },
    /// Hiện trên menu nhưng vô hiệu (đúng quy tắc hiện tại của OM9).
    Unbound,
}

pub struct Host {
    cfg: HostConfig,
    prefs: Prefs,
    prefs_path: PathBuf,
    persist: bool,
    runtimes: BTreeMap<RuntimeKind, Box<dyn Runtime>>,
    installed: BTreeMap<String, Installed>,
    active: BTreeMap<String, Active>,
    bindings: BTreeMap<String, String>,
    shortcuts: BTreeMap<String, String>,
    last_error: BTreeMap<String, String>,
    pub log: Vec<String>,
}

fn walk(dir: &Path) -> BTreeSet<String> {
    fn rec(base: &Path, d: &Path, out: &mut BTreeSet<String>, depth: usize) {
        if depth > 16 || out.len() > 5000 {
            return;
        }
        let Ok(rd) = fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            let p = e.path();
            if ft.is_dir() {
                rec(base, &p, out, depth + 1);
            } else if let Ok(rel) = p.strip_prefix(base) {
                out.insert(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = BTreeSet::new();
    rec(dir, dir, &mut out, 0);
    out
}

/// Kiểm tra an toàn + manifest của một gói zip. Trả về manifest và nội dung file (đã kiểm CRC).
/// Nội dung file trong gói: (đường dẫn tương đối, byte).
pub type PackageFiles = Vec<(String, Vec<u8>)>;

pub fn inspect_package(
    data: Vec<u8>,
    catalog: Option<&Catalog>,
) -> Result<(Manifest, PackageFiles), Vec<String>> {
    let one = |e: String| vec![e];
    let z = ZipArchive::open(data).map_err(one)?;
    let files: Vec<_> = z.entries.iter().filter(|e| !e.is_dir()).collect();
    if files.len() > MAX_FILES {
        return Err(one(format!(
            "quá nhiều file ({} > {MAX_FILES})",
            files.len()
        )));
    }
    // Cho phép zip bọc một thư mục ngoài cùng (người dùng hay nén cả folder)
    let prefix = {
        let root_manifest = files.iter().any(|e| e.name == manifest::FILE_NAME);
        let first = files
            .first()
            .and_then(|e| e.name.split_once('/'))
            .map(|(a, _)| format!("{a}/"));
        match first {
            Some(p)
                if !root_manifest
                    && files.iter().all(|e| e.name.starts_with(&p))
                    && files
                        .iter()
                        .any(|e| e.name == format!("{p}{}", manifest::FILE_NAME)) =>
            {
                p
            }
            _ => String::new(),
        }
    };
    let mut total: u64 = 0;
    let mut names = BTreeSet::new();
    let mut lower = BTreeSet::new();
    let mut out = Vec::new();
    for e in z.entries.iter() {
        let raw = e.name.trim_end_matches('/');
        if !manifest::safe_rel_path(raw) {
            return Err(one(format!(
                "đường dẫn không an toàn trong zip: {:?}",
                e.name
            )));
        }
        if e.is_symlink() {
            return Err(one(format!("không cho phép symlink: {}", e.name)));
        }
        if raw.split('/').any(|c| {
            let stem = c.split('.').next().unwrap_or("").to_ascii_lowercase();
            WINDOWS_RESERVED.contains(&stem.as_str())
                || c.ends_with('.')
                || c.ends_with(' ')
                || c.contains([':', '*', '?', '"', '<', '>', '|'])
        }) {
            return Err(one(format!(
                "tên file không dùng được trên Windows: {}",
                e.name
            )));
        }
        if e.is_dir() {
            continue;
        }
        let name = e.name[prefix.len()..].to_string();
        if name == RECEIPT {
            return Err(one(format!("tên {RECEIPT} dành cho host")));
        }
        if !names.insert(name.clone()) {
            return Err(one(format!("file lặp trong zip: {name}")));
        }
        if !lower.insert(name.to_lowercase()) {
            return Err(one(format!(
                "hai file chỉ khác hoa/thường (xung đột trên Windows): {name}"
            )));
        }
        total += e.size;
        if total > MAX_UNPACKED {
            return Err(one(format!(
                "dung lượng giải nén vượt {} MB",
                MAX_UNPACKED >> 20
            )));
        }
        if e.size > 1 << 20 && (e.compressed == 0 || e.size / e.compressed > MAX_RATIO) {
            return Err(one(format!(
                "tỉ lệ nén bất thường (nghi zip bomb): {}",
                e.name
            )));
        }
        out.push((name, z.read(e).map_err(one)?));
    }
    let Some((_, mtext)) = out.iter().find(|(n, _)| n == manifest::FILE_NAME) else {
        return Err(one(format!("thiếu {} ở gốc gói", manifest::FILE_NAME)));
    };
    let text = String::from_utf8(mtext.clone())
        .map_err(|_| one(format!("{} không phải UTF-8", manifest::FILE_NAME)))?;
    let m = manifest::parse(
        &text,
        &Check {
            files: Some(&names),
            catalog,
        },
    )?;
    Ok((m, out))
}

/// Build gói từ thư mục nguồn: áp dụng loại trừ mặc định + paths_exclude_pattern, kiểm tra, ghi zip tất định.
pub fn build_package(
    src: &Path,
    catalog: Option<&Catalog>,
) -> Result<(Manifest, Vec<u8>), Vec<String>> {
    let mpath = src.join(manifest::FILE_NAME);
    let text = fs::read_to_string(&mpath)
        .map_err(|e| vec![format!("không đọc được {}: {e}", mpath.display())])?;
    let pre = toml::parse(&text)
        .map_err(|e| vec![format!("{} lỗi cú pháp, {e}", manifest::FILE_NAME)])?;
    let mut patterns: Vec<String> = crate::glob::DEFAULT_EXCLUDES
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(Value::Array(a)) = pre
        .get("build")
        .and_then(Value::as_table)
        .and_then(|b| b.get("paths_exclude_pattern"))
    {
        patterns.extend(a.iter().filter_map(|v| v.as_str().map(str::to_string)));
    }
    let names: BTreeSet<String> = walk(src)
        .into_iter()
        .filter(|p| p == manifest::FILE_NAME || !crate::glob::excluded(p, &patterns))
        .collect();
    let m = manifest::parse(
        &text,
        &Check {
            files: Some(&names),
            catalog,
        },
    )?;
    let mut files = Vec::new();
    for n in &names {
        files.push((
            n.clone(),
            fs::read(src.join(n)).map_err(|e| vec![format!("{n}: {e}")])?,
        ));
    }
    let zip = crate::zip::write_stored(&files);
    // tự kiểm tra gói vừa build bằng chính bộ cài
    inspect_package(zip.clone(), catalog)?;
    Ok((m, zip))
}

impl Host {
    pub fn new(cfg: HostConfig) -> Host {
        let prefs_path = cfg.config_dir.join("userprefs.toml");
        let (prefs, warn) = Prefs::load(&prefs_path);
        let mut runtimes: BTreeMap<RuntimeKind, Box<dyn Runtime>> = BTreeMap::new();
        runtimes.insert(RuntimeKind::Data, Box::new(DataRuntime));
        let mut h = Host {
            cfg,
            prefs,
            prefs_path,
            persist: true,
            runtimes,
            installed: BTreeMap::new(),
            active: BTreeMap::new(),
            bindings: BTreeMap::new(),
            shortcuts: BTreeMap::new(),
            last_error: BTreeMap::new(),
            log: Vec::new(),
        };
        if let Some(w) = warn {
            h.log.push(w);
        }
        h
    }

    pub fn register_runtime(&mut self, kind: RuntimeKind, rt: Box<dyn Runtime>) {
        self.runtimes.insert(kind, rt);
    }

    pub fn user_repo(&self) -> PathBuf {
        self.cfg.config_dir.join("extensions").join("user")
    }

    pub fn prefs(&self) -> &Prefs {
        &self.prefs
    }

    fn repos(&self) -> Vec<(RepoKind, PathBuf)> {
        let mut r = Vec::new();
        if let Some(s) = &self.cfg.system_dir {
            r.push((RepoKind::System, s.clone()));
        }
        r.push((RepoKind::User, self.user_repo()));
        for d in &self.prefs.dev_paths {
            r.push((RepoKind::Dev, PathBuf::from(d)));
        }
        r
    }

    fn save(&mut self) -> Result<(), String> {
        if !self.persist {
            return Ok(());
        }
        self.prefs
            .save(&self.prefs_path)
            .map_err(|e| format!("không ghi được preferences: {e}"))
    }

    /// Quét lại các kho. Thứ tự ưu tiên: dev > user > system (bản sau che bản trước).
    pub fn discover(&mut self) {
        let mut found: BTreeMap<String, Installed> = BTreeMap::new();
        for (kind, root) in self.repos() {
            let Ok(rd) = fs::read_dir(&root) else {
                continue;
            };
            let mut dirs: Vec<PathBuf> = rd
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .map(|e| e.path())
                .collect();
            dirs.sort();
            for dir in dirs {
                let dname = dir.file_name().unwrap().to_string_lossy().to_string();
                if dname.starts_with('.') {
                    continue;
                }
                let mpath = dir.join(manifest::FILE_NAME);
                if !mpath.is_file() {
                    continue;
                }
                let files = walk(&dir);
                let (manifest, errors) = match fs::read_to_string(&mpath) {
                    Err(e) => (None, vec![format!("không đọc được manifest: {e}")]),
                    Ok(t) => match manifest::parse(
                        &t,
                        &Check {
                            files: Some(&files),
                            catalog: Some(&self.cfg.catalog),
                        },
                    ) {
                        Ok(m) if m.id != dname => (
                            None,
                            vec![format!(
                                "thư mục {dname:?} khác id {:?} trong manifest",
                                m.id
                            )],
                        ),
                        Ok(m) => (Some(m), Vec::new()),
                        Err(e) => (None, e),
                    },
                };
                let mut inst = Installed {
                    manifest,
                    errors,
                    repo: kind,
                    dir: dir.clone(),
                    shadows: Vec::new(),
                };
                if let Some(prev) = found.remove(&dname) {
                    inst.shadows = prev.shadows;
                    inst.shadows.push((prev.repo, prev.dir));
                }
                found.insert(dname, inst);
            }
        }
        self.installed = found;
    }

    fn compat(&self, m: &Manifest) -> Result<(), String> {
        let v = self.cfg.om9_version;
        if v < m.om9_version_min {
            return Err(format!("cần OM9 ≥ {}, đang dùng {v}", m.om9_version_min));
        }
        if let Some(max) = m.om9_version_max
            && v >= max
        {
            return Err(format!("chỉ hỗ trợ OM9 < {max}, đang dùng {v}"));
        }
        if !m
            .platforms
            .iter()
            .any(|p| p == "any" || *p == self.cfg.platform)
        {
            return Err(format!(
                "không hỗ trợ nền tảng {} (gói dành cho {})",
                self.cfg.platform,
                m.platforms.join(", ")
            ));
        }
        if !self.runtimes.contains_key(&m.runtime) {
            return Err(format!(
                "bản OM9 này chưa có runtime {}",
                m.runtime.as_str()
            ));
        }
        Ok(())
    }

    /// Khởi động: quét kho rồi bật các add-on đã bật lần trước. Lỗi của một add-on không chặn add-on khác.
    pub fn startup(&mut self, st: &Startup) -> usize {
        self.persist = !(st.safe_mode || st.only.is_some());
        // Libraries moved aside by an uninstall or update are free now.
        let _ = fs::remove_dir_all(self.user_repo().join(TRASH));
        self.discover();
        if st.safe_mode {
            self.log
                .push("safe mode: không bật add-on nào (preferences giữ nguyên)".into());
            return 0;
        }
        let ids: Vec<String> = match &st.only {
            Some(v) => v.clone(),
            None => self
                .prefs
                .addons
                .iter()
                .filter(|(_, a)| a.enabled)
                .map(|(k, _)| k.clone())
                .collect(),
        };
        let mut n = 0;
        for id in ids {
            match self.enable_inner(&id, false) {
                Ok(w) => {
                    n += 1;
                    self.log.extend(w);
                }
                Err(e) => {
                    self.log.push(format!("{id}: {e}"));
                    self.last_error.insert(id, e);
                }
            }
        }
        let _ = self.save(); // lưu settings đã di trú
        n
    }

    fn bind_shortcut(&mut self, code: &str, s: &str) -> Result<String, String> {
        let n = keymap::normalize(s)?;
        if self.cfg.reserved_shortcuts.contains(&n) {
            return Err(format!("{n} là phím tắt của OM9"));
        }
        if let Some(other) = self.shortcuts.get(&n)
            && other != code
        {
            return Err(format!("{n} đang dùng cho {other}"));
        }
        self.shortcuts.insert(n.clone(), code.to_string());
        Ok(n)
    }

    fn effective_shortcut(&self, code: &str, default: Option<&String>) -> Option<String> {
        match self.prefs.keymap.get(code) {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s.clone()),
            None => default.cloned(),
        }
    }

    fn enable_inner(&mut self, id: &str, grant: bool) -> Result<Vec<String>, String> {
        if self.active.contains_key(id) {
            return Ok(Vec::new());
        }
        let inst = self
            .installed
            .get(id)
            .ok_or_else(|| format!("add-on {id:?} chưa được cài"))?;
        let m = inst
            .manifest
            .clone()
            .ok_or_else(|| format!("manifest lỗi: {}", inst.errors.join("; ")))?;
        let dir = inst.dir.clone();
        self.compat(&m)?;
        let mut warnings = Vec::new();

        let ap = self.prefs.addons.entry(id.to_string()).or_default();
        let need: Vec<(&String, &String)> = m
            .permissions
            .iter()
            .filter(|(k, _)| !ap.granted.contains(*k))
            .collect();
        if !need.is_empty() {
            if !grant {
                let list = need
                    .iter()
                    .map(|(k, r)| format!("{k} ({r})"))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!("cần người dùng cấp quyền: {list}"));
            }
            for (k, _) in need {
                ap.granted.insert(k.clone());
            }
        }
        let (vals, notes) = settings::resolve(&m.settings, &ap.settings);
        warnings.extend(notes.into_iter().map(|n| format!("{id}: {n}")));

        for c in &m.commands {
            if let Some(owner) = self.bindings.get(&c.id)
                && owner != id
            {
                return Err(format!(
                    "lệnh {} đang do add-on {owner} cung cấp; tắt add-on đó trước",
                    c.id
                ));
            }
        }

        let rt = self
            .runtimes
            .get_mut(&m.runtime)
            .ok_or_else(|| format!("chưa có runtime {}", m.runtime.as_str()))?;
        let registered = match rt.load(&LoadRequest {
            id,
            dir: &dir,
            manifest: &m,
            settings: &vals,
        }) {
            Ok(r) => r,
            Err(e) => {
                rt.unload(id);
                return Err(format!("register() lỗi: {e}"));
            }
        };
        let declared: BTreeSet<&str> = m.commands.iter().map(|c| c.id.as_str()).collect();
        let extra: Vec<&String> = registered
            .iter()
            .filter(|r| !declared.contains(r.as_str()))
            .collect();
        if !extra.is_empty() {
            rt.unload(id);
            return Err(format!(
                "add-on đăng ký lệnh không khai báo trong manifest: {}",
                extra
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let reg: BTreeSet<&str> = registered.iter().map(String::as_str).collect();
        for d in declared.difference(&reg) {
            warnings.push(format!(
                "{id}: khai báo nhưng không đăng ký lệnh {d} (lệnh giữ trạng thái vô hiệu)"
            ));
        }

        for code in &registered {
            self.bindings.insert(code.clone(), id.to_string());
        }
        for c in m.commands.iter().filter(|c| reg.contains(c.id.as_str())) {
            if let Some(s) = self.effective_shortcut(&c.id, c.shortcut.as_ref())
                && let Err(e) = self.bind_shortcut(&c.id, &s)
            {
                warnings.push(format!("{}: bỏ phím tắt ({e})", c.id));
            }
        }
        let assets = m
            .assets
            .iter()
            .map(|a| (a.kind.clone(), dir.join(&a.path)))
            .collect();
        self.prefs.addons.get_mut(id).unwrap().settings = vals.clone();
        self.active.insert(
            id.to_string(),
            Active {
                runtime: m.runtime,
                commands: registered,
                assets,
                settings: vals,
            },
        );
        self.last_error.remove(id);
        Ok(warnings)
    }

    /// Bật add-on (nút tick trong Preferences > Add-ons).
    pub fn enable(&mut self, id: &str, grant_permissions: bool) -> Result<Vec<String>, String> {
        let r = self.enable_inner(id, grant_permissions);
        match &r {
            Ok(_) => {
                self.prefs.addons.entry(id.to_string()).or_default().enabled = true;
                self.save()?;
            }
            Err(e) => {
                self.last_error.insert(id.to_string(), e.clone());
            }
        }
        r
    }

    fn disable_inner(&mut self, id: &str) {
        if let Some(a) = self.active.remove(id) {
            if let Some(rt) = self.runtimes.get_mut(&a.runtime) {
                rt.unload(id);
            }
            self.bindings.retain(|_, owner| owner != id);
            self.shortcuts.retain(|_, code| !a.commands.contains(code));
        }
    }

    pub fn disable(&mut self, id: &str) -> Result<(), String> {
        if !self.installed.contains_key(id) && !self.active.contains_key(id) {
            return Err(format!("add-on {id:?} chưa được cài"));
        }
        self.disable_inner(id);
        self.last_error.remove(id);
        if let Some(a) = self.prefs.addons.get_mut(id) {
            a.enabled = false;
        }
        self.save()
    }

    fn read_dir_version(dir: &Path) -> Option<Version> {
        let t = toml::parse(&fs::read_to_string(dir.join(manifest::FILE_NAME)).ok()?).ok()?;
        Version::parse(t.get("version")?.as_str()?).ok()
    }

    /// Cài gói zip vào kho user. Nguyên tử: lỗi ở bất kỳ bước nào giữ nguyên bản cũ (kể cả trạng thái bật).
    pub fn install_zip(
        &mut self,
        path: &Path,
        opts: &InstallOptions,
    ) -> Result<InstallReport, Vec<String>> {
        let one = |e: String| vec![e];
        let len = fs::metadata(path)
            .map_err(|e| one(format!("{}: {e}", path.display())))?
            .len();
        if len > MAX_ZIP_BYTES {
            return Err(one(format!("gói lớn hơn {} MB", MAX_ZIP_BYTES >> 20)));
        }
        let data = fs::read(path).map_err(|e| one(format!("{}: {e}", path.display())))?;
        let digest = sha256::hex(&sha256::sha256(&data));
        let (m, files) = inspect_package(data, Some(&self.cfg.catalog))?;
        self.compat(&m).map_err(one)?;

        let user = self.user_repo();
        fs::create_dir_all(&user)
            .map_err(|e| one(format!("không tạo được {}: {e}", user.display())))?;
        let final_dir = user.join(&m.id);
        if final_dir.exists()
            && !opts.force
            && let Some(old) = Self::read_dir_version(&final_dir)
            && m.version <= old
        {
            return Err(one(format!(
                "{} {old} đã cài; bản {} không mới hơn (dùng --force để cài lại/hạ cấp)",
                m.id, m.version
            )));
        }

        let pid = std::process::id();
        let stage = user.join(format!(".staging-{}-{pid}", m.id));
        let backup = user.join(format!(".old-{}-{pid}", m.id));
        let _ = fs::remove_dir_all(&stage);
        let _ = fs::remove_dir_all(&backup);
        let write_all = || -> std::io::Result<()> {
            for (name, bytes) in &files {
                let p = stage.join(name);
                fs::create_dir_all(p.parent().unwrap())?;
                fs::write(&p, bytes)?;
            }
            let mut r = Table::new();
            r.insert("id".into(), Value::Str(m.id.clone()));
            r.insert("version".into(), Value::Str(m.version.to_string()));
            r.insert("package_sha256".into(), Value::Str(digest.clone()));
            r.insert(
                "files".into(),
                Value::Array(files.iter().map(|(n, _)| Value::Str(n.clone())).collect()),
            );
            fs::write(stage.join(RECEIPT), toml::to_string(&r))
        };
        if let Err(e) = write_all() {
            let _ = fs::remove_dir_all(&stage);
            return Err(one(format!("giải nén lỗi: {e}")));
        }

        let was_active = self.active.contains_key(&m.id)
            && self
                .installed
                .get(&m.id)
                .is_some_and(|i| i.repo == RepoKind::User);
        if was_active {
            self.disable_inner(&m.id);
        }
        let mut had_old = final_dir.exists();
        let mut restart = false;
        if had_old && let Err(e) = fs::rename(&final_dir, &backup) {
            // Windows keeps a library that was loaded mapped until OM9 exits, and then the
            // folder cannot be renamed. Move the locked files aside (allowed for mapped files)
            // and drop the old folder: no rollback copy, and the new library runs after a
            // restart.
            match remove_dir_evicting(&final_dir, &user.join(TRASH)) {
                Ok(moved) => {
                    had_old = false;
                    restart = moved;
                }
                Err(_) => {
                    let _ = fs::remove_dir_all(&stage);
                    if was_active {
                        let _ = self.enable_inner(&m.id, false);
                    }
                    return Err(one(format!("không thay được bản cũ (file đang mở?): {e}")));
                }
            }
        }
        if let Err(e) = fs::rename(&stage, &final_dir) {
            if had_old {
                let _ = fs::rename(&backup, &final_dir);
            }
            let _ = fs::remove_dir_all(&stage);
            self.discover();
            if was_active {
                let _ = self.enable_inner(&m.id, false);
            }
            return Err(one(format!("không công bố được thư mục cài: {e}")));
        }
        self.discover();

        let mut warnings = Vec::new();
        if restart {
            warnings.push(format!(
                "{}: bản cũ còn nạp trong OM9; khởi động lại OM9 để dùng bản {}",
                m.id, m.version
            ));
        }
        if let Some(i) = self.installed.get(&m.id)
            && i.repo == RepoKind::Dev
        {
            warnings.push(format!(
                "{} đang bị bản dev ở {} che; bản vừa cài chỉ dùng khi bỏ dev path",
                m.id,
                i.dir.display()
            ));
        }
        let mut enabled = false;
        if was_active || opts.enable {
            match self.enable_inner(&m.id, opts.grant_permissions) {
                Ok(w) => {
                    warnings.extend(w);
                    enabled = true;
                    self.prefs.addons.entry(m.id.clone()).or_default().enabled = true;
                }
                Err(e) if was_active => {
                    // rollback toàn bộ về bản cũ
                    self.disable_inner(&m.id);
                    let _ = fs::remove_dir_all(&final_dir);
                    if had_old {
                        let _ = fs::rename(&backup, &final_dir);
                    }
                    self.discover();
                    let _ = self.enable_inner(&m.id, false);
                    return Err(one(format!(
                        "bản mới không bật được ({e}); đã khôi phục bản cũ"
                    )));
                }
                Err(e) => {
                    warnings.push(format!("đã cài nhưng chưa bật: {e}"));
                    self.last_error.insert(m.id.clone(), e);
                }
            }
        }
        let _ = fs::remove_dir_all(&backup);
        self.save().map_err(one)?;
        Ok(InstallReport {
            id: m.id.clone(),
            version: m.version,
            enabled,
            sha256: digest,
            warnings,
        })
    }

    /// Gỡ add-on trong kho user (bản system/dev không bị xoá).
    pub fn uninstall(&mut self, id: &str) -> Result<(), String> {
        let inst = self
            .installed
            .get(id)
            .ok_or_else(|| format!("add-on {id:?} chưa được cài"))?
            .clone();
        if inst.repo != RepoKind::User {
            return Err(format!(
                "{id} thuộc kho {}; chỉ gỡ được add-on trong kho user",
                inst.repo.as_str()
            ));
        }
        let cmds: Vec<String> = inst
            .manifest
            .as_ref()
            .map(|m| m.commands.iter().map(|c| c.id.clone()).collect())
            .unwrap_or_default();
        self.disable_inner(id);
        let trash = self.user_repo().join(TRASH);
        let moved = remove_dir_evicting(&inst.dir, &trash)
            .map_err(|e| format!("không xoá được {}: {e}", inst.dir.display()))?;
        if moved {
            self.log.push(format!(
                "{id}: thư viện còn nạp trong OM9; đã chuyển vào thùng rác, xoá hẳn khi khởi động lại"
            ));
        }
        self.prefs.addons.remove(id);
        self.prefs.keymap.retain(|k, _| !cmds.contains(k));
        self.last_error.remove(id);
        self.discover();
        self.save()
    }

    pub fn add_dev_path(&mut self, p: &Path) -> Result<(), String> {
        let s = p.to_string_lossy().to_string();
        if !p.is_dir() {
            return Err(format!("{s} không phải thư mục"));
        }
        if !self.prefs.dev_paths.contains(&s) {
            self.prefs.dev_paths.push(s);
        }
        self.discover();
        self.save()
    }

    /// Danh sách cho Preferences > Add-ons.
    pub fn list(&self) -> Vec<AddonRow> {
        let mut rows: Vec<AddonRow> = self
            .installed
            .iter()
            .map(|(id, i)| {
                let status = if self.active.contains_key(id) {
                    AddonStatus::Enabled
                } else if let Some(m) = &i.manifest {
                    match self.compat(m) {
                        Err(e) => AddonStatus::Incompatible(e),
                        Ok(()) => match self.last_error.get(id) {
                            Some(e) => AddonStatus::Failed(e.clone()),
                            None => AddonStatus::Disabled,
                        },
                    }
                } else {
                    AddonStatus::Invalid(i.errors.clone())
                };
                let m = i.manifest.as_ref();
                AddonRow {
                    id: id.clone(),
                    name: m.map(|m| m.name.clone()).unwrap_or_else(|| id.clone()),
                    version: m.map(|m| m.version.to_string()),
                    tagline: m.and_then(|m| m.tagline.clone()),
                    repo: i.repo,
                    dir: i.dir.clone(),
                    status,
                    commands: m
                        .map(|m| m.commands.iter().map(|c| c.id.clone()).collect())
                        .unwrap_or_default(),
                    permissions: m.map(|m| m.permissions.clone()).unwrap_or_default(),
                    shadowed: i.shadows.iter().map(|(k, _)| *k).collect(),
                }
            })
            .collect();
        rows.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        rows
    }

    /// Lọc kiểu ô tìm kiếm của Blender: theo tên, id, tagline hoặc mã lệnh.
    pub fn search(&self, q: &str) -> Vec<AddonRow> {
        let q = q.trim().to_lowercase();
        self.list()
            .into_iter()
            .filter(|r| {
                q.is_empty()
                    || r.name.to_lowercase().contains(&q)
                    || r.id.contains(&q)
                    || r.tagline
                        .as_deref()
                        .is_some_and(|t| t.to_lowercase().contains(&q))
                    || r.commands.iter().any(|c| c.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn command_state(&self, code: &str) -> CommandState {
        match self.bindings.get(code) {
            Some(a) => CommandState::Bound {
                addon: a.clone(),
                shortcut: self
                    .shortcuts
                    .iter()
                    .find(|(_, c)| *c == code)
                    .map(|(s, _)| s.clone()),
            },
            None => CommandState::Unbound,
        }
    }

    /// Trạng thái từng lệnh theo nhóm menu — OM9 dùng để bật/xám icon.
    pub fn menu_states(&self) -> Vec<(String, Vec<(String, CommandState)>)> {
        self.cfg
            .catalog
            .groups
            .iter()
            .map(|(g, codes)| {
                (
                    g.clone(),
                    codes
                        .iter()
                        .map(|c| (c.clone(), self.command_state(c)))
                        .collect(),
                )
            })
            .collect()
    }

    /// Settings của một add-on (schema + giá trị hiệu lực) để vẽ panel.
    pub fn settings(&self, id: &str) -> Result<Vec<(SettingSpec, SettingValue)>, String> {
        let m = self
            .installed
            .get(id)
            .and_then(|i| i.manifest.as_ref())
            .ok_or_else(|| format!("add-on {id:?} chưa được cài hoặc manifest lỗi"))?;
        let stored = self
            .prefs
            .addons
            .get(id)
            .map(|a| a.settings.clone())
            .unwrap_or_default();
        let (vals, _) = settings::resolve(&m.settings, &stored);
        Ok(m.settings
            .iter()
            .map(|s| (s.clone(), vals[&s.key].clone()))
            .collect())
    }

    pub fn set_setting(&mut self, id: &str, key: &str, raw: &str) -> Result<SettingValue, String> {
        let specs = self
            .installed
            .get(id)
            .and_then(|i| i.manifest.as_ref())
            .map(|m| m.settings.clone())
            .ok_or_else(|| format!("add-on {id:?} chưa được cài"))?;
        let spec = specs
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| format!("{id} không có setting {key:?}"))?;
        let v = spec
            .parse_user(raw)
            .map_err(|e| format!("{}: {e}", spec.label))?;
        self.prefs
            .addons
            .entry(id.to_string())
            .or_default()
            .settings
            .insert(key.to_string(), v.clone());
        if let Some(a) = self.active.get_mut(id) {
            a.settings.insert(key.to_string(), v.clone());
            let s = a.settings.clone();
            let rt = a.runtime;
            if let Some(r) = self.runtimes.get_mut(&rt) {
                r.settings_changed(id, &s);
            }
        }
        self.save()?;
        Ok(v)
    }

    /// Đặt phím tắt cho mã lệnh. `None` = bỏ phím (kể cả phím mặc định của add-on).
    pub fn set_shortcut(
        &mut self,
        code: &str,
        keys: Option<&str>,
    ) -> Result<Option<String>, String> {
        let known = self.cfg.catalog.codes.contains(code)
            || self
                .installed
                .values()
                .filter_map(|i| i.manifest.as_ref())
                .any(|m| m.commands.iter().any(|c| c.id == code));
        if !known {
            return Err(format!(
                "mã lệnh {code:?} không có trong catalog hay add-on nào"
            ));
        }
        let norm = match keys {
            Some(k) => {
                let n = keymap::normalize(k)?;
                if self.cfg.reserved_shortcuts.contains(&n) {
                    return Err(format!("{n} là phím tắt của OM9"));
                }
                if let Some(other) = self.shortcuts.get(&n).filter(|c| *c != code) {
                    return Err(format!("{n} đang dùng cho {other}"));
                }
                Some(n)
            }
            None => None,
        };
        self.shortcuts.retain(|_, c| c != code);
        if let Some(n) = &norm
            && self.bindings.contains_key(code)
        {
            self.shortcuts.insert(n.clone(), code.to_string());
        }
        self.prefs
            .keymap
            .insert(code.to_string(), norm.clone().unwrap_or_default());
        self.save()?;
        Ok(norm)
    }

    /// Assets đang hoạt động theo loại (vd "gem_cut", "ring_size_table").
    pub fn assets(&self, kind: &str) -> Vec<(String, PathBuf)> {
        self.active
            .iter()
            .flat_map(|(id, a)| {
                a.assets
                    .iter()
                    .filter(|(k, _)| k == kind)
                    .map(move |(_, p)| (id.clone(), p.clone()))
            })
            .collect()
    }

    /// Bảng dịch giao diện từ các add-on đang bật có asset `lang` (thư mục chứa `vi.toml`).
    /// Ngôn ngữ lấy theo setting `language` của add-on đó (mặc định Tiếng Việt); một add-on
    /// chọn English thì mọi chữ giữ nguyên. Lỗi của file dịch được trả về để hiện cho người dùng.
    pub fn translator(&self) -> (crate::i18n::Tr, Vec<String>) {
        use crate::i18n::{Lang, Tr};
        let mut tr: Option<Tr> = None;
        let mut errs = Vec::new();
        for (id, dir) in self.assets("lang") {
            let lang = self
                .settings(&id)
                .ok()
                .and_then(|s| {
                    s.into_iter()
                        .find_map(|(spec, v)| match (spec.key.as_str(), v) {
                            ("language", SettingValue::Str(code)) => Lang::parse(&code),
                            _ => None,
                        })
                })
                .unwrap_or_default();
            if lang == Lang::En {
                return (Tr::english(), errs);
            }
            let file = dir.join(format!("{}.toml", lang.code()));
            let parsed = fs::read_to_string(&file)
                .map_err(|e| vec![e.to_string()])
                .and_then(|text| Tr::parse(&text, lang));
            match parsed {
                Ok(t) => match tr.as_mut() {
                    Some(all) => all.merge(t),
                    None => tr = Some(t),
                },
                Err(es) => errs.extend(
                    es.into_iter()
                        .map(|e| format!("{id} lang/{}.toml: {e}", lang.code())),
                ),
            }
        }
        (tr.unwrap_or_else(Tr::english), errs)
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        self.active.contains_key(id)
    }

    /// Phím tắt đang hiệu lực: (phím đã chuẩn hoá, mã lệnh).
    pub fn shortcuts(&self) -> Vec<(String, String)> {
        self.shortcuts
            .iter()
            .map(|(k, c)| (k.clone(), c.clone()))
            .collect()
    }

    /// Add-on đang cung cấp mã lệnh này (nếu có).
    pub fn owner_of(&self, code: &str) -> Option<&str> {
        self.bindings.get(code).map(String::as_str)
    }

    /// Mọi mã lệnh đang chạy được: (mã, add-on, nhãn hiển thị).
    pub fn bound_commands(&self) -> Vec<(String, String, String)> {
        self.bindings
            .iter()
            .map(|(code, owner)| {
                let label = self
                    .installed
                    .get(owner)
                    .and_then(|i| i.manifest.as_ref())
                    .and_then(|m| m.commands.iter().find(|c| &c.id == code))
                    .map(|c| c.label.clone())
                    .unwrap_or_else(|| code.clone());
                (code.clone(), owner.clone(), label)
            })
            .collect()
    }

    /// Chạy một lệnh: host điền mã lệnh và settings hiệu lực của add-on sở hữu,
    /// runtime chạy, rồi kết quả được kiểm tra lại trước khi trả về.
    pub fn invoke(&mut self, code: &str, input: CommandInput) -> Result<CommandOutput, String> {
        self.invoke_with(code, input, &BTreeMap::new())
    }

    /// One step of an interactive command (`interactive = true`): the JSON event goes to the
    /// owner's runtime unchanged and its JSON reply comes back unchanged. The app parses both.
    pub fn session(&mut self, code: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        let owner = self
            .bindings
            .get(code)
            .cloned()
            .ok_or_else(|| format!("lệnh {code} chưa có add-on nào cung cấp"))?;
        let kind = self
            .active
            .get(&owner)
            .ok_or_else(|| format!("add-on {owner} không còn bật"))?
            .runtime;
        let rt = self
            .runtimes
            .get_mut(&kind)
            .ok_or_else(|| format!("chưa có runtime {}", kind.as_str()))?;
        let reply = rt.session(&owner, event)?;
        if reply.len() > crate::runtime::MAX_SESSION_JSON {
            return Err(format!("{owner}: trả lời quá lớn"));
        }
        Ok(reply)
    }

    /// A bound command's declaration and its owner's settings (schema + effective values),
    /// for the builder (command-line options and builder panel).
    pub fn command_info(&self, code: &str) -> Option<CommandInfo> {
        let owner = self.bindings.get(code)?;
        let manifest = self.installed.get(owner)?.manifest.as_ref()?;
        let command = manifest.commands.iter().find(|c| c.id == code)?.clone();
        let settings = self
            .settings(owner)
            .ok()?
            .into_iter()
            .filter(|(spec, _)| spec.applies_to(code))
            .collect();
        Some(CommandInfo {
            owner: owner.clone(),
            addon_name: manifest.name.clone(),
            command,
            settings,
        })
    }

    /// Persist several setting values of one add-on at once (one write of `userprefs.toml`),
    /// e.g. the values a builder was accepted with. Every value is validated first; nothing is
    /// stored if one is invalid.
    pub fn store_settings(
        &mut self,
        id: &str,
        values: &BTreeMap<String, SettingValue>,
    ) -> Result<(), String> {
        let specs = self
            .installed
            .get(id)
            .and_then(|i| i.manifest.as_ref())
            .map(|m| m.settings.clone())
            .ok_or_else(|| format!("add-on {id:?} chưa được cài"))?;
        let mut checked = Vec::new();
        for (key, v) in values {
            let spec = specs
                .iter()
                .find(|s| &s.key == key)
                .ok_or_else(|| format!("{id} không có setting {key:?}"))?;
            checked.push((
                key.clone(),
                spec.validate(v)
                    .map_err(|e| format!("{}: {e}", spec.label))?,
            ));
        }
        let entry = self.prefs.addons.entry(id.to_string()).or_default();
        for (k, v) in &checked {
            entry.settings.insert(k.clone(), v.clone());
        }
        if let Some(a) = self.active.get_mut(id) {
            for (k, v) in checked {
                a.settings.insert(k, v);
            }
            let s = a.settings.clone();
            let rt = a.runtime;
            if let Some(r) = self.runtimes.get_mut(&rt) {
                r.settings_changed(id, &s);
            }
        }
        self.save()
    }

    /// Like [`Host::invoke`], with some settings replaced for this call only (a builder
    /// previewing values that are not stored yet). Overrides are validated against the schema.
    pub fn invoke_with(
        &mut self,
        code: &str,
        mut input: CommandInput,
        overrides: &BTreeMap<String, SettingValue>,
    ) -> Result<CommandOutput, String> {
        let owner = self
            .bindings
            .get(code)
            .cloned()
            .ok_or_else(|| format!("lệnh {code} chưa có add-on nào cung cấp"))?;
        let active = self
            .active
            .get(&owner)
            .ok_or_else(|| format!("add-on {owner} không còn bật"))?;
        input.command = code.to_owned();
        input.settings = active.settings.clone();
        if !overrides.is_empty() {
            let specs = self
                .installed
                .get(&owner)
                .and_then(|i| i.manifest.as_ref())
                .map(|m| m.settings.clone())
                .unwrap_or_default();
            for (key, v) in overrides {
                let spec = specs
                    .iter()
                    .find(|s| &s.key == key)
                    .ok_or_else(|| format!("{owner} không có setting {key:?}"))?;
                let v = spec
                    .validate(v)
                    .map_err(|e| format!("{}: {e}", spec.label))?;
                input.settings.insert(key.clone(), v);
            }
        }
        let kind = active.runtime;
        let rt = self
            .runtimes
            .get_mut(&kind)
            .ok_or_else(|| format!("chưa có runtime {}", kind.as_str()))?;
        let out = rt
            .invoke(&owner, &input)
            .map_err(|e| format!("{owner}: {e}"))?;
        out.validate().map_err(|e| format!("{owner}: {e}"))?;
        Ok(out)
    }
}

/// Folder (in the user repository, hidden from discovery) for files that could not be deleted.
const TRASH: &str = ".trash";

/// Remove `dir`. Files that cannot be deleted - on Windows a library loaded by OM9 stays mapped
/// until the program exits - are renamed into `trash` (Windows allows renaming a mapped file)
/// and the folder is removed after them. `Ok(true)` when something had to be moved.
fn remove_dir_evicting(dir: &Path, trash: &Path) -> std::io::Result<bool> {
    if fs::remove_dir_all(dir).is_ok() || !dir.exists() {
        return Ok(false);
    }
    fn files(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    files(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }
    let mut all = Vec::new();
    files(dir, &mut all);
    let mut moved = false;
    for (k, f) in all.iter().enumerate() {
        if fs::remove_file(f).is_ok() {
            continue;
        }
        fs::create_dir_all(trash)?;
        let name = f
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        fs::rename(
            f,
            trash.join(format!("{}-{stamp}-{k}-{name}", std::process::id())),
        )?;
        moved = true;
    }
    fs::remove_dir_all(dir)?;
    Ok(moved)
}
