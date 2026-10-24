// SPDX-License-Identifier: LGPL-2.1-or-later
//! What the Object Info overlay is allowed to say about one object
//! (OM9-L3-03-14-005).
//!
//! Matrix8 book 1, pages 32-34: "Object Info displays a quick pop up label beside
//! a selected object … It displays the type of object and important information
//! about it: surface or polysurface, closed or open, valid or invalid, and point
//! count (for curves) or surface count (for polysurfaces)."
//!
//! The rule this module keeps is the plan's negative oracle: a geometry whose
//! measurement is not available reports **unavailable**, never a zero that would
//! read as a real measurement. Validity is reported the same way: `Some(true)` /
//! `Some(false)` only when the model can actually judge it, `None` otherwise.
use crate::{
    Error, Result,
    geometry::{Geometry, Point3},
    identity::ObjectId,
    model::Document,
};

/// A measurement the overlay may show.
#[derive(Debug, Clone, PartialEq)]
pub enum Measurement {
    /// Size along x, y and z in document units.
    Size([f64; 3]),
    /// The model cannot measure this geometry; the reason is shown as written.
    Unavailable(&'static str),
}

/// The fields the Object Info overlay shows for the selected object.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInfo {
    /// "Point", "Line", "Circle", "Curve", "Surface", "Mesh", "Point cloud",
    /// "B-rep", "Native B-rep", "Instance".
    pub kind: &'static str,
    pub dimensions: Measurement,
    /// `None` when validity cannot be judged here (for example a native B-rep
    /// whose kernel payload is not loaded): the overlay says so instead of
    /// claiming a zero or a pass.
    pub valid: Option<bool>,
    /// The human line under the type: counts, radius, length, closed/open.
    pub detail: String,
    /// The layer colour the label is drawn in, when the object has one.
    pub color: Option<[u8; 4]>,
}

fn span(min: f64, max: f64) -> f64 {
    max - min
}

fn size_of(points: &[Point3]) -> Option<[f64; 3]> {
    let first = points.first()?;
    let mut min = *first;
    let mut max = *first;
    for point in points {
        if !(0..3).all(|i| point[i].is_finite()) {
            return None;
        }
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    Some([
        span(min[0], max[0]),
        span(min[1], max[1]),
        span(min[2], max[2]),
    ])
}

/// The one line the in-viewport label reads: the type, the size (or the reason it
/// is unavailable) and the validity, in the manual's order.
pub fn label_text(info: &ObjectInfo) -> String {
    let size = match info.dimensions {
        Measurement::Size([x, y, z]) => format!("{x:.3} × {y:.3} × {z:.3}"),
        Measurement::Unavailable(reason) => format!("unavailable ({reason})"),
    };
    let validity = match info.valid {
        Some(true) => "valid",
        Some(false) => "invalid",
        None => "validity unavailable",
    };
    format!("{} · {size} · {validity}", info.kind)
}

impl Document {
    /// The Object Info of one object, sourced from the model only.
    pub fn object_info(&self, id: ObjectId, limits: &crate::budget::Limits) -> Result<ObjectInfo> {
        let entity = self
            .entities
            .get(&id)
            .ok_or(Error::Missing("object info"))?;
        let color = entity
            .attributes
            .color
            .or_else(|| self.display_attributes(id).ok().and_then(|d| d.color));
        let valid = entity.geometry.validate(limits).ok().map(|_| true);
        let (kind, dimensions, detail) = match &entity.geometry {
            Geometry::Point(point) => (
                "Point",
                Measurement::Size([0., 0., 0.]),
                format!("at {:.3}, {:.3}, {:.3}", point[0], point[1], point[2]),
            ),
            Geometry::Curve(curve) => match curve {
                crate::geometry::Curve::Line { start, end, .. } => {
                    let size = size_of(&[*start, *end]).unwrap_or([0., 0., 0.]);
                    let length = (end[0] - start[0])
                        .hypot(end[1] - start[1])
                        .hypot(end[2] - start[2]);
                    (
                        "Line",
                        Measurement::Size(size),
                        format!("length {length:.3}"),
                    )
                }
                crate::geometry::Curve::Circle { radius, .. } if radius.is_finite() => (
                    "Circle",
                    Measurement::Size([2. * radius, 2. * radius, 0.]),
                    format!("radius {radius:.3}"),
                ),
                _ => (
                    "Curve",
                    Measurement::Unavailable("this curve has no measured size yet"),
                    "curve".to_owned(),
                ),
            },
            Geometry::Surface(_) => (
                "Surface",
                Measurement::Unavailable("surface bounds are not computed yet"),
                "surface".to_owned(),
            ),
            Geometry::Brep(brep) => (
                "B-rep",
                Measurement::Unavailable("B-rep bounds come from the kernel and are not loaded"),
                brep.source().to_owned(),
            ),
            Geometry::NativeBrep(_) => (
                "Native B-rep",
                Measurement::Unavailable("the native kernel payload is not loaded"),
                "native kernel shape".to_owned(),
            ),
            Geometry::Mesh(_) => (
                "Mesh",
                Measurement::Unavailable("mesh bounds are not computed yet"),
                "mesh".to_owned(),
            ),
            Geometry::Cloud(_) => (
                "Point cloud",
                Measurement::Unavailable("cloud bounds are not computed yet"),
                "point cloud".to_owned(),
            ),
            Geometry::Instance(_) => (
                "Instance",
                Measurement::Unavailable("an instance is measured through its definition"),
                "instance".to_owned(),
            ),
            Geometry::Light(_) => (
                "Light",
                Measurement::Unavailable("light bounds are not computed yet"),
                "light".to_owned(),
            ),
            Geometry::Hatch(hatch) => (
                "Hatch",
                Measurement::Unavailable("hatch bounds are not computed yet"),
                hatch.pattern.as_deref().map_or_else(
                    || "hatch".to_owned(),
                    |pattern| format!("pattern {pattern}"),
                ),
            ),
        };
        Ok(ObjectInfo {
            kind,
            dimensions,
            valid,
            detail,
            color,
        })
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;

    #[test]
    fn the_label_reads_type_size_and_validity() {
        let known = ObjectInfo {
            kind: "Line",
            dimensions: Measurement::Size([10., 4., 0.]),
            valid: Some(true),
            detail: "length 10.774".into(),
            color: None,
        };
        assert_eq!(label_text(&known), "Line · 10.000 × 4.000 × 0.000 · valid");
        let unknown = ObjectInfo {
            kind: "Mesh",
            dimensions: Measurement::Unavailable("no mesh bounds are computed yet"),
            valid: Some(false),
            detail: "mesh".into(),
            color: None,
        };
        assert_eq!(
            label_text(&unknown),
            "Mesh · unavailable (no mesh bounds are computed yet) · invalid"
        );
        let unjudged = ObjectInfo {
            kind: "B-rep",
            dimensions: Measurement::Unavailable(
                "B-rep bounds come from the kernel and are not loaded",
            ),
            valid: None,
            detail: "source".into(),
            color: None,
        };
        assert!(label_text(&unjudged).ends_with("validity unavailable"));
    }
}
