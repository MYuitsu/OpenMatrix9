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
        let (source, storage, proof, proof_storage) = self.reserve_attested_brep_candidate(output)?;
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
