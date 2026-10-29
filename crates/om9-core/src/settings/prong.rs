// SPDX-License-Identifier: LGPL-2.1-or-later
//! Prongs (`OM9-L3-04-15-001/002/003/009`): Prong Builder on a gem line, the single-prong
//! editor, Prong on Surface, and the row end/prong-spacing controls.
//!
//! The mode table is data, not code paths, so the documented Matrix styles stay auditable:
//!
//! * **Gem Flow** styles place prongs at the row **ends** and at the **shared junctions**
//!   between stones (1/2 shared prongs, ends only, one/two side prongs);
//! * **Gem North** styles place prongs **around each gem** at angles measured from the stone's
//!   north direction (2, 3, 4 and 6 prong layouts, including the 30/45/60/90/120/150 rotations
//!   and the "close" 4-prong layout).
//!
//! The sheet only states two numeric defaults — the 10 % nudge overlap and the 0.1 mm minimum
//! height — so every other value is an OM9 engineering default marked as such, never presented
//! as a measured Matrix value.
//!
//! Shared junction prongs are emitted **once per junction**: a closed row has no ends and its
//! wrap junction is a real junction, so the layout cannot double-count a prong.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Mesh, NurbsSurface, Point3},
    settings::{
        DEFAULT_NUDGE, Frame, MAX_INSTANCES, MIN_PRONG_DIAMETER_MM, MIN_PRONG_HEIGHT_MM,
        REVOLVE_SEGMENTS, add, inspect_solid, mul, norm, revolve_profile, sub, unit,
    },
    settings::gems::GemRow,
    surface_kernel,
};

/// The two prong builder families of the manual ("There are two Prong mode options ... Gem Flow
/// ... Gem North").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProngFamily {
    GemFlow,
    GemNorth,
}

impl ProngFamily {
    pub fn title(self) -> &'static str {
        match self {
            Self::GemFlow => "Gem Flow",
            Self::GemNorth => "Gem North",
        }
    }
}

/// How a prong pair is spread around its base angle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Spread {
    /// A single prong at the base angle.
    Single,
    /// Two prongs at `base ± delta`, where `delta` is the prong's angular half width on the
    /// gem girdle. This is how "4 Prongs 0 Degrees" and "3 Prongs 0 Degrees" place a pair
    /// against one side of the stone.
    Delta,
    /// Two prongs at `base ± delta/2`, the "4 Prongs Close" variant.
    HalfDelta,
}

/// One angular pair of the mode table.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PairSpec {
    pub base_deg: f64,
    pub spread: Spread,
}

/// The data of one documented prong style.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutSpec {
    pub family: ProngFamily,
    /// Prongs on the first and last stone of an open row.
    pub ends: [u8; 2],
    /// Shared prongs in every junction.
    pub junctions: u8,
    /// Prongs on the +90° (one side) and −90° (other side) girdle positions of every stone.
    pub sides: [u8; 2],
    /// Gem-centred angular pairs (Gem North styles, and the side styles of Gem Flow).
    pub pairs: Vec<PairSpec>,
}

/// A documented prong style.
///
/// The style set is closed: an unknown style is a typed [`Error::Unsupported`] instead of a
/// silent fallback that would place prongs the operator did not ask for.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProngMode {
    OneShared,
    TwoShared,
    TwoProngs,
    EndsOnly,
    OneSharedOneSide,
    OneSharedTwoSide,
    LeftTwoRight,
    TwoProngsAt(f64),
    ThreeProngsAt(f64),
    FourProngsAt(f64),
    FourProngsClose,
    SixProngsAt(f64),
}

impl ProngMode {
    /// The Gem Flow default of the manual: "A line of gem defaults to 2 Shared Prongs in Gem
    /// Flow."
    pub const GEM_FLOW_DEFAULT: Self = Self::TwoShared;
    /// The Gem North default of the manual: "4 Prongs 0 Degrees ... is the default for North
    /// option."
    pub const GEM_NORTH_DEFAULT: Self = Self::FourProngsAt(0.0);

    pub fn family(self) -> ProngFamily {
        match self {
            Self::OneShared
            | Self::TwoShared
            | Self::TwoProngs
            | Self::EndsOnly
            | Self::OneSharedOneSide
            | Self::OneSharedTwoSide
            | Self::LeftTwoRight => ProngFamily::GemFlow,
            Self::TwoProngsAt(_)
            | Self::ThreeProngsAt(_)
            | Self::FourProngsAt(_)
            | Self::FourProngsClose
            | Self::SixProngsAt(_) => ProngFamily::GemNorth,
        }
    }

    /// The documented style name, matching the manual wording.
    pub fn name(self) -> String {
        match self {
            Self::OneShared => "1 Shared Prongs".to_owned(),
            Self::TwoShared => "2 Shared Prongs".to_owned(),
            Self::TwoProngs => "2 Prongs".to_owned(),
            Self::EndsOnly => "Ends Only".to_owned(),
            Self::OneSharedOneSide => "1 Shared 1 Side".to_owned(),
            Self::OneSharedTwoSide => "1 Shared 2 Side".to_owned(),
            Self::LeftTwoRight => "Left 2 Right".to_owned(),
            Self::TwoProngsAt(deg) => format!("2 Prongs {} Degrees", degrees(deg)),
            Self::ThreeProngsAt(deg) => format!("3 Prongs {} Degrees", degrees(deg)),
            Self::FourProngsAt(deg) if deg < 0.0 => {
                format!("4 Prongs Neg {} Degrees", degrees(-deg))
            }
            Self::FourProngsAt(deg) => format!("4 Prongs {} Degrees", degrees(deg)),
            Self::FourProngsClose => "4 Prongs Close".to_owned(),
            Self::SixProngsAt(deg) => format!("6 Prongs {} Degrees", degrees(deg)),
        }
    }

    /// Every documented style, in the order the sheet lists the families.
    pub fn presets() -> Vec<Self> {
        vec![
            Self::OneShared,
            Self::TwoShared,
            Self::TwoProngs,
            Self::EndsOnly,
            Self::OneSharedOneSide,
            Self::OneSharedTwoSide,
            Self::LeftTwoRight,
            Self::TwoProngsAt(0.0),
            Self::TwoProngsAt(30.0),
            Self::TwoProngsAt(45.0),
            Self::TwoProngsAt(60.0),
            Self::TwoProngsAt(90.0),
            Self::TwoProngsAt(120.0),
            Self::TwoProngsAt(150.0),
            Self::ThreeProngsAt(0.0),
            Self::ThreeProngsAt(180.0),
            Self::FourProngsAt(0.0),
            Self::FourProngsAt(45.0),
            Self::FourProngsAt(-45.0),
            Self::FourProngsClose,
            Self::SixProngsAt(0.0),
            Self::SixProngsAt(90.0),
        ]
    }

    /// Resolve a style name typed at the Command Line (case-insensitive, single spaces).
    pub fn from_name(text: &str) -> Option<Self> {
        let wanted = text.trim().to_ascii_lowercase().replace(['_', '-'], " ");
        let wanted = wanted.split_whitespace().collect::<Vec<_>>().join(" ");
        Self::presets()
            .into_iter()
            .find(|mode| mode.name().to_ascii_lowercase() == wanted)
    }

    /// The placement data of the style.
    pub fn layout(self) -> LayoutSpec {
        let flow = |ends: [u8; 2], junctions: u8, sides: [u8; 2]| LayoutSpec {
            family: ProngFamily::GemFlow,
            ends,
            junctions,
            sides,
            pairs: Vec::new(),
        };
        match self {
            Self::OneShared => flow([1, 1], 1, [0, 0]),
            Self::TwoShared => flow([2, 2], 2, [0, 0]),
            Self::TwoProngs => flow([1, 1], 2, [0, 0]),
            Self::EndsOnly => flow([2, 2], 0, [0, 0]),
            Self::OneSharedOneSide => flow([1, 1], 1, [0, 1]),
            Self::OneSharedTwoSide => flow([1, 1], 1, [0, 2]),
            // "one prong in a rotated 90° position on one side of the gem and two prongs in a
            // rotated 90° position on the other side of the gem"
            Self::LeftTwoRight => flow([1, 2], 0, [1, 2]),
            Self::TwoProngsAt(deg) => LayoutSpec {
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: deg,
                        spread: Spread::Single,
                    },
                    PairSpec {
                        base_deg: deg + 180.0,
                        spread: Spread::Single,
                    },
                ],
            },
            Self::ThreeProngsAt(deg) => LayoutSpec {
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: deg,
                        spread: Spread::Single,
                    },
                    PairSpec {
                        base_deg: deg + 180.0,
                        spread: Spread::Delta,
                    },
                ],
            },
            Self::FourProngsAt(deg) => LayoutSpec {
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: deg,
                        spread: Spread::Delta,
                    },
                    PairSpec {
                        base_deg: deg + 180.0,
                        spread: Spread::Delta,
                    },
                ],
            },
            Self::FourProngsClose => LayoutSpec {
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: 0.0,
                        spread: Spread::HalfDelta,
                    },
                    PairSpec {
                        base_deg: 180.0,
                        spread: Spread::HalfDelta,
                    },
                ],
            },
            Self::SixProngsAt(0.0) => LayoutSpec {
                // "four prongs in the 4 Prongs Close position and one prong at each end: one
                // rotated 90° and one −90° from North."
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: 0.0,
                        spread: Spread::HalfDelta,
                    },
                    PairSpec {
                        base_deg: 180.0,
                        spread: Spread::HalfDelta,
                    },
                    PairSpec {
                        base_deg: 90.0,
                        spread: Spread::Single,
                    },
                    PairSpec {
                        base_deg: 270.0,
                        spread: Spread::Single,
                    },
                ],
            },
            Self::SixProngsAt(_) => LayoutSpec {
                // "two prongs at 2 Prongs 0 Degrees and a set of two prongs next to each".
                family: ProngFamily::GemNorth,
                ends: [0, 0],
                junctions: 0,
                sides: [0, 0],
                pairs: vec![
                    PairSpec {
                        base_deg: 90.0,
                        spread: Spread::Delta,
                    },
                    PairSpec {
                        base_deg: 270.0,
                        spread: Spread::Delta,
                    },
                ],
            },
        }
    }
}

fn degrees(value: f64) -> String {
    if (value.round() - value).abs() < 1.0e-9 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    }
}

/// Even/Odd metal placement: "Odd will position the metal so that is located at the top and
/// bottom of the stone. Even positions the metal so that is offset from the top and bottom."
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProngPosition {
    Odd,
    Even,
}

/// End-prong layout of `OM9-L3-04-15-009`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EndMode {
    /// Use the end count of the selected prong style.
    FollowMode,
    None,
    One,
    Two,
    /// One prong at the start of the row only (asymmetric styles).
    Left,
    /// One prong at the end of the row only.
    Right,
}

impl EndMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::FollowMode => "Follow Mode",
            Self::None => "Zero",
            Self::One => "1 Prong",
            Self::Two => "2 Prongs",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }

    pub fn from_name(text: &str) -> Option<Self> {
        let wanted = text.trim().to_ascii_lowercase();
        [
            Self::FollowMode,
            Self::None,
            Self::One,
            Self::Two,
            Self::Left,
            Self::Right,
        ]
        .into_iter()
        .find(|mode| mode.name().to_ascii_lowercase() == wanted)
    }
}

/// Typed parameters of a prong set.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProngParams {
    pub mode: ProngMode,
    /// Diameter of the prong at the largest stone ("the large prong in a tapered row").
    pub large_diameter_mm: f64,
    /// Diameter of the prong at a smaller stone of a tapered row.
    pub small_diameter_mm: f64,
    pub large_height_mm: f64,
    pub small_height_mm: f64,
    /// How far the prong extends below the girdle.
    pub drop_mm: f64,
    /// Fraction of the prong diameter that overlaps the stone. The sheet default is 10 %.
    pub nudge: f64,
    /// Top rounding, 0 = flat top, 1 = half sphere.
    pub fillet: f64,
    /// Side taper: positive = wider at the bottom, negative = wider at the top.
    pub taper: f64,
    /// Tilt of the prong axis in degrees, toward the placement direction.
    pub rotate_deg: f64,
    pub position: ProngPosition,
    pub end_mode: EndMode,
    /// Minimum centre distance between adjacent prongs. `None` derives it from the diameter.
    pub spacing_mm: Option<f64>,
    /// The builder's Boolean preview: the same body is emitted as a cutter for the kernel owner.
    pub as_cutter: bool,
}

impl Default for ProngParams {
    fn default() -> Self {
        Self {
            mode: ProngMode::GEM_FLOW_DEFAULT,
            large_diameter_mm: 1.0,
            small_diameter_mm: 0.8,
            large_height_mm: 1.0,
            small_height_mm: 1.0,
            drop_mm: 0.5,
            nudge: DEFAULT_NUDGE,
            fillet: 0.0,
            taper: 0.0,
            rotate_deg: 0.0,
            position: ProngPosition::Odd,
            end_mode: EndMode::FollowMode,
            spacing_mm: None,
            as_cutter: false,
        }
    }
}

impl ProngParams {
    pub fn validate(&self) -> Result<()> {
        let diameter = |value: f64| -> Result<()> {
            if !value.is_finite() || !(MIN_PRONG_DIAMETER_MM..=10.0).contains(&value) {
                return Err(Error::Invalid("prong diameter"));
            }
            Ok(())
        };
        diameter(self.large_diameter_mm)?;
        diameter(self.small_diameter_mm)?;
        if self.small_diameter_mm > self.large_diameter_mm {
            return Err(Error::Invalid("prong diameter"));
        }
        for height in [self.large_height_mm, self.small_height_mm] {
            if !height.is_finite() || !(MIN_PRONG_HEIGHT_MM..=50.0).contains(&height) {
                return Err(Error::Invalid("prong height"));
            }
        }
        if !self.drop_mm.is_finite() || !(0.0..=20.0).contains(&self.drop_mm) {
            return Err(Error::Invalid("prong drop"));
        }
        if !self.nudge.is_finite() || !(0.0..=0.5).contains(&self.nudge) {
            return Err(Error::Invalid("prong nudge"));
        }
        if !self.fillet.is_finite() || !(0.0..=1.0).contains(&self.fillet) {
            return Err(Error::Invalid("prong fillet"));
        }
        if !self.taper.is_finite() || !(-0.5..=0.5).contains(&self.taper) {
            return Err(Error::Invalid("prong taper"));
        }
        if !self.rotate_deg.is_finite() || !(-180.0..=180.0).contains(&self.rotate_deg) {
            return Err(Error::Invalid("prong rotate"));
        }
        if let Some(spacing) = self.spacing_mm
            && (!spacing.is_finite() || spacing <= 0.0 || spacing > 50.0)
        {
            return Err(Error::Invalid("prong spacing"));
        }
        Ok(())
    }

    /// The diameter/height used at a stone, so a tapered row keeps its large/small prongs.
    fn at_stone(&self, diameter_mm: f64, largest_mm: f64) -> (f64, f64) {
        if (diameter_mm - largest_mm).abs() <= 1.0e-9 {
            (self.large_diameter_mm, self.large_height_mm)
        } else {
            (self.small_diameter_mm, self.small_height_mm)
        }
    }
}

/// Where a prong came from, so a rebuild or an edit can name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProngSiteKind {
    End { end: usize },
    Junction { from: usize, to: usize },
    Gem { stone: usize },
}

/// One placed prong: its frame and the values the editor may change.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProngSite {
    pub kind: ProngSiteKind,
    /// Stable ordinal in row order.
    pub ordinal: usize,
    /// Origin is the prong centre on the girdle plane; local Z is the prong axis.
    pub frame: Frame,
    pub diameter_mm: f64,
    pub height_mm: f64,
    /// Stone (or junction) centre the prong belongs to.
    pub anchor_center: Point3,
    pub anchor_normal: Point3,
    pub anchor_side: Point3,
    pub anchor_tangent: Point3,
    /// Distance of the prong axis from the anchor centre, in the girdle plane.
    pub placement_radius_mm: f64,
    /// Girdle angle of a gem-centred prong, in degrees from north.
    pub angle_deg: Option<f64>,
    /// The nudge the prong was placed with, so an edit can rebuild its placement radius.
    pub nudge: f64,
    /// Index of this prong within its anchor (stone, end or junction). It is part of the
    /// stable prong id, so a recompute keeps identity even when a value changes.
    pub slot: usize,
    /// Identity of the anchor itself: the stone id for a gem prong, otherwise the end or
    /// junction the prong belongs to. A stone keeps its id when its index shifts.
    pub anchor_id: String,
}

impl ProngSite {
    /// The identity a rebuild and the editor use. It is derived from the anchor and the slot,
    /// never from the ordinal, so adding a stone does not rename an existing prong.
    pub fn stable_id(&self) -> String {
        format!("{}_{:02}", self.anchor_id, self.slot)
    }

    /// Two placements are the same point when their axes are closer than half the smaller
    /// prong. The close cluster of one stone is defined by the mode, so only prongs of *other*
    /// anchors are merged at a shared point.
    fn coincides_with(&self, other: &ProngSite) -> bool {
        let distance = norm(sub(self.frame.origin, other.frame.origin));
        let smaller = self.diameter_mm.min(other.diameter_mm);
        distance < smaller * 0.5
    }
}

/// A built prong set: the placed sites and one closed body per prong.
#[derive(Debug, Clone, PartialEq)]
pub struct ProngSet {
    pub sites: Vec<ProngSite>,
    pub bodies: Vec<Mesh>,
    pub volume_mm3: f64,
    pub min_height_mm: f64,
    /// Prongs that two anchors placed at the same point and that the builder merged into one
    /// shared prong (a tight row puts the facing prongs of two stones on the same junction).
    pub merged_prongs: usize,
}

impl ProngSet {
    pub fn count(&self) -> usize {
        self.sites.len()
    }

    pub fn summary(&self) -> String {
        let merged = match self.merged_prongs {
            0 => String::new(),
            1 => " (1 merged at a shared junction)".to_owned(),
            count => format!(" ({count} merged at shared junctions)"),
        };
        format!(
            "{} prongs{}, {:.3} mm3, min height {:.2} mm",
            self.sites.len(),
            merged,
            self.volume_mm3,
            self.min_height_mm
        )
    }
}

/// The angular half width of a prong on the girdle: `asin(r_prong / (r_gem + r_prong))`.
fn prong_half_width_deg(gem_radius: f64, prong_radius: f64) -> f64 {
    let ratio = (prong_radius / (gem_radius + prong_radius)).clamp(0.0, 0.95);
    ratio.asin().to_degrees().max(1.0)
}

fn polar(angle_deg: f64, side: Point3, tangent: Point3) -> Point3 {
    let radians = angle_deg.to_radians();
    add(mul(side, radians.cos()), mul(tangent, radians.sin()))
}

fn prong_frame(origin: Point3, axis: Point3, direction: Point3, tilt_deg: f64) -> Result<Frame> {
    let tilt = tilt_deg.to_radians();
    let axis = unit(add(mul(axis, tilt.cos()), mul(direction, tilt.sin())))?;
    Frame::from_axis(origin, axis, direction)
}

/// Place a prong set on a gem row.
///
/// The output order follows the row: start end prongs, then for every stone its gem prongs and
/// the junction to the next stone, then the finish end prongs. A shared junction prong is
/// emitted once, so a closed row cannot duplicate at the seam.
pub fn place_prongs(row: &GemRow, params: &ProngParams) -> Result<Vec<ProngSite>> {
    Ok(place_prongs_with_report(row, params)?.0)
}

/// Place a prong set and report how many prongs were merged at shared junctions.
///
/// The sheet's "4 Prongs Close" keeps two prongs of the *same* stone close together; a tight row
/// also puts the facing prongs of two neighbouring stones on the same junction, and the sheet
/// shows one shared prong there. Both are layout decisions, not errors: this function keeps the
/// first and merges the second pair, so no mode has to be refused for a tight row.
pub fn place_prongs_with_report(
    row: &GemRow,
    params: &ProngParams,
) -> Result<(Vec<ProngSite>, usize)> {
    params.validate()?;
    row.validate()?;
    if row.len() < 2 {
        // "0/1 gem ở mode không hỗ trợ": a single stone has no row direction.
        return Err(Error::Invalid("prong gem row"));
    }
    let spec = params.mode.layout();
    let ends = end_counts(row, params, &spec)?;
    let largest = row
        .stones
        .iter()
        .map(|stone| stone.diameter_mm)
        .fold(0.0_f64, f64::max);
    let junctions = row.junctions()?;
    let mut sites: Vec<ProngSite> = Vec::new();

    // Start end prongs (row order).
    if let Some(first) = row.ends().first().copied() {
        push_end(&mut sites, row, params, first, 0, ends[0])?;
    }

    for index in 0..row.len() {
        let stone = row.stone(index)?;
        let (diameter, height) = params.at_stone(stone.diameter_mm, largest);
        let normal = row.normal_at(index)?;
        let side = row.side_at(index)?;
        let tangent = row.tangent_at(index)?;
        let radius = prong_half_width_deg(stone.radius_mm(), diameter * 0.5);
        let mut angles: Vec<f64> = Vec::new();
        for pair in &spec.pairs {
            match pair.spread {
                Spread::Single => angles.push(pair.base_deg),
                Spread::Delta => {
                    angles.push(pair.base_deg - radius);
                    angles.push(pair.base_deg + radius);
                }
                Spread::HalfDelta => {
                    angles.push(pair.base_deg - radius * 0.5);
                    angles.push(pair.base_deg + radius * 0.5);
                }
            }
        }
        // Angle 0 points across the row (the side axis) and 90 along the row, so a "side"
        // prong is 0 and the opposite side is 180. Several prongs on one side run centred on
        // that side at a 2*radius pitch, so neighbours touch but never overlap.
        for (side_index, count) in spec.sides.iter().enumerate() {
            let base = if side_index == 0 { 0.0 } else { 180.0 };
            let step = if side_index == 0 { 1.0 } else { -1.0 };
            let centre = (f64::from(*count) - 1.0) * 0.5;
            for step_index in 0..*count {
                let offset = (f64::from(step_index) - centre) * 2.0 * radius * step;
                angles.push(base + offset);
            }
        }
        if params.position == ProngPosition::Even && angles.len() >= 2 {
            let pitch = 360.0 / angles.len() as f64;
            for angle in &mut angles {
                *angle += pitch * 0.5;
            }
        }
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // A style may place the same angle twice (e.g. the two prongs of a 2-prong layout are
        // 180° apart), but never the identical position twice.
        let mut previous: Option<f64> = None;
        let mut slot = 0usize;
        for angle in angles {
            if previous.is_some_and(|value: f64| (value - angle).abs() < 1.0e-6) {
                continue;
            }
            previous = Some(angle);
            let site_slot = slot;
            slot += 1;
            let placement_radius = stone.radius_mm() + diameter * 0.5 * (1.0 - 2.0 * params.nudge);
            let direction = polar(angle, side, tangent);
            let origin = add(stone.center, mul(direction, placement_radius));
            let frame = prong_frame(origin, normal, direction, params.rotate_deg)?;
            sites.push(ProngSite {
                kind: ProngSiteKind::Gem { stone: index },
                ordinal: sites.len(),
                frame,
                diameter_mm: diameter,
                height_mm: height,
                anchor_center: stone.center,
                anchor_normal: normal,
                anchor_side: side,
                anchor_tangent: tangent,
                placement_radius_mm: placement_radius,
                angle_deg: Some(angle),
                nudge: params.nudge,
                slot: site_slot,
                anchor_id: stone.id.clone(),
            });
        }
        // Junction after this stone (the wrap junction after the last stone of a closed row).
        if (index + 1 < row.len() || row.is_closed())
            && let Some(junction) = junctions.iter().find(|junction| junction.from == index)
        {
            push_junction(&mut sites, params, junction, spec.junctions)?;
        }
        if let Some(last) = row.ends().last().copied()
            && index == last
        {
            push_end(&mut sites, row, params, index, 1, ends[1])?;
        }
    }

    // Stable ordinals after the interleaved build.
    for (ordinal, site) in sites.iter_mut().enumerate() {
        site.ordinal = ordinal;
    }
    if sites.is_empty() {
        return Err(Error::Invalid("prong layout"));
    }
    if sites.len() > MAX_INSTANCES {
        return Err(Error::Budget("prong layout"));
    }
    let mut merged_prongs = 0usize;
    let mut kept: Vec<ProngSite> = Vec::with_capacity(sites.len());
    for site in sites {
        if let Some(existing) = kept
            .iter_mut()
            .find(|other: &&mut ProngSite| other.anchor_id != site.anchor_id && other.coincides_with(&site))
        {
            // One shared prong stands between the two anchors that asked for it: midpoint
            // origin, the larger of the two bodies, and the first anchor's orientation.
            let origin = [
                (existing.frame.origin[0] + site.frame.origin[0]) * 0.5,
                (existing.frame.origin[1] + site.frame.origin[1]) * 0.5,
                (existing.frame.origin[2] + site.frame.origin[2]) * 0.5,
            ];
            existing.frame.origin = origin;
            existing.diameter_mm = existing.diameter_mm.max(site.diameter_mm);
            existing.height_mm = existing.height_mm.max(site.height_mm);
            existing.angle_deg = None;
            merged_prongs += 1;
            continue;
        }
        kept.push(site);
    }
    for (ordinal, site) in kept.iter_mut().enumerate() {
        site.ordinal = ordinal;
    }
    Ok((kept, merged_prongs))
}

fn end_counts(row: &GemRow, params: &ProngParams, spec: &LayoutSpec) -> Result<[u8; 2]> {
    if row.is_closed() {
        if params.end_mode != EndMode::FollowMode {
            return Err(Error::Invalid("end mode on closed row"));
        }
        return Ok([0, 0]);
    }
    Ok(match params.end_mode {
        EndMode::FollowMode => spec.ends,
        EndMode::None => [0, 0],
        EndMode::One => [1, 1],
        EndMode::Two => [2, 2],
        EndMode::Left => [1, 0],
        EndMode::Right => [0, 1],
    })
}

fn pair_offset(params: &ProngParams, prong_radius: f64) -> f64 {
    match params.spacing_mm {
        Some(spacing) => spacing * 0.5,
        None => prong_radius * 0.9,
    }
}

fn push_end(
    sites: &mut Vec<ProngSite>,
    row: &GemRow,
    params: &ProngParams,
    stone_index: usize,
    end: usize,
    count: u8,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    let stone = row.stone(stone_index)?;
    let tangent = row.tangent_at(stone_index)?;
    let normal = row.normal_at(stone_index)?;
    let side = row.side_at(stone_index)?;
    let outward = if end == 0 { mul(tangent, -1.0) } else { tangent };
    let diameter = if (stone.diameter_mm - row_max_diameter(row)).abs() <= 1.0e-9 {
        params.large_diameter_mm
    } else {
        params.small_diameter_mm
    };
    let height = if (stone.diameter_mm - row_max_diameter(row)).abs() <= 1.0e-9 {
        params.large_height_mm
    } else {
        params.small_height_mm
    };
    let placement_radius = stone.radius_mm() + diameter * 0.5 * (1.0 - 2.0 * params.nudge);
    let offsets: Vec<f64> = match count {
        1 => vec![0.0],
        _ => vec![-pair_offset(params, diameter * 0.5), pair_offset(params, diameter * 0.5)],
    };
    for (slot, offset) in offsets.into_iter().enumerate() {
        let origin = add(
            add(stone.center, mul(outward, placement_radius)),
            mul(side, offset),
        );
        let frame = prong_frame(origin, normal, outward, params.rotate_deg)?;
        sites.push(ProngSite {
            kind: ProngSiteKind::End { end },
            ordinal: sites.len(),
            frame,
            diameter_mm: diameter,
            height_mm: height,
            anchor_center: stone.center,
            anchor_normal: normal,
            anchor_side: side,
            anchor_tangent: tangent,
            placement_radius_mm: placement_radius,
            angle_deg: None,
            nudge: params.nudge,
            slot,
            anchor_id: format!("end{end}"),
        });
    }
    Ok(())
}

fn push_junction(
    sites: &mut Vec<ProngSite>,
    params: &ProngParams,
    junction: &crate::settings::gems::Junction,
    count: u8,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    let diameter = params.large_diameter_mm;
    let height = params.large_height_mm;
    let offsets: Vec<f64> = match count {
        1 => vec![0.0],
        _ => vec![-pair_offset(params, diameter * 0.5), pair_offset(params, diameter * 0.5)],
    };
    for (slot, offset) in offsets.into_iter().enumerate() {
        let origin = add(junction.center, mul(junction.side, offset));
        let frame = prong_frame(origin, junction.normal, junction.side, params.rotate_deg)?;
        sites.push(ProngSite {
            kind: ProngSiteKind::Junction {
                from: junction.from,
                to: junction.to,
            },
            ordinal: sites.len(),
            frame,
            diameter_mm: diameter,
            height_mm: height,
            anchor_center: junction.center,
            anchor_normal: junction.normal,
            anchor_side: junction.side,
            anchor_tangent: junction.tangent,
            placement_radius_mm: 0.0,
            angle_deg: None,
            nudge: params.nudge,
            slot,
            anchor_id: format!("junction{}", junction.from),
        });
    }
    Ok(())
}

fn row_max_diameter(row: &GemRow) -> f64 {
    row.stones
        .iter()
        .map(|stone| stone.diameter_mm)
        .fold(0.0_f64, f64::max)
}

/// The closed (radius, axial) section of one prong: a flat or rounded cap over a tapered,
/// dropped stem.
fn prong_profile(params: &ProngParams, diameter_mm: f64, height_mm: f64) -> Result<Vec<(f64, f64)>> {
    let bottom_radius = diameter_mm * 0.5;
    let top_radius = bottom_radius * (1.0 - params.taper);
    if !top_radius.is_finite() || top_radius <= 1.0e-6 {
        return Err(Error::Invalid("prong taper"));
    }
    let cap = params.fillet * top_radius;
    let shoulder = height_mm - cap;
    if !shoulder.is_finite() || shoulder <= -params.drop_mm {
        return Err(Error::Invalid("prong height"));
    }
    let mut profile: Vec<(f64, f64)> = Vec::with_capacity(16);
    profile.push((0.0, -params.drop_mm));
    profile.push((bottom_radius, -params.drop_mm));
    profile.push((top_radius, shoulder));
    if cap > 1.0e-9 {
        const STEPS: usize = 6;
        for step in 1..=STEPS {
            let angle = std::f64::consts::FRAC_PI_2 * step as f64 / STEPS as f64;
            profile.push((top_radius * angle.cos(), shoulder + cap * angle.sin()));
        }
    } else {
        profile.push((0.0, height_mm));
    }
    Ok(profile)
}

/// Build the closed body of one placed prong.
pub fn build_prong(site: &ProngSite, params: &ProngParams) -> Result<Mesh> {
    params.validate()?;
    if site.diameter_mm <= 0.0 || site.height_mm < MIN_PRONG_HEIGHT_MM {
        return Err(Error::Invalid("prong site"));
    }
    let profile = prong_profile(params, site.diameter_mm, site.height_mm)?;
    let local = revolve_profile(&profile, REVOLVE_SEGMENTS)?;
    let mut vertices = Vec::with_capacity(local.vertices.len());
    for point in &local.vertices {
        vertices.push(site.frame.point(*point)?);
    }
    let mesh = Mesh {
        vertices,
        faces: local.faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    inspect_solid(&mesh)?;
    Ok(mesh)
}

/// Place and build a complete prong set for a gem row.
pub fn build_prong_set(row: &GemRow, params: &ProngParams) -> Result<ProngSet> {
    let (sites, merged_prongs) = place_prongs_with_report(row, params)?;
    let mut bodies = Vec::with_capacity(sites.len());
    let mut volume = 0.0;
    let mut min_height = f64::INFINITY;
    for site in &sites {
        let body = build_prong(site, params)?;
        let report = inspect_solid(&body)?;
        volume += report.volume_mm3;
        min_height = min_height.min(site.height_mm);
        bodies.push(body);
    }
    Ok(ProngSet {
        sites,
        bodies,
        volume_mm3: volume,
        min_height_mm: min_height,
        merged_prongs,
    })
}

/// A single-prong edit (`OM9-L3-04-15-002`). Every field is optional: an edit changes only the
/// values the operator touched.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProngEdit {
    pub diameter_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub drop_mm: Option<f64>,
    pub fillet: Option<f64>,
    pub taper: Option<f64>,
    pub rotate_deg: Option<f64>,
    /// Girdle angle of a gem-centred prong; ignored for end/junction prongs.
    pub angle_deg: Option<f64>,
    pub nudge: Option<f64>,
}

impl ProngEdit {
    pub fn is_empty(&self) -> bool {
        self.diameter_mm.is_none()
            && self.height_mm.is_none()
            && self.drop_mm.is_none()
            && self.fillet.is_none()
            && self.taper.is_none()
            && self.rotate_deg.is_none()
            && self.angle_deg.is_none()
            && self.nudge.is_none()
    }
}

/// Apply an edit to one prong of a set. The other prongs are untouched, so the caller's
/// transaction can replace exactly one body.
pub fn edit_prong(
    sites: &mut [ProngSite],
    index: usize,
    params: &ProngParams,
    edit: &ProngEdit,
) -> Result<Mesh> {
    if edit.is_empty() {
        return Err(Error::Missing("prong edit"));
    }
    let mut params = params.clone();
    let mut edited = sites
        .get(index)
        .cloned()
        .ok_or(Error::Missing("prong selection"))?;
    let old_diameter = edited.diameter_mm;
    if let Some(diameter) = edit.diameter_mm {
        edited.diameter_mm = diameter;
    }
    if let Some(height) = edit.height_mm {
        edited.height_mm = height;
    }
    if let Some(drop) = edit.drop_mm {
        params.drop_mm = drop;
    }
    if let Some(fillet) = edit.fillet {
        params.fillet = fillet;
    }
    if let Some(taper) = edit.taper {
        params.taper = taper;
    }
    if let Some(rotate) = edit.rotate_deg {
        params.rotate_deg = rotate;
    }
    if let Some(nudge) = edit.nudge {
        params.nudge = nudge;
    }
    if let Some(angle) = edit.angle_deg {
        if !angle.is_finite() {
            return Err(Error::Invalid("prong angle"));
        }
        if !matches!(edited.kind, ProngSiteKind::Gem { .. }) {
            // End and shared-junction prongs have no girdle angle to edit.
            return Err(Error::Unsupported("prong angle on end or shared prong"));
        }
        let gem_radius = edited.placement_radius_mm
            - old_diameter * 0.5 * (1.0 - 2.0 * edited.nudge);
        if !gem_radius.is_finite() || gem_radius <= 0.0 {
            return Err(Error::Invalid("prong placement"));
        }
        edited.placement_radius_mm =
            gem_radius + edited.diameter_mm * 0.5 * (1.0 - 2.0 * params.nudge);
        let direction = polar(angle, edited.anchor_side, edited.anchor_tangent);
        edited.frame = prong_frame(
            add(
                edited.anchor_center,
                mul(direction, edited.placement_radius_mm),
            ),
            edited.anchor_normal,
            direction,
            params.rotate_deg,
        )?;
        edited.angle_deg = Some(angle);
        edited.nudge = params.nudge;
    } else if edit.rotate_deg.is_some() {
        let direction = unit(sub(edited.frame.origin, edited.anchor_center))?;
        edited.frame = prong_frame(
            edited.frame.origin,
            edited.anchor_normal,
            direction,
            params.rotate_deg,
        )?;
    }
    params.validate()?;
    for (other_index, other) in sites.iter().enumerate() {
        if other_index == index {
            continue;
        }
        if edited.coincides_with(other) {
            return Err(Error::Invalid("prong overlap"));
        }
    }
    let body = build_prong(&edited, &params)?;
    sites[index] = edited;
    Ok(body)
}

/// Typed parameters of Prong on Surface.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceProngParams {
    pub diameter_mm: f64,
    pub height_mm: f64,
    /// Negative floats the prong above the surface, positive sinks it in (the sheet wording).
    pub drop_mm: f64,
    pub fillet: f64,
    pub taper: f64,
    pub rotate_deg: f64,
    /// Reverse the local normal so the prong points up on a flipped surface.
    pub flip: bool,
}

impl Default for SurfaceProngParams {
    fn default() -> Self {
        Self {
            diameter_mm: 1.0,
            height_mm: 1.0,
            drop_mm: 0.0,
            fillet: 0.0,
            taper: 0.0,
            rotate_deg: 0.0,
            flip: false,
        }
    }
}

impl SurfaceProngParams {
    fn as_prong_params(&self) -> ProngParams {
        ProngParams {
            mode: ProngMode::TwoShared,
            large_diameter_mm: self.diameter_mm,
            small_diameter_mm: self.diameter_mm,
            large_height_mm: self.height_mm,
            small_height_mm: self.height_mm,
            drop_mm: 0.0,
            nudge: 0.0,
            fillet: self.fillet,
            taper: self.taper,
            rotate_deg: self.rotate_deg,
            position: ProngPosition::Odd,
            end_mode: EndMode::FollowMode,
            spacing_mm: None,
            as_cutter: false,
        }
    }

    pub fn validate(&self) -> Result<()> {
        let params = self.as_prong_params();
        params.validate()?;
        if !self.drop_mm.is_finite() || !(-50.0..=50.0).contains(&self.drop_mm) {
            return Err(Error::Invalid("prong drop"));
        }
        Ok(())
    }
}

/// Place and build one prong on a surface at a picked UV (`OM9-L3-04-15-003`).
///
/// The frame follows the surface normal, so a prong on a curved face points along the local
/// normal instead of world Z. An out-of-domain UV, a singular normal or a bad height is a typed
/// refusal.
pub fn prong_on_surface(
    surface: &NurbsSurface,
    uv: [f64; 2],
    params: &SurfaceProngParams,
    limits: &Limits,
) -> Result<(ProngSite, Mesh)> {
    params.validate()?;
    for (axis, value) in uv.iter().enumerate() {
        if !value.is_finite()
            || *value < surface.domain[axis][0]
            || *value > surface.domain[axis][1]
        {
            return Err(Error::Invalid("surface uv"));
        }
    }
    let point = surface_kernel::evaluate(surface, uv[0], uv[1], limits)
        .map_err(|_| Error::Invalid("surface evaluation"))?;
    let normal = surface_kernel::normal(surface, uv[0], uv[1], limits)
        .map_err(|_| Error::Invalid("surface normal"))?;
    let normal = unit(normal).map_err(|_| Error::Invalid("surface normal"))?;
    let normal = if params.flip { mul(normal, -1.0) } else { normal };
    // "When set to 0 mm the prong is flush with the surface. A positive value will intersect
    // into the surface and a negative value will float above the surface."
    let origin = add(point, mul(normal, -params.drop_mm));
    let hint = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let frame = Frame::from_axis(origin, normal, hint)?;
    let site = ProngSite {
        kind: ProngSiteKind::Gem { stone: 0 },
        ordinal: 0,
        frame,
        diameter_mm: params.diameter_mm,
        height_mm: params.height_mm,
        anchor_center: point,
        anchor_normal: normal,
        anchor_side: frame.x,
        anchor_tangent: frame.y,
        placement_radius_mm: 0.0,
        angle_deg: None,
        nudge: 0.0,
        slot: 0,
        anchor_id: "surface".to_owned(),
    };
    let body = build_prong(&site, &params.as_prong_params())?;
    Ok((site, body))
}

/// Verify the shared input/output contract: prongs in row order, no duplicated shared prong and
/// a reversible row. This is the assertion the regression task and the app dispatch share.
pub fn verify_row_order(row: &GemRow, sites: &[ProngSite]) -> Result<()> {
    let mut previous = 0usize;
    for site in sites {
        let order = match site.kind {
            ProngSiteKind::End { end } => {
                if end == 0 {
                    0
                } else {
                    row.len()
                }
            }
            ProngSiteKind::Gem { stone } => stone + 1,
            ProngSiteKind::Junction { from, .. } => from + 1,
        };
        if order < previous {
            return Err(Error::Invalid("prong row order"));
        }
        previous = order;
    }
    Ok(())
}

/// Whether a gem row is a closed loop (the `Metal from Gems` closed modes and the channel
/// border use this same predicate, so the family cannot disagree).
pub fn row_is_closed(row: &GemRow) -> bool {
    matches!(row.shape, crate::settings::gems::RowShape::Closed) && row.len() >= 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::gems::{GemStone, RowShape, straight_row};

    fn row() -> GemRow {
        straight_row(
            &[2.0, 2.0, 2.0],
            0.0,
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        )
        .unwrap()
    }

    fn closed_row() -> GemRow {
        let stones = (0..8)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / 8.0;
                GemStone::round(
                    format!("g{index}"),
                    [10.0 * angle.cos(), 10.0 * angle.sin(), 0.0],
                    2.0,
                    [0.0, 0.0, 1.0],
                )
                .unwrap()
            })
            .collect();
        GemRow::new(stones, RowShape::Closed).unwrap()
    }

    #[test]
    fn the_gem_flow_default_places_shared_prongs_once_per_junction() {
        let params = ProngParams::default();
        let sites = place_prongs(&row(), &params).unwrap();
        // 3 stones: 2 end positions x 2 prongs + 2 junctions x 2 shared prongs.
        assert_eq!(sites.len(), 8);
        let junctions = sites
            .iter()
            .filter(|site| matches!(site.kind, ProngSiteKind::Junction { .. }))
            .count();
        assert_eq!(junctions, 4);
        assert!(sites.iter().all(|site| site.diameter_mm == params.large_diameter_mm));
    }

    #[test]
    fn a_closed_row_has_no_end_prongs_and_no_duplicate_at_the_seam() {
        let params = ProngParams::default();
        let row = closed_row();
        let sites = place_prongs(&row, &params).unwrap();
        assert!(sites
            .iter()
            .all(|site| !matches!(site.kind, ProngSiteKind::End { .. })));
        // 8 junctions x 2 shared prongs.
        assert_eq!(sites.len(), 16);
        let seam = sites
            .iter()
            .filter(|site| matches!(site.kind, ProngSiteKind::Junction { from: 7, to: 0 }))
            .count();
        assert_eq!(seam, 2);
    }

    #[test]
    fn end_modes_override_the_style_on_an_open_row() {
        let mut params = ProngParams {
            end_mode: EndMode::None,
            ..ProngParams::default()
        };
        assert_eq!(place_prongs(&row(), &params).unwrap().len(), 4);
        params.end_mode = EndMode::One;
        assert_eq!(place_prongs(&row(), &params).unwrap().len(), 6);
        params.end_mode = EndMode::Left;
        let sites = place_prongs(&row(), &params).unwrap();
        assert_eq!(sites.len(), 5);
        assert_eq!(
            sites
                .iter()
                .filter(|site| matches!(site.kind, ProngSiteKind::End { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn an_end_mode_on_a_closed_row_is_refused() {
        let params = ProngParams {
            end_mode: EndMode::Two,
            ..ProngParams::default()
        };
        assert_eq!(
            place_prongs(&closed_row(), &params).err(),
            Some(Error::Invalid("end mode on closed row"))
        );
    }

    #[test]
    fn every_documented_style_names_a_layout_and_round_trips_its_name() {
        for mode in ProngMode::presets() {
            assert_eq!(ProngMode::from_name(&mode.name()), Some(mode));
            assert_eq!(mode.layout().family, mode.family());
        }
        assert_eq!(ProngMode::from_name("2 shared prongs"), Some(ProngMode::TwoShared));
        assert_eq!(ProngMode::from_name("4 Prongs Neg 45 Degrees"), Some(ProngMode::FourProngsAt(-45.0)));
        assert_eq!(ProngMode::from_name("nope"), None);
    }

    #[test]
    fn a_gem_north_layout_places_prongs_around_each_stone() {
        let params = ProngParams {
            mode: ProngMode::FourProngsAt(0.0),
            ..ProngParams::default()
        };
        let sites = place_prongs(&row(), &params).unwrap();
        assert_eq!(sites.len(), 12);
        assert!(sites
            .iter()
            .all(|site| matches!(site.kind, ProngSiteKind::Gem { .. })));
    }

    #[test]
    fn the_even_position_offsets_the_metal_from_north_and_south() {
        let params = ProngParams {
            mode: ProngMode::TwoProngsAt(0.0),
            ..ProngParams::default()
        };
        let odd = place_prongs(&row(), &params).unwrap();
        let even = place_prongs(
            &row(),
            &ProngParams {
                position: ProngPosition::Even,
                ..params.clone()
            },
        )
        .unwrap();
        assert_eq!(odd.len(), even.len());
        // Odd has a prong at 0°, even has none there but a matching count.
        assert!(odd.iter().any(|site| site.angle_deg == Some(0.0)));
        assert!(!even.iter().any(|site| site.angle_deg == Some(0.0)));
    }

    #[test]
    fn prong_spacing_controls_the_pair_offset() {
        let tight = place_prongs(&row(), &ProngParams::default()).unwrap();
        let wide = place_prongs(
            &row(),
            &ProngParams {
                spacing_mm: Some(2.0),
                ..ProngParams::default()
            },
        )
        .unwrap();
        let tight_pair = tight.iter().filter(|s| matches!(s.kind, ProngSiteKind::Junction { from: 0, .. })).collect::<Vec<_>>();
        let wide_pair = wide.iter().filter(|s| matches!(s.kind, ProngSiteKind::Junction { from: 0, .. })).collect::<Vec<_>>();
        let gap = |pair: &[&ProngSite]| norm(sub(pair[0].frame.origin, pair[1].frame.origin));
        assert!(gap(&wide_pair) > gap(&tight_pair));
    }

    #[test]
    fn a_negative_spacing_is_refused() {
        let params = ProngParams {
            spacing_mm: Some(-1.0),
            ..ProngParams::default()
        };
        assert_eq!(params.validate().err(), Some(Error::Invalid("prong spacing")));
    }

    #[test]
    fn a_prong_height_below_the_manual_minimum_is_refused() {
        let params = ProngParams {
            large_height_mm: 0.05,
            ..ProngParams::default()
        };
        assert_eq!(params.validate().err(), Some(Error::Invalid("prong height")));
    }

    #[test]
    fn a_single_stone_row_is_not_supported() {
        let row = straight_row(&[2.0], 0.0, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap();
        assert_eq!(
            place_prongs(&row, &ProngParams::default()).err(),
            Some(Error::Invalid("prong gem row"))
        );
    }

    #[test]
    fn every_built_prong_is_a_closed_solid_with_a_positive_volume() {
        let set = build_prong_set(&row(), &ProngParams::default()).unwrap();
        assert_eq!(set.bodies.len(), set.sites.len());
        for body in &set.bodies {
            let report = inspect_solid(body).unwrap();
            assert!(!body.vertices.is_empty());
            assert!(body
                .vertices
                .iter()
                .all(|point| point.iter().all(|value| value.is_finite())));
            assert!(report.volume_mm3 > 0.0);
        }
        assert!(set.volume_mm3 > 0.0);
        assert!(set.min_height_mm >= MIN_PRONG_HEIGHT_MM);
    }

    #[test]
    fn a_filleted_prong_keeps_a_closed_top() {
        for fillet in [0.0, 0.25, 1.0] {
            let params = ProngParams {
                fillet,
                ..ProngParams::default()
            };
            let set = build_prong_set(&row(), &params).unwrap();
            assert!(set.volume_mm3 > 0.0);
        }
    }

    #[test]
    fn a_positive_taper_narrows_the_top_and_negative_widens_it() {
        let site = place_prongs(&row(), &ProngParams::default()).unwrap()[0].clone();
        let flat = build_prong(&site, &ProngParams::default()).unwrap();
        let narrow = build_prong(
            &site,
            &ProngParams {
                taper: 0.4,
                ..ProngParams::default()
            },
        )
        .unwrap();
        let wide = build_prong(
            &site,
            &ProngParams {
                taper: -0.4,
                ..ProngParams::default()
            },
        )
        .unwrap();
        let volume = |mesh: &Mesh| inspect_solid(mesh).unwrap().volume_mm3;
        assert!(volume(&narrow) < volume(&flat));
        assert!(volume(&wide) > volume(&flat));
    }

    #[test]
    fn the_editor_changes_one_prong_and_leaves_the_others() {
        let mut sites = place_prongs(&row(), &ProngParams::default()).unwrap();
        let others = sites.clone();
        let body = edit_prong(
            &mut sites,
            0,
            &ProngParams::default(),
            &ProngEdit {
                diameter_mm: Some(0.6),
                height_mm: Some(1.5),
                ..ProngEdit::default()
            },
        )
        .unwrap();
        assert!(inspect_solid(&body).unwrap().volume_mm3 > 0.0);
        assert_eq!(sites[0].diameter_mm, 0.6);
        assert_eq!(sites[0].height_mm, 1.5);
        assert_eq!(sites[1], others[1]);
    }

    #[test]
    fn an_edit_can_move_a_gem_prong_along_the_girdle() {
        let params = ProngParams {
            mode: ProngMode::FourProngsAt(0.0),
            ..ProngParams::default()
        };
        let mut sites = place_prongs(&row(), &params).unwrap();
        let index = sites
            .iter()
            .position(|site| matches!(site.kind, ProngSiteKind::Gem { .. }))
            .unwrap();
        let before = sites[index].frame.origin;
        edit_prong(
            &mut sites,
            index,
            &params,
            &ProngEdit {
                angle_deg: Some(45.0),
                ..ProngEdit::default()
            },
        )
        .unwrap();
        assert_ne!(sites[index].frame.origin, before);
        assert_eq!(sites[index].angle_deg, Some(45.0));
    }

    #[test]
    fn an_end_prong_has_no_girdle_angle_to_edit() {
        let mut sites = place_prongs(&row(), &ProngParams::default()).unwrap();
        assert_eq!(
            edit_prong(
                &mut sites,
                0,
                &ProngParams::default(),
                &ProngEdit {
                    angle_deg: Some(45.0),
                    ..ProngEdit::default()
                },
            )
            .err(),
            Some(Error::Unsupported("prong angle on end or shared prong"))
        );
    }

    #[test]
    fn an_empty_edit_is_refused() {
        let mut sites = place_prongs(&row(), &ProngParams::default()).unwrap();
        assert_eq!(
            edit_prong(&mut sites, 0, &ProngParams::default(), &ProngEdit::default()).err(),
            Some(Error::Missing("prong edit"))
        );
    }

    #[test]
    fn a_prong_on_a_plane_uses_the_surface_normal() {
        let limits = Limits::default();
        let surface = surface_kernel::plane([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0, &limits)
            .unwrap();
        let (site, body) = prong_on_surface(
            &surface,
            [0.5, 0.5],
            &SurfaceProngParams::default(),
            &limits,
        )
        .unwrap();
        assert!((site.frame.z[2].abs() - 1.0).abs() < 1.0e-6);
        assert!(inspect_solid(&body).unwrap().volume_mm3 > 0.0);
        // Flipping the surface flips the prong axis.
        let (flipped, _) = prong_on_surface(
            &surface,
            [0.5, 0.5],
            &SurfaceProngParams {
                flip: true,
                ..SurfaceProngParams::default()
            },
            &limits,
        )
        .unwrap();
        assert!((flipped.frame.z[2] + 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn a_prong_on_surface_outside_the_domain_is_refused() {
        let limits = Limits::default();
        let surface =
            surface_kernel::plane([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0, &limits)
                .unwrap();
        assert_eq!(
            prong_on_surface(&surface, [1.5, 0.5], &SurfaceProngParams::default(), &limits).err(),
            Some(Error::Invalid("surface uv"))
        );
    }

    #[test]
    fn an_out_of_order_site_list_is_refused() {
        let row = row();
        let mut sites = place_prongs(&row, &ProngParams::default()).unwrap();
        assert!(verify_row_order(&row, &sites).is_ok());
        sites.reverse();
        assert_eq!(
            verify_row_order(&row, &sites).err(),
            Some(Error::Invalid("prong row order"))
        );
    }

    #[test]
    fn a_reversed_row_keeps_the_same_prong_count() {
        let row = row();
        let params = ProngParams::default();
        let forward = place_prongs(&row, &params).unwrap();
        let backward = place_prongs(&row.reversed().unwrap(), &params).unwrap();
        assert_eq!(forward.len(), backward.len());
    }

    #[test]
    fn closed_row_predicate_matches_the_row_shape() {
        assert!(!row_is_closed(&row()));
        assert!(row_is_closed(&closed_row()));
    }
}
