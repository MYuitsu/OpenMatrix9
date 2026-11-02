// SPDX-License-Identifier: LGPL-2.1-or-later
//! Channel, Micro Prong and Bright Cut builders (`OM9-L2-04-17`).
//!
//! Four commands share one staged session: `BuilderChannelBuilder` sweeps a trench through a row
//! of stones, `BuilderMicroProngCutter` sinks one scallop per gap, `BuilderBrightCutChannel` lays
//! bright facets along the row and `BuilderBrightCutter` rings one stone with bright facets. The
//! Interactive Boolean cuts every selected metal body in the same transaction, and the committed
//! cutters carry `om9.cutter` with the row provenance (`row`: the stone labels in row order).
//!
//! The preview owns nothing: Enter commits one transaction, Cancel and Esc drop it.

use om9_core::{
    Error, Result,
    cutters::{
        boolean::{BooleanMode, CutterCut, CutterKeep, apply as apply_boolean},
        brightcut::{
            BrightParams, build_channel as build_bright_channel, build_single as build_bright_single,
        },
        channel::{ChannelEnd, ChannelMiddle, ChannelParams, build_row as build_channel},
        microprong::{MicroProngParams, build_row as build_microprong},
    },
    gem::gem_record_in,
    geometry::{Geometry, Mesh},
    mesh_csg::CsgOptions,
    model::Entity,
    transaction::Mutation,
};

use super::measure::{MeasureUnit, Measurement};
use super::*;

use crate::session::gem_cutters::{CUTTER_CUT_SLOT, CUTTER_PAYLOAD_VERSION, CutterStone};

/// `BuilderChannelBuilder`: the channel cutter through a row of stones.
pub const CHANNEL_BUILDER: &str = "BuilderChannelBuilder";
/// `BuilderMicroProngCutter`: the scallop row a channel leaves between its stones.
pub const MICRO_PRONG: &str = "BuilderMicroProngCutter";
/// `BuilderBrightCutChannel`: bright facets along a stone row.
pub const BRIGHT_CUT_CHANNEL: &str = "BuilderBrightCutChannel";
/// `BuilderBrightCutter`: bright facets around one stone.
pub const BRIGHT_CUTTER: &str = "BuilderBrightCutter";

/// The four row cutter builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowTool {
    ChannelBuilder,
    MicroProng,
    BrightCutChannel,
    BrightCutter,
}

impl RowTool {
    pub const ALL: [Self; 4] = [
        Self::ChannelBuilder,
        Self::MicroProng,
        Self::BrightCutChannel,
        Self::BrightCutter,
    ];

    pub fn feature(self) -> &'static str {
        match self {
            Self::ChannelBuilder => CHANNEL_BUILDER,
            Self::MicroProng => MICRO_PRONG,
            Self::BrightCutChannel => BRIGHT_CUT_CHANNEL,
            Self::BrightCutter => BRIGHT_CUTTER,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ChannelBuilder => "Channel Builder",
            Self::MicroProng => "Micro Prong Cutter",
            Self::BrightCutChannel => "Bright Cut Channel",
            Self::BrightCutter => "Bright Cutter",
        }
    }

    pub fn transaction_label(self) -> &'static str {
        match self {
            Self::ChannelBuilder => "ChannelBuilder",
            Self::MicroProng => "MicroProngCutter",
            Self::BrightCutChannel => "BrightCutChannel",
            Self::BrightCutter => "BrightCutter",
        }
    }

    /// The `kind` of the `om9.cutter` record.
    pub fn kind(self) -> &'static str {
        match self {
            Self::ChannelBuilder => "channel_cutter",
            Self::MicroProng => "micro_prong_cutter",
            Self::BrightCutChannel => "bright_cut_channel",
            Self::BrightCutter => "bright_cutter",
        }
    }

    pub fn from_feature(feature: &str) -> Option<Self> {
        match feature {
            CHANNEL_BUILDER => Some(Self::ChannelBuilder),
            MICRO_PRONG => Some(Self::MicroProng),
            BRIGHT_CUT_CHANNEL => Some(Self::BrightCutChannel),
            BRIGHT_CUTTER => Some(Self::BrightCutter),
            _ => None,
        }
    }

    /// The typed command names that reach the same handler.
    pub fn command_name(text: &str) -> Option<&'static str> {
        match text.trim().to_ascii_lowercase().replace([' ', '-', '_'], "").as_str() {
            "channelbuilder" | "gvchannelbuilder" | "channelcutter" | "gvchannel" => {
                Some(CHANNEL_BUILDER)
            }
            "microprongcutter" | "gvmicroprongcutter" | "microprong" | "microp" => Some(MICRO_PRONG),
            "brightcutchannel" | "gvbrightcutchannel" | "brightchannel" => {
                Some(BRIGHT_CUT_CHANNEL)
            }
            "brightcutter" | "gvbrightcutter" | "bright" => Some(BRIGHT_CUTTER),
            _ => None,
        }
    }

    pub fn handles(feature: &str) -> bool {
        Self::from_feature(feature).is_some()
    }
}

/// The staged row cutter builder.
#[derive(Debug, Clone)]
pub struct RowCutterSession {
    pub tool: RowTool,
    pub stones: Vec<CutterStone>,
    pub channel: ChannelParams,
    pub microprong: MicroProngParams,
    pub bright: BrightParams,
    pub boolean: BooleanMode,
    pub keep: CutterKeep,
    /// Every selected metal body the Interactive Boolean cuts, in selection order.
    pub targets: Vec<(om9_core::identity::ObjectId, Mesh)>,
    /// The cutter bodies in world space, in commit order.
    pub cutters: Vec<Mesh>,
    /// The cut preview of each target, when the Boolean is on.
    pub cuts: Vec<CutterCut>,
    pub message: String,
}

impl RowCutterSession {
    /// Open one builder for the current selection.
    pub fn open(controller: &Controller, tool: RowTool) -> Result<Self> {
        let selection = controller.selected_ordered();
        let mut stones = Vec::new();
        let mut targets = Vec::new();
        for id in &selection {
            let entity = controller
                .document()
                .entities
                .get(id)
                .ok_or(Error::Stale("cutter selection"))?;
            if let Some(record) = gem_record_in(controller.document(), *id) {
                stones.push(CutterStone {
                    id: *id,
                    // The document label is the row provenance a rebuilder reads; the gem asset
                    // stays in its own record and is not the user's row order.
                    label: entity.label.clone(),
                    spec: record.spec.clone(),
                    placement: entity.placement,
                });
                continue;
            }
            if let Geometry::Mesh(mesh) = &entity.geometry {
                if entity.extension(om9_core::model::CUTTER_EXTENSION_SLOT).is_none() {
                    targets.push((*id, mesh.clone()));
                }
            }
        }
        if stones.is_empty() {
            return Err(Error::Missing("cutter gems"));
        }
        let specs = stones
            .iter()
            .map(|stone| stone.spec.clone())
            .collect::<Vec<_>>();
        let channel = ChannelParams::for_row(&specs)?;
        let microprong = MicroProngParams::for_row(&specs)?;
        let bright = BrightParams::for_gem(&stones[0].spec)?;
        let mut session = Self {
            tool,
            stones,
            channel,
            microprong,
            bright,
            boolean: BooleanMode::Off,
            keep: CutterKeep::Preserve,
            targets,
            cutters: Vec::new(),
            cuts: Vec::new(),
            message: String::new(),
        };
        session.recompute()?;
        Ok(session)
    }

    /// Rebuild the cutters and, when the Boolean is on, the cut preview of every target.
    pub fn recompute(&mut self) -> Result<()> {
        self.cutters.clear();
        self.cuts.clear();
        let stones = self
            .stones
            .iter()
            .map(|stone| {
                (
                    format!("{}:{}", stone.label, stone.id),
                    stone.spec.clone(),
                    stone.placement,
                )
            })
            .collect::<Vec<_>>();
        match self.tool {
            RowTool::ChannelBuilder => {
                self.cutters
                    .push(build_channel(&stones, &self.channel)?.mesh);
            }
            RowTool::MicroProng => {
                let built = build_microprong(&stones, &self.microprong)?;
                self.cutters.extend(built.cutters);
            }
            RowTool::BrightCutChannel => {
                let built = build_bright_channel(&stones, &self.bright)?;
                self.cutters.extend(built.bodies);
            }
            RowTool::BrightCutter => {
                for (label, spec, placement) in &stones {
                    let built = build_bright_single(spec, placement, label, &self.bright)?;
                    self.cutters.extend(built.bodies);
                }
            }
        }
        if self.boolean.cuts() {
            if self.targets.is_empty() {
                return Err(Error::Missing("cutter boolean surface"));
            }
            for (_, mesh) in &self.targets {
                let cut = apply_boolean(
                    mesh,
                    &self.cutters,
                    self.boolean,
                    self.keep,
                    CsgOptions::default(),
                )?;
                self.cuts.push(cut);
            }
        }
        self.message = self.summary();
        Ok(())
    }

    pub fn total_volume_mm3(&self) -> f64 {
        self.cutters
            .iter()
            .map(|mesh| om9_core::cutters::volume_mm3(mesh).unwrap_or(0.0))
            .sum()
    }

    pub fn removed_mm3(&self) -> Option<f64> {
        if self.cuts.is_empty() {
            None
        } else {
            Some(self.cuts.iter().map(|cut| cut.removed_mm3).sum())
        }
    }

    /// The stone labels in row order: the provenance the committed payload carries.
    pub fn row(&self) -> Vec<String> {
        self.stones.iter().map(|stone| stone.label.clone()).collect()
    }

    pub fn summary(&self) -> String {
        let count = self.cutters.len();
        let mut text = format!(
            "{} on {} stone{} · {count} cutter{} · {:.3} mm³",
            self.tool.label(),
            self.stones.len(),
            if self.stones.len() == 1 { "" } else { "s" },
            if count == 1 { "" } else { "s" },
            self.total_volume_mm3()
        );
        if let Some(removed) = self.removed_mm3() {
            text.push_str(&format!(
                " · cut {removed:.3} mm³ on {} body{}",
                self.targets.len(),
                if self.targets.len() == 1 { "" } else { "ies" }
            ));
        } else if self.boolean.cuts() {
            text.push_str(" · no cut preview");
        }
        text
    }

    pub fn prompt(&self) -> &'static str {
        match self.tool {
            RowTool::ChannelBuilder => {
                "Width=, Depth=, Height=, Wall=, Spacing=, Start=, End=, StartDepth=, EndDepth=, Mirror=, Flip=, Middle=, Boolean=, Preserve=, or Enter"
            }
            RowTool::MicroProng => {
                "Radius=, Depth=, Height=, Count=, Spacing=, Angle=, Boolean=, Preserve=, or Enter"
            }
            RowTool::BrightCutChannel => {
                "Height=, Length=, Scale2D=, Angle=, Depth=, Mirror=, Boolean=, Preserve=, or Enter"
            }
            RowTool::BrightCutter => {
                "Height=, Length=, Scale2D=, Count=, Angle=, Depth=, Boolean=, Preserve=, or Enter"
            }
        }
    }

    pub fn options(&self) -> Vec<(&'static str, &'static str)> {
        let mut options: Vec<(&'static str, &'static str)> = match self.tool {
            RowTool::ChannelBuilder => vec![
                ("Width=", "Width="),
                ("Depth=", "Depth="),
                ("Height=", "Height="),
                ("Wall=", "Wall="),
                ("Spacing=", "Spacing="),
                ("Start=", "Start="),
                ("End=", "End="),
                ("StartDepth=", "StartDepth="),
                ("EndDepth=", "EndDepth="),
                ("Mirror=", "Mirror="),
                ("Flip=", "Flip="),
                ("Middle=", "Middle="),
            ],
            RowTool::MicroProng => vec![
                ("Radius=", "Radius="),
                ("Depth=", "Depth="),
                ("Height=", "Height="),
                ("Count=", "Count="),
                ("Spacing=", "Spacing="),
                ("Angle=", "Angle="),
            ],
            RowTool::BrightCutChannel => vec![
                ("Height=", "Height="),
                ("Length=", "Length="),
                ("Scale2D=", "Scale2D="),
                ("Angle=", "Angle="),
                ("Depth=", "Depth="),
                ("Mirror=", "Mirror="),
            ],
            RowTool::BrightCutter => vec![
                ("Height=", "Height="),
                ("Length=", "Length="),
                ("Scale2D=", "Scale2D="),
                ("Count=", "Count="),
                ("Angle=", "Angle="),
                ("Depth=", "Depth="),
            ],
        };
        options.extend([
            ("Boolean=", "Boolean="),
            ("Preserve=", "Preserve="),
            ("Reset", ""),
            ("Enter", ""),
            ("Cancel", ""),
        ]);
        options
    }

    /// Apply one typed option; a refused value keeps the previous preview alive.
    pub fn set_option(&mut self, text: &str) -> Result<()> {
        let original = self.clone();
        if let Err(error) = self.apply_option(text) {
            *self = original;
            return Err(error);
        }
        self.recompute().map_err(|error| {
            *self = original;
            error
        })
    }

    fn apply_option(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower == "reset" {
            self.reset()?;
            return Ok(());
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            return Err(Error::Invalid("cutter option"));
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        let number = |what: &'static str| -> Result<f64> {
            value
                .parse::<f64>()
                .ok()
                .filter(|number| number.is_finite())
                .ok_or(Error::Invalid(what))
        };
        let toggle = |what: &'static str| -> Result<bool> {
            match value.to_ascii_lowercase().as_str() {
                "on" | "yes" | "true" | "1" => Ok(true),
                "off" | "no" | "false" | "0" => Ok(false),
                _ => Err(Error::Invalid(what)),
            }
        };
        let count = |what: &'static str| -> Result<usize> {
            let value = number(what)?;
            if value.fract().abs() > 1.0e-9 || value < 0.0 {
                return Err(Error::Invalid(what));
            }
            Ok(value as usize)
        };
        match name.as_str() {
            "boolean" => {
                self.boolean = BooleanMode::from_tag(value)?;
                if self.boolean.cuts() && self.targets.is_empty() {
                    return Err(Error::Missing("cutter boolean surface"));
                }
            }
            "preserve" => {
                self.keep = CutterKeep::from_tag(value)?;
                if !self.keep.keeps() && !self.boolean.cuts() {
                    return Err(Error::Invalid("cutter preserve needs a boolean"));
                }
            }
            "width" if self.tool == RowTool::ChannelBuilder => {
                self.channel.width_mm = number("channel width")?
            }
            "depth" => match self.tool {
                RowTool::ChannelBuilder => self.channel.depth_mm = number("channel depth")?,
                RowTool::MicroProng => self.microprong.depth_mm = number("microprong depth")?,
                _ => self.bright.depth_mm = number("bright cutter depth")?,
            },
            "height" => match self.tool {
                RowTool::ChannelBuilder => self.channel.height_mm = number("channel height")?,
                RowTool::MicroProng => self.microprong.height_mm = number("microprong height")?,
                _ => self.bright.height_mm = number("bright cutter height")?,
            },
            "wall" if self.tool == RowTool::ChannelBuilder => {
                self.channel.wall_mm = number("channel wall")?
            }
            "spacing" => match self.tool {
                RowTool::ChannelBuilder => self.channel.spacing_mm = number("channel spacing")?,
                RowTool::MicroProng => self.microprong.spacing_mm = number("microprong spacing")?,
                _ => return Err(Error::Invalid("cutter option")),
            },
            "start" if self.tool == RowTool::ChannelBuilder => {
                self.channel.start = ChannelEnd::from_tag(value)?
            }
            "end" if self.tool == RowTool::ChannelBuilder => {
                self.channel.end = ChannelEnd::from_tag(value)?
            }
            "startdepth" if self.tool == RowTool::ChannelBuilder => {
                self.channel.start_depth_mm = number("channel end depth")?
            }
            "enddepth" if self.tool == RowTool::ChannelBuilder => {
                self.channel.end_depth_mm = number("channel end depth")?
            }
            "mirror" => match self.tool {
                RowTool::ChannelBuilder => self.channel.mirror = toggle("cutter mirror")?,
                RowTool::BrightCutChannel => self.bright.mirror = toggle("cutter mirror")?,
                _ => return Err(Error::Invalid("cutter option")),
            },
            "flip" if self.tool == RowTool::ChannelBuilder => {
                self.channel.flip = toggle("cutter flip")?
            }
            "middle" if self.tool == RowTool::ChannelBuilder => {
                self.channel.middle = ChannelMiddle::from_tag(value)?
            }
            "radius" if self.tool == RowTool::MicroProng => {
                self.microprong.radius_mm = number("microprong radius")?
            }
            "count" => match self.tool {
                RowTool::MicroProng => self.microprong.count = count("microprong count")?,
                RowTool::BrightCutter => self.bright.count = count("bright cutter count")?,
                _ => return Err(Error::Invalid("cutter option")),
            },
            "angle" => match self.tool {
                RowTool::MicroProng => self.microprong.angle_deg = number("microprong angle")?,
                _ => self.bright.angle_deg = number("bright cutter angle")?,
            },
            "length" => self.bright.length_mm = number("bright cutter length")?,
            "scale2d" => self.bright.scale_2d_percent = number("cutter 2d scale")?,
            "segments" => {
                let value = count("cutter segments")?;
                if value < 3 {
                    return Err(Error::Invalid("cutter segments"));
                }
                match self.tool {
                    RowTool::ChannelBuilder => self.channel.segments = value,
                    RowTool::MicroProng => self.microprong.segments = value,
                    _ => self.bright.segments = value,
                }
            }
            _ => return Err(Error::Invalid("cutter option")),
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        let specs = self
            .stones
            .iter()
            .map(|stone| stone.spec.clone())
            .collect::<Vec<_>>();
        self.channel = ChannelParams::for_row(&specs)?;
        self.microprong = MicroProngParams::for_row(&specs)?;
        self.bright = BrightParams::for_gem(&self.stones[0].spec)?;
        self.boolean = BooleanMode::Off;
        self.keep = CutterKeep::Preserve;
        Ok(())
    }
}

/// The typed payload every committed row cutter carries in `om9.cutter`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RowCutterPayload {
    pub version: u32,
    pub kind: String,
    pub seat: String,
    pub gem_id: String,
    pub stones: usize,
    /// The stone labels in row order: stable provenance for a later rebuild.
    pub row: Vec<String>,
    pub volume_mm3: f64,
    pub boolean: String,
    pub preserve: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_mm3: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl RowCutterPayload {
    pub fn to_extension(&self) -> Result<om9_core::model::Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("cutter payload"))?;
        om9_core::model::Extension::new(CUTTER_PAYLOAD_VERSION, payload)
    }
}

impl Controller {
    /// Dispatch the four row cutter builders.
    pub fn dispatch_row_cutter(&mut self, feature: &str) -> Result<()> {
        let tool = RowTool::from_feature(feature).ok_or(Error::Unsupported("cutter command"))?;
        if self.is_running() {
            return Err(Error::Busy);
        }
        let session = RowCutterSession::open(self, tool)?;
        let message = session.message.clone();
        self.row_cutters = Some(session);
        self.report_measurement(Measurement::with_text(
            tool.feature(),
            0.0,
            MeasureUnit::Count,
            message,
        ));
        Ok(())
    }

    pub fn row_cutter_session(&self) -> Option<&RowCutterSession> {
        self.row_cutters.as_ref()
    }

    pub fn row_cutter_session_mut(&mut self) -> Option<&mut RowCutterSession> {
        self.row_cutters.as_mut()
    }

    pub fn row_cutter_prompt(&self) -> Option<String> {
        self.row_cutters
            .as_ref()
            .map(|session| session.prompt().to_owned())
    }

    pub fn row_cutter_cancel(&mut self) -> bool {
        self.row_cutters.take().is_some()
    }

    /// A typed option, Enter or Cancel. `None` when no row builder is open.
    pub fn row_cutter_text(&mut self, text: &str) -> Option<Result<()>> {
        let feature = self
            .row_cutters
            .as_ref()
            .map(|session| session.tool.feature())?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower == "cancel" || lower == "esc" {
            self.row_cutters = None;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
            return Some(Ok(()));
        }
        if lower == "enter" || trimmed.is_empty() {
            return Some(self.row_cutter_commit());
        }
        let result = self
            .row_cutters
            .as_mut()
            .ok_or(Error::Missing("cutter session"))
            .and_then(|session| session.set_option(trimmed));
        if result.is_ok() {
            let message = self
                .row_cutters
                .as_ref()
                .map(|session| session.message.clone())
                .unwrap_or_default();
            self.report_measurement(Measurement::with_text(
                feature,
                0.0,
                MeasureUnit::Count,
                message,
            ));
        }
        Some(result)
    }

    /// Commit the open builder through one host transaction: the Boolean cuts of every target,
    /// then the cutter bodies (kept, or removed when the Preserve answer says so).
    pub fn row_cutter_commit(&mut self) -> Result<()> {
        let session = self
            .row_cutters
            .take()
            .ok_or(Error::Missing("cutter session"))?;
        if session.cutters.is_empty() {
            return Err(Error::Invalid("cutter bodies"));
        }
        let layer = self.document().current_layer;
        let removed = session.removed_mm3();
        let mut mutations = Vec::new();
        for ((target, _), cut) in session.targets.iter().zip(session.cuts.iter()) {
            let entity = self
                .document()
                .entities
                .get(target)
                .ok_or(Error::Stale("cutter boolean surface"))?;
            let Geometry::Mesh(_) = entity.geometry else {
                return Err(Error::Invalid("cutter boolean surface"));
            };
            let source = self.document().stamp(*target)?;
            let mut extensions = entity.extensions.clone();
            let record = serde_json::json!({
                "version": CUTTER_PAYLOAD_VERSION,
                "kind": session.tool.kind(),
                "seat": session.tool.label(),
                "row": session.row(),
                "removed_mm3": cut.removed_mm3,
                "applied": cut.applied,
                "preserve": session.keep.label(),
            })
            .to_string();
            extensions.insert(
                CUTTER_CUT_SLOT.to_owned(),
                om9_core::model::Extension::new(CUTTER_PAYLOAD_VERSION, record)?,
            );
            mutations.push(Mutation::Geometry {
                source,
                geometry: Geometry::Mesh(cut.result.clone()),
            });
            mutations.push(Mutation::Extensions { source, extensions });
        }
        if session.keep.keeps() || !session.boolean.cuts() {
            let row = session.row();
            for mesh in &session.cutters {
                let mut entity =
                    Entity::new(session.tool.label(), Geometry::Mesh(mesh.clone()), layer);
                let payload = RowCutterPayload {
                    version: CUTTER_PAYLOAD_VERSION,
                    kind: session.tool.kind().to_owned(),
                    seat: session.tool.label().to_owned(),
                    gem_id: row.first().cloned().unwrap_or_default(),
                    stones: session.stones.len(),
                    row: row.clone(),
                    volume_mm3: om9_core::cutters::volume_mm3(mesh)?,
                    boolean: session.boolean.label().to_owned(),
                    preserve: session.keep.label().to_owned(),
                    removed_mm3: removed,
                    params: serde_json::to_value(&(
                        session.channel,
                        session.microprong,
                        session.bright,
                    ))
                    .ok(),
                };
                entity.extensions.insert(
                    om9_core::model::CUTTER_EXTENSION_SLOT.to_owned(),
                    payload.to_extension()?,
                );
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity,
                });
            }
        }
        if mutations.is_empty() {
            return Err(Error::Invalid("cutter commit is empty"));
        }
        self.commit_utility_mutations(session.tool.transaction_label(), mutations)?;
        self.report_measurement(Measurement::with_text(
            session.tool.feature(),
            session.total_volume_mm3(),
            MeasureUnit::CubicMillimetre,
            session.message.clone(),
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_row_commands_resolve_and_keep_their_kinds() {
        for tool in RowTool::ALL {
            assert_eq!(RowTool::from_feature(tool.feature()), Some(tool));
            assert!(RowTool::handles(tool.feature()));
            assert!(tool.kind().contains("cut"));
        }
        assert_eq!(
            RowTool::command_name("gvChannelBuilder"),
            Some(CHANNEL_BUILDER)
        );
        assert_eq!(
            RowTool::command_name("micro prong cutter"),
            Some(MICRO_PRONG)
        );
        assert_eq!(
            RowTool::command_name("BrightCutChannel"),
            Some(BRIGHT_CUT_CHANNEL)
        );
        assert_eq!(RowTool::command_name("gvBrightCutter"), Some(BRIGHT_CUTTER));
        assert_eq!(RowTool::command_name("not a command"), None);
    }
}
