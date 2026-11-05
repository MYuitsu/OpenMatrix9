// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shape audit (tests only use it): when `OM9_AUDIT_DIR` is set, a test records the input
//! and output curves of a command run as sampled polylines, one JSON line per run, so the
//! shapes can be drawn and compared with what the command is documented to make.
use om9_addon_abi::session::{Geo, Vec3};
use std::io::Write;

/// Displayed points of a curve: a polyline's own corners, other curves densely sampled.
pub fn polyline_of(g: &Geo) -> Vec<Vec3> {
    match g {
        Geo::Point { point } => vec![*point],
        Geo::Polyline { points, closed } => {
            let mut p = points.clone();
            if *closed && !p.is_empty() {
                p.push(p[0]);
            }
            p
        }
        Geo::Line { start, end } => vec![*start, *end],
        _ => {
            let mut p = crate::nurbs::samples(g, 400);
            if matches!(g, Geo::Circle { .. }) && !p.is_empty() {
                p.push(p[0]);
            }
            p
        }
    }
}

/// Write one run to `$OM9_AUDIT_DIR/<file>-<test>-<n>.json` (no-op without the variable).
pub fn record(file: &str, test: &str, command: &str, inputs: &[Geo], outputs: &[Geo]) {
    let Ok(dir) = std::env::var("OM9_AUDIT_DIR") else {
        return;
    };
    let line = serde_json::json!({
        "test": test,
        "command": command,
        "inputs": inputs.iter().map(polyline_of).collect::<Vec<_>>(),
        "outputs": outputs.iter().map(polyline_of).collect::<Vec<_>>(),
        "kinds": outputs.iter().map(|g| match g {
            Geo::Point{..} => "point", Geo::Line{..} => "line", Geo::Polyline{..} => "polyline",
            Geo::Circle{..} => "circle", Geo::Arc{..} => "arc", Geo::Nurbs{..} => "nurbs", _ => "other"
        }).collect::<Vec<_>>(),
    });
    // One file per run: tests run in parallel, so one shared file would interleave lines.
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let k = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = format!("{file}-{}-{k}.json", test.replace([':', '/'], "_"));
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut f) = std::fs::File::create(std::path::Path::new(&dir).join(name)) {
        let _ = writeln!(f, "{line}");
    }
}
