// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem mesh -> editable OCCT B-Rep.
//!
//! The gem asset stays authored as a typed spec and a procedural closed mesh (`om9-core::gem`).
//! This adapter converts the same boundary into one OCCT solid so kernel operations (boolean,
//! fillet, measurement) can act on a real B-Rep without a second geometry source.
use crate::KernelError;
use crate::brep::from_mesh;
use om9_core::brep::Brep;
use om9_core::budget::Limits;
use om9_core::gem::GemMesh;

/// Display deflection used for the converted gem B-Rep. It is an order of magnitude finer than the
/// authored facet resolution so the converted body does not look coarser than its mesh.
pub const GEM_BREP_DEFLECTION_MM: f64 = 0.01;

/// Convert a closed gem mesh into one editable B-Rep solid.
///
/// The conversion refuses an open mesh, a non-manifold mesh and a body without volume; it never
/// repairs the author's boundary into a different shape.
pub fn gem_brep(gem: &GemMesh, limits: &Limits) -> Result<Brep, KernelError> {
    if !gem.closed || gem.volume_mm3 <= 0.0 {
        return Err(KernelError::InvalidInput("gem mesh is not closed".into()));
    }
    // A gem is small and dense: the sewing tolerance must stay well below the smallest authored
    // feature (0.05 mm) and the deflection controls only display tessellation.
    from_mesh(&gem.mesh, GEM_BREP_DEFLECTION_MM, limits)
}
