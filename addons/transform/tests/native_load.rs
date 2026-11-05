//! Package the built cdylib, install it through the real host and drive Move steps.
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
        "{}om9_transform{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    [deps.join(&name), deps.parent().unwrap().join(&name)]
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("{name} not found"))
}

#[test]
fn install_and_move() {
    let platform = om9_addons::host::current_platform();
    if platform == "unknown" {
        return;
    }
    let tmp = std::env::temp_dir().join(format!("om9-transform-native-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let stage = tmp.join("stage");
    fs::create_dir_all(&stage).unwrap();
    let lib = built_library();
    let entry = lib.file_name().unwrap().to_string_lossy().to_string();
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("addon/om9_addon.toml"))
            .unwrap()
            .replace("@PLATFORM@", platform)
            .replace("@ENTRY@", &entry);
    fs::write(stage.join("om9_addon.toml"), manifest).unwrap();
    fs::copy(&lib, stage.join(&entry)).unwrap();
    let ini = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Resources/menu/MainMenu.ini");
    let catalog = Catalog::from_main_menu_ini(&fs::read_to_string(ini).unwrap()).unwrap();
    let (_, zip) = build_package(&stage, Some(&catalog)).unwrap_or_else(|e| panic!("{e:?}"));
    fs::write(tmp.join("t.zip"), zip).unwrap();
    let mut host = Host::new(HostConfig::new(
        Version(0, 3, 6),
        catalog,
        tmp.join("config"),
    ));
    host.register_runtime(RuntimeKind::Native, Box::new(NativeRuntime::new()));
    host.startup(&Startup::default());
    assert!(
        host.install_zip(
            &tmp.join("t.zip"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            }
        )
        .unwrap()
        .enabled
    );
    let sel = json!([{"id": "1", "kind": "mesh", "name": "m", "bbox": [[0, 0, 0], [1, 1, 1]]}]);
    let mut state = Value::Null;
    let mut step = |input: Value| -> Value {
        let ev =
            json!({"command": "TopIconMove", "input": input, "state": state, "selection": sel});
        let r: Value = serde_json::from_slice(
            &host
                .session("TopIconMove", &serde_json::to_vec(&ev).unwrap())
                .unwrap(),
        )
        .unwrap();
        state = r["state"].clone();
        r
    };
    assert_eq!(
        step(json!({"kind": "start"}))["prompt"],
        "Point to move from"
    );
    step(json!({"kind": "point", "point": [0, 0, 0]}));
    let r = step(json!({"kind": "point", "point": [3, 4, 0]}));
    assert_eq!(r["done"], true);
    assert_eq!(r["transform"]["matrices"][0][3], 3.0);
    assert_eq!(r["transform"]["keep_original"], false);
    fs::remove_dir_all(&tmp).unwrap();
}
