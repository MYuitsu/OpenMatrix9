// SPDX-License-Identifier: LGPL-2.1-or-later
//! The host owns user rail assets; both command adapters receive the same snapshot.
use super::Controller;
use om9_core::{Error, Result, builder::BuilderKind, geometry::{Geometry, Transform},
    ring_rail_shape::{RailShape, RailShapeLibrary, MAX_LIBRARY_BYTES}};
use std::{io::Write, path::PathBuf};

impl Controller {
    pub fn ring_rail_is_open(&self) -> bool {
        self.builder.as_ref().is_some_and(|builder| builder.kind() == BuilderKind::RingRail)
            || self.addon_session.as_ref().is_some_and(|session|
                matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail")
                    && matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11"))
    }
    pub(super) fn submit_ring_rail_shape(&mut self, text: &str) -> Option<Result<()>> {
        if !self.ring_rail_is_open() { return None; }
        let (name, value) = text.trim().split_once('=')?;
        if name.eq_ignore_ascii_case("Shape") { return Some(self.select_ring_rail_shape(value.trim())); }
        if name.eq_ignore_ascii_case("ShapeSave") {
            return Some(self.add_selected_ring_rail_shape(value.trim()).and_then(|_| self.select_ring_rail_shape(value.trim())));
        }
        None
    }
    pub fn ring_rail_shape_names(&self) -> Vec<String> {
        self.ring_rail_library.shapes().iter().map(|shape| shape.name().to_owned()).collect()
    }
    pub fn ring_rail_selected_shape(&self) -> Option<&RailShape> {
        if let Some(builder) = self.builder.as_ref().filter(|builder| builder.kind() == BuilderKind::RingRail) {
            builder.rail_shape()
        } else { self.ring_rail_shape.as_ref() }
    }
    pub fn ring_rail_library_shape(&self, name: &str) -> Option<&RailShape> { self.ring_rail_library.find(name) }

    pub fn load_ring_rail_library(&mut self, path: PathBuf) -> Result<()> {
        if !path.exists() { self.ring_rail_library_path = Some(path); return Ok(()); }
        if std::fs::metadata(&path).map_err(|e| Error::Io(e.to_string()))?.len() > MAX_LIBRARY_BYTES as u64 {
            return Err(Error::Budget("custom Ring Rail library"));
        }
        let text = std::fs::read_to_string(&path).map_err(|e| Error::Io(e.to_string()))?;
        let library = RailShapeLibrary::from_json(&text, self.coordinator.limits())?;
        self.ring_rail_library = library;
        self.ring_rail_library_path = Some(path);
        Ok(())
    }

    /// Save an edited asset, or replace the same name. File failure preserves the
    /// current library and preview; this operation never changes drawing Undo.
    pub fn store_ring_rail_shape(&mut self, shape: RailShape) -> Result<()> {
        let mut library = self.ring_rail_library.clone();
        library.store(shape, self.coordinator.limits())?;
        if let Some(path) = &self.ring_rail_library_path {
            let parent = path.parent().ok_or(Error::Invalid("Ring Rail library path"))?;
            std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
            let temporary = parent.join(format!(".ring-rail-{}.tmp", uuid::Uuid::new_v4()));
            let result = (|| -> std::io::Result<()> {
                let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
                file.write_all(library.to_json().as_bytes())?;
                file.sync_all()?;
                drop(file);
                std::fs::rename(&temporary, path)
            })();
            if let Err(error) = result { let _ = std::fs::remove_file(&temporary); return Err(Error::Io(error.to_string())); }
        }
        self.ring_rail_library = library;
        Ok(())
    }

    /// Snapshot exactly one canonical selected curve, with placement and model
    /// units baked to world mm. Display sampling is never the saved source.
    pub fn add_selected_ring_rail_shape(&mut self, name: &str) -> Result<()> {
        let selected = self.selected_ordered();
        let [id] = selected.as_slice() else { return Err(Error::Invalid("select one closed custom Ring Rail curve")); };
        let entity = self.document().entities.get(id).ok_or(Error::Missing("custom Ring Rail curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else { return Err(Error::Invalid("custom Ring Rail must be a curve")); };
        let curve = om9_core::top11::bake_curve(curve, entity.placement)?;
        let scale = self.document().page_unit.factor();
        let curve = om9_core::top11::bake_curve(&curve, Transform([[scale,0.,0.,0.], [0.,scale,0.,0.], [0.,0.,scale,0.], [0.,0.,0.,1.]]))?;
        let shape = RailShape::new(name, curve, self.coordinator.limits())?;
        self.store_ring_rail_shape(shape)
    }

    pub fn select_ring_rail_shape(&mut self, name: &str) -> Result<()> {
        if self.ring_rail_history_pending() { return Err(Error::Busy); }
        let shape = if name.eq_ignore_ascii_case("Round") { None } else {
            Some(self.ring_rail_library.find(name).ok_or(Error::Missing("custom Ring Rail shape"))?.clone())
        };
        if let Some(builder) = self.builder.as_ref().filter(|builder| builder.kind() == BuilderKind::RingRail) {
            if !self.builder_editable() { return Err(Error::Permission("Ring Rail builder")); }
            let before = builder.clone();
            self.builder.as_mut().expect("Ring Rail").set_rail_shape(shape.clone(), self.coordinator.limits())?;
            if let Err(error) = self.builder_refresh() { self.builder = Some(before); return Err(error); }
        } else if self.addon_session.as_ref().is_some_and(|session|
            matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail")
                && matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11")) {
            let before = self.ring_rail_shape.clone();
            self.ring_rail_shape = shape.clone();
            let builder = self.addon_builder.as_mut().ok_or(Error::Missing("Ring Rail builder"))?;
            builder.revision = builder.revision.wrapping_add(1);
            if let Err(error) = self.session_step(om9_addon_abi::session::Input::Selection) {
                self.ring_rail_shape = before;
                return Err(error);
            }
        } else { return Err(Error::Missing("open Ring Rail before choosing a shape")); }
        self.ring_rail_shape = shape;
        Ok(())
    }
}
