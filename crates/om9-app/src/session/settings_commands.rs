// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix setting-parts commands (`OM9-L2-04-15`).
//!
//! Eight Builder icons share this one owner: Prong Adder, Prong Editor, Prong On Surface,
//! Metal Piece, Channel Border Curve, Bead On Curve, Bead On Surface and Pull To Rail. They all
//! read the **same** gem objects the gems owner publishes (`om9.gem`) and all of them commit
//! through the host transaction, so Undo restores the previous document.
//!
//! The commands are staged: the icon builds a preview and publishes the clickable options; the
//! user edits values and presses Enter (commit) or Cancel (discard). Every prong body carries an
//! `om9.settings.prong` payload (the site and the parameters it was built from), which is what
//! makes the editor and the settings mass report associative instead of guessing.
use om9_core::{
    gem::{GemRecord, gem_record_in},
    geometry::{Curve, Geometry, Mesh, NurbsSurface, Point3},
    model::{Entity, Extension, SETTINGS_PRONG_SLOT},
    settings::{
        associative::{FeatureMassPart, settings_mass},
        bead::{
            BeadCurve, BeadOnCurveParams, BeadOnSurfaceParams, SurfaceBead, bead_on_curve,
            bead_on_surface, build_bead_bodies,
        },
        channel_border::{ChannelBorder, ChannelBorderParams, ChannelExtension, build_border},
        gems::{GemRow, GemStone, RowShape},
        metal_piece::{
            MetalPiece, MetalPieceMode, MetalPieceParams, build as build_metal_piece,
            build_along_curve as build_metal_along_curve,
            build_with_border as build_metal_with_border,
        },
        prong::{
            EndMode, ProngEdit, ProngMode, ProngParams, ProngSet, ProngSite, SurfaceProngParams,
            build_prong_set, edit_prong, prong_on_surface,
        },
        rail::{MeshPullReport, PullResult, Rail, RailPullMode, pull_mesh, pull_to_rail},
    },
    transaction::Mutation,
};

use super::measure::{MeasureUnit, Measurement};
use super::*;

/// `Builder > Prong Adder`.
pub const PRONG_BUILDER: &str = "BuilderProngAdder";
/// `Builder > Prong Editor`.
pub const PRONG_EDITOR: &str = "BuilderProngEditor";
/// `Builder > Prong On Surface`.
pub const PRONG_ON_SURFACE: &str = "BuilderProngOnSurface";
/// `Builder > Metal Piece`.
pub const METAL_PIECE: &str = "BuilderMetalPiece";
/// `Builder > Channel Border Curve`.
pub const CHANNEL_BORDER: &str = "BuilderChannelBorderCurve";
/// `Builder > Bead On Curve`.
pub const BEAD_ON_CURVE: &str = "BuilderBeadOnCrv";
/// `Builder > Bead On Surface`.
pub const BEAD_ON_SURFACE: &str = "BuilderBeadOnSurface";
/// `Builder > Pull To Rail`.
pub const PULL_TO_RAIL: &str = "BuilderPullToRail";
/// `SettingsMass` is the mass read-out of the settings features built in this session. It has
/// no icon of its own; the sheet reaches it from the settings workflow, not a toolbar slot.
pub const SETTINGS_MASS: &str = "SettingsMass";

/// The payload slot every committed prong body carries.
pub const PRONG_SLOT: &str = SETTINGS_PRONG_SLOT;
const PAYLOAD_VERSION: u32 = 1;

/// One setting-parts command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTool {
    ProngBuilder,
    ProngEditor,
    ProngOnSurface,
    MetalPiece,
    ChannelBorder,
    BeadOnCurve,
    BeadOnSurface,
    PullToRail,
}

impl SettingsTool {
    pub const ALL: [(&'static str, SettingsTool); 8] = [
        (PRONG_BUILDER, SettingsTool::ProngBuilder),
        (PRONG_EDITOR, SettingsTool::ProngEditor),
        (PRONG_ON_SURFACE, SettingsTool::ProngOnSurface),
        (METAL_PIECE, SettingsTool::MetalPiece),
        (CHANNEL_BORDER, SettingsTool::ChannelBorder),
        (BEAD_ON_CURVE, SettingsTool::BeadOnCurve),
        (BEAD_ON_SURFACE, SettingsTool::BeadOnSurface),
        (PULL_TO_RAIL, SettingsTool::PullToRail),
    ];

    pub fn feature(self) -> &'static str {
        match self {
            Self::ProngBuilder => PRONG_BUILDER,
            Self::ProngEditor => PRONG_EDITOR,
            Self::ProngOnSurface => PRONG_ON_SURFACE,
            Self::MetalPiece => METAL_PIECE,
            Self::ChannelBorder => CHANNEL_BORDER,
            Self::BeadOnCurve => BEAD_ON_CURVE,
            Self::BeadOnSurface => BEAD_ON_SURFACE,
            Self::PullToRail => PULL_TO_RAIL,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ProngBuilder => "Prong Builder",
            Self::ProngEditor => "Prong Editor",
            Self::ProngOnSurface => "Prong On Surface",
            Self::MetalPiece => "Metal Piece",
            Self::ChannelBorder => "Channel Border Curve",
            Self::BeadOnCurve => "Bead On Curve",
            Self::BeadOnSurface => "Bead On Surface",
            Self::PullToRail => "Pull To Rail",
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
    SettingsTool::from_feature(feature).is_some()
}

/// Typed command aliases that reach the same owner.
pub fn from_alias(text: &str) -> Option<&'static str> {
    let wanted = text.trim().to_ascii_lowercase();
    let wanted = match wanted.as_str() {
        "prongbuilder" | "prong builder" | "prongadder" | "prong adder" => PRONG_BUILDER,
        "prongeditor" | "prong editor" => PRONG_EDITOR,
        "prongonsurface" | "prong on surface" => PRONG_ON_SURFACE,
        "metalpiece" | "metal piece" | "metalfromgems" => METAL_PIECE,
        "channelborder" | "channel border" | "channelbordercurve" => CHANNEL_BORDER,
        "beadoncurve" | "bead on curve" | "beadoncrv" | "milgrain" => BEAD_ON_CURVE,
        "beadonsurface" | "bead on surface" => BEAD_ON_SURFACE,
        "pulltorail" | "pull to rail" => PULL_TO_RAIL,
        _ => return None,
    };
    Some(wanted)
}

/// The prong payload stored on a committed prong body.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProngPayload {
    pub version: u32,
    pub feature_id: String,
    pub prong_id: String,
    pub site: ProngSite,
    pub params: ProngParams,
}

impl ProngPayload {
    pub fn new(feature_id: &str, site: &ProngSite, params: &ProngParams) -> Self {
        Self {
            version: PAYLOAD_VERSION,
            feature_id: feature_id.to_owned(),
            prong_id: site.stable_id(),
            site: site.clone(),
            params: params.clone(),
        }
    }

    pub fn extension(&self) -> Result<Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("prong payload"))?;
        Extension::new(self.version, payload)
    }

    pub fn from_extension(extension: &Extension) -> Option<Self> {
        let payload: Self = serde_json::from_str(&extension.payload).ok()?;
        (payload.version == PAYLOAD_VERSION).then_some(payload)
    }
}

/// What a started command is waiting to commit.
#[derive(Debug, Clone)]
pub enum SettingsDraft {
    Prongs {
        row: GemRow,
        params: ProngParams,
        set: ProngSet,
    },
    Editor {
        selected: Vec<(om9_core::identity::ObjectId, ProngPayload)>,
        edit: ProngEdit,
        params: ProngParams,
    },
    ProngSurface {
        surface: NurbsSurface,
        uv: [f64; 2],
        params: SurfaceProngParams,
    },
    Metal {
        row: GemRow,
        params: MetalPieceParams,
        piece: MetalPiece,
        /// The curve picked with the gems: the sheet's `Curve` mode orients the metal to it.
        curve: Option<Curve>,
    },
    Border {
        row: GemRow,
        params: ChannelBorderParams,
        border: ChannelBorder,
    },
    Beads {
        curve: Curve,
        params: BeadOnCurveParams,
        row: BeadCurve,
    },
    SurfaceBead {
        surface: NurbsSurface,
        uv: [f64; 2],
        params: BeadOnSurfaceParams,
        bead: SurfaceBead,
    },
    Pull {
        rail: Rail,
        mode: RailPullMode,
        targets: Vec<om9_core::identity::ObjectId>,
        results: Vec<PullResult>,
        meshes: Vec<(om9_core::identity::ObjectId, Mesh)>,
    },
}

/// A staged command: the tool, its draft and the prompt message.
#[derive(Debug, Clone)]
pub struct SettingsPending {
    pub tool: SettingsTool,
    pub draft: SettingsDraft,
    pub message: String,
}

impl SettingsPending {
    pub fn prompt(&self) -> &'static str {
        match self.tool {
            SettingsTool::ProngBuilder => {
                "Mode=, Diameter=, Height=, Nudge=, Ends=, Spacing=, or Enter"
            }
            SettingsTool::ProngEditor => "Diameter=, Height=, Angle=, Nudge=, Fillet=, or Enter",
            SettingsTool::ProngOnSurface => "Diameter=, Height=, Drop=, UV=, or Enter",
            SettingsTool::MetalPiece => "Mode=, Wall=, Height=, Depth=, Offset=, Border=, or Enter",
            SettingsTool::ChannelBorder => {
                "Offset=, EndOffset=, VOffset=, Corner=, Extension=, or Enter"
            }
            SettingsTool::BeadOnCurve => "Start=, End=, Spacing=, Count=, Size=, Taper=, or Enter",
            SettingsTool::BeadOnSurface => "Scale=, Offset=, Angle=, or Enter",
            SettingsTool::PullToRail => "Mode= or Enter",
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
        .ok_or(Error::Invalid("settings numeric option"))
}

fn yes_no_value(text: &str, key: &str) -> Result<Option<bool>> {
    let Some(value) = key_value(text, key) else {
        return Ok(None);
    };
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" => Ok(Some(true)),
        "no" | "false" | "off" => Ok(Some(false)),
        _ => Err(Error::Invalid("settings yes/no option")),
    }
}

fn uv_value(text: &str) -> Result<Option<[f64; 2]>> {
    let Some(value) = key_value(text, "uv") else {
        return Ok(None);
    };
    let (u, v) = value
        .split_once(',')
        .ok_or(Error::Invalid("settings uv option"))?;
    let u = u
        .trim()
        .parse::<f64>()
        .map_err(|_| Error::Invalid("settings uv option"))?;
    let v = v
        .trim()
        .parse::<f64>()
        .map_err(|_| Error::Invalid("settings uv option"))?;
    Ok(Some([u, v]))
}

/// Order selected stones into a row by nearest-neighbour from the first selected gem. The
/// gems owner keeps the selection order meaningful, so the first gem anchors the chain even
/// when the rest was box-selected.
fn chain_order(stones: Vec<GemStone>) -> Vec<GemStone> {
    let mut remaining = stones;
    if remaining.len() < 3 {
        return remaining;
    }
    let mut ordered = vec![remaining.remove(0)];
    while !remaining.is_empty() {
        let last = ordered[ordered.len() - 1].center;
        let mut best = 0;
        let mut best_distance = f64::INFINITY;
        for (index, stone) in remaining.iter().enumerate() {
            let distance = (0..3)
                .map(|axis| (stone.center[axis] - last[axis]).powi(2))
                .sum::<f64>();
            if distance < best_distance {
                best_distance = distance;
                best = index;
            }
        }
        ordered.push(remaining.remove(best));
    }
    ordered
}

impl Controller {
    /// The staging slot, exposed for the option formatter and the tests.
    pub fn settings_pending(&self) -> Option<&SettingsPending> {
        self.settings_pending.as_ref()
    }

    /// The settings feature volumes this session committed, used by `SettingsMass`.
    pub fn settings_features(&self) -> &[(String, f64)] {
        &self.settings_volumes
    }

    /// Settings features committed as cutters: they remove metal and never add mass.
    pub fn settings_cutter_count(&self) -> usize {
        self.settings_cutters.len()
    }

    pub fn settings_mass_report(&self) -> Option<&SettingsMassReport> {
        self.last_settings_mass.as_ref()
    }

    /// The gem row of the current selection: gem objects only, ordered along the row.
    pub fn gem_row_from_selection(&self, closed: bool) -> Result<GemRow> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("gem row selection"));
        }
        let mut stones = Vec::with_capacity(ids.len());
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("gem row object"))?;
            let Some(record): Option<GemRecord> = gem_record_in(self.document(), id) else {
                // A picked curve is the sheet's Curve mode input and rides along with the row;
                // any other non-gem object is refused instead of being silently dropped.
                match entity.geometry {
                    Geometry::Curve(_) => continue,
                    _ => return Err(Error::Invalid("settings need gem objects")),
                }
            };
            let matrix = entity.placement.0;
            let center: Point3 = [matrix[0][3], matrix[1][3], matrix[2][3]];
            let axis: Point3 = [matrix[0][2], matrix[1][2], matrix[2][2]];
            stones.push(GemStone::round(
                record.gem_id.to_string(),
                center,
                record.spec.width_mm,
                axis,
            )?);
        }
        if stones.is_empty() {
            return Err(Error::Invalid("settings need gem objects"));
        }
        let shape = if closed {
            RowShape::Closed
        } else {
            RowShape::Open
        };
        GemRow::new(chain_order(stones), shape)
    }

    fn settings_surface(&self, id: om9_core::identity::ObjectId) -> Result<NurbsSurface> {
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("settings surface"))?;
        match &entity.geometry {
            Geometry::Surface(surface) => Ok(surface.clone()),
            _ => Err(Error::Invalid("settings need a selected surface")),
        }
    }

    fn settings_curve(&self, id: om9_core::identity::ObjectId) -> Result<Curve> {
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("settings curve"))?;
        match &entity.geometry {
            Geometry::Curve(curve) => Ok(curve.clone()),
            _ => Err(Error::Invalid("settings need a selected curve")),
        }
    }

    /// The first picked curve in the selection: the sheet's `Curve` mode prompt ("select a
    /// curve on the target surface"). Gem bodies in the same selection stay the row input.
    fn picked_curve(&self) -> Option<Curve> {
        self.selected_ordered().into_iter().find_map(|id| {
            match self
                .document()
                .entities
                .get(&id)
                .map(|entity| &entity.geometry)
            {
                Some(Geometry::Curve(curve)) => Some(curve.clone()),
                _ => None,
            }
        })
    }

    /// Start one setting-parts command.
    pub fn dispatch_settings_tool(&mut self, feature: &str) -> Result<()> {
        let tool =
            SettingsTool::from_feature(feature).ok_or(Error::Unsupported("settings command"))?;
        if self.settings_pending.is_some() || self.is_running() {
            return Err(Error::Busy);
        }
        let limits = *self.coordinator.limits();
        let draft = match tool {
            SettingsTool::ProngBuilder => {
                let row = self.gem_row_from_selection(false)?;
                let params = ProngParams::default();
                let set = build_prong_set(&row, &params)?;
                SettingsDraft::Prongs { row, params, set }
            }
            SettingsTool::ProngEditor => {
                let mut selected = Vec::new();
                for id in self.selected_ordered() {
                    let entity = self
                        .document()
                        .entities
                        .get(&id)
                        .ok_or(Error::Missing("prong selection"))?;
                    let payload = entity
                        .extension(PRONG_SLOT)
                        .and_then(ProngPayload::from_extension)
                        .ok_or(Error::Invalid("prong editor needs prong bodies"))?;
                    selected.push((id, payload));
                }
                if selected.is_empty() {
                    return Err(Error::Missing("prong selection"));
                }
                let params = selected[0].1.params.clone();
                SettingsDraft::Editor {
                    selected,
                    edit: ProngEdit::default(),
                    params,
                }
            }
            SettingsTool::ProngOnSurface => {
                let id = *self
                    .selection
                    .iter()
                    .next()
                    .ok_or(Error::Missing("prong surface selection"))?;
                let surface = self.settings_surface(id)?;
                SettingsDraft::ProngSurface {
                    surface,
                    uv: [0.5, 0.5],
                    params: SurfaceProngParams::default(),
                }
            }
            SettingsTool::MetalPiece => {
                let row = self.gem_row_from_selection(false)?;
                let params = MetalPieceParams::default();
                let curve = self.picked_curve();
                let piece = metal_piece_for(
                    &row,
                    self.settings_border.as_ref(),
                    curve.as_ref(),
                    &params,
                    &limits,
                )?;
                SettingsDraft::Metal {
                    row,
                    params,
                    piece,
                    curve,
                }
            }
            SettingsTool::ChannelBorder => {
                let row = self.gem_row_from_selection(false)?;
                let params = ChannelBorderParams::default();
                let border = build_border(&row, &params, &limits)?;
                SettingsDraft::Border {
                    row,
                    params,
                    border,
                }
            }
            SettingsTool::BeadOnCurve => {
                let id = *self
                    .selection
                    .iter()
                    .next()
                    .ok_or(Error::Missing("bead guide curve"))?;
                let curve = self.settings_curve(id)?;
                let params = BeadOnCurveParams::default();
                let row = bead_on_curve(&curve, &params, &limits)?;
                SettingsDraft::Beads { curve, params, row }
            }
            SettingsTool::BeadOnSurface => {
                let id = *self
                    .selection
                    .iter()
                    .next()
                    .ok_or(Error::Missing("bead surface selection"))?;
                let surface = self.settings_surface(id)?;
                let params = BeadOnSurfaceParams::default();
                let (bead, _) = bead_on_surface(&surface, [0.5, 0.5], &params, &limits)?;
                SettingsDraft::SurfaceBead {
                    surface,
                    uv: [0.5, 0.5],
                    params,
                    bead,
                }
            }
            SettingsTool::PullToRail => {
                let ids = self.selected_ordered();
                if ids.is_empty() {
                    return Err(Error::Missing("pull selection"));
                }
                let mut rail = None;
                let mut targets = Vec::new();
                for id in ids {
                    let entity = self
                        .document()
                        .entities
                        .get(&id)
                        .ok_or(Error::Missing("pull object"))?;
                    if let Geometry::Curve(Curve::Circle {
                        center,
                        x,
                        y,
                        radius,
                        ..
                    }) = &entity.geometry
                        && rail.is_none()
                    {
                        let axis = om9_core::settings::unit(om9_core::settings::cross(*x, *y))?;
                        rail = Some(Rail::new(*center, axis, *radius)?);
                        continue;
                    }
                    targets.push(id);
                }
                let rail = rail.ok_or(Error::Missing("pull needs a selected circle rail"))?;
                if targets.is_empty() {
                    return Err(Error::Missing("pull selection"));
                }
                let mut results = Vec::new();
                let mut meshes = Vec::new();
                for id in &targets {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("pull object"))?;
                    let matrix = entity.placement.0;
                    let origin: Point3 = [matrix[0][3], matrix[1][3], matrix[2][3]];
                    let axis: Point3 = [matrix[0][2], matrix[1][2], matrix[2][2]];
                    results.push(pull_to_rail(origin, axis, &rail, RailPullMode::Rigid)?);
                    if let Geometry::Mesh(mesh) = &entity.geometry {
                        let (pulled, _) = pull_mesh(mesh, &rail, RailPullMode::Deform)?;
                        meshes.push((*id, pulled));
                    }
                }
                SettingsDraft::Pull {
                    rail,
                    mode: RailPullMode::Rigid,
                    targets,
                    results,
                    meshes,
                }
            }
        };
        let mut staged = SettingsPending {
            tool,
            draft,
            message: String::new(),
        };
        staged.message = settings_message(&staged);
        let message = staged.message.clone();
        self.settings_pending = Some(staged);
        self.report_measurement(Measurement::with_text(
            tool.feature(),
            0.0,
            MeasureUnit::Count,
            message,
        ));
        Ok(())
    }

    /// The clickable option row of a staged command.
    pub fn settings_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(pending) = &self.settings_pending else {
            return Vec::new();
        };
        let mut tokens = match pending.tool {
            SettingsTool::ProngBuilder => vec![
                ("Mode=", "Mode="),
                ("Diameter=", "Diameter="),
                ("Height=", "Height="),
                ("Nudge=", "Nudge="),
                ("Ends=", "Ends="),
                ("Spacing=", "Spacing="),
            ],
            SettingsTool::ProngEditor => vec![
                ("Diameter=", "Diameter="),
                ("Height=", "Height="),
                ("Angle=", "Angle="),
                ("Nudge=", "Nudge="),
                ("Fillet=", "Fillet="),
            ],
            SettingsTool::ProngOnSurface => vec![
                ("Diameter=", "Diameter="),
                ("Height=", "Height="),
                ("Drop=", "Drop="),
                ("UV=", "UV="),
            ],
            SettingsTool::MetalPiece => vec![
                ("Mode=", "Mode="),
                ("Wall=", "Wall="),
                ("Height=", "Height="),
                ("Depth=", "Depth="),
                ("Offset=", "Offset="),
                ("Border=", "Border="),
            ],
            SettingsTool::ChannelBorder => vec![
                ("Offset=", "Offset="),
                ("EndOffset=", "EndOffset="),
                ("VOffset=", "VOffset="),
                ("Corner=", "Corner="),
                ("Extension=", "Extension="),
            ],
            SettingsTool::BeadOnCurve => vec![
                ("Start=", "Start="),
                ("End=", "End="),
                ("Spacing=", "Spacing="),
                ("Count=", "Count="),
                ("Size=", "Size="),
                ("Taper=", "Taper="),
            ],
            SettingsTool::BeadOnSurface => vec![
                ("Scale=", "Scale="),
                ("Offset=", "Offset="),
                ("Angle=", "Angle="),
            ],
            SettingsTool::PullToRail => vec![("Mode=", "Mode=")],
        };
        tokens.push(("Enter", "Enter"));
        tokens.push(("Cancel", "Cancel"));
        tokens
    }

    /// Submit one staged option (or commit/cancel). `None` means the input was not ours.
    pub fn submit_settings_text(&mut self, text: &str) -> Option<Result<()>> {
        self.settings_pending.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower == "cancel" || lower == "esc" {
            self.settings_pending = None;
            return Some(Ok(()));
        }
        if lower == "enter" || trimmed.is_empty() {
            return Some(self.commit_settings_pending());
        }
        let result = self.apply_settings_option(trimmed);
        Some(result.map(|()| {
            if let Some(pending) = &self.settings_pending {
                self.report_measurement(Measurement::with_text(
                    pending.tool.feature(),
                    0.0,
                    MeasureUnit::Count,
                    pending.message.clone(),
                ));
            }
        }))
    }

    fn apply_settings_option(&mut self, text: &str) -> Result<()> {
        let original = self
            .settings_pending
            .take()
            .ok_or(Error::Missing("settings session"))?;
        let mut pending = original.clone();
        match self.apply_option_to_draft(&mut pending, text) {
            Ok(()) => {
                let message = settings_message(&pending);
                pending.message = message;
                self.settings_pending = Some(pending);
                Ok(())
            }
            Err(error) => {
                // A refused value keeps the command and its last valid preview alive.
                self.settings_pending = Some(original);
                Err(error)
            }
        }
    }

    fn apply_option_to_draft(&mut self, pending: &mut SettingsPending, text: &str) -> Result<()> {
        let limits = *self.coordinator.limits();
        let border = self.settings_border.clone();
        match &mut pending.draft {
            SettingsDraft::Prongs { row, params, set } => {
                if let Some(value) = number(text, "diameter")? {
                    params.large_diameter_mm = value;
                }
                if let Some(value) = number(text, "height")? {
                    params.large_height_mm = value;
                }
                if let Some(value) = number(text, "nudge")? {
                    params.nudge = value;
                }
                if let Some(value) = number(text, "spacing")? {
                    params.spacing_mm = Some(value);
                }
                if let Some(value) = key_value(text, "mode") {
                    params.mode =
                        ProngMode::from_name(value).ok_or(Error::Invalid("prong mode option"))?;
                }
                if let Some(value) = key_value(text, "ends") {
                    params.end_mode =
                        EndMode::from_name(value).ok_or(Error::Invalid("prong ends option"))?;
                }
                params.validate()?;
                *set = build_prong_set(row, params)?;
            }
            SettingsDraft::Editor {
                edit,
                params,
                selected,
            } => {
                if let Some(value) = number(text, "diameter")? {
                    edit.diameter_mm = Some(value);
                }
                if let Some(value) = number(text, "height")? {
                    edit.height_mm = Some(value);
                }
                if let Some(value) = number(text, "angle")? {
                    edit.angle_deg = Some(value);
                }
                if let Some(value) = number(text, "nudge")? {
                    edit.nudge = Some(value);
                }
                if let Some(value) = number(text, "fillet")? {
                    edit.fillet = Some(value);
                }
                if edit.is_empty() {
                    return Err(Error::Missing("prong edit"));
                }
                // Rebuild every selected prong now so the preview matches the commit.
                for (_, payload) in selected.iter() {
                    let mut sites = [payload.site.clone()];
                    edit_prong(&mut sites, 0, params, edit)?;
                }
            }
            SettingsDraft::ProngSurface {
                surface,
                uv,
                params,
            } => {
                if let Some(value) = number(text, "diameter")? {
                    params.diameter_mm = value;
                }
                if let Some(value) = number(text, "height")? {
                    params.height_mm = value;
                }
                if let Some(value) = number(text, "drop")? {
                    params.drop_mm = value;
                }
                if let Some(value) = number(text, "fillet")? {
                    params.fillet = value;
                }
                if let Some(value) = uv_value(text)? {
                    *uv = value;
                }
                params.validate()?;
                prong_on_surface(surface, *uv, params, &limits)?;
            }
            SettingsDraft::Metal {
                row,
                params,
                piece,
                curve,
            } => {
                if let Some(value) = key_value(text, "mode") {
                    params.mode = MetalPieceMode::from_name(value)
                        .ok_or(Error::Invalid("metal piece mode option"))?;
                }
                if let Some(value) = number(text, "wall")? {
                    params.wall_mm = value;
                }
                if let Some(value) = number(text, "height")? {
                    params.height_mm = value;
                }
                if let Some(value) = number(text, "depth")? {
                    params.depth_mm = value;
                }
                if let Some(value) = number(text, "offset")? {
                    params.gem_offset_mm = value;
                }
                if let Some(value) = yes_no_value(text, "border")? {
                    params.use_border = value;
                }
                params.validate()?;
                *piece = metal_piece_for(row, border.as_ref(), curve.as_ref(), params, &limits)?;
            }
            SettingsDraft::Border {
                row,
                params,
                border,
            } => {
                if let Some(value) = number(text, "offset")? {
                    params.side_offset_mm = value;
                }
                if let Some(value) = number(text, "endoffset")? {
                    params.end_offset_mm = value;
                }
                if let Some(value) = number(text, "voffset")? {
                    params.v_offset_mm = value;
                }
                if let Some(value) = number(text, "corner")? {
                    params.corner_angle = value;
                }
                if let Some(value) = key_value(text, "extension") {
                    params.extension = ChannelExtension::from_name(value)
                        .ok_or(Error::Invalid("channel border extension option"))?;
                }
                params.validate()?;
                *border = build_border(row, params, &limits)?;
            }
            SettingsDraft::Beads { curve, params, row } => {
                if let Some(value) = number(text, "start")? {
                    params.start_fraction = value;
                }
                if let Some(value) = number(text, "end")? {
                    params.end_fraction = value;
                }
                if let Some(value) = number(text, "spacing")? {
                    params.spacing_mm = value;
                    // Driving the run by spacing drops an explicit count, exactly like the
                    // sheet's count/spacing pair.
                    params.count = None;
                }
                if let Some(value) = number(text, "count")? {
                    let rounded = value.round();
                    if (value - rounded).abs() > 1.0e-9 || !(2.0..=4096.0).contains(&rounded) {
                        return Err(Error::Invalid("bead count"));
                    }
                    params.count = Some(rounded as usize);
                }
                if let Some(value) = number(text, "size")? {
                    params.start_size_mm = value;
                }
                if let Some(value) = yes_no_value(text, "taper")? {
                    params.taper_gems = value;
                }
                params.validate()?;
                *row = bead_on_curve(curve, params, &limits)?;
            }
            SettingsDraft::SurfaceBead {
                surface,
                uv,
                params,
                bead,
            } => {
                if let Some(value) = number(text, "scale")? {
                    params.scale = value;
                }
                if let Some(value) = number(text, "offset")? {
                    params.vertical_offset_mm = value;
                }
                if let Some(value) = number(text, "angle")? {
                    params.object_angle_deg = value;
                }
                if let Some(value) = uv_value(text)? {
                    *uv = value;
                }
                params.validate()?;
                let (built, _) = bead_on_surface(surface, *uv, params, &limits)?;
                *bead = built;
            }
            SettingsDraft::Pull {
                rail,
                mode,
                results,
                targets,
                ..
            } => {
                if let Some(value) = key_value(text, "mode") {
                    *mode = match value.to_ascii_lowercase().as_str() {
                        "rigid" => RailPullMode::Rigid,
                        "deform" => RailPullMode::Deform,
                        _ => return Err(Error::Invalid("pull mode option")),
                    };
                }
                results.clear();
                for id in targets.iter() {
                    let entity = self
                        .document()
                        .entities
                        .get(id)
                        .ok_or(Error::Missing("pull object"))?;
                    let matrix = entity.placement.0;
                    let origin: Point3 = [matrix[0][3], matrix[1][3], matrix[2][3]];
                    let axis: Point3 = [matrix[0][2], matrix[1][2], matrix[2][2]];
                    results.push(pull_to_rail(origin, axis, rail, *mode)?);
                }
            }
        }
        Ok(())
    }

    fn commit_settings_pending(&mut self) -> Result<()> {
        let pending = self
            .settings_pending
            .take()
            .ok_or(Error::Missing("settings session"))?;
        let feature_id = format!(
            "{}-{:04}",
            pending.tool.feature(),
            self.settings_sequence.wrapping_add(1)
        );
        self.settings_sequence = self.settings_sequence.wrapping_add(1);
        let mut volumes = Vec::new();
        let mut cutters = Vec::new();
        let mut mutations = Vec::new();
        let layer = self.document().current_layer;
        match &pending.draft {
            SettingsDraft::Prongs { set, params, .. } => {
                if set.bodies.len() != set.sites.len() {
                    return Err(Error::Invalid("prong body count"));
                }
                for (site, body) in set.sites.iter().zip(&set.bodies) {
                    let mut entity = Entity::new("Prong", Geometry::Mesh(body.clone()), layer);
                    entity.extensions.insert(
                        PRONG_SLOT.to_owned(),
                        ProngPayload::new(&feature_id, site, params).extension()?,
                    );
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                volumes.push((feature_id.clone(), set.volume_mm3));
            }
            SettingsDraft::Editor {
                selected,
                edit,
                params,
            } => {
                for (id, payload) in selected {
                    let mut sites = [payload.site.clone()];
                    let body = edit_prong(&mut sites, 0, params, edit)?;
                    let mut edited = payload.clone();
                    edited.site = sites[0].clone();
                    edited.params = params.clone();
                    let stamp = self.document().stamp(*id)?;
                    mutations.push(Mutation::Geometry {
                        source: stamp,
                        geometry: Geometry::Mesh(body),
                    });
                    mutations.push(Mutation::Extensions {
                        source: stamp,
                        extensions: std::collections::BTreeMap::from([(
                            PRONG_SLOT.to_owned(),
                            edited.extension()?,
                        )]),
                    });
                }
            }
            SettingsDraft::ProngSurface {
                surface,
                uv,
                params,
            } => {
                let (site, body) =
                    prong_on_surface(surface, *uv, params, self.coordinator.limits())?;
                let mut entity = Entity::new("Prong", Geometry::Mesh(body), layer);
                entity.extensions.insert(
                    PRONG_SLOT.to_owned(),
                    ProngPayload::new(&feature_id, &site, &ProngParams::default()).extension()?,
                );
                let report = om9_core::settings::inspect_solid(&entity_mesh(&entity)?)?;
                volumes.push((feature_id.clone(), report.volume_mm3));
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity,
                });
            }
            SettingsDraft::Metal { piece, .. } => {
                for body in &piece.bodies {
                    let entity = Entity::new("Metal Piece", Geometry::Mesh(body.clone()), layer);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                // A cutter removes metal; it never adds mass. Everything else is a metal body.
                if piece.is_cutter {
                    cutters.push(feature_id.clone());
                } else {
                    volumes.push((feature_id.clone(), piece.volume_mm3));
                }
            }
            SettingsDraft::Border { border, .. } => {
                // Keep the committed border so Metal Piece can follow it (the sheet's optional
                // channel border input).
                self.settings_border = Some(border.clone());
                for (name, curve) in [
                    ("Channel Border Left", &border.left),
                    ("Channel Border Right", &border.right),
                ] {
                    let entity = Entity::new(name, Geometry::Curve(curve.clone()), layer);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
            }
            SettingsDraft::Beads { row, .. } => {
                let bodies = build_bead_bodies(row, 24)?;
                let mut volume = 0.0;
                for body in bodies {
                    volume += om9_core::settings::inspect_solid(&body)?.volume_mm3;
                    let entity = Entity::new("Bead", Geometry::Mesh(body), layer);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity,
                    });
                }
                volumes.push((feature_id.clone(), volume));
            }
            SettingsDraft::SurfaceBead { bead, .. } => {
                let body = om9_core::settings::sphere(bead.frame.origin, bead.diameter_mm, 24)?;
                let report = om9_core::settings::inspect_solid(&body)?;
                volumes.push((feature_id.clone(), report.volume_mm3));
                let entity = Entity::new("Bead", Geometry::Mesh(body), layer);
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity,
                });
            }
            SettingsDraft::Pull {
                mode,
                targets,
                results,
                meshes,
                ..
            } => {
                for (id, result) in targets.iter().zip(results.iter()) {
                    match mode {
                        RailPullMode::Rigid => {
                            let entity = self
                                .document()
                                .entities
                                .get(id)
                                .ok_or(Error::Missing("pull object"))?;
                            let mut placement = entity.placement.0;
                            let matrix = entity.placement.0;
                            let origin: Point3 = [matrix[0][3], matrix[1][3], matrix[2][3]];
                            for axis in 0..3 {
                                placement[axis][3] += result.origin[axis] - origin[axis];
                            }
                            let stamp = self.document().stamp(*id)?;
                            mutations.push(Mutation::Placement {
                                source: stamp,
                                placement: om9_core::geometry::Transform(placement),
                            });
                        }
                        RailPullMode::Deform => {}
                    }
                }
                if *mode == RailPullMode::Deform {
                    for (id, mesh) in meshes {
                        let stamp = self.document().stamp(*id)?;
                        mutations.push(Mutation::Geometry {
                            source: stamp,
                            geometry: Geometry::Mesh(mesh.clone()),
                        });
                    }
                }
            }
        }
        if mutations.is_empty() {
            return Err(Error::Invalid("settings commit"));
        }
        self.commit_utility_mutations("Settings", mutations)?;
        for (id, volume) in volumes {
            self.settings_volumes.push((id, volume));
        }
        self.settings_cutters.extend(cutters);
        self.report_measurement(Measurement::with_text(
            pending.tool.feature(),
            0.0,
            MeasureUnit::Count,
            format!("{} committed", pending.message),
        ));
        Ok(())
    }

    /// Mass of every settings feature this session committed, using the shared metal owner.
    pub fn calculate_settings_mass(
        &mut self,
        material: Option<&str>,
    ) -> Result<SettingsMassReport> {
        if self.settings_volumes.is_empty() {
            return Err(Error::Missing("settings mass features"));
        }
        let material = material.unwrap_or(om9_core::metal_weight::DEFAULT_MATERIAL_ID);
        let parts: Vec<FeatureMassPart> = self
            .settings_volumes
            .iter()
            .map(|(id, volume)| FeatureMassPart::new(id.clone(), *volume, material))
            .collect();
        let registry = self.metal_registry.clone();
        let breakdown = settings_mass(&parts, &registry)?;
        let material_name = registry
            .get(material)
            .map(|material| material.name.clone())
            .map_err(|_| Error::Missing("metal material density"))?;
        let excluded = self.settings_cutters.len();
        let cutter_volume: f64 = self
            .settings_cutter_volumes
            .iter()
            .map(|(_, volume)| *volume)
            .sum();
        let net_volume = (breakdown.total_volume_mm3 - cutter_volume).max(0.0);
        let scale = if breakdown.total_volume_mm3 > 0.0 {
            net_volume / breakdown.total_volume_mm3
        } else {
            1.0
        };
        let net_grams = breakdown.total_grams * scale;
        let net_dwt = breakdown.total_dwt * scale;
        let cutter_note = if excluded == 0 {
            String::new()
        } else if cutter_volume > 1.0e-9 {
            // Only cutters that recorded their removed volume can report a net mass.
            // The cutter's own volume is a bound on the metal it removes: the intersection with
            // the settings body is not measured here, so the report says "net metal" not "removed".
            format!(
                " · {} cutter{} excluded ({} mm³ of cutter volume; net metal {} mm³ · {} g)",
                excluded,
                if excluded == 1 { "" } else { "s" },
                Measurement::format_value(cutter_volume),
                Measurement::format_value(net_volume),
                Measurement::format_value(net_grams),
            )
        } else {
            match excluded {
                1 => " · 1 cutter excluded".to_owned(),
                count => format!(" · {count} cutters excluded"),
            }
        };
        let text = format!(
            "Settings Mass: {} — {} features, {} mm³ · {} g · {} dwt{}",
            material_name,
            breakdown.entries.len(),
            Measurement::format_value(breakdown.total_volume_mm3),
            Measurement::format_value(breakdown.total_grams),
            Measurement::format_value(breakdown.total_dwt),
            cutter_note,
        );
        let report = SettingsMassReport {
            material: material.to_owned(),
            material_name,
            volume_mm3: breakdown.total_volume_mm3,
            grams: breakdown.total_grams,
            dwt: breakdown.total_dwt,
            features: breakdown.entries.len(),
            excluded_cutters: excluded,
            cutter_volume_mm3: cutter_volume,
            net_volume_mm3: net_volume,
            net_grams,
            net_dwt,
            text: text.clone(),
        };
        self.last_settings_mass = Some(report.clone());
        self.report_measurement(Measurement::with_text(
            SETTINGS_MASS,
            report.grams,
            MeasureUnit::Gram,
            text,
        ));
        Ok(report)
    }
}

/// Parse `SettingsMass [material=<id>]`; a different command returns `Ok(None)`.
pub fn parse_mass_command(text: &str) -> Result<Option<Option<String>>> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    let rest = if lower.starts_with("settingsmass") {
        &trimmed["settingsmass".len()..]
    } else if lower.starts_with("settings mass") {
        &trimmed["settings mass".len()..]
    } else {
        return Ok(None);
    };
    let mut material = None;
    for token in rest.split_whitespace() {
        if let Some((key, value)) = token.split_once('=')
            && key.eq_ignore_ascii_case("material")
        {
            if value.trim().is_empty() {
                return Err(Error::Missing("settings material id"));
            }
            material = Some(value.trim().to_owned());
        } else {
            return Err(Error::Invalid("settings mass option"));
        }
    }
    Ok(Some(material))
}

/// The settings mass read-out.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsMassReport {
    pub material: String,
    pub material_name: String,
    pub volume_mm3: f64,
    pub grams: f64,
    pub dwt: f64,
    pub features: usize,
    /// Cutter features committed in this session; they remove metal and are excluded from mass.
    pub excluded_cutters: usize,
    /// Total volume the cutters remove, mm³.
    pub cutter_volume_mm3: f64,
    /// Metal left after the cutters are subtracted, mm³.
    pub net_volume_mm3: f64,
    pub net_grams: f64,
    pub net_dwt: f64,
    pub text: String,
}

fn entity_mesh(entity: &Entity) -> Result<Mesh> {
    match &entity.geometry {
        Geometry::Mesh(mesh) => Ok(mesh.clone()),
        _ => Err(Error::Invalid("settings mesh")),
    }
}

fn settings_message(pending: &SettingsPending) -> String {
    match &pending.draft {
        SettingsDraft::Prongs { set, .. } => format!(
            "Prong Builder: {} prongs, {} mm³, {} min height",
            set.count(),
            Measurement::format_value(set.volume_mm3),
            Measurement::format_value(set.min_height_mm),
        ),
        SettingsDraft::Editor { selected, edit, .. } => {
            let fields = [
                edit.diameter_mm.map(|value| format!("D={value}")),
                edit.height_mm.map(|value| format!("H={value}")),
                edit.angle_deg.map(|value| format!("A={value}")),
                edit.nudge.map(|value| format!("N={value}")),
                edit.fillet.map(|value| format!("F={value}")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
            format!(
                "Prong Editor: {} selected, {}",
                selected.len(),
                if fields.is_empty() {
                    "no edit yet".to_owned()
                } else {
                    fields
                }
            )
        }
        SettingsDraft::ProngSurface { params, uv, .. } => format!(
            "Prong On Surface: D={} H={} at uv {:.3},{:.3}",
            Measurement::format_value(params.diameter_mm),
            Measurement::format_value(params.height_mm),
            uv[0],
            uv[1]
        ),
        SettingsDraft::Metal { piece, .. } => format!("Metal Piece: {}", piece.summary()),
        SettingsDraft::Border { border, .. } => format!("Channel Border: {}", border.summary()),
        SettingsDraft::Beads { row, .. } => format!("Bead On Curve: {}", row.summary()),
        SettingsDraft::SurfaceBead { bead, .. } => format!(
            "Bead On Surface: D={} at uv {:.3},{:.3}",
            Measurement::format_value(bead.diameter_mm),
            bead.uv[0],
            bead.uv[1]
        ),
        SettingsDraft::Pull {
            rail,
            mode,
            targets,
            results,
            ..
        } => {
            let moved = results
                .iter()
                .map(|result| result.displacement_mm)
                .fold(0.0, f64::max);
            format!(
                "Pull To Rail: {:?} {} objects onto r={} rail, max move {}",
                mode,
                targets.len(),
                Measurement::format_value(rail.radius_mm),
                Measurement::format_value(moved)
            )
        }
    }
}

/// `MeshPullReport` is part of the pull evidence; keep it reachable from the command module.
pub fn pull_report_text(report: &MeshPullReport) -> String {
    format!(
        "{:?} pull: {} vertices, max {} mm, radial error {} mm",
        report.mode,
        report.vertices,
        Measurement::format_value(report.max_displacement_mm),
        Measurement::format_value(report.max_radial_error_mm)
    )
}

/// Stage Metal Piece: the row offset by default, the committed channel border with `Border=On`,
/// or the picked curve in `Curve` mode. A missing border or curve is a typed refusal, never a
/// silent row-offset body.
fn metal_piece_for(
    row: &GemRow,
    border: Option<&ChannelBorder>,
    curve: Option<&Curve>,
    params: &MetalPieceParams,
    limits: &Limits,
) -> Result<MetalPiece> {
    if params.use_border {
        let border = border.ok_or(Error::Missing("metal piece channel border"))?;
        return build_metal_with_border(row, border, params);
    }
    if params.mode == MetalPieceMode::Curve {
        // The sheet's Curve mode prompts for a curve on the target surface and orients the
        // metal to it; a committed channel border and a picked curve are both such a curve.
        let curve = curve.ok_or(Error::Missing("metal piece curve"))?;
        return build_metal_along_curve(row, curve, params, limits);
    }
    build_metal_piece(row, params)
}
