//! Kiểm thử tích hợp: inflate so với zlib, gói tốt/xấu, và toàn bộ vòng đời add-on trên đĩa thật.

use om9_addons::catalog::Catalog;
use om9_addons::host::{
    AddonStatus, CommandState, Host, HostConfig, InstallOptions, RepoKind, Startup, inspect_package,
};
use om9_addons::inflate::inflate;
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::{CommandInput, CommandOutput, LoadRequest, Runtime};
use om9_addons::semver::Version;
use om9_addons::settings::SettingValue;
use om9_addons::sha256::{hex, sha256};
use om9_addons::toml::{self, Value};
use om9_addons::zip::{ZipArchive, write_stored};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn fx() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn catalog() -> Catalog {
    Catalog::from_main_menu_ini(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Resources/menu/MainMenu.ini"),
        )
        .unwrap(),
    )
    .unwrap()
}

// ---------------- inflate ----------------

#[test]
fn inflate_matches_zlib_on_all_fixtures() {
    let idx = toml::parse(&fs::read_to_string(fx().join("inflate/index.toml")).unwrap()).unwrap();
    let mut n = 0;
    // khoá TOML "text.l6" bị hiểu là bảng lồng: text → l6
    for (data, variants) in &idx {
        for (variant, meta) in variants.as_table().unwrap() {
            let meta = meta.as_table().unwrap();
            let name = format!("{data}.{variant}");
            let comp = fs::read(fx().join(format!("inflate/{name}.deflate"))).unwrap();
            let Value::Int(len) = meta["len"] else {
                panic!()
            };
            let out = inflate(&comp, len as usize).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(out.len(), len as usize, "{name}");
            assert_eq!(
                hex(&sha256(&out)),
                meta["sha256"].as_str().unwrap(),
                "{name}"
            );
            // giới hạn đầu ra nhỏ hơn 1 byte phải bị chặn
            if len > 0 {
                assert!(
                    inflate(&comp, len as usize - 1).is_err(),
                    "{name}: không chặn vượt giới hạn"
                );
            }
            // cắt cụt luồng phải báo lỗi, không panic
            if comp.len() > 4 {
                let _ = inflate(&comp[..comp.len() / 2], len as usize);
            }
            n += 1;
        }
    }
    assert!(n >= 70);
}

#[test]
fn inflate_never_panics_on_noise() {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..3000 {
        let mut buf = vec![0u8; (x % 300) as usize];
        for b in buf.iter_mut() {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *b = x as u8;
        }
        let _ = inflate(&buf, 1 << 16);
    }
}

// ---------------- gói ----------------

#[test]
fn good_packages_pass() {
    let c = catalog();
    for name in ["gem_on_curve", "ring_sizes", "nested_folder"] {
        let data = fs::read(fx().join(format!("packages/good/{name}.zip"))).unwrap();
        let (m, files) =
            inspect_package(data, Some(&c)).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(files.iter().any(|(n, _)| n == "om9_addon.toml"), "{name}");
        assert!(
            files.iter().all(|(n, _)| !n.starts_with("gem_on_curve/")),
            "{name}: chưa bỏ thư mục bọc ngoài"
        );
        assert!(!m.id.is_empty());
    }
}

#[test]
fn bad_packages_rejected_with_reason() {
    let c = catalog();
    let exp = toml::parse(&fs::read_to_string(fx().join("packages/expect.toml")).unwrap()).unwrap();
    let bad = exp["bad"].as_table().unwrap();
    assert!(bad.len() >= 19);
    for (name, msg) in bad {
        let data = fs::read(fx().join(format!("packages/bad/{name}.zip"))).unwrap();
        let errs = inspect_package(data, Some(&c))
            .err()
            .unwrap_or_else(|| panic!("{name}: lẽ ra phải bị từ chối"));
        let all = errs.join(" | ");
        assert!(
            all.contains(msg.as_str().unwrap()),
            "{name}: mong lỗi chứa {msg:?}, nhận: {all}"
        );
    }
}

// ---------------- vòng đời ----------------

#[derive(Default)]
struct Calls {
    loads: Vec<String>,
    unloads: Vec<String>,
    settings: Vec<(String, BTreeMap<String, SettingValue>)>,
    fail_version: Option<String>,
    extra: bool,
}

struct Fake(Arc<Mutex<Calls>>);

impl Runtime for Fake {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        let mut c = self.0.lock().unwrap();
        let v = req.manifest.version.to_string();
        c.loads.push(format!("{}@{v}", req.id));
        if c.fail_version.as_deref() == Some(v.as_str()) {
            return Err("ImportError giả lập".into());
        }
        assert!(req.dir.join(req.manifest.entry.as_ref().unwrap()).is_file());
        let mut out: Vec<String> = req.manifest.commands.iter().map(|c| c.id.clone()).collect();
        if c.extra {
            out.push("BuilderHalo".into());
        }
        Ok(out)
    }
    fn unload(&mut self, id: &str) {
        self.0.lock().unwrap().unloads.push(id.into());
    }
    fn settings_changed(&mut self, id: &str, s: &BTreeMap<String, SettingValue>) {
        self.0.lock().unwrap().settings.push((id.into(), s.clone()));
    }
    fn invoke(&mut self, _id: &str, input: &CommandInput) -> Result<CommandOutput, String> {
        // Echo the settings the host passed, so a test can see overrides vs stored values.
        Ok(CommandOutput {
            meshes: Vec::new(),
            messages: input
                .settings
                .iter()
                .map(|(k, v)| format!("{k}={}", v.display()))
                .collect(),
        })
    }
}

struct Env {
    dir: PathBuf,
    calls: Arc<Mutex<Calls>>,
}

impl Env {
    fn new(tag: &str) -> Env {
        let dir = std::env::temp_dir().join(format!("om9-it-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Env {
            dir,
            calls: Arc::new(Mutex::new(Calls::default())),
        }
    }
    fn host_v(&self, v: Version) -> Host {
        let mut h = Host::new(HostConfig::new(v, catalog(), self.dir.join("config")));
        h.register_runtime(RuntimeKind::Python, Box::new(Fake(self.calls.clone())));
        h
    }
    fn host(&self) -> Host {
        let mut h = self.host_v(Version(0, 1, 0));
        h.startup(&Startup::default());
        h
    }
    /// biến thể của gói gem_on_curve với manifest đã sửa
    fn variant(&self, name: &str, edit: impl Fn(String) -> String) -> PathBuf {
        let z = ZipArchive::open(fs::read(fx().join("packages/good/gem_on_curve.zip")).unwrap())
            .unwrap();
        let files: Vec<(String, Vec<u8>)> = z
            .entries
            .iter()
            .map(|e| {
                let d = z.read(e).unwrap();
                if e.name == "om9_addon.toml" {
                    (
                        e.name.clone(),
                        edit(String::from_utf8(d).unwrap()).into_bytes(),
                    )
                } else {
                    (e.name.clone(), d)
                }
            })
            .collect();
        let p = self.dir.join(format!("{name}.zip"));
        fs::write(&p, write_stored(&files)).unwrap();
        p
    }
    fn good(&self, name: &str) -> PathBuf {
        fx().join(format!("packages/good/{name}.zip"))
    }
    fn prefs_text(&self) -> String {
        fs::read_to_string(self.dir.join("config/userprefs.toml")).unwrap_or_default()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn bump(v: &str) -> impl Fn(String) -> String + '_ {
    move |m: String| m.replacen("version = \"0.2.0\"", &format!("version = \"{v}\""), 1)
}

#[test]
fn install_enable_persist_and_menu() {
    let env = Env::new("basic");
    let mut h = env.host();
    let r = h
        .install_zip(
            &env.good("gem_on_curve"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    assert_eq!(r.sha256.len(), 64);
    assert!(h.user_repo().join("gem_on_curve/__init__.py").is_file());
    assert!(
        h.user_repo()
            .join("gem_on_curve/.om9_receipt.toml")
            .is_file()
    );
    assert_eq!(
        h.command_state("BuilderGemOnCrv"),
        CommandState::Bound {
            addon: "gem_on_curve".into(),
            shortcut: Some("Ctrl+Shift+G".into())
        }
    );
    assert_eq!(h.command_state("BuilderHalo"), CommandState::Unbound);
    let gems = h
        .menu_states()
        .into_iter()
        .find(|(g, _)| g == "Gems")
        .unwrap()
        .1;
    assert_eq!(gems.len(), 35);
    assert_eq!(
        gems.iter()
            .filter(|(_, s)| matches!(s, CommandState::Bound { .. }))
            .count(),
        2
    );
    assert!(env.prefs_text().contains("enabled = true"));

    // khởi động lại: tự bật lại
    drop(h);
    let h2 = env.host();
    assert!(h2.is_enabled("gem_on_curve"));
    assert_eq!(
        env.calls.lock().unwrap().loads,
        vec!["gem_on_curve@0.2.0", "gem_on_curve@0.2.0"]
    );
    assert_eq!(h2.search("gem count").len(), 0);
    assert_eq!(h2.search("countoncrv").len(), 1, "tìm theo mã lệnh");
}

#[test]
fn settings_validate_persist_and_notify() {
    let env = Env::new("settings");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        h.set_setting("gem_on_curve", "gap", "0,35").unwrap(),
        SettingValue::Float(0.35)
    );
    assert!(
        h.set_setting("gem_on_curve", "gap", "9")
            .unwrap_err()
            .contains("ngoài khoảng")
    );
    assert!(h.set_setting("gem_on_curve", "layout", "spiral").is_err());
    assert!(h.set_setting("gem_on_curve", "nope", "1").is_err());
    h.set_setting("gem_on_curve", "layout", "fixed_gap")
        .unwrap();
    {
        let c = env.calls.lock().unwrap();
        let last = &c.settings.last().unwrap().1;
        assert_eq!(last["gap"], SettingValue::Float(0.35));
        assert_eq!(last["layout"], SettingValue::Str("fixed_gap".into()));
    }
    drop(h);
    let h = env.host();
    let s: BTreeMap<String, SettingValue> = h
        .settings("gem_on_curve")
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.key, v))
        .collect();
    assert_eq!(s["gap"], SettingValue::Float(0.35));
    assert_eq!(s["diameter"], SettingValue::Float(1.5));
}

#[test]
fn builder_overrides_are_per_call_and_accepted_values_persist() {
    let env = Env::new("builder");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    let info = h.command_info("BuilderGemOnCrv").unwrap();
    assert_eq!(info.owner, "gem_on_curve");
    assert!(info.settings.iter().any(|(s, _)| s.key == "diameter"));
    assert!(h.command_info("BuilderHalo").is_none(), "unbound command");

    let over = BTreeMap::from([("diameter".to_string(), SettingValue::Float(2.0))]);
    let out = h
        .invoke_with("BuilderGemOnCrv", CommandInput::default(), &over)
        .unwrap();
    assert!(
        out.messages.contains(&"diameter=2".to_string()),
        "{:?}",
        out.messages
    );
    // The override was for that call only.
    let out = h
        .invoke("BuilderGemOnCrv", CommandInput::default())
        .unwrap();
    assert!(out.messages.contains(&"diameter=1.5".to_string()));
    // Overrides are validated against the schema.
    let bad = BTreeMap::from([("diameter".to_string(), SettingValue::Float(999.0))]);
    assert!(
        h.invoke_with("BuilderGemOnCrv", CommandInput::default(), &bad)
            .is_err()
    );
    let unknown = BTreeMap::from([("nope".to_string(), SettingValue::Float(1.0))]);
    assert!(
        h.invoke_with("BuilderGemOnCrv", CommandInput::default(), &unknown)
            .is_err()
    );

    // Accepting a builder stores all its values at once; an invalid one stores nothing.
    let mixed = BTreeMap::from([
        ("diameter".to_string(), SettingValue::Float(1.3)),
        ("gap".to_string(), SettingValue::Float(99.0)),
    ]);
    assert!(h.store_settings("gem_on_curve", &mixed).is_err());
    let ok = BTreeMap::from([
        ("diameter".to_string(), SettingValue::Float(1.3)),
        ("gap".to_string(), SettingValue::Float(0.25)),
    ]);
    h.store_settings("gem_on_curve", &ok).unwrap();
    drop(h);
    let h = env.host();
    let s: BTreeMap<String, SettingValue> = h
        .settings("gem_on_curve")
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k.key, v))
        .collect();
    assert_eq!(s["diameter"], SettingValue::Float(1.3));
    assert_eq!(s["gap"], SettingValue::Float(0.25));
}

#[test]
fn shortcuts_reserved_conflict_and_override() {
    let env = Env::new("keys");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        h.set_shortcut("BuilderGemOnCrv", Some("ctrl+z"))
            .unwrap_err()
            .contains("của OM9")
    );
    assert!(
        h.set_shortcut("BuilderGemCountOnCrv", Some("Ctrl+Shift+G"))
            .unwrap_err()
            .contains("đang dùng")
    );
    assert!(h.set_shortcut("BuilderGemOnCrv", Some("g")).is_err());
    assert_eq!(
        h.set_shortcut("BuilderGemOnCrv", Some("alt+ctrl+g"))
            .unwrap()
            .as_deref(),
        Some("Ctrl+Alt+G")
    );
    assert!(h.set_shortcut("NoSuchCode", Some("Ctrl+Alt+K")).is_err());
    // phím cũ được giải phóng
    assert!(
        h.set_shortcut("BuilderGemCountOnCrv", Some("Ctrl+Shift+G"))
            .is_ok()
    );
    drop(h);
    let mut h = env.host();
    assert_eq!(
        h.command_state("BuilderGemOnCrv"),
        CommandState::Bound {
            addon: "gem_on_curve".into(),
            shortcut: Some("Ctrl+Alt+G".into())
        }
    );
    h.set_shortcut("BuilderGemOnCrv", None).unwrap();
    drop(h);
    let h = env.host();
    assert_eq!(
        h.command_state("BuilderGemOnCrv"),
        CommandState::Bound {
            addon: "gem_on_curve".into(),
            shortcut: None
        },
        "bỏ phím phải giữ sau khởi động lại"
    );
}

#[test]
fn one_provider_per_command() {
    let env = Env::new("conflict");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    let other = env.variant("other", |m| {
        m.replace("id = \"gem_on_curve\"", "id = \"other_gems\"")
    });
    let r = h
        .install_zip(
            &other,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!r.enabled);
    assert!(
        r.warnings
            .iter()
            .any(|w| w.contains("đang do add-on gem_on_curve")),
        "{:?}",
        r.warnings
    );
    let row = h.list().into_iter().find(|r| r.id == "other_gems").unwrap();
    assert!(matches!(row.status, AddonStatus::Failed(_)));
    h.disable("gem_on_curve").unwrap();
    h.enable("other_gems", false).unwrap();
    assert_eq!(
        h.command_state("BuilderGemOnCrv"),
        CommandState::Bound {
            addon: "other_gems".into(),
            shortcut: Some("Ctrl+Shift+G".into())
        }
    );
}

#[test]
fn upgrade_migrates_settings_and_rolls_back_on_failure() {
    let env = Env::new("upgrade");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    h.set_setting("gem_on_curve", "gap", "0.35").unwrap();
    assert!(
        h.install_zip(&env.good("gem_on_curve"), &InstallOptions::default())
            .unwrap_err()[0]
            .contains("không mới hơn")
    );

    // 0.3.0: max của gap giảm còn 0.3 → 0.35 không còn hợp lệ → về mặc định, có cảnh báo
    let v3 = env.variant("v3", |m| bump("0.3.0")(m).replace("max = 5.0", "max = 0.3"));
    let r = h.install_zip(&v3, &InstallOptions::default()).unwrap();
    assert!(r.enabled, "đang bật thì nâng cấp xong vẫn bật");
    assert!(
        r.warnings.iter().any(|w| w.contains("không còn hợp lệ")),
        "{:?}",
        r.warnings
    );
    let gap = h
        .settings("gem_on_curve")
        .unwrap()
        .into_iter()
        .find(|(s, _)| s.key == "gap")
        .unwrap()
        .1;
    assert_eq!(gap, SettingValue::Float(0.2));

    // 0.4.0 lỗi khi register → khôi phục 0.3.0, vẫn bật
    env.calls.lock().unwrap().fail_version = Some("0.4.0".into());
    let v4 = env.variant("v4", bump("0.4.0"));
    let e = h.install_zip(&v4, &InstallOptions::default()).unwrap_err();
    assert!(e[0].contains("khôi phục bản cũ"), "{e:?}");
    let row = h
        .list()
        .into_iter()
        .find(|r| r.id == "gem_on_curve")
        .unwrap();
    assert_eq!(row.version.as_deref(), Some("0.3.0"));
    assert_eq!(row.status, AddonStatus::Enabled);
    assert!(
        fs::read_to_string(h.user_repo().join("gem_on_curve/om9_addon.toml"))
            .unwrap()
            .contains("0.3.0")
    );
    let leftovers: Vec<_> = fs::read_dir(h.user_repo())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with('.'))
        .collect();
    assert!(leftovers.is_empty(), "còn thư mục tạm {leftovers:?}");

    // hạ cấp cần --force
    let v1 = env.variant("v1", bump("0.1.0"));
    assert!(h.install_zip(&v1, &InstallOptions::default()).is_err());
    assert!(
        h.install_zip(
            &v1,
            &InstallOptions {
                force: true,
                ..Default::default()
            }
        )
        .is_ok()
    );
}

#[test]
fn permissions_require_consent_and_reask_on_upgrade() {
    let env = Env::new("perm");
    let mut h = env.host();
    let p = env.variant("perm", |m| {
        m.replace(
            "[[commands]]",
            "permissions = { files = \"Đọc thư viện đá tuỳ chỉnh\" }\n\n[[commands]]",
        )
    });
    let r = h
        .install_zip(
            &p,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!r.enabled);
    assert!(
        r.warnings[0].contains("cần người dùng cấp quyền: files (Đọc thư viện đá tuỳ chỉnh)"),
        "{:?}",
        r.warnings
    );
    h.enable("gem_on_curve", true).unwrap();
    assert!(h.prefs().addons["gem_on_curve"].granted.contains("files"));
    // bản mới thêm quyền network → đang bật mà chưa đồng ý → rollback
    let p2 = env.variant("perm2", |m| bump("0.3.0")(m).replace("[[commands]]", "permissions = { files = \"Đọc thư viện\", network = \"Kiểm tra cập nhật\" }\n\n[[commands]]"));
    let e = h.install_zip(&p2, &InstallOptions::default()).unwrap_err();
    assert!(
        e[0].contains("network") && e[0].contains("khôi phục"),
        "{e:?}"
    );
    assert!(h.is_enabled("gem_on_curve"));
    let r = h
        .install_zip(
            &p2,
            &InstallOptions {
                grant_permissions: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled);
}

#[test]
fn undeclared_registration_is_refused() {
    let env = Env::new("extra");
    let mut h = env.host();
    env.calls.lock().unwrap().extra = true;
    let r = h
        .install_zip(
            &env.good("gem_on_curve"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!r.enabled);
    assert!(r.warnings[0].contains("không khai báo"), "{:?}", r.warnings);
    assert_eq!(
        h.command_state("BuilderGemOnCrv"),
        CommandState::Unbound,
        "không được bind dở dang"
    );
    assert_eq!(
        env.calls.lock().unwrap().unloads,
        vec!["gem_on_curve"],
        "phải gọi unload để dọn"
    );
}

#[test]
fn safe_mode_and_session_only_do_not_touch_prefs() {
    let env = Env::new("safe");
    let mut h = env.host();
    h.install_zip(
        &env.good("gem_on_curve"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    h.install_zip(
        &env.good("ring_sizes"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    drop(h);
    let before = env.prefs_text();

    let mut s = env.host_v(Version(0, 1, 0));
    assert_eq!(
        s.startup(&Startup {
            safe_mode: true,
            only: None
        }),
        0
    );
    assert!(!s.is_enabled("gem_on_curve"));
    s.disable("gem_on_curve").unwrap(); // thao tác trong safe mode không ghi file
    drop(s);
    assert_eq!(env.prefs_text(), before);

    let mut o = env.host_v(Version(0, 1, 0));
    assert_eq!(
        o.startup(&Startup {
            safe_mode: false,
            only: Some(vec!["ring_sizes".into()])
        }),
        1
    );
    assert!(o.is_enabled("ring_sizes") && !o.is_enabled("gem_on_curve"));
    drop(o);
    assert_eq!(env.prefs_text(), before);
    assert!(
        env.host().is_enabled("gem_on_curve"),
        "phiên bình thường bật lại như cũ"
    );
}

#[test]
fn compatibility_checks() {
    let env = Env::new("compat");
    let mut old = env.host_v(Version(0, 0, 9));
    old.startup(&Startup::default());
    assert!(
        old.install_zip(&env.good("gem_on_curve"), &InstallOptions::default())
            .unwrap_err()[0]
            .contains("cần OM9 ≥ 0.1.0")
    );

    let mut h = env.host();
    let mac = env.variant("mac", |m| {
        m.replace("platforms = [\"any\"]", "platforms = [\"macos-arm64\"]")
    });
    let e = h.install_zip(&mac, &InstallOptions::default()).unwrap_err();
    if cfg!(not(all(target_os = "macos", target_arch = "aarch64"))) {
        assert!(e[0].contains("nền tảng"), "{e:?}");
    }
    let maxed = env.variant("max", |m| {
        m.replace(
            "om9_version_min = \"0.1.0\"",
            "om9_version_min = \"0.1.0\"\nom9_version_max = \"0.2.0\"",
        )
    });
    h.install_zip(&maxed, &InstallOptions::default()).unwrap();
    drop(h);
    let mut newer = env.host_v(Version(0, 2, 0));
    newer.startup(&Startup::default());
    let row = newer
        .list()
        .into_iter()
        .find(|r| r.id == "gem_on_curve")
        .unwrap();
    assert!(
        matches!(&row.status, AddonStatus::Incompatible(e) if e.contains("< 0.2.0")),
        "{:?}",
        row.status
    );

    // host không có runtime python
    let mut bare = Host::new(HostConfig::new(
        Version(0, 1, 0),
        catalog(),
        env.dir.join("config"),
    ));
    bare.startup(&Startup::default());
    assert!(
        matches!(bare.list().into_iter().find(|r| r.id == "gem_on_curve").unwrap().status, AddonStatus::Incompatible(e) if e.contains("runtime python"))
    );
}

#[test]
fn library_assets_dev_repo_uninstall() {
    let env = Env::new("lib");
    let mut h = env.host();
    h.install_zip(
        &env.good("ring_sizes"),
        &InstallOptions {
            enable: true,
            ..Default::default()
        },
    )
    .unwrap();
    let a = h.assets("ring_size_table");
    assert_eq!(a.len(), 1);
    assert!(a[0].1.join("ring_sizes.json").is_file());

    // kho dev che bản user
    h.install_zip(&env.good("gem_on_curve"), &InstallOptions::default())
        .unwrap();
    let dev = env.dir.join("dev");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/examples/gem_on_curve");
    fs::create_dir_all(dev.join("gem_on_curve/icons")).unwrap();
    for f in ["om9_addon.toml", "__init__.py", "icons/gem_on_crv.svg"] {
        fs::copy(src.join(f), dev.join("gem_on_curve").join(f)).unwrap();
    }
    h.add_dev_path(&dev).unwrap();
    let row = h
        .list()
        .into_iter()
        .find(|r| r.id == "gem_on_curve")
        .unwrap();
    assert_eq!(row.repo, RepoKind::Dev);
    assert_eq!(row.shadowed, vec![RepoKind::User]);
    assert!(h.uninstall("gem_on_curve").unwrap_err().contains("kho dev"));

    h.uninstall("ring_sizes").unwrap();
    assert!(!h.user_repo().join("ring_sizes").exists());
    assert!(!h.prefs().addons.contains_key("ring_sizes"));
    assert!(h.assets("ring_size_table").is_empty());
}

#[test]
fn broken_installs_are_listed_not_fatal() {
    let env = Env::new("broken");
    let mut h = env.host();
    let user = h.user_repo();
    fs::create_dir_all(user.join("bad_manifest")).unwrap();
    fs::write(user.join("bad_manifest/om9_addon.toml"), "id = [[").unwrap();
    fs::create_dir_all(user.join("wrong_name/")).unwrap();
    let ok = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../addons/ring_sizes/om9_addon.toml"),
    )
    .unwrap();
    fs::write(user.join("wrong_name/om9_addon.toml"), ok).unwrap();
    fs::create_dir_all(user.join("wrong_name/tables")).unwrap();
    fs::write(user.join("wrong_name/tables/x.json"), "{}").unwrap();
    h.discover();
    let rows = h.list();
    assert!(
        matches!(&rows.iter().find(|r| r.id == "bad_manifest").unwrap().status, AddonStatus::Invalid(e) if e[0].contains("cú pháp"))
    );
    assert!(
        matches!(&rows.iter().find(|r| r.id == "wrong_name").unwrap().status, AddonStatus::Invalid(e) if e[0].contains("khác id"))
    );
    assert!(h.enable("bad_manifest", false).is_err());

    // preferences hỏng → khôi phục, không crash
    fs::write(env.dir.join("config/userprefs.toml"), "[[[").unwrap();
    let h2 = env.host_v(Version(0, 1, 0));
    assert!(h2.log.iter().any(|l| l.contains("hỏng")));
}

#[test]
fn nested_folder_zip_installs() {
    let env = Env::new("nested");
    let mut h = env.host();
    let r = h
        .install_zip(
            &env.good("nested_folder"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.id, "gem_on_curve");
    assert!(h.user_repo().join("gem_on_curve/om9_addon.toml").is_file());
    assert!(!h.user_repo().join("gem_on_curve/gem_on_curve").exists());
}

#[test]
fn build_is_deterministic_and_excludes() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/examples/gem_on_curve");
    let tmp = std::env::temp_dir().join(format!("om9-build-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("src/__pycache__")).unwrap();
    fs::create_dir_all(tmp.join("src/tests")).unwrap();
    fs::create_dir_all(tmp.join("src/icons")).unwrap();
    for f in ["om9_addon.toml", "__init__.py", "icons/gem_on_crv.svg"] {
        fs::copy(src.join(f), tmp.join("src").join(f)).unwrap();
    }
    fs::write(tmp.join("src/__pycache__/x.pyc"), "x").unwrap();
    fs::write(tmp.join("src/tests/test_x.py"), "x").unwrap();
    fs::write(tmp.join("src/.DS_Store"), "x").unwrap();
    let c = catalog();
    let (_, a) = om9_addons::host::build_package(&tmp.join("src"), Some(&c)).unwrap();
    let (_, b) = om9_addons::host::build_package(&tmp.join("src"), Some(&c)).unwrap();
    assert_eq!(a, b);
    let names: Vec<String> = ZipArchive::open(a)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(
        names,
        vec!["__init__.py", "icons/gem_on_crv.svg", "om9_addon.toml"]
    );
    fs::remove_dir_all(&tmp).unwrap();
}
