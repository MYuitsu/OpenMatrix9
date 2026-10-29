// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem instance edits and reports command surface (`OM9-L2-04-13`).
//!
//! Nine Matrix commands reach this module: Gem Positioner/Control/Match Attributes/Save+Load
//! Styles/Gem Update/Gem Map/Gem Report, plus the attribute-history report. Positioner and the
//! F6 properties editor already own their own surfaces (`session::gem`); this module covers the
//! remaining edit and report commands so every one of them shares ONE typed path:
//!
//! * `BuilderGemControl` edits cut/shape/dimensions/material/colour on the selected stones;
//! * `OthersMatchAttributes` copies the masked attributes from the first selected stone;
//! * `OthersSaveStyle` / `OthersLoadStyle` persist a versioned gem style store;
//! * `BuilderGemUpdate` rebuilds each stone's geometry from its stored record;
//! * `BuilderGemMap` and `BuilderGemReporter` write read-only map/report rows;
//! * `GemAttributeHistory` proves the ids and the placement link survive an edit.
//!
//! A mutating command builds every change in session state and commits ONE transaction on
//! Apply, so Undo restores exactly the previous records and geometry. Report commands never
//! touch the document. A failed apply keeps the session open for correction.

use std::path::PathBuf;

use om9_core::{
    Error, Result,
    gem::{GemCut, GemId, GemRecord, GemShape, build_mesh, gem_record, gem_record_in},
    gem_edits::{
        GemEdit, GemMapEntry, GemReport, GemStyle, GemStyleStore, MatchMask, gem_map, gem_report,
        map_line, report_line,
    },
    geometry::Geometry,
    identity::ObjectId,
    model::{Entity, GEM_EXTENSION_SLOT, GEM_PLACEMENT_EXTENSION_SLOT},
    transaction::Mutation,
};

use super::Controller;
use super::gem::material_color;

/// `Builder Gem Control` palette icon / `gvGemControl` (`OM9-L3-04-13-002`).
pub const CONTROL: &str = "BuilderGemControl";
/// `Others Match Attributes` palette icon / `gvMatchAttributes` (`OM9-L3-04-13-003`).
pub const MATCH: &str = "OthersMatchAttributes";
/// `Others Save Style` palette icon / `gvSaveStyle` (`OM9-L3-04-13-004`).
pub const SAVE_STYLE: &str = "OthersSaveStyle";
/// `Others Load Style` palette icon / `gvLoadStyle` (`OM9-L3-04-13-005`).
pub const LOAD_STYLE: &str = "OthersLoadStyle";
/// `Builder Gem Update` palette icon / `gvGemUpdate` (`OM9-L3-04-13-006`).
pub const UPDATE: &str = "BuilderGemUpdate";
/// `Builder Gem Map` palette icon / `gvGemMap` (`OM9-L3-04-13-007`).
pub const MAP: &str = "BuilderGemMap";
/// `Builder Gem Reporter` palette icon / `gvGemReport` (`OM9-L3-04-13-008`).
pub const REPORTER: &str = "BuilderGemReporter";
/// Attribute-history report of the selected stones (`OM9-L3-04-13-009`).
pub const HISTORY: &str = "GemAttributeHistory";

/// The default style store file; `OM9_GEM_STYLE_FILE` overrides it.
const DEFAULT_STYLE_FILE: &str = "gem-styles.json";

/// The edit/report tool an open session represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemEditsTool {
    Control,
    Match,
    SaveStyle,
    LoadStyle,
    Update,
    Map,
    Reporter,
    History,
}

impl GemEditsTool {
    pub fn command(self) -> &'static str {
        match self {
            Self::Control => CONTROL,
            Self::Match => MATCH,
            Self::SaveStyle => SAVE_STYLE,
            Self::LoadStyle => LOAD_STYLE,
            Self::Update => UPDATE,
            Self::Map => MAP,
            Self::Reporter => REPORTER,
            Self::History => HISTORY,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Control => "Gem Control",
            Self::Match => "Match Attributes",
            Self::SaveStyle => "Save Gem Style",
            Self::LoadStyle => "Load Gem Style",
            Self::Update => "Gem Update",
            Self::Map => "Gem Map",
            Self::Reporter => "Gem Reporter",
            Self::History => "Gem Attribute History",
        }
    }
    /// Whether Apply writes to the document or the style file.
    pub fn mutates(self) -> bool {
        matches!(
            self,
            Self::Control | Self::Match | Self::LoadStyle | Self::Update | Self::SaveStyle
        )
    }
    /// Read-only reports close on Enter and never commit.
    pub fn reports(self) -> bool {
        matches!(self, Self::Map | Self::Reporter | Self::History)
    }
}

/// Whether a feature belongs to this family.
pub fn handles(feature: &str) -> bool {
    matches!(
        feature,
        CONTROL | MATCH | SAVE_STYLE | LOAD_STYLE | UPDATE | MAP | REPORTER | HISTORY
    )
}

/// The typed command names that reach the same services.
pub fn command_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "gvgemcontrol" | "gemcontrol" | "gem control" => Some(CONTROL),
        "gvmatchattributes" | "matchattributes" | "match attributes" => Some(MATCH),
        "gvsavestyle" | "savestyle" | "save style" | "save gem style" => Some(SAVE_STYLE),
        "gvloadstyle" | "loadstyle" | "load style" | "load gem style" => Some(LOAD_STYLE),
        "gvgemupdate" | "gemupdate" | "gem update" => Some(UPDATE),
        "gvgemmap" | "gemmap" | "gem map" => Some(MAP),
        "gvgemreport" | "gemreport" | "gem report" | "gemreporter" => Some(REPORTER),
        "gvgemhistory" | "gemhistory" | "gem attribute history" => Some(HISTORY),
        _ => None,
    }
}

/// One planned stone change: the record after the edit and whether the mesh must be rebuilt.
#[derive(Debug, Clone, PartialEq)]
pub struct GemEditPlan {
    pub target: ObjectId,
    pub record: GemRecord,
    pub geometry_changed: bool,
    /// An edited entity label, when `Label=` was used.
    pub label: Option<String>,
}

/// The open edit/report session.
#[derive(Debug, Clone)]
pub struct GemEditsSession {
    pub tool: GemEditsTool,
    /// The selected stones, in selection order (the Match source is the first one).
    pub targets: Vec<ObjectId>,
    pub edit: GemEdit,
    pub label: Option<String>,
    pub mask: MatchMask,
    pub style_name: Option<String>,
    pub style_path: PathBuf,
    /// An explicit `Density=` override; `None` uses the material table.
    pub density: Option<f64>,
    /// Optional export path for the map/report rows.
    pub file: Option<PathBuf>,
    /// Planned document changes for the mutating tools.
    pub plans: Vec<GemEditPlan>,
    /// The typed rows of an open Gem Map, for the viewport overlay.
    pub map_rows: Vec<GemMapEntry>,
    /// The typed report of an open Gem Reporter.
    pub report: Option<GemReport>,
    /// Read-only text rows of the report tools.
    pub lines: Vec<String>,
    /// Style-store summary of Save Style (`replaced` or `added`).
    pub status: Option<String>,
}

impl GemEditsSession {
    /// Number of stones this session would change, or the rows it would print.
    pub fn preview_count(&self) -> usize {
        if self.tool.reports() {
            self.lines.len()
        } else if self.tool == GemEditsTool::SaveStyle {
            1
        } else {
            self.plans.len()
        }
    }

    /// The plan for one stone, for the evidence accessors.
    pub fn plan(&self, target: ObjectId) -> Option<&GemEditPlan> {
        self.plans.iter().find(|plan| plan.target == target)
    }

    pub fn report_text(&self) -> String {
        if let Some(status) = &self.status {
            return status.clone();
        }
        self.lines.join("\n")
    }
}

/// Open the tool for the current selection.
pub fn open(controller: &Controller, tool: GemEditsTool) -> Result<GemEditsSession> {
    let selection = controller.selected_ordered();
    let style_path = std::env::var_os("OM9_GEM_STYLE_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_STYLE_FILE));
    let mut session = GemEditsSession {
        tool,
        targets: Vec::new(),
        edit: GemEdit::default(),
        label: None,
        mask: MatchMask::ALL,
        style_name: None,
        style_path,
        density: None,
        file: None,
        plans: Vec::new(),
        map_rows: Vec::new(),
        report: None,
        lines: Vec::new(),
        status: None,
    };

    if tool.reports() {
        // An empty selection means the whole document; a selection limits the report.
        session.targets = if selection.is_empty() {
            controller
                .document()
                .entities
                .values()
                .filter(|entity| gem_record(entity).is_some())
                .map(|entity| entity.id)
                .collect()
        } else {
            selection
        };
        return Ok(session);
    }

    if selection.is_empty() {
        return Err(Error::Invalid("gem edit needs a gem selection"));
    }
    let mut gems = Vec::new();
    for id in &selection {
        let entity = controller
            .document()
            .entities
            .get(id)
            .ok_or(Error::Stale("gem edit target"))?;
        if gem_record(entity).is_none() {
            return Err(Error::Invalid("gem edit needs gems"));
        }
        if matches!(entity.geometry, Geometry::Instance(_)) {
            return Err(Error::Invalid(
                "gem instance geometry is owned by a definition",
            ));
        }
        controller.ensure_layer_writable(entity.attributes.layer)?;
        gems.push(*id);
    }
    match tool {
        GemEditsTool::Match if gems.len() < 2 => {
            return Err(Error::Invalid(
                "match attributes needs a source and a target",
            ));
        }
        _ => {}
    }
    session.targets = gems;
    Ok(session)
}

/// Recompute the preview from the current session parameters.
pub fn recompute(controller: &Controller, session: &mut GemEditsSession) -> Result<()> {
    session.plans.clear();
    session.lines.clear();
    session.status = None;
    match session.tool {
        GemEditsTool::Control => {
            if session.edit.is_empty() {
                return Err(Error::Invalid("gem edit needs a field"));
            }
            for target in session.targets.clone() {
                let record = controller
                    .document()
                    .entities
                    .get(&target)
                    .and_then(gem_record)
                    .ok_or(Error::Stale("gem control target"))?;
                let edited = session.edit.record(&record)?;
                session.plans.push(GemEditPlan {
                    target,
                    record: edited,
                    geometry_changed: session.edit.changes_geometry(),
                    label: session.label.clone(),
                });
            }
        }
        GemEditsTool::Match => {
            let source = *session
                .targets
                .first()
                .ok_or(Error::Missing("gem match source"))?;
            let source_record = controller
                .document()
                .entities
                .get(&source)
                .and_then(gem_record)
                .ok_or(Error::Stale("gem match source"))?;
            let mut targets = Vec::new();
            for target in session.targets.iter().skip(1) {
                targets.push(
                    controller
                        .document()
                        .entities
                        .get(target)
                        .and_then(gem_record)
                        .ok_or(Error::Stale("gem match target"))?,
                );
            }
            let matched =
                om9_core::gem_edits::match_attributes(&source_record, &targets, session.mask)?;
            for (target, record) in session.targets.iter().skip(1).zip(matched) {
                session.plans.push(GemEditPlan {
                    target: *target,
                    geometry_changed: session.mask.cut
                        || session.mask.shape
                        || session.mask.dimensions,
                    record,
                    label: None,
                });
            }
        }
        GemEditsTool::SaveStyle => {
            let source = *session
                .targets
                .first()
                .ok_or(Error::Missing("gem style source"))?;
            let record = controller
                .document()
                .entities
                .get(&source)
                .and_then(gem_record)
                .ok_or(Error::Stale("gem style source"))?;
            let name = session
                .style_name
                .clone()
                .unwrap_or_else(|| default_style_name(&record));
            let style = GemStyle::from_record(name, &record);
            let mut store = GemStyleStore::load_or_empty(&session.style_path)?;
            let replaced = store.insert(style)?;
            session.status = Some(format!(
                "{} · {} style(s) in {}",
                if replaced { "replaced" } else { "saved" },
                store.len(),
                session.style_path.display()
            ));
            session.plans.push(GemEditPlan {
                target: source,
                record,
                geometry_changed: false,
                label: None,
            });
        }
        GemEditsTool::LoadStyle => {
            let name = session
                .style_name
                .clone()
                .ok_or(Error::Missing("gem style name"))?;
            let store = GemStyleStore::load_or_empty(&session.style_path)?;
            let style = store
                .get(&name)
                .cloned()
                .ok_or(Error::Missing("gem style name"))?;
            for target in session.targets.clone() {
                let record = controller
                    .document()
                    .entities
                    .get(&target)
                    .and_then(gem_record)
                    .ok_or(Error::Stale("gem style target"))?;
                let applied = style.apply(&record)?;
                session.plans.push(GemEditPlan {
                    target,
                    geometry_changed: style.edit.changes_geometry(),
                    record: applied,
                    label: None,
                });
            }
        }
        GemEditsTool::Update => {
            for target in session.targets.clone() {
                let record = controller
                    .document()
                    .entities
                    .get(&target)
                    .and_then(gem_record)
                    .ok_or(Error::Stale("gem update target"))?;
                // Prove the stored record rebuilds before promising an update.
                let mesh = build_mesh(&record.spec)?;
                if !mesh.closed {
                    return Err(Error::Invalid("gem mesh open"));
                }
                session.plans.push(GemEditPlan {
                    target,
                    geometry_changed: true,
                    record,
                    label: None,
                });
            }
        }
        GemEditsTool::Map => {
            let mut entries = Vec::new();
            for target in &session.targets {
                let entity = controller
                    .document()
                    .entities
                    .get(target)
                    .ok_or(Error::Stale("gem map target"))?;
                let record = gem_record(entity).ok_or(Error::Stale("gem map target"))?;
                let point = entity
                    .placement
                    .point([0.0, 0.0, 0.0])
                    .map_err(|_| Error::Invalid("gem map point"))?;
                entries.push((GemId(*target), point, record.spec));
            }
            let rows: Vec<GemMapEntry> = gem_map(&entries)?;
            session.lines = rows.iter().map(map_line).collect();
            session.map_rows = rows;
            if let Some(path) = &session.file {
                write_text_file(path, &session.lines.join("\n"))?;
                session.status = Some(format!(
                    "{} map row(s) written to {}",
                    session.map_rows.len(),
                    path.display()
                ));
            }
        }
        GemEditsTool::Reporter => {
            let mut records = Vec::new();
            for target in &session.targets {
                let entity = controller
                    .document()
                    .entities
                    .get(target)
                    .ok_or(Error::Stale("gem report target"))?;
                records.push(gem_record(entity).ok_or(Error::Stale("gem report target"))?);
            }
            let report = gem_report(&records, session.density)?;
            session.lines = report.rows.iter().map(report_line).collect();
            let worst = report
                .worst_delta_percent
                .map(|delta| format!(" · worst Δ {delta:.1}%"))
                .unwrap_or_default();
            session.lines.push(format!(
                "TOTAL · {} stone(s) · {:.3} ct · {:.3} g{worst} · tolerance ±{:.0}% vs trade formula",
                report.total_count,
                report.total_carat,
                report.total_grams,
                report.tolerance_percent
            ));
            if report.assumed_density > 0 {
                session.lines.push(format!(
                    "ASSUMPTION · {} stone(s) have no known material density; diamond basis used",
                    report.assumed_density
                ));
            }
            if report.outside_tolerance > 0 {
                session.lines.push(format!(
                    "CHECK · {} row(s) fall outside ±{:.0}% of the trade formula",
                    report.outside_tolerance, report.tolerance_percent
                ));
            }
            session.lines.push(
                "NOT A CERTIFICATE · computed from the authored solid; a gemologist must certify \
                 the final weight"
                    .to_owned(),
            );
            if let Some(path) = &session.file {
                write_text_file(
                    path,
                    &serde_json::to_string_pretty(&report)
                        .map_err(|_| Error::Invalid("gem report json"))?,
                )?;
                session.status = Some(format!(
                    "{} stone(s) reported to {}",
                    report.total_count,
                    path.display()
                ));
            }
            session.report = Some(report);
        }
        GemEditsTool::History => {
            for target in &session.targets {
                let entity = controller
                    .document()
                    .entities
                    .get(target)
                    .ok_or(Error::Stale("gem history target"))?;
                let record = gem_record(entity).ok_or(Error::Stale("gem history target"))?;
                let link = entity
                    .extensions
                    .get(GEM_PLACEMENT_EXTENSION_SLOT)
                    .and_then(|slot| {
                        om9_core::gem_placement::PlacementLink::from_extension(slot).ok()
                    });
                let link_text = match link {
                    Some(link) => {
                        format!("link {} rev {}", link.source.tag(), link.source_revision)
                    }
                    None => "no placement link".to_owned(),
                };
                session.lines.push(format!(
                    "#{} · gem {} · entity {} · {} {} · {:.2} × {:.2} mm · {}",
                    session
                        .targets
                        .iter()
                        .position(|id| id == target)
                        .map(|index| index + 1)
                        .unwrap_or(0),
                    record.gem_id.0,
                    entity.id,
                    record.spec.shape.label(),
                    record.spec.cut.label(),
                    record.spec.length_mm,
                    record.spec.width_mm,
                    link_text
                ));
            }
        }
    }
    Ok(())
}

fn default_style_name(record: &GemRecord) -> String {
    format!(
        "{} {:.2} mm",
        record.spec.shape.label(),
        record.spec.length_mm
    )
}

fn write_text_file(path: &std::path::Path, text: &str) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|_| Error::Invalid("gem report directory"))?;
    }
    std::fs::write(path, text).map_err(|_| Error::Invalid("gem report file"))
}

/// Apply the open session: ONE transaction for document edits, an atomic file write for a
/// style save, and nothing at all for the read-only reports.
pub fn apply(controller: &mut Controller, session: &GemEditsSession) -> Result<()> {
    match session.tool {
        GemEditsTool::Control
        | GemEditsTool::Match
        | GemEditsTool::LoadStyle
        | GemEditsTool::Update => {
            if session.plans.is_empty() {
                return Err(Error::Missing("gem edit plans"));
            }
            let mut mutations = Vec::new();
            for plan in &session.plans {
                let entity = controller
                    .document()
                    .entities
                    .get(&plan.target)
                    .ok_or(Error::Stale("gem edit target"))?
                    .clone();
                controller.ensure_layer_writable(entity.attributes.layer)?;
                let source = controller.document().stamp(plan.target)?;
                if plan.geometry_changed {
                    let mesh = build_mesh(&plan.record.spec)?;
                    if !mesh.closed {
                        return Err(Error::Invalid("gem mesh open"));
                    }
                    let brep = matches!(entity.geometry, Geometry::Brep(_));
                    mutations.push(Mutation::Geometry {
                        source,
                        geometry: controller.gem_geometry(mesh, brep)?,
                    });
                }
                let mut extensions = entity.extensions.clone();
                extensions.insert(GEM_EXTENSION_SLOT.to_owned(), plan.record.as_extension()?);
                mutations.push(Mutation::Extensions { source, extensions });
                let edited_label = plan
                    .label
                    .as_deref()
                    .filter(|label| !label.trim().is_empty() && entity.label != *label);
                if let Some(label) = edited_label {
                    mutations.push(Mutation::Label {
                        source,
                        label: label.to_owned(),
                    });
                }
            }
            controller.commit_gem("Gem edit", mutations)
        }
        GemEditsTool::SaveStyle => {
            let source = *session
                .targets
                .first()
                .ok_or(Error::Missing("gem style source"))?;
            let record = controller
                .document()
                .entities
                .get(&source)
                .and_then(gem_record)
                .ok_or(Error::Stale("gem style source"))?;
            let name = session
                .style_name
                .clone()
                .unwrap_or_else(|| default_style_name(&record));
            let mut store = GemStyleStore::load_or_empty(&session.style_path)?;
            store.insert(GemStyle::from_record(name, &record))?;
            store.save_to(&session.style_path)
        }
        GemEditsTool::Map | GemEditsTool::Reporter | GemEditsTool::History => Ok(()),
    }
}

/// The prompt of an open session.
pub fn prompt(session: &GemEditsSession) -> String {
    let options = match session.tool {
        GemEditsTool::Control => {
            "Cut= Shape= Size= Length= Width= Depth= Girdle= Corner= Material= Color= Label="
        }
        GemEditsTool::Match => "Mask= (all|geometry|finish)",
        GemEditsTool::SaveStyle | GemEditsTool::LoadStyle => "Name= File=",
        GemEditsTool::Update => "",
        GemEditsTool::Map | GemEditsTool::History => "File=",
        GemEditsTool::Reporter => "Density= File=",
    };
    if options.is_empty() {
        return format!(
            "{}: {} item(s) — (Enter=Apply, Esc=Cancel): ",
            session.tool.title(),
            session.preview_count()
        );
    }
    format!(
        "{}: {} item(s) — {} (Enter=Apply, Esc=Cancel): ",
        session.tool.title(),
        session.preview_count(),
        options
    )
}

/// Parse and apply one option line; returns the recomputed session.
pub fn option(controller: &Controller, session: &mut GemEditsSession, text: &str) -> Result<()> {
    let (key, value) = match text.split_once('=') {
        Some(pair) => pair,
        None => ("", text),
    };
    let key = key.trim().to_ascii_lowercase();
    let value = value.trim();
    if key.is_empty() {
        return Err(Error::Invalid("gem edit option"));
    }
    let number = |value: &str| -> Result<f64> {
        value
            .trim_end_matches("mm")
            .trim()
            .parse::<f64>()
            .map_err(|_| Error::Invalid("gem edit option"))
    };
    if !option_used(session.tool, &key) {
        return Err(Error::Invalid("gem edit option"));
    }
    match key.as_str() {
        "cut" => session.edit.cut = Some(GemCut::from_tag(value)?),
        "shape" => session.edit.shape = Some(GemShape::from_tag(value)?),
        "size" => {
            let diameter = number(value)?;
            session.edit.length_mm = Some(diameter);
            session.edit.width_mm = Some(diameter);
        }
        "length" => session.edit.length_mm = Some(number(value)?),
        "width" => session.edit.width_mm = Some(number(value)?),
        "depth" => session.edit.depth_mm = Some(number(value)?),
        "girdle" => session.edit.girdle_mm = Some(number(value)?),
        "corner" => session.edit.corner_mm = Some(number(value)?),
        "material" => {
            if value.is_empty() {
                return Err(Error::Invalid("gem edit material"));
            }
            session.edit.material = Some(value.to_owned());
        }
        "color" | "colour" => session.edit.color = Some(parse_color(value)?),
        "label" => {
            if value.is_empty() || value.len() > 120 {
                return Err(Error::Invalid("gem label"));
            }
            session.label = Some(value.to_owned());
        }
        "mask" => session.mask = MatchMask::parse(value)?,
        "name" => {
            if value.is_empty() {
                return Err(Error::Invalid("gem style name"));
            }
            session.style_name = Some(value.to_owned());
        }
        "file" => {
            if value.is_empty() {
                return Err(Error::Invalid("gem report file"));
            }
            // A style command names its store; a report names its export.
            if matches!(
                session.tool,
                GemEditsTool::SaveStyle | GemEditsTool::LoadStyle
            ) {
                session.style_path = PathBuf::from(value);
            } else {
                session.file = Some(PathBuf::from(value));
            }
        }
        "density" => {
            let density = number(value)?;
            // Refuse an impossible basis immediately instead of printing a wrong weight.
            om9_core::gem_edits::validate_density(density)?;
            session.density = Some(density);
        }
        _ => return Err(Error::Invalid("gem edit option")),
    }
    recompute(controller, session)
}

fn option_used(tool: GemEditsTool, key: &str) -> bool {
    match key {
        "cut" | "shape" | "size" | "length" | "width" | "depth" | "girdle" | "corner"
        | "material" | "color" | "colour" | "label" => tool == GemEditsTool::Control,
        "mask" => tool == GemEditsTool::Match,
        "name" => matches!(tool, GemEditsTool::SaveStyle | GemEditsTool::LoadStyle),
        "file" => matches!(
            tool,
            GemEditsTool::SaveStyle
                | GemEditsTool::LoadStyle
                | GemEditsTool::Map
                | GemEditsTool::Reporter
                | GemEditsTool::History
        ),
        "density" => tool == GemEditsTool::Reporter,
        _ => false,
    }
}

fn parse_color(value: &str) -> Result<[u8; 4]> {
    if let Some(hex) = value.trim().strip_prefix('#') {
        if hex.len() == 6 {
            let parse = |range: std::ops::Range<usize>| -> Result<u8> {
                u8::from_str_radix(&hex[range], 16).map_err(|_| Error::Invalid("gem edit color"))
            };
            return Ok([parse(0..2)?, parse(2..4)?, parse(4..6)?, 255]);
        }
        return Err(Error::Invalid("gem edit color"));
    }
    material_color(value).ok_or(Error::Invalid("gem edit color"))
}

/// Whether the entity is a plain gem that an edit may rewrite.
pub fn editable_gem(entity: &Entity) -> Option<GemRecord> {
    if matches!(entity.geometry, Geometry::Instance(_)) {
        return None;
    }
    gem_record(entity)
}

/// A gem record by id, for tests and panels.
pub fn record(controller: &Controller, id: ObjectId) -> Option<GemRecord> {
    gem_record_in(controller.document(), id)
}

/// All gem ids in the document, in stable order.
pub fn all_gems(controller: &Controller) -> Vec<ObjectId> {
    controller
        .document()
        .entities
        .values()
        .filter(|entity| gem_record(entity).is_some())
        .map(|entity| entity.id)
        .collect()
}

/* --------------------------------- controller -------------------------------- */

impl Controller {
    /// Dispatch one edit/report command. A command that needs parameters opens first and asks
    /// for them; a report opens with its rows already computed.
    pub fn dispatch_gem_edits(&mut self, feature: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let feature = command_name(feature).unwrap_or(feature);
        let tool = match feature {
            CONTROL => GemEditsTool::Control,
            MATCH => GemEditsTool::Match,
            SAVE_STYLE => GemEditsTool::SaveStyle,
            LOAD_STYLE => GemEditsTool::LoadStyle,
            UPDATE => GemEditsTool::Update,
            MAP => GemEditsTool::Map,
            REPORTER => GemEditsTool::Reporter,
            HISTORY => GemEditsTool::History,
            _ => return Err(Error::Unsupported("gem edit command")),
        };
        let mut session = open(self, tool)?;
        if matches!(
            tool,
            GemEditsTool::Match
                | GemEditsTool::SaveStyle
                | GemEditsTool::Update
                | GemEditsTool::Map
                | GemEditsTool::Reporter
                | GemEditsTool::History
        ) {
            recompute(self, &mut session)?;
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            tool.command(),
            tool.title(),
            None,
        );
        self.gem_edits = Some(session);
        Ok(())
    }

    /// Cancel the open tool. Returns whether one was open.
    pub fn gem_edits_cancel(&mut self) -> bool {
        let Some(session) = self.gem_edits.take() else {
            return false;
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            session.tool.command(),
            "gem edit canceled",
            Some(crate::command_events::EventOutcome::Cancelled),
        );
        true
    }

    /// Command-line / Enter handling for an open edit/report tool.
    pub fn gem_edits_text(&mut self, text: &str) -> Option<Result<()>> {
        self.gem_edits.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.is_empty() || lower == "apply" || lower == "enter" {
            return Some(self.gem_edits_apply());
        }
        if lower == "cancel" || lower == "esc" || lower == "exit" {
            self.gem_edits_cancel();
            return Some(Ok(()));
        }
        Some(self.gem_edits_option(trimmed))
    }

    pub fn gem_edits_prompt(&self) -> Option<String> {
        self.gem_edits.as_ref().map(prompt)
    }

    fn gem_edits_option(&mut self, text: &str) -> Result<()> {
        let mut session = self
            .gem_edits
            .clone()
            .ok_or(Error::Missing("gem edit session"))?;
        option(self, &mut session, text)?;
        self.gem_edits = Some(session);
        Ok(())
    }

    /// Apply the open tool. A document edit commits ONE transaction; a style save writes the
    /// store atomically; a report just closes. A refusal keeps the session open.
    pub fn gem_edits_apply(&mut self) -> Result<()> {
        let session = self
            .gem_edits
            .clone()
            .ok_or(Error::Missing("gem edit session"))?;
        match apply(self, &session) {
            Ok(()) => {
                self.gem_edits = None;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    session.tool.command(),
                    "gem edit applied",
                    None,
                );
                Ok(())
            }
            Err(error) => {
                self.gem_edits = Some(session);
                Err(error)
            }
        }
    }

    /// The open session, for the panel and the tests.
    pub fn gem_edits_session(&self) -> Option<&GemEditsSession> {
        self.gem_edits.as_ref()
    }

    /// The report rows or the style-store status of the open tool.
    pub fn gem_edits_report_text(&self) -> Option<String> {
        self.gem_edits.as_ref().map(GemEditsSession::report_text)
    }

    /// The planned stone changes of the open tool.
    pub fn gem_edits_plans(&self) -> Vec<GemEditPlan> {
        self.gem_edits
            .as_ref()
            .map(|session| session.plans.clone())
            .unwrap_or_default()
    }

    /// The numbered rows of an open Gem Map, for the viewport overlay and the key window.
    pub fn gem_map_rows(&self) -> Vec<GemMapEntry> {
        self.gem_edits
            .as_ref()
            .filter(|session| session.tool == GemEditsTool::Map)
            .map(|session| session.map_rows.clone())
            .unwrap_or_default()
    }

    /// The typed count/weight report of an open Gem Reporter.
    pub fn gem_report_data(&self) -> Option<GemReport> {
        self.gem_edits
            .as_ref()
            .filter(|session| session.tool == GemEditsTool::Reporter)
            .and_then(|session| session.report.clone())
    }
}
