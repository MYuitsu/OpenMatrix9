// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem asset shapes, cuts, sizes and the loader/instance contract (`OM9-L2-04-09`).
//!
//! The module owns the typed gem record and the procedural geometry the M05 gem tools
//! share. It never copies a Matrix gemstone mesh: every outline and facet ring below is
//! generated from the typed [`GemSpec`], and the canonical library is a table of sizes
//! and parameters, not a binary asset blob.
//!
//! Ownership split:
//!
//! * [`GemSpec`] / [`GemCut`] / [`GemShape`] — the schema owned by `OM9-L3-04-09-001`.
//! * [`GemLibrary`] / [`GemAsset`] — the canonical descriptor and size ladder owned by
//!   `OM9-L3-04-09-002` and read by the loader (`OM9-L3-04-09-003`).
//! * [`build_mesh`] / [`gem_entity`] — the procedural mesh and the document record.
//! * [`girdle_outline`] / [`gem_reference`] — the facet/girdle reference API consumed by
//!   settings and cutters (`OM9-L3-04-09-008`).
//! * [`GemInstance`] — a rigid placement of a spec, with a normalised orientation basis
//!   (`OM9-L3-04-09-006`).
//!
//! Units are millimetres. `length_mm` is the girdle's long axis, `width_mm` the short
//! axis and the total depth is `crown_height_mm + girdle_mm + pavilion_depth_mm`.

use crate::{
    Error, Result,
    geometry::{Geometry, Mesh, Point3, Transform},
    identity::ObjectId,
    model::{Document, Entity, Extension, GEM_EXTENSION_SLOT},
};
use serde::{Deserialize, Serialize};

/// Version of the typed spec record (`om9.gem_spec.v1`).
pub const GEM_SPEC_SCHEMA: &str = "om9.gem_spec.v1";
/// Version written into the `om9.gem` entity extension.
pub const GEM_PAYLOAD_VERSION: u32 = 1;
/// Smallest admitted dimension, in millimetres. A gem below this is a data error, not a
/// rounding accident.
pub const MIN_DIMENSION_MM: f64 = 0.05;
/// Largest admitted dimension, in millimetres.
pub const MAX_DIMENSION_MM: f64 = 500.0;
/// Default material token for the canonical library.
pub const DEFAULT_MATERIAL: &str = "diamond";
/// Default display colour of an asset without a material record.
pub const DEFAULT_COLOR: [u8; 4] = [180, 180, 180, 255];

/// Stable identity of one gem: the `GemId` the loader and the settings rows exchange. It is
/// a document object identity, so it survives save/open and Undo/Redo unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GemId(pub ObjectId);

impl GemId {
    pub fn new() -> Self {
        Self(ObjectId::new())
    }
    pub fn object(self) -> ObjectId {
        self.0
    }
    pub fn is_nil(self) -> bool {
        self.0.is_nil()
    }
    pub fn parse(text: &str) -> Result<Self> {
        let id: ObjectId = serde_json::from_str(&format!("\"{}\"", text.trim()))
            .map_err(|_| Error::Invalid("gem id"))?;
        if id.is_nil() {
            return Err(Error::Invalid("gem id"));
        }
        Ok(Self(id))
    }
}
impl Default for GemId {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for GemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The loader cut families (Matrix `Cut Type`: Diamond, Simple, Gem, Cabochon).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GemCut {
    /// The full faceted family (the loader default).
    Diamond,
    /// Simplified cuts with fewer facets.
    Simple,
    /// Generic faceted gem cuts.
    Gem,
    /// Dome-topped cabochon cuts.
    Cabochon,
}

impl GemCut {
    pub const ALL: [Self; 4] = [Self::Diamond, Self::Simple, Self::Gem, Self::Cabochon];

    pub fn tag(self) -> &'static str {
        match self {
            Self::Diamond => "diamond",
            Self::Simple => "simple",
            Self::Gem => "gem",
            Self::Cabochon => "cabochon",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Diamond => "Diamond Cut",
            Self::Simple => "Simple Cut",
            Self::Gem => "Gem Cut",
            Self::Cabochon => "Cabochon",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "diamond" | "diamond cut" | "diamondcut" => Ok(Self::Diamond),
            "simple" | "simple cut" | "simplecut" => Ok(Self::Simple),
            "gem" | "gem cut" | "gemcut" => Ok(Self::Gem),
            "cabochon" => Ok(Self::Cabochon),
            _ => Err(Error::Invalid("gem cut")),
        }
    }
    /// The shapes this cut offers, in the loader's display order.
    pub fn shapes(self) -> &'static [GemShape] {
        match self {
            Self::Diamond => &[
                GemShape::Round,
                GemShape::Oval,
                GemShape::Emerald,
                GemShape::Baguette,
                GemShape::Octagon,
                GemShape::Square,
                GemShape::Cushion,
                GemShape::Princess,
                GemShape::Pear,
                GemShape::Marquise,
                GemShape::Trillion,
                GemShape::Heart,
            ],
            Self::Simple => &[
                GemShape::Round,
                GemShape::Oval,
                GemShape::Emerald,
                GemShape::Baguette,
                GemShape::Cushion,
                GemShape::Square,
            ],
            Self::Gem => &[
                GemShape::Round,
                GemShape::Oval,
                GemShape::Emerald,
                GemShape::Baguette,
                GemShape::Cushion,
                GemShape::Octagon,
                GemShape::Trillion,
                GemShape::Pear,
                GemShape::Marquise,
            ],
            Self::Cabochon => &[
                GemShape::Round,
                GemShape::Oval,
                GemShape::Square,
                GemShape::Cushion,
                GemShape::Octagon,
            ],
        }
    }
    /// The default facet count of the cut. A cabochon has no faceted crown ring.
    pub fn default_facet_count(self) -> u32 {
        match self {
            Self::Diamond => 16,
            Self::Simple => 8,
            Self::Gem => 16,
            Self::Cabochon => 32,
        }
    }
    pub fn uses_pavilion_rows(self) -> bool {
        matches!(self, Self::Gem | Self::Simple)
    }
}

/// The girdle outline family. The shape decides the silhouette; the cut decides the
/// crown/pavilion treatment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GemShape {
    Round,
    Oval,
    Emerald,
    Baguette,
    Octagon,
    Square,
    Cushion,
    Princess,
    Pear,
    Marquise,
    Trillion,
    Heart,
}

impl GemShape {
    pub const ALL: [Self; 12] = [
        Self::Round,
        Self::Oval,
        Self::Emerald,
        Self::Baguette,
        Self::Octagon,
        Self::Square,
        Self::Cushion,
        Self::Princess,
        Self::Pear,
        Self::Marquise,
        Self::Trillion,
        Self::Heart,
    ];

    pub fn tag(self) -> &'static str {
        match self {
            Self::Round => "round",
            Self::Oval => "oval",
            Self::Emerald => "emerald",
            Self::Baguette => "baguette",
            Self::Octagon => "octagon",
            Self::Square => "square",
            Self::Cushion => "cushion",
            Self::Princess => "princess",
            Self::Pear => "pear",
            Self::Marquise => "marquise",
            Self::Trillion => "trillion",
            Self::Heart => "heart",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Round => "Round",
            Self::Oval => "Oval",
            Self::Emerald => "Emerald",
            Self::Baguette => "Baguette",
            Self::Octagon => "Octagon",
            Self::Square => "Square",
            Self::Cushion => "Cushion",
            Self::Princess => "Princess",
            Self::Pear => "Pear",
            Self::Marquise => "Marquise",
            Self::Trillion => "Trillion",
            Self::Heart => "Heart",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "round" | "circle" => Ok(Self::Round),
            "oval" => Ok(Self::Oval),
            "emerald" => Ok(Self::Emerald),
            "baguette" | "baguette cut" => Ok(Self::Baguette),
            "octagon" => Ok(Self::Octagon),
            "square" => Ok(Self::Square),
            "cushion" => Ok(Self::Cushion),
            "princess" => Ok(Self::Princess),
            "pear" => Ok(Self::Pear),
            "marquise" => Ok(Self::Marquise),
            "trillion" | "triangle" => Ok(Self::Trillion),
            "heart" => Ok(Self::Heart),
            _ => Err(Error::Invalid("gem shape")),
        }
    }
    /// The default length/width ratio of the shape. A round stone is 1:1.
    pub fn default_aspect(self) -> f64 {
        match self {
            Self::Round | Self::Square | Self::Princess | Self::Cushion | Self::Trillion => 1.0,
            Self::Oval => 1.30,
            Self::Emerald => 1.35,
            Self::Baguette => 1.60,
            Self::Octagon => 1.20,
            Self::Pear => 1.50,
            Self::Marquise => 1.90,
            Self::Heart => 1.05,
        }
    }
    /// The default corner cut as a ratio of the short side, for the step-cut silhouettes.
    pub fn default_corner_ratio(self) -> f64 {
        match self {
            Self::Emerald => 0.22,
            Self::Octagon => 0.29,
            Self::Cushion => 0.18,
            _ => 0.0,
        }
    }
    pub fn is_round(self) -> bool {
        self == Self::Round
    }
}

/// A decorated, validated gem: the single typed record the loader, the builders, the
/// property editor and the downstream reference API read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GemSpec {
    pub shape: GemShape,
    pub cut: GemCut,
    /// Girdle long axis, in millimetres.
    pub length_mm: f64,
    /// Girdle short axis, in millimetres.
    pub width_mm: f64,
    /// Girdle band thickness, in millimetres.
    pub girdle_mm: f64,
    /// Crown height from the girdle top to the table, in millimetres.
    pub crown_height_mm: f64,
    /// Pavilion depth from the girdle bottom to the culet, in millimetres.
    pub pavilion_depth_mm: f64,
    /// Crown angle from the girdle plane, in degrees. It sets the table size.
    pub crown_angle_deg: f64,
    /// Girdle facets (crown outline sides) around the stone.
    pub facet_count: u32,
    /// Pavilion rows for the step/generic cut (`0..=6`). A diamond uses one row.
    pub pavilion_rows: u32,
    /// Pavilion bulge, `0.0..=1.0`; 0 is a straight cone.
    pub bulge: f64,
    /// Corner cut of the emerald/octagon silhouette, in millimetres. 0 for the other shapes.
    pub corner_mm: f64,
    /// Optional custom planar profile (centred, millimetres) that replaces the parametric
    /// girdle outline. The profile is the custom-gem owner (`OM9-L3-04-09-004`) record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Vec<[f64; 2]>>,
}

impl GemSpec {
    /// The total table-to-culet depth, in millimetres.
    pub fn depth_mm(&self) -> f64 {
        self.crown_height_mm + self.girdle_mm + self.pavilion_depth_mm
    }
    /// The table scale relative to the girdle. A crown angle near the girdle plane leaves a
    /// small table; a steep crown leaves a wide one. The manual states that editing the crown
    /// angle changes the table size, which is this relation.
    pub fn table_ratio(&self) -> f64 {
        let half = self.width_mm.min(self.length_mm) * 0.5;
        if half <= 0.0 || !self.crown_angle_deg.is_finite() {
            return 0.5;
        }
        let value = 1.0 - self.crown_angle_deg.to_radians().tan() * self.crown_height_mm / half;
        value.clamp(0.08, 0.92)
    }
    pub fn aspect(&self) -> f64 {
        if self.width_mm > 0.0 {
            self.length_mm / self.width_mm
        } else {
            1.0
        }
    }
    /// Validate the typed record. Refusals are explicit: a caller never gets a clamped gem.
    pub fn validate(&self) -> Result<()> {
        if !self.cut.shapes().contains(&self.shape) {
            return Err(Error::Invalid("gem shape for cut"));
        }
        if let Some(profile) = &self.profile {
            validate_profile(profile)?;
        }
        for value in [
            self.length_mm,
            self.width_mm,
            self.girdle_mm,
            self.crown_height_mm,
            self.pavilion_depth_mm,
            self.crown_angle_deg,
            self.bulge,
            self.corner_mm,
        ] {
            if !value.is_finite() {
                return Err(Error::Invalid("gem dimension is not finite"));
            }
        }
        for value in [
            self.length_mm,
            self.width_mm,
            self.girdle_mm,
            self.crown_height_mm,
        ] {
            if !(MIN_DIMENSION_MM..=MAX_DIMENSION_MM).contains(&value) {
                return Err(Error::Invalid("gem dimension range"));
            }
        }
        if self.shape.is_round() && self.profile.is_none() {
            let tolerance = 1.0e-9 * self.width_mm.max(1.0);
            if (self.length_mm - self.width_mm).abs() > tolerance {
                return Err(Error::Invalid("round gem needs one diameter"));
            }
        }
        if self.width_mm > self.length_mm + 1.0e-9 * self.length_mm.max(1.0) {
            return Err(Error::Invalid("gem width exceeds length"));
        }
        if !(0.0..=MAX_DIMENSION_MM).contains(&self.pavilion_depth_mm) {
            return Err(Error::Invalid("gem pavilion depth"));
        }
        if self.pavilion_depth_mm <= 0.0 && self.cut != GemCut::Cabochon {
            return Err(Error::Invalid("gem pavilion depth is zero"));
        }
        if self.cut == GemCut::Cabochon && self.pavilion_depth_mm != 0.0 {
            return Err(Error::Invalid("cabochon pavilion depth"));
        }
        if !(0.1..=89.0).contains(&self.crown_angle_deg) {
            return Err(Error::Invalid("gem crown angle"));
        }
        if !(0.0..=1.0).contains(&self.bulge) {
            return Err(Error::Invalid("gem bulge"));
        }
        if self.pavilion_rows > 6 {
            return Err(Error::Invalid("gem pavilion rows"));
        }
        if !(3..=128).contains(&self.facet_count) {
            return Err(Error::Invalid("gem facet count"));
        }
        if self.corner_mm < 0.0 {
            return Err(Error::Invalid("gem corner"));
        }
        if self.corner_mm > 0.0
            && self.corner_mm > self.length_mm.min(self.width_mm) * 0.5 + 1.0e-12
        {
            return Err(Error::Invalid("gem corner exceeds the girdle"));
        }
        if self.table_ratio() <= 0.0 || self.table_ratio() >= 1.0 {
            return Err(Error::Invalid("gem table ratio"));
        }
        Ok(())
    }
    /// The depth split used by the loaders: a new depth keeps the crown/pavilion proportion.
    pub fn set_depth_mm(mut self, depth_mm: f64) -> Result<Self> {
        if !depth_mm.is_finite() || depth_mm <= self.girdle_mm {
            return Err(Error::Invalid("gem depth"));
        }
        let crown_share = if self.cut == GemCut::Cabochon {
            1.0
        } else {
            let old = self.crown_height_mm + self.pavilion_depth_mm;
            if old > 0.0 {
                self.crown_height_mm / old
            } else {
                0.34
            }
        };
        let stone = depth_mm - self.girdle_mm;
        self.crown_height_mm = stone * crown_share;
        self.pavilion_depth_mm = if self.cut == GemCut::Cabochon {
            0.0
        } else {
            stone * (1.0 - crown_share)
        };
        self.validate()?;
        Ok(self)
    }
    /// Builder sizing: set the girdle axes, keeping the shape's round rule.
    pub fn set_girdle_mm(mut self, length_mm: f64, width_mm: f64) -> Result<Self> {
        if self.shape.is_round() && self.profile.is_none() {
            let diameter = width_mm.min(length_mm);
            self.length_mm = diameter;
            self.width_mm = diameter;
        } else {
            self.length_mm = length_mm;
            self.width_mm = width_mm;
        }
        self.validate()?;
        Ok(self)
    }
    pub fn set_crown_angle_deg(mut self, degrees: f64) -> Result<Self> {
        self.crown_angle_deg = degrees;
        self.validate()?;
        Ok(self)
    }
    pub fn set_facet_count(mut self, count: u32) -> Result<Self> {
        self.facet_count = count;
        self.validate()?;
        Ok(self)
    }
    pub fn set_girdle_mm_thickness(mut self, thickness: f64) -> Result<Self> {
        self.girdle_mm = thickness;
        self.validate()?;
        Ok(self)
    }
    pub fn set_corner_mm(mut self, corner: f64) -> Result<Self> {
        self.corner_mm = corner;
        self.validate()?;
        Ok(self)
    }
    pub fn set_pavilion_rows(mut self, rows: u32) -> Result<Self> {
        self.pavilion_rows = rows;
        self.validate()?;
        Ok(self)
    }
    pub fn set_bulge(mut self, bulge: f64) -> Result<Self> {
        self.bulge = bulge;
        self.validate()?;
        Ok(self)
    }
    pub fn set_pavilion_depth_mm(mut self, depth: f64) -> Result<Self> {
        self.pavilion_depth_mm = depth;
        self.validate()?;
        Ok(self)
    }
    pub fn to_json(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Documented<'a> {
            schema: &'static str,
            spec: &'a GemSpec,
        }
        serde_json::to_string(&Documented {
            schema: GEM_SPEC_SCHEMA,
            spec: self,
        })
        .map_err(|_| Error::Invalid("gem spec json"))
    }
    pub fn from_json(text: &str) -> Result<Self> {
        #[derive(Deserialize)]
        struct Documented {
            schema: String,
            spec: GemSpec,
        }
        let document: Documented =
            serde_json::from_str(text).map_err(|_| Error::Invalid("gem spec schema"))?;
        if document.schema != GEM_SPEC_SCHEMA {
            return Err(Error::Invalid("gem spec version"));
        }
        document.spec.validate()?;
        Ok(document.spec)
    }
    /// A round brilliant-shaped spec: one diameter and a total depth.
    pub fn round(diameter_mm: f64, depth_mm: f64) -> Result<Self> {
        Self::from_outer_dimensions(
            GemShape::Round,
            GemCut::Diamond,
            diameter_mm,
            diameter_mm,
            depth_mm,
        )
    }
    /// An oval brilliant-shaped spec: length, width and a total depth.
    pub fn oval(length_mm: f64, width_mm: f64, depth_mm: f64) -> Result<Self> {
        Self::from_outer_dimensions(
            GemShape::Oval,
            GemCut::Diamond,
            length_mm,
            width_mm,
            depth_mm,
        )
    }
    /// The loader's outer-dimension entry point: length, width and total depth.
    pub fn from_outer_dimensions(
        shape: GemShape,
        cut: GemCut,
        length_mm: f64,
        width_mm: f64,
        depth_mm: f64,
    ) -> Result<Self> {
        let facet_count = cut.default_facet_count();
        let girdle_mm = (width_mm.min(length_mm) * 0.02).max(0.06);
        let mut spec = Self {
            shape,
            cut,
            length_mm,
            width_mm,
            girdle_mm,
            crown_height_mm: 1.0,
            pavilion_depth_mm: 1.0,
            crown_angle_deg: 34.5,
            facet_count,
            pavilion_rows: if cut.uses_pavilion_rows() { 2 } else { 0 },
            bulge: if cut == GemCut::Gem { 0.25 } else { 0.0 },
            corner_mm: shape.default_corner_ratio() * width_mm.min(length_mm),
            profile: None,
        };
        let stone = (depth_mm - girdle_mm).max(MIN_DIMENSION_MM);
        spec.crown_height_mm = stone * 0.34;
        spec.pavilion_depth_mm = if cut == GemCut::Cabochon {
            0.0
        } else {
            stone * 0.66
        };
        if cut == GemCut::Cabochon {
            spec.crown_height_mm = stone;
        }
        spec.validate()?;
        Ok(spec)
    }
    /// The emerald builder: length, width, corner, depth and girdle are explicit.
    pub fn emerald(
        length_mm: f64,
        width_mm: f64,
        corner_mm: f64,
        depth_mm: f64,
        girdle_mm: f64,
    ) -> Result<Self> {
        let mut spec = Self::from_outer_dimensions(
            GemShape::Emerald,
            GemCut::Gem,
            length_mm,
            width_mm,
            depth_mm,
        )?;
        spec.girdle_mm = girdle_mm;
        spec.corner_mm = corner_mm;
        spec.facet_count = 8;
        spec.pavilion_rows = 2;
        spec = spec.set_depth_mm(depth_mm)?;
        Ok(spec)
    }
    /// The baguette builder: a rectangular step cut with an optional corner.
    pub fn baguette(
        length_mm: f64,
        width_mm: f64,
        corner_mm: f64,
        depth_mm: f64,
        girdle_mm: f64,
    ) -> Result<Self> {
        let mut spec = Self::from_outer_dimensions(
            GemShape::Baguette,
            GemCut::Gem,
            length_mm,
            width_mm,
            depth_mm,
        )?;
        spec.girdle_mm = girdle_mm;
        spec.corner_mm = corner_mm;
        spec.facet_count = if corner_mm > 0.0 { 8 } else { 4 };
        spec.pavilion_rows = 1;
        spec = spec.set_depth_mm(depth_mm)?;
        Ok(spec)
    }
    /// The custom builder: a closed planar profile plus the faceted parameters.
    pub fn from_profile(
        profile: &[[f64; 2]],
        cut_mode: GemCut,
        params: &CustomGemParams,
    ) -> Result<Self> {
        validate_profile(profile)?;
        // The gem frame's X axis is the long axis; a profile drawn taller than wide is
        // rotated into that frame instead of being stretched.
        let (min_x, max_x, min_y, max_y) = profile_bounds(profile);
        let mut points = profile.to_vec();
        if (max_x - min_x) < (max_y - min_y) {
            for point in &mut points {
                *point = [-point[1], point[0]];
            }
        }
        let (min_x, max_x, min_y, max_y) = profile_bounds(&points);
        let length_mm = (max_x - min_x).max(MIN_DIMENSION_MM);
        let width_mm = (max_y - min_y).max(MIN_DIMENSION_MM);
        let centered: Vec<[f64; 2]> = points
            .iter()
            .map(|p| {
                [
                    (p[0] - (min_x + max_x) * 0.5),
                    (p[1] - (min_y + max_y) * 0.5),
                ]
            })
            .collect();
        let facet_count = params.facet_count.max(centered.len() as u32);
        if !(3..=128).contains(&facet_count) {
            return Err(Error::Invalid("custom gem facet count"));
        }
        let mut spec = Self {
            shape: GemShape::Cushion,
            cut: if cut_mode == GemCut::Cabochon {
                GemCut::Cabochon
            } else {
                cut_mode
            },
            length_mm,
            width_mm,
            girdle_mm: params.girdle_mm,
            crown_height_mm: params.crown_height_mm,
            pavilion_depth_mm: params.pavilion_depth_mm,
            crown_angle_deg: params.crown_angle_deg,
            facet_count,
            pavilion_rows: params.pavilion_rows,
            bulge: params.bulge,
            corner_mm: 0.0,
            profile: Some(centered),
        };
        if spec.cut == GemCut::Cabochon {
            spec.pavilion_depth_mm = 0.0;
        }
        spec.validate()?;
        Ok(spec)
    }
    /// The canonical library key of this spec (`round-diamond`).
    pub fn asset_key(&self) -> String {
        format!("{}-{}", self.shape.tag(), self.cut.tag())
    }
}

/// Parameters of the custom gem builder. All lengths are millimetres, angles degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomGemParams {
    pub facet_count: u32,
    pub crown_angle_deg: f64,
    pub crown_height_mm: f64,
    pub girdle_mm: f64,
    pub pavilion_depth_mm: f64,
    pub pavilion_rows: u32,
    pub bulge: f64,
}

impl Default for CustomGemParams {
    fn default() -> Self {
        Self {
            facet_count: 16,
            crown_angle_deg: 34.5,
            crown_height_mm: 1.6,
            girdle_mm: 0.15,
            pavilion_depth_mm: 3.2,
            pavilion_rows: 2,
            bulge: 0.25,
        }
    }
}

fn profile_bounds(profile: &[[f64; 2]]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in profile {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);
        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
    }
    (min_x, max_x, min_y, max_y)
}

/// Refuse an open, self-intersecting, non-finite or zero-area profile with an explicit error.
pub fn validate_profile(profile: &[[f64; 2]]) -> Result<()> {
    if profile.len() < 3 {
        return Err(Error::Invalid("gem profile needs three points"));
    }
    if profile.len() > 128 {
        return Err(Error::Invalid("gem profile point count"));
    }
    for point in profile {
        if !point[0].is_finite() || !point[1].is_finite() {
            return Err(Error::Invalid("gem profile is not finite"));
        }
    }
    // The profile is a closed loop: a repeated last point is accepted, but a profile whose
    // first and last point are far apart means an open curve was passed by mistake.
    let first = profile[0];
    let last = profile[profile.len() - 1];
    let span = profile
        .iter()
        .map(|p| p[0].abs().max(p[1].abs()))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let mut points = profile;
    if (first[0] - last[0]).hypot(first[1] - last[1]) <= 1.0e-9 * span {
        points = &profile[..profile.len() - 1];
    }
    if points.len() < 3 {
        return Err(Error::Invalid("gem profile needs three points"));
    }
    if polygon_area(points) <= 1.0e-12 * span * span {
        return Err(Error::Invalid("gem profile has zero area"));
    }
    let n = points.len();
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];
        for j in (i + 1)..n {
            // Adjacent edges share an endpoint, and the wrap pair shares the first point.
            if j == i || (j + 1) % n == i || (i + 1) % n == j || (j + 1) % n == (i + 1) % n {
                continue;
            }
            let c = points[j];
            let d = points[(j + 1) % n];
            if segments_cross(a, b, c, d) {
                return Err(Error::Invalid("gem profile is self-intersecting"));
            }
        }
    }
    Ok(())
}

fn polygon_area(points: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a[0] * b[1] - b[0] * a[1];
    }
    (area * 0.5).abs()
}

fn segments_cross(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    fn orient(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> f64 {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    }
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0)
}

/// A published asset size: the loader lists these in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GemSize {
    pub length_mm: f64,
    pub width_mm: f64,
    pub depth_mm: f64,
}

impl GemSize {
    pub fn new(length_mm: f64, width_mm: f64, depth_mm: f64) -> Result<Self> {
        let size = Self {
            length_mm,
            width_mm,
            depth_mm,
        };
        size.validate()?;
        Ok(size)
    }
    pub fn validate(&self) -> Result<()> {
        if [self.length_mm, self.width_mm, self.depth_mm]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(Error::Invalid("gem size"));
        }
        Ok(())
    }
    /// The short-axis span used to match a size across shapes (`OM9-L3-04-09-003`).
    pub fn span_mm(&self) -> f64 {
        self.width_mm.min(self.length_mm)
    }
    pub fn label(&self) -> String {
        if (self.length_mm - self.width_mm).abs() <= 1.0e-9 {
            format!("{:.2} mm", self.length_mm)
        } else {
            format!("{:.2} x {:.2} mm", self.length_mm, self.width_mm)
        }
    }
}

/// One canonical gem asset: the descriptor and its size ladder. The descriptor is
/// procedural, so no unlicensed stone model enters the repository.
#[derive(Debug, Clone, PartialEq)]
pub struct GemAsset {
    pub shape: GemShape,
    pub cut: GemCut,
    /// Stable asset key, e.g. `round-diamond`.
    pub key: String,
    pub name: String,
    pub material: String,
    pub color: [u8; 4],
    /// Facet count used for a size without an explicit custom facet count.
    pub facet_count: u32,
    pub pavilion_rows: u32,
    pub bulge: f64,
    pub sizes: Vec<GemSize>,
}

impl GemAsset {
    pub fn spec_for(&self, size: &GemSize) -> Result<GemSpec> {
        let mut spec = GemSpec::from_outer_dimensions(
            self.shape,
            self.cut,
            size.length_mm,
            size.width_mm,
            size.depth_mm,
        )?;
        spec.facet_count = self.facet_count;
        spec.pavilion_rows = self.pavilion_rows;
        spec.bulge = self.bulge;
        if self.shape.default_corner_ratio() > 0.0 {
            spec.corner_mm = self.shape.default_corner_ratio() * size.width_mm.min(size.length_mm);
        }
        spec.validate()?;
        Ok(spec)
    }
}

/// The canonical, procedural gem library. The size ladders are OM9 defaults authored here;
/// they are not copied from Matrix. Every listed value is millimetres.
#[derive(Debug, Clone, PartialEq)]
pub struct GemLibrary {
    assets: Vec<GemAsset>,
}

impl Default for GemLibrary {
    fn default() -> Self {
        Self::canonical()
    }
}

impl GemLibrary {
    /// The authoring ladders. A `1.0` entry with one axis means a round diameter; the
    /// `(length, width)` entries are the elongated shapes.
    fn ladder(shape: GemShape) -> Vec<GemSize> {
        let round: &[f64] = &[
            1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0, 2.25, 2.5, 2.75, 3.0, 3.25, 3.5,
            3.75, 4.0, 4.25, 4.5, 4.75, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.0,
        ];
        let aspect = shape.default_aspect();
        round
            .iter()
            .map(|diameter| {
                let width = *diameter;
                let length = if shape.is_round() {
                    *diameter
                } else {
                    width * aspect
                };
                let depth = (width * 0.62).max(0.4);
                GemSize::new(
                    (length * 100.0).round() / 100.0,
                    width,
                    (depth * 100.0).round() / 100.0,
                )
                .expect("authored size")
            })
            .collect()
    }

    /// Build the canonical library (cut x shape descriptors with size ladders).
    pub fn canonical() -> Self {
        let mut assets = Vec::new();
        for cut in GemCut::ALL {
            for shape in cut.shapes() {
                let shape = *shape;
                let key = format!("{}-{}", shape.tag(), cut.tag());
                let name = format!("{} {}", shape.label(), cut.label());
                let facet_count = match (cut, shape) {
                    (GemCut::Cabochon, _) => 32,
                    (_, GemShape::Emerald) | (_, GemShape::Baguette) => 8,
                    (_, GemShape::Trillion) => 12,
                    (GemCut::Simple, _) => 8,
                    _ => 16,
                };
                let pavilion_rows = match (cut, shape) {
                    (GemCut::Cabochon, _) => 0,
                    (_, GemShape::Emerald) => 2,
                    (_, GemShape::Baguette) => 1,
                    (GemCut::Gem, _) => 2,
                    _ => 0,
                };
                let bulge = if cut == GemCut::Gem { 0.25 } else { 0.0 };
                assets.push(GemAsset {
                    shape,
                    cut,
                    key,
                    name,
                    material: DEFAULT_MATERIAL.to_owned(),
                    color: DEFAULT_COLOR,
                    facet_count,
                    pavilion_rows,
                    bulge,
                    sizes: Self::ladder(shape),
                });
            }
        }
        Self { assets }
    }

    pub fn assets(&self) -> &[GemAsset] {
        &self.assets
    }
    pub fn asset(&self, shape: GemShape, cut: GemCut) -> Option<&GemAsset> {
        self.assets
            .iter()
            .find(|asset| asset.shape == shape && asset.cut == cut)
    }
    pub fn assets_for_cut(&self, cut: GemCut) -> Vec<&GemAsset> {
        self.assets
            .iter()
            .filter(|asset| asset.cut == cut)
            .collect()
    }
    /// The nearest size by short-axis span. A shape without that size falls back to the
    /// closest one instead of failing.
    pub fn nearest_size(&self, shape: GemShape, cut: GemCut, span_mm: f64) -> Option<&GemSize> {
        let asset = self.asset(shape, cut)?;
        asset.sizes.iter().min_by(|a, b| {
            (a.span_mm() - span_mm)
                .abs()
                .total_cmp(&(b.span_mm() - span_mm).abs())
        })
    }
}

/// The built gem mesh and its feasibility numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct GemMesh {
    pub mesh: Mesh,
    /// Every undirected edge is shared by exactly two faces.
    pub closed: bool,
    /// Signed volume of the closed mesh, in mm^3.
    pub volume_mm3: f64,
    /// Axis-aligned extents in millimetres: length (X), width (Y), depth (Z).
    pub extents_mm: [f64; 3],
}

/// Build the procedural mesh of a validated spec. The result is a closed solid whose
/// extents match `length_mm`, `width_mm` and `depth_mm` within tolerance.
pub fn build_mesh(spec: &GemSpec) -> Result<GemMesh> {
    spec.validate()?;
    let outline = base_outline(spec)?;
    let points = outline.len();
    if points < 3 {
        return Err(Error::Invalid("gem outline"));
    }
    let girdle_top = spec.girdle_mm * 0.5;
    let table_ratio = spec.table_ratio();
    let top = girdle_top + spec.crown_height_mm;
    let table_outline: Vec<[f64; 2]> = outline
        .iter()
        .map(|p| [p[0] * table_ratio, p[1] * table_ratio])
        .collect();
    let table_center = [0.0, 0.0];

    let mut vertices: Vec<Point3> = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let push = |point: Point3, vertices: &mut Vec<Point3>| -> u32 {
        let index = vertices.len() as u32;
        vertices.push(point);
        index
    };

    // Girdle: top ring, then bottom ring.
    let mut girdle_top_ring = Vec::with_capacity(points);
    for p in &outline {
        girdle_top_ring.push(push([p[0], p[1], girdle_top], &mut vertices));
    }
    let mut girdle_bottom_ring = Vec::with_capacity(points);
    for p in &outline {
        girdle_bottom_ring.push(push([p[0], p[1], -girdle_top], &mut vertices));
    }

    // Girdle band, shared by every cut family.
    ring_faces(&mut faces, &girdle_bottom_ring, &girdle_top_ring);

    if spec.cut == GemCut::Cabochon {
        // Dome from the girdle top to an apex, flat bottom face at the girdle bottom.
        let rows = 6usize;
        let mut previous = girdle_top_ring.clone();
        for row in 1..rows {
            let phi = std::f64::consts::FRAC_PI_2 * row as f64 / rows as f64;
            let scale = phi.cos();
            let z = girdle_top + spec.crown_height_mm * phi.sin();
            let mut ring = Vec::with_capacity(points);
            for p in &outline {
                ring.push(push([p[0] * scale, p[1] * scale, z], &mut vertices));
            }
            ring_faces(&mut faces, &previous, &ring);
            previous = ring;
        }
        let apex = push([0.0, 0.0, top], &mut vertices);
        for index in 0..points {
            let a = previous[index];
            let b = previous[(index + 1) % points];
            faces.push(vec![a, b, apex]);
        }
        // Flat bottom.
        for index in 1..points - 1 {
            faces.push(vec![
                girdle_bottom_ring[0],
                girdle_bottom_ring[index + 1],
                girdle_bottom_ring[index],
            ]);
        }
    } else {
        // Table ring and table face.
        let mut table_ring = Vec::with_capacity(points);
        for p in &table_outline {
            table_ring.push(push([p[0], p[1], top], &mut vertices));
        }
        for index in 1..points - 1 {
            faces.push(vec![
                table_ring[0],
                table_ring[index],
                table_ring[index + 1],
            ]);
        }
        // Crown side faces.
        ring_faces(&mut faces, &girdle_top_ring, &table_ring);
        // Pavilion: rows of shrinking rings, then the culet point.
        let rows = if spec.cut.uses_pavilion_rows() {
            spec.pavilion_rows as usize
        } else {
            1
        }
        .max(1);
        let mut previous = girdle_bottom_ring.clone();
        for row in 1..rows {
            let t = row as f64 / rows as f64;
            let scale = ((1.0 - t).max(0.0)).powf(1.0 + spec.bulge);
            let z = -girdle_top - spec.pavilion_depth_mm * t;
            let mut ring = Vec::with_capacity(points);
            for p in &outline {
                ring.push(push([p[0] * scale, p[1] * scale, z], &mut vertices));
            }
            ring_faces(&mut faces, &previous, &ring);
            previous = ring;
        }
        let culet = push(
            [0.0, 0.0, -(girdle_top + spec.pavilion_depth_mm)],
            &mut vertices,
        );
        for index in 0..points {
            let a = previous[index];
            let b = previous[(index + 1) % points];
            faces.push(vec![a, b, culet]);
        }
        // `table_center` exists so the table polygon keeps one stable vertex convention.
        let _ = table_center;
    }

    if faces.iter().any(|face| face.len() < 3) {
        return Err(Error::Invalid("gem mesh face"));
    }
    // Facet rings are built in different frames (girdle, crown, pavilion); a shared edge must be
    // traversed in opposite directions by its two faces, otherwise the signed volume cancels and a
    // face is hidden inside the solid. Walk the adjacency once and flip whole faces.
    orient_faces_consistently(&mut faces)?;
    let mut volume = signed_volume(&vertices, &faces);
    if volume < 0.0 {
        for face in &mut faces {
            face.reverse();
        }
        volume = -volume;
    }
    if !volume.is_finite() || volume <= 0.0 {
        return Err(Error::Invalid("gem mesh volume"));
    }
    let mesh = Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let closed = mesh_is_closed(&mesh);
    if !closed {
        return Err(Error::Invalid("gem mesh open"));
    }
    let (min, max) = mesh_bounds(&mesh)?;
    let extents = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let tolerance = 1.0e-6 * spec.length_mm.max(1.0);
    for (actual, expected, name) in [
        (extents[0], spec.length_mm, "length"),
        (extents[1], spec.width_mm, "width"),
        (extents[2], spec.depth_mm(), "depth"),
    ] {
        if (actual - expected).abs() > tolerance.max(1.0e-9) {
            return Err(Error::Invalid(match name {
                "length" => "gem mesh length",
                "width" => "gem mesh width",
                _ => "gem mesh depth",
            }));
        }
    }
    Ok(GemMesh {
        mesh,
        closed,
        volume_mm3: volume,
        extents_mm: extents,
    })
}

/// Make every face traverse each shared edge opposite to its neighbour. The input is a closed
/// manifold, so one breadth-first pass fixes inconsistent ring/fan winding without moving a vertex.
fn orient_faces_consistently(faces: &mut [Vec<u32>]) -> Result<()> {
    use std::collections::{BTreeMap, VecDeque};
    let mut edges: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
    for (index, face) in faces.iter().enumerate() {
        for corner in 0..face.len() {
            let a = face[corner];
            let b = face[(corner + 1) % face.len()];
            if a == b {
                return Err(Error::Invalid("gem mesh face"));
            }
            let key = if a < b { (a, b) } else { (b, a) };
            edges.entry(key).or_default().push((index, a < b));
        }
    }
    if edges.values().any(|uses| uses.len() != 2) {
        return Err(Error::Invalid("gem mesh open"));
    }
    let mut flip = vec![false; faces.len()];
    let mut seen = vec![false; faces.len()];
    let mut queue = VecDeque::new();
    for start in 0..faces.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        queue.push_back(start);
        while let Some(face) = queue.pop_front() {
            let corners = faces[face].clone();
            for corner in 0..corners.len() {
                let a = corners[corner];
                let b = corners[(corner + 1) % corners.len()];
                let key = if a < b { (a, b) } else { (b, a) };
                let forward = a < b;
                let uses = &edges[&key];
                let (neighbour, neighbour_forward) =
                    if uses[0].0 == face { uses[1] } else { uses[0] };
                // Both faces must traverse the edge in opposite directions once their own flip is
                // applied: forward(face) ^ flip(face) != forward(neighbour) ^ flip(neighbour).
                let neighbour_flip = forward ^ flip[face] ^ neighbour_forward ^ true;
                if !seen[neighbour] {
                    seen[neighbour] = true;
                    flip[neighbour] = neighbour_flip;
                    queue.push_back(neighbour);
                }
            }
        }
    }
    for (face, reversed) in faces.iter_mut().zip(flip) {
        if reversed {
            face.reverse();
        }
    }
    Ok(())
}

fn ring_faces(faces: &mut Vec<Vec<u32>>, inner: &[u32], outer: &[u32]) {
    // `inner` is the lower/previous ring and `outer` the upper/next ring; the quad order
    // gives a consistent winding once the whole mesh is oriented by its signed volume.
    let n = inner.len();
    for index in 0..n {
        let next = (index + 1) % n;
        faces.push(vec![inner[index], inner[next], outer[next], outer[index]]);
    }
}

/// Compute the axis-aligned bounds of a mesh.
pub fn mesh_bounds(mesh: &Mesh) -> Result<(Point3, Point3)> {
    if mesh.vertices.is_empty() {
        return Err(Error::Invalid("gem mesh is empty"));
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in &mesh.vertices {
        if point.iter().any(|v| !v.is_finite()) {
            return Err(Error::Invalid("gem mesh is not finite"));
        }
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    Ok((min, max))
}

fn mesh_is_closed(mesh: &Mesh) -> bool {
    let mut edges: std::collections::BTreeMap<(u32, u32), u32> = std::collections::BTreeMap::new();
    for face in &mesh.faces {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            if a == b || a as usize >= mesh.vertices.len() || b as usize >= mesh.vertices.len() {
                return false;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_insert(0) += 1;
        }
    }
    edges.values().all(|count| *count == 2)
}

fn signed_volume(vertices: &[Point3], faces: &[Vec<u32>]) -> f64 {
    let mut volume = 0.0;
    for face in faces {
        let a = vertices[face[0] as usize];
        for index in 1..face.len() - 1 {
            let b = vertices[face[index] as usize];
            let c = vertices[face[index + 1] as usize];
            volume += (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
        }
    }
    volume
}

/// The girdle outline in the gem frame at `z = 0`, in counter-clockwise order. A custom
/// profile replaces the parametric base outline and is already centred.
pub fn girdle_outline(spec: &GemSpec) -> Result<Vec<Point3>> {
    spec.validate()?;
    let base = base_outline(spec)?;
    Ok(base.into_iter().map(|p| [p[0], p[1], 0.0]).collect())
}

/// The table outline at the table plane (girdle top + crown height).
pub fn table_outline(spec: &GemSpec) -> Result<Vec<Point3>> {
    spec.validate()?;
    let ratio = spec.table_ratio();
    let z = spec.girdle_mm * 0.5 + spec.crown_height_mm;
    Ok(base_outline(spec)?
        .into_iter()
        .map(|p| [p[0] * ratio, p[1] * ratio, z])
        .collect())
}

/// The 2D profile (radius, z) from the culet to the table centre. Downstream settings use
/// it to build a head or a bezel profile.
pub fn profile_points(spec: &GemSpec) -> Result<Vec<[f64; 2]>> {
    spec.validate()?;
    let radius = spec.width_mm.min(spec.length_mm) * 0.5;
    let girdle_top = spec.girdle_mm * 0.5;
    let mut points = vec![[0.0, -(girdle_top + spec.pavilion_depth_mm)]];
    if spec.cut != GemCut::Cabochon {
        let rows = if spec.cut.uses_pavilion_rows() {
            spec.pavilion_rows as usize
        } else {
            1
        }
        .max(1);
        for row in (1..rows).rev() {
            let t = row as f64 / rows as f64;
            let scale = ((1.0 - t).max(0.0)).powf(1.0 + spec.bulge);
            points.push([radius * scale, -girdle_top - spec.pavilion_depth_mm * t]);
        }
    }
    points.push([radius, -girdle_top]);
    points.push([radius, girdle_top]);
    if spec.cut == GemCut::Cabochon {
        let rows = 6usize;
        for row in (1..rows).rev() {
            let phi = std::f64::consts::FRAC_PI_2 * row as f64 / rows as f64;
            points.push([
                radius * phi.cos(),
                girdle_top + spec.crown_height_mm * phi.sin(),
            ]);
        }
        points.push([0.0, girdle_top + spec.crown_height_mm]);
    } else {
        points.push([
            radius * spec.table_ratio(),
            girdle_top + spec.crown_height_mm,
        ]);
        points.push([0.0, girdle_top + spec.crown_height_mm]);
    }
    Ok(points)
}

/// A named geometric reference of one gem, for downstream settings and cutters. The points
/// are real geometry points; a caller never indexes the display mesh triangles.
#[derive(Debug, Clone, PartialEq)]
pub struct GemReferenceGeometry {
    pub name: String,
    /// Closed outline points, in the gem frame.
    pub points: Vec<Point3>,
    pub closed: bool,
    /// Outward unit normal of the reference plane, or the facet normal.
    pub normal: Point3,
}

/// Resolve a geometric reference name of a gem spec.
///
/// Names: `girdle`, `table`, `culet`, `crown_facet:<n>`, `pavilion_facet:<n>`, `profile`.
pub fn gem_reference(spec: &GemSpec, name: &str) -> Result<GemReferenceGeometry> {
    spec.validate()?;
    let trimmed = name.trim().to_ascii_lowercase();
    if trimmed == "girdle" {
        let points = girdle_outline(spec)?;
        return Ok(GemReferenceGeometry {
            name: "girdle".to_owned(),
            normal: [0.0, 0.0, 1.0],
            closed: true,
            points,
        });
    }
    if trimmed == "table" {
        let points = table_outline(spec)?;
        return Ok(GemReferenceGeometry {
            name: "table".to_owned(),
            normal: [0.0, 0.0, 1.0],
            closed: true,
            points,
        });
    }
    if trimmed == "culet" {
        return Ok(GemReferenceGeometry {
            name: "culet".to_owned(),
            points: vec![[0.0, 0.0, -(spec.girdle_mm * 0.5 + spec.pavilion_depth_mm)]],
            closed: false,
            normal: [0.0, 0.0, -1.0],
        });
    }
    if trimmed == "profile" {
        let profile = profile_points(spec)?;
        let points: Vec<Point3> = profile.iter().map(|p| [p[0], 0.0, p[1]]).collect();
        return Ok(GemReferenceGeometry {
            name: "profile".to_owned(),
            normal: [0.0, 1.0, 0.0],
            closed: false,
            points,
        });
    }
    if let Some(index) = trimmed.strip_prefix("crown_facet:") {
        let index: usize = index
            .parse()
            .map_err(|_| Error::Invalid("gem facet index"))?;
        return crown_facet(spec, index);
    }
    if let Some(index) = trimmed.strip_prefix("pavilion_facet:") {
        let index: usize = index
            .parse()
            .map_err(|_| Error::Invalid("gem facet index"))?;
        return pavilion_facet(spec, index);
    }
    Err(Error::Invalid("gem reference name"))
}

/// A geometric reference bound to the gem that owns it (`OM9-L3-04-09-008`): the identity
/// the caller asked for plus the geometry, so a settings command can name both.
#[derive(Debug, Clone, PartialEq)]
pub struct GemGeometryReference {
    pub gem: GemId,
    pub reference: GemReferenceGeometry,
}

/// Resolve a named reference of a specific gem, refusing a gem whose payload does not own
/// that identity.
pub fn gem_reference_for(record: &GemRecord, name: &str) -> Result<GemGeometryReference> {
    if record.gem_id.is_nil() {
        return Err(Error::Invalid("gem id"));
    }
    Ok(GemGeometryReference {
        gem: record.gem_id,
        reference: gem_reference(&record.spec, name)?,
    })
}

/// One crown facet: the quad between girdle vertex `index` and its table counterpart.
pub fn crown_facet(spec: &GemSpec, index: usize) -> Result<GemReferenceGeometry> {
    spec.validate()?;
    let girdle = girdle_outline(spec)?;
    let table = table_outline(spec)?;
    let n = girdle.len();
    if index >= n {
        return Err(Error::Invalid("gem facet index"));
    }
    let next = (index + 1) % n;
    let points = vec![girdle[index], girdle[next], table[next], table[index]];
    let normal = facet_normal(&points)?;
    Ok(GemReferenceGeometry {
        name: format!("crown_facet:{index}"),
        points,
        closed: true,
        normal,
    })
}

/// One pavilion facet at the girdle edge `index` (or its first ring row).
pub fn pavilion_facet(spec: &GemSpec, index: usize) -> Result<GemReferenceGeometry> {
    spec.validate()?;
    if spec.cut == GemCut::Cabochon {
        return Err(Error::Invalid("cabochon has no pavilion facet"));
    }
    let girdle = girdle_outline(spec)?;
    let n = girdle.len();
    if index >= n {
        return Err(Error::Invalid("gem facet index"));
    }
    let next = (index + 1) % n;
    let rows = if spec.cut.uses_pavilion_rows() {
        spec.pavilion_rows as usize
    } else {
        1
    }
    .max(1);
    let t = 1.0 / rows as f64;
    let scale = ((1.0 - t).max(0.0)).powf(1.0 + spec.bulge);
    let z = -(spec.girdle_mm * 0.5 + spec.pavilion_depth_mm * t);
    let a = [girdle[index][0] * scale, girdle[index][1] * scale, z];
    let b = [girdle[next][0] * scale, girdle[next][1] * scale, z];
    let bottom = [girdle[index][0], girdle[index][1], -spec.girdle_mm * 0.5];
    let bottom_next = [girdle[next][0], girdle[next][1], -spec.girdle_mm * 0.5];
    let points = vec![bottom, a, b, bottom_next];
    let normal = facet_normal(&points)?;
    Ok(GemReferenceGeometry {
        name: format!("pavilion_facet:{index}"),
        points,
        closed: true,
        normal,
    })
}

fn facet_normal(points: &[Point3]) -> Result<Point3> {
    let a = points[0];
    let b = points[1];
    let c = points[2];
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let mut normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("gem facet normal"));
    }
    for value in &mut normal {
        *value /= length;
    }
    Ok(normal)
}

/// The parametric silhouette of the shape, centred and in millimetres.
fn base_outline(spec: &GemSpec) -> Result<Vec<[f64; 2]>> {
    if let Some(profile) = &spec.profile {
        validate_profile(profile)?;
        return Ok(profile.clone());
    }
    let half_x = spec.length_mm * 0.5;
    let half_y = spec.width_mm * 0.5;
    let count = spec.facet_count.clamp(3, 128) as usize;
    let unit: Vec<[f64; 2]> = match spec.shape {
        GemShape::Round | GemShape::Oval => (0..count)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / count as f64;
                [angle.cos(), angle.sin()]
            })
            .collect(),
        GemShape::Square | GemShape::Princess => {
            vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        }
        GemShape::Emerald | GemShape::Octagon | GemShape::Baguette => {
            let corner = if spec.corner_mm > 0.0 {
                spec.corner_mm / half_x.min(half_y)
            } else {
                spec.shape.default_corner_ratio()
            };
            rounded_rectangle(corner.clamp(0.0, 0.49))
        }
        GemShape::Cushion => rounded_corners(0.35, 5),
        GemShape::Pear => pear_outline(24),
        GemShape::Marquise => marquise_outline(24),
        GemShape::Trillion => trillion_outline(8),
        GemShape::Heart => heart_outline(32),
    };
    let (min_x, max_x, min_y, max_y) = profile_bounds(&unit);
    let span_x = (max_x - min_x).max(1.0e-12);
    let span_y = (max_y - min_y).max(1.0e-12);
    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    let mut outline: Vec<[f64; 2]> = unit
        .iter()
        .map(|p| {
            [
                (p[0] - center_x) / span_x * spec.length_mm,
                (p[1] - center_y) / span_y * spec.width_mm,
            ]
        })
        .collect();
    // The custom profile path returns before this point; every generated outline is
    // counter-clockwise so the table and the crown quads share one winding.
    if polygon_area(&outline) < 0.0 {
        outline.reverse();
    }
    Ok(outline)
}

fn rounded_rectangle(corner: f64) -> Vec<[f64; 2]> {
    if corner <= 0.0 {
        return vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    }
    let c = corner;
    vec![
        [-1.0, -1.0 + c],
        [-1.0 + c, -1.0],
        [1.0 - c, -1.0],
        [1.0, -1.0 + c],
        [1.0, 1.0 - c],
        [1.0 - c, 1.0],
        [-1.0 + c, 1.0],
        [-1.0, 1.0 - c],
    ]
}

fn rounded_corners(radius: f64, per_corner: usize) -> Vec<[f64; 2]> {
    // A stadium-ish rounded square sampled per corner.
    let radius = radius.clamp(0.0, 0.49);
    let mut points = Vec::new();
    for (cx, cy, start) in [
        (1.0 - radius, 1.0 - radius, 0.0),
        (1.0 - radius, -1.0 + radius, -std::f64::consts::FRAC_PI_2),
        (-1.0 + radius, -1.0 + radius, std::f64::consts::PI),
        (-1.0 + radius, 1.0 - radius, std::f64::consts::FRAC_PI_2),
    ] {
        for step in 0..per_corner {
            let angle = start
                + std::f64::consts::FRAC_PI_2 * step as f64 / (per_corner as f64 - 1.0).max(1.0);
            points.push([cx + radius * angle.cos(), cy + radius * angle.sin()]);
        }
    }
    points
}

fn pear_outline(count: usize) -> Vec<[f64; 2]> {
    let center = [0.0_f64, -0.5];
    let radius = 0.75_f64;
    let tip = [0.0_f64, 1.0];
    let distance = ((tip[0] - center[0]).powi(2) + (tip[1] - center[1]).powi(2)).sqrt();
    let offset = (radius / distance).acos();
    let base = std::f64::consts::FRAC_PI_2;
    let start = base + offset;
    let sweep = std::f64::consts::TAU - 2.0 * offset;
    let mut points = Vec::new();
    for index in 0..=count {
        let angle = start - sweep * index as f64 / count as f64;
        points.push([
            center[0] + radius * angle.cos(),
            center[1] + radius * angle.sin(),
        ]);
    }
    points.push(tip);
    points
}

fn marquise_outline(count: usize) -> Vec<[f64; 2]> {
    let width = 0.55;
    let center_y = (1.0 - width * width) / 2.0;
    let radius = 1.0 - center_y;
    let mut points = Vec::new();
    for index in 0..=count {
        let angle =
            std::f64::consts::FRAC_PI_2 - std::f64::consts::PI * index as f64 / count as f64;
        points.push([radius * angle.cos(), center_y + radius * angle.sin()]);
    }
    for index in 1..=count {
        let angle =
            -std::f64::consts::FRAC_PI_2 - std::f64::consts::PI * index as f64 / count as f64;
        points.push([radius * angle.cos(), center_y + radius * angle.sin()]);
    }
    points
}

fn trillion_outline(per_side: usize) -> Vec<[f64; 2]> {
    let corners = [
        [0.0, 1.0],
        [-(3.0_f64).sqrt() * 0.5, -0.5],
        [(3.0_f64).sqrt() * 0.5, -0.5],
    ];
    let mut points = Vec::new();
    for side in 0..3 {
        let from = corners[(side + 1) % 3];
        let to = corners[(side + 2) % 3];
        let center = corners[side];
        let start = (from[1] - center[1]).atan2(from[0] - center[0]);
        let end = (to[1] - center[1]).atan2(to[0] - center[0]);
        let radius = ((from[0] - center[0]).powi(2) + (from[1] - center[1]).powi(2)).sqrt();
        let mut delta = end - start;
        while delta > std::f64::consts::PI {
            delta -= std::f64::consts::TAU;
        }
        while delta < -std::f64::consts::PI {
            delta += std::f64::consts::TAU;
        }
        for step in 0..per_side {
            let angle = start + delta * step as f64 / per_side as f64;
            points.push([
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ]);
        }
    }
    points
}

fn heart_outline(count: usize) -> Vec<[f64; 2]> {
    (0..count)
        .map(|index| {
            let t = std::f64::consts::TAU * index as f64 / count as f64;
            let x = 16.0 * t.sin().powi(3);
            let y =
                13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
            [x, y]
        })
        .collect()
}

/// One placed gem instance: the spec plus a rigid local-to-parent transform.
#[derive(Debug, Clone, PartialEq)]
pub struct GemInstance {
    pub spec: GemSpec,
    pub placement: Transform,
    pub label: String,
}

impl GemInstance {
    /// Build a placement from a position, intrinsic X/Y/Z rotation (degrees) and a uniform
    /// scale. The rotation basis is normalised before it is stored.
    pub fn transform(position: Point3, rotation_deg: [f64; 3], scale: f64) -> Result<Transform> {
        if !position.iter().all(|v| v.is_finite()) || !scale.is_finite() || scale <= 0.0 {
            return Err(Error::Invalid("gem instance transform"));
        }
        let (sx, cx) = rotation_deg[0].to_radians().sin_cos();
        let (sy, cy) = rotation_deg[1].to_radians().sin_cos();
        let (sz, cz) = rotation_deg[2].to_radians().sin_cos();
        let rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
        let ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
        let rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
        let mut basis = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                basis[i][j] = (0..3).map(|k| rz[i][k] * ry[k][j]).sum();
            }
        }
        let mut composed = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                composed[i][j] = (0..3).map(|k| basis[i][k] * rx[k][j]).sum::<f64>() * scale;
            }
        }
        // Re-orthonormalise the axes so a long chain of edits cannot accumulate drift.
        let normalize = |row: [f64; 3]| -> Result<[f64; 3]> {
            let length = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
            if !length.is_finite() || length <= 0.0 {
                return Err(Error::Invalid("gem instance basis"));
            }
            Ok([row[0] / length, row[1] / length, row[2] / length])
        };
        let x_axis = normalize([composed[0][0], composed[1][0], composed[2][0]])?;
        let y_axis = normalize([composed[0][1], composed[1][1], composed[2][1]])?;
        let z_axis = normalize([composed[0][2], composed[1][2], composed[2][2]])?;
        let mut matrix = [
            [
                x_axis[0] * scale,
                y_axis[0] * scale,
                z_axis[0] * scale,
                position[0],
            ],
            [
                x_axis[1] * scale,
                y_axis[1] * scale,
                z_axis[1] * scale,
                position[1],
            ],
            [
                x_axis[2] * scale,
                y_axis[2] * scale,
                z_axis[2] * scale,
                position[2],
            ],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let transform = Transform(matrix);
        transform.validate()?;
        matrix[3] = [0.0, 0.0, 0.0, 1.0];
        Ok(Transform(matrix))
    }
    pub fn at(spec: GemSpec, position: Point3) -> Result<Self> {
        spec.validate()?;
        Ok(Self {
            label: format!("{} {}", spec.shape.label(), spec.cut.label()),
            spec,
            placement: Self::transform(position, [0.0, 0.0, 0.0], 1.0)?,
        })
    }
    pub fn with_rotation(mut self, rotation_deg: [f64; 3]) -> Result<Self> {
        let position = [
            self.placement.0[0][3],
            self.placement.0[1][3],
            self.placement.0[2][3],
        ];
        self.placement = Self::transform(position, rotation_deg, 1.0)?;
        Ok(self)
    }
    pub fn moved(mut self, delta: Point3) -> Result<Self> {
        for (axis, value) in delta.iter().enumerate() {
            self.placement.0[axis][3] += value;
        }
        self.placement.validate()?;
        Ok(self)
    }
    pub fn scaled(mut self, factor: f64) -> Result<Self> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(Error::Invalid("gem instance scale"));
        }
        let length = self.spec.length_mm * factor;
        let width = self.spec.width_mm * factor;
        self.spec = self.spec.set_girdle_mm(length, width)?;
        let depth = self.spec.depth_mm() * factor;
        self.spec = self.spec.set_depth_mm(depth)?;
        build_mesh(&self.spec)?;
        Ok(self)
    }
    /// The normalised orientation basis columns (X, Y, Z).
    pub fn basis(&self) -> [[f64; 3]; 3] {
        let m = self.placement.0;
        [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ]
    }
}

/// The typed `om9.gem` payload. `asset`, `material`, `color` and `carat` stay top-level so
/// the display cue and the measurement report keep reading the same fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GemRecord {
    pub version: u32,
    /// Stable gem identity. Older payloads without the field get a fresh id on read.
    #[serde(default)]
    pub gem_id: GemId,
    pub asset: String,
    pub material: String,
    pub color: [u8; 4],
    #[serde(default)]
    pub carat: Option<f64>,
    pub spec: GemSpec,
}

impl GemRecord {
    pub fn new(spec: GemSpec) -> Self {
        Self {
            version: GEM_PAYLOAD_VERSION,
            gem_id: GemId::new(),
            asset: spec.asset_key(),
            material: DEFAULT_MATERIAL.to_owned(),
            color: DEFAULT_COLOR,
            carat: None,
            spec,
        }
    }
    /// Bind the record to an explicit identity (the instance the document already owns).
    pub fn with_id(mut self, gem_id: GemId) -> Self {
        self.gem_id = gem_id;
        self
    }
    pub fn with_material(mut self, asset: &str, material: &str, color: [u8; 4]) -> Self {
        self.asset = asset.to_owned();
        self.material = material.to_owned();
        self.color = color;
        self
    }
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| Error::Invalid("gem payload json"))
    }
    pub fn from_json(text: &str) -> Result<Self> {
        let record: GemRecord =
            serde_json::from_str(text).map_err(|_| Error::Invalid("gem payload schema"))?;
        if record.version != GEM_PAYLOAD_VERSION {
            return Err(Error::Invalid("gem payload version"));
        }
        record.spec.validate()?;
        if let Some(carat) = record.carat
            && (!carat.is_finite() || carat < 0.0)
        {
            return Err(Error::Invalid("gem carat"));
        }
        Ok(record)
    }
    pub fn as_extension(&self) -> Result<Extension> {
        Extension::new(self.version, self.to_json()?)
    }
    /// The mesh-owning document entity for this gem. The payload's `GemId` is the
    /// entity's own document identity, so a saved/reopened gem keeps the same id.
    pub fn entity(&self, layer: Option<crate::identity::LayerId>) -> Result<Entity> {
        let mesh = build_mesh(&self.spec)?;
        let mut entity = Entity::new(
            format!("{} {}", self.spec.shape.label(), self.spec.cut.label()),
            Geometry::Mesh(mesh.mesh),
            layer,
        );
        let mut record = self.clone();
        record.gem_id = GemId(entity.id);
        entity
            .extensions
            .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
        Ok(entity)
    }
}

/// The typed gem record of an entity, if it carries one.
pub fn gem_record(entity: &Entity) -> Option<GemRecord> {
    let extension = entity.extension(GEM_EXTENSION_SLOT)?;
    GemRecord::from_json(&extension.payload).ok()
}

/// Find the gem record of an entity, or of the single member of an instance definition.
pub fn gem_record_in(document: &Document, id: ObjectId) -> Option<GemRecord> {
    let entity = document.entities.get(&id)?;
    if let Some(record) = gem_record(entity) {
        return Some(record);
    }
    if let Geometry::Instance(definition) = entity.geometry
        && let Some(definition) = document.definitions.get(&definition)
    {
        for member in &definition.members {
            if let Some(record) = gem_record(member) {
                return Some(record);
            }
        }
    }
    None
}

/// Build the mesh entity of a spec, ready for `Mutation::Insert`.
pub fn gem_entity(
    label: &str,
    spec: &GemSpec,
    layer: Option<crate::identity::LayerId>,
) -> Result<Entity> {
    spec.validate()?;
    let mut entity = GemRecord::new(spec.clone()).entity(layer)?;
    entity.label = label.to_owned();
    Ok(entity)
}

/// How many placed instances reference one definition.
pub fn instance_count(document: &Document, definition: crate::identity::DefinitionId) -> usize {
    document
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::Instance(id) if id == definition))
        .count()
}

/// A short human report of one gem, used by the property editor and evidence logs.
pub fn summary(record: &GemRecord) -> String {
    format!(
        "{} {} {} · {:.2}×{:.2}×{:.2} mm · {:.2} mm3",
        record.spec.shape.label(),
        record.spec.cut.label(),
        record.asset,
        record.spec.length_mm,
        record.spec.width_mm,
        record.spec.depth_mm(),
        build_mesh(&record.spec)
            .map(|m| m.volume_mm3)
            .unwrap_or(0.0)
    )
}
