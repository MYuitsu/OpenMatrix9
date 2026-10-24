//! Curve on Mesh (`Mesh`, OM9-CURVE-049).

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const MESH_CURVE: &str = "OM9-CURVE-049";

impl Controller {
    pub fn dispatch_mesh_curve(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        let mut mesh = None;
        let mut points = Vec::new();
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            match &entity.geometry {
                Geometry::Mesh(value) => mesh = Some(value.clone()),
                Geometry::Point(point) => points.push(*point),
                _ => return Err(Error::Invalid("mesh and points selection")),
            }
        }
        let Some(mesh) = mesh else {
            return Err(Error::Invalid("mesh selection"));
        };
        if points.len() < 2 {
            return Err(Error::Invalid("at least two points"));
        }
        let curve = om9_core::curve_surface_ops::sketch_on_mesh(
            &mesh,
            &points,
            1e9,
            self.coordinator.limits(),
        )?;
        self.commit_geometry(MESH_CURVE, "Mesh", Geometry::Curve(curve))
    }
}
