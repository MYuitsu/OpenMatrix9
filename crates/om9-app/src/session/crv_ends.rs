//! Mark curve start/end (`CrvStart`, `CrvEnd`, OM9-CURVE-033/032).
//!
//! The stable ids follow the authored FEATURES catalog: OM9-CURVE-033 is
//! `Mark Curve Start` and OM9-CURVE-032 is `Mark Curve End`, so the menu row
//! and the typed command cannot disagree about which end they mark.

use om9_core::geometry::{Geometry, Point3};

use super::{Controller, Error};

pub const CRV_START: &str = "OM9-CURVE-033";
pub const CRV_END: &str = "OM9-CURVE-032";

impl Controller {
    pub fn dispatch_crv_mark(&mut self, end: bool) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("curve selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("curve selection"));
        };
        let samples = om9_core::surface_kernel::sample_curve(curve, 2, self.coordinator.limits())?;
        let point: Point3 = if end {
            *samples.last().ok_or(Error::Invalid("curve samples"))?
        } else {
            samples[0]
        };
        let feature = if end { CRV_END } else { CRV_START };
        let label = if end { "CrvEnd" } else { "CrvStart" };
        self.commit_geometry(feature, label, Geometry::Point(point))
    }
}
