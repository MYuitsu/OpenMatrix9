// SPDX-License-Identifier: LGPL-2.1-or-later
//! Jewelry ring-size standards for the ring-rail builder (`OM9-L3-04-02-001`).
//!
//! The builder accepts an explicit inside diameter, which is the authoritative value, or a
//! named standard size. A standard is only accepted when this module carries a frozen table
//! and provenance for it; an unverified locale is refused instead of guessed.
//!
//! The US table is derived from the published US size increment (0.8128 mm per size) and
//! its 11.63 mm base, rounded to 0.01 mm, and cross-checked against the ISO 8653 nominal
//! circumference series. The values are frozen in [`US_TABLE`] so a library upgrade cannot
//! silently change a saved ring.
use crate::{Error, Result};

/// The provenance string recorded with a standard-size ring rail. It names the standard,
/// the conversion rule and the rounding, so a reviewer can reproduce the table.
pub const US_PROVENANCE: &str =
    "US ring size: inside diameter mm = 11.63 + 0.8128 * size, rounded to 0.01 mm; \
     cross-checked against ISO 8653 nominal circumference series; table frozen in om9-core";

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
        assert!(RingSizeStandard::Us.provenance().contains("ISO 8653"));
        assert!(RingSizeStandard::Us.provenance().contains("0.8128"));
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
