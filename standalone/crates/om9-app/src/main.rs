// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg_attr(windows, windows_subsystem = "windows")]
use om9_archive::Archive;
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Cloud, Curve, Geometry, Mesh},
    model::{Document, Entity},
    transaction::Cancellation,
};
use std::path::Path;

fn fixture() -> Document {
    let mut document = Document::new("OM9 authored standalone fixture");
    for (label, geometry) in [
        ("Point", Geometry::Point([1., 2., 3.])),
        (
            "Circle",
            Geometry::Curve(Curve::Circle {
                center: [0., 0., 0.],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 10.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            }),
        ),
        (
            "Mesh",
            Geometry::Mesh(Mesh {
                vertices: vec![[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]],
                faces: vec![vec![0, 1, 2]],
                normals: vec![],
                colors: vec![],
            }),
        ),
        (
            "Cloud",
            Geometry::Cloud(Cloud {
                points: vec![[2., 3., 4.], [5., 6., 7.]],
                normals: vec![],
                colors: vec![[255, 0, 0, 255], [0, 255, 0, 255]],
                hidden: vec![false, true],
            }),
        ),
    ] {
        let entity = Entity::new(label, geometry, document.current_layer);
        document.entities.insert(entity.id, entity);
    }
    document
}
fn report(document: &Document, diagnostics: &[String]) -> Result<()> {
    om9_app::report::write(
        std::io::stdout().lock(),
        document,
        diagnostics,
        &Limits::default(),
    )
}
fn create_fixture(path: &Path) -> Result<()> {
    let document = fixture();
    Archive::save(
        path,
        &document,
        &Limits::default(),
        &Cancellation::default(),
    )?;
    report(&document, &[])
}
fn migrate(path: &Path, version: u8) -> Result<()> {
    let limits = Limits::default();
    let cancellation = Cancellation::default();
    let empty = std::collections::BTreeSet::new();
    let restored = match version {
        1 => Archive::upgrade_v1_restored_in_place(path, &limits, &cancellation, &empty)?,
        2 => Archive::upgrade_v2_restored_in_place(path, &limits, &cancellation, &empty)?,
        _ => Archive::upgrade_v3_restored_in_place(path, &limits, &cancellation, &empty)?,
    };
    report(&restored.document, &restored.diagnostics)
}
fn inspect(path: &Path, hold: bool) -> Result<()> {
    let restored =
        Archive::open_restored(path, &Limits::default(), &std::collections::BTreeSet::new())?;
    report(&restored.document, &restored.diagnostics)?;
    if hold {
        use std::io::Write;
        std::io::stdout()
            .flush()
            .map_err(|e| Error::Io(e.to_string()))?;
        let mut release = String::new();
        // Audit owner controls this bounded diagnostic lifetime through stdin.
        // No background watcher or application child is started.
        std::io::stdin()
            .read_line(&mut release)
            .map_err(|e| Error::Io(e.to_string()))?;
    }
    Ok(())
}
/// Startup failures of the document/CLI path keep the document stage; the two
/// UI entry points classify their own document restore and window creation.
fn document_stage(result: Result<()>) -> std::result::Result<(), om9_app::startup::StartupFailure> {
    result.map_err(|error| {
        om9_app::startup::StartupFailure::from_error(
            om9_app::startup::StartupStage::Document,
            &error,
        )
    })
}
fn run() -> std::result::Result<(), om9_app::startup::StartupFailure> {
    use om9_app::startup::{StartupFailure, StartupStage};
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        return om9_app::ui::launch_classified(Document::new("Untitled"), None);
    }
    if args.len() != 2 {
        return Err(StartupFailure::new(
            StartupStage::Document,
            "usage: om9 --create-fixture|--inspect|--audit|--migrate-v1|--migrate-v2|--migrate-v3 path.om9",
        ));
    }
    let path = Path::new(&args[1]);
    match args[0].to_str() {
        Some("--ui-audit") => {
            om9_app::ui::launch_classified(Document::new("Untitled"), Some(path.to_path_buf()))
        }
        Some("--ui") => om9_app::ui::launch_file_classified(path),
        Some("--create-fixture") => document_stage(create_fixture(path)),
        Some("--migrate-v1") => document_stage(migrate(path, 1)),
        Some("--migrate-v2") => document_stage(migrate(path, 2)),
        Some("--migrate-v3") => document_stage(migrate(path, 3)),
        Some("--inspect") => document_stage(inspect(path, false)),
        Some("--audit") => document_stage(inspect(path, true)),
        _ => Err(StartupFailure::new(
            StartupStage::Document,
            "standalone command line",
        )),
    }
}
/// Only a real window start owns a user-visible startup dialog; file commands
/// and audits keep the receipt and stderr diagnostic.
fn displays_a_window(args: &[std::ffi::OsString]) -> bool {
    args.is_empty()
        || matches!(
            args.first().and_then(|arg| arg.to_str()),
            Some("--ui" | "--ui-audit")
        )
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let ui_mode = displays_a_window(&args);
    if let Err(failure) = run() {
        // A console-less executable must not exit silently: keep the stderr line,
        // write the owned receipt next to the application layout and, only for a
        // real UI start, show the native diagnostic.
        eprintln!("OM9: {}", failure.line());
        if let Err(error) = failure.report() {
            eprintln!("OM9: startup receipt not written: {error}");
        }
        if ui_mode {
            let _ = om9_app::ui::report_startup_failure(&failure);
        }
        std::process::exit(1);
    }
}
