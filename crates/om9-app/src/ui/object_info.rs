// SPDX-License-Identifier: LGPL-2.1-or-later

use eframe::egui;
#[cfg(feature = "occt-brep")]
use om9_core::object_info::Measurement;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform, Unit},
    identity::{DocumentId, ObjectId},
    model::Document,
    object_info::ObjectInfo,
};

#[derive(Clone, PartialEq)]
struct Key {
    document: DocumentId,
    document_epoch: u64,
    session_generation: u64,
    owner_session: om9_core::identity::SessionId,
    revision: u64,
    epoch: u64,
    id: ObjectId,
    generation: u64,
    geometry_revision: u64,
    placement: Transform,
    unit: Unit,
    limits: Limits,
}

struct Request {
    key: Key,
    document: std::sync::Arc<Document>,
    context: egui::Context,
}

struct Worker {
    requests: std::sync::mpsc::SyncSender<Request>,
    results: std::sync::mpsc::Receiver<(Key, Option<ObjectInfo>)>,
}

/// One bounded worker owns independent immutable documents; stale results are never displayed.
#[derive(Default)]
pub(super) struct AsyncCache {
    worker: Option<Worker>,
    pending: Option<Key>,
    ready: Option<(Key, Option<ObjectInfo>)>,
}

impl AsyncCache {
    pub(super) fn get(
        &mut self,
        document: std::sync::Arc<Document>,
        epoch: u64,
        id: ObjectId,
        limits: &Limits,
        context: &egui::Context,
    ) -> Option<ObjectInfo> {
        let entity = document.entities.get(&id)?;
        let key = Key {
            document: document.id,
            document_epoch: document.epoch,
            session_generation: document.session_generation,
            owner_session: document.owner_session,
            revision: document.revision,
            epoch,
            id,
            generation: entity.generation,
            geometry_revision: entity.geometry_revision,
            placement: entity.placement,
            unit: document.page_unit,
            limits: *limits,
        };
        if let Some(worker) = &self.worker {
            while let Ok((completed, info)) = worker.results.try_recv() {
                if self.pending.as_ref() == Some(&completed) {
                    self.pending = None;
                }
                if completed == key {
                    self.ready = Some((completed, info));
                }
            }
        }
        if let Some((_, info)) = self.ready.as_ref().filter(|(old, _)| *old == key) {
            return info.clone();
        }
        if self.pending.is_some() {
            return None;
        }
        if self.worker.is_none() {
            let (tx, rx) = std::sync::mpsc::sync_channel::<Request>(1);
            let (result_tx, result_rx) = std::sync::mpsc::channel();
            std::thread::Builder::new()
                .name("om9-object-info".into())
                .spawn(move || {
                    let mut cache = Cache::default();
                    while let Ok(request) = rx.recv() {
                        let info = cache
                            .get(&request.document, request.key.epoch, request.key.id, &request.key.limits)
                            .ok();
                        let _ = result_tx.send((request.key, info));
                        request.context.request_repaint();
                    }
                })
                .ok()?;
            self.worker = Some(Worker {
                requests: tx,
                results: result_rx,
            });
        }
        let request = Request {
            key: key.clone(),
            document,
            context: context.clone(),
        };
        if self.worker.as_ref()?.requests.try_send(request).is_ok() {
            self.pending = Some(key);
        }
        None
    }
}

#[derive(Default)]
pub(super) struct Cache(Option<(Key, ObjectInfo)>);

impl Cache {
    pub(super) fn get(
        &mut self,
        document: &Document,
        epoch: u64,
        id: ObjectId,
        limits: &Limits,
    ) -> om9_core::Result<ObjectInfo> {
        let entity = document
            .entities
            .get(&id)
            .ok_or(om9_core::Error::Missing("object info"))?;
        if !matches!(entity.geometry, Geometry::Brep(_)) {
            return document.object_info(id, limits);
        }
        let key = Key {
            document: document.id,
            document_epoch: document.epoch,
            session_generation: document.session_generation,
            owner_session: document.owner_session,
            revision: document.revision,
            epoch,
            id,
            generation: entity.generation,
            geometry_revision: entity.geometry_revision,
            placement: entity.placement,
            unit: document.page_unit,
            limits: *limits,
        };
        if let Some((_, info)) = self.0.as_ref().filter(|(old, _)| *old == key) {
            return Ok(info.clone());
        }
        let mut info = document.object_info(id, limits)?;
        // Portable validation checks source/display binding, not OCCT topology.
        info.valid = None;
        #[cfg(feature = "occt-brep")]
        if let Geometry::Brep(brep) = &entity.geometry {
            let measured = if entity.placement == Transform::identity() {
                om9_kernel::creation::measure(brep, limits).ok()
            } else {
                om9_kernel::transform_ops::transform_brep(
                    brep,
                    &entity.placement,
                    om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                    limits,
                )
                .ok()
                .and_then(|shape| om9_kernel::creation::measure(&shape, limits).ok())
            };
            if let Some(measured) = measured {
                info.valid = Some(true);
                let size = measured
                    .size_mm()
                    .map(|value| value / document.page_unit.factor());
                if size.iter().all(|value| value.is_finite() && *value >= 0.) {
                    info.dimensions = Measurement::Size(size);
                }
            }
        }
        self.0 = Some((key, info.clone()));
        Ok(info)
    }
}

#[cfg(all(test, feature = "occt-brep"))]
mod tests {
    use super::{AsyncCache, Cache};
    use crate::ui::MatrixApp;
    use om9_core::{
        budget::Limits,
        geometry::{Geometry, Transform, Unit},
        model::{Document, Entity},
    };
    use om9_kernel::creation::{self, DEFAULT_DEFLECTION_MM, Primitive};

    #[test]
    fn asynchronous_info_never_publishes_a_stale_selection() {
        use om9_core::object_info::Measurement;
        use std::{sync::Arc, time::{Duration, Instant}};
        let limits = Limits::default();
        let make = |size| creation::primitive(Primitive::Box { x_mm: size, y_mm: size, z_mm: size }, DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let mut doc = Document::new("async object info");
        let first = Entity::new("first", Geometry::Brep(make(4.)), None);
        let second = Entity::new("second", Geometry::Brep(make(8.)), None);
        let (first_id, second_id) = (first.id, second.id);
        doc.entities.insert(first_id, first);
        doc.entities.insert(second_id, second);
        let mut doc = Arc::new(doc);
        let ctx = eframe::egui::Context::default();
        let mut cache = AsyncCache::default();
        assert!(cache.get(doc.clone(), 1, first_id, &limits, &ctx).is_none());
        let until = Instant::now() + Duration::from_secs(10);
        let info = loop {
            if let Some(info) = cache.get(doc.clone(), 1, second_id, &limits, &ctx) { break info; }
            assert!(Instant::now() < until, "worker did not publish second object");
            std::thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(info.dimensions, Measurement::Size([8., 8., 8.]));
        assert_eq!(info.valid, Some(true));
        let edited = Arc::make_mut(&mut doc);
        edited.revision += 1;
        edited.entities.get_mut(&second_id).unwrap().placement.0[0][0] = 2.;
        assert!(cache.get(doc.clone(), 2, second_id, &limits, &ctx).is_none());
        let until = Instant::now() + Duration::from_secs(10);
        let info = loop {
            if let Some(info) = cache.get(doc.clone(), 2, second_id, &limits, &ctx) { break info; }
            assert!(Instant::now() < until, "worker did not publish edited object");
            std::thread::sleep(Duration::from_millis(5));
        };
        let Measurement::Size(size) = info.dimensions else { panic!("missing edited bounds") };
        for (actual, expected) in size.into_iter().zip([16., 8., 8.]) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn independent_native_measurements_can_run_concurrently() {
        let limits = Limits::default();
        let shape = creation::primitive(
            Primitive::Box { x_mm: 4., y_mm: 6., z_mm: 8. },
            DEFAULT_DEFLECTION_MM,
            &limits,
        ).unwrap();
        std::thread::scope(|scope| {
            for _ in 0..2 {
                let shape = shape.clone();
                scope.spawn(move || {
                    for _ in 0..3 {
                        let measured = creation::measure(&shape, &limits).unwrap();
                        assert_eq!(measured.size_mm(), [4., 6., 8.]);
                    }
                });
            }
        });
    }

    #[test]
    fn selected_brep_reports_world_bounds_in_document_units() {
        let limits = Limits::default();
        let shape = creation::primitive(
            Primitive::Box {
                x_mm: 4.,
                y_mm: 6.,
                z_mm: 8.,
            },
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("box", Geometry::Brep(shape), None);
        entity.placement = Transform([
            [0., -1., 0., 100.],
            [1., 0., 0., 200.],
            [0., 0., 1., 300.],
            [0., 0., 0., 1.],
        ]);
        let id = entity.id;
        let mut doc = Document::new("object info");
        doc.page_unit = Unit::Centimeter;
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.select_objects(&[id]).unwrap();
        let result = app.object_info_block().unwrap();
        assert_eq!(result["label"], "B-rep · 0.600 × 0.400 × 0.800 · valid");
    }

    #[test]
    fn bounds_cache_tracks_transform_unit_and_geometry_revision() {
        use om9_core::object_info::Measurement;
        let limits = Limits::default();
        let shape = creation::primitive(
            Primitive::Box {
                x_mm: 4.,
                y_mm: 6.,
                z_mm: 8.,
            },
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let entity = Entity::new("box", Geometry::Brep(shape), None);
        let id = entity.id;
        let mut doc = Document::new("bounds cache");
        doc.entities.insert(id, entity);
        let mut cache = Cache::default();
        let size = |cache: &mut Cache, doc: &Document| match cache
            .get(doc, 0, id, &limits)
            .unwrap()
            .dimensions
        {
            Measurement::Size(size) => size,
            other => panic!("missing bounds: {other:?}"),
        };
        assert_eq!(size(&mut cache, &doc), [4., 6., 8.]);
        let c = std::f64::consts::FRAC_1_SQRT_2;
        doc.entities.get_mut(&id).unwrap().placement = Transform([
            [c, -c, 0., 25.],
            [c, c, 0., -13.],
            [0., 0., 1., 7.],
            [0., 0., 0., 1.],
        ]);
        let rotated = size(&mut cache, &doc);
        assert!((rotated[0] - 10. * c).abs() < 5e-7, "{rotated:?}");
        assert!((rotated[1] - 10. * c).abs() < 5e-7, "{rotated:?}");
        assert!((rotated[2] - 8.).abs() < 5e-7, "{rotated:?}");
        doc.page_unit = Unit::Centimeter;
        let cm = size(&mut cache, &doc);
        assert!((cm[0] - c).abs() < 5e-8, "{cm:?}");
        let replacement = creation::primitive(
            Primitive::Box {
                x_mm: 2.,
                y_mm: 2.,
                z_mm: 2.,
            },
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let entity = doc.entities.get_mut(&id).unwrap();
        entity.geometry = Geometry::Brep(replacement);
        entity.geometry_revision += 1;
        let replaced = size(&mut cache, &doc);
        assert!((replaced[0] - 0.4 * c).abs() < 5e-8, "{replaced:?}");
        assert!((replaced[2] - 0.2).abs() < 5e-8, "{replaced:?}");
    }

    #[test]
    fn kernel_failure_never_reports_fabricated_dimensions() {
        use om9_core::object_info::Measurement;
        let limits = Limits::default();
        let shape = creation::primitive(
            Primitive::Box {
                x_mm: 4.,
                y_mm: 6.,
                z_mm: 8.,
            },
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let corrupt = om9_core::brep::Brep::from_native(
            "CASCADE Topology invalid".to_owned(),
            (*shape.display).clone(),
            &limits,
        )
        .unwrap();
        let entity = Entity::new("corrupt", Geometry::Brep(corrupt), None);
        let id = entity.id;
        let mut doc = Document::new("invalid kernel source");
        doc.entities.insert(id, entity);
        let info = Cache::default().get(&doc, 0, id, &limits).unwrap();
        assert!(matches!(info.dimensions, Measurement::Unavailable(_)));
        assert_eq!(info.valid, None);
    }

    #[test]
    fn rotated_sphere_does_not_expand_like_rotated_box_corners() {
        use om9_core::object_info::Measurement;
        let limits = Limits::default();
        let shape = creation::primitive(
            Primitive::Sphere { radius_mm: 5. },
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("sphere", Geometry::Brep(shape), None);
        let c = std::f64::consts::FRAC_1_SQRT_2;
        entity.placement = Transform([
            [c, -c, 0., 25.],
            [c, c, 0., -13.],
            [0., 0., 1., 7.],
            [0., 0., 0., 1.],
        ]);
        let id = entity.id;
        let mut doc = Document::new("sphere bounds");
        doc.entities.insert(id, entity);
        let info = Cache::default().get(&doc, 0, id, &limits).unwrap();
        let Measurement::Size(size) = info.dimensions else {
            panic!("sphere bounds unavailable")
        };
        for value in size {
            assert!((value - 10.).abs() < 5e-7, "{size:?}");
        }
    }
}
