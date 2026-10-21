// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed V5 exchange tables that are not geometry. Tables live on the document
//! so entities and layers reference one canonical record instead of copying
//! native payloads. Unknown native fields remain retained source, not invented.
use crate::{
    Error, Result,
    budget::{Limits, Usage, name},
    identity::{DimStyleId, HatchPatternId, LightId, LinePatternId, MaterialId, TextStyleId},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinePatternSegment {
    pub length_mm: f64,
    /// 1 = line, 2 = space. Mirrors ON_LinetypeSegment::eSegType.
    pub segment_type: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinePattern {
    pub id: LinePatternId,
    pub name: String,
    pub segments: Vec<LinePatternSegment>,
}
impl LinePattern {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("line pattern id"));
        }
        name(&self.name, l)?;
        let mut usage = Usage::array::<LinePatternSegment>(self.segments.capacity())?;
        usage.add(
            Usage {
                bytes: self.name.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?;
        for segment in &self.segments {
            if !segment.length_mm.is_finite() || segment.length_mm < 0. {
                return Err(Error::Invalid("line pattern segment length"));
            }
            if !matches!(segment.segment_type, 1 | 2) {
                return Err(Error::Invalid("line pattern segment type"));
            }
        }
        Ok(usage)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimStyle {
    pub id: DimStyleId,
    pub name: String,
    pub text_height: f64,
    pub length_resolution: f64,
}
impl DimStyle {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("dimension style id"));
        }
        name(&self.name, l)?;
        if !self.text_height.is_finite() || self.text_height <= 0. {
            return Err(Error::Invalid("dimension text height"));
        }
        if !self.length_resolution.is_finite() || self.length_resolution < 0. {
            return Err(Error::Invalid("dimension length resolution"));
        }
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderLight {
    pub id: LightId,
    pub name: String,
    pub style: u32,
    pub location: [f64; 3],
    pub direction: [f64; 3],
    pub diffuse: [u8; 4],
    pub intensity: f64,
}
impl RenderLight {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("light id"));
        }
        name(&self.name, l)?;
        if !self.location.iter().all(|v| v.is_finite())
            || !self.direction.iter().all(|v| v.is_finite())
            || !self.intensity.is_finite()
        {
            return Err(Error::Invalid("light fields"));
        }
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatchPatternLine {
    pub angle_radians: f64,
    pub base: [f64; 2],
    pub offset: [f64; 2],
    pub dashes: Vec<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatchPattern {
    pub id: HatchPatternId,
    pub name: String,
    pub fill_type: u32,
    pub lines: Vec<HatchPatternLine>,
}
impl HatchPattern {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("hatch pattern id"));
        }
        name(&self.name, l)?;
        let mut usage = Usage::array::<HatchPatternLine>(self.lines.capacity())?;
        for line in &self.lines {
            if !line.angle_radians.is_finite()
                || !line.base.iter().all(|v| v.is_finite())
                || !line.offset.iter().all(|v| v.is_finite())
                || !line.dashes.iter().all(|v| v.is_finite())
            {
                return Err(Error::Invalid("hatch pattern line"));
            }
            usage.add(Usage::array::<f64>(line.dashes.capacity())?, l)?;
        }
        usage.add(
            Usage {
                bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
                nodes: 1,
                ..Usage::default()
            },
            l,
        )?;
        Ok(usage)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderMaterial {
    pub id: MaterialId,
    pub name: String,
    pub diffuse: [u8; 4],
}
impl RenderMaterial {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("material id"));
        }
        name(&self.name, l)?;
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStyle {
    pub id: TextStyleId,
    pub name: String,
}
impl TextStyle {
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        if self.id.is_nil() {
            return Err(Error::Invalid("text style id"));
        }
        name(&self.name, l)?;
        Ok(Usage {
            bytes: std::mem::size_of::<Self>() as u64 + self.name.capacity() as u64,
            nodes: 1,
            ..Usage::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeMetadata {
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub line_patterns: BTreeMap<LinePatternId, LinePattern>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub text_styles: BTreeMap<TextStyleId, TextStyle>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub materials: BTreeMap<MaterialId, RenderMaterial>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub dim_styles: BTreeMap<DimStyleId, DimStyle>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub hatch_patterns: BTreeMap<HatchPatternId, HatchPattern>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        deserialize_with = "crate::identity::unique_uuid_map"
    )]
    pub lights: BTreeMap<LightId, RenderLight>,
}
impl ExchangeMetadata {
    pub fn is_empty(&self) -> bool {
        self.line_patterns.is_empty()
            && self.text_styles.is_empty()
            && self.materials.is_empty()
            && self.dim_styles.is_empty()
    }
    pub fn validate(&self, l: &Limits) -> Result<Usage> {
        let mut usage = Usage::btree::<LinePatternId, LinePattern>(self.line_patterns.len())?;
        for pattern in self.line_patterns.values() {
            usage.add(pattern.validate(l)?, l)?;
        }
        usage.add(
            Usage::btree::<TextStyleId, TextStyle>(self.text_styles.len())?,
            l,
        )?;
        for style in self.text_styles.values() {
            usage.add(style.validate(l)?, l)?;
        }
        usage.add(
            Usage::btree::<MaterialId, RenderMaterial>(self.materials.len())?,
            l,
        )?;
        for material in self.materials.values() {
            usage.add(material.validate(l)?, l)?;
        }
        Ok(usage)
    }
}
