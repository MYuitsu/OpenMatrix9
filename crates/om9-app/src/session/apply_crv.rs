//! Apply UV Curves (`ApplyCrv`, OM9-CURVE-024).

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const APPLY_CRV: &str = "OM9-CURVE-024";

impl Controller {
    pub fn dispatch_apply_crv(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        let mut surface = None;
        let mut curves = Vec::new();
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            match &entity.geometry {
                Geometry::Surface(value) => surface = Some(value.clone()),
                Geometry::Curve(value) => curves.push(value.clone()),
                _ => return Err(Error::Invalid("surface and curves selection")),
            }
        }
        let Some(surface) = surface else {
            return Err(Error::Invalid("surface selection"));
        };
        if curves.is_empty() {
            return Err(Error::Invalid("curve selection"));
        }
        for curve in curves {
            let samples =
                om9_core::surface_kernel::sample_curve(&curve, 32, self.coordinator.limits())?;
            let mut parameters = Vec::with_capacity(samples.len());
            for point in samples {
                parameters.push(
                    om9_core::curve_surface_ops::closest_point_on_surface(
                        &surface,
                        point,
                        self.coordinator.limits(),
                    )?
                    .parameters,
                );
            }
            let applied = om9_core::curve_surface_ops::sketch_on_surface(
                &surface,
                &parameters,
                self.coordinator.limits(),
            )?;
            self.commit_geometry(APPLY_CRV, "ApplyCrv", Geometry::Curve(applied))?;
        }
        Ok(())
    }
}
