// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder styles: versioned parameter snapshots for one builder mode
//! (`OM9-L2-03-12`).
//!
//! A style is a *declarative record*, not a document object: name, family tags, optional
//! builder tags, typed parameters and an optional thumbnail resource. Applying a style
//! answers with the fields it set; it never writes the document, so the preview/apply split
//! of `OM9-L3-03-12-003` stays with the builder session and the one Apply transaction.
//!
//! Rules carried by this module:
//!
//! * `OM9-L3-03-12-001` — `om9.builder_style.v1` roundtrip preserves parameter values, their
//!   unit (`length_mm`) and their type; an unknown schema or an invalid numeric is refused.
//! * `OM9-L3-03-12-002` — the library lists entries for the active mode with a thumbnail
//!   descriptor; a corrupt/missing thumbnail falls back to a placeholder instead of failing.
//! * `OM9-L3-03-12-003` — a style missing a mandatory input of the mode is refused before it
//!   can touch a preview.
//! * `OM9-L3-03-12-004` — user styles are a separate list from the built-ins; names are
//!   validated and unique, and the built-in list is never rewritten.
//! * `OM9-L3-03-12-005` — an entry is only enabled for a compatible mode/family; an
//!   incompatible style can be listed but not applied.
//! * `OM9-L3-03-12-006` — declared compatible attributes are copied from a source entity's
//!   extension slots into the style; entity identity is never rewritten.
use crate::builder::{BuilderKind, BuilderSession};
use crate::error::{Error, Result};
use crate::model::Entity;
use crate::request::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The schema tag of a style file. A different tag is refused, never guessed.
pub const STYLE_SCHEMA: &str = "om9.builder_style.v1";
/// Longest accepted style name, in bytes.
pub const MAX_STYLE_NAME_BYTES: usize = 64;
/// Largest library a single file may carry.
pub const MAX_STYLE_COUNT: usize = 512;
/// Longest accepted thumbnail resource key, in bytes.
pub const MAX_THUMBNAIL_BYTES: usize = 256;

/// The design family a style belongs to. Families decide which modes may offer the style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleFamily {
    /// A plain ring band.
    Band,
    /// A signet ring (flat or shaped head).
    Signet,
    /// A gem/stone builder style.
    Gem,
    /// A setting style around a gem.
    Setting,
}

impl StyleFamily {
    pub const ALL: [Self; 4] = [Self::Band, Self::Signet, Self::Gem, Self::Setting];

    pub fn name(self) -> &'static str {
        match self {
            Self::Band => "band",
            Self::Signet => "signet",
            Self::Gem => "gem",
            Self::Setting => "setting",
        }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|family| family.name().eq_ignore_ascii_case(name.trim()))
    }
    /// The family domain a builder mode belongs to. A ring-rail builder offers band, signet
    /// and setting styles; a gem style is offered by gem builders only.
    pub fn matches(self, kind: BuilderKind) -> bool {
        matches!(self, Self::Band | Self::Signet | Self::Setting) && kind.is_ring_builder()
    }
}

/// One typed parameter of a style. The variant carries the unit/type, so a roundtrip cannot
/// turn a length into a bare number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleValue {
    /// A length in millimetres, stored with its unit in the JSON (`{"length_mm": 2.2}`).
    LengthMm(f64),
    /// A dimensionless number.
    Number(f64),
    /// An angle in degrees, stored with its unit in the JSON (`{"angle_deg": 15.0}`).
    AngleDeg(f64),
    /// A named choice (a profile preset name, a solid mode, …).
    Choice(String),
    /// A flag.
    Boolean(bool),
}

impl StyleValue {
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::LengthMm(value) | Self::Number(value) => Some(*value),
            _ => None,
        }
    }
    /// Refuse a value that could never be a builder parameter: a non-finite number, a
    /// negative length, an empty choice.
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LengthMm(value) => {
                if !value.is_finite() || *value < 0.0 {
                    return Err(Error::Invalid("style numeric"));
                }
            }
            Self::Number(value) | Self::AngleDeg(value) => {
                if !value.is_finite() {
                    return Err(Error::Invalid("style numeric"));
                }
            }
            Self::Choice(name) => {
                if name.trim().is_empty() || name.len() > MAX_STYLE_NAME_BYTES {
                    return Err(Error::Invalid("style choice"));
                }
            }
            Self::Boolean(_) => {}
        }
        Ok(())
    }
    /// The request value the builder session understands.
    pub fn to_request(&self) -> Value {
        match self {
            Self::LengthMm(value) => Value::Quantity {
                value: *value,
                dimension: crate::request::Dimension::Length,
            },
            Self::AngleDeg(value) => Value::Quantity {
                value: *value,
                dimension: crate::request::Dimension::Angle,
            },
            Self::Number(value) => Value::Number(*value),
            Self::Choice(name) => Value::Choice(name.clone()),
            Self::Boolean(flag) => Value::Boolean(*flag),
        }
    }
    /// The style value for a request value, so a snapshot keeps the type the builder used.
    pub fn from_request(value: &Value) -> Option<Self> {
        match value {
            Value::Quantity {
                value,
                dimension: crate::request::Dimension::Length,
            } => Some(Self::LengthMm(*value)),
            Value::Quantity {
                value,
                dimension: crate::request::Dimension::Angle,
            } => Some(Self::AngleDeg(*value)),
            Value::Quantity { value, .. } => Some(Self::Number(*value)),
            Value::Number(value) => Some(Self::Number(*value)),
            Value::Choice(name) => Some(Self::Choice(name.clone())),
            Value::Boolean(flag) => Some(Self::Boolean(*flag)),
            Value::Text(_) => None,
        }
    }
    /// The JSON member the value serialises under. A roundtrip test uses it to prove the
    /// unit survives the file.
    pub fn json_key(&self) -> &'static str {
        match self {
            Self::LengthMm(_) => "length_mm",
            Self::AngleDeg(_) => "angle_deg",
            Self::Number(_) => "number",
            Self::Choice(_) => "choice",
            Self::Boolean(_) => "boolean",
        }
    }
}

/// The parameter names a style may carry. `profile_ref` is the design name of the builder's
/// `preset` field; the alias keeps style files readable without leaking the field spelling.
const PARAM_ALIASES: [(&str, &str); 22] = [
    ("width_mm", "width_mm"),
    ("height_mm", "height_mm"),
    ("diameter_mm", "diameter_mm"),
    ("thickness_mm", "thickness_mm"),
    ("offset_mm", "offset_mm"),
    ("target_diameter_mm", "target_diameter_mm"),
    ("corner_mm", "corner_mm"),
    ("stone_count", "stone_count"),
    ("stone_diameter_mm", "stone_diameter_mm"),
    ("top_width_mm", "top_width_mm"),
    ("top_length_mm", "top_length_mm"),
    ("shoulder_mm", "shoulder_mm"),
    ("raised_height_mm", "raised_height_mm"),
    ("raised_width_mm", "raised_width_mm"),
    ("profile_ref", "preset"),
    ("profile", "preset"),
    ("preset", "preset"),
    ("solid", "solid"),
    ("mode", "mode"),
    ("position", "position"),
    ("roll_deg", "roll_deg"),
    ("width", "width_mm"),
];

/// Map a style parameter name to the builder field it edits.
pub fn builder_field_for(name: &str) -> Option<&'static str> {
    PARAM_ALIASES
        .iter()
        .find(|(style, _)| style.eq_ignore_ascii_case(name.trim()))
        .map(|(_, field)| *field)
}

/// Translate a profile reference written in a style file to the builder's `preset` choice.
///
/// The style vocabulary names profiles by design reference (`basic_round`), while the builder
/// field speaks the preset names (`circle`). An unknown reference is refused instead of being
/// coerced to a default shape.
pub fn profile_choice(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "basic_round" | "round" | "circle" => Some("circle"),
        "basic_rect" | "basic_square" | "square" | "rectangle" => Some("rectangle"),
        "basic_round_rect" | "round_rect" | "rounded_rectangle" => Some("rounded_rectangle"),
        _ => crate::producers::ProfilePreset::ALL
            .iter()
            .find(|preset| preset.name().eq_ignore_ascii_case(name.trim()))
            .map(|preset| preset.name()),
    }
}

/// The style file: one schema tag and the styles it carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleFile {
    pub schema: String,
    #[serde(default)]
    pub styles: Vec<BuilderStyle>,
}

/// A named parameter snapshot for one builder family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuilderStyle {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub family: Vec<StyleFamily>,
    /// Builder tags the style declares. Empty means "every builder of its family".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub builders: Vec<String>,
    pub parameters: BTreeMap<String, StyleValue>,
    /// A thumbnail resource key. A missing or unknown key falls back to a placeholder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

impl BuilderStyle {
    /// Snapshot the session's fields as a style, keeping each value's type and unit.
    pub fn snapshot(session: &BuilderSession, name: impl Into<String>) -> Self {
        let parameters = session
            .fields()
            .iter()
            .filter_map(|field| {
                StyleValue::from_request(&field.value).map(|value| (field.name.to_string(), value))
            })
            .collect();
        Self {
            name: name.into(),
            family: vec![StyleFamily::Band],
            builders: vec![session.kind().tag().to_string()],
            parameters,
            thumbnail: None,
        }
    }

    /// The style a mode starts from: every field at its default, recorded explicitly.
    pub fn defaults_of(session: &BuilderSession, name: impl Into<String>) -> Self {
        Self::snapshot(session, name)
    }

    /// Validate the record: name rules, at least one family, at least one parameter, and
    /// every parameter typed and inside its domain.
    pub fn validate(&self) -> Result<()> {
        validate_style_name(&self.name)?;
        if self.family.is_empty() {
            return Err(Error::Invalid("style family"));
        }
        if self.parameters.is_empty() {
            return Err(Error::Invalid("style parameters"));
        }
        if self.parameters.len() > PARAM_ALIASES.len() {
            return Err(Error::Budget("style parameters"));
        }
        for (name, value) in &self.parameters {
            if builder_field_for(name).is_none() {
                return Err(Error::Invalid("style parameter"));
            }
            value.validate()?;
        }
        if self
            .builders
            .iter()
            .any(|tag| BuilderKind::from_tag(tag).is_none())
        {
            return Err(Error::Invalid("style builder"));
        }
        Ok(())
    }

    /// Whether the style declares this builder. An empty tag list matches every builder of
    /// its family.
    pub fn targets(&self, kind: BuilderKind) -> bool {
        if self.builders.is_empty() {
            return self.family.iter().any(|family| family.matches(kind));
        }
        self.builders
            .iter()
            .filter_map(|tag| BuilderKind::from_tag(tag))
            .any(|target| target == kind)
    }

    /// Whether any declared family fits this builder. A gem-only style never fits a ring
    /// builder (`OM9-L3-03-12-005`).
    pub fn compatible(&self, kind: BuilderKind) -> bool {
        self.family.iter().any(|family| family.matches(kind))
    }

    /// The parameters this mode requires but the style does not carry. A rail profile style
    /// without a profile reference cannot be applied (`OM9-L3-03-12-003`).
    pub fn missing_required(&self, kind: BuilderKind) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if matches!(
            kind,
            BuilderKind::RailProfile | BuilderKind::RailSweep | BuilderKind::RingResizer
        ) && !self.parameters.contains_key("profile_ref")
            && !self.parameters.contains_key("profile")
            && !self.parameters.contains_key("preset")
        {
            missing.push("profile_ref");
        }
        missing
    }

    /// Apply the parameters to a session. Preview only: the caller refreshes and decides to
    /// Apply. A style for another mode, an unknown parameter or a value the field refuses is
    /// an explicit error, never a silent no-op.
    pub fn apply(&self, session: &mut BuilderSession) -> Result<StyleApply> {
        self.validate()?;
        if !self.compatible(session.kind()) {
            return Err(Error::Invalid("style mode"));
        }
        let missing = self.missing_required(session.kind());
        if !missing.is_empty() {
            return Err(Error::Missing("style profile"));
        }
        let mut applied = Vec::new();
        let mut ignored = Vec::new();
        for (name, value) in &self.parameters {
            let field = builder_field_for(name).ok_or(Error::Invalid("style parameter"))?;
            if field == "mode" {
                continue;
            }
            if session.field(field).is_none() {
                ignored.push(name.clone());
                continue;
            }
            // A style file names the profile by its design reference (`basic_round`); the
            // builder field speaks preset names (`circle`). Translate once, here.
            let mut value = value.clone();
            if field == "preset"
                && let StyleValue::Choice(reference) = &value
            {
                let choice = profile_choice(reference).ok_or(Error::Invalid("style profile"))?;
                value = StyleValue::Choice(choice.to_string());
            }
            session.set_field(field, value.to_request())?;
            applied.push(field);
        }
        if applied.is_empty() {
            // Every parameter belonged to another mode: that is a mode mismatch, not success.
            return Err(Error::Invalid("style mode"));
        }
        // Report in field-name order so the panel and the command line read the same list.
        applied.sort_unstable();
        Ok(StyleApply {
            style: self.name.clone(),
            applied,
            ignored,
        })
    }

    /// Copy the declared compatible attributes of a source entity into this style. Only the
    /// attributes listed here are read; the target entity identity is never written, so
    /// stable IDs survive (`OM9-L3-03-12-006`).
    pub fn match_attributes(&mut self, entity: &Entity) -> Result<Vec<&'static str>> {
        let mut copied = Vec::new();
        let mut declared = false;
        for slot in STYLE_ATTRIBUTE_SLOTS {
            let Some(extension) = entity.extension(slot) else {
                continue;
            };
            declared = true;
            let payload: serde_json::Value = serde_json::from_str(&extension.payload)
                .map_err(|_| Error::Invalid("style attribute"))?;
            let serde_json::Value::Object(object) = payload else {
                return Err(Error::Invalid("style attribute"));
            };
            for (key, kind) in STYLE_ATTRIBUTES {
                let Some(value) = object.get(key) else {
                    continue;
                };
                let converted = match kind {
                    StyleAttributeKind::Length => serde_json::from_value::<f64>(value.clone())
                        .ok()
                        .map(StyleValue::LengthMm),
                    StyleAttributeKind::Choice => serde_json::from_value::<String>(value.clone())
                        .ok()
                        .map(StyleValue::Choice),
                    StyleAttributeKind::Boolean => serde_json::from_value::<bool>(value.clone())
                        .ok()
                        .map(StyleValue::Boolean),
                };
                let converted = converted.ok_or(Error::Invalid("style attribute"))?;
                converted.validate()?;
                self.parameters.insert(key.to_string(), converted);
                copied.push(key);
            }
        }
        if !declared {
            // No declared slot means there is nothing this style may copy; refusing is
            // clearer than silently returning an empty list.
            return Err(Error::Missing("style source"));
        }
        if copied.is_empty() {
            return Err(Error::Invalid("style attribute"));
        }
        Ok(copied)
    }
}

/// The type each copied attribute keeps, so `width=3` becomes a length and never a bare
/// number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleAttributeKind {
    Length,
    Choice,
    Boolean,
}

/// The attributes a style copies from a source entity, with the type each one keeps.
pub const STYLE_ATTRIBUTES: [(&str, StyleAttributeKind); 3] = [
    ("width_mm", StyleAttributeKind::Length),
    ("height_mm", StyleAttributeKind::Length),
    ("material", StyleAttributeKind::Choice),
];
/// The extension slots a style may copy attributes from. A source without any of them has
/// no compatible schema.
pub const STYLE_ATTRIBUTE_SLOTS: [&str; 2] = ["om9.rail", "om9.head"];

/// The outcome of applying a style: the fields it set and the parameters this mode ignored.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleApply {
    pub style: String,
    pub applied: Vec<&'static str>,
    pub ignored: Vec<String>,
}

impl StyleApply {
    /// A one-line report for the panel/command line.
    pub fn summary(&self) -> String {
        let applied = self.applied.join(", ");
        if self.ignored.is_empty() {
            format!("Style {} applied: {applied}", self.style)
        } else {
            format!(
                "Style {} applied: {applied} (ignored {})",
                self.style,
                self.ignored.join(", ")
            )
        }
    }
}

/// A thumbnail for the browser: an icon key and whether the entry fell back to the
/// placeholder because its resource was missing or corrupt (`OM9-L3-03-12-002`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleThumbnail {
    pub icon: &'static str,
    pub fallback: bool,
}

impl StyleThumbnail {
    /// The thumbnail the entry shows. Only the authored builder icons are accepted; any
    /// other key is treated as a corrupt resource and falls back without failing the list.
    pub fn for_style(style: &BuilderStyle) -> Self {
        const AUTHOURED: [&str; 3] = ["RingRail", "RailProfile", "RailSweep"];
        let icon = style.thumbnail.as_deref().and_then(|key| {
            AUTHOURED
                .into_iter()
                .find(|known| known.eq_ignore_ascii_case(key))
        });
        match icon {
            Some(icon) => Self {
                icon,
                fallback: false,
            },
            None => Self {
                icon: "RingRail",
                fallback: true,
            },
        }
    }
}

/// One library row: the style plus what the browser may show for the active mode.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleEntry<'a> {
    pub style: &'a BuilderStyle,
    /// Enabled for the active mode. An incompatible style is still listed, disabled.
    pub enabled: bool,
    pub thumbnail: StyleThumbnail,
    pub builtin: bool,
}

/// Built-in styles plus the user's own. The two lists are separate so saving can never
/// rewrite a default (`OM9-L3-03-12-004`).
#[derive(Debug, Clone, PartialEq)]
pub struct StyleLibrary {
    builtins: Vec<BuilderStyle>,
    user: Vec<BuilderStyle>,
}

impl Default for StyleLibrary {
    fn default() -> Self {
        Self::builtin()
    }
}

impl StyleLibrary {
    /// A library with the shipped defaults: two ring-band styles with independent widths and
    /// one gem style that a ring builder must never offer.
    pub fn builtin() -> Self {
        let band = |name: &str, width: f64, height: f64, thumbnail: Option<&str>| BuilderStyle {
            name: name.to_string(),
            family: vec![StyleFamily::Band],
            builders: Vec::new(),
            parameters: BTreeMap::from([
                ("width_mm".to_string(), StyleValue::LengthMm(width)),
                ("height_mm".to_string(), StyleValue::LengthMm(height)),
                (
                    "profile_ref".to_string(),
                    StyleValue::Choice("circle".to_string()),
                ),
            ]),
            thumbnail: thumbnail.map(str::to_string),
        };
        let mut gem = band("Brilliant 1.9", 1.9, 1.4, Some("RingRail"));
        gem.name = "Brilliant Gem 1.9".to_string();
        gem.family = vec![StyleFamily::Gem];
        gem.parameters.insert(
            "profile_ref".to_string(),
            StyleValue::Choice("circle".into()),
        );
        Self {
            builtins: vec![
                band("Classic 2.2", 2.2, 1.6, Some("RingRail")),
                band("Signet 3.0", 3.0, 2.0, Some("RailProfile")),
                gem,
            ],
            user: Vec::new(),
        }
    }

    pub fn builtins(&self) -> &[BuilderStyle] {
        &self.builtins
    }
    pub fn user(&self) -> &[BuilderStyle] {
        &self.user
    }
    pub fn all(&self) -> impl Iterator<Item = (&BuilderStyle, bool)> {
        self.builtins
            .iter()
            .map(|style| (style, true))
            .chain(self.user.iter().map(|style| (style, false)))
    }
    pub fn len(&self) -> usize {
        self.builtins.len() + self.user.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Find a style by name, case-insensitively. The user list wins a name clash, but
    /// [`Self::save_user`] refuses a clash before one can exist.
    pub fn find(&self, name: &str) -> Option<&BuilderStyle> {
        let wanted = name.trim();
        self.user
            .iter()
            .chain(self.builtins.iter())
            .find(|style| style.name.eq_ignore_ascii_case(wanted))
    }

    /// Every row the browser shows for one mode, compatible first, then the disabled rest.
    pub fn styles_for(&self, kind: BuilderKind) -> Vec<StyleEntry<'_>> {
        let mut rows: Vec<StyleEntry<'_>> = self
            .all()
            .map(|(style, builtin)| StyleEntry {
                style,
                enabled: style.compatible(kind) && style.targets(kind),
                thumbnail: StyleThumbnail::for_style(style),
                builtin,
            })
            .collect();
        rows.sort_by(|a, b| {
            b.enabled
                .cmp(&a.enabled)
                .then_with(|| a.style.name.cmp(&b.style.name))
        });
        rows
    }

    /// Save a user style. The name is validated and must not clash with any existing style;
    /// the built-in list is never touched.
    pub fn save_user(&mut self, style: BuilderStyle) -> Result<&BuilderStyle> {
        style.validate()?;
        if self.find(&style.name).is_some() {
            return Err(Error::Invalid("style exists"));
        }
        if self.user.len() >= MAX_STYLE_COUNT {
            return Err(Error::Budget("style library"));
        }
        self.user.push(style);
        Ok(self.user.last().expect("pushed"))
    }

    /// Remove a user style. A built-in name is refused: defaults are read-only.
    pub fn remove_user(&mut self, name: &str) -> Result<bool> {
        let wanted = name.trim();
        if self
            .builtins
            .iter()
            .any(|style| style.name.eq_ignore_ascii_case(wanted))
        {
            return Err(Error::Permission("style builtin"));
        }
        let before = self.user.len();
        self.user
            .retain(|style| !style.name.eq_ignore_ascii_case(wanted));
        Ok(self.user.len() != before)
    }

    /// The user list as a `v1` style file.
    pub fn to_json(&self) -> String {
        let file = StyleFile {
            schema: STYLE_SCHEMA.to_string(),
            styles: self.user.clone(),
        };
        serde_json::to_string_pretty(&file).expect("style file serialises")
    }

    /// Parse a style file. An unknown schema, an unknown member or an invalid parameter is
    /// refused; the caller decides whether a failed load is fatal.
    pub fn parse_styles(text: &str) -> Result<Vec<BuilderStyle>> {
        let file: StyleFile =
            serde_json::from_str(text).map_err(|_| Error::Invalid("style schema"))?;
        if file.schema != STYLE_SCHEMA {
            return Err(Error::Invalid("style schema"));
        }
        if file.styles.len() > MAX_STYLE_COUNT {
            return Err(Error::Budget("style library"));
        }
        let mut names = BTreeSet::new();
        for style in &file.styles {
            style.validate()?;
            if !names.insert(style.name.to_ascii_lowercase()) {
                return Err(Error::Invalid("style exists"));
            }
        }
        Ok(file.styles)
    }

    /// Replace the user list with the styles of `path`. A missing file is an empty user list;
    /// a refused write/permission is reported, never swallowed.
    pub fn load_from(&mut self, path: &Path) -> Result<usize> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let styles = Self::parse_styles(&text)?;
                self.user = styles;
                Ok(self.user.len())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.user.clear();
                Ok(0)
            }
            Err(_) => Err(Error::Permission("style library")),
        }
    }

    /// Write the user list to `path`, creating the directory when needed.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(directory) = path.parent()
            && !directory.as_os_str().is_empty()
        {
            std::fs::create_dir_all(directory).map_err(|_| Error::Permission("style library"))?;
        }
        std::fs::write(path, self.to_json()).map_err(|_| Error::Permission("style library"))
    }
}

/// Validate a style name: trimmed, bounded, no path separators, no traversal, no control
/// characters. A name is a key in the library, so it may never look like a path.
pub fn validate_style_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::Invalid("style name"));
    }
    if trimmed.len() != name.len() {
        return Err(Error::Invalid("style name"));
    }
    if trimmed.len() > MAX_STYLE_NAME_BYTES {
        return Err(Error::Budget("style name"));
    }
    if trimmed.contains(['/', '\\', '\0'])
        || trimmed.contains("..")
        || trimmed.chars().any(char::is_control)
        || trimmed.starts_with('.')
    {
        return Err(Error::Invalid("style name"));
    }
    Ok(())
}
