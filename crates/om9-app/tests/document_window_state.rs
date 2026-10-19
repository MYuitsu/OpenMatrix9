// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-01-01-006 (Active Document Window State), driven by
//! the fixtures in `implementation/fixtures/M01/OM9-L3-01-01-006/`.
use eframe::egui::{self, Event, Pos2, Rect, Vec2};
use om9_app::file_lifecycle::{FileKind, FileOutcome, FilePhase, UnsavedDecision};
use om9_app::session::{InvocationOrigin, LINE};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-01-01-006")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn case<'a>(document: &'a Value, id: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("fixture case {id}"))
}

const SIZE: Vec2 = Vec2::new(1280., 720.);

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SIZE)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn warmup(app: &mut MatrixApp, context: &egui::Context) {
    for _ in 0..3 {
        frame(app, context, Vec::new());
    }
}

/// A change the user can see: draw a line through the same controller path the
/// command box, the menu and F6 all use.
fn draw_a_line(app: &mut MatrixApp, _context: &egui::Context) {
    app.controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("start a line");
    app.controller
        .accept_world_point([0., 0., 0.])
        .expect("first point");
    app.controller
        .accept_world_point([10., 0., 0.])
        .expect("second point");
    assert!(!app.controller.is_running());
}

fn temp_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("om9-l3-006-{name}"));
    let _ = std::fs::remove_file(&path);
    path
}

fn token(app: &MatrixApp) -> om9_app::file_lifecycle::FileToken {
    app.controller
        .file_operation()
        .expect("an operation is in flight")
        .token()
}

fn title(app: &MatrixApp) -> String {
    app.window_title.clone()
}

fn marker(document: &Value) -> &str {
    document["title"]["dirty_marker"].as_str().unwrap()
}

fn prefix(document: &Value) -> &str {
    document["title"]["prefix"].as_str().unwrap()
}

#[test]
fn w1_a_new_session_is_named_and_clean() {
    let document = fixture("positive.json");
    let case = case(&document, "W1-a-new-session-is-named-and-clean");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    assert_eq!(title(&app), case["expect"]["title"].as_str().unwrap());
    assert!(!app.controller.is_dirty().unwrap());
    assert!(!title(&app).ends_with(marker(&document)));
    assert!(title(&app).starts_with(prefix(&document)));
}

#[test]
fn w2_a_changed_document_is_marked() {
    let document = fixture("positive.json");
    let case = case(&document, "W2-a-changed-document-is-marked");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    draw_a_line(&mut app, &context);
    frame(&mut app, &context, Vec::new());
    assert!(app.controller.is_dirty().unwrap());
    assert_eq!(case["expect"]["has_marker"], true);
    assert!(title(&app).ends_with(marker(&document)), "{}", title(&app));
}

#[test]
fn w3_saving_removes_the_marker_and_names_the_file() {
    let document = fixture("positive.json");
    let case = case(&document, "W3-saving-removes-the-marker-and-names-the-file");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    draw_a_line(&mut app, &context);
    let path = temp_path("w3-first.om9");
    let outcome = app.controller.begin_file_operation(FileKind::Save).unwrap();
    assert_eq!(outcome, FileOutcome::Pending);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::SavePath
    );
    app.controller.accept_file_path(token(&app), &path).unwrap();
    frame(&mut app, &context, Vec::new());
    assert!(!app.controller.is_dirty().unwrap());
    assert!(path.exists());
    assert_eq!(case["expect"]["file_written"], true);
    assert!(!title(&app).ends_with(marker(&document)), "{}", title(&app));
    assert!(
        title(&app).ends_with(case["expect"]["title_suffix"].as_str().unwrap()),
        "{}",
        title(&app)
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn w4_a_change_after_saving_marks_it_again() {
    let document = fixture("positive.json");
    let case = case(&document, "W4-a-change-after-saving-marks-it-again");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    draw_a_line(&mut app, &context);
    let path = temp_path("w4-first.om9");
    app.controller.begin_file_operation(FileKind::Save).unwrap();
    app.controller.accept_file_path(token(&app), &path).unwrap();
    frame(&mut app, &context, Vec::new());
    assert!(!app.controller.is_dirty().unwrap());
    draw_a_line(&mut app, &context);
    frame(&mut app, &context, Vec::new());
    assert!(app.controller.is_dirty().unwrap());
    assert_eq!(case["expect"]["has_marker"], true);
    assert!(title(&app).ends_with(marker(&document)), "{}", title(&app));
    assert!(
        title(&app).contains(case["expect"]["title_suffix"].as_str().unwrap()),
        "{}",
        title(&app)
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn w5_switching_documents_changes_the_name_and_clears_the_state() {
    let document = fixture("positive.json");
    let case = case(
        &document,
        "W5-switching-documents-changes-the-name-and-clears-the-state",
    );
    let context = egui::Context::default();
    let first = temp_path("w5-first.om9");
    let second = temp_path("w5-second.om9");

    // A session that draws one line, saves it, draws another and then opens the
    // saved file again: a switch with unsaved work in front of it.
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    draw_a_line(&mut app, &context);
    app.controller.begin_file_operation(FileKind::Save).unwrap();
    app.controller
        .accept_file_path(token(&app), &first)
        .unwrap();
    frame(&mut app, &context, Vec::new());
    draw_a_line(&mut app, &context);
    frame(&mut app, &context, Vec::new());
    assert!(app.controller.is_dirty().unwrap());
    // The switch target is the same document saved under another name.
    std::fs::copy(&first, &second).expect("copy the document");

    let outcome = app.controller.begin_file_operation(FileKind::Open).unwrap();
    assert_eq!(outcome, FileOutcome::Pending);
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::Unsaved
    );
    app.controller
        .decide_unsaved(token(&app), UnsavedDecision::Discard)
        .unwrap();
    assert_eq!(
        app.controller.file_operation().unwrap().phase(),
        FilePhase::OpenPath
    );
    app.controller
        .accept_file_path(token(&app), &second)
        .unwrap();
    frame(&mut app, &context, Vec::new());

    let expect = &case["expect"];
    assert!(!app.controller.is_dirty().unwrap());
    assert_eq!(expect["has_marker"], false);
    assert!(!title(&app).ends_with(marker(&document)), "{}", title(&app));
    assert!(
        title(&app).ends_with(expect["title_suffix"].as_str().unwrap()),
        "{}",
        title(&app)
    );
    assert_eq!(
        app.controller.document().entities.len() as u64,
        expect["entities"].as_u64().unwrap(),
        "the open document is the one on disk, not the one that was open"
    );
    assert_eq!(
        app.controller.coordinator().undo_len() as u64,
        expect["undo_len"].as_u64().unwrap(),
        "a freshly opened document has no history"
    );
    assert!(
        app.controller
            .document()
            .name
            .ends_with(expect["document_name_suffix"].as_str().unwrap()),
        "{}",
        app.controller.document().name
    );
    let _ = std::fs::remove_file(&first);
    let _ = std::fs::remove_file(&second);
}

#[test]
fn w6_only_menus_moved_leaves_the_document_clean() {
    let document = fixture("positive.json");
    let case = case(&document, "W6-only-menus-moved-leaves-the-document-clean");
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
    warmup(&mut app, &context);
    let before = title(&app);
    assert!(
        app.apply_panel_op(om9_app::panel_ops::PanelOp::ToggleCollapsed {
            section: om9_app::dock_layout::SectionId::Projects
        })
        .is_ok()
    );
    frame(&mut app, &context, Vec::new());
    assert!(!app.controller.is_dirty().unwrap());
    assert_eq!(case["expect"]["has_marker"], false);
    assert_eq!(
        title(&app),
        before,
        "moving a menu is not a document change"
    );
}

#[test]
fn w7_a_long_name_is_cut_on_a_character_boundary() {
    let document = fixture("positive.json");
    let case = case(&document, "W7-a-long-name-is-cut-on-a-character-boundary");
    let unit = case["document_name"].as_str().unwrap();
    let repeat = case["document_name_repeat"].as_u64().unwrap() as usize;
    let name = unit.repeat(repeat);
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new(&name)).unwrap();
    warmup(&mut app, &context);
    let title = title(&app);
    let prefix = prefix(&document);
    assert!(title.starts_with(prefix));
    let shown = &title[prefix.len()..];
    let cap = case["expect"]["title_name_bytes_at_most"].as_u64().unwrap() as usize;
    assert!(
        shown.len() <= cap,
        "{} bytes of name are shown",
        shown.len()
    );
    assert!(!shown.contains('\u{FFFD}'), "a character was split");
    assert_eq!(case["expect"]["no_replacement_character"], true);
    assert!(
        shown.starts_with(unit),
        "the name starts with the first unit"
    );
}

#[test]
fn c1_to_c9_unsaved_work_requires_an_explicit_decision() {
    let document = fixture("negative.json");
    for case in document["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Untitled")).unwrap();
        warmup(&mut app, &context);
        let saved = temp_path(&format!("{id}.om9"));
        let start = case["start"].as_str().unwrap();
        if start == "dirty" || start == "dirty-saved" {
            draw_a_line(&mut app, &context);
            if start == "dirty-saved" {
                app.controller.begin_file_operation(FileKind::Save).unwrap();
                app.controller
                    .accept_file_path(token(&app), &saved)
                    .unwrap();
                frame(&mut app, &context, Vec::new());
                draw_a_line(&mut app, &context);
                assert!(app.controller.is_dirty().unwrap(), "{id}: setup is dirty");
            }
        }
        let entities_before = app.controller.document().entities.len();
        let intent = case["intent"].as_str().unwrap();
        let outcome = match intent {
            "WindowClose" => app.controller.begin_window_close(),
            "Command(New)" => app.controller.begin_file_operation(FileKind::New),
            other => panic!("{id}: unknown intent {other}"),
        }
        .unwrap_or_else(|error| panic!("{id}: {error:?}"));

        if case["expect"]["phase"].is_string() {
            // The question comes first: nothing has closed and nothing is written.
            assert_eq!(outcome, FileOutcome::Pending, "{id}");
        }
        if let Some(expected) = case["expect"]["outcome"].as_str() {
            let expected = match expected {
                "CloseReady" => FileOutcome::CloseReady,
                "SavedAndCloseReady" => FileOutcome::SavedAndCloseReady,
                other => panic!("{id}: unknown outcome {other}"),
            };
            // The fixture states the outcome of the whole decision; a case with a
            // decision therefore reports it after the answer, a clean close at once.
            if case["decision"].is_null() {
                assert_eq!(outcome, expected, "{id}");
            }
        }
        let mut decided: Option<FileOutcome> = None;

        match case["decision"].as_str() {
            Some("Discard") => {
                decided = Some(
                    app.controller
                        .decide_unsaved(token(&app), UnsavedDecision::Discard)
                        .unwrap(),
                );
            }
            Some("Save") => {
                decided = Some(
                    app.controller
                        .decide_unsaved(token(&app), UnsavedDecision::Save)
                        .unwrap(),
                );
            }
            Some("Cancel") => {
                let stale = token(&app);
                app.controller.cancel_file_operation(stale).unwrap();
                if case["then"].is_string() {
                    let error = app
                        .controller
                        .decide_unsaved(stale, UnsavedDecision::Save)
                        .expect_err("a stale token must be refused");
                    let _ = format!("{error:?}");
                    assert_eq!(case["expect"]["error"], true, "{id}");
                }
            }
            _ => {}
        }
        frame(&mut app, &context, Vec::new());

        if let Some(phase) = case["expect"]["phase"].as_str() {
            let expected = match phase {
                "Unsaved" => FilePhase::Unsaved,
                "SavePath" => FilePhase::SavePath,
                other => panic!("{id}: unknown phase {other}"),
            };
            assert_eq!(
                app.controller
                    .file_operation()
                    .map(|operation| operation.phase()),
                Some(expected),
                "{id}"
            );
        }
        if let Some(expected) = case["expect"]["outcome"].as_str() {
            let expected = match expected {
                "CloseReady" => FileOutcome::CloseReady,
                "SavedAndCloseReady" => FileOutcome::SavedAndCloseReady,
                other => panic!("{id}: unknown outcome {other}"),
            };
            if let Some(decided) = decided {
                assert_eq!(decided, expected, "{id}");
            }
        }
        let expect = &case["expect"];
        if expect["has_operation"] == false {
            assert!(app.controller.file_operation().is_none(), "{id}");
        }
        if expect["still_dirty"] == true {
            assert!(app.controller.is_dirty().unwrap(), "{id}");
        }
        if expect["dirty"] == false {
            assert!(!app.controller.is_dirty().unwrap(), "{id}");
        }
        if expect["document_unchanged"] == true {
            assert_eq!(
                app.controller.document().entities.len(),
                entities_before,
                "{id}"
            );
        }
        if let Some(entities) = expect["entities"].as_u64() {
            assert_eq!(
                app.controller.document().entities.len() as u64,
                entities,
                "{id}"
            );
        }
        if let Some(undo) = expect["undo_len"].as_u64() {
            assert_eq!(app.controller.coordinator().undo_len() as u64, undo, "{id}");
        }
        if let Some(suffix) = expect["title_suffix"].as_str() {
            assert!(title(&app).ends_with(suffix), "{id}: {}", title(&app));
        }
        if expect["file_written"] == false {
            assert!(!saved.exists(), "{id}: nothing may be written");
        }
        if expect["file_written"] == true {
            assert!(saved.exists(), "{id}: the answer Save writes the file");
        }
        let _ = std::fs::remove_file(&saved);
    }
}

#[test]
fn fixture_design_pair_is_present() {
    for name in ["positive.json", "negative.json"] {
        let _ = fixture(name);
    }
    let api = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/design/M01/OM9-L3-01-01-006/api.md");
    assert!(api.exists(), "{}", api.display());
    let _ = InvocationOrigin::Cmd;
}
