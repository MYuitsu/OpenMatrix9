//! Revolve (`Revolve`, OM9-SURFACE-018): full revolution around the CPlane normal.

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const REVOLVE: &str = "OM9-SURFACE-018";

impl Controller {
    pub fn dispatch_revolve(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("profile selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected profile"))?;
        let Geometry::Curve(profile) = &entity.geometry else {
            return Err(Error::Invalid("profile curve selection"));
        };
        let plane = self.input_plane();
        let axis = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let rail = Curve::Circle {
            center: plane.origin,
            x: plane.x,
            y: plane.y,
            radius: 1.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        };
        let surface = om9_core::surface_ops::rail_revolve(
            profile,
            &rail,
            plane.origin,
            axis,
            [32, 64],
            [3, 3],
            self.coordinator.limits(),
        )?;
        self.commit_geometry(REVOLVE, "Revolve", Geometry::Surface(surface))
    }
}
