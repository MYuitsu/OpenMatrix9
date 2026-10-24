//! Sketch (`Sketch`, OM9-CURVE-045): planar polyline through selected points.

use om9_core::geometry::Geometry;

use super::{Controller, Error};

pub const SKETCH: &str = "OM9-CURVE-045";

impl Controller {
    pub fn dispatch_sketch(&mut self) -> Result<(), Error> {
        let mut points = Vec::new();
        for id in self.selected_ordered() {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selection"))?;
            let Geometry::Point(point) = entity.geometry else {
                return Err(Error::Invalid("point selection"));
            };
            points.push(point);
        }
        if points.len() < 2 {
            return Err(Error::Invalid("at least two points"));
        }
        let curve = om9_core::curve_build::polyline(&points)?;
        self.commit_geometry(SKETCH, "Sketch", Geometry::Curve(curve))
    }
}
