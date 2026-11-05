//! Package the built cdylib, install and enable it through the real host (native runtime),
//! and drive Polyline / Circle steps through `om9_addon_session` as OM9 does.
use om9_addons::catalog::Catalog;
use om9_addons::host::{Host, HostConfig, InstallOptions, Startup, build_package};
use om9_addons::manifest::RuntimeKind;
use om9_addons::native::NativeRuntime;
use om9_addons::semver::Version;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

fn built_library() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap();
    let name = format!(
        "{}om9_curves{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    [deps.join(&name), deps.parent().unwrap().join(&name)]
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("{name} not found next to {}", deps.display()))
}

#[test]
fn install_and_run_interactive_steps() {
    let platform = om9_addons::host::current_platform();
    if platform == "unknown" {
        return;
    }
    let tmp = std::env::temp_dir().join(format!("om9-curves-native-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let stage = tmp.join("stage");
    fs::create_dir_all(&stage).unwrap();
    let lib = built_library();
    let entry = lib.file_name().unwrap().to_string_lossy().to_string();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("addon/om9_addon.toml");
    let manifest = fs::read_to_string(src)
        .unwrap()
        .replace("@PLATFORM@", platform)
        .replace("@ENTRY@", &entry);
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::copy(&lib, stage.join(&entry)).unwrap();
    let ini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Resources/menu/MainMenu.ini");
    let catalog = Catalog::from_main_menu_ini(&fs::read_to_string(ini).unwrap()).unwrap();
    let (m, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(m.runtime, RuntimeKind::Native);
    assert!(m.commands.iter().all(|c| c.interactive));
    assert!(m.commands.iter().any(|c| c.aliases == ["Circle"]));
    fs::write(tmp.join("curves.zip"), zip).unwrap();

    let mut host = Host::new(HostConfig::new(
        Version(0, 3, 6),
        catalog,
        tmp.join("config"),
    ));
    host.register_runtime(RuntimeKind::Native, Box::new(NativeRuntime::new()));
    host.startup(&Startup::default());
    let r = host
        .install_zip(
            &tmp.join("curves.zip"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);

    let mut state = Value::Null;
    let mut step = |cmd: &str, input: Value| -> Value {
        let ev = json!({"command": cmd, "input": input, "state": state});
        let out = host
            .session(cmd, &serde_json::to_vec(&ev).unwrap())
            .unwrap();
        let reply: Value = serde_json::from_slice(&out).unwrap();
        state = reply["state"].clone();
        reply
    };
    assert_eq!(
        step("CurvePolylinePolyline", json!({"kind": "start"}))["prompt"],
        "Start of polyline"
    );
    step(
        "CurvePolylinePolyline",
        json!({"kind": "point", "point": [0, 0, 0]}),
    );
    step(
        "CurvePolylinePolyline",
        json!({"kind": "point", "point": [10, 0, 0]}),
    );
    let done = step("CurvePolylinePolyline", json!({"kind": "enter"}));
    assert_eq!(done["done"], true);
    assert_eq!(done["create"][0]["type"], "polyline");

    state = Value::Null;
    let mut step = |cmd: &str, input: Value| -> Value {
        let ev = json!({"command": cmd, "input": input, "state": state});
        let out = host
            .session(cmd, &serde_json::to_vec(&ev).unwrap())
            .unwrap();
        let reply: Value = serde_json::from_slice(&out).unwrap();
        state = reply["state"].clone();
        reply
    };
    step("CurveCircleCenterRadius", json!({"kind": "start"}));
    step(
        "CurveCircleCenterRadius",
        json!({"kind": "point", "point": [0, 0, 0]}),
    );
    let done = step(
        "CurveCircleCenterRadius",
        json!({"kind": "text", "text": "8.675"}),
    );
    assert_eq!(done["create"][0]["radius"], 8.675);
    // Malformed events are refused at the boundary, not inside the add-on.
    assert!(host.session("CurveCircleCenterRadius", b"{").is_err());
    host.disable("om9_curves").unwrap();
    assert!(host.session("CurveCircleCenterRadius", b"{}").is_err());
    fs::remove_dir_all(&tmp).unwrap();
}
