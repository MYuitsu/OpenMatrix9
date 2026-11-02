// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bezel Builder and the Scallop Bezel cutter (`OM9-L2-04-14`, manual pages 249-252 and 255).
//!
//! The Bezel Builder has the manual's three modes — `Bezel`, `Under Bezel` and `Gallery Rail` —
//! and builds a closed ring along the gem's girdle outline from one *edge profile*. The Scallop
//! Bezel (`gvBezelCutter`) builds the cutters used *with* the Bezel Builder; a cutter is committed
//! as its own body but is never counted as settings mass (the same cutter rule as the settings
//! mass owner).
//!
//! All geometry is portable safe Rust. The bezel section lives in the girdle frame: local `x` is
//! outward from the outline and local `y` is the girdle normal, so the same section sweeps the
//! whole girdle for every gem shape.
use crate::{
    Error, Result,
    geometry::Mesh,
    settings::{
        MAX_INSTANCES, Station, inspect_solid,
        head::{GemGirdle, Profile, ProfileKind, profile_section},
        sub, unit,
    },
};

/// The shortest bezel height the manual allows.
pub const MIN_BEZEL_HEIGHT_MM: f64 = 0.1;
/// Minimum top/base thickness of a bezel.
pub const MIN_BEZEL_THICKNESS_MM: f64 = 0.1;
/// Minimum seat depth of the bezel seat.
pub const MIN_BEZEL_SEAT_MM: f64 = 0.02;
/// Upper bound of the scallop cutter count.
pub const MAX_SCALLOP_CUTTERS: u8 = 64;
/// Ring stations of the finger rail the boolean cuts with.
pub const FINGER_RAIL_STATIONS: usize = 96;

/// The finger rail the Boolean option subtracts from the bezel (manual page 251).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FingerRailParams {
    /// Rail profile name from the rail library.
    pub profile: String,
    /// Rail section size, mm.
    pub size_mm: f64,
    /// Radial offset from the bezel outer face, mm.
    pub offset_mm: f64,
    /// How far the rail descends over the bezel face, mm.
    pub height_mm: f64,
}

impl Default for FingerRailParams {
    fn default() -> Self {
        Self {
            profile: "Round".to_string(),
            size_mm: 0.35,
            offset_mm: 0.03,
            height_mm: 0.30,
        }
    }
}

impl FingerRailParams {
    pub fn validate(&self) -> Result<()> {
        profile_section(ProfileKind::Rail, &self.profile, None)?;
        if !self.size_mm.is_finite() || self.size_mm < MIN_BEZEL_THICKNESS_MM {
            return Err(Error::Invalid("finger rail size"));
        }
        if !self.offset_mm.is_finite() || self.offset_mm < 0.0 {
            return Err(Error::Invalid("finger rail offset"));
        }
        if !self.height_mm.is_finite() || self.height_mm < MIN_BEZEL_THICKNESS_MM {
            return Err(Error::Invalid("finger rail height"));
        }
        Ok(())
    }
}

/// The three Bezel Builder modes of the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BezelMode {
    Bezel,
    UnderBezel,
    GalleryRail,
}

impl BezelMode {
    pub const ALL: [Self; 3] = [Self::Bezel, Self::UnderBezel, Self::GalleryRail];

    pub fn label(self) -> &'static str {
        match self {
            Self::Bezel => "Bezel",
            Self::UnderBezel => "Under Bezel",
            Self::GalleryRail => "Gallery Rail",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "bezel" | "full" => Ok(Self::Bezel),
            "underbezel" | "under bezel" | "under" => Ok(Self::UnderBezel),
            "galleryrail" | "gallery rail" | "gallery" => Ok(Self::GalleryRail),
            _ => Err(Error::Invalid("bezel mode")),
        }
    }
}

/// The typed Bezel Builder parameters.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BezelParams {
    pub mode: BezelMode,
    pub edge_profile: String,
    /// Bezel height from the base to the girdle, mm.
    pub height_mm: f64,
    /// Thickness at the girdle, mm.
    pub top_mm: f64,
    /// Thickness at the base, mm.
    pub base_mm: f64,
    /// Rounded outer face bulge, mm; 0 keeps the straight edge profile.
    pub dome_mm: f64,
    /// Angle of the outer face from vertical, degrees.
    pub angle_deg: f64,
    /// Seat angle at the girdle, degrees.
    pub seat_angle_deg: f64,
    /// Depth of the seat under the girdle, mm.
    pub seat_depth_mm: f64,
    /// Radial length of the seat, mm.
    pub seat_length_mm: f64,
    /// Vertical placement of the whole bezel, mm (positive = up).
    pub placement_mm: f64,
    /// Overall scale, `0.05..=20.0`.
    pub scale: f64,
    pub chamfer_x_mm: f64,
    pub chamfer_z_mm: f64,
    /// Shear the rail/edge profile into a parallelogram (manual default ON).
    pub shear_rails: bool,
    /// Boolean away the part inside the finger rail (manual default ON).
    pub boolean: bool,
    /// The finger rail the boolean subtracts.
    #[serde(default)]
    pub finger_rail: FingerRailParams,
    /// An explicit edge section (unit box), e.g. a Bezier profile; overrides the mode section.
    #[serde(default)]
    pub section_points: Option<Vec<(f64, f64)>>,
}

impl Default for BezelParams {
    fn default() -> Self {
        Self {
            mode: BezelMode::Bezel,
            edge_profile: "Chamfer".to_string(),
            height_mm: 1.1,
            top_mm: 0.6,
            base_mm: 0.75,
            dome_mm: 0.0,
            angle_deg: 4.0,
            seat_angle_deg: 15.0,
            seat_depth_mm: 0.15,
            seat_length_mm: 0.12,
            placement_mm: 0.0,
            scale: 1.0,
            chamfer_x_mm: 0.12,
            chamfer_z_mm: 0.12,
            shear_rails: true,
            boolean: true,
            finger_rail: FingerRailParams::default(),
            section_points: None,
        }
    }
}

impl BezelParams {
    pub fn mode(mode: BezelMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        profile_section(ProfileKind::Edge, &self.edge_profile, self.section_points.as_deref())?;
        if !self.height_mm.is_finite() || self.height_mm < MIN_BEZEL_HEIGHT_MM {
            return Err(Error::Invalid("bezel height"));
        }
        if !self.top_mm.is_finite() || self.top_mm < MIN_BEZEL_THICKNESS_MM {
            return Err(Error::Invalid("bezel top thickness"));
        }
        if !self.base_mm.is_finite() || self.base_mm < MIN_BEZEL_THICKNESS_MM {
            return Err(Error::Invalid("bezel base thickness"));
        }
        if !self.dome_mm.is_finite() || self.dome_mm < 0.0 {
            return Err(Error::Invalid("bezel dome height"));
        }
        if !self.angle_deg.is_finite() || !(-45.0..=60.0).contains(&self.angle_deg) {
            return Err(Error::Invalid("bezel angle"));
        }
        if !self.seat_angle_deg.is_finite() || !(0.0..=89.0).contains(&self.seat_angle_deg) {
            return Err(Error::Invalid("bezel seat angle"));
        }
        if !self.seat_depth_mm.is_finite() || self.seat_depth_mm < 0.0 {
            return Err(Error::Invalid("bezel seat depth"));
        }
        if !self.seat_length_mm.is_finite()
            || self.seat_length_mm < 0.0
            || self.seat_length_mm > self.base_mm * 0.5
        {
            return Err(Error::Invalid("bezel seat length"));
        }
        if !self.placement_mm.is_finite() {
            return Err(Error::Invalid("bezel placement"));
        }
        if !self.scale.is_finite() || !(0.05..=20.0).contains(&self.scale) {
            return Err(Error::Invalid("bezel scale"));
        }
        if !self.chamfer_x_mm.is_finite() || !self.chamfer_z_mm.is_finite()
            || self.chamfer_x_mm < 0.0 || self.chamfer_z_mm < 0.0
        {
            return Err(Error::Invalid("bezel chamfer"));
        }
        if self.chamfer_x_mm > self.top_mm || self.chamfer_z_mm > self.height_mm {
            return Err(Error::Invalid("bezel chamfer"));
        }
        self.finger_rail.validate()?;
        if let Some(points) = &self.section_points {
            Profile::from_section("Bezier", points)?;
        }
        Ok(())
    }

    /// The effective height including scale, mm.
    pub fn effective_height_mm(&self) -> f64 {
        self.height_mm * self.scale
    }

    pub fn effective_top_mm(&self) -> f64 {
        self.top_mm * self.scale
    }

    pub fn effective_base_mm(&self) -> f64 {
        self.base_mm * self.scale
    }
}

/// A built bezel ring.
#[derive(Debug, Clone, PartialEq)]
pub struct BezelSetting {
    pub gem_id: String,
    pub mode: BezelMode,
    pub mesh: Mesh,
    pub volume_mm3: f64,
    /// The profile used, after scaling.
    pub section: Vec<(f64, f64)>,
    pub sheared: bool,
    /// True when the finger rail was subtracted from the ring.
    pub boolean_cut: bool,
    /// Volume the boolean removed, mm³.
    pub removed_mm3: f64,
    /// The finger rail body the boolean consumed, when the cut ran.
    pub finger_rail: Option<Mesh>,
    pub summary: String,
}

/// Build the bezel ring along the girdle outline. With `boolean` on (the manual default) the
/// finger rail is built first and subtracted from the ring, so the groove under the rail is real
/// geometry rather than a parameter the builder ignores.
pub fn build(girdle: &GemGirdle, params: &BezelParams) -> Result<BezelSetting> {
    girdle.validate()?;
    params.validate()?;
    let section = section(girdle, params)?;
    let stations = girdle.ring_stations(BEZEL_STATIONS, 0.0)?;
    let mesh = crate::settings::sweep_section(&stations, &section, true)?;
    let report = inspect_solid(&mesh)?;
    let mut boolean_cut = false;
    let mut removed_mm3 = 0.0;
    let mut rail_body = None;
    if params.boolean {
        // The finger rail is a ring on the same girdle, so the boolean is an exact planar
        // subtraction in the section frame: the groove is the rail's section swept along the
        // outline. The plain ring is measured once to report the removed volume.
        let plain_section = section_plain(girdle, params)?;
        let plain = crate::settings::sweep_section(&stations, &plain_section, true)?;
        let plain_report = inspect_solid(&plain)?;
        removed_mm3 = (plain_report.volume_mm3 - report.volume_mm3).max(0.0);
        if removed_mm3 <= 1.0e-6 {
            return Err(Error::Invalid("bezel boolean result"));
        }
        boolean_cut = true;
        rail_body = Some(finger_rail(girdle, params)?);
    }
    let mut summary = format!(
        "Bezel {} on {}: {:.2} mm high, {:?} section",
        params.mode.label(),
        girdle.gem_id,
        params.effective_height_mm(),
        params.edge_profile
    );
    if boolean_cut {
        summary.push_str(&format!(", finger rail cut {removed_mm3:.3} mm³"));
    }
    Ok(BezelSetting {
        gem_id: girdle.gem_id.clone(),
        mode: params.mode,
        mesh,
        volume_mm3: report.volume_mm3,
        section,
        sheared: params.shear_rails && params.angle_deg.abs() > 1.0e-9,
        boolean_cut,
        removed_mm3,
        finger_rail: rail_body,
        summary,
    })
}

/// The closed finger rail ring the boolean subtracts: a rail profile riding the top outer edge.
pub fn finger_rail(girdle: &GemGirdle, params: &BezelParams) -> Result<Mesh> {
    params.validate()?;
    let rail = &params.finger_rail;
    let profile = profile_section(ProfileKind::Rail, &rail.profile, None)?;
    let rail_section = profile.scaled(rail.size_mm, rail.height_mm)?;
    let base = section_plain(girdle, params)?;
    let top = base.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    if !top.is_finite() {
        return Err(Error::Invalid("finger rail placement"));
    }
    let edge = base
        .iter()
        .filter(|point| (point.1 - top).abs() < 1.0e-9)
        .map(|point| point.0)
        .fold(0.0_f64, f64::max);
    if edge <= 1.0e-9 {
        return Err(Error::Invalid("finger rail placement"));
    }
    // Sit the rail exactly in the groove the boolean cuts: radial `edge..edge-size`, axial
    // `top-height..top`.
    let radial = edge - rail.size_mm + rail.offset_mm;
    let axial = top - rail.height_mm + rail.offset_mm;
    let section: Vec<(f64, f64)> = rail_section
        .iter()
        .map(|(r, z)| (r + radial, z + axial))
        .collect();
    let stations = girdle.ring_stations(FINGER_RAIL_STATIONS, 0.0)?;
    let mesh = crate::settings::sweep_section(&stations, &section, true)?;
    inspect_solid(&mesh).map_err(|_| Error::Invalid("finger rail solid"))?;
    Ok(mesh)
}


/// The closed section `(radial, axial)` of the bezel, in mm, before sweeping. With `boolean` on
/// the finger rail groove is cut out of the section, so the swept ring is the notched bezel.
pub fn section(girdle: &GemGirdle, params: &BezelParams) -> Result<Vec<(f64, f64)>> {
    let points = section_plain(girdle, params)?;
    if params.boolean {
        notch_for_finger_rail(&points, params)
    } else {
        Ok(points)
    }
}

/// Cut the finger rail groove out of the section: the rail is a ring, so the boolean is a planar
/// subtraction between the bezel section and the rail's section at the top outer corner.
fn notch_for_finger_rail(points: &[(f64, f64)], params: &BezelParams) -> Result<Vec<(f64, f64)>> {
    let rail = &params.finger_rail;
    let top = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    if !top.is_finite() {
        return Err(Error::Invalid("finger rail placement"));
    }
    let edge = points
        .iter()
        .filter(|point| (point.1 - top).abs() < 1.0e-9)
        .map(|point| point.0)
        .fold(f64::NEG_INFINITY, f64::max);
    if !edge.is_finite() || edge <= 1.0e-9 {
        return Err(Error::Invalid("finger rail placement"));
    }
    let mut points: Vec<(f64, f64)> = points.to_vec();
    // The removed block lives on the corner. Walk the outer face after it in the traversal
    // direction; if the polygon goes the other way, reverse it (the sweep normalises winding).
    let corner = |points: &[(f64, f64)]| {
        points
            .iter()
            .position(|point| (point.0 - edge).abs() < 1.0e-9 && (point.1 - top).abs() < 1.0e-9)
    };
    let mut index = corner(&points).ok_or(Error::Invalid("bezel boolean corner"))?;
    let n = points.len();
    if (points[(index + 1) % n].1 - top).abs() < 1.0e-9 {
        points.reverse();
        index = corner(&points).ok_or(Error::Invalid("bezel boolean corner"))?;
    }
    let size = rail.size_mm.min(edge - 1.0e-6);
    let r0 = edge - size;
    if r0 <= 1.0e-6 {
        return Err(Error::Invalid("finger rail placement"));
    }
    let z0 = top - rail.height_mm;
    if z0 >= top {
        return Err(Error::Invalid("finger rail height"));
    }
    // The top-face vertex at or inside `r0` that stays in the profile.
    let mut top_side = index;
    for _ in 0..n {
        let prev = (top_side + n - 1) % n;
        if (points[prev].1 - top).abs() < 1.0e-9 && points[prev].0 > r0 + 1.0e-9 {
            top_side = prev;
        } else {
            break;
        }
    }
    // Walk down the outer face to the segment that crosses `z0`.
    let mut low = index;
    for _ in 0..n {
        let next = (low + 1) % n;
        if points[next].1 > z0 + 1.0e-9 {
            low = next;
        } else {
            break;
        }
    }
    let next = (low + 1) % n;
    let (ar, az) = points[low];
    let (br, bz) = points[next];
    let span = az - bz;
    if span <= 1.0e-9 {
        return Err(Error::Invalid("bezel boolean corner"));
    }
    let t = ((az - z0) / span).clamp(0.0, 1.0);
    let outer = ar + (br - ar) * t;
    if outer <= r0 {
        return Err(Error::Invalid("finger rail height"));
    }
    // Rebuild: keep the profile down to the groove, add the groove, then the rest of the ring.
    let mut out = Vec::with_capacity(n + 3);
    out.push(points[top_side]);
    out.push((r0, top));
    out.push((r0, z0));
    out.push((outer, z0));
    let mut cursor = next;
    while cursor != top_side {
        out.push(points[cursor]);
        cursor = (cursor + 1) % n;
    }
    Ok(out)
}

/// The bezel section without the finger rail groove.
pub fn section_plain(girdle: &GemGirdle, params: &BezelParams) -> Result<Vec<(f64, f64)>> {
    params.validate()?;
    let height = params.effective_height_mm();
    let top = params.effective_top_mm();
    let base = params.effective_base_mm();
    let seat = params.seat_length_mm.clamp(0.0, base * 0.5);
    let place = params.placement_mm;
    let mut points: Vec<(f64, f64)> = match &params.section_points {
        // A profile section is half-width (`0..=0.5` radial, `0..=1` axial) like the library,
        // so it scales to the full top thickness.
        Some(points) if !points.is_empty() => points
            .iter()
            .map(|(r, z)| (r * 2.0 * top, place - height + z * height))
            .collect(),
        _ => match params.mode {
        BezelMode::Bezel => {
            let mut points = vec![(0.0, place), (top - params.chamfer_x_mm * params.scale, place)];
            if params.chamfer_x_mm > 1.0e-9 || params.chamfer_z_mm > 1.0e-9 {
                points.push((
                    top,
                    place - params.chamfer_z_mm * params.scale,
                ));
            } else {
                points.push((top, place));
            }
            let slope = params.angle_deg.to_radians().tan();
            if params.dome_mm > 1.0e-9 {
                points.push((top + params.dome_mm * params.scale, place - height * 0.5));
            }
            points.push((base + slope * height, place - height));
            points.push((seat, place - height));
            points.push((0.0, place - height + params.seat_depth_mm));
            points
        }
        BezelMode::UnderBezel => {
            let shift = -girdle.girdle_bottom_mm;
            vec![
                (0.0, shift + place),
                (top, shift + place),
                (base, shift - height + place),
                (seat, shift - height + place),
                (0.0, shift - height + params.seat_depth_mm + place),
            ]
        }
        BezelMode::GalleryRail => {
            // A rail band below the girdle: radial ring from the outline to `top`.
            let rail = base.max(MIN_BEZEL_THICKNESS_MM);
            let shift = -girdle.girdle_bottom_mm - height + place;
            vec![
                (0.0, shift),
                (top, shift),
                (top, shift + rail),
                (0.0, shift + rail),
            ]
        }
    },
    };
    if params.shear_rails && params.angle_deg.abs() > 1.0e-9 {
        let shear = params.angle_deg.to_radians().tan();
        let top_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        for point in &mut points {
            point.0 += shear * (top_y - point.1);
        }
    }
    if points.len() < 3 {
        return Err(Error::Invalid("bezel section"));
    }
    if points.iter().any(|(r, z)| !r.is_finite() || !z.is_finite() || *r < -1.0e-9) {
        return Err(Error::Invalid("bezel section"));
    }
    Ok(points)
}

/// The Scallop Bezel cutter parameters (manual page 255, `gvBezelCutter`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScallopParams {
    /// Edge profile of the cutter, from the same library as the bezel.
    pub profile: String,
    /// Number of cutters around the gem.
    pub count: u8,
    /// Pattern angle of the first cutter, degrees.
    pub angle_deg: f64,
    /// Offset of the cutter from the girdle outline, mm.
    pub offset_mm: f64,
    /// Radial depth of one cutter, mm.
    pub depth_mm: f64,
    /// An explicit cutter section (unit box), e.g. a Bezier profile.
    #[serde(default)]
    pub section_points: Option<Vec<(f64, f64)>>,
    /// The dimensional Bezel Cutter Builder (`OM9-L3-04-16-004`): when set, the cutters are the
    /// builder's tapered blocks instead of the swept edge profile. The manual documents the same
    /// `gvBezelCutter` command in the Cutters chapter, so the shared scallop owner carries both
    /// parameter contracts rather than a second command kernel.
    #[serde(default)]
    pub dims: Option<crate::cutters::bezel::BezelCutterDims>,
}

impl Default for ScallopParams {
    fn default() -> Self {
        Self {
            profile: "Round".to_string(),
            count: 12,
            angle_deg: 0.0,
            offset_mm: 0.05,
            depth_mm: 0.5,
            section_points: None,
            dims: None,
        }
    }
}

impl ScallopParams {
    pub fn validate(&self) -> Result<()> {
        profile_section(ProfileKind::Edge, &self.profile, self.section_points.as_deref())?;
        if !(3..=MAX_SCALLOP_CUTTERS).contains(&self.count) {
            return Err(Error::Invalid("scallop count"));
        }
        if !self.angle_deg.is_finite() {
            return Err(Error::Invalid("scallop angle"));
        }
        if !self.offset_mm.is_finite() || self.offset_mm < 0.0 {
            return Err(Error::Invalid("scallop offset"));
        }
        if !self.depth_mm.is_finite() || self.depth_mm < MIN_BEZEL_THICKNESS_MM {
            return Err(Error::Invalid("scallop depth"));
        }
        if let Some(dims) = &self.dims {
            dims.validate()?;
        }
        Ok(())
    }
}

/// The built scallop cutters: one closed body per cutter, never part of the settings mass.
#[derive(Debug, Clone, PartialEq)]
pub struct ScallopSetting {
    pub gem_id: String,
    pub profile: String,
    pub offset_mm: f64,
    pub cutters: Vec<Mesh>,
    pub volumes_mm3: Vec<f64>,
    pub total_volume_mm3: f64,
}

impl ScallopSetting {
    pub fn count(&self) -> usize {
        self.cutters.len()
    }

    /// Scallop cutters are cutters: the mass owner excludes them.
    pub fn is_cutter(&self) -> bool {
        true
    }

    pub fn summary(&self) -> String {
        format!(
            "Scallop bezel cutters: {} x {:?} at {:.1} mm offset",
            self.cutters.len(),
            self.profile,
            self.offset_mm
        )
    }

}

/// Build the scallop cutters around the girdle outline.
pub fn build_scallop(girdle: &GemGirdle, params: &ScallopParams) -> Result<ScallopSetting> {
    girdle.validate()?;
    params.validate()?;
    if let Some(dims) = &params.dims {
        // Dimensional Bezel Cutter Builder (`OM9-L3-04-16-004`): the same command, the block
        // parameter contract instead of the swept edge profile.
        let set = crate::cutters::bezel::build(girdle, dims)?;
        return Ok(ScallopSetting {
            gem_id: girdle.gem_id.clone(),
            profile: format!("Bezel Cutter {}", dims.position.label()),
            offset_mm: dims.y_offset_mm,
            cutters: set.cutters,
            volumes_mm3: set.volumes_mm3,
            total_volume_mm3: set.total_volume_mm3,
        });
    }
    let profile = profile_section(
        ProfileKind::Edge,
        &params.profile,
        params.section_points.as_deref(),
    )?;
    let count = params.count as usize;
    let mut cutters = Vec::with_capacity(count);
    let mut volumes = Vec::with_capacity(count);
    for index in 0..count {
        let fraction = index as f64 / count as f64 + params.angle_deg / 360.0;
        let point = girdle.point_at_fraction(fraction)?;
        let outward = girdle.outward_at(point)?;
        let base = crate::settings::add(point, crate::settings::mul(outward, params.offset_mm));
        let tangent = unit(crate::settings::cross(girdle.normal, outward))?;
        // The cutter is the edge profile extruded along the outline tangent: the section lives
        // in the (radial, axial) plane, so the sweep direction is perpendicular to it.
        let width = (girdle.outline_length_mm() / count as f64) * 0.6;
        let start = crate::settings::sub(base, crate::settings::mul(tangent, width * 0.5));
        let end = crate::settings::add(start, crate::settings::mul(tangent, width));
        let stations = [
            Station::new(start, outward, girdle.normal)?,
            Station::new(end, outward, girdle.normal)?,
        ];
        let section = profile.scaled(params.depth_mm, params.depth_mm)?;
        let mesh = crate::settings::sweep_section(&stations, &section, false)?;
        let report = inspect_solid(&mesh)?;
        cutters.push(mesh);
        volumes.push(report.volume_mm3);
    }
    let total = volumes.iter().sum();
    Ok(ScallopSetting {
        gem_id: girdle.gem_id.clone(),
        profile: params.profile.clone(),
        offset_mm: params.offset_mm,
        cutters,
        volumes_mm3: volumes,
        total_volume_mm3: total,
    })
}

/// The `Pull To Rail`-style clearance check shared by the bezel modes: no metal may sit inside
/// the gem surface below the girdle.
pub fn clearance(girdle: &GemGirdle, depth_mm: f64) -> f64 {
    let at_depth = girdle.surface_radius_at(-depth_mm);
    let outline = girdle.radius_mm();
    (outline - at_depth).max(0.0)
}

/// Distance from the gem axis, kept for callers that report the bezel placement.
pub fn axis_distance(girdle: &GemGirdle, point: [f64; 3]) -> f64 {
    let offset = sub(point, girdle.center);
    let axial = crate::settings::dot(offset, girdle.normal);
    crate::settings::norm(sub(offset, crate::settings::mul(girdle.normal, axial)))
}

/// The number of stations used per bezel ring; fixed so a rebuild is deterministic.
pub const BEZEL_STATIONS: usize = 128;

/// The highest number of cutters a scallop pattern may ask for.
pub fn scallop_budget(count: usize) -> Result<usize> {
    if count == 0 || count > MAX_INSTANCES {
        return Err(Error::Budget("scallop cutters"));
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_gem() -> GemGirdle {
        GemGirdle::round("gem:b1", [0.0, 0.0, 0.0], 6.0, [0.0, 0.0, 1.0]).expect("gem")
    }

    #[test]
    fn the_three_modes_build_a_closed_ring() {
        let girdle = round_gem();
        for mode in BezelMode::ALL {
            let params = BezelParams::mode(mode);
            let bezel = build(&girdle, &params).expect("bezel");
            assert_eq!(bezel.mode, mode);
            assert!(bezel.volume_mm3 > 0.0, "{mode:?} volume");
            assert!(inspect_solid(&bezel.mesh).expect("closed").closed);
            assert!(bezel.summary.contains(mode.label()));
            assert!(bezel.boolean_cut, "{mode:?} boolean default");
        }
    }

    #[test]
    fn the_edge_profile_is_read_from_the_library() {
        let girdle = round_gem();
        let mut params = BezelParams::default();
        params.edge_profile = "Nonexistent".to_string();
        let error = build(&girdle, &params).expect_err("profile");
        assert!(format!("{error:?}").contains("edge profile"));
        for name in ["Chamfer", "Round", "Step", "Knife"] {
            params.edge_profile = name.to_string();
            assert!(build(&girdle, &params).is_ok(), "{name}");
        }
    }

    #[test]
    fn the_boolean_cuts_the_finger_rail_groove_out_of_the_ring() {
        let girdle = round_gem();
        let mut params = BezelParams::default();
        params.boolean = false;
        let plain = build(&girdle, &params).expect("plain bezel");
        params.boolean = true;
        let cut = build(&girdle, &params).expect("cut bezel");
        assert!(cut.boolean_cut);
        assert!(cut.finger_rail.is_some());
        assert!(cut.removed_mm3 > 0.0);
        assert!(cut.volume_mm3 < plain.volume_mm3);
        assert!((plain.volume_mm3 - cut.volume_mm3 - cut.removed_mm3).abs() < 1.0e-6);
        assert!(inspect_solid(&cut.mesh).expect("closed").closed);
        assert!(cut.summary.contains("finger rail cut"));
        let rail = finger_rail(&girdle, &params).expect("finger rail");
        assert!(inspect_solid(&rail).expect("closed rail").closed);
    }

    #[test]
    fn the_finger_rail_is_validated_by_name() {
        let girdle = round_gem();
        let mut params = BezelParams::default();
        params.finger_rail.profile = "NotARail".to_string();
        let error = build(&girdle, &params).expect_err("finger rail profile");
        assert!(format!("{error:?}").contains("rail profile"));
        params.finger_rail.profile = "Square".to_string();
        params.finger_rail.size_mm = 0.01;
        assert!(format!("{:?}", build(&girdle, &params).unwrap_err()).contains("finger rail size"));
    }

    #[test]
    fn a_bezier_section_replaces_the_mode_section() {
        let girdle = round_gem();
        let mut params = BezelParams::default();
        params.boolean = false;
        let plain = build(&girdle, &params).expect("plain");
        let section = Profile::from_bezier(
            "Bezier",
            &[(0.0, 0.0), (1.0, 0.15), (1.0, 0.85), (0.55, 1.0), (0.0, 1.0)],
            40,
        )
        .expect("bezier");
        params.section_points = Some(section.points.clone());
        let bezier = build(&girdle, &params).expect("bezier bezel");
        assert_eq!(bezier.section.len(), section.points.len());
        assert!((bezier.volume_mm3 - plain.volume_mm3).abs() > 1.0e-6);
        // An explicit section is validated like a profile.
        params.section_points = Some(vec![(0.0, 0.0), (0.0, 0.0)]);
        assert!(build(&girdle, &params).is_err());
    }

    #[test]
    fn the_scallop_takes_an_explicit_section() {
        let girdle = round_gem();
        let mut params = ScallopParams::default();
        let plain = build_scallop(&girdle, &params).expect("plain scallop");
        let section = Profile::from_bezier("Bezier", &[(0.0, 0.0), (1.0, 0.5), (0.0, 1.0)], 24)
            .expect("bezier");
        params.section_points = Some(section.points.clone());
        let custom = build_scallop(&girdle, &params).expect("custom scallop");
        assert_eq!(custom.count(), plain.count());
        assert!((custom.total_volume_mm3 - plain.total_volume_mm3).abs() > 1.0e-9);
    }

    #[test]
    fn shear_turns_the_section_into_a_parallelogram() {
        let girdle = round_gem();
        let mut params = BezelParams::default();
        params.angle_deg = 15.0;
        params.shear_rails = true;
        let sheared = section(&girdle, &params).expect("section");
        params.shear_rails = false;
        let upright = section(&girdle, &params).expect("section");
        let sheared_top = sheared.iter().map(|p| p.0).fold(0.0_f64, f64::max);
        let upright_top = upright.iter().map(|p| p.0).fold(0.0_f64, f64::max);
        assert!((sheared_top - upright_top).abs() > 1.0e-6);
        let bezel = build(&girdle, &params).expect("bezel");
        assert!(!bezel.sheared);
    }

    #[test]
    fn the_under_bezel_and_gallery_rail_sit_below_the_girdle() {
        let girdle = round_gem();
        for mode in [BezelMode::UnderBezel, BezelMode::GalleryRail] {
            let params = BezelParams::mode(mode);
            let bezel = build(&girdle, &params).expect("bezel");
            let lowest = bezel.mesh.vertices.iter().map(|v| v[2]).fold(f64::INFINITY, f64::min);
            assert!(lowest < -girdle.girdle_bottom_mm, "{mode:?} sits below the girdle");
        }
    }

    #[test]
    fn scallop_builds_one_closed_cutter_per_position() {
        let girdle = round_gem();
        let params = ScallopParams::default();
        let scallop = build_scallop(&girdle, &params).expect("scallop");
        assert_eq!(scallop.count(), 12);
        assert!(scallop.is_cutter());
        assert!(scallop.total_volume_mm3 > 0.0);
        assert!(scallop.cutters.iter().all(|mesh| inspect_solid(mesh).is_ok()));
        assert!(scallop.summary().contains("Scallop bezel cutters: 12"));
        let mut bad = ScallopParams::default();
        bad.count = 200;
        assert!(build_scallop(&girdle, &bad).is_err());
        bad.count = 12;
        bad.profile = "NotAProfile".to_string();
        assert!(format!("{:?}", build_scallop(&girdle, &bad).unwrap_err()).contains("edge profile"));
    }

    #[test]
    fn the_clearance_report_measures_the_pavilion_cone() {
        let girdle = round_gem();
        let deep = clearance(&girdle, 1.0);
        let shallow = clearance(&girdle, 0.1);
        assert!(deep > shallow);
        assert!(deep > 0.5);
    }
}
