//! Duplicate Edge / Extract Wire Frame (`DupEdge`, `ExtractWire`, OM9-CURVE-040/054).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const DUP_EDGE: &str = "OM9-CURVE-040";
pub const EXTRACT_WIRE: &str = "OM9-CURVE-054";

impl Controller {
    pub fn dispatch_wireframe(&mut self, feature: &'static str) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("brep selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected brep"))?;
        let Geometry::Brep(brep) = &entity.geometry else {
            return Err(Error::Invalid("brep selection"));
        };
        let display = brep.display.clone();
        let mut groups: std::collections::BTreeMap<u32, Vec<(usize, usize)>> =
            std::collections::BTreeMap::new();
        for segment in &display.segments {
            groups
                .entry(segment[2])
                .or_default()
                .push((segment[0] as usize, segment[1] as usize));
        }
        if groups.is_empty() {
            return Err(Error::Invalid("brep has no display edges"));
        }
        let label = if feature == DUP_EDGE { "DupEdge" } else { "ExtractWire" };
        let mut committed = false;
        for segments in groups.values() {
            let mut lines = Vec::with_capacity(segments.len());
            for (start, end) in segments {
                let (Some(start), Some(end)) =
                    (display.points.get(*start), display.points.get(*end))
                else {
                    continue;
                };
                if start == end {
                    continue;
                }
                lines.push(Curve::Line {
                    start: *start,
                    end: *end,
                    domain: [0.0, 1.0],
                });
            }
            if lines.is_empty() {
                continue;
            }
            self.commit_geometry(
                feature,
                label,
                Geometry::Curve(Curve::Polycurve {
                    segments: lines,
                    reversed: false,
                }),
            )?;
            committed = true;
        }
        if !committed {
            return Err(Error::Invalid("brep has no display edges"));
        }
        Ok(())
    }
}
