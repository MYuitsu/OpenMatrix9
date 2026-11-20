// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT producers and topology provider for the associative history (`OM9-L2-01-04`).
//!
//! `om9-core` owns the producer table, the history kernel and the analytic reference
//! backend. This module supplies the two pieces that need the native kernel:
//!
//! * [`OcctSweepBackend`] sweeps the profile along the rail through `creation::pipe_shell`;
//! * [`OcctTopology`] measures edges/faces/vertices of the resulting B-Rep source so
//!   a semantic reference can resolve after a rebuild.
//!
//! [`history_registry`] is the one call a host (application or SDK host) needs: the rail and
//! profile stay pure Rust, the sweep runs in OCCT, and the topology policy reads the OCCT
//! source.
use om9_core::{
    Error, Result,
    associative::Subshape,
    associative::{RebuildRequest, RebuiltOutput},
    budget::{Limits, Usage},
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
        self.produce_admitted(request, &Limits::default(), |_, _| Ok(()))
    }
    fn produce_with_candidates(&self, request: &RebuildRequest) -> Result<(Vec<RebuiltOutput>, Vec<om9_core::brep::Brep>)> {
        self.produce_with_signatures(request, &Limits::default(), |_, _| Ok(()), || false, false)
            .map(|(outputs, _, candidates)| (outputs, candidates))
    }
}

impl RingFamilyBrepProducer {
    /// Account ring staging and snapshot allocations under a caller-owned
    /// reservation. Native OCCT internal allocations remain outside this hook.
    pub fn produce_admitted(
        &self,
        request: &RebuildRequest,
        limits: &Limits,
        admit: impl FnMut(u64, u64) -> Result<()>,
    ) -> Result<Vec<RebuiltOutput>> {
        self.produce_admitted_cancellable(request, limits, admit, || false)
    }

    pub fn produce_admitted_cancellable(
        &self,
        request: &RebuildRequest,
        limits: &Limits,
        admit: impl FnMut(u64, u64) -> Result<()>,
        canceled: impl Fn() -> bool + Sync,
    ) -> Result<Vec<RebuiltOutput>> {
        self.produce_with_signatures(request, limits, admit, canceled, false).map(|(outputs, _, _)| outputs)
    }

    fn produce_with_signatures(
        &self, request: &RebuildRequest, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>, canceled: impl Fn() -> bool + Sync, signatures: bool,
    ) -> Result<(Vec<RebuiltOutput>, Vec<crate::brep::SourceSignature>, Vec<om9_core::brep::Brep>)> {
        let output = match request.outputs.as_slice() {
            [output] => *output,
            _ => return Err(Error::Invalid("producer output count")),
        };
        let params = om9_core::ring_builder::RingParams::from_request(self.kind, request)?;
        // The ring sampler has a fixed number of vertices and quads. Admit the
        // caller-owned tables before building them; their actual capacities
        // remain held while the native adapter constructs its snapshot.
        let count = om9_core::ring_builder::SEGMENTS.checked_mul(4).ok_or(Error::Overflow("ring section count"))?;
        let mut planned = Usage::array::<om9_core::geometry::Point3>(count)?;
        planned.add(Usage::array::<Vec<u32>>(count)?, limits)?;
        planned.add(Usage::array::<u32>(count.checked_mul(4).ok_or(Error::Overflow("ring indices"))?)?, limits)?;
        planned.add(Usage::array::<[u32; 4]>(count)?, limits)?;
        admit(0, planned.bytes)?;
        let (points, quads, _wall) = om9_core::ring_builder::quad_mesh(&params)?;
        // The mesh path keeps `Vec<Vec<u32>>` faces; the sewing adapter takes fixed quads.
        let mut faces: Vec<[u32; 4]> = Vec::new();
        faces.try_reserve_exact(quads.len()).map_err(|_| Error::Budget("ring face table"))?;
        for quad in &quads {
            if quad.len() != 4 {
                return Err(Error::Invalid("ring solid face"));
            }
            faces.push([quad[0], quad[1], quad[2], quad[3]]);
        }
        let mut held = Usage::array::<om9_core::geometry::Point3>(points.capacity())?;
        held.add(Usage::array::<Vec<u32>>(quads.capacity())?, limits)?;
        for quad in &quads { held.add(Usage::array::<u32>(quad.capacity())?, limits)?; }
        held.add(Usage::array::<[u32; 4]>(faces.capacity())?, limits)?;
        admit(held.bytes, 0)?;
        let snapshot_limits = limits.remaining_after(held)?;
        let (brep, source_signatures, volume_mm3) = if signatures { creation::sewed_solid_with_signatures_cancellable(
            &points,
            &faces,
            RING_SEW_TOLERANCE_MM,
            DEFAULT_DEFLECTION_MM,
            &snapshot_limits,
            |live, pending| admit(held.bytes.checked_add(live).ok_or(Error::Overflow("ring production"))?, pending),
            &canceled,
        ) } else { creation::sewed_solid_with_volume_cancellable(
            &points, &faces, RING_SEW_TOLERANCE_MM, DEFAULT_DEFLECTION_MM, &snapshot_limits,
            |live, pending| admit(held.bytes.checked_add(live).ok_or(Error::Overflow("ring production"))?, pending), &canceled,
        ).map(|(brep, volume)| (brep, Vec::new(), volume)) }
        .map_err(|error| match error {
            creation::SewSolidError::Admission(error) => error,
            creation::SewSolidError::Creation(error) => Error::Unsupported(match error {
                creation::CreationError::Invalid(_) => "OCCT ring solid input",
                creation::CreationError::Budget(_) => "OCCT ring solid budget",
                creation::CreationError::Kernel(_) => "OCCT ring solid kernel",
                creation::CreationError::StaleHandle => "OCCT ring solid handle",
            }),
        })?;
        // The same gate the mesh path runs: a body that is not a closed ring with a positive
        // volume never reaches the document, whichever backend built it.
        om9_core::ring_builder::inspect_measures(true, volume_mm3)?;
        drop(points);
        drop(quads);
        drop(faces);
        let brep_bytes = brep.storage_usage(limits)?.bytes.checked_add(Usage::array::<crate::brep::SourceSignature>(source_signatures.capacity())?.bytes)
            .ok_or(Error::Overflow("ring source signatures"))?;
        let output_bytes = Usage::array::<RebuiltOutput>(1)?.bytes;
        Usage { bytes: brep_bytes.checked_add(output_bytes).ok_or(Error::Overflow("ring output"))?, ..Usage::default() }.check(limits)?;
        admit(brep_bytes, output_bytes)?;
        let geometry = Geometry::Brep(om9_core::brep::BrepSourceAccess::to_canonical(&brep));
        let mut outputs = Vec::new();
        outputs.try_reserve_exact(1).map_err(|_| Error::Budget("ring output table"))?;
        let live = brep_bytes.checked_add(Usage::array::<RebuiltOutput>(outputs.capacity())?.bytes)
            .ok_or(Error::Overflow("ring output"))?;
        Usage { bytes: live, ..Usage::default() }.check(limits)?;
        admit(live, 0)?;
        outputs.push(RebuiltOutput {
            object: output,
            geometry,
        });
        Ok((outputs, source_signatures, vec![brep]))
    }

    /// Build the ring and its current topology policy under one caller-owned lease.
    /// Names source measurements with a distinct namespace from legacy display roles.
    pub fn produce_feature_admitted(
        &self,
        request: &RebuildRequest,
        limits: &Limits,
        admit: impl FnMut(u64, u64) -> Result<()>,
    ) -> Result<om9_core::associative::ProducedFeature> {
        self.produce_feature_cancellable(request, limits, admit, || false)
    }

    pub fn produce_feature_cancellable(
        &self,
        request: &RebuildRequest,
        limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>,
        canceled: impl Fn() -> bool + Sync,
    ) -> Result<om9_core::associative::ProducedFeature> {
        use om9_core::associative::{ProducedFeature, ProducedSubshapes};
        let (outputs, source_signatures, brep_candidates) = self.produce_with_signatures(request, limits, &mut admit, &canceled, true)?;
        let output = &outputs[0];
        let Geometry::Brep(brep) = &output.geometry else { return Err(Error::Invalid("ring BRep output")); };
        let mut base = brep_candidates[0].storage_usage(limits)?.bytes
            .checked_add(Usage::array::<om9_core::brep::Brep>(brep_candidates.capacity())?.bytes).ok_or(Error::Overflow("ring candidates"))?
            .checked_add(Usage::array::<crate::brep::SourceSignature>(source_signatures.capacity())?.bytes)
            .ok_or(Error::Overflow("ring source signatures"))?
            .checked_add(Usage::array::<RebuiltOutput>(outputs.capacity())?.bytes)
            .ok_or(Error::Overflow("ring feature output"))?;
        let planned = Usage::array::<ProducedSubshapes>(1)?.bytes;
        Usage { bytes: base.checked_add(planned).ok_or(Error::Overflow("ring feature topology"))?, ..Usage::default() }.check(limits)?;
        admit(base, planned)?;
        let mut subshapes = Vec::new();
        subshapes.try_reserve_exact(1).map_err(|_| Error::Budget("ring topology envelope"))?;
        base = base.checked_add(Usage::array::<ProducedSubshapes>(subshapes.capacity())?.bytes)
            .ok_or(Error::Overflow("ring feature topology"))?;
        Usage { bytes: base, ..Usage::default() }.check(limits)?;
        admit(base, 0)?;
        let topology_limits = limits.remaining_after(Usage { bytes: base, ..Usage::default() })?;
        let names = source_subshapes_admitted(&source_signatures, &topology_limits, &mut |live, pending| {
            admit(base.checked_add(live).ok_or(Error::Overflow("ring feature topology"))?, pending)
        }, &canceled)?;
        let signature_bytes = Usage::array::<crate::brep::SourceSignature>(source_signatures.capacity())?.bytes;
        let name_bytes = names.iter().try_fold(Usage::array::<Subshape>(names.capacity())?.bytes, |sum, shape|
            sum.checked_add(shape.name.capacity() as u64).ok_or(Error::Overflow("ring feature names")))?;
        drop(source_signatures);
        admit(base.checked_sub(signature_bytes).and_then(|v| v.checked_add(name_bytes))
            .ok_or(Error::Overflow("ring feature topology"))?, 0)?;
        subshapes.push(ProducedSubshapes { output: output.object, subshapes: names });
        Ok(ProducedFeature { outputs, subshapes, brep_candidates })
    }
}

/// Inspect B-Rep rings from the OCCT source, never from viewport triangles.
/// Authored mesh geometry retains its portable edge/volume inspection.
pub fn inspect_ring_geometry(geometry: &Geometry, limits: &Limits) -> Result<om9_core::ring_builder::SolidReport> {
    match geometry {
        Geometry::Brep(brep) => {
            let measured = creation::measure(brep, limits).map_err(|error| Error::Io(format!("ring BRep measurement: {error}")))?;
            om9_core::ring_builder::inspect_measures(measured.closed && measured.solids == 1
                && measured.shells == 1 && measured.naked_edges == 0, measured.volume_mm3)
        }
        _ => om9_core::ring_builder::inspect_geometry(geometry, limits),
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
        self.sweep_with_candidates(rail, profile, solid, limits).map(|(geometry, _)| geometry)
    }
    fn sweep_with_candidates(&self, rail: &Curve, profile: &Curve, solid: bool,
        limits: &Limits) -> Result<(Geometry, Vec<om9_core::brep::Brep>)> {
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
        let source = om9_core::brep::BrepSourceAccess::to_canonical(&brep);
        Ok((Geometry::Brep(source), vec![brep]))
    }
}

/// Topology from the B-Rep snapshot OCCT publishes.
///
/// The snapshot is the bridge's own face/edge/vertex record (`brep_bridge.cpp`); this
/// provider refuses a `Brep` whose source is not an OCCT shape instead of naming native ids
/// of an unknown producer.
#[derive(Debug, Clone, Copy, Default)]
pub struct OcctTopology;

fn source_subshapes_admitted(values: &[crate::brep::SourceSignature], limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>, canceled: &dyn Fn() -> bool) -> Result<Vec<Subshape>> {
    use std::fmt::Write;
    use om9_core::associative::SubshapeKind;
    let planned = Usage::array::<Subshape>(values.len())?.bytes;
    Usage { bytes: planned, ..Usage::default() }.check(limits)?;
    admit(0, planned)?;
    let mut out = Vec::new();
    out.try_reserve_exact(values.len()).map_err(|_| Error::Budget("source subshape table"))?;
    let mut live = Usage::array::<Subshape>(out.capacity())?.bytes;
    Usage { bytes: live, ..Usage::default() }.check(limits)?;
    admit(live, 0)?;
    for value in values {
        if canceled() { return Err(Error::Canceled); }
        let prefix = match value.kind {
            SubshapeKind::Face => "source:face:", SubshapeKind::Edge => "source:edge:",
            SubshapeKind::Vertex => "source:vertex:", _ => return Err(Error::Invalid("source subshape kind")),
        };
        let digits = value.source_index.checked_ilog10().unwrap_or(0) as usize + 1;
        let capacity = prefix.len().checked_add(digits).ok_or(Error::Overflow("source subshape name"))?;
        let required = live.checked_add(capacity as u64).ok_or(Error::Overflow("source subshape names"))?;
        Usage { bytes: required, ..Usage::default() }.check(limits)?;
        admit(live, capacity as u64)?;
        let mut name = String::new();
        name.try_reserve_exact(capacity).map_err(|_| Error::Budget("source subshape name"))?;
        live = live.checked_add(name.capacity() as u64).ok_or(Error::Overflow("source subshape names"))?;
        Usage { bytes: live, ..Usage::default() }.check(limits)?;
        admit(live, 0)?;
        write!(&mut name, "{prefix}{}", value.source_index).map_err(|_| Error::Invalid("source subshape name"))?;
        out.push(Subshape { name, kind: value.kind, signature: value.signature });
    }
    Ok(out)
}

impl TopologyProvider for OcctTopology {
    fn subshapes(&self, geometry: &Geometry, limits: &Limits) -> Result<Vec<Subshape>> {
        if let Geometry::Brep(brep) = geometry {
            if !brep.source().contains("CASCADE Topology") { return Err(Error::Invalid("OCCT topology source")); }
            let values = crate::brep::source_signatures(brep.source(), limits, |_, _| Ok(()), || false)?;
            let held = Usage::array::<crate::brep::SourceSignature>(values.capacity())?.bytes;
            let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
            return source_subshapes_admitted(&values, &remaining, &mut |_, _| Ok(()), &|| false);
        }
        SnapshotTopology.snapshot_subshapes(geometry, limits)
    }
}

/// The builder chain with the OCCT sweep and source-derived topology.
pub fn history_registry() -> Result<ProducerRegistry> {
    let mut registry = ProducerRegistry::with_topology(OcctTopology);
    registry.register(
        om9_core::producers::JOIN_HISTORY_COMMAND,
        om9_core::producers::JoinHistoryProducer,
    )?;
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
    registry.register(
        om9_core::producers::JOIN_HISTORY_COMMAND,
        om9_core::producers::JoinHistoryProducer,
    )?;
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
