// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gem Cutter Library profiles (`gvGemCutterLibrary`, `OM9-L3-04-16-003`, manual book 2 pages
//! 279-280).
//!
//! The library is a fixed set of authored cutting shapes — the manual lists Channel Bur, Hart,
//! Truncated Cone, Cone and the "Setting" family — plus four controls that apply to every
//! preset: `Scale X`, `Scale Y`, `Z Scale` and `Z Offset` (Z offset in mm). A preset is a ring
//! table in stone-relative units, so one table serves every gem size and shape.
//!
//! The authored tables are OM9 vector geometry; they are *not* Matrix's binary style library.
//! Each table is measured in fractions of the gem's pavilion depth (Z) and girdle radius
//! (radius), which is exactly how the manual's percentage sizing works.
use crate::{
    Error, Result,
    cutters::{
        CUTTER_SEGMENTS, EdgeShape, EdgeKind, GemCutterReference, MAX_CUTTERS, Ring, loft,
        volume_mm3,
    },
    gem::GemSpec,
    geometry::{Mesh, Transform},
};

/// The ten Gem Cutter Library presets of the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterPreset {
    SettingAzure,
    HeartAzure,
    Setting,
    Straight,
    Brite,
    Grooves,
    ChannelBur,
    Hart,
    TruncatedCone,
    Cone,
}

impl CutterPreset {
    pub const ALL: [Self; 10] = [
        Self::SettingAzure,
        Self::HeartAzure,
        Self::Setting,
        Self::Straight,
        Self::Brite,
        Self::Grooves,
        Self::ChannelBur,
        Self::Hart,
        Self::TruncatedCone,
        Self::Cone,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::SettingAzure => "Setting Azure Cutter",
            Self::HeartAzure => "Heart Azure Cutter",
            Self::Setting => "Setting Cutter",
            Self::Straight => "Straight Cutter",
            Self::Brite => "Brite Cutter",
            Self::Grooves => "Grooves Cutter",
            Self::ChannelBur => "Channel Bur Cutter",
            Self::Hart => "Hart Cutter",
            Self::TruncatedCone => "Truncated Cone Cutter",
            Self::Cone => "Cone Cutter",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        let wanted = tag.trim().to_ascii_lowercase().replace(['-', '_'], " ");
        for preset in Self::ALL {
            if preset.label().eq_ignore_ascii_case(&wanted)
                || preset.tag().eq_ignore_ascii_case(&wanted)
            {
                return Ok(preset);
            }
        }
        match wanted.as_str() {
            "setting azure" | "azure" => Ok(Self::SettingAzure),
            "heart azure" | "heart" => Ok(Self::HeartAzure),
            "straight" => Ok(Self::Straight),
            "brite" | "bright" => Ok(Self::Brite),
            "groove" | "grooves" => Ok(Self::Grooves),
            "channel bur" | "channelbur" | "bur" => Ok(Self::ChannelBur),
            "hart" => Ok(Self::Hart),
            "truncated cone" | "truncatedcone" | "frustum" => Ok(Self::TruncatedCone),
            "cone" => Ok(Self::Cone),
            "" => Err(Error::Missing("cutter preset")),
            _ => Err(Error::Invalid("cutter preset")),
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::SettingAzure => "setting azure",
            Self::HeartAzure => "heart azure",
            Self::Setting => "setting",
            Self::Straight => "straight",
            Self::Brite => "brite",
            Self::Grooves => "grooves",
            Self::ChannelBur => "channel bur",
            Self::Hart => "hart",
            Self::TruncatedCone => "truncated cone",
            Self::Cone => "cone",
        }
    }

    /// The edge profile the preset's hole uses.
    pub fn edge(self) -> EdgeShape {
        match self {
            Self::HeartAzure => EdgeShape::new(EdgeKind::Heart),
            Self::Hart => EdgeShape::new(EdgeKind::Heart),
            Self::Brite => EdgeShape {
                kind: EdgeKind::Brite,
                grooves: 12,
            },
            Self::Grooves => EdgeShape {
                kind: EdgeKind::Grooves,
                grooves: 8,
            },
            Self::ChannelBur => EdgeShape {
                kind: EdgeKind::Brite,
                grooves: 6,
            },
            _ => EdgeShape::default(),
        }
    }

    /// `(z, radius)` in units of the pavilion depth (Z, positive below the girdle) and the gem
    /// girdle radius; bottom ring first. Every table keeps enough room for the stone profile,
    /// so the clearance gate passes on a standard cut.
    pub fn rings(self) -> &'static [(f64, f64)] {
        match self {
            Self::SettingAzure => &[(1.15, 0.50), (0.40, 0.95), (0.00, 1.05), (-0.20, 1.05)],
            Self::HeartAzure => &[(1.10, 0.45), (0.45, 0.95), (0.00, 1.05), (-0.15, 1.06)],
            Self::Setting => &[(0.70, 0.45), (0.35, 0.92), (0.00, 1.05), (-0.30, 1.06)],
            Self::Straight => &[(1.00, 0.95), (0.25, 0.95)],
            Self::Brite => &[(1.00, 0.35), (0.40, 0.80), (0.05, 1.06)],
            Self::Grooves => &[(1.05, 0.50), (0.30, 1.02)],
            Self::ChannelBur => &[(0.95, 0.40), (0.55, 0.85), (0.15, 0.98)],
            Self::Hart => &[(0.90, 0.30), (0.00, 1.05), (-0.10, 1.06)],
            Self::TruncatedCone => &[(1.00, 0.45), (0.10, 1.02)],
            Self::Cone => &[(1.00, 0.00), (0.10, 1.02)],
        }
    }
}

/// The four library controls of the manual, plus the shared segment budget.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LibraryParams {
    pub preset: CutterPreset,
    /// X scale, percentage of the stone size.
    pub scale_x_percent: f64,
    /// Y scale, percentage of the stone size.
    pub scale_y_percent: f64,
    /// Z scale, percentage of the preset's depth.
    pub z_scale_percent: f64,
    /// Z offset, mm.
    pub z_offset_mm: f64,
    pub segments: usize,
}

impl Default for LibraryParams {
    fn default() -> Self {
        Self {
            preset: CutterPreset::SettingAzure,
            scale_x_percent: 100.0,
            scale_y_percent: 100.0,
            z_scale_percent: 100.0,
            z_offset_mm: 0.0,
            segments: CUTTER_SEGMENTS,
        }
    }
}

impl LibraryParams {
    pub fn preset(preset: CutterPreset) -> Self {
        Self {
            preset,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        for (value, name) in [
            (self.scale_x_percent, "cutter library scale x"),
            (self.scale_y_percent, "cutter library scale y"),
            (self.z_scale_percent, "cutter library z scale"),
            (self.z_offset_mm, "cutter library z offset"),
        ] {
            if !value.is_finite() {
                return Err(Error::Invalid(name));
            }
        }
        if !(10.0..=400.0).contains(&self.scale_x_percent)
            || !(10.0..=400.0).contains(&self.scale_y_percent)
        {
            return Err(Error::Invalid("cutter library 2d scale"));
        }
        if !(10.0..=400.0).contains(&self.z_scale_percent) {
            return Err(Error::Invalid("cutter library z scale"));
        }
        if !(-50.0..=50.0).contains(&self.z_offset_mm) {
            return Err(Error::Invalid("cutter library z offset"));
        }
        if !(3..=crate::settings::MAX_INSTANCES).contains(&self.segments) {
            return Err(Error::Invalid("cutter segments"));
        }
        Ok(())
    }

    /// The preset table in the gem's local frame, bottom first.
    pub fn rings(&self, spec: &GemSpec) -> Result<Vec<Ring>> {
        spec.validate()?;
        self.validate()?;
        let reference = GemCutterReference::from_spec(spec)?;
        let girdle = reference.min_radius_mm();
        let scale_x = reference.radius_x / girdle * self.scale_x_percent / 100.0;
        let scale_y = reference.radius_y / girdle * self.scale_y_percent / 100.0;
        let pavilion = reference.pavilion_depth_mm * self.z_scale_percent / 100.0;
        let shape = self.preset.edge();
        let rings = self
            .preset
            .rings()
            .iter()
            .map(|(z, radius)| Ring {
                // `z` is a depth below the girdle, so the local axis (positive up) is negated.
                z: -reference.girdle_half_mm - z * pavilion + self.z_offset_mm,
                radius_x: girdle * radius * scale_x,
                radius_y: girdle * radius * scale_y,
                rotation_deg: 0.0,
                // The edge profile shapes the hole below the girdle; the seat rings stay round
                // so the bearing always clears the stone.
                shape: if *z > 0.05 {
                    shape
                } else {
                    EdgeShape::default()
                },
            })
            .collect::<Vec<_>>();
        Ok(rings)
    }
}

/// One built library cutter.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryCutter {
    pub gem_id: String,
    pub preset: CutterPreset,
    pub mesh: Mesh,
    pub volume_mm3: f64,
    pub min_clearance_mm: f64,
    pub ring_count: usize,
    pub summary: String,
}

/// Build one library cutter for one stone.
pub fn build(
    spec: &GemSpec,
    placement: &Transform,
    gem_id: &str,
    params: &LibraryParams,
) -> Result<LibraryCutter> {
    spec.validate()?;
    placement.validate()?;
    params.validate()?;
    let reference = GemCutterReference::from_spec(spec)?;
    let profile = crate::gem::profile_points(spec)?;
    let rings = params.rings(spec)?;
    let mut min_clearance = f64::INFINITY;
    for ring in &rings {
        let gem = reference.radius_at(&profile, ring.z, 1.0);
        let margin = ring.min_radius() - gem;
        if margin < -1.0e-9 {
            return Err(Error::Invalid("cutter library clearance"));
        }
        min_clearance = min_clearance.min(margin);
    }
    let mesh = loft(&rings, params.segments)?;
    let volume = volume_mm3(&mesh)?;
    if volume <= 1.0e-9 {
        return Err(Error::Invalid("cutter volume"));
    }
    let mesh = crate::cutters::place(&mesh, placement)?;
    let summary = format!(
        "{} on {} · {:.2} mm³ · {:.2} mm Z offset",
        params.preset.label(),
        gem_id,
        volume,
        params.z_offset_mm
    );
    Ok(LibraryCutter {
        gem_id: gem_id.to_owned(),
        preset: params.preset,
        mesh,
        volume_mm3: volume,
        min_clearance_mm: min_clearance,
        ring_count: rings.len(),
        summary,
    })
}

/// Build one library cutter per stone.
pub fn build_for_stones(
    stones: &[(String, GemSpec, Transform)],
    params: &LibraryParams,
) -> Result<Vec<LibraryCutter>> {
    if stones.is_empty() {
        return Err(Error::Missing("cutter gems"));
    }
    if stones.len() > MAX_CUTTERS {
        return Err(Error::Budget("cutter gems"));
    }
    stones
        .iter()
        .map(|(id, spec, placement)| build(spec, placement, id, params))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gem::{GemCut, GemShape};

    fn round_brilliant() -> GemSpec {
        GemSpec {
            shape: GemShape::Round,
            cut: GemCut::Diamond,
            length_mm: 6.5,
            width_mm: 6.5,
            girdle_mm: 0.4,
            crown_height_mm: 1.0,
            pavilion_depth_mm: 2.6,
            crown_angle_deg: 34.5,
            facet_count: 57,
            pavilion_rows: 2,
            bulge: 0.0,
            corner_mm: 0.0,
            profile: None,
        }
    }

    #[test]
    fn every_library_preset_builds_a_closed_cutter() {
        let spec = round_brilliant();
        for preset in CutterPreset::ALL {
            let params = LibraryParams::preset(preset);
            let cutter = build(&spec, &Transform::identity(), "gem:lib", &params)
                .unwrap_or_else(|error| panic!("{preset:?}: {error}"));
            assert!(cutter.volume_mm3 > 0.0, "{preset:?}");
            assert!(
                crate::settings::inspect_solid(&cutter.mesh).expect("closed").closed,
                "{preset:?}"
            );
            assert!(cutter.summary.contains(preset.label()));
        }
    }

    #[test]
    fn the_scale_controls_change_the_built_size() {
        let spec = round_brilliant();
        let mut params = LibraryParams::preset(CutterPreset::Cone);
        let plain = build(&spec, &Transform::identity(), "gem:lib", &params).expect("plain");
        params.scale_x_percent = 150.0;
        params.scale_y_percent = 150.0;
        let wide = build(&spec, &Transform::identity(), "gem:lib", &params).expect("wide");
        assert!(wide.volume_mm3 > plain.volume_mm3 * 2.0);
    }

    #[test]
    fn a_preset_that_would_cut_the_stone_is_refused() {
        let spec = round_brilliant();
        let mut params = LibraryParams::preset(CutterPreset::SettingAzure);
        params.scale_x_percent = 50.0;
        params.scale_y_percent = 50.0;
        assert_eq!(
            build(&spec, &Transform::identity(), "gem:lib", &params).err(),
            Some(Error::Invalid("cutter library clearance"))
        );
    }

    #[test]
    fn an_unknown_preset_name_is_a_typed_refusal() {
        assert_eq!(
            CutterPreset::from_tag("wax model").err(),
            Some(Error::Invalid("cutter preset"))
        );
        assert_eq!(CutterPreset::from_tag("Cone Cutter").expect("cone"), CutterPreset::Cone);
    }
}
