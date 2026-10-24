// SPDX-License-Identifier: LGPL-2.1-or-later
//! Display-only cues for the viewport: gem material placeholders, category visibility,
//! backface orientation and the persisted display preset.
//!
//! These are presentation decisions. They never write geometry, placements, layers or the
//! document journal; callers read them while building a frame.
use om9_core::model::{CUTTER_EXTENSION_SLOT, Document, Entity, GEM_EXTENSION_SLOT};
use serde::{Deserialize, Serialize};

pub const DEFERRED_WAIT_M05: &str = "DEFERRED_WAIT_M05";
pub const DEFERRED_WAIT_M07: &str = "DEFERRED_WAIT_M07";
pub const NEUTRAL_GEM: [u8; 4] = [180, 180, 180, 255];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemCueSource {
    /// The semantic gem provider supplied a colour token.
    Material,
    /// A gem record exists but no M05 material record does.
    NeutralPlaceholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemCue {
    pub color: [u8; 4],
    pub source: GemCueSource,
    /// The explicit token a caller can show in a report.
    pub token: &'static str,
}

/// A gem-like cue is only produced by the `om9.gem` jewellery-semantic extension. A name,
/// layer colour or mesh shape is not a gem.
/// A typed M05 gem material/asset record carried by the `om9.gem` extension.
/// The record is a provider input; this module never invents an asset or a gemstone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GemMaterialRecord {
    pub asset: String,
    pub material: String,
    pub color: [u8; 4],
}

pub fn gem_material(entity: &Entity) -> Option<GemMaterialRecord> {
    let extension = entity.extension(GEM_EXTENSION_SLOT)?;
    let payload: serde_json::Value = serde_json::from_str(&extension.payload).ok()?;
    let asset = payload.get("asset")?.as_str()?.to_owned();
    let material = payload.get("material")?.as_str()?.to_owned();
    let color = payload.get("color").and_then(colour_array)?;
    Some(GemMaterialRecord {
        asset,
        material,
        color,
    })
}

/// A typed M07 cutter record carried by the `om9.cutter` extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CutterRecord {
    pub kind: String,
    pub seat: String,
}

pub fn cutter_record(entity: &Entity) -> Option<CutterRecord> {
    let extension = entity.extension(CUTTER_EXTENSION_SLOT)?;
    let payload: serde_json::Value = serde_json::from_str(&extension.payload).ok()?;
    Some(CutterRecord {
        kind: payload.get("kind")?.as_str()?.to_owned(),
        seat: payload.get("seat")?.as_str()?.to_owned(),
    })
}

pub fn gem_cue(entity: &Entity) -> Option<GemCue> {
    if let Some(material) = gem_material(entity) {
        return Some(GemCue {
            color: material.color,
            source: GemCueSource::Material,
            token: "gem.material",
        });
    }
    let extension = entity.extension(GEM_EXTENSION_SLOT)?;
    let payload: serde_json::Value = serde_json::from_str(&extension.payload).ok()?;
    let colour = payload
        .get("color")
        .or_else(|| payload.get("colour"))
        .or_else(|| payload.get("material_color"))
        .and_then(colour_array);
    Some(match colour {
        Some(color) => GemCue {
            color,
            source: GemCueSource::Material,
            token: "gem.material",
        },
        None => GemCue {
            color: NEUTRAL_GEM,
            source: GemCueSource::NeutralPlaceholder,
            token: DEFERRED_WAIT_M05,
        },
    })
}

fn colour_array(value: &serde_json::Value) -> Option<[u8; 4]> {
    let values = value.as_array()?;
    if values.len() < 3 || values.len() > 4 {
        return None;
    }
    let mut out = [255u8; 4];
    for (index, value) in values.iter().enumerate() {
        let channel = value.as_u64()?;
        if channel > 255 {
            return None;
        }
        out[index] = channel as u8;
    }
    Some(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayCategory {
    Model,
    WorkingCurve,
    Cutter,
}

/// Whether a category has a provider in this build. Cutters wait for M07, so the
/// category is disabled rather than silently hiding unrelated model geometry.
pub fn category_available(document: &Document, category: DisplayCategory) -> bool {
    match category {
        DisplayCategory::Model | DisplayCategory::WorkingCurve => true,
        DisplayCategory::Cutter => document.entities.values().any(is_cutter),
    }
}

pub fn category_of(entity: &Entity) -> DisplayCategory {
    if is_cutter(entity) {
        DisplayCategory::Cutter
    } else if matches!(entity.geometry, om9_core::geometry::Geometry::Curve(_)) {
        DisplayCategory::WorkingCurve
    } else {
        DisplayCategory::Model
    }
}

fn is_cutter(entity: &Entity) -> bool {
    cutter_record(entity).is_some()
        || entity
            .extension(CUTTER_EXTENSION_SLOT)
            .is_some_and(|extension| !extension.payload.trim().is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisibilityFilter {
    pub hide_working_curves: bool,
    pub hide_cutters: bool,
}

impl VisibilityFilter {
    /// Presentation-only visibility. If M07 records are absent the cutter category is disabled,
    /// so this function never hides model geometry and never edits the document.
    pub fn shows(&self, document: &Document, entity: &Entity) -> bool {
        match category_of(entity) {
            DisplayCategory::WorkingCurve => !self.hide_working_curves,
            DisplayCategory::Cutter => {
                !(self.hide_cutters && category_available(document, DisplayCategory::Cutter))
            }
            DisplayCategory::Model => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceSide {
    Front,
    Back,
    EdgeOn,
}

/// The visual side of a face. Negative determinant (a mirrored placement) flips the
/// normal before the view comparison; this function never changes a face ID.
pub fn surface_side(normal: [f64; 3], view_toward_eye: [f64; 3], determinant: f64) -> SurfaceSide {
    if !normal.iter().all(|value| value.is_finite())
        || !view_toward_eye.iter().all(|value| value.is_finite())
        || !determinant.is_finite()
        || determinant == 0.
    {
        return SurfaceSide::EdgeOn;
    }
    let parity = if determinant < 0. { -1. } else { 1. };
    let dot = normal[0] * view_toward_eye[0] * parity
        + normal[1] * view_toward_eye[1] * parity
        + normal[2] * view_toward_eye[2] * parity;
    if dot > 1e-12 {
        SurfaceSide::Front
    } else if dot < -1e-12 {
        SurfaceSide::Back
    } else {
        SurfaceSide::EdgeOn
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayPreferences {
    pub version: u32,
    pub wireframe: bool,
    pub show_edges: bool,
    pub backface_cues: bool,
    pub hide_working_curves: bool,
    pub hide_cutters: bool,
    pub gem_cues: bool,
}

impl Default for DisplayPreferences {
    fn default() -> Self {
        Self {
            version: 1,
            wireframe: true,
            show_edges: true,
            backface_cues: true,
            hide_working_curves: false,
            hide_cutters: false,
            gem_cues: true,
        }
    }
}

impl DisplayPreferences {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| error.to_string())
    }

    /// Parse a saved preset. A missing/old/future schema keeps the sane defaults and returns a
    /// warning instead of panicking or half-applying a malformed record.
    pub fn from_json(text: &str) -> (Self, Option<String>) {
        let Ok(raw) = serde_json::from_str::<serde_json::Value>(text) else {
            return (
                Self::default(),
                Some("display preset schema is not valid JSON".to_owned()),
            );
        };
        let version = raw.get("version").and_then(serde_json::Value::as_u64);
        if version != Some(1) {
            return (
                Self::default(),
                Some(format!(
                    "display preset schema version {} is unsupported",
                    version.map_or_else(|| "missing".to_owned(), |value| value.to_string())
                )),
            );
        }
        let Ok(value) = serde_json::from_value::<Self>(raw) else {
            return (
                Self::default(),
                Some("display preset schema fields are invalid".to_owned()),
            );
        };
        let defaults = Self::default();
        (
            Self {
                version: 1,
                wireframe: value.wireframe,
                show_edges: value.show_edges,
                backface_cues: value.backface_cues,
                hide_working_curves: value.hide_working_curves,
                hide_cutters: value.hide_cutters,
                gem_cues: defaults.gem_cues && value.gem_cues,
            },
            None,
        )
    }

    pub fn apply_to_mode(&self, mode: &mut om9_scene::workspace::DisplayMode) {
        *mode = if self.wireframe {
            om9_scene::workspace::DisplayMode::Wireframe
        } else {
            om9_scene::workspace::DisplayMode::Shaded
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Extension,
    };

    fn entity_with(slot: &str, payload: &str) -> Entity {
        let mut entity = Entity::new("gem", Geometry::Point([0.; 3]), None);
        entity
            .extensions
            .insert(slot.to_owned(), Extension::new(1, payload).unwrap());
        entity
    }

    #[test]
    fn a_gem_colour_token_is_used_when_the_semantic_provider_has_it() {
        let cue = gem_cue(&entity_with(
            GEM_EXTENSION_SLOT,
            r#"{"color":[10,20,30,255]}"#,
        ))
        .expect("gem cue");
        assert_eq!(cue.source, GemCueSource::Material);
        assert_eq!(cue.color, [10, 20, 30, 255]);
        assert_eq!(cue.token, "gem.material");
    }

    #[test]
    fn typed_m05_and_m07_records_are_read_without_inventing_assets() {
        let gem = entity_with(
            GEM_EXTENSION_SLOT,
            r#"{"asset":"round_1","material":"emerald","color":[0,255,0,255]}"#,
        );
        let material = gem_material(&gem).expect("M05 record");
        assert_eq!(material.asset, "round_1");
        assert_eq!(material.material, "emerald");
        assert_eq!(material.color, [0, 255, 0, 255]);
        assert_eq!(gem_cue(&gem).unwrap().source, GemCueSource::Material);

        let mut document = Document::new("cutters");
        let cutter = entity_with(CUTTER_EXTENSION_SLOT, r#"{"kind":"seat","seat":"round"}"#);
        let record = cutter_record(&cutter).expect("M07 record");
        assert_eq!(record.kind, "seat");
        assert!(!category_available(&document, DisplayCategory::Cutter));
        let id = cutter.id;
        document.entities.insert(id, cutter);
        assert!(category_available(&document, DisplayCategory::Cutter));
        let filter = VisibilityFilter {
            hide_working_curves: false,
            hide_cutters: true,
        };
        assert!(!filter.shows(&document, &document.entities[&id]));
    }

    #[test]
    fn a_gem_without_m05_material_is_neutral_and_explicitly_deferred() {
        assert!(gem_cue(&Entity::new("plain", Geometry::Point([0.; 3]), None)).is_none());
        let cue =
            gem_cue(&entity_with(GEM_EXTENSION_SLOT, r#"{"shape":"round"}"#)).expect("gem cue");
        assert_eq!(cue.source, GemCueSource::NeutralPlaceholder);
        assert_eq!(cue.color, NEUTRAL_GEM);
        assert_eq!(cue.token, DEFERRED_WAIT_M05);
    }

    #[test]
    fn a_disabled_cutter_category_never_hides_model_geometry() {
        let mut document = Document::new("visibility");
        let model = Entity::new("mesh", Geometry::Point([0.; 3]), None);
        let curve = Entity::new(
            "curve",
            Geometry::Curve(Curve::Line {
                start: [0.; 3],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let model_id = model.id;
        let curve_id = curve.id;
        document.entities.insert(model_id, model);
        document.entities.insert(curve_id, curve.clone());
        let filter = VisibilityFilter {
            hide_working_curves: true,
            hide_cutters: true,
        };
        assert!(filter.shows(&document, &document.entities[&model_id]));
        assert!(!filter.shows(&document, &curve));
        assert!(!category_available(&document, DisplayCategory::Cutter));
    }

    #[test]
    fn a_mirrored_face_flips_its_side_and_never_touches_an_id() {
        assert_eq!(
            surface_side([0., 0., 1.], [0., 0., 1.], 1.),
            SurfaceSide::Front
        );
        assert_eq!(
            surface_side([0., 0., 1.], [0., 0., 1.], -1.),
            SurfaceSide::Back
        );
        assert_eq!(
            surface_side([0., 0., 1.], [1., 0., 0.], 1.),
            SurfaceSide::EdgeOn
        );
    }

    #[test]
    fn a_display_preset_round_trips_and_a_bad_schema_falls_back_with_a_warning() {
        let preset = DisplayPreferences {
            wireframe: false,
            show_edges: false,
            hide_cutters: true,
            ..DisplayPreferences::default()
        };
        let text = preset.to_json().unwrap();
        let (loaded, warning) = DisplayPreferences::from_json(&text);
        assert_eq!(loaded, preset);
        assert!(warning.is_none());

        let (fallback, warning) = DisplayPreferences::from_json("{not json");
        assert_eq!(fallback, DisplayPreferences::default());
        assert!(warning.is_some());

        let (fallback, warning) =
            DisplayPreferences::from_json(r#"{"version":99,"wireframe":false}"#);
        assert_eq!(fallback, DisplayPreferences::default());
        assert!(warning.unwrap().contains("99"));
    }
}
