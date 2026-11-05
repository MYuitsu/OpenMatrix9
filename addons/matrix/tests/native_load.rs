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
        "{}om9_matrix{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    [deps.join(&name), deps.parent().unwrap().join(&name)]
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| panic!("{name} not found next to {}", deps.display()))
}

fn copy_dir(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let dst = to.join(p.file_name().unwrap());
        if p.is_dir() {
            fs::create_dir_all(&dst).unwrap();
            copy_dir(&p, &dst);
        } else {
            fs::copy(&p, &dst).unwrap();
        }
    }
}

#[test]
fn install_and_run_interactive_steps() {
    let platform = om9_addons::host::current_platform();
    if platform == "unknown" {
        return;
    }
    let tmp = std::env::temp_dir().join(format!("om9-matrix-native-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let stage = tmp.join("stage");
    fs::create_dir_all(&stage).unwrap();
    let lib = built_library();
    let entry = lib.file_name().unwrap().to_string_lossy().to_string();
    copy_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("addon"), &stage);
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
    assert_eq!(m.commands.len(), om9_matrix::ALL.len());
    // The package carries one help page per command.
    assert!(stage.join("commands/Gems/BuilderHalo.md").is_file());
    fs::write(tmp.join("matrix.zip"), zip).unwrap();

    let mut host = Host::new(HostConfig::new(
        Version(0, 3, 6),
        catalog,
        tmp.join("config"),
    ));
    host.register_runtime(RuntimeKind::Native, Box::new(NativeRuntime::new()));
    host.startup(&Startup::default());
    let r = host
        .install_zip(
            &tmp.join("matrix.zip"),
            &InstallOptions {
                enable: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.enabled, "{:?}", r.warnings);

    // The installed package translates the interface: Vietnamese by default, English on
    // request. The language is an add-on setting, never a command option.
    let (tr, errs) = host.translator();
    assert!(errs.is_empty(), "{errs:?}");
    assert_eq!(
        tr.command("CurveLineSingleLine", "Single Line"),
        "Đoạn thẳng"
    );
    assert_eq!(tr.text("Start of polyline"), "Điểm đầu đường gấp khúc");
    assert_eq!(tr.token("BothSides", Some("No")), "Hai phía=Không");
    let line = host.command_info("CurveLineSingleLine").unwrap();
    assert!(line.settings.iter().all(|(s, _)| s.key != "language"));
    host.set_setting("om9_matrix", "language", "en").unwrap();
    let en = host.translator().0;
    assert_eq!(
        en.command("CurveLineSingleLine", "Single Line"),
        "Single Line"
    );
    assert_eq!(en.text("Start of polyline"), "Start of polyline");
    host.set_setting("om9_matrix", "language", "vi").unwrap();

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
    // Interactive commands of other menus answer through the same library.
    let mut state = Value::Null;
    let mut step = |cmd: &str, input: Value| -> Value {
        let ev = json!({"command": cmd, "input": input, "state": state, "settings": {"loader_cut": "pear", "loader_length": 6.0}});
        let out = host.session(cmd, &serde_json::to_vec(&ev).unwrap()).unwrap();
        let reply: Value = serde_json::from_slice(&out).unwrap();
        state = reply["state"].clone();
        reply
    };
    step("BuilderGemLoader", json!({"kind": "start"}));
    step("BuilderGemLoader", json!({"kind": "point", "point": [1, 2, 0]}));
    let done = step("BuilderGemLoader", json!({"kind": "enter"}));
    assert_eq!(done["create"][0]["meta"]["cut"], "pear");
    // The one-shot Gem On Crv still runs through om9_addon_invoke.
    let circle: Vec<[f64; 3]> = (0..256)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / 256.0;
            [8.675 * a.cos(), 8.675 * a.sin(), 0.0]
        })
        .collect();
    let input = om9_addons::runtime::CommandInput {
        curves: vec![om9_addons::runtime::CurveInput { points: circle, closed: true }],
        ..Default::default()
    };
    let out = host.invoke("BuilderGemOnCrv", input).unwrap();
    assert_eq!(out.meshes.len(), 32, "{:?}", out.messages);
    host.disable("om9_matrix").unwrap();
    assert!(host.session("CurveCircleCenterRadius", b"{}").is_err());
    fs::remove_dir_all(&tmp).unwrap();
}
