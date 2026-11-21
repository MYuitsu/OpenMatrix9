// SPDX-License-Identifier: LGPL-2.1-or-later
//! Build test documents through the same combined source/display import as v4 files.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, NativeBrep},
    model::Document,
};
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct Displays {
    displays: Vec<(Geometry, serde_json::Value)>,
    #[cfg(feature = "occt-brep")]
    proofs: std::cell::RefCell<Vec<(Geometry, om9_kernel::brep::TopologyAttestation)>>,
}

impl Displays {
    #[allow(dead_code)]
    pub fn brep(&mut self, body: om9_core::brep::Brep) -> Geometry {
        #[cfg(feature = "occt-brep")]
        let (body, proof) = om9_kernel::brep::capture_source_topology(
            body.source(),
            body.display().deflection_mm,
            0.3,
            &Limits::default(),
            |_, _| Ok(()),
        )
        .unwrap()
        .into_parts();
        let combined = serde_json::to_value(&body).unwrap();
        let geometry = Geometry::Brep(body.into_canonical_and_representation().0);
        #[cfg(feature = "occt-brep")]
        self.proofs.borrow_mut().push((geometry.clone(), proof));
        self.displays.push((geometry.clone(), combined));
        geometry
    }

    pub fn native(&mut self, body: NativeBrep) -> Geometry {
        let geometry = Geometry::NativeBrep(body.canonical_source());
        self.displays
            .push((geometry.clone(), serde_json::to_value(body).unwrap()));
        geometry
    }

    pub fn controller(&self, document: Document, limits: Limits) -> om9_core::Result<Controller> {
        if self.displays.is_empty() {
            return Controller::new(document, limits);
        }
        let mut metadata = serde_json::to_value(&document).unwrap();
        for entity in document.entities.values() {
            if let Some((geometry, full)) =
                self.displays.iter().find(|(g, _)| g == &entity.geometry)
            {
                let tag = if matches!(geometry, Geometry::NativeBrep(_)) {
                    "NativeBrep"
                } else {
                    "Brep"
                };
                metadata["entities"][entity.id.0.to_string()]["geometry"][tag] = full.clone();
            }
        }
        let metadata = serde_json::to_vec(&metadata).unwrap();
        let mut archive = vec![0; 88];
        archive[..8].copy_from_slice(b"OM9\0\r\n\x1a\n");
        archive[8..12].copy_from_slice(&4u32.to_le_bytes());
        archive[12..16].copy_from_slice(&1u32.to_le_bytes());
        archive[16..20].copy_from_slice(&1u32.to_le_bytes());
        archive[40..48].copy_from_slice(&88u64.to_le_bytes());
        archive[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
        archive[56..88].copy_from_slice(&Sha256::digest(&metadata));
        archive.extend_from_slice(&metadata);
        let path = std::env::temp_dir().join(format!("om9-top11-{}.om9", uuid::Uuid::new_v4()));
        std::fs::write(&path, archive).unwrap();
        let controller = Controller::open_archive_for_batch(&path, limits);
        std::fs::remove_file(path).unwrap();
        let mut controller = controller?;
        #[cfg(feature = "occt-brep")]
        for (geometry, proof) in self.proofs.take() {
            if let Some(entity) = controller
                .document()
                .entities
                .values()
                .find(|entity| entity.geometry == geometry)
            {
                let stamp = controller.document().stamp(entity.id)?;
                let bytes = proof.storage_bytes()?;
                let mut storage = controller.coordinator().reserve_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::CpuCache,
                    bytes,
                    None,
                    controller.scene_cache_usage()?,
                )?;
                storage.set_allocated_bytes(bytes)?;
                controller.adopt_topology_proof(stamp, proof, storage)?;
            }
        }
        Ok(controller)
    }
}

/// Serialize the live canonical sources with the production archive writer,
/// including the isolated topology worker for documents containing BReps.
pub fn round_trip(controller: &Controller, path: &std::path::Path, limits: Limits) -> Document {
    controller
        .save_archive_for_batch(path)
        .unwrap_or_else(|error| {
            panic!("save: {error}. Events: {:?}", controller.events().events())
        });
    let mut reopened = Controller::open_archive_for_batch(path, limits)
        .unwrap()
        .document()
        .clone();
    let source = controller.document();
    assert_ne!(reopened.owner_session, source.owner_session);
    assert!(reopened.session_generation > source.session_generation);
    // Opening assigns fresh runtime ownership. Compare every persisted field.
    reopened.owner_session = source.owner_session;
    reopened.session_generation = source.session_generation;
    reopened.epoch = source.epoch;
    reopened
}
