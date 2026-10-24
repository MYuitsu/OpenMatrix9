//! Create UV Curves (`CreateUVCrv`, OM9-CURVE-025): planar UV rectangle.

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const CREATE_UV: &str = "OM9-CURVE-025";

impl Controller {
    pub fn dispatch_create_uv(&mut self) -> Result<(), Error> {
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
        let Geometry::Surface(surface) = &entity.geometry else {
            return Err(Error::Invalid("surface selection"));
        };
        let [[u0, u1], [v0, v1]] = surface.domain;
        let width = u1 - u0;
        let height = v1 - v0;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Err(Error::Invalid("surface UV domain"));
        }
        let curve = om9_core::curve_build::rectangle(
            [u0, v0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            width,
            height,
        )?;
        self.commit_geometry(CREATE_UV, "CreateUVCrv", Geometry::Curve(curve))
    }
}
