//! Duplicate Border (`DupBorder`, OM9-CURVE-039): the four surface boundary isocurves.

use om9_core::geometry::{Curve, Geometry};
use om9_core::surface_kernel::SurfaceIso;

use super::{Controller, Error};

pub const DUP_BORDER: &str = "OM9-CURVE-039";

impl Controller {
    pub fn dispatch_dup_border(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("surface selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected surface"))?;
        let geometry = entity.geometry.clone();
        match geometry {
            Geometry::Surface(surface) => {
                let [[u0, u1], [v0, v1]] = surface.domain;
                for (iso, value) in [
                    (SurfaceIso::U, v0),
                    (SurfaceIso::U, v1),
                    (SurfaceIso::V, u0),
                    (SurfaceIso::V, u1),
                ] {
                    let curve = om9_core::surface_kernel::isocurve(
                        &surface,
                        iso,
                        value,
                        self.coordinator.limits(),
                    )?;
                    self.commit_geometry(
                        DUP_BORDER,
                        "DupBorder",
                        Geometry::Curve(Curve::Nurbs(curve)),
                    )?;
                }
            }
            Geometry::Brep(brep) => {
                let edges =
                    om9_core::curve_surface_ops::boundary_edges(&brep, self.coordinator.limits())?;
                if edges.is_empty() {
                    return Err(Error::Invalid("B-Rep has no boundary edges"));
                }
                for edge in edges {
                    let curve = om9_core::curve_surface_ops::duplicate_brep_edge(
                        &brep,
                        edge,
                        self.coordinator.limits(),
                    )?;
                    self.commit_geometry(DUP_BORDER, "DupBorder", Geometry::Curve(curve))?;
                }
            }
            _ => return Err(Error::Invalid("surface or B-Rep selection")),
        }
        Ok(())
    }
}
