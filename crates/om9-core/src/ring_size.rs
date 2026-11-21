// SPDX-License-Identifier: LGPL-2.1-or-later
//! Jewelry ring-size standards for the ring-rail builder (`OM9-L3-04-02-001`).
//!
//! The builder accepts an explicit inside diameter, which is the authoritative value, or a
//! named standard size. A standard is only accepted when this module carries a frozen table
//! and provenance for it; an unverified locale is refused instead of guessed.
//!
//! New Ring Rail input uses Stuller's published inside-diameter table (half sizes)
//! and explicitly interpolates quarter sizes. The older integer API remains frozen.
use crate::{Error, Result};

/// Existing direct-builder physical diameter domain, shared by both adapters.
pub const MIN_RAIL_DIAMETER_MM: f64 = 1.;
pub const MAX_RAIL_DIAMETER_MM: f64 = 40.;

pub fn rail_system_label(id: &str) -> &str {
    match id {
        "VNM" | "vnm" => "VNM (Việt Nam) — inside diameter (mm)",
        "vnm_circumference" => "VNM (Việt Nam) — inside circumference (mm)",
        "US" | "us" => "US / Canada (Stuller)",
        "explicit" | "diameter" => "Inside diameter (mm)",
        "britain" => "Britain",
        "japan" => "Japan",
        "swiss" => "Swiss",
        "european" => "European",
        "france" => "France",
        "german" => "German",
        "iso" => "Inside circumference (mm)",
        "diameter_inches" => "Inside diameter (inch)",
        "circumference_inches" => "Inside circumference (inch)",
        other => other,
    }
}

/// The named sizes shown by the Matrix region picker, expressed in quarter units.
pub fn rail_size_steps(system: &str) -> Option<(i32, i32, usize)> {
    Some(match system {
        "US" | "us" => (4, 60, 1),
        "britain" => (4, 134, 2),
        "japan" => (0, 144, 4),
        "swiss" => (4, 163, 1),
        "european" => (164, 320, 4),
        "france" => (164, 323, 1),
        "german" => (53, 90, 1),
        _ => return None,
    })
}

fn regional_diameter(system: &str, size: f64) -> f64 {
    match system {
        "britain" => 11.5 + 0.4 * size,
        "japan" => 12.67 + size / 3.,
        "swiss" => (40. + size) / std::f64::consts::PI,
        "european" | "france" => size / std::f64::consts::PI,
        "german" => size,
        _ => unreachable!("named region"),
    }
}

/// Shared Ring Rail input conversion. US quarter sizes interpolate the frozen
/// Stuller half-size table; custom dimensions are converted to world millimetres.
pub fn rail_diameter_mm(system: &str, size: f64) -> Result<f64> {
    if !size.is_finite() || size < 0.0 { return Err(Error::Invalid("ring size")); }
    let diameter = match system.trim().to_ascii_lowercase().as_str() {
        "us" | "usa" => {
            if !(1.0..=15.0).contains(&size) || (size * 4.0 - (size * 4.0).round()).abs() > 1e-8 {
                return Err(Error::Invalid("US ring size 1..15 in quarter steps"));
            }
            let index = ((size - 1.) * 2.).floor() as usize;
            let a = (US_HALF_DIAMETERS_MM[index] * 100.).round();
            let b = (US_HALF_DIAMETERS_MM[(index + 1).min(28)] * 100.).round();
            (a + (b - a) * ((size - 1.) * 2.).fract()).round() / 100.0
        }
        region if let Some((lo, hi, step)) = rail_size_steps(region) => {
            let quarter = size * 4.;
            if quarter < lo as f64 || quarter > hi as f64
                || (quarter - quarter.round()).abs() > 1e-8 || (quarter.round() as i32 - lo) % step as i32 != 0 {
                return Err(Error::Invalid("regional ring size"));
            }
            regional_diameter(region, size)
        }
        "diameter" | "explicit" | "vnm" => size,
        "iso" | "circumference" | "vnm_circumference" => size / std::f64::consts::PI,
        "diameter_inches" => size * 25.4,
        "circumference_inches" => size * 25.4 / std::f64::consts::PI,
        _ => return Err(Error::Unsupported("ring size standard")),
    };
    if !diameter.is_finite() || diameter <= 0.0 { return Err(Error::Invalid("ring rail diameter")); }
    if !(MIN_RAIL_DIAMETER_MM..=MAX_RAIL_DIAMETER_MM).contains(&diameter) { return Err(Error::Invalid("ring rail diameter 1..40 mm")); }
    Ok(diameter)
}

/// Preserve the physical diameter when changing the input system. US is snapped
/// to the nearest supported quarter size; VNM is an explicit mm dimension.
pub fn rail_size_for_diameter(system: &str, diameter_mm: f64) -> Result<f64> {
    rail_diameter_mm("diameter", diameter_mm)?;
    match system.trim().to_ascii_lowercase().as_str() {
        "us" | "usa" => {
            if !(US_TABLE[0].1..=US_TABLE[14].1).contains(&diameter_mm) { return Err(Error::Invalid("diameter outside US ring table")); }
            (4..=60).map(|quarter| quarter as f64 / 4.).min_by(|a, b|
                (rail_diameter_mm("us", *a).unwrap() - diameter_mm).abs().total_cmp(&(rail_diameter_mm("us", *b).unwrap() - diameter_mm).abs()))
                .ok_or(Error::Invalid("US ring size"))
        }
        region if let Some((lo, hi, step)) = rail_size_steps(region) => {
            (lo..=hi).step_by(step).map(|q| q as f64 / 4.).min_by(|a, b|
                (regional_diameter(region, *a) - diameter_mm).abs()
                    .total_cmp(&(regional_diameter(region, *b) - diameter_mm).abs()))
                .ok_or(Error::Invalid("regional ring size"))
        }
        "diameter" | "explicit" | "vnm" => Ok(diameter_mm),
        "iso" | "circumference" | "vnm_circumference" => Ok(diameter_mm * std::f64::consts::PI),
        "diameter_inches" => Ok(diameter_mm / 25.4),
        "circumference_inches" => Ok(diameter_mm * std::f64::consts::PI / 25.4),
        _ => Err(Error::Unsupported("ring size standard")),
    }
}

/// Canonical Front-view rail: starts at the top and turns toward +X.
pub fn front_rail(center: crate::geometry::Point3, diameter_mm: f64) -> Result<crate::geometry::Curve> {
    rail_curve(center, diameter_mm, "front")
}

/// Construct the common circle, including the frame of older saved recipes.
pub fn rail_curve(center: crate::geometry::Point3, diameter_mm: f64, frame: &str) -> Result<crate::geometry::Curve> {
    if !diameter_mm.is_finite() || diameter_mm <= 0. { return Err(Error::Invalid("ring rail diameter")); }
    if center.iter().any(|coordinate| !coordinate.is_finite()) { return Err(Error::Invalid("ring rail center")); }
    let (x, y) = match frame {
        "front" => ([0., 0., 1.], [1., 0., 0.]),
        "xy" => ([1., 0., 0.], [0., 1., 0.]),
        _ => return Err(Error::Unsupported("ring rail frame")),
    };
    Ok(crate::geometry::Curve::Circle { center, x, y,
        radius: diameter_mm / 2.0, domain: [0., std::f64::consts::TAU], angle_domain: None })
}

/// The provenance string recorded with a standard-size ring rail. It names the standard,
/// the conversion rule and the rounding, so a reviewer can reproduce the table.
pub const US_PROVENANCE: &str =
    "Stuller Ring Blank Lengths Chart (2013-06-05), inside diameter mm, half sizes 1..15; quarter sizes interpolated and rounded to 0.01 mm; https://www.stuller.com/articles/view/determining-ring-blank-lengths-in-b-and-s-gauge/";

/// Factual diameter data from the cited supplier chart; no artwork or catalog assets.
pub const US_HALF_DIAMETERS_MM: [f64; 29] = [
    12.44,12.85,13.26,13.66,14.07,14.47,14.88,15.29,15.69,16.10,
    16.51,16.91,17.32,17.73,18.13,18.54,18.95,19.35,19.76,20.16,
    20.57,20.98,21.38,21.79,22.20,22.60,23.01,23.42,23.82,
];

/// The frozen US size table, sizes 1..=15. Size 0 and negative sizes are not jewelry
/// standards handled by this builder.
pub const US_TABLE: [(u8, f64); 15] = [
    (1, 12.44),
    (2, 13.26),
    (3, 14.07),
    (4, 14.88),
    (5, 15.69),
    (6, 16.51),
    (7, 17.32),
    (8, 18.13),
    (9, 18.95),
    (10, 19.76),
    (11, 20.57),
    (12, 21.38),
    (13, 22.20),
    (14, 23.01),
    (15, 23.82),
];

/// One verified ring-size standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RingSizeStandard {
    /// United States ring sizes 1..=15.
    Us,
}

impl RingSizeStandard {
    pub const ALL: [Self; 1] = [Self::Us];

    /// Stable identifier stored in a builder field and a recipe.
    pub fn id(self) -> &'static str {
        match self {
            Self::Us => "US",
        }
    }
    /// The label the panel paints.
    pub fn label(self) -> &'static str {
        match self {
            Self::Us => "US size",
        }
    }
    /// The frozen provenance recorded with a generated rail.
    pub fn provenance(self) -> &'static str {
        match self {
            Self::Us => US_PROVENANCE,
        }
    }
    /// Parse a standard id. The empty string and `explicit` mean "no standard"; an unknown
    /// locale such as `UK` is refused by the caller as `Unsupported`.
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_uppercase().as_str() {
            "US" => Some(Self::Us),
            _ => None,
        }
    }
    /// The inside diameter of a standard size.
    pub fn diameter_mm(self, size: u8) -> Result<f64> {
        match self {
            Self::Us => US_TABLE
                .iter()
                .find(|(candidate, _)| *candidate == size)
                .map(|(_, diameter)| *diameter)
                .ok_or(Error::Invalid("US ring size 1..15")),
        }
    }
    /// The nearest standard size for an inside diameter, used for a report only. A diameter
    /// outside the table returns `None` rather than clamping to an edge.
    pub fn nearest_size(self, diameter_mm: f64) -> Option<u8> {
        if !diameter_mm.is_finite() || diameter_mm <= 0.0 {
            return None;
        }
        let mut best: Option<(u8, f64)> = None;
        for (size, diameter) in self.table() {
            let distance = (diameter - diameter_mm).abs();
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((*size, distance));
            }
        }
        best.filter(|(_, distance)| *distance <= 0.25)
            .map(|(size, _)| size)
    }
    /// The frozen table.
    pub fn table(self) -> &'static [(u8, f64)] {
        match self {
            Self::Us => &US_TABLE,
        }
    }
}

/// Resolve a requested standard. An empty id or `explicit` returns `None` (the explicit
/// diameter wins); a supported standard returns its enum; any other locale is refused with
/// the name in the error.
pub fn standard_from_id(id: &str) -> Result<Option<RingSizeStandard>> {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("explicit") {
        return Ok(None);
    }
    RingSizeStandard::from_id(trimmed)
        .map(Some)
        .ok_or(Error::Unsupported("ring size standard"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_sizes_have_a_frozen_diameter_table_and_provenance() {
        assert_eq!(RingSizeStandard::Us.diameter_mm(8).unwrap(), 18.13);
        assert_eq!(RingSizeStandard::Us.diameter_mm(1).unwrap(), 12.44);
        assert_eq!(RingSizeStandard::Us.diameter_mm(15).unwrap(), 23.82);
        assert!(RingSizeStandard::Us.provenance().contains("Stuller"));
        assert!(RingSizeStandard::Us.provenance().contains("0.01 mm"));
        assert_eq!(RingSizeStandard::Us.table().len(), 15);
    }

    #[test]
    fn an_unverified_standard_and_an_out_of_table_size_are_refused() {
        assert_eq!(
            standard_from_id("UK"),
            Err(Error::Unsupported("ring size standard"))
        );
        assert_eq!(
            RingSizeStandard::Us.diameter_mm(0),
            Err(Error::Invalid("US ring size 1..15"))
        );
        assert_eq!(
            RingSizeStandard::Us.diameter_mm(16),
            Err(Error::Invalid("US ring size 1..15"))
        );
    }

    #[test]
    fn explicit_and_standard_ids_are_distinct() {
        assert_eq!(standard_from_id("explicit").unwrap(), None);
        assert_eq!(standard_from_id("").unwrap(), None);
        assert_eq!(
            standard_from_id("us").unwrap(),
            Some(RingSizeStandard::Us)
        );
        assert_eq!(RingSizeStandard::Us.nearest_size(18.1), Some(8));
        assert_eq!(RingSizeStandard::Us.nearest_size(12.0), None);
    }
}
