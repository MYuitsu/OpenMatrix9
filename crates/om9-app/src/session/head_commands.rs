// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Head Builder, Bezel Builder and Scallop Bezel commands (`OM9-L2-04-14`).
//!
//! The three Builder icons read the selected gem's `om9.gem` record and its girdle outline, and
//! commit through the host transaction so Undo restores the previous document. The commands are
//! staged: the icon builds a preview and publishes the clickable options; the user edits values
//! and presses Enter (commit) or Cancel (discard).
//!
//! This owner publishes only *head* settings. The prong/metal/bead family is `OM9-L2-04-15`;
//! both share the one gem record the gems owner publishes and never keep a second gem store.
use om9_core::{
    Error, Result,
    gem::gem_record_in,
    geometry::{Geometry, Mesh},
    model::{
        Entity, Extension, SETTINGS_BEZEL_SLOT, SETTINGS_HEAD_SEAT_SLOT, SETTINGS_HEAD_SLOT,
        SETTINGS_SCALLOP_SLOT,
    },
    settings::{
        bezel::{
            BezelMode, BezelParams, BezelSetting, ScallopParams, ScallopSetting,
            build as build_bezel, build_scallop,
        },
        head::{
            GemGirdle, HeadEdit, HeadElement, HeadFeature, HeadField, HeadLevel, HeadMode,
            HeadParams, HeadSetting, HeadUpdate, Profile, ProfileKind, ProfileLibrary, SeatFrame,
            SeatParams, apply_edit as apply_head_edit, build as build_head, check as check_head,
            cut_seats, seat_frames, update as update_head,
        },
    },
    transaction::Mutation,
};

use super::measure::{MeasureUnit, Measurement};
use super::*;

/// `Setting > Head Builder` (`gvHeadBuilder`).
pub const HEAD_BUILDER: &str = "BuilderHead";
/// `Setting > Bezel Builder` (`gvBezelBuilder`).
pub const BEZEL_BUILDER: &str = "BuilderBezel";
/// `Setting > Scallop Bezel` (`gvBezelCutter`).
pub const BEZEL_CUTTER: &str = "BuilderBezelCutter";

const PAYLOAD_VERSION: u32 = 1;

/// One head/bezel command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadTool {
    HeadBuilder,
    BezelBuilder,
    BezelCutter,
}

impl HeadTool {
    pub const ALL: [(&'static str, HeadTool); 3] = [
        (HEAD_BUILDER, HeadTool::HeadBuilder),
        (BEZEL_BUILDER, HeadTool::BezelBuilder),
        (BEZEL_CUTTER, HeadTool::BezelCutter),
    ];

    pub fn feature(self) -> &'static str {
        match self {
            Self::HeadBuilder => HEAD_BUILDER,
            Self::BezelBuilder => BEZEL_BUILDER,
            Self::BezelCutter => BEZEL_CUTTER,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HeadBuilder => "Head Builder",
            Self::BezelBuilder => "Bezel Builder",
            Self::BezelCutter => "Scallop Bezel",
        }
    }

    pub fn from_feature(feature: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|(id, _)| *id == feature)
            .map(|(_, tool)| tool)
    }
}

/// Whether this module owns the feature id.
pub fn handles_command(feature: &str) -> bool {
    HeadTool::from_feature(feature).is_some()
}

/// Typed command aliases that reach the same owner.
pub fn from_alias(text: &str) -> Option<&'static str> {
    let wanted = text.trim().to_ascii_lowercase();
    let wanted = match wanted.as_str() {
        "headbuilder" | "head builder" | "heads" | "gvheadbuilder" => HEAD_BUILDER,
        "bezelbuilder" | "bezel builder" | "bezel" | "gvbezelbuilder" => BEZEL_BUILDER,
        "scallopbezel" | "scallop bezel" | "bezelcutter" | "bezel cutter" | "gvbezeltcutter"
        | "gvbezelcutter" => BEZEL_CUTTER,
        _ => return None,
    };
    Some(wanted)
}

/// The payload every committed head body carries: the gem dependency and the typed builder state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HeadPayload {
    pub version: u32,
    pub feature_id: String,
    pub gem_id: String,
    pub params: HeadParams,
    /// The level the head was last edited at, for the next edit session.
    pub level: HeadLevel,
    /// Seats cut into the stone when the head committed (0 = no seat cut).
    #[serde(default)]
    pub seat_count: u8,
    /// Stone volume the seat cut removed, mm³.
    #[serde(default)]
    pub seat_cut_removed_mm3: f64,
}

impl HeadPayload {
    pub fn new(feature_id: &str, gem_id: &str, params: &HeadParams, level: HeadLevel) -> Self {
        Self {
            version: PAYLOAD_VERSION,
            feature_id: feature_id.to_owned(),
            gem_id: gem_id.to_owned(),
            params: params.clone(),
            level,
            seat_count: 0,
            seat_cut_removed_mm3: 0.0,
        }
    }

    pub fn extension(&self) -> Result<Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("head payload"))?;
        Extension::new(self.version, payload)
    }

    pub fn from_extension(extension: &Extension) -> Option<Self> {
        let payload: Self = serde_json::from_str(&extension.payload).ok()?;
        (payload.version == PAYLOAD_VERSION).then_some(payload)
    }
}

/// The payload every committed bezel body carries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BezelPayload {
    pub version: u32,
    pub feature_id: String,
    pub gem_id: String,
    pub params: BezelParams,
}

impl BezelPayload {
    pub fn new(feature_id: &str, gem_id: &str, params: &BezelParams) -> Self {
        Self {
            version: PAYLOAD_VERSION,
            feature_id: feature_id.to_owned(),
            gem_id: gem_id.to_owned(),
            params: params.clone(),
        }
    }

    pub fn extension(&self) -> Result<Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("bezel payload"))?;
        Extension::new(self.version, payload)
    }

    pub fn from_extension(extension: &Extension) -> Option<Self> {
        let payload: Self = serde_json::from_str(&extension.payload).ok()?;
        (payload.version == PAYLOAD_VERSION).then_some(payload)
    }
}

/// The payload every committed scallop cutter carries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScallopPayload {
    pub version: u32,
    pub feature_id: String,
    pub gem_id: String,
    pub params: ScallopParams,
}

impl ScallopPayload {
    pub fn new(feature_id: &str, gem_id: &str, params: &ScallopParams) -> Self {
        Self {
            version: PAYLOAD_VERSION,
            feature_id: feature_id.to_owned(),
            gem_id: gem_id.to_owned(),
            params: params.clone(),
        }
    }

    pub fn extension(&self) -> Result<Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("scallop payload"))?;
        Extension::new(self.version, payload)
    }

    pub fn from_extension(extension: &Extension) -> Option<Self> {
        let payload: Self = serde_json::from_str(&extension.payload).ok()?;
        (payload.version == PAYLOAD_VERSION).then_some(payload)
    }
}

/// What a started head/bezel command is waiting to commit.
// The Head variant carries the full staged setting; boxing it would churn every match arm.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum HeadDraft {
    Head {
        girdle: GemGirdle,
        /// The gem body the seat cut is applied to on commit.
        gem_object: om9_core::identity::ObjectId,
        feature: HeadFeature,
        setting: HeadSetting,
        seats: Vec<SeatFrame>,
        seat_params: SeatParams,
        level: HeadLevel,
        element: Option<HeadElement>,
    },
    Bezel {
        girdle: GemGirdle,
        params: BezelParams,
        bezel: BezelSetting,
    },
    Scallop {
        girdle: GemGirdle,
        params: ScallopParams,
        scallop: ScallopSetting,
    },
}

/// A staged head/bezel command.
#[derive(Debug, Clone)]
pub struct HeadPending {
    pub tool: HeadTool,
    pub draft: HeadDraft,
    pub message: String,
}

impl HeadPending {
    pub fn prompt(&self) -> &'static str {
        match self.tool {
            HeadTool::HeadBuilder => {
                "Mode=, Prongs=, Profile=, Rails=, Height=, Drop=, Level=, or Enter"
            }
            HeadTool::BezelBuilder => {
                "Mode=, Profile=, Height=, Top=, Base=, Angle=, Seat=, Boolean=, or Enter"
            }
            HeadTool::BezelCutter => "Profile=, Count=, Angle=, Offset=, Depth=, Dims=on, or Enter",
        }
    }
}

fn key_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let (name, value) = text.split_once('=')?;
    if name.trim().eq_ignore_ascii_case(key) {
        Some(value.trim())
    } else {
        None
    }
}

fn number(text: &str, key: &str) -> Result<Option<f64>> {
    let Some(value) = key_value(text, key) else {
        return Ok(None);
    };
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(Some)
        .ok_or(Error::Invalid("head numeric option"))
}

fn count(text: &str, key: &str) -> Result<Option<u8>> {
    let Some(value) = key_value(text, key) else {
        return Ok(None);
    };
    value
        .parse::<u8>()
        .ok()
        .map(Some)
        .ok_or(Error::Invalid("head count option"))
}

fn yes_no_value(text: &str, key: &str) -> Result<Option<bool>> {
    let Some(value) = key_value(text, key) else {
        return Ok(None);
    };
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" => Ok(Some(true)),
        "no" | "false" | "off" => Ok(Some(false)),
        _ => Err(Error::Invalid("head yes/no option")),
    }
}

/// Parse a `Bezier:x,y;x,y;...` profile option into a sampled section. Any other value is a
/// profile name and returns `None`, so the caller can fall back to the library.
fn bezier_section(value: &str) -> Result<Option<Vec<(f64, f64)>>> {
    let Some((head, tail)) = value.split_once(':') else {
        return Ok(None);
    };
    if !head.trim().eq_ignore_ascii_case("bezier") {
        return Ok(None);
    }
    let mut control = Vec::new();
    for pair in tail.split(';') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (x, y) = pair
            .split_once(',')
            .ok_or(Error::Missing("profile bezier point"))?;
        let x: f64 = x
            .trim()
            .parse()
            .map_err(|_| Error::Missing("profile bezier point"))?;
        let y: f64 = y
            .trim()
            .parse()
            .map_err(|_| Error::Missing("profile bezier point"))?;
        control.push((x, y));
    }
    Ok(Some(Profile::from_bezier("Bezier", &control, 64)?.points))
}

/// Resolve a name against the session's user profiles first, then the caller falls back to the
/// built-in library.
fn user_profile(profiles: &[Profile], name: &str) -> Option<Vec<(f64, f64)>> {
    profiles
        .iter()
        .find(|profile| profile.name.eq_ignore_ascii_case(name.trim()))
        .map(|profile| profile.points.clone())
}

/// Save the pending section under a name in the session's editable profile library.
fn save_profile(
    profiles: &mut Vec<Profile>,
    name: &str,
    section: Option<Vec<(f64, f64)>>,
    kind: Option<Vec<(f64, f64)>>,
) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Missing("profile name"));
    }
    let points = section.or(kind).ok_or(Error::Missing("profile section"))?;
    let profile = Profile::from_section(name, &points)?;
    profiles.retain(|saved| !saved.name.eq_ignore_ascii_case(name));
    profiles.push(profile);
    Ok(())
}

impl Controller {
    pub fn head_pending(&self) -> Option<&HeadPending> {
        self.head_pending.as_ref()
    }

    /// The session's editable profile library: the Bezier profiles saved with `ProfileSave=`.
    pub fn head_profile_names(&self) -> Vec<String> {
        self.head_profiles
            .iter()
            .map(|profile| profile.name.clone())
            .collect()
    }

    /// The gem the head builder must be built on: exactly one selected gem record.
    fn head_gem_from_selection(&self) -> Result<(GemGirdle, om9_core::identity::ObjectId)> {
        for id in self.selected_ordered() {
            let Some(entity) = self.document().entities.get(&id) else {
                continue;
            };
            let Some(record) = gem_record_in(self.document(), id) else {
                continue;
            };
            let mut girdle = GemGirdle::from_spec(&record.spec, &entity.placement)?;
            girdle.gem_id = format!("gem:{}", record.gem_id.object());
            let girdle = {
                girdle.validate()?;
                girdle
            };
            return Ok((girdle, id));
        }
        Err(Error::Missing("head gem selection"))
    }

    pub fn dispatch_head_tool(&mut self, feature: &str) -> Result<()> {
        let tool = HeadTool::from_feature(feature).ok_or(Error::Unsupported("head command"))?;
        if self.head_pending.is_some() || self.settings_pending.is_some() || self.is_running() {
            return Err(Error::Busy);
        }
        let (girdle, gem_object) = self.head_gem_from_selection()?;
        let sequence = self.head_sequence.wrapping_add(1);
        let draft = match tool {
            HeadTool::HeadBuilder => {
                let mut params = HeadParams::basket(4);
                // Builder defaults follow the stone: the head base must stay above the culet and
                // the rails must hang outside the pavilion cone.
                params.head_drop_mm = (girdle.pavilion_depth_mm * 0.35).clamp(0.05, 0.9);
                params.rail_drop_mm =
                    (params.head_drop_mm + 0.5).min(girdle.pavilion_depth_mm * 0.9);
                let feature = HeadFeature::new(
                    format!("{HEAD_BUILDER}-{sequence:04}"),
                    girdle.gem_id.clone(),
                    params,
                );
                let setting = build_head(&girdle, &feature.params)?;
                let seat_params = SeatParams {
                    count: feature.params.prongs,
                    ..SeatParams::default()
                };
                let seats = seat_frames(&girdle, &seat_params)?;
                HeadDraft::Head {
                    girdle,
                    gem_object,
                    feature,
                    setting,
                    seats,
                    seat_params,
                    level: HeadLevel::Overall,
                    element: None,
                }
            }
            HeadTool::BezelBuilder => {
                let params = BezelParams::default();
                let bezel = build_bezel(&girdle, &params)?;
                HeadDraft::Bezel {
                    girdle,
                    params,
                    bezel,
                }
            }
            HeadTool::BezelCutter => {
                let params = ScallopParams::default();
                let scallop = build_scallop(&girdle, &params)?;
                HeadDraft::Scallop {
                    girdle,
                    params,
                    scallop,
                }
            }
        };
        let mut staged = HeadPending {
            tool,
            draft,
            message: String::new(),
        };
        staged.message = head_message(&staged);
        let message = staged.message.clone();
        self.head_pending = Some(staged);
        self.report_measurement(Measurement::with_text(
            tool.feature(),
            0.0,
            MeasureUnit::Count,
            message,
        ));
        Ok(())
    }

    /// The clickable option row of a staged head/bezel command.
    pub fn head_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.head_pending else {
            return Vec::new();
        };
        match (pending.tool, &pending.draft) {
            (HeadTool::HeadBuilder, HeadDraft::Head { level, .. }) => match level {
                HeadLevel::Overall => vec![
                    ("Mode=", "Mode="),
                    ("Prongs=", "Prongs="),
                    ("Profile=", "Profile="),
                    ("ProfileSave=", "ProfileSave="),
                    ("Rails=", "Rails="),
                    ("Height=", "Height="),
                    ("Drop=", "Drop="),
                    ("SeatDepth=", "SeatDepth="),
                    ("Level=", "Level="),
                    ("Enter", ""),
                    ("Cancel", ""),
                ],
                HeadLevel::Individual => vec![
                    ("Level=", "Level="),
                    ("Prong=", "Prong="),
                    ("Rail=", "Rail="),
                    ("Size=", "Size="),
                    ("RailDrop=", "RailDrop="),
                    ("RailSize=", "RailSize="),
                    ("Enter", ""),
                    ("Cancel", ""),
                ],
                HeadLevel::Grouped | HeadLevel::Ends => vec![
                    ("Level=", "Level="),
                    ("Size=", "Size="),
                    ("RailDrop=", "RailDrop="),
                    ("RailSize=", "RailSize="),
                    ("RailOffset=", "RailOffset="),
                    ("Enter", ""),
                    ("Cancel", ""),
                ],
            },
            (HeadTool::BezelBuilder, _) => vec![
                ("Mode=", "Mode="),
                ("Profile=", "Profile="),
                ("ProfileSave=", "ProfileSave="),
                ("Height=", "Height="),
                ("Top=", "Top="),
                ("Base=", "Base="),
                ("Seat=", "Seat="),
                ("Shear=", "Shear="),
                ("Boolean=", "Boolean="),
                ("Enter", ""),
                ("Cancel", ""),
            ],
            (HeadTool::BezelCutter, _) => vec![
                ("Profile=", "Profile="),
                ("ProfileSave=", "ProfileSave="),
                ("Count=", "Count="),
                ("Angle=", "Angle="),
                ("Offset=", "Offset="),
                ("Depth=", "Depth="),
                ("Enter", ""),
                ("Cancel", ""),
            ],
            _ => vec![("Enter", ""), ("Cancel", "")],
        }
    }

    pub fn submit_head_text(&mut self, text: &str) -> Option<Result<()>> {
        self.head_pending.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower == "cancel" || lower == "esc" {
            self.head_pending = None;
            return Some(Ok(()));
        }
        if lower == "enter" || trimmed.is_empty() {
            return Some(self.commit_head_pending());
        }
        let result = self.apply_head_option(trimmed);
        Some(result.map(|()| {
            if let Some(pending) = &self.head_pending {
                self.report_measurement(Measurement::with_text(
                    pending.tool.feature(),
                    0.0,
                    MeasureUnit::Count,
                    pending.message.clone(),
                ));
            }
        }))
    }

    fn apply_head_option(&mut self, text: &str) -> Result<()> {
        let original = self
            .head_pending
            .take()
            .ok_or(Error::Missing("head settings session"))?;
        let mut pending = original.clone();
        match self.apply_option_to_head(&mut pending, text) {
            Ok(()) => {
                pending.message = head_message(&pending);
                self.head_pending = Some(pending);
                Ok(())
            }
            Err(error) => {
                // A refused value keeps the command and its last valid preview alive.
                self.head_pending = Some(original);
                Err(error)
            }
        }
    }

    fn apply_option_to_head(&mut self, pending: &mut HeadPending, text: &str) -> Result<()> {
        match &mut pending.draft {
            HeadDraft::Head {
                girdle,
                gem_object: _,
                feature,
                setting,
                seats,
                seat_params,
                level,
                element,
            } => {
                if let Some(value) = key_value(text, "mode") {
                    feature.params.mode = match value.to_ascii_lowercase().as_str() {
                        "basket" | "prong" | "prongs" => HeadMode::Basket,
                        "halfbezel" | "half bezel" | "half-bezel" | "bezel" => HeadMode::HalfBezel,
                        _ => return Err(Error::Invalid("head mode option")),
                    };
                }
                if let Some(value) = count(text, "prongs")? {
                    feature.params.prongs = value;
                }
                if let Some(value) = key_value(text, "profile") {
                    match bezier_section(value)? {
                        Some(points) => {
                            feature.params.prong_profile = "Bezier".to_string();
                            feature.params.profile_section = Some(points);
                        }
                        None => {
                            feature.params.prong_profile = value.to_owned();
                            feature.params.profile_section =
                                user_profile(&self.head_profiles, value);
                        }
                    }
                }
                if let Some(value) = key_value(text, "profilesave") {
                    let fallback =
                        ProfileLibrary::find(ProfileKind::Prong, &feature.params.prong_profile)
                            .ok()
                            .map(|profile| profile.points);
                    save_profile(
                        &mut self.head_profiles,
                        value,
                        feature.params.profile_section.clone(),
                        fallback,
                    )?;
                }
                if let Some(value) = number(text, "seatdepth")? {
                    seat_params.depth_mm = value;
                }
                if let Some(value) = key_value(text, "railprofile") {
                    feature.params.rail_profile = value.to_owned();
                }
                if let Some(value) = count(text, "rails")? {
                    feature.params.rail_count = value;
                }
                if let Some(value) = number(text, "span")? {
                    feature.params.half_bezel.span_deg = value;
                }
                if let Some(value) = number(text, "position")? {
                    feature.params.half_bezel.position_deg = value;
                }
                if let Some(value) = number(text, "thickness")? {
                    feature.params.half_bezel.thickness_mm = value;
                }
                if let Some(value) = number(text, "bezelheight")? {
                    feature.params.half_bezel.height_mm = value;
                }
                if let Some(value) = number(text, "seatangle")? {
                    feature.params.half_bezel.seat_angle_deg = value;
                    seat_params.seat_angle_deg = value;
                }
                if let Some(value) = count(text, "seats")? {
                    seat_params.count = value;
                }
                if let Some(value) = count(text, "level")? {
                    *level = HeadLevel::from_number(value)?;
                    if *level != HeadLevel::Individual {
                        *element = None;
                    }
                }
                if let Some(value) = count(text, "prong")? {
                    if value == 0 {
                        return Err(Error::Invalid("head prong index"));
                    }
                    *element = Some(HeadElement::Prong(value as usize - 1));
                }
                if let Some(value) = count(text, "rail")? {
                    if value == 0 {
                        return Err(Error::Invalid("head rail index"));
                    }
                    *element = Some(HeadElement::Rail(value as usize - 1));
                }
                let mut edited = false;
                for (key, field) in [
                    ("height", HeadField::HeightAboveGirdle),
                    ("headdrop", HeadField::HeadDrop),
                    ("size", HeadField::ProngSize),
                    ("prongangle", HeadField::ProngAngle),
                    ("railDrop", HeadField::RailDrop),
                    ("railsize", HeadField::RailSize),
                    ("railoffset", HeadField::RailOffset),
                ] {
                    if let Some(value) = number(text, &key.to_ascii_lowercase())? {
                        let edit = HeadEdit {
                            level: *level,
                            element: *element,
                            field,
                            value,
                        };
                        apply_head_edit(setting, girdle, &mut feature.params, &edit)?;
                        edited = true;
                    }
                }
                if !edited {
                    // A structural option changed every prong/rail, so rebuild the whole preview.
                    setting_apply(feature, girdle, setting)?;
                }
                *seats = seat_frames(girdle, seat_params)?;
                Ok(())
            }
            HeadDraft::Bezel {
                girdle,
                params,
                bezel,
            } => {
                if let Some(value) = key_value(text, "mode") {
                    params.mode = BezelMode::from_tag(value)?;
                }
                if let Some(value) = key_value(text, "profile") {
                    match bezier_section(value)? {
                        Some(points) => {
                            params.edge_profile = "Bezier".to_string();
                            params.section_points = Some(points);
                        }
                        None => {
                            params.edge_profile = value.to_owned();
                            params.section_points = user_profile(&self.head_profiles, value);
                        }
                    }
                }
                if let Some(value) = key_value(text, "profilesave") {
                    let fallback = ProfileLibrary::find(ProfileKind::Edge, &params.edge_profile)
                        .ok()
                        .map(|profile| profile.points);
                    save_profile(
                        &mut self.head_profiles,
                        value,
                        params.section_points.clone(),
                        fallback,
                    )?;
                }
                if let Some(value) = number(text, "height")? {
                    params.height_mm = value;
                }
                if let Some(value) = number(text, "top")? {
                    params.top_mm = value;
                }
                if let Some(value) = number(text, "base")? {
                    params.base_mm = value;
                }
                if let Some(value) = number(text, "dome")? {
                    params.dome_mm = value;
                }
                if let Some(value) = number(text, "angle")? {
                    params.angle_deg = value;
                }
                if let Some(value) = number(text, "seat")? {
                    params.seat_angle_deg = value;
                }
                if let Some(value) = number(text, "seatdepth")? {
                    params.seat_depth_mm = value;
                }
                if let Some(value) = number(text, "seatlength")? {
                    params.seat_length_mm = value;
                }
                if let Some(value) = number(text, "placement")? {
                    params.placement_mm = value;
                }
                if let Some(value) = number(text, "scale")? {
                    params.scale = value;
                }
                if let Some(value) = number(text, "chamferx")? {
                    params.chamfer_x_mm = value;
                }
                if let Some(value) = number(text, "chamferz")? {
                    params.chamfer_z_mm = value;
                }
                if let Some(value) = yes_no_value(text, "shear")? {
                    params.shear_rails = value;
                }
                if let Some(value) = yes_no_value(text, "boolean")? {
                    params.boolean = value;
                }
                *bezel = build_bezel(girdle, params)?;
                Ok(())
            }
            HeadDraft::Scallop {
                girdle,
                params,
                scallop,
            } => {
                if let Some(value) = key_value(text, "profile") {
                    match bezier_section(value)? {
                        Some(points) => {
                            params.profile = "Bezier".to_string();
                            params.section_points = Some(points);
                        }
                        None => {
                            params.profile = value.to_owned();
                            params.section_points = user_profile(&self.head_profiles, value);
                        }
                    }
                }
                if let Some(value) = key_value(text, "profilesave") {
                    let fallback = ProfileLibrary::find(ProfileKind::Edge, &params.profile)
                        .ok()
                        .map(|profile| profile.points);
                    save_profile(
                        &mut self.head_profiles,
                        value,
                        params.section_points.clone(),
                        fallback,
                    )?;
                }
                if let Some(value) = count(text, "count")? {
                    params.count = value;
                }
                if let Some(value) = number(text, "angle")? {
                    params.angle_deg = value;
                }
                if let Some(value) = number(text, "offset")? {
                    params.offset_mm = value;
                }
                if let Some(value) = number(text, "depth")? {
                    params.depth_mm = value;
                }
                // The dimensional Bezel Cutter Builder (`OM9-L3-04-16-004`): the Cutters
                // chapter's `gvBezelCutter` controls, carried by this shared owner.
                if let Some(value) = yes_no_value(text, "dims")? {
                    params.dims = if value {
                        Some(Default::default())
                    } else {
                        None
                    };
                }
                if let Some(value) = key_value(text, "position") {
                    let mut dims = params
                        .dims
                        .clone()
                        .ok_or(Error::Invalid("bezel cutter position needs Dims=on"))?;
                    dims.position = om9_core::cutters::bezel::CutterPosition::from_tag(value)?;
                    params.dims = Some(dims);
                }
                if params.dims.is_some() {
                    let mut dims = params.dims.clone().unwrap_or_default();
                    if let Some(value) = number(text, "height")? {
                        dims.height_mm = value;
                    }
                    if let Some(value) = number(text, "length")? {
                        dims.length_mm = value;
                    }
                    if let Some(value) = number(text, "scale2d")? {
                        dims.scale_2d_percent = value;
                    }
                    if let Some(value) = number(text, "cutterlength")? {
                        dims.cutter_length_mm = value;
                    }
                    if let Some(value) = number(text, "xinside")? {
                        dims.x_inside_mm = value;
                    }
                    if let Some(value) = number(text, "xoutside")? {
                        dims.x_outside_mm = value;
                    }
                    if let Some(value) = number(text, "yinside")? {
                        dims.y_inside_mm = value;
                    }
                    if let Some(value) = number(text, "youtside")? {
                        dims.y_outside_mm = value;
                    }
                    if let Some(value) = number(text, "cutterangle")? {
                        dims.cutter_angle_deg = value;
                    }
                    if let Some(value) = number(text, "xoffset")? {
                        dims.x_offset_mm = value;
                    }
                    if let Some(value) = number(text, "yoffset")? {
                        dims.y_offset_mm = value;
                    }
                    if let Some(value) = number(text, "zoffset")? {
                        dims.z_offset_mm = value;
                    }
                    if let Some(value) = count(text, "count")? {
                        dims.count = value;
                    }
                    dims.validate()?;
                    params.count = dims.count;
                    params.dims = Some(dims);
                }
                *scallop = build_scallop(girdle, params)?;
                Ok(())
            }
        }
    }

    fn commit_head_pending(&mut self) -> Result<()> {
        let pending = self
            .head_pending
            .take()
            .ok_or(Error::Missing("head settings session"))?;
        self.head_sequence = self.head_sequence.wrapping_add(1);
        let layer = self.document().current_layer;
        let mut mutations = Vec::new();
        let mut volumes: Vec<(String, f64)> = Vec::new();
        let mut cutters: Vec<String> = Vec::new();
        let mut cutter_volumes: Vec<(String, f64)> = Vec::new();
        match &pending.draft {
            HeadDraft::Head {
                girdle,
                gem_object,
                feature,
                setting,
                seat_params,
                level,
                ..
            } => {
                let validity = check_head(setting)?;
                if !validity.closed || !validity.positive_volume {
                    return Err(Error::Invalid("head solid validity"));
                }
                let mut payload = HeadPayload::new(
                    &feature.feature_id,
                    &feature.gem_id,
                    &feature.params,
                    *level,
                );
                // The manual's head cuts the seats into the stone: the gem body is replaced by
                // the seated mesh in the same transaction, so Undo restores the whole document.
                let (gem_mesh, gem_extensions) = match self.document().entities.get(gem_object) {
                    Some(entity) => match &entity.geometry {
                        Geometry::Mesh(mesh) => (mesh.clone(), entity.extensions.clone()),
                        _ => return Err(Error::Invalid("head gem body")),
                    },
                    None => return Err(Error::Missing("head gem body")),
                };
                let seat_cut = cut_seats(&gem_mesh, girdle, seat_params)?;
                if seat_cut.removed_mm3 > 1.0e-9 {
                    payload.seat_count = seat_cut.count() as u8;
                    payload.seat_cut_removed_mm3 = seat_cut.removed_mm3;
                    let source = self.document().stamp(*gem_object)?;
                    let mut extensions = gem_extensions;
                    extensions.insert(SETTINGS_HEAD_SEAT_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Geometry {
                        source,
                        geometry: Geometry::Mesh(seat_cut.seated.clone()),
                    });
                    mutations.push(Mutation::Extensions { source, extensions });
                }
                for prong in &setting.prongs {
                    let mut entity =
                        Entity::new("Prong", Geometry::Mesh(prong.mesh.clone()), layer);
                    entity
                        .extensions
                        .insert(SETTINGS_HEAD_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                for rail in &setting.rails {
                    let mut entity =
                        Entity::new("Gallery Rail", Geometry::Mesh(rail.mesh.clone()), layer);
                    entity
                        .extensions
                        .insert(SETTINGS_HEAD_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                if let Some(band) = &setting.half_bezel {
                    let mut entity =
                        Entity::new("Half Bezel", Geometry::Mesh(band.mesh.clone()), layer);
                    entity
                        .extensions
                        .insert(SETTINGS_HEAD_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                volumes.push((feature.feature_id.clone(), setting.volume_mm3));
            }
            HeadDraft::Bezel {
                girdle,
                params,
                bezel,
            } => {
                let feature_id = format!("{BEZEL_BUILDER}-{:04}", self.head_sequence);
                let payload = BezelPayload::new(&feature_id, &girdle.gem_id, params);
                let mut entity = Entity::new("Bezel", Geometry::Mesh(bezel.mesh.clone()), layer);
                entity
                    .extensions
                    .insert(SETTINGS_BEZEL_SLOT.to_owned(), payload.extension()?);
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity,
                });
                if let Some(rail) = &bezel.finger_rail {
                    let mut entity =
                        Entity::new("Finger Rail", Geometry::Mesh(rail.clone()), layer);
                    entity
                        .extensions
                        .insert(SETTINGS_BEZEL_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                volumes.push((feature_id, bezel.volume_mm3));
            }
            HeadDraft::Scallop {
                girdle,
                params,
                scallop,
            } => {
                let feature_id = format!("{BEZEL_CUTTER}-{:04}", self.head_sequence);
                let payload = ScallopPayload::new(&feature_id, &girdle.gem_id, params);
                for cutter in &scallop.cutters {
                    let mut entity =
                        Entity::new("Scallop Cutter", Geometry::Mesh(cutter.clone()), layer);
                    entity
                        .extensions
                        .insert(SETTINGS_SCALLOP_SLOT.to_owned(), payload.extension()?);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                // A cutter removes metal; it never adds settings mass, but the removed volume is
                // recorded so the mass read-out can report the net metal.
                for (index, volume) in scallop.volumes_mm3.iter().enumerate() {
                    cutter_volumes.push((format!("{feature_id}:{:02}", index + 1), *volume));
                }
                cutters.push(feature_id);
            }
        }
        if mutations.is_empty() {
            return Err(Error::Invalid("head settings commit"));
        }
        self.commit_utility_mutations(pending.tool.label(), mutations)?;
        for (id, volume) in volumes {
            self.settings_volumes.push((id, volume));
        }
        self.settings_cutters.extend(cutters);
        self.settings_cutter_volumes.extend(cutter_volumes);
        self.report_measurement(Measurement::with_text(
            pending.tool.feature(),
            0.0,
            MeasureUnit::Count,
            format!("{} committed", pending.message),
        ));
        Ok(())
    }
}

/// Rebuild the preview after a structural parameter change.
fn setting_apply(
    feature: &mut HeadFeature,
    girdle: &GemGirdle,
    setting: &mut HeadSetting,
) -> Result<()> {
    feature.params.validate()?;
    *setting = update_head(feature, girdle)?.0;
    Ok(())
}

fn head_message(pending: &HeadPending) -> String {
    match &pending.draft {
        HeadDraft::Head {
            feature,
            setting,
            seats,
            seat_params,
            level,
            ..
        } => {
            let ignored = if setting.prong_profile_ignored {
                " · prong profile ignored (half bezel)"
            } else {
                ""
            };
            let seat_note = if seat_params.depth_mm > 0.0 {
                format!(" · seats cut {:.2} mm deep", seat_params.depth_mm)
            } else {
                String::new()
            };
            format!(
                "{} · {} prongs · {} rails · {:.1} mm above girdle · {:.3} mm³ · {} seats · level {}{}{}",
                pending.tool.label(),
                setting.prongs.len(),
                setting.rails.len(),
                feature.params.height_above_girdle_mm,
                setting.volume_mm3,
                seats.len(),
                level.number(),
                ignored,
                seat_note
            )
        }
        HeadDraft::Bezel { params, bezel, .. } => format!(
            "{} · {} · {:.2} mm high · scale {:.2} · {:.3} mm³",
            pending.tool.label(),
            params.mode.label(),
            params.effective_height_mm(),
            params.scale,
            bezel.volume_mm3
        ),
        HeadDraft::Scallop { scallop, .. } => format!(
            "{} · {} cutters · {:.3} mm³ cutter volume",
            pending.tool.label(),
            scallop.count(),
            scallop.total_volume_mm3
        ),
    }
}

/// The mass-report view of a committed head: its parameters and the gem it depends on.
pub fn head_feature_summary(payload: &HeadPayload) -> String {
    format!(
        "{} on {} ({} prongs)",
        payload.params.mode.label(),
        payload.gem_id,
        payload.params.prongs
    )
}

/// The typed update path a future edit session uses: same gem preserves ids, a new gem replaces.
pub fn updated_setting(
    payload: &HeadPayload,
    girdle: &GemGirdle,
) -> Result<(HeadSetting, HeadUpdate)> {
    let feature = HeadFeature::new(
        payload.feature_id.clone(),
        payload.gem_id.clone(),
        payload.params.clone(),
    );
    update_head(&feature, girdle)
}

/// A mesh body of a committed head, used by tests and by the mass check.
pub fn body_mesh(mesh: &Mesh) -> Result<om9_core::settings::SolidReport> {
    om9_core::settings::inspect_solid(mesh)
}
