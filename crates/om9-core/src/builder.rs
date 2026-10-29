// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder session for the ring rail / rail profile / rail sweep chain (`OM9-L2-03-11`,
//! `OM9-L2-04-02`).
//!
//! The UI layer owns geometry input and painting; this module owns the state a builder panel
//! and its viewport handles share:
//!
//! * typed fields with a declared domain (`diameter_mm`, `width_mm`, ...);
//! * a live preview computed through a [`ProducerRegistry`];
//! * viewport handle positions derived from the preview geometry;
//! * one commit plan (entity + typed recipe + published result) the host applies inside one
//!   transaction, so one Undo removes the whole feature.
//!
//! A preview is never written to the document: the caller paints it, and only
//! [`BuilderSession::plan`] produces the records a transaction inserts.
use crate::{
    Error, Result,
    associative::{FeaturePort, PublishedSubshapes, RebuildRequest, Subshape},
    budget::Limits,
    geometry::{Curve, Geometry, GeometryKind, Point3, Transform, Unit},
    history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    identity::{HistoryId, ObjectId},
    model::{Document, Entity},
    producers::{
        OUTSIDE_RAIL_COMMAND, ProducerRegistry, ProfilePreset, RAIL_PROFILE_COMMAND,
        RAIL_SWEEP_COMMAND, RING_RAIL_COMMAND, RING_RESIZER_COMMAND,
    },
    request::{Dimension, OptionDomain, Options, Selection, SourceBinding, Value},
    result::{ConversionReport, PublishedResult, ResultStatus},
};
use std::collections::{BTreeMap, BTreeSet};

/// The builder modes this owner implements. Each one maps to the command recorded on the
/// history node, so reopening the feature uses the same producer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuilderKind {
    RingRail,
    RailProfile,
    RailSweep,
    OutsideRail,
    RingResizer,
    EternityBand,
    SignetRing,
    RaisedBand,
    AwardRing,
}

impl BuilderKind {
    pub const ALL: [Self; 9] = [
        Self::RingRail,
        Self::RailProfile,
        Self::RailSweep,
        Self::OutsideRail,
        Self::RingResizer,
        Self::EternityBand,
        Self::SignetRing,
        Self::RaisedBand,
        Self::AwardRing,
    ];

    pub fn command(self) -> &'static str {
        match self {
            Self::RingRail => RING_RAIL_COMMAND,
            Self::RailProfile => RAIL_PROFILE_COMMAND,
            Self::RailSweep => RAIL_SWEEP_COMMAND,
            Self::OutsideRail => OUTSIDE_RAIL_COMMAND,
            Self::RingResizer => RING_RESIZER_COMMAND,
            Self::EternityBand => crate::ring_builder::ETERNITY_BAND_COMMAND,
            Self::SignetRing => crate::ring_builder::SIGNET_RING_COMMAND,
            Self::RaisedBand => crate::ring_builder::RAISED_BAND_COMMAND,
            Self::AwardRing => crate::ring_builder::AWARD_RING_COMMAND,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::RingRail => "Ring Rail",
            Self::RailProfile => "Rail Profile",
            Self::RailSweep => "Rail Sweep",
            Self::OutsideRail => "Outside Rail",
            Self::RingResizer => "Ring Resizer",
            Self::EternityBand => "Eternity Band",
            Self::SignetRing => "Signet Ring",
            Self::RaisedBand => "Raised Band",
            Self::AwardRing => "Award Ring",
        }
    }
    /// The ring builder family of this mode, if it is one (`OM9-L2-04-03`).
    pub fn ring_family(self) -> Option<crate::ring_builder::RingBuilderKind> {
        match self {
            Self::EternityBand => Some(crate::ring_builder::RingBuilderKind::Eternity),
            Self::SignetRing => Some(crate::ring_builder::RingBuilderKind::Signet),
            Self::RaisedBand => Some(crate::ring_builder::RingBuilderKind::Raised),
            Self::AwardRing => Some(crate::ring_builder::RingBuilderKind::Award),
            _ => None,
        }
    }
    /// Input roles in the order the command line/pick fills them.
    pub fn input_roles(self) -> &'static [&'static str] {
        match self {
            Self::RingRail => &[],
            Self::RailProfile | Self::OutsideRail => &["rail"],
            Self::RailSweep => &["rail", "profile"],
            Self::RingResizer => &["ring"],
            Self::EternityBand | Self::SignetRing | Self::RaisedBand | Self::AwardRing => &[],
        }
    }
    /// The geometry kind every input role expects.
    fn input_kind(self) -> GeometryKind {
        GeometryKind::Curve
    }
    /// The stable tag a style file, the panel tab and the evidence record use.
    pub fn tag(self) -> &'static str {
        match self {
            Self::RingRail => "ring_rail",
            Self::RailProfile => "rail_profile",
            Self::RailSweep => "rail_sweep",
            Self::OutsideRail => "outside_rail",
            Self::RingResizer => "ring_resizer",
            Self::EternityBand => crate::ring_builder::RingBuilderKind::Eternity.tag(),
            Self::SignetRing => crate::ring_builder::RingBuilderKind::Signet.tag(),
            Self::RaisedBand => crate::ring_builder::RingBuilderKind::Raised.tag(),
            Self::AwardRing => crate::ring_builder::RingBuilderKind::Award.tag(),
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.tag().eq_ignore_ascii_case(tag.trim()))
    }
    /// Whether this mode builds a ring. A gem/setting style is never offered here
    /// (`OM9-L3-03-12-005`).
    pub fn is_ring_builder(self) -> bool {
        matches!(
            self,
            Self::RingRail
                | Self::RailProfile
                | Self::RailSweep
                | Self::OutsideRail
                | Self::RingResizer
                | Self::EternityBand
                | Self::SignetRing
                | Self::RaisedBand
                | Self::AwardRing
        )
    }
}

/// One typed parameter of the builder.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderField {
    pub name: &'static str,
    pub label: &'static str,
    pub domain: OptionDomain,
    pub value: Value,
}

impl BuilderField {
    fn length(name: &'static str, label: &'static str, default: Value, min: f64, max: f64) -> Self {
        Self {
            name,
            label,
            domain: OptionDomain::Number {
                dimension: Dimension::Length,
                min: Some(min),
                min_inclusive: true,
                max: Some(max),
                step: Some(0.1),
            },
            value: default,
        }
    }
    fn ratio(name: &'static str, label: &'static str, default: f64) -> Self {
        Self {
            name,
            label,
            domain: OptionDomain::Number {
                dimension: Dimension::Ratio,
                min: Some(0.0),
                min_inclusive: true,
                max: Some(1.0),
                step: Some(0.01),
            },
            value: Value::ratio(default).expect("finite ratio default"),
        }
    }
    fn angle(name: &'static str, label: &'static str, default: f64) -> Self {
        Self {
            name,
            label,
            domain: OptionDomain::Number {
                dimension: Dimension::Angle,
                min: Some(-180.0),
                min_inclusive: true,
                max: Some(180.0),
                step: Some(1.0),
            },
            value: Value::angle(crate::units::AngleDeg::try_new(default).expect("finite angle")),
        }
    }
    fn count(name: &'static str, label: &'static str, default: u32, min: u32, max: u32) -> Self {
        Self {
            name,
            label,
            domain: OptionDomain::Number {
                dimension: Dimension::Count,
                min: Some(f64::from(min)),
                min_inclusive: true,
                max: Some(f64::from(max)),
                step: Some(1.0),
            },
            value: Value::count(default),
        }
    }
    pub fn numeric(&self) -> Option<f64> {
        match &self.value {
            Value::Quantity { value, .. } => Some(*value),
            _ => None,
        }
    }
    /// The quantisation step of a numeric field, in millimetres.
    pub fn step(&self) -> Option<f64> {
        match &self.domain {
            OptionDomain::Number { step, .. } => {
                step.filter(|step| step.is_finite() && *step > 0.0)
            }
            _ => None,
        }
    }
    /// Snap a value to this field's step. A slider therefore lands on exactly 2.5 mm
    /// instead of 2.5000000000000004, so the panel text, the typed model value and the
    /// preview read the same number (`OM9-L3-03-11-003`).
    pub fn quantize(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match self.step() {
            Some(step) => {
                let snapped = (value / step).round() * step;
                // 25 * 0.1 is not exactly 2.5 in binary floating point; clean the tail
                // instead of leaking it into the field and the command line.
                let clean = (snapped * 1.0e9).round() / 1.0e9;
                clean.is_finite().then_some(clean)
            }
            None => Some(value),
        }
    }
    /// The slider position of a numeric field, `0.0..=1.0`.
    pub fn slider_ratio(&self) -> Option<f64> {
        let value = self.numeric()?;
        let (min, max) = self.numeric_range()?;
        let span = max - min;
        if !span.is_finite() || span <= 0.0 {
            return None;
        }
        Some(((value - min) / span).clamp(0.0, 1.0))
    }
    /// The value a slider position means, quantised to the field step. The domain still
    /// decides whether the result is acceptable.
    pub fn from_slider_ratio(&self, ratio: f64) -> Option<f64> {
        if !ratio.is_finite() {
            return None;
        }
        let (min, max) = self.numeric_range()?;
        let value = min + (max - min) * ratio.clamp(0.0, 1.0);
        self.quantize(value).map(|value| value.clamp(min, max))
    }
    fn numeric_range(&self) -> Option<(f64, f64)> {
        match &self.domain {
            OptionDomain::Number { min, max, .. } => Some((
                min.unwrap_or(f64::NEG_INFINITY),
                max.unwrap_or(f64::INFINITY),
            )),
            _ => None,
        }
    }
    fn choice(
        name: &'static str,
        label: &'static str,
        choices: impl Iterator<Item = &'static str>,
        default: &'static str,
    ) -> Self {
        Self {
            name,
            label,
            domain: OptionDomain::Choice(choices.map(str::to_string).collect()),
            value: Value::Choice(default.into()),
        }
    }
    pub fn selected_choice(&self) -> Option<&str> {
        match &self.value {
            Value::Choice(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

/// A viewport handle the builder paints. Dragging it changes `field` by `delta_mm` along
/// `axis`; the panel and the handle share the one field, so they can never disagree.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderHandle {
    pub name: &'static str,
    pub field: &'static str,
    pub label: &'static str,
    pub position: Point3,
    pub axis: Point3,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    /// Field change per millimetre of drag. Length handles use 1; a ratio handle uses the
    /// rail-length conversion so a one-millimetre drag is a small fraction, not a jump to 1.
    pub scale: f64,
}

/// The geometry a preview paints. It is not part of the document.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderPreview {
    pub geometry: Geometry,
    pub subshapes: Vec<Subshape>,
    pub handles: Vec<BuilderHandle>,
    pub summary: String,
}

/// The records one Apply inserts, in one transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderPlan {
    pub entity: Entity,
    pub recipe: Recipe,
    pub published: PublishedResult,
}

/// A builder session: one mode, its fields and the selection it reads.
#[derive(Debug, Clone)]
pub struct BuilderSession {
    kind: BuilderKind,
    label: String,
    output: ObjectId,
    recipe: HistoryId,
    inputs: Vec<(String, ObjectId)>,
    fields: Vec<BuilderField>,
    preview: Option<BuilderPreview>,
    /// Presets the user removed from this session's library. A preview that still selects
    /// one refuses instead of silently falling back to another shape.
    removed_presets: BTreeSet<&'static str>,
    /// Monotonic edit counter. The panel captures it when it paints a widget and hands it
    /// back with the edit, so a panel event that raced a viewport drag is refused instead
    /// of overwriting the newer value (`OM9-L3-03-11-006`).
    revision: u64,
}

impl BuilderSession {
    /// Start one builder. `selection` must hold the curves the mode consumes, in command
    /// order; a wrong count or a non-curve selection is refused before a field is shown.
    pub fn new(kind: BuilderKind, document: &Document, selection: &[ObjectId]) -> Result<Self> {
        let roles = kind.input_roles();
        if selection.len() != roles.len() {
            return Err(Error::Invalid("builder selection count"));
        }
        let mut inputs = Vec::with_capacity(roles.len());
        for (role, object) in roles.iter().zip(selection) {
            let entity = document
                .entities
                .get(object)
                .ok_or(Error::Missing("builder input"))?;
            if entity.geometry.kind() != kind.input_kind() {
                return Err(Error::Invalid("builder input kind"));
            }
            inputs.push(((*role).to_string(), *object));
        }
        let (fields, label) = match kind {
            BuilderKind::RingRail => (
                vec![
                    BuilderField::length(
                        "diameter_mm",
                        "Diameter",
                        Value::length(crate::units::LengthMm::try_new(18.0)?),
                        1.0,
                        40.0,
                    ),
                    BuilderField::choice(
                        "standard",
                        "Standard",
                        ["explicit", "US"].into_iter(),
                        "explicit",
                    ),
                    BuilderField::count("us_size", "US size", 8, 1, 15),
                ],
                "Ring Rail".to_string(),
            ),
            BuilderKind::RailProfile => (
                vec![
                    BuilderField::length(
                        "width_mm",
                        "Width",
                        Value::length(crate::units::LengthMm::try_new(2.0)?),
                        0.1,
                        20.0,
                    ),
                    BuilderField::length(
                        "height_mm",
                        "Height",
                        Value::length(crate::units::LengthMm::try_new(1.5)?),
                        0.1,
                        20.0,
                    ),
                    BuilderField::length(
                        "corner_mm",
                        "Corner",
                        Value::length(crate::units::LengthMm::try_new(0.4)?),
                        0.0,
                        5.0,
                    ),
                    BuilderField::choice(
                        "preset",
                        "Profile",
                        ProfilePreset::ALL.iter().map(|preset| preset.name()),
                        "rectangle",
                    ),
                    BuilderField::ratio("position", "Position", 0.0),
                    BuilderField::angle("roll_deg", "Roll", 0.0),
                ],
                "Rail Profile".to_string(),
            ),
            BuilderKind::RailSweep => (
                vec![
                    BuilderField {
                        name: "solid",
                        label: "Solid",
                        domain: OptionDomain::Boolean,
                        value: Value::Boolean(false),
                    },
                    BuilderField {
                        name: "closed_rail",
                        label: "Closed rail",
                        domain: OptionDomain::Boolean,
                        value: Value::Boolean(true),
                    },
                ],
                "Rail Sweep".to_string(),
            ),
            BuilderKind::OutsideRail => (
                vec![BuilderField::length(
                    "offset_mm",
                    "Offset",
                    Value::length(crate::units::LengthMm::try_new(2.0)?),
                    0.1,
                    50.0,
                )],
                "Outside Rail".to_string(),
            ),
            BuilderKind::RingResizer => (
                vec![
                    BuilderField::length(
                        "target_diameter_mm",
                        "Target",
                        Value::length(crate::units::LengthMm::try_new(19.0)?),
                        1.0,
                        60.0,
                    ),
                    BuilderField::length(
                        "thickness_mm",
                        "Thickness",
                        Value::length(crate::units::LengthMm::try_new(1.5)?),
                        0.1,
                        10.0,
                    ),
                ],
                "Ring Resizer".to_string(),
            ),
            BuilderKind::EternityBand
            | BuilderKind::SignetRing
            | BuilderKind::RaisedBand
            | BuilderKind::AwardRing => (ring_fields(kind)?, kind.title().to_string()),
        };
        let session = Self {
            kind,
            label,
            output: ObjectId::new(),
            recipe: HistoryId::new(),
            inputs,
            fields,
            preview: None,
            removed_presets: BTreeSet::new(),
            revision: 0,
        };
        // Validate the defaults against their own domains before showing the panel.
        for field in &session.fields {
            field.domain.check(&field.value, &Limits::default())?;
        }
        Ok(session)
    }

    pub fn kind(&self) -> BuilderKind {
        self.kind
    }
    pub fn title(&self) -> &str {
        &self.label
    }
    pub fn fields(&self) -> &[BuilderField] {
        &self.fields
    }
    pub fn field(&self, name: &str) -> Option<&BuilderField> {
        self.fields.iter().find(|field| field.name == name)
    }
    pub fn inputs(&self) -> &[(String, ObjectId)] {
        &self.inputs
    }
    pub fn output(&self) -> ObjectId {
        self.output
    }
    /// Reuse the identity of an existing feature when the editor is reopened from its
    /// recorded recipe (`OM9-L3-03-13-005`): Apply then updates that object and recipe
    /// instead of inserting a second one.
    pub fn adopt_history(&mut self, output: ObjectId, recipe: HistoryId) {
        self.output = output;
        self.recipe = recipe;
    }
    pub fn recipe(&self) -> HistoryId {
        self.recipe
    }
    pub fn preview(&self) -> Option<&BuilderPreview> {
        self.preview.as_ref()
    }

    /// The edit counter a panel widget captured when it painted the current value.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether every input the builder reads is editable. A locked layer or a locked
    /// object makes the session read-only, so the panel hides its handles and a drag is
    /// refused instead of silently moving a locked input (`OM9-L3-03-11-005`).
    pub fn editable(&self, document: &Document) -> bool {
        self.inputs.iter().all(|(_, object)| {
            document.entities.get(object).is_some_and(|entity| {
                document
                    .permissions_for_entity(entity, false)
                    .is_ok_and(|permissions| permissions.editable)
            })
        })
    }

    /// The profile presets this session still offers. The library is static; removing one is
    /// a session edit that the next preview refuses to fall back from.
    pub fn presets(&self) -> Vec<ProfilePreset> {
        ProfilePreset::ALL
            .into_iter()
            .filter(|preset| !self.removed_presets.contains(preset.name()))
            .collect()
    }
    pub fn selected_preset(&self) -> Option<ProfilePreset> {
        self.field("preset")
            .and_then(BuilderField::selected_choice)
            .and_then(ProfilePreset::from_name)
    }
    /// Choose one profile from the selector window. The document is untouched; only the
    /// preview is recomputed.
    pub fn set_preset(&mut self, name: &str) -> Result<()> {
        let preset = ProfilePreset::from_name(name).ok_or(Error::Invalid("profile preset"))?;
        if self.removed_presets.contains(preset.name()) {
            return Err(Error::Missing("profile preset"));
        }
        self.set_field("preset", Value::Choice(preset.name().into()))
    }
    /// Remove one preset from this session's library. If it was the selected shape the
    /// preview is dropped, so the panel cannot apply a shape the library no longer lists.
    pub fn remove_preset(&mut self, name: &str) -> Result<bool> {
        let preset = ProfilePreset::from_name(name).ok_or(Error::Invalid("profile preset"))?;
        let removed = self.removed_presets.insert(preset.name());
        if removed && self.selected_preset() == Some(preset) {
            self.preview = None;
        }
        Ok(removed)
    }

    /// Apply one typed field value. The domain decides the range and the unit; a value of the
    /// wrong dimension is refused with the same rule the command line uses.
    pub fn set_field(&mut self, name: &str, value: Value) -> Result<()> {
        // An explicit diameter always wins over a standard: editing the diameter switches the
        // mode back to explicit so the next refresh cannot silently replace the value.
        if name == "diameter_mm"
            && self
                .field("standard")
                .and_then(BuilderField::selected_choice)
                .is_some_and(|standard| standard.eq_ignore_ascii_case("US"))
            && let Some(field) = self
                .fields
                .iter_mut()
                .find(|field| field.name == "standard")
        {
            field.value = Value::Choice("explicit".into());
        }
        let field = self
            .fields
            .iter_mut()
            .find(|field| field.name == name)
            .ok_or(Error::Missing("builder field"))?;
        field.domain.check(&value, &Limits::default())?;
        field.value = value;
        self.preview = None;
        self.revision = self.revision.wrapping_add(1);
        if self.kind == BuilderKind::RingRail {
            self.sync_standard_diameter()?;
        }
        Ok(())
    }

    /// The selected jewelry standard, or `None` for an explicit diameter.
    fn selected_standard(&self) -> Option<crate::ring_size::RingSizeStandard> {
        self.field("standard")
            .and_then(BuilderField::selected_choice)
            .and_then(|id| crate::ring_size::standard_from_id(id).ok().flatten())
    }

    /// Copy the frozen standard diameter into the `diameter_mm` field, which stays the one
    /// value the preview, handle, recipe and panel all read.
    fn sync_standard_diameter(&mut self) -> Result<()> {
        let Some(standard) = self.selected_standard() else {
            return Ok(());
        };
        let size = self
            .field("us_size")
            .and_then(BuilderField::numeric)
            .ok_or(Error::Missing("US ring size"))?;
        let size = u8::try_from(size as u32).map_err(|_| Error::Invalid("US ring size 1..15"))?;
        let diameter = standard.diameter_mm(size)?;
        if let Some(field) = self
            .fields
            .iter_mut()
            .find(|field| field.name == "diameter_mm")
        {
            field.value = Value::length(crate::units::LengthMm::try_new(diameter)?);
        }
        Ok(())
    }
    /// Apply a value only if no other edit happened since the caller painted the widget.
    /// A queued panel event that races a newer viewport drag is refused instead of
    /// overwriting the value the user just saw (`OM9-L3-03-11-006`).
    pub fn set_field_at(&mut self, name: &str, value: Value, expected_revision: u64) -> Result<()> {
        if expected_revision != self.revision {
            return Err(Error::Stale("builder field"));
        }
        self.set_field(name, value)
    }
    /// Move a numeric field by `delta` (a slider or a handle drag), clamped to its domain
    /// and quantised to its declared step, so a drag cannot invent sub-step values.
    pub fn adjust_field(&mut self, name: &str, delta: f64) -> Result<()> {
        if !delta.is_finite() {
            return Err(Error::Invalid("builder delta"));
        }
        let current = self
            .field(name)
            .ok_or(Error::Missing("builder field"))?
            .numeric()
            .ok_or(Error::Invalid("builder numeric field"))?;
        let (min, max) = match &self
            .field(name)
            .ok_or(Error::Missing("builder field"))?
            .domain
        {
            OptionDomain::Number { min, max, .. } => (
                min.unwrap_or(f64::NEG_INFINITY),
                max.unwrap_or(f64::INFINITY),
            ),
            _ => return Err(Error::Invalid("builder numeric field")),
        };
        let next = (current + delta).clamp(min, max);
        let stepped = self
            .field(name)
            .ok_or(Error::Missing("builder field"))?
            .quantize(next)
            .ok_or(Error::Invalid("builder delta"))?;
        let value = value_like(
            self.field(name).ok_or(Error::Missing("builder field"))?,
            stepped,
        );
        self.set_field(name, value)
    }
    /// Set a slider position directly: the field's own range and step decide the value.
    pub fn set_slider(&mut self, name: &str, ratio: f64) -> Result<()> {
        let value = self
            .field(name)
            .ok_or(Error::Missing("builder field"))?
            .from_slider_ratio(ratio)
            .ok_or(Error::Invalid("builder slider"))?;
        let value = value_like(
            self.field(name).ok_or(Error::Missing("builder field"))?,
            value,
        );
        self.set_field(name, value)
    }
    /// Reset every field to the values the session started with.
    pub fn reset(&mut self, document: &Document, selection: &[ObjectId]) -> Result<()> {
        *self = Self::new(self.kind, document, selection)?;
        Ok(())
    }

    /// Whether the current fields differ from the session defaults.
    pub fn is_dirty(&self) -> bool {
        match self.kind {
            BuilderKind::RingRail => {
                self.field("diameter_mm")
                    .and_then(BuilderField::numeric)
                    .is_none_or(|value| (value - 18.0).abs() > 1.0e-12)
                    || self
                        .field("standard")
                        .and_then(BuilderField::selected_choice)
                        .is_none_or(|value| !value.eq_ignore_ascii_case("explicit"))
                    || self
                        .field("us_size")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| (value - 8.0).abs() > 1.0e-12)
            }
            BuilderKind::RailProfile => {
                self.field("width_mm")
                    .and_then(BuilderField::numeric)
                    .is_none_or(|value| (value - 2.0).abs() > 1.0e-12)
                    || self
                        .field("height_mm")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| (value - 1.5).abs() > 1.0e-12)
                    || self
                        .field("corner_mm")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| (value - 0.4).abs() > 1.0e-12)
                    || self.selected_preset() != Some(ProfilePreset::Rectangle)
                    || self
                        .field("position")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| value.abs() > 1.0e-12)
                    || self
                        .field("roll_deg")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| value.abs() > 1.0e-12)
            }
            BuilderKind::RailSweep => {
                matches!(
                    self.field("solid").map(|field| &field.value),
                    Some(Value::Boolean(true))
                ) || matches!(
                    self.field("closed_rail").map(|field| &field.value),
                    Some(Value::Boolean(false))
                )
            }
            BuilderKind::OutsideRail => self
                .field("offset_mm")
                .and_then(BuilderField::numeric)
                .is_none_or(|value| (value - 2.0).abs() > 1.0e-12),
            BuilderKind::RingResizer => {
                self.field("target_diameter_mm")
                    .and_then(BuilderField::numeric)
                    .is_none_or(|value| (value - 19.0).abs() > 1.0e-12)
                    || self
                        .field("thickness_mm")
                        .and_then(BuilderField::numeric)
                        .is_none_or(|value| (value - 1.5).abs() > 1.0e-12)
            }
            BuilderKind::EternityBand
            | BuilderKind::SignetRing
            | BuilderKind::RaisedBand
            | BuilderKind::AwardRing => {
                // A ring mode is dirty when any field left the family defaults: the
                // comparison uses the same parameter list the recipe records.
                let defaults = self
                    .kind
                    .ring_family()
                    .map(|family| family.defaults().parameters())
                    .unwrap_or_default();
                defaults
                    .iter()
                    .any(|(name, value)| self.field(name).map(|field| &field.value) != Some(value))
            }
        }
    }

    fn parameters(&self) -> Options {
        self.fields
            .iter()
            .map(|field| (field.name.to_string(), field.value.clone()))
            .collect()
    }

    /// Build the synthetic request the producer receives. The output is the session's own
    /// object id, so a preview and the committed result are the same identity.
    fn request(&self, document: &Document) -> Result<RebuildRequest> {
        let mut inputs = Vec::with_capacity(self.inputs.len());
        for (role, object) in &self.inputs {
            let entity = document
                .entities
                .get(object)
                .ok_or(Error::Missing("builder input"))?;
            inputs.push(crate::associative::InputSnapshot {
                role: role.clone(),
                object: *object,
                entity: entity.clone(),
            });
        }
        Ok(RebuildRequest {
            recipe: self.recipe,
            command: self.kind.command().into(),
            document: document.id,
            parameters: self.parameters(),
            inputs,
            outputs: vec![self.output],
        })
    }

    /// Recompute the preview through the registry. A producer failure is returned as the
    /// builder's own error; the document is untouched.
    pub fn refresh(&mut self, document: &Document, registry: &ProducerRegistry) -> Result<()> {
        if self.kind == BuilderKind::RingRail {
            self.sync_standard_diameter()?;
        }
        if let Some(name) = self.field("preset").and_then(BuilderField::selected_choice)
            && self.removed_presets.contains(name)
        {
            return Err(Error::Missing("profile preset"));
        }
        let request = self.request(document)?;
        let produced = registry.produce(&request)?;
        let Some(output) = produced.outputs.first() else {
            return Err(Error::Invalid("builder producer output"));
        };
        let subshapes = produced
            .subshapes
            .iter()
            .find(|set| set.output == output.object)
            .map(|set| set.subshapes.clone())
            .unwrap_or_default();
        // A read-only input (locked layer or locked object) paints no handle: the builder
        // must not offer a drag that the document would refuse (`OM9-L3-03-11-005`).
        let handles = if self.editable(document) {
            self.handles(&output.geometry)?
        } else {
            Vec::new()
        };
        let summary = self.summary(&output.geometry);
        self.preview = Some(BuilderPreview {
            geometry: output.geometry.clone(),
            subshapes,
            handles,
            summary,
        });
        Ok(())
    }

    /// The records one Apply inserts. The geometry must come from the current preview; a
    /// caller cannot commit a preview from another field value.
    pub fn plan(&self, document: &Document) -> Result<BuilderPlan> {
        let preview = self
            .preview
            .as_ref()
            .ok_or(Error::Missing("builder preview"))?;
        let mut entity = Entity::new(
            self.label.clone(),
            preview.geometry.clone(),
            document.current_layer,
        );
        entity.id = self.output;
        let tolerance = document
            .tolerance()
            .cloned()
            .unwrap_or_else(Tolerance::om9_template);
        let bindings: Vec<SourceBinding> = self
            .inputs
            .iter()
            .map(|(role, object)| {
                Ok(SourceBinding {
                    role: role.clone(),
                    source: document.stamp(*object)?,
                    selection: Selection::Whole,
                })
            })
            .collect::<Result<_>>()?;
        let input_ports: Vec<FeaturePort> = self
            .inputs
            .iter()
            .map(|(role, _)| FeaturePort::geometry(role.clone(), &[self.kind.input_kind()]))
            .collect();
        let output_port = FeaturePort::geometry("result", &[preview.geometry.kind()]);
        let recipe = Recipe::new(
            document.id,
            self.kind.command(),
            bindings.clone(),
            vec![self.output],
            ReplayEnvironment {
                command_version: 1,
                solver: "om9-builder".into(),
                solver_version: env!("CARGO_PKG_VERSION").into(),
                schema_version: 1,
                length_unit: Unit::Millimeter,
                frame: Transform::identity(),
                tolerance,
            },
        )
        .with_ports(input_ports, vec![output_port.clone()]);
        let mut recipe = recipe;
        recipe.id = self.recipe;
        recipe.parameters = self.parameters();
        recipe.state = EvaluationState::Clean;
        let published = PublishedResult {
            operation: self.kind.command().into(),
            geometry_revision: entity.geometry_revision,
            geometry_snapshot: preview.geometry.clone(),
            source_stamps: bindings.iter().map(|binding| binding.source).collect(),
            conversion: ConversionReport::identity(preview.geometry.kind()),
            recipe: Some(recipe.id),
            status: ResultStatus::Current,
        };
        Ok(BuilderPlan {
            entity,
            recipe,
            published,
        })
    }

    /// The subshapes the preview published, for a reference binding the host may add.
    pub fn published_subshapes(&self) -> Vec<PublishedSubshapes> {
        match &self.preview {
            Some(preview) => vec![PublishedSubshapes {
                recipe: self.recipe,
                output: self.output,
                revision: 1,
                subshapes: preview.subshapes.clone(),
            }],
            None => vec![],
        }
    }

    fn summary(&self, geometry: &Geometry) -> String {
        match (self.kind, geometry) {
            (BuilderKind::RingRail, Geometry::Curve(Curve::Circle { radius, .. })) => {
                let diameter = radius * 2.0;
                let circumference = diameter * std::f64::consts::PI;
                match self.selected_standard() {
                    Some(standard) => {
                        let size = self
                            .field("us_size")
                            .and_then(BuilderField::numeric)
                            .unwrap_or(0.0);
                        format!(
                            "{} {size:.0} -> diameter {diameter:.2} mm, circumference {circumference:.2} mm; {}",
                            standard.label(),
                            standard.provenance()
                        )
                    }
                    None => {
                        format!("diameter {diameter:.2} mm, circumference {circumference:.2} mm")
                    }
                }
            }
            (BuilderKind::RailProfile, _) => {
                let width = self.field("width_mm").and_then(BuilderField::numeric);
                let height = self.field("height_mm").and_then(BuilderField::numeric);
                let preset = self
                    .selected_preset()
                    .map(ProfilePreset::label)
                    .unwrap_or("Profile");
                let position = self
                    .field("position")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                let roll = self
                    .field("roll_deg")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                match (width, height) {
                    (Some(width), Some(height)) => format!(
                        "{preset} {width:.2} x {height:.2} mm at {position:.2}, roll {roll:.0} deg"
                    ),
                    _ => preset.to_ascii_lowercase(),
                }
            }
            (BuilderKind::OutsideRail, Geometry::Curve(Curve::Circle { radius, .. })) => {
                let offset = self
                    .field("offset_mm")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                format!("outer radius {radius:.2} mm (offset {offset:.2} mm)")
            }
            (BuilderKind::RingResizer, Geometry::Curve(Curve::Circle { radius, .. })) => {
                let thickness = self
                    .field("thickness_mm")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                format!(
                    "inside diameter {:.2} mm, thickness {thickness:.2} mm",
                    radius * 2.0
                )
            }
            (BuilderKind::RailSweep, geometry) => {
                format!(
                    "sweep {}, {} names",
                    geometry.kind().tag(),
                    self.subshapes_count()
                )
            }
            (kind, Geometry::Brep(brep)) if kind.ring_family().is_some() => {
                let diameter = self
                    .field("diameter_mm")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                let stones = self
                    .field("stone_count")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                match kind.ring_family() {
                    Some(crate::ring_builder::RingBuilderKind::Eternity) => format!(
                        "⌀{diameter:.2} mm B-Rep, {stones:.0} stone frames, {} faces",
                        brep.display.faces.len()
                    ),
                    _ => format!(
                        "⌀{diameter:.2} mm B-Rep, {} faces",
                        brep.display.faces.len()
                    ),
                }
            }
            (kind, Geometry::Mesh(mesh)) if kind.ring_family().is_some() => {
                let diameter = self
                    .field("diameter_mm")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                let stones = self
                    .field("stone_count")
                    .and_then(BuilderField::numeric)
                    .unwrap_or(0.0);
                match kind.ring_family() {
                    Some(crate::ring_builder::RingBuilderKind::Eternity) => format!(
                        "⌀{diameter:.2} mm, {stones:.0} stone frames, {} faces",
                        mesh.faces.len()
                    ),
                    _ => format!("⌀{diameter:.2} mm, {} faces", mesh.faces.len()),
                }
            }
            _ => geometry.kind().tag().to_string(),
        }
    }

    fn subshapes_count(&self) -> usize {
        self.preview
            .as_ref()
            .map(|preview| preview.subshapes.len())
            .unwrap_or(0)
    }

    /// One numeric ring parameter, refused with the caller's label when the field is absent.
    fn ring_numeric(&self, name: &str, missing: &'static str) -> Result<f64> {
        self.field(name)
            .and_then(BuilderField::numeric)
            .ok_or(Error::Missing(missing))
    }

    /// Handle positions come from the preview geometry, not from a second copy of the
    /// parameters: a panel edit and a drag therefore paint the same point.
    fn handles(&self, geometry: &Geometry) -> Result<Vec<BuilderHandle>> {
        let mut handles = Vec::new();
        match self.kind {
            BuilderKind::RingRail => {
                let field = self
                    .field("diameter_mm")
                    .ok_or(Error::Missing("builder field"))?;
                let diameter = field
                    .numeric()
                    .ok_or(Error::Invalid("builder numeric field"))?;
                handles.push(BuilderHandle {
                    name: "diameter",
                    field: field.name,
                    label: "Diameter",
                    position: [diameter / 2.0, 0.0, 0.0],
                    axis: [1.0, 0.0, 0.0],
                    value: diameter,
                    min: 1.0,
                    max: 40.0,
                    scale: 1.0,
                });
            }
            BuilderKind::RailProfile => {
                let width = self
                    .field("width_mm")
                    .and_then(BuilderField::numeric)
                    .ok_or(Error::Missing("builder width"))?;
                let height = self
                    .field("height_mm")
                    .and_then(BuilderField::numeric)
                    .ok_or(Error::Missing("builder height"))?;
                let (width_position, width_axis, height_position, height_axis) = match geometry {
                    Geometry::Curve(Curve::Polycurve { .. }) => {
                        let corners = profile_corners(geometry)?;
                        (
                            corners[1],
                            unit(sub(corners[1], corners[0]))?,
                            corners[3],
                            unit(sub(corners[3], corners[0]))?,
                        )
                    }
                    Geometry::Curve(Curve::Circle {
                        center,
                        x,
                        y,
                        radius,
                        ..
                    }) => (
                        offset(*center, *x, *radius),
                        unit(*x)?,
                        offset(*center, *y, *radius),
                        unit(*y)?,
                    ),
                    _ => return Err(Error::Invalid("builder profile geometry")),
                };
                handles.push(BuilderHandle {
                    name: "width",
                    field: "width_mm",
                    label: "Width",
                    position: width_position,
                    axis: width_axis,
                    value: width,
                    min: 0.1,
                    max: 20.0,
                    scale: 1.0,
                });
                handles.push(BuilderHandle {
                    name: "height",
                    field: "height_mm",
                    label: "Height",
                    position: height_position,
                    axis: height_axis,
                    value: height,
                    min: 0.1,
                    max: 20.0,
                    scale: 1.0,
                });
                let (center, normal) = profile_center_and_normal(geometry)?;
                handles.push(BuilderHandle {
                    name: "position",
                    field: "position",
                    label: "Position",
                    position: center,
                    axis: normal,
                    value: self
                        .field("position")
                        .and_then(BuilderField::numeric)
                        .unwrap_or(0.0),
                    min: 0.0,
                    max: 1.0,
                    // One millimetre of drag moves the section one percent along the rail.
                    scale: 0.01,
                });
            }
            BuilderKind::RailSweep => {}
            BuilderKind::OutsideRail => {
                let Geometry::Curve(Curve::Circle {
                    center, x, radius, ..
                }) = geometry
                else {
                    return Err(Error::Invalid("outside rail geometry"));
                };
                let offset_value = self
                    .field("offset_mm")
                    .and_then(BuilderField::numeric)
                    .ok_or(Error::Missing("builder offset"))?;
                handles.push(BuilderHandle {
                    name: "offset",
                    field: "offset_mm",
                    label: "Offset",
                    position: offset(*center, *x, *radius),
                    axis: unit(*x)?,
                    value: offset_value,
                    min: 0.1,
                    max: 50.0,
                    scale: 1.0,
                });
            }
            BuilderKind::RingResizer => {
                let Geometry::Curve(Curve::Circle {
                    center, x, radius, ..
                }) = geometry
                else {
                    return Err(Error::Invalid("ring resizer geometry"));
                };
                let target = self
                    .field("target_diameter_mm")
                    .and_then(BuilderField::numeric)
                    .ok_or(Error::Missing("builder target"))?;
                handles.push(BuilderHandle {
                    name: "target",
                    field: "target_diameter_mm",
                    label: "Target",
                    position: offset(*center, *x, *radius),
                    axis: unit(*x)?,
                    value: target,
                    min: 1.0,
                    max: 60.0,
                    scale: 1.0,
                });
            }
            // The ring family puts one grip per silhouette parameter: the inner diameter on
            // the +X inner surface, the wall on the -X outer surface, the width on the +Z
            // edge and the kind's own pad/ridge/stones where they visibly change the body.
            BuilderKind::EternityBand
            | BuilderKind::SignetRing
            | BuilderKind::RaisedBand
            | BuilderKind::AwardRing => {
                let family = self
                    .kind
                    .ring_family()
                    .ok_or(Error::Invalid("ring builder kind"))?;
                let diameter = self.ring_numeric("diameter_mm", "builder diameter")?;
                let width = self.ring_numeric("width_mm", "builder width")?;
                let thickness = self.ring_numeric("thickness_mm", "builder wall")?;
                let inner = diameter / 2.0;
                let outer = inner + thickness;
                let half = width / 2.0;
                handles.push(BuilderHandle {
                    name: "diameter",
                    field: "diameter_mm",
                    label: "Diameter",
                    position: [inner, 0.0, 0.0],
                    axis: [1.0, 0.0, 0.0],
                    value: diameter,
                    min: 8.0,
                    max: 40.0,
                    // One millimetre of radial drag moves the radius, so the diameter moves by two.
                    scale: 2.0,
                });
                handles.push(BuilderHandle {
                    name: "width",
                    field: "width_mm",
                    label: "Width",
                    position: [0.0, outer, half],
                    axis: [0.0, 0.0, 1.0],
                    value: width,
                    min: 1.0,
                    max: 20.0,
                    // The band grows on both sides of its mid plane.
                    scale: 2.0,
                });
                handles.push(BuilderHandle {
                    name: "wall",
                    field: "thickness_mm",
                    label: "Wall",
                    position: [-outer, 0.0, 0.0],
                    axis: [-1.0, 0.0, 0.0],
                    value: thickness,
                    min: crate::ring_builder::MIN_WALL_MM,
                    max: 10.0,
                    scale: 1.0,
                });
                match family {
                    crate::ring_builder::RingBuilderKind::Eternity => {
                        let stones = self.ring_numeric("stone_diameter_mm", "builder stone")?;
                        handles.push(BuilderHandle {
                            name: "stone",
                            field: "stone_diameter_mm",
                            label: "Stone",
                            position: [outer, 0.0, 0.0],
                            axis: [1.0, 0.0, 0.0],
                            value: stones,
                            min: 0.1,
                            max: 10.0,
                            scale: 1.0,
                        });
                    }
                    crate::ring_builder::RingBuilderKind::Raised => {
                        let raised = self.ring_numeric("raised_height_mm", "builder ridge")?;
                        handles.push(BuilderHandle {
                            name: "raised",
                            field: "raised_height_mm",
                            label: "Ridge",
                            position: [outer + raised, 0.0, 0.0],
                            axis: [1.0, 0.0, 0.0],
                            value: raised,
                            min: 0.1,
                            max: 10.0,
                            scale: 1.0,
                        });
                    }
                    crate::ring_builder::RingBuilderKind::Signet
                    | crate::ring_builder::RingBuilderKind::Award => {
                        let shoulder = self.ring_numeric("shoulder_mm", "builder shoulder")?;
                        let top_width = self.ring_numeric("top_width_mm", "builder top width")?;
                        let top_length =
                            self.ring_numeric("top_length_mm", "builder top length")?;
                        let pad_outer = outer + shoulder;
                        let pad_half = half + ((top_width - width) / 2.0).max(0.0);
                        handles.push(BuilderHandle {
                            name: "shoulder",
                            field: "shoulder_mm",
                            label: "Shoulder",
                            position: [0.0, pad_outer, 0.0],
                            axis: [0.0, 1.0, 0.0],
                            value: shoulder,
                            min: crate::ring_builder::MIN_WALL_MM,
                            max: 10.0,
                            scale: 1.0,
                        });
                        handles.push(BuilderHandle {
                            name: "top_width",
                            field: "top_width_mm",
                            label: "Top Width",
                            position: [pad_outer, 0.0, pad_half],
                            axis: [0.0, 0.0, 1.0],
                            value: top_width,
                            min: 1.0,
                            max: 20.0,
                            scale: 2.0,
                        });
                        // The pad arc ends where the shoulders meet the band again.
                        let arc = (top_length / 2.0) / outer;
                        let (sin, cos) = arc.sin_cos();
                        handles.push(BuilderHandle {
                            name: "top_length",
                            field: "top_length_mm",
                            label: "Top Length",
                            position: [pad_outer * cos, pad_outer * sin, 0.0],
                            axis: [-sin, cos, 0.0],
                            value: top_length,
                            min: 1.0,
                            max: 30.0,
                            scale: 2.0,
                        });
                    }
                }
            }
        }
        Ok(handles)
    }
}

/// The panel fields of one ring builder mode, in the order the panel paints them.
///
/// The defaults come from [`crate::ring_builder::RingParams`], so the panel, the recipe
/// parameters and the producer read one definition of the family (`OM9-L2-04-03`).
fn ring_fields(kind: BuilderKind) -> Result<Vec<BuilderField>> {
    use crate::units::LengthMm;
    let family = kind
        .ring_family()
        .ok_or(Error::Invalid("ring builder kind"))?;
    let defaults = family.defaults();
    let mut fields = vec![
        BuilderField::length(
            "diameter_mm",
            "Diameter",
            Value::length(LengthMm::try_new(defaults.diameter_mm)?),
            8.0,
            40.0,
        ),
        BuilderField::length(
            "width_mm",
            "Width",
            Value::length(LengthMm::try_new(defaults.width_mm)?),
            1.0,
            20.0,
        ),
        BuilderField::length(
            "thickness_mm",
            "Wall",
            Value::length(LengthMm::try_new(defaults.thickness_mm)?),
            crate::ring_builder::MIN_WALL_MM,
            10.0,
        ),
    ];
    match family {
        crate::ring_builder::RingBuilderKind::Eternity => {
            fields.push(BuilderField::count(
                "stone_count",
                "Stones",
                defaults.stone_count,
                1,
                60,
            ));
            fields.push(BuilderField::length(
                "stone_diameter_mm",
                "Stone",
                Value::length(LengthMm::try_new(defaults.stone_diameter_mm)?),
                0.1,
                10.0,
            ));
        }
        crate::ring_builder::RingBuilderKind::Raised => {
            fields.push(BuilderField::length(
                "raised_height_mm",
                "Raised",
                Value::length(LengthMm::try_new(defaults.raised_height_mm)?),
                0.1,
                10.0,
            ));
            fields.push(BuilderField::length(
                "raised_width_mm",
                "Ridge",
                Value::length(LengthMm::try_new(defaults.raised_width_mm)?),
                0.1,
                20.0,
            ));
        }
        crate::ring_builder::RingBuilderKind::Signet
        | crate::ring_builder::RingBuilderKind::Award => {
            fields.push(BuilderField::length(
                "top_width_mm",
                "Top width",
                Value::length(LengthMm::try_new(defaults.top_width_mm)?),
                1.0,
                20.0,
            ));
            fields.push(BuilderField::length(
                "top_length_mm",
                "Top length",
                Value::length(LengthMm::try_new(defaults.top_length_mm)?),
                1.0,
                30.0,
            ));
            fields.push(BuilderField::length(
                "shoulder_mm",
                "Shoulder",
                Value::length(LengthMm::try_new(defaults.shoulder_mm)?),
                crate::ring_builder::MIN_WALL_MM,
                10.0,
            ));
        }
    }
    Ok(fields)
}

/// A numeric value carrying the same dimension as `field`, so a slider or drag on a ratio,
/// angle or count does not turn into a millimetre quantity.
fn value_like(field: &BuilderField, value: f64) -> Value {
    match &field.value {
        Value::Quantity { dimension, .. } => Value::Quantity {
            value,
            dimension: *dimension,
        },
        _ => Value::Quantity {
            value,
            dimension: Dimension::Length,
        },
    }
}

/// The section centre and the plane normal (the rail tangent) for the profile position handle.
fn profile_center_and_normal(geometry: &Geometry) -> Result<(Point3, Point3)> {
    match geometry {
        Geometry::Curve(Curve::Circle { center, x, y, .. }) => Ok((*center, unit(cross(*x, *y))?)),
        Geometry::Curve(Curve::Polycurve { .. }) => {
            let corners = profile_corners(geometry)?;
            let center = [
                (corners[0][0] + corners[1][0] + corners[2][0] + corners[3][0]) / 4.0,
                (corners[0][1] + corners[1][1] + corners[2][1] + corners[3][1]) / 4.0,
                (corners[0][2] + corners[1][2] + corners[2][2] + corners[3][2]) / 4.0,
            ];
            Ok((
                center,
                unit(cross(
                    sub(corners[1], corners[0]),
                    sub(corners[3], corners[0]),
                ))?,
            ))
        }
        _ => Err(Error::Invalid("builder profile geometry")),
    }
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn offset(center: Point3, axis: Point3, scale: f64) -> Point3 {
    [
        center[0] + axis[0] * scale,
        center[1] + axis[1] * scale,
        center[2] + axis[2] * scale,
    ]
}

fn unit(vector: Point3) -> Result<Point3> {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("builder handle axis"));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}

/// The four rectangle corners of the profile polycurve, in creation order.
fn profile_corners(geometry: &Geometry) -> Result<[Point3; 4]> {
    let Geometry::Curve(Curve::Polycurve { segments, .. }) = geometry else {
        return Err(Error::Invalid("builder profile geometry"));
    };
    if segments.len() < 4 {
        return Err(Error::Invalid("builder profile segments"));
    }
    let mut corners = [[0.0; 3]; 4];
    for (index, corner) in corners.iter_mut().enumerate() {
        let Curve::Line { start, .. } = &segments[index] else {
            return Err(Error::Invalid("builder profile segment"));
        };
        *corner = *start;
    }
    Ok(corners)
}

/// The producer behind a builder command, so a panel label can name what will run.
pub fn producer_name(kind: BuilderKind) -> &'static str {
    match kind {
        BuilderKind::RingRail => "RingRailProducer",
        BuilderKind::RailProfile => "RailProfileProducer",
        BuilderKind::RailSweep => "RailSweepProducer",
        BuilderKind::OutsideRail => "OutsideRailProducer",
        BuilderKind::RingResizer => "RingResizerProducer",
        BuilderKind::EternityBand => "EternityBandProducer",
        BuilderKind::SignetRing => "SignetRingProducer",
        BuilderKind::RaisedBand => "RaisedBandProducer",
        BuilderKind::AwardRing => "AwardRingProducer",
    }
}

/// Register the analytic builder chain (the OCCT registry replaces the sweep backend).
pub fn register_reference_producers(registry: &mut ProducerRegistry) -> Result<()> {
    registry.register(RING_RAIL_COMMAND, crate::producers::RingRailProducer)?;
    registry.register(RAIL_PROFILE_COMMAND, crate::producers::RailProfileProducer)?;
    registry.register(
        RAIL_SWEEP_COMMAND,
        crate::producers::RailSweepProducer::new(crate::producers::SurfaceSweepBackend),
    )?;
    registry.register(OUTSIDE_RAIL_COMMAND, crate::producers::OutsideRailProducer)?;
    registry.register(RING_RESIZER_COMMAND, crate::producers::RingResizerProducer)?;
    for family in crate::ring_builder::RingBuilderKind::ALL {
        registry.register(
            family.command(),
            crate::producers::RingFamilyProducer { kind: family },
        )?;
    }
    Ok(())
}

/// A field name/value list for a panel that edits by index.
pub fn field_values(session: &BuilderSession) -> BTreeMap<&'static str, Value> {
    session
        .fields()
        .iter()
        .map(|field| (field.name, field.value.clone()))
        .collect()
}
