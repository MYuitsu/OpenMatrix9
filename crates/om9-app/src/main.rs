// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg_attr(windows, windows_subsystem = "windows")]
use om9_archive::Archive;
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Cloud, Curve, Geometry, Mesh},
    model::{Document, Entity},
    transaction::{Cancellation, TransactionCoordinator, World},
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

                corner_uvs: Vec::new(),
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
    report_with_limits(document, diagnostics, &Limits::default())
}
fn report_with_limits(document: &Document, diagnostics: &[String], limits: &Limits) -> Result<()> {
    om9_app::report::write(
        std::io::stdout().lock(),
        document,
        diagnostics,
        limits,
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
/// Import a Rhino 3DM through the admitted native reader and save the resulting OM9 project.
fn import_3dm(source: &Path, target: &Path) -> Result<()> {
    let host = om9_app::three_dm_host::ThreeDmHost::configured()?;
    let document = Document::with_template_tolerance("3DM import");
    let id = document.id;
    let limits = om9_app::document_limits();
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![document], &limits)?, limits)?;
    if let Some(total) = om9_native_owner::physical_memory_bytes() {
        coordinator.constrain_managed_ram(total, om9_core::budget::Usage::default())?;
    }
    let receipt = host.import(&mut coordinator, id, source, &Cancellation::default())?;
    let document = coordinator
        .world()
        .documents
        .get(&id)
        .ok_or(Error::Missing("imported document"))?;
    // The serializer and verification copy share the same account as the
    // imported document; lease only the remaining admitted headroom.
    let document_usage = document.validate(&limits)?;
    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
    let reservation = coordinator.reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, physical, om9_core::budget::Usage::default(),
    )?;
    let mut archive_limits = limits;
    archive_limits.max_bytes = document_usage.bytes.checked_add(reservation.bytes())
        .ok_or(Error::Overflow("save memory budget"))?;
    Archive::save(target, document, &archive_limits, &Cancellation::default())?;
    drop(reservation);
    report_with_limits(document, &receipt.diagnostics, &limits)
}

/// Export the current OM9 project through the admitted native writer.
fn export_3dm(source: &Path, target: &Path) -> Result<()> {
    let host = om9_app::three_dm_host::ThreeDmHost::configured()?;
    let limits = om9_app::document_limits();
    let controller = om9_app::session::Controller::open_archive_for_batch(source, limits)?;
    let receipt = host.export(controller.coordinator(), controller.document().id, target, &Cancellation::default())?;
    report_with_limits(controller.document(), &receipt.diagnostics, &limits)
}

/// Renders every workspace viewport into a PNG under `out`, without starting the application.
fn render_views(project: &Path, out: &Path, width: u32, height: u32) -> Result<()> {
    let written = om9_app::headless::render_project(project, out, width, height)?;
    for path in written {
        println!("{}", path.display());
    }
    Ok(())
}

/// Renders the four workspace viewports into one 2x2 sheet.
fn render_sheet(project: &Path, out: &Path, width: u32, height: u32) -> Result<()> {
    let path = om9_app::headless::render_sheet(project, out, width, height)?;
    println!("{}", path.display());
    Ok(())
}

fn migrate(path: &Path, version: u8) -> Result<()> {
    let mut limits = om9_app::document_limits();
    let coordinator = TransactionCoordinator::new(
        World::new(vec![Document::new("Migration budget")], &limits)?, limits,
    )?;
    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
    // The legacy decoder, restored copy and verified save are bounded by this
    // lease while the source and result coexist. Unknown native allocations
    // are not covered by the managed reservation.
    let reservation = coordinator.reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, physical, om9_core::budget::Usage::default(),
    )?;
    limits.max_bytes = reservation.bytes();
    let cancellation = Cancellation::default();
    let empty = std::collections::BTreeSet::new();
    let restored = match version {
        1 => Archive::upgrade_v1_restored_in_place(path, &limits, &cancellation, &empty)?,
        2 => Archive::upgrade_v2_restored_in_place(path, &limits, &cancellation, &empty)?,
        _ => Archive::upgrade_v3_restored_in_place(path, &limits, &cancellation, &empty)?,
    };
    report_with_limits(&restored.document, &restored.diagnostics, &limits)
}
fn inspect(path: &Path, hold: bool) -> Result<()> {
    let limits = om9_app::document_limits();
    let coordinator = TransactionCoordinator::new(
        World::new(vec![Document::new("Inspection budget")], &limits)?, limits,
    )?;
    let sample = || om9_native_owner::physical_memory().map(|x| (x.total_bytes, x.available_bytes));
    let class = om9_core::budget::memory::MemoryClass::CpuCache;
    let (archive_memory, restored) = Archive::open_restored_retaining_reservation(
        path, &limits, &std::collections::BTreeSet::new(),
        |bytes| coordinator.reserve_external_memory(class, bytes.max(1), sample()),
        |reservation, bytes| coordinator.grow_external_memory_with_held(
            reservation, bytes, sample(), om9_core::budget::Usage::default(),
        ),
        |reservation, estimated| {
            let additional = estimated.saturating_sub(reservation.bytes());
            if additional == 0 { return Ok(()); }
            let batch = additional.max(64 * 1024);
            match coordinator.grow_external_memory_with_held(reservation, batch, sample(), om9_core::budget::Usage::default()) {
                Err(Error::Budget(_)) if batch != additional => coordinator.grow_external_memory_with_held(
                    reservation, additional, sample(), om9_core::budget::Usage::default(),
                ),
                result => result,
            }
        },
    )?;
    // Keep the conservative archive/restore lease, even after the input
    // buffer is gone, until the report and audit lifetime end.
    let report_memory = coordinator.reserve_remaining_external_memory_with_held(
        class, sample(), om9_core::budget::Usage::default(),
    )?;
    let mut report_limits = limits;
    report_limits.max_bytes = archive_memory.bytes().checked_add(report_memory.bytes())
        .ok_or(Error::Overflow("inspection report memory"))?.min(limits.max_bytes);
    report_with_limits(&restored.document, &restored.diagnostics, &report_limits)?;
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
/// Explicit plugin execution uses the same scene/Undo-aware service as CMD.
fn run_plugin(
    package: &Path,
    command: &std::ffi::OsStr,
    arguments: &Path,
    source: Option<&Path>,
    target: &Path,
) -> Result<()> {
    let limits = om9_app::document_limits();
    let command = command
        .to_str()
        .ok_or(Error::Invalid("UTF-8 plugin command"))?;
    use std::io::Read;
    let file = std::fs::File::open(arguments).map_err(|error| Error::Io(error.to_string()))?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Io(error.to_string()))?;
    if bytes.len() > 1024 * 1024 {
        return Err(Error::Budget("plugin argument JSON"));
    }
    let arguments: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| Error::Io(error.to_string()))?;
    if !arguments.is_object() {
        return Err(Error::Invalid("plugin arguments must be a JSON object"));
    }
    let mut controller = match source {
        Some(path) => om9_app::session::Controller::open_archive_for_batch(path, limits)?,
        None => om9_app::session::Controller::new(Document::with_template_tolerance("Plugin output"), limits)?,
    };
    let descriptor = controller.load_plugin(package)?;
    controller.run_loaded_plugin(&descriptor.id, command, arguments)?;
    controller.save_archive_for_batch(target)?;
    report_with_limits(controller.document(), &[], &limits)
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
        return om9_app::ui::launch_classified(Document::with_template_tolerance("Untitled"), None);
    }
    let command = args[0].to_str();
    match command {
        Some("--plugin-run") if args.len() == 5 => document_stage(run_plugin(
            Path::new(&args[1]),
            &args[2],
            Path::new(&args[3]),
            None,
            Path::new(&args[4]),
        )),
        Some("--plugin-run-on") if args.len() == 6 => document_stage(run_plugin(
            Path::new(&args[1]),
            &args[2],
            Path::new(&args[3]),
            Some(Path::new(&args[4])),
            Path::new(&args[5]),
        )),
        Some("--import-3dm") if args.len() == 3 => {
            document_stage(import_3dm(Path::new(&args[1]), Path::new(&args[2])))
        }
        Some("--export-3dm") if args.len() == 3 => {
            document_stage(export_3dm(Path::new(&args[1]), Path::new(&args[2])))
        }
        // Headless Model Context Protocol server: an agent client drives the same Controller as
        // CMD and reads typed state instead of screenshots.
        Some("--mcp") if args.len() == 1 => document_stage(om9_app::mcp::serve_stdio()),
        Some("--mcp-live") if args.len() == 1 => document_stage(om9_app::mcp::live::serve_stdio()),
        // Headless viewport capture: one PNG per workspace camera, no window or egui.
        Some("--render") if args.len() == 3 || args.len() == 4 => {
            let size = if args.len() == 4 {
                match om9_app::headless::parse_size(&args[3].to_string_lossy()) {
                    Ok(size) => size,
                    Err(error) => {
                        return Err(StartupFailure::from_error(StartupStage::Document, &error));
                    }
                }
            } else {
                (1280, 720)
            };
            document_stage(render_views(
                Path::new(&args[1]),
                Path::new(&args[2]),
                size.0,
                size.1,
            ))
        }
        // One 2x2 sheet of all four workspace cameras (tile size WIDTHxHEIGHT).
        Some("--render-4") if args.len() == 3 || args.len() == 4 => {
            let size = if args.len() == 4 {
                match om9_app::headless::parse_size(&args[3].to_string_lossy()) {
                    Ok(size) => size,
                    Err(error) => {
                        return Err(StartupFailure::from_error(StartupStage::Document, &error));
                    }
                }
            } else {
                (640, 400)
            };
            document_stage(render_sheet(
                Path::new(&args[1]),
                Path::new(&args[2]),
                size.0,
                size.1,
            ))
        }
        _ if args.len() != 2 => Err(StartupFailure::new(
            StartupStage::Document,
            "usage: om9 --create-fixture|--inspect|--audit|--migrate-v1|--migrate-v2|--migrate-v3 path.om9 | --import-3dm source.3dm target.om9 | --export-3dm source.om9 target.3dm | --plugin-run package.om9p command args.json output.om9 | --plugin-run-on package.om9p command args.json input.om9 output.om9 | --mcp | --render project.om9 out-dir [WIDTHxHEIGHT] | --render-4 project.om9 out.png [WIDTHxHEIGHT]",
        )),
        _ => {
            let path = Path::new(&args[1]);
            match command {
                Some("--ui-audit") => om9_app::ui::launch_classified(
                    Document::with_template_tolerance("Untitled"),
                    Some(path.to_path_buf()),
                ),
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
    // Explicit replay bootstrap barrier: the owning harness arms ETW before any swapchain exists.
    // This runs before the UI/event loop; normal application startup never waits for stdin.
    if std::env::var_os("OM9_NAVIGATION_AUDIT").is_some()
        && std::env::var("OM9_NAVIGATION_AUDIT_STDIN_GATE").is_ok_and(|v| v == "1")
    {
        let mut ready = String::new();
        if std::io::stdin().read_line(&mut ready).is_err() || ready.trim() != "ready" {
            std::process::exit(2);
        }
    }
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
