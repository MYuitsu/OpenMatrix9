// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem tools (`OM9-L2-04-09`).
//!
//! This module owns the Matrix gem command surface: the Loader (`gvLoader`, cut/shape/size),
//! the Custom Gem builder (`gvCustomGem`), the Emerald and Baguette builders
//! (`gvEmerald`/`gvBaguette`), the Gem Positioner (`gvGemPositioner`) and the F6 Gem
//! Properties editor. It is a dispatcher over the typed `om9_core::gem` owner:
//!
//! * the cut/shape/dimension schema, the canonical library, the procedural mesh and the
//!   girdle/facet reference API live in `om9_core::gem`;
//! * this module turns a command or a window edit into ONE document transaction
//!   (insert/replace/placement/definition), so Undo restores exactly the previous state;
//! * a preview is only the typed spec and its computed mesh in session state. Nothing is
//!   inserted into the document until Apply, so Cancel leaves no phantom object.
//!
//! The original Matrix command names are preserved (`gvLoader`, `gvEmerald`, `gvBaguette`,
//! `gvGemPositioner`) and are reachable from both the command line and the palette icons.

use std::collections::BTreeMap;

use om9_core::{
    Error, Result, curve_ops,
    gem::{
        CustomGemParams, GemCut, GemId, GemInstance, GemLibrary, GemMesh, GemRecord, GemShape,
        GemSize, GemSpec, build_mesh, gem_record, gem_record_in, instance_count,
    },
    geometry::{Geometry, Point3, Transform},
    identity::{DefinitionId, LayerId, ObjectId, SourceStamp},
    layers::PresetSlot,
    model::{Definition, Entity, GEM_EXTENSION_SLOT},
    transaction::{Mutation, TransactionRequest},
};

use super::Controller;

/// `Builder Gem Loader` palette icon / `gvLoader` command (`OM9-GEM-001`).
pub const LOADER: &str = "OM9-GEM-001";
/// `Builder Emerald` palette icon / `gvEmerald` command (`OM9-GEM-013`).
pub const EMERALD: &str = "OM9-GEM-013";
/// `Builder Baguette` palette icon / `gvBaguette` command (`OM9-GEM-014`).
pub const BAGUETTE: &str = "OM9-GEM-014";
/// `Builder Gem Positioner` palette icon / `gvGemPositioner` command (`OM9-GEM-033`).
pub const POSITIONER: &str = "OM9-GEM-033";
/// `Builder Custom Gem` palette icon. The icon catalogued it without a feature id.
pub const CUSTOM: &str = "BuilderCustomGem";
/// The F6 Gem Properties action (`OM9-L3-04-09-007`).
pub const PROPERTIES: &str = "GemProperties";
/// Palette label of the loader.
pub const PALETTE_LOADER: &str = "BuilderGemLoader";
/// Palette label of the emerald builder.
pub const PALETTE_EMERALD: &str = "Emerald";
/// Palette label of the positioner.
pub const PALETTE_POSITIONER: &str = "BuilderGemPositioner";

/// The gem tool an open session represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemTool {
    Loader,
    Custom,
    Emerald,
    Baguette,
    Positioner,
}

impl GemTool {
    pub fn command(self) -> &'static str {
        match self {
            Self::Loader => LOADER,
            Self::Custom => CUSTOM,
            Self::Emerald => EMERALD,
            Self::Baguette => BAGUETTE,
            Self::Positioner => POSITIONER,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Loader => "Gem Loader",
            Self::Custom => "Custom Gem Builder",
            Self::Emerald => "Emerald Builder",
            Self::Baguette => "Baguette Builder",
            Self::Positioner => "Gem Positioner",
        }
    }
    /// The material token an asset starts with. Every gem is procedural; the token only
    /// selects the display cue, never a binary asset.
    pub fn default_material(self) -> &'static str {
        "diamond"
    }
}

/// Whether this module owns the feature id.
pub fn handles(feature: &str) -> bool {
    matches!(
        feature,
        LOADER
            | PALETTE_LOADER
            | EMERALD
            | PALETTE_EMERALD
            | BAGUETTE
            | "BuilderBaguette"
            | POSITIONER
            | PALETTE_POSITIONER
            | CUSTOM
            | PROPERTIES
    )
}

/// The typed command names that reach the same services.
pub fn command_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "gvloader" | "gemloader" | "gem loader" | "loadgem" | "gem" => Some(LOADER),
        "gvemerald" | "emeraldbuilder" | "emerald builder" => Some(EMERALD),
        "gvbaguette" | "baguettebuilder" | "baguette builder" => Some(BAGUETTE),
        "gvgempositioner" | "gempositioner" | "gem positioner" | "positiongem" => Some(POSITIONER),
        "gvcustomgem" | "customgem" | "custom gem" | "customgembuilder" => Some(CUSTOM),
        "gemproperties" | "gem properties" => Some(PROPERTIES),
        _ => None,
    }
}

/// Gem material defaults used by the property editor. The colour is a display default, not a
/// material measurement.
pub fn material_color(material: &str) -> Option<[u8; 4]> {
    match material.trim().to_ascii_lowercase().as_str() {
        "" => None,
        "diamond" => Some([180, 180, 180, 255]),
        "ruby" => Some([190, 30, 45, 255]),
        "sapphire" => Some([30, 60, 160, 255]),
        "emerald" => Some([20, 130, 70, 255]),
        "amethyst" => Some([130, 70, 170, 255]),
        "topaz" => Some([230, 180, 60, 255]),
        "clear" => Some([220, 225, 230, 255]),
        _ => None,
    }
}

/// One open gem tool session. A session is the whole preview: no document object exists until
/// Apply commits one transaction.
#[derive(Debug, Clone)]
pub struct GemSession {
    pub tool: GemTool,
    pub cut: GemCut,
    pub shape: GemShape,
    /// Selected index in the asset's size ladder.
    pub size_index: usize,
    /// Selected page of the size dial (the manual calls it a size dial).
    pub size_page: usize,
    /// Keep the selected gem's original size while changing shape/cut.
    pub keep_original: bool,
    /// Length, width and total depth in millimetres (the manual's custom controls).
    pub dimensions: [f64; 3],
    pub girdle_mm: f64,
    pub corner_mm: f64,
    pub custom: CustomGemParams,
    /// The custom builder's closed planar profile, centred and in millimetres.
    pub profile: Option<Vec<[f64; 2]>>,
    /// The gem being replaced (the loader's optional existing `GemId`).
    pub replace: Option<ObjectId>,
    /// The replaced gem's spec, kept so Keep Original Size can restore its dimensions.
    pub replace_spec: Option<GemSpec>,
    /// The gem the positioner edits.
    pub target: Option<ObjectId>,
    /// The positioner's starting record, so its preview/apply never depends on session order.
    pub target_record: Option<GemRecord>,
    pub origin: Point3,
    pub moved: Point3,
    pub rotation_deg: [f64; 3],
    pub scale: f64,
    pub duplicate: bool,
    /// Explicit target layer; `None` routes through the Gem layer default.
    pub layer: Option<LayerId>,
    pub material: String,
    /// Convert the authored mesh into an editable kernel B-Rep at commit time. Enabled when the
    /// OCCT kernel is compiled in; `BRep=No` keeps the portable mesh body.
    pub brep: bool,
}

impl GemSession {
    fn new(tool: GemTool, controller: &Controller) -> Result<Self> {
        let library = GemLibrary::canonical();
        let selection = controller.selected_ordered();
        let selected_gem = match selection.as_slice() {
            [id] => controller
                .document()
                .entities
                .get(id)
                .and_then(|_| gem_record_in(controller.document(), *id))
                .map(|record| (*id, record)),
            _ => None,
        };
        let mut session = Self {
            tool,
            cut: GemCut::Diamond,
            shape: GemShape::Round,
            size_index: 22,
            size_page: 1,
            keep_original: false,
            dimensions: [6.0, 6.0, 3.9],
            girdle_mm: 0.15,
            corner_mm: 0.0,
            custom: CustomGemParams::default(),
            profile: None,
            replace: None,
            replace_spec: None,
            target: None,
            target_record: None,
            origin: [0.0; 3],
            moved: [0.0; 3],
            rotation_deg: [0.0; 3],
            scale: 1.0,
            duplicate: false,
            layer: None,
            material: tool.default_material().to_owned(),
            brep: cfg!(feature = "occt-brep"),
        };
        match tool {
            GemTool::Loader => {
                if let Some((id, record)) = &selected_gem {
                    session.replace = Some(*id);
                    session.replace_spec = Some(record.spec.clone());
                    session.shape = record.spec.shape;
                    session.cut = record.spec.cut;
                    session.material = record.material.clone();
                    session.dimensions = [
                        record.spec.length_mm,
                        record.spec.width_mm,
                        record.spec.depth_mm(),
                    ];
                    session.girdle_mm = record.spec.girdle_mm;
                    session.corner_mm = record.spec.corner_mm;
                }
                if let Some(asset) = library.asset(session.shape, session.cut)
                    && let Some(index) = asset
                        .sizes
                        .iter()
                        .position(|size| (size.span_mm() - 6.0).abs() < 1.0e-9)
                {
                    session.size_index = index;
                    if let Some(size) = asset.sizes.get(session.size_index) {
                        session.dimensions = [size.length_mm, size.width_mm, size.depth_mm];
                    }
                }
                session.size_page = session.size_index / 12 + 1;
                session.girdle_mm =
                    (session.dimensions[1].min(session.dimensions[0]) * 0.02).max(0.06);
                session.corner_mm = session.shape.default_corner_ratio()
                    * session.dimensions[1].min(session.dimensions[0]);
            }
            GemTool::Emerald => {
                session.shape = GemShape::Emerald;
                session.cut = GemCut::Gem;
                session.dimensions = [8.0, 6.0, 4.2];
                session.corner_mm = 1.2;
                session.girdle_mm = 0.15;
                session.size_index = 0;
            }
            GemTool::Baguette => {
                session.shape = GemShape::Baguette;
                session.cut = GemCut::Gem;
                session.dimensions = [7.0, 3.5, 2.8];
                session.corner_mm = 0.0;
                session.girdle_mm = 0.12;
                session.size_index = 0;
            }
            GemTool::Custom => {
                let profile = controller.selected_profile()?;
                session.profile = Some(profile.clone());
                session.cut = GemCut::Gem;
                let spec = GemSpec::from_profile(&profile, GemCut::Gem, &session.custom)?;
                session.dimensions = [spec.length_mm, spec.width_mm, spec.depth_mm()];
                session.corner_mm = 0.0;
                session.girdle_mm = spec.girdle_mm;
                session.custom = CustomGemParams {
                    facet_count: spec.facet_count,
                    crown_angle_deg: spec.crown_angle_deg,
                    crown_height_mm: spec.crown_height_mm,
                    girdle_mm: spec.girdle_mm,
                    pavilion_depth_mm: spec.pavilion_depth_mm,
                    pavilion_rows: spec.pavilion_rows,
                    bulge: spec.bulge,
                };
                session.size_index = 0;
            }
            GemTool::Positioner => {
                let (id, record) =
                    selected_gem.ok_or(Error::Invalid("gem positioner needs one gem"))?;
                let entity = controller
                    .document()
                    .entities
                    .get(&id)
                    .ok_or(Error::Stale("gem positioner target"))?;
                session.target = Some(id);
                session.target_record = Some(record.clone());
                session.shape = record.spec.shape;
                session.cut = record.spec.cut;
                session.dimensions = [
                    record.spec.length_mm,
                    record.spec.width_mm,
                    record.spec.depth_mm(),
                ];
                session.girdle_mm = record.spec.girdle_mm;
                session.corner_mm = record.spec.corner_mm;
                session.material = record.material.clone();
                session.layer = entity.attributes.layer;
                session.origin = [
                    entity.placement.0[0][3],
                    entity.placement.0[1][3],
                    entity.placement.0[2][3],
                ];
                session.size_index = 0;
            }
        }
        if let Some(asset) = library.asset(session.shape, session.cut) {
            session.size_page =
                session.size_index.min(asset.sizes.len().saturating_sub(1)) / 12 + 1;
        }
        Ok(session)
    }

    /// The selected size of the library asset.
    pub fn size(&self) -> Option<GemSize> {
        let library = GemLibrary::canonical();
        let asset = library.asset(self.shape, self.cut)?;
        let index = self.size_index.min(asset.sizes.len().saturating_sub(1));
        asset.sizes.get(index).copied()
    }

    /// The typed gem the session would commit. This is also the preview: computing it does not
    /// touch the document.
    pub fn spec(&self) -> Result<GemSpec> {
        if self.tool == GemTool::Positioner {
            return self
                .target_record
                .as_ref()
                .map(|record| record.spec.clone())
                .ok_or(Error::Invalid("gem positioner target is not a gem"));
        }
        if let Some(profile) = &self.profile {
            let mut params = self.custom;
            params.girdle_mm = self.girdle_mm;
            let mut spec = GemSpec::from_profile(profile, self.cut, &params)?;
            let param_depth = spec.depth_mm();
            if (self.dimensions[2] - param_depth).abs() > 1.0e-9 {
                spec = spec.set_depth_mm(self.dimensions[2])?;
            }
            return Ok(spec);
        }
        if self.keep_original
            && let Some(old) = &self.replace_spec
        {
            let mut spec = GemSpec::from_outer_dimensions(
                self.shape,
                self.cut,
                old.length_mm,
                old.width_mm,
                old.depth_mm(),
            )?;
            spec.girdle_mm = old.girdle_mm;
            spec.corner_mm = old.corner_mm;
            spec = spec.set_depth_mm(old.depth_mm())?;
            spec.validate()?;
            return Ok(spec);
        }
        let asset = GemLibrary::canonical()
            .asset(self.shape, self.cut)
            .cloned()
            .ok_or(Error::Invalid("gem shape for cut"))?;
        let mut spec = GemSpec::from_outer_dimensions(
            self.shape,
            self.cut,
            self.dimensions[0],
            self.dimensions[1],
            self.dimensions[2],
        )?;
        spec.facet_count = asset.facet_count;
        spec.pavilion_rows = asset.pavilion_rows;
        spec.bulge = asset.bulge;
        spec.girdle_mm = self.girdle_mm;
        if self.shape.default_corner_ratio() > 0.0 || self.corner_mm > 0.0 {
            spec.corner_mm = self.corner_mm;
        }
        // The typed depth is the total depth, so it must be re-split against the chosen girdle;
        // otherwise overriding the girdle thickness would silently change the total depth.
        spec = spec.set_depth_mm(self.dimensions[2])?;
        spec.validate()?;
        Ok(spec)
    }

    /// The built preview mesh of the session's current parameters.
    pub fn preview(&self) -> Result<GemMesh> {
        let mut spec = self.spec()?;
        // The positioner's Scale edits the committed spec; the preview must show the same size or
        // the overlay would disagree with the next Apply.
        if self.tool == GemTool::Positioner && (self.scale - 1.0).abs() > 1.0e-12 {
            let length = spec.length_mm * self.scale;
            let width = spec.width_mm * self.scale;
            let depth = spec.depth_mm() * self.scale;
            spec = spec.set_girdle_mm(length, width)?.set_depth_mm(depth)?;
            spec.validate()?;
        }
        build_mesh(&spec)
    }

    /// One line describing the preview for the window.
    pub fn summary(&self) -> String {
        match self.preview() {
            Ok(mesh) => format!(
                "{:.2} x {:.2} x {:.2} mm · {:.2} mm3 · {}",
                mesh.extents_mm[0],
                mesh.extents_mm[1],
                mesh.extents_mm[2],
                mesh.volume_mm3,
                if mesh.closed { "closed" } else { "open" }
            ),
            Err(error) => format!("refused: {error}"),
        }
    }
}

/// The session preview in world space. Painting it never mutates the document.
#[derive(Debug, Clone, PartialEq)]
pub struct GemPreview {
    pub mesh: GemMesh,
    pub placement: Transform,
}

/// The F6 gem property editor state.
#[derive(Debug, Clone)]
pub struct GemPropertiesEditor {
    pub target: ObjectId,
    pub record: GemRecord,
    pub label: String,
    pub material: String,
    pub color: [u8; 4],
    pub layer: Option<LayerId>,
    pub length_mm: f64,
    pub width_mm: f64,
    pub depth_mm: f64,
    pub girdle_mm: f64,
    pub corner_mm: f64,
    /// How many placed instances share the definition (0 for a plain mesh entity).
    pub shared: usize,
    pub status: Option<String>,
}

impl GemPropertiesEditor {
    pub fn shared_definition(&self) -> bool {
        self.shared > 1
    }
    pub fn editable_geometry(&self) -> bool {
        self.shared == 0
    }
    pub fn summary(&self) -> String {
        format!(
            "{} · {} · {:.2} x {:.2} x {:.2} mm{}",
            self.label,
            self.material,
            self.length_mm,
            self.width_mm,
            self.depth_mm,
            if self.shared_definition() {
                " · used by several instances"
            } else {
                ""
            }
        )
    }
}

impl Controller {
    /// Open the loader, a builder or the positioner. The caller has already checked
    /// `availability`; this method does the selection and transaction work.
    pub fn dispatch_gem(&mut self, feature: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let feature = match feature {
            PALETTE_LOADER => LOADER,
            PALETTE_EMERALD => EMERALD,
            "BuilderBaguette" => BAGUETTE,
            PALETTE_POSITIONER => POSITIONER,
            other => other,
        };
        if feature == PROPERTIES {
            return self.open_gem_properties();
        }
        let tool = match feature {
            LOADER => GemTool::Loader,
            CUSTOM => GemTool::Custom,
            EMERALD => GemTool::Emerald,
            BAGUETTE => GemTool::Baguette,
            POSITIONER => GemTool::Positioner,
            _ => return Err(Error::Unsupported("gem command")),
        };
        if self.gem_editor.is_some() {
            return Err(Error::Busy);
        }
        let session = GemSession::new(tool, self)?;
        // The session is the preview; building it once proves the initial parameters are legal.
        let _ = session.preview()?;
        self.gem_session = Some(session);
        self.events.emit(
            crate::command_events::EventKind::Started,
            tool.command(),
            tool.title(),
            None,
        );
        Ok(())
    }

    /// The open gem tool, if any.
    pub fn gem_session(&self) -> Option<&GemSession> {
        self.gem_session.as_ref()
    }
    pub fn gem_session_mut(&mut self) -> Option<&mut GemSession> {
        self.gem_session.as_mut()
    }
    pub fn gem_editor(&self) -> Option<&GemPropertiesEditor> {
        self.gem_editor.as_ref()
    }
    pub fn gem_editor_mut(&mut self) -> Option<&mut GemPropertiesEditor> {
        self.gem_editor.as_mut()
    }
    /// Ephemeral preview of the open tool: no document object is created.
    pub fn gem_preview(&self) -> Result<GemMesh> {
        self.gem_session
            .as_ref()
            .ok_or(Error::Missing("gem session"))?
            .preview()
    }

    /// Cancel: the session is dropped and the document was never touched.
    pub fn gem_cancel(&mut self) -> bool {
        let Some(session) = self.gem_session.take() else {
            return false;
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            session.tool.command(),
            "gem canceled",
            Some(crate::command_events::EventOutcome::Cancelled),
        );
        true
    }

    /// Apply the open tool as ONE transaction (Insert, or Geometry+Extensions for a replacement,
    /// or Placement/DefinitionInsert for the positioner).
    pub fn gem_apply(&mut self) -> Result<()> {
        let session = self
            .gem_session
            .clone()
            .ok_or(Error::Missing("gem session"))?;
        let result = if session.tool == GemTool::Positioner {
            self.apply_gem_positioner(&session)
        } else {
            self.apply_gem_spec(&session)
        };
        match result {
            Ok(()) => {
                self.gem_session = None;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    session.tool.command(),
                    "gem applied",
                    None,
                );
                Ok(())
            }
            Err(error) => {
                self.gem_session = Some(session);
                Err(error)
            }
        }
    }

    fn apply_gem_spec(&mut self, session: &GemSession) -> Result<()> {
        let spec = session.spec()?;
        spec.validate()?;
        let mesh = build_mesh(&spec)?;
        if !mesh.closed {
            return Err(Error::Invalid("gem mesh open"));
        }
        let layer = session.layer.or(self.gem_layer()?);
        if let Some(target) = session.replace {
            let source = self.document().stamp(target)?;
            let entity = self
                .document()
                .entities
                .get(&target)
                .ok_or(Error::Stale("gem replacement"))?;
            if matches!(entity.geometry, Geometry::Instance(_)) {
                return Err(Error::Invalid(
                    "gem instance geometry is owned by a definition",
                ));
            }
            self.ensure_layer_writable(entity.attributes.layer)?;
            self.ensure_layer_writable(layer)?;
            let mut record = GemRecord::new(spec.clone()).with_material(
                &spec.asset_key(),
                &session.material,
                material_color(&session.material).unwrap_or([180, 180, 180, 255]),
            );
            if let Some(old) = gem_record(entity) {
                record.gem_id = old.gem_id;
            } else {
                record.gem_id = GemId(entity.id);
            }
            let mut extensions = entity.extensions.clone();
            extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
            let mut mutations = vec![
                Mutation::Geometry {
                    source,
                    geometry: self.gem_geometry(mesh, session.brep)?,
                },
                Mutation::Extensions { source, extensions },
            ];
            if entity.attributes.layer != layer {
                let mut attributes = entity.attributes.clone();
                attributes.layer = layer;
                mutations.push(Mutation::Attributes { source, attributes });
            }
            self.commit_gem("Gem reload", mutations)
        } else {
            self.ensure_layer_writable(layer)?;
            let mut record = GemRecord::new(spec.clone()).with_material(
                &spec.asset_key(),
                &session.material,
                material_color(&session.material).unwrap_or([180, 180, 180, 255]),
            );
            record.gem_id = GemId::new();
            let mut entity = Entity::new(
                format!("{} {}", spec.shape.label(), spec.cut.label()),
                self.gem_geometry(mesh, session.brep)?,
                layer,
            );
            entity.placement = Transform::translation(session.origin);
            entity
                .extensions
                .insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
            self.commit_gem(
                "Gem loader",
                vec![Mutation::Insert {
                    document: self.document,
                    entity,
                }],
            )
        }
    }

    fn apply_gem_positioner(&mut self, session: &GemSession) -> Result<()> {
        let target = session
            .target
            .ok_or(Error::Missing("gem positioner target"))?;
        let source = self.document().stamp(target)?;
        let entity = self
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Stale("gem positioner target"))?
            .clone();
        if session.duplicate {
            let record = gem_record_in(self.document(), target)
                .ok_or(Error::Invalid("gem positioner target is not a gem"))?;
            let keep_brep = matches!(&entity.geometry, Geometry::Brep(_));
            let definition_member = {
                let mut member = Entity::new(
                    format!("{} {}", record.spec.shape.label(), record.spec.cut.label()),
                    entity.geometry.clone(),
                    None,
                );
                member.extensions = entity.extensions.clone();
                let mesh = build_mesh(&record.spec)?;
                member.geometry = self.gem_geometry(mesh, keep_brep)?;
                member
            };
            let definition = Definition::new(record.spec.asset_key(), vec![definition_member]);
            let definition_id = definition.id;
            let instance = Entity::new(
                format!("{} instance", record.spec.shape.label()),
                Geometry::Instance(definition_id),
                entity.attributes.layer,
            );
            let offset = (record.spec.length_mm * 1.15).max(1.0);
            let placement = Transform::translation([offset, 0.0, 0.0]).compose(entity.placement)?;
            let mut instance = instance;
            instance.placement = placement;
            let revision = self.document().revision;
            self.commit_gem(
                "Gem duplicate",
                vec![
                    Mutation::DefinitionInsert {
                        document: self.document,
                        expected_revision: revision,
                        definition: Box::new(definition),
                        instances: vec![instance],
                    },
                    Mutation::Geometry {
                        source,
                        geometry: Geometry::Instance(definition_id),
                    },
                    Mutation::Placement {
                        source,
                        placement: entity.placement,
                    },
                ],
            )
        } else {
            let placement = self.gem_placement(&entity.placement, session)?;
            let mut mutations = vec![Mutation::Placement { source, placement }];
            if (session.scale - 1.0).abs() > 1.0e-12 {
                let record = gem_record_in(self.document(), target)
                    .ok_or(Error::Invalid("gem positioner target is not a gem"))?;
                if matches!(entity.geometry, Geometry::Instance(_)) {
                    return Err(Error::Invalid(
                        "gem instance geometry is owned by a definition",
                    ));
                }
                let length = record.spec.length_mm * session.scale;
                let width = record.spec.width_mm * session.scale;
                let depth = record.spec.depth_mm() * session.scale;
                let spec = record
                    .spec
                    .set_girdle_mm(length, width)?
                    .set_depth_mm(depth)?;
                let mesh = build_mesh(&spec)?;
                let keep_brep = matches!(&entity.geometry, Geometry::Brep(_));
                let mut updated = GemRecord::new(spec.clone()).with_material(
                    &record.asset,
                    &record.material,
                    record.color,
                );
                updated.gem_id = record.gem_id;
                let mut extensions = entity.extensions.clone();
                extensions.insert(GEM_EXTENSION_SLOT.to_owned(), updated.as_extension()?);
                mutations.push(Mutation::Geometry {
                    source,
                    geometry: self.gem_geometry(mesh, keep_brep)?,
                });
                mutations.push(Mutation::Extensions { source, extensions });
            }
            self.commit_gem("Gem positioner", mutations)
        }
    }

    fn gem_placement(&self, base: &Transform, session: &GemSession) -> Result<Transform> {
        let center = [base.0[0][3], base.0[1][3], base.0[2][3]];
        let rotation = GemInstance::transform([0.0; 3], session.rotation_deg, 1.0)?;
        let to_origin = Transform::translation([-center[0], -center[1], -center[2]]);
        let back = Transform::translation(center);
        let rotated = back.compose(rotation)?.compose(to_origin)?.compose(*base)?;
        Transform::translation(session.moved).compose(rotated)
    }

    /// Build the committed body for a preview mesh. The kernel conversion owns the same authored
    /// boundary as the mesh, so a B-Rep gem is still described by one `GemRecord`; an open or
    /// non-manifold mesh is refused instead of being sealed.
    pub(super) fn gem_geometry(&self, mesh: GemMesh, brep: bool) -> Result<Geometry> {
        #[cfg(feature = "occt-brep")]
        if brep {
            let shape = om9_kernel::gem::gem_brep(&mesh, self.coordinator.limits())
                .map_err(|_| Error::Invalid("gem B-Rep conversion"))?;
            return Ok(Geometry::Brep(shape));
        }
        #[cfg(not(feature = "occt-brep"))]
        let _ = brep;
        Ok(Geometry::Mesh(mesh.mesh))
    }

    /// The session preview in world space: the mesh, its placement and nothing written to the
    /// document. The viewport overlay paints this so the loader/positioner show the gem before Apply.
    pub fn gem_world_preview(&self) -> Option<GemPreview> {
        let session = self.gem_session.as_ref()?;
        let mesh = session.preview().ok()?;
        let placement = if let Some(target) = session.target {
            let entity = self.document().entities.get(&target)?;
            self.gem_placement(&entity.placement, session).ok()?
        } else if let Some(replace) = session.replace {
            self.document().entities.get(&replace)?.placement
        } else {
            Transform::translation(session.origin)
        };
        Some(GemPreview { mesh, placement })
    }

    /// The layer the next gem routes to: an explicitly chosen Gem layer wins, then the Gem 01
    /// preset, then the document's current layer. This is the L3-007 layer default.
    pub fn gem_layer(&self) -> Result<Option<LayerId>> {
        let gem_slot = PresetSlot::new(5).map_err(|_| Error::Invalid("gem layer slot"))?;
        if let Some(current) = self.document().current_layer
            && let Some(layer) = self.document().layers.get(&current)
            && layer.preset_slot.is_some_and(|slot| {
                crate::layer_command::group(slot) == crate::layer_command::group(gem_slot)
            })
        {
            return Ok(Some(current));
        }
        if let Some(id) = self.layer_id(gem_slot) {
            return Ok(Some(id));
        }
        Ok(self.document().current_layer)
    }

    pub(super) fn ensure_layer_writable(&self, layer: Option<LayerId>) -> Result<()> {
        let Some(id) = layer else {
            return Ok(());
        };
        let Some(layer) = self.document().layers.get(&id) else {
            return Ok(());
        };
        if layer.locked {
            return Err(Error::Invalid("gem layer locked"));
        }
        if !layer.visible {
            return Err(Error::Invalid("gem layer hidden"));
        }
        Ok(())
    }

    pub(super) fn commit_gem(&mut self, name: &str, mutations: Vec<Mutation>) -> Result<()> {
        if mutations.is_empty() {
            return Ok(());
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("gem scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("gem scene"));
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            // Gems change geometry, placement and definitions, which the incremental
            // `update_owned` path does not cover; a full capture keeps the display exactly the
            // committed world.
            self.refresh_scene()?;
        }
        Ok(())
    }

    /// Sample the single selected closed planar curve into the custom builder's profile.
    pub(super) fn selected_profile(&self) -> Result<Vec<[f64; 2]>> {
        let selection = self.selected_ordered();
        let id = match selection.as_slice() {
            [id] => *id,
            _ => return Err(Error::Invalid("custom gem needs one closed curve")),
        };
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Stale("custom gem profile"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("custom gem needs a curve"));
        };
        let points = curve_ops::sample_curve(curve, 64, self.coordinator.limits())?;
        let mut world = Vec::with_capacity(points.len());
        for point in &points {
            world.push(entity.placement.point(*point)?);
        }
        let z = world[0][2];
        if world.iter().any(|p| (p[2] - z).abs() > 1.0e-6) {
            return Err(Error::Invalid("custom gem profile must be planar"));
        }
        let scale = world
            .iter()
            .map(|p| p[0].abs().max(p[1].abs()))
            .fold(0.0_f64, f64::max)
            .max(1.0);
        let first = world[0];
        let last = world[world.len() - 1];
        if (first[0] - last[0]).hypot(first[1] - last[1]) > 1.0e-6 * scale {
            return Err(Error::Invalid("custom gem profile is not closed"));
        }
        let mut profile: Vec<[f64; 2]> = world.iter().map(|point| [point[0], point[1]]).collect();
        profile.pop();
        om9_core::gem::validate_profile(&profile)?;
        Ok(profile)
    }

    /// Open the F6 Gem Properties editor for the single selected gem.
    pub fn open_gem_properties(&mut self) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let selection = self.selected_ordered();
        let id = match selection.as_slice() {
            [id] => *id,
            _ => return Err(Error::Invalid("gem properties needs one gem")),
        };
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Stale("gem properties target"))?
            .clone();
        let record = gem_record_in(self.document(), id)
            .ok_or(Error::Invalid("gem properties needs a gem"))?;
        self.ensure_layer_writable(entity.attributes.layer)?;
        let shared = match entity.geometry {
            Geometry::Instance(definition) => instance_count(self.document(), definition),
            _ => 0,
        };
        self.gem_editor = Some(GemPropertiesEditor {
            target: id,
            label: entity.label.clone(),
            material: record.material.clone(),
            color: record.color,
            layer: entity.attributes.layer,
            length_mm: record.spec.length_mm,
            width_mm: record.spec.width_mm,
            depth_mm: record.spec.depth_mm(),
            girdle_mm: record.spec.girdle_mm,
            corner_mm: record.spec.corner_mm,
            record,
            shared,
            status: None,
        });
        Ok(())
    }

    /// Apply the property editor in one transaction: label, colour/material, layer and (when the
    /// gem is not shared) the regenerated mesh.
    pub fn gem_properties_apply(&mut self) -> Result<()> {
        let editor = self
            .gem_editor
            .clone()
            .ok_or(Error::Missing("gem properties editor"))?;
        if editor.label.trim().is_empty() {
            return Err(Error::Invalid("gem label"));
        }
        if editor.material.trim().is_empty() {
            return Err(Error::Invalid("gem material"));
        }
        if editor.shared_definition() {
            return Err(Error::Invalid("shared gem definition"));
        }
        self.ensure_layer_writable(editor.layer)?;
        let source = self.document().stamp(editor.target)?;
        let entity = self
            .document()
            .entities
            .get(&editor.target)
            .ok_or(Error::Stale("gem properties target"))?
            .clone();
        let geometry_changed = (editor.length_mm - editor.record.spec.length_mm).abs() > 1.0e-9
            || (editor.width_mm - editor.record.spec.width_mm).abs() > 1.0e-9
            || (editor.depth_mm - editor.record.spec.depth_mm()).abs() > 1.0e-9
            || (editor.girdle_mm - editor.record.spec.girdle_mm).abs() > 1.0e-9
            || (editor.corner_mm - editor.record.spec.corner_mm).abs() > 1.0e-9;
        let mut record = editor.record.clone();
        record.material = editor.material.clone();
        record.color = editor.color;
        let mut mutations = Vec::new();
        if geometry_changed {
            if matches!(entity.geometry, Geometry::Instance(_)) {
                return Err(Error::Invalid(
                    "gem instance geometry is owned by a definition",
                ));
            }
            let mut spec = record.spec.clone();
            spec = spec.set_girdle_mm(editor.length_mm, editor.width_mm)?;
            spec.girdle_mm = editor.girdle_mm;
            spec.corner_mm = editor.corner_mm;
            spec = spec.set_depth_mm(editor.depth_mm)?;
            spec.validate()?;
            record.spec = spec;
            let mesh = build_mesh(&record.spec)?;
            let keep_brep = matches!(&entity.geometry, Geometry::Brep(_));
            mutations.push(Mutation::Geometry {
                source,
                geometry: self.gem_geometry(mesh, keep_brep)?,
            });
        }
        let mut extensions = entity.extensions.clone();
        extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
        mutations.push(Mutation::Extensions { source, extensions });
        let mut attributes = entity.attributes.clone();
        attributes.layer = editor.layer;
        attributes.color = Some(editor.color);
        if attributes != entity.attributes {
            mutations.push(Mutation::Attributes { source, attributes });
        }
        if editor.label != entity.label {
            mutations.push(Mutation::Label {
                source,
                label: editor.label.clone(),
            });
        }
        self.commit_gem("Gem properties", mutations)?;
        // A property edit is one command: once committed the editor closes, so the panel never
        // keeps the controller busy and Undo is immediately available.
        self.gem_editor = None;
        Ok(())
    }

    pub fn gem_properties_close(&mut self) {
        self.gem_editor = None;
    }

    /// Command-line / Enter handling for an open gem tool. `None` means no gem tool is open.
    pub fn gem_text(&mut self, text: &str) -> Option<Result<()>> {
        self.gem_session.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.is_empty() || lower == "apply" || lower == "enter" {
            return Some(self.gem_apply());
        }
        if lower == "cancel" || lower == "esc" || lower == "exit" {
            self.gem_cancel();
            return Some(Ok(()));
        }
        if let Some(result) = self.gem_option(trimmed) {
            return Some(result);
        }
        Some(Err(Error::Invalid("gem option")))
    }

    fn gem_option(&mut self, text: &str) -> Option<Result<()>> {
        let (key, value) = text.split_once('=')?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let mut session = self.gem_session.clone()?;
        let layers = self.document().layers.clone();
        let result = (|| -> Result<()> {
            match key.as_str() {
                "shape" => {
                    let shape = GemShape::from_tag(value)?;
                    if !session.cut.shapes().contains(&shape) {
                        return Err(Error::Invalid("gem shape for cut"));
                    }
                    session.shape = shape;
                    session.size_index = 0;
                    session.corner_mm = shape.default_corner_ratio()
                        * session.dimensions[1].min(session.dimensions[0]);
                }
                "cut" => {
                    let cut = GemCut::from_tag(value)?;
                    if !cut.shapes().contains(&session.shape) {
                        return Err(Error::Invalid("gem shape for cut"));
                    }
                    session.cut = cut;
                    session.size_index = 0;
                }
                "size" => {
                    let value = value.trim_end_matches("mm").trim();
                    let (length, width) = match value.split_once('x') {
                        Some((length, width)) => (
                            length
                                .trim()
                                .parse::<f64>()
                                .map_err(|_| Error::Invalid("gem size"))?,
                            width
                                .trim()
                                .parse::<f64>()
                                .map_err(|_| Error::Invalid("gem size"))?,
                        ),
                        None => {
                            let diameter = value
                                .parse::<f64>()
                                .map_err(|_| Error::Invalid("gem size"))?;
                            (diameter * session.shape.default_aspect(), diameter)
                        }
                    };
                    // A typed size may be off the authored ladder; the dial still keeps the
                    // nearest preset selected while the exact dimensions are used.
                    let asset = GemLibrary::canonical()
                        .asset(session.shape, session.cut)
                        .cloned()
                        .ok_or(Error::Invalid("gem shape for cut"))?;
                    let index = asset
                        .sizes
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            let da = (a.length_mm - length).abs() + (a.width_mm - width).abs();
                            let db = (b.length_mm - length).abs() + (b.width_mm - width).abs();
                            da.total_cmp(&db)
                        })
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    let size = asset.sizes[index];
                    session.size_index = index;
                    session.size_page = index / 12 + 1;
                    session.dimensions = [length, width, size.depth_mm];
                }
                "length" => {
                    session.dimensions[0] = parse_mm(value)?;
                }
                "width" => {
                    session.dimensions[1] = parse_mm(value)?;
                }
                "depth" => {
                    session.dimensions[2] = parse_mm(value)?;
                }
                "girdle" => {
                    session.girdle_mm = parse_mm(value)?;
                }
                "corner" => {
                    session.corner_mm = parse_mm(value)?;
                }
                "keeporiginal" | "keep_original" | "keep original size" => {
                    session.keep_original = parse_bool(value)?;
                }
                "material" => {
                    if material_color(value).is_none() {
                        return Err(Error::Invalid("gem material"));
                    }
                    session.material = value.to_ascii_lowercase();
                }
                "facetcount" | "facets" => {
                    session.custom.facet_count = value
                        .parse()
                        .map_err(|_| Error::Invalid("gem facet count"))?;
                }
                "crownangle" => {
                    session.custom.crown_angle_deg = parse_mm(value)?;
                }
                "crownheight" => {
                    session.custom.crown_height_mm = parse_mm(value)?;
                }
                "paviliondepth" => {
                    session.custom.pavilion_depth_mm = parse_mm(value)?;
                }
                "pavilionrows" => {
                    session.custom.pavilion_rows = value
                        .parse()
                        .map_err(|_| Error::Invalid("gem pavilion rows"))?;
                }
                "bulge" => {
                    session.custom.bulge = parse_mm(value)?;
                }
                "move" => {
                    session.moved = parse_point(value)?;
                }
                "rotate" => {
                    session.rotation_deg = parse_point(value)?;
                }
                "scale" => {
                    session.scale = parse_mm(value)?;
                }
                "duplicate" => {
                    session.duplicate = parse_bool(value)?;
                }
                "origin" => {
                    session.origin = parse_point(value)?;
                }
                "brep" => {
                    session.brep = parse_bool(value)?;
                }
                "layer" => {
                    let layer = layers
                        .values()
                        .find(|layer| layer.name.eq_ignore_ascii_case(value))
                        .ok_or(Error::Invalid("gem layer"))?;
                    if layer.locked {
                        return Err(Error::Invalid("gem layer locked"));
                    }
                    session.layer = Some(layer.id);
                }
                _ => return Err(Error::Invalid("gem option")),
            }
            // Every accepted option must still describe a legal gem: preview it now, so the
            // command line refuses a bad value instead of accepting it and failing at Apply.
            if session.tool != GemTool::Positioner {
                session.spec()?;
            }
            Ok(())
        })();
        if result.is_ok() {
            self.gem_session = Some(session);
        }
        Some(result)
    }

    /// The prompt of the open gem tool, or `None` when no gem tool is open.
    pub fn gem_prompt(&self) -> Option<String> {
        let session = self.gem_session.as_ref()?;
        let spec = session.spec().ok();
        let suffix = match spec {
            Some(spec) => format!(
                "{} {:.2}x{:.2}x{:.2} mm",
                spec.shape.label(),
                spec.length_mm,
                spec.width_mm,
                spec.depth_mm()
            ),
            None => "refused".to_owned(),
        };
        Some(match session.tool {
            GemTool::Loader => format!(
                "Gem Loader [{} · {} · {}] (Enter=Apply, Shape=, Cut=, Size=, Length=, Width=, Depth=, KeepOriginal=): ",
                session.cut.label(),
                suffix,
                session.size().map(|size| size.label()).unwrap_or_default()
            ),
            GemTool::Custom => format!(
                "Custom Gem [{} · {} facets] (Enter=Apply, CrownAngle=, FacetCount=, Bulge=): ",
                suffix, session.custom.facet_count
            ),
            GemTool::Emerald => format!(
                "Emerald Builder [{}] (Enter=Apply, Length=, Width=, Corner=, Depth=, Girdle=): ",
                suffix
            ),
            GemTool::Baguette => format!(
                "Baguette Builder [{}] (Enter=Apply, Length=, Width=, Corner=, Depth=, Girdle=): ",
                suffix
            ),
            GemTool::Positioner => format!(
                "Gem Positioner [move {:?} rotate {:?}] (Enter=Apply, Move=x,y,z, Rotate=x,y,z, Scale=, Duplicate=Yes): ",
                session.moved, session.rotation_deg
            ),
        })
    }
}

fn parse_mm(value: &str) -> Result<f64> {
    let value = value.trim_end_matches("mm").trim();
    let number: f64 = value.parse().map_err(|_| Error::Invalid("gem number"))?;
    if !number.is_finite() {
        return Err(Error::Invalid("gem number"));
    }
    Ok(number)
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" | "1" => Ok(true),
        "no" | "false" | "off" | "0" => Ok(false),
        _ => Err(Error::Invalid("gem flag")),
    }
}

fn parse_point(value: &str) -> Result<Point3> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        return Err(Error::Invalid("gem point"));
    }
    let mut point = [0.0; 3];
    for (axis, part) in parts.iter().enumerate() {
        point[axis] = parse_mm(part)?;
    }
    Ok(point)
}

/// The definition a gem instance belongs to, if it is an instance.
pub fn instance_definition(entity: &Entity) -> Option<DefinitionId> {
    match entity.geometry {
        Geometry::Instance(definition) => Some(definition),
        _ => None,
    }
}

/// A gem entity's own payload plus the payload of its definition member, which is what the
/// property editor and the positioner read.
pub fn entity_record(document: &om9_core::model::Document, entity: &Entity) -> Option<GemRecord> {
    gem_record(entity).or_else(|| match entity.geometry {
        Geometry::Instance(definition) => document
            .definitions
            .get(&definition)
            .and_then(|definition| definition.members.iter().find_map(gem_record)),
        _ => None,
    })
}

/// The extension map with `om9.gem` replaced, preserving every other slot.
pub fn with_gem_extension(
    entity: &Entity,
    record: &GemRecord,
) -> Result<BTreeMap<String, om9_core::model::Extension>> {
    let mut extensions = entity.extensions.clone();
    extensions.insert(GEM_EXTENSION_SLOT.to_owned(), record.as_extension()?);
    Ok(extensions)
}

/// The stamp of one gem, for a caller that composes its own transaction.
pub fn gem_source(document: &om9_core::model::Document, id: ObjectId) -> Result<SourceStamp> {
    document.stamp(id)
}
