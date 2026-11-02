// SPDX-License-Identifier: LGPL-2.1-or-later
//! `gvAzureCutter` — the Gem Cutter for one or many stones (`OM9-L3-04-16-001/-002`,
//! manual book 2 pages 272-279).
//!
//! The manual's cutter is a lathe around the stone axis with three modes:
//!
//! * **Gem** — the default mode, the fewest controls: girdle scale, girdle width, lower taper /
//!   depth / seat and the upper seat above the girdle;
//! * **Azure** — the same seat plus a through-azure: the body stops at the girdle, and the edge
//!   profile and the 1/2 rotations and Y scales shape the hole;
//! * **Advanced** — every control: five rings (bottom, lower seat, girdle, table, top), each
//!   with its own rotation and Y scale, plus the edge profile.
//!
//! Every dimension is a percentage of the *stone*, exactly as the manual states ("Dimension for
//! gem cutters are based on percentage because the cutter is built relative to the size and
//! shape of the gemstone"): a radius percentage is of the gem's girdle radius, a depth
//! percentage is of the gem's pavilion depth, and a height percentage is of the gem's crown
//! height. `GemCutterParams::for_gem` derives a full parameter set from the measured stone, so a
//! plain Round brilliant opens with a usable cutter instead of an arbitrary one.
use crate::{
    Error, Result,
    cutters::{EdgeShape, GemCutterReference, MAX_CUTTERS, Ring, loft, volume_mm3},
    gem::{GemSpec, profile_points},
    geometry::{Mesh, Transform},
};

/// The three Gem Cutter modes of the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterMode {
    Gem,
    Azure,
    Advanced,
}

impl CutterMode {
    pub const ALL: [Self; 3] = [Self::Gem, Self::Azure, Self::Advanced];

    pub fn label(self) -> &'static str {
        match self {
            Self::Gem => "Gem",
            Self::Azure => "Azure",
            Self::Advanced => "Advanced",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "gem" | "seat" => Ok(Self::Gem),
            "azure" | "azures" | "through" => Ok(Self::Azure),
            "advanced" | "advance" | "adv" => Ok(Self::Advanced),
            _ => Err(Error::Invalid("cutter mode")),
        }
    }

    /// Whether the mode builds the part above the girdle. Azure mode is a through-azure: it
    /// stops at the girdle, so the stone's crown is untouched.
    pub fn builds_upper(self) -> bool {
        !matches!(self, Self::Azure)
    }

    /// Whether the edge profile library is read. Gem mode has "the fewest controls", the
    /// manual puts the Edge Profile option under Azure/Advanced.
    pub fn uses_edge_profile(self) -> bool {
        !matches!(self, Self::Gem)
    }

    /// How many rotation/Y-scale controls the mode publishes, from the bottom ring up.
    pub fn rotation_controls(self) -> usize {
        match self {
            Self::Gem => 0,
            Self::Azure => 2,
            Self::Advanced => 5,
        }
    }

    /// The command options this mode publishes, in builder order.
    pub fn controls(self) -> &'static [&'static str] {
        match self {
            Self::Gem => &[
                "GirdleScale=",
                "GirdleWidth=",
                "LowerTaper=",
                "LowerDepth=",
                "LowerSeatSize=",
                "LowerSeatDepth=",
                "UpperSeatSize=",
                "UpperSeatHeight=",
                "UpperTaper=",
                "UpperHeight=",
                "ZOffset=",
            ],
            Self::Azure => &[
                "GirdleScale=",
                "GirdleWidth=",
                "LowerTaper=",
                "LowerDepth=",
                "LowerSeatSize=",
                "LowerSeatDepth=",
                "EdgeProfile=",
                "BottomDir=",
                "Rotation1=",
                "Rotation2=",
                "YScale1=",
                "YScale2=",
                "ZOffset=",
            ],
            Self::Advanced => &[
                "GirdleScale=",
                "GirdleWidth=",
                "LowerTaper=",
                "LowerDepth=",
                "LowerSeatSize=",
                "LowerSeatDepth=",
                "UpperSeatSize=",
                "UpperSeatHeight=",
                "UpperTaper=",
                "UpperHeight=",
                "Rotation1=",
                "Rotation2=",
                "Rotation3=",
                "Rotation4=",
                "Rotation5=",
                "YScale1=",
                "YScale2=",
                "YScale3=",
                "YScale4=",
                "YScale5=",
                "EdgeProfile=",
                "BottomDir=",
                "ZOffset=",
            ],
        }
    }
}

/// Which way the bottom of the cutter points (manual Azure mode: "Bottom Dir: Pick a point to
/// orient the direction of the bottom of the cutters").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BottomDirection {
    /// The cutter's bottom points down the stone axis (the default).
    Down,
    /// The cutter's bottom points up, so the seat is cut from the other side.
    Up,
}

impl BottomDirection {
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "down" | "bottom" | "-z" => Ok(Self::Down),
            "up" | "top" | "+z" => Ok(Self::Up),
            _ => Err(Error::Invalid("cutter bottom direction")),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Down => "Down",
            Self::Up => "Up",
        }
    }
}

/// The typed `gvAzureCutter` parameters. Every percentage is relative to the stone; `100` means
/// "the stone's own girdle radius / pavilion depth / crown height".
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GemCutterParams {
    pub mode: CutterMode,
    /// Girdle radius as a percentage of the gem's girdle radius.
    pub girdle_scale_percent: f64,
    /// Girdle band thickness as a percentage of the gem's girdle thickness.
    pub girdle_width_percent: f64,
    /// Lower taper as a percentage of the lower-seat radius.
    pub lower_taper_percent: f64,
    /// Bottom depth below the girdle, percentage of the pavilion depth.
    pub lower_depth_percent: f64,
    /// Lower seat radius, percentage of the cutter's girdle radius.
    pub lower_seat_size_percent: f64,
    /// Lower seat depth below the girdle, percentage of the pavilion depth.
    pub lower_seat_depth_percent: f64,
    /// Upper seat (table) radius, percentage of the cutter's girdle radius.
    pub upper_seat_size_percent: f64,
    /// Upper seat height above the girdle, percentage of the crown height.
    pub upper_seat_height_percent: f64,
    /// Upper taper as a percentage of the upper-seat radius.
    pub upper_taper_percent: f64,
    /// Top height above the table, percentage of the crown height.
    pub upper_height_percent: f64,
    /// Whole-cutter axial shift, percentage of the pavilion depth (positive = up).
    pub z_offset_percent: f64,
    /// `1 Rotation`..`5 Rotation`, degrees.
    pub rotations_deg: [f64; 5],
    /// `1 Y Scale`..`5 Y Scale`, percentage.
    pub y_scales_percent: [f64; 5],
    /// Hole/seat edge profile.
    pub edge: EdgeShape,
    /// Air kept between the cutter and the stone, mm. Zero makes the cutter touch the stone.
    pub clearance_mm: f64,
    pub bottom: BottomDirection,
    pub segments: usize,
}

impl Default for GemCutterParams {
    fn default() -> Self {
        Self {
            mode: CutterMode::Gem,
            girdle_scale_percent: 104.0,
            girdle_width_percent: 100.0,
            lower_taper_percent: 35.0,
            lower_depth_percent: 115.0,
            lower_seat_size_percent: 94.0,
            lower_seat_depth_percent: 25.0,
            upper_seat_size_percent: 104.0,
            upper_seat_height_percent: 8.0,
            upper_taper_percent: 10.0,
            upper_height_percent: 20.0,
            z_offset_percent: 0.0,
            rotations_deg: [0.0; 5],
            y_scales_percent: [100.0; 5],
            edge: EdgeShape::default(),
            clearance_mm: 0.05,
            bottom: BottomDirection::Down,
            segments: crate::cutters::CUTTER_SEGMENTS,
        }
    }
}

impl GemCutterParams {
    pub fn mode(mode: CutterMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// The same parameters with another builder mode.
    pub fn with_mode(&self, mode: CutterMode) -> Self {
        Self {
            mode,
            ..self.clone()
        }
    }

    /// A parameter set derived from the measured stone: the seat follows the pavilion and the
    /// upper rings follow the crown, so the cutter clears the stone instead of being refused.
    pub fn for_gem(spec: &GemSpec) -> Result<Self> {
        spec.validate()?;
        let reference = GemCutterReference::from_spec(spec)?;
        let profile = profile_points(spec)?;
        let girdle = reference.min_radius_mm();
        let at = |z: f64| reference.radius_at(&profile, z, 1.0);
        let percent = |radius: f64, margin: f64| ((radius / girdle) * 100.0 + margin).max(20.0);
        let mut params = Self::default();
        params.lower_seat_depth_percent = 25.0;
        let seat_z = -(reference.girdle_half_mm + reference.pavilion_depth_mm * 0.25);
        params.lower_seat_size_percent = percent(at(seat_z), 6.0).min(120.0);
        params.girdle_scale_percent = 104.0;
        let upper_seat_z = reference.girdle_half_mm + reference.crown_height_mm * 0.08;
        let top_z = reference.girdle_half_mm + reference.crown_height_mm * 0.20;
        params.upper_seat_size_percent = percent(at(upper_seat_z), 4.0).clamp(100.0, 140.0);
        let top_percent = percent(at(top_z), 4.0).clamp(60.0, 140.0);
        params.upper_taper_percent =
            ((1.0 - top_percent / params.upper_seat_size_percent) * 100.0).clamp(0.0, 80.0);
        params.validate()?;
        Ok(params)
    }

    pub fn validate(&self) -> Result<()> {
        self.edge.validate()?;
        for (value, name) in [
            (self.girdle_scale_percent, "cutter girdle scale"),
            (self.girdle_width_percent, "cutter girdle width"),
            (self.lower_taper_percent, "cutter lower taper"),
            (self.lower_depth_percent, "cutter lower depth"),
            (self.lower_seat_size_percent, "cutter lower seat size"),
            (self.lower_seat_depth_percent, "cutter lower seat depth"),
            (self.upper_seat_size_percent, "cutter upper seat size"),
            (self.upper_seat_height_percent, "cutter upper seat height"),
            (self.upper_taper_percent, "cutter upper taper"),
            (self.upper_height_percent, "cutter upper height"),
            (self.z_offset_percent, "cutter z offset"),
        ] {
            if !value.is_finite() {
                return Err(Error::Invalid(name));
            }
        }
        if !(20.0..=400.0).contains(&self.girdle_scale_percent) {
            return Err(Error::Invalid("cutter girdle scale"));
        }
        if !(5.0..=500.0).contains(&self.girdle_width_percent) {
            return Err(Error::Invalid("cutter girdle width"));
        }
        if !(0.0..=95.0).contains(&self.lower_taper_percent) {
            return Err(Error::Invalid("cutter lower taper"));
        }
        if !(0.0..=400.0).contains(&self.lower_depth_percent) {
            return Err(Error::Invalid("cutter lower depth"));
        }
        if !(10.0..=200.0).contains(&self.lower_seat_size_percent) {
            return Err(Error::Invalid("cutter lower seat size"));
        }
        if !(0.0..=400.0).contains(&self.lower_seat_depth_percent) {
            return Err(Error::Invalid("cutter lower seat depth"));
        }
        if self.lower_seat_depth_percent > self.lower_depth_percent + 1.0e-9 {
            return Err(Error::Invalid("cutter lower seat depth"));
        }
        if !(10.0..=200.0).contains(&self.upper_seat_size_percent) {
            return Err(Error::Invalid("cutter upper seat size"));
        }
        if !(0.0..=400.0).contains(&self.upper_seat_height_percent) {
            return Err(Error::Invalid("cutter upper seat height"));
        }
        if !(0.0..=95.0).contains(&self.upper_taper_percent) {
            return Err(Error::Invalid("cutter upper taper"));
        }
        if !(0.0..=400.0).contains(&self.upper_height_percent) {
            return Err(Error::Invalid("cutter upper height"));
        }
        if !(-200.0..=200.0).contains(&self.z_offset_percent) {
            return Err(Error::Invalid("cutter z offset"));
        }
        for rotation in self.rotations_deg {
            if !rotation.is_finite() || !(-360.0..=360.0).contains(&rotation) {
                return Err(Error::Invalid("cutter rotation"));
            }
        }
        for scale in self.y_scales_percent {
            if !scale.is_finite() || !(10.0..=400.0).contains(&scale) {
                return Err(Error::Invalid("cutter y scale"));
            }
        }
        if !self.clearance_mm.is_finite()
            || !(0.0..=10.0).contains(&self.clearance_mm)
        {
            return Err(Error::Invalid("cutter clearance"));
        }
        if !(3..=crate::settings::MAX_INSTANCES).contains(&self.segments) {
            return Err(Error::Invalid("cutter segments"));
        }
        Ok(())
    }

    /// The five (or four, in Azure mode) control rings in the gem frame, bottom first.
    ///
    /// The ring order is `bottom, lower seat, girdle bottom, girdle top[, upper seat, top]`; the
    /// manual's five rotation/Y-scale controls address `bottom, lower seat, girdle, table, top`,
    /// so the girdle's two rings share control 3.
    pub fn rings(&self, spec: &GemSpec) -> Result<Vec<Ring>> {
        spec.validate()?;
        self.validate()?;
        let reference = GemCutterReference::from_spec(spec)?;
        let girdle = reference.min_radius_mm();
        let pavilion = reference.pavilion_depth_mm;
        let crown = reference.crown_height_mm;
        let scale_x = reference.radius_x / girdle;
        let scale_y = reference.radius_y / girdle;
        let shifted = pavilion * self.z_offset_percent / 100.0;
        let half_girdle = reference.girdle_half_mm * (self.girdle_width_percent / 100.0);
        let ring = |percent: f64, z: f64, control: usize| {
            let radius = girdle * percent / 100.0;
            let y = self.y_scales_percent[control] / 100.0;
            let rotation = if control < self.mode.rotation_controls() {
                self.rotations_deg[control]
            } else {
                0.0
            };
            // The edge profile shapes the azure hole (the bottom ring); the seat, girdle and
            // upper rings stay round so the bearing always clears the stone.
            let shape = if control == 0 && self.mode.uses_edge_profile() {
                self.edge
            } else {
                EdgeShape::default()
            };
            Ring {
                z,
                radius_x: radius * scale_x,
                radius_y: radius * scale_y * y,
                rotation_deg: rotation,
                shape,
            }
        };
        let bottom_z = -(reference.girdle_half_mm + pavilion * self.lower_depth_percent / 100.0) + shifted;
        let seat_z = -(reference.girdle_half_mm + pavilion * self.lower_seat_depth_percent / 100.0) + shifted;
        let girdle_bottom = -half_girdle + shifted;
        let girdle_top = half_girdle + shifted;
        let bottom_percent = self.lower_seat_size_percent * (1.0 - self.lower_taper_percent / 100.0);
        let mut rings = vec![
            ring(bottom_percent, bottom_z, 0),
            ring(self.lower_seat_size_percent, seat_z, 1),
            ring(self.girdle_scale_percent, girdle_bottom, 2),
            ring(self.girdle_scale_percent, girdle_top, 2),
        ];
        if self.mode.builds_upper() {
            let upper_seat_z =
                girdle_top + crown * self.upper_seat_height_percent / 100.0;
            let top_z = upper_seat_z + crown * self.upper_height_percent / 100.0;
            let top_percent = self.upper_seat_size_percent * (1.0 - self.upper_taper_percent / 100.0);
            rings.push(ring(self.upper_seat_size_percent, upper_seat_z, 3));
            rings.push(ring(top_percent, top_z, 4));
        }
        if self.bottom == BottomDirection::Up {
            // Point the bottom of the cutter up: mirror the ring list about the girdle plane and
            // reverse it, which keeps the loft bottom-first and monotonic.
            for ring in &mut rings {
                ring.z = -ring.z;
            }
            rings.reverse();
        }
        // Two coincident rings would make the loft invalid; a zero-thickness girdle band is a
        // legal flat ring, so nudge the second girdle ring up by the smallest printable step.
        if rings[3].z <= rings[2].z + 1.0e-6 {
            rings[3].z = rings[2].z + (crate::cutters::MIN_CUTTER_MM * 0.5).max(1.0e-3);
        }
        Ok(rings)
    }

    /// The edge profile actually used: Gem mode always reads the Round profile.
    pub fn effective_edge(&self) -> EdgeShape {
        if self.mode.uses_edge_profile() {
            self.edge
        } else {
            EdgeShape::default()
        }
    }
}

/// One built cutter body in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct GemCutterBody {
    pub mesh: Mesh,
    pub volume_mm3: f64,
    pub rings: Vec<Ring>,
    /// Smallest air gap between the cutter and the measured stone profile, mm.
    pub clearance_mm: f64,
}

/// The built cutter for one stone.
#[derive(Debug, Clone, PartialEq)]
pub struct GemCutterPlan {
    pub gem_id: String,
    pub mode: CutterMode,
    pub bodies: Vec<GemCutterBody>,
    pub total_volume_mm3: f64,
    pub min_clearance_mm: f64,
    pub summary: String,
}

impl GemCutterPlan {
    pub fn count(&self) -> usize {
        self.bodies.len()
    }

    /// A cutter is a cutter: it never counts as settings mass.
    pub fn is_cutter(&self) -> bool {
        true
    }
}

/// Build one cutter for one stone (the multiple-stone workflow calls this per gem and shares
/// the same parameter set, so a row of stones gets identical cutters).
pub fn build(spec: &GemSpec, placement: &Transform, gem_id: &str, params: &GemCutterParams) -> Result<GemCutterPlan> {
    spec.validate()?;
    placement.validate()?;
    params.validate()?;
    let reference = GemCutterReference::from_spec(spec)?;
    let profile = profile_points(spec)?;
    let rings = params.rings(spec)?;
    let clearance = params.clearance_mm;

    // Pavilion clearance gate: the cutter may never cut into the stone. Each ring's smallest
    // radius (edge profile included) is compared with the measured stone radius at that height.
    let mut min_clearance = f64::INFINITY;
    for ring in &rings {
        let gem = reference.radius_at(&profile, ring.z, 1.0);
        let margin = ring.min_radius() - gem;
        if margin + 1.0e-9 < clearance {
            return Err(Error::Invalid("cutter pavilion clearance"));
        }
        min_clearance = min_clearance.min(margin);
    }
    if rings.iter().any(|ring| ring.radius_x > reference.radius_x * 4.0) {
        return Err(Error::Budget("cutter size"));
    }

    let mesh = loft(&rings, params.segments)?;
    let volume = volume_mm3(&mesh)?;
    if volume <= 1.0e-9 {
        return Err(Error::Invalid("cutter volume"));
    }
    let mesh = crate::cutters::place(&mesh, placement)?;
    let summary = format!(
        "Gem Cutter {} on {} · {:.2} mm girdle · {:.2} mm deep · {:.3} mm³ · clearance {:.2} mm",
        params.mode.label(),
        gem_id,
        reference.min_radius_mm() * 2.0 * params.girdle_scale_percent / 100.0,
        reference.girdle_half_mm * 2.0 + reference.pavilion_depth_mm,
        volume,
        min_clearance
    );
    Ok(GemCutterPlan {
        gem_id: gem_id.to_owned(),
        mode: params.mode,
        bodies: vec![GemCutterBody {
            mesh,
            volume_mm3: volume,
            rings,
            clearance_mm: min_clearance,
        }],
        total_volume_mm3: volume,
        min_clearance_mm: min_clearance,
        summary,
    })
}

/// Build one cutter per stone in `stones` (the manual's "one or multiple stones", and the
/// Gem Cutter's own "Draw All"-style allocation for a gem group).
pub fn build_for_stones(
    stones: &[(String, GemSpec, Transform)],
    params: &GemCutterParams,
) -> Result<Vec<GemCutterPlan>> {
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
    use crate::gem::{GemCut, GemShape, GemSpec};

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

    fn identity() -> Transform {
        Transform::identity()
    }

    #[test]
    fn the_three_modes_build_closed_cutters_with_different_heights() {
        let spec = round_brilliant();
        for mode in CutterMode::ALL {
            let params = GemCutterParams::for_gem(&spec).expect("params").with_mode(mode);
            let plan = build(&spec, &identity(), "gem:1", &params).expect("cutter");
            assert_eq!(plan.mode, mode);
            assert_eq!(plan.bodies.len(), 1);
            let body = &plan.bodies[0];
            assert!(crate::settings::inspect_solid(&body.mesh).expect("closed").closed);
            assert!(body.volume_mm3 > 0.0);
            assert!(body.clearance_mm >= params.clearance_mm - 1.0e-9);
        }
        let gem = GemCutterParams::for_gem(&spec).expect("params");
        let azure = build(&spec, &identity(), "gem:1", &gem.with_mode(CutterMode::Azure)).expect("azure");
        let advanced = build(&spec, &identity(), "gem:1", &gem.with_mode(CutterMode::Advanced)).expect("advanced");
        let azure_top = azure.bodies[0].rings.last().expect("ring").z;
        let advanced_top = advanced.bodies[0].rings.last().expect("ring").z;
        assert!(azure_top < advanced_top, "azure stops at the girdle");
    }

    #[test]
    fn the_edge_profile_only_changes_the_modes_that_publish_it() {
        let spec = round_brilliant();
        let mut params = GemCutterParams::for_gem(&spec).expect("params");
        params.edge = EdgeShape::new(crate::cutters::EdgeKind::Grooves);
        let gem = build(&spec, &identity(), "gem:1", &params).expect("gem mode");
        let azure = build(
            &spec,
            &identity(),
            "gem:1",
            &params.clone().with_mode(CutterMode::Azure),
        )
        .expect("azure mode");
        // Gem mode folds the profile back to Round: the same parameters produce the same body.
        let plain = build(
            &spec,
            &identity(),
            "gem:1",
            &GemCutterParams::for_gem(&spec).expect("params"),
        )
        .expect("plain");
        assert!((gem.total_volume_mm3 - plain.total_volume_mm3).abs() < 1.0e-9);
        assert!((azure.total_volume_mm3 - plain.total_volume_mm3).abs() > 1.0e-6);
    }

    #[test]
    fn a_cutter_that_would_cut_into_the_stone_is_refused() {
        let spec = round_brilliant();
        let mut params = GemCutterParams::for_gem(&spec).expect("params");
        params.girdle_scale_percent = 60.0;
        assert_eq!(
            build(&spec, &identity(), "gem:1", &params).err(),
            Some(Error::Invalid("cutter pavilion clearance"))
        );
    }

    #[test]
    fn the_lower_seat_cannot_sit_below_the_cutter_bottom() {
        let spec = round_brilliant();
        let mut params = GemCutterParams::for_gem(&spec).expect("params");
        params.lower_seat_depth_percent = 150.0;
        params.lower_depth_percent = 100.0;
        assert_eq!(
            params.validate().err(),
            Some(Error::Invalid("cutter lower seat depth"))
        );
    }

    #[test]
    fn multiple_stones_get_one_cutter_each_and_an_empty_group_is_refused() {
        let spec = round_brilliant();
        let params = GemCutterParams::for_gem(&spec).expect("params");
        let stones = vec![
            ("gem:1".to_string(), spec.clone(), identity()),
            ("gem:2".to_string(), spec.clone(), identity()),
        ];
        let plans = build_for_stones(&stones, &params).expect("cutters");
        assert_eq!(plans.len(), 2);
        assert!(plans.iter().all(|plan| plan.count() == 1));
        assert_eq!(
            build_for_stones(&[], &params).err(),
            Some(Error::Missing("cutter gems"))
        );
    }

    #[test]
    fn an_unknown_mode_is_a_typed_refusal() {
        assert_eq!(CutterMode::from_tag("seat? ").err(), Some(Error::Invalid("cutter mode")));
    }
}
