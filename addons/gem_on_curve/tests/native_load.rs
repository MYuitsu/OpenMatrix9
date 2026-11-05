//! Nạp chính thư viện cdylib vừa build qua host thật (om9-addons + NativeRuntime):
//! đóng gói zip → cài → bật → chạy lệnh → kiểm tra mesh và thông điệp → tắt (giải phóng DLL).

use om9_addons::catalog::Catalog;
use om9_addons::host::{CommandState, Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::native::NativeRuntime;
use om9_addons::runtime::{CommandInput, CurveInput};
use om9_addons::semver::Version;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn built_library() -> PathBuf {
    // tests chạy từ target/<profile>/deps/; cdylib nằm ở target/<profile>/
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap();
    let name = format!(
        "{}gem_on_curve{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    [deps.join(&name), deps.parent().unwrap().join(&name)]
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("không thấy {name} cạnh {}", deps.display()))
}

fn platform() -> &'static str {
    om9_addons::host::current_platform()
}

fn stage_package(tmp: &Path) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("addon");
    let stage = tmp.join("stage");
    fs::create_dir_all(stage.join("icons")).unwrap();
    let lib = built_library();
    let entry = lib.file_name().unwrap().to_string_lossy().to_string();
    let manifest = fs::read_to_string(src.join("om9_addon.toml"))
        .unwrap()
        .replace("@PLATFORM@", platform())
        .replace("@ENTRY@", &entry);
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::copy(
        src.join("icons/gem_on_crv.svg"),
        stage.join("icons/gem_on_crv.svg"),
    )
    .unwrap();
    fs::copy(&lib, stage.join(&entry)).unwrap();
    stage
}

fn catalog() -> Catalog {
    let ini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Resources/menu/MainMenu.ini");
    Catalog::from_main_menu_ini(&fs::read_to_string(ini).unwrap()).unwrap()
}

fn ring(d: f64, n: usize) -> Vec<[f64; 3]> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            [d / 2.0 * a.cos(), d / 2.0 * a.sin(), 0.0]
        })
        .collect()
}

#[test]
fn package_install_enable_invoke_disable() {
    if platform() == "unknown" {
        return;
    }
    let tmp = std::env::temp_dir().join(format!("om9-native-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let cat = catalog();
    let (m, zip) =
        build_package(&stage_package(&tmp), Some(&cat)).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(m.runtime, RuntimeKind::Native);
    let zpath = tmp.join("gem_on_curve.zip");
    fs::write(&zpath, zip).unwrap();

    let mut host = Host::new(HostConfig::new(Version(0, 1, 0), cat, tmp.join("config")));
    host.register_runtime(RuntimeKind::Native, Box::new(NativeRuntime::new()));
    host.startup(&Startup::default());
    let r = host
        .install_zip(
            &zpath,
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);
    assert!(matches!(
        host.command_state("BuilderGemOnCrv"),
        CommandState::Bound { .. }
    ));

    let input = CommandInput {
        command: String::new(),
        curves: vec![CurveInput {
            points: ring(17.35, 2000),
            closed: true,
        }],
        plane_normal: [0.0, 0.0, 1.0],
        settings: BTreeMap::new(),
    };
    let count = host.invoke("BuilderGemCountOnCrv", input.clone()).unwrap();
    assert!(count.meshes.is_empty());
    assert!(
        count.messages[0].contains("32 viên") && count.messages[0].contains("0.203"),
        "{:?}",
        count.messages
    );

    let out = host.invoke("BuilderGemOnCrv", input.clone()).unwrap();
    assert_eq!(out.meshes.len(), 32);
    assert!(out.meshes[0].meta_json.contains("\"diameter_mm\":1.5"));
    // settings từ Preferences được truyền tới add-on
    host.set_setting("gem_on_curve", "diameter", "1,3").unwrap();
    let out = host.invoke("BuilderGemOnCrv", input.clone()).unwrap();
    assert_eq!(out.meshes.len(), 36);

    // không chọn đường cong → lỗi có lý do, không panic
    let e = host
        .invoke(
            "BuilderGemOnCrv",
            CommandInput {
                curves: vec![],
                ..input.clone()
            },
        )
        .unwrap_err();
    assert!(e.contains("Chọn ít nhất một đường cong"), "{e}");
    // setting sai (end < start trên đường hở) → lỗi của add-on
    let open = CommandInput {
        curves: vec![CurveInput {
            points: vec![[0.0; 3], [10.0, 0.0, 0.0]],
            closed: false,
        }],
        ..input
    };
    host.set_setting("gem_on_curve", "start", "80").unwrap();
    host.set_setting("gem_on_curve", "end", "20").unwrap();
    assert!(
        host.invoke("BuilderGemOnCrv", open)
            .unwrap_err()
            .contains("End Position")
    );

    host.disable("gem_on_curve").unwrap();
    assert_eq!(host.command_state("BuilderGemOnCrv"), CommandState::Unbound);
    // DLL đã được giải phóng → gỡ được thư mục (Windows khoá file đang nạp)
    host.uninstall("gem_on_curve").unwrap();
    fs::remove_dir_all(&tmp).unwrap();
}
