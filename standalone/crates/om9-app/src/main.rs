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
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        return om9_app::ui::launch(Document::new("Untitled"), None);
    }
    if args.len() != 2 {
        return Err(Error::Invalid(
            "usage: om9 --create-fixture|--inspect|--audit|--migrate-v1|--migrate-v2|--migrate-v3 path.om9",
        ));
    }
    let path = Path::new(&args[1]);
    match args[0].to_str() {
        Some("--ui-audit") => {
            om9_app::ui::launch(Document::new("Untitled"), Some(path.to_path_buf()))
        }
        Some("--ui") => om9_app::ui::launch_file(path),
        Some("--create-fixture") => {
            let document = fixture();
            Archive::save(
                path,
                &document,
                &Limits::default(),
                &Cancellation::default(),
            )?;
            report(&document, &[])
        }
        Some("--migrate-v1") => {
            let restored = Archive::upgrade_v1_restored_in_place(
                path,
                &Limits::default(),
                &Cancellation::default(),
                &std::collections::BTreeSet::new(),
            )?;
            report(&restored.document, &restored.diagnostics)
        }
        Some("--migrate-v2") => {
            let restored = Archive::upgrade_v2_restored_in_place(
                path,
                &Limits::default(),
                &Cancellation::default(),
                &std::collections::BTreeSet::new(),
            )?;
            report(&restored.document, &restored.diagnostics)
        }
        Some("--migrate-v3") => {
            let restored = Archive::upgrade_v3_restored_in_place(
                path,
                &Limits::default(),
                &Cancellation::default(),
                &std::collections::BTreeSet::new(),
            )?;
            report(&restored.document, &restored.diagnostics)
        }
        Some("--inspect" | "--audit") => {
            let restored = Archive::open_restored(
                path,
                &Limits::default(),
                &std::collections::BTreeSet::new(),
            )?;
            report(&restored.document, &restored.diagnostics)?;
            if args[0] == "--audit" {
                use std::io::Write;
                std::io::stdout()
                    .flush()
                    .map_err(|e| Error::Io(e.to_string()))?;
                let mut release = String::new();
                // Audit owner controls this bounded diagnostic lifetime through
                // stdin. No background watcher or application child is started.
                std::io::stdin()
                    .read_line(&mut release)
                    .map_err(|e| Error::Io(e.to_string()))?;
            }
            Ok(())
        }
        _ => Err(Error::Invalid("standalone command line")),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("OM9: {error}");
        std::process::exit(1);
    }
}
