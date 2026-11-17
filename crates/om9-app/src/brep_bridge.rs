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

    fn import_admitted(&self, payload: &[u8], tolerance_mm: f64, deflection_mm: f64,
        limits: &om9_core::budget::Limits, admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>)
        -> om9_core::Result<om9_core::brep::Brep> {
        use om9_core::Error;
        let mut refusal = None;
        let source = {
            let mut guarded = |live, pending| {
                let result = admit(live, pending);
                if let Err(error) = &result { refusal = Some(error.clone()); }
                result
            };
            om9_kernel::brep::from_3dm_source_typed_admitted(payload, tolerance_mm, limits, &mut guarded)
        }.map_err(|error| refusal.take().unwrap_or(error))?;
        let source_bytes = source.capacity() as u64;
        let remaining = limits.remaining_after(om9_core::budget::Usage { bytes: source_bytes, ..Default::default() })?;
        let snapshot = {
            let mut guarded = |live: u64, pending| {
                let total = source_bytes.checked_add(live).ok_or(Error::Overflow("3DM BRep source"))?;
                let result = admit(total, pending);
                if let Err(error) = &result { refusal = Some(error.clone()); }
                result
            };
            om9_kernel::brep::capture_source_topology(&source, deflection_mm, 0.3, &remaining, &mut guarded)
        }.map_err(|error| refusal.take().unwrap_or_else(|| Error::Io(format!("3DM editable BRep: {error}"))))?;
        Ok(snapshot.into_parts().0)
    }
}

#[cfg(feature = "occt-brep")]
impl om9_exchange::BrepExport for OcctBrepBridge {
    fn export(
        &self,
        brep: &dyn om9_core::brep::BrepSourceAccess,
        tolerance_mm: f64,
        limits: &om9_core::budget::Limits,
    ) -> om9_core::Result<om9_core::geometry::NativeBrep> {
        om9_kernel::brep::to_3dm(brep, tolerance_mm, limits)
            .map_err(|error| om9_core::Error::Io(format!("3DM BRep export: {error}")))
    }

    fn export_admitted(&self, brep: &dyn om9_core::brep::BrepSourceAccess, tolerance_mm: f64,
        limits: &om9_core::budget::Limits, admit: &mut dyn FnMut(u64,u64)->om9_core::Result<()>)
        -> om9_core::Result<om9_core::geometry::NativeBrep> {
        use om9_core::{Error,geometry::Mesh};
        let mut refused = None;
        let (payload,crc) = {
            let mut guarded = |live,pending| {
                let result = admit(live,pending);
                if let Err(error) = &result { refused = Some(error.clone()); }
                result
            };
            om9_kernel::brep::source_to_3dm_payload_admitted(brep.source(),tolerance_mm,limits,&mut guarded)
        }.map_err(|error| refused.take().unwrap_or_else(||Error::Io(format!("3DM BRep export: {error}"))))?;
        admit(payload.capacity() as u64,512)?;
        Ok(om9_core::geometry::NativeBrep {display:std::sync::Arc::new(Mesh {vertices:vec![],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),payload:std::sync::Arc::new(payload),crc})
    }

    fn export_trimmed_surface(&self, entity:&om9_core::model::Entity, tolerance_mm:f64,
        limits:&om9_core::budget::Limits, admit:&mut dyn FnMut(u64,u64)->om9_core::Result<()>)
        -> om9_core::Result<om9_core::geometry::NativeBrep> {
        use om9_core::{Error,geometry::{Geometry,Mesh},model::TRIM_SURFACE_EXTENSION_SLOT,rhino_semantics::TrimmedSurface};
        let Geometry::Surface(surface)=&entity.geometry else {return Err(Error::Invalid("3DM trim surface"));};
        // The metadata orientation contract is not yet established for export.
        if surface.reversed {return Err(Error::Unsupported("3DM reversed trimmed surface"));}
        let extension=entity.extensions.get(TRIM_SURFACE_EXTENSION_SLOT).ok_or(Error::Missing("3DM trim metadata"))?;
        if extension.payload.len()>om9_core::model::trim_surface_payload_limit(limits) {return Err(Error::Unsupported("3DM trim metadata size"));}
        let scratch=u64::try_from(extension.payload.len()).map_err(|_|Error::Overflow("3DM trim scratch"))?
            .checked_mul(64).and_then(|n|n.checked_add(4096)).ok_or(Error::Overflow("3DM trim scratch"))?;
        admit(0,scratch)?;
        let trim=TrimmedSurface::from_extension(extension,limits)?;
        if trim.underlying!=*surface {return Err(Error::Invalid("3DM trim surface mismatch"));}
        admit(scratch,0)?;
        let mut refused=None;
        let source={let mut guarded=|live:u64,pending:u64| {
            let result=admit(scratch.checked_add(live).ok_or(Error::Overflow("3DM trim scratch"))?,pending);
            if let Err(e)=&result {refused=Some(e.clone());}result
        };
        om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,limits,&mut guarded)}
            .map_err(|e|refused.take().unwrap_or_else(||Error::Io(format!("3DM trimmed face: {e}"))))?;
        let held=scratch.checked_add(source.capacity() as u64).ok_or(Error::Overflow("3DM trim source"))?;
        let (payload,crc)={let mut guarded=|live:u64,pending:u64| {
            let result=admit(held.checked_add(live).ok_or(Error::Overflow("3DM trim source"))?,pending);
            if let Err(e)=&result {refused=Some(e.clone());}result
        };
        om9_kernel::brep::source_to_3dm_payload_admitted(&source,tolerance_mm,limits,&mut guarded)}
            .map_err(|e|refused.take().unwrap_or_else(||Error::Io(format!("3DM trimmed BRep: {e}"))))?;
        admit(held.checked_add(payload.capacity() as u64).ok_or(Error::Overflow("3DM trim payload"))?,512)?;
        Ok(om9_core::geometry::NativeBrep {display:std::sync::Arc::new(Mesh {vertices:vec![],faces:vec![],normals:vec![],colors:vec![],corner_uvs:vec![]}),
            payload:std::sync::Arc::new(payload),crc})
    }
}

#[cfg(all(test, feature = "occt-brep"))]
mod tests {
    use super::OcctBrepBridge;
    use om9_exchange::BrepExport;

    #[test]
    fn export_bridge_forwards_tolerance_to_kernel() {
        let limits = om9_core::budget::Limits::default();
        let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
        assert!(OcctBrepBridge.export(&sphere, 0.0, &limits).is_err());
        let result = OcctBrepBridge.export(&sphere, 0.02, &limits).unwrap();
        assert!(!result.payload.is_empty());
    }
}
