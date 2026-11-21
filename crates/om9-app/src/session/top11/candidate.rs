// SPDX-License-Identifier: LGPL-2.1-or-later
#[cfg(feature = "occt-brep")]
use super::*;
use om9_core::transaction::ExternalMemoryReservation;

pub(super) struct Top11Candidate {
    pub(super) reservation: Option<ExternalMemoryReservation>,
    #[cfg(feature = "occt-brep")]
    pub(super) proof: Option<(
        [u8; 32],
        om9_kernel::brep::TopologyAttestation,
        ExternalMemoryReservation,
    )>,
}

#[cfg(feature = "occt-brep")]
impl Controller {
    pub(super) fn reserve_top11_candidate(
        &self,
        output: om9_core::brep::Brep,
    ) -> Result<(om9_core::brep::CanonicalBrep, Top11Candidate)> {
        let held = self.scene_cache_usage()?;
        let limits = self.coordinator.limits();
        let physical =
            || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let source_bytes = output.storage_usage(limits)?.bytes;
        let mut input = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,
            source_bytes,
            physical(),
            held,
        )?;
        input.set_allocated_bytes(source_bytes)?;
        let mut storage = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache,
            1,
            physical(),
            held,
        )?;
        let snapshot = om9_kernel::brep::capture_source_topology(
            output.source(),
            output.display().deflection_mm,
            self.viewport_detail.angular(),
            limits,
            |live, pending| {
                let needed = live
                    .checked_add(pending)
                    .ok_or(Error::Overflow("Top11 topology capture"))?;
                if needed > storage.bytes() {
                    let extra = needed - storage.bytes();
                    self.coordinator.grow_external_memory_with_held(
                        &mut storage,
                        extra,
                        physical(),
                        held,
                    )?;
                }
                storage.set_allocated_bytes(live)?;
                storage.recheck_pending()
            },
        )
        .map_err(|error| Error::Io(format!("Top11 topology capture: {error}")))?;
        drop(output);
        drop(input);
        let (body, proof) = snapshot.into_parts();
        let proof_bytes = proof.storage_bytes()?;
        let bytes = body
            .storage_usage(limits)?
            .bytes
            .checked_add(proof_bytes)
            .ok_or(Error::Overflow("Top11 output topology"))?;
        storage.set_allocated_bytes(bytes)?;
        storage.shrink_to(bytes)?;
        let proof_storage = storage.split_off(proof_bytes)?;
        let source = self
            .coordinator
            .register_reserved_brep_candidate(body, &mut storage, held)?;
        let identity = source.canonical_fingerprints().1;
        Ok((
            source,
            Top11Candidate {
                reservation: Some(storage),
                proof: Some((identity, proof, proof_storage)),
            },
        ))
    }
}
