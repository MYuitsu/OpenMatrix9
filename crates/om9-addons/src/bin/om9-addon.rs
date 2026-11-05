//! om9-addon — công cụ dòng lệnh quản lý add-on OM9 (tương đương `blender --command extension ...`).

use om9_addons::catalog::Catalog;
use om9_addons::host::{
    self, AddonStatus, CommandState, Host, HostConfig, InstallOptions, Startup, default_config_dir,
};
use om9_addons::manifest::RuntimeKind;
use om9_addons::runtime::DeclareOnlyRuntime;
use om9_addons::semver::Version;
use om9_addons::settings::Kind;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "om9-addon [--config DIR] [--system DIR] [--catalog MainMenu.ini] [--om9-version X.Y.Z] [--declare-only] <lệnh>

  build <thư_mục> [-o gói.zip]     đóng gói (tất định) và tự kiểm tra
  validate <gói.zip>               kiểm tra an toàn + manifest
  install <gói.zip> [--enable] [--yes] [--force]
  enable <id> [--yes]              --yes = đồng ý các quyền add-on yêu cầu
  disable <id>
  uninstall <id>
  list [từ_khoá]
  settings <id>
  set <id> <key> <giá_trị>
  key <MÃ_LỆNH> <phím|none>
  menu [nhóm]                      trạng thái lệnh theo nhóm menu
  dev-add <thư_mục>                thêm kho dev (dùng mã nguồn tại chỗ)

--declare-only: dùng runtime giả cho python/wasm/native (chỉ kiểm tra entry và bind lệnh đã khai báo, không chạy mã).";

struct Args {
    rest: Vec<String>,
}

impl Args {
    fn flag(&mut self, f: &str) -> bool {
        if let Some(i) = self.rest.iter().position(|a| a == f) {
            self.rest.remove(i);
            true
        } else {
            false
        }
    }
    fn opt(&mut self, f: &str) -> Result<Option<String>, String> {
        if let Some(i) = self.rest.iter().position(|a| a == f) {
            if i + 1 >= self.rest.len() {
                return Err(format!("{f} cần giá trị"));
            }
            let v = self.rest.remove(i + 1);
            self.rest.remove(i);
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
    fn pos(&self, i: usize, what: &str) -> Result<String, String> {
        self.rest
            .get(i)
            .cloned()
            .ok_or_else(|| format!("thiếu {what}\n\n{USAGE}"))
    }
}

fn status_text(s: &AddonStatus) -> String {
    match s {
        AddonStatus::Enabled => "BẬT".into(),
        AddonStatus::Disabled => "tắt".into(),
        AddonStatus::Invalid(e) => format!("LỖI manifest: {}", e.join("; ")),
        AddonStatus::Incompatible(e) => format!("không tương thích: {e}"),
        AddonStatus::Failed(e) => format!("bật thất bại: {e}"),
    }
}

fn run() -> Result<(), String> {
    let mut a = Args {
        rest: std::env::args().skip(1).collect(),
    };
    if a.rest.is_empty() || a.flag("-h") || a.flag("--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let ver = Version::parse(&a.opt("--om9-version")?.unwrap_or_else(|| "0.1.0".into()))?;
    let catalog = match a
        .opt("--catalog")?
        .or_else(|| std::env::var("OM9_CATALOG").ok())
    {
        Some(p) => Some(Catalog::from_main_menu_ini(
            &std::fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"))?,
        )?),
        None => None,
    };
    let config = a
        .opt("--config")?
        .map(PathBuf::from)
        .unwrap_or_else(|| default_config_dir(ver));
    let system = a.opt("--system")?.map(PathBuf::from);
    let declare_only = a.flag("--declare-only");
    let cmd = a.pos(0, "lệnh")?;

    match cmd.as_str() {
        "build" => {
            let out = a.opt("-o")?;
            let src = PathBuf::from(a.pos(1, "thư mục nguồn")?);
            let (m, zip) = host::build_package(&src, catalog.as_ref()).map_err(|e| e.join("\n"))?;
            let out = out
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(format!("{}-{}.zip", m.id, m.version)));
            std::fs::write(&out, &zip).map_err(|e| format!("{}: {e}", out.display()))?;
            println!(
                "Đã tạo {} ({} byte, sha256 {})",
                out.display(),
                zip.len(),
                om9_addons::sha256::hex(&om9_addons::sha256::sha256(&zip))
            );
            return Ok(());
        }
        "validate" => {
            let p = a.pos(1, "gói zip")?;
            let data = std::fs::read(&p).map_err(|e| format!("{p}: {e}"))?;
            let (m, files) =
                host::inspect_package(data, catalog.as_ref()).map_err(|e| e.join("\n"))?;
            println!(
                "OK: {} {} [{} / {}] {} file, {} lệnh, {} setting, quyền: {}",
                m.id,
                m.version,
                if m.addon_type == om9_addons::manifest::AddonType::Library {
                    "library"
                } else {
                    "add-on"
                },
                m.runtime.as_str(),
                files.len(),
                m.commands.len(),
                m.settings.len(),
                if m.permissions.is_empty() {
                    "không".to_string()
                } else {
                    m.permissions.keys().cloned().collect::<Vec<_>>().join(", ")
                }
            );
            return Ok(());
        }
        _ => {}
    }

    let mut cfg = HostConfig::new(ver, catalog.unwrap_or_default(), config);
    cfg.system_dir = system;
    let mut h = Host::new(cfg);
    if declare_only {
        for k in [RuntimeKind::Python, RuntimeKind::Wasm, RuntimeKind::Native] {
            h.register_runtime(k, Box::new(DeclareOnlyRuntime));
        }
    } else {
        // Built with `--features native`, `enable` really loads the add-on library and checks
        // its ABI and command table, exactly as OM9 does.
        #[cfg(feature = "native")]
        h.register_runtime(
            RuntimeKind::Native,
            Box::new(om9_addons::native::NativeRuntime::new()),
        );
    }
    h.startup(&Startup::default());
    let flush = |h: &mut Host| {
        for l in h.log.drain(..) {
            eprintln!("· {l}");
        }
    };

    let res = match cmd.as_str() {
        "install" => {
            let opts = InstallOptions {
                force: a.flag("--force"),
                grant_permissions: a.flag("--yes"),
                enable: a.flag("--enable"),
            };
            let p = PathBuf::from(a.pos(1, "gói zip")?);
            match h.install_zip(&p, &opts) {
                Ok(r) => {
                    println!(
                        "Đã cài {} {}{} vào {}",
                        r.id,
                        r.version,
                        if r.enabled { " (đã bật)" } else { "" },
                        h.user_repo().join(&r.id).display()
                    );
                    for w in &r.warnings {
                        println!("  ! {w}");
                    }
                    // `--enable` that could not enable (library fails to load, ABI mismatch)
                    // is a failure for scripts, not a warning.
                    if opts.enable && !r.enabled {
                        Err(format!("{}: đã cài nhưng không bật được", r.id))
                    } else {
                        Ok(())
                    }
                }
                Err(e) => Err(e.join("\n")),
            }
        }
        "enable" => {
            let grant = a.flag("--yes");
            let id = a.pos(1, "id")?;
            h.enable(&id, grant).map(|w| {
                println!("Đã bật {id}");
                for x in w {
                    println!("  ! {x}");
                }
            })
        }
        "disable" => {
            let id = a.pos(1, "id")?;
            h.disable(&id).map(|_| println!("Đã tắt {id}"))
        }
        "uninstall" => {
            let id = a.pos(1, "id")?;
            h.uninstall(&id).map(|_| println!("Đã gỡ {id}"))
        }
        "list" => {
            let q = a.rest.get(1).cloned().unwrap_or_default();
            let rows = h.search(&q);
            if rows.is_empty() {
                println!("(chưa có add-on nào)");
            }
            for r in rows {
                println!(
                    "{:22} {:8} {:6} {}",
                    r.id,
                    r.version.unwrap_or_else(|| "?".into()),
                    r.repo.as_str(),
                    status_text(&r.status)
                );
                if !r.commands.is_empty() {
                    println!("{:22} lệnh: {}", "", r.commands.join(", "));
                }
                if !r.permissions.is_empty() {
                    println!(
                        "{:22} quyền: {}",
                        "",
                        r.permissions
                            .iter()
                            .map(|(k, v)| format!("{k} ({v})"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                if !r.shadowed.is_empty() {
                    println!(
                        "{:22} che bản: {}",
                        "",
                        r.shadowed
                            .iter()
                            .map(|k| k.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
            Ok(())
        }
        "settings" => {
            let id = a.pos(1, "id")?;
            h.settings(&id).map(|v| {
                if v.is_empty() {
                    println!("{id} không có setting");
                }
                for (s, val) in v {
                    let hint = match &s.kind {
                        Kind::Float { min, max, unit, .. } => format!(
                            "[{} … {}]{}",
                            min.map_or("-∞".into(), |x| x.to_string()),
                            max.map_or("∞".into(), |x| x.to_string()),
                            unit.suffix()
                        ),
                        Kind::Int { min, max } => format!(
                            "[{} … {}]",
                            min.map_or("-∞".into(), |x| x.to_string()),
                            max.map_or("∞".into(), |x| x.to_string())
                        ),
                        Kind::Enum { choices } => choices.join(" | "),
                        _ => String::new(),
                    };
                    println!("{:16} = {:12} {}  {}", s.key, val.display(), s.label, hint);
                }
            })
        }
        "set" => {
            let (id, key, val) = (a.pos(1, "id")?, a.pos(2, "key")?, a.pos(3, "giá trị")?);
            h.set_setting(&id, &key, &val)
                .map(|v| println!("{id}.{key} = {}", v.display()))
        }
        "key" => {
            let (code, k) = (a.pos(1, "mã lệnh")?, a.pos(2, "phím")?);
            let keys = if k.eq_ignore_ascii_case("none") {
                None
            } else {
                Some(k.as_str())
            };
            h.set_shortcut(&code, keys)
                .map(|n| println!("{code}: {}", n.unwrap_or_else(|| "(không phím tắt)".into())))
        }
        "menu" => {
            let only = a.rest.get(1).cloned();
            for (g, items) in h.menu_states() {
                if only.as_ref().is_some_and(|o| !o.eq_ignore_ascii_case(&g)) {
                    continue;
                }
                let on = items
                    .iter()
                    .filter(|(_, s)| matches!(s, CommandState::Bound { .. }))
                    .count();
                println!("{g}: {on}/{} lệnh chạy được", items.len());
                for (c, s) in items {
                    if let CommandState::Bound { addon, shortcut } = s {
                        println!(
                            "   ✓ {c:36} ← {addon}{}",
                            shortcut.map(|k| format!("  [{k}]")).unwrap_or_default()
                        );
                    } else if only.is_some() {
                        println!("   · {c}");
                    }
                }
            }
            Ok(())
        }
        "dev-add" => {
            let p = PathBuf::from(a.pos(1, "thư mục")?);
            h.add_dev_path(&p)
                .map(|_| println!("Đã thêm kho dev {}", p.display()))
        }
        other => Err(format!("lệnh không hợp lệ: {other}\n\n{USAGE}")),
    };
    flush(&mut h);
    res
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Lỗi: {e}");
            ExitCode::from(2)
        }
    }
}
