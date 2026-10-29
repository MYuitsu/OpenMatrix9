// SPDX-License-Identifier: LGPL-2.1-or-later
//! Jewelry metal volume, density and weight estimates (`OM9-L2-04-19`).
//!
//! The owner contract is deliberately narrow: the kernel measures a closed B-Rep, then this
//! module owns the unit, density and mass arithmetic. Every density is stored in
//! `g/cm^3`; a caller that has `kg/m^3` must say so through [`DensityUnit`], so a value can
//! never be silently read as the wrong unit. A missing material, an open shell, a zero-sized
//! or non-finite volume and a duplicate body entry are typed refusals rather than a plausible
//! but false mass.
use crate::{
    Error, Result,
    geometry::Unit,
};
use std::collections::{BTreeMap, BTreeSet};

/// One cubic centimetre in cubic millimetres.
pub const MM3_PER_CM3: f64 = 1000.0;

/// Grams in one pennyweight (dwt), the jewellery trade unit Matrix reports beside grams.
pub const GRAMS_PER_DWT: f64 = 1.555_173_84;

/// The built-in reference material used when a caller does not name one.
pub const DEFAULT_MATERIAL_ID: &str = "gold_18k_yellow";

/// A density may be stated in either unit the trade uses. The enum, not a bare `f64`, is
/// the guard against the `kg/m^3` value that would otherwise become a 1000x false weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityUnit {
    GramsPerCubicCentimeter,
    KilogramsPerCubicMeter,
}

impl DensityUnit {
    /// Convert `value` to canonical `g/cm^3`.
    pub fn to_grams_per_cubic_centimeter(self, value: f64) -> Result<f64> {
        if !value.is_finite() {
            return Err(Error::Invalid("metal material density"));
        }
        let density = match self {
            Self::GramsPerCubicCentimeter => value,
            Self::KilogramsPerCubicMeter => value / 1000.0,
        };
        if !density.is_finite() || density <= 0.0 {
            return Err(Error::Invalid("metal material density"));
        }
        Ok(density)
    }
}

/// A named metal/alloy density. `provenance` records where the number came from and
/// `test_only` marks a fixture density that must not be presented as a supplier value.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalMaterial {
    pub id: String,
    pub name: String,
    pub density_g_cm3: f64,
    pub provenance: String,
    pub test_only: bool,
    /// Relative density uncertainty in percent (0 for a synthetic fixture).
    pub uncertainty_percent: f64,
}

impl MetalMaterial {
    /// Build and validate a material. The density is converted from its declared unit first.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        density: f64,
        unit: DensityUnit,
        provenance: impl Into<String>,
    ) -> Result<Self> {
        let material = Self {
            id: id.into(),
            name: name.into(),
            density_g_cm3: unit.to_grams_per_cubic_centimeter(density)?,
            provenance: provenance.into(),
            test_only: false,
            uncertainty_percent: 0.0,
        };
        material.validate()?;
        Ok(material)
    }

    /// A fixture-only material. It is still validated and must carry a test-only marker.
    pub fn test_only(
        id: impl Into<String>,
        density_g_cm3: f64,
        provenance: impl Into<String>,
    ) -> Result<Self> {
        let mut material = Self::new(
            id,
            "Test-only metal",
            density_g_cm3,
            DensityUnit::GramsPerCubicCentimeter,
            provenance,
        )?;
        material.test_only = true;
        material.uncertainty_percent = 0.0;
        Ok(material)
    }

    pub fn with_uncertainty_percent(mut self, percent: f64) -> Result<Self> {
        if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
            return Err(Error::Invalid("metal material uncertainty"));
        }
        self.uncertainty_percent = percent;
        Ok(self)
    }

    pub fn density_kg_m3(&self) -> f64 {
        self.density_g_cm3 * 1000.0
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::Missing("metal material id"));
        }
        if self.name.trim().is_empty() {
            return Err(Error::Missing("metal material name"));
        }
        if self.provenance.trim().is_empty() {
            return Err(Error::Missing("metal material provenance"));
        }
        // 1000 g/cm^3 is far above any jewellery metal. It is the guard that catches a
        // kg/m^3 value entered as g/cm^3 (e.g. 15500) instead of accepting it.
        if !self.density_g_cm3.is_finite()
            || self.density_g_cm3 <= 0.0
            || self.density_g_cm3 > 1000.0
        {
            return Err(Error::Invalid("metal material density"));
        }
        if !self.uncertainty_percent.is_finite()
            || !(0.0..=100.0).contains(&self.uncertainty_percent)
        {
            return Err(Error::Invalid("metal material uncertainty"));
        }
        Ok(())
    }
}

/// A registry of the reference alloys plus any user/test materials added by the host.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetalMaterialRegistry {
    materials: BTreeMap<String, MetalMaterial>,
}

impl MetalMaterialRegistry {
    pub fn empty() -> Self {
        Self::default()
    }

    /// The small built-in reference set. Densities are nominal alloy values with their
    /// provenance recorded; a supplier lot must still be verified before a quote.
    pub fn builtin() -> Self {
        let mut registry = Self::empty();
        let entries = [
            (
                "gold_18k_yellow",
                "18k yellow gold",
                15.5,
                "OM9 reference nominal 18k yellow gold density (g/cm^3); alloy lot varies",
            ),
            (
                "gold_14k_yellow",
                "14k yellow gold",
                13.5,
                "OM9 reference nominal 14k yellow gold density (g/cm^3); alloy lot varies",
            ),
            (
                "gold_18k_white",
                "18k white gold",
                15.8,
                "OM9 reference nominal 18k white gold density (g/cm^3); alloy lot varies",
            ),
            (
                "platinum_950",
                "Platinum 950",
                20.8,
                "OM9 reference nominal Pt950 density (g/cm^3); alloy lot varies",
            ),
            (
                "palladium_950",
                "Palladium 950",
                12.0,
                "OM9 reference nominal Pd950 density (g/cm^3); alloy lot varies",
            ),
            (
                "sterling_silver",
                "Sterling silver",
                10.36,
                "OM9 reference nominal sterling silver density (g/cm^3); alloy lot varies",
            ),
        ];
        for (id, name, density, provenance) in entries {
            let material = MetalMaterial::new(
                id,
                name,
                density,
                DensityUnit::GramsPerCubicCentimeter,
                provenance,
            )
            .and_then(|material| material.with_uncertainty_percent(1.0))
            .expect("built-in materials are valid constants");
            registry
                .insert(material)
                .expect("built-in material insertion");
        }
        registry
    }

    /// Insert or replace a material under a case-insensitive id.
    pub fn insert(&mut self, material: MetalMaterial) -> Result<()> {
        material.validate()?;
        self.materials.insert(normalized_id(&material.id), material);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&MetalMaterial> {
        if id.trim().is_empty() {
            return Err(Error::Missing("metal material id"));
        }
        self.materials
            .get(&normalized_id(id))
            .ok_or(Error::Missing("metal material density"))
    }

    pub fn contains(&self, id: &str) -> bool {
        !id.trim().is_empty() && self.materials.contains_key(&normalized_id(id))
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &MetalMaterial> {
        self.materials.values()
    }
}

fn normalized_id(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

/// One kernel measurement in project units. A caller converting a B-Rep is expected to pass
/// `Unit::Millimeter`; a document in another unit must convert before it calls this function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidMeasure {
    pub volume: f64,
    pub unit: Unit,
    pub closed: bool,
    pub solids: usize,
}

impl SolidMeasure {
    /// A closed B-Rep whose kernel measurement is already in mm^3.
    pub fn closed_mm3(volume_mm3: f64, solids: usize) -> Self {
        Self {
            volume: volume_mm3,
            unit: Unit::Millimeter,
            closed: true,
            solids,
        }
    }
}

/// Convert a volume expressed in `unit^3` to canonical mm^3.
pub fn volume_mm3_from_project(volume: f64, unit: Unit) -> Result<f64> {
    if !volume.is_finite() {
        return Err(Error::Invalid("metal volume"));
    }
    let factor = unit.factor();
    let volume_mm3 = volume * factor * factor * factor;
    if !volume_mm3.is_finite() {
        return Err(Error::Overflow("metal volume"));
    }
    Ok(volume_mm3)
}

/// The L3-001/L3-005 admission rule: only a finite, positive volume from at least one
/// closed solid is admissible. An open shell never gets a number.
pub fn closed_solid_volume_mm3(measure: SolidMeasure) -> Result<f64> {
    if !measure.closed || measure.solids == 0 {
        return Err(Error::Invalid("metal weight requires a closed solid"));
    }
    if !measure.volume.is_finite() || measure.volume <= 0.0 {
        return Err(Error::Invalid("metal volume"));
    }
    let volume = volume_mm3_from_project(measure.volume, measure.unit)?;
    if !volume.is_finite() || volume <= 0.0 {
        return Err(Error::Invalid("metal volume"));
    }
    Ok(volume)
}

/// Convert grams to pennyweights.
pub fn grams_to_dwt(grams: f64) -> Result<f64> {
    if !grams.is_finite() || grams < 0.0 {
        return Err(Error::Invalid("metal mass"));
    }
    Ok(grams / GRAMS_PER_DWT)
}

/// Estimated mass for one material and one volume.
#[derive(Debug, Clone, PartialEq)]
pub struct MetalEstimate {
    pub volume_mm3: f64,
    pub material_id: String,
    pub density_g_cm3: f64,
    pub grams: f64,
    pub dwt: f64,
    /// Absolute density uncertainty translated to grams; 0 for a synthetic fixture.
    pub uncertainty_grams: f64,
}

impl MetalEstimate {
    pub fn new(volume_mm3: f64, material: &MetalMaterial) -> Result<Self> {
        if !volume_mm3.is_finite() || volume_mm3 <= 0.0 {
            return Err(Error::Invalid("metal volume"));
        }
        material.validate()?;
        let grams = volume_mm3 / MM3_PER_CM3 * material.density_g_cm3;
        if !grams.is_finite() || grams <= 0.0 {
            return Err(Error::Overflow("metal mass"));
        }
        let uncertainty_grams = grams * material.uncertainty_percent / 100.0;
        if !uncertainty_grams.is_finite() {
            return Err(Error::Overflow("metal mass uncertainty"));
        }
        Ok(Self {
            volume_mm3,
            material_id: material.id.clone(),
            density_g_cm3: material.density_g_cm3,
            grams,
            dwt: grams_to_dwt(grams)?,
            uncertainty_grams,
        })
    }
}

/// One selected body in a multi-material report. `instances` is the explicit count for a
/// body used more than once; a duplicate `key` is always refused because the caller must
/// express the count on one row rather than hide it as two rows.
#[derive(Debug, Clone, PartialEq)]
pub struct MassPart {
    pub key: String,
    pub volume_mm3: f64,
    pub material_id: String,
    pub instances: u32,
}

impl MassPart {
    pub fn new(
        key: impl Into<String>,
        volume_mm3: f64,
        material_id: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            volume_mm3,
            material_id: material_id.into(),
            instances: 1,
        }
    }

    pub fn with_instances(mut self, instances: u32) -> Self {
        self.instances = instances;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MassEntry {
    pub key: String,
    pub material_id: String,
    pub density_g_cm3: f64,
    pub volume_mm3: f64,
    pub instances: u32,
    pub grams: f64,
    pub dwt: f64,
}

/// Per-body and total mass for a selection. It is a read-only report; it never mutates the
/// document, so a failed run leaves the previous report and the document untouched.
#[derive(Debug, Clone, PartialEq)]
pub struct MassBreakdown {
    pub entries: Vec<MassEntry>,
    pub total_grams: f64,
    pub total_dwt: f64,
    pub total_volume_mm3: f64,
    pub total_uncertainty_grams: f64,
}

impl MassBreakdown {
    pub fn compute(parts: &[MassPart], registry: &MetalMaterialRegistry) -> Result<Self> {
        if parts.is_empty() {
            return Err(Error::Missing("metal weight parts"));
        }
        let mut keys = BTreeSet::new();
        let mut entries = Vec::with_capacity(parts.len());
        let mut total_grams = 0.0;
        let mut total_volume_mm3 = 0.0;
        let mut total_uncertainty_grams = 0.0;
        for part in parts {
            let key = part.key.trim();
            if key.is_empty() {
                return Err(Error::Missing("metal body id"));
            }
            if !keys.insert(key.to_ascii_lowercase()) {
                return Err(Error::Invalid("duplicate metal body"));
            }
            if part.instances == 0 {
                return Err(Error::Invalid("metal body instance count"));
            }
            if !part.volume_mm3.is_finite() || part.volume_mm3 <= 0.0 {
                return Err(Error::Invalid("metal volume"));
            }
            let material = registry.get(&part.material_id)?;
            let estimate = MetalEstimate::new(part.volume_mm3, material)?;
            let count = f64::from(part.instances);
            let grams = estimate.grams * count;
            let volume = part.volume_mm3 * count;
            if !grams.is_finite() || !volume.is_finite() {
                return Err(Error::Overflow("metal mass breakdown"));
            }
            total_grams += grams;
            total_volume_mm3 += volume;
            total_uncertainty_grams += estimate.uncertainty_grams * count;
            entries.push(MassEntry {
                key: key.to_owned(),
                material_id: estimate.material_id,
                density_g_cm3: estimate.density_g_cm3,
                volume_mm3: volume,
                instances: part.instances,
                grams,
                dwt: grams_to_dwt(grams)?,
            });
        }
        if !total_grams.is_finite() || !total_volume_mm3.is_finite() {
            return Err(Error::Overflow("metal mass breakdown"));
        }
        Ok(Self {
            entries,
            total_grams,
            total_dwt: grams_to_dwt(total_grams)?,
            total_volume_mm3,
            total_uncertainty_grams,
        })
    }
}
