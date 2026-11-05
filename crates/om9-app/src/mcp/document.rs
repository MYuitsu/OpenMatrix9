// SPDX-License-Identifier: LGPL-2.1-or-later
//! MCP document lifecycle tools: new, open, save and full report.
use super::{NO_DOCUMENT, Server, message, required_str};
use crate::session::Controller;
use om9_archive::Archive;
use om9_core::{model::Document, transaction::Cancellation};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

impl Server {
    pub(super) fn new_document(&mut self, args: &Value) -> Result<Value, String> {
        let name = args
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Untitled");
        let document = Document::with_template_tolerance(name);
        let mut controller = Controller::new(document, self.limits).map_err(message)?;
        if args.get("benchmark_no_undo").and_then(Value::as_bool) == Some(true) {
            controller.disable_undo_for_benchmark().map_err(message)?;
        }
        controller.record_changes();
        self.controller = Some(controller);
        self.path = None;
        self.last_sequence = 0;
        Ok(self.snapshot(100))
    }

    pub(super) fn open_document(&mut self, args: &Value) -> Result<Value, String> {
        let path = path_arg(args, "path")?;
        let mut controller = Controller::from_file(&path, self.limits).map_err(message)?;
        controller.record_changes();
        self.controller = Some(controller);
        self.path = Some(path);
        self.last_sequence = 0;
        Ok(self.snapshot(100))
    }

    pub(super) fn save(&mut self, args: &Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(Value::as_str) {
            Some(path) => PathBuf::from(path),
            None => self
                .path
                .clone()
                .ok_or("missing `path`: no document path is open yet")?,
        };
        {
            let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
            Archive::save(
                &path,
                controller.document(),
                &self.limits,
                &Cancellation::default(),
            )
            .map_err(message)?;
        }
        self.path = Some(path.clone());
        let bytes = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        Ok(json!({
            "saved": path.display().to_string(),
            "bytes": bytes,
            "state": self.snapshot(0),
        }))
    }

    pub(super) fn export_stl(&mut self, args: &Value) -> Result<Value, String> {
        let path = path_arg(args, "path")?;
        let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
        let linear = match args.get("linear_tolerance_mm") {
            Some(value) => value.as_f64().ok_or("linear_tolerance_mm must be a number")?,
            None => controller.document().effective_tolerance().map_err(message)?.absolute_mm,
        };
        let angular = match args.get("angular_tolerance_rad") {
            Some(value) => value.as_f64().ok_or("angular_tolerance_rad must be a number")?,
            None => 0.3,
        };
        let receipt = controller.export_stl_selected_with_profile(&path, crate::session::StlExportProfile { linear_tolerance_mm: linear, angular_tolerance_rad: angular }).map_err(message)?;
        Ok(json!({
            "path": receipt.path.display().to_string(),
            "triangles": receipt.triangles,
            "bytes": receipt.bytes,
            "requested_tolerance_mm": receipt.requested_tolerance_mm,
            "requested_angle_rad": receipt.requested_angle_rad,
            "max_occt_topology_tolerance_upper_mm": receipt.max_occt_topology_tolerance_upper_mm,
            "source_document_id": receipt.source_document_id.to_string(),
            "source_epoch": receipt.source_epoch,
            "open_edges": receipt.open_edges,
            "non_manifold_edges": receipt.non_manifold_edges,
            "inconsistent_edges": receipt.inconsistent_edges,
            "disconnected_vertices": receipt.disconnected_vertices,
            "edge_connected_components": receipt.edge_connected_components,
            "duplicate_faces": receipt.duplicate_faces,
            "encoded_vertex_identity_ambiguous": receipt.encoded_vertex_identity_ambiguous,
            "encoded_closed_by_topology_checks": receipt.encoded_closed_by_topology_checks(),
            "self_intersections_checked": false,
            "topology_scope": "source_scoped_indexed_soup_before_encoding",
            "geometric_deviation_verification": "unverified",
            "watertight": receipt.watertight(),
            "skipped": receipt.skipped,
            "warnings": receipt.warnings,
            "units": "mm",
            "stl_unit_metadata": false,
        }))
    }

    pub(super) fn report(&mut self, args: &Value) -> Result<Value, String> {        let path = match args.get("path").and_then(Value::as_str) {
            Some(path) => PathBuf::from(path),
            None => std::env::temp_dir().join(format!("om9-report-{}.json", std::process::id())),
        };
        let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
        let mut file = File::create(&path).map_err(message)?;
        crate::report::write(
            &mut file,
            controller.document(),
            controller.file_diagnostics(),
            &self.limits,
        )
        .map_err(message)?;
        file.flush().map_err(message)?;
        let bytes = file.metadata().map(|meta| meta.len()).unwrap_or(0);
        Ok(json!({"path": path.display().to_string(), "bytes": bytes}))
    }
}

fn path_arg(args: &Value, key: &str) -> Result<PathBuf, String> {
    required_str(args, key)
        .map(Path::new)
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod stl_lifetime_tests {
    use super::*;
    use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::Entity};

    #[test]
    fn failed_headless_open_keeps_capture_but_new_replacement_cancels_it() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = Document::with_template_tolerance("headless capture");
        let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let mut server = Server::new();
        server.controller = Some(controller);
        let missing = std::env::temp_dir().join(format!("om9-missing-{}.om9", uuid::Uuid::new_v4()));
        assert!(server.open_document(&json!({"path": missing})).is_err());
        assert!(capture.belongs_to_session(server.controller.as_ref().unwrap().document()));
        server.new_document(&json!({"name":"replacement"})).unwrap();
        let target = std::env::temp_dir().join(format!("om9-mcp-stl-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"previous destination").unwrap();
        assert!(matches!(capture.export_with_profile(&target, crate::session::StlExportProfile {
            linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3,
        }), Err(om9_core::Error::Canceled)));
        assert_eq!(std::fs::read(&target).unwrap(), b"previous destination");
        std::fs::remove_file(target).unwrap();
    }
}
