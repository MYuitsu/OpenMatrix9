// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT producers and topology provider for the associative history (`OM9-L2-01-04`).
//!
//! `om9-core` owns the producer table, the history kernel and the analytic reference
//! backend. This module supplies the two pieces that need the native kernel:
//!
//! * [`OcctSweepBackend`] sweeps the profile along the rail through `creation::pipe_shell`;
//! * [`OcctTopology`] publishes the edges/faces/vertices of the resulting B-Rep snapshot so
//!   a semantic reference can resolve after a rebuild.
//!
//! [`history_registry`] is the one call a host (application or SDK host) needs: the rail and
//! profile stay pure Rust, the sweep runs in OCCT, and the topology policy reads the OCCT
//! snapshot.
use om9_core::{
    Error, Result,
    associative::Subshape,
    associative::{RebuildRequest, RebuiltOutput},
    budget::Limits,
    geometry::{Curve, Geometry},
    producers::{
        FeatureProducer, ProducerRegistry, RailProfileProducer, RailSweepProducer,
        RingRailProducer, SnapshotTopology, SweepBackend, TopologyProvider,
    },
};

use crate::creation::{self, DEFAULT_DEFLECTION_MM};

/// Rail/profile sampling used by the OCCT sweep. These are the same station counts the
/// existing Sweep 1/2 History path uses, so a rebuild and a fresh build agree.
const RAIL_SAMPLES: usize = 33;
const PROFILE_SAMPLES: usize = 24;

/// Sewing tolerance of the ring family solid, in millimetres.
const RING_SEW_TOLERANCE_MM: f64 = 1.0e-6;

/// The jewelry ring family as an OCCT B-Rep (`OM9-L2-04-03`, `OM9-L3-04-03-007`).
///
/// The sections come from `ring_builder::quad_mesh` — the same call the mesh producer uses —
/// and [`creation::sewed_solid`] sews them into one closed solid. The shared feasibility gate
/// then measures the result, so a B-Rep body and the mesh preview agree on closedness and
/// volume instead of each carrying its own rules.
#[derive(Debug, Clone, Copy)]
pub struct RingFamilyBrepProducer {
    pub kind: om9_core::ring_builder::RingBuilderKind,
}

impl FeatureProducer for RingFamilyBrepProducer {
    fn produce(&self, request: &RebuildRequest) -> Result<Vec<RebuiltOutput>> {
        let output = match request.outputs.as_slice() {
            [output] => *output,
            _ => return Err(Error::Invalid("producer output count")),
        };
        let params = om9_core::ring_builder::RingParams::from_request(self.kind, request)?;
        let (points, quads, _wall) = om9_core::ring_builder::quad_mesh(&params)?;
        // The mesh path keeps `Vec<Vec<u32>>` faces; the sewing adapter takes fixed quads.
        let mut faces: Vec<[u32; 4]> = Vec::with_capacity(quads.len());
        for quad in &quads {
            if quad.len() != 4 {
                return Err(Error::Invalid("ring solid face"));
            }
            faces.push([quad[0], quad[1], quad[2], quad[3]]);
        }
        let limits = Limits::default();
        let brep = creation::sewed_solid(
            &points,
            &faces,
            RING_SEW_TOLERANCE_MM,
            DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .map_err(|error| {
            Error::Unsupported(match error {
                creation::CreationError::Invalid(_) => "OCCT ring solid input",
                creation::CreationError::Budget(_) => "OCCT ring solid budget",
                creation::CreationError::Kernel(_) => "OCCT ring solid kernel",
                creation::CreationError::StaleHandle => "OCCT ring solid handle",
            })
        })?;
        let geometry = Geometry::Brep(brep);
        // The same gate the mesh path runs: a body that is not a closed ring with a positive
        // volume never reaches the document, whichever backend built it.
        om9_core::ring_builder::inspect_geometry(&geometry, &limits)?;
        Ok(vec![RebuiltOutput {
            object: output,
            geometry,
        }])
    }
}

/// The OCCT sweep: profile wire swept along the rail spine, one B-Rep result.
#[derive(Debug, Clone, Copy, Default)]
pub struct OcctSweepBackend;

impl SweepBackend for OcctSweepBackend {
    fn sweep(
        &self,
        rail: &Curve,
        profile: &Curve,
        solid: bool,
        limits: &Limits,
    ) -> Result<Geometry> {
        let spine = om9_core::surface_kernel::sample_curve(rail, RAIL_SAMPLES, limits)
            .map_err(|_| Error::Unsupported("OCCT sweep rail sampling"))?;
        let section = om9_core::surface_kernel::sample_curve(profile, PROFILE_SAMPLES, limits)
            .map_err(|_| Error::Unsupported("OCCT sweep profile sampling"))?;
        let brep = creation::pipe_shell(
            &spine,
            None,
            &[section],
            solid,
            DEFAULT_DEFLECTION_MM,
            limits,
        )
        .map_err(|_| Error::Unsupported("OCCT sweep pipe shell"))?;
        Ok(Geometry::Brep(brep))
    }
}

/// Topology from the B-Rep snapshot OCCT publishes.
///
/// The snapshot is the bridge's own face/edge/vertex record (`brep_bridge.cpp`); this
/// provider refuses a `Brep` whose source is not an OCCT shape instead of naming native ids
/// of an unknown producer.
#[derive(Debug, Clone, Copy, Default)]
pub struct OcctTopology;

impl TopologyProvider for OcctTopology {
    fn subshapes(&self, geometry: &Geometry, limits: &Limits) -> Result<Vec<Subshape>> {
        if let Geometry::Brep(brep) = geometry
            && !brep.source().contains("CASCADE Topology")
        {
            return Err(Error::Invalid("OCCT topology source"));
        }
        SnapshotTopology.snapshot_subshapes(geometry, limits)
    }
}

/// The builder chain with the OCCT sweep and the OCCT snapshot topology.
pub fn history_registry() -> Result<ProducerRegistry> {
    let mut registry = ProducerRegistry::with_topology(OcctTopology);
    registry.register("RingRail", RingRailProducer)?;
    registry.register("RailProfile", RailProfileProducer)?;
    registry.register("RailSweep", RailSweepProducer::new(OcctSweepBackend))?;
    registry.register("OutsideRail", om9_core::producers::OutsideRailProducer)?;
    registry.register("RingResizer", om9_core::producers::RingResizerProducer)?;
    // The jewelry ring family (`OM9-L2-04-03`) is built as an OCCT B-Rep here: the same
    // sections the portable mesh producer samples are sewn into one solid, so the builder
    // panel, the history rebuild and an exported body share one shape.
    for family in om9_core::ring_builder::RingBuilderKind::ALL {
        registry.register(family.command(), RingFamilyBrepProducer { kind: family })?;
    }
    Ok(registry)
}

/// Register the chain into an existing registry (a host that already owns a topology
/// policy can keep it).
pub fn register_occt_producers(registry: &mut ProducerRegistry) -> Result<()> {
    registry.register("RingRail", RingRailProducer)?;
    registry.register("RailProfile", RailProfileProducer)?;
    registry.register("RailSweep", RailSweepProducer::new(OcctSweepBackend))?;
    registry.register("OutsideRail", om9_core::producers::OutsideRailProducer)?;
    registry.register("RingResizer", om9_core::producers::RingResizerProducer)?;
    for family in om9_core::ring_builder::RingBuilderKind::ALL {
        registry.register(family.command(), RingFamilyBrepProducer { kind: family })?;
    }
    Ok(())
}
