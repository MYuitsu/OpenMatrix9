// SPDX-License-Identifier: LGPL-2.1-or-later
//! L3-001 contract tests driven by the independent fixtures
//! `implementation/fixtures/M01/OM9-L3-01-01-001/{positive,negative}.json`.
//!
//! The fixtures are read from disk at run time, so a change in the startup
//! contract that no longer matches the recorded expectation fails here instead
//! of drifting silently.
use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::startup::{
    AppConfig, StartupFailure, StartupStage, adapter_note, should_report_dialog,
};
use om9_app::ui::MatrixApp;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../implementation/fixtures/M01/OM9-L3-01-01-001")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("fixture JSON")
}

fn pair(value: &Value, key: &str) -> [f32; 2] {
    let array = value[key].as_array().expect("a two-element array");
    [
        array[0].as_f64().unwrap() as f32,
        array[1].as_f64().unwrap() as f32,
    ]
}

fn config_from(value: &Value) -> AppConfig {
    let config = &value["config"];
    AppConfig {
        app_title: config["app_title"].as_str().unwrap().to_owned(),
        document_name: config["document_name"].as_str().unwrap().to_owned(),
        window: pair(config, "window"),
        min_window: pair(config, "min_window"),
        maximized: config["maximized"].as_bool().unwrap(),
        audit_dir: None,
        layout_path: None,
        startup_dialog: config["startup_dialog"].as_bool().unwrap(),
    }
}

fn frame(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn temp_directory(case: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("om9-startup-fixture-{case}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn positive_fixture_starts_one_window_with_the_openmatrix9_title_and_an_empty_document() {
    let fixture = fixture("positive.json");
    assert_eq!(fixture["l3"], "OM9-L3-01-01-001");
    let expect = &fixture["expect"];
    let config = config_from(&fixture);

    let mut app = MatrixApp::start(&config).expect("the positive start must succeed");
    assert_eq!(config.window_title(), expect["window_title_prefix"]);
    assert_eq!(
        app.controller.document().name,
        expect["document_name"].as_str().unwrap()
    );
    assert_eq!(
        app.controller.document().entities.len() as u64,
        expect["document_entities"].as_u64().unwrap()
    );
    assert_eq!(
        app.controller.is_dirty().unwrap(),
        expect["document_dirty"].as_bool().unwrap()
    );

    let context = egui::Context::default();
    frame(&mut app, &context, Vec2::new(1600., 1000.));
    frame(&mut app, &context, Vec2::new(1600., 1000.));

    assert_eq!(
        app.view_rects.len() as u64,
        expect["viewports"].as_u64().unwrap(),
        "the start must show the four Matrix viewports"
    );
    assert_eq!(app.window_title, expect["window_title"].as_str().unwrap());
    let sections: Vec<String> = app
        .dock
        .left_sections()
        .into_iter()
        .map(|id| id.key().to_owned())
        .collect();
    let expected_sections: Vec<String> = expect["left_sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(sections, expected_sections);
    assert_eq!(
        app.status.kind().label(),
        expect["status_kind"].as_str().unwrap()
    );
    assert_eq!(
        app.command_input_rect().is_some(),
        expect["command_line_present"].as_bool().unwrap()
    );
}

#[test]
fn negative_fixture_reports_every_start_failure_instead_of_exiting_silently() {
    let fixture = fixture("negative.json");
    assert_eq!(fixture["l3"], "OM9-L3-01-01-001");
    let cases = fixture["cases"].as_array().expect("negative cases");
    let directory = temp_directory("negative");

    for case in cases {
        let id = case["id"].as_str().unwrap();
        match id {
            "missing-document" => {
                let expect = &case["expect"];
                let failure =
                    StartupFailure::new(StartupStage::Document, case["detail"].as_str().unwrap());
                assert_eq!(failure.stage.label(), expect["stage"].as_str().unwrap());
                assert!(
                    failure
                        .line()
                        .contains(expect["line_contains"].as_str().unwrap())
                );
                let path = failure.write_receipt(&directory).unwrap();
                let receipt: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                assert_eq!(
                    receipt["schema"],
                    expect["receipt_schema"].as_str().unwrap()
                );
                assert_eq!(receipt["stage"], expect["receipt_stage"].as_str().unwrap());
                assert!(
                    path.ends_with(
                        expect["receipt_file"]
                            .as_str()
                            .unwrap_or(StartupFailure::RECEIPT_FILE)
                    )
                );
            }
            "window-smaller-than-the-minimum" => {
                let expect = &case["expect"];
                let config = AppConfig {
                    window: pair(&case["config"], "window"),
                    min_window: pair(&case["config"], "min_window"),
                    ..AppConfig::default()
                };
                let failure = config
                    .validate()
                    .expect_err("an unusable size must be refused");
                assert_eq!(failure.stage.label(), expect["stage"].as_str().unwrap());
                assert!(
                    failure
                        .detail()
                        .contains(expect["detail_contains"].as_str().unwrap())
                );
            }
            "no-hardware-adapter" => {
                let expect = &case["expect"];
                assert!(case["adapter"].is_null());
                let (adapter, note) = adapter_note(None);
                assert_eq!(adapter, expect["adapter"].as_str().unwrap());
                assert!(
                    note.expect("fallback diagnostic")
                        .contains(expect["diagnostic_contains"].as_str().unwrap())
                );
            }
            "silent-exit-forbidden" => {
                let expect = &case["expect"];
                assert_eq!(
                    StartupFailure::RECEIPT_FILE,
                    expect["receipt_file"].as_str().unwrap()
                );
                assert_eq!(
                    StartupFailure::DIALOG_ENV,
                    expect["dialog_env"].as_str().unwrap()
                );
                assert!(
                    expect["dialog_kept_for"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|value| value.is_null()),
                    "the fixture must keep the dialog for the default (unset) case"
                );
                for value in expect["dialog_suppressed_for"].as_array().unwrap() {
                    assert!(
                        !should_report_dialog(Some(
                            &value.as_str().map(str::to_owned).unwrap_or_default()
                        )),
                        "the dialog must be suppressed for {value}"
                    );
                }
                for value in expect["dialog_kept_for"].as_array().unwrap() {
                    let kept = match value.as_str() {
                        Some(text) => should_report_dialog(Some(text)),
                        None => should_report_dialog(None),
                    };
                    assert!(kept, "the dialog must stay for {value}");
                }
            }
            other => panic!("unexpected negative case {other}"),
        }
    }
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn a_failed_start_leaves_the_document_and_the_layout_untouched() {
    // The document stage runs before any window exists; a failure there must not
    // create a partial document or write a layout.
    let directory = temp_directory("unchanged");
    let layout = directory.join("ui-layout.json");
    let config = AppConfig {
        layout_path: Some(layout.clone()),
        window: [100., 100.],
        min_window: [720., 480.],
        ..AppConfig::default()
    };
    let started = MatrixApp::start(&config);
    let failure = match started {
        Ok(_) => panic!("the size guard must refuse"),
        Err(failure) => failure,
    };
    assert_eq!(failure.stage, StartupStage::Renderer);
    assert!(!layout.exists(), "a refused start writes no layout");
    std::fs::remove_dir_all(&directory).unwrap();
}
