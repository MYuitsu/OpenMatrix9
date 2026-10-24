//! Interp Curve on Surface (`InterpCrvOnSrf`, OM9-CURVE-047).

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const INTERP_ON_SURFACE: &str = "OM9-CURVE-047";

impl Controller {
    pub fn dispatch_interp_on_surface(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        let mut surface = None;
        let mut points = Vec::new();
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            match &entity.geometry {
                Geometry::Surface(value) => surface = Some(value.clone()),
                Geometry::Point(point) => points.push(*point),
                _ => return Err(Error::Invalid("surface and points selection")),
            }
        }
        let Some(surface) = surface else {
            return Err(Error::Invalid("surface selection"));
        };
        if points.len() < 2 {
            return Err(Error::Invalid("at least two points"));
        }
        let mut parameters = Vec::with_capacity(points.len());
        for point in points {
            let projection = om9_core::curve_surface_ops::closest_point_on_surface(
                &surface,
                point,
                self.coordinator.limits(),
            )?;
            parameters.push(projection.parameters);
        }
        let curve = om9_core::curve_surface_ops::sketch_on_surface(
            &surface,
            &parameters,
            self.coordinator.limits(),
        )?;
        self.commit_geometry(
            INTERP_ON_SURFACE,
            "InterpCrvOnSrf",
            Geometry::Curve(curve),
        )
    }
}
