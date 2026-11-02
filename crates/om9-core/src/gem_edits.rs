// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem instance edits and reports (`OM9-L2-04-13`).
//!
//! This owner turns typed attribute changes into new gem records, copies attributes between
//! stones, keeps a versioned gem-style store, aggregates count/weight reports, builds the
//! gem-map rows, and proves that an edit never renumbers the stone.
//!
//! Everything here is pure data: nothing mutates a document. The `om9-app` session applies the
//! returned records in ONE transaction, so Undo restores exactly the previous state and the
//! `om9.gem.placement` link is preserved because the caller keeps the entity extension it did
//! not rewrite.
//!
//! Refusals are typed. A size that would break the cut/shape, a style with an invalid name, a
//! report with a non-positive density or a map point that is not finite are all
//! `Error::Invalid`, never a clamped or silently dropped value.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::Result;
use crate::builder_style::validate_style_name;
use crate::gem::{GemCut, GemId, GemRecord, GemShape, GemSpec};
use crate::geometry::Point3;

/// The schema of one attribute edit (`om9.gem_edit.v1`).
pub const GEM_EDIT_SCHEMA: &str = "om9.gem_edit.v1";
/// The schema of one saved gem style (`om9.gem_style.v1`).
pub const GEM_STYLE_SCHEMA: &str = "om9.gem_style.v1";
/// The schema of the gem-style file that holds several styles.
pub const GEM_STYLE_FILE_SCHEMA: &str = "om9.gem_style_file.v1";
/// The style file format version.
pub const GEM_STYLE_VERSION: u32 = 1;
/// Upper bound of styles in one store (the shared builder store uses the same bound).
pub const MAX_GEM_STYLES: usize = 512;
/// Upper bound of report rows before the caller must split the scene.
pub const MAX_REPORT_ROWS: usize = 256;
/// Diamond density in grams per cubic centimetre, the reporter's default basis.
pub const DIAMOND_DENSITY_G_CM3: f64 = 3.52;
/// One carat in grams.
pub const CARAT_GRAMS: f64 = 0.2;
/// The accepted density range (grams per cubic centimetre).
pub const MIN_DENSITY_G_CM3: f64 = 0.1;
pub const MAX_DENSITY_G_CM3: f64 = 30.0;

/* ---------------------------------- attribute edit -------------------------------- */

/// A partial gem attribute edit. `None` means "leave this attribute alone", so a caller never
/// has to snapshot the whole record to change one field.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields, default)]
pub struct GemEdit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cut: Option<GemCut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<GemShape>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_mm: Option<f64>,
    /// Table-to-culet depth; the crown/pavilion split keeps its old proportion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_mm: Option<f64>,
    /// Girdle band thickness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub girdle_mm: Option<f64>,
    /// Corner cut of the emerald/octagon silhouette.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_mm: Option<f64>,
    /// Material name (`diamond`, `ruby`, …); validated by the caller's colour table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    /// Display colour; a style may carry it, a geometry edit may not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 4]>,
}

impl GemEdit {
    /// No field set: a valid no-op.
    pub fn is_empty(&self) -> bool {
        self.cut.is_none()
            && self.shape.is_none()
            && self.length_mm.is_none()
            && self.width_mm.is_none()
            && self.depth_mm.is_none()
            && self.girdle_mm.is_none()
            && self.corner_mm.is_none()
            && self.material.is_none()
            && self.color.is_none()
    }

    /// Whether the edit can change the mesh, so the caller knows to rebuild the geometry.
    pub fn changes_geometry(&self) -> bool {
        self.cut.is_some()
            || self.shape.is_some()
            || self.length_mm.is_some()
            || self.width_mm.is_some()
            || self.depth_mm.is_some()
            || self.girdle_mm.is_some()
            || self.corner_mm.is_some()
    }

    /// Validate the edit on its own: finite, positive sizes and a usable material name.
    pub fn validate(&self) -> Result<()> {
        for value in [
            self.length_mm,
            self.width_mm,
            self.depth_mm,
            self.girdle_mm,
        ]
        .into_iter()
        .flatten()
        {
            if !value.is_finite() || value <= 0.0 {
                return Err(Error::Invalid("gem edit size"));
            }
        }
        if let Some(corner) = self.corner_mm {
            // A zero corner is meaningful: it removes the corner cut.
            if !corner.is_finite() || corner < 0.0 {
                return Err(Error::Invalid("gem edit size"));
            }
        }
        if let Some(material) = &self.material {
            let material = material.trim();
            if material.is_empty() || material.len() > 64 {
                return Err(Error::Invalid("gem edit material"));
            }
        }
        Ok(())
    }

    /// Apply the edit to a spec. A cut change that would strand the shape, a round stone with
    /// two diameters and a depth below the girdle are all refused.
    pub fn spec(&self, base: &GemSpec) -> Result<GemSpec> {
        self.validate()?;
        let mut spec = base.clone();
        if let Some(cut) = self.cut {
            spec.cut = cut;
        }
        if let Some(shape) = self.shape {
            spec.shape = shape;
        }
        if !spec.cut.shapes().contains(&spec.shape) {
            return Err(Error::Invalid("gem shape for cut"));
        }
        if self.length_mm.is_some() || self.width_mm.is_some() {
            let length = self.length_mm.unwrap_or(spec.length_mm);
            let width = self.width_mm.unwrap_or(spec.width_mm);
            spec = spec.set_girdle_mm(length, width)?;
        }
        if let Some(depth) = self.depth_mm {
            spec = spec.set_depth_mm(depth)?;
        }
        if let Some(girdle) = self.girdle_mm {
            spec = spec.set_girdle_mm_thickness(girdle)?;
        }
        if let Some(corner) = self.corner_mm {
            spec = spec.set_corner_mm(corner)?;
        }
        spec.validate()?;
        Ok(spec)
    }

    /// Apply the edit to a record, keeping the gem identity and the asset key consistent.
    pub fn record(&self, base: &GemRecord) -> Result<GemRecord> {
        let spec = self.spec(&base.spec)?;
        let mut record = base.clone();
        record.spec = spec;
        record.asset = record.spec.asset_key();
        if let Some(material) = &self.material {
            record.material = material.trim().to_owned();
        }
        if let Some(color) = self.color {
            record.color = color;
        }
        Ok(record)
    }
}

/* ---------------------------------- match attributes ------------------------------ */

/// Which attribute groups Match Attributes copies from the source stone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MatchMask {
    pub cut: bool,
    pub shape: bool,
    pub dimensions: bool,
    /// Material and colour.
    pub finish: bool,
}

impl MatchMask {
    pub const ALL: Self = Self {
        cut: true,
        shape: true,
        dimensions: true,
        finish: true,
    };
    pub const GEOMETRY: Self = Self {
        cut: true,
        shape: true,
        dimensions: true,
        finish: false,
    };
    pub const FINISH: Self = Self {
        cut: false,
        shape: false,
        dimensions: false,
        finish: true,
    };
    pub const NONE: Self = Self {
        cut: false,
        shape: false,
        dimensions: false,
        finish: false,
    };

    pub fn is_empty(self) -> bool {
        !self.cut && !self.shape && !self.dimensions && !self.finish
    }

    /// Parse the prompt's mask: `all`, `geometry`, `finish`, or a comma list of the four names.
    pub fn parse(text: &str) -> Result<Self> {
        let mut mask = Self::NONE;
        for part in text.split([',', ' ', '+']).filter(|part| !part.trim().is_empty()) {
            match part.trim().to_ascii_lowercase().as_str() {
                "all" => mask = Self::ALL,
                "geometry" | "shape" => {
                    mask.cut = true;
                    mask.shape = true;
                    mask.dimensions = true;
                }
                "finish" | "material" => {
                    mask.finish = true;
                    mask.cut = false;
                    mask.shape = false;
                    mask.dimensions = false;
                }
                "cut" => mask.cut = true,
                "dimensions" | "size" => mask.dimensions = true,
                "color" | "colour" => mask.finish = true,
                _ => return Err(Error::Invalid("gem match mask")),
            }
        }
        if mask.is_empty() {
            return Err(Error::Invalid("gem match mask"));
        }
        Ok(mask)
    }

    pub fn label(self) -> &'static str {
        match (self.cut, self.shape, self.dimensions, self.finish) {
            (true, true, true, true) => "all",
            (true, true, true, false) => "geometry",
            (false, false, false, true) => "finish",
            _ => "custom",
        }
    }
}

/// The edit Match Attributes derives from a source record.
pub fn match_edit(source: &GemRecord, mask: MatchMask) -> GemEdit {
    let mut edit = GemEdit::default();
    if mask.cut {
        edit.cut = Some(source.spec.cut);
    }
    if mask.shape {
        edit.shape = Some(source.spec.shape);
    }
    if mask.dimensions {
        edit.length_mm = Some(source.spec.length_mm);
        edit.width_mm = Some(source.spec.width_mm);
        edit.depth_mm = Some(source.spec.depth_mm());
        edit.girdle_mm = Some(source.spec.girdle_mm);
        edit.corner_mm = Some(source.spec.corner_mm);
    }
    if mask.finish {
        edit.material = Some(source.material.clone());
        edit.color = Some(source.color);
    }
    edit
}

/// Copy the masked attributes from `source` onto every target, keeping every target identity.
pub fn match_attributes(
    source: &GemRecord,
    targets: &[GemRecord],
    mask: MatchMask,
) -> Result<Vec<GemRecord>> {
    if mask.is_empty() {
        return Err(Error::Invalid("gem match mask"));
    }
    let edit = match_edit(source, mask);
    let mut matched = Vec::new();
    matched
        .try_reserve_exact(targets.len())
        .map_err(|_| Error::Budget("gem match targets"))?;
    for target in targets {
        matched.push(edit.record(target)?);
    }
    Ok(matched)
}

/* ------------------------------------- gem styles --------------------------------- */

/// One named, versioned snapshot of gem attributes — the gem owner's preset record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct GemStyle {
    pub schema: String,
    pub version: u32,
    pub name: String,
    pub edit: GemEdit,
}

impl GemStyle {
    /// Snapshot every attribute of one stone. The gem identity, label and placement stay with
    /// the instance, not the style.
    pub fn from_record(name: impl Into<String>, record: &GemRecord) -> Self {
        Self {
            schema: GEM_STYLE_SCHEMA.to_owned(),
            version: GEM_STYLE_VERSION,
            name: name.into(),
            edit: GemEdit {
                cut: Some(record.spec.cut),
                shape: Some(record.spec.shape),
                length_mm: Some(record.spec.length_mm),
                width_mm: Some(record.spec.width_mm),
                depth_mm: Some(record.spec.depth_mm()),
                girdle_mm: Some(record.spec.girdle_mm),
                corner_mm: Some(record.spec.corner_mm),
                material: Some(record.material.clone()),
                color: Some(record.color),
            },
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema != GEM_STYLE_SCHEMA {
            return Err(Error::Invalid("gem style schema"));
        }
        if self.version != GEM_STYLE_VERSION {
            return Err(Error::Invalid("gem style version"));
        }
        validate_style_name(&self.name)?;
        if self.edit.is_empty() {
            return Err(Error::Invalid("gem style attributes"));
        }
        self.edit.validate()?;
        Ok(())
    }

    /// Apply the style to a record; the gem id is untouched.
    pub fn apply(&self, record: &GemRecord) -> Result<GemRecord> {
        self.validate()?;
        self.edit.record(record)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| Error::Invalid("gem style json"))
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let style: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("gem style json"))?;
        style.validate()?;
        Ok(style)
    }
}

/// The persisted store of gem styles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct GemStyleStore {
    pub schema: String,
    pub version: u32,
    pub styles: Vec<GemStyle>,
}

impl Default for GemStyleStore {
    fn default() -> Self {
        Self {
            schema: GEM_STYLE_FILE_SCHEMA.to_owned(),
            version: GEM_STYLE_VERSION,
            styles: Vec::new(),
        }
    }
}

impl GemStyleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.styles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.styles
            .iter()
            .map(|style| style.name.as_str())
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&GemStyle> {
        self.styles
            .iter()
            .find(|style| style.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Insert or replace by name. Returns whether an old style was replaced.
    pub fn insert(&mut self, style: GemStyle) -> Result<bool> {
        style.validate()?;
        if let Some(slot) = self
            .styles
            .iter_mut()
            .find(|slot| slot.name.eq_ignore_ascii_case(&style.name))
        {
            *slot = style;
            return Ok(true);
        }
        if self.styles.len() >= MAX_GEM_STYLES {
            return Err(Error::Budget("gem styles"));
        }
        self.styles.push(style);
        Ok(false)
    }

    /// Remove by name. Returns whether a style was removed.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.styles.len();
        self.styles
            .retain(|style| !style.name.eq_ignore_ascii_case(name.trim()));
        before != self.styles.len()
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema != GEM_STYLE_FILE_SCHEMA {
            return Err(Error::Invalid("gem style file schema"));
        }
        if self.version != GEM_STYLE_VERSION {
            return Err(Error::Invalid("gem style file version"));
        }
        if self.styles.len() > MAX_GEM_STYLES {
            return Err(Error::Budget("gem styles"));
        }
        let mut seen = BTreeMap::new();
        for style in &self.styles {
            style.validate()?;
            if seen.insert(style.name.to_ascii_lowercase(), ()).is_some() {
                return Err(Error::Invalid("gem style name duplicated"));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|_| Error::Invalid("gem style json"))
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let store: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("gem style json"))?;
        store.validate()?;
        Ok(store)
    }

    /// Read the store; a missing file is the documented empty store, never an error.
    pub fn load_or_empty(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::from_json(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(_) => Err(Error::Invalid("gem style file read")),
        }
    }

    /// Write the store atomically: a staging file in the same directory, then a rename (a copy
    /// fallback keeps the previous file replaceable on Windows).
    pub fn save_to(&self, path: &Path) -> Result<()> {
        let text = self.to_json()?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|_| Error::Invalid("gem style directory"))?;
        }
        let name = path
            .file_name()
            .ok_or(Error::Invalid("gem style filename"))?
            .to_string_lossy()
            .into_owned();
        let staging = parent.join(format!(".{name}.tmp"));
        {
            let mut file = std::fs::File::create(&staging)
                .map_err(|_| Error::Invalid("gem style file create"))?;
            file.write_all(text.as_bytes())
                .map_err(|_| Error::Invalid("gem style file write"))?;
            file.sync_all()
                .map_err(|_| Error::Invalid("gem style file sync"))?;
        }
        match std::fs::rename(&staging, path) {
            Ok(()) => Ok(()),
            Err(_) => {
                // Windows refuses a rename over an existing file; a copy keeps the old file
                // replaceable without ever deleting it first.
                let copied = std::fs::copy(&staging, path);
                let _ = std::fs::remove_file(&staging);
                copied
                    .map(|_| ())
                    .map_err(|_| Error::Invalid("gem style file publish"))
            }
        }
    }
}

/* ------------------------------------- weight report ------------------------------ */

/// Densities of the jewellery materials the reporter knows, in grams per cubic centimetre.
///
/// These are the standard gemological reference densities (for example diamond 3.52,
/// corundum 4.00, beryl 2.72). A material that is not listed is reported explicitly as an
/// assumption instead of silently using the diamond basis.
pub const MATERIAL_DENSITIES: &[(&str, f64)] = &[
    ("diamond", 3.52),
    ("moissanite", 3.22),
    ("cubic zirconia", 5.65),
    ("cz", 5.65),
    ("ruby", 4.00),
    ("sapphire", 4.00),
    ("corundum", 4.00),
    ("emerald", 2.72),
    ("aquamarine", 2.70),
    ("morganite", 2.80),
    ("topaz", 3.53),
    ("amethyst", 2.65),
    ("citrine", 2.65),
    ("quartz", 2.65),
    ("garnet", 3.60),
    ("rhodolite", 3.80),
    ("tsavorite", 3.60),
    ("peridot", 3.34),
    ("tourmaline", 3.06),
    ("zircon", 4.65),
    ("spinel", 3.60),
    ("tanzanite", 3.35),
    ("kunzite", 3.18),
    ("iolite", 2.65),
    ("diopside", 3.30),
    ("lapis", 2.75),
    ("turquoise", 2.75),
    ("jade", 3.30),
    ("malachite", 3.90),
    ("opal", 2.10),
    ("pearl", 2.71),
    ("coral", 2.65),
    ("onyx", 2.65),
    ("glass", 2.50),
];

/// The reference density of one material name, case-insensitive.
pub fn material_density(name: &str) -> Option<f64> {
    let name = name.trim().to_ascii_lowercase();
    MATERIAL_DENSITIES
        .iter()
        .find(|(material, _)| *material == name)
        .map(|(_, density)| *density)
}

/// The tolerance, in percent, inside which the mesh volume and the trade weight formula must
/// agree. The trade formula is itself an approximation, so a small spread is expected; a stone
/// outside this band is reported instead of presented as a trustworthy weight.
pub const REPORT_TOLERANCE_PERCENT: f64 = 15.0;

/// A trade weight formula for one shape: `carat ≈ L × W × depth × factor` (millimetres).
///
/// The factors are the published diamond trade estimates per shape. They are a cross-check on
/// the mesh volume, never the primary mass: the mesh is the authored solid and the formula is
/// an independent gemological sanity bound.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeightFormula {
    /// The trade name of the formula (`round brilliant`, `oval`, …).
    pub name: &'static str,
    pub factor: f64,
    /// The formula result in carats for this spec.
    pub carat: f64,
}

/// The trade formula of a spec, or `None` when the shape has no trade factor (a cabochon dome or
/// a custom profile is not a faceted outline, so the faceted trade formula would be wrong).
///
/// The result is a **diamond-basis** carat: the published factors are calibrated for diamond's
/// 3.52 g/cm³. The reporter scales it by the row's material density before comparing, because a
/// ruby of the same girdle is heavier by the density ratio, not by a different trade factor.
pub fn weight_formula(spec: &GemSpec) -> Option<WeightFormula> {
    if spec.cut == GemCut::Cabochon || spec.profile.is_some() {
        return None;
    }
    let (name, factor) = match spec.shape {
        GemShape::Round => ("round brilliant", 0.0061),
        GemShape::Oval => ("oval", 0.0062),
        GemShape::Marquise => ("marquise", 0.00565),
        GemShape::Pear => ("pear", 0.0058),
        GemShape::Heart => ("heart", 0.0059),
        GemShape::Trillion => ("trillion", 0.0057),
        GemShape::Emerald | GemShape::Octagon => ("step cut", 0.0080),
        GemShape::Baguette => ("baguette", 0.0085),
        GemShape::Square | GemShape::Princess | GemShape::Cushion => ("square", 0.0080),
    };
    let carat = spec.length_mm * spec.width_mm * spec.depth_mm() * factor;
    if !carat.is_finite() || carat <= 0.0 {
        return None;
    }
    Some(WeightFormula {
        name,
        factor,
        carat,
    })
}

/// The mass of one stone, derived from the closed procedural mesh.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GemWeight {
    pub volume_mm3: f64,
    pub grams: f64,
    pub carat: f64,
    pub density_g_cm3: f64,
}

/// Validate the reporter's density basis.
pub fn validate_density(density_g_cm3: f64) -> Result<()> {
    if !density_g_cm3.is_finite()
        || !(MIN_DENSITY_G_CM3..=MAX_DENSITY_G_CM3).contains(&density_g_cm3)
    {
        return Err(Error::Invalid("gem density"));
    }
    Ok(())
}

/// Weight of a validated spec: mesh volume × density. One carat is 0.2 g.
pub fn gem_weight(spec: &GemSpec, density_g_cm3: f64) -> Result<GemWeight> {
    validate_density(density_g_cm3)?;
    let mesh = crate::gem::build_mesh(spec)?;
    if !mesh.closed || !mesh.volume_mm3.is_finite() || mesh.volume_mm3 <= 0.0 {
        return Err(Error::Invalid("gem mesh volume"));
    }
    Ok(GemWeight {
        volume_mm3: mesh.volume_mm3,
        // 1 cm³ = 1000 mm³.
        grams: mesh.volume_mm3 * density_g_cm3 / 1000.0,
        carat: mesh.volume_mm3 * density_g_cm3 / 1000.0 / CARAT_GRAMS,
        density_g_cm3,
    })
}

/// One aggregated row of the reporter: same shape, cut, girdle and material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GemReportRow {
    pub shape: GemShape,
    pub cut: GemCut,
    pub length_mm: f64,
    pub width_mm: f64,
    pub material: String,
    pub density_g_cm3: f64,
    pub count: usize,
    /// Mass from the closed mesh volume and the material density: the primary value.
    pub carat: f64,
    pub grams: f64,
    /// The trade-formula cross-check, when the shape has a published factor.
    pub carat_formula: Option<f64>,
    /// `(carat − carat_formula) / carat_formula × 100`; positive means the mesh is heavier.
    pub delta_percent: Option<f64>,
}

/// The full count/weight report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GemReport {
    pub rows: Vec<GemReportRow>,
    pub total_count: usize,
    pub total_carat: f64,
    pub total_grams: f64,
    /// The summed trade-formula cross-check, when at least one row has a formula.
    pub total_carat_formula: Option<f64>,
    /// The `Density=` override, when the operator chose one instead of the material table.
    pub density_override: Option<f64>,
    /// Stones whose material is unknown and therefore use the diamond basis.
    pub assumed_density: usize,
    /// The largest absolute delta between the mesh mass and the trade formula.
    pub worst_delta_percent: Option<f64>,
    /// Rows whose cross-check falls outside [`REPORT_TOLERANCE_PERCENT`].
    pub outside_tolerance: usize,
    pub tolerance_percent: f64,
    /// How the mass was computed; the report never presents itself as a certificate.
    pub basis: &'static str,
}

/// Aggregate stones by (shape, cut, girdle, material) and total their mass. Rows are sorted by
/// shape, cut and size so a printed report is stable. Every record contributes exactly one
/// stone; the density comes from the material table unless the caller overrides it, and every
/// row also carries the trade-formula cross-check.
pub fn gem_report(records: &[GemRecord], density_override: Option<f64>) -> Result<GemReport> {
    if let Some(density) = density_override {
        validate_density(density)?;
    }
    let mut groups: BTreeMap<(String, String, u64, u64, String, u64), GemReportRow> =
        BTreeMap::new();
    let mut assumed_density = 0;
    for record in records {
        let spec = &record.spec;
        spec.validate()?;
        let material = record.material.trim().to_ascii_lowercase();
        let density = match (density_override, material_density(&material)) {
            (Some(density), _) => density,
            (None, Some(density)) => density,
            (None, None) => {
                assumed_density += 1;
                DIAMOND_DENSITY_G_CM3
            }
        };
        let weight = gem_weight(spec, density)?;
        let formula = weight_formula(spec);
        let key = (
            spec.shape.tag().to_owned(),
            spec.cut.tag().to_owned(),
            (spec.length_mm * 1.0e6).round() as u64,
            (spec.width_mm * 1.0e6).round() as u64,
            material.clone(),
            density.to_bits(),
        );
        let row = groups.entry(key).or_insert_with(|| GemReportRow {
            shape: spec.shape,
            cut: spec.cut,
            length_mm: spec.length_mm,
            width_mm: spec.width_mm,
            material: material.clone(),
            density_g_cm3: density,
            count: 0,
            carat: 0.0,
            grams: 0.0,
            carat_formula: None,
            delta_percent: None,
        });
        row.count += 1;
        row.carat += weight.carat;
        row.grams += weight.grams;
        if let Some(formula) = formula {
            // The factor is a diamond-basis estimate; scale it to the row's material density.
            let scaled = formula.carat * density / DIAMOND_DENSITY_G_CM3;
            row.carat_formula = Some(row.carat_formula.unwrap_or(0.0) + scaled);
        }
    }
    if groups.len() > MAX_REPORT_ROWS {
        return Err(Error::Budget("gem report rows"));
    }
    let mut rows: Vec<GemReportRow> = groups.into_values().collect();
    for row in &mut rows {
        if let Some(formula) = row.carat_formula {
            if formula > 0.0 {
                row.delta_percent = Some((row.carat - formula) / formula * 100.0);
            } else {
                row.carat_formula = None;
            }
        }
    }
    let total_count = rows.iter().map(|row| row.count).sum();
    let total_carat = rows.iter().map(|row| row.carat).sum();
    let total_grams = rows.iter().map(|row| row.grams).sum();
    let total_carat_formula = {
        let formula: f64 = rows.iter().filter_map(|row| row.carat_formula).sum();
        (formula > 0.0).then_some(formula)
    };
    let worst_delta_percent = rows
        .iter()
        .filter_map(|row| row.delta_percent)
        .map(f64::abs)
        .reduce(f64::max);
    let outside_tolerance = rows
        .iter()
        .filter(|row| row.delta_percent.is_some_and(|delta| delta.abs() > REPORT_TOLERANCE_PERCENT))
        .count();
    Ok(GemReport {
        rows,
        total_count,
        total_carat,
        total_grams,
        total_carat_formula,
        density_override,
        assumed_density,
        worst_delta_percent,
        outside_tolerance,
        tolerance_percent: REPORT_TOLERANCE_PERCENT,
        basis: "mesh volume × material density, cross-checked against the trade weight formula",
    })
}

/* --------------------------------------- gem map --------------------------------- */

/// One numbered entry of the gem map: the stone's size, shape and where it sits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GemMapEntry {
    pub gem: GemId,
    pub number: usize,
    pub shape: GemShape,
    pub cut: GemCut,
    pub length_mm: f64,
    pub width_mm: f64,
    pub point: Point3,
}

/// Build the map rows in the given scene order. Numbers start at one and never skip, and a
/// point that is not finite is refused rather than printed.
pub fn gem_map(entries: &[(GemId, Point3, GemSpec)]) -> Result<Vec<GemMapEntry>> {
    if entries.len() > MAX_REPORT_ROWS {
        return Err(Error::Budget("gem map rows"));
    }
    let mut rows = Vec::new();
    rows.try_reserve_exact(entries.len())
        .map_err(|_| Error::Budget("gem map rows"))?;
    for (index, (gem, point, spec)) in entries.iter().enumerate() {
        spec.validate()?;
        if !point.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("gem map point"));
        }
        rows.push(GemMapEntry {
            gem: *gem,
            number: index + 1,
            shape: spec.shape,
            cut: spec.cut,
            length_mm: spec.length_mm,
            width_mm: spec.width_mm,
            point: *point,
        });
    }
    Ok(rows)
}

/// A stable, human-readable line for one map row (`#3 · Round Cut · 6.00 × 6.00 mm @ (0, 0, 5)`).
pub fn map_line(entry: &GemMapEntry) -> String {
    format!(
        "#{} · {} {} · {:.2} × {:.2} mm @ ({:.3}, {:.3}, {:.3})",
        entry.number,
        entry.shape.label(),
        entry.cut.label(),
        entry.length_mm,
        entry.width_mm,
        entry.point[0],
        entry.point[1],
        entry.point[2]
    )
}

/// A stable, human-readable line for one report row, including the material density and the
/// trade-formula cross-check so the printed weight is auditable.
pub fn report_line(row: &GemReportRow) -> String {
    let mut line = format!(
        "{} {} · {:.2} × {:.2} mm · {} · {} stone(s) · {:.3} ct · {:.3} g · {:.2} g/cm³",
        row.shape.label(),
        row.cut.label(),
        row.length_mm,
        row.width_mm,
        row.material,
        row.count,
        row.carat,
        row.grams,
        row.density_g_cm3
    );
    if let Some(delta) = row.delta_percent {
        line.push_str(&format!(" · Δ {delta:+.1}% vs trade formula"));
    }
    line
}
