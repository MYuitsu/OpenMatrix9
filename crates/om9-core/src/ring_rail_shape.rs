// SPDX-License-Identifier: LGPL-2.1-or-later
//! User-authored non-round finger rails. Shapes retain their canonical curve in
//! world millimetres around F4; a size changes only a uniform scale and placement.
use crate::{Error, Result, budget::Limits, curve_ops, geometry::{Curve, Point3, Transform}};
use serde::{Deserialize, Serialize};

pub const MAX_SHAPE_BYTES: usize = 16 * 1024;
pub const MAX_SHAPES: usize = 64;
pub const MAX_LIBRARY_BYTES: usize = MAX_SHAPES * (MAX_SHAPE_BYTES + 128);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RailShape {
    name: String,
    curve: Curve,
}

impl RailShape {
    pub fn new(name: &str, curve: Curve, limits: &Limits) -> Result<Self> {
        let shape = Self { name: name.trim().to_owned(), curve };
        shape.validate(limits)?;
        Ok(shape)
    }
    pub fn name(&self) -> &str { &self.name }
    pub fn curve(&self) -> &Curve { &self.curve }
    pub fn from_json(text: &str, limits: &Limits) -> Result<Self> {
        if text.len() > MAX_SHAPE_BYTES.min(limits.max_name_bytes) { return Err(Error::Budget("custom Ring Rail snapshot")); }
        let shape: Self = serde_json::from_str(text).map_err(|_| Error::Invalid("custom Ring Rail snapshot"))?;
        shape.validate(limits)?;
        Ok(shape)
    }
    pub fn to_json(&self) -> String { serde_json::to_string(self).expect("finite validated Ring Rail shape") }
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        if self.name.trim() != self.name || self.name.is_empty() || self.name.len() > 64
            || self.name.chars().any(char::is_control) || self.name.eq_ignore_ascii_case("Round") {
            return Err(Error::Invalid("custom Ring Rail name"));
        }
        let mut bounded = *limits;
        bounded.max_depth = bounded.max_depth.min(24);
        bounded.max_points = bounded.max_points.min(512);
        self.curve.validate(&bounded)?;
        front_plane(&self.curve)?;
        curve_ops::validate_ring_rail(&self.curve, 1e-6, &bounded)?;
        let perimeter = curve_ops::arc_length(&self.curve, &bounded)?;
        if !perimeter.is_finite() || perimeter <= 1e-6 { return Err(Error::Invalid("custom Ring Rail perimeter")); }
        if self.to_json().len() > MAX_SHAPE_BYTES.min(limits.max_name_bytes) { return Err(Error::Budget("custom Ring Rail snapshot")); }
        Ok(())
    }
    /// The user chose inner circumference on 2026-11-17. `diameter_mm` is the
    /// equivalent round size, so the requested inner perimeter is pi * diameter.
    pub fn sized_curve(&self, diameter_mm: f64, center: Point3, frame: &str,
        mm_per_unit: f64, limits: &Limits) -> Result<Curve> {
        self.validate(limits)?;
        let diameter_mm = crate::ring_size::rail_diameter_mm("diameter", diameter_mm)?;
        if !mm_per_unit.is_finite() || mm_per_unit <= 0. { return Err(Error::Invalid("Ring Rail model units")); }
        let scale = std::f64::consts::PI * diameter_mm / curve_ops::arc_length(&self.curve, limits)? / mm_per_unit;
        let matrix = match frame {
            "front" => [[scale, 0., 0., center[0]], [0., scale, 0., center[1]], [0., 0., scale, center[2]], [0., 0., 0., 1.]],
            "xy" => [[scale, 0., 0., center[0]], [0., 0., scale, center[1]], [0., -scale, 0., center[2]], [0., 0., 0., 1.]],
            _ => return Err(Error::Unsupported("Ring Rail frame")),
        };
        let curve = crate::top11::bake_curve(&self.curve, Transform(matrix))?;
        curve.validate(limits)?;
        Ok(curve)
    }
}

// Check source control data, not a display mesh or sampled plane fit. Rational
// NURBS with all control points in this plane remain in it under evaluation.
fn front_plane(curve: &Curve) -> Result<()> {
    let point = |p: Point3| if p[1].abs() <= 1e-6 { Ok(()) } else { Err(Error::Invalid("draw the custom Ring Rail in the Front plane around F4")) };
    match curve {
        Curve::Line { start, end, .. } => { point(*start)?; point(*end) }
        Curve::Circle { center, x, y, radius, .. } => {
            if center[1].abs() + radius * x[1].hypot(y[1]) > 1e-6 { return Err(Error::Invalid("custom Ring Rail Front plane")); }
            Ok(())
        }
        Curve::Nurbs(curve) => { for p in &curve.poles { point(*p)?; } Ok(()) }
        Curve::Polycurve { segments, .. } => { for part in segments { front_plane(part)?; } Ok(()) }
    }
}

/// Same control-point edit used by CAD grips, in the library's Front/mm frame.
/// Intermediate curves may fail rail validation; Save validates the full shape.
pub fn move_shape_grip(curve: &Curve, index: usize, point: Point3, limits: &Limits) -> Result<Curve> {
    if point[1] != 0. { return Err(Error::Invalid("custom Ring Rail Front plane")); }
    if matches!(curve, Curve::Polycurve { .. }) {
        crate::curve_ops::move_grip_with_joints(curve, index, point, Transform::identity(), 1e-6, limits)
    } else { crate::curve_ops::move_grip(curve, index, point, limits) }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RailShapeLibrary {
    shapes: Vec<RailShape>,
}
impl RailShapeLibrary {
    pub fn shapes(&self) -> &[RailShape] { &self.shapes }
    pub fn find(&self, name: &str) -> Option<&RailShape> { self.shapes.iter().find(|shape| shape.name.eq_ignore_ascii_case(name.trim())) }
    pub fn store(&mut self, shape: RailShape, limits: &Limits) -> Result<()> {
        shape.validate(limits)?;
        if let Some(existing) = self.shapes.iter_mut().find(|entry| entry.name.eq_ignore_ascii_case(&shape.name)) { *existing = shape; }
        else {
            if self.shapes.len() >= MAX_SHAPES { return Err(Error::Budget("custom Ring Rail library")); }
            self.shapes.push(shape);
        }
        self.shapes.sort_by_key(|shape| shape.name.to_lowercase());
        Ok(())
    }
    pub fn from_json(text: &str, limits: &Limits) -> Result<Self> {
        if text.len() > MAX_LIBRARY_BYTES { return Err(Error::Budget("custom Ring Rail library")); }
        let value: serde_json::Value = serde_json::from_str(text).map_err(|_| Error::Invalid("custom Ring Rail library"))?;
        if value.get("schema").and_then(|value| value.as_str()) != Some("om9.ring-rail-library.v1") { return Err(Error::Unsupported("custom Ring Rail library schema")); }
        let library: Self = serde_json::from_value(value.get("library").cloned().ok_or(Error::Missing("custom Ring Rail library"))?)
            .map_err(|_| Error::Invalid("custom Ring Rail library"))?;
        if library.shapes.len() > MAX_SHAPES { return Err(Error::Budget("custom Ring Rail library")); }
        let mut names = std::collections::BTreeSet::new();
        for shape in &library.shapes {
            shape.validate(limits)?;
            if !names.insert(shape.name.to_lowercase()) { return Err(Error::Invalid("duplicate custom Ring Rail name")); }
        }
        Ok(library)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&serde_json::json!({"schema": "om9.ring-rail-library.v1", "library": self}))
            .expect("validated Ring Rail library")
    }
}
