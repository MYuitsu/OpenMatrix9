// SPDX-License-Identifier: LGPL-2.1-or-later
//! Channel cutter: one trench swept through a row of stones (`BuilderChannelBuilder`).
//!
//! The manual's Channel Builder takes an ordered row of gems and cuts the channel they sit in.
//! The cutter is a swept prism: one cross-section at each stone and one at each end, closed by
//! two end caps. The end caps carry the A/B profile and their own depth (the manual's end
//! orientation), and an optional middle section widens or deepens the channel at its centre.
//!
//! The parameters are millimetres. A channel narrower than its widest stone would cut the stone
//! itself, so that is a typed refusal (`channel width`) instead of a silently clipped cutter.

use crate::{
    Error, Result,
    cutters::{CUTTER_SEGMENTS, MAX_CUTTERS, MIN_CUTTER_MM, sweep, volume_mm3},
    gem::GemSpec,
    geometry::{Mesh, Point3, Transform},
};

/// The profile of one end cap of the channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ChannelEnd {
    /// A flat cap.
    #[default]
    Square,
    /// A rounded cap: the width narrows over a quarter circle.
    Round,
    /// A tapered cap: the width halves at the end station.
    Taper,
}

impl ChannelEnd {
    pub const ALL: [Self; 3] = [Self::Square, Self::Round, Self::Taper];

    pub fn label(self) -> &'static str {
        match self {
            Self::Square => "Square",
            Self::Round => "Round",
            Self::Taper => "Taper",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "square" | "flat" => Ok(Self::Square),
            "round" => Ok(Self::Round),
            "taper" | "tapered" => Ok(Self::Taper),
            _ => Err(Error::Invalid("channel end profile")),
        }
    }

    /// The width factor of the end station.
    pub fn factor(self) -> f64 {
        match self {
            Self::Square => 1.0,
            Self::Round => 0.45,
            Self::Taper => 0.5,
        }
    }
}

/// The optional middle section of the channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ChannelMiddle {
    /// No middle section: the channel keeps one width and one depth.
    #[default]
    None,
    /// A section 12% wider than the row, for a channel that opens at its centre.
    Wide,
    /// A section 18% deeper than the row, for a channel that sinks at its centre.
    Deep,
}

impl ChannelMiddle {
    pub const ALL: [Self; 3] = [Self::None, Self::Wide, Self::Deep];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Wide => "Wide",
            Self::Deep => "Deep",
        }
    }

    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "none" | "off" => Ok(Self::None),
            "wide" => Ok(Self::Wide),
            "deep" => Ok(Self::Deep),
            _ => Err(Error::Invalid("channel middle profile")),
        }
    }

    /// `(width factor, depth factor)` of the middle section.
    pub fn scale(self) -> (f64, f64) {
        match self {
            Self::None => (1.0, 1.0),
            Self::Wide => (1.12, 1.0),
            Self::Deep => (1.0, 1.18),
        }
    }
}

/// The typed parameters of the channel cutter.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChannelParams {
    /// The channel width across the row, mm.
    pub width_mm: f64,
    /// The channel depth below the girdle plane at a stone, mm.
    pub depth_mm: f64,
    /// How far the channel walls rise above the girdle plane, mm.
    pub height_mm: f64,
    /// The wall thickness kept outside the widest stone, mm (used to derive the default width).
    pub wall_mm: f64,
    /// Extra room added beyond the last stone at each end, mm.
    pub spacing_mm: f64,
    /// The profile of the first end.
    pub start: ChannelEnd,
    /// The profile of the last end.
    pub end: ChannelEnd,
    /// The depth of the first end cap, mm.
    pub start_depth_mm: f64,
    /// The depth of the last end cap, mm.
    pub end_depth_mm: f64,
    /// Mirror the profile: both ends take the deeper profile, so the channel is symmetric.
    pub mirror: bool,
    /// Flip A/B: the first end takes the B profile and depth and vice versa.
    pub flip: bool,
    /// The optional middle section.
    pub middle: ChannelMiddle,
    /// Circumferential segments of a rounded end.
    pub segments: usize,
}

impl Default for ChannelParams {
    fn default() -> Self {
        Self {
            width_mm: 2.40,
            depth_mm: 1.60,
            height_mm: 0.80,
            wall_mm: 0.25,
            spacing_mm: 0.10,
            start: ChannelEnd::Square,
            end: ChannelEnd::Square,
            start_depth_mm: 1.60,
            end_depth_mm: 1.60,
            mirror: false,
            flip: false,
            middle: ChannelMiddle::None,
            segments: CUTTER_SEGMENTS,
        }
    }
}

impl ChannelParams {
    /// Defaults measured from the row: the channel is as wide as the widest stone plus a wall on
    /// each side, as deep as 55% of the deepest pavilion, and rises above the girdle.
    pub fn for_row(stones: &[GemSpec]) -> Result<Self> {
        let widest = stones.iter().map(|spec| spec.width_mm).fold(0.0, f64::max);
        let deepest = stones
            .iter()
            .map(|spec| spec.pavilion_depth_mm)
            .fold(0.0, f64::max);
        if widest <= 0.0 || deepest <= 0.0 {
            return Err(Error::Invalid("channel gems"));
        }
        let wall = 0.25;
        let mut params = Self {
            width_mm: widest + wall * 2.0,
            depth_mm: (deepest * 0.55).max(MIN_CUTTER_MM),
            height_mm: 0.80,
            wall_mm: wall,
            ..Self::default()
        };
        params.start_depth_mm = params.depth_mm;
        params.end_depth_mm = params.depth_mm;
        params.validate(stones)?;
        Ok(params)
    }

    /// The widest stone in the row, mm; the channel width has to cover it.
    pub fn widest_stone_mm(stones: &[GemSpec]) -> f64 {
        stones.iter().map(|spec| spec.width_mm).fold(0.0, f64::max)
    }

    /// Validate against the row: the channel cannot be narrower than its widest stone, nor
    /// deeper than the stones it carries.
    pub fn validate(&self, stones: &[GemSpec]) -> Result<()> {
        let values = [
            self.width_mm,
            self.depth_mm,
            self.height_mm,
            self.wall_mm,
            self.spacing_mm,
            self.start_depth_mm,
            self.end_depth_mm,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("channel size"));
        }
        if self.width_mm < MIN_CUTTER_MM || self.depth_mm < MIN_CUTTER_MM {
            return Err(Error::Invalid("channel size"));
        }
        if self.height_mm < 0.0 || self.wall_mm < 0.0 || self.spacing_mm < 0.0 {
            return Err(Error::Invalid("channel height"));
        }
        if self.start_depth_mm < MIN_CUTTER_MM || self.end_depth_mm < MIN_CUTTER_MM {
            return Err(Error::Invalid("channel end depth"));
        }
        if !(8..=256).contains(&self.segments) {
            return Err(Error::Invalid("channel segments"));
        }
        let widest = Self::widest_stone_mm(stones);
        if widest > 0.0 && self.width_mm < widest {
            return Err(Error::Invalid("channel width"));
        }
        let deepest = stones
            .iter()
            .map(|spec| spec.pavilion_depth_mm)
            .fold(0.0, f64::max);
        if deepest > 0.0 && self.depth_mm > deepest * 1.6 {
            return Err(Error::Invalid("channel depth"));
        }
        Ok(())
    }

    /// The end profiles and depths after Mirror and Flip are applied.
    fn ends(&self) -> (ChannelEnd, ChannelEnd, f64, f64) {
        let (mut start, mut end) = (self.start, self.end);
        let (mut start_depth, mut end_depth) = (self.start_depth_mm, self.end_depth_mm);
        if self.mirror {
            // Symmetric channel: both ends take the deeper profile and the deeper cap.
            let profile = if self.end.factor() <= self.start.factor() {
                self.end
            } else {
                self.start
            };
            start = profile;
            end = profile;
            let depth = start_depth.max(end_depth);
            start_depth = depth;
            end_depth = depth;
        }
        if self.flip {
            std::mem::swap(&mut start, &mut end);
            std::mem::swap(&mut start_depth, &mut end_depth);
        }
        (start, end, start_depth, end_depth)
    }
}

/// One built channel cutter in world space.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCutter {
    /// The stones the channel was built through, in row order (stable provenance).
    pub gem_ids: Vec<String>,
    pub mesh: Mesh,
    pub width_mm: f64,
    /// The length of the channel from cap to cap, mm.
    pub length_mm: f64,
    pub volume_mm3: f64,
    /// The smallest wall left outside the widest stone on each side, mm.
    pub min_margin_mm: f64,
    pub stations: usize,
    pub summary: String,
}

impl ChannelCutter {
    pub fn is_cutter(&self) -> bool {
        true
    }
}

/// Build the channel cutter through an ordered stone row.
pub fn build_row(
    stones: &[(String, GemSpec, Transform)],
    params: &ChannelParams,
) -> Result<ChannelCutter> {
    if stones.is_empty() {
        return Err(Error::Missing("channel gems"));
    }
    if stones.len() > MAX_CUTTERS {
        return Err(Error::Invalid("channel row"));
    }
    let specs = stones
        .iter()
        .map(|(_, spec, _)| spec.clone())
        .collect::<Vec<_>>();
    params.validate(&specs)?;
    let centres = stones
        .iter()
        .map(|(_, _, placement)| placement.point([0.0, 0.0, 0.0]))
        .collect::<Result<Vec<_>>>()?;
    let count = centres.len();
    for pair in centres.windows(2) {
        let delta = crate::settings::sub(pair[1], pair[0]);
        if crate::settings::norm(delta) < MIN_CUTTER_MM {
            return Err(Error::Invalid("channel row"));
        }
    }
    // The row axis is the first-to-last direction; a single stone uses the X axis so the channel
    // still has a frame.
    let tangent = if count >= 2 {
        crate::settings::unit(crate::settings::sub(centres[count - 1], centres[0]))?
    } else {
        [1.0, 0.0, 0.0]
    };
    let up = [0.0, 0.0, 1.0];
    let mut across = crate::settings::cross(up, tangent);
    if crate::settings::norm(across) < 1.0e-9 {
        across = [1.0, 0.0, 0.0];
    }
    let across = crate::settings::unit(across)?;
    let (start, end, start_depth, end_depth) = params.ends();

    // One station per stone, plus one end cap at each end (and the optional middle section).
    let half = |spec: &GemSpec| (spec.length_mm * 0.5).max(MIN_CUTTER_MM);
    let mut stations: Vec<(Point3, f64, f64, f64)> = Vec::with_capacity(count + 3);
    let first_half = half(&specs[0]);
    let last_half = half(&specs[count - 1]);
    stations.push((
        crate::settings::add(
            centres[0],
            crate::settings::mul(tangent, -(first_half + params.spacing_mm)),
        ),
        params.width_mm * start.factor(),
        girdle_top(&specs[0], params.height_mm),
        -start_depth,
    ));
    for (index, centre) in centres.iter().enumerate() {
        stations.push((
            *centre,
            params.width_mm,
            girdle_top(&specs[index], params.height_mm),
            -params.depth_mm,
        ));
    }
    stations.push((
        crate::settings::add(
            centres[count - 1],
            crate::settings::mul(tangent, last_half + params.spacing_mm),
        ),
        params.width_mm * end.factor(),
        girdle_top(&specs[count - 1], params.height_mm),
        -end_depth,
    ));
    if params.middle != ChannelMiddle::None && count >= 2 {
        let (width_scale, depth_scale) = params.middle.scale();
        let mid = crate::settings::mul(
            crate::settings::add(centres[0], centres[count - 1]),
            0.5,
        );
        let girdle = specs
            .iter()
            .map(|spec| spec.girdle_mm * 0.5)
            .fold(0.0, f64::max);
        stations.push((
            mid,
            params.width_mm * width_scale,
            girdle + params.height_mm,
            -params.depth_mm * depth_scale,
        ));
        // Keep the sweep ordered along the row so the sections never fold back on each other.
        stations.sort_by(|a, b| {
            let left = a.0[0] * tangent[0] + a.0[1] * tangent[1] + a.0[2] * tangent[2];
            let right = b.0[0] * tangent[0] + b.0[1] * tangent[1] + b.0[2] * tangent[2];
            left.total_cmp(&right)
        });
    }

    let mut sections = Vec::with_capacity(stations.len());
    for (centre, width, top, bottom) in &stations {
        let side = crate::settings::mul(across, width * 0.5);
        let z_top = centre[2] + top;
        let z_bottom = centre[2] + bottom;
        let left = [centre[0] + side[0], centre[1] + side[1]];
        let right = [centre[0] - side[0], centre[1] - side[1]];
        sections.push(vec![
            [left[0], left[1], z_top],
            [right[0], right[1], z_top],
            [right[0], right[1], z_bottom],
            [left[0], left[1], z_bottom],
        ]);
    }
    let mesh = sweep(&sections)?;
    let volume_mm3 = volume_mm3(&mesh)?;
    let mut length = 0.0;
    for pair in centres.windows(2) {
        length += crate::settings::norm(crate::settings::sub(pair[1], pair[0]));
    }
    length += first_half + last_half + params.spacing_mm * 2.0;
    let margin = (params.width_mm - ChannelParams::widest_stone_mm(&specs)) * 0.5;
    let gem_ids = stones
        .iter()
        .map(|(id, _, _)| id.clone())
        .collect::<Vec<_>>();
    let summary = format!(
        "Channel Cutter {} × {:.2} mm on {} stone{} · {:.3} mm³",
        start.label(),
        params.width_mm,
        count,
        if count == 1 { "" } else { "s" },
        volume_mm3
    );
    Ok(ChannelCutter {
        gem_ids,
        mesh,
        width_mm: params.width_mm,
        length_mm: length,
        volume_mm3,
        min_margin_mm: margin,
        stations: stations.len(),
        summary,
    })
}

/// The girdle top of one stone in the channel frame, mm above the stone's girdle plane.
fn girdle_top(spec: &GemSpec, height_mm: f64) -> f64 {
    spec.girdle_mm * 0.5 + height_mm
}
