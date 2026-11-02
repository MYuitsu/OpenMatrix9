// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem Cutter and Gem Cutter Library commands (`OM9-L2-04-16`, `OM9-L3-04-16-001/002/003/005/
//! 006/007`).
//!
//! `Cutters > Gem Cutters` (`gvAzureCutter`) and `Cutters > Gem Cutter Library` are one staged
//! builder each: the icon reads the selected stones, builds a preview, publishes the manual's
//! options, and commits on Enter through the host transaction. The Interactive Boolean preview
//! is the *target minus the cutters*; the cutter bodies are kept or removed by the Preserve
//! option. Cancel throws the whole draft away.
//!
//! The geometry is the M07 cutter service (`om9_core::cutters`); this module only stages,
//! validates the selection, and commits. It never writes a second gem store: the stones are the
//! one `om9.gem` record, and every committed cutter carries the `om9.cutter` extension the
//! display filter already reads.
use om9_core::{
    Error, Result,
    cutters::{
        EdgeShape,
        boolean::{BooleanMode, CutterCut, CutterKeep, apply as apply_boolean},
        gem::{
            BottomDirection, CutterMode, GemCutterParams, GemCutterPlan, build as build_gem_cutter,
        },
        library::{CutterPreset, LibraryCutter, LibraryParams, build as build_library_cutter},
    },
    gem::{GemSpec, gem_record_in},
    geometry::{Geometry, Mesh, Transform},
    mesh_csg::CsgOptions,
    model::{CUTTER_EXTENSION_SLOT, Entity},
    transaction::Mutation,
};

use super::measure::{MeasureUnit, Measurement};
use super::*;

/// `Cutters > Gem Cutters` (`gvAzureCutter`).
pub const GEM_CUTTER: &str = "BuilderGemCutter";
/// `Cutters > Gem Cutter Library` (`gvGemCutterLibrary`).
pub const GEM_CUTTER_LIBRARY: &str = "BuilderGemCutterLibrary";

/// Version of the `om9.cutter` payload this owner writes.
pub const CUTTER_PAYLOAD_VERSION: u32 = 1;
/// Extension slot carried by a cut target: which cutter cut it and how much was removed.
pub const CUTTER_CUT_SLOT: &str = om9_core::model::CUTTER_CUT_EXTENSION_SLOT;

/// The two M07 cutter commands of this owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutterTool {
    GemCutter,
    GemCutterLibrary,
}

impl CutterTool {
    pub const ALL: [(&'static str, CutterTool); 2] = [
        (GEM_CUTTER, CutterTool::GemCutter),
        (GEM_CUTTER_LIBRARY, CutterTool::GemCutterLibrary),
    ];

    pub fn feature(self) -> &'static str {
        match self {
            Self::GemCutter => GEM_CUTTER,
            Self::GemCutterLibrary => GEM_CUTTER_LIBRARY,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::GemCutter => "Gem Cutter",
            Self::GemCutterLibrary => "Gem Cutter Library",
        }
    }

    pub fn transaction_label(self) -> &'static str {
        match self {
            Self::GemCutter => "GemCutter",
            Self::GemCutterLibrary => "GemCutterLibrary",
        }
    }

    pub fn from_feature(feature: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|(id, _)| *id == feature)
            .map(|(_, tool)| tool)
    }

    /// Typed command names that reach the same handler.
    pub fn command_name(text: &str) -> Option<&'static str> {
        match text.trim().to_ascii_lowercase().as_str() {
            "azurecutter" | "azure cutter" | "gemcutter" | "gem cutter" | "gvazurecutter"
            | "gvazure" | "gemcutters" => Some(GEM_CUTTER),
            "gemcutterlibrary" | "gem cutter library" | "cutterslibrary" | "gvgemcutterlibrary"
            | "cutterlibrary" => Some(GEM_CUTTER_LIBRARY),
            _ => None,
        }
    }

    pub fn handles(feature: &str) -> bool {
        Self::from_feature(feature).is_some()
    }
}

/// One selected stone: its document identity, spec and placement.
#[derive(Debug, Clone, PartialEq)]
pub struct CutterStone {
    pub id: om9_core::identity::ObjectId,
    pub label: String,
    pub spec: GemSpec,
    pub placement: Transform,
}

/// The staged cutter builder.
#[derive(Debug, Clone)]
pub struct GemCutterSession {
    pub tool: CutterTool,
    pub stones: Vec<CutterStone>,
    /// Gem Cutter parameters (also the Gem mode of the library preview).
    pub params: GemCutterParams,
    /// Library preset controls.
    pub library: LibraryParams,
    pub boolean: BooleanMode,
    pub keep: CutterKeep,
    /// The metal body the Interactive Boolean cuts, when one is selected.
    pub target: Option<om9_core::identity::ObjectId>,
    pub target_mesh: Option<Mesh>,
    /// Built cutters, one plan per stone (Gem Cutter) or one cutter per stone (Library).
    pub gem_plans: Vec<GemCutterPlan>,
    pub library_cutters: Vec<LibraryCutter>,
    /// The target minus every cutter, when the Boolean is on.
    pub cut: Option<CutterCut>,
    pub message: String,
}

impl GemCutterSession {
    /// Open one builder for the current selection: the gems become the stones, and the first
    /// selected mesh body that is not a gem becomes the Boolean surface.
    pub fn open(controller: &Controller, tool: CutterTool) -> Result<Self> {
        let selection = controller.selected_ordered();
        let mut stones = Vec::new();
        let mut target = None;
        let mut target_mesh = None;
        for id in &selection {
            let entity = controller
                .document()
                .entities
                .get(id)
                .ok_or(Error::Stale("cutter selection"))?;
            if let Some(record) = gem_record_in(controller.document(), *id) {
                // Bake the placement into the cutter so the committed body sits on the stone.
                stones.push(CutterStone {
                    id: *id,
                    label: record.asset.clone(),
                    spec: record.spec.clone(),
                    placement: entity.placement,
                });
                continue;
            }
            if target.is_none() {
                if let Geometry::Mesh(mesh) = &entity.geometry {
                    if entity.extension(CUTTER_EXTENSION_SLOT).is_none() {
                        target = Some(*id);
                        target_mesh = Some(mesh.clone());
                    }
                }
            }
        }
        if stones.is_empty() {
            return Err(Error::Missing("cutter gems"));
        }
        let params = GemCutterParams::for_gem(&stones[0].spec)?;
        let mut session = Self {
            tool,
            stones,
            params,
            library: LibraryParams::default(),
            boolean: BooleanMode::Off,
            keep: CutterKeep::Preserve,
            target,
            target_mesh,
            gem_plans: Vec::new(),
            library_cutters: Vec::new(),
            cut: None,
            message: String::new(),
        };
        session.recompute()?;
        Ok(session)
    }

    /// Rebuild the cutters and, when the Boolean is on, the cut preview.
    pub fn recompute(&mut self) -> Result<()> {
        self.gem_plans.clear();
        self.library_cutters.clear();
        self.cut = None;
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
            CutterTool::GemCutter => {
                for (label, spec, placement) in &stones {
                    self.gem_plans
                        .push(build_gem_cutter(spec, placement, label, &self.params)?);
                }
            }
            CutterTool::GemCutterLibrary => {
                for (label, spec, placement) in &stones {
                    self.library_cutters.push(build_library_cutter(
                        spec,
                        placement,
                        label,
                        &self.library,
                    )?);
                }
            }
        }
        if self.boolean.cuts() {
            let target = self
                .target_mesh
                .as_ref()
                .ok_or(Error::Missing("cutter boolean surface"))?;
            let bodies = self.body_meshes();
            self.cut = Some(apply_boolean(
                target,
                &bodies,
                self.boolean,
                self.keep,
                CsgOptions::default(),
            )?);
        }
        self.message = self.summary();
        Ok(())
    }

    /// The cutter bodies in world space, in commit order.
    pub fn body_meshes(&self) -> Vec<Mesh> {
        match self.tool {
            CutterTool::GemCutter => self
                .gem_plans
                .iter()
                .flat_map(|plan| plan.bodies.iter().map(|body| body.mesh.clone()))
                .collect(),
            CutterTool::GemCutterLibrary => self
                .library_cutters
                .iter()
                .map(|cutter| cutter.mesh.clone())
                .collect(),
        }
    }

    pub fn total_volume_mm3(&self) -> f64 {
        match self.tool {
            CutterTool::GemCutter => self
                .gem_plans
                .iter()
                .map(|plan| plan.total_volume_mm3)
                .sum(),
            CutterTool::GemCutterLibrary => self
                .library_cutters
                .iter()
                .map(|cutter| cutter.volume_mm3)
                .sum(),
        }
    }

    pub fn min_clearance_mm(&self) -> f64 {
        match self.tool {
            CutterTool::GemCutter => self
                .gem_plans
                .iter()
                .map(|plan| plan.min_clearance_mm)
                .fold(f64::INFINITY, f64::min),
            CutterTool::GemCutterLibrary => self
                .library_cutters
                .iter()
                .map(|cutter| cutter.min_clearance_mm)
                .fold(f64::INFINITY, f64::min),
        }
    }

    pub fn summary(&self) -> String {
        let cutters = self.body_meshes().len();
        let mut text = match self.tool {
            CutterTool::GemCutter => format!(
                "Gem Cutter {} on {} stone{} · {:.3} mm³",
                self.params.mode.label(),
                self.stones.len(),
                if self.stones.len() == 1 { "" } else { "s" },
                self.total_volume_mm3()
            ),
            CutterTool::GemCutterLibrary => format!(
                "Gem Cutter Library {} on {} stone{} · {:.3} mm³",
                self.library.preset.label(),
                self.stones.len(),
                if self.stones.len() == 1 { "" } else { "s" },
                self.total_volume_mm3()
            ),
        };
        if let Some(cut) = &self.cut {
            text.push_str(&format!(
                " · cut {:.3} mm³ ({})",
                cut.removed_mm3,
                if cut.keep_cutters {
                    "preserved"
                } else {
                    "removed"
                }
            ));
        } else if self.boolean.cuts() {
            text.push_str(" · no cut preview");
        }
        text.push_str(&format!(
            " · {cutters} cutter{} · clearance {:.2} mm",
            if cutters == 1 { "" } else { "s" },
            self.min_clearance_mm().max(0.0)
        ));
        text
    }

    /// The command line prompt of the open builder.
    pub fn prompt(&self) -> &'static str {
        match self.tool {
            CutterTool::GemCutter => {
                "Mode=, GirdleScale=, LowerSeatSize=, LowerDepth=, EdgeProfile=, Boolean=, Preserve=, or Enter"
            }
            CutterTool::GemCutterLibrary => {
                "Preset=, ScaleX=, ScaleY=, ZScale=, ZOffset=, Boolean=, Preserve=, or Enter"
            }
        }
    }

    /// The clickable options of the open builder.
    pub fn options(&self) -> Vec<(&'static str, &'static str)> {
        match self.tool {
            CutterTool::GemCutter => {
                let mut options = vec![("Mode=", "Mode=")];
                options.extend(
                    self.params
                        .mode
                        .controls()
                        .iter()
                        .map(|option| (*option, *option)),
                );
                options.extend([
                    ("Boolean=", "Boolean="),
                    ("Preserve=", "Preserve="),
                    ("Reset", ""),
                    ("Enter", ""),
                    ("Cancel", ""),
                ]);
                options
            }
            CutterTool::GemCutterLibrary => vec![
                ("Preset=", "Preset="),
                ("ScaleX=", "ScaleX="),
                ("ScaleY=", "ScaleY="),
                ("ZScale=", "ZScale="),
                ("ZOffset=", "ZOffset="),
                ("Boolean=", "Boolean="),
                ("Preserve=", "Preserve="),
                ("Reset", ""),
                ("Enter", ""),
                ("Cancel", ""),
            ],
        }
    }

    /// Apply one typed option. A refused value keeps the previous preview alive.
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
            let first = self.stones.first().ok_or(Error::Missing("cutter gems"))?;
            self.params = GemCutterParams::for_gem(&first.spec)?;
            self.library = LibraryParams::default();
            self.boolean = BooleanMode::Off;
            self.keep = CutterKeep::Preserve;
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
        match name.as_str() {
            "mode" => {
                let mode = CutterMode::from_tag(value)?;
                self.params = self.params.with_mode(mode);
            }
            "preset" => {
                self.library.preset = CutterPreset::from_tag(value)?;
            }
            "boolean" => {
                self.boolean = BooleanMode::from_tag(value)?;
                if self.boolean.cuts() && self.target.is_none() {
                    return Err(Error::Missing("cutter boolean surface"));
                }
            }
            "preserve" => {
                self.keep = CutterKeep::from_tag(value)?;
                if !self.keep.keeps() && !self.boolean.cuts() {
                    // The manual's Preserve/Remove answer is about the cutter body *after* the
                    // cut; with the Boolean off there is no cut to remove them from.
                    return Err(Error::Invalid("cutter preserve needs a boolean"));
                }
            }
            "girdlescale" => self.params.girdle_scale_percent = number("cutter girdle scale")?,
            "girdlewidth" => self.params.girdle_width_percent = number("cutter girdle width")?,
            "lowertaper" => self.params.lower_taper_percent = number("cutter lower taper")?,
            "lowerdepth" => self.params.lower_depth_percent = number("cutter lower depth")?,
            "lowerseatsize" => {
                self.params.lower_seat_size_percent = number("cutter lower seat size")?
            }
            "lowerseatdepth" => {
                self.params.lower_seat_depth_percent = number("cutter lower seat depth")?
            }
            "upperseatsize" => {
                self.params.upper_seat_size_percent = number("cutter upper seat size")?
            }
            "upperseatheight" => {
                self.params.upper_seat_height_percent = number("cutter upper seat height")?
            }
            "uppertaper" => self.params.upper_taper_percent = number("cutter upper taper")?,
            "upperheight" => self.params.upper_height_percent = number("cutter upper height")?,
            "zoffset" if self.tool == CutterTool::GemCutter && !self.library_is_active() => {
                self.params.z_offset_percent = number("cutter z offset")?
            }
            "zoffset" => self.library.z_offset_mm = number("cutter library z offset")?,
            "scale2d" => {
                let percent = number("cutter 2d scale")?;
                self.library.scale_x_percent = percent;
                self.library.scale_y_percent = percent;
            }
            "scalex" => self.library.scale_x_percent = number("cutter library scale x")?,
            "scaley" => self.library.scale_y_percent = number("cutter library scale y")?,
            "zscale" => self.library.z_scale_percent = number("cutter library z scale")?,
            "edgeprofile" => self.params.edge = EdgeShape::from_tag(value)?,
            "bottomdir" => self.params.bottom = BottomDirection::from_tag(value)?,
            "segments" => {
                let segments = number("cutter segments")?;
                if segments.fract().abs() > 1.0e-9 || segments < 3.0 {
                    return Err(Error::Invalid("cutter segments"));
                }
                self.params.segments = segments as usize;
            }
            "clearance" => self.params.clearance_mm = number("cutter clearance")?,
            _ => {
                let control = rotation_control(&name).or_else(|| y_scale_control(&name));
                match control {
                    Some((kind, index)) => {
                        let value = number("cutter rotation")?;
                        match kind {
                            ControlKind::Rotation => self.params.rotations_deg[index] = value,
                            ControlKind::YScale => {
                                self.params.y_scales_percent[index] = value;
                            }
                        }
                    }
                    None => return Err(Error::Invalid("cutter option")),
                }
            }
        }
        Ok(())
    }

    /// The Gem Cutter owns `ZOffset=`; the Library owns `ZOffset=` in mm. The tool tells them
    /// apart, so the guard below only exists to keep the two branches readable.
    fn library_is_active(&self) -> bool {
        matches!(self.tool, CutterTool::GemCutterLibrary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlKind {
    Rotation,
    YScale,
}

fn rotation_control(name: &str) -> Option<(ControlKind, usize)> {
    let value = name.strip_prefix("rotation")?;
    let index = value.parse::<usize>().ok()?;
    (1..=5)
        .contains(&index)
        .then(|| (ControlKind::Rotation, index - 1))
}

fn y_scale_control(name: &str) -> Option<(ControlKind, usize)> {
    let value = name.strip_prefix("yscale")?;
    let index = value.parse::<usize>().ok()?;
    (1..=5)
        .contains(&index)
        .then(|| (ControlKind::YScale, index - 1))
}

/// The typed payload every committed cutter carries in `om9.cutter`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CutterPayload {
    pub version: u32,
    /// `gem_cutter` or `gem_cutter_library`.
    pub kind: String,
    /// The mode or preset the cutter was built with (the display filter reads this as `seat`).
    pub seat: String,
    pub gem_id: String,
    pub stones: usize,
    pub volume_mm3: f64,
    pub clearance_mm: f64,
    pub boolean: String,
    pub preserve: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_mm3: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl CutterPayload {
    pub fn to_extension(&self) -> Result<om9_core::model::Extension> {
        let payload = serde_json::to_string(self).map_err(|_| Error::Invalid("cutter payload"))?;
        om9_core::model::Extension::new(CUTTER_PAYLOAD_VERSION, payload)
    }
}

impl Controller {
    /// Dispatch `BuilderGemCutter` / `BuilderGemCutterLibrary`.
    pub fn dispatch_gem_cutter(&mut self, feature: &str) -> Result<()> {
        let tool = CutterTool::from_feature(feature).ok_or(Error::Unsupported("cutter command"))?;
        if self.is_running() {
            return Err(Error::Busy);
        }
        let session = GemCutterSession::open(self, tool)?;
        let message = session.message.clone();
        self.gem_cutters = Some(session);
        self.report_measurement(Measurement::with_text(
            tool.feature(),
            0.0,
            MeasureUnit::Count,
            message,
        ));
        Ok(())
    }

    pub fn gem_cutter_session(&self) -> Option<&GemCutterSession> {
        self.gem_cutters.as_ref()
    }

    pub fn gem_cutter_session_mut(&mut self) -> Option<&mut GemCutterSession> {
        self.gem_cutters.as_mut()
    }

    /// The command line prompt of the open builder.
    pub fn gem_cutter_prompt(&self) -> Option<String> {
        self.gem_cutters
            .as_ref()
            .map(|session| session.prompt().to_owned())
    }

    /// Close the builder without committing.
    pub fn gem_cutter_cancel(&mut self) -> bool {
        self.gem_cutters.take().is_some()
    }

    /// A typed option, Enter or Cancel. `None` when no builder is open.
    pub fn gem_cutter_text(&mut self, text: &str) -> Option<Result<()>> {
        let feature = self
            .gem_cutters
            .as_ref()
            .map(|session| session.tool.feature())?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower == "cancel" || lower == "esc" {
            self.gem_cutters = None;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
            return Some(Ok(()));
        }
        if lower == "enter" || trimmed.is_empty() {
            return Some(self.gem_cutter_commit());
        }
        let result = self
            .gem_cutters
            .as_mut()
            .ok_or(Error::Missing("cutter session"))
            .and_then(|session| session.set_option(trimmed));
        if result.is_ok() {
            let message = self
                .gem_cutters
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

    /// Commit the open builder through one host transaction: the Boolean cut of the target,
    /// then the cutter bodies (kept, or removed when the Preserve answer says so).
    pub fn gem_cutter_commit(&mut self) -> Result<()> {
        let session = self
            .gem_cutters
            .take()
            .ok_or(Error::Missing("cutter session"))?;
        let bodies = session.body_meshes();
        if bodies.is_empty() {
            return Err(Error::Invalid("cutter bodies"));
        }
        let layer = self.document().current_layer;
        let mut mutations = Vec::new();
        let removed = session.cut.as_ref().map(|cut| cut.removed_mm3);
        if let Some(cut) = &session.cut {
            let target = session
                .target
                .ok_or(Error::Missing("cutter boolean surface"))?;
            let entity = self
                .document()
                .entities
                .get(&target)
                .ok_or(Error::Stale("cutter boolean surface"))?;
            let Geometry::Mesh(_) = entity.geometry else {
                return Err(Error::Invalid("cutter boolean surface"));
            };
            let source = self.document().stamp(target)?;
            let mut extensions = entity.extensions.clone();
            let record = serde_json::json!({
                "version": CUTTER_PAYLOAD_VERSION,
                "kind": match session.tool {
                    CutterTool::GemCutter => "gem_cutter",
                    CutterTool::GemCutterLibrary => "gem_cutter_library",
                },
                "seat": session.seat_label(),
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
            for (index, mesh) in bodies.iter().enumerate() {
                let mut entity = Entity::new("Gem Cutter", Geometry::Mesh(mesh.clone()), layer);
                let payload = CutterPayload {
                    version: CUTTER_PAYLOAD_VERSION,
                    kind: match session.tool {
                        CutterTool::GemCutter => "gem_cutter".to_owned(),
                        CutterTool::GemCutterLibrary => "gem_cutter_library".to_owned(),
                    },
                    seat: session.seat_label().to_owned(),
                    gem_id: session
                        .stones
                        .get(index)
                        .or_else(|| session.stones.first())
                        .map(|stone| stone.label.clone())
                        .unwrap_or_default(),
                    stones: session.stones.len(),
                    volume_mm3: session.total_volume_mm3(),
                    clearance_mm: session.min_clearance_mm(),
                    boolean: session.boolean.label().to_owned(),
                    preserve: session.keep.label().to_owned(),
                    removed_mm3: removed,
                    params: serde_json::to_value(&session.params).ok(),
                };
                entity
                    .extensions
                    .insert(CUTTER_EXTENSION_SLOT.to_owned(), payload.to_extension()?);
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

impl GemCutterSession {
    /// The mode or preset, as the `seat` field of the `om9.cutter` record.
    pub fn seat_label(&self) -> &str {
        match self.tool {
            CutterTool::GemCutter => self.params.mode.label(),
            CutterTool::GemCutterLibrary => self.library.preset.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_cutter_commands_resolve_and_share_one_handler() {
        assert_eq!(
            CutterTool::from_feature(GEM_CUTTER),
            Some(CutterTool::GemCutter)
        );
        assert_eq!(
            CutterTool::from_feature(GEM_CUTTER_LIBRARY),
            Some(CutterTool::GemCutterLibrary)
        );
        assert_eq!(CutterTool::command_name("gvAzureCutter"), Some(GEM_CUTTER));
        assert_eq!(
            CutterTool::command_name("gvGemCutterLibrary"),
            Some(GEM_CUTTER_LIBRARY)
        );
        assert_eq!(CutterTool::command_name("not a command"), None);
    }

    #[test]
    fn rotation_and_y_scale_controls_are_parsed() {
        assert_eq!(
            rotation_control("rotation3"),
            Some((ControlKind::Rotation, 2))
        );
        assert_eq!(y_scale_control("yscale5"), Some((ControlKind::YScale, 4)));
        assert_eq!(rotation_control("rotation6"), None);
        assert_eq!(y_scale_control("yscale0"), None);
    }
}
