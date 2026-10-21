// SPDX-License-Identifier: LGPL-2.1-or-later
//! Optional OCCT implementation of the 3DM BRep import bridge.
//!
//! The exchange crate owns the transaction and BRep policy; this module only
//! forwards an already-projected native payload to the complete kernel
//! operation. It is absent unless the `occt-brep` feature is built.
#[cfg(feature = "occt-brep")]
pub struct OcctBrepBridge;

#[cfg(feature = "occt-brep")]
impl om9_exchange::BrepBridge for OcctBrepBridge {
    fn import(
        &self,
        payload: &[u8],
        tolerance_mm: f64,
        deflection_mm: f64,
        limits: &om9_core::budget::Limits,
    ) -> om9_core::Result<om9_core::brep::Brep> {
        om9_kernel::brep::from_3dm(payload, tolerance_mm, deflection_mm, limits)
            .map_err(|error| om9_core::Error::Io(format!("3DM editable BRep: {error}")))
    }
}

#[cfg(feature = "occt-brep")]
impl om9_exchange::BrepExport for OcctBrepBridge {
    fn export(
        &self,
        brep: &om9_core::brep::Brep,
        limits: &om9_core::budget::Limits,
    ) -> om9_core::Result<om9_core::geometry::NativeBrep> {
        om9_kernel::brep::to_3dm(brep, 0.001, limits)
            .map_err(|error| om9_core::Error::Io(format!("3DM BRep export: {error}")))
    }
}
